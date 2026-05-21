//! C-9 (Phase 4G Autonomous Mode Core Infrastructure) — contract suite.
//!
//! Pins the named-type and wiring invariants C-9 ships:
//!   * AutoAdvanceEvaluator default registers one EvidenceCheck per
//!     EscalationTrigger axis plus an AutoAdvance catch-all.
//!   * EscalationPolicy default mirrors the legacy
//!     `AutonomousGateConfig::circuit_breaker_limit`.
//!   * EscalationTrigger taxonomy gained the SubstituteModelFailed +
//!     SplitTaskFailed variants (6 total).
//!   * SubstituteModel guards via ModelPool::can_fit_llm — fires
//!     SubstituteModelFailed when the target won't fit.
//!   * AutonomyDecision::SplitTask carries a SplitTaskPlan with
//!     MergeStrategy::Sequential.
//!   * GateMode::from_cli_args parses the --mode flag with the same three
//!     labels accepted by `parse_run_subcommand_args`.
//!   * ResearchTree::split_step realises a SplitTaskPlan with id rewrites.
//!   * apply_task_runner_wiring installs an AutonomousGate when
//!     `RuntimeWiring.gate_mode == Some(_)`.

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
        // C-10: anti-spiral extended fields default to empty/None so
        // pre-C-10 contracts compile unchanged.
        execution_history: elai_orchestrator::ExecutionHistory::default(),
        attempt_index: 0,
        suggested_prompt_prefix: None,
    }
}

struct FixedVramPool(u64);
impl ModelPool for FixedVramPool {
    fn available_vram(&self) -> u64 {
        self.0
    }
}

// ---------------------------------------------------------------------------
// Contract 1 — EscalationPolicy default
// ---------------------------------------------------------------------------

#[test]
fn c9_contract_escalation_policy_default_mirrors_legacy_limit() {
    let p = EscalationPolicy::default();
    assert_eq!(p.anti_spiral_threshold, 3);
    assert!(p.high_risk_pauses);
    assert_eq!(p.max_auto_advances_per_phase, 10);
    // Legacy AutonomousGateConfig::default() keeps the same number.
    let cfg = AutonomousGateConfig::default();
    assert_eq!(cfg.circuit_breaker_limit, p.max_auto_advances_per_phase);
    assert_eq!(
        cfg.escalation_policy.max_auto_advances_per_phase,
        p.max_auto_advances_per_phase
    );
}

// ---------------------------------------------------------------------------
// Contract 2 — AutoAdvanceEvaluator default registry
// ---------------------------------------------------------------------------

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
        assert!(
            ev.gate_checks.contains_key(&t),
            "AutoAdvanceEvaluator default must register {:?}",
            t
        );
    }
}

#[test]
fn c9_contract_evaluator_evaluate_pass_on_all_pass_evidence() {
    let ev = AutoAdvanceEvaluator::default();
    let v = ev.evaluate(&FsmTransition::AutoAdvance, &all_pass());
    assert_eq!(v, EvidenceVerdict::Pass);
}

