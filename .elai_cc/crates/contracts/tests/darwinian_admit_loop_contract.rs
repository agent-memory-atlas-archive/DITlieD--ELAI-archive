//! F-11 contract: the config-DGM admit loop closes ONE generation.
//!
//! Feature-gated: compiles only with `--features darwinian-evolve` (forwarded
//! to elai-orchestrator via contracts/Cargo.toml). Without the feature this is
//! an empty crate so a default `cargo test -p elai_cc_contracts` is unaffected.
//!
//! Asserts:
//!   - a clean candidate is ADMITTED, `insert_variant` lands an archive row,
//!     and `select_parent` runs AFTER (loop_closed = true).
//!   - the admitted variant's score equals its derived alpha (the FIRST
//!     production caller of config_archive::insert_variant).
//!   - determinism: the candidate fingerprint (blake3) + derived alpha are
//!     stable across runs under a fixed seed.
#![cfg(feature = "darwinian-evolve")]

use elai_core::SeededRng;
use elai_orchestrator::config_archive::ConfigArchive;
use elai_orchestrator::darwin_admit_loop::{
    run_one_generation, MutantCandidate, DEFAULT_SUITE_NAME,
};
use elai_orchestrator::promotion_gate::PromotionConfig;
use elai_orchestrator::self_mod_layer1a::PipelineMode;
use elai_orchestrator::{Ssb, SsbConfig};

fn clean_candidate() -> MutantCandidate {
    MutantCandidate {
        variant_label: "clean-loop-001".to_string(),
        pipeline_mode: "surgical".to_string(),
        config: serde_json::json!({ "retry_budget": 3, "force_alpha": 0.6 }),
        eval_log: "suite=math-floors score=0.6 clean; no traps".to_string(),
    }
}

async fn open_archive(dir: &std::path::Path) -> ConfigArchive {
    ConfigArchive::open(&dir.join("self_mod.db"))
        .await
        .expect("archive open")
}

fn open_ssb(dir: &std::path::Path) -> Ssb {
    let cfg = SsbConfig {
        db_path: dir.join("ssb.db"),
        pool_size: None,
        project_id: "contract".to_string(),
        session_id: "darwin-loop-contract".to_string(),
    };
    let clock: std::sync::Arc<dyn elai_core::clock::ClockProvider> =
        std::sync::Arc::new(elai_core::OsClock);
    Ssb::new(cfg, clock).expect("ssb init")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn clean_candidate_admits_and_closes_loop() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let archive = open_archive(dir.path()).await;
    let ssb = open_ssb(dir.path());

    let candidate = clean_candidate();
    let cfg = PromotionConfig::default();
    let mut rng = SeededRng::new(42);
    let summary = run_one_generation(
        &archive,
        &ssb,
        std::slice::from_ref(&candidate),
        PipelineMode::Surgical,
        42,
        DEFAULT_SUITE_NAME,
        &cfg,
        &mut rng,
    )
    .await
    .expect("generation runs");

    assert_eq!(summary.admitted, 1, "clean candidate must be admitted");
    assert_eq!(summary.rejected, 0, "clean candidate must not be rejected");
    assert!(summary.loop_closed, "select_parent must run after insert_variant");

    // insert_variant's FIRST production caller landed a real archive row.
    let stored = archive
        .get_variant("clean-loop-001")
        .await
        .expect("get_variant")
        .expect("the admitted variant must be in the archive");
    assert_eq!(stored.variant_id, "clean-loop-001");
    // Score equals the derived alpha (forced to 0.6 in the fixture).
    assert!((stored.score - 0.6).abs() < 1e-5, "score {} != 0.6", stored.score);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn candidate_fingerprint_and_alpha_are_deterministic() {
    // Two independent runs over the same candidate set must produce identical
    // fingerprints + alpha (ELAI_SEED determinism: blake3 fingerprint +
    // candidate-derived alpha have no clock/RNG input).
    let candidate = clean_candidate();
    let a1 = candidate.candidate_blake3();
    let a2 = candidate.candidate_blake3();
    assert_eq!(a1, a2);
    assert_eq!(a1.len(), 16);

    let alpha1 = candidate.derived_alpha();
    let alpha2 = candidate.derived_alpha();
    assert_eq!(alpha1, alpha2);
    assert!((0.0..=1.0).contains(&alpha1));
}
