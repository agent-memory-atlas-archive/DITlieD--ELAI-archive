//! D-9 — L3 frozen-surface auditor Tauri command surface.
//!
//! Two read-only commands feed the Svelte `LockedKeysPanel`:
//!
//! - [`get_locked_keys_status`] — parsed registry snapshot (entries +
//!   content blake3 + loaded_ts_ms). Sourced directly from
//!   `<cwd>/scripts/locked_keys_registry.toml` via
//!   `elai_orchestrator::locked_keys_status_snapshot`.
//! - [`get_locked_keys_recent_audits`] — last N `locked_keys_audit_run`
//!   SSB rows, sorted by ts_ms desc, joined with their
//!   `locked_keys_violation_detected` children grouped by `audit_id`.
//!
//! Additive — sibling of `commands_d7` (canary). The orchestrator
//! producer (`LockedKeysAuditor::audit_diff_ranged`) is the sole writer
//! of the SSB rows; this module only reads.
//!
//! See docs/thefinalglance/D-plan/D-9_l3_frozen_surfaces.md.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 500;

/// Flat status snapshot for the registry. Mirrors
/// `elai_orchestrator::LockedKeysStatusSnapshot` but lives in this
/// crate so the Tauri-side schema doesn't leak the orchestrator's
/// internal types through the IPC boundary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LockedKeysStatusDto {
    pub registry_path: String,
    pub n_entries: u32,
    pub registry_blake3: String,
    pub loaded_ts_ms: i64,
    pub entries: Vec<LockedKeysEntryDto>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LockedKeysEntryDto {
    pub path: String,
    pub line_range: Vec<u32>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LockedKeysAuditRunDto {
    pub audit_id: String,
    pub ts_ms: i64,
    pub n_audited: u32,
    pub n_violations: u32,
    pub registry_blake3: String,
    pub registry_path: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LockedKeysViolationDto {
    pub audit_id: String,
    pub ts_ms: i64,
    pub attempted_edit: String,
    pub attempted_lines: Vec<u32>,
    pub locked_key_path: String,
    pub reason: String,
}

/// Aggregated recent-audit response: latest run rows + per-audit-id
/// violation breakdown.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LockedKeysRecentAudits {
    pub runs: Vec<LockedKeysAuditRunDto>,
    pub violations_by_audit: HashMap<String, Vec<LockedKeysViolationDto>>,
}

/// Read the current registry from disk and return a typed snapshot.
/// `error` is populated when the registry is missing or malformed
/// (fail-soft on the dashboard surface; the auditor itself fails
/// closed when invoked — these are two different boundaries).
#[tauri::command]
pub async fn get_locked_keys_status(
    _state: State<'_, AppState>,
    registry_path: Option<String>,
) -> Result<LockedKeysStatusDto, String> {
    let path = registry_path
        .map(PathBuf::from)
        .unwrap_or_else(|| default_registry_path());
    match elai_orchestrator::locked_keys_status_snapshot(&path) {
        Ok(snap) => Ok(LockedKeysStatusDto {
            registry_path: snap.registry_path,
            n_entries: snap.n_entries,
            registry_blake3: snap.registry_blake3,
            loaded_ts_ms: snap.loaded_ts_ms,
            entries: snap
                .entries
                .into_iter()
                .map(|e| LockedKeysEntryDto {
                    path: e.path,
                    line_range: e.line_range,
                    reason: e.reason,
                })
                .collect(),
            error: None,
        }),
        Err(e) => Ok(LockedKeysStatusDto {
            registry_path: path.to_string_lossy().to_string(),
            n_entries: 0,
            registry_blake3: String::new(),
            loaded_ts_ms: 0,
            entries: Vec::new(),
            error: Some(e.to_string()),
        }),
    }
}

/// Return the most recent `locked_keys_audit_run` rows + their
/// matching `locked_keys_violation_detected` rows grouped by
/// `audit_id`. Empty response when no SSB viewer is attached.
#[tauri::command]
pub async fn get_locked_keys_recent_audits(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<LockedKeysRecentAudits, String> {
    let cap = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => {
            return Ok(LockedKeysRecentAudits {
                runs: Vec::new(),
                violations_by_audit: HashMap::new(),
            });
        }
    };

    let mut runs: Vec<LockedKeysAuditRunDto> = Vec::new();
    match viewer.recent_rows_of_type("locked_keys_audit_run", cap) {
        Ok(details) => {
            for d in details {
                if let Some(row) = detail_to_run_dto(d) {
                    runs.push(row);
                }
            }
        }
        Err(e) => {
            tracing::warn!(
                action_type = "locked_keys_audit_run",
                error = %e,
                "get_locked_keys_recent_audits: viewer error"
            );
        }
    }
    runs.sort_by(|a, b| b.ts_ms.cmp(&a.ts_ms));
    runs.truncate(cap);

    let mut violations_by_audit: HashMap<String, Vec<LockedKeysViolationDto>> = HashMap::new();
    // Over-fetch by 4x — most runs emit 0 violations; this keeps the
    // grouped view consistent with the run cap.
    let fetch = cap.saturating_mul(4).min(MAX_LIMIT * 4);
    match viewer.recent_rows_of_type("locked_keys_violation_detected", fetch) {
        Ok(details) => {
            for d in details {
                if let Some(v) = detail_to_violation_dto(d) {
                    violations_by_audit
                        .entry(v.audit_id.clone())
                        .or_default()
                        .push(v);
                }
            }
        }
        Err(e) => {
            tracing::warn!(
                action_type = "locked_keys_violation_detected",
                error = %e,
                "get_locked_keys_recent_audits: viewer error"
            );
        }
    }

    Ok(LockedKeysRecentAudits {
        runs,
        violations_by_audit,
    })
}

fn default_registry_path() -> PathBuf {
    // The Tauri app's cwd is `.elai_cc/crates/dashboard-v2/src-tauri`
    // during `tauri:dev`, so walk up to `.elai_cc/` to find scripts/.
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    for ancestor in cwd.ancestors() {
        let candidate = ancestor.join("scripts").join("locked_keys_registry.toml");
        if candidate.exists() {
            return candidate;
        }
    }
    cwd.join("scripts").join("locked_keys_registry.toml")
}

fn unwrap_variant<'a>(
    payload: &'a serde_json::Value,
    variant: &str,
) -> Option<&'a serde_json::Map<String, serde_json::Value>> {
    let outer = payload.as_object()?;
    if let Some(inner) = outer.get(variant).and_then(|v| v.as_object()) {
        return Some(inner);
    }
    Some(outer)
}

