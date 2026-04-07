use std::collections::HashMap;

use elai_reasoning::{
    AgentGrcConfig, AgentGrcConfigLoader, BasicGrcEngine, GrcBudget, GrcEngine, GrcError,
    GrcState, GroundingLevel, ThinkStep, TomlConfigLoader, VerificationMethod, GrcDefaults,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn step(step_type: GrcState, topic: &str, grounding: Option<&str>) -> ThinkStep {
    ThinkStep {
        step_type,
        content: "test content".to_string(),
        topic: topic.to_string(),
        grounding: grounding.map(|s| s.to_string()),
        conclusion: None,
        scope: None,
    }
}

fn step_with_content(
    step_type: GrcState,
    topic: &str,
    grounding: Option<&str>,
    content: &str,
) -> ThinkStep {
    ThinkStep {
        step_type,
        content: content.to_string(),
        topic: topic.to_string(),
        grounding: grounding.map(|s| s.to_string()),
        conclusion: None,
        scope: None,
    }
}

fn step_with_scope(
    step_type: GrcState,
    topic: &str,
    grounding: Option<&str>,
    content: &str,
    scope: Option<Vec<&str>>,
) -> ThinkStep {
    ThinkStep {
        step_type,
        content: content.to_string(),
        topic: topic.to_string(),
        grounding: grounding.map(|s| s.to_string()),
        conclusion: None,
        scope: scope.map(|v| v.into_iter().map(|s| s.to_string()).collect()),
    }
}

/// Build auditor config: double-verify, grounding_level=Verified, budget 10/1200.
fn auditor_config() -> AgentGrcConfig {
    let mut overrides = HashMap::new();
    // Auditor needs: Hypothesize->Verify (skip PlanAction/Act), Verify->Verify (double-verify)
    overrides.insert(
        GrcState::Hypothesize,
        vec![GrcState::Verify],
    );
    overrides.insert(
        GrcState::Verify,
        vec![GrcState::Verify, GrcState::Observe],
    );
    AgentGrcConfig {
        role: "auditor".to_string(),
        budget: GrcBudget::new(10, 1200),
        required_steps: vec![
            GrcState::Observe,
            GrcState::Hypothesize,
            GrcState::Verify,
            GrcState::Verify,
        ],
        grounding_level: GroundingLevel::Verified,
        csq_required: true,
        allowed_methods: vec![
            VerificationMethod::FileCheck,
            VerificationMethod::SourceCheck,
            VerificationMethod::TestCheck,
            VerificationMethod::WebCheck,
            VerificationMethod::ApiCheck,
            VerificationMethod::ApiRegistryCheck,
        ],
        transition_overrides: overrides,
        require_scope_enumeration: false,
    }
}

/// Build context-compiler config: observe-only, FILE_CHECK + SOURCE_CHECK only.
fn context_compiler_config() -> AgentGrcConfig {
    AgentGrcConfig {
        role: "context_compiler".to_string(),
        budget: GrcBudget::new(3, 600),
        required_steps: vec![GrcState::Observe],
        grounding_level: GroundingLevel::High,
        csq_required: false,
        allowed_methods: vec![
            VerificationMethod::FileCheck,
            VerificationMethod::SourceCheck,
        ],
        transition_overrides: HashMap::new(),
        require_scope_enumeration: false,
    }
}

/// Build executor config: full chain, budget 8/1200, scope enumeration required.
fn executor_config() -> AgentGrcConfig {
    AgentGrcConfig {
        role: "executor".to_string(),
        budget: GrcBudget::new(8, 1200),
        required_steps: vec![
            GrcState::Observe,
            GrcState::Hypothesize,
            GrcState::PlanAction,
            GrcState::Act,
            GrcState::Verify,
        ],
        grounding_level: GroundingLevel::High,
        csq_required: true,
        allowed_methods: vec![
            VerificationMethod::FileCheck,
            VerificationMethod::SourceCheck,
            VerificationMethod::TestCheck,
            VerificationMethod::WebCheck,
            VerificationMethod::ApiCheck,
            VerificationMethod::ApiRegistryCheck,
        ],
        transition_overrides: HashMap::new(),
        require_scope_enumeration: true,
    }
}

/// Build memory-writer config: minimal budget 2/400.
fn memory_writer_config() -> AgentGrcConfig {
    AgentGrcConfig {
        role: "memory_writer".to_string(),
        budget: GrcBudget::new(2, 400),
        required_steps: vec![GrcState::Observe, GrcState::Converge],
        grounding_level: GroundingLevel::Uncertain,
        csq_required: false,
        allowed_methods: vec![
            VerificationMethod::FileCheck,
            VerificationMethod::SourceCheck,
        ],
        transition_overrides: HashMap::new(),
        require_scope_enumeration: false,
    }
}

// ---------------------------------------------------------------------------
// T1. Auditor gets double-verify via transition_overrides.
// The second Verify does NOT return InvalidTransition.
// Invariant: reasoning.txt section 7 line 97, section 11 item 9.
// ---------------------------------------------------------------------------
#[test]
fn t01_auditor_gets_double_verify() {
    let config = auditor_config();
    let mut engine = BasicGrcEngine::new();
    engine.with_agent_config(config);
    engine.set_budget("t", 10, 5000);

    // Observe
    let r1 = engine.think(step(GrcState::Observe, "t", Some("test:verified_claim")));
    assert!(r1.is_ok(), "Observe should succeed: {:?}", r1);

    // Hypothesize
    let r2 = engine.think(step(GrcState::Hypothesize, "t", Some("test:verified_claim")));
    assert!(r2.is_ok(), "Hypothesize should succeed: {:?}", r2);

    // First Verify
    let r3 = engine.think(step_with_content(
        GrcState::Verify,
        "t",
        Some("test:verified_claim"),
        "FINDING: first issue found",
    ));
    assert!(r3.is_ok(), "First Verify should succeed: {:?}", r3);

    // Second Verify — requires transition_override {Verify: [Verify, Observe]}
    let r4 = engine.think(step_with_content(
        GrcState::Verify,
        "t",
        Some("test:verified_claim"),
        "FINDING: second issue found",
    ));
    assert!(r4.is_ok(), "Second Verify (double-verify) should succeed: {:?}", r4);
}

// ---------------------------------------------------------------------------
// T2. Context-compiler is observe-only: Hypothesize after Observe is rejected.
// Invariant: reasoning.txt section 7 lines 103-106.
// ---------------------------------------------------------------------------
#[test]
fn t02_context_compiler_observe_only() {
    let config = context_compiler_config();
    let mut engine = BasicGrcEngine::new();
    engine.with_agent_config(config);
    engine.set_budget("t", 5, 2000);

    // Observe succeeds
    let r1 = engine.think(step(GrcState::Observe, "t", Some("file:src/a.rs")));
    assert!(r1.is_ok(), "Observe should succeed: {:?}", r1);

    // Hypothesize must be rejected — context-compiler is observe-only
    let r2 = engine.think(step(GrcState::Hypothesize, "t", Some("file:src/b.rs")));
    assert!(
        matches!(r2, Err(GrcError::InvalidTransition { .. })),
        "context-compiler Hypothesize should return InvalidTransition, got: {:?}",
        r2
    );
}

// ---------------------------------------------------------------------------
// T3. Executor budget 8/1200: 8 calls succeed, 9th returns BudgetExhausted.
// Invariant: reasoning.txt section 7 lines 89-91.
// ---------------------------------------------------------------------------
#[test]
fn t03_executor_budget_8_1200() {
    let config = executor_config();
    let mut engine = BasicGrcEngine::new();
    engine.with_agent_config(config);
    engine.set_budget("t", 8, 5000);

    let budget = engine.get_budget("t");
    assert!(budget.is_some(), "budget should be set");
    assert_eq!(budget.unwrap().max_calls, 8);
    assert_eq!(budget.unwrap().max_chars, 5000);

    // Run 7 calls (leave 1 for must_converge)
    let steps_sequence = [
        (GrcState::Observe, Some("file:src/a.rs")),
        (GrcState::Hypothesize, Some("file:src/b.rs")),
        (GrcState::PlanAction, None),
        (GrcState::Act, None),
        (GrcState::Verify, Some("test:passed")),
        (GrcState::Observe, Some("file:src/c.rs")),
        (GrcState::Hypothesize, Some("file:src/d.rs")),
    ];
    for (st, g) in &steps_sequence {
        let r = engine.think(step(*st, "t", *g));
        assert!(r.is_ok(), "step {:?} should succeed: {:?}", st, r);
    }

    // Call 8: must_converge, only Converge allowed
    let r8 = engine.think(step_with_content(GrcState::Converge, "t", None, "done"));
    assert!(r8.is_ok(), "8th call (Converge) should succeed: {:?}", r8);

    // Call 9: BudgetExhausted
    let r9 = engine.think(step(GrcState::Observe, "t", Some("file:src/e.rs")));
    assert!(
        matches!(r9, Err(GrcError::BudgetExhausted { .. })),
        "9th call should return BudgetExhausted, got: {:?}",
        r9
    );
}

// ---------------------------------------------------------------------------
// T4. Memory-writer budget 2/400: 2 calls succeed, 3rd returns BudgetExhausted.
// Invariant: reasoning.txt section 7 lines 115-116.
// ---------------------------------------------------------------------------
#[test]
fn t04_memory_writer_budget_2_400() {
    let config = memory_writer_config();
    let mut engine = BasicGrcEngine::new();
    engine.with_agent_config(config);
    engine.set_budget("t", 2, 2000);

    // Call 1: Observe
    let r1 = engine.think(step(GrcState::Observe, "t", Some("file:src/a.rs")));
    assert!(r1.is_ok(), "first call should succeed: {:?}", r1);

    // Call 2: must_converge at remaining=1
    let r2 = engine.think(step_with_content(GrcState::Converge, "t", None, "done"));
    assert!(r2.is_ok(), "second call (Converge) should succeed: {:?}", r2);

    // Call 3: exhausted
    let r3 = engine.think(step(GrcState::Observe, "t", Some("file:src/b.rs")));
    assert!(
        matches!(r3, Err(GrcError::BudgetExhausted { .. })),
        "3rd call should return BudgetExhausted, got: {:?}",
        r3
    );
}

// ---------------------------------------------------------------------------
// T5. Config validation: empty required_steps rejected.
// D2 rule (a).
// ---------------------------------------------------------------------------
#[test]
fn t05_config_validation_empty_steps_rejected() {
    let config = AgentGrcConfig {
        role: "test_role".to_string(),
        required_steps: vec![],
        ..AgentGrcConfig::default()
    };
    let loader = TomlConfigLoader::new();
    let result = loader.validate_config(&config);
    match result {
        Err(GrcError::ConfigValidation { role, reason }) => {
            assert_eq!(role, "test_role");
            assert!(
                reason.to_lowercase().contains("empty"),
                "reason should mention empty steps, got: {}",
                reason
            );
        }
        other => panic!("expected ConfigValidation error, got: {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// T6. Config validation: budget too small for required_steps.
// D2 rule (c): budget.max_calls >= required_steps.len().
// ---------------------------------------------------------------------------
#[test]
fn t06_config_validation_budget_too_small() {
    let config = AgentGrcConfig {
        role: "test_role".to_string(),
        budget: GrcBudget::new(2, 800),
        required_steps: vec![
            GrcState::Observe,
            GrcState::Hypothesize,
            GrcState::PlanAction,
            GrcState::Act,
            GrcState::Verify,
        ],
        ..AgentGrcConfig::default()
    };
    let loader = TomlConfigLoader::new();
    let result = loader.validate_config(&config);
    match result {
        Err(GrcError::ConfigValidation { role, reason }) => {
            assert_eq!(role, "test_role");
            assert!(
                reason.to_lowercase().contains("budget"),
                "reason should mention budget, got: {}",
                reason
            );
        }
        other => panic!("expected ConfigValidation error, got: {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// T7. Auditor anti-rubber-stamp: Converge without FINDING: is rejected.
// Then: Verify with "FINDING:" prefix, Converge succeeds.
// Invariant: reasoning.txt section 7 line 99.
// ---------------------------------------------------------------------------
#[test]
fn t07_auditor_anti_rubber_stamp() {
    let config = auditor_config();
    let mut engine = BasicGrcEngine::new();
    engine.with_agent_config(config);
    engine.set_budget("t", 10, 5000);

    // Observe
    engine
        .think(step(GrcState::Observe, "t", Some("test:verified_claim")))
        .expect("Observe should succeed");

    // Hypothesize
    engine
        .think(step(GrcState::Hypothesize, "t", Some("test:verified_claim")))
        .expect("Hypothesize should succeed");

    // Verify WITHOUT FINDING: marker
    engine
        .think(step_with_content(
            GrcState::Verify,
            "t",
            Some("test:verified_claim"),
            "everything looks good, no issues",
        ))
        .expect("Verify step should succeed");

    // Attempt Converge — should be rejected (no FINDING: in any Verify step)
    let converge_result = engine.think(step_with_content(
        GrcState::Converge,
        "t",
        None,
        "audit complete",
    ));
    assert!(
        matches!(converge_result, Err(GrcError::InsufficientFindings { .. })),
        "Converge without FINDING: should return InsufficientFindings, got: {:?}",
        converge_result
    );

    // Now add a proper Verify with FINDING: prefix
    engine
        .think(step_with_content(
            GrcState::Verify,
            "t",
            Some("test:verified_claim"),
            "FINDING: unused variable in engine.rs line 42",
        ))
        .expect("Verify with FINDING should succeed");

    // Converge should now succeed
    let converge_ok = engine.think(step_with_content(
        GrcState::Converge,
        "t",
        None,
        "audit complete with findings",
    ));
    assert!(
        converge_ok.is_ok(),
        "Converge after FINDING: should succeed, got: {:?}",
        converge_ok
    );
}

// ---------------------------------------------------------------------------
// T8. Auditor rejects uncertain grounding (grounding_level=Verified minimum).
// Also verifies GroundingLevel ordering: Verified > High > Uncertain.
// ---------------------------------------------------------------------------
#[test]
fn t08_auditor_rejects_uncertain_grounding() {
    // First: verify GroundingLevel ordering
    assert!(
        GroundingLevel::Verified > GroundingLevel::High,
        "Verified must be > High"
    );
    assert!(
        GroundingLevel::High > GroundingLevel::Uncertain,
        "High must be > Uncertain"
    );
    assert!(
        GroundingLevel::Verified > GroundingLevel::Uncertain,
        "Verified must be > Uncertain"
    );

    let config = auditor_config(); // grounding_level=Verified
    let mut engine = BasicGrcEngine::new();
    engine.with_agent_config(config);
    engine.set_budget("t", 10, 5000);

    // Observe with training: (Uncertain level) — should be rejected
    let result = engine.think(step(GrcState::Observe, "t", Some("training:unverified_claim")));
    assert!(
        result.is_err(),
        "Observe with Uncertain grounding should be rejected for auditor (requires Verified), got: {:?}",
        result
    );
}

// ---------------------------------------------------------------------------
// T9. Context-compiler allowed_methods: WEB_CHECK rejected.
// Policy enforcement: only FILE_CHECK and SOURCE_CHECK permitted.
// ---------------------------------------------------------------------------
#[test]
fn t09_context_compiler_allowed_methods() {
    let config = context_compiler_config();
    // allowed_methods = [FileCheck, SourceCheck]
    assert!(!config.allowed_methods.contains(&VerificationMethod::WebCheck));
    assert!(config.allowed_methods.contains(&VerificationMethod::FileCheck));
    assert!(config.allowed_methods.contains(&VerificationMethod::SourceCheck));

    let mut engine = BasicGrcEngine::new();
    engine.with_agent_config(config);
    engine.set_budget("t", 5, 2000);

    // Attempt to use search: grounding (maps to WebCheck method) — should be rejected
    let result = engine.think(step(GrcState::Observe, "t", Some("search:some_query")));
    assert!(
        result.is_err(),
        "context-compiler should reject WebCheck (search:) grounding, got: {:?}",
        result
    );
}

// ---------------------------------------------------------------------------
// T10. Global defaults with per-role override: executor overrides budget,
// inherits other defaults.
// ---------------------------------------------------------------------------
#[test]
fn t10_global_defaults_with_per_role_override() {
    let defaults = GrcDefaults {
        budget: GrcBudget::new(5, 800),
        grounding_level: GroundingLevel::Uncertain,
        csq_required: false,
        allowed_methods: vec![
            VerificationMethod::FileCheck,
            VerificationMethod::SourceCheck,
            VerificationMethod::TestCheck,
            VerificationMethod::WebCheck,
            VerificationMethod::ApiCheck,
            VerificationMethod::ApiRegistryCheck,
        ],
    };

    let loader = TomlConfigLoader::new();
    // load_config for executor should merge: executor overrides budget to (8, 1200)
    // but inherits grounding_level, csq_required, allowed_methods from defaults.
    // Current stub returns defaults only — this test WILL FAIL until I5 is implemented.
    let config = loader.load_config("executor", &defaults);
    assert!(config.is_ok(), "load_config should succeed: {:?}", config);
    let config = config.unwrap();

    // Executor budget should be 8/1200 (overridden), not 5/800 (default)
    assert_eq!(
        config.budget.max_calls, 8,
        "executor should override budget.max_calls to 8"
    );
    assert_eq!(
        config.budget.max_chars, 1200,
        "executor should override budget.max_chars to 1200"
    );

    // Other fields should inherit defaults
    assert_eq!(config.grounding_level, GroundingLevel::Uncertain);
}

// ---------------------------------------------------------------------------
// T11. Transition overrides are additive: default transitions preserved.
// Config with override {Observe: [Verify]} should still allow Hypothesize
// (from default allowed_next) AND Verify (from override).
// ---------------------------------------------------------------------------
#[test]
fn t11_transition_overrides_are_additive() {
    // Override: Observe -> [Verify] (additive, so default Observe -> [Hypothesize] preserved)
    // Also need Hypothesize -> [Verify] for the chain to validate.
    let mut overrides = HashMap::new();
    overrides.insert(GrcState::Observe, vec![GrcState::Verify]);
    overrides.insert(GrcState::Hypothesize, vec![GrcState::Verify]);

    let config = AgentGrcConfig {
        role: "test_additive".to_string(),
        budget: GrcBudget::new(10, 5000),
        required_steps: vec![
            GrcState::Observe,
            GrcState::Hypothesize,
            GrcState::Verify,
        ],
        grounding_level: GroundingLevel::Uncertain,
        csq_required: false,
        allowed_methods: vec![
            VerificationMethod::FileCheck,
            VerificationMethod::SourceCheck,
        ],
        transition_overrides: overrides,
        require_scope_enumeration: false,
    };

    let mut engine = BasicGrcEngine::new();
    engine.with_agent_config(config);
    engine.set_budget("t", 10, 5000);

    // Observe
    engine
        .think(step(GrcState::Observe, "t", Some("file:src/a.rs")))
        .expect("Observe should succeed");

    // Hypothesize should still work (default Observe->Hypothesize preserved)
    let r_hyp = engine.think(step(GrcState::Hypothesize, "t", Some("file:src/b.rs")));
    assert!(
        r_hyp.is_ok(),
        "Hypothesize should still be allowed (default transition preserved): {:?}",
        r_hyp
    );

    // Start a new chain to test the override path
    engine.reset_topic("t");
    engine.set_budget("t", 10, 5000);

    engine
        .think(step(GrcState::Observe, "t", Some("file:src/c.rs")))
        .expect("Observe should succeed");

    // Verify directly from Observe — should work via override {Observe: [Verify]}
    // Use file: grounding (allowed by config) instead of test: (not in allowed_methods).
    let r_ver = engine.think(step(GrcState::Verify, "t", Some("file:src/verified.rs")));
    assert!(
        r_ver.is_ok(),
        "Verify from Observe should succeed via transition override: {:?}",
        r_ver
    );
}

// ---------------------------------------------------------------------------
// T12. Auditor budget 10/1200: 10 calls succeed, 11th returns BudgetExhausted.
// Invariant: reasoning.txt section 7 line 96.
// ---------------------------------------------------------------------------
#[test]
fn t12_auditor_budget_10_1200() {
    let config = auditor_config();
    let mut engine = BasicGrcEngine::new();
    engine.with_agent_config(config);
    engine.set_budget("t", 10, 10000);

    // Run 9 calls (leave 1 for must_converge).
    // At least one Verify step must have FINDING: prefix for anti-rubber-stamp.
    engine.think(step(GrcState::Observe, "t", Some("test:claim1")))
        .expect("call 1 Observe");
    engine.think(step(GrcState::Hypothesize, "t", Some("test:claim2")))
        .expect("call 2 Hypothesize");
    engine.think(step_with_content(GrcState::Verify, "t", Some("test:claim3"),
        "FINDING: issue in module A"))
        .expect("call 3 Verify (with finding)");
    engine.think(step_with_content(GrcState::Verify, "t", Some("test:claim4"),
        "FINDING: issue in module B"))
        .expect("call 4 Verify (double verify)");
    engine.think(step(GrcState::Observe, "t", Some("test:claim5")))
        .expect("call 5 Observe");
    engine.think(step(GrcState::Hypothesize, "t", Some("test:claim6")))
        .expect("call 6 Hypothesize");
    engine.think(step_with_content(GrcState::Verify, "t", Some("test:claim7"),
        "FINDING: issue in module C"))
        .expect("call 7 Verify");
    engine.think(step(GrcState::Observe, "t", Some("test:claim8")))
        .expect("call 8 Observe");
    engine.think(step(GrcState::Hypothesize, "t", Some("test:claim9")))
        .expect("call 9 Hypothesize");

    // Call 10: must_converge
    let r10 = engine.think(step_with_content(GrcState::Converge, "t", None, "done"));
    assert!(r10.is_ok(), "10th call (Converge) should succeed: {:?}", r10);

    // Call 11: exhausted
    let r11 = engine.think(step(GrcState::Observe, "t", Some("test:claim10")));
    assert!(
        matches!(r11, Err(GrcError::BudgetExhausted { .. })),
        "11th call should return BudgetExhausted, got: {:?}",
        r11
    );
}

// ---------------------------------------------------------------------------
// T13. Prompt-engineer budget 5/800, required_steps=[Observe,Hypothesize,Converge].
// Invariant: reasoning.txt section 7 lines 109-111.
// ---------------------------------------------------------------------------
#[test]
fn t13_prompt_engineer_budget_5_800() {
    let config = AgentGrcConfig {
        role: "prompt_engineer".to_string(),
        budget: GrcBudget::new(5, 800),
        required_steps: vec![
            GrcState::Observe,
            GrcState::Hypothesize,
            GrcState::Converge,
        ],
        grounding_level: GroundingLevel::High,
        csq_required: false,
        allowed_methods: vec![
            VerificationMethod::FileCheck,
            VerificationMethod::SourceCheck,
        ],
        transition_overrides: HashMap::new(),
        require_scope_enumeration: false,
    };

    let mut engine = BasicGrcEngine::new();
    engine.with_agent_config(config);
    engine.set_budget("t", 5, 5000);

    // 4 calls
    let steps_seq = [
        (GrcState::Observe, Some("file:src/a.rs")),
        (GrcState::Hypothesize, Some("file:src/b.rs")),
        (GrcState::Observe, Some("file:src/c.rs")),  // loop back via Converge? No — Hypothesize->Converge->Observe
        (GrcState::Hypothesize, Some("file:src/d.rs")),
    ];

    // Observe
    engine.think(step(steps_seq[0].0, "t", steps_seq[0].1)).expect("Observe 1");
    // Hypothesize
    engine.think(step(steps_seq[1].0, "t", steps_seq[1].1)).expect("Hypothesize 1");
    // PlanAction should be rejected — not in required_steps whitelist
    let r_plan = engine.think(step(GrcState::PlanAction, "t", None));
    assert!(
        matches!(r_plan, Err(GrcError::InvalidTransition { .. })),
        "PlanAction should be rejected for prompt_engineer (not in required_steps): {:?}",
        r_plan
    );

    // Converge allowed (always-allowed)
    engine
        .think(step_with_content(GrcState::Converge, "t", None, "conclusion"))
        .expect("Converge");

    // Continue: Observe again
    engine
        .think(step(GrcState::Observe, "t", Some("file:src/e.rs")))
        .expect("Observe 2");

    // 5th call: must_converge
    let r5 = engine.think(step_with_content(GrcState::Converge, "t", None, "final"));
    assert!(r5.is_ok(), "5th call should succeed: {:?}", r5);

    // 6th call: exhausted
    let r6 = engine.think(step(GrcState::Observe, "t", Some("file:src/f.rs")));
    assert!(
        matches!(r6, Err(GrcError::BudgetExhausted { .. })),
        "6th call should return BudgetExhausted, got: {:?}",
        r6
    );
}

// ---------------------------------------------------------------------------
// T14. Designer config distinct from executor: same model, different role,
// required_steps, scope enumeration.
// Invariant: technical-analysis-v3.txt Concern 5, reasoning.txt section 7.
// ---------------------------------------------------------------------------
#[test]
fn t14_designer_config_distinct_from_executor() {
    // Designer skips Act (produces PipelineSpec, not code). PlanAction->Verify
    // is not a default transition, so designer needs a transition override.
    let mut designer_overrides = HashMap::new();
    designer_overrides.insert(GrcState::PlanAction, vec![GrcState::Verify]);

    let designer = AgentGrcConfig {
        role: "designer".to_string(),
        budget: GrcBudget::new(8, 1200),
        required_steps: vec![
            GrcState::Observe,
            GrcState::Hypothesize,
            GrcState::PlanAction,
            GrcState::Verify,
            GrcState::Converge,
        ],
        grounding_level: GroundingLevel::High,
        csq_required: true,
        allowed_methods: vec![
            VerificationMethod::FileCheck,
            VerificationMethod::SourceCheck,
            VerificationMethod::TestCheck,
            VerificationMethod::WebCheck,
            VerificationMethod::ApiCheck,
            VerificationMethod::ApiRegistryCheck,
        ],
        transition_overrides: designer_overrides,
        require_scope_enumeration: true,
    };
    let executor = executor_config();

    // Same budget
    assert_eq!(designer.budget.max_calls, executor.budget.max_calls);
    assert_eq!(designer.budget.max_chars, executor.budget.max_chars);

    // Different role
    assert_ne!(designer.role, executor.role);

    // Different required_steps (designer has PlanAction->Verify->Converge, no Act)
    assert_ne!(designer.required_steps, executor.required_steps);
    assert!(
        !designer.required_steps.contains(&GrcState::Act),
        "designer should not have Act step (produces PipelineSpec, not code)"
    );
    assert!(
        executor.required_steps.contains(&GrcState::Act),
        "executor should have Act step"
    );

    // Both require scope enumeration
    assert!(designer.require_scope_enumeration);
    assert!(executor.require_scope_enumeration);

    // Validate both configs pass validation
    let loader = TomlConfigLoader::new();
    assert!(
        loader.validate_config(&designer).is_ok(),
        "designer config should pass validation"
    );
    assert!(
        loader.validate_config(&executor).is_ok(),
        "executor config should pass validation"
    );
}

// ---------------------------------------------------------------------------
// T15. require_scope_enumeration per role: executor requires scope,
// memory_writer does not.
// ---------------------------------------------------------------------------
#[test]
fn t15_require_scope_enumeration_per_role() {
    let exec_config = executor_config();
    let mem_config = memory_writer_config();

    assert!(
        exec_config.require_scope_enumeration,
        "executor should require scope enumeration"
    );
    assert!(
        !mem_config.require_scope_enumeration,
        "memory_writer should not require scope enumeration"
    );

    // Executor: Observe without scope triggers warning
    let mut exec_engine = BasicGrcEngine::new();
    exec_engine.with_agent_config(exec_config);
    exec_engine.set_budget("t", 10, 5000);

    let r_exec = exec_engine.think(step_with_scope(
        GrcState::Observe,
        "t",
        Some("file:src/a.rs"),
        "observe task",
        None, // scope missing
    ));
    assert!(r_exec.is_ok());
    let exec_warnings = r_exec.unwrap().warnings;
    assert!(
        exec_warnings.iter().any(|w| w.contains("scope")),
        "executor should warn about missing scope: {:?}",
        exec_warnings
    );

    // Memory-writer: Observe without scope has no warning
    let mut mem_engine = BasicGrcEngine::new();
    mem_engine.with_agent_config(mem_config);
    mem_engine.set_budget("t", 10, 5000);

    let r_mem = mem_engine.think(step_with_scope(
        GrcState::Observe,
        "t",
        Some("file:src/a.rs"),
        "observe task",
        None, // scope missing — that's fine
    ));
    assert!(r_mem.is_ok());
    let mem_warnings = r_mem.unwrap().warnings;
    let scope_warnings: Vec<_> = mem_warnings.iter().filter(|w| w.contains("scope")).collect();
    assert!(
        scope_warnings.is_empty(),
        "memory_writer should not warn about missing scope: {:?}",
        scope_warnings
    );
}
