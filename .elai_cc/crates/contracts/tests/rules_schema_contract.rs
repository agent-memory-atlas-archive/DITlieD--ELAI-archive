//! Contract tests for rules/ crate SQLite schema (Phase 3F.1).
//!
//! Validates: table existence, CHECK constraints, FK enforcement,
//! migration idempotency, schema versioning, and counterfactual columns.

use std::path::Path;

use elai_core::clock::SeededClock;
use elai_rules::schema::{open_rules_db, run_migration, schema_version, SCHEMA_VERSION};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn test_clock() -> SeededClock {
    SeededClock::new()
}

fn mem_db() -> rusqlite::Connection {
    open_rules_db(Path::new(":memory:"), &test_clock()).expect("in-memory DB should open")
}

fn table_names(conn: &rusqlite::Connection) -> Vec<String> {
    conn.prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
}

// ---------------------------------------------------------------------------
// Table existence
// ---------------------------------------------------------------------------

#[test]
fn contract_all_tables_created() {
    let conn = mem_db();
    let tables = table_names(&conn);
    for expected in &[
        "rules",
        "rule_metrics",
        "contract_tests",
        "violation_patterns",
        "rule_transitions",
        "schema_version",
    ] {
        assert!(
            tables.contains(&expected.to_string()),
            "missing table: {expected}"
        );
    }
}

// ---------------------------------------------------------------------------
// Schema version
// ---------------------------------------------------------------------------

#[test]
fn contract_schema_version_matches() {
    let conn = mem_db();
    assert_eq!(schema_version(&conn).unwrap(), SCHEMA_VERSION);
}

// ---------------------------------------------------------------------------
// Migration idempotency
// ---------------------------------------------------------------------------

#[test]
fn contract_migration_idempotent() {
    let conn = mem_db();
    run_migration(&conn, &test_clock()).unwrap();
    run_migration(&conn, &test_clock()).unwrap();
    assert_eq!(schema_version(&conn).unwrap(), SCHEMA_VERSION);
    // schema_version table must have exactly SCHEMA_VERSION rows (one per version applied).
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM schema_version", [], |row| row.get(0))
        .unwrap();
    assert_eq!(
        count, SCHEMA_VERSION,
        "idempotent migration must not duplicate version rows"
    );
}

// ---------------------------------------------------------------------------
// CHECK constraints on rules table
// ---------------------------------------------------------------------------

fn insert_rule(conn: &rusqlite::Connection, overrides: &str) -> rusqlite::Result<usize> {
    let sql = format!(
        "INSERT INTO rules (rule_id, domain, tier, status, title, description, \
         predicate, severity, created_at, source) VALUES {overrides}"
    );
    conn.execute(&sql, [])
}

#[test]
fn contract_tier_check_rejects_4() {
    let conn = mem_db();
    let r = insert_rule(&conn, "('t1','d',4,'ACTIVE','t','d','p','WARN',0,'SEED')");
    assert!(r.is_err(), "tier=4 must fail CHECK");
}

#[test]
fn contract_status_check_rejects_bogus() {
    let conn = mem_db();
    let r = insert_rule(&conn, "('t1','d',1,'BOGUS','t','d','p','WARN',0,'SEED')");
    assert!(r.is_err(), "status='BOGUS' must fail CHECK");
}

#[test]
fn contract_source_check_rejects_invalid() {
    let conn = mem_db();
    let r = insert_rule(&conn, "('t1','d',1,'ACTIVE','t','d','p','WARN',0,'INVALID')");
    assert!(r.is_err(), "source='INVALID' must fail CHECK");
}

#[test]
fn contract_severity_check_rejects_fatal() {
    let conn = mem_db();
    let r = insert_rule(&conn, "('t1','d',1,'ACTIVE','t','d','p','FATAL',0,'SEED')");
    assert!(r.is_err(), "severity='FATAL' must fail CHECK");
}

#[test]
fn contract_null_predicate_rejected() {
    let conn = mem_db();
    let r = conn.execute(
        "INSERT INTO rules (rule_id, domain, tier, status, title, description, \
         predicate, severity, created_at, source) \
         VALUES ('t1','d',1,'ACTIVE','t','d',NULL,'WARN',0,'SEED')",
        [],
    );
    assert!(r.is_err(), "NULL predicate must be rejected (NOT NULL)");
}

// ---------------------------------------------------------------------------
// FK enforcement
// ---------------------------------------------------------------------------

#[test]
fn contract_rule_metrics_fk_enforced() {
    let conn = mem_db();
    let r = conn.execute(
        "INSERT INTO rule_metrics (rule_id, window_start) VALUES ('ghost', 1000)",
        [],
    );
    assert!(r.is_err(), "FK on rule_metrics.rule_id must reject missing parent");
}

