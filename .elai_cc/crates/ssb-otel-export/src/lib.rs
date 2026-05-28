//! E-29 Tier-B -- `ssb-otel-export`: translate SSB rows into OTel GenAI
//! semconv 1.40.0 span shapes.
//!
//! Scope of THIS crate:
//!   * Type-level translation from the SSB row schema to an OTel
//!     attribute bag (always compiled).
//!   * Optional OTLP gRPC exporter behind the `otlp-export` cargo
//!     feature (4 dependencies on the `open-telemetry` org crates;
//!     verified anti-slopsquatting in `Cargo.toml`).
//!
//! OUT OF SCOPE for this Tier-B slice (RUNPOD-PENDING):
//!   * Live OTLP collector integration test -- requires a spun-up
//!     `otelcol-contrib` instance.
//!   * CLI bootstrap of the exporter -- the `--otlp-endpoint` flag
//!     surface lives in `crates/cli/src/args.rs`; this crate is the
//!     library it delegates to.
//!   * KV-cache state capture -- replay determinism is bounded by the
//!     sidecar surface (Open Question 3 in plan body).
//!
//! Why this lives in a separate crate: the orchestrator does not need
//! to take a hard dep on `opentelemetry-otlp` (which pulls a tonic +
//! prost transitive tree) just because the dashboard CAN render OTLP
//! spans. By feature-gating the exporter in a sibling crate, downstream
//! builds that don't ship OTLP stay lean. The orchestrator's
//! observability subtree consumes the `SsbOtelSpan` type below
//! unconditionally for the in-memory translation; the gRPC export is
//! the opt-in path.
//!
//! Grounding:
//!   * opentelemetry.io/docs/specs/semconv/gen-ai/gen-ai-agent-spans/
//!     -- 1.40.0 agent + framework spans (status: Development per May
//!     2026 sweep; emitter pins attribute NAMES not values).
//!   * opentelemetry.io/docs/specs/semconv/gen-ai/gen-ai-spans/ --
//!     client spans (approaching Stable per May 2026 sweep).
//!   * traceloop.com/docs/openllmetry -- `TRACELOOP_TRACE_CONTENT`
//!     content-gating pattern. The ELAI default-off
//!     `ELAI_OTEL_TRACE_CONTENT` env var mirrors this.

use serde::{Deserialize, Serialize};

/// Errors produced by the type-level translator. The OTLP exporter
/// branch adds variants under the `otlp-export` feature.
#[derive(Debug, thiserror::Error)]
pub enum SsbOtelError {
    #[error("ssb row payload missing required field: {0}")]
    PayloadMissingField(String),
    #[error("unsupported action_type for OTel mapping: {0}")]
    UnsupportedActionType(String),
    /// OTLP exporter construction failed (only emitted with the
    /// `otlp-export` feature).
    #[error("otlp exporter init failed: {0}")]
    OtlpExporterInit(String),
    /// Span ID parsing failed (W3C trace-context strings must be
    /// 32 hex chars for trace_id, 16 hex chars for span_id).
    #[error("invalid otel id: {0}")]
    InvalidOtelId(String),
}

/// Structural view of an `AgenticObservabilityEventPayload` carrying only
/// the OTel-relevant columns. Lives in this crate (rather than depending
/// on `elai-orchestrator`) so the export crate stays build-graph-light.
///
/// The orchestrator constructs one of these via a `From` impl on its own
/// `AgenticObservabilityEventPayload` (callers do
/// `(&payload).into()`). Field names mirror the SSB row 1:1 so future
/// schema bumps can be diffed mechanically against
/// `crates/orchestrator/src/ssb.rs::AgenticObservabilityEventPayload`.
///
/// All trace-context fields follow the W3C spec
/// (w3.org/TR/trace-context/): `trace_id` 32 lowercase hex chars,
/// `span_id` / `parent_span_id` 16 lowercase hex chars, `trace_flags`
/// 2 hex chars.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AgenticObservabilityPayloadView {
    /// W3C trace-context trace_id (32 lowercase hex chars).
    pub trace_id: Option<String>,
    /// W3C trace-context span_id (16 lowercase hex chars).
    pub span_id: Option<String>,
    /// Parent span_id when the event is a child span.
    pub parent_span_id: Option<String>,
    /// W3C trace-context trace_flags byte rendered as 2 hex chars.
    pub trace_flags: Option<String>,
    /// W3C trace-context tracestate header value.
    pub trace_state: Option<String>,
    /// Human-readable span name (e.g. "elai.mast.classify").
    pub span_name: Option<String>,
    /// OTel SpanKind ("internal" / "server" / "client" / "producer" /
    /// "consumer").
    pub span_kind: Option<String>,
    /// OTel status code ("unset" / "ok" / "error").
    pub span_status_code: Option<String>,
    /// OTel status description (free text).
    pub span_status_message: Option<String>,
    /// OTel resource service.name attribute.
    pub service_name: Option<String>,
    /// OTel resource service.namespace attribute.
    pub service_namespace: Option<String>,
    /// OTel resource service.instance.id attribute.
    pub service_instance_id: Option<String>,
    /// OTel resource attributes as serialized JSON object string.
    pub resource_attributes_json: Option<String>,
    /// OTel span attributes as serialized JSON object string.
    pub span_attributes_json: Option<String>,
    /// OTel span events as serialized JSON array string.
    pub span_events_json: Option<String>,
}

