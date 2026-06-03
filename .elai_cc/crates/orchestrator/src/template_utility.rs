//! F-7 / Slice A2 -- Laplace add-one utility scoring for CBR template ranking
//! (HALF B only).
//!
//! Crystallized templates are stored ONE-per-key in the [`compiled_pipeline`]
//! cache, each carrying a persisted `first_try_success_rate` (`f32`) and a
//! `use_count` (`u32`) (see
//! `crates/orchestrator/src/compiled_pipeline.rs:209-210`). The raw
//! `first_try_success_rate` over-trusts low-sample templates: a single lucky
//! firing reads as a perfect `1.0`, which would let a 1/1 template out-rank a
//! battle-tested 95/100 one at the `try_execute` reuse gate
//! (`compiled_pipeline.rs:499`, `first_try_success_rate < confidence_threshold`).
//!
//! This module applies Laplace add-one (Beta(ALPHA, BETA)) smoothing so the
//! score is the posterior mean of the success probability under a uniform
//! prior:
//!
//! ```text
//! utility = (successes + ALPHA) / (total + ALPHA + BETA),   ALPHA = BETA = 1.0
//! ```
//!
//! Consequences (the F-7 smoothing gate -- NOT any numeric uplift target; the
//! Springdrift +10.7% figure is UNGROUNDED and is deliberately encoded
//! nowhere):
//!   * `0/0`   -> `(0+1)/(0+2)        = 0.5`   (neutral prior: "unknown")
//!   * `1/1`   -> `(1+1)/(1+2)        = 2/3 ~= 0.667`  (NOT `1.0`)
//!   * `95/100`-> `(95+1)/(100+2)     = 96/102 ~= 0.941`
//! so the well-sampled 95/100 template out-ranks the lucky 1/1 template even
//! though its raw first-try ratio is lower.
//!
//! `ALPHA`/`BETA` are ELAI-owned constants. Everything here is pure and
//! deterministic: no clock, no randomness, no I/O. The recorded
//! `first_try_success_rate` is KEPT as the baseline (W-2 no-removal); the
//! dashboard surfaces both the raw ratio and the smoothed utility so a reviewer
//! can see the before/after.

/// Laplace pseudo-count for the "success" outcome (Beta prior alpha).
///
/// ELAI-owned. `1.0` = add-one / uniform-prior smoothing.
pub const LAPLACE_ALPHA: f32 = 1.0;

/// Laplace pseudo-count for the "failure" outcome (Beta prior beta).
///
/// ELAI-owned. `1.0` = add-one / uniform-prior smoothing.
pub const LAPLACE_BETA: f32 = 1.0;

/// Laplace add-one smoothed utility over an integer success / firing count.
///
/// Returns the posterior mean `(successes + ALPHA) / (total + ALPHA + BETA)`.
/// When `total == 0` there is no observed evidence, so the score falls to the
/// neutral prior mean `ALPHA / (ALPHA + BETA)` (`0.5` for the default add-one
/// constants) rather than dividing by zero.
///
/// `successes` is clamped to `total` so a caller cannot fabricate a score above
/// the no-failure ceiling by passing `successes > total`.
pub fn laplace_utility(successes: u32, total: u32) -> f32 {
    // Guard the degenerate zero-evidence case explicitly: no firings means the
    // posterior IS the prior mean, and `(0 + ALPHA) / (0 + ALPHA + BETA)`
    // already equals that, but we keep the branch so the "unknown" semantics
    // are obvious at the call site and never depend on float division by zero.
    if total == 0 {
        return LAPLACE_ALPHA / (LAPLACE_ALPHA + LAPLACE_BETA);
    }

    let clamped_successes = successes.min(total);
    (clamped_successes as f32 + LAPLACE_ALPHA) / (total as f32 + LAPLACE_ALPHA + LAPLACE_BETA)
}

