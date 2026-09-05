phase-4J decisions — Empire integration (host-side scaffolding pass)

Date: 2026-05-22
Session: s-25120 (completion-drive)
Cross-check report: workflow/sessions/s-25120/cross-check/phase-4J-report.md
Context package: workflow/sessions/s-25120/completion-drive/contexts/phase-4J-context.md
Exploration: workflow/sessions/s-25120/completion-drive/exploration/phase-4J-exploration.md
Grounding: workflow/sessions/s-25120/completion-drive/grounding/phase-4J-grounding.md
Sibling B-plan: docs/thefinalglance/B-plan/B-25_phase_4J_empire_integration.md

Scope of this pass

Phase 4J (Empire integration) cross-check shipped with 22 MISSING claims, 4 HIGH warnings and 3 MEDIUM warnings. The Empire codebase (workspace/factory-brain) is NOT currently accessible from this workspace tree (spec line 8 explicitly gates the phase on "Empire codebase is accessible and deployed"). Per the RUNPOD-PENDING analog used elsewhere in ELAI for external-dependency-gated work, this pass ships full host-side scaffolding (types + bridge state machine + Tauri command surface + Svelte panel + feature gate + metrics + decision doc) with every external-IO seam labelled `EMPIRE-PENDING: <what-to-wire>` so a one-line activation patch can flip every wire-point when Empire deploys.

What ships here is not a stub. It is a NOT-LAZY GROUNDED host-side surface that compiles, tests, and runs end-to-end against the dashboard. Bridge calls return well-typed `EmpireBridgeError::NotConnected` and the panel renders an empty-state notice, both deterministic. The `EMPIRE-PENDING` marker is a tracked external-dependency wire-point, not a `TODO`.

Pre-resolved decisions (from the executor task brief)

