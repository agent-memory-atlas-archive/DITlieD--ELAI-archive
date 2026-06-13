//! I-34 — CRANE hybrid grammar-pin gate contract tests.
//!
//! Coverage (4 arms; no #[ignore], no stubs, no placeholder asserts):
//!
//! (a) Gate eligibility: `crane_adaptive_eligible` returns true ONLY for
//!     `DecodingMode::Grammar { crane_adaptive: true }`, false for everything
//!     else.
//!
//! (b) Pin verification: `load_and_verify_grammar_manifest` detects a blake3
//!     digest mismatch and returns `GrammarPinError::PinViolation`.
//!
//! (c) Post-parse rejection: `re_validate_output` returns `Err(ValidationError)`
//!     when the output violates the declared schema.
//!
//! (d) Schema lock (golden 2/2): SCHEMA_VERSION_PUBLIC == 234 AND
//!     action_type_lock.rs reports TOTAL_VARIANTS == 543, confirming the
//!     atomically-locked SSB expansion.

use std::io::Write as IoWrite;
use tempfile::TempDir;

use elai_orchestrator::ssb::SCHEMA_VERSION_PUBLIC;
use elai_providers::{
    crane_adaptive_eligible, load_and_verify_grammar_manifest, re_validate_output, DecodingMode,
    GrammarPinError, GrammarPinManifest, GrammarPinRecord,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Write a temp `.ebnf` file and return (path, blake3_hex_of_content).
fn write_ebnf_fixture(dir: &std::path::Path, name: &str, content: &[u8]) -> (std::path::PathBuf, String) {
    let path = dir.join(name);
    let mut f = std::fs::File::create(&path).expect("create ebnf fixture");
    f.write_all(content).expect("write ebnf fixture");
    let hex = blake3::hash(content).to_hex().to_string();
    (path, hex)
}

/// Write a `GrammarPinManifest` as TOML to `manifest_path`.
fn write_manifest(dir: &std::path::Path, manifest: &GrammarPinManifest) -> std::path::PathBuf {
    let toml_str = toml::to_string(manifest).expect("serialize manifest");
    let path = dir.join("manifest.toml");
    std::fs::write(&path, &toml_str).expect("write manifest");
    path
}

// ---------------------------------------------------------------------------
// (a) Gate eligibility
// ---------------------------------------------------------------------------

/// `crane_adaptive_eligible` returns true for a crane-adaptive Grammar mode.
#[test]
fn gate_eligibility_crane_adaptive_is_true() {
    let mode = DecodingMode::Grammar {
        ebnf_path: std::path::PathBuf::from("fixtures/test.ebnf"),
        crane_adaptive: true,
    };
    assert!(
        crane_adaptive_eligible(&mode),
        "crane_adaptive_eligible must return true for Grammar {{ crane_adaptive: true }}"
    );
}

/// `crane_adaptive_eligible` returns false for a non-crane Grammar mode.
#[test]
fn gate_eligibility_non_crane_is_false() {
    let mode = DecodingMode::Grammar {
        ebnf_path: std::path::PathBuf::from("fixtures/test.ebnf"),
        crane_adaptive: false,
    };
    assert!(
        !crane_adaptive_eligible(&mode),
        "crane_adaptive_eligible must return false for Grammar {{ crane_adaptive: false }}"
    );
}

/// `crane_adaptive_eligible` returns false for free-text mode (no constraint).
#[test]
fn gate_eligibility_freetext_is_false() {
    let mode = DecodingMode::FreeText;
    assert!(
        !crane_adaptive_eligible(&mode),
        "crane_adaptive_eligible must return false for FreeText mode"
    );
}

// ---------------------------------------------------------------------------
// (b) Pin verification — mismatch detection
// ---------------------------------------------------------------------------

/// A tampered grammar file (bytes changed after pinning) triggers PinViolation.
#[test]
fn pin_verification_tamper_returns_violation() {
    let dir = TempDir::new().expect("tempdir");

    let original = b"start ::= \"hello\" ;";
    let (path, original_hex) = write_ebnf_fixture(dir.path(), "grammar.ebnf", original);

    let manifest = GrammarPinManifest {
        entries: vec![GrammarPinRecord {
            file_path: path.clone(),
            blake3_hex: original_hex,
            pinned_at_ms: 1_700_000_000_000,
        }],
    };
    let manifest_path = write_manifest(dir.path(), &manifest);

    // Overwrite the grammar with tampered content AFTER pinning.
    std::fs::write(&path, b"start ::= \"evil\" ;").expect("tamper");

    let err = load_and_verify_grammar_manifest(&manifest_path)
        .expect_err("expected PinViolation, got Ok");
    assert!(
        matches!(err, GrammarPinError::PinViolation { .. }),
        "expected GrammarPinError::PinViolation, got: {err:?}"
    );
}

/// A clean (untampered) grammar file passes pin verification.
#[test]
fn pin_verification_clean_passes() {
    let dir = TempDir::new().expect("tempdir");

    let content = b"start ::= \"hello\" ;";
    let (path, hex) = write_ebnf_fixture(dir.path(), "clean.ebnf", content);

    let manifest = GrammarPinManifest {
        entries: vec![GrammarPinRecord {
            file_path: path,
            blake3_hex: hex,
            pinned_at_ms: 1_700_000_000_000,
        }],
    };
    let manifest_path = write_manifest(dir.path(), &manifest);

    let loaded = load_and_verify_grammar_manifest(&manifest_path)
        .expect("clean manifest should pass verification");
    assert_eq!(loaded.entries.len(), 1, "expected 1 entry in loaded manifest");
}

/// A missing grammar file returns FileNotFound.
#[test]
fn pin_verification_missing_file_returns_not_found() {
    let dir = TempDir::new().expect("tempdir");

    let manifest = GrammarPinManifest {
        entries: vec![GrammarPinRecord {
            file_path: dir.path().join("nonexistent.ebnf"),
            blake3_hex: "deadbeef".into(),
            pinned_at_ms: 0,
        }],
    };
    let manifest_path = write_manifest(dir.path(), &manifest);

    let err = load_and_verify_grammar_manifest(&manifest_path)
        .expect_err("expected FileNotFound, got Ok");
    assert!(
        matches!(err, GrammarPinError::FileNotFound { .. }),
        "expected GrammarPinError::FileNotFound, got: {err:?}"
    );
}

// ---------------------------------------------------------------------------
// (c) Post-parse rejection arm
// ---------------------------------------------------------------------------

/// `re_validate_output` returns Err on a silently non-compliant output.
#[test]
fn post_parse_rejects_non_compliant_output() {
    // Schema declares 'name' (string) and 'score' (number) as required.
    let schema_json = r#"{"type":"object","required":["name","score"],"properties":{"name":{"type":"string"},"score":{"type":"number"}}}"#;
    // Missing required 'score' field — silently non-compliant.
    let non_compliant = r#"{"name":"test"}"#;
    let result = re_validate_output(non_compliant, schema_json);
    assert!(
        result.is_err(),
        "re_validate_output must return Err on non-compliant output; got Ok"
    );
    let err = result.unwrap_err();
    assert!(
        !err.message.is_empty(),
        "ValidationError.message must be non-empty"
    );
}

/// `re_validate_output` returns Ok on a compliant output.
#[test]
fn post_parse_accepts_compliant_output() {
    let schema_json = r#"{"type":"object","required":["name","score"],"properties":{"name":{"type":"string"},"score":{"type":"number"}}}"#;
    let compliant = r#"{"name":"test","score":0.9}"#;
    let result = re_validate_output(compliant, schema_json);
    assert!(
        result.is_ok(),
        "re_validate_output must return Ok on compliant output; got Err: {result:?}"
    );
}

/// `re_validate_output` rejects non-JSON output (not a valid JSON object).
#[test]
fn post_parse_rejects_non_json_output() {
    let schema_json = r#"{"type":"object","properties":{}}"#;
    let bad = "this is not json at all";
    let result = re_validate_output(bad, schema_json);
    assert!(
        result.is_err(),
        "re_validate_output must return Err on non-JSON output"
    );
}

// ---------------------------------------------------------------------------
// (d) Schema lock (golden 2/2)
// ---------------------------------------------------------------------------

/// SCHEMA_VERSION_PUBLIC must be >= 234 (I-34 floor: bump from 233 -> 234).
/// Subsequent I-plan sessions (e.g. I-38 at 235) increment further; the
/// floor asserts I-34 shipped and was not regressed.
#[test]
fn schema_version_is_234() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 234,
        "SCHEMA_VERSION_PUBLIC must be >= 234 (I-34 floor: GrammarGateEvent add); \
         got {} — a value below 234 means the I-34 SSB bump was regressed",
        SCHEMA_VERSION_PUBLIC,
    );
}

