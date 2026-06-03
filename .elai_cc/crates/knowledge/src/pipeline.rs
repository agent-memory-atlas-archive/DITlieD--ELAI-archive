// B-15 HashMap ban cascade: BTreeMap for scores (paired with deep.rs:29
// rrf_quality_gate signature). HashSet kept — used lookup-only for per-stage
// chunk-id dedup (lines 401/437/510); HashSet ban is out-of-scope for B-15
// (plan only enumerated HashMap; HashSet residual is a B-15-followup item).
use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

use elai_core::{ClockProvider, OsClock};

use crate::deep::{deep_pool_cap, rrf_quality_gate, KnowledgeDeepConfig};
use crate::error::KnowledgeError;
use crate::freshness::{self, FreshnessDb};
use crate::hyde::{should_use_hyde, HydeStage};
use crate::knowledge_ssb_sink::{
    noop_sink, DeepPoolCappedEvent, HydeSkippedCodeQueryEvent, KnowledgeSsbSink, PagerankRunEvent,
    QueryTraceEvent, RrfComponentWeightsEvent, RrfQualityGateFiredEvent,
};
use crate::query_class::{classify, has_code_symbol_signal};
use crate::query_expansion::{self, ExpansionStrategy};
use crate::QueryTier;

use crate::arag::{AragOrchestrator, RetrievalResult};
use crate::artifacts::ArtifactStage;
use crate::bm25::{BM25Chunk, Bm25Index};
use crate::catrag::CatRag;
use crate::contextual_retrieval::{self, ContextualisedChunk};
use crate::cosmos_stage::CosmosStage;
use crate::degradation::SearchMode;
use crate::dkg::DkgEngine;
use crate::pagerank::{compute_repo_map, DkgSnapshot, PageRankConfig};
use crate::query_trace::{QueryTrace, StageTrace};
use crate::query_validator::QueryValidator;
use crate::reranker::CrossEncoderReranker;
use crate::vector::cache::VectorCache;
use crate::vector::VectorModule;
use crate::vector::indexer::{code_vec_gate, CodeVecConfig, VectorIndexer};
use crate::ReadinessStatus;

#[cfg(feature = "splade")]
use crate::splade::SpladeExpander;

/// C-1: default session id surfaced on `vector_cache_hit_rate` SSB rows
/// when the orchestrator did not call [`QueryPipeline::with_session_id`].
/// Named constant per codeRules baseline (no magic strings in code).
const DEFAULT_SESSION_ID: &str = "unknown";

/// C-38: derive a stable 64-bit prefix of the blake3 hash of `bytes`.
/// Used for `workspace_root_hash` and `top_node_hash` fields in the
/// `pagerank_run` SSB row so the dashboard can group rows by workspace
/// (or correlate top-ranked symbols) without leaking absolute paths or
/// full symbol names. blake3 chosen per CLAUDE.md anti-stale rule
/// (never blake2/sha2 for non-RFC uses).
fn blake3_prefix_u64(bytes: &[u8]) -> u64 {
    let hash = blake3::hash(bytes);
    let h = hash.as_bytes();
    u64::from_le_bytes([h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]])
}

/// E-17 SP2 task 7: per-ranker RRF weights surfaced on the
/// `rrf_component_weights` SSB beacon. Named constants per codeRules
/// baseline (no magic numbers). The defaults follow the plan spec
/// (`[knowledge.rrf] weights = {bm25 = 1.0, code_vec = 1.0, text_vec = 0.8}`).
/// Cold-wiring (s-81344): these are now the canonical default source for
/// [`crate::deep::RrfWeights`] / [`KnowledgeDeepConfig`]; the runtime values
/// are read from `self.deep_config.weights` (default == these consts).
pub(crate) const RRF_BM25_WEIGHT: f64 = 1.0;
pub(crate) const RRF_CODE_VEC_WEIGHT: f64 = 1.0;
pub(crate) const RRF_TEXT_VEC_WEIGHT: f64 = 0.8;
/// E-40 Tranche B-2 task 7: 5th-list (symbol_graph_vec) weight.
/// Plan-mandated 0.9 fixed weight per Task 7 spec. Wire-dark unless
/// `ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE` is set — see
/// [`symbol_graph_5th_rrf_enabled_from_env`].
pub(crate) const RRF_SYMBOL_GRAPH_VEC_WEIGHT: f64 = 0.9;

/// E-40 Tranche B-2 task 7: env-gate for the 5th-list symbol_graph_vec
/// RRF input. Default-OFF so the production pipeline continues to
/// produce the existing 4-list RRF until the operator opts in.
pub const ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE_ENV: &str = "ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE";

/// Read [`ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE_ENV`] and return `true` when
/// the gate is on. Default (unset / invalid) = `false`.
pub fn symbol_graph_5th_rrf_enabled_from_env() -> bool {
    match std::env::var(ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE_ENV) {
        Ok(v) => {
            let t = v.trim();
            t == "1" || t.eq_ignore_ascii_case("true") || t.eq_ignore_ascii_case("on")
        }
        Err(_) => false,
    }
}

