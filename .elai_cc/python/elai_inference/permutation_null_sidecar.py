"""
H-22 (Tier-2 frozen sidecar): block-permutation null distribution + p-value.

NOT the fail-closed gate (that is the Rust Tier-1 trading_null_gate.rs).
This module computes the MATH: it block-permutes a returns series K times,
builds the null Sharpe distribution, and returns the (C+1)/(K+1) permutation
p-value plus the null band. The Rust gate receives the scalar p-value + the
null band and checks them against the LOCKED alpha / N-corrected threshold.

# Tier-2 stub registration
# TODO[H22.TIER2-SIDECAR]: integrate the live sidecar HTTP endpoint when the
# strategy-promotion / backtest-re-run path ships (re-runs the real backtest
# under each permutation to PRODUCE the returns series).
# will_resolve_in: a future session when the backtest-runner store ships.
# Registered as a WIRE-DARK stub: the Rust gate fails CLOSED when this sidecar
# is not invoked (missing series / missing null -> Block, R1/R2). This module is
# REAL runnable numpy (NOT a stub body); it is frozen and not invoked live this
# run (cloud-only override).

Anti-slopsquat verification (2026-06-05):
    numpy>=1.24 already in elai_inference/pyproject.toml (pre-existing).
    No new dependency: block permutation is pure numpy.

Method grounding (Ojala & Garriga 2010, JMLR 11:1833-1863):
    p = (C + 1) / (K + 1)  where C = #{null_stat >= observed}, K = #permutations.
    The +1s make p never exactly 0 and conservative (Phipson & Smyth 2010,
    PMC6405018; sklearn permutation_test_score returns (C+1)/(K+1) exactly).
    Smallest achievable p = 1/(K+1); to resolve p < alpha need K >= 1/alpha - 1
    (the K-floor; an under-powered null is not evidence of significance).
    Block permutation (NOT iid shuffle) preserves return autocorrelation; iid
    shuffle is anticonservative on time-series (Ojala & Garriga block note).

Implementation notes:
    - block_size > 1 preserves within-block autocorrelation; only block ORDER
      is randomized. The trailing short block is kept whole (length preserved).
    - ELAI_SEED seeds the permutation order for GD-2 reproducibility (same-env/
      same-seed only; cross-hardware FP drift is physics, TD-5).
    - The sidecar does NOT advertise fail-closed (W-10). Only the Rust gate is
      fail-closed. This module returns NaN for degenerate inputs and lets the
      Rust gate make the fail-closed decision.

Do NOT:
    - Use iid permutation (np.random.permutation of the whole series) for an
      autocorrelated returns series (anticonservative; use block permutation).
    - Compute p = C / K (missing the +1s; p=0 becomes possible).
    - Treat a small in-sample p as proof of an out-of-sample edge (it is not;
      the permutation test only rules out the in-sample randomness null).
"""

from __future__ import annotations

import math
import os

import numpy as np

# ---------------------------------------------------------------------------
# Named constants (CLAUDE.md no-magic-numbers; mirrors Rust trading_null_gate.rs)
# ---------------------------------------------------------------------------

# Significance level alpha (LOCKED, mutable=0, TD-2). Mirrors Rust ALPHA.
ALPHA: float = 0.05

# Default block size for the time-series block permutation. Mirrors Rust
# DEFAULT_BLOCK_SIZE.
DEFAULT_BLOCK_SIZE: int = 5

# Production default permutation count. Mirrors Rust K_MIN_PERMUTATIONS:
# p_min = 1/(K+1) = 0.001 << alpha = 0.05.
K_MIN_PERMUTATIONS: int = 999

# Minimum series length for a meaningful null. Mirrors Rust MIN_SERIES_LEN.
MIN_SERIES_LEN: int = 8


def k_floor(alpha: float) -> int:
    """Absolute resolution floor ceil(1/alpha) - 1 for a stated alpha.

    Below this, p-value resolution 1/(K+1) is coarser than alpha and the
    significant/not-significant verdict is unreliable. For alpha=0.05 -> 19.
    """
    if alpha <= 0.0 or alpha >= 1.0:
        return 2**63 - 1
    return max(0, math.ceil(1.0 / alpha) - 1)


def sharpe_ratio(pnl: np.ndarray) -> float:
    """Per-period Sharpe ratio mean/std (ddof=1) of a P&L stream. NaN for a
    degenerate (zero-variance/too-short) stream.

    Mirrors the Rust trading_null_gate::sharpe_ratio so the null distributions
    agree under the same seed (GD-2).
    """
    arr = np.asarray(pnl, dtype=np.float64)
    if arr.size < 2:
        return float("nan")
    mean = float(arr.mean())
    std = float(arr.std(ddof=1))
    if std <= 0.0 or not math.isfinite(std):
        return float("nan")
    return mean / std


