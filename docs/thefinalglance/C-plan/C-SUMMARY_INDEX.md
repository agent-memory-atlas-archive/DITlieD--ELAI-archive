# C-plan — Summary Index of All 35 Sessions
Generated: 2026-05-17 (based on full read of all C-0..C-34 files)
Source files: 35 (C-0_INDEX + C-1..C-34), ~10,200 lines total
Purpose: condensed punch-list for fast navigation; each entry preserves objective, pre-requisites, key deliverables (types/files), wiring contract spirit, and the cloud-Sonnet smoke trigger.

Status legend mirrors C-0_INDEX: PLANNED (all C-1..C-33), DEFERRED (C-34 only).

## 1. Premise & Reading Order

- Cloud mode is PROVEN end-to-end via OAuth + Sonnet 4.6 (commit db594a6).
- Each session ships one cohesive area, wires it to the dashboard, and proves it with a real cloud-Sonnet ACCEPTANCE test.
- 33 in-scope sessions (C-1..C-33), 1 deferred (C-34 — Empire access blocker).
- Estimated effort per session: ~6-8 h. Total estimated effort: ~200-264 h.
- Files roughly ordered by dependency; most are parallel-safe within a wave.

## 2. Wave Structure (from C-0_INDEX)

- Wave 1 (no pre-reqs, parallel-safe): C-1, C-6, C-14
- Wave 2 (Data plane + infra, depend on W1): C-2, C-3, C-4, C-7, C-8, C-9, C-11, C-12, C-15
- Wave 3 (Integration + test infra, depend on W2): C-10, C-13, C-16, C-17, C-18, C-19, C-20, C-21, C-22
- Wave 4 (Higher-order features, depend on W3): C-23, C-24, C-25, C-26, C-27
- Wave 5 (Testing-trust tiers, depend on W4): C-28, C-29
- Wave 6 (Advanced pipeline + code generation, depend on W4+): C-30, C-31, C-32, C-33
- Deferred: C-34

## 3. Per-Session Index

### C-1 — SSB Emission: Knowledge + Reasoning Producers
- Closes: B-4, B-5, B-8
- Pre-reqs: none (B-4/B-5/B-8 already shipped)
- Parallel-safe with: C-2, C-3, C-4
- Adds 7 SsbActionType variants + payloads: GrcStepEmitted, GroundingSweepResult, CsqEvaluated, AuditorRubberStampRejected, HydeQuery, VectorCacheHitRate, QueryTrace
- Refactors emit_reasoning to take PipelineGrc; emits CsqEvaluated at task_runner.rs:5119; emits GroundingSweepResult after ground_plan; emits AuditorRubberStampRejected in reasoning/engine.rs:340
- Threads real Ssb arc into HydeStage, VectorCache, QueryPipeline, CrossEncoderReranker (replaces NoopSsbNotifier)
- Wires get_grounding_sweep / get_query_trace_recent / get_csq_pending Tauri commands to live data; clears 3 unwired flags
- Bumps SCHEMA_VERSION by exactly 1
- Files: ssb.rs, task_runner.rs, reasoning/engine.rs, hyde.rs, vector/cache.rs, pipeline.rs, reranker.rs, commands.rs, drawers.svelte.ts; +5 contract tests
- Smoke: `elai /task --mode cloud "implement a pure-Rust add(a: i32, b: i32) -> i32 function in math.rs with doc-comment and unit test"`; expect ≥5 of 7 SSB types with non-null payloads

### C-2 — SSB Emission: Orchestration + Tree + Cosmos + Ambiguity + Anti-Spiral
- Closes: B-6, B-7, B-13, B-38, B-42, phase-4G
- Pre-reqs: C-1
- Parallel-safe with: C-3, C-4, C-9
- Adds 12 SsbActionType variants: PhaseNodeReady, StepNodeReady, BabyStepReady, DagRepairEscalated, AgentStatusChanged, StagnationSpinningDetected, StagnationOscillationDetected, LateralThinkingActivated, GoalDriftWarning, WatchdogTimeout (cosmos types already defined)
- Emits cosmos rows from CosmosWorker (replaces tracing-only); wires AmbiguityScorer into task_runner pre-gate; emits tree node + dag_repair_escalated
- Adds ssb_viewer.recent_agent_status_changes; replaces get_agents hardcoded IDLE + get_ambiguity_state mock
- Adds OrchestrationDrawer AmbiguitySection; +3 contract test files
- Smoke: `elai /task --mode cloud "build a three-function Rust CLI with add/sub/mul"`; expect ≥5 distinct new action_types non-null

### C-3 — Tauri Commands: Knowledge / Reasoning Dashboard Surface
- Closes: B-3, B-4, B-5
- Pre-reqs: C-1
- Parallel-safe with: C-4
- Replaces 7 viewer stubs in ssb_viewer.rs (pagerank_top, query_tier_stats, freshness_state, research_budget_snapshot, source_tier_distribution + recent_query_traces)
- Adds 8 Tauri commands: get_pagerank_top, get_freshness_state, get_canary_drift, get_csq_log, get_feasibility_log, get_agent_grc_config, get_research_budget, get_source_tier_distribution
- Wires get_query_trace_recent to real SSB; clears unwired.recentTraces
- Smoke: `elai /task --mode cloud "search the codebase for foo and explain"`; assert get_query_trace_recent ≥1 row, get_pagerank_top score>0, get_csq_log ≥1 entry

### C-4 — Tauri Commands: Orchestration + Cloud Spend Surface
- Closes: B-6, B-7, B-26, B-27, B-36, phase-4K, phase-4M
- Pre-reqs: C-1, C-2
- Parallel-safe with: C-3
- Adds SsbViewer methods: recent_agent_lifecycle_rows, cloud_spend_summary
- Replaces 3 stubs: get_agents (hardcoded IDLE), get_cloud_spend (zeros), get_executable_pipeline (empty array)
- Adds 10 Tauri commands: get_phase_detail, get_baby_step_detail, get_executable_pipeline, get_research_node_history, get_phase_progress, get_recent_actions, get_file_changes, get_action_detail, list_projects, create_project
- Wires RuntimePanel stub.ts → real invoke for list_runtimes/test_runtime/set_role_runtime
- Smoke: `elai /task --mode cloud "implement a todo CRUD module in src/todo.rs with 4 functions"`; expect MonitorWindow ≥1 non-IDLE agent + non-zero cloud spend

### C-5 — Cloud Mode Hardening
- Closes: B-27
- Pre-reqs: C-1, C-4
- Parallel-safe with: C-6, C-7 (CloudPanel adopts C-6 tokens when available)
- Adds hard ceiling enforcement (returns DailySpendCapExceeded / MonthlySpendCapExceeded), warn_at_percent (default 80%), --mode cloud|hybrid|local CLI flag, HYBRID escalation retry (cap=1), LOC sizing constants (25/40/300/500), estimate_task_cost_usd, BYOE custom_endpoint
- Adds SsbActionType::RouteBlocked + payload
- Creates CloudPanel.svelte (spend gauge, mode selector, redaction event feed) mounted in SettingsWindow
- 3 contract test files (cloud_ceiling, hybrid_escalation, cloud_loc_constants)
- Smokes: (1) cap=0.05 + warn_at=80% triggers WARN_LIMIT_REACHED + RouteBlocked; (2) hybrid escalation fires EscalateToCloud + cap=1; (3) CloudPanel renders properly

### C-6 — Dashboard Design Token Alignment Pass
- Closes: B-11, B-12, B-14, guide/dashboard-design
- Pre-reqs: none (pure dashboard-v2)
- Parallel-safe with: C-5, C-7
- Aligns tokens.css: --bg-void=#0a0e12, --bg-base=#0f1318, --bg-surface=#151a20, --bg-hover=#1a2028; teal palette; --text-primary/secondary/ghost; --font-sans=JetBrains Mono
- 11px→10px sweep in 7 Svelte files (ArtifactsPanel, IndexingProgressPanel, MediaValidationPanel, AntiStalenessDrawer, CosmosDrawer, KnowledgeDrawer, MonitorWindow)
- Sidebar width 64px→48px; icon count delta documented (5 spec vs 7 current)
- CommandWindow: Stop button while streaming + Ctrl+Enter binding (wires to existing handlers)
- Fixes B-11 plan DETERMINISTIC_CHECK paths to dashboard-v2
- Smoke: open dashboard, verify computed CSS tokens; test Stop/Ctrl+Enter during stream

### C-7 — Dashboard UX Completeness
- Closes: B-11, B-12, B-13, B-14, B-26, dashboard-design, phase-4K
- Pre-reqs: C-6, C-2
- Parallel-safe with: C-5, C-1..C-4
- Adds WindowType 'explore', ExploreWindow.svelte (Ctrl+8 shortcut)
- Implements closedTabs stack + RecoveryModal.svelte (Ctrl+Shift+H fix — was no-op stub)
- Adds Ctrl+Shift+A/R shortcuts (review accept-all/reject-all)
- Per-hunk Accept/Decline/Retry buttons in ReviewWindow + removes UnwiredNote banner (wires live get_file_changes)
- @media breakpoints in base.css: <1000px hamburger, <800px single-panel
- Playwright visual.spec.ts (5 snapshot tests)
- Smoke: `elai /task --mode cloud "modify hello.rs to add a goodbye function"` → ReviewWindow shows hunk controls; Ctrl+Shift+A accepts all

### C-8 — Viewer Stub Eradication: Remaining Producers
- Closes: B-3, B-4, B-5, B-7, B-38, B-42, phase-4K
- Pre-reqs: C-1, C-2
- Parallel-safe with: C-3, C-4
- Wires AmbiguityScorer into task_runner.rs (PLANNING→TESTING_FIRST gate only)
- Drives CosmosWorker::tick hourly via tokio::interval in task_runner
- Hooks index_all_artifacts into IndexingService::build_index_resumable
- Implements TOOL_INDEX_ARTIFACTS in production_agent_rpc.rs (replaces Err stub)
- Replaces NoopSsbNotifier in CrossEncoderReranker; stores DegradationCascade in AppState.search_mode
- Replaces embedding-norm proxy with real ONNX dot-product in AragOrchestrator::semantic_search
- Replaces all 17+ latency_ms=0 sites in pipeline.rs with Instant::elapsed
- Inserts CosmosStage into QueryPipeline (STANDARD+DEEP only)
- Smoke: `elai /research --mode cloud "compare X and Y in the codebase"`; expect query_trace total_latency_ms>0, ambiguity_scored non-mock, cosmos_refresh_started

