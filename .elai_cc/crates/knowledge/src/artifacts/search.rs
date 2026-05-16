//! Artifact search — Phase A: in-memory cosine fallback.
//!
//! Phase A loads all stored embeddings from SQLite, computes cosine similarity
//! in-process, sorts, and returns the top `limit` results. This is O(n) but
//! correct for the expected artifact count (<100 chunks).
//!
//! Phase B will wire HNSW (usearch) + tantivy for hybrid BM25+vector search.
//!
//! When no embeddings are stored (embedding=None), or when `vm` returns an
//! error, the function returns an empty result set (not an error).

use super::manifest::ArtifactError;
use super::store::ArtifactStore;
use super::types::ArtifactSearchResult;
use crate::vector::VectorModule;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Minimum cosine similarity score to include in results.
/// Chunks below this threshold are filtered out even if they rank in top-N.
pub const BM25_FALLBACK_SCORE_THRESHOLD: f32 = 0.10;

// ---------------------------------------------------------------------------
// artifact_search
// ---------------------------------------------------------------------------

/// Search across context artifact chunks.
///
/// `artifact_name = Some(name)` restricts the search to one artifact.
/// `limit` caps the result count.
///
/// Returns an empty `Vec` (not an error) when:
/// - the store is empty
/// - the query cannot be embedded (no VectorModule, or embed_text error)
/// - no chunk meets the score threshold
pub fn artifact_search(
    store: &ArtifactStore,
    vm: &dyn VectorModule,
    query: &str,
    artifact_name: Option<&str>,
    limit: usize,
) -> Result<Vec<ArtifactSearchResult>, ArtifactError> {
    if store.is_empty()? {
        return Ok(vec![]);
    }

    let query_vec = match vm.embed_text(query) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(error = %e, "artifact_search: embed_text failed — returning empty");
            return Ok(vec![]);
        }
    };

    artifact_search_with_precomputed_embedding(store, &query_vec, artifact_name, limit)
}

/// Search across context artifact chunks using a pre-computed query embedding.
///
/// Separating embedding from cosine scan allows async callers to compute the
/// embedding in the async context (via `vm.embed_text`) and then offload the
/// synchronous SQLite I/O + cosine scan to `tokio::task::spawn_blocking`.
///
/// `query_embedding` must match the dimensionality of stored chunk embeddings.
/// Chunks with a different dimensionality are silently skipped (score = 0.0
/// which falls below `BM25_FALLBACK_SCORE_THRESHOLD`).
pub fn artifact_search_with_precomputed_embedding(
    store: &ArtifactStore,
    query_embedding: &[f32],
    artifact_name: Option<&str>,
    limit: usize,
) -> Result<Vec<ArtifactSearchResult>, ArtifactError> {
    if store.is_empty()? {
        return Ok(vec![]);
    }

    let chunks = match artifact_name {
        Some(name) => store.list_artifact_chunks(name)?,
        None => store.list_all_chunks()?,
    };

    let mut scored: Vec<ArtifactSearchResult> = chunks
        .into_iter()
        .filter_map(|chunk| {
            let emb = chunk.embedding.clone()?;
            let score = cosine_similarity(query_embedding, &emb);
            Some(ArtifactSearchResult { chunk, score })
        })
        .filter(|r| r.score >= BM25_FALLBACK_SCORE_THRESHOLD)
        .collect();

    // Sort descending by score, then stable by artifact_name+relative_path for
    // deterministic output on ties.
    scored.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a.chunk
                    .artifact_name
                    .cmp(&b.chunk.artifact_name)
                    .then_with(|| a.chunk.relative_path.cmp(&b.chunk.relative_path))
            })
    });

    scored.truncate(limit);
    Ok(scored)
}

// ---------------------------------------------------------------------------
// Cosine similarity
// ---------------------------------------------------------------------------

/// Compute cosine similarity between two equal-length vectors.
///
/// Returns 0.0 when either vector has zero magnitude (avoids divide-by-zero).
/// Clamps result to [0.0, 1.0] to guard against floating-point edge cases.
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }

    (dot / (mag_a * mag_b)).clamp(0.0, 1.0)
}
