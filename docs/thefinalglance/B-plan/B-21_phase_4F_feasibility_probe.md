# B-21: Phase 4F — Feasibility Probe
Objective: implement Phase 4F per spec — system probe (GPU/RAM/disk
detection) + novel step stubs (<50 lines, run in Tier C sandbox) +
SSB branch-and-explore (max 3 alternatives per novel step).
Estimated effort: 2 sessions.
Pre-requisites: B-18 (Tier C sandbox active), B-7 (DecompositionTree
visible — feasibility runs at baby step level).
Parallel-safe with: B-22..B-28 (each is its own phase).

## Context

Per spec phase-4F.txt and phase-4-summary.md PHASE 4F (8 sub-steps):
- 4F.1-4F.6: skeleton, verify, tests, adversarial tests, implement, measure
- 4F.7-4F.8 EXTENSION (Baby Step Feasibility):
  - Probes at baby step level within DecompositionTree (not just top-level)
  - Novel baby step detection: unverified PackageRef, unused framework,
    untested capability
  - Feasibility runs PER CONCEPT (not all at once)
  - New types: BabyStepFeasibilityReport, BabyStepStubResult,
    BabyStepEscalation
  - 24 tests (16 contract + 8 adversarial); 12 testable invariants

Types per spec:
- SystemProbeResult, GpuInfo, SystemConstraints
- StubResult, ResourceDelta, ConstraintResult
- FeasibilityReport, FeasibilityEscalation, FeasibilityDecision
- FeasibilityProbe trait (object-safe, &self)

Hardware: nvml-wrapper (GPU VRAM), sysinfo 0.37+ (RAM/disk).
Stubs: <50 lines strict, Tier C sandbox, 25s graceful / 30s hard kill
(explicit child.kill().await — tokio kill_on_drop is false).
SSB branch-and-explore: max 3 alternatives per novel step.
Skip: novel_steps.is_empty() AND free >= required * 1.20 (all resources).
Escalation: 3 failures -> Escalate (INTERACTIVE) or Replan (AUTONOMOUS).

## Reads

- F:\.ELAI_workflow\docs\specs\guide\steps\phase-4F.txt (full step file)
- summary/phase-4-summary.md PHASE 4F
- crates/orchestrator/src/sandbox.rs (after B-18: real Tier C)
- nvml-wrapper, sysinfo crate docs

## Tasks

1. **SystemProbe** (4F.1): query GPU via nvml-wrapper (VRAM total/free
   per GPU), RAM/disk via sysinfo. Constraint check is pure function.

2. **Novel step detection.** Per spec: novel = unverified PackageRef
   (couples with 4D), unused framework, untested capability.

3. **Stub harness** (4F.5): <50-line stubs in Tier C sandbox; 25s
   graceful + 30s hard kill (explicit child.kill().await). Reuse
   4A agent process with FEASIBILITY tool restrictions.

4. **AgentDispatchConfig overrides for feasibility**:
   - GRC budget: 3/600
   - tool_filter: [stub_write, bash]
   - timeouts: 25s graceful / 30s hard
   - DispatchStage.agent is authoritative (not AgentRole alone)

5. **Skip criteria** (both must hold):
   - zero novel steps AND
   - all resources >= 20% margin

6. **SSB branch-and-explore**: max 3 alternatives per novel step.
   Per concept (not all at once): before executing concept N, probe
   all novel baby steps in concept N. Concept 2's approach may change
   based on concept 1.

7. **Output**: FeasibilityReport stored at stub_results.msgpack.

8. **Dashboard surface**: feasibility events visible in OrchestrationDrawer
   or a new FeasibilityPanel: per-concept probe results, novel step
   counts, alternatives explored.

## WIRING_CONTRACT

