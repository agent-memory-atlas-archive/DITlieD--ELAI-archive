//! E-38 (SCHEMA_VERSION 175) -- EU AI Act compliance pre-gate contract.
//!
//! Exercises [`elai_orchestrator::compliance::pre_gate_planning_to_testing_first`]
//! against 4 failing fixtures + 1 passing fixture + 1 D-1-absent
//! Skipped fixture. Verifies:
//!   - each of the 4 ComplianceAssertion arms fires on the correct
//!     missing precondition
//!   - the passing fixture yields a Pass verdict with the correct
//!     assertions_checked bitmask
//!   - the D-1-absent fixture passes but with TemplateHashChain bit
//!     CLEARED (plan Open Question 1)
//!   - block_payload + verified_payload helpers carry the right SSB
//!     fields
//!
//! See workflow/reasoning/E-38-decisions.md.

use elai_orchestrator::compliance::{
    AUDIT_TRAIL_MIN_SSB_ROWS, ComplianceAssertion, ComplianceVerdict, PlanContext,
    build_block_payload, build_verified_payload, pre_gate_planning_to_testing_first,
};
use elai_orchestrator::ssb::{
    COMPLIANCE_BIT_AUDIT_TRAIL_LIVE, COMPLIANCE_BIT_CLOCK_PROVIDER,
    COMPLIANCE_BIT_DETERMINISTIC_SEED, COMPLIANCE_BIT_TEMPLATE_HASH_CHAIN, RoadmapSignalKind,
};

fn passing_ctx() -> PlanContext {
    PlanContext {
        plan_session_id: "s-contract".into(),
        has_clock_provider: true,
        has_deterministic_seed: true,
        template_hash_chain_verified: true,
        template_archive_present: true,
        ssb_row_count_in_planning_phase: AUDIT_TRAIL_MIN_SSB_ROWS,
    }
}

#[test]
fn fixture_1_pass_all_assertions_intact() {
    let v = pre_gate_planning_to_testing_first(&passing_ctx());
    match v {
        ComplianceVerdict::Pass { assertions_checked } => {
            assert_eq!(
                assertions_checked,
                COMPLIANCE_BIT_CLOCK_PROVIDER
                    | COMPLIANCE_BIT_DETERMINISTIC_SEED
                    | COMPLIANCE_BIT_TEMPLATE_HASH_CHAIN
                    | COMPLIANCE_BIT_AUDIT_TRAIL_LIVE,
                "all 4 assertion bits set when all preconditions hold"
            );
            assert!(v.allows_transition());
        }
        other => panic!("fixture 1: expected Pass, got {other:?}"),
    }
}

#[test]
fn fixture_2_block_on_missing_clock_provider() {
    let mut ctx = passing_ctx();
    ctx.has_clock_provider = false;
    let v = pre_gate_planning_to_testing_first(&ctx);
    match v {
        ComplianceVerdict::Block { assertion, hint } => {
            assert_eq!(assertion, ComplianceAssertion::ClockProviderIntact);
            assert_eq!(assertion.as_str(), "clock_provider_intact");
            assert!(hint.contains("ClockProvider"));
            assert_eq!(assertion.bit(), COMPLIANCE_BIT_CLOCK_PROVIDER);
        }
        other => panic!("fixture 2: expected Block, got {other:?}"),
    }
}

#[test]
fn fixture_3_block_on_missing_deterministic_seed() {
    let mut ctx = passing_ctx();
    ctx.has_deterministic_seed = false;
    let v = pre_gate_planning_to_testing_first(&ctx);
    match v {
        ComplianceVerdict::Block { assertion, hint } => {
            assert_eq!(assertion, ComplianceAssertion::DeterministicSeed);
            assert_eq!(assertion.as_str(), "deterministic_seed");
            assert!(hint.contains("ELAI_DETERMINISTIC_SEED"));
        }
        other => panic!("fixture 3: expected Block, got {other:?}"),
    }
}

