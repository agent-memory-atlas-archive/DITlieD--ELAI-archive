//! E-17 SP2 task 4 — DEEP candidate pool cap + RRF quality gate.
//!
//! Two pure-function primitives wired into `pipeline.rs::rrf_rerank`:
//!
//! - [`deep_pool_cap`] — cap each input stage's results vector to at most
//!   `max` candidates BEFORE the RRF accumulator runs. Returns the total
//!   number of dropped candidates so callers can emit a `deep_pool_capped`
//!   SSB beacon. Applied for DEEP tier only; STANDARD + FAST untouched.
//!
//! - [`rrf_quality_gate`] — drop fused candidates whose final RRF score
//!   (post-freshness multiplier, sort-ready) falls below `floor`. Returns
//!   the number of dropped entries so callers can emit a
//!   `rrf_quality_gate_fired` SSB beacon.
//!
//! Ordering rule (HARD): cap runs BEFORE the RRF sort + accumulator; floor
//! runs AFTER the freshness multiplier. Calling them in the wrong order
//! breaks tie-break determinism (cap-after-sort) or over-prunes freshness-
//! rescued candidates (floor-before-multiplier). See
//! `crates/knowledge/tests/deep_pool_cap_contract.rs` for the trio of
//! contract assertions.
//!
//! Honest framing (per E-17-grounding-resolved.md):
//! - `DEEP_CAP_DEFAULT = 100`: prior-work-typical heuristic; avoids the
//!   phantom-hit zone observed at K=5000 (arxiv:2411.11767 S4.SS2.p2).
//!   The paper does NOT mandate this specific value (S4 fn1 only).
//! - `RRF_FLOOR_DEFAULT = 0.005`: ELAI design choice; validated by the SP6
//!   eval harness. k=60 is grounded in Cormack et al. SIGIR 2009.

use std::collections::HashMap;

use crate::arag::RetrievalResult;
use crate::pipeline::PipelineStage;

/// E-17 SP2: default upper bound on the per-stage candidate pool that
/// enters the RRF accumulator on the DEEP tier. Prior-work-typical
/// heuristic per arxiv:2411.11767 S4 fn1; the paper does NOT mandate this
/// specific value.
pub const DEEP_CAP_DEFAULT: usize = 100;

/// E-17 SP2: default RRF score floor below which fused candidates are
/// dropped post-freshness-multiplier. ELAI design choice (back-derived
/// from k=60 arithmetic: a single ranker at rank 140 produces score
/// 1/(60+140) = 0.005). Validated by the SP6 eval harness.
pub const RRF_FLOOR_DEFAULT: f64 = 0.005;

/// Cormack et al. SIGIR 2009 default. Already shipping in the RRF
/// accumulator at pipeline.rs:816 — this constant is the named-constant
/// twin (codeRules baseline: no magic numbers).
pub const RRF_K_CONSTANT: f64 = 60.0;

/// Cap each input stage's candidate vector to at most `max` entries.
///
/// Truncates `stage.results` in-place. Returns the total number of
/// candidates dropped across all stages so the caller can emit one
/// `deep_pool_capped` SSB beacon per query.
///
/// When `max == 0` the function is a no-op (returns 0). When every stage
/// already has `<= max` candidates the function returns 0. Stages with
/// no results are unchanged.
///
/// MUST run BEFORE the RRF accumulator: capping post-sort would break
/// tie-break determinism on the fused output.
pub fn deep_pool_cap(stages: &mut Vec<PipelineStage>, max: usize) -> usize {
    if max == 0 {
        return 0;
    }
    let mut dropped_total: usize = 0;
    for stage in stages.iter_mut() {
        if stage.results.len() > max {
            let dropped = stage.results.len() - max;
            stage.results.truncate(max);
            dropped_total = dropped_total.saturating_add(dropped);
        }
    }
    dropped_total
}

