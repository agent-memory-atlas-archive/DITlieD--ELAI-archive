//! INTERACTIVE gate: pause + evidence + human decision at every phase transition.
//!
//! Phase 3A.4 deliverable. Source: architecture-overview.txt §6 L213-215,
//! phase-3A.txt §3A.4.
//!
//! Design: `InteractiveGate` is a trait (not hardcoded to CLI) so tests use
//! `MockInteractiveGate`. The CLI crate implements it for real human I/O.
//! The gate collects evidence from evidence gates (3A.3), formats a summary,
//! pauses, and returns a `GateDecision`.

use crate::evidence::{self, EvidenceError, TransitionContext};
use crate::fsm::{FsmError, FsmState};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// GateDecision — human response to an interactive gate
// ---------------------------------------------------------------------------

/// Human decision at an interactive gate pause point.
/// Source: architecture-overview.txt §6 L215 (approve, modify, reject, redirect).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GateDecision {
    /// Advance FSM to the proposed target state.
    Approve,
    /// Return to source state. No state change occurs.
    Reject,
    /// Human provides modifications. Evidence gate re-evaluates before advancing.
    Modify {
        /// Updated transition context with human-provided changes.
        updated_context: TransitionContext,
    },
    /// Human redirects to a different target state (must be valid from source).
    Redirect {
        /// The alternative target state chosen by the human.
        target: FsmState,
    },
}

// ---------------------------------------------------------------------------
// EvidenceSummary — structured evidence for human display
// ---------------------------------------------------------------------------

/// Structured evidence summary presented to the human at a gate.
/// Formats evidence artifacts from the evidence gate protocol (3A.3)
/// into a human-readable form.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceSummary {
    /// Source FSM state.
    pub source: FsmState,
    /// Proposed target FSM state.
    pub target: FsmState,
    /// Current phase label (e.g., "3A.4").
    pub phase_label: String,
    /// Evidence gate results: Ok means gate passed, Err has details.
    /// Each entry is (gate_name, result_description).
    pub gate_results: Vec<GateResult>,
    /// Overall pass/fail: true if all gates passed.
    pub all_gates_passed: bool,
}

/// Result of a single evidence gate check, formatted for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResult {
    pub gate_name: String,
    pub passed: bool,
    pub detail: String,
}

impl EvidenceSummary {
    /// Build an evidence summary by running all gates for the given transition.
    pub fn from_transition(
        source: FsmState,
        target: FsmState,
        phase_label: &str,
        ctx: &TransitionContext,
    ) -> Self {
        let gates = evidence::gates_for_transition(source, target);
        let mut gate_results = Vec::with_capacity(gates.len());
        let mut all_passed = true;

        for gate in &gates {
            match gate.check(ctx) {
                Ok(()) => {
                    gate_results.push(GateResult {
                        gate_name: gate.name().to_owned(),
                        passed: true,
                        detail: "passed".to_owned(),
                    });
                }
                Err(e) => {
                    all_passed = false;
                    gate_results.push(GateResult {
                        gate_name: gate.name().to_owned(),
                        passed: false,
                        detail: e.to_string(),
                    });
                }
            }
        }

        // Transitions with no gates always pass (e.g., CONFIGURING -> IDLE).
        EvidenceSummary {
            source,
            target,
            phase_label: phase_label.to_owned(),
            gate_results,
            all_gates_passed: all_passed,
        }
    }
}

// ---------------------------------------------------------------------------
// InteractiveGate trait
// ---------------------------------------------------------------------------

/// Trait for presenting gate evidence to a human and collecting a decision.
/// Source: phase-3A.txt L209 (UNGROUNDED trait pattern, assumed for testability,
/// consistent with KnowledgeEngine/GrcEngine/EvidenceGate pattern).
///
/// CLI crate implements this for real terminal I/O. Tests use `MockInteractiveGate`.
/// No timeout in INTERACTIVE mode (human-paced). AUTONOMOUS auto-advance timeout
/// deferred to Phase 4G.
pub trait InteractiveGate: Send + Sync {
    /// Present the evidence summary to the human and await a decision.
    /// This is a blocking call in INTERACTIVE mode — it waits for human input.
    fn present_gate(
        &self,
        summary: &EvidenceSummary,
        risk_level: RiskLevel,
    ) -> GateDecision;
}

