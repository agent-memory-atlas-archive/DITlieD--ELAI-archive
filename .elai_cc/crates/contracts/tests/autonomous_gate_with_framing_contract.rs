//! B-40 — AutonomousGate + FRAMING substate contract.
//!
//! Extends the existing 14 autonomous_gate_contract.rs tests with 4+ cases
//! for the 7th `framing_converged` evidence check. The existing 14 tests
//! are untouched (only the helper struct literal got a `framing_converged: true`
//! field added — semantic behaviour preserved by the vacuous-default).
//!
//! Cases:
//!   1. Default Evidence has framing_converged == true (vacuous-default).
//!   2. Default Evidence's all_passed() is false for non-framing fields,
//!      proving framing_converged didn't auto-pass without other checks.
//!   3. all-pass Evidence with framing_converged=false produces all_passed()=false.
//!   4. summary() includes the new `fc=` suffix.
//!   5. failed_checks() includes "framing_converged" when the field is false.

use elai_orchestrator::AutonomousEvidence;

fn all_pass_legacy() -> AutonomousEvidence {
    AutonomousEvidence {
        knowledge_readiness: true,
        zero_ungrounded: true,
        tests_fail_correctly: true,
        test_flipped: true,
        zero_blocks: true,
        dkg_and_changelog_present: true,
        framing_converged: true,
        // B-41: 5 per-level audit zero-blocks vacuous-default true.
        concept_zero_blocks: true,
        phase_zero_blocks: true,
        step_zero_blocks: true,
        babystep_zero_blocks: true,
        plan_zero_blocks: true,
        // F-10: AUTO-ADVANCE security-monotonicity vacuous-default true.
        security_monotonicity: true,
        // SP5 wiring-fix: off-agent EXECUTING -> AUDITING legs default FALSE;
        // an all-true fixture sets them explicitly (ADR-8).
        layer3_entrypoint_covered: true,
        layer4_trace_present: true,
    }
}

#[test]
fn contract_default_evidence_has_framing_vacuous_default_true() {
    let e = AutonomousEvidence::default();
    assert!(
        e.framing_converged,
        "CONTRACT: Evidence::default().framing_converged must be true (vacuous-default for non-FRAMING contexts)"
    );
}

#[test]
fn contract_all_passed_requires_framing_converged() {
    let mut e = all_pass_legacy();
    assert!(e.all_passed());
    e.framing_converged = false;
    assert!(
        !e.all_passed(),
        "CONTRACT: all_passed() must return false when framing_converged is false"
    );
}

#[test]
fn contract_all_passed_is_false_with_default_evidence() {
    // Default evidence has knowledge_readiness=false, so all_passed must
    // be false even though framing_converged defaults to true. This proves
    // the 7th check is ADDITIVE, not replacing the other 6.
    let e = AutonomousEvidence::default();
    assert!(!e.all_passed());
}

#[test]
fn contract_summary_includes_fc_suffix() {
    let e = AutonomousEvidence::default();
    let s = e.summary();
    assert!(
        s.ends_with("fc=1") || s.ends_with("fc=0"),
        "CONTRACT: summary() must end with fc=<0|1>; got: {}",
        s
    );
}

#[test]
fn contract_failed_checks_includes_framing_converged_when_false() {
    let mut e = all_pass_legacy();
    e.framing_converged = false;
    let failed = e.failed_checks();
    assert!(
        failed.contains(&"framing_converged"),
        "CONTRACT: failed_checks() must include 'framing_converged' when false; got: {:?}",
        failed
    );
}

#[test]
fn contract_failed_checks_excludes_framing_converged_when_true() {
    let e = AutonomousEvidence::default(); // framing_converged = true
    let failed = e.failed_checks();
    assert!(
        !failed.contains(&"framing_converged"),
        "CONTRACT: failed_checks() must NOT include 'framing_converged' when true; got: {:?}",
        failed
    );
}

#[test]
fn contract_all_pass_legacy_evidence_still_passes() {
    // The 14 existing autonomous_gate_contract.rs tests rely on
    // `all_pass_evidence()` triggering AutoAdvance. With the new 7th check
    // vacuous-defaulted to true, the all-pass condition still holds.
    let e = all_pass_legacy();
    assert!(
        e.all_passed(),
        "CONTRACT: legacy all-pass evidence must still produce all_passed=true after B-40 7th check addition"
    );
}

// ---------------------------------------------------------------------------
// B-40 BLOCK-2 audit fix — derive_framing_evidence wiring contract.
//
// Cases:
//   (a) cold start (no IdeaFramingStarted rows) -> true (AutoAdvance allowed)
//   (b) IdeaFramingStarted + 0 unresolved BLOCK findings -> true
//   (c) IdeaFramingStarted + unresolved BLOCK Finding -> false (AutoAdvance
//       prevented)
// ---------------------------------------------------------------------------

