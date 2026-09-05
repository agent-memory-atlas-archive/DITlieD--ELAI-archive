B-6 Orchestration Core Producer Wiring — DECISIONS
Date: 2026-06-05 (backfill)
Source: workflow/sessions/s-plan-audit-2026-06-04/plan-status/B/B-6_orchestration_producer_wiring.md

SHIPPED
- 9 SSB action types in ssb.rs with payloads: evidence_gate_evaluated, supervision_event,
  anti_spiral_hit, bsp_superstep, context_reduction, ssb_filtered_by_overlap, intervention_request,
  intervention_response, agent_status_changed.
- Emit call sites (production): EvidenceGateEvaluated task_runner.rs:6153/9072; AntiSpiralHit :6670;
  SupervisionEvent :12368/12418; BspSuperstep bsp/engine.rs:506; ContextReduction context_budget.rs:
  1549/1571; SsbFilteredByOverlap task_runner.rs:5670; InterventionRequest :15158; AgentStatusChanged
  at bsp/engine.rs sites.
- 5 Tauri commands (registered lib.rs:639-643): get_evidence_gates (3074), get_agents (3113),
  get_supervision_log (3171), get_context_budget (3194), get_bsp_supersteps (3282); stub.ts wrappers
  via tryInvoke for all five.
- OrchestrationDrawer.svelte pulls all 9 streams via Promise.all in refresh().
- CliInteractiveGate installed at run_subcommand.rs:120 from the real CLI entrypoint;
  SupervisionManager::new() in TaskRunner init; file_overlap_score/compute_overlap wired
  (context_budget.rs:983/1005 -> task_runner.rs:5628-5668).

KEY DECISIONS
- Command rename: planned get_recent_evidence_gates / get_active_agents / get_last_context_budget
  shipped as get_evidence_gates / get_agents / get_context_budget — functionally equivalent.
- Panel consolidation: planned standalone AgentsPanel.svelte / EvidenceGatesPanel.svelte /
  BudgetBar.svelte were folded inline into OrchestrationDrawer.svelte rather than shipped as
  separate files. Intentional; not an absence.

DEFERRED / NOT DONE
- None functional. The only original gap was this decisions doc (now backfilled).

NOTES
- No ELAI_*_ENABLE flags; all B-6 SSB types default-ON (no allowlist row). wiring_contract_check
  FAIL was a prose-vs-symbol false negative; every emit + command + frontend wire verified present.
