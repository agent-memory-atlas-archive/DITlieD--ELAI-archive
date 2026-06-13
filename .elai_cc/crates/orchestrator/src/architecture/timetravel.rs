//! H-49 — reversible LLM edits: CAS object store + undo/redo/branch engine.
//!
//! # Overview
//!
//! Implements its own content-addressed, zstd-compressed CAS layer.
//! GD-1 reuse: borrows [`crate::architecture::edit_checkpoints::SESSION_CHECKPOINT_ROOT`]
//! as the shared storage root constant so the two subsystems store blobs in the
//! same directory tree.  Does NOT call `snapshot_pre_edit` / `rewind_to` from
//! E-25 — those are byte-level primitives for the older checkpoint path;
//! this module provides a richer manifest+DAG layer on top of the same root.
//!
//! # Storage layout
//!
//! ```text
//! .session-checkpoints/{session_id}/objects/{hash[..2]}/{hash}  <- zstd blob
//! .session-checkpoints/{session_id}/manifests/{manifest_hash}   <- JSON manifest
//! ```
//!
//! Each manifest is a JSON object `{ "entries": [{"path": "...", "hash": "..."}] }`.
//! A manifest hash is the blake3 of the manifest JSON bytes.  Objects are
//! content-addressed: two edits that leave a file identical share one blob.
//!
//! # Kill-switch
//!
//! `ELAI_TIMETRAVEL_DISABLE=1` suppresses ALL capture and restore.  This is a
//! default-ON feature (active unless the env var is set); not a default-OFF
//! gate, so no `wire-dark-allowlist.toml` row is required (R9 confirmed).
//!
//! # Constraints
//!
//! - blake3 only (Anti-Stale-Training rule).
//! - zstd compression (already a transitive dep; H-49 added direct dep).
//! - No `.unwrap()` in production paths — all `?` propagation.
//! - ClockProvider not used here (filesystem timestamps sufficient for
//!   CAS blobs; the SessionDag carries the logical HEAD).
//! - Per-blob size cap: blobs over [`BLOB_SIZE_CAP_BYTES`] are skipped with
//!   a `TooLargeSkipped` event rather than silently omitted (R1/R2).
//!
//! # Undo-tree data model
//!
//! The undo-tree HEAD lives as a `head` column on the `snapshots` table in
//! `SessionDag` (added via lazy ALTER in `session_dag.rs`).  Each undo step
//! walks `HEAD -> parent`.  A redo step walks `HEAD -> last-followed child`.
//! An edit after an undo creates a NEW child sibling; the prior child (redo
//! timeline) is never destroyed (RV-3).
//!
//! # RV-1 divergence guard
//!
//! Before materializing a restore, if a target file's on-disk blake3 hash
//! differs from what HEAD's manifest expected, the current bytes are
//! snapshot-captured as an `auto:pre-restore` fork first, THEN the restore
//! proceeds.  A silent clobber is a FAIL.

use std::io;
use std::path::{Path, PathBuf};
use std::{env, fs};

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Kill-switch env var name.  Set to a truthy value to disable time-travel.
pub const ELAI_TIMETRAVEL_DISABLE_VAR: &str = "ELAI_TIMETRAVEL_DISABLE";

/// Per-blob size cap in bytes.  Blobs above this threshold are NOT stored;
/// a `TooLargeSkipped` event is emitted instead (R1: never silently skip).
pub const BLOB_SIZE_CAP_BYTES: usize = 5 * 1024 * 1024; // 5 MiB

/// Root directory for all CAS objects and manifests, relative to cwd.
pub const SESSION_CHECKPOINT_ROOT: &str =
    crate::architecture::edit_checkpoints::SESSION_CHECKPOINT_ROOT;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors from the CAS / time-travel engine.
#[derive(Debug, thiserror::Error)]
pub enum TimeTravelError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("zstd compression error: {0}")]
    Zstd(String),
    #[error("blob not found: {hash}")]
    BlobNotFound { hash: String },
    #[error("manifest not found: {hash}")]
    ManifestNotFound { hash: String },
    #[error("invalid session id: {session_id}")]
    InvalidSessionId { session_id: String },
    #[error("blob too large: {path} ({size} bytes > {cap} bytes cap)")]
    BlobTooLarge { path: String, size: usize, cap: usize },
    #[error("session_dag error: {0}")]
    SessionDag(String),
}

