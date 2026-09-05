Source: HuggingFace transformers — DynamicCache and StaticCache (cache_utils.py)
Reliability: OFFICIAL_SDK = HIGH
Transformers version on remote sidecar (pythondev): 4.57.6
Local transformers version: 5.5.3 (NOT used by sidecar — all facts below verified against 4.57.6)
Files examined:
  /home/user/.venvs/elai-sidecar/lib/python3.12/site-packages/transformers/cache_utils.py (1493 lines)
  /home/user/.venvs/elai-sidecar/lib/python3.12/site-packages/transformers/generation/utils.py (verified key sections)
  /home/user/.venvs/elai-sidecar/lib/python3.12/site-packages/transformers/models/qwen2/modeling_qwen2.py (verified)
  /home/user/elai_inference/server.py (verified NF4 compute dtype and SDPA selection)
  F:/.ELAI_workflow/.elai_cc/models/Qwen2.5-Coder-3B-Instruct-abliterated/config.json (verified model dims)
Extraction date: 2026-05-05


CLASS HIERARCHY (cache_utils.py, verified)

CacheLayerMixin (abstract)
  DynamicLayer          -- grows by concat; default for DynamicCache
  DynamicSlidingWindowLayer(DynamicLayer) -- sliding window variant
  StaticLayer           -- pre-allocated; for torch.compile; is_compileable=True
  StaticSlidingWindowLayer(StaticLayer) -- static + sliding

Cache
  DynamicCache(Cache)   -- wraps list of DynamicLayer (or DynamicSlidingWindowLayer)
  StaticCache(Cache)    -- wraps list of StaticLayer (or StaticSlidingWindowLayer)
  QuantizedCache(Cache)


PUBLIC API: DynamicCache
[cache_utils.py lines 910-1028]

Constructor:
  DynamicCache(
    ddp_cache_data: Optional[Iterable[tuple[Tensor, Tensor]]] = None,
    config: Optional[PretrainedConfig] = None,
    offloading: bool = False,
    offload_only_non_sliding: bool = False,
  )
  - If config=None and ddp_cache_data=None: lazy init, layer_class_to_replicate=DynamicLayer
  - If config=model.config: infers layer types (full vs sliding) from config.sliding_window
  - Qwen2.5-Coder-3B has use_sliding_window=false and sliding_window=32768 in config,
    but layer_types is NOT set in config (no "layer_types" key in config.json).
    With config passed: since sliding_window=32768 is set, layer_types defaults to
    ["sliding_attention" * 36] -> all 36 layers become DynamicSlidingWindowLayer(32768).
    IMPORTANT: pass config=None or config=model.config carefully (see pitfalls).

update(key_states, value_states, layer_idx, cache_kwargs={}) -> tuple[Tensor, Tensor]:
  [Cache.update lines 742-781; DynamicLayer.update lines 98-121]
  - Signature on Cache: update(key_states, value_states, layer_idx, cache_kwargs=None)
  - Delegates to layer.update(key_states, value_states, cache_kwargs)
  - DynamicLayer.update: concatenates new keys/values along dim=-2 (seq dim)
    self.keys = torch.cat([self.keys, key_states], dim=-2)
    self.values = torch.cat([self.values, value_states], dim=-2)
    returns (self.keys, self.values) i.e. the FULL accumulated tensor up to this point
  - Return shape: (batch, num_kv_heads, total_seq_len, head_dim) for both K and V
  - CONFIRMED: returns (updated_keys, updated_values) as (full_k, full_v), NOT just new tokens
  - Lazy init on first call: creates empty tensor on key_states.dtype, key_states.device

get_seq_length(layer_idx=0) -> int:
  [Cache.get_seq_length lines 798-802]
  - Returns self.layers[layer_idx].get_seq_length()
  - DynamicLayer.get_seq_length: returns self.keys.shape[-2] if initialized, else 0
  - Returns int (not tensor)
  - Safe to call on empty cache: returns 0

reorder_cache(beam_idx: LongTensor):
  [Cache.reorder_cache lines 829-832; CacheLayerMixin.reorder_cache lines 77-81]
  - Iterates all layers, calls layer.reorder_cache(beam_idx)
  - Uses index_select(0, beam_idx) on self.keys and self.values
  - CONFIRMED present and functional for beam search

