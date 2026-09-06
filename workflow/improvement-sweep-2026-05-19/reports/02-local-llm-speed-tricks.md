# Local LLM speed tricks for ELAI

Scope: how to make Qwen2.5-Coder-7B-Instruct-abliterated NF4+FA2 token output FEEL fast on a 12 GB consumer GPU, plus what ACTUALLY ships more tokens per second on a dense uniform-layer model that must keep DoLa/ETC LogitsProcessor compatibility. 17 web sources, 4 papers/READMEs cached to .claude/papers/.

## TL;DR
- Ship FIRST: prompt-lookup decoding (n-gram speculative). 2x-4x on input-grounded tasks (code edits, refactors, summarization where output reuses prompt tokens). One line in HF transformers (`prompt_lookup_num_tokens=10`), drop-in to `model.generate`, KEEPS DoLa/ETC LogitsProcessor wiring intact. Zero training, zero extra VRAM. Source: apoorvumang/prompt-lookup-decoding + HF transformers v4.41+ assistive decoding API.
- Ship SECOND (UX, not speed): `smoothStream` style frame-paced rendering on the dashboard side. Vercel AI SDK ships this for free; the trick is the dashboard buffers raw SSE chunks and emits to the DOM on a fixed cadence (e.g. ~30-60 chars/sec) so the "letters coming up slowly" feel becomes "letters reading at a natural pace." Doesn't change tokens/sec, changes the perceived rhythm.
- Ship THIRD: prompt-prefix caching at the sidecar. ELAI runs `model.generate` directly; HF `StaticCache` + a manual prefix-KV reuse layer collapses repeat system-prompt prefill to near-zero. vLLM's APC shows the upper bound (multi-x lower TTFT on repeated prefixes). On 12 GB single-GPU agentic flows where every turn reuses the same 2-4 KB system block, this is bigger than any kernel trick.
- 32B-tier add-on: EAGLE-3 (2503.01840) once we move to the 48 GB box. 4.1-6.5x at temp 0 on dense models; vLLM/SGLang ship it; integrates as a decoding-time wrapper, not a model swap. Do NOT ship on 7B local because the draft-head VRAM cost competes with KV cache on 12 GB.
- Anti-trick worth NOT shipping: Skeleton-of-Thought. Breaks streaming semantics, hurts coherence on coding tasks, conflicts with DoLa contrastive layer reads. Good for Q&A bots, bad for ELAI.

## Key papers / sources

Spec-decoding
- arXiv 2503.01840 | EAGLE-3 (Li et al., NeurIPS 2025) | training-time-test + tri-layer feature fusion, 4.1-6.5x at temp 0, switches from feature- to direct-token prediction; production standard in vLLM/SGLang/Vertex.
- arXiv 2406.16858 | EAGLE-2 (Li et al., 2024) | dynamic context-dependent draft tree (vs static), beam-search per step, draft confidence ~= acceptance probability, ~3x speedup baseline.
- arXiv 2401.10774 | Medusa (Cai et al., ICML 2024) | extra MLP heads on the target model predicting tokens 2..N in parallel; Medusa-2 full-train + self-distill gives 2.2-3.6x.
- arXiv 2402.02057 | Lookahead Decoding (Fu et al., ICML 2024) | Jacobi iteration + n-gram cache, lossless, 1.5-2.3x single-GPU, up to 4x with strong scaling; no draft model required.
- arXiv 2311.08252 | REST (He et al., NAACL 2024) | retrieval-from-datastore speculative decoding; 1.62-2.36x on 7B-13B for code/text; plug-and-play on any decoder.
- github.com/apoorvumang/prompt-lookup-decoding | Prompt Lookup Decoding | replace draft model with last-n-gram suffix search inside the prompt; 2x-4x on input-grounded tasks; HF supports natively via `prompt_lookup_num_tokens`.
- arXiv 2307.15337 | Skeleton-of-Thought (Ning et al., ICLR 2024) | prompt-level parallel decoding: generate skeleton, expand each point in parallel; helpful for QA, breaks coherence on code.

