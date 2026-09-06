# Structured output for ELAI (May 2026)

## TL;DR
- Adopt XGrammar as the canonical Tier-2 backend in the sidecar (default for Qwen2.5-Coder); it is the default in vLLM, SGLang, TensorRT-LLM, MLC-LLM and runs at ~40us/token with cached schemas (SqueezeBits, Catalyst-CMU). Cache one compiled grammar per BabyStepSpec block + one per AgentRole and reuse.
- INSTRUCTOR mode (free CoT then schema) is the right default for any role that has to reason. CRANE (arxiv:2502.09061) shows pure-constraint mode drops reasoning accuracy; DCCD shows +24pp on GSM8K vs naive constrained. Pattern: unconstrained `<think>...</think>` then XGrammar-masked `<answer>{...}</answer>` (DeepSeek-R1 style, "Think Inside the JSON" arxiv:2502.14905).
- For editor agents adopt CodeAct (arxiv:2402.01030 ICML 2024) +20% success / -30% turns vs JSON tool-args, and SEARCH/REPLACE blocks for file edits (Aider polyglot benchmark, top model GPT-5 88.0% Nov 2025 leaderboard) over unified diff or full-rewrite.
- For Qwen2.5-Coder specifically use the `<tools>` tag parser (hanXen vllm fork) - hermes `<tool_call>` is ignored by the Coder variant. Qwen2.5-7B-Instruct uses native hermes.
- Never use vendor JSON mode (`type: json_object`) - it only checks syntax, not schema. Use strict JSON Schema (OpenAI Aug-2024, Anthropic `structured-outputs-2025-11-13`, Gemini `responseJsonSchema` Nov-2025) when offloading to cloud; all three now compile to grammar internally.

## Key sources
| URL / arxiv | what | takeaway |
|---|---|---|
| arxiv:2411.15100 (XGrammar) | CFG engine, default in vLLM/SGLang/TRT-LLM/MLC | 100x speedup, near-zero overhead with cached grammar |
| github.com/mlc-ai/xgrammar | Source repo, EBNF + JSON schema + regex | Supports Qwen, Llama, DeepSeek, Phi, Gemma |
| arxiv:2502.09061 (CRANE) | Constrained decoding hurts reasoning | Alternate unconstrained reasoning + constrained answer |
| arxiv:2402.01030 (CodeAct ICML 2024) | Python code actions over JSON tool args | +20% success, ~30% fewer turns on API-Bank |
| arxiv:2501.10868 (JSONSchemaBench) | 10K real schemas, 6 engines | XGrammar wins on cached, Guidance wins on TTFT |
| aider.chat/docs/leaderboards/edit.html | Polyglot edit-format leaderboard | GPT-5 high 88.0% Nov-2025; whole>diff for small models, diff>whole for frontier |
| arxiv:2502.14905 ("Think Inside the JSON") | `<think>`/`<answer>` RL+SFT for schema | GRPO + format-reward, R1-style two-tag protocol |
| github.com/guidance-ai/llguidance | On-the-fly mask, ~50us/128k vocab | No startup cost, better TTFT than XGrammar |
| github.com/noamgat/lm-format-enforcer | Char-level prefix-tree filter | Lower hallucination zero-turn (Qwen2.5-72B 0.49% FPR) but lags on multi-turn |
| developers.redhat.com/articles/2025/06/03/structured-outputs-vllm | vLLM 0.8+ guided decoding | XGrammar default, fallback guidance, outlines deprecated path |
| platform.claude.com/docs/en/build-with-claude/structured-outputs | Anthropic strict JSON schema beta | Compiles schema to grammar, masks tokens; 24h schema cache |
| openai.com/index/introducing-structured-outputs-in-the-api | OpenAI strict mode | 100% schema adherence via constrained sampling (was 93% prompt-only) |
| ai.google.dev/gemini-api/docs/structured-output | Gemini responseJsonSchema | Full JSON Schema since Nov-2025, additionalProperties supported |
| blog.squeezebits.com/guided-decoding-performance-vllm-sglang | XGrammar vs Guidance heads-up | XGrammar faster on cached grammar, Guidance better TTFT on unique schemas |
| arxiv:2603.03305 (DCCD) | Draft-conditioned constrained decoding | +24pp GSM8K over naive constrained on 1B model |
| chatforest.com/reviews/sglang-structured-generation-llm-serving | SGLang+XGrammar 2026 | 4200 tok/s 99.8% valid; XGrammar-2 paired with SGLang v0.5.11 May-2026 |
| github.com/hanXen/vllm-qwen2.5-coder-tool-parser | Qwen2.5-Coder `<tools>` parser | Coder variant ignores hermes; needs dedicated parser |

