//! B-38 contract tests for the AmbiguityScorer / AmbiguityScore types
//! and the GroundingSweepGate continuous-score extension.
//!
//! Source plan: docs/thefinalglance/B-plan/B-38_ambiguity_score_gate.md
//! Source paper: workflow/papers/ouroboros-ambiguity-interface.md
//!
//! These tests verify the public surface required by B-38.  They are written
//! BEFORE the implementation lands (TESTING_FIRST), so they should compile-fail
//! against the current tree (types/methods don't exist yet) — that is the
//! "right kind of failure" gate.

use std::sync::Arc;

use elai_orchestrator::evidence::{
    check_transition, EvidenceGate, GroundingSweepGate, TransitionContext,
};
use elai_orchestrator::fsm::FsmState;
use elai_reasoning::ambiguity::{
    AmbiguityError, AmbiguityScore, AmbiguityScorer, ComponentScore, Milestone, ScoreBreakdown,
    AMBIGUITY_THRESHOLD, AUTO_COMPLETE_STREAK_REQUIRED, BROWNFIELD_CONSTRAINT_CLARITY_WEIGHT,
    BROWNFIELD_CONTEXT_CLARITY_FLOOR, BROWNFIELD_CONTEXT_CLARITY_WEIGHT,
    BROWNFIELD_GOAL_CLARITY_WEIGHT, BROWNFIELD_SUCCESS_CRITERIA_CLARITY_WEIGHT, clamp01,
    CONSTRAINT_CLARITY_FLOOR, CONSTRAINT_CLARITY_WEIGHT, GOAL_CLARITY_FLOOR, GOAL_CLARITY_WEIGHT,
    round_to_4dp, SUCCESS_CRITERIA_CLARITY_FLOOR, SUCCESS_CRITERIA_CLARITY_WEIGHT,
};
use elai_providers::{ApiError, LlmFuture, LlmProvider, OllamaConfig};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn comp(name: &str, clarity: f32, weight: f32) -> ComponentScore {
    ComponentScore {
        name: name.to_string(),
        clarity_score: clarity,
        weight,
        justification: "test".to_string(),
    }
}

fn make_score_greenfield(goal: f32, constraint: f32, success: f32) -> AmbiguityScore {
    let weighted = goal * GOAL_CLARITY_WEIGHT
        + constraint * CONSTRAINT_CLARITY_WEIGHT
        + success * SUCCESS_CRITERIA_CLARITY_WEIGHT;
    AmbiguityScore {
        overall_score: round_to_4dp(1.0 - weighted),
        breakdown: ScoreBreakdown {
            goal_clarity: comp("goal_clarity", goal, GOAL_CLARITY_WEIGHT),
            constraint_clarity: comp("constraint_clarity", constraint, CONSTRAINT_CLARITY_WEIGHT),
            success_criteria_clarity: comp(
                "success_criteria_clarity",
                success,
                SUCCESS_CRITERIA_CLARITY_WEIGHT,
            ),
            context_clarity: None,
        },
    }
}

fn make_score_brownfield(goal: f32, constraint: f32, success: f32, context: f32) -> AmbiguityScore {
    let weighted = goal * BROWNFIELD_GOAL_CLARITY_WEIGHT
        + constraint * BROWNFIELD_CONSTRAINT_CLARITY_WEIGHT
        + success * BROWNFIELD_SUCCESS_CRITERIA_CLARITY_WEIGHT
        + context * BROWNFIELD_CONTEXT_CLARITY_WEIGHT;
    AmbiguityScore {
        overall_score: round_to_4dp(1.0 - weighted),
        breakdown: ScoreBreakdown {
            goal_clarity: comp("goal_clarity", goal, BROWNFIELD_GOAL_CLARITY_WEIGHT),
            constraint_clarity: comp(
                "constraint_clarity",
                constraint,
                BROWNFIELD_CONSTRAINT_CLARITY_WEIGHT,
            ),
            success_criteria_clarity: comp(
                "success_criteria_clarity",
                success,
                BROWNFIELD_SUCCESS_CRITERIA_CLARITY_WEIGHT,
            ),
            context_clarity: Some(comp("context_clarity", context, BROWNFIELD_CONTEXT_CLARITY_WEIGHT)),
        },
    }
}