KV-cache
- arXiv 2402.02750 | KIVI (Liu et al., ICML 2024) | tuning-free asymmetric 2-bit KV quant, 2.6x peak-mem cut, up to 4x batch, 2.35-3.47x throughput; key per-channel, value per-token.
- arXiv 2504.19874 | TurboQuant (ICLR 2026) | random-rotation + scalar quant for KV; quality-neutral at 3.5 bpc, marginal loss at 2.5 bpc; data-oblivious, no calibration. Already on ELAI's 32B roadmap.
- docs.vllm.ai/.../automatic_prefix_caching | vLLM APC | KV reuse across requests sharing the same prefix; multi-x TTFT cut on repeated system prompts (AMD MI300X measurements 2024-10).
- lmsys.org/blog/2024-01-17-sglang | SGLang RadixAttention | radix-tree LRU over the KV pool, up to 6.4x throughput on shared-prefix workloads (few-shot, agent loops, ToT/SoT).

Attention-kernel / serving
- arXiv 2407.08608 | FlashAttention-3 (Shah et al., NeurIPS 2024) | warp-specialization + FP8 on Hopper, 1.5-2x over FA2, 75% H100 utilization vs 35% for FA2. Not applicable on consumer Ampere/Ada (FA2 stays the ceiling on 3090/4090/12 GB tier).
- SOSP'23 / vllm-project | PagedAttention + continuous batching | block-paged KV (16-token blocks) + scheduler-level batch reformulation; 2-4x over vanilla HF on mixed-length workloads.
- huggingface.co/docs/transformers/.../perf_torch_compile | `torch.compile` + StaticCache | `mode="reduce-overhead"` with fixed/bucketed shapes, 20-50% latency cut on 7B-13B generate. Gotcha: shape-change recompilation; static-cache path needed for sustained speedup.

Perceived-latency UX
- nngroup / Doherty threshold (1982) referenced in modern TTFT writeups | 0.1 s instantaneous, 1 s flow-preserving, 10 s disengagement. TTFT <200 ms feels snappy, >1.5 s feels broken even if tail throughput is identical.
- ResearchGate 326858669 | Skeleton screens (Bilgihan et al.) | structural placeholders rate higher than spinners on perceived speed AND ease of navigation; users perceive load as 30-50% faster.
- arXiv 2401.12961 | Eloquent | robust transmission scheme for LLM token streaming; targets dropped/reordered chunks over lossy networks (relevant for the remote-sidecar tunnel).
- arXiv 2510.02758 | TokenFlow | preemptive scheduling for responsive streaming under request bursts; reduces tail-latency variance which is what users perceive as "stuttering."

Serving / streaming-architecture
- getstream.io/blog/websocket-sse + dev.to/polliog | SSE vs WebSocket for LLM streaming | SSE wins for 95% of token-streaming UIs; ~3 ms latency delta vs WS is irrelevant at human read rates; reuses HTTP auth/cache/middleware. Anthropic, OpenAI, Gemini all SSE.
- vercel.com/blog/ai-sdk-4-1 + ai-sdk.dev | `smoothStream` | client-side stream transform that smooths chunky provider deltas into frame-paced render; the SDK ships this because raw provider chunks are bursty.
- smashingmagazine.com/2026/05/designing-stable-interfaces-streaming-content | Stable streaming UI | buffer partial markdown, defer incomplete-structure render, prevent layout thrash on every token. Direct fix for the ELAI dashboard's "letters appearing" symptom.

## Actual-speedup techniques (Qwen2.5-Coder dense 7B/3B/32B compatible)

