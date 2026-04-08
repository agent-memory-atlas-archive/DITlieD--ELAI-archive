//! Rules-engine SQLite schema: 4 domain tables + versioned migration.
//!
//! Tables: `rules`, `rule_metrics`, `contract_tests`, `violation_patterns`.
//! Schema versioning via a dedicated `schema_version` table (not PRAGMA user_version)
//! for queryability and self-documentation.

use std::path::Path;
use std::time::UNIX_EPOCH;

use rusqlite::Connection;

use elai_core::clock::ClockProvider;

use crate::error::RulesDbError;

// ---------------------------------------------------------------------------
// Schema version — bump on any DDL change
// ---------------------------------------------------------------------------

/// Current schema version. Bump when any CREATE/ALTER statement changes.
pub const SCHEMA_VERSION: i64 = 2;

// ---------------------------------------------------------------------------
// DDL statements
// ---------------------------------------------------------------------------

const CREATE_SCHEMA_VERSION_TABLE: &str = "\
CREATE TABLE IF NOT EXISTS schema_version (
    version     INTEGER NOT NULL UNIQUE,
    applied_at  INTEGER NOT NULL
)";

const CREATE_RULES_TABLE: &str = "\
CREATE TABLE IF NOT EXISTS rules (
    rule_id     TEXT PRIMARY KEY,
    domain      TEXT NOT NULL,
    tier        INTEGER NOT NULL CHECK(tier IN (1, 2, 3)),
    status      TEXT NOT NULL CHECK(status IN (
        'PROPOSED', 'CANDIDATE', 'SHADOW', 'ACTIVE', 'RETIRED'
    )),
    title       TEXT NOT NULL,
    description TEXT NOT NULL,
    predicate   TEXT NOT NULL,
    phase_mask  INTEGER NOT NULL DEFAULT 255,
    severity    TEXT NOT NULL CHECK(severity IN ('BLOCK', 'WARN', 'INFO')),
    mutable     INTEGER NOT NULL DEFAULT 1,
    created_at  INTEGER NOT NULL,
    promoted_at INTEGER,
    version     INTEGER NOT NULL DEFAULT 1,
    source      TEXT NOT NULL CHECK(source IN (
        'SEED', 'AUDITOR', 'HUMAN', 'SELF_MOD'
    ))
)";

const CREATE_RULE_METRICS_TABLE: &str = "\
CREATE TABLE IF NOT EXISTS rule_metrics (
    rule_id             TEXT NOT NULL REFERENCES rules(rule_id),
    window_start        INTEGER NOT NULL,
    evaluations         INTEGER NOT NULL DEFAULT 0,
    true_positives      INTEGER NOT NULL DEFAULT 0,
    false_positives     INTEGER NOT NULL DEFAULT 0,
    missed              INTEGER NOT NULL DEFAULT 0,
    causal_catches      INTEGER NOT NULL DEFAULT 0,
    spurious_catches    INTEGER NOT NULL DEFAULT 0,
    edct_pass           INTEGER NOT NULL DEFAULT 0,
    edct_fail           INTEGER NOT NULL DEFAULT 0,
    last_true_positive_at INTEGER,
    confidence_score    REAL NOT NULL DEFAULT 1.0,
    PRIMARY KEY (rule_id, window_start)
)";

const CREATE_CONTRACT_TESTS_TABLE: &str = "\
CREATE TABLE IF NOT EXISTS contract_tests (
    test_id     TEXT PRIMARY KEY,
    rule_id     TEXT NOT NULL REFERENCES rules(rule_id),
    test_type   TEXT NOT NULL CHECK(test_type IN ('BLOCK', 'ALLOW', 'EDGE')),
    input_code  TEXT NOT NULL,
    expected    TEXT NOT NULL CHECK(expected IN ('BLOCK', 'ALLOW', 'WARN')),
    language    TEXT NOT NULL DEFAULT 'python',
    created_at  INTEGER NOT NULL,
    source      TEXT NOT NULL CHECK(source IN ('SEED', 'GENERATED', 'HUMAN'))
)";

