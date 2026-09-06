# LLM cost + token economics for ELAI (May 2026)

## TL;DR
- Anthropic Sonnet 4.6 is the price-floor flagship for ELAI cloud-mode at $3/$15 per MTok; cache reads at $0.30/MTok (10% of base) make prompt-cache discipline the single biggest spend lever. Stack Batch API (50% off) + 5-min cache (90% off reads) for up to ~95% combined savings on non-interactive workloads (Anthropic pricing doc).
- Agentic coding tasks burn 1-3.5M tokens per SWE-bench task with input outnumbering output 20-25x and 30x variance between runs on the same task (arxiv 2604.22750, Vantage). ELAI's `estimate_task_cost_usd` MUST model input-heavy with a wide variance band, not a point estimate.
- Anthropic cut prompt-cache TTL from 60min to 5min in early 2026, increasing effective API costs 30-60% for many production workloads (DEV community report). ELAI's HYBRID retry+escalate loop must re-warm cache within 5min or pay the cache-write multiplier twice.
- Local Qwen 7B on user's existing GPU is effectively $0 marginal cost vs ~$0.50-2.00 per coded task on Sonnet 4.6; cloud break-even for a daily heavy user (>1M tokens/day) sits at 6-12 month payback on a single RTX 4090/5090 per Introl 2026 hardware guide.
- "Denial-of-Wallet" is a real failure mode (LangChain 4-agent loop = $47K in 11 days, retry storm = 2.3M calls in a weekend). Observability alone does not stop spend - ELAI's `max_spend_per_task` / `max_spend_per_day` / `warn_at_percent` SSB stack must be a hard cutoff, not an alert (Ravoid / handsonarchitects).

## Key sources
| URL | what | takeaway |
|---|---|---|
| https://platform.claude.com/docs/en/about-claude/pricing | official Anthropic pricing | Opus 4.7 $5/$25, Sonnet 4.6 $3/$15, Haiku 4.5 $1/$5 per MTok; cache write 1.25x (5m) or 2x (1h); cache read 0.1x; Batch API 50% off; Fast mode 6x premium |
| https://www.finout.io/blog/anthropic-api-pricing | Anthropic pricing 2026 | Batch + caching stack to ~95% combined savings; 5-min cache pays off after 1 read, 1-hour after 2 |
| https://dev.to/whoffagents/claude-prompt-caching-in-2026-the-5-minute-ttl-change-thats-costing-you-money-4363 | Anthropic TTL cut | quietly changed cache TTL from 60min to 5min in early 2026, increasing many production bills 30-60% |
| https://developers.openai.com/api/docs/pricing | OpenAI pricing | GPT-5.5 $5/$30; GPT-5.4 $2.50/$15; GPT-5.4 mini $0.75/$4.50; cached reads 0.25-0.50x; auto-caching free |
| https://ai.google.dev/gemini-api/docs/pricing | Gemini pricing | Gemini 3 Flash $0.50/$3.00 per MTok; 3.1 Pro premium tier; large free quota for low-volume devs |
| https://api-docs.deepseek.com/quick_start/pricing | DeepSeek V4 pricing | V4 Flash $0.30/$0.50; V4 Pro $0.435/$0.870 (75% promo through May 31 2026); 1M context; 90% cache discount |
| https://arxiv.org/abs/2305.05176 | FrugalGPT (Chen et al.) | LLM cascade matches GPT-4 with 98% cost reduction; cheap-first chain w/ confidence gate |
| https://automix-llm.github.io/automix/ | AutoMix ICLR 2025 | self-verify cascade w/ POMDP router, >50% cost reduction at iso-quality |
| https://arxiv.org/abs/2604.22750 | "How Do AI Agents Spend Your Money" | agentic tasks 1000x more tokens than chat; 20-25x input/output ratio; 30x variance same-task |
| https://www.vantage.sh/blog/agentic-coding-costs | hidden cost drivers | agentic coding 1-3.5M tokens/task; input drives bill, not output |
| https://artificialanalysis.ai/agents/coding-agents | SWE-bench agent index | tracks cost-per-task on Verified; full eval runs $300+ |
| https://venturebeat.com/programming-development/devin-2-0-is-here-cognition-slashes-price-of-ai-software-engineer-to-20-per-month-from-500 | Devin pricing collapse | $500/mo > $20/mo Core, $2.25/ACU (1 ACU = ~15 min); moderate task = 5-20 ACU = $11-45 |
| https://www.morphllm.com/comparisons/morph-vs-aider-diff | Aider vs Claude Code token bench | Aider uses 4.2x fewer tokens than Claude Code; $60/mo heavy vs $100-200 Claude Max |
| https://leanopstech.com/blog/agentic-ai-cost-runaway-token-budget-2026/ | agentic cost runaway | agents burn 50x more tokens than chats; budget enforcement required at agent layer |
| https://ravoid.com/blog/ai-agent-budget-enforcement | $47K LangChain incident | 4-agent infinite loop; observability doesn't stop spend, only hard cutoff does |
| https://handsonarchitects.com/blog/2025/denial-of-wallet-cost-aware-rate-limiting-part-1/ | Denial-of-Wallet | RPS-based rate limits don't protect budgets; need cost-aware rate limiting per-request |
| https://tokenmix.ai/blog/groq-free-tier-limits-2026 | Groq free tier | 30 RPM / 6K TPM / 14.4K req/day default; Llama 4 Scout, Qwen3 32B, Kimi K2 available free |
| https://www.getaiperks.com/en/ai/cerebras-free-tier-guide | Cerebras free tier | 1M tokens/day free since April 2026 - most generous of any provider |
| https://introl.com/blog/local-llm-hardware-pricing-guide-2025 | local hardware ROI | RTX 4090/5090 ROI 6-12 months at 1-10M tokens/day; under 1M tokens/day = stick with API |
| https://ifeeltech.com/blog/local-ai-server-small-business-guide | Mac Studio TCO | M3 Ultra <300W peak, <$100/yr electricity; 2x RTX 5090 = 1600W, $500-700/yr + cooling |
| https://docs.vllm.ai/en/stable/design/prefix_caching/ | vLLM prefix caching | hash-based LRU eviction; <1% throughput penalty at 0% hit rate, several-x speedup at high hit rate |
| https://www.helicone.ai/blog/the-complete-guide-to-LLM-observability-platforms | observability comparison | Helicone $0/10K req gateway; Portkey 1T tokens/day March 2026; LangSmith $39/seat Plus + trace overage |

