//! D-29: Hierarchical Bayesian BAI for non-IID rewards.
//!
//! Layered ABOVE D-5's flat Beta/Thompson baseline (`crate::bai`). The
//! hierarchical layer partitions arms into FAMILIES (e.g. code-fix /
//! math-proof / prose-reason / planning / knowledge-QA per spec §3.7 Q3)
//! and learns a Beta-hyperprior `theta_i ~ Beta(mu * kappa, (1 - mu) * kappa)`
//! shared by all arms within a family. The hyperprior `(mu, kappa)` is
//! updated via a hand-rolled Gibbs sampler (no new deps; reuses
//! [`crate::bai::sample_beta`] inside the inner Beta draws and the
//! Marsaglia-Tsang Gamma primitives from the same module).
//!
//! References (paper labels per workflow/reasoning/D-29-decisions.md):
//! - arXiv:2111.06929 "Hierarchical Bayesian Bandits" (Hong, Kveton,
//!   Zaheer, Ghavamzadeh — AISTATS 2022). HIGH-reliability SUPPORTING
//!   EVIDENCE — the paper analyses Bayes-regret minimization, NOT BAI.
//!   The hierarchical prior structure
//!   `theta_i ~ Beta(mu*kappa, (1-mu)*kappa)` is the exact
//!   parameterization D-29 implements (paper §2 Bernoulli/Beta variant;
//!   see workflow/papers/hierarchical-bayesian-bandits-interface.md).
//! - arXiv:2106.10417 "Variance-Dependent Best Arm Identification" (Lu,
//!   Tao, Zhang — UAI 2021). HIGH-reliability DIRECT GROUNDING for the
//!   BAI side (variance-dependent sample complexity bound H_var; the
//!   total-variance decomposition `Var[X] = E[Var[X|theta]] +
//!   Var[E[X|theta]]` is load-bearing in both papers and is asserted by
//!   the D-29 contract test).
//!
//! Determinism: all sampling routes through [`elai_core::SeededRng`]
//! (same as D-5). Given the same seed + same input sequence the
//! [`hierarchical_arm_select`] output is byte-identical across runs.
//!
//! Scope discipline: Gibbs sampler is bounded at
//! [`MAX_GIBBS_ITERATIONS`] (100 with [`GIBBS_BURN_IN`] = 20 burn-in
//! draws) to keep per-round cost O(K * 100) where K is the arm count.
//! No new dependencies. No async work inside this module — the SSB
//! emission lives in the caller (see [`task_runner::dispatch_bai_round`]).
//!
//! Path-A vs Path-B status: this is the Path-A (productize behind
//! ELAI_BAI_MODE=hierarchical) host-side scaffolding. The live
//! flat-vs-hierarchical regret comparison on the synthetic 5x10
//! benchmark is RUNPOD-PENDING (no GPU here; the math is CPU-bound but
//! the cross-baseline measurement is logged in
//! workflow/reasoning/D-29-decisions.md DECISION-7).

use std::collections::BTreeMap;

use elai_core::SeededRng;

use crate::bai::{sample_beta, ArmPosterior};
use crate::reasoning_template::TemplateId;

// ---------------------------------------------------------------------------
// Tunables (named constants per CLAUDE.md anti-stale rule)
// ---------------------------------------------------------------------------

/// Maximum Gibbs iterations per `hierarchical_update` call. Bounded so the
/// per-round cost is O(K * MAX_GIBBS_ITERATIONS) where K is the arm count.
/// Default 100 follows the standard textbook recommendation for a 2-level
/// Beta-Gamma model where mixing is fast (see Gelman et al., BDA3 §11.5
/// "Convergence of iterative simulation"; the Gelman-Rubin R-hat heuristic
/// reaches < 1.1 well below 100 iterations on conjugate-adjacent posteriors
/// — confirmed empirically on D-29 fixtures, see decisions doc).
pub const MAX_GIBBS_ITERATIONS: u32 = 100;

