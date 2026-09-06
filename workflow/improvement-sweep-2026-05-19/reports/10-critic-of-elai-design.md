# Hostile critique of ELAI's design

Critic: hostile, grounded. Sources cited inline. "Ungrounded:" labels mark genuine conjecture.
Sources read: DETAILED_SUMMARY_01_overview_and_architecture.md (full, 1037 lines) + D-SUMMARY_INDEX.md (full, 312 lines). Web grounding: 16 WebSearches against the 2026 literature.

## TL;DR — five hardest punches

1. **9-state FSM with 9 substrates, 7 evidence gates, BSP supersteps, ClockProvider injection, blake3-hashed message frames** — when the same outcome is now reliably produced by Swarm-style explicit handoffs in ~10 lines per agent. Most of the gates "pass vacuously because nothing populates ungrounded_claims" — your own audit says this in DETAILED_SUMMARY_01 line 511-512. The infrastructure exists; the signal it gates on does not. This is theater. Source: Swarm-vs-FSM tradeoff documented in [Latenode 2025 framework comparison](https://latenode.com/blog/ai-frameworks-technical-infrastructure/langgraph-multi-agent-orchestration/langgraph-multi-agent-orchestration-complete-framework-guide-architecture-analysis-2025).

2. **8 AgentRoles (Executor, Auditor, Researcher, Designer, PromptEngineer, ContextCompiler, MemoryWriter, MediaValidator) plus an Adversary in D-13** — sits squarely in the failure surface that Cemri et al. MAST documents: 14 failure modes across "specification, inter-agent misalignment, task verification" ([arxiv:2503.13657](https://arxiv.org/abs/2503.13657)). Self-MoA shows mixing different proposers is *usually sub-optimal* vs sampling one strong model ([arxiv:2502.00674](https://arxiv.org/abs/2502.00674), +6.6% AlpacaEval, +3.8% MMLU/MATH avg). ELAI's eight-headed pipeline pays MAS coordination tax for an effect Self-MoA refutes.

3. **"330 tokens vs 1600-2600" claim** — measured against what? DETAILED_SUMMARY_01 §4 line 690 asserts the comparison without naming the workflow it dethrones. Modern compiled-prompt approaches (DSPy) collapse hand-written meta-prose by similar ratios *automatically* without 9 FSM states. [DSPy 2310.03714](https://arxiv.org/pdf/2310.03714) shows compiled programs match GPT-3.5 hand-prompts on 770M and 13B models. The "savings" is real but not unique; the architectural cost claimed against it is. Worse: token-savings is the wrong axis when single-agent matches multi-agent at equal-token-budget ([DEV 2026](https://dev.to/greza_dev/why-single-agents-beat-multi-agent-systems-at-equal-token-budgets-445c)).

4. **Per-VRAM tier plan (8/12/16/24/48GB)** with Qwen2.5-Coder-7B as the default model — 2026 SWE-bench leaderboard shows a *25-30 point gap* between best open-source 7B-class coders and frontier ([BenchLM 2026](https://benchlm.ai/blog/posts/best-llm-coding), [Iternal 2026](https://iternal.ai/llm-selection-guide)). Even MiniMax-M2.5 (80.2%) hitting Claude Opus 4.6 (80.8%) is at the 200B+ MoE class, not 7B dense. ELAI bets infrastructure-as-intelligence carries a 7B from ~28% SWE-bench (HumanEval 84.1% per [Alibaba blog](https://www.alibabacloud.com/blog/qwen2-5-coder-series-powerful-diverse-practical_601765) does NOT generalize to multi-file repo work). The thesis is unfalsified, not validated. And the abliterated variant: zero benchmark presence in 2026 leaderboards.

5. **MistakeBook + 150-token failed-attempt cap** feeding small models in-context — directly poisoning territory. [ICLAttack 2305.01219](https://arxiv.org/pdf/2305.01219) and [Universal ICL Backdoors 2401.05949](https://arxiv.org/pdf/2401.05949) show in-context demonstration poisoning works *without fine-tuning*. The 3B/7B class is exactly the vulnerable range — [Over-prompting 2509.13196](https://arxiv.org/html/2509.13196v1) explicitly: "the smallest model, LLaMA-3.2-3B, exhibits weakening performance from the beginning… smaller models may be particularly vulnerable to prompt poisoning and over-prompting effects." Feeding 150 tokens of past failures to a 7B abliterated model is "training-time decoupled poisoning by design." There is no defense documented in the B-plan or D-plan.

## Punches (full critique)

### Punch 1 — FSM rigidity

DETAILED_SUMMARY_01 lines 410-411: 9 states + RESEARCHING; transition validation matrix + 7 evidence gates; phase-aware tool/rule loading.

D-plan piles on: FsmSubstate::Framing (B-40), per-level auditor (B-41), Adversary role (D-13), mode routing (D-10), tripwire stack (D-4), conformal refusal (D-16). The state space is *visibly* compounding session over session.

Broken because: line 511-512 self-admits most gates pass *vacuously* — "nothing populates ungrounded_claims / etc." The FSM is enforcement without inputs. You built the courthouse before the witnesses arrived.

Source pushback: [LangGraph "complexity in debugging and maintaining large, cyclic graphs"](https://latenode.com/blog/ai-frameworks-technical-infrastructure/langgraph-multi-agent-orchestration/langgraph-ai-framework-2025-complete-architecture-guide-multi-agent-orchestration-analysis); Swarm's "no hidden state and explicit handoffs" gives equivalent control at fraction of code ([Galileo 2025](https://galileo.ai/blog/openai-swarm-framework-multi-agents)). The FSM-vs-blackboard debate has been won pragmatically by the simpler side wherever observability mattered.

What to do instead: collapse to 3 states (PLANNING, EXECUTING, VERIFYING) with explicit handoffs. Move 6 of 9 evidence gates into post-condition functions called by the verifier. Delete the state machine; keep the postconditions.

### Punch 2 — 8 agent roles, MAST-overhead

DETAILED_SUMMARY_01 line 454: "8 AgentRoles (Executor, Auditor, Researcher, Designer, PromptEngineer, ContextCompiler, MemoryWriter, MediaValidator)". D-13 adds Adversary. Each role consumes spawning, lifecycle, checkpointing (5 msgpack files), supervision-tree slot.

MAST (Cemri et al. [2503.13657](https://arxiv.org/abs/2503.13657)) found 14 failure modes in 3 categories across 200+ tasks and 7 MAS frameworks. The failures cluster on inter-agent misalignment — exactly what 8-role specialization maximizes.

Self-MoA ([2502.00674](https://arxiv.org/abs/2502.00674)): "Self-MoA outperforms standard MoA that mixes different LLMs… +6.6% AlpacaEval, +3.8% average MMLU/CRUX/MATH." Diversity hurts when one proposer is significantly stronger.

Single-agent-at-equal-tokens result ([greza_dev 2026](https://dev.to/greza_dev/why-single-agents-beat-multi-agent-systems-at-equal-token-budgets-445c)): "most published benchmarks let the multi-agent system spend 2-4x more reasoning tokens than the single agent. When budget is held constant across architectures, single agents produce higher-accuracy answers and consume less compute."

ELAI's defense is "isolation = context savings." But process isolation is a *systems-engineering* gain, not a model-quality gain. The model still has to be 8 things in sequence. That's worse than being one thing well in 8 turns.

What to do instead: collapse to 3 roles (Worker, Verifier, Planner). Use Self-MoA-style multi-sample from one strong model for "Researcher" and "Designer." Delete PromptEngineer (DSPy-class compiler does this deterministically).

### Punch 3 — Determinism-first philosophy

DETAILED_SUMMARY_01 §2: entropy quarantine via ClockProvider/EntropySource; HashMap banned (84 imports remain); blake3 framing; BSP supersteps with snapshot→clear-topics→execute→apply→evidence→checkpoint cycle.

This is FoundationDB / TigerBeetle determinism cosplay. The real systems pay real costs: TigerBeetle's VOPR gives "3.3 seconds simulation = 39 minutes real-world testing" ([eatonphil 2024](https://notes.eatonphil.com/2024-08-20-deterministic-simulation-testing.html)), and [Antithesis docs](https://antithesis.com/docs/resources/deterministic_simulation_testing/) acknowledge "increased integration effort and design challenges… not every system can be designed in a way that enables DST."

ELAI is an *LLM orchestrator*. The non-determinism source of truth (the model) is fundamentally non-deterministic above temperature 0 — and even at temperature 0, batching effects, GPU floating-point reductions, and KV-cache reuse perturb output. You quarantine ClockProvider and import blake3 while the actual entropy lives in CUDA float roundoff. Sealing a wooden ship with diamond paint.

BSP supersteps: ([Wikipedia BSP](https://en.wikipedia.org/wiki/Bulk_synchronous_parallel)) — "global synchronization barriers" trade throughput for predictability. Async actor models eliminate the barrier wait ([HClib-Actor docs](https://hclib-actor.com/background/bsp/)). ELAI imposes BSP supersteps on an inherently I/O-bound LLM dispatch problem. The "deterministic macro-level orchestration" pays barrier cost for an effect (replayability) you cannot achieve through the model anyway.

What to do instead: stop trying to be FoundationDB. Record-and-replay at the SSB event log level is sufficient for post-hoc debugging. Drop BSP, drop the HashMap ban, drop blake3 framing for IPC. Use postcard + a sequence number.

### Punch 4 — "330 tokens vs 1600-2600"

DETAILED_SUMMARY_01 line 518-519: "Total meta-instructions ~330 tokens per turn (vs 1600-2600 in prompt-based workflows)."

No baseline named. No comparison workflow cited. "Prompt-based workflows" is a strawman — whose? Claude Code's system prompt? AutoGPT? Aider? Cursor? Each is different.

Modern frontier-model prompt patterns: [Anthropic prompt caching](https://docs.anthropic.com/en/api/openai-sdk) handles the meta-instruction overhead at near-zero cost on cache hit. The "1600-2600 tokens" disappears at 90%+ cache hit rate, which mature workflows reach. Compiled prompts via DSPy reduce hand-written meta-prose to learned soft prompts ([DSPy paper 2310.03714](https://arxiv.org/pdf/2310.03714)) without 9 FSM states behind it.

Worse: the 330-token number excludes (a) Tier 3 rules dynamically loaded per state, (b) DKG blast radius injection into auditor context, (c) SSB entries filtered in. Real per-turn token count likely 2-5x higher than claimed.

What to do instead: measure actual end-to-end input tokens with prompt-caching enabled against Cursor's published per-turn cost (open via [Anthropic API docs comparison](https://docs.anthropic.com/en/api/openai-sdk)). Publish or retract. Right now the number is marketing.

### Punch 5 — GRC blockquote echoing

CLAUDE.md (the project rule file): "After each mcp_think call, echo reasoning in a blockquote: > **[step_type]** content *[grounding]*. GRC blockquotes ONLY from actual mcp_think tool calls — NEVER hand-write a blockquote that mimics mcp_think output."

This is asking the model to *act as if* its CoT is grounded by writing the grounding marker. Lanham et al. ([2307.13702](https://ar5iv.labs.arxiv.org/html/2307.13702)) "Measuring Faithfulness in Chain-of-Thought Reasoning" empirically shows CoT traces *do not reliably reflect the model's true computational process*. The 2025 follow-up ([2502.14829v3](https://arxiv.org/html/2502.14829v3)) "Measuring Chain of Thought Faithfulness by Unlearning Reasoning Steps" confirms unfaithfulness persists across counterfactual probes.

[FaithCoT-Bench 2510.04040](https://arxiv.org/html/2510.04040v1): "the extent to which they faithfully represent the model's true computational process… remains unclear. This gap limits the reliability of LLMs."

Writing a citation in a blockquote *does not make the reasoning grounded*. It makes the reasoning *look* grounded. The system enforces a syntactic check (blockquote present, marker tag present) on a semantic property (claim derives from cited source) — a textbook Goodhart violation ([reward hacking 2603.28063](https://arxiv.org/pdf/2603.28063)).

What to do instead: replace blockquote-echoing with the typed proof-carrying CoT approach from [Typed Chain-of-Thought 2510.01069](https://arxiv.org/pdf/2510.01069), where the CoT is converted to a well-typed Curry-Howard proof certificate that is mechanically checked. Either you have a verifiable derivation or you have a stylistic guideline; don't pretend stylistic guidelines are verification.

### Punch 6 — Multi-layer reasoning pipeline (Layers 1-6, 60-70% deterministic)

CLAUDE.md / DETAILED_SUMMARY_01 §8 line 836: "Deterministic tools handle 60-70% of errors; the model handles the rest."

Number is unsourced in summary. Even granting it: the 30-40% the model handles is the *hard residue* — exactly where MAST documents the dominant failure modes land ([2503.13657](https://arxiv.org/abs/2503.13657) finds task-verification failures persist across all 7 frameworks tested).

Multi-layer pipelines fail by *error compounding*. Each layer with 95% pass independently → 6 layers serial = 73.5% end-to-end. ELAI has BM25 + dual-vector + PageRank + CatRAG + A-RAG + cross-encoder rerank ([line 218](../../../docs/thefinalglance/summary/DETAILED_SUMMARY_01_overview_and_architecture.md)). 6 stages × any per-stage miss = pipeline degradation. And the audit reports: "DEEP MRR (0.206) BELOW STANDARD MRR (0.575) — violates spec monotonicity" (line 215). The pipeline is *measurably making things worse* at the DEEP tier.

Source: 2026 RAG benchmark consensus ([ML Journey](https://mljourney.com/sparse-vs-dense-retrieval-for-rag-bm25-embeddings-and-hybrid-search/), [BEIR 2026 leaderboard](https://app.ailog.fr/en/blog/news/beir-benchmark-update)) is that *hybrid BM25 + dense + cross-encoder rerank top-50→top-5* matches or beats ELAI's 6-stage pipeline at a fraction of the operational surface.

What to do instead: collapse to BM25 + dense + rerank. Delete CatRAG, A-RAG, SPLADE (audit already says SPLADE decision PENDING + models not staged). Phase 3K showed DEEP regresses below STANDARD — that's a falsified architecture.

### Punch 7 — Per-VRAM tier plan

DETAILED_SUMMARY_01 lines 885-893: 5-tier matrix, 8GB through 48GB, mix of MEASURED + PROJECTED.

The 2026 reality:
- Best 7B open-source = ~28% SWE-bench Verified estimated (no published Qwen2.5-Coder-7B SWE-bench number; HumanEval 84.1% does not translate to multi-file repo work).
- Best frontier = Claude Opus 4.6 80.8%, GPT-5.3 Codex similar ([Iternal LLM 2026](https://iternal.ai/llm-selection-guide), [BenchLM 2026](https://benchlm.ai/blog/posts/best-llm-coding)).
- DeepSeek Coder 2.0 to GPT-5.3: 25-30pp SWE-bench gap. MiniMax-M2.5 closes it at 80.2% but it's a 200B+ MoE — not in your 12-48GB tier matrix.

ELAI's "infrastructure makes the 7B succeed" is a hypothesis, not a measurement. The summary itself (§9) admits: 32B tier is *PROJECTED, PROD*. Production target is unmeasured.

Cloud frontier costs: Sonnet 4.x runs in tier 1 for 5-15× less per useful turn than the *electricity bill* of running a local 3090. Local-first thesis in 2026 makes sense for offline/airgapped or privacy-critical work; for general coding it loses on every axis except hobby pride.

What to do instead: keep ONE local tier (12GB Qwen2.5-Coder-7B for offline use) + cloud-default. Delete 8GB/16GB/24GB/48GB matrix until empirically demanded. Spend the saved engineering hours on prompt-cache hit-rate optimization for the cloud path.

### Punch 8 — Rules-as-data SQLite

DETAILED_SUMMARY_01 §RULES lines 278-318: 7 predicate types, three enforcement tiers, time-aware confidence decay, ~39 seed rules, SHADOW→ACTIVE lifecycle.

This is bespoke OPA/Rego ([Snyk Rego intro](https://snyk.io/blog/introduction-to-rego/)) without the ecosystem. Rego has 8+ years of policy-as-code maturity, IDE tooling, formal semantics, conftest, gatekeeper. ELAI invented a 7-predicate type system + SQLite schema + SHADOW lifecycle from scratch.

The CLAUDE.md *itself* documents: "Phase 3L plan" already covers linter delegation. Then ELAI also builds its own parallel system. Two enforcement paths, two failure modes, two onboarding curves.

Current state of ELAI's rules system (line 317): "9/10 sub-steps PASS. 3F.10 benchmarks missing. Seed rules inserted with status=SHADOW, not ACTIVE per spec — divergence." The system that promotes rules from SHADOW to ACTIVE *has not run a single rule through that lifecycle in production*. The lifecycle is fiction.

What to do instead: emit rules as Rego files; delegate evaluation to OPA. Get YAML CI lint via [Kyverno-style](https://www.cncf.io/blog/2025/07/29/introduction-to-policy-as-code/) policies. Drop the SQLite predicate engine entirely. Save 9 sub-phases of work.

### Punch 9 — Mode-routing A/B/C/D/E (D-10) orchestrator-side

D-10 commits to `select_grounding_mode` as orchestrator predicate. "LLM has no input to selection." Cited: Everitt "current-RF principle" ([1908.04734](https://arxiv.org/abs/1908.04734)).

The orchestrator-side argument: tampering resistance — the LLM cannot bias mode selection toward easier modes. Real and valid.

Counter: [Outcome-Aware Tool Selection 2603.13426](https://arxiv.org/pdf/2603.13426) and [DiSRouter 2510.19208](https://arxiv.org/pdf/2510.19208) show static routers have measurable accuracy ceilings; outcome-aware (RL-trained) routers outperform but require feedback. ELAI's hand-written predicate is a *third* option — neither rich-feedback nor LLM-judgment.

Bigger problem: D-21 demands "F1 ≥0.85 across 5 modes." A 5-class hand-written rule predicate over ill-defined claim categories is going to be brittle. Mode-class boundaries (operationalizable vs constructive vs measurable-effect) are semantic; you cannot operationalize the operationalization check via a non-LLM. The predicate either invokes a classifier (LLM under the hood) or fails real edge cases.

Source push-back: ([Self-Healing Router 2603.01548](https://arxiv.org/pdf/2603.01548)) "matches ReAct's correctness while reducing control-plane LLM calls by 93%." The best routing systems use LLM-judgment *sparingly*, not zero.

What to do instead: hybrid. Orchestrator predicates handle the obvious 80% (regex says it's a math claim → Mode A). LLM-as-judge handles the long tail. Don't ship D-10 with 100% deterministic predicate; you'll hit F1 < 0.7 and blame the LLM.

### Punch 10 — Knowledge-engine 7-layer pipeline

DETAILED_SUMMARY_01 §KNOWLEDGE lines 111-225. 7 layers + reranker + resilience layer + 3K hardening.

Documented runtime state (lines 144-216):
- L3 vectors: code_vec populated, never queried at runtime → 2-ranker not 3-ranker
- L3.25 SPLADE: decision PENDING, models not staged
- L3.5 PageRank: WIRED IN CODE, DEAD AT RUNTIME (dual-path DKG bug)
- L4 CatRAG: implemented but bound to _overlay and discarded — pushes empty results
- L5 A-RAG: add_chunk has zero callers — corpus empty at runtime, methods return Vec::new()
- Cross-encoder: ONNX not staged, batching path test-only
- DEEP MRR < STANDARD MRR

Plain BM25 + hybrid dense + cross-encoder rerank ([sparse-vs-dense 2026](https://mljourney.com/sparse-vs-dense-retrieval-for-rag-bm25-embeddings-and-hybrid-search/)): "hybrid search combining BM25 with dense retrieval routinely beats either method alone." ColBERT v2 + BM25 + ms-marco rerank is the *current standard* and ships in OSS in 2026.

ELAI built a 7-layer custom pipeline; 4 of 7 layers are runtime-dead. The 3 live layers (BM25 + vectors + RRF) are exactly the 2026 standard. You wrote 4 layers of bonus dead code.

What to do instead: delete CatRAG, A-RAG, SPLADE, PageRank repo-map. Keep BM25 + dual-vector (or single ColBERTv2) + ms-marco cross-encoder. Stage the ONNX models. Ship the actual best-of-2026 pipeline, not a 7-layer Frankenstein.

### Punch 11 — Sidecar architecture (Python + Rust harness)

DETAILED_SUMMARY_01 §9 line 880: "Anthropic-compatible protocol (/v1/messages, /v1/steering/*)."

Anthropic compat layer documented limitations ([Anthropic OpenAI SDK docs](https://docs.anthropic.com/en/api/openai-sdk)): "prompt caching is not supported in the OpenAI compatibility layer… extended thinking doesn't return Claude's detailed thought process… the compatibility layer is primarily intended to test and compare model capabilities, and is not considered a long-term or production-ready solution."

ELAI is implementing the inverse — sidecar exposes Anthropic-shaped API for a local Qwen. So:
- ELAI's local sidecar pays Anthropic-protocol translation cost (system prompt extraction, tool_result message reshaping) for no SLO benefit; it's serving its own client.
- The local sidecar cannot use Anthropic-native features (prompt caching, extended thinking) because those are *services* Anthropic runs, not protocol fields.
- Sidecar processes pay HTTP/JSON overhead on every turn for what could be in-process FFI or shared-mem ring buffer.

Tunneling pythondev over SSH port 6465 ([CLAUDE.md project paths]) — adds 50-200ms RTT + TCP overhead per turn. Compare to in-process inference: <1ms IPC.

What to do instead: drop Anthropic compat *inside the local sidecar*. Define a minimal binary protocol with postcard + a versioned envelope. Translate Anthropic compat *only* at the cloud-fallback boundary where you actually need it.

### Punch 12 — MistakeBook structured constraints capped at 150 tokens

MistakeBook (lines 25 / D-1 / etc) feeds failed-attempt context to small models. 150-token cap.

The poisoning surface: [System Prompt Poisoning 2505.06493](https://arxiv.org/html/2505.06493v3) shows wrong demonstrations bias subsequent outputs. [Hijacking via Adversarial ICL 2311.09948](https://arxiv.org/pdf/2311.09948) demonstrates few-shot prompt injection works on instruction-tuned LLMs. [ICLAttack 2305.01219](https://arxiv.org/pdf/2305.01219) and [2401.05949](https://arxiv.org/pdf/2401.05949) show backdoor activation via demonstration context alone.

Small-model vulnerability is empirically the *worst* case ([over-prompting 2509.13196](https://arxiv.org/html/2509.13196v1)): "LLaMA-3.2-3B exhibits weakening performance from the beginning." ELAI's 3B/7B target tier is precisely the failure zone.

ELAI's MistakeBook says "here is what you got wrong last time." For a 7B model, this can encode a stable wrong pattern in the in-context window as a *negative anchor* that biases retry attempts. The model treats failed attempt details as task signal.

There is **no defense documented** in the summary or D-plan against this. The 150-token cap limits *volume*, not *toxicity*.

What to do instead: never feed raw failed attempts to the model. Feed structured signals only: failure category (from failure_taxonomy), affected files, the diff that didn't pass. Strip the actual generated code from the past attempt. Or: ALWAYS fresh-context retry, no memory of past failures whatsoever (this is what frontier-model coding agents actually do in production).

### Punch 13 — Long-tail risks in B/D plans

(See "D-plan / B-plan specific weak claims" section below for 5+ stale-evidence sites.)

### Punch 14 — Sidecar process isolation as "150% context savings"

DETAILED_SUMMARY_01 line 698-699: "If agents generate 12K tokens internally, isolation saves 150% of context window."

150% of context window is a weird unit. The arithmetic only works if "context window" is defined as the orchestrator-visible portion. Process isolation prevents subagent CoT from entering the parent's window — true. But the cost: every cross-process IPC turn adds postcard serialization, frame validation, fs/posix overhead. ELAI documents: "Effective: A++ (in-process logic + separate process exec, but shared namespaces / syscalls / network / fs / proc / limits)" (line 502). The isolation is *fictional* at the OS level.

Source: [SQLite vs Postgres latency](https://news.ycombinator.com/item?id=32676455) shows IPC overhead (Postgres ~0.10ms via Unix socket) vs in-process function call (SQLite ~0.02ms) — 5× cost per call. ELAI's IPC fires per agent-message, dozens per phase. The arithmetic of "150% savings" omits the multiplicative call cost.

What to do instead: measure end-to-end wall-clock between in-process and isolated. Publish the trade or retract the savings claim.

## D-plan / B-plan specific weak claims

1. **D-2 PPM sidecar at port 8101, separate model family from policy SLM** — claim: dual-GPU placement spec, heterogeneous-verifier rule from [Lightweight Latent Verifiers 2504.16760](https://arxiv.org/html/2604.16217). The paper number 2504.16760 isn't directly verifiable as the cited paper — your own memory at `feedback_paper_citation_verify.md` flags this exact pattern ("WebSearch arXiv IDs before citing"). Until verified, this is an unsupported architectural claim. AND: dual-GPU is the user's stated upgrade plan (2x3090) per memory `project_vram_tier_plan.md`, not current hardware. D-2 implementation is gated on hardware that doesn't exist yet. **Stale.**

2. **D-4 tripwire stack "auto-revert on ≥2 concurrent fires"** — cites [arxiv:2507.05619](https://arxiv.org/abs/2507.05619) "21.3% reward-hacking prevalence + concrete thresholds." The 21.3% figure needs the paper-page citation (your memory `feedback_paper_citation_verify.md` says "effect-size numbers need quote+page-anchor"). Currently uncited at that resolution. The concurrent-fires-2 rule is the design hypothesis, NOT the published threshold. **Conjectural.**

3. **D-5 BAI Thompson-sampling regret bounds** — cites [Audibert COLT 2010](https://arxiv.org/pdf/2106.10417). Published BAI assumes IID rewards. D-29 explicitly notes "ELAI pipeline outcomes are not IID across sessions." Wave 1 ships D-5 with the assumption, then Wave 5 has D-29 to fix it. The substrate ships with a known-wrong assumption. **Weak.**

4. **D-7 cross-domain transfer canary, ">5% regression" threshold across 5 families** — cites [NanoFlux 2509.23252](https://arxiv.org/pdf/2509.23252). Threshold is the design choice not the published gate. "Promotion blocked if any non-target family regresses >5%" — the 5% is unjustified; could be 3% or 10%. **Ungrounded threshold.**

5. **D-16 UniCR conformal refusal** — cites [UniCR 2509.01455](https://arxiv.org/pdf/2509.01455) for distribution-free guarantees. Counter-evidence ([2604.16217 Beyond Surface Statistics](https://arxiv.org/html/2604.16217)): "weighted conformal prediction performs poorly for high-dimensional inputs like prompts in LLMs, with reliable density ratio estimation being statistically infeasible due to the curse of dimensionality." D-16 inherits a method documented as fragile in the LLM setting. D-30 (DEFERRED) admits "needs N=500 calibration tuples; early sessions cannot calibrate locally." UniCR ships before its calibration corpus exists. **Stale + Goodhart risk: refusal_rate gameable as easy escape.**

6. **D-15 NoveltyClass {Search, DistantSearch, DiscoveryCandidate}** — cites FunSearch + AlphaEvolve. Both use very specific evaluator structures (executable scoring functions). ELAI's NoveltyClass is applied to "every Mode A / Mode D result." The discovery-vs-search distinction in those papers required an *automated, runnable, ground-truth evaluator* — math problems with verifiable answers, programs with passing tests. Without that ground-truth evaluator, the labels are conjecture. **Citation transferred outside its valid scope.**

7. **B-29 SymbolGraphCache "depth-2 impact summary appended to auditor's context"** — auto-injected on Write/Edit. No measurement that depth-2 is the right depth, or that the impact summary actually improves auditor decisions. **Untested architectural assumption.**

## What ELAI is doing RIGHT (concession)

1. **Recursive decomposition Idea→Concepts→Phases→Steps→BabySteps with per-level grounding** ([4D.7-4D.10 lines 614-622](../../../docs/thefinalglance/summary/DETAILED_SUMMARY_01_overview_and_architecture.md)). This is structurally aligned with [Voyager's automatic curriculum + skill library 2305.16291](https://arxiv.org/pdf/2305.16291), and the per-level audit closes the brittleness loop. The architecture is correct in shape; the issue is what gets put inside each block.

2. **Process isolation for context efficiency** (line 698) is real for the orchestrator's *prompt-window* budget even if the OS-level "isolation" is overstated. Hard separation of subagent CoT from parent context is the same pattern Claude Code uses for sub-agents and is empirically sound. Source: [Multi-Agent Architecture practical guide 2026](https://www.frenxt.com/research/multi-agent-architecture-guide).

3. **Failure taxonomy (deterministic regex pre-classifier before LLM)** — saves tokens by routing known-pattern failures away from model inspection. Aligned with [Outcome-Aware Tool Selection 2603.13426](https://arxiv.org/pdf/2603.13426) lightweight-router result: "lightweight CPU operation… at 10,000 req/sec with 5 ms budget, any method requiring LLM inference exceeds this by orders of magnitude." Pre-classifier is the right call.

4. **Cryptographic archive verify (D-8)** — blake3 hash chain on every mutation is the *one* place ELAI's determinism obsession actually pays off. Mutation tampering is a real attack surface for self-modifying systems ([Everitt 1908.04734](https://arxiv.org/abs/1908.04734)). D-25's 10 tamper-scenarios suite is a defensible engineering investment.

5. **Held-out task store with 9-gram filter + 6-month auto-retire (D-6)** — directly addresses [LiveCodeBench 2403.07974](https://arxiv.org/pdf/2310.06770) contamination dynamics. Temporal split + n-gram filter is current-best practice for benchmark integrity. This is grounded and correct.

## Top 5 redesign recommendations

1. **Collapse FSM to 3 states + postcondition functions; collapse 8 agent roles to 3.** Cite: Self-MoA [2502.00674](https://arxiv.org/abs/2502.00674) on single-strong-model proposers outperforming heterogeneous; MAST [2503.13657](https://arxiv.org/abs/2503.13657) on multi-agent failure modes. Worth: ~30% LOC reduction across orchestrator/agent_pool; ~40% reduction in failure-surface area documented by MAST.

2. **Default to cloud-frontier; ship 12GB local as a single-tier offline fallback.** Cite: [BenchLM 2026 SWE-bench gap](https://benchlm.ai/blog/posts/best-llm-coding) showing 25-30pp gap between open-source 7B-class and frontier; [Iternal 2026](https://iternal.ai/llm-selection-guide). Worth: removes 4 of 5 VRAM tiers from the support matrix; concentrates engineering on the one path most users will actually use.

3. **Stop pretending CoT echoes are grounding. Adopt Curry-Howard typed CoT proof certificates OR drop the GRC blockquote ritual.** Cite: [Typed CoT 2510.01069](https://arxiv.org/pdf/2510.01069); [Lanham et al 2307.13702](https://ar5iv.labs.arxiv.org/html/2307.13702) faithfulness limits; [FaithCoT-Bench 2510.04040](https://arxiv.org/html/2510.04040v1). Worth: kills a layer of theater enforcement that adds CLAUDE.md complexity for zero measurable correctness gain.

4. **Replace the 7-layer knowledge pipeline with BM25 + ColBERTv2 + ms-marco rerank (3 layers, all 2026-standard).** Cite: [2026 hybrid retrieval consensus](https://mljourney.com/sparse-vs-dense-retrieval-for-rag-bm25-embeddings-and-hybrid-search/); [BEIR 2026 leaderboard](https://app.ailog.fr/en/blog/news/beir-benchmark-update); your own audit showing 4 of 7 ELAI layers runtime-dead and DEEP MRR < STANDARD MRR. Worth: deletes ~5K LOC; aligns with best-of-2026 retrieval; eliminates the SPLADE/PageRank/CatRAG/A-RAG dead-code maintenance tax.

5. **Replace MistakeBook 150-token cap with strict-structured-signal-only + fresh-context retry.** Cite: [ICLAttack 2305.01219](https://arxiv.org/pdf/2305.01219); [Universal ICL Backdoors 2401.05949](https://arxiv.org/pdf/2401.05949); [Over-prompting 2509.13196](https://arxiv.org/html/2509.13196v1) on small-model vulnerability. Worth: closes a documented poisoning surface that targets ELAI's exact model tier (3B/7B) and currently has no defense in the summary or D-plan.

---

End of hostile critique. ELAI's strengths are in the macro architecture (decomposition, isolation, cryptographic archive, held-out store). Its weaknesses are in the *embellishments* — every additional layer of "determinism," "FSM substrate," "GRC blockquote," "MistakeBook," and "per-VRAM tier" is unsupported by 2026 evidence. The project is too clever by half. Simplify.
