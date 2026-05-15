// B-30 Pass D: integration test for Go import resolution producing cross-edges.
//
// WARN 1 fix verification: GoModuleResolver returns a directory PathBuf but the
// consumer in resolve_multi_module_imports now enumerates .go files in that
// directory. This test asserts that at least one 'imports' edge appears between
// a Go module node in test.go and a module node in the resolved pkg/util/util.go.

use elai_knowledge::dkg::DkgEngine;
use rusqlite::Connection;
use std::path::Path;

fn write(dir: &Path, rel: &str, content: &str) {
    let full = dir.join(rel);
    std::fs::create_dir_all(full.parent().unwrap()).unwrap();
    std::fs::write(full, content).unwrap();
}

#[test]
fn test_go_cross_edge_after_dir_resolve_fix() {
    let tmp = tempfile::TempDir::new().unwrap();
    let project = tmp.path().join("goproject");
    std::fs::create_dir_all(&project).unwrap();

    // go.mod declares the module root.
    write(
        &project,
        "go.mod",
        "module github.com/test\n\ngo 1.21\n",
    );

    // The imported package.
    write(
        &project,
        "pkg/util/util.go",
        "package util\n\nfunc Helper() string { return \"help\" }\n",
    );

    // The importing file — uses the full module path.
    write(
        &project,
        "main.go",
        "package main\n\nimport \"github.com/test/pkg/util\"\n\nfunc main() { _ = util.Helper() }\n",
    );

    let db_path = tmp.path().join("dkg.db");
    let mut engine = DkgEngine::new(&db_path, None, 0i64).unwrap();
    engine.build_graph(&project).unwrap();

    let conn = Connection::open(&db_path).unwrap();

    // Verify at least one 'imports' edge exists — i.e. the Go resolver produced
    // a cross-edge from main.go's import node to util.go's module node.
    let edge_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM dkg_edges WHERE edge_type = 'imports'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    assert!(
        edge_count >= 1,
        "expected at least 1 'imports' edge for Go cross-module import, got {edge_count}"
    );
}
