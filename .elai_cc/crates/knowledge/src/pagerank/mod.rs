use std::path::PathBuf;

use crate::dkg::DkgEngine;
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
    pub fn from_engine(_engine: &DkgEngine) -> Result<DkgSnapshot, Box<dyn std::error::Error>> {
        todo!()
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
// Core function
// ---------------------------------------------------------------------------

pub fn compute_repo_map(
    _snapshot: &DkgSnapshot,
    _task: &str,
    _active_files: &[PathBuf],
    _config: &PageRankConfig,
    _token_estimator: &dyn Fn(&str) -> usize,
) -> RepoMap {
    todo!()
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

    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_basic_ranking() {
        let snap = make_snapshot(5, vec![(0, 1), (1, 2), (2, 3), (3, 4)]);
        let config = PageRankConfig::default();
        compute_repo_map(&snap, "find bugs", &[], &config, &dummy_token_estimator);
    }

    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_edge_weight_boost() {
        let snap = make_snapshot(3, vec![(0, 1), (0, 1), (1, 2)]);
        let config = PageRankConfig::default();
        compute_repo_map(&snap, "optimize", &[], &config, &dummy_token_estimator);
    }

    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_personalization() {
        let snap = make_snapshot(4, vec![(0, 1), (1, 2), (2, 3)]);
        let active = vec![PathBuf::from("src/mod_0.rs")];
        let config = PageRankConfig::default();
        compute_repo_map(&snap, "refactor", &active, &config, &dummy_token_estimator);
    }

    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_density_guard_active() {
        let snap = make_snapshot(3, vec![(0, 1), (1, 2)]);
        let config = PageRankConfig {
            density_threshold: 0.01,
            ..PageRankConfig::default()
        };
        compute_repo_map(&snap, "task", &[], &config, &dummy_token_estimator);
    }

    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_density_guard_inactive() {
        let snap = make_snapshot(3, vec![(0, 1), (1, 2)]);
        let config = PageRankConfig {
            density_threshold: 0.99,
            ..PageRankConfig::default()
        };
        compute_repo_map(&snap, "task", &[], &config, &dummy_token_estimator);
    }

    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_token_budget_fitting() {
        let snap = make_snapshot(10, vec![(0, 1), (1, 2), (2, 3)]);
        let config = PageRankConfig {
            max_map_tokens: 50,
            ..PageRankConfig::default()
        };
        compute_repo_map(&snap, "task", &[], &config, &dummy_token_estimator);
    }

    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_token_budget_undershoot() {
        let snap = make_snapshot(3, vec![(0, 1), (1, 2)]);
        let config = PageRankConfig {
            max_map_tokens: 100_000,
            ..PageRankConfig::default()
        };
        compute_repo_map(&snap, "task", &[], &config, &dummy_token_estimator);
    }

    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_special_files_included() {
        let snap = make_snapshot(5, vec![(0, 1), (1, 2)]);
        let active = vec![PathBuf::from("src/mod_3.rs"), PathBuf::from("src/mod_4.rs")];
        let config = PageRankConfig::default();
        compute_repo_map(&snap, "fix issue", &active, &config, &dummy_token_estimator);
    }

    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_personalization_normalization() {
        let snap = make_snapshot(4, vec![(0, 1), (1, 2), (2, 3)]);
        let active = vec![PathBuf::from("src/mod_0.rs"), PathBuf::from("src/mod_1.rs")];
        let config = PageRankConfig::default();
        compute_repo_map(&snap, "review", &active, &config, &dummy_token_estimator);
    }

    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_binary_search_convergence() {
        let snap = make_snapshot(20, vec![(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)]);
        let config = PageRankConfig {
            max_map_tokens: 100,
            ..PageRankConfig::default()
        };
        compute_repo_map(&snap, "task", &[], &config, &dummy_token_estimator);
    }

    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_empty_graph() {
        let snap = make_snapshot(0, vec![]);
        let config = PageRankConfig::default();
        compute_repo_map(&snap, "task", &[], &config, &dummy_token_estimator);
    }

    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_single_node() {
        let snap = make_snapshot(1, vec![]);
        let config = PageRankConfig::default();
        compute_repo_map(&snap, "task", &[], &config, &dummy_token_estimator);
    }

    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_zero_budget() {
        let snap = make_snapshot(5, vec![(0, 1), (1, 2)]);
        let config = PageRankConfig {
            max_map_tokens: 0,
            ..PageRankConfig::default()
        };
        compute_repo_map(&snap, "task", &[], &config, &dummy_token_estimator);
    }

    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_all_nodes_mentioned() {
        let snap = make_snapshot(3, vec![(0, 1), (1, 2)]);
        let config = PageRankConfig {
            max_map_tokens: 100_000,
            ..PageRankConfig::default()
        };
        compute_repo_map(&snap, "sym_0 sym_1 sym_2", &[], &config, &dummy_token_estimator);
    }

    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_dangling_node_redistribution() {
        // Node 2 is a leaf — receives edges but has no outbound.
        let snap = make_snapshot(3, vec![(0, 1), (1, 2)]);
        let config = PageRankConfig::default();
        compute_repo_map(&snap, "task", &[], &config, &dummy_token_estimator);
    }

    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_orphan_self_edges() {
        // Node 2 has no edges at all — should get self-edge.
        let snap = make_snapshot(3, vec![(0, 1)]);
        let config = PageRankConfig::default();
        compute_repo_map(&snap, "task", &[], &config, &dummy_token_estimator);
    }

    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_sqrt_reference_scaling() {
        let mut snap = make_snapshot(3, vec![(0, 1), (0, 1), (0, 1), (2, 1)]);
        snap.nodes[1].ref_count = 4;
        let config = PageRankConfig::default();
        compute_repo_map(&snap, "task", &[], &config, &dummy_token_estimator);
    }

    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_line_truncation() {
        let mut snap = make_snapshot(2, vec![(0, 1)]);
        snap.nodes[0].signature = "a".repeat(200);
        let config = PageRankConfig::default();
        compute_repo_map(&snap, "task", &[], &config, &dummy_token_estimator);
    }

    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_adversarial_dense_graph() {
        // Fully connected 10-node graph.
        let edges: Vec<(usize, usize)> = (0..10)
            .flat_map(|a| (0..10).filter(move |&b| b != a).map(move |b| (a, b)))
            .collect();
        let snap = make_snapshot(10, edges);
        let config = PageRankConfig::default();
        compute_repo_map(&snap, "task", &[], &config, &dummy_token_estimator);
    }

    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_adversarial_star_graph() {
        // Node 0 connects to all others (hub-and-spoke).
        let edges: Vec<(usize, usize)> = (1..20).map(|i| (0, i)).collect();
        let snap = make_snapshot(20, edges);
        let config = PageRankConfig::default();
        compute_repo_map(&snap, "task", &[], &config, &dummy_token_estimator);
    }

    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_adversarial_disconnected() {
        // Two disconnected components: {0,1} and {2,3}.
        let snap = make_snapshot(4, vec![(0, 1), (2, 3)]);
        let config = PageRankConfig::default();
        compute_repo_map(&snap, "task", &[], &config, &dummy_token_estimator);
    }

    mod proptests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            #[should_panic(expected = "not yet implemented")]
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
        }
    }
}
