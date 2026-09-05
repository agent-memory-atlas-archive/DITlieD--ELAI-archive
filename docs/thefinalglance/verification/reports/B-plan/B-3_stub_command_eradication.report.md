# Verification Report: B-3_stub_command_eradication
Source: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-3_stub_command_eradication.md
Date: 2026-05-17
Overall status: PARTIAL
Confidence: HIGH

## Summary

B-3 targeted ~30 dead UI elements across the V1 dashboard, but the plan itself issued a V2 rebaseline
that redirected all work to dashboard-v2. The V2 dashboard replaced scattered Ok(default()) stubs with
a centralized stub.ts layer (2460 lines) with tryInvoke() fallback wiring — structurally aligned with
the plan's CLASS A intent. However, most CLASS A backend producers remain unimplemented stubs returning
hardcoded zeros or empty arrays (pagerank_top, query_tier_stats, freshness_state, emit_reasoning real
values, rule_transition subscription, isolation metric renderer hiding). CLASS C feature-gating via
ELAI_DASHBOARD_DEV or Rust cfg(feature=) was not applied to AutoAdvanceStats, ProposalsPanel, or
MonitorWindow. The PlanWindow is registered in WindowHost.svelte. The reindex button calls a real
IndexingService.run_full_index(). The B-3-decisions.md file is absent from workflow/reasoning/.

## Claims verified DONE

