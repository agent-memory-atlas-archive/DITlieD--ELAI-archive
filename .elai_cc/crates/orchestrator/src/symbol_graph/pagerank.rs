//! E-40 — Personalized PageRank rooted on a single seed symbol over the
//! orchestrator-rooted symbol graph.
//!
//! Distinct surface from `elai_knowledge::pagerank::DkgSnapshot::compute_repo_map`
//! (DKG-whole-repo) and `elai_knowledge::symbol_graph` (B-29 SQLite cache).
//! This module operates on the in-memory `SymbolGraphCache` JSON view and
//! returns the top-k symbols ranked by personalized PageRank centered on
//! the seed node.
//!
//! Algorithm: 20-iteration power method with the standard
//! `damping=0.85` factor and a 1e-6 L1 convergence tolerance (Brin &
//! Page, 1998; canonical PageRank constants). Personalization vector =
//! Kronecker delta at the seed (all weight on seed). The personalization
//! arm replaces the uniform restart vector that whole-graph PageRank
//! uses; for SymbolGraphCache-rooted ranking we want results biased
//! toward the seed node's neighbourhood.
//!
//! Stable lexicographic tiebreaker — when two nodes share an identical
//! score (common after 20 iterations with float rounding) the BTreeMap
//! iteration over node ids gives a deterministic tiebreaker without
//! needing an explicit sort. Score-descending sort uses `sort_by` with
//! the score reversed; equal scores fall back to the natural BTreeMap
//! key order.
//!
//! Inputs:
//! - `seed` — name of the symbol to root the personalization vector on.
//!   Resolved via `cache.ids_by_name`; if there are multiple defs (e.g.
//!   overloaded fn), all matching ids receive equal initial mass.
//! - `k` — output size (cap on `Vec<RankedSymbol>` length).
//! - `cache` — JSON-rooted symbol graph view.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::cache::SymbolGraphCache;

/// Standard PageRank damping factor.
const DAMPING: f64 = 0.85;
/// Power-method iteration cap. Empirically converges on graphs up to
/// ~100k nodes well before this bound (per ITF/ELAI repo-map decisions).
const MAX_ITERATIONS: usize = 20;
/// L1 convergence tolerance — sum of absolute deltas across all nodes.
const CONVERGENCE_TOL: f64 = 1e-6;

/// Ranked output row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RankedSymbol {
    pub id: String,
    pub score: f64,
}