/// Drop fused candidates whose RRF score (post-freshness multiplier) is
/// strictly below `floor`. Returns the number of dropped entries so the
/// caller can emit one `rrf_quality_gate_fired` SSB beacon per query.
///
/// When `floor <= 0.0` the function is a no-op (returns 0).
///
/// MUST run AFTER the freshness multiplier has been applied. Running it
/// before the multiplier would prune candidates whose freshness boost
/// would have rescued them above the floor.
pub fn rrf_quality_gate(
    scores: &mut HashMap<String, (f64, RetrievalResult)>,
    floor: f64,
) -> usize {
    if floor <= 0.0 {
        return 0;
    }
    let before = scores.len();
    scores.retain(|_, (score, _)| *score >= floor);
    before.saturating_sub(scores.len())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_result(id: &str, score: f64) -> RetrievalResult {
        RetrievalResult {
            chunk_id: id.to_string(),
            snippet: format!("snippet for {id}"),
            score,
            tool_used: "test".to_string(),
        }
    }

    fn make_stage(name: &str, n: usize) -> PipelineStage {
        let results: Vec<RetrievalResult> = (0..n)
            .map(|i| make_result(&format!("{name}-{i}"), 1.0))
            .collect();
        PipelineStage {
            stage: name.to_string(),
            results,
            latency_ms: 0,
            circuit_breaker_state: None,
            error: None,
        }
    }

    #[test]
    fn cap_drops_excess_per_stage() {
        let mut stages = vec![make_stage("bm25", 150), make_stage("vector", 200)];
        let dropped = deep_pool_cap(&mut stages, DEEP_CAP_DEFAULT);
        // 150 -> 100 (drop 50) and 200 -> 100 (drop 100). total 150.
        assert_eq!(dropped, 150);
        assert_eq!(stages[0].results.len(), DEEP_CAP_DEFAULT);
        assert_eq!(stages[1].results.len(), DEEP_CAP_DEFAULT);
    }

    #[test]
    fn cap_zero_is_noop() {
        let mut stages = vec![make_stage("bm25", 50)];
        let dropped = deep_pool_cap(&mut stages, 0);
        assert_eq!(dropped, 0);
        assert_eq!(stages[0].results.len(), 50);
    }

    #[test]
    fn cap_under_threshold_is_noop() {
        let mut stages = vec![make_stage("bm25", 50)];
        let dropped = deep_pool_cap(&mut stages, DEEP_CAP_DEFAULT);
        assert_eq!(dropped, 0);
        assert_eq!(stages[0].results.len(), 50);
    }

    #[test]
    fn floor_drops_below_threshold() {
        let mut scores: HashMap<String, (f64, RetrievalResult)> = HashMap::new();
        scores.insert("a".into(), (0.010, make_result("a", 0.010)));
        scores.insert("b".into(), (0.004, make_result("b", 0.004)));
        scores.insert("c".into(), (0.006, make_result("c", 0.006)));
        scores.insert("d".into(), (0.001, make_result("d", 0.001)));
        let dropped = rrf_quality_gate(&mut scores, RRF_FLOOR_DEFAULT);
        // 0.004 + 0.001 = 2 dropped; 0.010 + 0.006 = 2 retained.
        assert_eq!(dropped, 2);
        assert_eq!(scores.len(), 2);
        assert!(scores.contains_key("a"));
        assert!(scores.contains_key("c"));
    }

    #[test]
    fn floor_zero_is_noop() {
        let mut scores: HashMap<String, (f64, RetrievalResult)> = HashMap::new();
        scores.insert("a".into(), (0.001, make_result("a", 0.001)));
        let dropped = rrf_quality_gate(&mut scores, 0.0);
        assert_eq!(dropped, 0);
        assert_eq!(scores.len(), 1);
    }

    #[test]
    fn floor_at_boundary_is_inclusive() {
        let mut scores: HashMap<String, (f64, RetrievalResult)> = HashMap::new();
        scores.insert("exact".into(), (RRF_FLOOR_DEFAULT, make_result("exact", RRF_FLOOR_DEFAULT)));
        let dropped = rrf_quality_gate(&mut scores, RRF_FLOOR_DEFAULT);
        // Score equal to floor is retained (`>= floor`).
        assert_eq!(dropped, 0);
        assert_eq!(scores.len(), 1);
    }

    #[test]
    fn cap_and_floor_constants_are_in_paper_envelope() {
        // arxiv:2411.11767 testable invariants: deep_cap is meaningful in
        // the [20, 200] window (S4 fn1 + S4.SS2.p2). RRF_FLOOR is non-zero
        // and below the practical single-ranker max (1/(60+1) ~= 0.0164).
        assert!(DEEP_CAP_DEFAULT >= 20 && DEEP_CAP_DEFAULT <= 200);
        assert!(RRF_FLOOR_DEFAULT > 0.0 && RRF_FLOOR_DEFAULT < 0.0164);
        assert!((RRF_K_CONSTANT - 60.0).abs() < f64::EPSILON);
    }
}