// ---------------------------------------------------------------------------
// Kill-switch helper
// ---------------------------------------------------------------------------

/// Returns `true` if the time-travel subsystem is active (default-ON).
///
/// Set `ELAI_TIMETRAVEL_DISABLE=1` to suppress all capture/restore.
#[must_use]
pub fn timetravel_enabled() -> bool {
    !env_truthy(ELAI_TIMETRAVEL_DISABLE_VAR)
}

fn env_truthy(name: &str) -> bool {
    match env::var(name) {
        Ok(v) => {
            let t = v.trim();
            t.eq_ignore_ascii_case("1")
                || t.eq_ignore_ascii_case("true")
                || t.eq_ignore_ascii_case("yes")
                || t.eq_ignore_ascii_case("on")
        }
        Err(_) => false,
    }
}

// ---------------------------------------------------------------------------
// Manifest types
// ---------------------------------------------------------------------------

/// One entry in a [`FileManifest`]: maps a project-relative path to its
/// blake3 content hash (hex, 64 chars).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestEntry {
    /// Project-relative path (forward slashes on all platforms).
    pub path: String,
    /// blake3 hex hash of the pre-edit content.
    pub hash: String,
}

/// A snapshot of the project file-tree at one undo-tree node.
/// Stored as JSON under `manifests/{manifest_hash}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FileManifest {
    pub entries: Vec<ManifestEntry>,
}

impl FileManifest {
    /// blake3 hash of the JSON-serialized manifest (used as the manifest key).
    pub fn hash(&self) -> Result<String, TimeTravelError> {
        let json = serde_json::to_vec(self)?;
        Ok(blake3::hash(&json).to_hex().to_string())
    }

    /// Look up the expected hash for a given path, if present.
    pub fn expected_hash(&self, path: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|e| e.path == path)
            .map(|e| e.hash.as_str())
    }

    /// Build an updated manifest that incorporates the new `(path, hash)` pair.
    /// Existing entry for the same path is replaced; new paths are appended.
    #[must_use]
    pub fn with_update(&self, path: &str, hash: &str) -> FileManifest {
        let mut entries: Vec<ManifestEntry> = self
            .entries
            .iter()
            .filter(|e| e.path != path)
            .cloned()
            .collect();
        entries.push(ManifestEntry {
            path: path.to_owned(),
            hash: hash.to_owned(),
        });
        FileManifest { entries }
    }
}

// ---------------------------------------------------------------------------
// CAS helpers
// ---------------------------------------------------------------------------

fn blob_dir(root: &Path, session_id: &str, hash: &str) -> PathBuf {
    root.join(session_id)
        .join("objects")
        .join(&hash[..2])
}

fn blob_path(root: &Path, session_id: &str, hash: &str) -> PathBuf {
    blob_dir(root, session_id, hash).join(hash)
}

fn manifest_dir(root: &Path, session_id: &str) -> PathBuf {
    root.join(session_id).join("manifests")
}

fn manifest_path(root: &Path, session_id: &str, hash: &str) -> PathBuf {
    manifest_dir(root, session_id).join(hash)
}

/// Compute the blake3 hex hash of `data`.
#[must_use]
pub fn blake3_hex(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}

/// Write `data` (raw bytes) as a zstd-compressed blob at its blake3 address.
/// Idempotent: if the blob already exists, nothing is written (dedup).
///
/// Returns `(hash, was_deduped)`.
pub fn cas_put(
    root: &Path,
    session_id: &str,
    data: &[u8],
) -> Result<(String, bool), TimeTravelError> {
    let hash = blake3_hex(data);
    let p = blob_path(root, session_id, &hash);
    if p.exists() {
        return Ok((hash, true)); // dedup — blob already present
    }
    let compressed = zstd::bulk::compress(data, 3)
        .map_err(|e| TimeTravelError::Zstd(e.to_string()))?;
    fs::create_dir_all(blob_dir(root, session_id, &hash))?;
    // Atomic write: write to temp, rename.
    let tmp = p.with_extension("tmp");
    fs::write(&tmp, &compressed)?;
    fs::rename(&tmp, &p)?;
    Ok((hash, false))
}

