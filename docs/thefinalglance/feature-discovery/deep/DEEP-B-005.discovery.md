# DEEP-B-005 canonical feature discovery

## Scope, provenance, and notation

- Chunk: **DEEP-B-005**, family B.
- Exact manifest section processed: <code>DEEP-SOURCE-MANIFEST.md:322-335</code>. The next heading is at line 337, so no adjacent original chunk was entered.
- Orientation only: <code>02-B-PLAN-SUMMARY.discovery.md</code>, read completely so additions and corrections could be compared. It is not treated as canonical evidence.
- Canonical evidence: only the eight originals in the manifest table below, each read from line 1 through its final logical line.
- Labels are deliberately separated: **[E]** is directly explicit in a canonical source; **[I]** is a conservative cross-file inference; **[A]** is an ambiguity/open choice; **[C]** is an internal or summary-to-source conflict. Summary deltas never silently overwrite canonical wording.
- Anchors use canonical source line numbers. A range means the entire stated behavior is supported within that range.
- Atomic counts at the end count the uniquely identified **[E]** ledger records. Cross-file inferences, ambiguities, conflicts, and summary-delta rows are counted separately and do not inflate the feature inventory.

## Exact processed manifest and integrity verification

Manifest declaration: **8 files; 2,578 logical lines; 126,541 bytes; target within 1,800–2,800 lines.**

| Order | Class | Lines | Bytes | SHA-256 | Exact absolute path | Verification |
|---:|---|---:|---:|---|---|---|
| 1 | numbered/index original | 197 | 8,949 | <code>e23091d6040c90d2be6aea5a07b33ab73e258570b39d1d4742f70cd06932cdb0</code> | <code>docs/thefinalglance/B-plan/B-6_orchestration_producer_wiring.md</code> | exact |
| 2 | numbered/index original | 155 | 7,682 | <code>51950a9b3c63a8dca4532c3e2b0d0836dc32d929131762fb61842e451386e0e0</code> | <code>docs/thefinalglance/B-plan/B-17_ground_vacuous_gates.md</code> | exact |
| 3 | numbered/index original | 277 | 12,204 | <code>c838bcbc1f868ad9b0dd3692b830bc421817e5562733d2a12987de758bb181db</code> | <code>docs/thefinalglance/B-plan/B-22_phase_4G_autonomous_mode.md</code> | exact |
| 4 | numbered/index original | 241 | 11,181 | <code>57c232b1a11c42ee88c4c05c09b2161f8d614993e4080a348e860db7121b1adc</code> | <code>docs/thefinalglance/B-plan/B-26_phase_4K_6_to_13_dashboard_completion.md</code> | exact |
| 5 | numbered/index original | 211 | 8,007 | <code>7c63d9abb7018a7100c9d742f898c5e9702ec9732e605ab7e089f71d20e42c26</code> | <code>docs/thefinalglance/B-plan/B-28_focus_tunnel.md</code> | exact |
| 6 | numbered/index original | 448 | 23,173 | <code>0b2fe1a0e0a0e9d72c29b2322db266b0e6bd377f5b35456b7dd96a316dcc607b</code> | <code>docs/thefinalglance/B-plan/B-29_symbol_level_impact_api.md</code> | exact |
| 7 | numbered/index original | 379 | 16,783 | <code>94d9048b0f8cc6f918a94d44eebe7fb15ef81d75670db2508bc8beceb3280e3c</code> | <code>docs/thefinalglance/B-plan/B-32_minified_file_chunking.md</code> | exact |
| 8 | numbered/index original | 670 | 38,562 | <code>ca5a8fcbc90071443721c3c25957435b61fa79dee82fdf86783dd02e99fc95e9</code> | <code>docs/thefinalglance/B-plan/B-43_one_line_install.md</code> | exact |
| **Total** |  | **2,578** | **126,541** |  | **8 unique paths** | **exact** |

Mechanical verification used <code>sha256sum</code> and <code>wc -l -c</code> on the eight exact paths. Every individual value and both totals match the manifest; there are zero missing, extra, or duplicate source paths.

## Per-file canonical ledgers

### F01 — B-6: Producer Wiring — Orchestration Core

Canonical source: <code>B-6_orchestration_producer_wiring.md</code>, 197 lines.

#### Explicit atoms

- **B005-F01-001 [E] — objective/status/dependencies (lines 1-7):** surface all seven evidence gates, agents, supervision, BSP execution, and ContextBudget’s per-slot allocation; estimate two sessions; require B-2’s FSM emit pattern, place B-10’s profile fix nearby so budget visualization is meaningful, and allow parallel work with B-4/B-5.
- **B005-F01-002 [E] — V2 backend remap (lines 9-13):** producer work is V2-agnostic; the V1 command surface moves from <code>crates/dashboard/src/commands.rs</code> to B-36’s <code>crates/dashboard-v2/src-tauri/src/lib.rs</code>.
- **B005-F01-003 [E] — V2 drawer remap (lines 14-32):** V1 <code>AgentsPanel.svelte</code>, <code>EvidenceGatesPanel.svelte</code>, and <code>BudgetBar.svelte</code> render inside <code>OrchestrationDrawer.svelte</code>, optionally as files under <code>src/lib/orchestration/</code>; the scaffold initially consumes stub methods <code>get_agents</code>, <code>get_supervision_log</code>, <code>get_evidence_gates</code>, <code>get_context_budget</code>, and <code>get_bsp_supersteps</code>; budget work must coordinate with <code>OrbitBudgetRing.svelte</code>.
- **B005-F01-004 [E] — compact/detail UI split (lines 34-36):** <code>ActivityWindow</code> shows seven evidence-gate pips between phase and step rings, while the drawer owns the detailed view.
- **B005-F01-005 [E] — existing-gap baseline (lines 38-43):** Phase 3A InvisibleGate, Phase 3C agents/supervision, Phase 3D BSP, and Phase 3E zeroed budget slots are not visible even though orchestration phases 3A–3E are described as at least 95% implemented.
- **B005-F01-006 [E] — gate event schema (lines 57-60):** after each of seven gates evaluates, emit SSB action <code>evidence_gate_evaluated</code> with <code>{gate_name, transition_from, transition_to, verdict: Pass|Block, missing_artifacts[], suggestion}</code>.
- **B005-F01-007 [E] — gate query/surface (lines 60-62):** <code>get_recent_evidence_gates(limit)</code> feeds the last seven evaluations, green/red verdicts, and missing-artifact detail into <code>OrchestrationDrawer</code>.
- **B005-F01-008 [E] — interactive intervention lifecycle (lines 64-70):** instantiate <code>CliInteractiveGate</code> in a real loop; a needed approval writes <code>intervention_request</code> with <code>{kind: Approve|Reject|Modify|Redirect, context, options[]}</code>, the dashboard renders a card, the user choice writes <code>intervention_response</code>, and the orchestrator unblocks. B-6 requires only INTERACTIVE wiring; autonomous MONITOR coupling is left to B-26.
- **B005-F01-009 [E] — active-agent API schema (lines 72-75):** <code>get_active_agents()</code> returns records with <code>role</code>, <code>agent_id</code>, <code>status: Spawned|Running|Completed|Crashed|TimedOut</code>, <code>spawned_at</code>, <code>last_heartbeat_ms</code>, and <code>restart_count</code>.
- **B005-F01-010 [E] — agent liveness UI (lines 75-77):** show eight role rows; heartbeat is green/pulsing normally, amber after five seconds, red after fifteen seconds, and each row exposes restart count.
- **B005-F01-011 [E] — supervision activation/event (lines 79-83):** wire <code>SupervisionManager</code> into <code>task_runner</code>; emit <code>supervision_event</code> with <code>{agent_id, event: Spawn|Heartbeat|Restart|MaxRestartsExhausted|Escalate, reason}</code>; display the latest twenty events.
- **B005-F01-012 [E] — restart safety invariant (lines 79-83, 113-114):** the production wiring closes the Phase-3C blocker in which infinite restart loops are possible.
- **B005-F01-013 [E] — anti-spiral observability (lines 85-88):** emit <code>anti_spiral_hit</code> from the already-wired <code>SpiralTracker</code>; show a red agent badge when <code>count == 3</code> and an escalation pulse after escalation.
- **B005-F01-014 [E] — BSP event schema (lines 90-94):** emit <code>bsp_superstep</code> for every superstep with <code>{step_number, channel_writes, parallel_agents_spawned, parallel_agents_completed, duration_ms, straggler_count}</code>; StatusBar renders text shaped like “Superstep #42 (3 agents in flight)”.
- **B005-F01-015 [E] — budget query schema (lines 96-100):** <code>get_last_context_budget(turn_id)</code> returns eight-slot records <code>{slot_name, tokens_allocated, tokens_used, fixed}</code> for a stacked horizontal bar in StatusBar or a drawer section.
- **B005-F01-016 [E] — budget visual encoding/fallback (lines 99-102):** teal means used, dim teal means allocated-but-unused, red means overflow; a missing <code>ModelCapabilityProfile</code> displays “degraded: no profile, falling back to zeros” until B-10 fixes TaskRunner.
- **B005-F01-017 [E] — adaptive-reduction state/event (lines 104-106):** the ordered reductions are <code>FilteredByRelevance → ReducedGranularity → ReducedScope → Terminal</code>; emit <code>context_reduction</code> at each level and expose a log plus count badge.
- **B005-F01-018 [E] — overlap producer/event (lines 108-111):** implement <code>compute_overlap()</code>, apply it to SSB-entry filtering, and emit per-turn <code>ssb_filtered_by_overlap</code> with <code>{entries_total, entries_kept, overlap_threshold}</code>.
- **B005-F01-019 [E] — contract symbol registry (lines 116-132):** promised symbols are <code>EvidenceGateEvaluated</code>, <code>InterventionRequest</code>, <code>InterventionResponse</code>, <code>SupervisionEvent</code>, <code>AntiSpiralHit</code>, <code>BspSuperstep</code>, <code>ContextReduction</code>, <code>SsbFilteredByOverlap</code>, <code>AgentStatusChanged</code>, <code>get_evidence_gates</code>, <code>get_agents</code>, <code>get_supervision_log</code>, <code>get_context_budget</code>, and <code>get_bsp_supersteps</code>.
- **B005-F01-020 [E] — reachability/stub bans (lines 133-143):** consumers are <code>OrchestrationDrawer.svelte</code> and <code>StatusBar.svelte</code>; reachability is through <code>task_runner.rs::drive_task</code> and dashboard task submission; dead <code>SupervisionManager</code>, uninstantiated <code>InteractiveGate</code>, and missing overlap logic are prohibited.
- **B005-F01-021 [E] — observable smoke contract (lines 144-147):** the specified local task command must yield SSB and artifact evidence named <code>task_submitted</code>.
- **B005-F01-022 [E] — deterministic done gates (lines 149-165):** production searches must find at least one <code>SupervisionManager::new</code> caller in TaskRunner, at least one non-test <code>CliInteractiveGate::new</code>, and at least one overlap function; a smoke is also required to exercise all nine new SSB action types.
- **B005-F01-023 [E] — acceptance gates (lines 167-178):** done requires seven gate evaluations, intervention cards, eight live agent roles, bounded supervision, visible spiral/BSP/budget/reduction signals, computed 3E.5 overlap, and no permanently IDLE heartbeat display.
- **B005-F01-024 [E] — implementation footprint (lines 180-193):** modify orchestration <code>evidence.rs</code>, <code>agent_pool.rs</code>, <code>task_runner.rs</code>, <code>bsp/engine.rs</code>, <code>context_budget.rs</code>, dashboard commands/viewer, add the three V1 panel components, and create <code>workflow/reasoning/B-6-decisions.md</code>.
- **B005-F01-025 [E] — downstream (lines 195-197):** the next plans are B-7 Pipeline Factory wiring and B-8 <code>PipelineGrc</code> activation.

#### Ambiguities/conflicts

- **B005-F01-A01 [A] (lines 60, 128-132):** task prose names <code>get_recent_evidence_gates(limit)</code> and <code>get_active_agents()</code>, while the wiring registry names <code>get_evidence_gates</code> and <code>get_agents</code>; the source does not state whether these are aliases or a rename.
- **B005-F01-A02 [A] (lines 66-68):** <code>kind</code> is written as <code>Approve|Reject|Modify|Redirect</code> on the request payload even though those read like response choices; a distinct response schema is not provided.
- **B005-F01-C01 [C] (lines 145-147, 164-165):** the observable contract checks only <code>task_submitted</code>, while the deterministic smoke says all nine new action types must fire; the former cannot by itself prove the latter.
- **B005-F01-A03 [A] (lines 169-178):** all acceptance boxes remain unchecked in this canonical file; “DONE” comes from the later orientation summary/session ledger, not from a checked canonical acceptance block.

### F02 — B-17: Ground the Vacuous Gates

Canonical source: <code>B-17_ground_vacuous_gates.md</code>, 155 lines.

#### Explicit atoms