const CREATE_VIOLATION_PATTERNS_TABLE: &str = "\
CREATE TABLE IF NOT EXISTS violation_patterns (
    pattern_id      TEXT PRIMARY KEY,
    pattern_hash    BLOB UNIQUE NOT NULL,
    occurrences     INTEGER NOT NULL DEFAULT 1,
    first_seen      INTEGER NOT NULL,
    last_seen       INTEGER NOT NULL,
    proposed_rule   TEXT,
    domain          TEXT,
    example_code    TEXT NOT NULL,
    score_clarity   INTEGER CHECK(score_clarity BETWEEN 0 AND 5),
    score_frequency INTEGER CHECK(score_frequency BETWEEN 0 AND 5),
    score_precision INTEGER CHECK(score_precision BETWEEN 0 AND 5)
)";

const CREATE_RULE_TRANSITIONS_TABLE: &str = "\
CREATE TABLE IF NOT EXISTS rule_transitions (
    transition_id    TEXT PRIMARY KEY,
    rule_id          TEXT NOT NULL REFERENCES rules(rule_id),
    from_status      TEXT NOT NULL,
    to_status        TEXT NOT NULL,
    reason           TEXT NOT NULL,
    approver_id      TEXT,
    timestamp        INTEGER NOT NULL,
    tp               INTEGER,
    fp               INTEGER,
    causal_catches   INTEGER,
    spurious_catches INTEGER,
    fpr              REAL,
    tpr              REAL,
    ccr              REAL
)";

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Open (or create) a rules database at the given path and run migrations.
///
/// Applies WAL journal mode and foreign-key enforcement on every open.
/// Returns a fully-migrated connection ready for use.
pub fn open_rules_db(path: &Path, clock: &impl ClockProvider) -> Result<Connection, RulesDbError> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;\
         PRAGMA foreign_keys = ON;",
    )?;
    run_migration(&conn, clock)?;
    Ok(conn)
}

/// Read the current schema version from the database.
///
/// Returns `0` if the `schema_version` table does not exist or is empty,
/// meaning no migration has been applied yet.
pub fn schema_version(conn: &Connection) -> Result<i64, RulesDbError> {
    // Check if the table exists first (avoids error on fresh DB).
    let table_exists: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='schema_version'",
        [],
        |row| row.get(0),
    )?;
    if !table_exists {
        return Ok(0);
    }

    let version: Option<i64> = conn.query_row(
        "SELECT MAX(version) FROM schema_version",
        [],
        |row| row.get(0),
    )?;
    Ok(version.unwrap_or(0))
}

