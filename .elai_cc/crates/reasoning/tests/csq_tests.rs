use elai_reasoning::{
    BasicGrcEngine, CsqCheck, CsqResult, CsqSeverity, FeasibilityOutcome, GrcEngine, GrcError,
    InlineFeasibilityState,
};

// ---------------------------------------------------------------------------
// T1: CSQ generates at least one check for a function touching files
// Invariant: reasoning.txt s6 "minimum 1 CSQ per function touching files."
// ---------------------------------------------------------------------------
#[test]
fn csq_generates_check_for_file_function() {
    let engine = BasicGrcEngine::new();
    let result = engine
        .generate_csq("write_config", &["std::fs::write".to_string()])
        .expect("generate_csq should not error");
    assert!(
        !result.checks.is_empty(),
        "CSQ must generate >= 1 check for file-touching function"
    );
    assert!(
        result
            .checks
            .iter()
            .any(|c| c.severity == CsqSeverity::Critical),
        "std::fs::write must produce at least one Critical check"
    );
}

// ---------------------------------------------------------------------------
// T2: Critical CSQ without guard fails
// Invariant: reasoning.txt s6 "dangerous consequence -> MUST add guard."
// ---------------------------------------------------------------------------
#[test]
fn csq_critical_without_guard_fails() {
    let result = CsqResult {
        function_id: "delete_user_data".to_string(),
        checks: vec![CsqCheck {
            assumption: "delete call succeeds".to_string(),
            consequence: "data loss if delete fails".to_string(),
            severity: CsqSeverity::Critical,
            guard_added: false,
        }],
    };
    assert!(
        !result.passed(),
        "CsqResult with Critical+no guard must fail"
    );
}

// ---------------------------------------------------------------------------
// T3: Critical CSQ with guard passes
// ---------------------------------------------------------------------------
#[test]
fn csq_critical_with_guard_passes() {
    let result = CsqResult {
        function_id: "delete_user_data".to_string(),
        checks: vec![CsqCheck {
            assumption: "delete call succeeds".to_string(),
            consequence: "data loss if delete fails".to_string(),
            severity: CsqSeverity::Critical,
            guard_added: true,
        }],
    };
    assert!(result.passed(), "CsqResult with Critical+guard must pass");
}

// ---------------------------------------------------------------------------
// T4: Warning CSQ without guard still passes (warnings don't block)
// ---------------------------------------------------------------------------
#[test]
fn csq_warning_without_guard_still_passes() {
    let result = CsqResult {
        function_id: "format_output".to_string(),
        checks: vec![CsqCheck {
            assumption: "formatter handles all edge cases".to_string(),
            consequence: "incorrect output possible".to_string(),
            severity: CsqSeverity::Warning,
            guard_added: false,
        }],
    };
    assert!(
        result.passed(),
        "CsqResult with Warning+no guard must still pass"
    );
}

// ---------------------------------------------------------------------------
// T5: No sensitive ops returns empty checks, passes
// ---------------------------------------------------------------------------
#[test]
fn csq_no_sensitive_ops_returns_empty() {
    let engine = BasicGrcEngine::new();
    let result = engine
        .generate_csq("helper_fn", &[])
        .expect("generate_csq with empty ops should not error");
    assert!(
        result.checks.is_empty(),
        "No sensitive ops = no CSQ checks"
    );
    assert!(
        result.passed(),
        "Empty checks = passed (no Critical violations)"
    );
}

// ---------------------------------------------------------------------------
// T6: Inline feasibility pass on known claim
// Note: realistic multi-word claim avoids trivial substring false positive.
// ---------------------------------------------------------------------------
#[test]
fn inline_feasibility_pass_on_known_claim() {
    let mut engine = BasicGrcEngine::new();
    let result = engine
        .check_inline_feasibility(
            "use tantivy BM25 index for full-text search",
            &["PipelineSpec: use tantivy BM25 index for full-text search".to_string()],
        )
        .expect("check_inline_feasibility should not error");
    assert_eq!(
        result.outcome,
        FeasibilityOutcome::Pass,
        "Claim present in sources must return Pass"
    );
}

// ---------------------------------------------------------------------------
// T7: Inline feasibility fail on unknown claim
// ---------------------------------------------------------------------------
#[test]
fn inline_feasibility_fail_on_unknown_claim() {
    let mut engine = BasicGrcEngine::new();
    let result = engine
        .check_inline_feasibility("unknown claim not in any source", &[])
        .expect("check_inline_feasibility should not error on unknown claim");
    assert_eq!(
        result.outcome,
        FeasibilityOutcome::Fail,
        "Claim absent from sources must return Fail"
    );
}

// ---------------------------------------------------------------------------
// T8: Inline feasibility max 2 calls enforced
// Invariant: reasoning.txt s8 "max 2 GRC calls."
// Invariant: reasoning.txt s11 "[2D] Inline feasibility max 2 calls (bounded)."
// ---------------------------------------------------------------------------
#[test]
fn inline_feasibility_max_two_calls() {
    let mut engine = BasicGrcEngine::new();
    // Initial state: 0 checks used
    assert_eq!(
        engine.inline_feasibility_state.checks_used, 0,
        "Initial checks_used must be 0"
    );

    // First call
    let _ = engine
        .check_inline_feasibility("claim one for feasibility test", &[])
        .expect("first inline check should succeed");

    // Second call
    let _ = engine
        .check_inline_feasibility("claim two for feasibility test", &[])
        .expect("second inline check should succeed");

    assert_eq!(
        engine.inline_feasibility_state.checks_used, 2,
        "After 2 calls, checks_used must be 2"
    );

    // Third call must fail with BudgetExhausted
    let err = engine
        .check_inline_feasibility("claim three should be rejected", &[])
        .expect_err("third inline check must return BudgetExhausted");
    assert!(
        matches!(err, GrcError::BudgetExhausted { .. }),
        "Expected BudgetExhausted, got: {:?}",
        err
    );
}

// ---------------------------------------------------------------------------
// T9: Inline feasibility blocked signals replan
// The CONTRADICTS_PLAN: prefix triggers Blocked detection (phase 2D convention).
// Invariant: reasoning.txt s8 "Fundamentally different approach requires
// EXECUTING -> BLOCK -> PLANNING."
// ---------------------------------------------------------------------------
#[test]
fn inline_feasibility_blocked_signals_replan() {
    let mut engine = BasicGrcEngine::new();
    let result = engine
        .check_inline_feasibility(
            "CONTRADICTS_PLAN: use completely different algorithm",
            &["use original algorithm".to_string()],
        )
        .expect("blocked check should return result, not error");
    assert_eq!(
        result.outcome,
        FeasibilityOutcome::Blocked,
        "CONTRADICTS_PLAN: prefix must trigger Blocked outcome"
    );
}

// ---------------------------------------------------------------------------
// T10: InlineFeasibilityState reset clears checks
// Ensures state is clean between phases/topics.
// ---------------------------------------------------------------------------
#[test]
fn inline_feasibility_state_reset() {
    let mut state = InlineFeasibilityState::default();
    // Exhaust the budget
    state.record_check();
    state.record_check();
    assert!(state.is_exhausted(), "After 2 checks, must be exhausted");
    assert_eq!(state.checks_used, 2);

    // Reset
    state.reset();
    assert_eq!(state.checks_used, 0, "After reset, checks_used must be 0");
    assert!(
        !state.is_exhausted(),
        "After reset, is_exhausted must be false"
    );
}