- **B005-F02-001 [E] — objective/dependency envelope (lines 1-8):** make formerly vacuous gates receive real production signals; cover <code>GroundingSweepGate</code>, the <code>SearchMode</code> cascade, and partially B-6’s <code>SupervisionManager</code>; require B-8/B-4/B-6, estimate one session, and permit parallel work with B-15/B-16/B-18/B-19/B-20.
- **B005-F02-002 [E] — V2 surface mapping (lines 10-29):** gate aggregation belongs in B-36’s V2 Tauri path and the detailed verdict UI belongs inline in <code>OrchestrationDrawer.svelte</code> or under <code>src/lib/orchestration/GatesPanel.svelte</code>; the compact ActivityWindow signal is seven pips.
- **B005-F02-003 [E] — liveness scope (lines 31-46):** verify production firing for <code>GroundingSweepGate</code>, <code>SearchMode</code> cascade, <code>SupervisionManager</code>, CSQ pre-tool enforcement, EXECUTING inline feasibility, Auditor anti-rubber-stamp, and <code>CliInteractiveGate</code>; this is a meta-step that adds regression protection and frequency visibility rather than merely defining schemas.
- **B005-F02-004 [E] — liveness harness contract (lines 56-58):** add a synthetic-flow suite in <code>gate_liveness.rs</code>; every exercised gate must fire at least once on non-vacuous input.
- **B005-F02-005 [E] — grounding fixture (lines 59-61):** an intentionally ungrounded claim must make <code>GroundingSweepGate</code> BLOCK rather than vacuously pass.
- **B005-F02-006 [E] — degradation fixture (lines 62-63):** disabling GraphCodeBERT mid-session must move <code>SearchMode</code> to <code>SparseOnly</code> and emit <code>knowledge_degradation</code>.
- **B005-F02-007 [E] — supervision fixture (lines 64-65):** an agent crashing three times must trigger <code>SupervisionManager</code> escalation.
- **B005-F02-008 [E] — CSQ fixture (line 66):** a destructive call without a guard must be blocked by a Critical CSQ.
- **B005-F02-009 [E] — feasibility fixture (line 67):** an injected infeasible inline claim must emit <code>ReplanRequest</code>.
- **B005-F02-010 [E] — anti-rubber-stamp fixture (lines 68-69):** an Auditor <code>Converge</code> without a <code>FINDING</code> must emit <code>anti_rubber_stamp_rejected</code>.
- **B005-F02-011 [E] — verdict-count panel (lines 71-79):** aggregate over the last N sessions and show GroundingSweep PASS/BLOCK, SearchMode transitions, supervision spawns/escalations, CSQ evaluated/blocked, feasibility PASS/FAIL/BLOCKED, and Auditor converges/rejections.
- **B005-F02-012 [E] — CI enforcement (lines 81-82):** run gate liveness on every pull request and fail the build if any gate becomes vacuous.
- **B005-F02-013 [E] — unresolved dismissal policy (lines 84-87):** <code>B-17-decisions.md</code> must choose whether dismissing an INTERACTIVE block logs and re-prompts or is treated as Reject.
- **B005-F02-014 [E] — performance gate (lines 89-91):** active-gate overhead must remain below 100 ms per FSM transition; if exceeded, optimize, with asynchronous <code>ground_plan</code> parallel to other PLANNING work given as an example.
- **B005-F02-015 [E] — spec lifecycle (lines 93-95):** if wiring changes verdict semantics, amend the specification rather than leaving old pass behavior documented.
- **B005-F02-016 [E] — concrete produced surfaces (lines 97-102):** the wiring contract records <code>crates/orchestrator/tests/gate_liveness.rs</code> with six active/two ignored tests, an inline <code>VERDICT COUNTS</code> block at <code>OrchestrationDrawer.svelte:418-421</code>, and <code>.github/workflows/gate-liveness.yml</code> plus <code>scripts/check_gate_liveness.sh</code>.
- **B005-F02-017 [E] — panel ownership correction (lines 100-102, 127-129):** B-17 deliberately does not create a separate <code>GatesPanel.svelte</code>; that filename belongs to B-38 ambiguity work.
- **B005-F02-018 [E] — concrete reachability (lines 103-110):** consume gates from B-2/B-4/B-6/B-8; run CI on push and pull request; dashboard reads <code>stub.get_gate_verdict_counts → commands::get_gate_verdict_counts → ssb_viewer::aggregate_gate_verdict_counts</code>; vacuous-pass-only regression is banned.
- **B005-F02-019 [E] — SSB evidence path (lines 111-115):** <code>SsbPayload::EvidenceGateEvaluated</code> is expected at <code>task_runner.rs:7103</code> for PLANNING→TESTING_FIRST and <code>task_runner.rs:10746</code> for EXECUTING→AUDITING, then aggregated with <code>GROUP BY gate_id, verdict</code>.
- **B005-F02-020 [E] — deterministic checks (lines 116-130):** run the orchestrator <code>gate_liveness</code> test with six active/two ignored, require workflow and shell runner files and execute the shell runner, and locate <code>gateVerdictCounts</code> at the declared drawer block.
- **B005-F02-021 [E] — acceptance gates (lines 132-138):** require at least one test per gate and at least six tests, all passing active tests, rendered per-gate counts, PR CI, and transition overhead below 100 ms.
- **B005-F02-022 [E] — implementation footprint (lines 140-151):** add the orchestrator test, shell and batch runners, and CI workflow; modify V2 commands/registration, <code>ssb_viewer.rs</code>, API types/stub, drawer state, and the drawer; create <code>workflow/reasoning/B-17-decisions.md</code>.
- **B005-F02-023 [E] — downstream (lines 153-155):** B-18 Tier-C sandbox activation is next.

#### Ambiguities/conflicts

- **B005-F02-C01 [C] (lines 56-58, 99-102, 140-143):** task prose first says <code>tests/contracts/gate_liveness.rs</code>; the contract/files-touched ledger instead fixes the path at <code>crates/orchestrator/tests/gate_liveness.rs</code>.
- **B005-F02-C02 [C] (lines 12-21, 100-102, 127-129):** the rebaseline allows a dedicated B-17 <code>GatesPanel.svelte</code>, but the later wiring contract explicitly bans that interpretation and assigns the file to B-38; the later, more concrete contract should be preserved as source correction.
- **B005-F02-A01 [A] (lines 33-46, 100, 119-120):** seven production concerns are listed, but the concrete suite is described as six active/two ignored and the ignored tests are not identified.
- **B005-F02-A02 [A] (lines 73, 84-87):** “last N sessions” leaves N unspecified, and the default dismissal action is intentionally unresolved.
- **B005-F02-A03 [A] (lines 132-138):** acceptance remains unchecked even though the wiring contract cites concrete line numbers and files; the canonical document does not itself declare SHIPPED/DONE.

### F03 — B-22: Phase 4G Autonomous Mode Enablement

Canonical source: <code>B-22_phase_4G_autonomous_mode.md</code>, 277 lines.

#### Explicit atoms

- **B005-F03-001 [E] — base objective (lines 1-6):** implement Phase 4G with <code>AutonomousGate</code> wrapping <code>InteractiveGate</code>, six-check <code>AutoAdvanceEvaluator</code>, ModelPool-based <code>SUBSTITUTE_MODEL</code>, Terminal-budget <code>SPLIT_TASK</code>, and a circuit breaker after more than ten auto-advances; estimate two to three sessions.
- **B005-F03-002 [E] — dependency/status envelope (lines 7-11):** prerequisites are live FSM B-2, surfaced evidence B-6, B-21 <code>FeasibilityProbe</code>, B-15–B-20 hardening, and B-36 Session 2+ for live MonitorWindow data; B-23–B-28 are declared parallel-safe.
- **B005-F03-003 [E] — V2 live-data path (lines 13-38):** orchestrator work is UI-agnostic; V2’s Tauri library replaces V1 commands, <code>stub.get_auto_advance_stats()</code> is swapped to real invoke after <code>auto_advance.rs</code>, and live stats render in <code>windows/MonitorWindow.svelte</code>.
- **B005-F03-004 [E] — model-routing UI/config path (lines 40-48):** <code>SettingsWindow.svelte</code> owns <code>--local|--api|--hybrid</code> and <code>/model</code>; <code>set_model_routing(role, target)</code> becomes a Tauri invoke; <code>get_inference_metrics</code> exposes token rate and swap history; <code>model_swap</code> writes must coordinate with B-36.
- **B005-F03-005 [E] — autonomous delegation rule (lines 52-56, 97-99):** <code>AutonomousGate</code> uses <code>DispatchPipeline</code>/<code>PlanDispatchTable</code>, checks evidence first, and delegates to a human only for HIGH risk, missing evidence, or an escalation trigger.
- **B005-F03-006 [E] — evidence set (lines 57-63, 101-102):** all six checks must run before advancement: knowledge readiness, zero <code>UNGROUNDED</code> claims, tests failing correctly, a flipped test, zero <code>BLOCK</code> findings, and DKG plus changelog evidence.
- **B005-F03-007 [E] — base escalation set (lines 64-65, 104-108):** escalation triggers are three hits on the same anti-spiral step, HIGH transition risk, <code>ReplanRequest(MAJOR)</code>, and more than ten session auto-advances.
- **B005-F03-008 [E] — split lifecycle (lines 66-67, 110-111):** Terminal ContextBudget creates at most four subtasks to depth two, couples to Phase Sizing, and merges them sequentially.
- **B005-F03-009 [E] — model substitution lifecycle (lines 68-69, 113-115):** at <code>max_restarts</code>, iterate <code>fallback_chain</code> through the <code>ModelPool</code> swap protocol and emit <code>model_swap</code>.
- **B005-F03-010 [E] — mode state (lines 70-71, 117-118):** <code>/mode autonomous|interactive</code> is per-session, not global, and takes effect at the next gate.
- **B005-F03-011 [E] — tree auto-advance policy (lines 72-80, 120-122):** advance at BabyStep granularity; risk is BabyStep LOW, Step LOW, Phase MEDIUM, Concept HIGH; Concept bubble-up always requires a human; force a human check after fifty auto baby steps per concept; represent this as <code>TreeAutoAdvancePolicy</code>.
- **B005-F03-012 [E] — base verification target (line 80):** the base Phase-4G target is 36 tests: 28 contract and 8 adversarial.
- **B005-F03-013 [E] — model-selection modes/config (lines 81-86, 124-126):** <code>--local</code> uses one sidecar model, <code>--hybrid</code> routes by role, <code>--api</code> is cloud only, <code>/model</code> swaps at runtime, and <code>elai_cc.toml [agents.{role}.backend]</code> accepts <code>local|api|auto</code>.
- **B005-F03-014 [E] — MonitorWindow telemetry (lines 128-130):** show live auto-advance count, latest escalation, circuit-breaker remaining, and advance counts per tree level.
- **B005-F03-015 [E] — produced type/API registry (lines 132-145):** produce <code>AutonomousGate</code>, <code>AutoAdvanceEvaluator</code>, <code>TreeAutoAdvancePolicy</code>, <code>SplitTaskPlan</code>, <code>SubTask</code>, <code>AutonomyDecision</code>, <code>GateDecisionPayload</code>, <code>ModelSwapPayload</code>, <code>TaskSplitPayload</code>, <code>SlashCommand::Mode</code>, and <code>SlashCommand::Model</code>.
- **B005-F03-016 [E] — reachability/safety bans (lines 146-157):** consume <code>ModelPool</code> and MonitorWindow; enter from CLI <code>run_async</code> and dashboard Tauri <code>setup</code>; prohibit unbounded advancement, HIGH-risk human bypass, and autonomous Concept bubble-up; activate the existing panel through a feature flag.
- **B005-F03-017 [E] — observable contract (lines 158-162):** running the CLI in autonomous mode must emit <code>gate_decision</code> and make MonitorWindow show a live count plus circuit-breaker gauge.
- **B005-F03-018 [E] — deterministic/acceptance gates (lines 164-191):** require a concrete <code>AutonomousGate</code>, at least 36 autonomous tests, <code>SlashCommand::Mode</code>, all six checks/four triggers, bounded split/model substitution, functional mode/model commands, model-selection UI, level risk, both breakers, MonitorWindow stats, and 36+ passing tests.
- **B005-F03-019 [E] — base implementation footprint (lines 193-202):** add <code>autonomous_gate.rs</code> and <code>auto_advance.rs</code>; modify gate, TaskRunner, CLI commands, agent-backend config, dashboard commands/settings/MonitorWindow; create <code>B-22-decisions.md</code>.
- **B005-F03-020 [E] — base shipped/follow-up boundary (lines 204-210):** B-22 shipped in session <code>s-26716</code> on 2026-05-09 with the lone three-identical-GRC-step rule; B-22b/4G.9 is explicitly not original B-22 scope and remains work to do.
- **B005-F03-021 [E] — Spinning detector (lines 212-218):** <code>Spinning</code> fires on at least three identical <code>phase_outputs</code> or <code>error_signatures</code>, compared by blake3; it subsumes the old same-step rule.
- **B005-F03-022 [E] — Oscillation detector (lines 212-218):** <code>Oscillation</code> recognizes period-two alternation and requires at least four outputs.
- **B005-F03-023 [E] — NoDrift detector (lines 212-218):** <code>NoDrift</code> fires when absolute drift delta is below 0.01 for three iterations.
- **B005-F03-024 [E] — DiminishingReturns detector (lines 212-218):** <code>DiminishingReturns</code> requires improvements below 0.01 that are also monotonically decreasing.
- **B005-F03-025 [E] — detector purity (lines 217-218):** <code>StagnationDetector</code> is stateless over <code>ExecutionHistory</code>.
- **B005-F03-026 [E] — persona affinity (lines 220-225):** Hacker maps to Spinning; Researcher to NoDrift/DiminishingReturns; Simplifier to DiminishingReturns/Oscillation; Architect to Oscillation/NoDrift; Contrarian to all patterns.
- **B005-F03-027 [E] — persona selection/exhaustion (lines 226-228):** <code>suggest_persona_for_pattern(pattern, exclude)</code> returns the first priority match; exhaustion emits <code>lateral_thinking_exhausted</code> and follows existing <code>EscalationPolicy</code>.
- **B005-F03-028 [E] — optional fan-out (lines 230-232):** an optional flag may spawn all five personas in parallel through Tier C, each with fresh context to avoid anchoring; first success wins.
- **B005-F03-029 [E] — watchdog timers (lines 234-238):** per BabyStep <code>GenerationWatchdog</code> defaults are 60 seconds with no SSB event, 180 seconds with events but no material progress, and a 600-second wall-clock safety cap.
- **B005-F03-030 [E] — watchdog materiality/dedup (lines 239-243):** material events are <code>phase_transition</code>, <code>evidence_gate_evaluated</code>, <code>integration_checkpoint_success</code>, <code>bubble_up_typed</code>, and state-changing <code>agent_lifecycle</code>; heartbeats/status echoes do not count; <code>workflow_fingerprint</code> dedup blocks replay gaming.
- **B005-F03-031 [E] — persona artifact set (lines 245-248):** add <code>persona-hacker.md</code>, <code>persona-researcher.md</code>, <code>persona-simplifier.md</code>, <code>persona-architect.md</code>, and <code>persona-contrarian.md</code> under <code>.claude/agents/</code>, following existing agent-file format.
- **B005-F03-032 [E] — goal-drift calculation (lines 250-253):** calculate <code>drift = 1 - jaccard(tokens(output), tokens(baby_step.goal))</code> using lowercase alphanumeric tokens split on whitespace.
- **B005-F03-033 [E] — goal-drift verdict/recovery (lines 254-256):** drift below 0.5 passes; 0.5–0.7 emits an advisory SSB warning and continues to audit; at least 0.7 blocks before audit and causes <code>BubbleUp RetryBabyStep</code> with at most one retry.
- **B005-F03-034 [E] — goal-drift skip/economy (lines 257-259):** skip if output has fewer than twenty tokens or goal is empty; every Block is intended to save at least one audit model call and must be measured in 4G.6 metrics.
- **B005-F03-035 [E] — goal-drift non-goals/provenance (lines 260-270):** do not port <code>constraint_drift</code> or <code>ontology_drift</code> because hooks and OSC cover them; port only the stated MIT-licensed ouroboros stagnation, lateral, watchdog, goal-drift, and multi-persona patterns.
- **B005-F03-036 [E] — follow-up effort/downstream (lines 272-277):** B-22b is about two sessions—one for patterns/watchdog and one for fan-out/persona files/canary—then B-23 self-modification follows.

