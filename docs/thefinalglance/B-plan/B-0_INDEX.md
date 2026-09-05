# B-Plan Index — Sequence to Finish ELAI MVP and Final Product
Date: 2026-04-25 (rebaselined 2026-05-07 against dashboard-v2)
Predecessor reading (mandatory before any B-N execution):
  ../summary/DETAILED_SUMMARY_01_overview_and_architecture.md
  ../summary/DETAILED_SUMMARY_02_current_state.md
  ../summary/DETAILED_SUMMARY_03_gaps_and_roadmap.md
  ../audit-implementation/_AGGREGATE.md
  ../audit-dashboard/_AGGREGATE.md
  workflow/sessions/s-d1-d5-2026-05-05/NEXT-SESSION-FINISH-DASHBOARD-V2.md
  workflow/sessions/s-46448/HANDOFF-track-d-v2-rebaseline.md
  .elai_cc/crates/dashboard-v2/DOMAIN-CONTEXT.md (§0 V2 wiring status)
  .elai_cc/crates/dashboard-v2/DESIGN.md (look/feel canonical)

How to use this index: each B-N is a self-contained prompt for one LLM
implementation session. Execute in order unless explicitly noted parallel-safe.
Each B-N has WIRING_CONTRACT, DETERMINISTIC_CHECK, ACCEPTANCE blocks.

CRITICAL: every B-N closes a wiring gap or activates an existing producer.
This is NOT a feature roadmap. Features come AFTER wiring is solid.

================================================================================
V2 DASHBOARD REBASELINE (2026-05-07) — READ BEFORE TOUCHING ANY DASHBOARD B-N
================================================================================

The original B-plan (B-1..B-35) was authored against the V1 dashboard at
`crates/dashboard/`. That crate is now excluded from the workspace (parked at
`crates/.bkp_old/dashboard/`). The active dashboard is V2 at
`crates/dashboard-v2/` (Tauri 2.10 + Svelte 5 runes + Vite 5 + three.js orbit).
Every dashboard-touching B-N below carries a "V2 REBASELINE" header with the
path/component map. Anything not yet rebased treats the V1 references as
historical intent — re-resolve against the V2 tree before coding.

Path map (V1 -> V2):
  crates/dashboard/src/commands.rs        -> crates/dashboard-v2/src-tauri/src/lib.rs
  crates/dashboard/src/lib.rs             -> crates/dashboard-v2/src-tauri/src/lib.rs
  crates/dashboard/src/ssb_viewer.rs      -> (no V2 equivalent yet — V2 commands
                                               read SSB directly via AppState)
  crates/dashboard/src-frontend/src/      -> crates/dashboard-v2/src/lib/
  crates/dashboard/Cargo.toml             -> crates/dashboard-v2/src-tauri/Cargo.toml
  bin name "elai-dashboard"               -> bin name "elai-dashboard-v2"
                                               (cli/src/dashboard.rs:7 still
                                               points at V1 — fix in Track D)

