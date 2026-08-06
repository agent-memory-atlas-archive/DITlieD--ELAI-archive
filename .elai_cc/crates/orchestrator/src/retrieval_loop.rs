// J-38 Unit 3: agentic retrieval loop wiring. Calls compute_context_sufficiency
// after context budget assembly; if insufficient and rounds remain, calls
// hybrid_search + expand_with_graph_neighbors to recover missing context.

use crate::context_sufficiency::{compute_context_sufficiency, SufficiencyReport};
use elai_knowledge::hybrid_search_tool::{check_round_limit, expand_with_graph_neighbors};
use elai_knowledge::arag::RetrievalResult;
use elai_knowledge::degradation::SearchMode;
use elai_knowledge::pipeline::QueryPipeline;
use elai_knowledge::dkg::DkgEngine;

pub struct RetrievalLoopInput<'a> {
    pub intent_symbols: Vec<String>,
    pub context_symbols: Vec<String>,
    pub pipeline: Option<&'a QueryPipeline>,
    pub dkg: Option<&'a DkgEngine>,
    pub current_round: usize,
}

pub struct RetrievalLoopOutput {
    pub sufficiency: SufficiencyReport,
    pub search_results: Vec<RetrievalResult>,
}

pub fn run_retrieval_loop(input: RetrievalLoopInput) -> RetrievalLoopOutput {
    let sufficiency = compute_context_sufficiency(&input.intent_symbols, &input.context_symbols);

    let mut search_results = Vec::new();
    if !sufficiency.is_sufficient && check_round_limit(input.current_round) {
        if let Some(dkg) = input.dkg {
            if let Some(pipeline) = input.pipeline {
                if let Ok(results) = pipeline.hybrid_search(
                    sufficiency.missing.join(" "),
                    10,
                    SearchMode::Hybrid,
                ) {
                    if let Ok(expanded) = expand_with_graph_neighbors(&results, dkg, 2) {
                        search_results = expanded;
                    } else {
                        search_results = results;
                    }
                }
            } else {
                if let Ok(all_nodes) = dkg.nodes() {
                    let dkg_hits: Vec<RetrievalResult> = all_nodes
                        .iter()
                        .filter(|n| {
                            sufficiency.missing.iter().any(|m| {
                                n.symbol_name.contains(m.as_str())
                                    || m.contains(n.symbol_name.as_str())
                            })
                        })
                        .take(10)
                        .map(|n| RetrievalResult {
                            chunk_id: n.symbol_name.clone(),
                            snippet: n.signature.clone(),
                            score: 0.6,
                            tool_used: "dkg_symbol_lookup".to_string(),
                        })
                        .collect();
                    if !dkg_hits.is_empty() {
                        if let Ok(expanded) = expand_with_graph_neighbors(&dkg_hits, dkg, 2) {
                            search_results = expanded;
                        } else {
                            search_results = dkg_hits;
                        }
                    }
                }
            }
        }
    }

    // J-76 C6: cap + head/tail place (lost-in-the-middle). No curve at this
    // seam yet => conservative DEFAULT cap. When a profile knee is later
    // threaded, swap in retrieval_chunk_cap(Some(knee)).
    let cap = crate::retrieval_chunk_placement::retrieval_chunk_cap(None);
    search_results = crate::retrieval_chunk_placement::cap_and_place_retrieval_chunks(
        search_results,
        cap,
        |r| r.score,
        |r| r.chunk_id.as_str(),
    );

    RetrievalLoopOutput { sufficiency, search_results }
}
