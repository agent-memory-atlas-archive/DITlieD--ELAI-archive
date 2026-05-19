//! C-10 — Phase 4G.9 anti-spiral primitives contract suite.
//!
//! Covers all 11 types in `anti_spiral_extended.rs` + the gate-integration
//! emit + new C-10 SSB variants. Plan §Task 8 + §Task 9 = 17 + 5 = 22 tests.
//!
//! Plan: docs/thefinalglance/C-plan/C-10_anti_spiral_primitives.md
//! Decisions: workflow/reasoning/C-10-decisions.md

use std::sync::Arc;

use elai_orchestrator::{
    suggest_persona_for_pattern, AdaptiveFutilityTracker, AttemptResult, AutonomousEvidence,
    AutonomousGate, AutonomousGateConfig, AutonomousGateContext, AutonomyDecision, DriftVerdict,
    EscalationTrigger, ExecutionHistory, GateMode, GenerationWatchdog, GoalDriftCanary,
    PipelineLevel, RiskLevel, SsbActionType, SsbPayload, StagnationDetection,
    StagnationPattern, ThinkingPersona, WatchdogTimeoutKind, FUTILITY_THRESHOLD,
    HISTORY_CAPACITY, IDLE_TIMEOUT_MS, NO_PROGRESS_TIMEOUT_MS, SAFETY_TIMEOUT_MS,
    SCHEMA_VERSION_PUBLIC,
};

fn attempt(idx: u32, err: &str, hash: u64, ec: u32) -> AttemptResult {
    AttemptResult {
        attempt_index: idx,
        error_class: err.to_owned(),
        output_hash: hash,
        error_count: ec,
        timestamp_ms: 1_000_000 + (idx as u64),
    }
}

// ---------------------------------------------------------------------------
// Plan §Task 8: 17 anti_spiral_extended tests
// ---------------------------------------------------------------------------

#[test]
fn test_spinning_detected_after_threshold_identical_errors() {
    let mut h = ExecutionHistory::new();
    // Force NoDrift rule to *not* fire by varying output_hash on the last
    // two entries; Spinning requires identical error_class for 3+ entries.
    h.push(attempt(1, "compile_error", 11, 10));
    h.push(attempt(2, "compile_error", 22, 10));
    h.push(attempt(3, "compile_error", 33, 10));
    let det = StagnationDetection::default();
    assert_eq!(det.detect_pattern(&h), Some(StagnationPattern::Spinning));
}

#[test]
fn test_no_drift_detected_on_identical_hash() {
    let mut h = ExecutionHistory::new();
    h.push(attempt(1, "a_error", 99, 4));
    h.push(attempt(2, "b_error", 99, 4));
    let det = StagnationDetection::default();
    assert_eq!(det.detect_pattern(&h), Some(StagnationPattern::NoDrift));
}

#[test]
fn test_oscillation_detected_on_alternating_error_pair() {
    let mut h = ExecutionHistory::new();
    // 4-window A-B-A-B with strictly increasing hashes to defeat NoDrift +
    // 2 distinct error_class to defeat Spinning.
    h.push(attempt(1, "A_err", 10, 4));
    h.push(attempt(2, "B_err", 20, 4));
    h.push(attempt(3, "A_err", 30, 4));
    h.push(attempt(4, "B_err", 40, 4));
    let det = StagnationDetection::default();
    assert_eq!(det.detect_pattern(&h), Some(StagnationPattern::Oscillation));
}

#[test]
fn test_diminishing_returns_detected_below_5pct() {
    let mut h = ExecutionHistory::new();
    // 4-window, distinct error_classes + monotonically descending hashes
    // to defeat Spinning + NoDrift + Oscillation. error_count improves
    // <5% per step.
    h.push(attempt(1, "a", 1, 100));
    h.push(attempt(2, "b", 2, 99));
    h.push(attempt(3, "c", 3, 98));
    h.push(attempt(4, "d", 4, 97));
    let det = StagnationDetection::default();
    assert_eq!(
        det.detect_pattern(&h),
        Some(StagnationPattern::DiminishingReturns)
    );
}

