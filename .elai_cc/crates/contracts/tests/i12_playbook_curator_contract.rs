// I-12: evidence-gated PlaybookCurator contract tests.
//
// Validates:
//   ARM-1: rule with evidence_count < MIN_EVIDENCE_COUNT emits
//          PlaybookEvent{ValidationRun} SSB row, does NOT promote to ACTIVE.
//   ARM-2: rule with evidence_count >= MIN_EVIDENCE_COUNT AND regression_count == 0
//          AND passing metrics promotes to ACTIVE and emits PlaybookEvent{Promoted}
//          with non-empty outcome_row_ids.
//   ARM-3: ACTIVE rule with FPR >= FPR_RETIRE_THRESHOLD gets retired and emits
//          PlaybookEvent{Demoted} SSB row.
//   ARM-4: export_verified writes an AGENTS.md block containing promoted entries
//          with embedded outcome_row_ids (grep check).
//   ARM-5: PlaybookEventPayload round-trips through SSB msgpack serde correctly.
//
// All tests: in-memory rules DB (":memory:"), TempDir SSB (no live inference,
// no GPU, no network). D-026 compliant.
//
// SCHEMA_VERSION 251, TOTAL_VARIANTS 560.

use std::path::Path;
use std::sync::Arc;

use elai_core::clock::{OsClock, SeededClock};
use elai_orchestrator::{
    ssb::{
        PlaybookEventKind, PlaybookEventPayload, SsbActionType, SsbConfig, SsbPayload, SsbReader,
    },
    rule_curator::{MIN_EVIDENCE_COUNT, PlaybookCurator},
    Ssb,
};
use elai_rules::{
    lifecycle::{FPR_RETIRE_THRESHOLD, MIN_CONTRACT_TESTS},
    schema::open_rules_db,
};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn test_clock() -> Arc<SeededClock> {
    Arc::new(SeededClock::new())
}

fn mem_rules_db() -> rusqlite::Connection {
    open_rules_db(Path::new(":memory:"), &SeededClock::new()).expect("in-memory rules DB")
}

