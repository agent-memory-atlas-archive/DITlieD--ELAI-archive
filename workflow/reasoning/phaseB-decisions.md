Phase B — Streaming + LMFE response_format wiring
Session: s-1976592
Date: 2026-05-06

What:
- Add 2 module-level error-code constants in server.py: _ERR_CONSTRAINED_DECODING_UNAVAILABLE, _ERR_CONSTRAINED_STREAMING_UNAVAILABLE.
- Replace 4 raw "constrained_*_unavailable" string literals with the new constants (also closes Phase C / §3.5 in same edit set).
- generate_streaming gains response_format kwarg; build prefix_fn via existing _build_constrained_prefix_fn before _thread_pool.submit; add prefix_allowed_tokens_fn to generation_kwargs when not None.
- _stream_messages_response gains response_format kwarg; threads to generate_streaming.
- handle_messages stream branch: replace unconditional 503 with conditional — LMFE-absent => 503 _ERR_CONSTRAINED_DECODING_UNAVAILABLE; LMFE-present => fall through to _stream_messages_response with response_format threaded.
- research_decompose_constrained.rs: replace fn stream_response_format_returns_503 with stream_response_format_constrained_sse_when_lmfe_installed. Guard on lmfe_required_ready; assert 200 + SSE Content-Type + at least one non-empty data event.

Key decisions:
- Reuse _build_constrained_prefix_fn as-is (single source of truth, handles all LMFE/schema validation).
- prefix_fn built in main thread BEFORE _thread_pool.submit — TokenEnforcer not thread-safe (paper §83).
- Unify error code: stream and non-stream paths now both use _ERR_CONSTRAINED_DECODING_UNAVAILABLE for LMFE-missing case (consolidates the inconsistent "constrained_streaming_unavailable" / "constrained_decoding_unavailable" split flagged in PART5 §3.5).
- _ERR_CONSTRAINED_STREAMING_UNAVAILABLE retained as a defined-but-unused constant — reserved for future use cases where streaming is unavailable for reasons OTHER than missing LMFE library (e.g., transient SSE-generator errors). Keeps the constant set complete and discoverable.

Expected test outcome:
- cargo check -p elai_cc_contracts: clean.
- ELAI_LIVE_SIDECAR=1 ELAI_LMFE_INSTALLED=1 cargo test ... research_decompose_constrained: 1/1 PASS on the new SSE assertion.
- Live curl with stream=true + response_format + valid schema returns 200 SSE.
- Backward compat: stream=true with no response_format unchanged.

Unknowns:
- Whether the SSE generator's drain pool interacts cleanly with prefix_fn-constrained generation (model.generate output ordering should be deterministic; the streamer should not see torn JSON tokens since LMFE constrains at logits-time).
- Whether the existing Gate B ("response_format unset still streams normally") still passes — must verify by running the full test file.
