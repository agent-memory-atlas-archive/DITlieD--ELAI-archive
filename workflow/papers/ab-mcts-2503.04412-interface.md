Algorithm: AB-MCTS — Adaptive Branching Monte Carlo Tree Search
ArXiv: 2503.04412
Paper: Wider or Deeper? Scaling LLM Inference-Time Compute with Adaptive Branching Tree Search

Inputs:
prompt | text | task prompt | coding or ML competition task
score_evaluator | external function | f(answer) -> float | REQUIRED; test case execution for coding, validation loss for ML; algorithm cannot function without this
model | LLM | any frontier LLM | used to generate candidate responses at each node

Outputs:
best_answer | text | highest-scored candidate | selected via Thompson Sampling from posterior
search_tree | tree | nodes = (prompt + partial solution, score_list) | Bayesian posteriors maintained per node

Hyperparameters:
bayesian_prior_type | normal-inverse-chi2 or Beta | AB-MCTS-A uses conjugate priors for closed-form updates
ab_mcts_m_mcmc_samples | int | used in AB-MCTS-M variant with mixed-effects model | computationally heavier

Critical invariants (testable assertions):
assert external_score_evaluator_required == True  # paper: "we assume there is some accessible score evaluator"
assert intrinsic_logprob_not_used_as_score == True  # paper never mentions mean_logprob or cosine as value function
assert thompson_sampling_drives_selection == True  # Bayesian posteriors converted to selections via Thompson Sampling
assert bayesian_update_per_node == True  # each node's posterior updated as scores accumulate

Complexity:
time | O(branching_factor * depth * score_evaluator_cost) | score evaluator cost dominates
space | O(nodes in tree) | posterior parameters stored per node

Failure modes:
Algorithm is undefined without an external score evaluator — using intrinsic signals (logprob, cosine) is a deviation from the paper's assumption and untested in the paper.
When score distribution range varies significantly, Bayesian priors may need tuning.
Authors acknowledge: "developing such an evaluator itself can be challenging depending on the task."

Implementation pitfalls (do NOT do this):
Do NOT use mean_logprob + cosine as a drop-in replacement for the score evaluator and claim AB-MCTS paper support — the paper never uses these signals.
Do NOT claim Bayesian posteriors validate intrinsic scoring — the Bayesian layer is only used to improve node selection given external scores.
Do NOT apply to tasks without reliable external feedback (open-ended generation) without documenting the deviation from the paper's assumptions.
The PRM-lite substitution (auditor-as-verifier) is a deviation: document it as such and cite what the paper actually requires.
