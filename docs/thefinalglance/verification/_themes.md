# Gap themes (clustered for C-plan session sizing)

The verification surfaced these recurring themes. Each theme becomes one or more C-N session-sized files in C-plan/. Each session = one cohesive area + dashboard wiring + a real cloud-Sonnet test as acceptance.

---

## Theme 1: SSB Action Type Completeness
Description: Roughly 25-30 SSB action type variants are defined in ssb.rs but never emitted from their intended producers, or are not yet defined at all. Many dashboard panels query SSB rows that never arrive, leaving the UI in permanent placeholder state.
Affected reports: B-3, B-4, B-5, B-7, B-8, B-13, B-17, B-21, B-23, B-38, B-41, B-42, phase-4G
Concrete actions (deduplicated):
- Add SsbActionType variants: GrcStepEmitted, GroundingSweepResult, CsqEvaluated, FeasibilityCheck, AuditorRubberStampRejected (crates/orchestrator/src/ssb.rs)
- Define phase_node_ready, step_node_ready, baby_step_ready, dag_repair_escalated in ssb.rs
- Emit real grc_step_emitted rows at all 11 emit_reasoning call sites in task_runner.rs (replace hardcoded constants with real budget/agent_id from PipelineGrc)
- Implement csq_evaluated SSB emit at task_runner.rs:5020 (TODO comment -> real emit)
- Add stagnation_spinning_detected, stagnation_oscillation_detected, lateral_thinking_activated, goal_drift_warning, watchdog_timeout SSB types (phase-4G)
- Emit CosmosRefreshStarted/CosmosArtifactUpdated/CosmosArtifactDecayed/CosmosGroundingUsed from cosmos_worker.rs
- Emit ambiguity_scored/ambiguity_floor_failed/ambiguity_streak_advanced from AmbiguityScorer (B-38)
- Emit real bubble_up_event rows so BubbleUpStrip.svelte shows real events
Estimated sessions to close: 2
Cloud-Sonnet test idea: Submit a /task via cloud Sonnet 4.6 OAuth; after completion assert that SSB rows for grc_step_emitted, csq_evaluated, phase_node_ready, ambiguity_scored, and at least one bubble_up_event exist in the SSB DB with non-null agent_id and budget_remaining fields.
Dashboard surface to wire: ReasoningDrawer (GRC budget bars), OrchestrationDrawer (ambiguity/stagnation rows), BubbleUpStrip (bubble_up_event), PlanWindow (phase/step/baby_step status rows)

---

## Theme 2: Missing Tauri Command Implementations
Description: Approximately 20 Tauri commands are registered in invoke_handler! or stub.ts but return hardcoded zeros, empty arrays, or Err. The dashboard cannot display live data until these are wired to real SSB viewer queries.
Affected reports: B-3, B-4, B-5, B-6, B-7, B-26, B-27, B-29, B-36, B-42, phase-4K
Concrete actions (deduplicated):
- Implement SsbViewer::cloud_spend_summary(); replace get_cloud_spend() stub at commands.rs:3306
- Implement get_query_trace_recent (B-4); set unwired.recentTraces=false in drawers.svelte.ts
- Add get_csq_log, get_feasibility_log, get_agent_grc_config commands (B-5)
- Add get_phase_detail, get_baby_step_detail (fix NotFound stub), get_executable_pipeline, get_research_node_history commands (B-7)
- Implement get_agents() with real SSB AgentLifecycle rows instead of hardcoded IDLE list (B-6)
- Add get_phase_progress, get_recent_actions, get_file_changes, get_action_detail, list_projects, create_project Tauri commands (phase-4K)
- Wire list_runtimes/test_runtime/set_role_runtime Tauri commands (phase-4M)
- Add get_pagerank_top, get_freshness_state, get_canary_drift Tauri commands (B-4)
- Resolve Tauri name mismatches: get_recent_evidence_gates->get_evidence_gates, get_active_agents->get_agents (B-6)
Estimated sessions to close: 2
Cloud-Sonnet test idea: After cloud /task completes, open the dashboard and assert that get_agents returns at least 1 entry with non-IDLE status, get_phase_progress returns a non-empty array, get_cloud_spend returns a non-zero value when Sonnet 4.6 was used.
Dashboard surface to wire: ActivityPanel (phase_progress, recent_actions), KnowledgeDrawer (pagerank_top, freshness_state, query traces), ReasoningDrawer (csq_log, grc_budgets per agent), MonitorWindow (live agent status, file changes)

---

## Theme 3: Cloud Mode Wiring Gaps
Description: Cloud mode has end-to-end OAuth proven (PROVEN per user), but key production-readiness gaps remain: spend cap enforcement is deferred, CLI --mode flag missing, CloudPanel.svelte absent, HYBRID escalation loop not wired.
Affected reports: B-27, guide/cloud-mode-design, guide/reasoning-architecture-design, guide/development-stones-analysis
Concrete actions (deduplicated):
- Implement hard ceiling enforcement in CloudModeRouter::route() using spend_cap_daily_usd/monthly (currently deferred at cloud_mode.rs:207)
- Implement warn_at_percent (80%) ceiling warning injection into COMMAND output
- Add CLI --mode cloud|hybrid|local flag to /task in crates/cli/src/args.rs; bridge to BackendConfig
- Wire HYBRID escalation retry loop in task_runner.rs (after local retries exhaust, emit EscalateToCloud)
- Create CloudPanel.svelte at dashboard-v2/src/lib/components/CloudPanel.svelte per WIRING_CONTRACT
- Implement pre-task cost estimation dialog (estimate tokens from plan skeleton, show "$X-$Y. Proceed?" before first cloud call)
- Add named baby-step LOC sizing constants (local_3b=25, local_7b=40, cloud_sonnet=300, cloud_opus=500)
- Add BYOE custom_endpoint support in CloudConfig (user-configured base_url)
- Create workflow/reasoning/B-27-decisions.md
Estimated sessions to close: 1
Cloud-Sonnet test idea: Run /task --mode hybrid with a task that exceeds local retries; assert SSB contains EscalateToCloud row and the response came from Sonnet 4.6; then set a spend_cap_daily_usd below the observed spend and assert the second /task is rejected with DailySpendCapExceeded.
Dashboard surface to wire: CloudPanel.svelte (spend gauge, mode selector, redaction log), CommandWindow (pre-task cost dialog), Sidebar (mode overlay INTERACTIVE/AUTONOMOUS toggle)