#### Ambiguities/conflicts

- **B005-F03-C01 [C] (lines 80, 171-173 versus orientation summary B-22):** the canonical plan requires 36 tests (28 contract + 8 adversarial), whereas the summary’s “eighteen extended contracts” is a different and unexplained count.
- **B005-F03-C02 [C] (lines 114, 158-162 versus orientation event registry):** canonical B-22 explicitly names <code>model_swap</code> and expects <code>gate_decision</code>; the summary instead registers <code>auto_advance_decision</code>, <code>model_swap</code>, <code>autonomy_escalation</code>, and <code>split_task_initiated</code>. The canonical file does not reconcile <code>gate_decision</code> with <code>auto_advance_decision</code>.
- **B005-F03-A01 [A] (lines 57, 93, 101-102):** the plan calls these “six evidence checks (one per gate)” while also pointing at seven <code>EvidenceGate</code> implementations; the omitted seventh mapping is not identified.
- **B005-F03-A02 [A] (lines 7-11, 204-210):** prerequisites include large unshipped ranges B-15–B-21, yet the base is later declared shipped; the canonical file does not explain whether those prerequisites were waived or satisfied elsewhere.
- **B005-F03-A03 [A] (lines 156-157):** the feature flag used to activate the already-built AutoAdvanceStats panel is not named.
- **B005-F03-A04 [A] (lines 179-191, 204-210):** base acceptance boxes remain unchecked despite the later explicit “B-22 shipped” statement; B-22b remains clearly unshipped.

### F04 — B-26: Phase 4K.6–4K.13 Dashboard Completion

Canonical source: <code>B-26_phase_4K_6_to_13_dashboard_completion.md</code>, 241 lines.

#### Explicit atoms

- **B005-F04-001 [E] — narrowed objective/status (lines 1-9):** V2 already ships Monitor/Explore/Review, notifications, and tabs; B-26 is a one-session final pass limited to six polish/intervention/shortcut gaps rather than rebuilding windows.
- **B005-F04-002 [E] — dependency/fallback envelope (lines 11-19):** B-22, B-7, and B-11–B-14 are stated shipped; B-36 supplies real invokes but does not block B-26 because <code>tryInvoke</code> plus realistic mocks already works; B-25/B-27/B-28 are parallel-safe.
- **B005-F04-003 [E] — rebaseline history (lines 21-29):** the 2026-05-07 pass changed scope to polish/lifecycle/animations; the 2026-05-11 pass records most earlier work shipped, deletes obsolete V1 tasks/checks, and leaves six gaps.
- **B005-F04-004 [E] — local-only layout persistence decision (lines 30-35):** drop <code>WINDOW_STATE</code> SSB; persist at localStorage key <code>elai.dashboard-v2.workspace.v1</code>; do not backlog cross-device sync; do not add <code>WINDOW_STATE</code> or <code>HEALTH_SNAPSHOT</code> to <code>ssb.rs</code>.
- **B005-F04-005 [E] — intervention-card ownership decision (lines 36-41):** keep the single-consumer orbit card inline at <code>MonitorWindow.svelte:376-409</code>; reconsider extraction only after a second consumer such as CommandWindow takeover or a MONITOR overlay appears.
- **B005-F04-006 [E] — shipped MonitorWindow behavior (lines 43-50):** five-second polling covers FSM/system/supervision/auto-advance/cloud-spend; inline cards offer Approve/Reject/Take over; the window includes progress and phase dots, supervision, MSI/Proposals, thinking pulse, 100 ms step flash, 200 ms fail border, three-second phase breath, and card glow.
- **B005-F04-007 [E] — shipped ExploreWindow behavior (lines 51-53):** <code>run_explore_query</code> drives Q&A streaming with file references, code blocks, citations, and a 2.2-second pending breath.
- **B005-F04-008 [E] — shipped ReviewWindow behavior (lines 54-56):** per-file tabs, sticky file header, per-hunk hover Accept/Decline, retry note, and file-level Accept All/Decline All/Retry already exist.
- **B005-F04-009 [E] — shipped notification behavior (lines 57-59):** <code>NotificationManager.svelte</code> uses the Browser Notification API, localStorage permission preference, FSM state-change triggers, a consent banner, and is mounted in Workspace.
- **B005-F04-010 [E] — shipped intervention schema/API (lines 60-66):** <code>InterventionRequest</code>/<code>InterventionResponse</code> exist in orchestrator SSB/viewer; <code>submit_intervention_response</code>, <code>set_webhook_url</code>, and <code>get_webhook_url</code> are wired with frontend wrappers.
- **B005-F04-011 [E] — shipped tab lifecycle (lines 67-70):** <code>openTab</code>, <code>closeTab</code>, and <code>moveTab</code> support cross-panel drag and schema-v1-validated localStorage persistence.
- **B005-F04-012 [E] — shipped shortcut set (lines 71-74):** existing shortcuts are Ctrl+1–7, Ctrl+N, Ctrl+W, Ctrl+D, Ctrl+., Ctrl+,, Ctrl+Shift+P, and Esc for sidebar navigation, tab/window actions, palettes/settings, and dismiss.
- **B005-F04-013 [E] — shipped recovery boundary (lines 75-77):** client-side session/workspace recovery uses the same localStorage key and intentionally has no SSB persistence.
- **B005-F04-014 [E] — webhook trigger set (lines 81-89):** new <code>notifications/webhook.ts</code> POSTs a Discord/Slack-shaped body when FSM enters <code>INTERVENTION_REQUIRED</code>, <code>AUDITING</code>, or post-task <code>IDLE</code>, or pending interventions becomes non-empty; a supervision <code>BLOCK</code> trigger is optional and a graceful no-op if unavailable.
- **B005-F04-015 [E] — webhook concurrency/failure behavior (lines 90-92):** invoke the webhook in parallel with <code>Notification.fire()</code>, gate both on user preference, and log/swallow webhook errors so a bad URL cannot break UI.
- **B005-F04-016 [E] — webhook configuration behavior (lines 94-98):** add a one-line Settings input using the existing get/set commands; persist on blur, accept only <code>http(s)://</code>, and show saved/invalid inline status.
- **B005-F04-017 [E] — Review shortcut isolation (lines 100-104):** Ctrl+Shift+A accepts all and Ctrl+Shift+R rejects all only when the active primary window is <code>review</code>; <code>shortcuts.ts</code> remains the single keydown authority.
- **B005-F04-018 [E] — misleading recovery shortcut resolution (lines 105-111):** default to removing Ctrl+Shift+H; a recently-closed ring buffer/reopen implementation is an explicit deferred alternative.
- **B005-F04-019 [E] — real plan progression (lines 113-119):** replace the static five-phase FSM mapping with plan-level SSB progress, either by extending <code>FsmSnapshot</code> or adding <code>get_plan_progress</code> plus a stub wrapper.
- **B005-F04-020 [E] — animation behavior (lines 121-126):** tab switches use 80 ms fade-out then 120 ms fade-in; verify existing two-second thinking pulse, 100 ms completion flash, 200 ms fail border, and three-second progress-dot timing and patch drift only.
- **B005-F04-021 [E] — responsive states (lines 128-131):** below 1000 px collapse Sidebar into a hamburger; below 800 px force <code>workspace.secondaryCollapsed = true</code> and hide <code>PanelDivider</code>; verify by resizing under <code>tauri:dev</code>.
- **B005-F04-022 [E] — produced/consumed registry (lines 133-149):** produce webhook module, guarded Review shortcuts, webhook input, and one of two phase-progress paths; consume existing webhook APIs, notification preference, <code>pending_interventions</code>, and <code>workspace.primary.activeTabId</code>; Oracle RESEARCHING is already consumed by ExploreWindow and adds no dependency.
- **B005-F04-023 [E] — frontend-only reachability/stub ban (lines 151-158):** Workspace-mounted NotificationManager makes webhook delivery a transparent side effect; webhook code must perform real <code>fetch</code>, not a log-only stub; B-26 has no Rust frontend pair.
- **B005-F04-024 [E] — observable contract (lines 160-168):** after configuring a URL and triggering intervention, the endpoint receives the payload while Browser Notification fires; Review bulk shortcuts work only in Review and are no-ops or yield to another shortcut elsewhere.
- **B005-F04-025 [E] — deterministic prohibitions (lines 170-202):** verify webhook import, Settings binding, both guarded shortcuts, removal or real backing of recovery shortcut, a non-static phase path, and absence of <code>WindowState|HealthSnapshot</code> in SSB.
- **B005-F04-026 [E] — acceptance gates (lines 204-214):** local-listener webhook verification, validated persistence, Review-only shortcuts, removal of Ctrl+Shift+H, real phase progress, both responsive breakpoints, and 80/120 ms fades are all required.
- **B005-F04-027 [E] — implementation footprint (lines 216-237):** add webhook module; modify NotificationManager, SettingsWindow, MonitorWindow, shortcut state, Sidebar, Workspace, and Panel; only maybe modify Rust/Tauri for a new command; create <code>B-26-decisions.md</code>.
- **B005-F04-028 [E] — downstream (lines 239-241):** B-27 Cloud Mode and B-28 Focus Tunnel are next and parallel-safe.

#### Ambiguities/conflicts

- **B005-F04-A01 [A] (lines 113-119, 234-236):** the phase-progress transport remains a real either/or choice: extend <code>FsmSnapshot</code> or add <code>get_plan_progress</code>.
- **B005-F04-A02 [A] (lines 88-89):** supervision BLOCK webhook behavior is optional and has no decision gate in this file.
- **B005-F04-A03 [A] (lines 30-41, 113-119):** “T4” is reused: the decision log’s T4 is InterventionCard extraction, while remaining-task T4 is phase dots.
- **B005-F04-A04 [A] (lines 204-214):** the six-gap acceptance list remains unchecked, so canonical status is “majority already shipped, six tasks remain,” not an unqualified DONE.

### F05 — B-28: Focus Tunnel

Canonical source: <code>B-28_focus_tunnel.md</code>, 211 lines.

#### Explicit atoms

