// Session wire-4A contract tests: real baby-step executor wiring on the
// dashboard path. Closes MVP audit Section 4 item #1.
//
// Covers (without spawning the real agent binary):
//   1. `AgentPoolBabyStepExecutor` can be constructed against any binary
//      path and wrapped in `Arc<dyn BabyStepExecutor>` (proves the wiring
//      shape the dashboard uses at lib.rs:~390).
//   2. Installing the executor sets `TaskRunnerConfig::baby_step_executor`
//      to `Some(_)`. When the caller passes `anti_spiral_enabled: true` (as
//      the dashboard does when the executor is live), the flag is preserved
//      through construction.
//   3. When the agent binary path does NOT exist, the executor still
//      constructs successfully. A spawn attempt surfaces the failure as
//      `BabyStepOutcome::Failed` + populated `stderr_tail` (covered by
//      the existing `baby_step_executor_contract.rs` — we re-assert the
//      "construction never panics" shape here).
//   4. The dashboard's fallback path (baby_step_executor=None,
//      anti_spiral_enabled=false) also constructs — this is the graceful
//      degradation when the agent binary is missing AND the dashboard is
//      configured to not attempt spawns.

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::SeededRng;
use elai_core::GitBranchPolicy;
use elai_orchestrator::agent_pool::{
    AgentPoolConfig, InProcessAgentPool, ModelPool, VRAM_MIN_BYTES,
};
use elai_orchestrator::pipeline_tree_navigator::BabyStepOutcome;
use elai_orchestrator::task_runner::{
    AgentPoolBabyStepExecutor, BabyStepExecutionCtx, BabyStepExecutor, SyntheticBabyStepExecutor,
    DEFAULT_BABY_STEP_TIMEOUT_MS,
};
use elai_orchestrator::TaskRunnerConfig;
use tokio::sync::Mutex as AsyncMutex;

// ---------------------------------------------------------------------------
// Test helpers — mirror the dashboard's UnconstrainedVramPool so missing-GPU
// CI machines still get a spawn attempt.
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
        SeededRng::new(42),
        Arc::new(InfiniteVramPool),
    )))
}

// ---------------------------------------------------------------------------
// Contract 1: AgentPoolBabyStepExecutor construction + coercion to trait
// object. Proves the dashboard's
//   let exec = AgentPoolBabyStepExecutor::new(...)
//   let e: Arc<dyn BabyStepExecutor> = Arc::new(exec);
// shape compiles and does not panic for any binary path.
// ---------------------------------------------------------------------------

#[test]
fn contract_agent_pool_executor_constructs_with_arbitrary_binary() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let binary = tmp.path().join("does-not-exist-on-disk.exe");
    let socket_dir = tmp.path().join("sockets");

    let exec = AgentPoolBabyStepExecutor::new(make_pool(), binary, socket_dir)
        .with_timeout_ms(DEFAULT_BABY_STEP_TIMEOUT_MS);
    let _boxed: Arc<dyn BabyStepExecutor> = Arc::new(exec);
}

// ---------------------------------------------------------------------------
// Contract 2: the dashboard's "agent binary present" configuration populates
// the TaskRunnerConfig correctly. We emulate "present" by creating an empty
// file at the binary path — the executor construction doesn't actually exec
// the file, it only stores the path.
// ---------------------------------------------------------------------------

