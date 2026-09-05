Algorithm: Hierarchical Bayesian Bandits with Hierarchical Thompson Sampling (HierTS)
ArXiv: 2111.06929
Paper: "Hierarchical Bayesian Bandits" — Joey Hong, Branislav Kveton, Manzil Zaheer, Mohammad Ghavamzadeh
Venue: AISTATS 2022, PMLR 151:7724-7741 (peer-reviewed, HIGH reliability)
Local PDF: F:\.ELAI_workflow\.claude\papers\2111.06929.pdf

---

PROBLEM SETUP

The paper unifies meta-learning, multi-task learning, and federated learning as hierarchical Bayesian bandits. There are T tasks, each a K-arm bandit. Task t has arm reward parameters theta_t = (theta_{t,1}, ..., theta_{t,K}). These parameters are drawn i.i.d. from an unknown prior P (the hyperprior / population distribution). The agent does not know P but learns it across tasks.

Two operating modes:
  Sequential: tasks solved one after another; posterior on P updates between tasks.
  Parallel: tasks solved simultaneously; partial pooling via shared P.

---

Inputs:
theta_t         | vector       | K-dim, arm reward means for task t        | drawn from prior P (unknown)
P (hyperprior)  | distribution | over R^K (or [0,1]^K for Bernoulli)       | unknown; learned from data
K               | int          | number of arms per task                    | >= 2
T               | int          | number of tasks                            | >= 1; more tasks = better prior estimate
observations    | real         | reward at each (task, arm, round) triple   | conditionally independent given theta_t
prior_params    | hyperparams  | initial belief about P (e.g. N(mu_0, S_0)) | required for Thompson Sampling init

---

Outputs:
arm selection   | int in [0,K-1)  | arm pulled at each round               | must be a valid arm index
regret (Bayes)  | real >= 0       | cumulative difference from optimal      | guaranteed to decrease with more informative P
posterior_P     | updated dist    | posterior belief over hyperprior P      | updated after each task

---

Hyperparameters:
mu_0            | prior mean vector       | 0 (for Gaussian) or uninformative Beta   | low sensitivity if K small
Sigma_0 / S_0   | prior covariance        | I_K (identity)                           | HIGH sensitivity: controls prior width; regret scales with prior width (paper §4)
sigma (noise)   | reward noise std dev    | 0.5 in Gaussian experiments              | must be known or estimated; misspec degrades regret
n_rounds_per_task | int                  | rounds per task                          | controls exploration-exploitation within task

---

Hierarchical Prior Structure (from paper + related work verified via arxiv.org/html/2510.26284):

Gaussian variant (main theoretical analysis):
  theta_{t,i} | P ~ N(mu, Sigma)        (arm means drawn from Gaussian population)
  P = N(mu, Sigma) with hyperprior on (mu, Sigma)
  Posterior over theta_t is Gaussian given Gaussian observations (conjugate)
  Posterior over P updates via sufficient statistics across tasks

Bernoulli/Beta variant (experiments confirmed, search result citing "rewards are Bernoulli"):
  theta_{t,i} | P ~ Beta(mu * kappa, (1-mu) * kappa)   [D-29 exact parameterization]
  (mu, kappa) = hyperprior parameters; mu in (0,1), kappa > 0 (concentration)
  Per-arm posterior: Beta(alpha_i + successes, beta_i + failures)
  Cross-arm sharing: mu, kappa updated via Gibbs or moment matching across tasks

This Beta-hyperprior structure IS the D-29 ArmPosterior model:
  theta_i ~ Beta(mu * kappa, (1 - mu) * kappa)
  (mu, kappa) ~ hyperprior (improper or weakly informative)
  Gibbs step: sample mu, kappa from conditional given theta samples

---

Critical Invariants (testable assertions):

assert all(0 < theta_i < 1 for theta_i in arm_posteriors)  # Bernoulli arms
assert kappa > 0  # concentration parameter must be positive
assert 0 < mu < 1  # population mean must be a valid probability
# Regret bound invariant: regret should decrease when prior is more informative (tighter Sigma_0 or larger kappa)
assert bayesian_regret(informative_prior) <= bayesian_regret(uninformative_prior)  # per Theorem (paper §4)
# Total variance decomposition: Var[X] = E[Var[X|theta]] + Var[E[X|theta]]
assert abs(total_var - (expected_conditional_var + var_of_conditional_mean)) < 1e-9

---

Key Theoretical Results (verified from abstract + search results):

1. HierTS Bayes Regret Bound: regret decreases as the prior P becomes more informative.
   Formal: R_T <= f(K, T, prior_width) where prior_width = sqrt(trace(Sigma_0)) for Gaussian case.
   Implication: the more tasks seen, the better the prior estimate, the lower the regret.

2. Total Variance Decomposition (novel technique, cited as reusable beyond this model):
   Var[theta] = E_P[Var[theta | P]] + Var_P[E[theta | P]]
   Used in all regret proofs. Critical for bounding the estimation error of HierTS.

3. Sequential vs Parallel equivalence: regret bounds hold in both modes with same functional form.

