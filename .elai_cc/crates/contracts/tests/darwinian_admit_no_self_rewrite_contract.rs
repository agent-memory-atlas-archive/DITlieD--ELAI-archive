//! F-11 contract: the admit loop produces a `ConfigVariant`, NEVER a Layer-3
//! `StructuralChange` — so the mutant never reaches
//! `self_mod_safety::human_approved` and stays inside the 2510.04399
//! bounded-capacity (config-only) regime by construction.
//!
//! Feature-gated (`darwinian-evolve`). The check is type-level: an admitted
//! candidate lands as a `config_archive::ConfigVariant` row whose `config_blob`
//! is the serialized config delta (bytes), with no agent-source field. We
//! assert the archive row is a ConfigVariant and that its blob round-trips as
//! the config JSON (a config/template delta), not a structural code change.
#![cfg(feature = "darwinian-evolve")]

use elai_core::SeededRng;
use elai_orchestrator::config_archive::ConfigArchive;
use elai_orchestrator::darwin_admit_loop::{
    run_one_generation, MutantCandidate, DEFAULT_SUITE_NAME,
};
use elai_orchestrator::promotion_gate::PromotionConfig;
use elai_orchestrator::self_mod_layer1a::PipelineMode;
use elai_orchestrator::self_mod_types::VariantStatus;
use elai_orchestrator::{Ssb, SsbConfig};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn admitted_mutant_is_a_config_variant_not_structural() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let archive = ConfigArchive::open(&dir.path().join("self_mod.db"))
        .await
        .expect("archive open");
    let ssb_cfg = SsbConfig {
        db_path: dir.path().join("ssb.db"),
        pool_size: None,
        project_id: "contract".to_string(),
        session_id: "no-self-rewrite".to_string(),
    };
    let clock: std::sync::Arc<dyn elai_core::clock::ClockProvider> =
        std::sync::Arc::new(elai_core::OsClock);
    let ssb = Ssb::new(ssb_cfg, clock).expect("ssb");

    let candidate = MutantCandidate {
        variant_label: "config-only-001".to_string(),
        pipeline_mode: "surgical".to_string(),
        config: serde_json::json!({ "retry_budget": 5, "force_alpha": 0.65 }),
        eval_log: "clean config delta".to_string(),
    };
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
    .expect("generation");
    assert_eq!(summary.admitted, 1);

    // The archive row is a ConfigVariant (config/template delta), Active.
    let stored = archive
        .get_variant("config-only-001")
        .await
        .expect("get_variant")
        .expect("admitted variant present");
    assert_eq!(stored.status, VariantStatus::Active);

    // The config_blob is the serialized config JSON (a config delta), and it
    // round-trips as a JSON object — NOT a code/StructuralChange payload.
    let blob_json: serde_json::Value =
        serde_json::from_slice(&stored.config_blob).expect("config_blob is JSON config delta");
    assert!(
        blob_json.is_object(),
        "config_blob must be a config object, got {blob_json:?}"
    );
    assert_eq!(blob_json.get("retry_budget").and_then(|v| v.as_i64()), Some(5));
}
