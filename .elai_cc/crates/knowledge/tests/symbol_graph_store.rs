// B-29 Phase 3 — store.rs integration tests.
//
// Tests the 5 read functions and 5 write helpers in symbol_graph::store.
// All tests use an in-memory rusqlite Connection (":memory:") and manually
// create the v3 schema to avoid depending on DkgEngine.

use std::collections::{BTreeMap, BTreeSet};

use elai_knowledge::symbol_graph::store;
use elai_knowledge::symbol_graph::types::{
    Confidence, SymbolEdge, SymbolGraphFilePayload, SymbolGraphMeta, SymbolKind, SymbolNode,
    SymbolRef, CallSite,
};
use rusqlite::Connection;

// ---------------------------------------------------------------------------
// Schema bootstrap (mirrors SCHEMA_SQL_V3 in dkg/mod.rs)
// ---------------------------------------------------------------------------

fn create_v3_schema(conn: &Connection) {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS symbol_name_index (
            shard_key    TEXT    NOT NULL,
            symbol_name  TEXT    NOT NULL,
            refs_msgpack BLOB    NOT NULL,
            PRIMARY KEY (shard_key, symbol_name)
        );
        CREATE TABLE IF NOT EXISTS symbol_reverse_index (
            shard_bucket         INTEGER NOT NULL,
            callee_file          TEXT    NOT NULL,
            caller_files_msgpack BLOB    NOT NULL,
            PRIMARY KEY (shard_bucket, callee_file)
        );
        CREATE TABLE IF NOT EXISTS symbol_file_payload (
            file             TEXT PRIMARY KEY,
            language         TEXT NOT NULL,
            content_hash     TEXT NOT NULL,
            symbols_msgpack  BLOB NOT NULL,
            edges_msgpack    BLOB NOT NULL
        );
        CREATE TABLE IF NOT EXISTS symbol_graph_meta (
            project_id          TEXT PRIMARY KEY,
            symbol_count        INTEGER NOT NULL,
            edge_count          INTEGER NOT NULL,
            file_count          INTEGER NOT NULL,
            unresolved_edge_pct REAL    NOT NULL,
            built_at            INTEGER NOT NULL,
            schema_version      INTEGER NOT NULL
        );
        ",
    )
    .expect("create v3 schema");
}

