//! F-13 — session archive on SSB schema-bump (wire-dark, forensic opt-in).
//!
//! Closes the most-damaging edge of the "SSB schema bumps are DESTRUCTIVE" gap
//! (gap #2 vs Springdrift's Backup Actor, arXiv:2604.04660 /
//! workflow/research/shard-prior-art/07-springdrift.md:49,:110): `run_migration`
//! DROPs the prior-version `ssb`/`ssb_files` tables on a version mismatch,
//! silently destroying that session's audit journal before recreating it. This
//! module copies the stale-version `ssb.db` (and, precautionarily, the
//! `session_dag.db`) to `workflow/sessions/{id}/archive/ssb-v{N}.sqlite` BEFORE
//! the DROP, gated behind one `ELAI_*_ENABLE` kill-switch.
//!
//! W-2 DISPOSITION (recorded in workflow/reasoning/F-13-session-archive-decisions.md):
//! this guard ships DEFAULT-OFF by design. It is a forensic opt-in over data the
//! migration's own comment marks "data loss is acceptable for old sessions"
//! (ssb.rs near the DROP), NOT a security/enforcement gate where default-off
//! would mean zero protection. R9 is satisfied by a wire-dark-allowlist.toml row
//! (added out-of-band by the controller; the gate is R8-frozen). This is the
//! F-11 in-file W-2 deviation precedent (F-0_INDEX.md:210), not a silent ship.
//!
//! Deliberate NON-adoption of the Backup-Actor mechanism: NO periodic
//! committer, NO `tokio::time::interval`, NO git shell-out, NO new dependency
//! (rusqlite + std::fs + blake3 are all already in-tree). The archive is a
//! single file-copy at the one destructive site. AGPL-3.0 clean-room: only the
//! archive-before-destroy CONCEPT is reused; no Springdrift code is copied.
//!
//! blake3 ONLY for the manifest digest (CLAUDE.md HARD RULE; mirrors
//! checkpoint.rs:194-195 `content_hash`). NO `.unwrap()`/`.expect()` on the
//! library paths — errors propagate via [`ArchiveError`] (thiserror).

use std::path::{Path, PathBuf};

use thiserror::Error;

/// Environment-variable name controlling the wire-dark gate on session-archive-
/// on-schema-bump. Set to a truthy value (`"1"` / `"true"` / `"yes"` / `"on"`)
/// to enable. Default OFF (forensic opt-in; see the W-2 disposition above).
///
/// Mirrors the `architecture/*_guard.rs` env-gate const convention.
pub const ARCHIVE_ON_BUMP_ENV: &str = "ELAI_SSB_ARCHIVE_ON_BUMP_ENABLE";

/// The per-session workflow root under which the archive subdir nests. The
/// canonical per-session artifact root (CLAUDE.md hook-executions.log /
/// SESSION_HANDOFF path convention). Resolved relative to the process CWD,
/// matching every other `workflow/sessions/{id}/...` writer in the harness.
const SESSIONS_ROOT: &str = "workflow/sessions";

/// The archive subdir name nested under `workflow/sessions/{id}/`.
const ARCHIVE_SUBDIR: &str = "archive";

/// Number of hex characters of the blake3 digest kept as the manifest prefix.
/// Matches the in-tree convention (`format!("{:.16}", h.to_hex())`,
/// ssb.rs test builders + checkpoint.rs content_hash style).
const MANIFEST_HEX_PREFIX_LEN: usize = 16;

/// Errors produced by the session-archive path. Best-effort forensics: the
/// caller (`run_migration`) treats these as fail-OPEN (log + continue the DROP)
/// because blocking the DROP would brick session startup.
#[derive(Debug, Error)]
pub enum ArchiveError {
    /// The source DB file to archive does not exist on disk.
    #[error("archive source db not found: {0}")]
    SourceMissing(PathBuf),

