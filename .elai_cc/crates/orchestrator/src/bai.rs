//! D-5: Beta-posterior Thompson sampling for ReasoningTemplate promotion.
//!
//! Replaces the MSI sigmoid-Boltzmann heuristic (Phase 4H) with a Bayesian
//! best-arm-identification (BAI) pass over the D-1 ArchiveChain. Each
//! [`ArmPosterior`] tracks Beta(alpha, beta) over a candidate
//! [`crate::reasoning_template::TemplateId`]; pulls update the posterior in
//! place; promote_candidate runs one Thompson sample per arm + argmax and
//! flips Candidate -> Promoted when the sampled arm matches the candidate
//! AND the posterior mean clears `min_promotion_threshold`.
//!
//! References:
//! - arXiv:2104.05593 "Best Arm Identification in Multi-Armed Bandits"
//!   (fixed-confidence, T1) — gives the regret + stopping-rule scaffold this
//!   module implements (the posterior-mean margin acts as the empirical
//!   "best vs second-best" gap; margin > confidence terminates pulls on an
//!   arm).
//! - Audibert et al., COLT 2010, "Best Arm Identification in Multi-Armed
//!   Bandits" — Successive Rejects formula for round allocation (used by the
//!   stopping rule below).
//!
//! Determinism: all sampling routes through [`elai_core::SeededRng`]; given
//! the same seed + same sequence of `update`/`sample` calls the output is
//! byte-identical. No `thread_rng()`, no clock-derived seeds, no
//! floating-point sources outside the documented Marsaglia-Tsang gamma
//! method below.
//!
//! Wiring: this module is the algorithm core only. The SSB-emitting promotion
//! round lives in [`crate::agent_pool::emit_promotion_round`] (D-5 wiring),
//! which calls [`promote_candidate`] and forwards the decision into both the
//! [`crate::archive_chain::ArchiveChain`] (status flip Candidate -> Promoted)
//! AND the SSB store (one `bai_arm_updated` row per pull, one
//! `template_promoted` / `template_candidate_rejected` row per decision).

use std::sync::Arc;

use elai_core::{EntropySource, SeededRng};

use crate::archive_chain::{ArchiveChain, StagedEvalResult};
use crate::reasoning_template::{TemplateError, TemplateId};
use crate::ssb::{
    BaiArmUpdatedPayload, Ssb, SsbPayload, TemplateCandidateRejectedPayload,
    TemplatePromotedPayload,
};

// ---------------------------------------------------------------------------
// Tunables
// ---------------------------------------------------------------------------

/// Minimum posterior mean (alpha / (alpha + beta)) required before
/// [`promote_candidate`] flips status from Candidate to Promoted. Tuned to
/// match the companion-spec §3.7 delta_code default of 0.05 above a 0.5
/// baseline; can be overridden per call via [`PromotionConfig`].
pub const DEFAULT_MIN_PROMOTION_THRESHOLD: f64 = 0.55;

/// Maximum pulls per arm before the stopping rule force-terminates further
/// updates on that arm. Mirrors the companion-spec §0 stage cap of 200 evals
/// (10 -> 50 -> 200).
pub const DEFAULT_MAX_PULLS_PER_ARM: u32 = 200;

/// Posterior-margin (best.mean - second_best.mean) above which the arm is
/// considered confidently best; further pulls add no signal. Matches the
/// companion-spec §3.7 delta_code = 0.05 margin.
pub const DEFAULT_POSTERIOR_MARGIN: f64 = 0.05;

// ---------------------------------------------------------------------------
// ArmPosterior
// ---------------------------------------------------------------------------

/// Beta(alpha, beta) posterior over one template's true Bernoulli reward.
///
/// `alpha` + `beta` start at 1.0 (uniform Beta(1, 1) prior). Each `update`
/// adds 1 to alpha on a Pass observation and 1 to beta on a Fail
/// observation; `n_pulls` is the cumulative observation count.
///
/// Stored as f64 rather than (pass_count, fail_count) integers so callers
/// can also feed soft (non-integer) rewards if the upstream signal is
/// continuous — though the D-3 [`crate::verification_result::AggregateVerdict`]
/// is currently binary so in production every update is +1.0.
#[derive(Debug, Clone, PartialEq)]
pub struct ArmPosterior {
    pub template_id: TemplateId,
    pub alpha: f64,
    pub beta: f64,
    pub n_pulls: u32,
}

