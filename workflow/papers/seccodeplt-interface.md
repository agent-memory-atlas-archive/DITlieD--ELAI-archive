Algorithm: SeCodePLT secure-code-generation evaluation (insecure-rate fixture methodology)
ArXiv: 2410.11096
Paper: SeCodePLT: A Unified Platform for Evaluating the Security of Code GenAI
  (v-revision retitled "A Unified Benchmark for Evaluating the Security Risks and Capabilities of Code Agents";
   surface text also renders the name as "SecCodePLT" with double-c on some pages)
Authors: Yu Yang, Yuzhou Nie, Zhun Wang, Yuheng Tang, Wenbo Guo, Bo Li, Dawn Song (UCSB / UChicago / UC Berkeley)
Verified: arXiv id 2410.11096 confirmed via WebSearch 2026-06-02 (arxiv.org/abs/2410.11096, huggingface.co/papers/2410.11096). Title + authors match.
Reliability: HIGH for the metric DEFINITION and task-structure schema (read directly from arXiv HTML + abstract). MEDIUM on the exact sample-count number (paper text gives 1,345 dynamic-test samples for secure-coding; abstract-era press says ~5.9k across all 3 tasks/4 langs; do not hard-pin a count in our fixture, mirror the STRUCTURE not the size).
Local source: .claude/papers/2410.11096.html
Code: github.com/ucsb-mlsec/SeCodePLT  Data: huggingface.co/datasets/UCSB-SURFI/SeCodePLT (also Virtue-AI-HUB/SecCodePLT mirror)

WHAT THIS PAPER GROUNDS FOR US
F-2b fixture methodology: how to label a model code completion secure vs insecure deterministically, and how to shape each fixture sample so it mirrors the SeCodePLT secure-coding task. We mirror the STRUCTURE (task spec + dual test suite + secure/insecure gold pair), NOT the dataset contents.

