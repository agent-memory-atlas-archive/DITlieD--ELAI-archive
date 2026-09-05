Algorithm: Structure Snowballing (failure-mode characterisation for constrained-decoding self-critique reflection)
ArXiv: 2604.06066
Paper: From Hallucination to Structure Snowballing: The Alignment Tax of Constrained Decoding in LLM Reflection
Author: Hongxu Zhou (Saarland University, EMLCT)
Repo: https://github.com/hongxuzhou/agentic_llm_structured_self_critique
Reliability: HIGH (peer-reviewed arXiv preprint, primary source, single 8B model study with full appendix + Pool A/B raw counts)

Headline claim (the C-37 grounding line):
Imposing Outlines-style FSM constrained decoding on an unfinetuned 8B reflector does NOT improve self-correction. It REPLACES hallucination-snowballing with a new failure mode, "structure snowballing" (death-loops of FORMATTING_MISMATCH diagnoses) and DROPS pool-merged accuracy from 0.50 -> 0.38 (McNemar p ~= 0.059, n=100, 23 correct->wrong vs 11 wrong->correct).

Inputs (Reflector role I/O contract per Section 3.3 + Appendix):
question | str | HotpotQA-distractor format (2 gold paragraphs + 8 distractors)
failed_trajectory | str | full Actor reasoning chain from prior trial (Thought + Action lines)
episodic_memory | list[dict] | accumulated correction rules from previous reflections, JSON-serialised
gold_answer_hint | none | reflector receives binary Evaluator signal only (NOT gold string)
schema | pydantic.BaseModel | Error_Type (Literal-of-5) + Correction_Rule (str), compiled via Outlines into an FSM mask applied to logits

Outputs (Reflector schema-bound):
error_type | Literal["RETRIEVAL_FOCUS","BRIDGE_FAILURE","HALLUCINATION","INFERENCE_ERROR","FORMATTING_MISMATCH"] | exactly one of 5 enum members, 100% schema adherence guaranteed by FSM mask
correction_rule | str | free-text remediation hint appended to episodic_memory
invariants:
  - sum_one_hot(error_type) == 1 (exclusive)
  - len(correction_rule) > 0
  - output parses as valid JSON against the pydantic schema (zero parse failures by construction)

Per-role decoding defaults (the C-37 numbers that bind to PER_ROLE_DECODING_DEFAULTS):
Actor (M_a)      | temperature=0.1 | max_new_tokens=1024 | sampler=greedy-with-tiny-stochasticity | constrained=NO  | model=Qwen3-8B
Evaluator (M_e)  | temperature=0.1 | max_new_tokens=1024 | sampler=greedy-with-tiny-stochasticity | constrained=NO  | model=Qwen3-8B (same instance; semantic-equivalence judge only)
Reflector (M_r)  | temperature=0.1 | max_new_tokens=1024 | sampler=greedy-with-tiny-stochasticity | constrained=YES via Outlines+pydantic FSM | model=Qwen3-8B
shared           | max_trials_per_question=5 | upstream-first attribution (only the FIRST fatal error is classified per Section 3.2)
note: top_p / top_k / seed values NOT reported in the paper text or tables — only temperature 0.1 and max_new_tokens 1024 are explicit; reproduction must treat top_p/top_k/seed as unspecified hyperparameters and either default to Hugging Face Qwen3-8B generation_config.json values OR re-derive from the repo at hongxuzhou/agentic_llm_structured_self_critique

