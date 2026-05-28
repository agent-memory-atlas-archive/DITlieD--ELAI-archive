//! E-36 — Visual-artifact verifier wrapper.
//!
//! Producer for `DocumentVisionRagEventKind::VisualArtifactCheckVerdict`.
//! The verifier composes:
//!   1. A vision retriever (ColModernVBERT / ColQwen2.5-v0.2) that
//!      returns top-k page candidates by multi-vector similarity.
//!   2. A sidecar VLM (Qwen2.5-VL via `PerRoleRuntime`) that issues
//!      a verdict over the retrieved pages.
//!
//! HARD RULE per plan task 5 + the ELAI source-grounding rule:
//! verdicts whose [`elai_reasoning::types::ArtifactReliability`] is
//! `VlmCaption` carry LOW reliability and downstream gates MUST
//! downweight. `verify_pure_compute` enforces this by returning the
//! tier alongside the verdict so callers cannot drop it.
//!
//! Wire-dark default: producer call path emits SSB rows only when
//! [`super::document_vision_rag_emit_enabled`] is true.

use elai_reasoning::types::{ArtifactReliability, VisualArtifactVerdict};
use serde::{Deserialize, Serialize};

/// A retrieved page candidate returned by the vision retriever. The
/// verifier inspects each one before producing a verdict.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageRef {
    /// Source-document path relative to project_root.
    pub source_path: String,
    /// 0-indexed page number.
    pub page_index: u32,
    /// blake3 hash of the page image bytes (hex-encoded).
    pub content_hash_hex: String,
    /// Retrieval score in [0.0, 1.0] (higher = more relevant).
    pub score: f32,
}

/// Outcome carried by every verifier call. Producer maps this onto a
/// `DocumentVisionRagEventKind::VisualArtifactCheckVerdict` SSB row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerificationOutcome {
    /// Confirm / Refute / Indeterminate verdict.
    pub verdict: VisualArtifactVerdict,
    /// Source pages cited by the VLM. Empty on `Indeterminate`.
    pub evidence_pages: Vec<PageRef>,
    /// HARD reliability tier. `VlmCaption` is LOW — downstream gates
    /// MUST downweight.
    pub reliability: ArtifactReliability,
    /// Per-claim original text (echoed for trace correlation).
    pub claim: String,
}

impl VerificationOutcome {
    /// HARD downweight rule per ELAI source-grounding HARD RULE:
    /// returns `true` when downstream gates MUST downweight this
    /// verdict (i.e. reliability tier is `VlmCaption`).
    #[must_use]
    pub fn must_downweight(&self) -> bool {
        self.reliability.is_low()
    }

    /// True when the verdict has actionable evidence (Confirm/Refute
    /// AND at least one evidence page).
    #[must_use]
    pub fn has_evidence(&self) -> bool {
        !self.evidence_pages.is_empty()
            && matches!(
                self.verdict,
                VisualArtifactVerdict::Confirm | VisualArtifactVerdict::Refute
            )
    }
}

/// Pure-compute verdict assembler. The actual VLM call lives behind
/// `PerRoleRuntime` / sidecar transport; this fn ships the typed
/// surface so the policy + contract tests have something to call.
///
/// The verdict-selection rule (used while the VLM dispatch lands as a
/// follow-on plan):
///   * No candidates -> `Indeterminate` + `VlmCaption` (lowest tier)
///   * Candidates present + reliability `DiagramSource` -> `Confirm`
///     (diagram source is ground-truth tier per plan task 5)
///   * Candidates present + reliability `OcrParse` -> `Confirm` only
///     when the top score >= 0.8 (matches REAL-MM-RAG arXiv:2502.12342
///     reported brittleness on table queries; conservative threshold)
///   * Candidates present + reliability `VlmCaption` -> `Indeterminate`
///     by default — the HARD downweight rule means callers should never
///     get a `Confirm` from a VlmCaption-only verifier.
///
/// This is the pure-compute kernel; the production verifier swaps in
/// real VLM dispatch + the actual top-k retrieval loop.
#[must_use]
pub fn verify_pure_compute(
    claim: impl Into<String>,
    candidates: Vec<PageRef>,
    reliability: ArtifactReliability,
) -> VerificationOutcome {
    let claim = claim.into();
    if candidates.is_empty() {
        return VerificationOutcome {
            verdict: VisualArtifactVerdict::Indeterminate,
            evidence_pages: Vec::new(),
            reliability: ArtifactReliability::VlmCaption,
            claim,
        };
    }
    let verdict = match reliability {
        ArtifactReliability::DiagramSource => VisualArtifactVerdict::Confirm,
        ArtifactReliability::OcrParse => {
            // Conservative confirm — require a strong top score.
            let top = candidates
                .iter()
                .map(|p| p.score)
                .fold(f32::NEG_INFINITY, f32::max);
            if top >= 0.8 {
                VisualArtifactVerdict::Confirm
            } else {
                VisualArtifactVerdict::Indeterminate
            }
        }
        ArtifactReliability::VlmCaption => {
            // HARD downweight — never confirm from a VLM-caption-only
            // verifier. The verdict is Indeterminate; callers must
            // route to a higher-tier verifier for a definitive answer.
            VisualArtifactVerdict::Indeterminate
        }
    };
    let evidence_pages = if matches!(verdict, VisualArtifactVerdict::Confirm) {
        candidates
    } else {
        Vec::new()
    };
    VerificationOutcome {
        verdict,
        evidence_pages,
        reliability,
        claim,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(score: f32) -> PageRef {
        PageRef {
            source_path: "fixtures/p.pdf".into(),
            page_index: 0,
            content_hash_hex: "deadbeef".into(),
            score,
        }
    }

    #[test]
    fn empty_candidates_emits_indeterminate_and_low_tier() {
        let out = verify_pure_compute(
            "the auth flow has 3 redirects",
            Vec::new(),
            ArtifactReliability::OcrParse,
        );
        assert_eq!(out.verdict, VisualArtifactVerdict::Indeterminate);
        assert_eq!(out.reliability, ArtifactReliability::VlmCaption);
        assert!(out.must_downweight());
        assert!(!out.has_evidence());
    }

    #[test]
    fn diagram_source_confirms_high_tier() {
        let out = verify_pure_compute(
            "X sits between Y and Z in the ERD",
            vec![page(0.7)],
            ArtifactReliability::DiagramSource,
        );
        assert_eq!(out.verdict, VisualArtifactVerdict::Confirm);
        assert!(!out.must_downweight());
        assert!(out.has_evidence());
        assert_eq!(out.evidence_pages.len(), 1);
    }

    #[test]
    fn ocr_parse_requires_strong_top_score() {
        let weak = verify_pure_compute(
            "claim",
            vec![page(0.5), page(0.4)],
            ArtifactReliability::OcrParse,
        );
        assert_eq!(weak.verdict, VisualArtifactVerdict::Indeterminate);
        assert!(!weak.must_downweight()); // OcrParse is MED, not LOW.

        let strong = verify_pure_compute(
            "claim",
            vec![page(0.85)],
            ArtifactReliability::OcrParse,
        );
        assert_eq!(strong.verdict, VisualArtifactVerdict::Confirm);
    }

    #[test]
    fn vlm_caption_tier_never_confirms() {
        let out = verify_pure_compute(
            "the dashboard shows a histogram",
            vec![page(0.99), page(0.95)],
            ArtifactReliability::VlmCaption,
        );
        assert_eq!(out.verdict, VisualArtifactVerdict::Indeterminate);
        assert!(out.must_downweight());
        assert!(!out.has_evidence());
    }
}
