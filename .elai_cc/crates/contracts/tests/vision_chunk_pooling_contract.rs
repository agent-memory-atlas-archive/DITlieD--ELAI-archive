//! E-36 Tier-B — vision_chunk_pooling_contract.
//!
//! Locks plan task 2 + DETERMINISTIC_CHECK item 5: 1030-vector input
//! pools to <= 128 vectors output AND the pooled cap stays at the
//! shipping constant. If a future refactor widens MAX_VECTORS_PER_PAGE
//! beyond 128 this contract fails loudly.

use elai_knowledge::chunking::image_artifact::{
    ChunkError, ImageArtifactChunker, VisionArtifactKind, MAX_VECTORS_PER_PAGE, VECTOR_DIM,
};

#[test]
fn max_vectors_per_page_is_at_most_128() {
    // Plan DETERMINISTIC_CHECK item 5: "pooling constant <= 128 visible".
    assert!(
        MAX_VECTORS_PER_PAGE <= 128,
        "MAX_VECTORS_PER_PAGE must be <= 128 per plan task 2 DETERMINISTIC_CHECK item 5"
    );
}

#[test]
fn vector_dim_matches_colpali_family() {
    // ColModernVBERT + ColQwen2.5-v0.2 both emit 128-dim per-patch embeddings
    // per their published model cards.
    assert_eq!(VECTOR_DIM, 128);
}

#[test]
fn pool_mean_reduces_1030_to_100() {
    // 1030 -> 100 is the canonical ColPali default page shape ->
    // Visual RAG Toolkit pooled target (arxiv:2602.12510).
    let raw: Vec<Vec<f32>> = (0..1030).map(|i| vec![i as f32; VECTOR_DIM]).collect();
    let out = ImageArtifactChunker::pool_mean(&raw, 100).expect("ok");
    assert_eq!(out.len(), 100);
    assert!(out.len() <= MAX_VECTORS_PER_PAGE);
}

#[test]
fn pool_mean_is_deterministic_across_runs() {
    // Plan ACCEPTANCE: "deterministic across 3 runs" for the RRF contract;
    // we extend the rule here for pooling so re-indexing the same artifact
    // produces the same multi_vec bytes (cache-friendly).
    let raw: Vec<Vec<f32>> = (0..500)
        .map(|i| vec![(i as f32) * 0.1_f32; VECTOR_DIM])
        .collect();
    let r1 = ImageArtifactChunker::pool_mean(&raw, 80).expect("ok");
    let r2 = ImageArtifactChunker::pool_mean(&raw, 80).expect("ok");
    let r3 = ImageArtifactChunker::pool_mean(&raw, 80).expect("ok");
    assert_eq!(r1, r2);
    assert_eq!(r2, r3);
}

#[test]
fn pool_mean_clamps_at_max_vectors_per_page() {
    // Even when the caller asks for 10_000 buckets, the cap holds.
    let raw: Vec<Vec<f32>> = (0..500).map(|_| vec![1.0_f32; VECTOR_DIM]).collect();
    let out = ImageArtifactChunker::pool_mean(&raw, 10_000).expect("ok");
    assert!(out.len() <= MAX_VECTORS_PER_PAGE);
}

#[test]
fn pool_mean_passes_through_when_already_below_target() {
    let raw: Vec<Vec<f32>> = (0..50).map(|i| vec![i as f32; VECTOR_DIM]).collect();
    let out = ImageArtifactChunker::pool_mean(&raw, 100).expect("ok");
    assert_eq!(out.len(), 50);
    // Same per-row values preserved.
    for (idx, row) in out.iter().enumerate() {
        assert_eq!(row[0], idx as f32);
    }
}

#[test]
fn pool_mean_rejects_dim_mismatch() {
    let raw: Vec<Vec<f32>> = vec![vec![1.0_f32; 64]];
    let r = ImageArtifactChunker::pool_mean(&raw, 16);
    assert!(matches!(r, Err(ChunkError::DimensionMismatch { got: 64 })));
}

#[test]
fn attach_pooled_vectors_sets_pooled_flag_when_reduced() {
    let mut artifact = ImageArtifactChunker::chunk_image(
        b"page bytes",
        "fixtures/p.png",
        Some(0),
        VisionArtifactKind::Page,
    )
    .expect("chunk ok");
    let raw: Vec<Vec<f32>> = (0..400).map(|_| vec![0.5_f32; VECTOR_DIM]).collect();
    ImageArtifactChunker::attach_pooled_vectors(&mut artifact, &raw, 64, None, Some(0.93))
        .expect("attach ok");
    assert_eq!(artifact.pooled, Some(true));
    assert_eq!(artifact.multi_vec.as_ref().unwrap().len(), 64);
    assert_eq!(artifact.parser_confidence, Some(0.93));
}
