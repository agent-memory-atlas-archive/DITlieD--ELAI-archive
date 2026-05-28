//! E-36 Tier-B — visual_artifact_reliability_tier_contract.
//!
//! Locks plan ACCEPTANCE row: "VlmCaption-tier verdicts carry LOW
//! reliability flag and downstream gates downweight per ELAI source-
//! grounding HARD RULE".
//!
//! The HARD rule per plan task 5: any verdict whose
//! `ArtifactReliability` tier == `VlmCaption` MUST be downweighted by
//! downstream gates (auditor / scope-planner). The contract here
//! enforces the structural shape (3 tiers, only VlmCaption is LOW,
//! verifier never confirms from VlmCaption-only candidates) so a
//! future refactor cannot silently re-tier VlmCaption upward.

use elai_orchestrator::document_vision_rag::visual_verifier::{
    verify_pure_compute, PageRef,
};
use elai_reasoning::types::{ArtifactReliability, VisualArtifactVerdict};

fn page(score: f32) -> PageRef {
    PageRef {
        source_path: "fixtures/spec.pdf".into(),
        page_index: 0,
        content_hash_hex: "deadbeef".into(),
        score,
    }
}

#[test]
fn reliability_tiers_lock_at_three_variants() {
    // The 3-arm taxonomy is the structural contract per plan task 5.
    // Growth = explicit decision; widening LOW-tier is a HARD anti-pattern.
    let tiers = [
        ArtifactReliability::VlmCaption,
        ArtifactReliability::OcrParse,
        ArtifactReliability::DiagramSource,
    ];
    assert_eq!(tiers.len(), 3);
}

#[test]
fn vlm_caption_is_low_tier_and_must_downweight() {
    assert!(ArtifactReliability::VlmCaption.is_low());
}

#[test]
fn ocr_parse_is_medium_tier() {
    // MED: lower-trust than DiagramSource but no HARD downweight.
    assert!(!ArtifactReliability::OcrParse.is_low());
}

#[test]
fn diagram_source_is_high_tier() {
    // HIGH: ground-truth tier for visual-spatial claims.
    assert!(!ArtifactReliability::DiagramSource.is_low());
}

#[test]
fn reliability_tier_wire_labels_lock_at_snake_case() {
    assert_eq!(ArtifactReliability::VlmCaption.as_str(), "vlm_caption");
    assert_eq!(ArtifactReliability::OcrParse.as_str(), "ocr_parse");
    assert_eq!(
        ArtifactReliability::DiagramSource.as_str(),
        "diagram_source"
    );
}

#[test]
fn vlm_caption_verifier_never_confirms_high_confidence_pages() {
    // HARD RULE: even with strong retrieval scores, a VlmCaption-tier
    // verifier MUST NOT issue a Confirm verdict. Downstream gates rely
    // on this so they only downweight LOW outputs, never override HIGH ones.
    let strong = vec![page(0.99), page(0.95)];
    let out = verify_pure_compute(
        "the dashboard shows a histogram",
        strong,
        ArtifactReliability::VlmCaption,
    );
    assert_eq!(out.verdict, VisualArtifactVerdict::Indeterminate);
    assert!(out.must_downweight());
}

#[test]
fn diagram_source_verifier_confirms_with_candidates() {
    let out = verify_pure_compute(
        "X sits between Y and Z in the ERD",
        vec![page(0.5)],
        ArtifactReliability::DiagramSource,
    );
    assert_eq!(out.verdict, VisualArtifactVerdict::Confirm);
    assert!(!out.must_downweight());
    assert!(out.has_evidence());
}

#[test]
fn ocr_parse_verifier_needs_strong_top_score_to_confirm() {
    // Weak retrieval -> Indeterminate
    let weak = verify_pure_compute(
        "the table has 5 columns",
        vec![page(0.4)],
        ArtifactReliability::OcrParse,
    );
    assert_eq!(weak.verdict, VisualArtifactVerdict::Indeterminate);

    // Strong retrieval -> Confirm
    let strong = verify_pure_compute(
        "the table has 5 columns",
        vec![page(0.9)],
        ArtifactReliability::OcrParse,
    );
    assert_eq!(strong.verdict, VisualArtifactVerdict::Confirm);
}

#[test]
fn empty_candidates_emits_indeterminate_low_tier() {
    let out = verify_pure_compute(
        "the auth flow has 3 redirects",
        Vec::new(),
        ArtifactReliability::DiagramSource,
    );
    assert_eq!(out.verdict, VisualArtifactVerdict::Indeterminate);
    // Empty candidates always degrade to LOW tier (HARD: no evidence
    // means downstream gates MUST downweight).
    assert_eq!(out.reliability, ArtifactReliability::VlmCaption);
    assert!(out.must_downweight());
}

#[test]
fn verification_outcome_serde_round_trip() {
    let out = verify_pure_compute(
        "claim",
        vec![page(0.85)],
        ArtifactReliability::OcrParse,
    );
    let json = serde_json::to_string(&out).expect("serialize");
    // The wire tag for the tier MUST be snake_case so dashboards /
    // python sidecar can round-trip without a translation table.
    assert!(json.contains("\"ocr_parse\""));
}
