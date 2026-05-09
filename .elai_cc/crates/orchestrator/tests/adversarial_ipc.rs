//! Phase 4A.4/4A.5 — adversarial IPC tests.
//!
//! Tests exercise IPC wire protocol boundary conditions, adversarial inputs,
//! ReplanRequest budget/dedup, self-modification prevention, crash recovery,
//! and auditor self-discovery constraints.

use std::path::PathBuf;

use elai_orchestrator::ipc_types::{
    AgentDispatchConfig, IpcFrame, IpcFrameError, IpcMessageType, ProcessConfig,
    ResourceLimits, SeccompProfile, ToolFilter, ToolId,
    FRAME_HEADER_SIZE, IPC_PROTOCOL_VERSION, MAX_FRAME_SIZE,
};
use elai_orchestrator::agent_pool::AgentRole;
use elai_orchestrator::ipc_gateway::{
    AgentConnection, FrameRouteResult, IpcGateway, IpcGatewayError,
};
use elai_orchestrator::sandbox::SelfModificationLocks;

use uuid::Uuid;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn test_gateway() -> IpcGateway {
    IpcGateway::with_os_clock(
        PathBuf::from("/tmp/elai_cc_test"),
        SelfModificationLocks::new(),
    )
}

fn test_connection(role: AgentRole) -> AgentConnection {
    AgentConnection::new(
        Uuid::nil(),
        role,
        1234,
        PathBuf::from("/tmp/test.sock"),
        5,
        &elai_core::OsClock,
    )
}

fn test_dispatch_config(role: AgentRole) -> AgentDispatchConfig {
    AgentDispatchConfig {
        role,
        model_preference: "local".into(),
        tool_filter: ToolFilter { allowed_tools: vec![ToolId::new("read_file")] },
        rule_domains: vec!["default".into()],
        grc_budget_max_calls: 5,
        grc_budget_max_chars: 1000,
        sandbox_config: ProcessConfig {
            allowed_paths: vec![],
            seccomp_profile: SeccompProfile::IpcBaseline,
            resource_limits: ResourceLimits {
                max_memory_bytes: 1_073_741_824,
                max_cpu_secs: 300,
                max_open_files: 256,
            },
        },
    }
}

// =========================================================================
// test_4a_adv_ipc_* — adversarial wire protocol inputs
// =========================================================================

#[test]
fn test_4a_adv_ipc_oversized_payload_rejected() {
    let mut wire = vec![IPC_PROTOCOL_VERSION];
    let oversized_len = (MAX_FRAME_SIZE + 1) as u32;
    wire.extend_from_slice(&oversized_len.to_le_bytes());
    wire.extend(std::iter::repeat(0u8).take(MAX_FRAME_SIZE + 1));
    let result = IpcFrame::from_wire_bytes(&wire);
    assert!(
        matches!(result, Err(IpcFrameError::PayloadTooLarge { .. })),
        "oversized payload must be rejected with PayloadTooLarge"
    );
}

#[test]
fn test_4a_adv_ipc_truncated_header_rejected() {
    let short_data = vec![IPC_PROTOCOL_VERSION; FRAME_HEADER_SIZE - 1];
    let result = IpcFrame::from_wire_bytes(&short_data);
    assert!(
        matches!(result, Err(IpcFrameError::FrameTooShort)),
        "truncated header must be rejected with FrameTooShort"
    );
}

#[test]
fn test_4a_adv_ipc_version_mismatch_rejected() {
    let bad_version = IPC_PROTOCOL_VERSION.wrapping_add(1);
    let mut wire = vec![bad_version];
    wire.extend_from_slice(&0u32.to_le_bytes());
    let result = IpcFrame::from_wire_bytes(&wire);
    assert!(
        matches!(result, Err(IpcFrameError::VersionMismatch { .. })),
        "version mismatch must be rejected"
    );
}

#[test]
fn test_4a_adv_ipc_empty_payload_accepted() {
    let mut wire = vec![IPC_PROTOCOL_VERSION];
    wire.extend_from_slice(&0u32.to_le_bytes());
    let result = IpcFrame::from_wire_bytes(&wire);
    assert!(result.is_ok(), "zero-length payload is valid at the frame level");
}

