# C-34: Phase 4J Empire Integration (DEFERRED)
Objective: DEFERRED — awaiting Empire codebase access from upstream maintainer.
This file preserves scope so the work is not lost. Do NOT begin execution until
the external prerequisite is met. Estimated effort: 3 sessions once unblocked.
Pre-requisites: external — Empire codebase access granted by upstream maintainer;
elai_cc.toml [empire.codebase_path] confirmed to point at a readable local clone.
Parallel-safe with: ALL other C-N (orthogonal). This plan does not block any
other C-N delivery.

## Context

Phase 4J (B-25) defined the Empire Integration layer: ELAI connects to the
Empire IDE/runtime via a named-pipe IPC transport (FactoryLineTransport),
exposes an ElaiNeuron MCP server endpoint, and registers a BrainManifest so
Empire can invoke ELAI capabilities as a first-class brain. The repair pipeline
handles the three Empire-specific code pathologies: CircularDep, HardImport,
StdlibShadow. All types live in a feature-gated crates/empire/ workspace member
to keep the core build clean when Empire is absent.

B-25 was never executed because the Empire codebase is not yet available locally.
The scope, types, and test plan are fully specified here so implementation can
begin immediately once access is granted without another planning session.

## Reads (PLANNED — verify paths once Empire access granted)

- F:\.ELAI_workflow\docs\specs\guide\steps\phase-4J.txt (full spec)
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-25_phase_4J_empire_integration.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\steps\phase-4J.report.md
- Empire codebase (TBD path) — source-grounding required at 4J.0

## Tasks

1. **Source-grounding pass (4J.0)** — spawn paper-ingestor (mode=source) against
   the Empire codebase transport and manifest interfaces. Produces
   workflow/papers/empire-protocol-interface.md. Required before any type
   definitions. Reliability tag depends on source quality.

2. **elai_cc.toml [empire] section** — add [empire] block with keys:
   codebase_path, pipe_name, connect_timeout_ms, brain_id. Validate at startup
   only when feature="empire" enabled; log a warning and skip otherwise.

3. **crates/empire/ workspace member (PLANNED)** — feature-gated via
   cfg(feature="empire"). Contains: FactoryLineTransport, FactoryLineClient,
   BrainManifest, EmpireEventBridge, ElaiNeuron, EmpireIssue, FactoryCmd.
   No empire/ code compiled in default build.

4. **IPC extension** — add Empire-protocol variant to IpcMessageType in the
   shared ipc-types crate. Must not break existing Sidecar/AnthropicApi IPC.
   Use a non-exhaustive enum extension or a nested variant to preserve forward
   compatibility.

5. **Repair pipeline types** — CircularDep, HardImport, StdlibShadow structs
   + RepairStrategy enum + RepairPipeline executor. Each pathology gets an
   independent repair pass; pipeline runs in order with per-step SSB emission.

6. **SSB action types** — add 5 new action_type strings to the SSB schema:
   empire_connected, empire_disconnected, empire_capability_invoked,
   empire_repair_started, empire_repair_complete. Each row includes brain_id
   and capability_name fields. SCHEMA_VERSION bump required.

7. **ElaiNeuron MCP endpoint** — HTTP handler that Empire calls to invoke ELAI
   capabilities. Registers the BrainManifest on connect. Runs in its own tokio
   task; shutdown on empire_disconnected SSB row.

8. **Tests first (TDD, 4J.3-4J.7)** — write the following before production
   wiring (4J.8):
   - 4J.3: FactoryLineTransport connect/disconnect round-trip (mock Empire pipe)
   - 4J.4: BrainManifest serialization matches Empire expected schema
   - 4J.5: RepairPipeline emits correct SSB rows per pathology
   - 4J.6: ElaiNeuron MCP invocation returns valid response shape
   - 4J.7: EmpireEventBridge forwards empire_* SSB rows without loss

9. **Production wiring (4J.8)** — wire ElaiNeuron into the orchestrator startup
   path (feature-gated). Bind EmpireEventBridge to the SSB event bus. Wire
   RepairPipeline into the task executor for empire_repair_started events.

10. **--empire CLI flag** — add to CLI arg parser. When present, enables the
    empire feature path and logs connection status at startup.

11. **EmpireBridgePanel.svelte (PLANNED path TBD)** — in dashboard-v2, a panel
    showing: Empire connection status (green/red), active brain_id, last
    capability invoked, repair pipeline history. Uses existing SSB stream.

12. **Contract tests** — add tests/contracts/empire_contract.rs verifying
    FactoryLineTransport, EmpireEventBridge, and RepairPipeline interface shapes
    against the wiring contract (stub_ban enforced).

## WIRING_CONTRACT

produces_symbols:
  - FactoryLineTransport, FactoryLineClient, BrainManifest, EmpireEventBridge
  - ElaiNeuron, EmpireIssue, FactoryCmd (crates/empire/)
  - IpcMessageType::Empire variant (ipc-types crate)
  - CircularDep, HardImport, StdlibShadow, RepairStrategy, RepairPipeline
  - SSB action_types: empire_connected, empire_disconnected,
    empire_capability_invoked, empire_repair_started, empire_repair_complete
  - EmpireBridgePanel.svelte (dashboard-v2, PLANNED)
  - --empire CLI flag
