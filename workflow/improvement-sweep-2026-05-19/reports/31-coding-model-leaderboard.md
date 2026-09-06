# Coding model leaderboard for ELAI (May 2026)

Scope: which dense, FA2-compat, NF4-quantizable model wins each VRAM tier ELAI ships to, and what cloud model the multi-runtime adapter should escalate to. Hard constraints: dense uniform-layer, head_dim 128, NF4 4-bit OK, no mixed head_dim (rules out Gemma family), no MoE-only (breaks DoLa/steering). Cloud cost matters; coding-bench number that matters most is SWE-bench Pro (contamination-resistant), not SWE-bench Verified.

## TL;DR
- 8GB: stay on Qwen2.5-Coder-7B-Instruct-abliterated NF4 (head_dim 128, 28Q/4KV GQA, dense, FA2 OK). Nothing dense in the head_dim-128 class has displaced it at this footprint in May 2026.
- 12GB (current ELAI default): same model, larger KV; the upgrade candidate is Qwen3.6-27B at Q3_K_M only if VRAM grows. Qwen3.6-27B is dense but uses hybrid Gated DeltaNet + Gated Attention with head_dim 256 on the attention blocks — that breaks the head_dim-128 constraint and is FA2-incompatible. KEEP 7B at this tier.
- 24GB: Qwen2.5-Coder-32B-Instruct (dense, head_dim 128, 64 layers, 40Q/8KV GQA) is still the best pure-dense head_dim-128 NF4 fit. Qwen3.6-27B beats it on SWE-bench Verified (77.2 vs older 32B numbers) but fails the head_dim constraint; only switch if ELAI drops the FA2 head_dim-128 invariant.
- 48GB: same 32B as a working node, plus a second instance OR keep one 32B and route the rest to cloud. Qwen3-Coder-Next (80B-A3B MoE) is the strongest open-weight but MoE breaks DoLa/steering and per CLAUDE.md is REJECTED for ELAI's local stack.
- Cloud escalation default: Sonnet 4.6 at $3/$15 per MTok delivers ~98% of Opus on SWE-bench Verified (79.6 vs 80.8) for ~1/5 the price; reserve Opus 4.7 ($5/$25) for SWE-bench Pro-style real-repo tasks where it actually wins (64.3 vs 53.4 Pro).

