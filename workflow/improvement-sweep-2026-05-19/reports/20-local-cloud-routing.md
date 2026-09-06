# Local vs cloud routing for ELAI

Scope: 2024-2026 SOTA in cost/quality routing across local+cloud models. ELAI today: LOCAL (Qwen2.5-Coder-7B NF4+FA2, 6 layers, $0), CLOUD (Sonnet/Opus, L2-5 skipped), HYBRID (local-first, escalate on L3+ failure). Multi-runtime B-37: 11 backends behind RuntimeAdapter+PerRoleRuntime+RuntimeOrchestrator. Hardening C-5: per-task/per-day ceilings, HYBRID retry cap=1, RouteBlocked SSB, CloudPanel.

## TL;DR
- Replace ELAI's all-or-nothing HYBRID escalation with a calibrated logprob/entropy cascade (AutoMix+C3PO pattern): L1 local emits a confidence score, only escalate when below a measured threshold, not on hard "L3 failure." Current "retry cap=1" leaves cost on the table for genuinely easy prompts and bails too late for genuinely hard ones.
- Add a small pre-execution difficulty classifier (BERT-scale, ~110M, RouteLLM-style) as gate keeper for HYBRID. RouteLLM hits ~40-50% cost reduction at near-GPT-4 quality without changing the generator stack. Hook: RuntimeOrchestrator pre-filter before PerRoleRuntime dispatch.
- Adopt model-personality routing (Sonnet 4.5 for execution / day-to-day code, Opus for cross-file architecture and trade-off reasoning) inside CloudPanel role assignment. Opus uses 19-50% fewer tokens on complex multi-file tasks while Sonnet wins on speed/$ for single-file work; the right default differs by L-layer, not by global config.
- Make data-leakage routing a first-class rule: regex+ML PII/secret scan in the LOCAL L1 deterministic layer must hard-block cloud escalation for any prompt touching env vars, secret-shaped strings, or paths flagged in a deny-list. Surface as a new RouteBlocked variant (PrivacyBlock) so the user sees why a "hybrid" stayed local.
- Track running monthly burn separately from per-task spend. The current `max_spend_per_task` + `max_spend_per_day` misses the "death by a thousand cheap calls" failure mode. Litellm/Langfuse-style daily-burn 2x alerts + projected-monthly burn in CloudPanel.

