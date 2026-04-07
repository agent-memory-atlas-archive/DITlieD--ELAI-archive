//! Contract tests for BSP superstep engine (Phase 3D.2).
//!
//! These tests validate cross-module integration: engine + channels + SSB + checkpoint.

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::{SeededClock, SeededRng};
use elai_orchestrator::agent_pool::{
    AgentPoolConfig, AgentRole, InProcessAgentPool, ModelPool,
};
use elai_orchestrator::bsp::channels::{
    ChannelId, ChannelRegistry, InMemoryChannelRegistry, UntrackedValue,
    UNTRACKED_DEFAULT_LIMIT_BYTES,
};
use elai_orchestrator::bsp::engine::{
    BspCheckpoint, BspConfig, BspEngine, BspError, NodeSpec,
};
use elai_orchestrator::ssb::{Ssb, SsbActionType, SsbConfig, SsbPayload};

struct MockModelPool;
impl ModelPool for MockModelPool {
    fn available_vram(&self) -> u64 {
        u64::MAX
    }
}

fn test_pool() -> InProcessAgentPool<SeededRng> {
    InProcessAgentPool::new(
        AgentPoolConfig::default(),
        SeededRng::new(42),
        Arc::new(MockModelPool),
    )
}

async fn test_ssb() -> Ssb {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("contract_test.db");
    let db_path_owned = db_path.to_path_buf();
    std::mem::forget(dir);
    Ssb::new(
        SsbConfig {
            db_path: db_path_owned,
            project_id: "contract-project".into(),
            session_id: "contract-session".into(),
            pool_size: None,
        },
        Arc::new(SeededClock::new()),
    )
    .expect("ssb creation")
}

fn contract_config(checkpoint_dir: PathBuf) -> BspConfig {
    BspConfig {
        checkpoint_dir,
        session_id: "contract-session".into(),
        project_id: "contract-project".into(),
        ..Default::default()
    }
}

fn chain_nodes(count: usize) -> Vec<NodeSpec> {
    (0..count)
        .map(|i| NodeSpec {
            id: format!("agent_{i}"),
            role: AgentRole::Executor,
            task: format!("contract_task_{i}"),
            depends_on: if i > 0 {
                vec![format!("agent_{}", i - 1)]
            } else {
                vec![]
            },
            is_deferred: false,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Contract 1: Full integration — 2 agents, 3 supersteps
// ---------------------------------------------------------------------------

#[tokio::test]
async fn bsp_engine_runs_3_supersteps_with_2_agents() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let config = contract_config(tmp.path().join("checkpoints"));

    let pool = test_pool();
    let clock = SeededClock::new();
    let ssb = test_ssb().await;
    let mut channels = InMemoryChannelRegistry::new();

    // 3 layers of dependency: each layer has 1 node depending on the previous.
    // This creates 3 supersteps (one node completes per superstep).
    let nodes = chain_nodes(3);

    let mut engine = BspEngine::new(config.clone(), pool, clock);
    let result = engine.run(nodes, &mut channels, &ssb).await.expect("run");

    assert_eq!(
        result.supersteps_completed, 3,
        "3-node chain must produce 3 supersteps"
    );

    // Verify all 3 checkpoint files exist
    let ckpt_dir = config.checkpoint_dir;
    for i in 0..3u64 {
        let path = ckpt_dir.join(format!("superstep_{i}.msgpack"));
        assert!(
            tokio::fs::metadata(&path).await.is_ok(),
            "superstep_{i}.msgpack must exist"
        );

        let bytes = tokio::fs::read(&path).await.expect("read checkpoint");
        let ckpt: BspCheckpoint =
            rmp_serde::from_slice(&bytes).expect("deserialize checkpoint");
        assert_eq!(ckpt.superstep, i);
    }
}

// ---------------------------------------------------------------------------
// Contract 2: Determinism — same seed produces byte-identical checkpoints
// ---------------------------------------------------------------------------

#[tokio::test]
async fn determinism_contract() {
    async fn run_once(seed: u64) -> Vec<Vec<u8>> {
        let tmp = tempfile::tempdir().expect("tempdir");
        let ckpt_dir = tmp.path().join("checkpoints");
        let config = BspConfig {
            checkpoint_dir: ckpt_dir.clone(),
            session_id: "det-session".into(),
            project_id: "det-project".into(),
            ..Default::default()
        };

        let pool = InProcessAgentPool::new(
            AgentPoolConfig::default(),
            SeededRng::new(seed),
            Arc::new(MockModelPool),
        );
        let clock = SeededClock::new();
        let ssb_dir = tempfile::tempdir().expect("tempdir");
        let ssb = Ssb::new(
            SsbConfig {
                db_path: ssb_dir.path().join("det.db"),
                project_id: "det-project".into(),
                session_id: "det-session".into(),
                pool_size: None,
            },
            Arc::new(clock.clone()),
        )
        .expect("ssb");
        std::mem::forget(ssb_dir);

        let mut channels = InMemoryChannelRegistry::new();
        let nodes = chain_nodes(2);

        let mut engine = BspEngine::new(config, pool, clock);
        engine
            .run(nodes, &mut channels, &ssb)
            .await
            .expect("run");

        let mut checkpoint_bytes = Vec::new();
        for i in 0..2u64 {
            let path = ckpt_dir.join(format!("superstep_{i}.msgpack"));
            let bytes = tokio::fs::read(&path).await.expect("read");
            checkpoint_bytes.push(bytes);
        }
        checkpoint_bytes
    }

    let run1 = run_once(42).await;
    let run2 = run_once(42).await;

    assert_eq!(run1.len(), run2.len());
    for (i, (a, b)) in run1.iter().zip(run2.iter()).enumerate() {
        assert_eq!(
            a, b,
            "checkpoint {i} must be byte-identical across runs with same seed"
        );
    }
}

// ---------------------------------------------------------------------------
// Contract 3: Max superstep safety valve
// ---------------------------------------------------------------------------

#[tokio::test]
async fn max_superstep_safety_valve() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let mut config = contract_config(tmp.path().join("checkpoints"));
    config.max_supersteps = 2;

    let pool = test_pool();
    let clock = SeededClock::new();
    let ssb = test_ssb().await;
    let mut channels = InMemoryChannelRegistry::new();

    // 5-node chain: would need 5 supersteps, but limit is 2
    let nodes = chain_nodes(5);

    let mut engine = BspEngine::new(config, pool, clock);
    let result = engine.run(nodes, &mut channels, &ssb).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        BspError::MaxSuperstepsReached(limit) => {
            assert_eq!(limit, 2, "safety valve must trigger at configured limit");
        }
        other => panic!("expected MaxSuperstepsReached, got: {other}"),
    }
}

