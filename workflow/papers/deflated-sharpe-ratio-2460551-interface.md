Algorithm: Deflated Sharpe Ratio (DSR) with PSR, MinBTL, and PBO via CSCV
Paper: The Deflated Sharpe Ratio: Correcting for Selection Bias, Backtest Overfitting and Non-Normality
Authors: David H. Bailey, Marcos Lopez de Prado
Published: Journal of Portfolio Management 40(5):94-107, 2014
SSRN: 2460551
Source PDF: https://www.davidhbailey.com/dhbpapers/deflated-sharpe.pdf
Companion PBO paper: Bailey, Borwein, Lopez de Prado, Zhu (2015), Journal of Computational Finance, SSRN 2326253
Harvey-Liu threshold source: Harvey, Liu & Zhu (2016), Review of Financial Studies 29(1):5-68, NBER w20592
Reliability: HIGH (peer-reviewed journal, reproducible from Wikipedia + multiple independent implementations)

Inputs:
sr_observed  | float | scalar | per-period (NOT annualized) Sharpe ratio of the candidate strategy; mandatory
T            | int   | scalar | number of return observations (sample length); must be >= 2; mandatory
skew         | float | scalar | skewness of returns (gamma_3); can be 0.0 if unknown; mandatory for non-normal adjustment
kurt         | float | scalar | EXCESS kurtosis of returns (gamma_4, Fisher=True / scipy.stats default); can be 0.0 for normal; mandatory
N            | int   | scalar | number of independent trial strategies tested; >= 1; persisted input for selection-bias correction; mandatory
sharpe_std   | float | scalar | cross-sectional std of Sharpe ratios across the N trials (sqrt(V[SR])); mandatory for SR_0; if N=1 then SR_0=0
sr_benchmark | float | scalar | benchmark Sharpe ratio for MinTRL; default 0.0; optional

Outputs:
dsr          | float | scalar | Deflated Sharpe Ratio in [0, 1]; probability candidate SR is skill not luck given N trials
psr          | float | scalar | Probabilistic Sharpe Ratio in [0, 1]; same formula as DSR with SR_0 = sr_benchmark (N=1 special case)
sr_0         | float | scalar | Expected maximum SR under N unskilled independent trials; >= 0; benchmark for DSR
sigma_sr     | float | scalar | Std error of Sharpe ratio; > 0; used as denominator in PSR/DSR
min_trl      | float | scalar | Minimum Track Record Length in PERIODS needed to reject null at alpha; >= 1

Critical invariants (testable assertions):
assert 0.0 <= dsr <= 1.0, "DSR must be a probability"
assert 0.0 <= psr <= 1.0, "PSR must be a probability"
assert sigma_sr > 0.0, "sigma_sr denominator must be positive"
assert sr_0 >= 0.0 or N == 1, "expected max SR under H0 must be non-negative"
assert min_trl >= 1.0, "MinTRL must be at least 1 period"
assert abs(dsr - norm.cdf((sr_observed - sr_0) / sigma_sr)) < 1e-9, "DSR = Phi((SR*-SR0)/sigma_sr)"

Core formulas (cite these equation numbers from the paper):

sigma_sr formula (paper Eq. correcting std error for non-normality):
  sigma_sr = sqrt((1 - skew*sr_observed + sr_observed**2 * (kurt - 1) / 4) / (T - 1))
  NOTE: kurt here is EXCESS kurtosis (kurt_fisher); for normal dist kurt=0 and formula reduces to 1/sqrt(T-1)
  NOTE: skew is raw skewness (not excess); for normal dist skew=0

PSR formula (Bailey & Lopez de Prado, DSR paper Section 2, "Probabilistic Sharpe Ratio"):
  psr = Phi((sr_observed - sr_benchmark) / sigma_sr)
  This is the N=1 case where SR_0 = sr_benchmark (no selection bias correction needed)

SR_0 formula (False Strategy Theorem, paper Section 3, Eq. for expected max SR under N independent trials):
  gamma_EM = 0.5772156649015328606  # Euler-Mascheroni constant
  maxZ = (1 - gamma_EM) * Phi_inv(1 - 1/N) + gamma_EM * Phi_inv(1 - exp(-1)/N)
  sr_0 = sharpe_std * maxZ
  SOURCE: Wikipedia Deflated Sharpe ratio article confirms formula with gamma as Euler-Mascheroni constant
  SOURCE: Balaena Medium article (2024) confirms V[SR] = sharpe_std**2 and the exact Phi_inv arguments
  NOTE: For N=1 sr_0=0 (no multiple testing); for large N, sr_0 grows as sqrt(2*ln(N))

DSR formula (paper Section 3):
  dsr = Phi((sr_observed - sr_0) / sigma_sr)
  DSR is PSR with SR_0 replacing the simple benchmark sr_benchmark
  Interpretation confirmed: DSR > 0.95 = strong evidence of skill (not explicitly a paper-cited threshold;
    Harvey-Liu t>3.0 (RFS 2016 p.5) corresponds to p>0.9987 which is more conservative than DSR>0.95)

MinTRL formula (paper Section 4, Minimum Track Record Length in periods):
  min_trl = 1 + (1 - skew*sr_observed + sr_observed**2 * (kurt - 1) / 4) * (Phi_inv(confidence) / (sr_observed - sr_benchmark))**2
  Default confidence = 0.95 -> Phi_inv = 1.6449
  Units: output is in the SAME period units as T (daily T gives daily MinTRL; divide by 252 for years)