## Pricing table May 2026
Format: Provider / Model | input $/MTok | output $/MTok | cache read | batch discount | citation

| Provider/Model | Input | Output | Cache read | Batch | Citation |
|---|---|---|---|---|---|
| Anthropic Opus 4.7 | $5.00 | $25.00 | $0.50 (0.1x) | 50% | platform.claude.com/pricing |
| Anthropic Sonnet 4.6 | $3.00 | $15.00 | $0.30 (0.1x) | 50% | platform.claude.com/pricing |
| Anthropic Haiku 4.5 | $1.00 | $5.00 | $0.10 (0.1x) | 50% | platform.claude.com/pricing |
| OpenAI GPT-5.5 | $5.00 | $30.00 | ~$1.25 (0.25x) | not-public | developers.openai.com/api/docs/pricing |
| OpenAI GPT-5.4 | $2.50 | $15.00 | ~$0.625 | not-public | developers.openai.com/api/docs/pricing |
| OpenAI GPT-5.4 mini | $0.75 | $4.50 | ~$0.19 | not-public | pricepertoken.com |
| Google Gemini 3 Flash | $0.50 | $3.00 | implicit caching free | implicit batch | ai.google.dev/gemini-api/docs/pricing |
| Google Gemini 3.1 Pro | premium tier | premium tier | implicit | implicit | aipricing.guru |
| DeepSeek V4 Flash | $0.30 | $0.50 | $0.03 (90% off) | n/a | api-docs.deepseek.com |
| DeepSeek V4 Pro (post-promo) | $1.74 | $3.48 | $0.17 (90% off) | n/a | api-docs.deepseek.com |
| Groq Llama 3.3 70B | $0.59 | ~$0.79 | n/a | n/a | cloudzero.com/blog/groq-pricing |
| Groq Llama 3.1 8B | $0.05 | $0.08 | n/a | n/a | tokenmix.ai/blog/groq-api-pricing |
| Local Qwen2.5-Coder 7B on user GPU | ~$0 marginal | ~$0 marginal | n/a | n/a | local power draw ~300W |

Anthropic cache write multipliers: 1.25x for 5-min TTL, 2x for 1-hour TTL. Fast mode (Opus only): 6x base rates ($30/$150). Tool-use system prompt: ~346 tokens fixed overhead per request when tools present.

## Per-task cost benchmarks (cited)

Numbers below are real measurements, not vendor marketing.

