//! B-31 Phase 3 (Cycle B): cross-process IndexLock primitive.
//!
//! Two OS processes must never run resumable indexing on the same project
//! concurrently — partial checkpoints from one would race the other's
//! file-hash writes through SQLite's WAL. This module provides
//! [`IndexLock::try_acquire`] which obtains an exclusive cross-process
//! advisory lock backed by [`fd_lock::RwLock`] (POSIX `flock()` / Windows
//! `LockFileEx` via `windows-sys`).
//!
//! Stale-lock reclamation
//! ----------------------
//! The lock file body holds the holder's PID. On contention we read the PID
//! and call [`pid_alive`]:
//! * Unix: `libc::kill(pid, 0)` — `ESRCH` means dead.
//! * Windows: `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid)`
//!   returns a null handle for non-existent PIDs.
//!
//! If the PID is dead, the lock file is deleted and acquisition retried
//! exactly once. Stale-reclamation is explicit-only: we never kill a live
//! holder (mirrors SocratiCode's `terminateLockHolder` design choice in
//! Cycle B — see `workflow/reasoning/B-31-decisions.md` DECISION-14).
//!
//! File layout
//! -----------
//! Lock files live alongside the per-project metadata DB:
//! ```text
//! <project_root>/.elai_cc/indexes/<scope>.lock
//! ```
//! where `<scope>` is `index` or `watch`. Each file contains:
//! ```text
//! pid=<u32>
//! started_at=<unix_secs>
//! scope=<index|watch>
//! ```
//! (UTF-8, newline-terminated).
//!
//! Drop semantics
//! --------------
//! [`IndexLockGuard`] is RAII: drop releases the fd-lock guard AND deletes
//! the lock file. Drop is panic-safe — unlink failures are logged via
//! `tracing::warn!` and swallowed (we never panic in `Drop`).

use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use elai_core::ClockProvider;
use fd_lock::{RwLock as FdRwLock, RwLockWriteGuard};

use crate::ssb::{LockAcquiredPayload, LockContestedPayload, Ssb, SsbPayload};

// ---------------------------------------------------------------------------
// Named constants (no magic numbers)
// ---------------------------------------------------------------------------

/// Directory under `<project_root>/.elai_cc/` where index + watch lock files
/// live. Mirrors `INDEXES_DIR_NAME` from `elai_core::paths` (the per-project
/// metadata DB and dkg DB share the same parent).
pub const LOCKS_DIR_NAME: &str = "indexes";

/// Lock file body field keys. Stable across versions so a foreign process
/// written by a future build can still be parsed for diagnostics.
const PID_FIELD: &str = "pid";
const STARTED_AT_FIELD: &str = "started_at";
const SCOPE_FIELD: &str = "scope";

/// Minimum on-disk size of the lock file body. The body is padded with
/// trailing spaces up to this length so a new (shorter) acquirer cannot
/// leave bytes from a longer prior body visible past their own content.
/// 256 bytes is more than enough for `pid=<u32>\nstarted_at=<i64>\nscope=watch\n`
/// (~42 chars worst-case) yet small enough to be invisible.
const LOCK_BODY_PAD_BYTES: usize = 256;

// ---------------------------------------------------------------------------
// In-process held-locks registry
// ---------------------------------------------------------------------------
//
// Windows LockFileEx is per-handle within a single process: two `File`
// handles in the same process can each acquire an exclusive byte-range lock
// without the kernel reporting contention. To preserve "exclusive single-
// acquirer" semantics inside the same process, we maintain an in-memory set
// of (canonicalised lock path) strings keyed on the lock files we currently
// hold. Mirrors `proper-lockfile`'s `heldLocks` Map in the SocratiCode JS
// reference (lock.ts:36).

fn held_locks() -> &'static Mutex<HashSet<PathBuf>> {
    static HELD: OnceLock<Mutex<HashSet<PathBuf>>> = OnceLock::new();
    HELD.get_or_init(|| Mutex::new(HashSet::new()))
}

fn held_locks_insert(path: &Path) -> bool {
    let mut g = match held_locks().lock() {
        Ok(g) => g,
        Err(poison) => poison.into_inner(),
    };
    g.insert(path.to_path_buf())
}

fn held_locks_remove(path: &Path) {
    if let Ok(mut g) = held_locks().lock() {
        g.remove(path);
    } else if let Err(poison) = held_locks().lock() {
        poison.into_inner().remove(path);
    }
}

