use elai_knowledge::dkg::{DkgEngine, EDGE_CONTAINS, EDGE_IMPLEMENTS, EDGE_IMPORTS};
use elai_knowledge::{DkgReadiness, SymbolKind};
use rusqlite::Connection;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a DkgEngine and return the TempDir (must stay alive), the engine,
/// and a second raw Connection to the same DB for test-setup inserts.
fn make_engine() -> (tempfile::TempDir, DkgEngine, Connection) {
    let dir = tempfile::TempDir::new().unwrap();
    let db = dir.path().join("test.db");
    let engine = DkgEngine::new(&db, None, 0i64).unwrap();
    let conn = Connection::open(&db).unwrap();
    (dir, engine, conn)
}

fn insert_node(
    conn: &Connection,
    file_path: &str,
    name: &str,
    kind: &str,
    signature: &str,
    start_line: u32,
    end_line: u32,
) -> i64 {
    conn.execute(
        "INSERT INTO dkg_nodes (file_path, name, kind, signature, start_line, end_line) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![file_path, name, kind, signature, start_line, end_line],
    )
    .unwrap();
    conn.last_insert_rowid()
}

fn insert_edge(conn: &Connection, source_id: i64, target_id: i64, edge_type: &str) {
    conn.execute(
        "INSERT INTO dkg_edges (source_id, target_id, edge_type) VALUES (?1, ?2, ?3)",
        rusqlite::params![source_id, target_id, edge_type],
    )
    .unwrap();
}

// ---------------------------------------------------------------------------
// Functional tests (7)
// ---------------------------------------------------------------------------

#[test]
fn parse_rust_file_extracts_nodes() {
    let dir = tempfile::TempDir::new().unwrap();
    let db = dir.path().join("test.db");
    let mut engine = DkgEngine::new(&db, None, 0i64).unwrap();
    let src_dir = dir.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(
        src_dir.join("lib.rs"),
        "pub fn hello() -> u32 { 42 }\npub struct Foo;\n",
    )
    .unwrap();

    // build_graph is a todo!() stub — this will panic (TDD red)
    let result = engine.build_graph(dir.path());
    assert!(result.is_ok(), "build_graph should succeed");
    let nodes = engine.nodes().unwrap();
    assert!(nodes.len() >= 2, "expected at least 2 nodes from lib.rs");
}

#[test]
fn query_by_function_name() {
    let (_dir, _engine, conn) = make_engine();
    let id = insert_node(&conn, "src/lib.rs", "my_func", "function", "fn my_func()", 1, 5);
    let found = _engine.node_by_id(id).unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().symbol_name, "my_func");
}

#[test]
fn query_by_struct_name() {
    let (_dir, _engine, conn) = make_engine();
    let id = insert_node(&conn, "src/lib.rs", "MyStruct", "struct", "struct MyStruct", 1, 3);
    let found = _engine.node_by_id(id).unwrap();
    assert!(found.is_some());
    let info = found.unwrap();
    assert_eq!(info.symbol_name, "MyStruct");
    assert_eq!(info.kind, SymbolKind::Struct);
}

#[test]
fn signature_populated_correctly() {
    let (_dir, _engine, conn) = make_engine();
    let sig = "fn compute(x: i32, y: i32) -> i64";
    let id = insert_node(&conn, "src/math.rs", "compute", "function", sig, 10, 20);
    let info = _engine.node_by_id(id).unwrap().unwrap();
    assert_eq!(info.signature, sig);
    assert_eq!(info.start_line, 10);
    assert_eq!(info.end_line, 20);
}

#[test]
fn incremental_update_changes_hash_and_stale() {
    let dir = tempfile::TempDir::new().unwrap();
    let db = dir.path().join("test.db");
    let mut engine = DkgEngine::new(&db, None, 0i64).unwrap();
    let src_dir = dir.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("lib.rs"), "pub fn v1() {}\n").unwrap();

    // build_graph is todo!() — TDD red (panics)
    let _r1 = engine.build_graph(dir.path());
    assert!(_r1.is_ok(), "first build should succeed");

    std::fs::write(src_dir.join("lib.rs"), "pub fn v2() {}\n").unwrap();
    let _r2 = engine.build_graph(dir.path());
    assert!(_r2.is_ok(), "incremental build should succeed");
    let nodes = engine.nodes().unwrap();
    assert!(
        nodes.iter().any(|n| n.symbol_name == "v2"),
        "v2 should appear after incremental update"
    );
}

