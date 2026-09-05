# Rules Engine + Hierarchical Plan Dashboard Wiring Audit
Date: 2026-04-25
Auditor stance: HOSTILE
Scope: Phase 3F (rules engine — three-tier self-improving) and
       Phase 3G (hierarchical PlanSkeleton + /plan command)
       — surfaced through Tauri dashboard backend
       (F:\.ELAI_workflow\.elai_cc\crates\dashboard\src\) and Svelte
       frontend (F:\.ELAI_workflow\.elai_cc\crates\dashboard\src-frontend\src\).

================================================================
OVERALL VERDICT: BARELY-WIRED RULES, ENTIRELY-INVISIBLE PLAN SKELETON
================================================================

Both phases ship rich backend surfaces that the dashboard mostly
ignores.

For 3F (rules engine), the dashboard exposes ONE shallow signal —
five lifecycle counts grouped from `rules.db` — through
`get_rule_lifecycle_stats`. A second, deeper command
(`get_rule_detail`) was added to lib.rs (lib.rs:1471) but is NEVER
invoked from any Svelte component (grep across `src-frontend/` for
`get_rule_detail` returns ZERO matches). The Layer-1A proposals
panel renders three fields (`table` / `new_row_summary` /
`fpr_estimate`) of pending rule changes, but routes through a
generic Layer1aProcessor — not the rules engine's own self-discovery
proposals from `violation_patterns` / `failure_category_proposals`
tables. Lifecycle transitions (PROPOSED→CANDIDATE→SHADOW→ACTIVE→
RETIRED), per-rule trigger counts, FPR/TPR/CCR, BLOCK/WARN/ALLOW
verdicts, deprecated_patterns matches, and predicate types are all
backend-real but ZERO of them surface. Of the seven predicate types
the engine supports, the dashboard renders no predicate type
distinctly anywhere — the user cannot tell a VERSION_GATE BLOCK
from a GREP_PATTERN WARN by looking at the UI.

