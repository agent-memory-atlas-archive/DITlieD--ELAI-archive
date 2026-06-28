use rustc_hash::FxHashMap;
use std::path::{Path, PathBuf};

use crate::dkg::DkgEngine;
use crate::error::KnowledgeError;
use crate::SymbolKind;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct PageRankNodeInfo {
    pub node_id: i64,
    pub symbol_name: String,
    pub file_path: PathBuf,
    pub signature: String,
    pub kind: SymbolKind,
    pub ref_count: usize,
}

#[derive(Debug, Clone)]
pub struct DkgSnapshot {
    pub nodes: Vec<PageRankNodeInfo>,
    pub edges: Vec<(usize, usize, f64)>,
}

impl DkgSnapshot {
    pub fn from_engine(engine: &DkgEngine) -> Result<DkgSnapshot, KnowledgeError> {
        let raw_nodes = engine.nodes()?;
        let raw_edges = engine.edges()?;

        // Map node_id (i64) -> index in the nodes vec
        let id_to_idx: FxHashMap<i64, usize> = raw_nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (n.node_id, i))
            .collect();

        // Count inbound edges per node and track connectivity
        let mut inbound_count: FxHashMap<i64, usize> = FxHashMap::default();
        let mut has_outbound = vec![false; raw_nodes.len()];
        let mut has_inbound = vec![false; raw_nodes.len()];

        for (src_id, tgt_id, _) in &raw_edges {
            *inbound_count.entry(*tgt_id).or_insert(0) += 1;
            if let Some(&src_idx) = id_to_idx.get(src_id) {
                has_outbound[src_idx] = true;
            }
            if let Some(&tgt_idx) = id_to_idx.get(tgt_id) {
                has_inbound[tgt_idx] = true;
            }
        }

        // Build PageRankNodeInfo vec with derived ref_count
        let nodes: Vec<PageRankNodeInfo> = raw_nodes
            .iter()
            .map(|n| PageRankNodeInfo {
                node_id: n.node_id,
                symbol_name: n.symbol_name.clone(),
                file_path: n.file_path.clone(),
                signature: n.signature.clone(),
                kind: n.kind,
                ref_count: inbound_count.get(&n.node_id).copied().unwrap_or(0),
            })
            .collect();

        // Build edge vec: map i64 IDs to usize indices, weight=1.0 for all types
        let mut edges: Vec<(usize, usize, f64)> = Vec::with_capacity(raw_edges.len());
        for (src_id, tgt_id, _edge_type) in &raw_edges {
            if let (Some(&src_idx), Some(&tgt_idx)) =
                (id_to_idx.get(src_id), id_to_idx.get(tgt_id))
            {
                edges.push((src_idx, tgt_idx, 1.0));
            }
        }

        // Orphan nodes (zero inbound AND zero outbound) get a self-edge
        for i in 0..nodes.len() {
            if !has_inbound[i] && !has_outbound[i] {
                edges.push((i, i, 0.1));
            }
        }

        Ok(DkgSnapshot { nodes, edges })
    }
}

#[derive(Debug, Clone)]
pub struct PageRankConfig {
    pub damping_factor: f64,
    pub max_iterations: usize,
    pub tolerance: f64,
    pub density_threshold: f64,
    pub max_map_tokens: usize,
}

