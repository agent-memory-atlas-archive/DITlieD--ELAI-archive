# DEEP-B-003 exhaustive deep-source discovery

## Scope, provenance, and extraction rules

- **Processed manifest section:** `DEEP-SOURCE-MANIFEST.md` lines 292-305, with the chunk-index cross-check at line 187.
- **Orientation only:** `02-B-PLAN-SUMMARY.discovery.md`; no claim below is promoted to source-explicit merely because it appears in that summary.
- **Original-source isolation:** read all eight originals listed for `DEEP-B-003`, from first through last logical line. No other B-plan original, referenced audit, implementation file, decision document, specification, paper, fixture, or source-code path was opened. Statements about those materials are therefore plan assertions, not independently verified implementation facts.
- **Labels:** `[E]` = explicit in a processed original; `[I]` = conservative cross-source inference; `[A]` = unresolved/ambiguous source wording; `[C]` = contradiction, stale section, or conflict that must remain visible.
- **Atomic-record rule:** one explicit record represents one independently selectable capability, behavior, state/schema/config contract, status/defer decision, or verification gate. Repeated appearances in a task, wiring contract, acceptance list, and touched-files list are consolidated into that record and cross-referenced rather than counted again.

## Exact processed manifest and integrity verification

Manifest declaration: family **B**; **8 files**, **2,576 logical lines**, **126,263 bytes**, target band **within 1,800-2,800** (`DEEP-SOURCE-MANIFEST.md` lines 292-305; chunk-index line 187).

| Ordinal | Class | Lines | Bytes | SHA-256 | Exact absolute path | Verification |
|---:|---|---:|---:|---|---|---|
| 1 | numbered/index original | 514 | 27022 | `05ba155a7fd48bd6c0fcff334bc4ab8336f0669da41bf4ea0b5b4422b5ef9211` | `docs/thefinalglance/B-plan/B-3_stub_command_eradication.md` | exact hash/lines/bytes match |
| 2 | numbered/index original | 153 | 6690 | `0aeec6b5d12127cfd46c95e580f19b18332bbf2195e1c93afa019a5898ce80ec` | `docs/thefinalglance/B-plan/B-8_pipelinegrc_activation.md` | exact hash/lines/bytes match |
| 3 | numbered/index original | 204 | 9948 | `6273a43aaafcee2a38af2c5a2aa97053fe2b928be042af69251fbf69386fac32` | `docs/thefinalglance/B-plan/B-9_sessiondag_activation.md` | exact hash/lines/bytes match |
| 4 | numbered/index original | 201 | 8836 | `f1c085dcbd1cef6d9258053dc011b3a89e3bdc36169fe903be78ed366b1202cf` | `docs/thefinalglance/B-plan/B-13_activity_feed_restoration.md` | exact hash/lines/bytes match |
| 5 | numbered/index original | 469 | 22540 | `56181ccf70c24ed8bad6002622d9ffac446d533e2df6ab018fddfe2fc9586b89` | `docs/thefinalglance/B-plan/B-15_phase0_debt_burndown.md` | exact hash/lines/bytes match |
| 6 | numbered/index original | 443 | 21321 | `a37a115862feee953a107401924a4a1d096bee21ef455d75cb5a903d7896cd34` | `docs/thefinalglance/B-plan/B-30_multilang_dkg_hardening.md` | exact hash/lines/bytes match |
| 7 | numbered/index original | 254 | 11781 | `7f2c225817e7ec5ed73e3df260cdd6ec4c71342e958d51b6240e44e64d9b28f7` | `docs/thefinalglance/B-plan/B-37_multi_runtime_backend.md` | exact hash/lines/bytes match |
| 8 | numbered/index original | 338 | 18125 | `9a48126f92bc959da36a8d1fe617a908296a01641a2c2f020baf055f99bec3df` | `docs/thefinalglance/B-plan/B-40_idea_framer_research_presets.md` | exact hash/lines/bytes match |
| **Total** |  | **2,576** | **126,263** |  | **8 originals** | **exact aggregate match** |

Verification used raw-file SHA-256 plus `wc -l -c`; all eight files are newline-terminated, so `wc -l` equals the manifest's logical-line count.

## Per-file exhaustive ledgers

### B-3 — Stub Command Eradication

Source: `docs/thefinalglance/B-plan/B-3_stub_command_eradication.md`, 514 lines.

#### Explicit ledger (45 atomic records)

- **E-B3-001 — Goal, dependency, and execution shape.** `[E]` Eliminate every misleading dead UI state among the approximately 30 audit items by implementing its producer or removing/defer-labeling its consumer; estimate 1-2 sessions/6-10 hours; prerequisites B-1 and B-2; parallel-safe with B-4..B-7. (B-3:L1-L10)
- **E-B3-002 — V2 centralized stub contract and offline mode.** `[E]` V2 replaces scattered V1 `Ok(default())` stubs with `crates/dashboard-v2/src/lib/api/stub.ts`; each entry that has a live command must call `invoke('<cmd>', ...)`, while mock fallback is allowed only under `ELAI_DASHBOARD_OFFLINE=1`. The refreshed source reports 198 exports as of 2026-05-31, up from the approximately 60+ authoring baseline. (B-3:L11-L29)
- **E-B3-003 — V2 surface/path rebaseline.** `[E]` Activity is the three-mode orbit, V2 has scaffolded `MonitorWindow.svelte`, `ExploreWindow.svelte`, `PlanWindow.svelte`, `ReviewWindow.svelte`, `SettingsWindow.svelte`, and `CommandWindow.svelte`, and ten named drawers. Exact remaps are V1 `commands.rs` -> V2 `src-tauri/src/lib.rs`; `src-frontend/src/` -> `src/lib/`; ActivityFeed/Entry -> `ActivityWindow.svelte` + orbit/captions; ChatWindow -> CommandWindow; Settings -> SettingsWindow; WindowRegistry -> AppShell routing. (B-3:L30-L62)
- **E-B3-004 — Three V2 disposition classes.** `[E]` Class A means a live Tauri invoke or real producer plus stub swap; Class B deletes stub and consumer, or hides a whole deferred section behind `ELAI_DASHBOARD_DEV=1`; Class C preserves mock fallback but renders `(post-MVP, see B-N)` verbatim. (B-3:L63-L73)
- **E-B3-005 — B-3/B-36 ownership boundary.** `[E]` B-3 owns audit/classification and Class B/C delete-or-relabel work; B-36 owns Class A live wiring, and the same `stub.ts` entries must not be edited twice. (B-3:L75-L77)
- **E-B3-006 — Inventory item 1, real language status.** `[E]` Replace the empty DKG-ready language vector with actual detected languages, falling back to DKG node extensions. The source marks V2 shipped through `summary.registry_status.loaded`, an `Option<LanguageRegistryStatus>` populated by `IndexingService`/`dkg.registry_status()`. (B-3:L113-L123)
- **E-B3-007 — Inventory item 2, PageRank.** `[E]` Implement `get_pagerank_top` using descending `dkg_nodes_pagerank` scores and a limit, while ensuring indexing populates the table. V2 is marked shipped via a table-exists-guarded viewer, Tauri wrapper, and invoke registration. (B-3:L124-L131)
- **E-B3-008 — Inventory item 3, query-tier stats.** `[E]` Aggregate `get_query_tier_stats` from B-4 `QueryTrace` SSB rows and hide the section until its producer exists; V2 viewer/wrapper and `lib.rs` registration are marked shipped. (B-3:L133-L138)
- **E-B3-009 — Inventory item 4, freshness state.** `[E]` Hide the freshness panel when `freshness_meta.db` is uninitialized, then let B-4 wire the producer; the V2 command wrapper and viewer are marked shipped. (B-3:L140-L144)
- **E-B3-010 — Inventory item 5, knowledge degradation.** `[E]` A real `SsbNotifier` must write `knowledge_degradation` when `SearchMode` changes. The refreshed status says no production `NoopSsbNotifier` remains, `SsbDegradationNotifier` exists and is consumed by `spawn_cascade_evaluator_loop`, but the production `cascade.evaluate` caller remains B-4 scope; status is explicitly PARTIAL. (B-3:L146-L156)
- **E-B3-011 — Inventory items 6-8, anti-staleness producer placeholders.** `[E]` `dep_audit`, `baseline_harness`, and `deterministic_audit` are Class C; show `(awaiting producer wire-up)` with B-23/B-4/B-15 links. V2 is marked shipped for typed `DepAuditCompleted`, `BaselineRunCompleted`, and `DeterministicAuditRun` SSB variants/payload plumbing, with `SCHEMA_VERSION` 97 -> 98. (B-3:L158-L170)
- **E-B3-012 — Inventory item 9, real GRC budgets.** `[E]` Replace hardcoded reasoning values with per-agent topic, remaining budget, grounding level, confidence, and elapsed time from `PipelineGrc`; source says V2 threading shipped through C-1. (B-3:L172-L179)
- **E-B3-013 — Inventory item 10, reasoning label.** `[E]` The `reasoning · N topics` label becomes truthful as a consequence of real PipelineGrc data; source marks it shipped with item 9. (B-3:L180-L182)
- **E-B3-014 — Inventory item 11, live rule transitions.** `[E]` Show the last five `rule_transition` states (`PROPOSED→CANDIDATE→SHADOW→ACTIVE→RETIRED`). The shipped V2 implementation deliberately preserves the existing 10-second ref-counted poll rather than the originally proposed 1 second, adds typed SSB/command/state/UI plumbing, and defers the actual rules-crate SSB producer to `rules::lifecycle`. (B-3:L184-L201)
- **E-B3-015 — Inventory item 12, PlanWindow reachability.** `[E]` Register the Plan window rather than routing PLAN to Workspace; V2 resolves this through `WindowHost.svelte`, which mounts `PlanWindow`. (B-3:L203-L210)
- **E-B3-016 — Inventory items 13-15, decomposition readers.** `[E]` Implement `get_decomposition_tree`, `get_phase_detail`, and `get_baby_step_detail` from SSB rows for concept/phase/step/babystep decomposition. V2 Tauri wrappers are marked shipped. (B-3:L212-L218)
- **E-B3-017 — Inventory item 16, handoff schema without ts-rs.** `[E]` Correct the frontend shape from `{id, branch, ts}` to `{seq, ts_ms, winner, abandoned, trunk_snap_id, from_snap_count}`. The refreshed source explicitly reclassifies the solution: hand-authored `api/types.ts`, no `ts-rs` or `tauri-specta`, because adding a dependency would violate workspace policy. (B-3:L220-L235)
- **E-B3-018 — Inventory item 17, real reindex.** `[E]` Replace the fake queued-finding behavior with real reindex plus progress. V2 renames the command to `reindex_now` and marks its settings contract migration shipped. (B-3:L237-L245)
- **E-B3-019 — Inventory item 18, merge event.** `[E]` The old renderer expected a branch-merge event and B-9 amendment originally called for `checkpoint`; the refreshed V2 status instead says `session_dag.rs` emits typed `SsbActionType::BranchMerged`, with the viewer querying both `BranchMerge` and `BranchMerged` as a C-23 supersession pattern. (B-3:L247-L255)
- **E-B3-020 — Inventory item 19, isolation metric removal.** `[E]` Hide the unreachable isolation renderer until B-18 Tier-C IPC interception. V2 meets Class B by lacking `ActivityEntry.svelte` and exposes only `get_isolation_metric_status()` as a defer flag citing B-18. (B-3:L257-L263)
- **E-B3-021 — Inventory item 20, `/continue`.** `[E]` Wire `/continue` to `SessionDag::load_handoff`; V2 `CommandWindow.svelte` invokes `load_handoff` and surfaces typed `DagBranchError` on rejection. (B-3:L265-L270)
- **E-B3-022 — Inventory items 21-24, live FSM UI.** `[E]` B-2 activates the formerly dead IDLE badge, 10-state color switch, EXECUTING pulse, and `MonitorWindow.currently`; V2 component wiring is marked shipped. (B-3:L272-L279)
- **E-B3-023 — Inventory item 25, dashboard fork.** `[E]` Add `/session fork` to the dashboard palette; V2 `CommandPalette.svelte` and `fork_session_branch` are marked shipped. (B-3:L281-L287)
- **E-B3-024 — Inventory item 26, merge/close reachability.** `[E]` Expose merge and close; V2 palette entries and registered `merge_session_branch`/`close_session_branch` commands exist, but the source explicitly calls this PARTIAL because deep producer wiring remains B-9-owned. (B-3:L289-L294)
- **E-B3-025 — Inventory item 27, autonomous-mode gate.** `[E]` Hide AutoAdvance statistics behind the `autonomous-mode` Cargo feature until B-22. V2 uses dual positive/negative `cfg` definitions on one Tauri symbol and frontend `featureFlags.autonomous_mode`. (B-3:L296-L305)
- **E-B3-026 — Inventory item 28, self-modification gate.** `[E]` Hide proposals behind `self-modification`; V2 dual-cfg commands, `featureFlags.self_modification`, and bootstrap through `get_feature_flags` are marked shipped. (B-3:L307-L313)
- **E-B3-027 — Inventory item 29, MONITOR.** `[E]` MONITOR was to be autonomous-only; V2 keeps `MonitorWindow` mounted but gates sensitive AutoAdvance/Proposals subpanels rather than the entire window. (B-3:L315-L318)
- **E-B3-028 — Inventory item 30, EXPLORE.** `[E]` The full Q&A path must eventually use B-26 Oracle Mode or be labeled BM25-only limited mode. V2 keeps `ExploreWindow`; Oracle wiring remains B-26-owned, so the surface is explicitly left intact/pending. (B-3:L320-L325)
- **E-B3-029 — Exact produced-symbol contract.** `[E]` Produced commands are `get_feature_flags`, `get_query_tier_stats`, `get_rule_transitions`; SSB variants are `DepAuditCompleted`, `BaselineRunCompleted`, `DeterministicAuditRun`, `RuleTransition`, `BranchMerged`; corresponding payloads are `DepAuditCompletedPayload`, `BaselineRunCompletedPayload`, `DeterministicAuditRunPayload`, `RuleTransitionPayload`, `BranchMergedPayload`. (B-3:L327-L342)
- **E-B3-030 — Consumption, reachability, and stub bans.** `[E]` Consumers are V2 `run`, stub `get_rule_transitions`, and drawer `lastTransitions`; reachability is through V2 `run` and `SsbActionType`; `get_feature_flags`, `get_query_tier_stats`, and `get_rule_transitions` must not equal `Ok(default())`. (B-3:L343-L354)
- **E-B3-031 — Observable contract.** `[E]` Trigger the `fsm_phase_transition_smoke_ssb` contract, expect `phase_transition`, and require zero unexplained blank/zero/`–` UI elements. (B-3:L355-L358)
- **E-B3-032 — Command-registration gate.** `[E]` `get_query_tier_stats`, `get_rule_transitions`, and `get_feature_flags` must all appear in V2 `invoke_handler!`. (B-3:L360-L368)
- **E-B3-033 — Typed SSB bijection gate.** `[E]` The four newly introduced anti-staleness/rule variants require the complete enum/SQL/string/payload/dispatcher/test-payload plumbing; orchestrator SSB tests expect 55/55 pass and byte-preserving typed round trips with variant-count equality. (B-3:L370-L381)
- **E-B3-034 — Dual-feature compilation gate.** `[E]` Dashboard compilation must pass both with and without `autonomous-mode self-modification`; the dual-cfg/single-registration pattern is an invariant. (B-3:L383-L389)
- **E-B3-035 — Dependency-negation/type-mirror gate.** `[E]` `ts-rs` must have zero dependency hits; `api/types.ts` manually mirrors Rust shapes and auditor review checks the mirror. (B-3:L391-L397)
- **E-B3-036 — No-default and visual gates.** `[E]` V2 commands must have zero non-test `Ok(...::default())` hits, and manual drawer/window smoke must show real data or an explicit awaiting-producer label. (B-3:L399-L407)
- **E-B3-037 — Completion claims.** `[E]` Acceptance is checked for all 30 classifications, all Class A/B/C dispositions, the manual handoff mirror, zero V2 command defaults, real reindex, dashboard fork, and PlanWindow registration/consolidation. (B-3:L409-L423)
- **E-B3-038 — Additional test obligations.** `[E]` Each new producer needs an integration test with triggered real data, and a dashboard E2E task must prove no permanently blank unlabeled element. A retained ts-rs regeneration/CI bullet is stale relative to the source's explicit no-ts-rs decision. (B-3:L425-L432)
- **E-B3-039 — Rust implementation surface.** `[E]` Named shipped surfaces include V2 commands/lib/Cargo features, orchestrator SSB/task-runner/session-DAG changes, and the CLI degradation notifier/loop; exact paths and statuses are enumerated in the source. (B-3:L434-L461)
- **E-B3-040 — Frontend implementation surface.** `[E]` Named shipped surfaces include manual API types, offline-aware `tryInvoke`, feature-flag state, 10-second rules polling, transition UI, Monitor gating, `/continue`, and `/session fork`. (B-3:L463-L489)
- **E-B3-041 — Offline fallback details.** `[E]` `tryInvoke` recognizes either `window.__ELAI_DASHBOARD_OFFLINE__` or `VITE_ELAI_DASHBOARD_OFFLINE`; `get_feature_flags` is exempt, and isolation status remains an explicit defer export. (B-3:L468-L474)
- **E-B3-042 — Decision artifact.** `[E]` `workflow/reasoning/B-3-decisions.md` records decisions A-E plus every per-item classification; the source marks it shipped. (B-3:L491-L493)
- **E-B3-043 — Retired-path non-goals.** `[E]` Do not retarget the removed V1 dashboard/frontend paths; do not add V1 `ts-rs` or generate `types.gen.ts`; V2 commands and hand-authored `api/types.ts` supersede them. (B-3:L495-L502)
- **E-B3-044 — Decision questions retained in plan history.** `[E]` The decision section asks for every item rationale, whether to add ts-rs, and PlanWindow/Workspace coexistence; later source text resolves ts-rs as “no” and V2 routing through `WindowHost`, while preserving the original questions as history. (B-3:L504-L510)
- **E-B3-045 — Follow-on dependencies.** `[E]` B-4 replaces Knowledge Class-A stubs; B-5/B-6/B-7 replace their domain Class-A stubs. (B-3:L511-L514)

