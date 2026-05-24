//! E-17 SP3 task 3 — Jina-rerank-v3 / BGE-v2-m3 graceful-degradation contract.
//!
//! Verifies that:
//!   (a) Selecting `ReRankerModel::JinaV3` when the ONNX file is NOT staged
//!       on disk produces `Err(ModelLoad)` rather than a panic. The
//!       JINA-MODEL-PENDING marker on `ReRankerModel::default()` pins
//!       MsMarcoMiniLmL12 as the default precisely so this graceful-degrade
//!       path stays unobserved by the default install.
//!   (b) Same shape for `ReRankerModel::BgeV2M3` — when the optional
//!       `onnx-community/bge-reranker-v2-m3-ONNX` model is not staged,
//!       loading returns `Err` not panic.
//!   (c) Deterministic enum<->path mapping: calling `as_str()` and
//!       `onnx_filename()` 100 times returns identical strings every time
//!       (no allocation-dependent ordering, no thread-local race).
//!
//! Honest framing: this test does NOT load the actual ONNX file — that
//! requires the model on disk and would skip in CI. It exercises the
//! enum + config + load-attempt surface that the live pipeline depends
//! on, mirroring the staging-fixture pattern at reranker.rs:1062-1075
//! (load_zero_byte_model_returns_err).
//!
//! The "3-trial determinism" check called out in the plan (per
//! reranker.rs:1101 std-dev < 1e-6 pattern) is a property of the enum
//! identity rather than the inference outputs — without the staged ONNX
//! we cannot reproduce a real score; we instead assert that the
//! variant->path mapping is bit-identical across 3+ trials.

use elai_core::OsClock;
use elai_knowledge::degradation::SsbNotifier;
use elai_knowledge::reranker::{
    CrossEncoderReranker, DegradationEvent, ReRankerModel, RerankerConfig, RerankerError,
};
use std::sync::Arc;
use tempfile::TempDir;

/// Test-only SSB notifier that drops all events. Production callers wire
/// a real SsbNotifier via `CrossEncoderReranker::with_ssb`.
struct DropSsbNotifier;

impl SsbNotifier for DropSsbNotifier {
    fn emit(&self, _event: DegradationEvent) {}
}

#[test]
fn jina_v3_load_returns_err_when_file_missing() {
    let tmp = TempDir::new().expect("tempdir");
    let cfg = RerankerConfig::with_project_root_and_variant(tmp.path(), ReRankerModel::JinaV3);
    let mut reranker =
        CrossEncoderReranker::with_ssb(cfg, Arc::new(OsClock), Arc::new(DropSsbNotifier));
    let result = reranker.load();
    assert!(
        matches!(result, Err(RerankerError::ModelLoad(_))),
        "JinaV3 load with unstaged ONNX must return Err(ModelLoad); got {result:?}"
    );
}

#[test]
fn bge_v2_m3_load_returns_err_when_file_missing() {
    let tmp = TempDir::new().expect("tempdir");
    let cfg = RerankerConfig::with_project_root_and_variant(tmp.path(), ReRankerModel::BgeV2M3);
    let mut reranker =
        CrossEncoderReranker::with_ssb(cfg, Arc::new(OsClock), Arc::new(DropSsbNotifier));
    let result = reranker.load();
    assert!(
        matches!(result, Err(RerankerError::ModelLoad(_))),
        "BgeV2M3 load with unstaged ONNX must return Err(ModelLoad); got {result:?}"
    );
}

#[test]
fn variant_path_mapping_is_deterministic_across_trials() {
    // Replays the reranker.rs:1101 "std-dev < 1e-6" pattern but for the
    // deterministic enum->string mapping (no ONNX inference required).
    for variant in [
        ReRankerModel::JinaV3,
        ReRankerModel::BgeV2M3,
        ReRankerModel::MsMarcoMiniLmL12,
    ] {
        let baseline_str = variant.as_str().to_string();
        let baseline_file = variant.onnx_filename().to_string();
        for trial in 1..=100 {
            assert_eq!(
                variant.as_str(),
                baseline_str,
                "trial {trial}: {variant:?}.as_str() drift"
            );
            assert_eq!(
                variant.onnx_filename(),
                baseline_file,
                "trial {trial}: {variant:?}.onnx_filename() drift"
            );
        }
    }
}

#[test]
fn jina_v3_config_path_documents_expected_staged_location() {
    let tmp = TempDir::new().expect("tempdir");
    let cfg = RerankerConfig::with_project_root_and_variant(tmp.path(), ReRankerModel::JinaV3);
    let path_str = cfg.model_path.display().to_string();
    assert!(
        path_str.ends_with("jina-rerank-v3.onnx"),
        "config.model_path must end with the documented filename; got {path_str}"
    );
    // Documents JINA-MODEL-PENDING: this path is where the file MUST land
    // if a future official ONNX export ships and the default is flipped.
}

#[test]
fn ms_marco_minilm_l12_default_preserved() {
    // Pinned ACCEPTANCE-bullet-3 contract: the default reranker MUST stay
    // MsMarcoMiniLmL12 until JINA-MODEL-PENDING resolves. Flipping the
    // default before staging the new ONNX file would silently degrade
    // every install (reranker.rs Default impl doc-comment for context).
    assert_eq!(ReRankerModel::default(), ReRankerModel::MsMarcoMiniLmL12);
}