#[test]
fn concept_column_null_by_default() {
    let (_dir, _engine, conn) = make_engine();
    let id = insert_node(&conn, "src/lib.rs", "foo", "function", "fn foo()", 1, 2);
    let concept: Option<String> = conn
        .query_row("SELECT concept FROM dkg_nodes WHERE id = ?1", [id], |row| {
            row.get(0)
        })
        .unwrap();
    assert!(concept.is_none(), "concept should be NULL by default");
}

#[test]
fn file_hash_round_trip_matches_blake2s() {
    let dir = tempfile::TempDir::new().unwrap();
    let db = dir.path().join("test.db");
    let mut engine = DkgEngine::new(&db, None, 0i64).unwrap();
    let src_dir = dir.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    let content = b"fn deterministic() {}";
    std::fs::write(src_dir.join("lib.rs"), content).unwrap();

    // build_graph is todo!() — TDD red (panics)
    let result = engine.build_graph(dir.path());
    assert!(result.is_ok());

    use blake2::{Blake2s256, Digest};
    let mut hasher = Blake2s256::new();
    hasher.update(content);
    let expected_hash = hasher.finalize();

    let conn = Connection::open(dir.path().join("test.db")).unwrap();
    let stored_hash: Vec<u8> = conn
        .query_row(
            "SELECT file_hash FROM dkg_files WHERE file_path LIKE '%lib.rs'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(stored_hash, expected_hash.as_slice());
}

// ---------------------------------------------------------------------------
// Edge type tests (4)
// ---------------------------------------------------------------------------

#[test]
fn impl_contains_edge_to_methods() {
    let (_dir, engine, conn) = make_engine();
    let impl_id = insert_node(&conn, "src/lib.rs", "impl Foo", "impl", "impl Foo", 1, 20);
    let method_id = insert_node(&conn, "src/lib.rs", "bar", "function", "fn bar(&self)", 2, 10);
    insert_edge(&conn, impl_id, method_id, EDGE_CONTAINS);

    let outbound = engine.outbound_edges(impl_id).unwrap();
    assert!(
        outbound.contains(&(method_id, EDGE_CONTAINS.to_string())),
        "impl should have 'contains' edge to method"
    );
}

#[test]
fn use_declaration_imports_edge() {
    let (_dir, engine, conn) = make_engine();
    let mod_id = insert_node(&conn, "src/lib.rs", "mod_a", "module", "mod mod_a", 1, 5);
    let use_id = insert_node(&conn, "src/lib.rs", "Foo", "struct", "use mod_a::Foo", 6, 6);
    insert_edge(&conn, use_id, mod_id, EDGE_IMPORTS);

    let outbound = engine.outbound_edges(use_id).unwrap();
    assert!(
        outbound.contains(&(mod_id, EDGE_IMPORTS.to_string())),
        "use declaration should have 'imports' edge"
    );
}

#[test]
fn impl_trait_implements_edge() {
    let (_dir, engine, conn) = make_engine();
    let trait_id = insert_node(&conn, "src/lib.rs", "MyTrait", "trait", "trait MyTrait", 1, 5);
    let impl_id = insert_node(
        &conn,
        "src/lib.rs",
        "impl MyTrait for Foo",
        "impl",
        "impl MyTrait for Foo",
        6,
        15,
    );
    insert_edge(&conn, impl_id, trait_id, EDGE_IMPLEMENTS);

    let outbound = engine.outbound_edges(impl_id).unwrap();
    assert!(
        outbound.contains(&(trait_id, EDGE_IMPLEMENTS.to_string())),
        "impl should have 'implements' edge to trait"
    );
}

#[test]
fn edge_direction_outbound_not_reverse() {
    let (_dir, engine, conn) = make_engine();
    let a = insert_node(&conn, "src/a.rs", "a_fn", "function", "fn a_fn()", 1, 2);
    let b = insert_node(&conn, "src/b.rs", "b_fn", "function", "fn b_fn()", 1, 2);
    insert_edge(&conn, a, b, EDGE_IMPORTS);

    let outbound_a = engine.outbound_edges(a).unwrap();
    let outbound_b = engine.outbound_edges(b).unwrap();
    let inbound_b = engine.inbound_edges(b).unwrap();
    let inbound_a = engine.inbound_edges(a).unwrap();

    assert!(outbound_a.contains(&(b, EDGE_IMPORTS.to_string())));
    assert!(outbound_b.is_empty(), "b has no outbound edges");
    assert!(inbound_b.contains(&(a, EDGE_IMPORTS.to_string())));
    assert!(inbound_a.is_empty(), "a has no inbound edges");
}

// ---------------------------------------------------------------------------
// Downstream API tests (5)
// ---------------------------------------------------------------------------

#[test]
fn edges_returns_all_with_correct_types() {
    let (_dir, engine, conn) = make_engine();
    let n1 = insert_node(&conn, "src/lib.rs", "n1", "function", "fn n1()", 1, 2);
    let n2 = insert_node(&conn, "src/lib.rs", "n2", "struct", "struct n2", 3, 4);
    let n3 = insert_node(&conn, "src/lib.rs", "n3", "function", "fn n3()", 5, 6);
    insert_edge(&conn, n1, n2, EDGE_CONTAINS);
    insert_edge(&conn, n2, n3, EDGE_IMPORTS);

    let all_edges = engine.edges().unwrap();
    assert_eq!(all_edges.len(), 2);
    assert!(all_edges.contains(&(n1, n2, EDGE_CONTAINS.to_string())));
    assert!(all_edges.contains(&(n2, n3, EDGE_IMPORTS.to_string())));
}

#[test]
fn outbound_edges_returns_only_outbound() {
    let (_dir, engine, conn) = make_engine();
    let a = insert_node(&conn, "src/a.rs", "a", "function", "fn a()", 1, 2);
    let b = insert_node(&conn, "src/a.rs", "b", "function", "fn b()", 3, 4);
    let c = insert_node(&conn, "src/a.rs", "c", "function", "fn c()", 5, 6);
    insert_edge(&conn, a, b, EDGE_CONTAINS);
    insert_edge(&conn, c, a, EDGE_IMPORTS);

    let outbound = engine.outbound_edges(a).unwrap();
    assert_eq!(outbound.len(), 1);
    assert_eq!(outbound[0], (b, EDGE_CONTAINS.to_string()));
}

#[test]
fn inbound_edges_returns_only_inbound() {
    let (_dir, engine, conn) = make_engine();
    let a = insert_node(&conn, "src/a.rs", "a", "function", "fn a()", 1, 2);
    let b = insert_node(&conn, "src/a.rs", "b", "function", "fn b()", 3, 4);
    let c = insert_node(&conn, "src/a.rs", "c", "function", "fn c()", 5, 6);
    insert_edge(&conn, a, b, EDGE_CONTAINS);
    insert_edge(&conn, c, a, EDGE_IMPORTS);

    let inbound = engine.inbound_edges(a).unwrap();
    assert_eq!(inbound.len(), 1);
    assert_eq!(inbound[0], (c, EDGE_IMPORTS.to_string()));
}

#[test]
fn nodes_iteration_covers_all_with_fields() {
    let (_dir, engine, conn) = make_engine();
    insert_node(&conn, "src/a.rs", "alpha", "function", "fn alpha()", 1, 5);
    insert_node(&conn, "src/b.rs", "Beta", "struct", "struct Beta", 1, 10);
    insert_node(&conn, "src/c.rs", "Gamma", "trait", "trait Gamma", 1, 15);

    let nodes = engine.nodes().unwrap();
    assert_eq!(nodes.len(), 3);

    let alpha = nodes.iter().find(|n| n.symbol_name == "alpha").unwrap();
    assert_eq!(alpha.kind, SymbolKind::Function);
    assert_eq!(alpha.start_line, 1);
    assert_eq!(alpha.end_line, 5);
    assert_eq!(alpha.signature, "fn alpha()");

    let beta = nodes.iter().find(|n| n.symbol_name == "Beta").unwrap();
    assert_eq!(beta.kind, SymbolKind::Struct);

    let gamma = nodes.iter().find(|n| n.symbol_name == "Gamma").unwrap();
    assert_eq!(gamma.kind, SymbolKind::Trait);
}

#[test]
fn failed_files_empty_when_all_parse_ok() {
    let (_dir, engine, _conn) = make_engine();
    let failed = engine.failed_files().unwrap();
    assert!(failed.is_empty(), "no failed files expected on fresh db");
}

// ---------------------------------------------------------------------------
// Parse failure tests (3)
// ---------------------------------------------------------------------------

#[test]
fn empty_file_zero_nodes_no_panic() {
    let dir = tempfile::TempDir::new().unwrap();
    let db = dir.path().join("test.db");
    let mut engine = DkgEngine::new(&db, None, 0i64).unwrap();
    let src_dir = dir.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("empty.rs"), "").unwrap();

    // build_graph is todo!() — TDD red (panics)
    let result = engine.build_graph(dir.path());
    assert!(result.is_ok(), "empty file should not cause panic");
    let nodes = engine.nodes().unwrap();
    let empty_nodes: Vec<_> = nodes
        .iter()
        .filter(|n| n.file_path.to_string_lossy().contains("empty.rs"))
        .collect();
    assert!(empty_nodes.is_empty(), "empty file should produce zero nodes");
}

