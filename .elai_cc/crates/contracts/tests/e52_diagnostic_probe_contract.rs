//! E-52 Tranche B-3 contract tests for the DiagnosticProbeGate +
//! TransitionContext.diagnostic_probe_outcome wiring.
//!
//! Pins:
//!   - None outcome -> Ok (back-compat for callers that haven't threaded the
//!     producer yet).
//!   - Clean Completed outcome -> Ok.
//!   - Error-severity Completed outcome -> Err with the failing file paths
//!     in `missing_artifacts`.
//!   - Error counters with ToolMissing run_status -> Ok (soft-pass per
//!     E-52-decisions.md (c)/(d)).
//!   - Error counters with TimedOut run_status -> Ok (soft-pass).
//!   - Gate is registered for EXECUTING -> AUDITING via `all_gates()`.
//!
//! Plan ref: workflow/sessions/s-22300/exploration/tranche-B3-E52.md S-5.
//! Mirror: crates/contracts/tests/ambiguity_scorer_contract.rs (TransitionContext
//! struct-update + Default::default() builder pattern).

use elai_knowledge::diagnostic::{
    Diagnostic, DiagnosticProbeOutcome, DiagnosticProbeRunStatus, DiagnosticSeverity, ProbeKind,
};
use elai_orchestrator::evidence::{
    all_gates, DiagnosticProbeGate, EvidenceGate, TransitionContext,
};
use elai_orchestrator::fsm::FsmState;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn clean_completed_outcome() -> DiagnosticProbeOutcome {
    DiagnosticProbeOutcome {
        kind: ProbeKind::RustAnalyzer,
        error_count: 0,
        warning_count: 0,
        info_count: 0,
        hint_count: 0,
        duration_ms: 12,
        diagnostics: vec![],
        run_status: DiagnosticProbeRunStatus::Completed,
    }
}

fn error_outcome(run_status: DiagnosticProbeRunStatus) -> DiagnosticProbeOutcome {
    let diag = Diagnostic {
        path: "src/lib.rs".into(),
        line: 42,
        column: 7,
        severity: DiagnosticSeverity::Error,
        message: "expected `;`".into(),
        code: Some("E0001".into()),
    };
    DiagnosticProbeOutcome {
        kind: ProbeKind::RustAnalyzer,
        error_count: 1,
        warning_count: 0,
        info_count: 0,
        hint_count: 0,
        duration_ms: 30,
        diagnostics: vec![diag],
        run_status,
    }
}

// ---------------------------------------------------------------------------
// 1. Back-compat: None outcome -> Ok
// ---------------------------------------------------------------------------

#[test]
fn test_diagnostic_probe_gate_passes_when_outcome_absent() {
    let ctx = TransitionContext {
        diagnostic_probe_outcome: None,
        ..Default::default()
    };
    assert!(
        DiagnosticProbeGate.check(&ctx).is_ok(),
        "None outcome must default-degrade to Ok per Tranche B-3 back-compat"
    );
}

// ---------------------------------------------------------------------------
// 2. Clean Completed outcome -> Ok
// ---------------------------------------------------------------------------

#[test]
fn test_diagnostic_probe_gate_passes_on_ok_clean() {
    let ctx = TransitionContext {
        diagnostic_probe_outcome: Some(clean_completed_outcome()),
        ..Default::default()
    };
    assert!(
        DiagnosticProbeGate.check(&ctx).is_ok(),
        "clean Completed outcome must pass the gate"
    );
}

// ---------------------------------------------------------------------------
// 3. Error-severity Completed outcome -> Err with paths
// ---------------------------------------------------------------------------

#[test]
fn test_diagnostic_probe_gate_fails_on_error_status() {
    let ctx = TransitionContext {
        diagnostic_probe_outcome: Some(error_outcome(DiagnosticProbeRunStatus::Completed)),
        ..Default::default()
    };
    let err = DiagnosticProbeGate.check(&ctx).unwrap_err();
    assert_eq!(err.gate_name, "diagnostic_probe");
    assert!(
        err.missing_artifacts.iter().any(|p| p.contains("src/lib.rs")),
        "expected failing path to appear in missing_artifacts; got {:?}",
        err.missing_artifacts
    );
    assert!(
        err.suggestion.to_lowercase().contains("error"),
        "expected 'error' in suggestion; got {:?}",
        err.suggestion
    );
}

// ---------------------------------------------------------------------------
// 4. ToolMissing run_status -> Ok (soft-pass)
// ---------------------------------------------------------------------------

#[test]
fn test_diagnostic_probe_gate_softpasses_on_tool_missing() {
    let ctx = TransitionContext {
        diagnostic_probe_outcome: Some(error_outcome(DiagnosticProbeRunStatus::ToolMissing)),
        ..Default::default()
    };
    assert!(
        DiagnosticProbeGate.check(&ctx).is_ok(),
        "ToolMissing must soft-pass per E-52-decisions.md (c)/(d), \
         even with non-zero error_count"
    );
}

// ---------------------------------------------------------------------------
// 5. TimedOut run_status -> Ok (soft-pass) — pinned to keep the auditor
//    responsible for honest reporting downstream.
// ---------------------------------------------------------------------------

#[test]
fn test_diagnostic_probe_gate_softpasses_on_timed_out() {
    let ctx = TransitionContext {
        diagnostic_probe_outcome: Some(error_outcome(DiagnosticProbeRunStatus::TimedOut)),
        ..Default::default()
    };
    assert!(
        DiagnosticProbeGate.check(&ctx).is_ok(),
        "TimedOut must soft-pass; auditor reports the partial outcome"
    );
}

// ---------------------------------------------------------------------------
// 6. DiagnosticProbeGate registered for EXECUTING -> AUDITING via all_gates.
// ---------------------------------------------------------------------------

#[test]
fn test_diagnostic_probe_gate_registered_for_executing_to_auditing() {
    let gates = all_gates();
    let found = gates.iter().any(|g| {
        g.name() == "diagnostic_probe"
            && g.transition() == (FsmState::Executing, FsmState::Auditing)
    });
    assert!(
        found,
        "DiagnosticProbeGate must be registered for EXECUTING -> AUDITING \
         (E-52 Tranche B-3 slot 9 in all_gates())"
    );
}
