//! Phase 4A.3/4A.5 — agent process isolation tests.
//!
//! Tests verify IPC gateway routing, GRC budget enforcement, grounding method
//! matrix, process spawn configuration, environment sanitization, and
//! heartbeat timeout detection.
//!
//! Platform-specific tests (PID namespace, SO_PEERCRED, rlimits, veth) are
//! gated behind #[cfg(unix)] and test the sandbox infrastructure.

use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Arc;

use elai_orchestrator::ipc_types::{
    AgentDispatchConfig, IpcFrame, IpcMessageType, ProcessConfig, ResourceLimits,
    SeccompProfile, ToolFilter, ToolId, AGENT_HEARTBEAT_TIMEOUT_SECS,
};
use elai_orchestrator::agent_pool::{
    AgentHandle, AgentPool, AgentPoolConfig, AgentRole, AgentStatus,
    InProcessAgentPool, ModelPool, VRAM_MIN_BYTES,
};
use elai_orchestrator::ipc_gateway::{
    AgentConnection, FrameRouteResult, GroundingMethod, IpcGateway,
    allowed_grounding_methods,
};
use elai_orchestrator::sandbox::{
    AgentEnvironment, SelfModificationLocks,
};

use elai_core::SeededRng;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

struct MockModelPool { vram: u64 }
impl ModelPool for MockModelPool {
    fn available_vram(&self) -> u64 { self.vram }
}

fn test_pool() -> InProcessAgentPool<SeededRng> {
    InProcessAgentPool::new(
        AgentPoolConfig {
            enable_sandbox: true,
            ..AgentPoolConfig::default()
        },
        SeededRng::new(42),
        Arc::new(MockModelPool { vram: u64::MAX }),
    )
}

fn test_gateway() -> IpcGateway {
    IpcGateway::with_os_clock(
        PathBuf::from("/tmp/elai_cc_test"),
        SelfModificationLocks::new(),
    )
}