#[test]
fn syntax_error_partial_no_crash_failed_status() {
    let dir = tempfile::TempDir::new().unwrap();
    let db = dir.path().join("test.db");
    let mut engine = DkgEngine::new(&db, None, 0i64).unwrap();
    let src_dir = dir.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("bad.rs"), "fn broken( { struct Incomplete\n").unwrap();

    // build_graph is todo!() — TDD red (panics)
    let result = engine.build_graph(dir.path());
    assert!(result.is_ok(), "syntax error should not crash build_graph");
    let failed = engine.failed_files().unwrap();
    assert!(
        failed.iter().any(|f| f.contains("bad.rs")),
        "bad.rs should be in failed_files"
    );
}

#[test]
fn binary_file_zero_nodes_graceful_failed_status() {
    let dir = tempfile::TempDir::new().unwrap();
    let db = dir.path().join("test.db");
    let mut engine = DkgEngine::new(&db, None, 0i64).unwrap();
    let src_dir = dir.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(
        src_dir.join("binary.rs"),
        &[0x00, 0xFF, 0xFE, 0x89, 0x50, 0x4E, 0x47],
    )
    .unwrap();

    // build_graph is todo!() — TDD red (panics)
    let result = engine.build_graph(dir.path());
    assert!(result.is_ok(), "binary file should not crash");
}

