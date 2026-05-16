//! ArtifactStage — B-35 Phase B, Task 8.
//!
//! A QueryPipeline stage that searches context artifacts (non-code project
//! knowledge: schemas, OpenAPI specs, k8s manifests, ADRs, etc.) and maps
//! results to `RetrievalResult` for pipeline fusion.
//!
//! The stage is skipped for FAST tier queries (no vector search budget).
//! For STANDARD and DEEP tiers it runs after the catrag stage.
//!
//! Internally calls `artifact_search` (sync, in-process cosine scan) which is
//! safe to call directly from a sync context.  When called from an async
//! context, the caller (pipeline.rs) must be aware this is blocking; the
//! pipeline's `query_with_tier` is synchronous so no `spawn_blocking` is
//! needed here.

use std::sync::Arc;

use super::manifest::ArtifactError;
use super::search::artifact_search;
use super::store::ArtifactStore;
use crate::arag::RetrievalResult;
use crate::vector::VectorModule;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default maximum number of artifact chunks to include per pipeline query.
pub const DEFAULT_ARTIFACT_LIMIT: usize = 10;

/// Default minimum cosine score (mirrors `BM25_FALLBACK_SCORE_THRESHOLD` from
/// `search.rs`, exposed here so pipeline builder can override without depending
/// on the search module's constant).
pub const DEFAULT_ARTIFACT_SCORE_THRESHOLD: f32 = 0.10;

/// `tool_used` label placed on `RetrievalResult` rows from this stage so
/// downstream RRF and dashboard can identify artifact-sourced results.
const STAGE_TOOL_LABEL: &str = "artifacts";

// ---------------------------------------------------------------------------
// ArtifactStage
// ---------------------------------------------------------------------------

/// Pipeline stage that searches context artifact chunks and returns
/// `Vec<RetrievalResult>` for RRF fusion with BM25/vector results.
///
/// Build via `QueryPipeline::with_artifact_stage`.
pub struct ArtifactStage {
    store: ArtifactStore,
    vm: Arc<dyn VectorModule + Send + Sync>,
    limit: usize,
    /// Minimum cosine similarity to pass results through. Chunks below this
    /// threshold are excluded even if they rank in the top-N.
    score_threshold: f32,
}

impl ArtifactStage {
    /// Construct a new `ArtifactStage`.
    ///
    /// - `store`: the per-project artifact store (sync; cheap to clone).
    /// - `vm`: embedding module shared with the rest of the pipeline.
    /// - `limit`: max results to return (capped internally by `artifact_search`).
    /// - `score_threshold`: minimum cosine similarity (0.0–1.0).
    pub fn new(
        store: ArtifactStore,
        vm: Arc<dyn VectorModule + Send + Sync>,
        limit: usize,
        score_threshold: f32,
    ) -> Self {
        Self {
            store,
            vm,
            limit,
            score_threshold,
        }
    }

    /// Run the artifact search for a query, optionally restricted to one
    /// artifact by name, and map the results to `RetrievalResult` rows.
    ///
    /// Returns an empty `Vec` (not an error) when:
    /// - the store is empty
    /// - the query cannot be embedded (VectorModule unavailable / error)
    /// - no chunk meets `score_threshold`
    pub fn run(
        &self,
        query: &str,
        artifact_name: Option<&str>,
    ) -> Result<Vec<RetrievalResult>, ArtifactError> {
        let raw = artifact_search(&self.store, self.vm.as_ref(), query, artifact_name, self.limit)?;

        let results = raw
            .into_iter()
            .filter(|r| r.score >= self.score_threshold)
            .map(|r| RetrievalResult {
                chunk_id: r.chunk.id.clone(),
                snippet: r.chunk.content.clone(),
                score: r.score as f64,
                tool_used: STAGE_TOOL_LABEL.to_string(),
            })
            .collect();

        Ok(results)
    }
}

impl std::fmt::Debug for ArtifactStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArtifactStage")
            .field("limit", &self.limit)
            .field("score_threshold", &self.score_threshold)
            .finish()
    }
}
