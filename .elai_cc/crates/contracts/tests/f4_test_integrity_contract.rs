//! F-4 Slice B4 -- TestIntegrityEvent + AntiFabricationGate contract tests.
//!
//! Pins:
//!   1. TestOutcomePayload: stdout_hash round-trips through msgpack as None
//!      (pre-B1 compat) and as Some (B1 captured run).
//!   2. TestIntegrityEvent{Verified}: SSB encode/decode round-trip + correct
//!      sql_str ("test_integrity_verified").
//!   3. TestIntegrityEvent{FabricationBlocked}: SSB encode/decode round-trip
//!      + correct sql_str ("anti_fabrication_block").
//!   4. AntiFabricationGate.check() returns BLOCK when integration_test_rows
//!      is None (SP5 fail-closed: absent evidence is a hard FAIL, R1/R2).
//!   5. AntiFabricationGate.check() returns ALLOW when all rows have
//!      stdout_hash=Some (all evidence present; gate silent).
//!   6. AntiFabricationGate.check() returns BLOCK when a row has
//!      passed=true && stdout_hash=None (fabrication signature).
//!
//! These tests fail-to-compile until F-4 Slice B1 lands
//! TestIntegrityEventKind, TestIntegrityEventPayload, and the SCHEMA_VERSION
//! bump 196->197; and B2 lands AntiFabricationGate + IntegrationTestRowSummary.
//!
//! Source: F-plan B4 wiring contract; handoff B1+B2.

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::evidence::{
    AntiFabricationGate, EvidenceGate, IntegrationTestRowSummary, TransitionContext,
};
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, Ssb, SsbActionType, SsbConfig, SsbPayload,
    TestIntegrityEventKind, TestIntegrityEventPayload, TestOutcomePayload,
};
use elai_orchestrator::SsbReader;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// 1. TestOutcomePayload: stdout_hash round-trip (None = pre-B1 compat)
// ---------------------------------------------------------------------------

#[test]
fn contract_test_outcome_payload_stdout_hash_none_roundtrips() {
    let payload = TestOutcomePayload {
        test_name: "crate::module::test_no_capture".into(),
        passed: true,
        duration_ms: 42,
        stdout_hash: None,
    };
    let encoded = rmp_serde::to_vec_named(&payload).expect("encode");
    let decoded: TestOutcomePayload = rmp_serde::from_slice(&encoded).expect("decode");
    assert_eq!(decoded.test_name, "crate::module::test_no_capture");
    assert!(decoded.passed);
    assert_eq!(decoded.duration_ms, 42);
    assert_eq!(
        decoded.stdout_hash, None,
        "pre-B1 row with no stdout_hash must round-trip as None"
    );
}

// ---------------------------------------------------------------------------
// 2. TestOutcomePayload: stdout_hash round-trip (Some = captured run)
// ---------------------------------------------------------------------------

#[test]
fn contract_test_outcome_payload_stdout_hash_some_roundtrips() {
    let payload = TestOutcomePayload {
        test_name: "integration::real_run".into(),
        passed: true,
        duration_ms: 123,
        stdout_hash: Some("abcdef0123456789".into()),
    };
    let encoded = rmp_serde::to_vec_named(&payload).expect("encode");
    let decoded: TestOutcomePayload = rmp_serde::from_slice(&encoded).expect("decode");
    assert_eq!(decoded.test_name, "integration::real_run");
    assert!(decoded.passed);
    assert_eq!(decoded.duration_ms, 123);
    assert_eq!(
        decoded.stdout_hash,
        Some("abcdef0123456789".into()),
        "captured stdout_hash must survive msgpack round-trip"
    );
}

