Algorithm: rStar-Math (Self-Evolved Deep Thinking for Small LLMs)
ArXiv: 2501.04519
Paper: rStar-Math: Small LLMs Can Master Math Reasoning with Self-Evolved Deep Thinking

SCOPE NOTE (ELAI D-2 wiring)
This paper is the structural ground-truth for wiring Qwen2.5-Math-PRM-7B as a verifier-only
runtime in ELAI. The PPM in rStar-Math is the architectural template: a scalar-head model
distinct from the policy SLM, called per MCTS step to produce q-value initialization.
rStar-Math trains its own PPM from a fine-tuned Qwen2.5-Math-7B base; ELAI substitutes
Qwen2.5-Math-PRM-7B (a pre-trained PRM) as a drop-in verifier. The interface contract below
reflects rStar-Math's design; deviations for ELAI are noted inline.


Inputs (MCTS runtime, per node expansion):
problem_text         | string     | natural language math problem, variable length     | mandatory
partial_trajectory   | string     | problem_text concatenated with steps s_1..s_i via newline delimiters | mandatory; must include ALL preceding steps in order (Sec 3.2, Eq 3)
step_candidates      | list[str]  | n candidate continuations at depth i, n=8 default (n=16 in Round 4) | mandatory; min 1
python_code_blocks   | list[str]  | extracted executable Python from each candidate     | optional; candidates with failed execution are discarded (Sec 3.2)
N_parent             | int        | visit count of parent node                          | mandatory for UCT; >= 1
N_node               | int        | visit count of current node                         | mandatory for UCT; >= 0
Q_node               | float      | accumulated q-sum of current node                  | mandatory; range unbounded but expected in [-1,1] per tanh

Inputs (PPM model call, per step):
partial_trajectory_i | string     | x concatenated with s_1 ... s_i (Sec 3.2, Eq 3)   | mandatory
                     |            | NOTE: this is the FULL prefix up to and including the current step |

Inputs (PPM training):
pos_step_pair        | (string, string) | (prefix, positive_step) where positive step = two candidates with highest Q-values (Sec 3.3) | mandatory
neg_step_pair        | (string, string) | (prefix, negative_step) where negative step = two candidates with lowest Q-values (Sec 3.3) | mandatory
NOTE: for intermediate steps, pos and neg pairs MUST share identical preceding steps (Sec 3.3, Eq 5). Violating this collapses comparison signal.
NOTE: final-answer steps use trajectories with highest/lowest average Q-values across all steps (Sec 3.3).

Inputs (training data pipeline):
problem_pool         | list[str]  | 747k math word problems with ground-truth answers (Sec 3.4.1; NuminaMath + MetaMath) | mandatory
ground_truth_answer  | string     | final answer label per problem                     | mandatory for terminal node scoring
rollout_count        | int        | MCTS rollouts per problem: Round 1=8, Rounds 2-3=16, Round 4=128 (Sec A.1) | controls Q-value reliability


Outputs (PPM inference):
q_value_initial      | float      | scalar in [-1, 1] via tanh (Sec 3.4.1); used as q(s_i)^0 in Eq 3 | mandatory
                     |            | invariant: -1.0 <= q_value_initial <= 1.0 (hard constraint from tanh)
                     |            | NOTE: Qwen2.5-Math-PRM-7B may output per-token step scores; ELAI must reduce to scalar in [-1,1]

Outputs (MCTS node selection):
selected_step        | string     | step candidate maximizing UCT(s) = Q(s) + c * sqrt(ln(N_parent) / N_node) (Sec 3.2, Eq 1) | mandatory
uct_score            | float      | UCT value for selected node                         | Q(s) = q_accumulated / N_node

Outputs (terminal node):
terminal_q           | float      | +1.0 if final answer correct, -1.0 if incorrect (Sec 3.2) | binary; no intermediate values
trajectory_q_avg     | float      | average Q-value across all steps in trajectory     | used for training data selection (top-2 per problem, Sec 3.4.1)

