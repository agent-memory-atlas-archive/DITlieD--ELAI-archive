//! E-19 task 8 — Difficulty estimator + difficulty-conditional TTC budget.
//!
//! Paper grounding: arXiv:2502.10954 (test-time-compute scaling per problem
//! difficulty), arXiv:2408.03314 (Snell et al. — compute-optimal scaling).
//! The signal we use is `FailureTaxonomy category x historical retry count`
//! (both existing ELAI primitives). Mapping (default):
//!   easy   -> (depth=2, n=2, n_lateral=0)
//!   medium -> (depth=3, n=3, n_lateral=0)
//!   hard   -> (depth=3, n=5, n_lateral=2)  // LToT lateral branches engage
//!
//! IO-free. The estimator takes a `(category_slug, retry_count)` tuple and
//! returns a [`Difficulty`]; the budget table is a `match` on the enum.

use serde::{Deserialize, Serialize};

/// Maximum retry count the estimator considers before clamping. Higher
/// counts saturate at `Difficulty::Hard`.
pub const MAX_RETRY_CLAMP: u32 = 4;

/// Three-arm difficulty bucket. Carried by the SSB
/// `ReasoningQualityProbe { kind: ProbeKind::L4TtcBudgetChosen }` row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    /// Snake-case wire tag.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Easy => "easy",
            Self::Medium => "medium",
            Self::Hard => "hard",
        }
    }
}

/// TTC budget chosen for a given [`Difficulty`].
///
/// `depth` is the AB-MCTS depth cap; `n_candidates` is the per-ply
/// candidate count; `n_lateral` is the LToT lateral-branch count
/// (only nonzero on Hard). See arXiv:2510.01500 LToT for the
/// mainline/lateral split.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TtcBudget {
    pub depth: u8,
    pub n_candidates: u8,
    pub n_lateral: u8,
}

impl TtcBudget {
    /// Lookup the canonical budget for a difficulty bucket.
    #[must_use]
    pub fn for_difficulty(d: Difficulty) -> Self {
        match d {
            Difficulty::Easy => Self {
                depth: 2,
                n_candidates: 2,
                n_lateral: 0,
            },
            Difficulty::Medium => Self {
                depth: 3,
                n_candidates: 3,
                n_lateral: 0,
            },
            Difficulty::Hard => Self {
                depth: 3,
                n_candidates: 5,
                n_lateral: 2,
            },
        }
    }
}

/// Difficulty estimator. Stateless; reusable across calls.
///
/// Signal model (heuristic, documented in `workflow/reasoning/E-19-decisions.md`):
///   * `failure_taxonomy_category` is one of the 9 ELAI taxonomy slugs.
///   * Categories marked HARD: `silent_exception_swallow`, `concurrent_state_race`,
///     `unicode_normalization`, `cross_lang_drift`.
///   * Categories marked EASY: `missing_import`, `magic_number`.
///   * Everything else defaults to MEDIUM.
///   * `retry_count >= MAX_RETRY_CLAMP` upgrades the bucket by one rung
///     (Easy -> Medium, Medium -> Hard, Hard -> Hard).
#[derive(Debug, Clone, Copy, Default)]
pub struct DifficultyEstimator;

impl DifficultyEstimator {
    /// Build a new estimator. Stateless; placeholder for future RNG / config
    /// fields if the heuristic gets calibrated.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Estimate difficulty from `(failure_taxonomy_category, retry_count)`.
    ///
    /// Category slug matching is case-sensitive snake_case (matches the
    /// canonical FailureTaxonomy wire labels emitted by
    /// `crate::orchestrator::self_mod_layer1a::classify`).
    #[must_use]
    pub fn estimate(&self, failure_taxonomy_category: &str, retry_count: u32) -> Difficulty {
        let base = match failure_taxonomy_category {
            "missing_import" | "magic_number" => Difficulty::Easy,
            "silent_exception_swallow"
            | "concurrent_state_race"
            | "unicode_normalization"
            | "cross_lang_drift" => Difficulty::Hard,
            _ => Difficulty::Medium,
        };

        if retry_count >= MAX_RETRY_CLAMP {
            match base {
                Difficulty::Easy => Difficulty::Medium,
                Difficulty::Medium => Difficulty::Hard,
                Difficulty::Hard => Difficulty::Hard,
            }
        } else {
            base
        }
    }

    /// Convenience: estimate + return the chosen budget in one call.
    #[must_use]
    pub fn estimate_with_budget(
        &self,
        failure_taxonomy_category: &str,
        retry_count: u32,
    ) -> (Difficulty, TtcBudget) {
        let d = self.estimate(failure_taxonomy_category, retry_count);
        (d, TtcBudget::for_difficulty(d))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn easy_categories_map_easy() {
        let e = DifficultyEstimator::new();
        assert_eq!(e.estimate("missing_import", 0), Difficulty::Easy);
        assert_eq!(e.estimate("magic_number", 0), Difficulty::Easy);
    }

    #[test]
    fn hard_categories_map_hard() {
        let e = DifficultyEstimator::new();
        assert_eq!(e.estimate("silent_exception_swallow", 0), Difficulty::Hard);
        assert_eq!(e.estimate("cross_lang_drift", 1), Difficulty::Hard);
    }

    #[test]
    fn unknown_category_defaults_medium() {
        let e = DifficultyEstimator::new();
        assert_eq!(e.estimate("something_unrecognized", 0), Difficulty::Medium);
    }

    #[test]
    fn retry_clamp_upgrades_bucket() {
        let e = DifficultyEstimator::new();
        assert_eq!(e.estimate("missing_import", MAX_RETRY_CLAMP), Difficulty::Medium);
        assert_eq!(e.estimate("unknown", MAX_RETRY_CLAMP), Difficulty::Hard);
        assert_eq!(e.estimate("cross_lang_drift", MAX_RETRY_CLAMP), Difficulty::Hard);
    }

    #[test]
    fn budget_table_matches_spec() {
        assert_eq!(
            TtcBudget::for_difficulty(Difficulty::Easy),
            TtcBudget { depth: 2, n_candidates: 2, n_lateral: 0 },
        );
        assert_eq!(
            TtcBudget::for_difficulty(Difficulty::Medium),
            TtcBudget { depth: 3, n_candidates: 3, n_lateral: 0 },
        );
        assert_eq!(
            TtcBudget::for_difficulty(Difficulty::Hard),
            TtcBudget { depth: 3, n_candidates: 5, n_lateral: 2 },
        );
    }

    #[test]
    fn difficulty_wire_tags() {
        assert_eq!(Difficulty::Easy.as_str(), "easy");
        assert_eq!(Difficulty::Medium.as_str(), "medium");
        assert_eq!(Difficulty::Hard.as_str(), "hard");
    }
}
