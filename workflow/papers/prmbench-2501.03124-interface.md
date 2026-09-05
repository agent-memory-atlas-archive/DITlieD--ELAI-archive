Algorithm: PRMBench - Process-Level Reward Model Benchmark
ArXiv: 2501.03124
Paper: PRMBench: A Fine-grained and Challenging Benchmark for Process-Level Reward Models
Venue: ACL 2025 (ACL Anthology 2025.acl-long.1230)
Authors: Mingyang Song, Zhaochen Su, Xiaoye Qu, Jiawei Zhou, Yu Cheng
Project: https://prmbench.github.io/  Code: https://github.com/ssmisya/PRMBench
Dataset: hitsmy/PRMBench_Preview (HuggingFace, Apache 2.0, 6,220 rows)


--- BENCHMARK STRUCTURE ---

Dataset scale:
  6,216 problems total, 83,456 step-level labels, avg 13.4 steps/problem, avg 2.1 error steps/problem
  Split: train (all 6,216 rows); no separate test split released publicly

Three evaluation dimensions, nine sub-categories:
  SIMPLICITY (2):
    NR  | Non-Redundancy        | Detect unnecessary/redundant steps in reasoning process
    NCL | Non-Circular Logic    | Detect circular reasoning where conclusion is reused as premise
  SOUNDNESS (4):
    ES  | Empirical Soundness   | Detect reasoning steps that contradict established facts (counterfactual errors)
    SC  | Step Consistency      | Detect contradictions between steps; ensure logical flow across steps
    DC  | Domain Consistency    | Detect misuse of domain-specific knowledge / wrong concept application
    CI  | Confidence Invariance | Detect overly confident incorrect statements; appropriate confidence throughout
  SENSITIVITY (3):
    PS  | Prerequisite Sensitivity  | Detect missing conditions/prerequisites essential for valid reasoning
    DR  | Deception Resistance      | Detect misleading but plausible-sounding statements
    MS  | Multi-Solution Consistency| Consistent evaluation across multiple valid solution paths


--- INPUTS ---

Per-problem record (JSON, one row in dataset):
  original_question  | str           | scalar     | The base problem statement; mandatory
  modified_question  | str           | scalar     | Evaluation version (often same as original); mandatory
  original_process   | list[str]     | [N steps]  | Correct solution steps; mandatory
  modified_process   | list[str]     | [N steps]  | Potentially altered steps used for evaluation; mandatory
  modified_steps     | list[int]     | [k ints]   | 0-based indices of steps that were changed; mandatory
  error_steps        | list[int]     | [m ints]   | 0-based indices of steps containing errors (m <= k); mandatory
  reason             | str           | scalar     | Textual explanation of the error; mandatory
  idx                | str/int       | scalar     | Unique identifier; mandatory
  classification     | str           | scalar     | One of the 9 sub-category codes (NR/NCL/ES/SC/DC/CI/PS/DR/MS); mandatory

PRM model invocation input (what the model receives per problem):
  question text  | str   | the modified_question field
  steps          | list  | the modified_process field (list of step strings)
  (model does NOT receive error_steps or classification during inference)

PRM model output expected:
  per-step scores  | list[float]  | [N scores]  | One scalar score per step; range [0.0, 1.0] (higher = more valid/correct)
  (binary label derived by caller via threshold, not by model)


--- OUTPUTS ---

From evaluation framework (score_dict, per problem):
  step_level_validity_scores      | list[float]  | [N floats]   | Raw PRM score per step, range [0,1]
  step_level_redundancy_scores    | list[float]  | [N floats]   | Raw PRM score per step for redundancy axis
  step_level_validity_labels      | list[int]    | [N ints]     | Binarized: 1=correct, 0=error (via threshold)
  step_level_redundancy_labels    | list[int]    | [N ints]     | Binarized redundancy labels (via threshold)
  solution_level_validity_scores  | float        | scalar       | Aggregate validity score for full solution
  solution_level_redundancy_scores| float        | scalar       | Aggregate redundancy score for full solution
  solution_level_validity_labels  | int          | scalar       | Binarized solution-level validity
  solution_level_redundancy_labels| int          | scalar       | Binarized solution-level redundancy

Aggregate benchmark metric:
  PRMScore  | float  | [0, 100]  | Primary reported metric; see formula below
  Per-dimension scores (Simplicity / Soundness / Sensitivity) | float | aggregated from sub-category F1s
  Per-sub-category scores (NR/NCL/ES/SC/DC/CI/PS/DR/MS) | float | F1-based


