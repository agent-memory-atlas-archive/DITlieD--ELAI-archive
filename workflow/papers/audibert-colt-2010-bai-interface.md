Algorithm: Best Arm Identification in Multi-Armed Bandits (BAI)
Paper: Best Arm Identification in Multi-Armed Bandits
Authors: Jean-Yves Audibert, Sebastien Bubeck, Remi Munos
Venue: COLT 2010 (23rd Annual Conference on Learning Theory)
URL: https://www.learningtheory.org/colt2010/papers/59Audibert.pdf
Local text: F:\.ELAI_workflow\workflow\papers\audibert-colt-2010-bai.txt
Session: s-25120
Scope: D-5_bai_thompson_promotion

---

PROBLEM FORMULATION

Setting: fixed-budget pure exploration. NOT fixed-confidence (fixed-confidence optimizes pulls for
a given delta; this paper fixes budget n and minimizes probability of error en).

K arms with unknown reward distributions nu_1, ..., nu_K, rewards in [0,1].
At each round t = 1..n: forecaster chooses arm I_t, receives reward from nu_{I_t}.
After n rounds: forecaster outputs recommendation J_n.

Regret: rn = mu* - mu_{J_n}  (gap between optimal mean and recommended arm mean).
Error:  en = P(J_n != i*)     (probability of wrong recommendation).
Relationship: Delta_{i*} * en <= E[rn] <= en.

Unique optimal arm i* assumed (mu_{i*} = mu* = max_k mu_k).
Gap of arm i: Delta_i = mu* - mu_i  for i != i*.
Gap of optimal arm: Delta_{i*} = min_{i != i*} Delta_i  (minimal gap, Delta_{(2)}).
Order statistics notation: (i) denotes i-th best arm, so Delta_{(1)} <= Delta_{(2)} <= ... <= Delta_{(K)}.

---

COMPLEXITY MEASURES (Section 2, Eq. 1)

H_1 = sum_{i=1}^{K} 1/Delta_i^2
    where sum includes i* with Delta_{i*} = min_{j != i*} Delta_j

H_2 = max_{i in {1,...,K}} i * Delta_{(i)}^{-2}
    where Delta_{(i)} is the gap of the i-th best arm (ordered by gap ascending)
    equivalently: H_2 = max_{i=2}^{K} i / Delta_{(i)}^2  (the i=1 term is Delta_{(1)}^{-2} = Delta_{i*}^{-2} dominated)

Relationship (Eq. 1, proven in Section 6.1):
    H_2 <= H_1 <= log_bar(K) * H_2
    where log_bar(K) = 1/2 + sum_{i=2}^{K} 1/i   (harmonic-ish constant, ~log(K)+0.5)

Tightness: Right equality holds when Delta_{(i)} = sqrt(i)*c for some c. Left equality when all gaps equal.

Intuition: H_1 ~ total sample complexity needed across all arms. H_2 ~ instance-dependent hardness
capturing the "most difficult discrimination" (the arm index where i/Delta_i^2 is maximized).

---

INPUTS (both algorithms)

n   | int  | scalar | budget (total number of arm pulls), mandatory
K   | int  | scalar | number of arms, mandatory, K >= 2
    | Note: rewards assumed in [0,1]; sub-Gaussian with parameter 1/2 implied by Hoeffding use

---

OUTPUTS (both algorithms)

J_n | int  | scalar in {1,...,K} | recommended arm index
    | Invariant: J_n is argmax of empirical means at round n (ties broken arbitrarily)

---

ALGORITHM 1: UCB-E (Figure 2, Section 3)

Parameter:
    a > 0  (exploration parameter)
    Optimal value: a = (25/36) * (n-K) / H_1
    Constraint: a <= (25/36) * (n-K) / H_1 for Theorem 1 to hold
    Note: H_1 is NOT observable; requires oracle knowledge or estimation

Pseudocode:
    Initialize: For i in {1,...,K}: B_{i,0} = +inf
    For each round t = 1, 2, ..., n:
        Draw I_t in argmax_{i} B_{i, T_i(t-1)}
        where B_{i,s} = X_hat_{i,s} + sqrt(a/s)  for s >= 1
    J_n = argmax_{i} X_hat_{i, T_i(n)}

UCB index: B_{i,s} = empirical_mean(arm i after s pulls) + sqrt(a/s)

Theorem 1 (Section 3, p.4):
    If a <= (25/36)*(n-K)/H_1, then:
        en <= 2*n*K*exp(-2*a/25)
    Optimal setting a = (25/36)*(n-K)/H_1 gives:
        en <= 2*n*K*exp(-(n-K)/(18*H_1))
    Bound is exponentially decreasing in n, rate proportional to 1/H_1.

Side effect: each suboptimal arm i pulled ~a/Delta_i^2 times w.h.p.
Failure mode: UCB-E requires knowing H_1 to set a optimally. Underestimating H_1 -> over-exploration
of suboptimal arms. Overestimating H_1 -> under-exploration, best arm may stop being drawn.

---

ALGORITHM 2: SR (Successive Rejects, Figure 3, Section 4)