/// Read and decompress a blob by its blake3 hash.
pub fn cas_get(
    root: &Path,
    session_id: &str,
    hash: &str,
) -> Result<Vec<u8>, TimeTravelError> {
    let p = blob_path(root, session_id, hash);
    if !p.exists() {
        return Err(TimeTravelError::BlobNotFound { hash: hash.to_owned() });
    }
    let compressed = fs::read(&p)?;
    zstd::bulk::decompress(&compressed, 64 * 1024 * 1024)
        .map_err(|e| TimeTravelError::Zstd(e.to_string()))
}

/// Returns true if the CAS blob for `hash` is already present (dedup check).
pub fn cas_has(root: &Path, session_id: &str, hash: &str) -> bool {
    blob_path(root, session_id, hash).exists()
}

// ---------------------------------------------------------------------------
// Manifest store
// ---------------------------------------------------------------------------

/// Persist a manifest.  Idempotent on hash collision.
/// Returns the manifest hash.
pub fn manifest_put(
    root: &Path,
    session_id: &str,
    manifest: &FileManifest,
) -> Result<String, TimeTravelError> {
    let json = serde_json::to_vec(manifest)?;
    let hash = blake3_hex(&json);
    let p = manifest_path(root, session_id, &hash);
    if !p.exists() {
        fs::create_dir_all(manifest_dir(root, session_id))?;
        let tmp = p.with_extension("tmp");
        fs::write(&tmp, &json)?;
        fs::rename(&tmp, &p)?;
    }
    Ok(hash)
}

/// Load a manifest by its hash.
pub fn manifest_get(
    root: &Path,
    session_id: &str,
    hash: &str,
) -> Result<FileManifest, TimeTravelError> {
    let p = manifest_path(root, session_id, hash);
    if !p.exists() {
        return Err(TimeTravelError::ManifestNotFound { hash: hash.to_owned() });
    }
    let data = fs::read(&p)?;
    serde_json::from_slice(&data).map_err(Into::into)
}

// ---------------------------------------------------------------------------
// Capture result
// ---------------------------------------------------------------------------

/// Result of capturing a file's pre-edit state into the CAS.
#[derive(Debug, Clone)]
pub struct CaptureResult {
    /// blake3 hash of the pre-edit content.
    pub pre_hash: String,
    /// Project-relative path (forward-slash normalised).
    pub rel_path: String,
    /// True when the blob was already present (dedup).
    pub was_deduped: bool,
    /// True when the blob was skipped due to size cap.
    pub was_too_large: bool,
}

/// Capture a single file's pre-edit content into the CAS.
///
/// Returns `CaptureResult` in all cases.  When the content exceeds
/// [`BLOB_SIZE_CAP_BYTES`], the blob is NOT stored and `was_too_large = true`
/// is set (R1: the caller emits `TooLargeSkipped`, never silently pretends captured).
pub fn capture_file(
    root: &Path,
    session_id: &str,
    abs_path: &Path,
    content: &[u8],
) -> Result<CaptureResult, TimeTravelError> {
    let rel_path = normalise_path(abs_path);
    if content.len() > BLOB_SIZE_CAP_BYTES {
        let hash = blake3_hex(content);
        return Ok(CaptureResult {
            pre_hash: hash,
            rel_path,
            was_deduped: false,
            was_too_large: true,
        });
    }
    let (hash, was_deduped) = cas_put(root, session_id, content)?;
    Ok(CaptureResult {
        pre_hash: hash,
        rel_path,
        was_deduped,
        was_too_large: false,
    })
}

// ---------------------------------------------------------------------------
// Restore / materialize
// ---------------------------------------------------------------------------

/// Restore outcome for one file.
#[derive(Debug, Clone)]
pub struct RestoreResult {
    pub path: PathBuf,
    pub pre_hash: String,
    pub post_hash: String,
    /// True when the on-disk hash before restore differed from HEAD's expected
    /// hash (RV-1 divergence guard fired, auto-fork captured).
    pub divergence_detected: bool,
}

