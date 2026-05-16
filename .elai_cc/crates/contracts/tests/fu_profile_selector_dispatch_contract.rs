//! Batch-fu (FU.3 / A-18) — GenerationProfile selector dispatch contract.
//!
//! Verifies:
//!   * `DefaultGenerationProfileSelector` returns `RepairRetry` for the
//!     `DispatchRole::Repair` role (the canonical BubbleUp retry path).
//!   * `build_dispatch_context` records the selector output on the
//!     `DispatchContext.profile` when the selector's choice is threaded via
//!     `PhaseSlot.profile_hint`.
//!   * The `ProfileSelected` SSB row is registered and round-trips.

use std::collections::BTreeMap;

use elai_orchestrator::{
    build_dispatch_context, BackendConfig, BackendMode, DefaultGenerationProfileSelector,
    DispatchRole, GenerationProfileSelector, PhaseSlot, ProfileSelectedPayload, SsbActionType,
    SsbPayload,
};
use elai_orchestrator::context_budget::GenerationProfile;
use elai_orchestrator::fsm::FsmState;

#[test]
fn contract_default_selector_maps_repair_role_to_repair_retry() {
    let s = DefaultGenerationProfileSelector;
    assert_eq!(
        s.select(DispatchRole::Repair, FsmState::Executing),
        GenerationProfile::RepairRetry,
    );
}

#[test]
fn contract_dispatch_context_records_profile_from_phase_slot() {
    let cfg = BackendConfig {
        mode: BackendMode::Local,
        sidecar_url: "http://localhost:8100".into(),
        model: "t".into(),
        max_tools_local: 4,
        fallback_url: None,
    };
    let prio: BTreeMap<String, u8> = BTreeMap::new();
    let tools: Vec<String> = vec!["read_file".into()];
    let selector = DefaultGenerationProfileSelector;
    let profile = selector.select(DispatchRole::Repair, FsmState::Executing);
    let slot = PhaseSlot {
        profile_hint: Some(profile),
        ..PhaseSlot::default()
    };
    let ctx =
        build_dispatch_context(FsmState::Executing, &cfg, &tools, &prio, None, Some(&slot))
            .expect("dispatch");
    assert_eq!(ctx.profile, GenerationProfile::RepairRetry);
}

#[test]
fn contract_profile_selected_is_a_registered_action_type() {
    assert_eq!(
        SsbActionType::ProfileSelected.as_sql_str(),
        "profile_selected",
    );
}

#[test]
fn contract_profile_selected_payload_roundtrips() {
    let p = ProfileSelectedPayload {
        profile: "RepairRetry".into(),
        role: "repair".into(),
        fsm_state: "EXECUTING".into(),
        step_id: "task_7".into(),
    };
    let wrapped = SsbPayload::ProfileSelected(p.clone());
    let bytes = rmp_serde::to_vec_named(&wrapped).expect("serialize");
    let round: SsbPayload = rmp_serde::from_slice(&bytes).expect("deserialize");
    assert!(matches!(round, SsbPayload::ProfileSelected(q) if q == p));
}