crop(max_length: int):
  [Cache.crop lines 834-837; DynamicLayer.crop lines 140-152]
  - Negative max_length: max_length = get_seq_length() - abs(max_length)
  - Crops self.keys and self.values to [..., :max_length, :] in-place
  - No-op if current seq_length <= max_length

to_legacy_cache() -> tuple[tuple[Tensor, Tensor], ...]:
  [DynamicCache.to_legacy_cache lines 1005-1013]
  - Returns tuple of (layer.keys, layer.values) for each layer
  - Shape per layer: (batch, num_kv_heads, seq_len, head_dim)

from_legacy_cache(past_key_values) -> DynamicCache:
  [DynamicCache.from_legacy_cache lines 1015-1028]
  - Classmethod; accepts tuple-of-tuples format
  - Calls update for each layer to populate DynamicLayer objects
  - Handles None gracefully with a warning

__getitem__(layer_idx) -> tuple[Tensor, Tensor]:
  [Cache.__getitem__ lines 881-891]
  - Returns (layer.keys, layer.values) for backward compat with legacy indexing
  - past_key_values[0][0].shape[2] works for seq_length access


PUBLIC API: StaticCache
[cache_utils.py lines 1031-1107]

Constructor:
  StaticCache(
    config: PretrainedConfig,    -- REQUIRED (no default)
    max_cache_len: int,          -- REQUIRED
    offloading: bool = False,
    offload_only_non_sliding: bool = True,
    **kwargs,                    -- BC kwargs accepted silently
  )
  - config is MANDATORY (unlike DynamicCache where it is optional)
  - Pre-allocates StaticLayer tensors of shape (batch, num_heads, max_cache_len, head_dim)
  - is_compileable=True; for torch.compile use
  - StaticLayer.update uses index_copy_(dim=2, cache_position, new_states) — IN-PLACE
  - Does NOT re-allocate on each call; incremental update via cache_position indexing
  - Supports streaming without reallocation PROVIDED cache_position is passed correctly

get_seq_length(layer_idx=0) -> int:
  [StaticLayer.get_seq_length lines 346-350]
  - SLOW: checks (self.keys[0,0].any(dim=-1)).sum() — scans for non-zero rows
  - NOT a simple counter; costs O(max_cache_len * head_dim) per call
  - Avoid calling repeatedly during generation


Qwen2.5-Coder-3B MODEL DIMENSIONS (from local config.json, verified)
  num_hidden_layers: 36
  num_attention_heads: 16
  num_key_value_heads: 2  (GQA — only 2 KV heads, NOT 16)
  hidden_size: 2048
  head_dim: hidden_size // num_attention_heads = 2048 // 16 = 128
  max_position_embeddings: 32768
  use_sliding_window: false
  sliding_window: 32768 (effectively disabled since use_sliding_window=false)


MEMORY MATH CORRECTION
Prompt estimate: 36 layers × 16 KV-heads × 128 head-dim × 2 (K+V) × 2 bytes = 294 KB/token
ACTUAL (from config.json, num_key_value_heads=2):
  36 layers × 2 KV-heads × 128 head-dim × 2 (K+V) × 2 bytes (bfloat16) = 36,864 bytes = 36 KB/token
  The prompt's estimate is 8x too high due to using num_attention_heads=16 instead of num_key_value_heads=2.

KV cache dtype under NF4:
  server.py line 1192: bnb_4bit_compute_dtype=torch.bfloat16
  NF4 quantization applies ONLY to model weight storage.
  KV cache tensors are allocated at compute dtype = bfloat16.
  Confirmed by DynamicLayer.lazy_initialization: self.dtype = key_states.dtype
  key_states are produced by k_proj which runs at bfloat16 compute dtype.
  KV cache dtype = bfloat16 regardless of weight quantization scheme.


model.generate() INTEGRATION
[generation/utils.py verified]

Accepted kwargs (passed via **kwargs to generate):
  past_key_values: Optional[Cache] — accepted; generate reads get_seq_length() from it
  use_cache: bool — default True via GenerationConfig; controls whether cache is populated
  cache_position: NOT a direct kwarg to generate(); computed internally

How generate() handles past_key_values:
  _get_initial_cache_position (lines 1811-1837):
    cache_position starts as torch.arange(0, seq_len)
    If past_key_values is provided: past_length = cache.get_seq_length()
    cache_position is then sliced: cache_position = cache_position[past_length:]
    Result: cache_position covers only the NEW tokens, starting at past_length
    This is passed as model_inputs["cache_position"] to the forward pass.

