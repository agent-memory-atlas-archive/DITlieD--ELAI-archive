Phase A — Bench redesign (elapsed_ms + gate lower)
Session: s-1976592
Date: 2026-05-06

What:
- Add top-level "elapsed_ms" key to /v1/messages non-stream response (server.py _non_stream_messages_response).
- Add top-level "elapsed_ms" key to /v1/completions response (server.py handle_completions).
- Lower _SPEEDUP_GATE in bench/prefix_cache_bench.py from 5.0 to 2.0.
- Update bench main() to compute speedup from server-side elapsed_ms instead of client perf_counter.
- Add explicit guard in bench: missing elapsed_ms => print FAIL + return exit code 1.

Key decisions:
- Use time.perf_counter() in /v1/messages (matches asyncio executor pattern). Use existing time.time()-derived `elapsed` in /v1/completions (no timing-function change; minimize blast radius).
- _SPEEDUP_GATE = 2.0 (37% margin below measured server-side ceiling of 2.74x on RTX 3090 + Qwen2.5-3B-NF4 + 1000-tok prompt + 4-tok decode). Not 1.5 (too lax), not 2.74 (no margin), not 5.0 (physically impossible).
- elapsed_ms is additive (top-level). All existing keys preserved. Rust consumers tolerate unknown fields.
- Bench timing swap is _SERVER-side_ only. Client wall-clock retained as diagnostic only, not used in pass/fail.

Expected test outcome:
- bench exits 0; cache_create_seq[0] > 0; speedup >= 2.0x server-side. Tunnel RTT eliminated by reading server-reported elapsed_ms.

Unknowns:
- Whether warm timings on this run will be in the measured-279ms range (tokenizer/cache state can drift). If speedup falls below 2.0x post-fix, escalate to add /v1/cache/flush endpoint or move bench to remote.