// ---------------------------------------------------------------------------
// 1. Weight sums (greenfield)
// ---------------------------------------------------------------------------

#[test]
fn weights_sum_to_one_greenfield() {
    let sum = GOAL_CLARITY_WEIGHT + CONSTRAINT_CLARITY_WEIGHT + SUCCESS_CRITERIA_CLARITY_WEIGHT;
    assert!((sum - 1.0).abs() < 1e-6, "greenfield weights must sum to 1.0, got {sum}");
}

// ---------------------------------------------------------------------------
// 2. Weight sums (brownfield)
// ---------------------------------------------------------------------------

#[test]
fn weights_sum_to_one_brownfield() {
    let sum = BROWNFIELD_GOAL_CLARITY_WEIGHT
        + BROWNFIELD_CONSTRAINT_CLARITY_WEIGHT
        + BROWNFIELD_SUCCESS_CRITERIA_CLARITY_WEIGHT
        + BROWNFIELD_CONTEXT_CLARITY_WEIGHT;
    assert!((sum - 1.0).abs() < 1e-6, "brownfield weights must sum to 1.0, got {sum}");
}

// ---------------------------------------------------------------------------
// 3. Perfect clarity -> overall 0.0
// ---------------------------------------------------------------------------

#[test]
fn ambiguity_score_perfect_clarity_is_zero() {
    let s = make_score_greenfield(1.0, 1.0, 1.0);
    assert_eq!(s.overall_score, 0.0);
}

// ---------------------------------------------------------------------------
// 4. No clarity -> overall 1.0
// ---------------------------------------------------------------------------

#[test]
fn ambiguity_score_no_clarity_is_one() {
    let s = make_score_greenfield(0.0, 0.0, 0.0);
    assert_eq!(s.overall_score, 1.0);
}

// ---------------------------------------------------------------------------
// 5. is_ready threshold boundary
// ---------------------------------------------------------------------------

#[test]
fn is_ready_at_threshold() {
    let at = AmbiguityScore {
        overall_score: 0.20,
        breakdown: make_score_greenfield(1.0, 1.0, 1.0).breakdown,
    };
    assert!(at.is_ready(), "score == AMBIGUITY_THRESHOLD must be ready");

    let over = AmbiguityScore {
        overall_score: 0.21,
        breakdown: make_score_greenfield(1.0, 1.0, 1.0).breakdown,
    };
    assert!(!over.is_ready(), "score > threshold must not be ready");
}

// ---------------------------------------------------------------------------
// 6. floor_failures returns dimension names
// ---------------------------------------------------------------------------

#[test]
fn floor_failures_returns_dimension_names() {
    // goal=0.7 < GOAL_CLARITY_FLOOR (0.75) — should appear in failures.
    let s = make_score_greenfield(0.7, 0.9, 0.9);
    let failures = s.floor_failures(false);
    assert!(
        failures.iter().any(|f| f == "goal_clarity"),
        "expected goal_clarity in failures, got {failures:?}"
    );
}

// ---------------------------------------------------------------------------
// 7. qualifies_for_advance requires BOTH threshold AND no floor failures
// ---------------------------------------------------------------------------

#[test]
fn qualifies_for_advance_requires_both_threshold_and_no_floors() {
    // overall is low (~0.18) but goal floor is breached (0.6 < 0.75).
    let s = make_score_greenfield(0.6, 1.0, 1.0);
    assert!(
        s.overall_score <= AMBIGUITY_THRESHOLD,
        "test fixture invalid; expected overall <= 0.2, got {}",
        s.overall_score
    );
    assert!(!s.floor_failures(false).is_empty());
    assert!(!s.qualifies_for_advance(false));

    let good = make_score_greenfield(0.9, 0.9, 0.9);
    assert!(good.qualifies_for_advance(false));
}

// ---------------------------------------------------------------------------
// 8. Milestone boundaries
// ---------------------------------------------------------------------------

#[test]
fn milestone_initial_progress_refined_ready_boundaries() {
    fn ms(score: f32) -> Milestone {
        AmbiguityScore {
            overall_score: score,
            breakdown: make_score_greenfield(1.0, 1.0, 1.0).breakdown,
        }
        .milestone()
    }
    assert!(matches!(ms(0.10), Milestone::Ready));
    assert!(matches!(ms(0.20), Milestone::Ready));
    assert!(matches!(ms(0.25), Milestone::Refined));
    assert!(matches!(ms(0.30), Milestone::Refined));
    assert!(matches!(ms(0.35), Milestone::Progress));
    assert!(matches!(ms(0.40), Milestone::Progress));
    assert!(matches!(ms(0.50), Milestone::Initial));
    assert!(matches!(ms(1.00), Milestone::Initial));
}

