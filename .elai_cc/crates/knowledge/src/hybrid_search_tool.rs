use crate::degradation::SearchMode;
use crate::dkg::DkgEngine;
use crate::error::KnowledgeError;
use crate::pipeline::QueryPipeline;
use crate::QueryTier;
use crate::arag::RetrievalResult;
use std::collections::HashSet;

const MAX_MID_TASK_ROUNDS: usize = 3;

pub fn check_round_limit(current_round: usize) -> bool {
    current_round < MAX_MID_TASK_ROUNDS
}

pub fn handle_hybrid_search(
    pipeline: &QueryPipeline,
    query: String,
    _top_k: usize,
    _mode: SearchMode,
) -> Result<Vec<RetrievalResult>, KnowledgeError> {
    let stages = pipeline.query_with_tier(&query, QueryTier::Fast)?;

    let mut all_results = Vec::new();
    for stage in stages {
        all_results.extend(stage.results);
    }

    Ok(all_results)
}

pub fn expand_with_graph_neighbors(
    results: &[RetrievalResult],
    dkg: &DkgEngine,
    max_depth: usize,
) -> Result<Vec<RetrievalResult>, KnowledgeError> {
    let all_nodes = dkg.nodes()?;
    let mut matched_ids: Vec<i64> = Vec::new();
    for r in results {
        let needle = r.chunk_id.as_str();
        for n in &all_nodes {
            if n.symbol_name.contains(needle)
                || needle.contains(n.symbol_name.as_str())
                || n.file_path.to_string_lossy().contains(needle)
                || needle.contains(n.file_path.to_string_lossy().as_ref())
            {
                matched_ids.push(n.node_id);
            }
        }
    }

    let mut seen: HashSet<i64> = matched_ids.iter().copied().collect();
    let mut frontier: Vec<i64> = matched_ids.clone();
    let mut neighbor_ids: Vec<i64> = Vec::new();
    for _ in 0..max_depth {
        let mut next_frontier: Vec<i64> = Vec::new();
        for node_id in &frontier {
            for (target_id, _) in dkg.outbound_edges(*node_id)? {
                if seen.insert(target_id) {
                    neighbor_ids.push(target_id);
                    next_frontier.push(target_id);
                }
            }
            for (source_id, _) in dkg.inbound_edges(*node_id)? {
                if seen.insert(source_id) {
                    neighbor_ids.push(source_id);
                    next_frontier.push(source_id);
                }
            }
        }
        frontier = next_frontier;
    }

    let mut merged: Vec<RetrievalResult> = results.to_vec();
    let mut seen_chunk: HashSet<String> = results.iter().map(|r| r.chunk_id.clone()).collect();
    for nid in neighbor_ids {
        if let Some(node) = dkg.node_by_id(nid)? {
            let chunk_id = node.symbol_name.clone();
            if seen_chunk.insert(chunk_id.clone()) {
                merged.push(RetrievalResult {
                    chunk_id,
                    snippet: node.signature.clone(),
                    score: 0.5,
                    tool_used: "graph_neighbor".to_string(),
                });
            }
        }
    }
    Ok(merged)
}
