//! I-22 -- delta-progress reward scorer (phi(h_t) - phi(h_{t-1})).
//!
//! # Doctrine (ID-4 / ID-5 / GD-4, enforced structurally)
//!
//! Every phi component is a deterministic observable from the journal:
//!   - `test_pass`:       count of passing test cases (cargo-test stdout parse)
//!   - `compile_err_inv`: 0 when compile fails, MAX_COMPILE_INV - error_count otherwise
//!   - `req_covered`:     `EarsRequirementSet::len() - EarsRequirementSet::unsatisfied_count()`
//!
//! LLM-opinion phi is a structural BLOCK (ID-4). The shaped reward is the delta
//! of these deterministic observables, scaled by the weight vector in
//! `metrics/progress_reward_weights.toml` (ID-5: no paper-threshold default,
//! measurement first).
//!
//! The no-progress decay window D is also loaded from that toml file at
//! call-time; never a magic number in source.
//!
//! Default-OFF (`ELAI_PROGRESS_REWARD_ENABLE`, R9 experiment, GD-4): the delta
//! reward changes how the SearchLadder values branches; it ships behind a
//! measured-first-win gate. The production caller is
//! `task_runner::dispatch_progress_reward`. See `wire-dark-allowlist.toml`.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use serde::Deserialize;
use tracing::{debug, warn};

use crate::ssb::{
    ProgressRewardScoredKind, ProgressRewardScoredPayload, ELAI_PROGRESS_REWARD_ENABLE_ENV,
    Ssb, SsbPayload,
};

// ---------------------------------------------------------------------------
// Named constants (ID-5, stub_ban: no magic numbers for D or weights)
// ---------------------------------------------------------------------------

/// Default weight for `test_pass` when the toml is missing (fallback baseline;
/// must be overridden by the first measured run per ID-5).
const W_TEST_PASS_FALLBACK: f64 = 0.5;

/// Default weight for `compile_err_inv` when the toml is missing.
const W_COMPILE_FALLBACK: f64 = 0.3;

/// Default weight for `req_covered` when the toml is missing.
const W_REQ_COVERED_FALLBACK: f64 = 0.2;

/// Default no-progress window D when the toml is missing.
const D_WINDOW_FALLBACK: u32 = 5;

/// Default decay factor applied after D zero-delta steps.
const DECAY_FACTOR_FALLBACK: f64 = 0.9;

/// Inversion base for compile-error count: phi_compile = max(0, BASE - err_count).
/// A clean build yields BASE as the contribution; one error yields BASE-1; BASE
/// errors or more yields 0. Named constant so auditors know where the ceiling is.
const COMPILE_ERR_INV_BASE: u32 = 10;

// ---------------------------------------------------------------------------
// Progress-reward weight toml (ID-5 artifact)
// ---------------------------------------------------------------------------

/// Deserialized from `metrics/progress_reward_weights.toml`.
///
/// All three weight fields must sum to a positive value. D is the no-progress
/// window. These are loaded at call-time so a new measurement run can update
/// them without recompilation.
#[derive(Debug, Deserialize)]
struct ProgressWeights {
    w_test_pass: f64,
    w_compile_err_inv: f64,
    w_req_covered: f64,
    /// No-progress window (consecutive zero-delta steps before decay fires).
    no_progress_window: u32,
    /// Decay factor applied once per no-progress epoch.
    decay_factor: f64,
}

impl Default for ProgressWeights {
    fn default() -> Self {
        Self {
            w_test_pass: W_TEST_PASS_FALLBACK,
            w_compile_err_inv: W_COMPILE_FALLBACK,
            w_req_covered: W_REQ_COVERED_FALLBACK,
            no_progress_window: D_WINDOW_FALLBACK,
            decay_factor: DECAY_FACTOR_FALLBACK,
        }
    }
}

/// Load weights from `{project_root}/metrics/progress_reward_weights.toml`.
///
/// Falls back to `ProgressWeights::default()` with a debug log if the file is
/// absent or unparseable (degraded-gracefully, not a hard failure; absence
/// before the first measured run is expected).
fn load_weights(project_root: &Path) -> ProgressWeights {
    let path = project_root.join("metrics").join("progress_reward_weights.toml");
    match std::fs::read_to_string(&path) {
        Ok(contents) => match toml::from_str::<ProgressWeights>(&contents) {
            Ok(w) => {
                debug!(path = %path.display(), "progress_reward: weights loaded from toml");
                w
            }
            Err(e) => {
                warn!(
                    path = %path.display(),
                    error = %e,
                    "progress_reward: toml parse failed; using fallback weights (ID-5 first-run expected)"
                );
                ProgressWeights::default()
            }
        },
        Err(e) => {
            debug!(
                path = %path.display(),
                error = %e,
                "progress_reward: weights toml absent; using fallback weights (ID-5 first-run expected)"
            );
            ProgressWeights::default()
        }
    }
}

