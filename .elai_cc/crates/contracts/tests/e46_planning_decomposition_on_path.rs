//! WAVE-A on-path smoke for the E-46 planning-decomposition SSB gate
//! (ELAI_PLANNING_DECOMPOSITION_ENABLE).
//!
//! Drives the REAL public gate predicate
//! `elai_orchestrator::planning_decomposition::is_enabled()` (default-OFF
//! pinned, ON on the truthy word-set) and pins the env-const drift contract:
//! the module's gate-name MUST equal `ssb::ELAI_PLANNING_DECOMPOSITION_ENABLE_ENV`
//! verbatim, or the producers silently disable with no SSB row to point at.
//! Production producer wiring is tracked by promotion_plan_id E-46.

use std::sync::Mutex;

use elai_orchestrator::planning_decomposition::{
    is_enabled, PLANNING_DECOMPOSITION_ENABLE_ENV,
};
use elai_orchestrator::ssb::ELAI_PLANNING_DECOMPOSITION_ENABLE_ENV;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn planning_decomposition_gate_off_by_default() {
    let _g = ENV_LOCK.lock().expect("env lock");
    std::env::remove_var(PLANNING_DECOMPOSITION_ENABLE_ENV);
    assert!(
        !is_enabled(),
        "CONTRACT: {PLANNING_DECOMPOSITION_ENABLE_ENV} MUST default OFF (wire-dark per E-46)"
    );
}

#[test]
fn planning_decomposition_gate_on_when_enabled() {
    let _g = ENV_LOCK.lock().expect("env lock");
    std::env::set_var(PLANNING_DECOMPOSITION_ENABLE_ENV, "1");
    let on = is_enabled();
    std::env::remove_var(PLANNING_DECOMPOSITION_ENABLE_ENV);
    assert!(
        on,
        "CONTRACT: setting the gate=1 MUST open the planning_decomposition emit gate"
    );
}

#[test]
fn planning_decomposition_env_const_matches_ssb() {
    // Drift guard: the producer-side gate name MUST equal the ssb-side const
    // verbatim or producers go dark with no row to point at.
    assert_eq!(
        PLANNING_DECOMPOSITION_ENABLE_ENV, ELAI_PLANNING_DECOMPOSITION_ENABLE_ENV,
        "CONTRACT: planning_decomposition gate-name MUST match the ssb const verbatim"
    );
}
