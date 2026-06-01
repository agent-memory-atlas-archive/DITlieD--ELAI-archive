//! s-22600 Tranche G G.4 -- DiagnosticProbeGate fire-site contract.
//!
//! Pins the static registry slice the drive_task gate-fire site reads:
//! `gates_for_transition(Executing, Auditing)` returns exactly one gate
//! today (DiagnosticProbeGate, slot 9 in `all_gates()` per Tranche B-3
//! order-stable append). Locking the slice size + name guards against
//! someone appending a second gate without updating the drive_task
//! emission site, which would silently drop telemetry for the new gate.
//!
//! The drive_task gate-fire path itself (task_runner.rs L8457 onward)
//! is integration-test territory (RUNPOD-PENDING for a live FSM-driven
//! end-to-end). This file covers the registry-shape invariant only.

use elai_orchestrator::evidence::{
    gates_for_transition, DiagnosticProbeGate, EvidenceGate, TransitionContext,
};
use elai_orchestrator::fsm::FsmState;

#[test]
fn gates_for_executing_auditing_returns_diagnostic_probe_only() {
    let gates = gates_for_transition(FsmState::Executing, FsmState::Auditing);
    assert_eq!(
        gates.len(),
        1,
        "EXECUTING -> AUDITING must currently have exactly one gate; \
         appending a new gate requires updating the drive_task gate-fire \
         emission site at task_runner.rs L8457"
    );
    assert_eq!(
        gates[0].name(),
        "diagnostic_probe",
        "the sole EXECUTING -> AUDITING gate must be DiagnosticProbeGate"
    );
}

#[test]
fn diagnostic_probe_gate_passes_on_none_outcome() {
    // The drive_task wire is wrapped in `if let Some(outcome) = ...`,
    // so a None outcome means the gate never fires. This test pins the
    // gate's own None-handling (defence-in-depth) so a future caller
    // that wires a non-None context with a None field still passes.
    let ctx = TransitionContext {
        diagnostic_probe_outcome: None,
        ..Default::default()
    };
    assert!(
        DiagnosticProbeGate.check(&ctx).is_ok(),
        "None outcome must Ok-fast-path even when called explicitly"
    );
}
