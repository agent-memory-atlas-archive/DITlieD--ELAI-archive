//! E-36 — vision artifact types + chunker for the document-vision RAG path.
//!
//! Tier-A foundation shipped the types-only enum + struct + blake3 helper.
//! Tier-B (this file) GROWS the existing surface with four Option<>-wrapped
//! fields (multi_vec / pooled / vlm_caption / parser_confidence) and adds the
//! `ImageArtifactChunker` producer that emits VisionArtifact rows.
//!
//! Plan: docs/thefinalglance/E-plan/E-36_document_vision_rag.md task 2.
//! The plan named the producer's output "VisionChunk"; we reconcile by
//! extending the existing VisionArtifact instead of introducing a parallel
//! struct (the Tier-A contract test stays green because the new fields are
//! Option<>-wrapped with `#[serde(default, skip_serializing_if)]`).
//!
//! Hash: blake3 per ELAI anti-stale rule. SHA-256 / blake2 are forbidden for
//! non-RFC uses; blake3 wins on throughput + cache-line behaviour and is the
//! workspace standard.
//!
//! Pooling: the plan task 2 cites arxiv:2602.12510 (Visual RAG Toolkit,
//! Feb 2026, "Scaling Multi-Vector Visual Retrieval with Training-Free
//! Pooling and Multi-Stage Search") for the ~10x storage reduction. Until
//! the ColModernVBERT / ColQwen path lands the actual late-interaction
//! vectors, we expose a `pool_mean` helper that mean-pools 1030 per-page
//! vectors down to the configured `MAX_VECTORS_PER_PAGE` (default 100,
//! matching the Visual RAG Toolkit's reported recall-preserving ceiling).
//! Mean-pool is the conservative fallback the plan permits when the
//! paper's training-free spatial pooling is not yet wired (see Open
//! Question 1 in the plan).
//!
//! Producer wiring to the real VLM (Qwen2.5-VL via PerRoleRuntime) is
//! gated behind ELAI_DOCUMENT_VISION_RAG_ENABLE and lives in the
//! orchestrator crate (`orchestrator::document_vision_rag::visual_verifier`).

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Maximum number of pooled vectors per page emitted by the chunker.
///
/// Plan task 2 + DETERMINISTIC_CHECK item 5 fix this at <= 128. We pin 100
/// (the Visual RAG Toolkit's reported recall-preserving ceiling per
/// arXiv:2602.12510) so the contract test can assert a stable cap.
pub const MAX_VECTORS_PER_PAGE: usize = 100;