attention_mask extension:
  _update_model_kwargs_for_generation (lines 959-1005):
    After each decode step, attention_mask is extended by 1 via:
      torch.cat([attention_mask, attention_mask.new_ones((batch, 1))], dim=-1)
    generate() handles this AUTOMATICALLY per decode step.
  For prefill with past_key_values: the caller must pass attention_mask of length
    (past_len + new_input_len) covering both the cached prefix AND the new tokens.
    generate() does NOT auto-pad the attention_mask to match a pre-filled cache.
    CRITICAL: if attention_mask covers only new_input_len tokens but cache has
    past_len tokens, the model will compute incorrect attention (causal mask mismatch).

cache_position in Qwen2 attention forward:
  [modeling_qwen2.py lines 160-163]
  cache_kwargs = {"sin": sin, "cos": cos, "cache_position": cache_position}
  key_states, value_states = past_key_values.update(key_states, value_states, layer_idx, cache_kwargs)
  For DynamicLayer: cache_position in cache_kwargs is ignored (DynamicLayer uses concat, not positional indexing)
  For StaticLayer: cache_position is REQUIRED and used for index_copy_


SDPA / FlashAttention compatibility with DynamicCache
  server.py line 1175: ELAI_FORCE_SDPA=1 sets _attn_impl = "sdpa"
  modeling_qwen2.py line 262: _supports_sdpa = True
  modeling_qwen2.py lines 166-167: attention_interface = ALL_ATTENTION_FUNCTIONS["sdpa"]
  DynamicCache has NO SDPA-specific code path restrictions in cache_utils.py.
  DynamicLayer.update returns standard (batch, num_heads, seq_len, head_dim) tensors
  which are compatible with torch.nn.functional.scaled_dot_product_attention.
  CONFIRMED: DynamicCache works with SDPA backend without modification.
  Note: flash_attention_2 has a known kv_length non-scalar mask bug in 4.57.6
  (server.py comment at line 1173); ELAI_FORCE_SDPA=1 is the workaround.
  StaticCache is_compileable=True but still works with SDPA (torch.compile optional).


VRAM RELEASE / EVICTION
  del cache on a DynamicCache removes the Python reference.
  Underlying tensors are torch.Tensor objects on GPU.
  Python GC will free them IF no other references exist.
  GPU memory is returned to PyTorch memory pool, NOT to OS immediately.
  torch.cuda.empty_cache() is needed to return memory to OS / make it visible in nvidia-smi.
  For reliable VRAM visibility after eviction: del cache; torch.cuda.empty_cache()
  No special eviction API exists on DynamicCache; it is plain Python object ownership.


BEHAVIORAL CONTRACTS (verified from source)

DynamicLayer.update -> cat-append to dim=-2; returns full accumulated K,V tensors
StaticLayer.update -> index_copy_ at cache_position; returns full pre-allocated K,V
Cache.update -> dispatches to layer.update; handles lazy layer creation when layer_class_to_replicate set
Cache.get_seq_length(layer_idx=0) -> delegates to layer.get_seq_length(); safe for empty cache
Cache.reorder_cache(beam_idx) -> index_select(0, beam_idx) on all layers; beam search support
Cache.crop(max_length) -> trims seq dim in-place on all layers
DynamicCache.to_legacy_cache() -> tuple of (keys, values) per layer
DynamicCache.from_legacy_cache(pkv) -> classmethod; reconstructs DynamicCache from tuple format
generate() -> auto-extends attention_mask by 1 token per decode step
generate() -> computes cache_position from get_seq_length() when past_key_values provided
generate() -> does NOT extend attention_mask for the prefill phase; caller must provide full mask


PLATFORM CONSTRAINTS

Python: 3.12 on remote sidecar (verified)
PyTorch: DynamicLayer.update uses torch.cat which allocates new memory each step (no in-place growth)
MPS fallback: StaticLayer.update has try/except NotImplementedError for index_copy_ on MPS
Thread safety: Cache has no locking; server.py uses threading.Lock() at generate() call level
Picklability: DynamicCache is picklable (plain Python object with Tensor fields)


GOTCHAS

1. DynamicCache with config=model.config on Qwen2.5-Coder-3B: sliding_window=32768 is present
   in config.json even though use_sliding_window=false. DynamicCache reads sliding_window
   attribute directly (not use_sliding_window). All 36 layers become DynamicSlidingWindowLayer.
   DynamicSlidingWindowLayer.crop raises ValueError if called after cache is full (>= sliding_window).
   Use config=None for DynamicCache unless sliding-window behavior is explicitly needed.

