//! C-46 — paper-grounded invariants for the routing module.
//!
//! Centralises the numeric guards from the Uno-Orchestra paper
//! (arXiv:2605.05007). Each guard is enforced once at construction in
//! [`super::policy::RoutingPolicy::new`] and additionally cited from
//! the contract test fence headers — there is one canonical source of
//! truth for the bound and the matching error variant.
//!
//! Paper anchors (see workflow/papers/uno-orchestra-interface.md for
//! the verified extracts):
//!
//!   - `assert alpha >= 0.0 and alpha <= 1.0`
//!     ("cost-correctness blend is a convex combination" — paper
//!      Section 5; default 0.1 keeps the verifier reward dominant).
//!   - `assert S_max <= 0.10`
//!     ("routing-shaping bonus must stay << verifier reward to prevent
//!      cost-hacking incorrect answers" — paper Section 5 cap is 0.10;
//!      raising the cap unlocks the cost-hacking failure mode
//!      explicitly enumerated in the paper's Implementation pitfalls
//!      block).
//!   - direct-answer-cost-zero: enforced structurally by
//!     [`super::selective::SelectiveDelegator::select_workers`]
//!     returning `Vec::new()` on lazy-mode (no SSB dispatch beacons
//!     spawned, no AgentPool::spawn invocation, no token cost).

use thiserror::Error;

/// Lower bound for the cost-correctness blend (paper Section 5 — convex
/// combination invariant).
pub const ALPHA_MIN: f32 = 0.0;

/// Upper bound for the cost-correctness blend (paper Section 5 — convex
/// combination invariant).
pub const ALPHA_MAX: f32 = 1.0;

/// Hard cap on the routing-shaping bonus `S(tau)` (paper Section 5 —
/// "bounded < 1 so cost-hacking incorrect answers is foreclosed").
pub const S_MAX_CAP: f32 = 0.10;

/// Errors raised when paper invariants are violated.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum InvariantError {
    /// `alpha` outside the convex blend range `[0, 1]`. Paper Section 5
    /// requires `alpha in [0, 1]` because the cost-correctness blend is
    /// a convex combination.
    #[error(
        "alpha={alpha} violates the convex-blend invariant alpha in [{min}, {max}] \
         (Uno-Orchestra Section 5)"
    )]
    AlphaOutOfRange {
        /// Offending value.
        alpha: f32,
        /// Paper-anchored lower bound.
        min: f32,
        /// Paper-anchored upper bound.
        max: f32,
    },
    /// `s_max` exceeds the paper-anchored cap `0.10`. Raising the cap
    /// unlocks the cost-hacking failure mode the paper explicitly calls
    /// out (router deliberately fails the verifier to harvest cheap-
    /// and-wrong cost bonuses).
    #[error(
        "s_max={s_max} exceeds the paper-anchored cost-hacking cap {cap} \
         (Uno-Orchestra Section 5; raising the cap enables \
         cost-hacking incorrect answers)"
    )]
    SMaxExceedsCap {
        /// Offending value.
        s_max: f32,
        /// Paper-anchored cap (always [`S_MAX_CAP`]).
        cap: f32,
    },
    /// `s_max` is negative — meaningless for a non-negative shaping
    /// bonus. Not a paper assertion but a sanity guard so dashboards
    /// never see a negative reward.
    #[error("s_max={s_max} is negative; the shaping bonus must be non-negative")]
    SMaxNegative {
        /// Offending value.
        s_max: f32,
    },
}

/// Validate `alpha` against the convex-blend invariant.
pub fn validate_alpha(alpha: f32) -> Result<(), InvariantError> {
    if !alpha.is_finite() || alpha < ALPHA_MIN || alpha > ALPHA_MAX {
        return Err(InvariantError::AlphaOutOfRange {
            alpha,
            min: ALPHA_MIN,
            max: ALPHA_MAX,
        });
    }
    Ok(())
}

/// Validate `s_max` against the paper-anchored cost-hacking cap.
pub fn validate_s_max(s_max: f32) -> Result<(), InvariantError> {
    if !s_max.is_finite() {
        return Err(InvariantError::SMaxExceedsCap {
            s_max,
            cap: S_MAX_CAP,
        });
    }
    if s_max < 0.0 {
        return Err(InvariantError::SMaxNegative { s_max });
    }
    if s_max > S_MAX_CAP {
        return Err(InvariantError::SMaxExceedsCap {
            s_max,
            cap: S_MAX_CAP,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alpha_paper_default_accepted() {
        validate_alpha(0.1).expect("paper default 0.1 must pass");
    }

    #[test]
    fn alpha_bounds_inclusive() {
        validate_alpha(0.0).expect("alpha=0 must pass (degenerate correctness-only)");
        validate_alpha(1.0).expect("alpha=1 must pass (degenerate cost-only)");
    }

    #[test]
    fn alpha_negative_rejected() {
        let err = validate_alpha(-0.1).expect_err("negative alpha must fail");
        assert!(matches!(err, InvariantError::AlphaOutOfRange { .. }));
    }

    #[test]
    fn alpha_above_one_rejected() {
        let err = validate_alpha(1.5).expect_err("alpha > 1 must fail");
        assert!(matches!(err, InvariantError::AlphaOutOfRange { .. }));
    }

    #[test]
    fn alpha_nan_rejected() {
        let err = validate_alpha(f32::NAN).expect_err("NaN alpha must fail");
        assert!(matches!(err, InvariantError::AlphaOutOfRange { .. }));
    }

    #[test]
    fn s_max_paper_cap_accepted() {
        validate_s_max(S_MAX_CAP).expect("paper cap 0.10 must pass at the boundary");
    }

    #[test]
    fn s_max_above_cap_rejected() {
        // Paper Section 5: raising the cap above 0.10 unlocks cost-hacking.
        let err = validate_s_max(0.15).expect_err("s_max > 0.10 must fail");
        assert!(matches!(err, InvariantError::SMaxExceedsCap { .. }));
    }

    #[test]
    fn s_max_negative_rejected() {
        let err = validate_s_max(-0.01).expect_err("negative s_max must fail");
        assert!(matches!(err, InvariantError::SMaxNegative { .. }));
    }

    #[test]
    fn s_max_zero_accepted() {
        validate_s_max(0.0).expect("s_max=0 must pass (degenerate no-shaping)");
    }
}