--- HYPERPARAMETERS ---

  redundancy_threshold  | 0.15  | (0.0, 1.0)  | Threshold for binarizing redundancy scores; example value in docs; sensitivity: changes classification boundary
  validity_threshold    | 0.5   | (0.0, 1.0)  | Fixed threshold for binary classification in evaluation; cited in R-PRM paper (arXiv:2503.21295) as adopted standard; HIGH sensitivity - shifts all F1 scores
  batch_size            | 2     | [1, GPU-limit]  | Inference batch size; MUST be 1 for API models (Gemini, OpenAI); parallelism must be disabled for API modes


--- SCORING FORMULA ---

Step-level binary label (per step i, per problem):
  label_i = 1  if score_i >= threshold else 0
  ground truth: 1 if step index NOT in error_steps, 0 if step index IN error_steps

Per sub-category c, positive F1 and negative F1:
  F1+_c = 2*precision+*recall+ / (precision+ + recall+)  [over correct steps]
  F1-_c = 2*precision-*recall- / (precision- + recall-)  [over error steps]

PRMScore (primary metric, Section 4 / Eq. in paper):
  PRMScore = (1/|C|) * sum_{c in C} (F1+_c + F1-_c) / 2
  where C = set of all 9 sub-categories
  Range: [0, 100] (random-guess baseline ~50; top models ~65-68.8)

Earliest Error Detection (secondary metric):
  Whether model assigns lowest score to the FIRST step in error_steps
  Binary per-problem, aggregated as accuracy


--- COMPLEXITY ---

  Time (inference): O(N_problems * N_steps * T_model) where T_model = PRM forward pass cost
  Space: O(N_steps) per problem for score storage; full dataset fits in memory (~6K problems)
  Parallelism: multi-process supported for local models; batch_size=1, no parallelism for API models
  No sequential step dependency in scoring (each step scored independently)


--- KNOWN FAILURE MODES (from paper/cited work) ---

  Positive reward bias: open-source PRMs systematically under-detect errors; assign high scores to wrong steps
  Simplicity failure: sub-50 PRMScore on NR/NCL sub-categories widely observed; models cannot detect redundancy
  Delayed error flagging: models detect error at step N+2 rather than N (earliest-error-detection metric captures this)
  Confidence Invariance failure: models reward confidently-stated wrong steps
  Domain Consistency failure: models apply correct procedure in wrong domain context without penalizing
  Deception Resistance failure: plausible-sounding but wrong steps are rated highly
  Score saturation: some PRMs collapse to near-constant scores across all steps, defeating F1 computation


--- IMPLEMENTATION PITFALLS (do NOT do this) ---

  Do NOT use only solution-level accuracy as the metric; PRMBench requires step-level F1, not solution pass/fail
  Do NOT aggregate step scores via simple average (min-product or mean misses earliest-error signal)
  Do NOT binarize with threshold 0.5 for redundancy axis; redundancy uses lower threshold (0.15 example in docs)
  Do NOT feed model the error_steps field during inference; that is ground truth, not input
  Do NOT skip per-sub-category breakdown; overall PRMScore alone is insufficient for diagnostic use
  Do NOT run API models (OpenAI/Gemini) with batch_size > 1 or multi-process; framework explicitly bans this
  Do NOT treat modified_steps == error_steps; modified_steps are all changed steps, error_steps is the subset that are wrong (some modifications may be correct alternatives)
  Do NOT assume F1+ and F1- are symmetric; PRMs with positive bias will have high F1+ but near-zero F1-


--- BASELINE RESULTS (verified from paper + R-PRM arXiv:2503.21295) ---

  Random guess:               PRMScore ~50
  ReasonEval-7B (GAIR):       ~60
  ReasonEval-34B:             ~63
  Qwen2.5-Math-PRM-7B:        ~65
  GPT-4o (critic-prompted):   ~66
  Gemini-2 (critic-prompted): ~67
  o1-mini (critic-prompted):  ~68.8
  Best open-source PRM:       ~65 (Qwen2.5-Math-PRM-7B)
  All models show < 60 on Simplicity dimension


--- GROUNDING NOTES ---

  Nine sub-category definitions: extracted from arXiv:2503.21295v1 Appendix A (R-PRM cites PRMBench definitions verbatim)
  PRMScore formula: verified via emergentmind.com topic page citing paper Section 4
  Threshold 0.5: cited explicitly in arXiv:2503.21295v1 "we adopt a fixed threshold of 0.5"
  Threshold 0.15 (redundancy): from GitHub docs/document.md example config
  Data schema fields: verified from GitHub README + HuggingFace dataset card (hitsmy/PRMBench_Preview)
  score_dict field names: verbatim from docs/document.md "temporary results is formatted as..."
  Dataset size 6,216 / 83,456: confirmed by ACL anthology page and abstract
  Baseline scores: from paper abstract + R-PRM Table 2 comparison