// ---------------------------------------------------------------------------
// Readiness tests (3)
// ---------------------------------------------------------------------------

#[test]
fn dkg_readiness_unavailable_before_build_graph() {
    // Spec: dkg_readiness() returns Unavailable before build_graph() is called.
    // Current impl returns Empty for 0-node DB — this test will FAIL (TDD red).
    // 1A.4 must fix impl to return Unavailable for never-built state.
    let (_dir, engine, _conn) = make_engine();
    assert_eq!(engine.dkg_readiness(), DkgReadiness::Unavailable);
}

#[test]
fn dkg_readiness_ready_after_successful_build() {
    let (_dir, engine, conn) = make_engine();
    insert_node(&conn, "src/lib.rs", "foo", "function", "fn foo()", 1, 2);
    assert_eq!(engine.dkg_readiness(), DkgReadiness::Ready);
}

#[test]
fn dkg_readiness_empty_after_empty_directory_build() {
    let dir = tempfile::TempDir::new().unwrap();
    let db = dir.path().join("test.db");
    let mut engine = DkgEngine::new(&db, None, 0i64).unwrap();
    let src_dir = dir.path().join("empty_project");
    std::fs::create_dir_all(&src_dir).unwrap();

    // build_graph is todo!() — TDD red (panics)
    let result = engine.build_graph(&src_dir);
    assert!(result.is_ok());
    assert_eq!(engine.dkg_readiness(), DkgReadiness::Empty);
}

// ---------------------------------------------------------------------------
// Build-graph integration tests (3)
// ---------------------------------------------------------------------------