// ---------------------------------------------------------------------------
// RiskLevel — per-transition risk classification
// ---------------------------------------------------------------------------

/// Risk level for a phase transition. Determines gate behavior in INTERACTIVE
/// vs AUTONOMOUS mode.
/// Source: architecture-overview.txt §6 L227-230.
///
/// In INTERACTIVE mode:
///   LOW    -> auto-advance (skip human prompt)
///   MEDIUM -> pause for human decision
///   HIGH   -> always pause for human decision
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Low => f.write_str("LOW"),
            RiskLevel::Medium => f.write_str("MEDIUM"),
            RiskLevel::High => f.write_str("HIGH"),
        }
    }
}

// ---------------------------------------------------------------------------
// Default risk classification per transition
// ---------------------------------------------------------------------------

/// Default risk level for a transition. Can be overridden per-project via
/// elai_cc.toml [fsm.risk_overrides].
/// Source: architecture-overview.txt §6 L228-230.
pub fn default_risk_level(source: FsmState, target: FsmState) -> RiskLevel {
    match (source, target) {
        // LOW: MEMORIZING -> next phase (auto in both modes)
        (FsmState::Memorizing, FsmState::Planning | FsmState::Complete) => RiskLevel::Low,
        // LOW: COMPLETE -> IDLE (terminal cleanup, no risk)
        (FsmState::Complete, FsmState::Idle) => RiskLevel::Low,
        // LOW: CONFIGURING -> IDLE (config exit, no risk)
        (FsmState::Configuring, FsmState::Idle) => RiskLevel::Low,

        // HIGH: EXECUTING -> PLANNING (replan = major course change)
        (FsmState::Executing, FsmState::Planning) => RiskLevel::High,
        // HIGH: AUDITING -> EXECUTING (fix BLOCKs = re-entering execution)
        (FsmState::Auditing, FsmState::Executing) => RiskLevel::High,
        // HIGH: AUDITING -> PLANNING (dead-end declaration)
        (FsmState::Auditing, FsmState::Planning) => RiskLevel::High,

        // MEDIUM: everything else (standard forward progress)
        _ => RiskLevel::Medium,
    }
}

// ---------------------------------------------------------------------------
// Gate orchestration: run evidence + interactive gate together
// ---------------------------------------------------------------------------

/// Result of running the full interactive gate protocol for a transition.
#[derive(Debug)]
pub enum InteractiveGateOutcome {
    /// Transition approved (evidence passed, human approved or LOW auto-advance).
    Approved { target: FsmState },
    /// Transition rejected by human. FSM stays at source state.
    Rejected,
    /// Human redirected to a different valid target.
    Redirected { new_target: FsmState },
    /// Evidence gate failed and human did not provide corrections.
    EvidenceFailed(EvidenceError),
    /// REDIRECT or transition to an invalid state.
    InvalidTransition(FsmError),
}

