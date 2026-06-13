// E-32: Contract assertions for escalation_policy module.
//
// Verifies the public surface: HandoffContext, HandoffDecision, EscalationPolicyError,
// constants, and threshold arithmetic. Pure compile-time + arithmetic checks;
// no sidecar or SSB required.

use elai_orchestrator::escalation_policy::{
    EscalationPolicyError, HandoffContext, HandoffDecision,
    DEEP_MODE_CTX_CAP, DEEP_MODE_MIN_FILES_REFERENCED, FALLBACK_EFFECTIVE_MAX_CONTEXT,
    HANDOFF_THRESHOLD_FRACTION,
};

// ---------------------------------------------------------------------------
// Constant values locked per E-32-decisions.md
// ---------------------------------------------------------------------------

#[test]
fn handoff_threshold_fraction_is_85_percent() {
    assert!(
        (HANDOFF_THRESHOLD_FRACTION - 0.85).abs() < f64::EPSILON,
        "HANDOFF_THRESHOLD_FRACTION must be 0.85 (E-32-decisions.md DECISION 4)"
    );
}

#[test]
fn fallback_effective_max_context_is_25k() {
    assert_eq!(
        FALLBACK_EFFECTIVE_MAX_CONTEXT, 25_000,
        "fallback must equal old ceiling (memory:project_qwen3b_baseline)"
    );
}

#[test]
fn deep_mode_ctx_cap_is_64k() {
    assert_eq!(
        DEEP_MODE_CTX_CAP, 64_000,
        "deep mode cap = 64K NF4 on 24GB per E-32-decisions.md DECISION 3"
    );
}

#[test]
fn min_files_referenced_is_3() {
    assert_eq!(
        DEEP_MODE_MIN_FILES_REFERENCED, 3,
        "cross-file synthesis qualifier = >3 files per E-32-decisions.md DECISION 3"
    );
}

// ---------------------------------------------------------------------------
// HandoffContext::handoff_threshold arithmetic
// ---------------------------------------------------------------------------

#[test]
fn threshold_32k_is_27200() {
    let ctx = HandoffContext {
        tokens_used: 0,
        effective_max_context: 32_000,
        files_referenced: 0,
        remote_24gb_reachable: false,
        session_id: 0,
    };
    assert_eq!(ctx.handoff_threshold(), 27_200);
}

#[test]
fn threshold_25k_is_21250() {
    let ctx = HandoffContext {
        tokens_used: 0,
        effective_max_context: 25_000,
        files_referenced: 0,
        remote_24gb_reachable: false,
        session_id: 0,
    };
    assert_eq!(ctx.handoff_threshold(), 21_250);
}

#[test]
fn threshold_64k_deep_mode_is_54400() {
    let ctx = HandoffContext {
        tokens_used: 0,
        effective_max_context: DEEP_MODE_CTX_CAP,
        files_referenced: 0,
        remote_24gb_reachable: false,
        session_id: 0,
    };
    assert_eq!(ctx.handoff_threshold(), 54_400);
}

#[test]
fn threshold_zero_ceiling_falls_back() {
    let ctx = HandoffContext {
        tokens_used: 0,
        effective_max_context: 0,
        files_referenced: 0,
        remote_24gb_reachable: false,
        session_id: 0,
    };
    // 85% of 25_000 = 21_250
    assert_eq!(ctx.handoff_threshold(), 21_250);
}

// ---------------------------------------------------------------------------
// HandoffDecision variants exist
// ---------------------------------------------------------------------------

fn _assert_handoff_decision_variants() {
    let _ = HandoffDecision::DeepMode {
        ctx_cap: 64_000,
        reason: "test".to_string(),
    };
    let _ = HandoffDecision::Cloud {
        reason: "test".to_string(),
        // I-11: belief_state_payload is None at construction; task_runner injects
        // the encoded BeliefState after adaptive_reduce returns.
        belief_state_payload: None,
    };
    let _ = HandoffDecision::Continue;
}

// ---------------------------------------------------------------------------
// EscalationPolicyError compiles
// ---------------------------------------------------------------------------

fn _assert_escalation_policy_error(_: EscalationPolicyError) {}
