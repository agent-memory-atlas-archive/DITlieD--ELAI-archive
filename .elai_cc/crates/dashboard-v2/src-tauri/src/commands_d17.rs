//! D-17 — Honest-refusal SLA monitoring read-only Tauri command
//! surface.
//!
//! Two read-only commands feed the Svelte `RefusalRateGauge` panel:
//!
//! - [`get_refusal_sla_recent`]    — last N joined
//!   `(refusal_rate_snapshot + refusal_sla_breach)` rows for the
//!   gauge's sparkline + last-breach chip.
//! - [`get_refusal_rate_current`]  — convenience accessor returning the
//!   single most-recent `refusal_rate_snapshot` row (`null` when no
//!   snapshot has been emitted yet).
//!
//! Additive — sibling of `commands_d16` (conformal calibration) +
//! `commands_d15` (novelty) + `commands_d18` (observable effect). The
//! orchestrator producer
//! (`elai_orchestrator::refusal_sla::RefusalRateMonitor::emit_snapshot`,
//! reachable via `agent_pool::dispatch_refusal_sla_check`) is the sole
//! writer of the underlying SSB rows; this module only reads.
//!
//! Grounding: arXiv:2503.01345 (UniCR honest refusal — the formal Mode
//! E surface this SLA monitors); docs/specs/guide/real-problem-solving-
//! grounded-2026-05-19.md §4.6.
//!
//! See docs/thefinalglance/D-plan/D-17_honest_refusal_sla.md.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 500;

/// Typed mirror of
/// [`elai_orchestrator::refusal_sla::RefusalRateSnapshotRow`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RefusalRateSnapshotRowDto {
    pub refusal_rate: f64,
    pub window_total: u32,
    pub abstain_count: u32,
    pub health_band: String,
    pub ts_ms: i64,
}

/// Typed mirror of
/// [`elai_orchestrator::refusal_sla::RefusalSlaBreachRow`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RefusalSlaBreachRowDto {
    pub rate: f64,
    pub threshold: f64,
    pub breach_kind: String,
    pub ts_ms: i64,
}

/// Aggregated response: recent snapshot rows + recent breach rows + an
/// `is_breaching` flag (true when the most-recent snapshot's
/// `health_band != "healthy"`) so the panel can render the gauge tint
/// without re-walking the rows in JS.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecentRefusalSla {
    pub snapshot_rows: Vec<RefusalRateSnapshotRowDto>,
    pub breach_rows: Vec<RefusalSlaBreachRowDto>,
    pub is_breaching: bool,
}

/// Return the most recent `refusal_rate_snapshot` + `refusal_sla_breach`
/// rows (capped at `MAX_LIMIT`). Empty response when no SSB viewer is
/// attached (Tauri-dev fresh boot, no DB).
#[tauri::command]
pub async fn get_refusal_sla_recent(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<RecentRefusalSla, String> {
    let cap = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => {
            return Ok(RecentRefusalSla {
                snapshot_rows: Vec::new(),
                breach_rows: Vec::new(),
                is_breaching: false,
            });
        }
    };

    let snapshot_rows: Vec<RefusalRateSnapshotRowDto> =
        elai_orchestrator::refusal_sla::recent_snapshot_rows(&viewer, cap)
            .into_iter()
            .map(|r| RefusalRateSnapshotRowDto {
                refusal_rate: r.refusal_rate,
                window_total: r.window_total,
                abstain_count: r.abstain_count,
                health_band: r.health_band,
                ts_ms: r.ts_ms,
            })
            .collect();

    let breach_rows: Vec<RefusalSlaBreachRowDto> =
        elai_orchestrator::refusal_sla::recent_breach_rows(&viewer, cap)
            .into_iter()
            .map(|r| RefusalSlaBreachRowDto {
                rate: r.rate,
                threshold: r.threshold,
                breach_kind: r.breach_kind,
                ts_ms: r.ts_ms,
            })
            .collect();

    // The producer sorts the rows newest-first; the first entry is the
    // most recent snapshot. "healthy" → not breaching, anything else
    // (empty / below / above) → breaching.
    let is_breaching = snapshot_rows
        .first()
        .map(|s| s.health_band != "healthy")
        .unwrap_or(false);

    Ok(RecentRefusalSla {
        snapshot_rows,
        breach_rows,
        is_breaching,
    })
}

/// Return the single most-recent `refusal_rate_snapshot` row, or `None`
/// when no snapshot has been emitted (or the viewer is not attached).
#[tauri::command]
pub async fn get_refusal_rate_current(
    state: State<'_, AppState>,
) -> Result<Option<RefusalRateSnapshotRowDto>, String> {
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(None),
    };
    let mut rows = elai_orchestrator::refusal_sla::recent_snapshot_rows(&viewer, 1);
    Ok(rows.pop().map(|r| RefusalRateSnapshotRowDto {
        refusal_rate: r.refusal_rate,
        window_total: r.window_total,
        abstain_count: r.abstain_count,
        health_band: r.health_band,
        ts_ms: r.ts_ms,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_dto_serializes_with_expected_field_names() {
        let dto = RefusalRateSnapshotRowDto {
            refusal_rate: 0.5,
            window_total: 100,
            abstain_count: 50,
            health_band: "above".into(),
            ts_ms: 1_700_000_000_000,
        };
        let s = serde_json::to_string(&dto).expect("serialize");
        for key in [
            "\"refusal_rate\"",
            "\"window_total\"",
            "\"abstain_count\"",
            "\"health_band\"",
            "\"ts_ms\"",
        ] {
            assert!(s.contains(key), "missing key {key} in {s}");
        }
    }

    #[test]
    fn breach_dto_serializes_with_expected_field_names() {
        let dto = RefusalSlaBreachRowDto {
            rate: 0.5,
            threshold: 0.2,
            breach_kind: "above".into(),
            ts_ms: 1_700_000_000_000,
        };
        let s = serde_json::to_string(&dto).expect("serialize");
        for key in [
            "\"rate\"",
            "\"threshold\"",
            "\"breach_kind\"",
            "\"ts_ms\"",
        ] {
            assert!(s.contains(key), "missing key {key} in {s}");
        }
    }

    #[test]
    fn aggregated_response_serializes_with_expected_field_names() {
        let r = RecentRefusalSla {
            snapshot_rows: Vec::new(),
            breach_rows: Vec::new(),
            is_breaching: false,
        };
        let s = serde_json::to_string(&r).expect("serialize");
        for key in [
            "\"snapshot_rows\"",
            "\"breach_rows\"",
            "\"is_breaching\"",
        ] {
            assert!(s.contains(key), "missing key {key} in {s}");
        }
    }
}
