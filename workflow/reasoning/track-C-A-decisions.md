Track C-A — Decisions Document
Session: s-934436
Date: 2026-05-05
Predecessor handoff: workflow/sessions/s-155876/SESSION_HANDOFF.md
Context package: workflow/sessions/s-934436/context-package-track-C-A.md
Paper-ingestor map: workflow/papers/transformers-dynamic-cache-interface.md

WHAT
Implement PrefixKVCache (LRU 4 entries) in server.py to skip prefill on
identical/longest-prefix requests. Populate
usage.cache_read_input_tokens / usage.cache_creation_input_tokens in
the /v1/messages response based on auto-detected prefix hit. Leave
cache_control marker parsing for Track C-D. Leave streaming path
(generate_streaming) untouched.

KEY DECISIONS

1. DynamicCache(config=None) is MANDATORY (not config=model.config).
   Why: Qwen2.5-Coder-3B's config.json sets sliding_window=32768. With
   config=model.config, every layer becomes DynamicSlidingWindowLayer
   and crop() raises ValueError once cached tokens exceed the window.
   Verified in workflow/papers/transformers-dynamic-cache-interface.md.
   With config=None, all 36 layers become DynamicLayer (full attention)
   which is what we want for prefix caching.

2. Per-token KV cost: 36 KB (NOT 294 KB as the parent prompt assumed).
   Why: Qwen2.5-Coder-3B uses GQA with num_key_value_heads=2 (NOT 16).
   36 layers × 2 KV-heads × 128 head-dim × 2 (K+V) × 2 bytes (bf16) = 36 KB/token.
   Implication: 4 entries × 8K prefix = ~1.1 GB VRAM. Fits 12-24 GB box
   easily. NF4 weight quantization does NOT affect KV dtype (KVs are
   bf16 = compute_dtype).

3. Auto-disable PrefixKVCache when --turboquant-bits > 0.
   Why: server.py's _make_cache returns _KVQuantCache when TQ>0.
   A pre-filled DynamicCache from PrefixKVCache cannot be substituted
   for _KVQuantCache. Production sidecar runs --turboquant-bits 0
   (per CLAUDE.md) so this is a no-op in production but a guard for
   future TQ enablement. Auto-disable = log a warning and pass-through.

4. Hash key: blake3 of input_ids[:N] tokens, NOT raw bytes of the prompt.
   Why: identical prompts yielding different tokenization (eg trailing
   whitespace, BOS handling) would miss the cache; tokenizing the
   prompt makes the key canonical. Use blake3 (NOT blake2/sha2) per
   CLAUDE.md anti-stale-training rule.

5. Prefix walk strategy: greedy longest-match from the LRU.
   Why: a single dict-keyed-by-full-input-id-hash misses partial matches.
   Walk: tokenize the new prompt → for each cache entry, find the
   longest token prefix that matches → pick the entry with the longest
   match. Hash entries are keyed by blake3 of the full token sequence
   they were originally seeded with, but we additionally store the
   token tensor so we can compute prefix matches in O(prefix_len) per
   entry. With LRU=4 and prefix_len≤8K, this is cheap.

6. Eviction: del cache + torch.cuda.empty_cache().
   Why: paper-ingestor confirmed `del cache` alone doesn't return memory
   to the pool (only releases the Python ref). Without empty_cache(),
   nvidia-smi will not reflect the freed VRAM and the next allocation
   may OOM-flake. Call empty_cache() ONLY on eviction, not per-request
   (it stalls the kernel).

7. Cache size & flag: --kv-cache-size N, default 4. Range 0..16.
   N=0 disables the cache entirely (also auto-disabled for TQ>0).
   Setting on argparse next to --no-dola etc.

8. Integration site: _generate_impl (NOT generate_streaming).
   Why: streaming path has no past_key_values support today (context-
   compiler finding). C-A scope is non-streaming only. Streaming
   support is deferred (worth its own track).

9. Cache field accounting:
   cache_read_input_tokens = matched_prefix_token_count
   cache_creation_input_tokens = newly_admitted_token_count
     (i.e. the suffix after the prefix that we will store on this turn
     for next-turn reuse)
   Wired into _non_stream_messages_response (lines 2673-2674 today
   hardcode 0). Stream variant (2486-2487) left at 0 (C-A doesn't
   touch streaming).

10. NO cache_control marker parsing in C-A. C-D adds it as a refinement.
    Why: the auto-detected longest-prefix mode already delivers
    non-zero cache_*. C-D is the user-controlled override.

11. NO gg_signals surfacing in C-A. C-D bonus owns that.
    Why: C-D's bonus changes the same _non_stream_messages_response
    function. Two tracks editing same function = merge conflicts.
    C-A leaves the underscore-discard at line 2636 alone.

EXPECTED TEST OUTCOMES
- Python micro-bench .elai_cc/python/elai_inference/bench/prefix_cache_bench.py:
    5 sequential calls with same prefix; t1 cold, t2..t5 ≥5x faster.
- Cargo contract crates/contracts/tests/research_decompose_cache_hit.rs:
    second run of D5 cascade ≤ 0.4 × first run (≥2.5x speedup).
- /v1/messages on second identical call: usage.cache_read_input_tokens > 0.

UNKNOWNS / RISKS
1. Whether tokenizer's apply_chat_template is stable across calls
   (i.e. yields identical input_ids for identical messages). The
   prompt template may include date/time placeholders. Mitigation:
   the executor must read the apply_chat_template call site and
   confirm no time-dependent fields. If unstable, hash on the
   tokenized template output AFTER apply_chat_template, NOT the
   raw messages list.
2. Whether SDPA backend tolerates a non-zero cache_position arg on
   the first decode step. Paper-ingestor said yes; verify with the
   micro-bench.
3. Whether the LRU 4-entry table thrashes on the D5 cascade pattern
   (L0->L1->L2->babysteps). The cascade nests prefixes; a 4-entry
   LRU with longest-match should hold all four. Bench will tell.

PRE-IMPLEMENTATION TOP-3 FAILURE MODES
1. The pre-filled DynamicCache references GPU tensors that get
   re-used across requests; concurrent /v1/messages calls would race.
   Mitigation: server.py is already single-request-serialized via a
   FastAPI lock or per-call mutex; confirm in context package.
   If not serialized, wrap PrefixKVCache.lookup_and_lease() in an
   asyncio.Lock so two requests can't share the same DynamicCache.
2. Forgetting to construct attention_mask of full prefix_len +
   new_input_len. generate() does NOT auto-pad. Mitigation:
   PrefixKVCache.lease(prefix_ids, full_input_ids) returns
   (cache, attention_mask) where attention_mask = ones(1, full_input_len).
3. apply_chat_template producing different ids on different calls
   (system date/time injection). Mitigation: hash the post-template
   token sequence, not the messages list.

OUT OF SCOPE
- cache_control marker parsing (Track C-D)
- gg_signals.* surfacing (Track C-D bonus)
- Streaming path (deferred)
- StaticCache (DynamicCache fits, lower complexity)

NO commits. Auditor-only cargo + pytest. User collects + commits manually.
