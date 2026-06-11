//! WAVE-A on-path smoke for the E-48 web-research-orchestrator SSB gate
//! (ELAI_WEB_RESEARCH_ORCHESTRATOR_ENABLE).
//!
//! The gate predicate lives on the orchestrator side
//! (`research::web_orchestrator::is_enabled()`) — every web-orchestrator
//! producer reads it before emitting its SSB row. Default-OFF pinned; ON on
//! the truthy word-set. Also pins the env-const drift contract against the
//! ssb-side const. Live client wiring (crawl4ai / Jina / Brave / Mojeek /
//! Cohere) is RUNPOD-pending under promotion_plan_id E-48.

use std::sync::Mutex;

use elai_orchestrator::research::web_orchestrator::{
    is_enabled, WEB_RESEARCH_ORCHESTRATOR_ENABLE_ENV,
};
use elai_orchestrator::ssb::ELAI_WEB_RESEARCH_ORCHESTRATOR_ENABLE_ENV;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn web_orchestrator_gate_off_by_default() {
    let _g = ENV_LOCK.lock().expect("env lock");
    std::env::remove_var(WEB_RESEARCH_ORCHESTRATOR_ENABLE_ENV);
    assert!(
        !is_enabled(),
        "CONTRACT: {WEB_RESEARCH_ORCHESTRATOR_ENABLE_ENV} MUST default OFF (wire-dark per E-48)"
    );
}

#[test]
fn web_orchestrator_gate_on_when_enabled() {
    let _g = ENV_LOCK.lock().expect("env lock");
    std::env::set_var(WEB_RESEARCH_ORCHESTRATOR_ENABLE_ENV, "1");
    let on = is_enabled();
    std::env::remove_var(WEB_RESEARCH_ORCHESTRATOR_ENABLE_ENV);
    assert!(
        on,
        "CONTRACT: gate=1 MUST open the web-research-orchestrator emit gate"
    );
}

#[test]
fn web_orchestrator_env_const_matches_ssb() {
    assert_eq!(
        WEB_RESEARCH_ORCHESTRATOR_ENABLE_ENV, ELAI_WEB_RESEARCH_ORCHESTRATOR_ENABLE_ENV,
        "CONTRACT: web-orchestrator gate-name MUST match the ssb const verbatim"
    );
}
