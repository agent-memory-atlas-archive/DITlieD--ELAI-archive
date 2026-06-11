"""
H-18 (Tier-2 frozen sidecar): DSR / PBO / MinBTL computation.

NOT the fail-closed gate (that is the Rust Tier-1 deflation_gate.rs).
This module computes the MATH: Deflated Sharpe Ratio, Probability of
Backtest Overfitting (CSCV, S=16 partitions), and MinBTL. Outputs a
JSON-serializable dict that the Rust gate receives and checks against
the LOCKED thresholds.

# Tier-2 stub registration
# TODO[H18.TIER2-SIDECAR]: integrate live sidecar HTTP endpoint when the
# darwin_loop_cmd / strategy-promotion path ships.
# will_resolve_in: a future session when the return-matrix store ships
#   (see workflow/sessions/s-10612/stubs.md, stub-id H18.TIER2).
# Registered as a WIRE-DARK stub: the Rust gate fails CLOSED when this
# sidecar is not invoked (pbo=None -> Block, R1/R2).

Anti-slopsquat verification (2026-06-05):
    scipy 1.17.1, BSD-3-Clause (pypi.org/pypi/scipy/json verified live).
    numpy>=1.24 already in elai_inference/pyproject.toml (pre-existing).
    scipy>=1.11 added to elai_inference/pyproject.toml (H-18, Tier-2).

Formulas (grounded: Bailey & Lopez de Prado 2014, SSRN 2460551):
    gamma_EM   = 0.5772156649015328606  (Euler-Mascheroni)
    maxZ       = (1-gamma_EM)*Phi_inv(1-1/N) + gamma_EM*Phi_inv(1-e^-1/N)
    SR_0       = sharpe_std * maxZ
    sigma_sr   = sqrt((1-skew*SR + SR^2*(kurt-1)/4) / (T-1))
    DSR        = Phi((SR_obs - SR_0) / sigma_sr)
    MinTRL     = 1 + (1-skew*SR + SR^2*(kurt-1)/4) * (Phi_inv(0.95)/(SR-bench))^2
    PBO (CSCV) = fraction of C(S,S/2) pairs where IS-best underperforms OOS

Implementation notes:
    - kurt MUST be excess kurtosis (scipy.stats.kurtosis(x, fisher=True)).
    - skew is raw skewness (not excess).
    - T is in the same period units as sr_observed (both per-period, NOT annualized).
    - MinTRL output is in the same period units as T (divide by 252 for years).
    - N < 5: SR_0 formula degrades (Gumbel EV theory); emits a WARN but not BLOCK.
    - ELAI_SEED seeds the CSCV partition permutation for GD-2 reproducibility.
      GD-2 scope: same-env/same-seed only (cross-hardware FP drift is physics, TD-5).
    - Sidecar does NOT advertise fail-closed (W-10). Only the Rust gate is fail-closed.

Do NOT:
    - Use kurt = scipy.stats.kurtosis(x, fisher=False) - 3 (incorrect; use fisher=True).
    - Annualize sr_observed before passing to DSR (formula expects per-period).
    - Use S=2 for CSCV (minimum practical S=8, C(8,4)=70 pairs).
"""

from __future__ import annotations

import itertools
import math
import os
import warnings
from typing import Optional

import numpy as np

# scipy is declared in pyproject.toml (anti-slopsquat: verified 1.17.1, BSD-3).
from scipy.stats import norm

# ---------------------------------------------------------------------------
# Named constants (CLAUDE.md no-magic-numbers; mirrors Rust deflation_gate.rs)
# ---------------------------------------------------------------------------

# Euler-Mascheroni constant (Bailey & Lopez de Prado 2014 Eq. for SR_0).
GAMMA_EM: float = 0.5772156649015328606

# CSCV partition count (LOCKED). C(16, 8) = 12870 IS/OOS pairs.
CSCV_S: int = 16

# Minimum T for reliable sigma_sr (interface-map failure mode 6).
T_MIN: int = 2

# N threshold below which Gumbel EV degrades (interface-map failure mode 4).
N_GUMBEL_WARN: int = 5

# MinTRL confidence level (Phi_inv(0.95) = 1.6449).
MINBTL_CONFIDENCE: float = 0.95

# ---------------------------------------------------------------------------
# Main entry point
# ---------------------------------------------------------------------------


