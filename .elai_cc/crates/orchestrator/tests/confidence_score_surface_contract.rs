//! E-20 task 1 -- confidence score surface contract.
//!
//! Asserts the entropy-of-first-token-derived score from
//! [`elai_orchestrator::cost::cloud_route::confidence::compute_score`]
//! matches the expected behaviour across canonical inputs:
//!   * empty distribution -> 0.0 confidence (defensive default)
//!   * near-one-hot distribution -> > 0.85 confidence
//!   * uniform distribution -> ~ 0.0 confidence
//!   * `mean_logprob = -2.3` -> ~ 0.5 confidence (sigmoid pivot)
//!
//! Paper anchor: arXiv:2510.01237 + arXiv:2511.07396 (probe + perplexity
//! signals dominate verbalized-self-confidence). See
//! workflow/reasoning/E-20-decisions.md DECISION-4.

use elai_orchestrator::cost::cloud_route::confidence;

#[test]
fn empty_first_token_distribution_yields_zero_confidence() {
    let score = confidence::from_first_token_distribution(&[]);
    assert_eq!(score, 0.0);
}

#[test]
fn near_one_hot_distribution_is_high_confidence() {
    // 97% mass on a single token -> very low entropy.
    let probs = [0.97, 0.01, 0.01, 0.01];
    let score = confidence::from_first_token_distribution(&probs);
    assert!(score > 0.85, "got {score}");
}

#[test]
fn uniform_distribution_is_zero_confidence() {
    let probs = [0.25, 0.25, 0.25, 0.25];
    let score = confidence::from_first_token_distribution(&probs);
    assert!(score < 0.05, "got {score}");
}

#[test]
fn mean_logprob_pivot_is_half_confidence() {
    let score = confidence::from_mean_logprob(-2.3);
    assert!((score - 0.5).abs() < 0.05, "got {score}");
}

#[test]
fn compute_score_prefers_distribution() {
    // Same call site receives both a distribution AND a misleading
    // mean_logprob (-100, would give ~0 confidence). The distribution
    // wins.
    let probs = [0.97, 0.01, 0.01, 0.01];
    let with_dist = confidence::compute_score(Some(&probs), -100.0);
    let without_dist = confidence::compute_score(None, -100.0);
    assert!(with_dist > 0.5);
    assert!(without_dist < 0.5);
}

#[test]
fn confidence_is_bounded_zero_one() {
    let probs = [0.5, 0.5];
    let score = confidence::from_first_token_distribution(&probs);
    assert!((0.0..=1.0).contains(&score), "score = {score} out of range");
}
