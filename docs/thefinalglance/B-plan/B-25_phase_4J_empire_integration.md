# B-25: Phase 4J — Empire Integration (conditional)
Objective: implement Phase 4J per spec — FactoryLine protocol client;
brain registration with heartbeat + reconnection; replace broken
brainctl CLI; Empire event subscription with SSB bridge; capability
exposure (ElaiNeuron with 4 capabilities); Empire repair pipeline for
3 verified issues.
Estimated effort: 2-3 sessions.
Pre-requisites: B-15..B-20 (production hardening); access to Empire
codebase.
Parallel-safe with: B-23, B-24, B-26, B-27, B-28.

CONDITIONAL — implement only when Empire codebase is accessible.

STATUS (post s-25120 closure pass): SHIPPED-ON-V2 host-side scaffolding +
B-25 residual closure. The orchestrator empire submodule, SSB lockstep
(SCHEMA_VERSION 101 -> 102 with 5 atomic empire_* variants), AppState
empire_bridge field, CLI --empire flag, EmpireBridgePanel mount in
SettingsWindow, FactoryLineTransport trait + Unix/Tcp impls, ElaiNeuron
ToolFilter integration, Empire repair pipeline types + heuristic
detector, and reasoning::VerificationMethod::EmpireEventCheck are all
SHIPPED. EMPIRE-PENDING remaining: the impl bodies on the transports +
the bridge's emit_action SSB writer call + FSB binary transport header
parser + EventFilter/RateLimiter — all gated on Empire deployment per
phase-4J-decisions.md F1..F11 followup list. See
workflow/reasoning/B-25-decisions.md for the 7 B-25-specific decisions
that closed the residuals on top of phase-4J's pre-resolved 6.

## Context

Per spec phase-4J.txt and integration/empire.txt
(9 sub-steps: 4J.1-4J.9):

Empire is an existing engine hierarchy (MKI/MKII/MKIII) with IPC dispatch,
hooks, buses, HTTP, events, persistence, WebSocket. Two bus layers:
FactoryLine control socket and IPC engine socket, both newline-JSON
over Unix streams.

6-step integration plan:
1. elai_cc speaks FactoryLine protocol (pure Rust, newline-JSON over
   Unix socket / TCP)
2. registers as a brain with heartbeat + reconnection + re-registration
3. replaces broken brainctl CLI (orchestrator drives FactoryLine
   directly, zero Python deps)
4. subscribes to Empire events with SSB bridge (rate-limited, filtered,
   priority-classified, Empire events become GRC grounding source)
5. exposes capabilities as MCP server (ElaiNeuron registers 4
   capabilities: task_submit, task_status, pipeline_result, health_check
   — ToolFilter enforced, zero internal state leaked)
6. fixes Empire's known issues (detect 3 verified: CircularDep,
   HardImport, StdlibShadow — generate PipelineSpec per issue, validate
   with TestOutcomeReport)

Cross-platform: UnixTransport (Linux) + TcpTransport (Windows) via
generic trait.
Graceful degradation: works without Empire connection.
Feature-gated: cfg(feature="empire").

## Reads

- F:\.ELAI_workflow\docs\specs\guide\steps\phase-4J.txt
- F:\.ELAI_workflow\docs\specs\integration\empire.txt
- summary/integration-summary.md (empire section)
- summary/phase-4-summary.md PHASE 4J

## Tasks

1. **FactoryLine protocol client** (4J.1) [PARTIAL-V2 + SHIPPED trait]:
   pure Rust, newline-JSON, Unix socket on Linux + TCP on Windows.
   Generic trait Transport. [SHIPPED] encode/build_command + newline
   contract unit-tested at empire/factory_line.rs; [SHIPPED via B-25-G]
   FactoryLineTransport trait + UnixTransport (cfg(unix)) + TcpTransport
   + build_transport factory. EMPIRE-PENDING: socket boot impl bodies.
2. **Brain registration** (4J.2) [PARTIAL-V2]: heartbeat (5s),
   reconnection on disconnect, re-registration after Empire restart.
   [SHIPPED] BrainManifest::validate with 4 rules; heartbeat_interval_ms
   default 5000 on FactoryLineConfig::default_unix. EMPIRE-PENDING:
   the live heartbeat loop / reconnect / re-register.
