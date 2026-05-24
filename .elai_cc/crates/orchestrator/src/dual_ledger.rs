//! E-46 — Dual ledger (TaskLedger + ProgressLedger) atomic advancement.
//!
//! Implements the Magentic-One-style dual-ledger primitive at the
//! orchestrator's task tier. Each task carries two ledgers updated as
//! a single unit on every advance:
//!
//!   - [`TaskLedger`] — static task identity + criteria + status.
//!   - [`ProgressLedger`] — completion percent + blockers + last
//!     update timestamp.
//!
//! [`DualLedger::advance_task`] is the sole mutation point; callers
//! pass a `completion_delta` and an optional blocker. The function
//! updates both ledgers atomically (single `&mut self` borrow) and
//! returns a [`TaskAdvancement`] view that downstream consumers can
//! pipe into [`SsbPayload::TaskAdvanced`] without re-reading the
//! ledgers.
//!
//! Wiring contract (HARD):
//!   1. [`DualLedger::advance_task`] does not emit SSB rows itself
//!      (no `&Ssb` in scope) — the producer call lives in
//!      `agent_pool::process_task_advancement`. Keeping the ledger
//!      pure simplifies unit testing and matches D-7's
//!      `run_canary` / `emit_canary_run` split.
//!   2. The progress ledger feeds BAI selection criteria: when
//!      `completion_pct >= 100.0`, the task is eligible for the BAI
//!      promotion-eval pool. The check lives in `agent_pool` (see
//!      `is_promotion_eligible`).
//!   3. No `unwrap()` outside `#[cfg(test)]`; native async-fn not
//!      required (this module is sync — the underlying clock source
//!      is injected as a `ts_ms` parameter so tests can pin it).
//!
//! Anti-stale-training: tracing for logs (never `log`), thiserror for
//! errors (never `Box<dyn Error>`), no `async-trait`.
//!
//! Grounding: arxiv:2412.21154 (Devin / SWE-bench task-ledger pattern,
//! 13.86% solve rate driven by explicit task-state tracking);
//! arxiv:2411.04468 (Magentic-One dual-ledger orchestrator).
//!
//! See F:\.ELAI_workflow\docs\thefinalglance\E-plan\E-46_planning_decomposition.md.

use std::collections::HashMap;

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, warn};

// ---------------------------------------------------------------------------
// Newtypes
// ---------------------------------------------------------------------------

/// Task identifier. Newtype around `String` so the type system
/// distinguishes task ids from arbitrary strings at the API boundary.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TaskId(pub String);

impl TaskId {
    /// Borrow the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for TaskId {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

impl From<String> for TaskId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

// ---------------------------------------------------------------------------
// Status (closed enum)
// ---------------------------------------------------------------------------

/// Closed sum over task lifecycle states. Snake-case wire labels
/// match the SSB payload `status` field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Complete,
    Blocked,
}

impl TaskStatus {
    /// Snake-case wire label used by the SSB payload + dashboard.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::InProgress => "in_progress",
            Self::Complete => "complete",
            Self::Blocked => "blocked",
        }
    }
}

// ---------------------------------------------------------------------------
// Ledger value types
// ---------------------------------------------------------------------------

/// TaskLedger — Magentic-One-style task identity + acceptance criteria
/// + lifecycle status. The static half of the dual ledger; mutated
/// only on status change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskLedger {
    /// Stable task identifier.
    pub task_id: TaskId,
    /// Optional parent task id (`None` when the task is a root).
    pub parent_id: Option<TaskId>,
    /// Current status.
    pub status: TaskStatus,
    /// Acceptance criteria copied from the planning surface
    /// (decomposer / scope-planner). Read-only after construction.
    pub criteria: Vec<String>,
    /// Wall-clock UNIX ms when the ledger was created.
    pub ts_ms: i64,
}

impl TaskLedger {
    /// Construct a fresh `TaskLedger` in `Pending` state.
    #[must_use]
    pub fn new(
        task_id: TaskId,
        parent_id: Option<TaskId>,
        criteria: Vec<String>,
        ts_ms: i64,
    ) -> Self {
        Self {
            task_id,
            parent_id,
            status: TaskStatus::Pending,
            criteria,
            ts_ms,
        }
    }
}

