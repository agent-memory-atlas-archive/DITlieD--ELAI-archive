//! I-33 (SCHEMA_VERSION 240): GovernancePrimitiveFired SSB read surface for
//! the Svelte `GovernanceMetricsChip` component.
//!
//! One read-only command:
//!
//! - [`get_governance_metrics`] -- aggregates the most recent N
//!   `governance_primitive_fired` rows into CDL / DIU / FSR governance
//!   metrics that the dashboard chip renders.
//!
//! Default-ON: `governance_primitive_fired` rows emit when
//! `elai bench governance-probe` runs or when the production
//! `governed_prompt_decision` call site fires.
//!
//! cross-lang-pair: dashboard/tauri/api

use serde::{Deserialize, Serialize};
use tauri::State;

use elai_orchestrator::ssb::{GovernancePrimitiveFiredKind, SsbActionType, SsbPayload};

use crate::state::AppState;

/// Governance metrics aggregated from recent `governance_primitive_fired` SSB rows.
///
/// Field names must match the TypeScript `GovernanceMetrics` interface in
/// `src/lib/api/types.ts`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GovernanceMetricsPayload {
    /// CDL: Cosmetic-Deadlock rate = CeflExternalized count / total rows.
    /// Proportion of decisions where the model received CEFL framing.
    pub cdl: f64,
    /// DIU: Deferral Info Utilisation = I6qRejected count / total rows.
    /// Proportion of decisions where a vacuous rationale was caught.
    pub diu: f64,
    /// FSR: Framing Success Rate = CommitRevealSealed count where the
    /// details_json does NOT contain "out_of_order" / total CommitRevealSealed.
    /// A rate of 1.0 means all sealed decisions had correct ordering.
    pub fsr: f64,
    /// Total number of `governance_primitive_fired` rows sampled.
    pub row_count: usize,
}

// ---------------------------------------------------------------------------
// Command
// ---------------------------------------------------------------------------

/// Return aggregated governance metrics from the most recent 50
/// `governance_primitive_fired` rows, or `None` when no viewer is attached
/// or no governance rows have been recorded.
///
/// Metric definitions:
/// - CDL = CeflExternalized count / total
/// - DIU = I6qRejected count / total
/// - FSR = CommitRevealSealed rows where details_json lacks "out_of_order" / CommitRevealSealed count
#[tauri::command]
pub async fn get_governance_metrics(
    state: State<'_, AppState>,
) -> Result<Option<GovernanceMetricsPayload>, String> {
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(None),
    };

    let tag = SsbActionType::GovernancePrimitiveFired(
        GovernancePrimitiveFiredKind::CeflExternalized,
    )
    .as_sql_str();
    let raw = viewer
        .recent_rows_of_type(tag, 50)
        .map_err(|e| e.to_string())?;

    if raw.is_empty() {
        return Ok(None);
    }

    let total = raw.len();
    let mut cefl_count: usize = 0;
    let mut i6q_count: usize = 0;
    let mut seal_total: usize = 0;
    let mut seal_clean: usize = 0;

    for entry in &raw {
        let pl: SsbPayload = match serde_json::from_value(entry.payload_json.clone()) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if let SsbPayload::GovernancePrimitiveFired(p) = pl {
            match p.kind {
                GovernancePrimitiveFiredKind::CeflExternalized => cefl_count += 1,
                GovernancePrimitiveFiredKind::I6qRejected => i6q_count += 1,
                GovernancePrimitiveFiredKind::CommitRevealSealed => {
                    seal_total += 1;
                    if !p.details_json.contains("out_of_order") {
                        seal_clean += 1;
                    }
                }
            }
        }
    }

    let cdl = cefl_count as f64 / total as f64;
    let diu = i6q_count as f64 / total as f64;
    let fsr = if seal_total > 0 {
        seal_clean as f64 / seal_total as f64
    } else {
        1.0
    };

    Ok(Some(GovernanceMetricsPayload {
        cdl,
        diu,
        fsr,
        row_count: total,
    }))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn governance_metrics_payload_serde_round_trip() {
        let m = GovernanceMetricsPayload {
            cdl: 0.5,
            diu: 0.1,
            fsr: 0.9,
            row_count: 10,
        };
        let json = serde_json::to_string(&m).expect("serialize");
        let decoded: GovernanceMetricsPayload =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(m, decoded);
    }
}
