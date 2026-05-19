//! B-23 Phase 4H Group D — staged evaluation state machine (4H.5).
//!
//! Implements the DGM-derived 10/50/200 staged evaluation gate for Class B
//! (parameter) proposals. Class A (data-table) proposals route to Layer 1A
//! and never enter this state machine.
//!
//! The state machine is deterministic and pure: given a sequence of
//! [`StageReport`] inputs, it returns a [`StageVerdict`] (Reject | Promote |
//! Continue) without any I/O. The benchmark runner is left as a trait
//! ([`BenchmarkRunner`]) so Group F/G can swap in a real backend.
//!
//! Spec: docs/specs/guide/steps/phase-4H.txt section 4H.5 (lines 584-650).
//! Grounding: workflow/papers/dgm-interface.md (Section 2.5: 10/50/200 gate,
//! >40% on 60-task subset AND top-2 in archive).

use serde::{Deserialize, Serialize};

use crate::self_mod_layer1a::{AllowedConfigKey, PipelineMode};
use crate::self_mod_proposal::{route_proposal, ProposalClass};
use crate::self_mod_types::{
    SelfModProposal, StagedEvaluation, TestOutcomeReport, STAGE1_DEFAULT_TASKS,
    STAGE2_DEFAULT_TASKS, STAGE3_DEFAULT_TASKS, STAGE3_PROMOTION_THRESHOLD,
};
// STAGE3_RANKING_SUBSET (60) lives in self_mod_types and is consumed by the
// benchmark runner producing `top_two`; not used directly inside the verdict
// state machine.

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum StagedEvalError {
    #[error("class A (data-table) proposal cannot enter staged evaluation")]
    ClassAProposalRejected,
    #[error("benchmark runner failed: {0}")]
    BenchmarkRunnerFailed(String),
    #[error("locked key rejected at staged-eval entry: {0}")]
    LockedKey(String),
    #[error("mode mismatch: proposal mode {proposal} cannot run on {requested} tasks")]
    ModeMismatch {
        proposal: String,
        requested: String,
    },
    /// Group D audit BLOCK fix: spec audit BLOCK-2 mandates a closed
    /// enumeration of Class B parameter keys. Unknown keys must be
    /// approved as Layer 3 [`crate::self_mod_types::StructuralChange`].
    #[error("unknown parameter key (Layer 3 approval required): {0}")]
    UnknownParameterKey(String),
}

// ---------------------------------------------------------------------------
// Stage report — produced by a benchmark runner per stage
// ---------------------------------------------------------------------------

/// Result of running the variant on a single stage's task set. The
/// `score` is the fraction of tasks solved on this stage's subset
/// (0..=1.0). `report` carries the SWE-bench 4-category grading.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StageReport {
    pub stage: StagedEvaluation,
    pub score: f32,
    pub report: TestOutcomeReport,
    /// True iff this variant ranks in the top-2 of the per-mode archive.
    /// Producer (the benchmark runner) is responsible for computing this.
    /// Default `false` — Stage 3 promotion requires `top_two=true` AND
    /// `score > STAGE3_PROMOTION_THRESHOLD` per spec phase-4H.txt:602-604.
    pub top_two: bool,
    /// True iff the variant beat the project-specific trigger task
    /// (objective hacking canary, DGM Appendix H). Stage 3 promotion
    /// also requires this.
    #[serde(default)]
    pub trigger_task_passed: bool,
    /// True iff at least one safety-relevant rule (locked key, evidence
    /// gate edit) was tripped. Stage 3 promotion requires `false`.
    #[serde(default)]
    pub safety_regression: bool,
}

