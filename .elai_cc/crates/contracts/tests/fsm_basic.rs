//! Contract: elai_cc orchestrator must expose an Fsm that can
//! transition IDLE -> EXECUTING -> IDLE with persistent state.
//!
//! RED phase — compiles against existing orchestrator types, fails
//! at runtime because no Fsm type or state machine exists yet.

use elai_tools::GlobalToolRegistry;
use serde_json::json;

/// The orchestrator must expose an FSM with transition/get_state.
/// We verify state persistence: after transitioning to EXECUTING,
/// get_state must return EXECUTING (not hardcoded IDLE).
#[test]
fn fsm_can_transition_idle_executing_idle() {
    let registry = GlobalToolRegistry::builtin();

    // CONTRACT: fsm_get_state must exist and return current state.
    let result = registry.execute("fsm_get_state", &json!({}));
    assert!(
        result.is_ok(),
        "CONTRACT: fsm_get_state tool must be registered — got: {:?}",
        result.err()
    );
    let state_json: serde_json::Value =
        serde_json::from_str(&result.unwrap()).expect("fsm_get_state must return valid JSON");
    assert_eq!(
        state_json["state"].as_str().unwrap_or(""),
        "IDLE",
        "CONTRACT: initial FSM state must be IDLE"
    );

    // CONTRACT: fsm_transition must transition to EXECUTING.
    let transition_result = registry.execute(
        "fsm_transition",
        &json!({"target": "EXECUTING"}),
    );
    assert!(
        transition_result.is_ok(),
        "CONTRACT: fsm_transition tool must be registered — got: {:?}",
        transition_result.err()
    );
    let tr_json: serde_json::Value =
        serde_json::from_str(&transition_result.unwrap()).expect("fsm_transition must return valid JSON");
    assert_eq!(
        tr_json["previous"].as_str().unwrap_or(""),
        "IDLE",
        "CONTRACT: previous state must be IDLE before first transition"
    );

    // CONTRACT: state must persist — get_state after transition must show EXECUTING.
    let after = registry.execute("fsm_get_state", &json!({}));
    let after_json: serde_json::Value =
        serde_json::from_str(&after.unwrap()).expect("fsm_get_state must return valid JSON");
    assert_eq!(
        after_json["state"].as_str().unwrap_or(""),
        "EXECUTING",
        "CONTRACT: FSM state must persist — expected EXECUTING after transition"
    );

    // Transition back to IDLE.
    let back = registry.execute("fsm_transition", &json!({"target": "IDLE"}));
    assert!(back.is_ok(), "CONTRACT: transition back to IDLE must succeed");
    let back_json: serde_json::Value =
        serde_json::from_str(&back.unwrap()).expect("valid JSON");
    assert_eq!(
        back_json["previous"].as_str().unwrap_or(""),
        "EXECUTING",
        "CONTRACT: previous state must be EXECUTING when transitioning back"
    );
}
