//! B-31 Phase 4 (Cycle B): startup lifecycle helpers — auto-resume and
//! graceful shutdown coordination across all known projects.
//!
//! See `workflow/reasoning/B-31-decisions.md` DECISION-15..DECISION-17 for
//! the locked design. Mirrors `docs/.source/SocratiCode-main/src/services/
//! startup.ts` (autoResumeIndexedProjects + gracefulShutdown) with two
//! deliberate adaptations:
//!
//! 1. **Discovery via filesystem walk**, not a single-project cwd: each
//!    project's `metadata.db` lives at `<indexes_root>/<project>/metadata.db`.
//!    Auto-resume walks `indexes_root` depth-2 and opens each `metadata.db`
//!    it finds. Single-project callers (the CLI) can pass the project's own
//!    indexes parent and get exactly one project resumed.
//! 2. **Process-global SHUTTING_DOWN flag** instead of a per-tree boolean:
//!    we want any in-flight task spawned by `auto_resume_indexed_projects`
//!    to observe `graceful_shutdown` even if it has already passed the
//!    initial entry check. The flag is set once and never cleared (process
//!    is exiting).
//!
//! SSB emission
//! ------------
//! B-31.5 (Cycle C): the `tracing::info!(target = "ssb_pending", ...)`
//! placeholders left by Cycle B have been replaced with real
//! `Ssb::write_payload` calls. LockAcquired / LockContested rows are
//! emitted by `IndexLock::try_acquire` itself via the process-global SSB
//! writer slot in `lock.rs`. IndexResume / IndexCancel rows are emitted
//! by `build_index_resumable` (and by `graceful_shutdown` for the
//! cancellation-request path). When the process-global writer is unset
//! (every contract test, cold start before SSB init) the emissions are
//! pure no-ops with a single `tracing::debug!` line.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use elai_core::{ClockProvider, OsClock};
use elai_knowledge::metadata::{
    self, IndexingStatus, ProjectMetadataStore, METADATA_DB_FILENAME,
};

use crate::indexing_service::{IndexResult, IndexingService};
use crate::lock::{IndexLock, IndexLockError, LockScope};
use crate::ssb::{IndexCancelPayload, Ssb, SsbPayload};

// ---------------------------------------------------------------------------
// Named constants (no magic numbers)
// ---------------------------------------------------------------------------

/// How long `graceful_shutdown` waits for active indexers to honour their
/// cancellation flag before force-releasing OS locks. Matches SocratiCode
/// startup.ts:153 (`awaitActiveIndexing(60_000)`).
pub const STARTUP_SHUTDOWN_DEADLINE_SECS: u64 = 60;

/// C-17 (DRIFT-L): canonical `Duration` form of the shutdown deadline.
/// Pinned by `crates/contracts/tests/c17_indexing_resilience_contract.rs`
/// so a future renaming or magic-number relapse trips the contract.
/// Constraint: 30s..=120s; the current 30s value keeps a single SIGINT
/// responsive while still letting in-flight batch flushes complete.
pub const SHUTDOWN_DEADLINE: Duration = Duration::from_secs(30);

/// Poll cadence inside the shutdown wait loop. Small enough to make a 60s
/// deadline feel responsive; large enough not to dominate CPU when the only
/// thing running is a cancellation-flag check.
const SHUTDOWN_POLL_INTERVAL_MS: u64 = 100;

/// Indexes directory name under the project root. Mirrors the constant in
/// `elai_core::paths` so we don't need to re-export the private string.
const INDEXES_DIR_NAME: &str = elai_core::paths::INDEXES_DIR_NAME;

// ---------------------------------------------------------------------------
// Process-global SHUTTING_DOWN flag (DECISION-17)
// ---------------------------------------------------------------------------

/// Process-global shutdown flag. Set once by [`graceful_shutdown`]; never
/// cleared. Checked at every per-project resume task entry point and by
/// long-running indexing batch loops (the indexing-service-side check lands
/// in PATCH-2 once `indexing_service.rs` unlocks).
static SHUTTING_DOWN: AtomicBool = AtomicBool::new(false);