// ---------------------------------------------------------------------------
// 9 & 10. clamp helper
// ---------------------------------------------------------------------------

#[test]
fn clamping_above_one_becomes_one() {
    assert_eq!(clamp01(1.5), 1.0);
    assert_eq!(clamp01(100.0), 1.0);
}

#[test]
fn clamping_below_zero_becomes_zero() {
    assert_eq!(clamp01(-0.1), 0.0);
    assert_eq!(clamp01(-100.0), 0.0);
}

// ---------------------------------------------------------------------------
// 11. Rounding to 4 dp
// ---------------------------------------------------------------------------

#[test]
fn rounding_to_four_decimal_places() {
    let v = round_to_4dp(0.123_456_7);
    assert!((v - 0.1235).abs() < 1e-7, "got {v}");
    let v2 = round_to_4dp(1.0 / 3.0);
    assert!((v2 - 0.3333).abs() < 1e-7, "got {v2}");
}

// ---------------------------------------------------------------------------
// 12. Constants assertions (sanity check the constants exist + correct values)
// ---------------------------------------------------------------------------

#[test]
fn constants_match_ouroboros() {
    assert_eq!(AMBIGUITY_THRESHOLD, 0.2);
    assert_eq!(AUTO_COMPLETE_STREAK_REQUIRED, 2);
    assert_eq!(GOAL_CLARITY_FLOOR, 0.75);
    assert_eq!(CONSTRAINT_CLARITY_FLOOR, 0.65);
    assert_eq!(SUCCESS_CRITERIA_CLARITY_FLOOR, 0.70);
    assert_eq!(BROWNFIELD_CONTEXT_CLARITY_FLOOR, 0.60);
}

// ---------------------------------------------------------------------------
// 13. Gate blocks when ambiguity > threshold
// ---------------------------------------------------------------------------

#[test]
fn gate_blocks_when_ambiguity_above_threshold() {
    let score = AmbiguityScore {
        overall_score: 0.25,
        breakdown: make_score_greenfield(0.9, 0.9, 0.9).breakdown,
    };
    let ctx = TransitionContext {
        ungrounded_claims: Some(vec![]),
        ambiguity_score: Some(score),
        streak_count: 0,
        brownfield: false,
        is_plan_revision: false,
        ..Default::default()
    };
    let err = GroundingSweepGate.check(&ctx).unwrap_err();
    assert!(
        err.suggestion.to_lowercase().contains("ambiguity")
            || err.missing_artifacts.iter().any(|m| m.contains("ambiguity")),
        "expected ambiguity in BLOCK reason, got suggestion={:?} missing={:?}",
        err.suggestion,
        err.missing_artifacts
    );
}

// ---------------------------------------------------------------------------
// 14. Gate blocks on floor failure even if overall <= threshold
// ---------------------------------------------------------------------------

#[test]
fn gate_blocks_when_floor_failed_even_below_threshold() {
    // overall ~ 0.18 (below 0.2) but goal 0.6 < floor 0.75.
    let score = make_score_greenfield(0.6, 1.0, 1.0);
    assert!(score.overall_score <= AMBIGUITY_THRESHOLD);
    let ctx = TransitionContext {
        ungrounded_claims: Some(vec![]),
        ambiguity_score: Some(score),
        streak_count: 0,
        brownfield: false,
        is_plan_revision: false,
        ..Default::default()
    };
    let err = GroundingSweepGate.check(&ctx).unwrap_err();
    assert!(
        err.suggestion.to_lowercase().contains("floor")
            || err.missing_artifacts.iter().any(|m| m.contains("goal_clarity")),
        "expected floor failure in BLOCK reason, got suggestion={:?} missing={:?}",
        err.suggestion,
        err.missing_artifacts
    );
}

// ---------------------------------------------------------------------------
// 15. Gate allows when qualifies AND streak met
// ---------------------------------------------------------------------------