## Key sources
| URL | Type | Takeaway |
|---|---|---|
| https://www.marc0.dev/en/leaderboard | SWE-bench Verified leaderboard, May 2026 | GPT-5.5 88.7, Opus 4.7 87.6, GPT-5.3-Codex 85.0, Opus 4.6 80.8, DeepSeek V4 Pro Max 80.6, Gemini 3.1 Pro 80.6 |
| https://www.morphllm.com/swe-bench-pro | SWE-bench Pro 2026 ranking | Claude Mythos Preview 45.9 Pro vs 93.9 Verified — same model, contamination gap; Pro is what to trust |
| https://www.morphllm.com/claude-benchmarks | Claude Opus/Sonnet/Haiku benchmarks 2026 | Opus 4.7 87.6 Verified / 64.3 Pro / 70 CursorBench; Sonnet 4.6 79.6 Verified; Haiku 4.5 73.3 coding-avg |
| https://benchlm.ai/blog/posts/claude-api-pricing | Claude API pricing April 2026 | Haiku 4.5 $1/$5, Sonnet 4.6 $3/$15, Opus 4.7 $5/$25 per MTok |
| https://qwen.ai/blog?id=qwen3.6-27b | Qwen3.6-27B blog | Dense 27B, hybrid Gated DeltaNet + Gated Attention, attention head_dim 256, 64 layers, 262K ctx, Apache 2.0, 22 Apr 2026 |
| https://insiderllm.com/guides/best-local-coding-models-2026/ | Local-by-VRAM guide 2026 | 24GB tier: Qwen3.6-27B SWE-bench Verified 77.2 / LiveCodeBench v6 83.9 / Terminal-Bench 2.0 59.3 |
| https://qwen.ai/blog?id=qwen3-coder-next | Qwen3-Coder-Next blog | 80B total / 3B active MoE, 512 experts, hybrid block; SWE-bench Verified 70.6 (SWE-Agent) / 71.3 (OpenHands); SWE-bench Pro 44.3 |
| https://www.compute-market.com/blog/qwen-3-coder-next-local-hardware-guide-2026 | Qwen3-Coder-Next hardware | Q4_K_M ~48.2GB combined VRAM+RAM; MoE so 3B active per token but full weights needed |
| https://huggingface.co/huihui-ai/Qwen2.5-Coder-7B-Instruct-abliterated | Model card | BF16, dense; abliteration: IF_Eval -1.24, MMLU Pro +0.02, TruthfulQA -3.00, BBH +0.19 vs base |
| https://arxiv.org/pdf/2409.12186 | Qwen2.5-Coder technical report | 7B: 28 layers, 28 Q heads / 4 KV heads, head_dim 128. 32B: 64 layers, 40Q / 8KV, head_dim 128. Dense GQA throughout. |
| https://mistral.ai/news/devstral | Mistral Devstral 2 release | Dense 123B + dense Small 24B, 256K ctx, 72.2 SWE-bench Verified (123B), 68.0 (Small 24B); same arch as Ministral 3 |
| https://www.nxcode.io/resources/news/deepseek-v4-release-specs-benchmarks-2026 | DeepSeek V4 specs | V4-Pro 1.6T/49B active, V4-Flash 284B/13B active; SWE-bench Verified 80.6, LiveCodeBench 93.5, Codeforces 3206 |
| https://blog.google/products-and-platforms/products/gemini/gemini-3-flash/ | Gemini 3 Flash announcement | Flash 78 SWE-bench Verified; surpasses Pro on coding |
| https://www.nxcode.io/resources/news/gemini-3-1-pro-complete-guide-benchmarks-pricing-api-2026 | Gemini 3.1 Pro | 80.6 SWE-bench Verified |
| https://openai.com/index/introducing-gpt-5-5/ | GPT-5.5 launch | 88.7 SWE-bench Verified, 82.7 Terminal-Bench 2.0, 58.6 SWE-bench Pro |
| https://www.atlascloud.ai/blog/guides/kimi-k2-6-vs-glm-5-1-vs-qwen-3-6-plus-vs-minimax-m2-7-coding-2026 | Chinese open weights | Kimi K2.6 80.2 / 58.6 Pro; GLM-5 77.8; MiniMax M2.7 56.22 Pro at $0.30/MTok |
| https://openhands.dev/blog/introducing-openhands-lm-32b-a-strong-open-coding-agent-model | OpenHands LM 32B | Fine-tune of Qwen2.5-Coder-32B-Instruct via RL; 37.2 SWE-bench Verified (older eval style) |
| https://metr.org/blog/2026-02-24-uplift-update/ | METR productivity update | Early-2025 study: AI tools made devs 19% slower. 2026 follow-up estimates -18% (faster) for returning devs, -4% for new — wide CIs, mixed signal |
| https://thamizhelango.medium.com/the-scaffold-problem-why-ai-coding-benchmarks-are-broken-and-what-the-numbers-actually-mean-729186931c26 | Scaffold-vs-model critique | Opus 4.5: 80.9 Verified vs 45.9 Pro on standardized scaffold — half of the "model" score is scaffold tuning |
| https://nathan.sapwell.net/posts/hauhaucs-abliteration-analysis/ | Abliteration benchmark | huihui 100% ASR (refusal removal), TruthfulQA drops across all variants; abliterated != free |

20 sources, all dated 2026 or with 2026 leaderboard snapshots.

## Local model matrix (dense + FA2-compat + NF4-able)
| Model | Params | head_dim | Dense? | SWE-bench Verified | Aider Polyglot | BigCodeBench | LiveCodeBench v6 | NF4 OK? | Citation |
|---|---|---|---|---|---|---|---|---|---|
| Qwen2.5-Coder-7B-Instruct (base) | 7B | 128 (28Q/4KV) | yes | not reported (small) | — | best 7B open-source per tech report | — | yes, NF4+FA2 verified in ELAI baseline | arxiv 2409.12186 |
| huihui Qwen2.5-Coder-7B-abliterated | 7B | 128 (inherits) | yes | inherits | — | inherits | — | yes | huggingface model card |
| Qwen2.5-Coder-32B-Instruct | 32B | 128 (40Q/8KV, 64 layers) | yes | not reported here; Aider 73.7 | — | best open-source per tech report (EvalPlus/LCB/BCB) | reported "best open-source" | yes | arxiv 2409.12186 / llm-stats |
| Qwen3.6-27B | 27B | **256** (hybrid Gated DeltaNet hd=128 + Gated Attention hd=256) | yes (dense) but hybrid attention | 77.2 | n/a | n/a | 83.9 | NF4 possible per Unsloth GGUF; **FA2 breaks on hd=256 + hybrid** | qwen.ai blog, insiderllm |
| Qwen3-Coder-Next (80B-A3B) | 80B total / 3B active | hybrid | **MoE** | 70.6–71.3 | n/a | n/a | n/a | Q4 yes but MoE | qwen.ai blog |
| Devstral Small 2 | 24B | unspecified in coverage (same arch as Ministral 3) | yes (dense) | 68.0 | — | — | — | unverified for NF4+FA2; treat as VERIFY | mistral.ai/news/devstral |
| Devstral 2 | 123B | unspecified | yes (dense) | 72.2 | — | — | — | needs 128GB; not in scope | mistral.ai/news/devstral |
| OpenHands LM 32B | 32B (fine-tune of Qwen2.5-Coder-32B) | 128 (inherits) | yes | 37.2 (older scaffolding) | — | — | — | yes (inherits) | openhands.dev blog |