/// Read-only accessor for the process-global shutdown flag.
pub fn shutting_down() -> bool {
    SHUTTING_DOWN.load(Ordering::SeqCst)
}

#[cfg(test)]
pub(crate) fn reset_shutting_down_for_test() {
    SHUTTING_DOWN.store(false, Ordering::SeqCst);
}

// ---------------------------------------------------------------------------
// B-31.5: optional process-global SSB writer for startup-side emissions
// ---------------------------------------------------------------------------
//
// The dashboard / CLI bootstrap calls `install_ssb_writer` to plumb the
// project's Ssb instance through. `auto_resume_indexed_projects` and
// `graceful_shutdown` consult this slot when emitting IndexResume /
// IndexCancel rows. When unset every emit is a no-op.

use std::sync::Mutex as StdMutex;
use std::sync::OnceLock;

fn ssb_writer_slot() -> &'static StdMutex<Option<Arc<Ssb>>> {
    static SLOT: OnceLock<StdMutex<Option<Arc<Ssb>>>> = OnceLock::new();
    SLOT.get_or_init(|| StdMutex::new(None))
}

/// B-31.5: install the process-global SSB writer used by
/// `auto_resume_indexed_projects` and `graceful_shutdown` to emit
/// IndexResume / IndexCancel rows. Idempotent.
pub fn install_ssb_writer(writer: Arc<Ssb>) {
    let mut g = match ssb_writer_slot().lock() {
        Ok(g) => g,
        Err(poison) => poison.into_inner(),
    };
    *g = Some(writer);
}

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

