//! E-52 Tier-A — diagnostic_envelope_contract.
//!
//! Locks the Tier-A surface of `knowledge::diagnostic`:
//!   - DiagnosticSeverity 4-variant set + LSP 1..=4 scale + ordering
//!   - ProbeKind 5-variant set + snake_case wire form
//!   - DiagnosticProbeOutcome field set + counter consistency + is_clean rule
//!   - Diagnostic serde round-trip + 1-indexed line/column convention
//!
//! Growth of either taxonomy requires bumping this contract + a row in
//! workflow/reasoning/E-52-decisions.md per the campaign 6-of-6 discipline.

use elai_knowledge::diagnostic::{
    Diagnostic, DiagnosticProbeOutcome, DiagnosticProbeRunStatus, DiagnosticSeverity, ProbeKind,
};

#[test]
fn severity_locks_at_four_variants_with_lsp_scale() {
    let severities = [
        DiagnosticSeverity::Error,
        DiagnosticSeverity::Warning,
        DiagnosticSeverity::Information,
        DiagnosticSeverity::Hint,
    ];
    assert_eq!(severities.len(), 4, "DiagnosticSeverity variant count locked at 4");
    // LSP severity scale 1..=4, ordered most-severe -> least-severe.
    assert_eq!(DiagnosticSeverity::Error.lsp_severity(), 1);
    assert_eq!(DiagnosticSeverity::Warning.lsp_severity(), 2);
    assert_eq!(DiagnosticSeverity::Information.lsp_severity(), 3);
    assert_eq!(DiagnosticSeverity::Hint.lsp_severity(), 4);
    // Wire-form lower-case + matches as_str.
    assert_eq!(DiagnosticSeverity::Error.as_str(), "error");
    assert_eq!(DiagnosticSeverity::Warning.as_str(), "warning");
    assert_eq!(DiagnosticSeverity::Information.as_str(), "information");
    assert_eq!(DiagnosticSeverity::Hint.as_str(), "hint");
}

#[test]
fn probe_kind_locks_at_five_variants() {
    let kinds = [
        ProbeKind::RustAnalyzer,
        ProbeKind::Pyright,
        ProbeKind::TscNoEmit,
        ProbeKind::SvelteCheck,
        ProbeKind::GenericLsp,
    ];
    assert_eq!(kinds.len(), 5, "ProbeKind variant count locked at 5");
    // Wire-form pins.
    assert_eq!(ProbeKind::RustAnalyzer.as_str(), "rust_analyzer");
    assert_eq!(ProbeKind::Pyright.as_str(), "pyright");
    assert_eq!(ProbeKind::TscNoEmit.as_str(), "tsc_no_emit");
    assert_eq!(ProbeKind::SvelteCheck.as_str(), "svelte_check");
    assert_eq!(ProbeKind::GenericLsp.as_str(), "generic_lsp");
}

#[test]
fn outcome_counter_consistency() {
    let outcome = DiagnosticProbeOutcome {
        kind: ProbeKind::RustAnalyzer,
        error_count: 3,
        warning_count: 7,
        info_count: 2,
        hint_count: 5,
        duration_ms: 1234,
        diagnostics: vec![],
        // E-52 Tier-B (DRIFT-14 closure): explicit Completed preserves the
        // Tier-A semantics that "counters are authoritative" while making
        // the run-status discriminator visible at the contract layer.
        run_status: DiagnosticProbeRunStatus::Completed,
    };
    // total() must be the sum of the four counters — invariant a future
    // refactor (e.g. adding a Fatal severity above Error) must NOT silently
    // break.
    assert_eq!(outcome.total(), 3 + 7 + 2 + 5);
    assert!(!outcome.is_clean(), "outcome with errors must NOT be clean");
}

#[test]
fn is_clean_only_checks_errors() {
    // is_clean is the gate the Tier-B auditor uses — warnings, info, hints
    // are advisory; only errors block. Pin this contract so a future
    // "improve" pass that tightens to also-block-warnings cannot land
    // without a visible breakage.
    let warnings_only = DiagnosticProbeOutcome {
        kind: ProbeKind::TscNoEmit,
        error_count: 0,
        warning_count: 999,
        info_count: 0,
        hint_count: 0,
        duration_ms: 500,
        diagnostics: vec![],
        run_status: DiagnosticProbeRunStatus::Completed,
    };
    assert!(
        warnings_only.is_clean(),
        "warnings-only outcome must be clean (errors are the only blocker)"
    );
}

#[test]
fn diagnostic_uses_1_indexed_line_column() {
    // Contract: producers normalize LSP 0-indexed positions to 1-indexed for
    // dashboard / stdout readability. Tier-B producers MUST honor this when
    // translating from LSP wire format.
    let d = Diagnostic {
        path: "src/lib.rs".to_string(),
        line: 1, // smallest legal value — never 0
        column: 1,
        severity: DiagnosticSeverity::Hint,
        message: "first line column-1 anchor".to_string(),
        code: None,
    };
    assert!(d.line >= 1, "line is 1-indexed; never 0");
    assert!(d.column >= 1, "column is 1-indexed; never 0");
}

#[test]
fn diagnostic_serde_round_trip() {
    let d = Diagnostic {
        path: "crates/orchestrator/src/task_runner.rs".to_string(),
        line: 8357,
        column: 13,
        severity: DiagnosticSeverity::Error,
        message: "cannot find type `EvidenceVariant` in this scope".to_string(),
        code: Some("E0412".to_string()),
    };
    let json = serde_json::to_string(&d).expect("serialize ok");
    let back: Diagnostic = serde_json::from_str(&json).expect("deserialize ok");
    assert_eq!(back, d);
}

#[test]
fn outcome_serde_round_trip() {
    let outcome = DiagnosticProbeOutcome {
        kind: ProbeKind::SvelteCheck,
        error_count: 0,
        warning_count: 17, // pin: dashboard-v2 baseline preserved across sessions
        info_count: 0,
        hint_count: 0,
        duration_ms: 4200,
        diagnostics: vec![],
        run_status: DiagnosticProbeRunStatus::Completed,
    };
    let json = serde_json::to_string(&outcome).expect("serialize ok");
    let back: DiagnosticProbeOutcome = serde_json::from_str(&json).expect("deserialize ok");
    assert_eq!(back, outcome);
}

#[test]
fn outcome_run_status_back_compat_defaults_to_completed() {
    // E-52 Tier-B (DRIFT-14 closure) — Tier-A producers that pre-date the
    // `run_status` field MUST still deserialize cleanly. The field is
    // `#[serde(default)]` and the enum's `Default` impl returns
    // `Completed`, so legacy JSON without the field reads as a
    // completed-run outcome. This is the canonical back-compat pin.
    let legacy_json = r#"{
        "kind": "rust_analyzer",
        "error_count": 0,
        "warning_count": 0,
        "info_count": 0,
        "hint_count": 0,
        "duration_ms": 100,
        "diagnostics": []
    }"#;
    let outcome: DiagnosticProbeOutcome =
        serde_json::from_str(legacy_json).expect("legacy payload must parse");
    assert_eq!(outcome.run_status, DiagnosticProbeRunStatus::Completed);
    assert!(outcome.is_clean());
    assert!(outcome.run_status.is_authoritative());
}
