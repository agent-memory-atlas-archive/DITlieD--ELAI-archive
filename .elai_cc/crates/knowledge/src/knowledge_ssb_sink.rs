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
}

/// Default no-op sink used in tests and when SSB is not configured.
/// All three methods drop their event silently.
pub struct NoopKnowledgeSsbSink;

impl KnowledgeSsbSink for NoopKnowledgeSsbSink {
    fn emit_hyde_query(&self, _event: HydeQueryEvent) {}
    fn emit_vector_cache_hit_rate(&self, _event: VectorCacheHitRateEvent) {}
    fn emit_query_trace(&self, _event: QueryTraceEvent) {}
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