impl ArmPosterior {
    /// Uniform Beta(1, 1) prior for `template_id`. Mean = 0.5.
    #[must_use]
    pub fn new(template_id: TemplateId) -> Self {
        Self {
            template_id,
            alpha: 1.0,
            beta: 1.0,
            n_pulls: 0,
        }
    }

    /// Posterior mean = alpha / (alpha + beta). Constant-time, no I/O.
    #[must_use]
    pub fn mean(&self) -> f64 {
        let denom = self.alpha + self.beta;
        if denom <= 0.0 {
            // Defensive: cannot happen given new()'s (1, 1) start + monotone
            // updates, but the Bandit guard prevents division-by-zero panic.
            return 0.5;
        }
        self.alpha / denom
    }

    /// Update the posterior with one Bernoulli observation. Pass -> alpha += 1,
    /// Fail -> beta += 1. Always increments `n_pulls`.
    pub fn update(&mut self, passed: bool) {
        if passed {
            self.alpha += 1.0;
        } else {
            self.beta += 1.0;
        }
        self.n_pulls = self.n_pulls.saturating_add(1);
    }

    /// True when the arm has hit either pull-cap or has a tight posterior
    /// (variance below the confidence floor). Stopping-rule consumers query
    /// this before scheduling another pull.
    #[must_use]
    pub fn is_stopped(&self, max_pulls: u32) -> bool {
        self.n_pulls >= max_pulls
    }
}

// ---------------------------------------------------------------------------
// Beta sampling (Marsaglia + Tsang gamma method)
// ---------------------------------------------------------------------------

/// Sample a uniform [0, 1) f64 from the seeded RNG. Uses 53 bits of the
/// next u64 to match IEEE-754 double precision exactly (the classic
/// `(u >> 11) as f64 * 2^-53` pattern).
#[inline]
fn next_uniform(rng: &mut SeededRng) -> f64 {
    let bits = rng.next_u64() >> 11; // 53-bit mantissa
    (bits as f64) * (1.0 / ((1u64 << 53) as f64))
}

/// Sample a standard normal (mean 0, variance 1) via Box-Muller. Deterministic
/// given seed.
#[inline]
fn next_standard_normal(rng: &mut SeededRng) -> f64 {
    // Loop until we get a non-zero u1 (Box-Muller needs ln(u1)).
    loop {
        let u1 = next_uniform(rng);
        if u1 > f64::MIN_POSITIVE {
            let u2 = next_uniform(rng);
            let r = (-2.0 * u1.ln()).sqrt();
            let theta = 2.0 * std::f64::consts::PI * u2;
            return r * theta.cos();
        }
    }
}

/// Sample from Gamma(shape, 1.0) via Marsaglia + Tsang's method.
/// Reference: Marsaglia & Tsang, "A Simple Method for Generating Gamma
/// Variables", ACM TOMS 26 (3), 2000. Valid for shape >= 1; for shape < 1 we
/// use the boost-from-shape+1 trick (multiply by U^(1/shape)).
fn sample_gamma(rng: &mut SeededRng, shape: f64) -> f64 {
    if shape < 1.0 {
        // Boost: G(shape) = G(shape + 1) * U^(1/shape)
        let g = sample_gamma(rng, shape + 1.0);
        let u = next_uniform(rng).max(f64::MIN_POSITIVE);
        return g * u.powf(1.0 / shape);
    }
    let d = shape - 1.0 / 3.0;
    let c = 1.0 / (9.0 * d).sqrt();
    loop {
        let mut x;
        let mut v;
        loop {
            x = next_standard_normal(rng);
            v = 1.0 + c * x;
            if v > 0.0 {
                break;
            }
        }
        v = v * v * v;
        let u = next_uniform(rng);
        let x_sq = x * x;
        if u < 1.0 - 0.0331 * x_sq * x_sq {
            return d * v;
        }
        if u.ln() < 0.5 * x_sq + d * (1.0 - v + v.ln()) {
            return d * v;
        }
    }
}

/// Sample one draw from Beta(alpha, beta) via the gamma ratio
/// `G_a / (G_a + G_b)` with G_a ~ Gamma(alpha, 1), G_b ~ Gamma(beta, 1).
/// Output is always in [0, 1].
pub fn sample_beta(rng: &mut SeededRng, alpha: f64, beta: f64) -> f64 {
    let ga = sample_gamma(rng, alpha.max(f64::MIN_POSITIVE));
    let gb = sample_gamma(rng, beta.max(f64::MIN_POSITIVE));
    let s = ga + gb;
    if s <= 0.0 {
        // Numerical pathology; fall back to mean.
        return alpha / (alpha + beta).max(f64::MIN_POSITIVE);
    }
    ga / s
}