- **B005-F05-001 [E] — objective/dependencies (lines 1-9):** implement Focus Tunnel E-1–E-7—FIM, diff, deterministic mode selection, streaming validation, depth-one preassembly, dependency resolution, and file routing—in three to four sessions; require B-15–B-20 and B-22 for full benefit; allow B-25/B-26/B-27 in parallel.
- **B005-F05-002 [E] — FIM mode (lines 16-22):** for a single-region edit in an existing file on native Qwen2.5-Coder, output raw code between prefix/suffix without tool schemas or JSON, saving about 350 tokens per step.
- **B005-F05-003 [E] — DIFF mode (lines 20-22):** multi-region edits use unified diffs applied through B-20’s three-tier matching and are estimated 30–50% smaller than full-file output.
- **B005-F05-004 [E] — STRUCTURED mode (line 22):** STRUCTURED remains the default and the fallback for new files and cloud.
- **B005-F05-005 [E] — deterministic selector (lines 24-29):** <code>edit_existing + single region + supports_fim</code> selects FIM; <code>edit_existing + multi-region</code> selects DIFF; <code>create_file</code> and every other case select STRUCTURED; no LLM participates.
- **B005-F05-006 [E] — token validation (lines 31-36):** check imports against <code>deprecated_patterns</code> and detect lazy code such as <code>todo!</code>/<code>pass</code>; terminate bad output early.
- **B005-F05-007 [E] — chunk validation (lines 31-36):** run a partial tree-sitter parse about every fifty tokens and terminate on structural impossibility; catching a bad generation at token 50 instead of 500 is estimated to save about 90%.
- **B005-F05-008 [E] — pipeline preassembly (lines 38-41):** while step N validates, preassemble context for N+1; depth is one; expected savings are 100–500 ms on the roughly 88% of steps that validate.
- **B005-F05-009 [E] — plan-owned routing (lines 43-45):** <code>target_file</code> comes from the plan, never the model; an AST mismatch creates a file microstep and retries.
- **B005-F05-010 [E] — dependency safety baseline (lines 47-52):** infer missing dependencies from imports, check anti-staleness, resolve a version and install it; the model may never run <code>cargo add</code> or <code>pip install</code>; autonomous mode uses a pre-approved allowlist.
- **B005-F05-011 [E] — claimed budget impact (lines 54-55):** estimate 6,800 tokens/27% saved on a thirty-step 25K-token task and a 45–65% effective budget increase when combined with ShortCoder.
- **B005-F05-012 [E] — FIM capabilities/prompts (lines 65-72):** add <code>ModelCapabilities.supports_fim</code>, true for Qwen2.5-Coder, StarCoder, and DeepSeek-Coder and false for cloud; <code>FimPromptAssembler</code> prefix contains header/imports/referenced symbols/pre-edit code and suffix contains post-edit code plus fifty lines or EOF; insert output at the edit region and run PostToolUse hooks; unsupported models fall back to STRUCTURED.
- **B005-F05-013 [E] — diff parser/recovery (lines 74-79):** parse hunks with file/line/added/removed data, apply three-tier matching, support multiple regions, and fall back to STRUCTURED on parse failure.
- **B005-F05-014 [E] — per-step preference/telemetry (lines 81-85):** <code>PipelineSpec</code> steps gain <code>preferred_mode: Option&lt;GenerationMode&gt;</code>; SSB records <code>mode_distribution</code> per task.
- **B005-F05-015 [E] — streaming telemetry (lines 87-92):** expose <code>early_termination_rate</code> and <code>tokens_saved</code>.
- **B005-F05-016 [E] — preassembly discard/adaptation (lines 94-98):** discard N+1 context if N fails because it depends on N’s output; start at depth one and disable if validation pass rate drops below 80%.
- **B005-F05-017 [E] — dependency-resolution pipeline (lines 100-107):** identify import → check anti-staleness → query registry → conflict check → edit manifest → install; interactive new dependencies require confirmation, while pre-approved autonomous dependencies and their transitives are auto-approved.
- **B005-F05-018 [E] — speculative microstep recovery (lines 109-118):** <code>target_file + edit_region</code> drive FIM/DIFF setup and write destination; undefined-symbol AST evidence produces a MINOR <code>ReplanRequest</code> “create {path} with {type}” in STRUCTURED mode, then retries original work with updated context; cap at two microsteps per BabyStep.
- **B005-F05-019 [E] — dashboard telemetry (lines 120-125):** show per-task FIM/DIFF/STRUCTURED percentages, early-termination-rate gauge, preassembly depth/waste rate, and auto-installed dependency version/source log.
- **B005-F05-020 [E] — symbol/event registry (lines 127-140):** produce <code>FimPromptAssembler</code>, <code>DiffParser</code>, <code>ModeSelector</code>, <code>StreamingValidator</code>, <code>PipelinePreAssembler</code>, <code>DependencyResolver</code>, <code>SpeculativeFileRouter</code>, <code>GenerationModeSelected</code>, <code>StreamingEarlyTerminated</code>, <code>PipelinePreAssembled</code>, and <code>DependencyResolved</code>.
- **B005-F05-021 [E] — consumed systems/reachability (lines 141-146):** depend on sidecar FIM tokens, B-20 edit matching, and 3F deprecated patterns; execute from <code>task_runner.rs::drive_task</code> for every BabyStep generation.
- **B005-F05-022 [E] — hard bans (lines 147-150):** disallow direct model package-manager execution, preassembly depth above one, and unconfirmed dependencies in INTERACTIVE mode.
- **B005-F05-023 [E] — observable contract (lines 151-156):** dashboard is the frontend pair; the <code>b28_focus_tunnel_smoke_ssb</code> contract must emit <code>generation_mode_selected</code> and show mode distribution plus preassembly stats.
- **B005-F05-024 [E] — deterministic/acceptance gates (lines 158-179):** locate FIM support, selector, and dependency resolver; require all seven E deliverables, working FIM/DIFF, deterministic selection, early termination, measurable 100–500 ms parallelism saving, allowlisted dependency automation, file microsteps, and dashboard panels.
- **B005-F05-025 [E] — implementation footprint (lines 181-192):** add seven <code>focus_tunnel</code> modules and <code>FocusTunnelPanel.svelte</code>; modify providers/tools/TaskRunner/SSB/V2 Tauri; create <code>B-28-decisions.md</code>.
- **B005-F05-026 [E] — Part-IV completion gate (lines 194-203):** after B-21–B-28, the plan claims Phase 4F–4J/4K post-MVP, cloud, Focus Tunnel, MSI self-modification, conditional Empire integration, and the architecture overview complete enough for v1.0.
- **B005-F05-027 [E] — deferred/downstream boundary (lines 205-211):** this was the intended end of the original B-plan; Phase 5 Empire deep work and compiled pipelines are deferred, Phase 3L Project Intelligence is recommended after B-1–B-20, and spec amendments are split across B-15/B-19/B-20.

#### Ambiguities/conflicts

- **B005-F05-A01 [A] (lines 24-29, 83-85):** the selector is canonical and deterministic, but the precedence/validation semantics of optional <code>preferred_mode</code> are not stated.
- **B005-F05-A02 [A] (lines 38-41, 96-98):** “depth 1 only” is firm; “adaptive depth” actually means on/off at depth one, not permission to increase depth.
- **B005-F05-A03 [A] (lines 54-55, 169-179):** token/time savings are estimates and acceptance targets, not attached benchmark evidence.
- **B005-F05-A04 [A] (lines 169-179):** every acceptance box is unchecked and the orientation snapshot labels B-28 PLANNED.

### F06 — B-29: Symbol-Level Impact Analysis API

Canonical source: <code>B-29_symbol_level_impact_api.md</code>, 448 lines.

#### Explicit atoms

- **B005-F06-001 [E] — objective/actors (lines 1-10):** add agent-callable blast radius, call flow, symbol context, and symbol list over a sharded/LRU-bounded <code>SymbolGraphCache</code> so Auditor, Designer, and Researcher can reason without loading the whole DKG; estimate 12–18 hours, require B-1, recommend B-34, and allow B-30/B-31/B-32/B-35 in parallel.
- **B005-F06-002 [E] — existing graph baseline (lines 12-26):** A-44 already persists a DKG at <code>.elai_cc/.elai_cc/indexes/dkg.db</code> with 8,075 nodes/4,148 edges; A-48 adds PARAMETER, RETURN, ASSIGNMENT, and FIELD_ACCESS data-flow edges; the missing piece is production traversal/query exposure.
- **B005-F06-003 [E] — bounded-cache invariant/provenance (lines 28-43):** port SocratiCode’s algorithmic pattern—not AGPL source—into Rust over ELAI SQLite; use 27 name buckets, 256 reverse buckets, and a 500-file payload LRU; no query may load every symbol or edge.
- **B005-F06-004 [E] — consuming workflows (lines 45-49):** Auditor automatically checks impact before Write/Edit review, Designer uses call flow in recursive decomposition, Researcher’s <code>/research</code> uses symbol context, and KnowledgeDrawer renders hop bands.
- **B005-F06-005 [E] — research design anchor (lines 51-65):** FocalAD motivates top-k local, hop-banded attention instead of global caller dumps; the plan maps local influence to its confidence tiers.
- **B005-F06-006 [E] — explicit future non-goal (lines 67-71):** do not add confidence-weighted fusion in B-29; ship uniform weighting within a hop band and leave adaptive weighting to B-23.
- **B005-F06-007 [E] — <code>symbol_name_index</code> schema (lines 100-107):** primary key <code>(shard_key, symbol_name)</code>; store <code>shard_key</code>, <code>symbol_name</code>, and MessagePack <code>Vec&lt;SymbolRef {file,id}&gt;</code>.
- **B005-F06-008 [E] — <code>symbol_reverse_index</code> schema (lines 108-113):** primary key <code>(shard_bucket, callee_file)</code>; bucket is 0–255 from the first SHA1 byte of <code>callee_file</code>; payload is MessagePack caller-file strings.
- **B005-F06-009 [E] — <code>symbol_file_payload</code> schema (lines 114-120):** file is primary key; persist language, blake3 content hash, MessagePack symbols, and MessagePack edges.
- **B005-F06-010 [E] — <code>symbol_graph_meta</code> schema (lines 121-131):** key by project ID and store symbol/edge/file counts, unresolved-edge percentage, injected-clock <code>built_at</code>, and schema version; migration is v3 and adds no SQLite dependency.
- **B005-F06-011 [E] — symbol domain types (lines 133-140):** <code>SymbolKind</code> values are Function, Method, Class, Struct, Trait, Module, Enum, Interface, Type; <code>SymbolNode</code>, <code>SymbolEdge</code>, <code>SymbolRef</code>, and <code>EntryPoint</code> have the exact fields stated, and all serialize through serde/MessagePack.
- **B005-F06-012 [E] — confidence model (lines 136-138):** <code>Confidence</code> is exactly <code>Local|Unique|MultipleCandidates|Unresolved</code>; each edge also carries candidate IDs and a <code>CallSite</code>.
- **B005-F06-013 [E] — sharding functions (lines 142-145):** <code>name_shard_key</code> returns the first lowercase ASCII letter or underscore; <code>reverse_shard_bucket</code> returns first SHA1 byte of file; <code>all_name_shard_keys()</code> returns all 27 keys.
- **B005-F06-014 [E] — extraction contract (lines 147-155):** <code>extract_symbols_and_calls(source, language, file)</code> uses existing tree-sitter grammars, not ast-grep; raw calls contain caller ID, callee name, and call site; every file gets a synthetic <code>&lt;module&gt;</code> caller for unattributed calls.
- **B005-F06-015 [E] — resolver tiers (lines 156-165):** resolution searches same-file Local, one-hop Imported, then two-hop Re-export dependencies; zero candidates yields Unresolved, one Unique, and multiple MultipleCandidates; mutate <code>outgoing_calls_by_file</code> in place.
- **B005-F06-016 [E] — eager metadata/lazy indexes (lines 166-177):** <code>SymbolGraphCache</code> eagerly holds tiny metadata; first name query loads 27 shards in parallel through <code>tokio::JoinSet</code>; first impact query loads 256 reverse shards in parallel; file payloads are lazy.
- **B005-F06-017 [E] — LRU implementation/capacity (lines 176-181):** cache at most 500 <code>Arc&lt;SymbolGraphFilePayload&gt;</code> entries; implement a tiny LRU with existing/standard facilities and obtain explicit confirmation before any new dependency; constructor accepts project ID, SQLite pool, and configurable capacity.
- **B005-F06-018 [E] — impact traversal API (lines 183-192):** <code>compute_impact_radius(cache,target,depth)</code> recognizes file path versus symbol name, BFSes the reverse-file index, returns deterministic <code>BTreeMap&lt;u32,Vec&lt;String&gt;&gt;</code> hop bands plus <code>truncated</code>, and clamps depth to ten.
- **B005-F06-019 [E] — call-flow API (lines 193-198):** <code>compute_call_flow</code> performs cycle-aware DFS, loads child payloads only while descending, clamps depth to ten, and annotates leaves with <code>TruncationReason::Depth|Cycle</code>.
- **B005-F06-020 [E] — symbol-context API (lines 199-202):** <code>compute_symbol_context(cache,name,file_hint)</code> returns every matching definition, not only the first, with callers/callees and confidence; file hint disambiguates.
- **B005-F06-021 [E] — symbol-list API (lines 203-204):** <code>list_symbols</code> filters by optional file, optional query, and required limit.
- **B005-F06-022 [E] — invalidation boundary (lines 206-210):** <code>invalidate_file(file)</code> evicts its payload and marks name/reverse indexes for partial rebuild; more than fifty changed files forces a full rebuild.
- **B005-F06-023 [E] — full build lifecycle (lines 212-223):** after DKG build, extract each file, resolve against the import graph, build deterministic shard maps, persist all four table families as MessagePack, and log files/symbols/edges/unresolved percentage.
- **B005-F06-024 [E] — incremental build lifecycle (lines 225-229):** when changed count is at most <code>INCREMENTAL_SYMBOL_THRESHOLD</code>, patch changed/removed payloads and only affected name/reverse buckets; otherwise rebuild fully.
- **B005-F06-025 [E] — dispatcher schemas (lines 231-243):** register <code>symbol_impact {target,depth?}</code>, <code>symbol_flow {entrypoint?,file?,depth?}</code>, <code>symbol_context {name,file?}</code>, and <code>symbol_list {file?,query?,limit?}</code>; flow without an entrypoint lists B-34 candidates if available, otherwise errors.
- **B005-F06-026 [E] — Auditor auto-injection (lines 245-250):** in PostToolUse Write/Edit, for every changed file call impact at depth two; if any files are impacted, append <code>IMPACT_SUMMARY</code> to the Auditor briefing.
- **B005-F06-027 [E] — query event schema (lines 252-254):** append <code>symbol_query</code> per query with <code>{tool,target,hop_count,total_files,latency_ms}</code> for dashboard activity.
- **B005-F06-028 [E] — dashboard impact panel (lines 256-260):** KnowledgeDrawer shows unresolved-edge percentage, last target/depth/total/hop table, and a <code>/symbol_impact</code> input routed through synthetic <code>submit_task</code> intent.
- **B005-F06-029 [E] — produced symbol registry (lines 262-278):** export cache/traversal/resolution/extraction/sharding functions, <code>SsbActionType::SymbolQuery</code>, and IPC constants <code>TOOL_SYMBOL_IMPACT</code>, <code>TOOL_SYMBOL_FLOW</code>, <code>TOOL_SYMBOL_CONTEXT</code>, <code>TOOL_SYMBOL_LIST</code>.
- **B005-F06-030 [E] — integration registry (lines 279-293):** full/update indexing call graph builds; TaskRunner PostToolUse performs depth-two impact; dashboard <code>get_impact_analysis</code> reads meta/hops; CLI research uses symbol context; paths are reachable from CLI, dashboard, and TaskRunner.
- **B005-F06-031 [E] — cache/data stub bans (lines 294-300):** first impact query must really load reverse index when metadata reports edges; LRU capacity must be 500; ELAI’s unresolved-edge percentage must be below 50%, with below 30% ideal but not mandatory.
- **B005-F06-032 [E] — frontend dependency (lines 301-302):** B-33 is the frontend pair and consumes the same cache for symbol-mode visualization.
- **B005-F06-033 [E] — observable contract (lines 303-307):** the four-tool dispatch smoke writes one <code>symbol_query</code> row per tool and stdout must contain a real “Hop 1” block with ELAI paths.
- **B005-F06-034 [E] — database acceptance bounds (lines 309-325):** all four tables must exist; ELAI meta requires at least 5,000 symbols, 8,000 edges, and under 50% unresolved; reverse index must contain a known multi-caller clock entry.
- **B005-F06-035 [E] — CLI behavioral checks (lines 326-341):** impact on <code>ClockProvider</code> should return depth three and at least twenty files; flow from CLI <code>main</code> should show a real call tree; an edit rename task must yield an Auditor <code>IMPACT_SUMMARY</code> with at least five dependent files.
- **B005-F06-036 [E] — boundedness check (lines 343-345):** after one thousand payload accesses, LRU size remains at most 500, covered by <code>test_lru_capacity_bounded</code>.
- **B005-F06-037 [E] — acceptance gates (lines 348-366):** property-test depth/cycles/target polymorphism, persist 27/256 shards, make v3 idempotent, integrate full/incremental indexing, expose all four tools through CLI/dashboard/IPC, inject impact, emit latency telemetry, render real UI, use no non-test unwrap, and add no unconfirmed dependency.
- **B005-F06-038 [E] — extraction/resolution tests (lines 368-375):** fixtures cover Rust/Python/TS/Go/JVM counts/names and Local/Imported/Re-export call sites with asserted confidence.
- **B005-F06-039 [E] — cache/traversal tests (lines 376-383):** test capacity, parallel shard loading, payload eviction/index preservation, depth bounds, cycle termination, all-match context, and list limits.
- **B005-F06-040 [E] — end-to-end/contracts (lines 384-387):** add IPC <code>symbol_query_dispatch.rs</code> and <code>tests/contracts/symbol_graph_contract.rs</code> for meta/schema sanity.
- **B005-F06-041 [E] — implementation footprint (lines 389-412):** create the complete <code>knowledge::symbol_graph</code> module set, store and migration; modify knowledge export, indexing, dispatcher, TaskRunner, SSB action types, V2 Tauri, KnowledgeDrawer; create <code>B-29-decisions.md</code>.
- **B005-F06-042 [E] — required decisions (lines 414-427):** document SQLite MessagePack shards, std-only LRU, in-place confidence mutation, always-on versus feature-gated <code>IMPACT_SUMMARY</code> (recommend always-on with env opt-out), default/user depth, and FocalAD as the design anchor.
- **B005-F06-043 [E] — unlocked capabilities (lines 429-442):** provide evidence-based edit scope, existing-project call-flow grounding, authoritative Oracle caller answers, B-23 blast-radius signals, and zero-extra-load B-33/B-34 reuse.
- **B005-F06-044 [E] — downstream (lines 444-448):** B-30 is independent; B-33 waits for the cache; B-34 waits for file payloads.