---

## Theme 4: Dashboard Design System and UX Completeness
Description: The CSS design token values in tokens.css are misaligned with the spec (bg-void, teal, text, semantic tokens all differ), fonts are wrong, font sizes off by 1-2px in many components, and several specified UI controls are absent.
Affected reports: B-11, B-12, B-13, B-14, B-26, guide/dashboard-design
Concrete actions (deduplicated):
- Align all --bg-* CSS tokens: bg-void=#0a0e12, bg-base=#0f1318, bg-surface=#151a20, bg-hover=#1a2028
- Align teal tokens: --teal=#1B7A6E, --teal-bright=#24a08e, --teal-soft=rgba(27,122,110,0.25)
- Align text tokens: --text-primary=#d0d7de, --text-secondary=#6e7681, --text-ghost=#383e47
- Fix body font: change --font-sans to JetBrains Mono; fix --font-display to Segoe UI Variable thin
- Sweep 11px occurrences in 7 Svelte files (ArtifactsPanel, IndexingProgressPanel, MediaValidationPanel, AntiStalenessDrawer, CosmosDrawer, KnowledgeDrawer, MonitorWindow); replace with 10px
- Reconcile sidebar icons: spec requires 5, current has 7; fix or document
- Fix Sidebar.svelte width: 64px -> 48px per spec
- Add Stop button to CommandWindow.svelte when commands.isStreaming is true
- Add Ctrl+Enter send binding in CommandWindow.svelte textarea
- Add Ctrl+Shift+A (accept-all) / Ctrl+Shift+R (reject-all) to shortcuts.ts
- Add responsive CSS breakpoints: hamburger <1000px, single-panel auto-collapse <800px
- Add per-hunk accept/decline controls to ReviewWindow.svelte
- Create ExploreWindow.svelte; register as WindowType 'explore' with Ctrl+8 shortcut
- Implement window recovery history (closedTabs stack, Ctrl+Shift+H recovery modal)
- Add visual regression test (visual.spec.ts with Playwright)
Estimated sessions to close: 2
Cloud-Sonnet test idea: Run a cloud /task via Sonnet 4.6 OAuth; observe the dashboard at runtime and assert that the BubbleUpStrip shows real events (not placeholder), the Stop button appears in CommandWindow when streaming, and OrbitShootingStarOverlay fires during the cloud execution phase.
Dashboard surface to wire: tokens.css, CommandWindow.svelte, ReviewWindow.svelte, Sidebar.svelte, ExploreWindow.svelte, shortcuts.ts, motion.css

---

## Theme 5: Stub Command and Viewer Eradication
Description: Multiple ssb_viewer.rs functions return hardcoded zeros or unimplemented placeholders. Related dashboard drawers show "unwired" placeholder badges. These stubs block real data flow throughout the UI.
Affected reports: B-3, B-7, B-38, B-42, phase-4K
Concrete actions (deduplicated):
- Implement pagerank_top SQL in ssb_viewer.rs:2699 (SELECT symbol, score FROM dkg_nodes_pagerank ORDER BY score DESC LIMIT ?)
- Implement query_tier_stats aggregate in ssb_viewer.rs:2708 from QueryTrace SSB rows
- Implement freshness_state real DB read in ssb_viewer.rs:2722
- Implement research_budget_snapshot() and source_tier_distribution() in ssb_viewer.rs (currently zeroes)
- Wire AmbiguityScorer into task_runner.rs before GroundingSweepGate fires; replace get_ambiguity_state mock
- Wire CosmosStage into QueryPipeline between ArtifactStage and ARagStage
- Wire CosmosWorker::tick() into task_runner.rs at hourly cadence
- Wire index_all_artifacts into IndexingService::build_index_resumable after code phase
- Implement TOOL_INDEX_ARTIFACTS in production_agent_rpc.rs (currently Err stub at line 1014)
Estimated sessions to close: 2
Cloud-Sonnet test idea: Index a real project, run /research via cloud Sonnet 4.6 OAuth; assert that get_query_trace_recent returns entries with non-null stage latencies, get_freshness_state returns a valid FreshnessState (not "UNVERIFIABLE"), and CosmosWorker status_snapshot() returns Fresh for at least one artifact after a tick().
Dashboard surface to wire: KnowledgeDrawer (IMPACT ANALYSIS, HyDE panel, DKG stats), RulesDrawer (rule_transition subscription), drawers.svelte.ts (set unwired flags to false)

---

