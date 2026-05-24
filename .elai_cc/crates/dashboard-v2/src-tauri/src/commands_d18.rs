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

/// D-18 closure: dispatched-row DTO for the
/// `observable_effect_dispatched` SSB variant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservableEffectDispatchedDto {
    pub effect_id: String,
    pub trigger: String,
    pub context_kind: String,
    pub ts_ms: i64,
}

/// D-18 closure: Goodhart-flag DTO for the `goodhart_risk_flag`
/// SSB variant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoodhartFlagDto {
    pub effect_id: String,
    pub risk_kind: String,
    pub evidence: String,
    pub ts_ms: i64,
}

/// D-18 closure: sandbox-observation DTO for the `sandbox_observation`
/// SSB variant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxObservationDto {
    pub effect_id: String,
    pub tier: String,
    pub outcome: String,
    pub evidence_count: u32,
    pub ts_ms: i64,
}

/// D-18 closure: production-observation-proposal DTO for the
/// `production_observation_proposal` SSB variant. Per Coscientist §4,
/// `requires_human_review` is `true` whenever `instrument_file` is
/// `None`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductionObservationProposalDto {
    pub effect_id: String,
    pub instrument_file: Option<String>,
    pub requires_human_review: bool,
    pub ts_ms: i64,
}

/// Aggregated response for the panel — recent rows + per-outcome
/// counts + per-sandbox-tier counts so the panel renders the
/// distribution chips without re-walking the rows in JS.
///
/// D-18 closure additive extension: also surfaces the 4 new SSB
/// variants in dedicated row vectors so the panel can render
/// dispatch / goodhart / sandbox / production-proposal sections
/// alongside the legacy `observable_effect_verified` rows.
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
    /// D-18 closure: dispatched-row family from the
    /// `observable_effect_dispatched` SSB variant.
    #[serde(default)]
    pub dispatched_rows: Vec<ObservableEffectDispatchedDto>,
    /// D-18 closure: Goodhart-flag family from the
    /// `goodhart_risk_flag` SSB variant.
    #[serde(default)]
    pub goodhart_flags: Vec<GoodhartFlagDto>,
    /// D-18 closure: sandbox-observation family from the
    /// `sandbox_observation` SSB variant.
    #[serde(default)]
    pub sandbox_observations: Vec<SandboxObservationDto>,
    /// D-18 closure: production-observation-proposal family from the
    /// `production_observation_proposal` SSB variant.
    #[serde(default)]
    pub production_proposals: Vec<ProductionObservationProposalDto>,
}

/// Return the most recent `observable_effect_verified` rows (capped at
/// `MAX_LIMIT`). Empty response when no SSB viewer is attached (Tauri-
/// dev fresh boot, no DB).
///
/// D-18 closure: additive — also surfaces the 4 new SSB variants
/// (observable_effect_dispatched + goodhart_risk_flag +
/// sandbox_observation + production_observation_proposal) in
/// dedicated row vectors. The base `rows` field stays the legacy
/// `observable_effect_verified` set so existing callers continue to
/// work without change.
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
                dispatched_rows: Vec::new(),
                goodhart_flags: Vec::new(),
                sandbox_observations: Vec::new(),
                production_proposals: Vec::new(),
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

    let dispatched_rows = read_dispatched_rows(&viewer, cap);
    let goodhart_flags = read_goodhart_flags(&viewer, cap);
    let sandbox_observations = read_sandbox_observations(&viewer, cap);
    let production_proposals = read_production_proposals(&viewer, cap);

    Ok(RecentObservableEffects {
        rows,
        counts_by_outcome,
        counts_by_sandbox_tier,
        total,
        dispatched_rows,
        goodhart_flags,
        sandbox_observations,
        production_proposals,
    })
}

fn read_dispatched_rows(
    viewer: &elai_orchestrator::ssb_viewer::SsbViewer,
    cap: usize,
) -> Vec<ObservableEffectDispatchedDto> {
    let mut out = Vec::new();
    let details = match viewer.recent_rows_of_type("observable_effect_dispatched", cap) {
        Ok(d) => d,
        Err(_) => return out,
    };
    for d in details {
        if let Some(dto) = extract_dispatched(d) {
            out.push(dto);
        }
    }
    out.sort_by(|a, b| b.ts_ms.cmp(&a.ts_ms));
    out.truncate(cap);
    out
}