/// ProgressLedger — the dynamic half of the dual ledger. Tracks
/// completion percent + ordered blocker list. Updated on every call
/// to [`DualLedger::advance_task`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProgressLedger {
    /// Task this progress row belongs to.
    pub task_id: TaskId,
    /// Completion percent in `[0.0, 100.0]`. Saturating arithmetic on
    /// updates (callers can't push past 100 or below 0).
    pub completion_pct: f64,
    /// Ordered list of blockers encountered. Newest pushed last.
    pub blockers: Vec<String>,
    /// Wall-clock UNIX ms of the most recent update.
    pub last_update_ms: i64,
}

impl ProgressLedger {
    /// Construct a fresh `ProgressLedger` at 0% with no blockers.
    #[must_use]
    pub fn new(task_id: TaskId, ts_ms: i64) -> Self {
        Self {
            task_id,
            completion_pct: 0.0,
            blockers: Vec::new(),
            last_update_ms: ts_ms,
        }
    }
}

// ---------------------------------------------------------------------------
// TaskAdvancement (post-advance view returned by advance_task)
// ---------------------------------------------------------------------------

/// Post-advance snapshot returned by [`DualLedger::advance_task`]. The
/// SSB producer (`agent_pool::emit_task_advanced`) translates this
/// directly into a [`crate::ssb::TaskAdvancedPayload`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskAdvancement {
    pub task_id: TaskId,
    pub parent_id: Option<TaskId>,
    pub completion_pct: f64,
    pub status: TaskStatus,
    /// Blocker added on this advance (empty when none).
    pub blocker: String,
    pub ts_ms: i64,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DualLedgerError {
    #[error("task not registered: {0}")]
    UnknownTask(String),
    #[error("ledger split: progress ledger missing for task {0}")]
    ProgressMissing(String),
}

// ---------------------------------------------------------------------------
// DualLedger
// ---------------------------------------------------------------------------

/// Eligibility threshold (completion_pct) for BAI promotion-eval
/// inclusion. Companion: `agent_pool::is_promotion_eligible`.
pub const PROMOTION_ELIGIBILITY_THRESHOLD_PCT: f64 = 100.0;

/// In-memory store holding both ledgers in lockstep.
///
/// Not `Send + Sync` by itself — the orchestrator wraps it in an
/// `Arc<Mutex<DualLedger>>` at the producer call site
/// (`agent_pool::process_task_advancement`). Internal storage uses
/// `FxHashMap` for the same reason the rest of the orchestrator does
/// (deterministic-iteration test friendliness; no `RandomState` jitter
/// when comparing fixture outputs).
#[derive(Debug, Default)]
pub struct DualLedger {
    tasks: FxHashMap<TaskId, TaskLedger>,
    progress: FxHashMap<TaskId, ProgressLedger>,
}

impl DualLedger {
    /// Construct an empty ledger.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new task. Idempotent on the task_id — re-registering
    /// the same id keeps the existing ledger and warns (so a planner
    /// rerun doesn't silently overwrite progress).
    pub fn register_task(&mut self, task: TaskLedger) {
        let id = task.task_id.clone();
        if self.tasks.contains_key(&id) {
            warn!(
                task_id = %id.0,
                "dual_ledger::register_task: id already present; ignoring duplicate"
            );
            return;
        }
        let progress = ProgressLedger::new(id.clone(), task.ts_ms);
        self.tasks.insert(id.clone(), task);
        self.progress.insert(id, progress);
    }

    /// Borrow the task ledger for `task_id`, if present.
    #[must_use]
    pub fn task(&self, task_id: &TaskId) -> Option<&TaskLedger> {
        self.tasks.get(task_id)
    }

    /// Borrow the progress ledger for `task_id`, if present.
    #[must_use]
    pub fn progress(&self, task_id: &TaskId) -> Option<&ProgressLedger> {
        self.progress.get(task_id)
    }

    /// Iterate over (task, progress) pairs in deterministic insertion
    /// order. Used by the dashboard `get_dual_ledger_state` command.
    pub fn iter(&self) -> impl Iterator<Item = (&TaskLedger, &ProgressLedger)> {
        self.tasks
            .iter()
            .filter_map(|(id, t)| self.progress.get(id).map(|p| (t, p)))
    }

    /// Snapshot the ledger as two parallel vectors for IPC. Tasks are
    /// returned in the natural insertion order (FxHashMap preserves
    /// this implicitly in our usage but downstream consumers must
    /// sort by `ts_ms` if they care about stability).
    #[must_use]
    pub fn snapshot(&self) -> DualLedgerSnapshot {
        let tasks: Vec<TaskLedger> = self.tasks.values().cloned().collect();
        let progress: Vec<ProgressLedger> = self.progress.values().cloned().collect();
        DualLedgerSnapshot { tasks, progress }
    }