## Theme 6: SSB Producer Wiring for Knowledge and Reasoning Pipelines
Description: The knowledge query pipeline, ARAG orchestrator, and reasoning engine emit telemetry via tracing::debug instead of SSB rows. The viewers that aggregate these into dashboard state find empty tables.
Affected reports: B-4, B-5, B-8, B-35, phase-3G, phase-3K
Concrete actions (deduplicated):
- Write query_trace SSB row in pipeline.rs after each query stage (replace tracing::debug with ssb.emit)
- Wire real SsbNotifier into live_cli.rs CrossEncoderReranker (replace NoopSsbNotifier)
- Add hyde_query SSB action type; emit from SidecarHydeClient (hypothesis text + cache hit + latency)
- Add vector_cache_hit_rate SSB action type; emit from vector indexer per-session
- Emit ArtifactsIndexed/ArtifactSearched/ArtifactStale SSB rows from context_artifacts pipeline
- Wire CoherenceReport violations to SSB FindingPayload writes after run_coherence_checks()
- Wire real query embedding (ONNX dot-product) into AragOrchestrator::semantic_search (replace embedding-norm proxy)
- Implement real AuditorRubberStampRejected emit in reasoning/src/engine.rs:329
- Wire GroundingSweepResult SSB emit; wire get_grounding_sweep to real SSB rows
Estimated sessions to close: 1
Cloud-Sonnet test idea: Run /research via cloud Sonnet 4.6; assert SSB DB contains query_trace rows with non-zero latency_ms per stage, at least one vector_cache_hit_rate row, and if an artifact is configured, at least one ArtifactSearched row.
Dashboard surface to wire: KnowledgeDrawer (query traces, HyDE preview), ActivityWindow (artifact event feed), OrchestrationDrawer (grounding sweep rows)

---

## Theme 7: Autonomous Mode and Anti-Spiral Infrastructure
Description: AutoAdvanceEvaluator, EscalationPolicy, StagnationDetection, GoalDriftCanary, and persona agent files are entirely absent. The autonomous mode (B-22/phase-4G) has only scaffolding types but no working execution path.
Affected reports: B-22, phase-4G
Concrete actions (deduplicated):
- Define AutoAdvanceEvaluator with gate_checks: HashMap<FsmTransition, EvidenceCheck> in autonomous_gate.rs
- Define EscalationPolicy struct (anti_spiral_threshold, high_risk_pauses, max_auto_advances_per_phase)
- Implement StagnationPattern enum + StagnationDetection + ExecutionHistory + ThinkingPersona + PersonaStrategy
- Implement GoalDriftCanary with Jaccard compute() and 3-band verdict (GREEN/YELLOW/RED)
- Implement GenerationWatchdog with WatchdogTimeoutKind enum + workflow_fingerprint deduplication
- Create 5 persona agent files: .claude/agents/persona-hacker/researcher/simplifier/architect/contrarian.md
- Wire TreeAutoAdvancePolicy into cli/main.rs production construction (currently autonomous_gate: None)
- Add [agents.{role}.backend] stanza to elai_cc.toml for local/api/hybrid per B-22
- Populate metrics/phase_4G.toml with runtime measurements (auto_advance_accuracy, mode_switch_latency_ms)
Estimated sessions to close: 2
Cloud-Sonnet test idea: Run a multi-step /task with --mode autonomous via cloud Sonnet 4.6; assert that at least one auto-advance occurs (FSM transitions without human input), the GoalDriftCanary produces a non-null verdict in SSB, and if a stagnation pattern is triggered, a persona switch SSB row is emitted.
Dashboard surface to wire: MonitorWindow (AutoAdvanceStats panel, ProposalsPanel), OrchestrationDrawer (stagnation/persona events)

---

## Theme 8: Sandbox and Process Isolation Completeness (Phase 4A/B-18)
Description: The Linux sandbox lacks setrlimit calls, hidepid mount, and capability drops. The Windows AppContainer path is untested. These are safety-critical gaps for Tier C agent execution.
Affected reports: B-18, phase-4A
Concrete actions (deduplicated):
- Implement setrlimit/prlimit64 calls in LinuxSandbox::launch_agent for CPU (60s/90s), memory (2GB), files (256), processes (32) (sandbox.rs)
- Mount /proc with hidepid=2 inside CLONE_NEWNS namespace
- Add PR_CAPBSET_DROP loop in clone_cb to drop all capabilities after PR_SET_NO_NEW_PRIVS
- Add RLIMIT_NPROC=1, RLIMIT_FSIZE=100MB, RLIMIT_AS=512MB setrlimit calls in sandbox.rs child setup
- Extend test_4a_isolation_pid_namespace_real: read /proc/{pid}/status; assert NSpid != 1
- Add Windows real isolation test using NtQueryInformationProcess to verify AppContainer SID
- Wire IpcHookDispatcher into run_agent_session in ipc_accept.rs
- Add test_4a_adv_ipc_u32_max_length_rejected (PayloadTooLarge before any allocation)
- Add adversarial tests: selfmod_path_traversal, bash_write_locked, git_commit_locked, symlink_resolved
Estimated sessions to close: 1
Cloud-Sonnet test idea: Submit a /task via cloud Sonnet 4.6 that includes a baby step triggering a Tier C sandboxed agent execution; assert the SSB contains AgentLifecycle rows for the spawned agent, the PID namespace boundary is verified (NSpid field), and no filesystem-escape attempts succeed (SSB FindingPayload with SANDBOX_VIOLATION should remain empty).
Dashboard surface to wire: MonitorWindow (agent isolation status indicator), OrchestrationDrawer (agent lifecycle rows)

---

## Theme 9: Phase 4G Autonomous Mode Extended Infrastructure
Description: The lateral-thinking, stagnation-detection, and anti-spiral machinery (4G.9) is entirely absent in code. These are the primitives required for the system to self-correct without human intervention.
Affected reports: phase-4G, B-22, guide/development-stones-analysis
Concrete actions (deduplicated):
- Implement 4G.9 test suite: stagnation_pattern >=4 tests, lateral_persona >=5 tests, generation_watchdog >=3 tests, goal_drift_canary tests
- Write 4G.7 dedicated tree auto-advance test suite (>=9 tests: baby-step/step/phase/concept, 50-step circuit breaker, bubble-up at 3 levels)
- Wire adaptive futility threshold: track per-error-category retry success rates in SSB; expose to repair-loop escalation
- Implement weaker evidence gates for zero-test codebases (detect via ProjectIntelligence; select relaxed gate variant)
- Add first-launch mode auto-selection: startup UX reads VramTier; recommends LOCAL/CLOUD/HYBRID
- Wire EvidenceCheck::expected_producer provenance flow using AgentPool::get_role()
Estimated sessions to close: 1
Cloud-Sonnet test idea: Run a /task where the first approach hits 3 identical errors; assert that a StagnationDetection SSB row appears, a persona switch event fires, the new persona produces a different approach, and the task ultimately completes rather than hitting the 50-step circuit breaker.
Dashboard surface to wire: MonitorWindow (stagnation events, lateral_thinking_activated rows), OrchestrationDrawer (goal_drift warnings)