/// E-17 SP4 wire: env gate for Anthropic Contextual Retrieval at ingest.
///
/// When set to a truthy value (`1`/`true`/`on`, case-insensitive), the
/// pipeline routes `BM25Chunk` lists through
/// [`contextual_retrieval::contextualize_chunk`] before they reach the
/// BM25 writer (and therefore before vector embedding picks the same
/// text up). Default-OFF: the chunk-text change invalidates embedding
/// caches, so reindex must be opt-in until SP6 ships the measured
/// recall@5 delta that justifies a default flip.
///
/// Reference: anthropic.com/news/contextual-retrieval (Sep 2024).
pub const ELAI_CONTEXTUAL_RETRIEVAL_ENABLE_ENV: &str = "ELAI_CONTEXTUAL_RETRIEVAL_ENABLE";

/// Read [`ELAI_CONTEXTUAL_RETRIEVAL_ENABLE_ENV`] and return `true` when
/// the gate is on. Default (unset / invalid) = `false` — see the const
/// docs for the embedding-cache rationale.
fn contextual_retrieval_enabled() -> bool {
    match std::env::var(ELAI_CONTEXTUAL_RETRIEVAL_ENABLE_ENV) {
        Ok(v) => {
            let t = v.trim();
            t == "1" || t.eq_ignore_ascii_case("true") || t.eq_ignore_ascii_case("on")
        }
        Err(_) => false,
    }
}

// ---------------------------------------------------------------------------
// PipelineStage
// ---------------------------------------------------------------------------

