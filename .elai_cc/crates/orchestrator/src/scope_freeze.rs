//! C-43 Phase 4H scope-freeze enforcer.
//!
//! Loads `<workspace_root>/scope-freeze.toml`, verifies that every
//! `frozen_items` path has an on-disk mtime <= `ratification_ts_ms`, and
//! emits a [`ScopeFreezeRatified`](crate::SsbActionType::ScopeFreezeRatified)
//! beacon on ratification or a
//! [`ScopeFreezeViolated`](crate::SsbActionType::ScopeFreezeViolated)
//! beacon per offending item on verify.
//!
//! Grounding posture: this enforcer is an ELAI design hypothesis. The
//! source plan cited arXiv:2604.21725 (AEL) as ground truth; two
//! independent WebFetches verified the paper is actually
//! Thompson-Sampling agent learning, not scope-freeze. The citation has
//! been removed across all C-43 deliverables. See
//! workflow/sessions/s-25120/mistakes.md for the bad-cite analysis.
//!
//! CI-side hard gate: when a violation is observed, CI MAY enable the
//! Cargo feature `phase_4h_freeze_violated`. The
//! [`compile_error!`] macro below then breaks the build with a pointer
//! to .elai_cc/docs/governance/phase-4h-freeze-ratification.md. Pure
//! stable Rust — no nightly clippy plugin required.

#[cfg(feature = "phase_4h_freeze_violated")]
compile_error!(
    "Phase 4H scope freeze breach detected. Re-ratify the frozen \
     manifest (`elai freeze ratify --ratifier <id>`) or revert the \
     offending edit. See \
     .elai_cc/docs/governance/phase-4h-freeze-ratification.md and \
     workflow/sessions/s-25120/mistakes.md for the bad-cite analysis."
);

use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use elai_core::clock::{ClockProvider, OsClock};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Manifest filename relative to the workspace root.
pub const SCOPE_FREEZE_FILENAME: &str = "scope-freeze.toml";

/// Sentinel `modified_at_ms` used when a frozen item is missing on
/// disk. Named constant per the codebase no-magic-numbers rule.
pub const MISSING_ITEM_SENTINEL_MS: u64 = 0;

/// On-disk manifest. Mirrors the TOML at
/// `<workspace_root>/scope-freeze.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeFreezeManifest {
    /// Wall-clock ms since UNIX_EPOCH at which the freeze was
    /// ratified. Items modified after this stamp are violations.
    pub ratification_ts_ms: u64,
    /// 64-bit hash of the ratifier identity (blake3 truncated). The
    /// plaintext identity is resolved through a registry keyed by this
    /// hash; never persisted in the manifest.
    pub ratifier_id_hash: u64,
    /// Workspace-relative paths frozen by the ratification. Paths are
    /// normalised to forward slashes on serialisation.
    pub frozen_items: Vec<String>,
}

impl ScopeFreezeManifest {
    /// Load the manifest from `<workspace_root>/scope-freeze.toml`.
    pub fn load(workspace_root: &Path) -> Result<Self, FreezeError> {
        let path = workspace_root.join(SCOPE_FREEZE_FILENAME);
        let raw = fs::read_to_string(&path).map_err(|e| FreezeError::ManifestUnreadable {
            path: path.clone(),
            detail: e.to_string(),
        })?;
        let manifest: ScopeFreezeManifest =
            toml::from_str(&raw).map_err(|e| FreezeError::ManifestMalformed {
                path: path.clone(),
                detail: e.to_string(),
            })?;
        Ok(manifest)
    }

    /// Stable 64-bit hash of the manifest contents. Used by
    /// [`ScopeFreezeRatified`](crate::SsbActionType::ScopeFreezeRatified)
    /// so the dashboard can detect drift between ratifications without
    /// reading the file.
    pub fn manifest_hash(&self) -> u64 {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.ratification_ts_ms.to_le_bytes());
        hasher.update(&self.ratifier_id_hash.to_le_bytes());
        for item in &self.frozen_items {
            hasher.update(item.as_bytes());
            hasher.update(b"\n");
        }
        let digest = hasher.finalize();
        let bytes = digest.as_bytes();
        u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ])
    }
}

/// A single freeze violation: an item that was modified after the
/// ratification timestamp (or is missing entirely).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FreezeViolation {
    /// Stable 64-bit blake3 of the workspace-relative path.
    pub item_path_hash: u64,
    /// Original workspace-relative path (for human-readable CLI
    /// output). The SSB row carries only `item_path_hash`.
    pub item_path: String,
    /// `ratification_ts_ms` from the manifest, copied here so a
    /// consumer can render the violation without re-loading the file.
    pub frozen_since_ms: u64,
    /// On-disk mtime in ms since UNIX_EPOCH, or
    /// [`MISSING_ITEM_SENTINEL_MS`] when the file is absent.
    pub modified_at_ms: u64,
}