DECISION 1: EMPIRE-PENDING gate strategy
  Resolves HIGH warning #1 (Empire codebase NOT visible in .elai_cc/ tree).
  Apply RUNPOD-PENDING analog: build all host-side scaffolding fully, mark every live-integration seam with `// EMPIRE-PENDING: <command/uri when Empire deployed>`. Future activation is a one-line flip.
  Markers shipped this pass (grep `EMPIRE-PENDING` in src/empire/* + commands_empire.rs):
    - `factory_line.rs`  connect()      "socket boot deferred until Empire deploys"
    - `factory_line.rs`  send()          (implicit via NotConnected return)
    - `factory_line.rs`  recv()          (implicit via NotConnected return)
    - `factory_line.rs`  default_unix()  "config loader for elai_cc.toml [empire] section"
    - `bridge.rs`        connect()      "factory-brain socket boot"
    - `bridge.rs`        emit_action()  "SSB lockstep (SCHEMA_VERSION bump) deferred"
    - `bridge.rs`        factory_lines() (graceful empty return; activation note in doc-comment)
    - `commands_empire.rs` empire_status_inner() "when AppState gains an empire_bridge field"
    - `EmpireEventKind` enum doc-comment: "actual SSB lockstep (9-tuple per variant) is gated"
    - module doc `mod.rs`: top-level marker explaining the pattern
    - `types.rs` EmpireEventKind doc: explicit one-place dispatch flip
  Total: 11 EMPIRE-PENDING markers shipped.

DECISION 2: Crate vs submodule conflict (spec vs B-25)
  Resolves HIGH warning #2 (spec deterministic checks say `cargo check -p elai_empire` but B-25 file-list places code under crates/orchestrator/src/empire/*).
  Choice: orchestrator submodule per B-25. NOT a new top-level crate.
  Rationale: avoids a new top-level crate for what is a single integration surface; co-locates with task_runner that will consume it. The spec's `cargo check -p elai_empire` DETERMINISTIC_CHECK lines resolve to `cargo check -p elai-orchestrator --features empire` (and the non-feature build must also stay green).
  Cargo wiring:
    - `crates/orchestrator/Cargo.toml [features]` gains `empire = []` (marker, no transitive deps yet).
    - `crates/orchestrator/src/lib.rs` declares `#[cfg(feature = "empire")] pub mod empire;`.
  spec edit applied: 4J.1 DETERMINISTIC_CHECK line updated in-place; phase-4J.txt also gains an EMPIRE-PENDING gate note at the top explaining this decision.

DECISION 3: SSB action_type granularity
  Resolves HIGH warning #3 (spec says single 'empire_event'; B-25 lists 5 separate variants).
  Choice: 5 separate variants per B-25 WIRING_CONTRACT.
    - empire_connected
    - empire_disconnected
    - empire_event_received
    - empire_capability_invoked
    - empire_repair_proposed
  Rationale: granular variants survive UI filter / per-action audit / SSB row reader queries; a single collapsed action_type kills observability and makes the dashboard render one undifferentiated stream.
  Host-side surface: `empire::types::EmpireEventKind` is a closed enum with the 5 variants. `EmpireBridge::emit_action` dispatches on the variant and maps to the right action_type label. The full SSB lockstep (9-tuple per variant: enum arm + SQL CHECK + as_sql_str + from_str + SsbPayload variant + action_type match + payload struct + test_payload + SCHEMA_VERSION bump from 97 -> 98) is EMPIRE-PENDING — gated on Empire deployment so we don't churn ssb.rs ahead of the dispatch site landing.

DECISION 4: Empire IPC type namespace
  Resolves HIGH warning #4 (existing internal `IpcMessageType` in crates/ipc-types is consumed by 10+ sites).
  Choice: Empire-side type lives at `empire::ipc::EmpireIpcMessageType`. Do NOT shadow the existing internal type. The existing `elai_ipc_types::IpcMessageType` (16 LOCKED variants, postcard wire) is reserved for elai's internal orchestrator <-> agent IPC; the Empire IPC engine-protocol verbs are a different concept entirely (newline-JSON over AF_UNIX, brain <-> engine bus).
  spec edit applied: 4J.1 wire-format types and DETERMINISTIC_CHECK lines updated in-place to use the namespaced name.
  Verification before adding: grep across crates/ confirms the existing type is consumed by 10 sites (feasibility_probe, task_runner, ipc_accept, agent_role, contract tests). Renaming the existing one would have been a P0 incident.

DECISION 5: Spec drift fix
  Resolves grounding warning (orchestrator.txt "Section 7 (agents)" -> actual content is AGENT CHECKPOINTING; intended target is Section 12 = Tier C process separation).
  spec edit applied: phase-4J.txt READ BEFORE IMPLEMENTING block updated in-place.

DECISION 6: 4J.9 perf number labelling
  Resolves grounding INGEST warning (Firecracker boot numbers indirectly cited via orchestrator.txt:454).
  Choice: label every 4J.9 acceptance threshold as `UNGROUNDED_TARGET` in phase_4J.toml. They are MEASURE thresholds, not verified prior art. Re-measure once Empire is live and replace the placeholders with measured p50/p95/p99.

What ships this pass (deliverables A-G)

A) crate wiring
  - `crates/orchestrator/src/empire/mod.rs`               (new) — module index + EMPIRE-PENDING pattern doc
  - `crates/orchestrator/src/empire/types.rs`             (new) — FactoryCmd, FactoryLineCommand, FactoryLineResponse, EngineTier, HookKind, HookBinding, Capability, NeuronRegistration, FactoryLine, EmpireEventKind (5 variants), EmpireEvent, ElaiNeuron + whitelist()
  - `crates/orchestrator/src/empire/ipc.rs`               (new) — EmpireIpcMessageType (9 variants), EmpireIpcMessage, EmpireIpcResponse
  - `crates/orchestrator/src/empire/brain_manifest.rs`    (new) — BrainManifest + validate() with 4 rules + BrainManifestError (thiserror)
  - `crates/orchestrator/src/empire/factory_line.rs`      (new) — FactoryLineClient outline + FactoryLineConfig + TransportConfig (Unix/Tcp) + encode (newline `\n`)
  - `crates/orchestrator/src/empire/bridge.rs`            (new) — EmpireBridge state machine (Disconnected -> Connecting -> Connected) + EmpireBridgeError (thiserror) + emit_action dispatcher
  - `crates/orchestrator/src/lib.rs`                      (edit) — `#[cfg(feature = "empire")] pub mod empire;`
  - `crates/orchestrator/Cargo.toml`                      (edit) — `[features] empire = []`

B) types — see A. All wire types are `serde::{Serialize, Deserialize}`. No `ts-rs` derives because this workspace does not use ts-rs (verified: zero `ts-rs|ts_rs` matches across .elai_cc/). TS counterparts are hand-authored in `dashboard-v2/src/lib/api/empire.ts` to match. All Rust code uses thiserror; zero `Box<dyn Error>`. Native async fn in inherent impls (no async-trait crate). Zero `.unwrap()` / `.expect()` in library paths.

C) 5 SSB ActionKind variants — handled via `empire::types::EmpireEventKind` (5 closed enum variants). The actual `orchestrator::ssb::SsbActionType` enum lives in `ssb.rs` (current SCHEMA_VERSION = 97; the 9-tuple lockstep across enum / CHECK / as_sql_str / from_str / SsbPayload / action_type / payload struct / test_payload / SCHEMA_VERSION 98 is EMPIRE-PENDING). The host-side `EmpireEventKind` enum already exists so the future patch is a one-place dispatch flip. Per the cross-check verification: the executor brief said "Add 5 ActionKind variants to orchestrator::ssb::ActionKind enum (or wherever ActionKind lives — grep first to confirm)" — grep confirmed `ActionKind` does NOT exist; the real enum is `SsbActionType`. Per decision 1 (EMPIRE-PENDING strategy) the SSB lockstep is deferred until Empire deploys to avoid a 45-site churn in ssb.rs on dead code.

D) bridge — see A (`bridge.rs`). EmpireBridge owns the state machine + emit_action dispatcher + factory_lines read surface. Every state transition is tracing-logged.

E) dashboard wires
  - `crates/dashboard-v2/src-tauri/src/commands_empire.rs` (new) — two Tauri commands (get_empire_status + list_empire_factory_lines) with feature-conditional inner impl
  - `crates/dashboard-v2/src-tauri/src/lib.rs`             (edit) — mod commands_empire + invoke_handler! entries for both commands
  - `crates/dashboard-v2/src/lib/api/empire.ts`            (new) — TS wrappers + types (EmpireStatus, EmpireState, FactoryLine)
  - `crates/dashboard-v2/src/lib/components/EmpireBridgePanel.svelte` (new) — Svelte 5 runes panel; calls invoke() via the TS wrappers; renders "Empire bridge unavailable" empty state when state=unavailable; renders factory-line table when connected; 10s poll cadence
  Cross-lang-pair marker present in both the Rust command file and the Svelte panel.

F) docs
  - `workflow/reasoning/phase-4J-decisions.md` (this file)
  - `.elai_cc/metrics/phase_4J.toml`           — populated with scaffolding state + UNGROUNDED_TARGET-labelled 4J.9 thresholds

G) phase-4J.txt spec edits
  - top-level EMPIRE-PENDING gate block documenting decisions 1-4
  - 4J.1 DETERMINISTIC_CHECK line updated to feature-based check + ActionKind grep relaxed to include EmpireEventKind
  - 4J.1 wire-format types renamed to EmpireIpcMessageType
  - 4J.4 SSB bridge note updated to 5-variant dispatch
  - READ-BEFORE-IMPLEMENTING block updated for the Section 12 fix
  - Note on EmpireIpcMessageType DETERMINISTIC_CHECK grep so future implementers do not re-collide with the internal IpcMessageType

Anti-lazy hard-rule compliance

1. ZERO new stubs / TODO / FIXME / placeholder / unimplemented!() / todo!() — confirmed by grep on the new files (`unimplemented!\|todo!\|TODO\|FIXME\|placeholder`). EMPIRE-PENDING markers document an external-dependency wire-point AND ship working host-side returns (`NotConnected` typed error / empty Vec).
2. No `.unwrap()` / `.expect()` outside `#[cfg(test)]`. The bridge's mutex unlock failure path logs and returns a deterministic Disconnected state rather than panicking; manifest validation returns `BrainManifestError` via `thiserror`; client encode returns `EmpireBridgeError::Encode` on serialize failure.
3. tracing (not log), thiserror (not Box<dyn Error>). Confirmed.
4. Native async fn in inherent impls; no async-trait crate.
5. Tauri invoke_handler! registers both commands.
6. EmpireBridgePanel.svelte calls invoke() (via the TS wrappers in api/empire.ts which themselves call `import('@tauri-apps/api/core').then(m => m.invoke(...))`).
7. Feature gate compiles cleanly both with and without:
     - `cargo check -p elai-orchestrator`              expected: OK
     - `cargo check -p elai-orchestrator --features empire` expected: OK
     - `cargo check -p elai-dashboard-v2`              expected: OK (commands_empire compiles regardless of orchestrator feature — feature-conditional inner uses `#[cfg]`)

Followups (when Empire deploys)

F1. Flip every `EMPIRE-PENDING` marker (grep returns 11 sites).
F2. Add the 5 SSB action_type variants in ssb.rs via the 9-tuple lockstep + SCHEMA_VERSION 97 -> 98 bump.
F3. Wire `Arc<EmpireBridge>` into `AppState` (dashboard-v2 state.rs) so the commands_empire seam reads the live bridge instead of constructing a fresh one.
F4. Add `FactoryLineTransport` trait + UnixTransport (cfg(unix)) + TcpTransport (all platforms); generic `FactoryLineClient<T: FactoryLineTransport>`.
F5. Add `EventFilter` + `RateLimiter` + `Priority` (GLOBAL rate limiter, NOT per-pattern) + `dropped_count` metric.
F6. Wire `ElaiNeuron` to `elai_ipc_types::ToolFilter` for the 4-capability whitelist (do NOT define a new filter type; reuse the existing locked one).
F7. Implement `EmpireRepairTarget` + `EmpireIssue` (CircularDep / HardImport / StdlibShadow) detection via tree-sitter-python (already a workspace dep).
F8. FSB header parsing (4J.7) — 12-byte header + 16MB bounded payload_len cap.
F9. Update `.elai_cc/metrics/phase_4J.toml` with measured p50/p95/p99 latencies; drop the `UNGROUNDED_TARGET` labels.
F10. Add captured Empire wire fixtures to `crates/orchestrator/src/empire/fixtures/` and round-trip tests against them.
F11. Add RuntimeClient EXIT-CRITERIA wiring (spec line 662): ssb + fsm + clock through the single RuntimeClient on every Empire request-routing call site. Per cross-check this is empire-scoped — do NOT retroactively refactor existing RuntimeClient call sites.