// ---------------------------------------------------------------------------
// 3. TestIntegrityEvent{Verified}: SSB round-trip + sql_str bijection
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_test_integrity_event_verified_roundtrips() {
    let tmp = TempDir::new().expect("tempdir");
    let cfg = SsbConfig {
        db_path: tmp.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: "f4-b4-verified".into(),
        session_id: "f4-b4-verified-session".into(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(Ssb::new(cfg, clock).expect("ssb"));

    let payload = SsbPayload::TestIntegrityEvent(TestIntegrityEventPayload {
        test_name: "crate::contracts::verified_test".into(),
        passed: true,
        stdout_hash: Some("0011223344556677".into()),
        kind: TestIntegrityEventKind::Verified,
    });
    assert_eq!(
        payload.action_type(),
        SsbActionType::TestIntegrityEvent(TestIntegrityEventKind::Verified)
    );

    ssb.write_payload("task_runner", 0, payload.clone(), Vec::new())
        .await
        .expect("write");

    let rows = ssb
        .query_by_type(SsbActionType::TestIntegrityEvent(
            TestIntegrityEventKind::Verified,
        ))
        .expect("query");
    assert_eq!(rows.len(), 1, "exactly one Verified row must be stored");

    let decoded: SsbPayload = rmp_serde::from_slice(&rows[0].payload).expect("decode");
    match decoded {
        SsbPayload::TestIntegrityEvent(p) => {
            assert_eq!(p.test_name, "crate::contracts::verified_test");
            assert!(p.passed);
            assert_eq!(p.stdout_hash, Some("0011223344556677".into()));
            assert_eq!(p.kind, TestIntegrityEventKind::Verified);
        }
        other => panic!("expected TestIntegrityEvent, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// 4. TestIntegrityEvent{FabricationBlocked}: SSB round-trip + sql_str
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_test_integrity_event_fabrication_blocked_roundtrips() {
    let tmp = TempDir::new().expect("tempdir");
    let cfg = SsbConfig {
        db_path: tmp.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: "f4-b4-blocked".into(),
        session_id: "f4-b4-blocked-session".into(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(Ssb::new(cfg, clock).expect("ssb"));

    let payload = SsbPayload::TestIntegrityEvent(TestIntegrityEventPayload {
        test_name: "integration::fabricated_test".into(),
        passed: true,
        stdout_hash: None,
        kind: TestIntegrityEventKind::FabricationBlocked,
    });
    assert_eq!(
        payload.action_type(),
        SsbActionType::TestIntegrityEvent(TestIntegrityEventKind::FabricationBlocked)
    );

    ssb.write_payload("task_runner", 0, payload.clone(), Vec::new())
        .await
        .expect("write");

    let rows = ssb
        .query_by_type(SsbActionType::TestIntegrityEvent(
            TestIntegrityEventKind::FabricationBlocked,
        ))
        .expect("query");
    assert_eq!(
        rows.len(),
        1,
        "exactly one FabricationBlocked row must be stored"
    );

    let decoded: SsbPayload = rmp_serde::from_slice(&rows[0].payload).expect("decode");
    match decoded {
        SsbPayload::TestIntegrityEvent(p) => {
            assert_eq!(p.test_name, "integration::fabricated_test");
            assert!(p.passed);
            assert_eq!(p.stdout_hash, None);
            assert_eq!(p.kind, TestIntegrityEventKind::FabricationBlocked);
        }
        other => panic!("expected TestIntegrityEvent(FabricationBlocked), got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// 5. sql_str bijection: Verified / FabricationBlocked
// ---------------------------------------------------------------------------

#[test]
fn contract_test_integrity_verified_sql_str_bijection() {
    let kind = TestIntegrityEventKind::Verified;
    assert_eq!(kind.as_sql_str(), "test_integrity_verified");
    assert_eq!(
        SsbActionType::TestIntegrityEvent(kind).as_sql_str(),
        "test_integrity_verified"
    );
    let parsed =
        ssb_action_type_from_str_public("test_integrity_verified").expect("parse verified");
    assert_eq!(
        parsed,
        SsbActionType::TestIntegrityEvent(TestIntegrityEventKind::Verified)
    );
}

#[test]
fn contract_test_integrity_fabrication_blocked_sql_str_bijection() {
    let kind = TestIntegrityEventKind::FabricationBlocked;
    assert_eq!(kind.as_sql_str(), "anti_fabrication_block");
    assert_eq!(
        SsbActionType::TestIntegrityEvent(kind).as_sql_str(),
        "anti_fabrication_block"
    );
    let parsed =
        ssb_action_type_from_str_public("anti_fabrication_block").expect("parse fabrication_blocked");
    assert_eq!(
        parsed,
        SsbActionType::TestIntegrityEvent(TestIntegrityEventKind::FabricationBlocked)
    );
}

// ---------------------------------------------------------------------------
// 6. SP5 fail-closed: BLOCK when integration_test_rows is None
// ---------------------------------------------------------------------------

#[test]
fn contract_anti_fabrication_gate_blocks_on_none_rows() {
    let ctx = TransitionContext {
        integration_test_rows: None,
        ..Default::default()
    };
    let err = AntiFabricationGate.check(&ctx).unwrap_err();
    assert_eq!(err.gate_name, "anti_fabrication");
    assert!(
        err.missing_artifacts
            .iter()
            .any(|a| a == "integration_test_rows"),
        "absent integration_test_rows must hard-FAIL AUDITING -> MEMORIZING \
         (R1/R2: missing evidence = proof of absence); got {:?}",
        err.missing_artifacts
    );
}

// ---------------------------------------------------------------------------
// 7. AntiFabricationGate: ALLOW when all rows have stdout_hash=Some
// ---------------------------------------------------------------------------

#[test]
fn contract_anti_fabrication_gate_allows_when_all_rows_have_hash() {
    let rows = vec![
        IntegrationTestRowSummary {
            test_name: "test_alpha".into(),
            passed: true,
            stdout_hash: Some("aabbccddeeff0011".into()),
        },
        IntegrationTestRowSummary {
            test_name: "test_beta".into(),
            passed: true,
            stdout_hash: Some("112233445566778899aa".into()),
        },
        IntegrationTestRowSummary {
            test_name: "test_gamma_failed".into(),
            passed: false,
            stdout_hash: None,
        },
    ];
    let ctx = TransitionContext {
        integration_test_rows: Some(rows),
        ..Default::default()
    };
    assert!(
        AntiFabricationGate.check(&ctx).is_ok(),
        "gate must ALLOW when every passing row has a stdout_hash; \
         failed rows with None hash are not a fabrication signal"
    );
}

// ---------------------------------------------------------------------------
// 8. AntiFabricationGate: BLOCK when a passing row has stdout_hash=None
// ---------------------------------------------------------------------------

#[test]
fn contract_anti_fabrication_gate_blocks_when_passing_row_has_no_hash() {
    let rows = vec![
        IntegrationTestRowSummary {
            test_name: "test_clean".into(),
            passed: true,
            stdout_hash: Some("deadbeefcafe0000".into()),
        },
        IntegrationTestRowSummary {
            test_name: "test_fabricated".into(),
            passed: true,
            stdout_hash: None,
        },
    ];
    let ctx = TransitionContext {
        integration_test_rows: Some(rows),
        ..Default::default()
    };
    let result = AntiFabricationGate.check(&ctx);
    assert!(
        result.is_err(),
        "gate must BLOCK when a passing row is missing stdout_hash"
    );
    let err = result.unwrap_err();
    assert!(
        err.missing_artifacts.contains(&"test_fabricated".to_string()),
        "error must name the fabricated test; got: {:?}",
        err.missing_artifacts
    );
    assert_eq!(
        err.gate_name, "anti_fabrication",
        "gate_name must be 'anti_fabrication'"
    );
}