Key gotcha: every Qwen3 generation (3-Coder-Next, 3.6-27B, 3.6-A3B) moved to hybrid attention with mixed head_dim. The "dense + head_dim 128 + FA2" lane is held by the Qwen2.5-Coder family. There is no head_dim-128 dense successor to 2.5-Coder-32B in the May 2026 open-weight landscape.

## Cloud model matrix
| Model | SWE-bench Verified | SWE-bench Pro | Aider Polyglot | LiveCodeBench | Input $/MTok | Output $/MTok | Citation |
|---|---|---|---|---|---|---|---|
| Claude Haiku 4.5 | 73.3 coding-avg | — | — | — | $1 | $5 | benchlm Claude pricing |
| Claude Sonnet 4.6 | 79.6 | — | — | — | $3 | $15 | morphllm Claude benchmarks |
| Claude Opus 4.7 | 87.6 | 64.3 | (Opus 4.5 leads Polyglot at 89.4) | — | $5 | $25 | morphllm, marc0.dev |
| GPT-5.5 | 88.7 | 58.6 | — | — | not in source | not in source | openai.com/introducing-gpt-5-5 |
| GPT-5.3-Codex | 85.0 | 56.8 | — | — | not in source | not in source | marc0.dev / morphllm Pro |
| Gemini 3 Flash | 78.0 | — | — | 90.8 (preview) | not in source | not in source | blog.google gemini-3-flash |
| Gemini 3.1 Pro | 80.6 | — | — | 91.7 (preview) | not in source | not in source | nxcode gemini 3.1 |
| DeepSeek V4-Pro (Max) | 80.6 | — | — | 93.5 | not on this row | not on this row | nxcode deepseek-v4 |
| DeepSeek V4-Flash | matches Pro within ~2pp | — | — | 91.6 | $0.14 input (cited) | — | nxcode deepseek-v4 |
| Kimi K2.6 | 80.2 | 58.6 | — | 89.6 | — | — | atlascloud chinese frontier |
| MiniMax M2.7 | — | 56.22 | — | — | $0.30 input | — | atlascloud chinese frontier |
| GLM-5 | 77.8 | — | — | — | — | — | atlascloud chinese frontier |

For ELAI's multi-runtime adapter B-37: keep Sonnet 4.6 as the default cloud route (best $/quality), add a "hard task" route to Opus 4.7 conditional on SWE-Pro-style markers (real-repo PRs, multi-file refactors), and add DeepSeek V4-Flash via Anthropic-compat shim as the cheap-volume escape hatch ($0.14/MTok is ~21× cheaper than Sonnet input).

## Per-VRAM tier recommendation
| VRAM | Best model | Citation | NF4+FA2 verified | Why this one |
|---|---|---|---|---|
| 8GB | Qwen2.5-Coder-7B-Instruct-abliterated (NF4, FA2) | huggingface card + ELAI Qwen3B baseline note | yes (Qwen2.5 head_dim=128 GQA) | Only dense head_dim-128 7B coder that fits with KV at 20–25K ctx; abliteration cost is TruthfulQA -3pp, coding benches barely move |
| 12GB | same 7B at 25K ctx (current ELAI default) | ELAI Qwen3B baseline measured 1.92GB NF4 7B | yes | Qwen3.6-27B at Q3_K_M would fit but fails head_dim-128 + FA2 invariant; do not switch |
| 24GB | Qwen2.5-Coder-32B-Instruct NF4 | arxiv 2409.12186, llm-stats | yes (head_dim 128, 64 layers, 40Q/8KV) | Still the strongest dense head_dim-128 coder. ~20GB at NF4 leaves room for KV at 32K. If ELAI ever drops the head_dim constraint, Qwen3.6-27B SWE-bench-Verified 77.2 wins, but it isn't FA2-clean |
| 48GB | 1× Qwen2.5-Coder-32B + headroom for Opus-class cloud KV / second instance | tech report + per-VRAM tier plan | yes | Qwen3-Coder-Next is the open-weight champ at this footprint (70–71 Verified, 44 Pro) but MoE is RULED OUT for ELAI per CLAUDE.md. Multi-tenant 32B is the safer ship. |

