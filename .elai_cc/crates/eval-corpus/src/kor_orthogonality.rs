//! E-54 KOR-orthogonality check at D-7 canary build.
//!
//! Per arXiv:2410.06526 KOR-Bench: 5 families are "knowledge-orthogonal"
//! when domain knowledge cannot transfer between any pair. ELAI's
//! operationalization: train a tiny adapter (e.g. LoRA on Qwen 2.5 Coder 3B)
//! on family A train; measure accuracy on family B test; abort canary build
//! if `family_b_accuracy > random_baseline + 10pp` (10pp = the empirical
//! threshold; documented as ELAI-specific operationalization, not direct from
//! paper -- see workflow/reasoning/E-54-decisions.md DECISION-5).
//!
//! The cross-family transfer protocol is RUNPOD-PENDING (LoRA training
//! requires GPU time on the remote 3090). This module ships:
//!   - the surface (cross_family_transfer + TransferResult + TransferError)
//!   - the threshold constants (KOR_DELTA_THRESHOLD_PP, MIN_RANDOM_BASELINE)
//!   - a synchronous heuristic fallback (`heuristic_orthogonality_score`)
//!     that the host-side dashboard can render until the LoRA loop lands.
//! Wire-dark by default per ELAI_EVAL_CORPUS_ENABLE.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::TaskFamily;

/// E-54 KOR-Bench cross-family transfer threshold in percentage points.
/// If family B accuracy exceeds random_baseline + this threshold, the two
/// families are NOT knowledge-orthogonal and the canary build aborts.
pub const KOR_DELTA_THRESHOLD_PP: f64 = 10.0;

/// E-54 KOR-orthogonality verdict for one family pair.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferResult {
    pub family_a: TaskFamily,
    pub family_b: TaskFamily,
    /// Accuracy of the family-A-trained adapter on family-B test set, in
    /// `[0.0, 1.0]`.
    pub accuracy: f64,
    /// Random baseline for family B (multiple-choice = 1/n; binary = 0.5;
    /// open-form = task-specific). Caller supplies; this module does NOT
    /// guess.
    pub random_baseline: f64,
    /// `accuracy - random_baseline`, scaled to percentage points.
    pub delta_pp: f64,
    /// True when `delta_pp > KOR_DELTA_THRESHOLD_PP` (= families NOT
    /// orthogonal -> canary build MUST abort).
    pub violated: bool,
}

/// KOR-orthogonality check errors.
#[derive(Debug, Error)]
pub enum KorError {
    #[error("eval-corpus kor: random_baseline must be in [0.0, 1.0], got {0}")]
    InvalidBaseline(f64),
    #[error("eval-corpus kor: accuracy must be in [0.0, 1.0], got {0}")]
    InvalidAccuracy(f64),
}

/// Compute the verdict for one family pair from raw accuracy + baseline.
pub fn evaluate_transfer(
    family_a: TaskFamily,
    family_b: TaskFamily,
    accuracy: f64,
    random_baseline: f64,
) -> Result<TransferResult, KorError> {
    if !(0.0..=1.0).contains(&accuracy) {
        return Err(KorError::InvalidAccuracy(accuracy));
    }
    if !(0.0..=1.0).contains(&random_baseline) {
        return Err(KorError::InvalidBaseline(random_baseline));
    }
    let delta_pp = (accuracy - random_baseline) * 100.0;
    let violated = delta_pp > KOR_DELTA_THRESHOLD_PP;
    Ok(TransferResult {
        family_a,
        family_b,
        accuracy,
        random_baseline,
        delta_pp,
        violated,
    })
}

/// RUNPOD-PENDING: live cross-family transfer training. Wires to remote 3090
/// LoRA harness once `crates/sandbox/` + adapter-runner are in place. Until
/// then this returns a `KorError::InvalidAccuracy(-1.0)` so the caller
/// surfaces "no data" rather than fabricating a passing verdict. Cap is one
/// tiny LoRA per (A,B) pair (10 pairs total for 5 families) at canary build
/// time; cached to `held-out/_audit/kor_matrix_{date}.json` with daily
/// refresh.
pub async fn cross_family_transfer(
    family_a: TaskFamily,
    family_b: TaskFamily,
    random_baseline: f64,
) -> Result<TransferResult, KorError> {
    let _ = (family_a, family_b, random_baseline);
    // TODO[E-54.5]: wire live LoRA harness. Until then, return a no-data
    // sentinel. Caller treats Err verdict as "not verified" (does NOT promote
    // canary build).
    Err(KorError::InvalidAccuracy(-1.0))
}

/// Heuristic orthogonality score in `[0.0, 1.0]`. Each family pair gets a
/// hand-anchored prior: the 4 reasoning-flavored families (CodeFix /
/// MathProof / ProseReason / PlanningDecomposition) share enough structure
/// that the heuristic prior caps at 0.55 (= "almost orthogonal"); ToolUse vs
/// any other family scores 0.80 (= "strongly orthogonal" -- interaction vs
/// recall axis per E-54-decisions.md DECISION-1).
///
/// Used by the dashboard EvalCorpusPanel for a stop-gap visualisation until
/// the live cross-family transfer LoRA harness comes online.
#[must_use]
pub fn heuristic_orthogonality_score(a: TaskFamily, b: TaskFamily) -> f64 {
    if a == b {
        return 0.0;
    }
    match (a, b) {
        (TaskFamily::ToolUse, _) | (_, TaskFamily::ToolUse) => 0.80,
        _ => 0.55,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kor_orthogonality_blocks_d7_on_violation() {
        // family-B accuracy 15pp above baseline -> violated.
        let r = evaluate_transfer(
            TaskFamily::CodeFix,
            TaskFamily::MathProof,
            0.45,
            0.30,
        )
        .unwrap();
        assert!(r.violated, "delta {} pp must violate", r.delta_pp);
    }

    #[test]
    fn kor_orthogonality_passes_at_threshold() {
        // delta exactly 5pp -> not violated.
        let r = evaluate_transfer(
            TaskFamily::CodeFix,
            TaskFamily::MathProof,
            0.35,
            0.30,
        )
        .unwrap();
        assert!(!r.violated);
    }

    #[test]
    fn invalid_accuracy_errors() {
        let r = evaluate_transfer(
            TaskFamily::CodeFix,
            TaskFamily::MathProof,
            1.5,
            0.30,
        );
        assert!(matches!(r, Err(KorError::InvalidAccuracy(_))));
    }

    #[test]
    fn heuristic_score_tool_use_is_strongly_orthogonal() {
        let s = heuristic_orthogonality_score(TaskFamily::ToolUse, TaskFamily::CodeFix);
        assert!(s >= 0.75);
    }
}
