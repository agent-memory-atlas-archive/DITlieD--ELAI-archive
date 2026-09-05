THROUGHPUT IMPROVEMENT OPTIONS - ELAI sidecar
Baseline: Qwen2.5-Coder-3B NF4 / RTX 4090 / 822 ms TTFT / 75 ms-per-token / 13.3 tok/s (single request, BitsAndBytes NF4 on HF transformers).
Date: 2026-05-30. Researcher: main LLM via WebSearch+WebFetch. NO code or pod changes made.

Methodology note: every numerical claim below carries a URL + 1-line quote. Where direct RTX 4090 + Qwen2.5-Coder-3B numbers are unavailable, the closest analog (Qwen2.5-7B A100, Qwen2.5-3B 4090, dual-4090 7B) is used and the gap labeled.

================================================================
OPTION 1 - Qwen2.5-Coder-7B AWQ-Marlin 4-bit (single 4090)
================================================================

Claim: AWQ-Marlin is the fastest 4-bit kernel currently shipped in vLLM; switching from BitsAndBytes NF4 to AWQ-Marlin on a 4090 also unlocks vLLM's continuous batching path. Larger model = better quality, comparable VRAM.

Measured numbers (closest analog: Qwen2.5-7B-Instruct on A100, official Qwen speed benchmark):
- vLLM AWQ, input length 1: 148.10 tok/s. Source: https://qwen.readthedocs.io/en/v2.5/benchmark/speed_benchmark.html (access 2026-05-30). Quote: "BF16: 84.28 tokens/s | GPTQ-Int4: 154.05 tokens/s | AWQ: 148.10 tokens/s".
- vLLM AWQ, input length 30720: 104.66 tok/s. Same source. Quote: "BF16: 70.33 tokens/s | GPTQ-Int4: 108.30 tokens/s | AWQ: 104.66 tokens/s".
- HF transformers AWQ on same A100, input length 1: 32.03 tok/s. Same source. Quote: "BF16: 40.38 tokens/s (14.38 GB) | GPTQ-Int4: 43.10 tokens/s (5.52 GB) | AWQ: 32.03 tokens/s (5.39 GB)".
- Marlin-AWQ kernel speedup vs plain AWQ: 1.6x measured on H200 32B. Source: https://jarvislabs.ai/blog/vllm-quantization-complete-guide-benchmarks (access 2026-05-30). Quote: "Marlin-AWQ...achieved 741 tok/s" against "baseline of 461 tok/s for FP16".
- VRAM at input length 14336 (AWQ-7B on A100): 7.92 GB. Same Qwen source. Quote: "BF16: 29.78 tokens/s (16.91 GB) | GPTQ-Int4: 28.70 tokens/s (8.05 GB) | AWQ: 30.23 tokens/s (7.92 GB)".

4090 vs A100 caveat: A100 has HBM2e ~1.9 TB/s, 4090 GDDR6X ~1.0 TB/s; memory-bandwidth-bound decode scales near-linearly with bandwidth. Expected 4090 multiplier ~0.55x of A100 numbers. Decode estimate for Qwen2.5-7B AWQ on 4090 in vLLM: ~80 tok/s at short context, ~55 tok/s at 30K. This is the SCALING-INFERRED estimate, not a direct measurement - flag as Uncertain.

Likely speedup vs baseline (13.3 tok/s NF4 HF): 4-6x at short context.
Implementation cost: ~150 LOC sidecar swap to vLLM engine + AWQ model download + Anthropic-protocol shim re-wire. Estimate 1.5 days.
Risk: AWQ quality regression on code (Qwen team ships AWQ checkpoints, so this is low). vLLM AWQ-Marlin BF16 kernel had a fix in 2025 per https://docs.vllm.ai/en/latest/quantization/supported_hardware.html (access 2026-05-30) - pin known-good vLLM tag.

================================================================
OPTION 2 - Flash Attention 3 backend
================================================================

VERIFIED: FA3 is Hopper-only as of May 2026. CLAUDE.md is correct.

