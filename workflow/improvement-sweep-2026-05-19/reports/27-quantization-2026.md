# Quantization for ELAI (2024-2026)

## TL;DR
- ELAI is currently on bnb-NF4 + FP16 compute for Qwen2.5-Coder-7B. On Qwen2.5-32B Wikitext-2 perplexity, BnB actually wins quality (6.67 vs FP16 6.56) but throughput is 168 tok/s vs Marlin-AWQ's 741 tok/s — a 4.4x speed gap (Jarvislabs, 2026). NF4 is best for fine-tuning, not for serving.
- SWITCH RECOMMENDATION: move serving to AWQ + Marlin kernel (W4A16). HumanEval pass@1 is the same as BnB (51.8 vs 51.8), perplexity slightly worse (6.84), throughput 4.4x faster. Keep NF4 only if you need QLoRA fine-tuning on the same checkpoint.
- For reasoning safety: arxiv 2504.04823 (COLM 2025) explicitly names W4A16 and W8A8 as the lossless ceilings — sub-W4 or aggressive activation quantization introduces real reasoning loss. ELAI's 7B + reasoning workload should stay >= W4A16.
- Long-context tax: at >64K context, BNB-NF4 can drop up to 59% on long-context tasks (arxiv 2505.20276). Qwen2.5-72B is robust under NF4, but smaller Qwens are not — ELAI's 7B at 25K is still inside the safer window, but pushing to 40K with NF4 is risky.
- KV cache: FP8 KV is the production default in vLLM 2026 (~2x context for free, no throughput loss). TurboQuant DEFER stays correct — vLLM's own May 2026 study says FP8 beats every TurboQuant variant on accuracy AND throughput; k3v4-nc drops ~20 reasoning points.

## Key sources
| URL | What | Takeaway |
|---|---|---|
| arxiv.org/abs/2504.04823 (COLM 2025) | Quantization Hurts Reasoning empirical study | W8A8/W4A16 are lossless ceilings; sub-W4 is risky |
| arxiv.org/abs/2505.20276 | Long-context quantization study | 4-bit drops up to 59% at >64K context; 8-bit ~0.8% |
| jarvislabs.ai/blog/vllm-quantization-complete-guide-benchmarks | Qwen2.5-32B 4 formats head-to-head | Marlin-AWQ 741 tok/s vs BnB 168 tok/s, same HumanEval |
| vllm.ai/blog/turboquant | vLLM's own TurboQuant benchmark | FP8 beats TurboQuant on accuracy AND throughput |
| github.com/turboderp-org/exllamav3 | EXL3 quantization library | QTIP-based, trellis coding, Hadamard + LDL |
| developer.nvidia.com/blog/introducing-nvfp4-for-efficient-and-accurate-low-precision-inference | NVFP4 official | 16-element group vs MXFP4's 32; better fine-grain scaling |
| arxiv.org/abs/2603.08747 | NVFP4/MXFP4 layer sensitivity | MLP up/down most FP4-sensitive; MXFP4 ~15% faster on B200 |
| aclanthology.org/2025.findings-acl.952 (MiniKV) | 2-bit KV cache | Push KV to 2-bit with selective coverage |
| docs.vllm.ai/en/latest/features/quantization/fp8/ | vLLM FP8 W8A8 doc | Hopper/Ada native FP8; ~2x mem, ~1.6x throughput |
| github.com/IST-DASLab/marlin + arxiv 2408.11743 | MARLIN W4A16 kernel | 10.9x AWQ speedup, near-ideal 4x up to batch 16-32 |
| developers.redhat.com/articles/2024/10/14/introducing-machete-mixed-input-gemm-kernel | Machete (Hopper successor to Marlin) | Hopper-tuned, in vLLM 0.6.2+ |
| arxiv.org/abs/2504.19874 (ICLR 2026) | TurboQuant paper | KV compression 4-7x; vLLM measured it as worse than FP8 |
| willitrunai.com/blog/quantization-guide-gguf-explained | GGUF K-quants quality tiering | Q4_K_M moderate loss noticeable on code; Q5_K_M sweet spot; Q6_K near-lossless |
| spheron.network/blog/fp4-quantization-blackwell-gpu-cost | Blackwell FP4 economics | FP4 matters only on B100/B200/RTX50 with native HW |
| dropbox.tech/machine-learning/halfquadratic-quantization-of-large-machine-learning-models | HQQ blog (Mobius/Dropbox) | Calibration-free, 50x faster to quantize than GPTQ |
| qwen.readthedocs.io/en/v2.5/benchmark/quantization_benchmark.html | Qwen official quant numbers | Qwen-native AWQ/GPTQ-int4/GPTQ-int8 baselines |