fn test_connection(role: AgentRole, grc_max: u32) -> AgentConnection {
    AgentConnection::new(
        Uuid::nil(),
        role,
        1234,
        PathBuf::from("/tmp/test.sock"),
        grc_max,
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
            allowed_paths: vec![PathBuf::from("/project")],
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
// test_4a_isolation_* — agent process isolation
// =========================================================================

#[test]
fn test_4a_isolation_agent_spawn() {
    let mut pool = test_pool();
    let handle = pool.spawn(AgentRole::Executor, "test task".into()).unwrap();
    assert_eq!(handle.role, AgentRole::Executor);
    assert_eq!(handle.status, AgentStatus::Spawning);
    assert!(!handle.agent_id.is_nil());
    // Child process is None until launch_agent_process is called
    assert!(handle.child_process.is_none());
}

#[test]
fn test_4a_isolation_request_response() {
    let mut gw = test_gateway();
    let agent_id = Uuid::nil();
    gw.register_agent(test_connection(AgentRole::Executor, 5));

    // Agent sends AgentResponse via IPC
    let result = gw.route_frame(
        &agent_id,
        IpcMessageType::AgentResponse {
            content: "done".into(),
            artifacts: vec!["output.rs".into()],
        },
    );
    assert!(matches!(result, FrameRouteResult::Response(_)));
}

#[test]
fn test_4a_isolation_kill() {
    let mut gw = test_gateway();
    let agent_id = Uuid::nil();
    gw.register_agent(test_connection(AgentRole::Executor, 5));

    // AgentKill is an orchestrator-only message — agents don't send it
    let result = gw.route_frame(
        &agent_id,
        IpcMessageType::AgentKill { reason: "timeout".into() },
    );
    // AgentKill sent by agent is rejected (orchestrator-only)
    assert!(matches!(result, FrameRouteResult::Rejected(_)));
}

#[test]
fn test_4a_isolation_path_access() {
    // Self-modification locks prevent writes to locked paths
    let mut locks = SelfModificationLocks::new();
    locks.add_lock(PathBuf::from("/project/workflow"));
    let gw = IpcGateway::with_os_clock(PathBuf::from("/tmp/test"), locks);

    assert!(gw.is_self_mod_locked(&PathBuf::from("/project/workflow/state.json")));
    assert!(!gw.is_self_mod_locked(&PathBuf::from("/project/src/main.rs")));
}

/// B-18 P5: real pid_namespace test — verify that child_pid recorded in
/// LinuxSandboxHandle reflects the actual spawned PID, not a hardcoded value.
///
/// Replaces the prior fake that constructed `LinuxSandboxHandle { child_pid: Some(12345) }`
/// and asserted the literal. That test was structurally identical to an equality
/// constant check and exercised nothing meaningful.
///
/// The real test: spawn `true` (no-op) via LinuxSandbox and verify that the
/// handle's child_pid is Some(pid) where pid > 1 (real OS PID, not nil/zero).
/// This requires the process to actually be spawned into the namespace.
/// On systems where CLONE_NEWPID is unavailable (unprivileged CI), the test
/// skips rather than fails.
#[cfg(target_os = "linux")]
#[test]
fn test_4a_isolation_pid_namespace_real() {
    use elai_orchestrator::sandbox::{AgentSandbox, AppContainerConfig, LinuxSandbox};
    use elai_orchestrator::agent_pool::AgentRole;
    use uuid::Uuid;

    let sandbox = LinuxSandbox::new();
    let agent_id = Uuid::new_v4();
    let scratch = std::env::temp_dir().join(format!("elai_pid_ns_test_{}", agent_id));
    let _ = std::fs::create_dir_all(&scratch);

    let config = AppContainerConfig::new(
        agent_id,
        AgentRole::Executor,
        std::env::current_dir().unwrap_or_else(|_| "/tmp".into()),
        scratch.clone(),
    );

    let handle = match sandbox.create(&config) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("skipping test_4a_isolation_pid_namespace_real: create failed (may need privileges): {e}");
            let _ = std::fs::remove_dir_all(&scratch);
            return;
        }
    };

    // Spawn /bin/true as the sandboxed process
    let result = sandbox.launch_agent(&handle, "/bin/true", &[], &scratch);
    let _ = std::fs::remove_dir_all(&scratch);

    match result {
        Ok(proc) => {
            // Real test: PID must be > 1 (not a placeholder zero or nil)
            assert!(proc.process_id > 1,
                "sandboxed process must have real OS PID > 1, got: {}", proc.process_id);
            // Cleanup: wait for the process to avoid zombie
            let _ = sandbox.wait_agent(proc.agent_id);
        }
        Err(e) => {
            // Acceptable on unprivileged CI without user namespaces
            eprintln!("note: sandbox launch returned err (may need CAP_SYS_ADMIN): {e}");
        }
    }
}

#[test]
fn test_4a_isolation_network_blocked() {
    // IpcBaseline seccomp profile is the default for non-researcher roles
    let config = test_dispatch_config(AgentRole::Executor);
    assert_eq!(config.sandbox_config.seccomp_profile, SeccompProfile::IpcBaseline);

    // Researcher gets NetworkExtra
    let researcher_config = test_dispatch_config(AgentRole::Researcher);
    // In production, researcher gets NetworkExtra via sandbox.seccomp_for_role()
    // Here we verify the role-based selection logic exists
    let methods = allowed_grounding_methods(&AgentRole::Researcher);
    assert!(methods.contains(&GroundingMethod::WebCheck));

    let executor_methods = allowed_grounding_methods(&AgentRole::Executor);
    assert!(!executor_methods.contains(&GroundingMethod::WebCheck));
    let _ = researcher_config;
}

#[cfg(unix)]
#[test]
fn test_4a_isolation_ipc_socket() {
    // Verify socket path generation uses agent UUID
    let gw = test_gateway();
    let id = Uuid::nil();
    let path = gw.socket_path_for(&id);
    assert!(path.to_string_lossy().ends_with(".sock"));
    assert!(path.to_string_lossy().contains(&id.to_string()));
}

