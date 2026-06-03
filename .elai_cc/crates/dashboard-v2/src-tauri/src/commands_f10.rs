//! F-10 / Slice A5: SecurityClassRegressionBlocked SSB read surface for the
//! Svelte `SecurityMonotonicityDrawer` drawer.
//!
//! cross-lang-skip: tauri_commands
//! (covered by the dedicated `security_monotonicity_drawer` pair, not the
//! broad `tauri_commands` / stub.ts wildcard).
//!
//! One read-only command:
//!
//! - [`get_security_monotonicity_blocks`] -- last N
//!   `security_class_regression_blocked` rows emitted by the
//!   security-monotonicity iteration guard.
//!
//! Default-ON (W-2): the three production legs (REFINE call-site, AUTO-ADVANCE
//! evidence-check, SELF-MOD precondition) write
//! `security_class_regression_blocked` rows with NO env override. The ONLY env
//! var is the kill-switch `ELAI_SECURITY_MONOTONICITY_DISABLE` (never a
//! default-off ENABLE flag). This command always queries the SSB viewer; the
//! `env_active` field reports whether the guard is enabled (true unless the
//! kill-switch is set), letting the panel distinguish "no rows yet" from
//! "guard disabled by operator".
//!
//! Sibling of `commands_f2` (SecurityRulesPanel surface) and `commands_f9`
//! (DepsSecurityPanel surface): one command per distinct Svelte drawer key
//! (W-1 wiring doctrine).
//!
//! cross-lang-pair: dashboard/tauri/api

use serde::{Deserialize, Serialize};
use tauri::State;

use elai_orchestrator::ssb::{
    SsbActionType, SsbPayload, ELAI_SECURITY_MONOTONICITY_DISABLE_ENV,
};

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 500;

// ---------------------------------------------------------------------------
// DTO
// ---------------------------------------------------------------------------

/// One `security_class_regression_blocked` row for the
/// SecurityMonotonicityDrawer. Field names are in exact sync with the
/// TypeScript `SecurityMonotonicityRow` interface in `src/lib/api/types.ts`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecurityMonotonicityRow {
    pub seq: i64,
    /// SSB row timestamp (epoch millis).
    pub ts: i64,
    /// Production leg that raised the regression: `"refine"` / `"auto_advance"`
    /// / `"self_mod"` (snake-case wire label of `LoopKind`).
    pub loop_kind: String,
    /// Stable identifier of the iteration-loop instance.
    pub loop_id: String,
    /// Security-class labels newly introduced by this iteration (the
    /// regression set; non-empty when a row is emitted).
    pub introduced_classes: Vec<String>,
    /// Security-class labels present in the prior iteration (the monotonic
    /// baseline being compared against).
    pub prior_classes: Vec<String>,
    /// blake3 hex-prefix of the regressing candidate edit blob (never the full
    /// source text).
    pub delta_blake3: String,
    /// Snake-case language label (e.g. `"rust"`, `"python"`, `"typescript"`).
    pub lang: String,
}

/// Response envelope for [`get_security_monotonicity_blocks`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecurityMonotonicityResponse {
    /// Most recent rows, newest first, capped at `DEFAULT_LIMIT`.
    pub rows: Vec<SecurityMonotonicityRow>,
    /// True when the guard is ENABLED (default-ON; false only when the
    /// `ELAI_SECURITY_MONOTONICITY_DISABLE` kill-switch is set).
    pub env_active: bool,
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

/// Return the last `limit` `security_class_regression_blocked` rows from SSB,
/// newest first. Returns an empty Vec only when the SSB viewer is not attached
/// (fresh install) or no rows have been written yet.
///
/// The `env_active` field reports whether the guard is ENABLED (default-ON;
/// `true` unless the `ELAI_SECURITY_MONOTONICITY_DISABLE` kill-switch is set).
#[tauri::command]
pub async fn get_security_monotonicity_blocks(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<SecurityMonotonicityResponse, String> {
    let cap = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let env_active = !is_env_truthy(ELAI_SECURITY_MONOTONICITY_DISABLE_ENV);

    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => {
            return Ok(SecurityMonotonicityResponse {
                rows: Vec::new(),
                env_active,
            });
        }
    };

    let raw = viewer
        .recent_rows_of_type(
            SsbActionType::SecurityClassRegressionBlocked.as_sql_str(),
            cap,
        )
        .map_err(|e| e.to_string())?;

    let mut rows: Vec<SecurityMonotonicityRow> = Vec::new();
    for entry in raw {
        let pl: SsbPayload = match serde_json::from_value(entry.payload_json.clone()) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if let SsbPayload::SecurityClassRegressionBlocked(p) = pl {
            // serde renames LoopKind to snake_case; serialize to a String so
            // the DTO carries a stable wire label without re-importing the enum
            // on the TS side.
            let loop_kind = match serde_json::to_value(p.loop_kind) {
                Ok(serde_json::Value::String(s)) => s,
                _ => "unknown".to_owned(),
            };
            rows.push(SecurityMonotonicityRow {
                seq: entry.seq,
                ts: entry.ts,
                loop_kind,
                loop_id: p.loop_id.clone(),
                introduced_classes: p.introduced_classes.clone(),
                prior_classes: p.prior_classes.clone(),
                delta_blake3: p.delta_blake3.clone(),
                lang: p.lang.clone(),
            });
        }
    }

    Ok(SecurityMonotonicityResponse { rows, env_active })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn security_monotonicity_row_serializes_expected_fields() {
        let row = SecurityMonotonicityRow {
            seq: 7,
            ts: 1_700_000_000_002,
            loop_kind: "refine".into(),
            loop_id: "conv-42".into(),
            introduced_classes: vec!["CWE-78".into()],
            prior_classes: vec![],
            delta_blake3: "0123456789abcdef".into(),
            lang: "rust".into(),
        };
        let s = serde_json::to_string(&row).expect("serialize");
        for key in [
            "\"seq\"",
            "\"ts\"",
            "\"loop_kind\"",
            "\"loop_id\"",
            "\"introduced_classes\"",
            "\"prior_classes\"",
            "\"delta_blake3\"",
            "\"lang\"",
        ] {
            assert!(s.contains(key), "missing key {key} in {s}");
        }
    }

    #[test]
    fn security_monotonicity_response_serializes_expected_fields() {
        let resp = SecurityMonotonicityResponse {
            rows: Vec::new(),
            env_active: true,
        };
        let s = serde_json::to_string(&resp).expect("serialize");
        assert!(s.contains("\"rows\""), "missing rows in {s}");
        assert!(s.contains("\"env_active\""), "missing env_active in {s}");
    }
}
