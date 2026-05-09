# MVP Wiring Audit — 2026-04-18

Scope: dashboard → `task_runner` → pipeline chain only. CLI `--local` / `--hybrid`
paths are out of scope (they reach further into the provider stack but do NOT
benefit the MVP user flow).

Entry point chain verified:

- `crates/dashboard/src/commands.rs::submit_task` (line 402) writes an
  `SsbPayload::TaskSubmitted` row.
- `crates/dashboard/src/lib.rs::run` (lines 241–360) spawns
  `start_task_runner` at Tauri `setup`, injecting 8 trait objects
  (launcher, resolver, web-research runner, scaffold applier, rule evaluator,
  interrogation store, checkpoint dir, baby-step executor=None).
- `crates/orchestrator/src/task_runner.rs::drive_task` (line 2119) is the
  dispatcher. It branches: scaffold → `drive_web_scaffold_task`; web-research
  → `drive_web_research_task`; else the default decompose-build-navigate
  pipeline. Baby steps execute via `resolved.baby_step_executor`, defaulting
  to `SyntheticBabyStepExecutor` (outcome hardcoded `Passed`, duration 1 ms).

## 1. Summary

Roughly **55–60% of completed A-steps have some wiring into the dashboard →
task_runner chain, but only ~30% are exercised by the default task path (not
scaffold, not research)**. The dashboard surfaces a handful of action_types
(`task_submitted`, `preview_available`, `reasoning_trace`, `phase_complete`,
`test_outcome`, `death_event`, `spiral_detected`, `intervention_request`) —
every other SSB variant renders as a generic "dim" row. Biggest unwired
categories: (1) the **LLM inference path** — `SidecarProvider`,
`build_dispatch_context`, HyDE, DoLa/TurboQuant/steering request plumbing are
CLI-only; task_runner talks to the sidecar ONLY via `SidecarGroundingClient`
for decomposition. (2) The **real baby-step executor** is off by default
(`baby_step_executor: None` in dashboard/lib.rs:357, so
`SyntheticBabyStepExecutor` runs — every step passes in 1 ms). (3)
**Feasibility / Autonomous / Self-modification** (phases 4F/4G/4H) have zero
code. (4) **Knowledge retrieval** (QueryPipeline, HydeStage, ShortCoder,
populate_tier3, IndexingService) is CLI-wired but dashboard-unwired. (5)
**Anti-spiral is off** (`anti_spiral_enabled: false` at lib.rs:358).

## 2. Wired (reached by dashboard → task_runner → pipeline)

- **A-4 Project Intelligence Tier 1** — `knowledge::detect_project_profile`
  — reached via `dashboard::commands::generate_initial_project_profile`
  (commands.rs:63–89, fires at `create_project`).
- **A-5 Project Intelligence Tier 2** — `knowledge::populate_tier2_conventions`
  — same caller (commands.rs:69). Tier 3 (LLM-assisted) is deferred per
  doc-comment.
- **A-32/A-33 PI completions** — rolled into A-4/A-5 call site above (same
  symbols).
- **A-20 MistakeBook SSB schema** — types defined; task_runner emits
  `MistakeEntry` rows at `record_mistake_on_failure`
  (task_runner.rs:~2493) and reads via `emit_mistake_book_constraints`
  (3F.REDO). Reaches only when baby step fails — which the synthetic executor
  never does, so in the default path this code is dead at runtime.
- **A-19 InterrogationTree storage** — `interrogation_store` trait object
  plumbed through TaskRunnerConfig; dashboard opens `rules.db` at lib.rs:318
  and passes a `SqliteInterrogationStore`. task_runner drives
  `traverse_with_answers` in `emit_interrogation_fix_suggestion` on failure
  (same caveat: synthetic executor = no failures).
- **A-12 Deterministic Auditor** — no. wire-a12 is listed in the gap-doc
  session chain but `DeterministicAuditor` is not invoked from
  task_runner. Marked REAL GAP.
- **A-22 RESEARCHING FSM state** — referenced only by `tool_filter.rs` tests;
  no task_runner transition into it.
- **A-43 Oracle Research Pipeline** — `run_research_pipeline` is dead code
  per Appendix B #8 (superseded by `WebResearchOrchestrator`). The latter IS
  wired via `DefaultWebResearchRunner` at dashboard/lib.rs:261.