/// Best-effort SSB emit from the startup module. Mirrors
/// `lock::emit_lock_ssb` — async-bridge with debug-log no-op fallback.
fn emit_startup_ssb(payload: SsbPayload) {
    let Some(writer) = current_ssb_writer() else {
        tracing::debug!(
            action_type = payload.action_type().as_sql_str(),
            "B-31.5: startup SSB writer absent — payload dropped",
        );
        return;
    };
    let action_label = payload.action_type().as_sql_str();
    let drive = move |writer: Arc<Ssb>, payload: SsbPayload| async move {
        if let Err(e) = writer
            .write_payload("startup", 0, payload, Vec::new())
            .await
        {
            tracing::warn!(
                target = "startup",
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
                                target = "startup",
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
                        target = "startup",
                        "B-31.5: startup-emit worker thread panicked",
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
                        target = "startup",
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
// Reports
// ---------------------------------------------------------------------------

/// Result of [`auto_resume_indexed_projects`]. Counts are populated as the
/// per-project tasks finish.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AutoResumeReport {
    /// Projects whose `metadata.db` could be opened.
    pub projects_discovered: u32,
    /// Projects with status=InProgress that we successfully acquired the
    /// IndexLock for and re-ran to completion / cancellation.
    pub projects_resumed: u32,
    /// Projects whose IndexLock was held by another live process; we left
    /// them alone.
    pub projects_skipped_locked: u32,
    /// Projects with status=Completed (no resume needed) OR Cancelled
    /// (durable signal — leave alone).
    pub projects_skipped_completed: u32,
    /// Projects that errored out at metadata-open or build-index-resume
    /// time. Their durable status row remains InProgress, so the next
    /// startup will retry.
    pub projects_errored: u32,
}

/// Result of [`graceful_shutdown`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShutdownReport {
    /// Projects whose cancellation flag we flipped via
    /// `IndexingService::cancel_indexing`.
    pub projects_cancelled: u32,
    /// Lock files we had to delete after the deadline expired.
    pub locks_force_released: u32,
    /// True when the wait loop ran out of time and we force-released locks.
    pub deadline_exceeded: bool,
}

// ---------------------------------------------------------------------------
// auto_resume_indexed_projects
// ---------------------------------------------------------------------------

/// Discover every project under `indexes_root` and resume any whose
/// persisted status is [`IndexingStatus::InProgress`].
///
/// Behaviour per project (mirrors SocratiCode startup.ts:37-150):
/// * Open `metadata.db` and load every project_id.
/// * Status=`Completed` / `Cancelled` → no-op (counted as
///   `projects_skipped_completed`).
/// * Status=`InProgress`:
///   * Try to acquire `LockScope::Index` for that project's root.
///   * On `AlreadyHeld` → `projects_skipped_locked` and log.
///   * On success → spawn a `tokio::task::spawn_blocking` running
///     `IndexingService::build_index_resumable`. The SSB rows
///     (`LockAcquired` / `IndexResume` / `IndexCancel`) are emitted by
///     the call sites themselves once `install_ssb_writer` has plumbed in
///     the per-project SSB writer (B-31.5).
///
/// Fire-and-forget per-project: each project's task is independent. The
/// returned future awaits all of them and returns the aggregate report.
pub async fn auto_resume_indexed_projects(
    indexes_root: PathBuf,
    clock: Arc<dyn ClockProvider>,
) -> AutoResumeReport {
    if shutting_down() {
        tracing::info!("auto_resume_indexed_projects: shutdown already in progress, skipping");
        return AutoResumeReport::default();
    }

    let mut report = AutoResumeReport::default();

    let metadata_dbs = discover_metadata_dbs(&indexes_root);
    if metadata_dbs.is_empty() {
        return report;
    }

    // Per-DB tasks: each runs in spawn_blocking (sync rusqlite + sync
    // indexing) and returns a partial report.
    let mut handles = Vec::with_capacity(metadata_dbs.len());
    for (db_path, project_indexes_dir) in metadata_dbs {
        let clock_clone = Arc::clone(&clock);
        let handle = tokio::task::spawn_blocking(move || {
            resume_db(&db_path, &project_indexes_dir, clock_clone)
        });
        handles.push(handle);
    }

    for handle in handles {
        match handle.await {
            Ok(partial) => {
                report.projects_discovered = report
                    .projects_discovered
                    .saturating_add(partial.projects_discovered);
                report.projects_resumed = report
                    .projects_resumed
                    .saturating_add(partial.projects_resumed);
                report.projects_skipped_locked = report
                    .projects_skipped_locked
                    .saturating_add(partial.projects_skipped_locked);
                report.projects_skipped_completed = report
                    .projects_skipped_completed
                    .saturating_add(partial.projects_skipped_completed);
                report.projects_errored = report
                    .projects_errored
                    .saturating_add(partial.projects_errored);
            }
            Err(join_err) => {
                tracing::warn!(error = %join_err, "auto_resume per-DB task join failed");
                report.projects_errored = report.projects_errored.saturating_add(1);
            }
        }
    }

    tracing::info!(?report, "auto_resume_indexed_projects: finished");

    // C-17 (DRIFT-F closure): start a FileWatcherService per indexed
    // project so LockScope::Watch is held while the host process is
    // alive. The watcher guard is intentionally leaked into the global
    // sink — we want the lock for the lifetime of the process, and
    // `graceful_shutdown`'s force_release loop already iterates both
    // LockScope::Index and LockScope::Watch on deadline expiry.
    //
    // Failure modes:
    //   * Another live process already holds the Watch lock — log and
    //     skip; the running process will keep watching.
    //   * Lock plumbing error — log and continue; the indexing run is
    //     already done so a missing watcher is observable but non-fatal.
    for (db_path, project_indexes_dir) in discover_metadata_dbs(&indexes_root) {
        let _ = db_path; // db_path informational here.
        let project_root = match grandparent(&project_indexes_dir) {
            Some(p) => p,
            None => continue,
        };
        match crate::watcher_service::FileWatcherService::start(
            &project_root,
            Arc::clone(&clock),
        ) {
            Ok(svc) => {
                tracing::info!(
                    project_root = %svc.project_root().display(),
                    holder_pid = svc.holder_pid(),
                    "auto_resume: FileWatcherService started (Watch lock held)"
                );
                stash_watcher(svc);
            }
            Err(e) => {
                tracing::warn!(
                    project_root = %project_root.display(),
                    error = %e,
                    "auto_resume: FileWatcherService start skipped (Watch lock unavailable)"
                );
            }
        }
    }

    report
}

// C-17: process-global stash for the per-project FileWatcherService
// guards. Push-only; entries are dropped (releasing the Watch lock) on
// process exit OR when `graceful_shutdown` runs its force_release loop.
fn watcher_stash_slot() -> &'static StdMutex<Vec<crate::watcher_service::FileWatcherService>> {
    static SLOT: OnceLock<StdMutex<Vec<crate::watcher_service::FileWatcherService>>> = OnceLock::new();
    SLOT.get_or_init(|| StdMutex::new(Vec::new()))
}

fn stash_watcher(svc: crate::watcher_service::FileWatcherService) {
    if let Ok(mut g) = watcher_stash_slot().lock() {
        g.push(svc);
    }
}

/// Synchronous body of one DB's resume cycle. Runs inside spawn_blocking.
fn resume_db(
    db_path: &Path,
    project_indexes_dir: &Path,
    clock: Arc<dyn ClockProvider>,
) -> AutoResumeReport {
    let mut report = AutoResumeReport::default();

    let store = match ProjectMetadataStore::open(db_path, Arc::clone(&clock)) {
        Ok(s) => s,
        Err(err) => {
            tracing::warn!(
                path = %db_path.display(),
                error = %err,
                "auto_resume: failed to open metadata DB"
            );
            report.projects_errored = 1;
            return report;
        }
    };

    let project_ids = match store.list_project_ids() {
        Ok(ids) => ids,
        Err(err) => {
            tracing::warn!(
                path = %db_path.display(),
                error = %err,
                "auto_resume: list_project_ids failed"
            );
            report.projects_errored = 1;
            return report;
        }
    };

    for project_id in project_ids {
        if shutting_down() {
            tracing::info!("auto_resume: shutdown signalled mid-iteration, bailing");
            return report;
        }
        report.projects_discovered = report.projects_discovered.saturating_add(1);
        let project_meta = match store.load(&project_id) {
            Ok(Some(m)) => m,
            Ok(None) => continue,
            Err(err) => {
                tracing::warn!(
                    project_id = %project_id,
                    error = %err,
                    "auto_resume: metadata load failed"
                );
                report.projects_errored = report.projects_errored.saturating_add(1);
                continue;
            }
        };

        match project_meta.indexing_status {
            IndexingStatus::Completed | IndexingStatus::Cancelled => {
                tracing::debug!(
                    project_id = %project_id,
                    status = ?project_meta.indexing_status,
                    "auto_resume: project not in-progress, skipping"
                );
                report.projects_skipped_completed =
                    report.projects_skipped_completed.saturating_add(1);
                continue;
            }
            IndexingStatus::InProgress => {}
        }

        let project_root = project_meta.project_path.clone();
        if !project_root.exists() {
            tracing::warn!(
                project_id = %project_id,
                path = %project_root.display(),
                "auto_resume: persisted project_path missing on disk; skipping"
            );
            report.projects_errored = report.projects_errored.saturating_add(1);
            continue;
        }

        // Lock attempt.
        let guard = match IndexLock::try_acquire(
            &project_root,
            LockScope::Index,
            Arc::clone(&clock),
        ) {
            Ok(g) => g,
            Err(IndexLockError::AlreadyHeld { holder_pid }) => {
                // B-31.5: LockContested SSB row is emitted by IndexLock::try_acquire
                // itself when the process-global writer is installed. Plain
                // tracing::info! here for dev-console visibility only.
                tracing::info!(
                    project_id = %project_id,
                    scope = LockScope::Index.as_str(),
                    holder_pid = holder_pid,
                    our_pid = std::process::id(),
                    "auto_resume: another process is indexing this project"
                );
                report.projects_skipped_locked =
                    report.projects_skipped_locked.saturating_add(1);
                continue;
            }
            Err(err) => {
                tracing::warn!(
                    project_id = %project_id,
                    error = %err,
                    "auto_resume: lock acquisition failed"
                );
                report.projects_errored = report.projects_errored.saturating_add(1);
                continue;
            }
        };

        // B-31.5: LockAcquired SSB row is emitted by IndexLock::try_acquire
        // when the process-global writer is installed. Plain tracing::info!
        // here for dev-console visibility only.
        tracing::info!(
            project_id = %project_id,
            scope = LockScope::Index.as_str(),
            holder_pid = guard.holder_pid(),
            "auto_resume: acquired lock"
        );

        // Resume the indexing run synchronously (we're in spawn_blocking).
        // AUDIT WARN FIX: attach the process-global Ssb writer (when
        // installed) so per-batch IndexCheckpoint / IndexResume / IndexCancel
        // rows are actually persisted instead of silently dropped on the
        // auto-resume path. Mirrors how the dashboard plumbs its writer.
        let mut service = IndexingService::new(project_root.clone(), project_indexes_dir.to_path_buf());
        if let Some(writer) = current_ssb_writer() {
            service = service.with_ssb_writer(writer);
        }
        let resume_outcome = service.build_index_resumable(&project_id, &store, Arc::clone(&clock));

        match resume_outcome {
            Ok(IndexResult::Completed { summary }) => {
                // Fresh first run that completed without skipping any files.
                // No IndexResume / IndexCancel SSB row is appropriate here —
                // build_index_resumable already emitted per-batch
                // IndexCheckpoint rows. Plain tracing::info! only.
                tracing::info!(
                    project_id = %project_id,
                    bm25_files = summary.bm25_files,
                    bm25_chunks = summary.bm25_chunks,
                    "auto_resume: project indexing completed (fresh)"
                );
                report.projects_resumed = report.projects_resumed.saturating_add(1);
            }
            Ok(IndexResult::Resumed {
                summary,
                resumed_from_batch,
                files_skipped,
            }) => {
                // IndexResume SSB row is emitted by build_index_resumable
                // itself on the Resumed return path. Plain tracing::info!
                // here for dev-console visibility only.
                tracing::info!(
                    project_id = %project_id,
                    resumed_from_batch = resumed_from_batch,
                    files_skipped = files_skipped,
                    bm25_files = summary.bm25_files,
                    "auto_resume: project indexing resumed and finished"
                );
                report.projects_resumed = report.projects_resumed.saturating_add(1);
            }
            Ok(IndexResult::Cancelled {
                batches_completed,
                files_indexed,
            }) => {
                // IndexCancel SSB row is emitted by build_index_resumable
                // itself on the Cancelled return path. Plain tracing::info!
                // here for dev-console visibility only.
                tracing::info!(
                    project_id = %project_id,
                    batches_completed = batches_completed,
                    files_indexed = files_indexed,
                    "auto_resume: project indexing cancelled mid-resume"
                );
                report.projects_resumed = report.projects_resumed.saturating_add(1);
            }
            Ok(IndexResult::AlreadyLocked { holder_pid }) => {
                // This path is reachable only once indexing_service adopts
                // with_index_lock and our outer guard is still held — i.e.
                // never in Cycle B, but recorded as a "skipped locked"
                // outcome for future patches.
                tracing::warn!(
                    project_id = %project_id,
                    holder_pid = holder_pid,
                    "auto_resume: build_index_resumable returned AlreadyLocked despite outer guard"
                );
                report.projects_skipped_locked =
                    report.projects_skipped_locked.saturating_add(1);
            }
            Err(err) => {
                tracing::warn!(
                    project_id = %project_id,
                    error = %err,
                    "auto_resume: build_index_resumable failed"
                );
                report.projects_errored = report.projects_errored.saturating_add(1);
            }
        }

        // Release guard explicitly so the next project's attempt sees the
        // lock file gone immediately.
        drop(guard);
    }

    report
}

// ---------------------------------------------------------------------------
// graceful_shutdown
// ---------------------------------------------------------------------------

/// Set the process-global [`SHUTTING_DOWN`] flag, cancel every in-progress
/// project's indexing run, wait up to `deadline` for them to honour the
/// cancellation flag, and force-release any orphan lock files left behind.
///
/// Mirrors SocratiCode startup.ts:180-207 (`gracefulShutdown`) — the only
/// behavioural difference is that we discover in-progress projects by
/// re-reading `metadata.db` rather than holding an in-memory registry,
/// which lets us call this from a Tauri `RunEvent::ExitRequested` closure
/// without requiring `&mut` on shared state (see gotchas.md L1047
/// DASHBOARD_SECOND_SSB_WRITER_MVP_TRADEOFF).
pub async fn graceful_shutdown(
    indexes_root: PathBuf,
    deadline: Duration,
    clock: Arc<dyn ClockProvider>,
) -> ShutdownReport {
    SHUTTING_DOWN.store(true, Ordering::SeqCst);
    tracing::info!(?deadline, "graceful_shutdown: SHUTTING_DOWN flag set");

    let mut report = ShutdownReport::default();
    let in_progress = list_in_progress_project_roots(&indexes_root, Arc::clone(&clock));

    if in_progress.is_empty() {
        tracing::info!("graceful_shutdown: no in-progress projects, exiting fast path");
        return report;
    }

    let service = IndexingService::new(
        // project_root is unused by cancel_indexing — it only consults the
        // process-global cancellation_registry. Pass any path.
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        indexes_root.clone(),
    );

    for project_id in &in_progress {
        let was_running = service.cancel_indexing(project_id);
        report.projects_cancelled = report.projects_cancelled.saturating_add(1);
        // B-31.5: emit IndexCancel SSB row to surface the shutdown-requested
        // cancellation in the dashboard activity feed. batches_completed /
        // files_indexed are zero here — the real per-batch counts are
        // emitted by build_index_resumable when it observes the cancel flag
        // at its next batch boundary.
        emit_startup_ssb(SsbPayload::IndexCancel(IndexCancelPayload {
            project_id: project_id.clone(),
            batches_completed: 0,
            files_indexed: 0,
        }));
        tracing::info!(
            project_id = %project_id,
            was_running = was_running,
            "graceful_shutdown: cancellation requested"
        );
    }

    // Wait loop: poll the persisted metadata for any project still
    // InProgress. Done when none remain or `deadline` elapses.
    // C-15: OsClock-backed deadline arithmetic (raw Instant).
    let shutdown_clock = OsClock;
    let start = shutdown_clock.instant_now();
    loop {
        if shutdown_clock.instant_now().saturating_duration_since(start) >= deadline {
            report.deadline_exceeded = true;
            break;
        }
        let still_running = list_in_progress_project_roots(&indexes_root, Arc::clone(&clock));
        if still_running.is_empty() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(SHUTDOWN_POLL_INTERVAL_MS)).await;
    }

    if report.deadline_exceeded {
        tracing::warn!(
            "graceful_shutdown: deadline {:?} exceeded — force-releasing index locks. \
             Progress is checkpointed; it will resume with no data loss.",
            deadline
        );
        // Force-release every project_root's index lock.
        for (_db, project_indexes_dir) in discover_metadata_dbs(&indexes_root) {
            // The project root is the grandparent of the metadata DB
            // (`<root>/.elai_cc/indexes/metadata.db`).
            let project_root = match grandparent(&project_indexes_dir) {
                Some(p) => p,
                None => continue,
            };
            for scope in [LockScope::Index, LockScope::Watch] {
                if IndexLock::is_locked_by_other(&project_root, scope).is_some() {
                    if let Err(err) = IndexLock::force_release(&project_root, scope) {
                        tracing::warn!(
                            error = %err,
                            path = %project_root.display(),
                            scope = scope.as_str(),
                            "graceful_shutdown: force_release failed"
                        );
                    } else {
                        report.locks_force_released =
                            report.locks_force_released.saturating_add(1);
                    }
                }
            }
        }
    }

    tracing::info!(?report, "graceful_shutdown: complete");
    report
}

