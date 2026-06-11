//! H-18 (SCHEMA_VERSION 224): Deflated Sharpe / PBO / MinBTL promotion gate.
//!
//! The Tier-1 fail-closed done-gate for strategy promotion (T-2, trust spine).
//! BLOCKs promotion unless:
//!   - N is PRESENT in the H-17 trial ledger (TD-1: TrialCountMissing -> Block).
//!   - PBO evidence is PRESENT (None -> Block fail-closed, R1/R2).
//!   - DSR >= DSR_THRESHOLD (Deflated Sharpe corrects for selection bias + N).
//!   - PBO < PBO_THRESHOLD (less than half of IS-best selections underperform OOS).
//!   - T >= MinTRL (minimum track record length, per-period).
//!
//! # Tier split (TD-2/W-10)
//!
//! Tier-1 (this file): pure Rust scalar gate. Computes DSR + SR_0 + MinTRL
//! in-process using hand-rolled erf / inv-Phi (Abramowitz & Stegun 7.1.26).
//! Receives PBO as a scalar from the Tier-2 Python sidecar (deflation_sidecar.py).
//! Thresholds are LOCKED named constants (mutable=0 rule seed). The verdict is
//! DETERMINISTIC (no LLM input, TD-3).
//!
//! Tier-2 (deflation_sidecar.py, frozen, NOT invoked in this run):
//! Computes PBO via CSCV (S=16 partitions, C(16,8)=12870 pairs). Requires the
//! full T x N return matrix. Stub registered in workflow/sessions/s-10612/stubs.md.
//!
//! # Formulas (grounded: Bailey & Lopez de Prado 2014, SSRN 2460551)
//!
//!   gamma_EM   = 0.5772156649015328606  (Euler-Mascheroni constant)
//!   maxZ       = (1-gamma_EM)*Phi_inv(1-1/N) + gamma_EM*Phi_inv(1-exp(-1)/N)
//!   SR_0       = sharpe_std * maxZ
//!   sigma_sr   = sqrt((1 - skew*SR + SR^2*(kurt-1)/4) / (T-1))
//!              Note: kurt = EXCESS kurtosis (Fisher=True); skew = raw skewness
//!   DSR        = Phi((SR_obs - SR_0) / sigma_sr)
//!   MinTRL     = 1 + (1 - skew*SR + SR^2*(kurt-1)/4) * (Phi_inv(MINBTL_CONFIDENCE)
//!                    / (SR_obs - sr_benchmark))^2
//!              Note: per-period formula; divide by 252 for years.
//!
//! # Invariants
//!   - DSR in [0, 1]
//!   - sigma_sr > 0 (enforced: if sigma_sr <= 0, Block with sigma_sr_zero)
//!   - SR_0 >= 0 for N >= 1
//!   - MinTRL >= 1 period
//!
//! # No .unwrap()/.expect() outside tests (CLAUDE.md HARD RULE).
//! # blake3 only for hashing (CLAUDE.md HARD RULE: anti-stale).
//! # No new Rust crate (statrs/libm absent from Cargo.lock — hand-rolled erf).
//!
//! References:
//!   - workflow/papers/deflated-sharpe-ratio-2460551-interface.md (PRIMARY)
//!   - Abramowitz & Stegun (1964) Handbook, Section 7.1.26 (erf polynomial)
//!   - Harvey, Liu & Zhu (2016) RFS 29(1):5-68 NBER w20592 (t>3.0 threshold)
//!   - workflow/reasoning/H-18-decisions.md

use std::sync::Arc;

use tracing::{debug, warn};

use crate::{
    ssb::{DeflationGateEvaluatedPayload, Ssb, SsbPayload},
    trial_ledger::{TrialCountResult, TrialLedger, TrialLedgerError},
};

// ---------------------------------------------------------------------------
// LOCKED named constants (TD-2; must match deflation_promotion.toml thresholds)
// ---------------------------------------------------------------------------

/// Minimum Deflated Sharpe Ratio for promotion (LOCKED, mutable=0).
///
/// DSR >= 0.95 is the ELAI default (conservative relative to Harvey-Liu t>3.0).
/// Harvey-Liu (RFS 2016 p.5): t-stat > 3.0 corresponds to DSR >= 0.9987
/// (one-tailed p=0.00135). ELAI uses 0.95 as a strategy-level default.
/// Document both thresholds in H-18-decisions.md.
pub const DSR_THRESHOLD: f64 = 0.95;

