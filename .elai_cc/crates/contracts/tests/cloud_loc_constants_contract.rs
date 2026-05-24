//! C-5 follow-on contract — baby-step LOC sizing constants + cost estimator.
//!
//! Verifies the surface added by C-5 Task 5 + Task 6:
//!   1. Four LOC constants ship at the spec values
//!      (cloud-mode-design.txt:121-128): 25 / 40 / 300 / 500.
//!   2. `baby_step_loc_for_mode` dispatches correctly for all 4 of the
//!      canonical (mode, model) combos plus a model-id substring miss.
//!   3. `estimate_task_cost_usd` returns (0, 0) for empty / local cases
//!      and (low, high) with the spec's ±30% spread for cloud cases.
//!   4. The UNGROUNDED_HEURISTIC + spread constants are non-zero.
//!
//! Source plan: docs/thefinalglance/C-plan/C-5_cloud_mode_hardening.md
//!              Task 5 + Task 6
//! Decisions:   workflow/reasoning/C-5-decisions.md

use elai_orchestrator::cloud_mode::{
    baby_step_loc_for_mode, estimate_task_cost_usd, ExecMode, BABY_STEP_LOC_CLOUD_OPUS,
    BABY_STEP_LOC_CLOUD_SONNET, BABY_STEP_LOC_LOCAL_3B, BABY_STEP_LOC_LOCAL_7B,
    COST_ESTIMATE_SPREAD_RATIO, TOKENS_PER_LOC_UNGROUNDED_HEURISTIC,
};
use elai_orchestrator::plan_skeleton::{PhaseEntry, PipelineId, PlanSkeleton};

#[test]
fn loc_constants_match_spec_values() {
    // Spec docs/specs/guide/cloud-mode-design.txt:121-128 — 4 of 5 covered
    // in C-5 (32B omitted, deferred to E-32).
    assert_eq!(BABY_STEP_LOC_LOCAL_3B, 25);
    assert_eq!(BABY_STEP_LOC_LOCAL_7B, 40);
    assert_eq!(BABY_STEP_LOC_CLOUD_SONNET, 300);
    assert_eq!(BABY_STEP_LOC_CLOUD_OPUS, 500);
}

#[test]
fn baby_step_loc_for_mode_dispatches_local_3b() {
    assert_eq!(
        baby_step_loc_for_mode(ExecMode::Local, "qwen2.5-coder-3b-instruct"),
        BABY_STEP_LOC_LOCAL_3B
    );
}

#[test]
fn baby_step_loc_for_mode_dispatches_local_7b() {
    assert_eq!(
        baby_step_loc_for_mode(ExecMode::Local, "qwen2.5-coder-7b-instruct"),
        BABY_STEP_LOC_LOCAL_7B
    );
}

#[test]
fn baby_step_loc_for_mode_dispatches_cloud_sonnet() {
    assert_eq!(
        baby_step_loc_for_mode(ExecMode::Cloud, "claude-sonnet-4-6"),
        BABY_STEP_LOC_CLOUD_SONNET
    );
}

#[test]
fn baby_step_loc_for_mode_dispatches_cloud_opus() {
    assert_eq!(
        baby_step_loc_for_mode(ExecMode::Cloud, "claude-opus-4-1"),
        BABY_STEP_LOC_CLOUD_OPUS
    );
}

#[test]
fn baby_step_loc_for_mode_local_unknown_falls_back_to_7b() {
    assert_eq!(
        baby_step_loc_for_mode(ExecMode::Local, "some-unknown-local-id"),
        BABY_STEP_LOC_LOCAL_7B
    );
}

#[test]
fn baby_step_loc_for_mode_auto_treats_as_cloud() {
    assert_eq!(
        baby_step_loc_for_mode(ExecMode::Auto, "claude-sonnet-4-6"),
        BABY_STEP_LOC_CLOUD_SONNET
    );
}

#[test]
fn ungrounded_heuristic_constant_is_positive_finite() {
    assert!(TOKENS_PER_LOC_UNGROUNDED_HEURISTIC > 0.0);
    assert!(TOKENS_PER_LOC_UNGROUNDED_HEURISTIC.is_finite());
}

#[test]
fn spread_ratio_is_in_valid_open_interval() {
    assert!(COST_ESTIMATE_SPREAD_RATIO > 0.0);
    assert!(COST_ESTIMATE_SPREAD_RATIO < 1.0);
}

#[test]
fn estimate_task_cost_usd_empty_skeleton_returns_zero() {
    let skel = PlanSkeleton::new(PipelineId::from("empty"));
    let (low, high) = estimate_task_cost_usd(&skel, "claude-sonnet-4-6");
    assert_eq!(low, 0.0);
    assert_eq!(high, 0.0);
}

#[test]
fn estimate_task_cost_usd_local_model_returns_zero() {
    let mut skel = PlanSkeleton::new(PipelineId::from("local"));
    let entry = PhaseEntry::new("p1", "step").unwrap();
    skel.add_phase(entry).unwrap();
    let (low, high) = estimate_task_cost_usd(&skel, "qwen2.5-coder-7b");
    assert_eq!(low, 0.0);
    assert_eq!(high, 0.0);
}

#[test]
fn estimate_task_cost_usd_sonnet_returns_low_high_in_spread() {
    let mut skel = PlanSkeleton::new(PipelineId::from("sonnet"));
    for i in 0..5 {
        let entry = PhaseEntry::new(format!("p{i}"), "step").unwrap();
        skel.add_phase(entry).unwrap();
    }
    let (low, high) = estimate_task_cost_usd(&skel, "claude-sonnet-4-6");
    assert!(low > 0.0);
    assert!(high > low);
    let expected_ratio =
        (1.0 + COST_ESTIMATE_SPREAD_RATIO) / (1.0 - COST_ESTIMATE_SPREAD_RATIO);
    let actual_ratio = high / low;
    assert!(
        (actual_ratio - expected_ratio).abs() < 1e-9,
        "ratio drift: expected {expected_ratio} got {actual_ratio}"
    );
}

#[test]
fn estimate_task_cost_usd_opus_is_more_expensive_than_sonnet() {
    let mut skel = PlanSkeleton::new(PipelineId::from("compare"));
    for i in 0..3 {
        let entry = PhaseEntry::new(format!("p{i}"), "step").unwrap();
        skel.add_phase(entry).unwrap();
    }
    let (_sonnet_low, sonnet_high) = estimate_task_cost_usd(&skel, "claude-sonnet-4-6");
    let (_opus_low, opus_high) = estimate_task_cost_usd(&skel, "claude-opus-4-1");
    assert!(
        opus_high > sonnet_high,
        "Opus must project a higher upper bound than Sonnet for the same skeleton"
    );
}
