//! F-7 / Slice A5: `TemplateUtilityRanked` SSB read surface for the Svelte
//! `TemplateRankingDrawer` drawer.
//!
//! One read-only command:
//!
//! - [`get_template_ranking`] -- last N `template_utility_ranked` rows emitted
//!   at the `CompiledExecutor::try_execute` gate-threshold seam (and by the
//!   `elai template-ranking list` CLI) when a crystallized-pipeline template's
//!   Laplace-smoothed outcome utility is computed.
//!
//! The drawer renders the smoothed `utility` and the raw recorded
//! `first_try_ratio` SIDE-BY-SIDE -- the before/after view of the F-7 change.
//! Laplace add-one smoothing means a 1/1 template scores `2/3 ~= 0.667` rather
//! than `1.0`, so a low-sample template stops being over-trusted; a high-sample
//! template's `utility` converges to its `first_try_ratio` (the two columns
//! line up once the evidence is thick).
//!
//! No wire-dark ENABLE gate (F-7 W-2 / B1): the beacon is default-ON. Any
//! `ELAI_TEMPLATE_CBR_*` env var is a kill-switch on the producer side, never a
//! default-off ENABLE. This command always queries the SSB viewer and returns
//! whatever rows exist (empty Vec on a fresh install with no rows yet).
//!
//! Sibling of `commands_f5` (StalenessQuarantineDrawer surface) and
//! `commands_f10` (SecurityMonotonicityDrawer surface): one command per distinct
//! Svelte drawer key (W-1 wiring doctrine).
//!
//! cross-lang-pair: dashboard/tauri/api

use serde::{Deserialize, Serialize};
use tauri::State;

use elai_orchestrator::ssb::{SsbActionType, SsbPayload};

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 500;

// ---------------------------------------------------------------------------
// DTO
// ---------------------------------------------------------------------------

/// One `template_utility_ranked` row for the TemplateRankingDrawer. Field names
/// are in exact sync with the TypeScript `TemplateRankingRow` interface in
/// `src/lib/api/types.ts`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplateRankingRow {
    /// SSB monotonic sequence number (newest first ordering key).
    pub seq: i64,
    /// SSB row timestamp (epoch millis).
    pub ts: i64,
    /// Concept-class key the crystallized template is bucketed under (one half
    /// of the cache key).
    pub concept_class: String,
    /// Stable hash of the plan skeleton (the other half of the cache key).
    pub plan_skeleton_hash: u64,
    /// Laplace-smoothed outcome utility:
    /// `(successes + ALPHA) / (total + ALPHA + BETA)`, `ALPHA = BETA = 1.0`.
    /// This is the "after" value the F-7 gate now compares.
    pub utility: f32,
    /// Raw recorded baseline: the persisted `first_try_success_rate` (the
    /// "before" value, kept for side-by-side display, W-2 no-removal).
    pub first_try_ratio: f32,
    /// Sample size (`total` firings = persisted `use_count`) backing the
    /// utility; low values are exactly where Laplace smoothing matters.
    pub sample_n: u32,
    /// Stable id of the crystallized pipeline template whose utility was ranked.
    pub chosen_pipeline_id: String,
}

/// Response envelope for [`get_template_ranking`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplateRankingResponse {
    /// Most recent rows, newest first, capped at `DEFAULT_LIMIT`.
    pub rows: Vec<TemplateRankingRow>,
}

// ---------------------------------------------------------------------------
// Command
// ---------------------------------------------------------------------------

/// Return the last `limit` `template_utility_ranked` rows from SSB, newest
/// first. Returns an empty Vec only when the SSB viewer is not attached (fresh
/// install) or no rows have been written yet.
///
/// The beacon has no wire-dark ENABLE gate (F-7 W-2): it is default-ON, so this
/// command always queries the SSB viewer.
#[tauri::command]
pub async fn get_template_ranking(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<TemplateRankingResponse, String> {
    let cap = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);

    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => {
            return Ok(TemplateRankingResponse { rows: Vec::new() });
        }
    };

    let raw = viewer
        .recent_rows_of_type(SsbActionType::TemplateUtilityRanked.as_sql_str(), cap)
        .map_err(|e| e.to_string())?;

    let mut rows: Vec<TemplateRankingRow> = Vec::new();
    for entry in raw {
        let pl: SsbPayload = match serde_json::from_value(entry.payload_json.clone()) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if let SsbPayload::TemplateUtilityRanked(p) = pl {
            rows.push(TemplateRankingRow {
                seq: entry.seq,
                ts: entry.ts,
                concept_class: p.concept_class.clone(),
                plan_skeleton_hash: p.plan_skeleton_hash,
                utility: p.utility,
                first_try_ratio: p.first_try_ratio,
                sample_n: p.sample_n,
                chosen_pipeline_id: p.chosen_pipeline_id.clone(),
            });
        }
    }

    Ok(TemplateRankingResponse { rows })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_ranking_row_serializes_expected_fields() {
        let row = TemplateRankingRow {
            seq: 7,
            ts: 1_700_000_000_007,
            concept_class: "rust_cli_scaffold".into(),
            plan_skeleton_hash: 0xDEAD_BEEF_CAFE_F00D,
            utility: 0.667,
            first_try_ratio: 1.0,
            sample_n: 1,
            chosen_pipeline_id: "tpl-abc123".into(),
        };
        let s = serde_json::to_string(&row).expect("serialize");
        for key in [
            "\"seq\"",
            "\"ts\"",
            "\"concept_class\"",
            "\"plan_skeleton_hash\"",
            "\"utility\"",
            "\"first_try_ratio\"",
            "\"sample_n\"",
            "\"chosen_pipeline_id\"",
        ] {
            assert!(s.contains(key), "missing key {key} in {s}");
        }
    }

    #[test]
    fn template_ranking_response_serializes_expected_fields() {
        let resp = TemplateRankingResponse { rows: Vec::new() };
        let s = serde_json::to_string(&resp).expect("serialize");
        assert!(s.contains("\"rows\""), "missing rows in {s}");
    }
}
