//! B-31 Phase 3 (Cycle B) integration tests for the cross-process IndexLock
//! primitive (see `crates/orchestrator/src/lock.rs`).
//!
//! Test list (per DECISION-19 in workflow/reasoning/B-31-decisions.md):
//!   1. test_exclusive_acquire_blocks_second_acquire
//!   2. test_drop_releases_lock_for_subsequent_acquire
//!   3. test_stale_lock_reclaim_when_pid_dead
//!   4. test_lockscope_index_and_watch_coexist
//!   5. test_with_index_lock_propagates_callback_value
//!   6. test_force_release_removes_lock_file

use std::path::Path;
use std::sync::Arc;

use elai_core::{ClockProvider, OsClock};
use elai_orchestrator::lock::{
    self, IndexLock, IndexLockError, LockScope, LOCKS_DIR_NAME,
};

fn os_clock() -> Arc<dyn ClockProvider> {
    Arc::new(OsClock)
}

fn lock_path(project_root: &Path, scope: LockScope) -> std::path::PathBuf {
    project_root
        .join(elai_core::paths::ELAI_DATA_DIR_NAME)
        .join(LOCKS_DIR_NAME)
        .join(scope.filename())
}

#[test]
fn test_exclusive_acquire_blocks_second_acquire() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let first =
        IndexLock::try_acquire(tmp.path(), LockScope::Index, os_clock()).expect("first acquire");

    // Second acquire must fail with AlreadyHeld carrying our PID.
    let result = IndexLock::try_acquire(tmp.path(), LockScope::Index, os_clock());
    match result {
        Err(IndexLockError::AlreadyHeld { holder_pid }) => {
            assert_eq!(holder_pid, std::process::id());
        }
        Ok(_) => panic!("second acquire should not succeed"),
        Err(other) => panic!("expected AlreadyHeld, got {other:?}"),
    }

    drop(first);
}

#[test]
fn test_drop_releases_lock_for_subsequent_acquire() {
    let tmp = tempfile::tempdir().expect("tempdir");
    {
        let _g =
            IndexLock::try_acquire(tmp.path(), LockScope::Index, os_clock()).expect("acquire");
    }
    // First guard out of scope — second acquire must succeed.
    let _g2 = IndexLock::try_acquire(tmp.path(), LockScope::Index, os_clock())
        .expect("second acquire after drop");
}

#[test]
fn test_stale_lock_reclaim_when_pid_dead() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = lock_path(tmp.path(), LockScope::Index);
    // Ensure parent dirs exist before we write the stale body.
    std::fs::create_dir_all(path.parent().unwrap()).expect("create dirs");

    // Hand-write a stale lock body with a guaranteed-dead PID.
    // PID 0 / 1 are reserved; PID 0 is treated as dead by both POSIX and
    // Windows implementations of pid_alive.
    let body = "pid=0\nstarted_at=1700000000\nscope=index\n";
    std::fs::write(&path, body).expect("write stale body");

    // Now acquire — the stale body's PID is dead so we should reclaim and
    // succeed on the first user call.
    let g = IndexLock::try_acquire(tmp.path(), LockScope::Index, os_clock())
        .expect("acquire after stale reclaim");

    // Our PID is in the body now.
    let info = lock::read_lock_holder(tmp.path(), LockScope::Index).expect("read holder");
    assert_eq!(info.pid, std::process::id());

    drop(g);
}

#[test]
fn test_lockscope_index_and_watch_coexist() {
    let tmp = tempfile::tempdir().expect("tempdir");

    let index_guard =
        IndexLock::try_acquire(tmp.path(), LockScope::Index, os_clock()).expect("acquire index");
    let watch_guard =
        IndexLock::try_acquire(tmp.path(), LockScope::Watch, os_clock()).expect("acquire watch");

    // Distinct file paths.
    assert_ne!(index_guard.lock_file(), watch_guard.lock_file());
    // Both files exist on disk.
    assert!(index_guard.lock_file().exists());
    assert!(watch_guard.lock_file().exists());

    drop(index_guard);
    drop(watch_guard);
}

#[test]
fn test_with_index_lock_propagates_callback_value() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let value = lock::with_index_lock(
        tmp.path(),
        LockScope::Index,
        os_clock(),
        |_g| 7_u64 * 6,
    )
    .expect("with_index_lock");
    assert_eq!(value, 42);

    // Released — a re-acquire must succeed.
    let _g =
        IndexLock::try_acquire(tmp.path(), LockScope::Index, os_clock()).expect("re-acquire");
}

#[test]
fn test_force_release_removes_lock_file() {
    let tmp = tempfile::tempdir().expect("tempdir");
    // Write a synthetic foreign body (PID 1 to look like a live external
    // process so a normal try_acquire would fail).
    let path = lock_path(tmp.path(), LockScope::Index);
    std::fs::create_dir_all(path.parent().unwrap()).expect("create dirs");
    let body = "pid=1\nstarted_at=1700000000\nscope=index\n";
    std::fs::write(&path, body).expect("write");

    IndexLock::force_release(tmp.path(), LockScope::Index).expect("force_release");
    assert!(!path.exists(), "force_release must remove the lock file");

    // Idempotent — calling again on a missing file is Ok.
    IndexLock::force_release(tmp.path(), LockScope::Index).expect("force_release idempotent");
}