| Technique | Reported speedup | Constraints | Works on Qwen2.5-Coder? | Integrates with `model.generate` + LogitsProcessor? |
|---|---|---|---|---|
| Prompt-Lookup Decoding (n-gram) | 2x-4x on input-grounded tasks | only useful when output reuses prompt tokens (code edits, refactor, summarize) | Yes, dense uniform-layer, no arch dep | Yes, native in HF `generate`, `prompt_lookup_num_tokens=N` |
| Lookahead Decoding (Jacobi) | 1.5-2.3x single-GPU, lossless | needs custom decoding loop; CUDA-graph friendly | Yes, model-agnostic | Partial — replaces the loop; LogitsProcessor must be moved into the verifier pass |
| Medusa / Medusa-2 | 2.2-3.6x | requires training extra heads on the target model | Yes if we train heads on Qwen2.5-Coder | Heads run BEFORE LogitsProcessor; compatible but draft tokens bypass DoLa contrast |
| EAGLE-2 | ~3x | needs trained draft head; one head per target model | Yes (vLLM/SGLang ship Qwen recipes) | Verification step runs the target model; LogitsProcessor applies to verification only |
| EAGLE-3 | 4.1-6.5x at temp 0 | needs trained draft head; tri-layer fusion access to hidden states | Yes, dense Qwen supported | Same caveat as EAGLE-2; speculative path skips DoLa on draft tokens |
| REST (retrieval-based) | 1.62-2.36x | needs datastore (code corpus); n-gram trie | Yes; ideal for code agents | Yes; verification at target model, LogitsProcessor preserved |
| Skeleton-of-Thought | wall-clock 1.5-2x (Q&A only) | needs two-stage prompting; breaks single-stream UX | Yes | NO — restructures the generation, DoLa contrasts go stale between skeleton/expand |
| vLLM Automatic Prefix Caching | multi-x TTFT cut on repeated prefixes | needs vLLM (not HF generate) | Yes | NO — vLLM switch loses LogitsProcessor wiring (vLLM has its own logits API) |
| SGLang RadixAttention | up to 6.4x on shared-prefix throughput | needs SGLang runtime | Yes | NO — same migration cost as vLLM |
| `torch.compile` + StaticCache (reduce-overhead) | 20-50% per-step latency | shape-change recompile; HF integration is finicky | Yes, supported on Qwen2 architectures | Yes; LogitsProcessor unchanged |
| HF StaticCache + manual prefix KV reuse | up to multi-x TTFT cut on agent loops | manual implementation; bookkeeping on KV slice/restore | Yes | Yes; KV reuse is upstream of logits |
| FlashAttention-3 | 1.5-2x over FA2 | Hopper (H100/H200) only | Architecturally yes, hardware no on 3090/4090 | Irrelevant on current ELAI hardware |
| KIVI 2-bit KV quant | 2.35-3.47x throughput, 2.6x mem | tuning-free; quality near-neutral on 7B | Yes | Yes; LogitsProcessor is post-attention |
| TurboQuant KV quant | 4-7x KV cache compression at near-zero loss | already on ELAI 32B roadmap | Yes | Yes; same KV-layer placement as KIVI |
| PagedAttention + continuous batching | 2-4x throughput | requires vLLM-style serving engine | Yes | NO direct HF path; full runtime migration |

Key implication for ELAI: the techniques that PRESERVE the LogitsProcessor chain (DoLa, ETC, steering) and the existing `model.generate` entry point are prompt-lookup, REST, lookahead, KIVI/TurboQuant, `torch.compile`+StaticCache, and HF prefix-KV reuse. Everything that requires vLLM/SGLang/TensorRT-LLM is a runtime swap and forfeits current steering/contrastive infra. This matches the existing CLAUDE.md note that DoLa is HARMFUL on 7B (-15% code quality, 4.7x slower) — the harm comes from DoLa itself, not the wiring, so we want to KEEP the LogitsProcessor API but stop applying DoLa on the 7B tier.

## Perceived-latency UX tricks

Doherty threshold (0.1 s feels instant, 1 s preserves flow, 10 s disengages) is the canonical anchor. For streaming chat: TTFT <200 ms feels snappy; >1.5 s feels broken regardless of subsequent throughput. The user's complaint ("letters come up slowly") is almost certainly NOT a tokens/sec problem — current Qwen2.5-Coder-7B NF4+FA2 on the remote 3090 should deliver well above human read rate (~5 words/sec ~= ~7 tok/sec for code). It's most likely:

1. SSE-chunk burstiness from the sidecar: chunks arrive in clumps (every 50-200 ms), so the eye sees pauses instead of a smooth stream. Vercel's `smoothStream` exists specifically for this. The fix is a frame-pacing buffer on the dashboard: enqueue chunks as they arrive, drain at a target rate (e.g. 40 chars/sec, user-tunable), and emit one char per `requestAnimationFrame` tick. Sub-second total latency increase, dramatic perceived improvement.
2. Per-char DOM updates causing layout thrash: every token resizes the message bubble and shifts content below. Stable-streaming UI recommendation: reserve final-size bubble height (skeleton-screen pattern), buffer partial markdown, render only when a structural boundary closes.
3. No "thinking" cue between submit and TTFT. A 200-800 ms gap with no visible state reads as "broken." A typing-indicator (or a "Qwen is thinking..." badge) bridges that.

