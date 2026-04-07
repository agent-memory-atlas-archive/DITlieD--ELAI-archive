use std::collections::HashSet;
use std::error::Error;

use crate::pagerank::DkgSnapshot;

// ---------------------------------------------------------------------------
// Query tokenization helpers
// ---------------------------------------------------------------------------

/// Split query into lowercase tokens: whitespace, camelCase, and snake_case splitting.
fn tokenize_query(query: &str) -> Vec<String> {
    let mut raw_tokens: Vec<String> = Vec::new();
    for word in query.split_whitespace() {
        // snake_case split
        for part in word.split('_') {
            if part.is_empty() {
                continue;
            }
            // camelCase split: insert break before uppercase following lowercase
            let mut current = String::new();
            for c in part.chars() {
                if c.is_uppercase() && !current.is_empty() {
                    raw_tokens.push(current.to_lowercase());
                    current = String::new();
                }
                current.push(c);
            }
            if !current.is_empty() {
                raw_tokens.push(current.to_lowercase());
            }
        }
    }
    // Deduplicate while preserving order
    let mut seen = HashSet::new();
    raw_tokens
        .into_iter()
        .filter(|t| !t.is_empty() && seen.insert(t.clone()))
        .collect()
}

/// Count shared characters between a symbol name and query tokens (for pruning).
fn name_token_similarity(name: &str, tokens: &[String]) -> f64 {
    let name_lower = name.to_lowercase();
    let mut score = 0.0_f64;
    for tok in tokens {
        if name_lower.contains(tok.as_str()) {
            score += tok.len() as f64;
        }
    }
    score
}

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
    TABLE.get(score as usize).copied().unwrap_or(TABLE[TABLE.len() - 1])
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
        query: &str,
        snapshot: &DkgSnapshot,
        pagerank_scores: &[(usize, f64)],
    ) -> Result<CatRagOverlay, Box<dyn Error>> {
        let n = snapshot.nodes.len();
        if n < 2 {
            return Ok(CatRagOverlay {
                reset_distribution: Vec::new(),
                edge_weight_deltas: Vec::new(),
            });
        }
        let max_edges = (n as u128) * (n as u128 - 1);
        let density = snapshot.edges.len() as f64 / max_edges as f64;
        if density < self.config.min_graph_density {
            return Ok(CatRagOverlay {
                reset_distribution: Vec::new(),
                edge_weight_deltas: Vec::new(),
            });
        }

        // Validate edge indices
        if snapshot.edges.iter().any(|&(s, t, _)| s >= n || t >= n) {
            return Err("DkgSnapshot contains out-of-bounds edge indices".into());
        }

        // --- Symbolic anchoring ---
        let tokens = tokenize_query(query);
        let mut combined: Vec<f64> = vec![0.0; n];

        // Anchored nodes: query token is substring of symbol_name or vice versa
        let mut anchored: Vec<bool> = vec![false; n];
        for (i, node) in snapshot.nodes.iter().enumerate() {
            let name_lower = node.symbol_name.to_lowercase();
            for tok in &tokens {
                let reverse_match = name_lower.len() >= 3
                    && tok.contains(name_lower.as_str());
                if name_lower.contains(tok.as_str()) || reverse_match {
                    anchored[i] = true;
                    let weight = self.config.epsilon
                        * (node.ref_count.max(1) as f64).recip();
                    combined[i] += weight;
                    break;
                }
            }
        }

        // Merge with pagerank scores
        for &(idx, pr_score) in pagerank_scores {
            if idx < n {
                combined[idx] += pr_score * self.config.epsilon;
            }
        }

        // Normalize to sum=1.0
        let sum: f64 = combined.iter().sum();
        if sum > 0.0 {
            for w in combined.iter_mut() {
                *w /= sum;
            }
        }

        let reset_distribution: Vec<(usize, f64)> = combined
            .iter()
            .enumerate()
            .filter(|(_, &w)| w > 0.0)
            .map(|(i, &w)| (i, w))
            .collect();

        // --- Edge weighting ---
        // Select top-N_seed nodes by combined weight (descending)
        let mut seed_indices: Vec<usize> = (0..n).filter(|&i| combined[i] > 0.0).collect();
        seed_indices.sort_by(|&a, &b| {
            combined[b]
                .partial_cmp(&combined[a])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        seed_indices.truncate(self.config.n_seed);

        let mut edge_weight_deltas: Vec<(usize, usize, f64)> = Vec::new();

        for &u in &seed_indices {
            // Collect outgoing edges from this seed
            let mut outgoing: Vec<(usize, usize, f64)> = snapshot
                .edges
                .iter()
                .filter(|&&(src, _, _)| src == u)
                .copied()
                .collect();

            // Prune to k_edge by target name similarity to query tokens
            if outgoing.len() > self.config.k_edge {
                outgoing.sort_by(|a, b| {
                    let sim_b = name_token_similarity(
                        &snapshot.nodes[b.1].symbol_name,
                        &tokens,
                    );
                    let sim_a = name_token_similarity(
                        &snapshot.nodes[a.1].symbol_name,
                        &tokens,
                    );
                    sim_b
                        .partial_cmp(&sim_a)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                outgoing.truncate(self.config.k_edge);
            }

            for (_, v, _) in outgoing {
                let mut score: u8 = 5; // base "references" tier
                let target_lower = snapshot.nodes[v].symbol_name.to_lowercase();
                for tok in &tokens {
                    if target_lower.contains(tok.as_str()) {
                        score = score.saturating_add(3);
                        break;
                    }
                }
                score = score.min(10);
                let weight = phi(score);
                if weight > 0.0 {
                    edge_weight_deltas.push((u, v, weight));
                }
            }
        }

        // --- Key-fact enhancement ---
        // T_seed = anchored nodes UNION top-N_seed
        let mut t_seed: Vec<bool> = anchored.clone();
        for &idx in &seed_indices {
            t_seed[idx] = true;
        }

        for &(src, tgt, base_weight) in &snapshot.edges {
            if !t_seed[src] {
                continue;
            }
            let src_lower = snapshot.nodes[src].symbol_name.to_lowercase();
            let has_match = tokens.iter().any(|tok| src_lower.contains(tok.as_str()));
            if !has_match {
                continue;
            }
            let boost = 1.0 + self.config.beta;
            if let Some(entry) = edge_weight_deltas
                .iter_mut()
                .find(|e| e.0 == src && e.1 == tgt)
            {
                entry.2 *= boost;
            } else if base_weight > 0.0 {
                edge_weight_deltas.push((src, tgt, base_weight * boost));
            }
        }

        Ok(CatRagOverlay {
            reset_distribution,
            edge_weight_deltas,
        })
    }
}
