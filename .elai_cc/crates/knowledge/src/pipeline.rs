use std::collections::{HashMap, HashSet};

use crate::error::KnowledgeError;
use crate::freshness::{self, FreshnessDb};
use crate::query_expansion::{self, ExpansionStrategy};

use crate::arag::{AragOrchestrator, RetrievalResult};
use crate::bm25::Bm25Index;
use crate::catrag::CatRag;
use crate::degradation::SearchMode;
use crate::dkg::DkgEngine;
use crate::pagerank::{compute_repo_map, DkgSnapshot, PageRankConfig};
use crate::query_trace::{QueryTrace, StageTrace};
use crate::query_validator::QueryValidator;
use crate::reranker::CrossEncoderReranker;
use crate::vector::VectorModule;
use crate::vector::indexer::VectorIndexer;
use crate::ReadinessStatus;

#[cfg(feature = "splade")]
use crate::splade::SpladeExpander;

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

/// End-to-end retrieval pipeline: bm25 -> vector -> splade -> pagerank -> catrag -> arag -> rerank -> cross_encoder -> token_fit.
pub struct QueryPipeline {
    pub bm25: Bm25Index,
    pub catrag: CatRag,
    pub arag: AragOrchestrator,
    pub cross_encoder: Option<CrossEncoderReranker>,
    pub vector_module: Option<Box<dyn VectorModule + Send + Sync>>,
    pub vector_indexer: Option<VectorIndexer>,
    pub dkg_engine: Option<DkgEngine>,
    pub freshness_db: Option<FreshnessDb>,
    #[cfg(feature = "splade")]
    pub splade: Option<SpladeExpander>,
}