fn open_mem() -> Connection {
    let conn = Connection::open_in_memory().expect("open in-memory db");
    create_v3_schema(&conn);
    conn
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn meta_fixture(project_id: &str) -> SymbolGraphMeta {
    SymbolGraphMeta {
        project_id: project_id.to_string(),
        symbol_count: 10,
        edge_count: 5,
        file_count: 3,
        unresolved_edge_pct: 20.0,
        built_at: 12345678,
        schema_version: 3,
    }
}

fn sym(id: &str, name: &str, file: &str) -> SymbolNode {
    SymbolNode {
        id: id.to_string(),
        name: name.to_string(),
        qualified_name: name.to_string(),
        kind: SymbolKind::Function,
        file: file.to_string(),
        line: 1,
        end_line: 10,
        language: "rust".to_string(),
    }
}

fn edge(caller_id: &str, callee_name: &str, file: &str) -> SymbolEdge {
    SymbolEdge {
        caller_id: caller_id.to_string(),
        callee_name: callee_name.to_string(),
        callee_candidates: vec![format!("{}::{}", file, callee_name)],
        confidence: Confidence::Unique,
        call_site: CallSite {
            file: file.to_string(),
            line: 5,
        },
    }
}

fn payload_fixture(file: &str) -> SymbolGraphFilePayload {
    SymbolGraphFilePayload {
        file: file.to_string(),
        language: "rust".to_string(),
        content_hash: "abc123".to_string(),
        symbols: vec![sym(&format!("{}::foo", file), "foo", file)],
        outgoing_calls: vec![edge(&format!("{}::foo", file), "bar", file)],
    }
}

// ---------------------------------------------------------------------------
// load_meta / upsert_meta
// ---------------------------------------------------------------------------

#[test]
fn load_meta_returns_none_when_no_row() {
    let conn = open_mem();
    let result = store::load_meta(&conn, "proj_missing").expect("load_meta should not error");
    assert!(result.is_none(), "expected None for missing project_id");
}

#[test]
fn upsert_and_load_meta_roundtrip() {
    let mut conn = open_mem();
    let meta = meta_fixture("proj1");

    store::upsert_meta(&mut conn, &meta).expect("upsert_meta");
    let loaded = store::load_meta(&conn, "proj1")
        .expect("load_meta")
        .expect("expected Some");

    assert_eq!(loaded.project_id, meta.project_id);
    assert_eq!(loaded.symbol_count, meta.symbol_count);
    assert_eq!(loaded.edge_count, meta.edge_count);
    assert_eq!(loaded.file_count, meta.file_count);
    assert_eq!(loaded.schema_version, meta.schema_version);
    assert!((loaded.unresolved_edge_pct - meta.unresolved_edge_pct).abs() < 0.001);
    assert_eq!(loaded.built_at, meta.built_at);
}

#[test]
fn upsert_meta_replace_updates_counts() {
    let mut conn = open_mem();
    let meta1 = meta_fixture("proj1");
    store::upsert_meta(&mut conn, &meta1).expect("first upsert");

    let meta2 = SymbolGraphMeta {
        symbol_count: 99,
        edge_count: 42,
        ..meta_fixture("proj1")
    };
    store::upsert_meta(&mut conn, &meta2).expect("second upsert");

    let loaded = store::load_meta(&conn, "proj1")
        .expect("load_meta")
        .expect("Some after second upsert");
    assert_eq!(loaded.symbol_count, 99, "symbol_count must be updated on replace");
    assert_eq!(loaded.edge_count, 42, "edge_count must be updated on replace");
}

// ---------------------------------------------------------------------------
// load_name_shard / upsert_name_shard
// ---------------------------------------------------------------------------

#[test]
fn name_shard_roundtrip() {
    let mut conn = open_mem();
    let mut entries: BTreeMap<String, Vec<SymbolRef>> = BTreeMap::new();
    entries.insert(
        "foo".to_string(),
        vec![SymbolRef {
            file: "a.rs".to_string(),
            id: "a.rs::foo#L1".to_string(),
        }],
    );

    store::upsert_name_shard(&mut conn, 'f', &entries).expect("upsert_name_shard");
    let loaded = store::load_name_shard(&conn, 'f').expect("load_name_shard");

    assert_eq!(loaded.len(), 1);
    assert!(loaded.contains_key("foo"));
    assert_eq!(loaded["foo"].len(), 1);
    assert_eq!(loaded["foo"][0].file, "a.rs");
    assert_eq!(loaded["foo"][0].id, "a.rs::foo#L1");
}

#[test]
fn name_shard_upsert_deletes_old_entries_for_shard() {
    let mut conn = open_mem();

    // Insert 3 names into shard 'f'.
    let mut entries1: BTreeMap<String, Vec<SymbolRef>> = BTreeMap::new();
    for name in &["foo", "bar", "baz"] {
        entries1.insert(
            name.to_string(),
            vec![SymbolRef {
                file: "x.rs".to_string(),
                id: format!("x.rs::{}#L1", name),
            }],
        );
    }
    store::upsert_name_shard(&mut conn, 'f', &entries1).expect("initial upsert");

    // Re-upsert with only 1 name; old 2 must be gone.
    let mut entries2: BTreeMap<String, Vec<SymbolRef>> = BTreeMap::new();
    entries2.insert(
        "foo".to_string(),
        vec![SymbolRef {
            file: "x.rs".to_string(),
            id: "x.rs::foo#L1".to_string(),
        }],
    );
    store::upsert_name_shard(&mut conn, 'f', &entries2).expect("second upsert");

    let loaded = store::load_name_shard(&conn, 'f').expect("load after re-upsert");
    assert_eq!(loaded.len(), 1, "only 1 name must remain after re-upsert");
    assert!(loaded.contains_key("foo"));
}

#[test]
fn name_shard_empty_returns_empty_map() {
    let conn = open_mem();
    let loaded = store::load_name_shard(&conn, 'z').expect("load empty shard");
    assert!(loaded.is_empty(), "missing shard returns empty map");
}

// ---------------------------------------------------------------------------
// load_reverse_shard / upsert_reverse_shard
// ---------------------------------------------------------------------------

#[test]
fn reverse_shard_roundtrip() {
    let mut conn = open_mem();
    let mut entries: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut callers = BTreeSet::new();
    callers.insert("a.rs".to_string());
    callers.insert("b.rs".to_string());
    entries.insert("lib.rs".to_string(), callers);

    store::upsert_reverse_shard(&mut conn, 0, &entries).expect("upsert_reverse_shard");
    let loaded = store::load_reverse_shard(&conn, 0).expect("load_reverse_shard");

    assert_eq!(loaded.len(), 1);
    assert!(loaded.contains_key("lib.rs"));
    assert_eq!(loaded["lib.rs"].len(), 2);
    assert!(loaded["lib.rs"].contains("a.rs"));
    assert!(loaded["lib.rs"].contains("b.rs"));
}

#[test]
fn reverse_shard_empty_bucket_returns_empty() {
    let conn = open_mem();
    let loaded = store::load_reverse_shard(&conn, 255).expect("load empty reverse shard");
    assert!(loaded.is_empty());
}

// ---------------------------------------------------------------------------
// load_file_payload / upsert_file_payload / delete_file_payload
// ---------------------------------------------------------------------------

#[test]
fn file_payload_roundtrip() {
    let mut conn = open_mem();
    let payload = payload_fixture("src/main.rs");

    store::upsert_file_payload(&mut conn, &payload).expect("upsert_file_payload");
    let loaded = store::load_file_payload(&conn, "src/main.rs")
        .expect("load_file_payload")
        .expect("expected Some");

    assert_eq!(loaded.file, payload.file);
    assert_eq!(loaded.language, payload.language);
    assert_eq!(loaded.content_hash, payload.content_hash);
    assert_eq!(loaded.symbols.len(), payload.symbols.len());
    assert_eq!(loaded.symbols[0].id, payload.symbols[0].id);
    assert_eq!(loaded.outgoing_calls.len(), payload.outgoing_calls.len());
    assert_eq!(loaded.outgoing_calls[0].caller_id, payload.outgoing_calls[0].caller_id);
}

#[test]
fn file_payload_returns_none_for_missing_file() {
    let conn = open_mem();
    let result = store::load_file_payload(&conn, "nonexistent.rs").expect("no error");
    assert!(result.is_none());
}

#[test]
fn upsert_file_payload_is_idempotent() {
    let mut conn = open_mem();
    let payload = payload_fixture("src/lib.rs");
    store::upsert_file_payload(&mut conn, &payload).expect("first upsert");
    store::upsert_file_payload(&mut conn, &payload).expect("second upsert (idempotent)");

    let loaded = store::load_file_payload(&conn, "src/lib.rs")
        .expect("load")
        .expect("Some");
    assert_eq!(loaded.symbols.len(), 1);
}

#[test]
fn delete_file_payload_removes_row() {
    let mut conn = open_mem();
    let payload = payload_fixture("del.rs");
    store::upsert_file_payload(&mut conn, &payload).expect("upsert");
    store::delete_file_payload(&mut conn, "del.rs").expect("delete");

    let result = store::load_file_payload(&conn, "del.rs").expect("load after delete");
    assert!(result.is_none(), "row must be gone after delete");
}

// ---------------------------------------------------------------------------
// list_all_files
// ---------------------------------------------------------------------------

#[test]
fn list_all_files_returns_all_inserted() {
    let mut conn = open_mem();
    for name in &["a.rs", "b.rs", "c.rs"] {
        store::upsert_file_payload(&mut conn, &payload_fixture(name)).expect("upsert");
    }

    let files = store::list_all_files(&conn).expect("list_all_files");
    assert_eq!(files.len(), 3);
    for name in &["a.rs", "b.rs", "c.rs"] {
        assert!(files.contains(&name.to_string()), "{name} missing");
    }
}

#[test]
fn list_all_files_empty_db_returns_empty() {
    let conn = open_mem();
    let files = store::list_all_files(&conn).expect("list_all_files empty");
    assert!(files.is_empty());
}