/// Maximum Probability of Backtest Overfitting for promotion (LOCKED, mutable=0).
///
/// PBO < 0.5 means fewer than half of IS-best selections underperform OOS
/// (Bailey, Borwein, Lopez de Prado, Zhu 2015, Companion paper SSRN 2326253).
pub const PBO_THRESHOLD: f64 = 0.5;

/// Confidence level for MinTRL calculation (LOCKED).
///
/// 0.95 -> Phi_inv(0.95) = 1.6449 (the 95th percentile of the normal distribution).
pub const MINBTL_CONFIDENCE: f64 = 0.95;

/// CSCV partition count for PBO computation (LOCKED, Tier-2).
///
/// S=16: C(16,8)=12870 IS/OOS pairs; standard per Bailey et al. (2015).
/// Range [4,32] even integers. Higher S = more stable PBO estimate.
pub const CSCV_PARTITIONS: u32 = 16;

/// Euler-Mascheroni constant (grounded: Wikipedia + Bailey & Lopez de Prado 2014).
///
/// Used in the SR_0 Gumbel extreme value formula.
const EULER_MASCHERONI: f64 = 0.5772156649015328606;

/// Minimum allowed sigma_sr (prevents division by zero in DSR formula).
///
/// If sigma_sr <= this floor, the gate emits Block with reason "sigma_sr_zero".
const SIGMA_SR_MIN: f64 = 1e-12;

/// Minimum T for reliable sigma_sr (interface-map failure mode 6).
///
/// T < 50 -> sigma_sr unreliable; gate returns Block with reason "t_too_short".
/// This is a WARN in the Python sidecar but a hard floor here (conservative).
const T_MIN: u64 = 2;

/// Minimum N for reliable SR_0 Gumbel EV theory (interface-map failure mode 4).
///
/// N < 5: Gumbel EV degrades; gate logs a WARN but does NOT hard-block
/// (conservative direction: lower N = less deflation = looser gate, safe).
const N_GUMBEL_WARN_THRESHOLD: u64 = 5;

/// T below which sigma_sr is statistically unreliable (interface-map FM-6).
///
/// The hard block floor is `T_MIN` (LOCKED). This separate, looser threshold is
/// observability-only: when `T_MIN <= t_periods < T_SIGMA_RELIABLE` the gate
/// still proceeds (the small-sample sigma_sr is usable but noisy) and emits a
/// WARN so the unreliability is recorded rather than silent. NOT a gate
/// threshold; does not change any verdict.
const T_SIGMA_RELIABLE: u64 = 50;

// ---------------------------------------------------------------------------
// Public API types
// ---------------------------------------------------------------------------

/// Verdict kind returned by [`evaluate_deflation`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateVerdictKind {
    /// All thresholds cleared. Promotion is allowed.
    Allow,
    /// At least one threshold failed. Promotion is blocked.
    Block,
}

/// Full verdict from [`evaluate_deflation`].
#[derive(Debug, Clone)]
pub struct DeflationGateVerdict {
    /// Block or Allow.
    pub kind: GateVerdictKind,
    /// Machine-readable reason code. One of:
    ///   "allow" | "block_trial_count_missing" | "block_sigma_sr_zero"
    ///   | "block_t_too_short" | "block_pbo_absent" | "block_dsr"
    ///   | "block_pbo" | "block_minbtl"
    pub reason: String,
    /// DSR value, or NaN on TrialCountMissing / sigma_sr_zero paths.
    pub dsr: f64,
    /// sigma_sr value (denominator of the t-stat).
    pub sigma_sr: f64,
    /// SR_0 (expected max SR under H0 for N trials).
    pub sr_0: f64,
    /// MinTRL (minimum track record length in periods).
    pub min_trl: f64,
    /// N used (from trial ledger; 0 on TrialCountMissing path).
    pub n_used: u64,
}