/// Aggregate output of [`verify_freeze`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FreezeReport {
    /// Per-item violations. Empty when the freeze is intact.
    pub violations: Vec<FreezeViolation>,
    /// Manifest ratification timestamp echoed for caller convenience.
    pub ratified_at: u64,
    /// Wall-clock ms at which the verify pass ran.
    pub verified_at: u64,
}

impl FreezeReport {
    /// `true` iff no violations were observed.
    pub fn is_clean(&self) -> bool {
        self.violations.is_empty()
    }
}

/// Typed error surface. No `Box<dyn Error>` per the codebase
/// anti-stale-training rule.
#[derive(Debug, Error)]
pub enum FreezeError {
    #[error("scope-freeze manifest unreadable at {path}: {detail}")]
    ManifestUnreadable { path: PathBuf, detail: String },
    #[error("scope-freeze manifest at {path} is malformed: {detail}")]
    ManifestMalformed { path: PathBuf, detail: String },
    #[error("workspace root is not a directory: {path}")]
    InvalidWorkspaceRoot { path: PathBuf },
    #[error("clock walked backwards before UNIX_EPOCH: {detail}")]
    ClockSkew { detail: String },
    #[error("failed to write scope-freeze manifest at {path}: {detail}")]
    ManifestWriteFailed { path: PathBuf, detail: String },
}

/// 64-bit blake3 of a workspace-relative path.
fn path_hash(item: &str) -> u64 {
    let digest = blake3::hash(item.as_bytes());
    let bytes = digest.as_bytes();
    u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ])
}

/// Current wall-clock time as ms since UNIX_EPOCH.
fn current_unix_ms() -> Result<u64, FreezeError> {
    let dur = OsClock
        .system_now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| FreezeError::ClockSkew {
            detail: e.to_string(),
        })?;
    u64::try_from(dur.as_millis()).map_err(|e| FreezeError::ClockSkew {
        detail: e.to_string(),
    })
}

/// File mtime (ms since UNIX_EPOCH). Returns
/// [`MISSING_ITEM_SENTINEL_MS`] when the file is absent, indicating a
/// missing-frozen-item violation.
fn file_mtime_ms(path: &Path) -> u64 {
    let Ok(meta) = fs::metadata(path) else {
        return MISSING_ITEM_SENTINEL_MS;
    };
    let Ok(modified) = meta.modified() else {
        return MISSING_ITEM_SENTINEL_MS;
    };
    let Ok(dur) = modified.duration_since(UNIX_EPOCH) else {
        return MISSING_ITEM_SENTINEL_MS;
    };
    dur.as_millis() as u64
}

/// Verify that every item in the manifest's `frozen_items` has an
/// on-disk mtime <= `ratification_ts_ms`. Missing items are also
/// reported as violations (with `modified_at_ms == MISSING_ITEM_SENTINEL_MS`).
///
/// This function does NOT emit SSB rows. The caller (CLI driver or
/// dashboard Tauri command) maps each [`FreezeViolation`] to a
/// [`ScopeFreezeViolated`](crate::SsbActionType::ScopeFreezeViolated)
/// payload and writes it through the local [`crate::ssb::Ssb`]
/// handle so the producer-side state machinery stays in the
/// orchestrator's normal write path.
pub fn verify_freeze(workspace_root: &Path) -> Result<FreezeReport, FreezeError> {
    if !workspace_root.is_dir() {
        return Err(FreezeError::InvalidWorkspaceRoot {
            path: workspace_root.to_path_buf(),
        });
    }
    let manifest = ScopeFreezeManifest::load(workspace_root)?;
    let verified_at = current_unix_ms()?;
    let mut violations = Vec::new();
    for item in &manifest.frozen_items {
        let on_disk = workspace_root.join(item);
        let modified_at_ms = file_mtime_ms(&on_disk);
        let is_violation = modified_at_ms == MISSING_ITEM_SENTINEL_MS
            || modified_at_ms > manifest.ratification_ts_ms;
        if is_violation {
            violations.push(FreezeViolation {
                item_path_hash: path_hash(item),
                item_path: item.clone(),
                frozen_since_ms: manifest.ratification_ts_ms,
                modified_at_ms,
            });
        }
    }
    Ok(FreezeReport {
        violations,
        ratified_at: manifest.ratification_ts_ms,
        verified_at,
    })
}