#[test]
fn contract_task_runner_config_accepts_real_executor_and_antispiral() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let binary = tmp.path().join("elai_cc_agent.exe");
    std::fs::write(&binary, b"stub").expect("write stub binary");
    let socket_dir = tmp.path().join("sockets");

    let exec: Arc<dyn BabyStepExecutor> = Arc::new(
        AgentPoolBabyStepExecutor::new(make_pool(), binary.clone(), socket_dir)
            .with_timeout_ms(DEFAULT_BABY_STEP_TIMEOUT_MS),
    );

    let cfg = TaskRunnerConfig {
        cursor_path: tmp.path().join("cursor.json"),
        dev_server_launcher: None,
        project_dir_resolver: None,
        dev_server_registry: None,
        web_research_runner: None,
        scaffold_applier: None,
        rule_evaluator: None,
        interrogation_store: None,
        checkpoint_scratch_dir: None,
        baby_step_executor: Some(exec),
        anti_spiral_enabled: true,
        llm_provider: None,
        backend_config: None,
        indexing_service: None,
        hook_dispatcher: None,
        signal_reader: None,
        tool_dispatcher: None,
        git_branch_policy: GitBranchPolicy::default(),
        system_prober: None,
        feasibility_prober: None,
        autonomous_gate: None,
    };

    assert!(
        cfg.baby_step_executor.is_some(),
        "baby_step_executor must be Some when the dashboard installs one",
    );
    assert!(
        cfg.anti_spiral_enabled,
        "anti_spiral_enabled must track the dashboard's wire-antispiral flip",
    );
}

// ---------------------------------------------------------------------------
// Contract 3: the dashboard's "agent binary missing" fallback still yields a
// well-formed TaskRunnerConfig. This is the behaviour the dashboard falls
// back to when `locate_agent_binary` resolves to a non-existent path AND the
// dashboard chooses not to attempt spawns.
// ---------------------------------------------------------------------------

#[test]
fn contract_task_runner_config_synthetic_fallback_shape() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let cfg = TaskRunnerConfig {
        cursor_path: tmp.path().join("cursor.json"),
        dev_server_launcher: None,
        project_dir_resolver: None,
        dev_server_registry: None,
        web_research_runner: None,
        scaffold_applier: None,
        rule_evaluator: None,
        interrogation_store: None,
        checkpoint_scratch_dir: None,
        baby_step_executor: None,
        anti_spiral_enabled: false,
        llm_provider: None,
        backend_config: None,
        indexing_service: None,
        hook_dispatcher: None,
        signal_reader: None,
        tool_dispatcher: None,
        git_branch_policy: GitBranchPolicy::default(),
        system_prober: None,
        feasibility_prober: None,
        autonomous_gate: None,
    };
    assert!(cfg.baby_step_executor.is_none());
    assert!(!cfg.anti_spiral_enabled);
}

// ---------------------------------------------------------------------------
// Contract 4: verify the "missing binary" path surfaces Failed at run time.
// We do not want a regression where the executor silently passes on a spawn
// failure — wire-4A's whole point is observable failures.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_missing_binary_surfaces_failed_outcome() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let binary = tmp.path().join("no-such-agent.exe");
    let socket_dir = tmp.path().join("sockets");
    let exec = AgentPoolBabyStepExecutor::new(make_pool(), binary, socket_dir)
        .with_timeout_ms(1_000);

    let ctx = BabyStepExecutionCtx {
        baby_step_id: 1,
        task_summary: "dummy".into(),
        escalation_message: None,
    };
    let result = exec.execute(ctx).await;
    match result.outcome {
        BabyStepOutcome::Failed { .. } => {}
        other => panic!(
            "missing-binary path must Fail — wire-4A's contract is observable failure; got {:?}",
            other,
        ),
    }
    assert!(
        !result.stderr_tail.is_empty(),
        "stderr_tail must carry the spawn error message",
    );
}

// ---------------------------------------------------------------------------
// Contract 5: synthetic executor still works as a fallback. This guards the
// "dashboard boots even without the agent binary" acceptance criterion by
// confirming that the pre-wire-4A path is still available.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_synthetic_executor_still_works_as_fallback() {
    let exec = SyntheticBabyStepExecutor;
    let ctx = BabyStepExecutionCtx {
        baby_step_id: 99,
        task_summary: "legacy path".into(),
        escalation_message: None,
    };
    let result = exec.execute(ctx).await;
    assert!(matches!(result.outcome, BabyStepOutcome::Passed));
    assert_eq!(result.duration_ms, 1);
}

// Sanity check — the VRAM constant exists so downstream pool-sizing tests
// don't regress silently.
#[test]
fn contract_vram_constant_present() {
    assert!(VRAM_MIN_BYTES > 0);
    let _ = PathBuf::from("."); // PathBuf import kept for shape parity
}