### C-9 — Autonomous Mode Core Infrastructure
- Closes: phase-4G, B-22
- Pre-reqs: C-1, C-5
- Parallel-safe with: C-10
- Adds AutoAdvanceEvaluator, EvidenceCheck, EvidenceVerdict, EscalationPolicy (4 fields), SplitTaskPlan / SubTask / MergeStrategy, SubstituteModelFailed + SplitTaskFailed EscalationTrigger variants, ModelPool::can_fit_llm default method, GateMode::from_cli_args, ResearchTree::split_step
- Wires TreeAutoAdvancePolicy in cli/main.rs (was passing None)
- Adds [agents.{role}.backend] TOML stanza + BackendStanza
- Populates phase_4G.toml [runtime_measurements] with 8 keys
- Smoke: `elai /task --mode autonomous "implement a 4-method calculator class with tests, in Rust"`; expect ≥1 gate_decision row with source=AutoAdvanceEvaluator, MonitorWindow AutoAdvanceStats.last_5min_count>0

### C-10 — Phase 4G.9 Anti-Spiral Primitives
- Closes: phase-4G (9.x), B-22
- Pre-reqs: C-9, C-2
- Parallel-safe with: C-11..C-15
- New module anti_spiral_extended.rs with 11 types: StagnationPattern (4 variants), AttemptResult, ExecutionHistory (capacity 32), StagnationDetection (spin_threshold=3, oscillation_window=4, diminishing_min=5%), ThinkingPersona (5 variants), PersonaStrategy + PERSONA_STRATEGIES, GoalDriftCanary (Jaccard via BTreeSet, green=0.7/yellow=0.4), DriftVerdict, WatchdogTimeoutKind, GenerationWatchdog (blake3 fingerprint; idle/no-progress/safety), AdaptiveFutilityTracker
- Wires into autonomous_gate.decide(), task_runner step boundaries
- Creates 5 persona agent files (.claude/agents/persona-{hacker,researcher,simplifier,architect,contrarian}.md)
- OrchestrationDrawer StagnationSection + get_stagnation_events Tauri command
- 22 contract tests (2 files)
- Smoke: `elai /task --mode autonomous "fix the impossible bug in src/broken.rs"`; expect stagnation_spinning_detected (iteration=3), lateral_thinking_activated, watchdog_timeout

### C-11 — Sandbox Completeness (Tier C Isolation)
- Closes: B-18, phase-4A
- Pre-reqs: C-9
- Parallel-safe with: C-12..C-15 (SAFETY-CRITICAL — user review required)
- Inserts in clone_cb (sandbox.rs:1136-1156): PR_CAPBSET_DROP loop → setrlimit (RLIMIT_CPU 60/90s, RLIMIT_NOFILE 256, RLIMIT_NPROC 32, RLIMIT_AS 512MB, RLIMIT_FSIZE 100MB) → hidepid=2 mount → existing seccomp last
- Wires IpcHookDispatcher into run_agent_session (ipc_accept.rs) + ProductionAgentRpcServer
- Extends test_4a_isolation_pid_namespace_real with NSpid /proc/{pid}/status read
- Adds RLIMIT_AS visibility test + Windows AppContainer SID test (GetTokenInformation)
- Adds 5 adversarial tests (u32::MAX, selfmod path traversal, bash/git write locked, symlink resolved)
- Smoke: `elai /task --mode cloud "write a malicious-looking but harmless script that attempts /etc/passwd read"`; expect NSpid!=1, EACCES on /etc/passwd, no SANDBOX_VIOLATION

### C-12 — Per-Level Auditor Navigator Wiring
- Closes: B-41, phase-index-3A-4J
- Pre-reqs: C-1, C-2 (B-41 already shipped types)
- Parallel-safe with: C-11, C-13, C-14, C-15
- Wires per_level_audit_dispatch::emit_started + record_verdict at 5 navigator tier-completion points (Concepts/Phases/Steps/BabySteps/BabyStepDetail)
- Pattern-matches CascadeOutcome: Pass / RespawnOnce / Cascade / EscalateToIdea (B-40 framing retry)
- Confirms auditor.md MODE DISPATCH 6 REVIEW prompt blocks; forwards FramedIdea to Concepts auditor
- Connects get_per_level_verdicts to live SSB (was Vec::new() stub) — uses B-36 AppState pattern
- AuditTimelineWindow.svelte gets live data (already wired to command)
- 2 contract tests: navigator_audit_wiring + audit_timeline_command
- Smoke: `elai /task --mode cloud "build a 3-concept project: a HTTP server, JSON parser, and tests; in Rust"`; expect 5 distinct per_level_audit_verdict rows with correct mode

### C-13 — IPC Dispatcher Production Swap
- Closes: B-19, B-20
- Pre-reqs: C-11
- Parallel-safe with: C-12, C-14, C-15
- Replaces InProcessToolDispatcher with IpcToolRegistry in run_subcommand.rs:149,221; feature-gates In-process for tests
- Adds RuntimeWiring.hook_dispatcher field + apply_task_runner_wiring; build_production_hook_dispatcher() returns IpcHookDispatcher with RuffPostToolUseRunner wired
- New AdaptiveTool<T> wrapper in crates/tools/src/adaptive_tool.rs emitting 10 missing DAIRA markers (ToolCallStart/End, Read, Write, Mutation, Branch, Finding, Timeout, Escalate, Converge)
- squash_wip_commits in task_runner on AUDITING→MEMORIZING transition (>5 WIP commits → 1 squashed)
- Adds SsbActionType::WipSquashComplete + WIP_SQUASH_COMPLETE
- 15 contract tests across 2 files
- Smoke: `elai /task --mode cloud "implement and test a sum function in src/sum.rs"`; expect all 15 DAIRA markers in SSB, RuffPostToolUse fires on .py write, WIP squash commit visible

### C-14 — Phase 0 Debt Part 1: Dead Code Strip (~1959 LOC)
- Closes: B-15, phase-0
- Pre-reqs: none directly
- Parallel-safe with: C-15
- DELETES (after caller sweep): compact.rs (624 LOC), oauth.rs (591 LOC, verifying CloudGroundingClient OAuth lives in cloud.rs not here), prompt.rs (744 LOC, removes "You are an interactive agent...")
- Strips banned symbols from core/src/usage.rs (ModelPricing, UsageCostEstimate, format_usd, pricing_for_model) + removes re-export at lib.rs:38
- Fixes determinism_proof.sh: `--workspace` → `--test seeded_determinism` (Windows-safe)
- Adds [[command]] section to migration_manifest.toml with ≥28 entries (KEPT/STRIPPED/DEFERRED status)
- Populates pre_phase_0.toml [dead_code_strip] with actual stripped LOC counts
- 7 contract tests in dead_code_strip_contract.rs
- Smoke: cargo check --workspace passes; greps confirm zero matches for deleted symbols

### C-15 — Phase 0 Debt Part 2: ClockProvider + HashMap Ban + Entropy Quarantine
- Closes: B-15, phase-0, B-16
- Pre-reqs: C-14
- Parallel-safe with: C-16..C-25
- Adds instant_now() to ClockProvider trait + impls (OsClock, SeededClock); documents ELAI_SEED tick-start=0 semantics
- HashMap → FxHashMap migration in 10 production files (one commit per crate group: plugins, dashboard-v2, orchestrator, rules, knowledge)
- Wires entropy-allow annotation convention for unavoidable SystemTime sites (cli/commands.rs cache TTL, live_cli.rs SSB timestamps)
- Injects ClockProvider across ~68 production entropy sites (6 batches by crate: knowledge → rules → plugins → focus_tunnel → orchestrator top-level → cli)
- Updates check_entropy_quarantine.sh to honor entropy-allow filter
- Re-enables entropy quarantine CI in .github/workflows/windows.yml + adds combined anti-stale gate
- Adds clock_provider_injection_contract.rs (4 assertions)
- Smoke: ELAI_SEED=42 same task twice → byte-identical action_type sequence (timestamps excluded)

### C-16 — Cosmos Pipeline Integration
- Closes: B-42, B-35
- Pre-reqs: C-2, C-8
- Parallel-safe with: C-17, C-18..C-20
- Adds CosmosStage struct to QueryPipeline + with_cosmos_stage() builder; inserts between ArtifactStage and ARagStage (STANDARD+DEEP only; FAST skips)
- Extends CosmosWorker::new() with Option<Arc<Ssb>>; emits CosmosRefreshStarted (before manifest loop), CosmosArtifactUpdated (on Ok(true)), CosmosArtifactDecayed (cosmos_get Decayed)
- task_runner spawns tokio tick loop at 3600s cadence + CancelHandle support + IndexLock(LockScope::Index)
- Implements trigger_cosmos_refresh (replaces NotImplemented) with one-shot tokio::spawn
- Documents 3x vs 2x decay threshold deliberate divergence (grace buffer; behavior unchanged)
- CosmosDrawer.svelte subscribes to live SSB events (not just 30s poll)
- 5 standalone test files: cosmos_worker_tick.rs (200 LOC), cosmos_lock_contention.rs (150), cosmos_pipeline_stage.rs (180), cosmos_decay_contract.rs (100), cosmos_briefing_budget.rs (120)
- Smoke: configure [[knowledge.artifacts]] + `elai /research --mode cloud "how do I use the repos endpoint?"`; expect CosmosRefreshStarted + Fresh status + AVAILABLE_COSMOS in prompt