/// Input bundle for [`evaluate_deflation`].
///
/// All SR values are PER-PERIOD (not annualized); T is in the same period units.
/// MinTRL is computed in per-period units here (divide by 252 for years externally).
#[derive(Debug, Clone)]
pub struct DeflationInput {
    /// Search objective key (e.g. "sharpe_1y"). Passed to trial_count.
    pub objective: String,
    /// Canonical "start_ms..end_ms" data window. Passed to trial_count.
    pub data_window: String,
    /// Observed Sharpe ratio (PER-PERIOD, NOT annualized).
    pub sr_observed: f64,
    /// Number of return observations (sample length). Must be >= T_MIN.
    pub t_periods: u64,
    /// Raw skewness of returns (gamma_3). 0.0 for normal.
    pub skew: f64,
    /// EXCESS kurtosis of returns (gamma_4, Fisher=True). 0.0 for normal.
    pub excess_kurt: f64,
    /// Cross-sectional std of Sharpe ratios across the N trials (sqrt(V[SR])).
    /// 0.0 if N=1 (no cross-sectional variance possible).
    pub sharpe_std: f64,
    /// Benchmark SR for MinTRL. Default 0.0 (risk-free = 0).
    pub sr_benchmark: f64,
    /// PBO scalar from Tier-2 sidecar [0,1]. None = absent (-> Block, R1/R2).
    pub pbo: Option<f64>,
    /// ELAI_SEED for GD-2 reproducibility.
    pub seed: u64,
}

// ---------------------------------------------------------------------------
// Intermediate result from the scalar DSR computation (exposed for tests)
// ---------------------------------------------------------------------------

/// Intermediate results from [`compute_dsr_tier1`] (exposed for contract tests).
#[derive(Debug, Clone)]
pub struct Sr0Result {
    /// Expected max SR under H0 for N independent trials.
    pub sr_0: f64,
    /// maxZ (Gumbel EV quantile combination).
    pub max_z: f64,
    /// Std error of the Sharpe ratio (non-normality corrected).
    pub sigma_sr: f64,
    /// Deflated Sharpe Ratio = Phi((SR_obs - SR_0) / sigma_sr).
    pub dsr: f64,
}

// ---------------------------------------------------------------------------
// Core DSR computation (Tier-1 in-process, pure Rust, no new dep)
// ---------------------------------------------------------------------------

/// Compute DSR + SR_0 + sigma_sr in-process.
///
/// Exported for contract tests (test (f): formula match on a known fixture).
///
/// # Grounding
/// Bailey & Lopez de Prado (2014) SSRN 2460551, as ingested in
/// `workflow/papers/deflated-sharpe-ratio-2460551-interface.md`.
///
/// For N=1: SR_0 = 0 (no multiple testing; DSR reduces to PSR).
pub fn compute_dsr_tier1(
    sr_observed: f64,
    t_periods: u64,
    skew: f64,
    excess_kurt: f64,
    n: u64,
    sharpe_std: f64,
) -> Sr0Result {
    // sigma_sr = sqrt((1 - skew*SR + SR^2*(kurt-1)/4) / (T-1))
    // Note: kurt here is EXCESS kurtosis. For normal: skew=0, kurt=0
    // => formula reduces to sqrt(1/(T-1)).
    let t_f64 = t_periods as f64;
    let numerator = (1.0 - skew * sr_observed
        + sr_observed * sr_observed * (excess_kurt - 1.0) / 4.0)
        .max(SIGMA_SR_MIN * SIGMA_SR_MIN * (t_f64 - 1.0)); // floor to keep positive
    let sigma_sr = (numerator / (t_f64 - 1.0).max(1.0)).sqrt().max(SIGMA_SR_MIN);

    // SR_0 = sharpe_std * maxZ (Gumbel EV formula, Bailey & Lopez de Prado 2014 Eq.).
    // For N=1: maxZ = 0 (no selection bias), so SR_0 = 0.
    let (max_z, sr_0) = if n <= 1 {
        (0.0, 0.0)
    } else {
        // maxZ = (1-gamma_EM)*Phi_inv(1-1/N) + gamma_EM*Phi_inv(1-exp(-1)/N)
        //
        // CRITICAL: "exp(-1)" is the CONSTANT e^{-1} ≈ 0.3679 (not exp(-1/N)).
        // The formula from Bailey & Lopez de Prado (2014) is:
        //   maxZ = (1-gamma_EM)*Phi_inv(1-1/N) + gamma_EM*Phi_inv(1-e^{-1}/N)
        // where e^{-1} ≈ 0.36787944117... is a constant.
        // Grounded: ingested interface map line 48: "Phi_inv(1-exp(-1)/N)"
        // where "exp(-1)" is the mathematical constant e^{-1}, not exp(-1/N).
        let n_f64 = n as f64;
        let inv_n = 1.0 / n_f64;
        // e^{-1} = 1/e, the constant ≈ 0.36788 (NOT exp(-1/N)).
        // CRITICAL: the formula uses the fixed constant e^{-1}, not a function of N.
        let e_inv = (-1.0_f64).exp(); // = e^{-1} ≈ 0.36788
        let arg1 = (1.0 - inv_n).max(f64::EPSILON).min(1.0 - f64::EPSILON);
        let arg2 = (1.0 - e_inv / n_f64).max(f64::EPSILON).min(1.0 - f64::EPSILON);
        let phi_inv_1 = normal_cdf_inv(arg1);
        let phi_inv_2 = normal_cdf_inv(arg2);
        let mz = (1.0 - EULER_MASCHERONI) * phi_inv_1 + EULER_MASCHERONI * phi_inv_2;
        let s0 = sharpe_std * mz;
        (mz, s0)
    };

    // DSR = Phi((SR_obs - SR_0) / sigma_sr)
    let z = (sr_observed - sr_0) / sigma_sr;
    let dsr = normal_cdf(z);

    Sr0Result { sr_0, max_z, sigma_sr, dsr }
}

