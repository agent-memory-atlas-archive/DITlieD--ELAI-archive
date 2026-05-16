//! B-41 Slice 3i — AutonomousGate ⨯ Per-Level evidence contract.
//!
//! Asserts that the 5 new B-41 per-level audit zero-blocks evidence checks
//! integrate into the existing Evidence struct WITHOUT breaking the B-40
//! framing invariant or the original 14 autonomous_gate_contract.rs cases.
//!
//! Coverage:
//!   1. Cold-start: a fresh SsbReader produces all-true evidence (vacuous
//!      default) — preserves the legacy "no rows = no block" semantics.
//!   2. derive_*_zero_blocks: each helper returns true when the SSB stream
//!      has no PerLevelAuditVerdict rows OR the rows have cascade_to_parent=
//!      false; returns false IFF a row exists with cascade_to_parent=true
//!      AND matching mode.
//!   3. AutonomousEvidence::all_passed() ANDs the 5 new fields — flipping
//!      any to false fails the gate.
//!   4. AutonomousEvidence::failed_checks() lists the failed B-41 fields.

use elai_orchestrator::ssb::{
    PerLevelAuditVerdictPayload, Ssb, SsbActionType, SsbConfig, SsbPayload, SsbReader,
};
use elai_orchestrator::{
    derive_babystep_zero_blocks, derive_concept_zero_blocks, derive_phase_zero_blocks,
    derive_plan_zero_blocks, derive_step_zero_blocks, AutonomousEvidence,
};

use elai_core::clock::SeededClock;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tempfile::TempDir;

fn make_ssb() -> (Arc<Ssb>, TempDir) {
    let dir = TempDir::new().expect("tempdir");
    let cfg = SsbConfig {
        db_path: dir.path().join("agpl.db"),
        pool_size: Some(2),
        project_id: "p".into(),
        session_id: "s".into(),
    };
    let clock = Arc::new(SeededClock::with_epoch(
        SystemTime::UNIX_EPOCH + Duration::from_millis(1_700_000_000_000),
    )) as Arc<dyn elai_core::ClockProvider>;
    let ssb = Ssb::new(cfg, clock).expect("ssb init");
    (Arc::new(ssb), dir)
}

#[test]
fn contract_default_evidence_per_level_fields_are_true() {
    // Vacuous-default true preserves the legacy "no rows = no block"
    // semantics. The 14 autonomous_gate_contract.rs tests rely on
    // Evidence::default() not silently flipping their assertions.
    let e = AutonomousEvidence::default();
    assert!(e.concept_zero_blocks);
    assert!(e.phase_zero_blocks);
    assert!(e.step_zero_blocks);
    assert!(e.babystep_zero_blocks);
    assert!(e.plan_zero_blocks);
}

#[test]
fn contract_all_passed_requires_each_per_level_field() {
    // Build an all-true evidence then flip each per-level field in turn —
    // every flip must make all_passed() false.
    let mut e = AutonomousEvidence {
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
    };
    assert!(e.all_passed(), "baseline all-true must pass");
    e.concept_zero_blocks = false;
    assert!(!e.all_passed(), "flipping concept_zero_blocks must fail");
    e.concept_zero_blocks = true;
    e.babystep_zero_blocks = false;
    assert!(!e.all_passed(), "flipping babystep_zero_blocks must fail");
}

#[test]
fn contract_failed_checks_lists_per_level_when_false() {
    let mut e = AutonomousEvidence::default();
    // default is all-false on the base 6 fields, true on framing + 5 B-41.
    e.knowledge_readiness = true;
    e.zero_ungrounded = true;
    e.tests_fail_correctly = true;
    e.test_flipped = true;
    e.zero_blocks = true;
    e.dkg_and_changelog_present = true;
    e.concept_zero_blocks = false;
    e.phase_zero_blocks = false;
    let failed = e.failed_checks();
    assert!(failed.contains(&"concept_zero_blocks"));
    assert!(failed.contains(&"phase_zero_blocks"));
    assert!(!failed.contains(&"step_zero_blocks"));
}

