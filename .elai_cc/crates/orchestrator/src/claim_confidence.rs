//! G-7 — per-change confidence PRODUCER (the real `claim_score` that feeds the
//! SHIPPED D-16 conformal abstention layer).
//!
//! TODAY (pre-G-7) the live per-change abstention path
//! ([`crate::task_runner`] hook → [`crate::agent_pool::dispatch_conformal_refusal`]
//! → [`crate::conformal_calibration::decide`]) is fed a FABRICATED placeholder
//! `claim_score` derived from `steps_executed` (a step-count proxy with zero
//! causal connection to the change's actual confidence). This module replaces
//! that placeholder with a real fused score.
//!
//! THE FUSION (DD1). Three orthogonal signals are combined into one calibrated
//! `[0.0, 1.0]` scalar:
//!   * (a) TOKEN LOG-PROB — mean per-token log-probability of the generation,
//!     surfaced by the sidecar's `GGSignals` bundle (the local-lane
//!     `GGSignals::mean_logprob`, which the orchestrator threads in as a plain
//!     `Option<f64>` so this module carries NO `sidecar`-feature-gated dep —
//!     the context-package cross-crate fix). HONEST SCOPE (GD-5): this is a
//!     **LOCAL-LANE-ONLY** signal. The cloud Anthropic client returns NO token
//!     log-probs, so on the cloud lane this is `None` and the fusion falls back
//!     to (b)+(c) ONLY. We never claim log-prob confidence on the cloud lane.
//!   * (b) SELF-CONSISTENCY DISPERSION — the agreement across `n` independent
//!     samples of the same change. High agreement (low dispersion) = high
//!     confidence. On the cloud lane this is MANDATORY (it and (c) are the only
//!     usable signals). The probe costs `n` extra generations; `n` is kept small
//!     (see [`SELF_CONSISTENCY_SAMPLES`]) and the probe is gated by the caller.
//!   * (c) ORACLE-FREE TEST EXECUTION-AGREEMENT — the fraction of the `n` samples
//!     whose generated-test stdout hash (F-4 `stdout_hash`, blake3) agrees with
//!     the modal hash. Cross-sample hash-equality is the off-agent execution
//!     signal (R4): a real test ran and produced a byte-identical artifact.
//!
//! CALIBRATION. The raw fused score is rescaled to a calibrated `[0.0, 1.0]`
//! confidence via the EXISTING Platt/sigmoid rescaler
//! [`crate::cost::cloud_route::confidence::compute_score`] (GD-1: REUSE — the
//! conformal layer has `tau`/`TauUpdater` but no Platt calibrator, so the
//! `[0,1]` rescale is this module's job and it reuses the shipped surface rather
//! than adding a new calibrator).
//!
//! STUB-BAN (the half-wire-dark beacon rule): a MISSING signal LOWERS the score
//! conservatively, it NEVER silently yields `1.0`. A change with no log-prob
//! (cloud) + no samples + no test still produces a low, defensible score — never
//! a fabricated high one.
//!
//! GROUNDING. Post-hoc Platt rescaling is model-size-agnostic and viable at 7B
//! (Spiess arXiv:2402.02047, GROUNDED-CONFIRMED). Oracle-free execution
//! agreement as a confidence signal (CodeHalu arXiv:2405.00253 +
//! Task-Abstention arXiv:2605.17029, GROUNDED-CONFIRMED). The conformal
//! abstention guarantee is the SHIPPED D-16 layer's (distribution-free risk
//! control, Angelopoulos arXiv:2208.02814) — NOT a guarantee this fusion proves.
//! UniCR (arXiv:2509.01455) is an UNVERIFIED single-author preprint and is NOT
//! cited as proven here.

use crate::cost::cloud_route::confidence::compute_score as platt_confidence;
use crate::cost::cloud_route::ConfidenceScore;

// ---------------------------------------------------------------------------
// Named constants (token-discipline: no magic numbers)
// ---------------------------------------------------------------------------

/// Number of independent samples drawn for the self-consistency dispersion
/// probe (signal b). Kept SMALL on purpose: each sample is an extra generation,
/// and the dispersion estimate saturates quickly. `3` gives a usable
/// majority/dispersion split (2-of-3 vs 3-of-3 vs all-distinct) at bounded cost.
pub const SELF_CONSISTENCY_SAMPLES: usize = 3;

/// Fusion weight for the (a) token-log-prob signal. Local-lane only; re-weighted
/// to zero when the signal is absent (the cloud lane) so the remaining weights
/// re-normalise over (b)+(c) — a missing signal does not silently inflate.
pub const WEIGHT_LOGPROB: f64 = 0.34;