/// Run schema migration to bring the database up to [`SCHEMA_VERSION`].
///
/// Idempotent: calling multiple times on an already-migrated database is a no-op.
/// Uses `CREATE TABLE IF NOT EXISTS` so partial failures are recoverable.
///
/// Migration strategy: for version 0 → 1, create all tables from scratch.
/// Future versions will use additive `ALTER TABLE` where possible and
/// drop-and-recreate when CHECK constraints change (SQLite limitation).
pub fn run_migration(conn: &Connection, clock: &impl ClockProvider) -> Result<(), RulesDbError> {
    let current = schema_version(conn)?;

    if current > SCHEMA_VERSION {
        return Err(RulesDbError::SchemaVersion {
            expected: SCHEMA_VERSION,
            found: current,
        });
    }

    if current == SCHEMA_VERSION {
        return Ok(());
    }

    // Version 0 → 1: initial schema creation.
    if current < 1 {
        conn.execute_batch(CREATE_SCHEMA_VERSION_TABLE)?;
        conn.execute_batch(CREATE_RULES_TABLE)?;
        conn.execute_batch(CREATE_RULE_METRICS_TABLE)?;
        conn.execute_batch(CREATE_CONTRACT_TESTS_TABLE)?;
        conn.execute_batch(CREATE_VIOLATION_PATTERNS_TABLE)?;

        let now_epoch = clock
            .system_now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        conn.execute(
            "INSERT INTO schema_version (version, applied_at) VALUES (?1, ?2)",
            rusqlite::params![1_i64, now_epoch],
        )?;
    }

    // Version 1 → 2: add rule_transitions table for lifecycle tracking.
    if current < 2 {
        conn.execute_batch(CREATE_RULE_TRANSITIONS_TABLE)?;

        let now_epoch = clock
            .system_now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        conn.execute(
            "INSERT INTO schema_version (version, applied_at) VALUES (?1, ?2)",
            rusqlite::params![2_i64, now_epoch],
        )?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use elai_core::clock::SeededClock;

    fn test_clock() -> SeededClock {
        SeededClock::new()
    }

    /// Helper: open an in-memory rules DB.
    fn mem_db() -> Connection {
        open_rules_db(Path::new(":memory:"), &test_clock()).expect("in-memory DB should open")
    }

    #[test]
    fn schema_version_after_migration() {
        let conn = mem_db();
        assert_eq!(schema_version(&conn).unwrap(), SCHEMA_VERSION);
    }

    #[test]
    fn migration_is_idempotent() {
        let conn = mem_db();
        // Already migrated by open_rules_db; run again — must not fail.
        run_migration(&conn, &test_clock()).unwrap();
        assert_eq!(schema_version(&conn).unwrap(), SCHEMA_VERSION);
    }

    #[test]
    fn all_five_tables_exist() {
        let conn = mem_db();
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert!(tables.contains(&"rules".to_string()));
        assert!(tables.contains(&"rule_metrics".to_string()));
        assert!(tables.contains(&"contract_tests".to_string()));
        assert!(tables.contains(&"violation_patterns".to_string()));
        assert!(tables.contains(&"rule_transitions".to_string()));
        assert!(tables.contains(&"schema_version".to_string()));
    }

    #[test]
    fn check_tier_rejects_invalid() {
        let conn = mem_db();
        let result = conn.execute(
            "INSERT INTO rules (rule_id, domain, tier, status, title, description, \
             predicate, severity, created_at, source) \
             VALUES ('t1', 'test', 4, 'ACTIVE', 't', 'd', 'p', 'WARN', 0, 'SEED')",
            [],
        );
        assert!(result.is_err(), "tier=4 must violate CHECK constraint");
    }

    #[test]
    fn check_status_rejects_invalid() {
        let conn = mem_db();
        let result = conn.execute(
            "INSERT INTO rules (rule_id, domain, tier, status, title, description, \
             predicate, severity, created_at, source) \
             VALUES ('t1', 'test', 1, 'BOGUS', 't', 'd', 'p', 'WARN', 0, 'SEED')",
            [],
        );
        assert!(result.is_err(), "status='BOGUS' must violate CHECK constraint");
    }

    #[test]
    fn check_source_rejects_invalid() {
        let conn = mem_db();
        let result = conn.execute(
            "INSERT INTO rules (rule_id, domain, tier, status, title, description, \
             predicate, severity, created_at, source) \
             VALUES ('t1', 'test', 1, 'ACTIVE', 't', 'd', 'p', 'WARN', 0, 'INVALID')",
            [],
        );
        assert!(result.is_err(), "source='INVALID' must violate CHECK constraint");
    }

    #[test]
    fn null_predicate_rejected() {
        let conn = mem_db();
        let result = conn.execute(
            "INSERT INTO rules (rule_id, domain, tier, status, title, description, \
             predicate, severity, created_at, source) \
             VALUES ('t1', 'test', 1, 'ACTIVE', 't', 'd', NULL, 'WARN', 0, 'SEED')",
            [],
        );
        assert!(result.is_err(), "NULL predicate must be rejected (NOT NULL)");
    }

    #[test]
    fn check_severity_rejects_invalid() {
        let conn = mem_db();
        let result = conn.execute(
            "INSERT INTO rules (rule_id, domain, tier, status, title, description, \
             predicate, severity, created_at, source) \
             VALUES ('t1', 'test', 1, 'ACTIVE', 't', 'd', 'p', 'FATAL', 0, 'SEED')",
            [],
        );
        assert!(result.is_err(), "severity='FATAL' must violate CHECK constraint");
    }

    #[test]
    fn valid_rule_insert_succeeds() {
        let conn = mem_db();
        let result = conn.execute(
            "INSERT INTO rules (rule_id, domain, tier, status, title, description, \
             predicate, severity, created_at, source) \
             VALUES ('r1', 'code_quality', 2, 'PROPOSED', 'Test Rule', 'A test', \
             'x > 0', 'WARN', 1000, 'SEED')",
            [],
        );
        assert!(result.is_ok(), "valid INSERT must succeed");
    }

    #[test]
    fn rule_metrics_fk_enforced() {
        let conn = mem_db();
        // Insert metrics for non-existent rule — must fail with FK enforcement.
        let result = conn.execute(
            "INSERT INTO rule_metrics (rule_id, window_start) \
             VALUES ('nonexistent', 1000)",
            [],
        );
        assert!(result.is_err(), "FK constraint on rule_metrics.rule_id must be enforced");
    }

    #[test]
    fn contract_tests_fk_enforced() {
        let conn = mem_db();
        let result = conn.execute(
            "INSERT INTO contract_tests (test_id, rule_id, test_type, input_code, \
             expected, created_at, source) \
             VALUES ('ct1', 'nonexistent', 'BLOCK', 'code', 'BLOCK', 0, 'SEED')",
            [],
        );
        assert!(result.is_err(), "FK constraint on contract_tests.rule_id must be enforced");
    }

    #[test]
    fn violation_patterns_unique_hash() {
        let conn = mem_db();
        let hash = b"deadbeef";
        conn.execute(
            "INSERT INTO violation_patterns (pattern_id, pattern_hash, first_seen, \
             last_seen, domain, example_code) \
             VALUES ('vp1', ?1, 0, 0, 'test', 'code')",
            rusqlite::params![hash.as_slice()],
        )
        .unwrap();
        let result = conn.execute(
            "INSERT INTO violation_patterns (pattern_id, pattern_hash, first_seen, \
             last_seen, domain, example_code) \
             VALUES ('vp2', ?1, 0, 0, 'test', 'code2')",
            rusqlite::params![hash.as_slice()],
        );
        assert!(result.is_err(), "duplicate pattern_hash must violate UNIQUE constraint");
    }

    #[test]
    fn violation_patterns_score_check_constraints() {
        let conn = mem_db();
        let hash = b"abc123";
        let result = conn.execute(
            "INSERT INTO violation_patterns (pattern_id, pattern_hash, first_seen, \
             last_seen, domain, example_code, score_clarity) \
             VALUES ('vp1', ?1, 0, 0, 'test', 'code', 6)",
            rusqlite::params![hash.as_slice()],
        );
        assert!(result.is_err(), "score_clarity=6 must violate CHECK(0..5)");
    }

    #[test]
    fn contract_tests_test_type_check() {
        let conn = mem_db();
        // Insert a rule first for FK.
        conn.execute(
            "INSERT INTO rules (rule_id, domain, tier, status, title, description, \
             predicate, severity, created_at, source) \
             VALUES ('r1', 'test', 1, 'ACTIVE', 't', 'd', 'p', 'WARN', 0, 'SEED')",
            [],
        )
        .unwrap();
        let result = conn.execute(
            "INSERT INTO contract_tests (test_id, rule_id, test_type, input_code, \
             expected, created_at, source) \
             VALUES ('ct1', 'r1', 'INVALID', 'code', 'BLOCK', 0, 'SEED')",
            [],
        );
        assert!(result.is_err(), "test_type='INVALID' must violate CHECK constraint");
    }

    #[test]
    fn future_schema_version_rejected() {
        let conn = mem_db();
        // Insert a future version row beyond current.
        conn.execute(
            "INSERT INTO schema_version (version, applied_at) VALUES (?1, 0)",
            rusqlite::params![SCHEMA_VERSION + 1],
        )
        .unwrap();
        let result = run_migration(&conn, &test_clock());
        assert!(result.is_err(), "schema version from the future must be rejected");
    }

    #[test]
    fn rule_metrics_counterfactual_columns_present() {
        let conn = mem_db();
        // Insert a rule then a metrics row with counterfactual columns.
        conn.execute(
            "INSERT INTO rules (rule_id, domain, tier, status, title, description, \
             predicate, severity, created_at, source) \
             VALUES ('r1', 'test', 1, 'ACTIVE', 't', 'd', 'p', 'WARN', 0, 'SEED')",
            [],
        )
        .unwrap();
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
}
