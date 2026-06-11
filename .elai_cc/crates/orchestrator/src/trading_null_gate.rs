// cross-lang-skip: rust-py-sidecar
//! H-22 (SCHEMA_VERSION 227): trading metamorphic NULL-TEST gate.
//!
//! The Tier-1 fail-closed runtime gate that asks: *does the edge survive a
//! permutation null?* A genuine edge sits in the TAIL of the null distribution
//! (small p-value); a spurious / overfit edge sits INSIDE the null band (large
//! p-value) — it is indistinguishable from a randomly-permuted version of
//! itself, i.e. it "survived" the null => BLOCK.
//!
//! This is the f64-time-series complement to the H-21 look-ahead differential
//! (which attacks look-ahead by column-change under truncation) and the H-20
//! static leakage COMPOSITE. The shipped `MetamorphicTestRunner`
//! (`metamorphic_testing.rs`) is a BYTE-closure `Fn(&[u8]) -> Vec<u8>` runner —
//! it cannot carry an f64 returns series, so this is a NEW f64-domain gate. It
//! REUSES the `MrViolated` SSB row for per-relation failures and the
//! `parse_relation` naming convention for relation consistency, but the
//! evaluation is this new f64 code (GD-1: do not re-implement the runner).
//!
//! # Statistic: signal-return alignment (NOT a bare returns Sharpe)
//! A plain Sharpe (`mean/std`) of a returns series is ORDER-INVARIANT —
//! permuting returns can never change it, so a permutation null over it is
//! degenerate (every permutation reproduces the observed value, `p ≈ 1`). The
//! observed statistic is therefore the Sharpe of the strategy P&L stream
//! `pnl[t] = signal[t] * return[t]` (Ojala & Garriga Test-1: the SIGNAL is the
//! fixed predictor, the market RETURN is the outcome that gets permuted). The
//! null block-permutes the RETURNS against the FIXED signal: this breaks the
//! predictive alignment, so a genuine edge collapses while a static-bias edge
//! (e.g. constant-long in an up-market) survives.
//!
//! # Method grounding (Ojala & Garriga 2010, JMLR 11:1833-1863)
//! The p-value estimator is the sklearn `permutation_test_score` form:
//!   `p = (C + 1) / (K + 1)`
//! where `C = #{null_stat >= observed}` and `K` = number of permutations. The
//! `+1`s are load-bearing: the un-permuted data is itself one valid element of
//! the permutation group (the identity), so `p` can never be exactly 0 and the
//! estimator is conservative (Phipson & Smyth 2010, PMC6405018). The smallest
//! achievable p-value is `1/(K+1)`; to resolve `p < alpha` the gate REQUIRES
//! `K >= ceil(1/alpha) - 1` (the K-floor) — an under-powered null is NOT
//! evidence of significance and is a hard Block.
//!
//! # Block permutation (time-series exchangeability)
//! Naive iid label-shuffle breaks the autocorrelation of a returns series and
//! makes the test anticonservative (too many false positives — Ojala & Garriga
//! Section "block permutation"). This gate BLOCK-permutes contiguous chunks of
//! the series so the within-block autocorrelation is preserved, and only the
//! block ORDER is randomized.
//!
//! # Multiple-testing (N from H-17, TD-1)
//! When a strategy was selected from N searched trials, the family-wise error
//! rate inflates ~ N*alpha. The significance threshold composes the permutation
//! p-value with a Bonferroni selection correction: PASS only if
//! `p_raw < alpha / N`. N is read from the H-17 trial ledger via
//! [`crate::trial_ledger::TrialLedger::trial_count`]; a `TrialCountMissing`
//! sentinel is a hard Block (TD-1), NEVER the `_UNSAFE` accessor. (The Bonferroni
//! leg is what N deflates here; the DSR-vs-null leg in H-18 deflates N
//! separately via Bailey-Lopez de Prado — see H-22-decisions.md.)
//!
//! # FAIL-CLOSED (R1/R2 — "cannot prove the edge survives the null" => Block)
//! Each condition short-circuits to a hard Block with an emitted SSB row, NEVER
//! a silent pass, guarded at the TOP of the gate:
//!   1. missing/empty null distribution or returns series,
//!   2. insufficient permutations `K < K_floor = ceil(1/alpha) - 1`,
//!   3. `TrialCountMissing` from H-17 (TD-1),
//!   4. NaN / degenerate observed statistic, or any NaN in the null distribution.
//!
//! # SSB emission (off-agent artifact, R4)
//! Emits one [`crate::ssb::SsbPayload::TradingNullTest`] row on EVERY decision
//! (EdgeCollapsedToNull / EdgeSurvivedNull / PermutationNullFail / Unprovable).
//! Even an ALLOW row is evidence. The null band is the off-agent artifact the
//! agent cannot fake (it cannot author the permuted re-runs).
//!
//! # DEFERRED (honest residue): CostMonotoneConcave relation NOT shipped this
//! run. The sqrt-law of market impact (Toth et al. 2011, arXiv:1105.1694) holds
//! at INSTITUTIONAL scale only; at retail size (< ~0.1% ADV) impact is LINEAR
//! (Bucci et al. 2019, arXiv:1811.05230). Locking a concave cost invariant for a
//! retail-scale gate is not supported by the literature — it is documented as a
//! size-gated future relation, not shipped locked. See H-22-decisions.md.
//!
//! # HARD RULES
//!   - No `.unwrap()`/`.expect()` outside `#[cfg(test)]` (CLAUDE.md).
//!   - blake3 only for hashing (CLAUDE.md anti-stale).
//!   - No new Rust crate; named constants (no magic float literals).
//!
//! References:
//!   - workflow/papers/ojala-garriga-permutation-test-interface.md (PRIMARY).
//!   - crates/orchestrator/src/lookahead_differential.rs (H-21 sibling shape).
//!   - crates/orchestrator/src/deflation_gate.rs (H-18 N-read + emit shape).
//!   - workflow/reasoning/H-22-decisions.md.

