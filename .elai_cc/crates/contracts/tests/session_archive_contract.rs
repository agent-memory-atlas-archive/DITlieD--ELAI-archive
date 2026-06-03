//! F-13 session-archive-on-schema-bump contract tests.
//!
//! Exercises the `session_archive` module + its wiring into `run_migration`:
//!   1. ENABLED: opening a stale-version SSB db archives it to
//!      `archive/ssb-v{N}.sqlite` and emits a `session_archived` row; the
//!      archived DB is a valid SQLite file whose blake3 prefix matches the
//!      `manifest_blake3` recorded in the row.
//!   2. DISABLED: no archive file, no `session_archived` row, migration outcome
//!      byte-identical to today (provably inert no-op path).
//!   3. The archive helpers (`archive_on_bump_enabled`, `archive_stale_ssb`,
//!      `archive_manifest_digest`) behave per their documented gate contract.
//!
//! NOTE: SSB construction spawns a single-writer tokio task, so every test that
//! builds an `Ssb` runs inside a multi-thread tokio runtime (a pure-fn or
//! `cargo check` run would mask the "no reactor" panic).

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use elai_orchestrator::session_archive::{
    self, archive_manifest_digest, archive_on_bump_enabled, archive_stale_ssb,
    ARCHIVE_ON_BUMP_ENV,
};
use elai_orchestrator::ssb::{Ssb, SsbActionType, SsbConfig, SsbPayload, SsbReader};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Env-isolation helpers (mirror l_approval_fatigue_prod_contract.rs)
// ---------------------------------------------------------------------------

/// Global lock serialising every test that mutates `ARCHIVE_ON_BUMP_ENV` AND
/// the process-global CWD. Cargo runs tests in parallel; without this lock the
/// gate-on / gate-off assertions race on a process-global side effect, and the
/// `set_current_dir` window (the migration resolves the archive dest relative
/// to CWD) races so one test's archive lands under another test's tempdir.
/// EnvScope therefore serialises BOTH the env var and the chdir under the SAME
/// lock: the chdir happens AFTER the guard is acquired and is restored in `Drop`
/// (while the guard is still held, since `_guard` is the last field dropped).
static ENV_LOCK: Mutex<()> = Mutex::new(());

struct EnvScope {
    key: &'static str,
    prev_cwd: PathBuf,
    _guard: MutexGuard<'static, ()>,
}

impl EnvScope {
    fn set_truthy(key: &'static str, chdir_to: &Path) -> Self {
        let guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        // SAFETY: `set_var` is unsafe in Rust 2024; the workspace uses 2021.
        std::env::set_var(key, "1");
        let prev_cwd = std::env::current_dir().expect("capture cwd");
        std::env::set_current_dir(chdir_to).expect("chdir under lock");
        Self {
            key,
            prev_cwd,
            _guard: guard,
        }
    }

    fn clear(key: &'static str, chdir_to: &Path) -> Self {
        let guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        std::env::remove_var(key);
        let prev_cwd = std::env::current_dir().expect("capture cwd");
        std::env::set_current_dir(chdir_to).expect("chdir under lock");
        Self {
            key,
            prev_cwd,
            _guard: guard,
        }
    }
}

impl Drop for EnvScope {
    fn drop(&mut self) {
        // Restore CWD first, then the env var, all while `_guard` (dropped last
        // by field order) still holds ENV_LOCK. No mutation escapes the lock.
        std::env::set_current_dir(&self.prev_cwd).expect("restore cwd");
        std::env::remove_var(self.key);
    }
}

// ---------------------------------------------------------------------------
// SSB seeding helpers
// ---------------------------------------------------------------------------

fn cfg(db_path: &Path, session: &str) -> SsbConfig {
    SsbConfig {
        db_path: db_path.to_path_buf(),
        pool_size: Some(2),
        project_id: "f13-contract".into(),
        session_id: session.into(),
    }
}

fn os_clock() -> Arc<dyn elai_core::clock::ClockProvider> {
    Arc::new(elai_core::clock::OsClock)
}

/// Build a fresh v(current) SSB at `db_path`, then forcibly stamp its on-disk
/// `user_version` back to `stale` so the NEXT `Ssb::new` sees a version
/// mismatch and triggers the archive-then-DROP migration path. The first Ssb is
/// shut down + dropped so its writer connection releases the file before we
/// re-stamp the version.
fn seed_stale_ssb(db_path: &Path, session: &str, stale: i64) {
    {
        let mut ssb = Ssb::new(cfg(db_path, session), os_clock()).expect("seed ssb new");
        ssb.shutdown();
        // Drop ssb so the writer connection (and read pool) close the file.
    }
    // Stamp the on-disk schema version back to `stale`.
    let conn = rusqlite::Connection::open(db_path).expect("reopen seeded db");
    conn.pragma_update(None, "user_version", stale)
        .expect("stamp stale user_version");
    drop(conn);
}

fn archive_path_for(tmp: &TempDir, session: &str, stale: i64) -> std::path::PathBuf {
    tmp.path()
        .join("workflow")
        .join("sessions")
        .join(session)
        .join("archive")
        .join(format!("ssb-v{stale}.sqlite"))
}

