Algorithm: OPRO — Optimization by PROmpting (Large Language Models as Optimizers)
ArXiv: 2309.03409
Paper: Large Language Models as Optimizers
Authors: Chengrun Yang, Xuezhi Wang, Yifeng Lu, Hanxiao Liu, Quoc V. Le, Denny Zhou, Xinyun Chen (Google DeepMind, 2023)
Source verified: https://arxiv.org/abs/2309.03409 (fetched 2026-05-27), HTML at https://arxiv.org/html/2309.03409

Inputs:
meta_prompt         | str                  | structured text       | mandatory; must contain (1) problem description, (2) prior solution-score pairs sorted ascending, (3) task exemplars
objective_function  | Callable[[str], float]| returns scalar score  | mandatory; evaluates a candidate instruction on training set
initial_solutions   | list[str]            | >= 1 item             | starting instruction(s); blank initialization performs poorly
training_set_size   | int                  | paper: 3.5-20% of full dataset | small subsets only; full dataset eval per step is prohibitive
optimizer_llm       | LLM                  | temperature=1.0       | the model acting as optimizer (distinct from task model)

Outputs:
best_instruction    | str                  | natural language      | instruction with highest observed score across all steps
optimization_trajectory | list[(str,float)]| ordered by discovery time | all (instruction, score) pairs for meta-prompt construction

Hyperparameters:
temperature          | 1.0    | [0.5, 2.0]    | critical: 0.0 underexplores (local optimum), 2.0 ignores trajectory; authors test all three
solutions_per_step   | 8      | [1, 20]       | 8 substantially outperforms 1; diminishing returns above 8
meta_prompt_window   | 20     | [5, 40]       | best N prior (instruction, score) pairs retained in ascending score order; recency + quality signal
num_exemplars        | 3      | [1, 5]        | 3 optimal; more creates distraction in meta-prompt; paper Section 3.2
max_steps            | 200    | [20, 500]     | early convergence common around step 50-100 for prompt optimization

Critical invariants (testable assertions):
# solution-score pairs MUST be sorted ascending (lowest score first, highest last)
# recency bias: most recent high-score solutions appear at end of meta-prompt
assert all(pairs[i][1] <= pairs[i+1][1] for i in range(len(pairs)-1)), "meta-prompt pairs must be ascending by score"
# temperature must be > 0 for non-degenerate exploration
assert temperature > 0.0
# training set must be small enough for per-step evaluation to be tractable
# final instruction score on training set does NOT equal test score; gap typically 5-20pp (Section 4.2)
assert num_exemplars >= 1, "zero exemplars degrades instruction quality (Section 3.2)"
# multi-instruction generation per step is required; single-instruction generation significantly underperforms (ablation Table 4)
assert solutions_per_step >= 2

Complexity:
time  | O(max_steps * solutions_per_step * (optimizer_llm_call + objective_eval)) per run
space | O(meta_prompt_window * avg_instruction_length + num_exemplars * exemplar_length)
parallelism | solutions_per_step instructions can be generated in parallel (independent samples); objective evals parallelizable over training set

Failure modes (from paper):
1. Ascending sort violated: trajectory signal lost; optimizer treats worse instructions as better recent progress
2. Training set too large: per-step evaluation cost prohibitive; paper uses 3.5% of GSM8K (265 examples)
3. Temperature too low (0.0): repeated near-identical instructions; optimization stalls within 20 steps
4. Zero exemplars: instruction generation ignores task format; produces generic text
5. TSP / combinatorial: OPRO degrades sharply above n=10 nodes; context window cannot fit large problem descriptions
6. Overfitting to training instructions: 5-20pp gap between train and test accuracy is expected and unavoidable
7. Problem description ambiguity: vague task description produces low-diversity instruction pool

Implementation pitfalls (do NOT do this):
- Do NOT sort pairs descending; ascending order is load-bearing for recency bias (higher scores = more recent = end of list)
- Do NOT use the same LLM for optimization and task execution without cost awareness; two separate model configs recommended
- Do NOT evaluate on full dataset per step; sample a fixed small subset and hold it constant across all steps
- Do NOT treat training accuracy as validation accuracy; report test set numbers separately (mandatory for honest evaluation)
- Do NOT skip the trajectory: one-shot instruction generation (no prior solutions in meta-prompt) significantly underperforms (ablation)
- Do NOT apply OPRO to combinatorial problems with n > 10 without chunking; context window is a hard constraint
