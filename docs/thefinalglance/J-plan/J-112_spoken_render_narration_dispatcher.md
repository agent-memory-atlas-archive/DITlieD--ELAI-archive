# J-112: spoken render + narration dispatcher — the voice output plane: every utterance is a rendered SSB row; the narrator cannot lie

STATUS: PLANNED (WAVE VOICE-1; M; BUILD-on-shipped — notify_dispatch.rs is live (V-1); ships FIRST in the wave, J-111 consumes it)
PROOF_SPINE: none (product-surface / voice-operator class; user-acked 2026-07-10, V-10)

Objective: the harness speaks — completion/needs-input alerts, step-grain progress, and on-demand cross-session
status — with the truth path deterministic end to end: a spoken-render table maps each narratable SSB event
family to a one-line template rendered off the row's typed fields, dispatched as a new spoken target on the
SHIPPED H-36 NotificationDispatcher (V-1), synthesized by a local TTS leg. A model appears in exactly ONE slot:
the on-demand digest lane, where it summarizes typed per-session digests and every spoken claim carries row-id
receipts plus a DETERMINISTIC footer (counts computed off the rows, appended outside the model text) so a
summary that dropped a session contradicts its own footer audibly. Hard boundaries: NO LLM-authored narration
text on the event path (V-9); NO second render dispatcher once N-7 ships (V-3 — this table MIGRATES into
render(verdict, target=spoken)); NO per-session voices (V-8 — one supervisor voice + one system earcon set);
NO unseeded variance (entropy quarantine). GD-1 do NOT re-implement: NotificationDispatcher + the
agent_notification_emitted family (H-36, V-1), the E-3 VoiceAnchor shape (V-4), H-51 topic digests, J-106
act-don't-narrate posture (applied here to the SPEAKER's rate limit).

Estimated effort: ~1.5 sessions (M). The render table + queue are mechanical; the digest lane's receipts+footer
and the negative arms are the care points.
Pre-requisites (re-grep, W-7): notify_dispatch.rs::NotificationDispatcher :57 + its dispatch_stop/RequestHuman
trigger sites (V-1, SHIPPED); the H-36 verbosity gap (dashboard NotificationManager needs-input leg). Soft:
E-3 voice.toml (fixture anchor until it ships, V-4); N-7 (forward-compat shape only, V-3); H-51 digests.
Parallel-safe with: J-109/J-110 (disjoint files). J-111 depends on THIS unit.
Source: the voice-control transcript Parts 3-4 + the 2026-07-10 spec-eval session; _grounding/J-VOICE-GROUNDING-MAP.md.
Doctrine: J-0_INDEX.md (W/GD/HD/TD/ID inherited; ADDS no delta family).

