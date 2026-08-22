//! C-46 — Routing policy: typed configuration for the selective
//! delegator.
//!
//! `RoutingPolicy` is the in-memory typed surface; `RoutingPolicyToml`
//! is the serde-deserialized mirror used by the CLI runtime-config
//! loader. Construction enforces the paper-anchored invariants from
//! [`super::invariants`] so a misconfigured TOML section can never
//! flow into the dispatch path with a cost-hacking-enabling S_max or
//! an out-of-band alpha.
//!
//! Default values (all paper-grounded against arXiv:2605.05007):
//!
//!   - `enabled = false` — selective delegation is OPT-IN per project
//!     (ELAI deployment policy; mirrors the `[agent.credit_attribution].enabled = false`
//!     default).
//!   - `alpha = 0.1` — paper Section 5 default; keeps verifier reward
//!     dominant.
//!   - `s_max = 0.10` — paper Section 5 cap; raising unlocks cost-
//!     hacking.
//!   - `max_workers_per_query = 8` — pilot scope cap (mirrors C-45
//!     `pilot_roles().len()`); a single dispatch never exceeds the
//!     8-base-role pilot set.

use serde::{Deserialize, Serialize};

use super::invariants::{validate_alpha, validate_s_max, InvariantError};

/// Paper-anchored default cost-correctness blend.
pub const DEFAULT_ALPHA: f32 = 0.1;

/// Paper-anchored default routing-shaping bonus cap.
pub const DEFAULT_S_MAX: f32 = 0.10;

/// Pilot scope cap — matches `credit_attribution::pilot_roles().len()`.
pub const DEFAULT_MAX_WORKERS_PER_QUERY: u32 = 8;

/// Typed in-memory routing policy. Constructed via
/// [`RoutingPolicy::new`] (enforces paper invariants) or
/// [`RoutingPolicy::default`] (uses paper defaults). The struct itself
/// derives `Copy + Clone` so the SelectiveDelegator can hold it by
/// value without lifetime-tracking.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RoutingPolicy {
    /// Master enable switch. When `false`, the SelectiveDelegator path
    /// is fully inert and callers fall back to the baseline always-
    /// spawn chain.
    pub enabled: bool,
    /// Cost-correctness blend in `[0, 1]` (paper Section 5; default 0.1).
    pub alpha: f32,
    /// Routing-shaping bonus cap in `[0, S_MAX_CAP]` (paper Section 5;
    /// default 0.10).
    pub s_max: f32,
    /// Maximum workers dispatched per query. Cap matches the C-45
    /// 8-base-role pilot scope.
    pub max_workers_per_query: u32,
}

impl Default for RoutingPolicy {
    fn default() -> Self {
        // Default is the OFF-by-default ELAI deployment policy with
        // paper-anchored knobs ready for activation.
        Self {
            enabled: false,
            alpha: DEFAULT_ALPHA,
            s_max: DEFAULT_S_MAX,
            max_workers_per_query: DEFAULT_MAX_WORKERS_PER_QUERY,
        }
    }
}

impl RoutingPolicy {
    /// Construct a routing policy with explicit knobs. Returns an
    /// [`InvariantError`] when `alpha` is outside `[0, 1]` or `s_max`
    /// exceeds the paper-anchored cost-hacking cap.
    pub fn new(
        enabled: bool,
        alpha: f32,
        s_max: f32,
        max_workers_per_query: u32,
    ) -> Result<Self, InvariantError> {
        validate_alpha(alpha)?;
        validate_s_max(s_max)?;
        Ok(Self {
            enabled,
            alpha,
            s_max,
            max_workers_per_query,
        })
    }

    /// True when this policy is fully inert (caller falls back to
    /// baseline). Mirrors the `[agent.credit_attribution].enabled`
    /// gate from C-45.
    #[must_use]
    pub fn is_inert(&self) -> bool {
        !self.enabled
    }
}

/// `[agent.selective_delegation]` TOML mirror. Loaded by the CLI's
/// runtime-config pass and converted to [`RoutingPolicy`] via
/// [`RoutingPolicyToml::to_policy`]. Field-level defaults mirror the
/// in-memory `Default` impl so an absent section preserves the OFF-by-
/// default behaviour exactly.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct RoutingPolicyToml {
    /// Master enable switch (TOML default: `false`).
    pub enabled: bool,
    /// Cost-correctness blend (TOML default: `0.1`).
    pub alpha: f32,
    /// Routing-shaping bonus cap (TOML default: `0.10`).
    pub s_max: f32,
    /// Max workers per query (TOML default: `8`).
    pub max_workers_per_query: u32,
}