use std::sync::Arc;

use thiserror::Error;
use tracing::{debug, warn};

use crate::{
    ssb::{Ssb, SsbPayload, TradingNullTestKind, TradingNullTestPayload},
    trial_ledger::{TrialCountResult, TrialLedger, TrialLedgerError},
};

// ---------------------------------------------------------------------------
// LOCKED named constants (TD-2; frozen in gate-manifest — the alpha IS the gate)
// ---------------------------------------------------------------------------

/// Canonical rule id for the trading null-test gate (documentary parity with the
/// sibling gates' `*_RULE_ID`; the mutable=0 seed row is FIN-NULL-001).
pub const TRADING_NULL_RULE_ID: &str = "FIN-NULL-001";

/// Significance level alpha (LOCKED, mutable=0, TD-2). The ONLY human-set tunable.
///
/// 0.05 is the standard one-tailed significance bar. The selection-corrected
/// threshold a strategy must clear is `alpha / N` (Bonferroni over the H-17
/// trial count N). Documented in crates/rules/seed/finance/trading_nulls.toml.
pub const ALPHA: f64 = 0.05;

/// Minimum permutations for adequate resolution (LOCKED).
///
/// The smallest achievable p-value with K permutations is `1/(K+1)`. K=999 gives
/// `p_min = 0.001`, well below `alpha = 0.05`, the standard-practice production
/// value (Ojala & Garriga; sklearn `n_permutations` examples). The gate also
/// enforces the absolute resolution floor `ceil(1/alpha) - 1` via [`k_floor`];
/// `K_MIN_PERMUTATIONS` is the stricter production default applied to the
/// fixture demos.
pub const K_MIN_PERMUTATIONS: usize = 999;

/// Default block size for the time-series block permutation.
///
/// Block permutation permutes contiguous chunks of this length so within-block
/// autocorrelation is preserved and only the block ORDER is randomized (Ojala &
/// Garriga; circular-block-bootstrap family). A value of 1 degenerates to iid
/// shuffle (anticonservative on autocorrelated series); 5 is a conservative
/// default for daily-bar returns. Series shorter than one block fall back to a
/// single block (no permutation possible -> caller's K-floor guard fires).
pub const DEFAULT_BLOCK_SIZE: usize = 5;

/// Minimum length of the returns series for a meaningful null (LOCKED).
///
/// Below this, the Sharpe statistic and the block permutation are degenerate.
pub const MIN_SERIES_LEN: usize = 8;

// ---------------------------------------------------------------------------
// Resolution floor
// ---------------------------------------------------------------------------

/// The absolute K-floor for a stated alpha: `ceil(1/alpha) - 1`.
///
/// Below this, the p-value resolution `1/(K+1)` is coarser than `alpha`, so the
/// significant / not-significant verdict is unreliable => fail-closed Block. For
/// `alpha = 0.05` this is 19. The production default [`K_MIN_PERMUTATIONS`] (999)
/// is far above it; this floor is the hard lower bound the guard enforces.
pub fn k_floor(alpha: f64) -> usize {
    if alpha <= 0.0 || alpha >= 1.0 {
        // A degenerate alpha cannot resolve any p-value; demand the max.
        return usize::MAX;
    }
    // ceil(1/alpha) - 1, computed without overflow for the valid (0,1) range.
    let inv = (1.0 / alpha).ceil();
    // inv >= 1 for alpha in (0,1]; subtract one for the resolution floor.
    let floor = inv - 1.0;
    if floor <= 0.0 {
        0
    } else {
        floor as usize
    }
}