## Context
Why this matters: the transcript's strongest conclusion (V-8) is that the voice supervisor is the HARNESS
narrating its own journal — "an audio dashboard": same rows the 4K dashboard reads, different render. That makes
the differentiator structural: competitors bolt a chat voice onto a model; ELAI's narrator is incapable of
inventing state because narrated text never passes through a model — the same argument the H canvas makes over
its journaled op-stream. The "personality" the user wants (transcript Part 4: "not robotic, a little
personality") is a PHRASING layer: template families keyed off the E-3 voice anchor with seeded variants —
deterministic data, safe phrasing, zero fact authority. The crowded-room problem is scheduling, not voice:
priority classes (needs-input preempts progress), per-session step-grain rate limiting, and the three verbosity
levels the user asked for (silent / summary / verbose) as deterministic row-class subscription filters.

## Existing state (V-IDs)
- NOTIFICATION DISPATCHER [SHIPPED] notify_dispatch.rs::NotificationDispatcher :57, triggered from dispatch_stop + RequestHuman, emits agent_notification_emitted (V-1). THE seam — J-112 adds a target, not a dispatcher.
- RENDER SUBSTRATE [PLANNED] N-7 render(verdict,target) (V-3) — author forward-compatible, migrate on ship.
- VOICE ANCHOR [PLANNED] E-3 voice.toml (V-4) — fixture anchor with the same TOML shape until E-3 lands.
- DIGESTS [PLANNED/PARTIAL] H-51 sleep-time topic digests — the digest lane's input shape; re-grep at edit.
- ENTROPY QUARANTINE [SHIPPED, Phase-0] — variant selection seeds from row id via EntropySource, never thread_rng.

## Reads (mandatory before coding)
1. _grounding/J-VOICE-GROUNDING-MAP.md (all V-IDs).
2. notify_dispatch.rs IN FULL (V-1: the target dispatch shape, the OSC9/BEL/OS-notify siblings to mirror).
3. H-36 entry (H-SUMMARY_INDEX.md:574) — the trigger sites + the R9 posture to inherit; N-7_human_render_verdict_field.md (the shape to stay compatible with).
4. E-3_model_handoff_ux.md tasks 1+8 (voice.toml shape; the display-layer-only normalizer pattern that bounds any future model-phrasing leg).
5. CLAUDE.md hard rules: W-2/R9, D-030, GD-2, TD-3, no .unwrap, anti-slopsquatting (TTS deps, V-7).

## Tasks (WAVE VOICE-1 — ships first)
1. crates/orchestrator/src/voice/spoken_render.rs (~180 LOC, NEW): the spoken-render table — per narratable event family (completion, needs-input, phase/step progress, audit verdict, cost ceiling, gate block), a deterministic template over the row's typed fields; 2-3 variants per family selected by seeded hash of row id (entropy quarantine); phrasing constants keyed off the voice-anchor TOML (fixture until E-3, V-4). N-7-forward-compat: pure fn render_spoken(row) -> String, registered per family — the target=spoken leg N-7 later absorbs (V-3). A family with no declared render = compile-time exhaustiveness error for the narratable set, not a silent skip (R1 posture).
2. Spoken target on NotificationDispatcher (~60 LOC in notify_dispatch.rs): mirror the OS-notify target; payload = render_spoken(row); emits agent_notification_emitted with target=spoken (+Kind, reuse — V-6 net 0..+1). Env gate ELAI_VOICE_ENABLE shared with J-111 (one flag for the voice surface; R9 allowlist row authored HERE, wave-owned).
3. Narration queue + levels (~90 LOC): priority classes (needs-input > completion > progress), per-session progress rate limit at STEP granularity (J-106 posture — never per-tool-call), verbosity levels silent/summary/verbose as row-class subscription filters (deterministic config, the transcript's notification-levels ask V-8). Queue state journals nothing new; utterances themselves are the rows.
4. python/elai_voice/ TTS leg (~120 LOC py): local synthesis (Kokoro-82M primary candidate / Piper fallback, V-7 — anti-slopsquat + USER CONFIRM at edit); one supervisor voice + a distinct system earcon set (V-8); receives render text over the same localhost socket J-111 will share.
5. On-demand spoken digest lane (~80 LOC): "what's the status" (a J-32 CHAT/status lane when J-32 lands; until then an `elai voice --status` CLI arm as the production entrypoint): the model reads typed per-session digests (H-51 shape), output carries row-id receipts; a DETERMINISTIC footer (N sessions / N events / N pending decisions computed off the rows) is appended OUTSIDE the model text and spoken last — the audible drift tripwire. Model text is display-layer (E-3 normalizer bound, V-9): it may phrase, it may not assert a fact with no receipt row-id.
6. workflow/reasoning/J-112-decisions.md (render-table-not-LLM; one-voice+earcons; footer-outside-model-text; N-7 migration commitment; the shared ELAI_VOICE_ENABLE flag).

## WARGAME
MOVE 1 render table + spoken target (tasks 1-2).
EXPECTED_OBSERVATION: driving a run to completion emits agent_notification_emitted{target=spoken} whose payload text byte-equals render_spoken(completion row); the TTS artifact speaks exactly that text.
ON_FAIL: payload diverges from render_spoken(row) -> something else authored the text (the lie channel); counter-move: the dispatcher calls render_spoken directly, the contract test asserts byte-equality row->payload->TTS-input.
MOVE 2 queue + levels (task 3).
EXPECTED_OBSERVATION: needs-input fixture preempts a queued progress utterance; verbose->summary flips silence the progress class; per-session progress never exceeds step grain.
ON_FAIL: progress chatter floods (the crowded room) -> counter-move: rate-limit keyed on (session, step-id), drop-and-coalesce duplicates, assert in the test.
MOVE 3 digest lane (task 5).
EXPECTED_OBSERVATION: the spoken digest ends with the deterministic footer; every model-text claim carries a row-id receipt; a fixture where one session's digest is withheld makes the footer contradict the body (count mismatch surfaces).
ON_FAIL: footer computed from the model's own output (self-grading) -> counter-move: footer computed off the SSB rows by the harness, concatenated after, never shown to the model.
FORK: N-7 ships before execution -> implement AS the N-7 target=spoken leg directly (trigger = N-7 render dispatcher exists at edit time); else the flat table + the migration note.
FORK: E-3 ships before execution -> key phrasing off the real voice.toml; else the fixture anchor (same shape).
FORKS on TTS dep: user declines new deps -> OS-native TTS (SAPI/say) as the degraded leg, same render text; trigger = the task-4 confirm.
RECON_NEEDED: the narratable event-family set v1 (completion/needs-input are V-1-grounded; the progress + audit + cost families need a re-grep of the live action_type list at edit, V-6).
RECON_NEEDED: H-51 digest artifact shape live status (re-grep; if absent, the digest lane reads the per-session tail of the journal directly — deterministic fold, more tokens, same receipts rule).
ABORT: any utterance path whose text does not originate in render_spoken(row) or a receipt-bound digest; any second dispatcher after N-7 ships; any per-session voice differentiation.
VERIFY: DETERMINISTIC_CHECK below + the contract test; pass artifacts = SSB rows + byte-equality assertions + checker exit codes (off-agent, R4).
REDTEAM (attacks run at authoring): (a) "the narrator sounds alive but drifted from state" — killed by MOVE 1 byte-equality + the MOVE 3 footer tripwire. (b) "summaries become a second source of truth" — killed by receipts + footer-outside-model + the E-3 display-layer bound (V-9). (c) "narration floods and the user disables the whole surface" — killed by MOVE 2 priority/rate/levels. Surviving residue (GD-5): TTS mispronunciation can garble a correct text (audio fidelity is not text fidelity); accepted — the text artifact is the record, the dashboard shows it.
INTEGRATION_SMOKE: rides the wave smoke specced in J-111 task 6 (spoken command -> ... -> spoken narration). Solo-unit smoke: the --status arm + a completion drive, both asserted off rows.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/voice/spoken_render.rs (the per-family deterministic render table + seeded variants)
  - the spoken target in crates/orchestrator/src/notify_dispatch.rs (+Kind on agent_notification_emitted)
  - the narration queue/levels + the `elai voice --status` digest arm + python/elai_voice/ TTS leg
consumes_symbols:
  - crates/orchestrator/src/notify_dispatch.rs::NotificationDispatcher :57 + dispatch_stop/RequestHuman triggers (V-1; ADD a target, never a dispatcher)
  - the SSB journal read-side (the narratable families; V-6 re-derive at edit)
  - E-3 voice.toml anchor shape (fixture until shipped, V-4); H-51 digests (or the journal-tail fold, RECON)
reachable_from:
  - dispatch_stop / RequestHuman -> NotificationDispatcher -> spoken target -> TTS [PRODUCTION notification path, SHIPPED seam V-1]
  - crates/cli `elai voice --status` -> digest lane [PRODUCTION status arm]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO LLM-authored narration text on the event path (V-9); NO utterance without a backing SSB row
  - NO footer derived from model output; NO digest claim without a row-id receipt
  - NO second render dispatcher post-N-7; NO per-session voices; NO thread_rng variance; NO .unwrap on prod paths
frontend_pair:
  the dashboard NotificationManager gains the spoken-target rows in its feed (the H-36 needs-input no-op leg is
  already H-36's own deliverable; J-112 reads the same rows). Minimal Tauri read of target=spoken rows. Justified.
observable_effect:
  trigger: "ELAI_VOICE_ENABLE=1: (a) drive a real `elai run` to completion; (b) drive it into a RequestHuman needs-input; (c) elai voice --status"
  expect_ssb: "agent_notification_emitted{target=spoken} rows the feature agent did not author, payload byte-equal to render_spoken(row); the digest arm emits its receipts (row ids) in the journaled utterance record"
  expect_artifact: "the TTS text artifact per utterance; the digest ends with the deterministic footer; NEGATIVE (must FAIL): an utterance text with no generating row; a digest whose footer counts mismatch its body sessions; a progress utterance below step grain"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-112_spoken_render_narration_dispatcher.md --run-trigger
rg "render_spoken|target=spoken|SpokenTarget" .elai_cc/crates/orchestrator/src/voice/spoken_render.rs .elai_cc/crates/orchestrator/src/notify_dispatch.rs   # the render + target legs
rg -c "thread_rng|rand::random" .elai_cc/crates/orchestrator/src/voice/   # MUST be 0 (entropy quarantine)
rg -c "generate|complete|chat" .elai_cc/crates/orchestrator/src/voice/spoken_render.rs   # MUST be 0 model calls on the event path (V-9)
python .claude/tools/production_caller_check.py --files .elai_cc/crates/orchestrator/src/voice/spoken_render.rs   # 0-DARK (D-030; the dispatcher target is the caller)
rg "ELAI_VOICE_ENABLE" .elai_cc/wire-dark-allowlist.toml   # the R9 row exists (wave-owned, authored here)
test -f workflow/reasoning/J-112-decisions.md
```

## ACCEPTANCE
- [ ] every spoken utterance's text originates in render_spoken(row) or the receipt-bound digest; byte-equality asserted (the cant-lie arm).
- [ ] the spoken target rides the SHIPPED NotificationDispatcher (V-1) — no new dispatcher; N-7 migration commitment recorded.
- [ ] priority preemption + step-grain rate limit + silent/summary/verbose filters behave per the contract test.
- [ ] digest lane: receipts on every claim + the deterministic footer appended outside model text.
- [ ] one supervisor voice + earcons; ELAI_VOICE_ENABLE default-OFF with complete allowlist row (R9); TTS deps anti-slopsquat-verified + user-confirmed.
- [ ] cargo check GREEN; auditor PASS -p orchestrator (+ the python TTS smoke); solo smoke run off-agent.

## Tests to add
- crates/contracts/tests/j112_spoken_render_contract.rs (~130 LOC): (a) completion row -> spoken payload byte-equals render_spoken(row); (b) an utterance with no backing row is unconstructible (negative arm); (c) needs-input preempts progress in the queue; (d) rate limit holds at step grain; (e) verbosity filters are row-class-exact; (f) variant selection is seeded (same row id -> same variant across runs); (g) digest footer counts derive from rows, mismatch fixture surfaces; (h) missing digest artifact degrades to journal-tail fold, never silence-as-success (R1/R2).

## Files touched
- CREATE: .elai_cc/crates/orchestrator/src/voice/spoken_render.rs (~270 LOC incl queue/levels)
- MODIFY: .elai_cc/crates/orchestrator/src/notify_dispatch.rs (~60 LOC: the spoken target; PRODUCTION_HOOK, D-030)
- MODIFY: .elai_cc/crates/cli (the `elai voice --status` arm, ~40 LOC)
- CREATE: .elai_cc/python/elai_voice/tts.py (+ pyproject deps, user-confirmed) (~120 LOC)
- MODIFY: .elai_cc/wire-dark-allowlist.toml (ELAI_VOICE_ENABLE row, wave-owned)
- MODIFY: ssb action_type + golden lockstep ONLY IF the +Kind requires it (V-6; net 0..+1, re-derive at edit)
- CREATE: crates/contracts/tests/j112_spoken_render_contract.rs (~130 LOC)
- CREATE: workflow/reasoning/J-112-decisions.md

## Citations / Grounding
- _grounding/J-VOICE-GROUNDING-MAP.md V-1 (the SHIPPED seam), V-3/V-4 (forward-compat shapes), V-6..V-10.
- The voice-control transcript Parts 3-4 (event-narrator conclusion, notification levels, personality-as-phrasing — user decisions V-8).
- H-36 entry (H-SUMMARY_INDEX.md:574-581); N-7_human_render_verdict_field.md; E-3_model_handoff_ux.md; CLAUDE.md W-2/R9, D-030, GD-2, TD-3.

## Open questions / risks
- The narratable-family set v1 is a curation call: start with the H-36 pair + phase/step progress + audit verdict + cost ceiling; widen by usage, never by default (each family added = a declared render, the R1 exhaustiveness arm).
- Latency of local TTS on long summaries: chunk at sentence boundaries and stream synthesis (V-7 sub-0.3s class makes this comfortable); never block the dispatcher on synthesis.
- If E-3's persona work later adds a model-phrasing pass, it is BOUND to the display layer (V-9); the row->payload byte-equality assertion then moves to the pre-normalizer text — record that seam in the decisions doc now.

## Next
J-112 alone already delivers the transcript's "call me when it needs me" (V-1 seam, spoken) and the audible
status digest. With J-111 it closes the full hands-free loop; with N-7 it becomes one render target among many;
the moat line for K-tier positioning: ELAI's voice cannot lie, because it renders journal rows.

End of J-112.
