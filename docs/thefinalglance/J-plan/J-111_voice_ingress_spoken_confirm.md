# J-111: voice ingress + spoken readback-confirm — speech is a second transducer into the J-32 front door, never a new command surface

STATUS: PLANNED (WAVE VOICE-2; M-L; BUILD-on-planned — depends J-32 HARD, ships AFTER front_door.rs exists; J-112 ships FIRST on shipped substrate)
PROOF_SPINE: none (product-surface / voice-operator class; user-acked 2026-07-10, V-10)

Objective: the user speaks to ELAI, not to a model — the voice input plane is a local ASR transducer whose
output lands on the J-32 FrontDoorRouter exactly like typed chat, plus the trust additions typed chat never
needed: taint-tagged untrusted acoustic ingress, journaled transcript + audio-hash for replay, deterministic
session addressing (call-signs), and a SPOKEN readback-confirm on mutating lanes built from the ROUTED typed
intent (never an echo of the raw transcript). Hard boundaries: NO second router (V-2 — a hand-rolled voice
command grammar is BANNED; the router IS J-32); NO model in the command path (T0/T1 lanes are ASR + deterministic
matchers end to end); NO always-on open mic (V-9); HIGH-risk lanes NEVER auto-execute from voice. GD-1 do NOT
re-implement: J-32 lanes/chip/RiskLevel/GateMode, J-35 IntentSpec card (the readback is that card spoken, V-5),
J-1 perception-gateway doctrine (audio is its second modality, same condition-before-any-model posture).

