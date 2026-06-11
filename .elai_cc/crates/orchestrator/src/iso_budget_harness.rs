//! E-6 — iso-budget ablation harness.
//!
//! Compares a full 8-role ELAI pipeline against a single-agent +
//! "Wait" reflection baseline at equal thinking-token budgets.
//! Output: a deterministic [`IsoBudgetReport`] + an `IsoBudgetMeasured`
//! SSB row so the dashboard's MastFailurePanel can render the
//! per-MAST-mode breakdown.
//!
//! H-40 extends this harness with a per-role decorrelation axis
//! (`unique_catch_rate` + `decorrelation_score`) and changes the
//! prunable predicate from `insufficient_lift` alone to
//! `insufficient_contribution = low_lift AND low_unique_catch`, so a
//! decorrelated low-accuracy verifier is retained rather than pruned.
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
//! - arXiv:2506.18203 (Weaver 2025) — weak verifiers close the
//!   generation-verification gap when their errors are conditionally
//!   independent given ground truth. A marginal-lift-only prune gate
//!   wrongly deletes a decorrelated verifier that catches ensemble misses.
//!   H-40 adds the unique-catch / decorrelation axis (RD-2).
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
// H-40 decorrelation constants — LOCKED floors (dated 2026-06-08).
//
// These are provisional until >=3 live iso-budget runs under ELAI_SEED land
// (RUNPOD-PENDING). Until then the deletion gate in `run_ablation` is guarded
// by `runpod_pending=true` and WILL NOT act on `insufficient_contribution`.
// GD-4: calibrate before trusting the auto-prune.
// ---------------------------------------------------------------------------

/// Lift floor: full_pipeline_pass_rate - single_agent_wait_pass_rate < this
/// => the role contributes insufficient marginal lift.
/// Locked 2026-06-08 at 0.05 (5 pp); GD-4 calibration updates this value.
const LOW_LIFT_FLOOR: f64 = 0.05;

/// Unique-catch floor: fraction of ensemble failures role R alone got right
/// < this => role catches too few misses to justify its cost when lift is
/// also low. Locked 2026-06-08 at 0.10 (10%); GD-4 calibration updates.
const LOW_UNIQUE_CATCH_FLOOR: f64 = 0.10;

// ---------------------------------------------------------------------------
// H-40 PerRoleOutcomes — per-item per-role pass/fail outcome matrix.
//
// RUNPOD-PENDING: the live path does not yet populate this matrix; the
// production harness stubs it empty and guards on `runpod_pending=true`
// before acting on `insufficient_contribution`. The contract test supplies a
// frozen synthetic fixture that exercises the decorrelation math directly.
// ---------------------------------------------------------------------------

/// Per-item outcome for a single agent role.
/// `outcomes[i] = true` iff the role passed item `i`.
/// `ground_truth[i] = true` iff item `i` is considered correct by the
/// external ground-truth label (a sound checker output, not self-assessed).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerRoleOutcomes {
    /// Ordered pass/fail booleans, one per eval item.
    pub outcomes: Vec<bool>,
    /// Ground-truth labels aligned with `outcomes`.
    pub ground_truth: Vec<bool>,
}

impl PerRoleOutcomes {
    /// Build from aligned bool slices.
    #[must_use]
    pub fn new(outcomes: Vec<bool>, ground_truth: Vec<bool>) -> Self {
        Self { outcomes, ground_truth }
    }
}

// ---------------------------------------------------------------------------
// H-40 decorrelation metric functions
// ---------------------------------------------------------------------------

/// Compute `unique_catch_rate(R)` for role `R` given the per-role outcome
/// matrix and consensus outcomes.
///
/// Definition: fraction of ensemble FAILURES (items the consensus got wrong,
/// i.e. `consensus[i] = false`) that role `R` alone got RIGHT
/// (`role.outcomes[i] = true`).
///
/// If there are no ensemble failures the rate is 0.0 (vacuously zero — no
/// ensemble failures means the axis is undefined; treat as non-decorrelated).
///
/// Deterministic integer/ratio math; no floating-point accumulation beyond
/// the final division. Seed-stable per GD-2.
#[must_use]
pub fn unique_catch_rate(
    role: &PerRoleOutcomes,
    consensus: &[bool],
) -> f64 {
    debug_assert_eq!(
        role.outcomes.len(),
        consensus.len(),
        "unique_catch_rate: outcome and consensus lengths must match"
    );
    let ensemble_failures: u64 = consensus.iter().filter(|&&c| !c).count() as u64;
    if ensemble_failures == 0 {
        return 0.0;
    }
    let unique_catches: u64 = role
        .outcomes
        .iter()
        .zip(consensus.iter())
        .filter(|(&r, &c)| r && !c)
        .count() as u64;
    unique_catches as f64 / ensemble_failures as f64
}