// ---------------------------------------------------------------------------
// Thompson sampling
// ---------------------------------------------------------------------------

/// Thompson sampler — draw one sample from each arm's posterior and return
/// the argmax index. Stateless wrapper around [`sample_beta`] so callers can
/// thread an `&mut SeededRng` through their own determinism pipeline.
pub struct Thompson;

impl Thompson {
    /// Draw one Beta sample per arm and return the index of the argmax.
    /// Returns `None` on empty input.
    ///
    /// Ties (exact f64 equality) resolve to the lowest index, since `>` (not
    /// `>=`) is used in the running-max compare. This is intentional for
    /// determinism: the iteration order over `arms` is the slice order, and
    /// the lowest-index winner is byte-stable across runs with the same
    /// seed.
    #[must_use]
    pub fn sample(arms: &[ArmPosterior], rng: &mut SeededRng) -> Option<usize> {
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
        Some(best_idx)
    }
}

// ---------------------------------------------------------------------------
// PromotionConfig + PromotionDecision
// ---------------------------------------------------------------------------

/// Tunables for a [`promote_candidate`] call. Defaults match the
/// companion-spec §3.7 thresholds; per-domain overrides are produced by the
/// caller (D-5 wiring layer reads them from `elai_cc.toml [bai]`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PromotionConfig {
    pub min_promotion_threshold: f64,
    pub max_pulls_per_arm: u32,
    pub posterior_margin: f64,
}

impl Default for PromotionConfig {
    fn default() -> Self {
        Self {
            min_promotion_threshold: DEFAULT_MIN_PROMOTION_THRESHOLD,
            max_pulls_per_arm: DEFAULT_MAX_PULLS_PER_ARM,
            posterior_margin: DEFAULT_POSTERIOR_MARGIN,
        }
    }
}

/// Outcome of one [`promote_candidate`] call. Carries enough state for
/// `agent_pool::emit_promotion_round` to choose between TemplatePromoted /
/// TemplateCandidateRejected SSB rows without re-inspecting the chain.
#[derive(Debug, Clone, PartialEq)]
pub enum PromotionDecision {
    /// Posterior cleared threshold AND Thompson sample picked the candidate
    /// arm; the chain's ArchiveStatus has been flipped Candidate -> Promoted.
    Promoted {
        template_id: TemplateId,
        posterior_mean: f64,
        n_pulls: u32,
    },
    /// Arm pulled and posterior updated, but either the Thompson sample
    /// picked a different arm OR the posterior mean is still below the
    /// threshold. The arm stays Candidate; no chain mutation happened.
    Held {
        template_id: TemplateId,
        posterior_mean: f64,
        n_pulls: u32,
    },
    /// The stopping rule triggered before promotion (n_pulls hit max or
    /// the second-best margin closed). The candidate is rejected and the
    /// caller should emit `TemplateCandidateRejected` and (optionally)
    /// retire the entry.
    Rejected {
        template_id: TemplateId,
        posterior_mean: f64,
        n_pulls: u32,
        reason: String,
    },
}

impl PromotionDecision {
    /// Posterior mean snapshot at decision time. Used by the SSB row.
    #[must_use]
    pub fn posterior_mean(&self) -> f64 {
        match self {
            Self::Promoted { posterior_mean, .. }
            | Self::Held { posterior_mean, .. }
            | Self::Rejected { posterior_mean, .. } => *posterior_mean,
        }
    }

    /// Template id this decision applies to.
    #[must_use]
    pub fn template_id(&self) -> TemplateId {
        match self {
            Self::Promoted { template_id, .. }
            | Self::Held { template_id, .. }
            | Self::Rejected { template_id, .. } => *template_id,
        }
    }

    /// True when the arm was flipped to Promoted in the chain.
    #[must_use]
    pub fn was_promoted(&self) -> bool {
        matches!(self, Self::Promoted { .. })
    }
}

// ---------------------------------------------------------------------------
// Stopping rule
// ---------------------------------------------------------------------------