// ---------------------------------------------------------------------------
// Discovery helpers
// ---------------------------------------------------------------------------

/// Walk `indexes_root` (depth-2 at most) and return every
/// `(metadata.db_path, project_indexes_dir)` pair we find.
///
/// Layout supported:
/// * `indexes_root/metadata.db` — single-project (CLI case).
/// * `indexes_root/<project_slug>/metadata.db` — multi-project dashboard.
fn discover_metadata_dbs(indexes_root: &Path) -> Vec<(PathBuf, PathBuf)> {
    let mut out = Vec::new();

    let direct = indexes_root.join(METADATA_DB_FILENAME);
    if direct.is_file() {
        out.push((direct, indexes_root.to_path_buf()));
    }

    let read = match std::fs::read_dir(indexes_root) {
        Ok(rd) => rd,
        Err(err) => {
            tracing::debug!(
                path = %indexes_root.display(),
                error = %err,
                "discover_metadata_dbs: read_dir failed (treating as empty)"
            );
            return out;
        }
    };

    for entry in read.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let candidate = path.join(METADATA_DB_FILENAME);
        if candidate.is_file() {
            out.push((candidate, path));
        }
    }

    out
}

/// Return every persisted `project_id` whose status is currently
/// `InProgress`, across every `metadata.db` under `indexes_root`.
fn list_in_progress_project_roots(
    indexes_root: &Path,
    clock: Arc<dyn ClockProvider>,
) -> Vec<String> {
    let mut out = Vec::new();
    for (db_path, _project_indexes_dir) in discover_metadata_dbs(indexes_root) {
        let store = match ProjectMetadataStore::open(&db_path, Arc::clone(&clock)) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let ids = match store.list_project_ids() {
            Ok(v) => v,
            Err(_) => continue,
        };
        for project_id in ids {
            if let Ok(Some(meta)) = store.load(&project_id) {
                if matches!(meta.indexing_status, IndexingStatus::InProgress) {
                    out.push(project_id);
                }
            }
        }
    }
    out
}