---

## Theme 10: Per-Level Auditor Mode Dispatch Wiring (B-41)
Description: The per_level_audit_dispatch.rs module is complete and correct, but pipeline_tree_navigator.rs has zero calls to it. The 5 audit insertion points (post-Concepts, post-Phases, post-Steps, post-BabySteps, post-BabyStepDetail) are absent from the navigator.
Affected reports: B-41, phase-index-3A-4J
Concrete actions (deduplicated):
- Wire per_level_audit_dispatch::record_verdict into pipeline_tree_navigator.rs at 5 tier-completion points
- Confirm auditor.md MODE DISPATCH section has 6 mode-specific REVIEW prompt sections (~80 LOC each)
- Confirm FramedIdea is passed as context to mode=Concepts auditor invocation (B-40 integration)
Estimated sessions to close: 1
Cloud-Sonnet test idea: Submit a /task via cloud Sonnet 4.6 with enough concepts that the pipeline runs all 5 navigator tiers; assert that 5 distinct per_level_audit SSB rows appear (one per tier), each with a mode field corresponding to Concepts/Phase/Step/BabyStep/BabyStepDetail.
Dashboard surface to wire: OrchestrationDrawer (per-level audit verdict rows), MonitorWindow (audit mode indicator)

---

## Theme 11: IPC Tool Registry and Hook Dispatcher Production Wiring (B-19/B-20)
Description: IpcToolRegistry exists but InProcessToolDispatcher is still used in production. IpcHookDispatcher exists but hook_dispatcher.rs (InProcessHookDispatcher) has not been deleted and is not properly replaced in runtime_setup.rs.
Affected reports: B-19, B-20
Concrete actions (deduplicated):
- Wire IpcToolRegistry as production dispatcher in cli/src/run_subcommand.rs:149,221 (replace InProcessToolDispatcher)
- Delete/stub-ban InProcessToolDispatcher from production after IpcToolRegistry wired
- Wire IpcHookDispatcher to production in runtime_setup.rs (set hook_dispatcher field)
- Delete crates/orchestrator/src/hook_dispatcher.rs after migration
- Install RuffPostToolUseRunner into production hook dispatcher
- Implement AdaptiveTool<T> wrapper: auto-emit 10 remaining DAIRA markers (Mutation, Branch, Finding, Read, Write, ToolCallStart, ToolCallEnd, Timeout, Escalate, Converge)
- Implement WIP commit squash: >5 WIP commits -> 1 phase-level commit on AUDITING->MEMORIZING transition
Estimated sessions to close: 1
Cloud-Sonnet test idea: Submit a /task via cloud Sonnet 4.6 with multiple tool calls; assert that DAIRA markers [CALL]/[RETURN]/[MUTATION] appear in the trace log with correct structure, and that a RuffPostToolUse check fires on any Python file produced by the task.
Dashboard surface to wire: ActivityWindow (DAIRA marker feed), MonitorWindow (hook execution log)

---

## Theme 12: Phase 0 Debt Burndown (B-15/phase-0)
Description: ~1959 LOC of claw-code billing concepts survive (compact.rs, oauth.rs, prompt.rs), 50+ SystemTime::now() sites are not migrated to ClockProvider, and the HashMap ban is incomplete in 10 production files.
Affected reports: B-15, phase-0, B-0_INDEX
Concrete actions (deduplicated):
- Delete crates/orchestrator/src/compact.rs (624 LOC) after confirming zero production callers
- Delete crates/providers/src/oauth.rs (591 LOC) after confirming zero callers
- Delete crates/orchestrator/src/prompt.rs (744 LOC) after confirming role_prompt wired from config
- Add instant_now() to ClockProvider trait in core/src/clock.rs; replace SystemTime::now in ~50 files
- Replace std::collections::HashMap in 10 remaining production files (tiered_analysis.rs, import_extractor.rs, media_validation.rs, streaming.rs, mode.rs, self_discovery.rs, reranker.rs, session_dag.rs, self_mod_layer1a.rs, render_adapters/mod.rs)
- Re-enable entropy quarantine check in .github/workflows/windows.yml
- Strip UsageTracker/ModelPricing/format_usd/pricing_for_model from core/src/usage.rs
- Fix determinism_proof.sh: use --test seeded_determinism not --workspace
- Resolve SeededClock ELAI_SEED: clock always starts at tick 0 (only RNG seeded)
Estimated sessions to close: 2
Cloud-Sonnet test idea: After burndown, run cargo check --workspace and the entropy quarantine CI step; assert zero HashMap usage in the 10 target files, zero SystemTime::now in production src, and the seeded_determinism test produces byte-identical output across two runs with the same ELAI_SEED.
Dashboard surface to wire: None directly, but the entropy quarantine CI gate is the observable proof.

---