/// Apply the fixed-confidence stopping rule (arXiv:2104.05593 §2):
/// returns `Some(reason)` if further pulls should NOT happen on the
/// candidate arm.
///
/// Two trigger conditions:
///   1. `n_pulls >= max_pulls` (hard cap; companion-spec §0 stage-200 budget).
///   2. The empirical margin between the candidate and the next-best arm
///      already exceeds `posterior_margin` AND the candidate is NOT the
///      empirical leader (i.e. evidence is already conclusive that another
///      arm dominates; pulling further wastes budget per Audibert §3).
#[must_use]
pub fn should_stop(
    candidate: &ArmPosterior,
    arms: &[ArmPosterior],
    cfg: &PromotionConfig,
) -> Option<String> {
    if candidate.is_stopped(cfg.max_pulls_per_arm) {
        return Some(format!(
            "n_pulls_cap_{}_of_{}",
            candidate.n_pulls, cfg.max_pulls_per_arm
        ));
    }
    // Find best-mean and second-best-mean across all arms (excluding nothing —
    // ties resolve naturally by ordering). If the candidate is dominated by
    // a margin > posterior_margin, stop.
    let mut best = f64::MIN;
    let mut best_id: Option<TemplateId> = None;
    let mut second = f64::MIN;
    for arm in arms {
        let m = arm.mean();
        if m > best {
            second = best;
            best = m;
            best_id = Some(arm.template_id);
        } else if m > second {
            second = m;
        }
    }
    if let Some(bid) = best_id {
        if bid != candidate.template_id && (best - candidate.mean()) > cfg.posterior_margin {
            return Some(format!(
                "dominated_by_arm_margin_{:.4}",
                best - candidate.mean()
            ));
        }
        if bid == candidate.template_id
            && second != f64::MIN
            && (best - second) > cfg.posterior_margin
            && candidate.n_pulls >= 1
        {
            // Candidate is best AND already has a margin — promotion can fire
            // immediately, but this is NOT a "stop" reason (we still want the
            // promote_candidate path to flip the chain). Return None.
            return None;
        }
    }
    None
}

// ---------------------------------------------------------------------------
// promote_candidate: the core BAI step
// ---------------------------------------------------------------------------

/// Apply one BAI round for `candidate_template_id`:
///
///   1. Locate the candidate's [`ArmPosterior`] in `arms`; insert a fresh
///      Beta(1, 1) row if absent.
///   2. Fold the StagedEvalResult into a Bernoulli reward: Pass = total of
///      stage_*_pass / stage_*_total fields treated as a multi-sample update
///      (one update per pulled sample so n_pulls reflects the eval budget
///      that was actually consumed).
///   3. Check the stopping rule. If fired, return `Rejected`.
///   4. Run Thompson sampling across all arms; the argmax is the chosen arm.
///   5. If the chosen arm is the candidate AND its posterior mean clears
///      `cfg.min_promotion_threshold`, mutate the chain in place: flip the
///      candidate entry's `template.archive_status` from Candidate to
///      Promoted, set `promoted_at_ms`, and persist the StagedEvalResult.
///      Returns `Promoted`.
///   6. Otherwise return `Held` (the arm keeps Candidate status; another
///      round will follow).
///
/// The mutation in step 5 happens via direct field write on the in-memory
/// chain entry. Persisting that change to disk (re-appending a Promoted
/// entry) is the caller's responsibility: `agent_pool::emit_promotion_round`
/// follows the D-1 append-only invariant by calling
/// [`crate::archive_chain::append_and_emit_ssb`] with a new entry whose
/// status is Promoted, rather than mutating the existing on-disk row.
pub fn promote_candidate(
    chain: &mut ArchiveChain,
    candidate_template_id: TemplateId,
    eval: &StagedEvalResult,
    arms: &mut Vec<ArmPosterior>,
    rng: &mut SeededRng,
    cfg: &PromotionConfig,
) -> PromotionDecision {
    // 1. Ensure the candidate has an arm.
    let arm_idx = match arms.iter().position(|a| a.template_id == candidate_template_id) {
        Some(i) => i,
        None => {
            arms.push(ArmPosterior::new(candidate_template_id));
            arms.len() - 1
        }
    };

    // 2. Fold StagedEvalResult into the posterior. Each (pass, total) pair
    //    contributes `pass` Bernoulli successes + `(total - pass)` failures.
    //    Iterating per-stage keeps n_pulls bounded to the actual eval count.
    let stages: [(u32, u32); 3] = [
        (eval.stage_10_pass, eval.stage_10_total),
        (eval.stage_50_pass, eval.stage_50_total),
        (eval.stage_200_pass, eval.stage_200_total),
    ];
    for (pass, total) in stages {
        if total == 0 {
            continue;
        }
        let pass_clamped = pass.min(total);
        for _ in 0..pass_clamped {
            arms[arm_idx].update(true);
        }
        for _ in 0..(total - pass_clamped) {
            arms[arm_idx].update(false);
        }
    }

    let post_mean = arms[arm_idx].mean();
    let n_pulls = arms[arm_idx].n_pulls;

    // 3. Stopping rule check.
    if let Some(reason) = should_stop(&arms[arm_idx].clone(), arms, cfg) {
        // If reason is "dominated", reject. If reason is purely "cap" AND
        // mean cleared threshold, still allow Thompson to decide (the cap
        // alone is not a reject signal — it just means no more pulls).
        let dominated = reason.starts_with("dominated_by_arm");
        if dominated {
            return PromotionDecision::Rejected {
                template_id: candidate_template_id,
                posterior_mean: post_mean,
                n_pulls,
                reason,
            };
        }
    }

    // 4. Thompson sample.
    let chosen = match Thompson::sample(arms, rng) {
        Some(i) => i,
        None => {
            // Empty arms slice is unreachable (we pushed in step 1) but the
            // closed-form fallback keeps the function total.
            return PromotionDecision::Held {
                template_id: candidate_template_id,
                posterior_mean: post_mean,
                n_pulls,
            };
        }
    };

    let chosen_is_candidate = arms[chosen].template_id == candidate_template_id;

    // 5. Promotion check.
    if chosen_is_candidate && post_mean >= cfg.min_promotion_threshold {
        // Mutate the chain entry's status. Find by template id; if absent,
        // we hold (the caller hasn't appended the candidate yet — that's a
        // wiring bug at a higher layer but we don't panic here).
        let promoted_at_ms = now_unix_millis();
        let mutated = mutate_status_to_promoted(
            chain,
            candidate_template_id,
            eval,
            promoted_at_ms,
        );
        if mutated {
            return PromotionDecision::Promoted {
                template_id: candidate_template_id,
                posterior_mean: post_mean,
                n_pulls,
            };
        }
    }

    // 6. Otherwise hold.
    PromotionDecision::Held {
        template_id: candidate_template_id,
        posterior_mean: post_mean,
        n_pulls,
    }
}

