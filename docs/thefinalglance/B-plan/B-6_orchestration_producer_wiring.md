# B-6: Producer Wiring — Orchestration Core
Objective: surface 7 evidence gates, agents panel, supervision events,
BSP visualization, and ContextBudget per-slot allocation in the dashboard.
Estimated effort: 2 sessions.
Pre-requisites: B-2 (FSM emit pattern). B-10 (ContextBudget profile fix)
should land near this for the budget viz to be meaningful.
Parallel-safe with: B-4, B-5.

## V2 REBASELINE NOTE (2026-05-07)

Producer-side work is V2-agnostic. Dashboard surface targets:
  V1 crates/dashboard/src/commands.rs            -> V2 crates/dashboard-v2/
                                                     src-tauri/src/lib.rs (B-36)
  V1 src-frontend/src/orchestration/AgentsPanel
     .svelte                                     -> Render inside V2's
                                                     OrchestrationDrawer
                                                     .svelte (already
                                                     scaffolded — stub-backed
                                                     via stub.ts: get_agents,
                                                     get_supervision_log,
                                                     get_evidence_gates,
                                                     get_context_budget,
                                                     get_bsp_supersteps).
                                                     If panels need own files,
                                                     add under
                                                     crates/dashboard-v2/src/lib/
                                                     orchestration/
  V1 EvidenceGatesPanel.svelte                   -> ditto OrchestrationDrawer
  V1 BudgetBar.svelte                            -> ditto (orbit also has
                                                     OrbitBudgetRing.svelte
                                                     for the canvas
                                                     surfacing — coordinate)

V2 ActivityWindow surfaces the 7 evidence-gate pips between phase ring and
step ring (DOMAIN-CONTEXT §2 ACTIVITY signals layer); the drawer is the
detailed view.

## Context

Per audit-dashboard/orchestration-core-wiring.md: 3A InvisibleGate, 3C
agents/supervision dead, 3D BSP not visualized, 3E budget zeroed slots
not visualized. The user can't see the orchestrator working at all even
though 3A-3E are 95%+ implemented in code.

## Reads

- audit-dashboard/orchestration-core-wiring.md
- audit-implementation/phase-3A.md, 3B.md, 3C.md, 3D.md, 3E.md
- summary/phase-3-summary.md
- crates/orchestrator/src/evidence.rs (7 EvidenceGate impls)
- crates/orchestrator/src/agent_pool.rs (8 AgentRoles, lifecycle)
- crates/orchestrator/src/bsp/engine.rs (superstep loop)
- crates/orchestrator/src/context_budget.rs (8 slots, profile machinery)

## Tasks

1. **EvidenceGate verdict panel.** New SSB action `evidence_gate_evaluated`
   payload {gate_name, transition_from, transition_to, verdict (Pass|Block),
   missing_artifacts[], suggestion}. Emit from each of 7 gates after
   evaluation. Tauri command `get_recent_evidence_gates(limit)`. Frontend:
   add to OrchestrationDrawer — last 7 gate evaluations with green/red
   verdict + missing-artifact details.

2. **InteractiveGate prompt UI.** When CliInteractiveGate (B-2 pattern,
   wire it into a real loop here) needs human approval, write SSB action
   `intervention_request` payload {kind: Approve|Reject|Modify|Redirect,
   context, options[]}. Dashboard renders intervention card; user choice
   writes `intervention_response`. Orchestrator unblocks. (Couples with
   B-26 deferred MONITOR window if doing autonomous mode; for B-6 the
   INTERACTIVE mode wiring is sufficient.)

3. **Agents panel.** New Tauri command `get_active_agents()` returns
   Vec<{role, agent_id, status: Spawned|Running|Completed|Crashed|TimedOut,
   spawned_at, last_heartbeat_ms, restart_count}>. Frontend: new component
   `AgentsPanel.svelte` in OrchestrationDrawer or as standalone window.
   Shows 8 role rows with current status, heartbeat liveness dot
   (green pulse, amber if >5s, red if >15s), restart count.

4. **Supervision event log.** Wire SupervisionManager into task_runner
   (per audit phase-3C R1 critical blocker — currently UNWIRED).
   Add SSB action `supervision_event` payload {agent_id, event:
   Spawn|Heartbeat|Restart|MaxRestartsExhausted|Escalate, reason}.
   Dashboard shows last 20 events.

5. **Anti-spiral hits visible.** SpiralTracker already wired
   (task_runner.rs:1046-1087) but events not surfaced. Add SSB
   `anti_spiral_hit` emit + dashboard renderer (red badge on agent panel
   when count==3, escalation pulse when escalated).

6. **BSP superstep counter + parallel-agent run.** Add SSB
   `bsp_superstep` emit per superstep with {step_number, channel_writes,
   parallel_agents_spawned, parallel_agents_completed, duration_ms,
   straggler_count}. Frontend: small inline indicator in StatusBar
   "Superstep #42 (3 agents in flight)".