#### Ambiguities and conflicts retained

- **C-B3-001.** The original Class-A rule asks for one-second RulesDrawer polling, but the shipped revision explicitly rejects it and preserves ten seconds. The later shipped clause governs the rebaseline; the older task remains historical. (B-3:L186-L201)
- **C-B3-002.** Item 16 initially requires `ts-rs`, the later shipped decision forbids it, the deterministic gate expects no dependency, yet “Tests to add” still demands ts-rs generation. This is an internal stale-section conflict, not an open dependency choice. (B-3:L222-L235, L391-L397, L425-L430, L500-L502)
- **C-B3-003.** B-9's amendment is described here as merge action `checkpoint`, while the refreshed implementation claim uses typed `BranchMerged` and a dual-name viewer. The source does not state whether a separate checkpoint row also emits. (B-3:L249-L255)
- **C-B3-004.** The plan-window heading says “3 items,” but numbered item 12 plus grouped items 13-15 cover four of the 30 inventory numbers. The global 30-item count remains arithmetically consistent. (B-3:L203-L218)
- **C-B3-005.** Acceptance says all Class-A producers are wired, but item 5 is PARTIAL, item 26 is PARTIAL, item 30 remains B-26-pending, and the rules transition producer is deferred. Treat completion as the re-scoped B-3 responsibility, not completion of every downstream producer. (B-3:L146-L156, L199-L201, L289-L294, L320-L325, L411-L423)
- **A-B3-001.** Item 29 originally says hide MONITOR unless autonomous, but the shipped V2 behavior mounts it and gates only sensitive panels. The source presents this as shipped without explicitly reclassifying the whole-window requirement. (B-3:L315-L318)

### B-8 — PipelineGrc Activation in Production Hot Paths

Source: `docs/thefinalglance/B-plan/B-8_pipelinegrc_activation.md`, 153 lines.

#### Explicit ledger (20 atomic records)

- **E-B8-001 — Goal, baseline, dependency, and scope.** `[E]` Give `verify_claim`, `ground_plan`, `generate_csq`, and `check_inline_feasibility` real production callers; they existed as zero-caller wrappers, leaving Phase-2 reasoning invisible and `GroundingSweepGate` vacuously passing because `ctx.ungrounded_claims` stayed empty. Estimate 1-2 sessions; B-5 prerequisite; parallel-safe with B-6/B-7/B-9/B-10. (B-8:L1-L22)
- **E-B8-002 — Plan grounding insertion point.** `[E]` Immediately before evidence evaluation on PLANNING -> TESTING_FIRST, call `pipeline_grc.ground_plan(plan)` and populate `ctx.ungrounded_claims` so `GroundingSweepGate` evaluates real claims. (B-8:L34-L40)
- **E-B8-003 — Claim verification before context fetch.** `[E]` ContextCompiler verifies every phase-plan claim before retrieval; claims failing `FILE_CHECK` or `SOURCE_CHECK` are skipped to avoid loading bogus dependencies and wasting tokens. (B-8:L41-L45)
- **E-B8-004 — CSQ pre-dispatch scope.** `[E]` Before `Write`, `Edit`, or `Bash` reaches the tool layer, call `pipeline_grc.generate_csq(function_signature, function_purpose)`. (B-8:L46-L50)
- **E-B8-005 — CSQ outcome policy.** `[E]` `CRITICAL && !guard_added` blocks the tool call, emits `csq_evaluated`, and surfaces a dashboard intervention; WARNING emits/logs but proceeds; passed proceeds silently. (B-8:L50-L55)
- **E-B8-006 — Inline feasibility outcome policy.** `[E]` During EXECUTING, novel claims such as a new crate/API invoke inline feasibility: Pass proceeds, Fail returns to the original plan approach, and Blocked emits `ReplanRequest(MAJOR)`. (B-8:L57-L64)
- **E-B8-007 — Per-babystep blocked conversion.** `[E]` The EXECUTING handler must convert the already-returned `FeasibilityOutcome::Blocked` into a `ReplanRequest` for each babystep. (B-8:L65-L67)
- **E-B8-008 — Anti-rubber-stamp runtime flow.** `[E]` Spawn the auditor, make it call `pipeline_grc.think()` with `FINDING:`-prefixed Verify steps, require at least one finding before Converge, and emit `auditor_rubber_stamp_rejected` when convergence is rejected. (B-8:L69-L77)
- **E-B8-009 — Phase-local feasibility state.** `[E]` Invoke `InlineFeasibilityState.reset()` at every FSM phase transition so feasibility state cannot bleed across phases. (B-8:L79-L80)
- **E-B8-010 — Exact produced APIs.** `[E]` Contracted methods are `PipelineGrc::{ground_plan,verify_claim,generate_csq,check_inline_feasibility,think}` and `InlineFeasibilityState::reset`. (B-8:L82-L90)
- **E-B8-011 — Hot-path reachability.** `[E]` `GroundingSweepGate` and `task_runner::drive_task` consume the work, and all activation must be reachable from `drive_task`; B-5 is the frontend/event pair. (B-8:L91-L101)
- **E-B8-012 — Stub bans.** `[E]` `ground_plan`, `generate_csq`, `check_inline_feasibility`, and `think` must not remain TODOs. (B-8:L96-L100)
- **E-B8-013 — Observable safety contract.** `[E]` The named safety-gate integration test must produce `csq_evaluated` and a red dashboard intervention card requiring confirm or cancel. (B-8:L102-L106)
- **E-B8-014 — Production-caller evidence gate.** `[E]` A non-test source search must find at least four calls across the five listed `pipeline_grc` methods, one per required activation method. (B-8:L108-L115)
- **E-B8-015 — Non-vacuity gate.** `[E]` A task containing an intentionally ungrounded claim must make `GroundingSweepGate` BLOCK. (B-8:L116-L118)
- **E-B8-016 — Destructive-write gate.** `[E]` A task attempting file deletion must demonstrate that CSQ requires a guard. (B-8:L119-L121)
- **E-B8-017 — Acceptance/done gates.** `[E]` Acceptance requires four production callers, non-empty grounding input, end-to-end Critical-without-guard blocking, surfaced blocked replanning, live anti-rubber-stamp rejection, per-phase reset, and no 2A-2D regressions; every checkbox is unchecked in this source. (B-8:L123-L131)
- **E-B8-018 — Implementation/test surface.** `[E]` Planned changes target `task_runner.rs`, `dispatcher.rs`, the pre-write location (`file_ops.rs` as then named), `evidence.rs`, `agent_pool.rs`, and possibly `grc.rs`; add `tests/contracts/safety_gate_integration.rs` and create `B-8-decisions.md`. (B-8:L133-L142)
- **E-B8-019 — Autonomous CSQ decision unresolved.** `[E]` The plan leaves a policy choice for an absent user: block plus webhook alert versus falling back to a safer approach. (B-8:L144-L147)
- **E-B8-020 — Grounding cadence unresolved and follow-on.** `[E]` The plan leaves open whether grounding fires only at PLANNING -> TESTING_FIRST or on every replan; next steps are B-9 and B-10. (B-8:L148-L153)

#### Ambiguities and conflicts retained

- **A-B8-001.** The objective names four zero-caller methods, while the wiring contract and caller-search also require `think`; “at least four hits” is weaker than proving all five methods have a production caller. (B-8:L1-L5, L84-L100, L111-L115)
- **A-B8-002.** The pre-write insertion point is given as `dispatcher.rs` “or wherever” Write/Edit/Bash routes, and `file_ops.rs` “or wherever pre-write lives”; final ownership is intentionally unresolved. (B-8:L46-L48, L135-L140)
- **A-B8-003.** The source itself does not claim shipped/DONE: all acceptance boxes remain unchecked. Any DONE status comes from the orientation summary, not this canonical chunk source. (B-8:L123-L131)

### B-9 — SessionDag Activation

Source: `docs/thefinalglance/B-plan/B-9_sessiondag_activation.md`, 204 lines.

#### Explicit ledger (24 atomic records)