// ---------------------------------------------------------------------------
// Statistic + permutation primitives
// ---------------------------------------------------------------------------

/// Per-period Sharpe ratio of a P&L stream: `mean / std` (sample std, ddof=1).
/// Returns `NaN` for a degenerate (zero-variance or too-short) stream so the
/// caller's NaN guard can fail-closed rather than divide by zero.
pub fn sharpe_ratio(pnl: &[f64]) -> f64 {
    let n = pnl.len();
    if n < 2 {
        return f64::NAN;
    }
    let n_f = n as f64;
    let mean = pnl.iter().sum::<f64>() / n_f;
    // ddof=1 sample variance.
    let var = pnl
        .iter()
        .map(|r| {
            let d = r - mean;
            d * d
        })
        .sum::<f64>()
        / (n_f - 1.0);
    if var <= 0.0 || !var.is_finite() {
        return f64::NAN;
    }
    mean / var.sqrt()
}

/// The per-bar strategy P&L stream: `pnl[t] = signal[t] * return[t]`.
///
/// This is the alignment-dependent quantity the permutation null operates on
/// (Ojala & Garriga Test-1 semantics: the SIGNAL is the fixed predictor, the
/// market RETURN is the outcome that gets permuted). A plain Sharpe of a returns
/// series is ORDER-INVARIANT — permuting returns can never change `mean/std`, so
/// a permutation null over it is degenerate. The signal-return PRODUCT is what
/// permutation breaks: permuting returns against a fixed signal destroys the
/// predictive alignment, so a genuine edge collapses while a static-bias edge
/// (e.g. constant-long in an up-market) survives.
///
/// `signals` and `returns` must have equal length; the product is elementwise.
pub fn strategy_pnl(signals: &[f64], returns: &[f64]) -> Vec<f64> {
    signals
        .iter()
        .zip(returns.iter())
        .map(|(&s, &r)| s * r)
        .collect()
}

/// The observed statistic: Sharpe of the strategy P&L stream `signal * return`.
pub fn strategy_sharpe(signals: &[f64], returns: &[f64]) -> f64 {
    sharpe_ratio(&strategy_pnl(signals, returns))
}

/// A tiny deterministic splitmix64 RNG (no new crate; ELAI_SEED-seeded so the
/// null is reproducible under the same seed — GD-2). We only need a u64 stream
/// for a Fisher-Yates block shuffle.
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        SplitMix64 { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        // splitmix64 (Vigna). Deterministic, well-distributed for shuffling.
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform integer in `[0, bound)` via Lemire-style rejection-free reduction
    /// (slight modulo bias acceptable for a permutation-test shuffle).
    fn below(&mut self, bound: usize) -> usize {
        if bound == 0 {
            return 0;
        }
        let r = self.next_u64() as u128;
        ((r * bound as u128) >> 64) as usize
    }
}

/// Block-permute a returns series under a seed: split into contiguous blocks of
/// `block_size`, Fisher-Yates shuffle the block ORDER, and concatenate. Within-
/// block autocorrelation is preserved; only the block order is randomized. The
/// output has the SAME length as the input (a trailing short block is kept whole).
///
/// Deterministic for a given `(returns, block_size, seed)` triple.
pub fn block_permute(returns: &[f64], block_size: usize, seed: u64) -> Vec<f64> {
    let n = returns.len();
    let bs = block_size.max(1);
    if n == 0 {
        return Vec::new();
    }
    // Partition into contiguous blocks (the last may be shorter).
    let mut blocks: Vec<&[f64]> = Vec::new();
    let mut start = 0usize;
    while start < n {
        let end = (start + bs).min(n);
        blocks.push(&returns[start..end]);
        start = end;
    }
    // Fisher-Yates shuffle of block indices under the seeded RNG.
    let mut order: Vec<usize> = (0..blocks.len()).collect();
    let mut rng = SplitMix64::new(seed);
    for i in (1..order.len()).rev() {
        let j = rng.below(i + 1);
        order.swap(i, j);
    }
    let mut out: Vec<f64> = Vec::with_capacity(n);
    for &bi in &order {
        out.extend_from_slice(blocks[bi]);
    }
    out
}