fn held_locks_contains(path: &Path) -> bool {
    match held_locks().lock() {
        Ok(g) => g.contains(path),
        Err(poison) => poison.into_inner().contains(path),
    }
}

// ---------------------------------------------------------------------------
// B-31.5: optional process-global SSB writer for LockAcquired / LockContested
// ---------------------------------------------------------------------------
//
// We avoid breaking the existing `try_acquire(project_root, scope, clock)`
// signature (live callers: `startup::auto_resume_indexed_projects`, the
// dashboard `cancel_indexing` plumbing, the integration test in
// `tests/index_lock.rs`). Instead we expose a process-global slot the
// dashboard / CLI bootstrap can set ONCE at startup; when set, the
// IndexLock paths emit LockAcquired / LockContested SSB rows. When unset
// (every contract test, cold start before SSB init, etc.) the paths are
// pure no-ops.

fn ssb_writer_slot() -> &'static Mutex<Option<Arc<Ssb>>> {
    static SLOT: OnceLock<Mutex<Option<Arc<Ssb>>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

/// B-31.5: install the process-global SSB writer used by `IndexLock::try_acquire`
/// to emit [`crate::ssb::SsbActionType::LockAcquired`] and
/// [`crate::ssb::SsbActionType::LockContested`] rows.
///
/// Intended caller is the dashboard / CLI bootstrap layer that already holds
/// the per-project [`Ssb`]. Idempotent: replacing an existing writer is
/// allowed (e.g. when the dashboard rebinds the project).
pub fn install_ssb_writer(writer: Arc<Ssb>) {
    let mut g = match ssb_writer_slot().lock() {
        Ok(g) => g,
        Err(poison) => poison.into_inner(),
    };
    *g = Some(writer);
}

/// B-31.5: clear the process-global SSB writer. Used by tests and shutdown.
#[cfg(test)]
pub(crate) fn clear_ssb_writer_for_test() {
    let mut g = match ssb_writer_slot().lock() {
        Ok(g) => g,
        Err(poison) => poison.into_inner(),
    };
    *g = None;
}

fn current_ssb_writer() -> Option<Arc<Ssb>> {
    let g = match ssb_writer_slot().lock() {
        Ok(g) => g,
        Err(poison) => poison.into_inner(),
    };
    g.as_ref().map(Arc::clone)
}

/// J-34 slice-2 fix: pub(crate) accessor over [`current_ssb_writer`] so the
/// calibration store's `insert()` can thread the process-global SSB writer
/// (installed by `runtime_setup` via [`install_ssb_writer`]) without a caller
/// having to pass it explicitly. Same semantics as `current_ssb_writer`:
/// `Some` only when the dashboard / CLI bootstrap has installed a writer;
/// `None` in cold-start / contract-test paths. Does NOT mutate the slot.
pub(crate) fn try_ssb_writer() -> Option<Arc<Ssb>> {
    current_ssb_writer()
}

/// B-31.5: derive the project_id slug for the SSB rows. Mirrors
/// `elai_knowledge::metadata::project_id_from_path` — kept in sync via the
/// re-export at `startup::project_id_from_path`. We do NOT take a hard
/// dependency on the knowledge crate's helper from this module to avoid a
/// cycle through SSB (knowledge → orchestrator::lock would be fine, but
/// keeping this lightweight is cleaner).
fn project_id_for_lock(project_root: &Path) -> String {
    elai_knowledge::metadata::project_id_from_path(project_root)
}

/// Best-effort SSB emit. Logs and swallows on failure (observability
/// scaffolding, never load-bearing). Mirrors the
/// `indexing_service::emit_ssb_payload` async-bridge pattern.
fn emit_lock_ssb(payload: SsbPayload) {
    let Some(writer) = current_ssb_writer() else {
        tracing::debug!(
            action_type = payload.action_type().as_sql_str(),
            "B-31.5: lock SSB writer absent — payload dropped",
        );
        return;
    };
    let action_label = payload.action_type().as_sql_str();
    let drive = move |writer: Arc<Ssb>, payload: SsbPayload| async move {
        if let Err(e) = writer
            .write_payload("index_lock", 0, payload, Vec::new())
            .await
        {
            tracing::warn!(
                target = "index_lock",
                action_type = action_label,
                error = %e,
                "B-31.5: SSB write_payload failed",
            );
        }
    };
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => match handle.runtime_flavor() {
            tokio::runtime::RuntimeFlavor::MultiThread => {
                tokio::task::block_in_place(|| {
                    handle.block_on(drive(writer, payload));
                });
            }
            _ => {
                let join = std::thread::spawn(move || {
                    let rt = match tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                    {
                        Ok(rt) => rt,
                        Err(e) => {
                            tracing::warn!(
                                target = "index_lock",
                                error = %e,
                                "B-31.5: failed to build transient runtime for SSB emit",
                            );
                            return;
                        }
                    };
                    rt.block_on(drive(writer, payload));
                });
                if let Err(_panic) = join.join() {
                    tracing::warn!(
                        target = "index_lock",
                        "B-31.5: lock-emit worker thread panicked",
                    );
                }
            }
        },
        Err(_) => {
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    tracing::warn!(
                        target = "index_lock",
                        error = %e,
                        "B-31.5: failed to build transient runtime for SSB emit",
                    );
                    return;
                }
            };
            rt.block_on(drive(writer, payload));
        }
    }
}

