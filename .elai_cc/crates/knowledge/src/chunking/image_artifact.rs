//! E-36 Tier-A — vision artifact types for the document-vision RAG path.
//!
//! Types-only foundation for the 9th VerificationMethod (the plan called it
//! "8th GroundingMethod"; reality has 8 VerificationMethod variants today, so
//! the new one would be the 9th — see workflow/reasoning/E-36-decisions.md
//! DRIFT-2). Defining the surface here in OWNED `knowledge::chunking` keeps
//! the contract addressable without entering the AVOIDED reasoning crate
//! where VerificationMethod itself lives.
//!
//! Producer wiring (PaddleOCR-VL / ColModernVBERT / Dolphin sidecar paths)
//! is DEFERRED to E-36.B — those calls cross into python/elai_inference and
//! providers/runtime, both AVOIDED this run.
//!
//! Hash: blake3 per ELAI anti-stale rule. SHA-256 / blake2 are forbidden for
//! non-RFC uses; blake3 wins on throughput + cache-line behaviour and is the
//! workspace standard.

use serde::{Deserialize, Serialize};

/// Stable artifact-kind tag carried by every vision chunk.
///
/// The variant set is locked at three kinds for Tier-A; any growth requires
/// a coordinated bump of the contract test `vision_chunk_types_contract.rs`
/// + a row in the E-36 decisions doc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VisionArtifactKind {
    /// A rasterized page (PNG / JPEG) extracted from a PDF or similar
    /// document. Renderable by a VLM directly; OCR optional.
    Page,
    /// A screenshot or annotated image NOT derived from a multi-page doc.
    /// Distinct from `Page` so the indexer can route differently.
    Screenshot,
    /// A diagram / chart / figure embedded inside a larger document.
    /// Producers may extract these from PDFs as separate artifacts.
    Diagram,
}

impl VisionArtifactKind {
    /// String tag mirroring serde's `snake_case` rename. Useful for SQL CHECK
    /// constraints + JSON wire shape interop without forcing serde callers.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Page => "page",
            Self::Screenshot => "screenshot",
            Self::Diagram => "diagram",
        }
    }
}

/// A single indexable vision artifact.
///
/// Fields are intentionally minimal at Tier-A. Tier-B will extend with VLM
/// extraction output (text, captions, bounding boxes) once the producer side
/// of the pipeline lands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisionArtifact {
    /// Source document path relative to project_root. For multi-page docs
    /// this is the doc path; the per-page index is in `page_index`.
    pub source_path: String,
    /// 0-indexed page number for `Page` and `Diagram` kinds derived from a
    /// multi-page document. `None` for standalone screenshots.
    pub page_index: Option<u32>,
    /// Stable artifact tag.
    pub kind: VisionArtifactKind,
    /// blake3 hash of the raw artifact bytes (32-byte digest, hex-encoded).
    /// Producers MUST emit this so downstream consumers can verify the cache
    /// is fresh against the upstream file without re-OCRing.
    pub content_hash_hex: String,
    /// Byte size of the underlying artifact. Used as a coarse skip-gate
    /// (e.g. > 10 MB defers to a streaming path Tier-B will land).
    pub byte_size: u64,
}

impl VisionArtifact {
    /// Compute the canonical content hash from raw bytes. Producer helper —
    /// using blake3 directly here avoids leaking the hash choice across the
    /// module boundary.
    #[must_use]
    pub fn hash_bytes(bytes: &[u8]) -> String {
        let digest = blake3::hash(bytes);
        digest.to_hex().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variant_str_round_trips() {
        for kind in [
            VisionArtifactKind::Page,
            VisionArtifactKind::Screenshot,
            VisionArtifactKind::Diagram,
        ] {
            let json = serde_json::to_string(&kind).expect("serialize ok");
            // serde rename_all=snake_case wraps single variants as a JSON
            // string; the str repr is the same value sans quotes.
            assert_eq!(json.trim_matches('"'), kind.as_str());
        }
    }

    #[test]
    fn hash_bytes_is_blake3_hex() {
        let h = VisionArtifact::hash_bytes(b"hello");
        // blake3 of "hello" — pinned to catch any accidental algorithm swap
        // (e.g. someone "refactoring" to sha2 — blocked by anti-stale rule).
        assert_eq!(
            h,
            "ea8f163db38682925e4491c5e58d4bb3506ef8c14eb78a86e908c5624a67200f"
        );
    }

    #[test]
    fn artifact_round_trip() {
        let bytes = b"page contents";
        let artifact = VisionArtifact {
            source_path: "docs/spec.pdf".into(),
            page_index: Some(3),
            kind: VisionArtifactKind::Page,
            content_hash_hex: VisionArtifact::hash_bytes(bytes),
            byte_size: bytes.len() as u64,
        };
        let json = serde_json::to_string(&artifact).expect("serialize ok");
        let back: VisionArtifact = serde_json::from_str(&json).expect("deserialize ok");
        assert_eq!(back, artifact);
    }
}
