//! D-16 — observable_effect integration test for the UniCR
//! conformal-refusal calibration.
//!
//! Constructs a 20-tuple synthetic calibration corpus, runs
//! `compute_tau_and_emit` + `decide_and_emit` through the
//! `agent_pool::dispatch_conformal_refusal` producer, and asserts:
//!
//!   1. exactly one `ConformalTauComputed` SSB payload is recorded by
//!      the tau emitter (tau in [0, 1], correct epsilon, correct
//!      corpus_size, snake-case source label).
//!   2. a low-confidence claim score routes to
//!      `RefusalDecision::Abstain` and emits one
//!      `ConformalRefusalDecided` row with `decision == "abstain"`.
//!   3. a high-confidence claim score routes to `Answer` and emits a
//!      second `ConformalRefusalDecided` row with `decision == "answer"`
//!      and the original claim_score preserved as `confidence`.
//!
//! Grounding: arXiv:2503.01345 (UniCR — unified conformal refusal) +
//! Vovk et al. *Algorithmic Learning in a Random World* (2005) §2.2.

use elai_orchestrator::{
    agent_pool::dispatch_conformal_refusal,
    calibration::{
        CalibrationCorpus, CalibrationTuple, CorpusSource, RefusalDecision,
        VecConformalRefusalDecidedEmitter, VecConformalTauComputedEmitter, DEFAULT_EPSILON,
    },
};

fn balanced_corpus(n_pairs: usize) -> CalibrationCorpus {
    let mut c = CalibrationCorpus::new(CorpusSource::HeldOut);
    for i in 0..n_pairs {
        c.append(CalibrationTuple::new(
            format!("t-{i}"),
            format!("c-{i}"),
            "true claim",
            true,
            0.9,
            1_700_000_000 + (i as i64),
        ))
        .expect("append true tuple");
        c.append(CalibrationTuple::new(
            format!("f-{i}"),
            format!("d-{i}"),
            "false claim",
            false,
            0.2,
            1_700_000_500 + (i as i64),
        ))
        .expect("append false tuple");
    }
    c
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dispatch_conformal_refusal_emits_tau_and_decisions_for_low_then_high_score() {
    // 20 pairs = 40 tuples; ε = 0.1 (default).
    let corpus = balanced_corpus(20);

    let tau_emitter = VecConformalTauComputedEmitter::new();
    let dec_emitter = VecConformalRefusalDecidedEmitter::new();

    // --- low-score claim → Abstain ---
    let d_low = dispatch_conformal_refusal(
        "claim-low",
        0.01,
        &corpus,
        DEFAULT_EPSILON,
        &tau_emitter,
        &dec_emitter,
        1_700_000_900_000,
    )
    .await
    .expect("dispatch ok (low)");
    assert!(matches!(d_low, RefusalDecision::Abstain));

    // --- high-score claim → Answer ---
    let d_high = dispatch_conformal_refusal(
        "claim-high",
        0.97,
        &corpus,
        DEFAULT_EPSILON,
        &tau_emitter,
        &dec_emitter,
        1_700_000_901_000,
    )
    .await
    .expect("dispatch ok (high)");
    match d_high {
        RefusalDecision::Answer { confidence } => {
            assert!((confidence - 0.97).abs() < 1e-9);
        }
        RefusalDecision::Abstain => panic!("expected Answer for high-confidence claim"),
    }

    // SSB rows: 2 tau (one per dispatch) + 2 refusal (one per dispatch).
    let tau_rows = tau_emitter.rows().await;
    assert_eq!(tau_rows.len(), 2, "expected 2 tau rows");
    for row in &tau_rows {
        assert!(
            (row.tau - 0.8).abs() < 1e-9,
            "expected tau 0.8 from balanced corpus; got {}",
            row.tau
        );
        assert!((row.epsilon - DEFAULT_EPSILON).abs() < 1e-12);
        assert_eq!(row.corpus_size, 40);
        assert_eq!(row.source, "held_out");
    }

    let dec_rows = dec_emitter.rows().await;
    assert_eq!(dec_rows.len(), 2, "expected 2 refusal rows");
    let low = &dec_rows[0];
    assert_eq!(low.claim_id, "claim-low");
    assert_eq!(low.decision, "abstain");
    assert!((low.claim_score - 0.01).abs() < 1e-9);
    assert!((low.tau - 0.8).abs() < 1e-9);
    assert_eq!(low.confidence, 0.0);

    let high = &dec_rows[1];
    assert_eq!(high.claim_id, "claim-high");
    assert_eq!(high.decision, "answer");
    assert!((high.confidence - 0.97).abs() < 1e-9);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn load_seed_jsonl_corpus_supports_tau_and_decide() {
    // Load the shipped seed corpus directly off disk so the test
    // verifies the JSONL round-trip path the orchestrator boots from.
    let repo_root = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR set under cargo test");
    // CARGO_MANIFEST_DIR points at crates/orchestrator/; walk up to
    // .elai_cc/.
    let manifest_dir = std::path::PathBuf::from(repo_root);
    let elai_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("walk up to .elai_cc/");
    let seed_path = elai_root.join("calibration/seed/calibration_seed.jsonl");

    let corpus = CalibrationCorpus::load_seed_jsonl(&seed_path)
        .expect("seed corpus must load from the shipped path");
    assert!(corpus.size() >= 20, "seed corpus has at least 20 tuples");
    assert!(corpus.frozen, "load_seed_jsonl returns a frozen corpus");

    let tau_emitter = VecConformalTauComputedEmitter::new();
    let dec_emitter = VecConformalRefusalDecidedEmitter::new();
    let d = dispatch_conformal_refusal(
        "claim-seed-low",
        0.01,
        &corpus,
        DEFAULT_EPSILON,
        &tau_emitter,
        &dec_emitter,
        1_700_000_902_000,
    )
    .await
    .expect("dispatch ok on seed corpus");
    assert!(matches!(d, RefusalDecision::Abstain));

    let tau_rows = tau_emitter.rows().await;
    assert_eq!(tau_rows.len(), 1);
    let r = &tau_rows[0];
    assert!(r.tau.is_finite() && r.tau >= 0.0 && r.tau <= 1.0);
    assert_eq!(r.source, "synthetic");
}
