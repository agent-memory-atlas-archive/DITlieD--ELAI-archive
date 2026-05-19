// C-25 Design 2: BackgroundTaskManager — soft/hard timeout auto-detach +
// SSB beacons for spawn/complete events.
//
// SYNOPSIS
// --------
// The orchestrator's existing `bash.rs::BashCommandOutput.background_task_id`
// only tracks user-flagged backgrounded processes; once child.spawn()
// returns, the child reference is dropped and there is no timeout-based
// cleanup. C-25 introduces a manager that OBSERVES already-detached
// children (DRIFT-I Option A — do not rewrite bash.rs internals): callers
// pass a `JoinHandle<i32>` (a wrapper around the spawned future) plus a
// caller-supplied `task_id`, the manager owns it, drives soft/hard
// timeout enforcement, and emits the two new SSB rows.
//
// GROUNDING
// ---------
// Mechanism per `docs/specs/guide/compiled-pipeline-design.txt` Design 2
// with the C-25 plan WIRING_CONTRACT and DRIFT-I resolution as the
// authoritative spec. See `workflow/reasoning/C-25-decisions.md`
// decisions (b)/(d) for the soft/hard timeout policy.
//
// PROCESS PRIORITY LOWERING
// -------------------------
// Spec calls it OPTIONAL. The context-package surveys `windows-sys`,
// `nix`, and `libc` (all already in the workspace) as candidates. C-25
// SHIPS WITHOUT priority lowering — the manager owns Joins only, not
// raw OS PIDs (the existing bash.rs path keeps the PID inside the
// BashCommandOutput struct). Wiring priority would require routing the
// PID through the manager, which is out of scope for C-25. Documented
// in C-25-decisions.md.
//
// NO STUBS. No .unwrap() in library paths.
//
// ---------------------------------------------------------------------------

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use thiserror::Error;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::ssb::{
    BackgroundTaskCompletePayload, BackgroundTaskStartedPayload, Ssb, SsbPayload,
};

// ---------------------------------------------------------------------------
// Constants — named, no magic numbers (per codeRules).
// ---------------------------------------------------------------------------

/// C-25 Task 4 — maximum concurrent background tasks. Spec default = 3.
pub const BACKGROUND_TASK_MAX_CONCURRENT: u8 = 3;

/// C-25 Task 4 — adaptive poll cadence. Starts at 5s, doubles each idle
/// cycle, capped at 30s.
pub const BACKGROUND_TASK_POLL_MIN_SECS: u32 = 5;
pub const BACKGROUND_TASK_POLL_MAX_SECS: u32 = 30;

/// Default soft / hard timeout values (per plan §Task 4). Callers may
/// override per spawn.
pub const BACKGROUND_TASK_DEFAULT_SOFT_SECS: u32 = 30;
pub const BACKGROUND_TASK_DEFAULT_HARD_SECS: u32 = 600;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// C-25 Task 4 — typed error from `BackgroundTaskManager` operations.
#[derive(Debug, Error)]
pub enum BackgroundTaskError {
    #[error("gated_spawn rejected: autonomous_mode=false (interactive mode forbids background)")]
    InteractiveModeRejected,
    #[error("max concurrent background tasks reached ({0})")]
    MaxConcurrentReached(u8),
}

// ---------------------------------------------------------------------------
// BackgroundTaskHandle — one in-flight background task.
// ---------------------------------------------------------------------------

/// C-25 Task 4 — one in-flight background task. `join_handle` is the
/// tokio JoinHandle the manager polls; `auto_detached` flips to `true`
/// when `enforce_timeouts` drops the JoinHandle at the soft boundary.
#[derive(Debug)]
pub struct BackgroundTaskHandle {
    pub task_id: String,
    pub spawned_at: Instant,
    pub soft_timeout: Duration,
    pub hard_timeout: Duration,
    /// Wrapped in Option so `enforce_timeouts` can take the handle
    /// without violating Send/Sync. `None` means the handle has already
    /// been harvested or detached.
    pub join_handle: Option<JoinHandle<i32>>,
    pub auto_detached: bool,
    pub command_summary: String,
}

impl BackgroundTaskHandle {
    /// Construct a handle (test/diagnostics).
    #[must_use]
    pub fn new(
        task_id: String,
        command_summary: String,
        soft_timeout: Duration,
        hard_timeout: Duration,
        join_handle: JoinHandle<i32>,
    ) -> Self {
        Self {
            task_id,
            spawned_at: Instant::now(),
            soft_timeout,
            hard_timeout,
            join_handle: Some(join_handle),
            auto_detached: false,
            command_summary,
        }
    }
}

