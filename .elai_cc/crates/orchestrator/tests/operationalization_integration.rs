//! D-12 — OperationalizationCheck integration smoke.
//!
//! End-to-end exercise of the baby-step predicate dispatch pipeline:
//!
//!   1. Parse a markdown MEASURABLE_PROCEDURE fixture with
//!      [`elai_orchestrator::parse_baby_step`] + manually attach a
//!      structured Claim payload (the markdown surface in D-12 carries
//!      the procedure + expected; the Claim is attached by the
//!      auditor at dispatch time from the agent's emit-record).
//!   2. Call [`elai_orchestrator::is_operationalized`] — must be
//!      `true` once the Claim is attached.
//!   3. Build a fake [`elai_orchestrator::CheckerRegistry`] with a
//!      [`elai_orchestrator::NumericRangeChecker`] that returns Pass.
//!   4. Call [`elai_orchestrator::run_operational_check`] — must emit
//!      one `baby_step_dispatched` row through
//!      [`elai_orchestrator::VecBabyStepDispatchedEmitter`] AND
//!      return `Ok(DispatchOutcome::Dispatched { .. })` with
//!      `overall_pass() == true`.
//!
//! This is the WIRING_CONTRACT.observable_effect for D-12.

use elai_orchestrator::{
    dispatch_check_for_baby_step, is_operationalized, parse_baby_step, run_operational_check,
    BabyStep, BabyStepDispatchOutcome, CheckerRegistry, NumericRangeChecker,
    VecBabyStepDispatchedEmitter,
};

const FIXTURE: &str = r#"
TITLE: bs-d12-smoke-1
PARENT: task-d12-smoke
DESCRIPTION: range checker must return Pass for the claim text '7'
MEASURABLE_PROCEDURE:
  steps:
    - "feed claim text to numeric range checker"
    - "compare result to expected boolean true"
  success_criterion: "checker returns Pass for the supplied value"
  checker_kind: numeric_range
  max_runtime_ms: 5000
EXPECTED_RESULT:
  kind: boolean_flag
  value: true
  tolerance: null
CLAIM: none
"#;

fn attach_claim(step: BabyStep, claim_text: &str) -> BabyStep {
    BabyStep {
        claim_payload: Some(elai_orchestrator::Claim {
            claim_id: format!("claim-{}", claim_text),
            text: claim_text.to_string(),
            claim_type: elai_orchestrator::ClaimType::Computational,
            evidence: vec![],
            ts_ms: 1_700_000_100_000,
        }),
        ..step
    }
}

#[tokio::test]
async fn end_to_end_pass_emits_one_ssb_row() {
    let parsed = parse_baby_step(FIXTURE).expect("fixture parses");
    // Pre-attach: predicate must be FALSE because CLAIM was "none".
    assert!(!is_operationalized(&parsed));

    // Auditor attaches the structured Claim payload.
    let step = attach_claim(parsed, "7");
    assert!(is_operationalized(&step));

    let mut reg = CheckerRegistry::new();
    reg.register("nr-0-10", Box::new(NumericRangeChecker::new(0.0, 10.0)));

    let emitter = VecBabyStepDispatchedEmitter::new();
    let outcome = run_operational_check(&step, &reg, &emitter, 1_700_000_101_000)
        .await
        .expect("dispatch must Ok");
    assert!(outcome.overall_pass());
    assert!(matches!(outcome, BabyStepDispatchOutcome::Dispatched { .. }));

    let rows = emitter.rows().await;
    assert_eq!(rows.len(), 1, "exactly one ssb row per dispatch");
    let row = &rows[0];
    assert_eq!(row.step_id, "bs-d12-smoke-1");
    assert_eq!(row.parent_task_id, "task-d12-smoke");
    assert_eq!(row.outcome, "pass");
    assert_eq!(row.checker_kinds_used, vec!["numeric_range".to_string()]);
    assert_eq!(row.ts_ms, 1_700_000_101_000);
}

#[tokio::test]
async fn missing_claim_returns_not_operationalized_and_emits_inapplicable_row() {
    let parsed = parse_baby_step(FIXTURE).expect("fixture parses");
    // No attach_claim call — operational predicate must be false.
    assert!(!is_operationalized(&parsed));

    let reg = CheckerRegistry::new();
    let emitter = VecBabyStepDispatchedEmitter::new();
    let err = run_operational_check(&parsed, &reg, &emitter, 1_700_000_102_000)
        .await
        .expect_err("must error on missing claim");
    match err {
        elai_orchestrator::OperationalError::NotOperationalized { step_id } => {
            assert_eq!(step_id, "bs-d12-smoke-1");
        }
        other => panic!("expected NotOperationalized, got {other:?}"),
    }
    let rows = emitter.rows().await;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].outcome, "inapplicable");
    assert!(rows[0].checker_kinds_used.is_empty());
}

#[tokio::test]
async fn fail_outcome_propagates_to_ssb_row() {
    let parsed = parse_baby_step(FIXTURE).expect("fixture parses");
    // Attach a claim whose text is out of [0, 10] -> NumericRange returns Fail.
    let step = attach_claim(parsed, "42");

    let mut reg = CheckerRegistry::new();
    reg.register("nr-0-10", Box::new(NumericRangeChecker::new(0.0, 10.0)));

    let emitter = VecBabyStepDispatchedEmitter::new();
    let err = run_operational_check(&step, &reg, &emitter, 1_700_000_103_000)
        .await
        .expect_err("Fail dispatch must error");
    match err {
        elai_orchestrator::OperationalError::NonPassOutcome { outcome, .. } => {
            assert_eq!(outcome, "fail");
        }
        other => panic!("expected NonPassOutcome, got {other:?}"),
    }
    let rows = emitter.rows().await;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].outcome, "fail");
    assert_eq!(rows[0].checker_kinds_used, vec!["numeric_range".to_string()]);
}

#[tokio::test]
async fn dispatch_check_for_baby_step_is_pure_no_emit() {
    // The pure dispatch function (no SSB emit) is the building block;
    // verify it matches the run_operational_check wrapper's outcome.
    let parsed = parse_baby_step(FIXTURE).expect("parses");
    let step = attach_claim(parsed, "3");
    let mut reg = CheckerRegistry::new();
    reg.register("nr-0-10", Box::new(NumericRangeChecker::new(0.0, 10.0)));
    let outcome = dispatch_check_for_baby_step(&step, &reg).await;
    assert!(outcome.overall_pass());
    assert_eq!(outcome.outcome_label(), "pass");
    assert_eq!(outcome.checker_kinds_used(), vec!["numeric_range".to_string()]);
}