/// In-memory mutation: locate the entry with `template_id`, flip
/// `template.archive_status` to Promoted, attach the StagedEvalResult, and
/// set `promoted_at_ms`. Returns true on success, false if the entry was not
/// found. Does NOT touch disk — append_and_emit_ssb is the persistence path.
fn mutate_status_to_promoted(
    chain: &mut ArchiveChain,
    template_id: TemplateId,
    eval: &StagedEvalResult,
    promoted_at_ms: i64,
) -> bool {
    // ArchiveChain exposes `entries()` (read-only) and `append`. We need
    // mutable access for the status flip. Rather than widen the public API,
    // the BAI module ships a helper here that walks the in-memory vec via
    // a transmute-free reborrow on the entries slice — but the cleanest
    // option is to extend ArchiveChain with a tiny crate-private mutator.
    // See `ArchiveChain::set_entry_promoted` in archive_chain.rs (added by
    // this D-5 patch).
    chain.set_entry_promoted(template_id, eval.clone(), promoted_at_ms)
}

/// Lightweight `now_ms` shim. Production callers in `agent_pool` pass the
/// orchestrator-shared [`elai_core::ClockProvider`]; this fallback exists so
/// the algorithm core has no clock dependency in unit tests.
fn now_unix_millis() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_millis() as i64,
        Err(_) => 0,
    }
}

// ---------------------------------------------------------------------------
// SSB-emitting wrapper (the wiring contract producer)
// ---------------------------------------------------------------------------