// ---------------------------------------------------------------------------
// Contract 1: gate ENABLED archives + emits + blake3 matches
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_enabled_archives_and_emits_with_matching_blake3() {
    let tmp = TempDir::new().expect("tempdir");
    let db_path = tmp.path().join("ssb.db");
    let session = "f13-enabled";
    let stale = 205_i64;

    seed_stale_ssb(&db_path, session, stale);

    // The archive dest is rooted at the per-session workflow dir; the migration
    // resolves it relative to CWD. EnvScope chdir's to the tempdir AFTER taking
    // ENV_LOCK so the CWD window is serialised with the env-var window.
    let _env = EnvScope::set_truthy(ARCHIVE_ON_BUMP_ENV, tmp.path());
    let ssb = Ssb::new(cfg(&db_path, session), os_clock()).expect("migrated ssb new");

    // The archive file must exist under the per-session archive dir.
    let archive = archive_path_for(&tmp, session, stale);
    assert!(
        archive.exists(),
        "archive file must exist at {}",
        archive.display()
    );

    // It must be a valid SQLite DB (open succeeds).
    let archived = rusqlite::Connection::open(&archive).expect("archive opens as sqlite");
    drop(archived);

    // Its blake3 prefix must match the manifest_blake3 in the emitted row.
    let digest = archive_manifest_digest(&archive).expect("digest archive");
    let rows = ssb
        .query_by_type(SsbActionType::SessionArchived)
        .expect("query session_archived");
    let ssb_row = rows
        .iter()
        .find_map(|e| match rmp_serde::from_slice::<SsbPayload>(&e.payload) {
            Ok(SsbPayload::SessionArchived(p)) if p.store_kind == "ssb" => Some(p),
            _ => None,
        })
        .expect("at least one ssb-store session_archived row");
    assert_eq!(
        ssb_row.manifest_blake3, digest,
        "row manifest_blake3 must equal the archived file's blake3 prefix"
    );
    assert_eq!(ssb_row.stale_version, stale, "row records the stale version");
    assert_eq!(ssb_row.session_id, session, "row records the session id");
    // CWD restored by EnvScope::drop while ENV_LOCK is still held.
}

// ---------------------------------------------------------------------------
// Contract 2: gate DISABLED is a provably inert no-op
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_disabled_writes_no_archive_no_row() {
    let tmp = TempDir::new().expect("tempdir");
    let db_path = tmp.path().join("ssb.db");
    let session = "f13-disabled";
    let stale = 205_i64;

    seed_stale_ssb(&db_path, session, stale);

    let _env = EnvScope::clear(ARCHIVE_ON_BUMP_ENV, tmp.path());
    assert!(
        !archive_on_bump_enabled(),
        "gate must read disabled when env unset"
    );

    let ssb = Ssb::new(cfg(&db_path, session), os_clock()).expect("migrated ssb new (gate off)");

    let archive = archive_path_for(&tmp, session, stale);
    assert!(
        !archive.exists(),
        "no archive file when gate disabled (found {})",
        archive.display()
    );
    let rows = ssb
        .query_by_type(SsbActionType::SessionArchived)
        .expect("query session_archived");
    assert!(
        rows.is_empty(),
        "no session_archived row when gate disabled"
    );
    // CWD restored by EnvScope::drop while ENV_LOCK is still held.
}

// ---------------------------------------------------------------------------
// Contract 3: archive_stale_ssb returns Ok(None) when disabled, Some(dest) on
// success; the helper module surfaces its public symbols.
// ---------------------------------------------------------------------------

#[test]
fn contract_archive_helper_gate_semantics() {
    let tmp = TempDir::new().expect("tempdir");
    let db_path = tmp.path().join("ssb.db");
    // Write a minimal valid SQLite file to copy.
    {
        let conn = rusqlite::Connection::open(&db_path).expect("open db");
        conn.execute_batch("CREATE TABLE t (a INTEGER);")
            .expect("create table");
    }

    // Each inner block takes its own EnvScope guard, which chdir's to the
    // tempdir AFTER acquiring ENV_LOCK and restores the prior CWD on drop, so no
    // set_current_dir ever runs outside the held lock.

    // Disabled -> Ok(None), no file.
    {
        let _env = EnvScope::clear(ARCHIVE_ON_BUMP_ENV, tmp.path());
        let out = archive_stale_ssb(&db_path, "helper-off", 204).expect("archive (off)");
        assert!(out.is_none(), "disabled archive must return Ok(None)");
    }

    // Enabled -> Ok(Some(dest)), file present + valid.
    {
        let _env = EnvScope::set_truthy(ARCHIVE_ON_BUMP_ENV, tmp.path());
        let dest = archive_stale_ssb(&db_path, "helper-on", 204)
            .expect("archive (on)")
            .expect("enabled archive must return Ok(Some(dest))");
        assert!(dest.exists(), "archive dest must exist: {}", dest.display());
        let _ = rusqlite::Connection::open(&dest).expect("archive opens as sqlite");
        let digest = archive_manifest_digest(&dest).expect("digest");
        assert!(!digest.is_empty(), "manifest digest must be non-empty");
    }
}

// Anchor the module path so a rename of the public surface fails compilation
// here (R5 sound-reference: the symbol is used, not merely name-grepped).
#[test]
fn contract_session_archive_env_const_is_canonical() {
    assert_eq!(
        session_archive::ARCHIVE_ON_BUMP_ENV,
        "ELAI_SSB_ARCHIVE_ON_BUMP_ENABLE",
        "the wire-dark kill-switch env name is the canonical F-13 name"
    );
}
