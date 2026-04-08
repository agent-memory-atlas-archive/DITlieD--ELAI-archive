//! Contract tests for rules/ lifecycle state machine (Phase 3F.4).
//!
//! Validates: lifecycle transitions, gate checks (insufficient tests,
//! circular trust, threshold failures, mutable guard), transition logging,
//! metrics snapshot computation, and CCR zero-denominator handling.

use std::path::Path;

use elai_core::clock::SeededClock;
use elai_rules::error::LifecycleError;
use elai_rules::lifecycle::{
    compute_metrics_snapshot, fetch_rule_record, transition_to_active, transition_to_candidate,
    transition_to_retired, transition_to_shadow, Approver, LifecycleStatus, MetricsSnapshot,
};
use elai_rules::schema::open_rules_db;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn test_clock() -> SeededClock {
    SeededClock::new()
}

fn mem_db() -> rusqlite::Connection {
    open_rules_db(Path::new(":memory:"), &test_clock()).expect("in-memory DB should open")
}

/// Insert a minimal rule row in PROPOSED status.
fn insert_proposed_rule(conn: &rusqlite::Connection, rule_id: &str, source: &str, mutable: bool) {
    conn.execute(
        "INSERT INTO rules (rule_id, domain, tier, status, title, description, \
         predicate, severity, mutable, created_at, source) \
         VALUES (?1, 'test', 1, 'PROPOSED', 'Test Rule', 'desc', \
         '{\"type\":\"regex\",\"pattern\":\".*\"}', 'WARN', ?2, 1000, ?3)",
        rusqlite::params![rule_id, mutable as i32, source],
    )
    .expect("insert proposed rule");
}

/// Insert a rule at a specific status.
fn insert_rule_at(
    conn: &rusqlite::Connection,
    rule_id: &str,
    status: &str,
    source: &str,
    mutable: bool,
) {
    conn.execute(
        "INSERT INTO rules (rule_id, domain, tier, status, title, description, \
         predicate, severity, mutable, created_at, source) \
         VALUES (?1, 'test', 1, ?2, 'Test Rule', 'desc', \
         '{\"type\":\"regex\",\"pattern\":\".*\"}', 'WARN', ?3, 1000, ?4)",
        rusqlite::params![rule_id, status, mutable as i32, source],
    )
    .expect("insert rule");
}

/// Insert rule_metrics rows for a rule.
fn insert_metrics(
    conn: &rusqlite::Connection,
    rule_id: &str,
    window_start: i64,
    tp: i64,
    fp: i64,
    causal: i64,
    spurious: i64,
) {
    conn.execute(
        "INSERT INTO rule_metrics (rule_id, window_start, evaluations, \
         true_positives, false_positives, causal_catches, spurious_catches) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![rule_id, window_start, tp + fp, tp, fp, causal, spurious],
    )
    .expect("insert metrics");
}

// ---------------------------------------------------------------------------
// PROPOSED -> CANDIDATE contracts
// ---------------------------------------------------------------------------

#[test]
fn contract_proposed_to_candidate_success() {
    let conn = mem_db();
    insert_proposed_rule(&conn, "r1", "SEED", true);
    let clock = test_clock();
    transition_to_candidate(&conn, "r1", 3, true, &clock).expect("should succeed");
    let record = fetch_rule_record(&conn, "r1").unwrap();
    assert!(matches!(record.status, LifecycleStatus::Candidate));
}

#[test]
fn contract_proposed_to_candidate_insufficient_tests() {
    let conn = mem_db();
    insert_proposed_rule(&conn, "r1", "SEED", true);
    let clock = test_clock();
    let err = transition_to_candidate(&conn, "r1", 2, true, &clock).unwrap_err();
    assert!(matches!(err, LifecycleError::InsufficientContractTests { .. }));
}

#[test]
fn contract_proposed_to_candidate_tests_failing() {
    let conn = mem_db();
    insert_proposed_rule(&conn, "r1", "SEED", true);
    let clock = test_clock();
    let err = transition_to_candidate(&conn, "r1", 5, false, &clock).unwrap_err();
    assert!(matches!(err, LifecycleError::ContractTestsFailing));
}

