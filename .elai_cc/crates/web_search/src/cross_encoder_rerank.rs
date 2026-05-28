//! E-48 Task 4 — cross-encoder rerank top-50 -> top-5.
//!
//! Sits between [`crate::source_quality::SourceQualityTable`] tier-filter
//! output and the WebResearchOrchestrator's LLM-input chunk packing.
//!
//! Default reranker: local sidecar `/rerank` endpoint. The endpoint
//! does not exist today (per the plan's open question #5); when absent
//! the cascade falls back to Cohere Rerank v3.5 if a key is configured,
//! or no-rerank (returns input top-N unchanged) otherwise.
//!
//! Naming clarification: there IS an existing
//! `elai_knowledge::reranker::CrossEncoderReranker` in the knowledge
//! crate (per the cross-check report). It serves a different surface
//! (RAG-retrieval reranking against a query embedding); this module
//! covers web-search results reranking against the natural-language
//! research question. The two coexist; the design choice is to keep
//! them separate so the knowledge crate's reranker stays focused on the
//! embedding-based RAG path and this module focuses on web-source
//! reranking via either the sidecar `/rerank` endpoint or Cohere's
//! managed reranker API.
//!
//! Grounding:
//!   * docs.cohere.com/docs/reranking-best-practices -- 4093-token
//!     chunks, max-pooled per source, top-50 -> top-5 (grounds
//!     COHERE_CHUNK_TOKEN_BUDGET + the top-50 -> top-5 contract).
//!   * Local sidecar /rerank: RUNPOD-PENDING.

use serde::{Deserialize, Serialize};

/// Cohere Rerank v3.5 per-chunk token budget. Per their best-practices
/// doc; chunks beyond this are truncated max-pooled.
pub const COHERE_CHUNK_TOKEN_BUDGET: u32 = 4093;

/// Top-N input cap: rerank operates on at most this many sources.
pub const DEFAULT_RERANK_INPUT_CAP: usize = 50;

/// Top-N output cap: rerank returns at most this many sources.
pub const DEFAULT_RERANK_OUTPUT_CAP: usize = 5;

/// Which backend handled this rerank call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RerankerBackend {
    /// Local sidecar `/rerank` endpoint (RUNPOD-PENDING).
    LocalSidecar,
    /// Cohere Rerank v3.5 managed API.
    Cohere,
    /// No reranker available -- returns input top-N unchanged.
    None,
}

impl RerankerBackend {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LocalSidecar => "local_sidecar",
            Self::Cohere => "cohere",
            Self::None => "none",
        }
    }
}

/// One source input to the reranker.
#[derive(Debug, Clone, PartialEq)]
pub struct RerankInput {
    pub url: String,
    pub title: String,
    /// Relevance text -- the snippet or full extracted content the
    /// reranker scores against the research question.
    pub text: String,
    /// Pre-rerank position (used for the rank-shift histogram).
    pub pre_rerank_position: usize,
}

/// One source after rerank, with its new position.
#[derive(Debug, Clone, PartialEq)]
pub struct RerankOutput {
    pub url: String,
    pub title: String,
    pub text: String,
    pub pre_rerank_position: usize,
    pub post_rerank_position: usize,
}

/// Configuration for the rerank cascade.
#[derive(Debug, Clone, PartialEq)]
pub struct CrossEncoderRerankConfig {
    /// Local sidecar `/rerank` endpoint URL. `None` disables the
    /// primary; cascade falls through to Cohere.
    pub local_sidecar_endpoint: Option<String>,
    /// Cohere API key. `None` disables Cohere; cascade falls through to
    /// `RerankerBackend::None`.
    pub cohere_api_key: Option<String>,
    pub input_cap: usize,
    pub output_cap: usize,
}

impl Default for CrossEncoderRerankConfig {
    fn default() -> Self {
        Self {
            local_sidecar_endpoint: None,
            cohere_api_key: None,
            input_cap: DEFAULT_RERANK_INPUT_CAP,
            output_cap: DEFAULT_RERANK_OUTPUT_CAP,
        }
    }
}

/// Resolve which backend the cascade will use given the configuration.
/// Cascade: LocalSidecar > Cohere > None.
#[must_use]
pub fn resolve_backend(cfg: &CrossEncoderRerankConfig) -> RerankerBackend {
    if cfg.local_sidecar_endpoint.is_some() {
        RerankerBackend::LocalSidecar
    } else if cfg.cohere_api_key.is_some() {
        RerankerBackend::Cohere
    } else {
        RerankerBackend::None
    }
}