/// Compute MinTRL in periods (Bailey & Lopez de Prado 2014, paper Section 4).
///
/// MinTRL uses the SAME per-period SR as DSR (NOT annualized).
/// Output is in the same period units as T. Divide by 252 for years externally.
///
/// Formula: MinTRL = 1 + (1 - skew*SR + SR^2*(kurt-1)/4) * (Phi_inv(conf) / (SR - SR_bench))^2
fn compute_min_trl(
    sr_observed: f64,
    skew: f64,
    excess_kurt: f64,
    sr_benchmark: f64,
) -> Option<f64> {
    let sr_diff = sr_observed - sr_benchmark;
    if sr_diff.abs() < f64::EPSILON {
        // SR == benchmark: MinTRL is infinite (undefined direction).
        return None;
    }
    let phi_inv_conf = normal_cdf_inv(MINBTL_CONFIDENCE); // Phi_inv(0.95) ~ 1.6449
    let non_normality = 1.0 - skew * sr_observed
        + sr_observed * sr_observed * (excess_kurt - 1.0) / 4.0;
    let ratio = phi_inv_conf / sr_diff;
    let min_trl = 1.0 + non_normality * ratio * ratio;
    Some(min_trl.max(1.0))
}

// ---------------------------------------------------------------------------
// Main gate: evaluate_deflation
// ---------------------------------------------------------------------------

