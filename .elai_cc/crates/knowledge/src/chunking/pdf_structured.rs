//! E-36 Tier-B — structured PDF chunker for mixed-modality documents.
//!
//! Bridges the text + vision paths: the parser emits markdown for the
//! flowing text + per-page HTML tables + figure crops + a per-page
//! confidence score. Downstream:
//!   * `markdown` -> existing E-17 text chunker
//!   * `figure_crops` -> [`super::image_artifact::ImageArtifactChunker`]
//!   * `page_confidences` -> the cloud-fallback gate in
//!     `orchestrator::document_vision_rag::policy`
//!
//! Producer wiring (PaddleOCR-VL 0.9B local / Dolphin-v2 local / Mistral
//! OCR cloud / Reducto cloud / LlamaParse cloud) lives in the
//! `orchestrator::document_vision_rag` subtree behind the
//! ELAI_DOCUMENT_VISION_RAG_ENABLE env gate. This module ships the
//! *types-only* + pure-compute fallback decision so the wire surface is
//! addressable without crossing the AVOID-zone today.
//!
//! Paper grounding (verified via WebSearch May 2026):
//!   * arXiv:2510.14528 — "PaddleOCR-VL: Boosting Multilingual Document
//!     Parsing via 0.9B Ultra-Compact VLM" (ERNIE-4.5-0.3B + NaViT visual
//!     encoder; 92.56 on OmniDocBench v1.5; 109 languages).
//!   * arXiv:2505.14059 — "Dolphin-v2" (ACL 2025 ByteDance; handwritten +
//!     photographed documents).
//!   * arXiv:2412.07626 — "OmniDocBench" (CVPR 2025; per-doc-type
//!     variance — drives the fallback_recommended trigger below).

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Default `cloud_fallback_threshold` per plan task 8. Pages whose parser
/// confidence drops below this fire
/// `DocumentVisionRagEventKind::VisionFallbackTriggered` and (when
/// `--mode != local`) escalate to the cloud parser. Configurable via
/// `[knowledge.doc_parser] cloud_fallback_threshold` in `elai_cc.toml`.
pub const DEFAULT_CLOUD_FALLBACK_THRESHOLD: f32 = 0.7;

/// Document-parser backend taxonomy. 3 local backends + 3 cloud
/// backends per plan task 8. The cloud arms are HARD-blocked when the
/// CLI is launched with `--mode local` (E-39 / C-5 cost gate); the
/// `DocumentVisionRagEventKind::DocParserCloudFallbackBlockedByMode`
/// fire-point exists specifically for that refusal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParserBackend {
    /// PaddleOCR-VL 0.9B local (arxiv:2510.14528). Default for visually-
    /// rich PDFs in [knowledge.doc_parser].
    PaddleOcrVl,
    /// MinerU local. Higher recall on table-heavy academic PDFs but
    /// larger weights; opt-in via config.
    MinerU,
    /// Dolphin-v2 3B local (arxiv:2505.14059). Best for handwritten /
    /// photographed inputs.
    Dolphin,
    /// Mistral OCR cloud. Strong on dense-equation PDFs; gated by
    /// `--mode cloud|hybrid`.
    MistralCloud,
    /// Reducto cloud. Strong on enterprise contracts + scanned legal.
    ReductoCloud,
    /// LlamaParse cloud. Generalist; LlamaIndex stack.
    LlamaParseCloud,
}

impl ParserBackend {
    /// Snake-case wire label.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::PaddleOcrVl => "paddleocr_vl",
            Self::MinerU => "mineru",
            Self::Dolphin => "dolphin",
            Self::MistralCloud => "mistral_cloud",
            Self::ReductoCloud => "reducto_cloud",
            Self::LlamaParseCloud => "llamaparse_cloud",
        }
    }

    /// True when this backend requires the cloud-mode gate. Used by the
    /// `DocParserCloudFallbackBlockedByMode` policy decision.
    #[must_use]
    pub const fn is_cloud(&self) -> bool {
        matches!(
            self,
            Self::MistralCloud | Self::ReductoCloud | Self::LlamaParseCloud
        )
    }
}