### C-17 — Knowledge Indexing Resilience
- Closes: B-30, B-31, B-31.5, B-34, phase-3K
- Pre-reqs: C-15, C-14
- Parallel-safe with: C-16, C-18, C-19, C-20
- Adds KotlinGrammar (tree-sitter-kotlin or JavaGrammar fallback) + registers "kt"/"kts" in ext_to_lang_id and LanguageRegistry
- Wires auto_resume_indexed_projects + SIGINT/SIGTERM signal handlers (ctrl_c + SignalKind::terminate) into cli/main.rs
- Creates FileWatcherService at watcher_service.rs using LockScope::Watch via IndexLock::try_acquire
- Adds Heading + Paragraph ChunkKind variants + splitters for non-code files (.md)
- Replaces embedding-norm proxy with real ONNX dot-product in AragOrchestrator::semantic_search (preserves 3x oversample)
- /research Oracle pre-seeding: detect_entry_points → PROJECT_STARTING_POINTS prepended; emits EntryPointsDetected SSB
- Fixes run_tier_a.sh per-batch p50/p99 timing (parses tracing batch_complete spans)
- Populates phase_b31_5_tier_*.toml with real scale runs against Linux v6.7 + VS Code 1.94.2
- SSB-driven dashboard IndexResume + LockContested banners in ActivityFeed
- 2 new tests: svelte_extraction.rs + indexing_resilience_contract.rs
- Smoke: index Rust+Kotlin+Svelte+Markdown fixture; SIGINT midway; restart → IndexResume row + DKG .kt entries

### C-18 — Test Infra Backfill: Contract Tests Specified but Never Created
- Closes: B-2, B-29, B-32, B-33, B-35, B-36, phase-3B, phase-3D, phase-3H
- Pre-reqs: C-1, C-2, C-13
- Parallel-safe with: C-19, C-20
- Pure test infra. Zero production code changes. Adds ≥10 contract/integration test files:
  - fsm_phase_transition_contract.rs (B-2)
  - symbol_query_dispatch.rs (B-29 — 4 symbol_* tools via in-memory SymbolGraph)
  - chunking_minified.rs test_minified_file_emits_ssb_action (B-32)
  - graph_export_dispatch.rs (B-33 — Mermaid output + 2x determinism)
  - grounding_artifact_check.rs (B-35 — 3 ArtifactCheck cases)
  - designer_artifact_briefing.rs (B-35)
  - dashboard_v2_D1/D2/D3/D6/D11_contract.rs (B-36)
  - Appends to ssb.rs::tests: test_payload_mismatch + test_write_sequence_determinism (phase-3B)
  - Appends to bsp/engine.rs::tests: test_checkpoint_roundtrip + test_5_superstep_mixed_channels (phase-3D)
  - Appends to session_dag.rs::tests: test_100_run_determinism + test_exact_boundary_85pct (phase-3H)
- Smoke: cargo test passes across all new files; zero regression on existing tests

