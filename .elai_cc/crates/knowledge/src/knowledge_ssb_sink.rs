//! C-1: typed knowledge-side SSB sink trait.
//!
//! Sibling to [`crate::degradation::SsbNotifier`]: that trait carries the
//! single `DegradationEvent` shape used by `DegradationCascade::evaluate`
//! and is kept untouched so the B-4 contract surface does not regress.
//! This trait carries the typed knowledge-pipeline events introduced by
//! C-1 (HyDE per-query, VectorCache flush, QueryPipeline per-query trace).
//!
//! Decision (resolves the HIGH ambiguity flagged in
//! `workflow/sessions/s-32952/context-package-C1.md`): introduce a sibling
//! trait rather than widening `DegradationEvent`. Rationale:
//!  1. The B-4 `SsbNotifier` lives in the `knowledge` crate today and
//!     would have to be widened into a 4-variant enum carrying mixed
//!     shapes, polluting the existing degradation surface.
//!  2. Reranker already uses `Arc<dyn SsbNotifier>` with a noop default —
//!     keeping that contract stable means no caller of B-4 changes.
//!  3. The new sink is method-per-event (Hyde / VectorCacheHitRate /
//!     QueryTrace) which matches the SSB payload shape directly — no
//!     intermediate enum to keep in lockstep with [`SsbPayload`].
//!
//! Production adapter: the orchestrator crate constructs an
//! [`Arc<dyn KnowledgeSsbSink>`] wrapping `Arc<Ssb>` (one trivial impl
//! per orchestrator crate; the knowledge crate stays SSB-store-agnostic).
//! Default (test / inert path): [`NoopKnowledgeSsbSink`].

use std::sync::Arc;

/// Per-call HyDE event emitted by [`crate::hyde::HydeStage::hypothesize`].
///
/// `query_hash` is the blake3 hex of the query bytes (16 chars / 64-bit
/// prefix is enough to distinguish queries in dashboard UIs without
/// leaking content). `hypothesis_len` is `0` when no hypothesis was
/// produced (short query / breaker open / sidecar error).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HydeQueryEvent {
    pub query_hash: String,
    pub hypothesis_len: u32,
    pub cache_hit: bool,
    pub latency_ms: u64,
}

/// Aggregate vector-cache hit/miss snapshot flushed by
/// [`crate::vector::cache::VectorCache::flush_hit_rate_ssb`].
///
/// `hit_rate_pct` ranges `0.0..=100.0`. When `hits + misses == 0`
/// (no lookups since last flush) the sink is intentionally NOT called
/// so empty rows do not flood the dashboard.
#[derive(Debug, Clone, PartialEq)]
pub struct VectorCacheHitRateEvent {
    pub session_id: String,
    pub hits: u32,
    pub misses: u32,
    pub hit_rate_pct: f64,
}

/// Per-query QueryPipeline trace emitted at the end of
/// [`crate::pipeline::QueryPipeline::query_with_tier`].
#[derive(Debug, Clone, PartialEq)]
pub struct QueryTraceEvent {
    pub query_id: String,
    pub tier: String,
    pub stage_count: u32,
    pub result_count: u32,
    pub total_latency_ms: u64,
}

/// E-17 SP2 task 4: emitted once per DEEP-tier query when the
/// [`crate::deep::deep_pool_cap`] gate trims one or more input stages.
/// `capped_count` is the total number of candidates dropped across all
/// stages; `max_allowed` is the per-stage cap that fired.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeepPoolCappedEvent {
    pub capped_count: u32,
    pub max_allowed: u32,
    pub stage_count: u32,
}

/// E-17 SP2 task 4: emitted once per DEEP-tier query when the
/// [`crate::deep::rrf_quality_gate`] drops one or more fused candidates.
#[derive(Debug, Clone, PartialEq)]
pub struct RrfQualityGateFiredEvent {
    pub dropped_count: u32,
    pub floor: f64,
    pub total_before: u32,
}

/// E-17 SP2 task 5: emitted when [`crate::hyde::should_use_hyde`] returns
/// false on a CodeSymbol or Mixed query that would otherwise have
/// triggered HyDE (DEEP tier + >= MIN_WORDS_FOR_HYDE).
#[derive(Debug, Clone, PartialEq)]
pub struct HydeSkippedCodeQueryEvent {
    pub query_class: String,
    pub word_count: u32,
    pub code_signal_present: bool,
}

/// E-17 SP2 task 7: emitted once per query (when the RRF stage runs)
/// carrying the per-ranker RRF weights that were active. Fields are
/// `None` when the corresponding ranker did not contribute (e.g.
/// `code_vec_weight` is None when `ELAI_CODE_VEC` is disabled).
#[derive(Debug, Clone, PartialEq)]
pub struct RrfComponentWeightsEvent {
    pub bm25_weight: Option<f64>,
    pub code_vec_weight: Option<f64>,
    pub text_vec_weight: Option<f64>,
}