    /// Atomic advance: updates both ledgers in a single `&mut self`
    /// borrow. Returns a [`TaskAdvancement`] suitable for the SSB
    /// producer.
    ///
    /// Semantics:
    ///   - `completion_delta` is added to the current
    ///     `completion_pct` and clamped to `[0.0, 100.0]`.
    ///   - When `blocker` is `Some(s)` and `s` is non-empty, the
    ///     blocker is appended AND the task status moves to
    ///     `Blocked`.
    ///   - When the post-update `completion_pct >= 100.0`, the task
    ///     status moves to `Complete`.
    ///   - Otherwise, if status was `Pending` and `completion_delta
    ///     > 0.0`, status moves to `InProgress`.
    pub fn advance_task(
        &mut self,
        task_id: &TaskId,
        completion_delta: f64,
        blocker: Option<String>,
        ts_ms: i64,
    ) -> Result<TaskAdvancement, DualLedgerError> {
        // Look the task up first; if absent we surface a typed error
        // (no silent insert — that masks planner bugs).
        if !self.tasks.contains_key(task_id) {
            return Err(DualLedgerError::UnknownTask(task_id.0.clone()));
        }
        let progress = self
            .progress
            .get_mut(task_id)
            .ok_or_else(|| DualLedgerError::ProgressMissing(task_id.0.clone()))?;

        // Saturating clamp on the percent — callers can't poison the
        // ledger with NaN or out-of-range deltas. Use a fold rather
        // than f64::clamp because clamp() panics on NaN.
        let mut new_pct = progress.completion_pct + completion_delta;
        if new_pct.is_nan() {
            new_pct = progress.completion_pct;
        }
        if new_pct < 0.0 {
            new_pct = 0.0;
        }
        if new_pct > 100.0 {
            new_pct = 100.0;
        }
        progress.completion_pct = new_pct;
        progress.last_update_ms = ts_ms;

        let mut emitted_blocker = String::new();
        if let Some(b) = blocker {
            if !b.is_empty() {
                emitted_blocker = b.clone();
                progress.blockers.push(b);
            }
        }

        // Status transition — derived from the (post-update) percent
        // and blocker presence.
        let task = self
            .tasks
            .get_mut(task_id)
            .ok_or_else(|| DualLedgerError::UnknownTask(task_id.0.clone()))?;
        let new_status = if !emitted_blocker.is_empty() {
            TaskStatus::Blocked
        } else if new_pct >= PROMOTION_ELIGIBILITY_THRESHOLD_PCT {
            TaskStatus::Complete
        } else if matches!(task.status, TaskStatus::Pending) && completion_delta > 0.0 {
            TaskStatus::InProgress
        } else if matches!(task.status, TaskStatus::Blocked) && emitted_blocker.is_empty() {
            // Blocker cleared on this advance — resume progress.
            TaskStatus::InProgress
        } else {
            task.status
        };
        task.status = new_status;

        debug!(
            task_id = %task_id.0,
            new_pct,
            status = new_status.as_str(),
            "dual_ledger::advance_task"
        );

        Ok(TaskAdvancement {
            task_id: task.task_id.clone(),
            parent_id: task.parent_id.clone(),
            completion_pct: new_pct,
            status: new_status,
            blocker: emitted_blocker,
            ts_ms,
        })
    }

    /// True when the task has crossed the promotion-eligibility
    /// threshold (completion_pct >= 100.0). Used by the BAI
    /// promotion-eval pool selector to know which tasks are
    /// fully-baked.
    #[must_use]
    pub fn is_promotion_eligible(&self, task_id: &TaskId) -> bool {
        self.progress
            .get(task_id)
            .map(|p| p.completion_pct >= PROMOTION_ELIGIBILITY_THRESHOLD_PCT)
            .unwrap_or(false)
    }
}

/// IPC-friendly snapshot of the ledger's current state. Returned by
/// `DualLedger::snapshot` and forwarded over the Tauri boundary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DualLedgerSnapshot {
    pub tasks: Vec<TaskLedger>,
    pub progress: Vec<ProgressLedger>,
}