3. **Replace brainctl** (4J.3) [N/A]: grep `brainctl` repo-wide = 0
   files; deletion-side satisfied by absence; driver-side EMPIRE-PENDING.
4. **Event subscription with SSB bridge** (4J.4) [PARTIAL-V2 +
   SHIPPED-ON-V2 SSB lockstep]:
   - Rate-limited (configurable, default 100 events/s) — config knob
     SHIPPED; live rate-limiter EMPIRE-PENDING (F5)
   - Filtered (per-event-type allowlist in config) — EMPIRE-PENDING (F5)
   - Priority-classified (LOW/MEDIUM/HIGH) — EMPIRE-PENDING (F5)
   - Becomes GRC grounding source (Empire event = SOURCE_CHECK
     grounding type) — [SHIPPED via B-25-F]
     reasoning::VerificationMethod::EmpireEventCheck { source,
     event_type } + requires_empire_bridge predicate
   - [SHIPPED via B-25-A] the 5 SSB action_type variants
     (empire_connected/empire_disconnected/empire_event_received/
     empire_capability_invoked/empire_repair_proposed) with full
     QUINTUPLE-UPDATE lockstep + SCHEMA_VERSION 101 -> 102 bump
5. **ElaiNeuron MCP server** (4J.5) [PARTIAL-V2 + SHIPPED ToolFilter]:
   registers 4 capabilities:
   - task_submit: queue a task for elai_cc execution
   - task_status: query task by id
   - pipeline_result: retrieve completed pipeline output
   - health_check: liveness probe
   ToolFilter enforced. [SHIPPED] ElaiNeuron::tool_filter field reuses
   elai_ipc_types::ToolFilter; ElaiNeuron::allows is the authoritative
   gate (denies the 6 internal capabilities by adversarial test).
   EMPIRE-PENDING: the MCP server request dispatcher itself.
6. **Empire repair pipeline** (4J.6) [PARTIAL-V2 — heuristic SHIPPED]:
   detect 3 verified issues:
   - CircularDep: import-graph cycle detection (heuristic SHIPPED;
     EMPIRE-PENDING AST walker)
   - HardImport: hardcoded path imports (heuristic SHIPPED)
   - StdlibShadow: user module shadowing stdlib (heuristic SHIPPED;
     24-entry stdlib name list; POSIX + Windows-path tolerant)
   [SHIPPED via B-25-E] EmpireRepairKind / EmpireRepairProposal /
   EmpireRepairError types at empire::repair plus
   detect_repair_kind(symbol_path). For each: PipelineSpec generation +
   TestOutcomeReport validation + commit-to-Empire EMPIRE-PENDING.
7. **cfg(feature="empire") gating** + graceful degradation [SHIPPED-ON-V2]:
   works without Empire.
8. **Dashboard surface** [PARTIAL-V2 + SHIPPED mount]: EmpireBridgePanel
   shows connection state [SHIPPED], last 20 events received
   [EMPIRE-PENDING], capability exposure status [EMPIRE-PENDING],
   repair pipeline results [EMPIRE-PENDING]. [SHIPPED via B-25-D]
   panel mounted in SettingsWindow as a sibling section between
   `runtime backends` and `cloud`.

9. **B-25 residual decisions** (NEW, this session) — see
   workflow/reasoning/B-25-decisions.md for the 7 B-25-specific
   decisions (B-25-A..B-25-G) layered on top of phase-4J-decisions.md's
   pre-resolved 6.

## WIRING_CONTRACT

produces_symbols:
  - FactoryLineClient
  - ElaiNeuron MCP server
  - Empire repair pipeline (3 issue types)
  - Transport trait + UnixTransport + TcpTransport
  - SSB action_types: empire_connected, empire_disconnected,
    empire_event_received, empire_capability_invoked, empire_repair_proposed
consumes_symbols:
  - GRC grounding (Empire event source)
  - dashboard EmpireBridgePanel
reachable_from:
  - cli with --empire flag (or feature enabled)
stub_ban:
  - Empire connection failure crashing elai_cc disallowed (graceful degradation)
  - Internal state leaks via MCP capability disallowed
