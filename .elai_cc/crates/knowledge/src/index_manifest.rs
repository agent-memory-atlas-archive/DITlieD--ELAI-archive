//! C-pack C2 — per-project index manifest.
//!
//! `IndexManifest` is the canonical record of every file the knowledge
//! pipeline has considered for indexing, the blake3 content hash at the time
//! of the last attempt, and the resulting status (Pending / Indexed /
//! Failed). It is persisted as JSON at `index_manifest_path(project_root)`
//! (see `elai_core::paths`).
//!
//! Design notes:
//!   * `files` is a `BTreeMap` (not `HashMap`/`FxHashMap`) so JSON
//!     round-trips are byte-stable — the path_unification contract test and
//!     downstream change-detection logic rely on deterministic ordering.
//!   * Content hashes are blake3 hex strings (workspace canonical — see
//!     `Cargo.toml` and CLAUDE.md "Anti-Stale-Training" rule). Never sha2,
//!     never blake2.
//!   * `save` writes atomically via a sibling `.tmp` file + rename so a
//!     mid-write crash cannot leave a half-formed manifest on disk.
//!   * `load` returns `Default::default()` when the manifest file is
//!     missing — first-boot semantics. Any other I/O error propagates.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Current manifest schema version. Bump whenever a non-backwards-compatible
/// change lands in `IndexFileEntry` or `IndexManifest`.
pub const INDEX_MANIFEST_VERSION: u32 = 1;

/// Status of a single tracked file in the index manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum IndexFileStatus {
    /// File is known to the manifest but has not yet been indexed.
    Pending,
    /// File was indexed successfully at the recorded `mtime_secs` /
    /// `content_blake3`.
    Indexed,
    /// Indexing attempt failed; `reason` carries a short human-readable
    /// description (e.g. "parse error", "binary file").
    Failed { reason: String },
}

/// One row in the index manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexFileEntry {
    /// Project-relative path of the tracked file.
    pub path: PathBuf,
    /// Last-known modification time, seconds since UNIX epoch. `i64` to
    /// match `std::fs::Metadata::modified` -> `SystemTime` arithmetic on
    /// every supported platform without overflow on negative epochs.
    pub mtime_secs: i64,
    /// blake3 hex digest of the file's contents at the time of the last
    /// indexing attempt. blake3 is workspace canonical — do NOT switch to
    /// sha2 / blake2.
    pub content_blake3: String,
    /// Indexing outcome.
    pub status: IndexFileStatus,
}

/// Per-project index manifest. Persisted as JSON at
/// `index_manifest_path(project_root)`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexManifest {
    /// Schema version (`INDEX_MANIFEST_VERSION`).
    pub version: u32,
    /// Tracked files, keyed by project-relative path. `BTreeMap` is
    /// deliberate — JSON output order is determined by the in-memory
    /// iteration order, and downstream consumers (and the contract test)
    /// rely on stable, sorted ordering.
    pub files: BTreeMap<PathBuf, IndexFileEntry>,
}

impl Default for IndexManifest {
    fn default() -> Self {
        Self {
            version: INDEX_MANIFEST_VERSION,
            files: BTreeMap::new(),
        }
    }
}

/// Errors emitted by `IndexManifest::load` / `save`.
#[derive(Debug, thiserror::Error)]
pub enum IndexManifestError {
    #[error("index manifest I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("index manifest JSON error: {0}")]
    Serde(#[from] serde_json::Error),
}

impl IndexManifest {
    /// Load a manifest from disk. Returns `Default::default()` (an empty
    /// manifest with the current version) if the file does not exist; all
    /// other I/O errors propagate.
    pub fn load(manifest_path: &Path) -> Result<Self, IndexManifestError> {
        match std::fs::read(manifest_path) {
            Ok(bytes) => {
                let manifest: IndexManifest = serde_json::from_slice(&bytes)?;
                Ok(manifest)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(IndexManifestError::Io(e)),
        }
    }

    /// Persist the manifest to disk atomically (write `<path>.tmp` then
    /// rename over `<path>`). Creates the parent directory if missing.
    pub fn save(&self, manifest_path: &Path) -> Result<(), IndexManifestError> {
        if let Some(parent) = manifest_path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }

        // `serde_json::to_vec_pretty` keeps diffs human-readable; BTreeMap
        // ordering guarantees stable output across runs.
        let bytes = serde_json::to_vec_pretty(self)?;

        let tmp_path = match manifest_path.file_name() {
            Some(name) => {
                let mut tmp_name = name.to_os_string();
                tmp_name.push(".tmp");
                manifest_path.with_file_name(tmp_name)
            }
            None => {
                return Err(IndexManifestError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "index manifest path has no file name component",
                )));
            }
        };

        std::fs::write(&tmp_path, &bytes)?;
        std::fs::rename(&tmp_path, manifest_path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_manifest_is_empty_with_current_version() {
        let m = IndexManifest::default();
        assert_eq!(m.version, INDEX_MANIFEST_VERSION);
        assert!(m.files.is_empty());
    }

    #[test]
    fn load_missing_file_returns_default() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let missing = tmp.path().join("nope.json");
        let m = IndexManifest::load(&missing).expect("missing -> default");
        assert!(m.files.is_empty());
    }

    #[test]
    fn save_then_load_preserves_entries_and_order() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("indexes").join("manifest.json");

        let mut m = IndexManifest::default();
        for (p, h) in [
            ("a/b.rs", "1111111111111111111111111111111111111111111111111111111111111111"),
            ("a/c.py", "2222222222222222222222222222222222222222222222222222222222222222"),
            ("z/x.svelte", "3333333333333333333333333333333333333333333333333333333333333333"),
        ] {
            m.files.insert(
                PathBuf::from(p),
                IndexFileEntry {
                    path: PathBuf::from(p),
                    mtime_secs: 1_700_000_000,
                    content_blake3: h.to_string(),
                    status: IndexFileStatus::Indexed,
                },
            );
        }

        m.save(&path).expect("save");
        let loaded = IndexManifest::load(&path).expect("load");
        assert_eq!(loaded, m);
    }
}
