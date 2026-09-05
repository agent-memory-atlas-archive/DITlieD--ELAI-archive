Tranche N decisions -- E-25 Compaction-Ladder dashboard panel wire
Source session: s-22600 Tranche N (finalized 2026-06-01 after process-death recovery)
Plan reference: E-25 Tranche C C.5 deferral (workflow/reasoning/E-25-trancheC-dashboard-panels-deferral.md)

CONTEXT

Tranche N closes the BLOCKED-ON-PRODUCER deferral that Tranche C left open:
CompactionLadderPanel.svelte + Tauri commands could not ship until task_runner.rs
held at least one production call site for run_compaction_ladder. Tranche N wires
the PreCompact branch as the FIRST production caller, then ships the read surface
that consumes the rows it emits.

The original Tranche N executor (s-22600) died partway through after landing the
five code artifacts but before writing the finalization paperwork. This document
closes the loop and records the decisions that were either pre-resolved in the
context-package or reconstructed from the shipped code.

DECISIONS

D-N1 -- Producer call-site lives in the PreCompact branch of drive_task
Decision: wire run_compaction_ladder under the PreCompact arm of drive_task at
crates/orchestrator/src/task_runner.rs:8896 inside the dispatch_pre_compact /
hook ladder. The PreCompact event is the canonical pre-existing budget-pressure
signal; running the ladder here matches the intent of B-23 / E-25 (cheapest-tier
first, stop once below target) without inventing a new hook surface.
Grounding: ALL_COMPACTION_TIERS const + LadderEventEmitter trait at
crates/orchestrator/src/architecture/claude_code_arch.rs; PreCompact already
emitted via dispatch_pre_compact at task_runner.rs prior to Tranche N.
Alternatives rejected: adaptive_reduce branch (too early in the pipeline; budget
not yet truly under pressure); invoke_telescope branch (cross-cuts a different
subsystem, would couple two unrelated optimizations).

D-N2 -- Sentinel pre-compact probe budget = 30_000 tokens
Decision: PRE_COMPACT_PROBE_BUDGET = 30_000 (crates/orchestrator/src/task_runner.rs:141)
+ PRE_COMPACT_LADDER_TARGET_BUDGET = 20_000 (crates/orchestrator/src/task_runner.rs:147).
The 30k -> 20k shape mirrors the
compaction_ladder.rs::tests::gate_on_above_target_emits_until_below acceptance
test (30_000 over -> ladder fires until 20_000 target) AND the LOCKED B-4 D1
budget shape.
Rationale: the producer is deterministic until a follow-up tranche wires
context_budget::current_session_budget() through. Until that happens, a fixed
sentinel keeps the wire emitting predictable rows the dashboard can render.
TODO[N-FOLLOWUP] registered inline at the const docstring; resolution = future
patch swapping the literal for a live ContextWindow probe.

D-N2.b -- Tauri command names: get_compaction_ladder_state + recent_compaction_events
Decision: two commands, both async, both returning Result<DTO, String>, registered
in lib.rs::invoke_handler! at lib.rs:806-807.
- get_compaction_ladder_state: pure config snapshot. Reads
  ELAI_E25_COMPACTION_LADDER_ENABLE env-gate + LOCKED 6-tier order + default
  target budget. No SSB read.
- recent_compaction_events: filters claude_code_architecture_event SSB rows to
  kind = compaction_ladder_tier_applied, flattens to per-row timeline DTO.
  Mirrors commands_c41::recent_skill_routing_events pattern.
