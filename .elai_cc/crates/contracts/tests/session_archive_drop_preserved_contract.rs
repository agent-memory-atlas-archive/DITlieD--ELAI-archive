//! F-13 DROP-semantics-preserved contract (killer_risk #5).
//!
//! The archive guard is ADDITIVE: the destructive `DROP TABLE` in
//! `run_migration` MUST still execute unconditionally AFTER the archive
//! attempt. A stale-version open must yield a FRESH, EMPTY `ssb` table at the
//! new `SCHEMA_VERSION` regardless of the gate state — the archive must NOT
//! leave stale rows in the live DB.
//!
//! Two cases, gate ON and gate OFF, both assert the live DB is recreated clean.
//! When the gate is ON we ALSO assert the pre-DROP bytes were captured to the
//! archive file (so the journal is cold-archived, not destroyed). SSB
//! construction spawns a writer task, so these run on a multi-thread runtime.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use std::sync::Arc;

use elai_orchestrator::session_archive::ARCHIVE_ON_BUMP_ENV;
use elai_orchestrator::ssb::{Ssb, SsbActionType, SsbConfig, SsbPayload, SsbReader};
use tempfile::TempDir;

// ENV_LOCK serialises BOTH the `ARCHIVE_ON_BUMP_ENV` var AND the process-global
// CWD: EnvScope chdir's AFTER acquiring the guard and restores the prior CWD in
// `Drop` (while `_guard`, the last field, still holds the lock). Without this,
// the `set_current_dir` window races under cargo's parallel runner.
static ENV_LOCK: Mutex<()> = Mutex::new(());

struct EnvScope {
    key: &'static str,
    prev_cwd: PathBuf,
    _guard: MutexGuard<'static, ()>,
}

impl EnvScope {
    fn set_truthy(key: &'static str, chdir_to: &Path) -> Self {
        let guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
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
        std::env::set_current_dir(&self.prev_cwd).expect("restore cwd");
        std::env::remove_var(self.key);
    }
}

fn cfg(db_path: &Path, session: &str) -> SsbConfig {
    SsbConfig {
        db_path: db_path.to_path_buf(),
        pool_size: Some(2),
        project_id: "f13-drop".into(),
        session_id: session.into(),
    }
}

fn os_clock() -> Arc<dyn elai_core::clock::ClockProvider> {
    Arc::new(elai_core::clock::OsClock)
}

/// Build a fresh SSB, write ONE real row (a FileWrite), shut it down, then
/// stamp the on-disk `user_version` back to `stale`. The row is the "old
/// journal" the migration must DROP (and, when gated, archive first).
async fn seed_stale_ssb_with_row(db_path: &Path, session: &str, stale: i64) {
    {
        let ssb = Ssb::new(cfg(db_path, session), os_clock()).expect("seed ssb new");
        ssb.write_payload(
            "seed-agent",
            0,
            SsbPayload::FileWrite(elai_orchestrator::ssb::FileWritePayload {
                path: "seed.rs".into(),
                size_bytes: 7,
                hash: "seedhash".into(),
            }),
            vec!["seed.rs".into()],
        )
        .await
        .expect("seed write");
        let mut ssb = ssb;
        ssb.shutdown();
    }
    let conn = rusqlite::Connection::open(db_path).expect("reopen seeded db");
    conn.pragma_update(None, "user_version", stale)
        .expect("stamp stale user_version");
    drop(conn);
}

fn live_user_version(db_path: &Path) -> i64 {
    let conn = rusqlite::Connection::open(db_path).expect("open live db");
    conn.pragma_query_value(None, "user_version", |r| r.get(0))
        .expect("read user_version")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn drop_recreates_clean_db_with_gate_enabled() {
    let tmp = TempDir::new().expect("tempdir");
    let db_path = tmp.path().join("ssb.db");
    let session = "f13-drop-on";
    let stale = 205_i64;

    seed_stale_ssb_with_row(&db_path, session, stale).await;

    let _env = EnvScope::set_truthy(ARCHIVE_ON_BUMP_ENV, tmp.path());
    let ssb = Ssb::new(cfg(&db_path, session), os_clock()).expect("migrated ssb new");

    // DROP still ran: the seed FileWrite row is GONE from the live DB.
    let file_writes = ssb
        .query_by_type(SsbActionType::FileWrite)
        .expect("query file_write");
    assert!(
        file_writes.is_empty(),
        "DROP must have removed the stale-version FileWrite rows from the LIVE db"
    );
    // Live DB is at the new schema version.
    drop(ssb);
    assert_eq!(
        live_user_version(&db_path),
        elai_orchestrator::ssb::SCHEMA_VERSION_PUBLIC,
        "live db must be recreated at the current SCHEMA_VERSION"
    );

    // The pre-DROP journal was cold-archived (gate ON).
    let archive = tmp
        .path()
        .join("workflow")
        .join("sessions")
        .join(session)
        .join("archive")
        .join(format!("ssb-v{stale}.sqlite"));
    assert!(
        archive.exists(),
        "gate ON: the pre-DROP db must be archived at {}",
        archive.display()
    );
    // CWD restored by EnvScope::drop while ENV_LOCK is still held.
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn drop_recreates_clean_db_with_gate_disabled() {
    let tmp = TempDir::new().expect("tempdir");
    let db_path = tmp.path().join("ssb.db");
    let session = "f13-drop-off";
    let stale = 205_i64;

    seed_stale_ssb_with_row(&db_path, session, stale).await;

    let _env = EnvScope::clear(ARCHIVE_ON_BUMP_ENV, tmp.path());
    let ssb = Ssb::new(cfg(&db_path, session), os_clock()).expect("migrated ssb new (gate off)");

    let file_writes = ssb
        .query_by_type(SsbActionType::FileWrite)
        .expect("query file_write");
    assert!(
        file_writes.is_empty(),
        "DROP must remove stale rows even with the archive gate disabled"
    );
    drop(ssb);
    assert_eq!(
        live_user_version(&db_path),
        elai_orchestrator::ssb::SCHEMA_VERSION_PUBLIC,
        "live db recreated at current SCHEMA_VERSION regardless of gate"
    );

    // No archive when disabled.
    let archive = tmp
        .path()
        .join("workflow")
        .join("sessions")
        .join(session)
        .join("archive")
        .join(format!("ssb-v{stale}.sqlite"));
    assert!(
        !archive.exists(),
        "gate OFF: no archive file should be written"
    );
    // CWD restored by EnvScope::drop while ENV_LOCK is still held.
}