Estimated effort: ~1.5-2 sessions (M-L). The python daemon + the ingress struct are small; the readback-confirm
state machine + the addressing vocabulary + the negative arms are the bulk.
Pre-requisites (re-grep, W-7): front_door.rs::FrontDoorRouter + IntentRouted SHIPPED (J-32 — V-2 says ABSENT
2026-07-10, this is the HARD gate); notify_dispatch.rs::NotificationDispatcher :57 (V-1, the J-112 speaker this
unit's readback rides); J-112 spoken_render.rs (the TTS leg that speaks the readback). Soft: J-35 IntentSpec.
Parallel-safe with: nothing in this wave (consumes J-112; blocked on J-32).
Source: the voice-control transcript + the 2026-07-10 spec-eval session; _grounding/J-VOICE-GROUNDING-MAP.md
V-1..V-10. Doctrine: J-0_INDEX.md (W/GD/HD/TD/ID inherited; ADDS no delta family — G-1 taint, TD-3, W-2/R9,
D-030 cover every new edge).

## Context
Why this matters: the transcript's core ask is "talk to the harness itself" — issue commands to sessions,
hands-free, while the harness narrates back (V-8). The input half is structurally cheap because J-32 already
makes chat = ELAI: a voice utterance is one more client of the same total-function lane routing. What voice ADDS
is a noisier, less-authenticated channel: ASR mis-transcription can silently retarget a mutating command
("session 30" vs "session 38"), and an open mic is an unauthenticated ingress (anyone in the room, audio from a
video). So the unit is mostly TRUST plane: push-to-talk gating, taint, replay pinning, phonetically-distinct
addressing, and readback-confirm — the spoken analog of the J-32 chip (Enter/Esc has no voice equivalent).
Latency is a non-problem for command lanes: ASR (~107-800ms local streaming class, V-7) + deterministic router,
no provider round-trip; only CHAT-lane discussion inherits provider streaming (V-7).

## Existing state (V-IDs from the grounding map)
- FRONT-DOOR ROUTER [ABSENT — J-32 PLANNED] front_door.rs does not exist (V-2). THE HARD DEP. J-111 never ships first.
- NOTIFICATION DISPATCHER [SHIPPED] notify_dispatch.rs::NotificationDispatcher :57 (V-1) — the readback speaker seam via J-112.
- INTENT CARD [PLANNED] J-35 IntentSpec/decision card (V-5) — the readback content shape.
- RISK LADDER [SHIPPED per J-32 entry] Phase-3A RiskLevel + B-22 GateMode — voice rides the same ladder, stricter floor.
- ENTROPY QUARANTINE [SHIPPED, Phase-0] ClockProvider/EntropySource — any variance in spoken output is seeded, never thread_rng.

## Reads (mandatory before coding)
1. _grounding/J-VOICE-GROUNDING-MAP.md (all V-IDs).
2. J-32 + J-35 detail entries in J-SUMMARY_INDEX.md; front_door.rs IN FULL once it exists (the route seam + IntentRouted emission).
3. notify_dispatch.rs IN FULL (V-1) + J-112's spoken_render.rs (the speaker this unit calls).
4. python/elai_inference/server.py boot/arg idioms (the sidecar process pattern the voice daemon MIRRORS — local, not remote).
5. CLAUDE.md hard rules: G-1 taint, TD-3, W-2/R9 kill-switch+allowlist, D-030 production-caller, no .unwrap on prod, anti-slopsquatting (V-7 deps).

## Tasks (WAVE VOICE-2 — ship AFTER J-32)
1. python/elai_voice/ ingress daemon (~250 LOC py, NEW): local mic capture, push-to-talk gate (open-mic mode exists but default-OFF, V-9), VAD, streaming local ASR (Moonshine v2 primary candidate / faster-whisper fallback, V-7 — anti-slopsquat verify + USER CONFIRM the deps at edit). Emits {transcript, asr_confidence, audio_blake3, asr_model_pin, ts} over localhost to the CLI host. Audio bytes never leave the machine; bytes NOT stored (hash only).
2. crates/orchestrator/src/voice/ingress.rs (~120 LOC, NEW): typed VoiceUtterance carrying the daemon fields + taint=Untrusted (G-1). Journals +VoiceUtteranceIngested BEFORE routing (the replay input is the transcript + hash + model pin — GD-2: a voice run recomputes from rows). Then FrontDoorRouter::route(transcript) — the SAME call typed chat makes; IntentRouted gains a source=voice discriminator (reuse, no new row).
3. Session addressing (~60 LOC in ingress.rs): a deterministic addressee matcher BEFORE lane routing over a call-sign vocabulary — phonetically distinct aliases assigned at session spawn (collision-checked at assignment); bare numerals accepted but ALWAYS read back on mutating lanes. Unresolved/ambiguous addressee -> spoken CLARIFY (never a guess; the J-32 fail-closed-to-asking posture).
4. Spoken readback-confirm (~100 LOC): for MEDIUM+ lanes, build the readback from the ROUTED intent (lane + addressee + arg_spans; the J-35 card spoken, V-5) — NOT the raw transcript — speak it via J-112, await an explicit confirm token; asr_confidence below floor -> CLARIFY regardless of lane. HIGH risk: readback + confirm is necessary but NOT sufficient — voice HIGH always ALSO requires the existing non-voice confirmation surface (chip/dashboard). Confirm/abort decisions journal as the same labeled tuples J-32's chip emits (D-16 calibration feed).
5. `elai voice` CLI subcommand (~80 LOC in crates/cli): the daemon host + production entrypoint (spawn daemon, own the socket, wire ingress). Env gate ELAI_VOICE_ENABLE default-OFF (hardware + new deps; type=experiment wire-dark-allowlist.toml row with owner/expiry/promotion_plan_id=J-111/smoke, R9).
6. INTEGRATION_SMOKE (with J-112, the wave smoke — R10): drive `elai voice` with a recorded fixture utterance end to end: ingress row -> IntentRouted(source=voice) -> readback utterance -> confirm -> dispatch rows IDENTICAL to the typed /fix sequence (GD-2 diff zero) -> completion -> J-112 narration row. HARD input: a mis-addressed utterance fixture ("session thirty" where 38 exists) must land in CLARIFY, zero dispatch.
7. workflow/reasoning/J-111-decisions.md (readback-from-routed-intent not transcript; call-sign vocabulary; HIGH never voice-only; open-mic default-OFF; the J-32-hard-dep sequencing).

## WARGAME
MOVE 1 daemon + ingress (tasks 1-2).
EXPECTED_OBSERVATION: a fixture utterance produces a VoiceUtteranceIngested row (transcript + audio_blake3 + asr_model_pin) BEFORE any route row.
ON_FAIL: row missing or ordered after routing -> the replay input is unpinned (GD-2 broken); counter-move: emit in ingress.rs before route(), assert ordering in the contract test.
MOVE 2 addressing + readback (tasks 3-4).
EXPECTED_OBSERVATION: "session <callsign>, fix X" routes to that session; the readback utterance text derives byte-wise from the routed intent fields; confirm token dispatches, Esc/abort dispatches nothing.
ON_FAIL: readback echoes the raw transcript (mis-transcription survives) -> counter-move: build readback exclusively from IntentRouted payload fields; the contract test feeds a corrupted-transcript fixture and asserts the readback names the ROUTED args.
FORK: J-32 unshipped at execution time -> STOP, BLOCKED handoff (R-4). The trigger is deterministic: front_door.rs absent = blocked. Do NOT hand-roll a router (V-2).
FORK: ASR dep fails anti-slopsquat or user declines the new deps -> fall back to the OS/system dictation path (daemon accepts text over the same socket, ASR fields marked provider=external); trigger = user says no at the task-1 confirm.
RECON_NEEDED: the exact FrontDoorRouter::route signature + IntentRouted payload fields (V-2 — unknowable until J-32 lands; re-grep at edit, W-7).
RECON_NEEDED: Moonshine v2 python package name/version/license live at edit (V-7 is directional; anti-slopsquat protocol).
ABORT: any path where a mutating dispatch can fire with no confirm row in the journal; any storage of raw audio bytes; any second command grammar outside J-32 lanes.
VERIFY: the DETERMINISTIC_CHECK block below + the contract test; pass artifacts are SSB rows + checker exit codes (off-agent, R4), never daemon stdout.
REDTEAM (attacks run at authoring): (a) "mis-heard session id dispatches to the wrong session" — killed by readback-from-routed-intent + call-signs + the CLARIFY floor (MOVE 2 negative arm). (b) "room audio injects a command" — killed by push-to-talk default + open-mic default-OFF + HIGH requiring the non-voice surface. (c) "voice run is unreplayable" — killed by MOVE 1 row ordering. Surviving residue (GD-5): a cooperative attacker with mic access during push-to-talk on a LOW lane; accepted — LOW lanes are non-mutating by the J-32 ladder.
INTEGRATION_SMOKE: task 6 (the wave smoke, spans J-111+J-112).

## WIRING_CONTRACT
produces_symbols:
  - python/elai_voice/ (ingress daemon: mic, push-to-talk, VAD, streaming ASR, localhost emit)
  - crates/orchestrator/src/voice/ingress.rs (VoiceUtterance + taint + journal-before-route + addressee matcher + readback-confirm)
  - crates/cli `elai voice` subcommand (the production entrypoint) + the ELAI_VOICE_ENABLE allowlist row
consumes_symbols:
  - crates/orchestrator/src/front_door.rs::FrontDoorRouter (J-32; the ONLY router — V-2, GD-1)
  - crates/orchestrator/src/voice/spoken_render.rs (J-112; speaks the readback + CLARIFY)
  - crates/orchestrator/src/notify_dispatch.rs::NotificationDispatcher :57 (V-1; the utterance dispatch path)
  - Phase-3A RiskLevel + B-22 GateMode (the risk ladder; voice floor is STRICTER, never looser)
reachable_from:
  - crates/cli `elai voice` -> daemon -> ingress.rs -> FrontDoorRouter::route [PRODUCTION voice front door]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO second router / bespoke voice grammar (V-2); NO model in the T0/T1 command path
  - NO raw-audio persistence (hash + model pin only); NO readback built from the raw transcript
  - NO mutating dispatch without a journaled confirm tuple; NO HIGH-risk voice-only execution
  - NO thread_rng anywhere in spoken variance (seeded EntropySource only); NO .unwrap on prod paths; NO always-on mic default
frontend_pair:
  the dashboard chat surface shows voice-sourced messages with a source=voice badge + the readback/confirm state
  (reuse the J-32 ACK-card surface; a Tauri read of the same rows). Justified minimal — the voice UI is audio.
observable_effect:
  trigger: "ELAI_VOICE_ENABLE=1 elai voice --fixture tests/fixtures/voice/fix_sum_utterance.wav (the recorded-fixture arm; then the live-mic arm manually)"
  expect_ssb: "VoiceUtteranceIngested (audio_blake3 + asr_model_pin) -> IntentRouted{source=voice} -> confirm tuple -> the IDENTICAL dispatch row sequence the typed /fix produces (GD-2 diff zero)"
  expect_artifact: "the readback utterance text equals the spoken render of the ROUTED intent; NEGATIVE (must FAIL): a mutating dispatch with no confirm row; a readback containing transcript text absent from the routed args; the mis-addressed fixture producing any dispatch row"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-111_voice_ingress_spoken_confirm.md --run-trigger
rg "VoiceUtteranceIngested|audio_blake3|asr_model_pin|taint" .elai_cc/crates/orchestrator/src/voice/ingress.rs   # the ingress legs
rg -c "thread_rng|rand::random" .elai_cc/crates/orchestrator/src/voice/   # MUST be 0 (entropy quarantine)
rg -c "\.unwrap\(\)|\.expect\(" .elai_cc/crates/orchestrator/src/voice/ingress.rs   # MUST be 0 on prod paths
python .claude/tools/production_caller_check.py --files .elai_cc/crates/orchestrator/src/voice/ingress.rs   # 0-DARK (D-030)
python .claude/tools/python_wire_check.py --files .elai_cc/python/elai_voice/daemon.py   # 0-DARK
rg "ELAI_VOICE_ENABLE" .elai_cc/wire-dark-allowlist.toml   # the R9 row exists
test -f workflow/reasoning/J-111-decisions.md
```

## ACCEPTANCE
- [ ] speech reaches sessions ONLY through FrontDoorRouter (no bespoke grammar; V-2/GD-1).
- [ ] every utterance journals transcript + audio_blake3 + asr_model_pin BEFORE routing (GD-2 replay pin).
- [ ] MEDIUM+ = readback-from-routed-intent + confirm tuple; HIGH additionally requires the non-voice surface; low-confidence ASR = CLARIFY.
- [ ] push-to-talk default; open-mic default-OFF; audio bytes never persisted.
- [ ] ELAI_VOICE_ENABLE default-OFF with complete allowlist row (R9); new ASR deps anti-slopsquat-verified + user-confirmed.
- [ ] cargo check GREEN; auditor PASS -p orchestrator -p cli (+ the python daemon smoke); the wave INTEGRATION_SMOKE (task 6) run off-agent.

## Tests to add
- crates/contracts/tests/j111_voice_ingress_contract.rs (~120 LOC): (a) ingress row precedes route row; (b) readback text derives from routed fields, corrupted-transcript fixture asserts divergence surfaces; (c) mis-addressed fixture -> CLARIFY, zero dispatch; (d) no-confirm mutating dispatch is impossible (negative arm); (e) HIGH voice-only dispatch is impossible; (f) confirm-tuple GD-2 diff-zero vs the typed /fix sequence; (g) corrupt/empty audio fixture degrades to CLARIFY never a guess (R1).

## Files touched
- CREATE: .elai_cc/python/elai_voice/daemon.py (+ pyproject deps, user-confirmed) (~250 LOC)
- CREATE: .elai_cc/crates/orchestrator/src/voice/ingress.rs (~280 LOC incl addressing + readback)
- MODIFY: .elai_cc/crates/cli (the `elai voice` subcommand, ~80 LOC; PRODUCTION_HOOK)
- MODIFY: .elai_cc/wire-dark-allowlist.toml (ELAI_VOICE_ENABLE row)
- MODIFY: ssb action_type + golden lockstep (+VoiceUtteranceIngested; V-6 re-derive at edit)
- CREATE: crates/contracts/tests/j111_voice_ingress_contract.rs (~120 LOC)
- CREATE: workflow/reasoning/J-111-decisions.md

## Citations / Grounding
- _grounding/J-VOICE-GROUNDING-MAP.md V-1..V-10 (code-grep + web verification, dated 2026-07-10).
- The voice-control transcript (design source; user decisions V-8) + the session decision ledger (topic=voice-mode).
- CLAUDE.md: G-1 taint, TD-3, W-2/R9, D-030, GD-2, anti-slopsquatting; J-32/J-35 entries (J-SUMMARY_INDEX.md:779/916).

## Open questions / risks
- The confirm-token grammar ("confirm"/"go"/Esc-word) must itself be a T1 deterministic matcher — decide the token set at edit; it is part of the frozen lane vocabulary, never model-classified.
- Call-sign assignment collides with the dashboard's session naming — reuse the same registry if one ships before this; do not fork the naming concern.
- Streaming-ASR partials: route only on end-of-utterance (VAD close) in v1; partial-transcript routing is a DEFERRED optimization (latency is already sub-second, V-7).

## Next
With J-112's narration plane, this completes the loop the transcript asked for: speak a command, hear the
readback, confirm, and the harness talks back off its own journal — sessions addressed hands-free, nothing
dispatchable without a row, nothing spoken without a row.

End of J-111.