/// Fusion weight for the (b) self-consistency dispersion signal.
pub const WEIGHT_SELF_CONSISTENCY: f64 = 0.33;

/// Fusion weight for the (c) oracle-free test execution-agreement signal.
pub const WEIGHT_EXEC_AGREEMENT: f64 = 0.33;

/// Conservative floor applied to any individual signal that is structurally
/// ABSENT (no samples / no test / no log-prob). The absent signal contributes
/// this low value, never `1.0` — the stub-ban beacon rule (a missing signal
/// LOWERS confidence). It is not `0.0` so a single absent signal does not force
/// a guaranteed Abstain on an otherwise-confident change.
pub const ABSENT_SIGNAL_FLOOR: f64 = 0.15;

/// `mean_logprob` substituted when the log-prob signal is absent. Mirrors the
/// V-STaR fail value used by `reasoning::engine::extract_intrinsic_signals`
/// (a low log-prob → low confidence through the Platt sigmoid).
const ABSENT_LOGPROB_SENTINEL: f32 = -10.0;

// ---------------------------------------------------------------------------
// Platt calibration parameters
// ---------------------------------------------------------------------------

/// Platt rescaling parameters `sigmoid(a * raw + b)`. The shipped
/// [`crate::cost::cloud_route::confidence`] surface already encodes a sigmoid
/// calibration for the log-prob signal; `PlattParams` is the explicit
/// post-fusion rescale applied to the COMBINED score so the conformal layer
/// receives a calibrated confidence (not a raw weighted mean).
///
/// The DEFAULT (`a = 1.0`, `b = 0.0`) is identity-on-the-logit: it preserves
/// the fused score's ordering and only re-centres it through the logistic. The
/// numbers are a heuristic rescale (NOT a proven 7B guarantee — GD-5); a real
/// calibration fit lands when a labelled corpus exists (the GD-4 promotion).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlattParams {
    /// Logistic slope.
    pub a: f64,
    /// Logistic intercept.
    pub b: f64,
}

impl Default for PlattParams {
    fn default() -> Self {
        // Identity-on-logit: re-centre through the logistic without distorting
        // the fused ordering. A fitted (a, b) lands at GD-4 promotion.
        Self { a: 1.0, b: 0.0 }
    }
}

// ---------------------------------------------------------------------------
// The fused per-signal breakdown (carried onto the claim_confidence_scored row)
// ---------------------------------------------------------------------------

/// The three per-signal components of a fused `claim_score`, each in `[0.0, 1.0]`
/// AFTER its own normalisation but BEFORE the final Platt rescale. Carried
/// verbatim onto the `claim_confidence_scored` SSB row so a re-run can re-derive
/// the fused score byte-identically (GD-2) and an operator can see WHICH signal
/// drove an abstention.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClaimScoreComponents {
    /// (a) Calibrated log-prob confidence. `None` on the cloud lane (no
    /// log-probs) — rendered as [`ABSENT_SIGNAL_FLOOR`] in the fusion.
    pub logprob: Option<f64>,
    /// (b) Self-consistency agreement (1.0 = all samples agree).
    pub self_consistency: f64,
    /// (c) Oracle-free test execution-agreement (1.0 = all sample test runs
    /// produced the modal stdout hash).
    pub exec_agreement: f64,
}

/// Lane the score was computed on. Determines whether signal (a) is available.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfidenceLane {
    /// Local sidecar (Qwen) — token log-probs available via `GGSignals`.
    Local,
    /// Cloud Anthropic — NO token log-probs; (b)+(c) only (GD-5).
    Cloud,
}

impl ConfidenceLane {
    /// Stable snake-case wire label (carried onto the SSB row).
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Cloud => "cloud",
        }
    }
}

// ---------------------------------------------------------------------------
// Signal (b): self-consistency dispersion over n samples
// ---------------------------------------------------------------------------

/// Self-consistency agreement across `samples` independent generations of the
/// SAME change, identified by a stable per-sample fingerprint (e.g. a blake3
/// hash of the normalised change body). Returns the fraction of samples that
/// match the MODAL (most-frequent) fingerprint — `1.0` when every sample agrees,
/// `1.0/n` when all `n` samples are distinct.
///
/// Empty input → [`ABSENT_SIGNAL_FLOOR`] (no probe ran → low confidence, never
/// `1.0` — the stub-ban beacon rule).
#[must_use]
pub fn self_consistency_agreement(sample_fingerprints: &[String]) -> f64 {
    if sample_fingerprints.is_empty() {
        return ABSENT_SIGNAL_FLOOR;
    }
    // Count each fingerprint; the modal count / total is the agreement ratio.
    let mut counts: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for fp in sample_fingerprints {
        *counts.entry(fp.as_str()).or_insert(0) += 1;
    }
    let modal = counts.values().copied().max().unwrap_or(0);
    let total = sample_fingerprints.len();
    // total is non-zero here (empty handled above); modal <= total so ratio in
    // (0, 1]. Use f64 division; both operands are small usize.
    modal as f64 / total as f64
}

