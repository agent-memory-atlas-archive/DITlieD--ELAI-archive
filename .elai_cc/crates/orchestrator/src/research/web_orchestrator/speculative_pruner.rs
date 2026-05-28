//! E-48 Task 5 — FlashResearch speculative branch + runtime pruner.
//!
//! Adds the FlashResearch (arXiv:2510.05145) three-arm BranchType
//! discriminator and the low-evidence-delta + low-gain-streak pruner
//! predicate. The actual node drop is delegated to the existing
//! [`crate::research_tree::ResearchTree::prune_speculative_children`]
//! so the B-7 pruner semantics stay the single source of truth — this
//! module only computes the prune verdict and the
//! [`crate::ssb::WebResearchEventKind::SpeculativeBranchPruned`]
//! telemetry payload.
//!
//! Grounding:
//!   * arXiv:2510.05145 -- FlashResearch / ParallelResearch (breadth /
//!     depth / speculative branches + runtime pruning on intermediate
//!     evidence delta).
//!   * arXiv:2510.12164 -- Survey on Parallel Reasoning (speculative
//!     pruning canon).

use serde::{Deserialize, Serialize};

/// Default evidence-delta ratio threshold below which a speculative
/// branch becomes a prune candidate. Empirical default from the plan
/// task spec; calibration data accrues automatically via the
/// `speculative_branch_pruned` SSB row's `evidence_delta` field so
/// future patches can tune.
pub const DEFAULT_SPECULATIVE_PRUNE_THRESHOLD: f32 = 0.10;

/// Default consecutive-low-gain-iterations count needed before a
/// speculative branch is considered for pruning. Mirrors the existing
/// [`crate::research_tree::ResearchNode::low_gain_streak`] cap pattern.
pub const DEFAULT_LOW_GAIN_STREAK_FLOOR: u32 = 2;

/// E-48 Task 5 -- FlashResearch branch discriminator.
///
/// Defaults to [`BranchType::Question`] for backwards compat -- legacy
/// research nodes (B-7 / B-40) deserialize as `Question` and the
/// pruner refuses to drop them (existing prune-protection semantics
/// preserved).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BranchType {
    /// Standard top-down decomposition fork (the B-7 / B-40 shape).
    /// Default for backwards compat -- legacy nodes deserialize here.
    #[default]
    Question,
    /// Breadth-first fan-out branch (multiple sibling sub-questions
    /// explored in parallel; arXiv:2510.05145 §3.1).
    Breadth,
    /// Depth-first drill-down branch (single sub-question pursued
    /// deeply; arXiv:2510.05145 §3.2).
    Depth,
    /// Speculative side-arm branch (probabilistic side bets on related
    /// questions; arXiv:2510.05145 §3.3). Pruned at runtime by the
    /// predicate in this module when intermediate evidence delta stays
    /// below threshold.
    Speculative,
}

impl BranchType {
    /// Snake-case wire label for SSB telemetry and log lines.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Question => "question",
            Self::Breadth => "breadth",
            Self::Depth => "depth",
            Self::Speculative => "speculative",
        }
    }
}

/// Inputs to the speculative-pruner predicate, captured at the end of
/// one REFLECT cycle for a single research node.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrunerInputs {
    /// The branch's typed kind. Only [`BranchType::Speculative`] is a
    /// prune candidate; all other variants short-circuit `false`.
    pub branch_type: BranchType,
    /// Ratio `|new_evidence ∪ existing| - |existing| / max(1, |existing|)`
    /// of the latest REFLECT cycle. NaN inputs are treated as 0.0 to
    /// avoid pathological prune decisions on numeric edge cases.
    pub evidence_delta: f32,
    /// Consecutive REFLECT cycles in which the node showed low gain.
    /// Mirrors the existing
    /// [`crate::research_tree::ResearchNode::low_gain_streak`] field.
    pub low_gain_streak: u32,
    /// E-48 Task 7 invariant -- the speculative pruner MUST refuse to
    /// drop any node carrying a prune-protected contradiction. Wired
    /// from the [`crate::research_tree::Contradiction::prune_protected`]
    /// field set by the contradiction classifier.
    pub prune_protected: bool,
}

/// Output of the prune predicate.
///
/// `should_prune == true` is the verdict; the caller hands the node-id
/// off to [`crate::research_tree::ResearchTree::prune_speculative_children`]
/// and the producer-side emit fn records a
/// [`crate::ssb::WebResearchEventKind::SpeculativeBranchPruned`] row.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrunerVerdict {
    pub should_prune: bool,
    /// Echoed for the SSB row -- saves the caller having to re-read the
    /// inputs.
    pub branch_type: BranchType,
    pub evidence_delta: f32,
    pub low_gain_streak: u32,
    /// Threshold the verdict was evaluated against (echoed for telemetry).
    pub threshold: f32,
}

/// Evaluate the prune predicate.
///
/// Returns `should_prune == true` only when ALL of the following hold:
///   1. `inputs.branch_type == BranchType::Speculative`
///   2. `inputs.prune_protected == false` (Task 7 invariant)
///   3. `inputs.evidence_delta < threshold` (NaN treated as 0.0)
///   4. `inputs.low_gain_streak > low_gain_streak_floor`
///
/// All other cases short-circuit to `false` so the existing pruner
/// behaviour for legacy branch types stays unchanged.
#[must_use]
pub fn evaluate(
    inputs: PrunerInputs,
    threshold: f32,
    low_gain_streak_floor: u32,
) -> PrunerVerdict {
    let delta = if inputs.evidence_delta.is_nan() {
        0.0
    } else {
        inputs.evidence_delta
    };
    let should_prune = matches!(inputs.branch_type, BranchType::Speculative)
        && !inputs.prune_protected
        && delta < threshold
        && inputs.low_gain_streak > low_gain_streak_floor;
    PrunerVerdict {
        should_prune,
        branch_type: inputs.branch_type,
        evidence_delta: delta,
        low_gain_streak: inputs.low_gain_streak,
        threshold,
    }
}

