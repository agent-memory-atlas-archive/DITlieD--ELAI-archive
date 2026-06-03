//! B-22 Phase 4G — Extended autonomous-mode contracts.
//!
//! Pins the 18 additional surfaces required by phase-4G.txt §4G beyond the
//! 14 contracts in `autonomous_gate_contract.rs` and the 4 in
//! `mode_switch_contract.rs`. Layout:
//!
//!   1..4   per-pipeline-level auto-advance (BabyStep, Step, Phase, Concept)
//!   5..6   tree-policy permutations
//!   7..12  per-evidence-field block contracts (one per field, named explicitly)
//!  13..15  risk-level contracts (LOW autonomous, MEDIUM interactive, HIGH autonomous)
//!  16..18  mode-switch contracts (decision flip / circuit breaker / active model)
//!
//! The /mode slash-command parser and `AgentPool::get_role` are exercised by
//! unit tests inside `crates/cli/src/commands.rs` and
//! `crates/orchestrator/src/agent_pool.rs` respectively (binary-only and
//! crate-private boundaries make those tests unreachable from this crate).

use std::sync::{Mutex, OnceLock};

use elai_orchestrator::{
    AutonomousGate, AutonomousGateConfig, AutonomousEvidence, AutonomousGateContext,
    AutonomyDecision, EscalationTrigger, GateMode, PipelineLevel, RiskLevel,
    TreeAutoAdvancePolicy,
};

fn env_mutex() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn all_pass() -> AutonomousEvidence {
    AutonomousEvidence {
        knowledge_readiness: true,
        zero_ungrounded: true,
        tests_fail_correctly: true,
        test_flipped: true,
        zero_blocks: true,
        dkg_and_changelog_present: true,
        // B-40: 7th check vacuous-default true for non-FRAMING contexts.
        framing_converged: true,
        // B-41: 5 per-level audit zero-blocks checks; vacuous-default true
        // for non-per-level contexts (Default impl flips these to true).
        // SP5 wiring-fix: the off-agent EXECUTING -> AUDITING legs default
        // FALSE in Default, so an all-pass fixture MUST set them true
        // explicitly (else gate.decide returns RequestHuman, not AutoAdvance).
        layer3_entrypoint_covered: true,
        layer4_trace_present: true,
        ..AutonomousEvidence::default()
    }
}

fn ctx_at(level: PipelineLevel, risk: RiskLevel) -> AutonomousGateContext {
    AutonomousGateContext {
        level,
        risk_level: risk,
        evidence: all_pass(),
        anti_spiral_fired: false,
        replan_major: false,
        max_restarts_exhausted: false,
        context_budget_terminal: false,
        auto_advance_count_for_concept: 0,
        step_id: "step-x".into(),
        // C-9: optional ModelPool handle; pre-C-9 contracts default to None.
        model_pool: None,
        // C-10: anti-spiral extended fields default to empty/None so
        // pre-C-10 contracts compile unchanged.
        execution_history: elai_orchestrator::ExecutionHistory::default(),
        attempt_index: 0,
        suggested_prompt_prefix: None,
        // F-6 (C6a): no candidate learned skill in the base contract fixture.
        candidate_skill_tier: None,
    }
}

fn autonomous_cfg() -> AutonomousGateConfig {
    AutonomousGateConfig {
        mode: GateMode::Autonomous,
        ..AutonomousGateConfig::default()
    }
}

// ---------------------------------------------------------------------------
// 1..4 — auto-advance per pipeline level
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_4g_baby_step_auto_advances_in_autonomous_low_risk() {
    let _g = env_mutex().lock().expect("env mutex");
    let gate = AutonomousGate::new(autonomous_cfg(), None);
    let ctx = ctx_at(PipelineLevel::BabyStep, RiskLevel::Low);
    assert_eq!(gate.decide(&ctx).await, AutonomyDecision::AutoAdvance);
}

#[tokio::test]
async fn contract_4g_step_auto_advances_in_autonomous_low_risk() {
    let _g = env_mutex().lock().expect("env mutex");
    let gate = AutonomousGate::new(autonomous_cfg(), None);
    let ctx = ctx_at(PipelineLevel::Step, RiskLevel::Low);
    assert_eq!(gate.decide(&ctx).await, AutonomyDecision::AutoAdvance);
}

#[tokio::test]
async fn contract_4g_phase_auto_advances_in_autonomous_low_risk() {
    let _g = env_mutex().lock().expect("env mutex");
    let gate = AutonomousGate::new(autonomous_cfg(), None);
    let ctx = ctx_at(PipelineLevel::Phase, RiskLevel::Low);
    assert_eq!(gate.decide(&ctx).await, AutonomyDecision::AutoAdvance);
}