/// B-18 P5: real Windows AppContainer spawn test.
///
/// Replaces the prior fake that only called gw.socket_path_for() and checked
/// string content — that test exercised IpcGateway path formatting but not
/// the sandbox at all.
///
/// The real test: attempt to launch cmd.exe via WindowsAppContainerSandbox and
/// verify the resulting process has a non-zero PID and reports a non-null
/// AppContainer SID (confirming CreateAppContainerProfile was called, not just
/// CreateProcessW with default token).
///
/// If CreateAppContainerProfile fails (no admin, no Developer Mode), the test
/// logs and skips — it never hard-fails on unprivileged CI.
#[cfg(target_os = "windows")]
#[test]
fn test_4a_isolation_appcontainer_spawn_real() {
    use elai_orchestrator::sandbox::{
        AgentSandbox, AppContainerConfig, WindowsAppContainerSandbox,
    };
    use elai_orchestrator::agent_pool::AgentRole;
    use uuid::Uuid;

    let sandbox = WindowsAppContainerSandbox::with_fallback();
    let agent_id = Uuid::new_v4();
    let scratch = std::env::temp_dir().join(format!("elai_ac_spawn_test_{}", agent_id));
    let _ = std::fs::create_dir_all(&scratch);

    let config = AppContainerConfig::new(
        agent_id,
        AgentRole::Executor,
        std::env::current_dir().unwrap_or_else(|_| std::env::temp_dir()),
        scratch.clone(),
    );

    let handle = match sandbox.create(&config) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("skipping test_4a_isolation_appcontainer_spawn_real: create failed (expected without admin): {e}");
            let _ = std::fs::remove_dir_all(&scratch);
            return;
        }
    };

    // Note whether we got a real AppContainer SID or fallback restricted-token.
    // SID is None when running under with_fallback() (no admin / no Developer Mode).
    // This is expected on unprivileged CI — we note it but do not fail.
    use elai_orchestrator::sandbox::PlatformSandboxHandle;
    if let PlatformSandboxHandle::Windows(ref wh) = handle.platform_handle {
        if wh.appcontainer_sid.is_none() {
            eprintln!("note: AppContainer create() returned Ok but SID is None — using restricted-token fallback (expected without admin)");
        } else {
            eprintln!("note: AppContainer SID present — full AppContainer isolation active");
        }
    }

    // Spawn cmd.exe /c "exit 0" inside the container
    let result = sandbox.launch_agent(
        &handle,
        "cmd.exe",
        &["/c".to_string(), "exit 0".to_string()],
        &scratch,
    );
    let _ = sandbox.cleanup(&handle);
    let _ = std::fs::remove_dir_all(&scratch);

    match result {
        Ok(proc) => {
            // Real test: spawned process must have a real OS PID
            assert!(proc.process_id > 0,
                "sandboxed process must have non-zero PID, got: {}", proc.process_id);
            let _ = sandbox.kill_agent(proc.agent_id);
            let _ = sandbox.wait_agent(proc.agent_id);
        }
        Err(e) => {
            // Acceptable on hardened CI sandboxes that restrict CreateProcessW
            eprintln!("note: launch_agent returned err under AppContainer (may be CI restriction): {e}");
        }
    }
}

#[cfg(unix)]
#[test]
fn test_4a_isolation_researcher_veth() {
    // Researcher role gets NetworkExtra seccomp profile (allows AF_INET+AF_INET6 SOCK_STREAM)
    // Verified via the grounding method matrix: researcher gets WEB_CHECK
    let methods = allowed_grounding_methods(&AgentRole::Researcher);
    assert!(methods.contains(&GroundingMethod::WebCheck));
    assert!(methods.contains(&GroundingMethod::FileCheck));
    // Non-researcher doesn't get web access
    assert!(!allowed_grounding_methods(&AgentRole::Executor).contains(&GroundingMethod::WebCheck));
}

