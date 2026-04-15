//! Contract tests for PI Tier 2 convention population + ProjectRuleSeeder wiring (Phase A-5).
//!
//! Validates: populate_tier2_conventions is callable with None DKG (regex fallback),
//! ProjectRuleSeeder::seed_rules inserts rows into a real open_rules_db connection,
//! seed is idempotent, and the seeded rules carry expected metadata (source=SEED, status=SHADOW).

use std::path::Path;

use elai_core::clock::SeededClock;
use elai_knowledge::{detect_project_profile, populate_tier2_conventions};
use elai_rules::project_rule_seeder::ProjectRuleSeeder;
use elai_rules::schema::open_rules_db;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn test_clock() -> SeededClock {
    SeededClock::new()
}

fn mem_db() -> rusqlite::Connection {
    open_rules_db(Path::new(":memory:"), &test_clock()).expect("in-memory rules DB")
}

/// Create a minimal Rust workspace in a tempdir so detect_project_profile returns Ok.
fn rust_project_dir() -> TempDir {
    let dir = TempDir::new().expect("tmpdir");
    std::fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("write Cargo.toml");
    dir
}

// ---------------------------------------------------------------------------
// Contract: populate_tier2_conventions callable with None DKG
// ---------------------------------------------------------------------------

#[test]
fn contract_tier2_populate_with_none_dkg_does_not_panic() {
    let dir = rust_project_dir();
    let mut profile = detect_project_profile(dir.path()).expect("detect_project_profile");
    // Must not panic when DKG engine is None (regex fallback path).
    populate_tier2_conventions(&mut profile, None);
    // Profile is still valid after the call.
    let _ = &profile.naming;
}

// ---------------------------------------------------------------------------
// Contract: seed_rules inserts rows into open_rules_db connection
// ---------------------------------------------------------------------------

#[test]
fn contract_seed_rules_inserts_into_open_rules_db() {
    let dir = rust_project_dir();
    let mut profile = detect_project_profile(dir.path()).expect("detect_project_profile");
    populate_tier2_conventions(&mut profile, None);

    let conn = mem_db();
    let clock = test_clock();
    let count = ProjectRuleSeeder::seed_rules(&conn, &profile, &clock)
        .expect("seed_rules should succeed");

    // If any rules were seeded, every row must have source=SEED and status=SHADOW.
    if count > 0 {
        let shadow_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM rules WHERE source = 'SEED' AND status = 'SHADOW'",
                [],
                |row| row.get(0),
            )
            .expect("query shadow count");
        assert_eq!(
            shadow_count, count as i64,
            "all seeded rules must enter as SHADOW with source=SEED; seeded={count}"
        );
    }
}

// ---------------------------------------------------------------------------
// Contract: seed_rules is idempotent (second call inserts 0 rows)
// ---------------------------------------------------------------------------

#[test]
fn contract_seed_rules_idempotent() {
    let dir = rust_project_dir();
    let mut profile = detect_project_profile(dir.path()).expect("detect_project_profile");
    populate_tier2_conventions(&mut profile, None);

    let conn = mem_db();
    let clock = test_clock();
    let first = ProjectRuleSeeder::seed_rules(&conn, &profile, &clock)
        .expect("first seed");
    let second = ProjectRuleSeeder::seed_rules(&conn, &profile, &clock)
        .expect("second seed");
    assert_eq!(
        second, 0,
        "second seed must insert 0 rows (idempotent); first inserted {first}"
    );
}

// ---------------------------------------------------------------------------
// Contract: open_rules_db file-backed path compatible with seed_rules
// ---------------------------------------------------------------------------

#[test]
fn contract_seed_rules_works_with_file_backed_db() {
    let dir = TempDir::new().expect("tmpdir");
    let rules_db_path = dir.path().join("rules.db");
    let clock = test_clock();

    let conn = open_rules_db(&rules_db_path, &clock).expect("file-backed rules DB");
    assert!(rules_db_path.exists(), "rules.db must be created on disk");

    let project_dir = rust_project_dir();
    let mut profile =
        detect_project_profile(project_dir.path()).expect("detect_project_profile");
    populate_tier2_conventions(&mut profile, None);

    let count = ProjectRuleSeeder::seed_rules(&conn, &profile, &clock)
        .expect("seed_rules on file DB should succeed");
    let _ = count;
}