7. **ContextBudget per-slot bar chart.** New Tauri command
   `get_last_context_budget(turn_id)` returns Vec<{slot_name (8 slots),
   tokens_allocated, tokens_used, fixed (bool)}>. Frontend: stacked
   horizontal bar chart in StatusBar or in a dedicated drawer section.
   Color: teal=used, dim teal=allocated-unused, red=overflow.
   When ModelCapabilityProfile is None (TaskRunner today; will be fixed
   in B-10), show "(degraded: no profile, falling back to zeros)".

8. **AdaptiveLevel events.** When 3-level adaptive reduction fires
   (FilteredByRelevance → ReducedGranularity → ReducedScope → Terminal),
   emit SSB `context_reduction` per level. Dashboard log + count badge.

9. **file_overlap_score (3E.5) producer.** Implement the missing
   `compute_overlap()` function and wire it into SSB entry filtering
   per spec. SSB action `ssb_filtered_by_overlap` per turn shows
   {entries_total, entries_kept, overlap_threshold}.

10. **SupervisionManager wired into task_runner** — fixes audit phase-3C
    blocker R1 "infinite restart loops are possible in production".

## WIRING_CONTRACT

produces_symbols:
  - EvidenceGateEvaluated
  - InterventionRequest
  - InterventionResponse
  - SupervisionEvent
  - AntiSpiralHit
  - BspSuperstep
  - ContextReduction
  - SsbFilteredByOverlap
  - AgentStatusChanged
  - get_evidence_gates
  - get_agents
  - get_supervision_log
  - get_context_budget
  - get_bsp_supersteps
consumes_symbols:
  - OrchestrationDrawer.svelte
  - StatusBar.svelte
reachable_from:
  - task_runner.rs::drive_task
  - dashboard task submit
stub_ban:
  - SupervisionManager wired (no longer dead)
  - InteractiveGate actually instantiated (no longer cli/main.rs:7354 dead)
  - file_overlap_score function exists
frontend_pair: same B-step
observable_effect:
  trigger: "F:/.ELAI_workflow/.elai_cc/target/debug/elai.exe" task "echo hello" --repo-root "F:/.ELAI_workflow/.elai_cc" --mode local --max-wall-seconds 10 --output-format json
  expect_ssb: task_submitted
  expect_artifact: task_submitted

## DETERMINISTIC_CHECK

```bash
# 1. SupervisionManager has production callers
rg "SupervisionManager::new\\b" .elai_cc/crates/orchestrator/src/task_runner.rs
# Expected: ≥1 hit.

# 2. InteractiveGate instantiated
rg "CliInteractiveGate::new\\b" .elai_cc/crates/cli/src --glob '!*test*'
# Expected: ≥1 hit (no longer test-only).

# 3. file_overlap_score function exists
rg "fn (compute_)?file_overlap_score|fn compute_overlap" .elai_cc/crates/orchestrator
# Expected: ≥1 hit.

# 4. Smoke: submit task, verify all 9 new SSB action_types fire
```

## ACCEPTANCE

- [ ] EvidenceGate panel shows 7 gate evaluations
- [ ] InteractiveGate prompts surface as intervention cards
- [ ] AgentsPanel shows 8 roles with live status
- [ ] SupervisionManager wired (no infinite restart loop possible)
- [ ] Anti-spiral hits visible
- [ ] BSP superstep counter visible
- [ ] ContextBudget per-slot bar chart visible
- [ ] AdaptiveLevel reduction events visible
- [ ] file_overlap_score computed (3E.5 closed)
- [ ] No "always IDLE" agent statuses (heartbeat pulses)

## Files touched

- MODIFY: crates/orchestrator/src/evidence.rs (emit per gate)
- MODIFY: crates/orchestrator/src/agent_pool.rs (status emit + supervision wire)
- MODIFY: crates/orchestrator/src/task_runner.rs (SupervisionManager,
  CliInteractiveGate)
- MODIFY: crates/orchestrator/src/bsp/engine.rs (superstep emit)
- MODIFY: crates/orchestrator/src/context_budget.rs
  (compute_overlap + AdaptiveLevel emit)
- MODIFY: crates/dashboard/src/commands.rs + ssb_viewer.rs
- ADD: crates/dashboard/src-frontend/src/orchestration/AgentsPanel.svelte
- ADD: crates/dashboard/src-frontend/src/orchestration/EvidenceGatesPanel.svelte
- ADD: crates/dashboard/src-frontend/src/orchestration/BudgetBar.svelte
- CREATE: workflow/reasoning/B-6-decisions.md

## Next

B-7 (Pipeline Factory wiring), B-8 (PipelineGrc activation).
