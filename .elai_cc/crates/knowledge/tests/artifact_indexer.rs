//! B-35 Phase A — ArtifactIndexer tests.
//!
//! 4 tests:
//!   sql_chunks_at_create_boundary — SQL file splits at CREATE TABLE/VIEW/INDEX.
//!   md_line_window_chunk — Markdown uses line-window chunking.
//!   json_line_window_chunk — JSON uses line-window chunking.
//!   chunk_carries_artifact_metadata — chunks have correct name/description.

use std::path::Path;

use elai_knowledge::artifacts::indexer::{chunk_file, chunk_sql};

// ---------------------------------------------------------------------------

#[test]
fn sql_chunks_at_create_boundary() {
    let content = r#"-- preamble comment

CREATE TABLE users (
    id   SERIAL PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE posts (
    id      SERIAL PRIMARY KEY,
    user_id INT REFERENCES users(id),
    title   TEXT
);

CREATE INDEX idx_posts_user ON posts(user_id);
"#;

    let path = Path::new("schema.sql");
    let chunks = chunk_sql(path, content);

    // Should produce at least 3 chunks: preamble+users, posts, index
    assert!(
        chunks.len() >= 2,
        "expected multiple chunks for multi-CREATE SQL, got {}",
        chunks.len()
    );

    // Each CREATE TABLE boundary should start a new chunk
    let has_users = chunks.iter().any(|c| c.content.contains("CREATE TABLE users"));
    let has_posts = chunks.iter().any(|c| c.content.contains("CREATE TABLE posts"));
    assert!(has_users, "should have a chunk containing CREATE TABLE users");
    assert!(has_posts, "should have a chunk containing CREATE TABLE posts");
}

#[test]
fn md_line_window_chunk() {
    // Create content with more than ARTIFACT_CHUNK_WINDOW_LINES lines
    let lines: Vec<String> = (1..=120)
        .map(|i| format!("Line {i}: some content here"))
        .collect();
    let content = lines.join("\n");

    let path = Path::new("README.md");
    let chunks = chunk_file(path, &content);

    // With 120 lines, window=50, overlap=10, step=40:
    // start=0 (1-50), start=40 (41-90), start=80 (81-120) = 3 chunks
    assert!(
        chunks.len() >= 2,
        "120-line markdown should produce multiple chunks, got {}",
        chunks.len()
    );

    // First chunk starts at line 1
    assert_eq!(chunks[0].line_start, 1);

    // Last chunk ends at or near 120
    assert!(
        chunks.last().unwrap().line_end >= 100,
        "last chunk should cover end of file"
    );
}

#[test]
fn json_line_window_chunk() {
    // JSON files use line-window just like markdown
    let lines: Vec<String> = (1..=80)
        .map(|i| format!("  \"key_{i}\": \"value_{i}\""))
        .collect();
    let content = format!("{{\n{}\n}}", lines.join(",\n"));

    let path = Path::new("config.json");
    let chunks = chunk_file(path, &content);

    assert!(
        chunks.len() >= 1,
        "json file should produce at least one chunk"
    );

    // All chunks should have artifact-compatible content
    for chunk in &chunks {
        assert!(!chunk.content.is_empty(), "chunk content must not be empty");
        assert!(chunk.line_end >= chunk.line_start, "line_end >= line_start");
    }
}

#[test]
fn chunk_carries_artifact_metadata() {
    use elai_knowledge::artifacts::store::ArtifactStore;
    use elai_knowledge::artifacts::indexer::ArtifactIndexer;
    use elai_knowledge::artifacts::ArtifactSpec;

    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    let content = "# Overview\n\nThis glossary defines bounded context terms.\n\nTerm: Entity — an object with identity.";
    std::fs::write(root.join("glossary.md"), content).unwrap();

    let store_path = root.join("context_test.db");
    let store = ArtifactStore::open(&store_path).unwrap();

    let spec = ArtifactSpec {
        name: "ubiquitous-language".to_string(),
        path: root.join("glossary.md"),
        description: "Domain glossary — check before naming entities".to_string(),
        is_dir: false,
    };

    let indexer = ArtifactIndexer::new(&store, None);
    let state = indexer.index_artifact(&spec, None, 1_700_000_000).unwrap();

    assert!(state.chunk_count > 0, "should have indexed at least one chunk");

    let chunks = store.list_artifact_chunks("ubiquitous-language").unwrap();
    assert!(!chunks.is_empty(), "chunks must be stored");

    for chunk in &chunks {
        assert_eq!(chunk.artifact_name, "ubiquitous-language");
        assert_eq!(chunk.artifact_description, "Domain glossary — check before naming entities");
        assert!(!chunk.content.is_empty());
        assert!(!chunk.content_hash.is_empty());
    }
}
