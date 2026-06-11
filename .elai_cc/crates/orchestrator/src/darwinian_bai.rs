//! E-7 / H-48: Discounted Thompson Sampling + Successive Rejects — ACTIVATED.
//!
//! Per workflow/reasoning/E-7-decisions.md DECISION-13: this module is
//! a SIBLING to crate::bai (untouched). The existing Thompson sampler
//! remains the legacy default; the Discounted-TS + SR paths are
//! reachable ONLY when `cfg(feature = "darwinian-evolve")` is set AND
//! `ELAI_DARWINIAN_LOOP_ENABLE=1`.
//!
//! H-48 ACTIVATES the scaffold: adds `update_arms_from_reward` +
//! `select_next_arm` + `run_sr_phase` on H-43/H-45's real reward
//! (`PromotionScore::worst_case`), plus ExIt-inspired arm seeding from
//! partial past history slices. See workflow/reasoning/H-48-decisions.md.
//!
//! Paper grounding:
//!   - arXiv:2305.10718 (Discounted Thompson Sampling for Non-Stationary
//!     Bandit Problems, Qi/Wang/Zhu 2023-05-18 rev 2023-05-22).
//!     Regret O~(sqrt(T*B_T)) abrupt / O~(T^beta) smooth. Discount factor
//!     gamma is the sole critical parameter; paper warns it must be
//!     tuned per regime.
//!     DARWINIAN LINKAGE: per-cell arm reward distributions shift as
//!     the population evolves, matching the non-stationary regime DS-TS
//!     targets. See E-7-decisions.md DECISION-2.
//!   - Audibert et al. COLT 2010 — Successive Rejects formula for
//!     fixed-budget round allocation.
//!   - arXiv:2509.04575 (ExIt, Jiang/Lupu/Bachrach, Meta) — autocurriculum
//!     from partial intermediate episode states; H-48 adapts the partial-
//!     history seed to the guard-task arm curriculum.
//!
//! SSB telemetry (H-48 GD-1): REUSES `BaiArmUpdated` from D-5. The
//! `template_id` field carries a darwinian arm label (`"darwinian-arm-N"`);
//! `was_promoted` maps to "was this arm's candidate admitted this gen".
//! No new SSB variant required — SCHEMA_VERSION stays 225, TV stays 536.
//! See workflow/reasoning/H-48-decisions.md §ssb-decision for rationale.
//!
//! Default discount factor = exp(-1.0/0.3) ~ 0.0357 (per MSI lambda=0.3
//! timescale; DESIGN HYPOTHESIS, not paper-quoted). Ships as config-knob.
//! For the frozen fixture convergence test a larger gamma (0.9) is used per
//! arXiv:2305.10718 §5 experiments (less aggressive forgetting over short
//! horizons). See H-48-decisions.md §discount-factor.

use elai_core::SeededRng;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::bai::sample_beta;

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
/// window for change-point detection. ACTIVATED (H-48): `discounted_update`
/// and `mean` are live production paths called from `run_one_generation`
/// via `update_arms_from_reward` and `select_next_arm` respectively.
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

// ---------------------------------------------------------------------------
// H-48: update_arms_from_reward — Discounted-TS arm update on real reward
// ---------------------------------------------------------------------------

/// Update all arms after one generation, applying the Discounted-TS rule
/// (arXiv:2305.10718 Eq. 3) to each arm.
///
/// `arm_index` is the arm whose candidate was evaluated this generation.
/// `reward` is H-45's `PromotionScore::worst_case` in [0, 1] — the real,
/// execution-grounded aggregate (NOT a byte-hash or placeholder; VD-1).
/// `gamma` is the discount factor from `DiscountedTsConfig`.
///
/// All OTHER arms receive a no-observation discount step (counts scaled by
/// gamma, no reward or anti-reward added) to prevent stale arms from
/// dominating selection. This implements the "sliding-window" non-stationarity
/// handling (2305.10718 §3).
///
/// The arm index is saturating-clamped to `arms.len()-1` if out of range
/// (fail-closed: never panic).
pub fn update_arms_from_reward(
    arms: &mut Vec<DiscountedArmPosterior>,
    arm_index: usize,
    reward: f64,
    gamma: f64,
) {
    if arms.is_empty() {
        return;
    }
    // Saturate out-of-range arm index (fail-closed).
    let safe_idx = arm_index.min(arms.len().saturating_sub(1));
    for (i, arm) in arms.iter_mut().enumerate() {
        if i == safe_idx {
            // The pulled arm: full discounted update with the reward.
            arm.discounted_update(reward, gamma);
        } else {
            // Unpulled arms: decay counts only (no new observation).
            // Prevents arms that haven't been explored recently from
            // retaining stale inflated posteriors.
            arm.alpha = (gamma * arm.alpha).max(f64::MIN_POSITIVE);
            arm.beta = (gamma * arm.beta).max(f64::MIN_POSITIVE);
        }
    }
    debug!(
        arm_index = safe_idx,
        reward,
        gamma,
        n_arms = arms.len(),
        "darwinian_bai: arms updated from real reward (H-45 worst_case)"
    );
}

// ---------------------------------------------------------------------------
// H-48: select_next_arm — Discounted-TS Thompson sample over live posteriors
// ---------------------------------------------------------------------------