/// Structured output emitted by [`PdfStructuredChunker::parse`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfStructuredOutput {
    /// Flowing-text markdown body for the entire PDF. Empty string when
    /// the PDF carries no extractable text (image-only scan).
    pub markdown: String,
    /// One HTML table per page (empty string for pages with no tables).
    /// Indexed by page number; `page_html_tables.len() == page_count`.
    pub page_html_tables: Vec<String>,
    /// Per-page figure crops (PNG paths) extracted by the parser. The
    /// `ImageArtifactChunker` consumes these.
    pub figure_crops: Vec<PathBuf>,
    /// Per-page parser confidence in [0.0, 1.0]. The cloud-fallback
    /// gate fires when any entry drops below
    /// `[knowledge.doc_parser] cloud_fallback_threshold`.
    pub page_confidences: Vec<f32>,
    /// True when at least one page confidence is below the configured
    /// threshold AND the cloud mode permits a fallback. Set by
    /// `compute_fallback_recommended` from the per-page confidence array.
    pub fallback_recommended: bool,
    /// Which backend produced this output. Recorded for downstream SSB
    /// telemetry + the cosmos artifact stale-check.
    pub parser: ParserBackend,
}

impl PdfStructuredOutput {
    /// Page count is inferred from the per-page confidence array length.
    #[must_use]
    pub fn page_count(&self) -> usize {
        self.page_confidences.len()
    }

    /// True when ANY page confidence is below `threshold`. Pure compute,
    /// no env-var lookups (callers pass the threshold from their resolved
    /// config so unit tests stay deterministic).
    #[must_use]
    pub fn any_page_below_threshold(&self, threshold: f32) -> bool {
        self.page_confidences.iter().any(|c| *c < threshold)
    }
}

/// Errors emitted by the parser. Wide enough to carry the sidecar
/// transport failures Tier-B will route through here.
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("pdf_structured: empty path")]
    EmptyPath,
    #[error("pdf_structured: parser backend {backend:?} requires cloud mode")]
    CloudRequired { backend: ParserBackend },
    #[error("pdf_structured: parser dispatch error: {0}")]
    Dispatch(String),
}

/// Pure-compute helper that computes the `fallback_recommended` flag.
///
/// Returns true when AT LEAST ONE page confidence is below `threshold`
/// AND the resolved `mode_allows_cloud` is true. Default threshold is
/// [`DEFAULT_CLOUD_FALLBACK_THRESHOLD`]; the producer call path looks up
/// the configured value from `elai_cc.toml`.
#[must_use]
pub fn compute_fallback_recommended(
    page_confidences: &[f32],
    threshold: f32,
    mode_allows_cloud: bool,
) -> bool {
    if !mode_allows_cloud {
        return false;
    }
    page_confidences.iter().any(|c| *c < threshold)
}

/// Pure-compute chunker driver. The actual parse step crosses into
/// `orchestrator::document_vision_rag`; this struct carries the typed
/// surface so callers can construct + inspect outputs without touching
/// the AVOID-zone today.
#[derive(Debug, Clone, Copy, Default)]
pub struct PdfStructuredChunker;

impl PdfStructuredChunker {
    /// Build a [`PdfStructuredOutput`] from already-parsed component
    /// streams. Used by the orchestrator-side dispatcher once the
    /// sidecar PaddleOCR-VL has returned. Computes
    /// `fallback_recommended` via [`compute_fallback_recommended`].
    pub fn finalize(
        markdown: String,
        page_html_tables: Vec<String>,
        figure_crops: Vec<PathBuf>,
        page_confidences: Vec<f32>,
        parser: ParserBackend,
        threshold: f32,
        mode_allows_cloud: bool,
    ) -> PdfStructuredOutput {
        let fallback_recommended =
            compute_fallback_recommended(&page_confidences, threshold, mode_allows_cloud);
        PdfStructuredOutput {
            markdown,
            page_html_tables,
            figure_crops,
            page_confidences,
            fallback_recommended,
            parser,
        }
    }

