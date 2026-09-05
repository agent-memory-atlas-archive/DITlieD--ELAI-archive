Algorithm: AlphaEvolve (evolutionary coding agent for scientific and algorithmic discovery)
ArXiv: 2506.13131
Paper: AlphaEvolve: A coding agent for scientific and algorithmic discovery
Authors: Alexander Novikov, Ngan Vu, Marvin Eisenberger, Emilien Dupont, Po-Sen Huang, Adam Zsolt Wagner et al., Google DeepMind
Published: 2025-06-16
Local PDF: F:\.ELAI_workflow\.claude\papers\2506.13131.pdf

SCOPE NOTE FOR D-15:
The paper does NOT define a formal "novelty class" taxonomy or a "surprise score" as a
measured output variable. Novelty and quality are balanced implicitly through (a) MAP-Elites
behavioral binning in the program database and (b) multi-objective evaluation via multiple
score metrics. D-15's novelty class structure must be derived/extended from the paper's
quality-diversity framework, not read directly from a paper-specified classification scheme.


Inputs:
problem_definition      | user-provided Python code  | any length          | must contain evaluate() function returning dict[str, float]; must contain EVOLVE-BLOCK-START/END markers; all metrics maximized by convention (Section 2.1)
initial_program         | source code (any language) | any length          | must be complete and runnable; can be rudimentary (e.g., returns constants); serves as root of evolutionary tree (Section 2.1)
evaluation_function     | Python callable h          | fixed signature      | maps solution -> dict[str, float]; can spawn distributed computation; can use evaluation cascade (Section 2.1, 2.4)
prompt_template         | text template              | optional            | includes explicit context, stochastic formatting alternatives, rendered evaluation results, meta-prompt instructions (Section 2.2)
llm_ensemble            | LLM configuration          | optional            | default: Gemini 2.0 Flash (high throughput) + Gemini 2.0 Pro (high quality) (Section 2.3); model-agnostic
compute_budget          | wall-clock hours or iters  | user-specified      | no hard minimum; practical examples use 100 compute-hours per evaluation (Section 2.4)


Outputs:
best_program            | source code                | same lang as input  | the program achieving highest score(s) under evaluation function h; provably correct only if evaluator enforces correctness
score_dictionary        | dict[str, float]           | n_metrics scalars   | one entry per evaluation metric; all maximized; invariant: each value is a real scalar (not NaN/None)
program_database        | collection of (program, scores) | unbounded size | stores all evaluated solutions with scores; used for prompt context in future generations
evolutionary_trace      | sequence of diffs          | unbounded           | the SEARCH/REPLACE diff sequence leading from initial to best program; not always recoverable in exact form


Hyperparameters:
llm_mix_ratio           | not fixed; Flash:Pro mix   | no stated default   | ablation (Fig. 8) shows "Small base LLM only" is significantly worse; balance throughput vs quality (Section 2.3, 4)
meta_prompt_evolution   | enabled                    | bool, default on    | ablation shows significant degradation when disabled for tensor decomposition (Section 4, Fig. 8)
full_file_evolution     | enabled                    | bool, default on    | ablation shows significant degradation when only loss function evolved (Section 4, Fig. 8)
evaluation_cascade      | optional, user-defined     | list of test stages | prune unpromising solutions early; new solutions evaluated on small scale before full test (Section 2.4)
abstraction_level       | direct/constructor/search  | categorical         | "direct solution" vs "constructor function" vs "search algorithm" vs "co-evolution"; symmetric solutions benefit from constructor form (Section 2.1)
database_algorithm      | MAP-Elites + island-model  | hybrid, configured  | inspired by MAP-Elites [ref 74] and island-based population models [ref 83, 97]; balances exploration/exploitation (Section 2.5)
context_window_content  | explicit/stochastic/meta   | user-configured     | explicit problem context, stochastic formatting, rendered evaluation results, meta-prompt co-evolution (Section 2.2)
multi_objective_metrics | list of metric names       | user-defined        | optimizing multiple metrics improves single-metric results due to diversity; each metric is a scalar float (Section 2.4)


Critical invariants (testable assertions):
assert callable(evaluate) and isinstance(evaluate(solution), dict) and all(isinstance(v, float) for v in evaluate(solution).values()), "evaluate() must return dict[str, float] (Section 2.1)"
assert all(v == v for v in score_dict.values()), "No NaN scores allowed in program database (implicit from Section 2.4 evaluation cascade)"
assert "# EVOLVE-BLOCK-START" in initial_program and "# EVOLVE-BLOCK-END" in initial_program, "Initial program must contain evolution block markers (Section 2.1)"
assert len(diff_blocks) >= 1 and all("<<<<<<< SEARCH" in d and "=======" in d and ">>>>>>> REPLACE" in d for d in diff_blocks), "LLM output must use SEARCH/REPLACE diff format (Section 2.3)"
assert best_score >= initial_score or initial_score == 0, "Evolution must be monotonically non-decreasing in best score over generations (implicit from evolutionary selection)"
assert all(metrics[m] == metrics[m] for m in metrics), "Multi-objective metrics must all be finite scalars, no None or inf (Section 2.4)"


