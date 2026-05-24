//! D-12 — OperationalizationCheck closure contract.
//!
//! Additive contract tests that pin the shipped D-12 substrate surface
//! against accidental drift. Grounded in workflow/papers/voyager-2305.16291-interface.md
//! INV-5 (state delta required for skill retention) — translated to ELAI as
//! "a claim is retained iff the D-12 predicate operationalises it AND the
//! dispatch produces a verifiable outcome."
//!
//! What this test pins (every assertion has a one-line rationale):
//!
//! 1. **SCHEMA floor (>=81)**. D-12 substrate landed at SCHEMA_VERSION 81
//!    (per ssb.rs:747 historic comment). Closure does NOT bump SCHEMA;
//!    head must remain at or above 81 forever — regression here means
//!    the SSB row family was removed.
//!
//! 2. **`DispatchOutcome` enum exhaustive serde 3-arm shape**. Locked
//!    closed enum `{NotOperationalized, Inapplicable, Dispatched{results}}`
//!    at .elai_cc/crates/orchestrator/src/baby_step.rs:199. Adding a
//!    4th arm would shatter downstream exhaustive matches; round-trip
//!    test pins the wire-shape against accidental rename.
//!
//! 3. **`BabyStepDispatchedPayload` round-trip**. Single SSB variant per
//!    DECISION-2 (consolidated over plan's 2-variant split); payload
//!    fields `{step_id, parent_task_id, checker_kinds_used, outcome,
//!    ts_ms}` round-trip through serde JSON.
//!
//! 4. **`is_operationalized` predicate conservative semantics**. Voyager
//!    INV-5 mapping: every field MUST be present for the predicate to
//!    return true. Three negative-cases pin the conservative rule:
//!    missing procedure, missing expected, missing claim all return
//!    false.
//!
//! 5. **Voyager INV-5 invariant**. A step with no operationalisation
//!    fields produces `DispatchOutcome::NotOperationalized` — never
//!    `Dispatched`. State delta required for retention.
//!
//! See workflow/reasoning/D-12-decisions.md for the consolidation
//! rationale (DECISION-1 .. DECISION-7).

use elai_orchestrator::{
    dispatch_check_for_baby_step, is_operationalized, BabyStep, BabyStepDispatchOutcome,
    CheckerRegistry, ExpectedKind, ExpectedResult, MeasurableProcedure,
};
use elai_orchestrator::checker_registry::CheckerKind;
use elai_orchestrator::claim::{Claim, ClaimType};
use elai_orchestrator::ssb::{
    BabyStepDispatchedPayload, SsbActionType, SCHEMA_VERSION_PUBLIC as SCHEMA_VERSION,
};

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/// D-12 substrate landed at SCHEMA_VERSION 81 (per ssb.rs:747 historic
/// bump comment). Closure does NOT touch SCHEMA. Floor pinned here so
/// any future regression that removes the SSB row family lights up.
const D12_SCHEMA_FLOOR: i64 = 81;

fn fully_operationalised_step() -> BabyStep {
    BabyStep {
        step_id: "bs-d12-contract".into(),
        parent_task_id: "task-d12-contract".into(),
        description: "contract fixture: all 3 fields present".into(),
        measurable_procedure: Some(MeasurableProcedure {
            steps: vec!["assert checker returns Pass".into()],
            success_criterion: "numeric range checker reports value in [0, 10]".into(),
            checker_kind_hint: Some(CheckerKind::NumericRange),
            max_runtime_ms: 5_000,
        }),
        expected_result: Some(ExpectedResult {
            kind: ExpectedKind::BooleanFlag,
            value: serde_json::json!(true),
            tolerance: None,
        }),
        claim_payload: Some(Claim {
            claim_id: "claim-d12-contract".into(),
            text: "5".into(),
            claim_type: ClaimType::Computational,
            evidence: vec![],
            ts_ms: 1_700_000_000_000,
        }),
    }
}

