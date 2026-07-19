//! Batch-fu (FU.1) — `build_dispatch_context` phase-slot contract.
//!
//! Pins the new `PhaseSlot` surface added to `build_dispatch_context`. The
//! contract verifies:
//!   * calling with `phase_slot: None` preserves pre-batch-fu behaviour
//!     (`phase_content == None`, `profile == FirstAttempt`).
//!   * calling with a populated `PhaseSlot` renders the CONSTRAINTS block
//!     into `DispatchContext.phase_content` exactly as `CONSTRAINTS:` +
//!     bullets + blank line + goal.
//!   * `phase_slot.profile_hint` is recorded on `DispatchContext.profile`.
//!   * `phase_slot.repair_snippet` survives as a caller-visible field on
//!     the `PhaseSlot` (the telescope invocation lives at the call site,
//!     not inside `build_dispatch_context`).

use std::collections::BTreeMap;

use elai_orchestrator::{
    build_dispatch_context, BackendConfig, BackendMode, PhaseSlot, CONSTRAINTS_BLOCK_HEADER,
};
use elai_orchestrator::context_budget::GenerationProfile;
use elai_orchestrator::fsm::FsmState;

fn sample_config() -> BackendConfig {
    BackendConfig {
        mode: BackendMode::Local,
        sidecar_url: "http://localhost:8100".into(),
        model: "test-model".into(),
        max_tools_local: 8,
        fallback_url: None,
        max_context: None,
    }
}

#[test]
fn contract_none_phase_slot_preserves_pre_batch_fu_behaviour() {
    let cfg = sample_config();
    let tools: Vec<String> = vec!["read_file".into()];
    let prio: BTreeMap<String, u8> = BTreeMap::new();

    let ctx = build_dispatch_context(FsmState::Planning, &cfg, &tools, &prio, None, None)
        .expect("build_dispatch_context");
    assert!(
        ctx.phase_content.is_none(),
        "None phase_slot must leave phase_content unset",
    );
    assert_eq!(ctx.profile, GenerationProfile::FirstAttempt);
}

#[test]
fn contract_populated_constraints_render_as_bulleted_block() {
    let cfg = sample_config();
    let tools: Vec<String> = vec!["read_file".into()];
    let prio: BTreeMap<String, u8> = BTreeMap::new();

    let slot = PhaseSlot {
        goal: "ship the baby step".into(),
        constraints: vec![
            "DO NOT USE blake2. Use blake3.".into(),
            "DO NOT USE rand 0.8. Use rand >= 0.9.".into(),
        ],
        profile_hint: Some(GenerationProfile::RepairRetry),
        repair_snippet: None,
    };
    let ctx = build_dispatch_context(
        FsmState::Executing,
        &cfg,
        &tools,
        &prio,
        None,
        Some(&slot),
    )
    .expect("build_dispatch_context");
    let body = ctx.phase_content.expect("phase_content populated");
    assert!(
        body.starts_with(CONSTRAINTS_BLOCK_HEADER),
        "phase_content must start with {CONSTRAINTS_BLOCK_HEADER}, got: {body}",
    );
    assert!(body.contains("- DO NOT USE blake2. Use blake3."));
    assert!(body.contains("- DO NOT USE rand 0.8. Use rand >= 0.9."));
    assert!(body.contains("ship the baby step"));
    assert_eq!(ctx.profile, GenerationProfile::RepairRetry);
}

#[test]
fn contract_empty_constraints_and_goal_yields_none() {
    let cfg = sample_config();
    let tools: Vec<String> = vec!["read_file".into()];
    let prio: BTreeMap<String, u8> = BTreeMap::new();
    let slot = PhaseSlot::default();
    let ctx = build_dispatch_context(
        FsmState::Executing,
        &cfg,
        &tools,
        &prio,
        None,
        Some(&slot),
    )
    .expect("build_dispatch_context");
    assert!(
        ctx.phase_content.is_none(),
        "Empty PhaseSlot must not create a stray empty phase_content",
    );
    assert_eq!(ctx.profile, GenerationProfile::FirstAttempt);
}

#[test]
fn contract_repair_snippet_is_preserved_on_phase_slot() {
    // The telescope invocation happens at the dispatch call site; this test
    // only verifies that the `repair_snippet` field exists on `PhaseSlot` and
    // round-trips through `#[derive(Default)]` so downstream callers can use
    // it unambiguously.
    let slot = PhaseSlot {
        repair_snippet: Some("fn broken() {}".into()),
        ..PhaseSlot::default()
    };
    assert_eq!(slot.repair_snippet.as_deref(), Some("fn broken() {}"));
    let d = PhaseSlot::default();
    assert!(d.repair_snippet.is_none());
    assert!(d.goal.is_empty());
    assert!(d.constraints.is_empty());
    assert!(d.profile_hint.is_none());
}
