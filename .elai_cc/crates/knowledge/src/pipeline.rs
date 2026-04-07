use std::error::Error;

use crate::arag::{AragOrchestrator, RetrievalResult};
use crate::bm25::Bm25Index;
use crate::catrag::CatRag;

// ---------------------------------------------------------------------------
// PipelineStage
// ---------------------------------------------------------------------------

/// Result of a single stage in the retrieval pipeline.
#[derive(Debug, Clone)]
pub struct PipelineStage {
    pub stage: String,
    pub results: Vec<RetrievalResult>,
    pub latency_ms: u64,
}

// ---------------------------------------------------------------------------
// QueryPipeline
// ---------------------------------------------------------------------------

/// End-to-end retrieval pipeline: bm25 -> catrag -> arag -> rerank -> token_fit.
pub struct QueryPipeline {
    pub bm25: Bm25Index,
    pub catrag: CatRag,
    pub arag: AragOrchestrator,
}

impl QueryPipeline {
    pub fn new(bm25: Bm25Index, catrag: CatRag, arag: AragOrchestrator) -> Self {
        Self { bm25, catrag, arag }
    }

    /// Execute the full retrieval pipeline for a query.
    pub fn query(
        &self,
        _query: &str,
    ) -> Result<Vec<PipelineStage>, Box<dyn Error>> {
        todo!()
    }
}
