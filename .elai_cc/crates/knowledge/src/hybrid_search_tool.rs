use crate::degradation::SearchMode;
use crate::error::KnowledgeError;
use crate::pipeline::QueryPipeline;
use crate::QueryTier;
use crate::arag::RetrievalResult;

const MAX_MID_TASK_ROUNDS: usize = 3;

// WIRE-DARK[J-38]: check_round_limit has no production caller; round-limit guard is exercised by handle_hybrid_search caller wiring in a later unit.
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