#[test]
fn fixture_4_block_on_template_hash_chain_failed() {
    let mut ctx = passing_ctx();
    ctx.template_hash_chain_verified = false;
    let v = pre_gate_planning_to_testing_first(&ctx);
    match v {
        ComplianceVerdict::Block { assertion, hint } => {
            assert_eq!(assertion, ComplianceAssertion::TemplateHashChain);
            assert_eq!(assertion.as_str(), "template_hash_chain");
            assert!(hint.contains("content_hash"));
        }
        other => panic!("fixture 4: expected Block, got {other:?}"),
    }
}

#[test]
fn fixture_5_block_on_audit_trail_below_threshold() {
    let mut ctx = passing_ctx();
    ctx.ssb_row_count_in_planning_phase = 0;
    let v = pre_gate_planning_to_testing_first(&ctx);
    match v {
        ComplianceVerdict::Block { assertion, hint } => {
            assert_eq!(assertion, ComplianceAssertion::AuditTrailLive);
            assert_eq!(assertion.as_str(), "audit_trail_live");
            assert!(hint.contains("AUDIT_TRAIL_MIN_SSB_ROWS"));
        }
        other => panic!("fixture 5: expected Block, got {other:?}"),
    }
}

#[test]
fn fixture_6_pass_with_template_hash_chain_skipped_when_d1_absent() {
    // Plan Open Question 1: when D-1 TemplateArchive isn't shipped,
    // TemplateHashChain assertion degrades to Skipped (transition
    // allowed) and the bit is cleared in `assertions_checked`.
    let ctx = PlanContext::for_passing_no_d1("s-skipped");
    let v = pre_gate_planning_to_testing_first(&ctx);
    match v {
        ComplianceVerdict::Pass { assertions_checked } => {
            assert_eq!(
                assertions_checked,
                COMPLIANCE_BIT_CLOCK_PROVIDER
                    | COMPLIANCE_BIT_DETERMINISTIC_SEED
                    | COMPLIANCE_BIT_AUDIT_TRAIL_LIVE,
                "TemplateHashChain bit must be CLEARED when D-1 is absent"
            );
            assert_eq!(
                assertions_checked & COMPLIANCE_BIT_TEMPLATE_HASH_CHAIN,
                0,
                "TemplateHashChain bit is not set in Skipped path"
            );
        }
        other => panic!("fixture 6: expected Pass, got {other:?}"),
    }
}

#[test]
fn block_payload_carries_assertion_and_hint() {
    let ctx = passing_ctx();
    let payload = build_block_payload(
        &ctx,
        ComplianceAssertion::AuditTrailLive,
        ComplianceAssertion::AuditTrailLive.remediation_hint(),
        1_700_000_175_001,
    );
    assert_eq!(payload.kind, RoadmapSignalKind::ComplianceBlocked);
    assert_eq!(payload.assertion_label, "audit_trail_live");
    assert!(payload.hint.contains("AUDIT_TRAIL_MIN_SSB_ROWS"));
    assert_eq!(payload.plan_session_id, "s-contract");
    assert_eq!(payload.assertions_checked, 0);
}

#[test]
fn verified_payload_carries_bitmask() {
    let ctx = passing_ctx();
    let mask = COMPLIANCE_BIT_CLOCK_PROVIDER | COMPLIANCE_BIT_AUDIT_TRAIL_LIVE;
    let payload = build_verified_payload(&ctx, mask, 1_700_000_175_002);
    assert_eq!(payload.kind, RoadmapSignalKind::ComplianceVerified);
    assert_eq!(payload.assertions_checked, mask);
    assert!(payload.assertion_label.is_empty());
    assert!(payload.hint.is_empty());
}

#[test]
fn audit_trail_min_ssb_rows_is_at_least_one() {
    // EU AI Act art. 12 (record-keeping): at minimum the FSM
    // PhaseTransition row from IDLE -> PLANNING must already exist by
    // the time the gate runs. Bumping the threshold above 1 is fine;
    // dropping it to 0 means the gate accepts an empty audit trail.
    assert!(
        AUDIT_TRAIL_MIN_SSB_ROWS >= 1,
        "AUDIT_TRAIL_MIN_SSB_ROWS must be >= 1 to enforce art. 12 record-keeping"
    );
}
