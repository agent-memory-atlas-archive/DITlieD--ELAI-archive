//! Contract: CheckpointManager must write 5 atomic files, record SSB metadata,
//! reject UntrackedValue channels, and round-trip AgentCheckpointData via msgpack.

use rustc_hash::FxHashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::UNIX_EPOCH;

use elai_core::{ClockProvider, SeededClock};
use elai_orchestrator::checkpoint::{
    AgentCheckpointData, ChannelKind, ChannelReference, CheckpointError, CheckpointManager,
    GrcHistoryEntry,
};
use elai_orchestrator::ssb::{Ssb, SsbConfig, SsbPayload};
use elai_orchestrator::{AgentHandle, AgentRole, AgentStatus};
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
        transition_overrides: FxHashMap::default(),
        require_scope_enumeration: false,
    }
}

fn test_handle(tmp_dir: &std::path::Path) -> AgentHandle {
    AgentHandle {
        agent_id: Uuid::nil(),
        role: AgentRole::Executor,
        status: AgentStatus::Running,
        task: "contract-test".into(),
        active_files: vec![],
        grc_config: test_grc_config(),
        scratch_dir: tmp_dir.to_path_buf(),
        grc_engine: BasicGrcEngine::default(),
        sandbox_handle: None,
        child_process: None,
        socket_path: None,
        appcontainer_profile_id: None,
        allowed_paths: vec![],
    }
}

fn test_data() -> AgentCheckpointData {
    AgentCheckpointData {
        agent_status: "running".into(),
        grc_history: vec![GrcHistoryEntry {
            step_type: "observe".into(),
            content: "contract observation".into(),
            grounding: "file:contract.rs:L1".into(),
            topic: "default".into(),
        }],
        active_files: vec![PathBuf::from("src/lib.rs")],
        tool_results: vec!["tool-result-1".into()],
        output_draft: "contract draft".into(),
        channel_references: vec![],
        schema_version: elai_orchestrator::checkpoint::LEGACY_CHECKPOINT_SCHEMA_VERSION,
    }
}

async fn make_ssb(clock: SeededClock) -> Ssb {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("contract-test.db");
    let db_path_owned = db_path.to_path_buf();
    std::mem::forget(dir);
    Ssb::new(
        SsbConfig {
            db_path: db_path_owned,
            project_id: "contract-project".into(),
            session_id: "contract-session".into(),
            pool_size: None,
        },
        Arc::new(clock),
    )
    .expect("ssb creation")
}

#[tokio::test]
async fn contract_write_roundtrip_via_ssb() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let handle = test_handle(tmp.path());
    let clock = SeededClock::new();
    let ssb = make_ssb(SeededClock::new()).await;
    let mut rx = ssb.subscribe();
    let mgr = CheckpointManager::new(clock);
    let data = test_data();

    mgr.write(&handle, &data, 10, &ssb).await.expect("write");

    // Verify SSB payload round-trip
    let entry = rx.recv().await.expect("entry");
    let payload: SsbPayload = rmp_serde::from_slice(&entry.payload).expect("deser");
    if let SsbPayload::Checkpoint(cp) = payload {
        assert_eq!(cp.agent_id, Uuid::nil().to_string());
        assert_eq!(cp.step_index, 10);
        assert_eq!(cp.file_sizes.len(), 5);
    } else {
        panic!("expected Checkpoint payload from SSB");
    }

    // Verify disk round-trip
    let read_back = CheckpointManager::<SeededClock>::read_latest(&handle)
        .await
        .expect("read_latest")
        .expect("should be Some");
    assert_eq!(read_back, data);
}

#[tokio::test]
async fn contract_untracked_value_rejected() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let handle = test_handle(tmp.path());
    let clock = SeededClock::new();
    let ssb = make_ssb(SeededClock::new()).await;
    let mgr = CheckpointManager::new(clock);

    let mut data = test_data();
    data.channel_references.push(ChannelReference {
        channel_id: "untracked-ch".into(),
        kind: ChannelKind::UntrackedValue,
    });

    let result = mgr.write(&handle, &data, 0, &ssb).await;
    assert!(matches!(
        result,
        Err(CheckpointError::UntrackedValueReference { .. })
    ));

    // Verify no files were written (validation happens before disk I/O)
    let ckpt_dir = tmp.path().join(handle.agent_id.to_string());
    assert!(
        !ckpt_dir.exists(),
        "checkpoint dir must not be created when validation fails"
    );
}

#[tokio::test]
async fn contract_clock_quarantine_enforced() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let handle = test_handle(tmp.path());
    let clock = SeededClock::new();
    // SeededClock epoch = UNIX_EPOCH + 1_700_000_000s
    let expected_ts = clock
        .system_now()
        .duration_since(UNIX_EPOCH)
        .expect("duration")
        .as_secs();

    let ssb = make_ssb(SeededClock::new()).await;
    let mgr = CheckpointManager::new(clock);
    mgr.write(&handle, &test_data(), 0, &ssb).await.expect("write");

    let ckpt_dir = tmp.path().join(handle.agent_id.to_string());
    let ts_str = std::fs::read_to_string(ckpt_dir.join("checkpoint_ts")).expect("read");
    let actual_ts: u64 = ts_str.parse().expect("parse");
    assert_eq!(
        actual_ts, expected_ts,
        "checkpoint_ts must use ClockProvider, not SystemTime::now()"
    );
}
