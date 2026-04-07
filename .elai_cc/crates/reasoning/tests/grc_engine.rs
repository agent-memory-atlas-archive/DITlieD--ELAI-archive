use elai_reasoning::{
    AgentGrcConfig, BasicGrcEngine, GrcBudget, GrcEngine, GrcError, GrcState, ThinkStep,
};

// Helper: build a ThinkStep with minimal boilerplate.
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

// T1. Observe step with no grounding returns UngroundedStep.
// Invariant: reasoning.txt section 11 item 1.
#[test]
fn t01_observe_requires_grounding() {
    let mut engine = BasicGrcEngine::new();
    let result = engine.think(step(GrcState::Observe, "t", None));
    match result {
        Err(GrcError::UngroundedStep { step_type, topic }) => {
            assert_eq!(step_type, GrcState::Observe);
            assert_eq!(topic, "t");
        }
        other => panic!("expected UngroundedStep, got {:?}", other),
    }
}

// T2. Budget overflow forces convergence then exhaustion.
// Invariant: reasoning.txt section 11 item 2.
#[test]
fn t02_budget_overflow_forces_convergence() {
    let mut engine = BasicGrcEngine::new();
    engine.set_budget("t", 3, 2000);

    // Call 1: Observe (uses call 1, remaining=2)
    let r1 = engine.think(step(GrcState::Observe, "t", Some("file:src/a.rs")));
    assert!(r1.is_ok());
    assert_eq!(r1.unwrap().budget_remaining.remaining_calls(), 2);

    // Call 2: Hypothesize (uses call 2, remaining=1 -> must_converge)
    let r2 = engine.think(step(GrcState::Hypothesize, "t", Some("file:src/b.rs")));
    assert!(r2.is_ok());
    let r2 = r2.unwrap();
    assert_eq!(r2.budget_remaining.remaining_calls(), 1);
    assert!(r2.budget_remaining.must_converge());

    // Call 3 attempt: non-Converge step rejected (must converge)
    let r3 = engine.think(step(GrcState::PlanAction, "t", None));
    assert!(matches!(r3, Err(GrcError::BudgetExhausted { .. })));

    // Call 3: Converge step succeeds (uses call 3, remaining=0)
    let r4 = engine.think(step_with_content(
        GrcState::Converge,
        "t",
        None,
        "converging",
    ));
    assert!(r4.is_ok());
    assert_eq!(r4.unwrap().budget_remaining.remaining_calls(), 0);

    // Call 4 attempt: fully exhausted
    let r5 = engine.think(step(GrcState::Observe, "t", Some("file:src/c.rs")));
    assert!(matches!(r5, Err(GrcError::BudgetExhausted { .. })));
}

// T3. Per-topic budgets are independent.
#[test]
fn t03_per_topic_budget_independence() {
    let mut engine = BasicGrcEngine::new();
    engine.set_budget("a", 2, 2000);
    engine.set_budget("b", 3, 2000);

    // Exhaust topic "a": Observe then Converge (must_converge at remaining=1)
    engine
        .think(step(GrcState::Observe, "a", Some("file:src/x.rs")))
        .unwrap();
    engine
        .think(step_with_content(
            GrcState::Converge,
            "a",
            None,
            "done",
        ))
        .unwrap();

    // Topic "a" is exhausted
    let a_budget = engine.get_budget("a").unwrap();
    assert!(a_budget.is_exhausted());

    // Topic "b" still has full budget
    let b_budget = engine.get_budget("b").unwrap();
    assert_eq!(b_budget.remaining_calls(), 3);
}

// T4. Invalid transition Observe -> Verify (skipping Hypothesize, PlanAction, Act) rejected.
#[test]
fn t04_invalid_transition_rejected() {
    let mut engine = BasicGrcEngine::new();
    engine
        .think(step(GrcState::Observe, "t", Some("file:src/a.rs")))
        .unwrap();

    let result = engine.think(step(GrcState::Verify, "t", Some("test:passed")));
    match result {
        Err(GrcError::InvalidTransition { from, to, allowed }) => {
            assert_eq!(from, Some(GrcState::Observe));
            assert_eq!(to, GrcState::Verify);
            // Observe -> allowed: [Hypothesize, Converge, Status]
            assert!(allowed.contains(&GrcState::Hypothesize));
            assert!(!allowed.contains(&GrcState::Verify));
        }
        other => panic!("expected InvalidTransition, got {:?}", other),
    }
}

