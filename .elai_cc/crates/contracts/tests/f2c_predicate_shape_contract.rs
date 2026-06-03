//! F-2c predicate-shape + seed-integrity contract tests.
//!
//! Validates that the four new CWE rules (798/321/532/95) are wired into the
//! load-bearing seed path:
//!   1. Each new CANONICAL_ROWS predicate_json is a `grep_pattern` (NOT a
//!      composite) — the F-2c controller design decision (NoOpAst makes any
//!      AND(Grep,Ast) rule wire-dark on the live gate).
//!   2. Each new predicate_json deserialises to `Predicate::GrepPattern` with
//!      `file_context = Production` and a registered language (python|typescript).
//!   3. The CANONICAL_ROWS count == the authored total (20) and the in-crate
//!      count assertion agrees.
//!   4. The blake3 seed-integrity check passes over the full 20-row set (the
//!      digest is content-derived, so adding rows is self-consistent — there is
//!      NO hard-coded digit literal to drift).
//!   5. Every new rule_id has a security_rule_meta arm (seeding it does NOT fall
//!      through to the Unknown/WARN default).

use std::path::Path;

use elai_core::clock::SeededClock;
use elai_rules::predicate::{FileContext, Predicate};
use elai_rules::project_rule_seeder::ProjectRuleSeeder;
use elai_rules::schema::open_rules_db;
use elai_rules::vibesec_seed::{canonical_hex_digest, verify_vibesec_seed_integrity, CANONICAL_ROWS};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The four new F-2c rule_id prefixes.
const NEW_PREFIXES: &[&str] = &["SEC-CREDS-", "SEC-EVAL-", "SEC-KEY-", "SEC-LOG-"];

/// The expected post-F-2c CANONICAL_ROWS total: 12 F-2a + 8 F-2c (py+ts per CWE).
const EXPECTED_TOTAL: usize = 20;

fn new_rows() -> Vec<(&'static str, &'static str)> {
    CANONICAL_ROWS
        .iter()
        .filter(|(id, _)| NEW_PREFIXES.iter().any(|p| id.starts_with(p)))
        .copied()
        .collect()
}

fn seeded_db() -> rusqlite::Connection {
    let clock = SeededClock::new();
    let conn = open_rules_db(Path::new(":memory:"), &clock).expect("in-memory rules DB must open");
    ProjectRuleSeeder::seed_security_rules(&conn, &clock)
        .expect("seed_security_rules must succeed");
    conn
}

// ---------------------------------------------------------------------------
// Contract 1: exactly 8 new rows, 20 total
// ---------------------------------------------------------------------------

#[test]
fn f2c_adds_eight_new_rows_for_twenty_total() {
    assert_eq!(
        CANONICAL_ROWS.len(),
        EXPECTED_TOTAL,
        "CANONICAL_ROWS must hold 20 rows (12 F-2a + 8 F-2c)"
    );
    assert_eq!(
        new_rows().len(),
        8,
        "F-2c must add exactly 8 rows (CWE-798/321/532/95 × py+ts)"
    );
}

// ---------------------------------------------------------------------------
// Contract 2: every new predicate is GrepPattern-only (NOT composite/ast)
// ---------------------------------------------------------------------------

#[test]
fn f2c_new_predicates_are_grep_only_production_registered_lang() {
    for (id, pred_json) in new_rows() {
        let pred: Predicate = serde_json::from_str(pred_json)
            .unwrap_or_else(|e| panic!("{id} predicate_json must deserialise: {e}"));
        match pred {
            Predicate::GrepPattern {
                language,
                file_context,
                ..
            } => {
                assert_eq!(
                    file_context,
                    Some(FileContext::Production),
                    "{id} must scope file_context=production"
                );
                let lang = language.unwrap_or_else(|| panic!("{id} must carry a language"));
                assert!(
                    lang == "python" || lang == "typescript",
                    "{id} language must be a registered grammar (python|typescript); got {lang}"
                );
            }
            other => panic!(
                "{id} must be a GrepPattern (F-2c controller decision: NoOpAst makes \
                 composite AST children wire-dark); got {other:?}"
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// Contract 3: each CWE class ships both a python and a typescript variant
// ---------------------------------------------------------------------------

#[test]
fn f2c_each_cwe_class_has_python_and_typescript() {
    for prefix in NEW_PREFIXES {
        let langs: Vec<String> = new_rows()
            .iter()
            .filter(|(id, _)| id.starts_with(prefix))
            .map(|(_, pred)| {
                let p: Predicate = serde_json::from_str(pred).expect("deserialise");
                match p {
                    Predicate::GrepPattern { language, .. } => language.unwrap_or_default(),
                    _ => String::new(),
                }
            })
            .collect();
        assert!(
            langs.iter().any(|l| l == "python"),
            "{prefix}* must include a python variant"
        );
        assert!(
            langs.iter().any(|l| l == "typescript"),
            "{prefix}* must include a typescript variant"
        );
    }
}

// ---------------------------------------------------------------------------
// Contract 4: CANONICAL_ROWS stays sorted by rule_id ASC over the 20 rows
// ---------------------------------------------------------------------------

#[test]
fn f2c_canonical_rows_sorted() {
    let ids: Vec<&str> = CANONICAL_ROWS.iter().map(|(id, _)| *id).collect();
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    assert_eq!(ids, sorted, "CANONICAL_ROWS must stay sorted by rule_id ASC");
}

// ---------------------------------------------------------------------------
// Contract 5: blake3 seed integrity passes over the full 20-row seeded DB
// ---------------------------------------------------------------------------

#[test]
fn f2c_seed_integrity_passes_over_twenty_rows() {
    let conn = seeded_db();
    let total: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM rules WHERE domain = 'security' AND source = 'SEED'",
            [],
            |row| row.get(0),
        )
        .expect("count");
    assert_eq!(total, EXPECTED_TOTAL as i64, "20 immune security rows seeded");
    verify_vibesec_seed_integrity(&conn)
        .expect("blake3 seed integrity must pass over the 20-row set");
    // The digest is content-derived; recomputing it must be stable + 64 hex.
    let digest = canonical_hex_digest();
    assert_eq!(digest.len(), 64, "blake3 hex digest is 64 chars");
}

// ---------------------------------------------------------------------------
// Contract 6: every new rule seeds with a real (non-default) severity/title
// ---------------------------------------------------------------------------

#[test]
fn f2c_new_rules_have_security_rule_meta_arm() {
    let conn = seeded_db();
    for (id, _) in new_rows() {
        let (title, severity): (String, String) = conn
            .query_row(
                "SELECT title, severity FROM rules WHERE rule_id = ?1",
                [id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap_or_else(|e| panic!("{id} must be seeded: {e}"));
        assert_ne!(
            title, "Unknown security rule",
            "{id} must have a security_rule_meta arm (not the Unknown fallthrough)"
        );
        assert_eq!(
            severity, "BLOCK",
            "{id} ships severity=BLOCK (F-2c: WARN is gate-silent in evaluate_vibegate)"
        );
        // The title must name the CWE id for the class.
        let cwe = if id.starts_with("SEC-CREDS-") {
            "CWE-798"
        } else if id.starts_with("SEC-KEY-") {
            "CWE-321"
        } else if id.starts_with("SEC-LOG-") {
            "CWE-532"
        } else {
            "CWE-95"
        };
        assert!(
            title.contains(cwe),
            "{id} title must name {cwe}; got: {title}"
        );
    }
}
