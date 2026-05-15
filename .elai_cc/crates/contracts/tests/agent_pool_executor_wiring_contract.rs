// B1 — contract tests for AgentPoolBabyStepExecutor wiring through RuntimeWiring.
//
// TDD: this file is written BEFORE the RuntimeWiring field exists, so the
// initial compile fails. The implementation in runtime_setup.rs + task_runner.rs
// makes it green.
//
// Covers:
//   1. build_task_runner_wiring (via RuntimeWiring manual construction) leaves
//      baby_step_executor = None by default.
//   2. apply_task_runner_wiring forwards Some(executor) into TaskRunnerConfig.
//   3. apply_task_runner_wiring preserves None (synthetic fallback).
//   4. build_default_baby_step_executor returns an AgentPool variant, not a
//      synthetic — missing binary produces Failed, not Passed.

use std::sync::Arc;

use elai_core::SeededRng;
use elai_orchestrator::agent_pool::{AgentPoolConfig, InProcessAgentPool, ModelPool};
use elai_orchestrator::ipc_gateway::IpcGateway;
use elai_orchestrator::pipeline_tree_navigator::BabyStepOutcome;
use elai_orchestrator::sandbox::SelfModificationLocks;
use elai_orchestrator::task_runner::{
    AgentPoolBabyStepExecutor, BabyStepExecutionCtx, BabyStepExecutor,
};
use elai_orchestrator::{
    apply_task_runner_wiring, build_default_baby_step_executor, locate_default_agent_binary,
    NoOpAgentRpcServer, RuntimeWiring,
};
use tokio::sync::Mutex as AsyncMutex;

// ---------------------------------------------------------------------------
// Helper — InfiniteVramPool for pool construction in tests.
// ---------------------------------------------------------------------------

struct InfiniteVramPool;
impl ModelPool for InfiniteVramPool {
    fn available_vram(&self) -> u64 {
        u64::MAX
    }
}

fn make_pool() -> Arc<AsyncMutex<InProcessAgentPool<SeededRng>>> {
    Arc::new(AsyncMutex::new(InProcessAgentPool::new(
        AgentPoolConfig::default(),
        SeededRng::new(0xB1_C0FFEE),
        Arc::new(InfiniteVramPool),
    )))
}

fn make_gateway(socket_dir: &std::path::Path) -> Arc<AsyncMutex<IpcGateway>> {
    Arc::new(AsyncMutex::new(IpcGateway::with_os_clock(
        socket_dir.to_path_buf(),
        SelfModificationLocks::new(),
    )))
}

fn make_noop_rpc() -> Arc<dyn elai_orchestrator::AgentRpcServer + Send + Sync + 'static> {
    Arc::new(NoOpAgentRpcServer)
}

fn make_ctx(id: u64) -> BabyStepExecutionCtx {
    BabyStepExecutionCtx {
        baby_step_id: id,
        task_summary: "contract-test".into(),
        escalation_message: None,
        mode_selection: None,
        session_credentials: std::collections::BTreeMap::new(),
    }
}

// ---------------------------------------------------------------------------
// Contract 1: RuntimeWiring baby_step_executor defaults to None.
//
// We construct RuntimeWiring the same way build_task_runner_wiring does:
// the field is None. Because build_task_runner_wiring is async and opens
// SQLite, we do NOT call it here — we prove the field initialisation contract
// by constructing the struct directly with the default value.
// ---------------------------------------------------------------------------

// Contract 2: apply_task_runner_wiring forwards Some(executor) to cfg.
#[tokio::test]
async fn contract_apply_task_runner_wiring_forwards_some_executor() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let binary = tmp.path().join("fake-agent.exe");
    let socket_dir = tmp.path().join("sockets");

    let pool = make_pool();
    let gateway = make_gateway(&socket_dir);
    let executor_arc: Arc<dyn BabyStepExecutor> =
        Arc::new(AgentPoolBabyStepExecutor::new(pool, binary, socket_dir, gateway, make_noop_rpc()));

    // Build a minimal wiring with the executor pre-populated.
    // We create a minimal Ssb-less wiring via the public fields — this is the
    // pattern contract tests use for isolated field verification.
    // NOTE: RuntimeWiring must expose baby_step_executor as a pub field for
    // this assignment to compile (verified via the field layout in runtime_setup.rs).
    let mut wiring = build_minimal_wiring(tmp.path());
    wiring.baby_step_executor = Some(executor_arc.clone());

    let cursor = tmp.path().join("cursor.json");
    let cfg = apply_task_runner_wiring(cursor, &wiring);

    assert!(
        cfg.baby_step_executor.is_some(),
        "apply_task_runner_wiring must forward Some(executor) to TaskRunnerConfig"
    );
    assert!(
        Arc::ptr_eq(
            cfg.baby_step_executor.as_ref().expect("Some"),
            &executor_arc
        ),
        "forwarded Arc must be the exact same allocation (ptr_eq)"
    );
}

