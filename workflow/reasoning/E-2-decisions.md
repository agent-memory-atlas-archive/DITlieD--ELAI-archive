E-2 — local LLM speed tricks decision doc
Session: s-25120 completion-drive
Plan: docs/thefinalglance/E-plan/E-2_local_llm_speed_tricks.md
Date: 2026-05-26

SCOPE-LOCK SUMMARY

Ships three perceived/actual-latency improvements as a coordinated bundle:
  A. Dashboard frame-paced render (FramePacer) + SkeletonBubble + CursorSentinel + thinking-indicator + Cancel-button affordance.
  B. Sidecar prompt-lookup decoding kwarg + acceptance log line.
  C. Sidecar HF StaticCache prefix-KV reuse as a SIDE-BY-SIDE env-gated path next to the existing custom PrefixKVCache.
  D. Two SSB action types: StreamingTimingRecorded + UxPacingVoteRecorded (atomic 7-site cascade x2 + SCHEMA 147 -> 149 + TOTAL_VARIANTS 420 -> 422 + golden +2 rows).
  E. Settings rows + MonitorWindow histogram surface.
  F. Determinism + lossless test harnesses (Python; GPU-gated, RUNPOD-PENDING for live numbers).

DECISION-1 (HF STATICCACHE COEXISTENCE — HIGH SEVERITY, FROM EXPLORER DRIFT)
Question: replace the existing custom PrefixKVCache (server.py:1490) with HF StaticCache, layer on top, or run side-by-side ?
Answer: SIDE-BY-SIDE under a distinct env gate ELAI_HF_PREFIX_KV_REUSE (NOT ELAI_PREFIX_KV_REUSE; the latter name collides semantically with the existing default-on custom path).
Rationale:
  * The existing PrefixKVCache is the baseline always-on capability (B-14-era + earlier). Replacing it would regress already-shipped TTFT improvements.
  * Plan text said "HF StaticCache + manual prefix-KV reuse" without specifying coexistence. We interpret this conservatively: ship the HF path as an OPT-IN trial, keep the custom path as the always-on baseline.
  * The HF StaticCache path is mutually exclusive PER REQUEST with the custom path: when ELAI_HF_PREFIX_KV_REUSE=1 the request bypasses the custom lease/admit and uses the HF path; otherwise the custom path runs unchanged.
  * Decision doc per memory:project_qwen3b_baseline (3B+NF4+FA2 is the production target). Promotion to default-on requires the determinism harness to pass 3x consecutive on remote pythondev.

DECISION-2 (HF STATICCACHE config=None — CRITICAL PITFALL, FROM GROUNDING)
Question: do we pass `config=model.config` or `config=None` when constructing the StaticCache ?
Answer: config=None.
Rationale (grounded via paper-ingestor source-transformers-dynamic-cache, HIGH reliability):
  * Qwen2.5-Coder-3B's config carries sliding_window=32768, which triggers per-layer sliding-window cache layers when config=model.config is passed.
  * Sliding-window layers silently corrupt prefix KV (cache_position math diverges past the window boundary).
  * config=None forces a homogenous static cache layer per attention head — correct for prefix reuse on Qwen2.5-Coder.
  * Pitfall is sticky: surface to operators in the env-gate startup log line so future debug sessions don't re-litigate.

DECISION-3 (PROMPT-LOOKUP KWARG, DEFAULT N=10, OFF BY DEFAULT)
Question: default value for prompt_lookup_num_tokens; default on or off ?
Answer: env-driven via ELAI_PROMPT_LOOKUP_NUM_TOKENS; default value 10 per apoorvumang/prompt-lookup-decoding README (GROUNDED, .claude/papers/prompt-lookup-decoding-README.md); default OFF (env not set = behave as before).
Rationale:
  * README TLDR reports 2.4x average on summarization+QA (Mistral-7B A100 greedy); 2x-4x is the range, not a guaranteed multiplier.
  * Promotion to default-on requires >=30% pooled acceptance rate over one week per plan Task 3 + report Open Question 2.
  * Both call sites are touched (server.py:2064 streaming + :2273 non-streaming) to avoid telemetry drift.

