//! Batch 4C (A-46) — ContextTelescope caller + aggregate stats.
//!
//! Wires [`elai_core::telescope::ContextTelescope`] into the repair-context
//! assembly path: when a baby step triggers the `RepairRetry`
//! [`crate::context_budget::GenerationProfile`], the telescope collapses
//! from the outermost layer inward until the token budget is satisfied.
//! Each collapse is reported via a `TelescopeCollapse` SSB row and an
//! atomic counter so the dashboard can render aggregate reduction stats
//! without rescanning the SSB.

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;

use elai_core::bounded_string::{BoundedString, TokenCounter, WordTokenCounter};
use elai_core::telescope::{
    ContextTelescope, LABEL_OVERHEAD_TOKENS, TELESCOPE_CONCEPT_MAX, TELESCOPE_PHASE_MAX,
    TELESCOPE_STEP_MAX, TELESCOPE_VISION_MAX,
};

use crate::ssb::{Ssb, SsbError, SsbPayload, TelescopeCollapsePayload};

/// Aggregate statistics for telescope invocations.
///
/// Lock-free via atomics so call sites do not have to thread a mutex.
/// Stored as `Arc<TelescopeStats>` on the task_runner config so tests and
/// the dashboard observe the same counter.
#[derive(Debug, Default)]
pub struct TelescopeStats {
    invocations: AtomicU64,
    from_tokens_sum: AtomicU64,
    to_tokens_sum: AtomicU64,
    max_layers_collapsed: AtomicU32,
}

impl TelescopeStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn invocations(&self) -> u64 {
        self.invocations.load(Ordering::Relaxed)
    }

    pub fn avg_reduction_pct(&self) -> f32 {
        let inv = self.invocations.load(Ordering::Relaxed);
        if inv == 0 {
            return 0.0;
        }
        let from = self.from_tokens_sum.load(Ordering::Relaxed) as f32;
        if from == 0.0 {
            return 0.0;
        }
        let to = self.to_tokens_sum.load(Ordering::Relaxed) as f32;
        ((from - to) / from) * 100.0
    }

    pub fn max_layers(&self) -> u32 {
        self.max_layers_collapsed.load(Ordering::Relaxed)
    }

    fn record(&self, from: u32, to: u32, layers: u32) {
        self.invocations.fetch_add(1, Ordering::Relaxed);
        self.from_tokens_sum
            .fetch_add(u64::from(from), Ordering::Relaxed);
        self.to_tokens_sum
            .fetch_add(u64::from(to), Ordering::Relaxed);
        loop {
            let prev = self.max_layers_collapsed.load(Ordering::Relaxed);
            if layers <= prev {
                break;
            }
            if self
                .max_layers_collapsed
                .compare_exchange(prev, layers, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }
    }
}

/// Result of a single telescope invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TelescopeResult {
    pub from_tokens: u32,
    pub to_tokens: u32,
    pub layers_collapsed: u32,
}

/// Count tokens across all present layers of a ContextTelescope.
fn count_tokens(t: &ContextTelescope, counter: &dyn TokenCounter) -> u32 {
    let mut total: u32 = 0;
    total = total.saturating_add(count_bounded(&t.project_vision, counter));
    if let Some(c) = t.concept_summary.as_ref() {
        total = total.saturating_add(count_bounded(c, counter));
    }
    if let Some(p) = t.phase_goal.as_ref() {
        total = total.saturating_add(count_bounded(p, counter));
    }
    if let Some(s) = t.step_context.as_ref() {
        total = total.saturating_add(count_bounded(s, counter));
    }
    total
}

fn count_bounded<const N: usize>(b: &BoundedString<N>, counter: &dyn TokenCounter) -> u32 {
    let t = counter.count(b.as_ref());
    // Each layer incurs a fixed label overhead in the assembled prompt.
    t.saturating_add(LABEL_OVERHEAD_TOKENS)
        .try_into()
        .unwrap_or(u32::MAX)
}

/// Rebuild a telescope by-value — `ContextTelescope` does not impl Clone so
/// we reconstruct its layers through the `BoundedString` constructor, which
/// is infallible on strings that were already bounded.
fn clone_telescope(t: &ContextTelescope) -> ContextTelescope {
    let c = WordTokenCounter;
    ContextTelescope {
        project_vision: BoundedString::<TELESCOPE_VISION_MAX>::new(
            t.project_vision.as_ref(),
            &c,
        )
        .unwrap_or_else(|_| {
            // Unreachable: the source is already bounded at N. Keep runtime
            // non-panicking by truncating to N words.
            let truncated: String = t
                .project_vision
                .as_ref()
                .split_whitespace()
                .take(TELESCOPE_VISION_MAX)
                .collect::<Vec<_>>()
                .join(" ");
            BoundedString::<TELESCOPE_VISION_MAX>::new(&truncated, &c).unwrap_or_else(|_| {
                BoundedString::<TELESCOPE_VISION_MAX>::new("", &c)
                    .expect("empty string fits any BoundedString")
            })
        }),
        concept_summary: t
            .concept_summary
            .as_ref()
            .and_then(|s| BoundedString::<TELESCOPE_CONCEPT_MAX>::new(s.as_ref(), &c).ok()),
        phase_goal: t
            .phase_goal
            .as_ref()
            .and_then(|s| BoundedString::<TELESCOPE_PHASE_MAX>::new(s.as_ref(), &c).ok()),
        step_context: t
            .step_context
            .as_ref()
            .and_then(|s| BoundedString::<TELESCOPE_STEP_MAX>::new(s.as_ref(), &c).ok()),
    }
}

