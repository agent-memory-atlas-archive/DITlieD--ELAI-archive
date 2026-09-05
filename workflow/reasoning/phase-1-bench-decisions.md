Phase 1 — bench redesign decisions

What: Apply fix (b) + (d) from s-1430428 handoff §3.1 to .elai_cc/python/elai_inference/bench/prefix_cache_bench.py.

Key decision: Defer (c) /v1/cache/flush endpoint. PrefixKVCache.clear() exists at server.py:L1305 already; HTTP route is separate scope. Keep 5.0x threshold (verify_ca_live_bench_speedup.py from s-1430428 proves achievable at 1000-tok prompt + 4-tok decode). Use named constants — _PROMPT_REPS=35, _MAX_DECODE_TOKENS=4, _SPEEDUP_GATE=5.0 — no bare magic numbers in gate logic.

Expected test outcome:
- Gate 1: cache_create_seq[0] > 0 (cold-start prefill occurred; Qwen 48-tok system prefix partial-hit on t1 is now acceptable)
- Gate 2: any(cache_read_seq[1:] > 0) (warm cache engaged)
- Gate 3: t1_elapsed / mean(t2..t5) >= 5.0
- Exit 0, final line PASS.

Unknowns:
- Exact post-tokenization length of PROMPT after Qwen chat template prepend. If speedup falls to [4.5, 5.0), auditor bumps _PROMPT_REPS to 40 and reruns.
- Whether t1 prefill at ~1000 tokens contends with 25K KV-cache ceiling (no — PrefixKVCache caps total_tokens at 25K and bench prompts are well under).

Audit cycle budget: 1 (per user constraint).
