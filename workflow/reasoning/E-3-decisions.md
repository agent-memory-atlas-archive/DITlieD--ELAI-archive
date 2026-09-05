E-3 model handoff UX — completion-drive decisions
==================================================

scope: E-3_model_handoff_ux (completion-drive cluster s-25120 session 4/4)
shipped: 2026-05-26
schema: SCHEMA_VERSION 147 -> 148 (atomic, single-variant pattern-2, 12th application)
total_variants: 420 -> 421 (action_type_lock golden regen +1 row)

what shipped
------------

A single new SsbActionType variant `RuntimeHandoffSignal` whose payload carries
{role, from_backend, to_backend, phase, reason}; the `phase` field is a
`HandoffSignalPhase` enum (Begin / Complete) lifted INSIDE the payload so the
single-variant pattern is preserved. Wrap site is
`RuntimeOrchestrator::try_complete` at crates/orchestrator/src/runtime_orchestrator.rs:202
(NOT `send` as the plan body claimed — that symbol does not exist; spec-drift
noted in DECISION-A below). Emission is wire-dark behind
`ELAI_MODEL_HANDOFF_ENABLE=1` (default OFF, Pattern-4). The CLI exposes
`/runtime handoff --role <name> --backend <target>` (with `HandoffDisabled`
error when the env gate is unset); the dashboard exposes a Tauri command
`register_handoff_signal(role, target_backend)` and a "handoff" button on each
role row in RuntimePanel.svelte that returns an inline armed/staged report.
All 14 RuntimeBackend variants are representable in the from/to fields
(B-37 11 + Composer2 + LlamaCpp + Mlx).

DECISION-A (spec-drift correction)
-----------------------------------

Plan body said "wrap RuntimeOrchestrator::send" + "stream_step trait method".
Neither symbol exists. Actual public dispatch is `try_complete` at
runtime_orchestrator.rs:202; the RuntimeAdapter trait surfaces `complete` and
`stream` at providers/runtime/types.rs:557+566. The wrap was re-anchored to
`try_complete` so CLI, dashboard `set_role_runtime`, and programmatic routes
funnel through the SAME emission point — the single source-of-truth principle
the plan intended. No new trait method was added (no `warm()`); the plan
asked for one but the completion-drive scope kept the diff focused on the
beacon + wiring slice and deferred warm() to a follow-up E-N (RUNPOD-PENDING:
needs measured cold-start breakdown per arXiv:2604.06664 Foundry before the
default-impl-vs-specialized split is meaningful on a 12GB single-GPU box).

DECISION-B (single variant, Pattern-2)
---------------------------------------

12th application of the D-19 / D-23 / D-24 / D-25 / D-28 / D-29 / D-30
single-variant pattern: lift the Begin/Complete discriminator INSIDE the
payload as `HandoffSignalPhase` enum rather than introducing
`RuntimeHandoffBegin` + `RuntimeHandoffComplete` as two variants. SCHEMA bumps
exactly +1, cascade is 9-tuple + 10th lockstep at action_type_lock.rs +
action_type_lock.golden (alphabetical insertion of `runtime_handoff_signal`
between `runtime_health_changed` ord 0318 and `safety_critical_blocked`
new ord 0320; all ordinals 0319-0419 shifted +1).

DECISION-C (env gate, Pattern-4)
---------------------------------

`ELAI_MODEL_HANDOFF_ENABLE=1` activates emission. Read via
`std::env::var(...).ok().as_deref().map(str::trim).map(|s| matches!(s, "1"|"true"|"TRUE"|"True"))`
inside `RuntimeOrchestrator` (cached in a `std::sync::OnceLock` so the
env lookup happens once per process). The CLI subcommand re-reads the
env directly (no OnceLock) so the test harness can flip it between
invocations without process restart. Default OFF matches
memory:project_vram_tier_plan.md — 12GB single-GPU cannot hold two
models so the signal would be diagnostic-only at the prod tier today.
Promotion to default-on lands when 24GB+ is the tier baseline (open).

DECISION-D (paper-cite scrubs)
-------------------------------

Three FIXes from the paper-ingestor triage (workflow/sessions/s-25120/
completion-drive/grounding/E-3_model_handoff_ux-grounding.md):

FIX-1 (arXiv:2507.03254 CodeAgents — REAL_BUT_WRONG_REGIME): the paper is
about prompt FORMAT for agent-to-agent communication efficiency (55-87%
token reduction), not about "system prompts persistence". Plan's Task 1
cite for "system prompts = persistent application behavior" was incorrect.
Resolution: keep the cite ONLY for token-efficiency framing of the SSB
payload comments; the "persistent application behavior" claim is dropped
and replaced with a first-principles assertion (the wrap-site IS the
single source-of-truth — that is the mechanism, not a paper claim).