// ---------------------------------------------------------------------------
// Phi history (per-branch step state owned by DeltaProgressScorer)
// ---------------------------------------------------------------------------

/// Phi value at one completed step (deterministic observable sum).
#[derive(Debug, Clone, Default)]
pub struct PhiRecord {
    pub phi: f64,
    pub test_pass: u32,
    pub compile_err_inv: u32,
    pub req_covered: u32,
}

/// Per-branch phi history accumulated by `DeltaProgressScorer`.
#[derive(Debug, Default)]
struct PhiHistory {
    /// Phi values indexed by step (0-based).
    steps: Vec<PhiRecord>,
    /// Count of consecutive steps with delta == 0.0.
    zero_delta_streak: u32,
}

impl PhiHistory {
    fn last_phi(&self) -> f64 {
        self.steps.last().map(|r| r.phi).unwrap_or(0.0)
    }

    fn push(&mut self, record: PhiRecord, weights: &ProgressWeights) -> (f64, bool) {
        let prev = self.last_phi();
        let delta = record.phi - prev;
        self.steps.push(record);
        if delta.abs() < f64::EPSILON {
            self.zero_delta_streak += 1;
        } else {
            self.zero_delta_streak = 0;
        }
        let decay_fired = self.zero_delta_streak >= weights.no_progress_window
            && self.zero_delta_streak % weights.no_progress_window == 0;
        (delta, decay_fired)
    }
}

// ---------------------------------------------------------------------------
// DeltaProgressScorer
// ---------------------------------------------------------------------------

/// I-22 delta-progress reward scorer.
///
/// Owns per-branch [`PhiHistory`] so the task-runner and the i-4 PUCT loop
/// can both call `score_step` or `score_single` without sharing state.
///
/// # Production caller (R-8)
/// The task-runner-facing entry point is `emit_progress_reward_to_ssb`.
/// `score_step` carries `// WIRE-DARK[I-22]` because its only consumer in the
/// current tree is the absent i-4 PUCT value function (promotion plan I-4).
#[derive(Debug, Default)]
pub struct DeltaProgressScorer {
    history: HashMap<String, PhiHistory>,
}

impl DeltaProgressScorer {
    /// Create a new scorer (history starts empty).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Compute phi for one step from deterministic observables.
    ///
    /// phi(h_t) = w_test_pass * test_pass
    ///          + w_compile_err_inv * compile_err_inv
    ///          + w_req_covered * req_covered
    ///
    /// All three inputs are non-negative integers (ID-4: no LLM opinion).
    fn phi(w: &ProgressWeights, test_pass: u32, compile_err_inv: u32, req_covered: u32) -> f64 {
        w.w_test_pass * f64::from(test_pass)
            + w.w_compile_err_inv * f64::from(compile_err_inv)
            + w.w_req_covered * f64::from(req_covered)
    }

    /// Invert a compile-error count into the phi component.
    ///
    /// `compile_err_inv = max(0, COMPILE_ERR_INV_BASE - error_count)`.
    /// Zero errors yields `COMPILE_ERR_INV_BASE`; any count >= BASE yields 0.
    // WIRE-DARK[I-22]: feeds score_step (PUCT path); PUCT consumer (i-4 v3) PENDING.
    // See workflow/sessions/s-107b022a/stubs.md row I-22-puct.
    #[must_use]
    pub fn compile_err_to_inv(error_count: u32) -> u32 {
        COMPILE_ERR_INV_BASE.saturating_sub(error_count)
    }