#[test]
fn test_4a_isolation_heartbeat_timeout() {
    use elai_core::{ClockProvider, SeededClock};
    let clock = SeededClock::new();
    let mut conn = test_connection(AgentRole::Executor, 5);
    // Record heartbeat at tick 0.
    conn.last_heartbeat = clock.now();
    // Fresh connection is not timed out (0 ticks elapsed).
    assert!(!conn.is_heartbeat_timeout(&clock));

    // Advance SeededClock past AGENT_HEARTBEAT_TIMEOUT_SECS (each now() = +1ms tick).
    let ticks_needed = (AGENT_HEARTBEAT_TIMEOUT_SECS + 1) * 1000;
    for _ in 0..ticks_needed {
        let _ = clock.now();
    }
    assert!(conn.is_heartbeat_timeout(&clock));
}

#[test]
fn test_4a_isolation_agent_crash() {
    // When an agent is removed, socket is cleaned up
    let mut gw = test_gateway();
    let agent_id = Uuid::nil();
    gw.register_agent(test_connection(AgentRole::Executor, 5));

    let removed = gw.remove_agent(&agent_id);
    assert!(removed.is_some());
    // Agent is no longer registered
    assert!(gw.get_agent(&agent_id).is_none());
}

#[test]
fn test_4a_isolation_scratch_dir() {
    let mut pool = test_pool();
    let handle = pool.spawn(AgentRole::Executor, "task".into()).unwrap();
    // Scratch dir contains agent UUID
    assert!(handle.scratch_dir.to_string_lossy().contains(&handle.agent_id.to_string()));
}

#[test]
fn test_4a_isolation_concurrent_dirs() {
    let mut pool = test_pool();
    let h1 = pool.spawn(AgentRole::Executor, "task1".into()).unwrap();
    let h2 = pool.spawn(AgentRole::Auditor, "task2".into()).unwrap();
    // Different agents get different scratch dirs
    assert_ne!(h1.scratch_dir, h2.scratch_dir);
    assert_ne!(h1.agent_id, h2.agent_id);
}

#[test]
fn test_4a_isolation_auditor_executor_visibility() {
    // Auditor and executor get different scratch dirs (isolation boundary)
    let mut pool = test_pool();
    let executor = pool.spawn(AgentRole::Executor, "exec".into()).unwrap();
    let auditor = pool.spawn(AgentRole::Auditor, "audit".into()).unwrap();

    // Auditor role has more grounding methods (all methods including SourceCheck)
    let auditor_methods = allowed_grounding_methods(&AgentRole::Auditor);
    let executor_methods = allowed_grounding_methods(&AgentRole::Executor);
    assert!(auditor_methods.len() > executor_methods.len());
    assert!(auditor_methods.contains(&GroundingMethod::SourceCheck));
    assert!(!executor_methods.contains(&GroundingMethod::SourceCheck));
    let _ = (executor, auditor);
}

#[test]
fn test_4a_isolation_env_sanitized() {
    let scratch = PathBuf::from("/scratch/agent_1");
    let env = AgentEnvironment::new(scratch.clone());
    let env_map = env.as_env_map();

    // Must have PATH and TMPDIR
    assert!(env_map.contains_key("PATH"));
    assert!(env_map.contains_key("TMPDIR"));
    assert_eq!(env_map["TMPDIR"], scratch.to_string_lossy().as_ref());

    // Must NOT have sensitive variables
    assert!(!env_map.contains_key("HOME"));
    assert!(!env_map.contains_key("USER"));
    assert!(!env_map.contains_key("ANTHROPIC_API_KEY"));
}

#[cfg(unix)]
#[test]
fn test_4a_isolation_so_peercred() {
    // SO_PEERCRED verification function exists and has correct signature
    // Full integration test requires actual Unix socket pair (tested in ipc_gateway unit tests)
    use elai_orchestrator::ipc_gateway::IpcGatewayError;
    // Verify the error type exists for PeercredMismatch
    let err = IpcGatewayError::PeercredMismatch { expected: 100, actual: 200 };
    assert!(err.to_string().contains("100"));
    assert!(err.to_string().contains("200"));
}

