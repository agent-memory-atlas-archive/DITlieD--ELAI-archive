# Inference engines for ELAI (May 2026)

## TL;DR
- KEEP the custom sidecar for now. vLLM V1 deprecated per-request custom logits processors and moved to batch-level global processors loaded at engine init. ELAI's DoLa + steering + TurboQuant stack does not cleanly map to that interface, and the only working precedent (EasySteer on vLLM v0.17.1) is "initial support" still missing CUDA graphs as of Mar 2026.
- MIGRATE the cloud sidecar path (RTX 3090 24GB pythondev) to vLLM 0.20 native-Windows or SGLang for the throughput tier. SGLang shows +29% total tok/s, +117% output tok/s, and 3x faster constrained decoding vs vLLM on H100, and SGLang ships a `CustomLogitProcessor` Python plugin with batch-aware execution that is closer to ELAI's needs than vLLM V1.
- ADOPT XGrammar via SGLang/vLLM for ELAI's structured outputs (planner JSON, tool calls). XGrammar enforces JSON/regex/EBNF at under 40 us/token and is the default backend in SGLang, vLLM, TRT-LLM, MLC-LLM as of Mar 2026 - this replaces any hand-rolled grammar code in the custom sidecar.
- ADD EAGLE-3 or n-gram prompt-lookup speculative decoding to the sidecar. 2.0x-6.5x tok/s in production reports, n-gram costs zero VRAM, EAGLE-3 needs draft head training but is the industrial default in 2026.
- DROP Ollama from any ELAI consideration. 10-30% raw overhead vs llama.cpp single-user, 19x throughput gap at concurrency (793 vs 41 tok/s peak), 1.2 GB extra RAM, earlier VRAM spillover halves perf on 12 GB cards.

## Key sources

