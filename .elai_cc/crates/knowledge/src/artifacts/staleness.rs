//! Artifact staleness detection.
//!
//! Compares current per-file blake3 hashes against the previously stored
//! [`ArtifactIndexState`]. Works purely on-disk; no SQLite access here.
//!
//! For directory artifacts, the file set is enumerated using the `ignore`
//! crate (same rules as the main code indexer) so hidden files and gitignored
//! entries are excluded consistently.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use tracing::debug;

use super::manifest::ArtifactError;
use super::types::{ArtifactIndexState, ArtifactSpec};

// ---------------------------------------------------------------------------
// StaleResult
// ---------------------------------------------------------------------------

/// Output of [`detect_staleness`].
#[derive(Debug, Default)]
pub struct StaleResult {
    /// Files that exist in both the current scan and prev_state with matching
    /// hashes.
    pub unchanged: Vec<PathBuf>,
    /// Files that exist in both but with different hashes.
    pub changed: Vec<PathBuf>,
    /// Files present now but absent from prev_state (new since last index).
    pub added: Vec<PathBuf>,
    /// Files in prev_state that no longer exist on disk.
    pub removed: Vec<PathBuf>,
}

impl StaleResult {
    /// True when nothing changed (no re-index needed).
    pub fn is_clean(&self) -> bool {
        self.changed.is_empty() && self.added.is_empty() && self.removed.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Compute the diff between on-disk artifact content and the previously
/// recorded [`ArtifactIndexState`].
///
/// `prev_state = None` means the artifact has never been indexed — every
/// file will appear in `StaleResult::added`.
pub fn detect_staleness(
    spec: &ArtifactSpec,
    prev_state: Option<&ArtifactIndexState>,
) -> Result<StaleResult, ArtifactError> {
    let current_hashes = hash_artifact_files(spec)?;
    let prev_hashes: BTreeMap<&str, &str> = prev_state
        .map(|s| {
            s.file_hashes
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect()
        })
        .unwrap_or_default();

    let mut result = StaleResult::default();

    for (rel, hash) in &current_hashes {
        let rel_str = rel.to_string_lossy();
        match prev_hashes.get(rel_str.as_ref()) {
            None => result.added.push(rel.clone()),
            Some(prev_hash) if *prev_hash != hash.as_str() => result.changed.push(rel.clone()),
            Some(_) => result.unchanged.push(rel.clone()),
        }
    }

    for prev_rel in prev_hashes.keys() {
        let as_path = PathBuf::from(prev_rel);
        if !current_hashes.contains_key(&as_path) {
            result.removed.push(as_path);
        }
    }

    debug!(
        artifact = spec.name,
        unchanged = result.unchanged.len(),
        changed = result.changed.len(),
        added = result.added.len(),
        removed = result.removed.len(),
        "staleness scan complete"
    );

    Ok(result)
}

/// Produce `{ relative_path -> blake3_hex }` for all files covered by `spec`.
pub(crate) fn hash_artifact_files(
    spec: &ArtifactSpec,
) -> Result<BTreeMap<PathBuf, String>, ArtifactError> {
    let mut out = BTreeMap::new();

    if spec.is_dir {
        enumerate_dir_files(&spec.path, &spec.path, &mut out)?;
    } else {
        let hash = hash_file(&spec.path)?;
        let rel = spec
            .path
            .file_name()
            .map(PathBuf::from)
            .unwrap_or_else(|| spec.path.clone());
        out.insert(rel, hash);
    }

    Ok(out)
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Recursively enumerate files in `dir`, skipping hidden files and honouring
/// `.gitignore` / `.ignore` rules via the `ignore` crate.
fn enumerate_dir_files(
    root: &Path,
    dir: &Path,
    out: &mut BTreeMap<PathBuf, String>,
) -> Result<(), ArtifactError> {
    let walker = ignore::WalkBuilder::new(dir)
        .hidden(true)
        .git_ignore(true)
        .git_global(false)
        .git_exclude(false)
        .build();

    for entry in walker {
        let entry = entry.map_err(|e| ArtifactError::Other(e.to_string()))?;
        if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
            let abs = entry.path();
            let rel = abs
                .strip_prefix(root)
                .unwrap_or(abs)
                .to_path_buf();
            let hash = hash_file(abs)?;
            out.insert(rel, hash);
        }
    }

    Ok(())
}

/// Compute the blake3 hex digest of a file's raw bytes.
fn hash_file(path: &Path) -> Result<String, ArtifactError> {
    let bytes = std::fs::read(path)?;
    let hash = blake3::hash(&bytes);
    Ok(hash.to_hex().to_string())
}
