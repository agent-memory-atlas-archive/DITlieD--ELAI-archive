# J-VOICE-GROUNDING-MAP — voice front-door wave (J-111..J-112)
Dated: 2026-07-10. Session: the voice-mode spec-eval session (decision ledger topic=voice-mode).
Design source: the user's voice-control call transcript (C:\Users\user\Downloads\conversation_transcript.md,
Parts 3-4: event-narrator-not-chatter, single supervisor voice, "a chatter that is the harness" with
deterministic truth, stream progress AND on-demand summaries, notification levels, focus/addressing) +
this session's grounding sweep over J-32/J-35/N-7/H-36/E-3/J-1. Detail files cite V-IDs, never re-assert.

V-1 [SHIPPED, code-grep 2026-07-10] .elai_cc/crates/orchestrator/src/notify_dispatch.rs::NotificationDispatcher
    (struct at :57) — the H-36 dev-event notification dispatcher (completion + needs-input, driven FROM real SSB
    rows; agent_notification_emitted family). THE J-112 TRIGGER SEAM. Re-grep the dispatch fn signatures at edit.
V-2 [ABSENT, code-grep 2026-07-10] crates/orchestrator/src/front_door.rs does NOT exist; FrontDoorRouter +
    IntentRouted absent from ssb/action_type.rs and task_runner.rs. J-32 is PLANNED, unbuilt. J-111 therefore
    depends HARD on J-32 and ships AFTER it; authoring J-111's ingress against a hand-rolled router instead of
    waiting is BANNED (it would fork the front-door concern N-plan exists to prevent).
V-3 [PLANNED] N-7 render(verdict, target) substrate (N-plan WAVE 5): human render as a REQUIRED verdict field,
    ONE dispatcher, declared targets. J-112's spoken-render table is authored as the target=spoken leg,
    N-7-forward-compatible: a flat table keyed by event family, migrated INTO N-7's dispatcher when it ships
    (never a second dispatcher after that point).
V-4 [PLANNED] E-3 voice.toml VoiceAnchor (single source of voice, blake3-stable preamble) + the env-gated
    display-layer style-normalizer. J-112 phrasing templates key off the anchor when it ships; a fixture anchor
    (same TOML shape) until then.
V-5 [PLANNED] J-35 IntentSpec {goal/constraints/hidden_assumptions/non_goals/validation_contract} + decision
    card. J-111's spoken readback-confirm is the J-35 decision card SPOKEN (soft dep; the readback text is built
    from the ROUTED typed intent either way, never from the raw transcript).
V-6 CANONICAL CONSTANTS: SCHEMA_VERSION is DERIVED at ssb.rs:1544 (= SSB_ACTION_TYPES.len() + SCHEMA_VERSION_DDL_EPOCH,
    epoch -309 at :1539); newest commit stamp = 271 (K-4 GateTaxMeasured wave, git log 2026-07-09). NEVER
    hard-code; re-derive SCHEMA + TOTAL_VARIANTS at edit (W-9); 9-tuple golden lockstep, bump sequentially.
V-7 WEB VERIFICATION (2026-07-10, all [EXTERNAL] directional — anti-slopsquat name+version+license re-verify at
    edit; NEW DEPS require explicit user confirmation per house rule):
    - local streaming ASR: Moonshine v2 streaming ~107ms latency on laptop CPU / ~269ms Linux x86 (encoder-state
      caching, compute-while-talking; modelslab.com + onresonant.com + northflank.com 2026 benchmarks);
      faster-whisper (SYSTRAN, CTranslate2) ~2x CPU / 4x GPU vs openai-whisper, the 500-800ms streaming class.
    - local TTS: Kokoro-82M (Apache-2.0, 82M params, CPU-viable, <0.3s synthesis; localaimaster + codesota +
      ocdevel 2026 roundups); Piper = minimal-footprint fallback.
    - the transcript's provider-latency worry DISSOLVES architecturally: command + narration planes never touch
      a provider; only CHAT-lane discussion inherits provider streaming latency.
V-8 DESIGN DECISIONS carried from the transcript + session (user-settled, do not re-litigate):
    - single supervisor voice, NOT per-session voices (crowded-room; user concurred, transcript Part 4);
    - the narrator is the HARNESS: deterministic truth path, personality is a phrasing layer, never a fact layer;
    - stream progress AND summaries (user: "don't trim the scope down") — progress at step granularity,
      needs-input preempts, per-session rate limit (the J-106 act-don't-narrate posture applied to the speaker).
V-9 REJECTED (feature-map Section-4 rows added this wave): per-session distinct TTS voices; always-on open mic
    (unauthenticated acoustic ingress; push-to-talk default, open-mic default-OFF); LLM-AUTHORED narration text
    (reintroduces the hallucination channel the render table closes — model phrasing allowed ONLY as the E-3
    style-normalizer pattern: display-layer, env-gated, facts pinned).
V-10 PROOF_SPINE: none (product-surface / voice-operator class) for BOTH units; user-acked 2026-07-10 in the
    authoring session (the PSR-5 note was surfaced first, user said "create the new plan files").

End of J-VOICE-GROUNDING-MAP.