mod derive_framing_evidence_cases {
    use elai_orchestrator::ssb::{
        FindingPayload, IdeaFramingStartedPayload, SsbActionType, SsbEntry, SsbError, SsbPayload,
        SsbReader,
    };
    use elai_orchestrator::task_runner::derive_framing_evidence;
    use std::sync::Arc;

    /// Minimal in-memory SsbReader returning canned `query_by_type` results.
    struct FakeReader {
        rows: Vec<SsbEntry>,
    }

    impl FakeReader {
        fn new() -> Self {
            Self { rows: Vec::new() }
        }
        fn push(mut self, seq: i64, action_type: SsbActionType, payload: SsbPayload) -> Self {
            let bytes = rmp_serde::to_vec(&payload).expect("encode payload");
            self.rows.push(SsbEntry {
                seq,
                project_id: "p".to_owned(),
                session_id: "s".to_owned(),
                agent_id: "a".to_owned(),
                superstep: 0,
                action_type,
                payload: Arc::from(bytes.as_slice()),
                created_at: 0,
            });
            self
        }
    }

    impl SsbReader for FakeReader {
        fn query_by_session(&self, _: &str) -> Result<Vec<SsbEntry>, SsbError> {
            Ok(vec![])
        }
        fn query_by_file(&self, _: &[&str]) -> Result<Vec<SsbEntry>, SsbError> {
            Ok(vec![])
        }
        fn query_by_type(
            &self,
            action_type: SsbActionType,
        ) -> Result<Vec<SsbEntry>, SsbError> {
            Ok(self
                .rows
                .iter()
                .filter(|r| r.action_type == action_type)
                .cloned()
                .collect())
        }
        fn query_by_superstep(&self, _: i64) -> Result<Vec<SsbEntry>, SsbError> {
            Ok(vec![])
        }
        fn query_conflict(
            &self,
            _: &[&str],
            _: &str,
            _: i64,
        ) -> Result<Vec<String>, SsbError> {
            Ok(vec![])
        }
        fn last_seq(&self) -> Result<Option<i64>, SsbError> {
            Ok(self.rows.last().map(|r| r.seq))
        }
        fn query_by_seq(&self, seq: i64) -> Result<Option<SsbEntry>, SsbError> {
            Ok(self.rows.iter().find(|r| r.seq == seq).cloned())
        }
        fn query_reasoning_trace(&self, _: &str) -> Result<Vec<SsbEntry>, SsbError> {
            Ok(vec![])
        }
        fn query_phase_complete(&self, _: &str) -> Result<Vec<SsbEntry>, SsbError> {
            Ok(vec![])
        }
        fn query_isolation_metrics(&self) -> Result<Vec<SsbEntry>, SsbError> {
            Ok(vec![])
        }
    }

    fn started_payload() -> SsbPayload {
        SsbPayload::IdeaFramingStarted(IdeaFramingStartedPayload {
            session_id: "s".to_owned(),
            prompt: "test".to_owned(),
            started_at_ms: 1,
        })
    }

    fn block_finding() -> SsbPayload {
        SsbPayload::Finding(FindingPayload {
            severity: "block".to_owned(),
            category: "framing".to_owned(),
            message: "unresolved BLOCK".to_owned(),
            file_path: None,
            line: None,
            failure_category: None,
            audit_step: None,
            few_shot_examples: None,
        })
    }

    #[test]
    fn case_a_cold_start_no_framing_rows_returns_true() {
        let reader = FakeReader::new();
        assert!(
            derive_framing_evidence(&reader),
            "CONTRACT (a): cold start (no IdeaFramingStarted rows) must yield framing_converged=true so non-FRAMING phases AutoAdvance"
        );
    }

    #[test]
    fn case_b_framing_started_no_block_findings_returns_true() {
        let reader = FakeReader::new().push(10, SsbActionType::IdeaFramingStarted, started_payload());
        assert!(
            derive_framing_evidence(&reader),
            "CONTRACT (b): IdeaFramingStarted with 0 unresolved BLOCK findings must yield framing_converged=true"
        );
    }

    #[test]
    fn case_c_framing_started_with_unresolved_block_returns_false() {
        let reader = FakeReader::new()
            .push(10, SsbActionType::IdeaFramingStarted, started_payload())
            .push(11, SsbActionType::Finding, block_finding());
        assert!(
            !derive_framing_evidence(&reader),
            "CONTRACT (c): IdeaFramingStarted + post-start BLOCK Finding must yield framing_converged=false to prevent AutoAdvance"
        );
    }

    #[test]
    fn pre_framing_block_finding_does_not_count() {
        // A BLOCK Finding emitted BEFORE IdeaFramingStarted is not an
        // unresolved-framing block; framing_converged should still be true.
        let reader = FakeReader::new()
            .push(5, SsbActionType::Finding, block_finding())
            .push(10, SsbActionType::IdeaFramingStarted, started_payload());
        assert!(
            derive_framing_evidence(&reader),
            "CONTRACT: BLOCK Finding before IdeaFramingStarted must not count toward framing convergence"
        );
    }
}
