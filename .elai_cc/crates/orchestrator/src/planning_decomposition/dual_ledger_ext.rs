//! Magentic-One {facts, guesses, plan} extension layered atop the shipped
//! [`crate::dual_ledger::TaskLedger`] / [`crate::dual_ledger::ProgressLedger`].
//!
//! Plan task 1 of E-46. The shipped [`crate::dual_ledger`] (s-62376,
//! 2026-05-20) implements a leaner TaskLedger {task_id, parent_id, status,
//! criteria, ts_ms} / ProgressLedger {task_id, completion_pct, blockers,
//! last_update_ms} pair that wires into the BAI promotion gate via
//! `dual_ledger::PROMOTION_ELIGIBILITY_THRESHOLD_PCT = 100.0`. The
//! cross-check re-audit flagged that pair as INCOMPATIBLE with the
//! plan-required Magentic-One spec-shape:
//!
//!   - `TaskLedger { facts: Vec<LedgerFact>, guesses: Vec<LedgerGuess>,
//!     plan: Vec<PlanStep> }`
//!   - `ProgressLedger { current_node: NodeId, assignment: AgentRole,
//!     blockers: Vec<Blocker> }`
//!
//! Choice: ADDITIVE layering. The shipped structs are kept as-is so the
//! 4 downstream call sites (`agent_pool::process_task_advancement`,
//! `ledger_snapshot`, `promotion_eligible_tasks`, `goal_drift_blocks_promotion`)
//! continue to compile unchanged. The Magentic-One spec-shape lives in
//! this module as a SIDECAR set of structs that hang off the same
//! `TaskId` via the in-memory [`MagenticLedgerStore`] — the
//! agent-pool can opt in by writing to the sidecar store under the
//! [`crate::planning_decomposition::PLANNING_DECOMPOSITION_ENABLE_ENV`]
//! gate without touching the canonical promotion path.
//!
//! Rationale documented in `workflow/reasoning/E-46-decisions.md` GAP-A
//! (TaskLedger spec-shape rewrite vs additive layering — additive chosen
//! because the alternative breaks 4+ downstream consumers and risks the
//! D-7 canary promotion-gate composition seam invariant
//! `PROMOTION_ELIGIBILITY_THRESHOLD_PCT == 100.0`).
//!
//! Reference: arXiv:2411.04468 Magentic-One dual-ledger orchestrator.

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::debug;

use crate::dual_ledger::TaskId;

/// One declarative fact known about a task. Magentic-One distinguishes
/// `facts` (verified) from `guesses` (unverified hypotheses) so the
/// orchestrator can replan on `facts`-changed without invalidating
/// `guesses`-changed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerFact {
    /// Free-form fact text.
    pub text: String,
    /// Optional evidence ref (file:line, SSB seq, etc).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_ref: Option<String>,
    /// Wall-clock UNIX ms when the fact was recorded.
    pub recorded_at_ms: i64,
}

/// One unverified hypothesis about a task. Distinct from a [`LedgerFact`]
/// so the replan trigger can fire on fact-changed without churning on
/// every guess-revision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerGuess {
    /// Free-form guess text.
    pub text: String,
    /// Confidence in [0, 100] (integer percent for serde stability;
    /// callers convert from `f32 in [0,1]` at the boundary).
    pub confidence_pct: u8,
    /// Wall-clock UNIX ms when the guess was recorded.
    pub recorded_at_ms: i64,
}

/// One step in the planned solution sequence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanStep {
    /// Step ordinal in the plan (0-based).
    pub ord: u32,
    /// Free-form description of the planned action.
    pub description: String,
    /// `assignment` mirrors the ProgressLedger.assignment field so a
    /// PlanStep can be inspected independently — same canonical agent-role
    /// label as the rest of the orchestrator.
    pub assignment: AgentRoleLabel,
    /// Wall-clock UNIX ms when the step was recorded.
    pub recorded_at_ms: i64,
}

/// Static-string agent-role label mirrored from
/// [`crate::pipeline_tree_navigator`] / `auditor_mode::AgentRole`. Kept
/// stringy so this module does not depend on the orchestrator's full
/// agent-role enum (avoids circular module dependency).
///
/// CONTRACT: callers use the canonical labels:
/// `executor` / `auditor` / `scope_planner` / `context_compiler` /
/// `memory_writer` / `prompt_engineer` / `synthesizer` /
/// `integration_explorer` / `impact_mapper` / `paper_ingestor` /
/// `fixer` / `test_attributer` / `wiring_contract_validator` /
/// `sidecar_health` / `dashboard_smoker`. Unrecognized labels are stored
/// verbatim; the contract test asserts the canonical set is accepted.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AgentRoleLabel(pub String);