impl Default for PageRankConfig {
    fn default() -> Self {
        Self {
            damping_factor: 0.85,
            max_iterations: 100,
            tolerance: 1e-6,
            density_threshold: 0.3,
            max_map_tokens: 2048,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RankedSymbol {
    pub node_id: i64,
    pub symbol_name: String,
    pub file_path: PathBuf,
    pub rank: f64,
    pub signature: String,
}

#[derive(Debug, Clone)]
pub struct RepoMap {
    pub ranked_symbols: Vec<RankedSymbol>,
    pub token_count: usize,
    pub density: f64,
    pub used_pagerank: bool,
    pub fit_iterations: usize,
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

const MAX_SIGNATURE_CHARS: usize = 100;

fn truncate_signature(sig: &str, max_chars: usize) -> String {
    if sig.len() <= max_chars {
        sig.to_string()
    } else {
        // Slice on a UTF-8 char boundary: `max_chars` counts characters, and
        // the byte offset of the nth char (or the full length if the string is
        // shorter) is the safe cut point. A naive `sig[..max_chars]` byte slice
        // panics when `max_chars` lands inside a multibyte codepoint.
        let cut = sig
            .char_indices()
            .nth(max_chars)
            .map_or(sig.len(), |(i, _)| i);
        sig[..cut].to_string()
    }
}

const SPECIAL_FILES: &[&str] = &[
    "Cargo.toml",
    "Cargo.lock",
    "README.md",
    "README",
    "LICENSE",
    ".gitignore",
    "Dockerfile",
    "docker-compose.yml",
    "build.rs",
    "rust-toolchain.toml",
    "clippy.toml",
    "rustfmt.toml",
    ".cargo/config.toml",
];

fn is_special_file(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    SPECIAL_FILES.iter().any(|s| path_str.ends_with(s))
}

/// Sparse row-stochastic transition for the PageRank power iteration.
///
/// Replaces the former dense `Vec<Vec<f64>>` adjacency matrix, which cost
/// O(n²) memory (a 23 392-node ELAI self-index built a 4.38 GB dense matrix
/// even though the graph has only ~11 k edges). This representation is
/// O(n + edges): `out_edges[from]` holds the row-normalized `(to, weight)`
/// pairs of source `from`, exactly the non-zero entries the dense matrix held.
/// `dangling` is the set of sources whose row was all-zero (no out-edges),
/// identical to the former `build_dangling_vector(&matrix)`.
///
/// Numerics are byte-for-byte equivalent to the dense path: per (from,to) the
/// raw weight is `weight * sqrt(ref_count_of_to)` (×50 when either endpoint is
/// active), accumulated per cell, then each source row is normalized to sum 1.0.
struct SparseTransition {
    /// Per source node: its outgoing (target, normalized_weight) edges.
    out_edges: Vec<Vec<(usize, f64)>>,
    /// Source indices with no outgoing edges (former all-zero matrix rows).
    dangling: Vec<usize>,
}

fn build_sparse_transition(
    snapshot: &DkgSnapshot,
    active_files: &[PathBuf],
) -> SparseTransition {
    let n = snapshot.nodes.len();

    // Accumulate raw weights per source, merging duplicate (from,to) cells the
    // way the dense matrix did with `matrix[from][to] += w`.
    let mut out_edges: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];

    for &(from, to, weight) in &snapshot.edges {
        if from >= n || to >= n {
            continue;
        }
        let ref_count = snapshot.nodes[to].ref_count;
        let scale = if ref_count > 0 {
            (ref_count as f64).sqrt()
        } else {
            1.0
        };
        let mut w = weight * scale;

        let from_active = active_files.iter().any(|f| f == &snapshot.nodes[from].file_path);
        let to_active = active_files.iter().any(|f| f == &snapshot.nodes[to].file_path);
        if from_active || to_active {
            w *= 50.0;
        }

        // Merge into an existing (from,to) cell if present, else push a new one.
        let row = &mut out_edges[from];
        if let Some(cell) = row.iter_mut().find(|(t, _)| *t == to) {
            cell.1 += w;
        } else {
            row.push((to, w));
        }
    }

    // Row-normalize: each source's outgoing weights sum to 1.0 (stochastic).
    let mut dangling = Vec::new();
    for (from, row) in out_edges.iter_mut().enumerate() {
        let sum: f64 = row.iter().map(|(_, w)| *w).sum();
        if sum > 0.0 {
            for (_, w) in row.iter_mut() {
                *w /= sum;
            }
        } else {
            // All-zero row == no resolvable out-edges == dangling node.
            dangling.push(from);
        }
    }

    SparseTransition { out_edges, dangling }
}

fn build_personalization(node_count: usize, task_mentioned_indices: &[usize]) -> Vec<f64> {
    if node_count == 0 {
        return vec![];
    }

    let mut pers = vec![1.0_f64; node_count];
    for &idx in task_mentioned_indices {
        if idx < node_count {
            pers[idx] = 10.0;
        }
    }

    let sum: f64 = pers.iter().sum();
    if sum > 0.0 {
        for val in pers.iter_mut() {
            *val /= sum;
        }
    }

    pers
}

fn power_iteration(
    transition: &SparseTransition,
    personalization: &[f64],
    damping: f64,
    max_iter: usize,
    tolerance: f64,
) -> Vec<f64> {
    let n = transition.out_edges.len();
    if n == 0 {
        return vec![];
    }
    let dangling_nodes = &transition.dangling;

    let mut ranks = vec![1.0 / n as f64; n];

    for _ in 0..max_iter {
        let mut new_ranks = vec![0.0_f64; n];

        // Dangling node contribution: sum of ranks of dangling nodes
        let dangling_sum: f64 = dangling_nodes.iter().map(|&i| ranks[i]).sum();

        // Sparse matrix multiplication: new_rank[i] = sum_j(matrix[j][i] * ranks[j]).
        // matrix[j][i] is non-zero only for the row-normalized out-edges of j, so
        // scatter ranks[j]*w into new_rank[to] over j's out-edges. Identical sum
        // to the dense loop, O(edges) instead of O(n²).
        for (j, row) in transition.out_edges.iter().enumerate() {
            let rj = ranks[j];
            if rj == 0.0 {
                continue;
            }
            for &(to, w) in row {
                new_ranks[to] += w * rj;
            }
        }

        // Apply damping + personalization + dangling redistribution
        for (i, rank) in new_ranks.iter_mut().enumerate() {
            *rank = damping * (*rank + dangling_sum * personalization[i])
                + (1.0 - damping) * personalization[i];

            if !rank.is_finite() {
                *rank = 0.0;
            }
        }

        // Convergence check: L1 norm
        let diff: f64 = ranks
            .iter()
            .zip(new_ranks.iter())
            .map(|(a, b)| (a - b).abs())
            .sum();

        ranks = new_ranks;

        if diff < tolerance {
            break;
        }
    }

    ranks
}

fn binary_search_fit(
    ranked: &[RankedSymbol],
    max_tokens: usize,
    estimator: &dyn Fn(&str) -> usize,
) -> (Vec<RankedSymbol>, usize) {
    if ranked.is_empty() || max_tokens == 0 {
        return (vec![], 0);
    }

    let initial_guess = (max_tokens / 25).min(ranked.len());
    let mut lo = 0_usize;
    let mut hi = ranked.len();
    let mut best_count = initial_guess.max(1).min(ranked.len());
    let mut iterations = 0;

    // Start with the initial guess and refine
    let mut current = initial_guess.max(1).min(ranked.len());

    for _ in 0..10 {
        iterations += 1;

        let tokens: usize = ranked[..current]
            .iter()
            .map(|s| {
                let truncated = truncate_signature(&s.signature, MAX_SIGNATURE_CHARS);
                estimator(&truncated)
            })
            .sum();

        if tokens <= max_tokens {
            best_count = current;
            lo = current;
            if current == ranked.len() {
                break;
            }
            // Try more
            current = ((current + hi) / 2).min(ranked.len());
            if current <= lo {
                break;
            }
        } else {
            hi = current;
            current = (lo + current) / 2;
            if current == 0 {
                best_count = 0;
                break;
            }
            if current >= hi {
                break;
            }
        }
    }

    let fitted: Vec<RankedSymbol> = ranked[..best_count].to_vec();
    (fitted, iterations)
}

// ---------------------------------------------------------------------------
// Core function
// ---------------------------------------------------------------------------

pub fn compute_repo_map(
    snapshot: &DkgSnapshot,
    task: &str,
    active_files: &[PathBuf],
    config: &PageRankConfig,
    token_estimator: &dyn Fn(&str) -> usize,
) -> RepoMap {
    let n = snapshot.nodes.len();

    if n == 0 {
        return RepoMap {
            ranked_symbols: vec![],
            token_count: 0,
            density: 0.0,
            used_pagerank: false,
            fit_iterations: 0,
        };
    }

    let edge_count = snapshot.edges.len();
    let density = if n <= 1 {
        0.0
    } else {
        edge_count as f64 / (n * (n - 1)) as f64
    };

    // Density guard: if too dense, PageRank is uninformative
    if density > config.density_threshold {
        let ranked_symbols: Vec<RankedSymbol> = snapshot
            .nodes
            .iter()
            .map(|node| RankedSymbol {
                node_id: node.node_id,
                symbol_name: node.symbol_name.clone(),
                file_path: node.file_path.clone(),
                rank: 1.0 / n as f64,
                signature: truncate_signature(&node.signature, MAX_SIGNATURE_CHARS),
            })
            .collect();

        let (fitted, fit_iterations) =
            binary_search_fit(&ranked_symbols, config.max_map_tokens, token_estimator);
        let token_count: usize = fitted
            .iter()
            .map(|s| token_estimator(&s.signature))
            .sum();

        return RepoMap {
            ranked_symbols: fitted,
            token_count,
            density,
            used_pagerank: false,
            fit_iterations,
        };
    }

    // Build sparse row-stochastic transition and run PageRank. Sparse (O(n+edges))
    // replaces the former dense O(n²) adjacency matrix that OOM'd on large repos.
    let transition = build_sparse_transition(snapshot, active_files);

    // Identify task-mentioned nodes
    let task_lower = task.to_lowercase();
    let task_mentioned: Vec<usize> = snapshot
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| task_lower.contains(&node.symbol_name.to_lowercase()))
        .map(|(i, _)| i)
        .collect();

    let personalization = build_personalization(n, &task_mentioned);

    let ranks = power_iteration(
        &transition,
        &personalization,
        config.damping_factor,
        config.max_iterations,
        config.tolerance,
    );

    // Build ranked symbols sorted by rank descending
    let mut ranked_symbols: Vec<RankedSymbol> = snapshot
        .nodes
        .iter()
        .enumerate()
        .map(|(i, node)| RankedSymbol {
            node_id: node.node_id,
            symbol_name: node.symbol_name.clone(),
            file_path: node.file_path.clone(),
            rank: ranks[i],
            signature: truncate_signature(&node.signature, MAX_SIGNATURE_CHARS),
        })
        .collect();

    ranked_symbols.sort_by(|a, b| b.rank.partial_cmp(&a.rank).unwrap_or(std::cmp::Ordering::Equal));

    // Ensure special files are always included
    let special_indices: Vec<usize> = snapshot
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| is_special_file(&node.file_path))
        .map(|(i, _)| i)
        .collect();

    // Binary search fit
    let (mut fitted, fit_iterations) =
        binary_search_fit(&ranked_symbols, config.max_map_tokens, token_estimator);

    // Re-insert special file nodes if they were cut
    for &idx in &special_indices {
        let node = &snapshot.nodes[idx];
        if !fitted.iter().any(|s| s.node_id == node.node_id) {
            fitted.push(RankedSymbol {
                node_id: node.node_id,
                symbol_name: node.symbol_name.clone(),
                file_path: node.file_path.clone(),
                rank: ranks[idx],
                signature: truncate_signature(&node.signature, MAX_SIGNATURE_CHARS),
            });
        }
    }

    let token_count: usize = fitted
        .iter()
        .map(|s| token_estimator(&s.signature))
        .sum();

    RepoMap {
        ranked_symbols: fitted,
        token_count,
        density,
        used_pagerank: true,
        fit_iterations,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_snapshot(node_count: usize, edges: Vec<(usize, usize)>) -> DkgSnapshot {
        let nodes: Vec<PageRankNodeInfo> = (0..node_count)
            .map(|i| PageRankNodeInfo {
                node_id: i as i64,
                symbol_name: format!("sym_{i}"),
                file_path: PathBuf::from(format!("src/mod_{i}.rs")),
                signature: format!("fn sym_{i}()"),
                kind: SymbolKind::Function,
                ref_count: 0,
            })
            .collect();

        let weighted_edges: Vec<(usize, usize, f64)> =
            edges.into_iter().map(|(a, b)| (a, b, 1.0)).collect();

        DkgSnapshot {
            nodes,
            edges: weighted_edges,
        }
    }

    fn dummy_token_estimator(s: &str) -> usize {
        s.len() / 4 + 1
    }

    // T1: linear chain, task mentions sym_3 -> sym_3 ranks >= sym_0
    #[test]
    fn test_basic_ranking() {
        let snap = make_snapshot(5, vec![(0, 1), (1, 2), (2, 3), (3, 4)]);
        let config = PageRankConfig::default();
        let result = compute_repo_map(&snap, "sym_3", &[], &config, &dummy_token_estimator);
        let rank_3 = result.ranked_symbols.iter().find(|s| s.symbol_name == "sym_3").unwrap().rank;
        let rank_0 = result.ranked_symbols.iter().find(|s| s.symbol_name == "sym_0").unwrap().rank;
        assert!(rank_3 >= rank_0, "sym_3 (mentioned) should rank >= sym_0");
    }

    // T2: active file boost -> boosted node ranks higher
    #[test]
    fn test_edge_weight_boost() {
        let snap = make_snapshot(3, vec![(0, 1), (0, 2)]);
        let active = vec![PathBuf::from("src/mod_1.rs")];
        let config = PageRankConfig {
            density_threshold: 0.99,
            ..PageRankConfig::default()
        };
        let result = compute_repo_map(&snap, "task", &active, &config, &dummy_token_estimator);
        assert!(result.used_pagerank, "pagerank should run for this test");
        let rank_1 = result.ranked_symbols.iter().find(|s| s.symbol_name == "sym_1").unwrap().rank;
        let rank_2 = result.ranked_symbols.iter().find(|s| s.symbol_name == "sym_2").unwrap().rank;
        assert!(rank_1 > rank_2, "sym_1 (active file) should rank higher than sym_2");
    }

    // T3: node name in task string -> personalization 10x -> ranks higher
    #[test]
    fn test_personalization() {
        let snap = make_snapshot(4, vec![(0, 1), (1, 2), (2, 3)]);
        let config = PageRankConfig::default();
        let result = compute_repo_map(&snap, "sym_2", &[], &config, &dummy_token_estimator);
        let rank_2 = result.ranked_symbols.iter().find(|s| s.symbol_name == "sym_2").unwrap().rank;
        let rank_0 = result.ranked_symbols.iter().find(|s| s.symbol_name == "sym_0").unwrap().rank;
        assert!(rank_2 > rank_0, "sym_2 (mentioned in task) should rank higher than sym_0");
    }

    // T4: density > threshold -> used_pagerank == false, insertion order
    #[test]
    fn test_density_guard_active() {
        // 3 nodes, 2 edges -> density = 2/(3*2) = 0.333
        let snap = make_snapshot(3, vec![(0, 1), (1, 2)]);
        let config = PageRankConfig {
            density_threshold: 0.01,
            ..PageRankConfig::default()
        };
        let result = compute_repo_map(&snap, "task", &[], &config, &dummy_token_estimator);
        assert!(!result.used_pagerank, "density > threshold should skip pagerank");
    }

    // T5: density < threshold on sparse graph -> used_pagerank == true
    #[test]
    fn test_density_guard_inactive() {
        let snap = make_snapshot(3, vec![(0, 1), (1, 2)]);
        let config = PageRankConfig {
            density_threshold: 0.99,
            ..PageRankConfig::default()
        };
        let result = compute_repo_map(&snap, "task", &[], &config, &dummy_token_estimator);
        assert!(result.used_pagerank, "density < threshold should use pagerank");
    }

    // T6: token_count <= max_map_tokens * 115%
    #[test]
    fn test_token_budget_fitting() {
        let snap = make_snapshot(10, vec![(0, 1), (1, 2), (2, 3)]);
        let config = PageRankConfig {
            max_map_tokens: 50,
            ..PageRankConfig::default()
        };
        let result = compute_repo_map(&snap, "task", &[], &config, &dummy_token_estimator);
        assert!(
            result.token_count <= config.max_map_tokens * 115 / 100,
            "token_count {} should be <= {}",
            result.token_count,
            config.max_map_tokens * 115 / 100
        );
    }

    // T7: 200 nodes, budget=1000 -> result includes >= 85 nodes
    #[test]
    fn test_token_budget_undershoot() {
        let edges: Vec<(usize, usize)> = (0..199).map(|i| (i, i + 1)).collect();
        let snap = make_snapshot(200, edges);
        let config = PageRankConfig {
            max_map_tokens: 1000,
            ..PageRankConfig::default()
        };
        let result = compute_repo_map(&snap, "task", &[], &config, &dummy_token_estimator);
        assert!(
            result.ranked_symbols.len() >= 85,
            "should include >= 85 nodes, got {}",
            result.ranked_symbols.len()
        );
    }

    // T8: snapshot has Cargo.toml path -> always in output
    #[test]
    fn test_special_files_included() {
        let mut snap = make_snapshot(5, vec![(0, 1), (1, 2)]);
        snap.nodes[4].file_path = PathBuf::from("Cargo.toml");
        let config = PageRankConfig::default();
        let result = compute_repo_map(&snap, "task", &[], &config, &dummy_token_estimator);
        assert!(
            result.ranked_symbols.iter().any(|s| s.file_path == PathBuf::from("Cargo.toml")),
            "Cargo.toml node should always be in output"
        );
    }

    // T9: personalization vec sums to 1.0 (checked via rank behavior)
    #[test]
    fn test_personalization_normalization() {
        let snap = make_snapshot(4, vec![(0, 1), (1, 2), (2, 3)]);
        let config = PageRankConfig {
            max_map_tokens: 100_000,
            ..PageRankConfig::default()
        };
        let result = compute_repo_map(&snap, "sym_0 sym_1", &[], &config, &dummy_token_estimator);
        assert!(result.used_pagerank);
        let total_rank: f64 = result.ranked_symbols.iter().map(|s| s.rank).sum();
        assert!(
            (total_rank - 1.0).abs() < 1e-4,
            "ranks should sum to ~1.0, got {}",
            total_rank
        );
    }

    // T10: 1000-node graph -> fit_iterations <= 10
    #[test]
    fn test_binary_search_convergence() {
        let edges: Vec<(usize, usize)> = (0..999).map(|i| (i, i + 1)).collect();
        let snap = make_snapshot(1000, edges);
        let config = PageRankConfig {
            max_map_tokens: 500,
            ..PageRankConfig::default()
        };
        let result = compute_repo_map(&snap, "task", &[], &config, &dummy_token_estimator);
        assert!(
            result.fit_iterations <= 10,
            "binary search should converge in <= 10 iterations, got {}",
            result.fit_iterations
        );
    }

    // T11: empty graph
    #[test]
    fn test_empty_graph() {
        let snap = make_snapshot(0, vec![]);
        let config = PageRankConfig::default();
        let result = compute_repo_map(&snap, "task", &[], &config, &dummy_token_estimator);
        assert_eq!(result.token_count, 0);
        assert_eq!(result.fit_iterations, 0);
        assert!(result.ranked_symbols.is_empty());
    }

    // T12: single node
    #[test]
    fn test_single_node() {
        let snap = make_snapshot(1, vec![]);
        let config = PageRankConfig::default();
        let result = compute_repo_map(&snap, "task", &[], &config, &dummy_token_estimator);
        assert_eq!(result.ranked_symbols.len(), 1);
    }

    // T13: max_map_tokens=0 -> empty, no panic
    #[test]
    fn test_zero_budget() {
        let snap = make_snapshot(5, vec![(0, 1), (1, 2)]);
        let config = PageRankConfig {
            max_map_tokens: 0,
            ..PageRankConfig::default()
        };
        let result = compute_repo_map(&snap, "task", &[], &config, &dummy_token_estimator);
        assert!(result.ranked_symbols.is_empty(), "zero budget should produce empty result");
    }

    // T14: all nodes mentioned -> no NaN/Inf
    #[test]
    fn test_all_nodes_mentioned() {
        let snap = make_snapshot(3, vec![(0, 1), (1, 2)]);
        let config = PageRankConfig {
            max_map_tokens: 100_000,
            ..PageRankConfig::default()
        };
        let result = compute_repo_map(&snap, "sym_0 sym_1 sym_2", &[], &config, &dummy_token_estimator);
        for sym in &result.ranked_symbols {
            assert!(sym.rank.is_finite(), "rank should be finite, got {}", sym.rank);
            assert!(!sym.rank.is_nan(), "rank should not be NaN");
        }
    }

    // T15: A->B->C chain, C is leaf -> C.rank > 0.0 (dangling redistribution)
    #[test]
    fn test_dangling_node_redistribution() {
        let snap = make_snapshot(3, vec![(0, 1), (1, 2)]);
        let config = PageRankConfig {
            max_map_tokens: 100_000,
            ..PageRankConfig::default()
        };
        let result = compute_repo_map(&snap, "task", &[], &config, &dummy_token_estimator);
        let rank_2 = result.ranked_symbols.iter().find(|s| s.symbol_name == "sym_2").unwrap().rank;
        assert!(rank_2 > 0.0, "dangling node sym_2 should have rank > 0.0, got {}", rank_2);
    }

    // T16: orphan node with self-edge manually set -> rank > 0.0
    #[test]
    fn test_orphan_self_edges() {
        let mut snap = make_snapshot(3, vec![(0, 1)]);
        // Manually add self-edge for orphan node 2 (from_engine would do this)
        snap.edges.push((2, 2, 0.1));
        let config = PageRankConfig {
            max_map_tokens: 100_000,
            ..PageRankConfig::default()
        };
        let result = compute_repo_map(&snap, "task", &[], &config, &dummy_token_estimator);
        let rank_2 = result.ranked_symbols.iter().find(|s| s.symbol_name == "sym_2").unwrap().rank;
        assert!(rank_2 > 0.0, "orphan with self-edge should have rank > 0.0, got {}", rank_2);
    }

    // T17: sqrt scaling — node with ref_count=100 vs ref_count=4, ratio < 8x
    #[test]
    fn test_sqrt_reference_scaling() {
        let mut snap = make_snapshot(3, vec![(2, 0), (2, 1)]);
        snap.nodes[0].ref_count = 100;
        snap.nodes[1].ref_count = 4;
        let config = PageRankConfig {
            max_map_tokens: 100_000,
            ..PageRankConfig::default()
        };
        let result = compute_repo_map(&snap, "task", &[], &config, &dummy_token_estimator);
        let rank_a = result.ranked_symbols.iter().find(|s| s.symbol_name == "sym_0").unwrap().rank;
        let rank_b = result.ranked_symbols.iter().find(|s| s.symbol_name == "sym_1").unwrap().rank;
        if rank_b > 0.0 {
            assert!(
                rank_a / rank_b < 8.0,
                "sqrt scaling: ratio should be < 8.0, got {}",
                rank_a / rank_b
            );
        }
    }

    // T18: 500-char signature, token count respects truncation
    #[test]
    fn test_line_truncation() {
        let mut snap = make_snapshot(2, vec![(0, 1)]);
        snap.nodes[0].signature = "a".repeat(500);
        let config = PageRankConfig::default();
        let result = compute_repo_map(&snap, "task", &[], &config, &dummy_token_estimator);
        // If truncation to 100 chars works, token estimate should be < 500/4 = 125
        assert!(
            result.token_count < 500 / 4,
            "token_count {} should be < 125 (truncation should apply)",
            result.token_count
        );
    }

    // A1: fully-connected 10-node graph -> density ~1.0 -> used_pagerank == false
    #[test]
    fn test_adversarial_dense_graph() {
        let edges: Vec<(usize, usize)> = (0..10)
            .flat_map(|a| (0..10).filter(move |&b| b != a).map(move |b| (a, b)))
            .collect();
        let snap = make_snapshot(10, edges);
        let config = PageRankConfig::default();
        let result = compute_repo_map(&snap, "task", &[], &config, &dummy_token_estimator);
        assert!(!result.used_pagerank, "fully-connected graph should trigger density guard");
    }

    // A2: star graph (node 0 -> 19 others) -> spoke nodes rank higher than hub
    // In standard PageRank, authorities (nodes receiving links) outrank hubs (nodes sending links)
    #[test]
    fn test_adversarial_star_graph() {
        let edges: Vec<(usize, usize)> = (1..20).map(|i| (0, i)).collect();
        let snap = make_snapshot(20, edges);
        let config = PageRankConfig {
            max_map_tokens: 100_000,
            ..PageRankConfig::default()
        };
        let result = compute_repo_map(&snap, "task", &[], &config, &dummy_token_estimator);
        let hub_rank = result.ranked_symbols.iter().find(|s| s.symbol_name == "sym_0").unwrap().rank;
        let spoke_rank = result.ranked_symbols.iter().find(|s| s.symbol_name == "sym_1").unwrap().rank;
        assert!(
            spoke_rank >= hub_rank,
            "spoke nodes should rank >= hub in star graph: spoke={}, hub={}",
            spoke_rank, hub_rank
        );
    }

    // A3: disconnected {0,1} and {2,3}, active=[src/mod_0.rs] -> sym_0 or sym_1 > sym_2
    #[test]
    fn test_adversarial_disconnected() {
        let snap = make_snapshot(4, vec![(0, 1), (2, 3)]);
        let active = vec![PathBuf::from("src/mod_0.rs")];
        let config = PageRankConfig {
            max_map_tokens: 100_000,
            ..PageRankConfig::default()
        };
        let result = compute_repo_map(&snap, "task", &active, &config, &dummy_token_estimator);
        let rank_0 = result.ranked_symbols.iter().find(|s| s.symbol_name == "sym_0").unwrap().rank;
        let rank_1 = result.ranked_symbols.iter().find(|s| s.symbol_name == "sym_1").unwrap().rank;
        let rank_2 = result.ranked_symbols.iter().find(|s| s.symbol_name == "sym_2").unwrap().rank;
        let max_active = rank_0.max(rank_1);
        assert!(
            max_active > rank_2,
            "active component should rank higher: max({},{}) > {}",
            rank_0, rank_1, rank_2
        );
    }

    mod proptests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            // P1: random graph -> all ranks finite and >= 0.0
            #[test]
            fn ranks_are_finite_and_non_negative(node_count in 1_usize..=20) {
                let snap = make_snapshot(node_count, vec![]);
                let config = PageRankConfig::default();
                let result = compute_repo_map(
                    &snap,
                    "test",
                    &[],
                    &config,
                    &dummy_token_estimator,
                );
                for sym in &result.ranked_symbols {
                    prop_assert!(sym.rank.is_finite());
                    prop_assert!(sym.rank >= 0.0);
                }
            }

            // P2: random graph where used_pagerank==true -> ranks sum ~1.0
            #[test]
            fn ranks_sum_to_one(node_count in 2_usize..=50) {
                let edges: Vec<(usize, usize)> = (0..node_count.saturating_sub(1))
                    .map(|i| (i, i + 1))
                    .collect();
                let snap = make_snapshot(node_count, edges);
                let config = PageRankConfig {
                    max_map_tokens: 1_000_000,
                    ..PageRankConfig::default()
                };
                let result = compute_repo_map(
                    &snap,
                    "test",
                    &[],
                    &config,
                    &dummy_token_estimator,
                );
                if result.used_pagerank {
                    let total: f64 = result.ranked_symbols.iter().map(|s| s.rank).sum();
                    prop_assert!(
                        (total - 1.0).abs() < 1e-4,
                        "ranks should sum to ~1.0, got {}",
                        total
                    );
                }
            }

            // P3: random graph, random budget -> token_count <= budget * 115%
            #[test]
            fn token_count_within_budget(
                node_count in 1_usize..=100,
                max_tokens in 100_usize..=10000
            ) {
                let edges: Vec<(usize, usize)> = (0..node_count.saturating_sub(1))
                    .map(|i| (i, i + 1))
                    .collect();
                let snap = make_snapshot(node_count, edges);
                let config = PageRankConfig {
                    max_map_tokens: max_tokens,
                    ..PageRankConfig::default()
                };
                let result = compute_repo_map(
                    &snap,
                    "test",
                    &[],
                    &config,
                    &dummy_token_estimator,
                );
                prop_assert!(
                    result.token_count <= max_tokens * 115 / 100,
                    "token_count {} exceeds budget {} * 115%",
                    result.token_count,
                    max_tokens
                );
            }
        }
    }
}
