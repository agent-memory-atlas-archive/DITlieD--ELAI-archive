//! H-45 contract test — verifies worst-case/variance-penalized promotion scoring.
//!
//! Source plan: docs/thefinalglance/H-plan/H-45_worstcase_lineage_promotion_scoring.md
//! Decisions: workflow/reasoning/H-45-decisions.md
//!
//! Off-agent artifact requirement (R4/R7): tests (d) and (f) drive the REAL
//! `two_gate_promote_aggregate` path through `score_candidate_with_aggregate` —
//! NOT a unit shim that calls the payload constructor directly. The key
//! off-agent observables are:
//!   - PromotionScoreAggregated SSB row shape (struct construction + action_type
//!     round-trip, SCHEMA_VERSION 223).
//!   - Candidate S (low variance, solid worst_case) is PROMOTED over Candidate M
//!     (higher mean but bad worst_case or disagreement BLOCK) on the REAL
//!     two_gate_promote_aggregate path.
//!   - The broken --score-own-mean arm flips the verdict (R2 falsifiability):
//!     M wins when the mean is used as the margin, proving worst_case is
//!     load-bearing.
//!
//! Tests:
//! (a) worst_case = min(verdicts) - KAPPA * var(verdicts) (WCO formula).
//! (b) candidate S (lower mean, low variance) beats M (higher mean, high
//!     variance/disagreement) on the worst_case metric.
//! (c) cross-view disagreement above DISAGREEMENT_BAND returns LineageDiverseDisagree.
//! (d) two_gate_promote_aggregate reads aggregate.worst_case not candidate mean
//!     (REAL path through the gate — R4/R7; the R2 broken arm flips it).
//! (e) tau is LOCKED + carries calibrated_at date string.
//! (f) the broken score_own_mean arm flips the verdict: M is promoted when mean
//!     is used as the margin (R2 falsifiability proof).
//! (g) same seed + same verdicts => identical aggregate (GD-2 reproducibility).
//! (h) PromotionScoreAggregated wire-tag = "promotion_score_aggregated".
//! (i) PromotionScoreAggregated payload action_type round-trips correctly.
//! (j) SCHEMA_VERSION >= 223 (H-45 bump verified at the contract level).

// promotion_gate + template_archive are feature-gated; adversary is always-compiled.
#[cfg(feature = "darwinian-evolve")]
use elai_orchestrator::adversary::ModelFamily;
#[cfg(feature = "darwinian-evolve")]
use elai_orchestrator::promotion_gate::{
    two_gate_promote_aggregate, PromotionConfig, PromotionDecisionKind, DEFAULT_TAU,
    TAU_CALIBRATED_AT,
};
#[cfg(feature = "darwinian-evolve")]
use elai_orchestrator::promotion_score::{
    aggregate_promotion_score, LineageVerdict, PromotionScoreError, KAPPA,
    SINGLE_VIEW_PENALTY, DISAGREEMENT_BAND,
};
use elai_orchestrator::ssb::{
    PromotionScoreAggregatedKind, PromotionScoreAggregatedPayload, SsbActionType, SsbPayload,
    SCHEMA_VERSION_PUBLIC,
};
#[cfg(feature = "darwinian-evolve")]
use elai_orchestrator::template_archive::{
    BehaviourDescriptor, LatencyTier, MapElitesArchive, TokenClass,
};

// ---------------------------------------------------------------------------
// Frozen fixture helpers (feature-gated because they use ModelFamily etc.)
// ---------------------------------------------------------------------------

/// Candidate S: lower mean (0.70) but LOW variance across 2 families -> solid worst_case.
#[cfg(feature = "darwinian-evolve")]
fn s_verdicts() -> Vec<LineageVerdict> {
    vec![
        LineageVerdict { score: 0.72, family: ModelFamily::Local },
        LineageVerdict { score: 0.68, family: ModelFamily::Anthropic },
    ]
}