/// Run the full interactive gate protocol:
/// 1. Collect evidence from evidence gates
/// 2. Build EvidenceSummary
/// 3. Check risk level — LOW auto-advances without human prompt
/// 4. Present to human via InteractiveGate trait
/// 5. Handle GateDecision (APPROVE/REJECT/MODIFY/REDIRECT)
///
/// MODIFY triggers evidence re-evaluation before advancing.
/// REDIRECT validates the new target against the transition matrix.
///
/// Max re-evaluations for MODIFY: 3 (prevents infinite loops).
pub fn run_interactive_gate(
    source: FsmState,
    target: FsmState,
    phase_label: &str,
    ctx: &TransitionContext,
    gate: &dyn InteractiveGate,
    risk_override: Option<RiskLevel>,
) -> InteractiveGateOutcome {
    let risk = risk_override.unwrap_or_else(|| default_risk_level(source, target));

    // LOW risk: auto-advance without human prompt, but evidence must still pass.
    if risk == RiskLevel::Low {
        return match evidence::check_transition(source, target, ctx) {
            Ok(()) => InteractiveGateOutcome::Approved { target },
            Err(e) => InteractiveGateOutcome::EvidenceFailed(e),
        };
    }

    // MEDIUM/HIGH: present to human.
    let mut current_ctx = ctx.clone();
    let max_modify_rounds = 3;

    for round in 0..=max_modify_rounds {
        let summary = EvidenceSummary::from_transition(source, target, phase_label, &current_ctx);
        let decision = gate.present_gate(&summary, risk);

        match decision {
            GateDecision::Approve => {
                // Even with human approval, evidence must pass.
                return match evidence::check_transition(source, target, &current_ctx) {
                    Ok(()) => InteractiveGateOutcome::Approved { target },
                    Err(e) => InteractiveGateOutcome::EvidenceFailed(e),
                };
            }
            GateDecision::Reject => {
                return InteractiveGateOutcome::Rejected;
            }
            GateDecision::Modify { updated_context } => {
                if round == max_modify_rounds {
                    // Exceeded max modify rounds — treat as rejection.
                    return InteractiveGateOutcome::Rejected;
                }
                current_ctx = updated_context;
                // Loop continues: re-evaluate evidence with updated context.
            }
            GateDecision::Redirect { target: new_target } => {
                // Validate that the redirect target is reachable from source.
                if !source.can_transition_to(new_target) {
                    return InteractiveGateOutcome::InvalidTransition(
                        FsmError::InvalidTransition {
                            source,
                            target: new_target,
                        },
                    );
                }
                // Run evidence gates for the new transition.
                return match evidence::check_transition(source, new_target, &current_ctx) {
                    Ok(()) => InteractiveGateOutcome::Redirected { new_target },
                    Err(e) => InteractiveGateOutcome::EvidenceFailed(e),
                };
            }
        }
    }

    // Should not reach here, but if it does, reject.
    InteractiveGateOutcome::Rejected
}

// ---------------------------------------------------------------------------
// MockInteractiveGate — for testing
// ---------------------------------------------------------------------------

/// Mock interactive gate that returns pre-configured decisions.
/// Tests construct it with a sequence of decisions to return.
#[cfg(test)]
pub struct MockInteractiveGate {
    decisions: std::sync::Mutex<Vec<GateDecision>>,
    /// Records each summary presented, for assertions.
    presented: std::sync::Mutex<Vec<EvidenceSummary>>,
}

