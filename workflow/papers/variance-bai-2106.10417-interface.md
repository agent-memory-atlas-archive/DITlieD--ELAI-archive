Algorithm: Variance-Dependent Best Arm Identification (VD-BESTARMID)
ArXiv: 2106.10417
Paper: Variance-Dependent Best Arm Identification
Authors: Pinyan Lu, Chao Tao, Xiaojin Zhang
Venue: UAI 2021 (37th Conference on Uncertainty in Artificial Intelligence)
Version used: arXiv:2106.10417v3 (27 May 2023)
Session: s-25120
Scope: D-5_bai_thompson_promotion

Inputs:
n           | int          | scalar      | n >= 2; number of arms indexed 1..n
delta       | float        | scalar      | 0 < delta < 0.1 (Theorem 1.3 and 1.2 require delta <= 0.1); failure probability
arm_rewards | distribution | [0,1]-supp  | each arm i has unknown dist D_i supported on [0,1], mean theta_i, variance sigma_i^2
arm_order   | assumption   | implicit    | theta_1 > theta_2 >= ... >= theta_n (unknown to algorithm, not an input)

Outputs:
best_arm    | int          | scalar in [1..n]   | the index of the arm with highest mean theta_1
guarantee   | prob         | >= 1-delta         | probability that returned arm is indeed the best arm (delta-correct)

H_var complexity measure (the variance-dependent hardness):
H_var = sum_{i=1}^{n} (sigma_i^2 / Delta_i^2 + 1 / Delta_i)
where Delta_i = theta_1 - theta_i for i >= 2, and Delta_1 = Delta_2 (convention for best arm)
Note: H_var is NOT a named symbol in the paper; the paper uses the sum expression directly.
This differs from variance-independent H_1 = sum_{i=2}^{n} 1/Delta_i^2 (Successive Elimination bound, eq 1)
and from worst-case H_2 = n/Delta_min^2. H_var replaces the constant-1 in the sigma^2 coefficient with sigma_i^2.

Sample complexity (upper bound, Theorem 5.1 / Theorem 1.1):
O( sum_{i=1}^{n} (sigma_i^2/Delta_i^2 + 1/Delta_i) * (ln(1/delta) + ln ln(1/Delta_i)) )
= O( H_var * (ln(1/delta) + ln ln(1/Delta_min)) )  [informal, per-arm log-log terms differ]
Eq (3) in paper: the i=1 term uses Delta_1 = Delta_2.

Lower bound (Theorem 1.3 / Theorem H.1):
Omega( sum_{i=1}^{n} (sigma_i^2/Delta_i^2 + 1/Delta_i) * ln(1/delta) )
Conditions: sigma_i^2 < 0.1, 0 < Delta_i < 0.1, delta < 0.1.
Gap to upper bound: only doubly-logarithmic (ln ln 1/Delta_i) terms.

Algorithm hierarchy (4 levels, Section structure):
Level 4 (main): VD-BESTARMID(n, delta)           -- Algorithm 7
Level 3 (helper): BESTARMEST(S, epsilon, delta)   -- Algorithm 4
Level 2 (core): ITERELIM(S, epsilon, delta)       -- Algorithm 5
Level 1 (primitive): GROUPELIM(S, epsilon, delta) -- Algorithm 6
Support: MEANEST(i, epsilon, delta)               -- Algorithm 3
Support: VAREST(i, delta, ell)                    -- Algorithm 1
Support: VARTEST(i, tau, delta, c)                -- Algorithm 2

Algorithm 7: VD-BESTARMID(n, delta)
Input: arm set S = [n], confidence level delta
r <- 1; S_1 <- S
while |S_r| > 1:
    epsilon_r <- 1 / 2^(r+2)
    delta_r <- (1 / (2*r^2)) * delta
    for each i in S_r: theta_hat_r_i <- MEANEST(i, epsilon_r/2, delta_r/18)
    a_r  <- BESTARMEST(S_r, epsilon_r/2, delta_r/18)
    a*_r <- BESTARMEST(S_r \ {a_r}, epsilon_r/2, delta_r/18)
    if |theta_hat_r_{a_r} - theta_hat_r_{a*_r}| > 2*epsilon_r: Output a_r  [early stopping]
    S_{r+1} <- S_r \ {i in S_r | theta_hat_r_i < theta_hat_r_{a_r} - epsilon_r}
    r <- r + 1
Output: remaining arm in S_r
Termination: r = O(ln(1/Delta_2)) rounds (Lemma 5.4)

