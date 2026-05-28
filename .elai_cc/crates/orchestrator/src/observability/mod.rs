//! E-29 agentic-observability subtree -- the Tier-B continuation of the
//! E-29 plan landed in cluster s-25120. Tier-A shipped TranscriptDrawer +
//! SpendDrawer + QueryDrawer (3 dashboard drawers) on top of pre-existing
//! Tauri commands. Tier-B in this slice lands:
//!
//! * [`mast_classify`] -- post-run classifier walking SsbViewer rows and
//!   projecting them onto the E-6-shipped [`crate::mast::MastMode`] 14-
//!   variant taxonomy. Reuses [`crate::mast::finding_to_mast_mode`] as the
//!   default classifier kernel; supplements with row-pattern heuristics.
//! * [`replay`] -- `ssb_resume_from(checkpoint_id, patch)` validation path
//!   that consumes [`crate::locked_keys_auditor::LockedKeysAuditor`] for
//!   the C-44/D-9 frozen-surface predicate. FAIL-CLOSED on any auditor
//!   error: the replay refuses all patches when the registry is
//!   unreadable, missing, or returns a violation.
//! * [`signal_match`] -- backfill primitive for Laminar-style natural-
//!   language outcome extraction. The classifier model itself is wire-
//!   dark behind the env gate; this module owns the payload shape +
//!   emitter helper.
//! * [`dataset_promote`] -- production-traces-to-eval-dataset promotion
//!   primitive (Langfuse / LangSmith pattern). The optional CI-gate via
//!   `ELAI_EVAL_GATE_REGRESSION_THRESHOLD` is documented but NOT
//!   enforced in this slice.
//! * [`pricing_rollup`] -- reuses
//!   [`providers::pricing::pricing_for_model`] to compute per-row USD
//!   cost; sums by phase / agent / tool. SpendDrawer already shipped
//!   over `get_cloud_spend`; this rollup is the per-session detail
//!   surface.
//!
//! Wire-dark posture: all four SSB fire-points hang off a SINGLE Pattern-2
//! variant [`crate::ssb::SsbActionType::AgenticObservabilityEvent`] with a
//! 4-arm kind discriminator (MastFailureClassified / ReplayResume /
//! SignalMatch / EvalDatasetMember). The variant is gated by
//! [`ELAI_AGENTIC_OBSERVABILITY_ENABLE_VAR`]; default-off keeps existing
//! SSB row counts stable.
//!
//! Source grounding:
//!   * arXiv:2503.13657 -- Cemri et al. NeurIPS 2025, "Why Do Multi-Agent
//!     LLM Systems Fail?" (MAST). PRIMARY anchor for [`mast_classify`].
//!     Verified in mast.rs module-doc (E-6 SHIPPED).
//!   * laminar.sh/article/2026-04-23-top-6-agent-observability-platforms
//!     -- Signals + SQL-over-traces + transcript-first observability.
//!     PRIMARY anchor for [`signal_match`] + the Tier-A QueryDrawer.
//!   * dev.to/sreeni5018/debugging-non-deterministic-llm-agents-
//!     implementing-checkpoint-based-state-replay-with-langgraph-5171 --
//!     LangGraph checkpoint replay pattern. PRIMARY anchor for
//!     [`replay`].
//!   * tianpan.co/blog/2026-04-12-deterministic-replay-debugging-non-
//!     deterministic-ai-agents -- temp=0+seed insufficient; full I/O
//!     capture required. Documented OUT-OF-SCOPE risk for
//!     [`replay`] (KV-state capture is RUNPOD-PENDING).
//!   * traceloop.com/docs/openllmetry/introduction -- OpenLLMetry
//!     `TRACELOOP_TRACE_CONTENT` content-gating pattern. PRIMARY anchor
//!     for the [`crate::ssb_otel_export`] content-gating contract.
//!   * github.com/langfuse/langfuse -- traces-to-datasets promotion
//!     pattern. PRIMARY anchor for [`dataset_promote`].
//!   * opentelemetry.io/docs/specs/semconv/gen-ai/gen-ai-agent-spans/ --
//!     GenAI semconv 1.40.0 (status: still labelled Development per May
//!     2026 sweep; ELAI emitter pins on 1.40.0 attribute names but
//!     tolerates drift). PRIMARY anchor for the optional
//!     `ssb-otel-export` crate behind the `otlp-export` feature.
//!
//! See `workflow/reasoning/E-29-decisions.md` (Tier-B section) for the
//! decision log + RUNPOD-PENDING list.