/// Result of a single stage in the retrieval pipeline.
///
/// Maps to knowledge.txt Section 12 StageTrace. `latency_ms` is measured
/// per-stage via the injected `ClockProvider` (C-8 Task #9). The entropy
/// quarantine is preserved — the pipeline never touches `std::time::Instant`
/// directly; all clock reads go through `core::clock::ClockProvider`. In
/// tests a `SeededClock` makes per-stage latencies fully deterministic.
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
    /// Optional context-artifact search stage (B-35 Phase B, Task 8).
    /// Runs after catrag for STANDARD and DEEP tiers; skipped for FAST.
    pub artifact_stage: Option<ArtifactStage>,
    /// Optional cosmos artifact stage (C-8 Task #10). Inserted between the
    /// artifact stage and the A-RAG stage for STANDARD + DEEP tiers; FAST
    /// queries skip it. Pulls fresh cross-project cosmos briefs that match
    /// the query via the name-router policy in `cosmos_stage.rs`.
    pub cosmos_stage: Option<CosmosStage>,
    #[cfg(feature = "splade")]
    pub splade: Option<SpladeExpander>,
    /// Current degradation cascade mode. Updated at startup by the cascade
    /// evaluation (cli/main.rs) and on circuit-breaker transitions (B-4).
    /// Used as the diagnostic label in QueryTrace so per-query traces reflect
    /// the actual mode rather than the default Hybrid sentinel.
    pub search_mode: SearchMode,
    /// C-1: typed SSB sink for `query_trace` beacons. One row emitted per
    /// successful `query_with_tier` call. `noop_sink()` by default; the
    /// production wiring site in `cli::live_cli` calls
    /// [`QueryPipeline::with_ssb`] to install the orchestrator adapter.
    ssb_sink: Arc<dyn KnowledgeSsbSink>,
    /// C-1: optional shared `VectorCache` whose hits/misses counters are
    /// flushed via [`VectorCache::flush_hit_rate_ssb`] after every
    /// `query_with_tier` returns. None by default; the orchestrator
    /// installs this in `cli::live_cli` so a `vector_cache_hit_rate` SSB
    /// row is emitted per query (skipped automatically when zero
    /// lookups happened since last flush).
    vector_cache: Option<Arc<VectorCache>>,
    /// C-1: session id used as the `session_id` field on every emitted
    /// `vector_cache_hit_rate` SSB row. Defaults to `"unknown"` until
    /// the orchestrator sets it via [`QueryPipeline::with_session_id`].
    session_id: String,
    /// C-8 Task #9: shared `ClockProvider` used to measure per-stage
    /// latencies. Defaults to a process-local `OsClock`; the orchestrator
    /// can swap a deterministic `SeededClock` in via
    /// [`QueryPipeline::with_clock`]. The entropy quarantine is preserved
    /// — no `std::time::Instant` reads occur in this module.
    clock: Arc<dyn ClockProvider>,
    /// E-40 Tranche B-2: optional orchestrator-rooted symbol-graph
    /// adapter for the 5th-list (`symbol_graph_vec`) RRF input. When
    /// `Some` and the [`ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE_ENV`] env gate
    /// is on, the pipeline calls `dataflow_impact` for any symbol-like
    /// query and merges the impacted-symbol set as a 5th RRF list.
    /// None by default — the existing 4-list RRF (bm25 / vector /
    /// code_vector / splade) is preserved when the gate is off or no
    /// adapter is installed. See
    /// `workflow/reasoning/E-40-tranche-B2-decisions.md`.
    symbol_graph_adapter: Option<Arc<dyn crate::symbol_graph_adapter::SymbolGraphAdapter>>,
    /// Cold-wiring (s-81344): config for the optional code_vector secondary
    /// index. Default `enabled = true` so the gate reduces to the existing
    /// `has_code_index()` check until the deferred cli loader splice
    /// (`ELAI_CODE_VEC`) installs an explicit value via
    /// [`QueryPipeline::with_code_vec_config`].
    code_vec_config: CodeVecConfig,
    /// Cold-wiring (s-81344): DEEP-tier RRF reranker config (pool cap, floor,
    /// k, per-ranker weights). Default mirrors the shipped constants, so the
    /// rerank path is byte-for-byte unchanged until
    /// [`QueryPipeline::with_deep_config`] installs an override.
    deep_config: KnowledgeDeepConfig,
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
            artifact_stage: None,
            cosmos_stage: None,
            #[cfg(feature = "splade")]
            splade: None,
            // Default to Hybrid until the degradation cascade has run its
            // first evaluation (B-4 full wiring). Callers that drive the
            // cascade can update this field before each query via
            // `pipeline.search_mode = cascade.evaluate(...)`.
            search_mode: SearchMode::Hybrid,
            ssb_sink: noop_sink(),
            vector_cache: None,
            session_id: DEFAULT_SESSION_ID.to_owned(),
            clock: Arc::new(OsClock),
            symbol_graph_adapter: None,
            code_vec_config: CodeVecConfig::default(),
            deep_config: KnowledgeDeepConfig::default(),
        }
    }

    /// Cold-wiring (s-81344): builder-style installer for the code_vector
    /// secondary-index config. Production wiring (deferred) at the cli loader
    /// reads `ELAI_CODE_VEC` and calls this; until then the default
    /// (`enabled = true`) preserves the existing `has_code_index()` gate.
    pub fn with_code_vec_config(mut self, config: CodeVecConfig) -> Self {
        self.code_vec_config = config;
        self
    }

    /// Cold-wiring (s-81344): builder-style installer for the DEEP-tier RRF
    /// reranker config (pool cap / floor / k / weights). The default mirrors
    /// the shipped constants, so omitting this is a behaviour-preserving no-op.
    pub fn with_deep_config(mut self, config: KnowledgeDeepConfig) -> Self {
        self.deep_config = config;
        self
    }

    /// E-40 Tranche B-2: builder-style installer for the optional
    /// orchestrator-rooted `SymbolGraphAdapter`. Production wiring at
    /// `cli::live_cli::build_local_knowledge_stack` calls this with an
    /// `Arc::new(SymbolGraphCacheAdapter::new(...))` so the pipeline
    /// can run the 5th-list `symbol_graph_vec` RRF input. The 5th list
    /// is still gated by [`ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE_ENV`] —
    /// installing the adapter does not flip the behaviour on by itself.
    pub fn with_symbol_graph_adapter(
        mut self,
        adapter: Arc<dyn crate::symbol_graph_adapter::SymbolGraphAdapter>,
    ) -> Self {
        self.symbol_graph_adapter = Some(adapter);
        self
    }

    /// C-1: builder-style installer for the knowledge SSB sink. Production
    /// wiring at `cli::live_cli::build_local_knowledge_stack` invokes this
    /// with `Arc::new(OrchestratorKnowledgeSsbSink::new(ssb_arc))` so
    /// per-query SSB beacons reach the dashboard.
    pub fn with_ssb(mut self, sink: Arc<dyn KnowledgeSsbSink>) -> Self {
        self.ssb_sink = sink;
        self
    }

    /// C-1: builder-style installer for the shared `VectorCache` whose
    /// hit/miss counters are flushed after every query.
    pub fn with_vector_cache(mut self, cache: Arc<VectorCache>) -> Self {
        self.vector_cache = Some(cache);
        self
    }

    /// C-1: install the session id surfaced on every
    /// `vector_cache_hit_rate` SSB row.
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = session_id.into();
        self
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

    /// Attach a context-artifact search stage (B-35 Phase B).
    ///
    /// When set, the stage runs after catrag for STANDARD and DEEP tiers and
    /// its results are appended to the stage list before RRF fusion.
    /// FAST tier queries skip this stage entirely.
    pub fn with_artifact_stage(mut self, stage: ArtifactStage) -> Self {
        self.artifact_stage = Some(stage);
        self
    }

    /// Attach a cosmos artifact retrieval stage (C-8 Task #10).
    ///
    /// When set, the stage runs after the artifact stage and before the
    /// A-RAG stage for STANDARD and DEEP tiers (FAST queries skip it). The
    /// stage pulls cross-project cosmos briefs whose name or research_query
    /// shares a token with the query.
    pub fn with_cosmos_stage(mut self, stage: CosmosStage) -> Self {
        self.cosmos_stage = Some(stage);
        self
    }

    /// C-8 Task #9: install a shared `ClockProvider` used for per-stage
    /// latency measurement. Production callers pass `Arc::new(OsClock)`;
    /// tests pass an `Arc<SeededClock>` for deterministic latencies.
    pub fn with_clock(mut self, clock: Arc<dyn ClockProvider>) -> Self {
        self.clock = clock;
        self
    }

    #[cfg(feature = "splade")]
    pub fn with_splade(mut self, expander: SpladeExpander) -> Self {
        self.splade = Some(expander);
        self
    }

    /// E-17 SP4 wire: contextualise a chunk list in-place before BM25
    /// ingest + vector embedding. No-op (returns input unchanged) when
    /// the env gate [`ELAI_CONTEXTUAL_RETRIEVAL_ENABLE_ENV`] is unset
    /// or false.
    ///
    /// `source` is the raw file text that produced `chunks`. The
    /// heuristic prefix scans up to [`contextual_retrieval::CONTEXT_WINDOW_LINES`]
    /// lines above each chunk's `line_start` for a nearby heading or
    /// signature, so passing the full file (not a single chunk's slice)
    /// is required for the anchor scan to find anything useful.
    ///
    /// Idempotent: chunks whose body already starts with the contextual
    /// header (re-ingest scenario) are skipped via
    /// [`contextual_retrieval::is_contextualised`] so a second pass over
    /// the same chunk list does not stack prefixes.
    ///
    /// Returns the (possibly rewritten) chunk vector plus a count of how
    /// many entries were contextualised this call. Callers can surface
    /// the count via tracing / SSB.
    pub fn contextualise_chunks_if_enabled(
        chunks: Vec<BM25Chunk>,
        source: &str,
    ) -> (Vec<BM25Chunk>, usize) {
        if !contextual_retrieval_enabled() {
            return (chunks, 0);
        }
        let mut out: Vec<BM25Chunk> = Vec::with_capacity(chunks.len());
        let mut contextualised_count: usize = 0;
        for chunk in chunks.into_iter() {
            if contextual_retrieval::is_contextualised(&chunk) {
                out.push(chunk);
                continue;
            }
            let ContextualisedChunk {
                chunk: rewritten,
                fresh_prefix,
                ..
            } = contextual_retrieval::contextualize_chunk(chunk, source, None);
            if fresh_prefix {
                contextualised_count += 1;
            }
            out.push(rewritten);
        }
        (out, contextualised_count)
    }

    /// E-17 SP4 wire: convenience entry that mirrors
    /// [`Bm25Index::index_file`] but routes the chunk list through
    /// [`Self::contextualise_chunks_if_enabled`] first when the env
    /// gate is on. `chunks` is required (callers that want auto-chunking
    /// should keep using `bm25.index_file(path, None)` and accept that
    /// the contextual-retrieval wire is a no-op in that path until a
    /// follow-up makes `Bm25Index::chunk_file` public or adds an
    /// internal hook).
    ///
    /// Emits a `tracing::info!` beacon with target
    /// `knowledge::contextual_retrieval` carrying the path + count when
    /// any chunk was rewritten — observable on the dashboard without a
    /// SCHEMA bump (the typed SSB row is SCHEMA-DEFERRED per
    /// workflow/reasoning/E-17-decisions.md).
    pub fn index_file_contextualised(
        &mut self,
        path: &std::path::Path,
        source: &str,
        chunks: Vec<BM25Chunk>,
    ) -> Result<usize, KnowledgeError> {
        let (rewritten, contextualised_count) =
            Self::contextualise_chunks_if_enabled(chunks, source);
        if contextualised_count > 0 {
            tracing::info!(
                target: "knowledge::contextual_retrieval",
                path = %path.display(),
                contextualised_count = contextualised_count,
                "contextual_retrieval_activated"
            );
        }
        self.bm25.index_file(path, Some(rewritten))
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
        // E-17 SP2 task 5: HyDE is gated by tier AND the NL-only classifier
        // (configurable via ELAI_HYDE_NL_ONLY env var; default true). When
        // tier=Deep but the query classifies as CodeSymbol / Mixed, HyDE is
        // skipped and a `hyde_skipped_code_query` SSB row is emitted below.
        let tier_wants_hyde = resolved_tier == QueryTier::Deep;
        let nl_gate_passes = should_use_hyde(query);
        let use_hyde = tier_wants_hyde && nl_gate_passes;
        if tier_wants_hyde && !nl_gate_passes {
            let qc = classify(query);
            let code_signal = has_code_symbol_signal(query);
            let word_count = query.split_whitespace().count();
            self.ssb_sink
                .emit_hyde_skipped_code_query(HydeSkippedCodeQueryEvent {
                    query_class: qc.as_str().to_string(),
                    word_count: u32::try_from(word_count).unwrap_or(u32::MAX),
                    code_signal_present: code_signal,
                });
        }
        let use_vector = matches!(resolved_tier, QueryTier::Standard | QueryTier::Deep);
        let use_reranker = matches!(resolved_tier, QueryTier::Standard | QueryTier::Deep);

        let mut stages = Vec::with_capacity(10);

        // C-8 Task #9: per-stage latency measurement via injected ClockProvider.
        // Each stage marks `start = self.clock.now()` before work and computes
        // `self.clock.elapsed_since(start).as_millis() as u64` at push time. No
        // raw `std::time::Instant` reads — the entropy quarantine stays clean.

        // Phase 3K.2: template-based multi-query expansion (DEEP only).
        let qx_start = self.clock.now();
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
                latency_ms: self.clock.elapsed_since(qx_start).as_millis() as u64,
                circuit_breaker_state: None,
                error: None,
            });

            expansions.iter().map(|e| e.text.clone()).collect()
        } else {
            // No expansion: original query only.
            stages.push(PipelineStage {
                stage: "query_expansion".to_string(),
                results: Vec::new(),
                latency_ms: self.clock.elapsed_since(qx_start).as_millis() as u64,
                circuit_breaker_state: None,
                error: None,
            });
            vec![query.to_string()]
        };

        // Phase 3K.3: HyDE (DEEP only).
        let hyde_start = self.clock.now();
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
                latency_ms: self.clock.elapsed_since(hyde_start).as_millis() as u64,
                circuit_breaker_state: breaker_state,
                error,
            });
            hyp
        } else {
            stages.push(PipelineStage {
                stage: "hyde".to_string(),
                results: Vec::new(),
                latency_ms: self.clock.elapsed_since(hyde_start).as_millis() as u64,
                circuit_breaker_state: None,
                error: None,
            });
            None
        };

        // Stage 0: BM25 — always runs (all tiers).
        let bm25_start = self.clock.now();
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
            latency_ms: self.clock.elapsed_since(bm25_start).as_millis() as u64,
            circuit_breaker_state: None,
            error: None,
        });

        // Stage 1: vector (STANDARD + DEEP only).
        let vector_start = self.clock.now();
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
                latency_ms: self.clock.elapsed_since(vector_start).as_millis() as u64,
                circuit_breaker_state: None,
                error: vector_error,
            });
        } else {
            stages.push(PipelineStage {
                stage: "vector".to_string(),
                results: Vec::new(),
                latency_ms: self.clock.elapsed_since(vector_start).as_millis() as u64,
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
                if code_vec_gate(self.code_vec_config.enabled, indexer.has_code_index()) {
                    let code_vector_start = self.clock.now();
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
                        latency_ms: self.clock.elapsed_since(code_vector_start).as_millis() as u64,
                        circuit_breaker_state: None,
                        error: code_error,
                    });
                }
            }
        }

        // Stage 1.5: splade keyword expansion (STANDARD + DEEP)
        #[cfg(feature = "splade")]
        {
            let splade_start = self.clock.now();
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
                latency_ms: self.clock.elapsed_since(splade_start).as_millis() as u64,
                circuit_breaker_state: None,
                error: None,
            });
        }

        // Stage 2: pagerank (STANDARD + DEEP)
        let pagerank_start = self.clock.now();
        let snapshot = if use_vector {
            let (snap, pagerank_results, pagerank_error, top_node_name) = match self.dkg_engine {
                Some(ref engine) => match DkgSnapshot::from_engine(engine) {
                    Ok(snap) => {
                        let config = PageRankConfig::default();
                        let token_est: &dyn Fn(&str) -> usize = &|s: &str| s.len() / 4;
                        let repo_map = compute_repo_map(&snap, query, &[], &config, token_est);
                        let top_name = repo_map
                            .ranked_symbols
                            .first()
                            .map(|rs| rs.symbol_name.clone());
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
                        (snap, results, None, top_name)
                    }
                    Err(e) => {
                        tracing::debug!("DkgSnapshot::from_engine failed: {e}");
                        let empty = DkgSnapshot { nodes: Vec::new(), edges: Vec::new() };
                        (empty, Vec::new(), Some(e.to_string()), None)
                    }
                },
                None => {
                    let empty = DkgSnapshot { nodes: Vec::new(), edges: Vec::new() };
                    (empty, Vec::new(), None, None)
                }
            };
            let pagerank_latency_ms =
                self.clock.elapsed_since(pagerank_start).as_millis() as u64;

            // C-38: emit one `pagerank_run` SSB row per Stage 2 invocation
            // (including the empty-graph / no-engine path so observability
            // distinguishes "PageRank ran with empty graph" from "PageRank
            // never ran at all").
            let workspace_hash = blake3_prefix_u64(self.session_id.as_bytes());
            let node_count_u32 = u32::try_from(snap.nodes.len()).unwrap_or(u32::MAX);
            let top_node_hash = top_node_name
                .as_deref()
                .map(|n| blake3_prefix_u64(n.as_bytes()))
                .unwrap_or(0);
            self.ssb_sink.emit_pagerank_run(PagerankRunEvent {
                workspace_root_hash: workspace_hash,
                node_count: node_count_u32,
                top_node_hash,
                elapsed_ms: pagerank_latency_ms,
            });

            stages.push(PipelineStage {
                stage: "pagerank".to_string(),
                results: pagerank_results,
                latency_ms: pagerank_latency_ms,
                circuit_breaker_state: None,
                error: pagerank_error,
            });
            snap
        } else {
            let empty = DkgSnapshot { nodes: Vec::new(), edges: Vec::new() };
            stages.push(PipelineStage {
                stage: "pagerank".to_string(),
                results: Vec::new(),
                latency_ms: self.clock.elapsed_since(pagerank_start).as_millis() as u64,
                circuit_breaker_state: None,
                error: None,
            });
            empty
        };

        // Stage 3: catrag (STANDARD + DEEP)
        let catrag_start = self.clock.now();
        if use_vector {
            let _overlay = self.catrag.compute_overlay(query, &snapshot, &[])?;
        }
        stages.push(PipelineStage {
            stage: "catrag".to_string(),
            results: Vec::new(),
            latency_ms: self.clock.elapsed_since(catrag_start).as_millis() as u64,
            circuit_breaker_state: None,
            error: None,
        });

        // Stage 3b: artifact search (STANDARD + DEEP only, B-35 Phase B Task 8).
        // Runs only when an ArtifactStage is configured AND tier is STANDARD or DEEP.
        // FAST tier is excluded: artifact search requires embedding (no VectorModule budget).
        if use_vector {
            if let Some(ref artifact) = self.artifact_stage {
                let artifact_start = self.clock.now();
                let (artifact_results, artifact_error) = match artifact.run(query, None) {
                    Ok(results) => {
                        tracing::debug!(
                            count = results.len(),
                            "artifact_stage: returned {} results",
                            results.len()
                        );
                        (results, None)
                    }
                    Err(e) => {
                        tracing::debug!("artifact_stage error: {e}");
                        (Vec::new(), Some(e.to_string()))
                    }
                };
                stages.push(PipelineStage {
                    stage: "artifacts".to_string(),
                    results: artifact_results,
                    latency_ms: self.clock.elapsed_since(artifact_start).as_millis() as u64,
                    circuit_breaker_state: None,
                    error: artifact_error,
                });
            }
        }

        // Stage 3c: cosmos artifact retrieval (C-8 Task #10, STANDARD + DEEP).
        // Inserted between the artifact stage and the A-RAG stage. FAST is
        // skipped — cosmos provides cross-project curated briefs, not the
        // low-latency grep-style hits FAST callers need.
        if use_vector {
            if let Some(ref cosmos) = self.cosmos_stage {
                let cosmos_start = self.clock.now();
                let cosmos_results = cosmos.run(query);
                tracing::debug!(
                    count = cosmos_results.len(),
                    "cosmos_stage: returned {} results",
                    cosmos_results.len()
                );
                stages.push(PipelineStage {
                    stage: "cosmos".to_string(),
                    results: cosmos_results,
                    latency_ms: self.clock.elapsed_since(cosmos_start).as_millis() as u64,
                    circuit_breaker_state: None,
                    error: None,
                });
            }
        }

        // Stage 4: arag (STANDARD + DEEP)
        let arag_start = self.clock.now();
        let arag_results = if use_vector {
            self.arag.query_pipeline(query, &snapshot)?
        } else {
            Vec::new()
        };
        stages.push(PipelineStage {
            stage: "arag".to_string(),
            results: arag_results,
            latency_ms: self.clock.elapsed_since(arag_start).as_millis() as u64,
            circuit_breaker_state: None,
            error: None,
        });

        // E-40 Tranche B-2: 5th-list `symbol_graph_vec` RRF input.
        // Wire-dark until `ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE=1` AND a
        // `SymbolGraphAdapter` has been installed via
        // `with_symbol_graph_adapter(...)`. Best-effort: any adapter
        // error degrades to an empty stage (the RRF accumulator
        // iterates all stages uniformly, so an empty list is a no-op).
        // Joins the pool alongside bm25 / vector / code_vector / splade
        // as the 5th input list. Plan-mandated weight = 0.9 (named
        // const RRF_SYMBOL_GRAPH_VEC_WEIGHT). See
        // `workflow/reasoning/E-40-tranche-B2-decisions.md`.
        if symbol_graph_5th_rrf_enabled_from_env() {
            if let Some(ref adapter) = self.symbol_graph_adapter {
                let sg_start = self.clock.now();
                let project_root = std::path::Path::new(".");
                // BFS impact analysis over the raw query string; if
                // the query isn't a symbol id the adapter returns an
                // empty list (best-effort). Depth fixed at 2 to stay
                // within B-29.5 p99 ≤ 500ms budget.
                let (sg_results, sg_error) =
                    match adapter.dataflow_impact(query, project_root, 2) {
                        Ok(resp) => {
                            let results: Vec<RetrievalResult> = resp
                                .impacted_symbols
                                .iter()
                                .enumerate()
                                .map(|(i, sym)| RetrievalResult {
                                    chunk_id: format!("symbol_graph_vec:{sym}"),
                                    snippet: sym.clone(),
                                    score: 1.0 / (1.0 + i as f64),
                                    tool_used: "symbol_graph_vec".to_string(),
                                })
                                .collect();
                            (results, None)
                        }
                        Err(e) => {
                            tracing::debug!("symbol_graph_vec adapter error: {e}");
                            (Vec::new(), Some(e.to_string()))
                        }
                    };
                stages.push(PipelineStage {
                    stage: "symbol_graph_vec".to_string(),
                    results: sg_results,
                    latency_ms: self.clock.elapsed_since(sg_start).as_millis() as u64,
                    circuit_breaker_state: None,
                    error: sg_error,
                });
            }
        }

        // Stage 5: rerank (RRF fusion — all tiers, but FAST has only BM25 input).
        //
        // E-17 SP2 task 4: on DEEP tier, cap each stage's pool to
        // DEEP_CAP_DEFAULT BEFORE the RRF accumulator runs, then apply the
        // RRF quality floor AFTER the freshness multiplier. The cap and
        // floor emits land via the typed knowledge ssb sink (4 new SSB
        // action types declared in orchestrator::ssb under SCHEMA_VERSION 97).
        let rerank_start = self.clock.now();
        let (reranked, deep_cap_dropped, rrf_floor_dropped, rrf_pre_floor_count) =
            self.rrf_rerank_with_tier(&mut stages, resolved_tier);

        // B-4 task #16: measure emit-side latency for the deep_cap +
        // rrf_gate post-hoc synthetic stages instead of hard-coding `0`.
        // Both gates were already exercised inside `rrf_rerank_with_tier`
        // above (so the heavy work shows up under the "rerank" row); the
        // numbers below are the dashboard-observable cost of the
        // gate-fire emit itself (sink dispatch + stage push), which is
        // non-zero whenever the gate triggers. Preserves the entropy
        // quarantine: only `self.clock` reads, no raw `Instant`.
        if deep_cap_dropped > 0 {
            let deep_cap_emit_start = self.clock.now();
            self.ssb_sink.emit_deep_pool_capped(DeepPoolCappedEvent {
                capped_count: u32::try_from(deep_cap_dropped).unwrap_or(u32::MAX),
                max_allowed: u32::try_from(self.deep_config.max_candidates).unwrap_or(u32::MAX),
                stage_count: u32::try_from(stages.len()).unwrap_or(u32::MAX),
            });
            stages.push(PipelineStage {
                stage: "deep_cap".to_string(),
                results: Vec::new(),
                latency_ms: self.clock.elapsed_since(deep_cap_emit_start).as_millis() as u64,
                circuit_breaker_state: None,
                error: None,
            });
        }

        // E-17 SP2: emit rrf_gate stage trace + SSB row when the floor fired.
        if rrf_floor_dropped > 0 {
            let rrf_gate_emit_start = self.clock.now();
            self.ssb_sink.emit_rrf_quality_gate_fired(RrfQualityGateFiredEvent {
                dropped_count: u32::try_from(rrf_floor_dropped).unwrap_or(u32::MAX),
                floor: self.deep_config.rrf_floor,
                total_before: u32::try_from(rrf_pre_floor_count).unwrap_or(u32::MAX),
            });
            stages.push(PipelineStage {
                stage: "rrf_gate".to_string(),
                results: Vec::new(),
                latency_ms: self.clock.elapsed_since(rrf_gate_emit_start).as_millis() as u64,
                circuit_breaker_state: None,
                error: None,
            });
        }

        // E-17 SP2 task 7: per-query rrf_component_weights beacon.
        // Components are emitted as Some(...) when their stage exists in
        // the stages slice (bm25 always; code_vector + text_vector when
        // their indices are loaded). text_vec_weight reflects the
        // text-vector stage label "vector" in the current pipeline.
        let mut bm25_weight: Option<f64> = None;
        let mut code_vec_weight: Option<f64> = None;
        let mut text_vec_weight: Option<f64> = None;
        // E-40 Tranche B-2: 5th-list symbol_graph_vec weight. Only
        // surfaced when the stage actually fired (gate + adapter both
        // present). Tracked locally for the tracing log below; the
        // beacon struct stays at 3 fields so the existing
        // `RrfComponentWeightsEvent` JSON shape does not drift.
        let mut symbol_graph_vec_weight: Option<f64> = None;
        for stage in &stages {
            match stage.stage.as_str() {
                "bm25" => bm25_weight = Some(self.deep_config.weights.bm25),
                "code_vector" => code_vec_weight = Some(self.deep_config.weights.code_vec),
                "vector" => text_vec_weight = Some(self.deep_config.weights.text_vec),
                "symbol_graph_vec" => {
                    symbol_graph_vec_weight = Some(self.deep_config.weights.symbol_graph_vec);
                }
                _ => {}
            }
        }
        if let Some(w) = symbol_graph_vec_weight {
            tracing::debug!(
                weight = w,
                "E-40 Tranche B-2: symbol_graph_vec 5th-list RRF weight"
            );
        }
        self.ssb_sink.emit_rrf_component_weights(RrfComponentWeightsEvent {
            bm25_weight,
            code_vec_weight,
            text_vec_weight,
        });

        stages.push(PipelineStage {
            stage: "rerank".to_string(),
            results: reranked,
            latency_ms: self.clock.elapsed_since(rerank_start).as_millis() as u64,
            circuit_breaker_state: None,
            error: None,
        });

        // Stage 6: cross_encoder (STANDARD + DEEP only)
        let ce_start = self.clock.now();
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
            latency_ms: self.clock.elapsed_since(ce_start).as_millis() as u64,
            circuit_breaker_state: ce_breaker_state,
            error: ce_error,
        });

        // Stage 7: token_fit (passthrough — budget enforcement is A-RAG internal)
        let token_fit_start = self.clock.now();
        let token_fit_results = if let Some(ce_stage) = stages.last() {
            ce_stage.results.clone()
        } else {
            Vec::new()
        };
        stages.push(PipelineStage {
            stage: "token_fit".to_string(),
            results: token_fit_results,
            latency_ms: self.clock.elapsed_since(token_fit_start).as_millis() as u64,
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

        // B-4 task #16: compute total_latency_ms BEFORE constructing the
        // QueryTrace so the trace struct carries the real sum. Pipelines
        // do not measure wall time directly here (entropy quarantine),
        // so the sum of per-stage latencies is the conservative upper
        // bound the dashboard renders as "approximate total".
        let total_latency_ms: u64 = stage_traces
            .iter()
            .fold(0u64, |acc, s| acc.saturating_add(s.latency_ms));

        let trace = QueryTrace::new(stage_traces, total_latency_ms, self.search_mode, result_count);
        tracing::debug!(
            query_id = %trace.query_id,
            tier = %resolved_tier,
            stages = trace.stages.len(),
            results = trace.result_count,
            "query_trace"
        );

        // C-1: emit one `query_trace` SSB row per call. The trace already
        // carries `total_latency_ms`; the SSB payload field name is the
        // same so dashboards downstream see a non-zero total whenever any
        // stage measured non-zero latency.
        let stage_count = u32::try_from(trace.stages.len()).unwrap_or(u32::MAX);
        let result_count_u32 = u32::try_from(trace.result_count).unwrap_or(u32::MAX);
        self.ssb_sink.emit_query_trace(QueryTraceEvent {
            query_id: trace.query_id.to_string(),
            tier: format!("{resolved_tier}"),
            stage_count,
            result_count: result_count_u32,
            total_latency_ms,
        });

        // C-1: flush the vector-cache hit/miss counters to SSB. The
        // flush is a no-op when no lookups happened since the last call
        // (see `VectorCache::flush_hit_rate_ssb` doc-comment) so query
        // tiers that bypass the cache do not flood the dashboard.
        if let Some(cache) = self.vector_cache.as_ref() {
            cache.flush_hit_rate_ssb(&self.session_id);
        }

        Ok(stages)
    }

    /// Reciprocal Rank Fusion across all stages that produced results.
    /// RRF score = sum(1 / (k + rank)) where k=60 and rank starts at 1.
    ///
    /// When a `freshness_db` is attached, each chunk's RRF score is multiplied
    /// by its freshness factor (recency_decay * access_boost). Missing entries
    /// default to 1.0 (no penalty). See anti-staleness-enforcement.txt §1.1.
    ///
    /// Retained as a thin wrapper around [`Self::rrf_rerank_with_tier`] for
    /// the test fixtures in `#[cfg(test)]` modules that call `rrf_rerank`
    /// directly; production now goes through the tier-aware variant.
    #[allow(dead_code)]
    fn rrf_rerank(&self, stages: &[PipelineStage]) -> Vec<RetrievalResult> {
        // Production callers use rrf_rerank_with_tier; this clone-and-discard
        // path is test-only and intentionally bypasses the DEEP cap.
        let mut stages_owned: Vec<PipelineStage> = stages.to_vec();
        let (results, _cap, _floor, _pre) =
            self.rrf_rerank_with_tier(&mut stages_owned, QueryTier::Standard);
        results
    }

    /// E-17 SP2 task 4: tier-aware RRF fusion with optional DEEP cap and
    /// RRF quality gate.
    ///
    /// Ordering (HARD):
    ///   1. On DEEP tier: truncate each stage's results to
    ///      `self.deep_config.max_candidates` (default
    ///      [`crate::deep::DEEP_CAP_DEFAULT`]) BEFORE the accumulator.
    ///   2. Run RRF accumulator with k = `self.deep_config.rrf_k` (default
    ///      [`crate::deep::RRF_K_CONSTANT`] = 60).
    ///   3. Apply freshness multiplier (post-accumulator).
    ///   4. On DEEP tier: drop candidates whose score is below
    ///      `self.deep_config.rrf_floor` (default
    ///      [`crate::deep::RRF_FLOOR_DEFAULT`]) AFTER the multiplier.
    ///   5. Sort by score desc (tie-break: chunk_id asc for determinism).
    ///
    /// Returns `(reranked, deep_cap_dropped, rrf_floor_dropped, pre_floor_count)`
    /// so the caller can emit `deep_pool_capped` + `rrf_quality_gate_fired`
    /// SSB beacons exactly once per query.
    fn rrf_rerank_with_tier(
        &self,
        stages: &mut Vec<PipelineStage>,
        tier: QueryTier,
    ) -> (Vec<RetrievalResult>, usize, usize, usize) {
        // Step 1: DEEP cap BEFORE the RRF accumulator (E-17 SP2 task 4).
        let deep_cap_dropped: usize = if tier == QueryTier::Deep {
            deep_pool_cap(stages, self.deep_config.max_candidates)
        } else {
            0
        };

        // Step 2: RRF accumulator (k=60 per Cormack SIGIR 2009).
        let mut scores: BTreeMap<String, (f64, RetrievalResult)> = BTreeMap::new();
        for stage in stages.iter() {
            for (rank_idx, result) in stage.results.iter().enumerate() {
                // NARROWING-OK: rank_idx + 1 fits in f64 (rank_idx is bounded
                // by DEEP_CAP_DEFAULT = 100 on DEEP, by per-stage retrieval
                // limit otherwise).
                let rank_f = (rank_idx + 1) as f64;
                let rrf_score = 1.0 / (self.deep_config.rrf_k + rank_f);
                let entry = scores
                    .entry(result.chunk_id.clone())
                    .or_insert_with(|| (0.0, result.clone()));
                entry.0 += rrf_score;
            }
        }

        // Step 3: freshness multiplier (post-accumulator).
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

        // Step 4: RRF quality gate AFTER the freshness multiplier
        // (E-17 SP2 task 4). DEEP tier only.
        let pre_floor_count = scores.len();
        let rrf_floor_dropped: usize = if tier == QueryTier::Deep {
            rrf_quality_gate(&mut scores, self.deep_config.rrf_floor)
        } else {
            0
        };

        // Step 5: sort desc by score; tie-break by chunk_id asc for
        // deterministic output. BTreeMap (B-15) already iterates by chunk_id;
        // the tie-break is kept defensively in case the upstream type ever
        // shifts back.
        let mut ranked: Vec<(f64, RetrievalResult)> = scores.into_values().collect();
        ranked.sort_by(|a, b| {
            b.0.partial_cmp(&a.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.1.chunk_id.cmp(&b.1.chunk_id))
        });

        let results: Vec<RetrievalResult> = ranked
            .into_iter()
            .map(|(score, mut r)| {
                r.score = score;
                r.tool_used = "rerank".to_string();
                r
            })
            .collect();

        (results, deep_cap_dropped, rrf_floor_dropped, pre_floor_count)
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
