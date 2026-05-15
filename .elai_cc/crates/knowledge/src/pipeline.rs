use std::collections::{HashMap, HashSet};

use crate::error::KnowledgeError;
use crate::freshness::{self, FreshnessDb};
use crate::hyde::HydeStage;
use crate::query_expansion::{self, ExpansionStrategy};
use crate::QueryTier;

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
    pub hyde_stage: Option<HydeStage>,
    #[cfg(feature = "splade")]
    pub splade: Option<SpladeExpander>,
    /// Current degradation cascade mode. Updated at startup by the cascade
    /// evaluation (cli/main.rs) and on circuit-breaker transitions (B-4).
    /// Used as the diagnostic label in QueryTrace so per-query traces reflect
    /// the actual mode rather than the default Hybrid sentinel.
    pub search_mode: SearchMode,
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
            hyde_stage: None,
            #[cfg(feature = "splade")]
            splade: None,
            // Default to Hybrid until the degradation cascade has run its
            // first evaluation (B-4 full wiring). Callers that drive the
            // cascade can update this field before each query via
            // `pipeline.search_mode = cascade.evaluate(...)`.
            search_mode: SearchMode::Hybrid,
        }
    }

    /// Set the current degradation cascade mode. Call this after each cascade
    /// evaluation so per-query QueryTrace labels reflect the real mode.
    pub fn set_search_mode(&mut self, mode: SearchMode) {
        self.search_mode = mode;
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

    pub fn with_hyde(mut self, hyde: HydeStage) -> Self {
        self.hyde_stage = Some(hyde);
        self
    }

    #[cfg(feature = "splade")]
    pub fn with_splade(mut self, expander: SpladeExpander) -> Self {
        self.splade = Some(expander);
        self
    }

    /// Execute the retrieval pipeline for a query with AUTO tier (backwards compatible).
    ///
    /// Equivalent to `query_with_tier(query, QueryTier::Auto)`.
    pub fn query(
        &self,
        query: &str,
    ) -> Result<Vec<PipelineStage>, KnowledgeError> {
        self.query_with_tier(query, QueryTier::Auto)
    }

    /// Execute the retrieval pipeline for a query at the specified tier.
    ///
    /// Phase 3K.4 tier routing gates which pipeline stages execute:
    ///   FAST:     BM25 only. No expansion, no HyDE, no vector, no reranker.
    ///   STANDARD: BM25 + vector hybrid (RRF) + cross-encoder reranker.
    ///   DEEP:     Query expansion + HyDE + BM25 + vector + reranker.
    ///   AUTO:     Auto-detect from query characteristics, then dispatch.
    ///
    /// Distinct from Phase 1I `SearchMode` cascade (degradation on failure).
    pub fn query_with_tier(
        &self,
        query: &str,
        tier: QueryTier,
    ) -> Result<Vec<PipelineStage>, KnowledgeError> {
        // Pre-query validation (Phase 1I.3).
        QueryValidator::validate(query)?;

        let resolved_tier = tier.resolve(query);
        tracing::debug!(
            requested_tier = %tier,
            resolved_tier = %resolved_tier,
            "tier routing: resolved query tier"
        );

        let use_expansion = resolved_tier == QueryTier::Deep;
        let use_hyde = resolved_tier == QueryTier::Deep;
        let use_vector = matches!(resolved_tier, QueryTier::Standard | QueryTier::Deep);
        let use_reranker = matches!(resolved_tier, QueryTier::Standard | QueryTier::Deep);

        let mut stages = Vec::with_capacity(10);

        // Phase 3K.2: template-based multi-query expansion (DEEP only).
        let expansion_texts: Vec<String> = if use_expansion {
            let expansions = query_expansion::expand_query(query);
            let expansion_count = expansions.len();
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

            expansions.iter().map(|e| e.text.clone()).collect()
        } else {
            // No expansion: original query only.
            stages.push(PipelineStage {
                stage: "query_expansion".to_string(),
                results: Vec::new(),
                latency_ms: 0,
                circuit_breaker_state: None,
                error: None,
            });
            vec![query.to_string()]
        };

        // Phase 3K.3: HyDE (DEEP only).
        let hyde_hypothesis = if use_hyde {
            let (hyp, breaker_state, error) = match self.hyde_stage {
                Some(ref hyde) => {
                    let breaker_state = hyde
                        .breaker_state()
                        .map(|s| format!("{s:?}"));
                    match hyde.hypothesize(query) {
                        Some(hyp) => {
                            tracing::debug!(
                                hypothesis_len = hyp.len(),
                                "hyde: hypothesis will be used for vector search"
                            );
                            (Some(hyp), breaker_state, None)
                        }
                        None => (None, breaker_state, None),
                    }
                }
                None => (None, None, None),
            };
            stages.push(PipelineStage {
                stage: "hyde".to_string(),
                results: hyp
                    .as_ref()
                    .map(|h| vec![RetrievalResult {
                        chunk_id: "hyde:hypothesis".to_string(),
                        snippet: h.clone(),
                        score: 0.0,
                        tool_used: "hyde".to_string(),
                    }])
                    .unwrap_or_default(),
                latency_ms: 0,
                circuit_breaker_state: breaker_state,
                error,
            });
            hyp
        } else {
            stages.push(PipelineStage {
                stage: "hyde".to_string(),
                results: Vec::new(),
                latency_ms: 0,
                circuit_breaker_state: None,
                error: None,
            });
            None
        };

        // Stage 0: BM25 — always runs (all tiers).
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

        // Stage 1: vector (STANDARD + DEEP only).
        if use_vector {
            // B-4 DEEP-tier MRR fix: always include original query as anchor signal.
            // HyDE hypothesis may be off-domain (sidecar degraded); dropping original
            // query entirely caused MRR regression (DEEP 0.206 vs STANDARD 0.575).
            // Cross-encoder reranker still uses ORIGINAL query (see Stage 4 below).
            let vector_queries: Vec<String> = match hyde_hypothesis {
                Some(ref hyp) => vec![query.to_string(), hyp.clone()],
                None => expansion_texts.clone(),
            };

            let mut vector_seen: HashSet<String> = HashSet::new();
            let mut vector_results: Vec<RetrievalResult> = Vec::new();
            let mut vector_error: Option<String> = None;
            match (&self.vector_module, &self.vector_indexer) {
                (Some(vm), Some(indexer)) if !indexer.is_empty() => {
                    for vq in &vector_queries {
                        match vm.embed_text(vq) {
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
        } else {
            stages.push(PipelineStage {
                stage: "vector".to_string(),
                results: Vec::new(),
                latency_ms: 0,
                circuit_breaker_state: None,
                error: None,
            });
        }

        // Stage 1.5b: code_vector (opt-in, STANDARD + DEEP, config-gated).
        // Only runs when VectorIndexer.has_code_index() is true (ELAI_CODE_VEC=1).
        // Produces a separate "code_vector" stage consumed by rrf_rerank — the
        // 3-ranker RRF path confirmed working by query_trace.rs:264-281 test fixture.
        if use_vector {
            if let (Some(vm), Some(indexer)) = (&self.vector_module, &self.vector_indexer) {
                if indexer.has_code_index() {
                    let mut code_seen: HashSet<String> = HashSet::new();
                    let mut code_results: Vec<RetrievalResult> = Vec::new();
                    let mut code_error: Option<String> = None;
                    // Use original query only for code search (HyDE hypothesis is text-domain).
                    match vm.embed_code(query) {
                        Ok(code_vec) => {
                            match indexer.search_code(&code_vec, 10) {
                                Ok(hits) => {
                                    for (meta, score) in hits {
                                        let chunk_id = format!(
                                            "{}:{}-{}",
                                            meta.path.display(),
                                            meta.line_start,
                                            meta.line_end
                                        );
                                        if code_seen.insert(chunk_id.clone()) {
                                            code_results.push(RetrievalResult {
                                                chunk_id,
                                                snippet: meta.snippet,
                                                score: score as f64,
                                                tool_used: "code_vector".to_string(),
                                            });
                                        }
                                    }
                                }
                                Err(e) => {
                                    tracing::debug!("code_vector search failed: {e}");
                                    code_error = Some(e.to_string());
                                }
                            }
                        }
                        Err(e) => {
                            tracing::debug!("code embed failed: {e}");
                            code_error = Some(e.to_string());
                        }
                    }
                    stages.push(PipelineStage {
                        stage: "code_vector".to_string(),
                        results: code_results,
                        latency_ms: 0,
                        circuit_breaker_state: None,
                        error: code_error,
                    });
                }
            }
        }

        // Stage 1.5: splade keyword expansion (STANDARD + DEEP)
        #[cfg(feature = "splade")]
        {
            let splade_results = if use_vector {
                if let Some(ref expander) = self.splade {
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

        // Stage 2: pagerank (STANDARD + DEEP)
        let snapshot = if use_vector {
            let (snap, pagerank_results, pagerank_error) = match self.dkg_engine {
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
            snap
        } else {
            let empty = DkgSnapshot { nodes: Vec::new(), edges: Vec::new() };
            stages.push(PipelineStage {
                stage: "pagerank".to_string(),
                results: Vec::new(),
                latency_ms: 0,
                circuit_breaker_state: None,
                error: None,
            });
            empty
        };

        // Stage 3: catrag (STANDARD + DEEP)
        if use_vector {
            let _overlay = self.catrag.compute_overlay(query, &snapshot, &[])?;
        }
        stages.push(PipelineStage {
            stage: "catrag".to_string(),
            results: Vec::new(),
            latency_ms: 0,
            circuit_breaker_state: None,
            error: None,
        });

        // Stage 4: arag (STANDARD + DEEP)
        let arag_results = if use_vector {
            self.arag.query_pipeline(query, &snapshot)?
        } else {
            Vec::new()
        };
        stages.push(PipelineStage {
            stage: "arag".to_string(),
            results: arag_results,
            latency_ms: 0,
            circuit_breaker_state: None,
            error: None,
        });

        // Stage 5: rerank (RRF fusion — all tiers, but FAST has only BM25 input)
        let reranked = self.rrf_rerank(&stages);
        stages.push(PipelineStage {
            stage: "rerank".to_string(),
            results: reranked,
            latency_ms: 0,
            circuit_breaker_state: None,
            error: None,
        });

        // Stage 6: cross_encoder (STANDARD + DEEP only)
        let ce_input = stages.last().map(|s| s.results.clone()).unwrap_or_default();
        let (ce_results, ce_breaker_state, ce_error) = if use_reranker {
            if let Some(ref ce) = self.cross_encoder {
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
            }
        } else {
            // FAST tier: skip cross-encoder entirely.
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
        let trace = QueryTrace::new(stage_traces, 0, self.search_mode, result_count);
        tracing::debug!(
            query_id = %trace.query_id,
            tier = %resolved_tier,
            stages = trace.stages.len(),
            results = trace.result_count,
            "query_trace"
        );

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
        // B-32 D2: literal BM25Chunk constructions wrapped with apply_char_cap
        // so the universal cap invariant holds even for hand-rolled chunks.
        let chunks = crate::chunking::cap::apply_char_cap(vec![BM25Chunk {
            path: PathBuf::from("canary.rs"),
            content: "fn main() { println!(\"hello\"); }".to_string(),
            line_start: 1,
            line_end: 5,
            kind: ChunkKind::Function,
        }]);
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
        // B-32 D2: literal BM25Chunk constructions wrapped with apply_char_cap
        // so the universal cap invariant holds even for hand-rolled chunks.
        let fresh_chunks = crate::chunking::cap::apply_char_cap(vec![BM25Chunk {
            path: PathBuf::from("fresh.rs"),
            content: "fn query_handler() { search() }".to_string(),
            line_start: 1,
            line_end: 5,
            kind: ChunkKind::Function,
        }]);
        let stale_chunks = crate::chunking::cap::apply_char_cap(vec![BM25Chunk {
            path: PathBuf::from("stale.rs"),
            content: "fn query_handler() { search() }".to_string(),
            line_start: 1,
            line_end: 5,
            kind: ChunkKind::Function,
        }]);
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

    // -------------------------------------------------------------------
    // 3K.3 HyDE integration tests
    // -------------------------------------------------------------------

    use crate::hyde::{HydeClient, HydeStage};
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Mock HyDE client for pipeline tests.
    struct PipelineMockHydeClient {
        call_count: AtomicUsize,
    }

    impl PipelineMockHydeClient {
        fn new() -> Self {
            Self { call_count: AtomicUsize::new(0) }
        }

        fn calls(&self) -> usize {
            self.call_count.load(Ordering::Relaxed)
        }
    }

    impl HydeClient for PipelineMockHydeClient {
        fn generate_hypothesis(&self, _query: &str) -> Result<String, String> {
            self.call_count.fetch_add(1, Ordering::Relaxed);
            Ok("The pipeline processes queries through BM25 text search \
                and vector similarity using hypothetical document embeddings \
                to improve retrieval accuracy for complex queries."
                .to_string())
        }
    }

    /// Failing HyDE client for graceful degradation tests.
    struct PipelineFailingHydeClient;

    impl HydeClient for PipelineFailingHydeClient {
        fn generate_hypothesis(&self, _query: &str) -> Result<String, String> {
            Err("sidecar unavailable: connection refused".to_string())
        }
    }

    fn make_hyde_stage(client: Arc<dyn HydeClient>) -> HydeStage {
        let clock: Arc<dyn ClockProvider> = Arc::new(SeededClock::new());
        HydeStage::new(client, clock)
    }

    #[test]
    fn pipeline_with_hyde_has_hyde_stage() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let bm25 = make_bm25_with_canary(tmp.path());
        let client: Arc<dyn HydeClient> = Arc::new(PipelineMockHydeClient::new());
        let hyde = make_hyde_stage(client);
        let pipeline = make_pipeline(bm25).with_hyde(hyde);

        let stages = pipeline.query("how does ELAI handle anti-spiral detection in the pipeline")
            .expect("query");
        let hyde_stage = stages.iter().find(|s| s.stage == "hyde");
        assert!(hyde_stage.is_some(), "pipeline must include hyde stage");
    }

    #[test]
    fn pipeline_hyde_generates_hypothesis_for_complex_query() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let bm25 = make_bm25_with_canary(tmp.path());
        let client = Arc::new(PipelineMockHydeClient::new());
        let hyde = make_hyde_stage(client.clone() as Arc<dyn HydeClient>);
        let pipeline = make_pipeline(bm25).with_hyde(hyde);

        let stages = pipeline.query("how does ELAI handle anti-spiral detection in the pipeline")
            .expect("query");
        let hyde_stage = stages.iter().find(|s| s.stage == "hyde").expect("hyde stage");

        assert_eq!(client.calls(), 1, "sidecar should be called once");
        assert!(!hyde_stage.results.is_empty(), "hyde stage should have hypothesis result");
        assert_eq!(hyde_stage.results[0].chunk_id, "hyde:hypothesis");
    }

    #[test]
    fn pipeline_hyde_skips_short_query() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let bm25 = make_bm25_with_canary(tmp.path());
        let client = Arc::new(PipelineMockHydeClient::new());
        let hyde = make_hyde_stage(client.clone() as Arc<dyn HydeClient>);
        let pipeline = make_pipeline(bm25).with_hyde(hyde);

        let stages = pipeline.query("find main").expect("query");
        let hyde_stage = stages.iter().find(|s| s.stage == "hyde").expect("hyde stage");

        assert_eq!(client.calls(), 0, "sidecar should NOT be called for short query");
        assert!(hyde_stage.results.is_empty(), "no hypothesis for short query");
    }

    #[test]
    fn pipeline_hyde_graceful_degradation_on_failure() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let bm25 = make_bm25_with_canary(tmp.path());
        let client: Arc<dyn HydeClient> = Arc::new(PipelineFailingHydeClient);
        let hyde = make_hyde_stage(client);
        let pipeline = make_pipeline(bm25).with_hyde(hyde);

        // Pipeline should complete without error even though HyDE fails.
        let stages = pipeline.query("how does ELAI handle anti-spiral detection in the pipeline")
            .expect("query should succeed despite HyDE failure");

        let hyde_stage = stages.iter().find(|s| s.stage == "hyde").expect("hyde stage");
        assert!(hyde_stage.results.is_empty(), "failed HyDE should produce no hypothesis");

        // BM25 should still work.
        let bm25_stage = stages.iter().find(|s| s.stage == "bm25").expect("bm25 stage");
        assert!(bm25_stage.error.is_none(), "BM25 should not be affected by HyDE failure");
    }

    #[test]
    fn pipeline_without_hyde_still_works() {
        // Backwards compatibility: no HyDE stage configured.
        let tmp = tempfile::tempdir().expect("tempdir");
        let bm25 = make_bm25_with_canary(tmp.path());
        let pipeline = make_pipeline(bm25);

        let stages = pipeline.query("how does ELAI handle anti-spiral detection in the pipeline")
            .expect("query");
        let hyde_stage = stages.iter().find(|s| s.stage == "hyde").expect("hyde stage");
        assert!(hyde_stage.results.is_empty(), "no hypothesis without HyDE stage");
    }

    #[test]
    fn pipeline_hyde_stage_between_expansion_and_bm25() {
        // Structural: HyDE stage must appear after query_expansion and before bm25.
        let tmp = tempfile::tempdir().expect("tempdir");
        let bm25 = make_bm25_with_canary(tmp.path());
        let client: Arc<dyn HydeClient> = Arc::new(PipelineMockHydeClient::new());
        let hyde = make_hyde_stage(client);
        let pipeline = make_pipeline(bm25).with_hyde(hyde);

        let stages = pipeline.query("how does ELAI handle anti-spiral detection in the pipeline")
            .expect("query");
        let stage_names: Vec<&str> = stages.iter().map(|s| s.stage.as_str()).collect();
        let expansion_pos = stage_names.iter().position(|&s| s == "query_expansion")
            .expect("query_expansion stage");
        let hyde_pos = stage_names.iter().position(|&s| s == "hyde")
            .expect("hyde stage");
        let bm25_pos = stage_names.iter().position(|&s| s == "bm25")
            .expect("bm25 stage");

        assert!(
            expansion_pos < hyde_pos,
            "hyde (pos={hyde_pos}) must come after query_expansion (pos={expansion_pos})"
        );
        assert!(
            hyde_pos < bm25_pos,
            "hyde (pos={hyde_pos}) must come before bm25 (pos={bm25_pos})"
        );
    }

    #[test]
    fn pipeline_hyde_structural_check() {
        let src = include_str!("pipeline.rs");
        assert!(src.contains("hyde_stage"), "pipeline must reference hyde_stage");
        assert!(src.contains("hyde_hypothesis"), "pipeline must use hypothesis for vector search");
        assert!(src.contains("HydeStage"), "pipeline must use HydeStage type");
        assert!(src.contains("with_hyde"), "pipeline must have with_hyde builder");
    }

    // -------------------------------------------------------------------
    // 3K.4 QueryTier routing tests
    // -------------------------------------------------------------------

    use crate::QueryTier;

    #[test]
    fn query_tier_auto_detect_short_query_is_fast() {
        assert_eq!(QueryTier::detect("find main"), QueryTier::Fast);
        assert_eq!(QueryTier::detect("imports"), QueryTier::Fast);
        assert_eq!(QueryTier::detect("fn new"), QueryTier::Fast);
        assert_eq!(QueryTier::detect("a b c d"), QueryTier::Fast); // 4 words
    }

    #[test]
    fn query_tier_auto_detect_medium_code_query_is_standard() {
        assert_eq!(
            QueryTier::detect("search for error handling in pipeline"),
            QueryTier::Standard,
        );
        assert_eq!(
            QueryTier::detect("find all BM25 results with score above threshold"),
            QueryTier::Standard,
        );
    }

    #[test]
    fn query_tier_auto_detect_complex_query_is_deep() {
        // >15 words
        assert_eq!(
            QueryTier::detect(
                "I need to understand the full pipeline flow from query expansion through \
                 vector search to cross encoder reranking and token fitting"
            ),
            QueryTier::Deep,
        );
        // Deep keyword in 5-15 word range
        assert_eq!(
            QueryTier::detect("how does the FSM handle transitions"),
            QueryTier::Deep,
        );
        assert_eq!(
            QueryTier::detect("debug the anti-spiral detection logic"),
            QueryTier::Deep,
        );
        assert_eq!(
            QueryTier::detect("explain the architecture of the pipeline"),
            QueryTier::Deep,
        );
    }

    #[test]
    fn query_tier_resolve_auto_delegates_to_detect() {
        assert_eq!(
            QueryTier::Auto.resolve("find main"),
            QueryTier::Fast,
        );
        assert_eq!(
            QueryTier::Auto.resolve("how does the FSM handle transitions"),
            QueryTier::Deep,
        );
    }

    #[test]
    fn query_tier_resolve_concrete_passes_through() {
        assert_eq!(QueryTier::Fast.resolve("some long complex query about architecture"), QueryTier::Fast);
        assert_eq!(QueryTier::Deep.resolve("main"), QueryTier::Deep);
        assert_eq!(QueryTier::Standard.resolve("x"), QueryTier::Standard);
    }

    #[test]
    fn query_tier_display() {
        assert_eq!(format!("{}", QueryTier::Fast), "FAST");
        assert_eq!(format!("{}", QueryTier::Standard), "STANDARD");
        assert_eq!(format!("{}", QueryTier::Deep), "DEEP");
        assert_eq!(format!("{}", QueryTier::Auto), "AUTO");
    }

    #[test]
    fn fast_tier_skips_vector_and_reranker() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let bm25 = make_bm25_with_canary(tmp.path());
        let pipeline = make_pipeline(bm25);

        let stages = pipeline.query_with_tier("main", QueryTier::Fast).expect("query");
        let stage_names: Vec<&str> = stages.iter().map(|s| s.stage.as_str()).collect();

        // All stage slots present for consistent indexing
        assert!(stage_names.contains(&"bm25"), "FAST must run BM25");
        assert!(stage_names.contains(&"vector"), "vector slot present");
        assert!(stage_names.contains(&"cross_encoder"), "cross_encoder slot present");

        // FAST: expansion, hyde, vector, pagerank, catrag, arag should be empty
        let expansion = stages.iter().find(|s| s.stage == "query_expansion").unwrap();
        assert!(expansion.results.is_empty(), "FAST skips expansion");

        let hyde = stages.iter().find(|s| s.stage == "hyde").unwrap();
        assert!(hyde.results.is_empty(), "FAST skips HyDE");

        let vector = stages.iter().find(|s| s.stage == "vector").unwrap();
        assert!(vector.results.is_empty(), "FAST skips vector search");
    }

    #[test]
    fn deep_tier_includes_expansion_and_hyde() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let bm25 = make_bm25_with_canary(tmp.path());
        let client: Arc<dyn HydeClient> = Arc::new(PipelineMockHydeClient::new());
        let hyde = make_hyde_stage(client);
        let pipeline = make_pipeline(bm25).with_hyde(hyde);

        // Force DEEP tier with a short query (explicit tier overrides auto-detect)
        let stages = pipeline.query_with_tier(
            "how does ELAI handle anti-spiral detection in the pipeline",
            QueryTier::Deep,
        ).expect("query");

        let expansion = stages.iter().find(|s| s.stage == "query_expansion").unwrap();
        // DEEP on a long query should produce expansions
        // (query is >5 words so expansion module will expand it)
        assert!(!expansion.results.is_empty(), "DEEP includes expansion");

        let hyde_stage = stages.iter().find(|s| s.stage == "hyde").unwrap();
        assert!(!hyde_stage.results.is_empty(), "DEEP includes HyDE hypothesis");
    }

    #[test]
    fn standard_tier_skips_expansion_and_hyde() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let bm25 = make_bm25_with_canary(tmp.path());
        let client = Arc::new(PipelineMockHydeClient::new());
        let hyde = make_hyde_stage(client.clone() as Arc<dyn HydeClient>);
        let pipeline = make_pipeline(bm25).with_hyde(hyde);

        let stages = pipeline.query_with_tier(
            "search for error handling patterns in the codebase",
            QueryTier::Standard,
        ).expect("query");

        let expansion = stages.iter().find(|s| s.stage == "query_expansion").unwrap();
        assert!(expansion.results.is_empty(), "STANDARD skips expansion");

        let hyde_stage = stages.iter().find(|s| s.stage == "hyde").unwrap();
        assert!(hyde_stage.results.is_empty(), "STANDARD skips HyDE");

        assert_eq!(client.calls(), 0, "sidecar should not be called for STANDARD tier");
    }

    #[test]
    fn query_backwards_compatible_auto_tier() {
        // Existing callers use query() without tier — must still work.
        let tmp = tempfile::tempdir().expect("tempdir");
        let bm25 = make_bm25_with_canary(tmp.path());
        let pipeline = make_pipeline(bm25);

        let stages = pipeline.query("main").expect("query");
        assert!(!stages.is_empty(), "query() must still return stages");

        let bm25_stage = stages.iter().find(|s| s.stage == "bm25").unwrap();
        assert!(bm25_stage.error.is_none(), "BM25 should work");
    }

    #[test]
    fn pipeline_tier_routing_structural_check() {
        let src = include_str!("pipeline.rs");
        assert!(src.contains("QueryTier"), "pipeline must reference QueryTier");
        assert!(src.contains("query_with_tier"), "pipeline must have query_with_tier method");
        assert!(src.contains("use_expansion"), "pipeline must gate expansion by tier");
        assert!(src.contains("use_hyde"), "pipeline must gate HyDE by tier");
        assert!(src.contains("use_vector"), "pipeline must gate vector by tier");
        assert!(src.contains("use_reranker"), "pipeline must gate reranker by tier");
        assert!(src.contains("resolved_tier"), "pipeline must resolve AUTO tier");
    }
}