- **E-B9-001 — Goal, prerequisites, and baseline.** `[E]` Replace legacy JSONL authority with the 3,090-LOC SessionDag, populate `stub_ssb_seq`, expose merge/close, and implement merge amendment #2; estimate 1-2 sessions; prerequisite B-1; parallel-safe with B-4..B-8/B-10. (B-9:L1-L9)
- **E-B9-002 — V2 surface mapping.** `[E]` Producer work is UI-version agnostic; V2 destinations are Tauri `src/lib.rs`, `CommandWindow.svelte`, `SettingsWindow.svelte`, `CommandPalette.svelte`, and a new `components/BranchTree.svelte`; preserve the existing API name `get_session_branches`. (B-9:L11-L42)
- **E-B9-003 — Runtime failure baseline.** `[E]` `stub_ssb_seq: None` makes trim pass 1 produce zero stubs; handoff/recovery/merge/close have zero non-test callers; `/resume` reads a DAG snap id but messages from JSONL; only fork is reachable; disk persistence violates msgpack spec. (B-9:L44-L57)
- **E-B9-004 — Real SSB stub linkage.** `[E]` Every CLI message persistence site must carry the actual preceding tool-result SSB sequence, threaded from the writer's `last_seq`, instead of `None`. (B-9:L68-L73)
- **E-B9-005 — Automatic handoff.** `[E]` In the REPL loop, `should_create_handoff()` fires at 85% of token budget and calls `SessionDag::create_handoff`. (B-9:L75-L77)
- **E-B9-006 — `/continue` recovery.** `[E]` Replace the placeholder with `SessionDag::load_handoff(last_snap_id)` and reconstruct the message stream. (B-9:L79-L81)
- **E-B9-007 — SessionDag authority and legacy export.** `[E]` SessionDag becomes authoritative; `render_jsonl_snapshot` remains only as a deprecated legacy export gated by `--export-jsonl`. (B-9:L83-L87)
- **E-B9-008 — MessagePack BLOB lifecycle.** `[E]` Findings, code changes, and tests written are stored as MessagePack BLOBs, and relevant decode paths migrate from `serde_json::from_slice` to `rmp_serde`; contracted helpers are `encode_blob`/`decode_blob`. (B-9:L88-L91, L128-L136)
- **E-B9-009 — Fine-grained checkpoints.** `[E]` Emit checkpoints after every GRC step, tool-call return, and FSM transition rather than only REPL-turn end. (B-9:L93-L99)
- **E-B9-010 — Dashboard branch actions.** `[E]` Add `/session fork [name]`, `merge <branch_id>`, `close <branch_id>`, and `list`, backed by Tauri wrappers for create/merge/close. (B-9:L101-L106)
- **E-B9-011 — Branch-tree surface.** `[E]` `/session list` renders a compact three-column tree: `branch_id`, status, and last activity. (B-9:L105-L108)
- **E-B9-012 — Merge observability amendment.** `[E]` `merge_branch` must emit an SSB row with action type `checkpoint`, replacing the dead `branch_merge` renderer case. (B-9:L110-L112)
- **E-B9-013 — Handoff type schema.** `[E]` Fix the Settings `SessionHandoffEntry` mismatch through ts-rs-generated bindings, explicitly assuming B-3 added ts-rs. (B-9:L114-L116)
- **E-B9-014 — Interactive handoff picker.** `[E]` CommandWindow `/continue` opens a recent-handoff picker; selecting one calls the load-handoff path. (B-9:L118-L120)
- **E-B9-015 — Budget gauge thresholds.** `[E]` StatusBar shows token-budget utilization, amber at 70% and red at 85%, where auto-handoff is imminent. (B-9:L122-L124)
- **E-B9-016 — Exact SessionDag contract.** `[E]` Produced symbols are `create_handoff`, `load_handoff`, `{merge,close,create}_branch_with_ssb`, `stub_ssb_seq`, `encode_blob`, and `decode_blob`; consumers include `prepare_turn_runtime`, CLI handoff loading, and V2 merge/close commands. (B-9:L126-L141)
- **E-B9-017 — Reachability and bans.** `[E]` Reachability must exist from CLI turn preparation and dashboard merge; production CLI must not pass `stub_ssb_seq: None`; JSONL rendering is export-only; merge and close require UI exposure. (B-9:L142-L149)
- **E-B9-018 — Observable contract.** `[E]` The provided SQLite trigger inserts `branch_merged`, expects that SSB event, and requires a visible branch tree where dashboard fork creates a visible branch. (B-9:L150-L154)
- **E-B9-019 — Deterministic stub check.** `[E]` A source search for `stub_ssb_seq:\s*None` in production CLI must return zero hits. (B-9:L156-L162)
- **E-B9-020 — Production-caller check.** `[E]` Search for SessionDag create/load/merge/close callers across CLI and orchestrator must yield at least four hits. (B-9:L163-L167)
- **E-B9-021 — Msgpack check.** `[E]` Relevant `session_dag.rs` BLOB paths must use `rmp_serde::{from_slice,encode}` in place of JSON decoding. (B-9:L168-L171)
- **E-B9-022 — Acceptance/done gates.** `[E]` Acceptance requires working stub recovery, 85% handoff, SessionDag `/continue`, msgpack, three checkpoint cadences, four dashboard actions, merge checkpoint emission, corrected handoff schema, StatusBar gauge, and CommandWindow handler; all boxes are unchecked. (B-9:L173-L184)
- **E-B9-023 — Implementation surface.** `[E]` Planned changes cover CLI main, SessionDag/session/task-runner, V1-named dashboard command/frontend paths, a new BranchTree, and `B-9-decisions.md`; the V2 rebaseline at the top supplies their actual destinations. (B-9:L186-L200)
- **E-B9-024 — Follow-on.** `[E]` Next is B-10 ContextBudget profile activation. (B-9:L202-L204)

#### Ambiguities and conflicts retained

- **C-B9-001.** The current B-3 source explicitly rejects ts-rs and uses a hand-authored `SessionHandoffEntry`; B-9 still assumes B-3 added ts-rs. The dependency assumption is stale. (B-9:L114-L116; B-3:L222-L235, L391-L397)
- **C-B9-002.** Merge event names conflict within the chunk: B-9 requires `checkpoint`, B-9's observable trigger inserts/expects `branch_merged`, and B-3 reports shipped `BranchMerged` plus a compatibility query. These may be multiple rows, but the sources do not say so. (B-9:L110-L112, L150-L154; B-3:L247-L255)
- **A-B9-001.** The source itself has unchecked acceptance and no shipped annotation; summary-level DONE plus “dashboard DAG writes deferred” is external orientation status, not established by this file. (B-9:L173-L184)
- **A-B9-002.** The reads/context mention `recover_stubs`, but no numbered task explicitly adds a production caller other than the implied trim/recovery workflow. (B-9:L52-L55, L68-L81)

### B-13 — Activity Feed Restoration, V2-replaced by Orbit

Source: `docs/thefinalglance/B-plan/B-13_activity_feed_restoration.md`, 201 lines.

#### Explicit ledger (29 atomic records)

- **E-B13-001 — Historical objective and dependencies.** `[E]` The original goal was text-feed auto-scroll, a Jump-to-Latest pill, preview links, and correct handoff types; estimate one re-scoped session; prerequisites B-3/B-12; parallel-safe with B-14..B-20. (B-13:L1-L9)
- **E-B13-002 — Authoritative V2 replacement.** `[E]` V2 retires the text feed; `ActivityWindow.svelte` is the deterministic orbit visualizer. The original text-feed task list does not apply to V2. (B-13:L11-L16, L30-L32)
- **E-B13-003 — PLAN orbit.** `[E]` PLAN renders Concept -> Phase -> Step -> BabyStep as a constellation, keeps the active concept centered, and uses backdrop dots for every step and baby step. (B-13:L18-L20)
- **E-B13-004 — MAP orbit.** `[E]` MAP renders an existing-project code map: PageRank-ranked symbols in score-banded rings with faint DKG edges. (B-13:L21-L22)
- **E-B13-005 — RESEARCH orbit.** `[E]` RESEARCH renders `FORMULATE -> SEARCH -> EXTRACT -> REFLECT -> CONVERGE` with phi/psi convergence micro-bars. (B-13:L23-L24)
- **E-B13-006 — Deterministic mode resolution.** `[E]` Explicit `mode` wins; otherwise `fixture.fsm` maps RESEARCHING to research, IDLE plus pagerank to map, and everything else to plan. (B-13:L26-L28)
- **E-B13-007 — Caption rules.** `[E]` `lib/orbit/captions.ts` derives captions without an LLM from `(fsm, grc.step_type, bubble_action?)`; missing DESIGN section-8 rules must be filled in. (B-13:L34-L37)
- **E-B13-008 — Bubble-up strip.** `[E]` New `BubbleUpStrip.svelte` consumes SSB `bubble_action` and displays the thin progression `BabyStep×3 -> Step×2 -> Phase×1 -> Concept×1 -> Escalate`. (B-13:L38-L41)
- **E-B13-009 — Orbit signal layers.** `[E]` `OrbitGrcDots`, `OrbitBudgetRing`, and `OrbitEvidenceGates` already exist as scaffolds and receive real B-5/B-6/B-10 producer data. (B-13:L42-L44)
- **E-B13-010 — Idle shooting star.** `[E]` Verify `OrbitShootingStar` fires approximately every 14 seconds from a fire counter and entropy-quarantined LCG. (B-13:L45-L47)
- **E-B13-011 — Handoff schema ownership.** `[E]` The Settings schema fix is moved out to B-3 and V2 `src/lib/api/types.ts`. (B-13:L48-L49)
- **E-B13-012 — Hover-reveal chrome.** `[E]` Activity header `CONCEPT …` and development mode pill `ORBIT …` must have opacity zero until panel hover, per DESIGN section 7. (B-13:L50-L53)
- **E-B13-013 — Retired surface map and non-goal.** `[E]` V1 ActivityFeed/ActivityEntry/JumpToLatest/FilterStrip are retired; V2 ActivityWindow is the orbit host. A classic text feed alongside the orbit requires a new B-N and is explicitly outside B-13. (B-13:L55-L71)
- **E-B13-014 — Retired task 1.** `[E, retired]` The old text design made ActivityFeed primary and constrained autonomy/feasibility/violations/proposals/interventions to a right/top rail of at most 30%. V2 replacement invalidates this task. (B-13:L95-L100)
- **E-B13-015 — Retired task 2.** `[E, retired]` The old feed followed the bottom, froze when scrolled up, counted new rows in a floating Jump-to-Latest pill, and smooth-scrolled on click. (B-13:L102-L104)
- **E-B13-016 — Retired task 3.** `[E, retired]` Previewable file/image/diff output produced a teal `[preview] <path>` link opening REVIEW. (B-13:L106-L108)
- **E-B13-017 — Retired task 4.** `[E, retired]` Reasoning entries defaulted to a 2-3-line summary and expanded to the complete real B-5 GRC chain. (B-13:L110-L112)
- **E-B13-018 — Retired task 5.** `[E, retired]` The executing final row pulsed a teal-soft border from 15% to 40% opacity over two seconds. (B-13:L114-L115)
- **E-B13-019 — Retired task 6.** `[E, retired]` Step headers used a thin teal separator, primary current text, secondary completed text, and one-line collapsed completion with pass/fail. (B-13:L117-L119)
- **E-B13-020 — Retired task 7.** `[E, retired]` Entry treatments were teal-ghost reasoning, collapsed diff edit with `+N -M`, compact validate, expandable pass/fail tests, and purple-ghost cloud with cost. (B-13:L121-L127)
- **E-B13-021 — Retired task 8.** `[E, retired]` Settings was to render the six-field Rust handoff shape via ts-rs. Ownership moved to B-3, whose refreshed solution is manual TypeScript. (B-13:L129-L131)
- **E-B13-022 — Retired task 9.** `[E, retired]` Optional per-entry-type filters were session-persisted. (B-13:L133-L135)
- **E-B13-023 — Retired task 10.** `[E, retired]` Ctrl+F searched/highlighted feed entries by paths and error messages. (B-13:L137-L138)
- **E-B13-024 — Stale wiring contract.** `[E, retired]` The retained contract produces feed placement, preview renderer, Jump pill, filter, and search; consumes SSB tail and REVIEW; bans buried feed/missing Jump pill; expects a preview within two seconds. It describes the retired V1 scope, not the V2 rebaseline. (B-13:L140-L160)
- **E-B13-025 — Stale deterministic checks.** `[E, retired]` Manual feed placement/scroll checks and a V1 ts-rs import search remain in the file despite the replacement. (B-13:L162-L175)
- **E-B13-026 — Stale acceptance gates.** `[E, retired]` All nine feed-oriented acceptance bullets remain unchecked; they cover layout, scrolling, preview, reasoning collapse, live pulse, schema, filters, and search. (B-13:L177-L187)
- **E-B13-027 — Stale implementation surface.** `[E, retired]` The retained files list modifies V1 ActivityFeed/Entry/Settings/ActivityWindow, adds Jump/Filter components, and creates `B-13-decisions.md`; the V2 path map says these frontend targets are retired. (B-13:L189-L197)
- **E-B13-028 — Historical feed semantics.** `[E, retired]` The old spec described `[reasoning]`, `[edit]`, `[validate]`, `[test]`, and `[cloud]`, collapsed reasoning, and file-path navigation to REVIEW; this is context for the superseded design, not a current V2 promise. (B-13:L73-L85)
- **E-B13-029 — Follow-on.** `[E]` Next is B-14 live token streaming chat. (B-13:L199-L201)

#### Ambiguities and conflicts retained

- **C-B13-001.** Lines 30-32 unambiguously say the original list does not apply, but lines 95-197 retain the complete V1 tasks, contract, checks, acceptance, and file list without deletion. The V2 rebaseline is authoritative; retained material is historical/stale, not concurrently required. (B-13:L30-L32, L95-L197)
- **C-B13-002.** The prerequisite and retired task still say B-3 fixes types via ts-rs, whereas refreshed B-3 explicitly declines ts-rs. (B-13:L7-L8, L48-L49, L129-L131; B-3:L222-L235)
- **A-B13-001.** The source has no rebaselined V2 acceptance checklist and makes no shipped claim for captions, strip, layers, shooting star, or hover chrome. Summary-level “re-scoped DONE” cannot identify which of these six checks actually passed. (B-13:L30-L53, L177-L187)

### B-15 — Phase 0 Debt Burndown

Source: `docs/thefinalglance/B-plan/B-15_phase0_debt_burndown.md`, 469 lines. This source carries a **2026-07-03** completion-drive-2 rebaseline and supersedes its own older baseline.

#### Explicit ledger (36 atomic records)

