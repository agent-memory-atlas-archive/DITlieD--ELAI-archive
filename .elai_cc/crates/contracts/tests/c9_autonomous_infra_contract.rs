//! C-9 (Phase 4G Autonomous Mode Core Infrastructure) — contract suite.

use std::sync::Arc;

use elai_orchestrator::research_tree::{QueryComplexity, ResearchPlan, ResearchTree};
use elai_orchestrator::{
    apply_task_runner_wiring, AutoAdvanceEvaluator, AutonomousEvidence, AutonomousGate,
    AutonomousGateConfig, AutonomousGateContext, AutonomyDecision, EscalationPolicy,
    EscalationTrigger, EvidenceVerdict, FsmTransition, GateMode, MergeStrategy, ModelPool,
    PipelineLevel, RiskLevel, RuntimeWiring, SplitTaskPlan, SubTask,
};

fn all_pass() -> AutonomousEvidence {
    AutonomousEvidence {
        knowledge_readiness: true,
        zero_ungrounded: true,
        tests_fail_correctly: true,
        test_flipped: true,
        zero_blocks: true,
        dkg_and_changelog_present: true,
        framing_converged: true,
        concept_zero_blocks: true,
        phase_zero_blocks: true,
        step_zero_blocks: true,
        babystep_zero_blocks: true,
        plan_zero_blocks: true,
        security_monotonicity: true,
        layer3_entrypoint_covered: true,
        layer4_trace_present: true,
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
        step_id: "c9-step".into(),
        model_pool: None,
        execution_history: elai_orchestrator::ExecutionHistory::default(),
        attempt_index: 0,
        suggested_prompt_prefix: None,
        candidate_skill_tier: None,
    }
}

struct FixedVramPool(u64);
impl ModelPool for FixedVramPool {
    fn available_vram(&self) -> u64 {
        self.0
    }
}

#[test]
fn c9_contract_escalation_policy_default_mirrors_legacy_limit() {
    let p = EscalationPolicy::default();
    assert_eq!(p.anti_spiral_threshold, 3);
    assert!(p.high_risk_pauses);
    assert_eq!(p.max_auto_advances_per_phase, 10);
    let cfg = AutonomousGateConfig::default();
    assert_eq!(cfg.circuit_breaker_limit, p.max_auto_advances_per_phase);
    assert_eq!(
        cfg.escalation_policy.max_auto_advances_per_phase,
        p.max_auto_advances_per_phase
    );
}

#[test]
fn c9_contract_auto_advance_evaluator_default_registers_5_axes() {
    let ev = AutoAdvanceEvaluator::default();
    assert_eq!(ev.gate_checks.len(), 5);
    for t in [
        FsmTransition::AntiSpiral,
        FsmTransition::HighRisk,
        FsmTransition::ReplanMajor,
        FsmTransition::CircuitBreaker,
        FsmTransition::AutoAdvance,
    ] {
        assert!(ev.gate_checks.contains_key(&t));
    }
}

#[test]
fn c9_contract_evaluator_evaluate_pass_on_all_pass_evidence() {
    let ev = AutoAdvanceEvaluator::default();
    assert_eq!(ev.evaluate(&FsmTransition::AutoAdvance, &all_pass()), EvidenceVerdict::Pass);
}

#[test]
fn c9_contract_evaluator_evaluate_fail_propagates_reason() {
    let ev = AutoAdvanceEvaluator::default();
    let mut evidence = all_pass();
    evidence.zero_ungrounded = false;
    match ev.evaluate(&FsmTransition::AutoAdvance, &evidence) {
        EvidenceVerdict::Fail { reason } => assert!(reason.contains("zero_ungrounded")),
        other => panic!("expected Fail, got {other:?}"),
    }
}

