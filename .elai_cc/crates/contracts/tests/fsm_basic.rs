//! Contract: elai_cc orchestrator must expose an Fsm that can
//! transition IDLE -> EXECUTING -> IDLE.
//!
//! RED phase — compiles against existing orchestrator types, fails
//! at runtime because no Fsm type or state machine exists yet.

use elai_tools::GlobalToolRegistry;
use serde_json::json;

/// The orchestrator must expose an FSM with transition/get_state.
/// We verify by calling execute("fsm_get_state", ...) on the
/// builtin tool registry. No such tool exists, so execute returns
/// Err — and we assert it returns Ok, causing a runtime failure.
#[test]
fn fsm_can_transition_idle_executing_idle() {
    let registry = GlobalToolRegistry::builtin();

    // CONTRACT: an "fsm_get_state" tool must exist and return
    // the current FSM state as JSON. It does not exist yet,
    // so execute() returns Err("unsupported tool: fsm_get_state").
    let result = registry.execute("fsm_get_state", &json!({}));
    assert!(
        result.is_ok(),
        "CONTRACT: fsm_get_state tool must be registered — got: {:?}",
        result.err()
    );

    // CONTRACT: an "fsm_transition" tool must exist.
    let transition_result = registry.execute(
        "fsm_transition",
        &json!({"target": "EXECUTING"}),
    );
    assert!(
        transition_result.is_ok(),
        "CONTRACT: fsm_transition tool must be registered — got: {:?}",
        transition_result.err()
    );
}