async fn push_verdict_row(
    ssb: &Arc<Ssb>,
    mode: &str,
    cascade: bool,
) {
    let payload = SsbPayload::PerLevelAuditVerdict(PerLevelAuditVerdictPayload {
        mode: mode.into(),
        tier: "step".into(),
        node_id: 1,
        verdict: if cascade { "block".into() } else { "allow".into() },
        findings_count: if cascade { 1 } else { 0 },
        respawned_once: cascade,
        cascade_to_parent: cascade,
    });
    ssb.write_payload("auditor", 0, payload, Vec::new())
        .await
        .expect("write_payload ok");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn derive_zero_blocks_helpers_cold_start_all_true() {
    let (ssb, _dir) = make_ssb();
    let reader: &dyn SsbReader = ssb.as_ref();
    assert!(derive_concept_zero_blocks(reader));
    assert!(derive_phase_zero_blocks(reader));
    assert!(derive_step_zero_blocks(reader));
    assert!(derive_babystep_zero_blocks(reader));
    assert!(derive_plan_zero_blocks(reader));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn derive_zero_blocks_returns_false_only_on_cascade_row() {
    let (ssb, _dir) = make_ssb();
    push_verdict_row(&ssb, "phases", false).await; // non-cascade allow row
    let reader: &dyn SsbReader = ssb.as_ref();
    assert!(
        derive_phase_zero_blocks(reader),
        "non-cascade allow row must not block"
    );
    push_verdict_row(&ssb, "phases", true).await; // cascade row
    assert!(
        !derive_phase_zero_blocks(reader),
        "cascade=true row must flip to false"
    );
    // Other modes unaffected.
    assert!(derive_concept_zero_blocks(reader));
    assert!(derive_step_zero_blocks(reader));
    assert!(derive_babystep_zero_blocks(reader));
    assert!(derive_plan_zero_blocks(reader));
}

/// B-41 audit WARN-1: a resolved BLOCK (BLOCK followed by ALLOW for the
/// same node_id) MUST recover the babystep_zero_blocks signal even though
/// the prior cascade=true row remains in the SSB stream. Asserts the
/// derive helper groups by (tier, node_id) and reads the LATEST verdict
/// per node — not the OR over all historical cascade flags.
async fn push_verdict_row_for_node(
    ssb: &Arc<Ssb>,
    mode: &str,
    tier: &str,
    node_id: u64,
    verdict: &str,
    cascade: bool,
) {
    let payload = SsbPayload::PerLevelAuditVerdict(PerLevelAuditVerdictPayload {
        mode: mode.into(),
        tier: tier.into(),
        node_id,
        verdict: verdict.into(),
        findings_count: if verdict == "block" { 1 } else { 0 },
        respawned_once: cascade || verdict == "allow",
        cascade_to_parent: cascade,
    });
    ssb.write_payload("auditor", 0, payload, Vec::new())
        .await
        .expect("write_payload ok");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn derive_babystep_zero_blocks_recovers_after_allow_supersedes_cascade() {
    let (ssb, _dir) = make_ssb();
    // Step 1: emit BLOCK+cascade=true for babystep node 7.
    push_verdict_row_for_node(&ssb, "babysteps", "baby_step", 7, "block", true).await;
    let reader: &dyn SsbReader = ssb.as_ref();
    assert!(
        !derive_babystep_zero_blocks(reader),
        "cascade=true row must initially block the babystep helper"
    );

    // Step 2: emit ALLOW for the SAME node (cascade=true persisted from
    // history — does NOT matter because the LATEST verdict is ALLOW).
    push_verdict_row_for_node(&ssb, "babysteps", "baby_step", 7, "allow", true).await;
    assert!(
        derive_babystep_zero_blocks(reader),
        "latest ALLOW verdict for the same node must recover zero-blocks regardless of \
         historical cascade=true rows (B-41 audit WARN-1)"
    );

    // Other modes still unaffected.
    assert!(derive_concept_zero_blocks(reader));
    assert!(derive_phase_zero_blocks(reader));
    assert!(derive_step_zero_blocks(reader));
    assert!(derive_plan_zero_blocks(reader));

    // Sanity: a DIFFERENT node still cascading keeps the helper false.
    push_verdict_row_for_node(&ssb, "babysteps", "baby_step", 99, "block", true).await;
    assert!(
        !derive_babystep_zero_blocks(reader),
        "a still-cascaded sibling node must keep the helper false"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn babystep_helper_folds_babystep_detail_cascades() {
    // DECISION-CASCADE: BabyStepDetail cascade should be visible on the
    // BabyStep zero-blocks helper.
    let (ssb, _dir) = make_ssb();
    push_verdict_row(&ssb, "babystep_detail", true).await;
    let reader: &dyn SsbReader = ssb.as_ref();
    assert!(
        !derive_babystep_zero_blocks(reader),
        "babystep helper must observe babystep_detail cascade"
    );
}