def strategy_sharpe(signals: np.ndarray, returns: np.ndarray) -> float:
    """Sharpe of the strategy P&L stream signal*return (elementwise product).

    The alignment-dependent statistic the permutation null operates on: a plain
    Sharpe of a returns series is order-invariant (permuting returns never
    changes mean/std), so the null must operate on the signal-return product,
    which permutation breaks (Ojala & Garriga Test-1: fixed predictor, permuted
    outcome). Mirrors the Rust trading_null_gate::strategy_sharpe.
    """
    sig = np.asarray(signals, dtype=np.float64)
    ret = np.asarray(returns, dtype=np.float64)
    return sharpe_ratio(sig * ret)


def block_permute(returns: np.ndarray, block_size: int, seed: int) -> np.ndarray:
    """Block-permute a returns series: split into contiguous blocks of
    ``block_size``, shuffle the block ORDER under ``seed``, concatenate.

    Within-block autocorrelation is preserved; only the block order is
    randomized. The output has the SAME length as the input (a trailing short
    block is kept whole). Deterministic for a given (returns, block_size, seed).
    """
    arr = np.asarray(returns, dtype=np.float64)
    n = arr.size
    bs = max(1, block_size)
    if n == 0:
        return arr.copy()
    starts = list(range(0, n, bs))
    blocks = [arr[s : min(s + bs, n)] for s in starts]
    rng = np.random.default_rng(seed)
    order = rng.permutation(len(blocks))
    return np.concatenate([blocks[i] for i in order]) if blocks else arr.copy()


def build_null_distribution(
    signals: np.ndarray,
    returns: np.ndarray,
    k: int,
    block_size: int,
    seed: int,
) -> np.ndarray:
    """Build the null distribution: ``k`` strategy-Sharpe statistics, each
    computed after BLOCK-PERMUTING the returns against the FIXED signal (Ojala &
    Garriga Test-1). Each permutation uses a distinct per-permutation seed.

    Mirrors the Rust per-permutation seed derivation
    (seed * 0x100000001b3 + i + 1) so the Rust and Python null distributions are
    comparable under the same base seed (GD-2 same-env).
    """
    sig = np.asarray(signals, dtype=np.float64)
    ret = np.asarray(returns, dtype=np.float64)
    null = np.empty(k, dtype=np.float64)
    fnv_prime = 0x100000001B3
    mask = (1 << 64) - 1
    for i in range(k):
        perm_seed = ((seed * fnv_prime) + i + 1) & mask
        permuted_returns = block_permute(ret, block_size, perm_seed)
        null[i] = strategy_sharpe(sig, permuted_returns)
    return null


def permutation_p_value(observed: float, null: np.ndarray) -> float:
    """p = (C + 1) / (K + 1), C = #{null_stat >= observed} (Ojala & Garriga 2010).

    Never 0; smallest value is 1/(K+1). Precondition: ``null`` non-empty + finite
    and ``observed`` finite (the Rust gate guards both; here we return NaN if the
    precondition fails so the gate can fail-closed).
    """
    arr = np.asarray(null, dtype=np.float64)
    k = arr.size
    if k == 0 or not math.isfinite(observed) or not np.all(np.isfinite(arr)):
        return float("nan")
    c = int(np.sum(arr >= observed))
    return (c + 1.0) / (k + 1.0)


