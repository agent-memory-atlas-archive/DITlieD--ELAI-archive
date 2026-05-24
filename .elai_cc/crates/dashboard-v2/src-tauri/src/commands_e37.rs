//! E-37: Prompt-injection defense dashboard surface (read-only).
//!
//! cross-lang-pair: dashboard/tauri/api
//!
//! Two read-only Tauri commands feed the Svelte `PromptArmorPanel`:
//!
//! - [`safety_status`] — aggregate count of the 9 E-37 SSB action types
//!   (injection_attempt_detected / injection_attempt_blocked /
//!   spotlight_wrapped / spotlight_breached / tool_description_pin_mismatch
//!   / capability_gate_rejected / provenance_tagged / prompt_armor_verdict /
//!   injection_fixture_replayed) + pin store path. Returns a snapshot with
//!   zeroed counts when the orchestrator has not wired a live SSB emitter
//!   yet (SAFETY-PENDING).
//! - [`list_safety_rows`] — last N E-37 SSB rows, newest first. Flattens
//!   the 9 action types into a single DTO list the frontend can render
//!   without re-loading the underlying payload.
//!
//! Additive — runs alongside `commands_c35.rs` (tool gate) /
//! `commands_c42.rs` (authz) / `commands_c47.rs` (calibration) /
//! `commands_e18.rs` (reliability) without touching them.
//!
//! See `workflow/reasoning/E-37-decisions.md`.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 200;
const MAX_LIMIT: usize = 2000;

/// Wire shape for the `safety_status` command. Mirrors the StatusRow
/// shape the CLI `elai safety status` handler emits so operators can
/// cross-verify CLI + dashboard counts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafetyStatusDto {
    pub injection_detected_count: u64,
    pub injection_blocked_count: u64,
    pub spotlight_wrapped_count: u64,
    pub spotlight_breached_count: u64,
    pub pin_mismatch_count: u64,
    pub capability_rejected_count: u64,
    pub provenance_tagged_count: u64,
    pub prompt_armor_verdict_count: u64,
    pub fixture_replayed_count: u64,
    pub pin_dir: String,
}

/// One SSB row of the safety panel — maps any E-37 action type to a
/// compact wire shape (action_type label + timestamp). Per-row payload
/// detail is intentionally omitted; the panel renders aggregate counts
/// only. Operators inspect individual hashes via the CLI surface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafetyRowDto {
    pub action_type: String,
    pub ts_ms: i64,
}

/// All 9 E-37 SSB action type wire labels. Keep alphabetically sorted
/// to match action_type_lock.golden discipline.
const E37_ACTION_TYPES: [&str; 9] = [
    "capability_gate_rejected",
    "injection_attempt_blocked",
    "injection_attempt_detected",
    "injection_fixture_replayed",
    "prompt_armor_verdict",
    "provenance_tagged",
    "spotlight_breached",
    "spotlight_wrapped",
    "tool_description_pin_mismatch",
];

/// Return the safety panel snapshot. When the orchestrator has not
/// wired a live SSB emitter yet, returns the zeroed snapshot (the
/// Svelte panel renders the "no events recorded yet" empty state).
#[tauri::command]
pub async fn safety_status(state: State<'_, AppState>) -> Result<SafetyStatusDto, String> {
    let pin_dir = ".elai_cc/.elai/mcp/tool_descriptions".to_string();
    let counts = if let Some(viewer) = &state.viewer {
        let count_for = |at: &str| -> u64 {
            viewer
                .recent_rows_of_type(at, MAX_LIMIT)
                .map(|v| v.len() as u64)
                .unwrap_or(0)
        };
        SafetyStatusDto {
            injection_detected_count: count_for("injection_attempt_detected"),
            injection_blocked_count: count_for("injection_attempt_blocked"),
            spotlight_wrapped_count: count_for("spotlight_wrapped"),
            spotlight_breached_count: count_for("spotlight_breached"),
            pin_mismatch_count: count_for("tool_description_pin_mismatch"),
            capability_rejected_count: count_for("capability_gate_rejected"),
            provenance_tagged_count: count_for("provenance_tagged"),
            prompt_armor_verdict_count: count_for("prompt_armor_verdict"),
            fixture_replayed_count: count_for("injection_fixture_replayed"),
            pin_dir,
        }
    } else {
        SafetyStatusDto {
            injection_detected_count: 0,
            injection_blocked_count: 0,
            spotlight_wrapped_count: 0,
            spotlight_breached_count: 0,
            pin_mismatch_count: 0,
            capability_rejected_count: 0,
            provenance_tagged_count: 0,
            prompt_armor_verdict_count: 0,
            fixture_replayed_count: 0,
            pin_dir,
        }
    };
    Ok(counts)
}

/// Return the last `limit` E-37 SSB rows across all 9 action types,
/// flattened to a typed DTO. Empty Vec when the SSB store is missing
/// or contains no E-37 rows yet.
#[tauri::command]
pub async fn list_safety_rows(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<SafetyRowDto>, String> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(Vec::new()),
    };

    let mut rows: Vec<SafetyRowDto> = Vec::with_capacity(E37_ACTION_TYPES.len() * limit);
    for at in E37_ACTION_TYPES {
        match viewer.recent_rows_of_type(at, limit) {
            Ok(details) => {
                for d in details {
                    // SsbActionDetail.ts is the wall-clock UNIX ms when the
                    // SSB row was persisted; the panel renders this as
                    // `ts_ms` in the wire DTO for parity with the E-37
                    // payload fields (all carry `ts_ms` directly).
                    rows.push(SafetyRowDto {
                        action_type: at.to_string(),
                        ts_ms: d.ts,
                    });
                }
            }
            Err(e) => {
                tracing::warn!(action_type = at, error = %e, "list_safety_rows: viewer error");
            }
        }
    }
    // Sort newest-first to match the panel's expected order.
    rows.sort_by(|a, b| b.ts_ms.cmp(&a.ts_ms));
    rows.truncate(limit);
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn e37_action_types_are_alphabetically_sorted() {
        let mut sorted: Vec<&str> = E37_ACTION_TYPES.to_vec();
        sorted.sort();
        assert_eq!(
            sorted.as_slice(),
            &E37_ACTION_TYPES[..],
            "E37_ACTION_TYPES must be alphabetically sorted (action_type_lock discipline)"
        );
        assert_eq!(E37_ACTION_TYPES.len(), 9, "E-37 ships exactly 9 SSB types");
    }

    #[test]
    fn safety_status_dto_serialises() {
        let dto = SafetyStatusDto {
            injection_detected_count: 1,
            injection_blocked_count: 2,
            spotlight_wrapped_count: 3,
            spotlight_breached_count: 0,
            pin_mismatch_count: 0,
            capability_rejected_count: 0,
            provenance_tagged_count: 0,
            prompt_armor_verdict_count: 0,
            fixture_replayed_count: 0,
            pin_dir: ".elai_cc/.elai/mcp/tool_descriptions".into(),
        };
        let s = serde_json::to_string(&dto).expect("serialize");
        let back: SafetyStatusDto = serde_json::from_str(&s).expect("deserialize");
        assert_eq!(dto, back);
    }
}
