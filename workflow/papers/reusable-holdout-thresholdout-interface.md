Algorithm: Thresholdout (Reusable Holdout) + Ladder Leaderboard Mechanism
ArXiv:
  Paper 1 (foundations):  1411.2664  -- Dwork, Feldman, Hardt, Pitassi, Reingold, Roth -- "Preserving Statistical Validity in Adaptive Data Analysis" (STOC 2015)
  Paper 2 (Thresholdout): 1506.02629 -- Dwork, Feldman, Hardt, Pitassi, Reingold, Roth -- "Generalization in Adaptive Data Analysis and Holdout Reuse" (NIPS 2015)
  Paper 3 (Ladder):       1502.04585 -- Blum, Hardt -- "The Ladder: A Reliable Leaderboard for Machine Learning Competitions" (ICML 2015)

Local PDFs:
  F:\.ELAI_workflow\.claude\papers\1411.2664.pdf
  F:\.ELAI_workflow\.claude\papers\1506.02629.pdf
  F:\.ELAI_workflow\.claude\papers\1502.04585.pdf


SECTION 1 -- THRESHOLDOUT ALGORITHM (source: 1506.02629 Figure 1, Section 4.1)

Inputs:
  St       | dataset  | training set of size m, drawn i.i.d. from P | mandatory
  Sh       | dataset  | holdout set of size n, drawn i.i.d. from P | mandatory
  T        | float64  | base threshold (e.g. 0.01 practical; T = 3*tau/4 in Theorem 25) | mandatory
  sigma    | float64  | Laplace noise rate parameter (= tau / (96 * ln(4*m_queries/beta)) in Theorem 25) | mandatory
  B        | uint64   | overfitting budget: max number of above-threshold (holdout-consulting) responses | mandatory
  phi      | function | query function phi: X -> [0,1], submitted adaptively by analyst | per-query

Outputs per query phi:
  answer   | float64  | either E_St[phi] (below-threshold path) or E_Sh[phi] + xi (above-threshold path) | in [0,1]
  OR "bottom" symbol if B < 1

Invariants on output:
  assert answer in [0.0, 1.0] or answer == BOTTOM_SYMBOL
  assert budget_decrements_total <= B_initial
  assert budget_decrements_total == count(above_threshold_responses)


SECTION 2 -- EXACT THRESHOLDOUT PSEUDOCODE (verbatim from 1506.02629 Figure 1)

Initialization (once before query loop):
  sample gamma ~ Lap(2 * sigma)
  T_hat <- T + gamma

Per query phi:
  (a) if B < 1: output BOTTOM (exhausted)
  (b) else:
      i.  sample eta ~ Lap(4 * sigma)
      ii. if |E_Sh[phi] - E_St[phi]| > T_hat + eta:   [ABOVE THRESHOLD -- budget charged]
              sample xi ~ Lap(sigma)
              sample gamma ~ Lap(2 * sigma)
              B <- B - 1
              T_hat <- T + gamma          [re-randomise threshold]
              output E_Sh[phi] + xi
          else:                                          [BELOW THRESHOLD -- free response]
              output E_St[phi]

Budget charge condition (CRITICAL):
  B is decremented IF AND ONLY IF |E_Sh[phi] - E_St[phi]| > T_hat + eta.
  This is the empirical holdout-train gap exceeding the noisy threshold.
  In terms of true distribution: each budget decrement corresponds to a query where
  |P[phi] - E_St[phi]| > tau/2 (Theorem 25 "overfitting counter" Zi definition).


SECTION 3 -- NOISE DISTRIBUTIONS AND EXACT LAPLACE SCALE PARAMETERS

All noise is Laplace (symmetric exponential).  Lap(scale) has PDF f(x) = exp(-|x|/c) / (2c)
where c = scale.  Mean = 0.  Variance = 2 * scale^2.

gamma (threshold noise, added to T to form T_hat):
  Distribution: Lap(2 * sigma)
  Scale parameter c_gamma = 2 * sigma
  Purpose: randomises the threshold so an adversary cannot probe it exactly.
  Re-sampled every time B is decremented (each above-threshold response).

eta (per-query comparison noise):
  Distribution: Lap(4 * sigma)
  Scale parameter c_eta = 4 * sigma
  Purpose: adds noise to the per-query threshold comparison.
  Sampled fresh for every query.

