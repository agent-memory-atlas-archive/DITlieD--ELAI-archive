//! E-10 -- hostile-critic structural simplifications dashboard surface.
//!
//! One read-only Tauri command feeds the Svelte CriticDecisionPanel:
//!
//! - get_critic_decision_state -- most recent critic_decision_event SSB
//!   rows projected into per-tile state for the 6-tile dashboard panel.
//!
//! Additive -- sibling of commands_e6 (MAST panel). The orchestrator
//! producer modules (fsm_ablation / role_ablation / rag_challenger /
//! cot_proof / cache_check / grc_blockquote) are the sole writers of
//! the underlying SSB rows; this module only reads.
//!
//! Paper grounding: arXiv:2502.00674 Self-MoA + arXiv:2503.13657 MAST
//! + arXiv:2510.01069 typed-CoT + arXiv:2307.13702 Lanham +
//! arXiv:2510.04040 FaithCoT-Bench. See
//! workflow/reasoning/E-10-decisions.md.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 100;
const MAX_LIMIT: usize = 1000;

/// One tile of the CriticDecisionPanel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CriticTileState {
    /// Snake-case wire label (e.g. "fsm_ablation").
    pub phase: String,
    /// Baseline arm score from the most recent row.
    pub baseline_score: f64,
    /// Challenger arm score from the most recent row.
    pub challenger_score: f64,
    /// Paired-bootstrap p-value (None if not available).
    pub paired_bootstrap_p: Option<f64>,
    /// Sample size of the bootstrap window.
    pub sample_size: u32,
    /// 0 = Keep, 1 = Simplify.
    pub verdict_code: u8,
    /// Wall-clock ts of the most recent row.
    pub ts_ms: i64,
}

/// Aggregated panel response. 6 tiles per DECISION-15: CloudDefault +
/// Cache fold into Tier; GrcBlockquote becomes a header badge;
/// MistakeBookSignal becomes a status badge.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CriticDecisionState {
    /// Per-tile most-recent state, keyed by phase wire label.
    pub tiles: Vec<CriticTileState>,
    /// Total critic_decision_event rows scanned.
    pub grand_total: u32,
}

/// Return the most recent critic_decision_event SSB rows projected into
/// per-tile state. Empty response when no SSB viewer is attached.
#[tauri::command]
pub async fn get_critic_decision_state(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<CriticDecisionState, String> {
    let cap = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(CriticDecisionState { tiles: Vec::new(), grand_total: 0 }),
    };
    let raw_rows = match viewer.recent_rows_of_type("critic_decision_event", cap) {
        Ok(d) => d,
        Err(e) => {
            tracing::warn!(
                action_type = "critic_decision_event",
                error = %e,
                "get_critic_decision_state: viewer error"
            );
            return Ok(CriticDecisionState { tiles: Vec::new(), grand_total: 0 });
        }
    };

    let mut latest: BTreeMap<String, CriticTileState> = BTreeMap::new();
    let mut grand_total: u32 = 0;
    for d in &raw_rows {
        let outer = match d.payload_json.as_object() {
            Some(o) => o,
            None => continue,
        };
        let obj = if let Some(inner) = outer.get("CriticDecisionEvent").and_then(|v| v.as_object()) {
            inner.clone()
        } else {
            outer.clone()
        };
        let phase = obj.get("phase").and_then(|v| v.as_str()).unwrap_or("").to_string();
        if phase.is_empty() { continue; }
        let ts_ms = obj.get("ts_ms").and_then(|v| v.as_i64()).unwrap_or(0);
        let candidate = CriticTileState {
            phase: phase.clone(),
            baseline_score: obj.get("baseline_score").and_then(|v| v.as_f64()).unwrap_or(0.0),
            challenger_score: obj.get("challenger_score").and_then(|v| v.as_f64()).unwrap_or(0.0),
            paired_bootstrap_p: obj.get("paired_bootstrap_p").and_then(|v| v.as_f64()),
            sample_size: obj.get("sample_size").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            verdict_code: obj.get("verdict_code").and_then(|v| v.as_u64()).unwrap_or(0) as u8,
            ts_ms,
        };
        match latest.get(&phase) {
            Some(existing) if existing.ts_ms >= ts_ms => {}
            _ => { latest.insert(phase, candidate); }
        }
        grand_total = grand_total.saturating_add(1);
    }
    Ok(CriticDecisionState { tiles: latest.into_values().collect(), grand_total })
}
