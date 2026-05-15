// B-30 Pass D: integration tests for CSS @import extraction.
//
// Tests the end-to-end path: DkgEngine parses a .svelte file with multiple
// @import directives in its <style> block and produces exactly the right set
// of DkgNodes — relative paths in, http URLs out.

use elai_knowledge::dkg::DkgEngine;
use rusqlite::Connection;

fn fresh_svelte_project(source: &str) -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let dir = tempfile::TempDir::new().unwrap();
    let project = dir.path().join("project");
    std::fs::create_dir_all(&project).unwrap();
    let file = project.join("Component.svelte");
    std::fs::write(&file, source).unwrap();
    let db_path = dir.path().join("dkg.db");
    (dir, project, db_path)
}

#[test]
fn test_css_at_import_extraction() {
    // Three @import directives:
    //   1. Double-quoted relative path  -> should be extracted
    //   2. url()-wrapped single-quoted  -> should be extracted
    //   3. https:// URL                 -> must be excluded
    let fixture = concat!(
        "<script lang=\"ts\">\nfunction noop() {}\n</script>\n",
        "<style>\n",
        "@import \"./tokens.css\";\n",
        "@import url('./layout.css');\n",
        "@import 'https://cdn.example.com/reset.css';\n",
        "</style>\n"
    );
    let (_dir, project, db_path) = fresh_svelte_project(fixture);

    let mut engine = DkgEngine::new(&db_path, None, 0i64).unwrap();
    engine.build_graph(&project).unwrap();

    let conn = Connection::open(&db_path).unwrap();
    let mut stmt = conn
        .prepare(
            "SELECT name, signature FROM dkg_nodes \
             WHERE file_path LIKE '%Component.svelte' AND kind='module'",
        )
        .unwrap();
    let css_nodes: Vec<(String, String)> = stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    // Exactly 2 relative imports should be extracted (https:// excluded).
    assert_eq!(
        css_nodes.len(),
        2,
        "expected exactly 2 css:@import nodes (https excluded), got: {:?}",
        css_nodes
    );

    assert!(
        css_nodes.iter().any(|(name, sig)| name == "./tokens.css" && sig.starts_with("css:@import")),
        "expected node './tokens.css', got: {:?}",
        css_nodes
    );
    assert!(
        css_nodes.iter().any(|(name, sig)| name == "./layout.css" && sig.starts_with("css:@import")),
        "expected node './layout.css' (url()-wrapped), got: {:?}",
        css_nodes
    );

    // Verify no https:// URL leaked through.
    assert!(
        !css_nodes.iter().any(|(name, _)| name.starts_with("https:")),
        "https:// URL must not be extracted, got: {:?}",
        css_nodes
    );
}
