//! I-34: Grammar-pin manifest with blake3 load-time verification.
//!
//! Mirrors the E-24 MCP-schema pin pattern from `crates/plugins/src/mcp_schema_pin.rs`.
//!
//! Every EBNF grammar file used by the constrained-decoding pipeline is
//! blake3-pinned in a manifest at `.elai/grammar_pins/manifest.toml`. Third-party
//! grammars are NEVER loaded; the pin manifest is the sole authority (CDA landmine #8).
//! The manifest is R8-frozen: the running system may not modify its own grammar manifest.
//!
//! # Layout
//!
//! ```text
//! .elai/grammar_pins/
//!   manifest.toml   -- list of GrammarPinRecord entries
//! ```
//!
//! # Pin record shape
//!
//! Each record carries:
//! * `file_path`    -- path to the EBNF file (absolute or relative to project root).
//! * `blake3_hex`   -- blake3 hex digest of the file bytes at pin time.
//! * `pinned_at_ms` -- UNIX-millisecond timestamp of the pin action.
//!
//! On `load_and_verify`, every listed file is re-hashed and compared against the
//! stored digest. A mismatch returns `GrammarPinError::PinViolation` (the caller
//! emits a `GrammarGateEvent(PinViolation)` SSB row).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

/// Error type for grammar-pin operations.
#[derive(Debug, Error)]
pub enum GrammarPinError {
    /// Manifest file could not be read.
    #[error("grammar pin manifest I/O error at {path}: {source_msg}")]
    Io { path: String, source_msg: String },

    /// Manifest TOML could not be parsed.
    #[error("grammar pin manifest parse error: {0}")]
    TomlDe(#[from] toml::de::Error),

    /// A grammar file's current blake3 digest does not match the pinned value.
    ///
    /// `path` is the file that was tampered with; `expected` is the pinned hex;
    /// `actual` is the freshly-computed hex.
    #[error("grammar pin violation for {path}: expected {expected}, got {actual}")]
    PinViolation {
        path: String,
        expected: String,
        actual: String,
    },

    /// A grammar file listed in the manifest does not exist on disk.
    #[error("grammar file not found: {path}")]
    FileNotFound { path: String },
}

// ---------------------------------------------------------------------------
// Pin record + manifest
// ---------------------------------------------------------------------------

/// A single pinned grammar file entry.
///
/// Mirrors `PinRecord` from `crates/plugins/src/mcp_schema_pin.rs` but scoped
/// to EBNF grammar files instead of MCP server schemas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GrammarPinRecord {
    /// Path to the EBNF file. May be absolute or relative to the project root.
    pub file_path: PathBuf,
    /// blake3 hex digest of the file bytes at pin time. Never blake2 or sha2.
    pub blake3_hex: String,
    /// UNIX-millisecond timestamp when this pin was recorded.
    pub pinned_at_ms: i64,
}

/// The full grammar-pin manifest. Loaded from `.elai/grammar_pins/manifest.toml`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GrammarPinManifest {
    /// All pinned grammar file records. An empty list means no grammars are
    /// pinned (valid — the orchestrator will skip the pin-check pass).
    pub entries: Vec<GrammarPinRecord>,
}