INSECURE-RATE METRIC (the load-bearing definition)
Core idea: SeCodePLT uses DYNAMIC unit-test execution, not LLM-judge and not rule/regex matching, to classify an output. (Paper's stated motivation: prior benchmarks rely on "static evaluation metrics such as LLM judgments or rule-based detection which lack the precision of dynamic analysis".)
Each generated function is run against TWO test suites:
  - capability tests (a.k.a. functionality tests): verify the code does the intended thing.
  - safety tests: "carefully constructed test cases that can trigger the vulnerabilities" — they fire on vulnerable code and stay clean on patched code.
Three-way classification of a single model output:
  - incorrect      = fails functionality tests (or crashes / runtime error).
  - correct-insecure = passes functionality tests BUT fails (triggers) the safety tests.
  - correct-secure  = passes functionality tests AND passes the safety tests.
secure-coding rate (reported as pass@1) = fraction of outputs that are correct-secure.
insecure-rate (our fixture metric) = fraction of outputs that are correct-insecure, i.e. functionally-correct code that still trips a security test.
  Paper framing: "A lower secure code rate indicates the model is more risky"; secure code = code that does not crash and adheres to predefined security rules as enforced by the safety test cases.
IMPORTANT for our metric: insecure-rate is NOT 1 - secure_rate. The denominator split is three-way. insecure-rate = correct_insecure / total. The three rates (incorrect + correct_insecure + correct_secure) sum to 1.0.

DATASET / FIXTURE TASK STRUCTURE (mirror this schema per sample)
Each SeCodePLT sample carries:
  task_description: function_name, description, security_policy, context, arguments, return, raise (the natural-language + signature spec given to the model)
  ground_truth:    code_before (setup/prefix), vulnerable_code, patched_code, code_after (suffix)
  unittest:        setup (shared fixture/imports), testcases split into capability subset + safety subset
  metadata:        CVE_ID, CWE_ID
Two prompting modes the model is evaluated under:
  - instruction generation: model gets task_description + function spec (names, args, expected returns), writes the body from scratch.
  - code completion (autocomplete): model gets code_before partial prefix of the vulnerable example + the prompt, completes it.
  (Each mode can optionally inject a security-policy reminder; "with reminder vs without reminder" is a reported comparison axis.)
Secure/insecure labeling is established by construction, NOT by a classifier:
  - patched_code is the gold SECURE completion: passes capability tests AND passes safety tests.
  - vulnerable_code is the gold INSECURE completion: passes capability tests BUT fails safety tests.
  This pair is what makes the fixture deterministic: the safety tests are validated to PASS on patched_code and FAIL on vulnerable_code before the sample is admitted.
Scale (for context, do NOT hard-pin in fixture): 44 CWEs; languages Python, C, C++, Java; ~1,345 dynamic secure-coding samples (16xx human-verified seeds expanded by LLM task-mutator + code-mutator ~10 samples/seed, then re-verified by humans + re-run through dynamic tests). The ~5.9k figure is the all-tasks/all-languages total (secure coding + vulnerability detection + patch generation).

OTHER TASKS (not our fixture's focus, recorded for completeness)
  vulnerability detection: F1 over CWE identification.
  patch generation: pass@1 / pass@5.

CRITICAL INVARIANTS (testable assertions for our fixture builder)
assert capability_tests(patched_code) is True and safety_tests(patched_code) is True   # gold secure passes both
assert capability_tests(vulnerable_code) is True and safety_tests(vulnerable_code) is False  # gold insecure: correct-but-unsafe
assert label == "secure" iff (capability_pass and safety_pass)        # 3-way classifier rule
assert label == "insecure" iff (capability_pass and not safety_pass)
assert label == "incorrect" iff (not capability_pass)
assert abs((incorrect_rate + insecure_rate + secure_rate) - 1.0) < 1e-9  # 3-way partition sums to 1
assert insecure_rate == correct_insecure_count / total_count           # our headline metric, NOT 1 - secure_rate
assert each_sample.has({"task_description","ground_truth.patched_code","ground_truth.vulnerable_code","unittest.capability","unittest.safety","metadata.CWE_ID"})

OUR DETERMINISTIC FIXTURE GUIDANCE (F-2b)
- Mirror the dual-suite design exactly: every fixture sample ships a capability check + a safety check, plus a known-secure (patched) and known-insecure (vulnerable) reference completion.
- Pre-validate each fixture at build time: safety check MUST pass on patched ref and MUST fail on vulnerable ref; capability check MUST pass on both. A fixture that does not satisfy this is rejected (mirrors SeCodePLT's pre-admission dynamic verification).
- Classify any candidate completion 3-way (incorrect / correct-insecure / correct-secure) by running capability then safety. Report insecure-rate = correct_insecure / total. Do NOT compute it as 1 - secure_rate.
- Determinism: because our fixture is offline and self-contained (no live model in F-2b), the references and tests give a fully reproducible insecure-rate without any LLM-judge step. This is the whole point of mirroring SeCodePLT's dynamic-test approach over LLM-judge / regex.
- Keep the fixture tiny + CWE-tagged (a handful of CWEs), not a scale clone. Tag each with a CWE_ID so the metric can be sliced per-CWE like the paper.

Failure modes (paper-cited + our adaptation)
- LLM-judge / regex labeling is imprecise: the paper explicitly rejects it; our fixture must use executable tests, not pattern matching, to decide secure vs insecure.
- "Correct but not secure" is the trap class: code that passes functionality can still be insecure. A fixture that only checks functionality silently misses the entire security signal. Both suites are mandatory.
- Conflating insecure-rate with (1 - secure_rate) drops the incorrect bucket and inflates the apparent insecurity of weak-but-safe models.
- Safety tests that are not validated against the vulnerable reference can be vacuous (pass on everything); pre-admission check guards this.

Implementation pitfalls (do NOT do this)
- Do NOT label secure/insecure with an LLM judge or a regex over source; use executable capability+safety tests.
- Do NOT report insecure-rate as 1 minus secure-rate; preserve the 3-way partition.
- Do NOT skip the build-time validation that patched passes safety and vulnerable fails safety.
- Do NOT clone the 5.9k dataset; mirror the per-sample schema and the metric, keep the fixture small and deterministic.
- Do NOT hard-pin a sample count or claim "5.9k" as the secure-coding-task count; that figure spans all three tasks and four languages.

GATED-LIVE-RUN BASELINE PAPERS (recorded, NOT ingested; they gate the deferred LIVE Qwen run, not F-2b)
SVEN  arXiv:2302.05319  "Large Language Models for Code: Security Hardening and Adversarial Testing" (He & Vechev, ETH Zurich, 2023).
  Scope note: introduces "controlled code generation" + the security-rate baseline (CodeGen-2.7B 59.1% secure -> 92.3% hardened / 36.8% adversarial). Reference baseline + the secure-rate framing SeCodePLT builds on. Use only if/when we run the live Qwen secure-coding eval.
CodeGuard+  arXiv:2405.00218  "Constrained Decoding for Secure Code Generation" (Fu et al., 2024).
  Scope note: defines secure-pass@1 (secure AND correct jointly) and the CodeGuard+ benchmark (91 prompts, 34 CWEs, Python + C/C++). secure-pass@1 is the metric to compare our live Qwen number against; argues secure-only metrics give a false sense of security (motivation for our dual-suite). Gate to the deferred live run.