// T5. must_converge fires at remaining_calls <= 1, non-Converge rejected.
#[test]
fn t05_must_converge_at_budget_limit() {
    let mut engine = BasicGrcEngine::new();
    engine.set_budget("t", 3, 2000);

    // Use 2 calls
    engine
        .think(step(GrcState::Observe, "t", Some("file:src/a.rs")))
        .unwrap();
    engine
        .think(step(GrcState::Hypothesize, "t", Some("file:src/b.rs")))
        .unwrap();

    // remaining=1 -> must_converge
    let budget = engine.get_budget("t").unwrap();
    assert!(budget.must_converge());

    // Non-Converge step must be rejected
    let result = engine.think(step(GrcState::PlanAction, "t", None));
    assert!(matches!(result, Err(GrcError::BudgetExhausted { .. })));

    // Converge step must succeed
    let result = engine.think(step_with_content(
        GrcState::Converge,
        "t",
        None,
        "conclusion",
    ));
    assert!(result.is_ok());
}

// T6. Malformed grounding (no colon separator) rejected.
#[test]
fn t06_malformed_grounding_rejected() {
    let mut engine = BasicGrcEngine::new();
    let result = engine.think(step(GrcState::Observe, "t", Some("no_colon_no_type")));
    match result {
        Err(GrcError::MalformedGrounding { raw, reason }) => {
            assert_eq!(raw, "no_colon_no_type");
            assert!(reason.contains("':'"));
        }
        other => panic!("expected MalformedGrounding, got {:?}", other),
    }
}

// T7. Status step is free (does not consume budget).
// reasoning.txt section 2: status is a free echo.
#[test]
fn t07_status_step_free() {
    let mut engine = BasicGrcEngine::new();
    engine.set_budget("t", 2, 2000);

    let result = engine.think(step(GrcState::Status, "t", None));
    assert!(result.is_ok());
    let budget = engine.get_budget("t").unwrap();
    assert_eq!(budget.calls_used, 0);
    assert_eq!(budget.chars_used, 0);
}

// T8. Default budget applied when think() called without prior set_budget().
#[test]
fn t08_default_budget_applied_on_first_think() {
    let mut engine = BasicGrcEngine::new();
    // No set_budget call — engine should apply default (5 calls, 800 chars).
    let result = engine.think(step(GrcState::Observe, "new_topic", Some("file:src/a.rs")));
    assert!(result.is_ok());

    let budget = engine.get_budget("new_topic").unwrap();
    assert_eq!(budget.max_calls, 5);
    assert_eq!(budget.max_chars, 800);
}

// T9. First step in a new topic must be Observe (Hypothesize rejected).
// Exception: Status and Converge are allowed as first step.
#[test]
fn t09_first_step_must_be_observe() {
    let mut engine = BasicGrcEngine::new();

    // Hypothesize as first step: rejected
    let result = engine.think(step(
        GrcState::Hypothesize,
        "t",
        Some("file:src/a.rs"),
    ));
    match result {
        Err(GrcError::InvalidTransition { from, to, .. }) => {
            assert_eq!(from, None); // no prior state
            assert_eq!(to, GrcState::Hypothesize);
        }
        other => panic!("expected InvalidTransition with from=None, got {:?}", other),
    }

    // Status as first step: allowed (free echo)
    let mut engine2 = BasicGrcEngine::new();
    let result = engine2.think(step(GrcState::Status, "t", None));
    assert!(result.is_ok());

    // Converge as first step: allowed (forced convergence path)
    let mut engine3 = BasicGrcEngine::new();
    let result = engine3.think(step_with_content(
        GrcState::Converge,
        "t",
        None,
        "immediate convergence",
    ));
    assert!(result.is_ok());
}

// T10. Scope enumeration present when required — warns on unaddressed items.
// Invariant: reasoning.txt section 10, technical-analysis-v3.txt Concern 3.
#[test]
fn t10_scope_enumeration_present_when_required() {
    let config = AgentGrcConfig {
        require_scope_enumeration: true,
    };
    let mut engine = BasicGrcEngine::with_config(config);
    engine.set_budget("t", 10, 5000);

    // Observe with scope declared
    let observe = step_with_scope(
        GrcState::Observe,
        "t",
        Some("file:src/a.rs"),
        "observing the task",
        Some(vec!["ui", "api", "tests"]),
    );
    let r1 = engine.think(observe);
    assert!(r1.is_ok());
    assert!(r1.unwrap().warnings.is_empty());

    // Hypothesize (required for valid transition)
    engine
        .think(step(GrcState::Hypothesize, "t", Some("file:src/b.rs")))
        .unwrap();

    // PlanAction addressing only "ui" and "api" — missing "tests"
    let plan = step_with_content(
        GrcState::PlanAction,
        "t",
        None,
        "plan: implement ui changes and api endpoints",
    );
    let r2 = engine.think(plan);
    assert!(r2.is_ok());
    let warnings = r2.unwrap().warnings;
    assert!(
        warnings.iter().any(|w| w.contains("tests")),
        "expected warning about unaddressed scope item 'tests', got: {:?}",
        warnings
    );
}