| URL | what | takeaway |
|-----|------|----------|
| https://docs.vllm.ai/en/latest/usage/v1_guide/ | vLLM V1 user guide | V1 deprecates per-request logits processors; moving to global processors |
| https://developers.redhat.com/articles/2025/04/28/performance-boosts-vllm-081-switching-v1-engine | Red Hat vLLM 0.8.1 V1 | Zero-overhead APC in V1, unified scheduler, chunked prefills + prefix cache + spec decode |
| https://docs.vllm.ai/en/stable/features/custom_logitsprocs/ | vLLM custom logits processors | V1 requires LogitsProcessor subclass, batch-level apply(), validate_params, update_state |
| https://docs.vllm.ai/en/stable/design/prefix_caching/ | vLLM APC design | V1 block table append-only; prefix cache ignored if prompt logprobs requested |
| https://particula.tech/blog/sglang-vs-vllm-inference-engine-comparison | SGLang vs vLLM 2026 | H100 L3.1-8B: SGLang 16200 tok/s vs vLLM 12500 (+29%); 894 vs 413 output tok/s; 79 vs 103 ms TTFT |
| https://www.spheron.network/blog/vllm-vs-tensorrt-llm-vs-sglang-benchmarks/ | 3-way bench | TRT-LLM 15-30% higher peak throughput vs vLLM on H100; vLLM fastest TTFT |
| https://www.morphllm.com/comparisons/llama-cpp-vs-ollama | llama.cpp vs Ollama | Ollama 10-30% overhead single-user, 3x gap concurrent; vLLM 793 vs Ollama 41 tok/s peak |
| https://github.com/sgl-project/sglang/issues/2291 | SGLang custom logits PR | SGLang CustomLogitProcessor batch-aware Python plugin, registered per-request via `custom_logit_processor` |
| https://blog.mlc.ai/2026/05/04/xgrammar-2-fast-customizable-structured-generation | XGrammar-2 May 2026 | <40 us/token JSON enforcement; default in vLLM, SGLang, TRT-LLM, MLC-LLM |
| https://blog.premai.io/speculative-decoding-2-3x-faster-llm-inference-2026/ | Spec decoding 2026 | EAGLE-3 industrial default; vLLM+SGLang+TRT-LLM all ship it; 2.0-6.5x tok/s |
| https://www.syncsoft.ai/en/blog/speculative-decoding-eagle3-medusa-deepseek-mtp-chinese-chuhai-2026 | Spec decoding deep dive | EAGLE-3 de facto standard; DeepSeek MTP 1.8x at >80% acceptance |
| https://github.com/vllm-project/vllm/issues/20402 | vLLM compile time RFC | torch.compile warm-start ~8s after JIT cache; cold 30-90s |
| https://tensorfuse.io/docs/blogs/reducing_gpu_cold_start | vLLM cold start | CUDA graph capture dominates end-to-end cold start in vLLM |
| https://www.marktechpost.com/2024/12/10/hugging-face-releases-text-generation-inference-tgi-v3-0-13x-faster-than-vllm-on-long-prompts/ | TGI v3.0 | 13x faster than vLLM on 200K-token prompts; 3x tokens/GPU on L4 24GB |
| https://docs.vllm.ai/en/stable/serving/integrations/claude_code/ | vLLM Claude Code | vLLM ships /v1/messages Anthropic endpoint; ANTHROPIC_BASE_URL swap |
| https://docs.vllm.ai/en/latest/api/vllm/entrypoints/anthropic/ | vLLM anthropic entrypoint | vllm.entrypoints.anthropic.serving_messages is upstream code |
| https://arxiv.org/abs/2509.25175 | EasySteer | vLLM-based steering framework, 10.8-22.3x speedup vs prior; v0.17.1 support Mar 2026 |
| https://github.com/UKGovernmentBEIS/vllm-lens | vLLM-Lens | Residual-stream activations + steering vectors plugin for vLLM |
| https://github.com/SystemPanic/vllm-windows | vLLM Windows fork | Native Windows kernels + builds, no WSL |
| https://aphrodite.pygmalion.chat/spec-decoding/ngram/ | n-gram prompt lookup | No second model, zero extra VRAM, modest speedup |
| https://github.com/ggml-org/llama.cpp/discussions/20969 | llama.cpp TurboQuant | TurboQuant KV cache quant discussion (ELAI uses TQ today) |
| https://blog.exolabs.net/day-1/ | exo benchmarks | 1.8x on 2 devices, 3.2x on 4; Thunderbolt 5 RDMA |
| https://huggingface.co/docs/text-generation-inference/en/index | TGI docs | Zero-config mode; auto-tuned for hardware |
| https://corelab.tech/llmgpu/ | GPU buyer's guide Apr 2026 | RTX 3060 12GB tier model fit guide |
| https://kaitchup.substack.com/p/serving-exllamav3-with-tabbyapi-accuracy | ExLlamaV3 + tabbyAPI | EXL3 120 tok/s @ 98.5% FP16 MMLU; 40% less bandwidth via fused kernels |
| https://github.com/vllm-project/vllm/issues/21313 | Anthropic /v1/messages PR | Upstream Anthropic endpoint feature tracking |

## Engine landscape table

