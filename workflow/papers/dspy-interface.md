Algorithm: DSPy — Declarative Language Model Programs with Self-Improving Pipelines
ArXiv: 2310.03714
Paper: DSPy: Compiling Declarative Language Model Calls into Self-Improving Pipelines
Authors: Omar Khattab, Arnav Singhvi, Paridhi Maheshwari, et al. (2023)
Source verified: https://arxiv.org/abs/2310.03714 (fetched 2026-05-27)
MIPROv2 API verified: https://dspy.ai/api/optimizers/MIPROv2/ (fetched 2026-05-27)

Inputs (MIPROv2.compile):
student           | dspy.Module subclass  | any topology         | must have at least one Predict/ChainOfThought module
trainset          | list[dspy.Example]    | n >= 1               | mandatory; each example must have input fields + expected output
valset            | list[dspy.Example]    | optional             | if provided, minibatch_size <= len(valset) strictly enforced
metric            | Callable[[pred,gold,trace?], float|bool] | returns scalar or bool | must be deterministic for reproducibility
prompt_model      | dspy LM object        | optional             | defaults to dspy.configure(lm=...); used to generate candidate instructions
task_model        | dspy LM object        | optional             | defaults to prompt_model; used to evaluate candidates

Inputs (MIPROv2 constructor):
metric                  | Callable      | required       | no default
max_bootstrapped_demos  | int           | default 4      | range [0, len(trainset)]
max_labeled_demos       | int           | default 4      | range [0, len(trainset)]
auto                    | str or None   | default 'light'| one of 'light','medium','heavy',None; None requires explicit num_candidates + num_trials
num_candidates          | int or None   | None           | mandatory when auto=None; number of candidate instructions to generate
seed                    | int           | default 9      | controls reproducibility of bootstrap sampling
init_temperature        | float         | default 1.0    | instruction generation temperature
minibatch_size          | int           | default 35     | must be <= len(valset)
num_trials              | int or None   | None           | mandatory when auto=None; optimization iterations (Bayesian search steps)

Outputs:
optimized_program | dspy.Module subclass  | same topology as student | instructions and few-shot demos embedded in module signatures
compile returns the student module mutated in-place and also returned

Hyperparameters:
auto                    | 'light'   | ['light','medium','heavy',None] | controls num_candidates + num_trials jointly; use None only if you know target trial count
max_bootstrapped_demos  | 4         | [0, 16]         | higher = more context per inference call; bottleneck at long context LMs
max_labeled_demos       | 4         | [0, 16]         | paper shows diminishing returns past 8 total demos
init_temperature        | 1.0       | [0.5, 2.0]      | lower = less diverse candidates; higher = incoherent instructions
minibatch_size          | 35        | [10, len(valset)]| smaller = noisier metric signal; larger = more LM calls per trial

Critical invariants (testable assertions):
assert len(trainset) >= 1
assert valset is None or minibatch_size <= len(valset)
assert auto in ('light', 'medium', 'heavy', None)
assert not (auto is not None and num_candidates is not None), "cannot set auto and num_candidates simultaneously"
assert not (auto is not None and num_trials is not None), "cannot set auto and num_trials simultaneously"
assert auto is None or (num_candidates is None and num_trials is None)
assert max_bootstrapped_demos >= 0 and max_labeled_demos >= 0
# compiled program must expose same input/output signature as original student
# metric callable must accept (prediction, gold_example) and return float or bool

Complexity:
time  | O(num_trials * minibatch_size * task_model_calls + num_candidates * prompt_model_calls) | no closed form; 'light' ~50-100 LM calls total, 'heavy' ~300+
space | O(num_candidates * avg_instruction_length + max_demos * example_size)
parallelism | candidates generated sequentially per trial; no built-in parallelism in base compile()

Failure modes (from paper + docs):
1. minibatch_size > len(valset) raises hard error at compile time
2. metric returning None instead of float/bool causes silent NaN in Bayesian search, wrong convergence
3. prompt_model and task_model both defaulting to None without dspy.configure raises AttributeError at compile
4. trainset examples missing expected output field cause bootstrap to produce empty demonstrations silently
5. Using auto mode with explicit num_candidates/num_trials raises ValueError
6. Student module with no Predict nodes compiles but produces no optimization (no-op)

Implementation pitfalls (do NOT do this):
- Do NOT use the same model object for prompt_model and task_model when budget-constrained; separate models allow cheaper task eval
- Do NOT skip valset when num_trials is large; without valset, full trainset is used for every trial eval (expensive)
- Do NOT assume optimized program is a new object; compile() mutates the student in-place, clone before compile if you need original
- Do NOT compare compiled programs across runs without fixing seed; Bayesian search is non-deterministic otherwise
- Do NOT use metric functions with side effects (logging, DB writes) inside compile(); called O(num_trials * minibatch_size) times
- Do NOT set minibatch_size to 1; metric noise at single-example level breaks Bayesian convergence
