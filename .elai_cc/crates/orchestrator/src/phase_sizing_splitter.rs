//! Batch FUL-B — PhaseSizing-aware SPLIT_TASK splitter.
//!
//! Provides a trait object that the [`crate::autonomous_gate::AutonomousGate`]
//! plumbing in `task_runner` consults when `AutonomyDecision::SplitTask`
//! fires. The trait returns real [`SubTaskSpec`] descriptors (id + summary +
//! preserved contracts + depth) rather than the deterministic `{step_id}.subN`
//! fallback introduced in Batch 3B.
//!
//! The default implementation, [`DefaultPhaseSizingSplitter`], is a
//! self-contained heuristic splitter that mirrors the Phase Sizing (Stage
//! 3.5) semantics recorded in the pipeline builder:
//!
//! * `SPLIT_RATIO = 0.6` — split when `estimate_complexity > budget * 0.6`
//!   (see [`crate::pipeline_builder::SPLIT_RATIO`]).
//! * `MAX_SPLIT_DEPTH = 2` — matches both
//!   [`crate::pipeline_builder::MAX_SPLIT_DEPTH`] and
//!   [`crate::plan_skeleton::PlanSkeletonError::SplitDepthExceeded`].
//! * `CONTRACT_OVERHEAD = 200` — per-contract token overhead used by the
//!   pipeline builder (see [`crate::pipeline_builder::CONTRACT_OVERHEAD`]).
//!
//! Contract-boundary split strategy: the splitter lays sub-tasks along the
//! input/output contract boundary, so the first sub-task inherits the
//! original inputs, the last sub-task inherits the original outputs, and
//! internal contracts stitch adjacent sub-tasks together. This matches the
//! `split_phase(preserve_contracts=true)` behaviour in
//! [`crate::plan_skeleton::PlanSkeleton::split_phase`] but does not mutate
//! the skeleton — the splitter is purely a per-decision helper for the
//! autonomous gate.
//!
//! Source: docs/specs/guide/steps/phase-index-3A-4J.txt §Phase 4E Stage 3.5;
//! docs/specs/guide/steps/phase-4G.txt §SPLIT_TASK.

use serde::{Deserialize, Serialize};

use crate::pipeline_builder::{CONTRACT_OVERHEAD, MAX_SPLIT_DEPTH, SPLIT_RATIO};
use crate::plan_skeleton::ContractRef;

// ---------------------------------------------------------------------------
// Default tuning — derived from pipeline_builder + 4G spec.
// ---------------------------------------------------------------------------

/// Maximum sub-tasks emitted per split. Spec §4G SPLIT_TASK: "max 4
/// sub-tasks". Exposed so the dashboard can display the cap.
pub const DEFAULT_MAX_SUB_TASKS: u32 = 4;

/// Minimum sub-tasks for a single split. A split producing fewer than two
/// sub-tasks is indistinguishable from a no-op.
pub const MIN_SUB_TASKS: u32 = 2;

/// Fallback token budget used when the caller does not supply one. Matches
/// the `TOKEN_HARD_CAP` in `plan_skeleton.rs` so both pathways agree.
pub const DEFAULT_TOKEN_BUDGET: u32 = 4096;

// ---------------------------------------------------------------------------
// SplitTarget — input wrapper.
// ---------------------------------------------------------------------------

/// Everything the splitter needs to reason about one candidate task. Kept
/// serde-clean so tests and higher layers can round-trip without dragging
/// the full `BabyStepSpec` graph in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SplitTarget {
    /// Canonical identifier of the task under evaluation (baby-step id in
    /// the task_runner emission site).
    pub step_id: String,
    /// One-liner summary of the task (typically `BabyStepSpec.task`).
    pub summary: String,
    /// Caller-supplied estimate of the task's token cost. When the caller
    /// has a structured estimate (e.g. `BabyStepSpec.estimated_tokens`),
    /// pass it here; otherwise pass `0` and the heuristic will fall back to
    /// `summary.len() / 4` as a coarse proxy.
    pub estimated_tokens: u32,
    /// Input contracts for the task — serve as split boundaries.
    pub input_contracts: Vec<ContractRef>,
    /// Output contracts for the task — serve as split boundaries.
    pub output_contracts: Vec<ContractRef>,
    /// Current split depth: 0 = not yet split, 1 = split once, etc. Splits
    /// emitted from a depth-N target carry depth N+1.
    pub current_depth: u8,
}

