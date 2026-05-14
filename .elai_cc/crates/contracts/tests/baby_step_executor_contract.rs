// Session wire-4A contract tests.
//
// Exercises the [`BabyStepExecutor`] wiring surface on the public task_runner
// API. The production agent-pool path (`AgentPoolBabyStepExecutor`) is
// exercised against a missing-binary path so the test stays hermetic — we
// prove the wiring surfaces the failure via `BabyStepOutcome::Failed` with a
// non-empty `stderr_tail` instead of panicking.
//
// Covers:
//   1. `TaskRunnerConfig::new_minimal` leaves `baby_step_executor = None` so
//      the pre-wire-4A synthetic path stays the default.
//   2. `SyntheticBabyStepExecutor::execute` returns `Passed` with
//      `duration_ms == RUNNER_SYNTHETIC_DURATION_MS` — the legacy contract
//      that existing tests rely on.
//   3. `AgentPoolBabyStepExecutor::execute` surfaces a missing-binary spawn
//      failure as `Failed { reason }` with the error captured in
//      `stderr_tail`. The wiring is the contract — the binary build is a
//      separate concern.
//   4. `BabyStepExecutionResult::synthetic_passed` round-trips the legacy
//      values (Passed, 1ms, None exit_code, empty stderr_tail).
//   5. `BabyStepExecutor` is object-safe — trait objects are storable in
//      `Arc<dyn BabyStepExecutor>`.

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::SeededRng;
use elai_orchestrator::agent_pool::{
    AgentPoolConfig, InProcessAgentPool, ModelPool, VRAM_MIN_BYTES,
};
use elai_orchestrator::pipeline_tree_navigator::BabyStepOutcome;
use elai_orchestrator::task_runner::{
    AgentPoolBabyStepExecutor, BabyStepExecutionCtx, BabyStepExecutionResult, BabyStepExecutor,
    SyntheticBabyStepExecutor, DEFAULT_BABY_STEP_TIMEOUT_MS,
};
use elai_orchestrator::{TaskRunnerConfig, RUNNER_AGENT_ID};
use tokio::sync::Mutex as AsyncMutex;

struct InfiniteVramPool;
impl ModelPool for InfiniteVramPool {
    fn available_vram(&self) -> u64 {
        u64::MAX
    }
}

struct TightVramPool;
impl ModelPool for TightVramPool {
    fn available_vram(&self) -> u64 {
        VRAM_MIN_BYTES / 2
    }
}

fn make_pool(
    vram: Arc<dyn ModelPool>,
) -> Arc<AsyncMutex<InProcessAgentPool<SeededRng>>> {
    Arc::new(AsyncMutex::new(InProcessAgentPool::new(
        AgentPoolConfig::default(),
        SeededRng::new(42),
        vram,
    )))
}

// Contract 1: TaskRunnerConfig::new_minimal leaves baby_step_executor = None.
#[test]
fn contract_task_runner_config_executor_default_none() {
    let cfg = TaskRunnerConfig::new_minimal(PathBuf::from("cursor.json"));
    assert!(
        cfg.baby_step_executor.is_none(),
        "wire-4A must not flip the default opt-in state; legacy tests depend on the synthetic path"
    );
    assert!(!cfg.anti_spiral_enabled);
    // Sanity: the RUNNER_AGENT_ID constant is stable — contract-key for rows.
    assert_eq!(RUNNER_AGENT_ID, "task_runner");
}

// Contract 2: Synthetic executor returns Passed + synthetic duration.
#[tokio::test]
async fn contract_synthetic_executor_returns_passed_1ms() {
    let exec = SyntheticBabyStepExecutor;
    let ctx = BabyStepExecutionCtx {
        baby_step_id: 7,
        task_summary: "print hello".into(),
        escalation_message: None,
        mode_selection: None,
    };
    let result = exec.execute(ctx).await;
    assert!(matches!(result.outcome, BabyStepOutcome::Passed));
    assert_eq!(result.duration_ms, 1);
    assert_eq!(result.exit_code, None);
    assert!(result.stderr_tail.is_empty());
}

// Contract 3: Missing binary path surfaces as Failed with populated stderr_tail.
#[tokio::test]
async fn contract_agent_pool_executor_missing_binary_reports_failed() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let binary = tmp.path().join("does-not-exist.exe");
    let socket_dir = tmp.path().join("sockets");

    let pool = make_pool(Arc::new(InfiniteVramPool));
    let exec = AgentPoolBabyStepExecutor::new(pool, binary, socket_dir)
        .with_timeout_ms(5_000);

    let ctx = BabyStepExecutionCtx {
        baby_step_id: 1,
        task_summary: "noop".into(),
        escalation_message: None,
        mode_selection: None,
    };
    let result = exec.execute(ctx).await;
    assert!(matches!(result.outcome, BabyStepOutcome::Failed { .. }));
    assert!(
        !result.stderr_tail.is_empty(),
        "stderr_tail must carry the spawn error so the dashboard can surface it"
    );
}

// Contract 4: Insufficient VRAM surfaces as Failed — the wiring surfaces
// pool.spawn errors. Proves pool -> executor -> Finding path works even
// before the binary exists.
#[tokio::test]
async fn contract_agent_pool_executor_vram_error_reports_failed() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let binary = tmp.path().join("unused.exe");
    let socket_dir = tmp.path().join("sockets");

    let pool = make_pool(Arc::new(TightVramPool));
    let exec = AgentPoolBabyStepExecutor::new(pool, binary, socket_dir);

    let ctx = BabyStepExecutionCtx {
        baby_step_id: 2,
        task_summary: "vram-starved".into(),
        escalation_message: None,
        mode_selection: None,
    };
    let result = exec.execute(ctx).await;
    assert!(matches!(result.outcome, BabyStepOutcome::Failed { .. }));
    assert!(result.stderr_tail.contains("insufficient VRAM"));
    assert_eq!(result.exit_code, None);
}

// Contract 5: `synthetic_passed` matches the old hardcoded values.
#[test]
fn contract_synthetic_passed_matches_legacy_defaults() {
    let r = BabyStepExecutionResult::synthetic_passed();
    assert!(matches!(r.outcome, BabyStepOutcome::Passed));
    assert_eq!(r.duration_ms, 1);
    assert_eq!(r.exit_code, None);
    assert!(r.stderr_tail.is_empty());
}

// Contract 6: BabyStepExecutor is object-safe (trait object storage).
#[test]
fn contract_baby_step_executor_trait_object_safe() {
    let _boxed: Arc<dyn BabyStepExecutor> = Arc::new(SyntheticBabyStepExecutor);
    // Compile-time assertion: if this builds, the trait is object-safe.
    assert!(DEFAULT_BABY_STEP_TIMEOUT_MS > 0);
}