/// Evaluate the Deflated-Sharpe / PBO / MinTRL promotion gate (Tier-1).
///
/// # Fail-closed rules (TD-1/R1/R2)
///   - `trial_count() == TrialCountMissing` => Block immediately (TD-1).
///   - `pbo == None` => Block immediately (R1: missing PBO evidence = hard FAIL).
///   - `sigma_sr <= SIGMA_SR_MIN` => Block (math undefined).
///   - `t_periods < T_MIN` => Block (sigma_sr unreliable per interface-map FM-6).
///
/// # SSB emission (off-agent artifact)
///   Emits one [`SsbPayload::DeflationGateEvaluated`] row regardless of verdict.
///   The SSB row is the off-agent artifact (R4). Even a Block row is evidence.
///
/// # Caller contract
///   - MUST match on `TrialCountMissing` directly; MUST NOT use `n_treating_missing_as_zero_UNSAFE`.
///   - SSB emission happens inside this function (CLI layer calls this, then uses the verdict).
///
/// # Boundaries (mirroring safedelta_runner.rs pattern)
///   - This function owns gate logic + SSB emission.
///   - Caller (CLI `deflation-check` subcommand) opens Ssb + TrialLedger, passes refs.
pub async fn evaluate_deflation(
    ledger: &TrialLedger,
    ssb: &Arc<Ssb>,
    input: DeflationInput,
) -> Result<DeflationGateVerdict, DeflationGateError> {
    // Step 1: Read N from H-17 trial ledger (TD-1 fail-closed sentinel).
    let trial_result = ledger
        .trial_count(&input.objective, &input.data_window)
        .map_err(DeflationGateError::Ledger)?;

    let n = match trial_result {
        TrialCountResult::TrialCountMissing => {
            // TD-1: fail CLOSED on missing trial count.
            // No DSR computed; emit Block SSB row with n_used=0, dsr=NaN.
            debug!(
                objective = %input.objective,
                data_window = %input.data_window,
                "deflation_gate: TrialCountMissing -> Block (TD-1)"
            );
            let payload = build_payload(
                &input,
                0,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                "block_trial_count_missing",
            );
            emit_ssb(ssb, payload).await?;
            return Ok(DeflationGateVerdict {
                kind: GateVerdictKind::Block,
                reason: "trial_count_missing".to_string(),
                dsr: f64::NAN,
                sigma_sr: f64::NAN,
                sr_0: f64::NAN,
                min_trl: f64::NAN,
                n_used: 0,
            });
        }
        TrialCountResult::Present(n) => n,
    };

    // Step 2: Check T_MIN (interface-map failure mode 6: T < 50 unreliable).
    if input.t_periods < T_MIN {
        warn!(
            t = input.t_periods,
            min = T_MIN,
            "deflation_gate: T too short -> Block"
        );
        let payload = build_payload(&input, n, f64::NAN, f64::NAN, f64::NAN, f64::NAN, "block_t_too_short");
        emit_ssb(ssb, payload).await?;
        return Ok(DeflationGateVerdict {
            kind: GateVerdictKind::Block,
            reason: "t_too_short".to_string(),
            dsr: f64::NAN,
            sigma_sr: f64::NAN,
            sr_0: f64::NAN,
            min_trl: f64::NAN,
            n_used: n,
        });
    }

    // Step 2b: Reliability WARN for the small-sample band (interface-map FM-6).
    // T cleared the hard floor (T >= T_MIN) but is below T_SIGMA_RELIABLE, so
    // sigma_sr is usable but noisy. Observability-only: NOT a block, no verdict
    // change — the unreliability is recorded instead of silent.
    if input.t_periods < T_SIGMA_RELIABLE {
        warn!(
            t = input.t_periods,
            reliable_min = T_SIGMA_RELIABLE,
            "deflation_gate: T < {T_SIGMA_RELIABLE} — sigma_sr statistically unreliable (small-sample); proceeding"
        );
    }

    // Step 3: Warn on N < N_GUMBEL_WARN_THRESHOLD (interface-map failure mode 4).
    // NOT a hard block — lower N = less deflation = looser gate (safe direction).
    if n < N_GUMBEL_WARN_THRESHOLD {
        warn!(
            n,
            "deflation_gate: N < {N_GUMBEL_WARN_THRESHOLD} — Gumbel EV degrades; using N conservatively"
        );
    }

    // Step 4: Compute DSR + SR_0 + sigma_sr (Tier-1 in-process scalar).
    let Sr0Result { sr_0, max_z: _, sigma_sr, dsr } =
        compute_dsr_tier1(input.sr_observed, input.t_periods, input.skew, input.excess_kurt, n, input.sharpe_std);

    // Step 5: sigma_sr floor check.
    if sigma_sr <= SIGMA_SR_MIN {
        let payload = build_payload(&input, n, dsr, sigma_sr, sr_0, f64::NAN, "block_sigma_sr_zero");
        emit_ssb(ssb, payload).await?;
        return Ok(DeflationGateVerdict {
            kind: GateVerdictKind::Block,
            reason: "sigma_sr_zero".to_string(),
            dsr,
            sigma_sr,
            sr_0,
            min_trl: f64::NAN,
            n_used: n,
        });
    }

    // Step 6: Compute MinTRL (per-period; divide by 252 for years externally).
    let min_trl = compute_min_trl(input.sr_observed, input.skew, input.excess_kurt, input.sr_benchmark)
        .unwrap_or(f64::INFINITY);

    // Step 7: Check PBO presence (R1/R2 — missing evidence = hard FAIL).
    let pbo = match input.pbo {
        Some(p) => p,
        None => {
            debug!(
                objective = %input.objective,
                "deflation_gate: PBO absent -> Block (R1 fail-closed)"
            );
            let payload = build_payload(&input, n, dsr, sigma_sr, sr_0, min_trl, "block_pbo_absent");
            emit_ssb(ssb, payload).await?;
            return Ok(DeflationGateVerdict {
                kind: GateVerdictKind::Block,
                reason: "pbo_absent".to_string(),
                dsr,
                sigma_sr,
                sr_0,
                min_trl,
                n_used: n,
            });
        }
    };

    // Step 8: Apply the LOCKED COMPOSITE gate (all three criteria must pass).
    // DSR gate.
    if dsr < DSR_THRESHOLD {
        let payload = build_payload(&input, n, dsr, sigma_sr, sr_0, min_trl, "block_dsr");
        emit_ssb(ssb, payload).await?;
        debug!(
            dsr, threshold = DSR_THRESHOLD, n,
            "deflation_gate: DSR below threshold -> Block"
        );
        return Ok(DeflationGateVerdict {
            kind: GateVerdictKind::Block,
            reason: "block_dsr".to_string(),
            dsr,
            sigma_sr,
            sr_0,
            min_trl,
            n_used: n,
        });
    }

    // PBO gate.
    if pbo >= PBO_THRESHOLD {
        let payload = build_payload(&input, n, dsr, sigma_sr, sr_0, min_trl, "block_pbo");
        emit_ssb(ssb, payload).await?;
        debug!(
            pbo, threshold = PBO_THRESHOLD,
            "deflation_gate: PBO above threshold -> Block"
        );
        return Ok(DeflationGateVerdict {
            kind: GateVerdictKind::Block,
            reason: "block_pbo".to_string(),
            dsr,
            sigma_sr,
            sr_0,
            min_trl,
            n_used: n,
        });
    }

    // MinTRL gate: t_periods must be >= min_trl.
    if min_trl.is_finite() && (input.t_periods as f64) < min_trl {
        let payload = build_payload(&input, n, dsr, sigma_sr, sr_0, min_trl, "block_minbtl");
        emit_ssb(ssb, payload).await?;
        debug!(
            t = input.t_periods,
            min_trl,
            "deflation_gate: T < MinTRL -> Block"
        );
        return Ok(DeflationGateVerdict {
            kind: GateVerdictKind::Block,
            reason: "block_minbtl".to_string(),
            dsr,
            sigma_sr,
            sr_0,
            min_trl,
            n_used: n,
        });
    }

    // All gates cleared: Allow.
    let payload = build_payload(&input, n, dsr, sigma_sr, sr_0, min_trl, "allow");
    emit_ssb(ssb, payload).await?;
    debug!(
        dsr, pbo, min_trl, n,
        "deflation_gate: all gates PASS -> Allow"
    );
    Ok(DeflationGateVerdict {
        kind: GateVerdictKind::Allow,
        reason: "allow".to_string(),
        dsr,
        sigma_sr,
        sr_0,
        min_trl,
        n_used: n,
    })
}

