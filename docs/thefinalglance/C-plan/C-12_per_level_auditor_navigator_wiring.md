# C-12: Per-Level Auditor Navigator Wiring
Objective: wire per_level_audit_dispatch::record_verdict into
pipeline_tree_navigator.rs at the 5 tier-completion points; confirm
auditor.md MODE DISPATCH has 6 mode-specific REVIEW prompt sections;
confirm FramedIdea is forwarded to mode=Concepts invocation; emit
per_level_audit_started/per_level_audit_verdict/per_level_audit_cascade
SSB rows at each call site; connect get_per_level_verdicts Tauri command
to live SSB rows so AuditTimelineWindow.svelte renders the full verdict
timeline with cascade arrows.
Estimated effort: 1 session.
Pre-requisites: C-1 and C-2 (SSB action types defined and emitting);
B-41 shipped (verified: per_level_audit_dispatch.rs correct, AuditorMode
6 variants, LevelAuditVerdict struct, 3 SSB action types, AuditTimelineWindow
scaffolded, get_per_level_verdicts registered but returning empty Vec).
Parallel-safe with: C-11, C-13, C-14, C-15.

## Context

B-41 shipped the full type system, dispatcher, and dashboard surface for
Per-Level Auditor Modes (verification report: MOSTLY_DONE). The one gap:
pipeline_tree_navigator.rs has zero B-41 references. The dispatcher module
(per_level_audit_dispatch.rs) is complete and correct — emit_started,
record_verdict, CascadeOutcome, respawn-once guard, cascade SSB row —
but the navigator never calls it. As a result:

- No per_level_audit_started SSB row is emitted before any auditor
  invocation, so the dashboard timeline never fires.
- No per_level_audit_verdict row is written after any tier completes,
  so AuditTimelineWindow.svelte always shows the "no verdicts" empty state.
- get_per_level_verdicts always returns Vec::new() because the Tauri
  command reads live SSB rows, and none were ever written.
- FramedIdea (B-40) is never forwarded to the Concepts-mode auditor,
  so the first audit tier lacks grounded context.
- auditor.md MODE DISPATCH inner prompt content (~80 LOC per mode) has
  only its section header confirmed; full content is unverifiable without
  reading and cross-referencing against the navigator call sites.

Five tier-completion points in the navigator produce "all resolved"
signals and are natural insertion sites:
  - all_phases_in_concept_resolved -> post-Concepts (mode=Concepts)
  - all_steps_in_phase_resolved    -> post-Phases   (mode=Phases)
  - all_baby_steps_in_step_resolved inside step loop -> post-Steps (mode=Steps)
  - BabyStep batch authored        -> post-BabySteps (mode=BabySteps)
  - BabyStepDetail authored        -> post-BabyStepDetail (mode=BabyStepDetail)

record_verdict returns CascadeOutcome; the navigator already has bubble-up
logic (test_baby_step_failure_threshold_triggers_bubble_up). C-12 routes
CascadeOutcome::RespawnOnce, Cascade, and EscalateToIdea into the
existing bubble-up path — no new types required.

get_per_level_verdicts currently returns Ok(Vec::new()) (commands.rs:6673).
Connecting it to live SSB rows requires the AppState's Ssb handle and a
filter on action_type = "per_level_audit_verdict". The B-36 pattern
(list_runtimes, test_runtime) shows the established approach for threading
AppState into Tauri commands.

## Reads

- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-41_per_level_auditor_modes.report.md (action list)
- .elai_cc/crates/orchestrator/src/per_level_audit_dispatch.rs (complete dispatcher)
- .elai_cc/crates/orchestrator/src/pipeline_tree_navigator.rs (5 insertion points at lines 768, 793, 818, 881, 904, 925)
- .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs:6665-6674 (empty Vec stub)
- .elai_cc/crates/dashboard-v2/src/lib/windows/AuditTimelineWindow.svelte (already wired to getPerLevelVerdicts)
- .claude/agents/auditor.md (MODE DISPATCH section; confirm inner prompt content per mode)

## Tasks

1. **Read auditor.md MODE DISPATCH section** (lines 199+). Verify 6
   mode-specific REVIEW prompt blocks exist (~80 LOC each: Concepts,
   Phase, Step, BabyStep, BabyStepDetail, Cosmos). If any block is
   missing or is a placeholder, add it now. Confirm the Concepts block
   accepts a `framed_idea` context field.

2. **Confirm FramedIdea propagation path**. Grep for FramedIdea in
   idea_framing.rs and task_runner.rs. Verify the struct is available
   at the point where mode=Concepts auditor is invoked. If it is not
   forwarded, add the field to the Concepts auditor invocation context
   in pipeline_tree_navigator.rs.