- **E-B15-001 — Current status ledger.** `[E]` Of 18 tasks: 11 are SHIPPED-ON-V2 (4,7,9-14,16-18); five are deferred by handoff (1,2,3,5,15); task 6 is ACTIVE-CLOSED for 8 of 17 production files but has 21 regrown violations; task 8 is REGRESSED+DEFERRED after reduction to one leak. A new CLI-size tripwire fired. (B-15:L1-L13)
- **E-B15-002 — Objective, history, and concurrency.** `[E]` Preserve the Phase-0 cleanup goal: dead-code removal, deterministic-map conversion, entropy quarantine, CLI decomposition, and missing scripts. Work had already spanned five named subphases plus completion-drive 1; no prerequisites; parallel-safe with B-1..B-14 and B-16..B-20. (B-15:L15-L30)
- **E-B15-003 — Reverified baseline.** `[E]` Current facts asserted by the source are: `compact.rs` 819 LOC live/deferred; `usage.rs` 98 LOC/deferred with pricing stripped; `oauth.rs` 652 LOC live/deferred; `remote.rs` deleted; `prompt.rs` 829 LOC live/deferred; 21 production HashMap violations plus 10+ unenforced HashSet residuals; one entropy leak; CLI main 1,632 LOC; tools relocation/scripts/metrics/hooks shipped; msgpack deferred. (B-15:L32-L67)
- **E-B15-004 — Task 1, compact defer.** `[E]` Do not blindly delete `compact.rs`: six live callers include `compact_history`, the public module, `/compact`, `ConversationRuntime::compact`, and two transitive sites. B-9 must retire them first; status DEFERRED -> B-9. (B-15:L83-L93)
- **E-B15-005 — Task 2, usage defer.** `[E]` Pricing helpers (`ModelPricing`, `UsageCostEstimate`, `pricing_for_model`, `format_usd`) are already stripped, but four token fields plus `UsageTracker`, 12 importers, `/cost`, and JSON `estimated_cost` remain. B-27 `CLOUD_SPEND` must replace them before the `<30 LOC`/two-field target; status PARTIAL/DEFERRED -> B-27. (B-15:L95-L105)
- **E-B15-006 — Task 3, OAuth defer/security bug.** `[E]` `oauth.rs` has 15 production callers across providers and `/login`, plus a live PowerShell-HOME defect near `credentials_home_dir`; B-27 owns coordinated auth removal. (B-15:L107-L114)
- **E-B15-007 — Task 4, remote removal.** `[E]` `orchestrator/src/remote.rs` is absent and marked SHIPPED-ON-V2 from B-15A session `s-28972`. (B-15:L116-L118)
- **E-B15-008 — Task 5, prompt defer.** `[E]` `prompt.rs` has six system-prompt builders plus ConversationRuntime construction; B-15D must migrate consumers to config/Rust before deletion. (B-15:L120-L126)
- **E-B15-009 — Deterministic map policy and corrected provenance.** `[E]` The plan retracts an ungrounded CLAUDE.md citation and roots the ban in audit/gotchas. Use `BTreeMap` when iteration/cross-process serialization matters, `FxHashMap` for lookup-only hot paths, and `IndexMap` for insertion order. HashMap/FxHashMap are not allowed for JSON ordering. (B-15:L128-L145)
- **E-B15-010 — Task 6 conversions.** `[E]` Explicit conversions include `file_plan::LanguageLimitsMap`, knowledge scores, fixtures, Firecracker snapshots, checker registry, compiled-pipeline maps/store, and CLI breaker states to `BTreeMap`; oracle LRU/background tasks/plugin registry to `FxHashMap`; an unused dispatch import is removed. Cascading tests/callers and `SnapshotKey: Ord` are named. (B-15:L146-L162, L416-L432)
- **E-B15-011 — Task 6 allowlist and residual lifecycle.** `[E]` Intentional retentions are `dual_ledger.rs` (serde IPC boundary), `reasoning_template.rs` (documented link), and test-only `sidecar_xgrammar_hook.rs`. Remaining 21 violations (16 imports, five qualified paths) span plugins/knowledge/providers/safety/orchestrator/dashboard-v2 and are routed to `B-15-followup`; dashboard Tauri files are coordination-excluded. (B-15:L163-L175)
- **E-B15-012 — Task 7, CI HashMap lint.** `[E]` Windows CI runs executable `scripts/check_hashmap_ban.sh`; source marks it shipped in B-15B and batch G. (B-15:L177-L180)
- **E-B15-013 — Task 8 entropy history.** `[E]` Entropy debt moved from 40+ original sites to 28 after C-GAPS, 16 after batch G, regressed to 82 files/192 occurrences, then fell to one `Instant::now` at `task_runner.rs:15311`. (B-15:L182-L197)
- **E-B15-014 — Task 8 recovery/defer behavior.** `[E]` Root cause was post-plan hot-path producers adding direct time calls. This completion drive performs no conversion; the single remaining site is routed to a one-session `B-15-regression-burndown`, and dashboard command files remain excluded. (B-15:L199-L213)
- **E-B15-015 — Injected clock capability.** `[E]` Stable `ClockProvider` exposes `now`, `elapsed_since`, `system_now`, `instant_now`, `now_ms`, and `now_secs`; `SeededClock` is the production deterministic implementation. (B-15:L215-L218)
- **E-B15-016 — Tasks 9-11, verification scripts.** `[E]` Executable shipped scripts are `check_entropy_quarantine.sh` (with unused `ELAI_DETERMINISM_SEED` as doc-only), `determinism_proof.sh`, and `smoke_test_ollama.sh` (fallback substring fragility is doc-only). (B-15:L220-L232)
- **E-B15-017 — Task 12, CLI decomposition regression.** `[E]` The prior 555-line state was accepted at 11% over a 500 target, but current `main.rs` is 1,632 lines after H/I/J/K/L dispatch arms and crosses the 600 tripwire. Six sibling modules remain (`repl.rs`, `commands.rs`, `sidecar.rs`, `indexing.rs`, `handoff.rs`, `dashboard.rs`); a new CLI-shrink plan must return it below 600. (B-15:L234-L251)
- **E-B15-018 — Task 13, ownership relocation.** `[E]` `file_ops.rs`, `bash_validation.rs`, and `stale_branch.rs` exist under `crates/tools/src` and are absent from orchestrator; marked shipped in batch G. (B-15:L253-L259)
- **E-B15-019 — Task 14, duplicate deletion.** `[E]` Only tools' `bash_validation.rs` remains; move-not-copy semantics subsume duplicate removal. (B-15:L261-L263)
- **E-B15-020 — Task 15, session msgpack defer.** `[E]` Legacy disk-session MessagePack work is deferred to B-9's SessionDag takeover; JSONL survives only behind `--export-jsonl`. (B-15:L265-L271)
- **E-B15-021 — Task 16, portable shell execution.** `[E]` `hooks.rs::shell_command()` uses `cmd /C` on Windows and `sh -c` elsewhere; source marks the hard-coded Windows `sh` bug shipped/fixed, with `sh -c` vs plugin `sh -lc` noted but nonbreaking. (B-15:L273-L281)
- **E-B15-022 — Task 17, baseline metric.** `[E]` `.elai_cc/metrics/pre_phase_0.toml` exists and is marked shipped; a warnings-field naming nit is nonblocking. (B-15:L283-L285)
- **E-B15-023 — Task 18, Windows CI workflow.** `[E]` The workflow runs workspace check, HashMap and entropy checks, anti-stale lint, hooks compatibility, and wiring-contract audit; placement was disputed then upgraded to IMPLEMENTED. (B-15:L287-L293)
- **E-B15-024 — Rebaselined produced/reachable symbols.** `[E]` CLI symbols are `run_repl`, `commands.rs`, `run_remote_sidecar`, `indexing.rs`, `list_managed_sessions`, and `run_dashboard`; consumed by 1,632-line `main.rs::run` and updated contracts; reachable from CLI `main` and Windows CI. (B-15:L295-L310)
- **E-B15-025 — Rebaselined stub-ban/defer state.** `[E]` Presence of compact/oauth/prompt/usage is allowed only with caller counts and named handoffs; remote must be absent; exactly the routed 21 map violations and one entropy leak may remain; CLI size explicitly fails its tripwire. Blind deletion is itself documented as a recurring failure. (B-15:L311-L328)
- **E-B15-026 — Observable contract.** `[E]` The named FSM smoke test expects a green build, only routed HashMap residuals, exactly one entropy leak, and `phase_transition` SSB evidence. (B-15:L329-L334)
- **E-B15-027 — Dead-code/usage deterministic gates.** `[E]` Verify remote absent; compact/oauth/prompt present with defer labels; `usage.rs` exactly 98 lines with pricing stripped and four-field tracking retained. (B-15:L336-L348)
- **E-B15-028 — Debt-count deterministic gates.** `[E]` HashMap lint is expected to report 21 routed violations; entropy lint exactly one named leak; CLI `wc -l` exactly 1,632 and over tripwire. These are evidence of known debt, not passing-zero gates. (B-15:L350-L364)
- **E-B15-029 — Artifact/path deterministic gates.** `[E]` All four scripts (including HashMap lint) are executable; pre-phase TOML and Windows workflow exist; three tools files exist and orchestrator copies do not; Windows cfg exists in hooks. (B-15:L366-L388)
- **E-B15-030 — Acceptance state.** `[E]` Checked: remote deletion, tools relocation, duplicate deletion, hook fix, metric, Windows CI, scripts, and CI-wired map lint. Deferred/partial/regressed/tripwire statuses remain explicit for the other tasks; this is not full B-15 completion. (B-15:L390-L410)
- **E-B15-031 — Completion-drive mutation boundary.** `[E]` This completion drive changed only the plan markdown; prior batch-G references list map conversions, the lint allowlist, rewritten decisions, and new followup. (B-15:L412-L440)
- **E-B15-032 — Decision artifact contents.** `[E]` `B-15-decisions.md` records shipped/deferred markers, per-file map rationale, zero-conversion entropy scope lock, CLI tripwire rationale, and the follow-up pointer. (B-15:L442-L456)
- **E-B15-033 — Regression follow-up.** `[E]` `B-15-regression-burndown` is a one-session targeted `ClockProvider` thread-through for `task_runner.rs:15311`. (B-15:L458-L462)
- **E-B15-034 — CLI-shrink follow-up.** `[E]` A new plan must move H/I/J/K/L dispatch arms into siblings and reduce CLI main from 1,632 to under 600. (B-15:L463-L465)
- **E-B15-035 — HashMap coordination follow-up.** `[E]` Close 16 import plus five qualified residuals with B-3/B-4/B-7 maintainers, respecting dashboard coordination. (B-15:L466-L468)
- **E-B15-036 — Sequence follow-on.** `[E]` Continue to B-16 anti-stale enforcement under the original sequencing. (B-15:L469)

#### Ambiguities and conflicts retained

- **C-B15-001.** Task 12 is tagged SHIPPED-ON-V2 even though the current 1,632-line state explicitly supersedes the rationale that made 555 acceptable. Interpret “shipped” as historical decomposition infrastructure, not current satisfaction of the size gate. (B-15:L236-L251, L401-L403)
- **A-B15-001.** The status header says task 6 closed “8 of 17 prod files,” but the conversion list names more than eight top-level files/cascades. The source does not define its file-count denominator precisely. (B-15:L5-L7, L146-L162)
- **A-B15-002.** The source says the original objective had 84 HashMap imports, while current lint reports 21 violations and 10+ HashSet residuals. These are different snapshots and rule scopes, not directly comparable completion percentages. (B-15:L20-L23, L49-L56)
- **A-B15-003.** File byte/line values and current violation counts are source assertions; this deep read did not open implementation paths, per chunk isolation. (B-15:L85-L124, L168-L180, L220-L247)

### B-30 — Multi-Language DKG Hardening

Source: `docs/thefinalglance/B-plan/B-30_multilang_dkg_hardening.md`, 443 lines.

#### Explicit ledger (42 atomic records)

