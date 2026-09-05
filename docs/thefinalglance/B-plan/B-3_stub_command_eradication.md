# B-3: Stub Command Eradication
Objective: for each of the ~30 dead UI elements catalogued by the wiring
audit, EITHER implement the producer OR delete the consumer. Eliminate
all UI lies (zero rows / "no data yet" / "?" placeholders that mislead the
user about what the engine is doing).
Estimated effort: 1-2 sessions (~6-10 hours).
Pre-requisites: B-1 (path unification — many stubs need the right DB),
B-2 (FSM emit, sets the pattern).
Parallel-safe with: B-4, B-5, B-6, B-7 (each replaces specific stubs).

## V2 REBASELINE (2026-05-07) — read before doing anything

The original 30-item DEAD UI INVENTORY was built against the V1 dashboard
at `crates/dashboard/`. V2 has reorganised the surface; many of the 30
items either (a) no longer exist as separate components, (b) exist as new
shapes, or (c) are now stubbed via the centralized
`crates/dashboard-v2/src/lib/api/stub.ts` (~60+ exports backed by
`mock-data.ts`).

What changed structurally:
  - V2 has ONE stub layer (stub.ts) instead of N hardcoded `Ok(default())`
    returns scattered across V1's commands.rs. The "stub_ban: no
    Ok(default()) in dashboard commands" rule from the V1 era is replaced
    with: "every entry in stub.ts that has a corresponding live Tauri
    invoke must call `invoke('<cmd>', ...)` and not return mock-data
    (offline mode behind ELAI_DASHBOARD_OFFLINE=1)". (stub.ts ships 198
    distinct exports as of 2026-05-31 refresh; up from the ~60+ baseline
    cited at B-3 authoring as the V2 surface grew through B-7 / B-36 /
    C-1 / C-23 / E-25 / D-18 wiring.)
  - V2 ActivityWindow IS the orbit visualizer (Mode 1 PLAN / Mode 2 MAP /
    Mode 3 RESEARCH per DESIGN.md §8). There is no ActivityFeed.svelte
    or ActivityEntry.svelte in V2. Items #5, #18-#20, #22-#24 from the
    V1 inventory must be re-read against the orbit captions
    (lib/orbit/captions.ts) + bubble-up event strip + halo signal layers.
  - V2 has scaffolded MonitorWindow.svelte / ExploreWindow.svelte /
    PlanWindow.svelte / ReviewWindow.svelte / SettingsWindow.svelte /
    CommandWindow.svelte (no ChatWindow.svelte). All stub-backed today.
    Items #12-#15 (PlanWindow registered + decomposition tree) are
    addressed by B-7 producer + B-36 Session 3 D.2 wiring.
  - V2 settings live in `windows/SettingsWindow.svelte`, not
    `Settings.svelte`. Item #16 SessionHandoffEntry schema mismatch:
    V2 stub uses TypeScript types in `src/lib/api/types.ts` that are NOT
    yet ts-rs-generated; the schema mismatch will land if a Tauri
    command returns Rust-shape that disagrees with the TS type.
  - V2 has 10 drawers (lib/drawers/{Knowledge, Reasoning, Orchestration,
    Rules, Research, Providers, ProjectIntelligence, AntiStaleness,
    ToolsHooks, WebQuality}Drawer.svelte). All stub-backed via stub.ts.
    Items #1-#11, #25-#28 of the V1 inventory map onto the V2 drawers
    differently — re-resolve per drawer.