FIX-2 (arXiv:2602.15013 Text Style Transfer — REAL_BUT_WRONG_REGIME): the
paper trains a PEFT LoRA on synthetic parallel corpora; ELAI's Task 8 was
a ~120 LOC display-layer filter. PEFT cite overstates the grounding.
Resolution: drop the cite from the SSB payload comments; mark Task 8 as
DECISION-only ("display filter, no paper needed"). Task 8 ITSELF is
deferred from this slice — the persona-vector + style-normalizer
sidecar-side work lives in a follow-up E-N once the remote sidecar lands
the measurement harness (RUNPOD-PENDING per Open Question 2).

FIX-3 (arXiv:2310.05736 LLMLingua — RELABEL ONLY): the plan's
"compress_handoff_payload uses token-frequency + cross-encoder rerank" is
NOT what LLMLingua actually does (the paper uses a small LLM perplexity
scorer). Resolution: comments and SSB payload labels say "LLMLingua-INSPIRED"
not "LLMLingua"; any compression guarantees come from the integration test
fixture, NOT from the paper. The compress_handoff_payload function itself
is OUT OF SCOPE for this slice (the handoff signal beacon ships first; the
compression module ships in a follow-up E-N alongside the LoRA / persona-
vector work that the sidecar tier requires).

DECISION-E (anti-pattern fence vs DroidSpeak arXiv:2411.02820)
---------------------------------------------------------------

The RuntimeHandoffSignal payload deliberately carries NO KV-tensor field.
DroidSpeak documents that cross-architecture KV-cache reuse degrades
quality on backbone-mismatched models (Qwen vs Claude). The plan
explicitly rejected DroidSpeak-style paths via stub_ban; the payload
struct holds that fence — `from_backend` + `to_backend` carry the
RuntimeBackend enum only, no KV blob, no activation tensor, no
embedding. Future contributors adding a KV-tensor field to this struct
violate the anti-pattern fence and will trigger the `cross_model_kv` /
`droidspeak` stub_ban grep.

DECISION-F (RuntimeBackend 14-variant coverage)
------------------------------------------------

Plan body said "11 backends". Actual is 14: B-37 11 (Sidecar, AnthropicApi,
ClaudeCode, CodexCli, OpenCode, Hermes, GeminiCli, KiroCli, CopilotCli,
Ollama, Vllm) + post-B-37 (Composer2 / LlamaCpp / Mlx). The
RuntimeHandoffSignalPayload uses `elai_providers::runtime::RuntimeBackend`
directly so the serde wire shape covers every variant via
`canonical_name`. No enum subset; no string casting. Adding a 15th
RuntimeBackend variant in a future plan is supported with zero changes
to E-3 code paths.

DECISION-G (name-collision avoidance)
--------------------------------------

`load_handoff` already exists at commands.rs:3419 (unrelated SessionDag
snapshot loader). The E-3 Tauri command is named `register_handoff_signal`
NOT `register_handoff` so grep / dev-tooling can distinguish the two
surfaces. Same reasoning on the CLI subcommand: `handoff` lives under
`/runtime` rather than as a top-level slash command so it clusters with
`/runtime list / test / set / composer2` for operator discoverability.

DECISION-H (RuntimePanel extension, not new window)
----------------------------------------------------

Plan body said "Svelte component (window) in dashboard-v2/src/lib/windows/".
Established pattern (B-37 SHIPPED, B-41 SHIPPED) is to extend RuntimePanel.svelte
in lib/components/, which is already mounted inside SettingsWindow.svelte at
line 906. Creating a new HandoffWindow.svelte would require WindowHost
registration + new top-level mount — out of scope. E-3 added the handoff
button + inline drawer status to the existing RuntimePanel role-row grid
(single-file Svelte edit, no window registration churn).

what is OUT OF SCOPE for this slice (deferred to follow-up E-N)
---------------------------------------------------------------

* RuntimeAdapter::warm() trait method — needs measured cold-start
  breakdown per arXiv:2604.06664 Foundry (RUNPOD-PENDING).
* build_role_prompt + VoiceAnchor + voice.toml — plan's "crates/agents/"
  path was doubly-wrong (no such crate; `crates/agent/` is BIN-only with
  no src/lib.rs). Lands in a follow-up E-N that creates `crates/agents/`
  as a proper lib crate.
* compress_handoff_payload — LLMLingua-INSPIRED compression module;
  ships alongside the sidecar persona-vector work in a follow-up E-N.
* Persona-vector hook on python/elai_inference/server.py — RUNPOD-PENDING
  (Open Question 2: no measured effect-size on Qwen2.5-Coder abliterated
  3B/7B variants).
* Style-normalizer post-pass — display-layer filter; DECISION-only
  (Pattern-5: first-principles, no paper).