xi (answer noise, added to holdout mean when above-threshold):
  Distribution: Lap(sigma)
  Scale parameter c_xi = sigma
  Purpose: privatises the released holdout mean.
  Sampled only on above-threshold (budget-decrementing) responses.

Relationship between sigma and target accuracy tau (Theorem 25):
  sigma = tau / (96 * ln(4 * m_queries / beta))
  where m_queries is total query count, beta is failure probability.

Privacy guarantee (Lemma 23, 1506.02629):
  Pure DP:     epsilon = 2*B / (sigma * n),  delta = 0
  Approx DP:   epsilon = sqrt(32*B*ln(2/delta)) / (sigma*n),  delta > 0
  These are the (epsilon, delta)-DP parameters achieved by Thresholdout over holdout Sh.

NOISE SCALE SUMMARY -- LOCKED CONSTANTS FOR IMPLEMENTATION:
  c_gamma = 2 * sigma
  c_eta   = 4 * sigma
  c_xi    = sigma
  sigma   = tau / (96 * ln(4 * m_queries / beta))   [from Theorem 25]


SECTION 4 -- BUDGET B: MEANING, BOUND, AND EXHAUSTION

Budget B is the number of times the holdout Sh may be consulted for an above-threshold
(overfitting-detected) answer before Thresholdout stops responding.

Budget interpretation:
  B counts the number of queries phi_j where |P[phi_j] - E_St[phi_j]| > tau/2
  i.e., queries where the training set is genuinely overfitting to the true distribution.
  Non-overfitting queries (below threshold) are FREE -- they do not consume budget.

