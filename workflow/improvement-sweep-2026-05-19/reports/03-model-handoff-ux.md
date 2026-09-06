# Seamless multi-model chat handoff

## TL;DR
- Unify voice at the SYSTEM-PROMPT layer, not via post-processing: a single "ELAI voice" preamble prepended to every role prompt (executor / auditor / etc.) shifts the rendered persona far more reliably than rewriting outputs after the fact, because system prompts are the primary mechanism for persistent application-level behavior (Tetrate; codified prompting, arxiv 2507.03254). Optional: a thin "style-normalizer" pass on a small local model that strips role-specific tells (e.g. auditor's "BLOCK:" prefix) before display.
- Treat the model swap as INVISIBLE by default and EXPOSABLE on demand, the way Aider and Cursor do: never name the model unless the user opens a panel; instead show the role ("auditor running") plus an estimated latency. Cursor 2.0's Composer ships an explicit selector but most users stay on the default; Devin and Replit Agent hide model identity entirely and that hiding is itself contentious (transparency complaints in COMPEL case study).
- For genuine cross-model KV/prefix-cache reuse: only feasible across "same-architecture variants" (DroidSpeak, arxiv 2411.02820). For ELAI's mixed Qwen-Coder local + Claude cloud setup, you CANNOT share KV. The realistic substitute is sleep-time compute (arxiv 2504.13171): pre-digest the rolling chat context into a dense intermediate that any model can ingest, refreshed during idle turns. This is the single most impactful change for the "loads and unloads many times" pain.
- Use Aider's architect/editor split as the canonical mental model: a strong reasoning model plans, a cheap fast model executes (aider.chat/2024/09/26/architect.html, SOTA 85% on aider's bench). Map to ELAI: prompt-engineer + context-compiler = architect tier; executor + memory-writer = editor tier. The boundary stays semantic, not visual.
- Predictively pre-load the NEXT-LIKELY model based on FSM state transition probabilities. CUDA-graph capture dominates cold start more than weight load now (arxiv 2604.06664 Foundry; weights = 1-2s via RDMA, graph capture = tens of seconds to minutes), so the win is keeping graph-captured models warm in a small rotating pool (vLLM warm-pool pattern, Medium/bhagyarana80). On 12GB single-GPU this is "no swap"; on 24/48GB tiers swap ONE adjacent role ahead.

## Key papers / sources

Routing:
- arxiv:2305.05176 | FrugalGPT | LLM cascade with DistilBERT correctness classifier escalates query up a cost-ordered chain; cost reduction at parity quality.
- arxiv:2406.18665 | RouteLLM (LMSYS) | Train a two-model strong/weak router from preference data; 85% cost cut on MT-Bench at GPT-4 quality.
- arxiv:2310.12963 | AutoMix | Few-shot self-verification as entailment check, POMDP router decides escalate; 50%+ compute cut, up to 89% incremental-benefit-per-cost gain.
- arxiv:2505.07680 | SpecRouter | Adaptive multi-level speculative-decoding routing reacting to per-request complexity + system load.
- arxiv:2502.00674 | Rethinking Mixture-of-Agents | Self-MoA (single top model, multiple samples) beats heterogeneous MoA by 6.6% on AlpacaEval-2; mixing different LLMs often HURTS.
- arxiv:2509.23537 | Beyond the Strongest LLM | Multi-turn orchestration matches/exceeds best single model but is PLANNER-LIMITED not executor-limited; invest in orchestrator, not bigger sub-agents.

Persona / voice continuity:
- arxiv:2507.21509 | Persona Vectors (Anthropic) | Linear directions in activation space encode traits (evil/sycophancy/hallucination); steer via vector addition at inference; controls drift better than prompt-only.
- arxiv:2602.15669 | PERSONA framework | PERSONA-BASE extracts orthogonal trait vectors; PERSONA-ALGEBRA = scalar mul / add / sub for intensity, composition, suppression; inference-time only.
- arxiv:2602.15013 | Text Style Transfer via PEFT + round-trip translation | Practical recipe for tone normalization across models when activation access is unavailable (i.e. cloud APIs).
- arxiv:2508.04826 | Persistent Instability in LLM Personality (AAAI 2026) | Conversation history and scale measurably destabilize persona over long contexts; prompt-only persona "frequently increases hallucination rates."
- arxiv:2507.18294 | StyleAdaptedLM | Stylistic instruction-following via lightweight adapter; useful if ELAI ever fine-tunes a "voice adapter" for its local model.