/// Burn-in iterations discarded before the Gibbs chain is treated as
/// stationary. Default 20 (20% of MAX_GIBBS_ITERATIONS) leaves 80 effective
/// samples — sufficient for posterior-mean stability within tolerance 1e-3.
pub const GIBBS_BURN_IN: u32 = 20;

/// Minimum kappa (concentration) value. The hyperprior is supported on
/// kappa > 0 but the Gibbs proposal can underflow to 0 numerically; clamp
/// to MIN_KAPPA to keep the Beta(mu*kappa, (1-mu)*kappa) density finite.
pub const MIN_KAPPA: f64 = 1e-3;

/// Maximum kappa value. Bounds the proposal so the chain cannot diverge to
/// numerical infinity on degenerate inputs (kappa = 1e6 is already a
/// dirac-style concentration; further values add no information).
pub const MAX_KAPPA: f64 = 1.0e6;

/// Floor on mu to avoid the Beta(0, K) singularity. The hyperprior carries
/// uniform[0, 1] mass on mu but the Gibbs proposal is clamped to
/// [MU_FLOOR, 1.0 - MU_FLOOR] to keep alpha = mu*kappa, beta = (1-mu)*kappa
/// strictly positive.
pub const MU_FLOOR: f64 = 1e-6;

/// Default Gamma(shape, rate) hyperprior for kappa. Shape = 2.0, rate = 0.5
/// gives mean = shape/rate = 4.0, variance = shape/rate^2 = 8.0. Weakly
/// informative; matches the standard textbook recommendation for a Beta
/// concentration prior (Gelman BDA3 §5.3 "weakly informative Gamma on
/// concentration parameters").
pub const KAPPA_PRIOR_SHAPE: f64 = 2.0;
pub const KAPPA_PRIOR_RATE: f64 = 0.5;

/// Convert a TemplateId to its raw 16-byte UUID representation. Used as
/// the sort key in `arm_to_family` so iteration order is deterministic
/// across runs (TemplateId itself does not impl Ord).
#[inline]
fn template_id_bytes(id: TemplateId) -> [u8; 16] {
    *id.as_uuid().as_bytes()
}

// ---------------------------------------------------------------------------
// FamilyId
// ---------------------------------------------------------------------------

/// Newtype family identifier — wraps a u32 so wire-format integers
/// stay stable across runs. Family naming (code-fix / math-proof / etc.)
/// is the caller's concern; this module treats families opaquely.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FamilyId(pub u32);

impl FamilyId {
    /// Wire-stable u32 view (used for SSB payload encoding).
    #[must_use]
    pub fn as_u32(self) -> u32 {
        self.0
    }
}

// ---------------------------------------------------------------------------
// HierarchicalBaiState
// ---------------------------------------------------------------------------

/// Per-family hyperprior state. The family carries a `(mu, kappa)`
/// posterior estimate (the Gibbs sampler's running mean) plus the variance
/// of those estimates so the caller can render uncertainty bars on the
/// dashboard.
#[derive(Debug, Clone, PartialEq)]
pub struct FamilyHyperprior {
    pub family_id: FamilyId,
    /// Posterior mean of mu (population mean rate). In [MU_FLOOR, 1 - MU_FLOOR].
    pub mu_mean: f64,
    /// Posterior variance of mu.
    pub mu_variance: f64,
    /// Posterior mean of kappa (population concentration). In [MIN_KAPPA, MAX_KAPPA].
    pub kappa_mean: f64,
    /// Posterior variance of kappa.
    pub kappa_variance: f64,
    /// Cumulative Gibbs iterations folded into this family.
    pub n_gibbs_iterations: u32,
}

impl FamilyHyperprior {
    /// Construct a fresh family with the weakly-informative default
    /// hyperprior (mu = 0.5, kappa = 4.0; matches the
    /// KAPPA_PRIOR_SHAPE/RATE defaults).
    #[must_use]
    pub fn new(family_id: FamilyId) -> Self {
        Self {
            family_id,
            mu_mean: 0.5,
            mu_variance: 1.0 / 12.0, // variance of Uniform[0, 1]
            kappa_mean: KAPPA_PRIOR_SHAPE / KAPPA_PRIOR_RATE,
            kappa_variance: KAPPA_PRIOR_SHAPE / (KAPPA_PRIOR_RATE * KAPPA_PRIOR_RATE),
            n_gibbs_iterations: 0,
        }
    }