Algorithm 6: GROUPELIM(S, epsilon, delta)  [grouped median elimination - core novelty]
Input: arm set S, accuracy epsilon, confidence delta
N <- ceil(log2(2/epsilon))   [number of variance buckets]
for each i in S: sigma_hat_i^2 <- VAREST(i, delta/(2*N^2), epsilon)
bucket B_hat_j <- {i in S | 2^(-j) < sigma_hat_i^2 <= 2^(-j+1)}, j = 1..N
T <- empty set
for j = 1 to N:
    if |B_hat_j| >= 2:
        theta_hat_i <- MEANEST(i, epsilon/2, delta/(9*N)) for all i in B_hat_j
        m_hat_j <- median of {theta_hat_i : i in B_hat_j}
        T_j <- B_hat_j \ {i : theta_hat_i < m_hat_j}  [eliminate below-median half]
        T <- T union T_j
    else:
        put arm in B_hat_j into recycle bin R
Output: T and R
Guarantee (Theorem 4.2):
  (a) |R| = O(ln(1/epsilon))
  (b) sum_{a in T}(sigma_a^2 + epsilon) <= (255/256) * sum_{a in S}(sigma_a^2 + epsilon)  [multiplicative reduction]
  (c) |theta_{(T union R)[1]} - theta_{S[1]}| <= epsilon  [best arm preserved in T union R]
  (d) O(sum_{i in S}(sigma_i^2/epsilon^2 + 1/epsilon)(ln(1/delta) + ln ln(1/epsilon))) samples

Algorithm 5: ITERELIM(S, epsilon, delta)
beta <- sqrt(255/16) * e^0.001
epsilon_r <- beta^r * (1-beta) * epsilon  for r >= 0
delta_r <- e^(-0.1*r) * (1-e^(-0.1)) * delta  for r >= 0
T_0 <- S; R_0 <- empty; r <- 0
while |T_r| > 10:
    <T_{r+1}, R_{r+1}> <- GROUPELIM(T_r, epsilon_r, delta_r)
    R_{r+1} <- R_r union R_{r+1}
    r <- r + 1
Output: T_r union R_r
Guarantee (Theorem 4.3):
  (a) |T| = O((ln|S|)^2 * ln(1/epsilon))
  (b) |theta_{T[1]} - theta_{S[1]}| <= epsilon
  (c) O(sum_{i in S}(sigma_i^2/epsilon^2 + 1/epsilon)(ln(1/delta) + ln ln(1/epsilon))) samples

Algorithm 4: BESTARMEST(S, epsilon, delta)
S_1 <- ITERELIM(S, epsilon/3, delta/3)
if epsilon^(-1) <= ln|S|: S_2 <- S_1
else: S_2 <- ITERELIM(S_1, epsilon/3, delta/3)
a <- NAIVEBESTARMEST(S_2, epsilon/3, delta/3)
Output: arm a
Guarantee (Theorem 4.1): outputs arm a with |theta_a - theta_{S[1]}| <= epsilon
Sample complexity: O(sum_{i in S}(sigma_i^2/epsilon^2 + 1/epsilon)(ln(1/delta) + ln ln(1/epsilon)))

Algorithm 3: MEANEST(i, epsilon, delta)
sigma_hat_i^2 <- VAREST(i, delta/2, epsilon)
sample arm i for ceil(8*sigma_hat_i^2/epsilon^2 + 2/(3*epsilon)) * ln(4/delta) times
Output: empirical mean theta_hat_i
Guarantee (Lemma 2.3): |theta_hat_i - theta_i| <= epsilon with prob >= 1-delta
Sample complexity: O((sigma_i^2/epsilon^2 + 1/epsilon) * ln(1/delta))

Algorithm 1: VAREST(i, delta, ell)   [ell > 0, precision parameter]
for r = 1, 2, 3, ...:
    tau_r <- 1/2^r
    if tau_r <= ell OR VARTEST(i, tau_r, delta/e, 80): Output tau_r; break
Guarantee (Lemma 2.2):
  (a) tau > ell/2 always; sample complexity O((1/ell) * ln(1/delta))
  (b) if sigma_i^2 in (ell, 1]: with prob >= 1-delta, tau in [sigma_i^2/4, 2*sigma_i^2)
      sample complexity O((1/sigma_i^2) * ln(1/delta))
  (c) if sigma_i^2 <= 2*ell: Pr[tau >= x] = O(delta * 2^(-20*rm)) for x >= max(2*ell, 2*sigma_i^2)
  rm = ceil(|log2(sigma_i^2 / tau)|) is logarithmic mistake ratio

