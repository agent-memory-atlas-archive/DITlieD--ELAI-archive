//! E-6 — MAST-grounded multi-agent failure-mode dashboard surface.
//!
//! Two read-only Tauri commands feed the Svelte `MastFailurePanel`:
//!
//! - [`get_mast_breakdown`] — counts of `audit_mast_tagged` SSB rows per
//!   (MastMode, AgentRole) within an optional session filter, plus the
//!   most recent N rows for the timeline strip.
//! - [`get_iso_budget_recent`] — most recent N `iso_budget_measured`
//!   rows for the ablation-result drawer.
//!
//! Additive — sibling of `commands_d13` (adversary panel). The orchestrator
//! producer (per_level_audit_dispatch follow-up + iso_budget_harness +
//! persona_anchor) is the sole writer of the underlying SSB rows; this
//! module only reads.
//!
//! Grounding: arXiv:2503.13657 (Cemri et al. 2025, MAST). Original plan
//! cite 2406.12624 was an ID_COLLISION (Judging the Judges); substituted
//! per workflow/reasoning/E-6-decisions.md DECISION-1 + gotchas.md
//! E6_CITATION_2406_12624_ID_COLLISION.
//!
//! See docs/thefinalglance/E-plan/E-6_multi_agent_failure_modes.md.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 100;
const MAX_LIMIT: usize = 1000;

/// One (mast_mode, agent_role) -> count row, surfaced by
/// [`get_mast_breakdown`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MastBreakdownRow {
    /// Wire-stable MAST tag (`fm_1_1_disobey_task_spec`, etc).
    pub mast_mode: String,
    /// AgentRole snake_case tag.
    pub agent_role: String,
    /// Count of `audit_mast_tagged` rows matching this (mode, role) pair
    /// in the queried session window.
    pub count: u32,
}

/// Per-MAST-mode aggregated count across all agent roles. The dashboard
/// panel renders one column per category and one row per mode; this is
/// the column-sum projection used for the panel header strip.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MastModeTotal {
    /// Wire-stable MAST tag.
    pub mast_mode: String,
    /// Sum of `count` across all agent_roles for this mode.
    pub total: u32,
}

/// Aggregated response for the panel. The breakdown table is the
/// (mode, role, count) grid; `totals` is the sum-by-mode projection
/// used for the category-band header.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MastBreakdownResponse {
    /// Per (mast_mode, agent_role) rows. Sorted by mast_mode then
    /// agent_role for stable PR diffs.
    pub rows: Vec<MastBreakdownRow>,
    /// Per-mode aggregated totals. Sorted by canonical MastMode
    /// declaration order (FM-1.1 first, FM-3.3 last).
    pub totals: Vec<MastModeTotal>,
    /// Total `audit_mast_tagged` rows scanned (sum of all counts).
    pub grand_total: u32,
}

/// Return per-(MastMode, AgentRole) counts of `audit_mast_tagged` SSB
/// rows. Empty response when no SSB viewer is attached (Tauri-dev fresh
/// boot, no DB).
///
/// `limit` caps the underlying SSB scan window at the most recent N
/// rows (default `DEFAULT_LIMIT`, max `MAX_LIMIT`); pass `None` to
/// accept the default.
#[tauri::command]
pub async fn get_mast_breakdown(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<MastBreakdownResponse, String> {
    let cap = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => {
            return Ok(MastBreakdownResponse {
                rows: Vec::new(),
                totals: Vec::new(),
                grand_total: 0,
            });
        }
    };
    // Read via SsbViewer::recent_rows_of_type (existing pattern from
    // commands_d13 / commands_d3). The viewer returns deserialised
    // payload_json values; we project (mast_mode, agent_role) out.
    let raw_rows = match viewer.recent_rows_of_type("audit_mast_tagged", cap) {
        Ok(d) => d,
        Err(e) => {
            tracing::warn!(
                action_type = "audit_mast_tagged",
                error = %e,
                "get_mast_breakdown: viewer error"
            );
            return Ok(MastBreakdownResponse {
                rows: Vec::new(),
                totals: Vec::new(),
                grand_total: 0,
            });
        }
    };

    // Aggregate per (mast_mode, agent_role).
    let mut grid: BTreeMap<(String, String), u32> = BTreeMap::new();
    let mut totals: BTreeMap<String, u32> = BTreeMap::new();
    let mut grand_total: u32 = 0;
    for d in &raw_rows {
        let outer = match d.payload_json.as_object() {
            Some(o) => o,
            None => continue,
        };
        // payload_json may wrap with the AuditMastTagged tag or come flat.
        let obj = if let Some(inner) = outer.get("AuditMastTagged").and_then(|v| v.as_object()) {
            inner.clone()
        } else {
            outer.clone()
        };
        let mast_mode = obj
            .get("mast_mode")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let agent_role = obj
            .get("agent_role")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if mast_mode.is_empty() {
            continue;
        }
        let key = (mast_mode.clone(), agent_role);
        let cell = grid.entry(key).or_insert(0);
        *cell = cell.saturating_add(1);
        let t = totals.entry(mast_mode).or_insert(0);
        *t = t.saturating_add(1);
        grand_total = grand_total.saturating_add(1);
    }

    let rows_dto: Vec<MastBreakdownRow> = grid
        .into_iter()
        .map(|((mast_mode, agent_role), count)| MastBreakdownRow {
            mast_mode,
            agent_role,
            count,
        })
        .collect();
    let totals_dto: Vec<MastModeTotal> = totals
        .into_iter()
        .map(|(mast_mode, total)| MastModeTotal { mast_mode, total })
        .collect();

    Ok(MastBreakdownResponse {
        rows: rows_dto,
        totals: totals_dto,
        grand_total,
    })
}

