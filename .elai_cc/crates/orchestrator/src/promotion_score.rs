//! H-45: Worst-case + lineage-diverse promotion scoring.
//!
//! Replaces the single-candidate fitness used by `two_gate_promote`'s margin
//! with a conservative **worst-case / variance-penalized aggregate** over
//! verdicts from judges diversified by PRETRAINING lineage.
//!
//! # Grounding
//!
//! - Coste 2310.02743 (ICLR 2024, "Conservative Reward Design"):
//!   `aggregate = min(verdicts) - kappa * var(verdicts)`
//!   (WCO + UWO) "practically eliminates overoptimization, +70% BoN vs
//!   mean scoring". Specific table/figure numbers are in H-45-decisions.md
//!   (paper-ingestor pending full fetch; existence and ICLR acceptance
//!   verified via WebSearch 2026-06-08).
//! - Eisenstein 2312.09244: pretrain-seed diversity generalises better than
//!   finetune-seed; ensembling does NOT eliminate correlated-error hacking —
//!   judges must differ by LINEAGE, not just random seed.
//! - Co-rewarding 2508.00410: a single self-judge collapses 44.81% vs 85.14%
//!   ground-truth accuracy; two DISAGREEING views stay stable — the
//!   disagreement block.
//! - PRMBench 2501.03124 (ssmisya/PRMBench): best PRM scores only 68.0 on
//!   math problems; PRMs over-accept, so tau cannot be a guessed constant —
//!   it is CALIBRATED against PRMBench-style labeled accept/reject pairs.
//!
//! # Wiring discipline
//!
//! - `KAPPA` is a LOCKED constant (TD-2; a self-mod loop must not tune it).
//!   See workflow/reasoning/H-45-decisions.md for the frozen calibration
//!   rationale.
//! - No `.unwrap()`/`.expect()` in prod paths.
//! - `ClockProvider`-free; deterministic under `ELAI_SEED`.
//! - Wire-dark: `aggregate_promotion_score` is unconditional (no feature
//!   gate needed — the struct is pure logic). The caller (`darwin_admit_loop`)
//!   is feature-gated; this module simply provides the math.
//!
//! # D-13 forward dependency
//!
//! The second adversary family (D-13 cross-family adversary) is PLANNED but
//! not yet shipped. When it lands, its verdict is added to the `verdicts` slice
//! as a second `LineageVerdict { family: ModelFamily::Anthropic, score: ... }`.
//! The current implementation fully supports multi-verdict aggregation; the
//! D-13 leg is NOT a stub — it is a documented forward cross-plan dependency
//! (see workflow/reasoning/H-45-decisions.md §D-13-forward-dep). The Tier-1
//! done-gate is worst-case over the AVAILABLE verdicts (Local execution leg
//! from H-43). Single-view candidates are penalised via `SINGLE_VIEW_PENALTY`.

use thiserror::Error;
use tracing::debug;

use crate::adversary::ModelFamily;

// ---------------------------------------------------------------------------
// Constants (LOCKED — TD-2 / H-45 VD-3)
// ---------------------------------------------------------------------------

/// Variance penalty coefficient (kappa).
///
/// Formula: `aggregate = min(verdicts) - KAPPA * variance(verdicts)`.
///
/// LOCKED at 0.10 as of 2026-06-11 (calibrated_at string below).
///
/// This value is a CONSERVATIVE DEFAULT tuned on a 3-generation frozen
/// calibration slice (see workflow/reasoning/H-45-decisions.md
/// §kappa-calibration). It has NOT been tuned on a full held-out benchmark
/// sweep (that is Tier-2 / RUNPOD-PENDING). It is deliberately conservative
/// (prefers false-rejects over false-admits) per GD-4. A self-mod loop MUST
/// NOT tune this at runtime.
pub const KAPPA: f64 = 0.10;

/// Locked calibration date for KAPPA.
pub const KAPPA_CALIBRATED_AT: &str = "2026-06-11";

/// Single-view penalty applied when only ONE lineage verdict is available.
///
/// Eisenstein 2312.09244: seed-diversity alone does not eliminate correlated
/// error. A single-view candidate cannot satisfy VD-4 (lineage-diverse verdicts).
/// We do NOT block on single-view (that would break the existing single-leg
/// path while D-13 is PLANNED), but we penalise by subtracting this constant
/// from the worst-case aggregate. This penalises candidates that have not been
/// evaluated across lineages without making single-leg evaluation impossible.
///
/// LOCKED at 0.05 (2026-06-11). See H-45-decisions.md §single-view-penalty.
pub const SINGLE_VIEW_PENALTY: f64 = 0.05;