- **E-B30-001 — Goal, breadth, dependency, and schedule.** `[E]` Extend DKG beyond Rust to Python, TypeScript, TSX, JavaScript, Go, Java, Kotlin, Scala, C, C++, C#, Ruby, PHP, Swift, Bash, Svelte, and Vue; include isolated grammar failures, module resolvers, TS aliases, and composite reparse. Estimate 3-4 sessions/18-24 hours; B-1 prerequisite; B-29 needed for traversal but extraction can land independently; parallel-safe with B-31..B-35. (B-30:L1-L11)
- **E-B30-002 — Problem/effect baseline.** `[E]` The asserted Rust-only baseline produced synthetic module-only non-Rust graphs, hollow research context, and empty B-29 caches; the stated objective is language-real system behavior rather than spec-only claims. (B-30:L13-L27)
- **E-B30-003 — Engine constraint and borrowed pattern.** `[E]` Retain tree-sitter and the existing SQLite/Language-ID investment; do not replace it with ast-grep. Port only per-grammar isolation, module resolvers, and composite-language reparse patterns. (B-30:L29-L46)
- **E-B30-004 — `LanguageGrammar` contract.** `[E]` Audit/stabilize `extract_imports -> Vec<ImportInfo>`, `extract_symbols -> Vec<DkgNode>`, and `extract_calls -> Vec<RawCall>`; reshape if required for B-29's per-file `ExtractedSymbols { symbols, raw_calls }`. (B-30:L68-L78)
- **E-B30-005 — Isolated registry lifecycle.** `[E]` `LanguageRegistry::register(Box<dyn LanguageGrammar>) -> Result<(), GrammarLoadError>` wraps each `Parser::set_language` in `catch_unwind`; one panic cannot abort other registrations. Track `loaded: BTreeSet<String>` and `failed: BTreeMap<String,String>` and expose `LanguageRegistryStatus { loaded: Vec<String>, failed: Vec<(String,String)> }`. (B-30:L80-L88)
- **E-B30-006 — Grammar observability surface.** `[E]` KnowledgeDrawer displays loaded/failed grammar counts and reasons; emit one startup `LanguageGrammarStatus` SSB action. (B-30:L90-L93)
- **E-B30-007 — Python extractor.** `[E]` Handle import/import-from, functions, classes, decorated definitions with decorator preservation, class-qualified methods, and calls with attribute resolution. (B-30:L95-L106)
- **E-B30-008 — TypeScript/TSX/JavaScript extractor.** `[E]` Handle imports, re-exports, `require`, dynamic import, classes/functions, const arrow functions, interfaces, aliases, enums, and class methods; keep distinct JavaScript/TypeScript/TSX grammar IDs. (B-30:L108-L117)
- **E-B30-009 — Go extractor.** `[E]` Handle grouped/single imports, functions, methods, and struct/interface/alias type declarations; module constants may be skipped unless completeness requires them. (B-30:L119-L125)
- **E-B30-010 — JVM extractor family.** `[E]` One module shares approximately 80% logic across three IDs: Java classes/interfaces/enums/methods/imports; Kotlin adds objects/functions; Scala handles classes/objects/traits/functions. (B-30:L126-L133)
- **E-B30-011 — C/C++ extractor.** `[E]` Handle local/system preprocessor includes, functions, structs, enums, and C++-only classes/namespaces. (B-30:L135-L140)
- **E-B30-012 — C# extractor.** `[E]` Handle `using`, classes, interfaces, methods, and both block and C#10+ file-scoped namespaces. (B-30:L141-L145)
- **E-B30-013 — Ruby/PHP/Swift/Bash extractors.** `[E]` Ruby supports class/module/method/singleton method; PHP function/class/method/trait/namespace; Swift function/class/struct/protocol/extension; Bash functions only. (B-30:L147-L154)
- **E-B30-014 — JVM suffix resolver.** `[E]` Scan `.java/.kt/.kts/.scala`, index path suffix after `/src/main/<lang>/`, use first match to avoid duplicate-class ambiguity, and consult the map during JVM import resolution. (B-30:L156-L167)
- **E-B30-015 — Deterministic C# namespace resolver.** `[E]` Regex block/file-scoped namespaces into `BTreeMap<String,Vec<String>>`; process files lexicographically, return deterministic candidate lists for `using`, and require byte-identical output across runs. (B-30:L169-L180)
- **E-B30-016 — Go module resolver/failure fallback.** `[E]` Parse root `go.mod` module prefix, strip it from imports, and resolve under project root; absent `go.mod` logs WARN and treats imports as external. (B-30:L182-L190)
- **E-B30-017 — TS/JS config alias resolver.** `[E]` Read `tsconfig.json` and `jsconfig.json` (prefer jsconfig for JS), parse `compilerOptions.paths`, follow `extends` with a visited-set cycle guard, and consult alias targets before filesystem fallback. (B-30:L192-L203)
- **E-B30-018 — Svelte/Vue composite reparse.** `[E]` Parse HTML `script_element` nodes, extract `raw_text`, reparse each as TypeScript, reuse the TypeScript extractor, and qualify symbols as `<file>.svelte::<symbol>` to avoid sibling collisions. (B-30:L205-L216)
- **E-B30-019 — CSS-style imports.** `[E]` Parse `style_element` raw text for `@import` and `@require`, skip HTTP(S), and emit edges flagged `is_css_import=true`. (B-30:L218-L223)
- **E-B30-020 — Default registry order and degraded loading.** `[E]` Register, in order, rust/python/typescript/javascript/tsx/go/java/kotlin/scala/c/cpp/csharp/ruby/php/swift/bash/svelte/vue; a missing/lagging grammar such as PHP fails locally and does not break others. (B-30:L224-L230)
- **E-B30-021 — Node-language schema.** `[E]` Verify DKG nodes already record `language`; if not, add migration v4. (B-30:L232-L233)
- **E-B30-022 — Per-language metrics.** `[E]` Update `.elai_cc/metrics/phase_1A.toml` with per-language nodes/edges and unresolved-import percentage, measured on a mixed Rust/Python/TS fixture. (B-30:L235-L238)
- **E-B30-023 — Extension dispatch/fallback node.** `[E]` `DkgEngine::{extract_symbols,resolve_imports}` chooses a registry grammar by extension; unloaded grammar uses a no-op extractor that still creates a synthetic `<module>` symbol. (B-30:L240-L244)
- **E-B30-024 — Real-codebase validation.** `[E]` Exercise SocratiCode TS, a Python project, Maven multi-module Java, file-scoped C#, and the Svelte dashboard. (B-30:L246-L253)
- **E-B30-025 — Exact produced-symbol inventory.** `[E]` The contract names the registry, 17 non-Rust grammar structs, `JvmResolver`, `CsNamespaceResolver`, `GoModuleResolver`, `TsPathAliasResolver`, and `SsbActionType::LanguageGrammarStatus`. (B-30:L255-L280)
- **E-B30-026 — Consumption and reachability.** `[E]` `IndexingService::build_index` dispatches extraction; `resolve_import` consults four resolvers; dashboard `get_language_status` reads SSB; entry paths are CLI index build, dashboard run, and indexing service. (B-30:L281-L291)
- **E-B30-027 — Stub/health minimums.** `[E]` Healthy startup requires at least 10 loaded grammars out of 13+ registered, an explicit error for every failed grammar, and populated `per_language_node_counts`. (B-30:L292-L298)
- **E-B30-028 — Observable effect.** `[E]` A release index build must emit `language_grammar_status`; phase-1A metrics must show nonzero nodes for at least five languages. (B-30:L299-L302)
- **E-B30-029 — Compile/registry checks.** `[E]` Knowledge tests must compile, and `test_default_registry_loads_all` must pass. (B-30:L304-L313)
- **E-B30-030 — Extraction fixture check.** `[E]` Every language fixture must yield symbols and the expected qualified-name set. (B-30:L315-L318)
- **E-B30-031 — Resolver determinism checks.** `[E]` JVM maps must be byte-identical over 100 runs; C# candidate lists must be lexicographic. (B-30:L320-L326)
- **E-B30-032 — Mixed-project end-to-end check.** `[E]` Index the bundled multilingual fixture and query DKG counts; at least five languages must be nonzero. (B-30:L328-L332)
- **E-B30-033 — Failure-isolation check.** `[E]` Corrupt one loader and verify the other 12 of 13 continue to load unaffected. (B-30:L334-L337)
- **E-B30-034 — Acceptance/done gates.** `[E]` Acceptance covers isolated registry, all listed extractors/resolvers, composite names, metrics, dashboard status, startup event, nonempty B-29 caches, zero non-test unwraps, and no unapproved crates; every checkbox is unchecked. (B-30:L340-L355)
- **E-B30-035 — Required test suite.** `[E]` Add per-language extraction, registry-failure injection, JVM/C#/Go/TS-config/Svelte resolver tests, plus `multilang_dkg_contract` for nonempty per-language metrics. (B-30:L357-L376)
- **E-B30-036 — Implementation surface.** `[E]` Create eleven language files, four resolver files, registry, multilingual fixture, decisions; modify language module, engine, SSB actions, and V2 KnowledgeDrawer. (B-30:L378-L407)
- **E-B30-037 — Engine-choice decision.** `[E]` The decision doc must record why tree-sitter remains despite the reference's ast-grep implementation. (B-30:L409-L416)
- **E-B30-038 — Alias-order decision.** `[E]` Precisely decide project tsconfig -> jsconfig -> workspace root -> filesystem fallback ordering. (B-30:L417-L418)
- **E-B30-039 — Qualified-name decision.** `[E]` Choose `<file>.svelte::<symbol>` versus `<file>::<symbol>` because B-29 index identity depends on it. (B-30:L419-L420)
- **E-B30-040 — Language-scope decision.** `[E]` Decide whether regex-only Dart/Lua/R join this step or are deferred. (B-30:L421-L422)
- **E-B30-041 — Explicit downstream benefits.** `[E]` Unlock non-Rust B-29 graphs, existing-project decomposition, grounded `/research`, accurate Project Intelligence language breakdown, honest Phase-1A metrics, and mixed-language Empire interop. (B-30:L424-L435)
- **E-B30-042 — Sequencing.** `[E]` B-29 should land first or in parallel; B-31 is independent; B-33 benefits through language-colored graph nodes. (B-30:L437-L443)

#### Ambiguities and conflicts retained

- **A-B30-001.** The objective/list has 17 non-Rust labels plus Rust (18 registry entries), while prose and health gates repeatedly say “13+” or “13/15.” These are minimum/legacy counts, not the full declared supported set. (B-30:L1-L7, L90-L93, L226-L230, L293-L294)
- **A-B30-002.** The trait audit is conditional: the final signature may be the three listed methods or a B-29-shaped `ExtractedSymbols` refactor. (B-30:L72-L78)
- **A-B30-003.** `catch_unwind` is directed in the task but the decision doc says empirically choose catch-unwind versus Result-only, defaulting to catch-unwind. Final failure-isolation mechanism is not fully closed. (B-30:L80-L88, L411-L416)
- **A-B30-004.** TS config precedence is partly stated (“prefer jsconfig for JS”) but the decision section proposes a different explicit sequence beginning with project tsconfig; final ordering must be documented. (B-30:L192-L203, L417-L418)
- **A-B30-005.** The source is a planned specification with all acceptance boxes unchecked; it does not claim implementation. (B-30:L340-L355)

### B-37 — Phase 4M Multi-Runtime Backend Abstraction

Source: `docs/thefinalglance/B-plan/B-37_multi_runtime_backend.md`, 254 lines.

#### Explicit ledger (28 atomic records)

- **E-B37-001 — Goal, prerequisites, and concurrency.** `[E]` Create one `RuntimeAdapter` for 11 backends, per-role hybrid routing, a backend fallback chain separate from model fallback, and Tier-C isolation for every CLI subprocess. Estimate 3-4 sessions; prerequisites B-1, B-15..B-20, B-22, B-18, and completed Phase 3J variants; parallel-safe with B-23/B-24/B-25/B-27/B-28. (B-37:L1-L12)
- **E-B37-002 — Runtime architecture.** `[E]` Orchestrator emits Anthropic-shaped messages once; adapters translate protocols, enabling combinations such as cloud Designer, local Executor, and Claude-Code Auditor without orchestrator forks. (B-37:L14-L33)
- **E-B37-003 — Exact backend set.** `[E]` Existing: `Sidecar`, `AnthropicApi`; CLI ports: `ClaudeCode`, `CodexCli`, `OpenCode`, `Hermes`, `GeminiCli`, `KiroCli`, `CopilotCli`; native HTTP: `Ollama`, `Vllm`. (B-37:L35-L39)
- **E-B37-004 — Process boundary/security.** `[E]` All seven CLI adapters run inside Phase-4A Tier C; the CLI binary is the agent boundary and existing IPC sanitization prevents raw stdout from entering prompt context. (B-37:L41-L44)
- **E-B37-005 — V2 control surface.** `[E]` Add RuntimePanel to V2 Settings plus Tauri `get_runtime_status`, `set_role_runtime`, and `test_runtime_health`. (B-37:L46-L52)
- **E-B37-006 — Third-party provenance artifact.** `[E]` Run source-mode paper ingestion over specified Ouroboros files and produce `workflow/papers/ouroboros-runtime-interface.md` tagged MEDIUM because it is reviewed third-party MIT code. (B-37:L63-L68)
- **E-B37-007 — Core types.** `[E]` Add the 11-variant `RuntimeBackend` and `RuntimeAdapter`; alias to `LlmProvider` only if vtable-compatible, otherwise use a thin wrapper. (B-37:L70-L72)
- **E-B37-008 — Resolver behavior.** `[E]` `resolve_backend(candidate)` returns a typed `ResolveBackendError` containing the complete supported list on an unknown name. (B-37:L74-L76)
- **E-B37-009 — Runtime factory lifecycle.** `[E]` `create_runtime(RuntimeFactoryConfig) -> Box<dyn RuntimeAdapter>` initially ports Sidecar/Anthropic and returns stub errors for new backends until their adapters land. (B-37:L78-L80)
- **E-B37-010 — One-release configuration migration.** `[E]` Migrate `[backend]` to `[runtime]`; alias `mode="local"` -> `backend="sidecar"` and `mode="api"` -> `backend="anthropic_api"`, emitting a deprecation warning for one release. (B-37:L82-L85)
- **E-B37-011 — CLI adapter protocol.** `[E]` Each CLI adapter uses subprocess async I/O, translates Anthropic messages to CLI stdin, parses CLI stdout back to Anthropic responses, and applies `permission_mode` where supported. (B-37:L87-L92)
- **E-B37-012 — CLI adapter failure/timeouts.** `[E]` Enforce `startup_output_timeout_seconds` and `stdout_idle_timeout_seconds`; missing CLI makes `health_check()` return `Unavailable`, never panic; sanitation and Tier C remain mandatory. (B-37:L92-L97)
- **E-B37-013 — Native HTTP adapters.** `[E]` Ollama uses `/api/chat`; vLLM uses `/v1/chat/completions`; both existing 3J providers move to the common factory signature. (B-37:L98-L100)
- **E-B37-014 — Per-role configuration/defaults.** `[E]` `[agents.<role>.runtime] backend="..."`; default AnthropicApi for designer/auditor/researcher/media_validator and Sidecar for executor/memory_writer/context_compiler/prompt_engineer. (B-37:L102-L110)
- **E-B37-015 — Runtime fallback state machine.** `[E]` On `Unavailable`, traverse `fallback_chain`, emit `runtime_fallback` with the traversed chain, then ESCALATE in INTERACTIVE or HALT in AUTONOMOUS when exhausted. This is orthogonal to 4G `SUBSTITUTE_MODEL`. (B-37:L112-L116)
- **E-B37-016 — CLI runtime control.** `[E]` `/runtime show` prints roles/backends; `/runtime set <role> <backend>` overrides; `/runtime fallback <role> <b1>,<b2>` sets the chain. (B-37:L118-L121)
- **E-B37-017 — RuntimePanel behavior.** `[E]` Show per-role backend and chain, green/yellow/red health for known backends, and a canary “Test backend” action. (B-37:L123-L127)
- **E-B37-018 — Metrics schema.** `[E]` `.elai_cc/metrics/phase_4M.toml` records per-backend p50/p99 latency, tokens/sec, cost per million tokens or per hour, fallback fire rate, and health-check uptime percentage. (B-37:L129-L131)
- **E-B37-019 — Migration guide.** `[E]` `workflow/papers/phase-4M-migration.md` must give a worked old `[backend] mode` -> new `[runtime] backend` example. (B-37:L133-L134)
- **E-B37-020 — Exact symbol/event contract.** `[E]` Core types also include `RuntimeFactoryConfig`, `PermissionMode`, `PerRoleRuntime`, `RuntimeCapabilities`, and `RuntimeHealth`; nine new adapter modules are named; SSB actions are `runtime_fallback`, `runtime_health_changed`, `cli_subprocess_spawned`, and `cli_subprocess_killed`. (B-37:L136-L150)
- **E-B37-021 — Consumers/reachability.** `[E]` Reuse existing providers, Tier C, and 4G lifecycle; configuration, Settings RuntimePanel, and CLI `/runtime` are all entry surfaces. (B-37:L151-L159)
- **E-B37-022 — Permission/security invariants.** `[E]` No CLI adapter may bypass Tier C; `PermissionMode` defaults to `AcceptEdits`; `DangerouslySkipPermissions` is explicit opt-in only; stdout must be sanitized before prompt context. (B-37:L160-L167)
- **E-B37-023 — Observable effect.** `[E]` After `/runtime set executor anthropic_api`, runtime selection is role-specific; unavailability emits the traversed fallback row; RuntimePanel reflects override; phase-4M metrics populate. (B-37:L168-L173)
- **E-B37-024 — Structural checks.** `[E]` Verify all 11 enum variants, the trait, resolver/factory, at least 11 adapter test groups plus integration, `SlashCommand::Runtime`, Tier-C references, and the migration guide. (B-37:L175-L199)
- **E-B37-025 — Acceptance/done gates.** `[E]` Acceptance covers ingestion, all adapters, no 3J regression, graceful missing CLIs, Tier-C contract evidence, sanitizer, permission default, idle kill, native migration, role config, orthogonal fallback, CLI/UI, metrics, guide, and tests; every box is unchecked. (B-37:L201-L224)
- **E-B37-026 — Backend implementation surface.** `[E]` Add runtime core plus 11 per-backend modules and `per_role_runtime.rs`; modify CLI/config; add three Tauri commands, RuntimePanel/Settings integration, two paper artifacts, metrics, and decisions. (B-37:L226-L249)
- **E-B37-027 — Source attribution boundary.** `[E]` The architecture is explicitly described as Ouroboros-derived under MIT; the plan cites a separate paper artifact as its ingest output rather than treating the external code as ELAI implementation. (B-37:L21-L27, L54-L61)
- **E-B37-028 — Follow-on.** `[E]` Next is B-38 or the next outstanding B-N depending on priority. (B-37:L251-L254)