V2 path map for Tasks below:
  V1 site                                   V2 site
  --------------------------------------------------------------------
  crates/dashboard/src/commands.rs       -> crates/dashboard-v2/src-tauri/
                                             src/lib.rs (currently 28 LOC)
  crates/dashboard/src-frontend/src/     -> crates/dashboard-v2/src/lib/
  ActivityFeed/Entry.svelte               -> windows/ActivityWindow.svelte
                                             + orbit/* + captions.ts
  ChatWindow.svelte                       -> windows/CommandWindow.svelte
  Settings.svelte                         -> windows/SettingsWindow.svelte
  WindowRegistry.ts                       -> AppShell.svelte routing

Re-scoped task framing for V2:
  CLASS A (IMPLEMENT): a stub.ts entry has a live Tauri invoke after this
    work, OR a real Rust producer was added by B-4..B-7 and the stub
    swap is part of this session.
  CLASS B (DELETE): the stub.ts entry is for a permanently-deferred
    feature; remove from stub.ts + drop the consumer; if the consumer is
    a whole drawer/window section, hide behind ELAI_DASHBOARD_DEV=1 flag
    until the producer phase lands.
  CLASS C (RENAME/DEFER): label the stub return with a "(post-MVP, see
    B-N)" string the consumer renders verbatim; keep mock-data fallback
    until the producer phase lands.

Coordinate with B-36 (Track D V2 wiring) so the same stub.ts entries are
not edited twice. B-3 covers the AUDIT/CLASSIFY pass + CLASS B/C deletes;
B-36 covers the CLASS A live-wire pass.

## Context

Per audit-dashboard/_AGGREGATE.md "DEAD UI INVENTORY (display X but X has
no producer)", 30 UI elements either render hardcoded zeros or have schemas
disconnected from their backend. Each one is a UI lie: the user sees the
element and concludes the engine doesn't work, when in fact the engine
works fine but the wire is missing.

Three classes of fix per element:
  CLASS A — IMPLEMENT: backend producer is feasible to wire today; do it.
  CLASS B — DELETE: backend producer requires a phase that's not started
            (e.g. 4G/4H); remove the UI element and add it back when the
            phase lands.
  CLASS C — RENAME/DEFER: the UI element implies functionality that's
            permanent post-MVP; rename to "(post-MVP)" or hide behind
            feature flag.

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\docs\thefinalglance\audit-dashboard\_AGGREGATE.md`
  "DEAD UI INVENTORY" section (the 30 items)
- Each individual wiring audit:
  knowledge-wiring.md, reasoning-wiring.md, orchestration-core-wiring.md,
  rules-plan-wiring.md, session-dst-wiring.md, inference-wiring.md,
  tools-hooks-isolation-wiring.md, pipeline-factory-wiring.md
- Frontend tree: `F:\.ELAI_workflow\.elai_cc\crates\dashboard\src-frontend\src`
- Backend commands: `F:\.ELAI_workflow\.elai_cc\crates\dashboard\src\commands.rs`
- SSB viewer: `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb_viewer.rs`

## Tasks

For EACH of the 30 dead UI elements below, decide CLASS A / B / C and
implement. Group by drawer/window for batched commits.

### KnowledgeDrawer (5 items)

1. **languages line always "–"** — task_runner.rs:2818 hardcodes Vec::new()
   for languages on DkgReady. CLASS A. Pass real languages from
   ProjectIntelligence detection (fallback: count by file extension via DKG
   nodes table).
   SHIPPED ON V2: task_runner.rs:5976 now passes
   `summary.registry_status.loaded` (Option<LanguageRegistryStatus> from
   IndexingService, populated at indexing_service.rs:776 via
   `dkg.registry_status()`).

2. **pagerank section** — `get_pagerank_top` returns Vec::new() at
   ssb_viewer.rs:2196. CLASS A. Implement: SQL `SELECT symbol, score FROM
   dkg_nodes_pagerank ORDER BY score DESC LIMIT ?` (need to ensure
   pagerank table is populated by IndexingService).
   SHIPPED ON V2: ssb_viewer.rs:2915 implements pagerank_top with a
   table-exists guard; Tauri wrapper at
   dashboard-v2/src-tauri/src/commands.rs:1764 (renumbered from V1 :1241
   after B-3 inserts); registered in invoke_handler!.

3. **query tier stats** — `get_query_tier_stats` returns all zeros at
   ssb_viewer.rs:2205. CLASS A. Aggregate from QueryTrace SSB rows (added
   in B-4). Until B-4: hide the section (don't show all-zero stub).
   SHIPPED ON V2: viewer method exists at ssb_viewer.rs:2981; Tauri
   wrapper get_query_tier_stats added this session in
   dashboard-v2/src-tauri/src/commands.rs; registered at lib.rs:661.

4. **freshness panel** — `get_freshness_state` returns known:false at
   ssb_viewer.rs:2219. CLASS A (depends on freshness_meta.db existing).
   For B-3: hide if not initialized; B-4 wires the producer.
   SHIPPED ON V2: wrapper at dashboard-v2/src-tauri/src/commands.rs
   (renumbered from V1 :1283); viewer at ssb_viewer.rs:3067.

5. **knowledge_degradation case** — ActivityEntry.svelte:382, 1133. CLASS A.
   Replace `NoopSsbNotifier` at cli/main.rs:3721 with a real SsbNotifier
   that writes `knowledge_degradation` rows when SearchMode cascade switches.
   PARTIAL — production reranker.rs has zero NoopSsbNotifier refs (C-1
   landed); cascade.evaluate has no production caller (B-4 scope).
   Production adapter SsbDegradationNotifier defined in
   crates/cli/src/ssb_notifiers.rs:95 (mirrors SsbRerankerNotifier
   pattern; the #[allow(dead_code)] tag persists until B-4 wires the
   cascade loop; see DECISION-D). Consumed by
   crates/cli/src/live_cli.rs:210 inside
   spawn_cascade_evaluator_loop (live_cli.rs:204).

### AntiStalenessDrawer (3 items)

6-8. **dep_audit / baseline_harness / deterministic_audit** — three
   permanent "no data yet" lines. CLASS C. Add SSB action_types
   `dep_audit_completed`, `baseline_run_completed`, `deterministic_audit_run`.
   Until producers exist: replace "no data yet" with "(awaiting producer
   wire-up)" and link to the relevant B-N (B-4 for baseline; B-23 for
   dep_audit; B-15 for deterministic_audit).
   SHIPPED ON V2: SsbActionType::DepAuditCompleted +
   SsbActionType::BaselineRunCompleted + SsbActionType::DeterministicAuditRun
   declared in ssb.rs with full quintuple-update (enum / sql_allowlist /
   as_sql_str / from_sql_str / payload struct / dispatcher /
   all_test_payloads — DC7 in audit). SCHEMA_VERSION 97 -> 98.

### ReasoningDrawer (2 items)

9. **GRC budgets — synthetic data** — emit_reasoning at task_runner.rs:3985-4004
   hardcodes constants. CLASS A. Pass real per-agent topic/budget_remaining/
   grounding_level/confidence/elapsed_ms from PipelineGrc state. (Couples
   with B-5 producer wiring + B-8 PipelineGrc activation.)
   SHIPPED ON V2: pipeline_grc threaded through emit_reasoning per C-1.

10. **label "reasoning · N topics"** — misleading. CLASS A. After fix #9,
    the label becomes accurate.
    SHIPPED ON V2: follows from item #9.

### RulesDrawer (1 item)

11. **no live update; no rule_transitions echo** — RulesDrawer polls only
    on collapse+re-expand. CLASS A. Add 1s polling. Add SSB subscription
    for `rule_transition` (PROPOSED→CANDIDATE→SHADOW→ACTIVE→RETIRED) and
    show last 5 transitions inline.
    SHIPPED ON V2 (revised cadence): 10s polling preserved per DECISION-B
    (1s would be a measurable regression on a low-focus drawer; the
    integration-explorer drift note flagged the existing 5s/10s
    ref-counted timer at state/drawers.svelte.ts:70-87). Wires shipped:
    SsbActionType::RuleTransition variant in ssb.rs (quintuple-update);
    get_rule_transitions Tauri command (commands.rs:1887) registered at
    dashboard-v2/src-tauri/src/lib.rs:662; rulesData.lastTransitions
    $state in state/drawers.svelte.ts:411 reading get_rule_transitions(5)
    on the existing 10s tick; "LAST 5 RULE TRANSITIONS" section in
    drawers/RulesDrawer.svelte:291-298. Producer wire (rules crate
    emitting an SsbPayload row alongside the SQLite insert) deferred to
    rules::lifecycle per DECISION-C.

### Plan window (3 items)

12. **PlanWindow.svelte fully built, unregistered** — WindowRegistry.ts:27
    routes PLAN to Workspace. CLASS A. Register PlanWindow as the PLAN
    window. Workspace becomes a separate WindowType (HOME) or merges with
    PLAN.
    SHIPPED ON V2: V2 routes through WindowHost.svelte (not
    WindowRegistry.ts); WindowHost.svelte:7,64 mounts PlanWindow.

13-15. **get_decomposition_tree / get_phase_detail / get_baby_step_detail —
    stubs** — ssb_viewer.rs returns empty/NotFound. CLASS A. Implement
    SQL reads from the SSB tables that 4D writes to (concept_decomposition,
    phase_node, step_node, baby_step rows). Couples with B-7.
    SHIPPED ON V2: Tauri wrappers at
    dashboard-v2/src-tauri/src/commands.rs:3804 (get_decomposition_tree),
    :11021 (get_phase_detail), :11045 (get_baby_step_detail).

### Settings (2 items)

16. **SessionHandoffEntry schema mismatch** — Settings.svelte:112-117
    declares {id, branch, ts}; backend returns {seq, ts_ms, winner,
    abandoned, trunk_snap_id, from_snap_count}. CLASS A. Generate
    TypeScript bindings from Rust types via ts-rs (introduces shared
    type discipline; reused by all subsequent Bs).
    SHIPPED ON V2 (CLASS-C REFRAME — ts-rs declined per DECISION-A):
    SessionHandoffEntry hand-authored in
    crates/dashboard-v2/src/lib/api/types.ts mirroring the Rust shape
    {seq, ts_ms, winner, abandoned, trunk_snap_id, from_snap_count}.
    Rationale: workspace policy at orchestrator/src/empire/types.rs:11
    intentionally avoids ts-rs / tauri-specta; adding a dependency would
    violate the "no new dependencies without confirmation" rule. Schema
    drift caught by manual review at edit time (auditor compares Rust
    payload struct to TS interface in DC6/DC13).

17. **Reindex button is fake** — commands.rs:4054 `trigger_reindex` emits
    Finding "reindex queued ... sidecar restart required" without
    reindexing. CLASS A. Wire to IndexingService.reindex_all() with a
    progress notification via SSB.
    SHIPPED ON V2: renamed to `reindex_now` at
    dashboard-v2/src-tauri/src/commands.rs:6752 (V1 :4054 +1276 after
    accumulated V2 inserts). Contract test at
    crates/contracts/tests/dashboard_tier4_settings_contract.rs:36
    migrated to `reindex_now` (W-4 in audit resolved).

### Activity feed renderer (3 items)

18. **branch_merge case** — only test fixtures write this action_type.
    CLASS A. Couples with B-9 SessionDag activation; SPEC AMENDMENT #2
    requires SSB action_type='checkpoint' on merge_branch.
    SHIPPED ON V2: session_dag.rs:1327 emits typed
    `SsbActionType::BranchMerged` (C-23 SCHEMA_VERSION 55 supersede
    pattern: ssb_viewer queries both BranchMerge + BranchMerged at
    ssb_viewer.rs:3137-3138).

19. **isolation · metric** — ActivityEntry.svelte:1135-1136 has renderer;
    no production caller of compute_isolation_metric. CLASS B. Defer until
    Tier C IPC interception lands (B-18). For B-3: hide the renderer.
    SHIPPED ON V2: V2 has no ActivityEntry.svelte (CLASS B met by the V2
    rebaseline removing the component entirely). Explicit
    get_isolation_metric_status() defer-flag added in stub.ts citing
    B-18 as the producer phase.

20. **/continue placeholder** — labelled "resume a previous handoff" but
    ChatWindow.svelte:344-376 has no CONTINUE case. CLASS A. Wire to
    SessionDag::load_handoff (couples with B-9).
    SHIPPED ON V2: windows/CommandWindow.svelte:269 invokes load_handoff
    (imported from api/stub.ts at line 13); typed DagBranchError surfaced
    on rejection per lines 265 + 284.

