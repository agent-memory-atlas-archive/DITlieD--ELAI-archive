# Verification Report: B-21_phase_4F_feasibility_probe.md
Source: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-21_phase_4F_feasibility_probe.md
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary

The core feasibility-probe implementation is present: feasibility_probe.rs, system_probe.rs, FSM FEASIBILITY_PROBE state, task_runner integration, stub_results.msgpack persistence, and 30 contract tests are all verified. The 5 claimed SSB action types exist, but under different names than specified (feasibility_report/system_probe/novel_step_detected/stub_run rather than feasibility_probe_started/feasibility_probe_complete/alternative_explored/feasibility_escalated). The FeasibilityPanel component is absent; feasibility is surfaced inline in ReasoningDrawer which explicitly notes "FeasibilityProbe not yet implemented" for the live-wired path. The agent_pool.rs modification is not present; AgentDispatchConfig overrides live in feasibility_probe.rs itself. The sysinfo dependency is not present; system_probe.rs shells out to nvidia-smi instead.

## Claims verified DONE

- ADD crates/orchestrator/src/feasibility_probe.rs — evidence: F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\feasibility_probe.rs (exists, ~1100+ LOC)
- FeasibilityProber trait (object-safe, &self) — evidence: feasibility_probe.rs:403
- struct StubResult — evidence: feasibility_probe.rs:141
- struct FeasibilityReport — evidence: feasibility_probe.rs:158
- struct BabyStepFeasibilityReport — evidence: feasibility_probe.rs:169
- struct BabyStepStubResult — evidence: feasibility_probe.rs:177
- enum FeasibilityEscalation — evidence: feasibility_probe.rs:150
- enum FeasibilityDecision (Escalate/Replan) — evidence: feasibility_probe.rs:215
- struct ConstraintResult — evidence: feasibility_probe.rs:201
- struct SystemConstraintsRequired — evidence: feasibility_probe.rs:301
- struct NovelStep (with alternatives field) — evidence: feasibility_probe.rs:132
- struct GpuInfo — evidence: system_probe.rs:72
- SystemProbeReport (replaces SystemProbeResult name) — evidence: system_probe.rs:112
- MODIFY crates/orchestrator/src/fsm.rs (FEASIBILITY_PROBE entry/exit) — evidence: fsm.rs:29, 62-63, 92, 940-944
- MODIFY crates/orchestrator/src/task_runner.rs (probe sequencing per concept) — evidence: task_runner.rs:4616, 4621, 4679, 4686, 7776-7850
- FeasibilityReport persisted to stub_results.msgpack — evidence: feasibility_probe.rs:910-932, 1041
- 25s graceful + 30s hard kill enforced (explicit child.kill().await) — evidence: feasibility_probe.rs:45-48, 818-843
- SSB action_types system_probe, feasibility_report, novel_step_detected, stub_run — evidence: ssb.rs:222, 531-540, 1012-1015
- Skip criteria (no novel steps + 20% margin) — evidence: feasibility_probe.rs:53, 80-81, 967-974; RESOURCE_MARGIN_THRESHOLD_PCT=20.0 at :53
- MAX_ALTERNATIVES_PER_STEP = 3 (branch-and-explore cap) — evidence: feasibility_probe.rs:391
- alternative_within_cap() enforces cap — evidence: feasibility_probe.rs:396-397
- AgentDispatchConfig overrides for feasibility (GRC 3/600, tool_filter stub_write+bash) — evidence: feasibility_probe.rs:524-547
- Per-concept probe sequencing in task_runner — evidence: task_runner.rs:4616, 7776, 7782, 4678-4686
- crates/contracts/tests/feasibility_probe_contract.rs — 30 test functions present — evidence: contract file verified; grep count=30
- get_feasibility_report command wired in dashboard-v2 — evidence: dashboard-v2/src-tauri/src/lib.rs:72, commands.rs:433, stub.ts:327
- BabyStepEscalation variants (StubFailed, StubTimeout) — evidence: feasibility_probe.rs:185-187

## Claims PARTIAL

- SSB produces 5 action_types named feasibility_probe_started, feasibility_probe_complete, novel_step_detected, alternative_explored, feasibility_escalated
  - exists: ssb.rs has 4 action_types for feasibility: system_probe, feasibility_report, novel_step_detected, stub_run (ssb.rs:222, 531-540)
  - gap: The plan names feasibility_probe_started and feasibility_probe_complete; actual types are system_probe and feasibility_report. alternative_explored and feasibility_escalated are absent entirely. Only novel_step_detected matches the plan name exactly.

