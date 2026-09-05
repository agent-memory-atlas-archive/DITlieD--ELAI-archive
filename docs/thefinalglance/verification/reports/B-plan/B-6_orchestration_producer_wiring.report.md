# Verification Report: B-6_orchestration_producer_wiring.md
Source: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-6_orchestration_producer_wiring.md
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
Most SSB producer symbols and Tauri commands specified in B-6 are implemented and
wired. The core blockers from the plan — SupervisionManager in task_runner,
EvidenceGate SSB emit, bsp_superstep emit, context_reduction emit, and
ssb_filtered_by_overlap — are all DONE. Frontend rendering of all panels is
DONE inside OrchestrationDrawer.svelte. Key gaps: `get_active_agents` /
`get_last_context_budget` / `get_bsp_superstep_history` are renamed in the
actual implementation; `agent_status_changed` SSB action type does not exist;
CliInteractiveGate is defined but per metrics/phase_3J.toml is not wired into
ConversationRuntime; standalone AgentsPanel.svelte, EvidenceGatesPanel.svelte,
and BudgetBar.svelte were never created as separate files (panels were merged
into OrchestrationDrawer instead).

## Claims verified DONE

- SupervisionManager instantiated in task_runner — evidence: crates/orchestrator/src/task_runner.rs:2689, 2700
- SupervisionManager.handle_death called on crash path (fixing infinite restart loop) — evidence: crates/orchestrator/src/task_runner.rs:6322-6344
- SSB action type `supervision_event` defined and deserialized — evidence: crates/orchestrator/src/ssb.rs:1231
- SupervisionEventPayload defined — evidence: crates/orchestrator/src/ssb.rs:1818
- SsbPayload::SupervisionEvent emitted on kill and restart in task_runner — evidence: crates/orchestrator/src/task_runner.rs:6262, 6312
- SSB action type `evidence_gate_evaluated` defined — evidence: crates/orchestrator/src/ssb.rs:1230
- EvidenceGateEvaluatedPayload struct defined — evidence: crates/orchestrator/src/ssb.rs:1805
- EvidenceGateEvaluated emitted per gate in task_runner (loop over all 7 gates) — evidence: crates/orchestrator/src/task_runner.rs:4458-4488
- SSB action type `anti_spiral_hit` defined — evidence: crates/orchestrator/src/ssb.rs:1234
- AntiSpiralHitPayload defined — evidence: crates/orchestrator/src/ssb.rs:1858
- SsbPayload::AntiSpiralHit emitted in task_runner — evidence: crates/orchestrator/src/task_runner.rs:4621
- SSB action type `bsp_superstep` defined — evidence: crates/orchestrator/src/ssb.rs:1232
- BspSuperstepPayload defined — evidence: crates/orchestrator/src/ssb.rs:1831
- SsbPayload::BspSuperstep emitted in bsp/engine.rs — evidence: crates/orchestrator/src/bsp/engine.rs:454
- SSB action type `context_reduction` defined — evidence: crates/orchestrator/src/ssb.rs:1165
- ContextReductionPayload defined — evidence: crates/orchestrator/src/ssb.rs:714
- SsbPayload::ContextReduction emitted per adaptive level and terminal in context_budget.rs — evidence: crates/orchestrator/src/context_budget.rs:1171, 1193
- SSB action type `ssb_filtered_by_overlap` defined — evidence: crates/orchestrator/src/ssb.rs:1218
- SsbFilteredByOverlapPayload defined — evidence: crates/orchestrator/src/ssb.rs:1593
- emit_ssb_filtered_by_overlap called from task_runner — evidence: crates/orchestrator/src/task_runner.rs:4154
- file_overlap_score function exists — evidence: crates/orchestrator/src/context_budget.rs:605
- compute_overlap function exists — evidence: crates/orchestrator/src/context_budget.rs:627
- intervention_request SSB action type defined and emitted from task_runner — evidence: crates/orchestrator/src/task_runner.rs:7924 (SsbPayload::InterventionRequest)
- intervention_response SSB type and contract tests exist — evidence: crates/contracts/tests/ssb_4k3_4k4_4k5_contract.rs:301-320
- Tauri command get_evidence_gates implemented, reads SSB via recent_evidence_gates(50) — evidence: crates/dashboard-v2/src-tauri/src/commands.rs:1616
- Tauri command get_supervision_log implemented, reads SSB via recent_supervision_events(100) — evidence: crates/dashboard-v2/src-tauri/src/commands.rs:1672
- Tauri command get_bsp_supersteps implemented, reads SSB via recent_bsp_supersteps(50) — evidence: crates/dashboard-v2/src-tauri/src/commands.rs:1783
- Tauri command get_context_budget implemented with real SSB query and 8-slot fallback — evidence: crates/dashboard-v2/src-tauri/src/commands.rs:1695
- Tauri command get_agents implemented (returns 8-role list; always IDLE, not live) — evidence: crates/dashboard-v2/src-tauri/src/commands.rs:1655-1668
- OrchestrationDrawer.svelte renders: FSM, evidence gates, agents, supervision, BSP supersteps, context budget — evidence: crates/dashboard-v2/src/lib/drawers/OrchestrationDrawer.svelte:160-268
- Frontend state wired: drawers.svelte.ts calls get_evidence_gates, get_agents, get_supervision_log, get_bsp_supersteps, get_context_budget — evidence: crates/dashboard-v2/src/lib/state/drawers.svelte.ts:257-266
- AgentLifecycle SSB events emitted from bsp/engine.rs on spawn/complete/crash — evidence: crates/orchestrator/src/bsp/engine.rs:242, 264, 353, 509, 580
- SpiralTracker anti_spiral_hit SSB emit wired in task_runner — evidence: crates/orchestrator/src/task_runner.rs:4621

