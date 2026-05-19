//! C-17 (DRIFT-F closure): basic FileWatcherService.
//!
//! Closes the B-31 sub-phase 10 PARTIAL — `LockScope::Watch` was declared
//! at `lock.rs:266-273` with the literal comment "Unused in Cycle B" and no
//! consumer existed. This service is the first production consumer: it
//! acquires an `IndexLock::try_acquire(.., LockScope::Watch, ..)` guard on
//! `start` and releases it on `stop` (Drop).
//!
//! Lifecycle
//! ---------
//! `FileWatcherService::start(project_root, clock)` attempts to acquire the
//! Watch-scope lock and returns a `Self` on success. The lock is released
//! when the returned value is dropped (the contained `IndexLockGuard`
//! handles file unlink + kernel-lock release).
//!
//! The current implementation is intentionally minimal — no inotify /
//! ReadDirectoryChangesW polling yet. The contract is the lock acquire +
//! release lifecycle plus a stable `is_running()` accessor for diagnostics.
//! Future revisions (C-18+) can layer real change notification on top of
//! this guard without disturbing the lock-acquire seam pinned by the
//! contract test (`crates/contracts/tests/c17_indexing_resilience_contract.rs`).
//!
//! Wiring
//! ------
//! `startup::auto_resume_indexed_projects` starts a watcher per resumed /
//! completed project at `startup.rs:318` (post-resume), so a long-lived
//! `elai run` retains an active Watch-scope lock for the indexed project.
//! The shutdown path inherits the existing `force_release` loop at
//! `startup.rs:620` which already iterates both `LockScope::Index` and
//! `LockScope::Watch`.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use elai_core::ClockProvider;

use crate::lock::{IndexLock, IndexLockError, IndexLockGuard, LockScope};

/// Per-project file watcher lifecycle. Holds a `LockScope::Watch`
/// `IndexLockGuard` for the lifetime of the value; dropping the value
/// releases the lock.
#[derive(Debug)]
pub struct FileWatcherService {
    project_root: PathBuf,
    guard: IndexLockGuard,
}

impl FileWatcherService {
    /// Acquire the Watch-scope lock for `project_root`. Returns `Err` when
    /// another live process already holds it.
    ///
    /// Errors map 1:1 from `IndexLock::try_acquire`:
    ///   * `IndexLockError::AlreadyHeld` — another process is watching.
    ///   * `IndexLockError::Io` — filesystem or fd-lock plumbing failure.
    ///   * `IndexLockError::StaleReclaimFailed` — could not reclaim a dead
    ///     holder's lock file.
    ///   * `IndexLockError::Clock` — clock provider returned an error.
    pub fn start(
        project_root: &Path,
        clock: Arc<dyn ClockProvider>,
    ) -> Result<Self, IndexLockError> {
        let guard = IndexLock::try_acquire(project_root, LockScope::Watch, clock)?;
        tracing::info!(
            project_root = %project_root.display(),
            holder_pid = guard.holder_pid(),
            "FileWatcherService: Watch-scope lock acquired"
        );
        Ok(Self {
            project_root: project_root.to_path_buf(),
            guard,
        })
    }

    /// Project root this service is watching.
    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    /// PID currently holding the Watch-scope lock (i.e. this process).
    pub fn holder_pid(&self) -> u32 {
        self.guard.holder_pid()
    }

    /// Always `true` while the service is alive. Returns `false` only after
    /// `Drop`; the watcher is dropped before this could be observed, so the
    /// method is provided for symmetry with future async watcher variants
    /// that may have explicit `stop` semantics.
    pub fn is_running(&self) -> bool {
        true
    }

    /// Explicit shutdown — drops the guard, releasing the OS-level lock.
    /// Equivalent to letting the value go out of scope.
    pub fn stop(self) {
        // Dropping `self` runs `IndexLockGuard::drop` which releases the
        // file lock and unlinks the lock file. The explicit `drop(self)`
        // makes the intent unambiguous at call sites.
        drop(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use elai_core::OsClock;
    use std::sync::Arc;
    use tempfile::TempDir;

    #[test]
    fn watcher_acquires_and_releases_watch_lock() {
        let dir = TempDir::new().expect("tempdir");
        let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
        let svc = FileWatcherService::start(dir.path(), Arc::clone(&clock))
            .expect("watcher start");
        assert!(svc.is_running());
        // While the watcher is alive, a second acquire on the same scope
        // from THIS process is allowed by fd-lock's reentrant semantics
        // on some platforms, so we only assert pid/scope here.
        assert_eq!(svc.holder_pid(), std::process::id());
        svc.stop();

        // After stop, a fresh acquire must succeed.
        let svc2 = FileWatcherService::start(dir.path(), clock)
            .expect("watcher restart");
        assert!(svc2.is_running());
    }
}
