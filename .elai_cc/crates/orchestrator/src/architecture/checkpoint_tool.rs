//! I-52 (Track 20 Wave 3) -- agent-callable checkpoint tool core.
//!
//! A thin, session-scoped wrapper over the SHIPPED H-49 time-travel CAS
//! substrate ([`crate::architecture::timetravel`]). It exposes the three verbs
//! the agent tool dispatch needs -- save / rewind / diff -- WITHOUT building a
//! second snapshot store (GD-1 / stub_ban): every blob lives in the H-49 CAS,
//! every restore routes through [`timetravel::materialize`] (the only writer,
//! carrying the RV-1 divergence guard).
//!
//! Safety doctrine (the load-bearing bit):
//!   * REWIND ALWAYS restores via [`timetravel::materialize`], which is
//!     NON-DESTRUCTIVE: before overwriting any covered path whose on-disk bytes
//!     diverged from the checkpoint, `materialize` fork-captures the current
//!     bytes into the CAS first (its RV-1 guard). So an ordinary speculative
//!     edit to a covered file is REWOUND, not refused -- the divergent state is
//!     preserved as a redo-able fork (H-49 undo-then-edit=fork semantics; plan
//!     line 58 "post-rewind bytes blake3-match pre-images").
//!   * REWIND is REFUSED ([`RewindOutcome::RefusedDiverged`], NO file written)
//!     ONLY when a tool-level PRE-CHECK -- run here BEFORE we call `materialize`
//!     -- finds a genuinely UNRECOVERABLE / UNSAFE condition for a covered path:
//!       (a) the checkpoint path ESCAPES the session/project scope (traversal),
//!       (b) a required CAS object is MISSING (the blob to restore is gone), or
//!       (c) the pre-rewind fork-capture of the current divergent bytes FAILS
//!           (we cannot safely preserve the divergence before overwriting it).
//!     In any of (a)/(b)/(c) we do NOT call `materialize`; we return the refusal
//!     naming the offending paths, and no file is overwritten.
//!   * SESSION SCOPE: `session_id` is supplied by the run context (never by the
//!     agent). The CAS root is physically per-session
//!     (`root/{session_id}/...`), so a label saved under one session is
//!     unreachable from a dispatch running under another -- the cross-session
//!     rewind guard is structural, not a check that can be bypassed.
//!
//! Labels: the agent supplies a human label; we persist a `label -> manifest
//! hash` map as one JSON file per session under `root/{session_id}/tags/`.

use std::collections::BTreeMap;
use std::path::Path;

use crate::architecture::timetravel::{
    self, blake3_hex, FileManifest, ManifestEntry, TimeTravelError,
};

/// Subdirectory (under `root/{session_id}/`) holding the label->manifest map.
const TAGS_DIR: &str = "tags";

/// File name (under the tags dir) for the label->manifest-hash map.
const TAGS_FILE: &str = "checkpoint_labels.json";

/// Errors from the checkpoint tool core.
#[derive(Debug, thiserror::Error)]
pub enum CheckpointToolError {
    #[error("time-travel error: {0}")]
    TimeTravel(#[from] TimeTravelError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("checkpoint label not found: {label}")]
    LabelNotFound { label: String },
    #[error("no paths supplied for checkpoint save")]
    EmptyPaths,
}

/// Outcome of a [`save`] call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveOutcome {
    /// The label the checkpoint was saved under.
    pub label: String,
    /// Project-relative paths captured into the manifest.
    pub paths: Vec<String>,
}

/// Outcome of a [`rewind`] call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RewindOutcome {
    /// The tree was clean; files were restored via `timetravel::materialize`.
    Rewound {
        /// Project-relative paths restored.
        paths: Vec<String>,
    },
    /// A tool-level pre-check found an UNRECOVERABLE / UNSAFE condition for one
    /// or more covered paths; the rewind was REFUSED and NO file was
    /// overwritten. Names the offending path set (the safety observable):
    /// a path that escapes the project scope, a missing CAS blob, or a covered
    /// path whose pre-rewind fork-capture failed.
    RefusedDiverged {
        /// Project-relative paths the pre-check refused on (scope escape /
        /// missing CAS object / fork-capture failure) -- the reason the rewind
        /// was refused.
        diverged_paths: Vec<String>,
    },
}

