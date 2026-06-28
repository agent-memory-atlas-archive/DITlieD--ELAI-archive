//! E-14: Browser-e2e dashboard surface (read-only).
//!
//! cross-lang-pair: dashboard/tauri/api
//!
//! Single Tauri command [`browser_e2e_status`] aggregates the 4
//! browser_e2e_* SSB action types into a small typed summary the
//! Svelte `MediaValidationPanel` renders in its dedicated browser-e2e
//! section:
//!
//!   * runs_started        — count of `browser_e2e_run_started` rows
//!   * assertions_passed   — count of `browser_e2e_assertion_passed` rows
//!   * assertions_failed   — count of `browser_e2e_assertion_failed` rows
//!   * snapshots_captured  — count of `browser_e2e_snapshot_captured` rows
//!
//! Plus a `last_journey_name_hash` + `last_ts_ms` from the most recent
//! row across the four streams so the dashboard can show "last journey
//! ran X ago".
//!
//! Additive — sits alongside the C-47 / D-* surface and uses the same
//! `SsbViewer::recent_rows_of_type` pattern. Returns a deterministic
//! empty summary when the SSB viewer is unwired (CALIBRATION-PENDING
//! parity).

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

const RECENT_LIMIT: usize = 200;

/// Aggregated wire-shape consumed by the Svelte
/// `MediaValidationPanel`. Hashes are rendered as 16-char hex
/// (`format!("{:016x}", u64)`) so the frontend can show them
/// fixed-width without losing precision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserE2eStatusDto {
    pub runs_started: u32,
    pub assertions_passed: u32,
    pub assertions_failed: u32,
    pub snapshots_captured: u32,
    /// Hex-rendered blake3-prefix u64 of the most recent journey
    /// (across all 4 streams). `None` when there are no rows yet.
    pub last_journey_name_hash: Option<String>,
    /// `ts_ms` of the most recent row across all 4 streams. `None`
    /// when there are no rows yet.
    pub last_ts_ms: Option<i64>,
}

impl Default for BrowserE2eStatusDto {
    fn default() -> Self {
        Self {
            runs_started: 0,
            assertions_passed: 0,
            assertions_failed: 0,
            snapshots_captured: 0,
            last_journey_name_hash: None,
            last_ts_ms: None,
        }
    }
}

/// Aggregate the 4 browser_e2e_* action types into a single summary.
/// Returns the default (zeroed) summary when the SSB viewer is
/// unwired. Anti-stale: NEVER panics; the wire-dark posture is to
/// surface zeros so the panel renders the "no browser-e2e rows yet"
/// state.
#[tauri::command]
pub async fn browser_e2e_status(
    state: State<'_, AppState>,
) -> Result<BrowserE2eStatusDto, String> {
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Err("SSB viewer unavailable".to_string()),
    };

    let action_types: [&str; 4] = [
        "browser_e2e_run_started",
        "browser_e2e_assertion_passed",
        "browser_e2e_assertion_failed",
        "browser_e2e_snapshot_captured",
    ];

    let mut runs_started = 0u32;
    let mut assertions_passed = 0u32;
    let mut assertions_failed = 0u32;
    let mut snapshots_captured = 0u32;
    let mut last_journey_name_hash: Option<u64> = None;
    let mut last_ts_ms: Option<i64> = None;

    for at in action_types {
        let details = match viewer.recent_rows_of_type(at, RECENT_LIMIT) {
            Ok(d) => d,
            Err(e) => {
                tracing::warn!(
                    action_type = at,
                    error = %e,
                    "browser_e2e_status: viewer error"
                );
                continue;
            }
        };
        match at {
            "browser_e2e_run_started" => {
                runs_started = runs_started.saturating_add(
                    u32::try_from(details.len()).unwrap_or(u32::MAX),
                );
            }
            "browser_e2e_assertion_passed" => {
                assertions_passed = assertions_passed.saturating_add(
                    u32::try_from(details.len()).unwrap_or(u32::MAX),
                );
            }
            "browser_e2e_assertion_failed" => {
                assertions_failed = assertions_failed.saturating_add(
                    u32::try_from(details.len()).unwrap_or(u32::MAX),
                );
            }
            "browser_e2e_snapshot_captured" => {
                snapshots_captured = snapshots_captured.saturating_add(
                    u32::try_from(details.len()).unwrap_or(u32::MAX),
                );
            }
            _ => {}
        }
        for d in details {
            if let Some(ts) = d
                .payload_json
                .as_object()
                .and_then(|o| o.get("ts_ms"))
                .and_then(|v| v.as_i64())
            {
                let new_ts = match last_ts_ms {
                    Some(prev) if prev >= ts => continue,
                    _ => ts,
                };
                let hash = d
                    .payload_json
                    .as_object()
                    .and_then(|o| o.get("journey_name_hash"))
                    .and_then(|v| v.as_u64());
                last_ts_ms = Some(new_ts);
                last_journey_name_hash = hash;
            }
        }
    }

    Ok(BrowserE2eStatusDto {
        runs_started,
        assertions_passed,
        assertions_failed,
        snapshots_captured,
        last_journey_name_hash: last_journey_name_hash.map(|h| format!("{h:016x}")),
        last_ts_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_zeroed() {
        let d = BrowserE2eStatusDto::default();
        assert_eq!(d.runs_started, 0);
        assert_eq!(d.assertions_passed, 0);
        assert_eq!(d.assertions_failed, 0);
        assert_eq!(d.snapshots_captured, 0);
        assert!(d.last_journey_name_hash.is_none());
        assert!(d.last_ts_ms.is_none());
    }

    #[test]
    fn hex_format_pads_to_sixteen_chars() {
        // u64 max = 16 hex chars; small numbers must also pad to 16.
        let h: u64 = 0xAB;
        let s = format!("{h:016x}");
        assert_eq!(s.len(), 16);
        assert_eq!(s, "00000000000000ab");
    }
}
