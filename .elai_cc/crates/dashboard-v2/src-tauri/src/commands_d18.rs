//! D-18 — typed-ObservableEffect read-only Tauri command surface.
//!
//! One read-only command feeds the Svelte `ObservableEffectPanel`:
//!
//! - [`get_observable_effects_recent`] — last N
//!   `observable_effect_verified` SSB rows plus a per-sandbox-tier
//!   distribution + per-outcome histogram so the panel renders without
//!   re-walking the rows in JS.
//!
//! Additive — sibling of `commands_d11` (constructive checkers) +
//! `commands_d12` (baby-step predicate) + `commands_d16` (UniCR
//! conformal refusal). The orchestrator producer
//! (`elai_orchestrator::observable_effect::verify_observable_effect`)
//! is the sole writer of the underlying SSB rows; this module only
//! reads.
//!
//! Citation: see `crates/orchestrator/src/observable_effect.rs`
//! TODO_DOC[D-18] — the brief's arXiv:2412.16367 citation was rejected
//! (WebSearch May 2026 confirmed it is unrelated battery-thermal-runaway
//! work). Paper-ingestor follow-up should anchor against Voyager
//! arxiv:2305.16291 §3 + Coscientist arxiv:2304.05332 §4.
//!
//! See docs/thefinalglance/D-plan/D-18_observable_effect_typed.md.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 500;

/// Typed mirror of [`elai_orchestrator::observable_effect::ObservableEffectRow`]
/// — wire form returned to the Svelte panel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservableEffectRowDto {
    pub effect_id: String,
    pub trigger_kind: String,
    pub observation_kind: String,
    pub sandbox_tier: String,
    pub outcome: String,
    pub confound_flags: String,
    pub elapsed_us: u64,
    pub ts_ms: i64,
}

/// Aggregated response for the panel — recent rows + per-outcome
/// counts + per-sandbox-tier counts so the panel renders the
/// distribution chips without re-walking the rows in JS.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecentObservableEffects {
    pub rows: Vec<ObservableEffectRowDto>,
    /// pass | fail | skipped → count.
    pub counts_by_outcome: BTreeMap<String, u32>,
    /// no_sandbox | process_isolated | container_isolated |
    /// network_blocked → count.
    pub counts_by_sandbox_tier: BTreeMap<String, u32>,
    /// Total number of rows surfaced (always <= rows.len() cap).
    pub total: u32,
}

/// Return the most recent `observable_effect_verified` rows (capped at
/// `MAX_LIMIT`). Empty response when no SSB viewer is attached (Tauri-
/// dev fresh boot, no DB).
#[tauri::command]
pub async fn get_observable_effects_recent(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<RecentObservableEffects, String> {
    let cap = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => {
            return Ok(RecentObservableEffects {
                rows: Vec::new(),
                counts_by_outcome: BTreeMap::new(),
                counts_by_sandbox_tier: BTreeMap::new(),
                total: 0,
            });
        }
    };

    let raw =
        elai_orchestrator::observable_effect::recent_observable_effects(&viewer, cap);
    let mut counts_by_outcome: BTreeMap<String, u32> = BTreeMap::new();
    let mut counts_by_sandbox_tier: BTreeMap<String, u32> = BTreeMap::new();
    for r in &raw {
        *counts_by_outcome.entry(r.outcome.clone()).or_insert(0) += 1;
        *counts_by_sandbox_tier
            .entry(r.sandbox_tier.clone())
            .or_insert(0) += 1;
    }
    let rows: Vec<ObservableEffectRowDto> = raw
        .into_iter()
        .map(|r| ObservableEffectRowDto {
            effect_id: r.effect_id,
            trigger_kind: r.trigger_kind,
            observation_kind: r.observation_kind,
            sandbox_tier: r.sandbox_tier,
            outcome: r.outcome,
            confound_flags: r.confound_flags,
            elapsed_us: r.elapsed_us,
            ts_ms: r.ts_ms,
        })
        .collect();
    let total = u32::try_from(rows.len()).unwrap_or(u32::MAX);

    Ok(RecentObservableEffects {
        rows,
        counts_by_outcome,
        counts_by_sandbox_tier,
        total,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dto_serializes_with_expected_field_names() {
        let dto = ObservableEffectRowDto {
            effect_id: "e-1".into(),
            trigger_kind: "test_command".into(),
            observation_kind: "string_match".into(),
            sandbox_tier: "no_sandbox".into(),
            outcome: "pass".into(),
            confound_flags: String::new(),
            elapsed_us: 42,
            ts_ms: 1_700_000_000_000,
        };
        let s = serde_json::to_string(&dto).expect("serialize");
        for key in [
            "\"effect_id\"",
            "\"trigger_kind\"",
            "\"observation_kind\"",
            "\"sandbox_tier\"",
            "\"outcome\"",
            "\"confound_flags\"",
            "\"elapsed_us\"",
            "\"ts_ms\"",
        ] {
            assert!(s.contains(key), "missing key {key} in {s}");
        }
    }

    #[test]
    fn recent_response_serializes_with_expected_field_names() {
        let r = RecentObservableEffects {
            rows: Vec::new(),
            counts_by_outcome: BTreeMap::new(),
            counts_by_sandbox_tier: BTreeMap::new(),
            total: 0,
        };
        let s = serde_json::to_string(&r).expect("serialize");
        for key in [
            "\"rows\"",
            "\"counts_by_outcome\"",
            "\"counts_by_sandbox_tier\"",
            "\"total\"",
        ] {
            assert!(s.contains(key), "missing key {key} in {s}");
        }
    }
}