| Agent | Tokens/task | Cost/task | Citation |
|---|---|---|---|
| Devin Core | 1 ACU = 15min Devin work = $2.25; moderate refactor = 5-20 ACU | $11-$45 | VentureBeat / costbench.com |
| Devin SWE-bench Verified solve rate | n/a | 13.86% pass rate (independent test) | openaitoolshub.org Devin review |
| Aider heavy daily user | ~126K tokens/task average | ~$60/mo total API spend | morphllm.com vs-aider-diff |
| Claude Code (Claude Max heavy) | ~4.2x Aider tokens | $100-200/mo | morphllm.com vs-aider-diff |
| All-Opus single session | n/a | $15-30 | morphllm best-ai-coding-agents-2026 |
| Mixed Haiku/Sonnet/Opus session | n/a | $3-7 | morphllm best-ai-coding-agents-2026 |
| SWE-bench Verified full agent eval | varies | $300+ per agent eval run | codesota.com leaderboard |
| Process Reward Model extra spend | n/a | ~$0.20 per additional resolved instance | arxiv 2509.02360 |
| Agentic coding general | 1M-3.5M tokens/task incl retries | varies 30x | arxiv 2604.22750 / vantage.sh |
| Cursor Ultra heavy | n/a | $200/mo fixed; usage credits $20-$4000/mo | nocode.mba/articles/cursor-pricing |
| Replit Agent effort-based | varies | $0.06-$1+ per task; pool $4K/mo cap | sidetool.co/replit-cursor-pricing |

ELAI implication: with Qwen 7B local at $0 + Sonnet 4.6 cloud escalation, a typical baby step (60K-200K input tokens + 5K-15K output) at 80% local resolution costs ~$0.30-$1.20 per escalated step. Variance band: 30x same-task means budget must be range-based, not point.

## Prompt caching effectiveness

Anthropic 5-min cache (the default ELAI HYBRID loop should use):
- Cache write: 1.25x base input. Cache read: 0.10x base input (90% off). Breaks even after 1 read.
- Real production result: legal-tech contract analysis 62% API cost reduction via caching (ngrok blog / promptbuilder.cc).
- RCA pipeline case study: 90% cost reduction at Stella Lin's team (dev.to/stella_lin_82914c71e25769).
- TTL caveat (CRITICAL May 2026): Anthropic cut TTL from 60min > 5min in early 2026, raising effective bills 30-60% for workflows where requests are spaced >5min apart (dev.to/whoffagents).

OpenAI auto-caching:
- Enabled by default on prompts >1024 tokens; no setup, no separate API.
- Cache reads 0.25-0.50x base input depending on model (50-75% off).
- Up to 80% TTFT reduction (developers.openai.com/api/docs/guides/prompt-caching).
- Real semantic-similarity rate in production: ~31% queries (introl.com/blog/prompt-caching-infrastructure).

DeepSeek V4: 90% off cached input ($0.03/MTok), automatic on shared-prefix prompts.

vLLM (relevant for self-hosted Qwen):
- Hash-based LRU prefix cache, <1% throughput penalty even at 0% hit rate.
- High hit rate gives "several-x" speedup (docs.vllm.ai/en/stable/design/prefix_caching).
- T-LRU RFC: 27.4% P95 TTFT reduction over standard LRU (github.com/vllm-project/vllm/issues/37823).

## Cascade economics

FrugalGPT cascade pattern (arxiv 2305.05176, TMLR 2024 version):
- Cheap LLM first > if confidence below threshold > next-cheapest > ... > frontier.
- Result: match GPT-4 quality at 98% cost reduction, or +4% accuracy at iso-cost.
- Three sub-techniques: prompt adaptation (shorten), LLM approximation (distillation), LLM cascade (this).

AutoMix (ICLR 2025, arxiv 2310.12963):
- Adds self-verify step between cascade tiers; uses POMDP router to convert binary self-eval into probabilistic confidence.
- >50% cost reduction at iso-quality across multiple LMs and datasets.
- Works with closed-source APIs, no weights access required.

C3PO (arxiv 2511.07396):
- Probabilistic cost-constraint optimization for cascades on reasoning tasks.
- Direct integration target for ELAI's HYBRID gate.

ELAI fit: existing L3+ failure escalation is a 2-tier cascade. Add a verifier prompt before escalation (a la AutoMix) to gate the retry-cap=1 jump - measured >50% cost reduction is realistic for ELAI's L0-L2 work.

## Local-vs-cloud break-even

Profile: ELAI heavy daily user (already has GPU box).

Per Introl 2026 local hardware pricing guide:
- Under 1M tokens/day total: stick with API, no payback on hardware.
- 1-10M tokens/day: single RTX 4090 ($1.6-2K) or RTX 5090 ($2-3.8K) achieves ROI in 6-12 months running quantized 7B-13B.
- 10M+ tokens/day: multi-GPU or workstation pays back in 3-6 months.