## Engine landscape
| engine | grammar/EBNF | regex | JSON schema | CFG | speed (cached) | Qwen2.5-Coder | citation |
|---|---|---|---|---|---|---|---|
| XGrammar | yes (EBNF) | yes | yes (compiled to EBNF) | yes | ~40us/tok mask, near-zero overhead | yes (default in vLLM) | arxiv:2411.15100, vLLM docs |
| llguidance (Guidance) | yes | yes | yes | yes | ~50us/128k vocab, no startup | yes via vLLM | github.com/guidance-ai/llguidance |
| Outlines / outlines-core | regex-FSM | yes (native) | via regex | partial | 98% overhead cut in v0.1, ~2x slower than XGrammar on cached | yes | dottxt-ai/outlines |
| lm-format-enforcer | char-level | yes | yes | limited | slower (per-step prefix tree) | yes | github.com/noamgat |
| llama.cpp GBNF | yes (GBNF) | limited | via GBNF | yes | inefficient `x? x? x?`, otherwise OK | yes (GGUF) | llama.cpp grammars/README |
| JSONFormer | no | no | yes (key-by-key) | no | slow (forced-decode each token of keys) | yes | (older, surpassed by XGrammar per JSONSchemaBench) |
| Microsoft Guidance (legacy) | yes | yes | yes | yes | superseded by llguidance | yes | guidance-ai org repo |
| OpenAI strict mode | yes (server-side CFG) | n/a | yes | yes | server-side, opaque | n/a (cloud) | openai.com/index/introducing-structured-outputs |
| Anthropic structured-outputs-2025-11-13 | yes (compiled grammar) | n/a | yes | yes | 24h schema cache | n/a (cloud) | platform.claude.com |
| Gemini responseJsonSchema | yes | n/a | yes (Nov-2025) | n/a | server-side | n/a (cloud) | ai.google.dev |

## Edit-format winner (with ablations)
Aider polyglot benchmark Nov-20-2025 leaderboard, 225 exercises:
- GPT-5 high: 88.0% / $29.08 (diff)
- o3-pro high: 84.9% / $146.32 (diff)
- Frontier models: `diff` (SEARCH/REPLACE) > `whole` on success and on cost (whole burns full-file tokens).
- Small/local models: `whole` > `diff` when the model can't reliably output a well-formed search block; aider docs call this out explicitly. For Qwen2.5-Coder-7B in INSTRUCTOR mode use `diff-fenced` or `whole` depending on file size; 7B is on the edge.
- Unified-diff: aimodels.fyi 2024 ablation - "Unified diffs make GPT-4 Turbo 3x less lazy"; helps frontier models stop truncating with "// ... rest unchanged". Diff-XYZ (arxiv:2510.12487) benchmark confirms format-choice is model-and-task dependent.
- CodeAct (arxiv:2402.01030, ICML 2024): +20% success, ~30% fewer turns on API-Bank vs JSON tool args; uses Python code as the action format. Wins specifically for multi-tool composition, branching, loops. Best for runner / executor agents that compose tools, not for surgical file edits.

Decision: SEARCH/REPLACE for file edits (Aider style), CodeAct for executor/runner that composes tool calls, JSON-Schema via XGrammar for everything else (BabyStepSpec, PipelineSpec, Auditor findings, FailureTaxonomy).

## When constraints hurt
- CRANE (arxiv:2502.09061): strict-grammar enforcement masks tokens that mid-reasoning would otherwise reach; reasoning accuracy drops because the LLM gets pushed off its high-probability path. Fix is grammar augmentation that allows free reasoning tokens then snaps into strict syntax for the final answer.
- "When Correct Isn't Usable" (arxiv:2605.02363): small models hit 77-85% naive GSM8K but collapse under structured-output contracts. The reasoning is there - the structured channel kills it.
- DCCD (arxiv:2603.03305): +24pp on GSM8K vs naive constrained (15.2% -> 39.0% on 1B). Mechanism: draft-conditioned constrained decoding lets the unconstrained draft do the math, then constrained decoding picks the closest valid emission.
- Latency: constrained decoding has 3.6x-8.2x overhead in some settings (arxiv:2605.02363 measured); XGrammar/llguidance largely solve this but the quality-on-reasoning hit is orthogonal to speed.
- "Think Inside the JSON" (arxiv:2502.14905): RL+SFT with `<think>`/`<answer>` tag-format reward stabilises strict schema adherence on small models.

Rule: any agent role whose job involves reasoning (auditor, planner, decomposer) must run INSTRUCTOR mode (unconstrained CoT then constrained answer). GRAMMAR-only mode acceptable for high-stakes-low-reasoning blocks (FailureTaxonomy classification, ReasoningTemplate header, deterministic schema fillers).

## Best techniques to adopt for ELAI
Ranked by impact:

