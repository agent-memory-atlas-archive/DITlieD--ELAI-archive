Algorithm: Permutation Tests for Studying Classifier Performance
Paper: Permutation Tests for Studying Classifier Performance
Journal: Journal of Machine Learning Research 11 (2010) 1833-1863
Authors: Markus Ojala, Gemma C. Garriga
Source: https://jmlr.csail.mit.edu/papers/volume11/ojala10a/ojala10a.pdf (open-access)
No arXiv ID (published directly in JMLR). sklearn implements Test 1 in permutation_test_score.
Companion: Phipson & Smyth (2010) on +1 correction; Bucci et al. (2019) PRL on sqrt-law scope.

NOTE ON CITATION IN H-22: the task description cited "Toth et al. 2011 arXiv:1004.0425" for the
sqrt-law. THAT arXiv ID IS WRONG -- 1004.0425 is a quantum walk paper (verified via WebFetch).
The correct Toth et al. 2011 paper is arXiv:1105.1694 (Anomalous price impact, Phys. Rev. X 1:021006).


THE TWO PERMUTATION NULL MODELS (Ojala & Garriga Section 2 / Table 1):

Test 1 (PermClass -- Label Permutation):
  What is permuted: the class labels y are randomly shuffled; features X stay fixed.
  Null hypothesis: labels are independent of features (no real class structure in data).
  What it tests: does the classifier exploit a real signal, or is its performance chance-level?
  If p is small: classifier found real label-feature dependency.
  If p is large: either no real dependency exists, OR the classifier failed to exploit it.
  sklearn implements this as permutation_test_score. This is the standard "no-edge" null.

Test 2 (PermFeats -- Feature Permutation Within Classes):
  What is permuted: features are permuted WITHIN each class (restricted randomization).
    Permuting within-class preserves the marginal class distributions.
    It destroys feature-feature dependency (joint structure) while keeping label-marginals.
  Null hypothesis: features are conditionally independent given the class label.
  What it tests: does the classifier exploit inter-feature dependency (joint structure),
    or would it perform equally if features were independent within each class?
  Diagnostic use: identifies descriptive/correlated features that are load-bearing.
  sklearn does NOT implement Test 2 natively.

Key distinction for H-22 trading gate:
  Test 1 (label perm) maps to: "is this strategy's Sharpe due to a real edge vs noise?"
    Permute the return series (labels/outcomes) while keeping time-ordering structure.
  Test 2 (feature perm) maps to: "does the strategy exploit feature co-dependencies?"
    Relevant for factor models; less relevant for simple returns-permutation gate.
  The H-22 null-test gate uses Test 1 semantics: permute returns/labels K times,
    build null Sharpe distribution, test observed Sharpe against the null.


P-VALUE ESTIMATOR (load-bearing formula for Rust gate):

  p = (C + 1) / (K + 1)

  where:
    C = number of permuted-statistic values >= observed statistic  (count, integer >= 0)
    K = total number of permutations performed  (integer, K >= 1)
    observed statistic = e.g. Sharpe computed on original un-permuted returns

  Why +1 in numerator:
    The original (un-permuted) data is itself one valid element of the permutation group.
    Including it as one implicit permutation (the identity) means p can never be 0.
    Without +1: if C=0, p=0/K=0, which is mathematically invalid (p-value cannot be
    exactly 0 from a finite Monte Carlo). The +1 makes the estimator conservative
    (slightly overstates the true p-value) and ensures stochastic dominance over uniform
    under the null. Grounding: Phipson & Smyth 2010 (SAGMB), PMC6405018 (exact p-values),
    sklearn permutation_test_score source (returns (C+1)/(K+1) exactly).

  Why +1 in denominator:
    Denominator is K+1 (total permutations including the identity), not K.
    This is consistent: numerator counts the identity as one permutation where the
    observed >= observed (trivially true), so +1 in denominator offsets +1 in numerator.

  Resolution bound (critical for gate fail-closed logic):
    Smallest achievable p-value with K permutations = 1/(K+1).
    Therefore: to achieve resolution p_min <= alpha, need K >= (1/alpha) - 1.
    For alpha=0.05: K_min = 19 (theoretical minimum, underpowered in practice).
    For alpha=0.05 with adequate power: K >= 999 (standard practice, p_min = 0.001).
    For alpha=0.01 with adequate power: K >= 9999.
    Gate MUST fail-closed if K < K_threshold: p-value resolution is coarser than alpha,
    making significant/non-significant verdict unreliable.

  Implementation assertion (Python reference):
    assert isinstance(C, int) and C >= 0
    assert isinstance(K, int) and K >= 1
    p_value = (C + 1) / (K + 1)
    assert 1/(K+1) <= p_value <= 1.0