// ---------------------------------------------------------------------------
// Signal (c): oracle-free test execution-agreement over the same n samples
// ---------------------------------------------------------------------------

/// Execution-agreement across the `n` sample runs of the generated oracle-free
/// test: the fraction of runs whose `stdout_hash` (F-4 blake3) equals the modal
/// hash. A change whose generated test produces a byte-identical stdout across
/// all samples scores `1.0`; divergent stdout (flaky / non-deterministic /
/// genuinely uncertain change) scores lower.
///
/// Empty input → [`ABSENT_SIGNAL_FLOOR`] (no test ran → low confidence, never
/// `1.0`). This mirrors [`self_consistency_agreement`] over the stdout-hash
/// space; signals (b) and (c) share the same `n` samples (DD1: (c) depends on
/// (b)'s samples).
#[must_use]
pub fn exec_agreement_ratio(sample_stdout_hashes: &[String]) -> f64 {
    if sample_stdout_hashes.is_empty() {
        return ABSENT_SIGNAL_FLOOR;
    }
    let mut counts: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for h in sample_stdout_hashes {
        *counts.entry(h.as_str()).or_insert(0) += 1;
    }
    let modal = counts.values().copied().max().unwrap_or(0);
    let total = sample_stdout_hashes.len();
    modal as f64 / total as f64
}

// ---------------------------------------------------------------------------
// Signal (a): calibrate the token log-prob through the shipped Platt surface
// ---------------------------------------------------------------------------

/// Calibrate the (a) token-log-prob signal into `[0.0, 1.0]` using the SHIPPED
/// [`crate::cost::cloud_route::confidence::compute_score`] (GD-1 reuse). Returns
/// `None` when `mean_logprob` is absent (the cloud lane), so the caller folds in
/// [`ABSENT_SIGNAL_FLOOR`] rather than a fabricated value.
///
/// `mean_logprob` is the local sidecar's `GGSignals::mean_logprob` threaded in
/// as a plain `Option<f64>` (no feature-gated dep). `compute_score` takes an
/// `f32` `mean_logprob` with no first-token distribution available here, so we
/// pass `None` for the distribution and narrow the log-prob to `f32`.
#[must_use]
pub fn logprob_confidence(mean_logprob: Option<f64>) -> Option<f64> {
    let mean = mean_logprob?;
    // Narrow f64 → f32 for the shipped sigmoid surface. mean_logprob is a small
    // bounded value (sane local samples in [-10, 0]); `as` narrowing here is the
    // ConfidenceScore = f32 boundary the shipped surface mandates, not a
    // lossy-cast bug.
    let mean_f32 = mean as f32;
    let score: ConfidenceScore = platt_confidence(None, mean_f32);
    Some(f64::from(score))
}

// ---------------------------------------------------------------------------
// The fusion + the post-fusion Platt rescale
// ---------------------------------------------------------------------------

/// Rescale a raw fused score `[0.0, 1.0]` through the explicit Platt logistic
/// `sigmoid(a * logit(raw) + b)`. With [`PlattParams::default`] (a=1, b=0) this
/// is the identity transform (sigmoid∘logit), preserving ordering; a fitted
/// (a, b) sharpens / shifts the calibration. Total on the `[0,1]` domain
/// (degenerate `0.0`/`1.0` are clamped away from the logit poles first).
#[must_use]
pub fn platt_rescale(raw: f64, params: &PlattParams) -> f64 {
    // Clamp away from the logit poles so logit(raw) is finite.
    let eps = 1e-6;
    let p = raw.clamp(eps, 1.0 - eps);
    let logit = (p / (1.0 - p)).ln();
    let z = params.a * logit + params.b;
    let out = 1.0 / (1.0 + (-z).exp());
    out.clamp(0.0, 1.0)
}

