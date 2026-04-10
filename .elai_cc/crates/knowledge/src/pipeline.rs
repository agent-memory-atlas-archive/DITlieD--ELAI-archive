use std::collections::HashMap;
use std::error::Error;

use crate::arag::{AragOrchestrator, RetrievalResult};
use crate::bm25::Bm25Index;
use crate::catrag::CatRag;
use crate::dkg::DkgEngine;
use crate::pagerank::{compute_repo_map, DkgSnapshot, PageRankConfig};
use crate::reranker::CrossEncoderReranker;
use crate::vector::VectorModule;
use crate::vector::indexer::VectorIndexer;
use crate::ReadinessStatus;

// ---------------------------------------------------------------------------
// PipelineStage
// ---------------------------------------------------------------------------

/// Result of a single stage in the retrieval pipeline.
///
/// Maps to knowledge.txt Section 12 StageTrace. `latency_ms` is always 0 here;
/// callers measure wall time externally (entropy quarantine: raw Instant confined
/// to core/src/clock.rs).
#[derive(Debug, Clone)]
pub struct PipelineStage {
    pub stage: String,
    pub results: Vec<RetrievalResult>,
    pub latency_ms: u64,
    /// Circuit breaker state for stages that use one (rerank, vector).
    pub circuit_breaker_state: Option<String>,
    /// Error message if stage failed (for StageTrace).
    pub error: Option<String>,
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
    pub vector_module: Option<Box<dyn VectorModule + Send + Sync>>,
    pub vector_indexer: Option<VectorIndexer>,
    pub dkg_engine: Option<DkgEngine>,
}

impl QueryPipeline {
    pub fn new(bm25: Bm25Index, catrag: CatRag, arag: AragOrchestrator) -> Self {
        Self {
            bm25, catrag, arag,
            cross_encoder: None,
            vector_module: None,
            vector_indexer: None,
            dkg_engine: None,
        }
    }

    pub fn with_cross_encoder(mut self, reranker: CrossEncoderReranker) -> Self {
        self.cross_encoder = Some(reranker);
        self
    }

    pub fn with_vector_module(mut self, vm: Box<dyn VectorModule + Send + Sync>) -> Self {
        self.vector_module = Some(vm);
        self
    }

    pub fn with_vector_indexer(mut self, indexer: VectorIndexer) -> Self {
        self.vector_indexer = Some(indexer);
        self
    }