#[test]
fn c9_contract_escalation_trigger_has_six_variants() {
    assert_eq!(EscalationTrigger::AntiSpiral.as_str(), "anti_spiral");
    assert_eq!(EscalationTrigger::HighRisk.as_str(), "high_risk");
    assert_eq!(EscalationTrigger::ReplanMajor.as_str(), "replan_major");
    assert_eq!(EscalationTrigger::CircuitBreaker.as_str(), "circuit_breaker");
    assert_eq!(
        EscalationTrigger::SubstituteModelFailed.as_str(),
        "substitute_model_failed"
    );
    assert_eq!(EscalationTrigger::SplitTaskFailed.as_str(), "split_task_failed");
}

#[tokio::test]
async fn c9_contract_substitute_model_failed_when_chain_exhausted() {
    let gate = AutonomousGate::new(
        AutonomousGateConfig {
            mode: GateMode::Autonomous,
            model_fallback_chain: vec!["only".into()],
            ..AutonomousGateConfig::default()
        },
        None,
    );
    let mut ctx = ctx_at(PipelineLevel::Step, RiskLevel::Low);
    ctx.max_restarts_exhausted = true;
    assert_eq!(
        gate.decide(&ctx).await,
        AutonomyDecision::Escalate {
            trigger: EscalationTrigger::SubstituteModelFailed,
        }
    );
}

#[tokio::test]
async fn c9_contract_substitute_model_failed_when_target_oversized() {
    let mut sizes = rustc_hash::FxHashMap::default();
    sizes.insert("huge".to_owned(), 24 * 1024 * 1024 * 1024);
    let gate = AutonomousGate::new(
        AutonomousGateConfig {
            mode: GateMode::Autonomous,
            model_fallback_chain: vec!["small".into(), "huge".into()],
            estimated_model_sizes: sizes,
            ..AutonomousGateConfig::default()
        },
        None,
    );
    let mut ctx = ctx_at(PipelineLevel::Step, RiskLevel::Low);
    ctx.max_restarts_exhausted = true;
    ctx.model_pool = Some(Arc::new(FixedVramPool(2 * 1024 * 1024 * 1024)));
    assert_eq!(
        gate.decide(&ctx).await,
        AutonomyDecision::Escalate {
            trigger: EscalationTrigger::SubstituteModelFailed,
        }
    );
}

#[tokio::test]
async fn c9_contract_substitute_model_succeeds_when_target_fits() {
    let mut sizes = rustc_hash::FxHashMap::default();
    sizes.insert("tiny".to_owned(), 256 * 1024 * 1024);
    sizes.insert("small".to_owned(), 1024 * 1024 * 1024);
    let gate = AutonomousGate::new(
        AutonomousGateConfig {
            mode: GateMode::Autonomous,
            model_fallback_chain: vec!["small".into(), "tiny".into()],
            estimated_model_sizes: sizes,
            ..AutonomousGateConfig::default()
        },
        None,
    );
    let mut ctx = ctx_at(PipelineLevel::Step, RiskLevel::Low);
    ctx.max_restarts_exhausted = true;
    ctx.model_pool = Some(Arc::new(FixedVramPool(8 * 1024 * 1024 * 1024)));
    match gate.decide(&ctx).await {
        AutonomyDecision::SubstituteModel { from, to } => {
            assert_eq!(from, "small");
            assert_eq!(to, "tiny");
        }
        other => panic!("expected SubstituteModel, got {other:?}"),
    }
}

#[tokio::test]
async fn c9_contract_split_task_carries_plan_with_sequential_merge() {
    let gate = AutonomousGate::new(
        AutonomousGateConfig {
            mode: GateMode::Autonomous,
            ..AutonomousGateConfig::default()
        },
        None,
    );
    let mut ctx = ctx_at(PipelineLevel::Step, RiskLevel::Low);
    ctx.context_budget_terminal = true;
    match gate.decide(&ctx).await {
        AutonomyDecision::SplitTask { split_plan } => {
            assert_eq!(split_plan.sub_task_count(), 4);
            assert_eq!(split_plan.depth(), 2);
            assert_eq!(split_plan.merge_strategy, MergeStrategy::Sequential);
            assert_eq!(split_plan.parent_phase, "c9-step");
            assert_eq!(split_plan.sub_tasks.len(), 4);
            assert!(split_plan.sub_tasks[0].task_id.starts_with("c9-step.sub"));
        }
        other => panic!("expected SplitTask, got {other:?}"),
    }
}

