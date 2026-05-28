//! E-20 task 3 -- BackendDifficultyAdapter classifier contract.
//!
//! Asserts that the deterministic heuristic classifier returns
//! sensible difficulty bins for the canonical easy / medium / hard
//! fixtures used by the cost-router decision matrix. Live RouteLLM
//! weights (arXiv:2406.18665) replace the heuristic via E-20.B.

use elai_providers::runtime::cloud_route_difficulty::{
    classify_difficulty, BackendDifficultyAdapter, DifficultyBin,
};

#[test]
fn empty_prompt_is_easy() {
    let verdict = classify_difficulty("");
    assert_eq!(verdict.bin, DifficultyBin::Easy);
    assert!(!verdict.cloud_recommended);
}

#[test]
fn very_short_repetitive_prompt_is_easy() {
    // Very short + low trigram variety + no frontier keywords ->
    // score below the 0.33 easy ceiling.
    let verdict = classify_difficulty("hi");
    assert_eq!(
        verdict.bin,
        DifficultyBin::Easy,
        "score = {} should be below easy ceil",
        verdict.score
    );
}

#[test]
fn keyword_heavy_long_prompt_is_hard() {
    let prompt = "Please prove correctness, optimize the algorithm, and derive the theorem \
                  for the lock-free concurrency primitive. Formal-verify the parallel monad. \
                  Refactor the architecture to be lock-free and monad-based and parallel.";
    let verdict = classify_difficulty(prompt);
    assert_eq!(verdict.bin, DifficultyBin::Hard);
    assert!(verdict.cloud_recommended);
}

#[test]
fn classifier_label_is_heuristic_until_routellm_ships() {
    let verdict = classify_difficulty("anything");
    assert_eq!(verdict.classifier, "heuristic");
}

#[test]
fn adapter_wraps_classify_helper() {
    let adapter = BackendDifficultyAdapter::new();
    let direct = classify_difficulty("fix it");
    let via_adapter = adapter.classify("fix it");
    assert_eq!(direct.bin, via_adapter.bin);
    assert!((direct.score - via_adapter.score).abs() < 1e-6);
}

#[test]
fn difficulty_score_is_bounded_zero_one() {
    let verdict = classify_difficulty("ordinary code change to a function body");
    assert!(verdict.score >= 0.0);
    assert!(verdict.score <= 1.0);
}

#[test]
fn bin_labels_stable() {
    assert_eq!(DifficultyBin::Easy.as_str(), "easy");
    assert_eq!(DifficultyBin::Medium.as_str(), "medium");
    assert_eq!(DifficultyBin::Hard.as_str(), "hard");
}
