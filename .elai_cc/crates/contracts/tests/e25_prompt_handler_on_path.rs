//! WAVE-A on-path smoke for the E-25 LLM prompt-handler gate
//! (ELAI_E25_PROMPT_HANDLER_ENABLE).
//!
//! Drives the REAL public gate predicate
//! `elai_orchestrator::architecture::hook_prompt::prompt_handler_enabled()`
//! (default-OFF pinned, ON when env truthy) AND exercises the gated
//! decision-type ON-path: `PromptDecision` round-trips its canonical wire
//! labels, which the SSB `details_json.decision` field and the dashboard
//! filter both consume. Production sidecar invocation is tracked by
//! promotion_plan_id E-25.

use std::sync::Mutex;

use elai_orchestrator::architecture::hook_prompt::{prompt_handler_enabled, PromptDecision};

static ENV_LOCK: Mutex<()> = Mutex::new(());

const GATE: &str = "ELAI_E25_PROMPT_HANDLER_ENABLE";

#[test]
fn prompt_handler_gate_off_by_default() {
    let _g = ENV_LOCK.lock().expect("env lock");
    std::env::remove_var(GATE);
    assert!(
        !prompt_handler_enabled(),
        "CONTRACT: {GATE} MUST default OFF (wire-dark per E-25)"
    );
}

#[test]
fn prompt_handler_gate_on_when_enabled() {
    let _g = ENV_LOCK.lock().expect("env lock");
    std::env::set_var(GATE, "1");
    let on = prompt_handler_enabled();
    std::env::remove_var(GATE);
    assert!(
        on,
        "CONTRACT: {GATE}=1 MUST open the E-25 prompt-handler gate"
    );
}

#[test]
fn prompt_decision_wire_labels_are_canonical() {
    // ON-path observable: the decision the gated handler returns must carry
    // the canonical snake_case wire labels the SSB row + dashboard filter
    // depend on. A rename here would silently break the consumer.
    assert_eq!(PromptDecision::Allow.as_str(), "allow");
    assert_eq!(PromptDecision::Deny("x".to_string()).as_str(), "deny");
    assert_eq!(PromptDecision::Ask("y".to_string()).as_str(), "ask");
    // Allow carries no reason; Deny/Ask carry one.
    assert!(PromptDecision::Allow.reason().is_none());
    assert_eq!(
        PromptDecision::Deny("blocked".to_string()).reason(),
        Some("blocked")
    );
}
