//! J-77 S5 contract test — perception faithfulness (I5 + I6).
//!
//! Source plan: `docs/thefinalglance/J-plan/J-77_*.md` (citation-prebind
//! over perception regions + per-element source/confidence).
//!
//! This contract calls the SAME production fns that the dispatch seam uses:
//!   * `perception::filter_ghost_regions` — called from
//!     `production_agent_rpc.rs` in the `ScaffoldEmitted` arm.
//!   * `perception::prebind_regions` — the production entry that
//!     `filter_ghost_regions` delegates to.
//!   * `perception::ElementSource` / `LiteralElement` — the I6 per-element
//!     provenance fields.
//!
//! FALSIFICATION NOTE: deleting the production call site of
//! `filter_ghost_regions` in `production_agent_rpc.rs` must fail
//! `production_caller_check --symbols filter_ghost_regions`. This test does
//! NOT implement that check by grepping; it is verified off-agent by the
//! wire checker.

use elai_orchestrator::perception::{
    prebind_regions, filter_ghost_regions, ElementSource, LiteralElement,
};
use elai_orchestrator::citation_prebind::CitationCheckOutcome;

fn elem(label: &str, bbox: [u32; 4], source: ElementSource, conf: f32) -> LiteralElement {
    LiteralElement {
        label: label.to_string(),
        bbox,
        source,
        confidence: conf,
    }
}

// (1) ghost_region_routed_to_unverified: detection {button} + VLM ghost
//     {fake_control} → fake in unverified, button in kept.
#[test]
fn ghost_region_routed_to_unverified() {
    let regions = vec![
        elem("button", [0, 0, 100, 50], ElementSource::Ocr, 1.0),
        elem("fake_control", [200, 200, 300, 250], ElementSource::Vlm, 0.3),
    ];
    let outcome = filter_ghost_regions(&regions, 0);
    assert!(
        outcome.unverified.iter().any(|e| e.label == "fake_control"),
        "VLM ghost region must be in unverified"
    );
    assert!(
        outcome.kept.iter().any(|e| e.label == "button"),
        "detection region must be in kept"
    );
    assert!(
        !outcome.kept.iter().any(|e| e.label == "fake_control"),
        "VLM ghost must NOT be in kept"
    );
}

// (2) closed_set_out_of_set_claim: freeze 2 regions; claim id 99 →
//     OutOfSet / not InSet.
#[test]
fn closed_set_out_of_set_claim() {
    let regions = vec![
        elem("region_a", [0, 0, 100, 100], ElementSource::Ocr, 1.0),
        elem("region_b", [100, 0, 200, 100], ElementSource::Ocr, 1.0),
    ];
    let outcome = prebind_regions(&regions, &[99], 0);
    assert_eq!(outcome.checks.len(), 1);
    assert_ne!(
        outcome.checks[0].outcome,
        CitationCheckOutcome::Bound,
        "out-of-set claim must not be Bound"
    );
    assert_eq!(
        outcome.checks[0].outcome,
        CitationCheckOutcome::Ghost,
        "out-of-set claim must be Ghost"
    );
}

// (3) source_confidence_roundtrip: serde LiteralElement with source=Vlm
//     confidence=0.3.
#[test]
fn source_confidence_roundtrip() {
    let element = elem("text", [10, 20, 30, 40], ElementSource::Vlm, 0.3);
    let json = serde_json::to_string(&element).expect("serialize");
    let deserialized: LiteralElement =
        serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.source, ElementSource::Vlm);
    assert!(
        (deserialized.confidence - 0.3).abs() < 1e-6,
        "confidence must round-trip"
    );
}

// (4) old JSON without source/confidence fields still deserializes
//     (serde defaults: source=Ocr, confidence=1.0).
#[test]
fn old_json_deserializes_with_defaults() {
    let old_json = r#"{"label":"old","bbox":[0,0,50,50]}"#;
    let element: LiteralElement =
        serde_json::from_str(old_json).expect("old JSON must deserialize");
    assert_eq!(element.label, "old");
    assert_eq!(element.source, ElementSource::Ocr);
    assert_eq!(element.confidence, 1.0);
}
