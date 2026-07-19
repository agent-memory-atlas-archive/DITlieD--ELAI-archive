// E-29 named contract for the privacy-first OpenLLMetry-style content
// gate. The OFF-AGENT artifact is the boolean returned by the production
// `elai_orchestrator::observability::otel_trace_content_enabled` symbol
// (wiring-enforcement R4): each test drives the real env-gated function
// rather than re-implementing its semantics, so a regression in the
// production gate surfaces here.

use elai_orchestrator::observability::{
    otel_trace_content_enabled, ELAI_OTEL_TRACE_CONTENT_VAR,
};

#[test]
fn content_gate_default_off_when_unset() {
    let prev = std::env::var(ELAI_OTEL_TRACE_CONTENT_VAR).ok();
    std::env::remove_var(ELAI_OTEL_TRACE_CONTENT_VAR);
    assert!(!otel_trace_content_enabled());
    if let Some(v) = prev {
        std::env::set_var(ELAI_OTEL_TRACE_CONTENT_VAR, v);
    }
}

#[test]
fn content_gate_on_for_one_and_true() {
    let prev = std::env::var(ELAI_OTEL_TRACE_CONTENT_VAR).ok();
    for v in ["1", "true"] {
        std::env::set_var(ELAI_OTEL_TRACE_CONTENT_VAR, v);
        assert!(
            otel_trace_content_enabled(),
            "expected enabled for {:?}",
            v
        );
    }
    restore(ELAI_OTEL_TRACE_CONTENT_VAR, prev);
}

#[test]
fn content_gate_off_for_zero_and_false() {
    let prev = std::env::var(ELAI_OTEL_TRACE_CONTENT_VAR).ok();
    for v in ["0", "false"] {
        std::env::set_var(ELAI_OTEL_TRACE_CONTENT_VAR, v);
        assert!(
            !otel_trace_content_enabled(),
            "expected disabled for {:?}",
            v
        );
    }
    restore(ELAI_OTEL_TRACE_CONTENT_VAR, prev);
}

fn restore(var: &str, prev: Option<String>) {
    match prev {
        Some(v) => std::env::set_var(var, v),
        None => std::env::remove_var(var),
    }
}
