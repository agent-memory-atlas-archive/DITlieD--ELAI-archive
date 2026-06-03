//! Contract test for F-5 Slice A2 (Phase 1): concept_stale write-through round-trip.
//!
//! The F-5 staleness-quarantine cascade demotes a DKG concept's freshness marker
//! by writing `concept_stale = 2` (0 = explicitly fresh, 1 = DDL default + the value
//! the sole node INSERT hardcodes, 2 = quarantined). There are ZERO production READ
//! consumers of `concept_stale` today, so there is NO read filter to exercise. The
//! observable for Phase 1 is therefore the UPDATE round-trip: write 2 via the public
//! `DkgEngine::set_concept_stale` mutator, re-query the row, and assert it reads back 2.
//!
//! Matching is on the `concept` TEXT column (dkg_nodes has no fact_id; the fact->concept
//! join is by text). A non-matching concept MUST be left untouched (it keeps its
//! default value of 1), proving the WHERE clause is scoped.

use elai_knowledge::dkg::DkgEngine;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn temp_db() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("f5_concept_stale.db");
    (dir, db_path)
}

/// Insert a dkg_nodes row carrying an explicit `concept`. `concept_stale` is omitted
/// so it takes the schema DEFAULT (1), mirroring the production INSERT path.
fn insert_node_with_concept(conn: &rusqlite::Connection, name: &str, concept: &str) -> i64 {
    conn.execute(
        "INSERT INTO dkg_nodes (file_path, name, kind, signature, start_line, end_line, concept) \
         VALUES (?1, ?2, 'function', '', 1, 5, ?3)",
        rusqlite::params!["test.rs", name, concept],
    )
    .expect("insert node with concept");
    conn.last_insert_rowid()
}

/// Read back the `concept_stale` value for a node by id.
fn read_concept_stale(conn: &rusqlite::Connection, node_id: i64) -> i64 {
    conn.query_row(
        "SELECT concept_stale FROM dkg_nodes WHERE id = ?1",
        rusqlite::params![node_id],
        |row| row.get(0),
    )
    .expect("read concept_stale")
}

// ---------------------------------------------------------------------------
// 1. Write-through round-trip: set_concept_stale(concept, 2) then re-query reads 2.
// ---------------------------------------------------------------------------

#[test]
fn set_concept_stale_write_through_round_trip() {
    let (_dir, db_path) = temp_db();
    let engine = DkgEngine::new(&db_path, None, 0).expect("engine");

    // Separate raw connection for direct insert + re-query (mirrors the A-48 pattern).
    let conn = rusqlite::Connection::open(&db_path).expect("open");
    let target_id = insert_node_with_concept(&conn, "decay_target", "stale_concept");

    // Precondition: defaults to the DDL default of 1.
    assert_eq!(
        read_concept_stale(&conn, target_id),
        1,
        "node must default to concept_stale=1 before quarantine"
    );

    // The quarantine write: demote to 2.
    let rows = engine
        .set_concept_stale("stale_concept", 2)
        .expect("set_concept_stale");
    assert_eq!(rows, 1, "exactly one row must be updated");

    // Re-query through the separate reader connection: the write is visible and reads 2.
    assert_eq!(
        read_concept_stale(&conn, target_id),
        2,
        "concept_stale must read back as 2 (quarantined) after the cascade write"
    );
}

// ---------------------------------------------------------------------------
// 2. Scoped WHERE: a non-matching concept is left untouched at its default of 1.
// ---------------------------------------------------------------------------

#[test]
fn set_concept_stale_leaves_non_matching_concept_untouched() {
    let (_dir, db_path) = temp_db();
    let engine = DkgEngine::new(&db_path, None, 0).expect("engine");

    let conn = rusqlite::Connection::open(&db_path).expect("open");
    let target_id = insert_node_with_concept(&conn, "decay_target", "stale_concept");
    let bystander_id = insert_node_with_concept(&conn, "fresh_target", "fresh_concept");

    let rows = engine
        .set_concept_stale("stale_concept", 2)
        .expect("set_concept_stale");
    assert_eq!(rows, 1, "only the matching concept's row must be updated");

    assert_eq!(
        read_concept_stale(&conn, target_id),
        2,
        "matching concept must be quarantined"
    );
    assert_eq!(
        read_concept_stale(&conn, bystander_id),
        1,
        "non-matching concept must keep its default of 1 (never disturbed by the cascade)"
    );
}

// ---------------------------------------------------------------------------
// 3. No-match returns zero rows updated (idempotent / safe on absent concepts).
// ---------------------------------------------------------------------------

#[test]
fn set_concept_stale_no_match_returns_zero() {
    let (_dir, db_path) = temp_db();
    let engine = DkgEngine::new(&db_path, None, 0).expect("engine");

    let conn = rusqlite::Connection::open(&db_path).expect("open");
    let _ = insert_node_with_concept(&conn, "decay_target", "stale_concept");

    let rows = engine
        .set_concept_stale("concept_that_does_not_exist", 2)
        .expect("set_concept_stale");
    assert_eq!(rows, 0, "no matching concept means zero rows updated");
}
