//! E-20 task 8 -- sample-of-3 majority vote on routing decision
//! (NOT on the answer).
//!
//! Anchors: arXiv:2406.12334 same-prompt different-result inconsistency
//! at temp=0 + arXiv:2504.01282 Prompt-Reverse Inconsistency. The
//! majority-vote stabilises the routing decision at temp > 0 without
//! invoking 3 cloud completions for the actual answer (cheap routing
//! discipline).

use elai_orchestrator::cost::cloud_route::majority_vote::{
    self, N_ROUTING_SAMPLES, MajorityVoteOutcome,
};
use elai_orchestrator::cost::cloud_route::HybridEscalationVerdict;

#[test]
fn sample_count_is_three() {
    assert_eq!(N_ROUTING_SAMPLES, 3);
}

#[test]
fn strict_majority_wins() {
    use HybridEscalationVerdict::*;
    let samples = [SkipCloud, SkipCloud, EscalateCloud];
    let MajorityVoteOutcome {
        winner, flapping, ..
    } = majority_vote::route(&samples);
    assert_eq!(winner, SkipCloud);
    assert!(!flapping);
}

#[test]
fn unanimous_vote_wins() {
    use HybridEscalationVerdict::*;
    let samples = [EscalateOpus, EscalateOpus, EscalateOpus];
    let outcome = majority_vote::route(&samples);
    assert_eq!(outcome.winner, EscalateOpus);
    assert!(!outcome.flapping);
}

#[test]
fn three_way_split_is_flapping_and_conservative() {
    use HybridEscalationVerdict::*;
    let samples = [SkipCloud, EscalateCloud, EscalateOpus];
    let outcome = majority_vote::route(&samples);
    assert!(outcome.flapping);
    // Conservative bias: when in doubt, escalate.
    assert_eq!(outcome.winner, EscalateCloud);
}

#[test]
fn noisy_classifier_stabilises_via_majority() {
    use HybridEscalationVerdict::*;
    // Simulate a 50/50 oscillating classifier with 3 samples; 2 votes
    // for SkipCloud, 1 for EscalateCloud -> stable SkipCloud.
    let samples = [SkipCloud, EscalateCloud, SkipCloud];
    let outcome = majority_vote::route(&samples);
    assert_eq!(outcome.winner, SkipCloud);
    assert!(!outcome.flapping);
}

#[test]
fn tally_preserves_per_verdict_counts() {
    use HybridEscalationVerdict::*;
    let samples = [SkipCloud, EscalateCloud, EscalateCloud];
    let outcome = majority_vote::route(&samples);
    assert_eq!(outcome.tally.get(&SkipCloud).copied(), Some(1));
    assert_eq!(outcome.tally.get(&EscalateCloud).copied(), Some(2));
}
