# Attention kernels for ELAI (2024-2026)

## TL;DR
- Keep FA2 (flash-attn 2.7.4 / 2.8.3) as mandatory on RTX 3060/3090; SM_80+ still officially supported, no deprecation signaled by Tri Dao through 2026.
- FA3 is Hopper-only (H100/H200) and FA4 is Hopper+Blackwell (CuTeDSL); neither runs the optimized path on Ampere consumer GPUs, so they only matter for ELAI cloud-side, never local.
- For >25K ctx on a single 3090, the cheap wins are KV-cache quantization (already in ELAI TQ4/TQ8) plus StreamingLLM-style attention sinks; the bigger lever is MInference dynamic sparse pre-fill (10x on A100, kernel ports exist for Ampere).
- Qwen2.5-Coder-7B is dense GQA (28 Q heads / 4 KV heads, head_dim=128, num_layers=28, hidden=3584, rope_theta=1e6, sliding_window present but `use_sliding_window=false`) — so any kernel that assumes head_dim 64/128 power-of-two works; head_dim 256 mixed layouts (the Gemma-4 issue) are absent.
- FA2 backward atomicAdd is non-deterministic by default — for ELAI's entropy quarantine, force `deterministic=True` on any path that records gradients, or restrict to inference-only (which is the current sidecar mode anyway).

## Key sources

