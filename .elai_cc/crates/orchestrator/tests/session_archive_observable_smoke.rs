//! F-13 — observable-effect smoke for session-archive-on-schema-bump.
//!
//! Implements the WIRING_CONTRACT.observable_effect declared in
//! docs/thefinalglance/F-plan/F-13_session_archive_schema_bump.md:
//!
//!   trigger:  open an SSB whose on-disk user_version < current SCHEMA_VERSION
//!             with ELAI_SSB_ARCHIVE_ON_BUMP_ENABLE=1 set; the migration
//!             archives the stale DB then DROPs + recreates.
//!   expect:   workflow/sessions/{id}/archive/ssb-v{N}.sqlite EXISTS and is a
//!             valid SQLite file (its blake3 prefix == manifest_blake3 in the
//!             session_archived row); the live ssb.db is freshly recreated at
//!             the new SCHEMA_VERSION (the DROP still happened).
//!
//! This drives the REAL `run_migration` path through `Ssb::new` (NOT a mock).
//! `Ssb::new` spawns a single-writer tokio task on construction, so the test
//! runs inside a multi-thread tokio runtime — a `cargo check` or pure-fn test
//! would mask the "no reactor running" panic. The archive row is emitted via a
//! direct synchronous INSERT inside `run_migration` (the async writer does not
//! exist yet at migration time); this smoke proves that emit actually lands.
//!
//! This is the W-5 ON-path smoke referenced by the wire-dark-allowlist.toml row
//! for ELAI_SSB_ARCHIVE_ON_BUMP_ENABLE.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use elai_orchestrator::session_archive::{archive_manifest_digest, ARCHIVE_ON_BUMP_ENV};
use elai_orchestrator::ssb::{Ssb, SsbActionType, SsbConfig, SsbPayload, SsbReader};
use tempfile::TempDir;

// ENV_LOCK serialises BOTH the `ARCHIVE_ON_BUMP_ENV` var AND the process-global
// CWD: EnvScope chdir's AFTER acquiring the guard and restores the prior CWD in
// `Drop` (while `_guard`, the last field, still holds the lock). A single chdir
// test cannot race itself, but folding the chdir into the guard keeps the
// invariant ("no set_current_dir outside the held lock") consistent across all
// three F-13 test files, so a future second test in this binary is race-safe.
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
        project_id: "f13-smoke".into(),
        session_id: session.into(),
    }
}

fn os_clock() -> Arc<dyn elai_core::clock::ClockProvider> {
    Arc::new(elai_core::clock::OsClock)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stale_open_with_gate_enabled_archives_and_emits() {
    let tmp = TempDir::new().expect("tempdir");
    let db_path = tmp.path().join("ssb.db");
    let session = "f13-smoke-session";
    let stale = 205_i64;

    // --- Seed a stale-version SSB with one real row to be archived ---
    {
        let ssb = Ssb::new(cfg(&db_path, session), os_clock()).expect("seed ssb new");
        ssb.write_payload(
            "seed-agent",
            0,
            SsbPayload::FileWrite(elai_orchestrator::ssb::FileWritePayload {
                path: "old_journal.rs".into(),
                size_bytes: 42,
                hash: "seed".into(),
            }),
            vec!["old_journal.rs".into()],
        )
        .await
        .expect("seed write");
        let mut ssb = ssb;
        ssb.shutdown();
    }
    {
        let conn = rusqlite::Connection::open(&db_path).expect("reopen seeded db");
        conn.pragma_update(None, "user_version", stale)
            .expect("stamp stale user_version");
    }

    // --- Trigger: re-open with the gate ENABLED, from the tempdir CWD ---
    // EnvScope chdir's to the tempdir AFTER taking ENV_LOCK, serialising the CWD
    // window with the env-var window.
    let _env = EnvScope::set_truthy(ARCHIVE_ON_BUMP_ENV, tmp.path());

    let ssb = Ssb::new(cfg(&db_path, session), os_clock()).expect("migrated ssb new");

    // --- Assert: archive file exists, valid SQLite, blake3 matches the row ---
    let archive = tmp
        .path()
        .join("workflow")
        .join("sessions")
        .join(session)
        .join("archive")
        .join(format!("ssb-v{stale}.sqlite"));
    assert!(
        archive.exists(),
        "archive file must exist at {}",
        archive.display()
    );
    let archived = rusqlite::Connection::open(&archive).expect("archive opens as sqlite");
    // The archived (pre-DROP) DB still carries the seeded FileWrite row.
    let archived_count: i64 = archived
        .query_row(
            "SELECT COUNT(*) FROM ssb WHERE action_type = 'file_write'",
            [],
            |r| r.get(0),
        )
        .expect("count archived file_write rows");
    assert_eq!(
        archived_count, 1,
        "the archive must preserve the pre-DROP journal row"
    );
    drop(archived);

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
        .expect("a session_archived (ssb store_kind) row must be emitted by the migration");
    assert_eq!(
        ssb_row.manifest_blake3, digest,
        "the emitted row's manifest_blake3 must equal the archived file's blake3 prefix"
    );
    assert_eq!(ssb_row.stale_version, stale);
    assert_eq!(ssb_row.session_id, session);

    // --- Assert: the LIVE db was freshly recreated (DROP still ran) ---
    let file_writes = ssb
        .query_by_type(SsbActionType::FileWrite)
        .expect("query live file_write");
    assert!(
        file_writes.is_empty(),
        "the live db must be freshly recreated (the DROP still executed)"
    );
    drop(ssb);
    let live_version: i64 = {
        let conn = rusqlite::Connection::open(&db_path).expect("open live db");
        conn.pragma_query_value(None, "user_version", |r| r.get(0))
            .expect("read live user_version")
    };
    assert_eq!(
        live_version,
        elai_orchestrator::ssb::SCHEMA_VERSION_PUBLIC,
        "live db must be recreated at the current SCHEMA_VERSION"
    );
    // CWD restored by EnvScope::drop while ENV_LOCK is still held.
}
