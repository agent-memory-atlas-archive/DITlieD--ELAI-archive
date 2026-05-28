//! E-6 — iso-budget ablation harness.
//!
//! Compares a full 8-role ELAI pipeline against a single-agent +
//! "Wait" reflection baseline at equal thinking-token budgets.
//! Output: a deterministic [`IsoBudgetReport`] + an `IsoBudgetMeasured`
//! SSB row so the dashboard's MastFailurePanel can render the
//! per-MAST-mode breakdown.
//!
//! ## Grounding
//!
//! - arXiv:2604.02460 (Tran & Kiela 2026) — single-agent matches or
//!   beats MAS on multi-hop tasks under equal thinking-token caps via
//!   the Data Processing Inequality (message passing loses information
//!   each hop).
//! - arXiv:2507.02778 (Tsui 2025) — appending `"Wait"` as a reflection
//!   token reduces single-agent blind-spot rate by 89.3% on the
//!   Self-Correction Bench corpus. The single-agent baseline in this
//!   harness mimics that pattern.
//! - arXiv:2502.00674 (Li & Lin 2025) — heterogeneous ensembling loses
//!   6.6% on AlpacaEval 2.0 vs sampling the single best with itself.
//!   Motivates the "iso-budget proof of lift" gate this harness emits.
//!
//! ## Scope
//!
//! Host-side scaffolding ONLY. The actual full-pipeline + single-agent
//! baseline runs require the remote 3090 sidecar + cloud-Sonnet route
//! (cross-family) + live multi-agent execution against the orchestrator's
//! FSM. Marked `RUNPOD-PENDING` in [`run_ablation`]. The deterministic
//! report path returns a stub with `runpod_pending=true` so the
//! dashboard panel renders and the CLI smoke-tests the wire format
//! end-to-end without live inference.

use std::collections::BTreeMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::mast::MastMode;
use crate::ssb::{IsoBudgetMeasuredPayload, MastModeCount, Ssb, SsbError, SsbPayload};
use elai_ipc_types::AgentRole;

// ---------------------------------------------------------------------------
// IsoBudgetReport — the deterministic harness output
// ---------------------------------------------------------------------------

/// Pass-rate + per-role token + per-MAST-mode count for one row of the
/// ablation table. Two rows per [`IsoBudgetReport`]: full-pipeline +
/// single-agent-with-wait.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IsoBudgetRow {
    /// Row label: `"full_pipeline"` or `"single_agent_wait"`. Stable
    /// wire tag.
    pub label: String,
    /// Pass rate in [0.0, 1.0] over the fixture's task set.
    pub pass_rate: f64,
    /// Per-role total token spend across the row's runs. `BTreeMap`
    /// for deterministic ordering.
    pub per_role_tokens: BTreeMap<AgentRole, u64>,
    /// Per-MAST-mode failure count across the row's runs.
    pub mast_mode_counts: BTreeMap<MastMode, u32>,
}

impl IsoBudgetRow {
    /// Empty row for the named pipeline. Used as the deterministic
    /// stub return shape when no live run is available.
    #[must_use]
    pub fn empty(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            pass_rate: 0.0,
            per_role_tokens: BTreeMap::new(),
            mast_mode_counts: BTreeMap::new(),
        }
    }
}

/// Result of one iso-budget ablation run. Two rows + a derived
/// `insufficient_lift` flag the dashboard surfaces in the
/// MastFailurePanel header strip.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IsoBudgetReport {
    /// Path to the task fixture used (TOML).
    pub task_fixture: String,
    /// Thinking-token budget applied to both rows.
    pub budget_tokens: u32,
    /// Full-pipeline (8-role) row.
    pub full_pipeline: IsoBudgetRow,
    /// Single-agent + "Wait" reflection baseline row.
    pub single_agent_wait: IsoBudgetRow,
    /// True iff `full_pipeline.pass_rate - single_agent_wait.pass_rate`
    /// is NOT >= 1 stddev of the fixture's seeds. Surfaced so the user
    /// sees the architectural cost on the dashboard.
    pub insufficient_lift: bool,
    /// True iff this report is a deterministic stub (host-side scaffold
    /// returned without a live multi-agent run). Always `true` until
    /// the RUNPOD-PENDING wiring is closed.
    pub runpod_pending: bool,
}

// ---------------------------------------------------------------------------
// IsoBudgetError
// ---------------------------------------------------------------------------

