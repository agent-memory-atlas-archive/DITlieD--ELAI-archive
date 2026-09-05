C-34 Empire Integration — Closure Decisions
Session: s-25120
Date: 2026-05-23
Status: CLOSURE_PENDING_ACTIVATION (host-side complete; live IO gated on Empire codebase deployment)
Plan: F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-34_empire_integration_deferred.md
Sibling decision docs (citations canonical):
  - workflow/reasoning/phase-4J-decisions.md  (6 decisions; ELAI submodule path + IPC namespace choices)
  - workflow/reasoning/B-25-decisions.md      (7 B-25-A..G decisions; SSB lockstep + AppState wiring + CLI flag + transports + repair types)

PURPOSE

C-34 is a DEFERRED plan re-opened for closure. The plan documents a 12-task
scope for Phase 4J Empire integration. By the time C-34 was re-opened, 10 of
12 tasks had already shipped via phase-4J + B-25 as HOST-SIDE SCAFFOLDING
(the activation patch — flipping the EMPIRE-PENDING markers and constructing
the real Arc<EmpireBridge> at orchestrator startup — remains gated on the
external Empire codebase deployment). The 2 ACTIVE items for this session
were Task 2 (elai_cc.toml [empire] section) and Task 12 (contract test file).
This document records the closure classification, the realized-vs-planned
drift, and the activation-patch readiness map.

CLASSIFICATION (12 tasks)

  | Task | Subject                                   | Classification              | Cite                                |
  |------|-------------------------------------------|-----------------------------|-------------------------------------|
  | 1    | Source-grounding pass (paper-ingestor)    | EMPIRE-PENDING              | external (Empire codebase absent)   |
  | 2    | elai_cc.toml [empire] section             | ACTIVE (closed this session) | this doc, MW1                       |
  | 3    | crates/empire workspace member            | SHIPPED-VIA-PHASE-4J        | phase-4J-decisions.md DECISION 2    |
  | 4    | IpcMessageType Empire variant extension   | SHIPPED-VIA-B-25            | phase-4J-decisions.md DECISION 4    |
  | 5    | Repair pipeline types                     | SHIPPED-VIA-B-25 (types)    | B-25-decisions.md DECISION B-25-E   |
  | 6    | 5 SSB action_type strings                 | SHIPPED-VIA-B-25            | B-25-decisions.md DECISION B-25-A   |
  | 7    | ElaiNeuron MCP endpoint                   | EMPIRE-PENDING (types ship) | B-25-decisions.md F6 closure note   |
  | 8    | TDD tests 4J.3-4J.7                       | PARTIAL-SHIPPED-VIA-B-25    | B-25-decisions.md F4+F7 closure     |
  | 9    | Production wiring 4J.8                    | EMPIRE-PENDING              | B-25-decisions.md F1 + F4-impls     |
  | 10   | --empire CLI flag                         | SHIPPED-VIA-B-25            | B-25-decisions.md DECISION B-25-C   |
  | 11   | EmpireBridgePanel.svelte                  | SHIPPED-VIA-PHASE-4J+B-25   | B-25-decisions.md DECISION B-25-D   |
  | 12   | Contract tests                            | ACTIVE (closed this session) | this doc, MW2                       |

Totals: SHIPPED-VIA-PHASE-4J=2, SHIPPED-VIA-B-25=4, EMPIRE-PENDING=4,
ACTIVE-this-session=2. Counts match the .elai_cc/metrics/phase_4J.toml
[c34_closure] block appended this session.

MW1 — elai_cc.toml [empire] section (Task 2)

Added [empire] block to elai_cc.toml with 6 fields (defaults in parens):
  enabled              (false)
  transport            ("unix")
  socket_path          ("$XDG_RUNTIME_DIR/elai-empire.sock")
  tcp_host             ("127.0.0.1")
  tcp_port             (7430 — IANA unassigned unprivileged port)
  bridge_state_default ("Disconnected")

Added EmpireConfig struct + Default impl + Field in ElaiRuntimeConfig +
empire_config() accessor + validate_empire_section_when_enabled validator
in F:\.ELAI_workflow\.elai_cc\crates\cli\src\runtime_config.rs.