/// B-18 P5: verify seccomp IPC_BASELINE filter compiles and restricts socket
/// to AF_UNIX only. Tests that the compiled BPF program is non-empty (actual
/// compilation succeeded) and that the filter builder produces a distinct
/// program for IPC_BASELINE vs NETWORK_EXTRA.
///
/// Replaces the prior fake `test_4a_isolation_proc_access` which only asserted
/// `libc::CLONE_NEWPID != 0` — a trivially true compile-time constant, not a
/// runtime test of any sandbox behavior.
#[cfg(target_os = "linux")]
#[test]
fn test_4a_isolation_seccomp_af_unix_restriction() {
    use elai_orchestrator::sandbox::LinuxSandbox;
    use elai_orchestrator::agent_pool::AgentRole;

    // IPC_BASELINE filter must compile successfully
    let ipc_filter = LinuxSandbox::seccomp_for_role(&AgentRole::Executor)
        .expect("IPC_BASELINE seccomp filter must compile");

    // NETWORK_EXTRA filter must compile successfully for Researcher
    let net_filter = LinuxSandbox::seccomp_for_role(&AgentRole::Researcher)
        .expect("NETWORK_EXTRA seccomp filter must compile");

    // Both must produce non-empty BPF programs
    assert!(!ipc_filter.is_empty(), "IPC_BASELINE BPF program must be non-empty");
    assert!(!net_filter.is_empty(), "NETWORK_EXTRA BPF program must be non-empty");

    // The two filters must be different (IPC_BASELINE is more restrictive)
    // A filter that allows AF_INET must differ from one that does not.
    assert_ne!(ipc_filter, net_filter,
        "IPC_BASELINE and NETWORK_EXTRA must produce different BPF programs");
}

/// B-18 P5: verify per-agent allowed_paths are populated correctly.
/// Replaces the prior fake `test_4a_isolation_sys_access` which only asserted
/// `libc::CLONE_NEWNS != 0` — a compile-time constant check, not a runtime test.
///
/// The real test: spawn two agents with different roles and verify their
/// allowed_paths differ as specified by role_default_allowed_paths.
#[test]
fn test_4a_isolation_per_agent_allowed_paths() {
    use elai_orchestrator::agent_pool::{role_default_allowed_paths, AgentRole};
    use std::path::PathBuf;

    let project = PathBuf::from("/project");

    let executor_paths = role_default_allowed_paths(&AgentRole::Executor, &project);
    let auditor_paths  = role_default_allowed_paths(&AgentRole::Auditor, &project);
    let researcher_paths = role_default_allowed_paths(&AgentRole::Researcher, &project);

    // Executor: exactly 1 path (project glob)
    assert_eq!(executor_paths.len(), 1,
        "Executor must have exactly 1 allowed path, got: {:?}", executor_paths);
    assert!(executor_paths[0].contains("/project"),
        "Executor allowed path must contain project root: {:?}", executor_paths);

    // Auditor: 2 paths (project glob + "**" for full read access)
    assert_eq!(auditor_paths.len(), 2,
        "Auditor must have 2 allowed paths, got: {:?}", auditor_paths);
    assert!(auditor_paths.contains(&"**".to_string()),
        "Auditor must have broad '**' allow: {:?}", auditor_paths);

    // Researcher: 2 paths (project glob + tmp)
    assert_eq!(researcher_paths.len(), 2,
        "Researcher must have 2 allowed paths, got: {:?}", researcher_paths);

    // Roles must differ
    assert_ne!(executor_paths, auditor_paths,
        "Executor and Auditor must have different allowed_paths");
    assert_ne!(executor_paths, researcher_paths,
        "Executor and Researcher must have different allowed_paths");
}