fn extract_dispatched(
    d: elai_orchestrator::ssb_viewer::SsbActionDetail,
) -> Option<ObservableEffectDispatchedDto> {
    let outer = d.payload_json.as_object()?;
    let obj = if let Some(inner) = outer
        .get("ObservableEffectDispatched")
        .and_then(|v| v.as_object())
    {
        inner.clone()
    } else {
        outer.clone()
    };
    Some(ObservableEffectDispatchedDto {
        effect_id: obj
            .get("effect_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        trigger: obj
            .get("trigger")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        context_kind: obj
            .get("context_kind")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        ts_ms: obj.get("ts_ms").and_then(|v| v.as_i64()).unwrap_or(0),
    })
}

fn read_goodhart_flags(
    viewer: &elai_orchestrator::ssb_viewer::SsbViewer,
    cap: usize,
) -> Vec<GoodhartFlagDto> {
    let mut out = Vec::new();
    let details = match viewer.recent_rows_of_type("goodhart_risk_flag", cap) {
        Ok(d) => d,
        Err(_) => return out,
    };
    for d in details {
        let outer = match d.payload_json.as_object() {
            Some(o) => o,
            None => continue,
        };
        let obj = if let Some(inner) = outer
            .get("GoodhartRiskFlag")
            .and_then(|v| v.as_object())
        {
            inner.clone()
        } else {
            outer.clone()
        };
        out.push(GoodhartFlagDto {
            effect_id: obj
                .get("effect_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            risk_kind: obj
                .get("risk_kind")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            evidence: obj
                .get("evidence")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            ts_ms: obj.get("ts_ms").and_then(|v| v.as_i64()).unwrap_or(0),
        });
    }
    out.sort_by(|a, b| b.ts_ms.cmp(&a.ts_ms));
    out.truncate(cap);
    out
}

fn read_sandbox_observations(
    viewer: &elai_orchestrator::ssb_viewer::SsbViewer,
    cap: usize,
) -> Vec<SandboxObservationDto> {
    let mut out = Vec::new();
    let details = match viewer.recent_rows_of_type("sandbox_observation", cap) {
        Ok(d) => d,
        Err(_) => return out,
    };
    for d in details {
        let outer = match d.payload_json.as_object() {
            Some(o) => o,
            None => continue,
        };
        let obj = if let Some(inner) = outer
            .get("SandboxObservation")
            .and_then(|v| v.as_object())
        {
            inner.clone()
        } else {
            outer.clone()
        };
        out.push(SandboxObservationDto {
            effect_id: obj
                .get("effect_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            tier: obj
                .get("tier")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            outcome: obj
                .get("outcome")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            evidence_count: u32::try_from(
                obj.get("evidence_count").and_then(|v| v.as_u64()).unwrap_or(0),
            )
            .unwrap_or(u32::MAX),
            ts_ms: obj.get("ts_ms").and_then(|v| v.as_i64()).unwrap_or(0),
        });
    }
    out.sort_by(|a, b| b.ts_ms.cmp(&a.ts_ms));
    out.truncate(cap);
    out
}

fn read_production_proposals(
    viewer: &elai_orchestrator::ssb_viewer::SsbViewer,
    cap: usize,
) -> Vec<ProductionObservationProposalDto> {
    let mut out = Vec::new();
    let details = match viewer.recent_rows_of_type("production_observation_proposal", cap) {
        Ok(d) => d,
        Err(_) => return out,
    };
    for d in details {
        let outer = match d.payload_json.as_object() {
            Some(o) => o,
            None => continue,
        };
        let obj = if let Some(inner) = outer
            .get("ProductionObservationProposal")
            .and_then(|v| v.as_object())
        {
            inner.clone()
        } else {
            outer.clone()
        };
        let instrument_file = obj
            .get("instrument_file")
            .and_then(|v| if v.is_null() { None } else { v.as_str() })
            .map(|s| s.to_string());
        out.push(ProductionObservationProposalDto {
            effect_id: obj
                .get("effect_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            instrument_file,
            requires_human_review: obj
                .get("requires_human_review")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            ts_ms: obj.get("ts_ms").and_then(|v| v.as_i64()).unwrap_or(0),
        });
    }
    out.sort_by(|a, b| b.ts_ms.cmp(&a.ts_ms));
    out.truncate(cap);
    out
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
            dispatched_rows: Vec::new(),
            goodhart_flags: Vec::new(),
            sandbox_observations: Vec::new(),
            production_proposals: Vec::new(),
        };
        let s = serde_json::to_string(&r).expect("serialize");
        for key in [
            "\"rows\"",
            "\"counts_by_outcome\"",
            "\"counts_by_sandbox_tier\"",
            "\"total\"",
            "\"dispatched_rows\"",
            "\"goodhart_flags\"",
            "\"sandbox_observations\"",
            "\"production_proposals\"",
        ] {
            assert!(s.contains(key), "missing key {key} in {s}");
        }
    }

    #[test]
    fn d18_dispatched_dto_serializes_with_expected_keys() {
        let dto = ObservableEffectDispatchedDto {
            effect_id: "e1".into(),
            trigger: "test_command".into(),
            context_kind: "sandbox".into(),
            ts_ms: 100,
        };
        let s = serde_json::to_string(&dto).expect("serialize");
        for key in [
            "\"effect_id\"",
            "\"trigger\"",
            "\"context_kind\"",
            "\"ts_ms\"",
        ] {
            assert!(s.contains(key), "missing key {key} in {s}");
        }
    }

    #[test]
    fn d18_production_proposal_dto_serializes_with_expected_keys() {
        let dto = ProductionObservationProposalDto {
            effect_id: "e1".into(),
            instrument_file: None,
            requires_human_review: true,
            ts_ms: 100,
        };
        let s = serde_json::to_string(&dto).expect("serialize");
        for key in [
            "\"effect_id\"",
            "\"instrument_file\"",
            "\"requires_human_review\"",
            "\"ts_ms\"",
        ] {
            assert!(s.contains(key), "missing key {key} in {s}");
        }
    }

    #[test]
    fn d18_sandbox_observation_dto_serializes_with_expected_keys() {
        let dto = SandboxObservationDto {
            effect_id: "e1".into(),
            tier: "container_isolated".into(),
            outcome: "confirmed".into(),
            evidence_count: 2,
            ts_ms: 100,
        };
        let s = serde_json::to_string(&dto).expect("serialize");
        for key in [
            "\"effect_id\"",
            "\"tier\"",
            "\"outcome\"",
            "\"evidence_count\"",
            "\"ts_ms\"",
        ] {
            assert!(s.contains(key), "missing key {key} in {s}");
        }
    }
}
