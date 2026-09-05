Algorithm: The Stationary Bootstrap
Citation: Politis, D.N. and Romano, J.P. (1994) "The Stationary Bootstrap"
Journal: Journal of the American Statistical Association, Vol 89, No 428, pp 1303-1313
DOI: 10.1080/01621459.1994.10476870
Reliability: HIGH (peer-reviewed JASA, Vol 89)
ArXiv: N/A (pre-arXiv, 1994 JASA publication)
Secondary reference: Kunsch, H.R. (1989) "The Jackknife and the Bootstrap for General Stationary Observations"
  Annals of Statistics, Vol 17, No 3, pp 1217-1261, DOI: 10.1214/aos/1176347265
  (original moving block bootstrap; Politis-Romano 1994 is the preferred modern variant)

Problem setup:
Bootstrap inference for estimators computed from serially correlated (weakly dependent) stationary time series data.
The key innovation over Kunsch 1989 MBB: block lengths are geometrically distributed (random), so the resampled
pseudo-time-series is itself stationary, fixing the non-stationarity defect of MBB.

Inputs:
x              | array[float]   | shape (n,)   | original time series, weakly dependent stationary observations
p              | float          | scalar       | geometric distribution parameter; E[L] = 1/p; must be in (0, 1)
B              | int            | scalar       | number of bootstrap replicates; typically 999 or 1000
seed           | int or None    | scalar       | RNG seed for reproducibility; None = non-deterministic

Outputs:
x_star         | array[float]   | shape (n,)   | one bootstrap replicate; same length as input
theta_star_b   | array[float]   | shape (B,)   | B bootstrap estimates of target statistic theta(x)
ci_lower       | float          | scalar       | lower confidence bound from percentile or normal method
ci_upper       | float          | scalar       | upper confidence bound

Hyperparameters:
p (block parameter) | default 1/sqrt(n) | range (0,1) | E[L]=1/p so default gives E[L]=sqrt(n); Politis-Romano §3 recommended default; lower p = longer expected blocks; high sensitivity: too small p loses resampling variance, too large loses serial correlation structure
B (replicates)      | default 999      | range [100, 10000] | 999 or 1000 standard; B=200 adequate for CI width; B=1000 for quantile tails

Critical invariants (testable assertions):
assert len(x_star) == len(x), "bootstrap replicate must have same length as original series"
assert 0 < p < 1, "geometric parameter p must be strictly in (0,1)"
assert abs(expected_block_length - 1.0 / p) < 1e-9, "E[L] = 1/p by geometric distribution definition"
# Consistency requires: E[L] -> inf and E[L]/n -> 0 as n -> inf
# For default p=1/sqrt(n): E[L]=sqrt(n) -> inf and sqrt(n)/n = 1/sqrt(n) -> 0. Both hold.
assert 1.0 / p < n, "expected block length must be less than series length for consistency"
# Bootstrap CI empirical coverage: for large n, |empirical_coverage - (1-alpha)| should be < delta
# No closed-form assertion; validated by Monte Carlo coverage simulation

Complexity:
Time per replicate: O(n) — each replicate builds n observations by geometric block draws with wrap-around
Space per replicate: O(n) — one replicate at a time; O(B*n) if all stored simultaneously
Parallelism: replicates are independent; embarrassingly parallel across B
Total for B replicates: O(B * n) time, O(n) space if streaming aggregation used

Algorithm (per replicate):
1. Draw starting index i uniformly from {0, 1, ..., n-1}
2. Draw block length L ~ Geometric(p), i.e. P(L=k) = p*(1-p)^(k-1)
3. Copy x[i], x[i+1 mod n], ..., x[i+L-1 mod n] into x_star (wrap-around at boundary)
4. Advance pointer by L; repeat from step 1 until x_star has n elements (truncate last block if needed)
5. Compute theta(x_star) and store

Failure modes:
p too large (short blocks): loses serial correlation structure; bootstrap variance underestimates true variance; CI too narrow
p too small (long blocks, E[L] approaching n): insufficient resampling; bootstrap distribution collapses; n/E[L] blocks available approaches 1
n too small (n < 50): consistency asymptotics do not apply; results unreliable regardless of p choice
Non-stationary input: theoretical guarantees void; trend/seasonality must be removed before bootstrap
Statistic non-smooth in x: bootstrap may fail for non-Hadamard-differentiable statistics (e.g. quantiles require correction)

Implementation pitfalls (do NOT do this):
Do NOT use fixed block length without wrap-around — creates boundary bias at series end (Kunsch MBB defect)
Do NOT sample blocks without replacement — resampling must be with replacement
Do NOT forget wrap-around (circular bootstrap): index must be taken mod n, not clamped at n
Do NOT use p=1/n (one block per replicate); no resampling occurs
Do NOT use p=1 (block length always 1); reduces to i.i.d. bootstrap, ignoring serial dependence
Do NOT compute E[L] as l (deterministic); stationary bootstrap uses geometric random length, not fixed l
Do NOT use the same seed across all B replicates (each replicate needs independent random draws)
Do NOT assume sqrt(n) default is optimal for all statistics; Politis-White 2004 gives data-driven selection

D-30 applicability:
PLAN-BODY NOTE: The D-30 plan (D-30_calibration_bootstrap_corpus.md) as written does NOT specify block-bootstrap
resampling of a time series. D-30's "bootstrap" is corpus-bootstrapping: importing a pre-existing generic
calibration corpus (TruthfulQA/MMLU/HaluEval) to warm-start tau before local data accumulates. The statistical
block-bootstrap (this paper) is NOT explicitly required by the D-30 plan body.

If block-bootstrap IS introduced at D-30 unblock time (e.g. for bootstrap confidence intervals on tau_bootstrap):
- bootstrap_resample(x=calibration_scores, p=1/sqrt(N), B=1000) per this paper's §3 default
- tau_recompute aggregates B replicates via conformal quantile: tau_ci = [quantile(theta_stars, alpha/2), quantile(theta_stars, 1-alpha/2)]
- Consistency requires N >= 100 calibration tuples (asymptotic); N=500 per D-16 finite-sample bound is adequate
- UniCR conformal floor: tau = max(tau_bootstrap_lower, tau_local) once local corpus >= 500

CQR secondary paper status:
Romano, Patterson, Candes 2019 "Conformalized Quantile Regression" (arXiv:1905.03222) VERIFIED on arxiv.org.
SKIP for D-30: D-30 plan body has NO explicit CQR call-site. Tau computation uses D-16 compute_tau directly.
CQR would be needed only if D-30 introduced heteroscedastic prediction intervals for tau — not present in plan.