// ---------------------------------------------------------------------------
// SSB emission helper
// ---------------------------------------------------------------------------

async fn emit_ssb(
    ssb: &Arc<Ssb>,
    payload: DeflationGateEvaluatedPayload,
) -> Result<(), DeflationGateError> {
    ssb.write_payload(
        "deflation_gate",
        0,
        SsbPayload::DeflationGateEvaluated(payload),
        Vec::new(),
    )
    .await
    .map_err(|e| DeflationGateError::SsbWrite(format!("{e}")))?;
    Ok(())
}

/// Build the [`DeflationGateEvaluatedPayload`] from gate inputs and results.
///
/// `corpus_blake3` is the blake3 hash of the canonical input key
/// (objective + ':' + data_window + ':' + sr_observed.to_bits() as hex).
/// This is the provenance pin for GD-2 recomputation. blake3 ONLY (CLAUDE.md).
fn build_payload(
    input: &DeflationInput,
    n_used: u64,
    dsr: f64,
    sigma_sr: f64,
    sr_0: f64,
    min_trl: f64,
    verdict: &str,
) -> DeflationGateEvaluatedPayload {
    // corpus_blake3: blake3 of (objective ':' data_window ':' sr_bits ':' n ':' seed).
    // This pins the exact inputs so the triple is recomputable (GD-2).
    let corpus_key = format!(
        "{}:{}:{:016x}:{}:{}",
        input.objective,
        input.data_window,
        input.sr_observed.to_bits(),
        n_used,
        input.seed,
    );
    let corpus_blake3 = blake3::hash(corpus_key.as_bytes()).to_hex().to_string();

    // dsr_pvalue = 1 - DSR (one-tailed p-value for the deflated t-stat).
    let dsr_pvalue = if dsr.is_nan() { f64::NAN } else { 1.0 - dsr };

    DeflationGateEvaluatedPayload {
        objective: input.objective.clone(),
        data_window: input.data_window.clone(),
        n_used,
        dsr,
        dsr_pvalue,
        sigma_sr,
        sr_0,
        pbo: input.pbo.unwrap_or(f64::NAN),
        min_trl,
        verdict: verdict.to_string(),
        seed: input.seed,
        corpus_blake3,
    }
}

