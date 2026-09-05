Source: agentic_llm_structured_self_critique (companion repo to arXiv:2604.06066)
Reliability: NONE (repo could not be fetched — fell back to paper-only grounding at MEDIUM reliability for the algorithm interface)
Files examined: F:/.ELAI_workflow/.claude/papers/2604.06066.html (paper HTML, 157 KB, fetched 2026-05-23)

REPO FETCH FAILURE (documented per agent spec)

Target URL: https://github.com/hongxuzhou/agentic_llm_structured_self_critique
Result: HTTP 404 from github.com/<user>/<repo>, HTTP 404 from api.github.com/repos/<user>/<repo>, "Repository not found" from git ls-remote, zero archived snapshots in archive.org Wayback Machine, zero results in github.com/search/repositories for the exact name or the keywords "structure snowballing self-critique".

User hongxuzhou exists (https://api.github.com/users/hongxuzhou — 25 public repos: NLP coursework, fast-transformers, flash-linear-attention, illusion-of-state, lct-thesis, fallacy_detection, ssm_expressivity_with_undo, undo_flip_flop, amr-eager-multilingual, etc). NONE of the 25 repo names contain "agentic", "structured", "self_critique", "snowballing", "reflexion", "hotpotqa", or any plausible substring of the target name.

Paper itself is real and verifiable:
  arXiv:2604.06066v1 — "From Hallucination to Structure Snowballing: The Alignment Tax of Constrained Decoding in LLM Reflection"
  Author: Hongxu Zhou (Erasmus Mundus Master's in Language and Communication Technology)
  Submitted: 2026-04-07, license CC BY 4.0
  arXiv API confirms ID; abstract explicitly links the same dead github URL ("Code and raw logs are available in the GitHub repository: https://github.com/hongxuzhou/agentic_llm_structured_self_critique").
  Most plausible explanation: author made the repo private or deleted it between paper submission (Apr 2026) and now (May 2026). Less likely: typo'd URL in paper that was never live. Either way, no executable reference implementation is recoverable.

Action: fall back to paper-only grounding. The paper provides enough algorithmic detail (Sections 3.1 / 3.2 / 3.3 / 4.1 / 4.2 / 4.3 / 7.1) to write a contract test against the published method even without the repo. Numbers below are quoted from the paper text and tables, not inferred.

CRITICAL CORRECTION TO C-37 ASSUMPTIONS

The task brief assumed PLANNER / CODER / CRITIC / REFLECTION roles. The paper does NOT describe those roles. The actual ternary architecture (paper Section 3.3, verbatim) is:
  - Actor (M_a): policy function that generates a reasoning Trajectory ~ pi_Theta(y|x) given input context
  - Evaluator (M_e): assesses the final generated answer against ground truth, provides binary reward signal
  - Reflector (M_r): diagnoses root cause of error from failed trajectory + Episodic Memory M of accumulated correction rules

All three roles run on the SAME 8B model (Qwen3-8B). Paper Section 4.3 verbatim: "we deploy the Qwen3-8B model for both generation and evaluation tasks. Maintaining a uniform model for both the Actor and Evaluator roles prevents capability mismatches and mitigates intra-model scoring bias."

If C-37 wires PLANNER+CODER+CRITIC+REFLECTION as four distinct roles with different per-role params, it is NOT replicating this paper. C-37 spec should be reviewed before EXECUTING — the role taxonomy assumed in the task brief does not match the paper.

Public API (algorithm interface inferred from paper Sections 3.1-3.3, not from code)

  Function: run_logic_guided_reflexion(question, gold_paragraphs, distractors, max_trials=5)
    Inputs:
      question: str — natural language multi-hop question
      gold_paragraphs: list[str] — 2 paragraphs containing the answer (HotpotQA distractor setting)
      distractors: list[str] — 8 distracting paragraphs
      max_trials: int — paper uses 5 (Pool A successes are those solved between trials 2 and 5)
    Returns:
      dict { answer: str, n_trials: int, status: "solved"|"failed", trajectory: list[Trial],
             episodic_memory: list[CorrectionRule], final_error_type: ErrorType|None }

  Function: actor_step(question, context, episodic_memory) -> Trajectory  (paper: M_a)
    Decoding: free-text, temperature=0.1, max_new_tokens=1024 (Section 4.3 verbatim)

  Function: evaluator_step(question, trajectory_answer, gold_answer) -> bool  (paper: M_e)
    Constraint: "strictly limit the evaluator to verifying semantic equivalence, avoiding subjective quality ranking" (Section 4.3)
    Implementation: LLM-as-a-judge pipeline using same Qwen3-8B model (Section 4.1)

  Function: reflector_step(failed_trajectory, episodic_memory) -> CorrectionRule  (paper: M_r)
    Decoding: CONSTRAINED via Outlines library FSM derived from Pydantic schema (Section 3.3)
    Schema: { error_type: ErrorType, correction_rule: str }  (5-class enum + free-string)
    Attribution strategy: FIRST fatal error only — upstream-first attribution (Section 3.2)

  Enum ErrorType (paper Section 3.1, verbatim names + definitions, exactly 5 categories — "5-class" verbatim):
    1. RETRIEVAL_FOCUS — model identifies the wrong paragraph or sentence initially
    2. BRIDGE_FAILURE — model locates initial info but fails to extract bridge entity for next step
    3. HALLUCINATION — model generates facts not present in provided context
    4. INFERENCE_ERROR — model locates info but makes logical/arithmetic deduction error
    5. FORMATTING_MISMATCH — model derives correct conclusion but fails required formatting

Behavioral contracts (paper-grounded)

  Per-role decoding params (Section 4.3): temperature=0.1, max_new_tokens=1024, applied to ALL roles uniformly.
    There is NO per-role temperature/top_p/freq_penalty differentiation in the paper.
    If C-37 wants a CRITIC with different decoding than PLANNER, that is a C-37 design extension, NOT a replication of this paper.

  Reflector decoding is constrained, not free-text. The constraint is enforced via the Outlines library (https://github.com/outlines-dev/outlines) translating the Pydantic schema to a regex/FSM that masks the logit distribution at every decode step. Tokens that violate JSON syntax or fall outside the 5-class ErrorType enum get zero probability mass.

  Actor and Evaluator decoding are NOT constrained. They produce free text.

  Episodic Memory: accumulated list of CorrectionRule objects across failed trials within one question's lifetime. NOT cross-question persistent in the paper's reported experiments.

  Stopping condition: trajectory continues until Evaluator returns true OR max_trials reached.

  Upstream-first attribution: Reflector MUST identify only the FIRST fatal error in the failed trajectory, even if downstream errors are also present. Paper Section 3.2 cites BIG-Bench Mistake (Tyen et al. 2024) annotation principle.

Dataset and fixtures (paper Section 4.1)

  Dataset: HotpotQA distractor setting (Yang et al. 2018)
    2 gold paragraphs + 8 distractors per question
    Pilot: 1,000 samples from HotpotQA -> Qwen3-8B solves 631 first-try
    After LLM-as-judge filtering for semantic equivalence + difficulty: ~100 samples split into Pool A (55) + Pool B (45)
    Pool A: solved by baseline Reflexion in trials 2-5 — metric is Average Trials (AT)
    Pool B: completely failed by baseline — metric is Success Rate (SR)
  Schema (HotpotQA standard): {question, answer, supporting_facts, context: list of [title, sentences]}
  LLM-judge filter: Qwen3-8B as judge, verifies semantic equivalence to gold answer (avoids EM/F1 brittleness — e.g. "4" vs "four" false negatives)

Per-role decoding configuration (verbatim from paper Section 4.3)

  All roles (Actor, Evaluator, Reflector):
    model: Qwen/Qwen3-8B
    temperature: 0.1
    max_new_tokens: 1024
    other params: NOT specified in paper (no top_p, no frequency_penalty, no repetition_penalty mentioned)
  Reflector additionally:
    constrained_decoder: outlines library, FSM compiled from Pydantic schema
    schema: pydantic.BaseModel with error_type: Literal["RETRIEVAL_FOCUS","BRIDGE_FAILURE","HALLUCINATION","INFERENCE_ERROR","FORMATTING_MISMATCH"], correction_rule: str

Metric implementation ("structure snowballing" measurement)

  Paper does NOT define a single function that returns a "structure snowballing score". The phenomenon is characterized by the joint reading of THREE empirical tables (Section 7.1 Tables 1, 2, 3):

  Table 1 (Performance Comparison): accuracy drops from baseline 0.50 to constrained 0.38; avg trials 0.63 -> 0.41; success@1 = 0.00; success@<=2 = 0.80 in the constrained setting.

  Table 2 (Status Flip Analysis, n=100): Correct->Correct 27%, Correct->Wrong 23%, Wrong->Correct 11%, Wrong->Wrong 39%. Net flip = +12% degradation (23 - 11).

  Table 3 (Error Type Distribution under constrained reflection, n=100): FORMATTING_MISMATCH = 96%, RETRIEVAL_FOCUS = 4%, all other categories 0%. This 96% concentration in FORMATTING_MISMATCH is the operational definition of "structure snowballing" — when constrained decoding is applied, the reflector's diagnosed errors collapse almost entirely into the formatting bucket, masking the underlying semantic failures.

  Operational threshold for "structure snowballing observed":
    score = fraction_of_diagnosed_errors_in_FORMATTING_MISMATCH
    Paper's reported value: 0.96
    Suggested threshold for a contract test: score > 0.50 indicates the failure mode has manifested. Score > 0.90 is "severe snowballing" (paper-equivalent).

  Token cost signature (Table 4): constrained mean=3183.48 tokens, max=8976.25, std=989.11. High variance + heavy tail = "death loops" qualitative phenomenon described in Section 5.3.

Required runtime deps (paper-derived)

  Python: 3.10+ (Outlines + Pydantic v2 default to 3.10+)
  Models: huggingface Qwen/Qwen3-8B (16 GB bf16 / 5 GB int4) — paper does not say if quantized
  Libraries:
    transformers (any recent version supporting Qwen3 architecture, >= 4.50)
    outlines (https://github.com/outlines-dev/outlines, MIT licensed, last verified version line 0.x as of 2025)
    pydantic >= 2.0 (for schema -> FSM translation)
    datasets (huggingface) for HotpotQA distractor split
    torch (CUDA recommended; paper does not specify GPU)
  Sidecar reqs (for ELAI integration): Qwen3-8B does NOT fit on the 3090 24GB at full precision alongside the 7B sidecar already loaded. C-37 must either (a) replace the sidecar with Qwen3-8B for the duration of the snowballing test, or (b) load Qwen3-8B at 4-bit quant. Anti-Slopsquatting: verify Qwen/Qwen3-8B HF repo exists and Outlines supports its tokenizer before EXECUTING.

How to invoke a single replication run (paper-derived — repo is unavailable)

  Since the reference repo is 404, there is NO ready-made entry-point script. A replication must be authored from scratch. Pseudocode following the paper:

    python -m c37_snowballing.run \
      --model Qwen/Qwen3-8B \
      --dataset hotpotqa \
      --split distractor \
      --pool A \
      --n-samples 55 \
      --max-trials 5 \
      --temperature 0.1 \
      --max-new-tokens 1024 \
      --reflector-mode constrained \
      --schema-path schemas/correction_rule.py \
      --output runs/snowballing-poolA.jsonl

  Expected output schema (one JSON object per question):
    { question_id, n_trials, status: "solved"|"failed", final_answer, gold_answer,
      trajectory: [{trial, actor_output, evaluator_verdict, reflector_output: {error_type, correction_rule}}],
      avg_trials_at_solve, token_count }

Platform constraints

  Outlines FSM compilation is single-threaded and adds startup latency proportional to vocab size * regex size. For Qwen3-8B (vocab ~152K), expect 5-30s one-time FSM build per unique schema.
  Outlines hooks into the model's logit processor — only compatible with HuggingFace transformers generate() path, not vLLM or TGI without adapter. C-37 must NOT assume sidecar's normal Anthropic-protocol streaming endpoint will accept an Outlines processor — it would need a custom inference path.
  Tokenizer compatibility: Outlines requires accurate tokenizer.json. Qwen3 tokenizer is a BPE variant; verify Outlines >= the version that added Qwen3 support before adding the dep.

Gotchas (paper-derived warnings + repo absence)

  - The 96% FORMATTING_MISMATCH concentration is the paper's CENTRAL finding, not a bug. Any C-37 replication that does NOT reproduce this concentration (within ~10pp) has failed to reproduce the paper.
  - Constrained-reflexion accuracy is LOWER than baseline (0.38 vs 0.50). This is intended. C-37's PASS criterion cannot be "accuracy goes up" — that contradicts the paper.
  - The paper's Pool B SR finding (not extracted here in full): structured reflection helps Pool B (cases baseline completely failed) more than Pool A (cases baseline solved in 2-5 trials). C-37 should distinguish Pool A vs Pool B metrics.
  - Paper Limitations (Section 6): findings rely on a single 8B model. Scaling to 70B may invalidate the snowballing finding. Static-EM datasets like HotpotQA inherently bias toward FORMATTING_MISMATCH because residual errors after filtering are dominated by metric brittleness ("survivor bias in the error pool" — paper's words). C-37 should not generalize beyond the paper's exact configuration without further evidence.
  - Anti-Slopsquatting: BEFORE adding the `outlines` Python dep, verify on pypi.org/pypi/outlines/json that the package exists, version, and license (paper-cited but ELAI hasn't used it yet). Verify Qwen3-8B path on huggingface.co/Qwen/Qwen3-8B.
  - REPO ABSENCE risk: anything in C-37 that depends on "matching the reference repo's exact metric implementation" is unverifiable. Either accept paper-only grounding (and document the lower reliability in the contract test), reach out to the author hongxuzhou for the repo, or skip the structure-snowballing replication in favor of a different paper with a live repo.

Critical assertions (testable — Python syntax, paper-grounded)

  # Reflector output schema is strictly 5-class
  assert reflector_output["error_type"] in {"RETRIEVAL_FOCUS","BRIDGE_FAILURE","HALLUCINATION","INFERENCE_ERROR","FORMATTING_MISMATCH"}

  # Upstream-first attribution: reflector returns exactly one error_type per failed trial, not a list
  assert isinstance(reflector_output["error_type"], str)
  assert "correction_rule" in reflector_output and isinstance(reflector_output["correction_rule"], str)

  # Per-role decoding params are uniform (paper Section 4.3)
  for role in (actor, evaluator, reflector):
      assert role.config.temperature == 0.1
      assert role.config.max_new_tokens == 1024
      assert role.config.model_name == "Qwen/Qwen3-8B"  # or sidecar-loaded equivalent

  # Constrained reflector produces JSON-parseable output 100% of the time (FSM guarantee)
  for trial in failed_trajectory:
      json.loads(trial.reflector_output_raw)  # must not raise

  # Structure snowballing measurement reproduces paper Table 3 within tolerance
  error_dist = Counter(t.reflector_output["error_type"] for q in pool_a_results for t in q.trajectory if t.evaluator_verdict is False)
  formatting_fraction = error_dist["FORMATTING_MISMATCH"] / sum(error_dist.values())
  assert formatting_fraction > 0.50, f"structure snowballing not observed: {formatting_fraction:.2f}"
  # Paper-equivalent severity:
  # assert formatting_fraction > 0.90  # paper reports 0.96 — replication should hit ~0.85-0.97

  # Accuracy drop confirms alignment tax (paper Table 1)
  assert constrained_accuracy < baseline_accuracy, "alignment tax not observed"
  # Paper-equivalent magnitude: baseline 0.50 -> constrained 0.38, i.e. drop ~0.12