## Theme 13: Cosmos Artifact Tier Integration (B-42)
Description: CosmosWorker and CosmosStage types exist but are not wired into the production query pipeline or task scheduler. The 5 prescribed integration tests are also absent.
Affected reports: B-42, B-35
Concrete actions (deduplicated):
- Wire CosmosStage into QueryPipeline between ArtifactStage and ARagStage (knowledge/src/pipeline.rs §5)
- Wire CosmosWorker::tick() into task_runner.rs or Layer 1A scheduler at hourly cadence
- Create crates/knowledge/tests/cosmos_worker_tick.rs (~200 LOC)
- Create crates/knowledge/tests/cosmos_lock_contention.rs (~150 LOC)
- Create crates/knowledge/tests/cosmos_pipeline_stage.rs (~180 LOC, depends on CosmosStage wiring)
- Create crates/contracts/tests/cosmos_decay_contract.rs (~100 LOC)
- Create crates/orchestrator/tests/cosmos_briefing_budget.rs (~120 LOC)
- Clarify decay threshold: plan says 2x cadence, code uses 3x; align or document
Estimated sessions to close: 1
Cloud-Sonnet test idea: Configure [[knowledge.artifacts]] in elai_cc.toml with a live OpenAPI spec URL; run /research via cloud Sonnet 4.6; assert that CosmosWorker tick fires (SSB row CosmosRefreshStarted), the artifact appears as Fresh in status_snapshot(), and the briefing compiler includes the artifact in the context injected to Sonnet.
Dashboard surface to wire: ArtifactsPanel.svelte (CosmosDrawer) — render CosmosRefreshStarted/CosmosArtifactUpdated/CosmosArtifactDecayed events

---

## Theme 14: Knowledge Indexing Completeness and Resilience (B-30/B-31/B-31.5/B-34)
Description: Kotlin grammar is not registered; the resumable indexer is not wired to CLI startup; SIGINT handlers are missing; scale benchmarks have not run against real codebases; /research entry-point oracle pre-seeding is not wired.
Affected reports: B-30, B-31, B-31.5, B-34, phase-3K
Concrete actions (deduplicated):
- Add KotlinGrammar struct; register "kt"/"kts" in ext_to_lang_id and LanguageRegistry::default()
- Create crates/knowledge/tests/svelte_extraction.rs (composite-language re-parse)
- Wire auto_resume_indexed_projects into cli::main (crates/cli/src/main.rs)
- Install SIGINT/SIGTERM signal handlers calling graceful_shutdown
- Implement LockScope::Watch wiring in watcher service (Task 10 from B-31)
- Create tests/contracts/indexing_resilience_contract.rs
- Add Heading and Paragraph variants to ChunkKind in bm25.rs; implement heading/blank-line splitter
- Wire real query embedding into AragOrchestrator::semantic_search (dot-product vs norm proxy)
- Implement /research Oracle pre-seeding: call detect_entry_points in SlashCommand::Research arm
- Run scripts/scale_test/run_all_tiers.sh against real ELAI_SCALE_FIXTURE_ROOT; populate TOML metrics
- Implement per-batch p50/p99 timing in run_tier_a.sh (currently outputs 0)
Estimated sessions to close: 2
Cloud-Sonnet test idea: Index a real multi-language project (Rust+Kotlin+Svelte); run /research via cloud Sonnet 4.6; assert that the SSB contains DKG entries for .kt files, the QueryPipeline returns results with real embedding scores (not norm proxies), and the PROJECT_STARTING_POINTS block appears in the research query context.
Dashboard surface to wire: IndexingProgressPanel (Kotlin/Svelte DKG coverage), KnowledgeDrawer (semantic search quality, entry point list)

---

## Theme 15: Testing Infrastructure Gaps (contract tests, adversarial tests, benchmarks)
Description: Across many B-plan phases, the prescribed contract tests, adversarial tests, and benchmark TOML files either were never created or exist with placeholder/zero values. These are blockers for auditor gates.
Affected reports: B-2, B-5, B-7, B-29, B-32, B-33, B-35, B-36, phase-3B, phase-3D, phase-3H, phase-4A, phase-4B, phase-4C, phase-4F, phase-4G, phase-4H, phase-4I
Concrete actions (deduplicated):
- Add crates/contracts/tests/fsm_phase_transition_contract.rs (B-2)
- Create crates/orchestrator/tests/symbol_query_dispatch.rs (B-29)
- Add test_minified_file_emits_ssb_action to knowledge/tests/chunking_minified.rs (B-32)
- Create crates/orchestrator/tests/graph_export_dispatch.rs (B-33)
- Create crates/reasoning/tests/grounding_artifact_check.rs (B-35)
- Create crates/orchestrator/tests/designer_artifact_briefing.rs (B-35)
- Add contract tests per D-command (crates/contracts/tests/dashboard_v2_<cmd>_contract.rs) (B-36)
- Add test_payload_mismatch and determinism tests to ssb.rs (phase-3B)
- Add checkpoint_roundtrip test and 5-superstep integration test (phase-3D)
- Add 100-run determinism test and exact boundary test for session_dag (phase-3H)
- Add 15 adversarial tests for phase-4A (path traversal, u32::MAX frame, bash_write_locked etc.)
- Author 8 adversarial tests for phase-4F (A1-A8); author T9/T13-T16 contract tests
- Add 9 quantified metric sections to metrics/phase_4C.toml; create tiered_analysis_bench.rs
- Add regression-gate assertion in benchmark_eval_improved.rs (delta_pct >= -5.0)
- Populate metrics/phase_4G.toml, phase_4F.toml, phase_4B.toml, phase_4M.toml with real measurements
Estimated sessions to close: 3
Cloud-Sonnet test idea: After wiring, run cargo test -p elai_cc_contracts and assert all contract tests green including symbol_query_dispatch, fsm_phase_transition_contract, designer_artifact_briefing, and grounding_artifact_check. Then submit a cloud Sonnet 4.6 task to trigger one adversarial path (e.g., path traversal attempt) and confirm the SSB SANDBOX_VIOLATION row is absent.
Dashboard surface to wire: None directly; these are auditor-facing gates that feed into the GRC confidence metric visible in ReasoningDrawer.

---