#### Ambiguities and conflicts retained

- **A-B37-001.** Whether `RuntimeAdapter` is an alias or thin wrapper remains conditional on vtable compatibility. (B-37:L70-L72)
- **A-B37-002.** The plan directs `std::process::Command` while banning direct spawn that bypasses Tier C; the exact sandbox-wrapped subprocess construction API is unspecified. (B-37:L87-L97, L160-L166)
- **A-B37-003.** `create_runtime` initially stubs nine adapters, but acceptance requires all 11 operational. This describes staged implementation, not a contradiction in final scope. (B-37:L78-L80, L203-L216)
- **C-B37-001.** The canonical source is an unchecked plan and contains no “live Sonnet 4.6” run. The orientation summary's SHIPPED status and cloud-run proof are later additions not evidenced by this file. (B-37:L201-L224)

### B-40 — Idea Framer, Domain Research Presets, and Critic Loop

Source: `docs/thefinalglance/B-plan/B-40_idea_framer_research_presets.md`, 338 lines.

#### Explicit ledger (39 atomic records)

- **E-B40-001 — Goal and blocking contract.** `[E]` Insert FRAMING before recursive decomposition, classify the project, run domain-specific parallel research and a 1-3-round ten-category critic, and block decomposition until framing/grounding is complete and the user answers the surfaced tradeoff card. (B-40:L1-L9)
- **E-B40-002 — Dependencies, schema, and concurrency.** `[E]` Prerequisites B-1/B-22/B-35; source baseline says SSB schema 28 and plans a 28 -> 29 bump for five actions; parallel-safe with B-24/B-25/B-27/B-28/B-37/B-38. (B-40:L10-L16)
- **E-B40-003 — Stage/evidence placement.** `[E]` FRAMING sits before Idea -> Concepts and adds a seventh AutoAdvance check: at least one round and zero unresolved `C-*`; presets can be refreshed through Phase-4H Layer 1A. (B-40:L18-L33)
- **E-B40-004 — Failure prevented.** `[E]` Catch wrong-problem decomposition before its asserted 5×5×3×3/225-babystep fanout; B-41 would otherwise detect framing errors late. (B-40:L39-L61)
- **E-B40-005 — Three subsystem additions.** `[E]` Add TOML domain presets/multi-root FlashResearch, an `idea_framer` Designer variant at GRC 5/800 producing `FramedIdea`/`TradeoffCard`, and a ten-category critic with a maximum of three rounds and final unresolved-block veto. (B-40:L63-L73)
- **E-B40-006 — Project classes/classifier.** `[E]` `ProjectClass` variants are `ConsumerMobileApp`, `WebApp`, `SaaSDashboard`, `DesktopApp`, `GameMod`, `ResearchTool`, `LibraryOrSDK`, `CLITool`, `Other`; deterministic regex runs first and an LLM only for `Other`. (B-40:L89-L97)
- **E-B40-007 — Research-preset schema.** `[E]` `[research_presets.<class>]` is an ordered list of streams with `name`, Tera `query_template` using `{idea}`/`{class}`, `min_source_tier` T1/T2, `parallelism_budget`, `cosmos_grounding`, and `mandatory`. Cosmos injection is a B-42 no-op when unavailable; optional streams can be dropped when budget is tight. (B-40:L99-L107)
- **E-B40-008 — Default preset floor.** `[E]` Seed at least ConsumerMobileApp/WebApp/SaaSDashboard/GameMod; decision docs enumerate each class's mandatory streams. (B-40:L109-L111)
- **E-B40-009 — Multi-root research convergence.** `[E]` `ResearchTree::new_multi_root(streams)` gives each stream a separate cache slot; every mandatory stream must converge or be named in `degraded_streams`. (B-40:L113-L116)
- **E-B40-010 — Framing roles/GRC sequence.** `[E]` Add `idea_framer` and `idea_critic` roles, both extending Designer with 5/800 and `[Observe,Hypothesize,Verify,Converge]`. (B-40:L118-L122)
- **E-B40-011 — `FramedIdea` schema.** `[E]` Fields are `project_class`, `research_summary`, `key_decisions`, `known_unknowns`, and `tradeoff_card`. (B-40:L124-L126)
- **E-B40-012 — `TradeoffCard` schema/user workflow.** `[E]` Ordered `Tradeoff` entries contain `name`, `options[]`, `default`, and `user_facing_question`; the card surfaces in dashboard/CommandWindow before decomposition. (B-40:L127-L129)
- **E-B40-013 — Critic schema.** `[E]` `CriticFinding` holds category, `BLOCK/WARN/INFO` severity, a research-stream `evidence_ref`, and `proposed_fix`; categories are STALE, SLOP, POLICY, FEASIBILITY, MARKET, SCOPE, CONTRADICTION, DESIGN_HARD, VAGUE, MISSING. (B-40:L130-L134)
- **E-B40-014 — Critic-loop state machine.** `[E]` Each round spawns `idea_critic`, groups findings, exits on zero BLOCKs, otherwise feeds `rescope_from_critic` to the framer before the cap; final remaining BLOCK emits `idea_framing_block` and prevents advance. (B-40:L136-L145)
- **E-B40-015 — FSM routing and tool permissions.** `[E]` Nontrivial `/task` flows PLANNING -> FRAMING -> PLANNING -> FEASIBILITY_PROBE; while FRAMING, PreToolUse permits only research/read and forbids Edit/Write/Bash outside research scratch. (B-40:L147-L151)
- **E-B40-016 — Framing evidence derivation.** `[E]` `derive_framing_evidence` queries `idea_framed`, `tradeoff_card_emitted`, and absence of unresolved `idea_framing_block`, adding a seventh AutoAdvance check only when state is FRAMING. (B-40:L153-L159)
- **E-B40-017 — V2 framing window.** `[E]` `IdeaFramingWindow.svelte` is a pre-Workspace first-task screen; it renders the framed idea/card, captures a radio answer for each tradeoff, and submits FRAMING -> PLANNING. (B-40:L161-L165)
- **E-B40-018 — Live framing state/UI badge.** `[E]` `get_idea_framing_state` must move from stub to live Tauri command; Monitor shows `Critic rounds: N/3` only in FRAMING. (B-40:L166-L169)
- **E-B40-019 — Artifact lifecycle.** `[E]` Persist `framed-idea.toml`, `critic-rounds/round-N.toml`, and `tradeoff-responses.toml` under `workflow/sessions/{sid}/idea-framing/`; Designer Concepts consumes them through B-35 auto-injection as ground-by-default artifacts. (B-40:L171-L175)
- **E-B40-020 — Exact produced-symbol/event contract.** `[E]` Produce `FramedIdea`, `TradeoffCard`, ten-variant `CriticCategory`, `run_critic_loop`, `ProjectClass`, `classify_project_class`, `ResearchTree::new_multi_root`, `FsmSubstate::Framing`, two role configs, and actions `idea_framing_started`, `project_class_classified`, `critic_round_complete`, `idea_framed`, `tradeoff_card_emitted`. (B-40:L177-L190)
- **E-B40-021 — Consumers and entry reachability.** `[E]` Consume ResearchTree, AgentPool spawn, BasicGrcEngine, and SourceQualityTable; reachable from CLI turn on nontrivial `/task` and dashboard framing submission. (B-40:L191-L200)
- **E-B40-022 — Stub/budget/research bans.** `[E]` Framing state must be live SSB, not mock; framer budget cannot exceed 5/800; mandatory streams cannot be silently skipped and instead become degraded. (B-40:L201-L205)
- **E-B40-023 — Observable sequence.** `[E]` Example “make an app like PictureThis” classifies ConsumerMobileApp, dispatches six mandatory streams, emits framing started -> classification -> research progress -> 1-3 critic completions -> framed -> card within 5-15 minutes on local 7B. (B-40:L209-L216)
- **E-B40-024 — Observable artifact predicates.** `[E]` Output must have class not `Other`, at least two tradeoffs, and research stream count equal to preset mandatory-stream count. (B-40:L217-L220)
- **E-B40-025 — Schema/action checks.** `[E]` Deterministic check expects `SCHEMA_VERSION: u32 = 29` and all five new action variants. (B-40:L222-L231)
- **E-B40-026 — Type/classifier/category checks.** `[E]` Verify `ProjectClass`, `classify_project_class`, and exactly ten uppercase critic variants. (B-40:L233-L239)
- **E-B40-027 — Preset/test floor.** `[E]` Config must contain at least four presets; run at least 28 framing tests, broken down as 15 unit, eight convergence, and five classifier tests. (B-40:L241-L246)
- **E-B40-028 — FSM/evidence/UI/build checks.** `[E]` Verify Framing wiring, `derive_framing_evidence`, IdeaFramingWindow existence, clean workspace `cargo check`, and clean `svelte-check`. (B-40:L248-L260)
- **E-B40-029 — Classifier acceptance.** `[E]` Achieve at least 90% accuracy on 50 labeled prompts committed at `.elai_cc/tests/fixtures/project_class_50.toml`. (B-40:L262-L265)
- **E-B40-030 — Research/critic acceptance.** `[E]` Ship at least four presets with at least four mandatory streams each; converge within three rounds on 95% of fixture ideas, with remaining 5% correctly emitting `idea_framing_block`. (B-40:L266-L268)
- **E-B40-031 — Artifact/gate/UI acceptance.** `[E]` Populate all framed-artifact fields; keep existing six autonomous checks passing while adding the seventh; use a live V2 window with no mock fallback. (B-40:L269-L275)
- **E-B40-032 — Ablation/build acceptance.** `[E]` On 50 ideas, post-framing Concept audit BLOCK rate should fall at least 30%; at least 28 tests and both build checks must pass. All B-40 acceptance boxes are unchecked. (B-40:L276-L278)
- **E-B40-033 — Test suite.** `[E]` Add 50-fixture classifier/preset/multi-root unit tests, critic convergence/artifact/tradeoff/event contract, final-round block contract, degraded-tier preset test, and at least four additional framing cases on the existing 14-test autonomous gate contract. (B-40:L280-L294)
- **E-B40-034 — Implementation surface/schema mutation.** `[E]` Create framing code/tests/window/fixture/decisions; modify research tree, FSM, SSB schema 28 -> 29, task runner, role pool/config, TOML presets, Tauri/stub wiring, and Monitor. (B-40:L296-L315)
- **E-B40-035 — FSM design constraint.** `[E]` Decisions must justify FRAMING as a PLANNING substate to reuse tool gating and avoid top-level matrix growth. (B-40:L317-L322)
- **E-B40-036 — Classifier latency constraint.** `[E]` Regex-first is intended to keep the common 90% classification path below 10 ms. (B-40:L323-L324)
- **E-B40-037 — Critic taxonomy/cap rationale.** `[E]` Decisions must justify exactly ten categories and three rounds; the source attributes the cap to diminishing returns after round two plus one safety-margin round. (B-40:L325-L330)
- **E-B40-038 — Honest degradation constraint.** `[E]` Missing mandatory research can only be declared degraded, never silently omitted, because downstream concepts would otherwise appear grounded. (B-40:L331-L333)
- **E-B40-039 — Follow-on.** `[E]` B-41 consumes the framed idea and presets for grounded Concepts-level audit. (B-40:L335-L338)

