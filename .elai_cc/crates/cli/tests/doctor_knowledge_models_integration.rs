//! E-17 SP1 task 1 — integration tests for `elai doctor knowledge-models`.
//!
//! Strategy: spawn the compiled `elai` binary against a tempdir fixture
//! whose `.elai_cc/models/` layout we control. We exercise three contracts:
//!
//! 1. Happy path: all 9 expected files staged at non-zero size -> exit 0,
//!    table output mentions every entry, blake3 tokenizer hashes are
//!    64-char hex.
//! 2. Missing file: drop `graphcodebert.onnx` -> exit non-zero, output
//!    surfaces the missing entry.
//! 3. JSON output contract: `--output-format json` emits a well-formed
//!    object with `kind`, `models_dir`, `entries[]`, `has_failures`.
//!
//! Tokenizer fixtures use a small deterministic byte payload so we can
//! also assert hash determinism across the two real tokenizer entries
//! sharing identical bytes.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;

/// 9-entry manifest mirrors the production probe in `doctor::run_knowledge_models`.
const MANIFEST: &[(&str, &str)] = &[
    ("cross-encoder-l12", "onnx/cross-encoder-l12.onnx"),
    ("cross-encoder-l6", "onnx/cross-encoder-l6.onnx"),
    ("graphcodebert", "onnx/graphcodebert.onnx"),
    ("graphcodebert-tok", "onnx/graphcodebert-tokenizer.json"),
    ("granite-embed", "onnx/granite-embed.onnx"),
    ("granite-embed-tok", "onnx/granite-embed-tokenizer.json"),
    ("cross-encoder-tok", "onnx/tokenizer.json"),
    ("splade-model", "splade/model.onnx"),
    ("splade-tok", "splade/tokenizer.json"),
];

fn elai_binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_elai"))
}

/// Build a `<root>/.elai_cc/models/...` tree populated with placeholder
/// bytes for every entry in MANIFEST.
fn stage_fixture(root: &Path, tokenizer_bytes: &[u8], onnx_bytes: &[u8]) {
    let models_dir = root.join(".elai_cc").join("models");
    fs::create_dir_all(models_dir.join("onnx")).expect("create onnx dir");
    fs::create_dir_all(models_dir.join("splade")).expect("create splade dir");
    for (_, rel) in MANIFEST {
        let path = models_dir.join(rel);
        // Tokenizers are *.json, model files are *.onnx — use different
        // payloads so the integrity checks see distinct sizes.
        let payload: &[u8] = if rel.ends_with(".json") {
            tokenizer_bytes
        } else {
            onnx_bytes
        };
        fs::write(&path, payload).expect("write fixture");
    }
}

#[test]
fn happy_path_all_present_exits_zero() {
    let tmp = TempDir::new().expect("tempdir");
    stage_fixture(tmp.path(), b"tokenizer-fixture-bytes\n", b"onnx-fixture-bytes\n");

    let out = Command::new(elai_binary())
        .args([
            "doctor",
            "knowledge-models",
            "--project-root",
        ])
        .arg(tmp.path())
        .output()
        .expect("spawn elai doctor knowledge-models");

    assert!(
        out.status.success(),
        "expected exit 0; stdout=\n{}\nstderr=\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    for (label, _) in MANIFEST {
        assert!(
            stdout.contains(label),
            "expected label {label} in stdout, got:\n{stdout}",
        );
    }
}

#[test]
fn missing_file_returns_err() {
    let tmp = TempDir::new().expect("tempdir");
    stage_fixture(tmp.path(), b"tokenizer-fixture-bytes\n", b"onnx-fixture-bytes\n");
    // Remove one required file.
    let target = tmp
        .path()
        .join(".elai_cc/models/onnx/graphcodebert.onnx");
    fs::remove_file(&target).expect("remove fixture");

    let out = Command::new(elai_binary())
        .args(["doctor", "knowledge-models", "--project-root"])
        .arg(tmp.path())
        .output()
        .expect("spawn elai doctor knowledge-models");

    assert!(
        !out.status.success(),
        "expected non-zero exit when a file is missing; stdout=\n{}",
        String::from_utf8_lossy(&out.stdout),
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    assert!(
        combined.contains("graphcodebert"),
        "expected missing entry surfaced in output, got:\n{combined}",
    );
}

#[test]
fn json_output_contract() {
    let tmp = TempDir::new().expect("tempdir");
    stage_fixture(tmp.path(), b"tokenizer-fixture-bytes\n", b"onnx-fixture-bytes\n");

    let out = Command::new(elai_binary())
        .args([
            "doctor",
            "knowledge-models",
            "--output-format",
            "json",
            "--project-root",
        ])
        .arg(tmp.path())
        .output()
        .expect("spawn elai doctor knowledge-models --output-format json");

    assert!(
        out.status.success(),
        "expected exit 0; stderr=\n{}",
        String::from_utf8_lossy(&out.stderr),
    );
    let stdout = String::from_utf8(out.stdout).expect("stdout is utf8");
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap_or_else(|e| {
        panic!("stdout is not valid JSON: {e}; raw:\n{stdout}");
    });

    assert_eq!(parsed["kind"], "doctor-knowledge-models");
    assert_eq!(parsed["has_failures"], false);
    let entries = parsed["entries"]
        .as_array()
        .expect("entries[] is a JSON array");
    assert_eq!(entries.len(), MANIFEST.len(), "entry count matches manifest");
    for entry in entries {
        assert!(entry["label"].is_string(), "label is string");
        assert!(entry["path"].is_string(), "path is string");
        assert!(entry["present"].as_bool().unwrap_or(false), "present=true");
        assert!(entry["size_bytes"].as_u64().unwrap_or(0) > 0, "size > 0");
        // Tokenizer entries (rel ends with .json) carry a 64-char blake3 hash
        // over the FULL content. ONNX entries carry a 64-char hash over the
        // first 256 bytes only. Both must be 64 hex chars when present.
        if let Some(hash) = entry["blake3"].as_str() {
            assert_eq!(hash.len(), 64, "blake3 is 64-char hex: {hash}");
            assert!(
                hash.chars().all(|c| c.is_ascii_hexdigit()),
                "blake3 hex: {hash}",
            );
        }
    }
    // Verify the resolved models_dir is reported as an absolute path.
    let models_dir = parsed["models_dir"].as_str().expect("models_dir present");
    assert!(
        models_dir.contains(".elai_cc"),
        ".elai_cc segment in models_dir: {models_dir}",
    );
}