/// Compute `decorrelation_score(R)` for role `R` against all other roles.
///
/// Definition: 1 - mean pairwise error-correlation of role R with every
/// other role, conditional on ground truth = false (items the ground truth
/// says are failures). Correlation here is the fraction of ground-truth-fail
/// items where BOTH role R and role X also fail (joint error rate).
///
/// Score of 1.0 = perfectly decorrelated (R's errors are independent of all
/// other roles). Score of 0.0 = perfectly correlated with all others.
///
/// Returns 1.0 when there are fewer than 2 roles or no ground-truth-fail
/// items (undefined; treat as fully decorrelated = retain the role).
#[must_use]
pub fn decorrelation_score(
    role: &PerRoleOutcomes,
    all_roles: &BTreeMap<AgentRole, PerRoleOutcomes>,
    this_role: &AgentRole,
) -> f64 {
    let n = role.outcomes.len();
    if n == 0 || all_roles.len() < 2 {
        return 1.0;
    }
    // Items where ground truth = false (the role got it wrong).
    let gt_fail_indices: Vec<usize> = role
        .ground_truth
        .iter()
        .enumerate()
        .filter(|(_, &gt)| !gt)
        .map(|(i, _)| i)
        .collect();
    if gt_fail_indices.is_empty() {
        return 1.0;
    }
    let gt_fail_n = gt_fail_indices.len() as f64;
    // Role R's error indicator on GT-fail items.
    let r_errors: Vec<bool> = gt_fail_indices
        .iter()
        .map(|&i| !role.outcomes[i])
        .collect();

    let r_err_count: f64 = r_errors.iter().filter(|&&e| e).count() as f64;
    let p_r = r_err_count / gt_fail_n;

    let mut correlation_sum = 0.0_f64;
    let mut pair_count = 0_u64;
    for (other_role, other) in all_roles {
        if other_role == this_role {
            continue;
        }
        if other.outcomes.len() != n {
            continue;
        }
        let x_err_count: f64 = gt_fail_indices
            .iter()
            .filter(|&&i| !other.outcomes[i])
            .count() as f64;
        let p_x = x_err_count / gt_fail_n;
        let p_joint: f64 = gt_fail_indices
            .iter()
            .zip(r_errors.iter())
            .filter(|(&i, &r_err)| r_err && !other.outcomes[i])
            .count() as f64
            / gt_fail_n;
        // Pearson phi for two Bernoulli error indicators:
        //   phi = (p_joint - p_r*p_x) / sqrt(p_r*(1-p_r) * p_x*(1-p_x))
        // Zero-variance guard: a role that never errs (p_r=0) or always errs
        // (p_r=1) has undefined correlation with any other role; treat it as
        // 0 correlation (fully decorrelated, retain the role) per the
        // conservative posture in fill_decorrelation.
        let var_r = p_r * (1.0 - p_r);
        let var_x = p_x * (1.0 - p_x);
        let denom = (var_r * var_x).sqrt();
        let corr = if denom < 1e-9 {
            0.0
        } else {
            (p_joint - p_r * p_x) / denom
        };
        correlation_sum += corr.abs().clamp(0.0, 1.0);
        pair_count += 1;
    }
    if pair_count == 0 {
        return 1.0;
    }
    let mean_corr = correlation_sum / pair_count as f64;
    (1.0 - mean_corr).clamp(0.0, 1.0)
}

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