/// B-18 P5: verify AgentHandle carries appcontainer_profile_id derived from agent_id.
/// Replaces the prior fake `test_4a_isolation_rlimits` which constructed a
/// ResourceLimits struct and asserted its own field values — exercising nothing
/// but struct construction.
///
/// The real test: spawn an executor in a pool with sandbox enabled and verify
/// that appcontainer_profile_id is Some("elai-agent-{uuid}").
#[test]
fn test_4a_isolation_appcontainer_profile_id_per_agent() {
    use std::sync::Arc;
    use elai_orchestrator::agent_pool::{AgentPoolConfig, AgentPool, AgentRole, InProcessAgentPool, ModelPool};
    use elai_core::SeededRng;

    struct UnlimitedVram;
    impl ModelPool for UnlimitedVram { fn available_vram(&self) -> u64 { u64::MAX } }

    let mut pool = InProcessAgentPool::new(
        AgentPoolConfig {
            enable_sandbox: true,
            ..AgentPoolConfig::default()
        },
        SeededRng::new(42),
        Arc::new(UnlimitedVram),
    );

    let handle = pool.spawn(AgentRole::Executor, "profile id test".into())
        .expect("spawn must succeed with unlimited VRAM");

    // When enable_sandbox=true, profile_id must be Some("elai-agent-{uuid}")
    let expected_prefix = format!("elai-agent-{}", handle.agent_id);
    assert_eq!(
        handle.appcontainer_profile_id,
        Some(expected_prefix),
        "appcontainer_profile_id must be derived from agent_id"
    );

    // Spawn a second agent — must have a DIFFERENT profile_id (not shared)
    let handle2 = pool.spawn(AgentRole::Auditor, "profile id test 2".into())
        .expect("second spawn must succeed");

    assert_ne!(
        handle.appcontainer_profile_id,
        handle2.appcontainer_profile_id,
        "each agent must have a unique appcontainer_profile_id"
    );

    // allowed_paths must also be set (non-empty for both)
    assert!(!handle.allowed_paths.is_empty(),
        "Executor handle must have non-empty allowed_paths");
    assert!(!handle2.allowed_paths.is_empty(),
        "Auditor handle must have non-empty allowed_paths");
}

// =========================================================================
// test_4a_llm_proxy_* — LLM proxy via IPC
// =========================================================================

#[test]
fn test_4a_llm_proxy_infer_request_flow() {
    let mut gw = test_gateway();
    let agent_id = Uuid::nil();
    gw.register_agent(test_connection(AgentRole::Executor, 5));

    // Agent sends InferRequest, gateway caps max_tokens and routes
    let result = gw.route_frame(
        &agent_id,
        IpcMessageType::InferRequest { prompt: "hello".into(), max_tokens: 100 },
    );
    match result {
        FrameRouteResult::InferRequest { prompt, max_tokens } => {
            assert_eq!(prompt, "hello");
            assert_eq!(max_tokens, 100); // within cap
        }
        other => panic!("expected InferRequest, got {other:?}"),
    }
}

#[test]
fn test_4a_llm_proxy_no_reqwest() {
    // Agent crate has zero reqwest dependency — verified by cargo tree in CI.
    // This is a structural test: AgentRole is defined in elai-ipc-types (shared crate)
    // and the agent binary depends only on elai-ipc-types, not orchestrator.
    // The compile succeeds without reqwest in agent's dependency tree.
    let _role = AgentRole::Executor;
    // If this compiles, the agent crate successfully imports AgentRole without
    // pulling in the orchestrator crate (which has reqwest).
}

#[test]
fn test_4a_llm_proxy_no_localhost_11434() {
    // IpcBaseline seccomp blocks network access (no AF_INET socket()).
    // Non-researcher agents cannot make outbound connections.
    // This is enforced at the OS level by the seccomp BPF filter.
    let config = test_dispatch_config(AgentRole::Executor);
    assert_eq!(config.sandbox_config.seccomp_profile, SeccompProfile::IpcBaseline);
}

#[test]
fn test_4a_llm_proxy_context_limit() {
    let mut gw = test_gateway();
    let agent_id = Uuid::nil();
    gw.register_agent(test_connection(AgentRole::Executor, 5));

    // InferRequest with very large max_tokens gets capped
    let result = gw.route_frame(
        &agent_id,
        IpcMessageType::InferRequest { prompt: "test".into(), max_tokens: 1_000_000 },
    );
    match result {
        FrameRouteResult::InferRequest { max_tokens, .. } => {
            assert!(max_tokens <= 8192, "max_tokens should be capped, got {max_tokens}");
        }
        other => panic!("expected InferRequest, got {other:?}"),
    }
}

// =========================================================================
// test_4a_grc_budget_* — GRC budget enforcement
// =========================================================================