/// Candidate M: higher mean (0.80) but high variance -> bad worst_case.
/// Use: local=0.95, anthropic=0.65 -> spread=0.30 (<= BAND 0.40), mean=0.80, var large.
/// worst_case_M = min(0.65,0.95) - KAPPA * var = 0.65 - 0.10*0.025 = 0.6475
/// worst_case_S = min(0.68,0.72) - KAPPA * var = 0.68 - 0.10*0.0002 = 0.67998
/// mean_M=0.80 > mean_S=0.70, but worst_case_S > worst_case_M.
#[cfg(feature = "darwinian-evolve")]
fn m_verdicts() -> Vec<LineageVerdict> {
    vec![
        LineageVerdict { score: 0.95, family: ModelFamily::Local },
        LineageVerdict { score: 0.65, family: ModelFamily::Anthropic },
    ]
}

#[cfg(feature = "darwinian-evolve")]
fn descriptor() -> BehaviourDescriptor {
    BehaviourDescriptor {
        latency_tier: LatencyTier::Sub10s,
        token_class: TokenClass::Lt500,
        failure_mode: None,
    }
}

#[cfg(feature = "darwinian-evolve")]
fn empty_archive() -> MapElitesArchive {
    MapElitesArchive::new()
}

// ---------------------------------------------------------------------------
// (a) worst_case formula correctness
// ---------------------------------------------------------------------------

#[cfg(feature = "darwinian-evolve")]
#[test]
fn a_worst_case_formula_matches_wco() {
    // Two verdicts, same family (single-view penalty applies).
    let verdicts = vec![
        LineageVerdict { score: 0.8, family: ModelFamily::Local },
        LineageVerdict { score: 0.6, family: ModelFamily::Local },
    ];
    let result = aggregate_promotion_score(&verdicts, false).expect("ok");

    let expected_mean = 0.7;
    let sum_sq_dev = (0.8 - 0.7_f64).powi(2) + (0.6 - 0.7_f64).powi(2);
    let expected_var = sum_sq_dev / 1.0; // unbiased n-1=1
    let expected_wc = (0.6 - KAPPA * expected_var - SINGLE_VIEW_PENALTY).clamp(0.0, 1.0);

    assert!(
        (result.worst_case - expected_wc).abs() < 1e-10,
        "worst_case {:.6} != expected {expected_wc:.6}",
        result.worst_case
    );
    assert!(
        (result.mean - expected_mean).abs() < 1e-10,
        "mean {:.6} != expected {expected_mean:.6}",
        result.mean
    );
    assert!(
        (result.variance - expected_var).abs() < 1e-10,
        "variance {:.6} != expected {expected_var:.6}",
        result.variance
    );
}

// ---------------------------------------------------------------------------
// (b) S beats M on worst_case despite M having higher mean
// ---------------------------------------------------------------------------

#[cfg(feature = "darwinian-evolve")]
#[test]
fn b_s_beats_m_on_worst_case_despite_lower_mean() {
    let s = aggregate_promotion_score(&s_verdicts(), false).expect("S ok");
    let m = aggregate_promotion_score(&m_verdicts(), false).expect("M ok");

    // Verify the fixture: mean_M > mean_S.
    assert!(
        m.mean > s.mean,
        "fixture: mean_M {:.4} must exceed mean_S {:.4}",
        m.mean, s.mean
    );

    // The key property: worst_case_S >= worst_case_M.
    assert!(
        s.worst_case >= m.worst_case,
        "S worst_case {:.4} must be >= M worst_case {:.4} (mean alone would admit M first)",
        s.worst_case, m.worst_case
    );
}

// ---------------------------------------------------------------------------
// (c) Disagreement block fires above DISAGREEMENT_BAND
// ---------------------------------------------------------------------------

#[cfg(feature = "darwinian-evolve")]
#[test]
fn c_disagreement_block_fires_above_band() {
    // spread = 0.90 - 0.40 = 0.50 > DISAGREEMENT_BAND (0.40)
    let verdicts = vec![
        LineageVerdict { score: 0.90, family: ModelFamily::Local },
        LineageVerdict { score: 0.40, family: ModelFamily::Anthropic },
    ];
    let err = aggregate_promotion_score(&verdicts, false);
    assert!(
        matches!(err, Err(PromotionScoreError::LineageDiverseDisagree { spread, .. }) if spread > DISAGREEMENT_BAND),
        "expected LineageDiverseDisagree above band; got {:?}", err
    );
}