#### Ambiguities/conflicts

- **B005-F06-C01 [C] (lines 37-39, 62-63, 136-138, 159-163 versus orientation summary):** canonical resolver **tiers** are Local/Imported/Re-export, but canonical <code>Confidence</code> values are Local/Unique/MultipleCandidates/Unresolved. The summary incorrectly presents Imported and Re-export as confidence values.
- **B005-F06-C02 [C] (lines 209-210, 225-229, 357-358 versus orientation summary):** canonical incremental behavior includes exactly fifty changed files (<code>&lt;= 50</code>); the summary says “fewer than fifty,” incorrectly excluding 50.
- **B005-F06-A01 [A] (lines 178-181, 419-423):** implementation prose allows <code>linked_hash_map</code> or std structures, while the decision recommendation says std-only and acceptance prohibits unconfirmed dependencies; whether <code>linked_hash_map</code> is already a workspace dependency must be checked.
- **B005-F06-C03 [C] (lines 45-49, 245-250):** the plan repeatedly calls the Auditor operation a “pre-edit” check but wires it in <code>PostToolUse</code> after Write/Edit and after <code>changed_files</code> exists.
- **B005-F06-A02 [A] (lines 14-16):** the double-prefixed path <code>.elai_cc/.elai_cc/indexes/dkg.db</code> is canonical text here but looks inconsistent with B-1 path unification; no correction is provided in this source.
- **B005-F06-A03 [A] (lines 108-109, 117, 142-145):** SHA1 is explicitly mandated for reverse bucket placement while blake3 is used for content staleness; this is an intentional source-specific exception/tension with the summary-wide “blake3 for non-RFC hashing” rule.
- **B005-F06-A04 [A] (lines 185-192):** file-versus-symbol target detection is named, but how a symbol target is converted to reverse-file BFS seeds is not specified.
- **B005-F06-A05 [A] (lines 252-254, 274, 405):** prose says add the action to <code>ssb.rs</code>; the footprint locates it in <code>crates/core/src/ssb/action_types.rs</code>.
- **B005-F06-A06 [A] (lines 298-300):** below 50% is the hard sanity gate and below 30% is only ideal; neither defines language-specific allowances.
- **B005-F06-A07 [A] (lines 422-424):** “Default depth values (B-2 in Auditor pre-check)” appears to mean depth two, but the text is typographically ambiguous.
- **B005-F06-A08 [A] (lines 348-366):** acceptance is entirely unchecked and the orientation summary labels B-29 PLANNED/highest leverage.

### F07 — B-32: Adaptive/Minified-File Chunking

Canonical source: <code>B-32_minified_file_chunking.md</code>, 379 lines.

#### Explicit atoms

- **B005-F07-001 [E] — objective/dependencies (lines 1-10):** prevent minified, vendored, packed, and generated one-line files from exceeding embedding context or creating useless BM25 chunks by adding deterministic detection and a universal cap; estimate three to five hours, require B-1, bundle naturally with B-31, and allow B-29/B-30/B-31/B-33/B-34/B-35 in parallel.
- **B005-F07-002 [E] — baseline quality (lines 12-18):** existing Phase-3K tree-sitter chunking uses function/impl/struct/class regions with fifty-line fallback and reports R@5 0.6333 and R@10 0.7667 on thirty queries.
- **B005-F07-003 [E] — failure chain (lines 20-36):** valid one-line JS/CSS/generated code parses as one giant AST region; 50K+ payloads exceed provider context or become noisy, while BM25 accepts semantically poor chunks; current cap occurs only at the provider boundary.
- **B005-F07-004 [E] — pre-strategy detector (lines 38-44):** compute average content length per line; a strict value above 500 routes immediately to character chunking and skips AST.
- **B005-F07-005 [E] — universal safety net (lines 46-52):** after AST, line-window, or character chunking, enforce <code>MAX_CHUNK_CHARS = 2000</code> before provider pretruncation.
- **B005-F07-006 [E] — safe splitting/identity (lines 54-58):** character chunking scans backward for newline, space, tab, semicolon, or comma and identifies chunks by byte offset rather than line number.
- **B005-F07-007 [E] — constants (lines 80-92):** define <code>MAX_AVG_LINE_LENGTH: usize = 500</code> and <code>MAX_CHUNK_CHARS: usize = 2000</code>; the latter must stay under embedding tokens times chars-per-token.
- **B005-F07-008 [E] — detector function (lines 94-103):** <code>is_minified(content,line_count)</code> returns <code>content.len() &gt; 500</code> when line count is zero, otherwise integer-divided <code>content.len()/line_count &gt; 500</code>; caller supplies the count to avoid allocation.
- **B005-F07-009 [E] — character API (lines 105-114):** <code>chunk_by_characters(file_path,relative_path,content,language) -&gt; Vec&lt;FileChunk&gt;</code> owns minified splitting.
- **B005-F07-010 [E] — character algorithm (lines 115-130):** iterate byte offset, cap provisional end at 2,000, scan backward to a safe delimiter when not at EOF, slice on a UTF-8 boundary, use <code>chunk_id(relative_path,offset)</code>, calculate start/end-line metadata from newline counts, and advance without overlap.
- **B005-F07-011 [E] — cap behavior (lines 132-147):** <code>apply_char_cap</code> mutates every oversized chunk by finding a safe boundary at 2,000 and truncating content there; it is the final operation on every strategy.
- **B005-F07-012 [E] — dispatch order (lines 149-188):** split lines/detect language; detect minified before AST; otherwise return capped single chunk for tiny files, capped AST regions when available, or capped line windows as final fallback; emit debug path and computed average on detection.
- **B005-F07-013 [E] — detection event (lines 191-195):** emit one <code>MinifiedFileChunked</code> per detected file with <code>{relative_path,char_count,chunk_count,avg_line_length}</code> and surface it under Indexing activity.
- **B005-F07-014 [E] — provider defense/recovery evidence (lines 197-203):** retain or add a provider token/character cap; if it fires after universal capping, log “chunk pretrunc fired — should not happen if MAX_CHUNK_CHARS configured correctly” so invariant regression is observable.
- **B005-F07-015 [E] — produced/consumed symbols (lines 205-217):** export constants, <code>is_minified</code>, <code>chunk_by_characters</code>, <code>apply_char_cap</code>, and <code>SsbActionType::MinifiedFileChunked</code>; audit <code>chunk_file_content</code>, <code>make_chunks_from_content</code>, and every other knowledge chunk producer for detect-first/cap-last behavior.
- **B005-F07-016 [E] — reachability/stub bans (lines 218-228):** execute in full indexing and watcher incremental updates; prohibit any chunk-return path after the cap; reference the max constant at both cap and provider boundary; B-31’s IndexingProgressPanel is the frontend pair.
- **B005-F07-017 [E] — observable contract (lines 229-235):** a 50K+ one-line fixture must emit <code>MinifiedFileChunked</code>, create at least twenty-five chunks, and persist no chunk over 2,000.
- **B005-F07-018 [E] — deterministic retrieval checks (lines 237-270):** unit tests cover detection/splitting/cap/UTF-8; the fixture creates at least twenty stored chunks; global max length is at most 2,000; search still finds minified substrings; at least one event exists per file; emoji input must not panic.
- **B005-F07-019 [E] — acceptance invariants (lines 273-294):** strict 500 threshold, delimiter scan, cap-last on all strategies, byte-offset IDs, UTF-8 safety, per-file event, regression fixtures, provider defense, unchanged-or-improved Phase-3K metrics, and zero non-test unwrap are required.
- **B005-F07-020 [E] — unit test inventory (lines 295-312):** named tests cover single-line detection, normal rejection, max size, safe delimiters, unique IDs, truncation, four-byte boundaries, SSB emission, dispatch selection, and cap coverage for giant AST regions.
- **B005-F07-021 [E] — fixture inventory (lines 313-317):** add 50K valid JS, 150K valid CSS, generated Go with long strings, and a 50K JS file containing a four-byte emoji every thousand characters.
- **B005-F07-022 [E] — implementation footprint (lines 319-337):** add chunking constants/detect/character/cap modules and fixtures; modify BM25 dispatch, knowledge exports, SSB action type, and B-31’s V2 IndexingProgressPanel with optional “Detected N minified files” text; create <code>B-32-decisions.md</code>.
- **B005-F07-023 [E] — threshold/model rationale (lines 339-348):** decision record must justify 500 as bundler-sensitive without catching legitimate long lines and verify 2,000 against 8,192-token nomic and 512-token Granite contexts, choosing the conservative smaller-model fit.
- **B005-F07-024 [E] — truncation default (lines 349-351):** default to truncating oversize AST/line chunks and discarding their suffix rather than split-and-retain; the latter needs a larger refactor.
- **B005-F07-025 [E] — overlap default (lines 352-355):** character chunks are non-overlapping by default because overlap is judged unhelpful for opaque minified content.
- **B005-F07-026 [E] — skip flag default (lines 356-358):** record a possible <code>ELAI_SKIP_MINIFIED=1</code>; default is off so vendored bundles remain searchable.
- **B005-F07-027 [E] — side benefits (lines 360-371):** bound provider cost/errors, keep vendored/generated code searchable, use an O(n) byte scan without parser allocation, and establish cap-last defense in depth.
- **B005-F07-028 [E] — downstream (lines 373-379):** B-31 shares the refactor, B-29 is independent, and B-35 reuses the chunker for artifacts.