#[test]
fn build_graph_multi_file_populates_all() {
    let dir = tempfile::TempDir::new().unwrap();
    let db = dir.path().join("test.db");
    let mut engine = DkgEngine::new(&db, None, 0i64).unwrap();
    let src_dir = dir.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("a.rs"), "pub fn a_fn() {}\n").unwrap();
    std::fs::write(src_dir.join("b.rs"), "pub struct B;\n").unwrap();

    // build_graph is todo!() — TDD red (panics)
    let result = engine.build_graph(dir.path());
    assert!(result.is_ok());
    let graph = result.unwrap();
    assert!(graph.files_parsed >= 2, "both files should be parsed");
    assert!(graph.node_count >= 2, "at least one node per file");
}

#[test]
fn build_graph_records_file_hash_in_dkg_files() {
    let dir = tempfile::TempDir::new().unwrap();
    let db = dir.path().join("test.db");
    let mut engine = DkgEngine::new(&db, None, 0i64).unwrap();
    let src_dir = dir.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("lib.rs"), "pub fn tracked() {}\n").unwrap();

    // build_graph is todo!() — TDD red (panics)
    let result = engine.build_graph(dir.path());
    assert!(result.is_ok());

    let conn2 = Connection::open(dir.path().join("test.db")).unwrap();
    let count: i64 = conn2
        .query_row("SELECT COUNT(*) FROM dkg_files", [], |row| row.get(0))
        .unwrap();
    assert!(count >= 1, "dkg_files should have at least one entry after build");
}

#[test]
fn build_graph_sets_parse_status_ok_for_valid_files() {
    let dir = tempfile::TempDir::new().unwrap();
    let db = dir.path().join("test.db");
    let mut engine = DkgEngine::new(&db, None, 0i64).unwrap();
    let src_dir = dir.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("valid.rs"), "pub fn valid() { 1 + 1; }\n").unwrap();

    // build_graph is todo!() — TDD red (panics)
    let result = engine.build_graph(dir.path());
    assert!(result.is_ok());

    let conn2 = Connection::open(dir.path().join("test.db")).unwrap();
    let status: String = conn2
        .query_row(
            "SELECT parse_status FROM dkg_files WHERE file_path LIKE '%valid.rs'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(status, "ok", "valid file should have parse_status='ok'");
}

// ---------------------------------------------------------------------------
// Isolation test (1)
// ---------------------------------------------------------------------------

#[test]
fn two_engines_different_db_paths_no_shared_state() {
    let dir1 = tempfile::TempDir::new().unwrap();
    let dir2 = tempfile::TempDir::new().unwrap();
    let db1 = dir1.path().join("a.db");
    let db2 = dir2.path().join("b.db");

    let engine1 = DkgEngine::new(&db1, None, 0i64).unwrap();
    let engine2 = DkgEngine::new(&db2, None, 0i64).unwrap();

    let conn1 = Connection::open(&db1).unwrap();
    insert_node(&conn1, "src/a.rs", "only_in_1", "function", "fn only_in_1()", 1, 2);

    let nodes1 = engine1.nodes().unwrap();
    let nodes2 = engine2.nodes().unwrap();
    assert_eq!(nodes1.len(), 1);
    assert!(nodes2.is_empty(), "engine2 must not see engine1 data");
}

// ---------------------------------------------------------------------------
// Property-based test (1)
// ---------------------------------------------------------------------------

mod proptest_suite {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn proptest_random_rust_snippets_valid_invariants(
            name in "[a-z_][a-z0-9_]{0,20}",
            kind in prop_oneof![
                Just("function"),
                Just("struct"),
                Just("enum"),
                Just("trait"),
                Just("impl"),
            ],
            start in 1u32..1000,
            span in 1u32..500,
        ) {
            let end = start + span;
            let dir = tempfile::TempDir::new().unwrap();
            let db = dir.path().join("prop.db");
            let engine = DkgEngine::new(&db, None, 0i64).unwrap();
            let conn = Connection::open(&db).unwrap();
            let sig = format!("{kind} {name}");
            let id = insert_node(&conn, "src/prop.rs", &name, kind, &sig, start, end);

            let info = engine.node_by_id(id).unwrap().unwrap();
            prop_assert_eq!(&info.symbol_name, &name);
            prop_assert_eq!(info.start_line, start);
            prop_assert_eq!(info.end_line, end);
            prop_assert_eq!(&info.signature, &sig);
            prop_assert!(info.node_id > 0);
        }
    }
}