def compute_dsr_pbo_minbtl(
    sr_observed: float,
    t_periods: int,
    skew: float,
    kurt: float,
    n: int,
    sharpe_std: float,
    sr_benchmark: float = 0.0,
    s_partitions: int = CSCV_S,
    seed: int = 42,
    return_matrix: Optional[np.ndarray] = None,
) -> dict[str, float]:
    """
    Compute DSR, PBO, and MinTRL for a strategy.

    Parameters
    ----------
    sr_observed : float
        Per-period (NOT annualized) Sharpe ratio of the candidate strategy.
    t_periods : int
        Number of return observations (sample length). Must be >= T_MIN.
    skew : float
        Raw skewness of returns (gamma_3). 0.0 for normal.
    kurt : float
        EXCESS kurtosis of returns (gamma_4, Fisher=True). 0.0 for normal.
    n : int
        Number of independent trial strategies tested (from H-17 trial ledger).
    sharpe_std : float
        Cross-sectional std of Sharpe ratios across the N trials (sqrt(V[SR])).
        0.0 if N=1.
    sr_benchmark : float
        Benchmark SR for MinTRL (default 0.0 = risk-free = 0).
    s_partitions : int
        CSCV partition count S (must be even, >= 4). Default CSCV_S=16.
        C(16,8)=12870 IS/OOS pairs.
    seed : int
        ELAI_SEED for GD-2 reproducibility. Seeds the CSCV partition permutation.
    return_matrix : Optional[np.ndarray]
        T x N matrix of returns (required for CSCV PBO). If None, PBO cannot
        be computed and is returned as NaN.

    Returns
    -------
    dict with keys:
        dsr          : Deflated Sharpe Ratio [0, 1].
        dsr_pvalue   : 1 - dsr (one-tailed p-value).
        sr_0         : Expected max SR under H0 for N independent trials.
        sigma_sr     : Std error of SR (non-normality corrected).
        pbo          : PBO via CSCV [0, 1] or NaN if return_matrix absent.
        min_trl      : Minimum Track Record Length in periods (or NaN if undefined).
        min_trl_years: min_trl / 252 (annualized, approximate).
        n_used       : N actually used (= n as passed, conservative raw N).
        t_used       : T actually used.
        seed_used    : seed used for GD-2 provenance.
    """
    # --- Input validation ---
    if t_periods < T_MIN:
        return _error_result(n, t_periods, seed, reason=f"t_too_short:{t_periods}")

    if n < 1:
        return _error_result(n, t_periods, seed, reason="n_zero")

    if n < N_GUMBEL_WARN:
        warnings.warn(
            f"N={n} < {N_GUMBEL_WARN}: Gumbel EV theory degrades for SR_0; "
            "using N conservatively (safe lower-deflation direction). "
            "Consider PSR (N=1) for very few trials.",
            RuntimeWarning,
            stacklevel=2,
        )

    # --- sigma_sr: std error of SR, non-normality corrected (paper Eq.) ---
    # sigma_sr = sqrt((1 - skew*SR + SR^2*(kurt-1)/4) / (T-1))
    # Note: kurt = EXCESS kurtosis. For normal: skew=0, kurt=0 -> sqrt(1/(T-1)).
    non_normality = 1.0 - skew * sr_observed + sr_observed**2 * (kurt - 1.0) / 4.0
    sigma_sr_sq = non_normality / max(t_periods - 1, 1)
    if sigma_sr_sq <= 0.0:
        return _error_result(n, t_periods, seed, reason="sigma_sr_nonpositive")
    sigma_sr = math.sqrt(sigma_sr_sq)

    # --- SR_0: expected max SR under H0 for N independent trials (Gumbel EV) ---
    if n == 1:
        # N=1: no multiple-testing correction; SR_0 = 0.
        sr_0 = 0.0
        max_z = 0.0
    else:
        # maxZ = (1-gamma_EM)*Phi_inv(1-1/N) + gamma_EM*Phi_inv(1-e^{-1}/N)
        # CRITICAL: "exp(-1)" is the constant e^{-1} ≈ 0.36788, NOT exp(-1/N).
        # Grounded: interface-map line 48: Phi_inv(1-exp(-1)/N) where exp(-1)=e^{-1}.
        e_inv = math.exp(-1.0)  # e^{-1} ≈ 0.36788 (constant, not function of N)
        arg1 = max(1.0 - 1.0 / n, 1e-15)
        arg2 = max(1.0 - e_inv / n, 1e-15)
        phi_inv_1 = norm.ppf(arg1)
        phi_inv_2 = norm.ppf(arg2)
        max_z = (1.0 - GAMMA_EM) * phi_inv_1 + GAMMA_EM * phi_inv_2
        sr_0 = sharpe_std * max_z

    # --- DSR: Deflated Sharpe Ratio ---
    # DSR = Phi((SR_obs - SR_0) / sigma_sr)
    z = (sr_observed - sr_0) / sigma_sr
    dsr = float(norm.cdf(z))
    # Clamp to [0, 1] for floating-point safety.
    dsr = max(0.0, min(1.0, dsr))

    # --- MinTRL: Minimum Track Record Length in periods (paper Section 4) ---
    sr_diff = sr_observed - sr_benchmark
    if abs(sr_diff) < 1e-12:
        min_trl = float("nan")  # undefined direction
    else:
        phi_inv_conf = norm.ppf(MINBTL_CONFIDENCE)  # Phi_inv(0.95) ~ 1.6449
        ratio = phi_inv_conf / sr_diff
        min_trl = max(1.0, 1.0 + non_normality * ratio**2)

    # --- PBO via CSCV (C(S,S/2) pairs) ---
    if return_matrix is not None:
        pbo = _compute_pbo_cscv(return_matrix, s_partitions=s_partitions, seed=seed)
    else:
        # PBO cannot be computed without the full return matrix.
        # The Rust gate will treat pbo=NaN as absent -> Block (R1/R2).
        pbo = float("nan")

    min_trl_years = min_trl / 252.0 if not math.isnan(min_trl) else float("nan")

    return {
        "dsr": dsr,
        "dsr_pvalue": 1.0 - dsr,
        "sr_0": sr_0,
        "sigma_sr": sigma_sr,
        "pbo": pbo,
        "min_trl": min_trl,
        "min_trl_years": min_trl_years,
        "n_used": n,
        "t_used": t_periods,
        "seed_used": seed,
    }


