//! DST reference model integration tests — TDD red phase (3I.4).
//!
//! The prop_state_machine! test lives in src/dst.rs (unit test) because
//! proptest trait impls need crate-local scope. These integration tests
//! exercise the inherent methods (preconditions, apply, check_invariants)
//! which are available to external consumers.

use std::collections::HashSet;

use elai_orchestrator::agent_pool::AgentRole;
use elai_orchestrator::dst::{FsmEvent, FsmReferenceModel, ReferenceState, SimulatorRuntime};
use elai_orchestrator::fsm::FsmState;
use elai_orchestrator::ssb::SsbActionType;

// ---------------------------------------------------------------------------
// Test 1: Determinism seed-replay test
// ---------------------------------------------------------------------------

#[test]
fn determinism_seed_replay() {
    let seed: u64 = 42;

    let events = vec![
        FsmEvent::PhaseTransition {
            from: FsmState::Idle,
            to: FsmState::Planning,
        },
        FsmEvent::SsbWrite {
            action_type: SsbActionType::PlanUpdate,
        },
        FsmEvent::PhaseTransition {
            from: FsmState::Planning,
            to: FsmState::TestingFirst,
        },
        FsmEvent::PhaseTransition {
            from: FsmState::TestingFirst,
            to: FsmState::Executing,
        },
        FsmEvent::AgentSpawn {
            role: AgentRole::Executor,
        },
        FsmEvent::SsbWrite {
            action_type: SsbActionType::FileWrite,
        },
        FsmEvent::SuperstepBoundary,
        FsmEvent::PhaseTransition {
            from: FsmState::Executing,
            to: FsmState::Auditing,
        },
        FsmEvent::AgentSpawn {
            role: AgentRole::Auditor,
        },
        FsmEvent::PhaseTransition {
            from: FsmState::Auditing,
            to: FsmState::Memorizing,
        },
    ];

    // Run 1
    let mut state_a = ReferenceState::initial();
    let _runtime_a = SimulatorRuntime::new(seed);
    for event in &events {
        if FsmReferenceModel::preconditions(&state_a, event) {
            state_a = FsmReferenceModel::apply(state_a, event);
        }
    }

    // Run 2 — identical seed, identical events
    let mut state_b = ReferenceState::initial();
    let _runtime_b = SimulatorRuntime::new(seed);
    for event in &events {
        if FsmReferenceModel::preconditions(&state_b, event) {
            state_b = FsmReferenceModel::apply(state_b, event);
        }
    }

    assert_eq!(
        state_a, state_b,
        "Determinism violation: same seed + events produced different states"
    );
    assert_eq!(
        state_a.fsm_state,
        FsmState::Memorizing,
        "Expected final state Memorizing after event sequence"
    );
    assert!(
        state_a.ssb_row_count >= 2,
        "Expected at least 2 SSB writes"
    );
}

// ---------------------------------------------------------------------------
// Test 2: Synthetic regression — inject invariant violation
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "FSM state mismatch")]
fn synthetic_regression_invariant_violation() {
    let ref_state = ReferenceState::initial();

    // Mutate reference state WITHOUT applying to SUT — deliberate divergence.
    let mutated_ref = ReferenceState {
        fsm_state: FsmState::Executing,
        ..ref_state
    };

    // SUT stays Idle, ref says Executing — must panic.
    FsmReferenceModel::check_invariants(FsmState::Idle, 0, &HashSet::new(), &mutated_ref);
}

// ---------------------------------------------------------------------------
// Test 3: All 9 FsmState variants covered in valid_targets
// ---------------------------------------------------------------------------

#[test]
fn all_nine_fsm_states_have_transitions() {
    let all_states = FsmState::ALL;
    assert_eq!(all_states.len(), 9, "Expected 9 FSM states");

    for state in &all_states {
        let targets = state.valid_targets();
        assert!(
            !targets.is_empty(),
            "FsmState::{:?} has no valid transitions",
            state
        );
    }
}

// ---------------------------------------------------------------------------
// Test 4: Preconditions reject invalid transitions
// ---------------------------------------------------------------------------

#[test]
fn preconditions_reject_invalid_transition() {
    let state = ReferenceState::initial(); // Idle

    // Idle -> Executing is not a valid transition
    let invalid = FsmEvent::PhaseTransition {
        from: FsmState::Idle,
        to: FsmState::Executing,
    };
    assert!(
        !FsmReferenceModel::preconditions(&state, &invalid),
        "Should reject Idle -> Executing"
    );

    // ToolCall in Idle should be rejected
    let tool_in_idle = FsmEvent::ToolCall {
        tool_name: "read".into(),
    };
    assert!(
        !FsmReferenceModel::preconditions(&state, &tool_in_idle),
        "Should reject ToolCall in Idle"
    );

    // AgentCrash with no active agents should be rejected
    let crash_none = FsmEvent::AgentCrash {
        role: AgentRole::Executor,
    };
    assert!(
        !FsmReferenceModel::preconditions(&state, &crash_none),
        "Should reject AgentCrash when no agents active"
    );
}