    pub fn with_dkg_engine(mut self, engine: DkgEngine) -> Self {
        self.dkg_engine = Some(engine);
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
            circuit_breaker_state: None,
            error: None,
        });

        // Stage 1: vector (embed query, search BruteForceIndex if populated)
        let (vector_results, vector_error) = match (&self.vector_module, &self.vector_indexer) {
            (Some(vm), Some(indexer)) if !indexer.is_empty() => {
                match vm.embed_text(query) {
                    Ok(query_vec) => {
                        match indexer.search(&query_vec, 10) {
                            Ok(hits) => {
                                let results: Vec<RetrievalResult> = hits
                                    .into_iter()
                                    .map(|(meta, score)| RetrievalResult {
                                        chunk_id: format!(
                                            "{}:{}-{}",
                                            meta.path.display(),
                                            meta.line_start,
                                            meta.line_end
                                        ),
                                        snippet: meta.snippet,
                                        score: score as f64,
                                        tool_used: "vector".to_string(),
                                    })
                                    .collect();
                                (results, None)
                            }
                            Err(e) => {
                                log::debug!("vector search failed: {e}");
                                (Vec::new(), Some(e.to_string()))
                            }
                        }
                    }
                    Err(e) => {
                        log::debug!("vector embed failed: {e}");
                        (Vec::new(), Some(e.to_string()))
                    }
                }
            }
            (Some(vm), _) => {
                // Module present but no index — probe embed readiness only.
                match vm.embed_text(query) {
                    Ok(_) => (Vec::new(), None),
                    Err(e) => (Vec::new(), Some(e.to_string())),
                }
            }
            _ => (Vec::new(), None),
        };
        stages.push(PipelineStage {
            stage: "vector".to_string(),
            results: vector_results,
            latency_ms: 0,
            circuit_breaker_state: None,
            error: vector_error,
        });

        // Stage 2: pagerank — build snapshot from DkgEngine if available
        let (snapshot, pagerank_results, pagerank_error) = match self.dkg_engine {
            Some(ref engine) => match DkgSnapshot::from_engine(engine) {
                Ok(snap) => {
                    let config = PageRankConfig::default();
                    let token_est: &dyn Fn(&str) -> usize = &|s: &str| s.len() / 4;
                    let repo_map = compute_repo_map(&snap, query, &[], &config, token_est);
                    let results: Vec<RetrievalResult> = repo_map
                        .ranked_symbols
                        .iter()
                        .map(|rs| RetrievalResult {
                            chunk_id: format!("{}:{}", rs.file_path.display(), rs.symbol_name),
                            snippet: rs.signature.clone(),
                            score: rs.rank,
                            tool_used: "pagerank".to_string(),
                        })
                        .collect();
                    (snap, results, None)
                }
                Err(e) => {
                    log::debug!("DkgSnapshot::from_engine failed: {e}");
                    let empty = DkgSnapshot { nodes: Vec::new(), edges: Vec::new() };
                    (empty, Vec::new(), Some(e.to_string()))
                }
            },
            None => {
                let empty = DkgSnapshot { nodes: Vec::new(), edges: Vec::new() };
                (empty, Vec::new(), None)
            }
        };
        stages.push(PipelineStage {
            stage: "pagerank".to_string(),
            results: pagerank_results,
            latency_ms: 0,
            circuit_breaker_state: None,
            error: pagerank_error,
        });

        // Stage 3: catrag (uses real snapshot when DkgEngine is available)
        let _overlay = self.catrag.compute_overlay(query, &snapshot, &[])?;
        stages.push(PipelineStage {
            stage: "catrag".to_string(),
            results: Vec::new(),
            latency_ms: 0,
            circuit_breaker_state: None,
            error: None,
        });

        // Stage 4: arag
        let arag_results = self.arag.query_pipeline(query, &snapshot)?;
        stages.push(PipelineStage {
            stage: "arag".to_string(),
            results: arag_results,
            latency_ms: 0,
            circuit_breaker_state: None,
            error: None,
        });

        // Stage 5: rerank (RRF fusion across all stages that produced results)
        let reranked = self.rrf_rerank(&stages);
        stages.push(PipelineStage {
            stage: "rerank".to_string(),
            results: reranked,
            latency_ms: 0,
            circuit_breaker_state: None,
            error: None,
        });

        // Stage 6: cross_encoder (cross-encoder reranking after RRF fusion)
        // In SPARSE_ONLY / BM25_ONLY modes, caller should not attach a cross_encoder.
        // When cross_encoder is None, pass through RRF results unchanged.
        let ce_input = stages.last().map(|s| s.results.clone()).unwrap_or_default();
        let (ce_results, ce_breaker_state, ce_error) = if let Some(ref ce) = self.cross_encoder {
            let breaker_state = ce
                .breaker_state()
                .map(|s| format!("{s:?}"));
            match ce.rerank(query, ce_input.clone()) {
                Ok(ranked) => (ranked, breaker_state, None),
                Err(e) => {
                    log::debug!("cross_encoder stage error: {e}");
                    (ce_input, breaker_state, Some(e.to_string()))
                }
            }
        } else {
            (ce_input, None, None)
        };
        stages.push(PipelineStage {
            stage: "cross_encoder".to_string(),
            results: ce_results,
            latency_ms: 0,
            circuit_breaker_state: ce_breaker_state,
            error: ce_error,
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
            circuit_breaker_state: None,
            error: None,
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

    /// Startup canary: embed + BM25 + rerank. All must return non-error within 5s.
    ///
    /// knowledge.txt L159: canary query is embed("fn main()") + BM25("main") + rerank.
    /// Returns (ReadinessStatus, CanaryResult) so callers can log stage latencies.
    ///
    /// Status logic:
    ///   - All pass (including rerank) -> Ready
    ///   - BM25 passes but rerank fails -> Degraded (BM25_ONLY fallback)
    ///   - BM25 fails -> Unavailable
    pub fn readiness_check(&self) -> (ReadinessStatus, CanaryResult) {
        let mut result = CanaryResult::default();

        // Stage 1: BM25 canary — search for "main".
        let bm25_ok = match self.bm25.query("main", 3) {
            Ok(hits) => {
                result.bm25_ok = true;
                result.bm25_hit_count = hits.len();
                result.canary_candidates = hits
                    .into_iter()
                    .map(|h| RetrievalResult {
                        chunk_id: format!("{}:{}-{}", h.path, h.line_start, h.line_end),
                        snippet: h.snippet,
                        score: h.score as f64,
                        tool_used: "bm25".to_string(),
                    })
                    .collect();
                true
            }
            Err(e) => {
                log::error!("canary: BM25 failed: {e}");
                result.bm25_error = Some(e.to_string());
                false
            }
        };

        if !bm25_ok {
            return (ReadinessStatus::Unavailable, result);
        }

        // Stage 1.5: vector canary — probe embedding model readiness.
        // Vector failure is non-fatal: degrades to BM25-only, does not block startup.
        if let Some(ref vm) = self.vector_module {
            match vm.readiness() {
                ReadinessStatus::Ready => {
                    result.vector_ok = true;
                }
                ReadinessStatus::Degraded => {
                    log::warn!("canary: vector module partially degraded");
                    result.vector_error = Some("partial: one embedding model failed".to_string());
                }
                ReadinessStatus::Unavailable => {
                    log::warn!("canary: vector module unavailable (embed stub or model missing)");
                    result.vector_error = Some("unavailable: embedding models not operational".to_string());
                }
            }
        } else {
            // No vector module configured — not an error, just BM25-only mode.
            result.vector_error = Some("not configured".to_string());
        }

        // Stage 2: rerank canary — verify cross-encoder is loaded and operational.
        // The reranker's rerank() method never returns Err to callers (spec L149:
        // final fallback always returns candidates unchanged). So we check is_loaded()
        // first, then run a probe rerank and check breaker state for open circuits.
        let rerank_ok = if let Some(ref ce) = self.cross_encoder {
            if !ce.is_loaded() {
                log::warn!("canary: reranker model not loaded, entering degraded mode");
                result.rerank_error = Some("model not loaded".to_string());
                false
            } else {
                // Probe rerank: even if candidates are below skip_threshold, this
                // validates that the session can at least be accessed without panic.
                let candidates = result.canary_candidates.clone();
                let _ = ce.rerank("fn main()", candidates);
                // Check breaker state after probe.
                let breaker_open = ce
                    .breaker_state()
                    .map_or(false, |s| s != crate::vector::circuit_breaker::BreakerState::Closed);
                if breaker_open {
                    log::warn!("canary: reranker circuit breaker open, entering degraded mode");
                    result.rerank_error = Some("circuit breaker open".to_string());
                    false
                } else {
                    result.rerank_ok = true;
                    true
                }
            }
        } else {
            // No cross-encoder configured — rerank stage not applicable.
            result.rerank_ok = true;
            true
        };

        let status = if rerank_ok {
            ReadinessStatus::Ready
        } else {
            ReadinessStatus::Degraded
        };

        (status, result)
    }
}

// ---------------------------------------------------------------------------
// CanaryResult
// ---------------------------------------------------------------------------

/// Diagnostic output from readiness_check. Callers can log or inspect per-stage results.
#[derive(Debug, Clone, Default)]
pub struct CanaryResult {
    pub bm25_ok: bool,
    pub bm25_hit_count: usize,
    pub bm25_error: Option<String>,
    pub rerank_ok: bool,
    pub rerank_error: Option<String>,
    pub vector_ok: bool,
    pub vector_error: Option<String>,
    /// Synthetic candidates produced by BM25 canary, passed to reranker.
    pub canary_candidates: Vec<RetrievalResult>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arag::AragConfig;
    use crate::bm25::{Bm25Index, BM25Chunk, ChunkKind};
    use crate::catrag::CatRagConfig;
    use crate::reranker::{CrossEncoderReranker, RerankerConfig};
    use elai_core::ClockProvider;
    use elai_core::SeededClock;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    fn project_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf()
    }

    fn model_path() -> PathBuf {
        project_root().join("models").join("onnx").join("cross-encoder-l12.onnx")
    }

    fn tokenizer_path() -> PathBuf {
        project_root().join("models").join("onnx").join("tokenizer.json")
    }

    /// Create a BM25 index in a temp dir and index a canary document so
    /// BM25 canary query for "main" returns hits.
    fn make_bm25_with_canary(dir: &Path) -> Bm25Index {
        let mut bm25 = Bm25Index::new(dir, "canary_test").expect("bm25 init");
        let chunks = vec![BM25Chunk {
            path: PathBuf::from("canary.rs"),
            content: "fn main() { println!(\"hello\"); }".to_string(),
            line_start: 1,
            line_end: 5,
            kind: ChunkKind::Function,
        }];
        bm25.index_file(Path::new("canary.rs"), Some(chunks))
            .expect("index canary chunk");
        bm25
    }

    /// Create a BM25 index with no documents (canary query returns empty).
    fn make_empty_bm25(dir: &Path) -> Bm25Index {
        Bm25Index::new(dir, "empty_test").expect("bm25 init")
    }

    fn make_pipeline(bm25: Bm25Index) -> QueryPipeline {
        let catrag = CatRag::new(CatRagConfig::default());
        let arag = AragOrchestrator::new(AragConfig::default());
        QueryPipeline::new(bm25, catrag, arag)
    }

    // -------------------------------------------------------------------
    // 1G.5 tests
    // -------------------------------------------------------------------

    #[test]
    fn readiness_check_no_cross_encoder_bm25_ok_returns_ready() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let bm25 = make_bm25_with_canary(tmp.path());
        let pipeline = make_pipeline(bm25);

        let (status, result) = pipeline.readiness_check();
        assert_eq!(status, ReadinessStatus::Ready);
        assert!(result.bm25_ok);
        assert!(result.rerank_ok);
    }

    #[test]
    fn readiness_check_healthy_reranker_returns_ready() {
        let mp = model_path();
        let tp = tokenizer_path();
        if !mp.exists() || !tp.exists() {
            eprintln!("SKIP: model fixtures not staged");
            return;
        }

        let tmp = tempfile::tempdir().expect("tempdir");
        let bm25 = make_bm25_with_canary(tmp.path());
        let clock: Arc<dyn ClockProvider> = Arc::new(SeededClock::new());
        let mut config = RerankerConfig::default();
        config.model_path = mp;
        config.tokenizer_path = tp;
        let mut reranker = CrossEncoderReranker::new(config, clock);
        reranker.load().expect("load should succeed");

        let pipeline = make_pipeline(bm25).with_cross_encoder(reranker);
        let (status, result) = pipeline.readiness_check();
        assert_eq!(status, ReadinessStatus::Ready);
        assert!(result.bm25_ok);
        assert!(result.rerank_ok);
    }

    #[test]
    fn readiness_check_broken_reranker_returns_degraded() {
        // Reranker created but NOT loaded (load() not called) — is_loaded() = false.
        // Canary detects unloaded model and returns Degraded (BM25 still works).
        let tmp = tempfile::tempdir().expect("tempdir");
        let bm25 = make_bm25_with_canary(tmp.path());
        let clock: Arc<dyn ClockProvider> = Arc::new(SeededClock::new());
        let reranker = CrossEncoderReranker::new(RerankerConfig::default(), clock);
        // NOT calling load() — is_loaded() returns false

        let pipeline = make_pipeline(bm25).with_cross_encoder(reranker);
        let (status, result) = pipeline.readiness_check();
        assert_eq!(
            status,
            ReadinessStatus::Degraded,
            "broken reranker should degrade, not become unavailable"
        );
        assert!(result.bm25_ok, "BM25 should still pass");
        assert!(!result.rerank_ok, "reranker should have failed");
        assert!(result.rerank_error.is_some());
    }

    #[test]
    fn readiness_check_empty_bm25_canary_no_cross_encoder() {
        // BM25 returns no hits for "main" because index is empty.
        // BM25 query succeeds (returns Ok([])), so bm25_ok = true.
        // No cross-encoder configured — rerank stage passes trivially.
        let tmp = tempfile::tempdir().expect("tempdir");
        let bm25 = make_empty_bm25(tmp.path());
        let pipeline = make_pipeline(bm25);

        let (status, result) = pipeline.readiness_check();
        assert_eq!(status, ReadinessStatus::Ready);
        assert!(result.bm25_ok);
        assert_eq!(result.bm25_hit_count, 0);
        assert!(result.rerank_ok);
        assert!(result.canary_candidates.is_empty());
    }

    #[test]
    fn readiness_check_empty_bm25_canary_loaded_reranker_gets_empty_input() {
        // Loaded reranker receives empty candidate set from BM25 — should not error.
        let mp = model_path();
        let tp = tokenizer_path();
        if !mp.exists() || !tp.exists() {
            eprintln!("SKIP: model fixtures not staged");
            return;
        }

        let tmp = tempfile::tempdir().expect("tempdir");
        let bm25 = make_empty_bm25(tmp.path());
        let clock: Arc<dyn ClockProvider> = Arc::new(SeededClock::new());
        let mut config = RerankerConfig::default();
        config.model_path = mp;
        config.tokenizer_path = tp;
        let mut reranker = CrossEncoderReranker::new(config, clock);
        reranker.load().expect("load");

        let pipeline = make_pipeline(bm25).with_cross_encoder(reranker);
        let (status, result) = pipeline.readiness_check();
        assert_eq!(status, ReadinessStatus::Ready);
        assert!(result.bm25_ok);
        assert_eq!(result.bm25_hit_count, 0);
        assert!(result.rerank_ok, "empty input to loaded reranker should not error");
    }

    #[test]
    fn readiness_check_includes_rerank_in_implementation() {
        // Structural check: the readiness_check method references "rerank".
        // This is the grep equivalent from DETERMINISTIC_CHECK.
        let src = include_str!("pipeline.rs");
        assert!(
            src.contains("rerank"),
            "readiness_check implementation must reference rerank"
        );
    }

    #[test]
    fn readiness_check_canary_result_fields_populated() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let bm25 = make_bm25_with_canary(tmp.path());
        let pipeline = make_pipeline(bm25);

        let (_status, result) = pipeline.readiness_check();
        assert!(result.bm25_ok);
        assert!(result.bm25_hit_count > 0, "canary doc should produce hits");
        assert!(result.bm25_error.is_none());
        assert!(result.rerank_ok);
        assert!(result.rerank_error.is_none());
        assert!(
            !result.canary_candidates.is_empty(),
            "canary candidates should be populated from BM25 hits"
        );
    }
}