#[tokio::test]
async fn contract_4g_concept_requests_human_under_default_policy() {
    let _g = env_mutex().lock().expect("env mutex");
    let gate = AutonomousGate::new(autonomous_cfg(), None);
    let ctx = ctx_at(PipelineLevel::Concept, RiskLevel::Low);
    let d = gate.decide(&ctx).await;
    assert!(
        matches!(d, AutonomyDecision::RequestHuman { .. }),
        "concept must always request human under default policy: got {d:?}"
    );
}

// ---------------------------------------------------------------------------
// 5..6 — tree-policy permutations
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_4g_phase_policy_off_blocks_even_low_risk() {
    let _g = env_mutex().lock().expect("env mutex");
    let cfg = AutonomousGateConfig {
        mode: GateMode::Autonomous,
        tree_auto_advance_policy: TreeAutoAdvancePolicy {
            phase_auto_advance_if_not_medium: false,
            ..TreeAutoAdvancePolicy::default()
        },
        ..AutonomousGateConfig::default()
    };
    let gate = AutonomousGate::new(cfg, None);
    let ctx = ctx_at(PipelineLevel::Phase, RiskLevel::Low);
    let d = gate.decide(&ctx).await;
    assert!(
        matches!(d, AutonomyDecision::RequestHuman { .. }),
        "phase_auto_advance_if_not_medium=false must block: got {d:?}"
    );
}

#[tokio::test]
async fn contract_4g_baby_step_policy_off_blocks() {
    let _g = env_mutex().lock().expect("env mutex");
    let cfg = AutonomousGateConfig {
        mode: GateMode::Autonomous,
        tree_auto_advance_policy: TreeAutoAdvancePolicy {
            baby_step_auto_advance: false,
            ..TreeAutoAdvancePolicy::default()
        },
        ..AutonomousGateConfig::default()
    };
    let gate = AutonomousGate::new(cfg, None);
    let ctx = ctx_at(PipelineLevel::BabyStep, RiskLevel::Low);
    let d = gate.decide(&ctx).await;
    assert!(
        matches!(d, AutonomyDecision::RequestHuman { .. }),
        "baby_step_auto_advance=false must block: got {d:?}"
    );
}

// ---------------------------------------------------------------------------
// 7..12 — per-evidence-field block contracts
// ---------------------------------------------------------------------------

async fn assert_field_blocks(
    mutate: impl FnOnce(&mut AutonomousEvidence),
    field_label: &str,
) {
    let gate = AutonomousGate::new(autonomous_cfg(), None);
    let mut ctx = ctx_at(PipelineLevel::BabyStep, RiskLevel::Low);
    mutate(&mut ctx.evidence);
    let d = gate.decide(&ctx).await;
    match d {
        AutonomyDecision::RequestHuman { reason, .. } => {
            assert!(
                reason.contains(field_label),
                "expected reason mentioning `{field_label}`, got `{reason}`"
            );
        }
        other => panic!("expected RequestHuman for {field_label}, got {other:?}"),
    }
}

#[tokio::test]
async fn contract_4g_block_when_knowledge_readiness_false() {
    let _g = env_mutex().lock().expect("env mutex");
    assert_field_blocks(|e| e.knowledge_readiness = false, "knowledge_readiness").await;
}

#[tokio::test]
async fn contract_4g_block_when_zero_ungrounded_false() {
    let _g = env_mutex().lock().expect("env mutex");
    assert_field_blocks(|e| e.zero_ungrounded = false, "zero_ungrounded").await;
}

#[tokio::test]
async fn contract_4g_block_when_tests_fail_correctly_false() {
    let _g = env_mutex().lock().expect("env mutex");
    assert_field_blocks(|e| e.tests_fail_correctly = false, "tests_fail_correctly").await;
}

#[tokio::test]
async fn contract_4g_block_when_test_flipped_false() {
    let _g = env_mutex().lock().expect("env mutex");
    assert_field_blocks(|e| e.test_flipped = false, "test_flipped").await;
}

#[tokio::test]
async fn contract_4g_block_when_zero_blocks_false() {
    let _g = env_mutex().lock().expect("env mutex");
    assert_field_blocks(|e| e.zero_blocks = false, "zero_blocks").await;
}

#[tokio::test]
async fn contract_4g_block_when_dkg_and_changelog_present_false() {
    let _g = env_mutex().lock().expect("env mutex");
    assert_field_blocks(
        |e| e.dkg_and_changelog_present = false,
        "dkg_and_changelog_present",
    )
    .await;
}