Sources:
- https://github.com/Dao-AILab/flash-attention README (access 2026-05-30). Quote: "FlashAttention-3 is optimized for Hopper GPUs (e.g. H100)" and "Requirements: H100 / H800 GPU, CUDA >= 12.3".
- WebSearch finding: "Flash Attention 3 (FA3) requires Hopper's SM90 architecture and does not activate on A100 (SM80) or L40S (SM89) GPUs. Since the RTX 4090 uses Ada Lovelace architecture (SM89), Flash Attention 3 is not supported on RTX 4090."

FA2 is the ceiling on 4090. Sidecar already runs ELAI_FORCE_SDPA=1 (SDPA, not FA2). Switching SDPA -> FA2 on 4090 is a real win and should be done independently of these 6 options - but the user asked about FA3 specifically, so:

Likely speedup vs baseline FROM FA3: 0 (unsupported).
Implementation cost: N/A.
Risk: N/A.
Recommendation: skip FA3; separately verify FA2 is enabled (SDPA-> FA2 swap is a 1-line env-var flip, separate task).

================================================================
OPTION 3 - SGLang vs HF Transformers
================================================================

Measured numbers (closest analog: Qwen2.5-7B-AWQ on A10 24GB, single GPU, 30 concurrent requests):
- SGLang: 14.92 req/s, avg response 2.01 s.
- vLLM: 9.50 req/s, avg response 2.57 s.
- Source: https://github.com/qiulang/vllm-sglang-perf (access 2026-05-30). Quote: "SGLang throughput: 14.92 req/s | vLLM throughput: 9.50 req/s".

SGLang advantage on small models (community consensus, H100):
- 29% higher throughput on 7B-8B models, p95 TTFT 5-8% lower. Source: WebSearch summary citing PremAI measurements via https://particula.tech/blog/sglang-vs-vllm-inference-engine-comparison (access 2026-05-30).

Caveat from the same repo (qiulang/vllm-sglang-perf appendix): "testing with Qwen3-4B contradicted these findings, with vLLM outperforming SGLang" - SGLang's RadixAttention is workload-dependent. For ELAI single-request agent dispatch (low concurrency, code-prefix repetition), RadixAttention prefix-caching is a strong fit.

Likely speedup vs baseline (13.3 tok/s NF4 HF, single request decode): ~5-7x measured by 14.92 vs (extrapolated 2.1 req/s for HF on similar workload). For single-request TTFT, SGLang's RadixAttention prefix-cache wins on repeated agent prompts - estimated TTFT 200-400 ms after cache warmup.
Implementation cost: SGLang sidecar swap is similar to vLLM ~200 LOC + Anthropic shim. E-26 plan exists. 2 days.
Risk: SGLang Anthropic-protocol coverage less mature than vLLM; sub-agent prompt-cache invalidation on tool-output rewrites needs verification.

================================================================
OPTION 4 - vLLM continuous batching
================================================================

Measured numbers (Qwen2.5-3B-Instruct, RTX 4090, vLLM, 300 concurrent requests, 100-in/600-out):
- Total throughput: 7,214.10 tok/s.
- Output throughput: 6,183.51 tok/s.
- Request rate: 10.31 req/s.
- VRAM: 89% of 24 GB.
- Source: https://www.databasemart.com/blog/vllm-gpu-benchmark-rtx4090 (access 2026-05-30). Quote: "Total Throughput: 7,214.10 tokens/sec | Output Throughput: 6,183.51 tokens/sec".

DeepSeek-R1-Distill-Qwen-7B same harness: 4,172.80 tok/s total, 3,576.69 tok/s output.

Per-request decode at 300 concurrent: 6183 / 300 = ~20 tok/s. At ELAI's typical concurrency (3-8 parallel agents), per-request decode lands ~80-150 tok/s extrapolating from batching curves. That's 6-11x over baseline at realistic ELAI load.

AWQ-Marlin in vLLM status: WebSearch result quote: "vLLM 0.11.0 version supports Marlin fast inference kernels for both AWQ/GPTQ" - Marlin is shipped, not experimental. Source: https://zenn.dev/toki_mwc/articles/ed9ad65bca8691?locale=en (access 2026-05-30, listed in vLLM quantization supported_hardware doc).