/// Laplace utility recovered from a persisted [`compiled_pipeline`] record's
/// `first_try_success_rate` (`f32` ratio) and `use_count` (`u32` firings).
///
/// The compiled-pipeline store keeps the success RATIO, not the raw success
/// count, so this reconstructs the integer success count as
/// `round(first_try_success_rate * use_count)`, clamps it into `[0, use_count]`
/// (a stored ratio is always in `[0, 1]`, but rounding at the boundary plus
/// float error could nudge it one over), and feeds [`laplace_utility`].
///
/// A `use_count` of `0` yields the neutral `0.5` prior, matching
/// [`laplace_utility`]`(_, 0)`.
pub fn laplace_utility_from_fields(first_try_success_rate: f32, use_count: u32) -> f32 {
    let successes = (first_try_success_rate * use_count as f32)
        .round()
        .max(0.0) as u32;
    let clamped = successes.min(use_count);
    laplace_utility(clamped, use_count)
}

/// A single template's smoothed ranking signal, ready to record on the
/// [`crate::ssb::SsbActionType::TemplateUtilityRanked`] beacon.
///
/// Pure data: `utility` is the smoothed score, `first_try_ratio` is the raw
/// recorded baseline (W-2 no-removal -- the dashboard shows both), and
/// `sample_n` is the firing count the smoothing was computed over.
#[derive(Debug, Clone, PartialEq)]
pub struct TemplateUtility {
    /// Concept-class key the template was crystallized under.
    pub concept_class: String,
    /// Stable hash of the plan skeleton the template encodes.
    pub plan_skeleton_hash: u64,
    /// Laplace add-one smoothed utility (the value that gates reuse).
    pub utility: f32,
    /// Raw recorded `first_try_success_rate` baseline (kept for before/after).
    pub first_try_ratio: f32,
    /// Number of observed firings the utility was smoothed over (`use_count`).
    pub sample_n: u32,
}

impl TemplateUtility {
    /// Build a [`TemplateUtility`] straight from a persisted compiled-pipeline
    /// record's `first_try_success_rate` + `use_count`, computing the smoothed
    /// `utility` via [`laplace_utility_from_fields`].
    ///
    /// Pure: no clock, no randomness, no I/O.
    pub fn from_pipeline_fields(
        concept_class: String,
        plan_skeleton_hash: u64,
        first_try_success_rate: f32,
        use_count: u32,
    ) -> Self {
        Self {
            concept_class,
            plan_skeleton_hash,
            utility: laplace_utility_from_fields(first_try_success_rate, use_count),
            first_try_ratio: first_try_success_rate,
            sample_n: use_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_evidence_falls_to_neutral_prior() {
        assert!((laplace_utility(0, 0) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn one_of_one_is_two_thirds_not_one() {
        let u = laplace_utility(1, 1);
        assert!((u - (2.0 / 3.0)).abs() < 1e-6, "got {u}");
        assert!(u < 1.0);
    }

    #[test]
    fn ninety_five_of_hundred_out_ranks_lucky_one_of_one() {
        assert!(laplace_utility(95, 100) > laplace_utility(1, 1));
    }

    #[test]
    fn monotonic_in_successes() {
        let total = 10;
        let mut prev = laplace_utility(0, total);
        for s in 1..=total {
            let cur = laplace_utility(s, total);
            assert!(cur > prev, "utility must increase with successes at s={s}");
            prev = cur;
        }
    }

    #[test]
    fn successes_clamped_to_total() {
        // Fabricating successes > total must not exceed the no-failure ceiling.
        assert_eq!(laplace_utility(50, 10), laplace_utility(10, 10));
    }

    #[test]
    fn from_fields_matches_integer_form() {
        // 1.0 ratio over 1 firing reconstructs successes=1, total=1.
        assert!((laplace_utility_from_fields(1.0, 1) - laplace_utility(1, 1)).abs() < 1e-6);
        // 0.95 over 100 reconstructs successes=95.
        assert!(
            (laplace_utility_from_fields(0.95, 100) - laplace_utility(95, 100)).abs() < 1e-6
        );
        // 0 firings -> neutral prior.
        assert!((laplace_utility_from_fields(0.0, 0) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn template_utility_carries_baseline() {
        let tu = TemplateUtility::from_pipeline_fields(
            "rest_crud_api".to_string(),
            0x5f3e_27a1_0b9c_d4e2,
            1.0,
            1,
        );
        assert_eq!(tu.first_try_ratio, 1.0);
        assert_eq!(tu.sample_n, 1);
        assert!((tu.utility - (2.0 / 3.0)).abs() < 1e-6);
    }
}
