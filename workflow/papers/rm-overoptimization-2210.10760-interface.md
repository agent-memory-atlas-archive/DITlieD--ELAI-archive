Algorithm: Scaling Laws for Reward Model Overoptimization (Gao et al., ICML 2023)
ArXiv: 2210.10760
Paper: Scaling Laws for Reward Model Overoptimization
Authors: Leo Gao, John Schulman, Jacob Hilton (OpenAI)
Session: s-25120 / D-4 tripwire stack


Inputs:
d              | float  | scalar >= 0    | d = sqrt(D_KL(pi || pi_init)); mandatory for both functional forms
n              | int    | n >= 1         | best-of-n sample count; used to compute KL_bon analytically; mandatory for BoN
alpha_bon      | float  | > 0 (fit)      | BoN scaling coefficient; fit per RM size; mandatory for BoN form
beta_bon       | float  | > 0 (fit)      | BoN overoptimization coefficient; fit per RM size; mandatory for BoN form
alpha_RL       | float  | > 0 (fit)      | RL scaling coefficient; approximately constant across RM sizes (Section 3.2)
beta_RL        | float  | > 0 (fit)      | RL overoptimization coefficient; scales logarithmically with RM param count
N_RM           | int    | 3M to 3B       | proxy RM parameter count; governs coefficient scaling; optional (for prediction)
N_data         | int    | >= 2000        | RM training comparison count; below ~2000 RM is near-chance (Section 3.3)
kl_penalty     | float  | >= 0           | PPO KL penalty coefficient; only affects convergence speed, not KL-gold frontier (Section 3.6); optional, default 0 in paper experiments


Outputs:
R_bon(d)       | float  | real           | gold reward under BoN at KL distance d; R(0) = 0 by definition; peaks then declines
R_RL(d)        | float  | real           | gold reward under RL at KL distance d; R(0) = 0; peaks then declines; undefined at d=0 (log singularity)
KL_bon(n)      | float  | >= 0           | KL divergence for best-of-n with n samples; analytically exact
d_star_bon     | float  | > 0            | BoN peak KL distance; gold reward maximized at this d
d_star_RL      | float  | > 0            | RL peak KL distance; gold reward maximized at this d
R_max_bon      | float  | real           | maximum achievable gold reward under BoN; closed-form prediction
proxy_gold_gap | float  | real           | proxy_score - gold_score; grows as d increases past d*; primary observable for CorrelationDrop tripwire


Functional Forms (verbatim from paper, Introduction):
BoN:  R_bon(d) = d * (alpha_bon - beta_bon * d)
RL:   R_RL(d)  = d * (alpha_RL - beta_RL * log(d))
Note: RL form has infinite slope at d=0; does not hold near origin (footnote 1, Introduction)
Note: R(0) := 0 by definition for both forms

Derived Formulas (not in paper verbatim but mathematically exact from the above):
BoN peak:  d_star_bon = alpha_bon / (2 * beta_bon)
BoN max:   R_max_bon  = alpha_bon^2 / (4 * beta_bon)   [confirmed: Figure 12 caption cites this closed form]
RL peak:   d_star_RL  = exp((alpha_RL - 1) / beta_RL)

KL for BoN (Stiennon et al. 2020, Appendix G.3, cited in paper Section 2):
KL_bon(n) = log(n) - (n - 1) / n


Hyperparameters / Coefficient Scaling:
alpha_bon  | varies with RM size  | > 0          | scales smoothly (approx log) with RM param count (Figure 3a); sensitive to RM size
beta_bon   | varies with RM size  | > 0          | scales smoothly (approx log) with RM param count (Figure 3b); governs overoptimization speed; lower = larger RM (more robust)
alpha_RL   | approx constant      | > 0          | nearly independent of RM param count (Figure 3, footnote 2); captures regressional Goodhart (Section 4.2.1)
beta_RL    | varies with RM size  | > 0          | scales logarithmically with RM param count (Figure 3c); governs overoptimization speed
N_data     | >= 2000 required     | int          | below ~2000 comparisons RM is near-chance; scaling of alpha/beta with data less clean than with RM size (Section 3.3)
policy_size| weakly affects R(0)  | 1.2B or 6B   | larger policy has higher initial gold score but same overoptimization amount; weak dependence (Section 3.4)
kl_penalty | 0 recommended        | >= 0         | nonzero penalty strictly increases proxy-gold gap; acts as early stopping on KL frontier (Section 3.6)


Critical Invariants (testable assertions):

# 1. BoN gold reward peaks then declines (Section 3.1, Figure 1a)
# Given fitted coefficients, the peak must occur at a positive d
assert alpha_bon > 0 and beta_bon > 0, "coefficients must be positive for valid overoptimization curve"
assert alpha_bon / (2 * beta_bon) > 0, "BoN peak d* must be positive"

# 2. Max BoN gold reward closed form (Figure 12 caption)
# R_max_bon = alpha_bon^2 / (4 * beta_bon)
def R_bon(d, alpha_bon, beta_bon):
    return d * (alpha_bon - beta_bon * d)
assert abs(R_bon(alpha_bon / (2 * beta_bon), alpha_bon, beta_bon) - alpha_bon**2 / (4 * beta_bon)) < 1e-10

# 3. KL for BoN is analytic (Section 2, citing Stiennon 2020)
import math
def KL_bon(n):
    return math.log(n) - (n - 1) / n
assert KL_bon(1) == 0.0  # n=1 means no selection, zero KL
assert KL_bon(10) > 0