- PlanWindow registered — evidence: [WindowHost.svelte:29-30](F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\layout\WindowHost.svelte#L29) routes `type === 'plan'` to PlanWindow component
- Reindex button wires to real IndexingService — evidence: [commands.rs:4028-4037](F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src-tauri\src\commands.rs#L4028) calls `IndexingService::new().run_full_index()` with SSB progress emit
- stub.ts centralization — evidence: [stub.ts:1-10](F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\api\stub.ts#L1) 2460-line file with tryInvoke() fallback replacing V1 scattered Ok(default()) returns
- Zero Ok(default()) in dashboard-v2 commands.rs — evidence: grep across [commands.rs](F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src-tauri\src\commands.rs) found no `Ok(.*::default())` hits
- KnowledgeDrawer items 2,3,4 show UnwiredNote with B-N references — evidence: [KnowledgeDrawer.svelte:169,207](F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\drawers\KnowledgeDrawer.svelte#L169) phase="B-4" labels on query trace and recall sections
- AntiStalenessDrawer dep_audit unwired note — evidence: [AntiStalenessDrawer.svelte:106-109](F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\drawers\AntiStalenessDrawer.svelte#L106) shows UnwiredNote with phase="A-15"
- get_decomposition_tree implemented (non-trivial) — evidence: [commands.rs:1904-1921](F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src-tauri\src\commands.rs#L1904) reads SsbViewer.decomposition_tree(), reshapes with concept/phase/step hierarchy
- fork_session_branch / merge_session_branch return typed error (not fake success) — evidence: [commands.rs:1853-1869](F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src-tauri\src\commands.rs#L1853) returns Err("...use CLI: /session fork") — CLASS B/C deferral acknowledged
- /continue listed in slash palette — evidence: [CommandWindow.svelte:47](F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\windows\CommandWindow.svelte#L47) command `'/continue'` in slash definitions list
- RulesDrawer polling active (10s) via makeRefCounted — evidence: [drawers.svelte.ts:65,82](F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\state\drawers.svelte.ts#L65) `POLL_INTERVAL_MS = 10_000` used in setInterval for all drawers including rules
- knowledge_degradation SSB type registered and emitter in knowledge crate — evidence: [degradation.rs:99-144](F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\degradation.rs#L99) emits `knowledge_degradation` on cascade switch

## Claims PARTIAL

- Item #1 languages always "–" — CLASS A
  - exists: [task_runner.rs:3985-3988](F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\task_runner.rs#L3985) `emit_dkg_ready(…, Vec::new())` with comment "A-35 scope; DKG does not expose per-language stats"
  - gap: languages field still hardcoded to Vec::new(); no file-extension fallback implemented; deferred to A-35

- Item #9 GRC budgets synthetic data — CLASS A
  - exists: [task_runner.rs:6218-6240](F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\task_runner.rs#L6218) emit_reasoning function exists with TODO comment for B-5 T3 refactor; `budget_remaining: 0`, `confidence: 0.9` (hardcoded)
  - gap: comment at line 6218 explicitly defers real per-agent values to B-5 T3; all 11 call sites use synthetic constants

- Item #20 /continue handler — CLASS A
  - exists: [CommandWindow.svelte:47,390](F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\windows\CommandWindow.svelte#L47) `/continue` listed in palette and button renders
  - gap: executeSlash() only prepopulates input with "/continue " — no SessionDag::load_handoff backend call; B-9 deferred

- Items #21-24 FSM badge / status bar live wiring — deferred to B-2
  - exists: B-2 report shows DONE; FSM state color mapping visible in [StatusBar.svelte:46](F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\layout\StatusBar.svelte#L46)
  - gap: EXECUTING pulse and MonitorWindow.currently not independently verified in this report; marked partial pending B-2 confirmation

## Claims MISSING

- Item #2 pagerank_top real SQL — searched: `pagerank_top` in ssb_viewer.rs; [ssb_viewer.rs:2699-2703](F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb_viewer.rs#L2699) returns `Ok(Vec::new())` — no SQL query implemented
- Item #3 query_tier_stats real aggregate — searched: `query_tier_stats` in ssb_viewer.rs; [ssb_viewer.rs:2708-2717](F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb_viewer.rs#L2708) returns all-zero QueryTierStats struct; B-4 SSB row not defined yet
- Item #4 freshness_state real read — searched: `freshness_state` in ssb_viewer.rs; [ssb_viewer.rs:2722-2728](F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb_viewer.rs#L2722) returns `known: false` hardcoded; no freshness_meta.db read
- Item #11 rule_transition SSB subscription (last 5 transitions) — searched: `rule_transition` across dashboard-v2 src; no SSB subscription or transition history display found in RulesDrawer.svelte or drawers.svelte.ts
- Item #16 ts-rs binding generation — searched: `ts-rs` in Cargo.toml and all .rs files; no ts-rs dependency, no `#[ts]` derive, no `types.gen.ts` file found anywhere in codebase
- Items #27-29 CLASS C feature gates (AutoAdvanceStats, ProposalsPanel, MonitorWindow) — searched: `ELAI_DASHBOARD_DEV`, `cfg(feature="autonomous-mode")`, `cfg(feature="self-modification")`; MonitorWindow.svelte:318,335 renders ProposalsPanel and AutoAdvanceStats unconditionally with no feature gate
- workflow/reasoning/B-3-decisions.md — searched: `B-3` in workflow/reasoning/; file does not exist

## Claims UNVERIFIABLE

- Item #5 knowledge_degradation wired from CLI SearchMode cascade — The degradation emitter exists in knowledge crate (degradation.rs:144), but verifying whether cli/main.rs replaced NoopSsbNotifier with a real SsbNotifier in the SearchMode switch path would require tracing startup_readiness_check through multiple files; live_cli.rs:1086 constructs DegradationCascade but the ssb notifier passed is not observable from the search scope — memory-bank changelog refs excluded per protocol
- Item #18 branch_merge dashboard renderer (V2 ActivityWindow) — B-9 SessionDag activation is a prerequisite; BranchMerge SSB type registered in ssb.rs:465, but ActivityWindow.svelte renderer for this event type not confirmed without full ActivityWindow read

## What must be done (action list)

- [ ] Implement `pagerank_top` SQL in [ssb_viewer.rs:2699](F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb_viewer.rs#L2699): `SELECT symbol, score FROM dkg_nodes_pagerank ORDER BY score DESC LIMIT ?`
- [ ] Implement `query_tier_stats` aggregate in [ssb_viewer.rs:2708](F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb_viewer.rs#L2708) from QueryTrace SSB rows (or keep hidden until B-4 lands)
- [ ] Implement `freshness_state` real DB read in [ssb_viewer.rs:2722](F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb_viewer.rs#L2722) (hide section if freshness_meta.db absent)
- [ ] Add languages fallback (count by file extension via DKG nodes table) in [task_runner.rs:3988](F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\task_runner.rs#L3988) or defer to A-35 with explicit label
- [ ] Refactor `emit_reasoning` in [task_runner.rs:6228](F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\task_runner.rs#L6228) to accept `Option<&Arc<AsyncMutex<PipelineGrc>>>` for real per-agent budget_remaining (B-5 T3 dependency)
- [ ] Add rule_transition SSB subscription in [drawers.svelte.ts](F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\state\drawers.svelte.ts) and render last 5 transitions in RulesDrawer.svelte
- [ ] Add ts-rs dep to dashboard-v2 Cargo.toml and generate types.gen.ts from Rust SessionHandoffEntry and related types
- [ ] Feature-gate AutoAdvanceStats and ProposalsPanel in [MonitorWindow.svelte:318,335](F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\windows\MonitorWindow.svelte#L318) behind ELAI_DASHBOARD_DEV or equivalent
- [ ] Wire /continue to actual SessionDag::load_handoff Tauri command (requires B-9 completion)
- [ ] Create workflow/reasoning/B-3-decisions.md with per-item A/B/C classification

## Notes

The V2 rebaseline was architecturally sound — centralizing stubs in stub.ts with tryInvoke() fallback
is strictly better than scattered Ok(default()) returns. The `reindex_now` command and `get_decomposition_tree`
are genuinely non-trivial implementations. The main structural gaps are: (1) three ssb_viewer.rs functions
still return hardcoded zeros, (2) ts-rs was never added, (3) feature-gating for phase-4G/4H UI was not
applied, (4) B-3-decisions.md was never written. Items dependent on B-5, B-9, B-17 are legitimately
deferred and the code reflects that with explicit comments.