## Key sources
| URL / arxiv | what | takeaway |
|---|---|---|
| arxiv:2305.05176 FrugalGPT (Chen, Zaharia, Zou; TMLR 2024) [PDF](https://lingjiaochen.com/papers/2024_FrugalGPT_TMLR.pdf) | Cascade of increasing LLMs + learned scoring judge | Up to 98% cost reduction at GPT-4 parity, or +4% accuracy at same cost; cascade beats single model on cost-quality Pareto |
| arxiv:2406.18665 RouteLLM (Ong et al., LMSYS 2024) [arxiv](https://arxiv.org/abs/2406.18665) [blog](https://www.lmsys.org/blog/2024-07-01-routellm/) | Preference-data trained routers (BERT/MF/LLM-judge/SW-ranking) | 40-50% cost cut at near-GPT-4 quality; open routers on HF; OpenAI-compatible drop-in |
| arxiv:2310.12963 AutoMix (Madaan, Aggarwal et al.) [arxiv](https://arxiv.org/abs/2310.12963) | Few-shot self-verification + POMDP meta-router | ~50%+ compute reduction at parity; SLM self-verifies, escalates noisy outputs |
| arxiv:2502.00674 Self-MoA (Wenzhe Li et al.) [arxiv](https://arxiv.org/abs/2502.00674) | Aggregating outputs from the SINGLE best LLM beats mixing different LLMs | +6.6% AlpacaEval 2.0, +3.8% avg across MMLU/CRUX/MATH; mixing-different-models is sensitive to weakest member |
| arxiv:2510.08439 xRouter (Salesforce, Oct 2025) [arxiv](https://arxiv.org/abs/2510.08439) | RL-trained cost-aware router across 20+ LLMs via DAPO | ~80-90% of GPT-5 acc at <1/5 cost; cost-aware reward inside RL; OpenAI-compatible API |
| arxiv:2510.00202 RouterArena (Sep 2025) [arxiv](https://arxiv.org/abs/2510.00202) | 8K-query bench across 9 Dewey domains, 3 Bloom difficulty levels, 5 metrics | First standardized router benchmark covering accuracy/cost/optimality/robustness/latency together |
| arxiv:2510.08731 When to Reason (IBM, NeurIPS 2025) [arxiv](https://arxiv.org/abs/2510.08731) | Semantic router for vLLM that toggles reasoning mode per query | +10.2pp MMLU-Pro accuracy, -47.1% latency, -48.5% tokens vs always-reason |
| arxiv:2510.01237 Confidence-Aware Routing (Oct 2025) [arxiv](https://arxiv.org/abs/2510.01237) | Multi-signal pre-generation confidence for hallucination mitigation | Probe-based and perplexity-based methods outperform verbalized self-confidence |
| arxiv:2511.07396 C3PO (Nov 2025) [PDF](https://arxiv.org/pdf/2511.07396) | Cascade with conformal cost calibration, learned defer thresholds | Decides accept-vs-defer with cost-budget constraint; calibrated thresholds beat heuristic ones |
| arxiv:2511.03808 Prompt Difficulty Prediction (Nov 2025) [html](https://arxiv.org/html/2511.03808) | Mid-layer LLM activations highly informative for difficulty/correctness | Activation-probe routing dramatically reduces inference cost at parity |
| arxiv:2505.07680 SpecRouter (May 2025) [arxiv](https://arxiv.org/abs/2505.07680) | Adaptive multi-level speculative decoding routing | Reformulates inference as routing problem; KV-cache rollback across heterogeneous models |
| arxiv:2504.01282 Prompt-Reverse Inconsistency (Apr 2025) [PDF](https://arxiv.org/pdf/2504.01282) | "Which are correct?" vs "Which are incorrect?" produce conflicting LLM answers | Logical inconsistency failure mode independent of temperature; routers built on self-eval prone to this |
| [OpenRouter Auto Router docs](https://openrouter.ai/docs/guides/routing/routers/auto-router) | Production meta-router across dozens of models | Free meta-routing; `:nitro` (fastest) and `:floor` (cheapest) shortcuts; staged rollout pattern |
| [Cosmic JS Sonnet 4.5 vs Opus 4.5](https://www.cosmicjs.com/blog/claude-sonnet-45-vs-opus-45-a-real-world-comparison) + [NxCode Sonnet 4.6 vs Opus 4.6](https://www.nxcode.io/resources/news/claude-sonnet-4-6-vs-opus-4-6-complete-comparison-2026) | Practitioner trade-off measurements | 80/20 rule: Sonnet for execution; Opus uses 19-50% fewer tokens on complex multi-file/architecture work |
| [Kunal Ganglani $500 GPU benchmark](https://www.kunalganglani.com/blog/local-llm-vs-claude-coding-benchmark) | Qwen2.5-Coder-32B on RTX 4070 TiS vs Sonnet | Local 32B hits 85-90% of Sonnet on straightforward function gen / code-explain; complex multi-file + subtle bug detection still favor Sonnet |
| [Ian Paterson 38-task routing table](https://ianlpaterson.com/blog/llm-benchmark-2026-38-actual-tasks-15-models-for-2-29/) | 15 models x 38 real coding tasks, $2.29 total | Production-ready routing table for code, gives empirical per-task model winners |
| [Databricks GEPA-opt gpt-oss-120b](https://www.databricks.com/blog/building-state-art-enterprise-agents-90x-cheaper-automated-prompt-optimization) | Optimized open-weights beats Sonnet 4 / Opus 4.1 baselines | 22-90x cheaper than Opus 4.1 while exceeding it on enterprise agent tasks; Pareto frontier shifting |
| [Protecto 2025 PII playbook](https://www.protecto.ai/blog/securing-sensitive-data-llm-applications/) + [Keysight PII leak in prompts](https://www.keysight.com/blogs/en/tech/nwvs/2025/08/04/pii-disclosure-in-user-request) | Pre/post-model redaction + gateway pattern | Send least necessary context; gateway-level policy enforcement; output filtering as second pass |
| [Truffle Security crawl 2025](https://www.kiteworks.com/cybersecurity-risk-management/prevent-llm-data-leakage-controls/) | ~12K live API keys/passwords in one training-data crawl, 63% repeated | Routing-to-cloud of any prompt containing secret-shaped strings is a known leak vector |
| [LiteLLM spend tracking](https://docs.litellm.ai/docs/proxy/cost_tracking) + [Langfuse cost obs](https://langfuse.com/docs/observability/features/token-and-cost-tracking) + [Traceloop budgets](https://www.traceloop.com/blog/from-bills-to-budgets-how-to-track-llm-token-usage-and-cost-per-user) | Production budget tracking patterns | Daily 2x-baseline alert + monthly burn-rate alert; attribute per-feature/per-user not just per-call |

## Routing technique landscape

| Technique | Cost-quality tradeoff | Works without trained router? | Latency overhead | Citation |
|---|---|---|---|---|
| Cascade w/ learned judge (FrugalGPT) | Up to 98% cost cut at GPT-4 parity | No (judge must be trained on workload) | +1 small-model call per query that defers | arxiv:2305.05176 |
| Preference-data router (RouteLLM) | ~40-50% cost cut at near-GPT-4 | No (router trained on preference pairs) | One ~110M BERT-scale forward pass | arxiv:2406.18665 |
| Self-verification cascade (AutoMix) | ~50%+ compute cut | Yes (few-shot self-verify) | +verification prompt on SLM | arxiv:2310.12963 |
| Confidence/logprob threshold (C3PO, CALM family) | Tunable via cost-budget constraint | Yes (uses model's own logprobs) | None on accept path; +defer cost on reject | arxiv:2511.07396, arxiv:2207.07061 |
| Activation-probe difficulty (mid-layer features) | Dramatic at parity (paper-claimed) | Needs trained probe on mid-layer | One partial forward pass | arxiv:2511.03808 |
| RL cost-aware orchestrator (xRouter) | ~80-90% GPT-5 acc at <1/5 cost | No (RL-trained router LLM) | Tool-call overhead + router LLM tokens | arxiv:2510.08439 |
| Semantic-intent reasoning toggle (When to Reason) | -47% latency, -48% tokens, +10pp acc | No (intent classifier) | Classifier pass | arxiv:2510.08731 |
| Speculative multi-level (SpecRouter) | Token-latency optimization, quality preserved | No (profiling + similarity metrics) | Negative (speedup), KV-cache complexity | arxiv:2505.07680 |
| Self-MoA aggregation (single-best ensemble) | Quality gain, cost up vs single call | Yes (just sample N from best model) | N x single-model latency | arxiv:2502.00674 |
| Meta-router gateway (OpenRouter Auto / `:floor`/`:nitro`) | Production-tunable per shortcut | Yes (operational, no training) | Provider-pick overhead, usually <100ms | [OpenRouter docs](https://openrouter.ai/docs/guides/routing/routers/auto-router) |

## Pre-execution difficulty estimation

What signals actually work for routing decisions BEFORE generating:

- **Mid-layer activations from the SLM itself**: trained probes on intermediate hidden states predict both difficulty and correctness; activation-probe routing reduces cost while preserving accuracy (arxiv:2511.03808). Cheaper than running a separate classifier because the SLM forward pass is happening anyway.
- **Preference-data routers (RouteLLM family)**: BERT-scale classifier (~110M params) trained on Chatbot Arena preference pairs; predicts whether the weaker model "wins" or "loses" against the stronger. Drop-in, ~40-50% cost cut (arxiv:2406.18665).
- **Verbalized self-confidence is NOT a reliable signal**: confidence-aware routing benchmarks show probe-based + perplexity-based methods significantly outperform self-reported confidence (arxiv:2510.01237). ELAI's HYBRID can't trust "the model said it was unsure."
- **Semantic intent classification**: classify whether a prompt needs reasoning vs lookup vs trivial completion (IBM "When to Reason" vLLM router, arxiv:2510.08731). 48.5% token reduction by not invoking reasoning mode on prompts that don't need it.
- **Item Response Theory (IRT-Router, ACL 2025)**: latent difficulty scores per query, latent ability scores per model; route to minimum-cost model whose ability >= difficulty. Calibrated via prior responses.
- **Activation entropy / token-prob distribution**: high-entropy first-token distributions correlate with eventual incorrectness on reasoning tasks (arxiv:2510.08146 "Think Just Enough"). Useful for L1 deterministic gate.

## Cost-quality Pareto for code tasks (2024-2026 measurements)

- **Sonnet 4.5/4.6 vs local 32B**: Qwen2.5-Coder-32B on consumer GPU (RTX 4070 Ti Super) hits 85-90% of Sonnet on single-function generation and code explanation; complex multi-file reasoning and subtle-bug detection still favor Sonnet ([Kunal Ganglani benchmark](https://www.kunalganglani.com/blog/local-llm-vs-claude-coding-benchmark)). For ELAI's 7B default, expect a wider gap on multi-file tasks.
- **SWE-bench Verified March 2026**: Opus 4.5 at 80.9%, Sonnet 4.5 at 77.2%, Qwen3-Coder ~70.6%, DeepSeek V3.2-Special 73.1% ([SWE-bench](https://www.vals.ai/benchmarks/swebench), [philarchive comparative analysis](https://philarchive.org/archive/JOSOVC)). Gap from Sonnet to a frontier open coder is ~6-7pp; from local-7B to Sonnet is much larger on agentic tasks.
- **Sonnet vs Opus on complex code**: Opus uses 19.3% fewer total tokens on complex applications, sometimes 20-50% fewer on multi-file refactors ([Cosmic JS](https://www.cosmicjs.com/blog/claude-sonnet-45-vs-opus-45-a-real-world-comparison), [NxCode](https://www.nxcode.io/resources/news/claude-sonnet-4-6-vs-opus-4-6-complete-comparison-2026)); on simple/medium tasks Sonnet wins on $/token throughput. So the right-cloud-model decision is task-shape-dependent, not a global toggle.
- **Pareto-frontier shift via optimization**: GEPA-optimized gpt-oss-120b matches Sonnet 4 / Opus 4.1 baselines at 22-90x lower cost on enterprise agent tasks ([Databricks](https://www.databricks.com/blog/building-state-art-enterprise-agents-90x-cheaper-automated-prompt-optimization)). Implication: ELAI's per-role prompt optimization is leveraged cost reduction independent of routing.
- **GEPA gpt-oss-120b** at 22-90x cost cut requires a runtime ELAI doesn't have wired by default (it's an open-weights 120B). Bookmark for future RuntimeAdapter entry, not a today-action.

## Routing failure modes (cited)

- **Self-evaluation is contradictory**: same LLM answering "which is correct?" and "which is incorrect?" gives logically inconsistent answers (arxiv:2504.01282 PRIN). Routers that use the small model to grade its own output (AutoMix-style) inherit this; meta-verifier mitigates but does not eliminate.
- **Calibration drift**: confidence thresholds set by intuition will be miscalibrated; production inputs are far more variable than test inputs ([Lorenzo Kotalla "Practical Failure Modes"](https://medium.com/@lorenzo.kotalla/practical-failure-modes-in-llm-systems-and-where-they-usually-come-from-8f0fd59b51c4)). Threshold-tuning is a continuous ops cost, not a one-shot.
- **Same-prompt different-result inconsistency** even at temp=0 across batch/version/hardware ([OpenAI dev forum](https://community.openai.com/t/debugging-inconsistent-llm-results-between-local-and-server/1373203), [arxiv:2406.12334](https://arxiv.org/pdf/2406.12334)). Routing decisions made on a single sample are noisy; majority-vote of 3 small-model samples is a cheap robustness fix.
- **Mixing different LLMs underperforms aggregating one strong model**: Self-MoA wins +3.8-6.6% vs MoA-across-models because MoA quality is bottlenecked by the weakest member (arxiv:2502.00674). Don't naively ensemble across the 11 backends; pick the best per-role.
- **Knowledge-probing inconsistency**: option-shuffling produces contradictory assessments of the same model (arxiv:2505.21701), so router-eval datasets are themselves noisy.
- **Recursive "router-is-wrong" problem**: a learned router has its own accuracy ceiling; xRouter (arxiv:2510.08439) addresses via RL but training cost is non-trivial; OpenRouter Auto sidesteps via meta-routing without training.

## Best techniques to adopt (ranked)

1. **Logprob/entropy-gated HYBRID escalation** | grounding: arxiv:2511.07396 C3PO, arxiv:2510.08146 entropy-as-confidence, arxiv:2510.01237 probe-based confidence | hook: RuntimeOrchestrator emits a per-response confidence_score from local L1; HYBRID escalation condition becomes `confidence_score < calibrated_threshold` instead of "L3 failed." Default threshold from a holdout calibration set; expose in CloudConfig.
2. **Pre-execution difficulty classifier (RouteLLM drop-in)** | grounding: arxiv:2406.18665 RouteLLM, arxiv:2511.03808 activation probes | hook: New RuntimeAdapter `DifficultyClassifierAdapter` runs before PerRoleRuntime dispatch; emits `difficulty: easy|medium|hard`; CloudPanel surfaces routing reason. Use RouteLLM's open BERT router as starting weights; fine-tune on ELAI's task trace.
3. **Model-personality role assignment** | grounding: Cosmic JS + NxCode practitioner comparisons; SWE-bench split | hook: PerRoleRuntime per-role config — execution/single-file roles default Sonnet; architecture/cross-file/trade-off roles default Opus; math-heavy escalation default R1 (separate backend). Move from single-model HYBRID to role-aware HYBRID.
4. **Privacy-routing gate (RouteBlocked::PrivacyBlock)** | grounding: Truffle 12K-secrets-in-crawl finding, Protecto 2025 playbook, Keysight 2025 prompt-leak | hook: New L1 deterministic check before cloud escalation; regex+ML scan for env-var names, secret-shaped tokens (AKIA*, sk-*, eyJ*), paths in deny-list. Adds a RouteBlocked variant so the SSB-emitted reason is visible in CloudPanel.
5. **Daily-burn-rate + projected-monthly view in CloudPanel** | grounding: LiteLLM + Langfuse + Traceloop production patterns | hook: Track running-7d/30d in same store as per-task spend; alert at 2x average daily burn AND when projected-monthly exceeds configured monthly ceiling; project from per-task histogram of recent N tasks.
6. **Sample-of-3 majority vote on the routing decision** (not the answer) | grounding: arxiv:2406.12334 prompt-engineering sensitivity, arxiv:2504.01282 PRIN | hook: Router classifier or confidence-gate runs 3 cheap samples on local; majority/median decides cloud-vs-local. Cost: 3 SLM passes; benefit: stable routing under temperature/version drift.
7. **Per-role meta-router shortcut (`:floor` / `:nitro` semantics)** | grounding: [OpenRouter Auto Router docs](https://openrouter.ai/docs/guides/routing/routers/auto-router) | hook: PerRoleRuntime gains `priority: cost|latency|quality` hint per role; orchestrator picks cheapest qualifying RuntimeAdapter (e.g., Groq for latency-priority chat roles, Sonnet for quality-priority code roles).
8. **Semantic reasoning-mode toggle for L3+** | grounding: arxiv:2510.08731 When to Reason | hook: Before invoking extended-thinking on cloud, classify whether the prompt benefits from reasoning; -48% token reduction on prompts that don't. Same pattern, applied at L3 escalation entry.
9. **Self-MoA aggregation as L4/L5 only** | grounding: arxiv:2502.00674 | hook: When the strongest cloud model is selected and quality matters > cost, sample N=3-5 from the SAME model and aggregate. Do NOT mix across different models. Useful for hard verification layers in HYBRID mode.

## Data leakage / safety routing rules (best practices, cited)

- Send the least necessary context (no full files when a 50-line snippet suffices); redact/tokenize/mask PII/PHI and secrets pre- AND post-model ([Protecto 2025](https://www.protecto.ai/blog/securing-sensitive-data-llm-applications/), [Kiteworks](https://www.kiteworks.com/cybersecurity-risk-management/prevent-llm-data-leakage-controls/)).
- Route every cloud call through a gateway with policy enforcement; the gateway inspects responses for unintended sensitive-data emission too ([API7 AI gateway](https://api7.ai/blog/ai-gateway-security-compliance)). For ELAI: CloudPanel + RuntimeAdapter is the gateway choke point — add policy checks there.
- Pattern + ML detection: regex for obvious shapes (API keys, JWTs, AWS IDs, private-key PEMs) PLUS ML classifier for fuzzy PII; combine with policy-based mask/tokenize/block, not just block.
- Truffle Security found ~12K live secrets in a major training-data crawl, 63% repeated across pages ([Kiteworks summary](https://www.kiteworks.com/cybersecurity-risk-management/prevent-llm-data-leakage-controls/)) — secrets that leave your machine assume they're public.
- Default-deny for sensitive paths (`.env`, `~/.ssh/`, `~/.aws/credentials`, `secrets/*.yml`); allow only with explicit user toggle, log every override.
- Output filtering as a second pass: even if the input is clean, the model can hallucinate or memorize-and-emit ([Protecto 2025 playbook](https://www.protecto.ai/blog/securing-sensitive-data-llm-applications/)). Run regex on responses before showing.

## Anti-patterns (cited)

- Trusting verbalized self-confidence ("how confident are you?" prompt): poorly calibrated; probe-based and perplexity-based dominate (arxiv:2510.01237).
- Naive mixture-of-different-LLMs ensembling: weakest member drags quality (arxiv:2502.00674 Self-MoA). Don't aggregate cloud+local outputs blindly.
- Heuristic confidence thresholds without calibration set: drifts with workload, fails silently ([Lorenzo Kotalla failure modes](https://medium.com/@lorenzo.kotalla/practical-failure-modes-in-llm-systems-and-where-they-usually-come-from-8f0fd59b51c4), arxiv:2511.07396 C3PO conformal calibration).
- Single-call routing decision at temp>0: same-prompt-different-result causes flapping (arxiv:2406.12334). Use majority-of-3 for the router signal.
- Auto-escalating to cloud on every "L3 failure" without checking whether L3 failure was actually quality-driven or syntactic: wastes budget on hard prompts that needed a better prompt, not a bigger model.
- All-or-nothing HYBRID retry cap=1: doesn't model the actual problem, which is a continuous confidence gradient; better: continuous threshold with budget-aware cap.
- Sending entire repo context to cloud "to be safe": leaks secrets + costs 50x what a 200-line excerpt would; principle of least context ([Protecto](https://www.protecto.ai/blog/securing-sensitive-data-llm-applications/)).
- Charging per-task only and ignoring per-day/per-month burn: death-by-thousand-cheap-calls ([LiteLLM spend tracking](https://docs.litellm.ai/docs/proxy/cost_tracking)).
- Building the router on training data from a different distribution than production traffic: routers must be evaluated on RouterArena-style multi-domain, multi-difficulty splits (arxiv:2510.00202).

## Open questions

1. What is the actual local-7B Qwen2.5-Coder-Abliterated -> Sonnet 4.5/Opus 4.5 routing curve on ELAI's own 6-layer L1-L5 task distribution? Need a held-out trace of 200+ tasks with both runs to fit a calibration threshold. Measurable: % of tasks where local-7B output passes auditor without cloud, segmented by L-layer.
2. Does logprob-entropy on the L1 deterministic layer correlate with auditor BLOCK on ELAI tasks? Test by logging L1 output entropy + auditor outcome over the next 100 tasks. Measurable: AUC of entropy-as-classifier for BLOCK prediction.
3. What fraction of HYBRID escalations are quality-driven vs prompt-driven failures? Currently retry cap=1 hides this. Measurable: log root-cause classification of each escalation event; target <30% prompt-quality.
4. Cost of a RouteLLM-trained classifier (~110M BERT) at inference time vs the cost of one wasted Sonnet call: at what daily-task volume does training-the-router pay back? Measurable: amortized cost per task with classifier vs without, at varying task volume.
5. Does Self-MoA-on-Opus at N=3 beat single-Opus-call for HEAVY L5 tasks at acceptable cost? Measurable: pass-rate delta per HEAVY task at 3x cost vs 1x.
