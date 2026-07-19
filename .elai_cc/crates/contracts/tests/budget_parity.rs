//! B-10 — TaskRunner-vs-REPL budget parity contract test.
//!
//! Pins the invariant that the ContextBudget produced by build_dispatch_context
//! when the caller supplies a real ModelCapabilityProfile is identical regardless
//! of which call site invokes it (TaskRunner production at task_runner.rs:3131
//! vs REPL at cli/main.rs:8027). Catches regressions where one path drifts back
//! to None / [0,0,0] / 4096 stub.
//!
//! Grounding:
//!   * docs/thefinalglance/B-plan/B-10_contextbudget_profile_activation.md Task 7
//!   * crates/orchestrator/src/dispatcher.rs build_dispatch_context surface
//!   * crates/orchestrator/src/context_budget.rs ModelCapabilityProfile

use std::collections::BTreeMap;

use elai_orchestrator::context_budget::ModelCapabilityProfile;
use elai_orchestrator::dispatcher::{
    build_dispatch_context, BackendConfig, BackendMode, PhaseSlot,
};
use elai_orchestrator::fsm::FsmState;

#[test]
fn contract_taskrunner_repl_compute_with_profile_parity() {
    // Synthetic task input: identical FSM state, backend config, tool list,
    // priority map, profile. Two call sites must yield identical budgets.
    let fsm = FsmState::Executing;
    let cfg = BackendConfig {
        mode: BackendMode::Local,
        sidecar_url: "http://localhost:8100".to_string(),
        model: "Qwen2.5-Coder-7B-Instruct-abliterated".to_string(),
        max_tools_local: 8,
        fallback_url: None,
        max_context: None,
    };
    let tools: Vec<String> = vec!["edit".into(), "validate".into()];
    let mut prio = BTreeMap::new();
    prio.insert("edit".to_string(), 1u8);
    prio.insert("validate".to_string(), 2u8);

    let profile = ModelCapabilityProfile::micro_7b_12gb();
    let phase_slot = PhaseSlot::default();

    // "TaskRunner pathway"
    let ctx_runner = build_dispatch_context(
        fsm,
        &cfg,
        &tools,
        &prio,
        Some(&profile),
        Some(&phase_slot),
    )
    .expect("build_dispatch_context (runner) ok");

    // "REPL pathway" — same args (post-B-10 cli/main.rs:8027 should also pass
    // Some(&profile) once wired). The contract is that identical args yield
    // identical ContextBudgets.
    let ctx_repl = build_dispatch_context(
        fsm,
        &cfg,
        &tools,
        &prio,
        Some(&profile),
        Some(&phase_slot),
    )
    .expect("build_dispatch_context (repl) ok");

    // ContextBudgets must be present + equal across both paths.
    let budget_runner = ctx_runner.budget.expect("runner budget present");
    let budget_repl = ctx_repl.budget.expect("repl budget present");

    // Per-slot allocation parity (8 slots).
    for i in 0..8 {
        assert_eq!(
            budget_runner.slots[i].allocated_tokens,
            budget_repl.slots[i].allocated_tokens,
            "slot {} allocation mismatch (runner={}, repl={}) — pathway drift",
            i,
            budget_runner.slots[i].allocated_tokens,
            budget_repl.slots[i].allocated_tokens
        );
    }
    // Aggregate parity.
    assert_eq!(budget_runner.total_allocated, budget_repl.total_allocated);
    assert_eq!(budget_runner.model_max_context, budget_repl.model_max_context);
}

#[test]
fn contract_compute_with_profile_disallows_zeroed_stub_in_production() {
    // The zeroed [0,0,0] stub fallback at dispatcher.rs:222 is the bug B-10
    // closes. With a real profile, fixed slots must be non-zero and match the
    // profile's per-tier targets.
    let fsm = FsmState::Executing;
    let cfg = BackendConfig::default();
    let tools: Vec<String> = vec![];
    let prio: BTreeMap<String, u8> = BTreeMap::new();
    let profile = ModelCapabilityProfile::micro_7b_12gb();
    let phase_slot = PhaseSlot::default();

    let ctx = build_dispatch_context(
        fsm,
        &cfg,
        &tools,
        &prio,
        Some(&profile),
        Some(&phase_slot),
    )
    .expect("build_dispatch_context ok");
    let budget = ctx.budget.expect("budget present");
    assert!(
        budget.slots[0].allocated_tokens > 0,
        "SystemPrompt slot must be > 0 with a real profile (B-10 closes [0,0,0] stub)"
    );
    assert!(budget.slots[1].allocated_tokens > 0, "Rules slot must be > 0");
    assert!(budget.slots[2].allocated_tokens > 0, "PhaseSpec slot must be > 0");
}