impl GrammarPinManifest {
    /// Returns `true` when the manifest contains no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the number of pinned grammar entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Load the grammar-pin manifest from `manifest_path` and verify every
/// listed grammar file against its pinned blake3 digest.
///
/// Returns `Ok(GrammarPinManifest)` when all digests match. Returns
/// `Err(GrammarPinError::PinViolation)` on the FIRST mismatch found (fail-fast
/// per CDA landmine #8 — the caller emits a `GrammarGateEvent(PinViolation)` SSB row).
///
/// This is the R8-frozen gate entry point. The function itself DOES NOT mutate
/// the manifest or emit SSB rows — it is a pure read+verify operation.
pub fn load_and_verify(manifest_path: &Path) -> Result<GrammarPinManifest, GrammarPinError> {
    let raw = std::fs::read_to_string(manifest_path).map_err(|e| GrammarPinError::Io {
        path: manifest_path.to_string_lossy().into_owned(),
        source_msg: e.to_string(),
    })?;
    let manifest: GrammarPinManifest = toml::from_str(&raw)?;
    for record in &manifest.entries {
        verify_grammar_file(record)?;
    }
    Ok(manifest)
}

/// Verify a single grammar file against its pinned blake3 digest.
///
/// Returns `Ok(())` on match. Returns `Err(GrammarPinError::PinViolation)` when
/// the file exists but the digest mismatches. Returns `Err(GrammarPinError::FileNotFound)`
/// when the file does not exist on disk.
pub fn verify_grammar_file(record: &GrammarPinRecord) -> Result<(), GrammarPinError> {
    let path = &record.file_path;
    if !path.exists() {
        return Err(GrammarPinError::FileNotFound {
            path: path.to_string_lossy().into_owned(),
        });
    }
    let bytes = std::fs::read(path).map_err(|e| GrammarPinError::Io {
        path: path.to_string_lossy().into_owned(),
        source_msg: e.to_string(),
    })?;
    let actual_hex = blake3::hash(&bytes).to_hex().to_string();
    if actual_hex != record.blake3_hex {
        return Err(GrammarPinError::PinViolation {
            path: path.to_string_lossy().into_owned(),
            expected: record.blake3_hex.clone(),
            actual: actual_hex,
        });
    }
    Ok(())
}

/// Returns the canonical path for the grammar-pin manifest given a project root.
///
/// Callers use this as the default manifest path. The path mirrors the
/// `.elai/mcp/pins/` convention from E-24 but uses a separate `.elai/grammar_pins/`
/// namespace so grammar pins never collide with MCP schema pins.
pub fn default_manifest_path(project_root: &Path) -> PathBuf {
    project_root.join(".elai").join("grammar_pins").join("manifest.toml")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn write_tmp_file(dir: &Path, name: &str, content: &[u8]) -> PathBuf {
        let path = dir.join(name);
        let mut f = std::fs::File::create(&path).expect("create tmp file");
        f.write_all(content).expect("write tmp file");
        path
    }

    #[test]
    fn verify_clean_file_passes() {
        let dir = TempDir::new().expect("tempdir");
        let content = b"rule ::= \"hello\" ;";
        let path = write_tmp_file(dir.path(), "test.ebnf", content);
        let hash = blake3::hash(content).to_hex().to_string();
        let record = GrammarPinRecord {
            file_path: path.clone(),
            blake3_hex: hash,
            pinned_at_ms: 1_700_000_000_000,
        };
        assert!(verify_grammar_file(&record).is_ok());
    }

    #[test]
    fn verify_tampered_file_returns_pin_violation() {
        let dir = TempDir::new().expect("tempdir");
        let original = b"rule ::= \"hello\" ;";
        let tampered = b"rule ::= \"evil\" ;";
        let path = write_tmp_file(dir.path(), "tamper.ebnf", tampered);
        let honest_hash = blake3::hash(original).to_hex().to_string();
        let record = GrammarPinRecord {
            file_path: path,
            blake3_hex: honest_hash,
            pinned_at_ms: 1_700_000_000_000,
        };
        let err = verify_grammar_file(&record).unwrap_err();
        assert!(matches!(err, GrammarPinError::PinViolation { .. }));
    }

    #[test]
    fn verify_missing_file_returns_file_not_found() {
        let dir = TempDir::new().expect("tempdir");
        let record = GrammarPinRecord {
            file_path: dir.path().join("nonexistent.ebnf"),
            blake3_hex: "deadbeef".into(),
            pinned_at_ms: 0,
        };
        let err = verify_grammar_file(&record).unwrap_err();
        assert!(matches!(err, GrammarPinError::FileNotFound { .. }));
    }

    #[test]
    fn load_and_verify_manifest_roundtrip() {
        let dir = TempDir::new().expect("tempdir");
        let content = b"start ::= \"test\" ;";
        let grammar_path = write_tmp_file(dir.path(), "file_plan.ebnf", content);
        let hash = blake3::hash(content).to_hex().to_string();

        let manifest = GrammarPinManifest {
            entries: vec![GrammarPinRecord {
                file_path: grammar_path,
                blake3_hex: hash,
                pinned_at_ms: 1_700_000_000_000,
            }],
        };

        // Write manifest to TOML.
        let manifest_path = dir.path().join("manifest.toml");
        let toml_str = toml::to_string(&manifest).expect("serialize manifest");
        std::fs::write(&manifest_path, &toml_str).expect("write manifest");

        // Load and verify -- should pass.
        let loaded = load_and_verify(&manifest_path).expect("load_and_verify");
        assert_eq!(loaded.entries.len(), 1);
    }

    #[test]
    fn load_and_verify_manifest_detects_tamper() {
        let dir = TempDir::new().expect("tempdir");
        let original = b"start ::= \"original\" ;";
        let grammar_path = write_tmp_file(dir.path(), "file_plan.ebnf", original);
        // Pin the hash of 'original' content.
        let hash = blake3::hash(original).to_hex().to_string();

        let manifest = GrammarPinManifest {
            entries: vec![GrammarPinRecord {
                file_path: grammar_path.clone(),
                blake3_hex: hash,
                pinned_at_ms: 1_700_000_000_000,
            }],
        };
        let manifest_path = dir.path().join("manifest.toml");
        let toml_str = toml::to_string(&manifest).expect("serialize manifest");
        std::fs::write(&manifest_path, &toml_str).expect("write manifest");

        // Now overwrite the grammar file with tampered content.
        std::fs::write(&grammar_path, b"start ::= \"evil\" ;").expect("tamper");

        // load_and_verify must detect the mismatch.
        let err = load_and_verify(&manifest_path).unwrap_err();
        assert!(matches!(err, GrammarPinError::PinViolation { .. }));
    }

    #[test]
    fn default_manifest_path_shape() {
        let root = PathBuf::from("/tmp/project");
        let p = default_manifest_path(&root);
        assert!(p.ends_with("grammar_pins/manifest.toml"));
    }
}