3. **Wire emit_started at 5 call sites** in pipeline_tree_navigator.rs.
   Before each Auditor* agent spawn, call:
     per_level_audit_dispatch::emit_started(&ssb, agent_id, superstep, mode, tier_node_id)
   Use the TierNodeId value already computed at that branch point
   (ConceptId/PhaseId/StepId/BabyStepId/BabyStepDetailId). Log
   emit failure as warn! — do not abort the pipeline.

4. **Wire record_verdict at 5 tier-completion points**. After each
   Auditor* agent returns its LevelAuditVerdict, call:
     per_level_audit_dispatch::record_verdict(&ssb, agent_id, superstep, verdict)
   Pattern-match CascadeOutcome:
     - Pass               -> continue navigator as-is
     - RespawnOnce        -> respawn the same Auditor* with updated verdict;
                            merge into existing bubble-up path
     - Cascade            -> bubble to parent tier decomposer; merge into
                            existing all_*_resolved bubble-up path
     - EscalateToIdea     -> route to B-40 FRAMING retry; emit ESCALATE log

5. **Connect get_per_level_verdicts to live SSB rows**. Thread AppState
   into the command (B-36 pattern). Query ssb.scan_rows() filtered to
   action_type IN ["per_level_audit_verdict", "per_level_audit_cascade"]
   for the given task_id. Deserialize into Vec<LevelAuditVerdictJson>.
   Replace Ok(Vec::new()) stub. No new SSB fields required.

6. **Verify AuditTimelineWindow renders correctly** with live data.
   The component already reads verdicts from getPerLevelVerdicts and
   renders cascade_to_parent arrows (confirmed at lines 91, 134, 155).
   Confirm no prop changes are needed after command is connected.

7. **Contract test: navigator_audit_wiring_contract.rs**. Write a
   contract test that:
     - Builds a minimal NavigatorTree (3 concepts, 1 phase each, 2 steps,
       2 baby steps per step)
     - Runs the navigator to tier-completion for all 5 tiers
     - Asserts 5 distinct per_level_audit_verdict SSB rows exist
     - Asserts each row has the correct mode field for its tier
     - Asserts per_level_audit_started rows precede their corresponding
       verdict rows (seq ordering)

8. **Contract test: audit_timeline_command_contract.rs**. Write a
   contract test that:
     - Seeds the SSB with 5 per_level_audit_verdict rows (one per mode)
       plus 2 per_level_audit_cascade rows
     - Calls get_per_level_verdicts logic directly
     - Asserts the returned Vec has exactly 5 LevelAuditVerdictJson entries
     - Asserts cascade_to_parent=true entries match the 2 seeded cascade rows

## WIRING_CONTRACT

produces_symbols:
  - pipeline_tree_navigator.rs: 5 call sites to emit_started + record_verdict
  - pipeline_tree_navigator.rs: CascadeOutcome pattern-match routing
  - commands.rs: get_per_level_verdicts reads live SSB rows (Vec<LevelAuditVerdictJson>)
  - tests/navigator_audit_wiring_contract.rs
  - tests/audit_timeline_command_contract.rs
consumes_symbols:
  - per_level_audit_dispatch::emit_started (exists, pub async fn)
  - per_level_audit_dispatch::record_verdict (exists, pub async fn)
  - per_level_audit_dispatch::CascadeOutcome (exists: Pass/RespawnOnce/Cascade/EscalateToIdea)
  - LevelAuditVerdict (exists in auditor_mode.rs:203-210)
  - AuditorMode::{Concepts,Phases,Steps,BabySteps,BabyStepDetail} (exists)
  - Ssb (exists; AppState already holds handle per B-36 pattern)
  - B-40 FramedIdea struct (exists in idea_framing.rs)
reachable_from:
  - pipeline_tree_navigator.rs tier-completion branches (5 insertion points)
  - AuditTimelineWindow.svelte via getPerLevelVerdicts Tauri command
  - dashboard Auditor Timeline tab
stub_ban:
  - get_per_level_verdicts MUST NOT return Ok(Vec::new()) after this task
  - emit_started failures MUST be warn! not propagated (pipeline must not abort)
  - record_verdict errors MUST surface as ESCALATE, not silent swallow
frontend_pair: AuditTimelineWindow.svelte + get_per_level_verdicts Tauri command
observable_effect:
  trigger: elai /task --mode cloud "build a 3-concept project: a HTTP server,
           JSON parser, and tests; in Rust" (cloud Sonnet 4.6; task large enough
           that the navigator runs all 5 tiers)
  expect_ssb: 5 distinct per_level_audit_verdict rows (one per tier:
              Concepts/Phases/Steps/BabySteps/BabyStepDetail); each row has
              mode field matching its tier; per_level_audit_started rows
              precede each verdict row in seq order
  expect_command: get_per_level_verdicts returns Vec with 5+ entries
  expect_artifact: AuditTimelineWindow.svelte renders timeline with 5 mode
                   markers; cascade arrows visible where cascade_to_parent=true;
                   mode-coloured dots match modeColour mapping in component