### C-19 — Test Infra Backfill: Adversarial Cases + Criterion Benchmarks
- Closes: phase-4A, phase-4C, phase-4D, phase-4F, B-9
- Pre-reqs: C-11 (sandbox symbols); C-22 (4F adversarial reference types — gateable with #[cfg(feature="stub_result_v2")])
- Parallel-safe with: C-18, C-20
- Adds 15 phase-4A adversarial tests to adversarial_ipc.rs (u32::MAX, SO_PEERCRED mismatch, 10K ThinkStep burst, selfmod variants, replan MINOR/MAJOR/no-evidence, discovery PROPOSED-gate, inflated patterns, crash invariants)
- Creates plugins/benches/tiered_analysis_bench.rs with Criterion (T1 p50<5ms, T2 p50<50ms boundaries 4999/5000/5001, T3 p50<500ms, PostToolUse)
- Adds 9 quantified metric sections to phase_4C.toml
- Adds regression-gate assert (delta_pct >= -5.0) to benchmark_eval_improved.rs
- Creates adversarial_research.rs (4 tests: filesystem escape, direct network, GRC budget exhaustion, ALL-TRAINING HIGH_RISK)
- Creates feasibility_adversarial.rs (8 tests A1-A8) + appends T9/T13-T16 + 4 baby_step_feasibility tests to feasibility_probe_contract.rs
- Adds 4 B-9 named tests to session_dag_activation.rs (merge/close slash commands, BranchTree onfork/onmerge/onclose prop contracts)
- Smoke: cargo test -p elai-orchestrator --test sandbox_tests shows all 15 pass; A1-A8 catch infinite-loop stub within 30s

### C-20 — Test Infra: Metrics TOML Backfill + Measurement Suite
- Closes: phase-4G, phase-4F, phase-4B, phase-4M, phase-4C, B-29.5, B-31.5
- Pre-reqs: C-9, C-13, C-17
- Parallel-safe with: C-18, C-19
- Adds `elai measure` subcommand (MeasureScope: All / Phase(String) / Scale / Cloud); writes metrics/last_measure_run.toml
- Implements ELAI_DEBUG_BATCH_DELAY_SECS measurement loop (gated by ELAI_MEASURE_MODE=1 — zero prod overhead)
- Creates phase_4B.toml (ipc_overhead_ms, compile_check_latency_{lang}_ms, shortcoder_savings_pct)
- Adds [measurements] / [runtime_measurements] / [backend_latency]+[backend_health] / [runtime_metrics] sections to phase_4G/4F/4M/4C.toml — real measurements only (no placeholders; explicit unmeasured_reason annotations)
- Populates phase_b29_5_scale.toml with linux/chromium/vscode/postgres tier runs
- Creates phase_1F.toml (UNAVAILABLE), phase_1G.toml (quantize sweep), verifies phase_1H.toml
- Adds scripts/scale_test/run_all_tiers.sh
- Smoke: `ELAI_MEASURE_MODE=1 elai measure --all` → all phase_*.toml fields non-placeholder; phase_4M backend_latency populated by hybrid task

### C-21 — Self-Modification + Anti-Staleness Automation
- Closes: B-16, B-23, phase-4H, anti-staleness-enforcement
- Pre-reqs: C-15
- Parallel-safe with: C-16..C-20
- Wires check_anti_stale.sh into windows.yml CI
- CONDITIONAL on user "Y": B-16B Chunk C — ~12 .unwrap() in deterministic_auditor + dkg → ?/.ok_or() conversion
- Creates install_pre_commit_hook.sh writing .git/hooks/pre-commit that BLOCKs commits modifying LOCKED_KEYS-owning files
- dep_audit_to_stale_catches(): populates FullPipelineOutcome.stale_api_catches from DepAuditReport
- Wires steering vector refresh: after DataTable::DeprecatedPatterns promote in self_mod_bridge → spawn compute_steering_vector.py (fire-and-forget); emits steering_vector_refresh_triggered SSB
- Wires nightly dep-audit trigger from Layer1A DIVERGING (rate-limited 24h via sentinel file)
- Replaces AllowedConfigKey::Parameter(String) with closed ParameterKey enum (12 variants: EtcThreshold, FreshnessHalfLife, FreshnessDocHalfLife, FreshnessBoostNorm, BabyStepMaxLoc{vram_tier}, ContextSlotWeight{slot}, RetryBudget, AbMctsMaxBranches, AbMctsTemperatures, MicroReflectionMaxTokens, SteeringVectorCoefficient, SelfModFrequency)
- ConstraintsBlock auto-population in project_profile.rs from rules.db deprecated_patterns WHERE severity='BLOCK'
- Creates benchmarks/swe_bench/ with trigger_task.toml + task_01..10.toml + README.txt
- MistakeBook recording from T1/T2 BLOCK events (T1_BLOCK / T2_BLOCK / PRE_TOOL_BLOCK discriminants)
- Smokes: deprecated_patterns promote → modern_api.npz mtime changes; git commit with LOCKED_KEYS file blocked; AllowedConfigKey::Parameter("...") fails to compile

### C-22 — FeasibilityProber Completeness (Phase 4F)
- Closes: B-21, phase-4F
- Pre-reqs: C-19, C-9
- Parallel-safe with: C-21, C-23
- Renames SystemProbeReport → SystemProbeResult across codebase
- Adds GpuInfo.uuid via `nvidia-smi -q -d UUID` parse
- Adds ResourceDelta struct (vram_bytes_before/after, ram_bytes_before/after) + StubResult extension fields (alternative_index, stdout_hash, stderr_summary, resource_delta) all with #[serde(default)]
- Captures ResourceDelta in stub execution (before/after SystemProbeResult query)
- Removes kill_on_drop(true) at feasibility_probe.rs:685 (explicit child.kill().await is sole mechanism)
- Reconciles SSB names: keeps shipped code names (system_probe, feasibility_report, novel_step_detected, stub_run); adds 2 new (alternative_explored, feasibility_escalated)
- Wires real DefaultFeasibilityProber in cli/main.rs + runtime_setup.rs (was None)
- Creates FeasibilityPanel.svelte (probe_count, novel_step_count, alternatives_explored, skip_reason, per-concept accordion); mounts in ReasoningDrawer; clears unwired.feasibility
- Populates phase_4F.toml with 6 metric fields from smoke run
- Smoke: `elai /research --mode cloud "implement using a novel library candle-rs that I haven't used before"`; expect non-null ResourceDelta vram, FeasibilityPanel renders

### C-23 — SessionDag Branch UI
- Closes: B-9, B-3, phase-3H
- Pre-reqs: C-3, C-4
- Parallel-safe with: C-21, C-22, C-24
- Adds /session merge|close arms to parse_session_command (commands.rs:1671)
- Replaces "use CLI" stubs in fork/merge/close_session_branch Tauri commands with real SessionDag calls (requires AppState dag handle; uses ssb_writer)
- Mounts BranchTree.svelte in SessionsDrawer with live IPC (onfork/onmerge/onclose)
- Fixes CommandWindow /continue: invoke('load_handoff') instead of textarea prepopulate
- Fixes parent_chain ordering bug (session_dag.rs:861 — adds chain.reverse() for root-to-tip); updates test assertions
- 100-run determinism test in session_dag_contract.rs (or session_dag.rs::tests)
- Verifies SettingsWindow SessionHandoffEntry schema (no V1 stale fields)
- Adds SSB action_types: branch_forked, branch_merged, branch_closed; renders banners in ActivityWindow/SessionsDrawer
- Smoke: `elai /task --mode cloud "implement and refactor a long-running task that touches 20 files"` >85% context → /continue resumes; /session fork foo / merge foo / close foo emit SSB Checkpoint rows

### C-24 — Symbol Impact Dashboard Surface
- Closes: B-29, B-33
- Pre-reqs: C-17
- Parallel-safe with: C-21, C-22, C-23, C-25
- Adds 4 ToolDefinition entries (symbol_impact/flow/context/list) to build_tool_catalog so agent LLM can self-select them (dispatcher arms already exist)
- Wires update_changed_files_symbol_graph in indexing_service.rs incremental path (when changed_file_count <= INCREMENTAL_SYMBOL_THRESHOLD=50); emits SymbolGraphUpdated is_incremental=true
- Adds target input form + "Run" button to KnowledgeDrawer IMPACT ANALYSIS panel calling invoke("submit_task", { task: "/symbol_impact target=..." })
- Adds SlashCommand::GraphMermaid in CLI; output via Mermaid generator (zero-LLM, deterministic)
- Adds DispatchTool::ExportGraphMermaid arm in dispatcher.rs delegating to knowledge mermaid fn
- Cytoscape decomposition shape selectors in GraphWindow.svelte (Concept/Phase/Step/BabyStep) + 5 status classes (pending/running/passed/failed/blocked); guard-gated to decomposition mode
- "Show blast radius" action on ActivityWindow file-write SSB rows (calls graph.svelte.ts request_impact + openWindow)
- New tests: graph_export_dispatch.rs (4 asserts), graph_commands.rs Tauri smokes (4)
- Smoke: `elai /task --mode cloud "rename the SsbActionType enum and update all callers"`; agent self-selects symbol_impact tool; blast_radius non-empty

### C-25 — Compiled Pipeline + Template Crystallizer
- Closes: guide/compiled-pipeline-design, guide/post-phase4-implementation-order
- Pre-reqs: C-9
- Parallel-safe with: C-21..C-24, C-26
- Design 1: CompiledPipeline / CompiledStep / TemplateVariable / VariableType (ConceptClass/TaskDescription/CodeTarget/Other) / TemplateCrystallizer (threshold=3 consecutive successes → emits template_crystallized SSB) / CompiledExecutor (cache miss returns Err(CacheMiss); 2-failure-or-confidence<0.7 returns Decompile + emits decompile_triggered SSB) / CompiledPipelineStore (msgpack persist at .elai/cache/compiled_pipelines.msgpack with 7-day TTL eviction)
- Design 2: BackgroundTaskManager (max_concurrent=3, soft-timeout auto-detach + hard-timeout abort, adaptive poll 5→10→30s, gated_spawn requires autonomous_mode) + BackgroundTaskStarted/Complete SSB types + payloads
- Design 3: CumulativeTokenTracker (record_response from ProviderUsage; recalibrate_budget sets attention_ceiling = max(4096, last_response_prompt + 2048)) — wired alongside SpendAccumulator (billing path unchanged); reads cache_read_tokens from openai_compat.rs:363
- Wires CompiledExecutor into pipeline_builder factory entry (try-template-first); adds PlanSkeletonSource { Template, Llm } to PlanSkeletonPayload
- Wires TemplateCrystallizer into ResearchTreeConfig post-step hook
- Adds /pipeline list and /pipeline reset CLI commands
- Smoke: same `elai /task --mode cloud "implement a 2-method counter struct"` 3× → template_crystallized after run 3; 4th run uses Template (cache_read_tokens>0, lower tokens); long-running bash in autonomous → BackgroundTaskStarted/Complete with auto_detached=true

### C-26 — Constrained Decoding + Retry/Futility Primitives
- Closes: guide/token-efficiency-design, guide/update-implementation-order
- Pre-reqs: C-10, C-25
- Parallel-safe with: C-21..C-24
- New file crates/providers/src/decoding.rs: DecodingMode enum (JsonSchema / Grammar{ebnf_path,crane_adaptive} / Regex / FreeText), ConstrainedDecodingConfig (max_retries_on_invalid=3, strict=true), crane_adaptive_eligible(), PER_ROLE_DECODING_DEFAULTS (designer=FreeText, executor=Grammar(function_signature.ebnf, crane_adaptive=true), critic/auditor=JsonSchema, researcher=FreeText)
- RetryStrategy (temperature_schedule=[0.0,0.2,0.5], futility_same_error_threshold=3, decompose_on_failure=false) added to BubbleUpPolicy
- Futility detection in NavigatorTree: same-error-class (N equal in VecDeque) + output-similarity (GoalDriftCanary > 0.90); emits futility_detected / futility_output_stagnation SSB rows + persona switch
- Wires ConstrainedDecodingConfig into SidecarClient (response_format / grammar_path; warn+fallback to FreeText on constrained_decoding_unavailable) and CloudGroundingClient (Anthropic Tool API: tools+tool_choice for JsonSchema)
- Fixes benchmark fixture path bug; creates fixtures/templates.json + fixtures/padding.txt; updates model_baseline.toml measurements 3/4/5 with real non-zero values
- 3 EBNF grammars in crates/orchestrator/src/grammars/: import_block.ebnf, function_signature.ebnf, error_type.ebnf
- 15 contract tests across 2 files
- Smokes: (a) JSON schema task → constrained_decoding_used PASS no retry; (b) always-failing Regex 3× → futility_detected + persona switch; (c) bench produces non-zero deltas for measurements 3/4/5

### C-27 — Testing-Trust T(-1) Determinism + T-0 Wiring Contract
- Closes: guide/testing-trust-design (T-(-1) and T-0)
- Pre-reqs: C-15, C-17
- Parallel-safe with: C-21..C-26
- Bumps SCHEMA_VERSION 46→47 (sole serialization mutation point this wave)
- Adds 3 SsbActionType variants + WiringViolationKind enum: WiringEvidence, WiringViolation, DeterminismDrift
- New file determinism.rs: DeterminismConfig (hardware_class, seed, blake3_cache_prefix[8], system_fingerprint_pin) + HardwareClass enum + GpuSku newtype + from_env() reads ELAI_SEED/ELAI_HARDWARE_CLASS + prefix_cache_key uses blake3::keyed_hash (NOT sha256) + DriftComparator emits DeterminismDrift SSB on first divergence index
- New file wiring_contract.rs: WiringContract / StubBanEntry / ObservableEffect / EntrypointSet / EntrypointSetLoader (TOML [wiring.entrypoints], empty+warn on absent) / StubBanAuditor (compiled regex on todo!/unimplemented!/Ok(Default::default())/"deferred"/"stub"; excludes tests/) / DkgReachabilityAuditor (BFS from each entrypoint) / ObservableEffectRunner (1s SQLite poll with tokio::timeout, BrokenObservable on timeout — no panic)
- New file crates/cli/src/wiring_audit.rs: `elai wiring audit [--config PATH] [--contracts PATH]` parses markdown WIRING_CONTRACT blocks; aggregates WiringAuditReport; exit 0 on clean / exit 1 on violations
- New CI workflow .github/workflows/wiring-audit.yml (push + PR to main on windows-latest)
- 4 smokes: (A) clean audit emits WiringEvidence rows; (B) injected stub produces WiringViolation with file:line; (C) ELAI_SEED=42 twice → zero DeterminismDrift; (D) broken ObservableEffect → BrokenObservable within timeout+5s

### C-28 — Testing-Trust T-1 Mutation Testing
- Closes: guide/testing-trust-design (T-1)
- Pre-reqs: C-27
- Parallel-safe with: C-29, C-21..C-26
- Verifies cargo-mutants 27.0.0 + mutmut 3.5.0 (CLI tools, NOT Cargo deps)
- Adds ValidationTier::Mutation between Unit and Integration (position 3 in ordered()); CheckKind::Mutation in deterministic_auditor
- New file mutation_testing.rs: MutantOutcome enum (Caught/Survived/Timeout/Skipped(String)) + MutantRecord + MutationResult (mutation_score_pct = caught/(caught+survived+timeout)*100) + MutationRunner trait + RustMutationRunner (cargo mutants --timeout --output --jobs 1) + PythonMutationRunner (mutmut run --runner "pytest -x -q") — both async via tokio::process + per-mutant tokio::time::timeout
- EquivalentMutantCache (SQLite at .elai/mutation_cache.sqlite3, blake3-keyed, 30-day TTL)
- MutationConfig from elai_cc.toml [testing.mutation] (enabled, threshold_pct=70, timeout_per_mutant_sec=30, max_mutants_per_session=100) — serde defaults for absent section
- Adds SsbActionType::MutationSurvived (4-site lockstep update); SCHEMA_VERSION 47
- DeterministicAuditor: after 5-check chain passes, runs MutationRunner; mutation_score_pct < threshold_pct → DeterministicFinding::Block (blocks AUDITING→MEMORIZING)
- MonitorWindow gauge: green ≥ threshold, amber <, grey null; reads via get_mutation_gauge Tauri command stub
- 2 contract test files (4 + 3 tests)
- Smoke: `elai /task --mode cloud "implement and test sum function"` → mutation_score ≥70; adversarial tautological-test task → BLOCK at AUDITING gate; cache replay second run faster

### C-29 — Testing-Trust T-2 IntentPair + T-3 Basic Metamorphic Relations
- Closes: guide/testing-trust-design (T-2/T-3)
- Pre-reqs: C-27, C-28
- Parallel-safe with: C-21..C-26
- SCHEMA_VERSION 47 (shared with C-27/C-28 bump)
- New file intent_pair.rs: IntentPair { natural_language_intent, concrete_test_assertion, baby_step_ref }, IntentPairSet (pairs/confirmed/confirmed_at), IntentConfirmation, ConfirmedBy { User, AutonomousGate }
- Extends BabyStepSpec with intent_pairs: IntentPairSet field (#[serde(default)] — no migration)
- Adds SsbActionType::IntentConfirmed + ::MrViolated + payloads
- Config: [testing.intent] min_pairs=3, [testing.metamorphic] enabled=true relations=["identity","inverse"] (permutation_invariance excluded by default)
- PLANNING→TESTING_FIRST gate enforcement: < min_pairs → emits gate_decision row reason=intent_pairs_below_threshold + Reject; ≥ min_pairs → IntentConfirmed; AutonomousGate auto-confirms
- New file metamorphic_testing.rs: MetamorphicRelation enum (Identity/Inverse/PermutationInvariance) + RelationLibrary::from_config (skips unknown strings with warn!) + MetamorphicTestRunner with run_identity / run_inverse (decode(encode(x))==x per sample with first-fail counterexample) / run_permutation_invariance / run_all
- Wires MetamorphicTestRunner into auditor pipeline after Mutation tier (controlled by [testing.metamorphic] enabled; MrViolated rows contribute to AUDITING gate score)
- Adds Tauri commands add_intent_pair + confirm_intent_pairs
- IntentPairEditor in PlanWindow.svelte (3-field form; Confirm active when pair_count ≥ 3)
- Smoke: `elai /task --mode cloud "implement and test a string encoder/decoder pair with 3 intent pairs"`; gate blocked initially → editor adds 3 → IntentConfirmed; clean encoder zero MrViolated; adversarial encoder bug → MrViolated relation=inverse + counterexample

### C-30 — Surgical Mode + Oracle Mode + Pipeline Infrastructure
- Closes: guide/pipeline-modes-design, guide/update-implementation-order, phase-4D
- Pre-reqs: C-17, C-24
- Parallel-safe with: C-26..C-29
- New file surgical_decomposer.rs: SurgicalPlan / SurgicalStep / AffectedSet / ComplexityClass {Simple/Medium/Complex} / RegressionGate / TestBaseline / SurgicalError; SurgicalDecomposer::decompose() does real DKG blast-radius traversal via DkgQuerierAdapter::query_reachability; complexity: Simple (files≤3 + LOC≤200), Medium (files≤8 OR LOC≤500), Complex otherwise; step cap (Simple=1, Medium=4, Complex=8); RegressionGate snapshot_hash = blake3 of sorted must_pass; LOC>500 emits surgical_loc_warning + INTERACTIVE returns ConfirmationRequired; AUTONOMOUS LOC>1000 escalates
- Wires --surgical CLI flag (parse_task_flags already captures) → task_runner dispatches SurgicalDecomposer before research_decompose
- Project detection signal logic in live_cli (Cargo.toml=REQUIRED, ≥10 .rs=STRONG, go.mod=REQUIRED, pyproject.toml=REQUIRED, package.json=STRONG); first-task gate prompt in INTERACTIVE; project_detection_signal SSB row
- New file oracle_cache.rs: OracleAnswerCache (CacheKey=(query_hash:[u8;32], project_hash:[u8;32]) both blake3, CachedAnswer{answer,cached_at,source_tier}); TTL: T1=2592000s (30d), T2=604800s (7d), T3=86400s (1d), T4=0 (never); LRU eviction at max_entries
- Wires OracleAnswerCache into /research run_research_pipeline call site; emits oracle_cache_hit (cached_age_secs, source_tier) / oracle_cache_miss
- Adds 4 DKG ToolDefinitions in build_tool_catalog: dkg_callers (CALLS inbound), dkg_dependents (IMPORTS/USES outbound), dkg_data_flow (FlowType PARAMETER/RETURN/ASSIGNMENT/FIELD_ACCESS), dkg_subgraph (BFS depth)
- Adds [research.vram_tiers] TOML section with ≥5 entries mirroring research_tier.rs:50-128 (8gb/12gb/12gb_7b/16gb/24gb/cloud)
- 17 contract tests
- Smokes: /task --surgical → SurgicalPlan complexity Simple/Medium + RegressionGate passes; /research ×2 → miss then hit; agent self-selects DKG tools

### C-31 — Code Completeness Layer 1: FilePlan + FileLineTracker
- Closes: guide/code-completeness-enforcement-design, guide/update-implementation-order, phase-4E
- Pre-reqs: C-13, C-9
- Parallel-safe with: C-32
- New file crates/core/src/file_plan.rs: FunctionSpec / FileSpec (path, language, max_lines, functions) / FileModSpec (max_added_lines) / FilePlan (new_files, modified_files, version) + validate() against LanguageLimitsMap + FilePlanError (LineBudgetExceeded / FunctionCountExceeded at MAX_FUNCTIONS_PER_FILE=20)
- Adds FILE_PLAN as 9th BabyStepSpec field with #[serde(default)] (no schema bump; backward-compatible)
- Adds FileLineTracker to TreeNavigatorState: cumulative_lines_by_step + step_write_generation u64 TOCTOU guard; record_lines_written returns Err(NavigatorError::WriteGeneration) on generation mismatch
- Adds MistakeRecorder trait (default no-op impl) wired into T1HookConfig + T2HookConfig; T1/T2 BLOCK events call record_mistake_event with FailureCategory::{LazyCode, EmptyBody, FileSizeExceeded}
- T2 Check 3: function-count guard against file_plan with FUNCTION_COUNT_SLACK=2 (per-file AST count > FileSpec.functions.len() + 2 → BLOCK)
- FileDisciplineConfig.language_limit_overrides populated from ProjectProfile.language_stats (≥5 detected files per language → production_warn = min(spec_default, project_median*1.5) etc.)
- 16 contract tests
- Smoke: `elai /task --mode cloud "implement a 3-file user CRUD module (user.rs, repo.rs, api.rs)"` → each BabyStepSpec has populated file_plan; T2 LazyCode → ≥0 MistakeBook entries; adversarial FILE_PLAN >400 LOC rejected before dispatch

### C-32 — Code Completeness Layer 2: Skeleton-First + MiniCritic + AcceptAndComplete
- Closes: guide/code-completeness-enforcement-design, guide/token-efficiency-design
- Pre-reqs: C-31, C-26
- Parallel-safe with: C-21..C-30 (disjoint file scope)
- New file skeleton_generator.rs: BlankId (u8 → "BLANK_{N}"), SkeletonSpec, FillResult, FilledBuffer (compute_completeness_ratio = non-BLOCK fills / blanks.len() clamped); SkeletonGenerator::generate(&FilePlan) signatures intact + bodies → "// BLANK_{N}"; fill_blanks runs MiniCritic per fill (zero disk writes); flush_to_disk = exactly one fs::write substituting BLANK_N
- New file mini_critic.rs: MiniCriticVerdict {PASS/WARN/BLOCK}; check order (first BLOCK short-circuits): (a) T1 regex subset (BORROWS LazyLock<Regex> statics from tiered_analysis.rs — no duplication), (b) line count < 3 → BLOCK "too_short", (c) tree-sitter fragment parse → ERROR node BLOCK; emits mini_critic_verdict SSB per call
- New file structural_retry.rs: StructuralRetryStage enum (Decompose / Template / FewShot / BestOfN) + StructuralRetryPlan + Sequencer.run_sequence (PASS→Converged; final fallback AcceptAndComplete if ratio ≥ 0.5 else Exhausted); BestOfN: N=3 candidates with score (compile+2, critic PASS+2, lines+1); NO INSTRUCTIONAL retry text ("try again" forbidden); emits structural_retry_stage_attempt SSB per stage
- New file bridge_template.rs: BridgeKind {ModRsAggregator / InitPyBarrel / IndexTsBarrel / GoPackageDoc} + BridgeFileTemplate.generate (deterministic, LLM-free, sorted lex, ELAI_SEED=42 byte-identical) + write_bridge (single fs::write); emits bridge_file_generated SSB
- Wires SkeletonGenerator + StructuralRetrySequencer into task_runner.rs baby-step dispatch (when FilePlan present from C-31)
- MonitorWindow completeness_ratio gauge (red<0.5, yellow 0.5-0.9, green ≥0.9) via get_completeness_ratio Tauri command; ReasoningDrawer mini_critic_verdict row renderer + "Show MiniCritic" toggle
- Registers 5 SSB action_types in ssb.rs: skeleton_generated, skeleton_flushed, mini_critic_verdict, structural_retry_stage_attempt, bridge_file_generated
- 14 contract tests
- Smoke: `elai /task --mode cloud "implement a 300-LOC Rust web server with 4 routes + middleware"` (ELAI_SEED=42) → completeness_ratio ≥0.9, zero BLANK_N remaining, cargo check passes; adversarial TODO fill → BLOCK → structural retry → convergence or AcceptAndComplete

### C-33 — Documentation and Phase Map Updates
- Closes: guide/update-phase-map, B-0_INDEX, B-39, B-40, B-21
- Pre-reqs: C-30 (Surgical/Oracle implementations referenced); C-22 (B-21 SSB reconciliation)
- Parallel-safe with: ALL (pure docs; the only code touch is failure_taxonomy.rs)
- NOTE: docs/specs/ is gitignored — new step files are local-only; flag at session end
- CREATES (local-only): phase-3L.txt (Project Intelligence 8-step spec), phase-3J-BASELINE.txt (Token Efficiency benchmark harness), phase-4D-SURGICAL.txt (SurgicalDecomposer spec — cross-refs C-30), phase-4D-ORACLE.txt (OracleAnswerCache spec)
- APPENDS (local-only): phase-3J.txt 3J.ADDENDUM (ETCLogitsProcessor / GG signals mean_logprob+entropy_mean / constrained decoding / tool routing A-26 / bash safety A-27 / git_commit A-28 / action classifier A-31 / steering vector batch A-8); phase-4G.txt 4G.ADDENDUM (AutoAdvanceEvaluator reasoning exhaustion + CloudSpendTracker escalation)
- Adds 3 FailureCategory variants (LazyCode, EmptyBody, FileSizeExceeded) to failure_taxonomy.rs + detect_category() + suggest_fix() arms; keeps ToolFailureCategory in sync (grep elai.map first)
- Reconciles B-39 plan: REFINE_CAP=2 → 3 (RefineBench arXiv:2511.22173 Table 3; contract test asserts ==3)
- Reconciles B-21 plan WIRING_CONTRACT action type names → match shipped code (system_probe, feasibility_report, novel_step_detected, stub_run); notes feasibility_escalated as DEFERRED until C-22
- Updates B-0_INDEX.md: status blocks for B-37/38/40/41/42; replaces "SCHEMA_VERSION=26 confirmed" with current 46 (cites progression history)
- Adds per-command KEEP/STRIP/DEFERRED [[file.commands]] sub-table to migration_manifest.toml (≥10 named entries; read commands.rs first — no fabricated names)

### C-34 — Phase 4J Empire Integration (DEFERRED)
- Closes: B-25, phase-4J
- Pre-reqs: EXTERNAL — Empire codebase access from upstream maintainer; elai_cc.toml [empire.codebase_path] confirmed
- Parallel-safe with: ALL other C-N (orthogonal). Does not block any other delivery.
- PLANNED scope (do NOT execute until external prerequisite met):
  - Source-grounding pass (paper-ingestor mode=source) → workflow/papers/empire-protocol-interface.md
  - elai_cc.toml [empire] section (codebase_path, pipe_name, connect_timeout_ms, brain_id)
  - Feature-gated crates/empire/ workspace member containing: FactoryLineTransport, FactoryLineClient, BrainManifest, EmpireEventBridge, ElaiNeuron, EmpireIssue, FactoryCmd
  - IpcMessageType::Empire variant (non-destructive extension)
  - CircularDep / HardImport / StdlibShadow repair pipeline (RepairStrategy + RepairPipeline)
  - 5 SSB action_types: empire_connected, empire_disconnected, empire_capability_invoked, empire_repair_started, empire_repair_complete (SCHEMA_VERSION bump required)
  - ElaiNeuron MCP HTTP endpoint registering BrainManifest
  - TDD tests 4J.3-4J.7 BEFORE production wiring 4J.8
  - --empire CLI flag
  - EmpireBridgePanel.svelte (connection status + brain_id + last capability + repair history)
- Estimated effort: 3 sessions once unblocked

## 4. B-N → C-N Closure Map (from C-0_INDEX)

| B-N | Closed by |
|---|---|
| B-1 | C-18 (contract test), C-8 (pagerank/AragOrchestrator) |
| B-2 | C-18 (fsm_phase_transition_contract) |
| B-3 | C-3, C-8, C-23 |
| B-4 | C-1, C-3, C-8 |
| B-5 | C-1, C-3, C-12 |
| B-6 | C-2, C-4 |
| B-7 | C-2, C-4, C-8 |
| B-8 | C-1, C-3 |
| B-9 | C-23 |
| B-10 | C-4 |
| B-11 | C-6, C-7 |
| B-12 | C-6, C-7 |
| B-13 | C-2, C-7 |
| B-14 | C-7 |
| B-15 | C-14, C-15 |
| B-16 | C-21 |
| B-17 | C-2, C-8 |
| B-18 | C-11 |
| B-19 | C-13 |
| B-20 | C-13 |
| B-21 | C-22 |
| B-22 | C-9, C-10 |
| B-23 | C-21 |
| B-24 | C-7, C-8 |
| B-25 | C-34 (DEFERRED) |
| B-26 | C-4, C-7 |
| B-27 | C-5 |
| B-28 | C-4 |
| B-29 | C-24 |
| B-30 | C-17 |
| B-31 | C-17 |
| B-34 | C-17 |
| B-35 | C-8, C-18 |
| B-36 | C-4, C-18 |
| B-37 | C-4 |
| B-38 | C-2, C-8 |
| B-39 | C-33 |
| B-40 | C-12, C-33 |
| B-41 | C-12 |
| B-42 | C-2, C-16 |
| B-0_INDEX | C-33 |

## 5. Guide → C-N Closure Map

| Guide spec | Closed by |
|---|---|
| compiled-pipeline-design | C-25 |
| token-efficiency-design | C-26 |
| pipeline-modes-design | C-30 |
| code-completeness-enforcement-design | C-31, C-32 |
| testing-trust-design | C-27 (T-(-1)/T-0), C-28 (T-1), C-29 (T-2/T-3) |
| anti-staleness-enforcement | C-21 |
| dashboard-design | C-6, C-7 |
| update-phase-map | C-33 |

## 6. Phase → C-N Closure Map

| Phase | Closed by |
|---|---|
| phase-0 | C-14, C-15 |
| phase-3B | C-18 |
| phase-3D | C-18 |
| phase-3H | C-18 |
| phase-3J | C-33 (addendum) |
| phase-3K | C-17 |
| phase-3L | C-33 (creates spec) |
| phase-4A | C-11, C-19 |
| phase-4B | C-20 |
| phase-4C | C-19, C-20 |
| phase-4D | C-30 (Surgical/Oracle), C-19 |
| phase-4D-SURGICAL | C-33 (creates spec; impl in C-30) |
| phase-4D-ORACLE | C-33 (creates spec; impl in C-30) |
| phase-4E | C-31 |
| phase-4F | C-22, C-19, C-20 |
| phase-4G | C-9, C-10, C-20, C-33 (addendum) |
| phase-4H | C-21 |
| phase-4J | C-34 (DEFERRED) |
| phase-4K | C-4, C-7, C-8 |
| phase-4M | C-4, C-20 |

## 7. SCHEMA_VERSION Bump Sites

- Current shipped: 46 (B-37/38/40/41/42 cumulative)
- C-1: bumps by exactly 1 (atomic for all 7 new variants) — earliest bump
- C-27: 46→47 (3 variants: WiringEvidence, WiringViolation, DeterminismDrift)
- C-28: 47 (MutationSurvived — shares wave bump)
- C-29: 47 (IntentConfirmed, MrViolated — shares wave bump)
- C-34: bump required once Empire ships (5 empire_* variants)

## 8. Cross-Cutting Hard Rules Across Sessions

- No .unwrap()/.expect() in production paths (test cfg + main entrypoint allowed)
- No InProcessToolDispatcher in non-test production after C-13
- No NoopSsbNotifier in production producer paths after C-1/C-8
- No hardcoded zero/mock returns in Tauri commands after C-3/C-4
- No kill_on_drop(true) in feasibility_probe after C-22 (explicit child.kill().await only)
- All new constants named (no magic numbers — see CLAUDE.md)
- blake3 for cache keys / fingerprints (never sha256/md5) — enforced in C-15/C-27/C-28
- FxHashMap over HashMap in production (C-15)
- ClockProvider injection over bare SystemTime::now/Instant::now (C-15)
- entropy-allow annotation required for unavoidable SystemTime usage (C-15)
- LOCKED_KEYS files cannot be committed without explicit override (C-21 pre-commit hook)
- AllowedConfigKey replaced by 12-variant ParameterKey enum (C-21 — compile-time exhaustiveness)
- AcceptAndComplete threshold = completeness_ratio ≥ 0.5 (C-32)
- AUDITING→MEMORIZING blocked by: mutation_score < threshold (C-28), MR aggregate (C-29), wiring audit (C-27)
- PLANNING→TESTING_FIRST blocked by: intent_pairs.len() < 3 (C-29)

## 9. Total Effort and Coverage Summary

- Total session slots: 59 (was 34; added C-35..C-59 on 2026-05-17)
- In-scope (executable now, CPU/cloud): 52 (C-1..C-33 + C-35..C-53)
- In-scope (RUNPOD-DEPENDENT): 6 (C-54..C-59 — flagship benchmarks)
- Deferred (external block): 1 (C-34)
- Files authored: 60 (incl. C-0_INDEX + this summary + C-35..C-59 added 2026-05-17)
- Estimated effort per implementation session (C-1..C-53): ~6-8 h
- Estimated effort per benchmark session (C-54..C-59): ~8-24 h wall-clock + RunPod cost
- Total estimated effort: ~370-500 h (excludes RunPod compute cost)
- Verification source for C-1..C-33: 103 reports (45 B-plan + 18 guide + 40 steps), 25 themes
- Citation source for C-35..C-59: 19 verified arXiv papers (2026-03-17..2026-05-17) + 7 competitor anchors (Devin, Aider, Cursor, Claude Code, OpenHands, mini-SWE-agent, GPT-5.4 / Composer 2)
- Verification status breakdown at C-1..C-34 generation: DONE 5, MOSTLY_DONE 77, PARTIAL 20, NOT_DONE 3, UNVERIFIABLE 0

## 10. Wave 7-9 Sessions (C-35..C-59) — Paper-Grounded Improvements + Benchmark Publication

Authored 2026-05-17. Sources: apps_workflow oracle pass 2026-05-17 (11 papers + 22 competitor capabilities) + Discord forum-dump (F:\.ELAI_workflow\.extras\discord-catchup-bot\forum-dump.md, 15 additional papers). All arXiv IDs verified via abs-page fetch.

### Wave 7 — Paper-grounded improvements, batch A

### C-35 — Tool Attention Dynamic Gating + Lazy MCP Schema Loading
- Cites: arXiv:2604.21816 (95% measured MCP-tax reduction; downstream quality figures are PROJECTIONS, not measured live)
- Pre-reqs: Phase 4K MCP bridge SHIPPED
- Parallel-safe with: C-36..C-40 (NOT C-42, overlapping surface)
- Adds: ToolGate, GatedCatalog, LazySchemaCache; per-FSM-state gate manifest at plugins/tool-gate.toml; SSB tool_gated/tool_eager/tool_lazy_resolved/tool_denied; ToolGateInspector.svelte; --tool-gate-off CLI flag
- Smoke: 30+ tool catalog; prompt_tokens_saved per feature 5-10K; SWE-bench delta ≤2pp vs --tool-gate-off

### C-36 — MUCOCO Mutation-Operator Catalog
- Cites: arXiv:2604.19086 ("15% inputs exposed inconsistencies")
- Pre-reqs: C-29 (testing-trust T2/T3) SHIPPED
- Parallel-safe with: C-35, C-37..C-53
- Adds: 9 MetamorphicRelation variants behind mucoco_operators feature; per-language applicability matrix; SSB mucoco_mutation_applied/_inconsistency_detected/_skip_reason
- Smoke: pilot detection-rate matches paper's 15% on ≥1 operator; existing 3,500+ test suite re-run produces detection table

### C-37 — PER_ROLE_DECODING_DEFAULTS Re-evaluation
- Cites: arXiv:2604.06066 (constrained decoding causes "structure snowballing" in reflection on 8B class)
- Pre-reqs: C-26 SHIPPED; RunPod for pilot
- Parallel-safe with: C-35, C-36, C-38..C-53
- Adds: structure_snowballing_harness; migration script for decoding defaults; possible LightlyConstrained DecodingMode; SSB decoding_mode_pilot_run/_default_flipped
- Smoke: pilot.md decision matrix applied; anti-rubber-stamp FINDING regression test passes under any mode

### C-38 — PageRank Production Contract + Aider Spec Accuracy Fix
- Source: prior-oracle 2026-05-17 finding — DETAILED_SUMMARY §10 mis-describes Aider repomap as file-level; verified symbol-level via direct read of aider/repomap.py
- Pre-reqs: B-1 (DKG runtime activation) confirmed SHIPPED
- Parallel-safe with: C-35..C-37, C-39, C-40
- Adds: pagerank_production_contract.rs; 50-file synth fixture with deliberate hub; aider-elai-pagerank-diff.md artifact; DETAILED_SUMMARY §10 corrected text; mistakes.md stale-external-reference row
- Smoke: contract test asserts compute_repo_map fires on production path; top-3 PageRank includes hub; absent-dkg triggers transparent rebuild

### C-39 — MCP Bridge CLI Surface Extension (OpenHands Parity)
- Source: OpenHands MCP integration (Apache 2.0, ~38K stars per prior oracle)
- Pre-reqs: C-35 SHIPPED (consumes gated catalog)
- Parallel-safe with: C-36..C-38, C-40
- Adds: MCPClient (cli/src/mcp_client.rs); McpClientDispatch + InProcessDispatch via Dispatch trait; --via-mcp CLI flag; CI parity job (every CLI test runs twice); SSB cli_mcp_dispatch/_server_started/_server_stopped
- Smoke: --via-mcp auto-launches co-process if no listener; CI parity green

### C-40 — GitHub PR Adapter (Devin Parity)
- Source: Devin 2.0 published autonomous PR + review-comment loop
- Pre-reqs: C-13 + C-23 SHIPPED
- Parallel-safe with: C-35..C-39
- Adds: crates/github_adapter/ (feature-gated); GhCliAdapter + RestApiAdapter; PrBodyComposer (includes mandatory "honesty section"); ReviewCommentWatcher + Router (default OFF auto-respond); --to-pr/--base/--watch-reviews/--draft flags; SSB 6 github_* action types
- Smoke: end-to-end draft PR open against test repo; body composed with mutation score + wiring contract + stub status + reviewer-instruction line

### C-41 — Skills-Driven Workflow Layer Formalization
- Cites: arXiv:2604.21910 (44%→83% measured intent accuracy)
- Pre-reqs: Phase 4A + prompt-engineer SHIPPED
- Parallel-safe with: C-35..C-40, C-42..C-53
- Adds: skills/ directory + registry.toml; SkillRegistry (trigram + sparse keyword); 9 initial skills (simple_query/focused_edit/heavy_plan/idea_intake/oracle/planning_workflow/paper_ingest/source_ingest/benchmark_run); SkillsPanel.svelte; SSB skill_lookup/_selected/_fell_through
- Smoke: SciQuest-style synthetic accuracy ≥73% (paper's 83% minus 10pp conservative)

### C-42 — Semantic Gateway Zero-Trust MCP Fuzzing
- Cites: arXiv:2604.25555 (100% unauthorized-state discovery via deterministic fuzzer)
- Pre-reqs: C-35 + C-39 SHIPPED
- Parallel-safe with: C-36..C-38, C-41, C-43..C-53 (NOT C-35, overlapping surface)
- Adds: AuthzPolicy + #[authz(states,actors)] tool attribute (DENY-ALL default for unlabeled); crates/mcp_fuzzer/ (200+ tuple corpus); FuzzReport (CRITICAL/P1/shape); AuthzPanel.svelte; CI fuzzer job (BLOCKING on CRITICAL); SSB mcp_authz_check/_reject/fuzzer_finding
- Smoke: known-bug fixture found by fuzzer; re-run after fix produces zero CRITICAL

### C-43 — Phase 4H 3-Layer Scope-Freeze Ratification
- Cites: arXiv:2604.21725 — AEL ablation: "adding mechanisms beyond memory+reflection consistently degrades performance"
- Pre-reqs: Phase 4H SHIPPED at current scope
- Parallel-safe with: all other C-N
- Adds: phase-4h-scope-policy.md (cites AEL verbatim); #[allow(self_mod_experimental, justify="...")] proc macro guardrail; ablation_harness.rs (delta vs baseline); synthetic DGM-like fixture surfaces negative delta; SSB self_mod_layer_proposed/_ablation_run; DETAILED_SUMMARY §Phase 4H footnote
- Smoke: ablation harness on synthetic DGM-like candidate produces NEGATIVE delta (proves harness can surface degradation)

### C-44 — Composer 2 Runtime Backend Tier
- Cites: arXiv:2603.24477 (SWE-bench Multilingual 73.7; $0.50/M input, $2.50/M output; built on Kimi K2.5)
- Pre-reqs: B-37 SHIPPED + C-39 SHIPPED + C-42 SHIPPED. GATED on API access verification.
- Parallel-safe with: C-35..C-43, C-45..C-53
- Adds: Composer2Backend impl of Backend trait; skill-routing (heavy_plan→Sonnet, focused_edit→Composer2, simple_query→local 7B); BackendCostPanel.svelte; SSB backend_composer2_invoked/_error/_cost_accrued
- Smoke: --enabled call routes through Composer 2; cost_accrued SSB rows with non-null usd_cents on every successful call

### Wave 8 — Paper-grounded improvements, batch B

### C-45 — CANTANTE Contrastive Credit Attribution
- Cites: arXiv:2605.13295
- Pre-reqs: 8 AgentRoles SHIPPED; pilot DEFERRED to RunPod
- Parallel-safe with: C-35..C-44, C-46..C-53
- Adds: AgentRoleDropper (per-role fallback table; no panic on drop); credit_attribution_harness; cantante-pilot.md per-role credit table; SSB agent_role_dropped/credit_attribution_run/_result
- Smoke: harness deterministic on synthetic fixture; pilot on 50-task SWE-bench subset (post-RunPod) → per-role credit table with CI

### C-46 — Uno-Orchestra Parsimonious Routing
- Cites: arXiv:2605.05007
- Pre-reqs: 8 AgentRoles SHIPPED; C-45 recommended
- Parallel-safe with: C-35..C-45, C-47..C-53
- Adds: TaskDifficultyClassifier (trigram + centrality + ref-task cosine); SelectiveDelegator (tier policy: <0.3→Executor+Auditor; 0.3-0.7→+CC+PE; ≥0.7→full chain); bubble-up on auditor "insufficient"; SSB selective_delegation_decision/_escalated/_cost_saved
- Smoke: pilot on 100-task SWE-bench subset → solve-rate within ±2pp, cost reduction ≥30%

### C-47 — Case-Based Calibration Tool-Use Telemetry
- Cites: arXiv:2605.15041
- Pre-reqs: C-35 + C-42 SHIPPED
- Parallel-safe with: C-46, C-48..C-53
- Adds: sled-backed CaseBase (LRU 100/tool); CaseRetriever (Jaccard + Levenshtein); hint injection on NEAR_MATCH parameter-rename; SSB case_base_consulted/_hint_emitted/_call_logged
- Smoke: 50-session replay shows ≥10% reduction in repeat-tool-errors

### C-48 — SafeHarbor Hierarchical Memory Guardrail
- Cites: arXiv:2605.05704
- Pre-reqs: memory-writer + C-42 SHIPPED
- Parallel-safe with: C-46, C-47, C-49..C-53
- Adds: TrustTier enum (T0 user-direct / T1 codebase-derived / T2 web-sourced / T3 model-emitted); MemoryView::T0_T1_only() filter for safety-critical queries; provenance propagation to prompts; migration script tags pre-C-48 rows as T3; SSB memory_tier_assigned/_query_filtered/safety_critical_blocked
- Smoke: synthetic prompt-injection fixture in T2 row does NOT influence safety-critical decision (4D.X.7)

### C-49 — Iterative Distillation of Experience for Prompt-Engineer
- Cites: arXiv:2605.14443 (most recent paper in batch)
- Pre-reqs: prompt-engineer SHIPPED; C-41 recommended
- Parallel-safe with: C-46..C-48, C-50..C-53
- Adds: ExperienceCorpus (per-project per-skill markdown at .elai/experience/{skill}.md); Distiller (5-gram shingling dedup); avoid-pattern integration from mistakes.md; token budget cap ~500; SSB experience_distilled/_consulted/_token_budget_exceeded
- Smoke: 10-session smoke on same codebase shows non-empty Distilled section

### C-50 — Contract-Coding Test Scaffolding
- Cites: arXiv:2604.13100 (47% functional success, near-perfect structural integrity on Greenfield-5)
- Pre-reqs: Phase 4F SHIPPED
- Parallel-safe with: C-46..C-49, C-51..C-53
- Adds: ContractSpec (preconditions + postconditions in target language); ContractEmitter runs post-FILE_PLAN pre-skeleton; ContractViolationCheck in MiniCritic; ContractViolationRetry; greenfield-init template enables contracts; SSB contract_emitted/_violation_detected/_retry_resolved
- Smoke: Greenfield-5 fixture reproduces ≥40% functional + ≥90% structural (conservative vs paper's 47%/near-perfect)

### C-51 — Aider Polyglot Harness Adoption (Test Infrastructure)
- Source: Aider MIT harness vendored at pinned SHA
- Pre-reqs: Phase 4F SHIPPED
- Parallel-safe with: C-46..C-50, C-52, C-53
- Adds: .elai_cc/external/aider_polyglot/ (vendored + 225 tasks); PolyglotAdapter; SandboxedTestRunner per language (cpp/go/java/js/python/rust); elai benchmark polyglot subcommand; SSB polyglot_task_started/_completed/_scorecard_emitted
- Smoke: dry-run 5 sample tasks against mock backend; full sweep refuses without GPU

### C-52 — Dafny Formal Verification Spike (RESEARCH-GRADE)
- Cites: arXiv:2604.22601 (Gemma 4-31B at 90.91% on NL2VC-60)
- Pre-reqs: C-48 (MemoryView candidate) recommended; Dafny toolchain
- Parallel-safe with: C-46..C-51, C-53
- Adds: 3 spike Dafny specs (FSM transition, MemoryView trust-filter, mutation aggregator); VerificationHarness; optional CI job; OR infeasibility note path with concrete blocker
- Smoke: 3 spike modules verify OR dafny-spike-infeasibility.md with ≥1 concrete blocker

### C-53 — NF4 Kernel Upstream Watch + In-Context Counter-Test
- Cites: arXiv:2604.02556 (NF4 kernel watch, passive; 2.0-2.2× kernel + 1.54× E2E speedup); arXiv:2604.27891 (counter-evidence: in-context prompting OBSOLETES orchestration for procedural tasks)
- Pre-reqs: none for watch; RunPod for counter-test pilot
- Parallel-safe with: all C-46..C-52
- Adds: .elai_cc/docs/upstream-watch.md (bitsandbytes tracked); cargo bench nf4_e2e; --baseline-single-prompt CLI flag; 30-task procedural subset fixture; SSB counter_test_run/procedural_routing_decision
- Smoke: bitsandbytes pinned in upstream-watch.md; counter-test pilot artifact authored post-RunPod

### Wave 9 — Public benchmark publication (RUNPOD-DEPENDENT)

### C-54 — SWE-bench Verified Run, 3 Configs (FLAGSHIP)
- Source: SWE-bench Verified is industry standard. ELAI has no published number per maintainer's honest assessment §8.
- Pre-reqs: C-21 corpus expansion (10→500 tasks) + C-35 + C-37 + C-39 + C-42 + C-44 (if hybrid C arm) SHIPPED. RunPod required.
- Parallel-safe with: C-55, C-56, C-57, C-58 (sequential on same RunPod)
- Adds: 500-task corpus at pinned SHA; 3 config TOMLs (A local-7B / B dual-local-7B / C hybrid Sonnet+Composer2+local); SwebenchHarness; 3 scorecard.json files; auto-generated methodology.md; publication results.md with mandatory honesty statement; SSB swe_bench_run_started/_task_completed/_scorecard_emitted/_methodology_locked
- Smoke: all 3 runs complete; no cherry-picking; methodology + results docs auto-generated from machine-readable inputs

### C-55 — Aider Polyglot Leaderboard Submission
- Source: aider.chat/docs/leaderboards/ — direct comparable to ollama qwen2.5-coder:32b at 72.9%
- Pre-reqs: C-51 SHIPPED (harness landed) + C-40 SHIPPED (PR adapter for submission). RunPod required.
- Parallel-safe with: C-54, C-56, C-57, C-58
- Adds: 2 config TOMLs (L local 32B, H hybrid); per-config scorecard.json; methodology + results docs; public DRAFT PR against Aider-AI/aider; SSB polyglot_submission_started/_completed/_pr_opened
- Smoke: 225-problem scorecards per config; comparison statement vs ollama 72.9% baseline + vs gpt-5 cost-per-task documented honestly

### C-56 — Codebase-Memory DKG 31-Repo Head-to-Head
- Cites: arXiv:2603.27277 (83% answer quality, 10× fewer tokens, 2.1× fewer tool calls on 31 real-world repos)
- Pre-reqs: B-29 + B-30 + C-38 SHIPPED. RunPod required.
- Parallel-safe with: C-54, C-55, C-57, C-58
- Adds: 31 vendored repos at pinned SHA; query corpus + ground-truth answers; CodebaseMemoryAdapter; AnswerQualityScorer (Sonnet 4.6 judge); hub-detection + caller-ranking specialized queries; per-repo + aggregate metrics; SSB codebase_memory_query_run/_answer_scored/_scorecard_emitted
- Smoke: ELAI's DKG matches or beats paper's 83% / 10× / 2.1× numbers; if not, improvement-target list in results.md

### C-57 — OpenHands Index + SWE-Cycle + SWE Atlas Combo
- Cites: openhands.dev/blog/openhands-index (Jan 2026), arXiv:2605.13139 (SWE-Cycle), arXiv:2605.08366 (SWE Atlas)
- Pre-reqs: C-54 SHIPPED (primary number first). RunPod required.
- Parallel-safe with: C-54, C-55, C-56, C-58
- Adds: 3 harness adapters (openhands_index/swe_cycle/swe_atlas); 3 scorecard.jsons; combined methodology + results doc; SSB multi_benchmark_run_started/_axis_completed/_scorecard_emitted
- Smoke: per-axis OpenHands Index scorecard; SWE-Cycle per-cycle solve-rate CURVE; SWE Atlas per-rubric scores

### C-58 — AgentForge 5-Role + RRK Reranker Head-to-Head
- Cites: arXiv:2604.13120 (AgentForge 40.0% SWE-bench Lite); arXiv:2604.26483 (RRK 8B 3-18× faster than 0.6-4B rerankers)
- Pre-reqs: C-45 recommended. RunPod required.
- Parallel-safe with: C-54, C-55, C-56, C-57
- Adds: vendored AgentForge at pinned SHA + LICENSE; AgentForgeAdapter; rrk_reranker feature + 8B loader; latency benchmark (p99 < 100ms gate); quality benchmark vs ELAI's ≤200M cross-encoder; SSB head_to_head_run_started/_arm_completed/reranker_eval_run/_decision
- Smoke: side-by-side scorecard vs AgentForge 40.0% baseline; RRK adoption decision config-gated (default unchanged)

### C-59 — Cross-Config Benchmark Master Report (CAPSTONE)
- Source: aggregator over C-54..C-58 + external baselines from prior oracle 2026-05-17. Optional TTS pilot (arXiv:2605.09329, 41% average acceptance-length improvement).
- Pre-reqs: C-54 + C-55 + C-56 + C-57 + C-58 all SHIPPED with completed runs.
- Parallel-safe with: none (consumes all benchmark outputs)
- Adds: MasterReportAggregator; external_baselines.toml (pins Devin/OpenHands/Aider/Claude Code/GPT-5.4/Composer 2/AgentForge/Codebase-Memory at version+date+URL); machine-regenerated benchmark_master_report.md with cross-config matrix + cross-competitor table + cost-per-task analysis + mandatory honesty section + improvement-target list; optional TTS pilot section; SSB master_report_aggregated/tts_pilot_run/_decision
- Smoke: report compiles deterministically; honesty + improvement-target sections present; "ELAI is competitive on X measurement at Y cost" claims become defensible

## 11. C-35..C-59 New SSB Action Types Summary

Wave 7 adds ~28 new action types: tool_gated/eager/lazy_resolved/denied (C-35), mucoco_mutation_applied/inconsistency_detected/skip_reason (C-36), decoding_mode_pilot_run/default_flipped (C-37), dkg_rebuild_started/complete + pagerank_run extension (C-38), cli_mcp_dispatch/server_started/server_stopped (C-39), 6 github_* (C-40), skill_lookup/selected/fell_through (C-41), mcp_authz_check/reject + fuzzer_finding (C-42), self_mod_layer_proposed/ablation_run (C-43), backend_composer2_invoked/error/cost_accrued (C-44).

Wave 8 adds ~21 new types: agent_role_dropped + credit_attribution_run/result (C-45), selective_delegation_decision/escalated/cost_saved (C-46), case_base_consulted/hint_emitted/call_logged (C-47), memory_tier_assigned/query_filtered/safety_critical_blocked (C-48), experience_distilled/consulted/token_budget_exceeded (C-49), contract_emitted/violation_detected/retry_resolved (C-50), polyglot_task_started/completed/scorecard_emitted (C-51), dafny_verify_run/pass/fail (C-52), counter_test_run + procedural_routing_decision (C-53).

Wave 9 adds ~16 new types: swe_bench_run_started/task_completed/scorecard_emitted/methodology_locked (C-54), polyglot_submission_started/completed/pr_opened (C-55), codebase_memory_query_run/answer_scored/scorecard_emitted (C-56), multi_benchmark_run_started/axis_completed/scorecard_emitted (C-57), head_to_head_run_started/arm_completed + reranker_eval_run/decision (C-58), master_report_aggregated + tts_pilot_run/decision (C-59).

Total new action types across Waves 7-9: ~65. SCHEMA_VERSION bumps spread across the 25 patches.

## 12. C-35..C-59 Paper Citations + Verification Status

All 19 papers verified via direct arXiv abs-page fetch by oracle researcher on 2026-05-17 (cache: oracle-elai-competitive-papers-2026-05-17 + oracle-elai-forum-dump-papers-2026-05-17). DGM (2505.22954, May 2025) and Live-SWE-agent (2511.13646, Nov 2025) OUT of window — excluded. 5 forum-dump papers excluded as not-applicable-to-ELAI by verifier (survey #3 / hardware-specific #4 #9 #12 / dominated #6).

| arXiv | Title | C-N |
|---|---|---|
| 2603.24477 | Composer 2 Technical Report | C-44 |
| 2603.27277 | Codebase-Memory | C-56 |
| 2604.02556 | Fast NF4 Dequantization Kernels | C-53 |
| 2604.06066 | Structure Snowballing (constrained decoding tax in reflection) | C-37 |
| 2604.13100 | Contract-Coding | C-50 |
| 2604.13120 | AgentForge | C-58 |
| 2604.19086 | MUCOCO | C-36 |
| 2604.21725 | AEL (anti-evidence) | C-43 |
| 2604.21816 | Tool Attention | C-35 |
| 2604.21910 | Skills-driven workflows | C-41 |
| 2604.22601 | Dafny NL2VC-60 | C-52 |
| 2604.25555 | Semantic Gateway (zero-trust fuzzing) | C-42 |
| 2604.26483 | RRK Listwise Reranker | C-58 |
| 2604.27891 | In-Context Prompting (counter-evidence) | C-53 |
| 2605.05007 | Uno-Orchestra | C-46 |
| 2605.05704 | SafeHarbor | C-48 |
| 2605.08366 | SWE Atlas | C-57 |
| 2605.09329 | Test-Time Speculation | C-59 (optional) |
| 2605.13139 | SWE-Cycle | C-57 |
| 2605.13295 | CANTANTE | C-45 |
| 2605.14443 | Iterative Distillation (most recent) | C-49 |
| 2605.15041 | Case-Based Calibration | C-47 |