For 3G (PlanSkeleton), the situation is worse. The dashboard never
opens any `plans/*.toml` file, never calls `PlanSkeleton::from_toml`
or `read_from_file`, never renders a phase DAG, never shows
`extract_contracts` output, never surfaces the `/plan` command,
never displays staleness warnings, and never reacts to
`SsbPayload::PlanUpdate` / `StubReference` rows beyond a one-word
breadcrumb in the activity feed. The `WindowType.PLAN` slot is wired
to `Workspace.svelte` (WindowRegistry.ts:27), which in turn wraps
`PlanTree.svelte` — and `PlanTree` polls `get_phase_progress`, which
reads `PhaseComplete` SSB rows (ssb_viewer.rs:790), NOT the
`PlanSkeleton` tree. The newer `PlanWindow.svelte` exists but is
EXPLICITLY UNREGISTERED (PlanWindow.svelte header comment: "Registration
in WindowRegistry.ts is intentionally deferred"). The
`get_decomposition_tree` viewer it depends on always returns an empty
snapshot (ssb_viewer.rs:1383-1391 — comment: "Returns an empty
snapshot today. The orchestrator does NOT yet persist a
`concept_decomposition` SSB row"). The user's "Plan" UI is a
dead Concept→Phase→Step→BabyStep tree mounted nowhere, plus a
legacy phase-completion timeline mislabeled as the plan.

The 3F audit (phase-3F.md) graded the engine ~88% complete; the
3G audit (phase-3G.md) graded the plan PASS-WITH-CAVEATS.
Neither audit measured frontend wiring. This audit closes that gap.

================================================================
PHASE 3F — RULES ENGINE (three-tier, self-improving)
================================================================

----------------------------------------------------------------
Backend signals available in principle (per phase-3F audit)
----------------------------------------------------------------
- 9 SQLite tables in `rules.db` (schema.rs:34-159): rules, rule_metrics,
  contract_tests, violation_patterns, rule_transitions,
  deprecated_patterns, interrogation_trees,
  failure_category_proposals, schema_version.
- 5-state lifecycle per rule: PROPOSED / CANDIDATE / SHADOW / ACTIVE /
  RETIRED, audited per-transition with reason + approver + tp/fp/causal/
  spurious/fpr/tpr/ccr columns (rule_transitions table).
- 7 predicate types: AstPattern, GrepPattern, SchemaRequired,
  MetricGate, Composite, GraphReachability, VersionGate (predicate.rs:101).
- Verdicts: Allow / Warn / Block / Escalate(Auditor|Human|NextTierRule).
- Per-rule metric columns: tp, fp, causal_catches, spurious_catches,
  edct_pass, edct_fail, confidence_score (rule_metrics table).
- 21 seeded deprecated_patterns (cargo: blake2→blake3, rand 0.8→0.9,
  log→tracing, getrandom 0.2→0.3, etc.; pip; npm). schema.rs:167-238.
- ~39 SHADOW seed rules (engineering.toml=12 + web.toml=27).
- T1Hook + T2Hook in plugins/tiered_analysis.rs invoke RuleEngine on
  every tool dispatch with measured latencies.
- Self-discovery proposals (`try_propose_rule` from
  violation_patterns when occurrences ≥ 3, blake3 pattern hash).
- Failure-category proposals (failure_category_proposals table).
- Interrogation trees seed data in elai_core::interrogation_tree.

----------------------------------------------------------------
Backend signal production reality
----------------------------------------------------------------
- Rules engine IS live in production: lib.rs:893,951 wires the
  engine + load_seed_rules_from_dir TWICE (initial load + hook engine).
  T1Hook and T2Hook in tiered_analysis.rs evaluate rules on every
  hook event. Verdicts feed back into HookVerdict::{Allow,Warn,Block}.
- Dashboard process opens `rules.db` read-only on every drawer call
  (commands.rs:3042 open_rules_db_readonly).
- Layer-1A processor IS instantiated and persists pending proposals
  to a `pending_proposals` table inside the same `rules.db` file
  (self_mod_layer1a.rs:500-543). Confirmed in lib.rs:566-594 where
  `rules_db_path = db_path.parent().join("rules.db")` is shared
  with `layer1a_processor` and `state.rules_db_path`.
- ZERO production callers exist for:
  * `get_rule_detail`             — 0 frontend invokes
  * `get_interrogation_tree`      — invoked by ReasoningDrawer only
                                    (drawer mounted in ActivityWindow,
                                    collapsed by default)
  * any per-rule metrics surface  — no command, no panel
  * any predicate-type surface    — no command, no panel
  * any rule_transitions surface  — no command, no panel
  * deprecated_patterns table     — no command, no panel surface
                                    (only ActivityEntry tag-handling
                                    for SSB Finding rows whose
                                    `category` happens to be
                                    "deprecated_pattern" / "version_gate")

----------------------------------------------------------------
Dashboard wiring — what reaches the UI
----------------------------------------------------------------

(A) RulesDrawer.svelte — `get_rule_lifecycle_stats`
    File: F:\.ELAI_workflow\.elai_cc\crates\dashboard\src-frontend\src\drawers\RulesDrawer.svelte
    - 102 LOC.
    - On expand (toggle handler line 47), fires ONE invoke
      `get_rule_lifecycle_stats` (line 37). NO polling, NO refresh,
      NO live update. If a rule transitions PROPOSED→ACTIVE 30s
      later, the drawer keeps showing the stale stats until the user
      collapses + re-expands.
    - Backend (commands.rs:3054-3087) runs `SELECT status, COUNT(*)
      FROM rules GROUP BY status` against rules.db read-only. This
      IS real data — verified at commands.rs:3061 read-only handle
      open + line 3062 prepare. Fields populated: proposed,
      candidate, shadow, active, retired, unknown, total.
    - Header shows `{stats.active} active · {stats.shadow} shadow ·
      {stats.total} total` (line 63). Body lists each lifecycle
      bucket with count (lines 71-80).
    - Hosted in TWO places via composition:
        ActivityWindow.svelte:45  `<RulesDrawer {windowProps} />`
        ChatWindow.svelte:470     `<RulesDrawer />` (no windowProps)
      Collapsed by default. User must click to load.
    - VERDICT: SURFACED but THIN. Live counts only. No per-rule
      drilldown despite backend `get_rule_detail` being registered.

(B) `get_rule_detail` — DEAD COMMAND
    File: F:\.ELAI_workflow\.elai_cc\crates\dashboard\src\commands.rs:3089-3150
    - Registered: lib.rs:1472 (alongside `get_rule_lifecycle_stats`).
    - SQL: SELECT rule_id, domain, tier, status, title, description
      (truncated to 4000 bytes), severity, mutable, version, source,
      created_at, promoted_at FROM rules WHERE rule_id = ?1.
    - Real read from rules.db. Returns Option<RuleDetail>.
    - Frontend invokers: ZERO. Grep across `src-frontend/` for
      `get_rule_detail` returns no matches.
    - VERDICT: DEAD. Command is plumbed end-to-end but nothing in
      the UI calls it. The user can never click a lifecycle bucket
      to see WHICH rules are in SHADOW vs ACTIVE, who proposed them,
      severity, title, description, version, or promoted_at.

(C) ActivityFeed.svelte — `get_rule_violations` strip
    File: F:\.ELAI_workflow\.elai_cc\crates\dashboard\src-frontend\src\ActivityFeed.svelte
    - Polls `get_rule_violations({stepId: ""})` at the same cadence
      as the main feed (POLL_MS, line 105-107). Returns up to 200
      Finding rows whose `audit_step` matches
      `POST_EXECUTION_RULES_AUDIT_STEP` (ssb_viewer.rs:864-...).
      This IS Batch-4B post-execution rules — the engine running
      AFTER a baby-step completes, not the realtime T1/T2 hook
      verdicts during a tool call.
    - Tallies severity buckets (`error` vs `warning`, lines 116-121).
    - When rule violations exist AND user clicks the strip header,
      the feed narrows to rows whose summary contains
      `post_execution_rules` (lines 145-151).
    - VERDICT: PARTIAL. One backend signal (post-execution audit)
      surfaces. Realtime hook verdicts (T1/T2 BLOCK/WARN/ALLOW)
      do NOT have a dedicated panel. The `tiered_analysis` SSB
      action_type IS rendered as one Activity row per invocation
      (ActivityEntry.svelte:308-313 — "Block=fail, Warn=warn,
      Allow=info") but with no per-rule attribution beyond the
      summary string.

(D) ActivityEntry.svelte — tag rendering
    Lines 71, 992 ("deprecated_patterns"), 549, 559 ("version_gate"),
    1105 ("plan_update"), 1109 ("stub_reference").
    - These are CATEGORY filters used to colourise SSB Finding rows
      whose `category` field happens to be one of these strings.
    - There is NO panel that aggregates "deprecated_pattern" matches
      separately, NO panel that lists active VERSION_GATE
      violations, and NO drilldown to the deprecated_patterns
      table itself.
    - VERDICT: DEAD VISUAL. The taxonomy exists in the renderer
      but no surface aggregates it.

(E) ProposalsPanel.svelte — Layer-1A pending proposals
    File: F:\.ELAI_workflow\.elai_cc\crates\dashboard\src-frontend\src\ProposalsPanel.svelte
    - Polls `get_pending_proposals` every 3000ms (line 24).
    - Renders up to 5 cards per refresh (MAX_VISIBLE = 5).
    - Each card shows: `table`, `pipeline_mode`, `fpr_estimate` as
      "fpr X.X%", occurrences, `new_row_summary` (truncated to 200
      bytes by summarise_row at commands.rs:1214). Accept and
      Reject buttons wired to `apply_proposal_from_user` and
      `reject_proposal`.
    - Backend reads from `Layer1aProcessor::list_pending` which
      pulls from a `pending_proposals` SQLite table inside
      rules.db (self_mod_layer1a.rs:500-512). The SOURCE of these
      proposals is the generic Layer-1A self-modification path
      (auto_from_outcome / manual), NOT the rules-engine
      self-discovery path that populates `violation_patterns` +
      `failure_category_proposals` tables. So while a USER might
      assume "proposals" means "new rules being proposed by the
      engine after seeing 3 violations", the panel actually shows
      generic table-row mutations the orchestrator wants to make.
    - VERDICT: SURFACED but MISLABELED. The panel does work and
      reads real DB rows, but it conflates two distinct proposal
      pipelines (rules-engine self-discovery vs Layer-1A
      auto_from_outcome). User cannot distinguish them.

(F) AntiStalenessDrawer.svelte — explicit STUB
    File: F:\.ELAI_workflow\.elai_cc\crates\dashboard\src-frontend\src\drawers\AntiStalenessDrawer.svelte
    - 73 LOC. Renders three lines: "no data yet" for nightly dep
      audit, baseline harness, deterministic audit.
    - Header literally reads `anti-staleness · deferred`.
    - Comment lines 6-13: "Plan §7 originally listed three
      commands... the Tier 3 partial audit dropped them as ghosts
      because no SSB row type emits dep_audit / baseline_harness /
      deterministic_audit results."
    - VERDICT: PARTIAL/STUB by design. Not a wiring failure — a
      documented deferral. But the user clicking "anti-staleness"
      learns NOTHING about the 21 deprecated_patterns that ARE
      seeded and ARE matched at runtime by VERSION_GATE.

----------------------------------------------------------------
SSB action-type subscriptions (3F-relevant)
----------------------------------------------------------------
Searched: rule_proposed, rule_promoted, RuleProposed, RulePromoted.
Result: ZERO matches in the dashboard frontend or backend.

The rules engine emits NO SSB action_type for "rule proposed" or
"rule promoted". Lifecycle transitions are written into the
`rule_transitions` SQL table (schema.rs:101-116) but never echoed
to SSB. So the activity feed cannot show "rule X promoted to ACTIVE"
or "auditor proposed new GREP_PATTERN" in real time. The only way
the user sees a lifecycle event is to manually re-expand the
RulesDrawer to refetch the bucket counts.

----------------------------------------------------------------
3F WIRING VERDICT TABLE
----------------------------------------------------------------
get_rule_lifecycle_stats        SURFACED  (RulesDrawer, on-expand only)
get_rule_detail                 DEAD       (registered, never invoked)
get_rule_violations             PARTIAL    (post-execution only, in feed)
get_interrogation_tree          INVISIBLE  (no host beyond Reasoning drawer)
deprecated_patterns table       INVISIBLE  (no panel)
rule_transitions table          INVISIBLE  (no panel, no SSB echo)
violation_patterns table        INVISIBLE  (no panel)
failure_category_proposals      INVISIBLE  (no panel)
rule_metrics (FPR/TPR/CCR)      INVISIBLE  (no panel)
predicate type per rule         INVISIBLE  (no surface)
T1/T2 verdict attribution       INVISIBLE  (only Allow/Warn/Block badge
                                            on tiered_analysis row,
                                            no per-rule drilldown)
get_pending_proposals (Layer1A) SURFACED   (ProposalsPanel — but
                                            misnamed; not engine self-
                                            discovery proposals)

================================================================
PHASE 3G — HIERARCHICAL PLAN MANAGEMENT (PlanSkeleton)
================================================================

----------------------------------------------------------------
Backend signals available in principle (per phase-3G audit)
----------------------------------------------------------------
- PlanSkeleton struct: pipeline_id, phases (IndexMap),
  global_constraints, version. 2939 LOC.
- PhaseEntry: phase_id, goal (≤120 chars enforced), inputs, outputs,
  constraints, depends_on, depended_by, version, output_type,
  visual_check, split_depth, summary.
- ContractRef: name, type_signature, invariants.
- ExtractedContracts: input_contracts, output_contracts,
  shared_constraints (computed from `extract_contracts`).
- PhaseDetail: phase_id, skeleton_version, contracts,
  implemented_contracts, declared_interfaces, content, token_count.
- TOML round-trip via `to_toml` / `from_toml` / `read_from_file` /
  `write_to_file` (lines 1004-1023, 1132).
- IndexMap with `serde_seq` preserves insertion order across
  round-trip (line 287).
- Token enforcement: TOKEN_ESTIMATE_THRESHOLD=3600,
  TOKEN_HARD_CAP=4096 (lines 27-28).
- Dependency DAG with Kahn cycle detection (line 329-399).
- propagate_changes BFS over depended_by emitting
  SsbPayload::PlanUpdate per reached phase (line 437-455).
- write_to_file emits SsbPayload::StubReference per phase without
  detail file (line 1098-1126).
- Coherence checks: contract_satisfied / no_undeclared_interfaces /
  version_incremented (coherence_check.rs).
- /plan slash command: SlashCommand::Plan { mode } at
  cli/commands.rs:1159. Handler at cli/main.rs:4572-4619 (no-arg
  list of phases; <phase_id> arg shows extracted contracts;
  fallback "No plan loaded" message).
- Plan loaded at startup via load_plan_if_exists
  (cli/main.rs:5372-5398) scanning `cwd/plans/*.toml`.
- PlanSkeleton woven into context budget at SIX call sites
  (build_phase_context_*, plan_phase_slot_tokens,
  build_phase_spec_content) per the 3G audit.

----------------------------------------------------------------
Backend signal production reality
----------------------------------------------------------------
- PlanSkeleton APIs ARE complete and tested (115 test fns,
  byte-identical TOML round-trip, lifecycle_full_5phase_integration,
  100x determinism harness).
- The `/plan` command is wired in CLI ONLY — it dumps to stdout.
  The dashboard does NOT have a slash-command palette wired to
  backend `SlashCommand::Plan`. Slash palette in
  chat/SlashPalette.svelte (per phase-4K audit) exists but is not
  wired to PlanSkeleton.
- SsbPayload::PlanUpdate IS emitted by `write_to_file` and
  `propagate_changes`. SsbPayload::StubReference IS emitted by
  `write_to_file`. Both reach the SSB.
- BUT: nothing in the dashboard project reads
  `plans/*.toml` directly, instantiates a PlanSkeleton, or calls
  `extract_contracts`. Grep across
  `F:\.ELAI_workflow\.elai_cc\crates\dashboard\` for `PlanSkeleton`
  returns ONE match — a comment in commands.rs:21 noting the name
  collision between `elai_orchestrator::PhaseDetail` (the
  PlanSkeleton struct) and the dashboard's own `PhaseDetail`
  (which is a different type used by `get_phase_detail`).
- Grep for `read_plan_skeleton`, `from_toml`, or
  `plans/*.toml`: ZERO matches in dashboard.

----------------------------------------------------------------
Dashboard wiring — what reaches the UI
----------------------------------------------------------------

(A) WindowType.PLAN routing
    File: F:\.ELAI_workflow\.elai_cc\crates\dashboard\src-frontend\src\shell\WindowRegistry.ts:27
    - `[WindowType.PLAN]: Workspace as Component<any>`
    - Workspace.svelte mounts PlanTree on the left + intervention
      cards + ProposalsPanel + ActivityFeed on the right.
    - The newer `PlanWindow.svelte` (which DOES try to render a
      Concept→Phase→Step→BabyStep tree) is NOT registered. From
      its own header comment (PlanWindow.svelte:7-8): "Registration
      in WindowRegistry.ts is intentionally deferred to a follow-up
      patch (parallel-track non-overlap with Track A)." The follow-up
      never landed. The user clicking the "plan" sidebar icon
      (Sidebar.svelte:21) gets Workspace, not PlanWindow.
    - VERDICT: WindowType.PLAN routes to a phase-completion
      timeline, NOT to the PlanSkeleton tree. Misnamed by routing.

(B) PlanTree.svelte — `get_phase_progress` + `step_progress`
    File: F:\.ELAI_workflow\.elai_cc\crates\dashboard\src-frontend\src\PlanTree.svelte
    - Polls `get_phase_progress` every 1500ms. On phase expand,
      polls `step_progress({phaseId})`.
    - Backend (commands.rs:74-78, 330-334) reads
      `SsbViewer::phase_progress` (ssb_viewer.rs:790-815), which
      iterates `PhaseComplete` SSB rows. Each row produces a
      PhaseStatus { phase_id, status: fsm_state, step_count =
      completed_count = inner.test_outcomes.len() }.
    - This IS NOT a PlanSkeleton — it is the historical timeline
      of phases the orchestrator has finished. A phase in the
      PlanSkeleton TOML that has NEVER run (e.g. a future phase the
      user just authored) will NOT appear here. A phase that
      depends on another phase has NO depends_on / depended_by
      arrow rendered. There is no DAG, no ContractRef, no
      ExtractedContracts, no global_constraints, no skeleton.version.
    - VERDICT: SURFACED for COMPLETED phases only. Renders a
      timeline-as-list, not the PlanSkeleton tree.

(C) PlanWindow.svelte + `get_decomposition_tree` — DEAD
    File: F:\.ELAI_workflow\.elai_cc\crates\dashboard\src-frontend\src\windows\PlanWindow.svelte
    - 273 LOC. Polls `get_decomposition_tree({projectId, taskId})`
      every 1500ms. Renders a Concept→Phase→Step→BabyStep tree
      with click-to-fetch detail panes via `get_phase_detail` and
      `get_baby_step_detail`.
    - Backend `get_decomposition_tree` calls
      `SsbViewer::decomposition_tree` (ssb_viewer.rs:1383-1391):
      "Returns an empty snapshot today. The orchestrator does NOT
      yet persist a `concept_decomposition` row...the dashboard
      renders a 'no decomposition yet' placeholder."
    - Backend `get_phase_detail` (ssb_viewer.rs:1412-1447):
      "Today the SSB persists no row carrying description /
      expected_outcome / dependencies. The implementation confirms
      the phase is real by scanning the most recent
      [PHASE_DETAIL_SCAN_LIMIT] phase_complete rows...returns a
      PhaseDetail::empty shell. When no matching row exists...
      NotFound." Phases that have not yet emitted phase_complete
      = NotFound. Phases that have = empty shell with no
      description, no expected_outcome, no dependencies.
    - Backend `get_baby_step_detail` (ssb_viewer.rs:1456-1478):
      "No BabyStepSpec persistence yet — every lookup is a
      NotFound." Hard-coded NotFound for ALL baby_step_id values.
    - PlanWindow itself is NEVER MOUNTED. WindowRegistry.ts has no
      `[WindowType.PLAN]: PlanWindow` entry; PLAN routes to
      Workspace. PlanWindow could be made the default with a
      one-line WindowRegistry change but doing so would expose
      the empty/NotFound-only contract above.
    - VERDICT: TRIPLE-DEAD. Component unregistered (so user never
      sees it). Even if mounted, the upstream commands return
      empty/NotFound. Even if signals arrived, they read from
      Concept/BabyStep/etc — a DIFFERENT data model
      (research_tree::ConceptDecomposition) than PlanSkeleton.
      None of this is the PlanSkeleton from
      F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\plan_skeleton.rs.

(D) SSB action-type rendering for plan events
    File: F:\.ELAI_workflow\.elai_cc\crates\dashboard\src-frontend\src\ActivityEntry.svelte
    - Lines 241-246: `case "plan_update":` and `case "stub_reference":`
      and `case "branch_merge":` all map to colour `"purple"` (the
      "structural / reasoning" bucket).
    - Lines 1105-1110: type-label for those rows is `"plan · update"`
      / `"plan · replan"` / `"stub · ref"`.
    - Effect: when `PlanSkeleton::write_to_file` emits a
      PlanUpdate or StubReference SSB row, the activity feed
      shows ONE single-line purple row with text like
      `plan · update` and a payload-derived summary. There is
      NO panel that aggregates these into a tree; NO state
      tracking that links a `plan_update` to which phase changed;
      NO action allowing the user to click the row and open the
      affected phase's contracts.
    - VERDICT: BREADCRUMB-ONLY. The signal reaches the user as
      a colourised log entry. No structural rendering.

(E) `/plan` slash command
    - CLI handler: cli/main.rs:4572-4619 (PASS in 3G audit).
    - Dashboard: NO wiring. SlashPalette in
      `chat/SlashPalette.svelte` exists (per phase-4K audit) but
      grep for `"plan"` in src-frontend returns ONLY
      `Sidebar.svelte:21` (the icon label) and
      `WindowRegistry.ts:39` (the title). There is no chat-side
      `/plan` handler that displays the PlanSkeleton tree. The
      user typing `/plan` in the dashboard's chat box does NOT
      reach the CLI handler — the CLI command is for the cli
      binary, not for the Tauri app.
    - VERDICT: ABSENT in dashboard. `/plan` is a CLI-only
      surface today.

(F) Coherence violations
    - `coherence_check::run_coherence_checks` produces
      CoherenceError variants with severity=BLOCK
      (coherence_check.rs:215-248).
    - Grep across dashboard/ for `coherence`, `Coherence`,
      `CoherenceError`: ZERO matches.
    - The auditor agent is the spec-mandated invoker
      (coherence_check.rs module-level comment "library, invoked
      BY auditor via IPC ToolRequest"). When the auditor blocks
      a phase, the user sees a generic FAIL row in the activity
      feed; there is NO surface that says "coherence: missing
      contract X in phase Y, declared by phase Z's outputs".
    - VERDICT: INVISIBLE.

(G) Staleness warnings
    - `load_phase_context` emits StalenessWarning when
      PhaseDetail.skeleton_version < skeleton.version
      (plan_skeleton.rs:930-940).
    - Grep for `StalenessWarning`, `staleness_warning`: ZERO
      matches in dashboard.
    - VERDICT: INVISIBLE.

----------------------------------------------------------------
3G WIRING VERDICT TABLE
----------------------------------------------------------------
WindowType.PLAN target          MISNAMED   (routes to Workspace,
                                            not PlanWindow)
PlanTree (in Workspace)         SURFACED   (phase_progress timeline,
                                            NOT skeleton DAG)
PlanWindow.svelte               DEAD       (component unregistered)
get_decomposition_tree          DEAD       (returns empty always)
get_phase_detail                DEAD       (returns empty shell or
                                            NotFound; no description/
                                            dependencies/outcome)
get_baby_step_detail            DEAD       (always NotFound)
PlanSkeleton TOML rendering     INVISIBLE  (no consumer)
extract_contracts surfacing     INVISIBLE  (no consumer)
ContractRef surfacing           INVISIBLE  (no consumer)
DAG (depends_on/depended_by)    INVISIBLE  (no consumer)
SsbPayload::PlanUpdate          PARTIAL    (1-line breadcrumb in feed)
SsbPayload::StubReference       PARTIAL    (1-line breadcrumb in feed)
coherence_check violations      INVISIBLE  (no consumer)
StalenessWarning                INVISIBLE  (no consumer)
/plan slash command (dashboard) ABSENT     (CLI-only)
plan_phase_slot_tokens / Slot 2 INVISIBLE  (used internally by
                                            orchestrator; no UI surface
                                            shows token usage)

================================================================
DEAD UI ELEMENTS SUMMARY
================================================================

Strict definition of DEAD: backend signal reachable end-to-end OR
component fully implemented, but the rendered UI is NEVER mounted /
NEVER invoked / always returns empty.

1. PlanWindow.svelte (273 LOC, registered nowhere).
   F:\.ELAI_workflow\.elai_cc\crates\dashboard\src-frontend\src\windows\PlanWindow.svelte:7-8
   header comment confirms intentional non-registration.

2. get_rule_detail Tauri command (62 LOC, registered, never invoked).
   F:\.ELAI_workflow\.elai_cc\crates\dashboard\src\commands.rs:3089-3150
   F:\.ELAI_workflow\.elai_cc\crates\dashboard\src\lib.rs:1472
   Frontend grep for `get_rule_detail` = ZERO matches.

3. get_decomposition_tree (registered + invoked, but viewer
   ALWAYS returns empty per ssb_viewer.rs:1383-1391).

4. get_baby_step_detail (registered + invoked, ALWAYS returns
   NotFound per ssb_viewer.rs:1473-1477).

5. get_phase_detail (registered + invoked, returns
   PhaseDetail::empty shell with no description/dependencies/
   expected_outcome — pure existence check).

6. AntiStalenessDrawer.svelte (renders three "no data yet" lines —
   deferred by design but the user expects something here given the
   21 deprecated_patterns ARE seeded).

7. ActivityEntry deprecated_pattern / version_gate category branches
   — colourise individual Finding rows but no aggregator panel.

================================================================
ANSWERS TO THE FIVE-POINT QUESTION SET
================================================================

(1) Path written:
    F:\.ELAI_workflow\docs\thefinalglance\audit-dashboard\rules-plan-wiring.md

(2) Overall verdict:
    BARELY-WIRED RULES, ENTIRELY-INVISIBLE PLAN SKELETON.
    3F ships ONE shallow lifecycle-counts drawer + ONE post-
    execution violations strip. 3G ships ZERO PlanSkeleton-aware
    surfaces — the WindowType.PLAN slot mounts a
    phase-completion timeline that has nothing to do with
    PlanSkeleton.

(3) Does RulesDrawer show real rule lifecycle from rules.db?
    YES, but partially. It shows real bucket counts
    (PROPOSED/CANDIDATE/SHADOW/ACTIVE/RETIRED/unknown/total) read
    from `SELECT status, COUNT(*) FROM rules GROUP BY status` on
    the live rules.db (commands.rs:3061-3084). It does NOT show:
      - which rules are in each bucket,
      - what they do (predicate type, title, description),
      - lifecycle TRANSITIONS (rule_transitions table is unread),
      - per-rule metrics (FPR/TPR/CCR — rule_metrics unread),
      - violation_patterns occurrences,
      - failure_category proposals.
    `get_rule_detail` exists for the per-rule view but no Svelte
    component invokes it. RulesDrawer does NOT poll — it re-fetches
    only on collapse+re-expand.

(4) Does Plan window render PlanSkeleton tree from plans/*.toml?
    NO. The Plan window:
      - WindowType.PLAN routes to `Workspace.svelte`, not
        PlanWindow.svelte (WindowRegistry.ts:27).
      - Workspace mounts `PlanTree.svelte`, which polls
        `get_phase_progress` returning PhaseComplete SSB rows
        (ssb_viewer.rs:790-815) — NOT PlanSkeleton TOML.
      - PlanWindow.svelte (the file that LOOKS like it would render
        the PlanSkeleton tree) is intentionally unregistered and
        even if registered would render empty data because
        `get_decomposition_tree` always returns empty,
        `get_phase_detail` returns an empty shell or NotFound, and
        `get_baby_step_detail` always returns NotFound.
      - No code anywhere in `crates/dashboard/` reads
        `plans/*.toml`, instantiates `PlanSkeleton::from_toml`, or
        calls `PlanSkeleton::extract_contracts`. The 6 sites in the
        ORCHESTRATOR that weave PlanSkeleton into context budget
        live in `crates/orchestrator/src/...` and never reach the
        Tauri layer.
      - SsbPayload::PlanUpdate / StubReference reach the activity
        feed as 1-line purple breadcrumbs only — no aggregation.

(5) DEAD UI elements:
    See "DEAD UI ELEMENTS SUMMARY" above. Top three by user
    surprise:
      a) PlanWindow.svelte exists but is unmounted; clicking the
         "plan" sidebar icon shows the OLD timeline view.
      b) get_rule_detail is registered, working, real-data-backed
         but has NO frontend invoker — the entire per-rule
         drilldown surface is missing.
      c) get_decomposition_tree / get_phase_detail /
         get_baby_step_detail are wired into PlanWindow but their
         viewer methods are documented to always return empty or
         NotFound (ssb_viewer.rs:1375-1478).

================================================================
RECOMMENDATIONS (HOSTILE — minimum work to remove the UI lie)
================================================================

A. Stop calling the WindowType.PLAN target a "plan". Either:
   - Rename Workspace.svelte's tab to "phase timeline" since that
     is what `get_phase_progress` produces; OR
   - Add a `get_plan_skeleton(project_id) -> PlanSkeleton` Tauri
     command that reads `{root_path}/plans/*.toml`, register
     PlanWindow as the WindowType.PLAN target, and render the
     real DAG (depends_on arrows + ContractRef per phase + version).

B. Hook RulesDrawer's bucket counts so a click expands into a list
   of rules in that bucket, each row invoking `get_rule_detail` to
   show predicate type / severity / title / mutable / version /
   source. The backend already exists — the frontend invocation is
   the one missing line.

C. Add a `get_recent_rule_transitions(limit)` Tauri command that
   reads from `rule_transitions` SQL table and pushes rows into the
   activity feed under a new `rule_transition` action_type colour,
   so the user sees PROPOSED→CANDIDATE→SHADOW→ACTIVE in real time
   without re-expanding the drawer.

D. Add a `get_deprecated_patterns(ecosystem)` command and a
   sub-section inside RulesDrawer that lists the 21 seeded entries
   plus any auto-discovered ones — the ecosystem most users care
   about (cargo / pip / npm) should be a tab.

E. Either delete PlanWindow.svelte (it is dead code that pretends
   to be alive) or wire it to a real PlanSkeleton source AND
   register it. Today it is documentation-only.

F. Wire SlashPalette to a backend `slash_command(input)` that
   dispatches /plan / /plan list / /plan <phase_id> through the
   same handler the CLI uses, so the dashboard chat surface
   reaches feature parity with the CLI.

================================================================
END
================================================================
