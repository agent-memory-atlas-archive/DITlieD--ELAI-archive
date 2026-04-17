use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use elai_core::SeededClock;
use elai_orchestrator::agent_pool::{AgentHandle, AgentRole, AgentStatus};
use elai_orchestrator::checkpoint::{
    AgentCheckpointData, CheckpointManager, GrcHistoryEntry,
};
use elai_orchestrator::recovery::{
    AgentDeathClassification, RecoveryError, RecoveryManager,
};
use elai_orchestrator::ssb::{Ssb, SsbConfig};
use elai_reasoning::{AgentGrcConfig, BasicGrcEngine, GrcBudget, GrcState, GroundingLevel};
use uuid::Uuid;

fn test_grc_config() -> AgentGrcConfig {
    AgentGrcConfig {
        role: "executor".into(),
        budget: GrcBudget {
            max_calls: 5,
            max_chars: 1000,
            calls_used: 0,
            chars_used: 0,
        },
        required_steps: vec![GrcState::Observe, GrcState::PlanAction, GrcState::Verify],
        grounding_level: GroundingLevel::Uncertain,
        csq_required: false,
        allowed_methods: vec![],
        transition_overrides: HashMap::new(),
        require_scope_enumeration: false,
    }
}

fn test_handle(scratch_dir: &std::path::Path) -> AgentHandle {
    AgentHandle {
        agent_id: Uuid::from_bytes([
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x47, 0x08, 0x89, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f, 0x10,
        ]),
        role: AgentRole::Executor,
        status: AgentStatus::Running,
        task: "contract-test".into(),
        active_files: vec![],
        grc_config: test_grc_config(),
        scratch_dir: scratch_dir.to_path_buf(),
        grc_engine: BasicGrcEngine::default(),
        sandbox_handle: None,
        child_process: None,
        socket_path: None,
    }
}

async fn make_ssb() -> Ssb {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("contract.db");
    let db_path_owned = db_path.to_path_buf();
    std::mem::forget(dir);
    Ssb::new(
        SsbConfig {
            db_path: db_path_owned,
            project_id: "contract-test".into(),
            session_id: "contract-session".into(),
            pool_size: None,
        },
        Arc::new(SeededClock::new()),
    )
    .expect("ssb creation")
}

#[tokio::test]
async fn contract_recovered_handle_preserves_agent_id() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let handle = test_handle(tmp.path());
    let ssb = make_ssb().await;

    let mgr = RecoveryManager::new(SeededClock::new());
    let (recovered, _ctx) = mgr
        .recover_agent(&handle, AgentDeathClassification::Crash, 0, &ssb)
        .await
        .expect("recovery must succeed");

    assert_eq!(
        recovered.agent_id, handle.agent_id,
        "CONTRACT: recovered AgentHandle.agent_id must equal original"
    );
    assert_eq!(
        recovered.status,
        AgentStatus::Spawning,
        "CONTRACT: recovered status must be Spawning"
    );
}

#[tokio::test]
async fn contract_corrupt_checkpoint_is_fresh_start_not_error() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let handle = test_handle(tmp.path());
    let ssb = make_ssb().await;

    // Write a valid checkpoint then corrupt it
    let clock = SeededClock::new();
    let ckpt_mgr = CheckpointManager::new(clock);
    let data = AgentCheckpointData {
        agent_status: "running".into(),
        grc_history: vec![GrcHistoryEntry {
            step_type: "observe".into(),
            content: "test".into(),
            grounding: "file:test.rs".into(),
            topic: "default".into(),
        }],
        active_files: vec![PathBuf::from("src/main.rs")],
        tool_results: vec![],
        output_draft: String::new(),
        channel_references: vec![],
    };
    ckpt_mgr
        .write(&handle, &data, 1, &ssb)
        .await
        .expect("checkpoint write");

    // Corrupt the state file
    let ckpt_dir = tmp.path().join(handle.agent_id.to_string());
    std::fs::write(ckpt_dir.join("state.msgpack"), b"corrupt garbage")
        .expect("corrupt write");

    let mgr = RecoveryManager::new(SeededClock::new());
    let result = mgr
        .recover_agent(&handle, AgentDeathClassification::Crash, 1, &ssb)
        .await;

    assert!(
        result.is_ok(),
        "CONTRACT: corrupt checkpoint must not return Err, got: {:?}",
        result.err()
    );

    let (_recovered, ctx) = result.expect("ok");
    assert!(
        !ctx.had_checkpoint,
        "CONTRACT: corrupt checkpoint must be treated as no checkpoint"
    );
    assert!(
        ctx.degraded,
        "CONTRACT: corrupt checkpoint must set degraded=true"
    );
}

#[tokio::test]
async fn contract_budget_exceeded_returns_err_no_ssb_write() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let handle = test_handle(tmp.path());
    let ssb = make_ssb().await;
    let mut rx = ssb.subscribe();

    let mgr = RecoveryManager::new(SeededClock::new());
    let result = mgr
        .recover_agent(
            &handle,
            AgentDeathClassification::BudgetExceeded,
            5,
            &ssb,
        )
        .await;

    assert!(
        matches!(result, Err(RecoveryError::BudgetExceeded)),
        "CONTRACT: BudgetExceeded must return Err(RecoveryError::BudgetExceeded)"
    );

    assert!(
        rx.try_recv().is_err(),
        "CONTRACT: BudgetExceeded must produce zero SSB writes"
    );
}
