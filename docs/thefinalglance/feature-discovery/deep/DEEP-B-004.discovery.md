# DEEP-B-004 — exhaustive feature discovery

## Scope, method, and source integrity

This pass deep-read exactly the eight numbered/index originals assigned to `DEEP-B-004` in `DEEP-SOURCE-MANIFEST.md` lines 307-320. No other B-plan original was opened. `02-B-PLAN-SUMMARY.discovery.md` was used only as an orientation/comparison layer; it is not treated as canonical where a chunk source is more specific or newer.

Classification used below:

- **E** — stated explicitly in the canonical chunk source.
- **I** — conservative synthesis whose individual links are explicit but whose composed interaction is not stated verbatim.
- **A** — unresolved choice, underspecified behavior, or ambiguity retained without resolution.
- **C** — internal or summary/source conflict that must be reconciled rather than silently normalized.

An “atomic feature” is one independently testable assertion, contract, state rule, configuration/default, evidence gate, or named capability set. Closed rosters (for example, all fields of one payload or all members of one enum) remain one atom, but every member is transcribed. Status tags such as `SHIPPED`, `PARTIAL-V2`, `EMPIRE-PENDING`, `PLANNED`, and `conditional` are preserved exactly enough to distinguish completed scaffolding from live behavior.

### Exact processed manifest and verification

| # | Manifest class | Exact absolute path | Manifest lines / bytes | Actual lines / bytes | Manifest SHA-256 | Verification |
|---:|---|---|---:|---:|---|---|
| 1 | numbered/index original | `docs/thefinalglance/B-plan/B-5_reasoning_producer_wiring.md` | 176 / 8723 | 176 / 8723 | `2dc125ed704490e7dcc384ffef2b1f2d62fcf658f1bcd070306d84db1e3fac1b` | exact |
| 2 | numbered/index original | `docs/thefinalglance/B-plan/B-14_live_token_streaming_chat.md` | 191 / 8379 | 191 / 8379 | `2b318fb705458d8f9807cb589f24c0c786f434949c77fd2ea1b9d7f4b96ed34a` | exact |
| 3 | numbered/index original | `docs/thefinalglance/B-plan/B-25_phase_4J_empire_integration.md` | 214 / 10501 | 214 / 10501 | `91b5af2c9cc0e340940f4d631dd0dbf389e0114aef9fc2976ea971510e86d78a` | exact |
| 4 | numbered/index original | `docs/thefinalglance/B-plan/B-27_cloud_mode.md` | 250 / 9698 | 250 / 9698 | `79853608c3a8598b8c5ec0a609a05b2204664db2da04606610af623ed29f498a` | exact |
| 5 | numbered/index original | `docs/thefinalglance/B-plan/B-31.5_resumable_indexing_scale_benchmark.md` | 499 / 19866 | 499 / 19866 | `9ba29b7b5fc57d3d70a33fa170a98b101cc683581d0ae9cf845d66ee8455ad29` | exact |
| 6 | numbered/index original | `docs/thefinalglance/B-plan/B-33_interactive_graph_viewer.md` | 501 / 22608 | 501 / 22608 | `e0216562c870cf2c3e98e303f5996b9fa44c7f81eb2b51c40bf8c64cca194b3d` | exact |
| 7 | numbered/index original | `docs/thefinalglance/B-plan/B-36_dashboard_v2_src_tauri_wiring.md` | 356 / 16455 | 356 / 16455 | `b6671f0e0e40bbd6250f4296c0703469bed8762862ecf61815558ba8c978a5b3` | exact |
| 8 | numbered/index original | `docs/thefinalglance/B-plan/B-42_cosmos_artifact_tier.md` | 408 / 20218 | 408 / 20218 | `45c46401f9b448a3be4e1470f6b60aa6c087d604a0da08ae9a03e767398a2cf9` | exact |
| **Total** | **8 originals** | — | **2595 / 116448** | **2595 / 116448** | — | **exact; 0 missing, 0 extra, 0 duplicate** |

Logical-line count is `wc -l`; byte count is `wc -c`. Every digest was recomputed with SHA-256 and matched the manifest.

## Per-file exhaustive ledgers

### 1. `B-5_reasoning_producer_wiring.md`

Status/source posture: objective is real Reasoning telemetry; the file is a plan with unchecked acceptance boxes, while its V2 rebaseline relocates the frontend/backend targets. B-8 is required for producers to fire on every production reasoning event; producer work itself has no prerequisite and is parallel-safe with B-4/B-6 (L1-L9).

- **B5-E01** [E, capability/status; L1-L9] Replace the synthetic ReasoningDrawer with real per-agent GRC budgets, grounding-sweep results, CSQ guards, and feasibility outcomes. Producer code is V2-agnostic; B-8 supplies production hot-path activation.
- **B5-E02** [E, surface/rebaseline; L11-L33] V1 `crates/dashboard/src/commands.rs` maps to V2 `crates/dashboard-v2/src-tauri/src/lib.rs`; V1 ReasoningDrawer maps to `crates/dashboard-v2/src/lib/drawers/ReasoningDrawer.svelte`; V1 `CsqBlockCard.svelte` maps to new `crates/dashboard-v2/src/lib/components/CsqBlockCard.svelte`; V2 has no `intervention/` directory yet.
- **B5-E03** [E, stub contract; L21-L33] Existing V2 stubs are `get_grc_budgets`, `get_grounding_sweep`, `get_csq_pending`, and `get_gg_signals`; live wiring must keep `stub.ts` names 1:1.
- **B5-E04** [E, failure/gap evidence; L35-L47] Current `emit_reasoning` hard-codes `agent_id="task_runner"`, `topic="task_runner"`, `budget_remaining=0`, `grounding_level="medium"`, `confidence=0.9`, and `elapsed_ms=0`; `BasicGrcEngine::think()` is test-only. Phases 2B/2C/2D lack Tauri surfaces and 2B/2D lack SSB types, making Critical CSQ blocks and `Blocked` feasibility outcomes invisible.
- **B5-E05** [E, data producer; L61-L64] Refactor the TaskRunner emission to consume a real `PipelineGrc` handle and source `agent_id`, `topic`, `budget_remaining_calls`, `budget_remaining_chars`, `grounding_level` enum, `confidence: f32`, and `elapsed_ms: u64`; hard-coded values are banned.
- **B5-E06** [E, event/schema; L66-L70] `grc_step_emitted` payload is `{agent_id, topic, step_type, grounding_level, char_count}`.
- **B5-E07** [E, event/schema; L66-L70] `grounding_sweep_result` payload is `{plan_id, claims_total, grounded, verify_needed, ungrounded[]}`.
- **B5-E08** [E, event/schema/security; L66-L70] `csq_evaluated` payload is `{function_name, severity: CRITICAL|WARNING, guard_present, blocked}`.
- **B5-E09** [E, event/schema; L66-L70] `feasibility_check` payload is `{claim, outcome: Pass|Fail|Blocked, source}`.
- **B5-E10** [E, command/schema; L72-L78] `get_grc_budgets_per_agent()` returns `Vec<{agent_id, budgets_per_topic[]}>`.
- **B5-E11** [E, commands; L72-L78] `get_grounding_sweep(plan_id)` returns the last sweep; `get_csq_log(limit)` and `get_feasibility_log(limit)` return recent event logs; `get_agent_grc_config()` exposes the `AgentGrcConfig` table so the B-15 divergence is visible.
- **B5-E12** [E, UI/data; L80-L85] ReasoningDrawer has eight budget rows for Executor, Auditor, ContextCompiler, PromptEngineer, MemoryWriter, Designer, Researcher, and MediaValidator. Each shows remaining/max calls and remaining/max characters, with a red bar below 20%.
- **B5-E13** [E, UI/workflow; L86-L87] Last-grounding-sweep panel is a pie with `GROUNDED` green, `VERIFY_NEEDED` amber, and `UNGROUNDED` red; clicking exposes the ungrounded-claim list.
- **B5-E14** [E, UI/security; L88-L90] CSQ list shows function, severity, and blocked state with a red badge for Critical unguarded blocks; feasibility UI shows Pass/Fail/Blocked counts and the last five events.
- **B5-E15** [E, UI/config; L91-L92] Collapsible per-role configuration viewer renders `AgentGrcConfig` fields and highlights cells red where the divergent table differs from the spec.
- **B5-E16** [E, event/observability; L94-L97] Silent Auditor convergence rejection becomes `auditor_rubber_stamp_rejected`, supporting a session counter such as “Auditor 12 rubber stamps rejected this session.”
- **B5-E17** [E, safety workflow/recovery; L99-L103] A Critical CSQ without a guard already blocks progression and must surface as a red intervention card naming function and CSQ (example: `delete_file` / `data_loss`) with View details, Override, and Decline actions; Decline returns `ReplanRequest` to the FSM.
- **B5-E18** [E, exact symbols; L105-L123] Produced Rust SSB variants are `SsbActionType::{GrcStepEmitted, GroundingSweepResult, CsqEvaluated, FeasibilityReport, AuditorRubberStampRejected}`. V2 command symbols are `get_grc_budgets`, `get_grounding_sweep`, `get_csq_log`, `get_feasibility_log`, `get_agent_grc_config`, and `get_auditor_rubber_stamp_rejections`; consumers are ReasoningDrawer and CsqBlockCard; reachability begins at `task_runner.rs::start_task_runner`.
- **B5-E19** [E, stub ban; L124-L129] `emit_reasoning` may not emit fixed `agent_id="task_runner"`, `grounding_level="medium"`, `confidence=0.9`, or `elapsed_ms=0`; frontend and backend belong to the same B-step.
- **B5-E20** [E, observability/evidence; L130-L151] The prescribed live trigger runs a delete-file task; evidence is a `csq_evaluated` SSB row plus ReasoningDrawer showing real budgets and the last CSQ. Deterministic checks require zero synthetic-agent hits, exactly four core type hits in the enumerated grep, and a smoke query against `ssb.db`.
- **B5-E21** [E, acceptance/done gate; L153-L162] Done requires real emission values, 4-5 new SSB action types, five live-data Tauri commands, eight gauges, grounding pie, live CSQ/feasibility panels, visible rubber-stamp count, and Critical-without-guard intervention UI.
- **B5-E22** [E, lifecycle/files; L164-L176] Planned touch set is TaskRunner, SSB payload/type definitions, dashboard commands/SSB viewer, ReasoningDrawer, new CsqBlockCard, and `workflow/reasoning/B-5-decisions.md`; B-6/B-7/B-8 fire these producers.
- **B5-C01** [C, identifier mismatch; L66-L78 vs L105-L118] Prose event `feasibility_check` maps to enum name `FeasibilityReport`, and prose command `get_grc_budgets_per_agent` maps to contract symbol/stub `get_grc_budgets`; `get_csq_pending` is an existing stub but the planned live command is `get_csq_log`. These are aliases or schema drift, not safe renames without reconciliation.
- **B5-C02** [C, count mismatch; L66-L70, L105-L118, L153-L157] Task prose introduces four reasoning types plus a fifth rubber-stamp type later; acceptance says “4-5.” The contract has five SSB variants and six command symbols, while acceptance says five commands. The sixth is `get_auditor_rubber_stamp_rejections`.
- **B5-C03** [C, path drift; L11-L33 vs L164-L170] Rebaseline makes V2 authoritative, but “Files touched” retains V1 paths. Execution must use the V2 map.
- **B5-A01** [A, permission semantics; L99-L103] The card offers `Override`, but the source does not define authorization, audit, or whether override can bypass a Critical unguarded CSQ; only Decline behavior is defined.

### 2. `B-14_live_token_streaming_chat.md`

Status/source posture: the plan replaces an intent-only dashboard stub with the real conversation runtime. The summary calls the V2-scoped work done, but this original retains unchecked acceptance items. B-12 Workspace is prerequisite; streaming layer itself has none; B-13/B-15..B-20 are parallel-safe (L1-L7).

