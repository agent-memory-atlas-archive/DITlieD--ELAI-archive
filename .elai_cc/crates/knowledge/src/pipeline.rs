use std::collections::HashMap;
use std::error::Error;

use crate::arag::{AragOrchestrator, RetrievalResult};
use crate::bm25::Bm25Index;
use crate::catrag::CatRag;
use crate::pagerank::DkgSnapshot;
use crate::reranker::CrossEncoderReranker;

// ---------------------------------------------------------------------------
// PipelineStage
// ---------------------------------------------------------------------------

/// Result of a single stage in the retrieval pipeline.
///
/// `latency_ms` is always 0 here; callers measure wall time externally
/// (entropy quarantine: raw Instant confined to core/src/clock.rs).
#[derive(Debug, Clone)]
pub struct PipelineStage {
    pub stage: String,
    pub results: Vec<RetrievalResult>,
    pub latency_ms: u64,
}

// ---------------------------------------------------------------------------
// QueryPipeline
// ---------------------------------------------------------------------------

/// End-to-end retrieval pipeline: bm25 -> vector -> pagerank -> catrag -> arag -> rerank -> cross_encoder -> token_fit.
pub struct QueryPipeline {
    pub bm25: Bm25Index,
    pub catrag: CatRag,
    pub arag: AragOrchestrator,
    pub cross_encoder: Option<CrossEncoderReranker>,
}

impl QueryPipeline {
    pub fn new(bm25: Bm25Index, catrag: CatRag, arag: AragOrchestrator) -> Self {
        Self { bm25, catrag, arag, cross_encoder: None }
    }

    pub fn with_cross_encoder(mut self, reranker: CrossEncoderReranker) -> Self {
        self.cross_encoder = Some(reranker);
        self
    }

    /// Execute the full retrieval pipeline for a query.
    ///
    /// Returns 8 stages in order. `latency_ms` is 0 for all stages;
    /// callers should measure overall pipeline time externally.
    pub fn query(
        &self,
        query: &str,
    ) -> Result<Vec<PipelineStage>, Box<dyn Error>> {
        let mut stages = Vec::with_capacity(7);

        // Stage 0: bm25
        let bm25_hits = self.bm25.query(query, 10).unwrap_or_default();
        let bm25_results: Vec<RetrievalResult> = bm25_hits
            .into_iter()
            .map(|h| RetrievalResult {
                chunk_id: format!("{}:{}-{}", h.path, h.line_start, h.line_end),
                snippet: h.snippet,
                score: h.score as f64,
                tool_used: "bm25".to_string(),
            })
            .collect();
        stages.push(PipelineStage {
            stage: "bm25".to_string(),
            results: bm25_results,
            latency_ms: 0,
        });

        // Stage 1: vector (stub — no vector index, graceful degrade)
        stages.push(PipelineStage {
            stage: "vector".to_string(),
            results: Vec::new(),
            latency_ms: 0,
        });

        // Stage 2: pagerank (empty DkgSnapshot — no DkgEngine field)
        let empty_snapshot = DkgSnapshot {
            nodes: Vec::new(),
            edges: Vec::new(),
        };
        stages.push(PipelineStage {
            stage: "pagerank".to_string(),
            results: Vec::new(),
            latency_ms: 0,
        });

        // Stage 3: catrag (empty snapshot triggers density guard -> empty overlay)
        let _overlay = self.catrag.compute_overlay(query, &empty_snapshot, &[])?;
        stages.push(PipelineStage {
            stage: "catrag".to_string(),
            results: Vec::new(),
            latency_ms: 0,
        });

        // Stage 4: arag
        let arag_results = self.arag.query_pipeline(query, &empty_snapshot)?;
        stages.push(PipelineStage {
            stage: "arag".to_string(),
            results: arag_results,
            latency_ms: 0,
        });

        // Stage 5: rerank (RRF fusion across all stages that produced results)
        let reranked = self.rrf_rerank(&stages);
        stages.push(PipelineStage {
            stage: "rerank".to_string(),
            results: reranked,
            latency_ms: 0,
        });

        // Stage 6: cross_encoder (cross-encoder reranking after RRF fusion)
        // In SPARSE_ONLY / BM25_ONLY modes, caller should not attach a cross_encoder.
        // When cross_encoder is None, pass through RRF results unchanged.
        let ce_input = stages.last().map(|s| s.results.clone()).unwrap_or_default();
        let ce_results = if let Some(ref ce) = self.cross_encoder {
            ce.rerank(query, ce_input).unwrap_or_else(|_| {
                // Graceful degradation: on reranker error, use RRF order
                stages.last().map(|s| s.results.clone()).unwrap_or_default()
            })
        } else {
            ce_input
        };
        stages.push(PipelineStage {
            stage: "cross_encoder".to_string(),
            results: ce_results,
            latency_ms: 0,
        });

        // Stage 7: token_fit (passthrough — budget enforcement is A-RAG internal)
        let token_fit_results = if let Some(ce_stage) = stages.last() {
            ce_stage.results.clone()
        } else {
            Vec::new()
        };
        stages.push(PipelineStage {
            stage: "token_fit".to_string(),
            results: token_fit_results,
            latency_ms: 0,
        });

        Ok(stages)
    }

    /// Reciprocal Rank Fusion across all stages that produced results.
    /// RRF score = sum(1 / (k + rank)) where k=60 and rank starts at 1.
    fn rrf_rerank(&self, stages: &[PipelineStage]) -> Vec<RetrievalResult> {
        let mut scores: HashMap<String, (f64, RetrievalResult)> = HashMap::new();
        for stage in stages {
            for (rank_idx, result) in stage.results.iter().enumerate() {
                let rrf_score = 1.0 / (60.0 + (rank_idx + 1) as f64);
                let entry = scores
                    .entry(result.chunk_id.clone())
                    .or_insert_with(|| (0.0, result.clone()));
                entry.0 += rrf_score;
            }
        }
        let mut ranked: Vec<(f64, RetrievalResult)> = scores.into_values().collect();
        ranked.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        ranked
            .into_iter()
            .map(|(score, mut r)| {
                r.score = score;
                r.tool_used = "rerank".to_string();
                r
            })
            .collect()
    }
}