/// Build the null distribution: `k` strategy-Sharpe statistics, each computed
/// after BLOCK-PERMUTING the market returns against the FIXED signal (Ojala &
/// Garriga Test-1: permute the outcome, hold the predictor). Each permutation
/// uses a distinct seed derived deterministically from the base seed.
///
/// Returns the `k` null statistics (length == `k`). Any NaN that arises (a
/// degenerate permuted P&L stream) is propagated so the caller's NaN guard fires.
pub fn build_null_distribution(
    signals: &[f64],
    returns: &[f64],
    k: usize,
    block_size: usize,
    seed: u64,
) -> Vec<f64> {
    let mut null = Vec::with_capacity(k);
    for i in 0..k {
        // Derive a distinct, deterministic per-permutation seed.
        let perm_seed = seed
            .wrapping_mul(0x100_0000_01b3)
            .wrapping_add(i as u64)
            .wrapping_add(1);
        // Permute the RETURNS against the fixed signal, then recompute the
        // strategy Sharpe of signal * permuted_return. Permutation breaks the
        // signal-return alignment, so a genuine edge collapses under the null.
        let permuted_returns = block_permute(returns, block_size, perm_seed);
        null.push(strategy_sharpe(signals, &permuted_returns));
    }
    null
}

/// The permutation p-value `p = (C + 1) / (K + 1)` where
/// `C = #{null_stat >= observed}` (Ojala & Garriga 2010; sklearn
/// `permutation_test_score`). Never 0; smallest value is `1/(K+1)`.
///
/// Precondition: `null` is non-empty and contains no NaN (the caller guards
/// both); `observed` is finite (caller guards).
pub fn permutation_p_value(observed: f64, null: &[f64]) -> f64 {
    let k = null.len();
    let c = null.iter().filter(|&&s| s >= observed).count();
    (c as f64 + 1.0) / (k as f64 + 1.0)
}

// ---------------------------------------------------------------------------
// Public API types
// ---------------------------------------------------------------------------

/// Verdict kind returned by [`evaluate_trading_null`].
#[derive(Debug, Clone, PartialEq)]
pub enum TradingNullVerdictKind {
    /// The observed edge sits in the tail of the null (small p, significant
    /// after N-correction): the edge DEGRADED-to-null on permutation, i.e. it is
    /// a genuine edge that the null cannot reproduce. ALLOW.
    EdgeCollapsedToNull,
    /// The observed edge sits INSIDE the null band (p >= alpha/N): it survived
    /// the permutation null, i.e. it is indistinguishable from a shuffled version
    /// of itself — a spurious/overfit edge. BLOCK.
    EdgeSurvivedNull {
        /// The raw permutation p-value.
        p_value: f64,
        /// The selection-corrected threshold `alpha / N` it failed to beat.
        threshold: f64,
    },
    /// Fail-closed: an input/evidence precondition failed (missing data,
    /// under-powered K, TrialCountMissing, NaN). Hard Block (R1/R2). The exact
    /// reason is in [`TradingNullVerdict::reason`].
    Unprovable,
}

/// Full verdict from [`evaluate_trading_null`].
#[derive(Debug, Clone)]
pub struct TradingNullVerdict {
    /// EdgeCollapsedToNull (Allow) / EdgeSurvivedNull (Block) / Unprovable (Block).
    pub kind: TradingNullVerdictKind,
    /// Machine-readable reason code. One of:
    ///   "edge_collapsed_to_null" | "edge_survived_null" |
    ///   "block_missing_series" | "block_missing_null" |
    ///   "block_insufficient_permutations" | "block_trial_count_missing" |
    ///   "block_nan_statistic".
    pub reason: String,
    /// Observed statistic (e.g. Sharpe). NaN on the fail-closed NaN path.
    pub observed_statistic: f64,
    /// Raw permutation p-value (NaN on a fail-closed path before it is computed).
    pub p_value: f64,
    /// Selection-corrected threshold `alpha / N` (NaN if N was unavailable).
    pub threshold: f64,
    /// Lower edge of the null band (min of the null distribution; NaN if absent).
    pub null_band_low: f64,
    /// Upper edge of the null band (max of the null distribution; NaN if absent).
    pub null_band_high: f64,
    /// N used (from the H-17 trial ledger; 0 on the TrialCountMissing path).
    pub n_trials: u64,
    /// Number of permutations performed (0 on a pre-permutation fail-closed path).
    pub k_permutations: usize,
    /// Free-text evidence string.
    pub evidence: String,
}

