//! Evidence gate protocol: 7 gates enforcing artifact requirements at FSM
//! transitions. DAIRA-derived adaptive context support.
//!
//! Phase 3A.3 deliverable. Source: orchestrator.txt §13 (L624-643),
//! phase-3A.txt §3A.3.
//!
//! Design: gates are decoupled from concrete crate types. The caller populates
//! `TransitionContext` with evidence data; each gate checks the data it needs.
//! This avoids coupling orchestrator -> knowledge/reasoning at compile time.

use crate::fsm::{FsmState, ReplanSeverity};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// EvidenceError — typed error from a failed gate check
// ---------------------------------------------------------------------------

/// Error produced when an evidence gate rejects a transition.
/// Source: phase-3A.txt L165-166.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceError {
    pub gate_name: &'static str,
    pub missing_artifacts: Vec<String>,
    pub suggestion: String,
}

impl std::fmt::Display for EvidenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "evidence gate '{}' failed: missing [{}]. {}",
            self.gate_name,
            self.missing_artifacts.join(", "),
            self.suggestion
        )
    }
}

impl std::error::Error for EvidenceError {}

// ---------------------------------------------------------------------------
// KnowledgeReadiness — injected status from knowledge engine
// ---------------------------------------------------------------------------

/// Knowledge engine readiness, mirroring knowledge crate's ReadinessStatus.
/// Decoupled: caller converts from the knowledge crate type.
/// Source: orchestrator.txt §13 L626-629.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KnowledgeReadiness {
    Ready,
    Degraded,
    Unavailable,
}

// ---------------------------------------------------------------------------
// TestOutcome — summary of test results for gates 3 and 4
// ---------------------------------------------------------------------------

/// Summary of a single test's outcome. Gates 3 and 4 inspect these.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestOutcome {
    pub name: String,
    pub passed: bool,
    /// If failed: was it a "right reason" failure (expected behavior not yet
    /// implemented) vs syntax/compile error?
    pub is_syntax_error: bool,
    /// Whether this test flipped from FAIL to PASS in this phase.
    pub flipped_to_pass: bool,
}

// ---------------------------------------------------------------------------
// AuditFinding — summary of audit findings for gate 6
// ---------------------------------------------------------------------------

/// An audit finding. Gate 6 checks that zero BLOCKs remain.
/// Source: orchestrator.txt §13 L639.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FindingSeverity {
    Block,
    Warn,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditFinding {
    pub severity: FindingSeverity,
    pub message: String,
}

// ---------------------------------------------------------------------------
// TransitionContext — evidence data for gate checks
// ---------------------------------------------------------------------------

/// All evidence data a caller can provide for gate evaluation.
/// Each gate reads only the fields it needs; unset fields are None.
/// Source: phase-3A.txt L164.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransitionContext {
    pub source: Option<FsmState>,
    pub target: Option<FsmState>,
    pub phase_label: Option<String>,

    // Gate 1: IDLE -> PLANNING
    pub knowledge_readiness: Option<KnowledgeReadiness>,

    // Gate 2: PLANNING -> TESTING_FIRST
    /// List of ungrounded claims from grounding_sweep_report.
    pub ungrounded_claims: Option<Vec<String>>,

    // Gate 3: TESTING_FIRST -> EXECUTING
    /// Test outcomes from the TDD test run.
    pub test_outcomes: Option<Vec<TestOutcome>>,

    // Gate 4: EXECUTING -> AUDITING (reuses test_outcomes, checks flipped)

    // Gate 5: EXECUTING -> PLANNING (fast-track replan)
    pub replan_severity: Option<ReplanSeverity>,
    pub replan_auditor_approved: Option<bool>,

    // Gate 6: AUDITING -> MEMORIZING
    pub audit_findings: Option<Vec<AuditFinding>>,

    // Gate 7: MEMORIZING -> next
    pub dkg_reflects_new_code: Option<bool>,
    pub changelog_has_entry: Option<bool>,

    // DAIRA: token budget for adaptive scope
    pub token_budget: Option<usize>,
}

// ---------------------------------------------------------------------------
// EvidenceGate trait
// ---------------------------------------------------------------------------

/// A gate that checks whether a transition has sufficient evidence.
/// Source: phase-3A.txt L163-164 (UNGROUNDED trait pattern, assumed for
/// testability, consistent with KnowledgeEngine/GrcEngine trait pattern).
pub trait EvidenceGate: Send + Sync {
    /// Human-readable gate name (e.g., "knowledge_readiness").
    fn name(&self) -> &'static str;

    /// The transition this gate guards (source, target).
    fn transition(&self) -> (FsmState, FsmState);

    /// Check whether the transition context has sufficient evidence.
    /// Returns Ok(()) if the gate passes, Err(EvidenceError) if not.
    fn check(&self, ctx: &TransitionContext) -> Result<(), EvidenceError>;
}

// ---------------------------------------------------------------------------
// Gate 1: KnowledgeReadinessGate (IDLE -> PLANNING)
// ---------------------------------------------------------------------------