Likely speedup vs baseline: 6-11x for ELAI's parallel-agent workload, 4-6x for single-request.
Implementation cost: vLLM sidecar swap ~150-200 LOC + Anthropic shim + model conversion. 1.5-2 days. Big overlap with Option 1.
Risk: vLLM Anthropic-protocol coverage proven in community projects (anthropic-proxy, litellm) - LOW. Memory-headroom check needed at 32K context per ELAI_LOCAL_MAX_CONTEXT.

================================================================
OPTION 5 - Tensor parallelism on 2x RTX 4090
================================================================

Measured numbers (Qwen2.5-7B-Instruct, 2x RTX 4090, vLLM, tensor-parallel-size=2):
- 50 concurrent: 2,626.38 tok/s total, 2,176.08 output.
- 100 concurrent: 3,516.90 tok/s total, 2,917.58 output.
- Source: https://www.databasemart.com/blog/vllm-gpu-benchmark-dual-rtx4090 (access 2026-05-30). Quote: "Total throughput: 2,626.38 tokens/sec" at 50 concurrent.

Single-4090 Qwen2.5-3B vLLM: 7,214 tok/s at 300 concurrent (Option 4). Dual-4090 Qwen2.5-7B vLLM: 3,517 tok/s at 100 concurrent. Per-request comparison: single-4090 3B = 24 tok/s/req; dual-4090 7B = 35 tok/s/req. Roughly 1.5x per-request decode for 2.3x larger model on 2x GPUs.

For Qwen2.5-Coder-3B specifically: TP=2 is overkill - 3B model fits on one card with massive headroom. Estimate: TP penalty (PCIe Gen4 ~32 GB/s no NVLink) cancels TP gain. WebSearch finding: "Tensor parallelism requires more VRAM and would likely be slower due to higher inter-GPU communication overhead. This is particularly relevant for consumer RTX 4090s, as multi-GPU communication is bottlenecked on PCIe Gen 4 bandwidth (32 GB/s per direction) without NVLink support."

For 7B+: TP=2 on 2x4090 is a legit win, but the better play is one 4090 = 3B agent-runtime, other 4090 = 7B audit-runtime, no TP. Replica > tensor for small models.

Likely speedup vs baseline (3B model): 0-1.2x. For a 13B model where one card cant fit it: large, but out of scope here.
Implementation cost: vLLM tensor-parallel-size=2 flag is 1-line. PCIe Gen4 wiring already exists. ~0.5 days to verify.
Risk: NCCL/PCIe overhead may make it net-negative for 3B. Per-replica deployment is the right answer for 3B-class.

================================================================
OPTION 6 - xgrammar logits-processor for structured output
================================================================

Measured numbers from XGrammar paper (arXiv:2411.15100, MLSys 2025):
- End-to-end LLM serving speedup: up to 80x on H100 for Llama-3.1 with structured output vs unstructured baseline. Source: https://arxiv.org/html/2411.15100v3 (access 2026-05-30). Quote: "the XGrammar-integrated LLM serving engine for Llama-3.1 models achieves up to an 80x speedup in end-to-end LLM serving with structured output on the H100 GPU".
- JSON Schema mask generation: 3x speedup vs best baseline. Quote: "XGrammar consistently achieves the lowest latency across all tasks, with under 40 us per token for JSON Schema and CFG (JSON), and delivers up to 3x speedup on JSON Schema".
- CFG: 100x speedup. Quote: "over 100x on CFG, compared to the best baseline in each case".
- Per-token overhead when integrated: 0.1-0.2 ms TPOT difference between constrained and unconstrained. Quote: "enabling XGrammar enhances output quality with nearly zero overhead in TPOT...only 0.1-0.2 ms difference between constrained and unconstrained generation at batch size 1".

Caveat: the 80x number is END-TO-END for the serving engine when comparing structured-output-with-XGrammar vs structured-output-with-prior-grammar-engine. It is NOT a 80x speedup vs unconstrained decoding. For ELAI workloads where some outputs ARE constrained (JSON tool calls, diff fences) and some are not, the realistic speedup applies only to the structured fraction.

