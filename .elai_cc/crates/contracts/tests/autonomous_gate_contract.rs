//! Batch 3B (Phase 4G) — AutonomousGate contract.
//!
//! Pins the behaviour of [`elai_orchestrator::AutonomousGate`] so the 4G
//! decision semantics stay executable. Every contract below mirrors a bullet
//! from the batch-3b prompt §11.

use std::sync::{Arc, Mutex, OnceLock};

use elai_orchestrator::{
    autonomous_gate_disabled_via_env, AutonomousGate, AutonomousGateConfig, AutonomousEvidence,
    AutonomousGateContext, AutonomyDecision, EscalationTrigger, GateMode, PipelineLevel,
    RiskLevel, TreeAutoAdvancePolicy,
};

/// Serialises env-mutating tests against env-reading tests. Rust's default
/// test harness runs tests in parallel, and the `ELAI_DISABLE_AUTONOMOUS_GATE`
/// variable is process-wide — without this guard the kill-switch test races
/// every other contract in this file.
fn env_mutex() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn all_pass_evidence() -> AutonomousEvidence {
    AutonomousEvidence {
        knowledge_readiness: true,
        zero_ungrounded: true,
        tests_fail_correctly: true,
        test_flipped: true,
        zero_blocks: true,
        dkg_and_changelog_present: true,
    }
}

fn ctx_at(level: PipelineLevel, risk: RiskLevel) -> AutonomousGateContext {
    AutonomousGateContext {
        level,
        risk_level: risk,
        evidence: all_pass_evidence(),
        anti_spiral_fired: false,
        replan_major: false,
        max_restarts_exhausted: false,
        context_budget_terminal: false,
        auto_advance_count_for_concept: 0,
        step_id: "step-1".into(),
    }
}

fn autonomous_config() -> AutonomousGateConfig {
    AutonomousGateConfig {
        mode: GateMode::Autonomous,
        ..AutonomousGateConfig::default()
    }
}

// ---------------------------------------------------------------------------
// Contract 1 — trait-style usage / Arc dispatch.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_gate_arc_dispatch() {
    let _g = env_mutex().lock().expect("env mutex");
    let gate: Arc<AutonomousGate> =
        Arc::new(AutonomousGate::new(autonomous_config(), None));
    let ctx = ctx_at(PipelineLevel::BabyStep, RiskLevel::Low);
    let d = gate.decide(&ctx).await;
    assert_eq!(d, AutonomyDecision::AutoAdvance);
}

// ---------------------------------------------------------------------------
// Contract 2 — each of the 6 evidence checks fires correctly on fixture state.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_each_evidence_failure_requests_human() {
    let _g = env_mutex().lock().expect("env mutex");
    let gate = AutonomousGate::new(autonomous_config(), None);
    for field_idx in 0..6 {
        let mut e = all_pass_evidence();
        match field_idx {
            0 => e.knowledge_readiness = false,
            1 => e.zero_ungrounded = false,
            2 => e.tests_fail_correctly = false,
            3 => e.test_flipped = false,
            4 => e.zero_blocks = false,
            5 => e.dkg_and_changelog_present = false,
            _ => unreachable!(),
        }
        let mut ctx = ctx_at(PipelineLevel::BabyStep, RiskLevel::Low);
        ctx.evidence = e;
        let d = gate.decide(&ctx).await;
        match d {
            AutonomyDecision::RequestHuman { reason, .. } => {
                assert!(
                    reason.contains("evidence failed"),
                    "field {field_idx}: unexpected reason {reason}"
                );
            }
            other => panic!("field {field_idx}: expected RequestHuman, got {other:?}"),
        }
    }
}

// ---------------------------------------------------------------------------
// Contract 3 — AutoAdvance when all 6 checks pass and risk is LOW.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_auto_advance_low_risk() {
    let _g = env_mutex().lock().expect("env mutex");
    let gate = AutonomousGate::new(autonomous_config(), None);
    let ctx = ctx_at(PipelineLevel::BabyStep, RiskLevel::Low);
    assert_eq!(gate.decide(&ctx).await, AutonomyDecision::AutoAdvance);
}

