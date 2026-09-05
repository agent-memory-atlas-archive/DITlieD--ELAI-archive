C-12 — Per-Level Auditor Navigator Wiring — Decisions

Session: s-32952
Date: 2026-05-18
Plan file: docs/thefinalglance/C-plan/C-12_per_level_auditor_navigator_wiring.md
Context package: workflow/sessions/s-32952/context-package-C12.md

==================================================================
Summary
==================================================================

C-12 wires per_level_audit_dispatch::emit_started + record_verdict at 5
tier-completion boundaries inside task_runner.rs:drive_task, replaces the
get_per_level_verdicts Tauri command stub with a live SsbViewer-backed
read, and lands a contract test covering the 5 emit sites + verdict
shape + IdeaFramed-absent WARN path. Two follow-up items are explicitly
deferred (documented below); none of them blocks the C-12 acceptance
matrix.

==================================================================
DECISION-LOCATION (resolves DRIFT-2)
==================================================================

The plan called for 5 emit sites inside pipeline_tree_navigator.rs.
The navigator has no Auditor* spawn sites and no boundary-detection
loop (the navigator's bubble-up paths run on FAILURE only, not on
SUCCESS-side tier completion). Per context-package DRIFT-2 option (b)
and the orchestrator decision, the 5 emit sites are attached at
task_runner.rs:drive_task, immediately after each PhaseNodeReady /
StepNodeReady / BabyStepReady SSB emit at lines ~4975-5047. The runner
already has `last_location` boundary detection, holds the Ssb writer,
and is where the C-2 ready-row family already fires.

Mapping (5 distinct call sites, one per AuditorMode):

  Site 1 — Concepts mode  (TierNodeId::Concept)
    Trigger: prev_concept_id != Some(cur_loc.concept_id)
    Performs: IdeaFramed-presence assertion (WARN if absent) +
              emit_started + record_verdict(allow)
  Site 2 — Phases mode    (TierNodeId::Phase)
    Trigger: prev_phase_id != Some(cur_loc.phase_id)
    Performs: emit_started + record_verdict(allow), immediately
              after the existing PhaseNodeReady emit.
  Site 3 — Steps mode     (TierNodeId::Step)
    Trigger: prev_step_id != Some(cur_loc.step_id)
    Performs: emit_started + record_verdict(allow), immediately
              after the existing StepNodeReady emit.
  Site 4 — BabySteps mode (TierNodeId::BabyStep)
    Trigger: every baby-step iteration
    Performs: emit_started + record_verdict(allow), immediately
              after the existing BabyStepReady emit.
  Site 5 — BabyStepDetail mode (TierNodeId::BabyStep — DECISION-CASCADE
                                re-uses the BabyStep id; see
                                auditor_mode.rs:126-131 and
                                B-41-decisions.md DECISION-CASCADE).
    Trigger: every baby-step iteration, after site 4.

The Plan mode (legacy single-cycle integration audit) is intentionally
out of scope — it has its own pre-B-41 path and is retained for
back-compat. C-12 wires the 5 NEW B-41 per-level modes only.

DETERMINISTIC_CHECK retargeting (DRIFT-5): the grep targets in the C-12
plan (items 1-5) hard-code pipeline_tree_navigator.rs. With the call
sites landing in task_runner.rs per DRIFT-2 resolution (b), the
effective deterministic checks become:

  rg "per_level_audit_dispatch|emit_audit_started|record_audit_verdict" \
     .elai_cc/crates/orchestrator/src/task_runner.rs
  rg "AuditorMode::(Concepts|Phases|Steps|BabySteps|BabyStepDetail)" \
     .elai_cc/crates/orchestrator/src/task_runner.rs
  rg -c "emit_audit_started" .elai_cc/crates/orchestrator/src/task_runner.rs
  rg -c "record_audit_verdict" .elai_cc/crates/orchestrator/src/task_runner.rs
  rg "CascadeOutcome::(Pass|RespawnOnce|Cascade|EscalateToIdea)" \
     .elai_cc/crates/orchestrator/src/task_runner.rs

