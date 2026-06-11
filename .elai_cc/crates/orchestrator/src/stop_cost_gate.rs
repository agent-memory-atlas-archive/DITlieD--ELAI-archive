//! H-38 — cost-confirmation gate predicate for TaskStop / pod_stop.
//!
//! A pure, sync, I/O-free predicate: given a `StopCostSnapshot` returns
//! `StopGateVerdict::Pass` or `StopGateVerdict::ConfirmRequired`.  All SSB
//! writes, gate presentation, and env-flag checks live in the caller
//! (`hook_dispatcher.rs`).  This mirrors the `route_cloud` pure-predicate
//! shape at `cloud_mode.rs:326`.
//!
//! Grounding: context-package-H38.md producer block + plan H-38 WIRING_CONTRACT.

/// Minimum day-spend (USD) above which an interactive confirm is required.
///
/// Named const per CLAUDE.md "no magic numbers".  Chosen as $0.10 — small
/// enough to catch real cloud work, high enough to avoid friction on tiny
/// ($0.01) local runs.
pub const CONFIRM_COST_FLOOR_USD: f64 = 0.10;

/// Snapshot of cost / unsaved-progress data captured at stop time.
///
/// Built by `dispatch_stop` from the H-9 CloudSpend ledger and the run's
/// pending-write set; passed into `evaluate_stop` for the pure predicate
/// decision.
#[derive(Debug, Clone, PartialEq)]
pub struct StopCostSnapshot {
    /// Total cloud spend accrued today (UTC day) in USD, from
    /// `SsbViewer::cloud_spend_summary`.  `0.0` when the viewer is absent or
    /// no `cloud_spend` rows exist.
    pub day_usd: f64,
    /// Number of unsaved / pending-write artifacts in the current run.
    /// `0` means fully saved.
    pub unsaved_artifacts: usize,
    /// Whether a billable pod is currently running.  Always `false` in the
    /// ELAI Rust layer today (pod lifecycle is a harness MCP concern); the
    /// field is present so the harness half can pass `true` once wired.
    pub pod_billable: bool,
}

/// Outcome of `evaluate_stop`.
#[derive(Debug, Clone, PartialEq)]
pub enum StopGateVerdict {
    /// Stop may proceed without interactive confirmation.
    Pass,
    /// Interactive confirmation is required before the stop proceeds.
    ConfirmRequired {
        /// The day-spend figure to surface in the confirm prompt (USD).
        cost_usd: f64,
        /// Number of unsaved artifacts (for the prompt).
        unsaved_count: usize,
        /// Human-readable reason string (for logs / hook finding message).
        reason: String,
    },
}

/// Evaluate whether an interactive confirm is required before a stop proceeds.
///
/// Pure + sync: no I/O, no env reads, no SSB writes.  The caller is
/// responsible for presenting the gate and emitting the SSB beacon.
///
/// Rules (OR'd — any true => `ConfirmRequired`):
///   * `day_usd > CONFIRM_COST_FLOOR_USD`  — real cloud money was spent today
///   * `unsaved_artifacts > 0`             — pending writes would be lost
///   * `pod_billable`                      — a running billable pod is present
#[must_use]
pub fn evaluate_stop(snapshot: StopCostSnapshot) -> StopGateVerdict {
    let cost_trigger = snapshot.day_usd > CONFIRM_COST_FLOOR_USD;
    let unsaved_trigger = snapshot.unsaved_artifacts > 0;
    let pod_trigger = snapshot.pod_billable;

    if cost_trigger || unsaved_trigger || pod_trigger {
        let mut parts: Vec<&str> = Vec::with_capacity(3);
        if cost_trigger {
            parts.push("nonzero day-spend");
        }
        if unsaved_trigger {
            parts.push("unsaved artifacts");
        }
        if pod_trigger {
            parts.push("billable pod running");
        }
        let reason = parts.join(", ");
        StopGateVerdict::ConfirmRequired {
            cost_usd: snapshot.day_usd,
            unsaved_count: snapshot.unsaved_artifacts,
            reason,
        }
    } else {
        StopGateVerdict::Pass
    }
}

// ---------------------------------------------------------------------------
// Unit tests (pure predicate — no I/O required)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pass_when_zero_spend_zero_unsaved_no_pod() {
        let verdict = evaluate_stop(StopCostSnapshot {
            day_usd: 0.0,
            unsaved_artifacts: 0,
            pod_billable: false,
        });
        assert_eq!(verdict, StopGateVerdict::Pass);
    }

    #[test]
    fn pass_when_spend_at_floor_exactly() {
        // Floor is exclusive (>), so exactly at floor = Pass.
        let verdict = evaluate_stop(StopCostSnapshot {
            day_usd: CONFIRM_COST_FLOOR_USD,
            unsaved_artifacts: 0,
            pod_billable: false,
        });
        assert_eq!(verdict, StopGateVerdict::Pass);
    }

    #[test]
    fn confirm_required_when_spend_above_floor() {
        let verdict = evaluate_stop(StopCostSnapshot {
            day_usd: 5.00,
            unsaved_artifacts: 0,
            pod_billable: false,
        });
        match verdict {
            StopGateVerdict::ConfirmRequired { cost_usd, .. } => {
                // cost_usd must carry the actual spend figure.
                assert!((cost_usd - 5.00).abs() < f64::EPSILON);
            }
            StopGateVerdict::Pass => panic!("expected ConfirmRequired for day_usd=5.00"),
        }
    }

    #[test]
    fn confirm_required_when_unsaved_artifacts() {
        let verdict = evaluate_stop(StopCostSnapshot {
            day_usd: 0.0,
            unsaved_artifacts: 2,
            pod_billable: false,
        });
        match verdict {
            StopGateVerdict::ConfirmRequired { unsaved_count, .. } => {
                assert_eq!(unsaved_count, 2);
            }
            StopGateVerdict::Pass => panic!("expected ConfirmRequired for unsaved=2"),
        }
    }

    #[test]
    fn confirm_required_when_pod_billable() {
        let verdict = evaluate_stop(StopCostSnapshot {
            day_usd: 0.0,
            unsaved_artifacts: 0,
            pod_billable: true,
        });
        assert!(matches!(verdict, StopGateVerdict::ConfirmRequired { .. }));
    }

    #[test]
    fn reason_contains_all_active_triggers() {
        let verdict = evaluate_stop(StopCostSnapshot {
            day_usd: 1.50,
            unsaved_artifacts: 3,
            pod_billable: true,
        });
        match verdict {
            StopGateVerdict::ConfirmRequired { reason, .. } => {
                assert!(reason.contains("nonzero day-spend"), "reason: {reason}");
                assert!(reason.contains("unsaved artifacts"), "reason: {reason}");
                assert!(reason.contains("billable pod running"), "reason: {reason}");
            }
            StopGateVerdict::Pass => panic!("expected ConfirmRequired"),
        }
    }
}