#### Ambiguities and conflicts retained

- **C-B40-001.** Task logic says exit immediately when a round has zero BLOCKs, while `stub_ban` says `run_critic_loop` must not short-circuit and `max_rounds=3` is enforced “even when 0 findings.” These cannot both govern the same zero-finding round. (B-40:L136-L145, L201-L205)
- **C-B40-002.** Source schema is explicitly 28 -> 29 with unchecked work and an expected 29 check. The orientation summary instead reports shipped schema 41, a B-34 pin-42 interaction, and 79/79 tests. Those are later/additional facts, not reconcilable inside this canonical file. (B-40:L14-L15, L224-L246, L262-L278)
- **A-B40-001.** Objective says the user must answer the tradeoff card before decomposition, but `derive_framing_evidence` lists only card emission, framed row, and no block; it does not explicitly query `tradeoff-responses.toml` or an answer event. UI submit advances, but the gate's proof of response is underspecified. (B-40:L1-L9, L155-L169, L171-L175)
- **A-B40-002.** “1-3 rounds” and evidence `rounds_used >= 1` allow early convergence, reinforcing the task logic and making the no-short-circuit ban look stale or misphrased. (B-40:L5-L6, L24-L26, L136-L145, L201-L205)
- **A-B40-003.** The source cites research papers and numerical effects, but no referenced paper was opened under chunk isolation. Treat the 31% and RefineBench rationale as plan provenance, not verified evidence. (B-40:L30-L37, L329-L330)
- **A-B40-004.** The source itself has no shipped marker and all acceptance boxes are unchecked; summary SHIPPED/79-test status is not source-explicit here. (B-40:L262-L278)

## Cross-source interactions

### Explicit interaction ledger

| Producer/owner | Consumer/dependency | Explicit interaction |
|---|---|---|
| B-1/B-2 | B-3 | Correct paths and FSM emission are B-3 prerequisites; B-2 activates four StatusBar items. (`E-B3-001`, `E-B3-022`)
| B-4/B-5/B-6/B-7/B-36 | B-3 | Domain plans create real producers; B-3 classifies/deletes/relabels while B-36 performs live invoke swaps. (`E-B3-005`, `E-B3-008`-`E-B3-016`, `E-B3-045`)
| B-3 | B-9/B-13 | Both older plans assume B-3 supplies ts-rs handoff types, but refreshed B-3 instead owns a manual TypeScript mirror. This is an explicit cross-file stale dependency. (`E-B3-017`, `C-B9-001`, `C-B13-002`)
| B-9 | B-3 | B-9 supplies `/continue`, branch operations, and merge events consumed by B-3 inventory items 18, 20, 25, and 26. (`E-B3-019`, `E-B3-021`, `E-B3-023`, `E-B3-024`; `E-B9-006`, `E-B9-010`, `E-B9-012`)
| B-5 | B-8 | B-5 event/UI wiring is prerequisite and frontend pair for B-8 production activation. (`E-B8-001`, `E-B8-011`)
| B-2 | B-9 | B-9 checkpoint emission attaches to FSM transitions. (`E-B9-009`)
| B-15 | B-9 | `compact.rs` deletion and disk-session msgpack are handed to B-9's SessionDag takeover. (`E-B15-004`, `E-B15-020`; `E-B9-007`, `E-B9-008`)
| B-3/B-4/B-7 | B-15 | Dashboard-owned HashMap residuals require coordinated closure rather than unilateral edits. (`E-B15-011`, `E-B15-035`)
| B-5/B-6/B-10 | B-13 | These producers supply real data to the three orbit signal layers. (`E-B13-009`)
| B-12 | B-13 | Workspace-primary is a prerequisite for the re-scoped Activity/orbit surface. (`E-B13-001`)
| B-1/B-29 | B-30 | B-1 supplies paths; B-29 consumes multilingual extraction for symbol traversal/cache, while extraction may land independently. (`E-B30-001`, `E-B30-023`, `E-B30-041`, `E-B30-042`)
| B-30 | B-33/B-25 | Multilingual graph data enables language-colored viewer nodes and mixed-language Empire interop. (`E-B30-041`, `E-B30-042`)
| B-18/B-22 | B-37 | Tier C is mandatory for CLI runtimes; `/runtime` mirrors B-22 `/mode`/`/model`, while backend fallback stays separate from `SUBSTITUTE_MODEL`. (`E-B37-001`, `E-B37-015`, `E-B37-021`)
| B-22/B-35 | B-40 | B-40 reuses AutoAdvance evidence and persists a B-35 ground-by-default artifact. (`E-B40-002`, `E-B40-003`, `E-B40-016`, `E-B40-019`)
| B-42 | B-40 | `cosmos_grounding` is resolved by B-42 when available and otherwise no-ops. (`E-B40-007`)
| B-40 | B-41 | B-41 receives the framed idea/presets for grounded Concepts audit. (`E-B40-039`)

### Inferred interactions and integration gaps

- **I-001.** `[I]` B-8 makes reasoning decisions non-vacuous; B-5 publishes them; B-3/B-13 render related reasoning/orbit signals. The individual links are explicit, but the single end-to-end chain is composed. (`E-B8-001`-`E-B8-009`, `E-B3-012`, `E-B13-009`)
- **I-002.** `[I]` If B-9's per-GRC checkpoint rule covers B-8 GRC calls, grounded-plan/CSQ/feasibility activity gains recoverable checkpoints. Neither file explicitly says which B-8 calls constitute a “GRC step,” so this must be confirmed. (`E-B8-002`-`E-B8-009`, `E-B9-009`)
- **I-003.** `[I]` Refreshed B-3's no-ts-rs policy implies B-9 and B-13 must consume the hand-authored six-field TypeScript interface; their retained ts-rs checks should be removed or rebaselined. (`E-B3-017`, `E-B3-035`, `C-B9-001`, `C-B13-002`)
- **I-004.** `[I]` B-37 adds per-role runtime configuration before B-40 adds `idea_framer`/`idea_critic`. No default runtime or fallback chain is specified for those two new roles, leaving a configuration-coverage gap. (`E-B37-014`, `E-B40-010`)
- **I-005.** `[I]` B-15's single remaining direct-time leak is in `task_runner.rs`, the same broad hot-path owner used by B-8 and B-40. New wiring there should use `ClockProvider` to avoid re-growing the quarantined debt; the exact affected function cannot be identified without opening code. (`E-B15-014`, `E-B15-015`, `E-B8-018`, `E-B40-016`)

## Requested-dimension completeness index

This index prevents categories that have no dedicated plan heading from being silently lost.

| Dimension | Exhaustive source-level coverage |
|---|---|
| Capabilities/actions/surfaces | B-3's 30 inventory dispositions and commands (`E-B3-006`-`E-B3-030`); B-8 gates (`E-B8-002`-`E-B8-009`); B-9 handoff/branch UI (`E-B9-004`-`E-B9-018`); B-13 orbit (`E-B13-002`-`E-B13-013`); B-30 grammars/resolvers (`E-B30-004`-`E-B30-026`); B-37 adapters/control (`E-B37-002`-`E-B37-020`); B-40 framing (`E-B40-005`-`E-B40-020`).
| Workflows/states | Class A/B/C and offline/dev modes; CSQ/feasibility outcomes; SessionDag authority/handoff; orbit mode precedence; B-15 task lifecycle; registry load/degrade; runtime fallback states; PLANNING -> FRAMING -> PLANNING -> FEASIBILITY_PROBE and critic rounds. (`E-B3-002`-`E-B3-005`, `E-B8-005`-`E-B8-009`, `E-B9-005`-`E-B9-009`, `E-B13-003`-`E-B13-006`, `E-B15-001`, `E-B30-005`, `E-B37-015`, `E-B40-014`-`E-B40-016`)
| Schemas/data/lifecycle | Manual six-field handoff type; SSB schema 97 -> 98; authoritative SessionDag/msgpack/checkpoints; Phase-1A language field/metrics; runtime TOML/metrics; B-40 preset/framed/card/finding/artifact schemas and proposed schema 29. (`E-B3-011`, `E-B3-017`, `E-B9-007`-`E-B9-009`, `E-B30-005`, `E-B30-021`-`E-B30-023`, `E-B37-010`, `E-B37-018`, `E-B40-002`, `E-B40-007`, `E-B40-011`-`E-B40-013`, `E-B40-019`)
| Flags/defaults/config | `ELAI_DASHBOARD_OFFLINE=1`, `ELAI_DASHBOARD_DEV=1`, `autonomous-mode`, `self-modification`, manual feature flags; map-selection rules; registry order; `[runtime]` migration and role defaults; research presets, GRC 5/800, maximum three rounds. (`E-B3-002`, `E-B3-004`, `E-B3-025`-`E-B3-027`, `E-B3-041`, `E-B15-009`, `E-B30-020`, `E-B37-010`, `E-B37-014`, `E-B37-022`, `E-B40-007`-`E-B40-010`, `E-B40-022`)
| Integrations/events | Exact SSB inventories and consumers are in `E-B3-011`, `E-B3-019`, `E-B3-029`; `E-B8-005`, `E-B8-008`; `E-B9-009`, `E-B9-012`, `E-B9-018`; `E-B30-006`, `E-B30-025`; `E-B37-015`, `E-B37-020`; `E-B40-016`, `E-B40-020`, `E-B40-023`.
| Permissions/security/privacy/compliance | Critical CSQ blocks and intervention; Tier-C isolation, IPC stdout sanitization, permission default/explicit dangerous opt-in; B-40 T1/T2 source floor and FRAMING read/research-only tool gate; B-15 avoids unsafe blind deletion and retains live auth until ownership transfer. No processed source states a regulatory-compliance regime or a broader privacy policy. (`E-B8-004`-`E-B8-008`, `E-B15-004`-`E-B15-008`, `E-B37-004`, `E-B37-011`-`E-B37-012`, `E-B37-022`, `E-B40-007`, `E-B40-015`)
| Failures/fallback/retry/recovery | B-8 pass/fail/blocked/replan and unattended-user question; B-9 stub recovery/handoff and legacy export; B-30 grammar isolation, missing-go.mod and unloaded-grammar fallbacks; B-37 unavailable/fallback/escalate/halt plus missing-CLI/timeout behavior; B-40 degraded streams, up-to-three critic rounds, and final block. (`E-B8-005`-`E-B8-009`, `E-B8-019`, `E-B9-004`-`E-B9-008`, `E-B30-005`, `E-B30-016`, `E-B30-020`, `E-B30-023`, `E-B37-012`, `E-B37-015`, `E-B40-009`, `E-B40-014`)
| Determinism/idempotency/concurrency | Deterministic orbit captions/LCG/mode; deterministic map choices/clock; lexicographic resolvers and visited-set cycle prevention; parallel-safe declarations, stream parallelism budgets, multi-root research; runtime fallback orthogonality. **No processed source specifies a formal idempotency guarantee.** (`E-B13-006`-`E-B13-010`, `E-B15-009`, `E-B15-015`, `E-B30-015`, `E-B30-017`, `E-B30-031`, `E-B37-015`, `E-B40-007`, `E-B40-009`)
| Observability/evidence | Every plan's WIRING_CONTRACT/DETERMINISTIC_CHECK records: B-3 `E-B3-029`-`E-B3-038`; B-8 `E-B8-010`-`E-B8-017`; B-9 `E-B9-016`-`E-B9-022`; B-13 `E-B13-024`-`E-B13-026` (retired); B-15 `E-B15-024`-`E-B15-030`; B-30 `E-B30-025`-`E-B30-035`; B-37 `E-B37-020`-`E-B37-025`; B-40 `E-B40-020`-`E-B40-033`.
| Tests/acceptance/done gates | B-3 claims re-scoped completion (`E-B3-037`) with caveats `C-B3-005`; B-15 has mixed checked/deferred/regressed gates (`E-B15-030`); B-8/B-9/B-13/B-30/B-37/B-40 checklists are unchecked or retired as recorded in their acceptance records. Exact test commands/thresholds are retained in the observability records above.
| Constraints/non-goals | No unexplained UI lies; no V1 retarget/no ts-rs; classic feed outside B-13; no blind dead-code deletion; tree-sitter retained/no unapproved crates/no non-test unwrap; no Tier-C bypass or unsafe permission default; framing write tools forbidden and mandatory research never silently skipped. (`E-B3-043`, `E-B13-013`, `E-B15-004`-`E-B15-008`, `E-B30-003`, `E-B30-034`, `E-B37-022`, `E-B40-015`, `E-B40-038`)
| Dependencies/status/deferred | Per-file headers and status ledgers (`E-B3-001`, `E-B8-001`, `E-B9-001`, `E-B13-001`, `E-B15-001`-`E-B15-003`, `E-B30-001`, `E-B37-001`, `E-B40-002`) plus all explicit handoffs/follow-ons (`E-B3-045`, `E-B8-019`-`E-B8-020`, `E-B9-024`, `E-B15-004`-`E-B15-008`, `E-B15-020`, `E-B15-033`-`E-B15-036`, `E-B30-042`, `E-B37-028`, `E-B40-039`).

## Deltas against `02-B-PLAN-SUMMARY.discovery.md`

The following records every material addition, correction, qualification, or source-provenance mismatch discovered relative to the orientation summary. “Summary addition” means a fact asserted by the summary but absent from this chunk's canonical file; it may come from a later B-wide source, but it is not source-explicit here.