#[cfg(test)]
impl MockInteractiveGate {
    /// Create a mock that returns the given decisions in order.
    /// The last decision repeats if more calls than decisions.
    pub fn new(decisions: Vec<GateDecision>) -> Self {
        Self {
            decisions: std::sync::Mutex::new(decisions),
            presented: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// All summaries that were presented to this mock.
    pub fn presented_summaries(&self) -> Vec<EvidenceSummary> {
        self.presented.lock().unwrap().clone()
    }

    /// Number of times present_gate was called.
    pub fn call_count(&self) -> usize {
        self.presented.lock().unwrap().len()
    }
}

#[cfg(test)]
impl InteractiveGate for MockInteractiveGate {
    fn present_gate(
        &self,
        summary: &EvidenceSummary,
        _risk_level: RiskLevel,
    ) -> GateDecision {
        self.presented.lock().unwrap().push(summary.clone());
        let mut decisions = self.decisions.lock().unwrap();
        if decisions.is_empty() {
            GateDecision::Approve
        } else {
            decisions.remove(0)
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::{
        AuditFinding, FindingSeverity, KnowledgeReadiness, TestOutcome, TransitionContext,
    };

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn ctx_idle_to_planning() -> TransitionContext {
        TransitionContext {
            knowledge_readiness: Some(KnowledgeReadiness::Ready),
            ..Default::default()
        }
    }

    fn ctx_planning_to_testing() -> TransitionContext {
        TransitionContext {
            ungrounded_claims: Some(vec![]),
            ..Default::default()
        }
    }

    fn ctx_testing_to_executing() -> TransitionContext {
        TransitionContext {
            test_outcomes: Some(vec![TestOutcome {
                name: "test_stub".into(),
                passed: false,
                is_syntax_error: false,
                flipped_to_pass: false,
            }]),
            ..Default::default()
        }
    }

    fn ctx_executing_to_auditing() -> TransitionContext {
        TransitionContext {
            test_outcomes: Some(vec![TestOutcome {
                name: "test_impl".into(),
                passed: true,
                is_syntax_error: false,
                flipped_to_pass: true,
            }]),
            ..Default::default()
        }
    }

    fn ctx_auditing_to_memorizing() -> TransitionContext {
        TransitionContext {
            audit_findings: Some(vec![AuditFinding {
                severity: FindingSeverity::Info,
                message: "style nit".into(),
            }]),
            ..Default::default()
        }
    }

    fn ctx_memorizing_to_complete() -> TransitionContext {
        TransitionContext {
            dkg_reflects_new_code: Some(true),
            changelog_has_entry: Some(true),
            ..Default::default()
        }
    }

    // -----------------------------------------------------------------------
    // GateDecision variants exist (deterministic check)
    // -----------------------------------------------------------------------

    #[test]
    fn gate_decision_has_four_variants() {
        // Verify all 4 variants are constructible (deterministic check from spec).
        let _approve = GateDecision::Approve;
        let _reject = GateDecision::Reject;
        let _modify = GateDecision::Modify {
            updated_context: TransitionContext::default(),
        };
        let _redirect = GateDecision::Redirect {
            target: FsmState::Idle,
        };
    }

    #[test]
    fn gate_decision_serialize_roundtrip() {
        let decisions = [
            GateDecision::Approve,
            GateDecision::Reject,
            GateDecision::Modify {
                updated_context: TransitionContext::default(),
            },
            GateDecision::Redirect {
                target: FsmState::Planning,
            },
        ];
        for d in &decisions {
            let json = serde_json::to_string(d).expect("serialize");
            let back: GateDecision = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(*d, back);
        }
    }

    // -----------------------------------------------------------------------
    // EvidenceSummary construction
    // -----------------------------------------------------------------------

    #[test]
    fn evidence_summary_all_gates_pass() {
        let ctx = ctx_idle_to_planning();
        let summary =
            EvidenceSummary::from_transition(FsmState::Idle, FsmState::Planning, "3A.4", &ctx);
        assert!(summary.all_gates_passed);
        assert_eq!(summary.source, FsmState::Idle);
        assert_eq!(summary.target, FsmState::Planning);
        assert!(!summary.gate_results.is_empty());
        assert!(summary.gate_results.iter().all(|g| g.passed));
    }

    #[test]
    fn evidence_summary_gate_fails() {
        // Missing knowledge_readiness -> gate fails.
        let ctx = TransitionContext::default();
        let summary =
            EvidenceSummary::from_transition(FsmState::Idle, FsmState::Planning, "3A.4", &ctx);
        assert!(!summary.all_gates_passed);
        assert!(summary.gate_results.iter().any(|g| !g.passed));
    }

    #[test]
    fn evidence_summary_ungated_transition() {
        // CONFIGURING -> IDLE has no gates, so summary has no gate_results.
        let ctx = TransitionContext::default();
        let summary = EvidenceSummary::from_transition(
            FsmState::Configuring,
            FsmState::Idle,
            "config",
            &ctx,
        );
        assert!(summary.all_gates_passed);
        assert!(summary.gate_results.is_empty());
    }

    // -----------------------------------------------------------------------
    // Risk level classification
    // -----------------------------------------------------------------------

    #[test]
    fn risk_level_low_transitions() {
        assert_eq!(
            default_risk_level(FsmState::Memorizing, FsmState::Planning),
            RiskLevel::Low
        );
        assert_eq!(
            default_risk_level(FsmState::Memorizing, FsmState::Complete),
            RiskLevel::Low
        );
        assert_eq!(
            default_risk_level(FsmState::Complete, FsmState::Idle),
            RiskLevel::Low
        );
        assert_eq!(
            default_risk_level(FsmState::Configuring, FsmState::Idle),
            RiskLevel::Low
        );
    }

    #[test]
    fn risk_level_medium_default() {
        // Standard forward transitions are MEDIUM.
        assert_eq!(
            default_risk_level(FsmState::Idle, FsmState::Planning),
            RiskLevel::Medium
        );
        assert_eq!(
            default_risk_level(FsmState::Planning, FsmState::TestingFirst),
            RiskLevel::Medium
        );
        assert_eq!(
            default_risk_level(FsmState::TestingFirst, FsmState::Executing),
            RiskLevel::Medium
        );
        assert_eq!(
            default_risk_level(FsmState::Executing, FsmState::Auditing),
            RiskLevel::Medium
        );
        assert_eq!(
            default_risk_level(FsmState::Auditing, FsmState::Memorizing),
            RiskLevel::Medium
        );
    }

    #[test]
    fn risk_level_high_transitions() {
        // Backward/replan transitions are HIGH.
        assert_eq!(
            default_risk_level(FsmState::Executing, FsmState::Planning),
            RiskLevel::High
        );
        assert_eq!(
            default_risk_level(FsmState::Auditing, FsmState::Executing),
            RiskLevel::High
        );
        assert_eq!(
            default_risk_level(FsmState::Auditing, FsmState::Planning),
            RiskLevel::High
        );
    }

    #[test]
    fn planning_to_testing_is_medium_not_low() {
        // Adversarial: if all gates default to LOW, INTERACTIVE auto-advances
        // everything. PLANNING -> TESTING_FIRST must be MEDIUM.
        assert_ne!(
            default_risk_level(FsmState::Planning, FsmState::TestingFirst),
            RiskLevel::Low
        );
    }

    // -----------------------------------------------------------------------
    // FsmConfig risk overrides (3A.5)
    // -----------------------------------------------------------------------

    #[test]
    fn resolve_risk_level_default_without_override() {
        // FsmConfig::default() has no overrides — falls back to default_risk_level.
        let config = crate::fsm::FsmConfig::default();
        assert_eq!(
            crate::fsm::resolve_risk_level(&config, FsmState::Planning, FsmState::TestingFirst),
            RiskLevel::Medium,
        );
        assert_eq!(
            crate::fsm::resolve_risk_level(&config, FsmState::Memorizing, FsmState::Complete),
            RiskLevel::Low,
        );
        assert_eq!(
            crate::fsm::resolve_risk_level(&config, FsmState::Executing, FsmState::Planning),
            RiskLevel::High,
        );
    }

    #[test]
    fn resolve_risk_level_config_override_applies() {
        // Override MEMORIZING->COMPLETE from Low to High via config.
        let mut config = crate::fsm::FsmConfig::default();
        config
            .risk_overrides
            .insert("MEMORIZING->COMPLETE".to_string(), RiskLevel::High);
        assert_eq!(
            crate::fsm::resolve_risk_level(&config, FsmState::Memorizing, FsmState::Complete),
            RiskLevel::High,
        );
    }

    #[test]
    fn resolve_risk_level_config_override_does_not_affect_other_transitions() {
        // An override for one transition does not affect others.
        let mut config = crate::fsm::FsmConfig::default();
        config
            .risk_overrides
            .insert("MEMORIZING->COMPLETE".to_string(), RiskLevel::High);
        // PLANNING->TESTING_FIRST should still be Medium (no override).
        assert_eq!(
            crate::fsm::resolve_risk_level(&config, FsmState::Planning, FsmState::TestingFirst),
            RiskLevel::Medium,
        );
    }

    #[test]
    fn all_valid_transitions_have_risk_level() {
        // Coverage: every valid transition in the matrix returns a RiskLevel
        // without panic. Ensures no gaps in default_risk_level.
        for &src in FsmState::ALL.iter() {
            for &tgt in src.valid_targets() {
                let risk = default_risk_level(src, tgt);
                // Just asserting it's one of the three variants (no panic).
                assert!(
                    matches!(risk, RiskLevel::Low | RiskLevel::Medium | RiskLevel::High),
                    "Missing risk level for {src}->{tgt}"
                );
            }
        }
    }

    #[test]
    fn risk_level_for_transition_delegates_to_default() {
        // crate::fsm::risk_level_for_transition is a convenience re-export.
        for &src in FsmState::ALL.iter() {
            for &tgt in src.valid_targets() {
                assert_eq!(
                    crate::fsm::risk_level_for_transition(src, tgt),
                    default_risk_level(src, tgt),
                );
            }
        }
    }

    #[test]
    fn fsm_config_serde_round_trip() {
        // FsmConfig must survive serde round-trip (required for elai_cc.toml loading).
        let mut config = crate::fsm::FsmConfig::default();
        config
            .risk_overrides
            .insert("IDLE->PLANNING".to_string(), RiskLevel::High);
        let json = serde_json::to_string(&config).expect("serialize FsmConfig");
        let loaded: crate::fsm::FsmConfig =
            serde_json::from_str(&json).expect("deserialize FsmConfig");
        assert_eq!(loaded.risk_overrides.len(), 1);
        assert_eq!(
            loaded.risk_overrides.get("IDLE->PLANNING").copied(),
            Some(RiskLevel::High),
        );
    }

    // -----------------------------------------------------------------------
    // APPROVE decision
    // -----------------------------------------------------------------------

    #[test]
    fn approve_advances_to_target() {
        let ctx = ctx_idle_to_planning();
        let mock = MockInteractiveGate::new(vec![GateDecision::Approve]);
        let outcome = run_interactive_gate(
            FsmState::Idle,
            FsmState::Planning,
            "3A.4",
            &ctx,
            &mock,
            None,
        );
        match outcome {
            InteractiveGateOutcome::Approved { target } => {
                assert_eq!(target, FsmState::Planning);
            }
            other => panic!("expected Approved, got {other:?}"),
        }
        assert_eq!(mock.call_count(), 1);
    }

    // -----------------------------------------------------------------------
    // REJECT decision
    // -----------------------------------------------------------------------

    #[test]
    fn reject_preserves_source_state() {
        let ctx = ctx_idle_to_planning();
        let mock = MockInteractiveGate::new(vec![GateDecision::Reject]);
        let outcome = run_interactive_gate(
            FsmState::Idle,
            FsmState::Planning,
            "3A.4",
            &ctx,
            &mock,
            None,
        );
        match outcome {
            InteractiveGateOutcome::Rejected => {}
            other => panic!("expected Rejected, got {other:?}"),
        }
        // Source state is preserved (FSM doesn't change) — caller responsibility,
        // but gate must return Rejected (not Approved or error).
    }

    #[test]
    fn reject_from_planning_to_testing() {
        let ctx = ctx_planning_to_testing();
        let mock = MockInteractiveGate::new(vec![GateDecision::Reject]);
        let outcome = run_interactive_gate(
            FsmState::Planning,
            FsmState::TestingFirst,
            "3A.4",
            &ctx,
            &mock,
            None,
        );
        assert!(matches!(outcome, InteractiveGateOutcome::Rejected));
    }

    // -----------------------------------------------------------------------
    // MODIFY decision
    // -----------------------------------------------------------------------

    #[test]
    fn modify_reevaluates_evidence_then_advances() {
        // First round: evidence fails (no knowledge_readiness).
        // Human provides MODIFY with corrected context.
        // Second round: evidence passes, human approves.
        let corrected_ctx = ctx_idle_to_planning();
        let mock = MockInteractiveGate::new(vec![
            GateDecision::Modify {
                updated_context: corrected_ctx,
            },
            GateDecision::Approve,
        ]);

        let empty_ctx = TransitionContext::default();
        let outcome = run_interactive_gate(
            FsmState::Idle,
            FsmState::Planning,
            "3A.4",
            &empty_ctx,
            &mock,
            None,
        );

        match outcome {
            InteractiveGateOutcome::Approved { target } => {
                assert_eq!(target, FsmState::Planning);
            }
            other => panic!("expected Approved after modify, got {other:?}"),
        }
        // Gate was called twice: once with failing evidence, once with corrected.
        assert_eq!(mock.call_count(), 2);
    }

    #[test]
    fn modify_evidence_gate_reruns() {
        // Verify evidence gate is called on each MODIFY round.
        // Loop runs rounds 0..=3 (max_modify_rounds=3). On round 3, if MODIFY
        // is returned, it hits the limit and forces Rejected.
        let still_bad_ctx = TransitionContext::default(); // still missing readiness
        let mock = MockInteractiveGate::new(vec![
            GateDecision::Modify {
                updated_context: still_bad_ctx.clone(),
            },
            GateDecision::Modify {
                updated_context: still_bad_ctx.clone(),
            },
            GateDecision::Modify {
                updated_context: still_bad_ctx.clone(),
            },
            GateDecision::Modify {
                updated_context: still_bad_ctx,
            },
        ]);

        let outcome = run_interactive_gate(
            FsmState::Idle,
            FsmState::Planning,
            "3A.4",
            &TransitionContext::default(),
            &mock,
            None,
        );

        // Round 3: MODIFY returned, but round == max_modify_rounds -> Rejected.
        assert!(matches!(outcome, InteractiveGateOutcome::Rejected));
        // Gate called 4 times: rounds 0, 1, 2, 3.
        assert_eq!(mock.call_count(), 4);
    }

    // -----------------------------------------------------------------------
    // REDIRECT decision
    // -----------------------------------------------------------------------

    #[test]
    fn redirect_to_valid_target() {
        // From PLANNING, redirect to FEASIBILITY_PROBE instead of TESTING_FIRST.
        let ctx = TransitionContext::default(); // no gates for PLANNING -> FEASIBILITY_PROBE
        let mock = MockInteractiveGate::new(vec![GateDecision::Redirect {
            target: FsmState::FeasibilityProbe,
        }]);

        let outcome = run_interactive_gate(
            FsmState::Planning,
            FsmState::TestingFirst,
            "3A.4",
            &ctx,
            &mock,
            None,
        );

        match outcome {
            InteractiveGateOutcome::Redirected { new_target } => {
                assert_eq!(new_target, FsmState::FeasibilityProbe);
            }
            other => panic!("expected Redirected, got {other:?}"),
        }
    }

    #[test]
    fn redirect_to_invalid_target_rejected() {
        // From PLANNING, redirect to COMPLETE — invalid transition.
        let ctx = ctx_planning_to_testing();
        let mock = MockInteractiveGate::new(vec![GateDecision::Redirect {
            target: FsmState::Complete,
        }]);

        let outcome = run_interactive_gate(
            FsmState::Planning,
            FsmState::TestingFirst,
            "3A.4",
            &ctx,
            &mock,
            None,
        );

        match outcome {
            InteractiveGateOutcome::InvalidTransition(FsmError::InvalidTransition {
                source,
                target,
            }) => {
                assert_eq!(source, FsmState::Planning);
                assert_eq!(target, FsmState::Complete);
            }
            other => panic!("expected InvalidTransition, got {other:?}"),
        }
    }

    #[test]
    fn redirect_to_every_invalid_target_from_planning() {
        // Adversarial: REDIRECT to all invalid targets from PLANNING.
        let valid = FsmState::Planning.valid_targets();
        let ctx = ctx_planning_to_testing();

        for &state in &FsmState::ALL {
            if valid.contains(&state) || state == FsmState::Planning {
                continue;
            }
            let mock = MockInteractiveGate::new(vec![GateDecision::Redirect {
                target: state,
            }]);
            let outcome = run_interactive_gate(
                FsmState::Planning,
                FsmState::TestingFirst,
                "3A.4",
                &ctx,
                &mock,
                None,
            );
            assert!(
                matches!(outcome, InteractiveGateOutcome::InvalidTransition(_)),
                "REDIRECT to {state} from PLANNING should fail"
            );
        }
    }

    // -----------------------------------------------------------------------
    // LOW risk auto-advance
    // -----------------------------------------------------------------------

    #[test]
    fn low_risk_auto_advances_without_human() {
        let ctx = ctx_memorizing_to_complete();
        let mock = MockInteractiveGate::new(vec![]);
        let outcome = run_interactive_gate(
            FsmState::Memorizing,
            FsmState::Complete,
            "3A.4",
            &ctx,
            &mock,
            None,
        );

        match outcome {
            InteractiveGateOutcome::Approved { target } => {
                assert_eq!(target, FsmState::Complete);
            }
            other => panic!("expected auto-advance Approved, got {other:?}"),
        }
        // Mock was never called — LOW risk skips human.
        assert_eq!(mock.call_count(), 0);
    }

    #[test]
    fn low_risk_still_checks_evidence() {
        // LOW risk auto-advances but evidence must still pass.
        let bad_ctx = TransitionContext::default(); // missing dkg + changelog
        let mock = MockInteractiveGate::new(vec![]);
        let outcome = run_interactive_gate(
            FsmState::Memorizing,
            FsmState::Complete,
            "3A.4",
            &bad_ctx,
            &mock,
            None,
        );
        assert!(matches!(outcome, InteractiveGateOutcome::EvidenceFailed(_)));
        assert_eq!(mock.call_count(), 0);
    }

    // -----------------------------------------------------------------------
    // Risk override
    // -----------------------------------------------------------------------

    #[test]
    fn risk_override_promotes_low_to_high() {
        // Override MEMORIZING -> COMPLETE from LOW to HIGH.
        // Now it requires human approval.
        let ctx = ctx_memorizing_to_complete();
        let mock = MockInteractiveGate::new(vec![GateDecision::Approve]);
        let outcome = run_interactive_gate(
            FsmState::Memorizing,
            FsmState::Complete,
            "3A.4",
            &ctx,
            &mock,
            Some(RiskLevel::High),
        );
        match outcome {
            InteractiveGateOutcome::Approved { target } => {
                assert_eq!(target, FsmState::Complete);
            }
            other => panic!("expected Approved, got {other:?}"),
        }
        // Human was prompted because risk was overridden to HIGH.
        assert_eq!(mock.call_count(), 1);
    }

    // -----------------------------------------------------------------------
    // Full pipeline tests (multiple transitions)
    // -----------------------------------------------------------------------

    #[test]
    fn approve_full_forward_pipeline() {
        // Test APPROVE through the main forward path:
        // IDLE -> PLANNING -> TESTING_FIRST -> EXECUTING -> AUDITING -> MEMORIZING -> COMPLETE
        let transitions: Vec<(FsmState, FsmState, TransitionContext)> = vec![
            (FsmState::Idle, FsmState::Planning, ctx_idle_to_planning()),
            (
                FsmState::Planning,
                FsmState::TestingFirst,
                ctx_planning_to_testing(),
            ),
            (
                FsmState::TestingFirst,
                FsmState::Executing,
                ctx_testing_to_executing(),
            ),
            (
                FsmState::Executing,
                FsmState::Auditing,
                ctx_executing_to_auditing(),
            ),
            (
                FsmState::Auditing,
                FsmState::Memorizing,
                ctx_auditing_to_memorizing(),
            ),
            (
                FsmState::Memorizing,
                FsmState::Complete,
                ctx_memorizing_to_complete(),
            ),
        ];

        for (source, target, ctx) in &transitions {
            let mock = MockInteractiveGate::new(vec![GateDecision::Approve]);
            let outcome =
                run_interactive_gate(*source, *target, "pipeline", ctx, &mock, None);
            match outcome {
                InteractiveGateOutcome::Approved { target: t } => {
                    assert_eq!(t, *target, "failed at {source} -> {target}");
                }
                other => panic!("expected Approved at {source} -> {target}, got {other:?}"),
            }
        }
    }

    // -----------------------------------------------------------------------
    // MockInteractiveGate behavior
    // -----------------------------------------------------------------------

    #[test]
    fn mock_records_summaries() {
        let ctx = ctx_idle_to_planning();
        let mock = MockInteractiveGate::new(vec![GateDecision::Approve]);
        let _ = run_interactive_gate(
            FsmState::Idle,
            FsmState::Planning,
            "test",
            &ctx,
            &mock,
            None,
        );

        let summaries = mock.presented_summaries();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].source, FsmState::Idle);
        assert_eq!(summaries[0].target, FsmState::Planning);
    }

    #[test]
    fn mock_defaults_to_approve_when_decisions_exhausted() {
        let ctx = ctx_idle_to_planning();
        let mock = MockInteractiveGate::new(vec![]); // empty
        let outcome = run_interactive_gate(
            FsmState::Idle,
            FsmState::Planning,
            "test",
            &ctx,
            &mock,
            None,
        );
        // Default is Approve when no decisions configured.
        assert!(matches!(outcome, InteractiveGateOutcome::Approved { .. }));
    }

    // -----------------------------------------------------------------------
    // InteractiveGate is object safe
    // -----------------------------------------------------------------------

    #[test]
    fn interactive_gate_is_object_safe() {
        fn _assert_object_safe(_: &dyn InteractiveGate) {}
        let mock = MockInteractiveGate::new(vec![]);
        _assert_object_safe(&mock);
    }
}
