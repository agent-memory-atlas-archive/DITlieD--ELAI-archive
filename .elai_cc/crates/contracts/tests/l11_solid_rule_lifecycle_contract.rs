// L-11: SOLID proxy seeding and SHADOW promotion lifecycle contract tests.
//
// ARM-1 covers the three fixed SOLID proxy rows and their SHADOW/severity metadata.
// ARM-2 covers one SHADOW row promoted by passing metric windows after one tick.
// ARM-3 covers fail-closed retention of an unobserved SHADOW row.

use std::path::Path;
use std::sync::Arc;

use elai_core::clock::{SeededClock, ClockProvider};
use elai_knowledge::detect_project_profile;
use elai_orchestrator::{rule_curator::PlaybookCurator, ssb::SsbConfig, Ssb};
use elai_rules::{project_rule_seeder::ProjectRuleSeeder, schema::open_rules_db};
use tempfile::TempDir;

fn test_clock() -> SeededClock { SeededClock::new() }

fn rust_project_dir() -> TempDir {
    let dir = TempDir::new().expect("tmpdir");
    std::fs::write(dir.path().join("Cargo.toml"),
        "[package]\nname = \"solid-contract\"\nversion = \"0.1.0\"\nedition = \"2021\"\n")
        .expect("write Cargo.toml");
    dir
}

fn make_ssb(tmp: &TempDir) -> Arc<Ssb> {
    let cfg = SsbConfig {
        db_path: tmp.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: "l11-test".into(),
        session_id: "l11-solid-session".into(),
    };
    let clock: Arc<dyn ClockProvider> = Arc::new(test_clock());
    Arc::new(Ssb::new(cfg, clock).expect("ssb init"))
}

fn insert_shadow_rule(conn: &rusqlite::Connection, rule_id: &str) {
    conn.execute(
        "INSERT INTO rules (rule_id, domain, tier, status, title, description, predicate, severity, mutable, created_at, source)\
         VALUES (?1, 'engineering', 2, 'SHADOW', 'Solid proxy', 'contract row',\
                 '{\"type\":\"metric_gate\"}', 'WARN', 1, 1000, 'SEED')",
        rusqlite::params![rule_id],
    ).expect("insert shadow rule");
}

fn insert_passing_metrics(conn: &rusqlite::Connection, rule_id: &str) {
    conn.execute(
        "INSERT INTO rule_metrics (rule_id, window_start, evaluations, true_positives, false_positives, missed, causal_catches, spurious_catches)\
         VALUES (?1, 0, 104, 100, 2, 0, 85, 2)",
        rusqlite::params![rule_id],
    ).expect("insert passing metrics");
}

fn file_db(tmp: &TempDir, rule_id: &str, with_metrics: bool) -> std::path::PathBuf {
    let path = tmp.path().join("rules.db");
    let conn = open_rules_db(&path, &test_clock()).expect("rules db");
    insert_shadow_rule(&conn, rule_id);
    if with_metrics { insert_passing_metrics(&conn, rule_id); }
    drop(conn);
    path
}

#[test]
fn contract_seed_rules_inserts_three_solid_rows_at_shadow() {
    let project = rust_project_dir();
    let profile = detect_project_profile(project.path()).expect("detect profile");
    let conn = open_rules_db(Path::new(":memory:"), &test_clock()).expect("memory db");
    ProjectRuleSeeder::seed_rules(&conn, &profile, &test_clock()).expect("seed rules");
    let expected = [
        ("C-MAINTAINABILITY-SRP", "BLOCK"),
        ("C-MAINTAINABILITY-DIP", "WARN"),
        ("C-MAINTAINABILITY-ISP", "WARN"),
    ];
    for (rule_id, severity) in expected {
        let row: (String, String) = conn.query_row(
            "SELECT status, severity FROM rules WHERE rule_id = ?1", [rule_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).expect("solid proxy row");
        assert_eq!(row, ("SHADOW".into(), severity.into()));
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_shadow_row_with_passing_metrics_promotes_after_one_tick() {
    let tmp = TempDir::new().expect("tempdir");
    let rule_id = "solid-shadow-passing";
    let path = file_db(&tmp, rule_id, true);
    let curator = PlaybookCurator::new(&path, Arc::new(test_clock()));
    curator.run_promotion_tick(&make_ssb(&tmp)).await.expect("tick");
    let conn = open_rules_db(&path, &test_clock()).expect("reopen db");
    let status: String = conn.query_row("SELECT status FROM rules WHERE rule_id = ?1", [rule_id], |r| r.get(0)).expect("status");
    assert_eq!(status, "ACTIVE");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_shadow_row_without_metrics_stays_shadow() {
    let tmp = TempDir::new().expect("tempdir");
    let rule_id = "solid-shadow-unobserved";
    let path = file_db(&tmp, rule_id, false);
    let curator = PlaybookCurator::new(&path, Arc::new(test_clock()));
    curator.run_promotion_tick(&make_ssb(&tmp)).await.expect("tick");
    let conn = open_rules_db(&path, &test_clock()).expect("reopen db");
    let status: String = conn.query_row("SELECT status FROM rules WHERE rule_id = ?1", [rule_id], |r| r.get(0)).expect("status");
    assert_eq!(status, "SHADOW");
}