/// IDLE -> PLANNING: knowledge engine must be READY or DEGRADED.
/// UNAVAILABLE blocks the transition.
/// Source: orchestrator.txt §13 L626-629.
pub struct KnowledgeReadinessGate;

impl EvidenceGate for KnowledgeReadinessGate {
    fn name(&self) -> &'static str {
        "knowledge_readiness"
    }

    fn transition(&self) -> (FsmState, FsmState) {
        (FsmState::Idle, FsmState::Planning)
    }

    fn check(&self, ctx: &TransitionContext) -> Result<(), EvidenceError> {
        match ctx.knowledge_readiness {
            None => Err(EvidenceError {
                gate_name: self.name(),
                missing_artifacts: vec!["knowledge_readiness_status".into()],
                suggestion: "Provide knowledge engine readiness status before transitioning."
                    .into(),
            }),
            Some(KnowledgeReadiness::Unavailable) => Err(EvidenceError {
                gate_name: self.name(),
                missing_artifacts: vec!["knowledge_engine_available".into()],
                suggestion:
                    "Knowledge engine is UNAVAILABLE. Fix startup issues before proceeding."
                        .into(),
            }),
            // READY and DEGRADED both pass. DEGRADED logs a warning at the
            // caller level (gate only returns pass/fail).
            Some(KnowledgeReadiness::Ready | KnowledgeReadiness::Degraded) => Ok(()),
        }
    }
}

// ---------------------------------------------------------------------------
// Gate 2: GroundingSweepGate (PLANNING -> TESTING_FIRST)
// ---------------------------------------------------------------------------

/// PLANNING -> TESTING_FIRST: zero UNGROUNDED claims in grounding_sweep_report.
/// Source: orchestrator.txt §13 L631.
pub struct GroundingSweepGate;

impl EvidenceGate for GroundingSweepGate {
    fn name(&self) -> &'static str {
        "grounding_sweep"
    }

    fn transition(&self) -> (FsmState, FsmState) {
        (FsmState::Planning, FsmState::TestingFirst)
    }

    fn check(&self, ctx: &TransitionContext) -> Result<(), EvidenceError> {
        match &ctx.ungrounded_claims {
            None => Err(EvidenceError {
                gate_name: self.name(),
                missing_artifacts: vec!["grounding_sweep_report".into()],
                suggestion: "Run grounding sweep before transitioning to TESTING_FIRST.".into(),
            }),
            Some(claims) if !claims.is_empty() => Err(EvidenceError {
                gate_name: self.name(),
                missing_artifacts: claims.clone(),
                suggestion: format!(
                    "{} ungrounded claim(s) remain. Ground all claims before proceeding.",
                    claims.len()
                ),
            }),
            Some(_) => Ok(()),
        }
    }
}

// ---------------------------------------------------------------------------
// Gate 3: TestFailReasonsGate (TESTING_FIRST -> EXECUTING)
// ---------------------------------------------------------------------------

/// TESTING_FIRST -> EXECUTING: all tests fail for right reasons (not syntax errors).
/// Source: orchestrator.txt §13 L632.
pub struct TestFailReasonsGate;

