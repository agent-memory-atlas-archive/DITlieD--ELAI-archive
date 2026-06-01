//! C-45 — Core types for the contrastive credit-attribution module.
//!
//! `DropPolicy` is the harness-side "drop one role" knob; threaded into
//! the run-with-policy callback so the production agent-spawn paths can
//! observe the gated role without RuntimeContext (which is clock/entropy,
//! not agent-policy — per the cross-check rebase note).
//!
//! `CreditScore` is a paper-anchored scalar in `[-1, +1]`. The newtype
//! enforces the range invariant at construction; the SkillRegistry write-
//! through (`integration::apply_credit_to_manifest`) preserves the same
//! clamp behaviour so downstream consumers cannot smuggle out-of-band
//! values.
//!
//! `CreditReport` is the per-pilot-run aggregate: a `HashMap<AgentRole,
//! f32>` of per-role credits plus a producer-side timestamp for SSB
//! ordering.

use elai_ipc_types::AgentRole;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

/// Harness-side policy gating one `AgentRole` out of the production
/// agent-spawn path. `None` means "baseline run" (all roles available);
/// `Some(role)` means "drop that role and route around it via the
/// per-role fallback".
///
/// The wire form is intentionally trivial — this is a HARNESS knob, not
/// an SSB row. The corresponding SSB beacon
/// (`SsbActionType::AgentRoleDropped`) is emitted by the harness driver
/// when it enters a non-baseline configuration.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DropPolicy {
    /// `Some(role)` to forcibly drop the role from the run; `None` for
    /// the baseline configuration.
    pub dropped_role: Option<AgentRole>,
}

impl DropPolicy {
    /// Build a baseline (no-drop) policy.
    #[must_use]
    pub fn baseline() -> Self {
        Self { dropped_role: None }
    }

    /// Build a single-role-drop policy.
    #[must_use]
    pub fn drop(role: AgentRole) -> Self {
        Self {
            dropped_role: Some(role),
        }
    }

    /// True for the baseline configuration.
    #[must_use]
    pub fn is_baseline(self) -> bool {
        self.dropped_role.is_none()
    }
}

/// Scalar credit score `c in [-1, +1]` per CANTANTE Section 3.2.
///
/// Negative values mean "dropping the role helped" (the role was net
/// noise on this task). Positive values mean "dropping the role hurt"
/// (the role contributed to system success). Magnitudes are NOT
/// probabilities — they are normalised attribution deltas.
///
/// Invariant: the inner `f32` is clamped to `[-1.0, +1.0]` at
/// construction. Round-trip via serde preserves the invariant because
/// the underlying value is always within range before serialisation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CreditScore(f32);

impl CreditScore {
    /// Wrap an `f32` with the paper-anchored `[-1.0, +1.0]` clamp.
    /// Non-finite values clamp to zero (no information about direction).
    #[must_use]
    pub fn clamped(value: f32) -> Self {
        if !value.is_finite() {
            return Self(0.0);
        }
        Self(value.clamp(-1.0, 1.0))
    }

    /// Underlying value (always in `[-1.0, +1.0]`).
    #[must_use]
    pub fn value(self) -> f32 {
        self.0
    }
}

impl Default for CreditScore {
    fn default() -> Self {
        Self(0.0)
    }
}

/// Per-pilot-run aggregate of contrastive credit scores.
///
/// `per_role` maps each `AgentRole` in the pilot's ablation set to its
/// scalar credit. Roles not present in the map were not part of this
/// pilot's ablation (e.g. the eight B-40/B-41 roles are intentionally
/// excluded in the C-45 pilot — see `credit_attribution::partition`).
///
/// `ts_ms` is the producer-side wall-clock timestamp (milliseconds
/// since UNIX epoch). The harness owns timestamp resolution via the
/// injected `ClockProvider` — never `std::time::SystemTime`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreditReport {
    /// Per-role credit map keyed by the canonical `AgentRole` enum
    /// (not a string), so consumers cannot smuggle a misspelled role.
    pub per_role: FxHashMap<AgentRole, f32>,
    /// Producer-side wall-clock timestamp in milliseconds since UNIX
    /// epoch.
    pub ts_ms: u64,
}

impl CreditReport {
    /// Build a fresh report with the given timestamp.
    #[must_use]
    pub fn new(ts_ms: u64) -> Self {
        Self {
            per_role: FxHashMap::default(),
            ts_ms,
        }
    }

    /// Record a credit score for a role (clamped via `CreditScore`).
    pub fn record(&mut self, role: AgentRole, score: CreditScore) {
        self.per_role.insert(role, score.value());
    }

    /// Lookup a role's score (returns `None` if the role was not part
    /// of this pilot's ablation set).
    #[must_use]
    pub fn get(&self, role: AgentRole) -> Option<f32> {
        self.per_role.get(&role).copied()
    }

    /// Total number of roles covered.
    #[must_use]
    pub fn len(&self) -> usize {
        self.per_role.len()
    }

    /// True when no roles are recorded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.per_role.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drop_policy_baseline_has_no_role() {
        let p = DropPolicy::baseline();
        assert!(p.is_baseline());
        assert_eq!(p.dropped_role, None);
    }

    #[test]
    fn drop_policy_drop_carries_role() {
        let p = DropPolicy::drop(AgentRole::Executor);
        assert!(!p.is_baseline());
        assert_eq!(p.dropped_role, Some(AgentRole::Executor));
    }

    #[test]
    fn credit_score_clamps_high() {
        let s = CreditScore::clamped(5.5);
        assert!((s.value() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn credit_score_clamps_low() {
        let s = CreditScore::clamped(-9.0);
        assert!((s.value() - -1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn credit_score_zero_for_nan() {
        let s = CreditScore::clamped(f32::NAN);
        assert!((s.value() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn credit_score_passes_in_band() {
        let s = CreditScore::clamped(0.3);
        assert!((s.value() - 0.3).abs() < f32::EPSILON);
    }

    #[test]
    fn credit_report_record_and_get() {
        let mut r = CreditReport::new(1_700_000_000_000);
        r.record(AgentRole::Executor, CreditScore::clamped(0.4));
        assert_eq!(r.get(AgentRole::Executor), Some(0.4));
        assert_eq!(r.get(AgentRole::Auditor), None);
        assert_eq!(r.len(), 1);
        assert!(!r.is_empty());
    }

    #[test]
    fn drop_policy_serde_round_trip() {
        let p = DropPolicy::drop(AgentRole::Researcher);
        let bytes = rmp_serde::to_vec(&p).expect("encode");
        let decoded: DropPolicy = rmp_serde::from_slice(&bytes).expect("decode");
        assert_eq!(decoded, p);
    }
}