impl AgentRoleLabel {
    /// Construct a canonical label.
    #[must_use]
    pub fn new(label: &str) -> Self {
        Self(label.to_string())
    }

    /// Borrow the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One rich-shape blocker (Magentic-One ProgressLedger.blockers element).
/// Carries a structured reason + the originating SSB row seq so the
/// replan handler can dereference back to evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Blocker {
    /// Free-form reason text.
    pub reason: String,
    /// SSB row seq if available (`-1` when unknown — callers should
    /// always supply a real seq when possible).
    pub ssb_seq: i64,
    /// Wall-clock UNIX ms when the blocker was recorded.
    pub recorded_at_ms: i64,
}

/// Magentic-One-shape sidecar TaskLedger living atop the canonical
/// [`crate::dual_ledger::TaskLedger`]. The two are linked by `task_id`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct MagenticTaskLedger {
    pub facts: Vec<LedgerFact>,
    pub guesses: Vec<LedgerGuess>,
    pub plan: Vec<PlanStep>,
}

/// Magentic-One-shape sidecar ProgressLedger living atop the canonical
/// [`crate::dual_ledger::ProgressLedger`]. Linked by `task_id`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct MagenticProgressLedger {
    /// Current node identifier (`{Concept,Phase,Step,BabyStep}Id` rendered
    /// as a stringy newtype via `format!`; callers stringify at insert).
    pub current_node: String,
    /// Agent currently assigned to drive this task forward.
    pub assignment: AgentRoleLabel,
    /// Ordered list of structured blockers.
    pub blockers: Vec<Blocker>,
}

/// Threshold at which [`MagenticProgressLedger::blockers.len`] is
/// considered grounds for the `LEDGER_RESET` bubble-up cascade (per plan
/// task 1 §37 default). Surface in `elai_cc.toml`
/// `[planning.ledger] reset_threshold` to override.
pub const LEDGER_RESET_BLOCKER_THRESHOLD: usize = 3;

/// In-memory store mapping [`TaskId`] to (MagenticTaskLedger,
/// MagenticProgressLedger). Sync `&mut self` mutation; wrap in a lock
/// for concurrent access. The store is wire-dark by construction: only
/// the producer call sites in
/// [`crate::planning_decomposition::producers`] write to it under the
/// env gate.
#[derive(Debug, Clone, Default)]
pub struct MagenticLedgerStore {
    rows: FxHashMap<TaskId, (MagenticTaskLedger, MagenticProgressLedger)>,
}

impl MagenticLedgerStore {
    /// New empty store.
    #[must_use]
    pub fn new() -> Self {
        Self {
            rows: FxHashMap::default(),
        }
    }

    /// Insert or replace the row for `task_id`.
    pub fn upsert(
        &mut self,
        task_id: TaskId,
        task: MagenticTaskLedger,
        progress: MagenticProgressLedger,
    ) {
        self.rows.insert(task_id, (task, progress));
    }

    /// Borrow the row for `task_id` if present.
    pub fn get(&self, task_id: &TaskId) -> Option<&(MagenticTaskLedger, MagenticProgressLedger)> {
        self.rows.get(task_id)
    }

    /// Mutably borrow the row for `task_id`, creating an empty row on
    /// first touch. Returns a mutable reference to the freshly-inserted-or-
    /// existing pair.
    pub fn get_or_insert(
        &mut self,
        task_id: TaskId,
    ) -> &mut (MagenticTaskLedger, MagenticProgressLedger) {
        self.rows
            .entry(task_id)
            .or_insert_with(|| (MagenticTaskLedger::default(), MagenticProgressLedger::default()))
    }

    /// Append a blocker to the progress ledger for `task_id`. Creates the
    /// row on first touch. Returns the post-append blocker count so the
    /// caller can compare against [`LEDGER_RESET_BLOCKER_THRESHOLD`].
    pub fn append_blocker(&mut self, task_id: TaskId, blocker: Blocker) -> usize {
        let row = self.get_or_insert(task_id);
        row.1.blockers.push(blocker);
        let len = row.1.blockers.len();
        debug!(
            target: "planning.dual_ledger_ext",
            blocker_count = len,
            "appended blocker to MagenticProgressLedger"
        );
        len
    }

