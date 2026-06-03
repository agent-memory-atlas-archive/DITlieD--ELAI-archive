//! F-1 / A5: VibeGate SSB read surface for the Svelte `VibeGateDrawer`
//! panel.
//!
//! cross-lang-skip: tauri_commands
//! (covered by the dedicated `vibegate_drawer` pair, not the broad
//! `tauri_commands` / stub.ts wildcard).
//!
//! One read-only command:
//!
//! - [`list_vibegate_blocks`] -- last N `vibesec_code_vuln_blocked` +
//!   `vibesec_secret_blocked` rows (the two active VibeGateBlocked Pattern-2
//!   arms). `DepHallucinated` + `Skill` are RESERVED; their rows are
//!   included in the query so the drawer handles them gracefully when a
//!   future slice activates their producers.
//!
//! Default-ON (B1/W-2): the orchestrator writes `VibeGateBlocked` rows with NO
//! env override. The ONLY env var is the kill-switch `ELAI_VIBEGATE_DISABLED`
//! (default unset = enabled). This command always queries the SSB viewer and
//! returns whatever rows exist; the `env_active` field reports whether the gate
//! is enabled (true unless the kill-switch is set) so the drawer can show a
//! "gate disabled" hint only when an operator explicitly killed it.
//!
//! This module exposes the per-surface split required by W-1 wiring doctrine:
//! one command per distinct Svelte drawer key.
//!
//! cross-lang-pair: dashboard/tauri/api

use serde::{Deserialize, Serialize};
use tauri::State;

use elai_orchestrator::ssb::{SsbActionType, SsbPayload, VibeGateBlockedKind};
use elai_orchestrator::vibegate::ELAI_VIBEGATE_DISABLED_ENV;

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 500;

// ---------------------------------------------------------------------------
// DTO
// ---------------------------------------------------------------------------

/// One row of the VibeGateDrawer. Field names are in exact sync with the
/// TypeScript `VibeGateRow` interface in `src/lib/api/types.ts`.
///
/// `kind` is one of: `"CodeVuln"` | `"Secret"` | `"DepHallucinated"` |
/// `"Skill"`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VibeGateRow {
    pub seq: i64,
    pub ts: i64,
    pub kind: String,
    pub rule_id: Option<String>,
    pub cwe: Option<String>,
    pub file_path: String,
    pub source_blake3: String,
    pub citation: Option<String>,
    pub fsm_state: Option<String>,
}

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn is_env_truthy(var: &str) -> bool {
    std::env::var(var)
        .map(|v| matches!(v.as_str(), "1" | "true" | "yes" | "on" | "TRUE" | "YES" | "ON"))
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Command
// ---------------------------------------------------------------------------

/// Return the last `limit` `VibeGateBlocked` rows from SSB across all four
/// kind arms, newest first. Returns an empty Vec only when the SSB viewer is
/// not attached (fresh install) or no rows have been written yet.
///
/// The `env_active` field reports whether the gate is ENABLED (default-ON;
/// `true` unless the `ELAI_VIBEGATE_DISABLED` kill-switch is set), letting the
/// Svelte panel distinguish "no rows yet" from "gate disabled by operator".
#[tauri::command]
pub async fn list_vibegate_blocks(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<VibeGateBlocksResponse, String> {
    let cap = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let env_active = !is_env_truthy(ELAI_VIBEGATE_DISABLED_ENV);

    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => {
            return Ok(VibeGateBlocksResponse {
                rows: Vec::new(),
                env_active,
            });
        }
    };

    let action_types = [
        SsbActionType::VibeGateBlocked(VibeGateBlockedKind::CodeVuln).as_sql_str(),
        SsbActionType::VibeGateBlocked(VibeGateBlockedKind::Secret).as_sql_str(),
        SsbActionType::VibeGateBlocked(VibeGateBlockedKind::DepHallucinated).as_sql_str(),
        SsbActionType::VibeGateBlocked(VibeGateBlockedKind::Skill).as_sql_str(),
    ];

    let mut rows: Vec<VibeGateRow> = Vec::new();

    for at in &action_types {
        let raw = match viewer.recent_rows_of_type(at, cap) {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!(
                    action_type = at,
                    error = %e,
                    "list_vibegate_blocks: viewer error"
                );
                continue;
            }
        };

        for entry in raw {
            let pl: SsbPayload = match serde_json::from_value(entry.payload_json.clone()) {
                Ok(v) => v,
                Err(_) => continue,
            };
            if let SsbPayload::VibeGateBlocked(p) = pl {
                let kind = match p.kind {
                    VibeGateBlockedKind::CodeVuln => "CodeVuln",
                    VibeGateBlockedKind::Secret => "Secret",
                    VibeGateBlockedKind::DepHallucinated => "DepHallucinated",
                    VibeGateBlockedKind::Skill => "Skill",
                };
                rows.push(VibeGateRow {
                    seq: entry.seq,
                    ts: entry.ts,
                    kind: kind.to_string(),
                    rule_id: p.rule_id.clone(),
                    cwe: p.cwe.clone(),
                    file_path: p.file_path.clone(),
                    source_blake3: p.source_blake3.clone(),
                    citation: p.citation.clone(),
                    fsm_state: p.fsm_state.clone(),
                });
            }
        }
    }

    rows.sort_by(|a, b| b.seq.cmp(&a.seq));
    rows.truncate(cap);

    Ok(VibeGateBlocksResponse { rows, env_active })
}

/// Response envelope for [`list_vibegate_blocks`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VibeGateBlocksResponse {
    pub rows: Vec<VibeGateRow>,
    /// True when the VibeGate is enabled (default-ON; false only when the
    /// `ELAI_VIBEGATE_DISABLED` kill-switch is set).
    pub env_active: bool,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vibegate_row_serializes_expected_fields() {
        let row = VibeGateRow {
            seq: 1,
            ts: 1_700_000_000_000,
            kind: "CodeVuln".into(),
            rule_id: Some("SEC-SQLI-001".into()),
            cwe: Some("CWE-89".into()),
            file_path: "src/db.py".into(),
            source_blake3: "abcdef1234567890".into(),
            citation: None,
            fsm_state: Some("executing".into()),
        };
        let s = serde_json::to_string(&row).expect("serialize");
        for key in [
            "\"seq\"",
            "\"ts\"",
            "\"kind\"",
            "\"rule_id\"",
            "\"cwe\"",
            "\"file_path\"",
            "\"source_blake3\"",
            "\"fsm_state\"",
        ] {
            assert!(s.contains(key), "missing key {key} in {s}");
        }
    }

    #[test]
    fn vibegate_blocks_response_serializes_expected_fields() {
        let resp = VibeGateBlocksResponse {
            rows: Vec::new(),
            env_active: false,
        };
        let s = serde_json::to_string(&resp).expect("serialize");
        assert!(s.contains("\"rows\""), "missing rows in {s}");
        assert!(s.contains("\"env_active\""), "missing env_active in {s}");
    }
}