- **A-44 Project Indexing Wiring** — `IndexingService` is wired in
  `cli/src/main.rs:82` / `:3036` but NOT plumbed through TaskRunnerConfig.
  So the dashboard path does NOT index the project's code before running —
  baby steps have no DKG/BM25 to query.
- **A-45 Source Tier Config** — type exists
  (`DefaultWebResearchRunner.tier_config_toml`), dashboard passes the empty
  default (`DEFAULT_SOURCE_TIERS_TOML = ""`). Shape wired; real TOML not.
- **A-46 Context Telescope Types** — types defined, referenced in
  `context_budget.rs`; no task_runner call site uses
  `GenerationProfile`/`RepairContext` (grep: 1 match, context_budget.rs).
- **A-48 DKG Data Flow Edges** — schema present in `knowledge` crate; only
  built when `IndexingService` runs, which (see A-44) isn't in the dashboard
  path.
- **4D.11 SearxSearchBackend / 4D.12 WebResearchOrchestrator** — wired
  (task_runner.rs:1100–1215 via `DefaultWebResearchRunner`, dashboard plugs
  it at lib.rs:261). Only reached when `is_web_research_task()` fires.
- **4D.14 WebContentIndex / WebChunk / A-RAG for web** — constructed fresh
  per research task inside `build_orchestrator()` (task_runner.rs:1159).
- **4D.15 VRAM-tiered research config** — dashboard hardcodes
  `VramTier::Vram12Gb` at lib.rs:264. No auto-detect.
- **4E.13 Integration checkpoints** — wired at all four levels (BabyStep /
  Step / Phase / Concept); `run_integration_checkpoint` is called inline
  (task_runner.rs:2436, plus `emit_level_transitions` /
  `flush_final_integration_checkpoints`). Runner is `NoOpIntegrationRunner`
  so every criterion trivially succeeds — wiring is the contract,
  observable outcome is always pass.
- **4E.14 Rules Engine → task_runner** — `RuleEvaluator` wired via
  `TaskRunnerConfig.rule_evaluator`; dashboard constructs
  `DefaultRuleEngine` + loads seed rules (lib.rs:281–305). `evaluate_at_pre_gate`
  fires every baby step. `ProjectProfile`-derived file_domains limited to
  `["web"]` if web project else `[]` (plus always-included engineering/security).
- **4E.15 Scaffold trigger** — `DashboardScaffoldApplier` wraps
  `scaffold::apply_template`; `drive_web_scaffold_task` detects
  "make me a <framework>" prefixes. Only reached for scaffold-intent tasks.
- **4E.1–4E.12 PipelineBuilder + DepthFirstNavigator** — used inside
  `drive_task` (task_runner.rs:2203, 2231). `PipelineBuilder::build` runs
  with `RuntimeToolLookup` (accepts-all) + `RuntimePredicates`
  (zero rules) — structural validation only, no real tool registry.
- **3C.REDO Checkpoint / Recovery** — wired via `CheckpointContext` inside
  `ResolvedConfig.checkpoint`; `emit_baby_step_checkpoint` +
  `emit_baby_step_recovery` called per baby step
  (task_runner.rs:2521–2527). Recovery only fires on a failing step — again,
  synthetic executor masks this.
- **3D.REDO BSP parallel phases** — wired via
  `run_parallel_phases_via_bsp` (task_runner.rs:2190, 2000–2117). Only
  fires for phases with `is_parallel = true`. The current decomposer does
  not set that flag, so this is dormant in practice.
- **3F.REDO MistakeBook read/write** — see A-20 above.
- **wire-4A InProcessAgentPool** — `AgentPoolBabyStepExecutor` exists
  (task_runner.rs:690–825). Dashboard sets
  `baby_step_executor: None` (lib.rs:357), so
  `SyntheticBabyStepExecutor` runs instead — the real agent binary is
  never spawned from the dashboard path.
- **wire-antispiral AntiSpiralDetector** — wired behind
  `anti_spiral_enabled` flag; dashboard sets it to `false` (lib.rs:358).
  Feature-off by default.
