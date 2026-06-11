//! WAVE-A on-path smoke for the E-40 5th-list symbol-graph RRF gate
//! (ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE).
//!
//! Drives the REAL public gate predicate
//! `elai_knowledge::pipeline::symbol_graph_5th_rrf_enabled_from_env()`
//! (default-OFF pinned, ON on the truthy word-set). The gate decides whether
//! the pipeline fuses the 5th symbol_graph_vec RRF input; it is the off-agent
//! observable the fusion path reads. Production fusion wiring is tracked by
//! promotion_plan_id E-40.

use std::sync::Mutex;

use elai_knowledge::pipeline::{
    symbol_graph_5th_rrf_enabled_from_env, ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE_ENV,
};

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn symbol_graph_5th_rrf_gate_off_by_default() {
    let _g = ENV_LOCK.lock().expect("env lock");
    std::env::remove_var(ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE_ENV);
    assert!(
        !symbol_graph_5th_rrf_enabled_from_env(),
        "CONTRACT: {ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE_ENV} MUST default OFF (wire-dark per E-40)"
    );
}

#[test]
fn symbol_graph_5th_rrf_gate_on_when_enabled() {
    let _g = ENV_LOCK.lock().expect("env lock");
    std::env::set_var(ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE_ENV, "1");
    let on = symbol_graph_5th_rrf_enabled_from_env();
    std::env::remove_var(ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE_ENV);
    assert!(
        on,
        "CONTRACT: gate=1 MUST enable the 5th symbol_graph_vec RRF input"
    );
}

#[test]
fn symbol_graph_5th_rrf_gate_accepts_truthy_word() {
    let _g = ENV_LOCK.lock().expect("env lock");
    std::env::set_var(ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE_ENV, "on");
    let on = symbol_graph_5th_rrf_enabled_from_env();
    std::env::remove_var(ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE_ENV);
    assert!(on, "CONTRACT: the word 'on' MUST be treated as enabled");
}
