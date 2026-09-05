# B-Plan Master Summary / Index
Generated: 2026-05-17 — synthesized from full read of B-0_INDEX + B-1..B-42 (45 files).
Source root: F:\.ELAI_workflow\docs\thefinalglance\B-plan\

Use this as a topological map: every B-N has objective, prereqs, deliverables,
status (from B-0 status blocks), and a one-sentence rationale. Files themselves
remain the canonical specs.

---

## TOC (parts)

- PART I  — MVP Completion (B-1..B-10)
- PART II — Dashboard Visual Restoration (B-11..B-14) [V2-rebaselined]
- PART III — Harness Hardening (B-15..B-20)
- PART IV — Final Product (B-21..B-28) — Phases 4F-4J + Cloud + Focus Tunnel
- PART V — Knowledge Engine Extensions (B-29..B-35) — SocratiCode-derived
- PART VI — Dashboard-V2 src-tauri Wiring (B-36)
- PART VII — Ouroboros-derived Extensions (B-37, B-38)
- PART VIII — Chat REFLECT + Idea-Framing + Per-Level Audits + Cosmos (B-39..B-42)
- Cross-cutting: V2 REBASELINE (2026-05-07), session log, shipped phases.

## Status legend
- DONE: shipped per session log in B-0 INDEX
- PARTIAL: shipped with explicit deferred items
- PLANNED: file exists, not yet executed
- DEFERRED: explicitly post-MVP / conditional

---

## V2 REBASELINE (2026-05-07) — context for any dashboard B-N

Original B-1..B-35 authored against V1 dashboard at `crates/dashboard/` (now parked
at `crates/.bkp_old/dashboard/`). Active dashboard is V2 at `crates/dashboard-v2/`
(Tauri 2.10 + Svelte 5 runes + Vite 5 + three.js orbit).

Path map V1→V2:
- `crates/dashboard/src/commands.rs` → `crates/dashboard-v2/src-tauri/src/lib.rs`
- `src-frontend/src/` → `src/lib/`
- `ChatWindow.svelte` → `windows/CommandWindow.svelte`
- `ActivityFeed.svelte` → `windows/ActivityWindow.svelte` (NOW the orbit visualizer,
  Mode 1 PLAN / Mode 2 MAP / Mode 3 RESEARCH per DESIGN.md §8 — text-feed retired)
- `Settings.svelte` → `windows/SettingsWindow.svelte`
- 10 drawers → `lib/drawers/{Knowledge,Reasoning,Orchestration,Rules,Research,
  Providers,ProjectIntelligence,AntiStaleness,ToolsHooks,WebQuality}Drawer.svelte`
- bin `elai-dashboard` → `elai-dashboard-v2` (cli/src/dashboard.rs:7 still V1 —
  fix in B-36)

V2 architecture invariants:
- Only 2 Tauri commands wired pre-B-36 (greet, get_grounding_cache_stats stub)
- ~60+ frontend "commands" stubbed via `src/lib/api/stub.ts` backed by mock-data.ts
- Svelte 5 runes only ($state/$derived/$effect/$bindable); no `$:` labels; no
  createEventDispatcher
- Mounts via Svelte 5 `mount(App, { target })` (commit f6f96b0)
- Engine crates intentionally omitted from src-tauri/Cargo.toml — adding any
  requires user confirmation

---

## SESSION LOG (from B-0 INDEX)

s-59964 (2026-05-07): shipped B-1, B-2, B-3, B-4, B-5, B-6, B-7 (PARTIAL→DONE via
  B-7b followup), B-8, B-9, B-10, B-13, B-14, B-24, B-26 (PARTIAL→DONE via Session 3
  followup), B-36 (PARTIAL→DONE via D.2-D.5 followup) + 2 net-new (auto-remote-sidecar,
  dashboard remote overlay UI). SCHEMA v25→26. Commits 04f68b8 etc.
s-26716 (2026-05-09): B-22 Phase 4G autonomous mode + patch-4G.1 (CommandPalette
  toggle live IPC + StatusBar gate badge promotion).
s-30584 (2026-05-10): B-39 chat REFLECT pipeline (observability v0; SCHEMA 28→29).
  Commit 6dd4af5.
s-34648 (2026-05-11): B-39C REFLECT answer substitution (text-only D2a). Commit
  6adb454.
ad-hoc (2026-05-11): oracle-gate.1 dashboard patch — RESEARCH intent chip with
  one-key confirm (Enter accept / Esc reject / Tab edit).

Per per-session memory updates:
- B-37 SHIPPED (commit fd5c2e5): Multi-Runtime Backend Abstraction Phase 4M;
  11 backends + RuntimeAdapter + PerRoleRuntime + RuntimeOrchestrator + 4 SSB
  action_types + /runtime CLI + RuntimePanel; live cloud Sonnet 4.6 PROVEN.
- B-40 SHIPPED: Idea-Framer + Research Presets + Critic Loop; types at
  orchestrator::idea_framing; FsmSubstate::Framing; 5 SSB variants; SCHEMA 41
  (pin 42 vs B-34); IPC v2 unchanged; 79/79 PASS.
- B-41 SHIPPED: Per-Level Auditor Modes; auditor_mode + per_level_audit_dispatch
  + AgentRole +5 + Evidence +5 + AuditTimelineWindow; 80 GREEN; SCHEMA 45;
  navigator-spawn deferred.
- B-42 SHIPPED: Cosmos Artifact Tier: cosmos_worker (~700 LOC) + briefing
  compiler + VerificationMethod::CosmosCheck + SCHEMA 44 + CosmosDrawer;
  spawn_loop dispatcher deferred.

Open patch-N (from git log front-of-this-session): patch-R2.3 lenient_vec
deserializer; patch-R2.2 Windows IPC path fix + agent stderr; patch-R2.1 B3
prompt rephrase; patch-R2 CloudGroundingClient async OAuth + B3 cloud-mode.

---

## PART I — MVP COMPLETION (B-1..B-10)

Objective: make what's built actually run end-to-end visibly. Gate: live FSM
badge, real engine telemetry per query, real GRC budgets per agent, DAG
visualization, branch UI, zero DEAD UI elements.

### B-1: Path Unification (.elai → .elai_cc everywhere)  [DONE s-59964]
- Highest-leverage fix in entire B-plan. Two-line patch unblocks PageRank,
  CatRAG, A-RAG, DkgQuerierAdapter at runtime.
- Tasks: audit `.elai/` literals; promote `ELAI_DATA_DIR_NAME = ".elai_cc"` +
  `INDEXES_DIR_NAME = "indexes"` to `crates/core/src/paths.rs`; helpers
  `dkg_db_path / bm25_dir_path / vectors_db_path / sessions_dir_path /
  metrics_dir_path / models_dir_path / rules_db_path`; fix cli/main.rs:3752,
  3756, 5437; update indexing_service.rs:15-18; migrate any `.elai/sessions/
  dkg.db` rule-engine reads; assert .elai_cc exists at CLI bootstrap (no
  silent EmptyDkgQuerier).
- Side benefits: Phase 1D PageRank yields real symbols; 1E CatRAG populated;
  Telescope gets real counts; ~5 phase audits flip PARTIAL→WIRED.

### B-2: FSM PhaseTransition Wiring (30-min fix)  [DONE s-59964]
- Append `writer.write_payload("phase_transition", 0,
  SsbPayload::PhaseTransition(...), Vec::new())` inside
  `Fsm::transition_with_metrics` at fsm.rs:639.
- Fsm gains `ssb_writer: Option<Arc<dyn SsbWriter>>` (Optional for tests).
- Migrate `emit_fsm_transition` at task_runner.rs:2346 to same `"phase_transition"`
  action_type (consolidate; do not maintain two).
- SSB write failure must NOT block FSM transition.
- Activates StatusBar badge, EXECUTING pulse, MonitorWindow.currently,
  ActivityEntry phase_transition renderer (V2: surfaces as halo on centre orbit
  node + bubble-up event strip).

### B-3: Stub Command Eradication  [DONE s-59964; V2 re-scoped to AUDIT/CLASSIFY + CLASS B/C]
- For each of 30 dead UI elements: CLASS A (IMPLEMENT) / CLASS B (DELETE) / CLASS C
  (RENAME/DEFER with "(post-MVP, see B-N)" label).
