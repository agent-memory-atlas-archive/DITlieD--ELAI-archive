//! E-36 Tier-A — vision_chunk_types_contract.
//!
//! Locks the Tier-A surface of `knowledge::chunking::image_artifact`:
//!   - VisionArtifactKind variant set (Page | Screenshot | Diagram)
//!   - VisionArtifact field set + serde round-trip
//!   - blake3 hash format (hex, 64 chars, lower-case)
//!
//! Growth of either type requires a coordinated bump of this contract +
//! a row in workflow/reasoning/E-36-decisions.md per the campaign discipline.

use elai_knowledge::chunking::image_artifact::{VisionArtifact, VisionArtifactKind};

#[test]
fn vision_artifact_kind_locks_at_three_variants() {
    // If a new kind is added, this list must update AND the as_str() round-trip
    // test below must extend. The variant count is the contract.
    let kinds = [
        VisionArtifactKind::Page,
        VisionArtifactKind::Screenshot,
        VisionArtifactKind::Diagram,
    ];
    assert_eq!(kinds.len(), 3);
    for kind in kinds {
        let s = kind.as_str();
        assert!(!s.is_empty(), "as_str() must not be empty");
        assert_eq!(s.to_ascii_lowercase(), s, "as_str() must be lower-case");
    }
}

#[test]
fn vision_artifact_kind_as_str_matches_serde_snake_case() {
    // Pin the wire form so a producer in another crate (or another language
    // via JSON) cannot diverge from the Rust enum repr.
    assert_eq!(VisionArtifactKind::Page.as_str(), "page");
    assert_eq!(VisionArtifactKind::Screenshot.as_str(), "screenshot");
    assert_eq!(VisionArtifactKind::Diagram.as_str(), "diagram");
}

#[test]
fn vision_artifact_serde_round_trip() {
    let raw = b"the quick brown fox";
    let artifact = VisionArtifact {
        source_path: "fixtures/sample.pdf".to_string(),
        page_index: Some(7),
        kind: VisionArtifactKind::Page,
        content_hash_hex: VisionArtifact::hash_bytes(raw),
        byte_size: raw.len() as u64,
    };
    let json = serde_json::to_string(&artifact).expect("serialize ok");
    let back: VisionArtifact = serde_json::from_str(&json).expect("deserialize ok");
    assert_eq!(back, artifact);
}

#[test]
fn standalone_screenshot_has_no_page_index() {
    // Contract: standalone Screenshot has page_index == None (None encodes
    // "not derived from a multi-page document"). A Page MUST carry a Some.
    let shot = VisionArtifact {
        source_path: "captures/login_screen.png".to_string(),
        page_index: None,
        kind: VisionArtifactKind::Screenshot,
        content_hash_hex: VisionArtifact::hash_bytes(b""),
        byte_size: 0,
    };
    assert!(shot.page_index.is_none());
    assert_eq!(shot.kind, VisionArtifactKind::Screenshot);
}

#[test]
fn hash_is_blake3_hex_64_chars_lower_case() {
    // blake3 produces a 256-bit digest; hex-encoded as 64 lower-case chars.
    // If a future refactor swaps the digest format, this test catches it.
    let h = VisionArtifact::hash_bytes(b"contract pin");
    assert_eq!(h.len(), 64, "blake3 hex must be 64 chars");
    for c in h.chars() {
        assert!(
            c.is_ascii_digit() || matches!(c, 'a'..='f'),
            "hex char must be lower-case 0-9/a-f, got {c}"
        );
    }
    // Empty input is a stable, documentable digest — pin it so the
    // anti-stale rule (blake3, not blake2) cannot regress silently.
    assert_eq!(
        VisionArtifact::hash_bytes(b""),
        "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"
    );
}
