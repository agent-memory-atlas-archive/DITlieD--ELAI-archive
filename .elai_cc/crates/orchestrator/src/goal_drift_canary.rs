//! E-46 — Goal-drift canary at the decomposition-walk level.
//!
//! Companion to D-7's [`crate::canary::CanaryFamily`] cross-domain
//! transfer canary. Where D-7 measures **per-family** regression vs a
//! baseline template, E-46's [`GoalDriftCanary`] measures
//! **intra-decomposition** drift across a single walk: the canary
//! holds the initial goal's content hash + a small set of sampled
//! step indices + a drift threshold, and is queried after every
//! sampled step with the current task summary.
//!
//! The drift signal is the Jaccard **distance** (1 - similarity) over
//! normalised whitespace-tokenised word sets between the initial
//! goal and the current summary. `0.0` means identical sets; `1.0`
//! means orthogonal. Threshold defaults to `0.45` (NanoFlux band) —
//! beyond that, the canary fires and the orchestrator dispatches a
//! REPLAN_PHASE bubble-up. This module ships the measurement +
//! firing predicate; the bubble-up handler is out of E-46 scope.
//!
//! Wiring contract (HARD):
//!   1. [`GoalDriftCanary::new`] is the sole constructor; it computes
//!      the initial goal's blake3 hash at construction time so
//!      subsequent firings carry a stable correlator.
//!   2. [`measure_drift`] is a pure function (no I/O, no clock
//!      reads). Tests pin both inputs.
//!   3. [`GoalDriftCanary::fired`] returns a bool only; the SSB
//!      emission happens in `agent_pool::emit_goal_drift_fired` so
//!      the canary itself stays sync-free of the writer.
//!
//! Reuses D-7's [`crate::canary::CanaryEvaluator`] / `CanaryFamily`
//! concepts additively when callers want to compose both surfaces —
//! see the doc-comment on [`GoalDriftCanary::sampled_at_steps`] for
//! when to use which. This module does NOT introduce a new closed-enum
//! variant on D-7's surface; it lives standalone.
//!
//! Anti-stale-training: blake3 (never sha2/blake2) for the goal hash
//! per CLAUDE.md HARD RULE; tracing for logs; thiserror not required
//! (the API is total — no fallible paths).
//!
//! Grounding: arxiv:2505.02709 (Goal Drift Evaluation, AIES 2025);
//! arxiv:2412.21154 (Devin task-ledger pattern, cross-step drift is a
//! documented failure mode).
//!
//! See F:\.ELAI_workflow\docs\thefinalglance\E-plan\E-46_planning_decomposition.md.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use tracing::debug;

// ---------------------------------------------------------------------------
// Tunables
// ---------------------------------------------------------------------------

/// Default Jaccard-distance threshold above which the canary fires.
/// NanoFlux operating band (companion §3.5) places 0.4-0.5 as the
/// noise-balanced range; we default to `0.45` because the E-46 plan
/// open-question (line 209) explicitly defers the calibration probe.
pub const DEFAULT_DRIFT_THRESHOLD: f64 = 0.45;

// ---------------------------------------------------------------------------
// GoalDriftCanary
// ---------------------------------------------------------------------------

/// Goal-drift canary state. Holds the initial-goal hash + sampling
/// schedule + threshold. Constructed once at the start of a
/// decomposition walk; queried after every sampled step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalDriftCanary {
    /// 64-char lowercase hex blake3 of the canonical initial goal
    /// string. Stable across firings so the dashboard joins
    /// consecutive `goal_drift_fired` rows back to the same walk.
    pub initial_goal_blake3: String,
    /// Initial goal text retained so [`fired`] can recompute the
    /// drift on demand without re-passing the goal at every call
    /// site. Held verbatim — the canary is intra-process so the
    /// memory cost is small.
    pub initial_goal_text: String,
    /// Step indices at which the canary should be sampled. The
    /// orchestrator's decomposition walk consults
    /// [`should_sample`] before computing drift.
    pub sampled_at_steps: Vec<u32>,
    /// Drift threshold above which [`fired`] returns true. Bounded
    /// by `[0.0, 1.0]` at construction time.
    pub drift_threshold: f64,
}

impl GoalDriftCanary {
    /// Construct a fresh canary. `sampled_at_steps` is sorted +
    /// dedup'd; `drift_threshold` is clamped to `[0.0, 1.0]`. The
    /// initial goal hash is computed once here via blake3.
    #[must_use]
    pub fn new(initial_goal: &str, sampled_at_steps: Vec<u32>, drift_threshold: f64) -> Self {
        let mut steps = sampled_at_steps;
        steps.sort_unstable();
        steps.dedup();
        let mut t = drift_threshold;
        if t.is_nan() {
            t = DEFAULT_DRIFT_THRESHOLD;
        }
        if t < 0.0 {
            t = 0.0;
        }
        if t > 1.0 {
            t = 1.0;
        }
        let hash = blake3::hash(initial_goal.as_bytes()).to_hex().to_string();
        Self {
            initial_goal_blake3: hash,
            initial_goal_text: initial_goal.to_owned(),
            sampled_at_steps: steps,
            drift_threshold: t,
        }
    }

    /// True when `step` is in the sampling schedule. The schedule is
    /// sorted so we use a binary search.
    #[must_use]
    pub fn should_sample(&self, step: u32) -> bool {
        self.sampled_at_steps.binary_search(&step).is_ok()
    }

    /// Compute the drift between the canary's initial goal and the
    /// supplied `current_summary`. Pure delegation to
    /// [`measure_drift`]; exposed as a method so call sites that have
    /// the canary in scope don't need to re-pass `initial_goal_text`.
    #[must_use]
    pub fn measure_drift(&self, current_summary: &str) -> f64 {
        measure_drift(&self.initial_goal_text, current_summary)
    }

