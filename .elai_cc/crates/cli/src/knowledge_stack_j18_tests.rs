//! J-18 LEG2 slice 2 falsification test, split out of `knowledge_stack.rs`
//! to keep that file under the MR-1 1000-line boundary. It is a child module
//! of `knowledge_stack`, so `use super::*` reaches the private production
//! constructor it falsifies.

use super::*;
use elai_knowledge::lsp_channel::ELAI_LSP_CHANNEL_RRF_ENABLE_ENV;

/// J-18 LEG2 slice 2 falsification: the production constructor
/// `build_base_pipeline` MUST install an `LspChannelAdapter`. Deleting the
/// `.with_lsp_channel_adapter(..)` call site makes this test fail while the
/// crate still compiles, which is the mutation this test exists to catch.
#[test]
fn build_base_pipeline_installs_the_lsp_channel_adapter() {
    let tmp = tempfile::TempDir::new().expect("CONTRACT: temp workspace MUST be created");
    let bm25 = Bm25Index::new(tmp.path(), "j18_leg2_install")
        .expect("CONTRACT: BM25 fixture MUST initialize");
    let clock: Arc<dyn ClockProvider> = Arc::new(elai_core::OsClock);
    let ssb: Option<Arc<elai_orchestrator::Ssb>> = None;

    std::env::set_var(ELAI_LSP_CHANNEL_RRF_ENABLE_ENV, "1");
    let pipeline = build_base_pipeline(
        bm25,
        tmp.path(),
        &tmp.path().join("models"),
        "j18-leg2-install-smoke",
        &clock,
        &ssb,
    );
    let stages = pipeline
        .query("workspace symbol probe")
        .expect("CONTRACT: pipeline query MUST succeed");
    std::env::remove_var(ELAI_LSP_CHANNEL_RRF_ENABLE_ENV);

    assert!(
        stages.iter().any(|s| s.stage == "lsp_channel"),
        "CONTRACT: build_base_pipeline MUST install the LspChannelAdapter, so a flag-ON query MUST carry an lsp_channel stage"
    );
}