Outputs (final answer selection, test time):
best_trajectory      | string     | trajectory with highest average PPM score (Sec A.1) | greedy; NOT majority vote
NOTE: paper does not use majority voting. Selecting by highest PPM step-score average is the canonical test-time strategy.


Hyperparameters:
max_tree_depth       | 16         | [8, 32]    | max solution steps per tree; hard depth cutoff (Sec 3.2, A.1)
n_candidates         | 8 (R1-3), 16 (R4) | [4, 32] | candidates generated per step; more = better Q estimates, higher compute
n_rollouts           | 8/16/128   | [8, 128]   | rollouts per problem per round; low rollout count yields unreliable Q-values (Sec 3.2)
uct_exploration_c    | 2          | [0.5, 5]   | UCT exploration constant; higher = more exploration, slower convergence
seq_len              | 4096       | [2048, 8192] | policy SLM token context; affects step granularity
lr_qwen              | 7e-6       | [1e-6, 1e-5] | linear schedule; authors note sensitivity (Sec 3.4.1)
lr_phi3              | 5e-6       | [1e-6, 1e-5] | cosine schedule
batch_policy         | 128        | [64, 256]  | policy SFT batch size (Sec 3.4.1)
batch_ppm            | 512        | [128, 512] | PPM preference training batch size (Sec 3.4.1)
epochs_policy        | 2          | [1, 3]     | policy fine-tune per round
epochs_ppm           | 1          | [1, 2]     | PPM fine-tune per round; single epoch to prevent overfitting
n_rounds             | 4          | [2, 4]     | self-evolution rounds; PPM-augmented Q-init starts Round 3 (Eq 3)
top_k_trajectories   | 2          | [1, 4]     | per-problem trajectories kept for SFT (highest avg Q, correct answer) (Sec 3.4.1)
ppm_noise_filter     | 0.50       | [0.3, 0.7] | discard synthetic problems if MCTS accuracy < 50% (Sec A.1)


Critical invariants (testable assertions):
# PPM output is bounded by tanh (Sec 3.4.1, scalar head design)
assert -1.0 - 1e-6 <= ppm_score <= 1.0 + 1e-6, f"PPM output {ppm_score} outside [-1,1]"

# UCT formula structure (Sec 3.2, Eq 1): Q(s) = accumulated_q / visit_count
assert abs(Q_s - (q_accumulated / N_node)) < 1e-9 if N_node > 0 else True

# Terminal node Q is binary (Sec 3.2)
assert terminal_q in (1.0, -1.0), f"terminal q must be +1 or -1, got {terminal_q}"

# Preference pair intermediate steps must share identical prefix (Sec 3.3, Eq 5)
assert positive_step_prefix == negative_step_prefix, "PPM pair prefix mismatch: collapses training signal"

# Q-value back-propagation accumulates across rollouts (Sec 3.2, Eq 2)
# q(s_i)^k = q(s_i)^(k-1) + q(s_d)^k  =>  q_accumulated increases monotonically with correct rollouts
assert q_accumulated_after == q_accumulated_before + terminal_q_of_rollout

# Round 3+ Q-init uses PPM, not zero (Sec 3.2, Eq 3)
if round_number >= 3:
    assert q_init != 0.0, "Round 3+ must initialize q from PPM, not zero"

# Candidates with failed Python execution must be discarded (Sec 3.2)
assert all(exec_success[i] for i in selected_candidates), "failed-exec candidates must not be in search tree"

# Test-time trajectory selection is by highest PPM score average, not majority vote (Sec A.1)
# assert final_answer == trajectory_with_max_avg_ppm_score(all_trajectories)


Complexity:
time per MCTS problem  | O(d * n * T_policy + d * T_ppm) where d=depth, n=candidates, T=LLM forward pass | dominant cost is policy generation
space per MCTS problem | O(d * n * L) where L=sequence length; full tree retained for Q back-propagation
PPM call overhead      | 1 forward pass per expanded node (prefix up to step i); NOT batched by default (Sec 3.2)
parallelism            | rollouts per problem are independent; can parallelize across problems; tree rollouts within a problem are sequential (back-propagation dependency)
training data scale    | 747k problems * top-2 trajectories * avg 16 steps = ~24M step samples (estimated)
PPM training pairs     | filtered subset of problems (mixed correct/incorrect); expect ~30-50% of problem set


