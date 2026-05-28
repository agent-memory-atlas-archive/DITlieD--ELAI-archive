//! E-19 task 1 — pre-L2 self-certainty confidence gate.
//!
//! Computes a logit-margin / self-certainty score from a candidate
//! distribution and decides whether L2 micro-reflection should fire. Paper
//! grounding: arXiv:2502.18581 (Self-Certainty BoN selection, Kang/Zhao/Song);
//! arXiv:2510.08308 (First Try Matters — reflections are predominantly
//! confirmatory and rarely flip wrong->right). The threshold default
//! `L2_CONFIDENCE_THRESHOLD = 0.80` is an initial seed; task 10 calibrates it
//! against a 200-trace holdout via Youden's-J max.
//!
//! IO-free. The orchestrator wrapper supplies the candidate distribution and
//! consumes the [`GateVerdict`] to choose between Skip / Reflect / L3Direct /
//! NoCotClass.

use serde::{Deserialize, Serialize};

/// Initial self-certainty threshold for skipping L2 micro-reflection.
///
/// Set to 0.80 (1-sigma calibration band) pending task-10 holdout sweep.
/// Production calibration is persisted at
/// `.elai_cc/eval/reasoning/calibration.toml`. Per-task scoping of this
/// constant lives behind `ELAI_REASONING_QUALITY_PROBE_ENABLE` so we can
/// hot-swap the threshold without redeploying.
pub const L2_CONFIDENCE_THRESHOLD: f32 = 0.80;

/// Lower bound for "L1 is too uncertain to bother with L2": below this we
/// jump straight to L3 (extrinsic verification via the InterrogationTree).
/// Paper grounding: arXiv:2507.02778 (Self-Correction Bench — LLMs correct
/// external errors, not their own, 64.5% blind-spot rate).
pub const L3_DIRECT_THRESHOLD: f32 = 0.40;

/// Outcome of `ConfidenceGate::evaluate` for a given candidate distribution.
///
/// The orchestrator wrapper records this verdict as a
/// `ReasoningQualityProbe { kind: ProbeKind::L2GateDecision }` SSB row before
/// dispatching the chosen path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateVerdict {
    /// L1 confidence >= threshold; skip L2 entirely.
    /// Emits `ProbeKind::L2GateDecision` with `score >= threshold`.
    SkipHighConfidence,
    /// Confidence in `[L3_DIRECT_THRESHOLD, L2_CONFIDENCE_THRESHOLD)` —
    /// proceed with L2 micro-reflection (re-sample with prior).
    Reflect,
    /// Confidence < `L3_DIRECT_THRESHOLD` — promote L3 ahead of L2.
    L3Direct,
    /// Task class is a known no-CoT-helps regime (FailureTaxonomy categories
    /// `implicit_statistical_learning` or `pattern_with_exceptions`) —
    /// skip L2 regardless of L1 confidence. Paper grounding: arXiv:2410.21333
    /// (Mind-Your-Step, up to 36.3pp accuracy drop on cognitive-psych tasks).
    NoCotClass,
}

impl GateVerdict {
    /// Snake-case wire tag for SSB payload serialization.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SkipHighConfidence => "skip_high_confidence",
            Self::Reflect => "reflect",
            Self::L3Direct => "l3_direct",
            Self::NoCotClass => "no_cot_class",
        }
    }
}

/// Computes a self-certainty score from a candidate distribution.
///
/// Implements the logit-margin formulation from arXiv:2502.18581: score is
/// the gap between the top-1 and top-2 normalized probability mass, clamped
/// to `[0.0, 1.0]`. A flat / two-way-tied distribution returns 0.0; a
/// degenerate one-hot distribution returns 1.0.
///
/// Returns 0.0 for empty distributions (degenerate case — orchestrator
/// MUST treat that as gate-skip, never as high-confidence).
///
/// IO-free. No unwrap (per CLAUDE.md anti-unwrap in library code).
#[must_use]
pub fn compute_score(candidate_distribution: &[f32]) -> f32 {
    if candidate_distribution.is_empty() {
        return 0.0;
    }

    let sum: f32 = candidate_distribution.iter().sum();
    if sum <= 0.0 || !sum.is_finite() {
        return 0.0;
    }

    // Normalize defensively (callers may pass logits or un-normalized
    // counts).
    let mut top1 = 0.0f32;
    let mut top2 = 0.0f32;
    for &p in candidate_distribution {
        let normalized = if p.is_finite() { p / sum } else { 0.0 };
        if normalized >= top1 {
            top2 = top1;
            top1 = normalized;
        } else if normalized > top2 {
            top2 = normalized;
        }
    }

    let margin = top1 - top2;
    margin.clamp(0.0, 1.0)
}

/// Stateful gate that maps a candidate distribution + optional no-CoT-class
/// flag to a [`GateVerdict`].
///
/// Construct via [`ConfidenceGate::new`] (custom threshold) or
/// [`ConfidenceGate::default`] (seeded `L2_CONFIDENCE_THRESHOLD`).
#[derive(Debug, Clone, Copy)]
pub struct ConfidenceGate {
    threshold: f32,
    l3_direct: f32,
}

