Algorithm: StreamingLLM: Efficient Streaming Language Models with Attention Sinks
ArXiv: 2309.17453
Paper: Efficient Streaming Language Models with Attention Sinks (Xiao et al., ICLR 2024)

Inputs:
tokens | int[] | (T,) unbounded sequence | autoregressive streaming; T can exceed pre-training window
kv_cache | KVCache | (2, num_layers, cache_size, num_heads, head_dim) | dual-part: [sink_tokens | rolling_window]
sink_count | int | scalar | number of initial tokens to permanently retain; default 4; range 1-8; 1-2 insufficient
window_size | int | scalar | rolling window of recent tokens; default 1024-2048; must be >= model's attention span need
position_ids | int[] | (cache_size,) | CRITICAL: re-indexed relative to cache position, NOT absolute text position

Outputs:
next_token_logits | tensor | (vocab_size,) float | autoregressive next-token distribution
kv_cache_updated | KVCache | (2, num_layers, sink_count + window_size, num_heads, head_dim) | fixed size regardless of T
Invariant: cache_size == sink_count + window_size at all times (constant memory)
Invariant: first sink_count slots always hold KV of original tokens 0..sink_count-1 (never evicted)
Invariant: sliding portion holds the most recent window_size tokens in order

Hyperparameters:
sink_count | 4 | 1-8 | HIGH sensitivity: 1-2 insufficient (paper Sec 4.2); 4 optimal; >4 shows diminishing returns
window_size | 2048 | 512-4096 | MEDIUM sensitivity: larger improves quality but not consistently above 2048 (paper Sec 4.3)
position_reindex_mode | relative | {relative, absolute} | CRITICAL: absolute positions break RoPE for tokens after eviction (see pitfalls)
pre_attn_key_cache | bool | True | keys must be cached BEFORE RoPE rotation so positions can be re-applied at decode time

Critical invariants (testable assertions):
assert len(kv_cache[0]) == sink_count + window_size  # constant cache size (Sec 3.1)
assert position_ids[:sink_count] == list(range(sink_count))  # sinks always at absolute 0..N-1
assert position_ids[sink_count:] == list(range(sink_count, sink_count + len(rolling_window)))  # rolling re-indexed
# VRAM invariant: streaming VRAM <= baseline VRAM (streaming is memory win, not speed loss)
assert vram_with_streaming <= vram_baseline  # throughput parity check per Task 8
# 3 consecutive deterministic runs must be byte-equivalent (forward path is deterministic: pure masking, no fused-CTA)
assert run1_output == run2_output == run3_output  # determinism check

Complexity:
Time per token: O(1) -- constant regardless of total sequence length T (Sec 3.1 Table 1)
Time vs sliding-window-recompute: 22.2x speedup measured (paper abstract)
Space: O(sink_count + window_size) = O(1) w.r.t. T; KV cache fixed at constant size
Pre-training required: None -- works with any pre-trained LLM with local attention window

Failure modes:
Information retrieval beyond window_size: impossible -- tokens evicted from cache are permanently lost (Sec 5 Limitations)
Non-monotonic perplexity with cache size: increasing window_size does not consistently improve quality (paper Sec 4.3); do not assume larger is always better
Tasks requiring full-document context: RAG/long-doc QA/summarization beyond window unsupported
Models without attention sink phenomenon: rare; some architectures do not exhibit strong initial-token attention (verify per-architecture)
Position encoding mismatch post-eviction: absolute positions assigned to evicted-gap tokens break RoPE coherence

Implementation pitfalls (do NOT do this):
Do NOT assign original text positions to cached tokens after eviction -- position_ids must be re-indexed relative to cache slot, NOT absolute text position
Do NOT apply RoPE to keys before caching -- cache raw keys, apply RoPE with re-indexed positions at each decode step
Do NOT use fewer than 4 sink tokens -- 1 or 2 initial tokens is insufficient; softmax denominator instability (paper Sec 4.2)
Do NOT evict the first sink_count tokens under any LRU or recency eviction policy -- sinks are permanent
Do NOT assume streaming is a throughput loss -- it is a memory saving with throughput parity; any regression indicates implementation error
Do NOT treat sinks as semantically important -- they are attended to for numerical softmax stability, not content (paper Sec 2)