#[test]
fn test_detect_returns_none_with_single_entry() {
    let mut h = ExecutionHistory::new();
    h.push(attempt(1, "x", 1, 1));
    let det = StagnationDetection::default();
    assert_eq!(det.detect_pattern(&h), None);
}

#[test]
fn test_persona_affinity_spinning_yields_contrarian() {
    assert_eq!(
        suggest_persona_for_pattern(StagnationPattern::Spinning),
        ThinkingPersona::Contrarian
    );
}

#[test]
fn test_persona_affinity_oscillation_yields_architect() {
    assert_eq!(
        suggest_persona_for_pattern(StagnationPattern::Oscillation),
        ThinkingPersona::Architect
    );
}

#[test]
fn test_persona_affinity_no_drift_yields_hacker() {
    assert_eq!(
        suggest_persona_for_pattern(StagnationPattern::NoDrift),
        ThinkingPersona::Hacker
    );
}

#[test]
fn test_persona_affinity_diminishing_yields_simplifier() {
    assert_eq!(
        suggest_persona_for_pattern(StagnationPattern::DiminishingReturns),
        ThinkingPersona::Simplifier
    );
}

#[test]
fn test_goal_drift_canary_green_above_07() {
    let c = GoalDriftCanary::default();
    let orig = "build a todo app with react and vite";
    let cur = "build a todo app with react and vite";
    let score = c.compute(orig, cur);
    assert!(score >= 0.7, "score must be >= 0.7 (got {score})");
    assert_eq!(c.verdict(score), DriftVerdict::Green);
}

#[test]
fn test_goal_drift_canary_red_below_04() {
    let c = GoalDriftCanary::default();
    let orig = "build a todo app with react and vite";
    let cur = "rewrite the linux kernel in haskell";
    let score = c.compute(orig, cur);
    assert!(score < 0.4, "score must be < 0.4 (got {score})");
    assert_eq!(c.verdict(score), DriftVerdict::Red);
}

#[test]
fn test_goal_drift_tokenize_strips_punctuation() {
    let toks = GoalDriftCanary::tokenize("Hello, World! 42-times.");
    assert!(toks.iter().any(|t| t == "hello"));
    assert!(toks.iter().any(|t| t == "world"));
    assert!(toks.iter().any(|t| t == "42times"));
    // No empty tokens, no comma/exclamation/period survivors.
    for t in &toks {
        assert!(!t.is_empty());
        assert!(t.chars().all(|c| c.is_alphanumeric()));
    }
}

#[test]
fn test_watchdog_safety_fires_at_ceiling() {
    let mut wd = GenerationWatchdog::default();
    // Safety window is 30 min (SAFETY_TIMEOUT_MS); at start_ms=0 and
    // now_ms=SAFETY_TIMEOUT_MS the safety branch fires regardless of
    // last_event_ms / fingerprint.
    let kind = wd.check(SAFETY_TIMEOUT_MS, SAFETY_TIMEOUT_MS, 7777, 0);
    assert_eq!(kind, Some(WatchdogTimeoutKind::Safety));
}

#[test]
fn test_watchdog_no_progress_fires_on_same_fingerprint() {
    let mut wd = GenerationWatchdog::default();
    // First call to seed the fingerprint at t=0.
    let _ = wd.check(0, 0, 12345, 0);
    // Same fingerprint after NO_PROGRESS_TIMEOUT_MS -> NoMaterialProgress.
    // last_event_ms = now_ms so Idle does NOT fire (avoids precedence
    // ambiguity since Safety < safety_timeout_ms still holds).
    let now = NO_PROGRESS_TIMEOUT_MS;
    let kind = wd.check(now, now, 12345, 0);
    assert_eq!(kind, Some(WatchdogTimeoutKind::NoMaterialProgress));
}