Alternatives rejected: a single list_compaction_events command (Tranche C deferral
memo's original sketch) -- splitting config snapshot from event stream lets the
panel render the wire-dark banner cheaply without paying for a SSB scan when
disabled.

D-N3.a -- Panel path corrected: src/lib/components/ NOT src/lib/drawers/
Decision: CompactionLadderPanel.svelte lives at
crates/dashboard-v2/src/lib/components/CompactionLadderPanel.svelte.
The Tranche C deferral memo and the stubs.md row originally named
src/lib/drawers/CompactionLadderPanel.svelte. That path was wrong.
Grounding: existing precedent set by SkillsPanel + CriticDecisionPanel which both
live under src/lib/components/ (not drawers/). Panels mount inside windows;
drawers/ is reserved for slide-out / overlay UI surfaces. Compaction-ladder is
a config-and-timeline panel mounted inside SettingsWindow, so components/ is the
correct sibling.
Action taken: panel created at the corrected path; SettingsWindow.svelge imports
from components/.

D-N4 -- Zero new SSB variant / Kind arm / SCHEMA_VERSION bump
Decision: the wire rides the SHIPPED Pattern-2 single-variant cascade
(ClaudeCodeArchitectureEvent + CompactionLadderTierApplied Kind arm).
Tranche N adds NO new SsbActionType, NO new ClaudeCodeArchitectureEventKind arm,
NO SCHEMA_VERSION bump. The 9-arm ClaudeCodeArchitectureEventKind enum
(SCHEMA_VERSION 191 at Tranche N entry) is LOCKED; the
CompactionLadderTierApplied arm has been shipped since B-4 / Tranche B-4 but had
zero production emitters until Tranche N wired the PreCompact branch.
Grounding: commands_e25.rs:38 hardcodes "claude_code_architecture_event" against
crate::ssb::SsbActionType::ClaudeCodeArchitectureEvent at
crates/orchestrator/src/ssb.rs:8618; commands_e25.rs:43 hardcodes
"compaction_ladder_tier_applied" against
ClaudeCodeArchitectureEventKind::CompactionLadderTierApplied::as_str() at
crates/orchestrator/src/architecture/claude_code_arch.rs:276.

D-N5 -- Wire-dark default OFF
Decision: producer side gated by ELAI_E25_COMPACTION_LADDER_ENABLE=1 on the
orchestrator. The CompactionLadderStateDto.enabled field reflects this env var
on the dashboard side so the panel renders a wire-dark banner when off.
Default: OFF. Same default as every other Tranche-N-adjacent E-25 wire.

D-N6 -- DEFAULT_LIMIT=50, MAX_LIMIT=500 on recent_compaction_events
Decision: limit defaults to 50 rows, capped at 500. Matches the
commands_c41::recent_skill_routing_events constants.
Grounding: c41 precedent set in B-N earlier this session; staying inside the
same bounds keeps dashboard polling pressure uniform across sibling panels.

DEVIATIONS FROM PRE-WRITTEN CONTEXT (reconstructed from shipped code)

DEV-N1 -- Panel path drift correction surfaced mid-execution
Original Tranche C deferral memo + s-22300 stubs.md row both named
.../src/lib/drawers/CompactionLadderPanel.svelte. The original Tranche N
executor caught the SkillsPanel / CriticDecisionPanel precedent and shipped at
.../src/lib/components/CompactionLadderPanel.svelte instead. This finalization
recognises that move as D-N3.a (correct decision; would have been recorded in
the live decisions doc had the executor survived).

DEV-N2 -- No explicit "list_compaction_events" command -- two-command split shipped
Tranche C deferral memo named a single "list_compaction_events" Tauri command.
The original Tranche N executor split this into get_compaction_ladder_state
(config snapshot, no SSB read) + recent_compaction_events (event stream, SSB
read). This split is the right shape (D-N2.b) -- the panel needs both pieces of
data and serving them from one command would have either over-fetched on every
config-only render or under-served when only events were needed.

PROCESS NOTE -- finalization after process-death recovery

The original Tranche N executor died partway through after landing five code
artifacts:
- crates/orchestrator/src/task_runner.rs (PRE_COMPACT_PROBE_BUDGET const @L141,
  PRE_COMPACT_LADDER_TARGET_BUDGET const @L147, run_compaction_ladder call inside
  PreCompact branch @L8896)
- crates/dashboard-v2/src-tauri/src/commands_e25.rs (NEW, 269 lines)
- crates/dashboard-v2/src/lib/components/CompactionLadderPanel.svelte (NEW)
- crates/dashboard-v2/src/lib/windows/SettingsWindow.svelte (panel mounted)
- crates/contracts/tests/n_compaction_ladder_wire_contract.rs (NEW)
- crates/dashboard-v2/src-tauri/src/lib.rs (mod decl @183 + invoke_handler!
  registration @806-807) -- VERIFIED LANDED before process death

This finalization session (s-22600 2026-06-01) closes the loop with:
- this decisions doc
- E-25-trancheC-dashboard-panels-deferral.md RESOLUTION block
- s-22300 stubs.md row update for E25-dashboard-panels (DOCUMENTED-DEFERRAL ->
  WIRED-Tranche-N)
- cargo check --workspace --tests GREEN verdict (only pre-existing unused-import
  warnings; zero errors; zero new warnings introduced by Tranche N code)

McpSchemaPanel + SessionForkPanel remain DESCOPED / REDUNDANT per Tranche C
DECISION-3 / B-4 D10; NOT shipped this tranche.

References
- workflow/sessions/s-22600/context-package-trancheN.md (originating context)
- workflow/reasoning/E-25-trancheC-dashboard-panels-deferral.md (deferral now
  resolved)
- workflow/reasoning/E-25-tranche-B4-decisions.md D8 (path-correction precedent)
- workflow/reasoning/E-25-decisions.md (E-25 overall decisions)
- crates/orchestrator/src/architecture/claude_code_arch.rs (LOCKED 9-arm
  ClaudeCodeArchitectureEventKind + ALL_COMPACTION_TIERS)
- crates/orchestrator/src/compaction_ladder.rs (run_compaction_ladder runner)
- crates/dashboard-v2/src-tauri/src/commands_c41.rs (sibling pattern for
  recent_compaction_events shape)
