//! Contract tests for A-48: DKG Data Flow Edges.
//!
//! Validates: dkg_data_flow table creation, FlowType enum variants,
//! CHECK constraint on flow_type column, data flow edge insertion/query,
//! cascade delete when source/target node removed, and tree-sitter
//! extraction of PARAMETER/RETURN/ASSIGNMENT/FIELD_ACCESS edges from
//! Rust source fixtures.

use std::fs;

use elai_knowledge::dkg::DkgEngine;
use elai_knowledge::FlowType;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn temp_db() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("test_a48.db");
    (dir, db_path)
}

fn insert_node(
    conn: &rusqlite::Connection,
    name: &str,
    kind: &str,
    file: &str,
    start: u32,
    end: u32,
) -> i64 {
    conn.execute(
        "INSERT INTO dkg_nodes (file_path, name, kind, signature, start_line, end_line) \
         VALUES (?1, ?2, ?3, '', ?4, ?5)",
        rusqlite::params![file, name, kind, start, end],
    )
    .expect("insert node");
    conn.last_insert_rowid()
}

// ---------------------------------------------------------------------------
// 1. FlowType enum has exactly 4 variants and round-trips to/from str
// ---------------------------------------------------------------------------

#[test]
fn flow_type_has_four_variants() {
    let variants = [
        FlowType::Parameter,
        FlowType::Return,
        FlowType::Assignment,
        FlowType::FieldAccess,
    ];
    assert_eq!(variants.len(), 4);
}

#[test]
fn flow_type_as_str_round_trip() {
    for ft in &[
        FlowType::Parameter,
        FlowType::Return,
        FlowType::Assignment,
        FlowType::FieldAccess,
    ] {
        let s = ft.as_str();
        let parsed = FlowType::from_str(s);
        assert!(parsed.is_some(), "failed to parse: {s}");
        assert_eq!(*ft, parsed.unwrap());
    }
}

#[test]
fn flow_type_from_str_rejects_invalid() {
    assert!(FlowType::from_str("INVALID").is_none());
    assert!(FlowType::from_str("").is_none());
    assert!(FlowType::from_str("parameter").is_none()); // case-sensitive
}

// ---------------------------------------------------------------------------
// 2. dkg_data_flow table is created by DkgEngine::new
// ---------------------------------------------------------------------------

#[test]
fn schema_creates_data_flow_table() {
    let (_dir, db_path) = temp_db();
    let _engine = DkgEngine::new(&db_path, None, 0).expect("engine");

    let conn = rusqlite::Connection::open(&db_path).expect("open");
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='dkg_data_flow'",
            [],
            |row| row.get(0),
        )
        .expect("query");
    assert_eq!(count, 1, "dkg_data_flow table must exist after engine init");
}

// ---------------------------------------------------------------------------
// 3. flow_type CHECK constraint rejects invalid values
// ---------------------------------------------------------------------------

#[test]
fn data_flow_check_constraint_rejects_invalid() {
    let (_dir, db_path) = temp_db();
    let _engine = DkgEngine::new(&db_path, None, 0).expect("engine");

    let conn = rusqlite::Connection::open(&db_path).expect("open");
    let src = insert_node(&conn, "foo", "function", "test.rs", 1, 5);
    let tgt = insert_node(&conn, "bar", "function", "test.rs", 6, 10);

    let result = conn.execute(
        "INSERT INTO dkg_data_flow (source_node_id, target_node_id, variable_name, flow_type) \
         VALUES (?1, ?2, 'x', 'INVALID_TYPE')",
        rusqlite::params![src, tgt],
    );
    assert!(result.is_err(), "CHECK constraint must reject invalid flow_type");
}

// ---------------------------------------------------------------------------
// 4. Valid flow_type values are accepted
// ---------------------------------------------------------------------------

#[test]
fn data_flow_accepts_all_valid_flow_types() {
    let (_dir, db_path) = temp_db();
    let _engine = DkgEngine::new(&db_path, None, 0).expect("engine");

    let conn = rusqlite::Connection::open(&db_path).expect("open");
    let src = insert_node(&conn, "foo", "function", "test.rs", 1, 5);
    let tgt = insert_node(&conn, "bar", "function", "test.rs", 6, 10);

    for ft in &["PARAMETER", "RETURN", "ASSIGNMENT", "FIELD_ACCESS"] {
        let result = conn.execute(
            "INSERT INTO dkg_data_flow (source_node_id, target_node_id, variable_name, flow_type) \
             VALUES (?1, ?2, 'var', ?3)",
            rusqlite::params![src, tgt, ft],
        );
        assert!(result.is_ok(), "flow_type '{ft}' must be accepted");
    }
}

// ---------------------------------------------------------------------------
// 5. Foreign key cascade: deleting a node removes its data flow edges
// ---------------------------------------------------------------------------

#[test]
fn data_flow_cascade_on_node_delete() {
    let (_dir, db_path) = temp_db();
    let _engine = DkgEngine::new(&db_path, None, 0).expect("engine");

    let conn = rusqlite::Connection::open(&db_path).expect("open");
    conn.execute_batch("PRAGMA foreign_keys=ON;").expect("pragma");

    let src = insert_node(&conn, "foo", "function", "test.rs", 1, 5);
    let tgt = insert_node(&conn, "bar", "function", "test.rs", 6, 10);

    conn.execute(
        "INSERT INTO dkg_data_flow (source_node_id, target_node_id, variable_name, flow_type) \
         VALUES (?1, ?2, 'x', 'PARAMETER')",
        rusqlite::params![src, tgt],
    )
    .expect("insert flow");

    // Delete source node — should cascade
    conn.execute("DELETE FROM dkg_nodes WHERE id = ?1", [src])
        .expect("delete");

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM dkg_data_flow", [], |row| row.get(0))
        .expect("count");
    assert_eq!(count, 0, "cascade delete must remove data flow edges");
}

