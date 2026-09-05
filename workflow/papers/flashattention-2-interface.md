Algorithm: FlashAttention-2: Faster Attention with Better Parallelism and Work Partitioning
ArXiv: 2307.08691
Paper: FlashAttention-2: Faster Attention with Better Parallelism and Work Partitioning (Tri Dao, 2023)

Inputs:
Q | tensor | (N, d) float16/bfloat16 | N = sequence length; d = head_dim, must be power-of-two in {32,64,128}; N must be divisible by block sizes Br, Bc
K | tensor | (N, d) float16/bfloat16 | same shape constraints as Q; for GQA/MQA kv_heads < q_heads, K shape is (kv_heads, N, d)
V | tensor | (N, d) float16/bfloat16 | same as K
causal_mask | bool | scalar | optional; when True enables skip of ~50% of blocks (1.7-1.8x speedup); mandatory for autoregressive decode
softmax_scale | float | scalar | default 1/sqrt(d); must be applied before max-subtract for numerical stability

Outputs:
O | tensor | (N, d) float16/bfloat16 | exact (non-approximate) softmax(QK^T)V; no approximation error
L | tensor | (N,) float32 | logsumexp per row = m + log(l); required for backward pass; stored in HBM
Invariant: O[i] = softmax(Q[i] @ K^T) @ V; no output approximation -- numerically identical to naive attention in float32 ref
Invariant: sum(softmax(Q[i] @ K^T)) == 1.0 for every row i (unit softmax)

Hyperparameters:
Br (row block size) | 64 or 128 | {32,64,128} | manually tuned; >128 causes register spilling on A100/Ampere
Bc (col block size) | 64 or 128 | {32,64,128} | manually tuned; no autotuning in v2
warps_per_block | 4 or 8 | {4,8} | architecture-specific; Ampere A100 uses 8 for best occupancy
head_dim d | 128 (Qwen2.5-Coder-7B) | power-of-two in {32,64,128} | head_dim=256 not supported in FA2; triggers silent fallback to SDPA/MATH

Critical invariants (testable assertions):
assert abs(O - naive_attention(Q, K, V, causal)).max() < 1e-2  # exact, not approximate (Section 2)
assert abs(softmax_rows.sum(axis=-1) - 1.0).max() < 1e-5  # unit softmax per row
assert L.shape == (batch, nheads, seqlen)  # logsumexp stored for backward
assert Br <= 128 and Bc <= 128  # register spill threshold on Ampere (Section 3)
assert head_dim in (32, 64, 128)  # FA2 supported dims; 256 forces SDPA fallback
# SDPA kernel pin (pytorch#133780): without explicit pin, silent MATH fallback on shape errors
assert backend_selected in ('flash_attention', 'efficient_attention')  # never MATH silently

Complexity:
Time: O(N^2 * d) FLOPs forward; O(2.5 * N^2 * d) backward (recompute S,P on-chip)
Space: O(N) HBM (stores O + L only; S,P never materialized in HBM)
Parallelism: forward parallelized across batch, heads, AND sequence (FA2 key improvement over FA1); single head split across thread blocks when batch*nheads < SM count (108 SMs on A100)
FLOPs utilization: 50-73% of theoretical max on A100; up to 225 TFLOPs/s measured on A100

Failure modes:
head_dim=256 or non-power-of-two: FA2 silently falls back to SDPA/MATH, losing all speedup (affects Gemma-3 family)
Br or Bc > 128 on Ampere: register spilling causes throughput degradation
SDPA without explicit backend pin (torch.nn.attention.sdpa_kernel): silent fallback to MATH on shape mismatch (pytorch#133780)
Backward pass on inference-only sidecar: backward uses atomicAdd = non-deterministic (transformers#31787); moot for inference-only ELAI sidecar; if fine-tune added, must set deterministic=True (~37.9% throughput cost)
FA3/FA4 on Ampere: FA3 is Hopper-only (H100/H200); FA4 targets Hopper+Blackwell; neither runs optimized path on RTX 3060/3090; deploying on Ampere silently reverts to SDPA or crashes

Implementation pitfalls (do NOT do this):
Do NOT rescale O at every inner block -- delay diag(l)^(-1) scaling to after the full loop to reduce non-matmul FLOPs by 16x
Do NOT store both m and l separately -- compute L = m + log(l) once, saves HBM write bandwidth
Do NOT split K/V across warps for backward -- split Q instead to eliminate inter-warp reduction overhead
Do NOT assume FA3 speedup numbers apply to Ampere -- "75% faster than FAv2" is H100-measured per pytorch 2.5 release notes
Do NOT use flash_attn with head_dim=256 -- it silently falls back; verify with torch.backends.cuda.flash_sdp_enabled()
Do NOT enable FA3/FA4 kernels on RTX 3090 -- no optimized Ampere path exists in either release
