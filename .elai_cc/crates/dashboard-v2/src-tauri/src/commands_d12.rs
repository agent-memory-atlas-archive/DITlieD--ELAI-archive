//! D-12 — OperationalizationCheck baby-step predicate Tauri command surface.
//!
//! Single read-only command feeds the Svelte `BabyStepPanel`:
//!
//! - [`get_baby_step_dispatches_recent`] — last N
//!   `baby_step_dispatched` SSB rows + per-outcome aggregate
//!   (pass / fail / inapplicable). The orchestrator producer
//!   (`elai_orchestrator::run_operational_check`) is the sole writer;
//!   this module only reads.
//!
//! Sibling of `commands_d10` (mode-routing), `commands_d11`
//! (constructive checkers), and `commands_d13` (adversary). The panel
//! groups recent dispatches by parent_task_id so the user can see
//! whether a whole task family is failing OperationalizationCheck.
//!
//! See docs/thefinalglance/D-plan/D-12_operationalization_check_predicate.md.
//!
//! Grounding: arxiv:2305.16291 Voyager (T1) skill-with-measurable-effect;
//! arxiv:2407.01489 AGENTLESS (T1) single-step verification.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 500;

/// Typed mirror of `elai_orchestrator::BabyStepDispatchedRow`. Lives
/// in this crate so the Tauri-side schema does not leak the
/// orchestrator's internal type through the IPC boundary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BabyStepDispatchedRowDto {
    pub step_id: String,
    pub parent_task_id: String,
    pub checker_kinds_used: Vec<String>,
    pub outcome: String,
    pub ts_ms: i64,
}

/// Aggregated response: recent rows + per-outcome counts + per-kind
/// counts so the panel can render two histograms (outcome / checker
/// kind) without re-iterating the row vector on the client side.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecentBabyStepDispatches {
    pub rows: Vec<BabyStepDispatchedRowDto>,
    /// Snake-case outcome label -> count within `rows`. Sorted-key
    /// `BTreeMap` so the JSON output is deterministic for snapshot
    /// tests. Pre-seeded with `pass / fail / inapplicable` at zero.
    pub counts_by_outcome: BTreeMap<String, u32>,
    /// Snake-case checker kind label -> count of dispatches that
    /// fired that kind. A single dispatch can contribute to multiple
    /// kinds (when `checker_kinds_used` has more than one entry).
    pub counts_by_kind: BTreeMap<String, u32>,
    /// Total row count (== `rows.len()`).
    pub total: u32,
}

/// Return the most recent `baby_step_dispatched` rows (capped at
/// `MAX_LIMIT`) + per-outcome and per-kind aggregations. Empty
/// response when no SSB viewer is attached (Tauri-dev fresh boot, no
/// DB).
#[tauri::command]
pub async fn get_baby_step_dispatches_recent(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<RecentBabyStepDispatches, String> {
    let cap = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => {
            return Ok(RecentBabyStepDispatches {
                rows: Vec::new(),
                counts_by_outcome: seeded_outcome_counts(),
                counts_by_kind: BTreeMap::new(),
                total: 0,
            });
        }
    };

    let rows: Vec<BabyStepDispatchedRowDto> =
        elai_orchestrator::recent_baby_step_dispatches(&viewer, cap)
            .into_iter()
            .map(|r| BabyStepDispatchedRowDto {
                step_id: r.step_id,
                parent_task_id: r.parent_task_id,
                checker_kinds_used: r.checker_kinds_used,
                outcome: r.outcome,
                ts_ms: r.ts_ms,
            })
            .collect();

    let mut counts_by_outcome = seeded_outcome_counts();
    let mut counts_by_kind: BTreeMap<String, u32> = BTreeMap::new();
    for row in &rows {
        *counts_by_outcome.entry(row.outcome.clone()).or_insert(0) += 1;
        for kind in &row.checker_kinds_used {
            *counts_by_kind.entry(kind.clone()).or_insert(0) += 1;
        }
    }

    let total = u32::try_from(rows.len()).unwrap_or(u32::MAX);
    Ok(RecentBabyStepDispatches {
        rows,
        counts_by_outcome,
        counts_by_kind,
        total,
    })
}

/// Seed every known outcome at zero so the panel histogram has a
/// stable column set even when an outcome is empty.
fn seeded_outcome_counts() -> BTreeMap<String, u32> {
    let mut m: BTreeMap<String, u32> = BTreeMap::new();
    for k in ["pass", "fail", "inapplicable"] {
        m.insert(k.into(), 0);
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dto_serializes_with_expected_field_names() {
        let dto = BabyStepDispatchedRowDto {
            step_id: "bs-1".into(),
            parent_task_id: "task-1".into(),
            checker_kinds_used: vec!["numeric_range".into()],
            outcome: "pass".into(),
            ts_ms: 1_700_000_000_000,
        };
        let s = serde_json::to_string(&dto).expect("serialize");
        assert!(s.contains("\"step_id\""));
        assert!(s.contains("\"parent_task_id\""));
        assert!(s.contains("\"checker_kinds_used\""));
        assert!(s.contains("\"outcome\""));
        assert!(s.contains("\"ts_ms\""));
    }

    #[test]
    fn seeded_outcome_counts_has_three_kinds() {
        let m = seeded_outcome_counts();
        assert_eq!(m.len(), 3);
        for k in ["pass", "fail", "inapplicable"] {
            assert_eq!(*m.get(k).unwrap(), 0);
        }
    }
}
