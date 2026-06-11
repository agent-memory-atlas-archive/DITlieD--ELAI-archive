//! WAVE-A on-path smoke for the E-52-followon LSP diagnostic-probe gate
//! (ELAI_LSP_DIAGNOSTIC_PROBE_ENABLE).
//!
//! Drives the REAL public gate predicate
//! `elai_orchestrator::diagnostic_probe::lsp_diagnostic_probe_enabled_from_env()`
//! (default-OFF pinned, ON only on strict "1") AND exercises the gated
//! language->ProbeKind mapping the probe-completed payload consumes. The
//! mapping is the off-agent observable. Real subprocess dispatch is tracked
//! by promotion_plan_id E-52-followon.

use std::sync::Mutex;

use elai_knowledge::diagnostic::ProbeKind;
use elai_orchestrator::diagnostic_probe::{
    lsp_diagnostic_probe_enabled_from_env, probe_kind_for_language,
};

static ENV_LOCK: Mutex<()> = Mutex::new(());

const GATE: &str = "ELAI_LSP_DIAGNOSTIC_PROBE_ENABLE";

#[test]
fn diagnostic_probe_gate_off_by_default() {
    let _g = ENV_LOCK.lock().expect("env lock");
    std::env::remove_var(GATE);
    assert!(
        !lsp_diagnostic_probe_enabled_from_env(),
        "CONTRACT: {GATE} MUST default OFF (wire-dark per E-52-followon)"
    );
}

#[test]
fn diagnostic_probe_gate_on_when_enabled() {
    let _g = ENV_LOCK.lock().expect("env lock");
    std::env::set_var(GATE, "1");
    let on = lsp_diagnostic_probe_enabled_from_env();
    std::env::remove_var(GATE);
    assert!(
        on,
        "CONTRACT: {GATE}=1 MUST open the E-52 diagnostic-probe dispatch gate"
    );
}

#[test]
fn probe_kind_mapping_covers_supported_languages() {
    // ON-path observable: the gated dispatch maps each language slug to the
    // probe binary kind the completed payload reports. Pin the canonical
    // mapping so a probe wired downstream emits the right ProbeKind.
    assert_eq!(probe_kind_for_language("rust"), ProbeKind::RustAnalyzer);
    assert_eq!(probe_kind_for_language("python"), ProbeKind::Pyright);
    assert_eq!(probe_kind_for_language("typescript"), ProbeKind::TscNoEmit);
    assert_eq!(probe_kind_for_language("svelte"), ProbeKind::SvelteCheck);
    // Unknown slug stays stable as GenericLsp (wire never breaks).
    assert_eq!(probe_kind_for_language("cobol"), ProbeKind::GenericLsp);
}