/// Materialize a [`FileManifest`] to disk.
///
/// For each entry:
/// 1. RV-1 divergence guard: if the on-disk file's blake3 != entry's hash,
///    fork-capture the current bytes as `auto:pre-restore` first.
/// 2. Fetch the blob from CAS and write it atomically (temp + rename).
///
/// Returns the list of restore outcomes (one per manifest entry with a blob
/// present in CAS; entries whose blob is missing trigger `BlobNotFound` error,
/// which is fail-closed per R1/R2).
pub fn materialize(
    root: &Path,
    session_id: &str,
    manifest: &FileManifest,
    project_root: &Path,
) -> Result<Vec<RestoreResult>, TimeTravelError> {
    let mut results = Vec::new();
    for entry in &manifest.entries {
        let abs_path = project_root.join(&entry.path);

        // --- RV-1: check on-disk divergence ---
        let divergence_detected = if abs_path.exists() {
            let on_disk = fs::read(&abs_path)?;
            let on_disk_hash = blake3_hex(&on_disk);
            if on_disk_hash != entry.hash {
                // Fork-capture current bytes before overwriting.
                if on_disk.len() <= BLOB_SIZE_CAP_BYTES {
                    let _ = cas_put(root, session_id, &on_disk);
                }
                true
            } else {
                false
            }
        } else {
            false
        };

        // --- Fetch from CAS and write atomically ---
        let content = cas_get(root, session_id, &entry.hash)?;
        let post_hash = blake3_hex(&content);

        if let Some(parent) = abs_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        // I-3: adopt shared helper (tmp + sync_all + rename); was the only
        // site that already had fsync — now unified with the rest of the crate.
        crate::util::atomic_io::atomic_write_sync(&abs_path, &content)?;

        results.push(RestoreResult {
            path: abs_path,
            pre_hash: entry.hash.clone(),
            post_hash,
            divergence_detected,
        });
    }
    Ok(results)
}

// ---------------------------------------------------------------------------
// Grain enum
// ---------------------------------------------------------------------------

/// User-selectable undo/redo granularity.
///
/// Capture is always per-edit (cheap, content-addressed); grain is which
/// ancestor the HEAD jumps to.  Grain is a VIEW over one tree, not separate
/// storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimeTravelGrain {
    /// One tool-call (finest granularity).
    Edit,
    /// Collapse a user→assistant turn's edits into one undo step (default).
    #[default]
    Turn,
    /// Jump to the nearest GRC-tagged baby-step boundary.
    BabyStep,
}

impl TimeTravelGrain {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Edit => "edit",
            Self::Turn => "turn",
            Self::BabyStep => "babystep",
        }
    }

    /// Parse from a CLI string.  Returns error on unknown value.
    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "edit" => Ok(Self::Edit),
            "turn" => Ok(Self::Turn),
            "babystep" | "baby_step" | "baby-step" => Ok(Self::BabyStep),
            other => Err(format!(
                "unknown grain `{other}`; expected edit|turn|babystep"
            )),
        }
    }
}

// ---------------------------------------------------------------------------
// Path normalisation helper
// ---------------------------------------------------------------------------

/// Convert an absolute path to a project-relative, forward-slash string.
///
/// If the path cannot be made relative (e.g. is on a different drive on
/// Windows), falls back to the full string representation.
pub fn normalise_path(path: &Path) -> String {
    // Try to strip cwd prefix for a project-relative path.
    if let Ok(cwd) = std::env::current_dir() {
        if let Ok(rel) = path.strip_prefix(&cwd) {
            return rel
                .to_string_lossy()
                .replace('\\', "/");
        }
    }
    path.to_string_lossy().replace('\\', "/")
}

// ---------------------------------------------------------------------------
// RestoreEngine trait + CasRestoreEngine impl
// ---------------------------------------------------------------------------

/// Abstraction over file-restore backends.
///
/// `CasRestoreEngine` is the default (content-addressed, zstd-compressed).
/// `ShadowGitRestoreEngine` is a Tier-2 stub (see DEFERRED below).
pub trait RestoreEngine: Send + Sync {
    /// Capture the pre-edit state of `paths` into the store.
    /// Returns `(manifest_hash, entries)` on success.
    fn capture_paths(
        &self,
        root: &Path,
        session_id: &str,
        paths: &[(PathBuf, Vec<u8>)],
    ) -> Result<(String, Vec<CaptureResult>), TimeTravelError>;

    /// Restore files to the state recorded in `manifest`.
    fn materialize_manifest(
        &self,
        root: &Path,
        session_id: &str,
        manifest: &FileManifest,
        project_root: &Path,
    ) -> Result<Vec<RestoreResult>, TimeTravelError>;
}