/// action_type_lock.rs declares TOTAL_VARIANTS >= 543 (floor from I-34; subsequent I-plans bump).
#[test]
fn total_variants_is_543() {
    let lock_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("contracts crate has a parent")
        .join("orchestrator")
        .join("tests")
        .join("action_type_lock.rs");
    let body = std::fs::read_to_string(&lock_path)
        .unwrap_or_else(|e| panic!("cannot read action_type_lock.rs at {}: {e}", lock_path.display()));
    // Use a floor check so this contract stays valid after subsequent I-plan schema bumps.
    let total_variants_value: usize = body
        .lines()
        .find_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with("const TOTAL_VARIANTS: usize = ") {
                trimmed
                    .trim_start_matches("const TOTAL_VARIANTS: usize = ")
                    .trim_end_matches(';')
                    .parse()
                    .ok()
            } else {
                None
            }
        })
        .unwrap_or_else(|| panic!(
            "could not parse TOTAL_VARIANTS from action_type_lock.rs; file: {}",
            lock_path.display()
        ));
    assert!(
        total_variants_value >= 543,
        "TOTAL_VARIANTS must be >= 543 (I-34 floor: GrammarGateEvent added 1 wire tag at 543); \
         got {} in {}",
        total_variants_value,
        lock_path.display()
    );
}