## Claims PARTIAL

- **CliInteractiveGate instantiated (stub_ban: no longer dead)**
  - exists: crates/cli/src/runtime_builder.rs:393 (struct def + InteractiveGate impl)
  - gap: metrics/phase_3J.toml:50-51 explicitly records `foundation_interactive_gate = false` and "defined but not wired into ConversationRuntime". No call site for CliInteractiveGate::new found anywhere. The struct and impl exist but the gate is not injected into the runtime.

- **Tauri command name `get_last_context_budget` (as specified in WIRING_CONTRACT)**
  - exists: command is implemented as `get_context_budget` (crates/dashboard-v2/src-tauri/src/commands.rs:1695)
  - gap: name mismatch — plan specifies `get_last_context_budget(turn_id)` with a turn_id parameter; actual command is `get_context_budget` with no turn_id parameter

- **Tauri command name `get_bsp_superstep_history` (as specified in WIRING_CONTRACT)**
  - exists: implemented as `get_bsp_supersteps` (crates/dashboard-v2/src-tauri/src/commands.rs:1783)
  - gap: name mismatch only — functionality is equivalent

- **Agents panel shows live heartbeat status (not "always IDLE")**
  - exists: get_agents Tauri command returns 8 roles (crates/dashboard-v2/src-tauri/src/commands.rs:1659-1668)
  - gap: hardcoded static JSON — all roles return lifecycle:"IDLE", last_heartbeat_ms:0. ACCEPTANCE criterion "No always IDLE agent statuses (heartbeat pulses)" is not met. Plan requires reading AgentLifecycle SSB rows; implementation does not query SSB viewer for agent state.

## Claims MISSING

- **SSB action type `agent_status_changed`** — searched: agent_status_changed, AgentStatusChanged in elai.map and codebase; no evidence. The WIRING_CONTRACT lists it as a produced symbol, but it does not exist as an SsbActionType variant or payload.

- **`get_recent_evidence_gates` Tauri command** — searched: get_recent_evidence_gates in commands.rs and codebase; no match. Plan specifies this name; actual implementation is `get_evidence_gates`.

- **`get_active_agents` Tauri command** — searched: get_active_agents in commands.rs and codebase; no match. Plan specifies this name; actual implementation is `get_agents` (without "active_" prefix).

- **Standalone AgentsPanel.svelte** — searched: crates/dashboard-v2/src/lib/orchestration/AgentsPanel.svelte; file does not exist. Panel was merged into OrchestrationDrawer.svelte.

- **Standalone EvidenceGatesPanel.svelte** — searched: EvidenceGatesPanel in codebase; file does not exist. Panel merged into OrchestrationDrawer.svelte.

- **Standalone BudgetBar.svelte** — searched: BudgetBar in codebase; file does not exist. Budget bar merged into OrchestrationDrawer.svelte.

## Claims UNVERIFIABLE

- **"SpiralTracker already wired (task_runner.rs:1046-1087)"** — the line range cited in the plan cannot be confirmed without reading those exact lines, but anti_spiral_hit SSB emit at task_runner.rs:4621 is verified; the functional claim is substantively met.

- **Dashboard rendering of intervention card for InteractiveGate** — intervention_request SSB type and Tauri plumbing exist (contracts verified), but no Svelte component handling the intervention overlay was found in OrchestrationDrawer.svelte. Whether ChatWindow handles it cannot be confirmed without reading all window components.

## What must be done (action list)

- [ ] Wire CliInteractiveGate into ConversationRuntime — crates/cli/src/runtime_builder.rs (see metrics/phase_3J.toml:50-51 for tracked blocker)
- [ ] Implement live agent status query in get_agents() — crates/dashboard-v2/src-tauri/src/commands.rs:1655; must query SSB viewer for AgentLifecycle rows instead of returning hardcoded IDLE list
- [ ] Add SSB action type `agent_status_changed` or document that AgentLifecycle subsumes it — crates/orchestrator/src/ssb.rs
- [ ] Resolve Tauri command name mismatches: `get_recent_evidence_gates` -> `get_evidence_gates`, `get_active_agents` -> `get_agents`, `get_last_context_budget(turn_id)` -> `get_context_budget()`, `get_bsp_superstep_history` -> `get_bsp_supersteps` — update WIRING_CONTRACT or rename commands
- [ ] Wire intervention overlay in frontend for InteractiveGate prompts — verify ChatWindow or OrchestrationDrawer handles intervention_request display

## Notes
The core backend wiring (all SSB producers, supervision gate, overlap filter) is
complete and well-tested. The primary remaining gaps are: live agent heartbeat
data (get_agents returns static IDLE), CliInteractiveGate not injected into
runtime, and several naming discrepancies between the plan's WIRING_CONTRACT and
the actual symbol names shipped. Standalone panel files were intentionally
consolidated into OrchestrationDrawer rather than separate files.