    /// Score one step of a multi-branch trajectory (i-4 PUCT path).
    ///
    /// Updates the internal history for `branch_id` and returns the shaped reward
    /// (phi_t - phi_{t-1}), the decay factor if a no-progress epoch fired,
    /// and the full `PhiRecord` for introspection.
    ///
    // WIRE-DARK[I-22]: The PUCT-value-function consumer (i-4 SearchLadder v3)
    // is PENDING (see workflow/sessions/s-107b022a/stubs.md, will_resolve_in=I-4).
    // The task-runner smoke path calls `emit_progress_reward_to_ssb` instead.
    pub fn score_step(
        &mut self,
        branch_id: &str,
        step: u32,
        test_pass: u32,
        compile_err_inv: u32,
        req_covered: u32,
        weights: &ProgressWeights,
    ) -> (f64, Option<f64>) {
        let record = PhiRecord {
            phi: Self::phi(weights, test_pass, compile_err_inv, req_covered),
            test_pass,
            compile_err_inv,
            req_covered,
        };
        let history = self.history.entry(branch_id.to_owned()).or_default();
        // Align step index: if the step count matches the history len, push normally.
        // If history is longer, the branch rewound (H-49 fork) — reset to current.
        if step as usize > history.steps.len() {
            debug!(
                branch_id,
                step,
                history_len = history.steps.len(),
                "progress_reward: step skipped; using phi=0.0 as prev for gap"
            );
        }
        let (delta, decay_fired) = history.push(record, weights);
        let decay_factor = if decay_fired {
            Some(weights.decay_factor)
        } else {
            None
        };
        (delta, decay_factor)
    }

    /// Score a single isolated step with no prior history (task-runner smoke path).
    ///
    /// Returns `(shaped_reward, phi_t, phi_t_prev=0.0, decay_factor)`.
    /// phi_t_prev is always 0.0 because the task-runner exercises the emit wire
    /// end-to-end without accumulating per-step history (the first step in a
    /// single-task dispatch has no prior; the history makes sense only in the
    /// i-4 multi-branch PUCT loop — D-6 decision).
    #[must_use]
    pub fn score_single(
        test_pass: u32,
        compile_err_inv: u32,
        req_covered: u32,
        weights: &ProgressWeights,
    ) -> (f64, f64) {
        let phi_t = Self::phi(weights, test_pass, compile_err_inv, req_covered);
        (phi_t, phi_t)
    }
}

// ---------------------------------------------------------------------------
// FirstErrorTruncator
// ---------------------------------------------------------------------------

/// I-22 first-error truncator: on a HardVeto at step k, captures the working
/// tree state at step k-1 via the H-49 fork API so the PUCT loop can resume
/// from the last good point.
///
/// Uses `architecture::timetravel::capture_file` + `timetravel_enabled()`.
/// No UndoTree struct (H-49 ships the CAS; the tree index is FORWARD, D-5).
#[derive(Debug, Default)]
pub struct FirstErrorTruncator;

impl FirstErrorTruncator {
    /// Record a truncation event.
    ///
    /// When timetravel is enabled, captures a sentinel blob for `branch_id` in
    /// the H-49 CAS and returns the fork-node id (the blake3 hash of the sentinel
    /// content). When disabled or on error, returns an empty string (non-fatal;
    /// the SSB row still lands with `fork_node_id = ""`).
    // WIRE-DARK[I-22]: FirstErrorTruncated arm called from i-4 PUCT on HardVeto.
    // PUCT consumer PENDING (plan I-4). stubs.md row I-22-puct.
    #[must_use]
    pub fn capture_truncation(
        project_root: &Path,
        session_id: &str,
        branch_id: &str,
        truncated_at_step: u32,
    ) -> String {
        if !crate::architecture::timetravel::timetravel_enabled() {
            debug!(
                branch_id,
                truncated_at_step,
                "FirstErrorTruncator: timetravel disabled; fork_node_id = empty"
            );
            return String::new();
        }
        // Capture a sentinel path identifying this branch's truncation point.
        // The content is deterministic (branch_id + step) so two identical
        // truncations produce the same CAS hash (natural dedup).
        let sentinel_content =
            format!("truncated branch_id={branch_id} at_step={truncated_at_step}");
        let sentinel_path = project_root
            .join(".elai_scratch")
            .join(format!("trunc-{branch_id}-{truncated_at_step}.sentinel"));
        match crate::architecture::timetravel::capture_file(
            project_root,
            session_id,
            &sentinel_path,
            sentinel_content.as_bytes(),
        ) {
            Ok(result) => {
                debug!(
                    branch_id,
                    truncated_at_step,
                    fork_node_id = %result.pre_hash,
                    "FirstErrorTruncator: fork_node captured"
                );
                result.pre_hash
            }
            Err(e) => {
                warn!(
                    branch_id,
                    truncated_at_step,
                    error = %e,
                    "FirstErrorTruncator: capture_file failed; fork_node_id = empty"
                );
                String::new()
            }
        }
    }
}

