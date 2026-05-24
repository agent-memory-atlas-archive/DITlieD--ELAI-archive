//! E-44 — Cross-process file lock for the PreCompact + 85% token
//! compact race.
//!
//! Both the PreCompact hook (auto-saves FSM state to SESSION_HANDOFF
//! per CLAUDE.md) and the 85% token compact path
//! (`SessionDag::create_handoff` triggered via `hook_dispatcher`) write
//! the same SESSION_HANDOFF artifact. Without a lock, concurrent
//! invocation can interleave writes and leave SSB in an inconsistent
//! state.
//!
//! Backing primitive: `fd_lock` (already a workspace dep at
//! `orchestrator/Cargo.toml:104`, version 4) which wraps `flock()` on
//! POSIX and `LockFileEx` on Windows. No new external dep is required.
//!
//! Anti-stale-training: `fs2` was the historical choice (last
//! published 2020) — the workspace already migrated to `fd-lock` for
//! B-31 Phase 3 (`crates/orchestrator/src/lock.rs:50`). This module
//! reuses the same primitive + the same `Box::into_raw` self-reference
//! pattern documented at `lock.rs:368-414`.

use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};

use fd_lock::{RwLock as FdRwLock, RwLockWriteGuard};
use thiserror::Error;

/// Errors raised by [`CompactLock::try_acquire`].
#[derive(Debug, Error)]
pub enum CompactLockError {
    /// Lock file could not be created or opened. Common cause: the
    /// parent directory does not exist or is not writable.
    #[error("compact lock open failed at {path}: {source}")]
    OpenFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Another process holds the lock. The caller should either wait
    /// + retry or emit an `SsbActionType::DurabilityLockBlocked` row
    /// and bail.
    #[error("compact lock contended at {path}")]
    Contended { path: PathBuf },
}

/// A cross-process advisory file lock at a given path. Acquisition is
/// non-blocking — call `try_acquire` and inspect the result; the
/// caller decides whether to retry or bail.
pub struct CompactLock {
    path: PathBuf,
}

impl CompactLock {
    /// Construct a lock primitive at the given path. The file is
    /// created at acquire time, not at construction.
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Return the path this lock guards.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Attempt to acquire the exclusive write lock. Returns a
    /// [`LockGuard`] on success; the OS-level lock is released when
    /// the guard is dropped.
    ///
    /// Non-blocking: if another process holds the lock, returns
    /// `CompactLockError::Contended` immediately. This is the
    /// deliberate contract — the caller (PreCompact hook + 85%
    /// compact path) decides whether to wait, retry, or emit
    /// `SsbActionType::DurabilityLockBlocked`.
    pub fn try_acquire(&self) -> Result<LockGuard, CompactLockError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| CompactLockError::OpenFailed {
                path: self.path.clone(),
                source: e,
            })?;
        }
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&self.path)
            .map_err(|e| CompactLockError::OpenFailed {
                path: self.path.clone(),
                source: e,
            })?;

        // Same Box::into_raw self-reference pattern as
        // `orchestrator::lock::try_acquire` (lock.rs:677-697). The
        // FdRwLock<File> is leaked to the heap so the borrow of its
        // write guard can outlive the function frame; the raw
        // pointer is reclaimed in `OwnedGuard::drop` after the write
        // guard releases the OS-level lock.
        let rwlock_box: Box<FdRwLock<File>> = Box::new(FdRwLock::new(file));
        let rwlock_ptr: *mut FdRwLock<File> = Box::into_raw(rwlock_box);
        // SAFETY: pointer just produced by Box::into_raw is exclusively owned.
        let rwlock_static: &'static mut FdRwLock<File> = unsafe { &mut *rwlock_ptr };

        let fd_guard = match rwlock_static.try_write() {
            Ok(g) => g,
            Err(_) => {
                // Reclaim the heap allocation BEFORE returning so we
                // do not leak. SAFETY: rwlock_ptr was produced by
                // Box::into_raw above and is unreferenced after this
                // point.
                unsafe { drop(Box::from_raw(rwlock_ptr)) };
                return Err(CompactLockError::Contended {
                    path: self.path.clone(),
                });
            }
        };

        Ok(LockGuard {
            path: self.path.clone(),
            owned: Some(OwnedGuard {
                fd_guard,
                rwlock_ptr,
            }),
        })
    }
}