// ---------------------------------------------------------------------------
// LockScope
// ---------------------------------------------------------------------------

/// Distinct lock surfaces a project may hold concurrently.
///
/// `Index` is taken by `IndexingService::build_index_resumable` (DECISION-13).
/// `Watch` is reserved for the future file-watcher service (B-31 sub-phase 10).
/// Both scopes coexist on a single project because they have distinct lock
/// filenames.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LockScope {
    /// Held while resumable indexing runs.
    Index,
    /// Held while a file watcher is active. Unused in Cycle B; included for
    /// API parity with the SocratiCode reference and the future watcher
    /// integration.
    Watch,
}

impl LockScope {
    /// Lowercase string representation used in the lock file body's `scope=`
    /// field and as the filename stem.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Index => "index",
            Self::Watch => "watch",
        }
    }

    /// Filename for this scope, including the `.lock` extension.
    /// E.g. `Index` → `"index.lock"`.
    pub fn filename(self) -> String {
        format!("{}.lock", self.as_str())
    }
}

impl std::fmt::Display for LockScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// LockHolderInfo
// ---------------------------------------------------------------------------

/// Parsed snapshot of a lock file's body. Returned by [`read_lock_holder`]
/// (non-acquiring read used by diagnostics + dashboard `LockContested`
/// reporting).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockHolderInfo {
    /// OS process ID that wrote the lock file body.
    pub pid: u32,
    /// Unix-seconds timestamp from the lock file body.
    pub started_at: i64,
    /// Scope the holder claims (`index` or `watch`).
    pub scope: LockScope,
}

// ---------------------------------------------------------------------------
// IndexLockError
// ---------------------------------------------------------------------------

/// Errors that can arise during lock acquisition / release.
///
/// Send + Sync is required by `crates/contracts/tests/indexing_service_a44_contract.rs`
/// (assert_send_sync). thiserror's auto-impls give us both.
#[derive(Debug, thiserror::Error)]
pub enum IndexLockError {
    /// Another live process holds the lock.
    #[error("index lock already held by pid={holder_pid}")]
    AlreadyHeld { holder_pid: u32 },

    /// Filesystem-level I/O failure (create_dir_all, open, read, write).
    #[error("index lock I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Reclamation of a stale lock failed (e.g. delete returned an error
    /// other than NotFound). Carries the underlying error message.
    #[error("index lock stale reclamation failed: {0}")]
    StaleReclaimFailed(String),

    /// Clock provider reported a pre-epoch time. Vanishingly unlikely.
    #[error("index lock clock error: {0}")]
    Clock(String),
}

// ---------------------------------------------------------------------------
// IndexLockGuard (RAII)
// ---------------------------------------------------------------------------

/// RAII guard returned by [`IndexLock::try_acquire`]. Dropping the guard
/// releases the underlying `fd_lock` write guard AND deletes the lock file.
///
/// Drop is panic-safe: filesystem unlink failures are logged via
/// `tracing::warn!` and never propagate. The `fd_lock` guard release is
/// always attempted via `drop` of the contained `Box<FdRwLock<File>>`.
pub struct IndexLockGuard {
    /// Full path to the on-disk lock file. Deleted in `Drop`.
    lock_file: PathBuf,
    /// `fd_lock` RwLock holding the exclusive write guard. Boxed so we can
    /// `drop` the inner guard explicitly in `Drop` before unlinking, avoiding
    /// a Windows file-still-mapped issue when LockFileEx hasn't released yet.
    ///
    /// The guard is in an `Option` so we can `take()` it during `Drop`
    /// without `unsafe`. Always `Some` between construction and `Drop`.
    guard: Option<OwnedGuard>,
    /// PID this guard holds. Stored for diagnostics.
    holder_pid: u32,
    /// Scope this guard holds.
    scope: LockScope,
}