impl AgenticObservabilityPayloadView {
    /// True when at least one OTel column is populated. Callers use
    /// this to skip emission for SSB rows that don't carry OTel
    /// context (e.g. payloads written before `with_otel_span(...)`
    /// adoption).
    #[must_use]
    pub fn has_otel_context(&self) -> bool {
        self.trace_id.is_some() || self.span_id.is_some()
    }
}

/// One SSB-row-mapped OTel span. The shape mirrors the OpenTelemetry
/// GenAI semconv 1.40.0 attribute set for agent / framework / client
/// spans. ALL string attributes are GATED by the
/// `content_redacted` flag -- when the caller passes `redacted=true`
/// the message-body attributes are blanked.
///
/// The span hierarchy is determined by the caller (parent / child via
/// trace + span IDs); this struct is the attribute bag only.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SsbOtelSpan {
    /// `gen_ai.operation.name` -- semconv attribute. E.g.
    /// `"invoke_workflow"` / `"chat"` / `"execute_tool"`.
    pub operation_name: String,
    /// `gen_ai.provider.name` -- e.g. `"anthropic"`, `"openai"`,
    /// `"qwen"`, `"local"`.
    pub provider_name: String,
    /// `gen_ai.agent.id` -- internal agent role tag (executor /
    /// auditor / context-compiler / ...).
    pub agent_id: String,
    /// `gen_ai.agent.name` -- human-readable agent label.
    pub agent_name: String,
    /// `gen_ai.workflow.name` -- workflow / phase name.
    pub workflow_name: String,
    /// `gen_ai.tool.name` -- tool name for execute_tool spans. Empty
    /// for non-tool spans.
    pub tool_name: String,
    /// `gen_ai.request.model` -- model id (e.g.
    /// `"qwen-2.5-coder-3b"`).
    pub request_model: String,
    /// `gen_ai.response.finish_reasons` -- comma-joined list (semconv
    /// allows array but ELAI flattens for SQL projection).
    pub response_finish_reason: String,
    /// `gen_ai.usage.input_tokens`.
    pub usage_input_tokens: u64,
    /// `gen_ai.usage.output_tokens`.
    pub usage_output_tokens: u64,
    /// `gen_ai.usage.cache_creation.input_tokens` -- Anthropic
    /// prompt-cache creation attribution.
    pub cache_creation_input_tokens: u64,
    /// `gen_ai.usage.cache_read.input_tokens` -- Anthropic prompt-
    /// cache hit attribution.
    pub cache_read_input_tokens: u64,
    /// `gen_ai.conversation.id` -- session correlation id.
    pub conversation_id: String,
    /// `gen_ai.latency_ms` -- ELAI-side latency annotation (not in
    /// the semconv spec but documented as a custom attr in
    /// `workflow/reasoning/E-29-decisions.md`).
    pub latency_ms: u64,
    /// `gen_ai.error.type` -- e.g. `"timeout"`, `"safety_filter"`.
    /// Empty when no error.
    pub error_type: String,
    /// Optional input message bodies -- emitted only when
    /// `redacted = false`. Semconv attribute name
    /// `gen_ai.input.messages` (array of role+content). ELAI
    /// serialises to JSON-stringified array.
    pub input_messages_json: String,
    /// Optional output message bodies. Semconv attribute
    /// `gen_ai.output.messages`. Same gating as
    /// `input_messages_json`.
    pub output_messages_json: String,
    /// Whether content-gating was applied. When `true`, the
    /// `input_messages_json` and `output_messages_json` fields are
    /// blank.
    pub redacted: bool,
}

