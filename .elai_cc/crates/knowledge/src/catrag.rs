use std::error::Error;

use crate::pagerank::DkgSnapshot;

// ---------------------------------------------------------------------------
// phi() — paper Table 7 relevance scoring
// ---------------------------------------------------------------------------

/// Map a 0-10 integer relevance score to a PPR reset weight.
///
/// Source: CatRAG paper, Table 7.
/// Scores 0-3 map to 0.0 (irrelevant), 4-6 are low weights,
/// 7+ are high weights indicating strong contextual relevance.
pub fn phi(score: u8) -> f64 {
    const TABLE: [f64; 11] = [
        0.0, // 0
        0.0, // 1
        0.0, // 2
        0.0, // 3
        0.2, // 4
        0.25, // 5
        0.3, // 6
        2.0, // 7
        2.5, // 8
        3.0, // 9
        5.0, // 10
    ];
    debug_assert!(score <= 10, "phi() score must be 0..=10, got {score}");
    if (score as usize) < TABLE.len() {
        TABLE[score as usize]
    } else {
        *TABLE.last().unwrap()
    }
}

// ---------------------------------------------------------------------------
// CatRagConfig
// ---------------------------------------------------------------------------

/// Configuration for CatRAG personalized PageRank overlay.
///
/// Defaults are paper-verified (Table 7 + Section 4.2).
#[derive(Debug, Clone)]
pub struct CatRagConfig {
    pub damping: f64,
    pub epsilon: f64,
    pub beta: f64,
    pub n_seed: usize,
    pub k_edge: usize,
    pub min_graph_density: f64,
}

impl Default for CatRagConfig {
    fn default() -> Self {
        Self {
            damping: 0.5,
            epsilon: 0.2,
            beta: 2.5,
            n_seed: 5,
            k_edge: 15,
            min_graph_density: 0.01,
        }
    }
}

// ---------------------------------------------------------------------------
// CatRagOverlay
// ---------------------------------------------------------------------------

/// PPR overlay produced by CatRAG: seed reset distribution + edge weight deltas.
///
/// Indices refer to positions in DkgSnapshot.nodes, not database IDs.
#[derive(Debug, Clone)]
pub struct CatRagOverlay {
    pub reset_distribution: Vec<(usize, f64)>,
    pub edge_weight_deltas: Vec<(usize, usize, f64)>,
}

// ---------------------------------------------------------------------------
// CatRag
// ---------------------------------------------------------------------------

/// CatRAG engine: computes a query-aware PPR overlay on top of the DKG graph.
#[derive(Debug, Clone)]
pub struct CatRag {
    pub config: CatRagConfig,
}

impl CatRag {
    pub fn new(config: CatRagConfig) -> Self {
        Self { config }
    }

    /// Compute a PPR overlay for the given query against the DKG snapshot.
    ///
    /// Returns an overlay with seed reset distribution and edge weight deltas.
    /// Skips computation if graph density < config.min_graph_density.
    pub fn compute_overlay(
        &self,
        _query: &str,
        snapshot: &DkgSnapshot,
        _pagerank_scores: &[(usize, f64)],
    ) -> Result<CatRagOverlay, Box<dyn Error>> {
        let n = snapshot.nodes.len();
        if n < 2 {
            return Ok(CatRagOverlay {
                reset_distribution: Vec::new(),
                edge_weight_deltas: Vec::new(),
            });
        }
        let max_edges = n * (n - 1);
        let density = snapshot.edges.len() as f64 / max_edges as f64;
        if density < self.config.min_graph_density {
            return Ok(CatRagOverlay {
                reset_distribution: Vec::new(),
                edge_weight_deltas: Vec::new(),
            });
        }
        todo!()
    }
}