/// Internal owned bundle: an `FdRwLock<File>` whose `try_write` guard
/// borrows from itself. We wrap both via a self-referencing pattern using
/// `Box::leak`-equivalent ownership via separate fields plus careful drop
/// ordering — the simpler approach is to keep the guard alongside the
/// `RwLock` and rely on Rust's drop order (struct fields drop in declaration
/// order). To avoid the borrow-checker complaining about a self-reference
/// we leak the `RwLock` for the lifetime of the guard, retaining it as a
/// `ManuallyDrop` resource cleaned up after the guard releases.
struct OwnedGuard {
    /// The fd-lock write guard. Drop releases the OS-level lock.
    /// SAFETY: `_rwlock` is leaked (Box::leak / static-equivalent) before
    /// the guard is constructed so the borrow is provably valid for `'static`.
    /// We then reclaim the heap allocation by reading the raw pointer back
    /// into a `Box` in `Drop` AFTER the guard is dropped.
    ///
    /// The field is "never read" from the compiler's perspective — only its
    /// Drop side effect (kernel lock release) matters. The allow is required
    /// because Rust's dead-code lint does not understand Drop-only fields.
    #[allow(dead_code)]
    fd_guard: RwLockWriteGuard<'static, File>,
    /// Raw pointer back to the leaked `FdRwLock<File>`. Reclaimed as a `Box`
    /// in `Drop` once `fd_guard` is gone.
    rwlock_ptr: *mut FdRwLock<File>,
}

// SAFETY: `RwLockWriteGuard` is Send + Sync when the inner `File` is, which
// it is. The raw pointer is only used in our own `Drop` to reclaim the
// allocation after the guard is dropped — it is never dereferenced
// concurrently. We can therefore mark the bundle Send + Sync explicitly.
unsafe impl Send for OwnedGuard {}
unsafe impl Sync for OwnedGuard {}

impl Drop for OwnedGuard {
    fn drop(&mut self) {
        // The compiler will drop `fd_guard` (releasing the OS lock) before
        // we hit this body, because struct fields are dropped in declaration
        // order. Here we just reclaim the heap allocation that backed the
        // `RwLock<File>` so the `File` itself is closed.
        //
        // SAFETY: `rwlock_ptr` was produced by `Box::into_raw` in
        // `IndexLock::try_acquire` and has not been freed elsewhere.
        unsafe {
            drop(Box::from_raw(self.rwlock_ptr));
        }
    }
}

impl IndexLockGuard {
    /// PID we wrote into the lock file when acquiring.
    pub fn holder_pid(&self) -> u32 {
        self.holder_pid
    }

    /// Scope this guard owns.
    pub fn scope(&self) -> LockScope {
        self.scope
    }

    /// Absolute path of the on-disk lock file.
    pub fn lock_file(&self) -> &Path {
        &self.lock_file
    }
}

impl std::fmt::Debug for IndexLockGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IndexLockGuard")
            .field("lock_file", &self.lock_file)
            .field("holder_pid", &self.holder_pid)
            .field("scope", &self.scope)
            .finish()
    }
}

