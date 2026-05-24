//! CosmosStage — C-8 (Task #10) — pipeline integration for the cosmos artifact tier.
//!
//! A QueryPipeline stage that pulls relevant fresh cosmos artifacts and emits
//! one `RetrievalResult` per artifact body found. Inserted between
//! `ArtifactStage` and `ARagStage` for STANDARD + DEEP tiers; FAST queries
//! skip it (no embedding budget, no cross-project knowledge required for
//! grep-style lookups).
//!
//! # Routing policy
//!
//! Each query is matched against every artifact whose `manifest.name` or
//! `research_query` shares a token-overlap with the query terms (lowercased,
//! word-split, length >= 3). When at least one shared token exists, the
//! artifact is included. This is intentionally permissive: artifacts are
//! ranked downstream by RRF fusion + cross-encoder reranker, so over-recall
//! at this stage costs only a body read, not a wrong answer.
//!
//! # Failure handling
//!
//! - `cosmos_get` errors are logged via `tracing::warn!` and the artifact is
//!   silently omitted (best-effort, no panic).
//! - Empty manifest → empty `Vec` (not an error).
//! - Empty query → empty `Vec` (not an error).
//!
//! # Concurrency
//!
//! `CosmosWorker` is filesystem-backed; reads are cheap (memory-mapped
//! by the OS, no SQLite I/O). `run()` is sync because `cosmos_get` is sync.
//! Callers may invoke this from an async context — it does not block the
//! tokio runtime for any meaningful duration in practice.
//!
//! Grounded: arXiv:2403.12958 "Dated Data" + arXiv:2511.14022 (cosmos
//! provenance + per-tier-relevance design). Plan reference:
//! F:/.ELAI_workflow/docs/thefinalglance/C-plan/C-8_viewer_stub_eradication.md
//! task #10. Drift acknowledgement: C-8 plan claimed `cosmos_get(query_terms)`
//! exists — actual signature `cosmos_get(name: &str)`. This module bridges
//! the two by adding the name-router policy here.

use std::sync::Arc;

use tracing::{debug, warn};

use crate::arag::RetrievalResult;
use crate::cosmos_worker::CosmosWorker;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// `tool_used` label placed on `RetrievalResult` rows from this stage so the
/// downstream RRF fusion and dashboard can identify cosmos-sourced results.
const STAGE_TOOL_LABEL: &str = "cosmos";

/// Minimum token length considered for the name-router policy. Below this
/// threshold tokens are stopwords or punctuation noise.
const MIN_ROUTING_TOKEN_LEN: usize = 3;

/// Snippet character cap. Cosmos bodies can be 64 KiB; we cap the snippet so
/// the pipeline result rows stay bounded.
const SNIPPET_CHAR_CAP: usize = 1024;

/// Static score for cosmos results. Cosmos artifacts are curated, top-source
/// briefs; RRF fusion handles relative ordering against BM25/vector hits.
const COSMOS_BASE_SCORE: f64 = 0.75;

// ---------------------------------------------------------------------------
// CosmosStage
// ---------------------------------------------------------------------------

/// Pipeline stage that maps cosmos artifact bodies into `RetrievalResult`
/// rows for RRF fusion.
///
/// Build via `QueryPipeline::with_cosmos_stage`.
pub struct CosmosStage {
    worker: Arc<CosmosWorker>,
}

impl CosmosStage {
    /// Construct a new `CosmosStage` wrapping a shared `CosmosWorker`.
    pub fn new(worker: Arc<CosmosWorker>) -> Self {
        Self { worker }
    }