// ---------------------------------------------------------------------------
// ComplexityEstimate — output of estimate_complexity.
// ---------------------------------------------------------------------------

/// Diagnostic output of [`TaskSplitter::estimate_complexity`]. Surfaces
/// both the numeric estimate and the boolean verdict so callers can emit
/// richer SSB rows (see [`crate::ssb::TaskSplitPayload::complexity_estimate`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComplexityEstimate {
    pub token_estimate: u32,
    pub should_split: bool,
}

// ---------------------------------------------------------------------------
// SubTaskSpec — output of split.
// ---------------------------------------------------------------------------

/// A single sub-task descriptor emitted by [`TaskSplitter::split`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubTaskSpec {
    /// Canonical sub-task identifier (deterministic format:
    /// `{parent_step_id}.sub{index}`).
    pub sub_task_id: String,
    /// Human-readable one-liner summarising the sub-task scope. Rendered
    /// verbatim in the dashboard `ActivityEntry` panel.
    pub summary: String,
    /// Subset of the parent's input contracts owned by this sub-task.
    pub input_contracts: Vec<ContractRef>,
    /// Subset of the parent's output contracts owned by this sub-task.
    pub output_contracts: Vec<ContractRef>,
    /// Depth of this sub-task (parent depth + 1). Capped at
    /// [`MAX_SPLIT_DEPTH`].
    pub depth: u8,
}

// ---------------------------------------------------------------------------
// TaskSplitter — trait object surface.
// ---------------------------------------------------------------------------

/// Pluggable splitter used by the autonomous gate's SPLIT_TASK path.
///
/// Kept deliberately minimal: two pure methods, no `&mut self`, no async.
/// Implementations must be `Send + Sync` so the task_runner can hold a
/// cloneable `Arc<dyn TaskSplitter>` across await points.
pub trait TaskSplitter: Send + Sync {
    /// Estimate the token cost of `target` and decide whether a split is
    /// warranted against the caller's current budget. Cheap; safe to call
    /// on every baby-step if the caller wants a running estimate.
    fn estimate_complexity(&self, target: &SplitTarget) -> ComplexityEstimate;

    /// Produce the concrete sub-tasks. Returns an empty vec when
    /// [`ComplexityEstimate::should_split`] is false OR when `target` is
    /// already at the maximum depth.
    fn split(&self, target: &SplitTarget, budget: u32) -> Vec<SubTaskSpec>;
}

// ---------------------------------------------------------------------------
// DefaultPhaseSizingSplitter — self-contained heuristic implementation.
// ---------------------------------------------------------------------------

/// Deterministic PhaseSizing-style splitter.
///
/// Default tuning pulls from the pipeline builder constants so every
/// moving part of the splitter stays traceable to a spec line:
/// * `max_sub_tasks = DEFAULT_MAX_SUB_TASKS (= 4)`.
/// * `max_depth = MAX_SPLIT_DEPTH (= 2)`.
/// * `budget_ratio = SPLIT_RATIO (= 0.6)`.
#[derive(Debug, Clone, Copy)]
pub struct DefaultPhaseSizingSplitter {
    pub max_sub_tasks: u32,
    pub max_depth: u8,
    pub budget_ratio: f32,
}

impl Default for DefaultPhaseSizingSplitter {
    fn default() -> Self {
        Self {
            max_sub_tasks: DEFAULT_MAX_SUB_TASKS,
            // NARROWING-OK: MAX_SPLIT_DEPTH = 2 (u32), const value always fits u8.
            max_depth: MAX_SPLIT_DEPTH as u8,
            budget_ratio: SPLIT_RATIO as f32,
        }
    }
}

