// J-24 contract: preassess_observation_size (LEG2) + adaptive_reduce wiring (S6b).
//
// Verifies the pure classification function:
// 1. NONE when observation_tokens <= slot_remaining * alpha.
// 2. SELECTIVE when observation_tokens > slot_remaining * alpha AND <= slot_remaining.
// 3. ALL when observation_tokens > slot_remaining.
// 4. NEGATIVE: a fitting observation does NOT trigger a non-NONE mode.
//
// Verifies adaptive_reduce integration (S6b):
// 5. adaptive_reduce returns SplitTask in Autonomous mode when budget is over capacity
//    and reduction levels cannot bring it below model_max_context.
// 6. preassess classification is independent of budget state (SELECTIVE/ALL still holds).
// 7. adaptive_reduce is non-fatal: the caller proceeds after the error.

use elai_orchestrator::context_budget::{
    preassess_observation_size, AdaptiveError, ContextBudget, ContextSlot, ExecutionMode,
    PreassessMode,
};
use elai_orchestrator::{Ssb, SsbConfig};
use elai_core::clock::OsClock;
use std::sync::Arc;
use tempfile::TempDir;

const ALPHA: f64 = 0.8;

#[test]
fn none_when_observation_fits_soft_threshold() {
    let mode = preassess_observation_size(100, 500, ALPHA);
    assert_eq!(mode, PreassessMode::None);
}

#[test]
fn none_at_soft_boundary() {
    let mode = preassess_observation_size(400, 500, ALPHA);
    assert_eq!(mode, PreassessMode::None);
}

#[test]
fn selective_above_soft_but_within_slot() {
    let mode = preassess_observation_size(450, 500, ALPHA);
    assert_eq!(mode, PreassessMode::Selective);
}

#[test]
fn all_when_observation_exceeds_slot() {
    let mode = preassess_observation_size(1000, 500, ALPHA);
    assert_eq!(mode, PreassessMode::All);
}

#[test]
fn negative_fitting_observation_is_none() {
    let mode = preassess_observation_size(50, 10_000, ALPHA);
    assert_eq!(mode, PreassessMode::None);
}

fn fresh_ssb() -> (TempDir, Ssb) {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("ssb.db");
    let cfg = SsbConfig {
        db_path: path,
        pool_size: Some(2),
        project_id: "j24-adaptive".into(),
        session_id: "j24-adaptive-test".into(),
    };
    let ssb = Ssb::new(cfg, Arc::new(OsClock)).expect("ssb init");
    (dir, ssb)
}

fn over_capacity_budget() -> ContextBudget {
    let mut budget = ContextBudget::new(100);
    let sp = &mut budget.slots[ContextSlot::SystemPrompt as usize];
    sp.allocated_tokens = 200;
    sp.floor_tokens = 200;
    sp.actual_tokens = 200;
    budget.recalculate_total_used();
    assert!(budget.total_used > budget.model_max_context);
    budget
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn adaptive_reduce_returns_split_task_in_autonomous_when_over_capacity() {
    let (_dir, ssb) = fresh_ssb();
    let mut budget = over_capacity_budget();
    let result = budget
        .adaptive_reduce(0.5, ExecutionMode::Autonomous, &ssb)
        .await;
    assert!(result.is_err());
    assert!(
        matches!(result.unwrap_err(), AdaptiveError::SplitTask),
        "expected SplitTask in Autonomous mode"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn adaptive_reduce_returns_escalate_in_interactive_when_over_capacity() {
    let (_dir, ssb) = fresh_ssb();
    let mut budget = over_capacity_budget();
    let result = budget
        .adaptive_reduce(0.5, ExecutionMode::Interactive, &ssb)
        .await;
    assert!(result.is_err());
    assert!(
        matches!(result.unwrap_err(), AdaptiveError::Escalate { .. }),
        "expected Escalate in Interactive mode"
    );
}

#[test]
fn preassess_selective_holds_regardless_of_budget_state() {
    let mode = preassess_observation_size(450, 500, ALPHA);
    assert_eq!(mode, PreassessMode::Selective);
    let _budget = over_capacity_budget();
    let mode_again = preassess_observation_size(450, 500, ALPHA);
    assert_eq!(mode_again, PreassessMode::Selective);
}

#[test]
fn preassess_all_holds_regardless_of_budget_state() {
    let mode = preassess_observation_size(1000, 500, ALPHA);
    assert_eq!(mode, PreassessMode::All);
    let _budget = over_capacity_budget();
    let mode_again = preassess_observation_size(1000, 500, ALPHA);
    assert_eq!(mode_again, PreassessMode::All);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn adaptive_reduce_non_fatal_proceeds_after_error() {
    let (_dir, ssb) = fresh_ssb();
    let mut budget = over_capacity_budget();
    let result = budget
        .adaptive_reduce(0.5, ExecutionMode::Autonomous, &ssb)
        .await;
    assert!(result.is_err());
    assert!(
        budget.total_used > budget.model_max_context,
        "caller proceeds after AdaptiveError: budget still readable, over-capacity state preserved"
    );
}