fn make_ssb(tmp: &TempDir) -> Arc<Ssb> {
    let cfg = SsbConfig {
        db_path: tmp.path().join("i12-ssb.db"),
        pool_size: Some(2),
        project_id: "i12-test".into(),
        session_id: "i12-contract-session".into(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    Arc::new(Ssb::new(cfg, clock).expect("i12 ssb init"))
}

fn insert_proposed_rule(conn: &rusqlite::Connection, rule_id: &str) {
    conn.execute(
        "INSERT INTO rules \
         (rule_id, domain, tier, status, title, description, \
          predicate, severity, mutable, created_at, source) \
         VALUES (?1, 'test', 1, 'PROPOSED', 'Test rule', 'desc', \
                 '{\"type\":\"regex\",\"pattern\":\".*\"}', 'WARN', 1, 1000, 'SEED')",
        rusqlite::params![rule_id],
    )
    .expect("insert rule");
}

fn insert_contract_tests(conn: &rusqlite::Connection, rule_id: &str, count: u32) {
    for i in 0..count {
        conn.execute(
            "INSERT INTO contract_tests \
             (test_id, rule_id, test_type, input_code, expected, language, created_at, source) \
             VALUES (?1, ?2, 'BLOCK', 'x=1', 'BLOCK', 'python', 1000, 'SEED')",
            rusqlite::params![format!("ct-{rule_id}-{i}"), rule_id],
        )
        .expect("insert contract_test");
    }
}

/// Insert rule_metrics rows so the rule passes FPR/TPR/CCR gates.
fn insert_passing_metrics(conn: &rusqlite::Connection, rule_id: &str) {
    // fpr = fp/(tp+fp) = 2/102 ≈ 0.02 < 0.05 (FPR_ACTIVE_THRESHOLD).
    // tpr = tp/(tp+missed) = 90/90 = 1.0 > 0.80 (TPR_ACTIVE_THRESHOLD).
    // ccr = causal/(causal+spurious) = 85/87 ≈ 0.98 > 0.50 (CCR_ACTIVE_THRESHOLD).
    conn.execute(
        "INSERT INTO rule_metrics \
         (rule_id, window_start, evaluations, \
          true_positives, false_positives, missed, causal_catches, spurious_catches) \
         VALUES (?1, 0, 104, 100, 2, 0, 85, 2)",
        rusqlite::params![rule_id],
    )
    .expect("insert rule_metrics");
}

/// Insert rule_metrics rows so the rule will FAIL the FPR gate (FPR ≥ FPR_RETIRE_THRESHOLD).
fn insert_failing_fpr_metrics(conn: &rusqlite::Connection, rule_id: &str) {
    // fpr = 40/50 = 0.80 > FPR_RETIRE_THRESHOLD (0.30).
    conn.execute(
        "INSERT INTO rule_metrics \
         (rule_id, window_start, evaluations, \
          true_positives, false_positives, missed, causal_catches, spurious_catches) \
         VALUES (?1, 0, 60, 10, 40, 5, 1, 10)",
        rusqlite::params![rule_id],
    )
    .expect("insert failing fpr metrics");
}

/// Emit N evidence-link SSB rows for rule_id by registering rule_id as a file_path.
async fn emit_evidence_rows(ssb: &Arc<Ssb>, rule_id: &str, count: u32) {
    for i in 0..count {
        let payload = PlaybookEventPayload {
            rule_id: rule_id.to_owned(),
            kind: PlaybookEventKind::ValidationRun,
            outcome_row_ids: vec![],
            lift: None,
            regression_count: 0,
            evidence_count: i,
            ts_ms: 1_000_000 + i64::from(i),
        };
        ssb.write_payload(
            "test-fixture",
            0,
            SsbPayload::PlaybookEvent(payload),
            vec![rule_id.to_owned()], // register rule_id as file_path for query_by_file
        )
        .await
        .expect("emit evidence row");
    }
}

// ---------------------------------------------------------------------------
// ARM-1: insufficient evidence => ValidationRun emitted, NOT promoted
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i12_arm1_insufficient_evidence_emits_validation_run() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp);

    let conn = mem_rules_db();
    insert_proposed_rule(&conn, "rule-arm1");
    insert_contract_tests(&conn, "rule-arm1", MIN_CONTRACT_TESTS);
    insert_passing_metrics(&conn, "rule-arm1");

    // Emit fewer than MIN_EVIDENCE_COUNT rows.
    let too_few = MIN_EVIDENCE_COUNT - 1;
    emit_evidence_rows(&ssb, "rule-arm1", too_few).await;

    // Write the in-memory rules DB to a temp path so PlaybookCurator::new can open it.
    let rules_db_path = tmp.path().join("rules.db");
    // Re-open with a file-backed DB using the same clock.
    let file_conn = open_rules_db(&rules_db_path, &SeededClock::new())
        .expect("file rules db");
    insert_proposed_rule(&file_conn, "rule-arm1");
    insert_contract_tests(&file_conn, "rule-arm1", MIN_CONTRACT_TESTS);
    insert_passing_metrics(&file_conn, "rule-arm1");
    drop(file_conn);

    let curator = PlaybookCurator::new(&rules_db_path, test_clock());
    curator.run_promotion_tick(&ssb).await.expect("promotion tick");

    // Assert: at least one ValidationRun SSB row was written for rule-arm1.
    let validation_rows = ssb
        .query_by_type(SsbActionType::PlaybookEvent(PlaybookEventKind::ValidationRun))
        .expect("query ValidationRun rows");
    let arm1_rows: Vec<_> = validation_rows
        .iter()
        .filter(|e| {
            rmp_serde::from_slice::<SsbPayload>(&e.payload)
                .ok()
                .and_then(|p| if let SsbPayload::PlaybookEvent(ref pe) = p {
                    Some(pe.rule_id == "rule-arm1")
                } else { None })
                .unwrap_or(false)
        })
        .collect();
    assert!(
        !arm1_rows.is_empty(),
        "ARM-1: expected at least one PlaybookEvent(ValidationRun) for rule-arm1, found 0"
    );

    // Assert: NOT promoted (no Promoted row for rule-arm1).
    // query_by_type returns ALL playbook_event rows; filter by rule_id AND kind=Promoted.
    let all_rows = ssb
        .query_by_type(SsbActionType::PlaybookEvent(PlaybookEventKind::Promoted))
        .expect("query playbook_event rows");
    let arm1_promoted: Vec<_> = all_rows
        .iter()
        .filter(|e| {
            rmp_serde::from_slice::<SsbPayload>(&e.payload)
                .ok()
                .and_then(|p| if let SsbPayload::PlaybookEvent(ref pe) = p {
                    Some(pe.rule_id == "rule-arm1" && pe.kind == PlaybookEventKind::Promoted)
                } else { None })
                .unwrap_or(false)
        })
        .collect();
    assert!(
        arm1_promoted.is_empty(),
        "ARM-1: rule-arm1 must NOT be promoted with insufficient evidence; found {} Promoted rows",
        arm1_promoted.len()
    );
}

