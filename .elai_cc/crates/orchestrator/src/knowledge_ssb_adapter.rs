//! C-1: orchestrator-side adapter bridging the knowledge crate's typed
//! [`elai_knowledge::knowledge_ssb_sink::KnowledgeSsbSink`] trait into
//! [`crate::ssb::Ssb`] write_payload calls.
//!
//! Lives in the orchestrator crate (not in `knowledge`) so the
//! orchestrator -> knowledge dependency direction stays acyclic — the
//! knowledge crate must not depend on `SsbPayload` directly.
//!
//! Production wiring: `cli::live_cli::build_local_knowledge_stack`
//! constructs `Arc::new(OrchestratorKnowledgeSsbSink::new(ssb_arc))`
//! and threads it into `HydeStage::with_ssb`, `VectorCache::with_ssb`,
//! and `QueryPipeline::with_ssb`. Tests inject `NoopKnowledgeSsbSink`
//! or a recording sink instead.

use std::sync::Arc;

use elai_knowledge::knowledge_ssb_sink::{
    HydeQueryEvent, KnowledgeSsbSink, QueryTraceEvent, VectorCacheHitRateEvent,
};
use tracing::warn;

use crate::ssb::{
    HydeQueryPayload, QueryTracePayload, Ssb, SsbPayload, VectorCacheHitRatePayload,
};
use crate::task_runner::{RUNNER_AGENT_ID, RUNNER_SUPERSTEP};

/// Adapter: converts typed knowledge events into [`SsbPayload`] rows
/// written via [`Ssb::write_payload`]. Every emission is async-spawned
/// onto the current tokio runtime so the sync emit path on the
/// hyde / vector-cache hot paths is not blocked.
pub struct OrchestratorKnowledgeSsbSink {
    ssb: Arc<Ssb>,
}

impl OrchestratorKnowledgeSsbSink {
    pub fn new(ssb: Arc<Ssb>) -> Self {
        Self { ssb }
    }

    fn dispatch_payload(&self, payload: SsbPayload) {
        let ssb = self.ssb.clone();
        // The trait methods are sync, but the SSB writer is async. Spawn
        // onto the current runtime so the producer hot-path is never
        // blocked. If no runtime is active (synchronous test context)
        // the row is dropped with a warning — same semantics as the
        // existing degradation NoopSsbNotifier path.
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                handle.spawn(async move {
                    if let Err(e) = ssb
                        .write_payload(
                            RUNNER_AGENT_ID,
                            RUNNER_SUPERSTEP,
                            payload,
                            Vec::new(),
                        )
                        .await
                    {
                        warn!(error = %e, "knowledge ssb sink: write_payload failed (non-fatal)");
                    }
                });
            }
            Err(_) => {
                warn!(
                    "knowledge ssb sink: no tokio runtime active; dropping knowledge SSB row"
                );
            }
        }
    }
}

impl KnowledgeSsbSink for OrchestratorKnowledgeSsbSink {
    fn emit_hyde_query(&self, event: HydeQueryEvent) {
        let payload = SsbPayload::HydeQuery(HydeQueryPayload {
            query_hash: event.query_hash,
            hypothesis_len: event.hypothesis_len,
            cache_hit: event.cache_hit,
            latency_ms: event.latency_ms,
        });
        self.dispatch_payload(payload);
    }

    fn emit_vector_cache_hit_rate(&self, event: VectorCacheHitRateEvent) {
        let payload = SsbPayload::VectorCacheHitRate(VectorCacheHitRatePayload {
            session_id: event.session_id,
            hits: event.hits,
            misses: event.misses,
            hit_rate_pct: event.hit_rate_pct,
        });
        self.dispatch_payload(payload);
    }

    fn emit_query_trace(&self, event: QueryTraceEvent) {
        let payload = SsbPayload::QueryTrace(QueryTracePayload {
            query_id: event.query_id,
            tier: event.tier,
            stage_count: event.stage_count,
            result_count: event.result_count,
            total_latency_ms: event.total_latency_ms,
        });
        self.dispatch_payload(payload);
    }
}
