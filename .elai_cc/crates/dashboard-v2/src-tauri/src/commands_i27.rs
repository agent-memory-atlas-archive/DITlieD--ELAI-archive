//! I-27 (SCHEMA_VERSION 265): ConvergenceCurveMeasured SSB read surface for
//! the Svelte `ConvergencePanel` drawer.
//!
//! One read-only command:
//!
//! - [`get_convergence_curve_rows`] -- the `N` most recent
//!   `convergence_curve_measured` flat-beacon rows emitted by
//!   `elai bench convergence`. Surfaces the full curve table (compute level /
//!   mean_rate / CI 95% / cost_per_verified_task), the knee label, and the
//!   non-monotonic-at set (GD-5 honesty).
//!
//! Default-ON: rows emit when `elai bench convergence --suite <fixture>` runs.
//! The bench command IS the entrypoint (no ELAI_*_ENABLE gate).
//!
//! Tier-1 fixture uses cost_per_task_cents proxy (E-49 live). Live
//! SearchNodeEvent compute-spend axis is WIRE-DARK[I-27] (PENDING-on-I-4).
//!
//! Mirror of commands_i38.rs::get_qdelta_latest pattern.
//!
//! cross-lang-pair: dashboard/tauri/api

use serde::{Deserialize, Serialize};
use tauri::State;

use elai_orchestrator::ssb::{SsbActionType, SsbPayload};

use crate::state::AppState;

// ---------------------------------------------------------------------------
// DTO
// ---------------------------------------------------------------------------

/// One curve point in a ConvergenceCurveMeasured row.
/// Field names match the TypeScript `ConvergenceCurvePoint` interface in
/// `ConvergencePanel.svelte`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConvergenceCurvePointDto {
    pub compute_level: String,
    pub mean_rate: f64,
    pub ci_low: f64,
    pub ci_high: f64,
    pub cost_per_verified_task: Option<f64>,
}

/// One row of the ConvergencePanel. Field names must match the TypeScript
/// `ConvergenceCurveMeasuredRow` interface in `ConvergencePanel.svelte`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConvergenceCurveMeasuredRow {
    pub fixture_suite: String,
    pub seed_count: u32,
    pub task_count: u32,
    pub curve_points: Vec<ConvergenceCurvePointDto>,
    pub knee_compute_level: String,
    pub non_monotonic_at: Vec<String>,
    pub seed: u64,
    pub fixture_blake3: String,
    pub runpod_pending: bool,
    pub ts_ms: i64,
}

// ---------------------------------------------------------------------------
// Command
// ---------------------------------------------------------------------------

/// Return the `limit` most recent `convergence_curve_measured` flat-beacon
/// rows from SSB. Returns an empty Vec when no viewer is attached or no
/// measurement has been recorded yet.
///
/// Cross-lang consumer of the bench_convergence.rs Rust emitter (same
/// pattern as commands_i38::get_qdelta_latest for QdeltaMeasured).
#[tauri::command]
pub async fn get_convergence_curve_rows(
    limit: u32,
    state: State<'_, AppState>,
) -> Result<Vec<ConvergenceCurveMeasuredRow>, String> {
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(Vec::new()),
    };

    let n = if limit == 0 { 10 } else { limit as usize };
    let raw = viewer
        .recent_rows_of_type(SsbActionType::ConvergenceCurveMeasured.as_sql_str(), n)
        .map_err(|e| e.to_string())?;

    let mut rows = Vec::with_capacity(raw.len());
    for entry in raw {
        let pl: SsbPayload = match serde_json::from_value(entry.payload_json.clone()) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if let SsbPayload::ConvergenceCurveMeasured(p) = pl {
            let curve_points = p
                .curve_points
                .iter()
                .map(|pt| ConvergenceCurvePointDto {
                    compute_level: pt.compute_level.clone(),
                    mean_rate: pt.mean_rate,
                    ci_low: pt.ci_low,
                    ci_high: pt.ci_high,
                    cost_per_verified_task: pt.cost_per_verified_task,
                })
                .collect();
            rows.push(ConvergenceCurveMeasuredRow {
                fixture_suite: p.fixture_suite.clone(),
                seed_count: p.seed_count,
                task_count: p.task_count,
                curve_points,
                knee_compute_level: p.knee_compute_level.clone(),
                non_monotonic_at: p.non_monotonic_at.clone(),
                seed: p.seed,
                fixture_blake3: p.fixture_blake3.clone(),
                runpod_pending: p.runpod_pending,
                ts_ms: p.ts_ms,
            });
        }
    }

    Ok(rows)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convergence_curve_point_dto_roundtrips_serde() {
        let pt = ConvergenceCurvePointDto {
            compute_level: "2x".to_string(),
            mean_rate: 0.75,
            ci_low: 0.62,
            ci_high: 0.88,
            cost_per_verified_task: Some(0.0042),
        };
        let json = serde_json::to_string(&pt).unwrap();
        let back: ConvergenceCurvePointDto = serde_json::from_str(&json).unwrap();
        assert_eq!(back.compute_level, "2x");
        assert!((back.mean_rate - 0.75).abs() < 1e-9);
        assert_eq!(back.cost_per_verified_task, Some(0.0042));
    }
}
