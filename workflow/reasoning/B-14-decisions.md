B-14 Live Token Streaming Chat — DECISIONS
Date: 2026-06-05 (backfill)
Source: workflow/sessions/s-plan-audit-2026-06-04/plan-status/B/B-14_live_token_streaming_chat.md

SHIPPED
- Streaming commands: start_chat_stream (commands.rs:5585) + cancel_chat_stream (:5931), registered
  in generate_handler! (lib.rs:754-755). SSE events emitted as chat-stream:{id} and
  chat-stream-end:{id} (app.emit() at commands.rs:5786/5808+).
- Production chain (not test-only): CommandWindow.svelte -> commands.svelte.ts submit() ->
  _runStreamingPath() -> streamChat() (stub.ts:1981) -> tryInvoke('start_chat_stream').
- Intent classifier: classify_intent (commands.rs:5343, lib.rs:747); frontend classifyIntent()
  (commands.svelte.ts:340) on every submit.

KEY DECISIONS
- Symbol renames vs plan: stream_chat -> start_chat_stream; IntentClassifier -> classify_intent.
  Functionally equivalent; the plan's contract names were the pre-implementation working names.
- VITE_ELAI_FRAME_PACER_ENABLE is the gate for a SECONDARY frame-paced renderer (frame_pacer.ts,
  commands.svelte.ts:271), not the core path. The raw (non-paced) streaming path is fully wired and
  unconditional; the pacer is an enhancement. Default-OFF.

DEFERRED / NOT DONE
- chat_message_streamed SSB action type: NOT implemented. The plan marked it optional (cross-window
  sync only); confirmed absent from ssb.rs + action_type_lock.rs. Non-blocking.

R9 / WIRE-DARK
- VITE_ELAI_FRAME_PACER_ENABLE was an unregistered default-OFF flag (the SHIPPED_WIRE_DARK verdict's
  sole cause). Now registered in wire-dark-allowlist.toml: type=experiment, owner=maintainer,
  expiry_date=2026-09-01, promotion_plan_id=B-14, smoke_test=crates/dashboard-v2/tests/streaming/
  frame_pacer.test.ts "FramePacer drains at the target cps within +/-10%". The env-gate-ON Vitest
  (flip the var and assert commands.svelte.ts selects the paced path) is the promotion follow-up.

NOTES
- All three wiring_contract_check FAILs were rename false negatives; core streaming verified wired.
