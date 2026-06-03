//! F-12 contract (killer-risk 1 — the rejected cloud-LLM judge MUST be
//! absent by construction):
//!
//! The adjudication path is a PURE LOCAL function. A cloud LLM judge
//! structurally breaks ELAI_SEED determinism (arXiv:2506.09501 —
//! temp=0+seed does not yield bit-identical cloud output; an auto-close
//! decision could flip between identical replays). So the semantic
//! contradiction modules must contain ZERO references to the cloud lane
//! or any network client.
//!
//! This is a Layer-2 STATIC leg (source grep over the two F-12 modules).
//! It is necessary-not-sufficient: the load-bearing backstop is that the
//! `NliJudge` trait `classify` is a synchronous local fn returning a
//! plain `Result` with no async / no http client in its signature (the
//! `adjudicate_semantic` smoke test drives the real path). The grep
//! guarantees no cloud branch was smuggled in.

use std::path::Path;

const BANNED_TOKENS: &[&str] = &[
    "cloud_mode",
    "route_with_spend",
    "CloudModeRouter",
    "anthropic",
    "Anthropic",
    "reqwest",
    "hyper::",
    "ANTHROPIC_BASE_URL",
    "/v1/messages",
];

fn module_path(rel: &str) -> std::path::PathBuf {
    // CARGO_MANIFEST_DIR for the contracts crate is .../crates/contracts.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("memory")
        .join("src")
        .join(rel)
}

fn assert_no_cloud_tokens(rel: &str) {
    let path = module_path(rel);
    let src = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {} failed: {e}", path.display()));
    for tok in BANNED_TOKENS {
        assert!(
            !src.contains(tok),
            "{} must not reference the cloud lane / a network client; found banned token {:?}. \
             The contradiction judge is LOCAL-only (killer-risk 1).",
            rel,
            tok
        );
    }
}

#[test]
fn semantic_contradiction_module_has_no_cloud_judge() {
    assert_no_cloud_tokens("semantic_contradiction.rs");
}

#[test]
fn nli_onnx_module_has_no_cloud_judge() {
    assert_no_cloud_tokens("nli_onnx.rs");
}

#[test]
fn source_files_actually_exist_and_were_scanned() {
    // R2 dead-evidence-path guard: a grep that scans nothing must FAIL, not
    // silently pass. Assert both files are present + non-trivial.
    for rel in ["semantic_contradiction.rs", "nli_onnx.rs"] {
        let path = module_path(rel);
        let src = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("F-12 module {} must exist: {e}", path.display()));
        assert!(
            src.len() > 200,
            "F-12 module {rel} is suspiciously small ({} bytes) — scan reached an empty file",
            src.len()
        );
    }
}