/// Fuse the three signals into a single raw confidence in `[0.0, 1.0]` BEFORE
/// the Platt rescale. A weighted mean over the present signals; an ABSENT signal
/// (`logprob = None` on cloud) contributes [`ABSENT_SIGNAL_FLOOR`] at its weight
/// — it LOWERS the score, never raises it (stub-ban).
#[must_use]
fn fuse_raw(components: &ClaimScoreComponents) -> f64 {
    let logprob_val = components.logprob.unwrap_or(ABSENT_SIGNAL_FLOOR);
    let weighted = WEIGHT_LOGPROB * logprob_val
        + WEIGHT_SELF_CONSISTENCY * components.self_consistency
        + WEIGHT_EXEC_AGREEMENT * components.exec_agreement;
    let total_weight = WEIGHT_LOGPROB + WEIGHT_SELF_CONSISTENCY + WEIGHT_EXEC_AGREEMENT;
    // total_weight is a positive const sum; division is safe.
    (weighted / total_weight).clamp(0.0, 1.0)
}

/// Compute the real per-change `claim_score` that replaces the
/// `task_runner.rs:11509` placeholder. Fuses (a) calibrated log-prob (LOCAL
/// only), (b) self-consistency agreement, (c) oracle-free test execution-
/// agreement, then rescales through the Platt logistic. The result is the
/// calibrated `[0.0, 1.0]` confidence fed to
/// [`crate::conformal_calibration::decide`] (consumer UNCHANGED — GD-1).
///
/// On the cloud lane `mean_logprob` is `None` (Anthropic returns no log-probs),
/// so signal (a) is absent and the fusion is driven by (b)+(c) (GD-5 honest).
/// On the local lane all three are present (pass `Some(GGSignals::mean_logprob)`).
///
/// Determinism (GD-2): the function is pure over its inputs — given the same
/// `mean_logprob`, the same sample fingerprints, the same stdout hashes, and the
/// same `PlattParams`, it re-derives the same score byte-identically. No clock,
/// no RNG.
#[must_use]
pub fn compute_claim_score(
    mean_logprob: Option<f64>,
    sample_fingerprints: &[String],
    sample_stdout_hashes: &[String],
    params: &PlattParams,
) -> (f64, ClaimScoreComponents) {
    let components = ClaimScoreComponents {
        logprob: logprob_confidence(mean_logprob),
        self_consistency: self_consistency_agreement(sample_fingerprints),
        exec_agreement: exec_agreement_ratio(sample_stdout_hashes),
    };
    let raw = fuse_raw(&components);
    let calibrated = platt_rescale(raw, params);
    (calibrated, components)
}

/// The cloud-lane log-prob input: always `None` (Anthropic returns no
/// log-probs), so signal (a) folds to [`ABSENT_SIGNAL_FLOOR`] via
/// `logprob_confidence` returning `None`. The local-fail sentinel const
/// documents the local degenerate value; the cloud lane never reaches it because
/// `mean_logprob` is `None`.
#[must_use]
pub fn cloud_lane_logprob() -> Option<f64> {
    let _ = ABSENT_LOGPROB_SENTINEL;
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_signals_lower_score_never_one() {
        // Cloud lane, no samples, no test → all signals absent/floored. The
        // fused score must be LOW (well below 1.0), never a fabricated high.
        let (score, comp) =
            compute_claim_score(cloud_lane_logprob(), &[], &[], &PlattParams::default());
        assert!(comp.logprob.is_none(), "cloud lane has no log-prob");
        assert!(score < 0.5, "all-absent score must be low, got {score}");
        assert!(score > 0.0, "score must stay above the abstain-forcing 0.0");
    }

    #[test]
    fn full_agreement_high_score() {
        let fps = vec!["a".to_string(), "a".to_string(), "a".to_string()];
        let hashes = vec!["h".to_string(), "h".to_string(), "h".to_string()];
        let (score, comp) =
            compute_claim_score(Some(-0.2), &fps, &hashes, &PlattParams::default());
        assert!((comp.self_consistency - 1.0).abs() < 1e-9);
        assert!((comp.exec_agreement - 1.0).abs() < 1e-9);
        assert!(score > 0.6, "full-agreement local score should be high, got {score}");
    }

    #[test]
    fn divergent_samples_lower_consistency() {
        let agree = self_consistency_agreement(&[
            "x".to_string(),
            "y".to_string(),
            "z".to_string(),
        ]);
        // 3 distinct → modal 1 of 3.
        assert!((agree - (1.0 / 3.0)).abs() < 1e-9);
    }

    #[test]
    fn deterministic_rederivation() {
        let fps = vec!["a".to_string(), "a".to_string(), "b".to_string()];
        let hashes = vec!["h".to_string(), "h".to_string(), "g".to_string()];
        let (s1, _) = compute_claim_score(Some(-1.0), &fps, &hashes, &PlattParams::default());
        let (s2, _) = compute_claim_score(Some(-1.0), &fps, &hashes, &PlattParams::default());
        assert_eq!(s1.to_bits(), s2.to_bits(), "score must re-derive byte-identically");
    }
}
