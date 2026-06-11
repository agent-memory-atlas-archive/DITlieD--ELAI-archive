//! H-46: Clade-Metaproductivity (CMP) parent-selection key for the DGM config archive.
//!
//! HGM (arXiv:2510.21614) identifies the Metaproductivity-Performance Mismatch:
//! a node's IMMEDIATE score has a surprisingly weak correlation with its long-term
//! potential to generate better descendants. CMP selects the node whose ENTIRE
//! descendant lineage (clade) produced the most-improving successors, rather than
//! the single highest-scoring node.
//!
//! This module:
//!   - Defines [`CladeScore`] carrying the CMP aggregate, clade size, and immediate
//!     sigmoid score (for the no-lineage fallback).
//!   - Defines [`SelectionKey`] to distinguish CMP-based from immediate-score
//!     fallback selection.
//!   - Provides [`clade_metaproductivity`]: given the full archive variant slice
//!     and a map of per-node execution-grounded fitness deltas (from H-43
//!     [`crate::execution_fitness`]), computes the CMP for each candidate.
//!
//! VD-1 compliance: CMP aggregates execution-grounded deltas (H-43 pass-rate
//! signal), NEVER byte-hash deltas (the old GAP-1 `derived_alpha` path).
//!
//! Spec: docs/thefinalglance/H-plan/H-46_clade_metaproductivity_selection.md
//! Research: HGM arXiv:2510.21614, metauto-ai/HGM (ICLR 2026 oral).

use std::collections::HashMap;

use crate::self_mod_types::{ConfigVariant, VariantId};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Minimum clade size required to override the immediate-score fallback.
/// A node with zero measured descendants has no lineage signal — use sigmoid.
pub const CMP_MIN_CLADE_SIZE: usize = 1;

/// Aggregation: sum of measured descendant execution-grounded fitness deltas
/// (HGM uses aggregate productive success of the clade; we use the mean delta
/// to normalise for clade size so large clades don't dominate simply by count).
/// Mean is reproducible and monotone in clade quality.
pub const CMP_AGGREGATE: CmpAggregate = CmpAggregate::MeanDelta;

/// How CMP aggregates the descendant deltas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmpAggregate {
    /// Arithmetic mean of descendant execution-grounded fitness deltas.
    /// Normalises for clade size (large clades don't auto-win). Deterministic.
    MeanDelta,
    /// Sum of descendant deltas. Not normalised — included for ablation.
    SumDelta,
}

// ---------------------------------------------------------------------------
// Output types
// ---------------------------------------------------------------------------

/// Which selection key was used for a given candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionKey {
    /// CMP override fired: clade_size >= [`CMP_MIN_CLADE_SIZE`] and CMP is
    /// the load-bearing signal.
    CladeMetaproductivity,
    /// Fallback: node has no measured descendants, or `force_immediate` is
    /// set. Selection falls back to the immediate sigmoid score.
    ImmediateScoreFallback,
}

impl SelectionKey {
    /// Stable wire string for the SSB kind arm.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CladeMetaproductivity => "clade_metaproductivity",
            Self::ImmediateScoreFallback => "immediate_score_fallback",
        }
    }
}

/// Output of [`clade_metaproductivity`] for one candidate node.
///
/// `cmp` is the aggregated execution-grounded delta over the node's descendant
/// clade. `clade_size` is the count of descendants with a measured delta.
/// `immediate_score` is the sigmoid score computed from the node's own `alpha`.
///
/// When `key` is [`SelectionKey::ImmediateScoreFallback`], `cmp` equals
/// `immediate_score` (the caller uses the same value through the same weight
/// path, so no special-casing is required in the CDF sampler).
#[derive(Debug, Clone, PartialEq)]
pub struct CladeScore {
    /// The selection weight: CMP aggregate OR immediate sigmoid score.
    /// Always strictly positive (sigmoid lower bound asymptotic 0 + cmp floor 0).
    pub cmp: f64,
    /// Number of descendants that contributed a measured delta to the CMP.
    /// Zero means the node has no lineage yet.
    pub clade_size: usize,
    /// Sigmoid score of this node's own `alpha` (retained for the SSB row
    /// and for the ImmediateScoreFallback arm).
    pub immediate_score: f64,
    /// Which key was used to produce `cmp`.
    pub key: SelectionKey,
}

