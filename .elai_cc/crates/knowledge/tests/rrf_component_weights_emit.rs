//! E-17 SP2 task 7 — rrf_component_weights SSB emit contract test.
//!
//! Verifies the new sink event type round-trips through the typed
//! KnowledgeSsbSink trait and that the no-op default sink drops the event
//! without error.

use elai_knowledge::knowledge_ssb_sink::{
    noop_sink, KnowledgeSsbSink, RrfComponentWeightsEvent,
};

#[test]
fn rrf_component_weights_event_drops_on_noop_sink() {
    let sink = noop_sink();
    sink.emit_rrf_component_weights(RrfComponentWeightsEvent {
        bm25_weight: Some(1.0),
        code_vec_weight: Some(1.0),
        text_vec_weight: Some(0.8),
    });
    // Should not panic and should not block.
}

#[test]
fn rrf_component_weights_event_supports_partial_fields() {
    // When code_vec is not enabled, its weight is None.
    let sink = noop_sink();
    sink.emit_rrf_component_weights(RrfComponentWeightsEvent {
        bm25_weight: Some(1.0),
        code_vec_weight: None,
        text_vec_weight: Some(0.8),
    });
    sink.emit_rrf_component_weights(RrfComponentWeightsEvent {
        bm25_weight: Some(1.0),
        code_vec_weight: None,
        text_vec_weight: None,
    });
}

/// Recording sink to observe a single emission.
struct RecordingSink {
    captured: std::sync::Mutex<Vec<RrfComponentWeightsEvent>>,
}

impl KnowledgeSsbSink for RecordingSink {
    fn emit_hyde_query(&self, _: elai_knowledge::knowledge_ssb_sink::HydeQueryEvent) {}
    fn emit_vector_cache_hit_rate(
        &self,
        _: elai_knowledge::knowledge_ssb_sink::VectorCacheHitRateEvent,
    ) {
    }
    fn emit_query_trace(&self, _: elai_knowledge::knowledge_ssb_sink::QueryTraceEvent) {}
    fn emit_deep_pool_capped(
        &self,
        _: elai_knowledge::knowledge_ssb_sink::DeepPoolCappedEvent,
    ) {
    }
    fn emit_rrf_quality_gate_fired(
        &self,
        _: elai_knowledge::knowledge_ssb_sink::RrfQualityGateFiredEvent,
    ) {
    }
    fn emit_hyde_skipped_code_query(
        &self,
        _: elai_knowledge::knowledge_ssb_sink::HydeSkippedCodeQueryEvent,
    ) {
    }
    fn emit_rrf_component_weights(&self, event: RrfComponentWeightsEvent) {
        self.captured.lock().expect("lock").push(event);
    }
}

#[test]
fn recording_sink_captures_emission() {
    let sink = std::sync::Arc::new(RecordingSink {
        captured: std::sync::Mutex::new(Vec::new()),
    });
    let dyn_sink: std::sync::Arc<dyn KnowledgeSsbSink> = sink.clone();
    dyn_sink.emit_rrf_component_weights(RrfComponentWeightsEvent {
        bm25_weight: Some(1.0),
        code_vec_weight: Some(1.0),
        text_vec_weight: Some(0.8),
    });
    let captured = sink.captured.lock().expect("lock");
    assert_eq!(captured.len(), 1);
    assert_eq!(captured[0].bm25_weight, Some(1.0));
    assert_eq!(captured[0].code_vec_weight, Some(1.0));
    assert_eq!(captured[0].text_vec_weight, Some(0.8));
}