// ---------------------------------------------------------------------------
// Contract 4 — HIGH risk always escalates (precedence over evidence checks).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_high_risk_escalates() {
    let _g = env_mutex().lock().expect("env mutex");
    let gate = AutonomousGate::new(autonomous_config(), None);
    let ctx = ctx_at(PipelineLevel::Phase, RiskLevel::High);
    assert_eq!(
        gate.decide(&ctx).await,
        AutonomyDecision::Escalate {
            trigger: EscalationTrigger::HighRisk,
        }
    );
}

// ---------------------------------------------------------------------------
// Contract 5 — circuit breaker: limit+1th auto-advance trips.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_circuit_breaker_trips_after_limit() {
    let _g = env_mutex().lock().expect("env mutex");
    let limit: u32 = 11;
    let cfg = AutonomousGateConfig {
        mode: GateMode::Autonomous,
        circuit_breaker_limit: limit,
        ..AutonomousGateConfig::default()
    };
    let gate = AutonomousGate::new(cfg, None);
    let ctx = ctx_at(PipelineLevel::BabyStep, RiskLevel::Low);
    for i in 0..limit {
        let d = gate.decide(&ctx).await;
        assert_eq!(
            d,
            AutonomyDecision::AutoAdvance,
            "iter {i} should still auto-advance"
        );
    }
    let d = gate.decide(&ctx).await;
    assert_eq!(
        d,
        AutonomyDecision::Escalate {
            trigger: EscalationTrigger::CircuitBreaker,
        }
    );
}

// ---------------------------------------------------------------------------
// Contract 6 — AntiSpiral fires the Escalate variant, overriding every other
// signal.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_anti_spiral_overrides_autoadvance() {
    let _g = env_mutex().lock().expect("env mutex");
    let gate = AutonomousGate::new(autonomous_config(), None);
    let mut ctx = ctx_at(PipelineLevel::BabyStep, RiskLevel::Low);
    ctx.anti_spiral_fired = true;
    assert_eq!(
        gate.decide(&ctx).await,
        AutonomyDecision::Escalate {
            trigger: EscalationTrigger::AntiSpiral,
        }
    );
}

// ---------------------------------------------------------------------------
// Contract 7 — SplitTask on context budget Terminal (autonomous only).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_split_task_on_context_terminal() {
    let _g = env_mutex().lock().expect("env mutex");
    let gate = AutonomousGate::new(autonomous_config(), None);
    let mut ctx = ctx_at(PipelineLevel::Step, RiskLevel::Low);
    ctx.context_budget_terminal = true;
    assert_eq!(
        gate.decide(&ctx).await,
        AutonomyDecision::SplitTask {
            sub_task_count: 4,
            depth: 2,
        }
    );
}

// ---------------------------------------------------------------------------
// Contract 8 — SubstituteModel on max_restarts (autonomous only).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_substitute_model_on_max_restarts() {
    let _g = env_mutex().lock().expect("env mutex");
    let cfg = AutonomousGateConfig {
        mode: GateMode::Autonomous,
        model_fallback_chain: vec!["primary".into(), "secondary".into()],
        ..AutonomousGateConfig::default()
    };
    let gate = AutonomousGate::new(cfg, None);
    let mut ctx = ctx_at(PipelineLevel::Step, RiskLevel::Low);
    ctx.max_restarts_exhausted = true;
    assert_eq!(
        gate.decide(&ctx).await,
        AutonomyDecision::SubstituteModel {
            from: "primary".into(),
            to: "secondary".into(),
        }
    );
}

// ---------------------------------------------------------------------------
// Contract 9 — set_mode flips config atomically.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_set_mode_roundtrip() {
    let _g = env_mutex().lock().expect("env mutex");
    let gate = AutonomousGate::new(AutonomousGateConfig::default(), None);
    assert_eq!(gate.mode().await, GateMode::Interactive);
    let prev = gate.set_mode(GateMode::Autonomous).await;
    assert_eq!(prev, GateMode::Interactive);
    assert_eq!(gate.mode().await, GateMode::Autonomous);
    let prev2 = gate.set_mode(GateMode::Interactive).await;
    assert_eq!(prev2, GateMode::Autonomous);
}

// ---------------------------------------------------------------------------
// Contract 10 — per-level risk defaults (BabyStep=Low, Phase=Medium,
// Concept=High).
// ---------------------------------------------------------------------------

