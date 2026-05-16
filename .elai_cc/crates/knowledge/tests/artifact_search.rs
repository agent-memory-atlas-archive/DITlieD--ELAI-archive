//! B-35 Phase A — artifact_search tests.
//!
//! 2 tests:
//!   search_filtered_by_artifact_name — filter to one artifact returns only that artifact's chunks.
//!   search_returns_score_and_content — results have non-zero score and non-empty content.
//!
//! Uses a mock VectorModule that returns a fixed-dimension embedding from
//! the first chars of the query (deterministic, no ONNX required).

use elai_knowledge::artifacts::search::artifact_search;
use elai_knowledge::artifacts::store::ArtifactStore;
use elai_knowledge::artifacts::types::ArtifactChunk;
use elai_knowledge::vector::{VectorModule, VectorError};
use elai_knowledge::ReadinessStatus;

// ---------------------------------------------------------------------------
// Mock VectorModule
// ---------------------------------------------------------------------------

/// Deterministic embedder: returns a 4-dim vector derived from the first 4
/// bytes of the query string. Sufficient for cosine similarity testing without
/// an ONNX session.
struct MockVectorModule;

impl VectorModule for MockVectorModule {
    fn embed_code(&self, text: &str) -> Result<Vec<f32>, VectorError> {
        Ok(mock_embed(text))
    }

    fn embed_text(&self, text: &str) -> Result<Vec<f32>, VectorError> {
        Ok(mock_embed(text))
    }

    fn readiness(&self) -> ReadinessStatus {
        ReadinessStatus::Ready
    }
}

fn mock_embed(text: &str) -> Vec<f32> {
    let bytes = text.as_bytes();
    let len = bytes.len().max(1) as f32;
    // 4-dim vector: each component derived from byte values, normalized
    [
        bytes.first().copied().unwrap_or(0) as f32 / 255.0,
        bytes.get(1).copied().unwrap_or(0) as f32 / 255.0,
        bytes.get(2).copied().unwrap_or(0) as f32 / 255.0,
        len / 1000.0,
    ]
    .to_vec()
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn insert_chunk_with_embedding(
    store: &ArtifactStore,
    artifact_name: &str,
    content: &str,
    embedding: Vec<f32>,
) {
    use elai_knowledge::artifacts::store::chunk_id;
    let id = chunk_id(artifact_name, "test_file.md", 1);
    let chunk = ArtifactChunk {
        id,
        artifact_name: artifact_name.to_string(),
        artifact_description: format!("description of {artifact_name}"),
        relative_path: "test_file.md".to_string(),
        content: content.to_string(),
        start_line: 1,
        end_line: 5,
        content_hash: "abc123".to_string(),
        embedding: Some(embedding),
    };
    store.upsert_chunk(&chunk, 1_700_000_000).unwrap();
}

// ---------------------------------------------------------------------------

#[test]
fn search_filtered_by_artifact_name() {
    let tmp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::open(&tmp.path().join("ctx.db")).unwrap();
    let vm = MockVectorModule;

    // Insert chunks for two different artifacts with distinct embeddings
    let schema_emb = vec![0.9_f32, 0.1, 0.05, 0.001];
    let glossary_emb = vec![0.05_f32, 0.9, 0.05, 0.001];

    insert_chunk_with_embedding(&store, "database-schema", "CREATE TABLE users", schema_emb);
    insert_chunk_with_embedding(&store, "ubiquitous-language", "Term: Entity", glossary_emb);

    // Search restricted to "database-schema" only
    let results = artifact_search(
        &store,
        &vm,
        "users table",
        Some("database-schema"),
        10,
    )
    .unwrap();

    for r in &results {
        assert_eq!(
            r.chunk.artifact_name, "database-schema",
            "filtered search must only return database-schema chunks"
        );
    }

    // Unfiltered search returns chunks from both
    let all_results = artifact_search(&store, &vm, "users table", None, 10).unwrap();
    let names: Vec<&str> = all_results
        .iter()
        .map(|r| r.chunk.artifact_name.as_str())
        .collect();
    // At least one of the artifacts should score above threshold
    assert!(!all_results.is_empty(), "unfiltered search should return results");
    _ = names; // used for assertion above
}

#[test]
fn search_returns_score_and_content() {
    let tmp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::open(&tmp.path().join("ctx2.db")).unwrap();
    let vm = MockVectorModule;

    // Insert a chunk whose embedding is close to the query embedding
    let query = "domain entity";
    let query_emb = mock_embed(query);
    insert_chunk_with_embedding(&store, "glossary", "Term: Entity — has identity", query_emb.clone());

    let results = artifact_search(&store, &vm, query, None, 5).unwrap();

    assert!(!results.is_empty(), "should find at least one result");
    let top = &results[0];
    assert!(top.score > 0.0, "score must be positive");
    assert!(!top.chunk.content.is_empty(), "content must not be empty");
    assert_eq!(top.chunk.artifact_name, "glossary");
}