// ---------------------------------------------------------------------------
// ARM-2: sufficient evidence + passing metrics => Promoted + ACTIVE
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i12_arm2_sufficient_evidence_promotes_and_emits_promoted() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp);

    // Set up file-backed rules DB.
    let rules_db_path = tmp.path().join("rules-arm2.db");
    let file_conn = open_rules_db(&rules_db_path, &SeededClock::new())
        .expect("file rules db");
    insert_proposed_rule(&file_conn, "rule-arm2");
    insert_contract_tests(&file_conn, "rule-arm2", MIN_CONTRACT_TESTS);
    insert_passing_metrics(&file_conn, "rule-arm2");
    drop(file_conn);

    // Emit exactly MIN_EVIDENCE_COUNT evidence rows linked to rule-arm2.
    emit_evidence_rows(&ssb, "rule-arm2", MIN_EVIDENCE_COUNT).await;

    let curator = PlaybookCurator::new(&rules_db_path, test_clock());
    curator.run_promotion_tick(&ssb).await.expect("promotion tick");

    // Assert: PlaybookEvent{Promoted} row written for rule-arm2.
    // query_by_type returns ALL playbook_event rows; filter by rule_id AND kind=Promoted.
    let all_rows = ssb
        .query_by_type(SsbActionType::PlaybookEvent(PlaybookEventKind::Promoted))
        .expect("query playbook_event rows arm2");

    let arm2_promoted: Vec<_> = all_rows
        .iter()
        .filter_map(|e| {
            rmp_serde::from_slice::<SsbPayload>(&e.payload).ok().and_then(|p| {
                if let SsbPayload::PlaybookEvent(pe) = p {
                    if pe.rule_id == "rule-arm2" && pe.kind == PlaybookEventKind::Promoted {
                        Some(pe)
                    } else { None }
                } else { None }
            })
        })
        .collect();

    assert_eq!(
        arm2_promoted.len(), 1,
        "ARM-2: expected exactly 1 PlaybookEvent(Promoted) for rule-arm2, got {}",
        arm2_promoted.len()
    );

    let promoted = &arm2_promoted[0];
    assert_eq!(
        promoted.kind,
        PlaybookEventKind::Promoted,
        "ARM-2: kind must be Promoted"
    );
    assert!(
        !promoted.outcome_row_ids.is_empty(),
        "ARM-2: Promoted row must have non-empty outcome_row_ids (evidence must be embedded)"
    );
    assert!(
        promoted.lift.is_some(),
        "ARM-2: Promoted row must carry a lift estimate"
    );
    assert_eq!(
        promoted.regression_count, 0,
        "ARM-2: regression_count must be 0 on Promoted path"
    );

    // Assert: rules DB shows ACTIVE status.
    let check_conn = open_rules_db(&rules_db_path, &SeededClock::new())
        .expect("reopen rules db");
    let status: String = check_conn
        .query_row(
            "SELECT status FROM rules WHERE rule_id = 'rule-arm2'",
            [],
            |row| row.get(0),
        )
        .expect("query status");
    assert_eq!(
        status, "ACTIVE",
        "ARM-2: rule-arm2 must be in ACTIVE status after promotion tick"
    );
}

// ---------------------------------------------------------------------------
// ARM-3: ACTIVE rule with high FPR => Demoted + RETIRED
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i12_arm3_high_fpr_active_rule_demoted() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp);

    // Seed an ACTIVE rule directly (skip the promotion path).
    let rules_db_path = tmp.path().join("rules-arm3.db");
    let file_conn = open_rules_db(&rules_db_path, &SeededClock::new())
        .expect("file rules db arm3");
    // Insert as ACTIVE directly (bypassing lifecycle for the test setup).
    file_conn.execute(
        "INSERT INTO rules \
         (rule_id, domain, tier, status, title, description, \
          predicate, severity, mutable, created_at, promoted_at, source) \
         VALUES ('rule-arm3', 'test', 1, 'ACTIVE', 'Test', 'desc', \
                 '{\"type\":\"regex\",\"pattern\":\".*\"}', 'WARN', 1, 1000, 1000, 'SEED')",
        [],
    )
    .expect("insert ACTIVE rule");
    insert_failing_fpr_metrics(&file_conn, "rule-arm3");
    drop(file_conn);

    let curator = PlaybookCurator::new(&rules_db_path, test_clock());
    curator.run_promotion_tick(&ssb).await.expect("promotion tick");

    // Assert: PlaybookEvent{Demoted} row written.
    let demoted_rows = ssb
        .query_by_type(SsbActionType::PlaybookEvent(PlaybookEventKind::Demoted))
        .expect("query Demoted rows");
    let arm3_demoted: Vec<_> = demoted_rows
        .iter()
        .filter(|e| {
            rmp_serde::from_slice::<SsbPayload>(&e.payload)
                .ok()
                .and_then(|p| if let SsbPayload::PlaybookEvent(ref pe) = p {
                    Some(pe.rule_id == "rule-arm3")
                } else { None })
                .unwrap_or(false)
        })
        .collect();
    assert!(
        !arm3_demoted.is_empty(),
        "ARM-3: expected PlaybookEvent(Demoted) for rule-arm3 with FPR={:.2} >= FPR_RETIRE_THRESHOLD={:.2}",
        0.80_f64, FPR_RETIRE_THRESHOLD
    );

    // Assert: rules DB shows RETIRED status.
    let check_conn = open_rules_db(&rules_db_path, &SeededClock::new())
        .expect("reopen rules db arm3");
    let status: String = check_conn
        .query_row(
            "SELECT status FROM rules WHERE rule_id = 'rule-arm3'",
            [],
            |row| row.get(0),
        )
        .expect("query status arm3");
    assert_eq!(
        status, "RETIRED",
        "ARM-3: rule-arm3 must be in RETIRED status after demotion sweep"
    );
}