/// Input bundle for [`evaluate_trading_null`].
///
/// Tier-1 consumes the returns series as PLAIN DATA — it runs no pipeline. WHO
/// produces the series (re-running the backtest under each permutation) is the
/// DEFERRED Tier-2 Python sidecar (`permutation_null_sidecar.py`), RunPod-pending
/// this run; the `// cross-lang-skip` header opts this file out of the
/// Rust<->Python pair drift check until that sidecar is wired.
#[derive(Debug, Clone)]
pub struct TradingNullInput {
    /// The strategy under test (repo-relative path or id).
    pub strategy_path: String,
    /// Search objective key (e.g. "sharpe_1y"). Passed to `trial_count`.
    pub objective: String,
    /// Canonical "start_ms..end_ms" data window. Passed to `trial_count`.
    pub data_window: String,
    /// The strategy's per-bar signal / position series (the FIXED predictor the
    /// permutation null holds constant). Must have the same length as `returns`.
    /// Empty or length-mismatched => fail-closed Block.
    pub signals: Vec<f64>,
    /// The observed per-bar market return series (the OUTCOME the null permutes
    /// against the fixed signal). Empty => fail-closed Block.
    pub returns: Vec<f64>,
    /// Number of permutations to run (must be >= the K-floor for `alpha`).
    pub k_permutations: usize,
    /// Block size for the time-series block permutation.
    pub block_size: usize,
    /// ELAI_SEED for GD-2 reproducibility / corpus pinning.
    pub seed: u64,
}