/// C-38: per-invocation `compute_repo_map` SSB beacon. Emitted from
/// `QueryPipeline::query_with_tier` Stage 2 after `compute_repo_map`
/// returns (whether or not it produced ranked symbols). Distinct from
/// `query_trace` which aggregates all stages — this is a Stage-2-only
/// row that lets the dashboard answer "did PageRank actually fire for
/// this repo, and what did the top-ranked node look like ?".
///
/// `workspace_root_hash` is a stable 64-bit blake3-prefix hash of the
/// project root path so the dashboard can group rows by workspace
/// without leaking absolute paths into SSB. `node_count` is the size
/// of the DKG snapshot fed to PageRank (zero on the absent-engine
/// path; the row is still emitted so "PageRank ran with empty graph"
/// is observable). `top_node_hash` is a 64-bit blake3-prefix hash of
/// the top-ranked symbol name (or 0 when `node_count == 0`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PagerankRunEvent {
    pub workspace_root_hash: u64,
    pub node_count: u32,
    pub top_node_hash: u64,
    pub elapsed_ms: u64,
}

/// C-38: emitted when the CLI's `ensure_dkg_or_rebuild` helper begins
/// a rebuild because the canonical DKG database (`.elai_cc/indexes/dkg.db`)
/// is absent or stale. The matching `DkgRebuildCompleteEvent` row brackets
/// the rebuild's end.
///
/// `trigger_reason` is a snake-case label (`"missing"`, `"stale"`)
/// carried so the dashboard can render WHY a rebuild fired. The
/// staleness window is the caller's choice (default 24h, configurable
/// per call); this event records the verdict, not the threshold.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DkgRebuildStartedEvent {
    pub workspace_root_hash: u64,
    pub trigger_reason: String,
}

/// C-38: emitted when `ensure_dkg_or_rebuild` finishes a rebuild
/// (success or failure). `success=false` rows MUST still be emitted so
/// dashboards see the failure and operators can drill into the
/// tracing log for the underlying error. `node_count` is the number
/// of DKG nodes produced (0 on failure or empty repo). `duration_ms`
/// measures the wall-clock time from start to end of the rebuild.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DkgRebuildCompleteEvent {
    pub workspace_root_hash: u64,
    pub success: bool,
    pub duration_ms: u64,
    pub node_count: u32,
}

// ---------------------------------------------------------------------------
// E-42: tree-sitter grammar health events
// ---------------------------------------------------------------------------

/// E-42: discriminator for [`TreeSitterGrammarEvent`]. Maps 1:1 onto
/// `TreeSitterGrammarEventKind` in `elai_orchestrator::ssb` (kept here so the
/// knowledge crate stays SSB-payload-agnostic while the adapter in the
/// orchestrator crate does the translation).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeSitterGrammarEventKind {
    /// A grammar's reported ABI version is outside the accepted range
    /// (`[14, 15]` per the May 2026 maturity table). `language_id`
    /// carries the offending language; `detail` carries the raw version
    /// string (e.g. `"version=13"`).
    AbiProbeMismatch,
    /// Svelte runes detected + ERROR/MISSING node ratio after parse.
    /// `detail` carries the ratio as a decimal string (e.g. `"0.12"`).
    SvelteRunesParseHealth,
    /// A Tier-B grammar was successfully registered. `language_id` is
    /// the language id; `detail` is `None`.
    TierBGrammarLoaded,
    /// A source file was skipped because its byte count exceeds the
    /// 2 MiB ceiling guard. `detail` carries the byte count as a string.
    TreeSitterSkippedLargeFile,
}

/// E-42: typed event emitted by `DkgEngine::build_graph` when
/// `ELAI_TREE_SITTER_GRAMMARS_ENABLE=1`. One row per mismatch, per
/// large-file skip, or per Svelte runes-health classification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeSitterGrammarEvent {
    /// Which sub-event fired (maps to the SSB 4-arm discriminator).
    pub kind: TreeSitterGrammarEventKind,
    /// Grammar language id (e.g. `"kotlin"`). `None` for large-file
    /// events where the association is with a file, not a grammar.
    pub language_id: Option<String>,
    /// Free-form details string (e.g. error ratio, byte count, ABI
    /// version). `None` when not applicable.
    pub detail: Option<String>,
}