/// DTO mirror of [`elai_orchestrator::ssb::IsoBudgetMeasuredPayload`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IsoBudgetMeasuredDto {
    pub task_fixture: String,
    pub budget_tokens: u32,
    pub full_pipeline_pass_rate: f64,
    pub single_agent_wait_pass_rate: f64,
    pub insufficient_lift: bool,
    pub runpod_pending: bool,
    pub mast_mode_counts: Vec<MastModeCountDto>,
}

/// Wire-stable mirror of `MastModeCount`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MastModeCountDto {
    pub mast_mode: String,
    pub count: u32,
}

/// Return the most recent N `iso_budget_measured` SSB rows. Default
/// 20; capped at 200.
#[tauri::command]
pub async fn get_iso_budget_recent(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<IsoBudgetMeasuredDto>, String> {
    const DEFAULT: usize = 20;
    const MAX: usize = 200;
    let cap = limit.unwrap_or(DEFAULT).min(MAX);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(Vec::new()),
    };
    let raw_rows = match viewer.recent_rows_of_type("iso_budget_measured", cap) {
        Ok(d) => d,
        Err(e) => {
            tracing::warn!(
                action_type = "iso_budget_measured",
                error = %e,
                "get_iso_budget_recent: viewer error"
            );
            return Ok(Vec::new());
        }
    };

    let mut out: Vec<IsoBudgetMeasuredDto> = Vec::new();
    for d in raw_rows {
        let outer = match d.payload_json.as_object() {
            Some(o) => o,
            None => continue,
        };
        let obj = if let Some(inner) = outer.get("IsoBudgetMeasured").and_then(|v| v.as_object()) {
            inner.clone()
        } else {
            outer.clone()
        };
        let task_fixture = obj
            .get("task_fixture")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let budget_tokens = obj
            .get("budget_tokens")
            .and_then(|v| v.as_u64())
            .map(|n| u32::try_from(n).unwrap_or(0))
            .unwrap_or(0);
        let full_rate = obj
            .get("full_pipeline_pass_rate")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let single_rate = obj
            .get("single_agent_wait_pass_rate")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let insuf = obj
            .get("insufficient_lift")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let runpod = obj
            .get("runpod_pending")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let mut counts: Vec<MastModeCountDto> = Vec::new();
        if let Some(arr) = obj.get("mast_mode_counts").and_then(|v| v.as_array()) {
            for c in arr {
                if let Some(co) = c.as_object() {
                    counts.push(MastModeCountDto {
                        mast_mode: co
                            .get("mast_mode")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        count: co
                            .get("count")
                            .and_then(|v| v.as_u64())
                            .map(|n| u32::try_from(n).unwrap_or(0))
                            .unwrap_or(0),
                    });
                }
            }
        }
        out.push(IsoBudgetMeasuredDto {
            task_fixture,
            budget_tokens,
            full_pipeline_pass_rate: full_rate,
            single_agent_wait_pass_rate: single_rate,
            insufficient_lift: insuf,
            runpod_pending: runpod,
            mast_mode_counts: counts,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breakdown_response_serializes_with_expected_field_names() {
        let r = MastBreakdownResponse {
            rows: vec![MastBreakdownRow {
                mast_mode: "fm_1_3_step_repetition".into(),
                agent_role: "executor".into(),
                count: 4,
            }],
            totals: vec![MastModeTotal {
                mast_mode: "fm_1_3_step_repetition".into(),
                total: 4,
            }],
            grand_total: 4,
        };
        let s = serde_json::to_string(&r).expect("serialize");
        for key in [
            "\"rows\"",
            "\"totals\"",
            "\"grand_total\"",
            "\"mast_mode\"",
            "\"agent_role\"",
            "\"count\"",
            "\"total\"",
        ] {
            assert!(s.contains(key), "missing key {key} in {s}");
        }
    }

    #[test]
    fn iso_budget_dto_serializes_with_expected_field_names() {
        let d = IsoBudgetMeasuredDto {
            task_fixture: "f".into(),
            budget_tokens: 1000,
            full_pipeline_pass_rate: 0.6,
            single_agent_wait_pass_rate: 0.65,
            insufficient_lift: true,
            runpod_pending: true,
            mast_mode_counts: Vec::new(),
        };
        let s = serde_json::to_string(&d).expect("serialize");
        for key in [
            "\"task_fixture\"",
            "\"budget_tokens\"",
            "\"full_pipeline_pass_rate\"",
            "\"single_agent_wait_pass_rate\"",
            "\"insufficient_lift\"",
            "\"runpod_pending\"",
            "\"mast_mode_counts\"",
        ] {
            assert!(s.contains(key), "missing key {key} in {s}");
        }
    }
}