    /// True when the drift between the canary's initial goal and
    /// `current_summary` exceeds the configured threshold. The
    /// orchestrator's decomposition walk fires the SSB row when this
    /// returns `true` (see `agent_pool::emit_goal_drift_fired`).
    #[must_use]
    pub fn fired(&self, current_summary: &str) -> bool {
        let d = self.measure_drift(current_summary);
        let fired = d > self.drift_threshold;
        debug!(
            initial_goal_blake3 = %self.initial_goal_blake3,
            drift = d,
            threshold = self.drift_threshold,
            fired,
            "goal_drift_canary::fired"
        );
        fired
    }
}

// ---------------------------------------------------------------------------
// Pure measurement
// ---------------------------------------------------------------------------

/// Compute Jaccard distance between two normalised whitespace-tokenised
/// word sets. Returns a value in `[0.0, 1.0]` where `0.0` means
/// identical sets and `1.0` means orthogonal.
///
/// Normalisation:
///   - ASCII-lowercase the entire string
///   - split on Unicode whitespace
///   - strip leading/trailing ASCII punctuation per token
///   - drop empty tokens after stripping
///
/// We intentionally do NOT lemmatise or apply a stopword list — the
/// canary measures literal-token drift, not semantic drift (D-7's
/// CanaryFamily covers the semantic axis). Both empty inputs return
/// `0.0` (the canary cannot fire on an unset baseline).
#[must_use]
pub fn measure_drift(initial: &str, current_summary: &str) -> f64 {
    let a = tokenize(initial);
    let b = tokenize(current_summary);
    if a.is_empty() && b.is_empty() {
        return 0.0;
    }
    let union_count = a.union(&b).count() as f64;
    if union_count == 0.0 {
        return 0.0;
    }
    let intersection_count = a.intersection(&b).count() as f64;
    let similarity = intersection_count / union_count;
    let distance = 1.0 - similarity;
    if distance < 0.0 {
        0.0
    } else if distance > 1.0 {
        1.0
    } else {
        distance
    }
}

/// Tokenise + normalise per the rules described on [`measure_drift`].
/// Hoisted into a free function so unit tests can pin individual
/// token sets.
fn tokenize(s: &str) -> HashSet<String> {
    s.to_ascii_lowercase()
        .split_whitespace()
        .map(|t| {
            t.trim_matches(|c: char| {
                c.is_ascii_punctuation() && c != '_' && c != '-' && c != '/' && c != '.'
            })
            .to_owned()
        })
        .filter(|t| !t.is_empty())
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_strings_zero_drift() {
        assert_eq!(measure_drift("hello world", "hello world"), 0.0);
    }

    #[test]
    fn empty_inputs_zero_drift() {
        assert_eq!(measure_drift("", ""), 0.0);
    }

    #[test]
    fn one_empty_input_max_drift() {
        let d = measure_drift("hello world", "");
        assert_eq!(d, 1.0);
    }

    #[test]
    fn disjoint_inputs_max_drift() {
        let d = measure_drift("alpha beta", "gamma delta");
        assert_eq!(d, 1.0);
    }

    #[test]
    fn half_overlap() {
        // {a,b} vs {b,c} — union=3, intersection=1, similarity=1/3, distance=2/3.
        let d = measure_drift("a b", "b c");
        assert!((d - (2.0 / 3.0)).abs() < 1e-9);
    }

    #[test]
    fn case_and_punctuation_normalized() {
        let d = measure_drift("Hello, World!", "hello world");
        assert_eq!(d, 0.0);
    }

    #[test]
    fn canary_fires_on_threshold_cross() {
        let canary = GoalDriftCanary::new("alpha beta gamma", vec![1, 5, 10], 0.4);
        // Identical -> not fired.
        assert!(!canary.fired("alpha beta gamma"));
        // Fully orthogonal -> fired.
        assert!(canary.fired("delta epsilon"));
    }

    #[test]
    fn canary_clamps_threshold() {
        let c = GoalDriftCanary::new("goal", vec![1], -5.0);
        assert_eq!(c.drift_threshold, 0.0);
        let c = GoalDriftCanary::new("goal", vec![1], 5.0);
        assert_eq!(c.drift_threshold, 1.0);
        let c = GoalDriftCanary::new("goal", vec![1], f64::NAN);
        assert_eq!(c.drift_threshold, DEFAULT_DRIFT_THRESHOLD);
    }

    #[test]
    fn canary_dedup_sorts_steps() {
        let c = GoalDriftCanary::new("goal", vec![10, 1, 5, 1, 10], 0.5);
        assert_eq!(c.sampled_at_steps, vec![1, 5, 10]);
    }

    #[test]
    fn should_sample_matches_schedule() {
        let c = GoalDriftCanary::new("goal", vec![1, 5, 10], 0.5);
        assert!(c.should_sample(1));
        assert!(c.should_sample(5));
        assert!(c.should_sample(10));
        assert!(!c.should_sample(2));
        assert!(!c.should_sample(0));
    }

    #[test]
    fn blake3_hash_is_stable() {
        let c1 = GoalDriftCanary::new("the same goal", vec![1], 0.5);
        let c2 = GoalDriftCanary::new("the same goal", vec![1], 0.5);
        assert_eq!(c1.initial_goal_blake3, c2.initial_goal_blake3);
        // Hex length 64 (256 bits).
        assert_eq!(c1.initial_goal_blake3.len(), 64);
    }
}