#[test]
fn contract_proposed_to_candidate_wrong_source_state() {
    let conn = mem_db();
    insert_rule_at(&conn, "r1", "ACTIVE", "SEED", true);
    let clock = test_clock();
    let err = transition_to_candidate(&conn, "r1", 5, true, &clock).unwrap_err();
    assert!(matches!(err, LifecycleError::InvalidTransition { .. }));
}

// ---------------------------------------------------------------------------
// CANDIDATE -> SHADOW contracts
// ---------------------------------------------------------------------------

#[test]
fn contract_candidate_to_shadow_human_approver() {
    let conn = mem_db();
    insert_rule_at(&conn, "r1", "CANDIDATE", "SEED", true);
    let clock = test_clock();
    transition_to_shadow(&conn, "r1", &Approver::Human, &clock).expect("human approval ok");
    let record = fetch_rule_record(&conn, "r1").unwrap();
    assert!(matches!(record.status, LifecycleStatus::Shadow));
}

#[test]
fn contract_candidate_to_shadow_second_auditor() {
    let conn = mem_db();
    insert_rule_at(&conn, "r1", "CANDIDATE", "AUDITOR", true);
    let clock = test_clock();
    let approver = Approver::SecondAuditor {
        auditor_id: "aud-002".to_string(),
    };
    transition_to_shadow(&conn, "r1", &approver, &clock).expect("second auditor ok");
    let record = fetch_rule_record(&conn, "r1").unwrap();
    assert!(matches!(record.status, LifecycleStatus::Shadow));
}

#[test]
fn contract_candidate_to_shadow_circular_trust_rejected() {
    let conn = mem_db();
    insert_rule_at(&conn, "r1", "CANDIDATE", "AUDITOR", true);
    let clock = test_clock();
    let approver = Approver::FindingSourceAuditor {
        auditor_id: "aud-001".to_string(),
    };
    let err = transition_to_shadow(&conn, "r1", &approver, &clock).unwrap_err();
    assert!(matches!(err, LifecycleError::CircularTrust));
}

#[test]
fn contract_candidate_to_shadow_wrong_source_state() {
    let conn = mem_db();
    insert_rule_at(&conn, "r1", "PROPOSED", "SEED", true);
    let clock = test_clock();
    let err = transition_to_shadow(&conn, "r1", &Approver::Human, &clock).unwrap_err();
    assert!(matches!(err, LifecycleError::InvalidTransition { .. }));
}

// ---------------------------------------------------------------------------
// SHADOW -> ACTIVE contracts
// ---------------------------------------------------------------------------

#[test]
fn contract_shadow_to_active_success() {
    let conn = mem_db();
    insert_rule_at(&conn, "r1", "SHADOW", "SEED", true);
    let clock = test_clock();
    let metrics = MetricsSnapshot {
        true_positives: 90,
        false_positives: 2,
        causal_catches: 80,
        spurious_catches: 10,
        false_positive_rate: 0.02,
        true_positive_rate: 0.90,
        counterfactual_causal_rate: Some(0.89),
    };
    transition_to_active(&conn, "r1", &metrics, false, &clock).expect("should promote");
    let record = fetch_rule_record(&conn, "r1").unwrap();
    assert!(matches!(record.status, LifecycleStatus::Active));
    assert!(record.promoted_at.is_some());
}

#[test]
fn contract_shadow_to_active_fpr_too_high() {
    let conn = mem_db();
    insert_rule_at(&conn, "r1", "SHADOW", "SEED", true);
    let clock = test_clock();
    let metrics = MetricsSnapshot {
        true_positives: 80,
        false_positives: 20,
        causal_catches: 70,
        spurious_catches: 10,
        false_positive_rate: 0.20,
        true_positive_rate: 0.90,
        counterfactual_causal_rate: Some(0.88),
    };
    let err = transition_to_active(&conn, "r1", &metrics, false, &clock).unwrap_err();
    assert!(matches!(err, LifecycleError::ThresholdNotMet { .. }));
}