## Format comparison
| Format | Bits | Quality loss (Qwen2.5-32B PPL vs FP16=6.56) | Speed (tok/s) | HW support | Works on Qwen2.5-Coder-7B | Citation |
|---|---|---|---|---|---|---|
| FP16 baseline | 16 | 6.56 | 461 | All | Yes (~14 GB, doesn't fit 12GB) | Jarvislabs |
| BnB NF4 | 4 (W4A16) | 6.67 (best 4-bit PPL) | 168 | All CUDA | Yes (~5.2 GB, ELAI today) | Jarvislabs / QLoRA paper |
| AWQ (no Marlin) | 4 (W4A16) | 6.84 | 68 | All CUDA | Yes | Jarvislabs |
| Marlin-AWQ | 4 (W4A16) | 6.84 | 741 (10.9x AWQ) | Ampere+ | Yes — recommended | arxiv 2408.11743 |
| GPTQ | 4 (W4A16) | 6.90 | 277 | All CUDA | Yes | Jarvislabs |
| Marlin-GPTQ | 4 (W4A16) | 6.97 | 712 | Ampere+ | Yes | Jarvislabs |
| GGUF Q4_K_M | ~4.5 | 6.74 | 93 (llama.cpp CPU+GPU offload) | All | Yes | Jarvislabs |
| GGUF Q5_K_M | ~5.5 | near 6.56 | slower | All | Yes | willitrunai |
| GGUF Q6_K | ~6.5 | near-lossless | slower | All | Yes | willitrunai |
| EXL3 | 1-8 bpw variable | KL-divergence near baseline at 4 bpw, coherent at 1.6 bpw on 70B | Tensor-core optimized, faster than AWQ on RTX 50 | Ampere+, best on Hopper/Blackwell | Yes | turboderp-org/exllamav3 |
| FP8 (E4M3/E5M2) | 8 (W8A8) | "Lossless" per arxiv 2504.04823 | ~1.6x FP16 native | Hopper/Ada/B200 | No (no H100 in ELAI VRAM tiers <48GB) | docs.vllm.ai/fp8 |
| MXFP8 | 8 + E8M0 scale | Near-FP8 | Blackwell native | B200+ | Future tier | OCP MX spec |
| MXFP6 | 6 + E8M0 scale | Mid | Blackwell native | B200+ | Future tier | OCP MX spec |
| MXFP4 | 4 + E8M0 scale (group 32) | "Non-negligible PTQ drop on small LLMs" | B200 +15% over NVFP4 | B200+ | Future tier | arxiv 2603.08747 |
| NVFP4 | 4 + scale (group 16) | Better than MXFP4 PTQ; NVIDIA's QAD recovers near-FP8 | Lower throughput than MXFP4 on B200 | B200+ | Future tier | NVIDIA NVFP4 blog |
| HQQ | 4/3/2 | Slightly worse than AWQ at 4-bit; calibration-free | varies (Marlin-compatible via 4-bit HQQ) | All | Yes | Dropbox HQQ blog |

## NF4 vs AWQ vs GPTQ vs Exl3 vs MXFP4 (May 2026 reality)

Pareto-frontier reality, Qwen2.5-32B on H100 (Jarvislabs Jan 2026):
- Quality (PPL, lower=better): BnB-NF4 (6.67) > GGUF-Q4_K_M (6.74) > AWQ/Marlin-AWQ (6.84) > GPTQ (6.90) > Marlin-GPTQ (6.97).
- HumanEval pass@1: FP16 56.1% -> AWQ/Marlin-AWQ/GGUF/BnB all 51.8%, GPTQ 46.3%. NF4 ties AWQ on code despite tiny PPL win.
- Throughput: Marlin-AWQ 741 t/s > Marlin-GPTQ 712 > FP16 461 > GPTQ 277 > BnB 168 > GGUF 93 > AWQ-no-kernel 68. Marlin is the multiplier — 10.9x AWQ uplift on identical weights.
- EXL3 on Blackwell: "~200 t/s on 70B" cited in turboderp's Blackwell notes (qualitative — author claim, not third-party measured). At low-bit targets (~2.5 bpw) EXL3 significantly outperforms uniform 4-bit methods on KL-divergence.
- MXFP4 vs NVFP4 on B200: MXFP4 ~15% higher throughput than NVFP4; NVFP4 better quality (group-16 scaling). For small LLMs PTQ drop is "non-negligible", QAT/QAD required.

For ELAI on consumer 12-24 GB Ampere/Ada:
- NF4: best PPL among 4-bit, terrible throughput, only choice if you also want QLoRA on the same checkpoint.
- Marlin-AWQ: best Pareto point — same code quality as NF4, 4.4x faster.
- GPTQ: behind on both axes vs AWQ; "still widely used due to tooling" but no reason to pick new.
- EXL3: best quality-at-low-bpw, but smaller ecosystem; right for 24GB tier wanting 32B class.
- MXFP4/NVFP4: irrelevant until ELAI's 48GB tier moves to Blackwell.

## KV-cache quantization

Production reality (vLLM 2026): FP8 KV cache is the default. ~2x context window for ~0% throughput cost on Hopper/Ada, near-zero accuracy loss (docs.vllm.ai/quantized_kvcache).

| Approach | Bits | Context multiplier | Quality risk | Citation |
|---|---|---|---|---|
| FP16 KV (baseline) | 16 | 1x | none | - |
| FP8 KV (E4M3) | 8 | 2x | "negligible accuracy loss" | docs.vllm.ai/fp8 |
| INT8 KV | 8 | 2x | minimal | arxiv 2601.04719 |
| KIVI 2-bit | 2 | 8x | per-channel K + per-token V, tuning-free; works on most | arxiv 2402.02750 |
| KVQuant | <4 | 4x+ | "10M context" claim with calibration | KVQuant paper |
| MiniKV | 2 | 8x | selective coverage to claw back accuracy | ACL 2025 findings.952 |
| TurboQuant k8v4 | mix | 4-6x | parity-with-FP8 accuracy, BUT 20-34% throughput hit | vllm.ai/blog/turboquant |
| TurboQuant 3bit-nc | 3 | 6x+ | ~20-point reasoning drop | vllm.ai/blog/turboquant |

For ELAI at 25K context on 12 GB: KV cache is the limit, not weights (KV ~921 MB at 25K per ELAI notes). Switching to FP8 KV unlocks ~50K context for free on Ada+, ~30-35K on Ampere via software FP8 sim. KIVI/KVQuant double that again but add per-token quant overhead — not worth it until 100K+.

## Per-model quantization sensitivity (Qwen-Coder family)

Findings from arxiv 2505.20276 (long-context Qwen2.5) and Qwen-official benchmarks:
- Qwen2.5-7B: 4-bit PPL stays within 6% of FP16 across AWQ/GPTQ/NF4 on Wikitext, BUT long-context (>64K) degrades hard — up to 59% drop on some 4-bit methods. Code-gen (HumanEval) is more robust than reasoning at 4-bit.
- Qwen2.5-32B: all four 4-bit formats within 6% PPL; BnB-NF4 wins on PPL but AWQ wins on every other axis (Jarvislabs).
- Qwen2.5-72B: notably robust under BnB-NF4 (unlike Llama-3.1-70B which drops 32% under same setting) — Qwen architecture quantizes well at scale.
- Qwen3 family: AWQ + Marlin is the default stack in vLLM. FP8 quantized variants ship from RedHatAI/Qwen3-8B-quantized.w4a16 etc — W4A16 is the Qwen3 reference precision.
- General rule (arxiv 2504.04823 COLM 2025): mathematical reasoning (AIME, MATH-500) degrades earlier than code generation; W4A16 holds, W4A8/W3A16 doesn't.

ELAI-specific: Qwen2.5-Coder-7B at NF4 is a known "accuracy at risk" zone for reasoning steps. Code quality is fine; chain-of-thought reasoning is where the loss shows up.

## Best recommendation for ELAI per VRAM tier

| Tier | Model | Format pick | Reasoning | Citation |
|---|---|---|---|---|
| 8 GB | Qwen2.5-Coder-3B | GGUF Q5_K_M or AWQ-Marlin | 3B is small enough that 4-bit hurts (smaller models tolerate quant worse); use Q5_K for headroom OR AWQ-Marlin if vLLM serving | willitrunai, arxiv 2504.04823 |
| 12 GB | Qwen2.5-Coder-7B | AWQ + Marlin (W4A16) + FP8 KV | Same code quality as ELAI's NF4, 4.4x faster, unlocks ~50K context via FP8 KV | Jarvislabs, docs.vllm.ai/fp8 |
| 24 GB | Qwen2.5-Coder-14B / Qwen3-14B | AWQ + Marlin OR EXL3 at 4 bpw | 24 GB fits 14B at Q6_K but AWQ-Marlin throughput crushes GGUF; EXL3 if you want headroom for 32K+ context | apxml.com/models/qwen3-14b, EXL3 docs |
| 48 GB | Qwen2.5-Coder-32B / Qwen3-32B | FP8 W8A8 on Ada/Hopper, else AWQ-Marlin + FP8 KV | At 32B, W8A8 FP8 is lossless per COLM 2025; falls back to W4A16+FP8-KV on Ampere | arxiv 2504.04823, docs.vllm.ai/fp8 |
| 48 GB + B200 | Qwen3-32B | NVFP4 with QAD calibration OR FP8 | NVFP4 PTQ has non-negligible drop on small/medium LLMs — only deploy with QAD; otherwise FP8 stays winner | NVIDIA NVFP4-QAD report, arxiv 2603.08747 |

Migration order:
1. (cheap) Swap KV cache to FP8 — biggest single win for context budget.
2. (medium) Re-quantize the 7B/3B to AWQ via AutoAWQ (~2 hr on consumer GPU), serve via vLLM with Marlin backend.
3. (later) Add EXL3 as the 14B-on-24GB path.
4. (future) Re-evaluate MXFP4/NVFP4 when ELAI moves to B200/RTX5090.

## TurboQuant ADOPT/DEFER call

DEFER stays correct as of May 2026. New evidence:

- vLLM's own measured study (vllm.ai/blog/turboquant) directly compared TurboQuant variants to FP8 KV: all TurboQuant variants underperform FP8 on throughput (20-34% slower) AND k3v4-nc / 3bit-nc drop ~20 reasoning points.
- k8v4 reaches FP8 parity on accuracy but has no throughput advantage — "no advantage over FP8".
- ELAI's stated raw-fp32-vectors-today plan is fine; switch directly to FP8 KV (already in vLLM) instead of TurboQuant.
- TurboQuant remains relevant for vector-DB-style retrieval (which the paper actually targets), not LLM KV cache in production. ELAI's RAG embedding store could revisit TurboQuant for embedding compression — that's a different decision than serving KV.
- Google has NOT released an official implementation as of April 2026 (only community ports). Production maturity not there.

## Anti-patterns
- Picking AWQ but not enabling Marlin/Machete backend: throws away 10.9x speedup, leaves you at 68 t/s (Jarvislabs).
- NF4 + FP16 compute on Ampere when bfloat16 is available: training is unstable with FP16 compute dtype; bfloat16 is the QLoRA-paper-recommended compute dtype (HF bitsandbytes blog).
- Pushing NF4 to >64K context on small Qwens: up to 59% degradation (arxiv 2505.20276); use FP8 KV or stay <40K.
- W4A8 / sub-W4 weights for reasoning steps: COLM 2025 (arxiv 2504.04823) flags this as the cliff.
- MXFP4 PTQ on small models without QAD: non-negligible accuracy drop, MLP up/down-projections particularly sensitive (arxiv 2603.08747).
- Trusting llama.cpp Q4_K_M for code: "moderate quality loss, more noticeable for coding and reasoning" — bump to Q5_K_M / Q6_K for code workloads.
- Deploying TurboQuant variants <4bit for KV cache in production: vLLM's own measurement says don't.
- Marlin on H100: Marlin is Ampere-optimized; H100 wants Machete (Red Hat / Neural Magic blog).

## Open questions
1. Measured Marlin-AWQ throughput for Qwen2.5-Coder-7B specifically on RTX 3060 12 GB (not 32B on H100) — what's the real ELAI-tier number?
2. Long-context PPL curve for Qwen2.5-Coder-7B under AWQ vs NF4 from 8K to 25K — does the 59% long-context risk apply at 25K or only at 64K+?
3. EXL3 vs AWQ-Marlin head-to-head on Qwen2.5-Coder-14B at 24 GB — third-party benchmark, not author claim.
4. FP8 KV cache impact on Qwen-Coder code-completion specifically (HumanEval, MBPP at FP8 KV vs FP16 KV).
5. Does TurboQuant earn its place in ELAI's RAG embedding store (768d/384d at 3 bits, 10.7x compression) vs naive INT8 quantization of embeddings? Different problem from KV cache — needs its own bake-off.