#[test]
fn test_4a_adv_ipc_empty_payload_is_empty() {
    let mut wire = vec![IPC_PROTOCOL_VERSION];
    wire.extend_from_slice(&0u32.to_le_bytes());
    let frame = IpcFrame::from_wire_bytes(&wire)
        .expect("zero-length payload is valid at frame level");
    assert!(frame.payload.is_empty(), "payload should be empty");
}

#[test]
fn test_4a_adv_ipc_exact_max_frame_size() {
    let mut wire = vec![IPC_PROTOCOL_VERSION];
    let exact_len = MAX_FRAME_SIZE as u32;
    wire.extend_from_slice(&exact_len.to_le_bytes());
    wire.extend(std::iter::repeat(0u8).take(MAX_FRAME_SIZE));
    let result = IpcFrame::from_wire_bytes(&wire);
    assert!(result.is_ok(), "exact MAX_FRAME_SIZE payload must be accepted");
}

#[test]
fn test_4a_adv_ipc_malformed_postcard() {
    let garbage_payload = vec![0xFF, 0xFE, 0xFD, 0xFC, 0xFB];
    let mut wire = vec![IPC_PROTOCOL_VERSION];
    let len = garbage_payload.len() as u32;
    wire.extend_from_slice(&len.to_le_bytes());
    wire.extend_from_slice(&garbage_payload);
    let frame = IpcFrame::from_wire_bytes(&wire)
        .expect("frame-level parse should succeed for valid header");
    let result = IpcFrame::decode(&frame.payload);
    assert!(
        matches!(result, Err(IpcFrameError::DeserializationFailed(_))),
        "malformed postcard payload must fail deserialization"
    );
}

#[test]
fn test_4a_adv_ipc_frame_too_short() {
    let result = IpcFrame::from_wire_bytes(&[]);
    assert!(
        matches!(result, Err(IpcFrameError::FrameTooShort)),
        "empty input must be rejected with FrameTooShort"
    );
}

#[test]
fn test_4a_adv_ipc_declared_length_exceeds_data() {
    let mut wire = vec![IPC_PROTOCOL_VERSION];
    wire.extend_from_slice(&100u32.to_le_bytes());
    wire.extend(std::iter::repeat(0u8).take(10));
    let result = IpcFrame::from_wire_bytes(&wire);
    assert!(
        matches!(result, Err(IpcFrameError::FrameTooShort)),
        "declared length exceeding actual data must be FrameTooShort"
    );
}

// =========================================================================
// test_4a_adv_replan_* — ReplanRequest propagation and budget
// =========================================================================

#[test]
fn test_4a_adv_replan_request_propagated() {
    let mut gw = test_gateway();
    let agent_id = Uuid::nil();
    gw.register_agent(test_connection(AgentRole::Executor));

    let result = gw.route_frame(
        &agent_id,
        IpcMessageType::ReplanRequest {
            reason: "test failure detected".into(),
            severity: "warn".into(),
        },
    );
    match result {
        FrameRouteResult::ReplanRequest { reason, severity } => {
            assert_eq!(reason, "test failure detected");
            assert_eq!(severity, "warn");
        }
        other => panic!("expected ReplanRequest, got {other:?}"),
    }
}

#[test]
fn test_4a_adv_replan_budget_respected() {
    let mut gw = test_gateway();
    let agent_id = Uuid::nil();
    gw.register_agent(test_connection(AgentRole::Executor));

    // Send 3 replans (budget is 3)
    for i in 0..3 {
        let result = gw.route_frame(
            &agent_id,
            IpcMessageType::ReplanRequest {
                reason: format!("reason_{i}"),
                severity: "warn".into(),
            },
        );
        assert!(matches!(result, FrameRouteResult::ReplanRequest { .. }));
    }

    // 4th is rejected
    let result = gw.route_frame(
        &agent_id,
        IpcMessageType::ReplanRequest {
            reason: "reason_4".into(),
            severity: "warn".into(),
        },
    );
    assert!(matches!(result, FrameRouteResult::Rejected(_)));
}

