//! E-39 -- HARD ceiling mid-task abort + per-role caps + layered caps.
//!
//! The C-5 cloud-mode router (crates/orchestrator/src/cloud_mode.rs)
//! currently surfaces `EscalateToCloud { cap_reached = true }` on a
//! soft-cap breach. E-39 layers a HARD cooperative abort on top:
//! * the in-flight cloud call is allowed to COMPLETE so its already-
//!   paid token cost is banked (avoids losing paid output)
//! * the next pending dispatch is refused
//! * the task is ABORTED with a partial-result envelope marked
//!   `aborted_by_spend_cap = true`
//! * a [`crate::ssb::CostEventKind::TaskAbortedBySpendCap`] row is
//!   written carrying `spent_usd_cents` / `cap_usd_cents` /
//!   `reason` / `last_action_type`
//!
//! Per-role caps:
//! * The plan's [`crate::cost::CostRoleBucket`] enum partitions spend
//!   into executor / auditor / scope_planner / memory_writer /
//!   safety_classifier / doc_parser buckets. Default: auditor 2x
//!   executor (DECISION-3 open-question default; revise on first-
//!   session feedback).
//! * Each bucket has its own [`PerRoleCapsConfig`] entry; the abort
//!   decision consults the BUCKET-SPECIFIC cap, not just the task
//!   total.
//!
//! Layered caps (rocketedge.com 2026 schema per plan Task 10):
//! * `soft_alert_usd`  -- default $50/day; emit warn row, allow call
//! * `hard_cutoff_usd` -- default $100/day; abort task
//! * `monthly_manual_override_usd` -- default $1000/month; requires
//!   typed-confirmation via `elai cloud override-cap` CLI subcommand;
//!   emits [`crate::ssb::CostEventKind::LayeredCapManualOverride`]
//!   carrying user_session_id + override_amount + confirmation_string
//!   (audit-trail composition with E-38 EU AI Act liveness gate).

use rustc_hash::FxHashMap;

use super::CostRoleBucket;

/// Why the task aborted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskAbortReason {
    /// Mid-task `max_spend_per_task` ceiling exceeded.
    SpendCapBreached,
    /// Rolling-window velocity cap breached.
    VelocityCapBreached,
    /// Daily layered hard cutoff exceeded.
    LayeredHardCutoffExceeded,
    /// Per-role cap exceeded for the active agent role.
    PerRoleCapBreached,
}

impl TaskAbortReason {
    /// Snake-case wire label for SSB details_json + dashboard rendering.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SpendCapBreached => "spend_cap_breached",
            Self::VelocityCapBreached => "velocity_cap_breached",
            Self::LayeredHardCutoffExceeded => "layered_hard_cutoff_exceeded",
            Self::PerRoleCapBreached => "per_role_cap_breached",
        }
    }
}

/// Caller decision returned by [`abort_task_on_cap_breach`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AbortDecision {
    /// Spend is within limits; keep running.
    Continue,
    /// Mid-task abort triggered; the cloud-dispatch path drops pending
    /// calls and emits the SSB row with the supplied reason + counters.
    Abort {
        reason: TaskAbortReason,
        spent_usd_cents: u64,
        cap_usd_cents: u64,
        role: Option<CostRoleBucket>,
    },
}

/// Per-role caps. Missing entries fall through to the task-level cap.
#[derive(Debug, Clone, Default)]
pub struct PerRoleCapsConfig {
    pub caps_usd_cents: FxHashMap<CostRoleBucket, u64>,
}

