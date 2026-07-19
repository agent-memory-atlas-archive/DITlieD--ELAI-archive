//! J-18 leg-3 on-path smoke for the agentic-grep repair gate
//! (ELAI_AGENTIC_GREP_REPAIR_ENABLE).
//!
//! Drives the REAL public gate predicate
//! `elai_knowledge::agentic_grep::agentic_grep_repair_enabled_from_env()`
//! (default-OFF pinned, ON on the truthy word-set). The gate decides whether
//! the post-token_fit pipeline stage fuses agentic-grep confirm hits; it is
//! the off-agent observable the repair path reads. Production wiring is
//! tracked by promotion_plan_id J-18; the gate-predicate smoke is the R9
//! observable for this flag (same shape as E-40's 5th-RRF gate smoke).

use std::sync::Mutex;

use elai_knowledge::agentic_grep::{
    agentic_grep_repair_enabled_from_env, ELAI_AGENTIC_GREP_REPAIR_ENABLE_ENV,
};

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn agentic_grep_repair_gate_off_by_default() {
    let _g = ENV_LOCK.lock().expect("env lock");
    std::env::remove_var(ELAI_AGENTIC_GREP_REPAIR_ENABLE_ENV);
    assert!(
        !agentic_grep_repair_enabled_from_env(),
        "CONTRACT: {ELAI_AGENTIC_GREP_REPAIR_ENABLE_ENV} MUST default OFF (wire-dark per J-18)"
    );
}

#[test]
fn agentic_grep_repair_gate_on_when_enabled() {
    let _g = ENV_LOCK.lock().expect("env lock");
    std::env::set_var(ELAI_AGENTIC_GREP_REPAIR_ENABLE_ENV, "1");
    let on = agentic_grep_repair_enabled_from_env();
    std::env::remove_var(ELAI_AGENTIC_GREP_REPAIR_ENABLE_ENV);
    assert!(
        on,
        "CONTRACT: gate=1 MUST enable the agentic-grep repair confirm pass"
    );
}

#[test]
fn agentic_grep_repair_gate_accepts_truthy_word() {
    let _g = ENV_LOCK.lock().expect("env lock");
    std::env::set_var(ELAI_AGENTIC_GREP_REPAIR_ENABLE_ENV, "on");
    let on = agentic_grep_repair_enabled_from_env();
    std::env::remove_var(ELAI_AGENTIC_GREP_REPAIR_ENABLE_ENV);
    assert!(on, "CONTRACT: the word 'on' MUST be treated as enabled");
}