/// Outcome of a [`diff`] call: the project-relative paths whose on-disk content
/// differs from the saved checkpoint (empty == clean).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffOutcome {
    /// The label diffed against.
    pub label: String,
    /// Paths that changed since the checkpoint was saved.
    pub changed_paths: Vec<String>,
}

// ---------------------------------------------------------------------------
// Label map persistence (label -> manifest hash)
// ---------------------------------------------------------------------------

fn tags_file_path(root: &Path, session_id: &str) -> std::path::PathBuf {
    root.join(session_id).join(TAGS_DIR).join(TAGS_FILE)
}

fn load_labels(root: &Path, session_id: &str) -> Result<BTreeMap<String, String>, CheckpointToolError> {
    let p = tags_file_path(root, session_id);
    if !p.exists() {
        return Ok(BTreeMap::new());
    }
    let data = std::fs::read(&p)?;
    Ok(serde_json::from_slice(&data)?)
}

fn store_labels(
    root: &Path,
    session_id: &str,
    labels: &BTreeMap<String, String>,
) -> Result<(), CheckpointToolError> {
    let p = tags_file_path(root, session_id);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_vec_pretty(labels)?;
    // Atomic write: temp + rename (mirrors the H-49 CAS write discipline).
    let tmp = p.with_extension("tmp");
    std::fs::write(&tmp, &json)?;
    std::fs::rename(&tmp, &p)?;
    Ok(())
}

fn resolve_manifest(
    root: &Path,
    session_id: &str,
    label: &str,
) -> Result<FileManifest, CheckpointToolError> {
    let labels = load_labels(root, session_id)?;
    let manifest_hash = labels
        .get(label)
        .ok_or_else(|| CheckpointToolError::LabelNotFound {
            label: label.to_owned(),
        })?;
    Ok(timetravel::manifest_get(root, session_id, manifest_hash)?)
}

/// Compute the set of manifest paths whose on-disk content differs from the
/// recorded hash. A missing file counts as diverged (its content no longer
/// matches the checkpoint). Returns the diverged project-relative paths.
fn diverged_paths(manifest: &FileManifest, project_root: &Path) -> Result<Vec<String>, CheckpointToolError> {
    let mut diverged = Vec::new();
    for entry in &manifest.entries {
        let abs = project_root.join(&entry.path);
        let matches = if abs.exists() {
            let on_disk = std::fs::read(&abs)?;
            blake3_hex(&on_disk) == entry.hash
        } else {
            false
        };
        if !matches {
            diverged.push(entry.path.clone());
        }
    }
    Ok(diverged)
}

// ---------------------------------------------------------------------------
// The three verbs
// ---------------------------------------------------------------------------

/// Save a labelled checkpoint: capture each path's current bytes into the H-49
/// CAS, build a [`FileManifest`], persist it, and record `label -> manifest
/// hash`. Re-using a label overwrites its mapping (a fresh checkpoint).
pub fn save(
    root: &Path,
    session_id: &str,
    label: &str,
    paths: &[String],
    project_root: &Path,
) -> Result<SaveOutcome, CheckpointToolError> {
    if paths.is_empty() {
        return Err(CheckpointToolError::EmptyPaths);
    }
    let mut entries = Vec::with_capacity(paths.len());
    let mut captured = Vec::with_capacity(paths.len());
    for rel in paths {
        let abs = project_root.join(rel);
        let content = std::fs::read(&abs)?;
        // capture_file puts the blob in the CAS (dedup-aware) and hashes it.
        let result = timetravel::capture_file(root, session_id, &abs, &content)?;
        entries.push(ManifestEntry {
            path: rel.clone(),
            hash: result.pre_hash,
        });
        captured.push(rel.clone());
    }
    let manifest = FileManifest { entries };
    let manifest_hash = timetravel::manifest_put(root, session_id, &manifest)?;

    let mut labels = load_labels(root, session_id)?;
    labels.insert(label.to_owned(), manifest_hash);
    store_labels(root, session_id, &labels)?;

    Ok(SaveOutcome {
        label: label.to_owned(),
        paths: captured,
    })
}