/// J-73 Slice C: retrieval abstained event emitted by the knowledge
/// retrieval path when a retrieval score is evaluated against a threshold
/// and the cascade either skips (score cleared T) or proceeds (score below
/// T). `kind_str` is a snake-case label (`"would_skip"`, `"would_proceed"`,
/// or other for override) that the orchestrator adapter maps onto the SSB
/// `RetrievalAbstainedKind` discriminator.
#[derive(Debug, Clone, PartialEq)]
pub struct RetrievalAbstainedEvent {
    /// Snake-case kind label mapped by the adapter to RetrievalAbstainedKind.
    pub kind_str: String,
    /// Retrieval score that was evaluated against the threshold.
    pub score: f64,
    /// Threshold T the score was compared against.
    pub threshold: f64,
    /// Task type label for the retrieval attempt.
    pub task_type: String,
    /// Whether observe-only mode was active.
    pub observe_only: bool,
    /// Blake3 hex prefix of the query bytes.
    pub query_hash: String,
}

/// C-1 knowledge-side SSB sink.
///
/// Implementations forward typed events to the orchestrator's
/// `Arc<Ssb>` writer. The knowledge crate never depends on the
/// orchestrator's `SsbPayload` enum directly; the adapter on the
/// orchestrator side does the translation. This keeps the knowledge
/// -> orchestrator dependency direction acyclic.
///
/// Every method MUST be infallible from the caller's perspective:
/// SSB write failures are logged via `tracing` but NEVER bubbled up,
/// matching the existing degradation/SsbNotifier contract.
pub trait KnowledgeSsbSink: Send + Sync {
    fn emit_hyde_query(&self, event: HydeQueryEvent);
    fn emit_vector_cache_hit_rate(&self, event: VectorCacheHitRateEvent);
    fn emit_query_trace(&self, event: QueryTraceEvent);
    /// E-17 SP2 task 4: emitted once per DEEP-tier query when the deep
    /// cap drops one or more input candidates.
    fn emit_deep_pool_capped(&self, event: DeepPoolCappedEvent);
    /// E-17 SP2 task 4: emitted once per DEEP-tier query when the RRF
    /// quality gate drops one or more fused candidates.
    fn emit_rrf_quality_gate_fired(&self, event: RrfQualityGateFiredEvent);
    /// E-17 SP2 task 5: emitted when HyDE is skipped because the query
    /// classified as CodeSymbol or Mixed.
    fn emit_hyde_skipped_code_query(&self, event: HydeSkippedCodeQueryEvent);
    /// E-17 SP2 task 7: per-query RRF component weights.
    fn emit_rrf_component_weights(&self, event: RrfComponentWeightsEvent);
    /// C-38: emitted once per Stage 2 `compute_repo_map` invocation
    /// in [`crate::pipeline::QueryPipeline::query_with_tier`]. Carries
    /// the workspace hash + node count + top-node hash + elapsed time.
    fn emit_pagerank_run(&self, event: PagerankRunEvent);
    /// C-38: emitted at the start of a CLI-driven DKG rebuild
    /// (production wiring lives in the cli crate's `dkg_lifecycle`
    /// module).
    fn emit_dkg_rebuild_started(&self, event: DkgRebuildStartedEvent);
    /// C-38: emitted at the end of a CLI-driven DKG rebuild
    /// (success OR failure). Bracketed with
    /// [`Self::emit_dkg_rebuild_started`].
    fn emit_dkg_rebuild_complete(&self, event: DkgRebuildCompleteEvent);
    /// E-42: emitted by `DkgEngine::build_graph` for ABI probe mismatches,
    /// large-file skips, and Svelte runes health rows. Gated on
    /// `ELAI_TREE_SITTER_GRAMMARS_ENABLE=1` at the call site; the sink
    /// itself is unconditional.
    fn emit_tree_sitter_grammar_event(&self, event: TreeSitterGrammarEvent);
    /// J-73 Slice C: emitted by the knowledge retrieval path when a
    /// retrieval score is evaluated against a threshold and the cascade
    /// either skips or proceeds.
    fn emit_retrieval_abstained(&self, event: RetrievalAbstainedEvent);
}

/// Default no-op sink used in tests and when SSB is not configured.
/// All methods drop their event silently.
pub struct NoopKnowledgeSsbSink;