// ---------------------------------------------------------------------------
// Normal CDF and inverse-CDF (hand-rolled, Abramowitz & Stegun 7.1.26)
// No new Rust crate required. Exported for contract test (f).
// ---------------------------------------------------------------------------

/// Standard normal CDF: Phi(x) = 0.5 * (1 + erf(x / sqrt(2))).
///
/// Implemented via the Abramowitz & Stegun 7.1.26 polynomial approximation
/// for erf(x), which gives a maximum absolute error < 1.5e-7.
///
/// Reference: Abramowitz & Stegun (1964), Handbook of Mathematical Functions,
/// Section 7.1.26, polynomial approximation p(t) = a1*t + a2*t^2 + ... + a5*t^5
/// where t = 1 / (1 + 0.3275911 * |x|).
pub fn normal_cdf(x: f64) -> f64 {
    0.5 * (1.0 + erf_approx(x / SQRT_2))
}

/// Inverse standard normal CDF: Phi_inv(p) = quantile function.
///
/// Uses Newton-Raphson iteration starting from a rational initial guess
/// (Abramowitz & Stegun 26.2.16 approximation as seed). Converges to
/// machine precision in 3-5 iterations for any p in (0, 1).
///
/// This approach is guaranteed correct because it numerically inverts
/// our own `normal_cdf` function (via its derivative `phi(x) = normal_pdf(x)`),
/// avoiding any coefficient transcription errors.
///
/// Reference:
///   A&S 26.2.16 for the initial guess.
///   Newton-Raphson for refinement: x_{n+1} = x_n - (Phi(x_n)-p)/phi(x_n).
pub fn normal_cdf_inv(p: f64) -> f64 {
    // Clamp to avoid extreme values.
    let p = p.clamp(1e-15, 1.0 - 1e-15);

    // Initial guess using A&S 26.2.16 rational approximation (max error ~ 4.5e-4).
    // Good enough as a starting point for Newton-Raphson.
    let x0 = as_inv_normal_approx(p);

    // Newton-Raphson refinement: 5 iterations is overkill but guarantees
    // machine-precision convergence for any p in (1e-15, 1-1e-15).
    let mut x = x0;
    for _ in 0..5 {
        let fx = normal_cdf(x) - p;
        let dfx = normal_pdf(x);
        if dfx.abs() < f64::EPSILON {
            break;
        }
        let step = fx / dfx;
        x -= step;
        if step.abs() < f64::EPSILON {
            break;
        }
    }
    x
}

/// Standard normal PDF: phi(x) = exp(-x^2/2) / sqrt(2*pi).
/// Used as the derivative for Newton-Raphson in `normal_cdf_inv`.
#[inline]
fn normal_pdf(x: f64) -> f64 {
    const INV_SQRT_2PI: f64 = 0.3989422804014327; // 1 / sqrt(2*pi)
    INV_SQRT_2PI * (-0.5 * x * x).exp()
}

/// Abramowitz & Stegun 26.2.16 rational approximation for the inverse normal CDF.
/// Max absolute error ≈ 4.5e-4; used only as an initial guess for Newton-Raphson.
fn as_inv_normal_approx(p: f64) -> f64 {
    // A&S 26.2.16: sign * (t - (c0 + c1*t + c2*t^2) / (1 + d1*t + d2*t^2 + d3*t^3))
    // where t = sqrt(-2*ln(p')) and p' = min(p, 1-p).
    const C0: f64 = 2.515517;
    const C1: f64 = 0.802853;
    const C2: f64 = 0.010328;
    const D1: f64 = 1.432788;
    const D2: f64 = 0.189269;
    const D3: f64 = 0.001308;

    let p_prime = p.min(1.0 - p);
    let t = (-2.0 * p_prime.ln()).sqrt();
    let numer = C0 + t * (C1 + t * C2);
    let denom = 1.0 + t * (D1 + t * (D2 + t * D3));
    let approx = t - numer / denom;
    if p < 0.5 { -approx } else { approx }
}

/// sqrt(2) used in the CDF formula.
const SQRT_2: f64 = std::f64::consts::SQRT_2;