impl DefaultPhaseSizingSplitter {
    /// Coarse structural estimate when the caller did not supply
    /// `estimated_tokens`. The summary string length / 4 approximates the
    /// standard "4 chars per token" rule-of-thumb; we then fold in a
    /// per-contract overhead from the pipeline builder's CONTRACT_OVERHEAD
    /// constant.
    fn estimate_tokens(target: &SplitTarget) -> u32 {
        let base = if target.estimated_tokens > 0 {
            target.estimated_tokens
        } else {
            (u32::try_from(target.summary.len()).unwrap_or(u32::MAX)) / 4
        };
        let contract_count = u32::try_from(
            target.input_contracts.len() + target.output_contracts.len(),
        )
        .unwrap_or(u32::MAX);
        // NARROWING-OK: CONTRACT_OVERHEAD = 200 (u64), const value always fits u32.
        base.saturating_add(contract_count.saturating_mul(CONTRACT_OVERHEAD as u32))
    }

    /// Decide how many sub-tasks to produce given the number of contract
    /// boundaries available. Caps at `max_sub_tasks` and floors at
    /// `MIN_SUB_TASKS` when a split is warranted.
    fn decide_sub_count(&self, target: &SplitTarget) -> u32 {
        // Each output contract is a natural boundary; add one so N
        // boundaries yield N sub-tasks. Floor at MIN_SUB_TASKS and cap at
        // max_sub_tasks.
        let boundaries = u32::try_from(target.output_contracts.len().max(1)).unwrap_or(u32::MAX);
        boundaries.clamp(MIN_SUB_TASKS, self.max_sub_tasks)
    }
}

impl TaskSplitter for DefaultPhaseSizingSplitter {
    fn estimate_complexity(&self, target: &SplitTarget) -> ComplexityEstimate {
        let token_estimate = Self::estimate_tokens(target);
        // Default budget used when estimate_complexity is called without a
        // paired split call. The split call below re-evaluates against the
        // caller-supplied budget and may flip the verdict.
        // NARROWING-OK: f32->u32; no TryFrom<f32> in std; result is clamped by
        // DEFAULT_TOKEN_BUDGET which is well within u32::MAX.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let threshold =
            ((DEFAULT_TOKEN_BUDGET as f32) * self.budget_ratio) as u32;
        let should_split = token_estimate > threshold
            && target.current_depth < self.max_depth;
        ComplexityEstimate {
            token_estimate,
            should_split,
        }
    }

    fn split(&self, target: &SplitTarget, budget: u32) -> Vec<SubTaskSpec> {
        // Gate 1: already at (or past) the depth cap. 4G spec: max depth 2.
        if target.current_depth >= self.max_depth {
            return Vec::new();
        }

        // Gate 2: complexity under the ratio threshold against the
        // caller-supplied budget. Cheap early-exit keeps the happy path
        // (small tasks) allocation-free.
        let token_estimate = Self::estimate_tokens(target);
        // NARROWING-OK: f32->u32; no TryFrom<f32> in std; budget is u32 so (budget as f32)*ratio
        // stays within u32::MAX range for any ratio in [0.0, 1.0].
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let threshold = ((budget as f32) * self.budget_ratio) as u32;
        if token_estimate <= threshold {
            return Vec::new();
        }

        let sub_count = self.decide_sub_count(target);
        let new_depth = target.current_depth.saturating_add(1);
        let mut subs: Vec<SubTaskSpec> = Vec::with_capacity(sub_count as usize);

        // Distribute output contracts round-robin across sub-tasks so the
        // last sub-task still ends up holding the final output boundary.
        // Distribute input contracts likewise across the first sub-task(s).
        for i in 0..sub_count {
            let sub_task_id = format!("{}.sub{}", target.step_id, i);
            let summary = format!(
                "{} (part {}/{})",
                short_summary(&target.summary),
                i + 1,
                sub_count
            );
            // First sub-task inherits ALL original inputs; others inherit
            // only the stitching contract from the prior sub-task.
            let input_contracts = if i == 0 {
                target.input_contracts.clone()
            } else {
                vec![stitch_contract(&target.step_id, i - 1)]
            };
            // Last sub-task inherits ALL original outputs; earlier
            // sub-tasks carry a stitching output contract handed to the
            // next sub-task's inputs.
            let output_contracts = if i + 1 == sub_count {
                target.output_contracts.clone()
            } else {
                vec![stitch_contract(&target.step_id, i)]
            };
            subs.push(SubTaskSpec {
                sub_task_id,
                summary,
                input_contracts,
                output_contracts,
                depth: new_depth,
            });
        }
        subs
    }
}

