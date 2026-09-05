DIGEST: p5-inference.md
SOURCE: e24faf0a-79e5-41aa-9197-f0573ac258a6.txt

TOPIC: Pillar P5 deep-dive — local LLM serving layer, HF Transformers vs alternatives in 2026. Scope: single-GPU 7B-class code model, consumer 24GB GPU (RTX 3090/4090), batch 1-4, autonomous agent workload, custom logits processors, deterministic decoding, NF4/4-bit, OpenAI/Anthropic-compatible SSE API.

REPORT VERDICTS:
- Overall = HARDEN: migrate core serving engine to vLLM V1, port logits processor chain in parallel. NOT a wholesale rewrite. (Note: this report prefers vLLM V1; the master-7pillars report preferred SGLang — divergence preserved.)
- HF Transformers = defensible but increasingly costly; uniquely strong only on unrestricted stateful Python LogitsProcessor composition at zero porting cost; every other axis (throughput, quant serving speed, KV reuse, reproducibility) better in vLLM V1 or SGLang.
- vLLM V1 = the migration target (1.8-24x HF throughput; first-class batch-level logits processors RFC merged; deterministic mode + batch invariance; APC built-in).
- SGLang = the alternative if logits processor chain cannot be ported to vLLM V1; RadixAttention up to 6.4x prefix-heavy; custom logits processors experimental (issue #2291, less mature than vLLM V1).
- llama.cpp = best fallback if vLLM V1 logits porting fails (instant cold start, GBNF grammar, Anthropic API, stable single-request ITL; throughput ceiling not penalizing for single-user desktop).
- TensorRT-LLM, candle, mistral.rs = NOT recommended for this config.
- KEEP Anthropic-Messages HTTP API shape (vLLM native OpenAI endpoint + thin adapter).
- REPLACE NF4/bitsandbytes with AWQ-Marlin for serving (offline AutoAWQ quant).

TOP 3 RANKED CHANGES: 1. Replace NF4/bitsandbytes with AWQ-Marlin in vLLM (5 stars, directly measured). 2. Enable vLLM Automatic Prefix Caching + VLLM_BATCH_INVARIANT=1 (4 stars, batch invariance beta). 3. Port logits processor chain to vLLM V1 batch-level API (3 stars, porting effort unverified for specific chain).

CANDIDATES:
HF Transformers | https://github.com/huggingface/transformers Apache 2.0 Production | Max logits-processor freedom; ~1.8x slower than vLLM batch-1; NF4 dequant serving penalty; GIL limits SSE concurrency | the audited backend (migrate away from serving)
vLLM V1 | https://github.com/vllm-project/vllm Apache 2.0 Production | 1.8-24x HF throughput; batch-level logits processors (RFC #17799 merged); VLLM_BATCH_INVARIANT=1 + seed determinism; APC built-in; cold start ~96s unoptimized; XGrammar backend; SamplingParams.extra_args per-request param | REPLACES HF serving (this report's primary)
SGLang | https://github.com/sgl-project/sglang Apache 2.0 Production | 29% higher throughput than vLLM on 7B-8B H100; RadixAttention up to 6.4x prefix-heavy; custom logits processors supported but experimental; XGrammar default (up to 10x faster JSON) | alternative if vLLM port fails
llama.cpp / llama-server | https://github.com/ggml-org/llama.cpp MIT Production | Fastest cold start (seconds vs ~96s vLLM); GGUF Q4_K_M ~92-94% quality vs FP16; flat throughput ceiling at concurrency>1 (44x lower TPS than vLLM at peak); GBNF grammar; Anthropic-Messages API; no first-class stateful Python logits processors | best fallback
mistral.rs | https://github.com/EricLBuehler/mistral.rs MIT Emerging | Rust-native; up to 2.79x faster than llama.cpp on Gemma 4 E4B Q8; CustomLogitsProcessor Rust trait; PyO3 Python bindings; Anthropic Messages endpoint; smaller community, immature AMD | NOT recommended (Rust port cost)
candle | https://github.com/huggingface/candle Apache 2.0 Experimental | HF Rust framework for serverless/edge not high-throughput; no vLLM-equivalent scheduling/KV management; WASM-capable | NOT recommended
TensorRT-LLM | https://github.com/NVIDIA/TensorRT-LLM Apache 2.0 Production (data-center) | Highest raw throughput (2,100 tok/s vs vLLM 1,850 at 50 req H100); 10-30 min engine compilation per model/GPU; custom logits engine-compiled not Python-callable at request time | NOT recommended (desktop sidecar)
XGrammar | no-url (https://developers.redhat.com/.../structured-outputs-vllm-... ; SGLang docs) | Structured-output backend already in vLLM; matches existing XGrammar dependency for tool-op grammar masking; --guided-decoding-backend xgrammar | KEEP, no behavioral change
AWQ + Marlin kernels | no-url (https://www.reddit.com/.../we_benchmarked_every_4bit_quantization_method_in/ ; https://www.index.dev/skill-vs-skill/ai-gptq-vs-awq-vs-gguf) | W4A16; ~95-99% FP16 accuracy; 712 tok/s vs BF16 461 tok/s same hw; offline AutoAWQ quant replaces runtime NF4 dequant | REPLACES NF4/bitsandbytes for serving
NF4 / bitsandbytes | no-url (https://github.com/bitsandbytes-foundation/bitsandbytes/releases/tag/0.45.0) | Weight-only quant w/ on-the-fly dequant; ~93-96% FP16; slower than FP16 batch-1; 0.45.0 (Dec 2024) +10-25% batch-1 on Turing/Ampere/Ada | the audited quant (move to QLoRA-only, not serving)
AutoAWQ | no-url | Offline W4A16 AWQ checkpoint production library | the quant tooling for #1 change
Transformers backend for vLLM | no-url (Ray Summit 2025 https://www.youtube.com/watch?v=LnXl5YR7dyA) | Run existing HF model code through vLLM Transformers backend, gain PagedAttention+continuous batching without touching logits processors; <3% throughput difference | alternative transition path
Qwen2.5-Coder-7B | no-url (https://www.promptquorum.com/local-llms/...) | ~45% SWE-Bench Verified, 7B, ~5GB Q4, 128K context | current model (superseded)
Qwen3-Coder 14B | no-url (https://www.index.dev/blog/qwen-ai-coding-review) | ~58% SWE-Bench, 14B, ~9GB Q4, 128K | RECOMMENDED highest-value drop-in upgrade (+13pp at ~9GB)
Qwen3-Coder 32B | no-url (https://failingfast.io/local-coding-ai-models/) | 72.9% Aider, 32B, ~20GB Q4, 128K; now feasible on 24GB | upgrade option (fits 24GB)
Qwen3-Coder-Next (80B/3B active) | no-url (https://www.marktechpost.com/2026/02/03/...) | 70.6% SWE-Bench, 3B active, ~46GB Q4 (needs CPU offload on 24GB), 256K; needs SGLang>=0.5.8 or vLLM>=0.15.0 | upgrade option (offload required)
Qwen3-Coder-480B-A35B | no-url (https://qwenlm.github.io/blog/qwen3-coder/) | SOTA open-source, 35B active, ~290GB Q4, 256K | reference ceiling (not local)

ATTACK-THE-DESIGN (strongest criticisms of current HF Transformers + NF4 setup):
- NF4 dequant penalty: weight-only quant dequants 4-bit->BF16 every forward pass; bitsandbytes INT8 4x slower than FP16 on L4; llama-7B NF4 12.2 tok/s vs ~28 tok/s AWQ W4A16; using NF4 in HF serving sacrifices 30-60% throughput for VRAM, no latency benefit at batch 1-4.
- Python GIL: model.generate() single Python thread; concurrent second request waits on GIL; no async scheduler like vLLM continuous batching; concurrent requests serialized -> head-of-line blocking.
- No automatic prefix caching: every agent turn re-prefills shared system prompt + tool schema (2,000-8,000 tokens) from scratch; compounding latency per request.
- Stateful LogitsProcessor duplication bug: HF raises ValueError on duplicate processor instances; contrastive-decoding entropy state + prompt-lookup state vulnerable if calling code reuses LogitsProcessorList object (analogous to vLLM #5329).
- Determinism fragile: greedy deterministic only w/ full stack fixed + torch.use_deterministic_algorithms(True), no bit-identical across PyTorch versions/GPU SKUs; sampling needs seeding Python/NumPy/PyTorch separately each call; no batch-invariance (adding a request changes CUDA graph shape, changes outputs for all in batch).
- KV memory headroom: dynamic KV allocation (grows per token, not pre-allocated); 32K 7B BF16 KV 4-8GB; no fragmentation management; one long request blocks VRAM for short requests; vLLM PagedAttention <4% waste vs 60-80% without paging.
- Throughput gap material even at batch 1-4: vLLM ~25 TPS vs HF ~14 TPS (1.8x) single-request Qwen2.5-7B; up to 24x at higher concurrency; RTX 4090 vLLM W4A16 1,328 tok/s vs 929 BF16; compounds over hundreds of tool calls.

NUMBERS (claimed source, unverified):
- vLLM 1.8-24x HF throughput [PagedAttention/LinkedIn benchmarks].
- vLLM ~25 TPS vs HF ~14 TPS single-request Qwen2.5-7B [LinkedIn controlled test].
- RTX 4090: vLLM W4A16 1,328 tok/s vs BF16 929 tok/s [Arcee/medium].
- SGLang 29% higher throughput vs vLLM 7B-8B H100; RadixAttention up to 6.4x prefix-heavy; XGrammar up to 10x JSON [TECHSY/LMSYS/LinkedIn].
- llama.cpp 44x lower TPS than vLLM at peak; cold start seconds vs ~96s vLLM [RedHat].
- TensorRT-LLM 2,100 tok/s vs vLLM 1,850 at 50 req H100; 10-30 min engine compile [Spheron/clore].
- mistral.rs up to 2.79x faster than llama.cpp Gemma 4 E4B Q8 (LinkedIn, indicative); Metal v0.5.0 37.94 tok/s vs llama.cpp 39.20 Llama 3.1 8B 8-bit [HF blog].
- NF4: bitsandbytes INT8 4x slower than FP16 on L4; llama-7B NF4 12.2 tok/s vs ~28 AWQ W4A16; AWQ Marlin 712 tok/s vs bitsandbytes GPTQ-no-Marlin 276 tok/s; bitsandbytes 0.45.0 +10-25% batch-1 [LinkedIn/Reddit/bitsandbytes].
- AWQ 4-bit ~95-99% FP16; AWQ Marlin 712 tok/s vs BF16 461 [index.dev/Reddit].
- vLLM PagedAttention <4% KV waste vs 60-80% without paging [Runpod].
- Quant quality table: AWQ 4-bit ~95-99%; GPTQ 4-bit Marlin ~90-95%; GGUF Q4_K_M ~92-94%; NF4 ~93-96%; Llama 3.1 4-bit accuracy delta +/-1pp MMLU/ARC across AWQ/AutoRound/GPTQ; AWQ perplexity +0.5-2% over FP16 WikiText-2.
- vLLM cold start: ~96s unoptimized, ~14s w/ GPU snapshot; RTX 5090 35B ~4.6 min; 7B weight load ~18s, expect 60-90s on RTX 3090/4090 [Modal/byteiota].
- Coder models: Qwen2.5-Coder-7B ~45%; Qwen3-Coder 14B ~58% (+13pp); Qwen3-Coder 32B 72.9% Aider; Qwen3-Coder-Next 70.6% SWE-Bench [various].
- vLLM batch invariance needs compute cap >=8.0 (RTX 3090=8.6, RTX 4090=8.9).
- vLLM prefix-cache head-of-line: up to 11-second TTFT bursty traffic RTX 4090 [ingero.io].

REPORT'S OWN FLAGS: SGLang custom logits processor exact porting effort for a stateful chain unverified (issue #2291 less mature than vLLM RFC); mistral.rs v0.8.2 2.79x claim is author LinkedIn post (indicative not independent); vLLM batch invariance labeled beta as of June 2026 (track Issue #27433); vLLM cold-start figures from Modal production data; logits-processor port effort for the specific contrastive-decoding + entropy + prompt-lookup chain unverified.