Latency masking / handoff:
- arxiv:2411.02820 | DroidSpeak (Microsoft) | Cross-LLM KV reuse for same-architecture variants only; 4x throughput, 3.1x faster prefill via selective layer recomputation.
- arxiv:2504.13171 | Sleep-time Compute | Pre-process context while idle into a dense re-representation; ~5x test-time compute reduction, up to 13% accuracy lift on reasoning. Direct answer to ELAI's "context loaded/unloaded many times" pain.
- arxiv:2310.05736 | LLMLingua | Up to 20x prompt compression with little loss, 1.7-5.7x practical speedup; useful for handoff payloads between models.
- arxiv:2604.06664 | Foundry (CUDA-graph templates) | Graph capture dominates cold start (tens of seconds to minutes) vs weight load (1-2s RDMA); template-based materialization for fast multi-model serving.
- arxiv:2512.01357 | Tangram | Serverless LLM loading via GPU memory reuse and affinity; relevant for warm-pool design on multi-tier VRAM.
- NVIDIA NIM docs | KV cache reuse / prefix caching | Production behavior of prefix caching: same model, same leading tokens = reuse; across different models = not supported in stock vLLM.

Product UX surveys:
- cursor.com/blog/2-0 | Cursor 2.0 + Composer | Up to 8 parallel agents, per-agent model assignment, side-by-side diff comparison; explicit selector but invisible default.
- aider.chat/2024/09/26/architect.html | Aider architect/editor mode | Strong planner + cheap editor; SOTA 85% on aider's code-edit benchmark with o1-preview + DeepSeek/o1-mini.
- developers.openai.com/codex/cli | OpenAI Codex CLI | /model command swaps GPT-5.4 / 5.4-mini / 5.3-Codex-Spark mid-thread; plan-then-act with inline approve/reject.
- code.claude.com/docs/en/sub-agents | Claude Code subagents | Each subagent = fresh context window, returns only a summary, parent never sees subagent transcript. Direct architectural prior for ELAI's FSM agent isolation.
- compelframework.org/articles/casestudy-devin-and-replit | Devin + Replit incidents | Devin's model selection is non-transparent and that opacity has caused incident-class user trust failures.

## How shipped products handle multi-model UX (survey)

