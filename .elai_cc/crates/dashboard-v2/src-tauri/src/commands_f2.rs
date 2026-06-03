//! F-2a / A5: SecurityRuleBlocked SSB read surface for the Svelte
//! `SecurityRulesPanel` drawer.
//!
//! cross-lang-skip: tauri_commands
//! (covered by the dedicated `security_rules_drawer` pair, not the broad
//! `tauri_commands` / stub.ts wildcard).
//!
//! Two read-only commands:
//!
//! - [`get_security_blocks_recent`] -- last N `security_rule_blocked`
//!   flat-beacon rows.
//! - [`get_insecure_rate_latest`] -- the most recent
//!   `insecure_rate_measured` flat-beacon row (the static-gate insecure-rate
//!   keystone: before -> after + gate_recall). This is the REAL ELAI static
//!   write-gate (`evaluate_vibegate`) measured over a SeCodePLT-structured
//!   CWE corpus; it is DISTINCT from the SeCodePLT dynamic secure-coding rate.
//!
//! Default-ON (B1/W-2): the orchestrator writes `security_rule_blocked` rows
//! with NO env override. The ONLY env var is the shared kill-switch
//! `ELAI_VIBEGATE_DISABLED` (default unset = enabled). This command always
//! queries the SSB viewer; the `env_active` field reports whether the gate is
//! enabled (true unless the kill-switch is set).
//!
//! Sibling of `commands_vibegate` (VibeGate surface). This module exposes the
//! per-surface split required by W-1 wiring doctrine: one command per distinct
//! Svelte drawer key.
//!
//! cross-lang-pair: dashboard/tauri/api

use serde::{Deserialize, Serialize};
use tauri::State;

use elai_orchestrator::ssb::{SsbActionType, SsbPayload};
use elai_orchestrator::vibegate::ELAI_VIBEGATE_DISABLED_ENV;

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 500;

// ---------------------------------------------------------------------------
// DTO
// ---------------------------------------------------------------------------

/// One row of the SecurityRulesPanel. Field names are in exact sync with
/// the TypeScript `SecurityRuleRow` interface in `src/lib/api/types.ts`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecurityRuleRow {
    pub seq: i64,
    pub ts: i64,
    pub rule_id: String,
    pub cwe: String,
    pub file_path: String,
    pub language: String,
    pub candidate_blake3: String,
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

/// Return the last `limit` `security_rule_blocked` flat-beacon rows from
/// SSB, newest first. Returns an empty Vec only when the SSB viewer is not
/// attached (fresh install) or no rows have been written yet.
///
/// The `env_active` field reports whether the gate is ENABLED (default-ON;
/// `true` unless the `ELAI_VIBEGATE_DISABLED` kill-switch is set).
#[tauri::command]
pub async fn get_security_blocks_recent(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<SecurityBlocksResponse, String> {
    let cap = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let env_active = !is_env_truthy(ELAI_VIBEGATE_DISABLED_ENV);

    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => {
            return Ok(SecurityBlocksResponse {
                rows: Vec::new(),
                env_active,
            });
        }
    };

    let raw = viewer
        .recent_rows_of_type(SsbActionType::SecurityRuleBlocked.as_sql_str(), cap)
        .map_err(|e| e.to_string())?;

    let mut rows: Vec<SecurityRuleRow> = Vec::new();
    for entry in raw {
        let pl: SsbPayload = match serde_json::from_value(entry.payload_json.clone()) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if let SsbPayload::SecurityRuleBlocked(p) = pl {
            rows.push(SecurityRuleRow {
                seq: entry.seq,
                ts: entry.ts,
                rule_id: p.rule_id.clone(),
                cwe: p.cwe.clone(),
                file_path: p.file_path.clone(),
                language: p.language.clone(),
                candidate_blake3: p.candidate_blake3.clone(),
            });
        }
    }

    Ok(SecurityBlocksResponse { rows, env_active })
}

/// Response envelope for [`get_security_blocks_recent`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecurityBlocksResponse {
    pub rows: Vec<SecurityRuleRow>,
    /// True when the gate is enabled (default-ON; false only when the
    /// `ELAI_VIBEGATE_DISABLED` kill-switch is set).
    pub env_active: bool,
}

// ---------------------------------------------------------------------------
// Insecure-rate keystone DTO + command (F-2b / Slice A5)
// ---------------------------------------------------------------------------