/// Collapse a [`ContextTelescope`] until `count_tokens(...) <= budget`.
///
/// Collapse order: step_context -> phase_goal -> concept_summary, never
/// project_vision (the outermost frame stays). Returns the number of
/// layers dropped. A fresh telescope is rebuilt so the caller's stays
/// untouched.
fn collapse_to_budget(t: &ContextTelescope, budget: u32) -> (ContextTelescope, u32) {
    let counter = WordTokenCounter;
    let mut out = clone_telescope(t);
    let mut dropped: u32 = 0;
    if count_tokens(&out, &counter) <= budget {
        return (out, 0);
    }
    if out.step_context.take().is_some() {
        dropped += 1;
    }
    if count_tokens(&out, &counter) <= budget {
        return (out, dropped);
    }
    if out.phase_goal.take().is_some() {
        dropped += 1;
    }
    if count_tokens(&out, &counter) <= budget {
        return (out, dropped);
    }
    if out.concept_summary.take().is_some() {
        dropped += 1;
    }
    (out, dropped)
}

/// Invoke the telescope for a repair context. Records aggregate stats and
/// emits a [`crate::ssb::TelescopeCollapsePayload`] row only when at least
/// one layer was dropped.
pub async fn invoke_telescope(
    writer: &Arc<Ssb>,
    stats: &Arc<TelescopeStats>,
    agent_id: &str,
    superstep: i64,
    step_id: &str,
    telescope: &ContextTelescope,
    budget_tokens: u32,
) -> Result<TelescopeResult, SsbError> {
    let counter = WordTokenCounter;
    let before = count_tokens(telescope, &counter);
    let (collapsed, layers) = collapse_to_budget(telescope, budget_tokens);
    let after = count_tokens(&collapsed, &counter);
    let result = TelescopeResult {
        from_tokens: before,
        to_tokens: after,
        layers_collapsed: layers,
    };
    stats.record(result.from_tokens, result.to_tokens, result.layers_collapsed);
    if layers > 0 {
        let payload = SsbPayload::TelescopeCollapse(TelescopeCollapsePayload {
            from_tokens: result.from_tokens,
            to_tokens: result.to_tokens,
            layers_collapsed: result.layers_collapsed,
            step_id: step_id.to_string(),
        });
        writer
            .write_payload(agent_id, superstep, payload, Vec::new())
            .await?;
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use elai_core::bounded_string::WordTokenCounter;

    fn sample_full_telescope() -> ContextTelescope {
        let c = WordTokenCounter;
        ContextTelescope {
            project_vision: BoundedString::new("alpha beta gamma delta epsilon zeta", &c).unwrap(),
            concept_summary: Some(BoundedString::new("one two three four", &c).unwrap()),
            phase_goal: Some(BoundedString::new("five six seven eight", &c).unwrap()),
            step_context: Some(BoundedString::new("nine ten eleven twelve", &c).unwrap()),
        }
    }

    #[test]
    fn collapse_leaves_within_budget_and_reports_layers() {
        let t = sample_full_telescope();
        // Huge budget: no collapse.
        let (_, layers) = collapse_to_budget(&t, 10_000);
        assert_eq!(layers, 0);
        // Tiny budget: must collapse at least two inner layers.
        let (out, layers) = collapse_to_budget(&t, 9);
        assert!(layers >= 2);
        assert!(out.step_context.is_none());
    }

    #[test]
    fn stats_record_aggregates_correctly() {
        let s = TelescopeStats::new();
        s.record(200, 100, 1);
        s.record(400, 100, 3);
        assert_eq!(s.invocations(), 2);
        assert_eq!(s.max_layers(), 3);
        // from_sum=600, to_sum=200, avg_reduction = (600 - 200) / 600 * 100
        // = 66.666..%
        let avg = s.avg_reduction_pct();
        assert!(
            (avg - 66.666_67).abs() < 0.01,
            "avg_reduction_pct was {avg}, expected ~66.666%"
        );
    }
}
