//! E-44 (Tranche 7C) — rolling-window agent reliability metrics.
//!
//! Producer-side tracker that accumulates per-tool outcomes inside a
//! bounded time window and rolls up four scalar metrics on each
//! `compute_metrics` call:
//!
//!   * `success_rate` — successes / total in the window (RDC-style
//!     Reliability Decay Curve scalar; lower bound on agent trust).
//!   * `p50_latency_ms` — median of the per-outcome latencies.
//!   * `p95_latency_ms` — 95th-percentile latency (tail signal).
//!   * `sample_window` — count of outcomes considered.
//!
//! Each rollup emits one [`DurabilityEventKind::AgentReliabilityMetrics`]
//! row (reserved at SCHEMA_VERSION 186; no further bump needed).
//!
//! Determinism: `VecDeque` for the rolling buffer, `BTreeMap` is not
//! required because the buffer is intrinsically ordered. Percentile
//! computation sorts a clone of the latencies; no HashMap, no
//! `unwrap()` / `expect()` in production paths.

use std::collections::VecDeque;
use std::time::Instant;

use elai_core::clock::{ClockProvider, OsClock};

use crate::durability::observability::notify_agent_reliability_metrics;
use crate::ssb::{Ssb, SsbError};

/// One observation of an agent / tool dispatch outcome.
#[derive(Debug, Clone)]
pub struct AgentReliability {
    pub agent_id: String,
    pub success: bool,
    pub latency_ms: u64,
    /// Optional bucket label from [`crate::durability::error_taxonomy`]
    /// for failure outcomes. `None` on success.
    pub last_failure_class: Option<String>,
}

/// Rolling-window buffer keyed by wall-clock `Instant`. Outcomes older
/// than `window_ms` are evicted on every push.
#[derive(Debug)]
pub struct ReliabilityWindow {
    rolling: VecDeque<(Instant, AgentReliability)>,
    window_ms: u64,
}

impl ReliabilityWindow {
    /// Construct a window of the given length in milliseconds.
    #[must_use]
    pub fn new(window_ms: u64) -> Self {
        Self {
            rolling: VecDeque::new(),
            window_ms,
        }
    }

    /// Append an observation. Evicts any observations older than the
    /// window threshold.
    pub fn push(&mut self, outcome: AgentReliability) {
        let now = OsClock.instant_now();
        self.rolling.push_back((now, outcome));
        self.evict_expired(now);
    }

    fn evict_expired(&mut self, now: Instant) {
        let window = std::time::Duration::from_millis(self.window_ms);
        while let Some((ts, _)) = self.rolling.front() {
            if now.duration_since(*ts) > window {
                self.rolling.pop_front();
            } else {
                break;
            }
        }
    }

    /// Number of outcomes currently in the window.
    #[must_use]
    pub fn len(&self) -> usize {
        self.rolling.len()
    }

    /// True iff the window holds no outcomes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rolling.is_empty()
    }

    /// Iterate over current observations (used by metric computation).
    pub fn iter(&self) -> impl Iterator<Item = &AgentReliability> {
        self.rolling.iter().map(|(_, o)| o)
    }
}

/// Rolled-up scalars produced by [`compute_metrics`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReliabilityMetrics {
    pub success_rate: f32,
    pub p50_latency_ms: u64,
    pub p95_latency_ms: u64,
    pub sample_window: u32,
}

/// Compute the rollup scalars over the current window. Empty windows
/// produce a zero-sample report (success_rate = 0.0; latencies = 0).
#[must_use]
pub fn compute_metrics(window: &ReliabilityWindow) -> ReliabilityMetrics {
    let total = window.len();
    if total == 0 {
        return ReliabilityMetrics {
            success_rate: 0.0,
            p50_latency_ms: 0,
            p95_latency_ms: 0,
            sample_window: 0,
        };
    }

    let mut successes = 0_usize;
    let mut latencies: Vec<u64> = Vec::with_capacity(total);
    for o in window.iter() {
        if o.success {
            successes += 1;
        }
        latencies.push(o.latency_ms);
    }
    latencies.sort_unstable();

    // NARROWING-OK: total is bounded by the rolling-window capacity;
    // f32 has enough precision for ratios in [0, 1].
    let success_rate = successes as f32 / total as f32;

    // Percentile helper. p in [0.0, 1.0]; uses the nearest-rank method.
    let pick = |p: f32| -> u64 {
        if latencies.is_empty() {
            return 0;
        }
        let idx_f = (p * (latencies.len() as f32 - 1.0)).round();
        // NARROWING-OK: idx_f is guarded for finiteness + non-negativity
        // before the cast. f32 -> usize has no TryFrom impl in std, so the
        // canonical idiom is a guard-then-saturating-cast (Rust 1.45+
        // saturating semantics). The .min(latencies.len()-1) below provides
        // an additional upper-bound guard.
        let idx = if idx_f.is_finite() && idx_f >= 0.0 {
            u64::try_from(idx_f as i64).unwrap_or(0) as usize
        } else {
            0_usize
        };
        let clamped = idx.min(latencies.len() - 1);
        latencies[clamped]
    };

    let sample_window = u32::try_from(total).unwrap_or(u32::MAX);
    ReliabilityMetrics {
        success_rate,
        p50_latency_ms: pick(0.50),
        p95_latency_ms: pick(0.95),
        sample_window,
    }
}

/// Emit one [`DurabilityEventKind::AgentReliabilityMetrics`] row.
/// Wire-dark when the env gate is off (observability layer short-circuits).
pub async fn emit_rollup(
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    session_id: Option<&str>,
    phase_id: &str,
    metrics: ReliabilityMetrics,
    ts_ms: i64,
) -> Result<Option<i64>, SsbError> {
    notify_agent_reliability_metrics(
        ssb,
        agent_id,
        superstep,
        session_id,
        phase_id,
        metrics.success_rate,
        metrics.p50_latency_ms,
        metrics.p95_latency_ms,
        metrics.sample_window,
        ts_ms,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn outcome(success: bool, latency_ms: u64) -> AgentReliability {
        AgentReliability {
            agent_id: "agent-A".into(),
            success,
            latency_ms,
            last_failure_class: if success { None } else { Some("transient".into()) },
        }
    }

    #[test]
    fn compute_metrics_empty_window() {
        let win = ReliabilityWindow::new(60_000);
        let m = compute_metrics(&win);
        assert_eq!(m.sample_window, 0);
        assert_eq!(m.success_rate, 0.0);
        assert_eq!(m.p50_latency_ms, 0);
        assert_eq!(m.p95_latency_ms, 0);
    }

    #[test]
    fn compute_metrics_mixed_outcomes() {
        let mut win = ReliabilityWindow::new(60_000);
        // 4 successes, 1 failure, latencies 10/20/30/40/100.
        win.push(outcome(true, 10));
        win.push(outcome(true, 20));
        win.push(outcome(true, 30));
        win.push(outcome(true, 40));
        win.push(outcome(false, 100));
        let m = compute_metrics(&win);
        assert_eq!(m.sample_window, 5);
        // 4/5 = 0.8
        assert!((m.success_rate - 0.8).abs() < 1e-6);
        // Median of sorted [10,20,30,40,100] = 30 (index 2 by nearest-rank).
        assert_eq!(m.p50_latency_ms, 30);
        // p95 of 5 samples by nearest-rank = round(0.95 * 4) = 4 -> 100
        assert_eq!(m.p95_latency_ms, 100);
    }
}
