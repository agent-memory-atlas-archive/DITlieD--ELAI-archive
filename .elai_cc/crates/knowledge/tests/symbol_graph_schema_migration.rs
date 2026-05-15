// Integration tests: schema migration v3 — symbol graph tables in dkg.db.
// Uses tempfile to simulate a fresh database; verifies idempotency and table
// existence after DkgEngine::new.

use elai_knowledge::dkg::DkgEngine;
use rusqlite::Connection;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Return a sorted list of table names from sqlite_master.
fn list_tables(conn: &Connection) -> Vec<String> {
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
        .expect("prepare sqlite_master query");
    let names: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .expect("query_map")
        .map(|r| r.expect("row"))
        .collect();
    names
}

/// Read the schema_version from dkg_meta.
fn schema_version(conn: &Connection) -> String {
    conn.query_row(
        "SELECT value FROM dkg_meta WHERE key = 'schema_version'",
        [],
        |row| row.get(0),
    )
    .expect("schema_version query")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn migration_v3_creates_all_four_tables() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let db_path = dir.path().join("dkg.db");

    let _engine = DkgEngine::new(&db_path, None, 0).expect("DkgEngine::new");

    let conn = Connection::open(&db_path).expect("open db for inspection");
    let tables = list_tables(&conn);

    assert!(
        tables.contains(&"symbol_name_index".to_owned()),
        "symbol_name_index missing; tables = {tables:?}"
    );
    assert!(
        tables.contains(&"symbol_reverse_index".to_owned()),
        "symbol_reverse_index missing; tables = {tables:?}"
    );
    assert!(
        tables.contains(&"symbol_file_payload".to_owned()),
        "symbol_file_payload missing; tables = {tables:?}"
    );
    assert!(
        tables.contains(&"symbol_graph_meta".to_owned()),
        "symbol_graph_meta missing; tables = {tables:?}"
    );
}

#[test]
fn migration_v3_idempotent_second_open() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let db_path = dir.path().join("dkg.db");

    // First open — creates schema.
    let _engine1 = DkgEngine::new(&db_path, None, 0).expect("DkgEngine::new first");
    // Second open — must not error (CREATE TABLE IF NOT EXISTS is idempotent).
    let _engine2 = DkgEngine::new(&db_path, None, 0).expect("DkgEngine::new second (idempotent)");

    let conn = Connection::open(&db_path).expect("open db for inspection");
    let tables = list_tables(&conn);
    // All four new tables must still be there after two opens.
    for name in &[
        "symbol_name_index",
        "symbol_reverse_index",
        "symbol_file_payload",
        "symbol_graph_meta",
    ] {
        assert!(
            tables.contains(&(*name).to_owned()),
            "table {name} missing after idempotent second open"
        );
    }
}

#[test]
fn schema_version_is_3() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let db_path = dir.path().join("dkg.db");

    let _engine = DkgEngine::new(&db_path, None, 0).expect("DkgEngine::new");

    let conn = Connection::open(&db_path).expect("open db for inspection");
    let version = schema_version(&conn);
    assert_eq!(
        version, "3",
        "expected schema_version='3' in dkg_meta, got '{version}'"
    );
}
