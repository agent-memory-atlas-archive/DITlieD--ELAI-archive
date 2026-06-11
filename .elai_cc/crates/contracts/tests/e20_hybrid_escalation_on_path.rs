//! WAVE-A on-path smoke for the E-20 local/cloud hybrid-routing SSB gate
//! (ELAI_LOCAL_CLOUD_ROUTING_ENABLE).
//!
//! Drives the REAL single-source-of-truth gate predicate
//! `elai_orchestrator::cost::cloud_route::local_cloud_routing_enabled()` —
//! the same gate every LocalCloudRoutingEvent producer reads before emitting
//! its SSB row. Default-OFF pinned; ON asserted when the env var is set.
//!
//! The HybridEscalationGate / cost-router production caller is wire-dark
//! (no task_runner caller yet); promotion_plan_id E-20 tracks that wire. The
//! gate predicate is the off-agent observable this registered-experiment
//! smoke asserts.

use std::sync::Mutex;

use elai_orchestrator::cost::cloud_route::local_cloud_routing_enabled;

static ENV_LOCK: Mutex<()> = Mutex::new(());

const GATE: &str = "ELAI_LOCAL_CLOUD_ROUTING_ENABLE";

#[test]
fn local_cloud_routing_gate_off_by_default() {
    let _g = ENV_LOCK.lock().expect("env lock");
    std::env::remove_var(GATE);
    assert!(
        !local_cloud_routing_enabled(),
        "CONTRACT: {GATE} MUST default OFF (wire-dark per E-20)"
    );
}

#[test]
fn local_cloud_routing_gate_on_when_enabled() {
    let _g = ENV_LOCK.lock().expect("env lock");
    std::env::set_var(GATE, "1");
    let on = local_cloud_routing_enabled();
    std::env::remove_var(GATE);
    assert!(
        on,
        "CONTRACT: {GATE}=1 MUST open the LocalCloudRoutingEvent emission gate"
    );
}

#[test]
fn local_cloud_routing_gate_strict_one_only() {
    // The gate is strict `== "1"` (NOT the word-set some siblings accept).
    // Pin that so a producer reading this gate agrees with the predicate.
    let _g = ENV_LOCK.lock().expect("env lock");
    std::env::set_var(GATE, "true");
    let on_word = local_cloud_routing_enabled();
    std::env::remove_var(GATE);
    assert!(
        !on_word,
        "CONTRACT: {GATE} is strict == \"1\"; the word 'true' MUST stay OFF"
    );
}