pub mod dataset_promote;
pub mod mast_classify;
pub mod pricing_rollup;
pub mod replay;
pub mod signal_match;

pub use dataset_promote::{
    promote_traces_to_dataset, EvalDatasetMember, EvalDatasetPromotion,
};
pub use mast_classify::{classify_closed_run, ClassifierError, MastModeHit};
pub use pricing_rollup::{compute_row_cost_usd, rollup_cost_by_phase, PricingRollupRow};
pub use replay::{ssb_resume_from, ReplayError, ReplayOutcome, ReplayPatch, ReplaySessionId};
pub use signal_match::{backfill_signal_matches, SignalMatchHit};

/// Environment-variable name controlling wire-dark SSB emission for the
/// E-29 agentic-observability event family. Set to `1` to enable;
/// default-off keeps existing tests' SSB row counts stable.
pub const ELAI_AGENTIC_OBSERVABILITY_ENABLE_VAR: &str = "ELAI_AGENTIC_OBSERVABILITY_ENABLE";

/// Returns true iff the E-29 SSB emission surface is wire-enabled via
/// the canonical env var. Default-off keeps the surface dark per
/// wire-dark posture (matches E-22 / E-23 / E-25 precedent).
#[must_use]
pub fn ssb_emission_enabled() -> bool {
    std::env::var(ELAI_AGENTIC_OBSERVABILITY_ENABLE_VAR)
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Environment-variable name controlling content-gating for OTel /
/// transcript content emission. When unset or `"false"`, the OTel
/// `gen_ai.input.messages` and `gen_ai.output.messages` attributes are
/// suppressed (matches OpenLLMetry `TRACELOOP_TRACE_CONTENT` default).
/// The transcript drawer respects this gate identically.
pub const ELAI_OTEL_TRACE_CONTENT_VAR: &str = "ELAI_OTEL_TRACE_CONTENT";

/// Returns true iff OTel / transcript content emission is enabled.
/// Default-off matches OpenLLMetry's privacy-first content-gating
/// pattern. Callers MUST consult this before emitting raw prompt /
/// completion bodies on any observability surface.
#[must_use]
pub fn otel_trace_content_enabled() -> bool {
    std::env::var(ELAI_OTEL_TRACE_CONTENT_VAR)
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Environment-variable name controlling the optional OTLP-export
/// feature gate. The `ssb-otel-export` crate is itself behind the
/// `otlp-export` cargo feature; this env var is an additional runtime
/// gate so a feature-built binary can still ship the surface dark.
/// Format: arbitrary OTLP gRPC endpoint (e.g. `http://127.0.0.1:4317`).
/// When unset, the exporter is constructed as a no-op.
pub const ELAI_OTLP_ENDPOINT_VAR: &str = "ELAI_OTLP_ENDPOINT";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enable_var_default_off() {
        // Unset state -- the function MUST return false. We do not call
        // `std::env::remove_var` here because the test process is
        // shared; rely on the default-off semantics being preserved
        // when the var is unset.
        let prev = std::env::var(ELAI_AGENTIC_OBSERVABILITY_ENABLE_VAR).ok();
        std::env::remove_var(ELAI_AGENTIC_OBSERVABILITY_ENABLE_VAR);
        assert!(!ssb_emission_enabled());
        if let Some(v) = prev {
            std::env::set_var(ELAI_AGENTIC_OBSERVABILITY_ENABLE_VAR, v);
        }
    }

    #[test]
    fn content_gate_default_off() {
        let prev = std::env::var(ELAI_OTEL_TRACE_CONTENT_VAR).ok();
        std::env::remove_var(ELAI_OTEL_TRACE_CONTENT_VAR);
        assert!(!otel_trace_content_enabled());
        if let Some(v) = prev {
            std::env::set_var(ELAI_OTEL_TRACE_CONTENT_VAR, v);
        }
    }
}