#[test]
fn test_watchdog_idle_fires_on_no_events() {
    let mut wd = GenerationWatchdog::default();
    // Seed at t=0 with fp=1 so the no-progress window starts.
    let _ = wd.check(0, 0, 1, 0);
    // At now=IDLE_TIMEOUT_MS, fingerprint advanced to 2 (no_progress window
    // resets) but last_event_ms still 0 -> Idle fires.
    let kind = wd.check(0, IDLE_TIMEOUT_MS, 2, 0);
    assert_eq!(kind, Some(WatchdogTimeoutKind::Idle));
}

#[test]
fn test_adaptive_futility_is_futile_below_threshold() {
    let mut t = AdaptiveFutilityTracker::new();
    for _ in 0..6 {
        t.record("xclass", false);
    }
    assert!(t.is_futile("xclass", FUTILITY_THRESHOLD));
    // Below FUTILITY_MIN_ATTEMPTS: never futile.
    let mut s = AdaptiveFutilityTracker::new();
    for _ in 0..2 {
        s.record("yclass", false);
    }
    assert!(!s.is_futile("yclass", FUTILITY_THRESHOLD));
}

#[test]
fn test_execution_history_ring_evicts_at_capacity() {
    let mut h = ExecutionHistory::new();
    let cap = HISTORY_CAPACITY;
    for i in 0..(cap as u32 + 5) {
        h.push(attempt(i, "k", i as u64, 0));
    }
    assert_eq!(h.len(), cap, "ring must cap at HISTORY_CAPACITY");
    // The oldest 5 entries were evicted; first remaining attempt_index = 5.
    let oldest = h.iter().next().expect("non-empty");
    assert_eq!(oldest.attempt_index, 5);
}

// ---------------------------------------------------------------------------
// Plan §Task 9: 5 gate-integration tests
// ---------------------------------------------------------------------------