impl Drop for IndexLockGuard {
    fn drop(&mut self) {
        // 1. Drop the fd-lock guard FIRST — this releases the kernel-level
        //    advisory lock and closes the underlying File descriptor.
        drop(self.guard.take());

        // 2. Unregister from the in-process held-locks set BEFORE unlinking
        //    so a follow-up acquire by the same process sees a clean slate
        //    even if remove_file fails.
        held_locks_remove(&self.lock_file);

        // 3. Then unlink the lock file body. Best-effort: log and swallow
        //    any error so we never panic in Drop.
        if let Err(err) = std::fs::remove_file(&self.lock_file) {
            if err.kind() != std::io::ErrorKind::NotFound {
                tracing::warn!(
                    path = %self.lock_file.display(),
                    error = %err,
                    "IndexLockGuard::drop: failed to remove lock file (lock released, file remains)"
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// IndexLock (free functions)
// ---------------------------------------------------------------------------

/// Namespace for cross-process lock operations on a project's index/watch
/// scope. No instance state — all functions are free / associated.
pub struct IndexLock;

impl IndexLock {
    /// Try to acquire an exclusive lock for `scope` on the project at
    /// `project_root`. Returns an [`IndexLockGuard`] that releases the lock
    /// in `Drop`.
    ///
    /// Behaviour on contention:
    /// 1. The lock file is opened with `OpenOptions::read+write+create(true)`.
    /// 2. `fd_lock::try_write()` is invoked.
    /// 3. On `WouldBlock` we read the existing PID from the body and check
    ///    liveness via [`pid_alive`].
    /// 4. If the PID is dead, we delete the lock file and retry exactly once.
    /// 5. After a successful acquisition we overwrite the body with our own
    ///    `pid=...\nstarted_at=...\nscope=...\n` triple.
    ///
    /// Errors:
    /// * [`IndexLockError::AlreadyHeld`] — a live process holds the lock.
    /// * [`IndexLockError::Io`] — filesystem error.
    /// * [`IndexLockError::StaleReclaimFailed`] — could not delete a stale
    ///   lock file after the prior holder was confirmed dead.
    /// * [`IndexLockError::Clock`] — clock provider failed.
    pub fn try_acquire(
        project_root: &Path,
        scope: LockScope,
        clock: Arc<dyn ClockProvider>,
    ) -> Result<IndexLockGuard, IndexLockError> {
        let lock_path = lock_file_path(project_root, scope);
        if let Some(parent) = lock_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // First attempt.
        let outcome = match try_acquire_once(&lock_path, scope, &clock) {
            Ok(guard) => Ok(guard),
            Err(IndexLockError::AlreadyHeld { holder_pid }) => {
                // Liveness check on the recorded PID.
                if pid_alive(holder_pid) {
                    Err(IndexLockError::AlreadyHeld { holder_pid })
                } else {
                    // Stale: reclaim by deleting and retry once.
                    if let Err(err) = std::fs::remove_file(&lock_path) {
                        if err.kind() != std::io::ErrorKind::NotFound {
                            return Err(IndexLockError::StaleReclaimFailed(err.to_string()));
                        }
                    }
                    try_acquire_once(&lock_path, scope, &clock)
                }
            }
            Err(other) => Err(other),
        };

        // B-31.5: emit LockAcquired / LockContested SSB rows when the
        // process-global writer has been installed. No-op otherwise.
        let project_id = project_id_for_lock(project_root);
        match &outcome {
            Ok(guard) => {
                emit_lock_ssb(SsbPayload::LockAcquired(LockAcquiredPayload {
                    project_id,
                    scope: scope.as_str().to_owned(),
                    holder_pid: guard.holder_pid(),
                }));
            }
            Err(IndexLockError::AlreadyHeld { holder_pid }) => {
                emit_lock_ssb(SsbPayload::LockContested(LockContestedPayload {
                    project_id,
                    scope: scope.as_str().to_owned(),
                    holder_pid: *holder_pid,
                }));
            }
            Err(_) => {
                // Other errors (Io / StaleReclaimFailed / Clock) are not
                // contention signals; no SSB row emitted.
            }
        }

        outcome
    }

    /// Non-acquiring liveness probe. Returns `Some(pid)` when another live
    /// process holds the lock, `None` otherwise (file missing, body
    /// unparseable, or holder confirmed dead).
    pub fn is_locked_by_other(project_root: &Path, scope: LockScope) -> Option<u32> {
        let info = read_lock_holder(project_root, scope)?;
        if info.pid == std::process::id() {
            return None;
        }
        if pid_alive(info.pid) {
            Some(info.pid)
        } else {
            None
        }
    }

    /// Force-delete the lock file regardless of holder.
    ///
    /// Used only by `startup::graceful_shutdown` when the deadline has
    /// expired. Never call from normal acquisition paths — always prefer
    /// the RAII guard's `Drop`.
    pub fn force_release(project_root: &Path, scope: LockScope) -> std::io::Result<()> {
        let path = lock_file_path(project_root, scope);
        match std::fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err),
        }
    }
}

// ---------------------------------------------------------------------------
// with_index_lock adapter
// ---------------------------------------------------------------------------

/// Idiomatic acquire-call-release wrapper. Acquires `scope`, calls `f`, and
/// releases via Drop on return. Propagates the callback's return value
/// unchanged.
///
/// Designed as the cleanest call site for `IndexingService::build_index_resumable`:
/// ```ignore
/// orchestrator::lock::with_index_lock(
///     project_root,
///     LockScope::Index,
///     clock.clone(),
///     |_guard| service.build_index_resumable(project_id, store, clock),
/// )?
/// ```
pub fn with_index_lock<F, T>(
    project_root: &Path,
    scope: LockScope,
    clock: Arc<dyn ClockProvider>,
    f: F,
) -> Result<T, IndexLockError>
where
    F: FnOnce(&IndexLockGuard) -> T,
{
    let guard = IndexLock::try_acquire(project_root, scope, clock)?;
    let out = f(&guard);
    Ok(out)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Compute the canonical lock file path for a given project + scope.
fn lock_file_path(project_root: &Path, scope: LockScope) -> PathBuf {
    project_root
        .join(elai_core::paths::ELAI_DATA_DIR_NAME)
        .join(LOCKS_DIR_NAME)
        .join(scope.filename())
}

/// Build the lock file body bytes for the current process.
fn build_lock_body(scope: LockScope, started_at: i64) -> String {
    format!(
        "{PID_FIELD}={pid}\n{STARTED_AT_FIELD}={started}\n{SCOPE_FIELD}={scope}\n",
        pid = std::process::id(),
        started = started_at,
        scope = scope.as_str(),
    )
}

/// Pad `body` to [`LOCK_BODY_PAD_BYTES`] with trailing spaces. Padding bytes
/// look like `pad-space-space-space-...` to a human eye but `parse_lock_body`
/// ignores any line that doesn't match `key=value`, so the padding is
/// semantically transparent.
fn pad_lock_body(body: &str) -> String {
    if body.len() >= LOCK_BODY_PAD_BYTES {
        return body.to_string();
    }
    let mut out = String::with_capacity(LOCK_BODY_PAD_BYTES);
    out.push_str(body);
    let needed = LOCK_BODY_PAD_BYTES - body.len();
    for _ in 0..needed {
        out.push(' ');
    }
    out
}

/// Single acquisition attempt. On contention returns
/// [`IndexLockError::AlreadyHeld`] with the foreign PID; the caller decides
/// whether to retry (stale-reclaim path).
fn try_acquire_once(
    lock_path: &Path,
    scope: LockScope,
    clock: &Arc<dyn ClockProvider>,
) -> Result<IndexLockGuard, IndexLockError> {
    // In-process check FIRST: if this process already holds this lock path,
    // refuse to acquire again with our own PID (Windows LockFileEx would
    // otherwise grant the second lock because LockFileEx is per-handle).
    if held_locks_contains(lock_path) {
        return Err(IndexLockError::AlreadyHeld {
            holder_pid: std::process::id(),
        });
    }

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(lock_path)?;

    // Box::leak gives us a 'static reference; we reclaim the heap allocation
    // in `OwnedGuard::drop` AFTER the fd_guard is dropped so the file is
    // closed and the OS lock released first.
    let rwlock_box: Box<FdRwLock<File>> = Box::new(FdRwLock::new(file));
    let rwlock_ptr: *mut FdRwLock<File> = Box::into_raw(rwlock_box);

    // SAFETY: the pointer is valid (just produced by Box::into_raw), and we
    // hold exclusive ownership of it for the lifetime of OwnedGuard.
    let rwlock_static: &'static mut FdRwLock<File> = unsafe { &mut *rwlock_ptr };

    let mut fd_guard = match rwlock_static.try_write() {
        Ok(g) => g,
        Err(_) => {
            // Reclaim the heap allocation BEFORE returning so we don't leak.
            // SAFETY: rwlock_ptr was produced by Box::into_raw above and is
            // unreferenced after this point.
            unsafe { drop(Box::from_raw(rwlock_ptr)) };
            let holder_pid = read_lock_holder_from_path(lock_path)
                .map(|info| info.pid)
                .unwrap_or(0);
            return Err(IndexLockError::AlreadyHeld { holder_pid });
        }
    };

    // We hold the exclusive lock. Stamp our PID + timestamp into the body
    // by writing through the locked file handle (DerefMut gives us &mut File).
    //
    // Windows quirk: the fd_lock impl on Windows takes a LockFileEx byte-
    // range lock on the first byte of the file. Calling `set_len(0)` on
    // the locked handle truncates the region the lock covers and silently
    // invalidates the lock. To preserve the lock we (a) do NOT truncate,
    // (b) pad the body to LOCK_BODY_PAD_BYTES with spaces so the on-disk
    // body is always at least that long and stale bytes from a prior
    // (longer) holder cannot bleed past our new content. Trailing spaces
    // are stripped by `parse_lock_body` (lines that don't match `key=val`
    // are ignored).
    // Body-write must reclaim the boxed RwLock on EVERY error path; otherwise
    // each transient I/O or clock failure leaks the heap allocation plus the
    // underlying File descriptor (Cycle B audit BLOCK). We compute the body in
    // an inner closure so a single `match` handles all four `?` sources
    // (clock, seek, write_all, flush) with one reclaim site.
    let body_write_result: Result<(), IndexLockError> = (|| {
        use std::io::Seek;
        let started_at = clock_unix_secs(clock)?;
        let body = pad_lock_body(&build_lock_body(scope, started_at));
        let inner_file: &mut File = &mut *fd_guard;
        inner_file.seek(std::io::SeekFrom::Start(0))?;
        inner_file.write_all(body.as_bytes())?;
        inner_file.flush()?;
        Ok(())
    })();
    if let Err(err) = body_write_result {
        // SAFETY: rwlock_ptr is the same pointer Box::into_raw produced
        // above; fd_guard is dropped here before the box, releasing the OS
        // lock first, then the heap allocation.
        drop(fd_guard);
        unsafe { drop(Box::from_raw(rwlock_ptr)) };
        return Err(err);
    }

    let holder_pid = std::process::id();

    // Register the lock in the in-process held-locks set. Drop unregisters.
    let inserted = held_locks_insert(lock_path);
    if !inserted {
        // Race: another thread in this process inserted between our
        // initial check and now. Release everything and report AlreadyHeld.
        drop(fd_guard);
        unsafe { drop(Box::from_raw(rwlock_ptr)) };
        return Err(IndexLockError::AlreadyHeld { holder_pid });
    }

    Ok(IndexLockGuard {
        lock_file: lock_path.to_path_buf(),
        guard: Some(OwnedGuard {
            fd_guard,
            rwlock_ptr,
        }),
        holder_pid,
        scope,
    })
}

/// Read and parse the lock file body for `(project_root, scope)`.
///
/// Public so dashboards / `LockContested` SSB emitters can introspect a
/// foreign holder without trying to acquire.
pub fn read_lock_holder(project_root: &Path, scope: LockScope) -> Option<LockHolderInfo> {
    let path = lock_file_path(project_root, scope);
    // Fast path: if WE hold the lock in this process, return our own info
    // synthesised from the registry — avoids a Windows ERROR_LOCK_VIOLATION
    // on a byte-0 read while the LockFileEx range still covers byte 0.
    if held_locks_contains(&path) {
        return Some(LockHolderInfo {
            pid: std::process::id(),
            // started_at unknown in registry; report 0 so any caller that
            // compares against STALE_CHECKPOINT_TIMEOUT_SECS still treats
            // it as a "recently-acquired" lock. This sentinel is only ever
            // surfaced to other code paths inside the same process.
            started_at: 0,
            scope,
        });
    }
    read_lock_holder_from_path(&path)
}

fn read_lock_holder_from_path(path: &Path) -> Option<LockHolderInfo> {
    let mut buf = String::new();
    let mut file = std::fs::File::open(path).ok()?;
    file.read_to_string(&mut buf).ok()?;
    parse_lock_body(&buf)
}

fn parse_lock_body(body: &str) -> Option<LockHolderInfo> {
    let mut pid: Option<u32> = None;
    let mut started_at: Option<i64> = None;
    let mut scope: Option<LockScope> = None;
    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (key, value) = match line.split_once('=') {
            Some(kv) => kv,
            None => continue,
        };
        match key.trim() {
            PID_FIELD => {
                pid = value.trim().parse::<u32>().ok();
            }
            STARTED_AT_FIELD => {
                started_at = value.trim().parse::<i64>().ok();
            }
            SCOPE_FIELD => {
                scope = match value.trim() {
                    "index" => Some(LockScope::Index),
                    "watch" => Some(LockScope::Watch),
                    _ => None,
                };
            }
            _ => {}
        }
    }
    Some(LockHolderInfo {
        pid: pid?,
        started_at: started_at?,
        scope: scope?,
    })
}

/// Cross-platform PID-alive check.
///
/// Returns `true` when a process with `pid` exists; `false` when it has
/// exited or we cannot tell (conservative — we treat "unknown" as dead so
/// that stale-reclamation can proceed, mirroring SocratiCode lock.ts:194-199).
pub fn pid_alive(pid: u32) -> bool {
    pid_alive_impl(pid)
}

#[cfg(unix)]
fn pid_alive_impl(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }
    // libc::kill(pid, 0) — signal 0 is the "existence check" form.
    // Returns 0 when the process exists (or we lack permission to signal it
    // but the PID is valid — EPERM still means alive).
    // SAFETY: pure FFI to a standards-defined libc function.
    let pid_t = pid as libc::pid_t;
    let rc = unsafe { libc::kill(pid_t, 0) };
    if rc == 0 {
        return true;
    }
    // SAFETY: errno read is the canonical way to disambiguate the result.
    // We treat EPERM as alive (we lack permission but the PID is valid).
    let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
    errno == libc::EPERM
}

#[cfg(windows)]
fn pid_alive_impl(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }
    // OpenProcess with PROCESS_QUERY_LIMITED_INFORMATION (0x1000) is the
    // least-privileged way to check existence. Returns a null handle for
    // non-existent PIDs. SAFETY: pure FFI to a documented Win32 function.
    use windows_sys::Win32::Foundation::{CloseHandle, FALSE};
    use windows_sys::Win32::System::Threading::{
        OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid) };
    if handle.is_null() {
        return false;
    }
    unsafe {
        let _ = CloseHandle(handle);
    }
    true
}