## DETERMINISTIC_CHECK

```bash
# 1. Navigator imports per_level_audit_dispatch
rg "per_level_audit_dispatch" .elai_cc/crates/orchestrator/src/pipeline_tree_navigator.rs

# 2. All 5 modes referenced in navigator
rg "AuditorMode::(Concepts|Phases|Steps|BabySteps|BabyStepDetail)" \
  .elai_cc/crates/orchestrator/src/pipeline_tree_navigator.rs

# 3. emit_started called 5 times
rg -c "emit_started" .elai_cc/crates/orchestrator/src/pipeline_tree_navigator.rs
# Expected: 5

# 4. record_verdict called 5 times
rg -c "record_verdict" .elai_cc/crates/orchestrator/src/pipeline_tree_navigator.rs
# Expected: 5

# 5. CascadeOutcome arms all handled
rg "CascadeOutcome::(Pass|RespawnOnce|Cascade|EscalateToIdea)" \
  .elai_cc/crates/orchestrator/src/pipeline_tree_navigator.rs

# 6. get_per_level_verdicts no longer returns empty Vec
rg "Vec::new\(\)" .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs
# Expected: zero matches on get_per_level_verdicts function body

# 7. Contract tests exist
test -f .elai_cc/crates/contracts/tests/navigator_audit_wiring_contract.rs
test -f .elai_cc/crates/contracts/tests/audit_timeline_command_contract.rs

# 8. Compile check
cargo check -p elai_orchestrator -p elai_dashboard_v2 2>&1 | grep "^error" | wc -l
# Expected: 0

# 9. Navigator wiring contract passes
cargo test -p elai_contracts -- navigator_audit_wiring 2>&1 | grep "test result"
# Expected: test result: ok. 1 passed

# 10. Timeline command contract passes
cargo test -p elai_contracts -- audit_timeline_command 2>&1 | grep "test result"
# Expected: test result: ok. 1 passed
```

## ACCEPTANCE

- [ ] auditor.md MODE DISPATCH: 6 mode-specific REVIEW prompt blocks
      confirmed present with substantive content (~80 LOC each);
      Concepts block accepts framed_idea context field
- [ ] FramedIdea forwarded to mode=Concepts auditor invocation; un-framed
      task emits WARN + fallback (no hard abort)
- [ ] emit_started called at all 5 tier-completion points before Auditor*
      spawn; emit failure is warn!, not pipeline abort
- [ ] record_verdict called at all 5 tier-completion points after Auditor*
      returns; CascadeOutcome fully pattern-matched (no _ catch-all)
- [ ] CascadeOutcome::RespawnOnce routes to Auditor* respawn via existing
      bubble-up path; no double-respawn (monotonic guard enforced)
- [ ] CascadeOutcome::Cascade bubbles to parent tier decomposer
- [ ] CascadeOutcome::EscalateToIdea routes to B-40 FRAMING retry path
- [ ] get_per_level_verdicts returns live SSB rows (not Vec::new());
      filtered to action_type per_level_audit_verdict + per_level_audit_cascade
- [ ] AuditTimelineWindow.svelte renders 5 mode-coloured markers with
      correct modeColour values; cascade arrows visible where
      cascade_to_parent=true; no prop changes required
- [ ] navigator_audit_wiring_contract: 5 verdict rows, correct mode per tier,
      started-before-verdict seq ordering — all pass
- [ ] audit_timeline_command_contract: 5 entries returned, cascade entries
      match seeded rows — all pass
- [ ] cargo check -p elai_orchestrator -p elai_dashboard_v2: 0 errors
- [ ] Cloud smoke test: elai /task --mode cloud "build a 3-concept project:
      a HTTP server, JSON parser, and tests; in Rust" produces 5 distinct
      per_level_audit_verdict SSB rows; each has correct mode field;
      get_per_level_verdicts returns >=5 entries; AuditTimelineWindow shows
      5 markers with cascade arrows where applicable

## Files touched

- MODIFY: .elai_cc/crates/orchestrator/src/pipeline_tree_navigator.rs
  (5 emit_started + 5 record_verdict call sites; CascadeOutcome routing;
  FramedIdea forwarding to Concepts invocation)
- MODIFY: .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs
  (get_per_level_verdicts: replace Vec::new() with live SSB scan)
- MODIFY: .claude/agents/auditor.md
  (if MODE DISPATCH inner prompt blocks are missing or placeholder; add
  6 mode-specific REVIEW prompt sections; no-op if already present)
- ADD: .elai_cc/crates/contracts/tests/navigator_audit_wiring_contract.rs
- ADD: .elai_cc/crates/contracts/tests/audit_timeline_command_contract.rs
- CREATE: workflow/reasoning/C-12-decisions.md

## Next

C-13 (IPC dispatcher production swap) or C-11 (sandbox completeness),
whichever unblocks the most C-plan integration surface.
