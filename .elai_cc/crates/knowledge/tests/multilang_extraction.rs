// B-30 Pass B — integration tests for 7 new language extractors + language column.
// B-30 Pass D — added test_vue_extraction and test_css_at_import_svelte.
//
// Each test creates a DkgEngine with a tempdir db, writes a fixture source file,
// calls build_graph(), then queries dkg_nodes via rusqlite to assert expected symbols.

use elai_knowledge::dkg::DkgEngine;
use rusqlite::Connection;

// ---------------------------------------------------------------------------
// Shared helper
// ---------------------------------------------------------------------------

/// Create a temp directory with a source file and an empty db path.
/// Returns (TempDir — keep alive!, project_dir, db_path).
fn fresh_db_and_project(
    lang_ext: &str,
    source: &str,
) -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let dir = tempfile::TempDir::new().unwrap();
    let project = dir.path().join("project");
    std::fs::create_dir_all(&project).unwrap();
    let file = project.join(format!("test.{}", lang_ext));
    std::fs::write(&file, source).unwrap();
    let db_path = dir.path().join("dkg.db");
    (dir, project, db_path)
}

/// Query all node names for the given file extension (matches file_path LIKE '%test.EXT').
fn node_names_for_ext(db_path: &std::path::Path, ext: &str) -> Vec<String> {
    let conn = Connection::open(db_path).unwrap();
    let pattern = format!("%test.{}", ext);
    let mut stmt = conn
        .prepare("SELECT name FROM dkg_nodes WHERE file_path LIKE ?1")
        .unwrap();
    stmt.query_map([&pattern], |row| row.get::<_, String>(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
}

// ---------------------------------------------------------------------------
// Test 1: JavaScript
// ---------------------------------------------------------------------------

#[test]
fn test_javascript_extraction() {
    let fixture = "function hello(x) { return x + 1; }\nclass Greeter { greet() {} }\n";
    let (_dir, project, db_path) = fresh_db_and_project("js", fixture);

    let mut engine = DkgEngine::new(&db_path, None, 0i64).unwrap();
    engine.build_graph(&project).unwrap();

    let names = node_names_for_ext(&db_path, "js");
    assert!(
        names.iter().any(|n| n == "hello"),
        "expected node 'hello' in javascript extraction, got: {:?}",
        names
    );
    assert!(
        names.iter().any(|n| n == "Greeter"),
        "expected node 'Greeter' in javascript extraction, got: {:?}",
        names
    );
}

// ---------------------------------------------------------------------------
// Test 2: C
// ---------------------------------------------------------------------------

#[test]
fn test_c_extraction() {
    let fixture =
        "#include <stdio.h>\nint add(int a, int b) { return a + b; }\nstruct Point { int x; int y; };\n";
    let (_dir, project, db_path) = fresh_db_and_project("c", fixture);

    let mut engine = DkgEngine::new(&db_path, None, 0i64).unwrap();
    engine.build_graph(&project).unwrap();

    let names = node_names_for_ext(&db_path, "c");
    assert!(
        names.iter().any(|n| n == "add"),
        "expected node 'add' in C extraction, got: {:?}",
        names
    );
    assert!(
        names.iter().any(|n| n == "Point"),
        "expected node 'Point' in C extraction, got: {:?}",
        names
    );
}

// ---------------------------------------------------------------------------
// Test 3: C++
// ---------------------------------------------------------------------------

#[test]
fn test_cpp_extraction() {
    let fixture = "#include <string>\nclass Foo { public: void bar() {} };\nvoid Foo::baz() {}\n";
    let (_dir, project, db_path) = fresh_db_and_project("cpp", fixture);

    let mut engine = DkgEngine::new(&db_path, None, 0i64).unwrap();
    engine.build_graph(&project).unwrap();

    let names = node_names_for_ext(&db_path, "cpp");
    assert!(
        names.iter().any(|n| n == "Foo"),
        "expected node 'Foo' in C++ extraction, got: {:?}",
        names
    );
    // Out-of-class method may be "baz" or "Foo::baz" depending on declarator resolution.
    assert!(
        names.iter().any(|n| n == "baz" || n == "Foo::baz"),
        "expected node 'baz' or 'Foo::baz' in C++ extraction, got: {:?}",
        names
    );
}

// ---------------------------------------------------------------------------
// Test 4: Scala
// ---------------------------------------------------------------------------

#[test]
fn test_scala_extraction() {
    let fixture =
        "class Counter {\n  def increment(): Int = 0\n}\nobject Main extends App {}\n";
    let (_dir, project, db_path) = fresh_db_and_project("scala", fixture);

    let mut engine = DkgEngine::new(&db_path, None, 0i64).unwrap();
    engine.build_graph(&project).unwrap();

    let names = node_names_for_ext(&db_path, "scala");
    assert!(
        names.iter().any(|n| n == "Counter"),
        "expected node 'Counter' in Scala extraction, got: {:?}",
        names
    );
    assert!(
        names.iter().any(|n| n == "increment"),
        "expected node 'increment' in Scala extraction, got: {:?}",
        names
    );
}

// ---------------------------------------------------------------------------
// Test 5: PHP
// ---------------------------------------------------------------------------

#[test]
fn test_php_extraction() {
    let fixture = "<?php\nfunction greet($name) { return 'Hello ' . $name; }\nclass User {\n  public function getName() { return $this->name; }\n}\n";
    let (_dir, project, db_path) = fresh_db_and_project("php", fixture);

    let mut engine = DkgEngine::new(&db_path, None, 0i64).unwrap();
    engine.build_graph(&project).unwrap();

    let names = node_names_for_ext(&db_path, "php");
    assert!(
        names.iter().any(|n| n == "greet"),
        "expected node 'greet' in PHP extraction, got: {:?}",
        names
    );
    assert!(
        names.iter().any(|n| n == "User"),
        "expected node 'User' in PHP extraction, got: {:?}",
        names
    );
}

// ---------------------------------------------------------------------------
// Test 6: Swift
// ---------------------------------------------------------------------------

#[test]
fn test_swift_extraction() {
    let fixture = "import Foundation\nfunc greet(name: String) -> String { return \"Hello \" + name }\nclass Animal { func speak() {} }\nprotocol Flyable {}\n";
    let (_dir, project, db_path) = fresh_db_and_project("swift", fixture);

    let mut engine = DkgEngine::new(&db_path, None, 0i64).unwrap();
    engine.build_graph(&project).unwrap();

    let names = node_names_for_ext(&db_path, "swift");
    assert!(
        names.iter().any(|n| n == "greet"),
        "expected node 'greet' in Swift extraction, got: {:?}",
        names
    );
    assert!(
        names.iter().any(|n| n == "Animal"),
        "expected node 'Animal' in Swift extraction, got: {:?}",
        names
    );
}

// ---------------------------------------------------------------------------
// Test 7: Bash
// ---------------------------------------------------------------------------

#[test]
fn test_bash_extraction() {
    let fixture =
        "#!/bin/bash\nfunction setup_env() {\n  export PATH=\"$HOME/bin:$PATH\"\n}\ncleanup() {\n  rm -rf /tmp/work\n}\n";
    let (_dir, project, db_path) = fresh_db_and_project("sh", fixture);

    let mut engine = DkgEngine::new(&db_path, None, 0i64).unwrap();
    engine.build_graph(&project).unwrap();

    let names = node_names_for_ext(&db_path, "sh");
    assert!(
        names.iter().any(|n| n == "setup_env"),
        "expected node 'setup_env' in Bash extraction, got: {:?}",
        names
    );
    assert!(
        names.iter().any(|n| n == "cleanup"),
        "expected node 'cleanup' in Bash extraction, got: {:?}",
        names
    );
}

// ---------------------------------------------------------------------------
// Test 8: language column populated (v4 migration)
// ---------------------------------------------------------------------------

#[test]
fn test_language_column_populated() {
    let fixture = "pub fn hello() -> u32 { 42 }\npub struct Foo;\n";
    let (_dir, project, db_path) = fresh_db_and_project("rs", fixture);

    let mut engine = DkgEngine::new(&db_path, None, 0i64).unwrap();
    engine.build_graph(&project).unwrap();

    let conn = Connection::open(&db_path).unwrap();
    let pattern = "%test.rs";
    let mut stmt = conn
        .prepare(
            "SELECT DISTINCT language FROM dkg_nodes WHERE file_path LIKE ?1 AND language IS NOT NULL",
        )
        .unwrap();
    let languages: Vec<String> = stmt
        .query_map([pattern], |row| row.get::<_, String>(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert_eq!(
        languages,
        vec!["rust"],
        "expected language column = 'rust' for .rs file, got: {:?}",
        languages
    );
}

// ---------------------------------------------------------------------------
// Test 9: Vue extraction (Pass D)
// ---------------------------------------------------------------------------

#[test]
fn test_vue_extraction() {
    // Minimal Vue SFC: one <script> block with a const declaration and one
    // <style> block with an @import.
    let fixture = "<script>\nconst greeting = 'hello';\nfunction sayHi() { return greeting; }\n</script>\n<style>\n@import \"./vars.css\";\n</style>\n";
    let (_dir, project, db_path) = fresh_db_and_project("vue", fixture);

    let mut engine = DkgEngine::new(&db_path, None, 0i64).unwrap();
    engine.build_graph(&project).unwrap();

    let names = node_names_for_ext(&db_path, "vue");
    assert!(
        !names.is_empty(),
        "expected at least 1 node from .vue file, got empty"
    );
    // The CSS @import should produce a Module node named "./vars.css".
    assert!(
        names.iter().any(|n| n == "./vars.css"),
        "expected node './vars.css' (css:@import) in vue extraction, got: {:?}",
        names
    );
}

// ---------------------------------------------------------------------------
// Test 10: CSS @import extraction from Svelte (Pass D)
// ---------------------------------------------------------------------------

#[test]
fn test_css_at_import_svelte() {
    // Svelte SFC with a <style> block containing an @import.
    let fixture = "<script lang=\"ts\">\nfunction hello() {}\n</script>\n<style>\n@import \"./variables.css\";\n</style>\n";
    let (_dir, project, db_path) = fresh_db_and_project("svelte", fixture);

    let mut engine = DkgEngine::new(&db_path, None, 0i64).unwrap();
    engine.build_graph(&project).unwrap();

    let conn = Connection::open(&db_path).unwrap();
    let pattern = "%test.svelte";
    let mut stmt = conn
        .prepare(
            "SELECT name, signature FROM dkg_nodes WHERE file_path LIKE ?1 AND kind='module'",
        )
        .unwrap();
    let module_nodes: Vec<(String, String)> = stmt
        .query_map([&pattern], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert!(
        module_nodes.iter().any(|(name, sig)| name == "./variables.css" && sig.starts_with("css:@import")),
        "expected css:@import node for './variables.css' from .svelte file, got: {:?}",
        module_nodes
    );
}
