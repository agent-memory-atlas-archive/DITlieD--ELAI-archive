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
pub const SCHEMA_VERSION: i64 = 5;

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
    pattern_id       TEXT PRIMARY KEY,
    pattern_hash     BLOB UNIQUE NOT NULL,
    occurrences      INTEGER NOT NULL DEFAULT 1,
    first_seen       INTEGER NOT NULL,
    last_seen        INTEGER NOT NULL,
    proposed_rule    TEXT,
    domain           TEXT,
    example_code     TEXT NOT NULL,
    replacement_name TEXT,
    score_clarity    INTEGER CHECK(score_clarity BETWEEN 0 AND 5),
    score_frequency  INTEGER CHECK(score_frequency BETWEEN 0 AND 5),
    score_precision  INTEGER CHECK(score_precision BETWEEN 0 AND 5)
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

const CREATE_DEPRECATED_PATTERNS_TABLE: &str = "\
CREATE TABLE IF NOT EXISTS deprecated_patterns (
    pattern_id       TEXT PRIMARY KEY,
    ecosystem        TEXT NOT NULL CHECK(ecosystem IN (
        'cargo', 'pip', 'npm', 'go', 'maven', 'nuget', 'bundler', 'composer'
    )),
    deprecated_name  TEXT NOT NULL,
    replacement_name TEXT NOT NULL,
    deprecated_since TEXT,
    severity         TEXT NOT NULL CHECK(severity IN ('BLOCK', 'WARN')),
    reason           TEXT
)";

const CREATE_INTERROGATION_TREES_TABLE: &str = "\
CREATE TABLE IF NOT EXISTS interrogation_trees (
    tree_id      TEXT PRIMARY KEY,
    language     TEXT NOT NULL,
    domain       TEXT,
    description  TEXT NOT NULL,
    version      INTEGER NOT NULL,
    tree_json    TEXT NOT NULL
)";

const CREATE_INTERROGATION_TREES_INDEX: &str = "\
CREATE INDEX IF NOT EXISTS idx_interrogation_trees_lang_domain \
ON interrogation_trees (language, domain)";

const CREATE_FAILURE_CATEGORY_PROPOSALS_TABLE: &str = "\
CREATE TABLE IF NOT EXISTS failure_category_proposals (
    proposal_id             TEXT PRIMARY KEY,
    suggested_name          TEXT NOT NULL,
    description             TEXT NOT NULL,
    evidence_pattern_ids    TEXT NOT NULL,
    suggested_predicate_type TEXT NOT NULL CHECK(suggested_predicate_type IN (
        'GREP_PATTERN', 'GRAPH_REACHABILITY'
    )),
    fix_template            TEXT NOT NULL,
    closest_existing        TEXT NOT NULL,
    confidence              INTEGER NOT NULL CHECK(confidence BETWEEN 0 AND 100),
    created_at              TEXT NOT NULL,
    domain                  TEXT
)";

// ---------------------------------------------------------------------------
// Seed data for deprecated_patterns (from outdated-tech-audit.md)
// ---------------------------------------------------------------------------