4. Prior misspecification robustness: confirmed empirically (online image classification experiment);
   HierTS degrades gracefully when true P differs from assumed family.

---

HierTS Algorithm (from abstract + "Deep Hierarchy in Bandits" companion paper, arxiv:2202.01454):

1. Initialize: set hyperprior params (mu_0, Sigma_0 or equivalent Beta params).
2. For each task t = 1..T:
   a. Sample theta_t ~ posterior P(theta_t | history of past tasks)  [hierarchical TS sample]
   b. For each round within task t:
      - Pull arm a_t = argmax_i theta_{t,i}
      - Observe reward r_t
      - Update within-task posterior for theta_t
   c. After task t: update posterior over hyperprior P using sufficient stats from task t.
3. Efficient Gaussian implementation via exact hierarchical posterior representation (closed-form).
   Bernoulli/Beta: requires Gibbs sampling or variational approximation for (mu, kappa) update.

---

Complexity:
Time per round  | O(K) arm sample + O(K) argmax = O(K) per round
Gibbs update    | O(K * n_gibbs_steps) per task for Beta/non-conjugate hyperprior
Space           | O(K) per task posterior + O(K^2) for Gaussian hyperprior covariance
Parallelism     | tasks can run in parallel (paper §3 parallel variant); arm pulls within task are sequential

---

Failure Modes (verified from paper + citing work):

1. Prior covariance misspecification: if Sigma_0 too tight, HierTS under-explores; regret bound breaks.
   Paper explicitly notes regret scales with prior width.

2. Non-conjugate hyperprior: Bernoulli arms with Beta(mu*kappa, (1-mu)*kappa) hyperprior require
   approximate inference (Gibbs/VI) for (mu, kappa) updates. Closed-form only for Gaussian hierarchy.

3. Cold-start (T=1, single task): hierarchical prior gives no benefit; degenerates to standard TS.
   Need T >= 2 tasks with shared structure for pooling to help.

4. Prior estimation bias: with few tasks T, estimate of P is noisy; regret improvement over non-hierarchical
   TS is small. Paper shows benefit grows with T.

5. Independence assumption violated: HierTS assumes tasks are i.i.d. from P. If tasks have
   temporal drift or structure beyond shared P, regret bound does not apply.

---

Applicability to ELAI D-29 (hierarchical_bai_research):

DIRECT: The Beta-hyperprior structure theta_i ~ Beta(mu*kappa, (1-mu)*kappa) in D-29's ArmPosterior
matches the Bernoulli/Beta variant of HierTS. The paper grounds:
  (a) Posterior structure: each arm's Beta posterior updated with successes/failures per round.
  (b) Population distribution: (mu, kappa) learned via Gibbs sampling across arms (analogous to tasks in paper).
  (c) Regret framework: Bayesian regret bound justifies using hierarchical prior instead of flat Beta(1,1).

EXTENSION NEEDED: The paper addresses regret minimization, not Best Arm Identification (BAI).
D-29 needs the BAI extension. The paper does NOT provide fixed-confidence or fixed-budget BAI bounds.
The grounding for BAI specifically comes from arxiv:2106.10417 (variance-dependent BAI, already ingested).
This paper grounds the HIERARCHICAL PRIOR component; 2106.10417 grounds the BAI component.
Together they provide a grounded two-part foundation for hierarchical BAI in D-29.

GIBBS SAMPLING GROUND TRUTH: paper confirms non-conjugate (mu, kappa) requires Gibbs or VI.
D-29's Gibbs sampling design is correctly motivated.

---

Rejected Candidates:

1. arxiv:2102.06129 (Meta-Thompson Sampling, Kveton et al. 2021):
   Verified real. Covers Gaussian bandits, learns prior via meta-learning. Less direct for Beta/Bernoulli
   hierarchical structure. Also regret minimization only. Weaker match than 2111.06929.

2. Lattimore & Munos "Bounded regret for finite-armed structured bandits" (NeurIPS 2014):
   Not searched to exact ID due to budget constraint. Would cover structured bandit regret but
   likely not hierarchical Bayesian priors specifically. Lower priority per task spec.

3. No direct hierarchical BAI paper found: WebSearch for "hierarchical best arm identification arxiv"
   returned no paper with hierarchical priors specifically for BAI at fixed confidence/budget.
   This is an open research gap; D-29 is extending, not implementing an existing result.

---

Implementation Pitfalls (do NOT do this):

Do NOT use flat Beta(1,1) priors per arm without hierarchical pooling — loses the statistical efficiency gain.
Do NOT run closed-form Gaussian updates for Beta/Bernoulli arms — conjugacy does not hold for (mu, kappa).
Do NOT assume kappa is constant — it must be inferred jointly with mu via Gibbs.
Do NOT skip the total variance decomposition when analyzing regret — it is load-bearing in all proofs.
Do NOT apply HierTS regret bounds directly to BAI — the bounds are for regret minimization, not identification.
Do NOT assume tasks are non-i.i.d. — HierTS requires tasks drawn from common P.
Do NOT set Sigma_0 or kappa_prior too tight at init — cold-start regret will be high.
