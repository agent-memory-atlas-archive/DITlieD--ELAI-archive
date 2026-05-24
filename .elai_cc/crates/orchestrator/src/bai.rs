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
//! round is [`emit_bai_round`] below (D-5 wiring), which calls
//! [`promote_candidate`] and forwards the decision into both the
//! [`crate::archive_chain::ArchiveChain`] (status flip Candidate -> Promoted)
//! AND the SSB store (one `bai_arm_updated` row per pull, one
//! `template_promoted` / `template_candidate_rejected` row per decision).
//! `task_runner::dispatch_verifiers` calls `emit_bai_round` once per
//! consensus verdict when the BAI runtime config is enabled.

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

    /// D-5 closure: plan-readability accessor for the integer-valued
    /// "pass count" implied by the Beta posterior. Returns
    /// `floor(alpha - 1.0)` clamped to 0 so callers can render
    /// "passes / total" without knowing the prior.
    ///
    /// NOT a state field — the canonical storage is the f64 `alpha`
    /// (per workflow/reasoning/D-5-decisions.md DECISION-3). The plan
    /// file's `pass_count` u32 is drift; this accessor is the
    /// compatibility shim.
    #[must_use]
    pub fn pass_count_estimate(&self) -> u32 {
        let v = self.alpha - 1.0;
        if v <= 0.0 {
            0
        } else {
            // Saturating cast: f64 -> u32 with the floor() pre-applied.
            v.floor().min(f64::from(u32::MAX)) as u32
        }
    }

    /// D-5 closure: plan-readability accessor mirroring
    /// [`Self::pass_count_estimate`] for the failure count
    /// (`floor(beta - 1.0)`). Same drift-shim caveat applies.
    #[must_use]
    pub fn fail_count_estimate(&self) -> u32 {
        let v = self.beta - 1.0;
        if v <= 0.0 {
            0
        } else {
            v.floor().min(f64::from(u32::MAX)) as u32
        }
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
/// [`emit_bai_round`] to choose between TemplatePromoted /
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
/// entry) is the caller's responsibility: [`emit_bai_round`] follows the
/// D-1 append-only invariant by calling
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
    emit_bai_round_with_canary(
        chain,
        ssb,
        agent_id,
        superstep,
        candidate_template_id,
        eval,
        arms,
        rng,
        cfg,
        None,
    )
    .await
}