#[test]
fn c9_contract_gate_mode_from_cli_args() {
    assert_eq!(GateMode::from_cli_args(None), GateMode::Interactive);
    assert_eq!(GateMode::from_cli_args(Some("autonomous")), GateMode::Autonomous);
    assert_eq!(GateMode::from_cli_args(Some("interactive")), GateMode::Interactive);
    assert_eq!(GateMode::from_cli_args(Some("hybrid")), GateMode::Interactive);
    assert_eq!(GateMode::from_cli_args(Some("garbage")), GateMode::Interactive);
    assert_eq!(GateMode::from_cli_args(Some("  AUTONOMOUS  ")), GateMode::Autonomous);
}

fn research_tree_for(question: &str) -> ResearchTree {
    use elai_core::clock::OsClock;
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    ResearchTree::new(
        ResearchPlan {
            root_question: question.into(),
            sub_questions: Vec::new(),
            complexity: QueryComplexity::Moderate,
            time_budget: std::time::Duration::from_secs(60),
            require_contradictions: false,
        },
        true,
        clock,
    )
    .expect("ResearchTree::new")
}

#[test]
fn c9_contract_research_tree_split_step_realises_plan() {
    let tree = research_tree_for("test question");
    let realised = tree.split_step(&SplitTaskPlan::deterministic("step-x", 3, 1));
    assert_eq!(realised.len(), 3);
    for (i, sub) in realised.iter().enumerate() {
        assert!(sub.task_id.contains("step-x"));
        assert_eq!(sub.task_id, format!("step-x.sub{i}"));
    }
}

#[test]
fn c9_contract_research_tree_split_step_empty_on_empty_plan() {
    let tree = research_tree_for("q");
    let empty_plan = SplitTaskPlan {
        parent_phase: "p".into(),
        sub_tasks: Vec::new(),
        merge_strategy: MergeStrategy::Sequential,
        max_depth: 2,
        current_depth: 0,
    };
    assert!(tree.split_step(&empty_plan).is_empty());
}

#[test]
fn c9_contract_research_tree_split_step_caps_at_max_depth() {
    let tree = research_tree_for("q");
    let at_cap = SplitTaskPlan {
        parent_phase: "p".into(),
        sub_tasks: vec![SubTask {
            task_id: "p.sub0".into(),
            scope: "s".into(),
            context_budget: 0,
            depends_on: Vec::new(),
        }],
        merge_strategy: MergeStrategy::Sequential,
        max_depth: 2,
        current_depth: 2,
    };
    assert!(tree.split_step(&at_cap).is_empty());
}

#[tokio::test]
async fn c9_contract_apply_task_runner_wiring_installs_gate() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let (ssb, feasibility_prober, layer1a_processor) = c9_wiring_fixture(tmp.path());

    assert_gate_mode(
        tmp.path(),
        ssb.clone(),
        feasibility_prober.clone(),
        layer1a_processor.clone(),
        None,
        None,
    )
    .await;
    assert_gate_mode(
        tmp.path(),
        ssb.clone(),
        feasibility_prober.clone(),
        layer1a_processor.clone(),
        Some(GateMode::Interactive),
        Some(GateMode::Interactive),
    )
    .await;
    assert_gate_mode(
        tmp.path(),
        ssb,
        feasibility_prober,
        layer1a_processor,
        Some(GateMode::Autonomous),
        Some(GateMode::Autonomous),
    )
    .await;
}