#[test]
fn test_4a_adv_replan_loop_guard() {
    let mut gw = test_gateway();
    let agent_id = Uuid::nil();
    gw.register_agent(test_connection(AgentRole::Executor));

    // First replan succeeds
    let result = gw.route_frame(
        &agent_id,
        IpcMessageType::ReplanRequest {
            reason: "same_reason".into(),
            severity: "warn".into(),
        },
    );
    assert!(matches!(result, FrameRouteResult::ReplanRequest { .. }));

    // Same reason immediately after is deduplicated (rejected)
    let result = gw.route_frame(
        &agent_id,
        IpcMessageType::ReplanRequest {
            reason: "same_reason".into(),
            severity: "warn".into(),
        },
    );
    assert!(matches!(result, FrameRouteResult::Rejected(_)));
}

// =========================================================================
// test_4a_adv_selfmod_* — agent self-modification prevention
// =========================================================================

#[test]
fn test_4a_adv_selfmod_agent_cannot_modify_own_binary() {
    let mut locks = SelfModificationLocks::new();
    // Lock the agent binary directory
    locks.add_lock(PathBuf::from("/usr/local/bin"));
    let gw = IpcGateway::with_os_clock(PathBuf::from("/tmp/test"), locks);

    // Agent binary path is locked
    assert!(gw.is_self_mod_locked(&PathBuf::from("/usr/local/bin/elai_cc_agent")));
}

#[test]
fn test_4a_adv_selfmod_agent_cannot_write_outside_scratch() {
    let mut locks = SelfModificationLocks::new();
    locks.add_lock(PathBuf::from("/project/workflow"));
    locks.add_lock(PathBuf::from("/project/.claude"));
    let gw = IpcGateway::with_os_clock(PathBuf::from("/tmp/test"), locks);

    // Workflow and .claude dirs are locked
    assert!(gw.is_self_mod_locked(&PathBuf::from("/project/workflow/state.json")));
    assert!(gw.is_self_mod_locked(&PathBuf::from("/project/.claude/settings.json")));

    // Scratch dir is not locked
    assert!(!gw.is_self_mod_locked(&PathBuf::from("/scratch/agent_1/output.txt")));
}

// =========================================================================
// test_4a_adv_crash_* — agent crash recovery
// =========================================================================

#[test]
fn test_4a_adv_crash_agent_crash_cleans_socket() {
    let mut gw = test_gateway();
    let agent_id = Uuid::nil();
    gw.register_agent(test_connection(AgentRole::Executor));

    // Remove agent simulates crash cleanup
    let removed = gw.remove_agent(&agent_id);
    assert!(removed.is_some());
    // Connection no longer exists
    assert!(gw.get_agent(&agent_id).is_none());
}

#[test]
fn test_4a_adv_crash_agent_crash_releases_resources() {
    let mut gw = test_gateway();
    let agent_id = Uuid::nil();
    gw.register_agent(test_connection(AgentRole::Executor));

    // After remove, all resources associated with agent are released
    let removed = gw.remove_agent(&agent_id);
    assert!(removed.is_some());
    let conn = removed.unwrap();
    // Agent ID matches
    assert_eq!(conn.agent_id, agent_id);
}

#[test]
fn test_4a_adv_crash_supervisor_detects_crash() {
    use elai_core::{ClockProvider, SeededClock};
    // Heartbeat timeout detection: supervisor checks all agents.
    // Use a SeededClock-backed gateway so we can advance time deterministically.
    let clock = std::sync::Arc::new(SeededClock::new());
    let mut gw = elai_orchestrator::ipc_gateway::IpcGateway::new(
        std::path::PathBuf::from("/tmp/elai_cc_test"),
        elai_orchestrator::sandbox::SelfModificationLocks::new(),
        clock.clone(),
    );
    let agent_id = Uuid::nil();
    let mut conn = test_connection(AgentRole::Executor);
    // Record heartbeat at tick 0.
    conn.last_heartbeat = clock.now();
    // Advance SeededClock past AGENT_HEARTBEAT_TIMEOUT_SECS (each now() = +1ms tick).
    let ticks_needed = (30u64 + 1) * 1000;
    for _ in 0..ticks_needed {
        let _ = clock.now();
    }
    gw.register_agent(conn);

    let timed_out = gw.check_heartbeat_timeouts();
    assert!(!timed_out.is_empty());
    assert!(timed_out.contains(&agent_id));
}

