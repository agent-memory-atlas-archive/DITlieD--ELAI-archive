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
    DeepPoolCappedEvent, DkgRebuildCompleteEvent, DkgRebuildStartedEvent, HydeQueryEvent,
    HydeSkippedCodeQueryEvent, KnowledgeSsbSink, PagerankRunEvent, QueryTraceEvent,
    RrfComponentWeightsEvent, RrfQualityGateFiredEvent, TreeSitterGrammarEvent,
    TreeSitterGrammarEventKind, VectorCacheHitRateEvent,
};
use tracing::warn;

use crate::ssb::{
    DeepPoolCappedPayload, DkgRebuildCompletePayload, DkgRebuildStartedPayload, HydeQueryPayload,
    HydeSkippedCodeQueryPayload, PagerankRunPayload, QueryTracePayload, RrfComponentWeightsPayload,
    RrfQualityGateFiredPayload, Ssb, SsbPayload, TreeSitterGrammarEventKind as SsbTsKind,
    TreeSitterGrammarEventPayload, VectorCacheHitRatePayload,
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

    fn emit_deep_pool_capped(&self, event: DeepPoolCappedEvent) {
        let payload = SsbPayload::DeepPoolCapped(DeepPoolCappedPayload {
            capped_count: event.capped_count,
            max_allowed: event.max_allowed,
            stage_count: event.stage_count,
        });
        self.dispatch_payload(payload);
    }

    fn emit_rrf_quality_gate_fired(&self, event: RrfQualityGateFiredEvent) {
        let payload = SsbPayload::RrfQualityGateFired(RrfQualityGateFiredPayload {
            dropped_count: event.dropped_count,
            floor: event.floor,
            total_before: event.total_before,
        });
        self.dispatch_payload(payload);
    }

    fn emit_hyde_skipped_code_query(&self, event: HydeSkippedCodeQueryEvent) {
        let payload = SsbPayload::HydeSkippedCodeQuery(HydeSkippedCodeQueryPayload {
            query_class: event.query_class,
            word_count: event.word_count,
            code_signal_present: event.code_signal_present,
        });
        self.dispatch_payload(payload);
    }

    fn emit_rrf_component_weights(&self, event: RrfComponentWeightsEvent) {
        let payload = SsbPayload::RrfComponentWeights(RrfComponentWeightsPayload {
            bm25_weight: event.bm25_weight,
            code_vec_weight: event.code_vec_weight,
            text_vec_weight: event.text_vec_weight,
        });
        self.dispatch_payload(payload);
    }

    fn emit_pagerank_run(&self, event: PagerankRunEvent) {
        let payload = SsbPayload::PagerankRun(PagerankRunPayload {
            workspace_root_hash: event.workspace_root_hash,
            node_count: event.node_count,
            top_node_hash: event.top_node_hash,
            elapsed_ms: event.elapsed_ms,
        });
        self.dispatch_payload(payload);
    }

    fn emit_dkg_rebuild_started(&self, event: DkgRebuildStartedEvent) {
        let payload = SsbPayload::DkgRebuildStarted(DkgRebuildStartedPayload {
            workspace_root_hash: event.workspace_root_hash,
            trigger_reason: event.trigger_reason,
        });
        self.dispatch_payload(payload);
    }

    fn emit_dkg_rebuild_complete(&self, event: DkgRebuildCompleteEvent) {
        let payload = SsbPayload::DkgRebuildComplete(DkgRebuildCompletePayload {
            workspace_root_hash: event.workspace_root_hash,
            success: event.success,
            duration_ms: event.duration_ms,
            node_count: event.node_count,
        });
        self.dispatch_payload(payload);
    }

    fn emit_tree_sitter_grammar_event(&self, event: TreeSitterGrammarEvent) {
        // Map knowledge-side kind enum to orchestrator-side SSB kind enum.
        let ssb_kind = match event.kind {
            TreeSitterGrammarEventKind::AbiProbeMismatch => SsbTsKind::AbiProbeMismatch,
            TreeSitterGrammarEventKind::SvelteRunesParseHealth => {
                SsbTsKind::SvelteRunesParseHealth
            }
            TreeSitterGrammarEventKind::TierBGrammarLoaded => SsbTsKind::TierBGrammarLoaded,
            TreeSitterGrammarEventKind::TreeSitterSkippedLargeFile => {
                SsbTsKind::TreeSitterSkippedLargeFile
            }
        };
        // Build a minimal details_json from the optional free-form fields.
        let details_json = {
            let mut m = serde_json::Map::new();
            if let Some(ref lid) = event.language_id {
                m.insert("language_id".to_owned(), serde_json::Value::String(lid.clone()));
            }
            if let Some(ref d) = event.detail {
                m.insert("detail".to_owned(), serde_json::Value::String(d.clone()));
            }
            serde_json::Value::Object(m)
        };
        let ts_ms = std::time::SystemTime::now() // entropy-allow: SSB journal ts_ms, no ClockProvider in scope (sync dispatch path)
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
            .unwrap_or(0);
        let payload = SsbPayload::TreeSitterGrammarEvent(TreeSitterGrammarEventPayload {
            kind: ssb_kind,
            file: event.language_id.clone(),
            details_json,
            ts_ms,
        });
        self.dispatch_payload(payload);
    }
}