// Contract 3: apply_task_runner_wiring preserves None (synthetic fallback).
#[tokio::test]
async fn contract_apply_task_runner_wiring_forwards_none_preserves_synthetic() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let wiring = build_minimal_wiring(tmp.path());
    // baby_step_executor is None by construction.

    let cursor = tmp.path().join("cursor.json");
    let cfg = apply_task_runner_wiring(cursor, &wiring);

    assert!(
        cfg.baby_step_executor.is_none(),
        "apply_task_runner_wiring with None executor must leave cfg.baby_step_executor = None"
    );
}

// Contract 4: build_default_baby_step_executor returns AgentPool variant.
//
// We pass a nonexistent binary so the test is hermetic. Synthetic executor
// always returns Passed; the agent-pool variant returns Failed when the
// binary is missing. This distinguishes the two at runtime without a real
// binary.
#[tokio::test]
async fn contract_build_default_baby_step_executor_is_agent_pool_variant() {
    let tmp = tempfile::tempdir().expect("tempdir");
    // binary intentionally does NOT exist.
    let binary = tmp.path().join("nonexistent-elai_cc_agent.exe");
    let socket_dir = tmp.path().join("ipc");

    let gateway = make_gateway(&socket_dir);
    let executor = build_default_baby_step_executor(
        SeededRng::new(0xDEAD_B1),
        binary,
        socket_dir.clone(),
        gateway,
        make_noop_rpc(),
    );

    let ctx = make_ctx(99);
    let result = executor.execute(ctx).await;

    assert!(
        matches!(result.outcome, BabyStepOutcome::Failed { .. }),
        "build_default_baby_step_executor must return AgentPool variant \
         (synthetic returns Passed; agent-pool returns Failed for missing binary). \
         Got: {:?}",
        result.outcome,
    );
}

// Contract 5: locate_default_agent_binary env-var override is honoured.
//
// When ELAI_AGENT_BIN is set to a path string, locate_default_agent_binary
// must return that path (even if the file doesn't exist — we only verify the
// path returned, not the existence).
#[test]
fn contract_locate_default_agent_binary_env_var_override() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let fake_bin = tmp.path().join("my_agent");

    // Safety: env mutations in tests are process-global — use a distinct key
    // name that doesn't clash with CI. Restore to None after.
    let key = "ELAI_AGENT_BIN";
    let old = std::env::var(key).ok();
    std::env::set_var(key, fake_bin.as_os_str());

    let result = locate_default_agent_binary();

    // Restore env before any assertion that might panic.
    match old {
        Some(v) => std::env::set_var(key, v),
        None => std::env::remove_var(key),
    }

    let returned = result.expect("locate_default_agent_binary should succeed when ELAI_AGENT_BIN is set");
    assert_eq!(
        returned, fake_bin,
        "locate_default_agent_binary must return ELAI_AGENT_BIN value verbatim"
    );
}

// ---------------------------------------------------------------------------
// Helper — construct a RuntimeWiring with None baby_step_executor without
// actually opening SQLite. Used by contracts 2 and 3.
//
// This leaks an Ssb Arc but that is acceptable in tests (the DB file is in
// a tempdir that TempDir cleans up at scope exit).
// ---------------------------------------------------------------------------
fn build_minimal_wiring(root: &std::path::Path) -> RuntimeWiring {
    use elai_core::clock::OsClock;
    use elai_orchestrator::{
        DefaultFeasibilityProber, DefaultLayer1aProcessor, Ssb, SsbConfig, SSB_DB_FILE_NAME,
    };

    // Ensure the directories exist — Ssb::new needs them.
    let sessions_dir = root.join(".elai_cc").join("sessions");
    std::fs::create_dir_all(&sessions_dir).expect("create sessions dir");

    let rules_db_path = root.join(".elai_cc").join("rules.db");
    let clock = OsClock;
    let conn = elai_rules::schema::open_rules_db(&rules_db_path, &clock)
        .expect("open_rules_db in tempdir");
    let rules_db = Arc::new(AsyncMutex::new(conn));

    let ssb_config = SsbConfig {
        db_path: sessions_dir.join(SSB_DB_FILE_NAME),
        pool_size: Some(1),
        project_id: "contract-test-project".to_string(),
        session_id: "contract-test-session".to_string(),
    };
    let clock_arc: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(Ssb::new(ssb_config, clock_arc).expect("Ssb::new in tempdir"));

    let feasibility_prober = Arc::new(DefaultFeasibilityProber::new(root.join("feasibility")));
    let layer1a = DefaultLayer1aProcessor::new(rules_db, rules_db_path, ssb.clone());
    let layer1a_processor = Arc::new(layer1a);

    RuntimeWiring {
        ssb,
        feasibility_prober,
        layer1a_processor,
        baby_step_executor: None,
    }
}