2. DynamicLayer.update return value: returns FULL cumulative keys/values (not just new tokens).
   Callers must NOT assume the return is only the new token's KV pair.

3. StaticCache.get_seq_length is O(max_cache_len) per call; do NOT use in hot path.

4. attention_mask for prefix reuse: if a DynamicCache is pre-filled with prefix tokens,
   the attention_mask passed to generate() or model.forward() MUST be length (prefix_len + new_len).
   generate() will NOT auto-pad it. A short mask causes incorrect causal attention.

5. Memory math: num_key_value_heads=2 (not 16) for Qwen2.5-Coder-3B. KV cache is 8x smaller
   than naive calculation using num_attention_heads. Per-token cost = 36 KB (bfloat16), not 294 KB.

6. NF4 quantization does NOT affect KV cache dtype; KV tensors are bfloat16 (compute dtype).

7. torch.cuda.empty_cache() is required after del cache to return VRAM to pool visibility.

8. DynamicSlidingWindowLayer.get_seq_length returns cumulative_length (total tokens seen),
   NOT the number of cached tokens. When cache is full (>= sliding_window), cached tokens
   = sliding_window - 1, but get_seq_length() returns the higher cumulative count.
   This affects cache_position calculation if using get_seq_length() for position offsets.


CRITICAL ASSERTIONS (testable)

assert isinstance(cache.update(k, v, 0), tuple) and len(cache.update(k, v, 0)) == 2
# Returns exactly (keys, values) 2-tuple [DynamicLayer.update lines 98-121]

assert returned_keys.shape == (batch, num_kv_heads, total_seq_len, head_dim)
# Full accumulated tensor returned, not just new tokens [DynamicLayer lines 119-121]

assert cache.get_seq_length(0) == returned_keys.shape[-2]
# get_seq_length == shape[-2] for DynamicLayer [lines 130-134]

assert returned_keys.dtype == key_states.dtype
# dtype preserved; bfloat16 for NF4 model [DynamicLayer.lazy_initialization line 93]

assert len(cache) == num_hidden_layers  # after full prefill
# Cache.__len__ returns len(self.layers) [line 901-907]

assert cache[0][0].shape == (batch, num_kv_heads, seq_len, head_dim)
# __getitem__ returns (keys, values) tuple [lines 881-891]


RECOMMENDATIONS FOR PrefixKVCache DESIGN

1. Use DynamicCache(config=None) not DynamicCache(config=model.config) for prefix caching.
   Reason: Qwen2.5-Coder-3B has sliding_window=32768 in config.json which causes all layers
   to become DynamicSlidingWindowLayer. DynamicSlidingWindowLayer.crop raises ValueError if
   the cache has seen more tokens than sliding_window, and its get_seq_length returns cumulative
   count (not cached count), breaking cache_position arithmetic. Plain DynamicLayer avoids this.

2. When passing a pre-filled DynamicCache to model.generate(), always construct attention_mask
   of shape (batch, prefix_len + new_input_len) before calling generate().
   generate() auto-extends attention_mask by 1 per decode step, but it does NOT extend it to
   cover a pre-filled prefix at prefill time. Missing prefix positions in attention_mask will
   corrupt causal attention scores.

3. The prompt's per-token KV cost estimate of 294 KB is incorrect for this model.
   Correct value: 36 KB/token (bfloat16, GQA with 2 KV heads, 36 layers, 128 head-dim).
   At 36 KB/token, a 2048-token prefix uses ~72 MB VRAM — well within the 3090's 24 GB.
   This makes prefix caching much more feasible than the original estimate suggested.

4. After evicting a DynamicCache, call torch.cuda.empty_cache() to return VRAM to the pool.
   del cache alone is insufficient for accurate VRAM accounting in nvidia-smi or torch.cuda.memory_reserved().

5. SDPA backend is fully compatible with DynamicCache in transformers 4.57.6.
   DynamicLayer returns standard (batch, num_kv_heads, seq_len, head_dim) tensors
   which scaled_dot_product_attention accepts directly. No special cache type is needed
   for SDPA; flash_attention_2 is explicitly avoided via ELAI_FORCE_SDPA=1 due to a
   known mask bug in 4.57.6 (server.py line 1173 comment).
