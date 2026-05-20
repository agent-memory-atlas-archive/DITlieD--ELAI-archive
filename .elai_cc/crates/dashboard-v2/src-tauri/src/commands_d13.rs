//! D-13 — adversary-run + cross-family verifier Tauri command surface.
//!
//! Two read-only commands feed the Svelte `AdversaryPanel`:
//!
//! - [`get_adversarial_runs_recent`] — last N joined
//!   `(adversary_generated + consensus_reevaluated_with_counter)` SSB
//!   row pairs (capped at MAX_LIMIT), sorted by `ts_ms` desc.
//! - [`get_adversary_config`] — the active
//!   [`elai_orchestrator::adversary::AdversaryConfig`] snapshot
//!   serialized for dashboard rendering (which model family is the
//!   active cross-family adversary, what severity floor is configured).
//!
//! Additive — sibling of `commands_d10` (mode routing) +
//! `commands_d11` (constructive checkers). The orchestrator producer
//! (`elai_orchestrator::agent_pool::run_adversarial_pass`) is the
//! sole writer of the underlying SSB rows; this module only reads.
//!
//! Grounding: arxiv:2403.07974 (T1) adversarial verification +
//! arxiv:2504.16760 (T1) lightweight latent verifiers (heterogeneous-
//! family verification).
//!
//! See docs/thefinalglance/D-plan/D-13_adversary_role_cross_family.md.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 500;

/// Typed mirror of `elai_orchestrator::agent_pool::AdversaryRunRow`.
/// Lives in this crate so the Tauri-side schema does not leak the
/// orchestrator's internal types through the IPC boundary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdversaryRunRowDto {
    pub counter_id: String,
    pub claim_id: String,
    pub severity: f64,
    pub model_family: String,
    pub original_verdict: String,
    pub post_counter_verdict: String,
    pub ts_ms: i64,
}

/// Aggregated response for the panel: recent joined rows + total count
/// + count of verdict inversions (rows where `original_verdict !=
/// post_counter_verdict`). The inversion count is what makes the
/// adversarial pass observably useful — if every original verdict
/// survives the counter-evidence pass, either the adversary is too
/// weak OR the verifier pool is genuinely robust. The panel surfaces
/// the count so the operator can spot the degenerate-no-effect case.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecentAdversarialRuns {
    pub rows: Vec<AdversaryRunRowDto>,
    /// `rows.len()` (redundant; keeps the panel's percentage math
    /// single-source).
    pub total: u32,
    /// Number of `rows` where `original_verdict != post_counter_verdict`.
    pub verdict_inversions: u32,
}

/// Return the most recent joined `adversary_generated` +
/// `consensus_reevaluated_with_counter` row pairs (capped at
/// `MAX_LIMIT`). Empty response when no SSB viewer is attached
/// (Tauri-dev fresh boot, no DB).
#[tauri::command]
pub async fn get_adversarial_runs_recent(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<RecentAdversarialRuns, String> {
    let cap = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => {
            return Ok(RecentAdversarialRuns {
                rows: Vec::new(),
                total: 0,
                verdict_inversions: 0,
            });
        }
    };

    let rows: Vec<AdversaryRunRowDto> =
        elai_orchestrator::agent_pool::recent_adversarial_runs(&viewer, cap)
            .into_iter()
            .map(|r| AdversaryRunRowDto {
                counter_id: r.counter_id,
                claim_id: r.claim_id,
                severity: r.severity,
                model_family: r.model_family,
                original_verdict: r.original_verdict,
                post_counter_verdict: r.post_counter_verdict,
                ts_ms: r.ts_ms,
            })
            .collect();

    let verdict_inversions = rows
        .iter()
        .filter(|r| !r.original_verdict.is_empty() && r.original_verdict != r.post_counter_verdict)
        .count();
    let total = u32::try_from(rows.len()).unwrap_or(u32::MAX);
    let inversions = u32::try_from(verdict_inversions).unwrap_or(u32::MAX);

    Ok(RecentAdversarialRuns {
        rows,
        total,
        verdict_inversions: inversions,
    })
}

/// DTO mirror of [`elai_orchestrator::agent_pool::AdversaryConfigDto`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdversaryConfigDto {
    pub adversary_model_family: String,
    pub max_counter_attempts: u32,
    pub severity_threshold: f64,
}

/// Return a snapshot of the currently configured
/// [`elai_orchestrator::adversary::AdversaryConfig`]. For the current
/// session the dashboard ships the default `default_for_local_worker`
/// snapshot (cloud Sonnet adversary, severity floor 0.5) — a future
/// patch will wire this to the orchestrator's loaded
/// `elai_cc.toml [adversary]` section. The default is deterministic
/// and matches the integration-test fixture, so the panel always has
/// a sensible render target.
#[tauri::command]
pub async fn get_adversary_config(
    _state: State<'_, AppState>,
) -> Result<AdversaryConfigDto, String> {
    let cfg = elai_orchestrator::adversary::AdversaryConfig::default_for_local_worker();
    let snap = elai_orchestrator::agent_pool::adversary_config_snapshot(&cfg);
    Ok(AdversaryConfigDto {
        adversary_model_family: snap.adversary_model_family,
        max_counter_attempts: snap.max_counter_attempts,
        severity_threshold: snap.severity_threshold,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_dto_serializes_with_expected_field_names() {
        let dto = AdversaryRunRowDto {
            counter_id: "k1".into(),
            claim_id: "c1".into(),
            severity: 0.8,
            model_family: "anthropic".into(),
            original_verdict: "pass".into(),
            post_counter_verdict: "fail".into(),
            ts_ms: 1_700_000_000_000,
        };
        let s = serde_json::to_string(&dto).expect("serialize");
        for key in [
            "\"counter_id\"",
            "\"claim_id\"",
            "\"severity\"",
            "\"model_family\"",
            "\"original_verdict\"",
            "\"post_counter_verdict\"",
            "\"ts_ms\"",
        ] {
            assert!(s.contains(key), "missing key {key} in {s}");
        }
    }

    #[test]
    fn config_dto_serializes_with_expected_field_names() {
        let dto = AdversaryConfigDto {
            adversary_model_family: "anthropic".into(),
            max_counter_attempts: 1,
            severity_threshold: 0.5,
        };
        let s = serde_json::to_string(&dto).expect("serialize");
        assert!(s.contains("\"adversary_model_family\""));
        assert!(s.contains("\"max_counter_attempts\""));
        assert!(s.contains("\"severity_threshold\""));
    }

    #[test]
    fn aggregated_response_serializes_with_expected_field_names() {
        let r = RecentAdversarialRuns {
            rows: Vec::new(),
            total: 0,
            verdict_inversions: 0,
        };
        let s = serde_json::to_string(&r).expect("serialize");
        assert!(s.contains("\"rows\""));
        assert!(s.contains("\"total\""));
        assert!(s.contains("\"verdict_inversions\""));
    }
}