/// Tool-level SAFETY pre-check for a rewind, run BEFORE `materialize`.
///
/// Returns the project-relative paths that are UNRECOVERABLE / UNSAFE to
/// restore (the refusal set). A covered path is unsafe when:
///   (a) it ESCAPES `project_root` (path traversal) -- restoring it would write
///       outside the session/project scope;
///   (b) its CAS blob is MISSING -- there is nothing to restore the file from;
///   (c) its on-disk bytes DIVERGED from the checkpoint AND fork-capturing those
///       current bytes into the CAS fails -- we cannot preserve the divergence
///       safely, so we must not overwrite it.
/// An ordinary clean divergence (on-disk != checkpoint, but fork-capture
/// succeeds) is NOT unsafe: `materialize` preserves it as a redo-able fork.
fn unsafe_paths(
    root: &Path,
    session_id: &str,
    manifest: &FileManifest,
    project_root: &Path,
) -> Result<Vec<String>, CheckpointToolError> {
    let mut refused = Vec::new();
    for entry in &manifest.entries {
        let rel = Path::new(&entry.path);

        // (a) scope escape: reuse the canonical sandbox path-traversal check.
        if crate::sandbox_policy::path_escapes_root(rel, project_root) {
            refused.push(entry.path.clone());
            continue;
        }

        // (b) missing CAS blob: the bytes to restore are gone.
        if !timetravel::cas_has(root, session_id, &entry.hash) {
            refused.push(entry.path.clone());
            continue;
        }

        // (c) divergent on-disk bytes whose pre-rewind fork-capture fails.
        let abs = project_root.join(rel);
        if abs.exists() {
            let on_disk = std::fs::read(&abs)?;
            if blake3_hex(&on_disk) != entry.hash {
                // Materialize WILL fork-capture this before overwriting; verify
                // that capture would succeed now so we never overwrite a
                // divergence we cannot preserve. (Size-cap blobs are stored by
                // materialize too; mirror that bound.)
                if on_disk.len() <= timetravel::BLOB_SIZE_CAP_BYTES
                    && timetravel::cas_put(root, session_id, &on_disk).is_err()
                {
                    refused.push(entry.path.clone());
                }
            }
        }
    }
    Ok(refused)
}

/// Rewind the working tree to a saved label.
///
/// ALWAYS restores through [`timetravel::materialize`] (the only writer, which
/// fork-captures any divergent on-disk bytes BEFORE overwriting -- so an
/// ordinary speculative edit is rewound non-destructively). REFUSES (no write)
/// ONLY when [`unsafe_paths`] finds an unrecoverable/unsafe covered path (scope
/// escape / missing CAS blob / fork-capture failure).
pub fn rewind(
    root: &Path,
    session_id: &str,
    label: &str,
    project_root: &Path,
) -> Result<RewindOutcome, CheckpointToolError> {
    let manifest = resolve_manifest(root, session_id, label)?;

    // SAFETY pre-check: refuse ONLY on an unrecoverable/unsafe condition. We
    // never call materialize when the refusal set is non-empty, so no offending
    // file is overwritten.
    let refused = unsafe_paths(root, session_id, &manifest, project_root)?;
    if !refused.is_empty() {
        return Ok(RewindOutcome::RefusedDiverged {
            diverged_paths: refused,
        });
    }

    // Safe: restore through the H-49 writer. `materialize` fork-captures any
    // covered path that diverged on disk (RV-1) before overwriting it, so the
    // speculative state is preserved as a redo-able fork, not destroyed.
    let restored = timetravel::materialize(root, session_id, &manifest, project_root)?;
    let paths = restored
        .iter()
        .map(|r| timetravel::normalise_path(&r.path))
        .collect();
    Ok(RewindOutcome::Rewound { paths })
}

/// Diff the working tree against a saved label (read-only): the project-relative
/// paths whose on-disk content differs from the checkpoint.
pub fn diff(
    root: &Path,
    session_id: &str,
    label: &str,
    project_root: &Path,
) -> Result<DiffOutcome, CheckpointToolError> {
    let manifest = resolve_manifest(root, session_id, label)?;
    let changed = diverged_paths(&manifest, project_root)?;
    Ok(DiffOutcome {
        label: label.to_owned(),
        changed_paths: changed,
    })
}