/// One `insecure_rate_measured` flat-beacon row, projected for the
/// SecurityRulesPanel before/after tile. Field names are in exact sync with
/// the TypeScript `InsecureRateRow` interface in `src/lib/api/types.ts`.
///
/// `insecure_rate_before` / `insecure_rate_after` / `gate_recall` are the
/// STATIC-gate insecure-rate keystone: the REAL ELAI static write-gate
/// (`evaluate_vibegate`) measured over a SeCodePLT-structured CWE corpus.
/// This is DISTINCT from the SeCodePLT dynamic secure-coding rate (the
/// dynamic 3-way score, functional pass@1, and the live-model number are a
/// separate, RunPod-gated measurement and are NOT carried here).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InsecureRateRow {
    /// Benchmark suite / harness label.
    pub suite: String,
    /// Model identifier of the candidate-generation source (provenance).
    pub model_id: String,
    /// Number of candidate completions evaluated.
    pub n_samples: u32,
    /// Corpus ground-truth insecure-rate across all candidates (before).
    pub insecure_rate_before: f64,
    /// Insecure-rate of the gate-approved subset (after = FN residual).
    pub insecure_rate_after: f64,
    /// Derived static-gate recall: (before - after) / before.
    pub gate_recall: f64,
    /// SSB row timestamp (epoch ms) of this measurement.
    pub ts: i64,
}

/// Return the most recent `insecure_rate_measured` flat-beacon row from SSB,
/// or `None` when the viewer is not attached (fresh install) or no
/// measurement has been recorded yet.
///
/// Read-only mirror of [`get_security_blocks_recent`]: it queries the SSB
/// viewer for the single newest `insecure_rate_measured` row and projects it
/// into [`InsecureRateRow`].
#[tauri::command]
pub async fn get_insecure_rate_latest(
    state: State<'_, AppState>,
) -> Result<Option<InsecureRateRow>, String> {
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(None),
    };

    let raw = viewer
        .recent_rows_of_type(SsbActionType::InsecureRateMeasured.as_sql_str(), 1)
        .map_err(|e| e.to_string())?;

    for entry in raw {
        let pl: SsbPayload = match serde_json::from_value(entry.payload_json.clone()) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if let SsbPayload::InsecureRateMeasured(p) = pl {
            return Ok(Some(InsecureRateRow {
                suite: p.suite.clone(),
                model_id: p.model_id.clone(),
                n_samples: p.n_samples,
                insecure_rate_before: p.insecure_rate_before,
                insecure_rate_after: p.insecure_rate_after,
                gate_recall: p.gate_recall,
                ts: entry.ts,
            }));
        }
    }

    Ok(None)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn security_rule_row_serializes_expected_fields() {
        let row = SecurityRuleRow {
            seq: 2,
            ts: 1_700_000_000_001,
            rule_id: "SEC-XSS-001".into(),
            cwe: "CWE-79".into(),
            file_path: "src/template.ts".into(),
            language: "typescript".into(),
            candidate_blake3: "fedcba0987654321".into(),
        };
        let s = serde_json::to_string(&row).expect("serialize");
        for key in [
            "\"seq\"",
            "\"ts\"",
            "\"rule_id\"",
            "\"cwe\"",
            "\"file_path\"",
            "\"language\"",
            "\"candidate_blake3\"",
        ] {
            assert!(s.contains(key), "missing key {key} in {s}");
        }
    }

    #[test]
    fn security_blocks_response_serializes_expected_fields() {
        let resp = SecurityBlocksResponse {
            rows: Vec::new(),
            env_active: true,
        };
        let s = serde_json::to_string(&resp).expect("serialize");
        assert!(s.contains("\"rows\""), "missing rows in {s}");
        assert!(s.contains("\"env_active\""), "missing env_active in {s}");
    }

    #[test]
    fn insecure_rate_row_serializes_expected_fields() {
        let row = InsecureRateRow {
            suite: "secodeplt-structured-cwe".into(),
            model_id: "qwen2.5-coder-7b".into(),
            n_samples: 320,
            insecure_rate_before: 0.42,
            insecure_rate_after: 0.11,
            gate_recall: 0.738,
            ts: 1_700_000_000_002,
        };
        let s = serde_json::to_string(&row).expect("serialize");
        for key in [
            "\"suite\"",
            "\"model_id\"",
            "\"n_samples\"",
            "\"insecure_rate_before\"",
            "\"insecure_rate_after\"",
            "\"gate_recall\"",
            "\"ts\"",
        ] {
            assert!(s.contains(key), "missing key {key} in {s}");
        }
    }
}