    /// Stable stage name surfaced on `PipelineStage.stage`.
    /// Free function for callers that need the label without constructing a
    /// stage instance (e.g. `pipeline.rs` push-empty-stage on FAST tier).
    pub const fn name() -> &'static str {
        STAGE_TOOL_LABEL
    }

    /// Run the cosmos artifact lookup for a query. Returns a (possibly
    /// empty) `Vec<RetrievalResult>` of matching artifact bodies.
    ///
    /// Errors from `cosmos_get` are logged and the artifact is silently
    /// omitted. The stage itself never returns an error.
    pub fn run(&self, query: &str) -> Vec<RetrievalResult> {
        let query_tokens = tokenise_for_routing(query);
        if query_tokens.is_empty() {
            return Vec::new();
        }

        let manifest = self.worker.manifest();
        if manifest.artifacts.is_empty() {
            return Vec::new();
        }

        let mut results: Vec<RetrievalResult> = Vec::with_capacity(manifest.artifacts.len());
        for spec in &manifest.artifacts {
            if !artifact_matches_query(spec, &query_tokens) {
                continue;
            }

            match self.worker.cosmos_get(&spec.name) {
                Ok(artifact) => {
                    let snippet = clip_snippet(&artifact.body, SNIPPET_CHAR_CAP);
                    debug!(
                        target: "cosmos",
                        artifact = %spec.name,
                        snippet_len = snippet.len(),
                        "cosmos_stage: artifact included in retrieval results",
                    );
                    results.push(RetrievalResult {
                        chunk_id: format!("cosmos:{}", spec.name),
                        snippet,
                        score: COSMOS_BASE_SCORE,
                        tool_used: STAGE_TOOL_LABEL.to_string(),
                    });
                }
                Err(e) => {
                    warn!(
                        target: "cosmos",
                        artifact = %spec.name,
                        error = %e,
                        "cosmos_stage: cosmos_get failed; artifact omitted (non-fatal)",
                    );
                }
            }
        }

        results
    }
}

// ---------------------------------------------------------------------------
// Internals
// ---------------------------------------------------------------------------

/// Lowercase + split-on-non-alphanumeric + drop short tokens. Mirrors the
/// permissive tokenisation used by `bm25` so cosmos-routed queries align
/// with the rest of the retrieval pipeline.
fn tokenise_for_routing(s: &str) -> Vec<String> {
    s.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.chars().count() >= MIN_ROUTING_TOKEN_LEN)
        .map(|t| t.to_string())
        .collect()
}

/// Whether a cosmos artifact matches the query token set. Matches if ANY
/// query token appears as a substring of the artifact name OR research_query
/// (case-insensitive — both sides already lowercased on the artifact side).
fn artifact_matches_query(
    spec: &crate::cosmos_worker::CosmosArtifactSpec,
    query_tokens: &[String],
) -> bool {
    let name_lc = spec.name.to_lowercase();
    let research_lc = spec.research_query.to_lowercase();
    query_tokens
        .iter()
        .any(|t| name_lc.contains(t.as_str()) || research_lc.contains(t.as_str()))
}

/// Clip a body to at most `cap` characters, preserving char boundaries so
/// we never split a multi-byte codepoint.
fn clip_snippet(body: &str, cap: usize) -> String {
    if body.chars().count() <= cap {
        return body.to_string();
    }
    let mut out = String::with_capacity(cap + 1);
    for (i, ch) in body.chars().enumerate() {
        if i >= cap {
            break;
        }
        out.push(ch);
    }
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenise_lowercases_splits_and_drops_short_tokens() {
        let toks = tokenise_for_routing("Compare iOS HIG to macOS HIG (in 2025)");
        // "ios" >= 3 chars stays; "hig" >= 3 chars stays; "to"/"in" dropped
        // because shorter than MIN_ROUTING_TOKEN_LEN (=3).
        assert!(toks.contains(&"compare".to_string()));
        assert!(toks.contains(&"ios".to_string()));
        assert!(toks.contains(&"hig".to_string()));
        assert!(toks.contains(&"macos".to_string()));
        assert!(toks.contains(&"2025".to_string()));
        assert!(!toks.contains(&"in".to_string()));
        assert!(!toks.contains(&"to".to_string()));
    }

    #[test]
    fn tokenise_empty_query_returns_empty_vec() {
        assert!(tokenise_for_routing("").is_empty());
        assert!(tokenise_for_routing("  ").is_empty());
        assert!(tokenise_for_routing("a! b? c.").is_empty()); // all single-char
    }

    #[test]
    fn clip_snippet_below_cap_returns_full_body() {
        let body = "short body";
        assert_eq!(clip_snippet(body, 1024), body);
    }

    #[test]
    fn clip_snippet_above_cap_truncates_at_char_boundary() {
        let body = "abcdefghij".repeat(200); // 2000 chars
        let clipped = clip_snippet(&body, 1024);
        assert_eq!(clipped.chars().count(), 1024);
        // Should not panic on multi-byte chars.
        let multi = "αβγδε".repeat(500); // 2500 chars, 2 bytes each
        let clipped_m = clip_snippet(&multi, 1024);
        assert_eq!(clipped_m.chars().count(), 1024);
    }

    #[test]
    fn stage_name_is_stable_label() {
        assert_eq!(CosmosStage::name(), "cosmos");
    }
}