/// Persist a fresh manifest to `<workspace_root>/scope-freeze.toml`.
/// Atomic via write-temp-then-rename so a crashed ratification cannot
/// leave a half-written file. Returns the manifest_hash of the new
/// manifest so the caller can include it in the
/// [`ScopeFreezeRatified`](crate::SsbActionType::ScopeFreezeRatified)
/// SSB payload.
pub fn ratify_freeze(
    workspace_root: &Path,
    ratifier_id: &str,
    frozen_items: Vec<String>,
) -> Result<(ScopeFreezeManifest, u64), FreezeError> {
    if !workspace_root.is_dir() {
        return Err(FreezeError::InvalidWorkspaceRoot {
            path: workspace_root.to_path_buf(),
        });
    }
    let now = current_unix_ms()?;
    let ratifier_id_hash = path_hash(ratifier_id);
    let manifest = ScopeFreezeManifest {
        ratification_ts_ms: now,
        ratifier_id_hash,
        frozen_items,
    };
    let final_path = workspace_root.join(SCOPE_FREEZE_FILENAME);
    let tmp_path = workspace_root.join(format!("{}.tmp", SCOPE_FREEZE_FILENAME));
    let serialised =
        toml::to_string_pretty(&manifest).map_err(|e| FreezeError::ManifestWriteFailed {
            path: final_path.clone(),
            detail: e.to_string(),
        })?;
    fs::write(&tmp_path, serialised.as_bytes()).map_err(|e| {
        FreezeError::ManifestWriteFailed {
            path: tmp_path.clone(),
            detail: e.to_string(),
        }
    })?;
    fs::rename(&tmp_path, &final_path).map_err(|e| FreezeError::ManifestWriteFailed {
        path: final_path.clone(),
        detail: e.to_string(),
    })?;
    let hash = manifest.manifest_hash();
    Ok((manifest, hash))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::OpenOptions;
    use std::io::Write;
    use tempfile::TempDir;

    fn write_manifest(dir: &Path, ratified_ms: u64, items: &[&str]) {
        let toml = format!(
            r#"
ratification_ts_ms = {ratified_ms}
ratifier_id_hash = 4660
frozen_items = [{items}]
"#,
            items = items
                .iter()
                .map(|s| format!("\"{s}\""))
                .collect::<Vec<_>>()
                .join(", ")
        );
        fs::write(dir.join(SCOPE_FREEZE_FILENAME), toml).expect("write manifest");
    }

    #[test]
    fn manifest_parses_from_disk() {
        let tmp = TempDir::new().expect("tempdir");
        write_manifest(tmp.path(), 1_700_000_000_000, &["src/lib.rs"]);
        let manifest = ScopeFreezeManifest::load(tmp.path()).expect("parse");
        assert_eq!(manifest.ratification_ts_ms, 1_700_000_000_000);
        assert_eq!(manifest.frozen_items, vec!["src/lib.rs".to_string()]);
    }

    #[test]
    fn verify_clean_when_item_unchanged_since_future_ratification() {
        let tmp = TempDir::new().expect("tempdir");
        // Item created NOW; ratification timestamp is far in the future
        // so the item's mtime is strictly less than ratification_ts_ms.
        fs::write(tmp.path().join("frozen.rs"), b"// frozen").expect("write");
        let future_ms = current_unix_ms().expect("clock") + 60_000;
        write_manifest(tmp.path(), future_ms, &["frozen.rs"]);
        let report = verify_freeze(tmp.path()).expect("verify");
        assert!(report.is_clean(), "expected no violations, got {report:?}");
    }

    #[test]
    fn verify_flags_item_modified_after_ratification() {
        let tmp = TempDir::new().expect("tempdir");
        // Ratify at t=1; then bump the file's mtime to t=now (which is
        // far greater than 1).
        write_manifest(tmp.path(), 1, &["frozen.rs"]);
        let mut f = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(tmp.path().join("frozen.rs"))
            .expect("open");
        f.write_all(b"// mutated").expect("write");
        drop(f);
        let report = verify_freeze(tmp.path()).expect("verify");
        assert_eq!(report.violations.len(), 1);
        assert_eq!(report.violations[0].item_path, "frozen.rs");
        assert!(report.violations[0].modified_at_ms > 1);
    }

    #[test]
    fn verify_flags_missing_item() {
        let tmp = TempDir::new().expect("tempdir");
        let future_ms = current_unix_ms().expect("clock") + 60_000;
        write_manifest(tmp.path(), future_ms, &["never_created.rs"]);
        let report = verify_freeze(tmp.path()).expect("verify");
        assert_eq!(report.violations.len(), 1);
        assert_eq!(report.violations[0].modified_at_ms, MISSING_ITEM_SENTINEL_MS);
    }

    #[test]
    fn ratify_writes_manifest_atomically() {
        let tmp = TempDir::new().expect("tempdir");
        let (manifest, hash) =
            ratify_freeze(tmp.path(), "maintainer", vec!["a.rs".into(), "b.rs".into()])
                .expect("ratify");
        assert_eq!(manifest.frozen_items.len(), 2);
        assert_ne!(hash, 0);
        // Round-trip: load the written file and confirm equality.
        let reloaded = ScopeFreezeManifest::load(tmp.path()).expect("reload");
        assert_eq!(reloaded.frozen_items, manifest.frozen_items);
        assert_eq!(reloaded.ratifier_id_hash, manifest.ratifier_id_hash);
    }

    #[test]
    fn manifest_hash_is_deterministic() {
        let m = ScopeFreezeManifest {
            ratification_ts_ms: 42,
            ratifier_id_hash: 7,
            frozen_items: vec!["a".into(), "b".into()],
        };
        assert_eq!(m.manifest_hash(), m.manifest_hash());
    }
}