#[test]
fn contract_contract_tests_fk_enforced() {
    let conn = mem_db();
    let r = conn.execute(
        "INSERT INTO contract_tests (test_id, rule_id, test_type, input_code, \
         expected, created_at, source) \
         VALUES ('ct1','ghost','BLOCK','code','BLOCK',0,'SEED')",
        [],
    );
    assert!(r.is_err(), "FK on contract_tests.rule_id must reject missing parent");
}

// ---------------------------------------------------------------------------
// Counterfactual columns in rule_metrics
// ---------------------------------------------------------------------------

#[test]
fn contract_rule_metrics_counterfactual_columns() {
    let conn = mem_db();
    insert_rule(&conn, "('r1','d',1,'ACTIVE','t','d','p','WARN',0,'SEED')").unwrap();
    conn.execute(
        "INSERT INTO rule_metrics (rule_id, window_start, causal_catches, \
         spurious_catches, edct_pass, edct_fail, confidence_score) \
         VALUES ('r1', 1000, 5, 2, 10, 1, 0.85)",
        [],
    )
    .unwrap();
    let (causal, spurious, edct_p, edct_f, conf): (i64, i64, i64, i64, f64) = conn
        .query_row(
            "SELECT causal_catches, spurious_catches, edct_pass, edct_fail, \
             confidence_score FROM rule_metrics WHERE rule_id = 'r1'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
        )
        .unwrap();
    assert_eq!(causal, 5);
    assert_eq!(spurious, 2);
    assert_eq!(edct_p, 10);
    assert_eq!(edct_f, 1);
    assert!((conf - 0.85).abs() < f64::EPSILON);
}

// ---------------------------------------------------------------------------
// violation_patterns constraints
// ---------------------------------------------------------------------------

#[test]
fn contract_violation_patterns_unique_hash() {
    let conn = mem_db();
    let hash = b"deadbeef";
    conn.execute(
        "INSERT INTO violation_patterns (pattern_id, pattern_hash, first_seen, \
         last_seen, domain, example_code) VALUES ('vp1',?1,0,0,'d','c')",
        rusqlite::params![hash.as_slice()],
    )
    .unwrap();
    let r = conn.execute(
        "INSERT INTO violation_patterns (pattern_id, pattern_hash, first_seen, \
         last_seen, domain, example_code) VALUES ('vp2',?1,0,0,'d','c2')",
        rusqlite::params![hash.as_slice()],
    );
    assert!(r.is_err(), "duplicate pattern_hash must violate UNIQUE");
}

#[test]
fn contract_violation_patterns_score_check() {
    let conn = mem_db();
    let hash = b"abc123";
    let r = conn.execute(
        "INSERT INTO violation_patterns (pattern_id, pattern_hash, first_seen, \
         last_seen, domain, example_code, score_clarity) VALUES ('vp1',?1,0,0,'d','c',6)",
        rusqlite::params![hash.as_slice()],
    );
    assert!(r.is_err(), "score_clarity=6 must violate CHECK(0..5)");
}

// ---------------------------------------------------------------------------
// contract_tests table constraints
// ---------------------------------------------------------------------------

#[test]
fn contract_test_type_check_rejects_invalid() {
    let conn = mem_db();
    insert_rule(&conn, "('r1','d',1,'ACTIVE','t','d','p','WARN',0,'SEED')").unwrap();
    let r = conn.execute(
        "INSERT INTO contract_tests (test_id, rule_id, test_type, input_code, \
         expected, created_at, source) VALUES ('ct1','r1','INVALID','code','BLOCK',0,'SEED')",
        [],
    );
    assert!(r.is_err(), "test_type='INVALID' must fail CHECK");
}

#[test]
fn contract_test_expected_check_rejects_invalid() {
    let conn = mem_db();
    insert_rule(&conn, "('r1','d',1,'ACTIVE','t','d','p','WARN',0,'SEED')").unwrap();
    let r = conn.execute(
        "INSERT INTO contract_tests (test_id, rule_id, test_type, input_code, \
         expected, created_at, source) VALUES ('ct1','r1','BLOCK','code','PASS',0,'SEED')",
        [],
    );
    assert!(r.is_err(), "expected='PASS' must fail CHECK");
}

// ---------------------------------------------------------------------------
// File-based open (tempfile)
// ---------------------------------------------------------------------------

#[test]
fn contract_file_based_open_and_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("rules.db");
    {
        let conn = open_rules_db(&path, &test_clock()).unwrap();
        insert_rule(&conn, "('r1','d',1,'ACTIVE','t','d','p','WARN',0,'SEED')").unwrap();
    }
    // Reopen — migration must be idempotent, data must persist.
    let conn = open_rules_db(&path, &test_clock()).unwrap();
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM rules", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 1, "data must persist across reopen");
    assert_eq!(schema_version(&conn).unwrap(), SCHEMA_VERSION);
}
