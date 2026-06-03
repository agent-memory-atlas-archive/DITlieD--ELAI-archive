//! s-22600 Tranche G G.4 -- EXECUTING -> AUDITING gate fire-site contract.
//!
//! Pins the static registry slice the drive_task gate-fire site reads:
//! `gates_for_transition(Executing, Auditing)`. After the SP5 wiring-fix
//! the slice carries THREE order-stable gates (slots 4 / 9 / 11 in
//! `all_gates()`):
//!   - `test_flipped`         (TestFlippedGate)
//!   - `diagnostic_probe`     (DiagnosticProbeGate, Tranche B-3 append)
//!   - `entrypoint_and_trace` (EntrypointAndTraceGate, SP5 wiring-fix append)
//! Locking the gate NAME SET (rather than a brittle exact-count-of-1) guards
//! against someone dropping a gate or appending a fourth without updating the
//! drive_task emission site, which would silently drop telemetry for the
//! changed gate.
//!
//! The drive_task gate-fire path itself (task_runner.rs L8457 onward)
//! is integration-test territory (RUNPOD-PENDING for a live FSM-driven
//! end-to-end). This file covers the registry-shape invariant only.

use elai_orchestrator::evidence::{
    gates_for_transition, DiagnosticProbeGate, EvidenceGate, TransitionContext,
};
use elai_orchestrator::fsm::FsmState;

#[test]
fn gates_for_executing_auditing_includes_entrypoint_and_trace() {
    let gates = gates_for_transition(FsmState::Executing, FsmState::Auditing);
    let names: Vec<&str> = gates.iter().map(|g| g.name()).collect();
    assert_eq!(
        gates.len(),
        3,
        "EXECUTING -> AUDITING currently has exactly three gates \
         (test_flipped + diagnostic_probe + entrypoint_and_trace); \
         changing the gate set requires updating the drive_task gate-fire \
         emission site at task_runner.rs L8457. Got: {names:?}"
    );
    for expected in ["test_flipped", "diagnostic_probe", "entrypoint_and_trace"] {
        assert!(
            names.contains(&expected),
            "EXECUTING -> AUDITING gate set must contain {expected:?}; got {names:?}"
        );
    }
}

#[test]
fn diagnostic_probe_gate_fails_on_none_outcome() {
    // SP5 wiring-fix: a None outcome means the LSP probe never ran. R1/R2
    // fail-closed: absent probe evidence is a hard FAIL, not a soft pass.
    // This pins the gate's own None-handling so a caller that wires a
    // non-None context with a None probe field is BLOCKED, not waved through.
    let ctx = TransitionContext {
        diagnostic_probe_outcome: None,
        ..Default::default()
    };
    let err = DiagnosticProbeGate.check(&ctx).unwrap_err();
    assert_eq!(err.gate_name, "diagnostic_probe");
    assert!(
        err.missing_artifacts
            .iter()
            .any(|a| a == "diagnostic_probe_outcome"),
        "None outcome must hard-FAIL (R1/R2); got {:?}",
        err.missing_artifacts
    );
}
