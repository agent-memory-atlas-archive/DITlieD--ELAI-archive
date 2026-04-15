//! Contract tests for A-33 PI Tier 2 completions.
//!
//! Validates: DependencyProfile defaults, dep_profile round-trip via serde,
//! check_and_promote_seeded_rules InsufficientShadowData early-return,
//! check_and_retire_seeded_rules threshold behavior.

use std::path::Path;

use elai_core::clock::SeededClock;
use elai_core::project_profile::{ClassificationDepth, DependencyProfile, PinningStrategy};
use elai_rules::project_rule_seeder::{
    check_and_promote_seeded_rules, check_and_retire_seeded_rules,
};
use elai_rules::schema::open_rules_db;
use rusqlite::params;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn test_clock() -> SeededClock {
    SeededClock::new()
}

fn mem_db() -> rusqlite::Connection {
    open_rules_db(Path::new(":memory:"), &test_clock()).expect("in-memory rules DB")
}

fn insert_seed_rule(conn: &rusqlite::Connection, rule_id: &str) {
    conn.execute(
        "INSERT INTO rules \
         (rule_id, domain, tier, status, title, description, predicate, \
          phase_mask, severity, mutable, created_at, source) \
         VALUES (?1, 'engineering', 2, 'SHADOW', 'test', 'test', \
         '{\"type\":\"grep_pattern\",\"pattern\":\".*\",\"scope\":null,\"context_lines\":0}', \
          24, 'WARN', 1, 1000, 'SEED')",
        params![rule_id],
    )
    .expect("insert seed rule");
}

fn insert_metrics(
    conn: &rusqlite::Connection,
    rule_id: &str,
    tp: i64,
    fp: i64,
    causal: i64,
    spurious: i64,
) {
    conn.execute(
        "INSERT INTO rule_metrics (rule_id, window_start, evaluations, \
         true_positives, false_positives, missed, causal_catches, spurious_catches) \
         VALUES (?1, 1000, ?2, ?3, ?4, 0, ?5, ?6)",
        params![rule_id, tp + fp, tp, fp, causal, spurious],
    )
    .expect("insert metrics");
}

// ---------------------------------------------------------------------------
// Contract: DependencyProfile defaults are DeclarationOnly + Unknown pinning
// ---------------------------------------------------------------------------

#[test]
fn contract_dependency_profile_defaults() {
    let dp = DependencyProfile::default();
    assert_eq!(dp.total, 0);
    assert!(dp.declared.is_empty());
    assert!(dp.dev_only.is_empty());
    assert!(dp.core.is_empty());
    assert!(dp.peripheral.is_empty());
    assert!(dp.unused.is_empty());
    assert_eq!(dp.classification_depth, ClassificationDepth::DeclarationOnly);
    assert_eq!(dp.pinning_strategy, PinningStrategy::Unknown);
}

// ---------------------------------------------------------------------------
// Contract: DependencyProfile serde round-trip
// ---------------------------------------------------------------------------

#[test]
fn contract_dependency_profile_serde_roundtrip() {
    let dp = DependencyProfile {
        total: 3,
        declared: vec!["serde".to_owned(), "tokio".to_owned(), "anyhow".to_owned()],
        dev_only: vec!["tempfile".to_owned()],
        core: vec!["serde".to_owned()],
        peripheral: vec!["anyhow".to_owned()],
        unused: Vec::new(),
        pinning_strategy: PinningStrategy::Ranged,
        classification_depth: ClassificationDepth::UsageBased,
    };
    let json = serde_json::to_string(&dp).expect("serialize");
    let back: DependencyProfile = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.total, 3);
    assert_eq!(back.classification_depth, ClassificationDepth::UsageBased);
    assert_eq!(back.pinning_strategy, PinningStrategy::Ranged);
}

// ---------------------------------------------------------------------------
// Contract: check_and_promote returns Ok on InsufficientShadowData (no metrics)
// ---------------------------------------------------------------------------

#[test]
fn contract_promote_insufficient_shadow_data_returns_ok() {
    let conn = mem_db();
    let clock = test_clock();

    insert_seed_rule(&conn, "proj.test.promote_check");
    // No metrics rows inserted — CCR will be None.

    let result = check_and_promote_seeded_rules(&conn, "proj.test.promote_check", &clock);
    assert!(result.is_ok(), "should return Ok on insufficient shadow data");

    // Rule must still be SHADOW (not promoted).
    let status: String = conn
        .query_row(
            "SELECT status FROM rules WHERE rule_id = ?1",
            params!["proj.test.promote_check"],
            |row| row.get(0),
        )
        .expect("query");
    assert_eq!(status, "SHADOW");
}

