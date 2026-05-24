//! E-30: Public-agent-incident SSB read surface for the Svelte
//! `IncidentsDrawer` panel.
//!
//! One read-only command feeds the drawer:
//!
//! - [`get_recent_incidents`] -- last N rows across the 8 E-30 SSB action
//!   types (argv_destructive_denied / argv_approval_required /
//!   trust_dialog_hook_deferred / trust_dialog_env_deferred /
//!   mcp_pin_version_refused / auditor_lineage_collision /
//!   untrusted_memory_write_blocked / incident_record_created), newest
//!   first, flattened to a typed DTO the frontend can render without a
//!   second lookup.
//!
//! Additive -- runs alongside the existing `commands_d*` handlers without
//! touching them. Mirrors the `commands_d4::get_recent_tripwires` shape
//! (preferred sibling drawer pattern).

use std::cmp::Ordering;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 500;

/// The 8 E-30 SSB action_type sql_str values (mirrors
/// crates/orchestrator/src/ssb.rs:3414-3421). Kept as `&'static str` so the
/// frontend gets canonical kind strings rather than enum-name guesses.
const E30_ACTION_TYPES: &[&str] = &[
    "argv_destructive_denied",
    "argv_approval_required",
    "trust_dialog_hook_deferred",
    "trust_dialog_env_deferred",
    "mcp_pin_version_refused",
    "auditor_lineage_collision",
    "untrusted_memory_write_blocked",
    "incident_record_created",
];

/// One row of the panel. `kind` is the E-30 action_type sql_str.
///
/// Optional fields are populated based on which action_type emitted the
/// row -- argv events carry `argv_hash` + `reason` + `citation`; trust
/// dialog events carry `defer_reason`; pin events carry `server_name` +
/// `pinned_version` + `observed_version`; lineage events carry
/// `auditor_lineage` + `executor_lineage`; memory events carry
/// `text_source`; postmortems carry `incident_id` + `title`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncidentRow {
    pub seq: i64,
    pub ts: i64,
    pub kind: &'static str,
    // argv events
    pub argv_hash: Option<String>,
    pub reason: Option<String>,
    pub citation: Option<String>,
    // trust dialog events
    pub defer_reason: Option<String>,
    // mcp pin events
    pub server_name: Option<String>,
    pub pinned_version: Option<String>,
    pub observed_version: Option<String>,
    // lineage events
    pub auditor_lineage: Option<String>,
    pub executor_lineage: Option<String>,
    // memory events
    pub text_source: Option<String>,
    // postmortems
    pub incident_id: Option<String>,
    pub title: Option<String>,
    pub source_url: Option<String>,
}

/// Return the last `limit` E-30 incident rows from SSB, newest first.
/// Empty Vec when the SSB store is missing (fresh repo) or contains no
/// E-30 rows. Same shape as `commands_d4::get_recent_tripwires`.
#[tauri::command]
pub async fn get_recent_incidents(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<IncidentRow>, String> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(Vec::new()),
    };

    let mut combined: Vec<IncidentRow> = Vec::new();
    for at in E30_ACTION_TYPES {
        match viewer.recent_rows_of_type(at, limit) {
            Ok(rows) => {
                for detail in rows {
                    if let Some(row) = incident_detail_to_row(detail) {
                        combined.push(row);
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    action_type = at,
                    error = %e,
                    "get_recent_incidents: viewer error"
                );
            }
        }
    }
    combined.sort_by(|a, b| b.seq.cmp(&a.seq).then(Ordering::Equal));
    combined.truncate(limit);
    Ok(combined)
}

/// Resolve the canonical kind label for an action_type string. Returns
/// the matching `&'static str` from E30_ACTION_TYPES so the frontend gets
/// stable kind constants.
fn canonical_kind(action_type: &str) -> Option<&'static str> {
    E30_ACTION_TYPES.iter().copied().find(|k| *k == action_type)
}

fn obj_str(obj: &serde_json::Map<String, serde_json::Value>, key: &str) -> Option<String> {
    obj.get(key).and_then(|v| v.as_str()).map(String::from)
}

fn incident_detail_to_row(d: elai_orchestrator::SsbActionDetail) -> Option<IncidentRow> {
    let kind = canonical_kind(d.action_type.as_str())?;
    let obj = d.payload_json.as_object();

    let mut row = IncidentRow {
        seq: d.seq,
        ts: d.ts,
        kind,
        argv_hash: None,
        reason: None,
        citation: None,
        defer_reason: None,
        server_name: None,
        pinned_version: None,
        observed_version: None,
        auditor_lineage: None,
        executor_lineage: None,
        text_source: None,
        incident_id: None,
        title: None,
        source_url: None,
    };

    if let Some(obj) = obj {
        match kind {
            "argv_destructive_denied" | "argv_approval_required" => {
                row.argv_hash = obj_str(obj, "argv_hash");
                row.reason = obj_str(obj, "reason");
                row.citation = obj_str(obj, "citation");
            }
            "trust_dialog_hook_deferred" | "trust_dialog_env_deferred" => {
                row.defer_reason = obj_str(obj, "defer_reason")
                    .or_else(|| obj_str(obj, "reason"));
                row.citation = obj_str(obj, "citation");
            }
            "mcp_pin_version_refused" => {
                row.server_name = obj_str(obj, "server_name")
                    .or_else(|| obj_str(obj, "name"));
                row.pinned_version = obj_str(obj, "pinned_version");
                row.observed_version = obj_str(obj, "observed_version");
                row.reason = obj_str(obj, "reason");
            }
            "auditor_lineage_collision" => {
                row.auditor_lineage = obj_str(obj, "auditor_lineage");
                row.executor_lineage = obj_str(obj, "executor_lineage");
                row.reason = obj_str(obj, "reason");
            }
            "untrusted_memory_write_blocked" => {
                row.text_source = obj_str(obj, "text_source")
                    .or_else(|| obj_str(obj, "source"));
                row.reason = obj_str(obj, "reason");
            }
            "incident_record_created" => {
                row.incident_id = obj_str(obj, "incident_id");
                row.title = obj_str(obj, "title");
                row.source_url = obj_str(obj, "source_url");
            }
            _ => {}
        }
    }

    Some(row)
}