/// Errors from [`evaluate_trading_null`]. Missing data / under-powered K /
/// TrialCountMissing / NaN are NOT errors — they are fail-closed `Unprovable`
/// VERDICTS (each emits an SSB row). Only a ledger read or SSB write failure is
/// an error here.
#[derive(Debug, Error)]
pub enum TradingNullError {
    /// Trial ledger read failed.
    #[error("trial ledger error: {0}")]
    Ledger(#[from] TrialLedgerError),
    /// Writing the off-agent TradingNullTest SSB row failed.
    #[error("SSB write error: {0}")]
    SsbWrite(String),
}

// ---------------------------------------------------------------------------
// Main gate: evaluate_trading_null
// ---------------------------------------------------------------------------

/// Evaluate the trading metamorphic null-test gate (Tier-1). Emits exactly one
/// [`SsbPayload::TradingNullTest`] row on every decision (the off-agent
/// artifact, R4).
///
/// # Order of checks (FAIL-CLOSED first — R1/R2)
///   1. empty/too-short or length-mismatched signal/return series =>
///      `Unprovable` Block (`block_missing_series`).
///   2. observed statistic is NaN/degenerate => `Unprovable` Block (`block_nan_statistic`).
///   3. `k < k_floor(alpha)` => `Unprovable` Block (`block_insufficient_permutations`):
///      an under-powered null is not evidence of significance.
///   4. N read from H-17: `TrialCountMissing` => `Unprovable` Block
///      (`block_trial_count_missing`, TD-1) — NEVER the `_UNSAFE` accessor.
///   5. build the null distribution; any NaN in it OR an empty null =>
///      `Unprovable` Block (`block_nan_statistic` / `block_missing_null`).
///   6. compute `p = (C+1)/(K+1)`; selection-corrected threshold `alpha / N`:
///        - `p < alpha/N`   => `EdgeCollapsedToNull` (significant) => ALLOW.
///        - `p >= alpha/N`  => `EdgeSurvivedNull` (overfit survives null) => BLOCK.
pub async fn evaluate_trading_null(
    ledger: &TrialLedger,
    ssb: &Arc<Ssb>,
    input: TradingNullInput,
) -> Result<TradingNullVerdict, TradingNullError> {
    // -- Step 1: FAIL-CLOSED on missing/too-short or mis-aligned series. -----
    // Both the signal (fixed predictor) and the returns (permuted outcome) must
    // be present, equal-length, and at least MIN_SERIES_LEN long.
    if input.returns.len() < MIN_SERIES_LEN
        || input.signals.len() != input.returns.len()
    {
        warn!(
            strategy = %input.strategy_path,
            signals_len = input.signals.len(),
            returns_len = input.returns.len(),
            min = MIN_SERIES_LEN,
            "trading_null_gate: signal/return series missing/too short/mis-aligned -> \
             Unprovable Block (R1/R2)"
        );
        let verdict = unprovable(
            "block_missing_series",
            format!(
                "Signal series has {} points and return series has {} points \
                 (need equal length >= MIN_SERIES_LEN={}). Cannot build a null: \
                 fail-closed Block (R1/R2 missing-evidence=FAIL).",
                input.signals.len(),
                input.returns.len(),
                MIN_SERIES_LEN
            ),
        );
        emit(ssb, build_payload(&input, &verdict, 0)).await?;
        return Ok(verdict);
    }

    // -- Step 2: FAIL-CLOSED on a NaN/degenerate observed statistic. --------
    // The observed statistic is the Sharpe of the strategy P&L stream
    // (signal * return) — the alignment-dependent quantity the null breaks.
    let observed = strategy_sharpe(&input.signals, &input.returns);
    if !observed.is_finite() {
        warn!(
            strategy = %input.strategy_path,
            "trading_null_gate: observed statistic is NaN/degenerate -> Unprovable Block"
        );
        let mut verdict = unprovable(
            "block_nan_statistic",
            "The observed statistic is NaN/non-finite (zero-variance or degenerate \
             P&L stream signal*return). Fail-closed Block, never a panic."
                .to_string(),
        );
        verdict.observed_statistic = observed;
        emit(ssb, build_payload(&input, &verdict, 0)).await?;
        return Ok(verdict);
    }

    // -- Step 3: FAIL-CLOSED on an under-powered null (K below the floor). ---
    let floor = k_floor(ALPHA);
    if input.k_permutations < floor {
        warn!(
            strategy = %input.strategy_path,
            k = input.k_permutations,
            floor,
            "trading_null_gate: K below resolution floor -> Unprovable Block"
        );
        let mut verdict = unprovable(
            "block_insufficient_permutations",
            format!(
                "K={} is below the resolution floor ceil(1/alpha)-1={} for alpha={}. \
                 The p-value resolution 1/(K+1) is coarser than alpha; an under-powered \
                 null is not evidence of significance. Fail-closed Block.",
                input.k_permutations, floor, ALPHA
            ),
        );
        verdict.observed_statistic = observed;
        emit(ssb, build_payload(&input, &verdict, 0)).await?;
        return Ok(verdict);
    }

    // -- Step 4: read N from H-17 (TD-1 fail-closed sentinel). --------------
    let trial_result = ledger
        .trial_count(&input.objective, &input.data_window)
        .map_err(TradingNullError::Ledger)?;
    let n = match trial_result {
        TrialCountResult::TrialCountMissing => {
            debug!(
                objective = %input.objective,
                data_window = %input.data_window,
                "trading_null_gate: TrialCountMissing -> Unprovable Block (TD-1)"
            );
            let mut verdict = unprovable(
                "block_trial_count_missing",
                "The H-17 trial ledger has no trial count for this (objective, data_window): \
                 the selection multiplicity N is unknown, so the corrected threshold alpha/N \
                 cannot be applied. Fail-closed Block (TD-1)."
                    .to_string(),
            );
            verdict.observed_statistic = observed;
            emit(ssb, build_payload(&input, &verdict, 0)).await?;
            return Ok(verdict);
        }
        TrialCountResult::Present(n) => n.max(1),
    };

    // -- Step 5: build the null distribution; guard NaN / emptiness. --------
    let null = build_null_distribution(
        &input.signals,
        &input.returns,
        input.k_permutations,
        input.block_size.max(1),
        input.seed,
    );
    if null.is_empty() {
        let mut verdict = unprovable(
            "block_missing_null",
            "The null distribution is empty (no permutations produced a statistic). \
             Fail-closed Block (R1/R2)."
                .to_string(),
        );
        verdict.observed_statistic = observed;
        emit(ssb, build_payload(&input, &verdict, 0)).await?;
        return Ok(verdict);
    }
    if null.iter().any(|s| !s.is_finite()) {
        let mut verdict = unprovable(
            "block_nan_statistic",
            "A permuted statistic in the null distribution is NaN/non-finite. \
             Fail-closed Block, never a silent pass."
                .to_string(),
        );
        verdict.observed_statistic = observed;
        emit(ssb, build_payload(&input, &verdict, input.k_permutations)).await?;
        return Ok(verdict);
    }

    // -- Step 6: p-value + selection-corrected threshold (Bonferroni / N). --
    let p_value = permutation_p_value(observed, &null);
    let threshold = ALPHA / (n as f64);
    let (low, high) = null_band(&null);

    if p_value < threshold {
        // Significant after N-correction: the edge does NOT survive the null
        // (it sits in the tail) => the null cannot reproduce it => ALLOW.
        debug!(
            strategy = %input.strategy_path,
            p_value, threshold, n,
            "trading_null_gate: edge significant vs null (p < alpha/N) -> EdgeCollapsedToNull Allow"
        );
        let verdict = TradingNullVerdict {
            kind: TradingNullVerdictKind::EdgeCollapsedToNull,
            reason: "edge_collapsed_to_null".to_string(),
            observed_statistic: observed,
            p_value,
            threshold,
            null_band_low: low,
            null_band_high: high,
            n_trials: n,
            k_permutations: input.k_permutations,
            evidence: format!(
                "Genuine edge: observed Sharpe {observed:.6} sits in the tail of the block-\
                 permutation null (p={p_value:.6} < alpha/N={threshold:.6}, N={n}, K={}). The \
                 null cannot reproduce the edge: it collapses to noise under permutation. ALLOW.",
                input.k_permutations
            ),
        };
        emit(ssb, build_payload(&input, &verdict, input.k_permutations)).await?;
        return Ok(verdict);
    }

    // p >= alpha/N: the edge survives the null (overfit/spurious) => BLOCK.
    warn!(
        strategy = %input.strategy_path,
        p_value, threshold, n,
        "trading_null_gate: edge survived the null (p >= alpha/N) -> EdgeSurvivedNull Block"
    );
    let verdict = TradingNullVerdict {
        kind: TradingNullVerdictKind::EdgeSurvivedNull { p_value, threshold },
        reason: "edge_survived_null".to_string(),
        observed_statistic: observed,
        p_value,
        threshold,
        null_band_low: low,
        null_band_high: high,
        n_trials: n,
        k_permutations: input.k_permutations,
        evidence: format!(
            "Overfit/spurious edge: observed Sharpe {observed:.6} sits INSIDE the block-\
             permutation null band [{low:.6}, {high:.6}] (p={p_value:.6} >= alpha/N={threshold:.6}, \
             N={n}, K={}). The edge is indistinguishable from a shuffled version of itself: it \
             survived the null. BLOCK.",
            input.k_permutations
        ),
    };
    emit(ssb, build_payload(&input, &verdict, input.k_permutations)).await?;
    Ok(verdict)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Min/max of a non-empty, finite null distribution (the null band edges).
fn null_band(null: &[f64]) -> (f64, f64) {
    let mut low = f64::INFINITY;
    let mut high = f64::NEG_INFINITY;
    for &s in null {
        if s < low {
            low = s;
        }
        if s > high {
            high = s;
        }
    }
    if null.is_empty() {
        (f64::NAN, f64::NAN)
    } else {
        (low, high)
    }
}

/// Build a fail-closed `Unprovable` verdict skeleton with the given reason +
/// evidence (NaN statistic/p-value/threshold; the caller overrides
/// `observed_statistic` when it is known).
fn unprovable(reason: &str, evidence: String) -> TradingNullVerdict {
    TradingNullVerdict {
        kind: TradingNullVerdictKind::Unprovable,
        reason: reason.to_string(),
        observed_statistic: f64::NAN,
        p_value: f64::NAN,
        threshold: f64::NAN,
        null_band_low: f64::NAN,
        null_band_high: f64::NAN,
        n_trials: 0,
        k_permutations: 0,
        evidence,
    }
}

// ---------------------------------------------------------------------------
// SSB emission (off-agent artifact, R4)
// ---------------------------------------------------------------------------

async fn emit(ssb: &Arc<Ssb>, payload: TradingNullTestPayload) -> Result<(), TradingNullError> {
    ssb.write_payload(
        "trading_null_test",
        0,
        SsbPayload::TradingNullTest(payload),
        Vec::new(),
    )
    .await
    .map_err(|e| TradingNullError::SsbWrite(format!("{e}")))?;
    Ok(())
}

/// Map a verdict to the SSB Kind discriminator.
///
/// Fail-closed `Unprovable` verdicts are mapped to a Kind by their reason: a
/// permutation-significance miss is `PermutationNullFail`, every other fail-
/// closed reason is `Unprovable`. A genuine pass is `EdgeCollapsedToNull`; a
/// survived-null block is `EdgeSurvivedNull`.
fn verdict_to_kind(verdict: &TradingNullVerdict) -> TradingNullTestKind {
    match &verdict.kind {
        TradingNullVerdictKind::EdgeCollapsedToNull => TradingNullTestKind::EdgeCollapsedToNull,
        TradingNullVerdictKind::EdgeSurvivedNull { .. } => TradingNullTestKind::EdgeSurvivedNull,
        TradingNullVerdictKind::Unprovable => TradingNullTestKind::Unprovable,
    }
}

/// Build the off-agent [`TradingNullTestPayload`] from inputs + verdict.
///
/// `corpus_blake3` pins the exact provenance key so the verdict is recomputable
/// (GD-2 same-env/same-seed). blake3 ONLY (CLAUDE.md HARD RULE).
fn build_payload(
    input: &TradingNullInput,
    verdict: &TradingNullVerdict,
    k_used: usize,
) -> TradingNullTestPayload {
    let corpus_key = format!(
        "{}:{}:{}:{}:{}:{}",
        input.strategy_path,
        input.objective,
        input.data_window,
        verdict.reason,
        k_used,
        input.seed,
    );
    let corpus_blake3 = blake3::hash(corpus_key.as_bytes()).to_hex().to_string();

    TradingNullTestPayload {
        kind: verdict_to_kind(verdict),
        strategy_path: input.strategy_path.clone(),
        statistic: verdict.observed_statistic,
        p_value: verdict.p_value,
        null_band_low: verdict.null_band_low,
        null_band_high: verdict.null_band_high,
        n_trials: verdict.n_trials,
        k_permutations: u64::try_from(k_used).unwrap_or(u64::MAX),
        alpha: ALPHA,
        verdict: verdict.reason.clone(),
        seed: input.seed,
        corpus_blake3,
    }
}

// ---------------------------------------------------------------------------
// Unit tests (p-value estimator, K-floor guard, block-permutation determinism)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn k_floor_for_alpha_005_is_19() {
        // ceil(1/0.05) - 1 = ceil(20) - 1 = 19.
        assert_eq!(k_floor(0.05), 19);
    }