DECISION-4 (BILGIHAN 30-50% CLAIM — SUPPORTING-ONLY TAG)
Question: cite Bilgihan ResearchGate 326858669 with the "30-50% perceived speed-up" headline ?
Answer: cite as SUPPORTING evidence ONLY; do NOT quote the 30-50% number in code comments, SSB payloads, or user-facing copy.
Rationale (per memory:feedback_paper_citation_verify):
  * Paper not ingested in workflow/papers/ or .claude/papers/.
  * 2 prior plans (B-40, B-42) cited bad arXiv IDs verbatim.
  * Grounding sweep (E-2_local_llm_speed_tricks-grounding.md) flagged this as UNGROUNDED — the primary UX-grounding claim cannot be confirmed as paper-measured.
  * UX values (cps target, skeleton height, thinking-indicator threshold) are derived from inline integration tests and Doherty threshold (well-established 1982 HCI result), NOT from the Bilgihan headline number.
  * Future ingestion: paper-ingestor mode=paper "Bilgihan skeleton screens perceived speed navigation". Until then SUPPORTING-only.

DECISION-5 (SCHEMA BUMP CASCADE PATTERN — 2-VARIANT ATOMIC)
Question: bump SCHEMA_VERSION 147 -> 149 (atomic 2-step) or 147 -> 148 -> 149 (staged) ?
Answer: atomic 147 -> 149.
Rationale (per D-30 SchemaBumpPerVariantPair precedent — see ssb.rs:1799-1825 D-30 comment block):
  * Both variants land in the same commit, in the same SsbActionType declaration order.
  * action_type_lock.rs TOTAL_VARIANTS bumps 420 -> 422 atomically; ALL_VARIANTS gets 2 new tail entries; exhaustive_match_guard +2 arms.
  * action_type_lock.golden adds 2 lines: streaming_timing_recorded after streaming_early_terminated (ordinal 0350 insertion shifts subsequent +1) and ux_pacing_vote_recorded after untrusted_memory_write_blocked (ordinal 0404 insertion shifts subsequent +1). After both inserts, subsequent ordinals shift +2.
  * Single comment block at the head of SCHEMA_VERSION block covers BOTH variants.

DECISION-6 (ENV-GATE NAMING — PATTERN-4 PRECEDENT)
Two env vars introduced default-OFF:
  ELAI_HF_PREFIX_KV_REUSE=1  -> enables HF StaticCache path per request (side-by-side with existing custom PrefixKVCache).
  ELAI_FRAME_PACER_ENABLE=1  -> opt-in for the dashboard FramePacer RAF buffer (default behaviour = pass-through token stream).
  ELAI_PROMPT_LOOKUP_NUM_TOKENS=N -> sets the integer kwarg when N > 0 (when unset or 0, kwarg is not passed; legacy generate path).
Per Pattern-4 D-cluster precedent: env-gated trials default OFF; flip default-on only after determinism+acceptance gates clear.

DECISION-7 (B-14 SHIPPED — NO CARRY-FORWARD)
Question: pre-author stream_chat Tauri handler ? (Context-package flagged as HIGH risk.)
Answer: NO. Explorer drift correction: B-14 IS shipped as commands.rs::start_chat_stream + cancel_chat_stream (lib.rs:501-502). The plan text's "stream_chat" is the TS wrapper name in stub.ts:1816 around the live Tauri commands. No carry-forward needed.
Telemetry wiring point: commands.rs:5658-5663 (stream-end emit on message_stop) is the single canonical SSB write site for StreamingTimingRecorded; the abort + drain paths also write the row but with stop_reason="aborted" or "error".

DECISION-8 (OBSERVABLE EFFECT — TELEMETRY COMPUTED DASHBOARD-SIDE)
Per explorer drift: timing data (TTFT, p50/p99 inter-chunk, total_ms, tokens_emitted) is computed in commands.rs:start_chat_stream from receive timestamps, NOT in the sidecar. Avoids one HTTP round-trip and aligns with the existing single-writer SsbWriter pattern. The sidecar telemetry only fields the request — the dashboard owns the metric collection.

