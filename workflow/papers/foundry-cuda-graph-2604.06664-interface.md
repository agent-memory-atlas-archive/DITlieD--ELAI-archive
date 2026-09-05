Algorithm: Foundry - Template-Based CUDA Graph Context Materialization
ArXiv: 2604.06664
Paper: Foundry: Template-Based CUDA Graph Context Materialization for Fast LLM Serving Cold Start
Source: UC San Diego / UC Berkeley (Apr 2026)
Reliability: HIGH (peer-reviewed, quantitative benchmarks verified via WebFetch)

Key diagnostic finding relevant to E-3 plan:
CUDA graph capture dominates cold-start latency, NOT weight loading.
Weight loading via RDMA: 1-2 seconds.
CUDA graph capture: tens-of-seconds to minutes (Qwen3-235B-A22B: ~10 minutes before Foundry).
Pre-warming via 1-token warmup request to trigger graph capture is the effective mitigation.

Inputs (Foundry SAVE phase -- offline):
model | loaded LLM | any dense or MoE model running vLLM-style serving |
kv_cache_size | int | bytes | MUST be specified before SAVE so memory layout is consistent on LOAD
gpu_count | int | 1 | single GPU only for offline capture; multi-GPU not supported

Inputs (Foundry LOAD phase -- serving):
saved_template | serialized graph template | produced by SAVE phase |
model_weights | loaded weights | must match SAVE-phase architecture exactly |
kv_cache_size | int | must match SAVE-phase value | layout consistency required

Outputs:
reconstructed_cuda_graph | executable CUDA graph | |
cold_start_latency | float | seconds | up to 99% reduction vs. from-scratch capture

Hyperparameters:
topology_grouping | default: group by identical topology | reduces template count | fewer templates = faster LOAD
parameter_update_strategy | on-demand | range: eager vs on-demand | on-demand preferred; avoids full rebuild

Critical invariants (testable assertions):
assert kv_cache_size_save == kv_cache_size_load  # memory layout breaks if sizes differ
assert gpu_count == 1  # distributed multi-GPU capture not supported; must use single GPU for SAVE
# graph topology must match: weights and architecture cannot change between SAVE and LOAD
assert model_architecture_hash_save == model_architecture_hash_load

Complexity:
SAVE: one-time offline; O(num_graphs * graph_complexity)
LOAD: fast template instantiation; Qwen3-235B-A22B: 10 min -> 3.9s (99% reduction)
Archive size: 4-5x smaller than full CUDA checkpoint approach

Failure modes:
Pipeline parallelism not supported; only SPMD-style parallelism works
Memory fragmentation possible if many transient allocs occur during initialization
Allocation sequence divergence between SAVE and LOAD phases requires explicit replay
KV cache size mismatch between SAVE and LOAD phase silently produces wrong layouts

Implementation pitfalls (do NOT do this):
Do NOT attempt Foundry SAVE/LOAD in ELAI directly -- this is a vLLM-level system; use the DIAGNOSTIC
  finding (graph capture is the bottleneck) and implement warm() as a 1-token warmup request instead
Do NOT pre-warm on 12GB single-GPU if holding 2 models is not possible (pre-warm = no-op per plan Task 4)
Do NOT skip kv_cache_size specification if implementing Foundry-style persistence
Do NOT run SAVE phase on multi-GPU setup expecting transfer to single-GPU LOAD (unsupported)

E-3 plan use: the plan does NOT implement Foundry. It uses Foundry's finding to justify:
  (a) why warm() sends a 1-token request (to trigger graph capture proactively)
  (b) why pre-warm only enables on 24GB+ tier (12GB cannot hold 2 models)
  (c) why weight-load latency is NOT the bottleneck to optimize