// T11. Scope enumeration missing when required — produces warning.
#[test]
fn t11_scope_enumeration_missing_when_required() {
    let config = AgentGrcConfig {
        require_scope_enumeration: true,
    };
    let mut engine = BasicGrcEngine::with_config(config);

    // Observe with scope=None (required but missing)
    let observe = step_with_scope(
        GrcState::Observe,
        "t",
        Some("file:src/a.rs"),
        "observing without scope",
        None,
    );
    let result = engine.think(observe);
    // Should succeed with a warning (not an error, per implementation)
    assert!(result.is_ok());
    let warnings = result.unwrap().warnings;
    assert!(
        warnings.iter().any(|w| w.contains("scope")),
        "expected warning about missing scope enumeration, got: {:?}",
        warnings
    );
}

// T12. Scope enumeration optional when not required — no scope warnings.
#[test]
fn t12_scope_enumeration_optional_when_not_required() {
    let config = AgentGrcConfig {
        require_scope_enumeration: false,
    };
    let mut engine = BasicGrcEngine::with_config(config);

    // Observe with scope=None and require_scope_enumeration=false
    let observe = step_with_scope(
        GrcState::Observe,
        "t",
        Some("file:src/a.rs"),
        "observing without scope",
        None,
    );
    let result = engine.think(observe);
    assert!(result.is_ok());
    let warnings = result.unwrap().warnings;
    let scope_warnings: Vec<_> = warnings.iter().filter(|w| w.contains("scope")).collect();
    assert!(
        scope_warnings.is_empty(),
        "expected no scope-related warnings, got: {:?}",
        scope_warnings
    );
}

// ---------------------------------------------------------------------------
// Adversarial tests
// ---------------------------------------------------------------------------

// A1. Observe.allowed_next() must NOT contain Verify (guards invalid skip).
#[test]
fn adversarial_observe_does_not_allow_verify() {
    let allowed = GrcState::Observe.allowed_next();
    assert!(
        !allowed.contains(&GrcState::Verify),
        "Observe.allowed_next() must not contain Verify"
    );
}

// A2. Budget remaining_calls never wraps to u32::MAX on overflow.
#[test]
fn adversarial_budget_no_underflow() {
    let budget = GrcBudget {
        max_calls: 3,
        max_chars: 100,
        calls_used: 5, // exceeds max
        chars_used: 200,
    };
    assert_eq!(budget.remaining_calls(), 0);
    assert_eq!(budget.remaining_chars(), 0);
    assert!(budget.is_exhausted());
}

// A3. reset_topic fully removes entry (no zombie state).
#[test]
fn adversarial_reset_topic_removes_entry() {
    let mut engine = BasicGrcEngine::new();
    engine.set_budget("t", 5, 2000);
    engine
        .think(step(GrcState::Observe, "t", Some("file:src/a.rs")))
        .unwrap();

    engine.reset_topic("t");
    assert!(engine.get_budget("t").is_none());
}

// A4. No timing-dependent code in tests (compile-time assertion via grep in CI).
// This is validated by the deterministic check command, not a runtime test.
// Included here as documentation that the constraint was considered.

// A5. Stubs for phase 2B methods use unimplemented!(), not placeholder Ok.
// Signature updated per D6, D7 (verify_claim now takes 3 args, ground_plan returns PlanGroundingResult).
#[test]
#[should_panic(expected = "not yet implemented")]
fn adversarial_verify_claim_is_stub() {
    let engine = BasicGrcEngine::new();
    let _ = engine.verify_claim("claim", "source", "context");
}

#[test]
#[should_panic(expected = "not yet implemented")]
fn adversarial_ground_plan_is_stub() {
    let engine = BasicGrcEngine::new();
    let _ = engine.ground_plan("plan");
}

// A6. Chars budget checked before increment — verify no state corruption on error.
#[test]
fn adversarial_chars_budget_no_state_corruption() {
    let mut engine = BasicGrcEngine::new();
    engine.set_budget("t", 10, 20); // tight char budget

    // First step: short content, fits
    let r1 = engine.think(step_with_content(
        GrcState::Observe,
        "t",
        Some("file:src/a.rs"),
        "ok",
    ));
    assert!(r1.is_ok());

    // Second step: content exceeds remaining chars
    let long_content = "x".repeat(30);
    let r2 = engine.think(step_with_content(
        GrcState::Hypothesize,
        "t",
        Some("file:src/b.rs"),
        &long_content,
    ));
    assert!(matches!(r2, Err(GrcError::BudgetExhausted { .. })));

    // Budget state should not be corrupted — calls_used should still be 1
    let budget = engine.get_budget("t").unwrap();
    assert_eq!(budget.calls_used, 1, "calls_used should not increment on error");
}