# 4. RL form is undefined / singular at d=0; do not evaluate at d <= 0
def R_RL(d, alpha_RL, beta_RL):
    assert d > 0, "RL form undefined at d=0 (log singularity, footnote 1)"
    return d * (alpha_RL - beta_RL * math.log(d))

# 5. Proxy-gold gap is non-negative at overoptimization (Section 3.5, Figure 8)
# proxy_score grows roughly linearly in sqrt(KL); gold score peaks and declines
# At d > d_star: proxy_score - gold_score > 0
# This gap is the primary observable for the CorrelationDrop tripwire (D-4)
# No closed form for proxy in paper; detect gap increase as tripwire signal

# 6. Regressional Goodhart identity (Equation 1, Section 4.2.1)
# E[X | X_hat = x_hat] = E[X] + (x_hat - E[X] - E[Z]) * Var(X) / (Var(X) + Var(Z))
# Alpha term = shortfall between proxy slope and gold slope = regressional Goodhart amount

# 7. Iterated RLHF gold score improvement (Section 4.3)
# R_RL_iterated(d, k) = d * (alpha_RL - beta_RL * log(d) + beta_RL * log(k))
# Benefit of k iterations: beta_RL * d * log(k)
def R_RL_iterated(d, k, alpha_RL, beta_RL):
    assert d > 0 and k >= 1
    return d * (alpha_RL - beta_RL * math.log(d) + beta_RL * math.log(k))
assert R_RL_iterated(d, 1, alpha_RL, beta_RL) == R_RL(d, alpha_RL, beta_RL)  # k=1 is single pass


Complexity:
BoN: O(n) policy rollouts at inference; O(1) per gold-score evaluation; KL analytic O(1)
RL: O(T) PPO steps; KL grows approx quadratically with step count in absence of penalty (Figure 16)
Fitting: nonlinear least squares on empirical (d, R_gold) pairs; O(S) per RM size S data points
Parallelism: BoN embarrassingly parallel; RL requires sequential policy updates


Failure Modes (verbatim or paraphrased from paper):
1. Adversarial Goodhart (Section 4.2.4): scaling laws may break down if policy is capable of manipulating the proxy; authors explicitly warn against extrapolation to highly capable systems
2. Synthetic-to-real transfer (Section 4.5): synthetic gold RM may not capture mismatch between labels and actual human intent; "main limitation of this work"
3. RL form near origin (footnote 1): R_RL(d) has infinite slope at d=0; form does not hold near origin; only valid for d > epsilon
4. KL as cross-method comparator (Section 4.1): KL is inadequate for comparing amount of optimization across RL and BoN; RL consumes far more KL than BoN for equivalent gold reward gain
5. KL penalty (Section 3.6): KL penalty does NOT improve the KL-gold frontier; it only causes early convergence; authors note result is "particularly sensitive to hyperparameters"
6. RM data below 2,000 comparisons (Section 3.3): RM is near-chance; functional forms and coefficients unreliable below this threshold
7. Proxy scores harder to model (Section 3.1, 3.2): proxy RM scores do not fit the same functional forms as gold; systematic underestimation at high KL
8. Policy size independence is surprising (Section 3.4): larger policies do NOT reduce overoptimization amount despite starting higher; do not assume bigger policy = safer RLHF
9. Validation loss proxy for robustness (Section 3.3): two RMs with equal validation loss may achieve equal robustness regardless of size/data combination; weak evidence only


Implementation Pitfalls (do NOT do this):
1. Do NOT evaluate R_RL(d) at d=0; the log singularity is undefined; always guard d > 0
2. Do NOT use KL distance to compare optimization amount across RL and BoN methods; they spend KL differently
3. Do NOT add a nonzero KL penalty expecting it to improve the gold-KL frontier; it only acts as early stopping
4. Do NOT assume the proxy reward score is a reliable guide to gold score beyond d*; proxy grows monotonically while gold declines
5. Do NOT treat the proxy-gold gap as zero at high KL; "systematic underestimates of the proxy reward model when extrapolated to higher KLs" (Section 3.2)
6. Do NOT extrapolate these scaling laws to adversarially capable systems (Section 4.2.4)
7. Do NOT fit with fewer than ~2,000 RM training comparisons; below this threshold RM is near-chance (Section 3.3)
8. Do NOT use alpha_RL as a scaling parameter with RM size; it is approximately constant (footnote 2)
9. Do NOT assume policy size controls overoptimization speed; proxy-gold gap is independent of policy size (Section 3.4)
10. Do NOT use the alternative RL forms (log(1+d) or power law) as they have worse extrapolation (Appendix B)


D-4 Tripwire Grounding (CorrelationDrop / Delta-rho signal):
The proxy-gold reward gap is the operative observable for the D-4 CorrelationDrop tripwire.
The paper does not define an explicit Pearson correlation rho formula.
The proxy-gold relationship is: proxy grows ~linearly in d; gold follows R_bon(d) or R_RL(d) and peaks at d*.
Observable signal: proxy_score - gold_score begins increasing past d_star (Figure 8, Section 3.5).
Monotone increase in this gap = overoptimization has occurred.
The rate of gap growth (d/d(KL) of proxy_gold_gap) is the analog of dρ/d(KL) referenced in D-4.
Threshold: no numerical threshold is stated in the paper. Authors only note the phenomenon.
Practical threshold recommendation (from paper structure): measure gap at baseline (d < 0.5), flag when
  gap exceeds 2x baseline value OR when gold score drops below initial gold score (R_gold(d) < R_gold(0)).
  These are implementation-level design choices not specified in the paper — label as ungrounded conjecture.