| # | technique | grounding | integration hook |
|---|---|---|---|
| 1 | XGrammar as canonical Tier-2 backend in sidecar | arxiv:2411.15100 + vLLM/SGLang defaults | python/elai_inference/server.py: add `xgrammar.Grammar` per compiled BabyStepSpec block; wire as `LogitsProcessor` in `model.generate()`; cache one Grammar per (role, block) pair |
| 2 | INSTRUCTOR mode (CoT then constrained answer) for reasoning roles | arxiv:2502.09061 CRANE + arxiv:2502.14905 + DeepSeek-R1 pattern | Provider trait `Provider::generate_structured`: emit `<think>...</think>` unconstrained, then `<answer>` with XGrammar mask; post-parse and validate. Default mode for auditor/planner/decomposer/idea-framer |
| 3 | GRAMMAR mode for low-reasoning enum/header blocks | XGrammar near-zero overhead | Use directly for FailureTaxonomy enum, BabyStepSpec status enum, ReasoningTemplate type-tag |
| 4 | SEARCH/REPLACE edit format for fixer / executor file edits | aider.chat leaderboard Nov-2025 | Replace JSON-tool file-write args with SEARCH/REPLACE blocks; parse server-side; better for Qwen2.5-Coder-7B than unified diff |
| 5 | CodeAct (Python tool-composition) for runner/orchestrator | arxiv:2402.01030 ICML 2024 +20% | New AgentRole `runner`: Python interpreter sandbox; tools exposed as Python functions; LLM emits code block instead of JSON tool-call. Replace JSON action format in B-N executor. |
| 6 | Qwen2.5-Coder `<tools>` parser, not hermes | hanXen/vllm-qwen2.5-coder-tool-parser | sidecar tool-call parser: detect model variant, switch hermes vs `<tools>` |
| 7 | Per-role grammar cache with content-hash key | XGrammar cache wins per JSONSchemaBench | sidecar: keyed cache `(role, blockname, schema_hash) -> compiled grammar`; warm at boot from all 8 BabyStepSpec blocks * 8 AgentRoles |
| 8 | Cloud fallback uses vendor strict mode (Anthropic structured-outputs-2025-11-13 / OpenAI strict / Gemini responseJsonSchema) | three vendors compile to grammar internally | CloudGroundingClient: when remote provider supports strict, pass schema; never use legacy `json_object` |
| 9 | DCCD-style draft-conditioned fallback for math/numeric blocks | arxiv:2603.03305 +24pp GSM8K | optional retry path: unconstrained draft -> select closest valid emission; reserve for CSQ counterfactuals and numeric Auditor evidence |
| 10 | Reject lm-format-enforcer for ELAI default | benchmarks: simpler but slower, weaker multi-turn | Keep as fallback only for tokenizer edge-cases XGrammar can't handle |

## Anti-patterns
- Using legacy `json_object` mode (OpenAI / Anthropic legacy) - syntax-only, no schema enforcement (developers.openai.com structured-outputs guide, openai.com/index/introducing-structured-outputs).
- Pure GRAMMAR mode for reasoning agents - CRANE arxiv:2502.09061 demonstrates the accuracy drop; arxiv:2605.02363 measures 10-30% drop on reasoning tasks under hard structural constraints.
- Forcing JSON tool-args where Python composition is natural - CodeAct (arxiv:2402.01030) measured +20% loss vs Python actions on multi-tool tasks.
- Using `whole`-file edit format for large files on Qwen2.5-Coder-7B - aider docs note token cost / context blow-up. Use SEARCH/REPLACE for >200 lines.
- Compiling new XGrammar grammars per request - SqueezeBits benchmark and JSONSchemaBench paper both confirm cache reuse is where XGrammar wins. Uncached XGrammar compilation can hit seconds to minutes for complex schemas.
- Hermes `<tool_call>` for Qwen2.5-Coder - the Coder variant ignores it; use the `<tools>` parser (github.com/hanXen/vllm-qwen2.5-coder-tool-parser).
- Trusting PCRE-style negative lookahead in constraint regex - does not compile to FSM (Outlines and XGrammar both reject; this is the original ELAI note).

## Open questions
1. Measure ELAI's current 7B INSTRUCTOR-mode JSON failure rate (free generation + post-hoc validate + retry) vs proposed XGrammar GRAMMAR mode + R1-style two-tag protocol. Hypothesis: GRAMMAR alone drops auditor accuracy, two-tag fixes it. Target: GSM-style synthetic eval over 100 BabyStepSpec generations.
2. Quantify XGrammar compilation cost on the actual 8-block * 8-role grammar matrix (64 grammars). If >1s wall-clock total at boot, cache to disk keyed by schema_hash.
3. For Qwen2.5-Coder-7B specifically: does CodeAct (Python action) actually beat JSON tool-args for the ELAI runner role at 7B scale? Original paper used 7B+ open-source plus GPT-3.5/4; need a local 100-task eval. Hypothesis: yes for multi-tool, marginal for single-tool.
4. Does SEARCH/REPLACE fail more than `whole` on Qwen2.5-Coder-7B for files <100 lines? Aider leaderboard is dominated by frontier models; the 7B cliff might cut the other way. Measure on the 8 BabyStepSpec block authoring tasks.
5. Is XGrammar-2 (May-2026, paired with SGLang v0.5.11) worth a sidecar upgrade vs stable XGrammar-1? Specifically the 6x tool-calling compile-speed claim - matters only if we cannot cache.