/// Per-vector dimensionality for the late-interaction multi-vector encoder.
///
/// ColModernVBERT / ColQwen2.5-v0.2 both emit 128-dim per-patch embeddings
/// per their published model cards. We store them as `f32` here rather than
/// the plan's suggested `f16` because the workspace does not depend on the
/// `half` crate and a 4-byte fp32 representation keeps the disk footprint
/// at ~400 KB/page (100 vectors * 128 dims * 4 bytes); the 200 KB savings
/// from fp16 are not worth a new dependency for Tier-B scope.
pub const VECTOR_DIM: usize = 128;

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
/// Tier-A landed `source_path` / `page_index` / `kind` / `content_hash_hex` /
/// `byte_size`. Tier-B adds 4 Option<>-wrapped fields for the producer
/// pipeline output:
///   * `multi_vec` — pooled late-interaction vectors (None until the VLM
///     encoder has run; producer call path is gated by
///     ELAI_DOCUMENT_VISION_RAG_ENABLE).
///   * `pooled` — true when `multi_vec` has been reduced from the native
///     ~1030-vector ColPali shape down to <= MAX_VECTORS_PER_PAGE per page.
///   * `vlm_caption` — Qwen2.5-VL natural-language caption attached at
///     ingest time. Carried as informational only; HARD RULE per the
///     plan's anti-pattern flag: VlmCaption-only verdicts carry LOW
///     reliability and downstream gates MUST downweight (see
///     `reasoning::types::ArtifactReliability`).
///   * `parser_confidence` — PaddleOCR-VL / Dolphin emitted confidence in
///     [0.0, 1.0]. Producer fires
///     `DocumentVisionRagEventKind::VisionFallbackTriggered` when this
///     drops below the cloud-fallback threshold (default 0.7).
///
/// Back-compat note: the Tier-A `vision_chunk_types_contract.rs` test
/// constructs a VisionArtifact via struct-literal syntax. Adding the four
/// new fields breaks that test unless we also update it; the contract
/// test extension lands in the same Tier-B commit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    /// Tier-B: pooled multi-vector index emitted by the ColModernVBERT /
    /// ColQwen2.5-v0.2 encoder. Each inner vec carries `VECTOR_DIM`
    /// f32 patch embeddings; the outer `Vec` length is bounded by
    /// `MAX_VECTORS_PER_PAGE` once `pooled == true`. None until the
    /// producer has run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub multi_vec: Option<Vec<Vec<f32>>>,
    /// Tier-B: true once `multi_vec` has been reduced from the native
    /// per-patch ColPali grid down to the per-page <= MAX_VECTORS_PER_PAGE
    /// ceiling. False or absent for raw (unpooled) producers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pooled: Option<bool>,
    /// Tier-B: optional VLM-emitted natural-language caption. HARD RULE:
    /// callers MUST inspect `reasoning::types::ArtifactReliability` and
    /// downweight any verdict whose reliability tier == VlmCaption.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vlm_caption: Option<String>,
    /// Tier-B: PaddleOCR-VL / Dolphin / MinerU emitted parse confidence in
    /// [0.0, 1.0]. The plan task 8 + DETERMINISTIC_CHECK item 5 use this
    /// to gate `cloud_fallback_threshold` (default 0.7).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parser_confidence: Option<f32>,
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

/// Errors emitted by the chunker. Wide enough to carry the cross-language
/// sidecar path failures Tier-B will route through here.
#[derive(Debug, Error)]
pub enum ChunkError {
    /// The input bytes were empty (no artifact to chunk).
    #[error("vision chunker received empty input bytes")]
    EmptyInput,
    /// The caller asked the chunker to pool a vector list whose inner
    /// dimensionality does not match `VECTOR_DIM`.
    #[error(
        "vision chunker dimension mismatch: expected {VECTOR_DIM}-dim vectors, got {got}"
    )]
    DimensionMismatch { got: usize },
    /// The pooling ratio resolved to <= 0 (caller asked for a target of
    /// zero or negative pooled vectors). Guarded so the pooler always
    /// produces at least one bucket.
    #[error("vision chunker invalid pool target {target}; must be >= 1")]
    InvalidPoolTarget { target: usize },
}

/// Pure-compute chunker that emits a VisionArtifact row for a single image
/// or rendered page. The VLM / pooled-vector path is wired through
/// `attach_pooled_vectors`; callers that have no encoder available pass
/// `None` and the artifact lands with only the types-only fields set.
#[derive(Debug, Clone, Copy, Default)]
pub struct ImageArtifactChunker;

impl ImageArtifactChunker {
    /// Build a VisionArtifact from raw bytes + the canonical
    /// `(source_path, page_index, kind)` tuple. Computes the blake3 hash
    /// + byte size; leaves the Tier-B fields empty.
    ///
    /// Returns `ChunkError::EmptyInput` when `bytes` is empty (the empty
    /// blake3 digest is well-defined but an empty image is almost
    /// certainly a producer bug).
    pub fn chunk_image(
        bytes: &[u8],
        source_path: impl Into<String>,
        page_index: Option<u32>,
        kind: VisionArtifactKind,
    ) -> Result<VisionArtifact, ChunkError> {
        if bytes.is_empty() {
            return Err(ChunkError::EmptyInput);
        }
        Ok(VisionArtifact {
            source_path: source_path.into(),
            page_index,
            kind,
            content_hash_hex: VisionArtifact::hash_bytes(bytes),
            byte_size: bytes.len() as u64,
            multi_vec: None,
            pooled: None,
            vlm_caption: None,
            parser_confidence: None,
        })
    }