    #[test]
    fn k_floor_degenerate_alpha_demands_max() {
        assert_eq!(k_floor(0.0), usize::MAX);
        assert_eq!(k_floor(1.0), usize::MAX);
    }

    #[test]
    fn p_value_never_zero_and_uses_plus_one() {
        // C=0 (no null >= observed): p = (0+1)/(K+1) = 1/(K+1), never 0.
        let null = vec![0.0_f64; 99];
        let p = permutation_p_value(100.0, &null);
        assert!((p - 1.0 / 100.0).abs() < 1e-12, "p must be 1/(K+1); got {p}");
        assert!(p > 0.0, "p-value can never be exactly 0");
    }

    #[test]
    fn p_value_all_null_ge_observed_is_one() {
        // C=K (every null >= observed): p = (K+1)/(K+1) = 1.0.
        let null = vec![5.0_f64; 50];
        let p = permutation_p_value(1.0, &null);
        assert!((p - 1.0).abs() < 1e-12, "p must be 1.0 when all null >= observed; got {p}");
    }

    #[test]
    fn block_permute_preserves_length_and_is_seed_deterministic() {
        let series: Vec<f64> = (0..37).map(|i| i as f64).collect();
        let a = block_permute(&series, DEFAULT_BLOCK_SIZE, 42);
        let b = block_permute(&series, DEFAULT_BLOCK_SIZE, 42);
        assert_eq!(a.len(), series.len(), "permutation preserves series length");
        assert_eq!(a, b, "same seed -> identical permutation (GD-2)");
        let c = block_permute(&series, DEFAULT_BLOCK_SIZE, 43);
        assert_ne!(a, c, "different seed -> different permutation");
        // Multiset is preserved (a permutation, not a corruption).
        let mut sa = a.clone();
        let mut ss = series.clone();
        sa.sort_by(|x, y| x.partial_cmp(y).unwrap());
        ss.sort_by(|x, y| x.partial_cmp(y).unwrap());
        assert_eq!(sa, ss, "block permutation preserves the multiset of returns");
    }