Complexity:
time_per_evaluation     | O(problem-dependent); can reach ~100 compute-hours per solution (Section 2.4)
parallelism             | embarrassingly parallel evaluation supported via async pipeline; controller + LLM samplers + evaluation nodes run concurrently (Section 2.6)
throughput_optimization | pipeline optimized for total IDEAS evaluated, not speed of single evaluation (Section 2.6)
space_database          | unbounded; grows with number of evaluated solutions; no stated eviction policy in paper beyond MAP-Elites quality-diversity binning (Section 2.5)
llm_calls_per_iter      | 1+ per new solution; ensemble calls Flash + Pro; meta-prompt step adds 1 extra LLM call when enabled (Section 2.2, 2.3)


Failure modes (cited by authors):
evaluator_required      | AlphaEvolve requires an automated evaluator; domains without machine-gradeable solutions (e.g., natural science experiments that cannot be simulated) are out of scope (Section 6)
large_matrix_OOM        | programs for matrix sizes beyond <5,5,5> run out of memory on single-GPU evaluators with 1000 random seeds; requires further optimization for larger sizes (Appendix A)
non_symmetric_bias      | constructor-function abstraction favors symmetric solutions; use direct solution or search-algorithm abstraction for non-symmetric problems (Section 2.1, 6)
no_evolution_baseline   | without evolutionary database (repeated same initial program), performance degrades dramatically (Fig. 8 ablation); no-context prompt also strongly degrades results
single_llm_degradation  | using only a small base LLM significantly reduces discovery rate; ensemble mix of Flash+Pro is load-bearing (Fig. 8 ablation)
lllm_hallucination_risk | LLM-provided evaluation is possible but not the optimized setting; programmatic evaluation with code execution is preferred to sidestep hallucination (Section 6)
convergence_risk        | without MAP-Elites diversity maintenance, evolutionary search converges prematurely; diversity is structural, not a scored output variable (Section 2.5)


Implementation pitfalls (do NOT do this):
do_not_use_natural_language_only_evaluation | LLM-graded evaluation without code execution is not the optimized path; programmatic evaluate() with actual execution is required for reliable signal (Section 6)
do_not_evolve_single_function_only          | FunSearch-style single-function evolution is strictly weaker; AlphaEvolve's full-file/codebase evolution is load-bearing per ablation (Section 4)
do_not_skip_meta_prompt_evolution           | disabling meta-prompt co-evolution causes statistically significant degradation on tensor decomposition (Fig. 8)
do_not_assume_novelty_is_a_scored_variable  | the paper has NO explicit "novelty score" or "surprise score" as an output of evaluate(); diversity is achieved via MAP-Elites structural binning, not a scalar novelty metric
do_not_treat_diversity_and_quality_as_competing_objectives | they are reconciled architecturally via the database (MAP-Elites bins), not via a Pareto front or weighted sum in the evaluation function
do_not_apply_without_automated_evaluator    | adding natural language goals without a deterministic evaluator is not the paper's setup; correctness guarantees evaporate (Section 2.1, 6)
do_not_hardcode_abstraction_level           | abstraction choice (direct/constructor/search algorithm) is problem-dependent; symmetric problems favor constructor, asymmetric favor search algorithm (Section 2.1)
do_not_evaluate_at_full_scale_before_cascade | evaluation cascade (small-scale pre-screening) is required to prune faulty programs early without burning compute (Section 2.4)


Novelty-Quality Framework (for D-15 novelty class labeling):
SOURCE: Derived from Sections 2.4, 2.5, and ablations (Fig. 8), NOT from a paper-defined taxonomy.

Quality axis: single or multi-objective scalar scores from evaluate(); all maximized; higher = better.
Diversity/novelty axis: implicit, implemented via MAP-Elites behavioral binning in program database.
  MAP-Elites bins: programs are placed into cells defined by discretized behavior descriptors.
  Per-cell selection: only top performers per cell are retained as parents.
  Island model: analogous to FunSearch island-based populations [ref 97]; separate databases that
    occasionally exchange programs; each island maintains independent evolutionary trajectory.
  Selection for prompting: programs from different cells (diverse behaviors) are sampled as
    "inspirations" in the prompt; this is the mechanism that drives novelty/exploration.

Novelty class inference for D-15:
  INCREMENTAL: high quality, same behavioral bin as parent; exploitation mode.
  COMBINATORIAL: high quality, different bin from parents; crossover of known solutions.
  SURPRISING: high quality, NEW bin (previously empty cell in MAP-Elites grid); genuine discovery.
  LOW_QUALITY: fails evaluation cascade or score < population threshold; not stored or rarely sampled.
  These class names are D-15 EXTENSIONS, not paper-defined terms. The paper only defines the
  MAP-Elites bin structure as the mechanism; D-15 must map bin transitions to class labels.

Evidence for "surprising" = "new MAP-Elites cell":
  "a growing number of solutions with evaluation results attached... to optimally resurface
   previously explored ideas in future generations" (Section 2.5) -- new cell = no prior occupant.
  "we observe AlphaEvolve performs increasingly better as the underlying LLM improves" (Sec 2.3)
    -- surprising solutions require frontier LLMs; small LLMs mostly produce INCREMENTAL class.
  Empirical signal: 20% of math problems yielded SOTA-beating solutions (Section 3.2)
    -- this is the empirical base rate of "surprising" class solutions in open math domain.


Contract test stubs: see below.