fn grandparent(p: &Path) -> Option<PathBuf> {
    Some(p.parent()?.parent()?.to_path_buf())
}

// ---------------------------------------------------------------------------
// Test helpers (re-exports for integration tests)
// ---------------------------------------------------------------------------

/// Public, side-effecting helper used by integration tests to verify that
/// `metadata::project_id_from_path` agrees with our resume code. Keeping
/// it here so tests don't have to import the knowledge crate's helper
/// directly.
pub fn project_id_from_path(path: &Path) -> String {
    metadata::project_id_from_path(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shutting_down_starts_false() {
        // Test ordering matters; reset to be sure.
        reset_shutting_down_for_test();
        assert!(!shutting_down());
    }

    #[test]
    fn discover_metadata_dbs_handles_missing_root() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let missing = tmp.path().join("nonexistent");
        assert!(discover_metadata_dbs(&missing).is_empty());
    }

    #[test]
    fn grandparent_handles_root() {
        let p = Path::new("/");
        assert!(grandparent(p).is_none());
    }

    #[test]
    fn report_defaults_zero() {
        let r = AutoResumeReport::default();
        assert_eq!(r.projects_discovered, 0);
        assert_eq!(r.projects_resumed, 0);
        assert_eq!(r.projects_skipped_locked, 0);
        assert_eq!(r.projects_skipped_completed, 0);
        assert_eq!(r.projects_errored, 0);
    }
}

// Force the constant to be considered "used" by tooling — see the test file
// using it.
const _: u64 = STARTUP_SHUTDOWN_DEADLINE_SECS;
const _: &str = INDEXES_DIR_NAME;
