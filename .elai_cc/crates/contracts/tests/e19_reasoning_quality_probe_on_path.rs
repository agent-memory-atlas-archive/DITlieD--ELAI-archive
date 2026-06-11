//! WAVE-A on-path smoke for the E-19 reasoning-quality umbrella gate
//! (ELAI_REASONING_QUALITY_PROBE_ENABLE).
//!
//! Drives the REAL public gate predicate
//! `elai_orchestrator::reasoning::umbrella_enabled()` with the env var set,
//! and asserts the gate flips ON (the off-agent observable: the gate-read
//! reflects the operator opt-in). Default-OFF is also pinned. This is the
//! registered-experiment smoke the wire-dark-allowlist row points at; live
//! production wiring of the dispatch paths is tracked by promotion_plan_id
//! E-19.

use std::sync::Mutex;

use elai_orchestrator::reasoning::umbrella_enabled;

// Cargo runs the tests in one binary in parallel; the env-mutating tests
// below race on the SAME process-global var. Serialize them.
static ENV_LOCK: Mutex<()> = Mutex::new(());

const GATE: &str = "ELAI_REASONING_QUALITY_PROBE_ENABLE";

#[test]
fn umbrella_gate_off_by_default() {
    let _g = ENV_LOCK.lock().expect("env lock");
    std::env::remove_var(GATE);
    assert!(
        !umbrella_enabled(),
        "CONTRACT: {GATE} MUST default OFF (wire-dark per E-19)"
    );
}

#[test]
fn umbrella_gate_on_when_enabled() {
    let _g = ENV_LOCK.lock().expect("env lock");
    std::env::set_var(GATE, "1");
    let on = umbrella_enabled();
    std::env::remove_var(GATE);
    assert!(
        on,
        "CONTRACT: {GATE}=1 MUST open the E-19 reasoning-quality umbrella gate"
    );
}

#[test]
fn umbrella_gate_accepts_truthy_word() {
    let _g = ENV_LOCK.lock().expect("env lock");
    std::env::set_var(GATE, "true");
    let on = umbrella_enabled();
    std::env::remove_var(GATE);
    assert!(on, "CONTRACT: {GATE}=true MUST be treated as ON");
}