// ---------------------------------------------------------------------------
// Free functions — summary truncation + stitching-contract factory.
// ---------------------------------------------------------------------------

/// Truncate a summary down to a single-line 80-char preview so the
/// dashboard does not render an entire task body per sub-task.
fn short_summary(raw: &str) -> String {
    const MAX: usize = 80;
    let cleaned = raw.trim().replace('\n', " ");
    if cleaned.chars().count() <= MAX {
        return cleaned;
    }
    let cut: String = cleaned.chars().take(MAX - 1).collect();
    format!("{cut}…")
}

/// Build the internal stitching contract that bridges sub-task `i` to
/// sub-task `i+1`. Mirrors the `split_output` naming used by
/// [`crate::plan_skeleton::PlanSkeleton::split_phase`].
fn stitch_contract(parent_step_id: &str, from_sub: u32) -> ContractRef {
    ContractRef {
        name: format!("{parent_step_id}.sub{from_sub}::split_output"),
        type_signature: "fn() -> SplitData".to_owned(),
        invariants: vec![],
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn cref(name: &str) -> ContractRef {
        ContractRef {
            name: name.to_owned(),
            type_signature: "fn() -> ()".to_owned(),
            invariants: vec![],
        }
    }

    #[test]
    fn default_splitter_small_target_does_not_split() {
        let splitter = DefaultPhaseSizingSplitter::default();
        let target = SplitTarget {
            step_id: "s".into(),
            summary: "small".into(),
            estimated_tokens: 50,
            input_contracts: vec![cref("i")],
            output_contracts: vec![cref("o")],
            current_depth: 0,
        };
        let est = splitter.estimate_complexity(&target);
        assert!(!est.should_split);
        assert!(splitter.split(&target, DEFAULT_TOKEN_BUDGET).is_empty());
    }

    #[test]
    fn default_splitter_large_target_splits_up_to_four() {
        let splitter = DefaultPhaseSizingSplitter::default();
        let target = SplitTarget {
            step_id: "big".into(),
            summary: "large step".into(),
            estimated_tokens: 8_000,
            input_contracts: (0..5)
                .map(|i| cref(&format!("i{i}")))
                .collect(),
            output_contracts: (0..5)
                .map(|i| cref(&format!("o{i}")))
                .collect(),
            current_depth: 0,
        };
        let subs = splitter.split(&target, 4096);
        assert!(!subs.is_empty());
        assert!(subs.len() as u32 <= 4);
    }

    #[test]
    fn default_splitter_depth_two_returns_empty() {
        let splitter = DefaultPhaseSizingSplitter::default();
        let target = SplitTarget {
            step_id: "deep".into(),
            summary: "deep step body ".repeat(200),
            estimated_tokens: 20_000,
            input_contracts: vec![cref("i")],
            output_contracts: vec![cref("o")],
            current_depth: 2,
        };
        assert!(splitter.split(&target, 4096).is_empty());
    }

    #[test]
    fn short_summary_truncates_and_preserves_single_line() {
        let raw = "a".repeat(500);
        let s = short_summary(&raw);
        assert!(s.chars().count() <= 80);
        let multi = "line1\nline2";
        let s2 = short_summary(multi);
        assert!(!s2.contains('\n'));
    }

    #[test]
    fn stitch_contract_name_matches_plan_skeleton_style() {
        let c = stitch_contract("step-1", 2);
        assert_eq!(c.name, "step-1.sub2::split_output");
        assert_eq!(c.type_signature, "fn() -> SplitData");
    }
}