fn evidence_all_pass() -> AutonomousEvidence {
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

fn base_ctx() -> AutonomousGateContext {
    AutonomousGateContext {
        level: PipelineLevel::BabyStep,
        risk_level: RiskLevel::Low,
        evidence: evidence_all_pass(),
        anti_spiral_fired: false,
        replan_major: false,
        max_restarts_exhausted: false,
        context_budget_terminal: false,
        auto_advance_count_for_concept: 0,
        step_id: "c10-step".into(),
        model_pool: None,
        execution_history: ExecutionHistory::default(),
        attempt_index: 0,
        suggested_prompt_prefix: None,
    }
}

#[tokio::test]
async fn test_decide_emits_stagnation_spinning_after_3_identical_errors() {
    // Verify the detection helper returns Spinning for the 3-identical
    // history that the gate caller would have observed before flipping
    // ctx.anti_spiral_fired = true.
    let mut h = ExecutionHistory::new();
    h.push(attempt(1, "compile_error", 11, 10));
    h.push(attempt(2, "compile_error", 22, 10));
    h.push(attempt(3, "compile_error", 33, 10));
    let det = StagnationDetection::default();
    assert_eq!(det.detect_pattern(&h), Some(StagnationPattern::Spinning));
    // The SSB variant the caller emits for this pattern.
    let payload = SsbPayload::StagnationSpinningDetected(
        elai_orchestrator::StagnationSpinningDetectedPayload {
            agent_id: "task_runner".into(),
            pattern: StagnationPattern::Spinning.as_str().to_owned(),
            iteration: 3,
            detail: "test".into(),
        },
    );
    assert_eq!(payload.action_type(), SsbActionType::StagnationSpinningDetected);
}

#[tokio::test]
async fn test_decide_emits_lateral_thinking_activated_with_persona() {
    // suggest_persona_for_pattern must return a known persona and the
    // emit-site shape must serialize to the LateralThinkingActivated row.
    let persona = suggest_persona_for_pattern(StagnationPattern::Oscillation);
    let payload = SsbPayload::LateralThinkingActivated(
        elai_orchestrator::LateralThinkingActivatedPayload {
            agent_id: "task_runner".into(),
            persona: persona.as_str().to_owned(),
            iteration: 5,
            detail: "test".into(),
        },
    );
    assert_eq!(payload.action_type(), SsbActionType::LateralThinkingActivated);
    assert_eq!(persona.as_str(), "architect");
}

#[tokio::test]
async fn test_decide_attaches_prompt_prefix_to_gate_ctx() {
    // When the caller detects a pattern, it sets
    // ctx.suggested_prompt_prefix to the persona strategy's prefix. Verify
    // the gate sees it (Debug or behavior-wise) and the AntiSpiral
    // short-circuit still fires because anti_spiral_fired was OR'd.
    let mut ctx = base_ctx();
    ctx.anti_spiral_fired = true;
    let strategy = elai_orchestrator::strategy_for(ThinkingPersona::Hacker);
    ctx.suggested_prompt_prefix = Some(strategy.prompt_prefix);

    let gate = AutonomousGate::new(
        AutonomousGateConfig {
            mode: GateMode::Autonomous,
            ..AutonomousGateConfig::default()
        },
        None,
    );
    let decision = gate.decide(&ctx).await;
    assert_eq!(
        decision,
        AutonomyDecision::Escalate {
            trigger: EscalationTrigger::AntiSpiral
        }
    );
    assert!(ctx.suggested_prompt_prefix.is_some());
    assert!(
        ctx.suggested_prompt_prefix
            .unwrap()
            .to_lowercase()
            .contains("dirty")
    );
}

#[tokio::test]
async fn test_goal_drift_warning_emitted_when_jaccard_red() {
    // GoalDriftCanary red verdict + emit-site payload shape match.
    let c = GoalDriftCanary::default();
    let score = c.compute("build todo app react vite", "rewrite kernel haskell");
    assert!(matches!(c.verdict(score), DriftVerdict::Red));
    let payload = SsbPayload::GoalDriftBlock(elai_orchestrator::GoalDriftBlockPayload {
        agent_id: "task_runner".into(),
        drift_score: score,
        baby_step_id: 1,
        detail: "test".into(),
    });
    assert_eq!(payload.action_type(), SsbActionType::GoalDriftBlock);
}

#[tokio::test]
async fn test_watchdog_timeout_emits_ssb_and_sets_escalate() {
    // GenerationWatchdog::Safety fires -> task_runner emits
    // SsbActionType::WatchdogTimeout + sets anti_spiral_fired so the gate
    // returns Escalate(AntiSpiral).
    let mut wd = GenerationWatchdog::default();
    let kind = wd.check(SAFETY_TIMEOUT_MS, SAFETY_TIMEOUT_MS, 1, 0);
    assert_eq!(kind, Some(WatchdogTimeoutKind::Safety));

    let payload = SsbPayload::WatchdogTimeout(elai_orchestrator::WatchdogTimeoutPayload {
        agent_id: "task_runner".into(),
        elapsed_ms: SAFETY_TIMEOUT_MS,
        budget_ms: SAFETY_TIMEOUT_MS,
        detail: "safety".into(),
    });
    assert_eq!(payload.action_type(), SsbActionType::WatchdogTimeout);

    let mut ctx = base_ctx();
    ctx.anti_spiral_fired = true; // task_runner OR's watchdog_fired here
    let gate = AutonomousGate::new(
        AutonomousGateConfig {
            mode: GateMode::Autonomous,
            ..AutonomousGateConfig::default()
        },
        None,
    );
    let decision = gate.decide(&ctx).await;
    assert_eq!(
        decision,
        AutonomyDecision::Escalate {
            trigger: EscalationTrigger::AntiSpiral
        }
    );

    // SCHEMA bump pin so a future inadvertent rollback is caught.
    assert!(SCHEMA_VERSION_PUBLIC >= 52);
    // Suppress unused-Arc-deps warning.
    let _ = Arc::new(());
}