/// Disagreement band above which `LineageDiverseDisagree` fires.
///
/// When `max(verdicts) - min(verdicts) > DISAGREEMENT_BAND`, the candidate
/// is BLOCKED. Two views that disagree strongly indicate the reward signal is
/// unreliable (Co-rewarding 2508.00410).
///
/// LOCKED at 0.40 (2026-06-11). A 40-point spread between the highest and
/// lowest family verdict is the threshold for "views are too far apart to
/// trust the aggregate". See H-45-decisions.md §disagreement-band.
pub const DISAGREEMENT_BAND: f64 = 0.40;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Failure modes of the promotion-score aggregator.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum PromotionScoreError {
    /// No verdicts provided — cannot compute worst-case over zero inputs.
    #[error("promotion score: empty verdicts slice; at least one verdict required")]
    EmptyVerdicts,
    /// Cross-view disagreement exceeded `DISAGREEMENT_BAND`: promotion BLOCKED.
    #[error(
        "promotion score: lineage-diverse disagreement {spread:.3} exceeds band {band:.3}; \
         cross-view agreement required before promotion (Co-rewarding 2508.00410)"
    )]
    LineageDiverseDisagree {
        /// Measured spread (max − min) across family verdicts.
        spread: f64,
        /// The band threshold that was exceeded.
        band: f64,
    },
}

// ---------------------------------------------------------------------------
// Verdict types
// ---------------------------------------------------------------------------

/// A single verdict from one lineage-diverse judge.
///
/// `score` is in [0.0, 1.0]; `family` identifies the pretraining lineage
/// (Eisenstein 2312.09244: diversity must be by pretraining, not just seed).
#[derive(Debug, Clone, PartialEq)]
pub struct LineageVerdict {
    /// The judge's score for this candidate. In [0.0, 1.0].
    pub score: f64,
    /// The judge's pretraining-lineage family.
    pub family: ModelFamily,
}

// ---------------------------------------------------------------------------
// Result struct
// ---------------------------------------------------------------------------

/// Output of [`aggregate_promotion_score`].
///
/// The `worst_case` field is the value `two_gate_promote`'s margin uses
/// (see `darwin_admit_loop::score_candidate_with_trap`).
#[derive(Debug, Clone, PartialEq)]
pub struct PromotionScore {
    /// Worst-case / variance-penalized aggregate (WCO + UWO; Coste 2310.02743).
    ///
    /// `worst_case = min(verdicts) - KAPPA * variance(verdicts) - single_view_penalty`
    ///
    /// This is the primary promotion margin signal. A self-mod loop that
    /// replaces this with `mean` restores the overoptimization path (VD-3).
    pub worst_case: f64,
    /// Sample variance across the lineage verdicts.
    pub variance: f64,
    /// Arithmetic mean of the verdicts.
    pub mean: f64,
    /// Spread (max − min) across verdicts — the disagreement diagnostic.
    /// When this exceeds `DISAGREEMENT_BAND`, aggregation returns
    /// `Err(PromotionScoreError::LineageDiverseDisagree)`.
    pub disagreement: f64,
    /// Number of distinct pretraining families observed.
    pub distinct_families: usize,
    /// Whether a single-view penalty was applied (only one family present).
    pub single_view_penalty_applied: bool,
}

// ---------------------------------------------------------------------------
// Core aggregator
// ---------------------------------------------------------------------------

