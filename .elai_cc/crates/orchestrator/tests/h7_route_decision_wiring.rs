//! H-7 — calibrated routing contract (Tier-1 source-wiring gate).
//!
//! Grounded scope (workflow/reasoning/H-7-decisions.md): the RouteDecisionEvent SSB
//! producer was wire-dark — the only emitter (cloud_mode emit_route_decision_log) is
//! tracing-only and lives on route(), which has ZERO production callers (the prod path
//! is route_with_spend in the task_runner cap block). H-7 emits RouteDecisionEvent on
//! the PRODUCTION cap path, gated on ELAI_ROUTE_DECISION_SSB_ENABLE (already allowlisted,
//! no schema bump). The calibrated `score`/layer_failed (HEAVY-1 via automix_gate) + the
//! per-role architect/editor/verifier dispatch (HEAVY-2) + the auditor-owned ON-path
//! smoke test (GAP-2) are documented follow-ons. Tier-2 (live route capture + the
//! calibrated routing) is DEFERRED.
//!
//! Placed in orchestrator/tests (NOT the currently-G-blocked elai_cc_contracts crate)
//! so it compile-verifies via `cargo check -p elai-orchestrator`.

use std::fs;
use std::path::PathBuf;

fn src(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

/// GAP-1 fix: RouteDecisionEvent is emitted on the PRODUCTION route path (the
/// task_runner cap block), not the caller-less route(). The off-agent SSB row now lands.
#[test]
fn route_decision_event_emitted_on_prod_cap_path() {
    let tr = src("task_runner.rs");
    assert!(
        tr.contains("RouteDecisionEvent"),
        "task_runner must emit a RouteDecisionEvent on the production route_with_spend cap path \
         (the legacy route() emitter has zero prod callers — GAP-1)"
    );
    assert!(
        tr.contains("route_decision_ssb_enabled"),
        "the emit must be gated on route_decision_ssb_enabled() (ELAI_ROUTE_DECISION_SSB_ENABLE, R9)"
    );
}

/// The gate fn is reachable from task_runner (promoted to pub(crate)).
#[test]
fn route_decision_gate_is_crate_visible() {
    let cm = src("cloud_mode.rs");
    assert!(
        cm.contains("pub(crate) fn route_decision_ssb_enabled"),
        "route_decision_ssb_enabled() must be pub(crate) so the task_runner prod path can gate on it"
    );
}