impl PerRoleCapsConfig {
    /// Plan Open Question default proposal: auditor 2x executor.
    /// `task_cap_cents` is the task-level ceiling that the per-role
    /// budgets fan out from; auditor gets 2/6 of it, executor 1/6,
    /// scope_planner 1/6, memory_writer 1/6, safety_classifier 0.5/6,
    /// doc_parser 0.5/6 -- summing to the task cap.
    #[must_use]
    pub fn default_proposal(task_cap_cents: u64) -> Self {
        let mut caps_usd_cents = FxHashMap::default();
        let unit = task_cap_cents / 6;
        caps_usd_cents.insert(CostRoleBucket::Executor, unit);
        caps_usd_cents.insert(CostRoleBucket::Auditor, unit.saturating_mul(2));
        caps_usd_cents.insert(CostRoleBucket::ScopePlanner, unit);
        caps_usd_cents.insert(CostRoleBucket::MemoryWriter, unit);
        caps_usd_cents.insert(CostRoleBucket::SafetyClassifier, unit / 2);
        caps_usd_cents.insert(CostRoleBucket::DocParser, unit / 2);
        Self { caps_usd_cents }
    }
}

/// Layered caps (rocketedge.com 2026 schema).
#[derive(Debug, Clone, Copy)]
pub struct LayeredCapsConfig {
    pub soft_alert_usd_cents: u64,
    pub hard_cutoff_usd_cents: u64,
    pub monthly_manual_override_usd_cents: u64,
}

impl Default for LayeredCapsConfig {
    fn default() -> Self {
        Self {
            soft_alert_usd_cents: 5_000,
            hard_cutoff_usd_cents: 10_000,
            monthly_manual_override_usd_cents: 100_000,
        }
    }
}

/// Task-execution context inputs the abort decider consumes.
#[derive(Debug, Clone)]
pub struct AbortCtx {
    pub task_spend_usd_cents: u64,
    pub max_spend_per_task_cents: u64,
    pub task_spend_by_role: FxHashMap<CostRoleBucket, u64>,
    pub active_role: Option<CostRoleBucket>,
    pub day_spend_usd_cents: u64,
    pub per_role_caps: PerRoleCapsConfig,
    pub layered_caps: LayeredCapsConfig,
    pub velocity_breach_present: bool,
}