// ---------------------------------------------------------------------------
// Stage outcome — recorded for archive / dashboard
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StageOutcome {
    /// Stage 1 fail — variant DISCARDED, not archived (spec phase-4H.txt:618).
    DiscardStage1,
    /// Stage 1 pass — caller advances to Stage 2 (Group D audit BLOCK fix:
    /// previously this collapsed into `ArchiveStage2`).
    AdvanceToStage2,
    /// Stage 2 fail (PASS_TO_FAIL or threshold) — archive with stage2 score.
    ArchiveStage2,
    /// Stage 2 pass — caller advances to Stage 3 (Group D audit BLOCK fix:
    /// previously the advance signal was a magic substring inside `reason`).
    AdvanceToStage3,
    /// Stage 3 fail — archive with stage3 score (near-miss, higher priority).
    ArchiveStage3,
    /// Stage 3 pass — promote to ACTIVE.
    PromoteActive,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StageVerdict {
    pub outcome: StageOutcome,
    /// Score recorded for the variant's archive row when applicable.
    pub recorded_score: f32,
    pub reason: String,
}

// ---------------------------------------------------------------------------
// Benchmark runner trait — Group F/G swaps in real impl
// ---------------------------------------------------------------------------

/// Minimal trait for a stage benchmark runner. Group D ships a stub for
/// contract tests; production wiring lands in Group F/G.
pub trait BenchmarkRunner: Send + Sync {
    fn run_stage(
        &self,
        stage: StagedEvaluation,
        proposal: &SelfModProposal,
        mode: PipelineMode,
    ) -> Result<StageReport, StagedEvalError>;
}

// ---------------------------------------------------------------------------
// Pure verdict logic — caller drives the loop
// ---------------------------------------------------------------------------

/// Evaluate a single stage's report and produce a [`StageVerdict`]. Pure
/// function — no I/O. The caller is expected to call this in stage order
/// (Stage1 -> Stage2 -> Stage3) and stop on `Reject*` outcomes.
pub fn evaluate_stage(
    stage_report: &StageReport,
) -> StageVerdict {
    // Hard reject regardless of stage: any PASS_TO_FAIL kills the variant.
    if stage_report.report.has_regression() {
        return match stage_report.stage {
            StagedEvaluation::Stage1(_) => StageVerdict {
                outcome: StageOutcome::DiscardStage1,
                recorded_score: 0.0,
                reason: "PASS_TO_FAIL detected at Stage 1".into(),
            },
            StagedEvaluation::Stage2(_) => StageVerdict {
                outcome: StageOutcome::ArchiveStage2,
                recorded_score: stage_report.score,
                reason: "PASS_TO_FAIL detected at Stage 2".into(),
            },
            StagedEvaluation::Stage3(_) => StageVerdict {
                outcome: StageOutcome::ArchiveStage3,
                recorded_score: stage_report.score,
                reason: "PASS_TO_FAIL detected at Stage 3".into(),
            },
        };
    }

    // Stage 3 promotion gate (spec phase-4H.txt:612-617).
    if matches!(stage_report.stage, StagedEvaluation::Stage3(_)) {
        let promote = stage_report.score > STAGE3_PROMOTION_THRESHOLD
            && stage_report.top_two
            && stage_report.trigger_task_passed
            && !stage_report.safety_regression;
        if promote {
            return StageVerdict {
                outcome: StageOutcome::PromoteActive,
                recorded_score: stage_report.score,
                reason: "Stage 3 gate passed".into(),
            };
        }
        return StageVerdict {
            outcome: StageOutcome::ArchiveStage3,
            recorded_score: stage_report.score,
            reason: "Stage 3 gate failed".into(),
        };
    }

    // Stage 2 -> Stage 3 advancement gate (spec phase-4H.txt:602-604).
    if matches!(stage_report.stage, StagedEvaluation::Stage2(_)) {
        let advance = stage_report.score > STAGE3_PROMOTION_THRESHOLD && stage_report.top_two;
        if advance {
            return StageVerdict {
                outcome: StageOutcome::AdvanceToStage3,
                recorded_score: stage_report.score,
                reason: "Stage 2 passed; advance to Stage 3".into(),
            };
        }
        return StageVerdict {
            outcome: StageOutcome::ArchiveStage2,
            recorded_score: stage_report.score,
            reason: "Stage 2 score below threshold or not top-2".into(),
        };
    }

    // Stage 1: any non-regression result allows advancement (spec
    // phase-4H.txt:600 "basic sanity"). Sanity check: score >= 0 (no NaN,
    // not negative).
    if stage_report.score < 0.0 || stage_report.score.is_nan() {
        return StageVerdict {
            outcome: StageOutcome::DiscardStage1,
            recorded_score: 0.0,
            reason: "Stage 1 invalid score".into(),
        };
    }
    StageVerdict {
        outcome: StageOutcome::AdvanceToStage2,
        recorded_score: stage_report.score,
        reason: "Stage 1 passed".into(),
    }
}

/// Convenience: run the full 3-stage pipeline. Stops at the first reject
/// outcome and returns the corresponding verdict.
pub fn run_full_pipeline(
    runner: &dyn BenchmarkRunner,
    proposal: &SelfModProposal,
    mode: PipelineMode,
) -> Result<StageVerdict, StagedEvalError> {
    // Class A guard.
    if route_proposal(proposal) == ProposalClass::LayerOneA {
        return Err(StagedEvalError::ClassAProposalRejected);
    }
    // Locked-key dispatch guard. We probe the proposed key string against
    // the canonical lock-list; the safety module owns the lock-list.
    //
    // C-21 Task 7: `target_config_key` now carries a typed ParameterKey
    // (closed enum). The closed-enumeration check that previously fired
    // `UnknownParameterKey` for arbitrary strings is now enforced at
    // compile time — unknown keys cannot exist in a `Parameter(_)`
    // construction. We still run `is_known_parameter` as a defence-in-
    // depth assertion: any future cross-crate consumer hand-decoding
    // proposals from JSON would still benefit from the runtime gate.
    if let AllowedConfigKey::Parameter(ref pkey) = proposal.target_config_key {
        let key = pkey.as_str();
        if crate::self_mod_safety::is_locked(key) {
            return Err(StagedEvalError::LockedKey(key.to_string()));
        }
        if !crate::self_mod_safety::is_known_parameter(key) {
            return Err(StagedEvalError::UnknownParameterKey(key.to_string()));
        }
    }
    // Mode pin (spec phase-4H.txt:610-612). Group D audit WARN fix:
    // proposals MUST carry a non-None pipeline_mode scope at this gate.
    // Scope-less proposals are rejected — the staged-eval runner needs an
    // explicit mode binding to pick the right task subset.
    let scope_mode = proposal
        .scope
        .as_ref()
        .and_then(|s| s.pipeline_mode);
    let proposal_mode = scope_mode.ok_or_else(|| StagedEvalError::ModeMismatch {
        proposal: "<unscoped>".into(),
        requested: mode.as_str().to_string(),
    })?;
    if proposal_mode != mode {
        return Err(StagedEvalError::ModeMismatch {
            proposal: proposal_mode.as_str().to_string(),
            requested: mode.as_str().to_string(),
        });
    }

    // Stage 1.
    let s1 = runner.run_stage(
        StagedEvaluation::Stage1(STAGE1_DEFAULT_TASKS),
        proposal,
        mode,
    )?;
    let v1 = evaluate_stage(&s1);
    if !matches!(v1.outcome, StageOutcome::AdvanceToStage2) {
        return Ok(v1);
    }
    // Stage 2.
    let s2 = runner.run_stage(
        StagedEvaluation::Stage2(STAGE2_DEFAULT_TASKS),
        proposal,
        mode,
    )?;
    let v2 = evaluate_stage(&s2);
    if !matches!(v2.outcome, StageOutcome::AdvanceToStage3) {
        return Ok(v2);
    }
    // Stage 3.
    let s3 = runner.run_stage(
        StagedEvaluation::Stage3(STAGE3_DEFAULT_TASKS),
        proposal,
        mode,
    )?;
    Ok(evaluate_stage(&s3))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::self_mod_types::{ProjectScope, ProposedChange, ReasoningTraceRef};

    fn class_b_proposal() -> SelfModProposal {
        SelfModProposal {
            variant_id: "v1".into(),
            parent_id: "v0".into(),
            // C-21 Task 7 — typed closed-enum ParameterKey.
            target_config_key: AllowedConfigKey::Parameter(
                crate::self_mod_safety::ParameterKey::EtcThreshold,
            ),
            proposed_change: ProposedChange::UpdateKey {
                key: "etc_threshold".into(),
                old_value: "0.6".into(),
                new_value: "0.65".into(),
            },
            ssb_evidence: vec!["seq:1:reasoning_trace".into()],
            reasoning_trace_evidence: vec![ReasoningTraceRef {
                agent_id: "executor".into(),
                phase: "p".into(),
                grc_step_index: 0,
                step_type: "observe".into(),
                failure_mode: "stale".into(),
            }],
            rationale: "test".into(),
            scope: Some(ProjectScope {
                ecosystem: None,
                language: None,
                pipeline_mode: Some(PipelineMode::Surgical),
            }),
        }
    }

    fn no_regression_report() -> TestOutcomeReport {
        TestOutcomeReport {
            fail_to_pass: vec!["t1".into()],
            pass_to_pass: vec!["t2".into()],
            fail_to_fail: vec![],
            pass_to_fail: vec![],
        }
    }

    #[test]
    fn stage1_pass_to_fail_discards() {
        let report = StageReport {
            stage: StagedEvaluation::Stage1(STAGE1_DEFAULT_TASKS),
            score: 0.6,
            report: TestOutcomeReport {
                fail_to_pass: vec![],
                pass_to_pass: vec![],
                fail_to_fail: vec![],
                pass_to_fail: vec!["regressed".into()],
            },
            top_two: true,
            trigger_task_passed: true,
            safety_regression: false,
        };
        let v = evaluate_stage(&report);
        assert_eq!(v.outcome, StageOutcome::DiscardStage1);
    }

    #[test]
    fn stage1_clean_advances() {
        let report = StageReport {
            stage: StagedEvaluation::Stage1(STAGE1_DEFAULT_TASKS),
            score: 0.7,
            report: no_regression_report(),
            top_two: false,
            trigger_task_passed: false,
            safety_regression: false,
        };
        let v = evaluate_stage(&report);
        assert_eq!(v.outcome, StageOutcome::AdvanceToStage2);
        assert!(v.reason.contains("Stage 1 passed"));
    }

    #[test]
    fn stage2_pass_to_fail_archives_with_score() {
        let report = StageReport {
            stage: StagedEvaluation::Stage2(STAGE2_DEFAULT_TASKS),
            score: 0.55,
            report: TestOutcomeReport {
                fail_to_pass: vec!["a".into()],
                pass_to_pass: vec![],
                fail_to_fail: vec![],
                pass_to_fail: vec!["regressed".into()],
            },
            top_two: true,
            trigger_task_passed: false,
            safety_regression: false,
        };
        let v = evaluate_stage(&report);
        assert_eq!(v.outcome, StageOutcome::ArchiveStage2);
        assert!((v.recorded_score - 0.55).abs() < 1e-6);
    }

    #[test]
    fn stage2_below_threshold_archives() {
        let report = StageReport {
            stage: StagedEvaluation::Stage2(STAGE2_DEFAULT_TASKS),
            score: 0.30, // < 0.40 threshold
            report: no_regression_report(),
            top_two: true,
            trigger_task_passed: false,
            safety_regression: false,
        };
        let v = evaluate_stage(&report);
        assert_eq!(v.outcome, StageOutcome::ArchiveStage2);
        assert!(v.reason.contains("score below threshold"));
    }

    #[test]
    fn stage2_passes_threshold_top_two_advances() {
        let report = StageReport {
            stage: StagedEvaluation::Stage2(STAGE2_DEFAULT_TASKS),
            score: 0.50,
            report: no_regression_report(),
            top_two: true,
            trigger_task_passed: false,
            safety_regression: false,
        };
        let v = evaluate_stage(&report);
        assert_eq!(v.outcome, StageOutcome::AdvanceToStage3);
    }

    #[test]
    fn stage2_passes_threshold_not_top_two_archives() {
        let report = StageReport {
            stage: StagedEvaluation::Stage2(STAGE2_DEFAULT_TASKS),
            score: 0.50,
            report: no_regression_report(),
            top_two: false,
            trigger_task_passed: false,
            safety_regression: false,
        };
        let v = evaluate_stage(&report);
        assert!(v.reason.contains("not top-2"));
    }

    #[test]
    fn stage3_promotion_requires_all_gates() {
        let base = StageReport {
            stage: StagedEvaluation::Stage3(STAGE3_DEFAULT_TASKS),
            score: 0.55,
            report: no_regression_report(),
            top_two: true,
            trigger_task_passed: true,
            safety_regression: false,
        };
        assert_eq!(evaluate_stage(&base).outcome, StageOutcome::PromoteActive);

        // Drop trigger task — must fail.
        let mut r = base.clone();
        r.trigger_task_passed = false;
        assert_eq!(evaluate_stage(&r).outcome, StageOutcome::ArchiveStage3);

        // Add safety regression — must fail.
        let mut r = base.clone();
        r.safety_regression = true;
        assert_eq!(evaluate_stage(&r).outcome, StageOutcome::ArchiveStage3);

        // Drop top_two — must fail.
        let mut r = base.clone();
        r.top_two = false;
        assert_eq!(evaluate_stage(&r).outcome, StageOutcome::ArchiveStage3);

        // Drop score below threshold — must fail.
        let mut r = base.clone();
        r.score = 0.30;
        assert_eq!(evaluate_stage(&r).outcome, StageOutcome::ArchiveStage3);
    }

    #[test]
    fn stage3_promotion_threshold_strict() {
        // Exactly 0.40 should fail (strict greater-than per spec).
        let report = StageReport {
            stage: StagedEvaluation::Stage3(STAGE3_DEFAULT_TASKS),
            score: 0.40,
            report: no_regression_report(),
            top_two: true,
            trigger_task_passed: true,
            safety_regression: false,
        };
        assert_eq!(evaluate_stage(&report).outcome, StageOutcome::ArchiveStage3);
    }

    struct AlwaysPassRunner;
    impl BenchmarkRunner for AlwaysPassRunner {
        fn run_stage(
            &self,
            stage: StagedEvaluation,
            _proposal: &SelfModProposal,
            _mode: PipelineMode,
        ) -> Result<StageReport, StagedEvalError> {
            Ok(StageReport {
                stage,
                score: 0.8,
                report: no_regression_report(),
                top_two: true,
                trigger_task_passed: true,
                safety_regression: false,
            })
        }
    }

    struct Stage1FailRunner;
    impl BenchmarkRunner for Stage1FailRunner {
        fn run_stage(
            &self,
            stage: StagedEvaluation,
            _proposal: &SelfModProposal,
            _mode: PipelineMode,
        ) -> Result<StageReport, StagedEvalError> {
            Ok(StageReport {
                stage,
                score: 0.5,
                report: TestOutcomeReport {
                    fail_to_pass: vec![],
                    pass_to_pass: vec![],
                    fail_to_fail: vec![],
                    pass_to_fail: vec!["x".into()],
                },
                top_two: true,
                trigger_task_passed: false,
                safety_regression: false,
            })
        }
    }

    #[test]
    fn full_pipeline_stage1_fail_short_circuits() {
        let proposal = class_b_proposal();
        let v = run_full_pipeline(&Stage1FailRunner, &proposal, PipelineMode::Surgical).unwrap();
        assert_eq!(v.outcome, StageOutcome::DiscardStage1);
    }

    #[test]
    fn full_pipeline_promotes_when_all_gates_pass() {
        let proposal = class_b_proposal();
        let v = run_full_pipeline(&AlwaysPassRunner, &proposal, PipelineMode::Surgical).unwrap();
        assert_eq!(v.outcome, StageOutcome::PromoteActive);
    }

    #[test]
    fn class_a_proposal_rejected_at_entry() {
        use crate::self_mod_layer1a::DataTable;
        let mut p = class_b_proposal();
        p.target_config_key = AllowedConfigKey::DataTable(DataTable::DeprecatedPatterns);
        p.proposed_change = ProposedChange::InsertRow {
            table: "deprecated_patterns".into(),
            row: vec![],
        };
        let err = run_full_pipeline(&AlwaysPassRunner, &p, PipelineMode::Surgical).unwrap_err();
        assert!(matches!(err, StagedEvalError::ClassAProposalRejected));
    }

    /// C-21 Task 7: LOCKED_KEYS rejection is now enforced at COMPILE time
    /// by the closed-enum `ParameterKey`. Lock-listed keys like
    /// `cloud_monthly_budget` have NO variant in `ParameterKey`, so the
    /// previous runtime construction `AllowedConfigKey::Parameter("cloud_monthly_budget")`
    /// no longer compiles. The compile-fail behaviour is exercised by the
    /// c21_anti_stale_contract.rs trybuild fixture in the contracts crate.
    ///
    /// As a runtime sanity check, we still verify the locked-key gate
    /// rejects when invoked through the public API surface (e.g. a future
    /// JSON-decoded proposal whose wire-form key string evades the type
    /// system would still trip `is_locked` and the gate at L226).
    #[test]
    fn locked_key_check_is_runtime_consistent() {
        assert!(crate::self_mod_safety::is_locked("cloud_monthly_budget"));
        assert!(crate::self_mod_safety::is_locked("score_weights"));
        assert!(!crate::self_mod_safety::is_locked("etc_threshold"));
    }

    #[test]
    fn mode_mismatch_rejected_at_entry() {
        let mut p = class_b_proposal();
        p.scope = Some(ProjectScope {
            ecosystem: None,
            language: None,
            pipeline_mode: Some(PipelineMode::Greenfield),
        });
        let err = run_full_pipeline(&AlwaysPassRunner, &p, PipelineMode::Surgical).unwrap_err();
        assert!(matches!(err, StagedEvalError::ModeMismatch { .. }));
    }
}