// ---------------------------------------------------------------------------
// BackgroundTaskManager
// ---------------------------------------------------------------------------

/// C-25 Task 4 — the manager itself. Owns a HashMap of active handles
/// keyed by `task_id`, an optional Ssb writer for spawn/complete rows,
/// and an adaptive poll interval. `autonomous_mode` is captured at
/// `gated_spawn` time so a single manager can be reused across
/// interactive / autonomous boundaries (the per-spawn flag is the gate).
pub struct BackgroundTaskManager {
    active_tasks: Mutex<HashMap<String, BackgroundTaskHandle>>,
    max_concurrent: u8,
    poll_interval_secs: Mutex<u32>,
    ssb_tx: Option<Arc<Ssb>>,
}

impl BackgroundTaskManager {
    #[must_use]
    pub fn new(ssb_tx: Option<Arc<Ssb>>) -> Self {
        Self {
            active_tasks: Mutex::new(HashMap::new()),
            max_concurrent: BACKGROUND_TASK_MAX_CONCURRENT,
            poll_interval_secs: Mutex::new(BACKGROUND_TASK_POLL_MIN_SECS),
            ssb_tx,
        }
    }

    #[must_use]
    pub fn with_max_concurrent(ssb_tx: Option<Arc<Ssb>>, max_concurrent: u8) -> Self {
        Self {
            active_tasks: Mutex::new(HashMap::new()),
            max_concurrent,
            poll_interval_secs: Mutex::new(BACKGROUND_TASK_POLL_MIN_SECS),
            ssb_tx,
        }
    }

    /// Spawn a tracked background task. Emits `background_task_started`
    /// SSB row. Equivalent to `gated_spawn` with `autonomous_mode=true`
    /// — exists for tests and explicit callers that have already
    /// verified the gate.
    pub async fn spawn_task(
        &self,
        task_id: String,
        command_summary: String,
        soft_timeout: Duration,
        hard_timeout: Duration,
        join_handle: JoinHandle<i32>,
    ) -> Result<String, BackgroundTaskError> {
        {
            let active = self.active_tasks.lock().await;
            let active_count =
                u8::try_from(active.len()).unwrap_or(u8::MAX);
            if active_count >= self.max_concurrent {
                return Err(BackgroundTaskError::MaxConcurrentReached(
                    self.max_concurrent,
                ));
            }
        }
        let soft_secs = u32::try_from(soft_timeout.as_secs()).unwrap_or(u32::MAX);
        let hard_secs = u32::try_from(hard_timeout.as_secs()).unwrap_or(u32::MAX);
        let handle = BackgroundTaskHandle::new(
            task_id.clone(),
            command_summary.clone(),
            soft_timeout,
            hard_timeout,
            join_handle,
        );
        {
            let mut active = self.active_tasks.lock().await;
            active.insert(task_id.clone(), handle);
        }
        if let Some(ssb) = self.ssb_tx.as_ref() {
            let payload = SsbPayload::BackgroundTaskStarted(BackgroundTaskStartedPayload {
                task_id: task_id.clone(),
                command_summary,
                soft_timeout_secs: soft_secs,
                hard_timeout_secs: hard_secs,
            });
            if let Err(e) = ssb
                .write_payload("background_task_manager", 0, payload, Vec::new())
                .await
            {
                tracing::warn!(
                    error = %e,
                    task_id = %task_id,
                    "background_task: spawn SSB write failed (non-fatal)"
                );
            }
        }
        Ok(task_id)
    }

    /// Gated spawn — rejects when `autonomous_mode=false` (per stub_ban
    /// contract). Delegates to `spawn_task` on the autonomous branch.
    pub async fn gated_spawn(
        &self,
        autonomous_mode: bool,
        task_id: String,
        command_summary: String,
        soft_timeout: Duration,
        hard_timeout: Duration,
        join_handle: JoinHandle<i32>,
    ) -> Result<String, BackgroundTaskError> {
        if !autonomous_mode {
            return Err(BackgroundTaskError::InteractiveModeRejected);
        }
        self.spawn_task(
            task_id,
            command_summary,
            soft_timeout,
            hard_timeout,
            join_handle,
        )
        .await
    }