5-class error taxonomy (Section 3.1, 4+1 design):
RETRIEVAL_FOCUS    | model picks wrong paragraph/sentence at the start
BRIDGE_FAILURE     | correct first hop, fails to extract bridge entity for next hop
HALLUCINATION      | generates facts not in context (per ROSCOE Golovneva et al. 2023)
INFERENCE_ERROR    | correct retrieval, wrong logical/arithmetic deduction
FORMATTING_MISMATCH| correct conclusion, output violates required format
(intentionally capped at 5 — paper cites Nathani et al. 2023 on cognitive overload at ROSCOE's 10-class granularity for sub-70B models)

Hyperparameters (paper-explicit only; UNGROUNDED to assume others):
temperature           | 0.1   | (0, 2) typ. | "minimise random variance" (Section 4.3)
max_new_tokens        | 1024  | >=512 useful | hard ceiling per trial
max_trials            | 5     | 3..10 typ.  | "maximum allowance of five trials" (Section 5.2 sample 5a8c7595...)
schema_class_count    | 5     | 3..10       | author warns: 10-class ROSCOE causes overload on 8B
taxonomy_coupling     | upstream-first | bool | only first fatal error classified per trial; downstream errors ignored

Eval datasets:
HotpotQA distractor setting | 1000 sample pilot -> 631 first-try correct -> 369 remain -> further filtered by LLM-as-judge semantic-equivalence -> 100 hard samples
Pool A | 55 samples | baseline (free-text Reflexion) solved between trial 2 and trial 5 | primary metric: Avg Trials (AT)
Pool B | 45 samples | baseline completely failed (often death-looped) within 5 trials | primary metric: Success Rate (SR)
LLM-judge: Qwen3-8B as same-model semantic-equivalence judge (NOT a separate critic); explicitly bounded to entailment, not quality ranking

Model class (canonical from paper):
Qwen3-8B (Yang et al. 2025, arXiv:2505.09388 Qwen3 Tech Report) — used for ALL three roles (Actor, Evaluator, Reflector); intentional single-model setup to "prevent capability mismatches and mitigate intra-model scoring bias"

Reproduction config (minimum viable for C-37 harness):
model               | huihui-ai/Qwen3-8B OR Qwen/Qwen3-8B (abliterated equivalent acceptable for autonomous-agent use)
quantisation        | NOT specified; paper used unquantised; NF4 4-bit reproduction is a deviation, document it
constrained_lib     | Outlines (Willard & Louf 2023) compiled from a pydantic BaseModel with 5-Literal Error_Type field
attribution         | upstream-first: reflector emits only ONE Error_Type per trial, targeting the FIRST upstream error
episodic_memory     | append-only list of {error_type, correction_rule} dicts, prepended to Actor prompt on next trial
evaluation_metric   | per-trial accuracy (binary) + Avg Trials + Success@1 + Success@<=2 + token consumption per (Status_t-1, Status_t) bucket
seed                | NOT reported (treat as a known unknown; pin in your repro and note divergence from paper)
top_p / top_k       | NOT reported; do not invent values — either use Qwen3-8B HF defaults or check the GitHub repo

Critical invariants (testable assertions, Python syntax):
assert error_type in {"RETRIEVAL_FOCUS","BRIDGE_FAILURE","HALLUCINATION","INFERENCE_ERROR","FORMATTING_MISMATCH"}
assert 0 < trial_idx <= 5
assert isinstance(correction_rule, str) and len(correction_rule) > 0
assert json.loads(reflector_output) is not None  # schema-adherence guaranteed by FSM
assert formatting_mismatch_rate_first_round >= 0.90  # paper reports 96/100 in trial 1; replication should land >=0.90 on Pool A+B
assert constrained_avg_tokens_per_trial > baseline_avg_tokens_per_trial  # alignment tax: paper reports 3183.48 vs unstated baseline; TF-group surge 4005.5 vs TT-group 2850 inside the constrained run
assert constrained_pool_accuracy <= 0.45  # paper: 0.38; replication should NOT exceed 0.45 or methodology has diverged
assert death_loop_count >= 50  # paper: 58 samples stuck in identical formatting-error loop across all 5 trials

Quantitative anchors (Tables 1-4 + Section 5):
Baseline accuracy                              | 0.50
Constrained accuracy                           | 0.38  (delta -0.12, McNemar p~=0.059, marginally significant)
Baseline Avg Trials                            | 0.63
Constrained Avg Trials                         | 0.41
Constrained Success@<=2                        | 0.80   (when it works, it works in <=2 trials — the recovered 11 cases)
Status flips (n=100): TT=27, TF=23, FT=11, FF=39
Error-type distribution trial 1: FORMATTING_MISMATCH=96, RETRIEVAL_FOCUS=4, others=0
Token consumption (constrained): mean 3183.48, max 8976.25, std 989.11
Token consumption split: TT-group 2850 mean, TF-group 4005.5 mean (~+40% before failure)
Death-loop count: 58/100 samples stuck repeating identical formatting diagnosis across all 5 trials

Complexity:
Time  | O(T * (gen_actor + gen_evaluator + gen_reflector)) per question, T<=5 | Outlines FSM mask adds O(|V|) logit mask per token, negligible vs forward pass
Space | episodic_memory grows linearly with T; bounded by max_new_tokens * T  | FSM index over vocab is O(|V| * |states|), built once per schema

Failure modes (cited by the paper):
1. Structure snowballing (the paper's central finding) — reflector locks onto FORMATTING_MISMATCH category for shallow string-match issues and never reaches RETRIEVAL_FOCUS / BRIDGE_FAILURE / INFERENCE_ERROR / HALLUCINATION diagnoses
2. Death loops — 58/100 samples emit identical formatting "correction rule" 5 times in a row, never breaking out
3. Alignment tax — token count climbs linearly across trials (sample 5a8c7595...: 2075 -> 3190 tokens) as model consumes capacity on syntax compliance, leaving none for multi-hop reasoning
4. Survivor-bias artefact — Section 6 warns: after baseline saturates HotpotQA, residual errors skew toward metric-brittleness rather than reasoning failures, exaggerating the FORMATTING_MISMATCH rate. C-37 harness MUST mitigate or document
5. Single-model scoring loop — using Qwen3-8B as its own judge creates intra-model bias; paper bounds judge to "semantic equivalence only" to mitigate, NOT to score quality

Implementation pitfalls (do NOT do this):
- Do NOT use a larger taxonomy than 5 classes when reproducing on a sub-70B model (paper-explicit warning citing Nathani et al. 2023)
- Do NOT skip upstream-first attribution — the paper treats this as a load-bearing component (Section 3.2). Allowing the reflector to classify multiple errors per trial breaks the hallucination-snowballing-disruption hypothesis
- Do NOT swap in a different constrained-decoding library (xgrammar, lm-format-enforcer, etc.) without documenting that the result no longer replicates Willard & Louf 2023 Outlines FSM semantics
- Do NOT use a temperature higher than 0.1 — paper-explicit choice to minimise variance; higher temps invalidate the "alignment tax" claim because token thrash becomes confounded with sampling stochasticity
- Do NOT silently substitute Qwen3-7B / Qwen2.5-Coder-7B / Qwen2.5-7B for Qwen3-8B — the "8B" parameter count is named in the title and Section 6 limitation; a different 7B model is a different study
- Do NOT report only mean accuracy — the contribution is the STATUS-FLIP MATRIX (TT/TF/FT/FF) and the FORMATTING_MISMATCH=96% diagnosis collapse. Without these the paper's contribution is invisible
- Do NOT add a dynamic-fallback / backtracking mechanism (Section 6 future work) and call it a replication — that's a NEW study, not a reproduction
- Do NOT cite this paper as evidence that "structured output is bad" — paper claim is bounded: structured output WITHOUT external training, on 8B-class models, with rigid string-match evaluation, can degrade accuracy. Larger models, trained critics, or execution-based evals are out of scope per Section 6

C-37 binding notes (specific to the ELAI executor harness):
- The "REFLECTION role" default in C-37 plan body line 29 maps to Reflector (M_r) in this paper. The paper's reported decoding config is temperature=0.1, max_new_tokens=1024, constrained=YES (Outlines+pydantic FSM, 5-class Literal). No other per-role hyperparameter values appear in the paper text.
- The "8B model" mention in C-37 plan body line 38 = Qwen3-8B specifically. If the C-37 executor runs on Qwen2.5-Coder-7B-Instruct-abliterated (the ELAI remote-sidecar default), document the model substitution as a known deviation; the structure-snowballing finding may transfer but the 0.38 / 96% numbers are not portable
- The paper's "observable metric" for structure snowballing is the DEATH-LOOP COUNT + FORMATTING_MISMATCH first-round rate, NOT cascading-error rate. C-37 contract tests should bind to these two metrics, not to a generic hallucination-amplification curve
- Cache cross-check: oracle-elai-competitive-papers-2026-05-17 already labels this paper as "counter-evidence for ELAI C-26 critic=JsonSchema default". That cached note is consistent with the full-text read above