## Theme 16: Self-Modification and Anti-Staleness Automation (B-16/B-23/phase-4H)
Description: The dep-audit, steering vector refresh, and nightly self-mod automation are documented but not connected to any trigger. The pre-commit hook for LOCKED_KEYS is absent. AllowedConfigKey uses a string enum not a closed enum.
Affected reports: B-16, B-23, phase-4H, guide/anti-staleness-enforcement
Concrete actions (deduplicated):
- Wire check_anti_stale.sh into GitHub Actions (add step to windows.yml or create anti-stale.yml)
- Execute B-16B Chunk C: ~12 .unwrap() sites in deterministic_auditor+dkg (requires user approval)
- Implement git-level enforcement: pre-commit hook rejecting commits modifying LOCKED_KEYS files
- Wire dep-audit output to FullPipelineOutcome.stale_api_catches (cli/src/dep_audit.rs -> self_mod_types.rs:185)
- Wire steering vector refresh after deprecated_patterns Layer 1A promote (trigger compute_steering_vector.py)
- Wire nightly dep audit trigger from Layer1A DIVERGING state in self_mod_layer1a.rs
- Replace AllowedConfigKey::Parameter(String) with closed enum of 12 named Parameter variants
- Implement ConstraintsBlock auto-population from rules.db deprecated_patterns table
- Create benchmarks/swe_bench/ with >=10 SWE-bench task fixtures; create trigger_task.toml canary
Estimated sessions to close: 1
Cloud-Sonnet test idea: Promote a deprecated_patterns entry via cloud Sonnet 4.6 task execution; assert that the self_mod_bridge triggers a compute_steering_vector.py run (file timestamp on modern_api.npz changes), the pre-commit hook blocks a commit to a LOCKED_KEYS file, and dep-audit catches at least one stale dep.
Dashboard surface to wire: MonitorWindow (ProposalsPanel, MsiPanel, nightly audit schedule display), AntiStalenessDrawer

---

## Theme 17: Feasibility Probe and Phase 4F Completeness
Description: Phase 4F (FeasibilityProber) has the core types but missing production wiring, struct fields, 8 adversarial tests, and 6 unmeasured metric fields. DefaultFeasibilityProber is never passed to runtime construction.
Affected reports: B-21, phase-4F
Concrete actions (deduplicated):
- Rename SystemProbeReport -> SystemProbeResult to satisfy DETERMINISTIC_CHECK grep
- Add GpuInfo.uuid field and shell-out parsing in system_probe.rs:72
- Add ResourceDelta struct (vram/ram before/after bytes) and wire into StubResult
- Add StubResult.alternative_index/stdout_hash/stderr_summary fields
- Wire production construction paths to pass real DefaultFeasibilityProber (cli/main.rs, runtime_setup.rs)
- Author 8 adversarial tests (A1-A8) per 4F.4 TASK in crates/contracts/tests/
- Author missing T9/T13-T16 contract tests
- Measure and populate 6 metric fields in phase_4F.toml (system_probe_latency_ms, stub_execution_p50/p99 etc.)
- Add FeasibilityPanel.svelte at dashboard-v2/src/lib/components/FeasibilityPanel.svelte
- Reconcile SSB action_type names (feasibility_probe_started vs system_probe) in B-21 WIRING_CONTRACT
Estimated sessions to close: 1
Cloud-Sonnet test idea: Submit a /research via cloud Sonnet 4.6 for a novel technology (triggers stub execution); assert that SSB contains feasibility_probe_started and feasibility_probe_complete rows, ResourceDelta fields are non-null, and FeasibilityPanel.svelte renders the probe result including probe_count and alternatives_explored.
Dashboard surface to wire: FeasibilityPanel.svelte (probe results, alternatives, system constraints), OrchestrationDrawer (feasibility events)

---

## Theme 18: SessionDag Branch UI and CLI Commands (B-9)
Description: The SessionDag producer side is complete but the branch-management CLI (/session merge, /session close) and the BranchTree.svelte mounting in any live screen are absent.
Affected reports: B-9, B-3
Concrete actions (deduplicated):
- Add /session merge <branch_id> to parse_session_command in cli/src/commands.rs
- Add /session close <branch_id> to parse_session_command
- Import and mount BranchTree.svelte in a live screen (e.g., SessionsDrawer)
- Wire BranchTree onfork/onmerge/onclose to dispatch CLI commands via IPC
- Wire /continue to actual SessionDag::load_handoff Tauri command (not stub)
- Verify /continue handler in CommandWindow.svelte triggers load_handoff
Estimated sessions to close: 1
Cloud-Sonnet test idea: Run a /task via cloud Sonnet 4.6 that runs long enough to create a handoff snapshot (>85% context budget); then run /continue in a new session and assert that the SessionDag::load_handoff fires, the BranchTree component shows the resumed branch with correct parent_chain, and the resumed task continues from the handoff point rather than restarting.
Dashboard surface to wire: SessionsDrawer (BranchTree.svelte mounted), CommandWindow (/continue confirmation)

---

## Theme 19: Symbol Impact API Dashboard Surface (B-29/B-33)
Description: The symbol-level impact API (symbol_impact/symbol_flow/symbol_context/symbol_list) and the interactive graph viewer (/graph mermaid) exist in orchestrator but are not exposed to agents via ToolCatalog and not surfaced in the dashboard.
Affected reports: B-29, B-33
Concrete actions (deduplicated):
- Add ToolDefinition entries for symbol_impact/symbol_flow/symbol_context/symbol_list to build_tool_catalog in production_agent_rpc.rs:312
- Wire update_changed_files_symbol_graph into indexing_service.rs incremental-update path
- Add /symbol_impact input form to KnowledgeDrawer.svelte IMPACT ANALYSIS panel
- Add /graph mermaid slash command to crates/cli/src/commands.rs
- Wire export_graph_mermaid as dispatcher tool in crates/orchestrator/src/dispatcher.rs
- Add decomposition-mode cytoscape style selectors to GraphWindow.svelte
- Add "Show blast radius" action to ActivityWindow.svelte file-write SSB row entries
- Create crates/orchestrator/tests/graph_export_dispatch.rs
Estimated sessions to close: 1
Cloud-Sonnet test idea: Submit a /task via cloud Sonnet 4.6 that modifies a widely-used symbol; assert that the agent's tool use includes a symbol_impact call, the response contains a non-empty blast_radius list, and GraphWindow renders the affected nodes correctly.
Dashboard surface to wire: KnowledgeDrawer (IMPACT ANALYSIS panel with /symbol_impact form), GraphWindow (decomposition-mode styles), ActivityWindow (blast-radius action on file-write rows)