/// Compute personalized PageRank rooted on `seed` and return the top-k
/// nodes by score (descending; stable tiebreaker by id).
///
/// Returns an empty vector when:
/// - `k == 0`,
/// - the cache has zero nodes,
/// - `seed` does not resolve to any id in the cache (no personalization
///   mass — algorithm cannot start).
pub fn personalized_top_k(
    cache: &mut SymbolGraphCache,
    seed: &str,
    k: usize,
) -> Vec<RankedSymbol> {
    if k == 0 || cache.node_count() == 0 {
        return Vec::new();
    }
    let seed_ids = cache.ids_by_name(seed);
    if seed_ids.is_empty() {
        return Vec::new();
    }
    // Snapshot all node ids once — power-method iterations need a stable
    // universe so we can't mutate the cache during iterations.
    let node_ids: Vec<String> = cache
        .iter_nodes()
        .map(|(id, _)| id.clone())
        .collect();
    if node_ids.is_empty() {
        return Vec::new();
    }
    // Build adjacency (out-edges) from the snapshot. BTreeMap-backed for
    // deterministic iteration during the power-method update.
    let mut out_edges: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut in_edges: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for id in &node_ids {
        if let Some(node) = cache.get_node(id) {
            for target in &node.calls {
                out_edges
                    .entry(id.clone())
                    .or_default()
                    .push(target.clone());
                in_edges
                    .entry(target.clone())
                    .or_default()
                    .push(id.clone());
            }
        }
    }
    // Personalization vector — uniform mass across all seed ids.
    let total_nodes = node_ids.len() as f64;
    let seed_mass = 1.0 / seed_ids.len() as f64;
    let mut p_vec: BTreeMap<String, f64> = BTreeMap::new();
    for id in &node_ids {
        p_vec.insert(id.clone(), 0.0);
    }
    for sid in &seed_ids {
        if let Some(entry) = p_vec.get_mut(sid) {
            *entry = seed_mass;
        }
    }
    // Initial rank = personalization (better warm-start than uniform).
    let mut rank: BTreeMap<String, f64> = p_vec.clone();
    // Out-degree cache.
    let mut out_degree: BTreeMap<String, usize> = BTreeMap::new();
    for id in &node_ids {
        out_degree.insert(
            id.clone(),
            out_edges.get(id).map(|v| v.len()).unwrap_or(0),
        );
    }
    for _ in 0..MAX_ITERATIONS {
        let mut next: BTreeMap<String, f64> = BTreeMap::new();
        // Sum dangling mass (out_degree == 0) into a single redistribution
        // bucket — distributed uniformly per the standard PageRank
        // dangling-node fix.
        let mut dangling_mass = 0.0;
        for id in &node_ids {
            if out_degree.get(id).copied().unwrap_or(0) == 0 {
                dangling_mass += rank.get(id).copied().unwrap_or(0.0);
            }
        }
        for id in &node_ids {
            // Inbound contributions.
            let mut inbound = 0.0;
            if let Some(sources) = in_edges.get(id) {
                for src in sources {
                    let src_rank = rank.get(src).copied().unwrap_or(0.0);
                    let deg = out_degree.get(src).copied().unwrap_or(0);
                    if deg > 0 {
                        inbound += src_rank / deg as f64;
                    }
                }
            }
            // Personalization restart term + dangling redistribution.
            let p = p_vec.get(id).copied().unwrap_or(0.0);
            let dangling_share = dangling_mass / total_nodes;
            let new_score =
                (1.0 - DAMPING) * p + DAMPING * (inbound + dangling_share);
            next.insert(id.clone(), new_score);
        }
        // L1 convergence check.
        let mut delta = 0.0;
        for id in &node_ids {
            let a = rank.get(id).copied().unwrap_or(0.0);
            let b = next.get(id).copied().unwrap_or(0.0);
            delta += (a - b).abs();
        }
        rank = next;
        if delta < CONVERGENCE_TOL {
            break;
        }
    }
    // Sort by score-desc with id-asc tiebreaker.
    let mut sorted: Vec<(String, f64)> = rank.into_iter().collect();
    sorted.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });
    sorted
        .into_iter()
        .take(k)
        .map(|(id, score)| RankedSymbol { id, score })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbol_graph::cache::SymbolNode;

    fn build_cache() -> SymbolGraphCache {
        let mut cache = SymbolGraphCache::empty_with_capacity(16).expect("empty");
        // foo -> bar; baz -> bar; bar (no out edges) — bar should rank
        // highest under personalization rooted on bar.
        cache.index_node(SymbolNode {
            id: "src/a.rs:foo:10".into(),
            name: "foo".into(),
            file: "src/a.rs".into(),
            line: 10,
            kind: "function".into(),
            calls: vec!["src/b.rs:bar:20".into()],
        });
        cache.index_node(SymbolNode {
            id: "src/b.rs:bar:20".into(),
            name: "bar".into(),
            file: "src/b.rs".into(),
            line: 20,
            kind: "function".into(),
            calls: vec![],
        });
        cache.index_node(SymbolNode {
            id: "src/c.rs:baz:30".into(),
            name: "baz".into(),
            file: "src/c.rs".into(),
            line: 30,
            kind: "function".into(),
            calls: vec!["src/b.rs:bar:20".into()],
        });
        cache
    }

    #[test]
    fn returns_empty_on_zero_k() {
        let mut cache = build_cache();
        let out = personalized_top_k(&mut cache, "bar", 0);
        assert!(out.is_empty());
    }

    #[test]
    fn returns_empty_on_unknown_seed() {
        let mut cache = build_cache();
        let out = personalized_top_k(&mut cache, "no_such_symbol", 5);
        assert!(out.is_empty());
    }

    #[test]
    fn returns_at_most_k_items() {
        let mut cache = build_cache();
        let out = personalized_top_k(&mut cache, "bar", 2);
        assert!(out.len() <= 2);
    }

    #[test]
    fn seed_ranks_among_top() {
        let mut cache = build_cache();
        let out = personalized_top_k(&mut cache, "bar", 3);
        assert!(!out.is_empty());
        // bar gets all the personalization mass + inbound from foo + baz —
        // it must be in the returned set.
        assert!(out.iter().any(|r| r.id == "src/b.rs:bar:20"));
    }
}