/// Allocate budget for speculative side-arms at a given tree depth.
///
/// Per the plan task spec:
///   `speculative_budget = min(remaining_total_budget * 0.20,
///                             parent_node_estimated_value * 0.5)`
///
/// Both inputs are budgeted in search-call units (matching
/// [`super::editable_plan::WebResearchPlan::planned_search_budget`]).
/// Returns 0 when either input is 0 to short-circuit zero allocation.
#[must_use]
pub fn speculative_budget(
    remaining_total_budget: usize,
    parent_node_estimated_value: usize,
) -> usize {
    if remaining_total_budget == 0 || parent_node_estimated_value == 0 {
        return 0;
    }
    let twenty_percent = remaining_total_budget / 5;
    let half_parent = parent_node_estimated_value / 2;
    twenty_percent.min(half_parent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn branch_type_default_is_question() {
        assert_eq!(BranchType::default(), BranchType::Question);
    }

    #[test]
    fn branch_type_as_str_canonical() {
        assert_eq!(BranchType::Question.as_str(), "question");
        assert_eq!(BranchType::Breadth.as_str(), "breadth");
        assert_eq!(BranchType::Depth.as_str(), "depth");
        assert_eq!(BranchType::Speculative.as_str(), "speculative");
    }

    #[test]
    fn non_speculative_never_prunes() {
        for bt in [BranchType::Question, BranchType::Breadth, BranchType::Depth] {
            let verdict = evaluate(
                PrunerInputs {
                    branch_type: bt,
                    evidence_delta: 0.0,
                    low_gain_streak: 100,
                    prune_protected: false,
                },
                DEFAULT_SPECULATIVE_PRUNE_THRESHOLD,
                DEFAULT_LOW_GAIN_STREAK_FLOOR,
            );
            assert!(!verdict.should_prune, "{bt:?} must never prune");
        }
    }

    #[test]
    fn prune_protected_never_prunes() {
        let verdict = evaluate(
            PrunerInputs {
                branch_type: BranchType::Speculative,
                evidence_delta: 0.0,
                low_gain_streak: 100,
                prune_protected: true,
            },
            DEFAULT_SPECULATIVE_PRUNE_THRESHOLD,
            DEFAULT_LOW_GAIN_STREAK_FLOOR,
        );
        assert!(!verdict.should_prune, "prune_protected MUST veto the prune");
    }

    #[test]
    fn speculative_prunes_on_low_delta_and_streak() {
        let verdict = evaluate(
            PrunerInputs {
                branch_type: BranchType::Speculative,
                evidence_delta: 0.05,
                low_gain_streak: 3,
                prune_protected: false,
            },
            DEFAULT_SPECULATIVE_PRUNE_THRESHOLD,
            DEFAULT_LOW_GAIN_STREAK_FLOOR,
        );
        assert!(verdict.should_prune);
        assert_eq!(verdict.threshold, DEFAULT_SPECULATIVE_PRUNE_THRESHOLD);
    }

    #[test]
    fn speculative_keeps_on_high_delta() {
        let verdict = evaluate(
            PrunerInputs {
                branch_type: BranchType::Speculative,
                evidence_delta: 0.50,
                low_gain_streak: 100,
                prune_protected: false,
            },
            DEFAULT_SPECULATIVE_PRUNE_THRESHOLD,
            DEFAULT_LOW_GAIN_STREAK_FLOOR,
        );
        assert!(!verdict.should_prune);
    }

    #[test]
    fn speculative_keeps_on_short_streak() {
        let verdict = evaluate(
            PrunerInputs {
                branch_type: BranchType::Speculative,
                evidence_delta: 0.01,
                low_gain_streak: 1,
                prune_protected: false,
            },
            DEFAULT_SPECULATIVE_PRUNE_THRESHOLD,
            DEFAULT_LOW_GAIN_STREAK_FLOOR,
        );
        assert!(!verdict.should_prune);
    }

    #[test]
    fn nan_delta_treated_as_zero() {
        let verdict = evaluate(
            PrunerInputs {
                branch_type: BranchType::Speculative,
                evidence_delta: f32::NAN,
                low_gain_streak: 5,
                prune_protected: false,
            },
            DEFAULT_SPECULATIVE_PRUNE_THRESHOLD,
            DEFAULT_LOW_GAIN_STREAK_FLOOR,
        );
        assert!(verdict.should_prune);
        assert_eq!(verdict.evidence_delta, 0.0);
    }

    #[test]
    fn speculative_budget_clamps_to_min_of_two_inputs() {
        // 20% of 100 = 20; half of 30 = 15 -> min = 15.
        assert_eq!(speculative_budget(100, 30), 15);
        // 20% of 50 = 10; half of 100 = 50 -> min = 10.
        assert_eq!(speculative_budget(50, 100), 10);
    }

    #[test]
    fn speculative_budget_zero_inputs_zero() {
        assert_eq!(speculative_budget(0, 100), 0);
        assert_eq!(speculative_budget(100, 0), 0);
        assert_eq!(speculative_budget(0, 0), 0);
    }
}
