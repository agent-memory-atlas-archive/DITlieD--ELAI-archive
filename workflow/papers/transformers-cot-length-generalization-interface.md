Algorithm: Transformer Chain-of-Thought Learning with Length Generalization
ArXiv: 2511.07378
Paper: Transformers Provably Learn Chain-of-Thought Reasoning with Length Generalization
Authors: Yu Huang, Zixin Wen, Aarti Singh, Yuejie Chi, Yuxin Chen
Venue: NeurIPS 2025 (full version)

PROBLEM

Prior expressivity results (Merrill & Sabharwal arXiv:2310.07923; Feng et al. arXiv:2402.12875) showed that transformers WITH CoT can escape the TC0 ceiling that bounds constant-depth transformers without CoT, reaching NC1-complete problems. However, those were expressivity / existential results: they did not prove that gradient descent on finite data actually LEARNS the NC1-capable solution. This paper provides the first OPTIMIZATION GUARANTEE (learnability by gradient descent with polynomial sample complexity) for constant-depth transformers on NC1-complete problems via CoT, and further proves length generalization: models trained on short sequences correctly solve longer ones.

METHOD

Task class: synthetic state-tracking tasks. The algebraic structure of the underlying state-tracking group governs both what can be solved and how far length generalization extends. (The paper studies group-composition / word-problem tasks; exact group families not confirmed from available sources but the analysis is parameterized by group algebraic properties.)

Three-part theoretical framework:
1. Gradient descent learnability: proves constant-depth transformers trained with gradient descent on polynomial sample size find a solution that correctly solves NC1-complete state-tracking problems when equipped with CoT. This is the "first optimization guarantee" going beyond TC0, contingent on TC0 != NC1 (widely believed conjecture).
2. Attention concentration mechanism: characterizes length generalization via a property of the trained attention layer called "attention concentration." Attention concentration links the retrieval robustness of the attention layer to the task structure of long-context state tracking. When attention concentrates appropriately (provably induced by training), the model generalizes to longer CoT sequences than seen during training.
3. Recursive self-training scheme: for transformers with insufficient reasoning length (CoT budget too short to solve a given problem length directly), a recursive self-training procedure provably extends the range of solvable problem lengths. The scheme progressively increases length coverage without retraining from scratch.

Experiments: broad set confirming (a) length generalization behavior matches theory and (b) attention concentration mechanism is observable and matches theoretical predictions.

INPUTS

state_tracking_sequence | discrete token sequence | length n (train), length n' > n (test for generalization) | tokens drawn from finite alphabet induced by the group
group_structure | algebraic specification (group G, operation table) | determines NC1 vs TC0 boundary for the task
transformer_architecture | constant-depth D, width W transformer | D and W are constants (not growing with input length); exact bounds not confirmed from available sources
training_data | (input, CoT-annotated output) pairs | polynomial sample count in n (specific polynomial not confirmed from available sources)
cot_length_budget | integer T | max intermediate reasoning tokens per inference; if T < required, recursive self-training required

OUTPUTS

learned_weights | transformer parameter tensor | the trained model that correctly solves NC1-complete state-tracking problems with CoT
attention_concentration_certificate | property of trained attention weights | provably induced by training; certifies length generalization will hold for inputs longer than training
length_generalization_bound | integer n_max | the maximum problem length the trained model can solve correctly; determined by algebraic structure of the group task and degree of attention concentration

HYPERPARAMETERS

depth D | constant (paper does not specify exact value in available sources) | must be constant (not growing with n); small constant sufficient per NC1 result | critical: increasing D does not substitute for CoT; the NC1 result depends on CoT providing serial computation, not depth
width W | constant per architecture | polynomial in task parameters | not confirmed sensitive
learning_rate | not confirmed from available sources | standard GD schedule assumed | not confirmed sensitive
sample_complexity n_train | polynomial in n (training length) | lower bound required for concentration guarantee | critical: below polynomial threshold attention concentration may not form
cot_steps T | integer, problem-dependent | must be >= ceil(log n) for NC1 problems up to length n | critical: too-short CoT budget requires recursive self-training to extend

CRITICAL INVARIANTS (testable assertions)

1. Without CoT: constant-depth transformers are confined to TC0 (Merrill-Sabharwal prior); any test measuring accuracy on NC1-complete tasks (e.g., simulating non-solvable-group automata, directed graph connectivity) WITHOUT intermediate tokens must show failure. assert acc_no_cot_nc1_task < chance + epsilon.

2. With CoT + sufficient training: accuracy on NC1-complete state-tracking tasks approaches 1.0 with polynomial training samples. assert acc_with_cot_nc1_task > 1 - delta for polynomial n_train.

3. Attention concentration increases monotonically during training on state-tracking tasks with sufficient data. assert attention_concentration_score(epoch_t+1) >= attention_concentration_score(epoch_t) - tolerance.

4. Length generalization: a model trained on sequences of length n correctly solves sequences of length n' > n IF attention concentration is achieved AND n' is within the algebraic bound determined by the group structure. assert acc(n_prime) > 1 - delta when attention_is_concentrated and n_prime <= algebraic_length_bound.

5. Recursive self-training monotonically extends solvable length: after k rounds of recursive self-training, the solvable length bound strictly increases. assert solvable_length_bound(round_k+1) > solvable_length_bound(round_k).