| engine | continuous batching | APC / prefix cache | logits processor | grammar / constrained dec | DoLa / steering compat | cold start | throughput note (Qwen-7B-NF4 class) | Windows? | citation |
|--------|---------------------|---------------------|------------------|----------------------------|------------------------|-----------|--------------------------------------|---------|----------|
| vLLM 0.20 V1 | yes, unified scheduler | yes, zero-overhead, append-only blocks | yes but BATCH-LEVEL, global, init-time only; AdapterLogitsProcessor for req-level | yes, XGrammar default | partial; EasySteer fork at v0.17.1 still adding CUDA graphs | 30-90s torch.compile; ~8s warm | ~700-800 tok/s peak agg on H100 8B class; lower on RTX 3060 | yes, native 0.20.0 Apr 2026 + WSL2 | docs.vllm.ai, redhat, EasySteer arXiv 2509.25175 |
| SGLang | yes | yes, RadixAttention radix tree, cross-request | yes, CustomLogitProcessor python plugin, batch-aware, per-request | yes, native + XGrammar | not native; needs custom processor | similar to vLLM | +29% total, +117% output vs vLLM @ H100 8B; 3.1x on DeepSeek V3 | NVIDIA primary, AMD growing | particula.tech, sgl-project/sglang #2291 |
| TGI v3.0 | yes | yes + 5us lookup overhead | limited; HF transformers-based hooks | yes | weak, transformer-internal access not exposed | similar | 13x vLLM on 200K prompts; 3x tokens/GPU L4 | docker only | marktechpost TGI 3.0 |
| llama.cpp | yes (server, multi-client) | yes (cache reuse) | yes via sampler chain (custom samplers C++) | yes (GBNF grammar built-in) | possible via samplers, no first-class API | seconds (load + warmup) | RTX 3060 ~42 tok/s 8B Q4_K_XL 16K; ELAI Qwen2.5-Coder-3B NF4 measured 25K ctx 100% pass | yes native | github ggml-org, llama.cpp #19480 |
| Ollama | yes | yes | wraps llama.cpp samplers | yes (via llama.cpp) | wraps llama.cpp; no exposed API | seconds | vLLM 793 vs Ollama 41 tok/s peak; -10-30% raw vs llama.cpp; 1.2GB extra RAM | yes | morphllm, codersera |
| ExLlamaV3 | basic (tabbyAPI) | basic | yes via filters | yes via filters | partial | seconds | 120 tok/s 8B EXL3 single-GPU; 2x4090 200 tok/s; EXL3 not raw NF4 - own quant | yes | kaitchup substack, johal.in |
| TensorRT-LLM | yes | yes | yes (CustomLogitsProcessor in NIM) | yes XGrammar | none documented | minutes, engine build per model+GPU | 15-30% higher peak vs vLLM H100; 1.8x on quantized Llama3 8B/70B | container only | spheron, NVIDIA NIM docs |
| Triton + TRT-LLM | yes (inflight) | yes | yes via TRT-LLM | yes | none | minutes | enterprise; warm pools mandatory | container | NVIDIA docs.nvidia.com |
| NVIDIA NIM | auto-picks vLLM/SGLang/TRT-LLM | auto | yes via NIM custom logits hook | yes | n/a | image pull + engine build | profile-dependent | container | docs.nvidia.com NIM custom logits |
| MLC-LLM (TVM) | basic | basic | yes via XGrammar | yes | none | TVM compile per arch | mobile NPU/GPU focus | yes | arxiv 2511.05502 |
| ExLlamaV2 (archived) | basic | basic | yes | yes | partial | seconds | superseded by V3 | yes | github turboderp-org |
| LM Studio | wraps llama.cpp / MLX | yes | wraps backend | yes | no | seconds | desktop UX, MLX on Apple | yes | contracollective, glukhov.org |
| Jan | extension-based | depends | depends | depends | no | seconds | offline-assistant UX | yes | dev.to synsun |
| LocalAI | wraps llama.cpp/vLLM/ExLlama | depends | wraps backend | yes | wraps | seconds | drop-in OpenAI API + many fmts | yes | glukhov.org |
| exo | sharded only | basic | sampler-pass | basic | no | seconds (per node) | 1.8x on 2 nodes, 3.2x on 4; TB5 RDMA | partial | blog.exolabs.net |
| Petals | volunteer swarm | n/a | basic | basic | no | network-dependent | 405B reachable, latency-bound | yes | petals.dev |
| HF transformers `.generate` | NO (single-stream) | NO | yes, mature `LogitsProcessor`; ELAI's current path | yes via processors | yes; full hidden-state access | seconds | A100 8B: 2-5 concurrent before latency degrades; vLLM 50-200+ on same HW | yes | medium yashwanths, f22labs |
| Custom sidecar (ELAI) | manual | manual | YES, full Python | YES via XGrammar drop-in | YES (DoLa, ETC, steering, TurboQuant) | seconds | matches transformers single-stream | yes via WSL/native | self |

## Custom sidecar vs vLLM/SGLang decision