/// Result of one iso-budget ablation run. Two rows + derived flags the
/// dashboard surfaces in the MastFailurePanel header strip.
///
/// H-40 adds `unique_catch_rate`, `decorrelation_score`, and
/// `insufficient_contribution` fields. The prunable predicate changes from
/// `insufficient_lift` alone to `insufficient_contribution = low_lift AND
/// low_unique_catch` so a decorrelated low-accuracy verifier is retained.
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
    /// is NOT >= 1 stddev of the fixture's seeds. Kept for backward
    /// compatibility; `insufficient_contribution` is the gating flag.
    pub insufficient_lift: bool,
    /// True iff this report is a deterministic stub (host-side scaffold
    /// returned without a live multi-agent run). Always `true` until
    /// the RUNPOD-PENDING wiring is closed.
    pub runpod_pending: bool,
    // --- H-40 decorrelation fields ---
    /// Per-role unique-catch rate (snake_case AgentRole wire tag -> rate).
    /// Fraction of ensemble failures that each role alone got right.
    /// Empty on stub runs (RUNPOD-PENDING).
    pub unique_catch_rate: BTreeMap<String, f64>,
    /// Per-role decorrelation score (snake_case AgentRole wire tag -> score).
    /// 1 - mean pairwise error-correlation conditional on ground truth.
    /// Empty on stub runs (RUNPOD-PENDING).
    pub decorrelation_score: BTreeMap<String, f64>,
    /// H-40 prunable predicate per role: `low_lift AND low_unique_catch_for_THAT_role`.
    /// Map key is the snake_case AgentRole wire tag; value is `true` iff that
    /// role is prunable. A decorrelated low-lift role has `false` and is RETAINED
    /// (RD-2). Callers MUST check `runpod_pending` before acting; the stub path
    /// returns an empty map (conservative: unknown prunable state for all roles).
    pub insufficient_contribution: BTreeMap<String, bool>,
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
    //
    // H-40: PerRoleOutcomes is also RUNPOD-PENDING (live per-item per-role
    // pass/fail vectors require actual inference runs). The stub path sets
    // unique_catch_rate=0.0, decorrelation_score=0.0, and
    // insufficient_contribution=true (conservative: prunable). Callers
    // MUST check runpod_pending before acting on insufficient_contribution.
    let report = IsoBudgetReport {
        task_fixture: task_fixture.to_owned(),
        budget_tokens,
        full_pipeline: IsoBudgetRow::empty("full_pipeline"),
        single_agent_wait: IsoBudgetRow::empty("single_agent_wait"),
        insufficient_lift: true,
        runpod_pending: true,
        // H-40 stub values (RUNPOD-PENDING, conservative posture).
        // insufficient_contribution is empty on stub: unknown per-role state.
        // Callers must check runpod_pending before acting on this map.
        unique_catch_rate: BTreeMap::new(),
        decorrelation_score: BTreeMap::new(),
        insufficient_contribution: BTreeMap::new(),
    };

    // H-40: stub path has empty maps; clone directly for SSB payload.
    // insufficient_contribution is an empty map on the stub path (unknown state,
    // conservative: callers must check runpod_pending before acting).
    let ucr_map: BTreeMap<String, f64> = report.unique_catch_rate.clone();
    let ds_map: BTreeMap<String, f64> = report.decorrelation_score.clone();
    let ic_map: BTreeMap<String, bool> = report.insufficient_contribution.clone();

    let payload = SsbPayload::IsoBudgetMeasured(IsoBudgetMeasuredPayload {
        task_fixture: report.task_fixture.clone(),
        budget_tokens: report.budget_tokens,
        full_pipeline_pass_rate: report.full_pipeline.pass_rate,
        single_agent_wait_pass_rate: report.single_agent_wait.pass_rate,
        insufficient_lift: report.insufficient_lift,
        runpod_pending: report.runpod_pending,
        mast_mode_counts: aggregate_mast_counts(&report),
        unique_catch_rate: ucr_map,
        decorrelation_score: ds_map,
        insufficient_contribution: ic_map,
    });
    ssb.write_payload(agent_id, superstep, payload, Vec::new())
        .await?;
    Ok(report)
}

/// Run the iso-budget ablation harness with live per-role outcome data.
///
/// This is the production entry point called when RUNPOD-PENDING is resolved
/// and real per-item per-role pass/fail vectors are available. It calls
/// [`run_ablation`] for the base SSB scaffolding (struct + row emit), then
/// immediately calls [`fill_decorrelation`] to populate H-40 metrics and
/// re-emits the enriched payload as a second SSB row (so the dashboard gets
/// the enriched version while the stub row is a tombstone).
///
/// `role_outcomes` maps each `AgentRole` to its per-item pass/fail vector
/// aligned with `consensus`. `lift_delta` is
/// `full_pipeline.pass_rate - single_agent_wait.pass_rate` from the live run.
///
/// The returned `IsoBudgetReport` has `runpod_pending=false` and all H-40
/// fields populated. Callers are responsible for setting `runpod_pending=false`
/// on the base pipeline rows before calling this function — the `..base`
/// spread in `fill_decorrelation` copies `runpod_pending` from `base`.
pub async fn run_ablation_with_outcomes(
    ssb: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
    task_fixture: &str,
    budget_tokens: u32,
    role_outcomes: &BTreeMap<AgentRole, PerRoleOutcomes>,
    consensus: &[bool],
    lift_delta: f64,
) -> Result<IsoBudgetReport, IsoBudgetError> {
    // Build the stub base report + emit the scaffold SSB row.
    let stub = run_ablation(ssb, agent_id, superstep, task_fixture, budget_tokens).await?;
    // Mark the live run as not pending before filling decorrelation so the
    // `runpod_pending` flag is correct on the enriched report.
    let live_base = IsoBudgetReport { runpod_pending: false, ..stub };
    // Populate H-40 metrics from the live outcome data.
    let enriched = fill_decorrelation(live_base, role_outcomes, consensus, lift_delta);
    // Re-emit the enriched SSB payload so the dashboard gets the populated fields.
    let ucr_map = enriched.unique_catch_rate.clone();
    let ds_map = enriched.decorrelation_score.clone();
    let ic_map = enriched.insufficient_contribution.clone();
    let payload = SsbPayload::IsoBudgetMeasured(IsoBudgetMeasuredPayload {
        task_fixture: enriched.task_fixture.clone(),
        budget_tokens: enriched.budget_tokens,
        full_pipeline_pass_rate: enriched.full_pipeline.pass_rate,
        single_agent_wait_pass_rate: enriched.single_agent_wait.pass_rate,
        insufficient_lift: enriched.insufficient_lift,
        runpod_pending: enriched.runpod_pending,
        mast_mode_counts: aggregate_mast_counts(&enriched),
        unique_catch_rate: ucr_map,
        decorrelation_score: ds_map,
        insufficient_contribution: ic_map,
    });
    ssb.write_payload(agent_id, superstep + 1, payload, Vec::new())
        .await?;
    Ok(enriched)
}