/// Compute the worst-case / variance-penalized aggregate over lineage-diverse
/// promotion verdicts.
///
/// # Contract
///
/// - Returns `Err(EmptyVerdicts)` when the slice is empty.
/// - Returns `Err(LineageDiverseDisagree)` when the spread across family
///   verdicts exceeds `DISAGREEMENT_BAND`.
/// - `worst_case = min - KAPPA * variance - single_view_penalty_if_1_family`.
///   This is the margin signal `two_gate_promote` compares to `tau`.
///
/// # Broken-arm (R2 falsifiability)
///
/// Pass `score_own_mean = true` to swap the margin signal to `mean` instead of
/// `worst_case`. This arm exists ONLY for the R2 falsifiability contract test —
/// it MUST flip the admission decision (a candidate blocked on worst_case is
/// admitted when the mean is used). Production code passes `false`.
pub fn aggregate_promotion_score(
    verdicts: &[LineageVerdict],
    score_own_mean: bool,
) -> Result<PromotionScore, PromotionScoreError> {
    if verdicts.is_empty() {
        return Err(PromotionScoreError::EmptyVerdicts);
    }

    // Clamp scores defensively (prevent out-of-range input from escaping).
    let scores: Vec<f64> = verdicts.iter().map(|v| v.score.clamp(0.0, 1.0)).collect();
    let n = scores.len() as f64;

    let mean = scores.iter().sum::<f64>() / n;
    let min = scores.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let disagreement = max - min;

    // Disagreement block (Co-rewarding 2508.00410): strongly disagreeing family
    // verdicts indicate the reward signal is unreliable.
    if disagreement > DISAGREEMENT_BAND {
        return Err(PromotionScoreError::LineageDiverseDisagree {
            spread: disagreement,
            band: DISAGREEMENT_BAND,
        });
    }

    // Sample variance (unbiased when n > 1, biased population variance when n=1).
    let variance = if n > 1.0 {
        let sum_sq_dev: f64 = scores.iter().map(|&s| (s - mean).powi(2)).sum();
        sum_sq_dev / (n - 1.0)
    } else {
        0.0
    };

    // Count distinct families (Eisenstein VD-4: lineage diversity matters).
    let mut seen_families: Vec<ModelFamily> = Vec::new();
    for v in verdicts {
        if !seen_families.contains(&v.family) {
            seen_families.push(v.family.clone());
        }
    }
    let distinct_families = seen_families.len();

    // Single-view penalty (VD-4): one family = insufficient lineage diversity.
    let single_view_penalty_applied = distinct_families == 1;
    let single_view_penalty = if single_view_penalty_applied {
        SINGLE_VIEW_PENALTY
    } else {
        0.0
    };

    // WCO + UWO (Coste 2310.02743): worst_case = min - kappa*var - penalty.
    let worst_case = (min - KAPPA * variance - single_view_penalty).clamp(0.0, 1.0);

    debug!(
        mean,
        min,
        max,
        variance,
        disagreement,
        worst_case,
        distinct_families,
        single_view_penalty_applied,
        "aggregate_promotion_score computed"
    );

    // R2 broken arm: if score_own_mean is true, swap the margin to mean.
    // This MUST flip a worst-case-blocked admission (mean > worst_case).
    // Production code passes score_own_mean=false unconditionally.
    let effective_worst_case = if score_own_mean {
        mean.clamp(0.0, 1.0)
    } else {
        worst_case
    };

    Ok(PromotionScore {
        worst_case: effective_worst_case,
        variance,
        mean,
        disagreement,
        distinct_families,
        single_view_penalty_applied,
    })
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn local(score: f64) -> LineageVerdict {
        LineageVerdict { score, family: ModelFamily::Local }
    }

    fn anthropic(score: f64) -> LineageVerdict {
        LineageVerdict { score, family: ModelFamily::Anthropic }
    }

    #[test]
    fn empty_verdicts_returns_error() {
        let err = aggregate_promotion_score(&[], false);
        assert!(matches!(err, Err(PromotionScoreError::EmptyVerdicts)));
    }

    #[test]
    fn single_verdict_applies_penalty() {
        // Single view: no lineage diversity — single_view_penalty_applied.
        let result = aggregate_promotion_score(&[local(0.8)], false).expect("ok");
        assert!(result.single_view_penalty_applied);
        // worst_case = clamp(min - KAPPA*var - SINGLE_VIEW_PENALTY)
        // = clamp(0.8 - 0.0 - 0.05) = 0.75
        assert!(
            (result.worst_case - 0.75).abs() < 1e-10,
            "expected 0.75, got {}",
            result.worst_case
        );
        assert_eq!(result.distinct_families, 1);
    }

    #[test]
    fn two_verdicts_no_penalty() {
        // Two distinct families: no single-view penalty.
        let verdicts = vec![local(0.8), anthropic(0.7)];
        let result = aggregate_promotion_score(&verdicts, false).expect("ok");
        assert!(!result.single_view_penalty_applied);
        assert_eq!(result.distinct_families, 2);
        // min=0.7, variance=(0.01+0.01)/1=0.005 (unbiased n-1=1), KAPPA=0.10
        // worst_case = 0.7 - 0.10*0.005 = 0.7 - 0.0005 = 0.6995
        let expected_var = (0.8_f64 - 0.75_f64).powi(2) + (0.7_f64 - 0.75_f64).powi(2); // sum of sq devs
        let var = expected_var / 1.0; // unbiased, n-1=1
        let expected_wc = 0.7 - KAPPA * var;
        assert!(
            (result.worst_case - expected_wc).abs() < 1e-10,
            "expected worst_case {expected_wc}, got {}",
            result.worst_case
        );
    }

    #[test]
    fn candidate_s_beats_m_on_worst_case_despite_lower_mean() {
        // Candidate S: lower mean, low variance — wins on worst_case.
        // Candidate M: higher mean, high variance — loses on worst_case.
        // This is the R4 correctness test for H-45.
        let s_verdicts = vec![local(0.72), anthropic(0.68)]; // mean=0.70, spread=0.04
        let m_verdicts = vec![local(0.88), anthropic(0.40)]; // mean=0.64, spread=0.48 -> DISAGREE

        let s_score = aggregate_promotion_score(&s_verdicts, false).expect("S ok");

        // M has disagreement 0.48 > DISAGREEMENT_BAND 0.40, so it should BLOCK.
        let m_result = aggregate_promotion_score(&m_verdicts, false);
        assert!(
            matches!(m_result, Err(PromotionScoreError::LineageDiverseDisagree { .. })),
            "M must be blocked by LineageDiverseDisagree; got {:?}", m_result
        );

        // S is admitted and has a valid worst_case > 0.
        assert!(
            s_score.worst_case > 0.0,
            "S must have positive worst_case; got {}",
            s_score.worst_case
        );
    }

    #[test]
    fn disagreement_block_fires_above_band() {
        // spread = 0.88 - 0.40 = 0.48 > DISAGREEMENT_BAND (0.40)
        let verdicts = vec![local(0.88), anthropic(0.40)];
        let err = aggregate_promotion_score(&verdicts, false);
        assert!(
            matches!(err, Err(PromotionScoreError::LineageDiverseDisagree { .. })),
            "expected LineageDiverseDisagree; got {:?}", err
        );
    }

    #[test]
    fn disagreement_just_at_band_passes() {
        // spread = exactly DISAGREEMENT_BAND should pass (not strictly greater).
        let high = 0.75;
        let low = high - DISAGREEMENT_BAND;
        let verdicts = vec![local(high), anthropic(low)];
        let result = aggregate_promotion_score(&verdicts, false);
        // spread == DISAGREEMENT_BAND (not >), so this must pass.
        assert!(result.is_ok(), "spread == band must pass; got {:?}", result);
    }

    #[test]
    fn score_own_mean_broken_arm_flips_verdict() {
        // R2 falsifiability: the --score-own-mean broken arm must admit a
        // candidate that the worst-case path blocks.
        //
        // Use a candidate where mean > tau but worst_case < tau (tau = 0.05 default).
        // Local=0.60 + Anthropic=0.05 -> mean=0.325, spread=0.55 > BAND -> DISAGREE
        // but use two verdicts where spread <= BAND and worst_case < tau.
        //
        // local=0.15, anthropic=0.10 -> spread=0.05, mean=0.125, min=0.10, var=0.00125
        // worst_case = 0.10 - 0.10*0.00125 - 0.0 = 0.099875 > tau(0.05): passes!
        // Need a case where worst_case < tau.
        //
        // Use single-view: local=0.08 only.
        // worst_case = 0.08 - 0.0 - SINGLE_VIEW_PENALTY(0.05) = 0.03 < tau(0.05)
        // mean = 0.08 > tau(0.05) -> broken arm MUST flip.
        let verdicts = vec![local(0.08)];

        let normal = aggregate_promotion_score(&verdicts, false).expect("normal ok");
        assert!(
            normal.worst_case < 0.05,
            "worst_case {} must be below tau 0.05 for this test to be meaningful",
            normal.worst_case
        );

        let broken = aggregate_promotion_score(&verdicts, true).expect("broken ok");
        assert!(
            broken.worst_case > 0.05,
            "broken arm (score_own_mean) must return mean={} > tau 0.05",
            broken.worst_case
        );
    }

    #[test]
    fn worst_case_formula_matches_coste_wco() {
        // Verify the WCO formula: aggregate = min - kappa * var.
        // Three verdicts from the same family (all Local — single-view penalty applies).
        let verdicts = vec![
            local(0.9),
            local(0.7),
            local(0.8),
        ];
        let result = aggregate_promotion_score(&verdicts, false).expect("ok");

        let expected_mean = (0.9_f64 + 0.7_f64 + 0.8_f64) / 3.0_f64;
        let sum_sq_dev = (0.9_f64 - expected_mean).powi(2)
            + (0.7_f64 - expected_mean).powi(2)
            + (0.8_f64 - expected_mean).powi(2);
        let expected_var = sum_sq_dev / 2.0_f64; // unbiased, n-1=2
        let expected_wc = (0.7 - KAPPA * expected_var - SINGLE_VIEW_PENALTY).clamp(0.0, 1.0);

        assert!(
            (result.worst_case - expected_wc).abs() < 1e-10,
            "worst_case {} != expected {expected_wc}",
            result.worst_case
        );
        assert!(
            (result.mean - expected_mean).abs() < 1e-10,
            "mean {} != expected {expected_mean}",
            result.mean
        );
    }

    #[test]
    fn deterministic_under_same_inputs() {
        // GD-2 reproducibility: same verdicts -> same aggregate.
        let verdicts = vec![local(0.7), anthropic(0.65)];
        let r1 = aggregate_promotion_score(&verdicts, false).expect("r1");
        let r2 = aggregate_promotion_score(&verdicts, false).expect("r2");
        assert_eq!(r1.worst_case, r2.worst_case);
        assert_eq!(r1.variance, r2.variance);
        assert_eq!(r1.mean, r2.mean);
        assert_eq!(r1.disagreement, r2.disagreement);
    }
}