==================================================================
DECISION-FINDINGS (resolves DRIFT-1)
==================================================================

get_per_level_verdicts returns LevelAuditVerdictJson rows with
`findings: Vec::new()` populated from SSB metadata only.

Rationale: PerLevelAuditVerdictPayload carries `findings_count: u32`
(ssb.rs:5306-5316), NOT the full Finding list. The full Findings live
on disk at `workflow/sessions/{sid}/audits/{mode}/{node_id}.md`
(persisted by auditor_mode::persist_verdict). Reconstructing the
finding list from SSB alone is impossible.

Two resolutions were considered:
  (a) Return empty findings — dashboard popover renders
      "No findings recorded for this verdict." (already supported at
      AuditTimelineWindow.svelte:174). All other dashboard surfaces
      (mode-coloured dots, cascade arrows, counts) depend only on
      metadata and render correctly.
  (b) Read from disk per row — adds file I/O per row to a Tauri
      command and requires project_root threading; slower; matches the
      "full timeline" plan acceptance more literally.

Resolution chosen: (a). The orchestrator pre-approved DRIFT-1 with the
note "Dashboard popover already supports empty state."

Follow-up (deferred, not blocking C-12 acceptance): if a future user
study shows the empty popover is a UX gap, switch to resolution (b).
Required surface change: the helper would accept `state.project_root`
+ session_id and parse the Markdown at `{root}/workflow/sessions/{sid}/
audits/{mode}/{node_id}.md` via a deterministic regex. Code surface
estimated at ~60 LOC; no schema changes required.

==================================================================
DECISION-FRAMED (resolves DRIFT-4)
==================================================================

FramedIdea forwarding to the Concepts auditor is implemented as
"IdeaFramed SSB row presence assertion + WARN log on absence" (per
the orchestrator decision). The dispatcher itself is not blocked when
IdeaFramed is absent — the Concepts auditor agent already reads
FramedIdea from the SSB row directly (auditor.md:222), so the
"forwarding" the plan calls for is observability, not a typed argument.

Implementation: at Site 1 (Concepts), task_runner reads
SsbActionType::IdeaFramed via writer.query_by_type and emits a single
WARN log per task when no IdeaFramed row exists. The dispatcher
proceeds regardless — this matches the plan acceptance: "un-framed
task emits WARN + fallback (no hard abort)".

No struct injection was performed (DRIFT-4 explicitly forbids inventing
an AuditorInvocation struct in this phase). When a future B-N task
introduces a typed AuditorInvocation context, the FramedIdea field
can be added there without disturbing this C-12 wiring.

==================================================================
DECISION-MODE-PROMPTS (resolves DRIFT-3)
==================================================================

auditor.md MODE DISPATCH section (lines 199-267) has all 6
mode-specific REVIEW prompt blocks present with substantive content
(BLOCK/WARN categories cited, rule cluster references, defer-to-lower
statements). Current sizes: Concepts 10 LOC, Phases 8 LOC, Steps 8 LOC,
BabySteps 9 LOC, BabyStepDetail 10 LOC, Plan 2 LOC.

Per the orchestrator decision and DRIFT-3 recommendation, the 6
existing REVIEW sections are accepted as substantive even though
shorter than the plan's aspirational "~80 LOC each". The
checklists carry concrete C-* finding categories, severity tags, and
explicit defer-to-lower-tier statements; expanding to 80 LOC each
would add ~480 net new tokens to every per-tier audit-agent
invocation without adding new constraints.

No changes to auditor.md in C-12.

Follow-up (deferred, not blocking): C-13 or later may expand the
shortest sections with concrete worked-example findings if dogfooding
reveals a recall gap.

==================================================================
DECISION-SESSION (resolves DRIFT-6)
==================================================================

get_per_level_verdicts filters rows by the latest session_id
discovered across the verdict + cascade row families. When the
dashboard sends the sentinel `"active"` (AuditTimelineWindow.svelte:31
default) or an empty string, the helper resolves to "latest session
that produced any audit row". An explicit caller-supplied session id
is honoured verbatim — this is a future-proof hook for per-task
scoping once the TaskRunner exposes a real task→session mapping to
the dashboard layer.