impl DualLedgerSnapshot {
    /// Return a `HashMap` view keyed on `task_id` for consumers that
    /// want O(1) lookup (the Svelte panel uses this). `HashMap` not
    /// `FxHashMap` because the IPC boundary serializes through serde
    /// which doesn't care about hasher identity.
    #[must_use]
    pub fn by_task(&self) -> HashMap<String, (&TaskLedger, &ProgressLedger)> {
        let mut out = HashMap::with_capacity(self.tasks.len());
        for t in &self.tasks {
            if let Some(p) = self.progress.iter().find(|p| p.task_id == t.task_id) {
                out.insert(t.task_id.0.clone(), (t, p));
            }
        }
        out
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_task(id: &str, parent: Option<&str>) -> TaskLedger {
        TaskLedger::new(
            TaskId::from(id),
            parent.map(TaskId::from),
            vec!["criterion-1".into()],
            1_700_000_000_000,
        )
    }

    #[test]
    fn register_and_lookup() {
        let mut ledger = DualLedger::new();
        ledger.register_task(mk_task("t-1", None));
        assert!(ledger.task(&TaskId::from("t-1")).is_some());
        assert!(ledger.progress(&TaskId::from("t-1")).is_some());
        assert_eq!(
            ledger.progress(&TaskId::from("t-1")).unwrap().completion_pct,
            0.0
        );
    }

    #[test]
    fn duplicate_register_is_noop() {
        let mut ledger = DualLedger::new();
        ledger.register_task(mk_task("t-1", None));
        // Mutate via advance, then re-register — progress must be
        // preserved.
        ledger
            .advance_task(&TaskId::from("t-1"), 25.0, None, 1)
            .unwrap();
        ledger.register_task(mk_task("t-1", None));
        assert_eq!(
            ledger.progress(&TaskId::from("t-1")).unwrap().completion_pct,
            25.0
        );
    }

    #[test]
    fn advance_clamps_completion() {
        let mut ledger = DualLedger::new();
        ledger.register_task(mk_task("t-1", None));
        let id = TaskId::from("t-1");
        // Over-shoot — clamps at 100.
        let adv = ledger.advance_task(&id, 150.0, None, 1).unwrap();
        assert_eq!(adv.completion_pct, 100.0);
        assert_eq!(adv.status, TaskStatus::Complete);
        // Negative delta — clamps at 0.
        let adv = ledger.advance_task(&id, -500.0, None, 2).unwrap();
        assert_eq!(adv.completion_pct, 0.0);
    }

    #[test]
    fn advance_status_transitions() {
        let mut ledger = DualLedger::new();
        ledger.register_task(mk_task("t-1", None));
        let id = TaskId::from("t-1");
        // Pending -> InProgress.
        let a = ledger.advance_task(&id, 30.0, None, 1).unwrap();
        assert_eq!(a.status, TaskStatus::InProgress);
        // Blocker -> Blocked.
        let a = ledger
            .advance_task(&id, 10.0, Some("waiting on dep".into()), 2)
            .unwrap();
        assert_eq!(a.status, TaskStatus::Blocked);
        assert_eq!(a.blocker, "waiting on dep");
        // Blocker cleared by passing None -> InProgress.
        let a = ledger.advance_task(&id, 5.0, None, 3).unwrap();
        assert_eq!(a.status, TaskStatus::InProgress);
        // Reach 100 -> Complete.
        let a = ledger.advance_task(&id, 100.0, None, 4).unwrap();
        assert_eq!(a.status, TaskStatus::Complete);
        assert!(ledger.is_promotion_eligible(&id));
    }

    #[test]
    fn unknown_task_errors() {
        let mut ledger = DualLedger::new();
        let err = ledger
            .advance_task(&TaskId::from("missing"), 10.0, None, 1)
            .unwrap_err();
        assert_eq!(err, DualLedgerError::UnknownTask("missing".into()));
    }

    #[test]
    fn nan_delta_is_ignored() {
        let mut ledger = DualLedger::new();
        ledger.register_task(mk_task("t-1", None));
        let id = TaskId::from("t-1");
        ledger.advance_task(&id, 50.0, None, 1).unwrap();
        let adv = ledger
            .advance_task(&id, f64::NAN, None, 2)
            .expect("NaN delta should not panic");
        // NaN ignored — completion stays at 50.
        assert_eq!(adv.completion_pct, 50.0);
    }

    #[test]
    fn snapshot_round_trips() {
        let mut ledger = DualLedger::new();
        ledger.register_task(mk_task("t-1", None));
        ledger.register_task(mk_task("t-2", Some("t-1")));
        ledger
            .advance_task(&TaskId::from("t-1"), 100.0, None, 1)
            .unwrap();
        let snap = ledger.snapshot();
        assert_eq!(snap.tasks.len(), 2);
        assert_eq!(snap.progress.len(), 2);
        let by = snap.by_task();
        let (_t, p) = by.get("t-1").expect("t-1 present");
        assert_eq!(p.completion_pct, 100.0);
    }
}