#[test]
fn contract_pipeline_level_risk_defaults() {
    assert_eq!(PipelineLevel::BabyStep.default_risk(), RiskLevel::Low);
    assert_eq!(PipelineLevel::Step.default_risk(), RiskLevel::Low);
    assert_eq!(PipelineLevel::Phase.default_risk(), RiskLevel::Medium);
    assert_eq!(PipelineLevel::Concept.default_risk(), RiskLevel::High);
}

// ---------------------------------------------------------------------------
// Contract 11 — auto-advance cap: 50 baby steps per concept.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_baby_step_cap_per_concept() {
    let _g = env_mutex().lock().expect("env mutex");
    let policy = TreeAutoAdvancePolicy::default();
    assert_eq!(policy.max_auto_baby_steps_per_concept, 50);
    let cfg = AutonomousGateConfig {
        mode: GateMode::Autonomous,
        tree_auto_advance_policy: policy,
        circuit_breaker_limit: 1_000, // avoid session-wide breaker
        ..AutonomousGateConfig::default()
    };
    let gate = AutonomousGate::new(cfg, None);
    let mut ctx = ctx_at(PipelineLevel::BabyStep, RiskLevel::Low);
    ctx.auto_advance_count_for_concept = 50;
    let d = gate.decide(&ctx).await;
    match d {
        AutonomyDecision::RequestHuman { reason, .. } => {
            assert!(reason.contains("cap reached"));
        }
        other => panic!("expected RequestHuman, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Contract 12 — absence (no autonomous_gate on TaskRunnerConfig) means the
// task runner keeps the pre-4G behaviour. We assert it by confirming that
// `TaskRunnerConfig::new_minimal` yields `None`.
// ---------------------------------------------------------------------------

#[test]
fn contract_task_runner_config_autonomous_gate_defaults_to_none() {
    let cfg = elai_orchestrator::TaskRunnerConfig::new_minimal(
        std::path::PathBuf::from("/tmp/elai_cc_test_cursor.json"),
    );
    assert!(
        cfg.autonomous_gate.is_none(),
        "new_minimal must leave autonomous_gate unset so None = pre-4G behaviour"
    );
}

// ---------------------------------------------------------------------------
// Contract 13 — ELAI_DISABLE_AUTONOMOUS_GATE kill switch returns RequestHuman
// regardless of mode.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_kill_switch_requests_human() {
    let _g = env_mutex().lock().expect("env mutex");
    // Set and unset via a guard so parallel tests don't interfere.
    struct EnvGuard(&'static str, Option<String>);
    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.1 {
                Some(v) => std::env::set_var(self.0, v),
                None => std::env::remove_var(self.0),
            }
        }
    }
    let key = "ELAI_DISABLE_AUTONOMOUS_GATE";
    let prev = std::env::var(key).ok();
    std::env::set_var(key, "1");
    let _guard = EnvGuard(key, prev);

    assert!(autonomous_gate_disabled_via_env());
    let gate = AutonomousGate::new(autonomous_config(), None);
    let ctx = ctx_at(PipelineLevel::BabyStep, RiskLevel::Low);
    let d = gate.decide(&ctx).await;
    assert!(matches!(d, AutonomyDecision::RequestHuman { .. }));
}

// ---------------------------------------------------------------------------
// Contract 14 — model chain swap semantics.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_model_chain_swap() {
    let _g = env_mutex().lock().expect("env mutex");
    let cfg = AutonomousGateConfig {
        mode: GateMode::Autonomous,
        model_fallback_chain: vec!["a".into(), "b".into(), "c".into()],
        ..AutonomousGateConfig::default()
    };
    let gate = AutonomousGate::new(cfg, None);
    assert_eq!(gate.active_model().await, "a");
    let s1 = gate.substitute_model().await;
    assert_eq!(s1, Some(("a".into(), "b".into())));
    assert_eq!(gate.active_model().await, "b");
    let s2 = gate.substitute_model().await;
    assert_eq!(s2, Some(("b".into(), "c".into())));
    assert_eq!(gate.active_model().await, "c");
    // Exhausted.
    assert_eq!(gate.substitute_model().await, None);
    assert_eq!(gate.active_model().await, "c");
}
