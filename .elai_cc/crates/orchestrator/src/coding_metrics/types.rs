// E-49 — Shared metric types for the coding-agent metric suite.
//
// CodingMetric is the canonical capture-point payload aggregated from a
// running ELAI session and surfaced through the CLI / dashboard.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// E-49 — single-session coding-agent metric snapshot.
///
/// The capture is intentionally minimal and stable; downstream consumers
/// (CLI `elai metrics capture`, dashboard `MetricsDrawer`, contract
/// tests) read the same shape.
///
/// Fields:
///   * `pass_at_k`: pass@k by k (default k=1). BTreeMap so JSON keys
///     serialize in deterministic ascending order.
///   * `refine_rate`: fraction of tasks where the model successfully
///     refined a flagged failure (RefineBench-style, scope-lean).
///     Range [0.0, 1.0]; NaN/unset is represented by `f32::NAN` only
///     when collector receives no signal.
///   * `cost_per_task_cents`: optional integer cost-per-task in cents.
///     `None` when ELAI's `PerRoleRuntime` has no token-cost surface
///     for the active runtime (RUNPOD-PENDING wiring).
///   * `ts_ms`: capture wall-clock in unix milliseconds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodingMetric {
    pub session_id: String,
    pub pass_at_k: BTreeMap<u32, f32>,
    pub refine_rate: f32,
    pub cost_per_task_cents: Option<u32>,
    pub ts_ms: u64,
}

impl CodingMetric {
    /// Construct an empty metric for `session_id` at `ts_ms`. Used by the
    /// collector before any sample is folded in.
    pub fn empty(session_id: impl Into<String>, ts_ms: u64) -> Self {
        Self {
            session_id: session_id.into(),
            pass_at_k: BTreeMap::new(),
            refine_rate: 0.0,
            cost_per_task_cents: None,
            ts_ms,
        }
    }

    /// True when at least one pass@k bucket has been populated.
    pub fn has_pass_samples(&self) -> bool {
        !self.pass_at_k.is_empty()
    }

    /// Returns the pass@1 sample if recorded; `None` otherwise.
    pub fn pass_at_1(&self) -> Option<f32> {
        self.pass_at_k.get(&1).copied()
    }
}