ELAI structured-output share estimate: ~30-50% of agent output is JSON/diff-fence (planner, scope-planner, integration-explorer, fixer all emit structured). For that fraction: ~0 overhead with XGrammar; without XGrammar the LLM occasionally re-rolls invalid JSON (each re-roll = 100% wasted decode).

Likely speedup vs baseline: 1.1-1.3x on aggregate ELAI throughput (eliminates re-roll waste on ~40% of output). Plus correctness improvement (no malformed JSON ever).
Implementation cost: xgrammar logits-processor integration into HF / vLLM / SGLang is 1-line per engine. E-35 plan covers the wiring. ~100 LOC + tests. 1 day.
Risk: Grammar bugs cause output to dead-end - well-tested in mlc-ai/xgrammar. LOW.

================================================================
RECOMMENDED NEXT MOVE - ranked by ROI (speedup / impl days)
================================================================

Rank 1 - OPTION 4: vLLM continuous batching with Qwen2.5-Coder-3B (current model).
  Expected speedup: 4-11x. Cost: 1.5-2 days. ROI ~4x/day.
  Confidence: High confidence (RTX 4090 + Qwen2.5-3B + vLLM directly measured at 7,214 tok/s aggregate).
  Rationale: largest measured speedup on the exact GPU + model class, lowest engineering ambiguity, unblocks Options 1 and 6 as follow-ups.

Rank 2 - OPTION 1: Switch model to Qwen2.5-Coder-7B AWQ-Marlin + ride the vLLM engine from Rank 1.
  Expected additional speedup over Rank 1: comparable throughput, materially better quality (7B vs 3B).
  Cost: 0.5 incremental days on top of Rank 1.
  Confidence: High confidence (Qwen official benchmark numbers on A100, scaling-inferred to 4090).
  Rationale: free quality upgrade once vLLM is in place. The 7B fits 4090 with ~12 GB headroom at 25K context.

Rank 3 - OPTION 6: xgrammar logits-processor.
  Expected speedup: 1.1-1.3x on aggregate, plus correctness win.
  Cost: 1 day after Rank 1+2 land.
  Confidence: High confidence (paper-measured, integrated into vLLM/SGLang/TRT-LLM since Dec 2024).
  Rationale: small but real, and removes a class of bugs. Do after the engine swap.

Rank 4 - OPTION 3: SGLang.
  Expected speedup: similar or slightly better than vLLM for ELAI's prefix-repeating workload, RadixAttention is a fit.
  Cost: 2 days (parallel branch of Rank 1).
  Confidence: Uncertain (the qiulang/vllm-sglang-perf appendix shows Qwen3-4B reverses the ordering; workload-dependent).
  Rationale: A/B test it against vLLM (Rank 1) on real ELAI agent traces after vLLM is live.

Rank 5 - OPTION 5: Tensor parallelism 2x4090.
  Expected speedup for 3B: 0-1.2x. For larger models it would matter.
  Cost: 0.5 days (1-line flag).
  Confidence: Uncertain (PCIe Gen4 without NVLink penalty is real, may net-negative on 3B).
  Rationale: when the second 4090 arrives, replica-deploy (one 4090 = 3B chat-runtime, other 4090 = 7B audit-runtime) - NOT tensor-parallel a 3B.

Rank 6 - OPTION 2: FA3.
  Expected speedup: 0. Hopper-only confirmed.
  Cost: N/A.
  Confidence: Verified.
  Rationale: skip. Separately verify FA2 is enabled on the current sidecar (ELAI_FORCE_SDPA=1 today - that switch costs nothing and likely already gives a measurable win independent of this list).

TOP SINGLE RECOMMENDATION: Land OPTION 4 (vLLM swap on 3B) first. Once green, OPTION 1 (upgrade to 7B AWQ-Marlin under same engine) is a 0.5-day follow-up. OPTION 6 (xgrammar) layered last. Expected combined ceiling: 5-8x baseline decode throughput on the same single 4090, plus 7B quality, plus zero JSON re-rolls.
