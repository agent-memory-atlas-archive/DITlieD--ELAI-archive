//! E-17 SP2 task 3 — ReRankerModel enum + with_project_root contract test.
//!
//! Verifies:
//!   - Each variant has a stable string id + ONNX filename.
//!   - `RerankerConfig::with_project_root` composes the expected absolute
//!     path under `<project_root>/.elai_cc/models/onnx/`.
//!   - `with_project_root_and_variant` swaps the model_path correctly.
//!   - Default = MsMarcoMiniLmL12.

use elai_knowledge::reranker::{ReRankerModel, RerankerConfig};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn enum_default_is_ms_marco_minilm_l12() {
    assert_eq!(ReRankerModel::default(), ReRankerModel::MsMarcoMiniLmL12);
}

#[test]
fn variant_string_identifiers_are_stable() {
    assert_eq!(ReRankerModel::JinaV3.as_str(), "jina-rerank-v3");
    assert_eq!(ReRankerModel::BgeV2M3.as_str(), "bge-reranker-v2-m3");
    assert_eq!(
        ReRankerModel::MsMarcoMiniLmL12.as_str(),
        "ms-marco-minilm-l12"
    );
    assert_eq!(
        ReRankerModel::ScoringHeadDistilled.as_str(),
        "scoring-head-distilled"
    );
}

#[test]
fn variant_onnx_filenames_are_correct() {
    assert_eq!(ReRankerModel::JinaV3.onnx_filename(), "jina-rerank-v3.onnx");
    assert_eq!(
        ReRankerModel::BgeV2M3.onnx_filename(),
        "bge-reranker-v2-m3.onnx",
    );
    assert_eq!(
        ReRankerModel::MsMarcoMiniLmL12.onnx_filename(),
        "cross-encoder-l12.onnx",
    );
    assert_eq!(
        ReRankerModel::ScoringHeadDistilled.onnx_filename(),
        "scoring-head-distilled.onnx",
    );
}

#[test]
fn with_project_root_composes_expected_paths() {
    let tmp = TempDir::new().expect("tempdir");
    let project_root = tmp.path();
    let cfg = RerankerConfig::with_project_root(project_root);

    let expected_models: PathBuf = project_root
        .join(".elai_cc")
        .join("models")
        .join("onnx");
    assert_eq!(cfg.model_path, expected_models.join("cross-encoder-l12.onnx"));
    assert_eq!(cfg.tokenizer_path, expected_models.join("tokenizer.json"));
    let fallback = cfg.fallback_model_path.expect("fallback path set");
    assert_eq!(fallback, expected_models.join("cross-encoder-l6.onnx"));
    assert_eq!(cfg.model_variant, ReRankerModel::MsMarcoMiniLmL12);
}

#[test]
fn with_project_root_and_variant_swaps_path() {
    let tmp = TempDir::new().expect("tempdir");
    let project_root = tmp.path();
    let cfg = RerankerConfig::with_project_root_and_variant(project_root, ReRankerModel::JinaV3);
    let expected = project_root
        .join(".elai_cc")
        .join("models")
        .join("onnx")
        .join("jina-rerank-v3.onnx");
    assert_eq!(cfg.model_path, expected);
    assert_eq!(cfg.model_variant, ReRankerModel::JinaV3);
}

#[test]
fn with_project_root_and_variant_scoring_head_distilled_path() {
    let tmp = TempDir::new().expect("tempdir");
    let project_root = tmp.path();
    let cfg = RerankerConfig::with_project_root_and_variant(
        project_root,
        ReRankerModel::ScoringHeadDistilled,
    );
    let expected = project_root
        .join(".elai_cc")
        .join("models")
        .join("onnx")
        .join("scoring-head-distilled.onnx");
    assert_eq!(cfg.model_path, expected);
    assert_eq!(cfg.model_variant, ReRankerModel::ScoringHeadDistilled);
}

#[test]
fn serde_roundtrip_for_all_variants() {
    for variant in [
        ReRankerModel::JinaV3,
        ReRankerModel::BgeV2M3,
        ReRankerModel::MsMarcoMiniLmL12,
        ReRankerModel::ScoringHeadDistilled,
    ] {
        let json = serde_json::to_string(&variant).expect("serialize");
        let back: ReRankerModel = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(variant, back, "round-trip failed for {variant:?}");
    }
}

#[test]
fn serde_scoring_head_distilled_kebab_case() {
    let json = serde_json::to_string(&ReRankerModel::ScoringHeadDistilled).expect("serialize");
    assert_eq!(json, "\"scoring-head-distilled\"");
    let back: ReRankerModel = serde_json::from_str("\"scoring-head-distilled\"").expect("deserialize");
    assert_eq!(back, ReRankerModel::ScoringHeadDistilled);
}
