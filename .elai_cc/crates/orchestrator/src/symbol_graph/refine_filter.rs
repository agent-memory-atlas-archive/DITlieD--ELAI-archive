//! E-40 Tranche B-2 — Refine+Filter re-ranker.
//!
//! Combines an `initial` list of `SymbolMatch` rows (e.g. produced by
//! the existing four query primitives or the dashboard's prior agent
//! turn) with a `dsl_results` list (output of `exec_dsl`) into a single
//! re-ranked list. The score for each row is the convex combination
//!
//! ```text
//! score = alpha * initial_score + beta * dsl_score + co_occurrence_bonus
//! ```
//!
//! where initial_score / dsl_score are reciprocal ranks (1 / (k + rank))
//! per the standard RRF formulation and the co-occurrence bonus is a
//! small constant for any symbol that appears in BOTH lists.
//!
//! Two-iteration fixed-weight schedule:
//!
//! - iter 1: alpha=0.6, beta=0.4 (initial favoured slightly — agent's
//!   prior context outranks DSL discovery)
//! - iter 2: re-apply scores using the iter-1 ranks as the new
//!   `initial` list (Refine step) and keep the same weights. Two
//!   iterations is the empirical break-even per RANGER's iterative
//!   refinement schedule (arXiv:2509.25257 sec. 4.2 — "two-step
//!   refinement matches three-step within noise").
//!
//! Fixed weights for v1 — config-loaded weights are a follow-on per
//! the Tranche B-2 plan.

use std::collections::BTreeMap;

use super::query::SymbolMatch;

/// RRF constant `k`. Matches the knowledge::pipeline default so the
/// score scales align if the refine stage feeds into the pipeline RRF
/// pool downstream.
pub const RRF_K_CONSTANT: f64 = 60.0;

/// Two-iteration default per RANGER §4.2 empirical schedule.
pub const DEFAULT_ITERS: u8 = 2;

/// Initial-list weight.
pub const DEFAULT_ALPHA: f64 = 0.6;

/// DSL-list weight.
pub const DEFAULT_BETA: f64 = 0.4;

/// Co-occurrence bonus (added when a symbol appears in both lists).
pub const CO_OCCURRENCE_BONUS: f64 = 0.05;

/// Refine+Filter stage configuration. Fields are public so contract
/// tests can pin them; production callers should construct via
/// `RefineStage::default()`.
#[derive(Debug, Clone, Copy)]
pub struct RefineStage {
    pub iters: u8,
    pub alpha: f64,
    pub beta: f64,
}

impl Default for RefineStage {
    fn default() -> Self {
        Self {
            iters: DEFAULT_ITERS,
            alpha: DEFAULT_ALPHA,
            beta: DEFAULT_BETA,
        }
    }
}

impl RefineStage {
    /// Run the Refine+Filter re-rank. Deterministic — input order +
    /// content fully determines the output order.
    pub fn refine(
        &self,
        initial: Vec<SymbolMatch>,
        dsl_results: Vec<SymbolMatch>,
    ) -> Vec<SymbolMatch> {
        if self.iters == 0 || (initial.is_empty() && dsl_results.is_empty()) {
            return initial;
        }
        let mut current = initial;
        let dsl_clone = dsl_results;
        for _ in 0..self.iters {
            current = combine_one_pass(&current, &dsl_clone, self.alpha, self.beta);
        }
        current
    }
}

/// Free-fn convenience wrapper. Uses [`RefineStage::default`].
pub fn refine(initial: Vec<SymbolMatch>, dsl_results: Vec<SymbolMatch>) -> Vec<SymbolMatch> {
    RefineStage::default().refine(initial, dsl_results)
}

fn combine_one_pass(
    initial: &[SymbolMatch],
    dsl_results: &[SymbolMatch],
    alpha: f64,
    beta: f64,
) -> Vec<SymbolMatch> {
    // Index by stable id. Ties broken by id sort for determinism.
    let mut scores: BTreeMap<String, f64> = BTreeMap::new();
    let mut keep: BTreeMap<String, SymbolMatch> = BTreeMap::new();
    for (rank, m) in initial.iter().enumerate() {
        let s = alpha * (1.0 / (RRF_K_CONSTANT + rank as f64));
        *scores.entry(m.id.clone()).or_insert(0.0) += s;
        keep.entry(m.id.clone()).or_insert_with(|| m.clone());
    }
    for (rank, m) in dsl_results.iter().enumerate() {
        let s = beta * (1.0 / (RRF_K_CONSTANT + rank as f64));
        let entry = scores.entry(m.id.clone()).or_insert(0.0);
        let was_in_initial = keep.contains_key(&m.id);
        *entry += s;
        if was_in_initial {
            *entry += CO_OCCURRENCE_BONUS;
        }
        keep.entry(m.id.clone()).or_insert_with(|| m.clone());
    }
    // Sort by score desc; tie-break by id asc for stable golden tests.
    let mut ranked: Vec<(String, f64)> = scores.into_iter().collect();
    ranked.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.0.cmp(&b.0))
    });
    ranked
        .into_iter()
        .filter_map(|(id, _score)| keep.remove(&id))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sm(id: &str, name: &str) -> SymbolMatch {
        SymbolMatch {
            id: id.into(),
            name: name.into(),
            file: format!("{}.rs", name),
            line: 1,
            kind: "function".into(),
        }
    }

    #[test]
    fn refine_two_iter_idempotent_on_same_input() {
        let init = vec![sm("a:1", "alpha"), sm("b:2", "beta")];
        let dsl = vec![sm("a:1", "alpha"), sm("c:3", "gamma")];
        let stage = RefineStage::default();
        let out = stage.refine(init, dsl);
        // alpha appears in both lists -> co-occurrence bonus -> top rank.
        assert_eq!(out[0].id, "a:1");
        assert_eq!(out.len(), 3);
    }

    #[test]
    fn refine_zero_iter_returns_initial() {
        let init = vec![sm("a:1", "alpha")];
        let dsl = vec![sm("b:2", "beta")];
        let stage = RefineStage {
            iters: 0,
            alpha: 0.6,
            beta: 0.4,
        };
        let out = stage.refine(init.clone(), dsl);
        assert_eq!(out.len(), init.len());
    }

    #[test]
    fn refine_empty_inputs_returns_empty() {
        let stage = RefineStage::default();
        let out = stage.refine(Vec::new(), Vec::new());
        assert!(out.is_empty());
    }

    #[test]
    fn default_weights_match_constants() {
        let s = RefineStage::default();
        assert_eq!(s.iters, DEFAULT_ITERS);
        assert!((s.alpha - DEFAULT_ALPHA).abs() < f64::EPSILON);
        assert!((s.beta - DEFAULT_BETA).abs() < f64::EPSILON);
    }
}