    /// Poll active handles, harvest finished ones, emit
    /// `background_task_complete` rows. Adaptive poll: if NO handles
    /// finished this cycle, the next poll interval doubles (capped at
    /// `BACKGROUND_TASK_POLL_MAX_SECS`); if at least one was harvested,
    /// the interval resets to `BACKGROUND_TASK_POLL_MIN_SECS`. Returns
    /// the number of harvested handles.
    pub async fn poll_completed(&self) -> usize {
        let mut harvested = 0_usize;
        let mut finished_keys: Vec<String> = Vec::new();
        {
            let mut active = self.active_tasks.lock().await;
            for (task_id, handle) in active.iter_mut() {
                let Some(join) = handle.join_handle.as_ref() else {
                    continue;
                };
                if join.is_finished() {
                    finished_keys.push(task_id.clone());
                }
            }
            for task_id in &finished_keys {
                if let Some(mut handle) = active.remove(task_id) {
                    let duration_secs =
                        u32::try_from(handle.spawned_at.elapsed().as_secs())
                            .unwrap_or(u32::MAX);
                    let exit_status = match handle.join_handle.take() {
                        Some(join) => match join.await {
                            Ok(code) => code,
                            Err(_join_err) => -1,
                        },
                        None => -1,
                    };
                    self.emit_complete(
                        &handle.task_id,
                        exit_status,
                        duration_secs,
                        handle.auto_detached,
                        false,
                    )
                    .await;
                    harvested += 1;
                }
            }
        }
        // Adaptive poll cadence.
        {
            let mut interval = self.poll_interval_secs.lock().await;
            if harvested == 0 {
                let doubled = interval.saturating_mul(2);
                *interval = doubled.min(BACKGROUND_TASK_POLL_MAX_SECS);
            } else {
                *interval = BACKGROUND_TASK_POLL_MIN_SECS;
            }
        }
        harvested
    }

    /// Enforce per-handle timeouts. Walks all active handles; for each:
    ///   * soft_timeout exceeded -> drop JoinHandle (auto_detached=true),
    ///     emit background_task_complete with auto_detached=true.
    ///   * hard_timeout exceeded -> abort the JoinHandle, emit
    ///     background_task_complete with timed_out=true.
    /// Returns the number of detached + aborted handles combined.
    pub async fn enforce_timeouts(&self) -> usize {
        let mut detached_or_aborted = 0_usize;
        let mut to_remove: Vec<(String, bool, bool, u32, i32)> = Vec::new(); // (id, detached, timed_out, dur, exit)
        {
            let mut active = self.active_tasks.lock().await;
            for (task_id, handle) in active.iter_mut() {
                let elapsed = handle.spawned_at.elapsed();
                if elapsed >= handle.hard_timeout {
                    // HARD: abort + remove.
                    if let Some(join) = handle.join_handle.take() {
                        join.abort();
                    }
                    let dur = u32::try_from(elapsed.as_secs()).unwrap_or(u32::MAX);
                    to_remove.push((task_id.clone(), handle.auto_detached, true, dur, -1));
                } else if elapsed >= handle.soft_timeout && !handle.auto_detached {
                    // SOFT: detach (drop handle) — task continues but the
                    // manager stops tracking it.
                    handle.auto_detached = true;
                    let _ = handle.join_handle.take();
                    let dur = u32::try_from(elapsed.as_secs()).unwrap_or(u32::MAX);
                    to_remove.push((task_id.clone(), true, false, dur, -1));
                }
            }
            for (id, _det, _to, _dur, _exit) in &to_remove {
                active.remove(id);
            }
        }
        for (task_id, detached, timed_out, dur, exit) in &to_remove {
            self.emit_complete(task_id, *exit, *dur, *detached, *timed_out)
                .await;
            detached_or_aborted += 1;
        }
        detached_or_aborted
    }

    /// Current poll interval (seconds). For diagnostics + scheduling.
    pub async fn poll_interval_secs(&self) -> u32 {
        *self.poll_interval_secs.lock().await
    }

    /// Active handle count.
    pub async fn active_count(&self) -> usize {
        self.active_tasks.lock().await.len()
    }

}

impl std::fmt::Debug for BackgroundTaskManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BackgroundTaskManager")
            .field("max_concurrent", &self.max_concurrent)
            .field("ssb_tx_wired", &self.ssb_tx.is_some())
            .finish()
    }
}

