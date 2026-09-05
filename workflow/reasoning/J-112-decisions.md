# J-112 decisions — the voice output plane (render-table-not-LLM; queue; deferred legs)

## Verdict per task (2026-08-09)

| task | status | evidence |
|------|--------|----------|
| 1 render table (`render_spoken`) | SHIPPED | slice 1 commit d25543a; voice/spoken_render.rs |
| 2 spoken target on NotificationDispatcher | SHIPPED | slice 1 commit d25543a; notify_dispatch.rs surfaces bit 2 via ELAI_VOICE_ENABLE |
| 3 narration queue + levels | SHIPPED | slice 2 commit 240e42d; voice/narration_queue.rs (7 unit tests) |
| 4 python/elai_voice TTS leg | DEFER (owner-gated V-7) | new deps need USER CONFIRM; not started |
| 5 on-demand spoken digest lane | FUTURE SLICE | model-on-event-path; receipts + deterministic footer; not started |
| 6 this decisions doc | DONE | this file |

Gate-config (the ELAI_VOICE_ENABLE R9 allowlist row) shipped ahead of the product slices
as commit 4e4a4c9 with a freeze-baseline re-lock (the post-merge gate_freeze_check blocks
an allowlist mutation mid-land; proven K-12/J-93 procedure).

## The decisions

### D1 — render-table-not-LLM (V-9)
The narrated text NEVER passes through a model. `render_spoken(kind, session_id, ts_ms)`
returns one of 2-3 deterministic seeded variants per family (Completion, NeedsInput). A
model appears in exactly one slot (the deferred digest lane, task 5), and even there its
text is display-layer-bound + receipt-anchored + footer-counterweighted. This is the moat:
ELAI's narrator is incapable of inventing state because it renders journal rows.

### D2 — one supervisor voice + earcons (V-8), entropy quarantine (V-7)
Variant selection is a seeded hash of (session_id, ts_ms) via DefaultHasher -- NO
thread_rng, NO ambient randomness. Same (session_id, ts_ms) -> same variant, always. The
queue (slice 2) inherits this: it is a pure ordering structure with no randomness surface.

### D3 — queue: priority + step-grain + verbosity (WARGAME MOVE 2)
The crowded-room problem is scheduling, not voice. NarrationQueue (slice 2):
- priority: NeedsInput > Completion > Progress (BTreeMap reverse-drain);
- progress coalesces at (session_id, step_id) grain -- J-106 posture, never per-tool-call;
  distinct steps are both kept; a drained step may re-enqueue (correct step-grain semantics);
- verbosity silent/summary/verbose as row-class subscription filters (Verbosity::admits).

### D4 — N-7 forward-compat (V-3)
`render_spoken` is a pure `fn(row) -> String`. When N-7 ships render(verdict, target),
this table MIGRATES into `target=spoken`; no second dispatcher. Commitment recorded; the
current flat-table shape is the pre-migration form. The contract test's byte-equality arm
moves to the pre-normalizer text if E-3 later adds a model-phrasing pass (V-9 bound).

### D5 — footer-outside-model-text (the digest-lane tripwire, task 5, FUTURE)
NOT YET BUILT. The design commitment for when task 5 lands: the spoken digest ends with a
DETERMINISTIC footer (N sessions / N events / N pending decisions) computed off the SSB
rows by the harness, appended OUTSIDE the model text, spoken last. Every model-text claim
carries a row-id receipt. A summary that dropped a session contradicts its own footer
audibly. Footer is NEVER derived from model output (no self-grading).

### D6 — the shared ELAI_VOICE_ENABLE flag (R9, wave-owned)
One flag gates the whole voice surface (J-111 will share it). Default-OFF. The
allowlist row is wave-owned, authored in J-112 (commit 4e4a4c9). Fail-toward-escape-only:
absent = no spoken target; terminal escape + OS notification + SSB row fire per their own
gates unchanged.

## What is NOT done (and why that is correct)
- TTS (task 4): owner-gated on new deps (Kokoro-82M / Piper, V-7 anti-slopsquat + USER
  CONFIRM). The render text artifact IS the record; TTS is a transport. OS-native (SAPI/say)
  is the degraded leg if the user declines new deps.
- Digest lane (task 5): the most care-intensive slice (model on the path, receipts,
  footer). Ships as its own slice with its own contract test; do NOT rush it onto the
  event path without the footer tripwire.
- Dispatcher -> queue WIRING: slice 2 ships the scheduler as a self-contained module;
  a later wiring slice calls NarrationQueue from notify_dispatch (the dispatcher today
  renders + writes the SSB row directly; the queue slots in front of the spoken leg).

## Residual risk (GD-5, accepted)
TTS mispronunciation can garble correct text (audio fidelity != text fidelity). Accepted:
the text artifact is the record; the dashboard shows it. The cant-lie arm (byte-equality
row -> payload) holds regardless of audio fidelity.
