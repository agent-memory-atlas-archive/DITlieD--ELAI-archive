//! D-20 — ARC-AGI-2 OOD probe read-only Tauri command surface.
//!
//! One read-only command feeds the Svelte `ARCBenchPanel`:
//!
//! - [`get_arc_bench_recent`] — last N `arc_benchmark_completed` SSB
//!   rows surfaced as `ARCBenchRunDto` rows so the panel can render
//!   the verdict banner + per-mode + per-novelty histograms across the
//!   recent runs without re-walking the SSB.
//!
//! Additive — sibling of `commands_d18` (typed ObservableEffect),
//! `commands_d19` (PRMBench gating), `commands_d21` (mode-routing
//! coverage). The orchestrator producer
//! (`elai_orchestrator::benchmarks::arc_agi2_runner::ARCAGI2Runner::run_all`)
//! is the sole writer of the underlying SSB rows; this module only
//! reads.
//!
//! See `docs/thefinalglance/D-plan/D-20_arc_agi2_ood_probe.md`.

use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::warn;

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 25;
const MAX_LIMIT: usize = 200;

/// Typed mirror of one `arc_benchmark_completed` SSB row — wire form
/// returned to the Svelte panel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ARCBenchRunDto {
    pub total: u32,
    pub honestly_evaluated_count: u32,
    pub gating_verdict: String,
    pub fixture_blake3: String,
    pub dominant_mode: String,
    pub dominant_novelty_class: String,
    pub ts_ms: i64,
}

/// Aggregated response for the panel — the recent N rows + the most
/// recent row's fields exposed at the top level so the banner can
/// render without indexing into `rows[0]` in JS.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecentARCBenchRuns {
    pub rows: Vec<ARCBenchRunDto>,
    /// Total rows surfaced (always <= cap).
    pub total: u32,
    /// Last verdict label (`honestly_evaluated` | `game_with_memorization`
    /// | `refused`) — empty string if no rows.
    pub last_verdict: String,
    /// Last `dominant_mode` — empty string if no rows.
    pub last_dominant_mode: String,
    /// Last `dominant_novelty_class` — empty string if no rows.
    pub last_dominant_novelty_class: String,
}

/// Return the most recent `arc_benchmark_completed` rows (capped at
/// `MAX_LIMIT`). Empty response when no SSB viewer is attached
/// (Tauri-dev fresh boot, no DB).
#[tauri::command]
pub async fn get_arc_bench_recent(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<RecentARCBenchRuns, String> {
    let cap = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => {
            return Ok(RecentARCBenchRuns {
                rows: Vec::new(),
                total: 0,
                last_verdict: String::new(),
                last_dominant_mode: String::new(),
                last_dominant_novelty_class: String::new(),
            });
        }
    };

    let details = match viewer.recent_rows_of_type("arc_benchmark_completed", cap) {
        Ok(d) => d,
        Err(e) => {
            warn!(
                action_type = "arc_benchmark_completed",
                error = %e,
                "get_arc_bench_recent: viewer error"
            );
            Vec::new()
        }
    };

    let mut rows: Vec<ARCBenchRunDto> = details.into_iter().filter_map(detail_to_row).collect();
    rows.sort_by(|a, b| b.ts_ms.cmp(&a.ts_ms));
    rows.truncate(cap);

    let total = u32::try_from(rows.len()).unwrap_or(u32::MAX);
    let (last_verdict, last_dominant_mode, last_dominant_novelty_class) = match rows.first() {
        Some(r) => (
            r.gating_verdict.clone(),
            r.dominant_mode.clone(),
            r.dominant_novelty_class.clone(),
        ),
        None => (String::new(), String::new(), String::new()),
    };

    Ok(RecentARCBenchRuns {
        rows,
        total,
        last_verdict,
        last_dominant_mode,
        last_dominant_novelty_class,
    })
}

fn detail_to_row(
    d: elai_orchestrator::ssb_viewer::SsbActionDetail,
) -> Option<ARCBenchRunDto> {
    if d.action_type.as_str() != "arc_benchmark_completed" {
        return None;
    }
    let outer = d.payload_json.as_object()?;
    // The payload may be nested under the enum variant name
    // (`ArcBenchmarkCompleted`) depending on serde tagging — match both.
    let obj = if let Some(inner) = outer
        .get("ArcBenchmarkCompleted")
        .and_then(|v| v.as_object())
    {
        inner.clone()
    } else {
        outer.clone()
    };
    Some(ARCBenchRunDto {
        total: obj
            .get("total")
            .and_then(|v| v.as_u64())
            .and_then(|v| u32::try_from(v).ok())
            .unwrap_or(0),
        honestly_evaluated_count: obj
            .get("honestly_evaluated_count")
            .and_then(|v| v.as_u64())
            .and_then(|v| u32::try_from(v).ok())
            .unwrap_or(0),
        gating_verdict: obj
            .get("gating_verdict")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        fixture_blake3: obj
            .get("fixture_blake3")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        dominant_mode: obj
            .get("dominant_mode")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        dominant_novelty_class: obj
            .get("dominant_novelty_class")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        ts_ms: obj.get("ts_ms").and_then(|v| v.as_i64()).unwrap_or(0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dto_serializes_with_expected_field_names() {
        let dto = ARCBenchRunDto {
            total: 24,
            honestly_evaluated_count: 11,
            gating_verdict: "honestly_evaluated".into(),
            fixture_blake3: "a".repeat(64),
            dominant_mode: "operational_deferred".into(),
            dominant_novelty_class: "discovery_candidate".into(),
            ts_ms: 1_700_000_000_000,
        };
        let s = serde_json::to_string(&dto).expect("serialize");
        for key in [
            "\"total\"",
            "\"honestly_evaluated_count\"",
            "\"gating_verdict\"",
            "\"fixture_blake3\"",
            "\"dominant_mode\"",
            "\"dominant_novelty_class\"",
            "\"ts_ms\"",
        ] {
            assert!(s.contains(key), "missing key {key} in {s}");
        }
    }

    #[test]
    fn recent_response_serializes_with_expected_field_names() {
        let r = RecentARCBenchRuns {
            rows: Vec::new(),
            total: 0,
            last_verdict: String::new(),
            last_dominant_mode: String::new(),
            last_dominant_novelty_class: String::new(),
        };
        let s = serde_json::to_string(&r).expect("serialize");
        for key in [
            "\"rows\"",
            "\"total\"",
            "\"last_verdict\"",
            "\"last_dominant_mode\"",
            "\"last_dominant_novelty_class\"",
        ] {
            assert!(s.contains(key), "missing key {key} in {s}");
        }
    }
}