#### Ambiguities/conflicts

- **B005-F07-A01 [A] (lines 41-49, 94-100, 115-130):** identifiers say “characters,” but Rust <code>str::len</code>, offsets, and the 2,000 cap are bytes; UTF-8 boundary flooring preserves validity but can yield fewer than 2,000 Unicode scalar values.
- **B005-F07-C01 [C] (lines 46-52, 197-203):** context prose says provider pretruncation “always fires” as a second line, whereas the implementation task says it should normally be unreachable and logs if it fires.
- **B005-F07-A02 [A] (lines 191-195, 227-228, 332-335):** task prose names ActivityFeed, while the concrete frontend pair is B-31’s <code>IndexingProgressPanel</code>; whether both surfaces are required is not resolved.
- **B005-F07-A03 [A] (lines 247, 263):** deterministic SQL uses both <code>.elai_cc/.elai_cc/indexes/dkg.db</code> and <code>.elai_cc/.elai/sessions/ssb.db</code>, which conflicts with the family’s path-unification intent and is not reconciled here.
- **B005-F07-A04 [A] (lines 349-358):** truncate/split, overlap, and skip-indexing are decision-doc questions with defaults, not evidence that an environment flag or alternate algorithms already ship.
- **B005-F07-A05 [A] (lines 273-294):** all acceptance boxes remain unchecked; orientation status is PLANNED.

### F08 — B-43: One-Line Install

Canonical source: <code>B-43_one_line_install.md</code>, 670 lines.

#### Explicit atoms

- **B005-F08-001 [E] — status/scale (lines 1-7):** plan <code>B-43 one_line_install</code> is PLANNED, HEAVY, budgeted at GRC 8/1500 per topic, seven phases, three to five sessions, roughly 12–18 new and 3–5 extended files.
- **B005-F08-002 [E] — predecessors (lines 8-10):** consume B-37 runtime dispatch, B-42 Cosmos directory semantics, and C-28’s idempotent <code>ToolchainMissing</code>/dev-tool installation precedent.
- **B005-F08-003 [E] — locked successors/non-goals (lines 11-17):** defer Apple notarization/Windows EV signing to B-44, Tauri bundles/desktop integration to B-45, package managers to B-46, delta auto-update to a separate plan, Apple-Silicon MLX to E-51, and Linux desktop packages/integration to another plan.
- **B005-F08-004 [E] — installer name-collision lock (lines 19-25):** introduce <code>HostInstaller</code> in CLI install code; do not name it <code>BootstrapPlan</code>/<code>InstallPlan</code> and do not rename or modify the unrelated core <code>BootstrapPlan</code>.
- **B005-F08-005 [E] — exact sidecar command lock (lines 26-29):** reuse the REMOTE-SIDECAR command unchanged: <code>ELAI_FORCE_SDPA=1</code>, SearXNG at port 8889, Python <code>server.py</code>, the named 3B Qwen model, port 8100, no DoLA, zero TurboQuant bits, no steering, and 4-bit load.
- **B005-F08-006 [E] — idempotent/resume lock (lines 30-32):** mirror the eight-step environment/venv/pip/torch/deps/CUDA/HF/footprint pattern and persist partial-resume state at <code>~/.elai/install-state.json</code>.
- **B005-F08-007 [E] — CLI-only product boundary (lines 33-34):** ship CLI, sidecar runtime, and doctor; do not download or stage the Tauri dashboard binary.
- **B005-F08-008 [E] — no-auto-boot lock (lines 35-37):** HARDWARE-HEAVY means install ends after <code>elai doctor</code> succeeds and prints the exact local-boot/remote-tunnel hint; it must not start sidecar automatically.
- **B005-F08-009 [E] — anti-slopsquatting lock (lines 38-40):** every URL needs a PRE_GATE verification row and every Hugging Face repo needs an explicit WebFetch before its consuming phase; training-memory URLs are forbidden.
- **B005-F08-010 [E] — commit constraint (lines 41-43):** make exactly one final commit after Phase 7/exit criteria, in the specified maintainer voice; no per-phase commits or coauthor trailer.
- **B005-F08-011 [E] — release-tool choice (lines 45-50):** hand-roll a GitHub Actions matrix, use no cargo-dist and no new Rust dependencies, keep each OS entry at most thirty lines, and consider cargo-dist only as future B-46 if maintenance warrants.
- **B005-F08-012 [E] — unsupported-architecture behavior (lines 51-55):** macOS arm64 and Linux aarch64 produce the exact structured E-51/remote-tunnel message and exit 1; never silently fall back to cloud-only.
- **B005-F08-013 [E] — SearXNG topology lock (lines 56-59):** use a native Python venv on canonical port 8889; drop Docker and <code>--searxng-docker</code> entirely.
- **B005-F08-014 [E] — signing boundary (lines 60-61):** use a blake3 manifest and GitHub build-provenance attestations; notarization/EV signing remain B-44.
- **B005-F08-015 [E] — script runtime lock (lines 62-64):** use pure <code>bash -euo pipefail</code> and PowerShell 7+ with stop-on-error; do not add a second-hop bootstrap binary.
- **B005-F08-016 [E] — model staging default (lines 65-67):** eagerly stage models so first task is fast; <code>--skip-models</code> supports CI, firewalled, and image-building workflows.
- **B005-F08-017 [E] — PRE_GATE targets (lines 69-93):** verify unresolved GitHub org/repo host, two Qwen HF repos, GraphCodeBERT, Granite embedding, MiniLM cross-encoder, SearXNG Git HEAD, and Python 3.12.10 download before consumption.
- **B005-F08-018 [E] — placeholder lifecycle (lines 95-97):** every consuming phase cites its PRE_GATE row; release-host placeholders may remain while PLANNED but must be resolved before SHIPPED.
- **B005-F08-019 [E] — plan-level produced artifacts (lines 99-115):** create POSIX/PowerShell entrypoints and mirrored libraries, <code>CHECKSUMS.blake3</code>, config template, <code>HostInstaller</code>, four doctor extensions, install CLI parsing/help, release workflow, ONNX staging extension, and runtime install-state JSON.
- **B005-F08-020 [E] — plan-level consumed systems (lines 116-124):** use <code>detect_vram_tier</code>, all six <code>VramTier</code> values, doctor result types, existing <code>run_doctor</code>, locked server env/flags without editing server.py, B-37 default routing, and B-42’s directory creation without populating Cosmos.
- **B005-F08-021 [E] — entrypoints/re-entry (lines 125-130):** support HTTPS <code>curl ... | bash</code>, <code>iwr ... | iex</code>, idempotent <code>elai install</code> repair/re-run, and tagged release asset production.
- **B005-F08-022 [E] — no-stub/error bans (lines 132-141):** forbid TODO/placeholder/no-op bodies, unresolved checksums at SHIPPED, PowerShell 5.1 fallback, raw tool-check crashes, skipped hash verification, and swallowed HF limits; retry HF with backoff at most three times.
- **B005-F08-023 [E] — plan-level observable success (lines 142-147):** doctor exits zero on Win64/Linux x86_64, remote/cloud <code>elai run "ok"</code> returns Anthropic-shaped content, shell lookup finds canonical binary, and all eight install-state steps read <code>done</code>.
- **B005-F08-024 [E] — release-matrix risk mitigation (lines 149-154):** constrain hand-maintained matrix size and leave cargo-dist adoption reversible.
- **B005-F08-025 [E] — unsupported-Apple mitigation (lines 156-159):** structured error must route users to the working remote-sidecar tunnel rather than a dead end.
- **B005-F08-026 [E] — checksum atomicity mitigation (lines 161-164):** build and upload manifest in the same tagged-release step as binaries so artifact/hash cannot come from two sources.
- **B005-F08-027 [E] — pipe-install security mitigation (lines 166-169):** HTTPS-only, GH-host release source, binary blake3 verification, GH attestations, and a documented download-to-file/review alternative mitigate injection.
- **B005-F08-028 [E] — Python-version mitigation (lines 171-176):** probe specifically for Python 3.12, give a verified install URL, and do not silently use 3.13; the source points to SDPA as an opt-in response to missing flash-attn compatibility.
- **B005-F08-029 [E] — mandatory implementation reads (lines 178-190):** coding must inspect canonical remote setup/install scripts, VRAM logic, doctor/CLI parsers/help, ONNX staging, tool installer/error pattern, gotchas, and an existing pinned workflow.
- **B005-F08-030 [E] — Phase-1 release outputs (lines 194-214):** on <code>v*.*.*</code> tags and manual backfill, build supported Win-x64/Linux-x64 assets, blake3 manifest, and GH attestations; report two aarch64/macOS blocked entries loudly.
- **B005-F08-031 [E] — Phase-1 pinning/security (lines 204-217):** use existing blake3 1.8, verify and SHA-pin <code>actions/attest-build-provenance</code> and checkout, pin Ubuntu 24.04 rather than <code>ubuntu-latest</code>, and verify external actions through PRE_GATE/GitHub API.
- **B005-F08-032 [E] — Phase-1 pre-release test (lines 219-223):** feature-branch <code>workflow_dispatch</code> must create a draft with both supported binaries/checksums/attestations; a spoofed unsupported matrix row must fail with the structured Q2 text.
- **B005-F08-033 [E] — Phase-1 adversarial/determinism gates (lines 225-251):** require matrix markers and author/attestation steps; refuse empty/TBD manifests on tags; repeated same-tag builds must have identical blake3; cap each entry at thirty minutes and never upload partial timed-out artifacts; pass a draft end-to-end smoke.
- **B005-F08-034 [E] — Phase-2 hardware thresholds (lines 253-260):** mirror Rust thresholds: at least 40 GiB→48, 20→24, 14→16, 10→12, otherwise 8.
- **B005-F08-035 [E] — Phase-2 detection APIs (lines 264-274):** implement <code>detect_os</code>, <code>detect_arch</code>, <code>detect_gpu</code>, and <code>detect_vram_tier</code> in both script libraries; consume quoted <code>uname</code>, <code>nvidia-smi</code>, PowerShell CIM, and return exactly <code>8|12|16|24|48|cloud</code>.
- **B005-F08-036 [E] — WSL/PowerShell behavior (lines 267-286):** prefer <code>/usr/lib/wsl/lib/nvidia-smi</code> when applicable; refuse PowerShell 5.1 with “PowerShell 7+ required”; simulated unsupported architecture must fail before any HTTP fetch; shell and Rust tier results must agree.
- **B005-F08-037 [E] — Phase-2 fallback/adversarial states (lines 303-317):** an x64 CPU-only host returns <code>cloud</code> and proceeds cloud-only; WSL prefers its path if both binaries exist; Apple arm64 exits with exact Q2 text; accept only after all four functions and cross-checks pass.
- **B005-F08-038 [E] — Phase-3 fetch/state APIs (lines 318-335):** implement <code>fetch_binary</code>, <code>verify_blake3</code>, and <code>mark_state(step,status)</code> in both platforms; consume curl/IWR, blake3 CLI or verified Python fallback, and GH release URLs; mark <code>fetch_binary=done</code> plus <code>blake3_verified=true</code> and skip on valid rerun.
- **B005-F08-039 [E] — Phase-3 hard failure behavior (lines 326-345):** do not re-enter curl-pipe-bash, do not continue after manifest-fetch failure, resume from failed fetch after network loss, and show expected versus computed hashes before exit on mismatch.
- **B005-F08-040 [E] — Phase-3 verify-before-install (lines 348-360):** verification must precede chmod/execution and GitHub asset fetching must remain HTTPS-only.
- **B005-F08-041 [E] — Phase-3 adversarial/recovery gates (lines 362-373):** detect MITM/post-fetch swaps and partial corruption, never consume leaked temp data without re-verification, demonstrate kill-9 resume idempotency, and publish the review-before-bash alternative.