produces_symbols:
  - crates/orchestrator/src/system_probe.rs::SystemProbeResult
  - crates/orchestrator/src/system_probe.rs::SystemProber
  - crates/orchestrator/src/system_probe.rs::DefaultSystemProber
  - crates/orchestrator/src/feasibility_probe.rs::FeasibilityProber
  - crates/orchestrator/src/feasibility_probe.rs::DefaultFeasibilityProber
  - crates/orchestrator/src/feasibility_probe.rs::FeasibilityReport
  - crates/orchestrator/src/feasibility_probe.rs::StubResult
  - crates/orchestrator/src/feasibility_probe.rs::FeasibilityEscalation
  - crates/orchestrator/src/feasibility_probe.rs::ResourceDelta
  - crates/orchestrator/src/feasibility_probe.rs::ConstraintResult
  - crates/orchestrator/src/feasibility_probe.rs::FeasibilityDecision
  - crates/orchestrator/src/feasibility_probe.rs::BabyStepFeasibilityReport
  - crates/orchestrator/src/feasibility_probe.rs::BabyStepStubResult
  - crates/orchestrator/src/feasibility_probe.rs::BabyStepEscalation
  - crates/orchestrator/src/ssb/action_type.rs::SystemProbe
  - crates/orchestrator/src/ssb/action_type.rs::FeasibilityReport
  - crates/orchestrator/src/ssb/action_type.rs::NovelStepDetected
  - crates/orchestrator/src/ssb/action_type.rs::StubRun
  - crates/orchestrator/src/ssb/action_type.rs::AlternativeExplored
  - crates/orchestrator/src/ssb/action_type.rs::FeasibilityEscalated
  - crates/orchestrator/src/fsm.rs::FeasibilityProbe
  # WIRING_DIVERGENCE_RESOLVED: plan-side names feasibility_probe_started /
  # feasibility_probe_complete were renamed to system_probe / feasibility_report
  # at implementation time. C-22 closed the remaining gap by atomically adding
  # alternative_explored + feasibility_escalated (SCHEMA 53->54). See
  # workflow/reasoning/C-22-decisions.md D4.
consumes_symbols:
  - crates/orchestrator/src/fsm.rs::FsmState
  - crates/orchestrator/src/feasibility_probe.rs::FeasibilityFsmBridge
  - crates/dashboard-v2/src/lib/components/FeasibilityPanel.svelte
reachable_from:
  - crates/orchestrator/src/task_runner.rs::drive_task
stub_ban:
  - tokio kill_on_drop=true disallowed (must explicit child.kill().await)
  - stubs >50 lines disallowed
frontend_pair: crates/dashboard-v2/src/lib/components/FeasibilityPanel.svelte
observable_effect:
  trigger: "F:\\.ELAI_workflow\\.elai_cc\\elai.bat" run --project-root F:\\.ELAI_workflow
  expect_ssb: system_probe
  expect_artifact: dashboard shows per-concept probe progress

## DETERMINISTIC_CHECK

```bash
# 1. Types defined
rg "struct SystemProbeResult|FeasibilityReport|StubResult" \
   .elai_cc/crates/orchestrator/src
# Expected: 9 types.

# 2. 24 tests
cd .elai_cc && cargo test --package elai-orchestrator feasibility 2>&1 | grep "test result"
# Expected: ≥24 tests.

# 3. SSB action_types added
rg "FeasibilityProbeStarted|NovelStepDetected" .elai_cc/crates/orchestrator/src/ssb.rs
```

## ACCEPTANCE

- [ ] All 8 sub-steps (4F.1-4F.8) deliverables present
- [ ] SystemProbe returns real GPU/RAM/disk telemetry
- [ ] Stubs <50 LOC, run in Tier C sandbox
- [ ] 25s graceful + 30s hard kill enforced
- [ ] Per-concept probe sequencing
- [ ] Skip criteria (no novel + 20% margin) honored
- [ ] FeasibilityReport persisted to msgpack
- [ ] Dashboard FeasibilityPanel renders real probe progress
- [ ] 24+ tests pass

## Files touched

- ADD: crates/orchestrator/src/feasibility.rs (the probe + types)
- MODIFY: crates/orchestrator/src/fsm.rs (FEASIBILITY_PROBE entry/exit emits)
- MODIFY: crates/orchestrator/src/agent_pool.rs (AgentDispatchConfig
  overrides for feasibility role)
- MODIFY: crates/orchestrator/src/task_runner.rs (probe sequencing per concept)
- MODIFY: crates/orchestrator/src/ssb.rs (5 new action_types)
- MODIFY: crates/dashboard-v2/src-tauri/src/lib.rs (V2 — was V1
  crates/dashboard/src/commands.rs; B-36 wires get_feasibility_report
  real path) + ADD crates/dashboard-v2/src/lib/components/FeasibilityPanel
  .svelte (V2 has no FeasibilityPanel yet; consumes
  stub.get_feasibility_report which already exists at stub.ts:194)
- ADD: deps nvml-wrapper, sysinfo (with confirmation)
- CREATE: workflow/reasoning/B-21-decisions.md

## Next

B-22 (Phase 4G Autonomous Mode).