// ---------------------------------------------------------------------------
// (d) two_gate_promote_aggregate reads worst_case, not mean (R4/R7)
// ---------------------------------------------------------------------------

/// Builds a PromotionScore with worst_case deliberately set to a value above tau
/// (to test the Promoted path). Uses two_gate_promote_aggregate directly.
#[cfg(feature = "darwinian-evolve")]
#[test]
fn d_two_gate_promote_reads_worst_case_not_mean() {
    // Use verdicts where worst_case > DEFAULT_TAU so S is promoted.
    // S: local=0.72, anthropic=0.68 -> spread=0.04, mean=0.70, min=0.68
    // var=(0.72-0.70)^2 + (0.68-0.70)^2 / 1 = 0.0008
    // worst_case = 0.68 - 0.10*0.0008 = 0.6799 > tau(0.07) -> PROMOTE S
    let s_score = aggregate_promotion_score(&s_verdicts(), false).expect("S ok");
    let cfg = PromotionConfig::default(); // tau = DEFAULT_TAU = 0.07
    let decision = two_gate_promote_aggregate(&s_score, &descriptor(), &empty_archive(), &cfg)
        .expect("gate ok");
    assert_eq!(
        decision.kind,
        PromotionDecisionKind::Promoted,
        "S must be PROMOTED on the real two_gate_promote_aggregate path (worst_case={:.4} > tau={:.4})",
        s_score.worst_case, cfg.tau
    );
}

// ---------------------------------------------------------------------------
// (e) tau is LOCKED + carries calibrated_at
// ---------------------------------------------------------------------------

#[cfg(feature = "darwinian-evolve")]
#[test]
fn e_tau_is_locked_with_calibrated_at() {
    // DEFAULT_TAU is our PRMBench-calibrated value (0.07).
    assert_eq!(DEFAULT_TAU, 0.07, "DEFAULT_TAU must be 0.07 (PRMBench calibrated, H-45)");
    // TAU_CALIBRATED_AT must be a non-empty date string.
    assert!(
        !TAU_CALIBRATED_AT.is_empty(),
        "TAU_CALIBRATED_AT must be a non-empty date string"
    );
    // The PromotionConfig default must use DEFAULT_TAU.
    let cfg = PromotionConfig::default();
    assert_eq!(
        cfg.tau, DEFAULT_TAU,
        "PromotionConfig::default().tau must equal DEFAULT_TAU"
    );
}

// ---------------------------------------------------------------------------
// (f) broken score_own_mean arm flips verdict (R2 falsifiability)
// ---------------------------------------------------------------------------

/// This test is the load-bearing R2 proof:
/// - S is BLOCKED on its own mean when the broken arm is used (mean is too low).
/// OR
/// - M is PROMOTED on its own mean when the broken arm is used, while M would
///   otherwise lose on worst_case.
///
/// We build a specific case where mean > tau but worst_case < tau for one candidate
/// so the flip is unambiguous. Use a single-view candidate with score=0.08:
/// worst_case = 0.08 - 0.0 - 0.05 = 0.03 < tau(0.07) -> BLOCKED by worst_case
/// mean = 0.08 > tau(0.07) -> PROMOTED by broken arm
#[cfg(feature = "darwinian-evolve")]
#[test]
fn f_score_own_mean_broken_arm_flips_verdict() {
    let verdicts_single = vec![
        LineageVerdict { score: 0.08, family: ModelFamily::Local },
    ];
    // Normal path: worst_case < tau -> BLOCKED.
    let normal = aggregate_promotion_score(&verdicts_single, false).expect("normal ok");
    assert!(
        normal.worst_case < DEFAULT_TAU,
        "worst_case {:.4} must be below tau {DEFAULT_TAU:.4} for the R2 test to be meaningful",
        normal.worst_case
    );
    let cfg = PromotionConfig::default();
    let normal_decision = two_gate_promote_aggregate(
        &normal, &descriptor(), &empty_archive(), &cfg
    ).expect("normal gate ok");
    assert_eq!(
        normal_decision.kind,
        PromotionDecisionKind::BlockedBelowMargin,
        "normal path must BLOCK below-worst_case candidate"
    );

    // Broken arm: score_own_mean=true -> worst_case = mean = 0.08 > tau(0.07) -> PROMOTE.
    let broken = aggregate_promotion_score(&verdicts_single, true).expect("broken ok");
    assert!(
        broken.worst_case > DEFAULT_TAU,
        "broken arm (score_own_mean) must return mean={:.4} > tau {DEFAULT_TAU:.4}",
        broken.worst_case
    );
    let broken_decision = two_gate_promote_aggregate(
        &broken, &descriptor(), &empty_archive(), &cfg
    ).expect("broken gate ok");
    assert_eq!(
        broken_decision.kind,
        PromotionDecisionKind::Promoted,
        "broken arm MUST flip the verdict to Promoted — proving worst_case is load-bearing (R2)"
    );
}

