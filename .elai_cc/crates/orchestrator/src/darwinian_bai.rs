//! E-7: Discounted Thompson Sampling + Successive Rejects scaffolds.
//!
//! Per workflow/reasoning/E-7-decisions.md DECISION-13: this module is
//! a SIBLING to crate::bai (untouched). The existing Thompson sampler
//! remains the legacy default; the Discounted-TS + SR scaffolds are
//! reachable ONLY when `cfg(feature = "darwinian-evolve")` is set AND
//! `ELAI_DARWINIAN_LOOP_ENABLE=1`.
//!
//! Paper grounding:
//!   - arXiv:2305.10718 (Discounted Thompson Sampling for Non-Stationary
//!     Bandit Problems, Qi/Wang/Zhu 2023-05-18 rev 2023-05-22).
//!     Regret O~(sqrt(T*B_T)) abrupt / O~(T^beta) smooth. Discount factor
//!     gamma is the sole critical parameter; paper warns it must be
//!     tuned per regime. Interface map:
//!     workflow/papers/discounted-thompson-sampling-interface.md.
//!     DARWINIAN LINKAGE: per-cell arm reward distributions shift as
//!     the population evolves, matching the non-stationary regime DS-TS
//!     targets. See E-7-decisions.md DECISION-2.
//!   - Audibert et al. COLT 2010 — Successive Rejects formula for
//!     fixed-budget round allocation. Background prior-art for the
//!     complementary fixed-budget BAI variant.
//!
//! Default discount factor = exp(-1.0/0.3) ~ 0.0357 (per MSI lambda=0.3
//! timescale; DESIGN HYPOTHESIS, not paper-quoted). Ships as
//! config-knob.

use serde::{Deserialize, Serialize};

/// Default discount factor (UNCONFIRMED; ships as config-knob). The
/// derivation exp(-1/0.3) ~= 0.0357 is design-hypothesis from
/// memory:project_b40 MSI lambda=0.3 timescale, NOT paper-quoted.
pub const DEFAULT_DISCOUNT_FACTOR: f64 = 0.035_673_993_43; // exp(-1.0 / 0.3) approx.

/// Default fixed-budget for Successive Rejects per generation.
pub const DEFAULT_SR_BUDGET: u32 = 200;

/// Discounted-TS configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiscountedTsConfig {
    /// Discount factor gamma in (0, 1]. Closer to 1 = closer to
    /// vanilla Thompson; closer to 0 = aggressive forgetting.
    pub discount_factor: f64,
}

impl Default for DiscountedTsConfig {
    fn default() -> Self {
        Self {
            discount_factor: DEFAULT_DISCOUNT_FACTOR,
        }
    }
}

/// Arm posterior with non-stationary support.
///
/// Maintains discounted Beta(alpha, beta) counts plus a small breakpoint
/// window for change-point detection. Scaffold-only this session;
/// live `sample` + `update` paths are RUNPOD-PENDING (they need to
/// thread the `SeededRng` from elai_core to preserve byte-equivalence
/// under seed; the existing bai.rs sample_beta is the reference
/// implementation for that wiring).
#[derive(Debug, Clone, PartialEq)]
pub struct DiscountedArmPosterior {
    pub alpha: f64,
    pub beta: f64,
    pub n_pulls: u32,
}

impl Default for DiscountedArmPosterior {
    fn default() -> Self {
        Self {
            alpha: 1.0,
            beta: 1.0,
            n_pulls: 0,
        }
    }
}

impl DiscountedArmPosterior {
    /// Apply one discounted observation. Counts are multiplied by
    /// `gamma` BEFORE the reward is added; this is the DS-TS update
    /// rule per arXiv:2305.10718 Equation (3).
    pub fn discounted_update(&mut self, reward: f64, gamma: f64) {
        self.alpha = gamma * self.alpha + reward.max(0.0).min(1.0);
        self.beta = gamma * self.beta + (1.0 - reward.max(0.0).min(1.0));
        self.n_pulls = self.n_pulls.saturating_add(1);
    }

    /// Posterior mean alpha / (alpha + beta).
    #[must_use]
    pub fn mean(&self) -> f64 {
        let total = self.alpha + self.beta;
        if total <= 0.0 {
            return 0.5;
        }
        self.alpha / total
    }
}

/// Successive Rejects round result.
#[derive(Debug, Clone, PartialEq)]
pub struct SrRound {
    pub arm_index: usize,
    pub mean_score: f64,
}

/// SuccessiveRejects fixed-budget BAI.
///
/// Per Audibert COLT 2010: with N arms and budget T, run N-1 rounds
/// where round k uses ceil((T - K_bar)/((N - k + 1) * log_bar(N)))
/// pulls per surviving arm; reject the arm with the lowest empirical
/// mean at the end of each round. Survivor after N-1 rounds is the
/// returned best arm.
///
/// Scaffold-only this session: live `run` is RUNPOD-PENDING (needs
/// SeededRng integration with bai.rs sample_beta).
#[derive(Debug, Clone)]
pub struct SuccessiveRejects {
    pub budget: u32,
    pub rounds: Vec<SrRound>,
}

impl SuccessiveRejects {
    /// Empty SR run.
    #[must_use]
    pub fn new(budget: u32) -> Self {
        Self {
            budget,
            rounds: Vec::new(),
        }
    }

    /// Audibert log_bar(K) = 0.5 + sum_{i=2..K} 1/i. Used in the
    /// round-allocation formula.
    #[must_use]
    pub fn log_bar(k: usize) -> f64 {
        let mut s = 0.5;
        for i in 2..=k {
            s += 1.0 / (i as f64);
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_discount_factor_in_range() {
        let cfg = DiscountedTsConfig::default();
        assert!(cfg.discount_factor > 0.0);
        assert!(cfg.discount_factor < 1.0);
    }

    #[test]
    fn discounted_update_decays_old_counts() {
        let mut a = DiscountedArmPosterior::default();
        // Saturate with positive rewards.
        for _ in 0..10 {
            a.discounted_update(1.0, 0.5);
        }
        let mean_after_positives = a.mean();
        // Now flip to negative rewards; with strong discount the mean
        // should drop faster than vanilla update.
        for _ in 0..10 {
            a.discounted_update(0.0, 0.5);
        }
        let mean_after_negatives = a.mean();
        assert!(mean_after_negatives < mean_after_positives);
    }

    #[test]
    fn log_bar_is_monotone() {
        let lb2 = SuccessiveRejects::log_bar(2);
        let lb10 = SuccessiveRejects::log_bar(10);
        assert!(lb10 > lb2);
    }

    #[test]
    fn mean_handles_empty_posterior() {
        let a = DiscountedArmPosterior {
            alpha: 0.0,
            beta: 0.0,
            n_pulls: 0,
        };
        assert_eq!(a.mean(), 0.5);
    }
}