/// Compute decorrelation metrics from live per-role outcome data and
/// return an extended `IsoBudgetReport` with the H-40 fields populated.
///
/// This is the live path called when RUNPOD-PENDING is resolved and real
/// per-role per-item outcome vectors are available. The `run_ablation`
/// stub path does NOT call this; it returns the conservative stub values.
///
/// `role_outcomes` maps each agent role to its per-item pass/fail vector
/// aligned with a shared ground-truth label vector. `consensus` is the
/// element-wise AND of all roles (item passes only if all roles pass it).
///
/// `base` must be a stub-mode `IsoBudgetReport` (runpod_pending=true) or
/// a live report whose decorrelation fields have not yet been filled.
/// Returns a new report with unique_catch_rate, decorrelation_score, and
/// insufficient_contribution populated; `runpod_pending` is left unchanged
/// (live callers set it to false on their own report).
#[must_use]
pub fn fill_decorrelation(
    base: IsoBudgetReport,
    role_outcomes: &BTreeMap<AgentRole, PerRoleOutcomes>,
    consensus: &[bool],
    lift_delta: f64,
) -> IsoBudgetReport {
    let ucr: BTreeMap<String, f64> = role_outcomes
        .keys()
        .map(|role| {
            let rate = role_outcomes
                .get(role)
                .map(|ro| unique_catch_rate(ro, consensus))
                .unwrap_or(0.0);
            (agent_role_wire_tag(role), rate)
        })
        .collect();

    let ds: BTreeMap<String, f64> = role_outcomes
        .keys()
        .map(|role| {
            let score = role_outcomes
                .get(role)
                .map(|ro| decorrelation_score(ro, role_outcomes, role))
                .unwrap_or(1.0);
            (agent_role_wire_tag(role), score)
        })
        .collect();

    // H-40 prunable predicate per role: low_lift AND low_unique_catch_for_THAT_role.
    // A role with low lift but high unique-catch is RETAINED (RD-2).
    // A role with low lift AND low unique-catch is prunable.
    let low_lift = lift_delta < LOW_LIFT_FLOOR;
    let insufficient_contribution: BTreeMap<String, bool> = ucr
        .iter()
        .map(|(tag, &rate)| (tag.clone(), low_lift && rate < LOW_UNIQUE_CATCH_FLOOR))
        .collect();

    IsoBudgetReport {
        unique_catch_rate: ucr,
        decorrelation_score: ds,
        insufficient_contribution,
        ..base
    }
}

/// Convert an `AgentRole` to its serde snake_case wire tag string.
/// Used to key the `unique_catch_rate` / `decorrelation_score` maps in
/// `IsoBudgetReport` (which must mirror the `BTreeMap<String, f64>` in the
/// SSB payload `IsoBudgetMeasuredPayload`).
///
/// AgentRole is `#[serde(rename_all = "snake_case")]`, so the canonical
/// conversion is via serde_json. The result is always a valid UTF-8 string.
fn agent_role_wire_tag(role: &AgentRole) -> String {
    serde_json::to_value(role)
        .ok()
        .and_then(|v| v.as_str().map(ToOwned::to_owned))
        .unwrap_or_else(|| format!("{role:?}").to_lowercase())
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
        // H-40: stub path emits empty maps; insufficient_contribution is empty map
        // (unknown per-role state; caller must check runpod_pending before acting).
        assert!(report.unique_catch_rate.is_empty(), "stub: no live outcomes");
        assert!(report.decorrelation_score.is_empty(), "stub: no live outcomes");
        assert!(report.insufficient_contribution.is_empty(), "stub: empty per-role map");
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
            unique_catch_rate: BTreeMap::new(),
            decorrelation_score: BTreeMap::new(),
            insufficient_contribution: BTreeMap::new(),
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