async fn assert_gate_mode(
    root: &std::path::Path,
    ssb: Arc<elai_orchestrator::Ssb>,
    feasibility_prober: Arc<elai_orchestrator::DefaultFeasibilityProber>,
    layer1a_processor: Arc<elai_orchestrator::DefaultLayer1aProcessor>,
    gate_mode: Option<GateMode>,
    expected: Option<GateMode>,
) {
    let wiring = c9_runtime_wiring(ssb, feasibility_prober, layer1a_processor, gate_mode);
    let cfg = apply_task_runner_wiring(root.join("cursor.json"), &wiring);
    match expected {
        None => assert!(cfg.autonomous_gate.is_none()),
        Some(expected_mode) => assert_gate_mode_is(&cfg, expected_mode).await,
    }
}

async fn assert_gate_mode_is(cfg: &elai_orchestrator::TaskRunnerConfig, expected: GateMode) {
    assert_eq!(
        cfg.autonomous_gate
            .as_ref()
            .expect("autonomous_gate must be installed")
            .mode()
            .await,
        expected
    );
}

fn c9_wiring_fixture(
    root: &std::path::Path,
) -> (
    Arc<elai_orchestrator::Ssb>,
    Arc<elai_orchestrator::DefaultFeasibilityProber>,
    Arc<elai_orchestrator::DefaultLayer1aProcessor>,
) {
    let sessions_dir = root.join(".elai_cc").join("sessions");
    std::fs::create_dir_all(&sessions_dir).expect("create sessions dir");
    let rules_db_path = root.join(".elai_cc").join("rules.db");
    let clock = elai_core::clock::OsClock;
    let conn = elai_rules::schema::open_rules_db(&rules_db_path, &clock).expect("open rules.db");
    let rules_db = Arc::new(tokio::sync::Mutex::new(conn));
    let ssb_config = elai_orchestrator::SsbConfig {
        db_path: sessions_dir.join(elai_orchestrator::SSB_DB_FILE_NAME),
        pool_size: Some(1),
        project_id: "c9-test".to_string(),
        session_id: "c9-session".to_string(),
    };
    let clock_arc: Arc<dyn elai_core::clock::ClockProvider> =
        Arc::new(elai_core::clock::OsClock);
    let ssb = Arc::new(elai_orchestrator::Ssb::new(ssb_config, clock_arc).expect("Ssb::new"));
    let feasibility_prober = Arc::new(
        elai_orchestrator::DefaultFeasibilityProber::new(root.join("feasibility")),
    );
    let layer1a_processor = Arc::new(elai_orchestrator::DefaultLayer1aProcessor::new(
        rules_db,
        rules_db_path,
        ssb.clone(),
    ));
    (ssb, feasibility_prober, layer1a_processor)
}

fn c9_runtime_wiring(
    ssb: Arc<elai_orchestrator::Ssb>,
    feasibility_prober: Arc<elai_orchestrator::DefaultFeasibilityProber>,
    layer1a_processor: Arc<elai_orchestrator::DefaultLayer1aProcessor>,
    gate_mode: Option<GateMode>,
) -> RuntimeWiring {
    RuntimeWiring {
        ssb,
        feasibility_prober,
        layer1a_processor,
        baby_step_executor: None,
        gate_mode,
        run_profile: None,
        cloud_config: None,
        provider_override: None,
        hook_dispatcher: None,
        crystallizer: None,
        compiled_pipeline_store: None,
        background_task_manager: None,
        skill_store: None,
        via_mcp: false,
        mcp_dispatch: None,
        buffer_editor: None,
        pipeline_grc: None,
        interactive_gate: None,
        vibegate_seed_ok: true,
        reflection_client: None,
        adversary_runtime: None,
        #[cfg(feature = "empire")]
        empire_bridge: None,
    }
}

#[test]
fn c9_contract_modelpool_can_fit_llm_default_uses_available_vram() {
    let pool = FixedVramPool(4 * 1024 * 1024 * 1024);
    assert!(pool.can_fit_llm(1024 * 1024 * 1024));
    assert!(pool.can_fit_llm(4 * 1024 * 1024 * 1024));
    assert!(!pool.can_fit_llm(8 * 1024 * 1024 * 1024));
}