6. The NC1 result is contingent: if TC0 == NC1 (conjecture fails), the claimed separation collapses. Any empirical test must use tasks that are KNOWN to be NC1-hard but not known to be in TC0.

COMPLEXITY

Time per training step: O(N^2 * d) per layer per forward pass (standard self-attention), where N = sequence length including CoT tokens, d = embedding dimension. CoT multiplies effective sequence length by factor T, so inference cost is O((N+T)^2 * d) per step. This is the SAME asymptotic cost per token as standard self-attention -- CoT does NOT reduce per-step compute, it redirects compute into serial token generation.
Space: O(N * d) KV-cache per inference. With CoT of length T, O((N+T) * d).
Training sample complexity: polynomial in n (training length); exact polynomial not confirmed from available sources.
Parallelism: training is standard gradient descent on transformer; inference is autoregressive (each CoT token depends on previous), so CoT inference is SERIAL in CoT steps (cannot parallelize CoT generation).
Depth: constant D (does not scale with n). This is what makes the result non-trivial: a constant-depth network with polynomial training learns an NC1-complete problem via CoT.

PRIOR ART BOUNDARY (what this paper goes beyond)

Merrill & Sabharwal (2310.07923): transformers WITH CoT can EXPRESS (not learn) P/poly functions; without CoT confined to TC0. Expressivity only, no learning guarantee.
Feng et al. (2402.12875): CoT empowers transformers to solve inherently serial problems (NC1, NL, P-complete tasks). Expressivity + some training analysis on specific tasks. Does not provide a general optimization guarantee with length generalization.
This paper: FIRST optimization guarantee (gradient descent learns NC1-complete solution) + attention concentration mechanism explains WHY training induces length generalization + recursive self-training extends length coverage. Goes beyond TC0 strictly (under TC0 != NC1).

FAILURE MODES

1. Group structure outside the paper's algebraic characterization: if the state-tracking group does not match the algebraic assumptions, length generalization bound may not hold.
2. Insufficient training data (below polynomial threshold): attention concentration may not form; length generalization collapses.
3. CoT length budget T too small: model cannot solve problems beyond what T tokens permit; recursive self-training required, adding training-time cost.
4. TC0 == NC1 conjecture fails: the complexity separation that makes the NC1 result non-trivial collapses; result becomes a special case of TC0 expressibility.
5. Distribution shift between training and test groups: the algebraic structure governs generalization only within-distribution; out-of-distribution group tasks may not generalize.
6. Synthetic task gap: all theoretical and experimental results are on synthetic state-tracking tasks; application to natural language CoT reasoning is not proven and is a significant open question.

IMPLEMENTATION PITFALLS (do NOT do this)

Do NOT claim CoT reduces per-step compute: each CoT token still costs O(N^2 * d) attention. The benefit is serial depth, not per-step efficiency.
Do NOT conflate expressivity (Merrill-Sabharwal) with learnability: the NC1 result in prior work was existential; this paper proves learnability -- these are distinct claims.
Do NOT assume length generalization is free: it requires (a) polynomial training data, (b) verified attention concentration, and (c) target length within algebraic bound. No free lunch.
Do NOT interpret recursive self-training as solving arbitrary-length problems: each round extends the bound by a constant factor; very long problems require many rounds and the training cost scales.
Do NOT test the NC1 result on tasks that are in TC0: the test must use provably TC0-hard tasks to confirm the separation.
Do NOT skip the conjecture caveat: the headline result is conditional on TC0 != NC1. Implementations must note this. If the conjecture is eventually disproven, the NC1-separation result has no force.
Do NOT assume results transfer to LLMs: the theory is for constant-depth transformers on synthetic group tasks with gradient descent and exact CoT supervision. Production LLMs differ on all these axes.

STANCE VS HALLUCINATION-CEILING THESIS (arXiv:2507.07505)

This paper CONTRADICTS the strong form of the O(N^2*d) hallucination-ceiling thesis as applied to CoT. The thesis argues that each reasoning token costs another O(N^2*d) and buys nothing at the complexity level: that CoT is cosmetic repetition within TC0. This paper proves the opposite at the learnability level: with sufficient training data and CoT, constant-depth transformers PROVABLY LEARN NC1-complete problems -- a strictly larger complexity class than TC0 (under TC0 != NC1). CoT is not cosmetic: it provides the serial depth that breaks the TC0 ceiling and allows learning of problems TC0 cannot express. The per-token compute cost O(N^2*d) is correct but irrelevant to the complexity separation: the serial composition of CoT tokens creates effective depth that constant-depth parallelism cannot replicate, and this paper proves the optimizer actually finds that solution. The ceiling is at NC1, not TC0.
Caveat: the contradiction is strongest at the complexity-theory level and on synthetic tasks; whether it transfers to natural language hallucination in production LLMs is unproven. The thesis may still hold empirically for specific NL tasks even if the theoretical ceiling is NC1 not TC0.

SOURCES
arXiv:2511.07378 abstract and OpenReview metadata (primary source)
arXiv:2310.07923 (Merrill & Sabharwal, Expressive Power of Transformers with CoT) -- prior work
arXiv:2402.12875 (Feng et al., Chain of Thought Empowers Transformers to Solve Inherently Serial Problems) -- prior work
arXiv:2507.07505 (Hallucination Stations, the target thesis paper)
