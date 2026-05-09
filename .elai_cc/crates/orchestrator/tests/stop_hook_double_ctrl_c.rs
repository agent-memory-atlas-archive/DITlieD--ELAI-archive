//! B-19A T4 — Stop hook: double-Ctrl+C-within-3s gate.
//!
//! The Ctrl+C handler used to call `abort_signal.abort()` unconditionally on
//! the first signal. After B-19A:
//!   * If FSM == Idle: the first Ctrl+C aborts immediately (legacy behaviour).
//!   * If FSM != Idle: the first Ctrl+C records the timestamp + writes a
//!     warn-tier intervention prompt to the user (does NOT abort). A second
//!     Ctrl+C delivered within 3 seconds triggers the abort. A second
//!     Ctrl+C delivered after the 3 second window resets the gate (the
//!     user effectively cancelled their first attempt).
//!
//! These tests exercise the decision gate in
//! `elai_orchestrator::hooks::StopHookGate` directly so the cross-thread
//! signal-handler runtime in cli/main.rs is tested by composition.

use elai_orchestrator::hooks::{StopHookDecision, StopHookGate};
use elai_orchestrator::FsmState;

#[test]
fn idle_state_aborts_on_first_signal() {
    let gate = StopHookGate::new();
    let now_ms = 1_000;
    let decision = gate.observe_signal(FsmState::Idle, now_ms);
    assert_eq!(
        decision,
        StopHookDecision::Abort,
        "Ctrl+C in IDLE must abort immediately"
    );
    assert!(gate.is_aborted());
}

#[test]
fn executing_state_warns_on_first_signal_then_aborts_within_window() {
    let gate = StopHookGate::new();
    let t0 = 1_000_000_i64;
    let first = gate.observe_signal(FsmState::Executing, t0);
    assert_eq!(
        first,
        StopHookDecision::WarnAndRecord,
        "first Ctrl+C in non-IDLE state warns, does NOT abort"
    );
    assert!(!gate.is_aborted(), "abort flag must NOT be set after first press");

    // Second press 2 seconds later — within the 3s window.
    let second = gate.observe_signal(FsmState::Executing, t0 + 2_000);
    assert_eq!(
        second,
        StopHookDecision::Abort,
        "second Ctrl+C within 3s must abort"
    );
    assert!(gate.is_aborted());
}

#[test]
fn second_ctrl_c_after_3s_resets_window() {
    let gate = StopHookGate::new();
    let t0 = 1_000_000_i64;
    let first = gate.observe_signal(FsmState::Executing, t0);
    assert_eq!(first, StopHookDecision::WarnAndRecord);

    // Second press 5 seconds later — outside the 3s window.
    // This should restart the warn cycle (a new "first press"), NOT abort.
    let second = gate.observe_signal(FsmState::Executing, t0 + 5_000);
    assert_eq!(
        second,
        StopHookDecision::WarnAndRecord,
        "second Ctrl+C after 3s window must restart the warn cycle"
    );
    assert!(!gate.is_aborted());

    // Third press 1s after the second — within the new window.
    let third = gate.observe_signal(FsmState::Executing, t0 + 6_000);
    assert_eq!(third, StopHookDecision::Abort);
    assert!(gate.is_aborted());
}

#[test]
fn testing_first_state_also_warns_then_aborts() {
    let gate = StopHookGate::new();
    let t0 = 50_000_i64;
    assert_eq!(
        gate.observe_signal(FsmState::TestingFirst, t0),
        StopHookDecision::WarnAndRecord
    );
    assert_eq!(
        gate.observe_signal(FsmState::TestingFirst, t0 + 1_000),
        StopHookDecision::Abort
    );
}

#[test]
fn auditing_state_also_warns_then_aborts() {
    let gate = StopHookGate::new();
    let t0 = 50_000_i64;
    assert_eq!(
        gate.observe_signal(FsmState::Auditing, t0),
        StopHookDecision::WarnAndRecord
    );
    assert_eq!(
        gate.observe_signal(FsmState::Auditing, t0 + 100),
        StopHookDecision::Abort
    );
}

#[test]
fn aborted_gate_stays_aborted_on_subsequent_signals() {
    let gate = StopHookGate::new();
    let t0 = 0_i64;
    gate.observe_signal(FsmState::Idle, t0);
    assert!(gate.is_aborted());
    // Subsequent presses are idempotent — gate stays aborted.
    let next = gate.observe_signal(FsmState::Executing, t0 + 100_000);
    assert_eq!(next, StopHookDecision::Abort);
    assert!(gate.is_aborted());
}
