//! WAVE-A on-path smoke for the E-50 SaaS-arch-patterns SSB gate
//! (ELAI_SAAS_ARCH_PATTERNS_ENABLE).
//!
//! Drives the REAL public gate predicate
//! `elai_orchestrator::architecture::saas_patterns::ssb_emission_enabled()`
//! (default-OFF pinned, ON on strict "1"). The gate is the off-agent
//! observable every SaasArchPatternEvent producer reads before emitting.
//! Production configure_agents / pipeline_builder wiring is tracked by
//! promotion_plan_id E-50.

use std::sync::Mutex;

use elai_orchestrator::architecture::saas_patterns::{
    ssb_emission_enabled, ELAI_SAAS_ARCH_PATTERNS_ENABLE_VAR,
};

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn saas_patterns_gate_off_by_default() {
    let _g = ENV_LOCK.lock().expect("env lock");
    std::env::remove_var(ELAI_SAAS_ARCH_PATTERNS_ENABLE_VAR);
    assert!(
        !ssb_emission_enabled(),
        "CONTRACT: {ELAI_SAAS_ARCH_PATTERNS_ENABLE_VAR} MUST default OFF (wire-dark per E-50)"
    );
}

#[test]
fn saas_patterns_gate_on_when_enabled() {
    let _g = ENV_LOCK.lock().expect("env lock");
    std::env::set_var(ELAI_SAAS_ARCH_PATTERNS_ENABLE_VAR, "1");
    let on = ssb_emission_enabled();
    std::env::remove_var(ELAI_SAAS_ARCH_PATTERNS_ENABLE_VAR);
    assert!(
        on,
        "CONTRACT: gate=1 MUST open the SaasArchPatternEvent emission gate"
    );
}

#[test]
fn saas_patterns_gate_strict_one_only() {
    let _g = ENV_LOCK.lock().expect("env lock");
    std::env::set_var(ELAI_SAAS_ARCH_PATTERNS_ENABLE_VAR, "0");
    let off = ssb_emission_enabled();
    std::env::remove_var(ELAI_SAAS_ARCH_PATTERNS_ENABLE_VAR);
    assert!(
        !off,
        "CONTRACT: gate is strict == \"1\"; the value \"0\" MUST keep emission dark"
    );
}