#[test]
fn gate_allows_when_qualifies_and_streak_met() {
    let score = make_score_greenfield(0.95, 0.95, 0.95);
    assert!(score.qualifies_for_advance(false));
    let ctx = TransitionContext {
        ungrounded_claims: Some(vec![]),
        ambiguity_score: Some(score),
        // streak_count + 1 == 2 meets AUTO_COMPLETE_STREAK_REQUIRED
        streak_count: 1,
        brownfield: false,
        is_plan_revision: false,
        ..Default::default()
    };
    assert!(GroundingSweepGate.check(&ctx).is_ok());
}

// ---------------------------------------------------------------------------
// 16. Backward compat: no ambiguity score = B-17 behavior
// ---------------------------------------------------------------------------

#[test]
fn gate_allows_when_no_ambiguity_score_provided() {
    let ctx = TransitionContext {
        ungrounded_claims: Some(vec![]),
        ambiguity_score: None,
        streak_count: 0,
        brownfield: false,
        is_plan_revision: false,
        ..Default::default()
    };
    assert!(
        GroundingSweepGate.check(&ctx).is_ok(),
        "no ambiguity score must preserve B-17 behavior"
    );

    // And the full check_transition wrapper too.
    assert!(
        check_transition(FsmState::Planning, FsmState::TestingFirst, &ctx).is_ok()
    );
}

// ---------------------------------------------------------------------------
// 17. Ungrounded block has priority over ambiguity block
// ---------------------------------------------------------------------------

#[test]
fn gate_preserves_ungrounded_block_priority() {
    let score = AmbiguityScore {
        overall_score: 0.95,
        breakdown: make_score_greenfield(0.0, 0.0, 0.0).breakdown,
    };
    let ctx = TransitionContext {
        ungrounded_claims: Some(vec!["unverified claim".into()]),
        ambiguity_score: Some(score),
        streak_count: 0,
        brownfield: false,
        is_plan_revision: false,
        ..Default::default()
    };
    let err = GroundingSweepGate.check(&ctx).unwrap_err();
    // The message must mention the ungrounded claim, NOT the ambiguity score.
    assert!(
        err.missing_artifacts.iter().any(|m| m.contains("unverified claim")),
        "expected ungrounded claim in error, got {:?}",
        err.missing_artifacts
    );
}

// ---------------------------------------------------------------------------
// 18. mock_llm_scorer_round_trip — greenfield + brownfield, fenced + raw
// ---------------------------------------------------------------------------

struct MockLlm {
    canned: std::sync::Mutex<Vec<String>>,
    model: String,
}

impl MockLlm {
    fn new(model: &str, responses: Vec<String>) -> Self {
        Self {
            canned: std::sync::Mutex::new(responses),
            model: model.into(),
        }
    }
}

impl LlmProvider for MockLlm {
    fn complete<'a>(
        &'a self,
        _prompt: &'a str,
        _config: &'a OllamaConfig,
    ) -> LlmFuture<'a, Vec<String>> {
        Box::pin(async move {
            let next = self
                .canned
                .lock()
                .map_err(|e| ApiError::Auth(format!("mock lock poisoned: {e}")))?
                .pop()
                .ok_or_else(|| ApiError::Auth("mock: no canned responses left".into()))?;
            Ok(vec![next])
        })
    }
    fn embed<'a>(&'a self, _text: &'a str) -> LlmFuture<'a, Vec<f32>> {
        Box::pin(async { Ok(vec![]) })
    }
    fn token_count(&self, text: &str) -> Result<usize, ApiError> {
        Ok(text.len() / 4 + 1)
    }
    fn token_count_estimate(&self, text: &str) -> usize {
        text.len() / 4 + 1
    }
    fn model_name(&self) -> &str {
        &self.model
    }
}

const GREENFIELD_FENCED: &str = "Here is the score:\n```json\n{\
  \"goal_clarity\": {\"clarity_score\": 0.9, \"justification\": \"goal clear\"},\
  \"constraint_clarity\": {\"clarity_score\": 0.8, \"justification\": \"constraints stated\"},\
  \"success_criteria_clarity\": {\"clarity_score\": 0.85, \"justification\": \"criteria measurable\"}\
}\n```";