MINIMUM K FOR ADEQUATE POWER (gate configuration):

  Resolution floor: K >= 1/(alpha) - 1. At alpha=0.05, K >= 19 (absolute minimum).
  Standard practice (sklearn default): K=100 (adequate for detection, not for exact alpha).
  Recommended for production gates: K >= 999 for alpha=0.05.
  For high-confidence rejection of null: K >= 4999.
  Sources: UW Biostatistics SISG lectures (Rice & Lumley), sklearn n_permutations=1000 example.
  Gate rule: if K < ceil(1/alpha) - 1, FAIL-CLOSED with "insufficient permutations for
    stated alpha; p-value resolution 1/(K+1) > alpha".


RELATIONSHIP TO MULTIPLE TESTING (H-17 N-deflation):

  When N strategies/parameter sets are searched and the best is selected:
    Each strategy is tested at level alpha individually.
    Family-wise error rate (FWER) inflates: P(at least one false positive) ~ N * alpha.
    Naive permutation p-value is NOT corrected for selection across N trials.

  Bonferroni correction: apply threshold alpha_corrected = alpha / N per test.
    Conservative but valid for independent tests.
    For correlated strategies, Bonferroni over-corrects.

  Deflated Sharpe / Bailey-Lopez de Prado (2014, arXiv:1402.3665):
    Adjusts Sharpe threshold upward as function of N trials, T observations, skewness, kurtosis.
    This is the preferred correction for financial strategy search.
    Formula: SR* = SR * sqrt((1 - skew*SR + ((kurt-1)/4)*SR^2) * (T/N))  (approximate).
    ELAI H-22 already uses this; permutation p-value COMPOSES with it as follows:
      Step 1: apply permutation test to get raw p-value p_raw.
      Step 2: compare p_raw to deflated alpha = alpha / N (Bonferroni) or use
        Deflated-Sharpe threshold directly instead of the permutation p-value.
      Combined gate: PASS only if p_raw < alpha/N AND observed Sharpe > SR*.

  Caution: permutation tests assume exchangeability under the null. For time-series returns,
    naive iid label permutation breaks autocorrelation structure. Block permutation
    (permute blocks of consecutive returns) preserves autocorrelation better. The gate
    should use block permutation or circular block bootstrap for time-series data.


WHAT THE PERMUTATION TEST CANNOT PROVE (honest residue):

1. Out-of-sample robustness: a significant in-sample permutation p-value does NOT imply
   the strategy will perform out-of-sample. The test only rules out the specific null
   that the observed performance is entirely random given the in-sample data structure.

2. Absence of data leakage: if the data pipeline leaks future information into features,
   both the observed statistic AND the permuted statistics see the leaked data (since
   features are not permuted in Test 1). The null distribution is also inflated, and the
   test may still return low p-value. Leakage is not detected by Test 1.
   Test 2 (PermFeats) is more leak-sensitive but still not a direct leakage test.

3. Causality: statistical significance implies dependence, not causation. A spuriously
   correlated feature will pass the permutation test during periods of correlation.

4. Future market regime stability: the null is tested on historical data under a specific
   regime. Regime change invalidates the null distribution's relevance.

5. Overfitting via strategy selection: the permutation test on a single selected strategy
   ignores the search procedure that found it. Multiple-testing correction is REQUIRED
   when the strategy was selected from a pool (see above).

GATE SPECIFICATION FOR H-22 Rust implementation:

  Input fields:
    returns: Vec<f64>  -- the observed return series (or performance metric per bar)
    statistic_fn: fn(&[f64]) -> f64  -- computes the performance scalar (e.g. Sharpe)
    K: usize  -- number of permutations (must be >= K_min for stated alpha)
    alpha: f64  -- significance level (e.g. 0.05)
    K_min: usize  -- = ceil(1.0/alpha) - 1  -- fail-closed below this

  Outputs:
    observed_stat: f64  -- statistic_fn(&returns)
    null_distribution: Vec<f64>  -- K statistic values under permuted labels
    C: usize  -- count of null_distribution values >= observed_stat
    p_value: f64  -- (C + 1) as f64 / (K + 1) as f64
    significant: bool  -- p_value < alpha AND K >= K_min

  Invariants (testable assertions in Rust):
    assert!(K >= K_min, "insufficient permutations: p-value resolution {:.4} > alpha {:.4}", 1.0/(K+1) as f64, alpha);
    assert!(C <= K);
    let p = (C + 1) as f64 / (K + 1) as f64;
    assert!(p >= 1.0 / (K as f64 + 1.0) - f64::EPSILON);
    assert!(p <= 1.0 + f64::EPSILON);
    // Gate is fail-closed: if K < K_min, significant = false regardless of p
    if K < K_min { return GateVerdict::Block("under-powered null".into()); }

  Fail-closed rules:
    K < K_min -> BLOCK (not SKIP, not WARN)
    null_distribution.len() != K -> BLOCK (incomplete permutation run)
    NaN in null_distribution -> BLOCK
    observed_stat.is_nan() -> BLOCK