Parameter: NONE (parameter-free, reward-scale-independent)

Definitions:
    log_bar(K) = 1/2 + sum_{i=2}^{K} 1/i
    n_0 = 0
    For k in {1, ..., K-1}:
        n_k = floor( (1/log_bar(K)) * (n-K)/(K+1-k) )
    A_1 = {1, ..., K}  (active arm set)

Pseudocode:
    For each phase k = 1, 2, ..., K-1:
        (1) For each arm i in A_k: pull arm i for (n_k - n_{k-1}) additional rounds
        (2) A_{k+1} = A_k \ { argmin_{i in A_k} X_hat_{i, n_k} }
            (remove arm with lowest empirical mean; ties broken randomly)
    J_n = unique element of A_K

Budget proof: sum_{k=1}^{K-1} (K+1-k)*(n_k - n_{k-1}) <= n
    Verified: 1-arm at phase 1 gets n_1 pulls, ..., 2-arms at phase K-1 get n_{K-1} pulls.
    Total pulls = n_1 + ... + n_{K-1} + n_{K-1} <= n  (from log_bar definition)

For K=2: SR reduces to uniform allocation (n/2 pulls each), optimal.

Theorem 2 (Section 4, p.5):
    en <= K*(K-1)/2 * exp(-(n-K)/(log_bar(K)*H_2))
    Proof via Hoeffding + union bound over K*(K-1)/2 error events.

Theorem 3 (Section 4, p.6) - Sampling lower bound:
    With prob >= 1 - K^3/2 * exp(-(n-K)/(4*log_bar(K)*H_2)):
        T_j(n) >= (n-K) / (4*log_bar(K)*H_2*Delta_j^2)  for all arms j
    Dismissed arm at phase k satisfies:
        (1/4)*Delta_{(K+1-k)} <= (1/2)*Delta_{dismissed} <= gap_at_dismissal <= (3/2)*Delta_{dismissed} <= 3*Delta_{(K+1-k)}

Implication: n = Theta(log_bar(K) * H_2) samples needed for constant success prob.
SR achieves O(H_2 * log_bar(K)) = O(H_1 * log(K)) effectively.

---

ALGORITHM 3: Adaptive UCB-E (Figure 4, Section 7 - no theorem, experimental only)

Exploration rate c > 0 (practical parameter).
Uses time points t_k (where SR would dismiss an arm) to build online estimate H_hat_{1,k}.
H_hat_{1,k} = max_{K-k+1 <= i <= K} i * Delta_hat_{<i>}^{-2}
a = c * n / H_hat_{1,k}  used in UCB-E between t_k and t_{k+1}.
WARNING: no theoretical optimality guarantee; justified by experiments only.

---

LOWER BOUND (Section 5, Theorem 4)

Theorem 4 (Lower Bound):
    Let nu_1,...,nu_K be Bernoulli distributions with parameters in [p, 1-p], p in (0,1/2).
    For ANY forecaster, there exists a permutation sigma of {1,...,K} such that the permuted
    problem has error:
        en >= exp( -(5+o(1))*n / (p*(1-p)*H_2) )
    where o(1) -> 0 as n -> inf (depends only on K, p, n).

Interpretation: even with oracle knowledge of distributions up to permutation, no forecaster
can achieve error below exp(-c*n/H_2). UCB-E achieves exp(-c*n/H_1) = exp(-c*n/(log(K)*H_2))
within a log(K) factor. SR achieves exp(-c*n/(log_bar(K)*H_2)), matching lower bound up to
a constant (log_bar(K) vs 1/(p*(1-p))).

KL bound used in proof: KL(Ber(p), Ber(q)) <= (p-q)^2 / (q*(1-q))

Gap to optimality: log_bar(K) factor between SR upper bound and lower bound.
Open problem cited: whether the log factor can be removed (Problem (iii) in Conclusion).

---

CRITICAL INVARIANTS (testable assertions)

# 1. H_2 <= H_1 (Eq. 1)
assert H_2 <= H_1 + 1e-10, "H_2 must not exceed H_1"

# 2. H_1 <= log_bar(K) * H_2 (Eq. 1)
import math
log_bar_K = 0.5 + sum(1.0/i for i in range(2, K+1))
assert H_1 <= log_bar_K * H_2 + 1e-10, "H_1 must be <= log_bar(K)*H_2"

# 3. SR total pulls do not exceed budget n
total_pulls = sum((K+1-k) * (n_k - n_prev_k) for k, n_k, n_prev_k in phase_data)
assert total_pulls <= n, "SR must not exceed budget n"

# 4. SR dismisses exactly one arm per phase (|A_k| decreases by 1)
for phase_k in range(1, K):
    assert len(A[phase_k+1]) == len(A[phase_k]) - 1, "SR must remove exactly one arm per phase"

# 5. J_n is from active set (last surviving arm)
assert len(A_K) == 1, "SR final set must contain exactly one arm"
assert J_n == list(A_K)[0], "J_n must be the unique survivor"

# 6. UCB-E exploration parameter bound (Theorem 1 condition)
assert a <= (25.0/36.0) * (n - K) / H_1, "a exceeds Theorem 1 validity range"

