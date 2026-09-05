Algorithm: LiveCodeBench (Contamination-Free Holistic Code LLM Benchmark)
ArXiv: 2403.07974
Paper: LiveCodeBench: Holistic and Contamination Free Evaluation of Large Language Models for Code
Authors: Jain, Han, Gu, Li, Yan, Zhang, Wang, Solar-Lezama, Sen, Stoica (UC Berkeley / MIT / Cornell)
Version: v2 (HTML fetched 2025-12-22)

Core design principle: contamination avoidance via temporal windowing, not n-gram decontamination.
Problems are tagged with contest release date D. Evaluation is filtered to problems released AFTER
a model's training cutoff date. There is NO 9-gram test or semantic fingerprint in this paper.
Those methods are cited as external prior work (Shi et al. 2023; Riddell et al. 2024 in S6.SS3).

Scope note for D-6: the "held-out task store by date" IS the contamination-resistance mechanism.
Task retirement policy is NOT described in this paper. The paper proposes adding a private unreleased
test set as future work (S7) to reduce reliance on public problems.


DATA SCHEMA (per problem tuple, S3.SS1)

problem_id     | str  | platform-specific contest identifier
P              | str  | natural language problem statement (with example I/O)
T              | list[tuple[input, output]] | test cases (~17 avg, Table 1)
S              | str  | ground truth solution (human-submitted, where available)
D              | date | contest release date (YYYY-MM-DD); used for temporal filtering
difficulty     | enum | Easy / Medium / Hard (platform-specific rating mapping)
platform       | enum | LeetCode / AtCoder / CodeForces
starter_code   | str  | optional; LeetCode only (functional format)


PLATFORM DIFFICULTY MAPPINGS (S3.SS2)

LeetCode:    platform-native Easy / Medium / Hard labels
AtCoder:     [0-200) = Easy, [200-400) = Medium, [400-500] = Hard; abc contests only; arc/agc excluded
CodeForces:  {800} = Easy, (800-1000] = Medium, (1000-1300] = Hard; Div 3 + Div 4 only
AtCoder max difficulty cutoff: 500 (problems rated >500 excluded)
CodeForces max difficulty cutoff: 1300


DATASET STATISTICS at paper publication (Table 1)

LCB full (May23-May24):   511 problems, 182 Easy, 206 Medium, 123 Hard, avg 17.0 tests
LCB Sep-window (Sep23+):  349 problems, 125 Easy, 136 Medium, 88 Hard, avg 18.0 tests
AtCoder:                  267 problems, avg 15.6 tests
LeetCode:                 235 problems, avg 19.0 tests
CodeForces:               9 problems, avg 11.1 tests


FOUR EVALUATION SCENARIOS (S2)

1. Code Generation
   Input:  problem statement P + example tests
   Output: Python program
   Metric: Pass@1, functional correctness on unseen test cases T
   Programs must pass ALL tests to be considered correct (S4.p3)

2. Self-Repair
   Input:  P + incorrect candidate program + error feedback (exception, failing test, WA, TLE)
   Output: repaired Python program
   Metric: Pass@1 on final program (either initial correct OR repaired correct)
   Error feedback types: syntax error, runtime error, wrong answer, time-limit error (S4.SS1.p3)

3. Code Execution
   Input:  Python function f + test input i
   Output: predicted output value
   Metric: execution correctness: assert f(input) == generated_output passes (S2.p6)
   Source: ~2000 human-submitted correct LeetCode solutions filtered to 479 samples from 85 problems

4. Test Output Prediction
   Input:  problem statement P + test case input
   Output: expected output for that input
   Metric: string equivalence after parsing; LeetCode example inputs used as test inputs
   Dataset: 442 instances from 181 LeetCode problems
   NOTE: evaluates reasoning only; test INPUT prediction is explicitly avoided (S2.p7)


CONTAMINATION-RESISTANCE MECHANISM (S1.p2, S3.SS1, S5.SS1)

Primary method: temporal windowing
  - Each problem tagged with release date D (= contest date)
  - For a model with cutoff C, only evaluate on problems where D > C
  - If training cutoff not published, use release date as proxy (S3.SS1.p2)

Evidence of contamination detected:
  - DS-Ins-33B (DeepSeek): Pass@1 drops from ~60 on May'23 LeetCode to ~0 on Sep'23 LeetCode
  - DS-Base-33B: same drop pattern (contamination in pretraining corpus confirmed)
  - GPT-4-O: performance drops on LeetCode problems released after Nov'23 (its stated cutoff)
  - Codestral: Pass@1 36.5 on May23-Jan24 window vs 28.3 on Feb24+ window

Key finding: contamination is LeetCode-specific; AtCoder shows stable performance across time (S5.SS1.p3)

