//! F-9 / Slice D5: DepsSecurityPanel SSB read surface for the Svelte
//! `DepsSecurityPanel` drawer.
//!
//! cross-lang-skip: tauri_commands
//! (covered by the dedicated `deps_security_drawer` pair, not the broad
//! `tauri_commands` / stub.ts wildcard).
//!
//! One read-only command:
//!
//! - [`write_time_slop_status`] -- aggregates recent
//!   `write_time_slopsquat_blocked` SSB rows into a DTO with per-verdict-kind
//!   counts and a list of recent rows (newest-first, capped at 50).
//!
//! Default-ON (B1/W-2): `DepWriteTimeGate::analyze` writes
//! `WriteTimeSlopsquatBlocked` rows with NO env override. The ONLY kill-switch
//! is `ELAI_HOOKS_DISABLED` (global hook dispatcher disable). The `env_active`
//! field in the response reports whether the gate is active (true unless
//! `ELAI_HOOKS_DISABLED` is set), letting the panel distinguish "no rows yet"
//! from "hooks disabled by operator".
//!
//! Lookup helpers (`lookup_u32` / `lookup_i64` / `lookup_u64`) are re-declared
//! locally — they mirror the private helpers in `commands_e41.rs` but are NOT
//! imported (private visibility there). This is the canonical approach per the
//! D5 handoff notes.
//!
//! cross-lang-pair: dashboard/tauri/api

use serde::{Deserialize, Serialize};
use tauri::State;

use elai_orchestrator::hook_dispatcher::ELAI_HOOKS_DISABLED_ENV;
use elai_orchestrator::ssb::SsbActionType;

use crate::state::AppState;

type CmdResult<T> = Result<T, String>;

const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 200;

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

/// One recent `write_time_slopsquat_blocked` row for the DepsSecurityPanel.
/// Field names are in exact sync with the TypeScript `WriteTimeSlopRow`
/// interface in `src/lib/api/types.ts`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WriteTimeSlopRow {
    pub seq: i64,
    pub ts_ms: i64,
    /// blake3 64-bit prefix of the dependency name (never the raw name).
    pub dep_name_hash: u64,
    /// Package ecosystem: `"cargo"` / `"pypi"` / `"npm"`.
    pub ecosystem: String,
    /// Verdict kind label: `"not_found"` / `"too_new"`.
    pub verdict_kind: String,
    /// Package age in days (only present for the `"too_new"` arm).
    pub registry_age_days: Option<u32>,
}

/// Per-verdict-kind block count for the summary header.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WriteTimeSlopCount {
    pub verdict_kind: String,
    pub count: u32,
}

/// Top-level DTO returned by [`write_time_slop_status`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WriteTimeSlopStatusDto {
    /// Whether the dep-writetime gate is active (true unless
    /// `ELAI_HOOKS_DISABLED` is set — global hook dispatcher kill-switch).
    pub env_active: bool,
    /// Total number of `write_time_slopsquat_blocked` rows in the SSB (capped
    /// at the query window; not the true all-time total).
    pub total_blocked: u32,
    /// Per-verdict-kind breakdown from the recent rows.
    pub counts_by_verdict: Vec<WriteTimeSlopCount>,
    /// Most recent rows, newest first, capped at `DEFAULT_LIMIT`.
    pub recent_rows: Vec<WriteTimeSlopRow>,
}

// ---------------------------------------------------------------------------
// Payload lookup helpers (local re-declarations; mirrors commands_e41.rs)
// ---------------------------------------------------------------------------

fn lookup_u64(payload: &serde_json::Value, prefix: &str, key: &str) -> u64 {
    payload
        .get(prefix)
        .and_then(|v| v.get(key))
        .and_then(serde_json::Value::as_u64)
        .or_else(|| payload.get(key).and_then(serde_json::Value::as_u64))
        .unwrap_or(0)
}

/// Lookup helper present for API symmetry with `commands_e41.rs`; used in
/// the test suite below and reserved for future row fields.
#[allow(dead_code)]
fn lookup_u32(payload: &serde_json::Value, prefix: &str, key: &str) -> u32 {
    u32::try_from(lookup_u64(payload, prefix, key)).unwrap_or(u32::MAX)
}

fn lookup_i64(payload: &serde_json::Value, prefix: &str, key: &str) -> i64 {
    payload
        .get(prefix)
        .and_then(|v| v.get(key))
        .and_then(serde_json::Value::as_i64)
        .or_else(|| payload.get(key).and_then(serde_json::Value::as_i64))
        .unwrap_or(0)
}

fn lookup_str_opt(payload: &serde_json::Value, prefix: &str, key: &str) -> Option<String> {
    payload
        .get(prefix)
        .and_then(|v| v.get(key))
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
        .or_else(|| {
            payload
                .get(key)
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
        })
}