Validation policy (warn-not-panic, anti-lazy compliance point 2):
  - enabled=false: silent (no-op)
  - enabled=true + empire cargo feature OFF: tracing::warn, continue
  - transport not in {unix,tcp}: tracing::warn, continue
  - bridge_state_default != "Disconnected": tracing::warn, continue
  - transport=tcp + tcp_port=0: tracing::warn, continue

The validator only catches static-config mismatches the user can fix in
elai_cc.toml. The runtime "Empire codebase reachable" check is performed
by EmpireBridge::connect() returning NotConnected (bridge.rs L111-118).

Unit tests in runtime_config.rs::tests (all PASS):
  empire_section_defaults_when_absent
  empire_section_parses_explicit_values
  empire_validator_is_silent_when_disabled
  empire_validator_does_not_panic_on_unknown_transport

MW2 — crates/contracts/tests/c34_empire_integration_contract.rs (Task 12)

NEW contract test file at F:\.ELAI_workflow\.elai_cc\crates\contracts\tests\
c34_empire_integration_contract.rs with 8 assertions. The file is
`#[cfg(feature = "empire")]`-gated; crates/contracts/Cargo.toml gains a
forwarded `empire = ["elai-orchestrator/empire"]` feature so the test
compiles only under `cargo test -p elai_cc_contracts --features empire`.

8 contract assertions:
  C1 — EmpireBridge default state is Disconnected; connect() returns NotConnected
  C2 — FactoryLineTransport trait is dyn-safe (Arc<dyn FactoryLineTransport> compiles)
  C3 — EmpireRepairKind has exactly 4 variants (CircularDep / HardImport /
       StdlibShadow / Other) + detect_repair_kind('') returns EmptyPath
  C4 — [empire] toml section default shape deserializes (6 fields, defaults)
  C5 — ssb_action_type_from_str_public accepts all 5 empire_* labels;
       REJECTS stale plan-text names empire_repair_started + empire_repair_complete
  C6 — AppState-shape Option<Arc<EmpireBridge>> compiles + Arc round-trips
  C7 — emit_action returns NotConnected for ALL 5 EmpireEventKind variants
       in Disconnected state; factory_lines() returns empty Vec
  C8 — ElaiNeuron.tool_filter denies 6 internal capabilities
       (fsm_transition / evidence_gate / rules_query / ssb_direct_write /
       config_modify / agent_spawn) while allowing the 4 whitelisted
       (task_submit / task_status / pipeline_result / health_check);
       EmpireIpcMessageType has 9 variants

Verification:
  cargo check -p elai_cc_contracts --features empire --test c34_empire_integration_contract  -> PASS
  cargo test  -p elai_cc_contracts --features empire --test c34_empire_integration_contract  -> 8/8 PASS

PLAN-vs-REALIZED DRIFT (5 entries, all RESOLVED by citation)

DRIFT-A — Task 3 path
  Plan text (C-34 line 42-46): "crates/empire/ workspace member ... feature-
  gated via cfg(feature='empire'). Contains: FactoryLineTransport,
  FactoryLineClient, BrainManifest, EmpireEventBridge, ElaiNeuron,
  EmpireIssue, FactoryCmd."
  Realized: orchestrator::empire submodule (7 files in crates/orchestrator/
  src/empire/) per phase-4J-decisions.md DECISION 2. Rationale: avoids a
  workspace-member entry + a top-level crate for ~2200 LOC that all belongs
  to one logical surface; the cfg(feature="empire") gate on the orchestrator
  crate already achieves the "zero compile/runtime cost when off" goal.
  ALL 7 promised types ARE present:
    FactoryLineTransport, FactoryLineClient -> factory_line.rs
    BrainManifest                          -> brain_manifest.rs
    EmpireEventBridge (renamed EmpireBridge) -> bridge.rs
    ElaiNeuron                              -> types.rs
    EmpireIssue (renamed EmpireRepairKind) -> repair.rs
    FactoryCmd                              -> types.rs
  Resolution: NO CODE CHANGE. The plan file remains the historical contract;
  this decision doc records the realized path. C-34 Files section line 162-
  173 remains unchanged.