consumes_symbols:
  - IpcMessageType (ipc-types crate, extend non-destructively)
  - SSB event bus (orchestrator)
  - TaskExecutor (orchestrator)
  - elai_cc.toml [empire] config section
reachable_from:
  - elai_cc.toml [empire.codebase_path]
  - CLI --empire flag
  - dashboard EmpireBridgePanel
stub_ban:
  - FactoryLineTransport MUST NOT be a no-op stub in production feature build
  - RepairPipeline MUST emit SSB rows; silent swallow of repair errors is banned
  - ElaiNeuron MUST validate BrainManifest schema before registering
frontend_pair: EmpireBridgePanel.svelte (PLANNED — implement after Rust types)
observable_effect:
  trigger: /task --mode cloud "use Empire capability X" (once unblocked)
  expect_ssb: empire_connected row (non-null brain_id) + empire_capability_invoked
              row (non-null capability_name) present in SSB output
  expect_artifact: EmpireBridgePanel shows connection green + capability name

## DETERMINISTIC_CHECK (DEFERRED — run after Empire access granted)

```bash
# 1. crates/empire/ exists and is feature-gated
grep 'features.*empire' .elai_cc/Cargo.toml

# 2. All 7 empire types present
rg "FactoryLineTransport|FactoryLineClient|BrainManifest|EmpireEventBridge|ElaiNeuron|EmpireIssue|FactoryCmd" .elai_cc/crates/empire/src

# 3. 5 SSB empire action types
rg "empire_connected|empire_disconnected|empire_capability_invoked|empire_repair_started|empire_repair_complete" .elai_cc/crates/orchestrator/src

# 4. TDD contract tests
cargo test -p elai_empire -- empire 2>&1 | grep "test result"

# 5. --empire flag in CLI
rg '\"empire\"' .elai_cc/crates/cli/src/main.rs

# 6. EmpireBridgePanel exists
test -f .elai_cc/crates/dashboard-v2/src/lib/components/EmpireBridgePanel.svelte
```

## ACCEPTANCE (DEFERRED — evaluate after implementation complete)

- [ ] Source-grounding produces workflow/papers/empire-protocol-interface.md
- [ ] crates/empire/ compiles only with feature="empire"; default build clean
- [ ] All 7 Empire types implemented with no placeholder bodies
- [ ] IpcMessageType::Empire variant does not break existing IPC tests
- [ ] RepairPipeline handles CircularDep, HardImport, StdlibShadow independently
- [ ] All 5 SSB empire action types emitted with non-null brain_id + capability_name
- [ ] SCHEMA_VERSION bumped; migration tested
- [ ] TDD tests 4J.3-4J.7 written and passing before 4J.8 production wiring
- [ ] Contract tests in tests/contracts/empire_contract.rs pass
- [ ] ElaiNeuron MCP endpoint reachable; BrainManifest schema valid
- [ ] --empire CLI flag wires the feature path; missing Empire pipe returns
      graceful error (not panic)
- [ ] EmpireBridgePanel renders connection status from SSB stream
- [ ] Cloud Sonnet 4.6 smoke test: /task --mode cloud "use Empire capability X"
      produces SSB rows with empire_connected + empire_capability_invoked,
      non-null brain_id and capability_name fields

## Files (PLANNED — paths TBD until Empire access granted)

- ADD: .elai_cc/crates/empire/src/lib.rs (PLANNED)
- ADD: .elai_cc/crates/empire/src/transport.rs (PLANNED)
- ADD: .elai_cc/crates/empire/src/manifest.rs (PLANNED)
- ADD: .elai_cc/crates/empire/src/bridge.rs (PLANNED)
- ADD: .elai_cc/crates/empire/src/neuron.rs (PLANNED)
- ADD: .elai_cc/crates/empire/src/repair.rs (PLANNED)
- MODIFY: .elai_cc/crates/ipc-types/src/lib.rs (IpcMessageType empire variant)
- MODIFY: .elai_cc/crates/orchestrator/src/task_runner.rs (empire wiring)
- MODIFY: .elai_cc/crates/cli/src/main.rs (--empire flag)
- MODIFY: .elai_cc/Cargo.toml (empire workspace member, feature gate)
- ADD: .elai_cc/crates/dashboard-v2/src/lib/components/EmpireBridgePanel.svelte (PLANNED)
- ADD: .elai_cc/crates/orchestrator/tests/empire_contract.rs (PLANNED)
- ADD: workflow/papers/empire-protocol-interface.md (paper-ingestor output)
- MODIFY: .elai_cc/elai_cc.toml ([empire] section)

## Affected Verification Reports

- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-25_phase_4J_empire_integration.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\steps\phase-4J.report.md

Both reports should be updated to COMPLETE only after all acceptance items above
are checked off in a full audit cycle.

## Next

After C-34 unblocks: run full audit cycle (auditor + wiring-contract-validator),
then memory-writer. Then transition FSM MEMORIZING -> COMPLETE -> IDLE.