| Product | Model strategy | What user sees | What they hide | Failure mode |
|---|---|---|---|---|
| Cursor 2.0 + Composer | Selector per-agent; up to 8 parallel; Composer = in-house fast model | Model name in dropdown; side-by-side diff; "Run All Agents" | Token routing internals, per-tool model swaps | User picks wrong model for task, no auto-routing guard |
| Aider | Architect/editor pair; explicit `--architect` flag | Two model names in startup banner; diff format | Which model emitted which token of the plan | Editor model botches diff format, architect plan is lost |
| Continue.dev | Per-feature config (chat/autocomplete/edit) in `config.json` | Three model slots in YAML | Nothing — fully exposed | Config drift across features, voice mismatch chat vs autocomplete |
| Cline | Single primary model + optional separate "thinker" model (GH disc #387) | Single model badge; thinking model toggle | Tool-loop internals | Long agent loops, model switch surprises user mid-task |
| Claude Code | Subagents with isolated context; each returns summary; orchestrator stays user-facing | "Running X agent" status; agent name | Subagent transcripts, model identity inside subagents | Latency from cold subagent context build; lost nuance via summary |
| Codex CLI | `/model` swap mid-thread; plan-then-act with inline approval | Model name in `/model` output; plan preview | Reasoning-effort tier internals | Plan/execute mismatch if model swapped between steps |
| Replit Agent | Single conversational agent, model hidden | Step-by-step build progress | Model identity entirely | Trust loss when failures occur — user can't reason about why |
| Devin | Autonomous; model identity non-transparent (per COMPEL case study) | Task plan + execution log | Model selection, escalation logic | Opacity itself becomes the failure (incident trust) |

The pattern: tools whose users are developers (Cursor, Aider, Continue, Codex CLI) EXPOSE the model selector and let power-users tune it. Tools whose users are "deliver me a result" (Devin, Replit Agent) HIDE the model and pay a transparency cost. ELAI's user is a developer, so default toward exposure but make it a "drawer" not a primary surface.

## Techniques for cross-model coherence

1. Unified system-prompt preamble | Tetrate "system vs user prompts" guidance; codified prompting (arxiv:2507.03254) | Define one `voice.md` template prepended to every role prompt (executor, auditor, prompt-engineer, ...) before role-specific instructions. Integration hook: `.elai_cc/crates/agents/src/role_prompt.rs` build function — single source of `tone`, `formality`, `forbidden-phrasings`, `confidence-calibration` clauses. Lowest cost, highest leverage. Same intervention Aider uses by sharing the diff format spec across architect+editor.

2. Sleep-time digest of chat context | arxiv:2504.13171 | Spawn a background `digest-writer` agent (small local model) during IDLE FSM transitions. It produces a "rolling chat summary" + "stable user-intent vector" in workflow memory. Any next-turn agent reads the digest instead of replaying raw chat. Cuts the "loads and unloads many times" pain by making the LOAD payload constant-size and pre-built. Integration hook: extend `memory-writer` agent with mode=digest, fire from `IDLE` transition only.

3. Style-normalizer post-pass on a tiny local model | arxiv:2602.15013 (round-trip TST); StyleAdaptedLM (arxiv:2507.18294) | Optional thin pass that strips agent-specific tells ("BLOCK:" prefix, "Verified:" leader, JSON-mode artifacts) before display. Only triggered when source model != "voice anchor" model. Integration hook: `crates/cli/src/live_cli.rs` rendering layer, gated by `ELAI_STYLE_NORMALIZE=1`. Watch latency cost — add only if A/B testing shows real coherence win.

4. Persona vectors on the LOCAL model | arxiv:2507.21509 (Anthropic), arxiv:2602.15669 (PERSONA) | Compute an "ELAI assistant" steering vector for Qwen2.5-Coder via contrastive activations; apply during local-model decode. Cannot apply to cloud models (no activation access), so this only stabilizes the LOCAL half — but the LOCAL half is where the biggest variance is. Integration hook: `python/elai_inference/server.py` decode loop, optional `--persona-vector` arg.

5. Cross-model prefix cache: SKIP unless same-arch variants | arxiv:2411.02820 DroidSpeak | DroidSpeak only works for variants of the same backbone (e.g. base + fine-tuned). Qwen-Coder vs Claude = NO. Even on the SAME local model, prefix caching is automatic in vLLM and should already be in effect — verify the `enable_prefix_caching` flag is set. Integration hook: `python/elai_inference/server.py`, set `enable_prefix_caching=True` on the engine config.

6. Predictive model preload on FSM transition | arxiv:2604.06664 Foundry; vLLM warm-pool (bhagyarana80) | When FSM enters `AUDITING`, the next state is most likely `EXECUTING` or `MEMORIZING` — pre-warm those role weights and (more importantly) capture their CUDA graphs. Only meaningful on 24/48GB tiers; on 12GB this is a no-op. Integration hook: `crates/cli/src/runtime_cmd.rs` `RuntimeOrchestrator` — add a `predicted_next_role` hint from FSM dispatcher.

7. Unified handoff payload via LLMLingua compression | arxiv:2310.05736 | When handing off context from a big cloud model to a small local model (or vice versa), compress the rolling history 5-20x. Integration hook: `crates/agents/src/handoff.rs` — already exists per git status, extend with a compressor pass before serializing the handoff blob.

8. Architect/editor split mapped onto ELAI roles | aider.chat/2024/09/26/architect.html, SOTA 85% | Treat `prompt-engineer` + `context-compiler` as the "architect tier" (always cloud-capable, deeper reasoning). Treat `executor` + `memory-writer` as the "editor tier" (cheap, local-first, must follow plan verbatim). This is the same boundary you already enforce — make it explicit in the runtime so the user sees ONE "ELAI is reasoning then editing" UX, not "switching to model X".

## When NOT to swap

- Single 12GB GPU tier, simple tasks: arxiv:2502.00674 Rethinking-MoA shows Self-MoA (one strong model, sampled multiple times) beats heterogeneous MoA by 6.6% on AlpacaEval-2. Don't swap; sample once on the local model.
- Tasks <1000 tokens: warm-pool/graph-capture overhead (arxiv:2604.06664) dominates; a single model's first token will arrive sooner than the second model can spin up.
- Stylistic continuity in a long-running chat: arxiv:2508.04826 — persona instability grows with conversation history; mid-thread model swap amplifies this. Pin the voice-bearing role (the one whose tokens the user reads in the CommandWindow) to ONE model for the chat session.
- Cross-architecture KV reuse hopes: DroidSpeak (arxiv:2411.02820) explicitly limits gains to "same architecture" variants. Don't design around KV transfer between Qwen and Claude.
- Planner-limited tasks: arxiv:2509.23537 — adding more sub-agent scale doesn't help if the orchestrator is weak. If audits regress, upgrade the orchestrator model (or the role prompt for `prompt-engineer`/FSM dispatcher), don't add more specialized executors.
- "Mid-step" reasoning chains: Codex CLI deliberately treats `/model` as a between-thread switch, not within a plan-then-act sequence. ELAI's per-FSM-state swap is fine because FSM states ARE the boundaries, but never swap inside a single agent's response.

## Anti-patterns

- Naming the model in the user-facing message ("Now consulting Claude Opus..."). Devin/Replit incidents show this either (a) over-promises capability the user then tests and finds missing, or (b) erodes trust when results are bad ("you said you used the smart one"). Show ROLE, hide MODEL. (compelframework case study).
- Heterogeneous Mixture-of-Agents as a default. arxiv:2502.00674: mixing different LLMs underperforms Self-MoA on a single top model by ~6.6% on AlpacaEval-2 and ~3.8% averaged. Don't ensemble for the sake of ensembling.
- Aggressive prompt-only persona pinning. arxiv:2508.04826 + arxiv:2507.21509 both flag that strong prompt-based persona instructions "frequently increase hallucination rates" and remain unstable over long contexts. Use a short anchor + activation steering, not a long persona monologue.
- Cross-architecture KV cache reuse plans. arxiv:2411.02820 limits the technique to same-architecture variants. Building infra to "share KV between Qwen and Claude" is wasted engineering.
- Hidden model swaps mid-response. Codex CLI / Aider both lock model PER step; swapping models WHILE a single response streams produces tone discontinuity within one bubble (worst possible UX surface).
- Reactive cold-start on every FSM transition. arxiv:2604.06664: graph capture is the bottleneck, not weights. If you don't reuse the captured graph, every transition will feel like a 10-60s pause. The "loads/unloads many times" symptom is most likely THIS, not weight transfer.
- Post-processing style-normalizer ON BY DEFAULT. It will mask real signal from auditor BLOCKs and confidence calibration tags. Only enable for the user-facing CommandWindow stream, never inside agent-to-agent communication.

## Open questions

1. What is ELAI's current per-FSM-transition cold-start cost broken down by (a) weight load, (b) CUDA graph capture, (c) prefix re-encode? Measure before optimizing — Foundry (arxiv:2604.06664) suggests graph capture dominates but this should be confirmed for the specific Qwen-Coder + 12GB/24GB configs.
2. Does the local Qwen2.5-Coder respond to persona-vector steering with the same effect sizes Anthropic measured on production-scale models (arxiv:2507.21509)? Steering vectors are model-specific; need to extract+test on the actual 3B/7B abliterated variants.
3. What is the measurable voice-coherence delta (human eval + an LLM-judge) between (i) unified system-prompt preamble alone, (ii) preamble + style-normalizer post-pass, (iii) preamble + persona vectors? Need an A/B harness — add to the metrics pipeline.
4. At what conversation length does sleep-time digest (arxiv:2504.13171) start to outperform raw re-prompting on a Qwen2.5-Coder-3B local? The 5x compute-cut number is from larger reasoning models; small-model behavior may differ.
5. Does Self-MoA (arxiv:2502.00674) on the local model alone beat ELAI's current swap-to-cloud-for-hard-tasks pattern on the actual ELAI audit/executor benchmarks, controlled for cost?