DRIFT-B — Task 4 IPC namespace
  Plan text (C-34 line 48-51): "IPC extension — add Empire-protocol variant
  to IpcMessageType in the shared ipc-types crate. ... Use a non-exhaustive
  enum extension or a nested variant to preserve forward compatibility."
  Realized: orchestrator::empire::ipc::EmpireIpcMessageType (9 variants)
  per phase-4J-decisions.md DECISION 4 + B-25-decisions.md DECISION 4.
  Rationale: elai_ipc_types::IpcMessageType is the INTERNAL orchestrator
  <-> agent IPC type with 16 LOCKED variants, postcard wire, 10+ consumer
  sites (feasibility_probe, task_runner, ipc_accept, agent_role, contract
  tests). Empire IPC is a semantically distinct concept (newline-JSON over
  AF_UNIX, brain<->engine bus). Aliasing the two would conflate two
  separate wire formats and require touching every elai_ipc_types consumer.
  Verification: grep `IpcMessageType::Empire|EmpireIpcMessageType` in
  ipc-types/src/lib.rs = 0 hits (the ipc-types crate is UNTOUCHED).
  Resolution: NO CODE CHANGE. Plan file remains historical; this doc
  records the realized namespace.

DRIFT-C — Task 6 SSB action_type name set
  Plan text (C-34 line 56-59): "5 new action_type strings: empire_connected,
  empire_disconnected, empire_capability_invoked, empire_repair_started,
  empire_repair_complete."
  Realized: 5 SsbActionType arms per B-25-decisions.md DECISION B-25-A:
    empire_connected, empire_disconnected, empire_event_received,
    empire_capability_invoked, empire_repair_proposed
  Semantic difference: the plan's "started + complete" pair conflates the
  repair-event lifecycle (a single repair action with 2 phases). B-25's
  "event_received + repair_proposed" pair separates "Empire-side event
  surfaces on the bus" (e.g. a deploy notification with no repair attached)
  from "elai-side repair proposal" (a typed EmpireRepairProposal payload).
  This is the stronger split — the dashboard can render the two streams
  independently (event ribbon vs repair pipeline pane) without parsing
  structured JSON.
  Count: 5 either way; no variant count drift, no SCHEMA_VERSION drift.
  Verification: grep `empire_connected|empire_disconnected|
  empire_event_received|empire_capability_invoked|empire_repair_proposed`
  in ssb.rs = 22 occurrences across as_sql_str + from_str + SsbPayload +
  payload structs + test_payload + action_type_lock golden + CREATE_SSB_TABLE
  CHECK constraint.
  Resolution: contract C5 in c34_empire_integration_contract.rs ASSERTS
  the realized name set + REJECTS the stale plan-text names so future
  regression is blocked at test time.

DRIFT-D — Contract test path
  Plan text (C-34 line 173): "ADD: .elai_cc/crates/orchestrator/tests/
  empire_contract.rs (PLANNED)".
  Realized: .elai_cc/crates/contracts/tests/c34_empire_integration_contract.rs.
  Rationale: workspace convention for cross-crate contract tests is
  crates/contracts/tests/cNN_<name>_contract.rs per existing pattern
  (c11_sandbox_isolation_contract.rs, c31_file_plan_contract.rs,
  c33_phase_map_contract.rs). The orchestrator crate's own integration
  tests live in crates/orchestrator/tests/ for tests that exercise the
  crate's internal-to-the-module surface; cross-crate WIRING_CONTRACT
  assertions live in crates/contracts/tests/.
  Resolution: file authored at the convention-matching path; this doc
  records the path choice.

DRIFT-E — SCHEMA_VERSION drift past B-25-A close
  B-25-decisions.md DECISION B-25-A narrates SCHEMA_VERSION 101 -> 102 via
  the 7-site QUINTUPLE-UPDATE lockstep.
  Realized today: ssb.rs L1043 reads `const SCHEMA_VERSION: i64 = 103;`.
  The 5 empire arms ARE intact (55 EmpireConnected* match-arm hits + 22
  lowercase empire_* string hits across ssb.rs). The 103 increment is from
  C-5 follow-on (route_blocked / warn_limit_reached / cost_estimate per
  ssb.rs L3448-3451 + L3841-3843).
  Resolution: NO REGRESSION. The empire arms layered cleanly under the
  C-5 bump. Noted here for future archaeology.