SQRT-LAW VERIFY (one-line verdict for H-22 CostMonotoneConcave relation):

UNCERTAIN -- DEFER locking CostMonotoneConcave at retail scale.

Grounding:
  Toth et al. 2011 (arXiv:1105.1694, Phys. Rev. X 1:021006) establishes sqrt-law for
  institutional metaorders. The correct paper does NOT have arXiv ID 1004.0425
  (that ID is a quantum walk paper -- a citation error in the H-22 task description).

  Bucci, Benzaquen, Lillo, Bouchaud (2019, PRL 122:108302, arXiv:1811.05230) establishes
  a crossover: market impact is LINEAR for trades x < ~0.1% of ADV and follows sqrt-law
  for x >= ~0.1% ADV. Retail-sized trades (single orders, <$10K in liquid US equities)
  typically fall in the LINEAR regime (x << 0.1% ADV for any mid-cap+ stock).

  Verdict: at retail position size, impact is more likely LINEAR (monotone, but NOT
  concave in the sqrt sense). CostMonotoneConcave (impact ~ sqrt(Q)) holds for
  institutional scale. For retail scale, CostMonotoneLinear is the better approximation.
  Locking CostMonotoneConcave as a metamorphic invariant for a retail-scale gate is
  NOT supported by the empirical literature. DEFER or gate behind a size-threshold check:
    if trade_size_fraction_adv < 0.001 { use linear; } else { use sqrt; }


COMPLEXITY:
  Time: O(K * T) where T = length of return series per permutation evaluation
  Space: O(K) for null distribution storage (or O(1) if streaming C count)
  Parallelism: K permutations are fully independent; embarrassingly parallel (rayon)
  No ordering constraints between permutations.


FAILURE MODES CITED IN LITERATURE:
  1. K too small: p-value resolution coarser than alpha; spurious significance.
  2. Using p=C/K (no +1): p=0 is possible; conservative bound violated; FWER issues.
  3. iid label permutation on autocorrelated time-series: null distribution is wrong;
     test is anticonservative (too many false positives). Use block permutation instead.
  4. Permuting labels without re-fitting the model: only valid if model was fit on same data.
     For walk-forward / out-of-sample evaluation, permute before re-fitting per fold.
  5. Treating Test 1 significance as proof of out-of-sample edge (it is not).
  6. Not correcting for multiple testing when strategy was selected from N candidates.
  7. Applying sqrt-law cost model at retail scale: linear regime applies below ~0.1% ADV.


IMPLEMENTATION PITFALLS (DO NOT DO THIS):
  Do not compute p = C / K (missing +1 in both places).
  Do not return p=0 or treat p=0 as a valid output.
  Do not use SKIP or WARN if K < K_min; use BLOCK (fail-closed).
  Do not permute the full feature matrix for Test 1 (that is Test 2 semantics).
  Do not use the permutation p-value as a substitute for walk-forward OOS validation.
  Do not apply the gate to a strategy selected from N>1 without N-deflation.
  Do not use iid permutation for return series with known autocorrelation structure.
  Do not assume arXiv:1004.0425 is the Toth market impact paper -- it is not.
  Do not assume CostMonotoneConcave holds at retail scale -- it holds at institutional scale.


SOURCES VERIFIED:
  Ojala & Garriga 2010: https://jmlr.csail.mit.edu/papers/volume11/ojala10a/ojala10a.pdf
  sklearn implementation (Test 1): https://scikit-learn.org/stable/modules/generated/sklearn.model_selection.permutation_test_score.html
  Phipson & Smyth +1 correction: https://pmc.ncbi.nlm.nih.gov/articles/PMC6405018/
  Toth et al. 2011 (correct ID): https://arxiv.org/abs/1105.1694
  Bucci et al. 2019 crossover: https://arxiv.org/abs/1811.05230