fn detail_to_run_dto(d: elai_orchestrator::SsbActionDetail) -> Option<LockedKeysAuditRunDto> {
    if d.action_type.as_str() != "locked_keys_audit_run" {
        return None;
    }
    let obj = unwrap_variant(&d.payload_json, "LockedKeysAuditRun")?;
    Some(LockedKeysAuditRunDto {
        audit_id: obj
            .get("audit_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        ts_ms: obj.get("ts_ms").and_then(|v| v.as_i64()).unwrap_or(0),
        n_audited: obj
            .get("n_audited")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32,
        n_violations: obj
            .get("n_violations")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32,
        registry_blake3: obj
            .get("registry_blake3")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        registry_path: obj
            .get("registry_path")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
    })
}

fn detail_to_violation_dto(
    d: elai_orchestrator::SsbActionDetail,
) -> Option<LockedKeysViolationDto> {
    if d.action_type.as_str() != "locked_keys_violation_detected" {
        return None;
    }
    let obj = unwrap_variant(&d.payload_json, "LockedKeysViolationDetected")?;
    let attempted_lines: Vec<u32> = obj
        .get("attempted_lines")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|n| n.as_u64().map(|u| u as u32))
                .collect()
        })
        .unwrap_or_default();
    Some(LockedKeysViolationDto {
        audit_id: obj
            .get("audit_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        ts_ms: obj.get("ts_ms").and_then(|v| v.as_i64()).unwrap_or(0),
        attempted_edit: obj
            .get("attempted_edit")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        attempted_lines,
        locked_key_path: obj
            .get("locked_key_path")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        reason: obj
            .get("reason")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
    })
}