Component map (V1 -> V2):
  ChatWindow.svelte                       -> windows/CommandWindow.svelte
  ActivityFeed.svelte / ActivityEntry     -> windows/ActivityWindow.svelte
                                               (now THE ORBIT VISUALIZER, not a
                                               text feed — see DOMAIN-CONTEXT §2
                                               + DESIGN.md §8 mode resolution)
  Settings.svelte                         -> windows/SettingsWindow.svelte
  PlanWindow.svelte                       -> windows/PlanWindow.svelte
  MonitorWindow.svelte                    -> windows/MonitorWindow.svelte
  ExploreWindow.svelte                    -> windows/ExploreWindow.svelte
  ReviewWindow.svelte                     -> windows/ReviewWindow.svelte
  Shell.svelte                            -> layout/AppShell.svelte
  Workspace.svelte                        -> screens/Workspace.svelte
  StatusBar.svelte                        -> layout/StatusBar.svelte
  Sidebar.svelte                          -> layout/Sidebar.svelte
  drawers (10 of them)                    -> lib/drawers/<Name>Drawer.svelte
  app.css design tokens                   -> src/app.css (or src/lib/styles/*)
  WindowRegistry.ts                       -> (no V2 equivalent yet — routing
                                               via AppShell + Panel +
                                               WindowHost)

V2 architecture notes (tasked B-Ns must respect):
  - V2 currently wires EXACTLY 2 Tauri commands: greet, get_grounding_cache_stats
    (the latter returns hard-coded zeros — Track A.6 stub).
  - All ~60+ frontend "commands" are stubs in src/lib/api/stub.ts backed by
    mock-data.ts; no live IPC anywhere yet.
  - V2's ActivityWindow IS the deterministic-orbit visualizer (Mode 1 PLAN,
    Mode 2 MAP, Mode 3 RESEARCH per DESIGN.md §8). Replaces the V1 text feed.
  - V2 has no engine crates as deps. Adding elai-orchestrator etc. to
    `crates/dashboard-v2/src-tauri/Cargo.toml` requires user confirmation
    (per the V2 README: "Engine crates intentionally omitted until the new
    dashboard's design is finalized").
  - V2 uses Svelte 5 runes ($state/$derived/$effect/$bindable) — DO NOT
    introduce $: reactive labels or createEventDispatcher (Svelte 4 patterns
    forbidden).
  - V2 mounts via Svelte 5 `mount(App, { target })` in src/main.ts (not
    `new App({ target })`) — see commit f6f96b0.

Track D (D.1..D.13 src-tauri command wiring) is the dedicated dashboard wiring
session. The HANDOFF for Session 1 of Track D rebaselining lives at
workflow/sessions/s-46448/HANDOFF-track-d-v2-rebaseline.md. Track D itself
is now indexed below as B-36 — see PART VI.

================================================================================

## SESSION s-59964 — MASTER ORCHESTRATION (2026-05-07) — STATUS UPDATE

This session shipped 14 B-N phases + 2 net-new features (auto-remote-sidecar +
dashboard overlay UI). Single FSM cycle per phase: context-compiler -> executor
-> auditor -> fixer -> memorize. Max 5 agents in parallel.

### Phases shipped (14)

| Phase | LOC net | Files | Verdict | Key deltas |
|-------|---------|-------|---------|------------|
| B-1 | ~35 | 5 | GREEN-WITH-WARN -> fixed (4 WARNs) | SSB_SESSIONS_SUBDIR=".elai_cc/sessions"; session_control.rs + handoff.rs use canonical constants; dashboard-v2/state.rs uses sessions_dir_path(); path_unification.rs +1 test |
| B-2 | ~134 | 4 | GREEN-WITH-WARN -> fixed (W-1,W-4,W-5,W-6) | OnceLock<Arc<Ssb>> on Fsm; set_ssb_writer(&self); fsm.rs emit PhaseTransition SSB; ssb_viewer recent_phase_transitions(); commands.rs bug fixes |
| B-3 | ~144 | 4 | GREEN-WITH-WARN -> fixed | get_grounding_cache_stats -> CmdResult<Value>; D.1 get_sidecar_health implemented (83 LOC backend + types.ts SidecarHealth) |
| B-4 | ~311 | 6 | DONE (gaps noted) | code_vec opt-in (ELAI_CODE_VEC=1); DEEP-tier HyDE MRR anchor fix; CrossEncoderReranker fallback_model_path; CircuitBreakerStateChange SSB variant + ssb_viewer.embedding_breakers() |
| B-5 | ~87 | 4 | DONE (Task 3 partial) | agent_grc_config_for_role() in reasoning::config; ContextCompiler/MediaValidator required_steps fixed; per-agent budget wiring documented as TODO in emit_reasoning |
| B-6 | ~468 | 8 | BLOCK->fixed | SCHEMA_VERSION 25->26; 5 new SSB variants (EvidenceGateEvaluated, SupervisionEvent, BspSuperstep, AgentLifecycle, AntiSpiralHit); SupervisionManager wiring; BSP superstep emit; evidence gate emit; commit 04f68b8 |
| B-7 | ~520 | 11 | PARTIAL (Task 5 deferred) | BubbleUpEventEmitter + IntegrationCheckpointEmitter traits; DagRepairEmitter; DecompositionTreeEmitter; 4 SSB variants (BubbleUpTyped, IntegrationCheckpointSuccess, DagRepairRound, ConceptDecompositionNode); 6 contract/test files updated |
| B-8 | ~42 | 3 | DONE | PipelineGrc::new_from_default(); TaskRunnerConfig::with_pipeline_grc() builder; 6 contract tests in pipeline_grc_wired_contract.rs |
| B-9 | ~339 | 6 | DONE (dag-write deferred) | BranchTree.svelte (170 LOC); fork/merge/close stubs with CLI-delegation; get_session_branches real branch-status logic; StatusBar context-budget gauge; CommandPalette /session entries |
| B-10 | ~161 | 4 | DONE | context_budget_computed_snapshot() in ssb_viewer; get_context_budget replaced hardcoded fractions with real ContextBudgetComputedPayload; compute_overlap call site wired in task_runner |
| B-13 | ~289 | 4 | DONE | captions.ts +8 rows (FEASIBILITY_PROBE, per-state status rows); BubbleUpStrip.svelte (147 LOC new); OrbitShootingStarOverlay.svelte (121 LOC new, LCG deterministic); mounted into OrbitMode1Plan |
| B-14 | ~370 | 8 | DONE | reqwest stream feature added; streaming_http client (no read timeout); active_streams AbortHandle store; start_chat_stream + cancel_chat_stream commands; streamChat() TS helper with 3-level offline fallback; ResponseCardPayload type; CommandWindow streaming render |
| B-26 | ~557 | 9 | DONE | MonitorWindow live polling (get_fsm_state/get_system_status/get_supervision_log); ExploreWindow graceful empty; ReviewWindow retry-note bar; NotificationManager.svelte (singleton, 165 LOC); webhook_url AppState + set/get commands; DESIGN.md animation timings |
| B-36 | ~637 | 4 | PARTIAL (9/13 D-cmds) | D.13 grounding_cache_stats live SSB; D.1 reindex_now via IndexingService; D.4 fetch_settings_get/set (TOML); D.7 one_shot_prompt subprocess; D.8 get_tool_visibility mirrored const; D.11 sidecar_settings GET/POST + restart_sidecar SSH; D.12 sidecar_search; D.2/D.3/D.5 deferred |

### Net-new features (2)

| Feature | LOC | Files | Description |
|---------|-----|-------|-------------|
| Auto-remote-sidecar | ~296 | 3 | remote_sidecar.rs (274 LOC new); elai.bat --remote dispatch; main.rs --remote-sidecar CliAction; SSH boot + PowerShell tunnel + exponential-backoff health poll + dashboard launch |
| Dashboard remote overlay UI | ~562 | 8 | RemoteSidecarBootOverlay.svelte (185 LOC new); remote_sidecar_helpers.rs (140 LOC new); AppState remote_sidecar_progress field; start/get/cancel_remote_sidecar commands; bootRemoteSidecar() TS helper with browser-dev simulation; Workspace.svelte ?remote=1 trigger |

### Final audit

BLOCK verdict (3 BLOCKs) -> all fixed by FINAL-FIXER-REPORT.md.
BLOCK-1: test_payload() non-exhaustive match in ssb.rs (9 arms missing for B-6+B-7 variants) — fixed +63 LOC in #[cfg(test)].
BLOCK-2: adversarial_state_space.rs struct literal missing step_hash+count — fixed with assert!(matches!(.., ..)).
BLOCK-3: anti_spiral_runner_contract.rs pattern missing new fields — fixed with `..` wildcard.
Release builds: cargo build --release -p elai-cli PASS, cargo build --release -p elai-dashboard-v2 PASS.
SCHEMA_VERSION=63 current (ssb.rs:493). Progression: B-6 to 26; B-23 to 28;
B-27 to 27; B-28 E-3/E-4 to 31/32; B-28 WIRING-1 to 33-35; B-30 to 36; B-29
to 37; B-32 to 38; B-31.5 to 39; B-33 to 40; B-40 to 41; B-34 to 42; B-38
to 43; B-42 to 44; B-41 to 45; B-37C to 46; C-1 to 47; C-2 to 48; C-7 to 49;
C-8 to 50; C-5 to 51; C-10 to 52; C-13 to 53; C-22 to 54; C-23 to 55; C-25
to 56-57; C-26 to 58; C-27 to 59; C-28 to 60; C-29 to 61; C-30 to 62; C-32
to 63. See ssb.rs:50-465 inline comments for full per-bump history.
(Historical note: this line previously read "SCHEMA_VERSION=26 confirmed";
updated by C-33 documentation reconciliation 2026-05-19.) All
invoke_handler! entries verified present.
Passing contract tests: path_unification 3/3, pipeline_grc_wired_contract 6/6, layer1a_contract 20/20.
Remaining WARNs (WARN-1/WARN-2 in final audit): uncommitted working-tree changes from other sessions — not B-6 regressions.

### Status of B-N items vs the plan

B-1 DONE in s-59964 — path unification (.elai -> .elai_cc) in ssb.rs, session_control.rs, handoff.rs, dashboard-v2/state.rs
B-2 DONE — FSM PhaseTransition SSB emit; OnceLock<Arc<Ssb>> pattern; latest_fsm_state dual-type fix
B-3 DONE (D.13 fixed, D.1 get_sidecar_health implemented; remaining 28 stubs deferred to B-36)
B-4 DONE (code_vec opt-in via ELAI_CODE_VEC=1; CircuitBreakerStateChange SSB variant; production emitter pending)
B-5 DONE (AgentGrcConfig fix; per-agent budget wiring documented as TODO; feasibility/CSQ overlap deferred)
B-6 DONE (SCHEMA v25->v26; 5 new SSB action_types; SupervisionManager + EvidenceGate + BSP wired; commit 04f68b8)
B-7 PARTIAL (4 emitter traits + adapters + contract updates done; WebResearchStepEmitter per-stage deferred as B-7b; ssb_viewer decomposition_tree() stub pending v26 rows)
B-8 DONE (with_pipeline_grc() builder; PipelineGrc::new_from_default(); ground_plan/csq/feasibility wireable)
B-9 DONE (BranchTree.svelte; fork/merge/close stubs with CLI guidance; dag-write-in-dashboard deferred)
B-10 DONE (hardcoded fractions replaced; compute_overlap call site wired; ContextBudgetComputedPayload live)
B-13 DONE (orbit captions +8 rows; BubbleUpStrip; OrbitShootingStarOverlay deterministic LCG)
B-14 DONE (Response::chunk streaming; reqwest stream feature; AbortHandle store; 3-level offline fallback)
B-24 DONE — Phase 4I media validator end-to-end: Tier1 (in-process PNG/WAV/3D/UI parsers) + Tier2 (Mock/Ollama/SGLang VLM advisory) + RCE visual-feedback loop (feature media-validator-rce) + lazy gate (wire_media_validator_for_spec) + SsbMediaSink (Arc<Ssb>-backed production sink) + task_runner integration (drive_task L3658 post spec_from_decomposition) + dashboard-v2 MediaValidationPanel (4 Tauri cmds, 6 SSB action_types subscribed); 71 lib + 21 RCE + 19 dashboard + 2 e2e tests pass; cloud_mode_live #[ignore]'d (claude-sonnet-4-5-20250929).
B-26 DONE (MonitorWindow/ExploreWindow/ReviewWindow live; NotificationManager; webhook plumbing)
B-31 DONE — Resumable batched indexing + cross-process IndexLock + startup auto-resume; ProjectMetadataStore/lock.rs/startup.rs all production-wired (cli c17_lifecycle + dashboard lib.rs); 5 SSB payloads; IndexResult::{Cancelled,Resumed,AlreadyLocked}; ELAI_DISABLE_INDEXING registered in wire-dark-allowlist.toml (R9); prior PLANNED marker was stale (s-plan-audit-2026-06-04)
B-36 PARTIAL (9 of 13 D-cmds wired; D.2/D.3 deferred to B-36 Session 3; D.5 auditor-owned; obscura deferred)
B-37 SHIPPED — commit fd5c2e5. Multi-Runtime Backend Abstraction (Phase 4M):
  RuntimeAdapter trait + 11 backends (cloud, sidecar-local, sidecar-remote,
  ollama, sglang, vllm, llama-cpp, mlx, mock, harness-pin, harness-record)
  + PerRoleRuntime + RuntimeOrchestrator + 4 SSB action types + /runtime CLI
  + RuntimePanel dashboard window. SCHEMA_VERSION 46 (final B-37C bump).
  Evidence: per_role_runtime.rs:169, runtime_orchestrator.rs, state.rs:488.
  Live cloud Sonnet 4.6 PROVEN end-to-end.
B-38 SHIPPED — Ambiguity-score continuous-metric gate retrofit. 3 new SSB
  action types tied to the ambiguity-score gate beacons; SCHEMA_VERSION
  42 -> 43. Producer wiring per ssb.rs:147 inline note. (No standalone
  MEMORY.md block; status confirmed via ssb.rs bump comment and the
  shipped enum variants.)
B-40 SHIPPED — Idea-Framer + Research Presets + Critic Loop. idea_framing.rs
  1358 LOC; FsmSubstate::Framing; 5 SSB action types; IdeaFramingWindow.svelte;
  51 tests; SCHEMA_VERSION 40 -> 41. Paper citation corrected to
  arXiv:2409.04109 (Si et al. 2024) in shipped code; plan-side cite is doc
  debt only — see workflow/sessions/<s>/B-40 ship note. References:
  arXiv:2510.05145 (FlashResearch), arXiv:2511.22173 (RefineBench).
B-41 SHIPPED — Per-Level Auditor Modes. auditor_mode + per_level_audit_dispatch
  + AgentRole+5 + Evidence+5 + AuditTimelineWindow. SCHEMA_VERSION 45. 80
  GREEN tests. navigator-spawn deferred. Reference: arXiv:2404.03602
  (Kamoi COLM, prompt-scope recall 17-37 pp).
B-42 SHIPPED — Cosmos Artifact Tier. cosmos_worker ~700 LOC, briefing
  compiler, VerificationMethod::CosmosCheck, SCHEMA_VERSION 44, CosmosDrawer.
  spawn_loop dispatcher deferred. References: arXiv:2403.12958 (Dated Data),
  arXiv:2511.14022 (Code-Aware LLM Freshness).

### Open follow-ups (recommended next session)

B-7b: wire WebResearchStepEmitter per-stage events (Formulate/Search/Extract/Reflect/Converge) — ~150 LOC
B-6b: production emit sites for AgentLifecycle "exited" + get_agents live aggregator
B-4b: code_vec full rollout (remove ELAI_CODE_VEC=1 opt-in gate after testing)
B-36 Session 3: D.2 run_planning_task + D.3 run_research (requires elai-cli dep or library refactor)
B-9 dag-write-in-dashboard: Arc<Mutex<SessionDag>> in AppState (~80 LOC, path resolver needed)
WARN-1/WARN-2 cleanup: rule_engine.rs HashMap->FxHashMap + dispatcher.rs HashMap->BTreeMap uncommitted session changes need synchronized contract test commits
Live UI verification with real model (sidecar required; 3-step health check first)

### Cross-reference

Full per-phase reports under workflow/sessions/s-59964/.
Final audit at workflow/sessions/s-59964/FINAL-AUDIT.md.
Final fixer at workflow/sessions/s-59964/FINAL-FIXER-REPORT.md.

### FOLLOWUP ROUND (2026-05-07, same session) — closing the 'what's truly left' gap

| Follow-up | Files | LOC | Verdict | Highlights |
|---|---|---|---|---|
| Server.py /v1/settings | 1 | ~80 | PASS (py_compile) | GET+POST routes lines 2838-2896, OrjsonResponse pattern, _engine attrs |
| SearXNG on remote (port 8889) | 2 + remote venv install | n/a | LIVE — 21 results for "python programming" | server.py SEARXNG_URL env var; CLAUDE.md + REMOTE-SIDECAR.md updated |
| AgentLifecycle "exited" | 1 | ~120 | PASS | bsp/engine.rs covers clean_exit/timeout/crash via JoinSet panic walk |
| WebResearchStage per-stage emitter | 8 | ~224 | PASS — no schema bump | 10 wire sites at FORMULATE/SEARCH/EXTRACT/REFLECT/CONVERGE x started/completed; additive optional fields keep v26 |
| D.2 run_planning_task + D.3 run_research | 4 | ~240 | PASS — no new deps | TaskSubmitted broadcast + DefaultWebResearchRunner direct, TauriWebResearchStepEmitter |
| D.4 obscura_test_fetch + D.5 run_d5_live_contract | 4 | ~? | PASS — no new deps | Subprocess pattern, B-14 active_streams reuse |
| Post-smoke fixer | 4 | 17 | PASS | get_sidecar_health dual-shape, 2 warnings cleaned, tunnel timeout 5->10s |
| Final fixer (2 BLOCKs) | 2 | 3 | PASS | bsp/engine.rs 4-tuple destructure + ssb_viewer.rs ClockProvider import |

Audit + smoke summary:
- B-2/B-3 audit: GREEN-WITH-WARN 0.85 -> fixer closed all 4 critical WARNs.
- Final cross-workspace audit: 3 BLOCKs found (test-side missing match arms after schema bump) -> fixed.
- Followup-round final audit: 2 BLOCKs (4-tuple destructure + missing ClockProvider import) -> fixed in the final fixer.
- Live smoke vs friend's RTX 3090: 6/7 sidecar endpoints PASS; /v1/settings now ADDED but needs rsync+restart.

Open WARNs after follow-up final audit (cosmetic / scoped follow-ups):
- WARN-1: cancel_d5_live_contract orphan cargo test subprocess (need child.kill())
- WARN-2: cancel_research doesn't cancel in-flight reqwest fetches (matches B-14 pattern)
- WARN-3: run_d5_live_contract test_name arg unsanitized (no shell-injection risk; validate to [a-zA-Z0-9_-]+)
- WARN-4: workflow/reasoning/ decision docs missing for follow-up phases (paperwork; non-blocking)

Updated status of PARTIAL items from first s-59964 update:
- B-7 PARTIAL -> DONE (B-7b wired all 4 emitter adapters into production constructors)
- B-36 PARTIAL -> DONE (D.2/D.3/D.4/D.5 all implemented; only D.4 obscura needs web_search dep wired — currently subprocess fallback)

User actions to make new wiring live:
  rsync -e "ssh -i ~/.ssh/elai_remote -p 6465" \
    F:/.ELAI_workflow/.elai_cc/python/elai_inference/server.py \
    maintainer@192.0.2.10:~/elai_inference/
  ssh -i ~/.ssh/elai_remote -p 6465 maintainer@192.0.2.10 '~/start-searxng.sh'
  ssh -i ~/.ssh/elai_remote -p 6465 maintainer@192.0.2.10 '~/restart-sidecar.sh'
  # Then test: elai.bat --remote

Cross-reference: full per-phase reports under workflow/sessions/s-59964/ — including
FOLLOWUP-FINAL-AUDIT.md, FOLLOWUP-FINAL-FIXER-REPORT.md, searxng-deploy-report.md,
and the per-item execution reports.


================================================================================
## SESSION s-26716 — B-22 Phase 4G AUTONOMOUS MODE (2026-05-09)
================================================================================

Single autonomous-mode session. Constraint: no commits, single audit cycle.
Plan: B-22_phase_4G_autonomous_mode.md. Spec: docs/specs/guide/steps/phase-4G.txt.

### Phase shipped (1 main + 1 follow-up patch)

| Phase     | LOC net | Files | Verdict             | Key deltas |
|-----------|---------|-------|---------------------|------------|
| B-22      | ~480    | 12    | DONE (1 BLOCK fixed) | 18 extended contracts (phase_4g_extended_contract.rs); SlashCommand::Mode + parser tests; derive_autonomous_evidence helper replaces hardcoded Evidence at task_runner.rs L3996 (queries DkgReady/Finding/TestOutcome/PhaseTransition with vacuous-default cold-start); SsbViewer::auto_advance_stats DTO + method; dashboard-v2 get_auto_advance_stats wired through viewer; MonitorWindow Section 6 AutoAdvanceStats panel (always visible); AgentPool::get_role trait method + InProcessAgentPool impl + 2 unit tests; metrics/phase_4G.toml; phase-4G-decisions.md |
| patch-4G.1 | ~90    | 5     | DONE (0 BLOCKs)      | CommandPalette toggle now fires stub.set_gate_mode IPC (was UI-local only); StatusBar gate badge promoted from .verbose hover-only to always-visible cluster (.gate-label--autonomous teal tint); MonitorWindow auto-advance panel header carries live "gate <mode>" mini-badge; session.reconcileGateMode() pulls live backend state on Workspace.svelte mount |

### Test results

- crates/contracts/tests/phase_4g_extended_contract.rs: 18/18 pass.
- crates/contracts/tests/autonomous_gate_contract.rs: 14/14 pass (unchanged).
- crates/contracts/tests/mode_switch_contract.rs: 7/7 pass (3 extra pre-existing).
- crates/orchestrator agent_pool unit tests: 25/25 pass (2 new get_role tests).
- crates/cli commands unit tests: 180/180 pass (3 new /mode parser tests, slash_command_specs assertion bumped 145->146).
- cargo check --workspace: clean.
- npx svelte-check (dashboard-v2): 0 errors / 9 pre-existing warnings (unrelated drawers).

### Audit (single cycle each, per autonomous-mode constraint)

B-22 audit: 1 BLOCK (stale slash_command_specs().len() assertion 145, fixed in-cycle to 146 at commands.rs:4278); 3 WARNs (unbounded query_by_type in evidence helper — documented; pre-existing 7 mode_switch contracts vs 4 anticipated; 2 pre-existing cli_tests.rs failures unrelated to 4G).
patch-4G.1 audit: 0 BLOCKs; 2 doc-level WARNs (decision-doc append + cold-start gate-mode default INTERACTIVE without localStorage — documented as intentional).

### Status of B-N items vs the plan

B-22 DONE in s-26716 — Phase 4G autonomous mode shipped end-to-end with dashboard-v2 wiring; CLI /mode parser is informational-only by design (CLI is binary-only with no [lib], parser test lives in crates/cli/src/commands.rs); orbit per-decision visualization deferred (OrbitFixture not yet fed by GateDecision rows).

### Open follow-ups (carried in workflow/sessions/s-26716/session-handoff.md)

- /mode CLI live wiring: today the parser lands SlashCommand::Mode but live_cli.rs handler is informational only, pointing the user at the dashboard toggle or ELAI_GATE_MODE env var. A real wiring would require the CLI to either spawn a Tauri IPC client or write to a shared control file the runner reads.
- Orbit per-decision visualization: OrbitCanvas derives mode from FSM state only (RESEARCHING / IDLE+pagerank / plan); no GateDecision rows feed orbit nodes today. Would require OrbitFixture extension.
- derive_autonomous_evidence in task_runner.rs uses unbounded query_by_type — at multi-thousand-row sessions this becomes a full table scan per baby step. Add a LIMIT 1000 clause in a follow-up patch.
- last_24h_rate filtering uses dashboard ClockProvider but does not yet honour SeededClock under deterministic replay.
- AutoAdvanceStats panel polling is the existing 5s interval; per-stat realtime push via SSB broadcast deferred.
- production CLI construction paths still pass autonomous_gate: None (cli/main.rs); 4G.7-4G.8 broader tree auto-advance integration unwired in production.

### Cross-reference

- Decision doc: workflow/reasoning/phase-4G-decisions.md (B-22 + patch-4G.1 sections).
- Audit findings: workflow/sessions/s-26716/audit-findings.md, audit-findings-patch-4G.1.md.
- Handoff: workflow/sessions/s-26716/session-handoff.md.
- Metrics: .elai_cc/metrics/phase_4G.toml (7 spec categories; tests=36, deliverables=7).


================================================================================
## SESSION s-30584 — B-39 CHAT REFLECT PIPELINE (2026-05-10)
================================================================================

Single-phase session. Plan: B-39 chat REFLECT pipeline (TRIAGE+REASONER+AUDITOR+SYNTHESIZE).
Spec: docs/specs/guide/steps/phase-4K3-4K4-4K5.txt (reasoning pipeline sub-steps).
Papers ingested: paper-agentless, paper-compute-optimal-tts, paper-refinebench (local RAG).

### Phase shipped (1)

| Phase | LOC net | Files | Verdict | Key deltas |
|-------|---------|-------|---------|------------|
| B-39  | +1944/-14 | 20 | PASS (0 BLOCKs) | 4-stage per-message REFLECT pipeline wrapping LiveCli::run_turn at live_cli.rs:1685; opt-in via /reason on|off (slash count 146 -> 147); N=4 best-of-N candidates (deliberate latency tradeoff vs Liu arXiv:2502.06703 N=16 elbow, documented); REFINE_CAP=3 from RefineBench arXiv:2511.22173 guided-mode evidence; ChatInferenceClient + ReflectEventSink traits in reasoning/ (avoids circular Cargo deps); SidecarChatClient adapter in cli/live_cli.rs:4032 bridges SidecarProvider; 4 new SsbActionType variants (ReflectTriaged/Candidate/AuditVerdict/Refined); SCHEMA_VERSION 28 -> 29; ssb_viewer.get_reflect_state + Tauri command + ReasoningDrawer 7th panel (Svelte 5 runes only); pipeline never panics — falls back to passthrough on any error; 2 new files (chat_reflect.rs 530 LOC, chat_reflect_contract.rs 491 LOC), 18 modified |

### Test results

- crates/contracts/tests/chat_reflect_contract.rs: 12/12 pass.
- crates/orchestrator/src/reasoning/ unit tests: 16/16 pass.
- crates/cli commands unit tests: 39/39 pass.
- cargo check --workspace: clean.
- npx svelte-check (dashboard-v2): 0 errors.
- pre-commit tiers 1+2+3: all PASS.

### Status of B-N items vs the plan

B-39 DONE in s-30584 — chat REFLECT pipeline (observability-only v0); pipeline emits SSB events but runtime.run_turn still produces the user-visible reply; substitution deferred to B-39C; 3 papers ingested during phase.

### Open follow-ups (carried in workflow/sessions/s-30584/HANDOFF-B-39C.md)

- B-39C: REFLECT answer substitution — un-underscored pipeline output capture; bypass runtime.run_turn when not skipped (text-only D2a). Handoff: workflow/sessions/s-30584/HANDOFF-B-39C.md.
- B-39B: AISLE/Mythos jagged-frontier eval suite (PLANNED — needs 3090 box, currently unavailable).
- B-39D: REFLECT progress UI — stream stage transitions to dashboard ReasoningDrawer.

### Cross-reference

- Commit: 6dd4af5 (pushed origin/main 2026-05-10).
- Session: workflow/sessions/s-30584/.


================================================================================
## SESSION s-34648 — B-39C REFLECT ANSWER SUBSTITUTION (2026-05-11)
================================================================================

Single-phase session continuing B-39 (chat REFLECT pipeline, observability-only
v0 shipped 2026-05-10 in commit 6dd4af5). Predecessor handoff:
workflow/sessions/s-30584/HANDOFF-B-39C.md.

### Phase shipped (1)

| Phase  | LOC net | Files | Verdict             | Key deltas |
|--------|---------|-------|---------------------|------------|
| B-39C  | ~274    | 4     | PASS (0 BLOCKs, 3 WARNs) | Un-underscored ChatReflectPipeline.run output capture (live_cli.rs:1707); when !output.skipped, AUDITOR-selected reply renders via TerminalRenderer::render_markdown and runtime.run_turn fall-through is bypassed entirely (D2a — text-only, no tool execution); when skipped=true, falls through to prepare_turn_runtime unchanged. Three additive thin delegates on ConversationRuntime (push_user_text, push_assistant_text, record_pipeline_usage) for non-run_turn history maintenance. /reason help text + commands.rs doc-comment flag the text-only semantics. 4 new contract tests (chat_reflect_contract.rs tests 13-16). gotchas.md: 2 new entries (text-only-by-design + auto-compaction-bypass). Commit 6adb454. |

### Test results

- crates/contracts/tests/chat_reflect_contract.rs: 16/16 pass (12 existing + 4 new — substitute_final_reply_when_not_skipped, runtime_history_includes_pipeline_reply, skipped_falls_through_to_runtime, no_tool_use_blocks_in_pipeline_reply).
- cargo check -p elai-cli -p elai-orchestrator -p elai_cc_contracts: clean.
- cargo test -p elai-cli --bins -- --test-threads=1: 179/180 pass. Single failure (managed_sessions_default_to_jsonl_and_resolve_legacy_json) is pre-existing on main (cli_tests.rs unmodified by B-39C; last touched commit 93d75ce, before B-39).
- pre-commit hook tiers 1-3: PASS (determinism / quality / unit 56 passed).

### Audit (single cycle, WARN-only)

- BLOCK count: 0.
- WARN-1 (auto-compaction never fires on REFLECT turns): architectural-by-design — substitute path bypasses run_turn so maybe_auto_compact never invokes; record_pipeline_usage sets input_tokens=0 by design (pipeline tracks aggregate output tokens only). Long /reason on sessions grow session.messages unbounded. Documented in gotchas.md, deferred to B-39D/E.
- WARN-2 (test 16 tautology): addressed in-cycle. Refactored to drive push_assistant_text and inspect runtime.session().messages[0].blocks instead of constructing the vec inline.
- WARN-3 (gratuitous .clone() on backend_config at live_cli.rs:1691): pre-existing in B-39 inheritance, not introduced by B-39C. Left per CLAUDE.md "no unsolicited refactoring" rule.

### Status of B-N items vs the plan

B-39C DONE in s-34648 — REFLECT pipeline is now answer-substituting (was observability-only after B-39). /reason on now produces a deeper text reply via TRIAGE→REASONER N=4→AUDITOR→SYNTHESIZE; users opt out of tool execution by enabling /reason on; tool-using fast chat remains the default with /reason off. Conversation history is coherent across substituted turns via the new ConversationRuntime delegates.

### Open follow-ups (separate B-Ns when scoped)

- B-39D: REFLECT progress UI — stream stage transitions to dashboard ReasoningDrawer (TRIAGE running... REASONER 2/4... AUDITOR... SYNTHESIZE... DONE) so user sees why /reason on takes 12-30s. Touches dashboard-v2 + a new SSB action_type or extension of the existing 4. Decoupled from B-39C.
- B-39E: D2f system-hint injection — opt-in mode that runs the standard run_turn tool loop with ReflectOutput.reasoning_summary injected as a system hint, giving "REFLECT-grounded tool execution". Restores tool calls under /reason on at the cost of one extra model call per turn.
- B-39F (or fold into B-39E): session-length compaction guard for /reason on — either (a) call maybe_auto_compact directly from the substitute path when message count exceeds a threshold, or (b) estimate input tokens from effective_input.len() in record_pipeline_usage as a coarse proxy. Until shipped, /reason on users on long sessions must manually /compact.
- B-39B (deferred earlier, still open): AISLE/Mythos eval suite — eval REFLECT against github.com/stanislavfort/mythos-jagged-frontier reproduction repo. Requires the 3090 box.
- Pre-existing test failure: cli_tests::managed_sessions_default_to_jsonl_and_resolve_legacy_json is broken on main (legacy `.elai/sessions/` path resolution drifted after the `.elai_cc/` migration). Not B-39C's bug; needs its own patch-N.X.

### Cross-reference

- Decision doc: workflow/reasoning/B-39C-decisions.md.
- Context package: workflow/sessions/s-34648/context-package.md.
- Transformed prompt: workflow/sessions/s-34648/transformed-prompt.md.
- Impact report: workflow/sessions/s-34648/impact-report.md (YELLOW: additive blast, 3 advisory notes).
- Audit report: workflow/sessions/s-34648/audit-report.md.
- Predecessor handoff: workflow/sessions/s-30584/HANDOFF-B-39C.md.
- Gotchas added: workflow/memory-bank/gotchas.md — B39C_REASON_ON_BYPASSES_TOOL_EXECUTION_BY_DESIGN, B39C_AUTO_COMPACTION_NEVER_FIRES_ON_REFLECT_TURNS.
- Commit: 6adb454 (pushed origin/main 2026-05-11).


================================================================================
## SESSION (dashboard-only ad-hoc) — ORACLE MODE AUTO-DETECT CHIP GATE (2026-05-11)
================================================================================

Single-session UX patch. No B-N — pure dashboard-v2 quality-of-life feature on
top of the existing /ask + /research slash routing. CLI untouched. FSM untouched.
No new deps. The user's question that motivated this: "using / something always
is bad… can we detect [research intent] from user messages?" Conclusion: yes,
but never silently — gate every auto-fire behind a one-key confirm chip.

### Patch shipped (1)

| Patch | LOC net | Files | Verdict | Key deltas |
|-------|---------|-------|---------|------------|
| oracle-gate.1 | ~210 | 6 | PASS (0 errors, 0 new warnings) | (1) intent.ts: Intent union extended with EXPLORE_CODE + RESEARCH; PATTERNS reordered so code-context anchors fire BEFORE knowledge-context anchors ("what calls foo()?" -> EXPLORE_CODE, "what's the latest pytorch version?" -> RESEARCH); SLASH_MAP remapped /explore→EXPLORE_CODE, /ask→RESEARCH, /research→RESEARCH, /status retained as EXPLORE. (2) api/types.ts: IntentType union synced with the two new variants. (3) intent.test-cases.ts (new, ~80 LOC): 23 classifier cases + 5 gate-firing counterfactuals (no test runner wired — static const for human eyeball-verification + future Vitest pickup). (4) commands.svelte.ts: new $state pendingResearchChip + refocusInputTick; submit() refactored to extract _runStreamingPath helper and insert gate AFTER slash short-circuit and BEFORE user-card append (gate fires only when slash===null && intent==='RESEARCH' && !isStreaming && pendingResearchChip===null — user card is deferred so Edit rolls back cleanly); three lifecycle methods acceptResearchChip/rejectResearchChip/editResearchChip exposed on the store. Accept routes through /research (the wired backend, not /ask which is SLASH_NO_BACKEND) — documented inline. (5) CommandWindow.svelte: chip rendered as orbit-card--intervention variant inside cards stream; focusable container with tabindex=-1 + scoped onkeydown (Enter→accept, Esc→reject, Tab→edit); two $effects (autofocus chip on appear, refocus textarea after Edit); textarea + slash-toggle bound to disabled={!!commands.pendingResearchChip} to kill the typing-during-chip race; minimal CSS reusing existing classes. (6) ExploreWindow.svelte: 3-line comment documenting that the chip is intentionally NOT applied there (ExploreWindow IS the Oracle destination — would be redundant). |

### Test results

- npm run check (svelte-check + tsc --noEmit) in dashboard-v2: 0 ERRORS, 9 WARNINGS (all pre-existing unused-CSS-selector warnings in unrelated drawer files; none in edited files). EXIT=0.
- 23 classifier test cases mentally verified against new patterns — all pass.
- No Rust changes — no cargo runs needed.

### Audit (self-verified, no auditor agent spawned for UX patch)

- BLOCK count: 0.
- Counterfactual checks (chip firing decision) — all expected:
  - `/research foo` → chip? NO (slash bypass; parseSlashCommand returns non-null).
  - `/ask foo` → chip? NO (slash bypass).
  - `?` → chip? NO (classifier returns AMBIGUOUS at intent.ts:93, length≤8 fallback).
  - `isStreaming === true` → chip? NO (gate condition includes !isStreaming).
  - `what calls foo()?` → chip? NO (matches EXPLORE_CODE pattern at intent.ts:42).
  - `what's the latest pytorch version?` → chip? YES (matches RESEARCH pattern at intent.ts:60).

### Design choices (documented inline)

- Conservative RESEARCH patterns: false positives are worse than false negatives. A miss falls through to TASK → local stream (no web hit). A false positive shows the chip on every code question (annoying). Acronym/CamelCase/hyphenated-term gating on "what is X" filters chatty fuzzy questions out.
- EXPLORE_CODE matches BEFORE RESEARCH — guarantees code-grep questions never trip the Oracle gate.
- /status retained as plain EXPLORE (meta info, neither code-grep nor web-research).
- User card deferred until chip resolution → Edit rollback is trivial (no card to remove from stream).
- Window-level keydown listener rejected in favor of chip-scoped tabindex=-1 + onkeydown — automatic cleanup via Svelte 5 lifecycle, no leaked listeners across rapid submits.
- Slash palette + chip mutually exclusive by construction: chip can only appear on submit (which clears slashOpen=false), and textarea+toggle disabled while chip pending prevents new '/' from opening the palette.
- Accept dispatches /research not /ask. /ask is in SLASH_NO_BACKEND (returns error card); /research is wired to run_research(). CLI-side these are aliases (commands.rs:1051-1054), but dashboard-side only /research has a backend.

### Status

DONE — gate is live in source. Requires rebuild for elai.bat (modes 1/2/5) to pick it up: `npm run build` in dashboard-v2 → `cargo build --release` from .elai_cc, or `npm run tauri:build`. For quick visual verification: `npm run dev` in dashboard-v2 (stub backend; gate UX fully testable). For full integration: `npm run tauri:dev` (real /research backend).

### Open follow-ups (none required; nice-to-haves)

- A11y polish: announce chip appearance via aria-live region for screen readers (current chip uses role="dialog" + aria-label but no announcement on creation).
- Telemetry: log gate-decision rate (accept/reject/edit ratios) to help tune RESEARCH patterns over time. Would need a new SSB action_type or a logStub variant.
- Optional sidecar-LLM intent classifier behind a config flag (~50 tokens, ~200ms) to catch RESEARCH questions the regex misses ("what is pytorch?" lowercase form). Disabled by default — regex is the deterministic baseline.
- Pattern tuning: monitor real-world classifier output; adjust regex if users report false-positive chips on code questions.

### Cross-reference

- Conversation thread: ad-hoc, single session, no plan file. User question motivated the design.
- Files touched (under .elai_cc/crates/dashboard-v2/): src/lib/utils/intent.ts, src/lib/utils/intent.test-cases.ts (new), src/lib/api/types.ts, src/lib/state/commands.svelte.ts, src/lib/windows/CommandWindow.svelte, src/lib/windows/ExploreWindow.svelte.
- No commit yet — work staged for user review.


================================================================================
PART I — MVP COMPLETION (B-1 through B-10)
================================================================================
Objective: make what's already built actually run end-to-end, visibly.
Estimated total: 10 implementation sessions. Independent paths marked.

  B-1   Path Unification (.elai → .elai_cc everywhere)
        Highest-leverage fix. Activates DKG/PageRank/CatRAG/A-RAG/STUBS
        runtime paths in one stroke. Unblocks B-4, B-7.
        Independent — can run before any other B-N.

  B-2   FSM PhaseTransition Wiring (the 30-min fix)
        ~10-line patch. Activates StatusBar badge, EXECUTING pulse,
        MonitorWindow.currently, ActivityEntry phase_transition.
        Independent — can run in parallel with B-1.

  B-3   Stub Command Eradication
        Delete or implement each of 30 dead UI elements. Reduces user
        confusion. Sets the inventory for B-4..B-7.
        Sequential after B-1 + B-2.

  B-4   Producer Wiring — Knowledge engine
        QueryTrace inspector, SearchMode cascade live, breaker grid,
        freshness panel, R@5/R@10/MRR badges, code_vec query, model staging,
        DEEP-tier MRR fix.
        Sequential after B-1 (needs path fix); parallel-safe with B-5/B-6.

  B-5   Producer Wiring — Reasoning
        Per-agent GRC budgets (real, not synthetic), GroundingSweep result
        panel, CSQ guard prompts, feasibility outcomes, role config viewer.
        Independent of B-4/B-6 — parallel-safe.

  B-6   Producer Wiring — Orchestration
        7 EvidenceGate verdict panel, Agents panel, SupervisionManager log,
        BSP visualization, ContextBudget per-slot bar chart.
        Independent — parallel-safe with B-4/B-5.

  B-7   Producer Wiring — Pipeline Factory
        DecompositionTree visualizer, DAG repair log, research tree per-node
        events, IntegrationCheckpoint successes.
        Sequential after B-1 (needs path fix).

  B-8   PipelineGrc Activation
        Call verify_claim/ground_plan/generate_csq/check_inline_feasibility
        from production hot paths. Activates safety gates.
        Sequential after B-5 (depends on dashboard surfaces being ready
        to render the activations).

  B-9   SessionDag Activation
        Replace JSONL with SessionDag; populate stub_ssb_seq; expose
        branch ops in UI; SPEC AMENDMENT #2 SSB action_type='checkpoint'.
        Independent — parallel-safe.

  B-10  ContextBudget Profile Activation
        Pass real ModelCapabilityProfile from task_runner; implement
        file_overlap_score (3E.5).
        Independent — parallel-safe.

GATE FOR PART I: after B-1 through B-10, the user should see:
  - Live FSM badge transitions
  - Real engine telemetry per query
  - GRC budgets per agent (real values)
  - DAG visualization for executing tasks
  - Branch UI in session panel
  - No DEAD UI elements (all stubs deleted or implemented)


================================================================================
PART II — DASHBOARD VISUAL RESTORATION (B-11 through B-14) [V2 REBASELINED]
================================================================================
Objective: rebaselined 2026-05-07 — V2 already ships the spec visual identity
(JetBrains Mono, Workspace-primary, orbit ActivityWindow). The original
"restoration" framing is now mostly carry-forward audits + the chat streaming
that V2 still stubs.

  B-11  Design System Restoration  [LARGELY DONE in V2]
        Verify V2 tokens match dashboard-design.txt; backfill any drift.
        Snapshot test of V2 workspace + drawers in CI.

  B-12  Tier-0 Shell vs Workspace Decision  [DONE in V2]
        V2 ships screens/Workspace.svelte as the canonical primary shell.
        layout/AppShell.svelte hosts it. Tier-0 Shell concept deprecated.
        Remaining work: verify status bar lives inside Workspace + first-run
        wizard if still desired.

  B-13  Activity Feed Restoration  [REPLACED by orbit visualizer in V2]
        V2 ActivityWindow IS the orbit (Mode 1 PLAN / Mode 2 MAP / Mode 3
        RESEARCH). The text-feed concept is retired per DOMAIN-CONTEXT §2.
        Re-scoped to: orbit captions wiring (lib/orbit/captions.ts), bubble-up
        event strip beneath the tree, FSM-state halo on centre node, and the
        OrbitShootingStar idle effect (already implemented; needs SSB feed).

  B-14  Live Token Streaming Chat
        windows/CommandWindow.svelte streams real LLM tokens via sidecar
        /v1/messages SSE. Today the chat shell + slash palette are scaffolded
        (lib/layout/CommandPalette.svelte) but submit_task is a stub.

GATE FOR PART II: V2 dashboard matches dashboard-design.txt visual identity
AND streams real chat tokens. User's "feel" complaint resolved at architecture
level (orbit centerpiece + Workspace shell); Track D wiring closes the
remaining content gaps.


================================================================================
PART III — HARNESS HARDENING (B-15 through B-20)
================================================================================
Objective: kill dead code, enforce CLAUDE.md hard rules, deduplicate
parallel implementations.

  B-15  Phase 0 Debt Burndown   [PARTIAL — main.rs decomposition DONE 2026-05-08]
        Strip compact.rs, usage.rs ModelPricing, oauth.rs, remote.rs,
        prompt.rs. Replace 84 std HashMap imports. Fix 40+ entropy leaks.
        Decompose cli/src/main.rs (currently 11,181 LOC vs <500 spec).
        Add missing scripts (entropy check, determinism proof, smoke test).
        2026-05-08: cli/src/main.rs decomposed 10,570 -> 252 LOC. Split into
        10 sibling modules: args.rs (928), auth.rs (298), help.rs (216),
        live_cli.rs (3932), progress.rs (403), resume.rs (747),
        runtime_builder.rs (490), runtime_client.rs (753), tool_executor.rs
        (257), tool_formatting.rs (581). Tests moved to cli_tests.rs (2273)
        + sandbox_report_tests.rs (56). Release builds pass: elai-cli +
        elai-dashboard-v2. Tests: 178/180 pass; 2 pre-existing failures
        (legacy .elai/ vs .elai_cc/ path test, unchanged from pre-refactor).
        Other B-15 items (HashMap ban, entropy leaks, dead-code strip,
        scripts, hooks.rs Windows fix) remain pending.

  B-16  Anti-Stale Rule Enforcement
        ort stable; thiserror over Box<dyn Error> in 15+ public APIs;
        blake3 spec-text update; .unwrap/.expect removal; .try_into()
        over as-casts.

  B-17  Ground the Vacuous Gates
        Producer for ungrounded_claims; CSQ enforcement in pre-tool
        dispatch; feasibility blocks in EXECUTING; SearchMode cascade
        wired into query path; SupervisionManager wired into task_runner.

  B-18  Tier C Sandbox Activation
        clone3 + seccomp on Linux; AppContainer on by default on Windows;
        visibility filter "not found" semantics (per spec R31).

  B-19  Hook System Unification
        Pick InProcessHookDispatcher OR IpcHookDispatcher — delete the
        other. Expand DAIRA from 2 to 15 markers. Implement ruff
        PostToolUse subprocess. Stop hook actually enforces non-IDLE block.

  B-20  Tool Registry Unification
        Pick InProcessToolDispatcher OR IpcToolRegistry — delete the
        other. Re-align failure taxonomy to spec's 9 categories.

GATE FOR PART III: zero CLAUDE.md hard rule violations; no dead/duplicate
implementations remaining.


================================================================================
PART IV — FINAL PRODUCT (B-21 through B-28)
================================================================================
Objective: deliver phases 4F-4J + remaining 4K + cloud + focus tunnel.
Each is a substantial multi-session phase.

  B-21  4F Feasibility Probe
        SystemProbe (nvml-wrapper + sysinfo); novel step stubs in Tier C
        sandbox; FeasibilityReport; SSB branch-and-explore (max 3
        alternatives per novel step). 24 tests minimum.

  B-22  4G Autonomous Mode Enablement
        AutonomousGate wraps InteractiveGate; AutoAdvanceEvaluator with
        6 evidence checks; SUBSTITUTE_MODEL via ModelPool swap chain;
        SPLIT_TASK on context budget Terminal (max 4 sub-tasks, depth 2);
        circuit breaker (>10 auto-advances).

  B-23  4H Self-Modification (DGM/CEL/DarwinNet-derived)
        3 layers (data, config, structural); DGM sigmoid-weighted parent
        selection (lambda=10, alpha_0=0.5); staged eval (10/50/200);
        TestOutcomeReport 4-category SWE-bench grading; MSI exp-decay
        (lambda=0.3, N=10) per PipelineMode; AllowedConfigKey enum;
        4 enforcement layers (dispatch + IPC + filesystem + git).

  B-24  4I Media Validator (conditional — only if pipeline produces media)
        Tier 1 deterministic (ImageMagick/FFmpeg/Tesseract/Blender, <100ms);
        Tier 2 VLM advisory (Qwen3.5-VL-30B Mode A; InternVL3.5-38B Mode B);
        CLAP for audio; MediaRenderAdapter trait per output type;
        Render-Capture-Evaluate visual feedback loop.

  B-25  4J Empire Integration (conditional — when Empire codebase accessible)
        FactoryLine protocol client; brain registration with heartbeat +
        reconnection; replace broken brainctl CLI; Empire event subscription
        with SSB bridge; capability exposure (ElaiNeuron 4 capabilities);
        Empire repair pipeline for 3 verified issues.

  B-26  4K.6-4K.13 Dashboard Completion  [V2 REBASELINE — see B-26 file header]
        V2 ships scaffolded MonitorWindow.svelte + ExploreWindow.svelte +
        ReviewWindow.svelte (all stub-backed). Re-scoped to: replace stubs
        with real Tauri invokes, wire intervention card lifecycle (V2 has no
        InterventionCard.svelte yet), browser Notification API + optional
        webhook, V2 panel arrangement per DESIGN.md, Lucide icons audit,
        animation timings per DESIGN.md §7 (hover-reveal) + §8 (orbit modes).
        Window management (tabs/history/recovery) is post-MVP per V2 design
        — defer unless explicitly requested.

  B-27  Cloud Mode (D-1 through D-4 from post-phase4-implementation-order.txt)
        D-1: cloud config + provider routing + cost tracking + CLOUD_SPEND
             SSB action type
        D-2: cloud context budget profile (200K) + baby step sizing per mode
        D-3: privacy redaction pipeline + BYOE + air-gapped mode
        D-4: HYBRID escalation + pre-task cost estimation display

  B-28  Focus Tunnel (E-1 through E-7)
        E-1: FIM prompt assembly + model capability detection
        E-2: diff parser + edit matching integration
        E-3: generation mode selection (FIM/DIFF/STRUCTURED)
        E-4: streaming validation framework (token-level + chunk-level)
        E-5: pipeline parallelism for context pre-assembly
        E-6: automated dependency detection + resolution
        E-7: speculative file routing + micro-step creation

GATE FOR PART IV: full system per architecture-overview.txt, all integration
specs satisfied, dashboard surfaces all of them, cloud + focus tunnel
operational, MSI converging in healthy range.


================================================================================
PART V — KNOWLEDGE ENGINE EXTENSIONS (B-29 through B-35)
================================================================================
Objective: extend the post-A-48 knowledge engine with symbol-level
traversal, multi-language hardening, indexing robustness, and project-
typed artifact knowledge. Source: SocratiCode patterns evaluated 2026-05-05
(33,975-LOC AGPL TS implementation; ELAI re-implements in Rust). Each
B-N includes a SocratiCode-pattern reference but ports the algorithm,
not the source code.

NOTE on motivation: Phases 0..4E and A-0..A-48 are DONE per audit.
These are NEW extensions on top of an already-complete knowledge engine,
not retrofits. They unblock symbol-level safety for the Auditor,
multi-language coverage for the Designer's existing-project mode, and
artifact-grounded reasoning for the Researcher.

  B-29  Symbol-Level Impact Analysis API   (★ highest leverage)
        SymbolGraphCache (LRU + sharded reverse + name index) backing
        four traversal tools: compute_impact_radius, compute_call_flow,
        compute_symbol_context, list_symbols. Auditor pre-edit safety
        check, Designer call-flow grounding. Pre-req: B-1.

  B-29.5 Symbol Graph Scale Benchmark       (sub-step verification)
        Real benchmarks against Linux kernel (~36M LOC), Chromium
        (~35M LOC), VS Code (~2.45M LOC), Postgres (~1.5M LOC).
        Verifies p99 <= 500ms across 4 traversal tools, cache memory
        bounded under 2GB, hop-band determinism across repeated calls.
        Pre-req: B-29 + B-31.

  B-30  Multi-Language DKG Hardening        (closes A-35)
        Python / TS / TSX / JS / Go / Java / Kotlin / Scala / C / C++ /
        C# / Ruby / PHP / Swift / Bash / Svelte / Vue extractors with
        per-grammar failure isolation, JVM suffix map, C# namespace
        map, Go module info, TS path aliases, Svelte/Vue HTML-then-TS
        re-parse. Pre-req: B-1.

  B-31  Resumable Indexing + Lock + Auto-Resume
        Per-batch checkpoint with blake3 content-hash skip; cross-process
        IndexLock (LockScope::Index | Watch) with stale-PID reclaim;
        startup auto-resume of in-progress projects; graceful shutdown
        with 60s deadline. 5 new SSB action types. Pre-req: B-1.

  B-31.5 Resumable Indexing Scale Benchmark (sub-step verification)
        Five-tier benchmark: full clean baseline (Tier A), kill-resume
        identity at 10/50/90% (Tier B), cross-process lock contention
        + stale-PID reclaim (Tier C), graceful shutdown deadline +
        forced release (Tier D), Chromium long-tail memory stability
        (Tier E). Same fixtures as B-29.5. Pre-req: B-31.

  B-32  Adaptive Chunking: Minified Files
        Pre-strategy minified detection (avg_line_length > 500); safe-
        boundary character chunker; universal MAX_CHUNK_CHARS=2000 cap
        applied to ALL chunker outputs (AST + line-window + character).
        Pre-req: B-1.

  B-33  Interactive Decomposition + Graph Viewer (Mermaid + cytoscape)
        Tauri+Svelte+cytoscape graph window with file/symbol/decompose
        modes, blast-radius right-click overlay, search, layout switch,
        PNG export. Plus Mermaid text export (paste into PR/chat).
        Pre-req: B-7, B-29.

  B-34  Entry-Point Detection Tool
        Three-heuristic union (orphans + conventional names + framework
        routes + tests) with reason-tagged EntryPoint output. Auto-
        injected into Designer's ExistingProject decomposition and
        Researcher's Oracle pipeline. Pre-req: B-29.

  B-35  Context Artifacts Pipeline
        Full second indexing pipeline parallel to code: declare DB
        schemas, OpenAPI specs, k8s manifests, ADRs, glossaries via
        [[knowledge.artifacts]] block. Per-project context_* collection,
        blake3 staleness, ArtifactStage in QueryPipeline (STANDARD +
        DEEP), GroundingMethod::ArtifactCheck as 7th GRC method,
        AVAILABLE_ARTIFACTS Designer briefing block. Pre-req: B-1, B-31,
        B-32.

GATE FOR PART V: Auditor calls compute_impact_radius pre-edit on every
Write/Edit; multi-lang DKG operates on real Python/TS/Go/Java repos with
unresolved_edge_pct < 50%; full re-index of 50K-file project survives
mid-run kill -9; minified bundles index without provider truncation;
GraphWindow renders ELAI's own crate dependency graph with right-click
blast radius working; Designer's ExistingProject decomposition references
real entry points + AVAILABLE_ARTIFACTS in briefings.


================================================================================
PART VII — OUROBOROS-DERIVED EXTENSIONS (B-37, B-38)  [NEW 2026-05-10]
================================================================================
Objective: port the four directly-applicable patterns from
github.com/Q00/ouroboros (MIT-licensed). The ouroboros review surfaced
patterns ELAI's existing phases benefit from. Some go INTO existing
B-Ns (B-22/B-23) as additive sub-steps; the runtime backend abstraction
and the ambiguity score retrofit are independent enough to warrant
their own B-Ns.

  B-37  Phase 4M — Multi-Runtime Backend Abstraction
        Single RuntimeAdapter trait covering 11 backends (2 existing +
        7 ouroboros CLI ports + 2 native HTTP). Per-role hybrid topology.
        Runtime fallback chain orthogonal to 4G SUBSTITUTE_MODEL. ALL
        CLI subprocesses run inside Tier C isolation. Spec lives at
        docs/specs/guide/steps/phase-4M.txt.
        Pre-req: B-1, B-15..B-20, B-22, B-18; Phase 3J + 3J.LOCAL +
        3J.PIPELINE done in code.
        Parallel-safe with: B-23, B-24, B-25, B-27, B-28.
        Estimated effort: 3-4 sessions.

  B-38  Ambiguity-Score Continuous Metric (small standalone retrofit)
        Adds ouroboros's ambiguity formula + per-dimension floors +
        2-snapshot streak requirement onto B-17's existing
        GroundingSweepGate. Continuous score replaces binary all-or-
        nothing verdict. Floors block "aggregate passes but one
        dimension is dangerously low" cases.
        Pre-req: B-17 (DONE), B-8 (PipelineGrc activation).
        Parallel-safe with: any non-grounding-related B-N.
        Estimated effort: 1 session.

  Additive sub-steps already injected into existing B-Ns:

    Phase 4G + B-22b (autonomous mode follow-up):
      4G.9 STAGNATION + PERSONAS + WATCHDOG + GOAL-DRIFT CANARY
        Replaces ELAI's lone "3x same GRC step -> escalate" anti-spiral
        with a 4-pattern detector (Spinning/Oscillation/NoDrift/
        DiminishingReturns) + 5-persona unstuck path
        (Hacker/Researcher/Simplifier/Architect/Contrarian) with
        explicit affinity table + GenerationWatchdog (3 timeout kinds
        + workflow_fingerprint dedup) + optional parallel multi-persona
        fan-out for context-isolated lateral thinking + goal-drift
        Jaccard canary that blocks topically misaligned baby step
        outputs BEFORE audit (saves the audit LLM call on Block).
        See: docs/specs/guide/steps/phase-4G.txt 4G.9 section,
             docs/thefinalglance/B-plan/B-22_phase_4G_autonomous_mode.md
             "Follow-up: B-22b" section.

    Phase 4H + B-23 (self-modification):
      4H.7B OSC -- Ontology-Similarity Convergence
        ConfigVariantSimilarity::compute uses ouroboros's
        0.5*key_overlap + 0.3*type_match + 0.2*exact_match. Stagnant
        verdict over 3 consecutive >= 0.95 boosts exploration.
        Oscillating verdict via PERIOD-2 FULL VERIFICATION freezes
        differing keys.
      4H.7C REG -- Cross-Pipeline AC Regression
        RegressionDetector spans full PipelineOutcome history (catches
        slow regressions TestOutcomeReport.PASS_TO_FAIL misses).
        Blocks promotion in 4H.5; triggers auto-revert in 4H.6.
      4H.7D EVO -- Evolution Gate
        Catches "fake IMPROVING" when MSI moves on score noise but
        config_blob is structurally identical. Withhold -> MSI forced
        to STABILIZING.
      4 new SSB action_types: convergence_stagnant, config_oscillation,
        regression_detected, evolution_withheld.
        See: docs/specs/guide/steps/phase-4H.txt 4H.7B-D sections,
             docs/thefinalglance/B-plan/B-23_phase_4H_self_modification.md
             tasks 14-17.

GATE FOR PART VII: 4G.9 stagnation+persona+watchdog active in
autonomous loop, 4H.7B-D OSC/REG/EVO gating self-mod variant
promotion, B-37 multi-runtime adapters operational, B-38 ambiguity
score continuous gate live in PLANNING -> TESTING_FIRST.


================================================================================
PART VI — DASHBOARD-V2 SRC-TAURI WIRING (B-36)  [NEW 2026-05-07]
================================================================================
Objective: complete Track D of NEXT-SESSION-FINISH-DASHBOARD-V2.md — wire the
13 D-commands + 15 net-new V2 surfaces into crates/dashboard-v2/src-tauri/src/
lib.rs and replace the corresponding entries in src/lib/api/stub.ts with real
Tauri invokes. This is the dedicated dashboard-v2 wiring track.

  B-36  Track D V2 src-tauri Wiring (D.1..D.13 + 15 net-new surfaces)
        Pre-req: B-1 path unification; B-2 FSM phase transition; B-3 stub
                 audit; B-7 decomposition tree producer; B-29 (only if
                 D.x consumes symbol-level data).
        Parallel-safe: with B-29..B-35 (different crate scope).
        Multi-session: planned as Sessions 1..4 in
        workflow/sessions/s-46448/HANDOFF-track-d-v2-rebaseline.md.
          Session 1 — REBASELINE (analysis-only, no code): produces
            docs/specs/track-d-v2-spec.md + B-plan delta document.
          Session 2 — engine wiring: add elai-orchestrator + elai-cli
            dependencies to src-tauri/Cargo.toml; AppState with shared
            handles; D.1 (sidecar_health) + D.13 (sidecar_settings).
          Session 3 — research/planning: D.2 (run_planning_task with
            web_research_step emit), D.3 (run_research), D.7 (one_shot_prompt).
          Session 4 — settings/observability: D.4-D.6 + D.8-D.12 + the 15
            net-new surfaces.
        Deliverables per session: spec, prompt-engineer transformed prompt,
        track-D-Sn-execution-N.md, audit report, screenshot of each new
        live command in dev mode, memory-writer pass.

GATE FOR PART VI: every command listed in DOMAIN-CONTEXT.md §1 + the gap-list
block has either a live Tauri invoke OR an explicit "(post-MVP / deferred to
B-N)" label in stub.ts. cargo build --release -p elai-dashboard-v2 succeeds.
pnpm tauri:dev launches; each window exercised once with screenshot in
workflow/sessions/<sid>/screenshots/D-<item>.png. CLI launcher constant
DASHBOARD_BINARY_NAME at crates/cli/src/dashboard.rs:7 reads
"elai-dashboard-v2" (or override via ELAI_DASHBOARD_BINARY env var documented
in elai.bat).

Recommended ranking (cut from bottom if needed):
  1. B-29   highest leverage / unblocks Auditor pre-edit safety
  2. B-31   robustness baseline / unblocks multi-million-LOC repos
  3. B-30   closes A-35 / multi-language coverage
  4. B-35   Designer + Researcher grounding fuel
  5. B-32   cheap; fixes minified-file bugs
  6. B-33   visible polish; consumes B-29 + B-7
  7. B-34   smallest; nice-to-have for Designer
  8. B-29.5 scale verification of B-29 (post-B-29 acceptance gate)
  9. B-31.5 scale verification of B-31 (post-B-31 acceptance gate)

  NOTE: B-29.5 and B-31.5 are sub-step verifications, not new features.
  They run AFTER their parent B-N implements + ships, and produce a
  signed metrics artifact proving the parent's claims at scale.
  Without them, "ELAI handles 50M-LOC repos" remains unverified
  marketing — the same gap SocratiCode left in their own claims.

Independence map (parallel-safe):
  B-29 ⟂ B-30 ⟂ B-31 ⟂ B-32 (all parallel-safe with each other)
  B-33 needs B-29 + B-7
  B-34 needs B-29
  B-35 needs B-1 + B-31 + B-32
  B-29.5 needs B-29 + B-31 (so resumable indexing can survive the
                            multi-hour fixture builds)
  B-31.5 needs B-31 (and reuses B-29.5 fixtures if B-29.5 ran first)


================================================================================
B-N FILE STRUCTURE (REQUIRED FORMAT)
================================================================================

Each B-N file follows this skeleton:

```
# B-N: <title>
Objective: <one sentence>
Estimated effort: <hours/sessions>
Pre-requisites: <list of B-N or external>
Parallel-safe with: <list of other B-N>

## Context
<background — why this matters, what audit found>

## Reads (mandatory before coding)
- <file:line-range>
- <file:line-range>

## Tasks
1. <numbered actionable step>
2. ...

## WIRING_CONTRACT (per Layer 0 of testing-trust-design.txt)
produces_symbols:
  - <fully-qualified-path>
consumes_symbols:
  - <fully-qualified-path>
reachable_from:
  - <entrypoint>
stub_ban:
  - <config_field_path != forbidden_value>
frontend_pair: <baby-step-id-or-null>
observable_effect:
  trigger: <command on shipped binary>
  expect_ssb: <action_type within Nms>
  expect_artifact: <file/stdout pattern>

## DETERMINISTIC_CHECK
<commands the auditor runs to verify done>

## ACCEPTANCE
- <binary pass/fail per deliverable>

## Tests to add
- <list>

## Files touched
- CREATE: <path> (<purpose>)
- MODIFY: <path> (<change summary>)
- DELETE: <path> (<reason>)

## Decision doc
write workflow/reasoning/B-N-decisions.md before EXECUTING

## Next
B-N+1 (or branch points)
```


================================================================================
COMMIT DISCIPLINE (PER CLAUDE.md)
================================================================================

  - Commit message format: "B-N: <imperative summary>"
  - Include test pass/fail counts in EVERY commit
  - Update memory-bank after every build/test (no batching)
  - Never skip hooks (--no-verify) or bypass signing
  - No new dependencies without explicit user confirmation
  - Always commit and push after finishing a B-N
  - workflow/, memory-bank/, spec files are gitignored — only .elai_cc
    folder contents are tracked


================================================================================
TIME ESTIMATE
================================================================================

  Part I   (B-1..10):  ~10-15 sessions
  Part II  (B-11..14): ~4-6 sessions
  Part III (B-15..20): ~6-10 sessions
  Part IV  (B-21..28): ~16-24 sessions (each big phase is 2-3 sessions)
  Part V   (B-29..35): ~11-17 sessions (B-29: 2-3, B-30: 3-4, B-31: 2,
                        B-32: 0.5-1, B-33: 2-3, B-34: 1, B-35: 2)
  Part V scale verifications (B-29.5, B-31.5): ~2 sessions (mostly
                        wall-clock waiting on multi-hour indexing of
                        Linux + Chromium fixtures; ~6-8h compute each)
  Part VI  (B-36):     ~4 sessions (Track D V2 wiring per
                        workflow/sessions/s-46448/
                        HANDOFF-track-d-v2-rebaseline.md)
  Part VII (B-37, B-38): ~5 sessions (B-37 multi-runtime backend
                        abstraction: 3-4; B-38 ambiguity score
                        retrofit: 1; plus the 4G.9 + 4H.7B-D additive
                        sub-steps already injected into B-22/B-23
                        spend their effort inside those B-Ns)

  TOTAL: ~53-78 implementation sessions to ship full system per spec
         + Part V knowledge-engine extensions + scale verifications
         + Part VI dashboard-v2 src-tauri wiring.

  After Part I+II (~14-21 sessions): MVP is visually correct and functionally
  wired. Demo-able to a stakeholder.

  After Part III (~20-31 sessions): production-grade hardening. Ready to ship.

  After Part IV (~36-55 sessions): full-spec system including autonomous
  mode, self-modification, cloud, focus tunnel.

  After Part V (~47-72 sessions): symbol-level safety enforcement,
  multi-language coverage, indexing robustness, artifact-grounded
  Designer + Researcher. ELAI is no longer Rust-only in practice.


================================================================================
END INDEX — proceed to B-1_path_unification.md
================================================================================

Part V file list:
  B-29_symbol_level_impact_api.md
  B-29.5_symbol_graph_scale_benchmark.md
  B-30_multilang_dkg_hardening.md
  B-31_resumable_indexing_lock_lifecycle.md
  B-31.5_resumable_indexing_scale_benchmark.md
  B-32_minified_file_chunking.md
  B-33_interactive_graph_viewer.md
  B-34_entry_point_detection.md
  B-35_context_artifacts_pipeline.md


================================================================================
## SESSION s-22300 + s-22600 -- PLAN-STATUS RECONCILIATION (2026-05-31)
================================================================================

Append-only session-narrative reconciliation block. The s-night-2026-05-30 +
s-21636 + s-22300 orchestrate chain drove 12 plans to a verdict against the
canonical surface; the prior B-0_INDEX hygiene attempt (s-22300 Tranche D-5)
to update per-plan status rows was ABORTED after explorer found the file is a
session-narrative structure (5 blocks at L89/L244/L294/L333/L385), not a
row-per-plan-status table. 11 of the 12 plan codes returned zero hits on a
naive Grep, so per-row Edit attempts would have created orphan content. This
block is the correct shape per Tranche-D-hygiene-decisions.md DECISION-5: one
new SESSION block at end-of-file, prose entries per plan verdict.

### Plan verdicts (12)

E-47 SHIPPED (Tranche B-1, s-22300) -- LoRA ship-gate with triple-AND token
predicate (`lora_calibration_passed` AND `lora_artifact_pinned` AND
`lora_runbook_executed`); 320 LOC `lora_ship_gate.rs` + 3-arm
`LoraShipGateOutcome` + 5 unit tests; Pattern-2 44th application with
`SsbActionType::LoraShipGateEvent` + 3-arm `LoraShipGateEventPhase`
(Refused / CalibrationMissing / CalibrationPassed); SCHEMA_VERSION_PUBLIC
191 -> 192; new CLI subcommand `elai lora-gate-check`; LoRA artifact
PRODUCER remains RUNPOD-PENDING long-lived stub (see s-22300
stubs.md E47-LORA-PRODUCER); env gate `ELAI_LORA_SHIP_GATE_ENABLE`
wire-dark default OFF; flip runbook at workflow/decisions/E-47-flip-runbook.md.

E-40 SHIPPED (Tranche B-2, s-22300) -- SymbolGraphDSL with nom 7 parser
(330 LOC dsl.rs SELECTOR/FILTER chain) + BFS depth-bounded dataflow analyzer
(250 LOC dataflow.rs) + 2-iter RRF refine_filter alpha=0.6 beta=0.4
(180 LOC); knowledge crate gets 5th-list RRF integration at weight 0.9
behind `ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE`; SymbolGraphAdapter trait on
knowledge side (75 LOC); Pattern-2 45th application with SymbolGraphEvent
+ 4-arm SymbolGraphEventKind (SymbolDslExecuted / DataflowImpactComputed /
AgentImpactClaimRefuted / ScopeGraphResolved); SCHEMA 192 -> 193; 2 new
TOOL_SYMBOL_DSL_QUERY + TOOL_SYMBOL_DATAFLOW_IMPACT ipc-types;
e40_symbol_graph_contract.rs 9 tests. Live translate_tool_request wiring
in production_agent_rpc.rs deferred (currently returns empty typed
responses + emit beacons).

E-52 SHIPPED LOCAL-ONLY (Tranche B-3, s-22300) -- LSP verifier
local-only with diagnostic_probe.rs (400 LOC) using which/where binary
detection + tokio timeout + ToolMissing soft-pass fast-path; new
TransitionContext.diagnostic_probe_outcome field + DiagnosticProbeGate
+ all_gates() slot 9; env-gated dispatch loop in run_validate_dispatch
Ok-branch with TODO[E-52-S3] for FSM driver wiring;
e52_diagnostic_probe_contract.rs 6 tests; no SCHEMA bump (Pattern-2
reuse). 3 follow-ons deferred (multilspy, ts-morph, ScipRefresh) -- all
need user dep confirmation (anti-slopsquatting) or rust SCIP producer
that does not exist today; tracked under E-52-multilspy /
E-52-tsmorph / E-52-ScipRefresh stubs.

E-25 SHIPPED (Tranche B-4, s-22300) -- compaction_ladder T1..T6 LOCKED
cheapest-first order with stop-at-budget (290 LOC); skill_templating
shell !`cmd` expander with 4-layer defense (direct Command no-shell +
metachar pre-allowlist reject + 4-cmd allowlist git/ls/pwd/date +
wire-dark gate, 340 LOC); Fsm.opus_plan_active AtomicBool overlay (NOT
new FsmSubstate); SessionFork.forked_from Option<String> back-compat;
2 emit sites at handoff.rs L568 DAG + L628 fallback; dashboard
fork_session_branch emit; no SCHEMA bump (Pattern-2 reuse via
ClaudeCodeArchitectureEventKind::{CompactionLadderTierApplied,
SkillTemplateExecuted, SessionForked}). 4 env gates wire-dark default
OFF. 3 deferred stubs (notify crate dep, dashboard panels,
edit_checkpoints).

D-18 SHIPPED (Tranche B-5, s-22300) -- ObservableEffect real executor
replaces s-21636 smoke beacon; observable_effect.rs gets
NoOpSandboxExecutor (L379) + check_goodhart_risk_with_min back-compat
(L1151); task_runner.rs::dispatch_observable_effect_hook real-wires the
ObservableEffectDispatched -> verify_effect_batch ->
check_goodhart_risk_with_min(2) chain at L10602-L10849; legacy
smoke-beacon retained behind ELAI_OBSERVABLE_EFFECT_SMOKE_LEGACY_ENABLE
for one release window then drops; d18_observable_effect_real_contract.rs
6 tests; no SCHEMA bump (GoodhartRiskFlag variant shipped s-21636).

C-42 SHIPPED plain-fn guard (Tranche B-6 quad, s-22300) -- new
McpAuthzSsbEmitter trait + compose_with_tool_gate_emit wrapper at
plugins/authz/compose.rs:84 (legacy 6-arg signature preserved). The
original proc-macro authz attribute (`#[authz]`) is DEFERRED to a future
C-42b plan -- the plain-fn guard closes the SHIPPED-WIRED gap without
first-proc-macro friction. Pattern-2 47th application contributes
McpAuthzEvent variant to the B-6 quad SCHEMA bump 193 -> 194.

C-47 SHIPPED (Tranche B-6 quad, s-22300) -- CaseBase 3 method extension
(add_case / select_case / apply_calibration); Pattern-2 48th application
collapses 3 deprecated SsbActionType variants
(tool_calibration_hint_emitted / drift_detected / call_logged) into
single-variant ToolCalibrationEvent + 6-arm Kind discriminator
(HintEmitted / DriftDetected / CallLogged / CaseAdded / CaseSelected /
CalibrationApplied); deprecated variants kept one-release-window;
SCHEMA 193 -> 194 (combined with B-6 quad bumps); sled REJECTED on
anti-slopsquat (dormant, 57 dl/mo equivalent), rusqlite 0.39 already
shipped; persistence layer (live SQLite-backed CaseBase swap)
DEFERRED to C-47b.

B-39 SHIPPED soft-warn (Tranche B-6 quad, s-22300) -- REFLECT pipeline
budget soft-warn at 85% of hard cap; new
REFLECT_BUDGET_SOFT_WARN_PCT=85 const + ChatReflectConfig
.budget_soft_warn_pct field + ReflectEvent::BudgetSoftWarn 5th variant
{turn_id, used, soft_cap, hard_cap, pct}; Pattern-2 49th application
adds SsbActionType::ReflectBudgetSoftWarn under
ELAI_B39_SOFTWARN_ENABLE gate; dashboard reflect_bridge.rs:123 needed a
retry to land the 5th match-arm correctly (the first retry invented
field names; the second retry read chat_reflect.rs:280-286 for the
canonical shape).

C-50 SHIPPED (Tranche B-6 quad, s-22300) -- contract-coding retry
emit; new crates/orchestrator/src/contract_coding/retry_emit.rs
(180 LOC) + 3 emit sites at task_runner.rs L1058 / L1123 / L1159
(Healed / EscalatedToHuman / iterations++Failed); Pattern-2 50th
application adds ContractCodingEvent. SCHEMA 193 -> 194 (combined
B-6 quad bump). drive_task call-site wiring DEFERRED -- currently
fix_and_rerun_until_green emits in isolation, guarded by
is_web_project.

E-13 SHIPPED (Tranche B-7 triple, s-22300) -- survival_cache.rs
365 LOC rusqlite-backed with SurvivorStatus 3-arm + ToolchainMissing
-> Skipped short-circuit; new workflow/tooling/mutant-hunter.md
agent doc (150 LOC; cargo-mutants live run RUNPOD-PENDING); Pattern-2
46th application TddVerificationEvent + 4-arm Kind (PropertyGatePass /
MutationGateSurvivorBlock / AntiMockBlock / AssertionOnlyDiffBlock);
SCHEMA 194 -> 195; env gates ELAI_E13_TDD_VERIFICATION_EMIT +
ELAI_E13_SURVIVAL_CACHE wire-dark.

E-12 partial (Tranche B-7 triple, s-22300) -- 3 framework manifests
shipped at workflow/archetypes/{wasp,encore,t3}.toml with
UNVERIFIED-PENDING-CONFIRM version pins + TODO[E12-anti-slopsquat]
markers (orgs verified via WebFetch but release-date staleness flagged
by executor). Anti-slopsquat verification refresh queued for a 2026
release-tag pass per workflow/reasoning/E-12-decisions.md follow-up
once registry data is current.

E-50 SHIPPED (Tranche B-7 triple + Tranche D-2, s-22300; residual D-3
prose drift closed s-22600 ITEM 3) -- SaasArchPatternEventKind
extended with 12th within-variant arm SaasArchetypeSelected (NO new
top-level variant); +recommender.rs ArchetypeSelectionSource 5-arm +
recommend_archetype_with_emit (225 LOC); Pattern-2 reuse, no SCHEMA
bump from E-50 itself but rides the B-7 195 line. Vocabulary
dictionary block prepended in Tranche D-2 documenting legacy
4-bucket -> canonical 5-variant SaasArchetype mapping
(SaasDashboard / B2bMarketplace / InternalTooling / DeveloperApi /
ConsumerSocial). Residual 7-line prose drift at L88/L120/L169-171/
L201/L213/L229/L250-256 closed in s-22600 Tranche A ITEM 3 (7
mechanical edits referencing the dictionary block).

### Cross-reference

- s-night-2026-05-30 handoff: workflow/sessions/s-night-2026-05-30/
- s-21636 handoff (6 tranches, 7-9K LOC): workflow/sessions/s-21636/
- s-22300 handoff (10 tranches, ~6K LOC): workflow/sessions/s-22300/handoff/HANDOFF-s-22300-2026-05-31.md
- s-22600 hygiene tranches: workflow/sessions/s-22600/exploration/trancheA-prose-report.md
- Decisions doc anchoring this block: workflow/reasoning/Tranche-D-hygiene-decisions.md DECISION-5

### Status

NO COMMITS made for s-22300 + s-22600 per user feedback_no_auto_commit;
ONE commit when user authorizes, maintainer-voice per
.claude/rules/commit-cadence.md (no AI trailer, no Phase-N prefix).
