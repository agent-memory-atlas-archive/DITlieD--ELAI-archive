//! Batch-fu (FU.2 / A-7) — constitutional CONSTRAINTS injection contract.
//!
//! Verifies:
//!   * `ConstraintsInjected` is registered on the `SsbActionType` enum with
//!     the expected snake-case name `constraints_injected`.
//!   * `ConstraintsInjectedPayload` round-trips via the SsbPayload enum.
//!   * Populated `ProjectProfile.constraints` becomes a rendered CONSTRAINTS
//!     block in the assembled `DispatchContext.phase_content`.
//!   * Empty constraints yields no CONSTRAINTS block (non-injection is the
//!     pre-batch-fu default).

use std::collections::BTreeMap;

use elai_orchestrator::{
    build_dispatch_context, BackendConfig, BackendMode, ConstraintsInjectedPayload, PhaseSlot,
    SsbActionType, SsbPayload, CONSTRAINTS_BLOCK_HEADER,
};
use elai_orchestrator::context_budget::GenerationProfile;
use elai_orchestrator::fsm::FsmState;

fn cfg() -> BackendConfig {
    BackendConfig {
        mode: BackendMode::Local,
        sidecar_url: "http://localhost:8100".into(),
        model: "test".into(),
        max_tools_local: 4,
        fallback_url: None,
        max_context: None,
    }
}

#[test]
fn contract_constraints_injected_is_a_registered_action_type() {
    assert_eq!(
        SsbActionType::ConstraintsInjected.as_sql_str(),
        "constraints_injected"
    );
}

#[test]
fn contract_constraints_injected_payload_roundtrips() {
    let p = ConstraintsInjectedPayload {
        project_id: "proj-1".into(),
        constraint_count: 3,
        step_id: "task_42".into(),
    };
    let wrapped = SsbPayload::ConstraintsInjected(p.clone());
    let bytes = rmp_serde::to_vec_named(&wrapped).expect("serialize");
    let round: SsbPayload = rmp_serde::from_slice(&bytes).expect("deserialize");
    match round {
        SsbPayload::ConstraintsInjected(q) => assert_eq!(q, p),
        other => panic!("expected ConstraintsInjected, got {other:?}"),
    }
}

#[test]
fn contract_constraints_block_rendered_in_phase_content() {
    let c = cfg();
    let prio: BTreeMap<String, u8> = BTreeMap::new();
    let tools: Vec<String> = vec!["read_file".into()];
    let slot = PhaseSlot {
        goal: String::new(),
        constraints: vec!["no blake2".into(), "no rand<=0.8".into()],
        profile_hint: Some(GenerationProfile::FirstAttempt),
        repair_snippet: None,
    };
    let ctx =
        build_dispatch_context(FsmState::Executing, &c, &tools, &prio, None, Some(&slot))
            .expect("dispatch");
    let body = ctx.phase_content.expect("phase_content set");
    assert!(body.contains(CONSTRAINTS_BLOCK_HEADER));
    assert!(body.contains("- no blake2"));
    assert!(body.contains("- no rand<=0.8"));
}

#[test]
fn contract_empty_constraints_produce_no_block() {
    let c = cfg();
    let prio: BTreeMap<String, u8> = BTreeMap::new();
    let tools: Vec<String> = vec!["read_file".into()];
    let slot = PhaseSlot {
        goal: "goal only".into(),
        constraints: Vec::new(),
        profile_hint: None,
        repair_snippet: None,
    };
    let ctx =
        build_dispatch_context(FsmState::Executing, &c, &tools, &prio, None, Some(&slot))
            .expect("dispatch");
    let body = ctx.phase_content.expect("phase_content set");
    assert!(
        !body.contains(CONSTRAINTS_BLOCK_HEADER),
        "empty constraints must not inject a CONSTRAINTS block, got: {body}",
    );
}