    /// Validate parser eligibility against the cloud-mode gate. Returns
    /// `Err(ParseError::CloudRequired)` when the caller asks for a
    /// cloud backend but the resolved mode is `local`. Used by the
    /// policy gate to fire
    /// `DocumentVisionRagEventKind::DocParserCloudFallbackBlockedByMode`.
    pub fn check_backend_eligibility(
        backend: ParserBackend,
        mode_allows_cloud: bool,
    ) -> Result<(), ParseError> {
        if backend.is_cloud() && !mode_allows_cloud {
            return Err(ParseError::CloudRequired { backend });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_backend_local_vs_cloud() {
        assert!(!ParserBackend::PaddleOcrVl.is_cloud());
        assert!(!ParserBackend::MinerU.is_cloud());
        assert!(!ParserBackend::Dolphin.is_cloud());
        assert!(ParserBackend::MistralCloud.is_cloud());
        assert!(ParserBackend::ReductoCloud.is_cloud());
        assert!(ParserBackend::LlamaParseCloud.is_cloud());
    }

    #[test]
    fn fallback_fires_when_page_below_threshold_and_cloud_allowed() {
        let confs = [0.95_f32, 0.6_f32, 0.92_f32];
        assert!(compute_fallback_recommended(
            &confs,
            DEFAULT_CLOUD_FALLBACK_THRESHOLD,
            true
        ));
    }

    #[test]
    fn fallback_blocked_in_local_mode() {
        let confs = [0.4_f32, 0.5_f32];
        assert!(!compute_fallback_recommended(
            &confs,
            DEFAULT_CLOUD_FALLBACK_THRESHOLD,
            false
        ));
    }

    #[test]
    fn fallback_not_recommended_when_all_high() {
        let confs = [0.95_f32, 0.92_f32, 0.99_f32];
        assert!(!compute_fallback_recommended(
            &confs,
            DEFAULT_CLOUD_FALLBACK_THRESHOLD,
            true
        ));
    }

    #[test]
    fn finalize_attaches_fallback_flag() {
        let out = PdfStructuredChunker::finalize(
            "body".into(),
            vec!["".into()],
            vec![],
            vec![0.6],
            ParserBackend::PaddleOcrVl,
            DEFAULT_CLOUD_FALLBACK_THRESHOLD,
            true,
        );
        assert!(out.fallback_recommended);
        assert_eq!(out.parser, ParserBackend::PaddleOcrVl);
        assert_eq!(out.page_count(), 1);
    }

    #[test]
    fn check_backend_blocks_cloud_in_local_mode() {
        let r =
            PdfStructuredChunker::check_backend_eligibility(ParserBackend::MistralCloud, false);
        assert!(matches!(r, Err(ParseError::CloudRequired { .. })));
    }

    #[test]
    fn check_backend_allows_local_anywhere() {
        assert!(PdfStructuredChunker::check_backend_eligibility(
            ParserBackend::PaddleOcrVl,
            false
        )
        .is_ok());
        assert!(PdfStructuredChunker::check_backend_eligibility(
            ParserBackend::Dolphin,
            true
        )
        .is_ok());
    }

    #[test]
    fn any_page_below_threshold_helper() {
        let out = PdfStructuredChunker::finalize(
            String::new(),
            vec![],
            vec![],
            vec![0.95, 0.55, 0.88],
            ParserBackend::PaddleOcrVl,
            DEFAULT_CLOUD_FALLBACK_THRESHOLD,
            true,
        );
        assert!(out.any_page_below_threshold(DEFAULT_CLOUD_FALLBACK_THRESHOLD));
        assert!(!out.any_page_below_threshold(0.4));
    }
}