- **WEB.1 scaffold templates** — loaded via
  `DashboardScaffoldApplier`; `default_templates_dir()` picks
  `ELAI_TEMPLATES_DIR` or the dev fallback (lib.rs:63). Reached via 4E.15.
- **WEB.3 dev server preview** — `WebValidatorLauncher::default_vite()`
  wired at lib.rs:249–250; fires at first web-project baby step, parks
  handle in `DevServerRegistry`, emits `PreviewAvailable` SSB row. This
  is the biggest green light in the flow — the Svelte frontend handles
  `preview_available` with a clickable URL.
- **B-3 writer** — `Ssb` + `SsbViewer` wired via `DashboardState` in
  lib.rs:146–156. All SSB writes from task_runner flow here; the frontend
  polls via Tauri commands every 500 ms.

## 3. Built but unwired (code exists, nothing calls it from MVP path)

- **A-1 Baseline measurement harness** — standalone binary; not part of MVP
  runtime. Status: EVALUATION (DROP). No wiring needed.
- **A-2 T1/T2 hook fix** — T1/T2 live in `plugins` crate; task_runner does
  not invoke `PreToolUse`/`PostToolUse` hooks (no IPC hook dispatcher in
  the dashboard path). Blocker: task_runner's synthetic executor skips the
  hook surface entirely.
- **A-6 VERSION_GATE + deprecated patterns** — seeded in rules.db and
  compiled into the evaluator at lib.rs:281–305, BUT
  `evaluate_at_pre_gate` runs against the natural-language task text
  (task_runner.rs:~2303), not real source files. Grep/AST rules will never
  match. Blocker: no file content at PRE_GATE until real agent dispatch lands.
- **A-7 Constitutional injection + T1 VERSION_GATE hook** — same blocker
  as A-2/A-6.
- **A-8 Freshness steering vector** — `compute_vector_via_sidecar` has ZERO
  callers anywhere (providers/src/layer_config/steering.rs:501). Blocker:
  no steering UI, Python scripts bypass the Rust wrapper. Gap-doc #3 rules
  it DEAD CODE.
- **A-9 ETC entropy monitoring** — `ETCLogitsProcessor` in sidecar; Rust
  side has no reader for `last_search_trigger` / `last_gg_signals` (per
  patch-1.36 gap). Blocker: conversation layer lives in CLI, not
  task_runner.
- **A-10 Context budget adaptation** — `context_budget.rs` has the slot
  adapter, but task_runner never calls `build_dispatch_context` (the only
  call site is CLI main.rs:7632). Blocker: SyntheticGroundingClient
  bypasses the context budget entirely.
- **A-11 PI Tier 3 + Telescope** — LLM-backed; deferred in
  `commands.rs:229` per doc-comment. Blocker: needs sidecar call path from
  task_runner.
- **A-12 Deterministic-first auditor** — `DeterministicAuditor::new` has
  test-only callers (gap-doc Appx A.2). Blocker: no AUDITING FSM state
  handler in task_runner. Listed as `wire-a12` in gap-doc session chain
  but not landed.
- **A-13 RAG freshness scoring** — lives in `knowledge` crate; reached only
  via the CLI's `QueryPipeline`, which task_runner does not invoke.
- **A-14 T2 AST import extraction** — same blocker as A-2/A-6 (no hooks
  on task_runner path).
- **A-15 Nightly dependency audit** — `cli/src/dep_audit.rs` is a CLI
  subcommand, not reachable from dashboard.
- **A-17 GG sidecar response extension** — sidecar-side implemented;
  `SidecarProvider.last_search_trigger` / `last_gg_signals` populated.
  Blocker: task_runner does not use `SidecarProvider` (only
  `SidecarGroundingClient`), so the extended fields never reach the
  reasoning loop.
- **A-18 Repair context profile** — `GenerationProfile` referenced only in
  `context_budget.rs`; no caller selects a profile per dispatch.
- **A-19 InterrogationTree consumer** — WIRED in task_runner
  (`emit_interrogation_fix_suggestion`, section 2 above) but dormant because
  synthetic executor produces no failures. Classify as "wired but
  unreachable in practice".
- **A-21 CC hook enforcement** — T1 regex + T2 AST; relies on hook
  dispatcher (same blocker as A-2).