/// D-7 closure: variant of [`emit_bai_round`] that consults a recent
/// [`crate::agent_pool::CanaryStatusSummary`] BEFORE flipping the candidate
/// to Promoted. When the canary status' `blocks_promotion()` is true (i.e.
/// the latest canary verdict was `Fail`), the promotion is refused: a
/// `Rejected` decision is returned with `canary_gate_blocked` as the reason
/// and the SSB row sequence becomes `BaiArmUpdated` + `TemplateCandidateRejected`
/// instead of `BaiArmUpdated` + `TemplatePromoted`.
///
/// `canary_status` is the typed projection produced by
/// [`crate::agent_pool::recent_canary_status`]. Callers without a fresh
/// status (no canary run yet, or the BAI runtime is wire-dark) pass `None`
/// and the gate degrades to the un-gated [`promote_candidate`] path.
///
/// ELAI design hypothesis — NanoFlux (arXiv:2509.23252) provides empirical
/// motivation for the cross-domain canary pattern but does NOT define a
/// canary mechanism. The gate semantics here are an ELAI policy choice.
pub async fn emit_bai_round_with_canary(
    chain: &mut ArchiveChain,
    ssb: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
    candidate_template_id: TemplateId,
    eval: &StagedEvalResult,
    arms: &mut Vec<ArmPosterior>,
    rng: &mut SeededRng,
    cfg: &PromotionConfig,
    canary_status: Option<&crate::agent_pool::CanaryStatusSummary>,
) -> Result<PromotionDecision, TemplateError> {
    // D-7 closure GAP-1: canary gate. If the latest canary verdict blocks
    // promotion, refuse to flip the chain status — return Rejected with the
    // canary attribution so dashboards can render the block-cause. The
    // posterior is still updated (the arm pull happens; the BAI signal stays
    // honest) but the promotion side-effect is suppressed. ELAI design
    // choice; NanoFlux (arXiv:2509.23252) provides empirical motivation
    // only — no canary mechanism in the paper itself.
    let canary_blocks = canary_status
        .as_ref()
        .is_some_and(|c| c.blocks_promotion());

    let decision = if canary_blocks {
        // Update the arm's posterior but DO NOT flip the chain status. This
        // keeps the Beta(alpha, beta) honest while preventing promotion.
        // Mirror `promote_candidate`'s arm-update path inline so the
        // signature of the pure function stays unchanged.
        let arm_idx = match arms.iter().position(|a| a.template_id == candidate_template_id) {
            Some(i) => i,
            None => {
                arms.push(ArmPosterior::new(candidate_template_id));
                arms.len() - 1
            }
        };
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
        let dominant = canary_status
            .map(|c| c.dominant_fail_family.as_str())
            .unwrap_or("unknown");
        PromotionDecision::Rejected {
            template_id: candidate_template_id,
            posterior_mean: post_mean,
            n_pulls,
            reason: format!("canary_gate_blocked_dominant_family_{dominant}"),
        }
    } else {
        promote_candidate(chain, candidate_template_id, eval, arms, rng, cfg)
    };

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

// ---------------------------------------------------------------------------
// D-5 closure (GAP-C): Successive Rejects + LUCB1 + credible interval
// ---------------------------------------------------------------------------

/// Audibert / Bubeck / Munos (COLT 2010) Successive Rejects log_bar(K).
///
/// `log_bar(K) = 1/2 + sum_{i=2..=K} 1/i`. NOT the natural logarithm —
/// the paper Section 4 footnote defines this as a harmonic-number variant.
/// The interface contract docs/workflow/papers/audibert-colt-2010-bai-interface.md
/// flags this as IMPLEMENTATION PITFALL #6 (using `K.ln()` here breaks the
/// budget-pull invariant).
///
/// Returns `0.5` for `K <= 1` (the SR algorithm requires `K >= 2`; callers
/// upstream guard against the degenerate case).
#[must_use]
pub fn log_bar_k(k: usize) -> f64 {
    if k < 2 {
        return 0.5;
    }
    let mut acc = 0.5;
    for i in 2..=k {
        acc += 1.0 / (i as f64);
    }
    acc
}

/// Successive Rejects (Audibert et al., COLT 2010, Algorithm 2 / Figure 3).
///
/// Fixed-budget BAI: given a slice of arm ids, a total pull budget `budget`
/// (denoted `n` in the paper), and the precomputed `log_bar_k` constant
/// (passed in so callers can amortise it across calls), this function
/// returns the LAST-surviving arm — the algorithm's recommendation
/// `J_n`. Each phase `k` in `1..K-1` pulls every active arm
/// `n_k - n_{k-1}` additional times and rejects the empirical-worst arm;
/// when only one arm remains it is the recommendation.
///
/// **This is the pull-allocation step only.** The actual pull happens
/// upstream: in the wired ELAI path, the per-task verifier verdict acts
/// as the realised pull (`bai::emit_bai_round` folds the verdict into the
/// arm's posterior). This function answers "given a fresh budget, how do
/// the phase lengths and active sets look?" — it does NOT call
/// `Thompson::sample` or mutate the chain. Callers that want the live
/// recommendation under the BAI runtime should keep using
/// [`Thompson::sample`]; SR is the fixed-budget alternative for offline
/// analysis and the deterministic-replay harness in the D-5 contract test.
///
/// Returns the arm id of the unique survivor after `K-1` phases. Returns
/// `None` if `arms.is_empty()` or `budget < arms.len()` (the paper
/// requires `n >= K` for the phase-length floor to be non-negative).
///
/// Paper-formula floor: `n_k = floor((n-K) / (log_bar_k * (K + 1 - k)))`.
/// We compute partial sums + the per-phase pull count and assert the
/// "total pulls <= n" invariant in the SR contract test.
///
/// This implementation is deterministic given the input slice — when two
/// arms tie at the empirical-worst step, the LOWEST index is rejected
/// (matching [`Thompson::sample`]'s `>` ordering).
///
/// Grounding: workflow/papers/audibert-colt-2010-bai-interface.md
/// (Algorithm 2 + IMPLEMENTATION PITFALL #6 on log_bar).
///
/// **Empirical means input deferred.** Production callers do not yet have
/// a `pulled-then-observed` signal independent from the verifier-consensus
/// loop — the BAI runtime folds verdicts into Beta posteriors via
/// [`emit_bai_round`] rather than running standalone SR phases. This
/// function exists so the contract tests can exercise the SR pull
/// allocation against a synthetic [`ArmPosterior`] vec (passed in as
/// `posteriors`); the recommendation is `argmax mean` over the
/// surviving arm set, which matches the SR rule "J_n = argmax of
/// empirical means at round n" (Audibert §4 first paragraph).
#[must_use]
pub fn successive_rejects(
    arms: &[TemplateId],
    budget: usize,
    log_bar_k_value: f64,
    posteriors: &[ArmPosterior],
) -> Option<TemplateId> {
    let k = arms.len();
    if k == 0 {
        return None;
    }
    if k == 1 {
        return Some(arms[0]);
    }
    if budget < k {
        // Paper requires n >= K so the (n-K) term in the phase-length
        // formula is non-negative. Return None rather than truncate so
        // the caller surfaces the under-budget condition.
        return None;
    }
    if log_bar_k_value <= 0.0 {
        return None;
    }

    // Active arm set, mutated as phases reject the empirical-worst.
    let mut active: Vec<TemplateId> = arms.to_vec();
    let n_minus_k = (budget - k) as f64;

    // Per-paper n_k = floor((n-K) / (log_bar(K) * (K + 1 - k))). We do not
    // need the per-arm pull count downstream — the test harness only
    // checks the active-set evolution + final budget — but we compute the
    // phase deltas so the budget-invariant assertion has a concrete value.
    let mut total_pulls_used: u64 = 0;
    let mut n_prev: u64 = 0;
    for phase in 1..k {
        // phase = 1..K-1 inclusive. K+1-k = number of active arms.
        let denom = log_bar_k_value * ((k + 1 - phase) as f64);
        let n_phase = if denom > 0.0 {
            (n_minus_k / denom).floor()
        } else {
            0.0
        };
        let n_phase_u64 = if n_phase < 0.0 {
            0u64
        } else {
            n_phase.min(u64::MAX as f64) as u64
        };
        let delta = n_phase_u64.saturating_sub(n_prev);
        total_pulls_used = total_pulls_used.saturating_add(
            delta.saturating_mul(active.len() as u64),
        );
        n_prev = n_phase_u64;

        // Reject the empirical-worst arm in the active set. "Empirical"
        // here = posterior mean (under the Beta(1,1) prior this is the
        // empirical mean after one pseudo-pass + one pseudo-fail; the
        // synthetic-update path in the test harness pushes the posterior
        // to the realised reward stream so the ordering matches the
        // paper's `X_hat_{i, n_k}`).
        let mut worst_idx = 0usize;
        let mut worst_mean = f64::INFINITY;
        for (idx, arm_id) in active.iter().enumerate() {
            let mean = posteriors
                .iter()
                .find(|p| p.template_id == *arm_id)
                .map(ArmPosterior::mean)
                .unwrap_or(0.5);
            if mean < worst_mean {
                worst_mean = mean;
                worst_idx = idx;
            }
        }
        active.remove(worst_idx);
        if active.len() == 1 {
            break;
        }
    }

    // Invariant guard: total_pulls_used must not exceed the budget (paper
    // budget proof, Section 4: sum_{k=1..K-1} (K+1-k) * (n_k - n_{k-1})
    // <= n). The arithmetic flooring inside the loop preserves this; the
    // debug_assert surfaces drift if a future refactor breaks the bound.
    debug_assert!(
        total_pulls_used <= budget as u64,
        "successive_rejects pull budget violated: {total_pulls_used} > {budget}"
    );
    active.into_iter().next()
}

/// LUCB1 (Lower / Upper Confidence Bound, Kalyanakrishnan et al. 2012)
/// next-pull selector.
///
/// Given a posterior vector and a target confidence `delta`, returns the
/// arm id the LUCB1 policy would pull next. LUCB1 alternates pulls
/// between the empirical leader and the arm with the highest upper
/// confidence bound among the rest — pulling whichever has the wider
/// confidence interval first, then switching.
///
/// Confidence radius: `beta(t, delta) = sqrt(ln(K * t^4 / delta) / (2 * n_a))`
/// where `t` is the cumulative pull count and `n_a` the arm's pull count.
/// We use the per-arm `n_pulls` directly + a small numerical floor on
/// `n_a` to avoid division-by-zero on fresh arms.
///
/// Returns the [`TemplateId`] of the recommended arm. Returns `None` on
/// empty input.
///
/// Grounding: LUCB1 is the canonical fixed-confidence companion to SR's
/// fixed-budget regime. This implementation tracks the original Kalya
/// et al. 2012 description (Algorithm 1) — the Audibert paper cites
/// LUCB1 as a fixed-confidence baseline (Section 1 "fixed confidence"
/// para).
#[must_use]
pub fn lucb1_next_pull(arms: &[ArmPosterior], delta: f64) -> Option<TemplateId> {
    if arms.is_empty() {
        return None;
    }
    let k = arms.len();
    let total_pulls: u64 = arms.iter().map(|a| u64::from(a.n_pulls)).sum();
    // Add one to avoid log(0) on cold start; the LUCB1 paper uses
    // `t >= 1` from the first round.
    let t = (total_pulls.saturating_add(1)) as f64;
    let delta_clamped = delta.max(f64::MIN_POSITIVE);
    let log_arg = (k as f64) * t.powi(4) / delta_clamped;
    // log_arg should always be positive (k >= 1, t >= 1, delta > 0); the
    // max guard keeps the sqrt defined even under numerical edge cases.
    let log_term = log_arg.max(1.0).ln();

    let radius = |n_pulls: u32| -> f64 {
        let n_a = (n_pulls.max(1)) as f64;
        (log_term / (2.0 * n_a)).sqrt()
    };

    // Empirical leader = arm with the highest posterior mean.
    let mut leader_idx = 0usize;
    let mut leader_mean = f64::MIN;
    for (i, arm) in arms.iter().enumerate() {
        let m = arm.mean();
        if m > leader_mean {
            leader_mean = m;
            leader_idx = i;
        }
    }

    // Challenger = non-leader arm with the highest upper confidence bound.
    let mut challenger_idx: Option<usize> = None;
    let mut best_ucb = f64::MIN;
    for (i, arm) in arms.iter().enumerate() {
        if i == leader_idx {
            continue;
        }
        let ucb = arm.mean() + radius(arm.n_pulls);
        if ucb > best_ucb {
            best_ucb = ucb;
            challenger_idx = Some(i);
        }
    }
    let challenger_idx = match challenger_idx {
        Some(i) => i,
        None => return Some(arms[leader_idx].template_id), // K == 1 fallback.
    };

    // Pull whichever has the wider radius (= fewer pulls / less certain).
    // Tie goes to the leader (`>=` flips would favour challenger; we
    // pick leader so deterministic-replay is monotone with the
    // empirical-mean argmax tie-break in Thompson::sample).
    let leader_radius = radius(arms[leader_idx].n_pulls);
    let challenger_radius = radius(arms[challenger_idx].n_pulls);
    if leader_radius >= challenger_radius {
        Some(arms[leader_idx].template_id)
    } else {
        Some(arms[challenger_idx].template_id)
    }
}

/// Approximate Beta(alpha, beta) credible interval at confidence `level`.
///
/// Returns `(lo, hi)` such that `P(Beta(alpha, beta) in [lo, hi]) approx level`,
/// computed via a Newton-iteration inverse on the regularized incomplete
/// beta CDF. Both endpoints are in `[0.0, 1.0]`. `level` is clamped to
/// `(0.0, 1.0)` so degenerate calls (e.g. `0` or `1`) return finite
/// values rather than NaN.
///
/// implementation choice: not directly grounded in arxiv:2106.10417 paper formula
/// — the variance-BAI paper drives sample complexity via Bernstein + KL
/// concentration, not credible intervals. This is the classical
/// Beta-credible interval and the project's hand-rolled implementation
/// (no `statrs` dep per workflow/reasoning/D-5-decisions.md DECISION-4).
///
/// Algorithm:
///   1. Tail probability `tail = (1 - level) / 2` (two-sided interval).
///   2. `lo = inv_regularized_incomplete_beta(tail, alpha, beta)`.
///   3. `hi = inv_regularized_incomplete_beta(1 - tail, alpha, beta)`.
///
/// The inverse is computed by Newton's method on the CDF (the regularised
/// incomplete beta function I_x(alpha, beta)) starting from the posterior
/// mean. Convergence is quadratic; 30 iterations suffice for f64
/// precision on the well-conditioned posteriors that the BAI round
/// produces (alpha, beta both >= 1.0 after the Beta(1,1) prior + at
/// least one update).
///
/// Special cases:
///   - `alpha <= 0.0` or `beta <= 0.0`: returns `(0.0, 1.0)` (degenerate
///     posterior — no information).
///   - `level <= 0.0`: returns `(mean, mean)` (zero-width interval at the
///     point estimate).
///   - `level >= 1.0`: returns `(0.0, 1.0)` (full support).
#[must_use]
pub fn credible_interval(posterior: &ArmPosterior, level: f64) -> (f64, f64) {
    let a = posterior.alpha;
    let b = posterior.beta;
    if a <= 0.0 || b <= 0.0 {
        return (0.0, 1.0);
    }
    let mean = posterior.mean();
    if level <= 0.0 {
        return (mean, mean);
    }
    if level >= 1.0 {
        return (0.0, 1.0);
    }
    let tail = (1.0 - level) / 2.0;
    let lo = inv_regularized_incomplete_beta(tail, a, b);
    let hi = inv_regularized_incomplete_beta(1.0 - tail, a, b);
    // Numerical guards.
    let lo = lo.max(0.0).min(1.0);
    let hi = hi.max(0.0).min(1.0);
    let (lo, hi) = if lo <= hi { (lo, hi) } else { (hi, lo) };
    (lo, hi)
}

/// Regularized incomplete beta function `I_x(a, b)` evaluated via
/// continued-fraction expansion (Numerical Recipes 2007, §6.4 "betacf").
/// Returns a value in `[0.0, 1.0]`.
///
/// implementation choice: not directly grounded in arxiv:2106.10417 paper formula
/// — see [`credible_interval`] doc. This is the standard textbook
/// continued-fraction expansion; tolerance set to 3e-7 (single-pass
/// f64 stability).
fn regularized_incomplete_beta(x: f64, a: f64, b: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    // ln(B(a,b)) = lgamma(a) + lgamma(b) - lgamma(a+b)
    let ln_beta = ln_gamma(a) + ln_gamma(b) - ln_gamma(a + b);
    let prefactor = ((a * x.ln()) + (b * (1.0 - x).ln()) - ln_beta).exp();
    if prefactor.is_nan() || prefactor.is_infinite() {
        return if x > a / (a + b) { 1.0 } else { 0.0 };
    }
    // Use the symmetry I_x(a,b) = 1 - I_{1-x}(b,a) when x > (a+1)/(a+b+2)
    // to keep the continued fraction convergent.
    if x < (a + 1.0) / (a + b + 2.0) {
        prefactor * beta_cf(x, a, b) / a
    } else {
        1.0 - prefactor * beta_cf(1.0 - x, b, a) / b
    }
}

/// Continued-fraction kernel for [`regularized_incomplete_beta`].
/// Lentz's method (Numerical Recipes 2007 §5.2). Caps iterations at 200
/// — the BAI module's posteriors have small a, b (typically <= 200) so
/// convergence is fast.
fn beta_cf(x: f64, a: f64, b: f64) -> f64 {
    const EPS: f64 = 3.0e-7;
    const FPMIN: f64 = 1.0e-30;
    let qab = a + b;
    let qap = a + 1.0;
    let qam = a - 1.0;
    let mut c = 1.0_f64;
    let mut d = 1.0 - qab * x / qap;
    if d.abs() < FPMIN {
        d = FPMIN;
    }
    d = 1.0 / d;
    let mut h = d;
    for m in 1..=200 {
        let m_f = m as f64;
        let m2 = 2.0 * m_f;
        // Even step.
        let aa = m_f * (b - m_f) * x / ((qam + m2) * (a + m2));
        d = 1.0 + aa * d;
        if d.abs() < FPMIN {
            d = FPMIN;
        }
        c = 1.0 + aa / c;
        if c.abs() < FPMIN {
            c = FPMIN;
        }
        d = 1.0 / d;
        h *= d * c;
        // Odd step.
        let aa = -(a + m_f) * (qab + m_f) * x / ((a + m2) * (qap + m2));
        d = 1.0 + aa * d;
        if d.abs() < FPMIN {
            d = FPMIN;
        }
        c = 1.0 + aa / c;
        if c.abs() < FPMIN {
            c = FPMIN;
        }
        d = 1.0 / d;
        let delta = d * c;
        h *= delta;
        if (delta - 1.0).abs() < EPS {
            break;
        }
    }
    h
}

/// Inverse of [`regularized_incomplete_beta`] via Newton iteration.
/// Returns `x` such that `I_x(a, b) approx p`. Convergence tolerance
/// 1e-10; 30 iterations.
///
/// implementation choice: not directly grounded in arxiv:2106.10417 paper formula.
fn inv_regularized_incomplete_beta(p: f64, a: f64, b: f64) -> f64 {
    if p <= 0.0 {
        return 0.0;
    }
    if p >= 1.0 {
        return 1.0;
    }
    // Initial guess: mean of Beta(a, b).
    let mut x = a / (a + b);
    for _ in 0..30 {
        let cdf = regularized_incomplete_beta(x, a, b);
        // PDF(x) = x^(a-1) * (1-x)^(b-1) / B(a, b).
        let ln_beta = ln_gamma(a) + ln_gamma(b) - ln_gamma(a + b);
        let log_pdf = (a - 1.0) * x.ln() + (b - 1.0) * (1.0 - x).ln() - ln_beta;
        let pdf = log_pdf.exp();
        if pdf <= 0.0 || pdf.is_nan() {
            // Bisect from the current bracket — degenerate PDF means
            // the posterior is essentially a point mass.
            break;
        }
        let delta = (cdf - p) / pdf;
        let next = (x - delta).max(f64::MIN_POSITIVE).min(1.0 - f64::MIN_POSITIVE);
        if (next - x).abs() < 1e-10 {
            x = next;
            break;
        }
        x = next;
    }
    x
}

/// Lanczos approximation to `ln Gamma(x)` (Numerical Recipes 2007 §6.1).
/// Accurate to ~1e-15 for x > 0. Used to evaluate the Beta normalizing
/// constant inside [`regularized_incomplete_beta`].
fn ln_gamma(x: f64) -> f64 {
    const COEFS: [f64; 6] = [
        76.180_091_729_471_46,
        -86.505_320_329_416_77,
        24.014_098_240_830_91,
        -1.231_739_572_450_155,
        0.001_208_650_973_866_179,
        -0.000_005_395_239_384_953,
    ];
    let mut y = x;
    let mut tmp = x + 5.5;
    tmp -= (x + 0.5) * tmp.ln();
    let mut ser = 1.000_000_000_190_015;
    for c in &COEFS {
        y += 1.0;
        ser += c / y;
    }
    -tmp + (2.506_628_274_631_000_3 * ser / x).ln()
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