// ---------------------------------------------------------------------------
// 13..15 — risk-level contracts
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_4g_risk_low_autonomous_advances() {
    let _g = env_mutex().lock().expect("env mutex");
    let gate = AutonomousGate::new(autonomous_cfg(), None);
    let ctx = ctx_at(PipelineLevel::BabyStep, RiskLevel::Low);
    assert_eq!(gate.decide(&ctx).await, AutonomyDecision::AutoAdvance);
}

#[tokio::test]
async fn contract_4g_risk_medium_interactive_requests_human() {
    let _g = env_mutex().lock().expect("env mutex");
    // Default config = Interactive mode.
    let gate = AutonomousGate::new(AutonomousGateConfig::default(), None);
    let ctx = ctx_at(PipelineLevel::Phase, RiskLevel::Medium);
    let d = gate.decide(&ctx).await;
    assert!(
        matches!(d, AutonomyDecision::RequestHuman { .. }),
        "interactive + medium must request human: got {d:?}"
    );
}

#[tokio::test]
async fn contract_4g_risk_high_escalates_under_autonomous() {
    let _g = env_mutex().lock().expect("env mutex");
    let gate = AutonomousGate::new(autonomous_cfg(), None);
    let ctx = ctx_at(PipelineLevel::Phase, RiskLevel::High);
    let d = gate.decide(&ctx).await;
    assert!(
        matches!(
            d,
            AutonomyDecision::Escalate {
                trigger: EscalationTrigger::HighRisk
            }
        ),
        "HIGH risk must escalate via HighRisk: got {d:?}"
    );
}

// ---------------------------------------------------------------------------
// 16..18 — mode-switch contracts
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_4g_mode_switch_changes_decision_for_low_risk_phase() {
    let _g = env_mutex().lock().expect("env mutex");
    // Start interactive: Phase + LOW still requires human (interactive non-LOW
    // requests human; the gate's interactive branch already short-circuits at
    // non-Low, so we use a Step under MEDIUM here for a robust comparison).
    let gate = AutonomousGate::new(AutonomousGateConfig::default(), None);
    let ctx_med = ctx_at(PipelineLevel::Step, RiskLevel::Medium);
    let interactive_d = gate.decide(&ctx_med).await;
    assert!(
        matches!(interactive_d, AutonomyDecision::RequestHuman { .. }),
        "interactive + medium must request human: got {interactive_d:?}"
    );
    // Flip to autonomous; Step + LOW must auto-advance.
    let prev = gate.set_mode(GateMode::Autonomous).await;
    assert_eq!(prev, GateMode::Interactive);
    let ctx_low = ctx_at(PipelineLevel::Step, RiskLevel::Low);
    let autonomous_d = gate.decide(&ctx_low).await;
    assert_eq!(autonomous_d, AutonomyDecision::AutoAdvance);
}

#[tokio::test]
async fn contract_4g_mode_switch_preserves_circuit_breaker_count() {
    let _g = env_mutex().lock().expect("env mutex");
    let gate = AutonomousGate::new(
        AutonomousGateConfig {
            mode: GateMode::Autonomous,
            circuit_breaker_limit: 50,
            ..AutonomousGateConfig::default()
        },
        None,
    );
    // Burn 3 auto-advances under Autonomous.
    let ctx = ctx_at(PipelineLevel::BabyStep, RiskLevel::Low);
    for _ in 0..3 {
        assert_eq!(gate.decide(&ctx).await, AutonomyDecision::AutoAdvance);
    }
    let count_before = gate.circuit_breaker_count();
    assert_eq!(count_before, 3);

    // Switch to interactive and back. The breaker count must NOT reset.
    gate.set_mode(GateMode::Interactive).await;
    gate.set_mode(GateMode::Autonomous).await;
    let count_after = gate.circuit_breaker_count();
    assert_eq!(
        count_after, 3,
        "mode switch must not reset circuit breaker; before={count_before} after={count_after}"
    );
}

#[tokio::test]
async fn contract_4g_mode_switch_preserves_active_model() {
    let _g = env_mutex().lock().expect("env mutex");
    let gate = AutonomousGate::new(
        AutonomousGateConfig {
            mode: GateMode::Autonomous,
            model_fallback_chain: vec!["m-a".into(), "m-b".into(), "m-c".into()],
            ..AutonomousGateConfig::default()
        },
        None,
    );
    assert_eq!(gate.active_model().await, "m-a");
    let swap = gate.substitute_model().await;
    assert_eq!(swap, Some(("m-a".into(), "m-b".into())));
    assert_eq!(gate.active_model().await, "m-b");

    // Mode switch must not rewind active_model.
    gate.set_mode(GateMode::Interactive).await;
    assert_eq!(gate.active_model().await, "m-b");
    gate.set_mode(GateMode::Autonomous).await;
    assert_eq!(
        gate.active_model().await,
        "m-b",
        "mode switch must not rewind substituted active_model"
    );
}