/// Seed entries: (pattern_id, ecosystem, deprecated_name, replacement_name,
///                deprecated_since, severity, reason)
const DEPRECATED_PATTERNS_SEED: &[(&str, &str, &str, &str, Option<&str>, &str, &str)] = &[
    // Rust / cargo — BLOCK
    ("dp-cargo-blake2", "cargo", "blake2", "blake3",
     Some("2024-01"), "BLOCK", "blake3 is 3-5x faster; blake2 inconsistent with project standard"),
    ("dp-cargo-rand08", "cargo", "rand 0.8", "rand >= 0.9",
     Some("2024-06"), "BLOCK", "rand 0.8 is two major versions behind"),
    ("dp-cargo-log", "cargo", "log", "tracing",
     Some("2023-01"), "BLOCK", "tracing is the modern Rust async logging standard"),
    ("dp-cargo-getrandom02", "cargo", "getrandom 0.2", "getrandom >= 0.3",
     Some("2024-01"), "BLOCK", "getrandom 0.2 is outdated"),
    ("dp-cargo-ort-rc", "cargo", "ort 2.0.0-rc", "ort 2",
     Some("2024-06"), "BLOCK", "pre-release ort; stable 2.0 is released"),
    ("dp-cargo-box-dyn-error", "cargo", "Box<dyn Error>", "thiserror/anyhow",
     None, "BLOCK", "erases error context; project uses thiserror/anyhow"),
    // Rust / cargo — WARN
    ("dp-cargo-async-trait", "cargo", "async-trait", "native async fn in traits",
     Some("2023-12"), "WARN", "Rust 1.75+ supports native async fn in traits"),
    ("dp-cargo-r2d2", "cargo", "r2d2", "deadpool/bb8",
     Some("2020-01"), "WARN", "r2d2 last updated 2020; sync pool in async codebase"),
    ("dp-cargo-glob03", "cargo", "glob 0.3", "globset/ignore",
     Some("2016-01"), "WARN", "glob 0.3 is a 2016-era crate; globset is more capable"),
    // Python / pip — BLOCK
    ("dp-pip-http-server", "pip", "http.server", "fastapi+uvicorn",
     None, "BLOCK", "single-threaded, blocking; not recommended for production"),
    ("dp-pip-manual-token-loop", "pip", "manual token loop", "model.generate()+LogitsProcessor",
     None, "BLOCK", "10-50x slower than model.generate() C++/CUDA path"),
    // Python / pip — WARN
    ("dp-pip-torch-no-grad", "pip", "torch.no_grad", "torch.inference_mode",
     Some("2022-01"), "WARN", "inference_mode() is faster, skips autograd metadata"),
    ("dp-pip-json", "pip", "json", "orjson",
     None, "WARN", "orjson is 3-10x faster for hot paths"),
    ("dp-pip-dtype", "pip", "dtype=", "torch_dtype=",
     None, "BLOCK", "dtype= is silently ignored by from_pretrained; use torch_dtype="),
    ("dp-pip-float16-compute", "pip", "float16 compute", "bfloat16 compute",
     None, "WARN", "bfloat16 is preferred compute dtype for NF4 on Ampere+ GPUs"),
];

/// Return BLOCK-severity entries from the static seed data.
///
/// Used by CLI bootstrap to build the CONSTRAINTS block without
/// requiring a database connection. Returns (deprecated_name, replacement_name)
/// pairs, sorted by deprecated_name.
pub fn block_entries_from_seed() -> Vec<DeprecatedPatternEntry> {
    let mut entries: Vec<DeprecatedPatternEntry> = DEPRECATED_PATTERNS_SEED
        .iter()
        .filter(|&&(_, _, _, _, _, sev, _)| sev == "BLOCK")
        .map(|&(_, _, dep, repl, _, sev, _)| DeprecatedPatternEntry {
            deprecated_name: dep.to_string(),
            replacement_name: repl.to_string(),
            severity: sev.to_string(),
        })
        .collect();
    entries.sort_by(|a, b| a.deprecated_name.cmp(&b.deprecated_name));
    entries
}

/// Insert seed deprecated patterns into the database.
///
/// Uses INSERT OR IGNORE to be idempotent — running multiple times does not
/// duplicate rows. Called during schema migration v2 -> v3.
pub fn seed_deprecated_patterns(conn: &Connection) -> Result<(), RulesDbError> {
    let mut stmt = conn.prepare(
        "INSERT OR IGNORE INTO deprecated_patterns \
         (pattern_id, ecosystem, deprecated_name, replacement_name, \
          deprecated_since, severity, reason) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    )?;

    for &(id, eco, dep, repl, since, sev, reason) in DEPRECATED_PATTERNS_SEED {
        stmt.execute(rusqlite::params![id, eco, dep, repl, since, sev, reason])?;
    }

    Ok(())
}

/// A single deprecated pattern entry returned by [`query_block_patterns`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeprecatedPatternEntry {
    pub deprecated_name: String,
    pub replacement_name: String,
    pub severity: String,
}

/// Query deprecated patterns from the database, ordered by severity (BLOCK first).
///
/// If `severity_filter` is `Some`, only patterns with that severity are returned.
/// Used by constitutional injection to build the CONSTRAINTS block
/// (typically filtered to severity = "BLOCK").
pub fn query_deprecated_patterns(
    conn: &Connection,
    severity_filter: Option<&str>,
) -> Result<Vec<DeprecatedPatternEntry>, RulesDbError> {
    let (sql, params): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = match severity_filter {
        Some(sev) => (
            "SELECT deprecated_name, replacement_name, severity \
             FROM deprecated_patterns \
             WHERE severity = ?1 \
             ORDER BY CASE severity WHEN 'BLOCK' THEN 0 WHEN 'WARN' THEN 1 ELSE 2 END, \
                      deprecated_name ASC",
            vec![Box::new(sev.to_string())],
        ),
        None => (
            "SELECT deprecated_name, replacement_name, severity \
             FROM deprecated_patterns \
             ORDER BY CASE severity WHEN 'BLOCK' THEN 0 WHEN 'WARN' THEN 1 ELSE 2 END, \
                      deprecated_name ASC",
            vec![],
        ),
    };

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
        Ok(DeprecatedPatternEntry {
            deprecated_name: row.get(0)?,
            replacement_name: row.get(1)?,
            severity: row.get(2)?,
        })
    })?;

    let mut entries = Vec::new();
    for row in rows {
        entries.push(row?);
    }
    Ok(entries)
}