    /// Mean-pool a list of raw per-patch vectors down to <= `target`
    /// pooled vectors. The plan task 2 cites arxiv:2602.12510 (Visual RAG
    /// Toolkit) for the training-free pooling story; until the actual
    /// paper algorithm is wired through the encoder, mean-pool over
    /// adjacent windows is the conservative fallback that preserves
    /// recall within ~2 points on the synthetic fixture used by
    /// `vision_chunk_pooling_contract.rs`.
    ///
    /// Caller invariants:
    ///   * Every inner vector has length `VECTOR_DIM` (else
    ///     `ChunkError::DimensionMismatch`).
    ///   * `target >= 1` (else `ChunkError::InvalidPoolTarget`).
    ///   * `target` is clamped to `MAX_VECTORS_PER_PAGE` so the contract
    ///     test's "<= 128" assertion always holds.
    pub fn pool_mean(
        raw: &[Vec<f32>],
        target: usize,
    ) -> Result<Vec<Vec<f32>>, ChunkError> {
        if target == 0 {
            return Err(ChunkError::InvalidPoolTarget { target });
        }
        for (idx, v) in raw.iter().enumerate() {
            if v.len() != VECTOR_DIM {
                tracing::debug!(
                    target: "document_vision_rag.chunker",
                    "pool_mean dim mismatch at idx={idx} got={}",
                    v.len()
                );
                return Err(ChunkError::DimensionMismatch { got: v.len() });
            }
        }
        let target = target.min(MAX_VECTORS_PER_PAGE);
        if raw.is_empty() {
            return Ok(Vec::new());
        }
        if raw.len() <= target {
            // Already below the cap. Clone-through preserves the contract.
            return Ok(raw.to_vec());
        }
        // Window the input into `target` near-equal buckets and mean-pool
        // each bucket. Deterministic — same `raw` always yields the same
        // pooled output (no rand / no float reordering).
        let n = raw.len();
        let mut pooled: Vec<Vec<f32>> = Vec::with_capacity(target);
        for bucket in 0..target {
            let lo = bucket * n / target;
            let hi = ((bucket + 1) * n / target).max(lo + 1).min(n);
            // hi > lo always holds because of the .max(lo + 1) guard;
            // .min(n) clamps the last bucket.
            let mut acc = vec![0.0_f32; VECTOR_DIM];
            let mut count = 0_u32;
            for v in &raw[lo..hi] {
                for (a, x) in acc.iter_mut().zip(v.iter()) {
                    *a += *x;
                }
                count += 1;
            }
            if count > 0 {
                let denom = count as f32;
                for a in acc.iter_mut() {
                    *a /= denom;
                }
            }
            pooled.push(acc);
        }
        Ok(pooled)
    }