fn clock_unix_secs(clock: &Arc<dyn ClockProvider>) -> Result<i64, IndexLockError> {
    let now = clock.system_now();
    let dur = now
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| IndexLockError::Clock(e.to_string()))?;
    i64::try_from(dur.as_secs()).map_err(|_| IndexLockError::Clock("epoch overflow".into()))
}

// ---------------------------------------------------------------------------
// Tests (acquire/drop unit-level — integration scenarios in tests/index_lock.rs)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use elai_core::OsClock;

    fn os_clock() -> Arc<dyn ClockProvider> {
        Arc::new(OsClock)
    }

    #[test]
    fn lockscope_string_forms() {
        assert_eq!(LockScope::Index.as_str(), "index");
        assert_eq!(LockScope::Watch.as_str(), "watch");
        assert_eq!(LockScope::Index.filename(), "index.lock");
        assert_eq!(LockScope::Watch.filename(), "watch.lock");
    }

    #[test]
    fn parse_lock_body_happy_path() {
        let body = "pid=123\nstarted_at=1700000000\nscope=index\n";
        let info = parse_lock_body(body).expect("parse");
        assert_eq!(info.pid, 123);
        assert_eq!(info.started_at, 1_700_000_000);
        assert_eq!(info.scope, LockScope::Index);
    }

    #[test]
    fn parse_lock_body_rejects_missing_fields() {
        assert!(parse_lock_body("pid=123\n").is_none());
        assert!(parse_lock_body("scope=watch\n").is_none());
    }

    #[test]
    fn acquire_writes_body() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let guard = IndexLock::try_acquire(tmp.path(), LockScope::Index, os_clock())
            .expect("acquire");
        assert_eq!(guard.holder_pid(), std::process::id());
        assert_eq!(guard.scope(), LockScope::Index);
        // Body should be readable.
        let info = read_lock_holder(tmp.path(), LockScope::Index).expect("read holder");
        assert_eq!(info.pid, std::process::id());
        assert_eq!(info.scope, LockScope::Index);
    }

    #[test]
    fn drop_releases_and_deletes_file() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = {
            let guard = IndexLock::try_acquire(tmp.path(), LockScope::Index, os_clock())
                .expect("acquire");
            guard.lock_file().to_path_buf()
        };
        // Guard dropped — file removed.
        assert!(!path.exists(), "lock file should be removed on drop");
    }

    #[test]
    fn pid_zero_is_not_alive() {
        assert!(!pid_alive(0));
    }

    #[test]
    fn current_pid_is_alive() {
        assert!(pid_alive(std::process::id()));
    }

    #[test]
    fn with_index_lock_calls_and_releases() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let result: i32 = with_index_lock(
            tmp.path(),
            LockScope::Index,
            os_clock(),
            |_guard| 42,
        )
        .expect("with_index_lock");
        assert_eq!(result, 42);
        // Released: a second acquire must succeed.
        let _g2 = IndexLock::try_acquire(tmp.path(), LockScope::Index, os_clock())
            .expect("second acquire");
    }
}
