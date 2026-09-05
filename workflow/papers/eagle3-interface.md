Algorithm: EAGLE-3 (Extrapolation Algorithm for Greater Language-model Efficiency, v3)
ArXiv: 2503.01840
Paper: EAGLE-3: Scaling up Inference Acceleration of Large Language Models via Training-Time Test

Inputs:
target_model_features | tensor | shape [seq_len, k] per layer (low/mid/high) | three decoder layers required; k = hidden dim of target model
token_embeddings | tensor | shape [seq_len, k] | standard embedding lookup from target model vocab
sampled_token | int | scalar | the last accepted/generated token index; mandatory at each draft step
draft_tree_config | struct | depth=8, total_nodes=60 (7B/8B), 50 (13B), 48 (70B) | node count must match EAGLE-2 budget
data_scale | float | multiplier vs ShareGPT baseline (1x=~68K, 8x=~532K entries) | minimum 1x; scaling monotonically improves speedup

Outputs:
draft_tokens | list[int] | length <= draft_tree_depth (max 8) | lossless: strict speculative sampling acceptance; output distribution identical to target model
accepted_length | float | scalar (tau, mean accepted tokens per target-model forward pass) | tau >= 1.0 always; EAGLE-3 achieves tau ~5.8-6.6 at temperature=0 for best benchmarks
speedup_ratio | float | scalar | measured wall-clock; not theoretical; must be >= 1.0 in all practical batch sizes up to BS=64

Hyperparameters:
optimizer | AdamW | beta_1=0.9, beta_2=0.95 | standard; not sensitive per paper
learning_rate | 5e-5 | range not ablated | paper does not report sensitivity; treat as fixed
gradient_clip | 0.5 | fixed | standard stability measure
draft_tree_depth | 8 | EAGLE-2 used 6 | increased from 6 to 8 due to higher acceptance rates; node count held constant
fc_reduction | learned | input 3k -> output k | fuses low/mid/high features; must reduce to k before draft decoder
feature_layers | low, mid, high | three decoder layers | paper does not specify exact layer indices; implementor must tune per model
training_steps_per_sequence | 3 | simulates 3-step drafting during training | minimum to capture multi-step distribution shift; paper ablates implicitly via Table 2
chain_length_sgllang | 3 | no tree in SGLang batch experiments | chain length 2 used for vLLM batch experiments

Critical invariants (testable assertions):
assert abs(sum(acceptance_probs) / len(acceptance_probs) - n_alpha_flat) < 0.05  # n-alpha stays flat ~0.78-0.81 across positions 0-7 (Figure 7)
assert n_alpha[0] >= 0.78 and n_alpha[7] >= 0.75  # EAGLE-3 flat acceptance; EAGLE degrades to ~0.52 at position 7
assert speedup_bs64_sglang >= 1.30  # EAGLE-3 throughput > 1.30x at BS=64 (Table 3: 1.38x); EAGLE falls to 0.99x
assert tau_vicuna13b_humaneval >= 6.0  # best-case tau=7.54 (HumanEval, Vicuna 13B, temp=0, Table 1)
assert fused_feature_dim == target_hidden_dim  # FC layer must reduce [3k] -> [k]; no other dim is valid
assert l_fea_weight == 0.0  # feature prediction loss removed entirely; any nonzero value reintroduces the scaling bottleneck
assert draft_decoder_layers == 1  # single Transformer decoder layer; adding layers is untested and changes cost model

Complexity:
draft_forward | O(seq_len * k^2) per draft step | single decoder layer; lightweight vs target model | each draft step is ~1/N of target model cost where N = target model layers
fc_fusion | O(seq_len * 3k * k) | matrix multiply once per sequence position | negligible vs attention
tree_expansion | O(nodes * k^2) | 48-60 nodes depending on model size | parallelizable; same as EAGLE-2
target_verify | O(nodes) parallel positions in one forward pass | batches all draft tokens in single target model call | memory bottleneck at large BS
training_time_test | 3x sequence length in attention mask per training step | diagonal mask avoids full quadratic; uses vector dot products for diagonal portions | see Section 3.1 / Figure 6

Failure modes:
distribution_shift_at_step2plus: removing l_fea without training-time test causes 1-alpha to collapse to ~0.20-0.30 (Figure 4 right panel); never deploy EAGLE-3 without training-time test enabled
data_scaling_ignored: EAGLE-2/EAGLE architecture flatlines at 1x data; must use EAGLE-3 architecture to benefit from scaling beyond ShareGPT
large_batch_speculation_disabled: EAGLE (not EAGLE-3) drops below 1.0x throughput at BS>=24 in SGLang and BS>=32 in vLLM; this is architecture-specific, not a general speculative sampling property
wrong_feature_layers: using only top-layer features (as in EAGLE) forfeits the multi-layer fusion gain (~15% speedup lost per Table 2 ablation)
tree_depth_6: keeping depth=6 from EAGLE-2 wastes acceptance capacity; EAGLE-3's higher n-alpha supports depth=8 at same node budget
temperature0_only_comparisons: methods like Medusa relax acceptance at temperature>0 (not lossless); cross-comparing with EAGLE-3 at temp=1 is invalid
reasoning_model_data: DeepSeek-R1-Distill variants require OpenThoughts-114k-math in training set; omitting it degrades math acceptance rates

Implementation pitfalls (do NOT do this):
do NOT retain l_fea (feature prediction loss) in any form — it is the root cause of the scaling ceiling and is removed entirely in EAGLE-3
do NOT feed ground-truth target features as input at Step 2+ during inference — ground-truth features are unavailable for unverified tokens; use draft model outputs a_hat instead
do NOT use standard lower-triangular attention mask for Step 2+ during training — diagonal mask required for predicted token positions (each chain is independent); using full causal mask leaks ground-truth information
do NOT use full matrix multiplication for diagonal attention portions — use vector dot products to avoid O(n^2) waste on zero blocks (Section 3.1)
do NOT compare EAGLE-3 throughput to EAGLE without controlling for batch size — EAGLE goes negative at BS>=24 while EAGLE-3 does not; aggregate numbers hide this crossover
do NOT treat HASS as equivalent to EAGLE-3 — HASS retains l_fea and top-layer-only features; EAGLE-3 speedup significantly exceeds HASS (Figure 2)
do NOT fix training data at ShareGPT only — EAGLE-3 scaling law is monotonic; 8x data (~532K) yields 4.4x speedup vs 3.7x at 1x data for LLaMA-Instruct 3.1 8B
do NOT assume chain_length=3 for vLLM — vLLM batch experiments use chain_length=2; SGLang uses chain_length=3; framework matters