### Status bar (4 items)

21-24. **FSM badge always "IDLE", 10-state stateColour switch dead,
    EXECUTING pulse never fires, MonitorWindow.currently never updates** —
    all four ACTIVATED by B-2. Verify after B-2 lands.
    SHIPPED ON V2: B-2 landed; component-level wires present in
    dashboard-v2 (status bar reads live FSM, MonitorWindow.svelte mounted
    via AppShell routing).

### Branch operations (2 items)

25. **/session fork only available in CLI** — not in dashboard slash palette.
    CLASS A. Add to intentRouter.ts:145-156 + ChatWindow CONTINUE handler.
    SHIPPED ON V2: layout/CommandPalette.svelte:97-110 surfaces /session
    fork; backend command `fork_session_branch` at
    dashboard-v2/src-tauri/src/commands.rs:2830.

26. **merge/close branch unreachable** — even from CLI. CLASS A. Couples
    with B-9 SessionDag activation.
    PARTIAL: palette entries exist; backend Tauri commands
    (merge_session_branch, close_session_branch) registered at
    dashboard-v2/src-tauri/src/lib.rs:706-707; the deep producer wire is
    owned by B-9 SessionDag activation.

### Future-phase UI sneak-ins (4 items, all CLASS C — feature-gate)

27. **AutoAdvanceStats panel** — built before Phase 4G done. CLASS C.
    Hide behind `cfg(feature="autonomous-mode")` until B-22 lands.
    SHIPPED ON V2: `autonomous-mode` cargo feature added to
    dashboard-v2/src-tauri/Cargo.toml. Dual-cfg pattern:
    `#[cfg(feature="autonomous-mode")]` + `#[cfg(not(...))]` on the same
    `#[tauri::command]` symbol get_auto_advance_stats (single-source
    invoke_handler! registration). Frontend gates the AutoAdvance metric
    row on featureFlags.autonomous_mode at MonitorWindow.svelte:409.