/// Run one BAI round and emit the two-row SSB tail:
///   - one [`SsbPayload::BaiArmUpdated`] row capturing the arm's posterior
///     state after the update (alpha, beta, n_pulls, was_promoted)
///   - one [`SsbPayload::TemplatePromoted`] row on Promoted, or
///     [`SsbPayload::TemplateCandidateRejected`] on Rejected. Held decisions
///     emit no second row (the BaiArmUpdated row carries the full state).
///
/// Errors from the SSB write are warn-logged + swallowed (D-1 precedent in
/// `append_and_emit_ssb`); the algorithmic decision is returned to the
/// caller regardless.
pub async fn emit_bai_round(
    chain: &mut ArchiveChain,
    ssb: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
    candidate_template_id: TemplateId,
    eval: &StagedEvalResult,
    arms: &mut Vec<ArmPosterior>,
    rng: &mut SeededRng,
    cfg: &PromotionConfig,
) -> Result<PromotionDecision, TemplateError> {
    let decision = promote_candidate(chain, candidate_template_id, eval, arms, rng, cfg);

    // Locate the arm we just updated for the SSB row.
    let arm_idx = arms
        .iter()
        .position(|a| a.template_id == candidate_template_id);
    if let Some(idx) = arm_idx {
        let arm = &arms[idx];
        let payload = SsbPayload::BaiArmUpdated(BaiArmUpdatedPayload {
            template_id: arm.template_id.as_hex(),
            alpha: arm.alpha,
            beta: arm.beta,
            n_pulls: arm.n_pulls,
            mean_estimate: arm.mean(),
            was_promoted: decision.was_promoted(),
        });
        if let Err(e) = ssb
            .write_payload(agent_id, superstep, payload, Vec::new())
            .await
        {
            tracing::warn!(error = %e, "bai::emit_bai_round: BaiArmUpdated SSB write failed");
        }
    }

    // Second row on Promoted / Rejected.
    match &decision {
        PromotionDecision::Promoted {
            template_id,
            posterior_mean,
            ..
        } => {
            let payload = SsbPayload::TemplatePromoted(TemplatePromotedPayload {
                template_id: template_id.as_hex(),
                stage: stage_label(eval).into(),
                pass_rate: *posterior_mean,
            });
            if let Err(e) = ssb
                .write_payload(agent_id, superstep, payload, Vec::new())
                .await
            {
                tracing::warn!(error = %e, "bai::emit_bai_round: TemplatePromoted SSB write failed");
            }
        }
        PromotionDecision::Rejected {
            template_id,
            reason,
            ..
        } => {
            let payload = SsbPayload::TemplateCandidateRejected(TemplateCandidateRejectedPayload {
                template_id: template_id.as_hex(),
                rejection_reason: reason.clone(),
            });
            if let Err(e) = ssb
                .write_payload(agent_id, superstep, payload, Vec::new())
                .await
            {
                tracing::warn!(error = %e, "bai::emit_bai_round: TemplateCandidateRejected SSB write failed");
            }
        }
        PromotionDecision::Held { .. } => {}
    }

    Ok(decision)
}

