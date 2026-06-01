//! E-54 generator-rotation enforcement (task 7).
//!
//! Per arXiv:2412.08905 Phi-4 (50 distinct synthetic-data techniques for
//! 400B tokens) + arXiv:2509.00072 Test-of-Time (synthesis can OBSCURE
//! contamination):
//!   - corpus growth REQUIRES at least 3 distinct generators
//!   - each generator must contribute >= 30% of the new batch
//!
//! When expanding from N to M tasks, the caller passes the per-task
//! [`crate::task_schema::Provenance`] entries; this module aggregates the
//! distribution and produces a verdict.

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::task_schema::Task;

/// E-54 minimum share per generator. Below this -> violation.
pub const MIN_GENERATOR_SHARE: f64 = 0.30;

/// E-54 minimum distinct generator count.
pub const MIN_GENERATORS: usize = 3;

/// Per-generator share aggregation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DistributionResult {
    /// Generator label -> count of tasks attributed to it.
    pub per_generator: FxHashMap<String, usize>,
    /// Smallest non-zero share in the corpus, in `[0.0, 1.0]`.
    pub min_share: f64,
    /// True when (i) `per_generator.len() < MIN_GENERATORS` OR
    /// (ii) `min_share < MIN_GENERATOR_SHARE`.
    pub violated: bool,
    /// Total tasks counted (denominator).
    pub total: usize,
}

/// Compute the distribution from a slice of tasks.
#[must_use]
pub fn verify_distribution(tasks: &[Task]) -> DistributionResult {
    let mut per_generator: FxHashMap<String, usize> = FxHashMap::default();
    for t in tasks {
        *per_generator.entry(t.provenance.generator.clone()).or_insert(0) += 1;
    }
    compute_distribution(per_generator, tasks.len())
}

/// Compute the distribution from a pre-aggregated count map. Convenience
/// constructor for tests + migrate paths that already have totals on hand.
#[must_use]
pub fn compute_distribution(
    per_generator: FxHashMap<String, usize>,
    total: usize,
) -> DistributionResult {
    if total == 0 {
        return DistributionResult {
            per_generator,
            min_share: 0.0,
            violated: true,
            total,
        };
    }
    let mut min_share = 1.0_f64;
    for &count in per_generator.values() {
        if count == 0 {
            continue;
        }
        let share = count as f64 / total as f64;
        if share < min_share {
            min_share = share;
        }
    }
    let violated = per_generator.len() < MIN_GENERATORS || min_share < MIN_GENERATOR_SHARE;
    DistributionResult {
        per_generator,
        min_share,
        violated,
        total,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generator_rotation_minimum_distribution() {
        // 2 generators -> violated even at 50/50.
        let mut m = FxHashMap::default();
        m.insert("a".to_string(), 5);
        m.insert("b".to_string(), 5);
        let r = compute_distribution(m, 10);
        assert!(r.violated, "<3 generators must violate");
    }

    #[test]
    fn three_generators_with_one_at_20_pct_violated() {
        let mut m = FxHashMap::default();
        m.insert("a".to_string(), 4);
        m.insert("b".to_string(), 4);
        m.insert("c".to_string(), 2);
        let r = compute_distribution(m, 10);
        assert!(r.violated, "20% generator share must violate");
    }

    #[test]
    fn three_generators_at_floor_pass() {
        let mut m = FxHashMap::default();
        m.insert("a".to_string(), 30);
        m.insert("b".to_string(), 35);
        m.insert("c".to_string(), 35);
        let r = compute_distribution(m, 100);
        assert!(!r.violated, "30/35/35 must pass");
    }

    #[test]
    fn empty_corpus_violates() {
        let r = compute_distribution(FxHashMap::default(), 0);
        assert!(r.violated);
    }
}