- **B14-E01** [E, capability; L1-L7] Dashboard chat must stream real model tokens from sidecar `/v1/messages` SSE instead of only writing an intent row.
- **B14-E02** [E, surface; L9-L25] V2 replaces ChatWindow with `CommandWindow.svelte` plus `CommandPalette.svelte`: a vertical card list, outlined `.input-shell` using `--bg-input` and focused `--teal-line`, and a slash dialog anchored above the shell that auto-opens when `/` starts the textarea.
- **B14-E03** [E, commands/UI; L21-L25] Palette groups are Workflow (`/task /fix /research /continue`), Project (`/explore /status /config`), and Help (`/ask`). Buttons use outlined language only: teal Approve; neutral Reject/Modify/Redirect; solid-fill CTAs are prohibited.
- **B14-E04** [E, rebaseline paths; L27-L40] `submit_chat_message` maps to new V2 `stream_chat`; ChatWindow maps to CommandWindow; SlashPalette maps to existing CommandPalette; InterventionCard is either a new V2 component or a CommandWindow card variant; `intent_classifier.rs` is unchanged and V2-agnostic.
- **B14-E05** [E, integration/events; L42-L53] `stream_chat(message)` uses Tauri 2 `Window::emit("token", ...)`; B-36 D.7 shares the SSE-via-emit pattern. Channel vocabulary must stay consistent: `turn`, `streaming_started`, `response`, `token`, `stop`.
- **B14-E06** [E, frontend lifecycle; L49-L64] CommandWindow imports `listen` from `@tauri-apps/api/event`; Stop invokes `cancel_stream` with a server-side AppState cancellation handle. Token buffer uses Svelte 5 `$state`; `$effect` owns subscription cleanup; legacy `$:` and `createEventDispatcher` are banned.
- **B14-E07** [E, runtime workflow; L66-L76] The reused path is `ConversationRuntime -> RuntimeClient(Local) -> sidecar /v1/messages -> DoLaSteeringProcessor -> model.generate -> SSE -> render`, already wired in CLI/REPL but absent from dashboard.
- **B14-E08** [E, command/workflow; L88-L95] New `stream_chat(message)` either returns token events as a stream or emits them through Tauri, reusing the CLI runtime; frontend renders tokens incrementally with a typing indicator while streaming.
- **B14-E09** [E, deterministic routing; L97-L102] Every input is classified before routing as `TASK|FIX|EXPLORE|CONFIG|INTERVENE|STEER|ABORT`; slash commands/keywords are deterministic and ambiguous natural language falls back to the LLM, with a FLOW-4 disambiguation card.
- **B14-E10** [E, UI; L104-L109] Slash autocomplete filters `/task /fix /explore /config /status /continue`; response types include Status, Intervention, Error, Disambiguation, and Steering acknowledgment using the specification layouts.
- **B14-E11** [E, workflow/state; L111-L115] Placeholder text is stateful: IDLE = “describe a task, ask a question, or type / for commands”; EXECUTING = “add constraints, or wait for intervention cards”; active intervention = “respond to the card above, or type to add context.”
- **B14-E12** [E, constraint/non-goal; L116-L121] Command must not emit “thinking...” messages; work progress belongs in ACTIVITY, and COMMAND updates only for input needs or milestones. A Stop icon appears during streaming and cancels the in-flight sidecar request.
- **B14-E13** [E, exact symbols; L123-L140] Produced symbols are `dashboard-v2/src-tauri/src/commands.rs::stream_chat` and `intent_classifier.rs::classify_intent`; consumers are CommandWindow/CommandPalette; reachability is `CommandWindow.svelte::submitInput`; observable evidence is `chat_message` plus character-by-character token rendering.
- **B14-E14** [E, stub ban; L133-L140] Remove the intent-only `submit_chat_message` stub and render tokens as they arrive, never only after completion.
- **B14-E15** [E, evidence; L142-L155] Deterministic checks require `stream_chat` presence, all seven intent categories, and a `/task hello` smoke showing planning state.
- **B14-E16** [E, acceptance/done gate; L157-L165] Done requires real streaming, correct seven-way classification, autocomplete, five response-card types, FSM placeholders, effective cancellation, and zero “thinking...” messages.
- **B14-E17** [E, lifecycle/files; L167-L175] Original touch set names commands, ChatWindow, SlashPalette, InterventionCard (possibly reuse B-5), new intent classifier, and `workflow/reasoning/B-14-decisions.md`; V2 rebaseline supersedes V1 frontend paths.
- **B14-E18** [E, part gate; L177-L187] Part II gate claims B-11..B-14 collectively deliver design-spec appearance, Workspace primary with in-shell status and 48px/five-icon Sidebar, Activity as centerpiece with auto-scroll, preview links, and collapsible reasoning, plus real-time chat—fully resolving the cited UX complaint before Part III hardening.
- **B14-C01** [C, card count; L15-L16 vs L107-L109 vs L157-L163] V2 card roster has six variants including `user`; task/acceptance count only five response types and omit user because it is an input card. Preserve six UI variants versus five response types.
- **B14-C02** [C, slash roster; L21-L22 vs L104-L105] Rebaseline includes `/research` and `/ask` but task autocomplete lists `/explore /config /status` and omits both. The union is not explicitly declared authoritative.
- **B14-C03** [C, channel semantics; L42-L53 vs L88-L95] Source calls the transport SSE but specifies Tauri event emission/listening, not direct browser SSE; treat sidecar SSE and frontend Tauri events as separate hops.
- **B14-C04** [C, path/check drift; L27-L64, L123-L147, L167-L174] Rebaseline and contract use V2, but deterministic grep and files list still point at V1 `dashboard/src/commands.rs`/ChatWindow.
- **B14-A01** [A, component choice; L34-L38] Intervention UI placement is unresolved: standalone `InterventionCard.svelte` or absorbed CommandWindow variant; decision doc must capture it.
- **B14-A02** [A, stream API; L90-L92] Command return mechanism remains an alternative: Rust stream return versus Tauri emitter.
- **B14-A03** [A, cancellation contract; L52-L53, L120-L121] Handle identity, cancellation idempotency, terminal event, error mapping, and race behavior after completion are unspecified.

### 3. `B-25_phase_4J_empire_integration.md`

Status/source posture: conditional on Empire codebase/deployment access. Post-`s-25120`, host-side scaffolding and residual closure are `SHIPPED-ON-V2`, while all live transport/dispatcher/filter pieces called `EMPIRE-PENDING` remain conditional (L1-L26).

- **B25-E01** [E, objective/capabilities; L1-L12] Phase 4J comprises FactoryLine client, heartbeat/reconnect brain registration, replacement of broken `brainctl`, Empire-event-to-SSB bridge, four-capability `ElaiNeuron`, and repair handling for three verified issue classes; B-15..B-20 plus Empire access are prerequisites.
- **B25-E02** [E, shipped status; L14-L26] Shipped scaffolding includes the orchestrator Empire module, SSB schema `101 -> 102` with five `empire_*` variants, `AppState.empire_bridge`, CLI `--empire`, mounted EmpireBridgePanel, transport trait plus Unix/TCP implementations, ToolFilter integration, repair types/heuristic detector, and `VerificationMethod::EmpireEventCheck`.
- **B25-E03** [E, deferred status; L21-L26] `EMPIRE-PENDING` items are transport implementation bodies, bridge `emit_action` SSB writer call, FSB binary transport-header parser, and `EventFilter`/`RateLimiter`, governed by `phase-4J-decisions.md` F1..F11.
- **B25-E04** [E, integration/protocol; L28-L56] Empire exposes MKI/MKII/MKIII engine hierarchy with IPC dispatch, hooks, buses, HTTP, events, persistence, and WebSocket. FactoryLine control socket and IPC engine socket are described as newline-JSON over Unix streams.
- **B25-E05** [E, workflow/protocol; L38-L43] ELAI speaks pure-Rust newline-JSON via Unix socket or TCP, registers as a brain, heartbeats/reconnects/re-registers, and drives FactoryLine directly with zero Python dependencies.
- **B25-E06** [E, event workflow/grounding; L44-L45] Empire event subscription is rate-limited, filtered, priority-classified, bridged into SSB, and usable as a GRC grounding source.
- **B25-E07** [E, permissions/privacy; L46-L48] `ElaiNeuron` exposes exactly `task_submit`, `task_status`, `pipeline_result`, and `health_check`; ToolFilter is mandatory and no internal state may leak.
- **B25-E08** [E, repair workflow; L49-L51] Repair scope is exactly `CircularDep`, `HardImport`, and `StdlibShadow`; each produces a `PipelineSpec` and is validated with `TestOutcomeReport`.
- **B25-E09** [E, platform/failure/config; L53-L56] Generic transport selects `UnixTransport` on Linux and `TcpTransport` on Windows; absence/failure of Empire must degrade gracefully; integration is behind `cfg(feature="empire")`.
- **B25-E10** [E, transport status; L67-L72] `encode`, `build_command`, and newline contract tests are shipped; `FactoryLineTransport`, cfg-unix `UnixTransport`, `TcpTransport`, and `build_transport` factory are shipped; live socket boot bodies remain pending.
- **B25-E11** [E, state/lifecycle; L73-L77] Brain heartbeat interval is 5 seconds. `BrainManifest::validate` has four rules and `FactoryLineConfig::default_unix` defaults `heartbeat_interval_ms=5000`; live heartbeat/reconnect/re-register loop remains pending.
- **B25-E12** [E, deletion/non-goal; L78-L79] Repo-wide `brainctl` count is zero, satisfying deletion by absence; the direct driver remains pending.
- **B25-E13** [E, config/default; L80-L89] Event ingestion defaults to 100 events/s with configurable rate, event-type allowlist, and priorities `LOW|MEDIUM|HIGH`; config knob exists but live rate limiter/filter/classifier remain pending. Grounding uses `EmpireEventCheck { source, event_type }` plus `requires_empire_bridge` as `SOURCE_CHECK`-equivalent evidence.
- **B25-E14** [E, events/schema; L90-L93] Five lockstep SSB action types are `empire_connected`, `empire_disconnected`, `empire_event_received`, `empire_capability_invoked`, and `empire_repair_proposed`; all five received the full “QUINTUPLE-UPDATE” and schema bump `101 -> 102`.
- **B25-E15** [E, permissions/security; L94-L103] `ElaiNeuron::tool_filter` reuses `elai_ipc_types::ToolFilter`; `ElaiNeuron::allows` is authoritative and adversarial tests prove denial of six internal capabilities. MCP request dispatch remains pending.
- **B25-E16** [E, repair/data; L104-L114] `CircularDep` has shipped heuristic cycle detection but pending AST walker; `HardImport` hard-coded-path heuristic shipped; `StdlibShadow` shipped with a 24-name stdlib list and POSIX/Windows path tolerance. Shipped types are `EmpireRepairKind`, `EmpireRepairProposal`, `EmpireRepairError`, and `detect_repair_kind(symbol_path)`; generation/validation/commit remain pending.
- **B25-E17** [E, config/fallback; L115-L116] Feature-gated no-Empire operation is shipped and must remain functional.
- **B25-E18** [E, surface/status; L117-L122] `EmpireBridgePanel` is mounted in SettingsWindow between `runtime backends` and `cloud`; connection state shipped, while last-20 events, capability status, and repair results remain pending.
- **B25-E19** [E, exact symbols; L129-L150] Contract produces `FactoryLineClient`, `ElaiNeuron` MCP server, repair pipeline, transport trait/implementations, and the five SSB types; consumers are GRC grounding and EmpireBridgePanel; reachability is CLI `--empire`/feature; expected evidence is `empire_connected` within 5s, per-event `empire_event_received`, and live dashboard stream.
- **B25-E20** [E, stub/security bans; L141-L150] Empire connection failure must not crash ELAI, and MCP must not expose internal state.
- **B25-E21** [E, deterministic evidence; L152-L159] Static gates look for `feature="empire"` in orchestrator Cargo configuration and `trait Transport` under the Empire module.
- **B25-E22** [E, acceptance/done gate; L161-L172] Full completion requires all nine sub-step deliverables, correct FactoryLine protocol, live registration/reconnect, zero-Python brainctl replacement, rate/filter/priority subscription, four guarded capabilities, three detected/generated repair pipelines, feature gating, graceful degradation, and live panel.
- **B25-E23** [E, files/evidence; L174-L210] Shipped file ledger explicitly names seven Empire module files, Cargo feature, SSB update sites, action-type lock test/golden update `298 -> 303`, reasoning verification type, V2 `AppState.empire_bridge`, `with_empire_bridge`, `empire_bridge_snapshot`, `commands_empire` disconnected fallback, UI/TS wrapper/settings mount, CLI `--empire` on `elai run`, absent Python shims, and decision doc; the four pending implementation clusters remain deferred. The B-25 doc records seven residual decisions layered on six pre-resolved Phase-4J decisions.
- **B25-C01** [C, protocol; L33-L36 vs L21-L23] Context says both buses use newline-JSON, while status demands an FSB binary transport-header parser. Whether FSB wraps/replaces newline JSON is not explained.
- **B25-C02** [C, status; L12-L26 vs L161-L172] Header says conditional and residual scaffolding shipped, but every acceptance box remains unchecked and major live behavior is pending. “Shipped” cannot be read as end-to-end Phase 4J completion.
- **B25-C03** [C, taxonomy; L44-L45, L86-L89] High-level text calls Empire events `SOURCE_CHECK`; implementation names distinct `VerificationMethod::EmpireEventCheck`. Treat as semantic role versus concrete variant, not identical identifier.
- **B25-A01** [A, sub-step count; L30-L31, L38-L51, L124-L127, L161-L164] Source says nine sub-steps but details a six-step integration plan plus tasks 1-9; exact 4J.7-4J.9 feature boundaries are not enumerated here.
- **B25-A02** [A, failure/retry; L73-L77] Reconnect backoff, retry ceiling, heartbeat failure threshold, duplicate-registration idempotency, and delivery ordering across reconnect are unspecified.
- **B25-A03** [A, event security; L80-L93] Default allowlist contents, priority mapping, overflow/drop/backpressure policy, and whether dropped events are observable are unspecified.
- **B25-A04** [A, repair mutation; L104-L114] Commit permissions, rollback, concurrency with Empire, and failure recovery for generated repair pipelines are deferred/unspecified.

### 4. `B-27_cloud_mode.md`

Status/source posture: planned D-1..D-4 implementation with unchecked acceptance gates. It depends on B-10 profile activation and B-15..B-20 hardening; parallel-safe with B-25/B-26/B-28 (L1-L10).