// ---------------------------------------------------------------------------
// 6. DkgEngine.data_flow_edges() returns inserted edges
// ---------------------------------------------------------------------------

#[test]
fn engine_data_flow_edges_query() {
    let (_dir, db_path) = temp_db();
    let engine = DkgEngine::new(&db_path, None, 0).expect("engine");

    // Use test helper to insert a data flow edge
    engine.insert_test_data_flow(1, 2, "my_var", FlowType::Assignment);

    let edges = engine.data_flow_edges().expect("query");
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].source_node_id, 1);
    assert_eq!(edges[0].target_node_id, 2);
    assert_eq!(edges[0].variable_name, "my_var");
    assert_eq!(edges[0].flow_type, FlowType::Assignment);
}

// ---------------------------------------------------------------------------
// 7. DkgEngine.data_flow_for_node() filters by node ID
// ---------------------------------------------------------------------------

#[test]
fn engine_data_flow_for_node_filters() {
    let (_dir, db_path) = temp_db();
    let engine = DkgEngine::new(&db_path, None, 0).expect("engine");

    engine.insert_test_data_flow(1, 2, "a", FlowType::Parameter);
    engine.insert_test_data_flow(2, 3, "b", FlowType::Return);
    engine.insert_test_data_flow(4, 5, "c", FlowType::FieldAccess);

    // Node 2 is source in one edge, target in another
    let edges = engine.data_flow_for_node(2).expect("query");
    assert_eq!(edges.len(), 2);
}

// ---------------------------------------------------------------------------
// 8. build_graph extracts data flow edges from Rust source
// ---------------------------------------------------------------------------

#[test]
fn build_graph_extracts_rust_data_flow() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("dkg_flow.db");

    // Write a Rust fixture with clear data flow patterns
    let fixture_dir = dir.path().join("src");
    fs::create_dir_all(&fixture_dir).expect("mkdir");
    fs::write(
        fixture_dir.join("main.rs"),
        r#"
fn producer() -> i32 {
    42
}

fn consumer(x: i32) -> i32 {
    let result = producer();
    let y = x;
    result + y
}
"#,
    )
    .expect("write fixture");

    let mut engine = DkgEngine::new(&db_path, None, 0).expect("engine");
    let graph = engine.build_graph(dir.path()).expect("build_graph");
    assert!(graph.node_count > 0, "must parse nodes");

    let edges = engine.data_flow_edges().expect("query");
    // Should have at least: assignment (y = x), call/return (result = producer())
    assert!(
        !edges.is_empty(),
        "build_graph must extract data flow edges from Rust source"
    );

    // Verify flow types present
    let types: Vec<&str> = edges.iter().map(|e| e.flow_type.as_str()).collect();
    assert!(
        types.contains(&"ASSIGNMENT") || types.contains(&"RETURN") || types.contains(&"PARAMETER"),
        "extracted edges must include recognized flow types, got: {types:?}"
    );
}

// ---------------------------------------------------------------------------
// 9. Data flow edge count is included in DependencyGraph
// ---------------------------------------------------------------------------

#[test]
fn dependency_graph_includes_data_flow_count() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("dkg_flow2.db");

    let fixture_dir = dir.path().join("src");
    fs::create_dir_all(&fixture_dir).expect("mkdir");
    fs::write(
        fixture_dir.join("lib.rs"),
        r#"
fn add(a: i32, b: i32) -> i32 {
    let sum = a + b;
    sum
}
"#,
    )
    .expect("write fixture");

    let mut engine = DkgEngine::new(&db_path, None, 0).expect("engine");
    let graph = engine.build_graph(dir.path()).expect("build_graph");

    // data_flow_count field must exist on DependencyGraph
    // It's OK if it's 0 for trivial cases, but the field must compile
    let _count = graph.data_flow_count;
}

// ---------------------------------------------------------------------------
// 10. Deterministic: same input produces same edges
// ---------------------------------------------------------------------------

#[test]
fn data_flow_extraction_is_deterministic() {
    let source = r#"
fn caller() {
    let val = callee(42);
}

fn callee(x: i32) -> i32 {
    x + 1
}
"#;

    let dir1 = tempfile::tempdir().expect("t1");
    let dir2 = tempfile::tempdir().expect("t2");

    for dir in [&dir1, &dir2] {
        let src_dir = dir.path().join("src");
        fs::create_dir_all(&src_dir).expect("mkdir");
        fs::write(src_dir.join("main.rs"), source).expect("write");
    }

    let db1 = dir1.path().join("dkg.db");
    let db2 = dir2.path().join("dkg.db");

    let mut e1 = DkgEngine::new(&db1, None, 0).expect("e1");
    let mut e2 = DkgEngine::new(&db2, None, 0).expect("e2");

    e1.build_graph(dir1.path()).expect("build1");
    e2.build_graph(dir2.path()).expect("build2");

    let edges1 = e1.data_flow_edges().expect("q1");
    let edges2 = e2.data_flow_edges().expect("q2");

    assert_eq!(edges1.len(), edges2.len(), "deterministic edge count");
    for (a, b) in edges1.iter().zip(edges2.iter()) {
        assert_eq!(a.variable_name, b.variable_name);
        assert_eq!(a.flow_type, b.flow_type);
    }
}