/// Errors surfaced by [`run_ablation`]. Hard signals — callers do not
/// retry silently.
#[derive(Debug, thiserror::Error)]
pub enum IsoBudgetError {
    /// SSB write failed (typically backpressure / DB lock).
    #[error("iso-budget SSB write failed: {0}")]
    Ssb(#[from] SsbError),
    /// Live run dispatch is not yet wired — caller invoked the live path
    /// without the RUNPOD-PENDING closure landing.
    #[error("iso-budget live dispatch is RUNPOD-PENDING: {0}")]
    RunpodPending(String),
}

// ---------------------------------------------------------------------------
// run_ablation — host-side entry
// ---------------------------------------------------------------------------

/// Run the iso-budget ablation harness and emit an `IsoBudgetMeasured`
/// SSB row. Currently returns a deterministic stub
/// `IsoBudgetReport` with both rows empty + `runpod_pending=true`;
/// the live full-pipeline + single-agent baseline runs are
/// RUNPOD-PENDING (require live multi-agent execution against the
/// remote 3090 sidecar with the cross-family adversary path active).
///
/// `task_fixture` is the path to the fixture TOML; `budget_tokens` is
/// the thinking-token cap applied to BOTH rows. Both inputs are
/// preserved in the emitted SSB row so post-hoc analysis can
/// reconstruct the run config.
pub async fn run_ablation(
    ssb: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
    task_fixture: &str,
    budget_tokens: u32,
) -> Result<IsoBudgetReport, IsoBudgetError> {
    // RUNPOD-PENDING: live 8-role ELAI pipeline + single-agent-with-Wait
    // baseline runs not yet wired. The host-side scaffolding below
    // (struct + SSB emit + CLI dispatch) ships so the wire-format is
    // exercised end-to-end. When the live wiring lands, the empty rows
    // below are replaced with measured pass_rate + per-role tokens +
    // per-MAST-mode counts.
    let report = IsoBudgetReport {
        task_fixture: task_fixture.to_owned(),
        budget_tokens,
        full_pipeline: IsoBudgetRow::empty("full_pipeline"),
        single_agent_wait: IsoBudgetRow::empty("single_agent_wait"),
        insufficient_lift: true,
        runpod_pending: true,
    };

    let payload = SsbPayload::IsoBudgetMeasured(IsoBudgetMeasuredPayload {
        task_fixture: report.task_fixture.clone(),
        budget_tokens: report.budget_tokens,
        full_pipeline_pass_rate: report.full_pipeline.pass_rate,
        single_agent_wait_pass_rate: report.single_agent_wait.pass_rate,
        insufficient_lift: report.insufficient_lift,
        runpod_pending: report.runpod_pending,
        mast_mode_counts: aggregate_mast_counts(&report),
    });
    ssb.write_payload(agent_id, superstep, payload, Vec::new())
        .await?;
    Ok(report)
}

/// Aggregate per-MAST-mode counts across both rows of the report.
/// Used by the SSB payload to give the dashboard a single histogram
/// without the consumer having to merge the two rows itself.
fn aggregate_mast_counts(report: &IsoBudgetReport) -> Vec<MastModeCount> {
    let mut sum: BTreeMap<MastMode, u32> = BTreeMap::new();
    for (mode, n) in &report.full_pipeline.mast_mode_counts {
        *sum.entry(*mode).or_insert(0) = sum.get(mode).copied().unwrap_or(0).saturating_add(*n);
    }
    for (mode, n) in &report.single_agent_wait.mast_mode_counts {
        *sum.entry(*mode).or_insert(0) = sum.get(mode).copied().unwrap_or(0).saturating_add(*n);
    }
    sum.into_iter()
        .map(|(mode, count)| MastModeCount {
            mast_mode: mode.as_sql_str().to_owned(),
            count,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssb::SsbConfig;
    use elai_core::clock::SeededClock;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn test_clock() -> Arc<SeededClock> {
        Arc::new(SeededClock::new())
    }

    fn test_config(dir: &TempDir) -> SsbConfig {
        SsbConfig {
            db_path: dir.path().join("iso_budget_ssb.db"),
            pool_size: Some(2),
            project_id: "test-proj".into(),
            session_id: "test-session".into(),
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn run_ablation_emits_iso_budget_measured_row() {
        let dir = TempDir::new().expect("tempdir");
        let ssb = Ssb::new(test_config(&dir), test_clock()).expect("ssb new");
        let ssb_arc = Arc::new(ssb);

        let report = run_ablation(&ssb_arc, "test-agent", 0, "fixtures/smoke.toml", 2000)
            .await
            .expect("run_ablation must succeed in stub mode");

        assert!(report.runpod_pending, "stub run must mark runpod_pending=true");
        assert!(report.insufficient_lift, "empty rows imply zero lift");
        assert_eq!(report.budget_tokens, 2000);
        assert_eq!(report.full_pipeline.label, "full_pipeline");
        assert_eq!(report.single_agent_wait.label, "single_agent_wait");
    }

    #[test]
    fn empty_row_zero_pass_rate_zero_counts() {
        let r = IsoBudgetRow::empty("test");
        assert_eq!(r.pass_rate, 0.0);
        assert!(r.per_role_tokens.is_empty());
        assert!(r.mast_mode_counts.is_empty());
    }

    #[test]
    fn aggregate_counts_sums_across_rows() {
        let mut full = IsoBudgetRow::empty("full_pipeline");
        full.mast_mode_counts.insert(MastMode::Fm13StepRepetition, 3);
        let mut sing = IsoBudgetRow::empty("single_agent_wait");
        sing.mast_mode_counts.insert(MastMode::Fm13StepRepetition, 5);
        sing.mast_mode_counts.insert(MastMode::Fm22FailToAskForClarification, 2);
        let report = IsoBudgetReport {
            task_fixture: "f".into(),
            budget_tokens: 1000,
            full_pipeline: full,
            single_agent_wait: sing,
            insufficient_lift: true,
            runpod_pending: true,
        };
        let agg = aggregate_mast_counts(&report);
        assert_eq!(agg.len(), 2);
        // Find Fm13 sum 8
        let fm13 = agg
            .iter()
            .find(|c| c.mast_mode == "fm_1_3_step_repetition")
            .expect("fm_1_3 present");
        assert_eq!(fm13.count, 8);
    }
}