Failure modes (cited in paper):
geometry problems unsolvable: visual understanding required; 8 of 15 unsolved AIME problems were geometry (Sec 4.2)
policy model capability floor: PPM becomes effective only after policy SLM reaches baseline capability; starting from too-weak base degrades self-evolution (Sec 5)
synthetic problem label noise: manual review found 19/20 sampled unsolved synthetic problems had wrong ground-truth labels (Sec 3.4.2); noisy labels propagate through terminal Q scoring
low rollout count yields unreliable Q-values: extensive rollouts required for statistical reliability of Q estimates (Sec 3.2); too few rollouts = high-variance Q, poor PPM training signal
PPM collapses on uniform trajectories: problems where ALL trajectories are correct or all are incorrect provide no preference signal; must be filtered from PPM training (Sec 3.4.1)
incremental policy fine-tuning degrades: training policy from previous-round checkpoint (not base model) causes drift; paper trains from base each round (Sec A.1)
per-step vs per-trajectory training instability: splitting trajectory into per-step PPM samples causes instability; must treat trajectory as single sample with simultaneous step prediction (Sec A.1)


Implementation pitfalls (do NOT do this):
DO NOT use ORM (outcome reward model only): step-level signal from PPM/PRM outperforms ORM; using only final-answer correctness loses intermediate feedback (Table 8)
DO NOT apply direct Q-value regression for PPM: authors explicitly replace MSE regression with pairwise preference loss (Bradley-Terry, Eq 4); regression yields weaker step discrimination (Sec 3.3)
DO NOT skip prefix sharing for intermediate PPM pairs: pos/neg steps MUST share the same preceding context (Sec 3.3, Eq 5); shuffling prefixes breaks the controlled comparison
DO NOT use majority voting at test time: rStar-Math selects by highest-PPM-average trajectory, not majority vote; conflating these changes selection behavior (Sec A.1)
DO NOT fine-tune policy incrementally across rounds: always restart from the initial base model per round to avoid drift accumulation (Sec A.1)
DO NOT start PPM augmentation before Round 3: Q-values from Rounds 1-2 are terminal-guided only (Eq 2); PPM init (Eq 3) requires a trained PPM from Round 2 to be reliable
DO NOT batch PPM calls by truncating prefix: full partial trajectory x concatenated with s_1..s_i is required input; truncating prefix loses causal context for step evaluation
DO NOT retain candidates with failed Python execution: code-augmented CoT requires successful exec; failed-exec candidates must be pruned before Q scoring (Sec 3.2)
DO NOT treat PPM and policy as the same runtime: PPM uses a scalar head (not next-token head); they cannot share inference stack without architecture divergence (Sec 3.4.1)
DO NOT use tanh-sigmoid confusion: PPM output is tanh (range [-1,1]), not sigmoid (range [0,1]); positive Q means correct-leaning, zero is neutral, negative is incorrect-leaning


ELAI D-2 specific notes (heterogeneous-verifier wiring):
Qwen2.5-Math-PRM-7B is a pre-trained PRM, not a PPM. Its outputs may be per-token step scores
rather than a single scalar per step. ELAI must:
  (a) identify the score token position convention used by Qwen2.5-Math-PRM-7B
  (b) reduce per-step token scores to a single scalar in [-1,1] (mean, last-token, or softmax ratio)
  (c) keep this verifier runtime on a separate inference endpoint from the policy SLM
  (d) ensure the policy SLM generates step text; verifier scores it; neither model shares weights at runtime
The paper's PPM is initialized FROM the fine-tuned policy model (Sec 3.4.1). Qwen2.5-Math-PRM-7B
is not derived from ELAI's policy model - this is an acceptable deviation since PRM-7B was trained
on math process reward data independently, but the score normalization step (b) above is mandatory
to maintain the [-1,1] invariant the MCTS UCT formula depends on.