#[test]
fn c9_contract_evaluator_evaluate_fail_propagates_reason() {
    let ev = AutoAdvanceEvaluator::default();
    let mut e = all_pass();
    e.zero_ungrounded = false;
    match ev.evaluate(&FsmTransition::AutoAdvance, &e) {
        EvidenceVerdict::Fail { reason } => {
            assert!(reason.contains("zero_ungrounded"));
        }
        other => panic!("expected Fail, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Contract 3 — EscalationTrigger extended taxonomy (6 variants)
// ---------------------------------------------------------------------------

#[test]
fn c9_contract_escalation_trigger_has_six_variants() {
    // Compile-time + runtime check of the label round-trip.
    assert_eq!(EscalationTrigger::AntiSpiral.as_str(), "anti_spiral");
    assert_eq!(EscalationTrigger::HighRisk.as_str(), "high_risk");
    assert_eq!(EscalationTrigger::ReplanMajor.as_str(), "replan_major");
    assert_eq!(EscalationTrigger::CircuitBreaker.as_str(), "circuit_breaker");
    assert_eq!(
        EscalationTrigger::SubstituteModelFailed.as_str(),
        "substitute_model_failed"
    );
    assert_eq!(
        EscalationTrigger::SplitTaskFailed.as_str(),
        "split_task_failed"
    );
}

// ---------------------------------------------------------------------------
// Contract 4 — SubstituteModelFailed when fallback chain exhausted
// ---------------------------------------------------------------------------

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
    let d = gate.decide(&ctx).await;
    assert_eq!(
        d,
        AutonomyDecision::Escalate {
            trigger: EscalationTrigger::SubstituteModelFailed,
        }
    );
}

// ---------------------------------------------------------------------------
// Contract 5 — SubstituteModelFailed when target won't fit in VRAM
// ---------------------------------------------------------------------------

#[tokio::test]
async fn c9_contract_substitute_model_failed_when_target_oversized() {
    // s-68264 audit cleanup: AutonomousGateConfig.estimated_model_sizes is FxHashMap<String,u64>.
    let mut sizes: rustc_hash::FxHashMap<String, u64> = rustc_hash::FxHashMap::default();
    sizes.insert("huge".to_owned(), 24 * 1024 * 1024 * 1024); // 24 GiB
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
    let d = gate.decide(&ctx).await;
    assert_eq!(
        d,
        AutonomyDecision::Escalate {
            trigger: EscalationTrigger::SubstituteModelFailed,
        }
    );
}

// ---------------------------------------------------------------------------
// Contract 6 — SubstituteModel succeeds when target fits
// ---------------------------------------------------------------------------

#[tokio::test]
async fn c9_contract_substitute_model_succeeds_when_target_fits() {
    // s-68264 audit cleanup: AutonomousGateConfig.estimated_model_sizes is FxHashMap<String,u64>.
    let mut sizes: rustc_hash::FxHashMap<String, u64> = rustc_hash::FxHashMap::default();
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

// ---------------------------------------------------------------------------
// Contract 7 — SplitTaskPlan + AutonomyDecision::SplitTask carrier
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Contract 8 — GateMode::from_cli_args
// ---------------------------------------------------------------------------

#[test]
fn c9_contract_gate_mode_from_cli_args() {
    assert_eq!(GateMode::from_cli_args(None), GateMode::Interactive);
    assert_eq!(
        GateMode::from_cli_args(Some("autonomous")),
        GateMode::Autonomous
    );
    assert_eq!(
        GateMode::from_cli_args(Some("interactive")),
        GateMode::Interactive
    );
    // hybrid resolves to Interactive at the gate tier (hybrid is enforced
    // at the dispatcher).
    assert_eq!(
        GateMode::from_cli_args(Some("hybrid")),
        GateMode::Interactive
    );
    // Unknown labels fall back to default.
    assert_eq!(
        GateMode::from_cli_args(Some("garbage")),
        GateMode::Interactive
    );
    // Whitespace + casing are tolerated.
    assert_eq!(
        GateMode::from_cli_args(Some("  AUTONOMOUS  ")),
        GateMode::Autonomous
    );
}

// ---------------------------------------------------------------------------
// Contract 9 — ResearchTree::split_step
// ---------------------------------------------------------------------------

#[test]
fn c9_contract_research_tree_split_step_realises_plan() {
    use elai_core::clock::OsClock;
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let plan = ResearchPlan {
        root_question: "test question".into(),
        sub_questions: Vec::new(),
        complexity: QueryComplexity::Moderate,
        time_budget: std::time::Duration::from_secs(60),
        require_contradictions: false,
    };
    let tree = ResearchTree::new(plan, true, clock).expect("ResearchTree::new");
    let split_plan = SplitTaskPlan::deterministic("step-x", 3, 1);
    let realised = tree.split_step(&split_plan);
    assert_eq!(realised.len(), 3);
    for (i, sub) in realised.iter().enumerate() {
        // Re-stamped to the parent_phase prefix; preserved scope/budget.
        assert!(sub.task_id.contains("step-x"));
        assert_eq!(
            sub.task_id,
            format!("step-x.sub{i}"),
            "split_step must re-stamp task ids to parent_phase prefix"
        );
    }
}

#[test]
fn c9_contract_research_tree_split_step_empty_on_empty_plan() {
    use elai_core::clock::OsClock;
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let plan = ResearchPlan {
        root_question: "q".into(),
        sub_questions: Vec::new(),
        complexity: QueryComplexity::Moderate,
        time_budget: std::time::Duration::from_secs(60),
        require_contradictions: false,
    };
    let tree = ResearchTree::new(plan, true, clock).expect("ResearchTree::new");
    let empty_plan = SplitTaskPlan {
        parent_phase: "p".into(),
        sub_tasks: Vec::new(),
        merge_strategy: MergeStrategy::Sequential,
        max_depth: 2,
        current_depth: 0,
    };
    let realised = tree.split_step(&empty_plan);
    assert!(realised.is_empty(), "empty plan must return empty Vec");
}

#[test]
fn c9_contract_research_tree_split_step_caps_at_max_depth() {
    use elai_core::clock::OsClock;
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let plan = ResearchPlan {
        root_question: "q".into(),
        sub_questions: Vec::new(),
        complexity: QueryComplexity::Moderate,
        time_budget: std::time::Duration::from_secs(60),
        require_contradictions: false,
    };
    let tree = ResearchTree::new(plan, true, clock).expect("ResearchTree::new");
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
    let realised = tree.split_step(&at_cap);
    assert!(realised.is_empty(), "at max_depth: split_step must refuse");
}

// ---------------------------------------------------------------------------
// Contract 10 — apply_task_runner_wiring installs AutonomousGate
// ---------------------------------------------------------------------------

#[tokio::test]
async fn c9_contract_apply_task_runner_wiring_installs_gate() {
    // Build a minimal RuntimeWiring in a tempdir (mirrors
    // agent_pool_executor_wiring_contract::build_minimal_wiring shape).
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
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
    let clock_arc: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(elai_core::clock::OsClock);
    let ssb = Arc::new(elai_orchestrator::Ssb::new(ssb_config, clock_arc).expect("Ssb::new"));
    let feasibility_prober =
        Arc::new(elai_orchestrator::DefaultFeasibilityProber::new(root.join("feasibility")));
    let layer1a_processor = Arc::new(elai_orchestrator::DefaultLayer1aProcessor::new(
        rules_db,
        rules_db_path,
        ssb.clone(),
    ));

    // Case A: gate_mode = None preserves the legacy
    // `autonomous_gate: None` invariant.
    let wiring_none = RuntimeWiring {
        ssb: ssb.clone(),
        feasibility_prober: feasibility_prober.clone(),
        layer1a_processor: layer1a_processor.clone(),
        baby_step_executor: None,
        gate_mode: None,
        cloud_config: None,
        provider_override: None,
        hook_dispatcher: None,
        // C-25 audit follow-up: tests at the pre-C-25 layer pass None so
        // the legacy "no crystallizer" path remains exercised.
        crystallizer: None,
        compiled_pipeline_store: None,
        background_task_manager: None,
        via_mcp: false,
    };
    let cfg_none = apply_task_runner_wiring(root.join("cursor.json"), &wiring_none);
    assert!(
        cfg_none.autonomous_gate.is_none(),
        "gate_mode=None must yield autonomous_gate=None"
    );

    // Case B: gate_mode = Some(Interactive) installs a real gate.
    let wiring_inter = RuntimeWiring {
        ssb: ssb.clone(),
        feasibility_prober: feasibility_prober.clone(),
        layer1a_processor: layer1a_processor.clone(),
        baby_step_executor: None,
        gate_mode: Some(GateMode::Interactive),
        cloud_config: None,
        provider_override: None,
        hook_dispatcher: None,
        crystallizer: None,
        compiled_pipeline_store: None,
        background_task_manager: None,
        via_mcp: false,
    };
    let cfg_inter = apply_task_runner_wiring(root.join("cursor.json"), &wiring_inter);
    let gate = cfg_inter
        .autonomous_gate
        .as_ref()
        .expect("autonomous_gate must be installed");
    assert_eq!(gate.mode().await, GateMode::Interactive);

    // Case C: gate_mode = Some(Autonomous) installs a gate in Autonomous mode.
    let wiring_auto = RuntimeWiring {
        ssb,
        feasibility_prober,
        layer1a_processor,
        baby_step_executor: None,
        gate_mode: Some(GateMode::Autonomous),
        cloud_config: None,
        provider_override: None,
        hook_dispatcher: None,
        crystallizer: None,
        compiled_pipeline_store: None,
        background_task_manager: None,
        via_mcp: false,
    };
    let cfg_auto = apply_task_runner_wiring(root.join("cursor.json"), &wiring_auto);
    let gate_auto = cfg_auto
        .autonomous_gate
        .as_ref()
        .expect("autonomous_gate must be installed");
    assert_eq!(gate_auto.mode().await, GateMode::Autonomous);
}

// ---------------------------------------------------------------------------
// Contract 11 — ModelPool::can_fit_llm default impl
// ---------------------------------------------------------------------------

#[test]
fn c9_contract_modelpool_can_fit_llm_default_uses_available_vram() {
    let pool = FixedVramPool(4 * 1024 * 1024 * 1024);
    assert!(pool.can_fit_llm(1024 * 1024 * 1024));
    assert!(pool.can_fit_llm(4 * 1024 * 1024 * 1024));
    assert!(!pool.can_fit_llm(8 * 1024 * 1024 * 1024));
}
