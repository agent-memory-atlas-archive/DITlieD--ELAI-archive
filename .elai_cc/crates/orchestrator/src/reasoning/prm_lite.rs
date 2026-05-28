//! E-19 task 4 — PRM-lite L4 node scoring via the existing auditor agent.
//!
//! Paper grounding:
//!   - arXiv:2503.04412 (AB-MCTS, Misaki/Sakana 2025) — paper assumes external
//!     feedback (test execution); ELAI's current `mean_logprob+cosine` is a
//!     documented deviation. PRM-lite augments — does NOT replace — the
//!     existing intrinsic signal.
//!   - aclanthology:2025.acl-long.1230 (PRMBench ACL 2025) — Qwen2.5-Math-PRM-7B
//!     is the SOTA OPEN PRM benchmark. D-2 (PPM sidecar) is the upgrade path:
//!     when D-2 ships, the `auditor_pass` boolean is replaced with a real
//!     PPM scalar score; the weighted-combine math is unchanged.
//!   - arXiv:2502.18581 (Self-Certainty, Kang/Zhao/Song) re-anchored as the
//!     citation for "logit signal is a noisy proxy for correctness" — the
//!     plan's original cite to arXiv:2307.13702 (Lanham CoT-faithfulness)
//!     was WRONG_REGIME per E-19 grounding triage (Lanham measures CoT-text
//!     faithfulness, NOT logit quality).
//!
//! IO-free: this module is the pure value-tuple + weighted-combine math.
//! The orchestrator wraps it around a real call to
//! `per_level_audit_dispatch::record_verdict` in
//! `AuditorMode::DeterministicSyntactic`.

use serde::{Deserialize, Serialize};

/// Auditor-deterministic verdict weight (default 0.6). Sum of all three
/// weights MUST equal 1.0; the constructor enforces.
pub const AUDITOR_WEIGHT: f32 = 0.6;
/// Mean-logprob weight (default 0.2). Documented deviation from the paper.
pub const MEAN_LOGPROB_WEIGHT: f32 = 0.2;
/// Cosine-similarity weight (default 0.2). Documented deviation from the paper.
pub const COSINE_WEIGHT: f32 = 0.2;

/// One score row for one AB-MCTS candidate node.
///
/// `auditor_pass` is the deterministic-syntactic verdict from the B-41
/// `per_level_audit_dispatch` surface (pass = no BLOCK findings from rule
/// checks, e.g. compile + clippy + rule-table); `mean_logprob` and `cosine`
/// are the existing GG intrinsic scalars carried alongside the candidate.
///
/// `weighted_score` is the convex combination
/// `auditor_pass * AUDITOR_WEIGHT + mean_logprob * MEAN_LOGPROB_WEIGHT +
/// cosine * COSINE_WEIGHT`. All inputs MUST be normalized to `[0.0, 1.0]`;
/// the constructor clamps.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AuditorVerifierScore {
    /// True when the auditor pass returned ALLOW (no BLOCK findings).
    /// Wire-encoded as 0.0/1.0 inside the weighted score.
    pub auditor_pass: bool,
    /// Existing GG intrinsic mean-logprob score, normalized `[0.0, 1.0]`.
    pub mean_logprob: f32,
    /// Existing GG intrinsic cosine-similarity score, normalized `[0.0, 1.0]`.
    pub cosine: f32,
    /// Auditor weight in `[0.0, 1.0]`; defaults to [`AUDITOR_WEIGHT`].
    pub auditor_weight: f32,
    /// Mean-logprob weight in `[0.0, 1.0]`; defaults to
    /// [`MEAN_LOGPROB_WEIGHT`].
    pub mean_logprob_weight: f32,
    /// Cosine weight in `[0.0, 1.0]`; defaults to [`COSINE_WEIGHT`].
    pub cosine_weight: f32,
}

impl AuditorVerifierScore {
    /// Build a score row with default weights. Inputs are clamped to
    /// `[0.0, 1.0]` for safety.
    #[must_use]
    pub fn new(auditor_pass: bool, mean_logprob: f32, cosine: f32) -> Self {
        Self {
            auditor_pass,
            mean_logprob: clamp01(mean_logprob),
            cosine: clamp01(cosine),
            auditor_weight: AUDITOR_WEIGHT,
            mean_logprob_weight: MEAN_LOGPROB_WEIGHT,
            cosine_weight: COSINE_WEIGHT,
        }
    }

    /// Build a score row with custom weights. The weights MUST sum to 1.0
    /// within `1e-3`; otherwise this returns `None` so the caller can fall
    /// back to defaults rather than ship a silently-broken score.
    #[must_use]
    pub fn with_weights(
        auditor_pass: bool,
        mean_logprob: f32,
        cosine: f32,
        auditor_weight: f32,
        mean_logprob_weight: f32,
        cosine_weight: f32,
    ) -> Option<Self> {
        let sum = auditor_weight + mean_logprob_weight + cosine_weight;
        if !(0.999..=1.001).contains(&sum) {
            return None;
        }
        Some(Self {
            auditor_pass,
            mean_logprob: clamp01(mean_logprob),
            cosine: clamp01(cosine),
            auditor_weight: clamp01(auditor_weight),
            mean_logprob_weight: clamp01(mean_logprob_weight),
            cosine_weight: clamp01(cosine_weight),
        })
    }

    /// Weighted convex combination in `[0.0, 1.0]`.
    #[must_use]
    pub fn weighted_score(&self) -> f32 {
        let auditor_term = if self.auditor_pass { 1.0 } else { 0.0 };
        let score = auditor_term * self.auditor_weight
            + self.mean_logprob * self.mean_logprob_weight
            + self.cosine * self.cosine_weight;
        clamp01(score)
    }
}

fn clamp01(x: f32) -> f32 {
    if !x.is_finite() {
        0.0
    } else {
        x.clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_weights_sum_to_one() {
        let sum = AUDITOR_WEIGHT + MEAN_LOGPROB_WEIGHT + COSINE_WEIGHT;
        assert!((sum - 1.0).abs() < 1e-6, "default weights must sum to 1.0, got {sum}");
    }

    #[test]
    fn auditor_pass_dominates_default_combine() {
        // auditor_pass=true alone => 0.6; both intrinsics 0 => total 0.6
        let s = AuditorVerifierScore::new(true, 0.0, 0.0);
        assert!((s.weighted_score() - 0.6).abs() < 1e-6);
    }

    #[test]
    fn all_max_inputs_score_one() {
        let s = AuditorVerifierScore::new(true, 1.0, 1.0);
        assert!((s.weighted_score() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn all_min_inputs_score_zero() {
        let s = AuditorVerifierScore::new(false, 0.0, 0.0);
        assert!(s.weighted_score().abs() < 1e-6);
    }

    #[test]
    fn custom_weights_must_sum_to_one() {
        let bad = AuditorVerifierScore::with_weights(true, 0.5, 0.5, 0.5, 0.3, 0.3);
        assert!(bad.is_none(), "weights summing to 1.1 must be rejected");
        let good = AuditorVerifierScore::with_weights(true, 0.5, 0.5, 0.5, 0.25, 0.25);
        assert!(good.is_some());
    }

    #[test]
    fn nan_inputs_treated_as_zero() {
        let s = AuditorVerifierScore::new(false, f32::NAN, f32::INFINITY);
        assert_eq!(s.mean_logprob, 0.0);
        assert_eq!(s.cosine, 0.0);
        assert_eq!(s.weighted_score(), 0.0);
    }
}
