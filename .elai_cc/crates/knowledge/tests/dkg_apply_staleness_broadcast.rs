//! F-5 / Slice A4 — DkgEngine::apply_staleness_broadcast cascade.
//!
//! A quarantined record (joined fact->concept via the `concept` TEXT column)
//! must (1) flip the matching concept's `concept_stale` to 2 and (2) insert a
//! non-destructive `staleness_quarantined` self-loop marker edge on the
//! matched node. Non-matching concepts (and their legacy `concept_stale = 1`)
//! must be left untouched.
//!
//! Design reference: workflow/reasoning/F-5-decisions.md (Slice A4).

use elai_knowledge::dkg::{
    DkgEngine, QuarantineRecordView, CONCEPT_STALE_QUARANTINED, EDGE_STALENESS_QUARANTINED,
};
use rusqlite::Connection;

/// Create a DkgEngine plus a second raw Connection to the same DB for
/// test-setup inserts and assertions. The TempDir must stay alive for the DB.
fn make_engine() -> (tempfile::TempDir, DkgEngine, Connection) {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let db = dir.path().join("test.db");
    let engine = DkgEngine::new(&db, None, 0i64).expect("engine");
    let conn = Connection::open(&db).expect("conn");
    (dir, engine, conn)
}

/// Insert a node carrying an explicit `concept` (and the default
/// `concept_stale = 1`). Returns the node's row id.
fn insert_concept_node(conn: &Connection, name: &str, concept: &str) -> i64 {
    conn.execute(
        "INSERT INTO dkg_nodes \
            (file_path, name, kind, signature, start_line, end_line, concept) \
         VALUES (?1, ?2, 'function', '', 1, 2, ?3)",
        rusqlite::params![format!("{name}.rs"), name, concept],
    )
    .expect("insert node");
    conn.last_insert_rowid()
}

fn concept_stale_of(conn: &Connection, node_id: i64) -> i64 {
    conn.query_row(
        "SELECT concept_stale FROM dkg_nodes WHERE id = ?1",
        [node_id],
        |r| r.get::<_, i64>(0),
    )
    .expect("concept_stale read")
}

fn marker_edge_count(conn: &Connection, node_id: i64) -> i64 {
    conn.query_row(
        "SELECT COUNT(*) FROM dkg_edges \
         WHERE source_id = ?1 AND target_id = ?1 AND edge_type = ?2",
        rusqlite::params![node_id, EDGE_STALENESS_QUARANTINED],
        |r| r.get::<_, i64>(0),
    )
    .expect("edge count read")
}

#[test]
fn quarantined_record_flips_concept_stale_and_inserts_marker_edge() {
    let (_dir, engine, conn) = make_engine();

    // A dependent concept that exactly matches the quarantined fact id, plus an
    // unrelated concept that must NOT be disturbed.
    let matched_id = insert_concept_node(&conn, "alpha_fn", "fact-alpha");
    let other_id = insert_concept_node(&conn, "beta_fn", "fact-beta");

    // Pre-conditions: both carry the default-unknown 1, no marker edges.
    assert_eq!(concept_stale_of(&conn, matched_id), 1);
    assert_eq!(concept_stale_of(&conn, other_id), 1);
    assert_eq!(marker_edge_count(&conn, matched_id), 0);

    let records = vec![QuarantineRecordView::new("fact-alpha", 1.8)];
    let stats = engine
        .apply_staleness_broadcast(&records)
        .expect("apply broadcast");

    // The matched concept is demoted to the quarantine marker value 2, and one
    // self-loop marker edge is inserted on its node.
    assert_eq!(stats.dependents_demoted, 1, "one matched concept demoted");
    assert_eq!(stats.marker_edges_inserted, 1, "one marker edge inserted");
    assert_eq!(
        concept_stale_of(&conn, matched_id),
        CONCEPT_STALE_QUARANTINED
    );
    assert_eq!(marker_edge_count(&conn, matched_id), 1);

    // The unrelated concept keeps its legacy 1 and gets no marker edge.
    assert_eq!(concept_stale_of(&conn, other_id), 1);
    assert_eq!(marker_edge_count(&conn, other_id), 0);
}

#[test]
fn apply_is_idempotent_on_repeat() {
    let (_dir, engine, conn) = make_engine();
    let node_id = insert_concept_node(&conn, "gamma_fn", "fact-gamma");

    let records = vec![QuarantineRecordView::new("fact-gamma", 2.0)];

    let first = engine
        .apply_staleness_broadcast(&records)
        .expect("first apply");
    assert_eq!(first.marker_edges_inserted, 1);

    // A second identical broadcast must not double-insert the marker edge
    // (INSERT OR IGNORE) and the concept stays quarantined.
    let second = engine
        .apply_staleness_broadcast(&records)
        .expect("second apply");
    assert_eq!(
        second.marker_edges_inserted, 0,
        "marker edge already present, INSERT OR IGNORE counts zero"
    );
    assert_eq!(concept_stale_of(&conn, node_id), CONCEPT_STALE_QUARANTINED);
    assert_eq!(marker_edge_count(&conn, node_id), 1, "no duplicate edge");
}

#[test]
fn no_match_demotes_nothing() {
    let (_dir, engine, conn) = make_engine();
    let node_id = insert_concept_node(&conn, "delta_fn", "fact-delta");

    let records = vec![QuarantineRecordView::new("fact-nonexistent", 3.0)];
    let stats = engine
        .apply_staleness_broadcast(&records)
        .expect("apply broadcast");

    assert_eq!(stats.dependents_demoted, 0);
    assert_eq!(stats.marker_edges_inserted, 0);
    assert_eq!(concept_stale_of(&conn, node_id), 1, "legacy 1 untouched");
}