- **B005-F08-042 [E] — Phase-4 staging outputs/entrypoints (lines 375-391):** extend <code>stage_onnx_models.py</code> with <code>verify_blake3()</code> and manifest authoring; honor <code>HF_HOME</code>; select 3B, 7B, or no local model from Phase-2 tier; run eagerly from install step four, skip through <code>--skip-models</code>, and restage through <code>elai install --models</code>.
- **B005-F08-043 [E] — Phase-4 download integrity (lines 392-399):** preserve 1 MiB ONNX and 1 KiB tokenizer minimum-size guards, refuse hash mismatch without overwrite, verify all HF repositories and pin/verify <code>huggingface_hub</code>.
- **B005-F08-044 [E] — Phase-4 tier behavior (lines 401-407):** Vram24Gb stages 7B plus all four ONNX recipes; Vram8Gb stages 3B plus ONNX and warns that 7B was skipped; <code>--skip-models</code> avoids invoking the staging script and records <code>stage_models=skipped</code>.
- **B005-F08-045 [E] — Phase-4 retry/cache/recovery (lines 408-435):** retry HF 429 at most three times with exponential backoff then structured failure; on partial data refuse and refetch; reuse a valid cache without duplicate download; warn on suspicious tier override and suggest reset; a skipped-model task must later fail gracefully with a restaging instruction.
- **B005-F08-046 [E] — Phase-5 SearXNG APIs (lines 437-452):** implement <code>bootstrap_searxng()</code>, <code>_probe_python312()</code>, and <code>_probe_port_free()</code>; consume Python 3.12, upstream Git, and platform port probes; reserve <code>~/.elai/searxng.pid</code> for runtime.
- **B005-F08-047 [E] — Phase-5 no-start/topology behavior (lines 448-458):** install a native venv with no Docker fallback; port/Python failures use structured errors; do not auto-start SearXNG, leave PID absent, and direct the user to the canonical start script.
- **B005-F08-048 [E] — Phase-5 conflict/corruption recovery (lines 460-491):** port 8889 conflicts name the PID and exit; missing Python points to the verified 3.12.10 page; a corrupt venv is never auto-deleted and instead asks the user to remove it and rerun; successful rerun is idempotent/no-op.
- **B005-F08-049 [E] — Phase-5 deferred alternate port (lines 481-486):** error text may suggest <code>--searxng-port &lt;alt&gt;</code>, but that flag is explicitly future and not B-43.
- **B005-F08-050 [E] — Phase-6 config/credential artifacts (lines 493-513):** render <code>elai_cc.toml.template</code> with detected VRAM and B-37 runtime default; create <code>~/.elai/credentials.json</code> as <code>{}</code> with POSIX 0600 or restrictive NTFS ACL; NVIDIA defaults to Sidecar and cloud-only to AnthropicApi.
- **B005-F08-051 [E] — Phase-6 preservation/security (lines 503-513):** never overwrite existing config without diff plus <code>--reset-config</code>, never overwrite nonempty credentials, and reject empty runtime backend.
- **B005-F08-052 [E] — Phase-6 rerun/drift behavior (lines 515-544):** first install renders; rerun prompts/preserves; API keys survive untouched; detected-tier drift warns and suggests reset/override; acceptance requires correct fresh values and platform-restricted credential permissions.
- **B005-F08-053 [E] — Phase-7 CLI integration (lines 546-568):** add <code>HostInstaller</code>, <code>CliAction::Install</code>, parser, and Discovery/debugging help; extend doctor with real <code>check_runtime_assets</code>, <code>check_sidecar_health</code>, <code>check_vram_tier</code>, and <code>check_model_staging</code>; reuse existing doctor types and parse pattern.
- **B005-F08-054 [E] — Phase-7 code-quality/idempotency bans (lines 564-568):** prohibit <code>unimplemented!()</code>, non-test <code>unwrap()</code>, mock-OK diagnostics, and silent no-op on second install; report skipped cached steps.
- **B005-F08-055 [E] — Phase-7 post-install behavior (lines 569-575):** already-installed <code>elai install</code> reruns the eight-step state machine, doctor exits zero on supported hosts, and the locked no-auto-boot hint is printed; confirm install.rs is new and the unrelated core bootstrap is untouched.
- **B005-F08-056 [E] — Phase-7 TDD recovery (lines 577-585):** fresh and second install both end with successful doctor; skipped models produce “models not staged, run elai install --models” rather than panic; if doctor is initially green but run fails, the next doctor surfaces a Fail-level diagnostic.
- **B005-F08-057 [E] — Phase-7 deterministic done gates (lines 587-604):** verify file/name collision rules, all four doctor checks, CLI action/help registration, and defer the full workspace build check to the Auditor rather than the main implementation model.
- **B005-F08-058 [E] — supported-platform integration sequence (lines 606-613):** on clean Win64/Linux x86_64 VMs run the one-liner, finish in at most twelve minutes on Vram12Gb with eager models, pass doctor, return Anthropic-shaped <code>elai run "ok"</code> through a remote tunnel, and make the second install a no-op.
- **B005-F08-059 [E] — cloud-only/unsupported integration (lines 614-624):** <code>--skip-models</code> plus remote tunnel must succeed on supported x64; simulated aarch64/Apple Silicon must exit 1 with exact Q2 text; skipped local models must fail local task use gracefully, never panic.
- **B005-F08-060 [E] — Phase-7 acceptance (lines 626-631):** ship HostInstaller, four composed doctor checks, CLI surfaces, supported-OS end-to-end smokes, and unsupported-architecture error verification.
- **B005-F08-061 [E] — global exit gates (lines 633-648):** all seven phases require Auditor PASS; manifests need real hashes; supported doctor and unsupported errors must pass; exactly one constrained commit; no unconfirmed Rust dependency; author <code>B-43-decisions.md</code>; keep plan at most 700 lines and phase blocks at most ninety; resolve GitHub org and <code>elai.dev</code> before SHIPPED.
- **B005-F08-062 [E] — implementation footprint (lines 650-664):** create six install assets, release workflow, CLI install module, and decision record; extend doctor, args, commands, and ONNX staging with the approximate LOC budgets stated.
- **B005-F08-063 [E] — downstream (lines 666-670):** after B-43 ships, user chooses B-44 signing or B-45 dashboard installer.

#### Ambiguities/conflicts

- **B005-F08-A01 [A] (lines 7, 30-32, 147, 569-580):** the plan has seven delivery phases but repeatedly requires an eight-step runtime state machine; the eight runtime step names/order are not enumerated as a single schema.
- **B005-F08-A02 [A] (lines 75-77, 95-97, 125-130, 647-648):** the GitHub org/repository and <code>elai.dev</code> distribution domain are unresolved placeholders; no install URL may be considered production-ready before those confirmations.
- **B005-F08-A03 [A] (lines 26-29, 385-405):** the locked boot command always names the 3B model, while Phase 4 stages 7B on a 24GB tier; the plan does not state how the boot command selects the staged 7B.
- **B005-F08-A04 [A] (lines 213-214, 247-248):** Phase 1 says “4 artifacts (win-x64, linux-x64)” but elsewhere describes two supported targets plus two blocked matrix entries; the four artifact filenames are not enumerated.
- **B005-F08-C01 [C] (lines 125-130, 166-169, 332-333):** the one-liner executes the fetched installer script immediately, while the security prose says “blake3 verify before exec”; concrete Phase-3 verification clearly gates the downloaded binary, not the already-executing bootstrap script. The review-first alternative is the only stated inspection path for the script itself.
- **B005-F08-A05 [A] (lines 83-86, 401-403):** PRE_GATE lists three ONNX repositories, but Vram24Gb TDD says “all 4 ONNX recipes”; the fourth recipe/repository is not identified here.
- **B005-F08-A06 [A] (lines 161-164, 385-396):** one path/name <code>scripts/install/CHECKSUMS.blake3</code> is described for release binaries and model outputs; namespacing/atomic update behavior when both author it is not specified.
- **B005-F08-A07 [A] (lines 449-458, 462-486):** install refuses when port 8889 is occupied even though SearXNG is not started during install; the rationale for requiring the port to be free at staging time is unstated.
- **B005-F08-A08 [A] (lines 166-169):** “TLS pinned to GH host” is asserted as mitigation, but no certificate/SPKI pin implementation is specified and public one-liners use <code>elai.dev</code>.
- **B005-F08-A09 [A] (lines 303-305, 385-391, 493-513):** x64 CPU-only proceeds cloud-only, yet the plan does not state whether it still stages ONNX-only assets/SearXNG or how project-root <code>elai_cc.toml</code> is selected when invoked from a global one-liner.
- **B005-F08-A10 [A] (lines 633-648):** all phase and exit checkboxes are unchecked; canonical status is unambiguously PLANNED.

## Cross-file interactions

### Explicit interaction graph

- **X01 [E]:** B-6 requires B-2’s emit pattern and a nearby B-10 profile fix; B-17 then consumes B-6 alongside B-2/B-4/B-8 as the liveness meta-test (<code>F01:1-7, 101-102; F02:1-8, 31-46, 103-115</code>).
- **X02 [E]:** B-6 creates the INTERACTIVE request/response/card lifecycle; B-22 wraps the InteractiveGate for autonomy; B-26 identifies intervention schema/API/card as already shipped and adds remote webhook notification (<code>F01:64-70; F03:52-56, 97-99; F04:36-41, 60-66, 81-92</code>).
- **X03 [E]:** B-22 produces auto-advance signals; B-26’s already-shipped MonitorWindow polls and displays them, while B-36 is the real-invoke bridge and realistic stubs are the nonblocking fallback (<code>F03:18-38, 128-130; F04:11-17, 43-50</code>).
- **X04 [E]:** B-17 explicitly tests the B-6 supervision producer with a three-crash escalation and aggregates B-6-style <code>EvidenceGateEvaluated</code> rows at two TaskRunner transitions (<code>F02:56-69, 99-115</code>).
- **X05 [E]:** B-28 depends on B-20’s three-tier edit matcher and 3F anti-stale patterns, and relies on B-22 for full autonomous benefit; B-26 and B-28 are intentionally parallel-safe (<code>F05:1-9, 74-79, 100-107, 141-150; F04:19, 239-241</code>).
- **X06 [E]:** B-29 requires B-1’s canonical DKG path, integrates B-31 watcher invalidation, backs B-33 symbol visualization, shares cache payloads with B-34, and is parallel-safe with B-32/B-35 (<code>F06:1-10, 206-229, 301-302, 429-448</code>).
- **X07 [E]:** B-32 requires B-1, is a natural B-31 batch-refactor sibling, adds minified telemetry to B-31’s progress panel, and supplies B-35’s artifact chunking (<code>F07:1-10, 205-228, 319-337, 373-379</code>).
- **X08 [E]:** B-43 uses B-37 to render hardware-appropriate runtime defaults, creates but does not populate B-42’s Cosmos directory, and reuses C-28’s structured toolchain/idempotency pattern (<code>F08:8-10, 116-124, 493-513</code>).
- **X09 [E]:** B-43 explicitly excludes the dashboard despite B-26 being dashboard-only; B-45, not B-43, owns packaged Tauri installation (<code>F08:11-17, 33-34</code>; <code>F04:1-7</code>).
- **X10 [E]:** B-29 and B-32 both extend the indexing lifecycle but at distinct layers: B-29 builds a symbol graph after DKG construction, while B-32 changes content chunk dispatch before embedding/BM25; both enter full and watcher update paths (<code>F06:212-229</code>; <code>F07:149-188, 218-221</code>).

### Conservative inferences

- **B005-I-001 [I]:** composing B-6, B-17, B-22, and B-26 yields a control loop of producer verdict → non-vacuity regression proof → autonomous decision/human escalation → live Monitor/intervention notification. Each edge is explicit, but this end-to-end sequence is not stated in one canonical file.
- **B005-I-002 [I]:** because B-22 can auto-advance only after evidence and B-17 fails CI when gate inputs go vacuous, B-17 is indirectly a safety gate for autonomous execution, not merely dashboard coverage.
- **B005-I-003 [I]:** B-26’s real-invoke-or-mock architecture allows UI acceptance work to proceed before B-36, but production evidence still needs the real Tauri route; mock success cannot prove producer reachability.
- **B005-I-004 [I]:** B-28’s “preferred mode” should be treated as a hint constrained by deterministic capability/safety rules, because allowing it to override unsupported FIM or new-file STRUCTURED fallback would violate explicit bans.
- **B005-I-005 [I]:** B-29’s reverse index is file-based even when a symbol is queried, so an implementation must first resolve the symbol to one or more seed files before its documented BFS; that missing bridge is implementation work, not permission to scan the whole graph.
- **B005-I-006 [I]:** B-29’s PostToolUse placement means <code>IMPACT_SUMMARY</code> grounds audit of an edit that already occurred; it cannot be a pre-write prevention gate unless an additional pre-tool hook is added.
- **B005-I-007 [I]:** B-32’s 2,000 “character” invariant is operationally a maximum of 2,000 UTF-8 bytes, because all canonical arithmetic and IDs use byte length/offset.
- **B005-I-008 [I]:** B-32’s default cap can lose the tail of a giant AST/line chunk; retrieval completeness for minified input is preserved only because detected minified files go through repeated character chunks rather than truncating one AST region.
- **B005-I-009 [I]:** B-43’s no-auto-boot rule and required post-install doctor exit zero imply <code>check_sidecar_health</code> must distinguish “installed but intentionally stopped” from an installation failure, likely as non-Fail, though the source does not state the diagnostic level.
- **B005-I-010 [I]:** B-43’s eager model staging and B-28’s FIM capability detection are complementary: a local tier can be ready for fast FIM after install, but only if runtime model selection resolves the 3B/7B command ambiguity.
- **B005-I-011 [I]:** B-43’s unsupported-architecture hard error is a product-install gate, not just a local-sidecar gate; therefore even cloud-capable Apple/Linux ARM users cannot use this one-line flow until E-51 or a later installer changes the locked decision.
- **B005-I-012 [I]:** B-43’s partial-state, hash, and preservation rules make reruns idempotent only for validated artifacts; corrupt venvs/config conflicts intentionally stop for user action rather than self-heal destructively.

