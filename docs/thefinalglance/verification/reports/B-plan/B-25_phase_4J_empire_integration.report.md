# Verification Report: B-25_phase_4J_empire_integration.md
Source: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-25_phase_4J_empire_integration.md
Date: 2026-05-17 (initial audit), 2026-05-23 (C-34 closure update)
Overall status: PARTIAL (CLOSURE_PENDING_ACTIVATION)
Confidence: HIGH

## C-34 closure update (2026-05-23)

Post initial audit, phase-4J + B-25 host-side scaffolding shipped (see
workflow/reasoning/phase-4J-decisions.md + workflow/reasoning/B-25-decisions.md).
The "MISSING" list below was correct at audit time (2026-05-17); the items
have since been delivered AS HOST-SIDE SCAFFOLDING per the deferral policy
documented in workflow/reasoning/C-34-decisions.md. Live IO bodies stay
EMPIRE-PENDING (47 markers across 16 files) until the Empire codebase
(workspace/factory-brain/) is deployed. The C-34 contract test at
.elai_cc/crates/contracts/tests/c34_empire_integration_contract.rs pins
the SHIPPED interface shapes (8 assertions, all green).

Realized vs original plan path drift (resolved, not regressed):
- crates/empire/ separate top-level crate -> orchestrator::empire submodule
  (phase-4J-decisions.md DECISION 2).
- IpcMessageType extension in shared ipc-types crate -> empire::ipc::
  EmpireIpcMessageType namespace (phase-4J-decisions.md DECISION 4).
- SSB action_type names: plan said
  "empire_repair_started/empire_repair_complete"; B-25 landed
  "empire_event_received/empire_repair_proposed" (same 5-count, different
  semantic split — see C-34-decisions.md DRIFT-C).

Activation patch (out of C-34 scope, gated on Empire codebase access):
- Flip 47 EMPIRE-PENDING markers (single grep-driven follow-up pass).
- Construct Arc<EmpireBridge> at orchestrator startup; chain to AppState
  via with_empire_bridge (B-25-decisions.md F1 + F4-impl-bodies).
- Source-grounding pass (paper-ingestor mode=source) against
  workspace/factory-brain/.

Original 2026-05-17 audit body preserved below for archaeology.

---

## Summary

B-25 is explicitly marked CONDITIONAL ("implement only when Empire codebase is accessible"). Zero Empire-integration symbols exist in the codebase: no empire/ directory under crates/orchestrator/src/, no FactoryLineClient, no ElaiNeuron, no Transport trait variants (UnixTransport/TcpTransport), no SSB action types for empire_*, no EmpireBridgePanel, and no cfg(feature="empire") in any Cargo.toml. The project's own wiring audit (docs/mvp-wiring-audit-2026-04-18.md:258) independently confirms: "4J Empire integration — zero matches (conditional, OK to skip)." No Python brainctl shims were found either, so there is nothing to delete. PipelineSpec and TestOutcomeReport exist but are unrelated to Empire (they are part of pipeline_builder and self-mod pipelines).

## Claims verified DONE

None.

## Claims PARTIAL

None.

## Claims MISSING

- FactoryLine protocol client (pure Rust, newline-JSON, Unix/TCP) — searched: FactoryLine, FactoryLineClient, empire, empire/ dir; no evidence
- Transport trait + UnixTransport + TcpTransport for Empire IPC — searched: UnixTransport, TcpTransport in orchestrator/src/empire/; no evidence (Transport trait exists in unrelated mcp_client.rs/mcp_stdio.rs context only)
- Brain registration with heartbeat (5s) and reconnection — searched: heartbeat, brain.*register, re-registration; no evidence
- brainctl replacement (Python shim deletion, orchestrator-direct FactoryLine) — searched: brainctl; no files found anywhere in .elai_cc/
- Empire event subscription with SSB bridge (rate-limit/filter/priority-classification) — searched: empire_connected, empire_disconnected, empire_event_received, empire_capability_invoked, empire_repair_proposed in ssb.rs and whole .elai_cc/; no matches
- Empire events as GRC SOURCE_CHECK grounding source — searched: empire.*grounding, SOURCE_CHECK context (exists in ipc_gateway.rs:446 but unrelated to Empire); no evidence
- ElaiNeuron MCP server with 4 capabilities (task_submit, task_status, pipeline_result, health_check) — searched: ElaiNeuron, MCP server with these names in empire context; no evidence (task_submitted exists in ssb.rs as internal SSB action, unrelated to MCP capability)
- ToolFilter enforcement on ElaiNeuron capabilities — searched: ElaiNeuron, empire.*ToolFilter; no evidence
- Empire repair pipeline for 3 issue types (CircularDep, HardImport, StdlibShadow) — searched: CircularDep, HardImport, StdlibShadow; no evidence
- PipelineSpec generation per Empire issue + TestOutcomeReport validation — note: PipelineSpec and TestOutcomeReport exist in pipeline_builder/regression/self_mod context but NOT wired to Empire repair; no empire-specific usage found
- cfg(feature="empire") feature gate in Cargo.toml — searched: feature.*empire in orchestrator/Cargo.toml; no matches
- Graceful degradation (elai_cc works without Empire connection) — searched: empire.*degradation, graceful, no evidence of empire-aware startup path
- EmpireBridgePanel Svelte component in dashboard-v2 — searched: EmpireBridgePanel in crates/dashboard-v2/src/; not present in full svelte file listing

## Claims UNVERIFIABLE

- "Access to Empire codebase" prerequisite — the Empire engine itself is external; whether it is accessible cannot be determined from this repo
- "Replace broken brainctl CLI" deletion claim — no brainctl files found so the pre-existing state cannot be confirmed as Python-based or broken

## What must be done (action list)

- [ ] Gain access to Empire codebase (MKI/MKII/MKIII) before any implementation
- [ ] Create crates/orchestrator/src/empire/ module with: client.rs, transport.rs, neuron.rs, repair.rs
- [ ] Add Transport trait with UnixTransport (Linux) + TcpTransport (Windows) in crates/orchestrator/src/empire/transport.rs
- [ ] Implement FactoryLineClient (newline-JSON over socket) in crates/orchestrator/src/empire/client.rs
- [ ] Add brain registration logic with 5s heartbeat + reconnection loop in empire/client.rs
- [ ] Add empire SSB action types to crates/orchestrator/src/ssb.rs: empire_connected, empire_disconnected, empire_event_received, empire_capability_invoked, empire_repair_proposed
- [ ] Wire Empire events as SOURCE_CHECK grounding source in GRC pipeline
- [ ] Implement ElaiNeuron MCP server in crates/orchestrator/src/empire/neuron.rs with 4 capabilities (task_submit, task_status, pipeline_result, health_check) + ToolFilter
- [ ] Implement Empire repair pipeline in crates/orchestrator/src/empire/repair.rs for CircularDep, HardImport, StdlibShadow detection
- [ ] Add cfg(feature="empire") to crates/orchestrator/Cargo.toml and gate all empire modules behind it
- [ ] Add EmpireBridgePanel.svelte to crates/dashboard-v2/src/lib/components/ gated behind feature flag
- [ ] Wire --empire CLI flag in crates/cli/src/args.rs
- [ ] Create workflow/reasoning/B-25-decisions.md

## Notes

The plan is marked CONDITIONAL and the wiring audit explicitly marks this as "OK to skip." All 13 claimed deliverables are absent. This is expected pre-implementation state, not a regression. No action required until Empire codebase access is confirmed.
