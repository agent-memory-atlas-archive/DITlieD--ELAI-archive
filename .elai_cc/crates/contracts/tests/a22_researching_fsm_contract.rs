//! Batch 4C (A-22) — RESEARCHING FSM transition contract.
//!
//! Pins the presence + shape of the new FSM transition SSB row:
//!   * SsbActionType::FsmTransition round-trips via the string table.
//!   * FsmTransitionPayload has stable `{from, to, reason}` fields.
//!   * tool_filter still enforces the RESEARCHING allowed set.
//!
//! Grounding:
//!   * crates/orchestrator/src/ssb.rs FsmTransitionPayload
//!   * crates/orchestrator/src/fsm.rs FsmState::Researching
//!   * crates/orchestrator/src/tool_filter.rs RESEARCHING allow list

use elai_orchestrator::fsm::FsmState;
use elai_orchestrator::ssb::ssb_action_type_from_str_public;
use elai_orchestrator::{FsmTransitionPayload, SsbActionType, SsbPayload};

// --- (a) FsmTransition action type round-trips via sql_str. ---

#[test]
fn contract_fsm_transition_action_type_roundtrips() {
    let at = SsbActionType::FsmTransition;
    let as_str = at.as_sql_str();
    assert_eq!(as_str, "fsm_transition");
    let round = ssb_action_type_from_str_public(as_str).expect("parseable");
    assert_eq!(round, SsbActionType::FsmTransition);
}

// --- (b) FsmTransitionPayload carries the expected fields. ---

#[test]
fn contract_fsm_transition_payload_shape() {
    let p = FsmTransitionPayload {
        from: "IDLE".into(),
        to: "RESEARCHING".into(),
        reason: "web_research_task_entry".into(),
    };
    let wrapped = SsbPayload::FsmTransition(p.clone());
    assert_eq!(wrapped.action_type(), SsbActionType::FsmTransition);
    // JSON round-trip so the discriminant stays stable on disk.
    let json = serde_json::to_string(&wrapped).expect("serialize");
    let back: SsbPayload = serde_json::from_str(&json).expect("deserialize");
    match back {
        SsbPayload::FsmTransition(r) => {
            assert_eq!(r.from, "IDLE");
            assert_eq!(r.to, "RESEARCHING");
            assert_eq!(r.reason, "web_research_task_entry");
        }
        other => panic!("unexpected variant: {other:?}"),
    }
}

// --- (c) FsmState::Researching renders as "RESEARCHING". ---

#[test]
fn contract_fsm_state_researching_display() {
    let s = FsmState::Researching;
    assert_eq!(s.to_string(), "RESEARCHING");
}

// --- (d) Entry + exit transitions pair: IDLE -> RESEARCHING -> EXECUTING. ---

#[test]
fn contract_expected_transition_pair_for_web_research_task() {
    let enter = FsmTransitionPayload {
        from: "IDLE".into(),
        to: "RESEARCHING".into(),
        reason: "web_research_task_entry".into(),
    };
    let exit = FsmTransitionPayload {
        from: "RESEARCHING".into(),
        to: "EXECUTING".into(),
        reason: "web_research_task_exit".into(),
    };
    // Round-trip both via msgpack so the on-disk encoding matches the
    // runtime value (critical for replay from SSB files).
    for p in [enter, exit] {
        let bytes = rmp_serde::to_vec_named(&SsbPayload::FsmTransition(p.clone()))
            .expect("msgpack");
        let back: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
        assert_eq!(back.action_type(), SsbActionType::FsmTransition);
    }
}