- **A-22 RESEARCHING FSM state** — tool_filter.rs tests reference it, but
  no FSM transition into it from task_runner.
- **A-23 Micro-reflection types** — no production emitter.
- **A-24 ShortCoder context compression** — `plugins` crate has it;
  `PostToolUse` hook hosts it per design. Blocker: no hooks on task_runner
  path.
- **A-25 GG generation novelty signal** — piggybacks on A-17 blocker.
- **A-26 Tool routing layer** — `tools/src/routing_rules.rs:38` comment
  "Config serde-ready; not yet wired to elai_cc.toml loader". Task_runner
  does not invoke the router.
- **A-27 Bash safety filter** — plugin exists; no hook dispatcher.
- **A-28 Git commit workflow tool** — not invoked from task_runner.
- **A-29 Agent prompt template** — not reachable without real agent
  dispatch (wire-4A blocker).
- **A-30 Output postprocessor** — `output_postprocessor.rs:37, 167` carry
  A-42-T2 stubs (identity functions). Not called from task_runner.
- **A-31 Action classifier** — `tools/src/action_classifier.rs` referenced
  only in CLI; task_runner has no confirmation gate.
- **A-34 RA self-discovery — failure taxonomy extension** —
  `rules/src/self_discovery.rs:36, 334` marked as STUB ("tracking not yet
  implemented"). Gap-doc confirms un-wired.
- **A-35 PI multi-language DKG grammars** — present in `knowledge` crate;
  built only when `IndexingService` runs (CLI-only, see A-44).
- **A-36 PI Tier 3 + telescope completions** — deferred with A-11.
- **A-37 RA VRAM-tier layer configuration** — sidecar-side; Rust provider
  not used from task_runner.
- **A-38 PI incremental profile updates** — `populate_tier2_conventions`
  is called once at project create, not incrementally per baby step.
- **A-39 AS during-generation safety defaults** — sidecar config; not
  pulled from Rust orchestrator config in task_runner path.
- **A-40 MICRO per-slot optimizations** — depends on A-10 (context budget
  not invoked from task_runner).
- **A-41 AS deterministic auditor completions** — depends on A-12.
- **A-42 Token reclamation techniques** — `output_postprocessor.rs:37`
  marked stub ("tree-sitter not wired yet"); orchestrator already depends
  on `elai-knowledge` + `tree-sitter = "0.26"` (per gap-doc Appx B #9), so
  the cycle note is stale. Real wiring = ~60 min of work, nothing uses it.
- **A-47 /research CLI command** — lives in cli/src/main.rs; dashboard has
  no slash-command surface.
- **B-1 Constrained decoding / B-2 Constrained imports** — no matches in
  codebase (grep: `ConstrainedDecoding|constrained_decoding|grammar_guided`
  = 0 files).
- **B-3 Maintenance automation** — freshness decay is post-MVP; not wired.
- **Phase 1I KnowledgeEngine trait** — zero impls (gap-doc Appx B #4
  confirms INTENTIONAL DESIGN marker for 1I; sub-engines wired directly in
  CLI).
- **Phase 3G hierarchical plan management** — `PlanSkeleton` types exist;
  task_runner does not load a plan skeleton (it goes directly from task
  text → `research_decompose`).
- **Phase 3H session handoff** — implemented in CLI; no task_runner hook.
- **Phase 3J providers (SidecarProvider, VllmProvider, DoLa, TurboQuant,
  steering, ActAdd, MCP)** — ALL routed via CLI `ConversationRuntime`;
  task_runner uses `SidecarGroundingClient` only (a narrower reqwest
  client). Per gap-doc Session patch-1.32/1.33, these sit at the
  `Local` backend which task_runner never invokes.
- **4A Agent process isolation (elai_cc_agent)** — binary referenced by
  `AgentPoolBabyStepExecutor`; dashboard never constructs the executor
  (wire-4A unlanded). `InProcessAgentPool::spawn` zero production callers.
- **4B Tool suite (git2, 5-tier validation, DAIRA, edit matching, failure
  taxonomy, incremental compilation)** — all in `tools` crate; task_runner
  has no `ToolDispatcher` field.
- **4C Plugins (hooks + tiered static analysis)** — per gap-doc Appx A.3,
  `mcp` feature default=off; T1/T2/T3 hooks not invoked from task_runner.
- **4D.10 Research LLM path (real prompts)** — `research_decompose` uses
  `SidecarGroundingClient` but only against a synthetic-fallback template;
  the four L0/L1/L2/L3 prompts don't exist as files (gap-doc Session
  4D.10 gaps #5, #6).
- **4I Media validator** — zero matches in code (gap-doc Appx A.4
  confirms).
- **4J Empire integration** — zero matches (conditional, OK to skip).

## 4. Top 10 wiring gaps to close before 4F/4G/4H

1. **wire-4A real baby-step executor** — P0 — ~300 LOC to swap in the
   dashboard plus ~150 LOC in the `elai_cc_agent` binary boot path — unblocks
   EVERYTHING else (synthetic executor masks failures, so mistake book,
   interrogation tree, checkpoint/recovery, rule evaluation on real file
   content, all become observable). One-sentence fix: in
   `dashboard/src/lib.rs:357` replace `baby_step_executor: None` with
   `Some(Arc::new(AgentPoolBabyStepExecutor::new(...)))` and ensure the
   `elai_cc_agent` binary is built and reachable.

2. **4D.10 research LLM prompt templates and real decomposition** — P0 —
   ~400 LOC + 4 prompt .txt files — unblocks: every task that isn't
   scaffold/research falls through to a synthetic decomposition that the
   dashboard renders as "decomposed 0 concepts" or a stubby one-phase
   graph. One-sentence fix: author
   `crates/orchestrator/src/research_prompts/{l0..l3}.txt`, make
   `research_decompose` call the sidecar once per level, replace
   `SyntheticGroundingClient` fallback with a typed error.

3. **patch-1.32 dashboard LLM backend wiring** — P0 — ~200 LOC —
   unblocks all of phase 3J on the dashboard path. One-sentence fix:
   plumb a `SidecarProvider` (or `LocalDispatcher`) into `TaskRunnerConfig`
   so baby-step agents and `research_decompose` share the same provider
   instance and `build_dispatch_context` runs per dispatch.

4. **wire-a12 DeterministicAuditor in AUDITING** — P1 — ~200 LOC —
   unblocks real auditor gating (A-12/A-41) and the auditor-sourced rule
   proposal loop (A-34, self-discovery). One-sentence fix: in
   `task_runner::drive_task`, after each `emit_checkpoint_outcome`, call
   `DeterministicAuditor::run_checks(&profile, &changed_files)` and emit
   Block verdicts as `Finding` rows before the bubble-up pass.

5. **enable anti-spiral by default** — P1 — ~5 LOC — one-sentence fix:
   flip `anti_spiral_enabled: false` → `true` at
   `dashboard/src/lib.rs:358` once wire-4A is in. Emits
   `SsbPayload::SpiralDetected` which the frontend already renders red.

6. **A-44 IndexingService in task_runner** — P1 — ~150 LOC — unblocks A-13
   (RAG freshness), A-24 (ShortCoder), A-35 (multi-lang DKG), A-48 (data
   flow edges), and any rule that needs real code to fire against.
   One-sentence fix: add an `IndexingService` field to `TaskRunnerConfig`,
   run `build_initial_indexes` on first task per project_id, feed
   `DkgQuerierAdapter` into the rule engine instead of `EmptyDkgQuerier`
   (dashboard/src/lib.rs:284).

7. **A-11 Project Intelligence Tier 3 on first task** — P1 — ~150 LOC —
   unblocks framework-aware decomposition. One-sentence fix: when
   task_runner sees a project_id whose `project-profile.toml` lacks a
   `tier3` block, invoke `populate_tier3(&sidecar, &mut profile)` once
   and persist.

8. **Dashboard frontend action-type coverage** — P1 — ~80 LOC in Svelte —
   unblocks observability. `ActivityEntry.svelte` recognizes 6 action
   types and dims the other 25 (gap: `web_research_step`,
   `integration_checkpoint`, `rule_violation`, `mistake_entry`,
   `interrogation_fix_suggestion`, `bubble_up`, `agent_failed`,
   `checkpoint`, `recovery_event`, `finding`). One-sentence fix: extend
   `colour()` + `summary` formatters per action_type.

9. **Rule evaluator gets real source text** — P1 — ~50 LOC (depends on
   wire-4A) — unblocks A-6, A-7, A-14, A-21. Currently
   `evaluate_at_pre_gate` runs against `task_summary` (natural-language
   task text), so `GrepPattern`/`AstPattern` rules never match. One-sentence
   fix: after the agent executor returns, re-run the rule engine with the
   diff's changed-file contents as the `source` field of
   `EvaluationContext` before the auditor pass.

10. **A-42-T2 tree-sitter pretty-print (output_postprocessor)** — P2 — ~60
    min — unblocks deterministic LLM output formatting. The dep-cycle
    excuse at `output_postprocessor.rs:37` is stale (per gap-doc Appx B #9).
    One-sentence fix: replace the identity stub with a
    `Tree::root_node().utf8_text()` walk + indent-by-depth, update the
    identity contract at `context_budget_contract.rs:511`.

## 5. Notes on `mvp-wiring-gaps.md` delta

Already captured by the gap-doc (no delta needed):

- The entire Appendix A.1/A.2/A.3 inventory of stubs, test-only constructors,
  and feature-flag dead code is accurate as of 2026-04-17 (verified by
  spot-check of `SidecarProvider`, `build_dispatch_context`,
  `InProcessAgentPool`, `WebContentIndex` callers).
- Sessions 4D.10/4D.11/4D.12/4E.13/4E.14/4E.15 status (done/partial/open)
  matches code state.
- Appendix B's 10 "unsure" items have been resolved; recommendations in
  this report agree with the gap-doc's verdicts (wire-4A, wire-antispiral,
  wire-a12 as REAL GAPs; DolaDecoder, steering, run_research_pipeline as
  DEAD CODE; KnowledgeEngine, SsbPayload variants, mcp-feature as
  INTENTIONAL DESIGN).

Not currently captured in the gap-doc (or only implicit):

- **Dashboard `baby_step_executor: None` is the single load-bearing toggle**
  that makes the whole MVP flow look synthetic. The gap-doc identifies
  wire-4A as P0 but does not flag that the dashboard explicitly passes
  `None` at lib.rs:357 — flipping the flag (after the agent binary exists)
  is the smallest dashboard-side change with the largest behavioural impact.
- **Dashboard also passes `anti_spiral_enabled: false`** (lib.rs:358) —
  gap-doc mentions wire-antispiral but not the explicit `false` wire.
  Flipping is a one-line patch after wire-4A.
- **Task runner reads `SidecarGroundingClient` but never
  `SidecarProvider`** — gap-doc notes this for the CLI; for the dashboard
  it means the whole providers crate (DoLa, TurboQuant, steering, HyDE,
  MCP, MILP, GPU multiplexing, workload profiling) is cold in the MVP
  flow. Surfacing this as a "dashboard vs CLI divergence" item would
  help the user pick whether to promote CLI wiring or write a second
  wiring pass for the dashboard.
- **Frontend action-type coverage** (item 8 above) — not in the gap-doc.
  Backend can emit 31 SSB action types; the Svelte ActivityEntry only
  colour-codes 6 of them. A user watching the activity feed today sees
  most of the real work as undifferentiated dim text.
- **Decomposer never sets `is_parallel = true`** — 3D.REDO BSP wiring is
  dormant because `ConceptDecomposition` phases come out with the flag
  unset. Either formally drop BSP from MVP or have the research LLM
  populate the flag on phases with ≥2 independent steps.
- **VRAM auto-detect absent** — dashboard hardcodes `Vram12Gb` at lib.rs:264.
  gap-doc `4D.15` is marked done but the auto-detection (SystemProbe-like)
  is not present; phase 4F would have supplied it.
- **IndexingService missing from task_runner** — gap-doc has it under A-44
  as "wired to CLI" but doesn't highlight that the dashboard path skips it.
  Item 6 above.
- **ELAI_TEMPLATES_DIR / ELAI_RULES_DIR env-vars are load-bearing for
  non-dev bundles** — dashboard/lib.rs:34, 56 fall back to
  `CARGO_MANIFEST_DIR` which doesn't exist in a shipped Tauri bundle.
  The rules-dir audit note was added in 4E.14 but the templates-dir fallback
  has the same problem and isn't called out in the gap-doc.