    /// Number of rows currently stored.
    #[must_use]
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// True iff empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Returns true iff the blocker count for `task_id` has crossed
    /// [`LEDGER_RESET_BLOCKER_THRESHOLD`] and a `LEDGER_RESET` cascade
    /// should be emitted.
    pub fn should_emit_ledger_reset(&self, task_id: &TaskId, threshold: usize) -> bool {
        let threshold = threshold.max(1);
        self.rows
            .get(task_id)
            .is_some_and(|(_t, p)| p.blockers.len() >= threshold)
    }
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum MagenticLedgerError {
    #[error("plan step ordinal {0} is not strictly monotonic — last ord was {1}")]
    PlanStepOrdNotMonotonic(u32, u32),
}

impl MagenticTaskLedger {
    /// Append a plan step, enforcing strict-monotonic ord.
    pub fn push_plan_step(&mut self, step: PlanStep) -> Result<(), MagenticLedgerError> {
        if let Some(last) = self.plan.last() {
            if step.ord <= last.ord {
                return Err(MagenticLedgerError::PlanStepOrdNotMonotonic(step.ord, last.ord));
            }
        }
        self.plan.push(step);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_task() -> TaskId {
        TaskId::from("task-1")
    }

    #[test]
    fn store_empty_on_new() {
        let s = MagenticLedgerStore::new();
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
        assert!(s.get(&mk_task()).is_none());
    }

    #[test]
    fn store_upsert_replaces_row() {
        let mut s = MagenticLedgerStore::new();
        let t = mk_task();
        s.upsert(
            t.clone(),
            MagenticTaskLedger::default(),
            MagenticProgressLedger {
                current_node: "node-a".into(),
                assignment: AgentRoleLabel::new("executor"),
                blockers: vec![],
            },
        );
        assert_eq!(s.get(&t).unwrap().1.current_node, "node-a");
        s.upsert(
            t.clone(),
            MagenticTaskLedger::default(),
            MagenticProgressLedger {
                current_node: "node-b".into(),
                assignment: AgentRoleLabel::new("auditor"),
                blockers: vec![],
            },
        );
        assert_eq!(s.get(&t).unwrap().1.current_node, "node-b");
    }

    #[test]
    fn append_blocker_increments_count() {
        let mut s = MagenticLedgerStore::new();
        let t = mk_task();
        let count = s.append_blocker(
            t.clone(),
            Blocker {
                reason: "test failure".into(),
                ssb_seq: 100,
                recorded_at_ms: 1,
            },
        );
        assert_eq!(count, 1);
        let count = s.append_blocker(
            t.clone(),
            Blocker {
                reason: "audit block".into(),
                ssb_seq: 101,
                recorded_at_ms: 2,
            },
        );
        assert_eq!(count, 2);
    }

    #[test]
    fn should_emit_ledger_reset_fires_at_threshold() {
        let mut s = MagenticLedgerStore::new();
        let t = mk_task();
        for i in 0..LEDGER_RESET_BLOCKER_THRESHOLD {
            s.append_blocker(
                t.clone(),
                Blocker {
                    reason: format!("b{}", i),
                    ssb_seq: i as i64,
                    recorded_at_ms: i as i64,
                },
            );
        }
        assert!(s.should_emit_ledger_reset(&t, LEDGER_RESET_BLOCKER_THRESHOLD));
    }

    #[test]
    fn should_emit_ledger_reset_not_yet_below_threshold() {
        let mut s = MagenticLedgerStore::new();
        let t = mk_task();
        s.append_blocker(
            t.clone(),
            Blocker {
                reason: "b0".into(),
                ssb_seq: 0,
                recorded_at_ms: 0,
            },
        );
        assert!(!s.should_emit_ledger_reset(&t, LEDGER_RESET_BLOCKER_THRESHOLD));
    }

    #[test]
    fn push_plan_step_enforces_monotonic_ord() {
        let mut tl = MagenticTaskLedger::default();
        tl.push_plan_step(PlanStep {
            ord: 0,
            description: "step 0".into(),
            assignment: AgentRoleLabel::new("scope_planner"),
            recorded_at_ms: 1,
        })
        .unwrap();
        tl.push_plan_step(PlanStep {
            ord: 1,
            description: "step 1".into(),
            assignment: AgentRoleLabel::new("executor"),
            recorded_at_ms: 2,
        })
        .unwrap();
        let err = tl
            .push_plan_step(PlanStep {
                ord: 1,
                description: "duplicate ord".into(),
                assignment: AgentRoleLabel::new("executor"),
                recorded_at_ms: 3,
            })
            .unwrap_err();
        match err {
            MagenticLedgerError::PlanStepOrdNotMonotonic(got, last) => {
                assert_eq!(got, 1);
                assert_eq!(last, 1);
            }
        }
    }

    #[test]
    fn agent_role_label_roundtrip() {
        let l = AgentRoleLabel::new("executor");
        assert_eq!(l.as_str(), "executor");
    }
}