Budget bound (Theorem 25, 1506.02629):
  The holdout remains valid (responses are tau-accurate) for up to m_queries total queries,
  with at most B of them being above-threshold, provided:

  n >= min( n0(B, sigma, tau/8, beta/(2*m)), n1(B, sigma, tau/8, beta/(2*m)) )

  where:
    n0(B, sigma, tau, beta) = max{ 2*B / (sigma*tau),  ln(6/beta) / tau^2 }
    n1(B, sigma, tau, beta) = 80 * sqrt(B * ln(1/(tau*beta))) / (tau * sigma)

  Simplified leading order with sigma = tau/(96*ln(4*m/beta)):
    n >= O( ln(m/beta) / tau^2 ) * min( B,  sqrt(B * ln(ln(m/beta)/tau)) )

  Exponential query capacity:
    m_queries can be exponential in n provided B grows sub-quadratically in n,
    i.e., B <= n^(2-c) for some constant c > 0.
    (Stated explicitly after Theorem 25 remark: "m can be exponentially large in n
    as long as B grows sub-quadratically in n.")

Budget exhaustion (HARD LIMIT -- HONEST LIMIT):
  Once B is decremented to 0, Thresholdout outputs BOTTOM for all subsequent queries.
  The holdout Sh CANNOT be refreshed in-place; it MUST be replaced with FRESH data
  drawn independently from P.  Re-labelling or re-splitting the same underlying sample
  does NOT restore validity.  This is the honest exhaustion limit: after B overfit queries
  the differential privacy guarantee is spent and the holdout is no longer reusable.


SECTION 5 -- VALIDITY / GENERALIZATION GUARANTEE (Theorem 25, 1506.02629)

Setting: T = 3*tau/4, sigma = tau/(96*ln(4*m/beta)), holdout size n as above.

Guarantee: For every i in [m_queries]:
  P[ exists i : Z_i < B AND |a_i - P[phi_i]| >= tau ] <= beta

where Z_i = |{j <= i : |P[phi_j] - E_St[phi_j]| > tau/2}| is the true-overfitting counter.

Meaning: as long as the true overfitting count stays below B, every answer a_i is within
tau of the true expectation P[phi_i], with probability >= 1 - beta.

Tolerance scaling: tau ~ 1/sqrt(n) (up to log factor), same as non-adaptive setting.

Key asymmetry: Thresholdout only "corrects" overfitting toward the holdout when the
above-threshold condition fires.  The below-threshold path returns E_St[phi], which may
still be tau/2-off from P[phi] due to finite sample variance on St -- this is acceptable
because that error is no worse than using St alone.


SECTION 6 -- LAPLACE MECHANISM IN PAPER 1411.2664 (foundational context)

Section 5.1 of 1411.2664 gives the standalone Laplace mechanism for answering m adaptive
statistical queries.  Thresholdout is built on top of this via the Sparse Vector Algorithm.

Theorem 14 (Laplace mechanism, 1411.2664):
  Answer each query phi_i with E_S[phi_i] + Lap(0, sigma_lm).
  Under (epsilon, 0)-DP:   n >= C * m * log(1/beta) / (epsilon * tau)
  Under (epsilon, delta)-DP: n >= C * sqrt(m * log(1/delta)) * log(1/beta) / (epsilon * tau)

Theorem 9 (generalization from epsilon-DP, 1411.2664):
  If A is epsilon-DP and n >= 12*ln(4/beta) / tau^2 and epsilon <= tau/2:
  P[ |P[phi] - E_S[phi]| > tau ] <= beta
  This is the core transfer theorem used in Thresholdout's proof.


SECTION 7 -- LADDER ALGORITHM (source: 1502.04585 Section 3, Figure 1)

The Ladder is the leaderboard analog of Thresholdout.  It addresses a simpler goal:
maintain an accurate running-minimum loss estimate across adaptively submitted classifiers.

Inputs:
  S        | dataset  | holdout set of size n drawn i.i.d. from D | mandatory
  eta      | float64  | step size (rounding granularity) > 0 | mandatory (or use param-free variant)
  f_t      | function | classifier f: X -> Y, submitted adaptively | per-round

Outputs per round t:
  R_t      | float64  | current best leaderboard loss estimate, in [0,1] | mandatory
  Invariant: R_t is non-increasing across rounds (Ladder only moves the score down)
  assert R_t <= R_{t-1} for all t

Pseudocode (Figure 1, 1502.04585):
  R_0 <- infinity
  for each round t = 1, 2, ...:
    receive f_t
    if R_S(f_t) < R_{t-1} - eta:
        R_t <- round_to_multiple_of_eta( R_S(f_t) )
    else:
        R_t <- R_{t-1}
    output R_t

where R_S(f_t) = (1/n) * sum_{i=1}^{n} loss(f_t(x_i), y_i)   [empirical loss on holdout]
and round_to_multiple_of_eta(x) means floor(x/eta)*eta or nearest-multiple.

Ladder step size (optimal, Theorem 3.1):
  eta = O( n^{-1/3} * log^{1/3}(k*n) )
  Optimal leaderboard error: O( log^{1/3}(k*n) / n^{1/3} )
  where k = number of submissions.

Ladder error bound (Theorem 3.1, exact form):
  For any sequence of k adaptively chosen classifiers, for all t <= k and epsilon > 0:
  Pr[ |min_{i<=t} R_D(f_i) - R_t| > epsilon + eta ] <= exp(-2*epsilon^2*n + B_tree + 1)

  where B_tree = (1/eta + 2) * log(4*t/eta)

  The description-length bound on the adaptive tree:  |T| <= 2^{B_tree}
  (Claim 3.2: at most 1/eta steps can ever lower R, so the tree has size 2^{(1/eta+2)*log(4t/eta)})

Lower bound (Theorem 3.3):
  No estimator can achieve leaderboard error < Omega( sqrt(log(k)/n) ).
  Gap between upper bound O((log(kn)/n)^{1/3}) and lower bound Omega(sqrt(log(k)/n)) is open.

Parameter-free Ladder (Figure 2, 1502.04585):
  Uses a one-sided paired t-test instead of fixed eta.
  Step condition: R_S(f_t) < R_{t-1} - s/sqrt(n)
  where s = std(loss_vector_t - loss_vector_{t-1})  [sample std of per-sample loss differences]
  This corresponds to ~0.15 significance level.
  Output is rounded to nearest multiple of 1/n.

Budget analog for Ladder:
  At most ceil(1/eta) steps lower the leaderboard (monotone steps in [0,1] at resolution eta).
  For eta = n^{-1/3}: at most n^{1/3} score updates in the entire competition.
  This is the Ladder's implicit "budget" -- it is structurally enforced, not tracked explicitly.


SECTION 8 -- HYPERPARAMETERS AND SENSITIVITY

Thresholdout hyperparameters:
  T (threshold base) | typical: T = 3*tau/4 (Theorem 25); practical: 0.01-0.04 (experiments) | range (0, 0.5)
  sigma (noise rate)  | tau/(96*ln(4*m/beta)) from theory; governs all three Laplace scales | high sensitivity: sets privacy budget spent per query
  B (budget)          | problem-dependent; B <= n^2 for exponential query capacity | must be set before interaction starts; cannot be increased mid-session
  tau (target accuracy)| end-to-end accuracy goal | range (0, 0.5); scales inversely with sqrt(n)

Authors note (Section 5 experiments): practical parameters T=0.04, sigma via Gaussian noise
at standard deviation ~0.01 work well empirically and prevent overfitting even though they
are below the theoretically required levels.  Gaussian noise was substituted for Laplace in
the experiment (stronger concentration).  Theoretical guarantees are for Laplace only.

Ladder hyperparameters:
  eta (step size)  | O(n^{-1/3} * log^{1/3}(kn)) from theory; or parameter-free via t-test | range (0, 1); governs leaderboard granularity and overfitting resistance
  n (holdout size) | must be fixed before competition | governs accuracy floor ~n^{-1/3}


SECTION 9 -- CRITICAL TESTABLE INVARIANTS

# Thresholdout noise scale invariants (all scales relative to sigma):
assert abs(c_gamma - 2 * sigma) < 1e-12
assert abs(c_eta   - 4 * sigma) < 1e-12
assert abs(c_xi    -     sigma) < 1e-12

# Budget is non-increasing:
assert budget_after <= budget_before  # for every query

# Budget is charged only on above-threshold responses:
assert budget_decremented == (abs(E_Sh_phi - E_St_phi) > T_hat + eta_sample)

# Below-threshold path returns training mean exactly (no noise added):
assert answer == E_St_phi  # when NOT above threshold

# Above-threshold path returns noisy holdout mean:
assert abs(answer - (E_Sh_phi + xi_sample)) < 1e-12  # when above threshold

# Holdout mean estimate error bound (Lemma 24 n0 condition):
# P[|P[phi] - E_S[phi]| >= tau] <= beta  when n >= max(2*B/(sigma*tau), ln(6/beta)/tau^2)
assert n >= max(2*B / (sigma * tau), math.log(6/beta) / tau**2)

# Ladder monotonicity:
assert R_t <= R_prev  # for every round; the leaderboard only moves down

# Ladder step fires only when empirical improvement exceeds eta:
assert (R_t < R_prev) == (R_S_ft < R_prev - eta)

# Ladder description-length budget (implicit):
# Number of score updates ever <= ceil(1/eta)
assert total_score_updates <= math.ceil(1.0 / eta)

# Budget exhaustion honest limit:
# After B decrements: Thresholdout MUST NOT answer further queries from the same Sh
assert B_remaining == 0 implies output == BOTTOM_SYMBOL


SECTION 10 -- COMPUTATIONAL COMPLEXITY

Thresholdout:
  Per query: O(n) to compute E_Sh[phi] and E_St[phi] for a 1-sensitivity function phi: X->[0,1]
  Space: O(n) for holdout + O(1) state (T_hat, B, last gamma)
  Parallelism: queries must be processed sequentially (B is shared mutable state)
  No batching of queries: each query sees the current T_hat and B

Ladder:
  Per submission: O(n) to compute empirical loss on holdout
  Space: O(n) for holdout loss vector of previous best (needed for paired t-test variant)
  Parallelism: rounds must be sequential (R_{t-1} depends on previous round)


SECTION 11 -- FAILURE MODES (from papers)

Thresholdout failure modes:
  1. Budget B exhausted: once B=0, Thresholdout outputs BOTTOM -- no more answers.
     Fix: fresh holdout required; cannot re-use same Sh.
  2. Sigma too large: answers become too noisy to be useful (xi noise dominates).
  3. Sigma too small: differential privacy epsilon too large, generalization breaks down.
  4. T too small: excessive budget consumption on queries that are not truly overfitting.
  5. T too large: fails to detect genuine overfitting; analyst gets biased E_St answers.
  6. Holdout not i.i.d.: theoretical guarantees apply only to i.i.d. samples from P.
  7. delta > 0 path: approximate DP generalization (Theorem 8, 1506.02629) requires n >= 2*ln(8/delta)/epsilon^2; if violated, guarantees degrade.

Ladder failure modes:
  1. eta too small: leaderboard updates too frequently; exponential tree grows; guarantees weaken.
  2. eta too large: leaderboard too coarse; misses real improvements.
  3. n too small: floor accuracy ~n^{-1/3} is not useful.
  4. Kaggle-style mechanism (release exact score): vulnerable to boosting attack achieving bias Omega(sqrt(k/n)) after k submissions (Corollary 5.2).
  5. Parameter-free variant is a heuristic: t-test is invalid when f_t depends on S; no formal guarantee for the significance threshold 0.15.


SECTION 12 -- IMPLEMENTATION PITFALLS (DO NOT DO THESE)

Thresholdout pitfalls:
  1. DO NOT add noise to below-threshold responses (training mean path must be exact).
  2. DO NOT re-use gamma across queries without re-sampling -- T_hat must be freshly randomised after each budget decrement (the pseudocode re-samples gamma inside the above-threshold branch).
  3. DO NOT charge the budget when the noisy condition |E_Sh - E_St| > T_hat + eta fails (eta is freshly sampled per query; the condition uses the freshly sampled eta, not a fixed eta).
  4. DO NOT set sigma = tau directly -- the correct scale is sigma = tau/(96*ln(4*m/beta)); setting sigma = tau overshoots and burns privacy budget.
  5. DO NOT refresh the holdout Sh with re-labelled or permuted versions of the same data after B is spent -- fresh i.i.d. samples from P are required.
  6. DO NOT use Gaussian noise and claim Laplace DP guarantees -- Gaussian gives (epsilon, delta)-DP not pure DP; the noise scales differ.
  7. DO NOT skip re-randomising T_hat on each above-threshold event -- the pseudocode explicitly re-samples gamma and updates T_hat inside branch A.B.

Ladder pitfalls:
  1. DO NOT release the raw empirical loss on every submission (Kaggle mechanism) -- this is vulnerable to the boosting attack.
  2. DO NOT update R_t when the improvement is exactly eta (condition is strict: R_S(ft) < R_{t-1} - eta, not <=).
  3. DO NOT omit rounding to the nearest eta multiple -- without rounding the description-length argument breaks.
  4. DO NOT use the parameter-free t-test variant as a formal proof ingredient -- it is a heuristic; the formal guarantee requires fixed eta.
  5. DO NOT interpret the Ladder's leaderboard accuracy guarantee as a per-query accuracy guarantee -- it only bounds the running minimum estimate, not individual query estimates.


SECTION 13 -- RELATIONSHIP TO H-19 LOCKED CONSTANTS

The ELAI H-19 SAFEDELTA gate uses Thresholdout to validate whether benchmark query answers
are overfitting to the holdout set.  The LOCKED constants from the papers are:

  THRESHOLDOUT_NOISE_GAMMA_SCALE_MULTIPLIER = 2   # c_gamma = 2 * sigma  (Lap(2*sigma))
  THRESHOLDOUT_NOISE_ETA_SCALE_MULTIPLIER   = 4   # c_eta   = 4 * sigma  (Lap(4*sigma))
  THRESHOLDOUT_NOISE_XI_SCALE_MULTIPLIER    = 1   # c_xi    =     sigma  (Lap(sigma))
  THRESHOLDOUT_SIGMA_DENOMINATOR_CONSTANT   = 96  # sigma = tau / (96 * ln(4*m/beta))
  THRESHOLDOUT_T_FRACTION_OF_TAU            = 3.0/4.0  # T = 3*tau/4 (Theorem 25)

These are not tunable -- they are the exact parameters proved correct in Theorem 25 of
1506.02629.  Any deviation changes the privacy budget and breaks the generalization guarantee.

For H-19 the budget B is set as a function of the holdout size n and target accuracy tau:
  B_max = floor(n^2 / (tau^2 * ln(m/beta)))   [informal: B <= n^{2-epsilon} for exponential m]

The holdout MUST be replaced with fresh data after B_max decrements.


SECTION 14 -- UNVERIFIED / NOT PINNED IN PAPERS

The following items are NOT directly stated as closed-form constants in the papers and
must be chosen conservatively by the executor:

  1. The exact constant C in n >= C * n0(B, sigma, tau/8, beta/(2m)) is stated as "sufficiently large" in Lemma 24 -- the papers do not pin it.  Conservative choice: C = 2.

  2. The n1 bound uses "sufficiently small beta and tau" -- the regime is not precisely specified.  Conservative choice: use n0 when in doubt.

  3. The Gaussian noise variant (used in Section 5 experiments with T=0.04, sigma~0.01) has no formal DP guarantee derived in the papers.  Do not conflate it with the Laplace analysis.

  4. The parameter-free Ladder t-test threshold of 0.15 significance is explicitly called "primarily heuristic" by the authors (Section 4.1).  Do not treat it as a formal bound.