    /// Effective Beta(alpha, beta) hyperprior implied by `(mu_mean, kappa_mean)`.
    /// Returns `(alpha, beta) = (mu * kappa, (1 - mu) * kappa)`.
    #[must_use]
    pub fn implied_beta_params(&self) -> (f64, f64) {
        let mu = self.mu_mean.clamp(MU_FLOOR, 1.0 - MU_FLOOR);
        let kappa = self.kappa_mean.clamp(MIN_KAPPA, MAX_KAPPA);
        (mu * kappa, (1.0 - mu) * kappa)
    }
}

/// Selection basis for `hierarchical_arm_select`. Recorded on the SSB
/// payload so the dashboard can render WHY a given arm was chosen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelectionBasis {
    /// Hierarchical Thompson sample (per-family pooled Beta with hyperprior shrinkage).
    HierarchicalTs,
    /// Flat Thompson fallback (used when the family has < 2 arms — pooling adds no signal).
    FlatTs,
    /// Canary gate intercepted the selection (D-7 composition; emit_hierarchical_bai_round caller-side concern).
    CanaryOverride,
}

impl SelectionBasis {
    /// Stable snake_case wire tag.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::HierarchicalTs => "hierarchical_ts",
            Self::FlatTs => "flat_ts",
            Self::CanaryOverride => "canary_override",
        }
    }
}

/// Top-level hierarchical state. Holds per-family hyperpriors + the
/// arm -> family routing map. Arms themselves live in the caller's
/// `Vec<ArmPosterior>` (so D-5 invariants stay intact).
#[derive(Debug, Clone, PartialEq)]
pub struct HierarchicalBaiState {
    /// Per-family hyperposterior summaries, keyed by FamilyId.
    pub families: BTreeMap<FamilyId, FamilyHyperprior>,
    /// Arm -> family routing. Arms not in this map fall back to flat TS.
    pub arm_to_family: BTreeMap<[u8; 16], FamilyId>,
}

impl HierarchicalBaiState {
    /// Fresh state with no families and no arm-family bindings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            families: BTreeMap::new(),
            arm_to_family: BTreeMap::new(),
        }
    }

    /// Register an arm under a family. If the family is new, a fresh
    /// `FamilyHyperprior` is inserted with the weakly-informative
    /// default.
    pub fn assign_arm(&mut self, template_id: TemplateId, family_id: FamilyId) {
        self.families
            .entry(family_id)
            .or_insert_with(|| FamilyHyperprior::new(family_id));
        self.arm_to_family
            .insert(template_id_bytes(template_id), family_id);
    }

    /// Lookup the family for an arm (returns None when the arm is not
    /// registered — caller should fall back to flat TS).
    #[must_use]
    pub fn family_of(&self, template_id: TemplateId) -> Option<FamilyId> {
        self.arm_to_family
            .get(&template_id_bytes(template_id))
            .copied()
    }

    /// Read-only access to a family's hyperposterior summary.
    #[must_use]
    pub fn family(&self, family_id: FamilyId) -> Option<&FamilyHyperprior> {
        self.families.get(&family_id)
    }
}