---

## Theme 20: Compiled Pipeline and Token Efficiency Infrastructure
Description: CompiledPipeline/TemplateCrystallizer/BackgroundTaskManager and constrained decoding infrastructure are entirely absent despite being required by multiple guide specs (compiled-pipeline-design, token-efficiency-design, post-phase4-implementation-order).
Affected reports: guide/compiled-pipeline-design, guide/token-efficiency-design, guide/post-phase4-implementation-order
Concrete actions (deduplicated):
- Create CompiledPipeline/CompiledStep/TemplateVariable types in orchestrator/src/
- Create TemplateCrystallizer with N=3 consecutive success threshold monitoring
- Create CompiledExecutor with decompilation logic (2 consecutive failures OR confidence <0.7)
- Create BackgroundTaskManager with soft-timeout auto-detach; add BACKGROUND_TASK_STARTED/COMPLETE SSB types
- Define ConstrainedDecodingConfig struct and DecodingMode enum in providers crate
- Define RetryStrategy struct and wire to BubbleUpPolicy (temperature_schedule, futility_same_error_threshold)
- Implement futility detection in NavigatorTree (same-error-class 3x, output-similarity >90%)
- Create CumulativeTokenTracker; wire into cloud provider response parsing; forward prompt_cache_read_tokens to context_budget
- Fix benchmark harness fixture bug (measurements 3/4/5 report 0%); re-run and update model_baseline.toml
Estimated sessions to close: 2
Cloud-Sonnet test idea: Run a repeated /task pattern via cloud Sonnet 4.6 where the same task succeeds 3 consecutive times; assert that TemplateCrystallizer fires and stores a CompiledPipeline entry. Then run the same task again and assert that the CompiledExecutor uses the template without redecomposing. Verify CumulativeTokenTracker reports lower total tokens on the compiled run.
Dashboard surface to wire: MonitorWindow (CompiledPipeline indicator), ReasoningDrawer (token efficiency delta, futility detection events)

---

## Theme 21: Phase 4G / Empire Integration (B-25/phase-4J) — Out-of-Scope Until Empire Available
Description: Empire Integration (B-25/phase-4J) requires access to an external codebase (Empire MKI/MKII/MKIII). This theme is blocked by access, not implementation complexity. Defer until Empire codebase is accessible.
Affected reports: B-25, phase-4J
Concrete actions (deduplicated):
- Confirm Empire codebase accessible at elai_cc.toml [empire.codebase_path] before any work begins
- Once accessible: create crates/empire/ as feature-gated workspace member
- Implement FactoryLineTransport, FactoryLineClient, BrainManifest, EmpireEventBridge, ElaiNeuron, EmpireIssue
- Write 4J.3-4J.7 tests first (TDD); implement 4J.8 after tests defined
Estimated sessions to close: 3 (after access granted)
Cloud-Sonnet test idea: Once Empire accessible, connect ElaiNeuron MCP server; submit a task via cloud Sonnet 4.6 that triggers an Empire capability invocation; assert SSB contains empire_connected and empire_capability_invoked rows with non-null brain_id.
Dashboard surface to wire: EmpireBridgePanel.svelte (connection status, capability invocations)

---

## Theme 22: Testing Trust Infrastructure (guide/testing-trust-design) — Large Scope
Description: The 14 testing-trust tiers (T-(-1) through T-13) are entirely unimplemented. This is the largest single un-shipped design block in the system. Implementing them in order unlocks mutation testing, intent pairs, metamorphic testing, provenance logging, and formal verification hooks.
Affected reports: guide/testing-trust-design, guide/post-phase4-implementation-order
Concrete actions (deduplicated):
- Implement T-(-1): DeterminismConfig + hardware-class lock + blake3 prefix-cache keys + DETERMINISM_DRIFT SSB type
- Implement T-0: WiringContract type + StubBanAuditor + DkgReachabilityAuditor + ObservableEffectRunner + WiringEvidence/WiringViolation SSB types + EntrypointSetLoader
- Implement T-1: MutationRunner wrapping cargo-mutants (Rust) + mutmut (Python); add Mutation tier to ValidationTier
- Implement T-2: IntentPair/IntentPairSet/IntentConfirmation; INTENT_CONFIRMED SSB type; PLANNING->TESTING_FIRST gate requiring >=3 pairs
- Implement T-3 through T-13 progressively (metamorphic relations, risk classification, EquivalentMutantCache, RegressionCorpusManager, FormalVerifierHook, ProvenanceLog, CiteAudit, PRPO hybrid MSI, Rulebreakers Paired-Accuracy, XGrammar, FACET R-DAG)
- Add [wiring.entrypoints] section to elai_cc.toml with closed production entrypoint set
- Add elai wiring audit CLI subcommand
Estimated sessions to close: 8 (one tier per session minimum; T-0 and T-1 are 2-session tiers)
Cloud-Sonnet test idea: After T-0 ships, run elai wiring audit via a cloud Sonnet 4.6-powered session; assert that WiringEvidence SSB rows appear for each entrypoint, no WiringViolation rows appear for the core orchestrator, and the StubBanAuditor catches at least one known stub in the codebase.
Dashboard surface to wire: MonitorWindow (wiring audit results, mutation score gauge), ReasoningDrawer (provenance log, citation audit trail)

