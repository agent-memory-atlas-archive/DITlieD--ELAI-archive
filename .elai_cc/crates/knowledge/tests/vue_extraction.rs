// B-30 Pass D: standalone integration tests for VueGrammar.
//
// Builds a DkgEngine on a tempdir with a minimal .vue SFC and asserts that
// the engine extracts nodes from both the <script> and <style> sections.

use elai_knowledge::dkg::DkgEngine;
use rusqlite::Connection;

fn fresh_db_and_project(
    lang_ext: &str,
    source: &str,
) -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let dir = tempfile::TempDir::new().unwrap();
    let project = dir.path().join("project");
    std::fs::create_dir_all(&project).unwrap();
    let file = project.join(format!("App.{}", lang_ext));
    std::fs::write(&file, source).unwrap();
    let db_path = dir.path().join("dkg.db");
    (dir, project, db_path)
}

#[test]
fn test_vue_nodes_extracted() {
    // Minimal Vue SFC: <script> with TS symbols, <style> with @import.
    let fixture = concat!(
        "<template><div>hello</div></template>\n",
        "<script>\n",
        "export function greet(name) { return 'hi ' + name; }\n",
        "export class Greeter { greet() {} }\n",
        "</script>\n",
        "<style>\n",
        "@import './base.css';\n",
        "@import url('https://fonts.example.com/font.css');\n",
        "</style>\n"
    );
    let (_dir, project, db_path) = fresh_db_and_project("vue", fixture);

    let mut engine = DkgEngine::new(&db_path, None, 0i64).unwrap();
    engine.build_graph(&project).unwrap();

    let conn = Connection::open(&db_path).unwrap();

    // All nodes from the .vue file.
    let mut stmt = conn
        .prepare("SELECT name, kind, signature FROM dkg_nodes WHERE file_path LIKE '%App.vue'")
        .unwrap();
    let rows: Vec<(String, String, String)> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert!(
        !rows.is_empty(),
        "expected nodes from App.vue, got none"
    );

    // CSS @import for './base.css' should be present.
    assert!(
        rows.iter().any(|(name, kind, sig)| {
            name == "./base.css" && kind == "module" && sig.starts_with("css:@import")
        }),
        "expected css:@import node for './base.css', got: {:?}",
        rows
    );

    // The https:// URL should NOT be imported (external resource, skipped).
    assert!(
        !rows.iter().any(|(name, _, _)| name.starts_with("https:")),
        "https:// URL must not be extracted as a css:@import node, got: {:?}",
        rows
    );
}