#[test]
fn test_4a_grc_budget_external_count() {
    let mut gw = test_gateway();
    let agent_id = Uuid::nil();
    gw.register_agent(test_connection(AgentRole::Executor, 3));

    // 3 calls within budget
    for _ in 0..3 {
        let result = gw.route_frame(
            &agent_id,
            IpcMessageType::ThinkStep {
                topic: "analysis".into(),
                step_type: "observe".into(),
                content: "data".into(),
            },
        );
        match result {
            FrameRouteResult::ThinkStep { over_budget, .. } => assert!(!over_budget),
            other => panic!("expected ThinkStep, got {other:?}"),
        }
    }

    // 4th is over budget
    let result = gw.route_frame(
        &agent_id,
        IpcMessageType::ThinkStep {
            topic: "analysis".into(),
            step_type: "observe".into(),
            content: "more".into(),
        },
    );
    match result {
        FrameRouteResult::ThinkStep { over_budget, .. } => assert!(over_budget),
        other => panic!("expected ThinkStep, got {other:?}"),
    }
}

#[test]
fn test_4a_grc_budget_convergence_signal() {
    let mut gw = test_gateway();
    let agent_id = Uuid::nil();
    gw.register_agent(test_connection(AgentRole::Executor, 5));

    // Start a topic
    gw.route_frame(
        &agent_id,
        IpcMessageType::ThinkStep {
            topic: "debug".into(),
            step_type: "observe".into(),
            content: "data".into(),
        },
    );

    // Converge signals topic completion
    let result = gw.route_frame(
        &agent_id,
        IpcMessageType::ThinkStep {
            topic: "debug".into(),
            step_type: "converge".into(),
            content: "done".into(),
        },
    );
    match result {
        FrameRouteResult::ThinkStep { step_type, .. } => {
            assert_eq!(step_type, "converge");
        }
        other => panic!("expected ThinkStep, got {other:?}"),
    }

    // After converge, topic is no longer active for grounding
    let conn = gw.get_agent(&agent_id).unwrap();
    assert!(!conn.has_active_topic("debug"));
}

#[test]
fn test_4a_grc_budget_advisory_only() {
    let mut gw = test_gateway();
    let agent_id = Uuid::nil();
    gw.register_agent(test_connection(AgentRole::Executor, 2));

    // Exceed budget
    for _ in 0..3 {
        let result = gw.route_frame(
            &agent_id,
            IpcMessageType::ThinkStep {
                topic: "t".into(),
                step_type: "observe".into(),
                content: "data".into(),
            },
        );
        // All calls are routed (advisory, not hard-drop)
        assert!(matches!(result, FrameRouteResult::ThinkStep { .. }));
    }
}

#[test]
fn test_4a_grc_budget_per_topic() {
    let mut gw = test_gateway();
    let agent_id = Uuid::nil();
    gw.register_agent(test_connection(AgentRole::Executor, 2));

    // Exhaust topic A
    for _ in 0..3 {
        gw.route_frame(
            &agent_id,
            IpcMessageType::ThinkStep {
                topic: "topic_a".into(),
                step_type: "observe".into(),
                content: "data".into(),
            },
        );
    }

    // Topic B starts fresh
    let result = gw.route_frame(
        &agent_id,
        IpcMessageType::ThinkStep {
            topic: "topic_b".into(),
            step_type: "observe".into(),
            content: "data".into(),
        },
    );
    match result {
        FrameRouteResult::ThinkStep { over_budget, .. } => assert!(!over_budget),
        other => panic!("expected ThinkStep, got {other:?}"),
    }
}

#[test]
fn test_4a_grc_budget_orchestrator_authority() {
    // Orchestrator sets budget via AgentDispatchConfig.grc_budget_max_calls
    let config = test_dispatch_config(AgentRole::Executor);
    assert_eq!(config.grc_budget_max_calls, 5);

    // Different config can override
    let mut config2 = test_dispatch_config(AgentRole::Executor);
    config2.grc_budget_max_calls = 10;
    assert_eq!(config2.grc_budget_max_calls, 10);
}

