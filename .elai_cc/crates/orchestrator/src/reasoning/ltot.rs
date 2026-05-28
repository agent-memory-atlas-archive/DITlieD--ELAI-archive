//! E-19 task 5 — Lateral ToT (LToT) mainline / lateral split for L4
//! AB-MCTS expansion.
//!
//! Paper grounding: arXiv:2510.01500 (Lateral Tree-of-Thoughts surpasses ToT
//! by incorporating logically-consistent, low-utility candidates). The paper
//! introduces "Lateral Racing with Short-Circuit" (LR-SC) — a budgeted race
//! with aggressive culling. ELAI's E-19 ships the SIMPLIFIED version:
//!
//!   * Keep the top `N_MAINLINE = 3` candidates by utility score.
//!   * Keep up to `N_LATERAL = 2` low-utility-but-LOGICALLY-CONSISTENT
//!     candidates (consistency == the auditor's deterministic-syntactic
//!     `auditor_pass=true` from `prm_lite::AuditorVerifierScore`).
//!   * Lateral candidates survive ONE extra ply only; they are pruned at
//!     the next expansion if they do not surface a top-3 utility score.
//!
//! See `workflow/reasoning/E-19-decisions.md` DECISION-LTOT-SIMPLIFICATION.
//!
//! IO-free.

use serde::{Deserialize, Serialize};

/// Number of high-utility candidates retained at each AB-MCTS ply.
pub const N_MAINLINE: usize = 3;
/// Number of low-utility-but-logically-consistent candidates retained one
/// extra ply.
pub const N_LATERAL: usize = 2;

/// One candidate input to the lateral split.
///
/// `id` is the navigator-assigned candidate identifier (caller's wire). The
/// pair `(utility, consistent)` is the scoring tuple — `utility` from the
/// AB-MCTS scorer (typically `prm_lite::AuditorVerifierScore::weighted_score`),
/// and `consistent` from the deterministic auditor syntactic-pass flag.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LToTCandidate {
    /// Caller-assigned candidate id (used in SSB row payloads).
    pub id: String,
    /// Combined utility score in `[0.0, 1.0]`.
    pub utility: f32,
    /// True when the auditor returns ALLOW (deterministic-syntactic mode).
    pub consistent: bool,
}

/// Output of [`lateral_branch`]: the partitioned mainline / lateral lists.
///
/// `mainline` is the top-`N_MAINLINE` by utility; `lateral` is the next up
/// to `N_LATERAL` candidates that are logically consistent but did not
/// make the utility cut. Both lists preserve descending-utility order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LToTSplit {
    pub mainline: Vec<LToTCandidate>,
    pub lateral: Vec<LToTCandidate>,
}

/// Partition the input candidates into the mainline + lateral lists per the
/// E-19 LToT-lite rule.
///
/// The function is total: empty input yields empty output; fewer than
/// `N_MAINLINE` candidates yields a shorter mainline (no padding); zero
/// consistent low-utility candidates yields an empty lateral list. Caller
/// is expected to dispatch the lateral list one extra ply only (the
/// pruning is the caller's responsibility — keeping this pure makes it
/// trivial to unit-test).
#[must_use]
pub fn lateral_branch(mut candidates: Vec<LToTCandidate>) -> LToTSplit {
    // Sort by utility descending. Stable so equal-utility candidates retain
    // their input order (deterministic across runs).
    candidates.sort_by(|a, b| b.utility.partial_cmp(&a.utility).unwrap_or(std::cmp::Ordering::Equal));

    let mainline_len = candidates.len().min(N_MAINLINE);
    let mainline: Vec<LToTCandidate> = candidates.iter().take(mainline_len).cloned().collect();

    let lateral: Vec<LToTCandidate> = candidates
        .into_iter()
        .skip(mainline_len)
        .filter(|c| c.consistent)
        .take(N_LATERAL)
        .collect();

    LToTSplit { mainline, lateral }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(id: &str, utility: f32, consistent: bool) -> LToTCandidate {
        LToTCandidate {
            id: id.to_string(),
            utility,
            consistent,
        }
    }

    #[test]
    fn empty_input_yields_empty_split() {
        let split = lateral_branch(vec![]);
        assert!(split.mainline.is_empty());
        assert!(split.lateral.is_empty());
    }

    #[test]
    fn three_candidates_all_mainline_no_lateral() {
        let split = lateral_branch(vec![c("a", 0.9, true), c("b", 0.5, true), c("c", 0.3, true)]);
        assert_eq!(split.mainline.len(), 3);
        assert!(split.lateral.is_empty());
    }

    #[test]
    fn five_candidates_three_mainline_two_lateral() {
        let split = lateral_branch(vec![
            c("a", 0.9, true),
            c("b", 0.8, true),
            c("c", 0.7, true),
            c("d", 0.2, true), // low utility but consistent
            c("e", 0.1, true), // low utility but consistent
        ]);
        assert_eq!(split.mainline.len(), N_MAINLINE);
        assert_eq!(split.lateral.len(), N_LATERAL);
        let mainline_ids: Vec<_> = split.mainline.iter().map(|c| c.id.clone()).collect();
        assert_eq!(mainline_ids, vec!["a", "b", "c"]);
        let lateral_ids: Vec<_> = split.lateral.iter().map(|c| c.id.clone()).collect();
        assert_eq!(lateral_ids, vec!["d", "e"]);
    }

    #[test]
    fn inconsistent_low_utility_excluded_from_lateral() {
        let split = lateral_branch(vec![
            c("a", 0.9, true),
            c("b", 0.8, true),
            c("c", 0.7, true),
            c("d", 0.2, false), // low utility AND inconsistent -> excluded
            c("e", 0.1, true),  // low utility but consistent -> lateral
        ]);
        assert_eq!(split.mainline.len(), N_MAINLINE);
        assert_eq!(split.lateral.len(), 1);
        assert_eq!(split.lateral[0].id, "e");
    }

    #[test]
    fn over_n_lateral_truncated() {
        // 3 mainline + 4 consistent low-utility -> only N_LATERAL=2 survive.
        let split = lateral_branch(vec![
            c("a", 0.9, true),
            c("b", 0.8, true),
            c("c", 0.7, true),
            c("d", 0.3, true),
            c("e", 0.2, true),
            c("f", 0.1, true),
            c("g", 0.05, true),
        ]);
        assert_eq!(split.lateral.len(), N_LATERAL);
        assert_eq!(split.lateral[0].id, "d");
        assert_eq!(split.lateral[1].id, "e");
    }
}