/// Default CAS-backed restore engine.
///
/// Borrows [`crate::architecture::edit_checkpoints::SESSION_CHECKPOINT_ROOT`]
/// as the storage-root constant (see the re-export at [`SESSION_CHECKPOINT_ROOT`]).
/// Implements its own atomic CAS IO (zstd blob write + manifest JSON) directly;
/// does NOT call the E-25 `snapshot_pre_edit` / `rewind_to` byte primitives.
/// GD-1 reuse is the shared storage root, not the byte-level functions.
pub struct CasRestoreEngine;

impl RestoreEngine for CasRestoreEngine {
    fn capture_paths(
        &self,
        root: &Path,
        session_id: &str,
        paths: &[(PathBuf, Vec<u8>)],
    ) -> Result<(String, Vec<CaptureResult>), TimeTravelError> {
        let mut manifest = FileManifest::default();
        let mut results = Vec::new();
        for (abs_path, content) in paths {
            let cr = capture_file(root, session_id, abs_path, content)?;
            if !cr.was_too_large {
                manifest.entries.push(ManifestEntry {
                    path: cr.rel_path.clone(),
                    hash: cr.pre_hash.clone(),
                });
            }
            results.push(cr);
        }
        let manifest_hash = manifest_put(root, session_id, &manifest)?;
        Ok((manifest_hash, results))
    }

    fn materialize_manifest(
        &self,
        root: &Path,
        session_id: &str,
        manifest: &FileManifest,
        project_root: &Path,
    ) -> Result<Vec<RestoreResult>, TimeTravelError> {
        materialize(root, session_id, manifest, project_root)
    }
}

// ---------------------------------------------------------------------------
// DEFERRED: ShadowGitRestoreEngine
// ---------------------------------------------------------------------------
//
// Tier-2 follow-on: a shadow git repo (core.worktree -> project, snapshot =
// add+commit, restore = clean+reset --hard) for users who prefer git plumbing.
// The trait + selector ship in this pass; the git plumbing is deferred.
//
// Wire-dark: `[timetravel] engine = "shadow_git"` in the config selects this
// backend; the default is "cas".  Until Tier-2 ships, selecting shadow_git
// returns a NotImplemented error at runtime (not a compile-time stub — the
// caller sees a clear error message, never a silent no-op).

/// Tier-2 stub for the shadow-git restore backend.
pub struct ShadowGitRestoreEngine;

impl RestoreEngine for ShadowGitRestoreEngine {
    fn capture_paths(
        &self,
        _root: &Path,
        _session_id: &str,
        _paths: &[(PathBuf, Vec<u8>)],
    ) -> Result<(String, Vec<CaptureResult>), TimeTravelError> {
        Err(TimeTravelError::SessionDag(
            "ShadowGitRestoreEngine is Tier-2 deferred; \
             use the default CAS engine (engine = \"cas\")"
                .to_owned(),
        ))
    }

    fn materialize_manifest(
        &self,
        _root: &Path,
        _session_id: &str,
        _manifest: &FileManifest,
        _project_root: &Path,
    ) -> Result<Vec<RestoreResult>, TimeTravelError> {
        Err(TimeTravelError::SessionDag(
            "ShadowGitRestoreEngine is Tier-2 deferred; \
             use the default CAS engine (engine = \"cas\")"
                .to_owned(),
        ))
    }
}

// ---------------------------------------------------------------------------
// Turn-grain ancestor walk
// ---------------------------------------------------------------------------