fn is_env_truthy(var: &str) -> bool {
    std::env::var(var)
        .map(|v| matches!(v.as_str(), "1" | "true" | "yes" | "on" | "TRUE" | "YES" | "ON"))
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Command
// ---------------------------------------------------------------------------

/// Aggregate recent `write_time_slopsquat_blocked` SSB rows into a summary DTO
/// for the DepsSecurityPanel (DrawerKey `deps-security`).
///
/// Returns an empty-state DTO when the SSB viewer is not attached (fresh
/// install) or when no rows have been written yet.
#[tauri::command]
pub async fn write_time_slop_status(state: State<'_, AppState>) -> CmdResult<WriteTimeSlopStatusDto> {
    let env_active = !is_env_truthy(ELAI_HOOKS_DISABLED_ENV);

    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => {
            return Ok(WriteTimeSlopStatusDto {
                env_active,
                total_blocked: 0,
                counts_by_verdict: Vec::new(),
                recent_rows: Vec::new(),
            });
        }
    };

    let cap = DEFAULT_LIMIT.min(MAX_LIMIT);

    let raw = viewer
        .recent_rows_of_type(
            SsbActionType::WriteTimeSlopsquatBlocked.as_sql_str(),
            cap,
        )
        .map_err(|e| e.to_string())?;

    let mut rows: Vec<WriteTimeSlopRow> = Vec::new();
    for entry in &raw {
        let dep_name_hash = lookup_u64(
            &entry.payload_json,
            "write_time_slopsquat_blocked",
            "dep_name_hash",
        );
        let ecosystem = lookup_str_opt(
            &entry.payload_json,
            "write_time_slopsquat_blocked",
            "ecosystem",
        )
        .unwrap_or_else(|| "unknown".to_owned());
        let verdict_kind = lookup_str_opt(
            &entry.payload_json,
            "write_time_slopsquat_blocked",
            "verdict_kind",
        )
        .unwrap_or_else(|| "not_found".to_owned());
        // registry_age_days is Option<u32> in the payload (skip_serializing_if absent).
        // Read directly from JSON to preserve null vs missing semantics.
        let registry_age_days = entry
            .payload_json
            .get("write_time_slopsquat_blocked")
            .and_then(|v| v.get("registry_age_days"))
            .or_else(|| entry.payload_json.get("registry_age_days"))
            .and_then(serde_json::Value::as_u64)
            .map(|n| n as u32);

        rows.push(WriteTimeSlopRow {
            seq: entry.seq,
            ts_ms: lookup_i64(
                &entry.payload_json,
                "write_time_slopsquat_blocked",
                "ts_ms",
            ),
            dep_name_hash,
            ecosystem,
            verdict_kind,
            registry_age_days,
        });
    }

    // Newest-first (viewer returns newest-first already, but sort for safety).
    rows.sort_by(|a, b| b.seq.cmp(&a.seq));

    let total_blocked = u32::try_from(rows.len()).unwrap_or(u32::MAX);

    // Per-verdict-kind counts. BTreeMap (not HashMap) per CLAUDE.md coding
    // standards; the result is sorted by count below, so iteration order here
    // is not observable (behavior-neutral swap).
    let mut count_map: std::collections::BTreeMap<String, u32> = std::collections::BTreeMap::new();
    for row in &rows {
        *count_map.entry(row.verdict_kind.clone()).or_insert(0) += 1;
    }
    let mut counts_by_verdict: Vec<WriteTimeSlopCount> = count_map
        .into_iter()
        .map(|(verdict_kind, count)| WriteTimeSlopCount { verdict_kind, count })
        .collect();
    counts_by_verdict.sort_by(|a, b| b.count.cmp(&a.count));

    Ok(WriteTimeSlopStatusDto {
        env_active,
        total_blocked,
        counts_by_verdict,
        recent_rows: rows,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_time_slop_row_serializes_expected_fields() {
        let row = WriteTimeSlopRow {
            seq: 1,
            ts_ms: 1_700_000_000_000,
            dep_name_hash: 0xdeadbeef_cafe1234,
            ecosystem: "cargo".into(),
            verdict_kind: "not_found".into(),
            registry_age_days: None,
        };
        let s = serde_json::to_string(&row).expect("serialize");
        for key in [
            "\"seq\"",
            "\"ts_ms\"",
            "\"dep_name_hash\"",
            "\"ecosystem\"",
            "\"verdict_kind\"",
        ] {
            assert!(s.contains(key), "missing key {key} in {s}");
        }
    }

    #[test]
    fn write_time_slop_status_dto_serializes_expected_fields() {
        let dto = WriteTimeSlopStatusDto {
            env_active: true,
            total_blocked: 0,
            counts_by_verdict: Vec::new(),
            recent_rows: Vec::new(),
        };
        let s = serde_json::to_string(&dto).expect("serialize");
        for key in [
            "\"env_active\"",
            "\"total_blocked\"",
            "\"counts_by_verdict\"",
            "\"recent_rows\"",
        ] {
            assert!(s.contains(key), "missing key {key} in {s}");
        }
    }

    #[test]
    fn write_time_slop_count_serializes_expected_fields() {
        let c = WriteTimeSlopCount {
            verdict_kind: "not_found".into(),
            count: 3,
        };
        let s = serde_json::to_string(&c).expect("serialize");
        assert!(s.contains("\"verdict_kind\""), "missing verdict_kind in {s}");
        assert!(s.contains("\"count\""), "missing count in {s}");
    }

    #[test]
    fn lookup_u64_prefers_prefixed_key() {
        let v: serde_json::Value = serde_json::json!({
            "write_time_slopsquat_blocked": { "dep_name_hash": 42 },
            "dep_name_hash": 99,
        });
        assert_eq!(lookup_u64(&v, "write_time_slopsquat_blocked", "dep_name_hash"), 42);
    }

    #[test]
    fn lookup_u64_falls_back_to_bare_key() {
        let v: serde_json::Value = serde_json::json!({ "dep_name_hash": 77 });
        assert_eq!(lookup_u64(&v, "write_time_slopsquat_blocked", "dep_name_hash"), 77);
    }
}