impl Default for ConfidenceGate {
    fn default() -> Self {
        Self {
            threshold: L2_CONFIDENCE_THRESHOLD,
            l3_direct: L3_DIRECT_THRESHOLD,
        }
    }
}

impl ConfidenceGate {
    /// Build a gate with a custom L2 threshold (kept the L3-direct floor at
    /// `L3_DIRECT_THRESHOLD` for safety; the floor is below the L2 cap so
    /// only the upper threshold is tunable here).
    #[must_use]
    pub fn new(threshold: f32) -> Self {
        Self {
            threshold: threshold.clamp(0.0, 1.0),
            l3_direct: L3_DIRECT_THRESHOLD,
        }
    }

    /// Current L2 threshold.
    #[must_use]
    pub fn threshold(&self) -> f32 {
        self.threshold
    }

    /// Current L3-direct floor.
    #[must_use]
    pub fn l3_direct(&self) -> f32 {
        self.l3_direct
    }

    /// Evaluate a candidate distribution with an optional no-CoT-class flag.
    ///
    /// The `no_cot_class` flag is set by the caller after consulting the
    /// FailureTaxonomy classifier (orchestrator wrapper). When set, the gate
    /// returns [`GateVerdict::NoCotClass`] regardless of the computed score.
    #[must_use]
    pub fn evaluate(&self, candidate_distribution: &[f32], no_cot_class: bool) -> GateVerdict {
        if no_cot_class {
            return GateVerdict::NoCotClass;
        }
        let score = compute_score(candidate_distribution);
        if score >= self.threshold {
            GateVerdict::SkipHighConfidence
        } else if score < self.l3_direct {
            GateVerdict::L3Direct
        } else {
            GateVerdict::Reflect
        }
    }

    /// Convenience: evaluate + return the computed score so callers can
    /// include the numeric value in the SSB row payload.
    #[must_use]
    pub fn evaluate_with_score(
        &self,
        candidate_distribution: &[f32],
        no_cot_class: bool,
    ) -> (GateVerdict, f32) {
        let score = compute_score(candidate_distribution);
        if no_cot_class {
            return (GateVerdict::NoCotClass, score);
        }
        let verdict = if score >= self.threshold {
            GateVerdict::SkipHighConfidence
        } else if score < self.l3_direct {
            GateVerdict::L3Direct
        } else {
            GateVerdict::Reflect
        };
        (verdict, score)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_distribution_scores_zero() {
        assert_eq!(compute_score(&[]), 0.0);
    }

    #[test]
    fn one_hot_distribution_scores_one() {
        let s = compute_score(&[1.0, 0.0, 0.0, 0.0]);
        assert!((s - 1.0).abs() < 1e-6, "one-hot should score 1.0, got {s}");
    }

    #[test]
    fn flat_distribution_scores_zero() {
        let s = compute_score(&[0.25, 0.25, 0.25, 0.25]);
        assert!(s.abs() < 1e-6, "uniform should score ~0.0, got {s}");
    }

    #[test]
    fn margin_distribution_scores_correctly() {
        // top1 = 0.6 / 1.0 = 0.6; top2 = 0.3 / 1.0 = 0.3 -> margin 0.3
        let s = compute_score(&[0.6, 0.3, 0.1]);
        assert!((s - 0.3).abs() < 1e-6, "expected ~0.3, got {s}");
    }

    #[test]
    fn gate_skips_above_threshold() {
        let gate = ConfidenceGate::default();
        let verdict = gate.evaluate(&[0.95, 0.05], false);
        assert_eq!(verdict, GateVerdict::SkipHighConfidence);
    }

    #[test]
    fn gate_reflects_in_band() {
        let gate = ConfidenceGate::default();
        // margin 0.5 -> in [0.40, 0.80) -> Reflect
        let verdict = gate.evaluate(&[0.75, 0.25], false);
        assert_eq!(verdict, GateVerdict::Reflect);
    }

    #[test]
    fn gate_promotes_l3_when_low() {
        let gate = ConfidenceGate::default();
        // margin 0.2 -> below 0.40 -> L3Direct
        let verdict = gate.evaluate(&[0.6, 0.4], false);
        assert_eq!(verdict, GateVerdict::L3Direct);
    }

    #[test]
    fn gate_short_circuits_on_no_cot_class() {
        let gate = ConfidenceGate::default();
        let verdict = gate.evaluate(&[1.0, 0.0], true);
        assert_eq!(verdict, GateVerdict::NoCotClass);
    }

    #[test]
    fn gate_verdict_wire_tags() {
        assert_eq!(GateVerdict::SkipHighConfidence.as_str(), "skip_high_confidence");
        assert_eq!(GateVerdict::Reflect.as_str(), "reflect");
        assert_eq!(GateVerdict::L3Direct.as_str(), "l3_direct");
        assert_eq!(GateVerdict::NoCotClass.as_str(), "no_cot_class");
    }
}
