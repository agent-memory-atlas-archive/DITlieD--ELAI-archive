Phase 1F Step 2: Ollama Speculative Decoding API Availability Check
Date: 2026-04-07
Hardware: RTX 3060 12GB, Windows 11
Ollama version: 0.20.2
Main model: gemma4:e2b (5.1B, Q4_K_M, 7.9GB loaded)
Draft model candidate: gemma3:4b (4.3B, Q4_K_M)

DETERMINATION: NOT_SUPPORTED

Evidence:
1. PR #8134 (speculative decoding for ollama) CLOSED Jan 9, 2025 due to merge conflicts with backend rewrite.
   Source: https://github.com/ollama/ollama/pull/8134
2. Issue #5800 (Enable speculative decoding) remains OPEN with no implementation merged.
   Source: https://github.com/ollama/ollama/issues/5800
3. Issue #9216 closed as duplicate of #5800 (May 16, 2025). No newer PR exists.
   Source: https://github.com/ollama/ollama/issues/9216
4. Ollama API docs (/api/generate, /api/chat) do NOT document draft_model, speculative, draft, or num_draft parameters.
   Source: https://github.com/ollama/ollama/blob/main/docs/api.md
5. Ollama CLI (--help, run --help) has no --draft flag.
6. Live probe: sent /api/generate with draft_model=gemma3:4b and num_draft=5 in options.
   Ollama silently ignored both parameters. Response structure identical to baseline.
   Baseline keys: [context, created_at, done, done_reason, load_duration, model, prompt_eval_count, prompt_eval_duration]
   Draft keys: identical.
   Baseline output: "1, 2, 3, 4, 5"
   Draft output: "1, 2, 3, 4, 5"
   No draft-specific keys (draft_tokens_accepted, etc.) in response.

Fallback evaluation (per phase-1F.txt L116-119):
1. llama.cpp direct: NOT available. llama-server/llama-cli not installed on system.
   Even if installed, our architecture uses ollama as the provider interface.
   Adding a llama.cpp direct provider is out of scope for Phase 1.
2. vLLM: NOT available until Phase 2+ (per providers.txt L55).

Decision: Phase 1F terminates at step 1F.2 with NOT_AVAILABLE status.
Steps 1F.3-1F.5 are SKIPPED.
Per phase-1F.txt L248-249: "If NOT_AVAILABLE (from 1F.2): DROP. Document speculative decoding not available on current stack as of [date]."

DROP: speculative decoding not available on current stack as of 2026-04-07.
Re-evaluate in Phase 2 when vLLM integration exists, or when ollama merges speculative decoding support.
VRAM budget (4.8GB headroom after gemma4:e2b) remains available for embedding models (Phase 1C vector search).

Probe implementation: crates/providers/src/providers/ollama.rs
  ProbeResult enum (Supported | NotSupported) with serde support
  probe_speculative_support() async fn — tests version, baseline vs draft API calls, detects silent ignore
  9 unit tests (all pass), 1 integration test (pass against live ollama)