Algorithm 2: VARTEST(i, tau, delta, c)   [c >= 1]
T <- (c/tau) * ln(1/delta)
sample arm i for 2T times: x_1, ..., x_{2T}
sigma_hat_i^2 <- (1/T) * sum_{r=1}^{T} (x_r - x_{r+T})^2 / 2   [paired-difference estimator]
if sigma_hat_i^2 > tau: Output true  else: Output false
Sample complexity: 2c/tau * ln(1/delta)
Guarantee (Lemma 2.1): if sigma_i^2 >= 2*tau, outputs true with prob >= 1 - delta*(tau/sigma_i^2)^c
                        if sigma_i^2 <= tau/2, outputs false with prob >= 1 - delta*(sigma_i^2/tau)^c

Concentration inequalities used (Appendix A):
- Bernstein Inequality (Proposition A.2): for i.i.d. X_i with |X_i| <= M a.s., mean mu, variance sigma^2:
  Pr[|X_bar - mu| > t] <= 2 * exp(-n*t^2/2 / (sigma^2 + M*t/3))
  Used to set sample count in MEANEST: n = (8*sigma^2/epsilon^2 + 2/(3*epsilon)) * ln(4/delta)
- Multiplicative Chernoff bound (Proposition A.1): used in variance estimation analysis
Reward distribution: supported on [0,1], so M = 1 in Bernstein; variance sigma_i^2 in [0, 0.25]

Hyperparameters:
delta             | 0.05 typical  | (0, 0.1]         | high sensitivity -- lower delta increases all log factors
epsilon (internal)| 1/2^(r+2)     | decreases each round | not set by user; determined by round schedule
beta (ITERELIM)   | sqrt(255/16)*e^0.001 | fixed constant | determines geometric decay of epsilon schedule
c (VARTEST)       | 80            | >= 1              | controls confidence exponent in variance test; hardcoded to 80 in VAREST calls
N_buckets         | ceil(log2(2/epsilon)) | depends on epsilon | number of variance buckets in GROUPELIM
delta budget split| delta_r = delta/(2*r^2) per round | geometric series | sum converges; r^2 schedule chosen for convergence

Variant with bounded expected complexity:
VD-BESTARMID*(n, delta): runs interleaved copies A_i = VD-BESTARMID(n, delta/2^i)
Required: delta <= 0.1 (Theorem 1.2)
Expected sample complexity: O(H_var * (ln(1/delta) + ln ln(1/Delta_min)))

Critical invariants (testable assertions):
assert delta > 0 and delta <= 0.1, "delta must be in (0, 0.1] per Theorems 1.2, 1.3"
assert all(sigma_sq >= 0 and sigma_sq <= 0.25 for sigma_sq in sigmas), "variances of [0,1]-supported arms bounded by 0.25"
assert all(Delta > 0 for Delta in deltas[1:]), "all sub-optimal arms must have strictly positive gap"
assert abs(delta_1 - delta_2) < 1e-12, "convention: Delta_1 is set equal to Delta_2"
# GROUPELIM variance reduction per round:
assert sum_T_variances_plus_eps <= (255/256) * sum_S_variances_plus_eps, "Theorem 4.2(b) multiplicative reduction"
# VARTEST paired-difference estimator:
sigma_hat_sq = sum((x[r] - x[r+T])**2 for r in range(T)) / (2*T)
# Bernstein sample count in MEANEST (with estimated variance):
n_samples = int(8*sigma_hat_sq/eps**2 + 2/(3*eps)) * math.ceil(math.log(4/delta))
assert n_samples >= 1, "at least one sample required"
# ITERELIM terminates when active set shrinks to <= 10 arms
assert len(T_final) <= 10 or all_rounds_exhausted
# VD-BESTARMID stopping condition:
assert abs(theta_hat_a_r - theta_hat_a_star_r) > 2 * epsilon_r  # triggers early termination

Complexity:
Time (sample complexity): O(H_var * (ln(1/delta) + ln ln(1/Delta_min)))  [Theorem 5.1]
Space: O(n) active arm set + O(log(1/epsilon)) buckets per GROUPELIM call
Rounds in VD-BESTARMID: O(ln(1/Delta_2)) rounds before termination (Lemma 5.4)
GROUPELIM bucket count: N = ceil(log2(2/epsilon)) buckets per call
ITERELIM rounds: geometric decay; converges in O(log(sum sigma_i^2 / epsilon)) rounds
Parallelism: arms within a bucket in GROUPELIM are independent -- parallelizable
Best arm problem structure: no parallelism in the outer round loop (sequential elimination)