- Dashboard surface: FeasibilityPanel renders real probe progress
  - exists: get_feasibility_report wired in commands.rs:433; feasibility section rendered in ReasoningDrawer.svelte:255-280
  - gap: No standalone FeasibilityPanel.svelte component exists (Glob over dashboard-v2/src returned no match); ReasoningDrawer explicitly notes "FeasibilityProbe not yet implemented" for live path at ReasoningDrawer.svelte:259; d.unwired.feasibility flag is true in drawers.svelte.ts:232

- 24 tests minimum (plan claims 16 contract + 8 adversarial)
  - exists: feasibility_probe_contract.rs has 30 test functions counted
  - gap: 30 exceeds 24, so test count is satisfied; however the split between contract/adversarial cannot be verified by file count alone. Note: plan text says "24 tests (16 contract + 8 adversarial)"; actual test functions total to 30.

- MODIFY crates/orchestrator/src/agent_pool.rs (AgentDispatchConfig overrides for feasibility role)
  - exists: agent_pool.rs file exists at orchestrator/src/agent_pool.rs
  - gap: grep of agent_pool.rs for feasibility shows no matches; the AgentDispatchConfig override is implemented inside feasibility_probe.rs:524-547, not in agent_pool.rs as specified

## Claims MISSING

- Hardware dependency sysinfo 0.37+ for RAM/disk detection — searched: Cargo.toml sysinfo; system_probe.rs line 12 explicitly states "This module DOES NOT add new dependencies (sysinfo / nvml-wrapper)"; RAM/disk obtained via shell-out, not sysinfo crate
- SSB action_types alternative_explored and feasibility_escalated — searched: ssb.rs for "alternative_explored", "feasibility_escalated", "AlternativeExplored", "FeasibilityEscalated"; no evidence
- SSB action_type feasibility_probe_started — searched: ssb.rs; action is named "system_probe" not "feasibility_probe_started"
- SSB action_type feasibility_probe_complete — searched: ssb.rs; action is named "feasibility_report" not "feasibility_probe_complete"
- FeasibilityPanel.svelte component (standalone) — searched: Glob dashboard-v2/src/**/*easibility*; Glob dashboard-v2/src/**/*Panel*; no FeasibilityPanel.svelte found; plan line 138-140 specifies ADD crates/dashboard-v2/src/lib/components/FeasibilityPanel.svelte
- 3-failures -> Escalate / Replan logic in task_runner or feasibility_probe — searched: feasibility_probe.rs for "3 failures", "checks_used >= 3", escalation threshold; no explicit 3-failure counter found in production path (safety_gate_integration.rs:367 references inline_feasibility_state.checks_used but that is a different subsystem)

## Claims UNVERIFIABLE

- B-18 prerequisite (Tier C sandbox active) — reason: B-18 completion status not verifiable from code alone; sandbox.rs exists but Tier C runtime state cannot be confirmed statically
- stub_ban: stubs >50 lines disallowed — reason: enforcement is a test-time invariant (feasibility_probe_contract.rs:572 validates STR_STRICT_LT_50 constant); runtime enforcement of line count on user-supplied stub code is not visible in the probe execution path

## What must be done (action list)

- [ ] Add SSB action_types alternative_explored and feasibility_escalated to ssb.rs to match WIRING_CONTRACT specification (or update B-21 spec to reflect actual names stub_run/feasibility_report)
- [ ] Rename or alias SSB types to match plan: feasibility_probe_started -> system_probe reconciliation, feasibility_probe_complete -> feasibility_report reconciliation; update WIRING_CONTRACT
- [ ] Add FeasibilityPanel.svelte at crates/dashboard-v2/src/lib/components/FeasibilityPanel.svelte with per-concept probe results, novel step counts, alternatives explored
- [ ] Wire d.unwired.feasibility = false in drawers.svelte.ts:232 once FeasibilityPanel is rendering real SSB data
- [ ] Confirm or add sysinfo dependency for RAM/disk OR update B-21 spec to document shell-out approach as the approved method (system_probe.rs already documents this choice; spec still says sysinfo)
- [ ] Add the 3-failures -> Escalate/Replan escalation path in task_runner or feasibility_probe if per-concept probe failure counting is required

## Notes

kill_on_drop(true) is set on the child process at feasibility_probe.rs:685, which the WIRING_CONTRACT stub_ban says is disallowed. This is a direct conflict: the spec bans kill_on_drop=true and requires explicit child.kill().await — but explicit child.kill().await is ALSO present at lines 818 and 832. The kill_on_drop=true at line 685 acts as a safety net; it may be intentional defense-in-depth or may violate the stub_ban. This should be reviewed.

The SystemProbeResult name from the spec does not appear in code; the actual struct is SystemProbeReport (system_probe.rs:112). All callers use SystemProbeReport consistently.