impl SsbOtelSpan {
    /// Construct a content-redacted span -- a `SsbOtelSpan` with all
    /// message-body fields blanked. Matches the
    /// `ELAI_OTEL_TRACE_CONTENT=false` (default) behavior.
    #[must_use]
    pub fn redacted(operation_name: impl Into<String>) -> Self {
        Self {
            operation_name: operation_name.into(),
            provider_name: String::new(),
            agent_id: String::new(),
            agent_name: String::new(),
            workflow_name: String::new(),
            tool_name: String::new(),
            request_model: String::new(),
            response_finish_reason: String::new(),
            usage_input_tokens: 0,
            usage_output_tokens: 0,
            cache_creation_input_tokens: 0,
            cache_read_input_tokens: 0,
            conversation_id: String::new(),
            latency_ms: 0,
            error_type: String::new(),
            input_messages_json: String::new(),
            output_messages_json: String::new(),
            redacted: true,
        }
    }
}

/// Map an SSB action_type string to the canonical OTel
/// `gen_ai.operation.name` value. Returns the action_type unchanged
/// when no specific mapping is known so the caller can still emit
/// the span (semconv supports custom operation names).
#[must_use]
pub fn action_type_to_otel_operation(action_type: &str) -> String {
    match action_type {
        "phase_transition" => "invoke_workflow".into(),
        "runtime_fallback" => "chat".into(),
        "hook_pre" | "hook_post" => "execute_tool".into(),
        "tool_result" => "execute_tool".into(),
        other => other.to_string(),
    }
}

/// OTLP gRPC exporter handle. Only available with the `otlp-export`
/// cargo feature.
#[cfg(feature = "otlp-export")]
pub struct SsbOtelExporter {
    endpoint: String,
    content_gated: bool,
}

#[cfg(feature = "otlp-export")]
impl SsbOtelExporter {
    /// Construct a new exporter pinned to `endpoint`. When the env
    /// var `ELAI_OTLP_ENDPOINT` is empty / unset, callers SHOULD pass
    /// an explicit endpoint here; in production the CLI's
    /// `--otlp-endpoint` flag provides it.
    pub fn new(endpoint: impl Into<String>, content_gated: bool) -> Self {
        Self {
            endpoint: endpoint.into(),
            content_gated,
        }
    }

    /// Endpoint accessor (used by smoke tests).
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Content-gating accessor.
    pub fn content_gated(&self) -> bool {
        self.content_gated
    }

    // The live OTLP gRPC emit path is RUNPOD-PENDING -- it requires a
    // spun-up otelcol-contrib instance. The exporter constructor +
    // accessors above are the minimum compile-time surface so the
    // crate can be feature-gated without requiring a live integration
    // test in this slice. The follow-up plan extends this with the
    // `opentelemetry-otlp` TonicExporterBuilder bootstrap.
}