/// Compute the rank-shift histogram for one rerank pass.
///
/// `histogram[i]` = number of sources whose absolute rank shift (i.e.
/// `(post - pre).abs()`) equals exactly `i + 1`. Length capped at 5 to
/// mirror the top-5 output cap. Sources that did NOT shift contribute
/// to no bucket.
#[must_use]
pub fn rank_shift_histogram(outputs: &[RerankOutput]) -> [u32; 5] {
    let mut hist = [0_u32; 5];
    for out in outputs {
        let shift = out
            .post_rerank_position
            .max(out.pre_rerank_position)
            - out.post_rerank_position.min(out.pre_rerank_position);
        if shift == 0 || shift > 5 {
            continue;
        }
        let bucket = shift - 1;
        hist[bucket] = hist[bucket].saturating_add(1);
    }
    hist
}

/// No-op rerank that preserves input order. Used when
/// [`resolve_backend`] returns [`RerankerBackend::None`] so callers
/// can flow through the same data path regardless of backend
/// availability.
#[must_use]
pub fn passthrough_rerank(
    inputs: Vec<RerankInput>,
    output_cap: usize,
) -> Vec<RerankOutput> {
    inputs
        .into_iter()
        .take(output_cap)
        .enumerate()
        .map(|(pos, inp)| RerankOutput {
            url: inp.url,
            title: inp.title,
            text: inp.text,
            pre_rerank_position: inp.pre_rerank_position,
            post_rerank_position: pos,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_labels_canonical() {
        assert_eq!(RerankerBackend::LocalSidecar.as_str(), "local_sidecar");
        assert_eq!(RerankerBackend::Cohere.as_str(), "cohere");
        assert_eq!(RerankerBackend::None.as_str(), "none");
    }

    #[test]
    fn resolve_backend_prefers_local_sidecar() {
        let cfg = CrossEncoderRerankConfig {
            local_sidecar_endpoint: Some("http://127.0.0.1:8100".to_owned()),
            cohere_api_key: Some("key".to_owned()),
            ..Default::default()
        };
        assert_eq!(resolve_backend(&cfg), RerankerBackend::LocalSidecar);
    }

    #[test]
    fn resolve_backend_falls_through_to_cohere() {
        let cfg = CrossEncoderRerankConfig {
            local_sidecar_endpoint: None,
            cohere_api_key: Some("key".to_owned()),
            ..Default::default()
        };
        assert_eq!(resolve_backend(&cfg), RerankerBackend::Cohere);
    }

    #[test]
    fn resolve_backend_falls_through_to_none() {
        let cfg = CrossEncoderRerankConfig::default();
        assert_eq!(resolve_backend(&cfg), RerankerBackend::None);
    }

    #[test]
    fn passthrough_caps_at_output_cap() {
        let inputs: Vec<RerankInput> = (0..50)
            .map(|i| RerankInput {
                url: format!("https://u{i}"),
                title: format!("t{i}"),
                text: format!("text{i}"),
                pre_rerank_position: i,
            })
            .collect();
        let out = passthrough_rerank(inputs, DEFAULT_RERANK_OUTPUT_CAP);
        assert_eq!(out.len(), DEFAULT_RERANK_OUTPUT_CAP);
        assert_eq!(out[0].post_rerank_position, 0);
        assert_eq!(out[4].post_rerank_position, 4);
    }

    #[test]
    fn rank_shift_histogram_buckets_by_distance() {
        let outputs = vec![
            // Shift 1.
            RerankOutput {
                url: "a".into(),
                title: "".into(),
                text: "".into(),
                pre_rerank_position: 0,
                post_rerank_position: 1,
            },
            // Shift 3.
            RerankOutput {
                url: "b".into(),
                title: "".into(),
                text: "".into(),
                pre_rerank_position: 1,
                post_rerank_position: 4,
            },
            // No shift.
            RerankOutput {
                url: "c".into(),
                title: "".into(),
                text: "".into(),
                pre_rerank_position: 2,
                post_rerank_position: 2,
            },
        ];
        let h = rank_shift_histogram(&outputs);
        assert_eq!(h[0], 1, "shift-1 bucket");
        assert_eq!(h[2], 1, "shift-3 bucket");
        assert_eq!(h[1], 0);
        assert_eq!(h[3], 0);
        assert_eq!(h[4], 0);
    }

    #[test]
    fn cohere_chunk_budget_is_4093() {
        // Pinning the constant against accidental drift.
        assert_eq!(COHERE_CHUNK_TOKEN_BUDGET, 4093);
    }
}