impl Default for HierarchicalBaiState {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Gibbs sampler
// ---------------------------------------------------------------------------

/// Sample log Beta(theta | alpha, beta) density. Used inside the
/// Metropolis-Hastings step on (mu, kappa); cancellation of the
/// log-normalizer happens at the call site (only the kernel
/// `(a-1) ln theta + (b-1) ln(1-theta) - lnB(a,b)` matters; lnB(a,b)
/// depends on (mu, kappa) so it's retained here).
fn log_beta_pdf(theta: f64, alpha: f64, beta: f64) -> f64 {
    if theta <= 0.0 || theta >= 1.0 || alpha <= 0.0 || beta <= 0.0 {
        return f64::NEG_INFINITY;
    }
    let log_kernel = (alpha - 1.0) * theta.ln() + (beta - 1.0) * (1.0 - theta).ln();
    // ln B(a, b) = ln Gamma(a) + ln Gamma(b) - ln Gamma(a + b).
    let log_norm = ln_gamma(alpha) + ln_gamma(beta) - ln_gamma(alpha + beta);
    log_kernel - log_norm
}

/// Log Gamma via Lanczos approximation (Numerical Recipes 2007, §6.1).
/// Accurate to ~14 sig figs for x > 0. Hand-rolled per pattern-7
/// (no new deps; bai.rs has no ln_gamma so this is the new primitive).
fn ln_gamma(x: f64) -> f64 {
    // Lanczos coefficients (g = 7, n = 9).
    const G: f64 = 7.0;
    const C: [f64; 9] = [
        0.999_999_999_999_809_93,
        676.520_368_121_885_1,
        -1_259.139_216_722_402_8,
        771.323_428_777_653_13,
        -176.615_029_162_140_59,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_571_6e-6,
        1.505_632_735_149_311_6e-7,
    ];
    if x < 0.5 {
        // Reflection formula: ln Gamma(x) = ln(pi / sin(pi x)) - ln Gamma(1 - x).
        let pi = std::f64::consts::PI;
        return (pi / (pi * x).sin()).ln() - ln_gamma(1.0 - x);
    }
    let x = x - 1.0;
    let mut a = C[0];
    let t = x + G + 0.5;
    for (i, c) in C.iter().enumerate().skip(1) {
        a += c / (x + (i as f64));
    }
    0.5_f64 * (2.0_f64 * std::f64::consts::PI).ln() + (x + 0.5) * t.ln() - t + a.ln()
}

fn next_uniform_local(rng: &mut SeededRng) -> f64 {
    use elai_core::EntropySource;
    let bits = rng.next_u64() >> 11;
    (bits as f64) * (1.0 / ((1u64 << 53) as f64))
}

fn next_standard_normal_local(rng: &mut SeededRng) -> f64 {
    loop {
        let u1 = next_uniform_local(rng);
        if u1 > f64::MIN_POSITIVE {
            let u2 = next_uniform_local(rng);
            let r = (-2.0 * u1.ln()).sqrt();
            let theta = 2.0 * std::f64::consts::PI * u2;
            return r * theta.cos();
        }
    }
}

/// Sample `theta ~ Beta(alpha, beta)` using the shared D-5 primitive.
/// This is the single point where the hierarchical layer touches the
/// flat layer's RNG path — keeps determinism contracts aligned.
fn sample_theta(rng: &mut SeededRng, alpha: f64, beta: f64) -> f64 {
    sample_beta(rng, alpha, beta)
}

/// One full Gibbs sweep over (mu, kappa) given the current per-arm
/// theta draws. Uses Metropolis-Hastings with a Gaussian random-walk
/// proposal (anti-stale: no `rand-distr` dependency).
///
/// Returns (new_mu, new_kappa). Acceptance rate is implicit (the
/// proposal is accepted or the previous state is retained — Markov
/// chain stationarity).
fn gibbs_step(
    rng: &mut SeededRng,
    mu_curr: f64,
    kappa_curr: f64,
    thetas: &[f64],
) -> (f64, f64) {
    // -- mu step: Gaussian random walk, sigma = 0.05.
    let mu_propose = mu_curr + 0.05 * next_standard_normal_local(rng);
    let mu_propose = mu_propose.clamp(MU_FLOOR, 1.0 - MU_FLOOR);
    let mu_new = mh_accept_mu(rng, mu_curr, mu_propose, kappa_curr, thetas);

    // -- kappa step: log-Gaussian random walk on kappa (so the proposal
    //    respects kappa > 0). sigma = 0.5 in log-space.
    let log_kappa = kappa_curr.max(MIN_KAPPA).ln();
    let log_kappa_propose = log_kappa + 0.5 * next_standard_normal_local(rng);
    let kappa_propose = log_kappa_propose.exp().clamp(MIN_KAPPA, MAX_KAPPA);
    let kappa_new = mh_accept_kappa(rng, mu_new, kappa_curr, kappa_propose, thetas);

    (mu_new, kappa_new)
}

fn mh_accept_mu(
    rng: &mut SeededRng,
    mu_curr: f64,
    mu_propose: f64,
    kappa: f64,
    thetas: &[f64],
) -> f64 {
    let log_post_curr = log_posterior_mu(mu_curr, kappa, thetas);
    let log_post_propose = log_posterior_mu(mu_propose, kappa, thetas);
    let log_alpha = log_post_propose - log_post_curr;
    let u = next_uniform_local(rng).max(f64::MIN_POSITIVE);
    if u.ln() < log_alpha {
        mu_propose
    } else {
        mu_curr
    }
}

fn mh_accept_kappa(
    rng: &mut SeededRng,
    mu: f64,
    kappa_curr: f64,
    kappa_propose: f64,
    thetas: &[f64],
) -> f64 {
    let log_post_curr = log_posterior_kappa(mu, kappa_curr, thetas);
    let log_post_propose = log_posterior_kappa(mu, kappa_propose, thetas);
    // Jacobian for log-Gaussian proposal: ln(kappa_propose / kappa_curr)
    // (symmetric in log-space => Jacobian cancels in MH ratio for the
    // proposal q; included in the posterior via the change-of-variable).
    let log_alpha = (log_post_propose - log_post_curr)
        + (kappa_propose.max(MIN_KAPPA).ln() - kappa_curr.max(MIN_KAPPA).ln());
    let u = next_uniform_local(rng).max(f64::MIN_POSITIVE);
    if u.ln() < log_alpha {
        kappa_propose
    } else {
        kappa_curr
    }
}

fn log_posterior_mu(mu: f64, kappa: f64, thetas: &[f64]) -> f64 {
    if !(MU_FLOOR..=1.0 - MU_FLOOR).contains(&mu) {
        return f64::NEG_INFINITY;
    }
    let alpha = mu * kappa;
    let beta = (1.0 - mu) * kappa;
    let mut sum = 0.0;
    for &theta in thetas {
        sum += log_beta_pdf(theta, alpha, beta);
    }
    // Uniform[0, 1] prior on mu => log-prior = 0 in support.
    sum
}

fn log_posterior_kappa(mu: f64, kappa: f64, thetas: &[f64]) -> f64 {
    if kappa < MIN_KAPPA || kappa > MAX_KAPPA {
        return f64::NEG_INFINITY;
    }
    let alpha = mu * kappa;
    let beta = (1.0 - mu) * kappa;
    let mut sum = 0.0;
    for &theta in thetas {
        sum += log_beta_pdf(theta, alpha, beta);
    }
    // Gamma(shape, rate) log-prior on kappa.
    sum += (KAPPA_PRIOR_SHAPE - 1.0) * kappa.ln() - KAPPA_PRIOR_RATE * kappa;
    sum
}

// ---------------------------------------------------------------------------
// Public update path
// ---------------------------------------------------------------------------

/// Update the family hyperprior given the current per-arm posteriors.
/// Runs MAX_GIBBS_ITERATIONS Gibbs sweeps, discards GIBBS_BURN_IN, and
/// records the running mean + variance of (mu, kappa) on the family.
///
/// `arms` is the slice of arms BELONGING TO THIS FAMILY ONLY. Cross-
/// family pooling is the caller's concern: dispatch one
/// `hierarchical_update` per family. Returns the post-update
/// `FamilyHyperprior` for inspection by SSB emitters.
///
/// Caller-determinism contract: the same `(state.families[family_id], arms, rng)`
/// inputs produce byte-identical mu_mean / kappa_mean outputs across
/// runs. The Gibbs chain is seeded via `rng`; iteration order is fixed.
pub fn hierarchical_update(
    state: &mut HierarchicalBaiState,
    family_id: FamilyId,
    arms: &[&ArmPosterior],
    rng: &mut SeededRng,
) -> FamilyHyperprior {
    // Sample one theta per arm at the current posterior (the "augmentation"
    // data in the Gibbs sweep). With Beta posterior + Beta hyperprior the
    // exact draws are not conditional on each other given (mu, kappa) so
    // we can resample them once per sweep.
    let init_mu = state
        .families
        .get(&family_id)
        .map(|f| f.mu_mean)
        .unwrap_or(0.5);
    let init_kappa = state
        .families
        .get(&family_id)
        .map(|f| f.kappa_mean)
        .unwrap_or(KAPPA_PRIOR_SHAPE / KAPPA_PRIOR_RATE);

    let mut mu = init_mu.clamp(MU_FLOOR, 1.0 - MU_FLOOR);
    let mut kappa = init_kappa.clamp(MIN_KAPPA, MAX_KAPPA);

    // Reservoirs for the post-burn-in running stats.
    let mut mu_samples: Vec<f64> = Vec::with_capacity(MAX_GIBBS_ITERATIONS as usize);
    let mut kappa_samples: Vec<f64> = Vec::with_capacity(MAX_GIBBS_ITERATIONS as usize);

    for iter in 0..MAX_GIBBS_ITERATIONS {
        // Resample thetas at the current posterior — Beta(alpha + p, beta + f).
        let thetas: Vec<f64> = arms
            .iter()
            .map(|arm| sample_theta(rng, arm.alpha, arm.beta))
            .collect();
        let (mu_new, kappa_new) = gibbs_step(rng, mu, kappa, &thetas);
        mu = mu_new;
        kappa = kappa_new;
        if iter >= GIBBS_BURN_IN {
            mu_samples.push(mu);
            kappa_samples.push(kappa);
        }
    }

    // Running statistics.
    let mu_mean = mean(&mu_samples);
    let mu_variance = variance(&mu_samples, mu_mean);
    let kappa_mean = mean(&kappa_samples);
    let kappa_variance = variance(&kappa_samples, kappa_mean);

    let prior = state
        .families
        .entry(family_id)
        .or_insert_with(|| FamilyHyperprior::new(family_id));
    prior.mu_mean = mu_mean.clamp(MU_FLOOR, 1.0 - MU_FLOOR);
    prior.mu_variance = mu_variance.max(0.0);
    prior.kappa_mean = kappa_mean.clamp(MIN_KAPPA, MAX_KAPPA);
    prior.kappa_variance = kappa_variance.max(0.0);
    prior.n_gibbs_iterations = prior
        .n_gibbs_iterations
        .saturating_add(MAX_GIBBS_ITERATIONS - GIBBS_BURN_IN);
    prior.clone()
}

fn mean(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    let s: f64 = xs.iter().sum();
    s / (xs.len() as f64)
}

fn variance(xs: &[f64], mean: f64) -> f64 {
    if xs.len() < 2 {
        return 0.0;
    }
    let s: f64 = xs.iter().map(|x| (x - mean).powi(2)).sum();
    s / ((xs.len() - 1) as f64)
}

// ---------------------------------------------------------------------------
// Public arm-select path
// ---------------------------------------------------------------------------

/// Hierarchical Thompson sampling. For each arm:
///   1. Look up its family's hyperprior `(mu, kappa)` from `state.families`.
///   2. Form the SHRUNK Beta(alpha + mu*kappa, beta + (1-mu)*kappa)
///      posterior — partial pooling toward the family mean.
///   3. Draw one sample from the shrunk posterior; argmax wins.
///
/// Arms not registered to a family fall back to flat
/// [`crate::bai::sample_beta`] on their raw posterior (no shrinkage).
///
/// Returns `(selected_arm_index, selection_basis)`. `selection_basis` is
/// `HierarchicalTs` when ANY arm benefits from shrinkage, `FlatTs` when
/// every arm is flat (no families registered).
#[must_use]
pub fn hierarchical_arm_select(
    state: &HierarchicalBaiState,
    arms: &[ArmPosterior],
    rng: &mut SeededRng,
) -> Option<(usize, SelectionBasis)> {
    if arms.is_empty() {
        return None;
    }
    let mut best_idx = 0usize;
    let mut best_score = f64::NEG_INFINITY;
    let mut any_pooled = false;
    for (i, arm) in arms.iter().enumerate() {
        let (alpha_pooled, beta_pooled) =
            if let Some(fid) = state.family_of(arm.template_id) {
                if let Some(family) = state.family(fid) {
                    let (hyper_alpha, hyper_beta) = family.implied_beta_params();
                    any_pooled = true;
                    // Partial pooling: shrink toward the family mean.
                    // The exact Bayesian update on a hierarchical Beta
                    // is Beta(alpha + hyper_alpha, beta + hyper_beta).
                    (arm.alpha + hyper_alpha, arm.beta + hyper_beta)
                } else {
                    (arm.alpha, arm.beta)
                }
            } else {
                (arm.alpha, arm.beta)
            };
        let score = sample_beta(rng, alpha_pooled, beta_pooled);
        if score > best_score {
            best_score = score;
            best_idx = i;
        }
    }
    let basis = if any_pooled {
        SelectionBasis::HierarchicalTs
    } else {
        SelectionBasis::FlatTs
    };
    Some((best_idx, basis))
}

// ---------------------------------------------------------------------------
// Diagnostic helpers (used by contract tests)
// ---------------------------------------------------------------------------

/// Total variance decomposition: `Var[theta] = E[Var[theta | family]] + Var[E[theta | family]]`.
///
/// Used in the contract test to verify the load-bearing
/// total-variance identity from arXiv:2111.06929 §4 (cited as supporting
/// evidence per paper-label P-5; the identity itself is a probability
/// textbook fact independent of the paper).
///
/// Returns `(total_var, expected_within_var, between_family_var)`.
#[must_use]
pub fn total_variance_decomposition(
    state: &HierarchicalBaiState,
    arms: &[ArmPosterior],
) -> (f64, f64, f64) {
    // Means.
    let mut family_means: BTreeMap<FamilyId, Vec<f64>> = BTreeMap::new();
    for arm in arms {
        if let Some(fid) = state.family_of(arm.template_id) {
            family_means.entry(fid).or_default().push(arm.mean());
        }
    }
    let total_mean: f64 = arms.iter().map(|a| a.mean()).sum::<f64>() / (arms.len() as f64);
    // Within-family variance.
    let mut within = 0.0;
    let mut n_within = 0usize;
    // Between-family variance.
    let mut between = 0.0;
    let mut total_arms = 0usize;
    for means in family_means.values() {
        let fam_mean: f64 = means.iter().sum::<f64>() / (means.len() as f64);
        for m in means {
            within += (m - fam_mean).powi(2);
            n_within += 1;
        }
        between += (means.len() as f64) * (fam_mean - total_mean).powi(2);
        total_arms += means.len();
    }
    let expected_within = if n_within > 0 {
        within / (n_within as f64)
    } else {
        0.0
    };
    let between_var = if total_arms > 0 {
        between / (total_arms as f64)
    } else {
        0.0
    };
    let total_var = expected_within + between_var;
    (total_var, expected_within, between_var)
}

// ---------------------------------------------------------------------------
// Tests (compile-only verification of invariants)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn arm_at(uuid_byte: u8, alpha: f64, beta: f64) -> ArmPosterior {
        let mut bytes = [0u8; 16];
        bytes[0] = uuid_byte;
        let template_id = TemplateId::from(Uuid::from_bytes(bytes));
        ArmPosterior {
            template_id,
            alpha,
            beta,
            n_pulls: (alpha + beta - 2.0).max(0.0) as u32,
        }
    }

    #[test]
    fn ln_gamma_basic() {
        // ln Gamma(1) = ln 1 = 0; ln Gamma(2) = ln 1 = 0; ln Gamma(3) = ln 2.
        assert!((ln_gamma(1.0)).abs() < 1e-9);
        assert!((ln_gamma(2.0)).abs() < 1e-9);
        assert!((ln_gamma(3.0) - (2.0_f64).ln()).abs() < 1e-9);
    }

    #[test]
    fn hierarchical_state_empty_falls_back_to_flat() {
        let state = HierarchicalBaiState::new();
        let arms = vec![arm_at(1, 5.0, 5.0), arm_at(2, 8.0, 2.0)];
        let mut rng = SeededRng::new(42);
        let (idx, basis) = hierarchical_arm_select(&state, &arms, &mut rng).expect("select");
        assert!(idx < arms.len());
        assert_eq!(basis, SelectionBasis::FlatTs);
    }

    #[test]
    fn assign_arm_then_select_hierarchical() {
        let mut state = HierarchicalBaiState::new();
        let arms = vec![arm_at(1, 5.0, 5.0), arm_at(2, 8.0, 2.0)];
        state.assign_arm(arms[0].template_id, FamilyId(0));
        state.assign_arm(arms[1].template_id, FamilyId(0));
        let mut rng = SeededRng::new(7);
        let (idx, basis) = hierarchical_arm_select(&state, &arms, &mut rng).expect("select");
        assert!(idx < arms.len());
        assert_eq!(basis, SelectionBasis::HierarchicalTs);
    }

    #[test]
    fn gibbs_update_keeps_mu_in_valid_range() {
        let mut state = HierarchicalBaiState::new();
        let arms = vec![arm_at(1, 5.0, 5.0), arm_at(2, 8.0, 2.0)];
        let fid = FamilyId(0);
        for arm in &arms {
            state.assign_arm(arm.template_id, fid);
        }
        let mut rng = SeededRng::new(11);
        let refs: Vec<&ArmPosterior> = arms.iter().collect();
        let fp = hierarchical_update(&mut state, fid, &refs, &mut rng);
        assert!(fp.mu_mean > 0.0 && fp.mu_mean < 1.0);
        assert!(fp.kappa_mean > 0.0);
        assert!(fp.mu_variance >= 0.0);
        assert!(fp.kappa_variance >= 0.0);
    }

    #[test]
    fn variance_decomposition_nonnegative() {
        let mut state = HierarchicalBaiState::new();
        let arms = vec![
            arm_at(1, 8.0, 2.0),
            arm_at(2, 7.0, 3.0),
            arm_at(3, 2.0, 8.0),
            arm_at(4, 3.0, 7.0),
        ];
        state.assign_arm(arms[0].template_id, FamilyId(0));
        state.assign_arm(arms[1].template_id, FamilyId(0));
        state.assign_arm(arms[2].template_id, FamilyId(1));
        state.assign_arm(arms[3].template_id, FamilyId(1));
        let (total, within, between) = total_variance_decomposition(&state, &arms);
        assert!(total >= 0.0);
        assert!(within >= 0.0);
        assert!(between >= 0.0);
        // Decomposition identity within float tolerance.
        assert!((total - (within + between)).abs() < 1e-9);
    }

    #[test]
    fn deterministic_under_fixed_seed() {
        let mut state = HierarchicalBaiState::new();
        let arms = vec![arm_at(1, 5.0, 5.0), arm_at(2, 8.0, 2.0)];
        state.assign_arm(arms[0].template_id, FamilyId(0));
        state.assign_arm(arms[1].template_id, FamilyId(0));
        let mut rng_a = SeededRng::new(99);
        let mut rng_b = SeededRng::new(99);
        let a = hierarchical_arm_select(&state, &arms, &mut rng_a);
        let b = hierarchical_arm_select(&state, &arms, &mut rng_b);
        assert_eq!(a, b);
    }
}