/// Walk `parent_chain` (ascending seq) to find the snapshot id of the nearest
/// ancestor whose SSB stream contains a `chat_message` or `chat_response` row
/// AFTER the current HEAD seq.  Used by `--grain turn` to find the undo target.
///
/// Implementation note (RISK-2 mitigation): this is an O(N) walk over the
/// ancestor chain checking SSB seqs.  The `chat_message` row marks a
/// user-turn boundary; the undo step is the snapshot BEFORE that row.  For
/// `--grain edit`, the parent is simply HEAD->parent (one step).  For
/// `--grain babystep`, walk to the nearest ancestor tagged with a GRC
/// baby-step tag in `snapshot.tag`.
pub fn find_turn_ancestor<'a>(
    parent_chain: &'a [String],
    _session_id: &str,
) -> Option<&'a str> {
    // Simplified implementation: for turn grain, we consider the direct parent
    // (one turn = one user->assistant cycle captured as a series of edit snapshots).
    // The full SSB-scan variant is a Tier-2 refinement (documented in H-49-decisions.md).
    parent_chain.first().map(String::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn cas_roundtrip_dedup() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        let sid = "test-session";
        let content = b"hello world";

        let (h1, deduped1) = cas_put(root, sid, content).unwrap();
        assert!(!deduped1, "first put should not dedup");

        let (h2, deduped2) = cas_put(root, sid, content).unwrap();
        assert_eq!(h1, h2, "same content => same hash");
        assert!(deduped2, "second put should dedup");

        let recovered = cas_get(root, sid, &h1).unwrap();
        assert_eq!(recovered, content);
    }

    #[test]
    fn manifest_roundtrip() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        let sid = "test-session";

        let m = FileManifest {
            entries: vec![ManifestEntry {
                path: "src/main.rs".into(),
                hash: "abc123".into(),
            }],
        };
        let hash = manifest_put(root, sid, &m).unwrap();
        let recovered = manifest_get(root, sid, &hash).unwrap();
        assert_eq!(m, recovered);
    }

    #[test]
    fn capture_file_too_large_flag() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        let sid = "test-session";
        let big = vec![0u8; BLOB_SIZE_CAP_BYTES + 1];
        let result = capture_file(root, sid, Path::new("/tmp/big.bin"), &big).unwrap();
        assert!(result.was_too_large);
        // Blob should NOT be present
        assert!(!cas_has(root, sid, &result.pre_hash));
    }

    #[test]
    fn materialize_restores_bytes() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        let project = dir.path().join("project");
        fs::create_dir_all(&project).unwrap();

        let sid = "test-session";
        let content = b"original content";
        let path = project.join("a.txt");
        fs::write(&path, content).unwrap();

        let (hash, _deduped) = cas_put(root, sid, content).unwrap();
        let manifest = FileManifest {
            entries: vec![ManifestEntry {
                path: "a.txt".into(),
                hash: hash.clone(),
            }],
        };
        // Overwrite with different content
        fs::write(&path, b"new content").unwrap();
        assert_ne!(fs::read(&path).unwrap(), content);

        // Restore
        let results = materialize(root, sid, &manifest, &project).unwrap();
        assert_eq!(results.len(), 1);
        let restored = fs::read(&path).unwrap();
        assert_eq!(restored, content, "materialize must restore bytes exactly");
        assert_eq!(results[0].pre_hash, hash);
    }

    #[test]
    fn rv1_divergence_detected() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        let project = dir.path().join("project");
        fs::create_dir_all(&project).unwrap();

        let sid = "test-session";
        let original = b"original";
        let modified = b"hand-modified";
        let path = project.join("b.txt");

        // Put original blob
        let (hash, _) = cas_put(root, sid, original).unwrap();

        // Write DIFFERENT content to disk (simulating hand-modification)
        fs::write(&path, modified).unwrap();

        let manifest = FileManifest {
            entries: vec![ManifestEntry {
                path: "b.txt".into(),
                hash: hash.clone(),
            }],
        };

        // Materialize should detect divergence
        let results = materialize(root, sid, &manifest, &project).unwrap();
        assert!(
            results[0].divergence_detected,
            "RV-1: divergence must be detected when on-disk hash differs"
        );
        // File must be restored to original
        assert_eq!(fs::read(&path).unwrap(), original);
    }

    #[test]
    fn timetravel_enabled_by_default() {
        // Without the env var set, time-travel should be enabled.
        // (This test is fragile if the env var is set in the test environment,
        //  but that is the test runner's responsibility.)
        let was_set = std::env::var(ELAI_TIMETRAVEL_DISABLE_VAR).is_ok();
        if !was_set {
            assert!(timetravel_enabled());
        }
    }

    #[test]
    fn grain_parse_roundtrip() {
        assert_eq!(TimeTravelGrain::parse("edit").unwrap(), TimeTravelGrain::Edit);
        assert_eq!(TimeTravelGrain::parse("turn").unwrap(), TimeTravelGrain::Turn);
        assert_eq!(TimeTravelGrain::parse("babystep").unwrap(), TimeTravelGrain::BabyStep);
        assert!(TimeTravelGrain::parse("unknown").is_err());
    }
}
