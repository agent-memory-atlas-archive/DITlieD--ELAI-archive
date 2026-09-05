Algorithm: NL2VC tiered prompting for Dafny-verified code generation
ArXiv: 2604.22601
Paper: From Natural Language to Verified Code: Toward AI Assisted Problem-to-Code Generation with Dafny-Based Formal Verification
Authors: Md Erfan, Md Kamal Hossain Chowdhury, Ahmed Ryan, Md Rayhanur Rahman
Verified: 2026-05-24 via WebFetch arxiv.org/abs/2604.22601 + arxiv.org/html/2604.22601 + WebSearch cross-confirmation
Verified status: PAPER_INGESTED (file: F:\.ELAI_workflow\.claude\papers\2604.22601.html; RAG ingest deferred -- IPC timeout, known RAG reranker crash per memory rag_reranker_crash)

Inputs:
problem_description | str | natural language algorithmic spec from UVa Online Judge, avg 179+ words | mandatory
method_signature | str | Dafny method signature with preconditions/postconditions | optional in RQ1 (contextless), mandatory in RQ2/RQ3
verifier_feedback | str | Dafny 4.11.0 verifier error output from previous attempt | mandatory only in RQ3 self-healing tier
temperature | float | T in {0.0, 0.2, 0.4, 0.6, 0.8} | mandatory
k | int | verify@k, k in {1, 3, 5} | mandatory
model_id | str | one of seven open-weight LLMs | mandatory
max_heal_iter | int | self-healing max iterations | default 10
response_timeout | int | seconds | default 180

Outputs:
dafny_source | str | one Dafny source file, .dfy | must compile under Dafny 4.11.0 + Z3
verify_result | bool | true iff Dafny verifier reports zero errors AND uDebug functional tests pass | non-vacuous (uDebug guards against trivial-spec passes)
heal_trace | list[str] | iterative repair history when RQ3 used | length <= max_heal_iter

Hyperparameters (per paper):
temperature | sweep {0.0, 0.2, 0.4, 0.6, 0.8} | optimal varies per model (Gemma 4-31B peak at T=0.2/0.6 RQ3a, T=0.6/0.8 RQ3b)
max_heal_iter | 10 | hard cap on self-healing rounds
response_timeout | 180s | per-attempt LLM response cap
verify_k | 5 | strongest signal; verify@1/3/5 all reported

Critical invariants (testable assertions):
assert verify_result is True implies dafny_source has compiled successfully under Dafny 4.11.0
assert verify_result is True implies uDebug functional tests pass (guards vacuous verification)
assert tier in {"contextless", "signature", "self_healing"}
assert temperature >= 0.0 and temperature <= 0.8
assert len(heal_trace) <= max_heal_iter
assert k in {1, 3, 5}

Complexity:
time | O(k * max_heal_iter * (LLM_inference + dafny_verify)) per problem; verify dominates on hard problems | parallelism via verify@k independent samples
space | bounded by single Dafny source size (<~10KB typical) + iter history
parallelism | k samples per problem are independent; healing iterations are strictly sequential

Failure modes (paper section: Failure Mode Analysis):
syntactic_fragility | models without structural anchors generate Pythonic indentation, misuse Dafny keywords | RQ1 contextless near-universal failure
semantic_drift | correct imperative logic without inductive loop invariants -> Z3 cannot close proof
vacuous_verification | weak specs trivially satisfied (constant return) | uDebug platform required to detect
contextual_dependence | Dafny compiler errors too abstract for model to navigate without signature scaffolding

Implementation pitfalls (do NOT do this):
do NOT skip uDebug functional validation -- raw Dafny pass != correctness, vacuous specs are a documented attack
do NOT remove method signature from RQ2/RQ3 input -- contextless tier shows near-universal failure across 6 of 7 models
do NOT use Gemma 4-31B in ELAI sidecar -- ruled out per memory feedback_no_gemma4.md; reference Gemma 4-31B results are PAPER-ONLY, not for ELAI integration. Substitutes for ELAI use: Qwen 3 Coder 30B (45.55% RQ2 verify@5) or Codestral-22B (63.64% RQ2 verify@5)
do NOT exceed max_heal_iter=10 -- paper does not characterize diminishing returns past 10
do NOT assume single temperature optimum -- optimal T varies per model and per tier
do NOT report raw verify@k without uDebug pass rate -- vacuous-pass inflation otherwise

Headline numerical results (verify@5, peaks across temperature sweep):
RQ1 contextless: Gemma 4-31B 54.55%; Codestral-22B 27.27%; all others 0%
RQ2 signature: Qwen 3.5-9B 72.73% (T=0.8); Gemma 4-31B 63.64%; GPT-OSS-120B 63.64%; Codestral-22B 63.64%; Qwen 3.6-35B 63.64%
RQ3a self-heal contextless: Gemma 4-31B 90.91% (T=0.2 and T=0.6)
RQ3b self-heal signature-guided: Gemma 4-31B 81.82%; GPT-OSS-120B 81.82%

Seven models evaluated (Table III, verbatim names):
GPT-OSS-120B
Qwen 3.6-35B-A3B (MoE)
Gemma 4-31B (multimodal, 31B)
Qwen 3 Coder 30B
Codestral-22B-v0.1
GPT-OSS-20B
Qwen 3.5-9B

Dataset: NL2VC-60
60 algorithmic problems from UVa Online Judge with ground-truth Dafny + uDebug test suites
~300 person-hours authorship + 50h conflict resolution

Infrastructure (paper Table II):
4x NVIDIA RTX 6000 Ada (48GB each, 192GB total VRAM)
NVIDIA driver 580.126.09, CUDA 13.0
LM Studio v0.4.8 build 1
Dafny 4.11.0, Z3 (paper-stated), Python 3.14.4

GEMMA-4 FLAG (per memory feedback_no_gemma4.md):
The paper itself uses Gemma 4-31B as one of seven evaluated models and reports it as the top performer in RQ3a (90.91% verify@5) and RQ3b-tied (81.82%). Citing the paper's results is fine. INTEGRATING Gemma 4 into ELAI sidecar is RULED OUT by user feedback. If C-52 plan claims to ship "Gemma 4-31B integration" based on this paper, that is a misread of the citation -- the paper compares models; it does not prescribe one. ELAI substitutes (per memory): Qwen 2.5 Coder series for sidecar; for Dafny-style verification roles, Qwen 3 Coder 30B or Codestral-22B-v0.1 are the closest non-Gemma open-weight options from the paper's evaluation set, with verify@5 of 45.45% and 63.64% respectively on signature tier.

C-52 plan grounding recommendation:
- Use this paper as DESIGN-EVIDENCE for tiered prompting (contextless -> signature -> self-healing) and uDebug-style vacuous-spec guard.
- Cite verify@k = 5 as the headline metric.
- DO NOT carry over "Gemma 4-31B" model selection into ELAI; substitute Qwen 3 Coder 30B (closest non-Gemma top-tier on signature) or Qwen 2.5 Coder 7B (current ELAI sidecar baseline) and document expected verify@5 degradation.
- The paper does NOT prescribe a specific runtime model for downstream use; results are cross-sectional comparison.