impl EvidenceGate for TestFailReasonsGate {
    fn name(&self) -> &'static str {
        "test_fail_reasons"
    }

    fn transition(&self) -> (FsmState, FsmState) {
        (FsmState::TestingFirst, FsmState::Executing)
    }

    fn check(&self, ctx: &TransitionContext) -> Result<(), EvidenceError> {
        match &ctx.test_outcomes {
            None => Err(EvidenceError {
                gate_name: self.name(),
                missing_artifacts: vec!["test_outcomes".into()],
                suggestion: "Run TDD tests before transitioning to EXECUTING.".into(),
            }),
            Some(outcomes) => {
                // All tests must either pass (pre-existing green) or fail for
                // right reasons (not syntax errors).
                let syntax_errors: Vec<String> = outcomes
                    .iter()
                    .filter(|t| !t.passed && t.is_syntax_error)
                    .map(|t| t.name.clone())
                    .collect();

                if syntax_errors.is_empty() {
                    Ok(())
                } else {
                    Err(EvidenceError {
                        gate_name: self.name(),
                        missing_artifacts: syntax_errors,
                        suggestion:
                            "Tests are failing due to syntax errors, not missing implementation. \
                             Fix syntax before proceeding."
                                .into(),
                    })
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Gate 4: TestFlippedGate (EXECUTING -> AUDITING)
// ---------------------------------------------------------------------------

/// EXECUTING -> AUDITING: at least 1 test flipped FAIL -> PASS.
/// Source: orchestrator.txt §13 L633.
pub struct TestFlippedGate;

impl EvidenceGate for TestFlippedGate {
    fn name(&self) -> &'static str {
        "test_flipped"
    }

    fn transition(&self) -> (FsmState, FsmState) {
        (FsmState::Executing, FsmState::Auditing)
    }

    fn check(&self, ctx: &TransitionContext) -> Result<(), EvidenceError> {
        match &ctx.test_outcomes {
            None => Err(EvidenceError {
                gate_name: self.name(),
                missing_artifacts: vec!["test_outcomes".into()],
                suggestion: "Run tests and provide outcomes before transitioning to AUDITING."
                    .into(),
            }),
            Some(outcomes) => {
                let flipped = outcomes.iter().any(|t| t.flipped_to_pass);
                if flipped {
                    Ok(())
                } else {
                    Err(EvidenceError {
                        gate_name: self.name(),
                        missing_artifacts: vec!["at_least_one_test_flipped".into()],
                        suggestion:
                            "No test flipped from FAIL to PASS. Implementation must make progress."
                                .into(),
                    })
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Gate 5: ReplanFastTrackGate (EXECUTING -> PLANNING)
// ---------------------------------------------------------------------------

/// EXECUTING -> PLANNING (fast-track replan): ReplanRequest(MINOR) with
/// auditor validation, or ReplanRequest(MAJOR) with human approval.
/// Source: orchestrator.txt §13 L634-638.
pub struct ReplanFastTrackGate;

impl EvidenceGate for ReplanFastTrackGate {
    fn name(&self) -> &'static str {
        "replan_fast_track"
    }

    fn transition(&self) -> (FsmState, FsmState) {
        (FsmState::Executing, FsmState::Planning)
    }

    fn check(&self, ctx: &TransitionContext) -> Result<(), EvidenceError> {
        match ctx.replan_severity {
            None => Err(EvidenceError {
                gate_name: self.name(),
                missing_artifacts: vec!["replan_request".into()],
                suggestion: "Submit a ReplanRequest before transitioning back to PLANNING.".into(),
            }),
            Some(ReplanSeverity::Minor) => {
                // MINOR: auditor must have approved
                match ctx.replan_auditor_approved {
                    Some(true) => Ok(()),
                    Some(false) => Err(EvidenceError {
                        gate_name: self.name(),
                        missing_artifacts: vec!["auditor_approval".into()],
                        suggestion: "Auditor rejected the MINOR replan. Continue EXECUTING.".into(),
                    }),
                    None => Err(EvidenceError {
                        gate_name: self.name(),
                        missing_artifacts: vec!["auditor_validation".into()],
                        suggestion: "MINOR replan requires auditor validation before proceeding."
                            .into(),
                    }),
                }
            }
            Some(ReplanSeverity::Major) => {
                // MAJOR: the interactive gate handles human approval (3A.4).
                // Evidence gate only checks that the request exists. Human
                // decision is delegated to InteractiveGate.
                Ok(())
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Gate 6: ZeroBlocksGate (AUDITING -> MEMORIZING)
// ---------------------------------------------------------------------------

/// AUDITING -> MEMORIZING: zero BLOCK findings remaining.
/// Source: orchestrator.txt §13 L639.
pub struct ZeroBlocksGate;

impl EvidenceGate for ZeroBlocksGate {
    fn name(&self) -> &'static str {
        "zero_blocks"
    }

    fn transition(&self) -> (FsmState, FsmState) {
        (FsmState::Auditing, FsmState::Memorizing)
    }

    fn check(&self, ctx: &TransitionContext) -> Result<(), EvidenceError> {
        match &ctx.audit_findings {
            None => Err(EvidenceError {
                gate_name: self.name(),
                missing_artifacts: vec!["audit_findings".into()],
                suggestion: "Run audit before transitioning to MEMORIZING.".into(),
            }),
            Some(findings) => {
                let blocks: Vec<String> = findings
                    .iter()
                    .filter(|f| f.severity == FindingSeverity::Block)
                    .map(|f| f.message.clone())
                    .collect();

                if blocks.is_empty() {
                    Ok(())
                } else {
                    Err(EvidenceError {
                        gate_name: self.name(),
                        missing_artifacts: blocks,
                        suggestion: "BLOCK findings must be resolved before MEMORIZING.".into(),
                    })
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Gate 7: DkgChangelogGate (MEMORIZING -> next)
// ---------------------------------------------------------------------------

/// MEMORIZING -> PLANNING (next phase) or COMPLETE: DKG reflects new code,
/// changelog has entry.
/// Source: orchestrator.txt §13 L640.
pub struct DkgChangelogGate;

impl EvidenceGate for DkgChangelogGate {
    fn name(&self) -> &'static str {
        "dkg_changelog"
    }

    fn transition(&self) -> (FsmState, FsmState) {
        // Guards both MEMORIZING -> PLANNING and MEMORIZING -> COMPLETE
        (FsmState::Memorizing, FsmState::Planning)
    }

    fn check(&self, ctx: &TransitionContext) -> Result<(), EvidenceError> {
        let mut missing = Vec::new();

        match ctx.dkg_reflects_new_code {
            Some(true) => {}
            Some(false) => missing.push("dkg_reflects_new_code".into()),
            None => missing.push("dkg_status".into()),
        }

        match ctx.changelog_has_entry {
            Some(true) => {}
            Some(false) => missing.push("changelog_entry".into()),
            None => missing.push("changelog_status".into()),
        }

        if missing.is_empty() {
            Ok(())
        } else {
            Err(EvidenceError {
                gate_name: self.name(),
                missing_artifacts: missing,
                suggestion: "Update DKG and changelog before leaving MEMORIZING.".into(),
            })
        }
    }
}

// ---------------------------------------------------------------------------
// Gate registry: maps transitions to their required gates
// ---------------------------------------------------------------------------

/// All 7 evidence gates. Caller looks up the gate(s) for a transition.
pub fn all_gates() -> Vec<Box<dyn EvidenceGate>> {
    vec![
        Box::new(KnowledgeReadinessGate),
        Box::new(GroundingSweepGate),
        Box::new(TestFailReasonsGate),
        Box::new(TestFlippedGate),
        Box::new(ReplanFastTrackGate),
        Box::new(ZeroBlocksGate),
        Box::new(DkgChangelogGate),
    ]
}

/// Find the evidence gate(s) for a given transition.
/// Returns empty vec if no gate is required (e.g., FEASIBILITY_PROBE transitions
/// which are ungated in Phase 3A per phase-3A.txt L169-171).
pub fn gates_for_transition(source: FsmState, target: FsmState) -> Vec<Box<dyn EvidenceGate>> {
    // Special case: DkgChangelogGate guards both MEMORIZING -> PLANNING and
    // MEMORIZING -> COMPLETE. The gate's transition() returns one pair, but
    // we match both targets here.
    let is_memorizing_exit =
        source == FsmState::Memorizing && (target == FsmState::Planning || target == FsmState::Complete);

    all_gates()
        .into_iter()
        .filter(|gate| {
            let (gs, gt) = gate.transition();
            (gs == source && gt == target)
                || (is_memorizing_exit && gs == FsmState::Memorizing && gate.name() == "dkg_changelog")
        })
        .collect()
}

/// Run all gates for a transition. Returns Ok(()) if all pass, or the first
/// EvidenceError encountered.
pub fn check_transition(
    source: FsmState,
    target: FsmState,
    ctx: &TransitionContext,
) -> Result<(), EvidenceError> {
    let gates = gates_for_transition(source, target);
    for gate in &gates {
        gate.check(ctx)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// DAIRA adaptive context: scope reduction on budget overflow
// ---------------------------------------------------------------------------

/// Result of a DAIRA scope reduction attempt.
#[derive(Debug, Clone)]
pub struct DairaScopeResult<T> {
    pub value: T,
    pub attempts: u32,
    pub final_scope: DairaScope,
}

/// Scope level for DAIRA adaptive context reduction.
/// Source: orchestrator.txt §13 L642-643.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DairaScope {
    Full,
    Reduced,
    Minimal,
}

/// DAIRA adaptive context: run a variable-output tool, check output size,
/// reduce scope if > budget * 0.3. Max 3 attempts.
/// Source: orchestrator.txt §13 L642-643.
///
/// `run_fn` takes a scope level and returns (output, output_size_tokens).
/// If the output exceeds the budget threshold, the scope is reduced and
/// the function is retried.
pub fn daira_adaptive_run<F, T>(
    token_budget: usize,
    mut run_fn: F,
) -> DairaScopeResult<T>
where
    F: FnMut(DairaScope) -> (T, usize),
{
    // NARROWING-OK: f64->usize; no TryFrom<f64> for usize in std; token_budget is bounded
    // by model context window which is well within usize on supported platforms.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let threshold = (token_budget as f64 * 0.3) as usize;
    let scopes = [DairaScope::Full, DairaScope::Reduced, DairaScope::Minimal];

    for (attempt, &scope) in scopes.iter().enumerate() {
        let (value, size) = run_fn(scope);
        if size <= threshold || attempt == scopes.len() - 1 {
            return DairaScopeResult {
                value,
                attempts: u32::try_from(attempt + 1).unwrap_or(u32::MAX),
                final_scope: scope,
            };
        }
    }

    // Unreachable: loop always returns on last iteration
    unreachable!()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn ctx_with_knowledge(status: KnowledgeReadiness) -> TransitionContext {
        TransitionContext {
            knowledge_readiness: Some(status),
            ..Default::default()
        }
    }

    fn ctx_with_grounding(claims: Vec<String>) -> TransitionContext {
        TransitionContext {
            ungrounded_claims: Some(claims),
            ..Default::default()
        }
    }

    fn passing_test(name: &str) -> TestOutcome {
        TestOutcome {
            name: name.into(),
            passed: true,
            is_syntax_error: false,
            flipped_to_pass: false,
        }
    }

    fn failing_test(name: &str, syntax: bool, flipped: bool) -> TestOutcome {
        TestOutcome {
            name: name.into(),
            passed: false,
            is_syntax_error: syntax,
            flipped_to_pass: flipped,
        }
    }

    fn flipped_test(name: &str) -> TestOutcome {
        TestOutcome {
            name: name.into(),
            passed: true,
            is_syntax_error: false,
            flipped_to_pass: true,
        }
    }

    // -----------------------------------------------------------------------
    // Gate 1: KnowledgeReadinessGate
    // -----------------------------------------------------------------------

    #[test]
    fn test_gate1_ready_passes() {
        let gate = KnowledgeReadinessGate;
        assert!(gate.check(&ctx_with_knowledge(KnowledgeReadiness::Ready)).is_ok());
    }

    #[test]
    fn test_gate1_degraded_passes_with_warning() {
        // DEGRADED passes the gate (warning logged at caller level).
        // Source: phase-3A.txt adversarial L183-184.
        let gate = KnowledgeReadinessGate;
        assert!(gate.check(&ctx_with_knowledge(KnowledgeReadiness::Degraded)).is_ok());
    }

    #[test]
    fn test_gate1_unavailable_blocks() {
        let gate = KnowledgeReadinessGate;
        let err = gate
            .check(&ctx_with_knowledge(KnowledgeReadiness::Unavailable))
            .unwrap_err();
        assert_eq!(err.gate_name, "knowledge_readiness");
        assert!(err.missing_artifacts.contains(&"knowledge_engine_available".into()));
    }

    #[test]
    fn test_gate1_missing_status_blocks() {
        let gate = KnowledgeReadinessGate;
        let err = gate.check(&TransitionContext::default()).unwrap_err();
        assert_eq!(err.gate_name, "knowledge_readiness");
    }

    // -----------------------------------------------------------------------
    // Gate 2: GroundingSweepGate
    // -----------------------------------------------------------------------

    #[test]
    fn test_gate2_all_grounded_passes() {
        let gate = GroundingSweepGate;
        assert!(gate.check(&ctx_with_grounding(vec![])).is_ok());
    }

    #[test]
    fn test_gate2_ungrounded_claims_blocks() {
        // Source: phase-3A.txt adversarial L177-178.
        let gate = GroundingSweepGate;
        let claims = vec!["blake3 API returns u32".into(), "tokio 1.x has spawn_blocking".into()];
        let err = gate.check(&ctx_with_grounding(claims.clone())).unwrap_err();
        assert_eq!(err.gate_name, "grounding_sweep");
        assert_eq!(err.missing_artifacts, claims);
    }

    #[test]
    fn test_gate2_missing_report_blocks() {
        let gate = GroundingSweepGate;
        let err = gate.check(&TransitionContext::default()).unwrap_err();
        assert_eq!(err.gate_name, "grounding_sweep");
        assert!(err.missing_artifacts.contains(&"grounding_sweep_report".into()));
    }

    // -----------------------------------------------------------------------
    // Gate 3: TestFailReasonsGate
    // -----------------------------------------------------------------------

    #[test]
    fn test_gate3_correct_failures_pass() {
        let gate = TestFailReasonsGate;
        let ctx = TransitionContext {
            test_outcomes: Some(vec![
                failing_test("test_new_feature", false, false),
                passing_test("test_existing"),
            ]),
            ..Default::default()
        };
        assert!(gate.check(&ctx).is_ok());
    }

    #[test]
    fn test_gate3_syntax_error_blocks() {
        let gate = TestFailReasonsGate;
        let ctx = TransitionContext {
            test_outcomes: Some(vec![
                failing_test("test_broken_syntax", true, false),
                failing_test("test_correct_fail", false, false),
            ]),
            ..Default::default()
        };
        let err = gate.check(&ctx).unwrap_err();
        assert_eq!(err.gate_name, "test_fail_reasons");
        assert_eq!(err.missing_artifacts, vec!["test_broken_syntax"]);
    }

    #[test]
    fn test_gate3_all_passing_ok() {
        // All tests passing is fine (pre-existing green suite).
        let gate = TestFailReasonsGate;
        let ctx = TransitionContext {
            test_outcomes: Some(vec![passing_test("test_a"), passing_test("test_b")]),
            ..Default::default()
        };
        assert!(gate.check(&ctx).is_ok());
    }

    #[test]
    fn test_gate3_missing_outcomes_blocks() {
        let gate = TestFailReasonsGate;
        let err = gate.check(&TransitionContext::default()).unwrap_err();
        assert_eq!(err.gate_name, "test_fail_reasons");
    }

    // -----------------------------------------------------------------------
    // Gate 4: TestFlippedGate
    // -----------------------------------------------------------------------

    #[test]
    fn test_gate4_flipped_passes() {
        let gate = TestFlippedGate;
        let ctx = TransitionContext {
            test_outcomes: Some(vec![
                flipped_test("test_new_feature"),
                passing_test("test_existing"),
            ]),
            ..Default::default()
        };
        assert!(gate.check(&ctx).is_ok());
    }

    #[test]
    fn test_gate4_no_flip_blocks() {
        let gate = TestFlippedGate;
        let ctx = TransitionContext {
            test_outcomes: Some(vec![
                passing_test("test_existing"),
                failing_test("test_still_failing", false, false),
            ]),
            ..Default::default()
        };
        let err = gate.check(&ctx).unwrap_err();
        assert_eq!(err.gate_name, "test_flipped");
        assert!(err.missing_artifacts.contains(&"at_least_one_test_flipped".into()));
    }

    #[test]
    fn test_gate4_missing_outcomes_blocks() {
        let gate = TestFlippedGate;
        let err = gate.check(&TransitionContext::default()).unwrap_err();
        assert_eq!(err.gate_name, "test_flipped");
    }

    // -----------------------------------------------------------------------
    // Gate 5: ReplanFastTrackGate
    // -----------------------------------------------------------------------

    #[test]
    fn test_gate5_minor_approved_passes() {
        let gate = ReplanFastTrackGate;
        let ctx = TransitionContext {
            replan_severity: Some(ReplanSeverity::Minor),
            replan_auditor_approved: Some(true),
            ..Default::default()
        };
        assert!(gate.check(&ctx).is_ok());
    }

    #[test]
    fn test_gate5_minor_rejected_blocks() {
        let gate = ReplanFastTrackGate;
        let ctx = TransitionContext {
            replan_severity: Some(ReplanSeverity::Minor),
            replan_auditor_approved: Some(false),
            ..Default::default()
        };
        let err = gate.check(&ctx).unwrap_err();
        assert_eq!(err.gate_name, "replan_fast_track");
        assert!(err.missing_artifacts.contains(&"auditor_approval".into()));
    }

    #[test]
    fn test_gate5_minor_no_validation_blocks() {
        let gate = ReplanFastTrackGate;
        let ctx = TransitionContext {
            replan_severity: Some(ReplanSeverity::Minor),
            replan_auditor_approved: None,
            ..Default::default()
        };
        let err = gate.check(&ctx).unwrap_err();
        assert_eq!(err.gate_name, "replan_fast_track");
    }

    #[test]
    fn test_gate5_major_passes_no_auditor() {
        // MAJOR replan: evidence gate passes (human decision delegated to InteractiveGate 3A.4).
        let gate = ReplanFastTrackGate;
        let ctx = TransitionContext {
            replan_severity: Some(ReplanSeverity::Major),
            ..Default::default()
        };
        assert!(gate.check(&ctx).is_ok());
    }

    #[test]
    fn test_gate5_missing_replan_blocks() {
        let gate = ReplanFastTrackGate;
        let err = gate.check(&TransitionContext::default()).unwrap_err();
        assert_eq!(err.gate_name, "replan_fast_track");
    }

    // -----------------------------------------------------------------------
    // Gate 6: ZeroBlocksGate
    // -----------------------------------------------------------------------

    #[test]
    fn test_gate6_no_blocks_passes() {
        let gate = ZeroBlocksGate;
        let ctx = TransitionContext {
            audit_findings: Some(vec![
                AuditFinding {
                    severity: FindingSeverity::Warn,
                    message: "naming convention".into(),
                },
                AuditFinding {
                    severity: FindingSeverity::Info,
                    message: "perf note".into(),
                },
            ]),
            ..Default::default()
        };
        assert!(gate.check(&ctx).is_ok());
    }

    #[test]
    fn test_gate6_blocks_present_fails() {
        let gate = ZeroBlocksGate;
        let ctx = TransitionContext {
            audit_findings: Some(vec![
                AuditFinding {
                    severity: FindingSeverity::Block,
                    message: "SQL injection vulnerability".into(),
                },
                AuditFinding {
                    severity: FindingSeverity::Warn,
                    message: "minor style issue".into(),
                },
            ]),
            ..Default::default()
        };
        let err = gate.check(&ctx).unwrap_err();
        assert_eq!(err.gate_name, "zero_blocks");
        assert_eq!(err.missing_artifacts, vec!["SQL injection vulnerability"]);
    }

    #[test]
    fn test_gate6_missing_findings_blocks() {
        let gate = ZeroBlocksGate;
        let err = gate.check(&TransitionContext::default()).unwrap_err();
        assert_eq!(err.gate_name, "zero_blocks");
    }

    // -----------------------------------------------------------------------
    // Gate 7: DkgChangelogGate
    // -----------------------------------------------------------------------

    #[test]
    fn test_gate7_both_present_passes() {
        let gate = DkgChangelogGate;
        let ctx = TransitionContext {
            dkg_reflects_new_code: Some(true),
            changelog_has_entry: Some(true),
            ..Default::default()
        };
        assert!(gate.check(&ctx).is_ok());
    }

    #[test]
    fn test_gate7_missing_dkg_blocks() {
        let gate = DkgChangelogGate;
        let ctx = TransitionContext {
            dkg_reflects_new_code: Some(false),
            changelog_has_entry: Some(true),
            ..Default::default()
        };
        let err = gate.check(&ctx).unwrap_err();
        assert_eq!(err.gate_name, "dkg_changelog");
        assert!(err.missing_artifacts.contains(&"dkg_reflects_new_code".into()));
    }

    #[test]
    fn test_gate7_missing_changelog_blocks() {
        let gate = DkgChangelogGate;
        let ctx = TransitionContext {
            dkg_reflects_new_code: Some(true),
            changelog_has_entry: Some(false),
            ..Default::default()
        };
        let err = gate.check(&ctx).unwrap_err();
        assert_eq!(err.gate_name, "dkg_changelog");
        assert!(err.missing_artifacts.contains(&"changelog_entry".into()));
    }

    #[test]
    fn test_gate7_missing_both_blocks() {
        let gate = DkgChangelogGate;
        let err = gate.check(&TransitionContext::default()).unwrap_err();
        assert_eq!(err.gate_name, "dkg_changelog");
        assert_eq!(err.missing_artifacts.len(), 2);
    }

    // -----------------------------------------------------------------------
    // Adversarial: empty artifacts always reject (except DEGRADED)
    // Source: phase-3A.txt L297-298.
    // -----------------------------------------------------------------------

    #[test]
    fn test_all_gates_reject_empty_context() {
        let empty = TransitionContext::default();
        let gates = all_gates();
        for gate in &gates {
            let result = gate.check(&empty);
            assert!(
                result.is_err(),
                "gate '{}' should reject empty context",
                gate.name()
            );
        }
    }

    // -----------------------------------------------------------------------
    // Adversarial: PLANNING -> TESTING_FIRST with ungrounded claims must fail
    // Source: phase-3A.txt adversarial L177-178.
    // -----------------------------------------------------------------------

    #[test]
    fn test_adversarial_planning_to_testing_with_ungrounded() {
        let ctx = TransitionContext {
            source: Some(FsmState::Planning),
            target: Some(FsmState::TestingFirst),
            ungrounded_claims: Some(vec!["unverified API claim".into()]),
            ..Default::default()
        };
        let err = check_transition(FsmState::Planning, FsmState::TestingFirst, &ctx)
            .unwrap_err();
        assert_eq!(err.gate_name, "grounding_sweep");
        assert!(err.missing_artifacts.contains(&"unverified API claim".into()));
    }

    // -----------------------------------------------------------------------
    // Adversarial: IDLE -> PLANNING with UNAVAILABLE knowledge engine
    // Source: phase-3A.txt adversarial L180-181.
    // -----------------------------------------------------------------------

    #[test]
    fn test_adversarial_idle_to_planning_unavailable() {
        let ctx = ctx_with_knowledge(KnowledgeReadiness::Unavailable);
        let err =
            check_transition(FsmState::Idle, FsmState::Planning, &ctx).unwrap_err();
        assert_eq!(err.gate_name, "knowledge_readiness");
    }

    // -----------------------------------------------------------------------
    // Adversarial: DEGRADED must NOT block (only warn)
    // Source: phase-3A.txt adversarial L183-184.
    // -----------------------------------------------------------------------

    #[test]
    fn test_adversarial_degraded_does_not_block() {
        let ctx = ctx_with_knowledge(KnowledgeReadiness::Degraded);
        assert!(
            check_transition(FsmState::Idle, FsmState::Planning, &ctx).is_ok(),
            "DEGRADED should pass the gate, not block"
        );
    }

    // -----------------------------------------------------------------------
    // Gate registry: ungated transitions return empty vec
    // Source: phase-3A.txt L169-171.
    // -----------------------------------------------------------------------

    #[test]
    fn test_ungated_transitions_have_no_gates() {
        // FEASIBILITY_PROBE transitions are ungated in Phase 3A
        assert!(gates_for_transition(FsmState::Planning, FsmState::FeasibilityProbe).is_empty());
        assert!(gates_for_transition(FsmState::FeasibilityProbe, FsmState::TestingFirst).is_empty());
        assert!(gates_for_transition(FsmState::FeasibilityProbe, FsmState::Planning).is_empty());

        // CONFIGURING -> IDLE is ungated
        assert!(gates_for_transition(FsmState::Configuring, FsmState::Idle).is_empty());

        // COMPLETE -> IDLE is ungated
        assert!(gates_for_transition(FsmState::Complete, FsmState::Idle).is_empty());

        // RESEARCHING transitions are ungated (research-only, no code changes)
        assert!(gates_for_transition(FsmState::Idle, FsmState::Researching).is_empty());
        assert!(gates_for_transition(FsmState::Researching, FsmState::Idle).is_empty());
        assert!(gates_for_transition(FsmState::Researching, FsmState::Researching).is_empty());
        assert!(gates_for_transition(FsmState::Researching, FsmState::Planning).is_empty());
    }

    // -----------------------------------------------------------------------
    // Gate registry: gated transitions have exactly one gate
    // -----------------------------------------------------------------------

    #[test]
    fn test_gated_transitions_have_gate() {
        assert_eq!(gates_for_transition(FsmState::Idle, FsmState::Planning).len(), 1);
        assert_eq!(gates_for_transition(FsmState::Planning, FsmState::TestingFirst).len(), 1);
        assert_eq!(gates_for_transition(FsmState::TestingFirst, FsmState::Executing).len(), 1);
        assert_eq!(gates_for_transition(FsmState::Executing, FsmState::Auditing).len(), 1);
        assert_eq!(gates_for_transition(FsmState::Executing, FsmState::Planning).len(), 1);
        assert_eq!(gates_for_transition(FsmState::Auditing, FsmState::Memorizing).len(), 1);
    }

    // -----------------------------------------------------------------------
    // Gate registry: MEMORIZING exits are gated
    // -----------------------------------------------------------------------

    #[test]
    fn test_memorizing_exits_gated() {
        assert_eq!(
            gates_for_transition(FsmState::Memorizing, FsmState::Planning).len(),
            1
        );
        assert_eq!(
            gates_for_transition(FsmState::Memorizing, FsmState::Complete).len(),
            1
        );
    }

    // -----------------------------------------------------------------------
    // check_transition: happy path passes
    // -----------------------------------------------------------------------

    #[test]
    fn test_check_transition_happy_path() {
        // IDLE -> PLANNING with READY knowledge
        assert!(
            check_transition(
                FsmState::Idle,
                FsmState::Planning,
                &ctx_with_knowledge(KnowledgeReadiness::Ready)
            )
            .is_ok()
        );

        // PLANNING -> TESTING_FIRST with all grounded
        assert!(
            check_transition(
                FsmState::Planning,
                FsmState::TestingFirst,
                &ctx_with_grounding(vec![])
            )
            .is_ok()
        );
    }

    // -----------------------------------------------------------------------
    // All 7 gate names are unique
    // -----------------------------------------------------------------------

    #[test]
    fn test_gate_names_unique() {
        let gates = all_gates();
        let mut names: Vec<&str> = gates.iter().map(|g| g.name()).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), 7);
    }

    // -----------------------------------------------------------------------
    // EvidenceGate trait is object-safe (compile-time check)
    // -----------------------------------------------------------------------

    #[test]
    fn test_evidence_gate_is_object_safe() {
        let _: Box<dyn EvidenceGate> = Box::new(KnowledgeReadinessGate);
        let _: Vec<Box<dyn EvidenceGate>> = all_gates();
    }

    // -----------------------------------------------------------------------
    // DAIRA adaptive context tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_daira_within_budget_first_try() {
        let result = daira_adaptive_run(1000, |_scope| ("output", 200));
        assert_eq!(result.attempts, 1);
        assert_eq!(result.final_scope, DairaScope::Full);
        assert_eq!(result.value, "output");
    }

    #[test]
    fn test_daira_exceeds_budget_reduces() {
        let result = daira_adaptive_run(1000, |scope| {
            let size = match scope {
                DairaScope::Full => 500,    // > 300 threshold
                DairaScope::Reduced => 200, // <= 300 threshold
                DairaScope::Minimal => 100,
            };
            ("output", size)
        });
        assert_eq!(result.attempts, 2);
        assert_eq!(result.final_scope, DairaScope::Reduced);
    }

    #[test]
    fn test_daira_always_exceeds_uses_minimal() {
        let result = daira_adaptive_run(1000, |scope| {
            let size = match scope {
                DairaScope::Full => 900,
                DairaScope::Reduced => 500,
                DairaScope::Minimal => 400, // still over, but max attempts reached
            };
            ("output", size)
        });
        assert_eq!(result.attempts, 3);
        assert_eq!(result.final_scope, DairaScope::Minimal);
    }

    #[test]
    fn test_daira_zero_budget_always_minimal() {
        let result = daira_adaptive_run(0, |_scope| ("output", 0));
        assert_eq!(result.attempts, 1);
        assert_eq!(result.final_scope, DairaScope::Full);
    }

    // -----------------------------------------------------------------------
    // Property: empty outcomes always reject gates 3 and 4
    // -----------------------------------------------------------------------

    #[test]
    fn test_empty_outcomes_passes_gate3_but_blocks_gate4() {
        // Gate 3: empty outcomes = no syntax errors = pass
        let gate3 = TestFailReasonsGate;
        let ctx = TransitionContext {
            test_outcomes: Some(vec![]),
            ..Default::default()
        };
        assert!(gate3.check(&ctx).is_ok());

        // Gate 4: empty outcomes = no flip = fail
        let gate4 = TestFlippedGate;
        assert!(gate4.check(&ctx).is_err());
    }

    // -----------------------------------------------------------------------
    // EvidenceError Display formatting
    // -----------------------------------------------------------------------

    #[test]
    fn test_evidence_error_display() {
        let err = EvidenceError {
            gate_name: "test_gate",
            missing_artifacts: vec!["artifact_a".into(), "artifact_b".into()],
            suggestion: "Fix it.".into(),
        };
        let display = format!("{err}");
        assert!(display.contains("test_gate"));
        assert!(display.contains("artifact_a, artifact_b"));
        assert!(display.contains("Fix it."));
    }
}