MinBTL formula (Bailey, Borwein, Lopez de Prado, Zhu 2014 AMS Notices companion paper, NOT the DSR paper):
  max_z = (1-gamma_EM)*Phi_inv(1-1/N) + gamma_EM*Phi_inv(1-exp(-1)/N)
  min_btl_years = (max_z / sr_is_annualized)**2
  Upper bound approximation: 2*ln(N) / sr_is_annualized**2
  NOTE: MinBTL uses the annualized IS Sharpe; if actual_years < min_btl_years the backtest is too short

PBO via CSCV (from companion paper SSRN 2326253, Bailey Borwein Lopez de Prado Zhu 2015):
  Step 1: Assemble T x N matrix M of returns (T periods, N strategies)
  Step 2: Choose S (even, typically S=16); partition M into S equal time-blocks of size T/S
  Step 3: Generate all C(S, S/2) complementary IS/OOS pairs (e.g. S=16 -> C(16,8)=12870 pairs)
  Step 4: For each pair: identify IS-best strategy n* = argmax(perf(IS_block_n))
  Step 5: Compute OOS rank R_bar = rank(perf(OOS_block_{n*})) normalized to [0,1] over N strategies
  Step 6: Compute lambda = log(R_bar / (1 - R_bar))  [logit of OOS rank]
  Step 7: PBO = fraction of C(S,S/2) pairs where lambda < 0 (IS-best performs below median OOS)
  Performance metric: Sharpe ratio (default) or Omega ratio
  Threshold: PBO < 0.5 means less than half of IS-best selections underperform OOS (not overfit)
  PBO source: CRAN pbo vignette (https://cran.r-project.org/web/packages/pbo/vignettes/pbo.html) confirms
    lambda formula and C(S,S/2) partition count

N vs Ne (Effective Number of Independent Trials):
  N  = raw count of strategy configurations tested (directly observable; PERSISTED in trial ledger)
  Ne = effective number after adjusting for correlation between strategies (Ne <= N always)
  Ne is computed by ONC algorithm (see ONC interface map); Ne replaces N in DSR/SR_0 when available
  If Ne not computed: use N (conservative; overestimates selection bias; DSR is lower-bounded)
  Paper does not define Ne directly; Ne concept bridged from ONC paper (Lopez de Prado & Lewis 2019)

Harvey-Liu t>3.0 threshold (external grounding, NOT from DSR paper itself):
  Harvey, Liu & Zhu (2016) RFS 29(1):5-68 state: "by 2012 a new factor needed a t-statistic of at least 3.0"
  t-stat 3.0 <-> two-tailed p=0.0027 <-> one-tailed p=0.00135 <-> PSR/DSR ~ 0.9987
  ELAI H-18 default gate threshold: DSR >= 0.95 (conservative relative to Harvey-Liu; appropriate for
    strategy-level not factor-level; use DSR >= 0.9987 for strict Harvey-Liu compliance)

Hyperparameters:
S (CSCV partitions)  | default 16 | range [4, 32] even integers | higher S = more splits, more stable PBO estimate
confidence           | default 0.95 | range (0, 1) | alpha for MinTRL Phi_inv
sr_benchmark         | default 0.0 | range any real | risk-free rate or hurdle SR for PSR/MinTRL

Complexity:
Time: PSR/DSR O(1); MinTRL O(1); SR_0 O(1); PBO O(C(S,S/2) * N * T) where C(16,8)=12870
Space: O(N * T) for the strategy return matrix; O(C(S,S/2)) for lambda distribution
Parallelism: each CSCV partition pair is independent; trivially parallelizable

Failure modes:
1. Mixing annualized SR with per-period T: sigma_sr uses per-period sr_observed and T in same units
2. Using raw kurtosis (scipy kurtosis(fisher=False) adds 3) instead of excess kurtosis in sigma_sr
3. Under-reporting N (only counting final configs, not intermediate trials): inflates DSR
4. N < 5: SR_0 formula based on Gumbel extreme value theory degrades; use PSR (N=1) or increase trial tracking
5. PBO with S too small (S=2 -> C(2,1)=2 pairs): PBO estimate is noisy; use S >= 8
6. T < 50: sigma_sr is unreliable; MinTRL bound not tight
7. Correlated strategies counted as N independent: if trials are correlated, Ne < N; ONC correction required

Implementation pitfalls (do NOT do this):
Do NOT annualize sr_observed before passing to DSR/PSR; the formula expects per-period SR
Do NOT use kurt = scipy.stats.kurtosis(x, fisher=False) - 3; use scipy.stats.kurtosis(x, fisher=True) directly
Do NOT set N=1 when multiple strategies were tried; this ignores selection bias entirely
Do NOT use PBO as the only gate; it is necessary but not sufficient; combine with DSR and MinBTL
Do NOT use T-1 vs T inconsistently; paper uses T-1 in denominator of sigma_sr
Do NOT use S=2 for CSCV; minimum practical S is 8 (C(8,4)=70 pairs)
Do NOT replace sharpe_std with 1.0 as a default; must be computed from the actual trial SR distribution