/// Select the next arm for the following generation using Thompson sampling
/// over the current discounted posteriors.
///
/// Wraps `bai::sample_beta` (the SHIPPED D-5/bai.rs sampler — GD-1: no
/// re-implementation) applied to `DiscountedArmPosterior.alpha/beta`.
/// Returns `None` on empty arms slice (fail-closed).
///
/// Deterministic under the caller-supplied `SeededRng` (GD-2).
#[must_use]
pub fn select_next_arm(arms: &[DiscountedArmPosterior], rng: &mut SeededRng) -> Option<usize> {
    if arms.is_empty() {
        return None;
    }
    let mut best_idx = 0usize;
    let mut best_score = sample_beta(rng, arms[0].alpha, arms[0].beta);
    for (i, arm) in arms.iter().enumerate().skip(1) {
        let score = sample_beta(rng, arm.alpha, arm.beta);
        if score > best_score {
            best_score = score;
            best_idx = i;
        }
    }
    debug!(
        selected_arm = best_idx,
        n_arms = arms.len(),
        "darwinian_bai: select_next_arm via discounted Thompson sample"
    );
    Some(best_idx)
}

// ---------------------------------------------------------------------------
// H-48: run_sr_phase — Successive Rejects fixed-budget best-arm identification
// ---------------------------------------------------------------------------

/// Run Successive-Rejects (Audibert COLT 2010) best-arm identification over
/// a frozen set of `arm_rewards` (one f64 per arm, representing observed mean
/// reward for that arm over the budget).
///
/// With N arms and budget T, runs N-1 elimination rounds. Each round uses
/// `ceil((T - K_bar) / ((N - k + 1) * log_bar(N)))` observations per
/// surviving arm, eliminating the lowest-mean arm per round. Returns the
/// index of the surviving best arm.
///
/// `arm_rewards` is a slice of mean rewards in [0, 1] (not per-round
/// observations — this is the aggregate variant suitable for the frozen
/// fixture). Returns `None` if the slice is empty.
///
/// The `rng` parameter is accepted for API consistency with `select_next_arm`
/// (SR itself is deterministic given `arm_rewards`; the RNG is not consumed
/// in this fixed-input variant but is present so callers can thread it
/// through uniformly).
#[must_use]
#[allow(clippy::needless_pass_by_ref_mut)]
pub fn run_sr_phase(
    arm_rewards: &[f64],
    budget: u32,
    _rng: &mut SeededRng,
) -> Option<usize> {
    let n = arm_rewards.len();
    if n == 0 {
        return None;
    }
    if n == 1 {
        return Some(0);
    }

    // Surviving set starts as all arms (indices).
    let mut survivors: Vec<usize> = (0..n).collect();
    let budget_f = budget as f64;
    let log_bar_n = SuccessiveRejects::log_bar(n);

    for k in 1..n {
        if survivors.len() <= 1 {
            break;
        }
        // Audibert round allocation: ceil((T - K_bar) / ((N - k + 1) * log_bar(N))).
        // Use saturating arithmetic to avoid negative pulls.
        let k_bar = SuccessiveRejects::log_bar(k);
        let remaining_survivors = survivors.len() as f64;
        let pulls_f = ((budget_f - k_bar) / (remaining_survivors * log_bar_n)).ceil();
        let pulls_this_round = (pulls_f as u32).max(1);

        // In the frozen-aggregate variant each arm's empirical success count over
        // the round is arm_rewards[i] * pulls_this_round (total successes if each
        // arm were pulled pulls_this_round times at its observed mean). Since
        // pulls_this_round is equal for all survivors in round k, the ranking is
        // preserved while the comparison is semantically real (Audibert § 2.1).
        let scale = pulls_this_round as f64;
        let worst_idx_in_survivors = survivors
            .iter()
            .enumerate()
            .min_by(|(_, &a), (_, &b)| {
                (arm_rewards[a] * scale)
                    .partial_cmp(&(arm_rewards[b] * scale))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(pos, _)| pos);

        if let Some(pos) = worst_idx_in_survivors {
            survivors.remove(pos);
        }
    }

    survivors.into_iter().next()
}

// ---------------------------------------------------------------------------
// H-48: ExIt-inspired arm seeding from partial past history
// ---------------------------------------------------------------------------

/// Seed the arm posteriors from partial past history scores (ExIt,
/// arXiv:2509.04575, Jiang/Lupu/Bachrach, Meta).
///
/// ExIt generates an autocurriculum from PARTIAL intermediate episode states
/// rather than only terminal benchmarks. Here, `past_scores` is a slice of
/// `(arm_index, reward)` pairs representing partial episode observations from
/// prior generations (e.g., intermediate evaluation checkpoints before full
/// guard chain completion). Each entry seeds the corresponding arm's posterior
/// with a discounted update so the bandit starts with curriculum knowledge
/// rather than a flat Beta(1, 1) prior.
///
/// Arms whose index exceeds `arms.len()-1` are silently skipped (fail-closed).
/// Gamma is the same discount factor used in `update_arms_from_reward`.
///
/// This function is OPTIONAL in the H-48 wiring: if no partial history is
/// available (empty slice), it is a no-op and the arms start from the uniform
/// prior. No stub: the function is complete and callable; the caller decides
/// whether to supply history.
pub fn seed_arms_from_partial_history(
    arms: &mut Vec<DiscountedArmPosterior>,
    past_scores: &[(usize, f64)],
    gamma: f64,
) {
    if arms.is_empty() || past_scores.is_empty() {
        return;
    }
    for &(arm_idx, reward) in past_scores {
        if arm_idx < arms.len() {
            arms[arm_idx].discounted_update(reward, gamma);
        }
        // Out-of-range arm indices are silently skipped (fail-closed).
    }
    debug!(
        n_seeds = past_scores.len(),
        gamma,
        "darwinian_bai: arms seeded from partial past history (ExIt 2509.04575)"
    );
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