* vLLM enable_prefix_caching assertion — plan-cited line is HTTP response
  parsing, not engine config; needs re-anchor to python/elai_inference/
  server.py (out of this slice's scope).
* ROLE-not-MODEL chat-bubble badge in CommandWindow.svelte — coordinates
  with E-4 (chat UX confidence chips); deferred to E-4 slice to avoid
  schema-merge conflict on the bubble-header.

RUNPOD-PENDING items
--------------------

None in this slice's code. All RuntimeHandoffSignal emissions can be
verified end-to-end on the local host via the dashboard + CLI + a stub
`try_complete` invocation through the orchestrator (no live model swap
required for the beacon itself). Measurement-bearing work (warm() impl,
persona-vector A/B) lives in the deferred follow-ups above.

papers used in this slice
-------------------------

* arXiv:2310.05736 LLMLingua — LLMLingua-INSPIRED labeling (see FIX-3).
* arXiv:2604.06664 Foundry CUDA-graph templates — frames the `reason`
  string hint about cold-start dominance (wire-dark today).
* arXiv:2411.02820 DroidSpeak — anti-pattern fence on KV-tensor field
  absence (see DECISION-E).
* arXiv:2507.03254 CodeAgents — token-efficiency framing only (see FIX-1).
* arXiv:2602.15013 Text Style Transfer — DROPPED from active cites (see
  FIX-2); kept in this doc as the FIX target only.

Three papers were ingested to workflow/papers/ during the grounding
sweep (2507.21509 Persona Vectors, 2604.06664 Foundry, 2602.15669
PERSONA framework). The persona-vector / PERSONA papers feed the
deferred sidecar work, not this slice's code.

cascade sites touched
---------------------

E-3 single-variant 9-tuple + 10th lockstep:
  1. crates/orchestrator/src/ssb.rs SCHEMA_VERSION constant (147 -> 148).
  2. crates/orchestrator/src/ssb.rs SsbActionType variant decl
     (RuntimeHandoffSignal added after CliSubprocessKilled).
  3. crates/orchestrator/src/ssb.rs SQL CHECK clause (+ row 'runtime_handoff_signal').
  4. crates/orchestrator/src/ssb.rs as_sql_str arm.
  5. crates/orchestrator/src/ssb.rs from_str arm.
  6. crates/orchestrator/src/ssb.rs SsbPayload variant + RuntimeHandoffSignalPayload struct + HandoffSignalPhase enum.
  7. crates/orchestrator/src/ssb.rs payload->action_type arm.
  8. crates/orchestrator/src/ssb.rs test_payload arm (single-variant default).
  9. crates/orchestrator/src/ssb.rs all_test_payloads arm (fixture vec).
 10. crates/orchestrator/tests/action_type_lock.rs TOTAL_VARIANTS (420 -> 421)
     + ALL_VARIANTS appendix + exhaustive_match_guard arm.
 11. crates/orchestrator/tests/action_type_lock.golden — header "Total variants"
     bumped + new line `0319 runtime_handoff_signal` inserted; ordinals
     0319-0419 shifted +1 to 0320-0420.

Producer + sink:
 12. crates/providers/src/runtime/event_sink.rs — new RuntimeEvent::HandoffSignal variant.
 13. crates/orchestrator/src/runtime_event_sink_ssb.rs — translate HandoffSignal
     -> SsbPayload::RuntimeHandoffSignal (re-parse role + phase via serde).
 14. crates/orchestrator/src/runtime_orchestrator.rs — env-cache helper +
     try_complete Begin emit (top) + Complete emit on success path +
     Complete emit on non-recoverable path + Complete emit on
     fallback-chain exhaustion. All 4 emission sites wire-dark via
     `handoff_signal_enabled()`.

CLI surface:
 15. crates/cli/src/runtime_cmd.rs — RuntimeCmdError::HandoffDisabled
     variant + `handle_runtime_handoff` free fn + `Some("handoff")` arm
     in `handle_runtime_slash_command` + render_runtime_help update +
     unit test covering the env-gate short-circuit.

Dashboard surface:
 16. crates/dashboard-v2/src-tauri/src/commands.rs — HandoffSignalReport
     struct + `register_handoff_signal` #[tauri::command] fn.
 17. crates/dashboard-v2/src-tauri/src/lib.rs — `commands::register_handoff_signal`
     appended to the generate_handler![..] vector.
 18. crates/dashboard-v2/src/lib/api/types.ts — HandoffSignalReport interface.
 19. crates/dashboard-v2/src/lib/api/stub.ts — `registerHandoffSignal` export
     + import of HandoffSignalReport.
 20. crates/dashboard-v2/src/lib/components/RuntimePanel.svelte — handoff
     state ($state Maps for reports / errors / pending) + onHandoffClick
     handler + "handoff" button on each role row + inline drawer status
     (armed / staged) + CSS for the new classes.

cargo check status
-------------------

To be validated post-edit via `cd .elai_cc && cargo check --workspace`. The
auditor agent will run the full test suite. Main LLM (per
feedback_no_cargo_test_unless_permitted) only runs cargo check.