impl KnowledgeSsbSink for NoopKnowledgeSsbSink {
    fn emit_hyde_query(&self, _event: HydeQueryEvent) {}
    fn emit_vector_cache_hit_rate(&self, _event: VectorCacheHitRateEvent) {}
    fn emit_query_trace(&self, _event: QueryTraceEvent) {}
    fn emit_deep_pool_capped(&self, _event: DeepPoolCappedEvent) {}
    fn emit_rrf_quality_gate_fired(&self, _event: RrfQualityGateFiredEvent) {}
    fn emit_hyde_skipped_code_query(&self, _event: HydeSkippedCodeQueryEvent) {}
    fn emit_rrf_component_weights(&self, _event: RrfComponentWeightsEvent) {}
    fn emit_pagerank_run(&self, _event: PagerankRunEvent) {}
    fn emit_dkg_rebuild_started(&self, _event: DkgRebuildStartedEvent) {}
    fn emit_dkg_rebuild_complete(&self, _event: DkgRebuildCompleteEvent) {}
    fn emit_tree_sitter_grammar_event(&self, _event: TreeSitterGrammarEvent) {}
    fn emit_retrieval_abstained(&self, _event: RetrievalAbstainedEvent) {}
}

/// Convenience constructor for the default no-op sink.
/// Test code and inert wiring paths can call this without naming
/// the unit struct.
pub fn noop_sink() -> Arc<dyn KnowledgeSsbSink> {
    Arc::new(NoopKnowledgeSsbSink)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct RecordingSink {
        hyde: Mutex<Vec<HydeQueryEvent>>,
        cache: Mutex<Vec<VectorCacheHitRateEvent>>,
        trace: Mutex<Vec<QueryTraceEvent>>,
    }

    impl RecordingSink {
        fn new() -> Self {
            Self {
                hyde: Mutex::new(Vec::new()),
                cache: Mutex::new(Vec::new()),
                trace: Mutex::new(Vec::new()),
            }
        }
    }

    impl KnowledgeSsbSink for RecordingSink {
        fn emit_hyde_query(&self, event: HydeQueryEvent) {
            self.hyde.lock().expect("hyde lock").push(event);
        }
        fn emit_vector_cache_hit_rate(&self, event: VectorCacheHitRateEvent) {
            self.cache.lock().expect("cache lock").push(event);
        }
        fn emit_query_trace(&self, event: QueryTraceEvent) {
            self.trace.lock().expect("trace lock").push(event);
        }
        fn emit_deep_pool_capped(&self, _event: DeepPoolCappedEvent) {}
        fn emit_rrf_quality_gate_fired(&self, _event: RrfQualityGateFiredEvent) {}
        fn emit_hyde_skipped_code_query(&self, _event: HydeSkippedCodeQueryEvent) {}
        fn emit_rrf_component_weights(&self, _event: RrfComponentWeightsEvent) {}
        fn emit_pagerank_run(&self, _event: PagerankRunEvent) {}
        fn emit_dkg_rebuild_started(&self, _event: DkgRebuildStartedEvent) {}
        fn emit_dkg_rebuild_complete(&self, _event: DkgRebuildCompleteEvent) {}
        fn emit_tree_sitter_grammar_event(&self, _event: TreeSitterGrammarEvent) {}
        fn emit_retrieval_abstained(&self, _event: RetrievalAbstainedEvent) {}
    }

    #[test]
    fn noop_sink_drops_all_events() {
        let sink: Arc<dyn KnowledgeSsbSink> = noop_sink();
        sink.emit_hyde_query(HydeQueryEvent {
            query_hash: "abc".into(),
            hypothesis_len: 0,
            cache_hit: false,
            latency_ms: 0,
        });
        sink.emit_vector_cache_hit_rate(VectorCacheHitRateEvent {
            session_id: "s".into(),
            hits: 0,
            misses: 0,
            hit_rate_pct: 0.0,
        });
        sink.emit_query_trace(QueryTraceEvent {
            query_id: "id".into(),
            tier: "FAST".into(),
            stage_count: 0,
            result_count: 0,
            total_latency_ms: 0,
        });
    }

    #[test]
    fn recording_sink_captures_all_three_event_types() {
        let sink = Arc::new(RecordingSink::new());
        let dyn_sink: Arc<dyn KnowledgeSsbSink> = sink.clone();
        dyn_sink.emit_hyde_query(HydeQueryEvent {
            query_hash: "h1".into(),
            hypothesis_len: 128,
            cache_hit: false,
            latency_ms: 10,
        });
        dyn_sink.emit_vector_cache_hit_rate(VectorCacheHitRateEvent {
            session_id: "s1".into(),
            hits: 5,
            misses: 2,
            hit_rate_pct: 71.43,
        });
        dyn_sink.emit_query_trace(QueryTraceEvent {
            query_id: "q1".into(),
            tier: "DEEP".into(),
            stage_count: 9,
            result_count: 12,
            total_latency_ms: 245,
        });

        assert_eq!(sink.hyde.lock().expect("hyde").len(), 1);
        assert_eq!(sink.cache.lock().expect("cache").len(), 1);
        assert_eq!(sink.trace.lock().expect("trace").len(), 1);
    }
}
