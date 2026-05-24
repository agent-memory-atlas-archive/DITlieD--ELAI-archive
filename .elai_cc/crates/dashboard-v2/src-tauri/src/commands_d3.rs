//! D-3: Multi-verifier consensus Tauri command surface.
//!
//! One read-only command feeds the Svelte `VerificationConsensusPanel`:
//!
//! - [`get_recent_verifications`] — last N consensus rows (mix of
//!   `verifier_consensus_pass` + `verifier_consensus_fail`) from the SSB
//!   store, flattened to a typed DTO the frontend can render without a
//!   second lookup.
//!
//! Additive — runs alongside the existing `commands::*` + `commands_d1::*` +
//! `commands_d2::*` handlers without touching them. Aligns with the D-3
//! plan §panel: the panel renders chronologically ordered consensus
//! decisions with per-verifier breakdown + verdict + signal counts.

use std::cmp::Ordering;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 500;

/// One row of the panel. `kind` is either `"pass"` (consensus reached) or
/// `"fail"` (consensus rejected / inconclusive). `verifier_count` +
/// `pass_count` are populated for pass rows only; `failing_verifier_ids` +
/// `reason` are populated for fail rows only.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerificationConsensusRow {
    pub seq: i64,
    pub ts: i64,
    /// One of: `"pass"`, `"fail"`, `"ppm_unavailable"`,
    /// `"verifier_disagreement"`, `"cross_context_dispatched"`. The Svelte
    /// panel's `VerificationConsensusRow` union mirrors this set; widen
    /// both sides together when a new kind lands.
    pub kind: &'static str,
    pub candidate_id: String,
    pub template_id: String,
    pub verdict: String,
    pub rule_id: String,
    pub verifier_count: Option<u32>,
    pub pass_count: Option<u32>,
    pub failing_verifier_ids: Vec<String>,
    pub reason: Option<String>,
}

/// Return the last `limit` consensus rows from SSB, newest first.
/// Empty Vec when the SSB store is missing (fresh repo) or contains no
/// consensus rows.
#[tauri::command]
pub async fn get_recent_verifications(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<VerificationConsensusRow>, String> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(Vec::new()),
    };

    // D-3 closure: surface the 3 new SSB variants alongside the existing
    // pass / fail rows so the panel timeline carries the full consensus
    // history. The Svelte side widens its `kind` union to match.
    let mut combined = Vec::new();
    for at in [
        "verifier_consensus_pass",
        "verifier_consensus_fail",
        "ppm_unavailable",
        "verifier_disagreement",
        "cross_context_dispatched",
    ] {
        match viewer.recent_rows_of_type(at, limit) {
            Ok(rows) => {
                for detail in rows {
                    if let Some(row) = consensus_detail_to_row(detail) {
                        combined.push(row);
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    action_type = at,
                    error = %e,
                    "get_recent_verifications: viewer error"
                );
            }
        }
    }
    combined.sort_by(|a, b| b.seq.cmp(&a.seq).then(Ordering::Equal));
    combined.truncate(limit);
    Ok(combined)
}

fn consensus_detail_to_row(
    d: elai_orchestrator::SsbActionDetail,
) -> Option<VerificationConsensusRow> {
    match d.action_type.as_str() {
        "verifier_consensus_pass" => {
            let obj = d.payload_json.as_object()?;
            Some(VerificationConsensusRow {
                seq: d.seq,
                ts: d.ts,
                kind: "pass",
                candidate_id: obj.get("candidate_id")?.as_str()?.to_string(),
                template_id: obj.get("template_id")?.as_str()?.to_string(),
                verdict: obj
                    .get("verdict")
                    .and_then(|v| v.as_str())
                    .unwrap_or("pass")
                    .to_string(),
                rule_id: obj.get("rule_id")?.as_str()?.to_string(),
                verifier_count: obj
                    .get("verifier_count")
                    .and_then(|v| v.as_u64())
                    .and_then(|v| u32::try_from(v).ok()),
                pass_count: obj
                    .get("pass_count")
                    .and_then(|v| v.as_u64())
                    .and_then(|v| u32::try_from(v).ok()),
                failing_verifier_ids: Vec::new(),
                reason: None,
            })
        }
        "verifier_consensus_fail" => {
            let obj = d.payload_json.as_object()?;
            let failing: Vec<String> = obj
                .get("failing_verifier_ids")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|item| item.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            Some(VerificationConsensusRow {
                seq: d.seq,
                ts: d.ts,
                kind: "fail",
                candidate_id: obj.get("candidate_id")?.as_str()?.to_string(),
                template_id: obj.get("template_id")?.as_str()?.to_string(),
                verdict: "fail".to_string(),
                rule_id: obj
                    .get("rule_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                verifier_count: None,
                pass_count: None,
                failing_verifier_ids: failing,
                reason: obj
                    .get("reason")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
            })
        }
        // D-3 closure: PpmUnavailable — PPM sidecar down. Render with
        // candidate_id + verifier_id + reason.
        "ppm_unavailable" => {
            let obj = d.payload_json.as_object()?;
            let reason = obj
                .get("reason")
                .and_then(|v| v.as_str())
                .unwrap_or("ppm_unavailable")
                .to_string();
            let verifier_id = obj
                .get("verifier_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            Some(VerificationConsensusRow {
                seq: d.seq,
                ts: d.ts,
                kind: "ppm_unavailable",
                candidate_id: obj.get("candidate_id")?.as_str()?.to_string(),
                template_id: String::new(),
                verdict: "ppm_unavailable".to_string(),
                rule_id: String::new(),
                verifier_count: None,
                pass_count: None,
                failing_verifier_ids: vec![verifier_id],
                reason: Some(reason),
            })
        }
        // D-3 closure: VerifierDisagreement — 1+ Pass + 1+ Reject in the
        // same run. Render with both kinds + score delta.
        "verifier_disagreement" => {
            let obj = d.payload_json.as_object()?;
            let kind_a = obj
                .get("verifier_a_kind")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            let kind_b = obj
                .get("verifier_b_kind")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            let delta = obj.get("score_delta").and_then(|v| v.as_f64()).unwrap_or(0.0);
            Some(VerificationConsensusRow {
                seq: d.seq,
                ts: d.ts,
                kind: "verifier_disagreement",
                candidate_id: obj.get("candidate_id")?.as_str()?.to_string(),
                template_id: obj.get("template_id")?.as_str()?.to_string(),
                verdict: "verifier_disagreement".to_string(),
                rule_id: String::new(),
                verifier_count: None,
                pass_count: None,
                failing_verifier_ids: Vec::new(),
                reason: Some(format!("{kind_a} vs {kind_b} (Δ={delta:.3})")),
            })
        }
        // D-3 closure: CrossContextDispatched — emitted every cross-context
        // run regardless of agreement outcome. Render the agreement label
        // + jaccard.
        "cross_context_dispatched" => {
            let obj = d.payload_json.as_object()?;
            let agreement = obj
                .get("agreement")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string();
            let jaccard = obj.get("jaccard").and_then(|v| v.as_f64()).unwrap_or(0.0);
            Some(VerificationConsensusRow {
                seq: d.seq,
                ts: d.ts,
                kind: "cross_context_dispatched",
                candidate_id: obj
                    .get("claim_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                template_id: String::new(),
                verdict: agreement.clone(),
                rule_id: String::new(),
                verifier_count: None,
                pass_count: None,
                failing_verifier_ids: Vec::new(),
                reason: Some(format!("{agreement} (j={jaccard:.3})")),
            })
        }
        _ => None,
    }
}