// ---------------------------------------------------------------------------
// Core computation
// ---------------------------------------------------------------------------

/// Compute CMP for each variant in `variants`.
///
/// `fitness_deltas` maps `variant_id -> execution-grounded fitness delta`
/// (H-43 signal). A positive delta means this variant improved over its parent;
/// a negative delta means it regressed.
///
/// The CMP for variant `v` is the mean delta of all variants whose `parent_id`
/// traces back to `v` (direct children only for Tier-1; multi-generation
/// lineage is a Tier-2 checkbox per W-10). A variant with no measured
/// descendants uses `ImmediateScoreFallback`.
///
/// Returns a `Vec` parallel to `variants` (same index, same order).
///
/// # Determinism
///
/// Pure function. Given the same `variants` and `fitness_deltas` the output
/// is byte-identical (no randomness, no wall-clock). GD-2 compliant.
///
/// # VD-1 compliance
///
/// `fitness_deltas` MUST come from H-43 [`crate::execution_fitness::execution_grounded_fitness`].
/// Passing byte-hash values is a stub-ban violation. The caller is responsible
/// for supplying the correct signal; this function does not re-derive it.
pub fn clade_metaproductivity(
    variants: &[ConfigVariant],
    fitness_deltas: &HashMap<VariantId, f64>,
) -> Vec<CladeScore> {
    // Build a map variant_id -> index for O(1) parent lookup.
    let id_to_idx: HashMap<&str, usize> = variants
        .iter()
        .enumerate()
        .map(|(i, v)| (v.variant_id.as_str(), i))
        .collect();

    // For each variant, collect the fitness deltas of its direct children.
    // Tier-1: direct children only (first-generation descendants).
    // Tier-2 (multi-generation) is separately gated per W-10.
    let mut children_deltas: Vec<Vec<f64>> = vec![Vec::new(); variants.len()];
    for v in variants {
        if let Some(parent_id) = v.parent_id.as_deref() {
            if let Some(&parent_idx) = id_to_idx.get(parent_id) {
                if let Some(&delta) = fitness_deltas.get(v.variant_id.as_str()) {
                    children_deltas[parent_idx].push(delta);
                }
            }
        }
    }

    // Compute CladeScore for each variant.
    variants
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let immediate_score = f64::from(crate::config_archive::sigmoid_perf_score(v.score));
            let deltas = &children_deltas[i];
            if deltas.len() < CMP_MIN_CLADE_SIZE {
                // No measured descendants: fall back to immediate sigmoid score.
                CladeScore {
                    cmp: immediate_score,
                    clade_size: 0,
                    immediate_score,
                    key: SelectionKey::ImmediateScoreFallback,
                }
            } else {
                let clade_size = deltas.len();
                let aggregate = match CMP_AGGREGATE {
                    CmpAggregate::MeanDelta => {
                        let sum: f64 = deltas.iter().sum();
                        sum / clade_size as f64
                    }
                    CmpAggregate::SumDelta => deltas.iter().sum(),
                };
                // CMP is the aggregate delta offset from the immediate score.
                // We add immediate_score as the floor so the weight stays
                // positive even when the clade regresses
                // (sigmoid lower bound is always > 0; aggregate can be negative).
                //
                // cmp = max(1e-9, immediate_score + aggregate)
                //
                // This keeps the CDF sampler strictly positive (DGM invariant)
                // and preserves the direction: a clade with improving descendants
                // (positive aggregate) lifts the weight above the immediate score;
                // a regressing clade (negative aggregate) pushes it below.
                let raw = immediate_score + aggregate;
                let cmp = raw.max(1e-9_f64);
                CladeScore {
                    cmp,
                    clade_size,
                    immediate_score,
                    key: SelectionKey::CladeMetaproductivity,
                }
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::self_mod_layer1a::PipelineMode;
    use crate::self_mod_types::{ConfigVariant, VariantStatus};

    fn variant(id: &str, parent: Option<&str>, score: f32) -> ConfigVariant {
        ConfigVariant {
            variant_id: id.to_string(),
            parent_id: parent.map(|s| s.to_string()),
            score,
            children_count: 0,
            config_blob: vec![],
            created_at: 0,
            pipeline_mode: PipelineMode::Greenfield,
            status: VariantStatus::Archived,
            pipeline_outcomes: vec![],
        }
    }

    /// Node A: high immediate score (0.9), children regress -> bad CMP.
    /// Node B: mediocre immediate score (0.4), children improve -> good CMP.
    ///
    /// sigmoid(0.9) ≈ 0.9820, sigmoid(0.4) ≈ 0.2689.
    /// Delta margin -0.5/+0.5 is large enough to cross the sigmoid gap:
    ///   cmp(A) = 0.9820 + (-0.5) = 0.4820
    ///   cmp(B) = 0.2689 + (+0.5) = 0.7689
    #[test]
    fn cmp_selects_b_over_a_when_clades_differ() {
        let a = variant("A", None, 0.9);
        let b = variant("B", None, 0.4);
        // A's child regresses strongly
        let a_child = variant("A1", Some("A"), 0.3);
        // B's child improves strongly
        let b_child = variant("B1", Some("B"), 0.8);
        let variants = vec![a, b, a_child, b_child];

        let mut deltas = HashMap::new();
        deltas.insert("A1".to_string(), -0.5_f64); // A's child regressed strongly
        deltas.insert("B1".to_string(), 0.5_f64);  // B's child improved strongly

        let scores = clade_metaproductivity(&variants, &deltas);

        let a_score = &scores[0];
        let b_score = &scores[1];

        // Both should have clade size 1 (one direct child with a measured delta).
        assert_eq!(a_score.clade_size, 1, "A clade_size");
        assert_eq!(b_score.clade_size, 1, "B clade_size");

        // Both use CladeMetaproductivity key.
        assert_eq!(a_score.key, SelectionKey::CladeMetaproductivity);
        assert_eq!(b_score.key, SelectionKey::CladeMetaproductivity);

        // Despite A having a higher immediate score, B's cmp should exceed A's.
        assert!(
            b_score.cmp > a_score.cmp,
            "cmp(B)={:.4} must exceed cmp(A)={:.4} (CMP over execution deltas wins over immediate)",
            b_score.cmp,
            a_score.cmp,
        );

        // Verify immediate scores are from sigmoid.
        assert!(a_score.immediate_score > b_score.immediate_score,
            "A immediate={:.4} must exceed B immediate={:.4}",
            a_score.immediate_score, b_score.immediate_score);
    }

    #[test]
    fn no_descendants_uses_fallback() {
        let root = variant("root", None, 0.7);
        let variants = vec![root];
        let scores = clade_metaproductivity(&variants, &HashMap::new());
        assert_eq!(scores[0].clade_size, 0);
        assert_eq!(scores[0].key, SelectionKey::ImmediateScoreFallback);
        // cmp == immediate_score on fallback path.
        assert!((scores[0].cmp - scores[0].immediate_score).abs() < 1e-10);
    }

    #[test]
    fn deterministic_under_same_input() {
        let a = variant("A", None, 0.9);
        let b = variant("B", None, 0.4);
        let a_child = variant("A1", Some("A"), 0.3);
        let b_child = variant("B1", Some("B"), 0.8);
        let variants = vec![a, b, a_child, b_child];
        let mut deltas = HashMap::new();
        deltas.insert("A1".to_string(), -0.5_f64);
        deltas.insert("B1".to_string(), 0.5_f64);
        let r1 = clade_metaproductivity(&variants, &deltas);
        let r2 = clade_metaproductivity(&variants, &deltas);
        for (s1, s2) in r1.iter().zip(r2.iter()) {
            assert_eq!(s1.cmp, s2.cmp, "CMP must be deterministic (GD-2)");
            assert_eq!(s1.clade_size, s2.clade_size);
        }
    }

    #[test]
    fn regressing_clade_floors_at_positive() {
        let a = variant("A", None, 0.01); // nearly-zero immediate score
        let a_child = variant("A1", Some("A"), 0.0);
        let variants = vec![a, a_child];
        let mut deltas = HashMap::new();
        deltas.insert("A1".to_string(), -10.0_f64); // extreme regression
        let scores = clade_metaproductivity(&variants, &deltas);
        // cmp must stay positive (>= 1e-9) for CDF sampler invariant.
        assert!(scores[0].cmp >= 1e-9, "cmp must be >= 1e-9 even for extreme regression");
    }
}