const BROWNFIELD_RAW: &str = "{\
  \"goal_clarity\": {\"clarity_score\": 0.95, \"justification\": \"g\"},\
  \"constraint_clarity\": {\"clarity_score\": 0.9, \"justification\": \"c\"},\
  \"success_criteria_clarity\": {\"clarity_score\": 0.9, \"justification\": \"s\"},\
  \"context_clarity\": {\"clarity_score\": 0.7, \"justification\": \"ctx\"}\
}";

#[tokio::test(flavor = "current_thread")]
async fn mock_llm_scorer_round_trip_greenfield_fenced() {
    let mock = Arc::new(MockLlm::new("test-model", vec![GREENFIELD_FENCED.to_string()]));
    let scorer = AmbiguityScorer::new(mock, "test-model".into());
    let result = scorer
        .score("plan text", "grounding summary", false, &[])
        .await
        .expect("scorer should parse fenced JSON");
    // overall = 1 - (0.9*0.4 + 0.8*0.3 + 0.85*0.3) = 1 - (0.36 + 0.24 + 0.255) = 1 - 0.855 = 0.145
    assert!(
        (result.overall_score - 0.145).abs() < 1e-3,
        "expected ~0.145, got {}",
        result.overall_score
    );
    assert!(result.breakdown.context_clarity.is_none());
}

#[tokio::test(flavor = "current_thread")]
async fn mock_llm_scorer_round_trip_brownfield_raw() {
    let mock = Arc::new(MockLlm::new("test-model", vec![BROWNFIELD_RAW.to_string()]));
    let scorer = AmbiguityScorer::new(mock, "test-model".into());
    let result = scorer
        .score("plan text", "grounding summary", true, &["decide later: X".into()])
        .await
        .expect("scorer should parse raw JSON");
    // overall = 1 - (0.95*0.35 + 0.9*0.25 + 0.9*0.25 + 0.7*0.15)
    //        = 1 - (0.3325 + 0.225 + 0.225 + 0.105) = 1 - 0.8875 = 0.1125
    assert!(
        (result.overall_score - 0.1125).abs() < 1e-3,
        "expected ~0.1125, got {}",
        result.overall_score
    );
    assert!(result.breakdown.context_clarity.is_some());
}

// ---------------------------------------------------------------------------
// 19. Missing required score field -> hard error (no retry)
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "current_thread")]
async fn missing_score_field_is_hard_error() {
    // No clarity_score field on goal_clarity component.
    let response = r#"{"goal_clarity": {"justification": "x"}, "constraint_clarity": {"clarity_score": 0.8, "justification": "y"}, "success_criteria_clarity": {"clarity_score": 0.8, "justification": "z"}}"#;
    let mock = Arc::new(MockLlm::new("test-model", vec![response.to_string()]));
    let scorer = AmbiguityScorer::new(mock, "test-model".into());
    let err = scorer
        .score("plan", "ground", false, &[])
        .await
        .expect_err("must error on missing score key");
    assert!(
        matches!(err, AmbiguityError::MissingField { .. }),
        "expected MissingField, got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// 20. Streak reset semantics (caller resets via is_plan_revision flag)
// ---------------------------------------------------------------------------

#[test]
fn streak_reset_on_plan_revision_is_callers_responsibility() {
    // When is_plan_revision=true, the gate STILL evaluates streak_count as-is.
    // The intent is: the caller resets streak_count to 0 BEFORE invoking the
    // gate when the plan was revised. This test pins that the flag exists
    // and that the gate consumes the streak_count field — actual reset is
    // a caller-side concern documented in B-38-decisions.md.
    let score = make_score_greenfield(0.95, 0.95, 0.95);
    let ctx = TransitionContext {
        ungrounded_claims: Some(vec![]),
        ambiguity_score: Some(score.clone()),
        streak_count: 0,
        brownfield: false,
        is_plan_revision: true,
        ..Default::default()
    };
    // With streak_count=0 (reset), 0+1 < AUTO_COMPLETE_STREAK_REQUIRED (2),
    // so gate must BLOCK with the streak reason.
    let err = GroundingSweepGate.check(&ctx).unwrap_err();
    assert!(
        err.suggestion.to_lowercase().contains("streak")
            || err.missing_artifacts.iter().any(|m| m.contains("streak")),
        "expected streak reason after reset, got suggestion={:?} missing={:?}",
        err.suggestion,
        err.missing_artifacts
    );
}