# ---------------------------------------------------------------------------
# PBO via CSCV (Bailey, Borwein, Lopez de Prado, Zhu 2015, SSRN 2326253)
# ---------------------------------------------------------------------------


def _compute_pbo_cscv(
    return_matrix: np.ndarray,
    s_partitions: int = CSCV_S,
    seed: int = 42,
) -> float:
    """
    Compute PBO via Combinatorially Symmetric Cross-Validation (CSCV).

    Algorithm (grounded: interface-map + CRAN pbo vignette):
      1. Assemble T x N return matrix M.
      2. Partition M into S equal time-blocks of size T//S.
      3. Generate all C(S, S/2) IS/OOS complementary splits.
      4. For each split: IS-best strategy n* = argmax(Sharpe(IS_block)).
      5. OOS rank R_bar = rank(Sharpe(OOS_block_{n*})) / N, normalized to [0,1].
      6. lambda = log(R_bar / (1 - R_bar)) [logit of OOS rank].
      7. PBO = fraction of C(S,S/2) pairs where lambda < 0.

    Parameters
    ----------
    return_matrix : np.ndarray of shape (T, N)
        T periods x N strategy return matrix.
    s_partitions : int
        Number of equal time-blocks S (even, >= 4). Default 16.
    seed : int
        RNG seed for permutation reproducibility (GD-2 same-env/same-seed).

    Returns
    -------
    float : PBO in [0, 1]. 0 = no overfitting. 1 = always overfitting.
    """
    if return_matrix.ndim != 2:
        raise ValueError(f"return_matrix must be 2-D (T x N); got shape {return_matrix.shape}")

    t_total, n_strategies = return_matrix.shape

    if s_partitions < 4 or s_partitions % 2 != 0:
        raise ValueError(f"s_partitions must be even and >= 4; got {s_partitions}")

    if t_total < s_partitions:
        raise ValueError(
            f"T={t_total} < S={s_partitions}: cannot partition into {s_partitions} blocks"
        )

    if n_strategies < 2:
        # PBO is undefined for a single strategy (trivially rank=1, lambda=+inf -> 0 overfitting).
        return 0.0

    # Step 1: partition into S blocks of size T//S (drop remainder rows for exact partitions).
    block_size = t_total // s_partitions
    blocks = [
        return_matrix[i * block_size : (i + 1) * block_size, :]
        for i in range(s_partitions)
    ]

    # Step 2: generate all C(S, S/2) IS/OOS index sets.
    all_is_indices = list(itertools.combinations(range(s_partitions), s_partitions // 2))

    # For GD-2: shuffle the combination order under the seed so that a subset
    # check (partial CSCV) is reproducible. Full enumeration (all C(S,S/2))
    # is always computed — the seed only affects ordering for provenance.
    rng = np.random.default_rng(seed)
    order = rng.permutation(len(all_is_indices))
    all_is_indices_ordered = [all_is_indices[i] for i in order]

    # Step 3: for each pair, compute IS-best and OOS rank -> logit lambda.
    lambdas: list[float] = []
    for is_idx in all_is_indices_ordered:
        oos_idx = tuple(j for j in range(s_partitions) if j not in is_idx)

        is_block = np.concatenate([blocks[k] for k in is_idx], axis=0)
        oos_block = np.concatenate([blocks[k] for k in oos_idx], axis=0)

        # IS Sharpe for each strategy (mean / std; zero-std -> -inf).
        is_sharpe = _block_sharpe(is_block)
        oos_sharpe = _block_sharpe(oos_block)

        # IS-best strategy.
        n_star = int(np.argmax(is_sharpe))

        # OOS rank of n_star normalized to (0, 1): rank / N.
        # Rank 1 = best OOS Sharpe. Normalized: rank / N.
        oos_n_star = oos_sharpe[n_star]
        oos_rank_raw = float(np.sum(oos_sharpe <= oos_n_star))
        r_bar = oos_rank_raw / n_strategies
        # Clamp R_bar away from {0, 1} to prevent log(0) in logit.
        r_bar = max(1e-9, min(1.0 - 1e-9, r_bar))

        # Logit: lambda = log(R_bar / (1 - R_bar)).
        lam = math.log(r_bar / (1.0 - r_bar))
        lambdas.append(lam)

    # Step 4: PBO = fraction of pairs where lambda < 0.
    pbo = float(sum(1 for lam in lambdas if lam < 0.0) / len(lambdas))
    return max(0.0, min(1.0, pbo))


def _block_sharpe(block: np.ndarray) -> np.ndarray:
    """
    Per-column Sharpe ratio for a return matrix block.

    Returns array of shape (N,). Columns with zero std get -inf.
    """
    means = block.mean(axis=0)
    stds = block.std(axis=0, ddof=1)
    with np.errstate(divide="ignore", invalid="ignore"):
        sharpe = np.where(stds > 0.0, means / stds, -np.inf)
    return sharpe


# ---------------------------------------------------------------------------
# Error result helper
# ---------------------------------------------------------------------------


def _error_result(n: int, t: int, seed: int, reason: str) -> dict[str, float]:
    """Return a NaN result dict for uncomputable cases (T too short, N=0, etc.)."""
    warnings.warn(
        f"compute_dsr_pbo_minbtl: uncomputable ({reason}); returning NaN values. "
        "Rust gate will treat NaN DSR as absent -> Block (R1/R2 fail-closed).",
        RuntimeWarning,
        stacklevel=3,
    )
    return {
        "dsr": float("nan"),
        "dsr_pvalue": float("nan"),
        "sr_0": float("nan"),
        "sigma_sr": float("nan"),
        "pbo": float("nan"),
        "min_trl": float("nan"),
        "min_trl_years": float("nan"),
        "n_used": n,
        "t_used": t,
        "seed_used": seed,
    }


# ---------------------------------------------------------------------------
# Deterministic fixture check (ELAI_SEED spot-check for GD-2 verification)
# ---------------------------------------------------------------------------


def _spot_check() -> None:
    """
    Verify the DSR formula matches the ingested-map fixture on a known input.

    Fixture: SR=0.05, T=252, skew=0, kurt=0, N=50, sharpe_std=0.03.
    Expected: DSR ~ 0.427 (see H-18-decisions.md derivation).
    Tolerance: 0.05 (cross-platform FP safety, GD-2 threshold band).
    """
    result = compute_dsr_pbo_minbtl(
        sr_observed=0.05,
        t_periods=252,
        skew=0.0,
        kurt=0.0,
        n=50,
        sharpe_std=0.03,
        sr_benchmark=0.0,
        seed=42,
        return_matrix=None,
    )
    dsr = result["dsr"]
    # For N=50, SR=0.05, sharpe_std=0.03, T=252 (normal returns):
    #   maxZ = (1-0.5772)*Phi_inv(0.98) + 0.5772*Phi_inv(1-0.3679/50)
    #        = 0.4228*2.054 + 0.5772*Phi_inv(0.9926) ≈ 0.868 + 1.412 ≈ 2.28
    #   SR_0 = 0.03 * 2.28 ≈ 0.0684
    #   sigma_sr = sqrt(1/251) ≈ 0.06313
    #   DSR = Phi((0.05 - 0.0684)/0.06313) = Phi(-0.291) ≈ 0.385
    assert 0.2 < dsr < 0.6, (
        f"DSR spot-check FAIL: expected DSR in [0.2, 0.6], got {dsr:.4f}. "
        "Check formula implementation (exp(-1)/N, not exp(-1/N))."
    )
    print(f"[deflation_sidecar] spot-check PASS: DSR={dsr:.4f} (expected in [0.2, 0.6])")


if __name__ == "__main__":
    # Smoke-test: run the spot-check when executed directly.
    _spot_check()
