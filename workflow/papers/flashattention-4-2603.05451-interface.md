Algorithm: FlashAttention-4 (FA4)
ArXiv: 2603.05451
Paper: FlashAttention-4: Algorithm and Kernel Pipelining Co-Design for Asymmetric Hardware Scaling
Authors: Ted Zadouri, Markus Hoehnerbach, Jay Shah, Timmy Liu, Vijay Thakkar, Tri Dao
Submitted: 2026-03-05

CITATION VERIFICATION STATUS: REAL PAPER. URL https://arxiv.org/html/2603.05451v1 resolves. Title, authors, abstract confirmed via WebFetch and WebSearch cross-check.

Hardware targets:
Primary: NVIDIA Blackwell B200, GB200 (SM120)
Secondary: NVIDIA Hopper H100 (SM90)
Also: SM100 (files present in repo per github.com/Dao-AILab/flash-attention/tree/main/flash_attn/cute)
NOT supported: Ampere (SM80/SM86), Ada Lovelace (SM89), consumer GPUs including RTX 3090
RTX 3090 verdict: CONFIRMED NOT SUPPORTED by FA4. RTX 3090 = SM86 (Ampere). FA4 README states "Hopper and Blackwell GPUs" only.

Inputs:
Q | bf16 or fp16 tensor | (batch, seqlen, nheads, headdim) | headdim in {64, 128, (192,128)}
K | bf16 or fp16 tensor | (batch, seqlen_k, nheads_k, headdim) | same dtype as Q
V | bf16 or fp16 tensor | (batch, seqlen_k, nheads_k, headdim) | same dtype as Q
softmax_scale | float | scalar | typically 1/sqrt(headdim)
causal | bool | | optional, enables causal masking
varlen | bool | | optional, variable sequence length mode

Outputs:
out | same dtype as Q | (batch, seqlen, nheads, headdim) | exact attention output (not approximate)
lse | float32 | (batch, nheads, seqlen) | log-sum-exp for backward pass

Mathematical invariants (testable):
output is exact attention (not approximate): FA4 uses conditional softmax rescaling with final normalization step that corrects deviations — same result as standard scaled dot-product attention
assert out.dtype == Q.dtype  # output dtype preserved
assert lse.dtype == torch.float32  # lse always fp32
Backward pass has deterministic mode: serialize reductions via semaphore locks; nondeterministic mode is faster but not reproducible

Hyperparameters / compile-time config:
tile_size | 128x128 | {64x128, 128x128} | larger tiles vs FA3's 64x128 for Hopper
deterministic_bwd | False | {True, False} | True = reproducible gradients, lower throughput
pipeline_stages | auto | hardware-dependent | controlled by CuTe-DSL scheduler

Algorithmic changes vs FA2 / FA3:
vs FA2: asynchronous MMA operations, software-emulated softmax (vs hardware exp unit), tensor memory usage, 2-CTA MMA mode in backward
vs FA3: redesigned pipeline for Blackwell asymmetric scaling (tensor core throughput 2x, other units unchanged); larger 128x128 tiles; halved atomic reductions in backward; CuTe-DSL Python implementation vs C++ templates
Key insight: Blackwell doubles tensor core FLOP/s without doubling memory bandwidth or other units, so FA3's pipeline balance assumptions break on B200

Performance (GROUNDED — quoted from Abstract and Section 5):
1613 TFLOPs/s on B200 BF16 forward pass (71% theoretical utilization)
1.3x speedup over cuDNN 9.13 on B200 (range 1.1-1.3x, Figure 4)
2.7x speedup over Triton on B200 (range 2.1-2.7x, Figure 4)
Benchmark range: sequence lengths 1k-32k, head dims 64/128
FA2 2.8.3 used as comparison baseline in paper

Implementation:
Language: CuTe-DSL embedded in Python (not C++ templates)
Compile time: 20-30x faster than C++ template approach (Abstract, Table 4)
CUDA requirement: 13.1 (stated in benchmarking section)
Repo: github.com/Dao-AILab/flash-attention/tree/main/flash_attn/cute
pip package: flash-attn-4 (per repo README, cu13 extra for CUDA 13 systems)
License: CC BY-NC-SA 4.0 (non-commercial — not permissive for commercial use despite paper saying "permissive")

PyTorch / SDPA integration:
NOT mentioned in paper. Paper states integration with "popular libraries" is ongoing work.
No confirmed torch.nn.functional.scaled_dot_product_attention backend registration for FA4 as of paper date.
SDPA on Ampere continues to work independently of FA4.

Failure modes:
FA4 will not compile or run on Ampere/Ada (SM < 90). Hard hardware gate.
Nondeterministic backward can cause non-reproducible training; use deterministic_bwd=True for RL
Sequence lengths not aligned to tile size may have edge-case handling overhead
CuTe-DSL requires CUDA 13.1; older CUDA installs will fail at import

Implementation pitfalls (do NOT do this):
Do NOT assume FA4 runs on RTX 3090, A100, or any pre-Hopper GPU
Do NOT assume FA4 is pip-installable as flash-attn (different package: flash-attn-4)
Do NOT assume CC BY-NC-SA 4.0 is permissive for commercial use — it is not
Do NOT assume PyTorch SDPA automatically uses FA4 kernels — integration is not confirmed
Do NOT skip deterministic_bwd=True for training workloads requiring reproducibility

Sidecar relevance for this project:
RTX 3090 (Ampere SM86) = FA4 NOT applicable. Current ELAI_FORCE_SDPA=1 is correct.
FA2 via flash-attn pip package IS supported on Ampere (SM86 >= 8.0 requirement met).
FA4 becomes relevant only if/when user acquires Hopper (H100) or Blackwell (B200/GB200).
Recommendation: keep SDPA for Qwen2.5-Coder-3B on RTX 3090; FA4 is irrelevant to current stack.