// ---------------------------------------------------------------------------
// Contract 4: Memory monitoring force-clear integration
// ---------------------------------------------------------------------------

#[tokio::test]
async fn memory_monitoring_force_clear_contract() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let config = contract_config(tmp.path().join("checkpoints"));

    let pool = test_pool();
    let clock = SeededClock::new();
    let ssb = test_ssb().await;
    let mut rx = ssb.subscribe();
    let mut channels = InMemoryChannelRegistry::new();

    // 25 MB Vec<u8> — exceeds force-clear threshold (10 MB * 2 = 20 MB)
    let id: ChannelId<UntrackedValue<Vec<u8>>> = ChannelId::new("contract_big");
    channels
        .write_untracked(&id, vec![0u8; 25 * 1024 * 1024])
        .expect("write untracked");

    let nodes = vec![NodeSpec {
        id: "agent_0".into(),
        role: AgentRole::Executor,
        task: "contract_task_0".into(),
        depends_on: vec![],
        is_deferred: false,
    }];

    let mut engine = BspEngine::new(config, pool, clock);
    engine.run(nodes, &mut channels, &ssb).await.expect("run");

    // Verify SSB contains MemoryMonitoring with action_taken="force_clear"
    let mut found = false;
    while let Ok(entry) = rx.try_recv() {
        if entry.action_type == SsbActionType::MemoryMonitoring {
            let payload: SsbPayload =
                rmp_serde::from_slice(&entry.payload).expect("deser");
            if let SsbPayload::MemoryMonitoring(mm) = payload {
                if mm.metric_name == "contract_big" {
                    assert_eq!(
                        mm.action_taken, "force_clear",
                        "25 MB channel must trigger force_clear"
                    );
                    assert!(
                        mm.value >= 25_000_000.0,
                        "reported size must be >= 25 MB, got {}",
                        mm.value
                    );
                    assert_eq!(
                        mm.threshold,
                        UNTRACKED_DEFAULT_LIMIT_BYTES as f64,
                        "threshold must match UNTRACKED_DEFAULT_LIMIT_BYTES"
                    );
                    found = true;
                }
            }
        }
    }
    assert!(
        found,
        "SSB must contain MemoryMonitoring force_clear event for contract_big"
    );

    // Channel must have been force-cleared
    let erased = ChannelId::<()>::new("contract_big");
    assert_eq!(
        channels.deep_size_untracked(&erased),
        0,
        "force-cleared channel must have size 0 after engine run"
    );
}