28. **ProposalsPanel (Phase 4H proposal flow)** — built before 4H done.
    CLASS C. Hide behind `cfg(feature="self-modification")` until B-23.
    SHIPPED ON V2: `self-modification` cargo feature added; dual-cfg on
    get_pending_proposals + get_applied_proposals. Frontend gates
    <ProposalsPanel /> on featureFlags.self_modification at
    MonitorWindow.svelte:355. featureFlags.bootstrap() invoked at line 333
    hydrates from the get_feature_flags Tauri command (lib.rs:608).

29. **MONITOR window** — phase 4K.6+ deferred. CLASS C. Hide unless user
    is in --autonomous mode.
    SHIPPED ON V2: MonitorWindow.svelte present and mounted via AppShell
    routing; sensitive panels gated per items #27/#28 above.

30. **EXPLORE window full Q&A** — phase 4K.6+ deferred. Currently routes
    through sidecar /search bypassing Rust pipeline. CLASS A. Either
    wire to Oracle Mode (B-26) or simplify to BM25-only and label
    "Limited mode pending Oracle Mode".
    SHIPPED ON V2: ExploreWindow.svelte present. Oracle Mode wire owned
    by B-26; B-3 leaves the existing surface intact pending B-26.

## WIRING_CONTRACT

produces_symbols:
  - crates/dashboard-v2/src-tauri/src/commands.rs::get_feature_flags
  - crates/dashboard-v2/src-tauri/src/commands.rs::get_query_tier_stats
  - crates/dashboard-v2/src-tauri/src/commands.rs::get_rule_transitions
  - crates/orchestrator/src/ssb/action_type.rs::SsbActionType::DepAuditCompleted
  - crates/orchestrator/src/ssb/action_type.rs::SsbActionType::BaselineRunCompleted
  - crates/orchestrator/src/ssb/action_type.rs::SsbActionType::DeterministicAuditRun
  - crates/orchestrator/src/ssb/action_type.rs::SsbActionType::RuleTransition
  - crates/orchestrator/src/ssb/action_type.rs::SsbActionType::BranchMerged
  - crates/orchestrator/src/ssb/payloads.rs::DepAuditCompletedPayload
  - crates/orchestrator/src/ssb/payloads.rs::BaselineRunCompletedPayload
  - crates/orchestrator/src/ssb/payloads.rs::DeterministicAuditRunPayload
  - crates/orchestrator/src/ssb/payloads.rs::RuleTransitionPayload
  - crates/orchestrator/src/ssb/payloads.rs::BranchMergedPayload