/// Owned bundle holding the write guard plus the raw pointer to the
/// leaked `FdRwLock<File>`. Drop order is field-declaration order:
/// `fd_guard` drops first (releasing the OS lock), then `rwlock_ptr`
/// is reclaimed.
#[derive(Debug)]
struct OwnedGuard {
    /// fd-lock write guard. Drop releases the kernel-level advisory
    /// lock. Field is "never read" from the compiler's perspective —
    /// only its Drop side effect matters.
    #[allow(dead_code)]
    fd_guard: RwLockWriteGuard<'static, File>,
    /// Raw pointer back to the leaked `FdRwLock<File>`. Reclaimed as
    /// a `Box` in `Drop` once `fd_guard` is gone.
    rwlock_ptr: *mut FdRwLock<File>,
}

// SAFETY: the raw pointer is uniquely owned by this guard and only
// dereferenced in our own Drop after the write guard releases. The
// inner File is Send + Sync. Mirrors the rationale in
// `lock.rs:OwnedGuard` Send/Sync impls.
unsafe impl Send for OwnedGuard {}
unsafe impl Sync for OwnedGuard {}

impl Drop for OwnedGuard {
    fn drop(&mut self) {
        // fd_guard is dropped by field-declaration order BEFORE this
        // body runs (releasing the OS lock + closing the File). Here
        // we only reclaim the heap allocation.
        //
        // SAFETY: rwlock_ptr was produced by Box::into_raw in
        // `CompactLock::try_acquire` and has not been freed elsewhere.
        unsafe { drop(Box::from_raw(self.rwlock_ptr)) };
    }
}

/// RAII guard for an acquired [`CompactLock`]. Drop releases the
/// OS-level lock and deletes the lock file.
#[derive(Debug)]
pub struct LockGuard {
    path: PathBuf,
    owned: Option<OwnedGuard>,
}

impl LockGuard {
    /// Path of the lock file this guard owns.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        // 1. Drop the fd-lock guard FIRST (releases OS lock + closes file).
        drop(self.owned.take());
        // 2. Best-effort unlink. Never propagate from Drop.
        let _ = std::fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn acquire_in_fresh_dir_succeeds() {
        let dir = tempdir().expect("tempdir");
        let lock = CompactLock::new(dir.path().join(".compact-lock"));
        let _guard = lock.try_acquire().expect("acquire ok");
    }

    #[test]
    fn release_via_drop_allows_reacquire() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join(".compact-lock");
        {
            let lock = CompactLock::new(path.clone());
            let _g = lock.try_acquire().expect("first acquire");
        }
        let lock2 = CompactLock::new(path);
        let _g2 = lock2.try_acquire().expect("reacquire after drop ok");
    }

    #[test]
    fn path_accessor_round_trips_constructor_input() {
        let dir = tempdir().expect("tempdir");
        let p = dir.path().join(".compact-lock");
        let lock = CompactLock::new(p.clone());
        assert_eq!(lock.path(), p.as_path());
    }

    #[test]
    fn open_failure_when_parent_unwritable() {
        // Use a Windows-invalid path so OpenOptions errors. The exact
        // OS error varies but the error variant must be OpenFailed.
        #[cfg(windows)]
        let path = PathBuf::from("CON:\\nope\\.compact-lock");
        #[cfg(unix)]
        let path = PathBuf::from("/dev/null/cannot-be-a-dir/.compact-lock");
        let lock = CompactLock::new(path);
        let err = lock.try_acquire().expect_err("must fail to open");
        assert!(matches!(err, CompactLockError::OpenFailed { .. }));
    }
}