    /// Creating the archive destination directory failed.
    #[error("failed to create archive dir {path}: {source}")]
    CreateDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Copying the source DB to the archive destination failed.
    #[error("failed to copy {from} -> {to}: {source}")]
    Copy {
        from: PathBuf,
        to: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Reading the archived bytes back for the manifest digest failed.
    #[error("failed to read archived file {path} for digest: {source}")]
    DigestRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

/// Returns `true` iff [`ARCHIVE_ON_BUMP_ENV`] is set to a recognised truthy
/// value. Default OFF: any unset/empty/unrecognised value is `false` (the
/// no-op). Mirrors the `architecture/*_guard.rs` truthy-set convention.
#[must_use]
pub fn archive_on_bump_enabled() -> bool {
    env_is_truthy(ARCHIVE_ON_BUMP_ENV)
}

/// Truthy-env helper (named, no magic literals). Recognises `1` / `true` /
/// `yes` / `on` case-insensitively; everything else (including unset) is false.
fn env_is_truthy(var: &str) -> bool {
    match std::env::var(var) {
        Ok(v) => {
            let t = v.trim().to_ascii_lowercase();
            t == "1" || t == "true" || t == "yes" || t == "on"
        }
        Err(_) => false,
    }
}

/// Compute the archive destination path for a store at the given stale version:
/// `workflow/sessions/{session_id}/archive/ssb-v{stale_version}.sqlite`.
/// `db_file_stem` is the logical store name embedded in the file name (`ssb`
/// for the SSB journal, `session_dag` for the precautionary DAG copy).
fn archive_dest(session_id: &str, stale_version: i64, db_file_stem: &str) -> PathBuf {
    PathBuf::from(SESSIONS_ROOT)
        .join(session_id)
        .join(ARCHIVE_SUBDIR)
        .join(format!("{db_file_stem}-v{stale_version}.sqlite"))
}

/// Copy `src` to `dest`, creating the destination's parent dir. Shared by the
/// SSB and session-DAG archive paths. Returns the destination on success.
fn copy_to_archive(src: &Path, dest: PathBuf) -> Result<PathBuf, ArchiveError> {
    if !src.exists() {
        return Err(ArchiveError::SourceMissing(src.to_path_buf()));
    }
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|source| ArchiveError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    std::fs::copy(src, &dest).map_err(|source| ArchiveError::Copy {
        from: src.to_path_buf(),
        to: dest.clone(),
        source,
    })?;
    Ok(dest)
}

/// Archive the stale-version SSB db file BEFORE the destructive DROP, gated on
/// [`archive_on_bump_enabled`].
///
/// * Disabled -> `Ok(None)` (the no-op; zero behaviour change vs today).
/// * Enabled  -> copies `db_path` to
///   `workflow/sessions/{session_id}/archive/ssb-v{stale_version}.sqlite` and
///   returns `Ok(Some(dest))`.
///
/// WAL byte-faithfulness is the CALLER's responsibility: `run_migration`
/// runs `PRAGMA wal_checkpoint(TRUNCATE)` on the live writer connection BEFORE
/// calling this, so a raw file copy captures all pages (the archive's blake3
/// prefix then matches a re-read of the file). This helper is a pure file copy;
/// it does not touch the live connection.
pub fn archive_stale_ssb(
    db_path: &Path,
    session_id: &str,
    stale_version: i64,
) -> Result<Option<PathBuf>, ArchiveError> {
    if !archive_on_bump_enabled() {
        return Ok(None);
    }
    let dest = archive_dest(session_id, stale_version, "ssb");
    copy_to_archive(db_path, dest).map(Some)
}

/// Precautionary copy of the session-DAG db file, gated on the same env var.
///
/// The session-DAG migration is NON-destructive (CREATE TABLE IF NOT EXISTS +
/// lazy ALTER in `SessionDag::open()`), so this is belt-and-suspenders, not
/// loss-prevention (the W-7 honesty correction vs the source verdict). Returns
/// `Ok(None)` when disabled OR when the DAG file does not exist (a fresh
/// session may have no DAG db yet — that is not an error here).
pub fn archive_session_dag(
    dag_path: &Path,
    session_id: &str,
    stale_version: i64,
) -> Result<Option<PathBuf>, ArchiveError> {
    if !archive_on_bump_enabled() {
        return Ok(None);
    }
    if !dag_path.exists() {
        return Ok(None);
    }
    let dest = archive_dest(session_id, stale_version, "session_dag");
    copy_to_archive(dag_path, dest).map(Some)
}

/// Compute the blake3 hex-prefix digest of the archived file's bytes, written
/// into the `session_archived` SSB row so the archive is integrity-checkable
/// later. blake3 ONLY (CLAUDE.md HARD RULE); mirrors checkpoint.rs:194-195.
///
/// Returns the first [`MANIFEST_HEX_PREFIX_LEN`] hex chars of the blake3 hash,
/// matching the in-tree `format!("{:.16}", h.to_hex())` convention.
pub fn archive_manifest_digest(dest: &Path) -> Result<String, ArchiveError> {
    let bytes = std::fs::read(dest).map_err(|source| ArchiveError::DigestRead {
        path: dest.to_path_buf(),
        source,
    })?;
    let hash = blake3::hash(&bytes);
    let full = hash.to_hex();
    // `to_hex()` yields 64 hex chars; take the documented prefix length.
    let prefix: String = full.as_str().chars().take(MANIFEST_HEX_PREFIX_LEN).collect();
    Ok(prefix)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // Tests that mutate ARCHIVE_ON_BUMP_ENV serialise through this lock; cargo
    // runs unit tests in parallel and the env var is a process-global.
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn write_dummy_sqlite(path: &Path) {
        let conn = rusqlite::Connection::open(path).expect("open dummy db");
        conn.execute_batch("CREATE TABLE t (a INTEGER); INSERT INTO t VALUES (1);")
            .expect("seed dummy db");
    }

    #[test]
    fn dest_path_is_per_session_archive_subdir() {
        let dest = archive_dest("sess-1", 205, "ssb");
        let s = dest.to_string_lossy().replace('\\', "/");
        assert!(
            s.ends_with("workflow/sessions/sess-1/archive/ssb-v205.sqlite"),
            "unexpected dest path: {s}"
        );
    }

    #[test]
    fn disabled_is_noop() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        std::env::remove_var(ARCHIVE_ON_BUMP_ENV);
        assert!(!archive_on_bump_enabled());
        let tmp = TempDir::new().expect("tempdir");
        let db = tmp.path().join("ssb.db");
        write_dummy_sqlite(&db);
        let prev = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(tmp.path()).expect("chdir");
        let out = archive_stale_ssb(&db, "noop-sess", 204).expect("archive off");
        std::env::set_current_dir(prev).expect("restore");
        assert!(out.is_none(), "disabled archive must be Ok(None)");
    }

    #[test]
    fn enabled_copies_and_digest_is_stable() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = TempDir::new().expect("tempdir");
        let db = tmp.path().join("ssb.db");
        write_dummy_sqlite(&db);
        let prev = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(tmp.path()).expect("chdir");
        std::env::set_var(ARCHIVE_ON_BUMP_ENV, "1");
        let dest = archive_stale_ssb(&db, "on-sess", 204)
            .expect("archive on")
            .expect("Some(dest)");
        assert!(dest.exists(), "archive must exist");
        let d1 = archive_manifest_digest(&dest).expect("digest1");
        let d2 = archive_manifest_digest(&dest).expect("digest2");
        std::env::remove_var(ARCHIVE_ON_BUMP_ENV);
        std::env::set_current_dir(prev).expect("restore");
        assert_eq!(d1, d2, "digest must be stable across reads");
        assert_eq!(d1.len(), MANIFEST_HEX_PREFIX_LEN, "digest is the hex prefix");
    }

    #[test]
    fn missing_source_is_error_when_enabled() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = TempDir::new().expect("tempdir");
        let missing = tmp.path().join("does-not-exist.db");
        let prev = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(tmp.path()).expect("chdir");
        std::env::set_var(ARCHIVE_ON_BUMP_ENV, "1");
        let res = archive_stale_ssb(&missing, "miss-sess", 204);
        std::env::remove_var(ARCHIVE_ON_BUMP_ENV);
        std::env::set_current_dir(prev).expect("restore");
        assert!(matches!(res, Err(ArchiveError::SourceMissing(_))));
    }

    #[test]
    fn session_dag_missing_file_is_noop_not_error() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let tmp = TempDir::new().expect("tempdir");
        let missing = tmp.path().join("session_dag.db");
        let prev = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(tmp.path()).expect("chdir");
        std::env::set_var(ARCHIVE_ON_BUMP_ENV, "1");
        let out = archive_session_dag(&missing, "dag-sess", 204).expect("dag archive");
        std::env::remove_var(ARCHIVE_ON_BUMP_ENV);
        std::env::set_current_dir(prev).expect("restore");
        assert!(out.is_none(), "absent DAG file is a no-op, not an error");
    }
}
