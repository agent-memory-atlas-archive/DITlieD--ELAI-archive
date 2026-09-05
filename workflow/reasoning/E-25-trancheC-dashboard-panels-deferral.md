E-25 Tranche C (C.5) -- dashboard panels documentation-only deferral
Source session: s-22600 Tranche C
Date: 2026-05-31

DEFERRED: CompactionLadderPanel.svelte + Tauri list_compaction_events command.
DESCOPED: McpSchemaPanel.svelte.
REDUNDANT: SessionForkPanel.svelte.

CompactionLadderPanel BLOCKED-ON-PRODUCER (Tranche-C explorer SUBSYSTEM 5 GAP-5):
- run_compaction_ladder has ZERO production callers in task_runner.rs (17033 lines verified by explorer; only contract test e25_tranche_b4_contract.rs:178 + 3 in-module unit tests exercise the runner).
- No CompactionLadderTierApplied rows are persisted by any live wire site today. Shipping the panel now would render permanently empty and violate B-23 4I.1 graceful-empty rule.
- Will-resolve-in: future tranche that FIRST wires task_runner.rs adaptive_reduce / invoke_telescope / compact_session branches to call run_compaction_ladder under ELAI_E25_COMPACTION_LADDER_ENABLE; AFTER at least one production row lands, ship the panel.

McpSchemaPanel DESCOPED (session-3 DECISION-3):
- Depends on E-22 Task 3 / C-35 lazy_schemas substrate which is unshipped.
- Adding the panel now would lie about ELAI's surface; the entry will be added in a follow-up once E-22 / C-35 ships.

SessionForkPanel REDUNDANT (B-4 D10):
- Existing SessionsDrawer.svelte + BranchTree.svelte covers fork visualization end-to-end.
- Shipping a parallel SessionForkPanel would double the UX surface; explicitly NOT shipped.

References:
- workflow/sessions/s-22600/exploration/trancheC-integration.md SUBSYSTEM 5 (GAP-5 + producer-zero-callers verification)
- workflow/reasoning/E-25-decisions.md DECISION-3 (McpSchemaPanel descope)
- workflow/reasoning/E-25-tranche-B4-decisions.md D8 / D10 (path correction + SessionFork redundancy)
- crates/orchestrator/src/task_runner.rs (zero hits for CompactionLadderTierApplied / run_compaction_ladder)
- crates/dashboard-v2/src-tauri/src/commands.rs:3434-3475 (canonical SessionForked Pattern-2 producer precedent for the future panel)

RESOLUTION

Tranche N (s-22600 2026-06-01, finalized after process-death recovery) closed
this deferral: producer wired @task_runner.rs L8896 (PreCompact branch calls
run_compaction_ladder under ELAI_E25_COMPACTION_LADDER_ENABLE; sentinel budget
PRE_COMPACT_PROBE_BUDGET=30_000 -> PRE_COMPACT_LADDER_TARGET_BUDGET=20_000 per
the compaction_ladder gate_on_above_target_emits_until_below acceptance shape)
+ commands_e25.rs Tauri surface (get_compaction_ladder_state for env-gate +
LOCKED 6-tier order + default target; recent_compaction_events for filtered
claude_code_architecture_event SSB rows of kind=compaction_ladder_tier_applied)
+ src/lib/components/CompactionLadderPanel.svelge mounted in SettingsWindow.svelte
+ n_compaction_ladder_wire_contract.rs contract test. cargo check --workspace
--tests GREEN.

D-N3.a corrects original panel path from src/lib/drawers/ to src/lib/components/
per SkillsPanel + CriticDecisionPanel precedent (panels mounted inside windows
live in components/; drawers/ is reserved for slide-out / overlay surfaces).

D-N4: zero new SsbActionType / Kind arm / SCHEMA_VERSION bump. Wire rides the
SHIPPED Pattern-2 single-variant cascade (ClaudeCodeArchitectureEvent +
CompactionLadderTierApplied Kind arm).

McpSchemaPanel + SessionForkPanel remain DESCOPED / REDUNDANT per original C.5
deferral; NOT shipped this tranche.

See workflow/reasoning/N-trancheN-decisions.md for the full 6-decision set
(D-N1 producer call-site, D-N2 sentinel budget, D-N2.b two-command split,
D-N3.a panel path correction, D-N4 zero-schema-bump, D-N5 wire-dark default
OFF, D-N6 DEFAULT_LIMIT/MAX_LIMIT bounds).