fn empty_step() -> BabyStep {
    BabyStep {
        step_id: "bs-d12-empty".into(),
        parent_task_id: "task-d12-empty".into(),
        description: "contract fixture: zero operationalisation fields".into(),
        measurable_procedure: None,
        expected_result: None,
        claim_payload: None,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn schema_version_at_or_above_d12_floor() {
    // D-12 substrate row landed at SCHEMA_VERSION 81 (ssb.rs:747). Any
    // regression below 81 means the row family was removed or the
    // substrate was rolled back. Failure here MUST be investigated
    // before continuing — DECISION-1 in D-12-decisions.md.
    assert!(
        SCHEMA_VERSION >= D12_SCHEMA_FLOOR,
        "SCHEMA_VERSION ({SCHEMA_VERSION}) regressed below D-12 floor ({D12_SCHEMA_FLOOR}); D-12 substrate row family was likely removed",
    );
}

#[test]
fn dispatch_outcome_three_arm_serde_roundtrip() {
    // Pin the 3-arm shape against accidental rename or 4th-arm
    // addition. Closed enum locked at baby_step.rs:199; downstream
    // pattern-matches rely on exhaustive coverage.
    let variants = [
        BabyStepDispatchOutcome::NotOperationalized,
        BabyStepDispatchOutcome::Inapplicable,
        BabyStepDispatchOutcome::Dispatched { results: vec![] },
    ];
    for v in variants {
        let bytes = serde_json::to_vec(&v).expect("serialise");
        let back: BabyStepDispatchOutcome =
            serde_json::from_slice(&bytes).expect("deserialise");
        assert_eq!(v, back, "round-trip must preserve variant identity");
    }
}

#[test]
fn dispatched_outcome_overall_pass_helper_is_conservative() {
    // `overall_pass` returns false for NotOperationalized + Inapplicable
    // + empty-results Dispatched. Pinned because the helper is the
    // auditor's gate signal (DECISION-5 mapping of Voyager INV-5
    // "state delta required").
    assert!(!BabyStepDispatchOutcome::NotOperationalized.overall_pass());
    assert!(!BabyStepDispatchOutcome::Inapplicable.overall_pass());
    assert!(!BabyStepDispatchOutcome::Dispatched { results: vec![] }.overall_pass());
}

#[test]
fn dispatched_outcome_label_locked_three_strings() {
    // The SSB row's `outcome` discriminator string is the consolidation
    // signal (DECISION-2: single BabyStepDispatched variant + outcome
    // label replaces the plan's 2-variant split). Locked to exactly
    // these three tokens — adding a 4th breaks the dashboard
    // commands_d12 BTreeMap pre-seeding.
    assert_eq!(BabyStepDispatchOutcome::NotOperationalized.outcome_label(), "inapplicable");
    assert_eq!(BabyStepDispatchOutcome::Inapplicable.outcome_label(), "inapplicable");
    assert_eq!(
        BabyStepDispatchOutcome::Dispatched { results: vec![] }.outcome_label(),
        "inapplicable",
    );
}

#[test]
fn baby_step_dispatched_payload_roundtrip_via_json() {
    // Single consolidated SSB variant per DECISION-2. Payload field
    // set locked at ssb.rs:12083; round-trip pins the wire shape.
    let payload = BabyStepDispatchedPayload {
        step_id: "bs-d12-roundtrip".into(),
        parent_task_id: "task-d12-roundtrip".into(),
        checker_kinds_used: vec!["numeric_range".into(), "regex_match".into()],
        outcome: "pass".into(),
        ts_ms: 1_700_000_500_000,
    };
    let bytes = serde_json::to_vec(&payload).expect("serialise payload");
    let back: BabyStepDispatchedPayload =
        serde_json::from_slice(&bytes).expect("deserialise payload");
    assert_eq!(payload.step_id, back.step_id);
    assert_eq!(payload.parent_task_id, back.parent_task_id);
    assert_eq!(payload.checker_kinds_used, back.checker_kinds_used);
    assert_eq!(payload.outcome, back.outcome);
    assert_eq!(payload.ts_ms, back.ts_ms);
}

#[test]
fn baby_step_dispatched_action_type_wire_tag_is_snake_case() {
    // Wire tag locked at ssb.rs:4115 (as_sql_str arm). Snake-case
    // matches the wider SsbActionType naming convention; a rename
    // would break action_type_lock.golden + dashboard polling.
    assert_eq!(SsbActionType::BabyStepDispatched.as_sql_str(), "baby_step_dispatched");
}

#[test]
fn is_operationalized_predicate_conservative_all_three_fields_required() {
    // Voyager INV-5 mapping (DECISION-5). The predicate is the SOLE
    // gate the auditor uses; absence of any field means false. Three
    // negative cases pin the conservative rule; one positive case
    // pins the happy path.
    let full = fully_operationalised_step();
    assert!(is_operationalized(&full), "full step must be operationalised");

    let no_procedure = BabyStep {
        measurable_procedure: None,
        ..fully_operationalised_step()
    };
    assert!(!is_operationalized(&no_procedure), "missing procedure -> false");

    let no_expected = BabyStep {
        expected_result: None,
        ..fully_operationalised_step()
    };
    assert!(!is_operationalized(&no_expected), "missing expected -> false");

    let no_claim = BabyStep {
        claim_payload: None,
        ..fully_operationalised_step()
    };
    assert!(!is_operationalized(&no_claim), "missing claim -> false");
}

#[tokio::test]
async fn voyager_inv5_no_state_change_returns_not_operationalized() {
    // INV-5 (Voyager arxiv:2305.16291 §3) translated: a baby step
    // with NO operationalisation fields can never produce a
    // measurable state delta; dispatch MUST short-circuit to
    // NotOperationalized rather than synthesising a Dispatched arm
    // with empty results. Pinned because a careless implementation
    // might iterate the registry and emit Dispatched{results: vec![]}
    // — which would falsely advertise "we tried to operationalise"
    // when in fact no skill-with-measurable-effect was ever
    // attempted.
    let step = empty_step();
    let reg = CheckerRegistry::new();
    let outcome = dispatch_check_for_baby_step(&step, &reg).await;
    assert!(
        matches!(outcome, BabyStepDispatchOutcome::NotOperationalized),
        "INV-5 violation: no operationalisation fields must return NotOperationalized, got {outcome:?}",
    );
    assert!(!outcome.overall_pass());
    assert_eq!(outcome.outcome_label(), "inapplicable");
    assert!(outcome.checker_kinds_used().is_empty());
}

#[tokio::test]
async fn voyager_inv5_partial_fields_still_block_dispatch() {
    // Stronger INV-5 check: even with TWO of three fields present,
    // the predicate must hold the line. Voyager retention requires
    // the FULL skill triple (effect + verification + claim); partial
    // structures cannot demonstrate measurable state change.
    let full = fully_operationalised_step();
    let no_procedure = BabyStep {
        measurable_procedure: None,
        ..full.clone()
    };
    let no_expected = BabyStep {
        expected_result: None,
        ..full.clone()
    };
    let no_claim = BabyStep {
        claim_payload: None,
        ..full
    };
    let reg = CheckerRegistry::new();
    for step in [no_procedure, no_expected, no_claim] {
        let outcome = dispatch_check_for_baby_step(&step, &reg).await;
        assert!(
            matches!(outcome, BabyStepDispatchOutcome::NotOperationalized),
            "partial-field step ({}) must return NotOperationalized, got {outcome:?}",
            step.step_id,
        );
    }
}