Comparison with prior work (from paper Section 1.1 and 1.2):
- Even-Dar et al. 2002 Successive Elimination: O(sum_{i>=2} Delta_i^{-2} (ln(1/delta) + ln n + ln ln Delta_i^{-1}))
  VD-BESTARMID achieves O(sum (sigma_i^2/Delta_i^2 + 1/Delta_i)(...)) -- replaces Delta_i^{-2} with sigma_i^2/Delta_i^2 + 1/Delta_i
  Advantage when sigma_i^2 << 1 (low-variance arms); VD-BESTARMID also removes the ln n factor on best arm
- Karnin et al. 2013 / Jamieson et al. 2014 lil'UCB: O(sum_{i>=2} Delta_i^{-2} (ln(1/delta) + ln ln Delta_i^{-1}))
  Eq (1): same dependency on Delta but no variance exploitation
- lil-KLUCB (Tanczos et al. 2017): uses KL divergence; still has ln(n/delta) term on best arm
  VD-BESTARMID removes ln n factor from best arm term (key contribution)
- Example 1 (Bernoulli arms theta_i = 1 - i/n): VD-BESTARMID = O(n ln n (ln(1/delta) + ln ln n));
  gap-only algorithms: Omega(n^2 ln(1/delta)) -- quadratic vs near-linear improvement

Failure modes:
1. delta > 0.1: lower bound (Theorem 1.3) and expected complexity (Theorem 1.2) proofs require delta < 0.1; behavior outside this range is unproven
2. Reward distribution not supported on [0,1]: Bernstein bound (Proposition A.2) requires bounded support; algorithm breaks for heavy-tailed or unbounded rewards without modification
3. Non-unique best arm (theta_1 = theta_2): convention Delta_1 = Delta_2 handles tie between best and second-best, but guarantees degrade when gap is exactly 0 (Delta_i -> 0 makes H_var -> infinity)
4. Large constant factors: authors explicitly warn (Section 6) "empirical performance suffers from the large constant factors introduced by multiple subroutines" -- the algorithm is theoretically optimal but practically expensive
5. Recycle bin arms (R in ITERELIM): arms placed in recycle bin R are excluded from future GROUPELIM rounds but re-included in the ITERELIM output; forgetting to include R in the final output breaks correctness guarantee (Theorem 4.2(c))
6. Variance estimate errors: if VAREST returns tau far from sigma_i^2, arms may be placed in wrong buckets; the "bad bucket" analysis (Lemma 4.11, 4.12) bounds this but requires the full confidence-parameter threading to be correct

Implementation pitfalls (do NOT do this):
1. Do NOT use a single unified confidence budget across all subroutine calls -- each level has its own fractional delta budget (delta/3, delta_r/18, delta/(9N), etc.); mixing budgets breaks union-bound proofs
2. Do NOT implement median elimination without variance bucketing (GROUPELIM) -- naive median elimination fails to achieve O(H_var) because it cannot guarantee the total variance reduces by a constant fraction per round when arm variances are non-uniform (Section 4 explicitly states this failure mode)
3. Do NOT stop ITERELIM when the best arm is identified with high probability -- ITERELIM runs until |T_r| <= 10 (a size threshold), not until convergence on arm identity; early stopping breaks the sample complexity bound
4. Do NOT substitute empirical standard deviation for the paired-difference variance estimator in VARTEST -- the paired estimator (x_r - x_{r+T})^2 / 2 is specifically designed for the concentration analysis; a naive sample variance would change the constant factors
5. Do NOT skip the second ITERELIM call in BESTARMEST when epsilon^{-1} > ln|S| -- this conditional double-elimination step is required to drive |S_2| down to O((ln ln|S|)^2) so the NAIVEBESTARMEST call does not reintroduce an ln|S| factor
6. Do NOT set the VARTEST parameter c lower than 1 or omit the c=80 hardcoding in VAREST -- c controls the tail decay of the variance estimate error; c=80 is chosen to make the error probability O(delta * 2^{-20*rm}) per Lemma 2.2, which is needed for the sample complexity telescoping
7. Do NOT use Delta_1 = 0 for the best arm -- set Delta_1 = Delta_2 per the paper's convention; otherwise the H_var sum diverges for the best arm term
8. Do NOT implement VD-BESTARMID and assume bounded expected complexity -- Theorem 1.1/5.1 gives only high-probability bound; for bounded expected complexity use VD-BESTARMID* (Algorithm 8, Theorem 1.2) which requires delta <= 0.1
