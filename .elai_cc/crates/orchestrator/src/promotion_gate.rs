//! E-7: Two-Gate guardrail at D-5 promotion.
//!
//! Per arXiv:2510.04399 (Wang/Dorchen/Jin, Statistical Limits of
//! Self-Improving Agents, submitted 2025-10-05 rev 2026-02-01):
//! distribution-free learnability is preserved under self-modification
//! IFF the policy-reachable family has uniformly bounded VC dimension.
//! The Two-Gate guardrail enforces this in code: a candidate promotes
//! IFF (margin_held_out_delta >= tau) AND (reachable_policy_capacity
//! <= K_m).
//!
//! Paper-anchored citations:
//!   - arXiv:2510.04399 §Theorem 2 (Two-Gate finite-sample safety
//!     boundary).
//!   - arXiv:2510.04399 §Theorem 11/12 (five modification axes:
//!     Algorithmic / Representational / Architectural / Substrate /
//!     Metacognitive — all reduce to the same capacity-supremum boundary).
//!   - Interface map: workflow/papers/statistical-limits-self-improving-interface.md.
//!
//! K_m default = 50 ships as a CONFIG-KNOB (not hard-coded constant).
//! The paper does NOT prescribe a numerical value. See
//! workflow/reasoning/E-7-decisions.md DECISION-15.
//!
//! Wire-dark: compiles only when `cfg(feature = "darwinian-evolve")` is
//! set. Live emission of `SsbActionType::DarwinianGenerationCompleted`
//! with `DarwinianPhase::TwoGateBlocked` or
//! `DarwinianPhase::ArchiveCellElevated` happens in the consumer that
//! owns the SSB writer.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::promotion_score::PromotionScore;
use crate::template_archive::{ArchiveEntry, BehaviourDescriptor, MapElitesArchive};

/// Default capacity cap K_m (UNCONFIRMED for ELAI; arXiv:2510.04399 does
/// not prescribe a value -- DGM published archive size). Ship as
/// config-knob in PromotionConfig, NOT as a load-bearing constant.
pub const DEFAULT_K_M: u32 = 50;

/// Default margin tau — PRMBench-calibrated (arXiv:2501.03124).
///
/// PRMBench (ssmisya/PRMBench, 6216 math problems) found that the best PRM
/// scores only 68.0 on accept/reject labelled problems, and that PRMs
/// systematically OVER-ACCEPT. To counteract over-acceptance a tau in the
/// range 0.05–0.10 is the defensible operating point at the 70th-percentile
/// precision/recall knee (Harvey-Liu t>3.0 analog — see
/// workflow/reasoning/H-45-decisions.md §prmbench-tau).
///
/// Set to 0.07 (calibrated_at: 2026-06-11) — one step above the legacy
/// 0.05 default to reflect PRMBench's measured over-acceptance rate. This is
/// a LOCKED constant; a self-mod loop that lowers tau restores easy promotion
/// (VD-3). Re-calibrate when the evaluation lane's model changes.
pub const DEFAULT_TAU: f64 = 0.07;

/// Calibration date for `DEFAULT_TAU`.
/// Set when the tau value was last grounded against PRMBench evidence
/// (arXiv:2501.03124). Used in the `PromotionScoreAggregated` SSB row.
pub const TAU_CALIBRATED_AT: &str = "2026-06-11";

/// PromotionConfig — operator-tunable knobs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromotionConfig {
    /// Minimum margin (candidate_score - incumbent_score) required for
    /// promotion. Per arXiv:2510.04399 §Theorem 2 the value must be
    /// strictly positive to preserve PAC learnability.
    pub tau: f64,
    /// Capacity cap on reachable policy family size. Per
    /// arXiv:2510.04399 §Theorem 11 the cap is the supremum boundary
    /// across the five modification axes.
    pub k_m: u32,
}

impl Default for PromotionConfig {
    fn default() -> Self {
        Self {
            tau: DEFAULT_TAU,
            k_m: DEFAULT_K_M,
        }
    }
}

