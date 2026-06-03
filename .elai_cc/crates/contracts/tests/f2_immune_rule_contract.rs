//! F-2a immune (mutable=0) security rule contract tests.
//!
//! Validates:
//!   1. Seeded security rows all have mutable=0 after seed_security_rules.
//!   2. A per-project disable attempt via transition_to_retired(force=false) is
//!      rejected with ImmutableRuleCannotAutoRetire for a mutable=0 rule.
//!   3. verify_vibesec_seed_integrity passes on a pristine seeded DB.
//!   4. verify_vibesec_seed_integrity fails with DigestMismatch on a tampered predicate.
//!   5. canonical_hex_digest is deterministic across two calls.

use std::path::Path;

use elai_core::clock::SeededClock;
use elai_rules::error::LifecycleError;
use elai_rules::lifecycle::transition_to_retired;
use elai_rules::project_rule_seeder::ProjectRuleSeeder;
use elai_rules::schema::open_rules_db;
use elai_rules::vibesec_seed::{canonical_hex_digest, verify_vibesec_seed_integrity};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn test_clock() -> SeededClock {
    SeededClock::new()
}

fn mem_db() -> rusqlite::Connection {
    open_rules_db(Path::new(":memory:"), &test_clock()).expect("in-memory rules DB must open")
}

/// Seed the 12 security rules and return the connection.
fn seeded_db() -> rusqlite::Connection {
    let conn = mem_db();
    let clock = test_clock();
    ProjectRuleSeeder::seed_security_rules(&conn, &clock)
        .expect("seed_security_rules must succeed");
    conn
}

// ---------------------------------------------------------------------------
// Contract 1: mutable=0 for all seeded security rows
// ---------------------------------------------------------------------------

#[test]
fn contract_seeded_security_rows_are_mutable_0() {
    let conn = seeded_db();
    let mutable_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM rules \
             WHERE domain = 'security' AND source = 'SEED' AND mutable != 0",
            [],
            |row| row.get(0),
        )
        .expect("query mutable count");
    assert_eq!(
        mutable_count, 0,
        "all seeded security rows must have mutable=0; found {mutable_count} mutable rows"
    );

    let total: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM rules WHERE domain = 'security' AND source = 'SEED'",
            [],
            |row| row.get(0),
        )
        .expect("query total security rows");
    assert_eq!(
        total, 20,
        "seed_security_rules must insert exactly 20 rows (12 F-2a + 8 F-2c); got {total}"
    );
}

// ---------------------------------------------------------------------------
// Contract 2: per-project disable attempt on mutable=0 row is rejected
// ---------------------------------------------------------------------------

#[test]
fn contract_mutable0_disable_attempt_rejected() {
    let conn = seeded_db();
    let clock = test_clock();

    // Move one security rule to ACTIVE so the ACTIVE -> RETIRED transition path
    // is reachable (the immune guard fires only after the status check).
    conn.execute(
        "UPDATE rules SET status = 'ACTIVE' WHERE rule_id = 'SEC-SQLI-001'",
        [],
    )
    .expect("set status to ACTIVE");

    let err = transition_to_retired(
        &conn,
        "SEC-SQLI-001",
        "project override: disable rule",
        false, // force=false — must be rejected by the immutable guard
        &clock,
    )
    .expect_err("mutable=0 rule must refuse force=false retire");

    assert!(
        matches!(err, LifecycleError::ImmutableRuleCannotAutoRetire),
        "expected ImmutableRuleCannotAutoRetire for mutable=0 rule; got: {err}"
    );
}

// ---------------------------------------------------------------------------
// Contract 3: verify_vibesec_seed_integrity passes on pristine seeded DB
// ---------------------------------------------------------------------------

#[test]
fn contract_integrity_check_passes_pristine_db() {
    let conn = seeded_db();
    verify_vibesec_seed_integrity(&conn)
        .expect("integrity check must pass on a freshly seeded DB");
}

// ---------------------------------------------------------------------------
// Contract 4: verify_vibesec_seed_integrity fails on tampered predicate
// ---------------------------------------------------------------------------

#[test]
fn contract_integrity_check_fails_on_tampered_predicate() {
    let conn = seeded_db();

    // Tamper with one row's predicate.
    conn.execute(
        "UPDATE rules SET predicate = 'tampered_payload' WHERE rule_id = 'SEC-SQLI-001'",
        [],
    )
    .expect("tamper update");

    let err = verify_vibesec_seed_integrity(&conn)
        .expect_err("integrity check must fail after predicate tampering");

    assert!(
        matches!(err, elai_rules::vibesec_seed::IntegrityError::DigestMismatch { .. }),
        "expected DigestMismatch after tampering; got: {err}"
    );
}

// ---------------------------------------------------------------------------
// Contract 5: canonical_hex_digest is deterministic and 64 chars
// ---------------------------------------------------------------------------

#[test]
fn contract_canonical_digest_is_deterministic_and_64_chars() {
    let d1 = canonical_hex_digest();
    let d2 = canonical_hex_digest();
    assert_eq!(d1, d2, "canonical_hex_digest must be deterministic");
    assert_eq!(d1.len(), 64, "blake3 hex digest must be 64 chars; got {}", d1.len());
    assert!(
        d1.chars().all(|c| c.is_ascii_hexdigit()),
        "canonical_hex_digest must be lowercase hex"
    );
}

// ---------------------------------------------------------------------------
// Contract 6: seed_security_rules is idempotent
// ---------------------------------------------------------------------------

#[test]
fn contract_seed_security_rules_is_idempotent() {
    let conn = mem_db();
    let clock = test_clock();

    let first = ProjectRuleSeeder::seed_security_rules(&conn, &clock)
        .expect("first seed must succeed");
    assert_eq!(first, 20, "first call must insert 20 rows (12 F-2a + 8 F-2c)");

    let second = ProjectRuleSeeder::seed_security_rules(&conn, &clock)
        .expect("second seed must succeed");
    assert_eq!(second, 0, "second call must be a no-op (rows already exist)");
}
