Algorithm: DS-TS (Discounted Thompson Sampling for Non-Stationary Bandits)
ArXiv: 2305.10718
Paper: Discounted Thompson Sampling for Non-Stationary Bandit Problems
Authors: Han Qi, Yue Wang, Li Zhu
Submitted: May 18, 2023; revised May 22, 2023

Inputs:
K | int | number of bandit arms | K >= 2; mandatory
T | int | time horizon | T >= 1; mandatory
gamma | float | discount factor | (0, 1); controls how fast past rewards are forgotten; mandatory
prior_mu | float per arm | Gaussian prior mean | any real; default 0.0
prior_sigma | float per arm | Gaussian prior std | > 0; default 1.0
reward_observations | sequence | real-valued rewards per round | generated online; mandatory

Outputs:
arm_selection a_t | int in [0, K-1] | arm chosen at each round t | one per timestep
cumulative_regret R_T | float | sum of gaps between optimal and chosen arm rewards | monitored for convergence
posterior_params | (mu_t, sigma_t) per arm | discounted Gaussian posterior | updated each round

Hyperparameters:
gamma | no universal default; paper tunes per regime | (0, 1) | high sensitivity: gamma too close to 1 = slow forgetting; too close to 0 = ignores history; paper: regime-specific tuning required
prior_sigma | 1.0 | (0, inf) | moderate sensitivity; wider prior = more exploration early

Critical invariants (testable assertions):
assert 0 < gamma < 1, "discount factor must be strictly in (0,1)"
assert regret_abrupt <= C * sqrt(T * B_T) * log(T), "Theorem: abrupt-change regret is O~(sqrt(T*B_T)) where B_T = number of breakpoints"
assert regret_smooth <= C * T**beta * log(T), "Theorem: smooth-change regret is O~(T^beta) where beta depends on smoothness class"
assert all(sigma_t > 0 for _, sigma_t in posteriors), "discounted posterior variance must remain positive"
assert sum(gamma**(t-s) * I(s<=t) for s in range(t)) > 0, "discounted sufficient statistics must be positive"

Complexity:
Per-round time: O(K) for sampling + O(K) for posterior update = O(K) total
Space: O(K) for per-arm posteriors (2 floats each)
Parallelism: arm posterior updates are independent; all K posteriors can update in parallel

Failure modes:
gamma near 1 in rapidly-changing environment: slow forgetting causes high regret on abrupt changes
gamma near 0 in slowly-changing environment: over-discounting destroys information; near-random behavior
No change-point detection: DS-TS is passive (no active detection); breakpoint-adaptive algorithms outperform it when B_T is large and detectable
Prior misspecification (non-Gaussian rewards): Gaussian TS with non-Gaussian rewards loses Bayesian correctness; theoretical bounds rely on Gaussian model
T >> K requirement: regret bounds assume sufficient exploration; very short horizons may not satisfy concentration conditions

Implementation pitfalls (do NOT do this):
Do not use gamma = 1.0 (reduces to standard stationary Thompson Sampling, loses non-stationarity tracking)
Do not use gamma = 0.0 (ignores all history, degenerates to pure random)
Do not apply a single fixed gamma across all deployment environments; tune per change-rate
Do not confuse B_T (number of breakpoints) with T (total rounds) when computing theoretical regret
Do not use Bernoulli/Beta conjugate update if rewards are continuous Gaussian -- breaks posterior form