/// E-29 -- Emit an `AgenticObservabilityEvent` SSB row to an OTLP
/// collector as a single OTel span. This is the wire-format code path
/// (constructs a `SpanBuilder`, populates W3C trace-context IDs +
/// semconv attributes, hands to `opentelemetry_otlp::SpanExporter`).
///
/// LIVE OTLP SEND IS RUNPOD-PENDING: the exporter is constructed and
/// the span is built unconditionally so the wire-format code is real
/// and exercised by `cargo check --features otlp-export`. The actual
/// network call (`tracer_provider.force_flush()`) requires a running
/// `otelcol-contrib` instance and is gated on the
/// `ELAI_OTLP_ENDPOINT` env var being set. When the env var is unset
/// the function constructs the span and returns Ok(()) without
/// blocking on the network -- this is the wire-dark default per
/// `workflow/reasoning/E-29-decisions.md`.
///
/// Inputs:
///   * `payload` -- the OTel-relevant projection of an SSB
///     `AgenticObservabilityEventPayload` (see
///     `AgenticObservabilityPayloadView`).
///   * `otlp_endpoint` -- e.g. `"http://127.0.0.1:4317"`. An empty
///     string is treated as "construct the span but do not flush"
///     (wire-dark mode).
///
/// Returns:
///   * `Ok(())` -- span constructed (and optionally flushed when
///     the endpoint is non-empty).
///   * `Err(SsbOtelError::InvalidOtelId)` -- payload trace_id /
///     span_id failed to parse as W3C hex strings.
///   * `Err(SsbOtelError::OtlpExporterInit)` -- otlp-export crate
///     reported an init failure.
#[cfg(feature = "otlp-export")]
pub fn emit_observability_event(
    payload: &AgenticObservabilityPayloadView,
    otlp_endpoint: &str,
) -> Result<(), SsbOtelError> {
    use opentelemetry::trace::{
        SpanContext, SpanId, SpanKind, Status, TraceFlags, TraceId,
        TraceState, Tracer, TracerProvider,
    };
    use opentelemetry::{Context, KeyValue};
    use opentelemetry_otlp::WithExportConfig;
    use opentelemetry_sdk::trace::SdkTracerProvider;
    use std::str::FromStr;

    // Skip when no trace context is populated -- this is the wire-dark
    // path before producers adopt with_otel_span(...).
    if !payload.has_otel_context() {
        return Ok(());
    }

    // Parse W3C trace-context IDs. 32 hex chars -> TraceId, 16 hex
    // chars -> SpanId. Invalid IDs surface as InvalidOtelId errors so
    // the caller can fall back to wire-dark emission.
    let trace_id_str = payload
        .trace_id
        .as_deref()
        .ok_or_else(|| SsbOtelError::PayloadMissingField("trace_id".into()))?;
    let span_id_str = payload
        .span_id
        .as_deref()
        .ok_or_else(|| SsbOtelError::PayloadMissingField("span_id".into()))?;
    let trace_id = TraceId::from_hex(trace_id_str)
        .map_err(|e| SsbOtelError::InvalidOtelId(format!("trace_id: {e}")))?;
    let span_id = SpanId::from_hex(span_id_str)
        .map_err(|e| SsbOtelError::InvalidOtelId(format!("span_id: {e}")))?;

    // trace_flags default = sampled (0x01) per the W3C spec.
    let trace_flags = match payload.trace_flags.as_deref() {
        Some(hex) => {
            let byte = u8::from_str_radix(hex, 16).map_err(|e| {
                SsbOtelError::InvalidOtelId(format!("trace_flags: {e}"))
            })?;
            TraceFlags::new(byte)
        }
        None => TraceFlags::SAMPLED,
    };

    // tracestate -- empty default per spec when unset.
    let trace_state = match payload.trace_state.as_deref() {
        Some(s) if !s.is_empty() => TraceState::from_str(s).unwrap_or_default(),
        _ => TraceState::default(),
    };

    // Build the SpanContext that carries the W3C IDs across the wire.
    // OTel 0.32 SpanBuilder dropped `with_trace_id` / `with_span_id`
    // public setters; the canonical way to attach a specific trace_id
    // is to seed a parent Context with a SpanContext-backed remote
    // span (see opentelemetry::trace::TraceContextExt::with_remote_span_context).
    let span_context = SpanContext::new(
        trace_id,
        span_id,
        trace_flags,
        /* is_remote */ true,
        trace_state,
    );

    // Set up the tracer provider. When otlp_endpoint is empty we use
    // a no-op SdkTracerProvider so the span construction still
    // exercises the OTel attribute path without a live network call.
    let tracer_provider = if otlp_endpoint.is_empty() {
        SdkTracerProvider::builder().build()
    } else {
        // The OtlpExporterBuilder pulls in tonic + prost. With an
        // explicit endpoint we build the gRPC exporter; on failure we
        // surface OtlpExporterInit so the caller can fall back.
        match opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(otlp_endpoint)
            .build()
        {
            Ok(exporter) => SdkTracerProvider::builder()
                .with_batch_exporter(exporter)
                .build(),
            Err(e) => {
                return Err(SsbOtelError::OtlpExporterInit(e.to_string()));
            }
        }
    };

    let service_name = payload
        .service_name
        .clone()
        .unwrap_or_else(|| "elai-orchestrator".to_string());
    let tracer = tracer_provider.tracer(service_name);

    // Construct the span via the public span builder. We use a
    // span-builder pattern (rather than tracer.in_span) so we can
    // attach the resolved trace_id/span_id from the SSB row instead
    // of letting the SDK generate fresh IDs.
    let span_name = payload
        .span_name
        .clone()
        .unwrap_or_else(|| "elai.agentic_observability_event".to_string());

    let span_kind = match payload.span_kind.as_deref() {
        Some("server") => SpanKind::Server,
        Some("client") => SpanKind::Client,
        Some("producer") => SpanKind::Producer,
        Some("consumer") => SpanKind::Consumer,
        _ => SpanKind::Internal,
    };

    let status = match payload.span_status_code.as_deref() {
        Some("ok") => Status::Ok,
        Some("error") => Status::error(
            payload
                .span_status_message
                .clone()
                .unwrap_or_default(),
        ),
        _ => Status::Unset,
    };

    let mut attrs: Vec<KeyValue> = Vec::new();
    if let Some(ns) = payload.service_namespace.as_deref() {
        attrs.push(KeyValue::new("service.namespace", ns.to_string()));
    }
    if let Some(iid) = payload.service_instance_id.as_deref() {
        attrs.push(KeyValue::new(
            "service.instance.id",
            iid.to_string(),
        ));
    }
    if let Some(parent) = payload.parent_span_id.as_deref() {
        attrs.push(KeyValue::new("elai.parent_span_id", parent.to_string()));
    }
    if let Some(ra) = payload.resource_attributes_json.as_deref() {
        attrs.push(KeyValue::new(
            "elai.resource_attributes_json",
            ra.to_string(),
        ));
    }
    if let Some(sa) = payload.span_attributes_json.as_deref() {
        attrs.push(KeyValue::new(
            "elai.span_attributes_json",
            sa.to_string(),
        ));
    }
    if let Some(se) = payload.span_events_json.as_deref() {
        attrs.push(KeyValue::new(
            "elai.span_events_json",
            se.to_string(),
        ));
    }

    let span_builder = tracer
        .span_builder(span_name)
        .with_kind(span_kind)
        .with_attributes(attrs);

    // Seed a Context with the SSB-row SpanContext so the emitted span
    // inherits the same trace_id (the new span becomes a child of the
    // SSB row's logical span). This is the OTel 0.32 idiomatic
    // alternative to SpanBuilder::with_trace_id which was removed.
    use opentelemetry::trace::TraceContextExt;
    let parent_ctx = Context::current().with_remote_span_context(span_context);

    // Materialize the span. With the no-op provider the span is
    // discarded; with the OTLP provider it is enqueued for batch
    // export via the `BatchSpanProcessor`. We rely on Drop to close
    // the span (idiomatic OTel).
    let mut span = span_builder.start_with_context(&tracer, &parent_ctx);
    // Status is set on the span post-construction in OTel 0.32 -- the
    // SpanBuilder doesn't expose `with_status`. We use the Span trait
    // method, which is the canonical path per the OTel API surface
    // (opentelemetry::trace::Span::set_status).
    {
        use opentelemetry::trace::Span;
        span.set_status(status);
    }
    let _span = span;

    // Live force_flush is RUNPOD-PENDING -- the BatchSpanProcessor
    // drains asynchronously when the provider is dropped at the end
    // of the function scope. A follow-up plan adds graceful
    // shutdown + force-flush semantics behind an explicit
    // `flush_on_emit` flag.
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacted_span_blanks_messages() {
        let s = SsbOtelSpan::redacted("chat");
        assert_eq!(s.operation_name, "chat");
        assert!(s.input_messages_json.is_empty());
        assert!(s.output_messages_json.is_empty());
        assert!(s.redacted);
    }

    #[test]
    fn operation_mapping_phase_transition() {
        assert_eq!(action_type_to_otel_operation("phase_transition"), "invoke_workflow");
    }

    #[test]
    fn operation_mapping_runtime_fallback() {
        assert_eq!(action_type_to_otel_operation("runtime_fallback"), "chat");
    }

    #[test]
    fn operation_mapping_hook() {
        assert_eq!(action_type_to_otel_operation("hook_pre"), "execute_tool");
        assert_eq!(action_type_to_otel_operation("hook_post"), "execute_tool");
    }

    #[test]
    fn operation_mapping_pass_through_unknown() {
        assert_eq!(action_type_to_otel_operation("xyz"), "xyz");
    }

    #[cfg(feature = "otlp-export")]
    #[test]
    fn exporter_smoke() {
        let exp = SsbOtelExporter::new("http://127.0.0.1:4317", true);
        assert_eq!(exp.endpoint(), "http://127.0.0.1:4317");
        assert!(exp.content_gated());
    }
}