DETERMINISTIC_CHECK CORRECTIONS (DC1-DC6)

C-34 plan lines 121-139 list 6 deterministic checks; 5 have path/name drift
and need corrected forms for the auditor pipeline:

DC1 — `grep 'features.*empire' .elai_cc/Cargo.toml`  (DRIFT)
  Corrected: `grep '^empire = ' .elai_cc/crates/orchestrator/Cargo.toml`
  Verification: matches line 175 `empire = []` inside [features] block.

DC2 — `rg "...types..." .elai_cc/crates/empire/src`  (DRIFT — path)
  Corrected: `rg "FactoryLineTransport|FactoryLineClient|BrainManifest|
  ElaiNeuron|EmpireRepairKind|FactoryCmd" .elai_cc/crates/orchestrator/src/empire`
  Verification: returns >=20 hits across 7 files.

DC3 — `rg "empire_connected|empire_disconnected|empire_capability_invoked|
       empire_repair_started|empire_repair_complete" ...`  (DRIFT — names)
  Corrected: `rg "empire_connected|empire_disconnected|empire_event_received|
              empire_capability_invoked|empire_repair_proposed"
              .elai_cc/crates/orchestrator/src`
  Verification: 22 occurrences in ssb.rs.

DC4 — `cargo test -p elai_empire -- empire`  (DRIFT — crate name)
  Corrected: `cargo test -p elai-orchestrator --features empire --lib -- empire`
              + `cargo test -p elai_cc_contracts --features empire --test c34_empire_integration_contract`
  Verification: both PASS per the B-25-decisions.md + the MW2 verification
  block above.

DC5 — `rg '"empire"' .elai_cc/crates/cli/src/main.rs`  (DRIFT — file)
  Corrected: `rg '"--empire"|empire: bool'
              .elai_cc/crates/cli/src/{args.rs,run_subcommand.rs}`
  Verification: 6 hits (args.rs:207 field, args.rs:850 default, args.rs:918
  parser arm, run_subcommand.rs:46 field, run_subcommand.rs:146 + 156).

DC6 — `test -f .elai_cc/crates/dashboard-v2/src/lib/components/
        EmpireBridgePanel.svelte`  (UNCHANGED)
  Verification: file present (B-25-D).

ACTIVATION-PATCH READINESS MAP (4 EMPIRE-PENDING tasks remain)

Single one-line / one-file flips:
  - Task 1: spawn paper-ingestor mode=source path=workspace/factory-brain/
    src/lib/ipc.py,src/core/bus.py (produces workflow/papers/empire-
    protocol-interface.md). Blocked: Empire codebase must be in tree.
  - Task 9 (part 1): construct Arc::new(EmpireBridge::new()) at orchestrator
    startup; chain .with_empire_bridge(bridge) onto AppState. Estimated ~30
    LOC in main.rs / runtime_builder.rs.

Multi-line surface flips (single follow-up patch, grep-driven):
  - Task 7: ElaiNeuron HTTP handler + tokio task lifecycle + shutdown on
    empire_disconnected SSB row. Add neuron_serve() async fn to
    empire/types.rs, spawn from bridge.connect() body once the body is
    flipped from NotConnected -> real socket boot.
  - Task 8 (live round-trip): FactoryLineTransport round-trip /
    EmpireEventBridge SSB forwarding integration tests against a real
    Empire pipe. Cannot run until Empire deploys.
  - Task 9 (part 2): RepairPipeline executor (loops detected issues +
    emits SSB rows per step). Gated on bridge.emit_action body flip.
  - F1 (47 markers): single grep-driven follow-up patch flips every
    EMPIRE-PENDING marker. Run `grep -rn "EMPIRE-PENDING" .elai_cc/` to
    enumerate.
  - F4 (impl bodies): factory_line.rs UnixTransport / TcpTransport
    connect/send/recv bodies (6 functions). Replace NotConnected with
    tokio::net::UnixStream::connect / TcpStream::connect + write_all +
    read_to_end with newline framing.
  - F5 (NEW): EventFilter + RateLimiter + Priority + dropped_count metric
    (~150-250 LOC) in NEW empire/event_filter.rs. GLOBAL rate limiter
    (not per-pattern) per R-NEW-2 from phase-4J spec.
  - F7 (AST walker): replace detect_repair_kind heuristic with
    tree-sitter-python AST walker (~100-200 LOC swap in repair.rs).
  - F8 (NEW): FSB binary transport header parser (12-byte header) in NEW
    empire/fsb.rs (~100-150 LOC). 16MB bounded payload_len cap.
  - F9: replace UNGROUNDED_TARGET perf numbers in phase_4J.toml
    [aspirational_targets] with measured numbers. APPEND new rows; do
    NOT overwrite existing UNGROUNDED_TARGET rows (metrics APPEND-only).
  - F10: captured Empire wire fixtures -> crates/orchestrator/src/empire/
    fixtures/. Needs Empire access. Snapshot real Empire IPC payloads.
  - F11: RuntimeClient EXIT-CRITERIA wiring (empire-scoped only — per
    B-25-decisions.md F11, do NOT retroactively refactor existing
    RuntimeClient call sites). ~3-5 Empire-originated call sites once
    Empire deploys.