/// Decide whether to abort the task. Designed to be called AFTER the
/// most recent cloud call completes (cooperative-abort discipline:
/// drain the in-flight call's already-paid cost before deciding).
///
/// Precedence: velocity breach > task ceiling > layered hard cutoff >
/// per-role cap. Soft-alert thresholds do NOT abort; they fall
/// through to `Continue` (callers emit a separate warn row).
#[must_use]
pub fn abort_task_on_cap_breach(ctx: &AbortCtx) -> AbortDecision {
    if ctx.velocity_breach_present {
        return AbortDecision::Abort {
            reason: TaskAbortReason::VelocityCapBreached,
            spent_usd_cents: ctx.task_spend_usd_cents,
            cap_usd_cents: ctx.max_spend_per_task_cents,
            role: ctx.active_role,
        };
    }
    if ctx.task_spend_usd_cents >= ctx.max_spend_per_task_cents
        && ctx.max_spend_per_task_cents > 0
    {
        return AbortDecision::Abort {
            reason: TaskAbortReason::SpendCapBreached,
            spent_usd_cents: ctx.task_spend_usd_cents,
            cap_usd_cents: ctx.max_spend_per_task_cents,
            role: ctx.active_role,
        };
    }
    if ctx.day_spend_usd_cents >= ctx.layered_caps.hard_cutoff_usd_cents
        && ctx.layered_caps.hard_cutoff_usd_cents > 0
    {
        return AbortDecision::Abort {
            reason: TaskAbortReason::LayeredHardCutoffExceeded,
            spent_usd_cents: ctx.day_spend_usd_cents,
            cap_usd_cents: ctx.layered_caps.hard_cutoff_usd_cents,
            role: ctx.active_role,
        };
    }
    if let Some(role) = ctx.active_role {
        if let Some(role_spent) = ctx.task_spend_by_role.get(&role).copied() {
            if let Some(role_cap) = ctx.per_role_caps.caps_usd_cents.get(&role).copied() {
                if role_spent >= role_cap && role_cap > 0 {
                    return AbortDecision::Abort {
                        reason: TaskAbortReason::PerRoleCapBreached,
                        spent_usd_cents: role_spent,
                        cap_usd_cents: role_cap,
                        role: Some(role),
                    };
                }
            }
        }
    }
    AbortDecision::Continue
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_ctx() -> AbortCtx {
        AbortCtx {
            task_spend_usd_cents: 0,
            max_spend_per_task_cents: 1_000,
            task_spend_by_role: FxHashMap::default(),
            active_role: None,
            day_spend_usd_cents: 0,
            per_role_caps: PerRoleCapsConfig::default(),
            layered_caps: LayeredCapsConfig::default(),
            velocity_breach_present: false,
        }
    }

    #[test]
    fn continues_when_no_breach() {
        let d = abort_task_on_cap_breach(&baseline_ctx());
        assert_eq!(d, AbortDecision::Continue);
    }

    #[test]
    fn aborts_on_velocity_breach_first() {
        let mut c = baseline_ctx();
        c.velocity_breach_present = true;
        c.task_spend_usd_cents = 2_000;
        match abort_task_on_cap_breach(&c) {
            AbortDecision::Abort { reason, .. } => {
                assert_eq!(reason, TaskAbortReason::VelocityCapBreached);
            }
            other => panic!("expected velocity abort, got {other:?}"),
        }
    }

    #[test]
    fn aborts_on_task_ceiling() {
        let mut c = baseline_ctx();
        c.task_spend_usd_cents = 1_500;
        c.max_spend_per_task_cents = 1_000;
        match abort_task_on_cap_breach(&c) {
            AbortDecision::Abort {
                reason,
                spent_usd_cents,
                cap_usd_cents,
                ..
            } => {
                assert_eq!(reason, TaskAbortReason::SpendCapBreached);
                assert_eq!(spent_usd_cents, 1_500);
                assert_eq!(cap_usd_cents, 1_000);
            }
            other => panic!("expected task ceiling abort, got {other:?}"),
        }
    }

    #[test]
    fn aborts_on_layered_hard_cutoff() {
        let mut c = baseline_ctx();
        c.max_spend_per_task_cents = 100_000;
        c.day_spend_usd_cents = 11_000;
        // default hard_cutoff_usd_cents = 10_000
        match abort_task_on_cap_breach(&c) {
            AbortDecision::Abort { reason, .. } => {
                assert_eq!(reason, TaskAbortReason::LayeredHardCutoffExceeded);
            }
            other => panic!("expected layered hard cutoff abort, got {other:?}"),
        }
    }

    #[test]
    fn aborts_on_per_role_cap() {
        let mut c = baseline_ctx();
        c.max_spend_per_task_cents = 10_000;
        c.active_role = Some(CostRoleBucket::Executor);
        let mut by_role = FxHashMap::default();
        by_role.insert(CostRoleBucket::Executor, 1_500);
        c.task_spend_by_role = by_role;
        let mut caps = PerRoleCapsConfig::default();
        caps.caps_usd_cents
            .insert(CostRoleBucket::Executor, 1_000);
        c.per_role_caps = caps;
        match abort_task_on_cap_breach(&c) {
            AbortDecision::Abort {
                reason,
                role,
                ..
            } => {
                assert_eq!(reason, TaskAbortReason::PerRoleCapBreached);
                assert_eq!(role, Some(CostRoleBucket::Executor));
            }
            other => panic!("expected per-role cap abort, got {other:?}"),
        }
    }

    #[test]
    fn default_proposal_auditor_double_executor() {
        let caps = PerRoleCapsConfig::default_proposal(60_000);
        let exec = *caps
            .caps_usd_cents
            .get(&CostRoleBucket::Executor)
            .expect("executor");
        let aud = *caps
            .caps_usd_cents
            .get(&CostRoleBucket::Auditor)
            .expect("auditor");
        assert_eq!(aud, exec * 2);
    }
}