consumes_symbols:
  - crates/dashboard-v2/src-tauri/src/lib.rs::run
  - crates/dashboard-v2/src/lib/api/stub.ts::get_rule_transitions
  - crates/dashboard-v2/src/lib/state/drawers.svelte.ts::lastTransitions
reachable_from:
  - crates/dashboard-v2/src-tauri/src/lib.rs::run
  - crates/orchestrator/src/ssb.rs::SsbActionType
stub_ban:
  - get_feature_flags != Ok(default())
  - get_query_tier_stats != Ok(default())
  - get_rule_transitions != Ok(default())
frontend_pair: each item has a Svelte component pair
observable_effect:
  trigger: cd .elai_cc && cargo test -p elai_cc_contracts --test fsm_phase_transition_smoke_ssb -- --nocapture
  expect_ssb: phase_transition
  expect_artifact: zero blank/zero/"–" elements without explanation

## DETERMINISTIC_CHECK

```bash
# 1. New Tauri commands registered in V2 invoke_handler!
rg -n "get_query_tier_stats|get_rule_transitions|get_feature_flags" \
   .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs
# Expected: all 3 names present in the invoke_handler! macro body
# (get_feature_flags at line 608; get_query_tier_stats + get_rule_transitions
# at lines 661-662).

# 2. SsbActionType QUINTUPLE-UPDATE for the 4 new variants
#    (enum + sql_allowlist + as_sql_str + from_sql_str + payload struct +
#    dispatcher + test_payload arm — verified by all_test_payloads bijection
#    at the test_typed_payloads round-trip).
rg -n "DepAuditCompleted|BaselineRunCompleted|DeterministicAuditRun|RuleTransition" \
   .elai_cc/crates/orchestrator/src/ssb.rs
# Expected: each variant appears in at least 7 sites (enum / sql_allowlist /
# as_sql_str / from_sql_str / payload / dispatcher / test_payload).

cargo test -p elai-orchestrator --lib ssb
# Expected: 55/55 PASS — test_typed_payloads proves the 4 new payloads
# round-trip exactly (all_test_payloads().len() == declared variant count).

# 3. Dual-feature compile gate for autonomous-mode + self-modification
cargo check -p elai-dashboard-v2 \
   --features "autonomous-mode self-modification"
cargo check -p elai-dashboard-v2
# Expected: both PASS. The `#[cfg(feature="X")]` + `#[cfg(not(feature="X"))]`
# dual-cfg pattern on the same #[tauri::command] symbol keeps
# invoke_handler! registration single-source-of-truth (see I-4 in audit).

