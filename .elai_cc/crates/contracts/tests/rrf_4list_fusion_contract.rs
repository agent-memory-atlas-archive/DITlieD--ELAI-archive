//! E-36 Tier-B — rrf_4list_fusion_contract.
//!
//! Locks plan task 6 + ACCEPTANCE row: "4-list RRF with vision_vec
//! produces stable ordering; deterministic across 3 runs".
//!
//! Plan default weights (per [knowledge.rrf] weights):
//!   bm25       = 1.0
//!   code_vec   = 1.0
//!   text_vec   = 0.8
//!   vision_vec = 0.6
//!
//! The vision_vec weight is intentionally LOWER than text_vec because
//! REAL-MM-RAG (arxiv:2502.12342) reports vision-retriever brittleness
//! on text-heavy / table queries. The contract here pins the shipping
//! defaults so a future refactor cannot silently widen the vision
//! weight above the text weight.

use elai_knowledge::vector::{rrf_score_4_weighted, RrfWeights4, RRF_K};

#[test]
fn default_weights_match_plan_task_6_defaults() {
    let w = RrfWeights4::default();
    assert_eq!(w.bm25, 1.0);
    assert_eq!(w.code_vec, 1.0);
    assert_eq!(w.text_vec, 0.8);
    assert_eq!(w.vision_vec, 0.6);
}

#[test]
fn vision_weight_must_be_lte_text_weight_per_plan() {
    // Plan task 6 + REAL-MM-RAG grounding: vision_vec <= text_vec.
    let w = RrfWeights4::default();
    assert!(
        w.vision_vec <= w.text_vec,
        "vision_vec ({}) must be <= text_vec ({}) per plan task 6 / arxiv:2502.12342",
        w.vision_vec,
        w.text_vec
    );
}

#[test]
fn rrf_k_constant_is_60_per_cormack_2009() {
    // Cormack et al. SIGIR'09: RRF constant k=60 is the canonical default.
    assert_eq!(RRF_K, 60.0);
}

#[test]
fn rrf_score_4_weighted_sums_all_lists() {
    let w = RrfWeights4::default();
    // Candidate ranked 1 in all 4 lists.
    let s = rrf_score_4_weighted(Some(1), Some(1), Some(1), Some(1), &w);
    // Expected: 1.0/61 + 1.0/61 + 0.8/61 + 0.6/61 = 3.4/61
    let expected = (w.bm25 + w.code_vec + w.text_vec + w.vision_vec) / (RRF_K + 1.0);
    assert!((s - expected).abs() < 1e-6, "got {s}, expected {expected}");
}

#[test]
fn absent_list_contributes_zero() {
    let w = RrfWeights4::default();
    // Candidate only in bm25 at rank 1.
    let s = rrf_score_4_weighted(Some(1), None, None, None, &w);
    let expected = w.bm25 / (RRF_K + 1.0);
    assert!((s - expected).abs() < 1e-6);
}

#[test]
fn rrf_score_is_deterministic_across_runs() {
    // Plan ACCEPTANCE row: "deterministic across 3 runs".
    let w = RrfWeights4::default();
    let s1 = rrf_score_4_weighted(Some(2), Some(5), Some(1), Some(3), &w);
    let s2 = rrf_score_4_weighted(Some(2), Some(5), Some(1), Some(3), &w);
    let s3 = rrf_score_4_weighted(Some(2), Some(5), Some(1), Some(3), &w);
    assert_eq!(s1, s2);
    assert_eq!(s2, s3);
}

#[test]
fn higher_rank_yields_lower_score() {
    let w = RrfWeights4::default();
    let s_at_1 = rrf_score_4_weighted(Some(1), None, None, None, &w);
    let s_at_5 = rrf_score_4_weighted(Some(5), None, None, None, &w);
    let s_at_10 = rrf_score_4_weighted(Some(10), None, None, None, &w);
    assert!(s_at_1 > s_at_5);
    assert!(s_at_5 > s_at_10);
}

#[test]
fn ordering_is_stable_with_vision_lane() {
    let w = RrfWeights4::default();
    // Candidate A: rank 1 in BM25 + text_vec, absent from code + vision.
    let a = rrf_score_4_weighted(Some(1), None, Some(1), None, &w);
    // Candidate B: rank 1 in all four lists. B should beat A.
    let b = rrf_score_4_weighted(Some(1), Some(1), Some(1), Some(1), &w);
    assert!(b > a, "B with vision lane must outscore A without it");
}

#[test]
fn custom_weights_round_trip_through_serde() {
    let w = RrfWeights4 {
        bm25: 1.2,
        code_vec: 0.9,
        text_vec: 0.7,
        vision_vec: 0.5,
    };
    let json = serde_json::to_string(&w).expect("serialize");
    let back: RrfWeights4 = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(w, back);
}