ELAI.MAP FOLLOWUP

`grep "FactoryLine|empire_bridge" .claude/tools/elai.map` returns 0 hits;
the codebase map has not been regenerated since the empire submodule
landed. The C-34 acceptance criteria do NOT enumerate elai.map freshness,
so this is OUT OF SCOPE for the closure. Recommend running:
  python .claude/tools/codebase_mapper.py .elai_cc elai_graph.json
         --compact --map elai.map --scip-dir .claude/tools/scip_indices
as a post-session workspace-maintenance task so future impact_radius.py
runs see the empire symbols. NOT a C-34 blocker; documented here so
follow-up sessions schedule the regen.

ANTI-LAZY COMPLIANCE (HARD RULES from executor prompt)

1. ZERO new stubs / TODO / FIXME / placeholder added in this session.
   The 47 EMPIRE-PENDING markers are tracked external-dependency wire-
   points per the documented exception in CLAUDE.md "Zero Lazy Code" rule;
   EVERY marker has a one-line activation note and a corresponding
   gracefully-degrading typed-error return (NotConnected) or empty Vec.
   This session added NO new markers — count stayed at 47.
2. NO unwrap() / expect() introduced in production paths. Test code
   (#[cfg(test)] / tests/) uses .expect("string literal") which is
   allowed per CLAUDE.md "test code unwrap is fine".
3. tracing for validator warns; thiserror was already used by
   EmpireBridgeError; serde derive on EmpireConfig + EmpireToml.
4. cargo check -p elai-orchestrator --features empire: GREEN (49s).
   cargo check -p elai_cc_contracts --features empire
   --test c34_empire_integration_contract: GREEN.
   cargo test  -p elai_cc_contracts --features empire
   --test c34_empire_integration_contract: 8/8 PASS.
   cargo test  -p elai-cli --bins runtime_config::tests::empire:
   4/4 PASS.
5. Contract test imports use exact paths verified by grep against
   crates/orchestrator/src/empire/mod.rs re-export block (no guessed
   paths).

SCHEMA_VERSION

103 (unchanged). The C-34 closure does NOT bump SCHEMA_VERSION — the
5 empire SsbActionType arms shipped at SCHEMA_VERSION 102 (B-25-A) and
sibling C-5 work bumped to 103 (DRIFT-E above). Adding the [empire] toml
section + the contract test file does NOT touch any SSB row or wire
format, so no bump is warranted or attempted.

NEXT

Auditor cycle for C-34 closure (one cycle, scoped):
  cargo check -p elai_cc_contracts --features empire
              --test c34_empire_integration_contract
  cargo test  -p elai_cc_contracts --features empire
              --test c34_empire_integration_contract
  cargo test  -p elai-cli --bins runtime_config::tests::empire
  grep "EMPIRE-PENDING" .elai_cc/ | wc -l   (expect 47)
  grep '^empire = ' .elai_cc/crates/orchestrator/Cargo.toml (expect 1)
  grep '\[empire\]' .elai_cc/elai_cc.toml (expect 1)

Memory-writer cycle: log this closure to memory-bank under "C-34 closed
host-side; activation patch gated on Empire codebase". Cross-reference
phase-4J-decisions.md + B-25-decisions.md.