---

## Theme 23: Surgical Mode, Oracle Mode, and Pipeline Mode Infrastructure
Description: SurgicalDecomposer, SurgicalPlan, project detection signal logic, Oracle Mode answer cache, and pipeline mode switching are absent. These are prerequisite for intelligent task routing.
Affected reports: guide/pipeline-modes-design, guide/update-implementation-order, phase-4D
Concrete actions (deduplicated):
- Implement SurgicalPlan/AffectedSet/ComplexityClass/RegressionGate/TestBaseline types
- Implement SurgicalDecomposer: DKG blast radius traversal; constrained decomposition
- Wire surgical flag from parse_task_flags through task_runner dispatch
- Add project detection signal logic (weighted-signal table) and first-task indexing trigger
- Add Oracle Mode answer-level cache with tier-based TTL (T1=30d, T2=7d, T3=1d)
- Add blast radius LOC threshold gates with user confirmation for autonomous mode
- Expose DKG query tools to agents (dkg_callers/dkg_dependents/dkg_data_flow/dkg_subgraph)
- Add [research.vram_tiers] TOML section to elai_cc.toml with all 5+ tier entries
Estimated sessions to close: 2
Cloud-Sonnet test idea: Submit a bug-fix /task --mode surgical via cloud Sonnet 4.6 targeting a single well-known function; assert that the SurgicalDecomposer produces a plan with blast radius <= 3 files, the AffectedSet is populated from DKG, and the RegressionGate passes after the fix without touching unrelated files.
Dashboard surface to wire: PlanWindow (surgical mode indicator, blast radius badge), KnowledgeDrawer (DKG query results from agent tool calls)

---

## Theme 24: Code Completeness Enforcement (guide/code-completeness-enforcement-design)
Description: FilePlan/FileSpec as the 9th BabyStepSpec block, FileLineTracker, skeleton-first generation, MiniCritic, BridgeFileTemplate, and AcceptAndComplete recovery are all absent. These prevent ELAI from enforcing code completeness at generation time.
Affected reports: guide/code-completeness-enforcement-design, guide/update-implementation-order, phase-4E
Concrete actions (deduplicated):
- Define FilePlan/FileSpec/FileModSpec/FunctionSpec structs; add FILE_PLAN as 9th BabyStepSpec block in core or orchestrator
- Implement FileLineTracker in TreeNavigator with cumulative line count per baby step (TOCTOU prevention)
- Implement skeleton-first generation with fill protocol (SkeletonGenerator, BLANK_N placeholders, in-memory buffer, single disk write)
- Implement MiniCritic between fills (tree-sitter fragment parse, >3 lines check, T1 regex subset)
- Implement AcceptAndComplete recovery (completeness_ratio >= 0.5 threshold)
- Implement structural retry sequence (decompose -> template -> few-shot; no instructional retry)
- Implement BridgeFileTemplate (language-parameterized, deterministic orchestrator-written bridges)
- Wire MistakeBook recording from T1/T2 BLOCK events (LazyCode, EmptyBody, FileSizeExceeded) into SSB
Estimated sessions to close: 2
Cloud-Sonnet test idea: Submit a /task via cloud Sonnet 4.6 that produces a large file (>300 LOC target); assert that the skeleton-first flow is triggered (SkeletonGenerator SSB row), at least one MiniCritic evaluation fires, and the final file has completeness_ratio >= 0.9 with no BLANK_N placeholders remaining.
Dashboard surface to wire: PlanWindow (FILE_PLAN block display per baby step), MonitorWindow (completeness_ratio gauge, MiniCritic verdict rows)

---

## Theme 25: Documentation and Phase Map Updates (guide/update-phase-map + misc)
Description: Several step files are missing (phase-3L.txt, phase-4D-SURGICAL.txt, phase-4D-ORACLE.txt) and 10+ existing step files need spec updates to match shipped implementations (phase-1A, phase-3J, phase-4B, phase-4C, phase-4D, phase-4E, phase-4G, phase-4H).
Affected reports: guide/update-phase-map, B-0_INDEX, B-39, B-40
Concrete actions (deduplicated):
- Create steps/phase-3L.txt (8-step Project Intelligence spec)
- Create steps/phase-3J-BASELINE.txt (Token Efficiency benchmark harness spec)
- Create steps/phase-4D-SURGICAL.txt (SurgicalDecomposer spec)
- Create steps/phase-4D-ORACLE.txt (Oracle Mode /research pipeline spec)
- Update B-0_INDEX.md: add status blocks for B-37/38/40/41/42; update SCHEMA_VERSION to 46
- Update steps/phase-3J.txt: add ETCLogitsProcessor/GG signals/constrained decoding/tool routing/steering vector batch mode
- Update steps/phase-4G.txt: add AutoAdvanceEvaluator reasoning exhaustion check, CloudSpendTracker escalation policy
- Add LAZY_CODE/EMPTY_BODY/FILE_SIZE_EXCEEDED to FailureCategory enum in tools/src/failure_taxonomy.rs
- Reconcile B-39 REFINE_CAP (plan=2, code=3); reconcile B-21 SSB action type names with spec
- Add per-command KEEP/STRIP breakdown for 28+ slash commands to migration_manifest.toml
Estimated sessions to close: 1
Cloud-Sonnet test idea: Not applicable directly. Correctness is verified by spec_link_check.py passing for all updated step files and wiring_contract_check.py finding no WIRING_DIVERGENCE entries for the reconciled B-21/B-39 contracts.
Dashboard surface to wire: None directly; these are spec documents consumed by context-compiler and auditor agents.