DECISION-9 (CANCEL BUTTON — REINFORCE EXISTING B-14 PATH)
Plan Task 6 calls out a Cancel button on the bubble for perceived-latency relief. CommandWindow.svelte already has a Stop button wired to commands.cancelStream() (commands.svelte.ts:395 -> cancel_stream -> commands::cancel_chat_stream at lib.rs:502). No new IPC; no new Tauri command. The improvement is purely CSS + reinforcing the existing surface (always-visible during streaming, no shrink-on-hover).

DECISION-10 (UX-VOTE SSB ROW + SETTINGS A/B BUTTON)
Plan Task 10 needs persistence of self-reported "snappy/slow" votes. UxPacingVoteRecorded payload:
  request_id: String,
  vote: String,                   // "snappy" or "slow"
  frame_pace_enabled_at_vote: bool,
  cps_at_vote: u32,
  effort_level: String,
  ts_ms: i64,
Settings A/B button mounts in SettingsWindow under the new "streaming" section. Default mode: opt-in toggle (no random session assignment in V1 to keep the change minimal; N=5 self-rating sessions per plan is enough to confirm direction).

DECISION-11 (RUNPOD-PENDING ITEMS — EXPLICIT LIST)
Items requiring live GPU inference for confirmation; harness shipped, numbers RUNPOD-PENDING:
  * Determinism test for HF StaticCache prefix reuse (test_prefix_kv_determinism.py) — needs Qwen2.5-Coder-3B NF4 on remote pythondev; 3x consecutive byte-equivalence required before default-on.
  * Lossless test for prompt-lookup decoding (test_prompt_lookup_lossless.py) — temp=0 greedy byte-equivalence across 5 prompts.
  * P50/p99 inter-chunk telemetry baseline (one week minimum on remote 3090) before promoting FramePacer to default-on.
  * Bilgihan paper ingestion via paper-ingestor mode=paper (background; not blocking).

OUT-OF-SCOPE (REJECTED OR DEFERRED, NO IMPLEMENTATION)
  * EAGLE-3 (arXiv:2503.01840) — deferred to 32B/48GB tier per report; draft-head VRAM competes with KV on 12GB.
  * Medusa (arXiv:2401.10774) — draft tokens bypass DoLa contrast (DoLa already disabled on 7B but Medusa is a permanent architectural commit; not adopted).
  * Lookahead Decoding (arXiv:2402.02057) — would replace the generate loop and complicate LogitsProcessor placement.
  * REST (arXiv:2311.08252) — lower integration cost via prompt-lookup is preferred.
  * KIVI 2-bit KV quant (arXiv:2402.02750) — deferred to context >25K E-N.
  * TurboQuant (arXiv:2504.19874) — on 32B roadmap; not adopted here.
  * vLLM / SGLang / TensorRT-LLM migration — forfeits LogitsProcessor chain per CLAUDE.md hard rule.
  * Skeleton-of-Thought — breaks single-stream UX + DoLa contrasts.
  * MoE swap — CLAUDE.md hard rule.

PROMOTION GATES (post-merge, RUNPOD-PENDING)
  G1. ELAI_HF_PREFIX_KV_REUSE -> default-on once determinism harness passes 3x consecutive on remote 3090.
  G2. ELAI_PROMPT_LOOKUP_NUM_TOKENS=10 -> default-on once pooled acceptance >= 30% over 1 week of telemetry.
  G3. ELAI_FRAME_PACER_ENABLE -> default-on once p50 inter-chunk delta confirmed >100ms over 1 week.

PAPER LABELS (per memory:feedback_paper_citation_verify discipline)
  * apoorvumang/prompt-lookup-decoding README — DIRECT GROUNDING for prompt_lookup_num_tokens kwarg + 2x-4x range (.claude/papers/prompt-lookup-decoding-README.md).
  * source-transformers-dynamic-cache (RAG ingest, HIGH reliability) — DIRECT GROUNDING for StaticCache pitfall (config=None on sliding-window models).
  * Doherty threshold (Doherty + Kelisky, IBM Research 1982) — CONVENTION GROUNDING for the 200ms thinking-indicator threshold.
  * Bilgihan ResearchGate 326858669 — SUPPORTING ONLY (not paper-measured; ingestion deferred; 30-50% headline not quoted in code).
  * Vercel AI SDK 4.1 smoothStream — VENDOR REFERENCE (informs the FramePacer design; smoothStream itself not depended on).

end of E-2-decisions.md.