impl Default for RoutingPolicyToml {
    fn default() -> Self {
        Self {
            enabled: false,
            alpha: DEFAULT_ALPHA,
            s_max: DEFAULT_S_MAX,
            max_workers_per_query: DEFAULT_MAX_WORKERS_PER_QUERY,
        }
    }
}

impl RoutingPolicyToml {
    /// Convert the TOML mirror into a paper-invariant-checked policy.
    /// Returns an [`InvariantError`] when the TOML values violate the
    /// paper bounds — the runtime-config loader is expected to surface
    /// this error to the user (deterministic warn-not-panic per the
    /// `[agent.credit_attribution]` precedent).
    pub fn to_policy(self) -> Result<RoutingPolicy, InvariantError> {
        RoutingPolicy::new(
            self.enabled,
            self.alpha,
            self.s_max,
            self.max_workers_per_query,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routing::invariants::{ALPHA_MAX, S_MAX_CAP};

    #[test]
    fn default_matches_paper_defaults() {
        let p = RoutingPolicy::default();
        assert!(!p.enabled, "default must be OFF (opt-in)");
        assert!((p.alpha - DEFAULT_ALPHA).abs() < f32::EPSILON);
        assert!((p.s_max - DEFAULT_S_MAX).abs() < f32::EPSILON);
        assert_eq!(p.max_workers_per_query, DEFAULT_MAX_WORKERS_PER_QUERY);
        assert!(p.is_inert(), "default must be inert");
    }

    #[test]
    fn new_accepts_paper_defaults() {
        let p = RoutingPolicy::new(false, DEFAULT_ALPHA, DEFAULT_S_MAX, 8)
            .expect("paper defaults must pass");
        assert!((p.alpha - DEFAULT_ALPHA).abs() < f32::EPSILON);
    }

    #[test]
    fn new_rejects_alpha_above_one() {
        let err = RoutingPolicy::new(true, 1.5, DEFAULT_S_MAX, 8).expect_err("alpha > 1 must fail");
        assert!(matches!(err, InvariantError::AlphaOutOfRange { .. }));
    }

    #[test]
    fn new_rejects_s_max_above_cap() {
        let err =
            RoutingPolicy::new(true, DEFAULT_ALPHA, 0.5, 8).expect_err("s_max > 0.10 must fail");
        assert!(matches!(err, InvariantError::SMaxExceedsCap { .. }));
    }

    #[test]
    fn new_accepts_alpha_at_max() {
        RoutingPolicy::new(true, ALPHA_MAX, DEFAULT_S_MAX, 8)
            .expect("alpha=1 must pass (boundary)");
    }

    #[test]
    fn new_accepts_s_max_at_cap() {
        RoutingPolicy::new(true, DEFAULT_ALPHA, S_MAX_CAP, 8)
            .expect("s_max=0.10 must pass (boundary)");
    }

    #[test]
    fn toml_default_round_trip() {
        let toml = RoutingPolicyToml::default();
        let policy = toml.to_policy().expect("paper defaults must round-trip");
        assert!(!policy.enabled);
        assert!(policy.is_inert());
    }

    #[test]
    fn toml_rejects_bad_alpha() {
        let bad = RoutingPolicyToml {
            enabled: true,
            alpha: -0.5,
            s_max: DEFAULT_S_MAX,
            max_workers_per_query: 8,
        };
        let err = bad.to_policy().expect_err("alpha < 0 must fail");
        assert!(matches!(err, InvariantError::AlphaOutOfRange { .. }));
    }

    #[test]
    fn legacy_policy_key_is_ignored() {
        const LEGACY_POLICY_TOML_FIXTURE: &str = r#"
enabled = true
alpha = 0.2
s_max = 0.05
max_turns = 99
max_workers_per_query = 4
"#;
        let toml: RoutingPolicyToml =
            toml::from_str(LEGACY_POLICY_TOML_FIXTURE).expect("legacy TOML must parse");
        let policy = toml.to_policy().expect("legacy TOML must convert");
        assert!(policy.enabled);
        assert!((policy.alpha - 0.2).abs() < f32::EPSILON);
        assert!((policy.s_max - 0.05).abs() < f32::EPSILON);
        assert_eq!(policy.max_workers_per_query, 4);
    }
}