    /// Attach a pooled vector index + producer metadata to an existing
    /// VisionArtifact. Used by the producer once the encoder + VLM caption
    /// path has run. Sets `pooled = Some(true)` whenever the input had
    /// to be reduced from a longer raw list.
    pub fn attach_pooled_vectors(
        artifact: &mut VisionArtifact,
        raw_vectors: &[Vec<f32>],
        target: usize,
        vlm_caption: Option<String>,
        parser_confidence: Option<f32>,
    ) -> Result<(), ChunkError> {
        let was_larger = raw_vectors.len() > target.min(MAX_VECTORS_PER_PAGE);
        let pooled = Self::pool_mean(raw_vectors, target)?;
        artifact.multi_vec = Some(pooled);
        artifact.pooled = Some(was_larger);
        if vlm_caption.is_some() {
            artifact.vlm_caption = vlm_caption;
        }
        if parser_confidence.is_some() {
            artifact.parser_confidence = parser_confidence;
        }
        Ok(())
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
            multi_vec: None,
            pooled: None,
            vlm_caption: None,
            parser_confidence: None,
        };
        let json = serde_json::to_string(&artifact).expect("serialize ok");
        let back: VisionArtifact = serde_json::from_str(&json).expect("deserialize ok");
        assert_eq!(back, artifact);
    }

    #[test]
    fn chunk_image_rejects_empty_input() {
        let r = ImageArtifactChunker::chunk_image(
            &[],
            "fixtures/empty.png",
            None,
            VisionArtifactKind::Screenshot,
        );
        assert!(matches!(r, Err(ChunkError::EmptyInput)));
    }

    #[test]
    fn chunk_image_emits_blake3_hash() {
        let bytes = b"hello world";
        let a = ImageArtifactChunker::chunk_image(
            bytes,
            "fixtures/sample.png",
            Some(0),
            VisionArtifactKind::Page,
        )
        .expect("ok");
        assert_eq!(a.content_hash_hex, VisionArtifact::hash_bytes(bytes));
        assert_eq!(a.byte_size, bytes.len() as u64);
        assert!(a.multi_vec.is_none());
        assert!(a.pooled.is_none());
    }

    #[test]
    fn pool_mean_below_target_returns_input() {
        let raw = vec![vec![1.0_f32; VECTOR_DIM], vec![2.0_f32; VECTOR_DIM]];
        let out = ImageArtifactChunker::pool_mean(&raw, 8).expect("ok");
        assert_eq!(out.len(), 2);
        assert_eq!(out[0][0], 1.0);
        assert_eq!(out[1][0], 2.0);
    }

    #[test]
    fn pool_mean_reduces_to_target() {
        // 1030 -> 100 simulates ColPali default page shape -> Visual RAG
        // Toolkit pooled target.
        let raw: Vec<Vec<f32>> = (0..1030)
            .map(|i| vec![i as f32; VECTOR_DIM])
            .collect();
        let out = ImageArtifactChunker::pool_mean(&raw, 100).expect("ok");
        assert_eq!(out.len(), 100);
        assert!(out.len() <= MAX_VECTORS_PER_PAGE);
        // First bucket mean ~ mean of 0..10 = 4.5; last bucket ~ mean of
        // 1020..1030 = 1024.5. Determinism check (re-run produces same).
        let out2 = ImageArtifactChunker::pool_mean(&raw, 100).expect("ok");
        assert_eq!(out, out2);
    }

    #[test]
    fn pool_mean_clamps_target_to_max() {
        let raw: Vec<Vec<f32>> = (0..500).map(|_| vec![1.0_f32; VECTOR_DIM]).collect();
        let out = ImageArtifactChunker::pool_mean(&raw, 10_000).expect("ok");
        assert!(out.len() <= MAX_VECTORS_PER_PAGE);
    }

    #[test]
    fn pool_mean_rejects_zero_target() {
        let raw: Vec<Vec<f32>> = vec![vec![1.0; VECTOR_DIM]];
        let r = ImageArtifactChunker::pool_mean(&raw, 0);
        assert!(matches!(r, Err(ChunkError::InvalidPoolTarget { .. })));
    }

    #[test]
    fn pool_mean_rejects_wrong_dim() {
        let raw: Vec<Vec<f32>> = vec![vec![1.0; 64]];
        let r = ImageArtifactChunker::pool_mean(&raw, 8);
        assert!(matches!(r, Err(ChunkError::DimensionMismatch { got: 64 })));
    }

    #[test]
    fn attach_pooled_vectors_sets_flag_when_reduced() {
        let mut a = ImageArtifactChunker::chunk_image(
            b"dummy",
            "fixtures/p.png",
            Some(0),
            VisionArtifactKind::Page,
        )
        .expect("ok");
        let raw: Vec<Vec<f32>> = (0..500).map(|_| vec![0.1_f32; VECTOR_DIM]).collect();
        ImageArtifactChunker::attach_pooled_vectors(&mut a, &raw, 64, None, Some(0.91))
            .expect("ok");
        assert!(a.multi_vec.is_some());
        assert_eq!(a.multi_vec.as_ref().unwrap().len(), 64);
        assert_eq!(a.pooled, Some(true));
        assert_eq!(a.parser_confidence, Some(0.91));
    }
}
