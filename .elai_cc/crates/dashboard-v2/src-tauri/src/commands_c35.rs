//! C-35: Tool gate dashboard surface (read-only).
//!
//! cross-lang-pair: dashboard/tauri/api
//!
//! Two read-only commands feed the Svelte `ToolGateInspector`:
//!
//! - [`get_tool_gate_snapshot`] -- current FSM state's gate (eager / gated /
//!   denied tool names). Returns an empty snapshot when the gate is not
//!   wired into AppState yet (TOOL-GATE-PENDING).
//! - [`list_tool_gate_rows`] -- last N tool_* SSB rows (tool_gated /
//!   tool_eager / tool_lazy_resolved / tool_denied), flattened to a typed DTO
//!   the frontend can render without re-loading the underlying payload.
//!
//! Additive -- runs alongside the existing `commands::*` + `commands_d*::*`
//! handlers without touching them. Sibling of `commands_empire`.
//!
//! Status mapping:
//!   - manifest absent or feature off => empty snapshot, empty rows (the
//!     Svelte panel renders "Gate snapshot unavailable" + empty histogram)
//!   - manifest loaded => snapshot reflects the current FSM state's gate;
//!     rows reflect any emit-sites the orchestrator has fired

use std::cmp::Ordering;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 200;
const MAX_LIMIT: usize = 2000;

/// Gate snapshot wire shape consumed by `ToolGateInspector.svelte`.
/// Matches the `GateSnapshot` interface on the svelte side.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolGateSnapshotDto {
    /// FSM state the snapshot was computed against (lowercase string).
    pub fsm_state: String,
    /// Tool names rendered as full schemas in the prompt.
    pub eager: Vec<String>,
    /// Tool names rendered as 1-line summaries in the prompt.
    pub gated: Vec<String>,
    /// Tool names omitted from the prompt entirely.
    pub denied: Vec<String>,
    /// True when the orchestrator was built with the `mcp` feature.
    pub feature_enabled: bool,
    /// Free-form human note. Surfaces the TOOL-GATE-PENDING reason.
    pub note: String,
}

/// One row of the panel -- maps a `ToolGateEventPayload` SSB row to the wire
/// shape the panel consumes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolGateRowDto {
    pub tool_name: String,
    pub fsm_state: String,
    pub prompt_tokens_saved: i64,
    /// One of `"tool_gated" | "tool_eager" | "tool_lazy_resolved" | "tool_denied"`.
    pub action_type: String,
    pub ts_ms: i64,
}

#[cfg(not(feature = "mcp"))]
const NOTE_UNAVAILABLE: &str =
    "Tool gate unavailable: dashboard built without the `mcp` feature. \
     Rebuild with --features mcp to activate dynamic gating.";

#[cfg(feature = "mcp")]
const NOTE_PENDING: &str =
    "Tool gate present but not wired into AppState yet. TOOL-GATE-PENDING: \
     orchestrator bootstrap will attach the live DefaultMcpBridge handle.";

/// Return the current tool-gate snapshot for the given FSM state. When no
/// `fsm_state` is supplied, defaults to "planning" (the most common request
/// during the EXECUTING flow's planning sub-phase).
#[tauri::command]
pub async fn get_tool_gate_snapshot(
    _state: State<'_, AppState>,
    fsm_state: Option<String>,
) -> Result<ToolGateSnapshotDto, String> {
    let state_label = fsm_state
        .as_deref()
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "planning".to_string());
    let (feature_enabled, eager, gated, denied, note) =
        snapshot_inner(&state_label);
    Ok(ToolGateSnapshotDto {
        fsm_state: state_label,
        eager,
        gated,
        denied,
        feature_enabled,
        note: note.to_string(),
    })
}

/// Return the last `limit` tool_* SSB rows, newest first. Reads from the
/// shared `SsbViewer` -- the orchestrator-side emit path
/// (`OrchestratorToolGateSsbEmitter`) writes the matching rows.
///
/// Empty Vec when the SSB store is missing (fresh repo) or contains no
/// tool_* rows yet.
#[tauri::command]
pub async fn list_tool_gate_rows(
    state: State<'_, AppState>,
    window_ms: Option<i64>,
    limit: Option<usize>,
) -> Result<Vec<ToolGateRowDto>, String> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(Vec::new()),
    };

    // Pull from all 4 action types. Each action_type lookup is bounded by
    // `limit` so the worst-case total is 4 * limit rows before the window
    // filter + sort.
    let action_types = [
        "tool_gated",
        "tool_eager",
        "tool_lazy_resolved",
        "tool_denied",
    ];
    let mut rows: Vec<ToolGateRowDto> = Vec::with_capacity(action_types.len() * limit);
    for at in action_types {
        match viewer.recent_rows_of_type(at, limit) {
            Ok(details) => {
                for d in details {
                    if let Some(row) = detail_to_row(d) {
                        rows.push(row);
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    action_type = at,
                    error = %e,
                    "list_tool_gate_rows: viewer error"
                );
            }
        }
    }

    // Newest first by ts_ms (recent_rows_of_type returns newest first per
    // action_type, but the cross-type merge needs to be re-sorted).
    rows.sort_by(|a, b| b.ts_ms.cmp(&a.ts_ms).then(Ordering::Equal));

    // Optional window filter (default: keep all returned rows).
    if let Some(window) = window_ms {
        if window > 0 {
            let cutoff_ms = now_ms().saturating_sub(window);
            rows.retain(|r| r.ts_ms >= cutoff_ms);
        }
    }

    rows.truncate(limit);
    Ok(rows)
}

fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

fn detail_to_row(d: elai_orchestrator::SsbActionDetail) -> Option<ToolGateRowDto> {
    let obj = d.payload_json.as_object()?;
    Some(ToolGateRowDto {
        tool_name: obj.get("tool_name")?.as_str()?.to_string(),
        fsm_state: obj.get("fsm_state")?.as_str()?.to_string(),
        prompt_tokens_saved: obj.get("prompt_tokens_saved")?.as_i64().unwrap_or(0),
        action_type: d.action_type.clone(),
        ts_ms: d.ts,
    })
}

// ---------------------------------------------------------------------------
// Feature-conditional implementation seam
// ---------------------------------------------------------------------------

#[cfg(feature = "mcp")]
fn snapshot_inner(
    fsm_state: &str,
) -> (bool, Vec<String>, Vec<String>, Vec<String>, &'static str) {
    // Read the gate manifest from the canonical project-root path. The bridge
    // wiring step (TOOL-GATE-PENDING) will replace this with a direct read of
    // the live `DefaultMcpBridge::tool_gate()` handle once AppState carries it.
    let manifest_path = manifest_path_from_env();
    let gate = match elai_plugins::load_gate(&manifest_path) {
        Ok(g) => g,
        Err(e) => {
            tracing::warn!(
                error = %e,
                path = %manifest_path.display(),
                "C-35 get_tool_gate_snapshot: load_gate failed"
            );
            return (true, Vec::new(), Vec::new(), Vec::new(), NOTE_PENDING);
        }
    };

    // Filter against an empty catalog -- the snapshot exposes the gate
    // *configuration* (which tool NAMES land in which bucket), not which
    // tools are currently registered. For the configuration view, we read
    // the underlying state config directly via the per-state grep below.
    let (eager, gated, denied) = read_state_lists(&gate, fsm_state);
    (true, eager, gated, denied, NOTE_PENDING)
}

#[cfg(feature = "mcp")]
fn manifest_path_from_env() -> std::path::PathBuf {
    // Match the same default path elai_cc.toml [tool_gating] specifies.
    std::env::var("ELAI_TOOL_GATE_MANIFEST")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("plugins/tool-gate.toml"))
}

#[cfg(feature = "mcp")]
fn read_state_lists(
    gate: &elai_plugins::ToolGate,
    fsm_state: &str,
) -> (Vec<String>, Vec<String>, Vec<String>) {
    // Re-derive the per-state lists by intersecting the loaded config with a
    // synthetic full catalog containing one stub per declared name. We can
    // re-read the toml file directly to avoid plumbing a new accessor into
    // ToolGate -- the snapshot is read-only and infrequent (panel poll every
    // 5s), so the duplicate read cost is acceptable.
    let manifest_path = manifest_path_from_env();
    let content = match std::fs::read_to_string(&manifest_path) {
        Ok(c) => c,
        Err(_) => return (Vec::new(), Vec::new(), Vec::new()),
    };
    #[derive(serde::Deserialize)]
    struct TomlManifest {
        gate: std::collections::HashMap<String, TomlStateEntry>,
    }
    #[derive(serde::Deserialize)]
    struct TomlStateEntry {
        eager: Option<Vec<String>>,
        gated: Option<Vec<String>>,
        denied: Option<Vec<String>>,
    }
    let raw: TomlManifest = match toml::from_str(&content) {
        Ok(m) => m,
        Err(_) => return (Vec::new(), Vec::new(), Vec::new()),
    };
    // Avoid an unused-var warning when the manifest path resolves but the
    // gate handle is consumed elsewhere; the safe_default-vs-loaded distinction
    // is irrelevant to the snapshot view.
    let _ = gate.reason();
    if let Some(entry) = raw.gate.get(&fsm_state.to_lowercase()) {
        (
            entry.eager.clone().unwrap_or_default(),
            entry.gated.clone().unwrap_or_default(),
            entry.denied.clone().unwrap_or_default(),
        )
    } else {
        (Vec::new(), Vec::new(), Vec::new())
    }
}

#[cfg(not(feature = "mcp"))]
fn snapshot_inner(
    _fsm_state: &str,
) -> (bool, Vec<String>, Vec<String>, Vec<String>, &'static str) {
    (
        false,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        NOTE_UNAVAILABLE,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dto_shapes_round_trip_serde() {
        let snap = ToolGateSnapshotDto {
            fsm_state: "planning".into(),
            eager: vec!["knowledge.search".into()],
            gated: vec!["fs.read".into()],
            denied: vec!["git.commit".into()],
            feature_enabled: true,
            note: "ok".into(),
        };
        let s = serde_json::to_string(&snap).expect("serialise");
        let back: ToolGateSnapshotDto = serde_json::from_str(&s).expect("parse");
        assert_eq!(back, snap);
    }

    #[test]
    fn row_dto_round_trip_serde() {
        let row = ToolGateRowDto {
            tool_name: "fs.read".into(),
            fsm_state: "planning".into(),
            prompt_tokens_saved: 42,
            action_type: "tool_gated".into(),
            ts_ms: 1_700_000_000_000,
        };
        let s = serde_json::to_string(&row).expect("serialise");
        let back: ToolGateRowDto = serde_json::from_str(&s).expect("parse");
        assert_eq!(back, row);
    }
}