Concrete dashboard recommendations:
- Skeleton bubble: pre-allocate a placeholder bubble with shimmer animation the instant the user submits. Pages with skeleton screens score higher on perceived speed AND ease of navigation (Bilgihan et al.).
- Frame-paced reveal: client-side buffer + RAF-tick draining at a per-user-configurable rate (default 60 chars/sec on code, 100 chars/sec on plain text). Optional "instant" mode for power users.
- Cursor sentinel: blinking caret at the end of the stream. Cheap signal that the stream is alive between tokens.
- Don't batch-reveal whole responses. Research on TTFT and on streaming-vs-non-streaming (40-60% perceived-faster with streaming at identical total latency) is unambiguous: streaming beats batch every time, smoothness beats raw rate.
- Soft-commit partial structures: render a code fence as soon as its opening token arrives, even if the body is empty. Lets the user start orienting on structure before content.
- Cancel button on the bubble. Even unused, its presence reduces felt latency because the user knows they can escape.

## Best techniques to adopt

Ranked by expected ELAI impact per integration cost.

1. Dashboard frame-paced rendering (smoothStream-style buffer)
   - Grounding: Vercel AI SDK 4.1 ships `smoothStream` because raw provider chunks are bursty (vercel.com/blog/ai-sdk-4-1); Smashing 2026-05 on stable streaming UI; nngroup/Doherty.
   - Integration hook: dashboard SSE consumer (likely in `crates/dashboard-v2`). Add a per-channel RAF-paced drain queue between the SSE event source and the message-bubble component.
   - Risk: very low. Adds ~50-200 ms total to perceived end-of-stream; user usually doesn't notice. Reversible (a single toggle).

2. Skeleton bubble + cursor sentinel
   - Grounding: ResearchGate 326858669 (skeleton screens); standard pattern in modern chat UIs.
   - Integration hook: chat-panel component on submit; placeholder bubble before first SSE chunk.
   - Risk: nil.

3. Prompt-lookup decoding (n-gram speculative) on the sidecar
   - Grounding: github.com/apoorvumang/prompt-lookup-decoding; HF transformers native (`prompt_lookup_num_tokens`); 2x-4x on input-grounded code tasks.
   - Integration hook: `server.py` `model.generate(...)` call — add `prompt_lookup_num_tokens=10` (or configurable). Keeps DoLa/ETC LogitsProcessor chain intact (they apply to the verification pass).
   - Risk: low on input-grounded tasks (code edits, refactor, summarization). On pure code-completion-from-scratch the n-gram hit rate is low and you pay a small overhead; gate it per request type. Lossless, output identical to greedy/sampling baseline.

4. HF prefix-KV reuse + StaticCache for the ELAI system prompt
   - Grounding: huggingface.co/docs/transformers/.../kv_cache (StaticCache); vLLM APC measurements (multi-x TTFT cut).
   - Integration hook: `server.py` — pre-compute KV for the immutable system prefix once at boot, slice/concat with per-request suffix on each `generate`. Use StaticCache to lock the shape so `torch.compile` paths stabilize.
   - Risk: medium. KV bookkeeping is fiddly and easy to get wrong (off-by-one breaks coherence silently). Needs determinism test. Plays nicely with `torch.compile`.

5. `torch.compile(model, mode="reduce-overhead")` with bucketed sequence lengths
   - Grounding: HF docs (perf_torch_compile.md) + mljourney 2024; 20-50% per-step latency cut at 7B-13B.
   - Integration hook: `server.py` model load. Compile once at boot with a fixed set of context-length buckets (e.g. 4k / 8k / 16k / 25k). Pin StaticCache.
   - Risk: medium. Recompile-on-shape-change can spike latency unpredictably; bucketing mitigates. Verify Qwen2.5 patch level supports `_attn_implementation="flash_attention_2"` under compile.