// ---------------------------------------------------------------------------
// Contract: check_and_promote with sufficient data promotes to ACTIVE
// ---------------------------------------------------------------------------

#[test]
fn contract_promote_with_sufficient_metrics_promotes() {
    let conn = mem_db();
    let clock = test_clock();

    insert_seed_rule(&conn, "proj.test.promote_good");
    // Insert metrics: 6 TP, 0 FP, causal=6, spurious=0 -> CCR=1.0, FPR=0.0, TPR=1.0
    insert_metrics(&conn, "proj.test.promote_good", 6, 0, 6, 0);

    let result = check_and_promote_seeded_rules(&conn, "proj.test.promote_good", &clock);
    assert!(result.is_ok());

    let status: String = conn
        .query_row(
            "SELECT status FROM rules WHERE rule_id = ?1",
            params!["proj.test.promote_good"],
            |row| row.get(0),
        )
        .expect("query");
    assert_eq!(status, "ACTIVE");
}

// ---------------------------------------------------------------------------
// Contract: check_and_retire with low FPs does nothing
// ---------------------------------------------------------------------------

#[test]
fn contract_retire_below_threshold_no_op() {
    let conn = mem_db();
    let clock = test_clock();

    // Insert as ACTIVE (retire only works from ACTIVE).
    conn.execute(
        "INSERT INTO rules \
         (rule_id, domain, tier, status, title, description, predicate, \
          phase_mask, severity, mutable, created_at, source, promoted_at) \
         VALUES ('proj.test.retire_low', 'engineering', 2, 'ACTIVE', 'test', 'test', \
         '{\"type\":\"grep_pattern\",\"pattern\":\".*\",\"scope\":null,\"context_lines\":0}', \
          24, 'WARN', 1, 1000, 'SEED', 2000)",
        [],
    )
    .expect("insert active rule");
    // 2 FPs, below threshold of 3.
    insert_metrics(&conn, "proj.test.retire_low", 5, 2, 3, 0);

    let result = check_and_retire_seeded_rules(&conn, "proj.test.retire_low", &clock);
    assert!(result.is_ok());

    let status: String = conn
        .query_row(
            "SELECT status FROM rules WHERE rule_id = ?1",
            params!["proj.test.retire_low"],
            |row| row.get(0),
        )
        .expect("query");
    assert_eq!(status, "ACTIVE", "should stay ACTIVE below FP threshold");
}

// ---------------------------------------------------------------------------
// Contract: check_and_retire with high FPs retires
// ---------------------------------------------------------------------------

#[test]
fn contract_retire_above_threshold_retires() {
    let conn = mem_db();
    let clock = test_clock();

    conn.execute(
        "INSERT INTO rules \
         (rule_id, domain, tier, status, title, description, predicate, \
          phase_mask, severity, mutable, created_at, source, promoted_at) \
         VALUES ('proj.test.retire_high', 'engineering', 2, 'ACTIVE', 'test', 'test', \
         '{\"type\":\"grep_pattern\",\"pattern\":\".*\",\"scope\":null,\"context_lines\":0}', \
          24, 'WARN', 1, 1000, 'SEED', 2000)",
        [],
    )
    .expect("insert active rule");
    // 4 FPs, above threshold of 3.
    insert_metrics(&conn, "proj.test.retire_high", 2, 4, 1, 0);

    let result = check_and_retire_seeded_rules(&conn, "proj.test.retire_high", &clock);
    assert!(result.is_ok());

    let status: String = conn
        .query_row(
            "SELECT status FROM rules WHERE rule_id = ?1",
            params!["proj.test.retire_high"],
            |row| row.get(0),
        )
        .expect("query");
    assert_eq!(status, "RETIRED");
}

// ---------------------------------------------------------------------------
// Contract: check_and_promote on nonexistent rule returns Ok (no panic)
// ---------------------------------------------------------------------------

#[test]
fn contract_promote_nonexistent_rule_returns_ok() {
    let conn = mem_db();
    let clock = test_clock();
    let result = check_and_promote_seeded_rules(&conn, "nonexistent.rule", &clock);
    assert!(result.is_ok());
}