Power costs:
- RTX 5090 ~575W, 450W steady inference, ~$500-750/yr electricity at $0.10-0.30/kWh.
- RTX 4090 ~450W steady, similar bracket.
- Mac Studio M3 Ultra <300W peak, <$100/yr electricity (ifeeltech.com).
- Dual-RTX-5090 workstation up to 1600W under full load, $500-700/yr + HVAC = $800-1200/yr (ifeeltech.com).

For ELAI's specific case (Qwen 7B NF4 4-bit on existing 12GB GPU = ~1.92 GB VRAM, no incremental hardware spend):
- Marginal cost per task is essentially electricity-only.
- Even at 100K tokens/task and $0.30/kWh, per-task power cost is fractions of a cent.
- Break-even vs Sonnet 4.6 ($3/MTok input + $15/MTok output) is immediate - any non-trivial task that completes locally beats the cloud bill.
- Real comparison is OPPORTUNITY COST: local 7B accuracy on hard tasks vs Sonnet 4.6 accuracy; ELAI's existing HYBRID escalation correctly bridges this.

## Best cost-management techniques to adopt
Ranked by impact-to-integration-effort ratio.

1. **5-min prompt caching on every cloud call** | grounding: 90% read discount, breaks even after 1 read (Anthropic pricing doc); legal-tech 62% real savings (ngrok). | integration: route every `CloudGroundingClient` request through Anthropic SDK with `cache_control` blocks on system prompt + project spec + repo manifest. Cache write 1.25x; cache read 0.10x.

2. **Hard cost caps at agent layer, not provider layer** | grounding: $47K LangChain loop incident; observability does not stop spend (Ravoid). | integration: ELAI's existing `max_spend_per_task` / `max_spend_per_day` MUST be a hard cutoff that aborts task at threshold, not just warn at `warn_at_percent`. Add per-agent-role cap (executor vs auditor have different ceilings). Add hourly velocity cap to catch retry-storm pattern.

3. **AutoMix-style self-verify before HYBRID escalation** | grounding: AutoMix ICLR 2025, >50% cost reduction at iso-quality; FrugalGPT 98% cost reduction. | integration: Before L3+ failure triggers cloud escalation, run a cheap local "is this output structurally valid" verifier. ELAI already has retry cap=1 - add a verifier that gates whether the cloud call happens at all. Hooks into existing `RuntimeOrchestrator`.

4. **Batch API for non-interactive workloads** | grounding: 50% off across all Anthropic models, 24h SLA (Anthropic pricing doc). | integration: B-N audit runs, weekly RAG re-index, memory-bank consolidation, full-workspace test analysis - all candidates for Batch API. Add a `CLOUD_SPEND_BATCH` SSB variant tagged 24h SLA.

5. **Token budget per role, not per task** | grounding: agentic input/output 20-25x ratio, 30x variance same-task (arxiv 2604.22750). | integration: `RuntimeOrchestrator` already routes by role - add per-role input-token ceilings. Auditor gets larger context budget (reads everything), executor gets smaller (focused edits). Variance band means budget must be P50+P95, not point estimate.

6. **Cerebras / Groq free tier for fast-loop scaffolding** | grounding: Cerebras 1M tokens/day free, Groq 14.4K req/day default (getaiperks.com, tokenmix.ai). | integration: scaffold prompts (idea-framer, prompt-engineer, decompose) often need fast turnaround on simple structure - route to free-tier Groq/Cerebras backend when available. Add to B-37 RuntimeAdapter as `Backend::GroqFree`, `Backend::CerebrasFree`.

7. **DeepSeek V4 Flash as cloud floor** | grounding: $0.30/$0.50 per MTok with 90% cache discount (api-docs.deepseek.com). | integration: Cheapest cloud-mode escalation target. Add `Backend::DeepSeekV4Flash` for non-critical L3 failures where Sonnet 4.6 is overkill.

8. **Prefix-cache layout discipline in prompts** | grounding: vLLM 27.4% TTFT reduction with T-LRU; high cache hit rate "several-x speedup" (docs.vllm.ai). | integration: stable prefix = system prompt > spec > tools schema > task-specific context > variable user message. Anthropic 5-min cache requires content stays bit-identical. Audit all prompts in `crates/llm-providers` to ensure variable content is suffix-only.

9. **Cost-aware rate limiter per route** | grounding: Denial-of-Wallet attacks bypass RPS limits (handsonarchitects.com). | integration: replace any current RPS limit with $-per-minute limit. Each SSB action with cost estimate decrements a rolling 1min/10min/1h/24h budget.