- **B27-E01** [E, modes/default; L17-L22] Per-task modes are `LOCAL` (default, full local pipeline, all six reasoning layers, $0), `CLOUD` (cloud generation, deterministic L1 still runs, L2-L5 skipped), and `HYBRID` (local first; only the failing L3+ step escalates).
- **B27-E02** [E, integration; L24-L27] Routing reuses the providers crate. `CLOUD_SPEND` records cost; per-task and per-day ceilings and pre-task estimation are mandatory.
- **B27-E03** [E, context defaults; L29-L44] Cloud versus local: context 200K vs 8-25K; baby step 200-500 LOC vs 25-40; depth 2 vs 4; phase split 50K tokens vs 3K; telescope optional vs mandatory. Cloud slots are `system_prompt=500`, `rules=200`, `phase_spec=2000`, `repo_map=4000`, `code_context=150000`, `ssb_entries=2000`, `grc_history=1000`, `output_reserve=8000`.
- **B27-E04** [E, privacy/default; L46-L51] `redact_env_vars` defaults true; `redact_patterns` is a regex list; `blocked_paths` is a glob list. Every cloud call audits timestamp, provider/model, token count, cost, included files, and redaction count.
- **B27-E05** [E, integration/privacy; L53-L58] BYOE accepts an API-compatible endpoint such as vLLM/TGI/Ollama on user-owned hardware. Air-gapped mode explicitly disables all external networking; nightly dependency audit, cloud escalation, and Oracle web search degrade off while core operation continues.
- **B27-E06** [E, escalation workflow; L60-L62] On local Layer 3+ failure, emit `ESCALATE_TO_CLOUD`, switch provider for that step only, then continue; D-3 redaction applies to escalated context.
- **B27-E07** [E, config/schema/default; L76-L91] `[cloud]` fields/defaults are `provider="anthropic"`, `model="claude-sonnet-4-6"`, `escalation_model="claude-opus-4-6"`, `api_key_env="ANTHROPIC_API_KEY"`, `max_spend_per_task_usd=2.00`, `max_spend_per_day_usd=10.00`, `warn_at_percent=80`; `[execution].default_mode="local"`.
- **B27-E08** [E, provider routing/fallback; L93-L99] Routing steps are read config, verify API key, probe model availability with a <100ms tokenize operation, use `[cloud.fallback]` if probe fails, and degrade to LOCAL with a notification when no fallback exists.
- **B27-E09** [E, spend lifecycle; L100-L101] `CLOUD_SPEND` accumulates per task and per day and enforces both ceilings.
- **B27-E10** [E, surfaces/config; L103-L106] `/task --mode cloud|hybrid|local` selects mode; `/config` changes the default; session metadata gains `execution_mode`.
- **B27-E11** [E, context profile; L108-L110] `[context_budget.cloud]` implements all eight specified slots.
- **B27-E12** [E, sizing/defaults; L111-L113] Baby-step LOC values are `local_3b=25`, `local_7b=40`, `local_32b=100`, `cloud_sonnet=300`, and `cloud_opus=500`; maximum decomposition depth is LOCAL 4 and CLOUD 2.
- **B27-E13** [E, reasoning/retry; L114-L116] CLOUD runs L1 deterministic plus L6 generation, skipping L2-L5. On L1 failure, inject findings, retry at most three times, then use `escalation_model`.
- **B27-E14** [E, hybrid workflow; L117-L117] HYBRID runs L1-L4 locally and makes one cloud call on L3+ failure.
- **B27-E15** [E, privacy schema/defaults; L119-L128] `[cloud.privacy]` includes `allow_source_upload=true`, `allow_test_upload=true`, `redact_env_vars=true`, regex patterns (example API-key assignment), and blocked defaults `.env`, `*.pem`, `*.key`, `credentials.*`.
- **B27-E16** [E, privacy workflow; L130-L131] Redaction runs before context assembly and substitutes `[REDACTED:type]`.
- **B27-E17** [E, audit/privacy; L133-L134] Cloud-call audit stores timestamp, provider/model, token counts, cost, file paths only (never content), and redaction count.
- **B27-E18** [E, BYOE config; L136-L137] BYOE is `[cloud] provider="custom", custom_endpoint=URL` through the OpenAI-compatible adapter.
- **B27-E19** [E, air-gap config; L139-L140] `[execution] air_gapped=true` disables all external network and requires graceful feature disablement.
- **B27-E20** [E, event/workflow; L142-L145] `ESCALATE_TO_CLOUD` records L3+ failure escalation, limits provider switch to the current step, and preserves privacy redaction.
- **B27-E21** [E, cost gate; L147-L148] Estimate is `phase_count * avg_tokens * pricing`; user sees a range such as `$0.12-$0.35` and must answer Proceed `[y/n]`.
- **B27-E22** [E, UI/observability; L150-L157] Monitor status shows running per-task spend including `$0.00`. At default 80%, COMMAND receives a warning; at 100%, HYBRID degrades and CLOUD fails the step. CloudPanel controls provider/model and air-gap, and shows per-task/per-day gauges plus redaction statistics.
- **B27-E23** [E, exact symbols; L159-L188] Produced orchestrator symbols are `CloudModeRouter`, `CloudConfig`, `ExecMode`, `RoutedProvider`, `SpendAccumulator`, `RedactingBabyStepExecutor`, `estimate_task_cost_usd`, `resolve_exec_mode`, `baby_step_loc_for_mode`, `route_with_spend`, `escalate_hybrid_step_to_cloud`, and `prompt_cost_estimate_gate`; privacy symbols are `redact_context`/`PrivacyConfig`; context is `cloud_200k`; SSB types are `CloudSpend`, `EscalateToCloud`, `RouteBlocked`, `CostEstimate`, `WarnLimitReached`, `RedactionApplied`, `ExecutionModeChanged`; CLI config is `CloudConfigToml`; frontend is CloudPanel; commands are `set_execution_mode`, `get_execution_mode`, `submit_task`.
- **B27-E24** [E, reachability/dependencies; L189-L197] Consumers/reachability are `runtime_setup::apply_task_runner_wiring`, `task_runner::start_task_runner`, `run_subcommand::run_async`, `task_cmd::drive_task`, and dashboard `submit_task`.
- **B27-E25** [E, security/stub bans; L198-L202] A cloud executor without redaction is prohibited; `route_with_spend` must enforce declared caps; `redact_context` must scrub environment-token secrets.
- **B27-E26** [E, evidence; L203-L217] Smoke evidence is `cloud_spend` plus dashboard running spend; static checks require config schema, `CloudSpend`, and orchestrator privacy-redaction tests.
- **B27-E27** [E, acceptance/done gate; L219-L230] All four D groups, three selectable modes, spend ceilings, cloud profile/sizing/collapse, privacy defaults/BYOE/air-gap, single-call HYBRID escalation, visible estimate, and functional CloudPanel are required.
- **B27-E28** [E, lifecycle/files; L232-L246] Create `cloud_mode.rs` and `privacy_redactor.rs`; modify context budget, TaskRunner, SSB, CLI flags, V2 Tauri/UI, TOML schemas; create `B-27-decisions.md`.
- **B27-C01** [C, cap identifier; L85-L86 vs L198-L201] Configuration and prose define per-task/per-day caps, but stub ban says `route_with_spend` must enforce daily/**monthly** caps. No monthly field is defined.
- **B27-C02** [C, CloudPanel path/name; L185 vs L240-L244] Wiring contract places `CloudPanel.svelte` under `screens/`; files ledger says `components/CloudPanel.svelte` consumed by SettingsWindow.
- **B27-C03** [C, context arithmetic; L36-L44] The eight cloud slot values sum to 167,700, not 200,000. “200K profile” therefore denotes model capacity, not a fully allocated total; remaining headroom policy is unstated.
- **B27-A01** [A, fallback config; L93-L99] `[cloud.fallback]` fields, ordering, probe retry, authentication-error policy, and notification schema are not defined.
- **B27-A02** [A, spend concurrency; L100-L101, L147-L154] Pricing source/version, currency/rounding, atomicity across concurrent sessions, day timezone/reset boundary, reservation versus post-call accounting, and idempotency on retry are unspecified.
- **B27-A03** [A, 100% behavior; L153-L154] “HYBRID degrades” does not state whether it continues locally, pauses, or requests intervention; CLOUD “fails step” does not state recovery/replan semantics.
- **B27-A04** [A, privacy semantics; L121-L140] Interaction between upload booleans, blocked paths, regex redaction, symlinks, binary files, and custom endpoint trust is unspecified; audit retention/access control is also absent.
- **B27-A05** [A, user gate; L147-L148] Non-interactive/autonomous behavior when no human can answer the cost estimate is not specified.

### 5. `B-31.5_resumable_indexing_scale_benchmark.md`

Status/source posture: planned, hardware-heavy release benchmark (about 6-8 hours stated, though phase estimates total much longer wall time) over B-31. It reuses B-29.5 fixtures and can run alongside B-29.5 because metrics are orthogonal (L1-L13).

- **B315-E01** [E, objective/determinism; L1-L10] Prove per-batch checkpointing, `IndexLock`, startup auto-resume, and graceful shutdown on multi-million-LOC repositories under 10/50/90% SIGKILL, concurrent instances, and load; required output is byte-identical non-timestamp state with bounded memory.
- **B315-E02** [E, scale rationale; L15-L37] Scale must exercise 50K files/1000+ SQLite checkpoint writes, the 30-second lock refresh over multi-hour runs, a second process starting 3 seconds later, and a 60-second drain deadline with >60 seconds in flight; this is the evidence gate for a “50M LOC” claim.
- **B315-E03** [E, fixtures/dependency; L33-L47] Real fixtures are Linux, Chromium, VS Code, and Postgres, acquired exactly as B-29.5. Referenced SocratiCode coverage is only about 1,000 files, while this plan targets roughly 1,000× larger.
- **B315-E04** [E, fixture determinism; L55-L64] Reuse `scripts/fixtures/clone_scale_fixtures.sh`; pinned revisions are Linux `v6.7`, Chromium `121.0.6167.184`, VS Code `1.94.2`, and Postgres `REL_17_0`.
- **B315-E05** [E, lifecycle/reset; L65-L71] Every fixture’s existing `.elai_cc/` is deleted before a fresh scenario so baselines and kill percentages do not inherit state.
- **B315-E06** [E, Tier A workflow; L73-L83] Uninterrupted indexing on all four fixtures records total wall time, per-batch mean/p99 (drift indicates leak), peak RSS, total versus checkpointed batches (must match), and final `dkg_nodes` as resume ground truth.
- **B315-E07** [E, evidence/artifact; L84-L91] Capture each final DKG as gzipped SQLite `.dump` at `metrics/baseline_<tier>.sql.gz` for Tier B identity comparison.
- **B315-E08** [E, metrics schema; L93-L104] `metrics/phase_b31_5_tier_a.toml` records per fixture `wall_secs`, `peak_rss_mb`, `batches_total`, `batches_checkpointed`, `per_batch_p50_ms`, `per_batch_p99_ms`, `final_dkg_nodes`, and `final_chunk_count`.
- **B315-E09** [E, kill harness; L106-L133] `scripts/scale_test/kill_at_progress.sh <fixture> <kill_at_pct>` starts indexing, polls the latest SSB checkpoint every 5 seconds, computes `batches_processed*100/batches_total`, and sends SIGKILL only once the requested deterministic progress threshold is reached.
- **B315-E10** [E, deterministic signal; L108-L133, L345-L349] Kill injection must poll SSB checkpoint rows, never file size or wall clock.
- **B315-E11** [E, Tier B matrix; L135-L157] Linux and VS Code are mandatory; Chromium and Postgres are nice-to-have. Each fresh fixture is killed at 10%, 50%, and 90%, resumed, then node-count and SQL-dump diffs are persisted, producing six mandatory runs.
- **B315-E12** [E, identity criterion; L158-L160] Diff must be empty except entropy-quarantined `last_indexed_at` timestamps; every non-timestamp column must match byte-for-byte.
- **B315-E13** [E, resume state schema; L162-L173] Tier-B tables record `killed_at_batch`, `resumed_from_batch` (must equal progress at kill), `resume_wall_secs`, positive `files_skipped_on_resume`, `identity_diff_lines=0`, and exactly one `ssb_index_resume_action_count`.
- **B315-E14** [E, Tier C race; L175-L194] Start instance A, wait 3 seconds, then B on the same Linux fixture. B must exit within 1 second; stderr must contain “Another process is already indexing this project, skipping,” holder PID must be logged, and `.elai_cc/.index.lock` must contain A’s PID.
- **B315-E15** [E, stale-lock recovery; L196-L209] After SIGKILL leaves a Postgres lock, the next indexer detects the dead PID, reclaims without manual deletion, resumes, and journals `LockContested` then `LockAcquired`.
- **B315-E16** [E, lock metrics; L211-L223] Tier-C metrics store B exit milliseconds/message/PID evidence plus orphan detection, reclaim time, resume completion, and exactly one contested/acquired action; contract threshold for reclaim is <5 seconds.
- **B315-E17** [E, graceful shutdown; L225-L240] On Chromium after 60 seconds of work, SIGTERM must finish within 65 seconds (60-second deadline plus 5-second slack), emit `IndexCancel` with nonzero completed batches, and release the lock.
- **B315-E18** [E, forced recovery; L242-L251] `ELAI_DEBUG_BATCH_DELAY_SECS=120` simulates a stuck Postgres batch; SIGTERM after 30 seconds must force release and exit within 65 seconds, logging `force-released at <ts> by pid <X>`.
- **B315-E19** [E, shutdown metrics; L253-L265] Tier-D metrics distinguish clean and forced cases and record wall time, configured deadline, deadline honored/exceeded, cancel count, lock release, and forced-release log evidence.
- **B315-E20** [E, Tier E long-tail; L267-L279] A complete Chromium ~35M-LOC run records wall time, RSS every 30 minutes, per-batch wall over time, final node/edge counts, and embedding rate-limit evidence including sidecar 429 and OpenAI backoff events.
- **B315-E21** [E, memory failure gate; L280-L293] Tier-E metrics include RSS samples, p50/p99 batch time, provider-rate-limit count, final nodes/chunks. Monotonic RSS without a plateau reopens B-31 as a memory leak.
- **B315-E22** [E, aggregate schema; L295-L307] `metrics/phase_b31_5_scale.toml` has boolean pass fields for tiers A-E and `overall_pass`; all must be true in the illustrated aggregate.
- **B315-E23** [E, contract test; L309-L314] `scale_resume_identity_contract.rs` reads populated Tier-B metrics and asserts zero identity diffs, positive skipped files, and exactly one `IndexResume` per resume; any failure breaks CI.
- **B315-E24** [E, contract test; L316-L319] `scale_lock_contention_contract.rs` requires instance-B exit <1000ms and stale reclaim <5s.
- **B315-E25** [E, contract test; L320-L323] `scale_long_tail_memory_contract.rs` requires the RSS sequence not to remain monotonically growing—there must be a plateau or drop.
- **B315-E26** [E, exact symbols/dependencies; L325-L344] Produced scripts/contracts are `clone_scale_fixtures`, `kill_at_progress`, and the three named contracts. They consume B-31 `IndexingService::build_index`, `IndexLock`, `auto_resume_indexed_projects`, `graceful_shutdown`, and `SsbActionType::{IndexCheckpoint, IndexResume, IndexCancel, LockAcquired, LockContested}`; `run_all_tiers.sh` and three cargo-test entry points provide reachability.
- **B315-E27** [E, evidence/fail-closed; L345-L355] Contract tests must read populated metrics and may not default-pass when files are missing. End-to-end artifact is `phase_b31_5_scale.toml` with `overall_pass=true` plus green `scale_*` tests.
- **B315-E28** [E, deterministic checks; L357-L388] Prescribed sequence is Tier A four clean indexes with per-fixture upper bounds (example Linux <4h on i9+64GB), Tier B six crash runs plus contract, Tier C two scenarios plus contract, Tier D two <65s shutdown paths, Tier E Chromium plus memory contract, then aggregate grep.
- **B315-E29** [E, acceptance/Tier A-B; L390-L399] Done requires clean all-four fixture baselines, baseline SQL artifacts, all six mandatory resume diffs at zero, exactly one `IndexResume` each, and positive hash-skip evidence.
- **B315-E30** [E, acceptance/Tier C-D; L400-L409] Done requires B exit <1s with explicit message/PID, orphan reclaim <5s without deletion, graceful shutdown <65s with cancel/release, and synthetic forced-release log evidence.
- **B315-E31** [E, acceptance/Tier E; L410-L415] Chromium must finish, RSS must plateau, per-batch p99 must not progressively exceed 2× baseline, three CI contracts must pass, aggregate must pass, and no dependency may be added without explicit user confirmation.
- **B315-E32** [E, test/artifact inventory; L417-L436] Deliverables enumerate one shared fixture script, kill harness, `run_tier_a` through `run_tier_e`, `run_all_tiers`, three Rust contracts, six TOML artifacts (A-E plus aggregate), and a reproduction README.
- **B315-E33** [E, file lifecycle; L438-L447] Create seven scale-test shell scripts, three contracts, six metrics TOMLs, README, and `B-31.5-decisions.md`; modify indexing service only if the debug-delay hook is needed.
- **B315-E34** [E, release decision; L449-L459] Decision doc must choose official hardware; decide all-five release gate versus A+B+C minimum. Recommendation is A+B+C minimum, with D+E required for “production-ready”; Linux+VS Code mandatory, Chromium long-tail and Postgres C-focus optional, while Chromium is recommended as an Empire D2 ship blocker.
- **B315-E35** [E, deadline decision; L460-L463] Reaffirm the 60-second shutdown rationale or raise to 120 seconds when cloud embedding batches commonly exceed 60 seconds.
- **B315-E36** [E, identity decision; L464-L466] Choose byte-exact versus chunk-count-equivalent identity; recommendation is byte-exact non-timestamp data because blake3 file hashes are deterministic.
- **B315-E37** [E, observability limitation; L467-L470] A four-hour non-monotonic RSS check is necessary but insufficient for slow leaks; record the caveat and recommend a nightly long-haul soak.
- **B315-E38** [E, CI cadence decision; L471-L473] Tier B should not run every commit; nightly is desirable, while pre-release-only is recommended until nightly budget is approved.
- **B315-E39** [E, side effects/quality gates; L475-L491] Benchmark grounds scale claims, detects checkpoint off-by-one/hash/deletion regressions, exit deadlocks, enterprise slow leaks, supplies B-29.5 baseline and B-25 corpus evidence, and establishes scale rerun before merging indexing-service changes.
- **B315-E40** [E, downstream; L493-L499] B-29.5 shares fixtures and may run in parallel; after Tier B passes, B-30 should add per-language coverage to the benchmark.
- **B315-C01** [C, contract count; L345-L355 vs L309-L323, L390-L414] `stub_ban` and observable text say “all four contract tests,” but only three contracts are named, inventoried, and accepted.
- **B315-C02** [C, metrics layout; L93-L104 vs L360-L363] Task defines one `phase_b31_5_tier_a.toml` with fixture tables; deterministic check expects four `phase_b31_5_<tier>_tier_a.toml` files. File inventory returns to one Tier-A TOML.
- **B315-C03** [C, SSB path; L115 vs other source paths L87-L88] Kill harness uses `.elai_cc/.elai/sessions/ssb.db`, while DKG uses `.elai_cc/.elai_cc/indexes/dkg.db`; whether `.elai` is intentional legacy/session layout is not reconciled.
- **B315-C04** [C, duration estimate; L8-L9 vs L73, L106, L267] Header estimates one 6-8-hour session, but Tier A, B, and E each estimate 3-4 waiting hours, excluding setup/C/D/aggregation; total serial wall time exceeds that estimate.
- **B315-C05** [C, release gate; L295-L307, L390-L415 vs L449-L455] Aggregate/acceptance require A-E `overall_pass=true`, while decision recommendation gates ordinary B-31 release on A+B+C and reserves D+E for production-ready. Two different “done” levels need naming.
- **B315-C06** [C, metrics fields; L75-L83, L93-L104, L269-L290] Tier A asks for per-batch mean and p99 but its sample schema stores p50 and p99; Tier E asks for final `dkg_nodes/dkg_edges` but its sample stores `final_dkg_nodes/final_chunk_count`. Exact metric contract needs reconciliation.
- **B315-A01** [A, fixture optionality; L135-L136, L267-L293, L449-L459] Chromium/Postgres are nice-to-have for Tier B, yet Chromium is mandatory for Tier E and recommended as an Empire blocker; exact minimum matrix depends on the selected release stamp.
- **B315-A02** [A, identity method; L151-L160] SQL `.dump` comparison allows timestamp exceptions but does not specify canonical row ordering or normalization procedure, so “byte-identical non-timestamp columns” needs a deterministic comparator.
- **B315-A03** [A, process targeting; L108-L133] Harness starts `elai.bat` in the background and kills its recorded PID; whether that PID is the real worker versus a wrapper/child is not specified.
- **B315-A04** [A, memory test; L320-L323, L467-L470] Any single plateau/drop passes the proposed monotonic test even if growth resumes; leak slope/window thresholds are explicitly acknowledged as insufficient and deferred to soak testing.

### 6. `B-33_interactive_graph_viewer.md`

Status/source posture: planned dashboard/CLI visualization with unchecked acceptance. It requires B-7, B-29, and B-36 Session 2, except Mermaid export can land independently; B-26 is parallel-safe (L1-L14).

- **B33-E01** [E, capabilities; L1-L8] Visualize file imports, symbol calls, and Concept→Phase→Step→BabyStep decomposition; support right-click impact radius, live search, layout switching, PNG export, and Mermaid text for PR/Claude/webview-free use.
- **B33-E02** [E, V2 rebaseline; L16-L31] Resolve V1 commands/windows/drawers/package/tests into V2 `src-tauri/src/lib.rs`, `src/lib/windows`, `src/lib/drawers`, V2 `package.json`, and `src-tauri/tests`.
- **B33-E03** [E, dependency/permission; L33-L43] A new GraphWindow would be V2’s eighth window and requires user confirmation plus possible DOMAIN-CONTEXT update. `cytoscape` is a new ~80KB-gzipped dependency and requires confirmation; existing three.js/troika can instead render MAP. Mermaid text is pure data with no dependency. Svelte code must use `$state/$effect`, never `$:`.
- **B33-E04** [E, surface options; L45-L56] Execution must choose A: cytoscape plus an eighth GraphWindow; B: ActivityWindow orbit Mode-2 MAP; or C: KnowledgeDrawer-only. B/C are recommended unless a contextual right-click flyout demands A.
- **B33-E05** [E, current gap; L58-L78] Invisible inputs include DecompositionTree, ExecutablePipeline DAG, repair events, per-node research stages, file imports, symbol calls, and edit blast radius; existing PartialPlanTree only polls `PhaseComplete` rows.
- **B33-E06** [E, reference constraints; L80-L97] Reference UX has 20K-symbol/60K-edge caps, file/symbol toggle, click/right-click/search, six layouts, PNG, language colors, and red cyclic edges. Implementation uses npm libraries, not vendored AGPL JavaScript; interactive view serves immediate exploration, Mermaid serves portable sharing.
- **B33-E07** [E, schema; L120-L135] `VizData` contains `project: VizProject`, `files`, `file_edges`, `symbols`, `symbol_edges`, `symbols_by_file: BTreeMap<String, Vec<VizSymbol>>`, `symbol_mode: Full|Capped|Omitted`, and optional `symbol_omit_reason`.
- **B33-E08** [E, schema; L136-L143] `VizProject` fields are `name`, `files`, `edges`, optional `symbols`, optional `call_edges`, and optional `built_at: i64`.
- **B33-E09** [E, schema; L144-L157] `VizFile` has relative-path `id`, basename `label`, `language`, `deps`, `dependents`, and `symbol_count`; `VizFileEdge` has `source`, `target`, `kind: Import|DynamicImport|ReExport`, and `cyclic`.
- **B33-E10** [E, schema; L158-L170] `VizSymbol` has `id`, `file`, `name`, `qualified_name`, `kind`, and `line: u32`; `VizSymbolEdge` has `source`, `target`, and `knowledge::symbol_graph::Confidence`.
- **B33-E11** [E, bounds/fallback; L176-L184] Constants are `MAX_VIZ_SYMBOLS=20_000`, `MAX_VIZ_EDGES=60_000`, `MAX_VIZ_SYMS_PER_FILE=200`. Above symbol/edge caps return Capped or Omitted with a reason directing users to `/symbol_impact target=X`.
- **B33-E12** [E, command/schema; L186-L205] `get_graph_view_data(project_id, mode)` returns `VizData`; mode is `file|symbol|decomposition`.
- **B33-E13** [E, file mode; L196-L198] File mode loads imports, per-file symbol counts, and at most 200 sidebar symbols per file; total size determines Full/Capped/Omitted.
- **B33-E14** [E, symbol mode; L199-L199] Symbol mode adds all symbol edges only when under caps.
- **B33-E15** [E, decomposition mode; L200-L204] Decomposition mode reads SSB or live TreeNavigator state and maps Concept/Phase/Step/BabyStep nodes to `VizFile`, with parent-child and dependency arrows as `VizFileEdge`.
- **B33-E16** [E, impact command/data; L206-L217] `get_impact_overlay(project_id, target, depth)` wraps B-29 `compute_impact_radius` and returns `{files: BTreeSet<String>, by_depth: BTreeMap<u32, Vec<String>>}` for hop-band highlighting.
- **B33-E17** [E, export command; L219-L239] `export_graph_mermaid(project_id, mode, max_nodes)` returns a string for file/symbol/decomposition; generator lives at `knowledge/src/visualization/mermaid.rs`.
- **B33-E18** [E, Mermaid behavior; L228-L239] Mermaid output includes per-language `classDef`, red dotted `-.->|cycle|` cyclic edges, sanitized compact path IDs, legend, header, and footer. `max_nodes` defaults 200; caller owns copy/save/inline presentation.
- **B33-E19** [E, UI layout; L241-L247] GraphWindow has header with project/stats/mode, cytoscape canvas, selected-node right sidebar, and footer search/layout/Mermaid/PNG controls.
- **B33-E20** [E, UI states; L248-L254] Files mode always enabled; Symbols is disabled on Omitted with `/symbol_impact` hint; Decomposition enabled only when current task has a tree. Adding `cytoscape`, `cytoscape-dagre`, and `cytoscape-fcose` requires explicit user confirmation.
- **B33-E21** [E, node actions; L256-L259] Click shows language, line range, inbound/outbound dependencies, at-most-200 symbols, and actions `Trace flow from here`, `Show blast radius`, `Open file`.
- **B33-E22** [E, impact interaction; L260-L262] Right-click calls impact command, colors depth 1 bright red and deeper hops progressively lighter, and shows an overlay hop counter.
- **B33-E23** [E, search/layout; L263-L266] Search fuzzy-matches label, qualified name, and path live. Layout choices are `dagre`, `fcose`, `concentric`, `breadth-first`, `grid`, and `circle`.
- **B33-E24** [E, export workflow; L267-L270] PNG uses `cy.png({output:'blob', full:true})` and Tauri save dialog. Mermaid button opens a text modal with clipboard and `.mmd` save actions.
- **B33-E25** [E, visual design; L272-L281] Node colors are Rust teal `#1B7A6E`, Python `#3776AB`, TS/JS `#F7DF1E`, Svelte `#FF3E00`, Go `#00ADD8`, JVM `#7C5CBF`, C# `#239120`, and other neutral gray.
- **B33-E26** [E, edge/state design; L282-L285] Import edges solid, dynamic imports dashed, re-exports dotted, cycles red; selection adds halo and dims other nodes; impact overlay uses red gradient.
- **B33-E27** [E, decomposition states; L287-L294] Concept is large rounded rectangle, Phase medium rounded rectangle, Step small rounded rectangle, BabyStep circle. `pending` gray, `running` blue pulse, `passed` green, `failed` red, `blocked` orange; parent-child edges solid, dependencies dashed.
- **B33-E28** [E, registration/surface; L296-L302] Register GraphWindow in `WindowRegistry.ts`, add graph/network Sidebar icon and Ctrl+G, and provide Refresh that re-fetches while graph rebuilding follows B-31 watcher behavior.
- **B33-E29** [E, event/lifecycle; L303-L305] Subscribe to `IndexingComplete` and `SymbolGraphUpdated`; either automatically re-fetches the graph.
- **B33-E30** [E, cross-surface workflows; L306-L312] Plan node “Show in Graph” opens/focuses decomposition; Activity file-write “Show blast radius” opens file mode and runs impact; KnowledgeDrawer adds Visualize beside symbol results.
- **B33-E31** [E, CLI/integration; L313-L318] REPL `/graph mermaid > graph.md` maps to dispatcher tool `export_graph_mermaid`, emitting paste-ready text without dashboard.
- **B33-E32** [E, exact symbols; L320-L344] Produced types are `VizData`, `VizProject`, `VizFile`, `VizFileEdge`, `VizSymbol`, `VizSymbolEdge`, `SymbolMode`; generator is `generate_mermaid`; commands are the three named handlers; UI includes GraphWindow, GraphSidebarPanel, MermaidExportModal; SSB adds `SymbolGraphUpdated`; consumers are SymbolGraphCache, DKG engine, DecompositionTree, WindowRegistry; reachability is dashboard run and `cli::main::handle_graph_command`.
- **B33-E33** [E, fail-safe/stub bans; L345-L350] Non-Full mode requires a reason; GraphWindow must never feed >60,000 edges to cytoscape; empty graph Mermaid must still return project header rather than an empty `unwrap_or_default` result.
- **B33-E34** [E, observable evidence; L351-L358] Real-repo `/index`, Ctrl+G, and file click should produce `SymbolGraphUpdated`, about 16 ELAI crate nodes/edges, and an orchestrator impact overlay highlighting cli/dashboard/contracts.
- **B33-E35** [E, checks; L360-L397] Gates include compiled/registered commands, CLI Mermaid header/classes, manual upstream rendering, >1KB PNG, overlay count parity with `/symbol_impact`, disabled symbol mode at 25K symbols, repaint within 2s of IndexingComplete, and confirmed package dependencies.
- **B33-E36** [E, acceptance; L399-L419] Required: stable VizData contract, all three commands/modes, hop overlay, search, six layouts, PNG, valid deterministic Mermaid 11.x, CLI stdout, caps/reasons, Plan/Activity cross-links, auto-refetch, explicit dependency approval, and zero non-test `.unwrap()`.
- **B33-E37** [E, tests; L421-L436] Tests cover VizData round-trip/caps; Mermaid cycle/color/parser/determinism/truncation; Tauri fixture smoke; CLI end-to-end; and `graph_viewer_contract.rs` schema stability.
- **B33-E38** [E, file lifecycle; L438-L461] Create visualization modules and three Svelte components; modify registry/sidebar/package/CLI/dispatcher/SSB/Activity; package edits remain pending user confirmation; create `B-33-decisions.md`.
- **B33-E39** [E, decision/dependency evidence; L463-L472] Decision doc records ~370KB cytoscape, ~50KB dagre plugin, ~80KB fcose plugin (~500KB total), user approval, npm/tree-shake rationale versus vendoring, and Mermaid 11.x minimum.
- **B33-E40** [E, bounds/decision; L473-L479] Decomposition caps at 500 nodes with `/plan list` fallback; impact defaults to right-click rather than selection; Mermaid labels include node line counts/sizes when graph exceeds 50 nodes.
- **B33-E41** [E, downstream; L481-L493] This satisfies B-7 visualization, enables future repair before/after overlays, portable PR diagrams and CI artifacts, visual UX improvement, and graph-node references for B-26 EXPLORE.
- **B33-C01** [C, chosen architecture; L45-L56 vs L92-L97, L241-L319, L399-L418] Source keeps A/B/C placement unresolved and recommends B/C, but all task, contract, and acceptance detail assumes Option A GraphWindow + cytoscape. Those tasks are conditional on the decision, not an already-ratified surface.
- **B33-C02** [C, path drift; L16-L31 vs L188-L190, L243, L298-L318, L438-L460] V2 paths are authoritative, yet tasks/contracts/files repeatedly use V1 `crates/dashboard` and `dashboard/svelte` paths and names.
- **B33-C03** [C, window baseline; L33-L36 vs L64-L66] V2 rebaseline says seven existing windows; older context says dashboard ships nine. They refer to different baselines and cannot determine registration count without V2 inventory.
- **B33-C04** [C, node caps; L176-L184, L236-L237, L473-L475] Symbol viewer caps at 20K symbols/60K edges; Mermaid defaults 200 and cites ~250 visual limit; decomposition separately caps 500. These bounds apply to different modes but source sometimes uses generic “graph” language.
- **B33-C05** [C, file operation; L188-L190 vs L443-L444] Task says add three commands to existing `commands.rs`; file ledger labels the same path `CREATE` and then says “add to existing file.”
- **B33-C06** [C, line schema; L158-L165 vs L256-L259] `VizSymbol` carries only one `line: u32`, but sidebar promises a line range. An end-line field or derived lookup is missing from the declared transport shape.
- **B33-A01** [A, permission gate; L33-L56, L252-L254] User confirmation is required both for an eighth window and new packages, but behavior if approval is denied—automatic B/C fallback versus stop—is not specified.
- **B33-A02** [A, determinism; L122-L170, L425-L430] Same-graph/same-string is required, but ordering rules for vector-backed files/edges/symbols and handling of optional `built_at` are not defined.
- **B33-A03** [A, graph freshness/concurrency; L298-L305] Refresh during a background rebuild and simultaneous SSB refetch lacks versioning, cancellation, deduplication, or stale-response handling.
- **B33-A04** [A, export security; L145-L170, L231-L239] Escaping of Mermaid labels/quotes/control text beyond ID sanitization and save-path permission/error behavior are unspecified.

### 7. `B-36_dashboard_v2_src_tauri_wiring.md`

Status/source posture: this canonical file is a four-session Track-D plan with unchecked acceptance; it says Tracks A/B/C were complete and D was deferred by user instruction. The orientation summary records a later follow-up as done, but those implementation facts are not asserted by this original (L1-L21).

- **B36-E01** [E, objective/scope; L1-L12] Implement 13 Track-D command rows plus 15 V2-only surfaces in `dashboard-v2/src-tauri/src/lib.rs`, replace matching `stub.ts` functions with live invokes, and keep B-29..B-35 parallel-safe; prerequisites are B-1/B-2/B-3/B-7.
- **B36-E02** [E, baseline/status; L14-L21] Track D is the rebaselined remainder of the finish-dashboard prompt after named Track A/B/C sessions; work was deferred by the explicit “no dashboard wiring for now” instruction.
- **B36-E03** [E, baseline commands; L23-L30] V2 backend initially registers exactly `greet` and `get_grounding_cache_stats`; the latter returns hard-coded zeros. Roughly 60+ frontend stubs use mock data, 80-240ms artificial latency, and `console.info('[stub]', name, args)`.
- **B36-E04** [E, dependency/permission; L31-L34] V2 initially has no engine-crate dependencies by design. Adding orchestrator/CLI/core/providers is Session-2 scope and requires user confirmation.
- **B36-E05** [E, launcher/config; L35-L38] CLI constant still names `elai-dashboard` while V2 builds `elai-dashboard-v2.exe`; `ELAI_DASHBOARD_BINARY` is an existing override.
- **B36-E06** [E, source-of-truth constraint; L40-L55] Only `get_grounding_cache_stats` maps verbatim from V1. Do not back-port 57 V1 handlers; authoritative shapes are DOMAIN-CONTEXT and TESTED-AND-DASHBOARD-WIRING.
- **B36-E07** [E, net-new provenance/data; L57-L65] New surfaces derive from MCP bridge (three SSB variants/settings row), freshness steering (`recompute_steering_vector` plus three endpoints), GG fields `mean_logprob`, `min_logprob`, `entropy_mean`, `entropy_max`, `generation_novelty`, cache-control boundary parsing, top-level `meta.gg_signals`, constrained-decoding `response_format`, and `/v1/messages.elapsed_ms`.
- **B36-E08** [E, analysis deliverable; L98-L107] Session 1 creates `track-d-v2-spec.md` with at least six sections, at least 13 §2 entries, a D2 row for each non-obsolete row, and OPEN_QUESTIONS including binary mismatch; it also creates a per-file B-plan delta using `CRITICAL|HIGH|MEDIUM|LOW` severity.
- **B36-E09** [E, handoff/evidence; L108-L111] Session 1 creates an implementation handoff with phase-cluster table, reading order, and acceptance gates; HANDOFF G1..G5 must pass and PLAN Auditor must report zero BLOCKs.
- **B36-E10** [E, dependencies/config; L113-L124] Proposed Cargo additions are path dependencies `elai-orchestrator`, `elai-cli`, `elai-core`, `elai-providers`, workspace Tokio with `rt-multi-thread,macros`, and workspace reqwest; confirmation precedes addition.
- **B36-E11** [E, state/concurrency; L126-L130] `AppState` owns shared `Arc<Mutex<...>>` handles to `Arc<dyn SsbWriter>`, `Arc<ResearchCache>`, and `ProjectPaths`, registered via `.manage(AppState::new(...))`.
- **B36-E12** [E, D.1 integration/schema; L132-L135] `sidecar_health()` probes `http://127.0.0.1:8100/health` or minimally `/v1/messages`, returning alive/model/VRAM/head_dim/layers and `dola`, `kv_quant`, `steering` flags in the cited canonical schema.
- **B36-E13** [E, D.13 commands/security; L137-L140] `get_sidecar_settings`, `set_sidecar_settings`, and `restart_sidecar` edit the canonical boot-command file; restart is SSH to the remote heavy box through `~/restart-sidecar.sh`, never local.
- **B36-E14** [E, launcher decision; L142-L145] Resolve launcher by renaming `DASHBOARD_BINARY_NAME` to `elai-dashboard-v2` or documenting `ELAI_DASHBOARD_BINARY=elai-dashboard-v2` in `elai.bat`; record the chosen branch.
- **B36-E15** [E, D.2 workflow/events; L147-L152] `run_planning_task(desc, tier?)` drives `/task --new` through `WebResearchRunner` and emits `web_research_step` events for live orbit-caption timeline.
- **B36-E16** [E, D.3 schema; L154-L156] `run_research(query)` drives `/research` and returns `ResearchAnswer {prose, citations, sources, path}`.
- **B36-E17** [E, D.7 integration/events; L158-L162] `one_shot_prompt(text, backend, port, allowed_tools)` spawns `elai.exe --backend=local` with `ANTHROPIC_BASE_URL`, `ANTHROPIC_API_KEY`, `ELAI_LOCAL_MIN_PROMPT`, `ELAI_LOCAL_MAX_CONTEXT`, streaming `turn`, `streaming_started`, and `response`.
- **B36-E18** [E, D.4 config/integration; L164-L167] `fetch_settings_get`, `fetch_settings_set`, and synchronous `obscura_test_fetch(url)` provide settings persistence and `obscura.exe` driving.
- **B36-E19** [E, D.5 test workflow; L169-L171] `run_d5_live_contract(autostart)` spawns a cargo test with `ELAI_LIVE_SIDECAR=1`; when autostart, first run `workflow/remote-setup/start-tunnel.ps1`; return per-query timings.
- **B36-E20** [E, D.6 lifecycle/schema; L173-L175] `reindex_now()` synchronously calls `IndexingService.reindex_all()` on active project and returns final `DkgReadyPayload`/`dkg_ready` row.
- **B36-E21** [E, D.8 permissions/observability; L177-L178] `get_tool_visibility()` returns `{advertised, hidden}` from `MODEL_HIDDEN_TOOL_NAMES` and `filter_tool_specs` output.
- **B36-E22** [E, D.9 config; L180-L182] `set_local_max_context(tokens?)` and `get_local_budget()` persist `ELAI_LOCAL_MAX_CONTEXT` to the settings file consumed by spawned ELAI.
- **B36-E23** [E, D.10 config; L184-L185] `set_local_min_prompt(enabled)` and `get_local_min_prompt()` persist `ELAI_LOCAL_MIN_PROMPT` through the same pattern.
- **B36-E24** [E, D.11 integration; L187-L189] `sidecar_search(query)` GETs `http://127.0.0.1:8100/search?q={query}` and maps to existing `SearchResult` TypeScript shape.
- **B36-E25** [E, D.12 observability; L191-L194] Upgrade `get_grounding_cache_stats()` from zeros to real `grounding_cache_event` SSB rows through AppState’s live writer/state handle.
- **B36-E26** [E, 15 commands; L196-L212] V2-only handler roster is `recompute_steering_vector`, `get_steering_status`, `get_mcp_status`, `get_gg_signals`, `get_dkg_stats`, `get_sidecar_status`, `get_bubble_up_events`, `get_hook_executions`, `get_shortcoder_stats`, `run_explore_query`, `continue_task`, `get_grc_budgets`, `get_evidence_gates`, `get_context_budget`, `get_search_mode`.
- **B36-E27** [E, frontend integration; L214-L222] Live frontend imports `invoke` from `@tauri-apps/api/core` and calls the exact command; retain `mock-data.ts` only for explicit offline preview selected by new `ELAI_DASHBOARD_OFFLINE=1`.
- **B36-E28** [E, produced surfaces/events; L224-L238] Contract produces 13 D-row handlers, 15 V2-only handlers, AppState, launcher-visible V2 binary, and event names `web_research_step`, `turn`, `streaming_started`, `response`; consumes SSB/cache/index/research/CLI types and sidecar `/v1/messages`, `/search`, `/health`.
- **B36-E29** [E, reachability; L239-L241] Dashboard is reachable through `elai.bat` after launcher fix and `elai.exe --dashboard`.
- **B36-E30** [E, fail-closed/stub ban; L242-L248] No live-command stub may return hard-coded mock data outside offline mode; commands must read real state or return typed `NotImplementedYet`, never `Ok(default())`; every D command has a window/drawer pair.
- **B36-E31** [E, evidence; L249-L253] Exercise each command from its surface, require corresponding real SSB rows within the specified-but-symbolic `Nms`, and capture `workflow/sessions/<sid>/screenshots/D-<item>.png`.
- **B36-E32** [E, deterministic gates; L255-L279] Gates count Tauri annotations (expected at least 28), inspect `generate_handler!`, run cargo/svelte checks, observe declining `logStub` count while offline mocks remain, and confirm launcher constant or documented override.
- **B36-E33** [E, acceptance; L281-L297] Done requires all 13 D rows, all 15 net-new commands, live invokes/offline mocks, release and Tauri builds, resolved launcher, approved deps, one screenshot per command, a ≥6-section/≥13-row spec, a delta reviewing ≥10 B-plan files, and a phase-cluster handoff.
- **B36-E34** [E, tests; L299-L306] Add one MockRuntime/process contract per D command, an `ELAI_LIVE_SIDECAR`-gated D.1+D.11 remote test, and Playwright/Vitest snapshots for every window.
- **B36-E35** [E, file lifecycle; L308-L324] Modify V2 lib from 28 to roughly 600+ lines, Cargo/stubs/CLI launcher; add AppState and optional per-command modules if lib exceeds ~800 LOC; add spec/delta/handoff; write per-session B-36 decision records.
- **B36-E36** [E, decision records; L326-L334] Session decisions cover analysis rationale/open questions, AppState/dependencies/launcher branch, event naming/backpressure, and ownership mapping for each net-new surface.
- **B36-E37** [E, constraints/security; L336-L349] Prohibited: V1 wholesale back-port, orbit edits without request, local heavy-sidecar boot, commit without explicit request, and any unconfirmed npm/Cargo dependency. Remote-only sidecar rule names host `maintainer@192.0.2.10` port `6465` and requires HARDWARE-HEAVY declaration.
- **B36-E38** [E, downstream/status goal; L351-L356] Once shipped, V2 should be fully wired and demoable against remote sidecar, closing the finish-dashboard effort and enabling B-22/B-23/B-26 consumers.
- **B36-C01** [C, count arithmetic; L1-L5, L196-L212, L257-L260] Scope says 13 D rows + 15 net-new, with two baseline commands, but deterministic minimum says “2 baseline + 13 D-rows + 13 net-new = ≥28.” Correct full roster implies up to 30 annotations before accounting for multi-handler rows.
- **B36-C02** [C, row versus handler count; L132-L194, L226-L229] “13 D-commands” means 13 D rows, yet D.13 has three handlers, D.4 three, and D.9/D.10 pairs; raw `#[tauri::command]` count is not equal to D-row count.
- **B36-C03** [C, AppState type; L126-L130, L191-L194] `AppState` lists `Arc<dyn SsbWriter>` but D.12 needs reading/querying SSB rows; writer capability alone does not establish a read API.
- **B36-C04** [C, status; L14-L38, L281-L297] Original is a baseline plan with all gates unchecked, whereas orientation summary says follow-up DONE. This deep ledger must not back-project summary implementation claims into the original.
- **B36-A01** [A, launcher choice; L142-L145] Constant rename versus environment documentation is deliberately unresolved.
- **B36-A02** [A, sidecar probe; L132-L135] `/health` versus minimal `/v1/messages` is unresolved; timeout, offline fallback, authentication, and typed failure shape are absent.
- **B36-A03** [A, concurrency/backpressure; L126-L130, L149-L162, L328-L334] Lock granularity, poison recovery, stream buffering, event ordering, slow-listener backpressure, subprocess cancellation, and duplicate-request behavior are left to Session-3 decisions.
- **B36-A04** [A, offline split; L214-L222] The flag is “to be added,” but selection logic and whether individual commands may fall back to mock after live failure are not defined; stub ban implies no implicit fallback.
- **B36-A05** [A, steering inventory; L57-L65 vs L196-L212] Provenance says recompute plus three A-8 endpoints, while final net-new roster visibly names only recompute and `get_steering_status`; the other steering endpoint identities are not recoverable from this source.

### 8. `B-42_cosmos_artifact_tier.md`

Status/source posture: source is an implementation plan with unchecked acceptance. It extends B-35 and requires B-23’s Layer-1A scheduler and B-31 locks; B-40/B-41 are recommended consumers. The source anticipates schema `+1` from baseline 28 or whatever is current; the orientation summary reports a later shipped implementation with a deferred dispatcher (L1-L22).

- **B42-E01** [E, objective/capability; L1-L12] Add reusable cross-project knowledge for current design trends, app-store rules, UX patterns, IP boundaries, palettes, and SaaS monetization so similar product ideas reuse prior research rather than repeat $35-55/30-90-minute work.
- **B42-E02** [E, dependency/status; L13-L22] B-35 supplies artifact indexing, B-23 supplies Layer-1A growth/decay scheduling, B-31 supplies coexistence locks; B-40 classifies project and B-41 audits against Cosmos. B-24/B-25/B-27/B-28/B-37/B-38 are parallel-safe.
- **B42-E03** [E, research lifecycle; L24-L40] Refresh reuses Layer-1A: scheduled web research -> blake3 artifact -> `SHADOW` -> `ACTIVE` only after FPR threshold. FlashResearch converges to storage, CatRAG/A-RAG gain a second user-global collection, and cadence is domain half-life based (nightly fast-moving, weekly slow in the reference rationale).
- **B42-E04** [E, storage/privacy; L42-L63] Cosmos lives at `~/.elai/cosmos/`, user-scoped and outside every project repo. It uses B-35-style Markdown plus TOML metadata with blake3 hashes and separate `cosmos_global` vector collection.
- **B42-E05** [E, retrieval/integration; L64-L65] Indexing reuses B-32 chunking and B-35 hybrid vector + BM25 reciprocal-rank fusion.
- **B42-E06** [E, population/config; L66-L69] Each artifact declares `refresh_cadence: daily|weekly|monthly|quarterly` and a `research_query` template; Layer-1A ticks launch FlashResearch and convergence writes Cosmos.
- **B42-E07** [E, consumption; L70-L72] Designer, B-40 idea-framer, and B-41 per-level auditor inject `AVAILABLE_COSMOS`, filtered by detected `ProjectClass` and tier relevance.
- **B42-E08** [E, grounding; L73-L76] `GroundingMethod::CosmosCheck` is the eighth method after ArtifactCheck and handles project-class-relevant design, IP, monetization, app-store, and marketing claims.
- **B42-E09** [E, determinism/idempotency; L78-L80] Every refresh compares blake3 content; unchanged content produces no rewrite, saving disk and dashboard/SSB noise.
- **B42-E10** [E, lifecycle/fallback; L81-L84] Artifacts older than 2× cadence become `SHADOW`; consumers are warned and refresh is reprioritized. B-31 locks allow code indexing and Cosmos scheduling to coexist cleanly.
- **B42-E11** [E, directory schema; L98-L116] Layout is `manifest.toml`, `artifacts/<name>.md` + `<name>.toml`, and `index/cosmos_global.bm25` + `cosmos_global.vec`; examples include design trends, iOS guidelines, UX patterns, and copyright/trade dress.
- **B42-E12** [E, manifest schema; L118-L140] Manifest `schema_version=1`; each `[[cosmos.artifacts]]` has `name`, `project_classes`, `refresh_cadence`, `min_source_tier`, `research_query`, `max_size_bytes`, and a `tier_relevance` map. At least 12 entries ship.
- **B42-E13** [E, example config; L122-L138] `design_trends_2026`: ConsumerMobileApp/WebApp/SaaSDashboard, weekly, T1, 32,768 bytes, relevance concepts .9/phases .5/steps .2/babysteps 0. `ios_app_store_guidelines`: ConsumerMobileApp, monthly, T1/apple.com, 65,536 bytes, concepts .7/phases .4/babysteps .6/plan 1.0.
- **B42-E14** [E, worker schedule; L143-L145] `CosmosWorker::tick()` runs hourly by default and skips artifacts not due.
- **B42-E15** [E, worker workflow; L146-L149] For each due artifact, build query from template/current year/minimum source tier, await convergence, write Markdown/blake3/update metadata; unchanged hash logs skip and emits no SSB churn.
- **B42-E16** [E, concurrency; L150-L151] Worker acquires B-31 `IndexLock(LockScope::Index)` in the same namespace as code indexing, guaranteeing a single writer.
- **B42-E17** [E, lazy refresh/fallback; L153-L156] `cosmos_get(name) -> Result<Artifact>` blocks up to 30 seconds to refresh content older than 2× cadence; on timeout it returns stale artifact with WARN metadata.
- **B42-E18** [E, pipeline; L158-L162] Insert `CosmosStage` between `ArtifactStage` and `ARagStage`, with the same vector+BM25 RRF contract as B-35.
- **B42-E19** [E, verification schema; L164-L171] `CosmosCheck` dispatch categories are `design`, `ip_boundary`, `monetization`, `app_store`, `marketing_pattern`, `ux_convention`; output is ArtifactCheck-shaped binary pass/fail with quote and `source_ref`. Failure adds an ungrounded plan claim and blocks B-22 AUTO-advance.
- **B42-E20** [E, briefing selector; L173-L178] `compile_cosmos_briefing(project_class, tier_relevance_threshold)` selects artifacts whose classes include the project and whose current-tier relevance meets threshold; returns `AVAILABLE_COSMOS` capped at 1,500 tokens, summarized with ShortCoder if oversized.
- **B42-E21** [E, briefing defaults; L179-L183] B-40 full threshold .5; B-41 Concepts full .4; B-41 Phases filtered .6; B-41 BabySteps skipped; Designer full .5.
- **B42-E22** [E, state machine; L185-L190] Status is `FRESH` at age ≤ cadence, `STALE` at age ≤2× cadence, and `DECAYED` above 2×; DECAYED drops to SHADOW, warns callers, and moves to front of refresh queue.
- **B42-E23** [E, events/schema; L192-L194] Add `cosmos_refresh_started`, `cosmos_artifact_updated`, `cosmos_artifact_decayed`, and `cosmos_grounding_used`, with schema +1.
- **B42-E24** [E, dashboard; L196-L203] New CosmosDrawer lists status/refreshed time and per-artifact manual refresh; MonitorWindow shows `cosmos: X fresh / Y stale / Z decayed` only during PLANNING/FRAMING; `get_cosmos_status` and `trigger_cosmos_refresh` become live Tauri commands.
- **B42-E25** [E, seed data; L205-L217] Default artifact roster is `design_trends_2026`, `ios_app_store_guidelines`, `android_play_store_guidelines`, `web_app_ux_patterns_2026`, `saas_dashboard_patterns`, `copyright_tradedress_guide`, `asset_licensing_quickref`, `monetization_patterns_consumer_apps`, `monetization_patterns_saas`, `color_palette_trends_2026`, `typography_trends_2026`, `accessibility_wcag_aa_quickref`.
- **B42-E26** [E, exact symbols; L219-L242] Produced symbols: `CosmosWorker`, `cosmos_get`, `CosmosStatus::{FRESH,STALE,DECAYED}`, `CosmosStage`, `GroundingMethod::CosmosCheck`, `compile_cosmos_briefing`, four events, and 12 seed artifacts. Consumers are B-35 indexes, ResearchTree, SourceQualityTable, IndexLock, Layer-1A scheduler; reachable from hourly tick, three briefing builders, `PipelineGrc::verify_claim`, and manual dashboard refresh.
- **B42-E27** [E, provenance/security; L244-L247] Cosmos artifacts must be research-generated; manual edits producing a blake3 mismatch trigger warning and re-research. Refresh may not bypass `min_source_tier`.
- **B42-E28** [E, fallback/constraint; L248-L252] DECAYED content cannot be silently consumed; briefing cannot exceed 1,500 tokens (ShortCoder or skip, never overflow); worker cannot run in AUDITING or EXECUTING.
- **B42-E29** [E, end-to-end workflow; L255-L271] Fresh plant-ID task in FRAMING requests Cosmos; empty store blocks up to 30 seconds each for three mandatory artifacts, emits 3 refresh-started then 3 artifact-updated hashes, builds nonempty briefing, and emits at least one grounding-use event per relevant claim category; resulting artifact hash metadata and `refreshed_at` must be recent (<30 minutes).
- **B42-E30** [E, deterministic gates; L273-L305] Checks require current schema bump, all four event identifiers, worker module, pipeline stage, exactly eight grounding methods, ≥12 manifest entries, blake3 (not SHA-256), V2 drawer, and explicit 1,500 briefing limit.
- **B42-E31** [E, tests/build evidence; L307-L316] At least 25 Cosmos tests must cover tick, idempotency, contention, retrieval, decay, budget, and grounding; workspace cargo check and Svelte check must pass.
- **B42-E32** [E, acceptance/storage-worker; L318-L325] Done requires global layout, scheduled due detection, lock compliance, and zero `artifact_updated` rows on unchanged rerun.
- **B42-E33** [E, acceptance/pipeline-grounding; L325-L333] Done requires exact stage position, six-category Cosmos dispatch, briefing consumption/cap, and DECAYED warning/reprioritization.
- **B42-E34** [E, acceptance/evidence; L334-L340] Populate ≥12 artifacts within 30 minutes, render a fully live drawer, and run a 30-task consumer-app ablation showing ≥50% reduction in ungrounded-claim BLOCKs; ≥25 tests and clean checks required.
- **B42-E35** [E, tests; L342-L356] Named tests cover cadence/idempotency, indexer contention, hybrid-scoring fairness, grounding failure rows, 1,500-token/ShortCoder fallback, and DECAYED warning/no-silent-stale behavior.
- **B42-E36** [E, file lifecycle; L358-L378] Create worker, seed manifest, six named test files, briefing module, drawer, and decision doc; modify knowledge pipeline/lib, reasoning verifier, SSB, TaskRunner scheduler, V2 commands/stubs, and Monitor badge.
- **B42-E37** [E, storage decision; L380-L387] User-global path avoids redundant project research; `/etc/elai/cosmos` is rejected because Windows lacks it and Linux would require sudo.
- **B42-E38** [E, source-quality/security; L388-L392] blake3 follows anti-stale rule. `min_source_tier` defaults T1 because consumers treat Cosmos as authoritative; T2/T3 belong in active research where contradictions can surface.
- **B42-E39** [E, budget/failure decision; L393-L397] 1,500 tokens is chosen to leave ≥18K headroom in a 25K local ceiling. DECAYED content is retained because WARNed stale data is preferable to no data when offline.
- **B42-E40** [E, non-goal/boundary; L398-L402] Deterministic design hard rules remain in `rules.db`/B-41, not Cosmos; Cosmos is probabilistic reference material and must not blur the Phase-3F enforcement boundary.
- **B42-E41** [E, downstream; L404-L408] A future “B-43 Visual Auditor,” if approved, would consume design/color/typography Cosmos artifacts as VLM comparison ground truth.
- **B42-C01** [C, stale terminology; L81-L82, L153-L156, L187-L190] `cosmos_get` calls age >2× “stale,” while the formal enum calls that `DECAYED`; formal `STALE` is age between one and two cadences. Lazy-refresh trigger is therefore DECAYED-only as written.
- **B42-C02** [C, lifecycle model; L26-L29 vs L187-L190] Spec reference says research writes SHADOW then becomes ACTIVE after FPR; task state model says FRESH/STALE/DECAYED, with only DECAYED mapping to SHADOW. How newly written SHADOW/FPR promotion composes with freshness is not specified.
- **B42-C03** [C, schema baseline/status; L19-L21 vs orientation summary] Source expects +1 from 28-or-current; orientation reports shipped schema 44. The original does not establish the later numeric version.
- **B42-C04** [C, grounding identifier; L73-L76, L164-L170, L221-L228 vs orientation implementation] Source consistently specifies `GroundingMethod::CosmosCheck`; later summary calls shipped `VerificationMethod::CosmosCheck`. Concrete code enum must resolve identifier drift.
- **B42-C05** [C, B-43 identity; L404-L408] This source reserves B-43 for Visual Auditor, while the current B-plan filename catalog uses B-43 for one-line install. Identifier reuse/drift is explicit and unresolved within the chunk.
- **B42-C06** [C, source tier default; L91-L95 vs L118-L140, L244-L247, L388-L392] Mandatory-read note says Cosmos sources must be T1/T2 “by default,” while manifest examples/decision rationale set `min_source_tier=T1` and call T1 authoritative. Treat T1 as the declared seed default; whether T2 is allowed per artifact needs an explicit policy.
- **B42-A01** [A, scheduler dispatch; L143-L151, L238-L242] Source defines `tick()` and reachability but not the long-running spawn/dispatcher, retry/backoff, crash recovery, or missed-tick semantics; orientation says `spawn_loop` remains deferred.
- **B42-A02** [A, deterministic time; L146-L149, L187-L190] Query interpolation uses current year and status uses wall-clock age, but injected clock/timezone/atomic metadata-write mechanics are not specified.
- **B42-A03** [A, lock fairness; L150-L151] Same namespace guarantees a single writer but not priority/fairness; how hourly refresh avoids starving code indexing (or vice versa) is absent.
- **B42-A04** [A, privacy/compliance; L57-L63, L118-L140] User-global research retention has no stated encryption, filesystem permissions, deletion/export policy, audit retention, multi-user separation, or copyright/source-attribution lifecycle.
- **B42-A05** [A, manual refresh; L196-L203] Permission, deduplication with scheduled refresh, cancellation, progress/error UI, and whether manual refresh is disabled in AUDITING/EXECUTING are not specified.

## Cross-file interactions

### Explicit interactions

- **X-E01** [E] B-5 defines producers/surfaces but explicitly relies on B-8 for production firing; B-36 later names live `get_grc_budgets` as a V2-only handler “post-B-5” (B-5 L1-L9, L173-L176; B-36 L196-L212).
- **X-E02** [E] B-14 and B-36 D.7 explicitly share Tauri emit streaming. B-14 requires consistent `turn`, `streaming_started`, `response`, `token`, `stop`; B-36’s one-shot command emits the first three (B-14 L42-L53; B-36 L158-L162, L226-L234).
- **X-E03** [E] B-14 permits its InterventionCard to reuse B-5’s CsqBlockCard/intervention work; both surface safety-relevant user decisions in Command/Reasoning UI (B-14 L167-L174; B-5 L99-L103, L164-L170).
- **X-E04** [E] B-25 mounts EmpireBridgePanel in SettingsWindow between runtime and cloud; B-27 puts CloudPanel/BYOE/spend configuration in SettingsWindow’s cloud area (B-25 L117-L122, L196-L202; B-27 L232-L245).
- **X-E05** [E] B-27’s cloud mode is explicitly another B-10 ContextBudget profile and is gated on B-10 activation (B-27 L1-L10, L108-L117).
- **X-E06** [E] B-31.5 reuses B-29.5’s exact fixtures, can run in parallel, and explicitly turns B-25’s Empire-corpus claim into a testable scale assertion (B-31.5 L10-L13, L475-L499).
- **X-E07** [E] B-33 backend availability depends on B-36 Session-2 AppState/engine dependencies, while its data depends on B-7 DecompositionTree and B-29 SymbolGraphCache; Refresh relies on B-31 watcher behavior (B-33 L9-L14, L298-L305, L335-L344).
- **X-E08** [E] B-36’s real V2 handlers explicitly consume B-4 `get_search_mode`, B-5 `get_grc_budgets`, B-6 `get_evidence_gates`, and B-10 `get_context_budget`, making B-36 their dashboard adapter rather than their producer (B-36 L196-L212).
- **X-E09** [E] B-42 is structurally B-35 storage/retrieval plus B-23 scheduling plus B-31 locking; B-40 project classification and B-41 audit modes select and consume its briefings (B-42 L13-L22, L57-L76, L219-L242).
- **X-E10** [E] B-42 intentionally shares `LockScope::Index` with code indexing. Its dedicated contention test is a Cosmos-worker + code-indexer test, distinct from B-31.5’s two-code-indexer contention benchmark (B-42 L143-L156, L342-L349; B-31.5 L175-L223).
- **X-E11** [E] B-33’s GraphWindow can receive `IndexingComplete`/`SymbolGraphUpdated` updates, while B-31.5 establishes that the indexing/checkpoint lifecycle is stable under crashes and long runs; they are complementary UI versus durability evidence, not duplicate tests (B-33 L298-L305; B-31.5 L1-L7, L309-L323).
- **X-E12** [E] B-42’s global `~/.elai/cosmos/` path is intentionally outside project repositories, unlike project-local `.elai_cc` data used by indexing and other B plans; the source explicitly rejects project-local and `/etc` placement (B-42 L57-L63, L380-L387).

### Inferred interactions (not independent promises)

- **X-I01** [I] A unified V2 chat path likely uses B-36 AppState/subprocess/event plumbing to host B-14’s `stream_chat` and cancellation handle. Both halves are explicit, but neither source states the final shared type/module boundary.
- **X-I02** [I] Exact live frontend naming should converge on `get_grc_budgets`: B-5 says stub names remain 1:1 and its contract produces that identifier; B-36 consumes exactly it. The longer B-5 prose name is likely a descriptive predecessor, but this remains a reconciliation inference.
- **X-I03** [I] SettingsWindow’s likely section sequence is runtime -> Empire -> cloud, combining B-25’s explicit insertion point with B-27’s CloudPanel ownership. Responsive/layout ownership is not defined by these files.
- **X-I04** [I] GRC can eventually present both Empire and Cosmos evidence: B-25 adds `EmpireEventCheck`, B-42 adds `CosmosCheck`, and B-5 exposes reasoning/grounding outcomes. No chunk source promises a dedicated per-method breakdown in ReasoningDrawer.
- **X-I05** [I] Schema updates must be applied against the repository’s then-current registry rather than literal plan baselines, because B-25 names 101->102 while B-42 says “current +1.” Atomic variant lockstep is explicit only in B-25; applying equivalent lock discipline to Cosmos is prudent but inferred.
- **X-I06** [I] B-31.5’s durability benchmark does not prove B-42’s same-lock Cosmos/code coexistence unless the B-42 contention test is also run. The two tests cover different contenders despite sharing the lock primitive.
- **X-I07** [I] If B-33 chooses orbit MAP (Option B), the work becomes an explicit exception to B-36’s “do not touch orbit unless asked” constraint and therefore requires the user choice documented by B-33.
- **X-I08** [I] B-14’s “no thinking messages” and B-36’s live `web_research_step` imply progress should flow through Activity/orbit while Command displays milestones/tokens/interventions; this composed routing is consistent but not specified as one end-to-end state machine.
- **X-I09** [I] B-27’s pre-call redaction and spend gate should wrap any cloud-backed execution reached from dashboard commands, but B-36’s D.7 subprocess environment does not explicitly call `CloudModeRouter` or `RedactingBabyStepExecutor`.
- **X-I10** [I] B-42’s no-rewrite hash behavior and B-31/B-31.5 checkpoint identity establish complementary idempotency layers: artifact-level content idempotency versus batch-level index resume identity.

### Cross-file ambiguities/conflicts

- **X-A01** [A] B-27 air-gap disables all external network, but B-36 assumes a remote sidecar, B-42 schedules web research, and B-25 supports TCP Empire. The precedence/gating contract for those three integrations under `air_gapped=true` is unstated.
- **X-A02** [A] B-14 Stop needs an AppState cancellation handle; B-36 AppState lists only SSB writer, ResearchCache, and ProjectPaths. The cancellation registry’s owner, lifetime, and concurrency semantics are missing.
- **X-A03** [A] B-27 privacy redaction covers cloud model context, but whether B-42 FlashResearch queries/results and B-25 Empire event payloads pass the same redactor/audit trail is not stated.
- **X-C01** [C] B-33 Option B requires orbit edits, while B-36 forbids orbit changes absent explicit request. This is resolvable only by the B-33 user decision, not by assuming the lighter recommendation wins.
- **X-C02** [C] B-25’s literal schema 101->102, B-42’s baseline-28/current+1 language, and orientation’s later schema values describe different repository snapshots. No numeric migration order can be synthesized safely from this chunk.
- **X-C03** [C] B-5’s six command contract, B-36’s single `get_grc_budgets` consumer, and B-5 acceptance “five commands” leave the rubber-stamp rejection reader without a confirmed V2 handler inventory.

## Orientation-summary additions and corrections

These records compare the deep sources with `02-B-PLAN-SUMMARY.discovery.md`; they are comparison deltas, not new feature atoms.

### B-5 deltas (orientation L67-L75)

- **SD-B5-01** Addition: payload fields for all four core reasoning events, exact eight roles, dual call/character budgets, `<20%` red threshold, pie colors/click behavior, last-five feasibility list, and exact intervention actions/replan behavior are only in source L61-L103.
- **SD-B5-02** Correction: orientation lists five prose commands, but source contract L105-L118 uses `get_grc_budgets` (not `_per_agent`) and adds `get_auditor_rubber_stamp_rejections`, for six produced command symbols.
- **SD-B5-03** Correction: orientation lists `feasibility_check`; source’s concrete enum is `FeasibilityReport` (L66-L70, L107-L112). Preserve both until serialization mapping is verified.
- **SD-B5-04** Addition: source exposes existing stubs `get_csq_pending`/`get_gg_signals`, explicit synthetic constants, deterministic SQLite evidence, and unresolved Override authorization (L21-L33, L35-L47, L136-L162).

### B-14 deltas (orientation L167-L176)

- **SD-B14-01** Addition: exact channel set `turn/streaming_started/response/token/stop`, AppState cancellation handle, palette group membership, outlined-button roles, placeholder strings, and prohibition on “thinking...” are source L15-L25, L42-L64, L104-L121.
- **SD-B14-02** Correction: orientation’s six card variants are accurate for V2, while acceptance’s “five response types” excludes `user`; they are different counts, not a missing sixth response implementation (source L15-L16, L107-L109, L157-L163).
- **SD-B14-03** Addition/ambiguity: standalone InterventionCard versus CommandWindow variant and stream-return versus emitter remain choices (source L34-L38, L90-L92); orientation presents the overall V2 surface without those branches.
- **SD-B14-04** Correction: source’s deterministic/files sections retain obsolete V1 paths despite the V2 rebaseline (L27-L64, L142-L175); checks should target V2.

### B-25 deltas (orientation L316-L326)

- **SD-B25-01** Status correction: orientation labels B-25 deferred/conditional, while source L14-L26 records shipped host scaffolding plus an explicit `EMPIRE-PENDING` residual list. Neither supports calling live Phase 4J complete.
- **SD-B25-02** Addition: exact events, schema `101->102`, lock-test total `298->303`, 5-second config default, ToolFilter’s adversarial denial of six internal capabilities, 24-name stdlib heuristic, and partial panel fields are source L67-L122, L174-L210.
- **SD-B25-03** Correction: orientation’s feature prose does not distinguish shipped trait/types/config from pending socket bodies, heartbeat loop, filters, MCP dispatcher, AST repair, PipelineSpec validation, and commit (source L67-L120).
- **SD-B25-04** Addition/conflict: the source simultaneously describes newline-JSON buses and a pending FSB binary-header parser (L33-L36, L21-L23); orientation omits this protocol ambiguity.

### B-27 deltas (orientation L342-L352)

- **SD-B27-01** Addition: exact config defaults including API key env and dollar caps, all eight slot values, five LOC profiles, max-three L1 retries, upload booleans, placeholders, audit contents, and seven SSB symbols appear in source L76-L188.
- **SD-B27-02** Correction: source stub ban says daily/monthly caps, but defined schema is per-task/per-day only (L78-L101, L198-L201); “monthly” is unsupported drift.
- **SD-B27-03** Addition: air-gap explicitly disables nightly dependency audit and Oracle web search as well as cloud escalation; provider routing has a <100ms probe, fallback, and LOCAL notification (L53-L62, L93-L99).
- **SD-B27-04** Addition/ambiguity: autonomous answer to the cost prompt, concurrent spend accounting/day boundary, and exact HYBRID behavior at 100% remain unspecified.

### B-31.5 deltas (orientation L421-L430)

- **SD-B315-01** Correction: orientation says the three contract names are unstated; source names `scale_resume_identity_contract`, `scale_lock_contention_contract`, and `scale_long_tail_memory_contract` at L309-L344.
- **SD-B315-02** Correction: orientation says Tier-D shutdown within 60 seconds; source defines a 60-second deadline plus 5-second test slack, so observed-process acceptance is `<65s` (L225-L265, L375-L378, L405-L409).
- **SD-B315-03** Addition: pinned fixture versions, every metrics field/artifact, exact SSB counts, per-batch p99 ≤2× baseline, fail-on-missing metrics, and pre-release/nightly decisions are source L55-L173, L295-L473.
- **SD-B315-04** Correction: identity is byte-exact only for non-timestamp columns; `last_indexed_at` is excluded (L158-L160). A canonical comparator is still unspecified.
- **SD-B315-05** Addition/conflict: source accidentally says “all four contract tests” although it names three, and ordinary release may accept A+B+C while full/production-ready acceptance requires D+E (L345-L355, L390-L473).

### B-33 deltas (orientation L443-L454)

- **SD-B33-01** Addition: complete `VizData`/project/file/edge/symbol schemas, mode-loading semantics, impact response, empty-graph fallback, cross-window links, event subscriptions, and exact test suite are source L120-L239, L296-L436.
- **SD-B33-02** Correction: exact packages are `cytoscape`, `cytoscape-dagre`, and `cytoscape-fcose`, all user-confirmed; generic “dagre/fcose” are layout/plugin shorthand (L252-L254, L394-L397).
- **SD-B33-03** Addition/conflict: placement remains A/B/C and B/C is recommended, but the body/acceptance assumes A. Orientation retains the choice but not the degree to which tasks are conditional (L45-L56, L241-L319, L399-L418).
- **SD-B33-04** Addition: exact decomposition shapes/status colors, 2-second live-refetch gate, >1KB PNG gate, 500-node decomposition cap, right-click default, and >50-node label decision are source L287-L305, L360-L397, L463-L479.

### B-36 deltas (orientation L484-L496)

- **SD-B36-01** Status boundary: orientation reports post-plan completion and a 637-LOC implementation ledger; the chunk original only specifies the four-session plan and unchecked gates. Those later facts require their own cited evidence and are not promoted to source-explicit atoms here.
- **SD-B36-02** Correction: source identifier is `get_tool_visibility()` (L177-L178), not orientation shorthand `tool_visibility`; D.12 is `get_grounding_cache_stats()` (L191-L194).
- **SD-B36-03** Addition: exact proposed dependencies, AppState shape, offline flag, typed `NotImplementedYet` rule, screenshot evidence, one-contract-per-command rule, no-commit rule, and remote-heavy constraint appear at L113-L140, L214-L253, L281-L349.
- **SD-B36-04** Addition/conflict: 13 D rows are not 13 handlers; deterministic arithmetic counts only 13 of the 15 net-new items and expects ≥28 (L132-L212, L257-L260). Orientation does not flag the count mismatch.
- **SD-B36-05** Addition: source records five GG fields and net-new provenance including cache-control, response format, and elapsed time (L57-L65), not just handler names.

### B-42 deltas (orientation L580-L597)

- **SD-B42-01** Status boundary: orientation says shipped with `spawn_loop` deferred and schema 44; source is a plan expecting “current +1,” with unchecked acceptance. The later status cannot be proven from this original alone (source L19-L21, L318-L340).
- **SD-B42-02** Identifier correction: source specifies `GroundingMethod::CosmosCheck`; orientation names later `VerificationMethod::CosmosCheck`. Preserve drift pending code verification (source L73-L76, L164-L170, L219-L228).
- **SD-B42-03** Addition: SHADOW→ACTIVE/FPR lifecycle, exact manifest examples/sizes/relevance, binary quote/source result, manual-edit warning, no AUDITING/EXECUTING execution, ≥25-test gate, and ≥50% ablation target are source L24-L40, L118-L170, L244-L340.
- **SD-B42-04** Correction: source estimates worker `~500 LOC` plus separate ~250-LOC briefing (L358-L369); orientation’s “roughly 700-LOC worker plus briefing” is a later implementation characterization, not the plan estimate.
- **SD-B42-05** Addition/conflict: source calls future Visual Auditor B-43, conflicting with current B-43 one-line-install naming; orientation correctly flags this drift. Source also distinguishes formal STALE from DECAYED even though `cosmos_get` loosely calls >2× stale (L153-L190, L404-L408).

## Classification and atomic-count audit

| Source | Explicit E | Inferred I | Ambiguous A | Conflicting C | Source atoms |
|---|---:|---:|---:|---:|---:|
| B-5 | 22 | 0 | 1 | 3 | 26 |
| B-14 | 18 | 0 | 3 | 4 | 25 |
| B-25 | 23 | 0 | 4 | 3 | 30 |
| B-27 | 28 | 0 | 5 | 3 | 36 |
| B-31.5 | 40 | 0 | 4 | 6 | 50 |
| B-33 | 41 | 0 | 4 | 6 | 51 |
| B-36 | 38 | 0 | 5 | 4 | 47 |
| B-42 | 41 | 0 | 5 | 6 | 52 |
| **Canonical-source subtotal** | **251** | **0** | **31** | **35** | **317** |
| Cross-file interactions | 12 | 10 | 3 | 3 | 28 |
| **Grand atomic feature/interaction count** | **263** | **10** | **34** | **38** | **345** |

The 35 `SD-*` comparison records are intentionally excluded from the 345 feature/interaction count: they restate source atoms only to identify summary additions/corrections. Including those non-feature comparison records yields 380 labeled records.

Ambiguity IDs are `B5-A01`; `B14-A01..A03`; `B25-A01..A04`; `B27-A01..A05`; `B315-A01..A04`; `B33-A01..A04`; `B36-A01..A05`; `B42-A01..A05`; and `X-A01..A03`. Conflict IDs are `B5-C01..C03`; `B14-C01..C04`; `B25-C01..C03`; `B27-C01..C03`; `B315-C01..C06`; `B33-C01..C06`; `B36-C01..C04`; `B42-C01..C06`; and `X-C01..C03`. All inferred records are isolated as `X-I01..I10`; no inference is presented as source-explicit.

## Zero-omission / zero-duplicate coverage audit

| File | Complete source bands accounted for | Atom IDs | Count |
|---|---|---|---:|
| B-5 | L1-10 scope/status; L11-34 V2/stubs; L35-58 gap/provenance; L59-104 tasks; L105-135 contract/effect; L136-162 checks/acceptance; L164-176 files/next | `B5-*` | 26 |
| B-14 | L1-8 scope/status; L9-65 V2 delta; L66-87 runtime/provenance; L88-122 tasks; L123-141 contract/effect; L142-166 checks/acceptance; L167-176 files; L177-191 part gate/next | `B14-*` | 25 |
| B-25 | L1-27 scope/current status; L28-57 integration model; L58-64 provenance; L65-128 tasks/status; L129-151 contract/effect; L152-173 checks/acceptance; L174-211 file/deferred ledger; L212-214 next | `B25-*` | 30 |
| B-27 | L1-11 scope; L12-63 modes/privacy/escalation; L64-73 provenance; L74-158 D1-D4 tasks; L159-207 contract/effect; L208-231 checks/acceptance; L232-250 files/next | `B27-*` | 36 |
| B-31.5 | L1-14 scope; L15-52 rationale/provenance; L53-105 fixtures/Tier A; L106-174 Tier B; L175-224 Tier C; L225-266 Tier D; L267-294 Tier E; L295-356 aggregation/contracts; L357-416 checks/acceptance; L417-448 artifacts/files; L449-474 decisions; L475-499 effects/next | `B315-*` | 50 |
| B-33 | L1-15 scope; L16-57 rebaseline/choice; L58-117 gap/provenance; L118-240 schema/backend; L241-295 UI/design; L296-359 wiring/contract; L360-420 checks/acceptance; L421-462 tests/files; L463-480 decisions; L481-501 effects/next | `B33-*` | 51 |
| B-36 | L1-13 scope; L14-39 baseline; L40-68 authority/net-new provenance; L69-95 mandatory-read provenance; L96-146 Sessions 1-2; L147-195 Sessions 3-4 D rows; L196-254 net-new/contract/effect; L255-298 checks/acceptance; L299-335 tests/files/decisions; L336-356 constraints/next | `B36-*` | 47 |
| B-42 | L1-23 scope/dependencies; L24-41 spec lifecycle; L42-95 context/provenance; L96-157 storage/worker; L158-184 pipeline/briefing; L185-218 decay/UI/seeds; L219-272 contract/effect; L273-341 checks/acceptance; L342-379 tests/files; L380-408 decisions/next | `B42-*` | 52 |

Audit conclusions:

- Exactly **8 canonical originals / 2,595 logical lines / 116,448 bytes** were processed; recomputed line, byte, and SHA-256 values match the manifest exactly.
- Exactly **317 canonical-source atoms** were assigned unique IDs. Closed rosters retain every stated member; no atom ID is duplicated and each source assertion has one primary ledger home.
- Exactly **28 cross-file interaction atoms** were separated from source atoms so synthesis cannot masquerade as a canonical promise.
- The mandatory `Reads` lists were read as part of each assigned file, but their linked originals were **not opened**; they are dependency/provenance, not hidden extra chunk input.
- Headings, estimates, code examples, file ledgers, decision requirements, `Next` sections, and acceptance lists were either captured in the cited atom bands or treated as non-feature prose only when they merely name a source already represented. There are **0 unaccounted source line bands**.
- Summary deltas point back to the same source atoms and are excluded from duplicate/feature counts. Manifest rows are inventory evidence, also excluded. Result: **0 missing files, 0 extra originals, 0 duplicate originals, 0 duplicate atom IDs, and 0 silently resolved ambiguities/conflicts**.