# 7. H_1 includes all K arms (including i* with Delta_{i*} = min gap)
assert abs(H_1 - sum(1.0/delta_i**2 for delta_i in all_deltas)) < 1e-10

---

COMPLEXITY

UCB-E:
    Time per round: O(K)  (argmax over K UCB indices)
    Total time: O(n*K)
    Space: O(K)  (empirical means + pull counts)
    Parallelism: rounds are sequential (arm choice depends on previous pulls)

SR:
    Time: O(K^2) total  (K-1 phases, each phase pulls (K+1-k) arms for n_k-n_{k-1} rounds + one argmin)
    More precisely: O(n) total pulls + O(K^2) for phase bookkeeping
    Space: O(K)
    Parallelism: within a phase, pulls of different active arms can be parallelized

log_bar(K) computation: O(K)
n_k computation: O(K) total precomputation

---

FIXED-BUDGET vs FIXED-CONFIDENCE DISTINCTION

Fixed-budget (this paper's primary focus):
    Input: budget n (fixed)
    Output: J_n (arm recommendation)
    Metric: minimize en = P(J_n != i*)
    Algorithms: UCB-E, SR

Fixed-confidence (NOT this paper's primary contribution; mentioned as contrast):
    Input: confidence delta (fixed)
    Output: J + stopping time tau
    Metric: minimize E[tau] subject to P(error) <= delta
    Algorithms: Hoeffding Race, Even-Dar et al. (2006) - cited as prior work
    Note: racing algorithms "try to identify the best action at a given confidence level
          while consuming the minimal number of pulls" (Section 1)

Key contrast: "They optimize the budget for a given confidence level, instead of optimizing
the quality of the recommendation for a given budget size." (Section 1)

---

FAILURE MODES (cited in paper)

1. UCB-E with a = O(log n): equivalent to UCB1, gives polynomial NOT exponential error decay.
   (Section 3: "taking a of order log n is inappropriate to reach exponentially small probability of error")

2. UCB-E with a > (25/36)*(n-K)/H_1: no guarantee; may degrade to uniform allocation behavior.
   (Section 3: Theorem 1 does not cover this case)

3. Overestimating H_1 for UCB-E tuning: best arm may stop being drawn, biased estimate never corrected.
   (Section 3: "overestimating H_1 leads to low exploring")

4. Underestimating H_1 for UCB-E tuning: suboptimal arms drawn too much, best arm not identified.
   (Section 3: "underestimating H_1 leads to draw too much the suboptimal arms")

5. Uniform allocation: error ~ exp(-c*n*Delta_{i*}^2/K), which is exp(-c*n/(K*Delta_{i*}^{-2})).
   Substantially worse than SR/UCB-E when H_1 << K*Delta_{i*}^{-2}.
   (Section 3: "performs much worse than UCB-E ... potentially H_1 << K*Delta_{i*}^{-2}")

6. K=1 or non-unique optimal arm: paper assumes unique i*, K>=2. Behavior undefined otherwise.
   (Section 2: "we will assume ... there is a unique optimal arm")

---

IMPLEMENTATION PITFALLS (do NOT do this)

1. Do NOT use a = c*log(n) for UCB-E; this gives polynomial not exponential error reduction.

2. Do NOT confuse H_1 (sum over all K arms) with H_2 (max over i of i*Delta_{(i)}^{-2}).
   H_1 sums ALL arms; H_2 is a max. H_1 is used in UCB-E; H_2 governs SR and the lower bound.

3. Do NOT include i* in H_2 calculation with Delta_{i*}=0; use the order statistic form
   max_{i=1}^{K} i*Delta_{(i)}^{-2} where Delta_{(1)} = Delta_{i*} > 0 (smallest nonzero gap).
   Note: when i=1 in H_2 formula, Delta_{(1)} = Delta_{i*} (min gap of a suboptimal arm).

4. Do NOT discard H_1 contribution from i*: paper defines Delta_{i*} = min_{j!=i*} Delta_j,
   so the i* term in H_1 = 1/Delta_{i*}^2 = 1/(min_gap)^2. This IS included.

5. Do NOT implement SR to pull arms sequentially within a phase; pull each active arm the
   SAME number of times (n_k - n_{k-1}) per phase before dismissing.

6. Do NOT use log(K) (natural log) for log_bar(K); log_bar(K) = 1/2 + sum_{i=2}^{K} 1/i
   (a harmonic number variant, NOT the natural logarithm of K).

7. Do NOT adapt phase lengths dynamically in SR (that would make it Adaptive UCB-E, which
   has no theoretical guarantee). Standard SR phase lengths are fixed at precomputed n_k.

8. Do NOT implement fixed-confidence stopping in SR; SR always runs exactly n rounds.

9. Do NOT omit the floor operation in n_k: n_k = floor((1/log_bar(K))*(n-K)/(K+1-k)).
   Omitting floor can push total pulls above n.

10. Do NOT use UCB-E's greedy index for the final recommendation; always recommend
    argmax of empirical means at the END of n rounds, not the last arm pulled.