### B-3 summary deltas

- **Correction — ts-rs:** summary lines 50 and 700 say B-3 introduced ts-rs and B-9/B-13 reuse it. Refreshed B-3 explicitly says ts-rs was declined, zero dependency hits are required, and the six-field handoff interface is hand-authored. (`E-B3-017`, `E-B3-035`, `C-B3-002`, `C-B9-001`, `C-B13-002`)
- **Correction — polling:** the summary's inventory says RulesDrawer needs live update but omits that the original one-second cadence was rejected in favor of ten seconds and that the rules producer remains deferred. (`E-B3-014`, `C-B3-001`)
- **Correction/qualification — completion:** summary lines 44-52 call re-scoped B-3 DONE. The source does check B-3 acceptance, but also labels degradation and merge/close PARTIAL, keeps EXPLORE pending, and defers the rules producer; DONE is the B-3 ownership boundary, not all downstream functionality. (`E-B3-010`, `E-B3-024`, `E-B3-028`, `C-B3-005`)
- **Addition — refreshed V2 scale/config:** 198 centralized exports, two offline signals, feature-flags exemption, development hiding, and exact V1-to-V2 routes are absent from the summary's approximately-60+ baseline. (`E-B3-002`-`E-B3-005`, `E-B3-041`, `E-B3-043`)
- **Addition — exact shipped/disposition ledger:** the summary compresses item names; the source supplies per-item shipped/partial/deferred behavior, exact command/event/type identifiers, schema 97 -> 98, dual-cfg features, 55/55 typed SSB proof, manual schema discipline, and implementation paths. (`E-B3-006`-`E-B3-040`, `E-B3-042`)
- **Unresolved correction — merge event:** summary/B-9 wording says merge emits `checkpoint`; refreshed B-3 reports `BranchMerged` and compatibility with `BranchMerge`. (`E-B3-019`, `C-B3-003`, `C-B9-002`)

### B-8 summary deltas

- **Addition — complete branch semantics:** summary lines 110-115 omit `FILE_CHECK`/`SOURCE_CHECK`, Write/Edit/Bash scope, WARNING/pass behavior, feasibility Fail fallback, explicit per-babystep conversion, the `FINDING:` auditor flow, and exact phase-boundary reset. (`E-B8-003`-`E-B8-009`)
- **Addition — evidence and unresolved policy:** exact produced APIs, stub bans, intervention artifact, non-vacuity/destructive tests, implementation paths, unattended-user CSQ choice, and replan grounding cadence are source-only details. (`E-B8-010`-`E-B8-020`)
- **Summary addition/status qualification:** summary calls B-8 DONE; canonical B-8 has every acceptance box unchecked and does not claim shipment. (`A-B8-003`)

### B-9 summary deltas

- **Addition — lifecycle/UI detail:** authoritative-store semantics, export-only JSONL, exact blob fields/helpers, branch-tree columns, recent-handoff picker, amber 70%/red 85% gauge, exact `_with_ssb` APIs, and deterministic checks are compressed away. (`E-B9-007`-`E-B9-011`, `E-B9-014`-`E-B9-023`)
- **Correction — generated types:** summary line 128 repeats the ts-rs correction, but refreshed B-3 makes manual TypeScript the actual chunk-local policy. (`C-B9-001`)
- **Conflict — merge observability:** summary line 127 preserves `checkpoint`; B-9's own trigger expects `branch_merged`, and B-3 reports `BranchMerged`. (`C-B9-002`)
- **Summary addition/status qualification:** summary says DONE with dashboard DAG writes deferred; canonical B-9 has unchecked acceptance and does not state that defer. (`A-B9-001`)

### B-13 summary deltas

- **Addition — exact orbit semantics:** summary lines 161-165 omit active-concept centering, backdrop dots, PageRank score bands/DKG edges, explicit-mode precedence, exact fallback mode rules, caption input tuple, full bubble multiplicities, and hover element labels. (`E-B13-003`-`E-B13-012`)
- **Addition — non-goal/stale split:** classic text feed is a new B-N; the entire retained V1 task/contract/checklist/files section is explicitly non-applicable yet still present. (`E-B13-013`-`E-B13-028`, `C-B13-001`)
- **Summary addition/status qualification:** summary calls the re-scoped form DONE, but this source has no V2 acceptance and all retained V1 gates are unchecked. (`A-B13-001`)

### B-15 summary deltas

- **Correction — status snapshot:** summary lines 178-190 reflect an older state (10,570 -> 252 LOC, ten named modules, 178/180 tests). The 2026-07-03 source says original 11,181, prior 555, current **1,632** after H/I/J/K/L growth, six sibling modules, tripwire 600 fired, and gives no 178/180 result. (`E-B15-001`, `E-B15-003`, `E-B15-017`, `C-B15-001`)
- **Correction — dead files:** summary lists `remote.rs` among remaining deletions; source says it is already absent/shipped. `compact.rs`, `oauth.rs`, and `prompt.rs` remain deliberately live with 6/15/6 caller counts; usage is already stripped of pricing but retains four fields/12 importers. (`E-B15-004`-`E-B15-008`)
- **Correction — debt quantities:** summary says replace 84 HashMaps and 40+ clocks; current source says 21 map violations remain and entropy has one direct-time leak after a documented regression/recovery. (`E-B15-009`-`E-B15-015`)
- **Addition — current shipped/deferred matrix:** exact 11/5/1/1 status split, allowlists, CI scripts, tool moves, hooks behavior, metric/CI steps, expected nonzero debt checks, and three named follow-up tracks are absent. (`E-B15-001`-`E-B15-036`)
- **Confirmation with nuance:** summary correctly mentions scripts, tool relocation, Windows hook, metrics, and partial status, but the source now marks these shipped while explicitly failing map/entropy/CLI completion. (`E-B15-016`, `E-B15-018`-`E-B15-023`, `E-B15-030`)

### B-30 summary deltas

- **Addition — extractor atomics:** summary lines 396-403 name languages/resolvers but omit exact node kinds, decorator/call behavior, re-exports/dynamic imports, JS dialect split, optional Go constants, JVM family differences, C++-only nodes, and every Ruby/PHP/Swift/Bash symbol kind. (`E-B30-007`-`E-B30-013`)
- **Addition — resolver/fallback atomics:** first-match JVM semantics, C# byte identity, no-go.mod WARN/external fallback, JS-config preference/cycle set, Svelte raw-text reparse, CSS `@require`, startup order, v4 conditional migration, no-op `<module>` fallback, and real-codebase matrix are source-only. (`E-B30-014`-`E-B30-024`)
- **Addition — exact contracts/gates/open choices:** the full grammar/resolver symbol list, >=10 healthy minimum, five-language artifact gate, named tests, unwrap/no-dependency constraints, implementation paths, alias/name/language-scope decisions, and downstream benefits are absent or compressed. (`E-B30-025`-`E-B30-042`)
- **Qualification — counts/status:** “13+” is only a floor against an 18-entry declared registry, and canonical acceptance is wholly unchecked, consistent with summary PLANNED. (`A-B30-001`, `A-B30-005`)

### B-37 summary deltas

- **Addition — protocol/config details:** summary lines 502-510 omit paper ingestion, alias-vs-wrapper choice, initial stub factory stage, `mode="api"` alias, both timeout names, stdin/stdout translation, exact native endpoints, canary control, full metrics, migration guide, three command identifiers, and test gates. (`E-B37-006`-`E-B37-013`, `E-B37-017`-`E-B37-026`)
- **Summary addition/status qualification:** SHIPPED and the live Sonnet 4.6 proof appear only in the summary; the canonical file has unchecked acceptance and no such run. (`C-B37-001`)
- **Integration gap added:** B-40's new roles lack B-37 runtime defaults. (`I-004`)

### B-40 summary deltas

- **Correction — schema/tests/status:** summary lines 543-558 say SHIPPED, schema 41/pin-42, and 79/79 tests. Canonical B-40 plans schema 28 -> 29, checks for 29, expects >=28 tests, and leaves every acceptance box unchecked. (`E-B40-002`, `E-B40-025`-`E-B40-032`, `C-B40-002`, `A-B40-004`)
- **Addition — blocking/user/permission behavior:** user tradeoff answers, T1/T2 source floors, optional versus mandatory stream behavior, idea role GRC sequence, exact FramedIdea/Tradeoff/Finding fields, `rescope_from_critic`, nontrivial-task FSM route, and read/research-only permissions are source-only details. (`E-B40-001`, `E-B40-007`-`E-B40-015`)
- **Addition — observable/acceptance thresholds:** six-stream PictureThis example, 5-15-minute window, artifact predicates, 90%/50-prompt classifier, 95% critic convergence, at least four streams per preset, 30% ablation improvement, and named test suite are compressed away. (`E-B40-023`-`E-B40-033`)
- **Internal conflict omitted by summary:** early exit on zero BLOCKs conflicts with the no-short-circuit-even-on-zero stub ban. (`C-B40-001`, `A-B40-002`)
- **Summary addition — citation correction:** summary says one bad citation was corrected; this canonical source still states the paper claims and does not record that correction. (`A-B40-003`)

## Zero-omission and zero-duplicate audit

### Complete line-span coverage

Every logical line belongs to exactly one non-overlapping span below. Headings, blank lines, and mandatory-read lists are retained as structural/provenance material even when they do not create a feature atom.

| Source | Non-overlapping processed spans | Ledger coverage |
|---|---|---|
| B-3 (514) | 1-10; 11-78; 79-107; 108-326; 327-359; 360-408; 409-433; 434-503; 504-510; 511-514 | `E-B3-001`; `E-B3-002`-`005`; context/reads; `E-B3-006`-`028`; `029`-`031`; `032`-`036`; `037`-`038`; `039`-`043`; `044`; `045` plus B-3 A/C records |
| B-8 (153) | 1-22; 23-33; 34-81; 82-107; 108-122; 123-132; 133-143; 144-153 | `E-B8-001`; reads; `002`-`009`; `010`-`013`; `014`-`016`; `017`; `018`; `019`-`020` plus B-8 A records |
| B-9 (204) | 1-10; 11-43; 44-58; 59-67; 68-125; 126-155; 156-172; 173-185; 186-201; 202-204 | `E-B9-001`; `002`; `003`; reads; `004`-`015`; `016`-`018`; `019`-`021`; `022`; `023`; `024` plus B-9 A/C records |
| B-13 (201) | 1-10; 11-72; 73-94; 95-139; 140-161; 162-176; 177-188; 189-198; 199-201 | `E-B13-001`; `002`-`013`; context/reads + `028`; `014`-`023`; `024`; `025`; `026`; `027`; `029` plus B-13 A/C records |
| B-15 (469) | 1-31; 32-68; 69-80; 81-127; 128-181; 182-233; 234-252; 253-272; 273-294; 295-335; 336-389; 390-411; 412-441; 442-457; 458-469 | `E-B15-001`-`002`; `003`; reads; `004`-`008`; `009`-`012`; `013`-`016`; `017`; `018`-`020`; `021`-`023`; `024`-`026`; `027`-`029`; `030`; `031`; `032`; `033`-`036` plus B-15 A/C records |
| B-30 (443) | 1-12; 13-47; 48-67; 68-94; 95-155; 156-223; 224-254; 255-303; 304-339; 340-356; 357-377; 378-408; 409-423; 424-436; 437-443 | `E-B30-001`; `002`-`003`; reads; `004`-`006`; `007`-`013`; `014`-`019`; `020`-`024`; `025`-`028`; `029`-`033`; `034`; `035`; `036`; `037`-`040`; `041`; `042` plus B-30 A records |
| B-37 (254) | 1-13; 14-45; 46-53; 54-62; 63-135; 136-174; 175-200; 201-225; 226-250; 251-254 | `E-B37-001`; `002`-`004` + `027`; `005`; reads; `006`-`019`; `020`-`023`; `024`; `025`; `026`; `028` plus B-37 A/C records |
| B-40 (338) | 1-17; 18-38; 39-74; 75-88; 89-117; 118-152; 153-176; 177-221; 222-261; 262-279; 280-295; 296-316; 317-334; 335-338 | `E-B40-001`-`002`; `003` + provenance ambiguity; `004`-`005`; reads; `006`-`009`; `010`-`015`; `016`-`019`; `020`-`024`; `025`-`028`; `029`-`032`; `033`; `034`; `035`-`038`; `039` plus B-40 A/C records |

Blank separator lines are assigned to an adjacent span, so the span sets cover every line number from 1 through each file's manifest total without gaps or overlap. No source text outside the eight manifest originals was treated as an original.

### Duplicate-control and atomic counts

| File | Lines | Explicit atomic records | Ambiguous/conflicting records |
|---|---:|---:|---:|
| B-3 | 514 | 45 | 6 |
| B-8 | 153 | 20 | 3 |
| B-9 | 204 | 24 | 4 |
| B-13 | 201 | 29 | 3 |
| B-15 | 469 | 36 | 4 |
| B-30 | 443 | 42 | 5 |
| B-37 | 254 | 28 | 4 |
| B-40 | 338 | 39 | 6 |
| **Total** | **2,576** | **263** | **35** |

- **Inferred records:** 5 (`I-001`-`I-005`).
- **Grand labeled analytic records:** **303** = 263 explicit + 35 ambiguous/conflicting + 5 inferred.
- **Duplicate audit:** repeated task/contract/checklist/file-list mentions were consolidated into one explicit atom wherever they describe the same behavior. Retired B-13 V1 atoms are retained once and labeled retired; they are not counted as current V2 features. Summary deltas and interaction tables reference ledger IDs and add no duplicate feature atoms.
- **Manifest audit:** **8/8 files**, **2,576/2,576 lines**, **126,263/126,263 bytes**, and **8/8 hashes** verified; no omission and no manifest duplicate.