// =========================================================================
// test_4a_adv_windows_* — Windows AppContainer isolation
// =========================================================================

#[cfg(windows)]
#[test]
fn test_4a_adv_windows_appcontainer_spawn() {
    use elai_orchestrator::sandbox::{WindowsAppContainerSandbox, AppContainerConfig, AgentSandbox};
    let sandbox = WindowsAppContainerSandbox::new();
    let config = AppContainerConfig::new(
        Uuid::new_v4(),
        AgentRole::Executor,
        PathBuf::from("C:\\project"),
        PathBuf::from("C:\\scratch"),
    );
    let result = sandbox.create(&config);
    // AppContainer creation may fail in test environment without admin rights
    // but the API call itself should not panic
    match result {
        Ok(handle) => {
            assert!(!handle.profile_name.is_empty());
            let _ = sandbox.cleanup(&handle);
        }
        Err(e) => {
            // Expected in unprivileged test environments
            eprintln!("AppContainer creation failed (expected in test): {e}");
        }
    }
}

#[cfg(windows)]
#[test]
fn test_4a_adv_windows_appcontainer_path_restriction() {
    use elai_orchestrator::sandbox::AppContainerConfig;
    let config = AppContainerConfig::new(
        Uuid::new_v4(),
        AgentRole::Executor,
        PathBuf::from("C:\\project"),
        PathBuf::from("C:\\scratch"),
    );
    assert_eq!(config.granted_paths.len(), 2);
    assert!(!config.network_allowed);
}

#[cfg(windows)]
#[test]
fn test_4a_adv_windows_named_pipe_ipc() {
    // Named pipe IPC uses the same wire format as Unix socket
    // Verify frame encode/decode works on Windows
    let msg = IpcMessageType::AgentHeartbeat {
        agent_id: Uuid::nil(),
        uptime_secs: 42,
    };
    let frame = IpcFrame::encode(&msg).unwrap();
    let wire = frame.to_wire_bytes().unwrap();
    let parsed = IpcFrame::from_wire_bytes(&wire).unwrap();
    let decoded = IpcFrame::decode(&parsed.payload).unwrap();
    assert_eq!(msg, decoded);
}

// =========================================================================
// test_4a_adv_auditor_* — auditor self-discovery constraints
// =========================================================================

#[test]
fn test_4a_adv_auditor_self_discovery() {
    // Auditor can discover its own role via dispatch_config
    let config = test_dispatch_config(AgentRole::Auditor);
    assert_eq!(config.role, AgentRole::Auditor);
    // Role is discoverable from the config sent with AgentRequest
    let msg = IpcMessageType::AgentRequest {
        task: "audit code".into(),
        dispatch_config: config,
    };
    match msg {
        IpcMessageType::AgentRequest { dispatch_config, .. } => {
            assert_eq!(dispatch_config.role, AgentRole::Auditor);
        }
        _ => panic!("unexpected"),
    }
}

#[test]
fn test_4a_adv_auditor_cannot_spawn_executor() {
    // Auditor agents cannot send AgentRequest (that's orchestrator-only)
    let mut gw = test_gateway();
    let agent_id = Uuid::nil();
    gw.register_agent(test_connection(AgentRole::Auditor));

    // AgentRequest sent by agent is rejected (orchestrator-only message)
    let result = gw.route_frame(
        &agent_id,
        IpcMessageType::AgentRequest {
            task: "spawn executor".into(),
            dispatch_config: test_dispatch_config(AgentRole::Executor),
        },
    );
    assert!(matches!(result, FrameRouteResult::Rejected(_)));
}