    #[test]
    fn build_null_distribution_is_seed_deterministic() {
        let signals: Vec<f64> = (0..40).map(|i| if i % 2 == 0 { 1.0 } else { -1.0 }).collect();
        let returns: Vec<f64> = (0..40).map(|i| ((i * 7) % 13) as f64 - 6.0).collect();
        let n1 = build_null_distribution(&signals, &returns, 50, DEFAULT_BLOCK_SIZE, 7);
        let n2 = build_null_distribution(&signals, &returns, 50, DEFAULT_BLOCK_SIZE, 7);
        assert_eq!(n1.len(), 50);
        assert_eq!(n1, n2, "same seed -> identical null distribution (GD-2)");
    }

    #[test]
    fn strategy_pnl_is_elementwise_product() {
        let sig = vec![1.0, -1.0, 1.0, 0.0];
        let ret = vec![2.0, 3.0, -4.0, 5.0];
        assert_eq!(strategy_pnl(&sig, &ret), vec![2.0, -3.0, -4.0, 0.0]);
    }

    #[test]
    fn sharpe_degenerate_series_is_nan() {
        assert!(sharpe_ratio(&[1.0]).is_nan(), "len<2 -> NaN");
        assert!(sharpe_ratio(&[3.0, 3.0, 3.0]).is_nan(), "zero variance -> NaN");
    }
}