10. **Local-first hard preference until ungrounded** | grounding: Qwen 7B local = $0 marginal, Sonnet 4.6 escalation = $0.30-$1.20 per step. | integration: already implemented in ELAI HYBRID mode; reinforce by raising `ELAI_LOCAL_MIN_PROMPT` from 1 to a measured value (1K-5K) so only non-trivial prompts can even attempt cloud route.

## "Wallet attack" defenses

Cited incidents and defenses, ranked by ELAI integration value.

| Incident/Pattern | Defense | Citation |
|---|---|---|
| $47K LangChain infinite-loop (4 agents ping-pong 11 days) | Hard per-task budget cap that aborts mid-task | ravoid.com/blog/ai-agent-budget-enforcement |
| 2.3M API calls in weekend (data enrichment agent misreading error code) | Velocity caps (requests/min, $/hour) + retry-limit per error-class | leanopstech.com/blog/agentic-ai-cost-runaway |
| Denial-of-Wallet via expensive prompts (RPS-based limits insufficient) | Cost-aware rate limiting per-route, $/min not req/min | handsonarchitects.com |
| Observability lag: alerts fire after spend already burned | Pre-flight cost estimation + circuit breaker at threshold | ravoid.com |
| Agent re-prompts itself indefinitely (loop) | Max-iterations cap per agent role | leanopstech.com |
| Token budget exhausted but agent continues | Terminate-and-return-partial on budget breach, not pause-and-wait | techaheadcorp.com |

ELAI mapping: existing `CLOUD_SPEND` SSB + `max_spend_per_task` + `max_spend_per_day` covers cases 1, 4, 6. Missing: (a) velocity caps (case 2), (b) per-route cost-aware limiter (case 3), (c) max-iterations per role (case 5). Layered cap recommendation from rocketedge.com 2026: $50/day soft alert > $100/day hard cutoff > $1000/month requires manual override.

## Anti-patterns

- **Trusting observability to stop spend.** Helicone/Portkey/LangSmith dashboards show what was spent, not what is about to be spent. The kill switch must be in the agent runtime (ravoid.com).
- **RPS-based rate limits on LLM endpoints.** A single 200K-token prompt can outspend 10K 1K-token prompts. Use $-per-window, not req-per-window (handsonarchitects.com).
- **Point-estimate cost budgets.** Same-task variance is 30x (arxiv 2604.22750). Always budget as P50+P95 band; never `if cost > X` against a point.
- **Stale prompt caching.** Anthropic's 5-min TTL change in early 2026 silently raised many bills 30-60%. Audit cache hit rates monthly (dev.to/whoffagents).
- **All-Opus sessions for simple work.** $15-30 per session vs $3-7 mixed Haiku/Sonnet/Opus session, same quality on most steps (morphllm.com).
- **Running a sidecar that's only used occasionally.** 575W idle is real money. ELAI already correctly stops the pod between sessions (memory:feedback_stop_pod_after_session).
- **Batch API for interactive workloads.** 24h SLA breaks user feedback loops. Only use Batch for background tasks: audits, RAG rebuilds, weekly memory consolidation.
- **Cache-write multiplier debt.** Writing to 1-hour cache (2x base) without 2+ reads loses money. Default to 5-min (1.25x, pays after 1 read).
- **Forgetting tool-use system prompt cost.** Every tool-bearing request adds 346 tokens (Anthropic). At 1000 requests/day on Sonnet that's $1+/day in tool overhead alone.
- **Reading agent benchmarks as a one-time cost.** Cognition acknowledged in 2024 that running their own SWE-bench evals burned "serious dollars"; agent eval runs are $300+ per run on the verified leaderboard (codesota.com). ELAI's contract-test sweep is the analog - cap it.

## Open questions

1. What is the real measured cache hit rate of ELAI's current `CloudGroundingClient` over a typical B-N execution? (Add cache-hit-rate gauge to MONITOR, log per-session, target >70%.)
2. What is the cost-per-task delta between Sonnet 4.6 cloud and Qwen 7B local at iso-quality on B-N audit tasks specifically? (Run head-to-head on 20 randomly-sampled audits, measure pass-rate + $.)
3. Does AutoMix's POMDP verifier improve ELAI's HYBRID gate accuracy over the current binary L3+ trigger? (A/B test on 30 baby-steps.)
4. What is the actual 5-minute cache TTL utilization rate in HYBRID escalation loops? (If most retries land >5min after first call, cache is wasted; consider 1-hour cache write for retry-heavy paths.)
5. Is the Cerebras 1M-tokens-free-per-day a sustainable subsidy or a launch promo? (Re-check Q3 2026; do not architect dependency on it without contingency.)