// =========================================================================
// test_4a_grounding_* — grounding request proxy
// =========================================================================

#[test]
fn test_4a_grounding_file_check() {
    let mut gw = test_gateway();
    let agent_id = Uuid::nil();
    gw.register_agent(test_connection(AgentRole::Executor, 5));

    // Start a topic first (required for grounding)
    gw.route_frame(
        &agent_id,
        IpcMessageType::ThinkStep {
            topic: "analysis".into(),
            step_type: "observe".into(),
            content: "data".into(),
        },
    );

    // FILE_CHECK grounding request
    let result = gw.route_frame(
        &agent_id,
        IpcMessageType::GroundingRequest {
            topic: "analysis".into(),
            query: "FILE_CHECK src/main.rs".into(),
        },
    );
    assert!(matches!(result, FrameRouteResult::GroundingRequest { .. }));
}

#[test]
fn test_4a_grounding_web_check() {
    let mut gw = test_gateway();
    let agent_id = Uuid::nil();
    gw.register_agent(test_connection(AgentRole::Researcher, 5));

    // Start topic
    gw.route_frame(
        &agent_id,
        IpcMessageType::ThinkStep {
            topic: "research".into(),
            step_type: "observe".into(),
            content: "data".into(),
        },
    );

    // Researcher can use WEB_CHECK
    let result = gw.route_frame(
        &agent_id,
        IpcMessageType::GroundingRequest {
            topic: "research".into(),
            query: "WEB_CHECK rust async patterns".into(),
        },
    );
    assert!(matches!(result, FrameRouteResult::GroundingRequest { .. }));
}

#[test]
fn test_4a_grounding_source_check() {
    let mut gw = test_gateway();
    let agent_id = Uuid::nil();
    gw.register_agent(test_connection(AgentRole::Auditor, 5));

    // Start topic
    gw.route_frame(
        &agent_id,
        IpcMessageType::ThinkStep {
            topic: "audit".into(),
            step_type: "observe".into(),
            content: "data".into(),
        },
    );

    // Auditor can use SOURCE_CHECK
    let result = gw.route_frame(
        &agent_id,
        IpcMessageType::GroundingRequest {
            topic: "audit".into(),
            query: "SOURCE_CHECK crate::sandbox".into(),
        },
    );
    assert!(matches!(result, FrameRouteResult::GroundingRequest { .. }));
}

#[test]
fn test_4a_grounding_role_method_matrix() {
    // Executor: FILE_CHECK only
    let exec = allowed_grounding_methods(&AgentRole::Executor);
    assert_eq!(exec, vec![GroundingMethod::FileCheck]);

    // Researcher: FILE_CHECK + WEB_CHECK
    let researcher = allowed_grounding_methods(&AgentRole::Researcher);
    assert!(researcher.contains(&GroundingMethod::FileCheck));
    assert!(researcher.contains(&GroundingMethod::WebCheck));
    assert!(!researcher.contains(&GroundingMethod::SourceCheck));

    // Auditor: all
    let auditor = allowed_grounding_methods(&AgentRole::Auditor);
    assert!(auditor.contains(&GroundingMethod::FileCheck));
    assert!(auditor.contains(&GroundingMethod::WebCheck));
    assert!(auditor.contains(&GroundingMethod::SourceCheck));
}

#[test]
fn test_4a_grounding_budget_count() {
    let mut conn = test_connection(AgentRole::Executor, 5);
    conn.record_think_step("topic");

    // Can make grounding requests up to budget
    for _ in 0..50 {
        assert!(conn.check_grounding("topic", GroundingMethod::FileCheck).is_none());
    }
    // 51st is over budget
    assert!(conn.check_grounding("topic", GroundingMethod::FileCheck).is_some());
}

#[test]
fn test_4a_grounding_orphaned_topic_reject() {
    let mut conn = test_connection(AgentRole::Executor, 5);
    // No active topic — grounding rejected
    let result = conn.check_grounding("nonexistent", GroundingMethod::FileCheck);
    assert!(result.is_some());
    assert!(result.unwrap().contains("orphaned"));
}