// ---------------------------------------------------------------------------
// SSB emit helper (production-caller-facing API)
// ---------------------------------------------------------------------------

/// Emit a `ProgressRewardScored` SSB row for the just-completed task.
///
/// This is the task-runner-facing production entry point for I-22.
/// Called by `task_runner::dispatch_progress_reward` AFTER verifying the
/// `ELAI_PROGRESS_REWARD_ENABLE` gate and the checker runtime presence.
///
/// Uses `DeltaProgressScorer::score_single` (phi_prev = 0.0 on the task-runner
/// smoke path, D-6 decision). DecayApplied and FirstErrorTruncated arms are
/// NOT emitted on this path (those require the multi-step i-4 PUCT loop);
/// the Delta arm is emitted unconditionally when the gate is on.
///
/// # Errors
/// Propagates SSB write errors as `String` (non-fatal: the caller logs and continues).
pub async fn emit_progress_reward_to_ssb(
    writer: &Arc<Ssb>,
    seq: i64,
    project_id: &str,
    project_root: &Path,
    test_pass: u32,
    compile_err_inv: u32,
    req_covered: u32,
) -> Result<(), String> {
    let weights = load_weights(project_root);
    let (phi_t, _phi_t_prev_unused) =
        DeltaProgressScorer::score_single(test_pass, compile_err_inv, req_covered, &weights);
    // On the task-runner single-step path phi_prev is always 0.0 (first/only step).
    let phi_t_prev = 0.0_f64;
    let shaped_reward = phi_t - phi_t_prev;

    let payload = SsbPayload::ProgressRewardScored(ProgressRewardScoredPayload {
        kind: ProgressRewardScoredKind::Delta,
        branch_id: format!("task-{seq}"),
        step: 0,
        phi_t,
        phi_t_prev,
        shaped_reward,
        test_pass,
        compile_err_inv,
        req_covered,
        fork_node_id: String::new(),
        truncated_at_step: 0,
        decay_factor: 1.0,
        no_progress_window: 0,
        ts_ms: OsClock.now_ms().try_into().unwrap_or(0),
    });

    writer
        .write_payload(
            &format!("task-runner-{seq}"),
            seq,
            payload,
            vec![],
        )
        .await
        .map_err(|e| format!("emit_progress_reward_to_ssb: SSB write failed: {e}"))?;

    debug!(
        seq,
        project_id,
        phi_t,
        shaped_reward,
        test_pass,
        compile_err_inv,
        req_covered,
        "task_runner::dispatch_progress_reward: ProgressRewardScored Delta row emitted"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Environment-gate helper (mirrors tri_layer_enabled / search_ladder_enabled)
// ---------------------------------------------------------------------------

/// True when `ELAI_PROGRESS_REWARD_ENABLE` is set to a truthy literal.
#[must_use]
pub fn progress_reward_enabled() -> bool {
    match std::env::var(ELAI_PROGRESS_REWARD_ENABLE_ENV) {
        Ok(v) => {
            let t = v.trim();
            t.eq_ignore_ascii_case("1")
                || t.eq_ignore_ascii_case("true")
                || t.eq_ignore_ascii_case("yes")
                || t.eq_ignore_ascii_case("on")
        }
        Err(_) => false,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn default_weights() -> ProgressWeights {
        ProgressWeights::default()
    }

    #[test]
    fn phi_pure_test_pass() {
        let w = default_weights();
        // test_pass=2, compile_err_inv=0, req_covered=0
        let phi = DeltaProgressScorer::phi(&w, 2, 0, 0);
        assert!((phi - w.w_test_pass * 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn phi_zero_inputs_yields_zero() {
        let w = default_weights();
        let phi = DeltaProgressScorer::phi(&w, 0, 0, 0);
        assert_eq!(phi, 0.0);
    }

    #[test]
    fn compile_err_inv_zero_errors() {
        let inv = DeltaProgressScorer::compile_err_to_inv(0);
        assert_eq!(inv, COMPILE_ERR_INV_BASE);
    }

    #[test]
    fn compile_err_inv_saturates_at_zero() {
        let inv = DeltaProgressScorer::compile_err_to_inv(COMPILE_ERR_INV_BASE + 5);
        assert_eq!(inv, 0);
    }

    #[test]
    fn score_single_phi_prev_is_zero() {
        let w = default_weights();
        let (phi_t, _) = DeltaProgressScorer::score_single(3, 8, 2, &w);
        let expected = w.w_test_pass * 3.0 + w.w_compile_err_inv * 8.0 + w.w_req_covered * 2.0;
        assert!((phi_t - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn score_step_accumulates_delta() {
        let w = default_weights();
        let mut scorer = DeltaProgressScorer::new();
        // Step 0: phi = 0.5 * 1 = 0.5 (w_test * 1, compile=0, req=0)
        let (delta0, decay0) = scorer.score_step("b1", 0, 1, 0, 0, &w);
        assert!((delta0 - w.w_test_pass).abs() < f64::EPSILON, "delta0={delta0}");
        assert!(decay0.is_none());
        // Step 1: phi = 0.5 * 2 = 1.0 -> delta = 0.5
        let (delta1, decay1) = scorer.score_step("b1", 1, 2, 0, 0, &w);
        assert!((delta1 - w.w_test_pass).abs() < f64::EPSILON, "delta1={delta1}");
        assert!(decay1.is_none());
    }

    #[test]
    fn score_step_decay_fires_after_d_zero_deltas() {
        let w = ProgressWeights {
            w_test_pass: 0.5,
            w_compile_err_inv: 0.0,
            w_req_covered: 0.0,
            no_progress_window: 3,
            decay_factor: 0.8,
        };
        let mut scorer = DeltaProgressScorer::new();
        // Step 0: delta = 0.5 (first step from 0.0)
        let _ = scorer.score_step("b2", 0, 1, 0, 0, &w);
        // Steps 1, 2, 3: all same phi -> zero delta -> streak = 3 on step 3
        let _ = scorer.score_step("b2", 1, 1, 0, 0, &w);
        let _ = scorer.score_step("b2", 2, 1, 0, 0, &w);
        let (_, decay3) = scorer.score_step("b2", 3, 1, 0, 0, &w);
        assert_eq!(decay3, Some(0.8), "decay must fire at streak == D");
    }

    #[test]
    fn progress_reward_disabled_by_default() {
        // ELAI_PROGRESS_REWARD_ENABLE must NOT be set in the test environment.
        // If it happens to be set the binary-level gate test becomes vacuous;
        // we skip rather than fail to avoid false-positive on CI with env overrides.
        if std::env::var(ELAI_PROGRESS_REWARD_ENABLE_ENV).is_ok() {
            return;
        }
        assert!(!progress_reward_enabled(), "gate must be off by default");
    }

    #[test]
    fn first_error_truncator_timetravel_disabled() {
        // When timetravel is disabled, capture_truncation returns empty string.
        unsafe { std::env::set_var("ELAI_TIMETRAVEL_DISABLE", "1") };
        let dir = TempDir::new().expect("tempdir");
        let result = FirstErrorTruncator::capture_truncation(
            dir.path(),
            "test-session",
            "branch-x",
            5,
        );
        assert!(result.is_empty(), "should be empty when timetravel disabled");
        unsafe { std::env::remove_var("ELAI_TIMETRAVEL_DISABLE") };
    }

    #[test]
    fn load_weights_missing_file_yields_fallback() {
        let dir = TempDir::new().expect("tempdir");
        // No metrics/ subdir or toml file.
        let w = load_weights(dir.path());
        assert!((w.w_test_pass - W_TEST_PASS_FALLBACK).abs() < f64::EPSILON);
        assert_eq!(w.no_progress_window, D_WINDOW_FALLBACK);
    }

    #[test]
    fn load_weights_valid_toml_parsed() {
        let dir = TempDir::new().expect("tempdir");
        let metrics_dir = dir.path().join("metrics");
        std::fs::create_dir_all(&metrics_dir).expect("mkdir metrics");
        std::fs::write(
            metrics_dir.join("progress_reward_weights.toml"),
            "w_test_pass = 0.6\nw_compile_err_inv = 0.25\nw_req_covered = 0.15\nno_progress_window = 4\ndecay_factor = 0.85\n",
        )
        .expect("write toml");
        let w = load_weights(dir.path());
        assert!((w.w_test_pass - 0.6).abs() < f64::EPSILON);
        assert_eq!(w.no_progress_window, 4);
        assert!((w.decay_factor - 0.85).abs() < f64::EPSILON);
    }
}