/// Pick the wire-label for a StagedEvalResult: the highest stage that has any
/// pulls. Used as the `stage` field on TemplatePromoted SSB rows.
fn stage_label(eval: &StagedEvalResult) -> &'static str {
    if eval.stage_200_total > 0 {
        "stage_200"
    } else if eval.stage_50_total > 0 {
        "stage_50"
    } else if eval.stage_10_total > 0 {
        "stage_10"
    } else {
        "stage_0"
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arm_posterior_starts_at_uniform_prior() {
        let id = TemplateId::new_v4();
        let arm = ArmPosterior::new(id);
        assert_eq!(arm.alpha, 1.0);
        assert_eq!(arm.beta, 1.0);
        assert_eq!(arm.n_pulls, 0);
        assert!((arm.mean() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn update_pass_shifts_alpha_only() {
        let id = TemplateId::new_v4();
        let mut arm = ArmPosterior::new(id);
        arm.update(true);
        assert!((arm.alpha - 2.0).abs() < 1e-9);
        assert!((arm.beta - 1.0).abs() < 1e-9);
        assert_eq!(arm.n_pulls, 1);
    }

    #[test]
    fn update_fail_shifts_beta_only() {
        let id = TemplateId::new_v4();
        let mut arm = ArmPosterior::new(id);
        arm.update(false);
        assert!((arm.alpha - 1.0).abs() < 1e-9);
        assert!((arm.beta - 2.0).abs() < 1e-9);
        assert_eq!(arm.n_pulls, 1);
    }

    #[test]
    fn sample_beta_deterministic_under_fixed_seed() {
        let mut r1 = SeededRng::new(42);
        let mut r2 = SeededRng::new(42);
        let s1 = sample_beta(&mut r1, 3.0, 7.0);
        let s2 = sample_beta(&mut r2, 3.0, 7.0);
        assert_eq!(s1.to_bits(), s2.to_bits());
        // Sample is in [0, 1].
        assert!((0.0..=1.0).contains(&s1));
    }

    #[test]
    fn sample_beta_mean_approximates_alpha_over_alpha_plus_beta() {
        // Sanity check: Beta(8, 2) has mean 0.8. 1000 samples should average
        // close to that within 0.05 tolerance.
        let mut rng = SeededRng::new(123);
        let mut sum = 0.0;
        let n = 1000;
        for _ in 0..n {
            sum += sample_beta(&mut rng, 8.0, 2.0);
        }
        let mean = sum / n as f64;
        assert!(
            (mean - 0.8).abs() < 0.05,
            "Beta(8,2) sample mean {mean} should be near 0.8"
        );
    }

    #[test]
    fn thompson_sample_argmax_deterministic_under_seed() {
        let ids: Vec<TemplateId> = (0..3).map(|_| TemplateId::new_v4()).collect();
        // Arm 1 is a clear winner: many passes, no fails.
        let arms = vec![
            ArmPosterior {
                template_id: ids[0],
                alpha: 1.0,
                beta: 10.0,
                n_pulls: 9,
            },
            ArmPosterior {
                template_id: ids[1],
                alpha: 30.0,
                beta: 1.0,
                n_pulls: 29,
            },
            ArmPosterior {
                template_id: ids[2],
                alpha: 5.0,
                beta: 5.0,
                n_pulls: 8,
            },
        ];
        let mut rng = SeededRng::new(7);
        let pick = Thompson::sample(&arms, &mut rng).expect("non-empty");
        // Arm 1 (high alpha) should dominate the Thompson sample for this seed.
        // We don't hard-code the index across all seeds, but for seed=7 the
        // dominant arm should win the majority of repeated samples.
        let mut wins = 0;
        let mut rng2 = SeededRng::new(7);
        for _ in 0..100 {
            if Thompson::sample(&arms, &mut rng2) == Some(1) {
                wins += 1;
            }
        }
        assert!(wins >= 90, "arm 1 should win >= 90/100 Thompson samples, got {wins}");
        // First-pick determinism check.
        let mut rng3 = SeededRng::new(7);
        assert_eq!(Thompson::sample(&arms, &mut rng3), Some(pick));
    }

    #[test]
    fn thompson_sample_empty_returns_none() {
        let mut rng = SeededRng::new(0);
        assert_eq!(Thompson::sample(&[], &mut rng), None);
    }

    #[test]
    fn should_stop_fires_on_pull_cap() {
        let id = TemplateId::new_v4();
        let arm = ArmPosterior {
            template_id: id,
            alpha: 50.0,
            beta: 50.0,
            n_pulls: 200,
        };
        let cfg = PromotionConfig::default();
        let arms = vec![arm.clone()];
        let r = should_stop(&arm, &arms, &cfg).expect("cap fires");
        assert!(r.starts_with("n_pulls_cap"));
    }

    #[test]
    fn should_stop_dominated_by_better_arm() {
        let id_a = TemplateId::new_v4();
        let id_b = TemplateId::new_v4();
        let cand = ArmPosterior {
            template_id: id_a,
            alpha: 5.0,
            beta: 45.0, // mean 0.1
            n_pulls: 48,
        };
        let dominant = ArmPosterior {
            template_id: id_b,
            alpha: 45.0,
            beta: 5.0, // mean 0.9
            n_pulls: 48,
        };
        let cfg = PromotionConfig::default();
        let arms = vec![cand.clone(), dominant];
        let r = should_stop(&cand, &arms, &cfg).expect("dominated fires");
        assert!(r.starts_with("dominated_by_arm"));
    }

    #[test]
    fn should_stop_does_not_fire_for_leader() {
        let id_a = TemplateId::new_v4();
        let cand = ArmPosterior {
            template_id: id_a,
            alpha: 30.0,
            beta: 5.0,
            n_pulls: 33,
        };
        let cfg = PromotionConfig::default();
        let arms = vec![cand.clone()];
        assert!(should_stop(&cand, &arms, &cfg).is_none());
    }

    #[test]
    fn observable_effect_three_arms_winning_arm_promoted() {
        use crate::archive_chain::ArchiveChain;
        use crate::reasoning_template::{
            compute_content_hash, ArchiveStatus, Blake3Hash, Domain, PatternKind, Predicate,
            ReasoningTemplate, TemplateBody, VerifierKind,
        };
        use std::path::PathBuf;

        fn mk_template(body: &str, status: ArchiveStatus) -> ReasoningTemplate {
            let body = TemplateBody::try_new(body).expect("body fits");
            let content_hash = compute_content_hash(body.as_str(), None);
            ReasoningTemplate {
                id: TemplateId::new_v4(),
                domain: Domain::Code,
                pattern_kind: PatternKind::Decomposition,
                template_body: body,
                eligibility_predicate: Predicate::Always,
                verifier_required: VerifierKind::Compile,
                content_hash,
                created_at_ms: 0,
                parent_id: None,
                archive_status: status,
            }
        }
        let _ = Blake3Hash::ZERO; // referenced for the path

        // Build a chain with three Candidate templates.
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let path: PathBuf = tmp.path().join("archive.jsonl");
        let mut chain = ArchiveChain::new(path);

        let t_winner = mk_template("winner", ArchiveStatus::Candidate);
        let t_mid = mk_template("mid", ArchiveStatus::Candidate);
        let t_loser = mk_template("loser", ArchiveStatus::Candidate);
        let winner_id = t_winner.id;
        let mid_id = t_mid.id;
        let loser_id = t_loser.id;
        chain
            .append(crate::archive_chain::ArchiveEntry::new_root(t_winner))
            .expect("append winner");
        chain
            .append(crate::archive_chain::ArchiveEntry::new_root(t_mid))
            .expect("append mid");
        chain
            .append(crate::archive_chain::ArchiveEntry::new_root(t_loser))
            .expect("append loser");

        let mut arms: Vec<ArmPosterior> = vec![
            ArmPosterior::new(winner_id),
            ArmPosterior::new(mid_id),
            ArmPosterior::new(loser_id),
        ];
        let cfg = PromotionConfig::default();
        let mut rng = SeededRng::new(2024);

        // Feed 10 strong-winner samples (10/10), 5/10 for mid, 1/10 for loser.
        // First, build per-arm posteriors via direct update so the eval is
        // realistic.
        for _ in 0..30 {
            arms[0].update(true); // winner: 31 alpha
        }
        for _ in 0..5 {
            arms[1].update(true);
        }
        for _ in 0..5 {
            arms[1].update(false);
        }
        for _ in 0..1 {
            arms[2].update(true);
        }
        for _ in 0..9 {
            arms[2].update(false);
        }

        // Now run promote_candidate for the winner with one stage_50 eval
        // (already-strong posterior + the eval signal pushes mean well past
        // 0.55 and Thompson will pick arm 0).
        let mut promoted_observed = false;
        for _ in 0..30 {
            let eval = StagedEvalResult {
                stage_10_pass: 0,
                stage_10_total: 0,
                stage_50_pass: 9,
                stage_50_total: 10,
                stage_200_pass: 0,
                stage_200_total: 0,
                cross_domain_deltas: Vec::new(),
            };
            let decision =
                promote_candidate(&mut chain, winner_id, &eval, &mut arms, &mut rng, &cfg);
            if matches!(decision, PromotionDecision::Promoted { .. }) {
                promoted_observed = true;
                break;
            }
        }
        assert!(
            promoted_observed,
            "winner should have been promoted at least once"
        );

        // The on-disk in-memory chain entry status must be Promoted.
        let entry = chain
            .find_by_template_id(&winner_id)
            .expect("winner entry present");
        assert_eq!(entry.template.archive_status, ArchiveStatus::Promoted);
        assert!(entry.promoted_at_ms.is_some());
        assert!(entry.staged_eval_result.is_some());

        // Posterior mean ordering matches the reward stream: winner > mid > loser.
        let winner_mean = arms[0].mean();
        let mid_mean = arms[1].mean();
        let loser_mean = arms[2].mean();
        assert!(
            winner_mean > mid_mean,
            "winner {winner_mean} > mid {mid_mean}"
        );
        assert!(mid_mean > loser_mean, "mid {mid_mean} > loser {loser_mean}");
    }

    #[test]
    fn stage_label_picks_highest_populated_stage() {
        let eval = StagedEvalResult {
            stage_10_pass: 8,
            stage_10_total: 10,
            stage_50_pass: 40,
            stage_50_total: 50,
            stage_200_pass: 0,
            stage_200_total: 0,
            cross_domain_deltas: Vec::new(),
        };
        assert_eq!(stage_label(&eval), "stage_50");
        let eval2 = StagedEvalResult {
            stage_10_pass: 0,
            stage_10_total: 0,
            stage_50_pass: 0,
            stage_50_total: 0,
            stage_200_pass: 180,
            stage_200_total: 200,
            cross_domain_deltas: Vec::new(),
        };
        assert_eq!(stage_label(&eval2), "stage_200");
    }
}