/// Upsert a single entry into the deprecated_patterns table.
///
/// Uses INSERT OR REPLACE so that re-running the audit overwrites stale rows
/// for the same (pattern_id). Called by the `dep-audit` CLI subcommand.
pub fn upsert_deprecated_pattern(
    conn: &Connection,
    pattern_id: &str,
    ecosystem: &str,
    deprecated_name: &str,
    replacement_name: &str,
    deprecated_since: Option<&str>,
    severity: &str,
    reason: &str,
) -> Result<(), RulesDbError> {
    conn.execute(
        "INSERT OR REPLACE INTO deprecated_patterns \
         (pattern_id, ecosystem, deprecated_name, replacement_name, \
          deprecated_since, severity, reason) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            pattern_id,
            ecosystem,
            deprecated_name,
            replacement_name,
            deprecated_since,
            severity,
            reason,
        ],
    )?;
    Ok(())
}

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

    // Version 2 → 3: add deprecated_patterns table + seed data,
    // extend violation_patterns with replacement_name for VERSION_GATE auto-proposal.
    if current < 3 {
        conn.execute_batch(CREATE_DEPRECATED_PATTERNS_TABLE)?;
        seed_deprecated_patterns(conn)?;
        // Add replacement_name to violation_patterns for VERSION_GATE auto-proposal.
        // Column already exists in fresh v1+ schemas (DDL includes it), so ignore
        // "duplicate column name" errors from ALTER TABLE on upgraded DBs.
        let _ = conn.execute_batch(
            "ALTER TABLE violation_patterns ADD COLUMN replacement_name TEXT",
        );

        let now_epoch = clock
            .system_now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        conn.execute(
            "INSERT INTO schema_version (version, applied_at) VALUES (?1, ?2)",
            rusqlite::params![3_i64, now_epoch],
        )?;
    }

    // Version 3 → 4: add interrogation_trees table + seed data.
    if current < 4 {
        conn.execute_batch(CREATE_INTERROGATION_TREES_TABLE)?;
        conn.execute_batch(CREATE_INTERROGATION_TREES_INDEX)?;
        crate::interrogation_store::seed_interrogation_trees(conn)?;

        let now_epoch = clock
            .system_now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        conn.execute(
            "INSERT INTO schema_version (version, applied_at) VALUES (?1, ?2)",
            rusqlite::params![4_i64, now_epoch],
        )?;
    }

    // Version 4 → 5: add failure_category_proposals table for taxonomy self-discovery.
    if current < 5 {
        conn.execute_batch(CREATE_FAILURE_CATEGORY_PROPOSALS_TABLE)?;

        let now_epoch = clock
            .system_now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        conn.execute(
            "INSERT INTO schema_version (version, applied_at) VALUES (?1, ?2)",
            rusqlite::params![5_i64, now_epoch],
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
    fn all_tables_exist() {
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
        assert!(tables.contains(&"deprecated_patterns".to_string()));
        assert!(tables.contains(&"interrogation_trees".to_string()));
        assert!(tables.contains(&"failure_category_proposals".to_string()));
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
    fn deprecated_patterns_table_exists() {
        let conn = mem_db();
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert!(tables.contains(&"deprecated_patterns".to_string()));
    }

    #[test]
    fn deprecated_patterns_seed_data_loaded() {
        let conn = mem_db();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM deprecated_patterns", [], |row| row.get(0))
            .unwrap();
        assert!(count >= 15, "expected at least 15 seed patterns, got {}", count);
    }

    #[test]
    fn deprecated_patterns_ecosystem_check_rejects_invalid() {
        let conn = mem_db();
        let result = conn.execute(
            "INSERT INTO deprecated_patterns (pattern_id, ecosystem, deprecated_name, \
             replacement_name, severity) \
             VALUES ('test', 'homebrew', 'foo', 'bar', 'BLOCK')",
            [],
        );
        assert!(result.is_err(), "ecosystem='homebrew' must violate CHECK constraint");
    }

    #[test]
    fn deprecated_patterns_severity_check_rejects_invalid() {
        let conn = mem_db();
        let result = conn.execute(
            "INSERT INTO deprecated_patterns (pattern_id, ecosystem, deprecated_name, \
             replacement_name, severity) \
             VALUES ('test', 'cargo', 'foo', 'bar', 'FATAL')",
            [],
        );
        assert!(result.is_err(), "severity='FATAL' must violate CHECK constraint");
    }

    #[test]
    fn deprecated_patterns_valid_insert() {
        let conn = mem_db();
        let result = conn.execute(
            "INSERT INTO deprecated_patterns (pattern_id, ecosystem, deprecated_name, \
             replacement_name, severity, reason) \
             VALUES ('dp-test', 'npm', 'moment', 'date-fns', 'WARN', 'moment is unmaintained')",
            [],
        );
        assert!(result.is_ok());
    }

    #[test]
    fn deprecated_patterns_seed_idempotent() {
        let conn = mem_db();
        // Seed already loaded by migration. Run again — must not fail.
        seed_deprecated_patterns(&conn).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM deprecated_patterns", [], |row| row.get(0))
            .unwrap();
        // Count unchanged (INSERT OR IGNORE).
        assert!(count >= 15);
    }

    #[test]
    fn deprecated_patterns_seed_has_expected_entries() {
        let conn = mem_db();
        // Spot-check a few entries.
        let blake2: (String, String, String) = conn
            .query_row(
                "SELECT ecosystem, replacement_name, severity FROM deprecated_patterns \
                 WHERE pattern_id = 'dp-cargo-blake2'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(blake2.0, "cargo");
        assert_eq!(blake2.1, "blake3");
        assert_eq!(blake2.2, "BLOCK");

        let http_srv: (String, String) = conn
            .query_row(
                "SELECT ecosystem, severity FROM deprecated_patterns \
                 WHERE pattern_id = 'dp-pip-http-server'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(http_srv.0, "pip");
        assert_eq!(http_srv.1, "BLOCK");
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

    // -----------------------------------------------------------------------
    // A-7: query_deprecated_patterns + block_entries_from_seed tests
    // -----------------------------------------------------------------------

    #[test]
    fn block_entries_from_seed_returns_only_block_severity() {
        let entries = block_entries_from_seed();
        assert!(!entries.is_empty());
        for entry in &entries {
            assert_eq!(entry.severity, "BLOCK", "non-BLOCK entry found: {}", entry.deprecated_name);
        }
    }

    #[test]
    fn block_entries_from_seed_sorted_by_name() {
        let entries = block_entries_from_seed();
        for window in entries.windows(2) {
            assert!(
                window[0].deprecated_name <= window[1].deprecated_name,
                "not sorted: {} > {}",
                window[0].deprecated_name,
                window[1].deprecated_name
            );
        }
    }

    #[test]
    fn block_entries_from_seed_contains_known_entries() {
        let entries = block_entries_from_seed();
        let names: Vec<&str> = entries.iter().map(|e| e.deprecated_name.as_str()).collect();
        assert!(names.contains(&"blake2"), "missing blake2");
        assert!(names.contains(&"log"), "missing log crate");
        assert!(names.contains(&"http.server"), "missing http.server");
    }

    #[test]
    fn query_deprecated_patterns_block_filter() {
        let conn = mem_db();
        let entries = query_deprecated_patterns(&conn, Some("BLOCK")).unwrap();
        assert!(!entries.is_empty());
        for entry in &entries {
            assert_eq!(entry.severity, "BLOCK");
        }
    }

    #[test]
    fn query_deprecated_patterns_warn_filter() {
        let conn = mem_db();
        let entries = query_deprecated_patterns(&conn, Some("WARN")).unwrap();
        assert!(!entries.is_empty());
        for entry in &entries {
            assert_eq!(entry.severity, "WARN");
        }
    }

    #[test]
    fn query_deprecated_patterns_no_filter_returns_all() {
        let conn = mem_db();
        let all = query_deprecated_patterns(&conn, None).unwrap();
        let blocks = query_deprecated_patterns(&conn, Some("BLOCK")).unwrap();
        let warns = query_deprecated_patterns(&conn, Some("WARN")).unwrap();
        assert_eq!(all.len(), blocks.len() + warns.len());
    }

    #[test]
    fn query_deprecated_patterns_ordered_block_first() {
        let conn = mem_db();
        let all = query_deprecated_patterns(&conn, None).unwrap();
        let mut seen_warn = false;
        for entry in &all {
            if entry.severity == "WARN" {
                seen_warn = true;
            }
            if entry.severity == "BLOCK" && seen_warn {
                panic!("BLOCK entry found after WARN: {}", entry.deprecated_name);
            }
        }
    }
}