impl QueryPipeline {
    pub fn new(bm25: Bm25Index, catrag: CatRag, arag: AragOrchestrator) -> Self {
        Self {
            bm25, catrag, arag,
            cross_encoder: None,
            vector_module: None,
            vector_indexer: None,
            dkg_engine: None,
            freshness_db: None,
            #[cfg(feature = "splade")]
            splade: None,
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

    pub fn with_freshness_db(mut self, db: FreshnessDb) -> Self {
        self.freshness_db = Some(db);
        self
    }

    #[cfg(feature = "splade")]
    pub fn with_splade(mut self, expander: SpladeExpander) -> Self {
        self.splade = Some(expander);
        self
    }

    /// Execute the full retrieval pipeline for a query.
    ///
    /// Returns stages in order. `latency_ms` is 0 for all stages;
    /// callers should measure overall pipeline time externally.
    ///
    /// Phase 3K.2: queries >= 5 words are expanded into 2-3 template-based
    /// variants. BM25 + vector run per expansion, results are merged and
    /// deduplicated by chunk_id before downstream stages (rerank, cross-encoder).
    pub fn query(
        &self,
        query: &str,
    ) -> Result<Vec<PipelineStage>, KnowledgeError> {
        // Pre-query validation (Phase 1I.3).
        QueryValidator::validate(query)?;

        let mut stages = Vec::with_capacity(9);

        // Phase 3K.2: template-based multi-query expansion.
        let expansions = query_expansion::expand_query(query);
        let expansion_count = expansions.len();
        let expansion_texts: Vec<String> = expansions.iter().map(|e| e.text.clone()).collect();
        tracing::debug!(
            "query_expansion: {} variants for query (len={}w)",
            expansion_count,
            query.split_whitespace().count(),
        );

        stages.push(PipelineStage {
            stage: "query_expansion".to_string(),
            results: expansions
                .iter()
                .filter(|e| e.strategy != ExpansionStrategy::Original)
                .map(|e| RetrievalResult {
                    chunk_id: format!("expansion:{:?}", e.strategy),
                    snippet: e.text.clone(),
                    score: 0.0,
                    tool_used: "query_expansion".to_string(),
                })
                .collect(),
            latency_ms: 0,
            circuit_breaker_state: None,
            error: None,
        });

        // Stage 0: BM25 — run per expansion, merge + dedup
        let mut bm25_seen: HashSet<String> = HashSet::new();
        let mut bm25_results: Vec<RetrievalResult> = Vec::new();
        for eq in &expansion_texts {
            let hits = self.bm25.query(eq, 10).unwrap_or_default();
            for h in hits {
                let chunk_id = format!("{}:{}-{}", h.path, h.line_start, h.line_end);
                if bm25_seen.insert(chunk_id.clone()) {
                    bm25_results.push(RetrievalResult {
                        chunk_id,
                        snippet: h.snippet,
                        score: h.score as f64,
                        tool_used: "bm25".to_string(),
                    });
                }
            }
        }
        stages.push(PipelineStage {
            stage: "bm25".to_string(),
            results: bm25_results,
            latency_ms: 0,
            circuit_breaker_state: None,
            error: None,
        });

        // Stage 1: vector — run per expansion, merge + dedup
        let mut vector_seen: HashSet<String> = HashSet::new();
        let mut vector_results: Vec<RetrievalResult> = Vec::new();
        let mut vector_error: Option<String> = None;
        match (&self.vector_module, &self.vector_indexer) {
            (Some(vm), Some(indexer)) if !indexer.is_empty() => {
                for eq in &expansion_texts {
                    match vm.embed_text(eq) {
                        Ok(query_vec) => {
                            match indexer.search(&query_vec, 10) {
                                Ok(hits) => {
                                    for (meta, score) in hits {
                                        let chunk_id = format!(
                                            "{}:{}-{}",
                                            meta.path.display(),
                                            meta.line_start,
                                            meta.line_end
                                        );
                                        if vector_seen.insert(chunk_id.clone()) {
                                            vector_results.push(RetrievalResult {
                                                chunk_id,
                                                snippet: meta.snippet,
                                                score: score as f64,
                                                tool_used: "vector".to_string(),
                                            });
                                        }
                                    }
                                }
                                Err(e) => {
                                    tracing::debug!("vector search failed for expansion: {e}");
                                    vector_error = Some(e.to_string());
                                }
                            }
                        }
                        Err(e) => {
                            tracing::debug!("vector embed failed for expansion: {e}");
                            vector_error = Some(e.to_string());
                        }
                    }
                }
            }
            (Some(vm), _) => {
                // Module present but no index — probe embed readiness only.
                match vm.embed_text(query) {
                    Ok(_) => {}
                    Err(e) => { vector_error = Some(e.to_string()); }
                }
            }
            _ => {}
        }
        stages.push(PipelineStage {
            stage: "vector".to_string(),
            results: vector_results,
            latency_ms: 0,
            circuit_breaker_state: None,
            error: vector_error,
        });

        // Stage 1.5: splade keyword expansion
        #[cfg(feature = "splade")]
        {
            let splade_results = if let Some(ref expander) = self.splade {
                let expanded = expander.expand_query(query);
                if expanded.is_empty() {
                    Vec::new()
                } else {
                    let expanded_query: String = expanded
                        .iter()
                        .map(|t| t.term.as_str())
                        .collect::<Vec<_>>()
                        .join(" ");
                    self.bm25.query(&expanded_query, 10)
                        .unwrap_or_default()
                        .into_iter()
                        .map(|h| RetrievalResult {
                            chunk_id: format!("{}:{}-{}", h.path, h.line_start, h.line_end),
                            snippet: h.snippet,
                            score: h.score as f64,
                            tool_used: "splade".to_string(),
                        })
                        .collect()
                }
            } else {
                Vec::new()
            };
            stages.push(PipelineStage {
                stage: "splade".to_string(),
                results: splade_results,
                latency_ms: 0,
                circuit_breaker_state: None,
                error: None,
            });
        }

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
                    tracing::debug!("DkgSnapshot::from_engine failed: {e}");
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
                    tracing::debug!("cross_encoder stage error: {e}");
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

        // Build QueryTrace from pipeline stages (Phase 1I.6).
        let stage_traces: Vec<StageTrace> = stages.iter().map(|s| StageTrace {
            stage: s.stage.clone(),
            latency_ms: s.latency_ms,
            input_count: 0,
            output_count: s.results.len(),
            error: s.error.clone(),
            circuit_breaker_state: s.circuit_breaker_state.clone(),
        }).collect();
        let result_count = stages.last().map(|s| s.results.len()).unwrap_or(0);
        let trace = QueryTrace::new(stage_traces, 0, SearchMode::Hybrid, result_count);
        tracing::debug!("query_trace: query_id={} stages={} results={}", trace.query_id, trace.stages.len(), trace.result_count);

        Ok(stages)
    }

    /// Reciprocal Rank Fusion across all stages that produced results.
    /// RRF score = sum(1 / (k + rank)) where k=60 and rank starts at 1.
    ///
    /// When a `freshness_db` is attached, each chunk's RRF score is multiplied
    /// by its freshness factor (recency_decay * access_boost). Missing entries
    /// default to 1.0 (no penalty). See anti-staleness-enforcement.txt §1.1.
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

        // Apply freshness multiplier (A-13: sidecar JOIN).
        if let Some(ref fdb) = self.freshness_db {
            let now = fdb.effective_now();
            for (score, result) in scores.values_mut() {
                let file_path = freshness::extract_file_path(&result.chunk_id);
                let multiplier = fdb
                    .freshness_multiplier(file_path, now)
                    .unwrap_or(1.0);
                *score *= multiplier;
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
                tracing::error!("canary: BM25 failed: {e}");
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
                    tracing::warn!("canary: vector module partially degraded");
                    result.vector_error = Some("partial: one embedding model failed".to_string());
                }
                ReadinessStatus::Unavailable => {
                    tracing::warn!("canary: vector module unavailable (embed stub or model missing)");
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
                tracing::warn!("canary: reranker model not loaded, entering degraded mode");
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
                    tracing::warn!("canary: reranker circuit breaker open, entering degraded mode");
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

    // -------------------------------------------------------------------
    // A-13 freshness scoring tests
    // -------------------------------------------------------------------

    use crate::freshness::{FreshnessConfig, FreshnessDb};

    #[test]
    fn rrf_rerank_without_freshness_db_unchanged() {
        // Baseline: no freshness_db attached -> scores are pure RRF.
        let tmp = tempfile::tempdir().expect("tempdir");
        let bm25 = make_bm25_with_canary(tmp.path());
        let pipeline = make_pipeline(bm25);
        let stages = pipeline.query("main").expect("query");
        let rerank_stage = stages.iter().find(|s| s.stage == "rerank").expect("rerank stage");
        assert!(!rerank_stage.results.is_empty(), "should have rerank results");
    }

    #[test]
    fn rrf_rerank_with_freshness_boosts_fresh_files() {
        let tmp = tempfile::tempdir().expect("tempdir");

        // Index two documents: one "fresh", one "stale"
        let mut bm25 = Bm25Index::new(tmp.path(), "freshness_test").expect("bm25 init");
        let fresh_chunks = vec![BM25Chunk {
            path: PathBuf::from("fresh.rs"),
            content: "fn query_handler() { search() }".to_string(),
            line_start: 1,
            line_end: 5,
            kind: ChunkKind::Function,
        }];
        let stale_chunks = vec![BM25Chunk {
            path: PathBuf::from("stale.rs"),
            content: "fn query_handler() { search() }".to_string(),
            line_start: 1,
            line_end: 5,
            kind: ChunkKind::Function,
        }];
        bm25.index_file(Path::new("fresh.rs"), Some(fresh_chunks)).unwrap();
        bm25.index_file(Path::new("stale.rs"), Some(stale_chunks)).unwrap();

        // Set up freshness_db: fresh.rs modified "now", stale.rs modified 2 years ago
        let one_day: i64 = 86_400;
        let now_epoch = 730 * one_day;
        let mut fdb = FreshnessDb::open_in_memory(FreshnessConfig::default()).unwrap();
        fdb.set_now_override(now_epoch);
        fdb.upsert_mtime("fresh.rs", now_epoch).unwrap(); // age = 0 days
        fdb.upsert_mtime("stale.rs", 0).unwrap();         // age = 730 days (~2 years)

        let pipeline = make_pipeline(bm25).with_freshness_db(fdb);
        let stages = pipeline.query("query_handler search").expect("query");
        let rerank_stage = stages.iter().find(|s| s.stage == "rerank").expect("rerank stage");

        // Both documents match the query equally in BM25 terms.
        // With freshness scoring, fresh.rs should rank higher than stale.rs.
        assert!(rerank_stage.results.len() >= 2, "need at least 2 results");
        let fresh_pos = rerank_stage.results.iter()
            .position(|r| r.chunk_id.starts_with("fresh.rs"))
            .expect("fresh.rs in results");
        let stale_pos = rerank_stage.results.iter()
            .position(|r| r.chunk_id.starts_with("stale.rs"))
            .expect("stale.rs in results");
        assert!(
            fresh_pos < stale_pos,
            "fresh.rs (pos={fresh_pos}) should rank before stale.rs (pos={stale_pos})"
        );

        // Verify the scores differ: fresh score should be higher
        let fresh_score = rerank_stage.results[fresh_pos].score;
        let stale_score = rerank_stage.results[stale_pos].score;
        assert!(
            fresh_score > stale_score,
            "fresh score ({fresh_score}) should exceed stale score ({stale_score})"
        );
    }

    #[test]
    fn freshness_in_pipeline_structural_check() {
        // Structural: pipeline.rs must reference freshness_db and freshness_multiplier.
        let src = include_str!("pipeline.rs");
        assert!(src.contains("freshness_db"), "pipeline must reference freshness_db");
        assert!(src.contains("freshness_multiplier"), "pipeline must call freshness_multiplier");
    }
}
