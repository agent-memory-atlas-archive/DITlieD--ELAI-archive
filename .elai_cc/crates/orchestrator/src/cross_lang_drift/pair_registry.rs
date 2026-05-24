//! E-45 — TOML loader for the cross-language pair registry.
//!
//! Reads `.elai_cc/cross-lang-pairs.toml` and produces a typed
//! `Vec<DriftPair>`. The registry is the single source of truth for
//! which Rust files have which mirror; the detector consumes it
//! read-only.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

use super::types::{DriftPair, PairKind};

/// Default location of the registry, relative to repo root.
pub const REGISTRY_RELATIVE_PATH: &str = ".elai_cc/cross-lang-pairs.toml";

#[derive(Debug, Error)]
pub enum PairRegistryError {
    #[error("registry file not found at {0}")]
    NotFound(PathBuf),
    #[error("registry I/O at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("registry TOML parse error at {path}: {source}")]
    TomlParse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("unknown pair kind '{name}' at row {row}; valid kinds: {valid}")]
    UnknownKind {
        name: String,
        row: usize,
        valid: String,
    },
    #[error("registry contains zero pairs at {0}; refusing to run with empty registry")]
    Empty(PathBuf),
}

/// Loaded + validated pair registry.
#[derive(Debug, Clone)]
pub struct PairRegistry {
    pairs: Vec<DriftPair>,
}

impl PairRegistry {
    /// Load the registry from the default path under `repo_root`.
    /// `repo_root` must be the workflow root (containing `.elai_cc/`),
    /// not `.elai_cc/` itself.
    pub fn load(repo_root: &Path) -> Result<Self, PairRegistryError> {
        let path = repo_root.join(REGISTRY_RELATIVE_PATH);
        Self::load_from(&path)
    }

    /// Load from an explicit path. Used by tests + callers that want to
    /// load from a custom location.
    pub fn load_from(path: &Path) -> Result<Self, PairRegistryError> {
        if !path.exists() {
            return Err(PairRegistryError::NotFound(path.to_path_buf()));
        }
        let text = fs::read_to_string(path).map_err(|e| PairRegistryError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        let raw: RawRegistry =
            toml::from_str(&text).map_err(|e| PairRegistryError::TomlParse {
                path: path.to_path_buf(),
                source: e,
            })?;

        let mut pairs = Vec::with_capacity(raw.pairs.len());
        for (idx, row) in raw.pairs.into_iter().enumerate() {
            let kind = PairKind::from_wire_tag(&row.name).ok_or_else(|| {
                PairRegistryError::UnknownKind {
                    name: row.name.clone(),
                    row: idx,
                    valid: PAIR_KIND_NAMES.join(", "),
                }
            })?;
            pairs.push(DriftPair {
                kind,
                rust_path: PathBuf::from(row.rust_path),
                mirror_path: PathBuf::from(row.mirror_path),
                note: row.note.unwrap_or_default(),
            });
        }

        if pairs.is_empty() {
            return Err(PairRegistryError::Empty(path.to_path_buf()));
        }
        Ok(Self { pairs })
    }

    /// Read-only iterator over the registry pairs.
    pub fn pairs(&self) -> &[DriftPair] {
        &self.pairs
    }

    /// Number of registered pairs. Used by the detector to emit the
    /// `pair_count` field on `CrossLangPairScannedPayload`.
    pub fn len(&self) -> usize {
        self.pairs.len()
    }

    /// Whether the registry has zero pairs. Should never return `true`
    /// because `load_from` rejects empty registries, but the accessor
    /// is here for clippy + explicit-check use.
    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }
}

const PAIR_KIND_NAMES: &[&str] = &[
    "sidecar_orchestrator_schemas",
    "tauri_commands_ts_stub",
    "tauri_types_lib",
    "sidecar_anthropic_provider",
    "sidecar_runtime_client",
    "ipc_types_dashboard",
];

#[derive(Debug, Deserialize)]
struct RawRegistry {
    #[serde(default)]
    pairs: Vec<RawPair>,
}

#[derive(Debug, Deserialize)]
struct RawPair {
    name: String,
    rust_path: String,
    mirror_path: String,
    #[serde(default)]
    note: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_toml(dir: &TempDir, body: &str) -> PathBuf {
        let path = dir.path().join("cross-lang-pairs.toml");
        fs::write(&path, body).expect("write registry");
        path
    }

    #[test]
    fn loads_well_formed_registry() {
        let dir = TempDir::new().expect("tempdir");
        let path = write_toml(
            &dir,
            r#"
[[pairs]]
name = "sidecar_orchestrator_schemas"
rust_path = "crates/orchestrator/src/ssb.rs"
mirror_path = "python/elai_inference/schemas/ssb_mirror.py"
note = "core SSB types"

[[pairs]]
name = "tauri_commands_ts_stub"
rust_path = "crates/dashboard-v2/src-tauri/src/commands.rs"
mirror_path = "crates/dashboard-v2/src/lib/api/stub.ts"
"#,
        );
        let reg = PairRegistry::load_from(&path).expect("load");
        assert_eq!(reg.len(), 2);
        assert_eq!(reg.pairs()[0].kind, PairKind::SidecarOrchestratorSchemas);
        assert_eq!(reg.pairs()[0].note, "core SSB types");
        assert_eq!(reg.pairs()[1].note, "");
    }

    #[test]
    fn rejects_unknown_pair_kind() {
        let dir = TempDir::new().expect("tempdir");
        let path = write_toml(
            &dir,
            r#"
[[pairs]]
name = "made_up_name"
rust_path = "a"
mirror_path = "b"
"#,
        );
        let err = PairRegistry::load_from(&path).expect_err("must fail");
        match err {
            PairRegistryError::UnknownKind { name, .. } => assert_eq!(name, "made_up_name"),
            other => panic!("expected UnknownKind, got {other:?}"),
        }
    }

    #[test]
    fn rejects_empty_registry() {
        let dir = TempDir::new().expect("tempdir");
        let path = write_toml(&dir, "");
        let err = PairRegistry::load_from(&path).expect_err("must fail");
        match err {
            PairRegistryError::Empty(_) => {}
            other => panic!("expected Empty, got {other:?}"),
        }
    }

    #[test]
    fn rejects_missing_file() {
        let dir = TempDir::new().expect("tempdir");
        let missing = dir.path().join("nonexistent.toml");
        let err = PairRegistry::load_from(&missing).expect_err("must fail");
        match err {
            PairRegistryError::NotFound(_) => {}
            other => panic!("expected NotFound, got {other:?}"),
        }
    }
}