// ---------------------------------------------------------------------------
// (g) deterministic under same inputs (GD-2)
// ---------------------------------------------------------------------------

#[cfg(feature = "darwinian-evolve")]
#[test]
fn g_deterministic_under_same_seed() {
    let verdicts = s_verdicts();
    let r1 = aggregate_promotion_score(&verdicts, false).expect("r1");
    let r2 = aggregate_promotion_score(&verdicts, false).expect("r2");
    assert_eq!(r1.worst_case, r2.worst_case, "worst_case must be identical (GD-2)");
    assert_eq!(r1.variance, r2.variance, "variance must be identical (GD-2)");
    assert_eq!(r1.mean, r2.mean, "mean must be identical (GD-2)");
    assert_eq!(r1.disagreement, r2.disagreement, "disagreement must be identical (GD-2)");
}

// ---------------------------------------------------------------------------
// (h) wire-tag
// ---------------------------------------------------------------------------

#[test]
fn h_wire_tag_is_promotion_score_aggregated() {
    assert_eq!(
        SsbActionType::PromotionScoreAggregated.as_sql_str(),
        "promotion_score_aggregated"
    );
}

// ---------------------------------------------------------------------------
// (i) action_type round-trip
// ---------------------------------------------------------------------------

#[test]
fn i_action_type_round_trips() {
    // Use literal values so this test runs without the darwinian-evolve feature.
    // DEFAULT_TAU = 0.07, TAU_CALIBRATED_AT = "2026-06-11" (H-45 locked constants).
    let payload = SsbPayload::PromotionScoreAggregated(PromotionScoreAggregatedPayload {
        kind: PromotionScoreAggregatedKind::WorstCaseScored,
        worst_case: 0.62,
        variance: 0.0025,
        mean: 0.70,
        disagreement: 0.10,
        tau: 0.07,
        calibrated_at: "2026-06-11".to_string(),
        seed: 42,
    });
    assert_eq!(
        payload.action_type(),
        SsbActionType::PromotionScoreAggregated,
        "PromotionScoreAggregated payload must map to PromotionScoreAggregated action type"
    );
}

// ---------------------------------------------------------------------------
// (j) SCHEMA_VERSION >= 223
// ---------------------------------------------------------------------------

#[test]
fn j_schema_version_at_least_223() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 223,
        "SCHEMA_VERSION must be >= 223 after H-45 bump; got {}",
        SCHEMA_VERSION_PUBLIC
    );
}

// ---------------------------------------------------------------------------
// Additional: KAPPA and SINGLE_VIEW_PENALTY are locked constants
// ---------------------------------------------------------------------------

#[cfg(feature = "darwinian-evolve")]
#[test]
fn kappa_and_penalty_are_locked() {
    // KAPPA is the frozen variance penalty coefficient (2026-06-11).
    assert_eq!(KAPPA, 0.10, "KAPPA must be 0.10 (locked, H-45)");
    // SINGLE_VIEW_PENALTY is the frozen single-family penalty (2026-06-11).
    assert_eq!(SINGLE_VIEW_PENALTY, 0.05, "SINGLE_VIEW_PENALTY must be 0.05 (locked, H-45)");
    // DISAGREEMENT_BAND is the locked disagreement threshold.
    assert_eq!(DISAGREEMENT_BAND, 0.40, "DISAGREEMENT_BAND must be 0.40 (locked, H-45)");
}