impl BackgroundTaskManager {
    async fn emit_complete(
        &self,
        task_id: &str,
        exit_status: i32,
        duration_secs: u32,
        auto_detached: bool,
        timed_out: bool,
    ) {
        if let Some(ssb) = self.ssb_tx.as_ref() {
            let payload = SsbPayload::BackgroundTaskComplete(BackgroundTaskCompletePayload {
                task_id: task_id.to_string(),
                exit_status,
                duration_secs,
                auto_detached,
                timed_out,
            });
            if let Err(e) = ssb
                .write_payload("background_task_manager", 0, payload, Vec::new())
                .await
            {
                tracing::warn!(
                    error = %e,
                    task_id = %task_id,
                    "background_task: complete SSB write failed (non-fatal)"
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests — narrow, in-module, real tokio JoinHandles.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn spawn_task_emits_no_ssb_when_writer_absent() {
        let mgr = BackgroundTaskManager::new(None);
        let h = tokio::spawn(async { 0_i32 });
        let id = mgr
            .spawn_task(
                "t1".into(),
                "echo hi".into(),
                Duration::from_secs(10),
                Duration::from_secs(60),
                h,
            )
            .await
            .expect("spawn ok");
        assert_eq!(id, "t1");
        assert_eq!(mgr.active_count().await, 1);
    }

    #[tokio::test]
    async fn gated_spawn_rejects_interactive_mode() {
        let mgr = BackgroundTaskManager::new(None);
        let h = tokio::spawn(async { 0_i32 });
        let r = mgr
            .gated_spawn(
                false,
                "t1".into(),
                "echo hi".into(),
                Duration::from_secs(10),
                Duration::from_secs(60),
                h,
            )
            .await;
        assert!(matches!(r, Err(BackgroundTaskError::InteractiveModeRejected)));
    }

    #[tokio::test]
    async fn max_concurrent_enforced() {
        let mgr = BackgroundTaskManager::with_max_concurrent(None, 1);
        let h1 = tokio::spawn(async {
            tokio::time::sleep(Duration::from_millis(200)).await;
            0_i32
        });
        let h2 = tokio::spawn(async { 0_i32 });
        mgr.spawn_task(
            "a".into(),
            "x".into(),
            Duration::from_secs(10),
            Duration::from_secs(60),
            h1,
        )
        .await
        .expect("ok");
        let r = mgr
            .spawn_task(
                "b".into(),
                "x".into(),
                Duration::from_secs(10),
                Duration::from_secs(60),
                h2,
            )
            .await;
        assert!(matches!(r, Err(BackgroundTaskError::MaxConcurrentReached(_))));
    }

    #[tokio::test]
    async fn poll_completed_harvests_finished_handles() {
        let mgr = BackgroundTaskManager::new(None);
        let h = tokio::spawn(async { 42_i32 });
        mgr.spawn_task(
            "done".into(),
            "x".into(),
            Duration::from_secs(10),
            Duration::from_secs(60),
            h,
        )
        .await
        .expect("ok");
        // Give the task time to complete.
        tokio::time::sleep(Duration::from_millis(50)).await;
        let harvested = mgr.poll_completed().await;
        assert_eq!(harvested, 1);
        assert_eq!(mgr.active_count().await, 0);
    }

    #[tokio::test]
    async fn poll_adaptive_cadence_doubles_on_idle() {
        let mgr = BackgroundTaskManager::new(None);
        let before = mgr.poll_interval_secs().await;
        assert_eq!(before, BACKGROUND_TASK_POLL_MIN_SECS);
        // No active tasks, no harvest -> doubles.
        mgr.poll_completed().await;
        let after = mgr.poll_interval_secs().await;
        assert_eq!(after, before * 2);
        // Cap at MAX (5 -> 10 -> 20 -> 30).
        for _ in 0..5 {
            mgr.poll_completed().await;
        }
        let saturated = mgr.poll_interval_secs().await;
        assert!(saturated <= BACKGROUND_TASK_POLL_MAX_SECS);
    }

    #[tokio::test]
    async fn enforce_timeouts_soft_detaches_long_running() {
        let mgr = BackgroundTaskManager::new(None);
        // Spawn a task that sleeps longer than the soft timeout.
        let h = tokio::spawn(async {
            tokio::time::sleep(Duration::from_millis(500)).await;
            0_i32
        });
        mgr.spawn_task(
            "slow".into(),
            "x".into(),
            Duration::from_millis(10), // soft = 10ms
            Duration::from_secs(60),   // hard = 60s (won't fire)
            h,
        )
        .await
        .expect("ok");
        // Wait past the soft boundary.
        tokio::time::sleep(Duration::from_millis(50)).await;
        let detached = mgr.enforce_timeouts().await;
        assert_eq!(detached, 1, "soft timeout must detach");
        assert_eq!(mgr.active_count().await, 0);
    }
}