- KnowledgeDrawer (5): languages line, pagerank, query tier stats, freshness, knowledge_degradation
- AntiStalenessDrawer (3): dep_audit, baseline_harness, deterministic_audit (CLASS C)
- ReasoningDrawer (2): GRC synthetic data (couples B-5/B-8), label fix
- RulesDrawer (1): live update + rule_transitions echo
- Plan window (3): PlanWindow unregistered (V1 specific), tree readers stubs
- Settings (2): SessionHandoffEntry schema fix via ts-rs; Reindex button fake
- Activity (3): branch_merge, isolation metric (CLASS B), /continue placeholder
- StatusBar (4): all activated by B-2
- Branch ops (2): /session fork in dashboard, merge/close UI
- Future-phase (4 CLASS C): AutoAdvanceStats, ProposalsPanel, MONITOR, EXPLORE
- ts-rs dep added (reused by all subsequent B-Ns).
- V2: B-3 covers AUDIT/CLASSIFY pass + CLASS B/C deletes; B-36 covers CLASS A
  live-wire (don't double-edit stub.ts).

### B-4: Producer Wiring — Knowledge  [DONE s-59964; code_vec opt-in via ELAI_CODE_VEC=1]
- Stage 3 ONNX models (cross-encoder-l12, GraphCodeBERT, Granite) at
  `.elai_cc/models/onnx/`; setup script with blake3 verify.
- Wire SearchMode cascade into QueryPipeline (replace NoopSsbNotifier at
  cli/main.rs:3721 with real one).
- Query code_vec at runtime → 3-ranker RRF (BM25 + code_vec + text_vec).
- QueryTrace inspector dashboard surface (`get_query_trace`); per-model
  EmbeddingCircuitBreaker grid (`get_circuit_breaker_states`); HyDE preview
  (`hyde_query` SSB); per-tier R@5/R@10/MRR badges.
- Fix DEEP-tier MRR < STANDARD-tier MRR monotonicity violation.
- Real per-stage latency (thread Clock through stage constructors; fix entropy
  quarantine workaround at pipeline.rs:30-33).
- Fix `.expect("semaphore closed")` at reranker.rs:465 → circuit breaker path.
- Replace 14 `Box<dyn Error>` with thiserror enums (Bm25Error/VectorError/
  PagerankError) in bm25.rs, vector/indexer.rs, pagerank/mod.rs.
- New SSB action_types: knowledge_degradation, hyde_query, query_trace,
  circuit_breaker_state_change, vector_cache_hit_rate.

### B-5: Producer Wiring — Reasoning  [DONE s-59964; per-agent budget wire as TODO]
- Replace synthetic emit_reasoning at task_runner.rs:3985-4004 (hardcoded
  agent_id="task_runner", grounding_level="medium" literal, etc.) with real
  PipelineGrc-sourced values.
- 4-5 new SSB action_types: grc_step_emitted, grounding_sweep_result,
  csq_evaluated, feasibility_check, auditor_rubber_stamp_rejected.
- 5 new Tauri commands: get_grc_budgets_per_agent, get_grounding_sweep,
  get_csq_log, get_feasibility_log, get_agent_grc_config.
- ReasoningDrawer redesign: 8 per-agent budget gauges + grounding sweep pie
  chart + CSQ guards section + feasibility checks + per-role config viewer
  (highlights B-15 divergent-table bug in red).
- Anti-rubber-stamp counter; CSQ Critical-without-guard intervention card.

### B-6: Producer Wiring — Orchestration  [DONE s-59964; SCHEMA 25→26; 5 new SSB variants]
- EvidenceGate verdict panel (7 gates from evidence.rs:471).
- InteractiveGate prompt UI (intervention_request + intervention_response SSB).
- AgentsPanel: 8 roles with live status + heartbeat dot + restart count.
- SupervisionManager wired into task_runner (fixes phase-3C R1 — closes infinite
  restart loop risk).
- Anti-spiral hits visible (red badge at count=3).
- BSP superstep counter + parallel-agent run (`bsp_superstep` SSB).
- ContextBudget per-slot bar chart (`get_last_context_budget`).
- AdaptiveLevel reduction events.
- `compute_overlap` function for 3E.5 file_overlap_score producer.
- 9 SSB action_types: evidence_gate_evaluated, intervention_request,
  intervention_response, supervision_event, anti_spiral_hit, bsp_superstep,
  context_reduction, ssb_filtered_by_overlap, agent_status_changed.

### B-7: Producer Wiring — Pipeline Factory (4D+4E)  [DONE via B-7b; WebResearchStage emitter wired]
- Register PlanWindow.svelte (V1 path; V2 already scaffolded under
  `lib/windows/PlanWindow.svelte`).
- DecompositionTree producer: emit concept_decomposition + phase_node_ready +
  step_node_ready + baby_step_ready.
- Implement real get_decomposition_tree / get_phase_detail / get_baby_step_detail
  reads (replace NotFound stubs).
- DAG repair events log (dag_repair_round, dag_repair_escalated SSB).
- ExecutablePipeline DAG visualizer (d3 / cytoscape.js).
- PhaseSizingSplitter events; TreeNavigator state cursor; FailureRecord per
  baby step; BubbleUpEvent typed surface (5 actions cascade).
- IntegrationCheckpoint successes (currently only failures emit).
- Research tree per-node FORMULATE/SEARCH/EXTRACT/REFLECT/CONVERGE events
  (granular per-stage instead of 5 coarse end-of-iteration tokens at
  web_research_orchestrator.rs:166-396).
- PackageRef.verified anti-hallucination badges; TechStackDecision visualization
  (RESEARCH/EXISTING_PROJECT/USER_SPECIFIED/FALLBACK).
- 14 new SSB action_types; 9 new/replaced Tauri commands.

### B-8: PipelineGrc Activation in Production Hot Paths  [DONE s-59964 via with_pipeline_grc()]
- Wire `ground_plan` into PLANNING → TESTING_FIRST (populates
  ctx.ungrounded_claims so GroundingSweepGate has real data).
- `verify_claim` into ContextCompiler pre-fetch (skip bogus deps).
- `generate_csq` into pre-write dispatch (CSQ Critical without guard BLOCKS).
- `check_inline_feasibility` into EXECUTING (Blocked → ReplanRequest(MAJOR)).
- Anti-rubber-stamp Auditor activation (engine.rs:297-315 enforcement live).
- `InlineFeasibilityState.reset()` at FSM phase boundary.

### B-9: SessionDag Activation  [DONE s-59964; dag-write-in-dashboard deferred]
- Populate stub_ssb_seq at cli/main.rs:4061, 4151 (currently None everywhere →
  trim Pass 1 produces zero stubs).
- Wire SessionDag::create_handoff at 85% token budget threshold.
- Wire load_handoff for /continue (replace JSONL).
- Migrate fields from serde_json to rmp_serde for BLOB columns (msgpack per
  Phase 0.7b spec).
- Per-GRC-step / per-tool-call / per-phase-transition checkpoints.
- Branch UI in dashboard: /session fork|merge|close|list.
- SPEC AMENDMENT #2: SSB action_type='checkpoint' on merge_branch.
- ts-rs SessionHandoffEntry fix (B-3 dep); /continue ChatWindow handler;
  handoff threshold gauge in StatusBar.

### B-10: ContextBudget Profile Activation + 3E.5 file_overlap_score  [DONE s-59964]
- Stop passing `profile=None` from TaskRunner at 5 sites (1842, 6448, 6565, 6759,
  7513) — falls through to ContextBudget::compute(raw_max, [0,0,0], 4096) zeroed.
- Detect model tier at bootstrap (sidecar /v1/models, vLLM /v1/models,
  Ollama /api/tags, cloud config).
- Plumb ModelCapabilityProfile through all 5 task_runner sites.
- Implement compute_overlap(agent_files, ssb_entries) for SSB entry filtering;
  emit `ssb_filtered_by_overlap` SSB.
- Per-tier slot weights match token-efficiency-design.txt §2 (e.g. 7B-12GB:
  12.5/66.7/12.5/8.3 for RepoMap/CodeContext/SsbEntries/GrcHistory).
- TaskRunner-vs-REPL parity contract test.

---

## PART II — DASHBOARD VISUAL RESTORATION (B-11..B-14)  [V2 REBASELINED]

### B-11: Design System Restoration  [LARGELY DONE in V2]
- Re-scoped: verify V2 tokens match dashboard-design.txt; backfill drift.
- Token audit (bg-void #0a0e12, teal #1B7A6E, purple #7C5CBF, etc.);
  type-scale audit (10/12/13/15/18 only); animation timing audit per DESIGN.md
  §7 hover-reveal + §8 orbit modes.
- Playwright/Vitest visual regression test in
  `crates/dashboard-v2/tests/visual.spec.ts`.
- V2 already has JetBrains Mono via @fontsource-variable/jetbrains-mono;
  Lucide icons via @lucide/svelte; three.js + troika-three-text for orbit.

### B-12: Tier-0 Shell vs Workspace Decision  [DONE in V2]
- V2 ships `screens/Workspace.svelte` as canonical primary shell; Tier-0 Shell
  concept retired.
- Remaining: verify StatusBar lives inside Workspace; first-run wizard at
  `screens/ProjectSelector.svelte` shouldn't ask tier0/workspace; Sidebar
  audit (5 icons, 48px); drop `elai_cc.toml [dashboard] primary_shell` config.

### B-13: Activity Feed Restoration  [REPLACED by orbit; DONE s-59964 in re-scoped form]
- V2 ActivityWindow IS the orbit visualizer per DOMAIN-CONTEXT §2 + DESIGN.md §8;
  3 FSM-driven modes (Mode 1 PLAN constellation, Mode 2 MAP existing-project,
  Mode 3 RESEARCH FORMULATE→SEARCH→EXTRACT→REFLECT→CONVERGE with phi/psi twin
  micro-bars).
- Re-scoped: orbit captions wiring (lib/orbit/captions.ts); bubble-up event
  strip (NEW `lib/orbit/BubbleUpStrip.svelte`); signal layers (OrbitGrcDots,
  OrbitBudgetRing, OrbitEvidenceGates); idle effect OrbitShootingStar every
  ~14s deterministic via entropy-quarantined LCG; hover-reveal chrome per
  DESIGN.md §7.

### B-14: Live Token Streaming Chat  [DONE s-59964]
- Add Tauri `stream_chat(message)` in dashboard-v2/src-tauri/src/lib.rs.
- V2 chat lives in `lib/windows/CommandWindow.svelte` + `lib/layout/
  CommandPalette.svelte` (NOT ChatWindow.svelte / SlashPalette.svelte).
- Frontend SSE via `import { listen } from '@tauri-apps/api/event'` (Tauri 2).
- Intent classifier (deterministic + LLM fallback): TASK / FIX / EXPLORE /
  CONFIG / INTERVENE / STEER / ABORT.
- Card variants per DOMAIN-CONTEXT §2 COMMAND: status / intervention / error /
  disambiguation / steering / user. Outlined-button language only (no solid
  CTAs per DESIGN.md differentiator #6).
- Input placeholder per FSM state; Stop button cancels via cancel_stream.
- Svelte 5 runes only ($state for token buffer, $effect for SSE subscription
  cleanup) — no `$:` labels, no createEventDispatcher.

---

## PART III — HARNESS HARDENING (B-15..B-20)

### B-15: Phase 0 Debt Burndown  [PARTIAL — main.rs decomposition DONE 2026-05-08]
- 2026-05-08: cli/src/main.rs decomposed 10,570 → 252 LOC across 10 modules
  (args.rs 928, auth.rs 298, help.rs 216, live_cli.rs 3932, progress.rs 403,
  resume.rs 747, runtime_builder.rs 490, runtime_client.rs 753,
  tool_executor.rs 257, tool_formatting.rs 581). Tests moved to cli_tests.rs
  + sandbox_report_tests.rs. 178/180 pass.
- REMAINING:
  - Delete compact.rs (696 LOC), oauth.rs, remote.rs, prompt.rs.
  - Reduce usage.rs to spec minimum (delete ModelPricing/UsageCostEstimate/
    format_usd; B-27 reintroduces cost via CLOUD_SPEND).
  - Replace 84 std::collections::HashMap imports (BTreeMap / IndexMap /
    FxHashMap per usage); CI lint.
  - Replace 40+ SystemTime::now / Instant::now entropy quarantine leaks
    (inject ClockProvider).
  - Move file_ops/bash_validation/stale_branch from orchestrator/ to tools/.
  - Fix hooks.rs:634 Windows "sh" hardcoded.
  - Add 3 scripts: check_entropy_quarantine.sh, determinism_proof.sh,
    smoke_test_ollama.sh.
  - Generate `.elai_cc/metrics/pre_phase_0.toml`; CI workflow.

### B-16: Anti-Stale Rule Enforcement  [PLANNED]
- ort 2.x rc-track pin maintained (`=2.0.0-rc.12` today at knowledge/Cargo.toml:54 + orchestrator/Cargo.toml:68 optional). B-16 enforces the rc-track rule, NOT a stable-only rule. Re-evaluate when ort 2.0.0 stable releases. Rule amendment authored by E-34 (CLAUDE.md:18 + .elai_cc/outdated-tech-audit.md 2026-05 appendix); see workflow/reasoning/E-34-decisions.md.
- 15 Box<dyn Error> → thiserror enums (Bm25Error/VectorError/PagerankError).
- .unwrap/.expect removal outside tests/main/const-init.
- log:: → tracing::.
- Magic numbers → named constants.
- `as` narrowing casts → `.try_into()?`.
- Spec amendment: blake3 over blake2 (impl is correct per CLAUDE.md).
- CI anti-stale lint: Box<dyn .*Error>, .unwrap() outside test, use log::,
  `as i\d+`/`as u\d+`/`as f\d+`, std::collections::HashMap, SystemTime::now.

### B-17: Ground the Vacuous Gates  [PLANNED — meta-step over B-2/B-4/B-6/B-8]
- Regression suite `tests/contracts/gate_liveness.rs`: per-gate liveness test
  (GroundingSweepGate, SearchMode cascade, SupervisionManager, CSQ, inline
  feasibility, anti-rubber-stamp).
- Dashboard GatesPanel: per-gate verdict counts.
- CI gate-liveness check; decision doc for INTERACTIVE block-on-dismiss
  semantics; perf check <100ms per FSM transition with gates active.

### B-18: Tier C Sandbox Activation  [PLANNED]
- Linux: clone3 with CLONE_NEWNS|CLONE_NEWPID|CLONE_NEWNET; seccomp-bpf
  allowlist (IPC_BASELINE + NETWORK_EXTRA for researcher);
  prctl(PR_SET_NO_NEW_PRIVS); rlimits (60s soft CPU, 2GB mem, 256 files, 32
  procs); /proc hidepid=2; PR_CAPBSET_DROP.
- Windows: default enable_sandbox=true at 3 sites (dashboard/src/lib.rs:1183,
  task_runner.rs:2477, agent_pool.rs:153); wire self.sandbox.launch_agent();
  AppContainer profile per agent_id; restricted token + Job object fallback.
- IPC accept loop: UnixListener (Linux) / NamedPipeServer (Windows); route
  frames via IpcGateway; wire IpcHookDispatcher (currently dead).
- Visibility filter R31 compliance: replace `error_code="visibility_filtered"`
  with NotFound semantics (per-agent allowed_paths field on AgentHandle).
- 4-layer self-mod locks (IPC + filesystem + git + dispatch).
- Replace audit-flagged FAKE tests (`test_4a_isolation_pid_namespace` just
  asserts struct fields) with real /proc/{pid}/status NSpid reads.
- Real metrics in phase_4A.toml (NOT in-process postcard round-trip labeled
  IPC latency).

### B-19: Hook System Unification  [PLANNED]
- Decision: keep IpcHookDispatcher (after B-18 lands) — spec-compliant.
  Delete InProcessHookDispatcher (orchestrator/src/hook_dispatcher.rs).
- Migrate task_runner.rs hook call sites (3338, 3589, 3877, 3898) to IPC.
- Stop hook actually enforces non-IDLE block (currently telemetry-only).
- PreCompact wired to SessionDag::create_handoff (currently no-op stub at
  task_runner.rs:3898).
- Ruff PostToolUse subprocess on Python writes (5s timeout, content-hash cache).
- DAIRA: expand 2 → 15 markers (call/return/mutation/branch/error/finding/
  read/write/tool_call_start/tool_call_end/hook_pre/hook_post/timeout/escalate/
  converge).
- AdaptiveTool<T> wrapper; MarkerReport SSB.
- T1<1ms / T2<10ms / T3<5ms hook latency budgets enforced; emit
  `hook_latency_violation` on overrun.
- Auditor-discovered rules require independent approval before ACTIVE
  promotion (per C12).

### B-20: Tool Registry Unification  [PLANNED]
- Decision: keep IpcToolRegistry (after B-18). Delete InProcessToolDispatcher.
- Migrate FSM filtering + TestingFirst gate to IPC path.
- Failure taxonomy: 13 → spec 9 (SYNTAX_ERROR, TYPE_MISMATCH, UNDEFINED_SYMBOL,
  MISSING_IMPORT, LIFETIME_ERROR, BORROW_ERROR, UNUSED_WARNING, TEST_ASSERTION,
  RUNTIME_PANIC).
- SequenceMatcher similarity hint on edit failure (after 3-tier edit matching).
- File outline mode: force_full flag + 50% budget guard.
- IntegrationTestRunner success rows (currently only failures emit).
- WIP commits squashed after audit success (5+ edits → per-WIP commit; squash
  on AUDITING→MEMORIZING).

---

## PART IV — FINAL PRODUCT (B-21..B-28)

### B-21: Phase 4F Feasibility Probe  [PLANNED]
- SystemProbe (nvml-wrapper GPU VRAM + sysinfo RAM/disk); SystemConstraints
  pure-function check.
- Novel step detection (unverified PackageRef + unused framework + untested
  capability).
- <50-LOC stub harness in Tier C; 25s graceful + 30s hard kill (explicit
  child.kill().await — tokio kill_on_drop=false).
- AgentDispatchConfig overrides for feasibility role (GRC 3/600,
  tool_filter=[stub_write,bash], 25/30s timeouts).
- Skip iff zero novel steps AND all resources >= 20% margin.
- SSB branch-and-explore: max 3 alternatives per novel step; per concept (not
  all at once).
- 9 types (SystemProbeResult, GpuInfo, SystemConstraints, StubResult,
  ResourceDelta, ConstraintResult, FeasibilityReport, FeasibilityEscalation,
  FeasibilityDecision); FeasibilityProbe trait object-safe.
- 24+ tests (16 contract + 8 adversarial); dashboard FeasibilityPanel.

### B-22: Phase 4G Autonomous Mode  [DONE s-26716; LOC ~480 + patch-4G.1 ~90]
- AutonomousGate wraps InteractiveGate via DispatchPipeline / PlanDispatchTable;
  delegates to human only on HIGH risk / missing evidence / escalation trigger.
- 6 evidence checks (knowledge readiness, zero UNGROUNDED, tests fail correctly,
  test flipped, zero BLOCKs, DKG+changelog).
- 4 escalation triggers (anti-spiral 3x, HIGH risk, ReplanRequest(MAJOR),
  circuit breaker >10 auto-advances).
- SPLIT_TASK on context budget Terminal (max 4 sub-tasks, depth 2).
- SUBSTITUTE_MODEL on max_restarts via ModelPool swap chain; emit `model_swap`
  SSB.
- /mode + /model CLI commands; TreeAutoAdvancePolicy (BabyStep=LOW, Step=LOW,
  Phase=MEDIUM, Concept=HIGH; concept always human; 50-baby-steps-per-concept
  circuit breaker).
- Implementation: 18 extended contracts in phase_4g_extended_contract.rs;
  SlashCommand::Mode; derive_autonomous_evidence helper at task_runner.rs:3996
  (queries DkgReady/Finding/TestOutcome/PhaseTransition); SsbViewer::
  auto_advance_stats DTO; MonitorWindow Section 6 AutoAdvanceStats panel;
  AgentPool::get_role trait method.
- patch-4G.1: CommandPalette toggle fires stub.set_gate_mode IPC; StatusBar
  gate badge promoted from .verbose-hover-only to always-visible cluster;
  session.reconcileGateMode() on Workspace mount.
- Open follow-ups: /mode CLI live wiring (currently informational; needs IPC
  client or shared control file); Orbit per-decision viz (OrbitFixture not yet
  fed by GateDecision rows); LIMIT 1000 on query_by_type in evidence helper;
  SeededClock for deterministic last_24h_rate.
- FOLLOWUP B-22b (4G.9 ouroboros port): 4-pattern StagnationDetector
  (Spinning/Oscillation/NoDrift/DiminishingReturns) replacing lone "3x same
  step" rule; 5-persona LateralThinking (Hacker/Researcher/Simplifier/
  Architect/Contrarian) with affinity table; GenerationWatchdog (3 timeouts:
  idle 60s, no-material-progress 180s, safety 600s; workflow_fingerprint
  dedup); goal-drift Jaccard canary (0.5/0.7 verdict bands; cheap pre-audit
  Block saves audit LLM call); 5 persona agent files under .claude/agents/
  persona-*.md.

### B-23: Phase 4H Self-Modification (DGM/CEL/DarwinNet)  [PLANNED]
- 3 layers: L1 automatic metrics (no LLM), L1A automatic data-table growth
  (no LLM, no staged eval — additive rows in deprecated_patterns, FailureTaxonomy,
  lazy-code regex, source-tier; DarwinNet Gene Pool), L2 config proposals
  (DGM sigmoid parent selection lambda=10 alpha_0=0.5, archive max 50 variants
  per PipelineMode, staged eval 10/50/200, TestOutcomeReport 4-category
  SWE-bench grading PASS_TO_FAIL immediate-reject), L3 structural (HUMAN-ONLY
  stubs; emit `l3_proposal`).
- MSI exp-decay lambda=0.3 N=10 per PipelineMode (IMPROVING >0.3 continue,
  STABILIZING -0.1..0.3 reduce, DIVERGING ≤-0.1 pause+alert). Auto-revert on
  >10% degradation over 3 pipelines.
- AllowedConfigKey closed-set enum (locked: cloud budget, gate definitions,
  score weights — never modifiable).
- Safety: 4 enforcement layers (dispatch + IPC + filesystem + git). Self-mod
  agent cannot read lock list.
- Ouroboros ports (4H.7B-D):
  - OSC (4H.7B): ConfigVariantSimilarity 0.5*key_overlap + 0.3*type_match +
    0.2*exact_match. Stagnant >= 0.95 over 3 → exploration boost.
    Oscillating period-2 → PERIOD-2 FULL VERIFICATION freezes keys.
  - REG (4H.7C): RegressionDetector spans full PipelineOutcome history;
    blocks 4H.5 promotion + triggers 4H.6 auto-revert (3 consecutive
    regressed pipelines).
  - EVO (4H.7D): Evolution Gate — fake IMPROVING when MSI moves on noise but
    config_blob structurally identical. Withhold → MSI forced STABILIZING.
- 11 SSB action_types (7 base + convergence_stagnant + config_oscillation +
  regression_detected + evolution_withheld); 54+ tests.

### B-24: Phase 4I Media Validator  [DONE s-59964; 71 lib + 21 RCE + 19 dashboard + 2 e2e tests pass]
- Tier1 (in-process PNG/WAV/3D/UI parsers, <100ms): format/resolution via
  ImageMagick; SSIM/LPIPS via ONNX Runtime CPU EP (NOT GPU — non-deterministic
  per spec); 3D vertex/triangle via Blender CLI; audio sample rate via FFmpeg;
  OCR via Tesseract.
- Tier2 VLM advisory (GPU 2-5s): Mode A Qwen3.5-VL-30B-A3B via ollama
  (~17GB Q4); Mode B InternVL3.5-38B via SGLang (~38GB 8-bit). Constrained
  decoding for VlmValidationOutput JSON. Bounded concurrency (max 4 Tier 1,
  max 1 Tier 2).
- CLAP audio-text always-loaded (~0.4GB FP16) when audio active.
- Veto logic: Tier1 fail → Fail; Tier1 pass + Tier2 disagree → HumanReview;
  VLM unavailable → Degraded.
- RCE visual-feedback loop: execute → render → VLM evaluate → structured
  feedback → executor iterates. Convergence stop when issues unchanged.
- 5 MediaRenderAdapter (Playwright web, Blender 3D, FFmpeg animation, direct
  read screenshots, ComfyUI AI-gen).
- Lazy module loading (instantiated only when PipelineSpec has MediaSpec).
- Implementation: Tier1+Tier2 (Mock/Ollama/SGLang VLM advisory) + RCE
  feature media-validator-rce + lazy gate wire_media_validator_for_spec +
  SsbMediaSink (Arc<Ssb>-backed production sink) + task_runner integration
  at L3658 post spec_from_decomposition + dashboard-v2 MediaValidationPanel
  (4 Tauri cmds + 6 SSB action_types).
- cloud_mode_live #[ignore]'d (claude-sonnet-4-5-20250929).

### B-25: Phase 4J Empire Integration  [DEFERRED — conditional]
- FactoryLine protocol client (pure Rust, newline-JSON, UnixTransport Linux +
  TcpTransport Windows, generic Transport trait).
- Brain registration with 5s heartbeat + reconnection + re-registration.
- Replace broken brainctl CLI (zero Python deps).
- Empire event subscription with SSB bridge (rate-limited 100/s default,
  filter allowlist, LOW/MEDIUM/HIGH priority; events become GRC
  SOURCE_CHECK grounding source).
- ElaiNeuron MCP server: 4 capabilities (task_submit, task_status,
  pipeline_result, health_check). ToolFilter enforced; zero internal state
  leaked.
- Empire repair pipeline: detect 3 verified issues (CircularDep / HardImport /
  StdlibShadow) → generate PipelineSpec → TestOutcomeReport validate.
- `cfg(feature="empire")` gating; graceful degradation.

### B-26: Phase 4K.6-4K.13 Dashboard Completion (V2 final pass)  [DONE s-59964; 6 remaining tasks polish]
- Already shipped in V2: MonitorWindow (live polling), ExploreWindow,
  ReviewWindow, NotificationManager (Browser API + localStorage),
  InterventionRequest/Response SSB types, submit_intervention_response +
  set_webhook_url + get_webhook_url Tauri commands, openTab/closeTab/moveTab
  workspace.svelte.ts with drag-tabs-between-panels, schema v1 localStorage
  persistence, Ctrl+1..7/N/W/D/.,/Shift+P/Esc keyboard shortcuts, FSM-state
  Notification triggers, client-side session recovery.
- DECIDED 2026-05-11: D1=(a) DROP WINDOW_STATE SSB (use localStorage only;
  no cross-device sync); T4 DROP intervention card extraction (inline
  orbit-card in MonitorWindow.svelte:376-409 — premature abstraction).
- Remaining 6 tasks: webhook firing (POST Discord/Slack-shape on
  INTERVENTION_REQUIRED/AUDITING/IDLE-after-task and pending_interventions
  becoming non-empty); SettingsWindow webhook URL input;
  REVIEW window-scoped Ctrl+Shift+A accept-all / Ctrl+Shift+R reject-all
  (active-window guard in shortcuts.ts); remove misleading Ctrl+Shift+H
  (currently maps to openCommandPalette but description says "Recover closed
  window"); phase-dot model real plan progression (replace static 5-phase
  map at MonitorWindow.svelte:80-110 — new get_plan_progress Tauri command
  OR extend FsmSnapshot); panel-transition animation backfill (fade-out
  80ms / fade-in 120ms on Panel.svelte tab switches per DESIGN.md §8);
  responsive breakpoints (<1000px sidebar→hamburger; <800px secondary
  collapse).

### B-27: Cloud Mode (D-1..D-4)  [PLANNED]
- 3 modes per-task: LOCAL (default, 6 reasoning layers, $0), CLOUD (cloud
  generates, local L1 deterministic still runs, L2-5 skipped), HYBRID (local
  first, auto-escalate to cloud on L3+ failure for that step only).
- D-1: cloud config (`[cloud] provider = "anthropic" model =
  "claude-sonnet-4-6" escalation_model = "claude-opus-4-6" max_spend_per_task
  max_spend_per_day warn_at_percent`); CLOUD_SPEND SSB; ceiling enforcement;
  /task --mode flag.
- D-2: `[context_budget.cloud]` 200K profile (vs 8-25K local); baby step
  300-500 LOC vs 25-40; decomposition depth 2 vs 4; phase split threshold
  50K vs 3K; reasoning layer collapse.
- D-3: privacy redaction (redact_env_vars default, redact_patterns regex,
  blocked_paths globs); audit trail per cloud call; BYOE (custom_endpoint URL,
  OpenAI-compatible); air-gapped mode (`[execution] air_gapped = true`).
- D-4: ESCALATE_TO_CLOUD SSB; pre-task cost estimation; per-task spend gauge
  in MONITOR; warn at 80% / 100%; CloudPanel.

### B-28: Focus Tunnel (E-1..E-7)  [PLANNED]
- 3 modes deterministic selection:
  - FIM (Qwen2.5-Coder native fill-in-middle; no tool schemas, ~350 tokens/step
    saved; single-region edits).
  - DIFF (unified diff, ~30-50% smaller than full file; multi-region;
    3-tier edit matching via B-20).
  - STRUCTURED (current default; fallback for new files + cloud).
- ModelCapabilities.supports_fim (Qwen2.5-Coder/StarCoder/DeepSeek-Coder = true,
  cloud = false).
- ModeSelector rules per spec.
- Streaming validation framework:
  - Token-level: import validation (deprecated_patterns), lazy-code detection
    (todo!/pass), early termination.
  - Chunk-level: tree-sitter partial parse every ~50 tokens.
  - Saves ~90% generation time on bad gens caught at token 50 vs 500.
- Pipeline parallelism (depth 1): pre-assemble context for step N+1 while N
  validates. Adaptive disable if validation pass rate <80%.
- Automated dependency resolution: detect missing imports → anti-staleness
  check → registry → conflict check → manifest add → install. Model NEVER
  runs cargo add / pip install. Pre-approved allowlist for autonomous mode.
- Speculative file routing: plan specifies target_file; AST mismatch → micro-
  step create file → retry (max 2 micro-steps per baby step).
- Token budget impact: ~6,800 tokens saved on 30-step task = 27% of 25K;
  combined with ShortCoder = 45-65% effective budget increase.
- 4 SSB action_types: generation_mode_selected, streaming_early_terminated,
  pipeline_pre_assembled, dependency_resolved.

---

## PART V — KNOWLEDGE ENGINE EXTENSIONS (B-29..B-35)  [SocratiCode-derived, Rust reimpl, AGPL source NOT copied]

### B-29: Symbol-Level Impact Analysis API (★ highest leverage)  [PLANNED]
- 4 agent-callable tools backed by sharded-index, LRU-bounded SymbolGraphCache:
  - compute_impact_radius (BFS reverse-call, hop-banded, depth-clamp 10)
  - compute_call_flow (DFS with cycle detection, lazy children, depth-clamp 10)
  - compute_symbol_context (definition + Vec<Caller> + Vec<Callee> with confidence)
  - list_symbols (file/query/limit filter)
- SymbolGraphCache: 27 name shards (a-z + `_`) + 256 reverse shards
  (single-byte SHA1 prefix) + 500-file payload LRU. Critical: no query loads
  every symbol or edge.
- 4 Confidence tiers per cross-file resolver: Local / Imported (1-hop) /
  Re-export (2-hop) / Unresolved.
- Schema v3: symbol_name_index, symbol_reverse_index, symbol_file_payload,
  symbol_graph_meta tables (msgpack BLOBs).
- 4 dispatcher tool entries (symbol_impact / symbol_flow / symbol_context /
  symbol_list).
- Auditor PostToolUse auto-injects IMPACT_SUMMARY (depth=2) for changed files.
- Cache invalidation: <50 changed files = incremental, >50 = full rebuild.
- Academic anchor: FocalAD (arXiv:2506.11419) validates top-k local-focus
  > global aggregation for downstream decision-making.
- Std-only LRU (no `lru` crate); CLAUDE.md anti-dep policy.

### B-29.5: Symbol Graph Scale Benchmark  [PLANNED — sub-step verification]
- Real benchmarks against 4 pinned-commit fixtures: Linux kernel v6.7 (~36M
  LOC), Chromium 121.0.6167.184 (~35M LOC), VS Code 1.94.2 (~2.45M LOC),
  Postgres REL_17_0 (~1.5M LOC).
- Verify: p99 ≤ 500ms across 4 traversal tools, cache LRU ≤ 2GB peak RSS on
  1000-query workload, hop-band determinism across 5 runs (BTreeMap byte-identical
  output), per-language unresolved_edge_pct documented.
- Criterion harness + scale_memory.rs (RSS via procfs Linux / GetProcessMemoryInfo
  Windows) + workload.rs (60% impact_radius / 25% symbol_context / 10%
  call_flow / 5% list_symbols).
- 3 CI contract tests (scale_p99_budget, scale_resume_identity,
  scale_long_tail_memory).
- Sets precedent: knowledge-engine changes require scale benchmark re-run.

### B-30: Multi-Language DKG Hardening (closes A-35)  [PLANNED]
- 13+ tree-sitter grammars with per-grammar try/catch isolation
  (std::panic::catch_unwind around tree_sitter::Parser::set_language).
- Per-language extractors: Python / TypeScript / TSX / JavaScript / Go /
  Java / Kotlin / Scala / C / C++ / C# / Ruby / PHP / Swift / Bash / Svelte / Vue.
- Multi-module resolvers:
  - JVM suffix map (Maven/Gradle layout `/src/main/<lang>/` → FQN)
  - C# namespace map (block + file-scoped C#10+; lexicographic ordering for
    deterministic candidates)
  - Go module info (parse go.mod `module path/here`)
  - TS path aliases (tsconfig.json + jsconfig.json `compilerOptions.paths`
    with `extends` chain + cycle prevention)
- Svelte/Vue composite-language re-parse (HTML tree-sitter finds
  `script_element`, re-parse body as TypeScript; symbols qualified
  `<file>.svelte::<symbol>`).
- CSS `@import` extraction from `<style>` blocks.
- LanguageRegistry with `loaded: BTreeSet<String>` and `failed: BTreeMap<String, String>`.
- SSB `LanguageGrammarStatus` action published at startup.
- Tree-sitter retained per CLAUDE.md anti-stale (no ast-grep swap).

### B-31: Resumable Indexing + Cross-Process Lock + Startup Auto-Resume  [PLANNED]
- Per-batch checkpoint (50-file batches): file_hashes map (blake3 — anti-stale
  rule) with content-hash skip on resume. Cancellation flag honored at every
  batch boundary.
- Schema v2: project_metadata + file_hashes tables.
- IndexLock (fs2 crate IFF already workspace; else hand-rolled O_EXCL+PID).
  LockScope::Index | Watch. PID liveness check (kill(pid,0) POSIX /
  OpenProcess Windows). Stale = mtime > 2min; refresh every 30s. RAII Drop.
- `auto_resume_indexed_projects()` called from BOTH cli main and dashboard
  run; per-project tokio task; fire-and-forget; status in-progress → resume,
  completed → start watcher, cancelled → leave.
- `graceful_shutdown(signal)`: 60s deadline; cancels all in-progress; releases
  watcher locks; force-release lock files after deadline.
- 5 new SSB action_types: IndexCheckpoint, IndexResume, IndexCancel,
  LockAcquired, LockContested.
- IndexingProgressPanel with live IndexCheckpoint rows + Resume/Cancel banners.
- migrateAbsolutePathKeys one-shot migration after B-1 path unification.

### B-31.5: Resumable Indexing Scale Benchmark  [PLANNED]
- 5 tiers on B-29.5 fixtures:
  - Tier A: full clean baseline (4 tiers, ~3-4h wall)
  - Tier B: kill-resume identity at 10/50/90% (byte-identical post-resume
    state via `diff` of sqlite ".dump"; ≥6 runs)
  - Tier C: cross-process race (2 instances, instance B exits <1s with
    "Another process is indexing" + holder PID) + stale-PID reclaim <5s
  - Tier D: graceful shutdown under load (SIGTERM, deadline ≤60s; forced
    release after deadline via synthetic ELAI_DEBUG_BATCH_DELAY_SECS)
  - Tier E: Chromium long-tail (4h wall, RSS plateau check — no monotonic
    growth = no memory leak)
- 3 CI contract tests; sets precedent.

### B-32: Adaptive Chunking — Minified File Detection + Universal Char Cap  [PLANNED]
- Pre-strategy detection: `avg_line_length = len/max(line_count,1) > 500` →
  character-based chunker. No AST attempt.
- Universal MAX_CHUNK_CHARS=2000 cap applied as last step to ALL chunkers
  (AST + line-window + character) regardless of strategy. Defense-in-depth.
- chunk_by_characters: backwards-scan safe-boundary (\n/space/tab/`;`/`,`).
  Chunk ID = byte offset (not line number; minified files often one line).
- UTF-8 char-boundary safe (floor_char_boundary).
- SSB MinifiedFileChunked.
- Fixture project: vendor/bundle.min.js 50K-char single line, bootstrap.min.css
  150K-char, *.pb.go multi-thousand strings, UTF-8 emoji variant.

### B-33: Interactive Decomposition + Symbol Graph Viewer (Mermaid + cytoscape)  [PLANNED]
- 3 modes: File / Symbol / Decomposition.
- Cytoscape from npm (cytoscape + dagre + fcose) — USER CONFIRMATION needed
  per CLAUDE.md no-new-deps.
- Decision: Option A (new GraphWindow alongside 7 V2 windows, full flexibility),
  Option B (reuse orbit Mode 2 MAP, lightest), Option C (drawer-only in
  KnowledgeDrawer, smallest blast). Recommend B or C unless user wants
  right-click contextual flyout.
- Sidebar interactions: click=details, right-click=blast-radius (red gradient
  by hop depth), search filter, 6 layouts (dagre/fcose/concentric/breadth-first/
  grid/circle), PNG export, Mermaid export.
- Mermaid text export (graph-analysis.ts:113-210 algo): language-coded
  classDef colors (rust teal #1B7A6E, python #3776AB, ts/js #F7DF1E, svelte
  #FF3E00, go #00ADD8, JVM purple #7C5CBF, csharp #239120), cyclic edges in
  red dotted (`-.->|cycle|`); deterministic; <250 nodes legible.
- CLI `/graph mermaid` slash command (paste-ready stdout).
- Size caps: MAX_VIZ_SYMBOLS=20_000, MAX_VIZ_EDGES=60_000,
  MAX_VIZ_SYMS_PER_FILE=200. SymbolMode = Full | Capped | Omitted.
- 3 Tauri commands (get_graph_view_data, get_impact_overlay,
  export_graph_mermaid).

### B-34: Entry-Point Detection Tool  [PLANNED]
- 4-heuristic union over SymbolGraphCache:
  - Orphan (file with no dependents AND has outgoing calls)
  - WellKnownName (main/Main/__main__ per language; 14 languages
    case-sensitive)
  - FrameworkRoute (callee_name matches get/post/put/delete/patch/head/
    options/all/use → "framework:http-X"; route → "framework:route")
  - Test (^test[_A-Z] / ^Test[A-Z_] / exactly `it` or `describe`)
- Dedup by (id, reason) — same symbol can be Orphan + WellKnownName but not
  twice with same reason.
- Performance: <500ms p99 on 10K-file project via payload selectivity (orphan
  files + conventional filenames only — <10% of files payload-loaded).
- Auto-inject for Designer ExistingProject decomposition (EXISTING_ENTRY_POINTS
  briefing block) + Researcher /research (PROJECT_STARTING_POINTS seed).
- ENTRY_POINT_NAMES configurable via `[entry_points]` block in elai_cc.toml
  (append-only override with built-ins).

### B-35: Context Artifacts Pipeline (full second indexing pipeline)  [PLANNED]
- `[[knowledge.artifacts]]` blocks in elai_cc.toml: name + path (file or dir)
  + description (written as "check this before X").
- Per-project context_* vector index collection (separate from code DKG).
- Reuses B-32 chunker for non-code text; SQL chunked at CREATE TABLE/VIEW/
  INDEX regex boundaries.
- blake3 staleness; idempotent ProjectMetadataStore.
- ArtifactStage in QueryPipeline (STANDARD + DEEP tiers; FAST = code-only).
- GroundingMethod::ArtifactCheck (7th method): dispatches search_artifacts
  with artifact_name filter; Verified if any result score > 0.5;
  MalformedGrounding if artifact_name not declared.
- 4 dispatcher tools (list/index/search/remove_artifacts).
- 3 SSB action_types (ArtifactsIndexed, ArtifactSearched, ArtifactStale).
- AVAILABLE_ARTIFACTS auto-injection for Designer ExistingProject.
- Security: paths resolving outside project_root rejected at manifest load.
- Failure non-fatal: artifact errors never block code indexing.

---

## PART VI — DASHBOARD-V2 SRC-TAURI WIRING (B-36)

### B-36: Track D V2 src-tauri Wiring (D.1..D.13 + 15 net-new surfaces)  [PARTIAL→DONE via 2026-05-07 followup]
- Multi-session: planned as 4 sessions per workflow/sessions/s-46448/
  HANDOFF-track-d-v2-rebaseline.md.
- Session 1 REBASELINE (analysis-only): docs/specs/track-d-v2-spec.md +
  B-plan-delta-2026-05-07.md.
- Session 2: engine wiring (add elai-orchestrator/elai-cli/elai-core/
  elai-providers to src-tauri/Cargo.toml — USER CONFIRMATION needed),
  AppState struct, D.1 sidecar_health (GET http://127.0.0.1:8100/health),
  D.13 get_sidecar_settings/set_sidecar_settings/restart_sidecar (SSH to
  remote box — NEVER local sidecar per HARDWARE-HEAVY rule), DASHBOARD_BINARY_NAME
  fix.
- Session 3 (research/planning): D.2 run_planning_task (web_research_step
  emit), D.3 run_research (returns Track A.2 ResearchAnswer), D.7
  one_shot_prompt (subprocess with ANTHROPIC_BASE_URL/ANTHROPIC_API_KEY/
  ELAI_LOCAL_MIN_PROMPT/ELAI_LOCAL_MAX_CONTEXT env).
- Session 4 (settings/observability + 15 net-new): D.4 settings + obscura_test_fetch,
  D.5 run_d5_live_contract, D.6 reindex_now, D.8 tool_visibility, D.9
  set_local_max_context, D.10 set_local_min_prompt, D.11 sidecar_search,
  D.12 grounding_cache_stats (live SSB upgrade).
- 15 net-new V2-only surfaces: recompute_steering_vector (A-8 freshness),
  get_steering_status, get_mcp_status (batch-ful-d MCP bridge), get_gg_signals
  (A-17 sidecar signals — mean_logprob/min_logprob/entropy_mean/entropy_max/
  generation_novelty), get_dkg_stats, get_sidecar_status, get_bubble_up_events,
  get_hook_executions, get_shortcoder_stats, run_explore_query, continue_task,
  get_grc_budgets/get_evidence_gates/get_context_budget/get_search_mode (real
  post-B-4/5/6/10).
- Implementation: 9/13 D-cmds wired in s-59964; D.2/D.3/D.5 done in followup;
  D.4 obscura via subprocess fallback. ~637 LOC across 4 files.
- Anti-patterns: do NOT back-port V1 commands.rs verbatim (only D.13 maps
  cleanly); do NOT touch orbit visual; do NOT boot local sidecar; do NOT
  introduce new frontend deps; do NOT commit without user request.

---

## PART VII — OUROBOROS-DERIVED EXTENSIONS (B-37, B-38)  [MIT-licensed source]

### B-37: Phase 4M — Multi-Runtime Backend Abstraction  [SHIPPED commit fd5c2e5]
- Single RuntimeAdapter trait covering 11 backends: Sidecar + AnthropicApi
  (existing, ported) + 7 ouroboros CLI ports (ClaudeCode / CodexCli / OpenCode /
  Hermes / GeminiCli / KiroCli / CopilotCli) + 2 native HTTP (Ollama / Vllm).
- ALL CLI subprocesses run inside Tier C isolation; stdout sanitized via 4A
  IPC sanitizer before reaching orchestrator prompt context.
- resolve_backend(candidate) with full supported-list error on miss.
- create_runtime(cfg) factory.
- PerRoleRuntime: `[agents.<role>.runtime] backend = "..."`. Default mapping:
  designer→AnthropicApi, executor→Sidecar, auditor→AnthropicApi,
  memory_writer/context_compiler/prompt_engineer→Sidecar,
  researcher→AnthropicApi, media_validator→AnthropicApi.
- Runtime fallback chain orthogonal to 4G SUBSTITUTE_MODEL. On Unavailable:
  traverse fallback_chain; emit `runtime_fallback` SSB; on exhausted chain
  → ESCALATE (INTERACTIVE) or HALT (AUTONOMOUS).
- /runtime show|set|fallback CLI command.
- RuntimePanel in V2 SettingsWindow with live health green/yellow/red.
- PermissionMode default AcceptEdits, NEVER DangerouslySkipPermissions
  without explicit opt-in. stdout_idle_timeout enforced (kill subprocess on
  stuck output).
- 4 SSB action_types: runtime_fallback, runtime_health_changed,
  cli_subprocess_spawned, cli_subprocess_killed.
- Migration: `[backend] mode = "local"` → `[runtime] backend = "sidecar"`
  deprecation alias for one release.
- Live cloud Sonnet 4.6 PROVEN.

### B-38: Ambiguity-Score Continuous Metric  [PLANNED — small ouroboros retrofit]
- Extends B-17 GroundingSweepGate with continuous score
  (`ambiguity = 1 − Σ clarity_i × weight_i`, threshold ≤ 0.2).
- 4 dimensions per ouroboros ambiguity.py:
  - Goal Clarity (40% greenfield / 35% brownfield)
  - Constraint Clarity (30% / 25%)
  - Success Criteria Clarity (30% / 25%)
  - Context Clarity (— / 15% brownfield)
- Per-dimension floors: Goal ≥ 0.75, Constraint ≥ 0.65, Success ≥ 0.70,
  Context (brownfield) ≥ 0.60. Floors block "aggregate passes but one
  dimension dangerously low" cases.
- Auto-complete streak: 2 consecutive ready snapshots required (single noisy
  snapshot can't pass).
- Streak counter persisted in SessionState (resets on plan revision).
- LLM temperature=0.1 for reproducibility (matches ouroboros
  SCORING_TEMPERATURE).
- Adaptive token allocation: starts 2048, doubles on truncation up to 5
  retries, max 5 format-error retries.
- 3 SSB action_types: ambiguity_scored, ambiguity_floor_failed,
  ambiguity_streak_advanced.
- Verdict: ungrounded > 0 → BLOCK (existing). Else if ambiguity > 0.2 OR any
  floor_failure → BLOCK. Else if streak < 2 → WAIT. Else → ALLOW.

---

## PART VIII — CHAT REFLECT + IDEA-FRAMING + PER-LEVEL AUDITS + COSMOS (B-39..B-42)

### B-39: Chat REFLECT Pipeline  [DONE s-30584; B-39C answer-substitution DONE s-34648]
- Per-message 4-stage reasoning pipeline wrapping LiveCli::run_turn at
  live_cli.rs:1685: TRIAGE → REASONER (N=4 best-of-N) → AUDITOR (REFINE
  cap=2 per RefineBench arXiv:2511.22173) → SYNTHESIZE.
- Opt-in via /reason on|off (slash count 146 → 147).
- Per-stage KV reset = separate /v1/messages calls (Agentless pattern,
  arXiv:2407.01489).
- N=4 candidates per Liu arXiv:2502.06703 inflection (3B-7B class), documented
  vs N=16 elbow.
- ChatInferenceClient + ReflectEventSink traits in reasoning/ (avoids circular
  Cargo deps); SidecarChatClient adapter in cli/live_cli.rs:4032 bridges
  SidecarProvider.
- 4 new SSB action_types: ReflectTriaged / ReflectCandidate / ReflectAuditVerdict /
  ReflectRefined; SCHEMA 28 → 29.
- ssb_viewer.get_reflect_state + Tauri command + ReasoningDrawer 7th panel
  (Svelte 5 runes only).
- Pipeline never panics — falls back to passthrough on any error.
- Budget guard: aborts remaining candidates at 85% of budget_cap_tokens
  (default 20K, leaves ≥5K headroom of 25K hard ceiling).
- 2 new files (chat_reflect.rs 530 LOC, chat_reflect_contract.rs 491 LOC),
  18 modified. Commit 6dd4af5.
- B-39C (s-34648): un-underscored ChatReflectPipeline.run output capture; when
  !output.skipped AUDITOR-selected reply renders via TerminalRenderer::
  render_markdown and runtime.run_turn bypassed entirely (D2a text-only, no
  tool execution); when skipped=true falls through. 3 thin delegates on
  ConversationRuntime (push_user_text, push_assistant_text,
  record_pipeline_usage). Commit 6adb454.
- Deferred: B-39B AISLE/Mythos jagged-frontier eval (needs 3090 box),
  B-39D REFLECT progress UI streaming, B-39E D2f system-hint mode (run_turn
  tool loop with reasoning_summary injected as system hint — restores tool
  calls under /reason on at cost of one extra model call/turn), B-39F
  session-length compaction guard.

### B-40: Idea-Framer + Domain Research Presets + Critic Loop  [SHIPPED]
- Inserts FRAMING substate before Phase 4D Recursive Decomposition; classifies
  product idea (ConsumerMobileApp / WebApp / SaaSDashboard / DesktopApp /
  GameMod / ResearchTool / LibraryOrSDK / CLITool / Other) via deterministic
  regex classifier first; LLM fallback only on Other.
- `[research_presets.<class>]` TOML schema: ordered ResearchStream with
  name + query_template (Tera `{idea}`/`{class}`) + min_source_tier (T1/T2)
  + parallelism_budget + cosmos_grounding + mandatory flag.
- Multi-root FlashResearch dispatch: ResearchTree::new_multi_root(streams).
  ALL mandatory streams must converge OR explicit degraded_streams emitted.
- 10 critic categories: STALE / SLOP / POLICY / FEASIBILITY / MARKET / SCOPE /
  CONTRADICTION / DESIGN_HARD / VAGUE / MISSING.
- Critic loop: max 3 rounds (RefineBench cap). BLOCK on any unresolved C-*
  finding at final round → emit `idea_framing_block` SSB; FRAMING substate
  refuses to advance.
- 5 SSB action_types: idea_framing_started, project_class_classified,
  critic_round_complete, idea_framed, tradeoff_card_emitted.
- types at orchestrator::idea_framing (FramedIdea, TradeoffCard, CriticFinding,
  CriticCategory enum, ProjectClass enum, classify_project_class).
- FsmSubstate::Framing.
- AutoAdvanceEvaluator 7th evidence check (derive_framing_evidence pattern
  mirroring B-22's derive_autonomous_evidence at task_runner.rs:3996).
- IdeaFramingWindow.svelte (pre-Workspace screen) + MonitorWindow "Critic
  rounds N/3" badge.
- Artifact: workflow/sessions/{sid}/idea-framing/{framed-idea.toml,
  critic-rounds/round-N.toml, tradeoff-responses.toml}. Re-used by Designer
  Concepts decomposition as ground-by-default artifact (B-35).
- Papers: arXiv:2510.05145 FlashResearch (tree-structured + parallel sub-Q +
  dual-threshold convergence), arXiv:2511.22173 RefineBench (REFINE
  diminishing returns past iteration 2), arXiv:2409.04109 Si et al.
  (substituted for bad arXiv:2407.21787 cite — paper-citation-verify rule
  enforced).
- SCHEMA_VERSION 41 (pin 42 vs B-34); IPC v2 unchanged. 79/79 PASS.

### B-41: Per-Level Auditor Modes  [SHIPPED]
- 6-mode per-level auditor replacing single-mode end-of-implementation auditor:
  - mode=Concepts (after Designer produces Concepts — market fit, IP,
    scope, contradictions vs framed idea; GRC 6/1000)
  - mode=Phases (after each Concept → Phases; ordering, contract sufficiency,
    cycles; GRC 5/800)
  - mode=Steps (after each Phase → Steps; granularity, test coverage; GRC
    4/700)
  - mode=BabySteps (after each Step → BabySteps; 8-block completeness:
    GROUNDING/PRE_GATE/TASK/TDD/DETERMINISTIC_CHECK/ACCEPTANCE/INTEGRATION/
    EXIT_CRITERIA; GRC 4/700)
  - mode=BabyStepDetail (after batch ≤4 BabySteps authored; line-level
    lazy-code/unwrap/hardcoded-secret/API-version-stale; GRC 3/600)
  - mode=Plan (existing single end-of-task auditor; GRC 10/1200)
- Visual reserved variant (cfg(feature="visual_audit")) for future B-43.
- BLOCK policy: respawn level-N decomposer ONCE with rescope_from_audit
  hint, NO re-audit. If second tier audit ALSO BLOCKs → cascade via
  BubbleUpHandler to parent tier (real fault is higher up).
- Rule cluster (fixed load order): anti-stale → coding-standards → design-hard
  → app-store → source-tiers → slopsquatting → risk-levels.
- 5 new rule domains seeded under crates/rules/seed/ (design_hard.toml ~25,
  app_store.toml ~15, source_tiers.toml ~10, slopsquatting.toml ~30 patterns,
  risk_levels.toml ~12).
- Parametric `AgentRole::Auditor(AuditorMode)` (not 6 separate variants).
- AutoAdvanceEvaluator extends with 5 per-level checks (concept_zero_blocks,
  phase_zero_blocks, step_zero_blocks, babystep_zero_blocks, plan_zero_blocks).
- 3 SSB action_types: per_level_audit_started, per_level_audit_verdict,
  per_level_audit_cascade.
- types at auditor_mode + per_level_audit_dispatch + AgentRole +5 +
  Evidence +5 + AuditTimelineWindow.svelte (NEW).
- Artifact: workflow/sessions/{sid}/audits/{mode}/{node_id}.md.
- 80 GREEN tests; SCHEMA 45; navigator-spawn deferred.
- Paper: arXiv:2404.03602 Kamoi COLM (prompt-scope recall 17-37pp); per-level
  scoping = design hypothesis. Substituted arXiv:2406.01297 (Kamoi TACL bad
  cite).

### B-42: Cosmos Artifact Tier  [SHIPPED]
- Extends B-35 per-project artifact pipeline with global "cosmos" tier at
  `~/.elai/cosmos/` (user-scoped, cross-project — NOT in any project repo).
- Storage: manifest.toml + artifacts/*.md+*.toml (blake3 hash + refresh
  metadata) + index/cosmos_global.bm25 + cosmos_global.vec.
- Manifest entry per artifact: name + project_classes (which B-40 classes
  consume) + refresh_cadence (daily/weekly/monthly/quarterly) +
  min_source_tier (T1 default) + research_query template + max_size_bytes +
  tier_relevance map (concepts/phases/steps/babysteps relevance scores).
- CosmosWorker.tick() on Layer 1A schedule (default hourly; due-detection
  per cadence). Per due artifact: build FlashResearch query → await
  convergence → write Markdown + blake3. blake3 unchanged → skip (no SSB
  churn).
- Time-decay: artifacts older than 2× refresh_cadence → DECAYED status →
  SHADOW → caller WARN'd → re-prioritize refresh queue.
- Lazy cosmos_get(name) → 30s blocking refresh if stale; fall back to stale
  artifact + WARN if timeout.
- CosmosStage in QueryPipeline between ArtifactStage and ARagStage; same
  hybrid (vector + BM25 RRF).
- GroundingMethod::CosmosCheck (8th method) — dispatched for design/
  ip_boundary/monetization/app_store/marketing_pattern/ux_convention claim
  categories.
- compile_cosmos_briefing(project_class, tier_relevance_threshold) → ≤1500
  token AVAILABLE_COSMOS block; ShortCoder fallback or skip on overflow.
  Designer threshold=0.5, idea-framer=0.5, mode=concepts=0.4, mode=phases=0.6,
  mode=babysteps=skipped.
- Default ≥12 artifacts shipped at first install: design_trends_2026 (mobile
  +web), ios_app_store_guidelines, android_play_store_guidelines,
  web_app_ux_patterns_2026, saas_dashboard_patterns, copyright_tradedress_guide,
  asset_licensing_quickref, monetization_patterns_consumer_apps + saas,
  color_palette_trends_2026, typography_trends_2026, accessibility_wcag_aa_quickref.
- Lock interaction: shared B-31 LockScope::Index (embedding rate-limit shared);
  must NOT run during AUDITING/EXECUTING.
- 4 SSB action_types: cosmos_refresh_started, cosmos_artifact_updated,
  cosmos_artifact_decayed, cosmos_grounding_used.
- CosmosDrawer.svelte + MonitorWindow "cosmos N fresh / Y stale / Z decayed"
  mini-badge during PLANNING / FRAMING.
- Implementation: cosmos_worker.rs (~700 LOC) + briefing compiler +
  VerificationMethod::CosmosCheck + SCHEMA 44 + CosmosDrawer; spawn_loop
  dispatcher deferred.
- Papers: arXiv:2403.12958 Dated Data, arXiv:2511.14022 Code-Aware LLM
  Freshness; substituted bad arXiv:2305.18290 (DPO) cite.

---

## Cross-cutting summary

### Schema bumps (SsbActionType)
| B-N | bump | added action_types |
|---|---|---|
| B-2 | — | "phase_transition" (string only — already in enum) |
| B-4 | — | knowledge_degradation, hyde_query, query_trace, circuit_breaker_state_change, vector_cache_hit_rate |
| B-5 | — | grc_step_emitted, grounding_sweep_result, csq_evaluated, feasibility_check, auditor_rubber_stamp_rejected |
| B-6 | 25→26 (s-59964) | evidence_gate_evaluated, intervention_request, intervention_response, supervision_event, anti_spiral_hit, bsp_superstep, context_reduction, ssb_filtered_by_overlap, agent_status_changed |
| B-7 | included in 26 | concept_decomposition, phase_node_ready, step_node_ready, baby_step_ready, dag_repair_round, dag_repair_escalated, phase_sized_split, tree_position_changed, failure_recorded, bubble_up_typed, integration_checkpoint_success, research_node_stage, package_ref_verified, tech_stack_decided |
| B-9 | — | "checkpoint" on merge_branch (SPEC AMENDMENT #2) |
| B-22 | — (s-26716) | auto_advance_decision, model_swap, autonomy_escalation, split_task_initiated |
| B-23 | (per session) | 11 self-mod: self_mod_proposed/evaluated/promoted/reverted, msi_state_changed, data_table_grew, l3_proposal + OSC/REG/EVO (convergence_stagnant, config_oscillation, regression_detected, evolution_withheld) |
| B-24 | (s-59964) | 6 media variants |
| B-26 | D1=(a) DROPPED WindowState/HealthSnapshot |
| B-27 | — | CLOUD_SPEND, ESCALATE_TO_CLOUD |
| B-28 | — | generation_mode_selected, streaming_early_terminated, pipeline_pre_assembled, dependency_resolved |
| B-29 | v3 dkg schema | SymbolQuery |
| B-30 | — | LanguageGrammarStatus |
| B-31 | v2 metadata | IndexCheckpoint, IndexResume, IndexCancel, LockAcquired, LockContested |
| B-32 | — | MinifiedFileChunked |
| B-33 | — | SymbolGraphUpdated |
| B-34 | — | EntryPointsDetected |
| B-35 | — | ArtifactsIndexed, ArtifactSearched, ArtifactStale |
| B-37 | (fd5c2e5) | runtime_fallback, runtime_health_changed, cli_subprocess_spawned, cli_subprocess_killed |
| B-38 | — | ambiguity_scored, ambiguity_floor_failed, ambiguity_streak_advanced |
| B-39 | 28→29 (6dd4af5) | ReflectTriaged, ReflectCandidate, ReflectAuditVerdict, ReflectRefined |
| B-40 | →41 (pin 42 vs B-34) | idea_framing_started, project_class_classified, critic_round_complete, idea_framed, tradeoff_card_emitted |
| B-41 | →45 | per_level_audit_started, per_level_audit_verdict, per_level_audit_cascade |
| B-42 | →44 | cosmos_refresh_started, cosmos_artifact_updated, cosmos_artifact_decayed, cosmos_grounding_used |

### Hard rules invoked across files
- blake3 (NEVER blake2/sha2 for non-RFC uses) — B-1, B-29, B-31, B-32, B-35,
  B-42 deps.
- No `.unwrap()` outside test/main/const-init — universal.
- No new external dependencies without user confirmation — B-3 (ts-rs), B-4
  (3 ONNX models), B-16 (ort stable), B-23 (cytoscape npm deps), B-31 (fs2
  if not present), B-33, B-37 (engine crates to src-tauri), B-38, B-42.
- No std::collections::HashMap in non-test (use BTreeMap/IndexMap/FxHashMap)
  — B-15, B-16 lint.
- Entropy quarantine: SystemTime::now/Instant::now ONLY in core/src/clock.rs +
  tests/ — B-15.
- Tier C isolation MANDATORY for CLI subprocesses (B-37 7 ports).
- Tree-sitter retained (no ast-grep swap) — B-30.
- HARDWARE-HEAVY flag before sidecar boot; remote-only (never local) —
  B-25, B-36, B-37.

### File ownership map
- Path constants source of truth: B-1 creates `crates/core/src/paths.rs`.
- elai_cc.toml schema owners: B-22 ([agents.{role}.backend]), B-27 ([cloud],
  [cloud.privacy], [execution]), B-35 ([[knowledge.artifacts]]), B-37
  ([runtime], [agents.<role>.runtime]), B-40 ([research_presets.<class>]),
  B-42 (~/.elai/cosmos/manifest.toml — user-global, NOT project elai_cc.toml).
- Decision docs: every B-N produces workflow/reasoning/B-N-decisions.md.
- Metrics: phase_3K_rag.toml + phase_3K_rag_improved.toml (currently in repo
  status), phase_4F.toml (B-21), phase_4G.toml (B-22 shipped), phase_4A.toml
  (B-18 real metrics), phase_b29_5_*.toml (B-29.5), phase_b31_5_*.toml
  (B-31.5), phase_4M.toml (B-37 shipped).

### Cross-B-N dependency graph
- B-1 unblocks: B-3, B-4, B-7, B-29, B-31, B-35.
- B-2 unblocks: B-3 (StatusBar verification), B-22 (AutoAdvanceStats badge
  via FSM events).
- B-3 ts-rs dep reused by: B-4, B-9 (SessionHandoffEntry), B-13.
- B-5 + B-8 are co-shipped (B-5 producer, B-8 activates production callers).
- B-6 + B-10 are sibling (ContextBudget bar chart).
- B-11..B-14 mostly DONE in V2; verification + drift backfill remaining.
- B-17 = meta-step over B-2/B-4/B-6/B-8 (regression suite + dashboard panel).
- B-18 → B-19 → B-20 sequential (IPC accept loop → hook unify → tool unify).
- B-21..B-25 are major phase implementations (4F..4J); each 2-3 sessions.
- B-22 unblocks B-23 (autonomous loop), B-26 (MonitorWindow signals).
- B-26 has 6 remaining polish tasks post-2026-05-07 shipped majority.
- B-29 unblocks B-33 (Decomposition graph viewer), B-34 (entry points), B-35
  (artifact pipeline overlaps).
- B-29.5 + B-31.5 are sub-step verifications (gate B-29/B-31 release).
- B-30 makes B-29 multi-language (extractors).
- B-35 unblocks B-40 (artifact-grounded framing), B-42 (cosmos extends).
- B-36 wires backend for everything dashboard-consumed (B-22 through B-42
  surfaces).
- B-37 unblocks per-role cloud routing for B-23/B-24/B-25/B-27/B-28.
- B-38 augments B-17 + B-8 with continuous score.
- B-39 + B-39C are independent of FSM but reuse B-5/B-8 GRC primitives.
- B-40 → B-41 → B-42 chain: framing produces context for per-level audits;
  cosmos provides cross-project ground-truth for both.

### Recommendation order (per B-0 INDEX Part V ranking)
1. B-29 highest leverage / unblocks Auditor pre-edit safety
2. B-31 robustness baseline / unblocks multi-million-LOC repos
3. B-30 closes A-35 / multi-language coverage
4. B-35 Designer + Researcher grounding fuel
5. B-32 cheap; fixes minified-file bugs
6. B-33 visible polish; consumes B-29 + B-7
7. B-34 smallest; nice-to-have for Designer
8. B-29.5 scale verification gate
9. B-31.5 scale verification gate

### Total effort estimate (per B-0 INDEX)
| Part | Sessions |
|------|---|
| I (B-1..10) | 10-15 |
| II (B-11..14) | 4-6 |
| III (B-15..20) | 6-10 |
| IV (B-21..28) | 16-24 |
| V (B-29..35) | 11-17 |
| V scale (B-29.5, B-31.5) | ~2 (wall-clock multi-hour indexing) |
| VI (B-36) | 4 |
| VII (B-37, B-38) | 5 |
| VIII (B-39..B-42) | ~6 (mostly shipped) |
| **TOTAL** | ~53-78 |

Milestones:
- After Part I+II (~14-21 sessions): MVP visually correct + functionally wired.
- After Part III (~20-31 sessions): production-grade hardening. Ship-ready.
- After Part IV (~36-55 sessions): full-spec including autonomous, self-mod,
  cloud, focus tunnel.
- After Part V (~47-72 sessions): symbol-level safety + multi-language +
  indexing robustness + artifact grounding. ELAI no longer Rust-only in
  practice.

---

## Status snapshot (as of 2026-05-17)

SHIPPED (commit-verified): B-1..B-10 (PARTIAL B-7 closed via B-7b),
B-11..B-14 (V2 re-scoped subset DONE), B-15 (cli/main.rs decomp only —
remaining items PLANNED), B-22 (Phase 4G + patch-4G.1), B-24 (Phase 4I
end-to-end), B-26 (majority shipped; 6 polish tasks remaining), B-36 (Track D
DONE), B-37 (fd5c2e5 — 11 backends + RuntimePanel + live cloud Sonnet 4.6),
B-39 (s-30584 commit 6dd4af5), B-39C (s-34648 commit 6adb454), B-40, B-41,
B-42.

PLANNED (file exists, not executed): B-16, B-17, B-18, B-19, B-20, B-21,
B-23, B-25 (conditional), B-27, B-28, B-29, B-29.5, B-30, B-31, B-31.5,
B-32, B-33, B-34, B-35, B-38, B-22b (4G.9 ouroboros port).

DEFERRED: B-25 (conditional on Empire codebase access), B-39B (AISLE/Mythos
eval needs 3090 box), B-39D/E/F (REFLECT progress UI + system-hint mode +
compaction guard).

OPEN FOLLOWUPS (memory + B-0 status): B-7b WebResearchStepEmitter per-stage
DONE; B-6b AgentLifecycle "exited" + live get_agents DONE; B-4b code_vec
full rollout (remove ELAI_CODE_VEC=1 opt-in after testing); B-9 dag-write-
in-dashboard (~80 LOC); WARN-1/WARN-2 cleanup (rule_engine.rs HashMap→
FxHashMap + dispatcher.rs HashMap→BTreeMap sync); live UI verification
with real model.

End of summary.