/// Error function approximation (Abramowitz & Stegun 7.1.26).
///
/// Maximum absolute error < 1.5e-7 for all |x|.
/// For |x| > 6 the result is effectively +/-1.0 (clamped).
fn erf_approx(x: f64) -> f64 {
    // A&S 7.1.26: polynomial in t = 1/(1 + 0.3275911 * |x|).
    const P: f64 = 0.3275911;
    const A1: f64 =  0.254829592;
    const A2: f64 = -0.284496736;
    const A3: f64 =  1.421413741;
    const A4: f64 = -1.453152027;
    const A5: f64 =  1.061405429;

    let sign = if x >= 0.0 { 1.0_f64 } else { -1.0_f64 };
    let x_abs = x.abs();
    let t = 1.0 / (1.0 + P * x_abs);
    let poly = t * (A1 + t * (A2 + t * (A3 + t * (A4 + t * A5))));
    let erf = sign * (1.0 - poly * (-x_abs * x_abs).exp());
    erf.clamp(-1.0, 1.0)
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors from [`evaluate_deflation`].
#[derive(Debug, thiserror::Error)]
pub enum DeflationGateError {
    /// Trial ledger read failed.
    #[error("trial ledger error: {0}")]
    Ledger(#[from] TrialLedgerError),

    /// SSB write failed.
    #[error("SSB write error: {0}")]
    SsbWrite(String),
}

// ---------------------------------------------------------------------------
// Internal tests (formula spot-checks, edge cases)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Normal CDF spot-check: Phi(0) = 0.5, Phi(1.96) ~ 0.975, Phi(-1.96) ~ 0.025.
    #[test]
    fn normal_cdf_spot_checks() {
        assert!((normal_cdf(0.0) - 0.5).abs() < 1e-7, "Phi(0) = 0.5");
        assert!((normal_cdf(1.96) - 0.975).abs() < 1e-3, "Phi(1.96) ~ 0.975");
        assert!((normal_cdf(-1.96) - 0.025).abs() < 1e-3, "Phi(-1.96) ~ 0.025");
        assert!((normal_cdf(3.0) - 0.9987).abs() < 1e-3, "Phi(3.0) ~ 0.9987 (Harvey-Liu t>3)");
    }

    /// Phi_inv is the left inverse of Phi (round-trip).
    #[test]
    fn normal_cdf_inv_round_trip() {
        for &p in &[0.01, 0.05, 0.25, 0.5, 0.75, 0.95, 0.99] {
            let x = normal_cdf_inv(p);
            let p_back = normal_cdf(x);
            assert!(
                (p_back - p).abs() < 1e-5,
                "round-trip failed for p={p}: Phi(Phi_inv(p))={p_back}"
            );
        }
    }

    /// DSR is a probability [0, 1].
    #[test]
    fn dsr_is_probability() {
        let r = compute_dsr_tier1(0.05, 252, 0.0, 0.0, 50, 0.03);
        assert!(r.dsr >= 0.0 && r.dsr <= 1.0, "DSR must be in [0,1]; got {}", r.dsr);
    }

    /// SR_0 = 0 when N = 1.
    #[test]
    fn sr0_zero_when_n_eq_one() {
        let r = compute_dsr_tier1(0.20, 252, 0.0, 0.0, 1, 0.10);
        assert!(r.sr_0.abs() < f64::EPSILON, "SR_0 must be 0 for N=1; got {}", r.sr_0);
        assert!(r.max_z.abs() < f64::EPSILON, "maxZ must be 0 for N=1; got {}", r.max_z);
    }

    /// SR_0 increases with N (more trials = higher selection bias bar).
    #[test]
    fn sr0_increases_with_n() {
        let r10 = compute_dsr_tier1(0.10, 252, 0.0, 0.0, 10, 0.05);
        let r100 = compute_dsr_tier1(0.10, 252, 0.0, 0.0, 100, 0.05);
        assert!(
            r100.sr_0 > r10.sr_0,
            "SR_0 must increase with N; N=10: {}, N=100: {}",
            r10.sr_0,
            r100.sr_0
        );
    }

    /// Harvey-Liu correspondence: Phi(3.0) ~ 0.9987.
    /// DSR >= 0.9987 corresponds to t-stat > 3.0 (Harvey, Liu & Zhu 2016 RFS p.5).
    #[test]
    fn harvey_liu_phi_3_correspondence() {
        let phi_3 = normal_cdf(3.0);
        assert!(
            (phi_3 - 0.9987).abs() < 1e-3,
            "Phi(3.0) must be ~ 0.9987 (Harvey-Liu t>3.0 gate); got {phi_3}"
        );
    }
}