What this paper does NOT do (important for D-6 scope):
  - No n-gram/9-gram decontamination check on training corpora
  - No semantic fingerprinting of problems
  - No task retirement by "uniformly-passing" criterion
  - The cited AST + edit-distance method is from Riddell et al. 2024 (S6.SS3), NOT this benchmark


EVALUATION PROTOCOL (S4)

Generations per problem: 10 candidates
Sampling: nucleus sampling, temperature=0.2, top_p=0.95
Framework: API calls or vLLM
Metric: Pass@1 = fraction of 10 candidates passing all tests
Base models: code generation only (one-shot prompt, stdin/functional format)
Instruction models: zero-shot for codegen and test-output-prediction; few-shot for code execution

Test collection (S3.SS1.p3):
  - Platform tests used when available
  - Otherwise: GPT-4-Turbo generates input generators (not inputs directly) via in-context learning
  - Generator samples inputs based on problem specifications
  - Some failing tests from auto-grader collected for recent problems


CRITICAL INVARIANTS (testable for D-6 task store implementation)

1. assert task.release_date > model.training_cutoff  # only evaluate on post-cutoff problems
2. assert task.release_date >= datetime(2023, 5, 1)  # earliest valid problem date
3. assert task.difficulty in ("Easy", "Medium", "Hard")  # valid difficulty enum
4. assert task.platform in ("LeetCode", "AtCoder", "CodeForces")  # valid source
5. assert len(task.tests) >= 1  # every problem must have at least one test case
6. assert task.solution is not None or len(task.tests) > 0  # can grade via test or solution
7. # AtCoder difficulty rating must be <= 500 if included
   if task.platform == "AtCoder": assert task.platform_rating <= 500
8. # CodeForces rating must be <= 1300
   if task.platform == "CodeForces": assert task.platform_rating <= 1300


PERFORMANCE VARIANCE NOTES (S7 Limitations)

Benchmark size creates 1-1.5% performance noise (bootstrapped on 349 problems from 511-problem set)
Models with tight performance differences (<2 points) cannot be reliably distinguished
Self-repair: 349 problems; Code execution: 188 problems; Test output prediction: 254 problems
(Post-cutoff window shrinks sample size further for newer models)

HumanEval vs LCB correlation: only 0.72 (S5.SS2.p4)
Fine-tuned open models cluster in "overfit" region: high HumanEval+, low LCB
Example: DS-Ins-1.3B gets 59.8% HumanEval+ but only 26.3% LCB-Easy


FAILURE MODES (from paper)

1. Contaminated evaluation: evaluating on problems released BEFORE model cutoff date inflates scores
2. Benchmark overfitting: models optimized on HumanEval do NOT generalize to LCB (low 0.72 correlation)
3. Sample size noise: small post-cutoff windows (<200 problems) create 1-1.5% measurement noise
4. Python-only limitation: benchmark does not cover other programming languages
5. Competition domain gap: competition problems != real-world SWE tasks; results not fully general
6. CoT prompt sensitivity: open models sometimes perform WORSE with CoT on code execution (S7.p5)
7. Format variance: no per-model prompt tuning; results may understate some models' true capability


IMPLEMENTATION PITFALLS (do NOT do this)

1. Do NOT use May23-May24 full window for contaminated models (DeepSeek, GPT-4-O, Codestral);
   use post-cutoff window only
2. Do NOT infer contamination from a single month's data; look for step-change pattern across months
3. Do NOT compare models with <2 point Pass@1 difference as statistically reliable
4. Do NOT apply 9-gram decontamination as a substitute for temporal windowing; the paper explicitly
   avoids n-gram approaches as insufficient (rephrasing evades them, S1.I1.i1)
5. Do NOT include AtCoder arc/agc contests (only abc; harder rounds excluded)
6. Do NOT include CodeForces Div 1/Div 2 (only Div 3 + Div 4)
7. Do NOT include AtCoder problems rated >500 or CodeForces problems rated >1300
8. Do NOT exclude CodeForces entirely; include but note it is a small subset (9 problems at paper time)
9. Do NOT implement task retirement based on "uniformly-passing" criterion; paper does not define this


COMPLEXITY

Problem collection: O(weekly scrape per platform), ~4 problems per LeetCode contest
Test generation: GPT-4-Turbo generator-based approach when platform tests unavailable
Evaluation: O(N_problems * 10_samples * N_tests) per model; parallelizable
Self-repair evaluation depends on code generation results (sequential dependency)


D-6 SCOPE ALIGNMENT

This paper provides the ground truth for the "held-out task store by date" pattern in D-6.
The contamination-resistance mechanism IS the date-windowing: store problems with release_date,
filter evaluation window to [model_cutoff, now].
The 9-gram test mentioned in D-6 scope description appears to be an EXTENSION beyond this paper
(possibly from related work Shi et al. 2023 or community implementations). Treat as UNGROUNDED
unless a separate source is ingested.
Similarly, "task retirement for uniformly-passing tasks > 6 months" is NOT in this paper.