#[test]
fn contract_shadow_to_active_tpr_too_low() {
    let conn = mem_db();
    insert_rule_at(&conn, "r1", "SHADOW", "SEED", true);
    let clock = test_clock();
    let metrics = MetricsSnapshot {
        true_positives: 50,
        false_positives: 2,
        causal_catches: 40,
        spurious_catches: 10,
        false_positive_rate: 0.03,
        true_positive_rate: 0.50,
        counterfactual_causal_rate: Some(0.80),
    };
    let err = transition_to_active(&conn, "r1", &metrics, false, &clock).unwrap_err();
    assert!(matches!(err, LifecycleError::ThresholdNotMet { .. }));
}

#[test]
fn contract_shadow_to_active_ccr_too_low() {
    let conn = mem_db();
    insert_rule_at(&conn, "r1", "SHADOW", "SEED", true);
    let clock = test_clock();
    let metrics = MetricsSnapshot {
        true_positives: 90,
        false_positives: 2,
        causal_catches: 20,
        spurious_catches: 80,
        false_positive_rate: 0.02,
        true_positive_rate: 0.90,
        counterfactual_causal_rate: Some(0.20),
    };
    let err = transition_to_active(&conn, "r1", &metrics, false, &clock).unwrap_err();
    assert!(matches!(err, LifecycleError::ThresholdNotMet { .. }));
}

#[test]
fn contract_shadow_to_active_insufficient_shadow_data() {
    let conn = mem_db();
    insert_rule_at(&conn, "r1", "SHADOW", "SEED", true);
    let clock = test_clock();
    let metrics = MetricsSnapshot {
        true_positives: 0,
        false_positives: 0,
        causal_catches: 0,
        spurious_catches: 0,
        false_positive_rate: 0.0,
        true_positive_rate: 0.0,
        counterfactual_causal_rate: None,
    };
    let err = transition_to_active(&conn, "r1", &metrics, false, &clock).unwrap_err();
    assert!(matches!(err, LifecycleError::InsufficientShadowData));
}

#[test]
fn contract_shadow_to_active_safety_regression_blocks() {
    let conn = mem_db();
    insert_rule_at(&conn, "r1", "SHADOW", "SEED", true);
    let clock = test_clock();
    let metrics = MetricsSnapshot {
        true_positives: 90,
        false_positives: 2,
        causal_catches: 80,
        spurious_catches: 10,
        false_positive_rate: 0.02,
        true_positive_rate: 0.90,
        counterfactual_causal_rate: Some(0.89),
    };
    let err = transition_to_active(&conn, "r1", &metrics, true, &clock).unwrap_err();
    assert!(matches!(err, LifecycleError::SafetyRegression));
}

#[test]
fn contract_shadow_to_active_mutable0_ccr_exempt() {
    let conn = mem_db();
    insert_rule_at(&conn, "r1", "SHADOW", "SEED", false);
    let clock = test_clock();
    // mutable=false rules are exempt from CCR gate
    let metrics = MetricsSnapshot {
        true_positives: 90,
        false_positives: 2,
        causal_catches: 0,
        spurious_catches: 0,
        false_positive_rate: 0.02,
        true_positive_rate: 0.90,
        counterfactual_causal_rate: None,
    };
    transition_to_active(&conn, "r1", &metrics, false, &clock)
        .expect("mutable=0 should bypass CCR gate");
    let record = fetch_rule_record(&conn, "r1").unwrap();
    assert!(matches!(record.status, LifecycleStatus::Active));
}

// ---------------------------------------------------------------------------
// ACTIVE -> RETIRED contracts
// ---------------------------------------------------------------------------

#[test]
fn contract_active_to_retired_success() {
    let conn = mem_db();
    insert_rule_at(&conn, "r1", "ACTIVE", "SEED", true);
    let clock = test_clock();
    transition_to_retired(&conn, "r1", "fpr degraded", false, &clock).expect("should retire");
    let record = fetch_rule_record(&conn, "r1").unwrap();
    assert!(matches!(record.status, LifecycleStatus::Retired));
}