### Keep custom sidecar - in favour
- ELAI's DoLa + ETC LogitsProcessors map to HF transformers' native LogitsProcessor API, which is the only stack that exposes full per-step access to logits AND hidden states on a per-request basis. vLLM V1 explicitly deprecated per-request logits processors and moved to batch-level global processors loaded at engine init (https://docs.vllm.ai/en/v0.8.1/getting_started/v1_user_guide.html).
- DoLa needs intermediate layer outputs (contrastive layer pair), not just final logits. vLLM's `apply(logits)` only sees the final logits tensor (https://docs.vllm.ai/en/stable/features/custom_logitsprocs/). DoLa as a vLLM custom processor is not implementable without forking the model runner.
- Steering vectors on vLLM exist (EasySteer, vLLM-Lens) but EasySteer is pinned to v0.17.1 with "initial support" status and still missing CUDA graphs as of Mar 2026 (https://arxiv.org/abs/2509.25175). For a production stack that's a hard hold-back.
- TurboQuant KV-cache quantization is being actively discussed in llama.cpp (ggml-org/llama.cpp#20969) but no vLLM/SGLang upstream support is documented. ELAI already runs TurboQuant in its own sidecar.
- ELAI ships /v1/messages Anthropic-compatible. vLLM has a `vllm.entrypoints.anthropic.serving_messages` upstream, but bug reports on multi-turn tool calling and image inputs are still open as of early 2026 (vllm#38738, vllm-ascend#6938).

### Migrate to vLLM/SGLang - in favour
- 2-4x throughput vs HF `.generate` at concurrent load, conservative number (https://blog.vllm.ai/2025/04/11/transformers-backend.html). Naive transformers FastAPI: 2-5 concurrent on A100; vLLM 50-200+.
- SGLang structured generation (XGrammar) is 3x faster than vLLM guided decoding at 96-98.2% compliance (particula.tech). Free win for ELAI's planner/critic JSON output.
- EAGLE-3, P-EAGLE, n-gram prompt-lookup all built-in. ELAI's custom sidecar would need to implement these manually (P-EAGLE merged in vLLM 0.16.0, https://github.com/vllm-project/vllm).
- APC / RadixAttention save real money on multi-turn cosmos-tier prompts and B-N briefings that share preambles. Sidecar today has no cross-request KV reuse.
- Cold-start cost of vLLM is real (30-90s torch.compile) but amortizable on long-running ELAI sidecars where the sidecar is up for hours; only matters on burst restart.

### Net call
HYBRID. Local (RTX 3060 12GB) tier stays on custom sidecar because DoLa + ETC + TurboQuant cannot be replicated in vLLM V1 today, and llama.cpp/ExLlamaV3 do not solve ELAI's research-engine needs. Cloud tier (RTX 3090 24GB pythondev) - run a parallel vLLM 0.20 native-Windows OR SGLang adapter behind B-37's RuntimeAdapter, gate by role: research/DoLa goes to custom sidecar, planner/critic/structured-JSON goes to SGLang+XGrammar.

## Speculative decoding support matrix

| stack | EAGLE-3 | Medusa | prompt-lookup / n-gram | DeepSeek MTP | suffix-decoding | citation |
|-------|---------|--------|-----------------------|--------------|------------------|----------|
| vLLM 0.20 V1 | yes (P-EAGLE merged 0.16.0) | partial / deprecated | yes (built-in) | yes | research only | premai blog, vllm-project/vllm#32887 |
| SGLang | yes (production) | partial | yes | yes (3.1x DeepSeek V3) | research only | premai, particula |
| TensorRT-LLM | yes (EAGLE-3 + CUDA graphs) | yes (Medusa heads) | basic | yes | no | spheron, NVIDIA |
| llama.cpp | partial (draft model) | no | yes (ngram via grammar samplers) | no | no | github ggml-org |
| HF transformers | yes (assistant_model api) | research code | yes (PromptLookupCandidateGenerator) | no | research | huggingface.co/docs |
| Custom sidecar (today) | not implemented | not implemented | not implemented | not implemented | no | self |

Production caveat: "speculative decoding into production often underperforms lab numbers by 40-60% due to workload characteristics like larger batch sizes, shorter outputs, and stricter output constraints" - tianpan.co Apr 2026.

## Cold-start latency comparison

| stack | typical cold start | dominated by | citation |
|-------|---------------------|--------------|----------|
| vLLM V1 (first-ever) | 30-90s | torch.compile graph capture | tensorfuse, vllm#20402 |
| vLLM V1 (warm JIT cache) | ~8s | CUDA graph replay + weight load | vllm#20402 |
| SGLang | similar to vLLM warm | CUDA graphs + RadixAttention init | sgl-project/sglang |
| TGI v3.0 | seconds | weight load + zero-config tuning | marktechpost TGI v3.0 |
| TensorRT-LLM | minutes (first build) + seconds (warm) | TRT engine build per (model, GPU, batch) | spheron, NVIDIA |
| Triton + TRT-LLM | minutes | engine cache + model repo load | docs.nvidia.com |
| llama.cpp | seconds | mmap weight load + KV alloc | github ggml-org |
| ExLlamaV3 | seconds | EXL3 weight load | kaitchup |
| Custom sidecar (ELAI now) | seconds | NF4 weight load + warmup token | self / measured |

ELAI implication: vLLM's torch.compile cold start kills "boot sidecar on demand" UX. Either keep sidecar warm-resident (current model), or use llama.cpp / custom-sidecar for the dev tier where instant cold-start matters.

## Best technique to adopt for ELAI

Ranked by ROI given ELAI's spec (Phase 3J, B-37 RuntimeAdapter):

1. **XGrammar drop-in for structured outputs**. Engine: any (vLLM, SGLang, TRT-LLM, custom). Grounding: blog.mlc.ai/2026/05/04, <40 us/token, default backend in 4 engines. Integration hook: replace whatever JSON-mode/regex logic ELAI's planner/critic uses today; wrap as a LogitsProcessor in the custom sidecar so behaviour is identical across runtimes. Drops latency on planner JSON to near-zero overhead.

2. **EAGLE-3 or P-EAGLE speculative decoding in custom sidecar**. Engine: custom sidecar (impl from EAGLE-3 paper). Grounding: blog.premai.io 2.0-6.5x, vllm#32887 P-EAGLE merged Sep 2025. Integration hook: new role-tier in RuntimeOrchestrator, "spec-decode" branch. Use n-gram prompt-lookup as zero-cost fallback (aphrodite docs, zero VRAM). Acceptance rate on Qwen2.5-Coder code completion is the ELAI-specific number we need to measure.

3. **SGLang as a new RuntimeAdapter backend (cloud tier)**. Engine: SGLang. Grounding: particula 16200 vs 12500 tok/s, 894 vs 413 output, 3.1x DeepSeek V3. Integration hook: B-37 RuntimeAdapter trait + new SglangBackend struct. Route research/DoLa traffic to custom sidecar, route planner/critic/structured-JSON traffic to SGLang.

4. **vLLM 0.20 native-Windows behind same adapter**. Grounding: dasroot.net + SystemPanic/vllm-windows. Use case: when WSL2 friction is unacceptable on a dev box but the user wants vLLM throughput. Cold-start cost remains; not a sidecar replacement for the 12 GB dev tier.

5. **EasySteer for activation-steering at scale (research path)**. Engine: vLLM + EasySteer. Grounding: arxiv 2509.25175 10.8-22.3x speedup, Mar 2026. Integration hook: experimental track only - EasySteer is at v0.17.1 / no CUDA graphs yet. Watch.

6. **TGI v3.0 for long-context cosmos-tier briefings**. Engine: TGI 3.0. Grounding: marktechpost 13x vLLM on 200K-token prompts, 3x tokens per GPU on L4. Integration hook: B-42 cosmos worker could send the cosmos-check briefing to TGI when prompt > 50K tokens, fall back to sidecar below. Container-only deploy.

7. **APC / RadixAttention on cross-request prefixes**. Engine: vLLM V1 or SGLang. Grounding: vllm V1 APC zero-overhead; SGLang up to 6.4x on prefix-heavy. Integration hook: any system-prompt that repeats per request (the persona prompt, the role-runtime prompt) becomes free past first hit. Already in scope if (3) lands.

8. **NIM as deploy format for the cloud sidecar**. Engine: NIM (wraps vLLM/SGLang/TRT-LLM). Grounding: NVIDIA NIM docs custom logits + Qwen models May 2026. Integration hook: long-term path if ELAI wants a "drop on any NVIDIA box" container with auto-selected backend, including in production with the second 3090.

Skip-list with grounding: Ollama (codersera 19x throughput gap), Petals (volunteer net unreliable, public exposure incompatible with private codebase), exo on TB5 (needs hardware ELAI does not have).

## Anti-patterns

- "Just swap to vLLM, it's faster." vLLM V1 deprecated per-request custom logits processors. ELAI's DoLa/ETC/steering would silently degrade or refuse to load. https://docs.vllm.ai/en/v0.8.1/getting_started/v1_user_guide.html.
- "Use Ollama for production multi-user." 19x peak-throughput gap, 1.2 GB extra RAM, earlier CPU spillover. https://www.morphllm.com/comparisons/llama-cpp-vs-ollama.
- "Roll our own speculative decoding from scratch." 2026 production reports show 40-60% drop vs lab numbers on first attempt; use EAGLE-3 paper or P-EAGLE upstream as the reference. https://tianpan.co/blog/2026-04-17-speculative-decoding-production-hidden-traps.
- "Implement DoLa as a vLLM batch-level processor." DoLa requires per-layer hidden states; vLLM's `apply(logits)` sees only final logits. Would require a fork of the model runner. https://docs.vllm.ai/en/stable/features/custom_logitsprocs/.
- "Trust training data on package names" - vLLM Windows native is a community fork (SystemPanic/vllm-windows) plus upstream Windows wheels as of 2026; do not assume `pip install vllm` works on Windows without verifying the build status. https://github.com/SystemPanic/vllm-windows.
- "Cold-start vLLM on demand per request." 30-90s torch.compile + CUDA graph capture. Always keep warm or use llama.cpp/sidecar for burst paths. https://tensorfuse.io/docs/blogs/reducing_gpu_cold_start.
- "EXL3 instead of NF4 to get 120 tok/s." EXL3 is its own quantization; throws out ELAI's existing NF4 calibration + TurboQuant chain. https://kaitchup.substack.com/p/serving-exllamav3-with-tabbyapi-accuracy.

## Open questions (measurable)

1. Measure Qwen2.5-Coder-7B NF4 tok/s on RTX 3060 12GB under (a) current custom sidecar transformers+bnb, (b) vLLM 0.20 native-Windows with BnB quantization, (c) llama.cpp Q4_K_M GGUF, (d) ExLlamaV3 EXL3. Numbers in the wild are 14B-class @ 22-29 tok/s on the same card; ELAI-specific 7B-NF4 needs first-party measurement.
2. Measure DoLa quality delta (truthfulness, factual-recall benchmark of choice) when implemented as (a) HF transformers LogitsProcessor in custom sidecar today vs (b) vLLM V1 batch-level processor with synthesized final-only logits (no intermediate layers). Is the degradation acceptable for a runtime swap, or does (b) lose the DoLa benefit entirely?
3. EAGLE-3 acceptance rate on Qwen2.5-Coder for ELAI's actual prompts (code-edit, planning JSON, cosmos briefings). The 2.0-6.5x range is workload-dependent; without measuring on ELAI's task mix, the speedup estimate is conjecture.
4. SGLang RadixAttention hit rate on ELAI's request stream. The 75-95% multi-turn / 85-95% few-shot numbers (particula.tech) are for chat / few-shot; ELAI's planner+critic+executor workflow may have different prefix-sharing characteristics.
5. vLLM 0.20 native-Windows stability on RTX 3060 specifically (compute capability 8.6, 12 GB). Most reported benchmarks use H100 / GB200. Need bring-up on dev hardware before any switch.