6. KIVI / TurboQuant 2.5-3.5 bit KV quant
   - Grounding: arXiv 2402.02750 (KIVI), arXiv 2504.19874 (TurboQuant).
   - Integration hook: KV cache layer in the sidecar. Already on the 32B roadmap; on 7B/12 GB it buys back context length (could lift 25k -> 35-40k).
   - Risk: medium. Quality regression on long-context code; needs A/B with the existing 25k pass@1 baseline before promotion.

7. EAGLE-3 (deferred to 32B/48 GB tier)
   - Grounding: arXiv 2503.01840; vLLM/SGLang/Vertex ship it; 4.1-6.5x at temp 0.
   - Integration hook: requires a trained draft head per target model; runs under vLLM in production. Means a partial migration off `model.generate` for that tier OR using HF's emerging assistive-decoding path.
   - Risk: high integration cost; only worth it once we're past the 12 GB tier and once we've stress-tested the LogitsProcessor compatibility against the speculative path.

## Anti-patterns

- Skeleton-of-Thought on coding: prompts the LLM to enumerate a skeleton, then expand each point in parallel. Breaks single-stream UX (multiple streams to stitch), corrupts DoLa contrast (different layer hidden states per branch), and code coherence suffers when each function is generated in isolation from its neighbors.
- Switching to vLLM/SGLang/TensorRT-LLM just for speed: forfeits the LogitsProcessor chain (DoLa, ETC, steering all live in the HF logits hook surface). The CLAUDE.md note specifies DoLa is harmful on 7B but the wiring itself is shared infra with ETC/steering. A runtime swap is a B-N-sized project, not a patch.
- MoE swaps (Mixtral-class): explicitly rejected for ELAI per CLAUDE.md ("MoE rejected... breaks DoLa/steering"). Don't revisit on speed grounds.
- Naive batched reveal (hold 100 chars, render at once): perceptually WORSE than smooth streaming. Research is consistent: streaming-at-identical-total-latency feels 40-60% faster than batch. Batched-reveal optimizes the wrong axis.
- Character-by-character at raw SSE chunk rate: this is the current pain. Chunks arrive in bursts so the eye sees clumps. Pace it (frame-pacing buffer), don't slow it.
- "Thinking..." spinner alone with no skeleton: nngroup-grade anti-pattern. Skeleton screens consistently beat spinners on perceived speed.
- Speculative decoding without lossless guarantees: any draft-then-resample where the verifier uses different LogitsProcessor settings than the draft will drift outputs. Verify acceptance is byte-equivalent.
- Speculative decoding on temp > 0 paths without tested accept-rates: EAGLE/Medusa speedup numbers above are at temp 0; sampling regimes drop acceptance rate. Need per-temperature benchmark before claiming the headline number.

## Open questions

1. Measured tokens/sec on the remote 3090 right now (`huihui-ai/Qwen2.5-Coder-3B-Instruct-abliterated`, NF4, SDPA, no DoLa, 8k prefill). Without this number, perceived-slowness fixes are blind. Experiment: a 200-token sustained-generation benchmark, log inter-chunk delta-ms histogram client-side; if p50 inter-chunk > 100 ms and p99 > 500 ms, frame-pacing is the right first fix.
2. Effective n-gram acceptance rate for prompt-lookup decoding on ELAI's actual workload mix (code-edit, refactor, scaffold, plan). Each task class has a different prompt-reuse profile. Experiment: instrument `model.generate` with acceptance counters under each class for one week; promote the technique only if pooled accept-rate >= 30%.
3. Does HF StaticCache + flash_attention_2 + `torch.compile(reduce-overhead)` actually fuse on Qwen2.5 in our py3.12/flash-attn 2.7.4 env? GitHub issue 30055 suggests historical breakage. Experiment: micro-benchmark eager vs StaticCache vs compiled-StaticCache on 1k/4k/16k prefill, three runs each, look for the 20-50% literature number.
4. KIVI 2-bit quality regression on long-context code review tasks. Experiment: run the existing 25k pass@1 baseline with KIVI 2-bit KV; if pass@1 stays within -2pp, promote and lift context to 35k as a stretch.
5. End-user perceived-latency study: ship the frame-pacing toggle behind a setting (default off vs default on, randomized per session), log self-reported "snappy/slow" votes. Even N=5 sessions of self-rating is enough to confirm the smoothness direction. Anchor to the streaming-vs-batch literature's 40-60% perceived-faster claim.