## Required additions and corrections to the orientation summary

The entries below are deltas for <code>02-B-PLAN-SUMMARY.discovery.md</code>. “Addition” means canonical detail absent or materially compressed there. “Correction” means the summary’s wording conflicts with the canonical source. Atom references point to the nonduplicated ledger above.

### B-6 summary deltas

- **SD-F01-01 — addition:** add V2 command/drawer/orbit mappings, the five initial stub calls, seven-pip/detail split, and B-10 degraded-zero behavior (B005-F01-002–005, 015–016).
- **SD-F01-02 — addition:** preserve exact event/query payloads, limits, heartbeat thresholds, supervision enum, BSP fields/UI string, adaptive state chain, and overlap fields (B005-F01-006–018).
- **SD-F01-03 — addition:** include <code>AgentStatusChanged</code>, the produced command registry, exact reachability/stub bans, deterministic searches, smoke gap, acceptance, and footprint (B005-F01-019–025).
- **SD-F01-04 — summary-only claim/status caveat:** schema <code>25 → 26</code> and unqualified DONE are not stated in canonical B-6; its acceptance boxes are unchecked (B005-F01-A03). Preserve the schema/status as later-summary/session evidence, not a canonical B-6 assertion.

### B-17 summary deltas

- **SD-F02-01 — addition:** add the concrete six fixture inputs/outcomes and all count categories, not merely the gate names (B005-F02-004–011).
- **SD-F02-02 — addition:** add six-active/two-ignored reality, exact CI/dashboard call chain, two TaskRunner emission sites, <code>GROUP BY gate_id, verdict</code>, batch runner, and implementation footprint (B005-F02-016–023).
- **SD-F02-03 — correction:** canonical concrete path is <code>crates/orchestrator/tests/gate_liveness.rs</code>, not the earlier <code>tests/contracts</code> location (B005-F02-C01).
- **SD-F02-04 — correction:** do not describe a B-17 standalone <code>GatesPanel.svelte</code>; later canonical contract says inline VERDICT COUNTS and reserves that filename for B-38 (B005-F02-C02).
- **SD-F02-05 — addition/open:** retain unspecified N, unresolved dismissal default, and unidentified ignored tests instead of implying they are decided (B005-F02-A01–A02).

### B-22 summary deltas

- **SD-F03-01 — addition:** add V2 stub-to-live routing, per-session/next-gate semantics, sequential split merge, model-selection modes/config, exact produced types, feature-flag constraint, and canonical <code>gate_decision</code> evidence (B005-F03-003–004, 008–019).
- **SD-F03-02 — correction:** canonical base verification target is 36 tests (28 contract + 8 adversarial); the summary’s eighteen-contract phrase is not a replacement count (B005-F03-012, B005-F03-C01).
- **SD-F03-03 — correction:** distinguish canonical <code>gate_decision</code>/<code>model_swap</code> from summary registry names <code>auto_advance_decision</code>, <code>autonomy_escalation</code>, and <code>split_task_initiated</code>; their relationship is unresolved (B005-F03-C02).
- **SD-F03-04 — addition:** add exact stagnation predicates, persona affinity/selection/exhaustion, optional all-five Tier-C fan-out, watchdog material-event allowlist/exclusions/dedup, drift formula/tokenization/bands/retry/skip, and explicit non-ports (B005-F03-021–035).
- **SD-F03-05 — summary-only additions:** <code>derive_autonomous_evidence</code> query types, <code>SsbViewer::auto_advance_stats</code>, <code>AgentPool::get_role</code>, live-mode/LIMIT-1000/SeededClock follow-ups, and the “eighteen extended contracts” detail occur only in the orientation summary, not canonical B-22; keep them attributed to that later source.

### B-26 summary deltas

- **SD-F04-01 — addition:** add exact already-shipped polling intervals, commands, animation timings, Explore/Review behavior, localStorage key/schema, and shortcut inventory (B005-F04-004–013).
- **SD-F04-02 — addition:** add webhook payload shape/optional trigger/concurrency/swallowed error, URL validation/on-blur state, shortcut single-source rule, default removal branch, real-fetch stub ban, and deterministic SSB prohibitions (B005-F04-014–025).
- **SD-F04-03 — status correction:** retain “majority shipped; six remaining gaps with unchecked acceptance,” not blanket completion (B005-F04-001–003, 026, B005-F04-A04).

### B-28 summary deltas

- **SD-F05-01 — addition:** add exact FIM prefix/suffix composition and PostToolUse validation, diff hunk schema/parse fallback, <code>preferred_mode</code>, <code>mode_distribution</code>, <code>early_termination_rate</code>, and <code>tokens_saved</code> (B005-F05-012–015).
- **SD-F05-02 — addition:** add preassembly discard, confirmation/transitive rules, exact MINOR microstep/retry, dashboard waste/dependency details, hard bans, smoke, footprint, and Part-IV/deferred boundaries (B005-F05-016–027).
- **SD-F05-03 — caveat:** savings are estimates and acceptance remains unchecked/PLANNED (B005-F05-A03–A04).

### B-29 summary deltas

- **SD-F06-01 — correction:** Imported/Re-export are resolver search tiers, not <code>Confidence</code> variants; use <code>Local|Unique|MultipleCandidates|Unresolved</code> for confidence (B005-F06-012, 015, B005-F06-C01).
- **SD-F06-02 — correction:** incremental update applies at <code>changed_file_count &lt;= 50</code>; full rebuild is strictly above fifty (B005-F06-022–024, B005-F06-C02).
- **SD-F06-03 — addition:** add all schema columns, domain types, SHA1/blake3 division, synthetic module, cache load mechanics, exact traversal return/recovery types, dispatcher schemas, SSB payload, panel fields, and sanity bounds (B005-F06-007–036).
- **SD-F06-04 — addition:** add all property/fixture/end-to-end tests, no-unwrap/no-dependency gates, decision-doc branches, and unlocked consumer workflows (B005-F06-037–044).
- **SD-F06-05 — correction/caveat:** canonical hook is PostToolUse despite “pre-edit” wording; std-only LRU is a recommendation constrained by dependency policy; the double-prefixed DB path and SHA1 exception remain unresolved rather than normalized (B005-F06-C03, B005-F06-A01–A03).

### B-32 summary deltas

- **SD-F07-01 — addition:** add strict <code>&gt; 500</code> detector semantics including zero-line case, complete byte/UTF-8 algorithm, cap-as-truncation, tiny/AST/line dispatch order, event payload, and provider fail-safe (B005-F07-004–016).
- **SD-F07-02 — addition:** add exact observable thresholds, retrieval/UTF-8 checks, named tests/fixtures, metric nonregression, default truncation/no-overlap/indexing behavior, and progress-panel footprint (B005-F07-017–028).
- **SD-F07-03 — correction/caveat:** call the 2,000 bound byte-based in implementation, preserve the provider-pretrunc wording conflict, and do not silently resolve ActivityFeed versus IndexingProgressPanel or inconsistent DB paths (B005-F07-A01–A04, B005-F07-C01).

### B-43 summary deltas

- **SD-F08-01 — addition:** the orientation summary contains no B-43 feature inventory; all 63 canonical atoms B005-F08-001–063 are post-summary additions.
- **SD-F08-02 — correction:** the summary’s B-41 “future visual audit is B-43” identifier is superseded/collides with canonical <code>B-43_one_line_install.md</code>; do not attribute visual audit behavior to this plan.
- **SD-F08-03 — addition/status:** add the explicit PLANNED/HEAVY status, seven locked decisions, six resolved questions, seven-phase release/install/doctor lifecycle, supported/blocked platforms, security and recovery behavior, and all deferred successor boundaries; also retain all ten open ambiguities and the bootstrap-verification conflict rather than presenting B-43 as ready to ship.

## Status, deferred, security, and evidence roll-up

- **Canonical status:** B-43 explicitly says PLANNED. B-22 explicitly says the base shipped and B-22b remains work. B-26 explicitly says the majority is shipped and six gaps remain. B-6/B-17/B-28/B-29/B-32 retain unchecked acceptance; their DONE/PLANNED labels in the orientation summary are later status evidence and are not silently promoted into the canonical files.
- **Primary deferred boundaries:** B-6 autonomous MONITOR coupling; B-17 dismissal policy; B-22b 4G.9; B-26 recently-closed recovery alternative and phase transport choice; B-28 Phase 5/compiled pipelines; B-29 confidence-weighted hop fusion and B-33/B-34 consumers; B-32 skip/overlap/split alternatives; B-43 signing, dashboard packaging, package managers, updater, ARM/MLX, and desktop integration.
- **Security/privacy/compliance:** B-26 layout is local-only and webhook errors cannot affect UI; B-28 requires confirmation/allowlists and forbids model package-manager commands; B-29 reimplements rather than copies AGPL code and bans unconfirmed deps; B-32 preserves provider defense; B-43 requires URL pre-gates, HTTPS, hashes, attestations, restricted credentials, no destructive overwrite, structured errors, supported-platform refusal, and no heavy auto-boot.
- **Failure/retry/recovery:** intervention responses unblock B-6; B-17 detects vacuity; B-22 splits/substitutes/escalates with bounded retries and watchdogs; B-26 swallows webhook-only failures; B-28 falls back to STRUCTURED and caps microsteps; B-29 truncates traversals/cycles and rebuilds above threshold; B-32 uses cap/provider fallback; B-43 uses resumable state, hash refusal, HF retry, preservation prompts, explicit corrupt-venv cleanup, and repeatable install.
- **Determinism/idempotency/concurrency:** deterministic B-28 selection, B-29 BTreeMap hop bands and bounded cache, B-32 byte-boundary chunking, B-43 same-tag hashes and idempotent state; concurrency is explicit in B-22 persona fan-out/watchdog, B-26 parallel notifications, B-28 depth-one preassembly, B-29 shard JoinSets, and B-43 release matrix/HF retry bounds.

## Zero-omission and zero-duplicate audit

### Files, lines, bytes, and explicit atomic counts

| File ledger | Canonical file | Lines | Bytes | Explicit atoms |
|---|---|---:|---:|---:|
| F01 | <code>B-6_orchestration_producer_wiring.md</code> | 197 | 8,949 | 25 |
| F02 | <code>B-17_ground_vacuous_gates.md</code> | 155 | 7,682 | 23 |
| F03 | <code>B-22_phase_4G_autonomous_mode.md</code> | 277 | 12,204 | 36 |
| F04 | <code>B-26_phase_4K_6_to_13_dashboard_completion.md</code> | 241 | 11,181 | 28 |
| F05 | <code>B-28_focus_tunnel.md</code> | 211 | 8,007 | 27 |
| F06 | <code>B-29_symbol_level_impact_api.md</code> | 448 | 23,173 | 44 |
| F07 | <code>B-32_minified_file_chunking.md</code> | 379 | 16,783 | 28 |
| F08 | <code>B-43_one_line_install.md</code> | 670 | 38,562 | 63 |
| **Total** | **8 files** | **2,578** | **126,541** | **274** |

Supplemental records: **12 [I] inferences**, **51 [A]/[C] ambiguity-or-conflict records**, **31 summary-delta records**, and **10 explicit cross-file interaction records**. These are deliberately excluded from the 274 feature atoms because they reference, qualify, or compose existing atoms.

### Completeness disposition

- Every canonical source was read from line 1 through EOF at its verified manifest hash. No source from DEEP-B-004, DEEP-B-006, or any other original chunk was opened.
- Feature-bearing prose, task items, schemas, WIRING_CONTRACT symbols, lifecycle/reachability, stub bans, observable effects, deterministic checks, acceptance conditions, test inventories, implementation surfaces, decisions, risks, next/deferred work, and explicit status were mapped into F01–F08.
- Non-atomic material was still reviewed: headings/blank lines, repeated restatements, shell formatting, and mandatory “Reads” bibliographies. Reference-only ranges include F01:45-53; F02:48-52; F03:88-94; F05:57-61; F06:73-94; F07:63-76; and F08’s plan/phase Reads blocks. Their operational requirements are captured where they impose verification, dependency, licensing, or anti-slopsquatting behavior.
- Repeated canonical statements were consolidated only when their semantics were identical. Distinct task, contract, test, and acceptance claims remain separate when they add a payload, threshold, state, reachability fact, or done gate.
- Atom IDs are scoped by file and monotonically contiguous: F01 001–025, F02 001–023, F03 001–036, F04 001–028, F05 001–027, F06 001–044, F07 001–028, F08 001–063. No atom ID is reused.
- Path uniqueness audit: 8 manifest rows, 8 unique absolute paths, 8 unique SHA-256 values, zero missing rows, zero extras, zero duplicate originals.
- Line/byte parity audit: per-file and aggregate <code>wc -l -c</code> values match the manifest exactly.
- Classification audit: explicit claims stay in the per-file ledgers; inferred composition stays only in the inference section; open choices stay [A]; contradictions stay [C]; summary-only material is attributed only in the delta section.

### Mechanical count checks

- Expected explicit-ID regex count: **274**.
- Expected duplicate explicit IDs: **0**.
- Expected missing numbers inside each F01–F08 sequence: **0**.
- Expected source total: **8 files / 2,578 lines / 126,541 bytes**.