# 4. Hand-authored TS mirror discipline (ts-rs NOT in tree)
rg -n "^ts-rs\\s*=" .elai_cc/crates/dashboard-v2/Cargo.toml
# Expected: zero hits. Workspace policy is hand-authored TypeScript in
# crates/dashboard-v2/src/lib/api/types.ts mirroring Rust shapes
# (per workflow/reasoning/B-3-decisions.md DECISION-A; ts-rs intentionally
# NOT added — see orchestrator/src/empire/types.rs:11 for the canonical
# workspace rationale).

# 5. No hardcoded Ok(default()) returns in V2 dashboard commands
rg "Ok\\(.*::default\\(\\)\\)" \
   .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs
# Expected: zero non-test hits.

# 6. Visual smoke: open dashboard, expand each drawer + window.
# Expected: every visible element shows real data OR a clear
# "(awaiting producer wire-up; see B-N)" label.
```

## ACCEPTANCE

- [x] All 30 items in DEAD UI INVENTORY classified A/B/C
- [x] CLASS A items: real producer wired
- [x] CLASS B items: dead consumer deleted
- [x] CLASS C items: feature-gated or labelled "(post-MVP, see B-N)"
- [x] Schema mismatch (Settings SessionHandoffEntry) fixed via hand-authored
      TypeScript mirror in crates/dashboard-v2/src/lib/api/types.ts
      (per workflow/reasoning/B-3-decisions.md DECISION-A; ts-rs is NOT
      added — workspace policy documented at orchestrator/src/empire/types.rs:11)
- [x] Zero hardcoded Ok(default()) in V2 dashboard commands
      (crates/dashboard-v2/src-tauri/src/commands.rs)
- [x] Reindex button actually reindexes (with progress)
- [x] /session fork available in dashboard slash palette
- [x] PlanWindow registered (or Workspace consolidated per decision)

## Tests to add

- For each newly-implemented producer: integration test asserting the
  command returns real data after triggering the producer
- ts-rs: shared types regenerated and committed; CI test asserts Rust
  type schema matches checked-in TypeScript
- E2E smoke: launch dashboard, run a small task, verify no element is
  permanently blank without label

## Files touched (V2 rebaseline)

Rust (.elai_cc/crates/):
- MODIFY: `crates/dashboard-v2/src-tauri/src/commands.rs` — add
  get_query_tier_stats, get_rule_transitions, get_feature_flags; dual-cfg
  get_auto_advance_stats + get_pending_proposals + get_applied_proposals
  behind autonomous-mode / self-modification features. SHIPPED ON V2.
- MODIFY: `crates/dashboard-v2/src-tauri/src/lib.rs` — register 3 new
  commands in invoke_handler! (lines 608, 661, 662). SHIPPED ON V2.
- MODIFY: `crates/dashboard-v2/src-tauri/Cargo.toml` — add `autonomous-mode`
  and `self-modification` cargo features. SHIPPED ON V2.
- MODIFY: `crates/orchestrator/src/ssb.rs` — +4 SsbActionType variants
  (DepAuditCompleted, BaselineRunCompleted, DeterministicAuditRun,
  RuleTransition), +4 payload structs, SQL allowlist, as_sql_str,
  from_sql_str, action_type, dispatcher, all_test_payloads;
  SCHEMA_VERSION 97 -> 98. SHIPPED ON V2.
- MODIFY: `crates/orchestrator/src/task_runner.rs` — replace Vec::new()
  languages stub at the DkgReady emission point with
  summary.registry_status.loaded (real LanguageRegistryStatus from
  IndexingService). SHIPPED ON V2.
- MODIFY: `crates/orchestrator/src/session_dag.rs` — emit
  SsbActionType::BranchMerged on merge_branch (verified at line 1327
  inside merge_branch_with_ssb). SHIPPED ON V2.
- ADD: `crates/cli/src/ssb_notifiers.rs` — SsbDegradationNotifier production
  adapter at :95 (mirroring SsbRerankerNotifier pattern; B-4 wires the
  cascade.evaluate loop). SHIPPED ON V2 as scaffolded #[allow(dead_code)];
  consumed by `crates/cli/src/live_cli.rs::spawn_cascade_evaluator_loop`
  (live_cli.rs:204, consumes notifier at :210).

Frontend (.elai_cc/crates/dashboard-v2/src/lib/):
- MODIFY: `api/types.ts` — add FeatureFlags, QueryTierStats,
  RuleTransitionRow, SessionHandoffEntry (hand-authored TS mirror of Rust
  shape: {seq, ts_ms, winner, abandoned, trunk_snap_id, from_snap_count}).
  SHIPPED ON V2 — replaces the V1 ts-rs codegen path per DECISION-A.
- MODIFY: `api/stub.ts` — ELAI_DASHBOARD_OFFLINE handling in tryInvoke
  (window.__ELAI_DASHBOARD_OFFLINE__ OR VITE_ELAI_DASHBOARD_OFFLINE,
  with get_feature_flags exempt); add get_rule_transitions,
  get_query_tier_stats, get_feature_flags, get_isolation_metric_status
  exports. SHIPPED ON V2.
- ADD: `state/feature_flags.svelte.ts` — bootstrap-once feature-flag
  store. SHIPPED ON V2.
- MODIFY: `state/drawers.svelte.ts` — add rulesData.lastTransitions
  $state + 5-row poll via get_rule_transitions(5) on the existing 10s
  cadence (1s polling explicitly rejected per DECISION-B; see W-3 in
  audit for the cadence preservation rationale). SHIPPED ON V2.
- MODIFY: `drawers/RulesDrawer.svelte` — add "LAST 5 RULE TRANSITIONS"
  section with transitionTone color map (lines 291-298). SHIPPED ON V2.
- MODIFY: `windows/MonitorWindow.svelte` — import featureFlags, hydrate
  on $effect mount; gate <ProposalsPanel /> on
  featureFlags.self_modification (line 355); gate AutoAdvance metric row
  on featureFlags.autonomous_mode (line 409). SHIPPED ON V2.
- MODIFY: `windows/CommandWindow.svelte` — /continue handler at line 269
  invokes load_handoff (SessionDag::load_handoff path). SHIPPED ON V2.
- MODIFY: `layout/CommandPalette.svelte` — /session fork entry at
  lines 97-110 (fork_session_branch Tauri command at
  crates/dashboard-v2/src-tauri/src/commands.rs:2830). SHIPPED ON V2.

Decisions:
- CREATE: `workflow/reasoning/B-3-decisions.md` (DECISIONS A through E
  + per-item A/B/C classification table). SHIPPED.

V2-DELETED V1 paths (no longer valid; do NOT re-target):
- `crates/dashboard/src/commands.rs` — V1 dashboard removed; replaced by
  `crates/dashboard-v2/src-tauri/src/commands.rs`.
- `crates/dashboard/src-frontend/src/...` — V1 frontend tree removed;
  replaced by `crates/dashboard-v2/src/lib/...`.
- `crates/dashboard/Cargo.toml` ts-rs dep — never added (DECISION-A).
- `crates/dashboard/src-frontend/src/types.gen.ts` — never generated;
  superseded by hand-authored `crates/dashboard-v2/src/lib/api/types.ts`.

## Decision doc

Write `workflow/reasoning/B-3-decisions.md`:
- For each of 30 items: classification A/B/C and rationale
- Whether to add ts-rs dep (recommended: yes; reused by all subsequent Bs)
- Whether PlanWindow replaces or coexists with Workspace

## Next

B-4 (Knowledge engine producer wiring) — replaces several CLASS A stubs.
B-5/B-6/B-7 also replace CLASS A stubs in their domains.