#[test]
fn contract_active_to_retired_mutable0_auto_blocked() {
    let conn = mem_db();
    insert_rule_at(&conn, "r1", "ACTIVE", "SEED", false);
    let clock = test_clock();
    let err = transition_to_retired(&conn, "r1", "auto degraded", false, &clock).unwrap_err();
    assert!(matches!(err, LifecycleError::ImmutableRuleCannotAutoRetire));
}

#[test]
fn contract_active_to_retired_mutable0_force_allowed() {
    let conn = mem_db();
    insert_rule_at(&conn, "r1", "ACTIVE", "SEED", false);
    let clock = test_clock();
    transition_to_retired(&conn, "r1", "human override", true, &clock)
        .expect("force=true should bypass mutable guard");
    let record = fetch_rule_record(&conn, "r1").unwrap();
    assert!(matches!(record.status, LifecycleStatus::Retired));
}

// ---------------------------------------------------------------------------
// Transition logging contracts
// ---------------------------------------------------------------------------

#[test]
fn contract_transition_log_written() {
    let conn = mem_db();
    insert_proposed_rule(&conn, "r1", "SEED", true);
    let clock = test_clock();
    transition_to_candidate(&conn, "r1", 3, true, &clock).unwrap();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM rule_transitions WHERE rule_id = 'r1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1, "one transition log row expected");

    let (from_s, to_s): (String, String) = conn
        .query_row(
            "SELECT from_status, to_status FROM rule_transitions WHERE rule_id = 'r1'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(from_s, "PROPOSED");
    assert_eq!(to_s, "CANDIDATE");
}

// ---------------------------------------------------------------------------
// compute_metrics_snapshot contracts
// ---------------------------------------------------------------------------

#[test]
fn contract_compute_metrics_multi_window() {
    let conn = mem_db();
    insert_rule_at(&conn, "r1", "SHADOW", "SEED", true);
    insert_metrics(&conn, "r1", 1000, 40, 2, 30, 5);
    insert_metrics(&conn, "r1", 2000, 50, 1, 40, 3);

    let snap = compute_metrics_snapshot(&conn, "r1").unwrap();
    assert_eq!(snap.true_positives, 90);
    assert_eq!(snap.false_positives, 3);
    assert_eq!(snap.causal_catches, 70);
    assert_eq!(snap.spurious_catches, 8);
    // CCR = 70 / (70 + 8) = 0.8974...
    let ccr = snap.counterfactual_causal_rate.unwrap();
    assert!((ccr - 70.0 / 78.0).abs() < 0.001);
}

#[test]
fn contract_compute_metrics_zero_rows() {
    let conn = mem_db();
    insert_rule_at(&conn, "r1", "SHADOW", "SEED", true);
    // No metrics rows inserted
    let snap = compute_metrics_snapshot(&conn, "r1").unwrap();
    assert_eq!(snap.true_positives, 0);
    assert!(snap.counterfactual_causal_rate.is_none());
}

// ---------------------------------------------------------------------------
// fetch_rule_record contracts
// ---------------------------------------------------------------------------

#[test]
fn contract_fetch_rule_record_not_found() {
    let conn = mem_db();
    let err = fetch_rule_record(&conn, "nonexistent").unwrap_err();
    assert!(matches!(err, LifecycleError::RuleNotFound { .. }));
}

#[test]
fn contract_fetch_rule_record_fields() {
    let conn = mem_db();
    insert_rule_at(&conn, "r1", "SHADOW", "AUDITOR", false);
    let record = fetch_rule_record(&conn, "r1").unwrap();
    assert_eq!(record.rule_id, "r1");
    assert_eq!(record.domain, "test");
    assert_eq!(record.tier, 1);
    assert!(matches!(record.status, LifecycleStatus::Shadow));
    assert!(!record.mutable);
}