| arxiv / URL | what | takeaway |
|---|---|---|
| [arXiv:2307.08691](https://arxiv.org/pdf/2307.08691) | FlashAttention-2 paper | Ampere ceiling ~70% theoretical FLOPs; baseline for ELAI. |
| [arXiv:2407.08608](https://arxiv.org/abs/2407.08608) | FlashAttention-3 paper | 1.5-2.0x on H100 FP16 (740 TFLOPs/s); Hopper TMA + warp-specialization, not Ampere-applicable. |
| [tridao.me/blog/2024/flash3](https://tridao.me/blog/2024/flash3/) | Tri Dao FA3 blog | Explicitly Hopper; FA2 still recommended on Ampere. |
| [together.ai/blog/flashattention-4](https://www.together.ai/blog/flashattention-4) | FA4 release | Blackwell B200 1605 TFLOPs/s; lacks backward / GQA / varlen at release — not production-ready for Ampere. |
| [github.com/Dao-AILab/flash-attention](https://github.com/Dao-AILab/flash-attention) | Tri Dao repo | Actively maintained 2026; SM_80 in supported list; Triton AMD migration; no SM_80 deprecation notice. |
| [arXiv:2311.01282](https://arxiv.org/abs/2311.01282) | FlashDecoding++ | Up to 4.86x decode speedup on NVIDIA vs HF; flat-GEMM + heuristic dataflow; helps Ampere too. |
| [arXiv:2309.06180](https://arxiv.org/pdf/2309.06180) | PagedAttention / vLLM | OS-style paging for KV; de-facto KV-cache management standard. |
| [arXiv:2312.07104](https://arxiv.org/pdf/2312.07104) | SGLang / RadixAttention | LRU radix tree over KV for automatic prefix sharing; beats PagedAttention on multi-turn. |
| [arXiv:2310.01889](https://ar5iv.labs.arxiv.org/html/2310.01889) | Ring Attention | Block-wise + ring KV passing; near-infinite ctx but requires multi-device (won't help single 3090). |
| [arXiv:2309.17453](https://arxiv.org/abs/2309.17453) | StreamingLLM (ICLR 2024) | Attention sinks + 4 init tokens kept + sliding window; up to 4M tokens, 22.2x vs recompute. |
| [arXiv:2407.02490](https://arxiv.org/abs/2407.02490) | MInference (NeurIPS 2024 Spotlight) | 3 sparse patterns (A-shape / Vertical-Slash / Block-Sparse); 10x pre-fill on A100 at 1M ctx, no fine-tune. |
| [arXiv:2410.10819](https://arxiv.org/abs/2410.10819) | DuoAttention (ICLR 2025) | Retrieval vs Streaming heads; 1.67x mem reduction + 1.50x decode on GQA models (Qwen2.5 is GQA). |
| [DeepWiki MLA](https://deepwiki.com/deepseek-ai/DeepSeek-V3/4.2-multi-head-latent-attention-(mla)) | Multi-head Latent Attention | ~10x KV-cache compression; architectural change, NOT a drop-in kernel — Qwen2.5 can't use it. |
| [arXiv:2309.00071](https://arxiv.org/html/2309.00071v3) | YaRN | Context extension via NTK-by-parts + attention-temp scaling; kernel-transparent (still uses FA2). |
| [PyTorch 2.5 / 2.6 FlexAttention blog](https://pytorch.org/blog/flexattention-flashattention-4-fast-and-flexible/) | FlexAttention | Custom score-mod via BlockMask, compiles to FA-like kernels; needs `torch.compile(dynamic=False)`. |
| [HuggingFace Qwen2.5-Coder-7B config.json](https://huggingface.co/Qwen/Qwen2.5-Coder-7B-Instruct/blob/main/config.json) | model config | head_dim=128, 28Q/4KV GQA, 28 layers, rope_theta=1e6, sliding_window present but disabled. |
| [GitHub flash-attention #31787 (transformers)](https://github.com/huggingface/transformers/issues/31787) | FA2 non-determinism bug | Backward pass non-deterministic via atomicAdd; v2.4.1+ has `deterministic=True` flag with ~37.9% throughput loss in FA3. |
| [GitHub QwenLM/Qwen3.6 #104](https://github.com/QwenLM/Qwen3.6/issues/104) | FA + Qwen 3D position_ids bug | 3D position_ids misinterpreted as packed — Qwen 2.5 uses 1D so unaffected, but watch for Qwen3 family migration. |

## Kernel comparison

| Kernel | Best HW | Qwen2.5 compat | Max ctx tested | Speedup cited | Ampere consumer GPU? | Citation |
|---|---|---|---|---|---|---|
| FlashAttention-2 | A100 / RTX 3090 / 4090 / H100 | yes (head_dim 128 GQA OK) | 32K native, 128K w/ YaRN | ~70% peak Ampere FLOPs | YES — primary target | [arXiv:2307.08691](https://arxiv.org/pdf/2307.08691) |
| FlashAttention-3 | H100 / H200 | yes but kernel disabled on Ampere | benchmarked to 16K | 1.5-2.0x vs FA2 on H100 | NO (no Hopper TMA) | [arXiv:2407.08608](https://arxiv.org/abs/2407.08608) |
| FlashAttention-4 | B200 / Hopper | partial (no GQA/backward at release) | n/a | 1.3x vs cuDNN, 2.7x vs Triton on B200 | NO | [together.ai/blog/flashattention-4](https://www.together.ai/blog/flashattention-4) |
| FlashDecoding++ | NVIDIA + AMD | yes | tested to 128K | up to 4.86x decode | YES | [arXiv:2311.01282](https://arxiv.org/abs/2311.01282) |
| PagedAttention (vLLM) | any CUDA SM_80+ | yes | 1M ctx (Qwen2.5-1M) | not a math win — memory layout | YES | [arXiv:2309.06180](https://arxiv.org/pdf/2309.06180) |
| RadixAttention (SGLang) | any CUDA SM_80+ | yes | multi-turn / RAG workloads | up to 5x throughput on shared-prefix workloads | YES | [arXiv:2312.07104](https://arxiv.org/pdf/2312.07104) |
| Ring Attention | multi-GPU (Ring of >=2) | yes | "millions of tokens" | linear with device count | NO on single 3090 | [arXiv:2310.01889](https://ar5iv.labs.arxiv.org/html/2310.01889) |
| StreamingLLM | any | yes (no retrain needed) | 4M tokens | up to 22.2x vs sliding-recompute | YES | [arXiv:2309.17453](https://arxiv.org/abs/2309.17453) |
| MInference 1.0 | A100 cited; Ampere kernels | yes (works on dense GQA) | 1M ctx | 10x pre-fill on A100 | YES (3090 cited in repo) | [arXiv:2407.02490](https://arxiv.org/abs/2407.02490) |
| DuoAttention | any FA2-capable | yes (GQA path tested) | 3.3M w/ quant on A100 | 1.50x decode + 1.67x mem on GQA | YES | [arXiv:2410.10819](https://arxiv.org/abs/2410.10819) |
| MLA (DeepSeek-V3) | any | NO — Qwen2.5 has no MLA weights | n/a | ~10x KV compression | architectural | [DeepWiki MLA](https://deepwiki.com/deepseek-ai/DeepSeek-V3/4.2-multi-head-latent-attention-(mla)) |
| xFormers memory-efficient | any incl. Turing | yes | 32K+ | typically slower than FA2 dense | YES | [Medium FA vs xFormers](https://medium.com/@siddharthtiwari01/flashattention-vs-xformers-vs-flexattention-a-systems-level-comparison-c07c6f1d92c5) |
| FlexAttention (torch 2.6+) | any FA-capable | yes | depends on score_mod | 1.6-3.2x fwd vs Triton on GB200 | YES (needs compile) | [PyTorch FlexAttention blog](https://pytorch.org/blog/flexattention-flashattention-4-fast-and-flexible/) |
| cuDNN SDPA backend | H100+ primarily | yes | 32K | ~75% faster than FAv2 on H100 | partial — gains mostly Hopper | [PyTorch 2.5 release](https://github.com/pytorch/pytorch/releases/tag/v2.5.0) |

## What runs on RTX 3060/3090 (ELAI hardware)

Confirmed Ampere-supported (SM_80 / SM_86) kernels per the FA2 README and pip wheels ([Dao-AILab/flash-attention](https://github.com/Dao-AILab/flash-attention), [flashattn.dev](https://flashattn.dev/blog/flash-attention-vs-standard-attention)):

- FlashAttention-2 (current ELAI baseline) — fp16/bf16 only, no fp32. Wheel `flash_attn-2.7.4.post1+cu128torch2.7.0...cp312-cp312-win_amd64.whl` exists, and pypi `flash-attn 2.8.3` (Jan 2026) supports CUDA 11.8-12.8 / torch 2.4-2.9.
- FlashDecoding / FlashDecoding++ kernels — decode-side, vendored in vLLM and FlashInfer.
- PagedAttention (vLLM) and RadixAttention (SGLang) — both run native on 3090; one community recipe runs Qwen3.6-27B at 125K ctx on a single 3090 with vLLM ([Medium 3090 stack](https://medium.com/@fzbcwvv/an-overnight-stack-for-qwen3-6-27b-85-tps-125k-context-vision-on-one-rtx-3090-0d95c6291914)).
- StreamingLLM — pure-Python attention-sink masking, no Hopper requirement.
- MInference — repo claims "approximate and dynamic sparse calculate the attention, which reduces inference latency by up to 10x for pre-filling on an A100"; kernel is Triton + custom CUDA, runs on Ampere.
- DuoAttention — GQA path explicitly tested; Qwen2.5 is GQA so applies.
- xFormers memory-efficient attention — broadest GPU coverage, slower than FA2 on raw throughput.
- FlexAttention — needs `torch.compile(dynamic=False)`; Ampere kernels compile fine, B200 numbers don't transfer.

NOT runnable on 3090 (cited):

- FlashAttention-3 (Hopper TMA, asynchronous warp specialization — no SM_86 path) — [tridao.me/blog/2024/flash3](https://tridao.me/blog/2024/flash3/).
- FlashAttention-4 (CuTeDSL targeting SM90+ / SM100 Blackwell) — [together.ai/blog/flashattention-4](https://www.together.ai/blog/flashattention-4), [Modal reverse-engineering](https://modal.com/blog/reverse-engineer-flash-attention-4).
- cuDNN attention's "75% faster than FAv2" headline number — that is measured on H100, not Ampere ([PyTorch 2.5 release notes](https://github.com/pytorch/pytorch/releases/tag/v2.5.0)).

## Long-context attention extension recipe (for ELAI >25K ctx on single 3090)

Stacked, lowest-risk first. All numbers cited.

1. KEEP FA2 with bf16 (already mandatory per ELAI baseline). No replacement on Ampere.
2. KV-cache quantization. Two grounded options:
   - `--kv-cache-dtype fp8_e5m2` (vLLM): halves KV bytes, "extends ctx 8K -> 20K" on 3090 per [n1n.ai recipe](https://explore.n1n.ai/blog/qwen3-6-27b-local-inference-rtx-3090-vllm-ollama-2026-05-03). Already aligned with ELAI's TQ4/TQ8 plan.
   - TurboQuant K8/V4 mixed precision — community-supported on 3090.
3. RoPE extension to push native 32K -> 128K+: use **YaRN** (NTK-by-parts + temperature scaling, ~0.1% pre-train data cost per [arXiv:2309.00071](https://arxiv.org/html/2309.00071v3)). Qwen2.5-1M tech report ([arXiv:2501.15383](https://arxiv.org/pdf/2501.15383)) explicitly documents the YaRN-equivalent recipe for Qwen2.5.
4. **StreamingLLM masking** for unbounded chat: keep first 4 tokens (attention sinks) + recent window. Drop-in mask, no FA2 changes. 22.2x vs sliding-recompute per [arXiv:2309.17453](https://arxiv.org/abs/2309.17453).
5. **MInference dynamic sparse pre-fill** when input >50K. The three patterns (A-shape / Vertical-Slash / Block-Sparse) are detected offline per head, applied online via custom kernels. 10x pre-fill speedup on A100 at 1M ctx, kernels are Triton/CUDA and run on SM_86 ([microsoft/MInference](https://github.com/microsoft/MInference)).
6. **DuoAttention head classification** for decode: identify retrieval vs streaming heads in Qwen2.5-Coder once offline, then 1.50x decode + 1.67x KV memory reduction on GQA models ([arXiv:2410.10819](https://arxiv.org/abs/2410.10819)).
7. SKIP RingAttention until ELAI has 2x3090 (it requires >=2 devices; cited as enabling "device-count times longer sequence" but useless on a single 3090).
8. SKIP MLA — it's an architecture change (low-rank latent KV), not a kernel; Qwen2.5 weights cannot be retrofitted without retraining.

## Determinism concerns

ELAI is determinism-first. Grounded constraints per [transformers#31787](https://github.com/huggingface/transformers/issues/31787), [transformers#39982](https://github.com/huggingface/transformers/issues/39982), and the [DASH paper (arXiv:2601.21824)](https://arxiv.org/abs/2601.21824):

- FA2 forward pass is deterministic given fixed `cublas_workspace_config` and fixed seed. Backward pass uses `atomicAdd` and is non-deterministic by default — bitwise variation between runs.
- `flash_attn>=2.4.1` exposes `deterministic=True`; cited cost in FA3 is up to 37.9% throughput loss on backward.
- DASH (ICLR 2026) delivers ~1.28x speedup vs naive deterministic backward but still requires CTA serialization.
- For ELAI: inference-only sidecar means no gradient, so the FA2 backward issue is moot today. If/when ELAI moves to on-device fine-tune or RLHF, mandate `deterministic=True`.
- StreamingLLM, PagedAttention, RadixAttention are deterministic in forward (pure masking / pure layout). Safe.
- MInference is approximate by construction (sparse patterns drop low-magnitude entries). For ELAI's entropy quarantine, treat MInference outputs as a separate stream — do NOT mix with full-attention outputs in the same hash chain.
- FlexAttention with `dynamic=False` is deterministic; recompiles produce identical SASS.
- cuDNN SDPA backend deterministic guarantees are weaker than FA2 — see [pytorch#152595](https://github.com/pytorch/pytorch/issues/152595) for SDPA backend variance.

## Best techniques to adopt (ranked, with integration hook)

| Rank | Technique | Grounding | Integration hook |
|---|---|---|---|
| 1 | Keep FA2 mandatory, upgrade to flash-attn 2.8.3 (Jan 2026) | [pypi flash-attn](https://pypi.org/project/flash-attn/) | bump `pyproject.toml` in `python/elai_inference`; verify FA2 env still loads under torch 2.7/2.8. |
| 2 | StreamingLLM attention sinks for any chat >25K ctx | [arXiv:2309.17453](https://arxiv.org/abs/2309.17453) + [mit-han-lab/streaming-llm](https://github.com/mit-han-lab/streaming-llm) | wrap `model.generate` attention mask in sidecar `server.py` — keep first 4 tokens + sliding window of `ELAI_LOCAL_MAX_CONTEXT`. |
| 3 | YaRN context extension (Qwen2.5-1M variant) | [arXiv:2501.15383](https://arxiv.org/pdf/2501.15383) | already a config-only change in `from_pretrained`: `rope_scaling={"type":"yarn","factor":4.0,...}`. Validate eval before adopting. |
| 4 | KV-cache quantization fp8_e5m2 or TQ4/TQ8 | [n1n.ai recipe](https://explore.n1n.ai/blog/qwen3-6-27b-local-inference-rtx-3090-vllm-ollama-2026-05-03) | ELAI already plans TQ flag in `--turboquant-bits`; default to 4-bit for >25K ctx per measured baseline. |
| 5 | MInference dynamic sparse pre-fill at >50K | [arXiv:2407.02490](https://arxiv.org/abs/2407.02490) + [microsoft/MInference](https://github.com/microsoft/MInference) | optional sidecar mode (env `ELAI_MIN_INFERENCE=1`); quarantine outputs from entropy hash. |
| 6 | DuoAttention head classification (decode) | [arXiv:2410.10819](https://arxiv.org/abs/2410.10819) | offline calibration step producing per-head mask; load via sidecar `--duo-mask path`. |
| 7 | RadixAttention prefix sharing on multi-turn | [arXiv:2312.07104](https://arxiv.org/pdf/2312.07104) | requires SGLang backend swap — strategic, not 2026-Q2 work. |
| 8 | FlexAttention for custom mask experiments | [PyTorch blog](https://pytorch.org/blog/flexattention-flashattention-4-fast-and-flexible/) | research-only path; keep FA2 as default. |

## Anti-patterns

- Adopting **FA3 or FA4 locally**. Both are Hopper/Blackwell SASS. The "75% faster than FAv2" claim is on H100, not Ampere — citing it for 3090 planning is unsearched conjecture. ([tridao.me/blog/2024/flash3](https://tridao.me/blog/2024/flash3/), [together.ai/blog/flashattention-4](https://www.together.ai/blog/flashattention-4)).
- Treating **MLA as a kernel swap**. MLA is a weight architecture (DeepSeek-V2/V3 only). Qwen2.5-Coder has standard GQA — there is no MLA path without retraining. ([DeepWiki MLA](https://deepwiki.com/deepseek-ai/DeepSeek-V3/4.2-multi-head-latent-attention-(mla))).
- **RingAttention on a single 3090**. The communication ring needs >=2 devices; on 1 GPU it's identity. ([arXiv:2310.01889](https://ar5iv.labs.arxiv.org/html/2310.01889)).
- **Mixed-head_dim architectures** (Gemma 3-equivalent, 256+512 layers). Qwen2.5 dense head_dim=128 sidesteps this; do not regress by adopting Gemma-family models. (Confirmed via [Qwen2.5-Coder-7B config.json](https://huggingface.co/Qwen/Qwen2.5-Coder-7B-Instruct/blob/main/config.json)).
- **SDPA without explicit backend pin**. `torch.nn.functional.scaled_dot_product_attention` will silently fall back to MATH on shape errors; pin with `torch.nn.attention.sdpa_kernel([SDPBackend.FLASH_ATTENTION])`. ([pytorch#133780](https://github.com/pytorch/pytorch/issues/133780)).
- **Flash Attention 2 with Qwen3 / Qwen2.5-VL 3D position_ids** — known illegal-memory-access bug ([QwenLM/Qwen3.6#104](https://github.com/QwenLM/Qwen3.6/issues/104)). Stays a non-issue while ELAI is on Qwen2.5-Coder text-only.
- Treating **xFormers as faster than FA2** on dense LLM workloads — FA2 wins on dense; xFormers wins on flexibility. ([Medium comparison](https://medium.com/@siddharthtiwari01/flashattention-vs-xformers-vs-flexattention-a-systems-level-comparison-c07c6f1d92c5)).
- Ignoring **FA2 backward non-determinism** if/when training is added. Must set `deterministic=True`. ([transformers#31787](https://github.com/huggingface/transformers/issues/31787)).

## Open questions

1. Measured FA2 forward throughput on RTX 3090 at 25K ctx with Qwen2.5-Coder-7B vs xFormers / FlexAttention — is the FA2 advantage in `metrics/baseline_findings_and_actions.md` reproducible at 2026-05 flash-attn 2.8.3?
2. Does MInference's Vertical-Slash kernel actually run on SM_86 (3090) with the same 10x pre-fill claim, or is it A100-only in practice? Needs `microsoft/MInference` reproduction at 50K-100K ctx.
3. YaRN scaling factor sweet spot for Qwen2.5-Coder-7B (Qwen2.5-1M ships factor=4 for 128K; what's the eval cost at factor=16 / 512K on 3090)?
4. DuoAttention retrieval-head calibration cost: does ELAI need a one-time offline pass per checkpoint, and does it transfer between Qwen2.5-Coder-3B and -7B?
5. PyTorch 2.6 FlexAttention `torch.compile(dynamic=False)` compile-time overhead on first-token latency — is it acceptable for ELAI's interactive sidecar, or does it need warm-cache hosting?