// ---------------------------------------------------------------------------
// ARM-4: export_verified writes AGENTS.md block with outcome_row_ids
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i12_arm4_export_verified_writes_block_with_outcome_ids() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp);

    // Seed one ACTIVE rule.
    let rules_db_path = tmp.path().join("rules-arm4.db");
    let file_conn = open_rules_db(&rules_db_path, &SeededClock::new())
        .expect("file rules db arm4");
    file_conn.execute(
        "INSERT INTO rules \
         (rule_id, domain, tier, status, title, description, \
          predicate, severity, mutable, created_at, promoted_at, source) \
         VALUES ('rule-arm4', 'test', 1, 'ACTIVE', 'Export Test', 'desc', \
                 '{\"type\":\"regex\",\"pattern\":\".*\"}', 'WARN', 1, 1000, 1000, 'SEED')",
        [],
    )
    .expect("insert ACTIVE rule arm4");
    drop(file_conn);

    // Emit evidence rows so query_by_file finds them.
    emit_evidence_rows(&ssb, "rule-arm4", MIN_EVIDENCE_COUNT).await;

    let export_path = tmp.path().join("AGENTS.md");
    let curator = PlaybookCurator::new(&rules_db_path, test_clock());
    let report = curator
        .export_verified(&ssb, &export_path, false)
        .await
        .expect("export_verified must succeed");

    assert!(
        export_path.exists(),
        "ARM-4: AGENTS.md must be written to {}", export_path.display()
    );
    assert_eq!(
        report.entries_exported, 1,
        "ARM-4: exactly 1 ACTIVE rule must be exported"
    );

    let content = std::fs::read_to_string(&export_path).expect("read AGENTS.md");
    assert!(
        content.contains("rule-arm4"),
        "ARM-4: exported AGENTS.md must contain rule_id 'rule-arm4'"
    );
    assert!(
        content.contains("outcome_row_ids"),
        "ARM-4: exported AGENTS.md must contain 'outcome_row_ids' block"
    );
}

// ---------------------------------------------------------------------------
// ARM-5: PlaybookEventPayload SSB serde roundtrip
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i12_arm5_playbook_event_payload_ssb_roundtrip() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp);

    let payload_in = PlaybookEventPayload {
        rule_id: "rule-serde-test".to_owned(),
        kind: PlaybookEventKind::Promoted,
        outcome_row_ids: vec!["1".into(), "2".into(), "3".into()],
        lift: Some(0.87),
        regression_count: 0,
        evidence_count: 7,
        ts_ms: 1_700_000_000_000,
    };

    ssb.write_payload(
        "i12-serde-test",
        0,
        SsbPayload::PlaybookEvent(payload_in.clone()),
        vec![],
    )
    .await
    .expect("write PlaybookEvent");

    let rows = ssb
        .query_by_type(SsbActionType::PlaybookEvent(PlaybookEventKind::Promoted))
        .expect("query PlaybookEvent rows");
    assert_eq!(rows.len(), 1, "ARM-5: expected 1 Promoted row, got {}", rows.len());

    let decoded = rmp_serde::from_slice::<SsbPayload>(&rows[0].payload)
        .expect("ARM-5: SsbPayload must deserialize from msgpack");

    if let SsbPayload::PlaybookEvent(ref out) = decoded {
        assert_eq!(out.rule_id, payload_in.rule_id, "ARM-5: rule_id roundtrip");
        assert_eq!(out.kind, payload_in.kind, "ARM-5: kind roundtrip");
        assert_eq!(out.outcome_row_ids, payload_in.outcome_row_ids, "ARM-5: outcome_row_ids roundtrip");
        assert_eq!(out.evidence_count, payload_in.evidence_count, "ARM-5: evidence_count roundtrip");
        assert!((out.lift.unwrap_or(0.0) - payload_in.lift.unwrap_or(0.0)).abs() < 1e-5_f32,
            "ARM-5: lift roundtrip");
    } else {
        panic!("ARM-5: expected PlaybookEvent payload, got {:?}", decoded);
    }
}
