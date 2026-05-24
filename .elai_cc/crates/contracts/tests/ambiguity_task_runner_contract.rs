//! C-8 contract test — task_runner -> GroundingSweepGate wiring of the
//! `AmbiguityScore` produced by `AmbiguityScorer::score()` at the
//! PLANNING -> TESTING_FIRST transition.
//!
//! Source plan: docs/thefinalglance/C-plan/C-8_viewer_stub_eradication.md
//!   (plan ADD clause for tests/ambiguity_task_runner_contract.rs)
//! Decisions:   workflow/reasoning/C-8-decisions.md (C-8 closure entries
//!   `DRIFT-N` / `DRIFT-O` / `DRIFT-P` documenting plan amendments)
//!
//! Substantive AmbiguityScorer / gate-matrix coverage already lives in
//! `crates/contracts/tests/ambiguity_scorer_contract.rs` (lines 250-344).
//! THIS file is the C-8 wiring witness: it pins the contract that the
//! `TransitionContext.ambiguity_score` field is the single channel by
//! which `AmbiguityScorer::score()` results reach `GroundingSweepGate`,
//! so any future refactor that drops the field will fail this test
//! before reaching production.
//!
//! Tests are intentionally narrow (two assertions, one per concern):
//!   (1) Some(AmbiguityScore { overall_score > AMBIGUITY_THRESHOLD })
//!       -> gate returns BLOCK with an ambiguity-tagged reason.
//!   (2) Some(low-score + qualified_streak) -> gate returns ALLOW.
//!   (3) None -> gate falls back to B-17 streak-only behavior (ALLOW
//!       when ungrounded_claims is empty).

use elai_orchestrator::evidence::{
    check_transition, EvidenceGate, GroundingSweepGate, TransitionContext,
};
use elai_orchestrator::fsm::FsmState;
use elai_reasoning::ambiguity::{
    AmbiguityScore, ComponentScore, ScoreBreakdown, CONSTRAINT_CLARITY_WEIGHT,
    GOAL_CLARITY_WEIGHT, SUCCESS_CRITERIA_CLARITY_WEIGHT,
};

// ---------------------------------------------------------------------------
// Helpers (mirror the canonical pattern from ambiguity_scorer_contract.rs)
// ---------------------------------------------------------------------------

fn comp(name: &str, clarity: f32, weight: f32) -> ComponentScore {
    ComponentScore {
        name: name.to_string(),
        clarity_score: clarity,
        weight,
        justification: "c8-wiring-fixture".to_string(),
    }
}

/// Build a greenfield AmbiguityScore with the requested per-dimension
/// clarities. Mirrors `make_score_greenfield` in
/// `ambiguity_scorer_contract.rs:41` so the fixture stays in lock-step
/// with the substantive gate-matrix tests.
fn make_score_greenfield(goal: f32, constraint: f32, success: f32) -> AmbiguityScore {
    let weighted = goal * GOAL_CLARITY_WEIGHT
        + constraint * CONSTRAINT_CLARITY_WEIGHT
        + success * SUCCESS_CRITERIA_CLARITY_WEIGHT;
    AmbiguityScore {
        overall_score: 1.0 - weighted,
        breakdown: ScoreBreakdown {
            goal_clarity: comp("goal_clarity", goal, GOAL_CLARITY_WEIGHT),
            constraint_clarity: comp(
                "constraint_clarity",
                constraint,
                CONSTRAINT_CLARITY_WEIGHT,
            ),
            success_criteria_clarity: comp(
                "success_criteria_clarity",
                success,
                SUCCESS_CRITERIA_CLARITY_WEIGHT,
            ),
            context_clarity: None,
        },
    }
}

// ---------------------------------------------------------------------------
// (1) Wiring witness: a high-overall_score AmbiguityScore reaches the gate
//     via `TransitionContext.ambiguity_score` and triggers BLOCK.
// ---------------------------------------------------------------------------

#[test]
fn ambiguity_score_field_carries_block_signal_into_gate() {
    // overall = 0.25 is clearly above AMBIGUITY_THRESHOLD (0.2 per the
    // ambiguity_scorer_contract.rs gate matrix). Wiring witness: the
    // gate must observe the field and emit an ambiguity-tagged BLOCK.
    let score = AmbiguityScore {
        overall_score: 0.25,
        breakdown: make_score_greenfield(0.9, 0.9, 0.9).breakdown,
    };
    let ctx = TransitionContext {
        ungrounded_claims: Some(Vec::new()),
        ambiguity_score: Some(score),
        streak_count: 0,
        brownfield: false,
        is_plan_revision: false,
        ..Default::default()
    };
    let err = GroundingSweepGate
        .check(&ctx)
        .expect_err("high overall_score must BLOCK");
    let mentions_ambiguity = err.suggestion.to_lowercase().contains("ambiguity")
        || err
            .missing_artifacts
            .iter()
            .any(|m| m.to_lowercase().contains("ambiguity"));
    assert!(
        mentions_ambiguity,
        "BLOCK reason must reference ambiguity (witness: field reached the gate); \
         got suggestion={:?} missing={:?}",
        err.suggestion, err.missing_artifacts,
    );
}

// ---------------------------------------------------------------------------
// (2) Wiring witness: a qualifying AmbiguityScore + met streak reaches
//     the gate and produces ALLOW end-to-end through `check_transition`.
// ---------------------------------------------------------------------------

#[test]
fn ambiguity_score_field_allows_transition_when_qualified_and_streak_met() {
    // Perfect clarities -> overall ~ 0.0; qualifies_for_advance == true.
    // streak_count = 1 -> +1 from this snapshot meets the required count
    // (mirrors `gate_allows_when_qualifies_and_streak_met` in the sibling
    // ambiguity_scorer_contract.rs:306).
    let score = make_score_greenfield(0.95, 0.95, 0.95);
    let ctx = TransitionContext {
        ungrounded_claims: Some(Vec::new()),
        ambiguity_score: Some(score),
        streak_count: 1,
        brownfield: false,
        is_plan_revision: false,
        ..Default::default()
    };
    // Direct gate check.
    assert!(
        GroundingSweepGate.check(&ctx).is_ok(),
        "qualifying score + met streak must ALLOW"
    );
    // Full check_transition wrapper (PLANNING -> TESTING_FIRST is the
    // exact transition C-8 Task 1 wires AmbiguityScorer.score() ahead of).
    assert!(
        check_transition(FsmState::Planning, FsmState::TestingFirst, &ctx).is_ok(),
        "qualifying score + met streak must ALLOW via check_transition"
    );
}

// ---------------------------------------------------------------------------
// (3) Wiring witness: a `None` ambiguity_score (B-17 fallback path —
//     the documented `tracing::warn!` branch in task_runner.rs:4955 when
//     `AmbiguityScorer.score()` returns Err) preserves the legacy
//     streak-only ALLOW behavior, with no ambiguity-tagged BLOCK.
// ---------------------------------------------------------------------------

#[test]
fn missing_ambiguity_score_preserves_b17_fallback() {
    let ctx = TransitionContext {
        ungrounded_claims: Some(Vec::new()),
        ambiguity_score: None,
        streak_count: 0,
        brownfield: false,
        is_plan_revision: false,
        ..Default::default()
    };
    assert!(
        GroundingSweepGate.check(&ctx).is_ok(),
        "ambiguity_score: None must preserve B-17 ALLOW behavior"
    );
    assert!(
        check_transition(FsmState::Planning, FsmState::TestingFirst, &ctx).is_ok(),
        "B-17 fallback must ALLOW via check_transition wrapper as well"
    );
}