This mirrors the established B-40 pattern used by
get_idea_framing_state (commands.rs:6649-6654).

==================================================================
DECISION-CASCADE-ROUTING (deferred)
==================================================================

C-12 ships SYNTHETIC `LevelAuditVerdict::allow(...)` verdicts at every
emit site. The agent layer that produces REAL verdicts (with BLOCK
findings → RespawnOnce → Cascade → EscalateToIdea routing) is the
next slice; the orchestrator-side bubble-up integration is gated on
that agent layer existing.

The synthetic-Allow path always returns CascadeOutcome::Pass at the
dispatcher; the call sites pattern-match all 4 CascadeOutcome variants
exhaustively (no `_` catch-all), so when the agent layer lands and
the Allow-only contract is lifted, the only edit required is to
replace the synthetic verdict construction with the agent-returned
verdict at each site.

This is documented at the helper function `run_per_level_audit_hook`
and the contract test
`contract_five_emit_sites_produce_five_started_and_five_verdict_rows`
asserts the synthetic-Allow invariant (5 verdict rows, 0 cascade rows)
so the bubble-up wiring slice can be added later without breaking the
C-12 acceptance matrix.

==================================================================
Risks acknowledged
==================================================================

R1 — Verdict storm in long-running tasks. Each baby-step iteration
emits 2 verdict rows (BabySteps + BabyStepDetail). At
MAX_BABY_STEPS_PER_TASK=10_000 that's 20_000 verdict rows per task in
the worst case. The SSB has a 500-row scan cap in
`rows_with_session_by_action_type` (ssb_viewer.rs:4141) so the dashboard
read is bounded; the on-disk SQLite cost is ~32 bytes per row × 20K =
640 KB per task at the upper bound. Acceptable for MVP. A future
optimisation could collapse synthetic-Allow rows behind a feature flag.

R2 — IdeaFramed query on every concept boundary. The query is a single
SQLite SELECT against an indexed action_type column; cost <1 ms in the
hot path. The `idea_framed_warn_emitted` flag ensures the WARN log is
emitted at most once per task to avoid log spam.

R3 — Contract test runtime ordering. The C-12 contract test asserts
seq ordering between started + verdict rows. The SeededClock fixture
guarantees deterministic seq assignment under the multi-thread tokio
runtime since SSB writes are serialized through the writer pool.

==================================================================
Files touched
==================================================================

MODIFIED:
- .elai_cc/crates/orchestrator/src/task_runner.rs
    (imports: auditor_mode + per_level_audit_dispatch types;
     idea_framed_warn_emitted local; 5 audit hook call sites;
     run_per_level_audit_hook helper function)
- .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs
    (compute_per_level_verdicts helper; rewired get_per_level_verdicts
     to call the helper instead of returning Vec::new())

ADDED:
- .elai_cc/crates/contracts/tests/c12_navigator_audit_wiring_contract.rs
    (6 test cases covering 10 contract checks)

CREATE:
- workflow/reasoning/C-12-decisions.md (this file)

UNCHANGED:
- .claude/agents/auditor.md (DECISION-MODE-PROMPTS — accepted as-is)
- .elai_cc/crates/orchestrator/src/per_level_audit_dispatch.rs
    (dispatcher was already complete and correct per B-41)
- .elai_cc/crates/orchestrator/src/auditor_mode.rs
    (types complete per B-41)
- .elai_cc/crates/orchestrator/src/ssb.rs (no SCHEMA_VERSION bump)
- .elai_cc/crates/dashboard-v2/src/lib/windows/AuditTimelineWindow.svelte
    (component bug-free; data shape match — no prop changes required)
- .elai_cc/crates/dashboard-v2/src/lib/api/stub.ts (TS invoke wrapper
    unchanged — Tauri snake_case ↔ camelCase mapping handles `task_id`)
- .elai_cc/crates/dashboard-v2/src/lib/api/types.ts (DTOs unchanged)