frontend_pair: EmpireBridgePanel
observable_effect:
  trigger: start Empire + start elai_cc with --empire
  expect_ssb: empire_connected within 5s; empire_event_received per Empire event
  expect_artifact: dashboard shows live event stream

## DETERMINISTIC_CHECK

```bash
# 1. Feature flag
rg "feature\\s*=\\s*\"empire\"" .elai_cc/crates/orchestrator/Cargo.toml
# 2. Transport trait
rg "trait Transport" .elai_cc/crates/orchestrator/src/empire/
```

## ACCEPTANCE

- [ ] All 9 sub-steps deliverables present
- [ ] FactoryLine client speaks protocol correctly
- [ ] Brain registers with heartbeat + reconnect
- [ ] brainctl replaced (no Python deps)
- [ ] Event subscription with rate-limit/filter/priority
- [ ] ElaiNeuron exposes 4 capabilities; ToolFilter enforced
- [ ] 3 Empire issues detected + repair pipelines generated
- [ ] cfg(feature="empire") gating works
- [ ] Graceful degradation (no Empire = elai_cc still works)
- [ ] dashboard EmpireBridgePanel renders live state

## Files touched

- ADD [SHIPPED-ON-V2]: crates/orchestrator/src/empire/* (client, transport,
  neuron, repair) — 7 files: mod.rs + types.rs + ipc.rs + brain_manifest.rs +
  factory_line.rs (now also holds FactoryLineTransport trait + Unix/Tcp
  impls per B-25-G) + bridge.rs + repair.rs (NEW per B-25-E)
- MODIFY [SHIPPED-ON-V2]: crates/orchestrator/Cargo.toml (cfg feature empire)
- MODIFY [SHIPPED via B-25-A]: crates/orchestrator/src/ssb.rs
  (SCHEMA_VERSION 101 -> 102; 7-site QUINTUPLE-UPDATE for 5 empire_*
  action types; 5 new payload structs)
- MODIFY [SHIPPED via B-25-A]: crates/orchestrator/tests/action_type_lock.rs
  (TOTAL_VARIANTS 298 -> 303; 5 ALL_VARIANTS entries; exhaustive_match_guard
  arms) + tests/action_type_lock.golden (regenerated to 303 lines)
- MODIFY [SHIPPED via B-25-F]: crates/reasoning/src/types.rs
  (VerificationMethod::EmpireEventCheck + requires_empire_bridge)
- MODIFY [SHIPPED-ON-V2]: crates/dashboard-v2/src-tauri/src/lib.rs
  (V2 — was V1 crates/dashboard/src/commands.rs)
- MODIFY [SHIPPED via B-25-B]: crates/dashboard-v2/src-tauri/src/state.rs
  (AppState.empire_bridge field + with_empire_bridge installer +
  empire_bridge_snapshot helper, all cfg(feature="empire") gated)
- MODIFY [SHIPPED-ON-V2]: crates/dashboard-v2/src-tauri/src/commands_empire.rs
  (now reads state.empire_bridge_snapshot() with disconnected fallback)
- ADD [SHIPPED-ON-V2]: crates/dashboard-v2/src/lib/components/
  EmpireBridgePanel.svelte (V2 has no equivalent — gated behind
  cfg(feature="empire") same as the orchestrator side)
- ADD [SHIPPED-ON-V2]: crates/dashboard-v2/src/lib/api/empire.ts (TS wrappers)
- MODIFY [SHIPPED via B-25-D]: crates/dashboard-v2/src/lib/windows/
  SettingsWindow.svelte (mounts EmpireBridgePanel in a new section between
  `runtime backends` and `cloud`)
- MODIFY [SHIPPED via B-25-C]: crates/cli/src/args.rs +
  crates/cli/src/main.rs + crates/cli/src/run_subcommand.rs (--empire flag
  on `elai run`)
- DELETE [N/A — file absent]: any Python brainctl shims
- CREATE [SHIPPED]: workflow/reasoning/B-25-decisions.md
- DEFERRED [EMPIRE-PENDING]: FactoryLineTransport impl bodies +
  EmpireBridge::emit_action SSB writer call + FSB binary transport
  header + EventFilter/RateLimiter — per phase-4J-decisions.md F1..F11

## Next

B-26 (4K.6-4K.13 Dashboard Completion).
