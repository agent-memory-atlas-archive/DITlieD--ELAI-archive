//! E-39 contract: the cost router never routes all tasks to Opus on
//! the default config (plan stub_ban: "no all-Opus default routing;
//! <10% Opus on default config").
//!
//! Drives 100 synthetic tasks through `route_for_difficulty_and_budget`
//! with the `TaskDifficultyClassifier` and `Criticality::Medium`
//! (the default). Counts Opus decisions. The stub_ban requires < 10%
//! on this baseline config.
//!
//! This is a deterministic check (no SSB write needed; the contract is
//! a behavioral invariant of the router, not a round-trip test).
//!
//! Production entrypoint: elai run -> drive_task -> cost_token_economics
//! path -> `route_for_difficulty_and_budget` + `TaskDifficultyClassifier`
//! (task_runner.rs run_cost_router_preclassify).

use elai_orchestrator::cost::cost_router::{
    route_for_difficulty_and_budget, Criticality, RouteDecision, TaskDifficultyClassifier,
};

#[test]
fn contract_no_all_opus_default_medium_criticality() {
    let classifier = TaskDifficultyClassifier::new();
    let tasks = [
        "rename a variable",
        "add a unit test for foo",
        "wire a new tauri command",
        "fix a typo in a doc comment",
        "implement a simple parser",
        "add a config field",
        "refactor the spend accumulator",
        "extract a helper function",
        "fix a clippy warning",
        "update the changelog",
    ];
    let mut opus_count: u32 = 0;
    for t in tasks.iter().cycle().take(100) {
        let diff = classifier.classify(t);
        let decision = route_for_difficulty_and_budget(diff, 30.0, Criticality::Medium);
        if matches!(decision, RouteDecision::Opus { .. }) {
            opus_count += 1;
        }
    }
    assert!(
        opus_count < 10,
        "stub_ban: no all-Opus default routing — expected < 10% Opus on 100 medium tasks, got {opus_count}%"
    );
}

#[test]
fn contract_no_opus_without_high_criticality() {
    // The router must never emit Opus for Low or Medium criticality
    // regardless of difficulty or budget.
    let classifier = TaskDifficultyClassifier::new();
    let hard_task =
        "prove the optimize algorithm using a formal-verify theorem for a lock-free monad in a parallel concurrency derive proof";
    let diff = classifier.classify(hard_task);
    // Low criticality must not route Opus.
    let low = route_for_difficulty_and_budget(diff, 200.0, Criticality::Low);
    assert!(
        !matches!(low, RouteDecision::Opus { .. }),
        "Low criticality must never route Opus, got {low:?}"
    );
    // Medium criticality must not route Opus.
    let medium = route_for_difficulty_and_budget(diff, 200.0, Criticality::Medium);
    assert!(
        !matches!(medium, RouteDecision::Opus { .. }),
        "Medium criticality must never route Opus, got {medium:?}"
    );
}
