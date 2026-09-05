Phase 3 — streaming response_format 503 gate decisions

What: Replace warn-and-ignore at server.py L2854-2858 with a 503 short-circuit mirroring the non-stream C-B pattern at L2809-2824. Add Gate D to research_decompose_constrained.rs to lock the new behavior.

Key decisions:
- Status code 503 (mirror non-stream); error code "constrained_decoding_unavailable"; detail string mentions lm-format-enforcer.
- Gate fires when stream=True AND response_format is not None (regardless of _LMFE_AVAILABLE — the existing L2809 gate already handles LMFE-absent first; streaming+response_format is "feature not wired" so we 503 universally on this combo).
- Error code differs from non-stream: "constrained_streaming_unavailable" (vs "constrained_decoding_unavailable") — different root cause.
- Test extends research_decompose_constrained.rs (Gate D); no separate file. Reuses the hand-rolled HTTP/1.1 helper.
- Deploy server.py to remote via tar+ssh (rsync on Windows is not always available); restart sidecar with canonical boot args.

Expected test outcome:
- Gate D returns 503 with error="constrained_decoding_unavailable"
- Existing Gates A/B/C unaffected (no regression)
- Non-stream non-LMFE 503 path unchanged.

Unknowns:
- None material. Gate D is independent of LMFE install state — it tests the stream+response_format combo which is rejected regardless. Gate D requires ELAI_LIVE_SIDECAR=1 and runs against the patched sidecar.

Audit cycle budget: 1.