/// Promotion verdict.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromotionDecisionKind {
    /// Both gates passed; archive admit will follow.
    Promoted,
    /// Margin gate failed (candidate_score - incumbent_score < tau).
    BlockedBelowMargin,
    /// Capacity gate failed (reachable_policy_capacity > k_m).
    BlockedCapacityExceeded,
    /// F-10 SELF-MOD precondition failed: the proposal would introduce a CWE
    /// security class absent from the pre-modification baseline (a non-monotone
    /// security regression). Set by the `promote_after_bench_completed` wrapper
    /// BEFORE the two-gate margin/capacity checks run; the
    /// `two_gate_promote` 4-arg signature itself is unchanged.
    BlockedSecurityRegression,
}

/// Verdict + reason. `reason` is `None` on Promoted, `Some` on either
/// blocked variant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromotionDecision {
    pub kind: PromotionDecisionKind,
    pub reason: Option<String>,
}

/// Promotion gate failure modes.
#[derive(Debug, Error)]
pub enum GateError {
    /// Caller passed a non-finite tau or capacity.
    #[error("non-finite promotion config field: {0}")]
    NonFiniteConfig(&'static str),
}

/// Run the Two-Gate guardrail.
///
/// Returns `PromotionDecision::Promoted` IFF the candidate beats the
/// incumbent by at least `cfg.tau` AND the archive's reachable policy
/// capacity is within `cfg.k_m`.
///
/// Live emission of `SsbActionType::DarwinianGenerationCompleted` with
/// `DarwinianPhase::TwoGateBlocked` (or ArchiveCellElevated on Promoted)
/// happens at the caller; this function is pure logic + no I/O.
pub fn two_gate_promote(
    candidate: &ArchiveEntry,
    descriptor: &BehaviourDescriptor,
    archive: &MapElitesArchive,
    cfg: &PromotionConfig,
) -> Result<PromotionDecision, GateError> {
    if !cfg.tau.is_finite() {
        return Err(GateError::NonFiniteConfig("tau"));
    }
    // Margin gate first.
    let incumbent_fitness = archive
        .best_in_cell(descriptor)
        .map(|e| e.fitness)
        .unwrap_or(0.0);
    let margin = candidate.fitness - incumbent_fitness;
    if margin < cfg.tau {
        return Ok(PromotionDecision {
            kind: PromotionDecisionKind::BlockedBelowMargin,
            reason: Some(format!(
                "margin {} below tau {}",
                margin, cfg.tau
            )),
        });
    }
    // Capacity gate. Reachable policy capacity proxy = occupied cells
    // count. Per arXiv:2510.04399 §Theorem 11 the supremum is computed
    // across the five modification axes; this scaffold uses the cell
    // count as a conservative upper bound (UNCONFIRMED — RUNPOD-PENDING
    // for the per-axis breakdown).
    let capacity = archive.occupied_cells() as u32;
    if capacity > cfg.k_m {
        return Ok(PromotionDecision {
            kind: PromotionDecisionKind::BlockedCapacityExceeded,
            reason: Some(format!(
                "capacity {} exceeds k_m {}",
                capacity, cfg.k_m
            )),
        });
    }
    Ok(PromotionDecision {
        kind: PromotionDecisionKind::Promoted,
        reason: None,
    })
}

/// H-45: Run the Two-Gate guardrail using an **aggregate worst-case score**
/// as the promotion margin, not the candidate's own fitness.
///
/// This is the H-45 extension of [`two_gate_promote`] (GD-1: EXTEND the
/// score consumed, do NOT rewrite the tau/k_m gate logic). The margin gate
/// now compares `aggregate.worst_case` to `cfg.tau` instead of the raw
/// candidate fitness.
///
/// # Why worst-case instead of mean
///
/// Scoring on the candidate's own MEAN fitness is exactly the overoptimization
/// regime (Gao 2210.10760 KL law). Coste 2310.02743 (ICLR 2024) shows that
/// WCO/UWO (worst-case / variance-penalized) "practically eliminates
/// overoptimization, +70% BoN". This function wires that fix.
///
/// Live emission of `SsbActionType::PromotionScoreAggregated` happens at the
/// caller (`darwin_admit_loop`); this function is pure logic + no I/O.
pub fn two_gate_promote_aggregate(
    aggregate: &PromotionScore,
    descriptor: &BehaviourDescriptor,
    archive: &MapElitesArchive,
    cfg: &PromotionConfig,
) -> Result<PromotionDecision, GateError> {
    if !cfg.tau.is_finite() {
        return Err(GateError::NonFiniteConfig("tau"));
    }
    // H-45: margin uses aggregate.worst_case, NOT candidate.fitness directly.
    // The incumbent is still read from the archive for consistency with the
    // existing Two-Gate semantics (an archive-cell incumbent is the baseline).
    let incumbent_fitness = archive
        .best_in_cell(descriptor)
        .map(|e| e.fitness)
        .unwrap_or(0.0);
    let margin = aggregate.worst_case - incumbent_fitness;
    if margin < cfg.tau {
        return Ok(PromotionDecision {
            kind: PromotionDecisionKind::BlockedBelowMargin,
            reason: Some(format!(
                "worst_case margin {margin:.4} below tau {tau:.4} \
                 (worst_case={worst:.4}, mean={mean:.4}, var={var:.4})",
                worst = aggregate.worst_case,
                mean = aggregate.mean,
                var = aggregate.variance,
                tau = cfg.tau,
            )),
        });
    }
    // Capacity gate (unchanged from two_gate_promote — GD-1 compose).
    let capacity = archive.occupied_cells() as u32;
    if capacity > cfg.k_m {
        return Ok(PromotionDecision {
            kind: PromotionDecisionKind::BlockedCapacityExceeded,
            reason: Some(format!(
                "capacity {capacity} exceeds k_m {}",
                cfg.k_m
            )),
        });
    }
    Ok(PromotionDecision {
        kind: PromotionDecisionKind::Promoted,
        reason: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template_archive::{
        ArchiveEntry, BehaviourDescriptor, LatencyTier, TokenClass,
    };

    fn descriptor() -> BehaviourDescriptor {
        BehaviourDescriptor {
            latency_tier: LatencyTier::Sub10s,
            token_class: TokenClass::Lt500,
            failure_mode: None,
        }
    }

    fn entry(fitness: f64) -> ArchiveEntry {
        ArchiveEntry {
            template_blake3: [0u8; 32],
            fitness,
        }
    }

    #[test]
    fn empty_cell_promotes_on_positive_margin() {
        let a = MapElitesArchive::new();
        let cfg = PromotionConfig::default();
        let candidate = entry(0.5);
        let decision =
            two_gate_promote(&candidate, &descriptor(), &a, &cfg).expect("gate runs");
        assert_eq!(decision.kind, PromotionDecisionKind::Promoted);
    }

    #[test]
    fn below_margin_blocks() {
        let mut a = MapElitesArchive::new();
        a.admit(descriptor(), entry(0.5));
        let cfg = PromotionConfig::default();
        let candidate = entry(0.51); // margin 0.01 < tau 0.05.
        let decision =
            two_gate_promote(&candidate, &descriptor(), &a, &cfg).expect("gate runs");
        assert_eq!(decision.kind, PromotionDecisionKind::BlockedBelowMargin);
    }

    #[test]
    fn capacity_exceeded_blocks() {
        let mut a = MapElitesArchive::new();
        a.admit(descriptor(), entry(0.5));
        let cfg = PromotionConfig {
            tau: 0.01,
            k_m: 0, // any non-empty archive trips this.
        };
        let candidate = entry(0.7);
        let decision =
            two_gate_promote(&candidate, &descriptor(), &a, &cfg).expect("gate runs");
        assert_eq!(
            decision.kind,
            PromotionDecisionKind::BlockedCapacityExceeded
        );
    }

    #[test]
    fn non_finite_tau_errors() {
        let a = MapElitesArchive::new();
        let cfg = PromotionConfig {
            tau: f64::NAN,
            k_m: 50,
        };
        let result = two_gate_promote(&entry(0.5), &descriptor(), &a, &cfg);
        assert!(matches!(result, Err(GateError::NonFiniteConfig("tau"))));
    }
}