## Coding-specialized fine-tunes worth trying
- OpenHands LM 32B — fine-tune of Qwen2.5-Coder-32B via RL on OpenHands trajectories, 37.2 SWE-bench Verified on older scaffold; same head_dim=128 lineage so NF4+FA2 should transfer. https://openhands.dev/blog/introducing-openhands-lm-32b-a-strong-open-coding-agent-model
- Devstral Small 2 (Mistral, 24B dense) — 68.0 SWE-bench Verified, 256K ctx, MIT-modified license. Architecture inherits Ministral 3; head_dim not confirmed in public material — VERIFY before adopting. https://mistral.ai/news/devstral
- huihui-ai/qwen3-coder-abliterated (Ollama listing) — abliterated Qwen3-Coder; useful only if ELAI later drops the head_dim-128 invariant. Same TruthfulQA caveat as the 2.5-Coder abliteration. https://ollama.com/huihui_ai/qwen3-coder-abliterated
- No "Hermes 3 Coder" exists as of May 2026 — Hermes 3 is the general instruct-tuned Llama 3.1 line, no dedicated coder variant. https://arxiv.org/pdf/2408.11857

## Anti-patterns
- SWE-bench Verified scores >80 are largely contamination + scaffold tuning. Opus 4.5: 80.9 Verified, 45.9 Pro on standardized scaffold. OpenAI's audit confirmed verbatim-patch reproduction of Verified tasks by GPT-5.2, Opus 4.5, Gemini 3 Flash. Pick models by SWE-bench Pro or by your own private bench. https://www.morphllm.com/swe-bench-pro https://thamizhelango.medium.com/the-scaffold-problem-why-ai-coding-benchmarks-are-broken-and-what-the-numbers-actually-mean-729186931c26
- "Claude Mythos Preview leads at 93.9 Verified" — same model is 45.9 on Pro. Treat preview / unreleased model leaderboard wins as marketing, not signal. https://www.morphllm.com/swe-bench-pro
- Qwen3.6-27B "beats 397B on coding" headlines refer to MoE-A17B, not dense 397B. The dense apples-to-apples win is over Qwen2.5-Coder-32B and is real (77.2 vs not-reported-but-lower-on-Verified), but it costs you head_dim-128 + FA2. https://qwen.ai/blog?id=qwen3.6-27b https://www.buildfastwithai.com/blogs/qwen3-6-27b-review-2026
- Qwen3-Coder-Next "3B active beats 49B active" is a per-active-param efficiency claim, not absolute. Total Pro is 44.3 vs DeepSeek-V4-Pro presumed higher. MoE breaks DoLa/steering and is REJECTED for ELAI's pipeline regardless. https://qwen.ai/blog?id=qwen3-coder-next
- Abliterated models trade ~3pp TruthfulQA across the board and 100% refusal removal. For ELAI's "code-printer" use the trade is fine; do not deploy the same abliterated model as a fact-grounding model. https://nathan.sapwell.net/posts/hauhaucs-abliteration-analysis/
- METR 2025 found AI tools made experienced OSS devs 19% slower; the 2026 follow-up swung to -18% but with CI -38 to +9. Self-reported speedups are useless; ELAI's bench must measure wall-clock task completion, not user satisfaction. https://metr.org/blog/2026-02-24-uplift-update/

## Open questions
- Does Qwen3.6-27B actually break FA2 in practice, or is the hybrid Gated-Attention block FA2-routable while DeltaNet falls back to SDPA? Need to read the reference impl and run a 1-prompt smoke test on a 24GB box before ruling it out for the 24GB tier.
- Is there a dense head_dim-128 32B+ coder shipping in Q3 2026 that ELAI could adopt without breaking FA2? Watch for a Qwen2.5-Coder-Refresh, a Devstral-Dense-32B, or a Yi-Coder-2 announcement.
- What is the actual measured SWE-bench Pro number for Qwen2.5-Coder-32B-Instruct + a standard agent scaffold? The tech report predates SWE-bench Pro; need a private eval.
- Cost/quality crossover: at what input volume does DeepSeek V4-Flash (~$0.14/MTok) beat Sonnet 4.6 for ELAI's mixed workload after retry-rate factored in? Measurable with the multi-runtime adapter logs.
- Does the huihui abliteration of Qwen2.5-Coder-7B-Instruct measurably regress HumanEval / MBPP / Aider compared to the base, or is the regression purely on TruthfulQA-class evals? ELAI should rerun the in-tree code bench on both to confirm the abliteration cost is purely off-task.
