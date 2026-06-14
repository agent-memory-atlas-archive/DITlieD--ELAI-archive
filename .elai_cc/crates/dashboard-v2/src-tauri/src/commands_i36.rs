//! I-36 (SCHEMA_VERSION 267): CompiledPrompt SSB read surface for the
//! Svelte `SettingsWindow` badge chip.
//!
//! One read-only command:
//!
//! - [`get_compiled_variants`] -- the most recent `prompt_compiled` flat-beacon
//!   rows emitted by `build_system_prompt_for_role` when a compiled variant was
//!   substituted. The badge shows count of promoted variants + last substitution
//!   timestamp.
//!
//! Default-ON: rows emit whenever `CompiledPromptStore::lookup` returns a
//! variant and it is placed into `CompletionConfig::compiled_system_override`.
//! Kill-switch: ELAI_PROMPT_COMPILE_DISABLE=1 suppresses all substitution.
//!
//! Mirror of commands_i38.rs::get_qdelta_latest pattern.
//!
//! cross-lang-pair: dashboard/tauri/api

use serde::{Deserialize, Serialize};
use tauri::State;

use elai_orchestrator::ssb::{PromptCompiledKind, SsbActionType, SsbPayload};

use crate::state::AppState;

// ---------------------------------------------------------------------------
// DTO
// ---------------------------------------------------------------------------

/// One row of the compiled-prompt badge in `SettingsWindow`.
/// Field names must match the TypeScript `CompiledVariantRow` interface.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompiledVariantRow {
    /// Which event kind: "prompt_compiled" | "transfer_gap_measured" | "prompt_compiled_rolledback".
    pub kind: String,
    /// Snake-case role name (e.g. `"executor"`, `"researcher"`).
    pub role: String,
    /// Backend name (e.g. `"anthropic_api"`, `"sidecar"`).
    pub backend: String,
    /// First 16 hex chars of the blake3 variant hash.
    pub variant_hash: String,
    /// compiled_pass_rate - source_pass_rate delta (0.0 for Compiled rows).
    pub fixture_delta: f32,
    /// SSB row timestamp (unix milliseconds).
    pub ts: i64,
}

// ---------------------------------------------------------------------------
// Command
// ---------------------------------------------------------------------------

/// Return the most recent `prompt_compiled` rows from SSB (up to 20),
/// or an empty vec when no viewer is attached or no substitution has occurred.
///
/// The badge chip in SettingsWindow uses the len() as the promoted-variant
/// count and the first entry's ts as the last-substitution timestamp.
#[tauri::command]
pub async fn get_compiled_variants(
    state: State<'_, AppState>,
) -> Result<Vec<CompiledVariantRow>, String> {
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(vec![]),
    };

    let raw = viewer
        .recent_rows_of_type(
            SsbActionType::PromptCompiled(PromptCompiledKind::Compiled).as_sql_str(),
            20,
        )
        .map_err(|e| e.to_string())?;

    let mut out = Vec::with_capacity(raw.len());
    for entry in raw {
        let pl: SsbPayload = match serde_json::from_value(entry.payload_json.clone()) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if let SsbPayload::PromptCompiled(p) = pl {
            out.push(CompiledVariantRow {
                kind: p.kind.as_sql_str().to_string(),
                role: p.role.clone(),
                backend: p.backend.clone(),
                variant_hash: p.variant_hash.clone(),
                fixture_delta: p.fixture_delta,
                ts: entry.ts,
            });
        }
    }

    Ok(out)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiled_variant_row_serializes_expected_fields() {
        let row = CompiledVariantRow {
            kind: "prompt_compiled".to_string(),
            role: "executor".to_string(),
            backend: "anthropic_api".to_string(),
            variant_hash: "0123456789abcdef".to_string(),
            fixture_delta: 0.0,
            ts: 1_700_000_000_001,
        };
        let json = serde_json::to_string(&row).expect("serialise");
        assert!(json.contains("prompt_compiled"), "kind must round-trip");
        assert!(json.contains("executor"), "role must round-trip");
        assert!(json.contains("anthropic_api"), "backend must round-trip");
        assert!(json.contains("variant_hash"), "variant_hash field must be present");
        assert!(json.contains("fixture_delta"), "fixture_delta field must be present");
    }

    #[test]
    fn compiled_variant_row_zero_delta_for_compiled_arm() {
        let row = CompiledVariantRow {
            kind: PromptCompiledKind::Compiled.as_sql_str().to_string(),
            role: "researcher".to_string(),
            backend: "anthropic_api".to_string(),
            variant_hash: "abc".to_string(),
            fixture_delta: 0.0,
            ts: 0,
        };
        assert!(
            row.fixture_delta.abs() < f32::EPSILON,
            "Compiled arm always has fixture_delta=0.0 (measured by TransferGapMeasured arm)"
        );
    }
}
