//! G-10 (SCHEMA_VERSION 218): ModelDriftSuspected SSB read surface for the
//! Svelte `ModelDriftPanel` drawer.
//!
//! One read-only command:
//!
//! - [`get_model_drift_status`] -- the most recent `model_drift_suspected`
//!   Pattern-2 beacon row emitted at `elai run` exit or `elai model-drift check`.
//!   Surfaces the identity_key + kind + signals + per_family_delta +
//!   affected_template_ids + confirmed_by_fingerprint + baseline_captured_at.
//!
//! Default-ON: `model_drift_suspected` rows emit when drift is detected.
//! The check itself is default-ON; kill-switch ELAI_MODEL_DRIFT_DISABLE.
//!
//! HONESTY (GD-5): cloud-lane rows always have `confirmed_by_fingerprint=false`
//! (Anthropic's system_fingerprint is opaque). The panel shows a
//! fingerprint-confirmed/behavioral badge based on this field. See
//! workflow/reasoning/G-10-decisions.md.
//!
//! Mirror of commands_g6.rs::get_safety_delta_latest per context-package
//! instruction (consumes_symbols SC-3 verified anchor).
//!
//! cross-lang-pair: dashboard/tauri/api

use serde::{Deserialize, Serialize};
use tauri::State;

use elai_orchestrator::ssb::{SsbActionType, SsbPayload};

use crate::state::AppState;

// ---------------------------------------------------------------------------
// DTO
// ---------------------------------------------------------------------------

/// Status DTO returned to the Svelte `ModelDriftPanel` drawer.
/// Field names MUST byte-match the TypeScript `ModelDriftStatus` interface
/// in `src/lib/api/types.ts` (cross_lang_diff enforced).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelDriftStatus {
    /// blake3 identity key of the (model_pin, fingerprint, lane) tuple.
    pub identity_key: String,
    /// Primary drift signal label (one of: fingerprint_changed, canary_regressed,
    /// malformed_output_spike).
    pub kind: String,
    /// All drift signal labels that fired (ordered).
    pub signals: Vec<String>,
    /// Per-family regression delta: (family_label, transfer_score) pairs.
    /// Negative transfer_score = regression; threshold is DEFAULT_FAIL_BAND (0.05).
    pub per_family_delta: Vec<(String, f64)>,
    /// Domain labels of regressed families (for review routing surface, DD5).
    pub affected_template_ids: Vec<String>,
    /// `true` ONLY for LOCAL lane + changed system fingerprint (GD-5).
    /// NEVER `true` for CLOUD lane (behavioral-suspected only).
    pub confirmed_by_fingerprint: bool,
    /// Wall-clock ms at which the stored baseline was captured.
    pub baseline_captured_at: u64,
    /// SSB row timestamp (unix milliseconds).
    pub ts: i64,
}

// ---------------------------------------------------------------------------
// Command
// ---------------------------------------------------------------------------

/// Return the most recent `model_drift_suspected` Pattern-2 beacon row from
/// SSB, or `None` when no viewer is attached or no drift has been detected yet.
///
/// Read-only mirror of [`crate::commands_g6::get_safety_delta_latest`]:
/// queries the SSB viewer for the single newest `model_drift_suspected` row
/// and projects it into [`ModelDriftStatus`].
#[tauri::command]
pub async fn get_model_drift_status(
    state: State<'_, AppState>,
) -> Result<Option<ModelDriftStatus>, String> {
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(None),
    };

    let raw = viewer
        .recent_rows_of_type(SsbActionType::ModelDriftSuspected.as_sql_str(), 1)
        .map_err(|e| e.to_string())?;

    for entry in raw {
        let pl: SsbPayload = match serde_json::from_value(entry.payload_json.clone()) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if let SsbPayload::ModelDriftSuspected(p) = pl {
            return Ok(Some(ModelDriftStatus {
                identity_key: p.identity_key.clone(),
                kind: p.kind.label().to_owned(),
                signals: p.signals.clone(),
                per_family_delta: p.per_family_delta.clone(),
                affected_template_ids: p.affected_template_ids.clone(),
                confirmed_by_fingerprint: p.confirmed_by_fingerprint,
                baseline_captured_at: p.baseline_captured_at,
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
    fn model_drift_status_serializes_expected_fields() {
        let row = ModelDriftStatus {
            identity_key: "0123456789abcdef".to_string(),
            kind: "canary_regressed".to_string(),
            signals: vec!["canary_regressed".to_string()],
            per_family_delta: vec![("code_fix".to_string(), -0.20)],
            affected_template_ids: vec!["domain:code_fix".to_string()],
            confirmed_by_fingerprint: false,
            baseline_captured_at: 1_700_000_000_000,
            ts: 1_700_000_000_001,
        };
        let json = serde_json::to_string(&row).expect("serialise");
        assert!(json.contains("identity_key"), "JSON must contain identity_key");
        assert!(json.contains("kind"), "JSON must contain kind");
        assert!(json.contains("signals"), "JSON must contain signals");
        assert!(
            json.contains("confirmed_by_fingerprint"),
            "JSON must contain confirmed_by_fingerprint"
        );
        assert!(
            json.contains("per_family_delta"),
            "JSON must contain per_family_delta"
        );
        assert!(
            json.contains("affected_template_ids"),
            "JSON must contain affected_template_ids"
        );
        assert!(
            json.contains("baseline_captured_at"),
            "JSON must contain baseline_captured_at"
        );

        let parsed: ModelDriftStatus =
            serde_json::from_str(&json).expect("round-trip");
        assert_eq!(parsed.kind, "canary_regressed");
        assert!(!parsed.confirmed_by_fingerprint);
        assert_eq!(parsed.baseline_captured_at, 1_700_000_000_000);
    }
}