def compute_null_band(
    signals: np.ndarray,
    returns: np.ndarray,
    n_trials: int,
    k: int = K_MIN_PERMUTATIONS,
    block_size: int = DEFAULT_BLOCK_SIZE,
    alpha: float = ALPHA,
    seed: int = 42,
) -> dict[str, float]:
    """Compute the block-permutation null band + p-value for a strategy.

    The Tier-2 producer of the off-agent artifact the Rust gate cannot fake. The
    statistic is the Sharpe of the strategy P&L stream signal*return; the null
    permutes the returns against the fixed signal (Ojala & Garriga Test-1).

    Parameters
    ----------
    signals : np.ndarray
        The strategy's per-bar signal/position (the FIXED predictor).
    returns : np.ndarray
        Observed per-bar market return series (the OUTCOME the null permutes).
        Must have the same length as ``signals``.
    n_trials : int
        N searched trials (from H-17), for the Bonferroni-corrected threshold.
    k : int
        Number of block permutations (must be >= k_floor(alpha)).
    block_size : int
        Block size for the time-series block permutation.
    alpha : float
        LOCKED significance level.
    seed : int
        ELAI_SEED for GD-2 reproducibility.

    Returns
    -------
    dict with keys: observed_statistic, p_value, threshold (alpha/N),
        null_band_low, null_band_high, k_permutations, n_trials, alpha,
        seed_used, significant (bool as float 1.0/0.0), reason.

    The Rust gate is the authority on the verdict; this dict supplies the
    evidence. For degenerate/under-powered inputs the relevant fields are NaN and
    ``reason`` names the fail-closed condition the Rust gate will enforce.
    """
    sig = np.asarray(signals, dtype=np.float64)
    ret = np.asarray(returns, dtype=np.float64)

    if ret.size < MIN_SERIES_LEN or sig.size != ret.size:
        return _nan_band(k, n_trials, alpha, seed, reason="missing_series")

    observed = strategy_sharpe(sig, ret)
    if not math.isfinite(observed):
        return _nan_band(k, n_trials, alpha, seed, reason="nan_statistic")

    if k < k_floor(alpha):
        return _nan_band(k, n_trials, alpha, seed, reason="insufficient_permutations")

    if n_trials < 1:
        return _nan_band(k, n_trials, alpha, seed, reason="trial_count_missing")

    null = build_null_distribution(sig, ret, k, block_size, seed)
    if null.size == 0 or not np.all(np.isfinite(null)):
        band = _nan_band(k, n_trials, alpha, seed, reason="nan_statistic")
        band["observed_statistic"] = observed
        return band

    p_value = permutation_p_value(observed, null)
    threshold = alpha / float(n_trials)
    low = float(np.min(null))
    high = float(np.max(null))
    significant = 1.0 if p_value < threshold else 0.0

    return {
        "observed_statistic": observed,
        "p_value": p_value,
        "threshold": threshold,
        "null_band_low": low,
        "null_band_high": high,
        "k_permutations": float(k),
        "n_trials": float(n_trials),
        "alpha": alpha,
        "seed_used": float(seed),
        "significant": significant,
        "reason": "edge_collapsed_to_null" if significant else "edge_survived_null",
    }


def _nan_band(k: int, n_trials: int, alpha: float, seed: int, reason: str) -> dict[str, float]:
    """NaN result dict for a fail-closed condition. The Rust gate is the
    authority and will Block on these; this records WHICH condition fired."""
    return {
        "observed_statistic": float("nan"),
        "p_value": float("nan"),
        "threshold": float("nan"),
        "null_band_low": float("nan"),
        "null_band_high": float("nan"),
        "k_permutations": float(k),
        "n_trials": float(n_trials),
        "alpha": alpha,
        "seed_used": float(seed),
        "significant": 0.0,
        "reason": reason,
    }


# ---------------------------------------------------------------------------
# Deterministic fixture check (ELAI_SEED spot-check for GD-2 verification)
# ---------------------------------------------------------------------------


def _spot_check() -> None:
    """Verify the two demo fixtures match the Rust gate semantics.

    Overfit fixture (zero-mean alternating): the observed statistic sits inside
    the null band -> survived the null -> NOT significant.
    Genuine fixture (strong positive drift): the observed Sharpe sits in the
    tail -> significant.
    """
    # Sign-alternating market with varying magnitude (i%3+1).
    market = np.array(
        [((i % 3) + 1) * (1.0 if i % 2 == 0 else -1.0) for i in range(120)],
        dtype=np.float64,
    )
    # Overfit: constant-long signal on a zero-mean market -> survives the null.
    overfit_signal = np.ones(120, dtype=np.float64)
    # Genuine: perfectly-aligned signal (sign of this bar's return) -> collapses.
    genuine_signal = np.sign(market)
    genuine_signal[genuine_signal == 0.0] = 1.0

    band_overfit = compute_null_band(overfit_signal, market, n_trials=1, seed=42)
    band_genuine = compute_null_band(genuine_signal, market, n_trials=1, seed=42)

    assert band_overfit["significant"] == 0.0, (
        f"overfit fixture must NOT be significant (edge survives null); "
        f"got p={band_overfit['p_value']:.4f} threshold={band_overfit['threshold']:.4f}"
    )
    assert band_genuine["significant"] == 1.0, (
        f"genuine fixture must be significant (edge collapses to null); "
        f"got p={band_genuine['p_value']:.4f} threshold={band_genuine['threshold']:.4f}"
    )
    print(
        f"[permutation_null_sidecar] spot-check PASS: "
        f"overfit p={band_overfit['p_value']:.4f} (survives null), "
        f"genuine p={band_genuine['p_value']:.4f} (collapses to null)"
    )


if __name__ == "__main__":
    # Smoke-test: run the spot-check when executed directly.
    # ELAI_SEED override for GD-2 provenance.
    _seed = int(os.environ.get("ELAI_SEED", "42"))
    _spot_check()
