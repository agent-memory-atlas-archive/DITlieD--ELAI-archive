# C-plan: ELAI completion roadmap (post-verification)
Generated: 2026-05-17
Source: docs/thefinalglance/verification/ (103 reports, 25 themes clustered into 33 sessions + 1 deferred)
Premise: cloud mode is PROVEN end-to-end via OAuth + Sonnet 4.6. Each C-N session ships one cohesive area, wires it to the dashboard, and proves it with a real cloud-Sonnet test as ACCEPTANCE.

## Reading order

The C-N files are roughly ordered by dependency. Pre-requisites are listed in each file's "Pre-requisites" line. Most can run in parallel within a wave.

## Status legend
- PLANNED — file written, work not started
- IN PROGRESS — currently being shipped in an active session
- DONE — shipped and verified by cloud-Sonnet smoke test
- DEFERRED — blocked on external dependency
- NOT YET AUTHORED — session slot reserved; file not yet written

## Sessions

### Wave 1 — Foundations (no pre-requisites; parallel-safe)

- [C-1](C-1_ssb_emission_knowledge_reasoning.md) — SSB emission pass 1: knowledge + reasoning producers — PLANNED
  Closes gaps: B-4, B-5, B-8. Adds GrcStepEmitted, GroundingSweepResult, CsqEvaluated, AuditorRubberStampRejected, HydeQuery, VectorCacheHitRate, QueryTrace SSB variants.
  Pre-reqs: none (B-4/B-5/B-8 already shipped).
  Cloud test: /task --mode cloud asserts ≥5 of 7 new SSB types fire with non-null payloads.

- [C-6](C-6_dashboard_design_tokens.md) — Dashboard design token alignment — PLANNED
  Closes gaps: B-11, B-12, guide/dashboard-design. Aligns all --bg-*, teal, text CSS tokens; fixes fonts; Sidebar 64→48px.
  Pre-reqs: none (pure dashboard-v2 CSS/Svelte; no backend changes).
  Cloud test: open dashboard after cloud /task; assert token values match spec via computed CSS.

- [C-14](C-14_phase0_debt_dead_code_strip.md) — Phase 0 debt part 1: dead code strip (~1959 LOC) — PLANNED
  Closes gaps: B-15, phase-0. Deletes compact.rs (624 LOC), oauth.rs (591 LOC), prompt.rs (744 LOC); strips banned symbols from usage.rs; fixes determinism_proof.sh Windows-shell mismatch.
  Pre-reqs: none directly.
  Cloud test: cargo check --workspace PASS; entropy quarantine CI step passes after dead-code removal.

### Wave 2 — Data plane and infra (depend on Wave 1)

- [C-2](C-2_ssb_emission_orchestration.md) — SSB emission pass 2: orchestration + cosmos + ambiguity + stagnation + anti-spiral — PLANNED
  Closes gaps: B-6, B-7, B-13, B-38, B-42, phase-4G. Adds stagnation_*, lateral_thinking_*, cosmos_*, ambiguity_scored, bubble_up_event SSB types.
  Pre-reqs: C-1 (establishes SsbActionType variant pattern and task_runner emit sites).
  Cloud test: /task --mode cloud asserts stagnation, cosmos, ambiguity, and bubble_up rows present in SSB DB.

- [C-3](C-3_tauri_commands_knowledge_surface.md) — Tauri commands: knowledge dashboard surface — PLANNED
  Closes gaps: B-3, B-4, B-5. Implements get_query_trace_recent, get_pagerank_top, get_freshness_state, get_canary_drift, get_csq_log, get_feasibility_log, get_agent_grc_config Tauri commands.
  Pre-reqs: C-1 (QueryTrace/CsqEvaluated SSB rows must exist before viewer queries them).
  Cloud test: dashboard KnowledgeDrawer and ReasoningDrawer show non-empty live data after cloud /task.

- [C-4](C-4_tauri_commands_orchestration_cloud.md) — Tauri commands: orchestration + cloud spend surface — PLANNED
  Closes gaps: B-6, B-7, B-26, B-27, B-36, phase-4K. Wires get_agents live, get_cloud_spend, get_phase_detail, get_baby_step_detail, get_executable_pipeline, get_phase_progress, get_recent_actions, list_runtimes, set_role_runtime.
  Pre-reqs: C-1 (AgentLifecycle SSB rows required for get_agents live query).
  Cloud test: dashboard MonitorWindow returns ≥1 non-IDLE agent and non-zero cloud spend after cloud /task.

- [C-7](C-7_dashboard_ux_completeness.md) — Dashboard UX completeness — PLANNED
  Closes gaps: B-11, B-12, B-13, B-14, B-26. Sweeps 11px→10px in 7 Svelte files; adds Stop button, Ctrl+Enter, Ctrl+Shift+A/R shortcuts; responsive breakpoints; ExploreWindow.svelte; window recovery history; per-hunk accept/decline in ReviewWindow.
  Pre-reqs: C-6 (design tokens aligned for visual baselines), C-2 (DispatchEditHunks/FileWrite SSB rows wired).
  Cloud test: cloud /task streams to CommandWindow; Stop button visible; BubbleUpStrip shows real events.

- [C-8](C-8_viewer_stub_eradication.md) — Viewer stub eradication: remaining producers — PLANNED
  Closes gaps: B-3, B-4, B-5, B-7, B-38, B-42, phase-4K. Wires AmbiguityScorer, CosmosWorker::tick, TOOL_INDEX_ARTIFACTS, real SsbNotifier in CrossEncoderReranker, ONNX dot-product in AragOrchestrator, CosmosStage into QueryPipeline; clears unwired flags in drawers.svelte.ts.
  Pre-reqs: C-1 (SSB variant definitions), C-2 (cosmos/ambiguity SSB types defined).
  Cloud test: get_query_trace_recent returns non-zero stage latencies; CosmosWorker status shows Fresh.

- [C-9](C-9_autonomous_infrastructure.md) — Autonomous mode infrastructure — PLANNED
  Closes gaps: B-22, phase-4G (7.x). Defines AutoAdvanceEvaluator, EscalationPolicy; wires TreeAutoAdvancePolicy into cli/main.rs production; adds [agents.{role}.backend] stanza; populates phase_4G.toml metrics.
  Pre-reqs: C-1 (gate_decision SSB rows required for smoke test).
  Cloud test: /task --mode autonomous asserts ≥1 auto-advance FSM transition without human input.

- [C-11](C-11_sandbox_completeness.md) — Sandbox completeness (Tier C isolation) — PLANNED
  Closes gaps: B-18, phase-4A. Adds setrlimit/prlimit64 calls, hidepid=2 mount, PR_CAPBSET_DROP loop, IpcHookDispatcher into ipc_accept.rs; real NSpid isolation test; Windows AppContainer SID test; 5 missing 4A.4 adversarial tests.
  Pre-reqs: C-9 (multi-agent spawn needed for smoke test executor role).
  Cloud test: cloud /task with Tier C baby step; SSB contains AgentLifecycle rows; no SANDBOX_VIOLATION.

- [C-12](C-12_per_level_auditor_navigator_wiring.md) — Per-level auditor navigator wiring — PLANNED
  Closes gaps: B-41, phase-index-3A-4J. Wires per_level_audit_dispatch::record_verdict at 5 navigator tier-completion points; confirms auditor.md 6 mode-specific REVIEW sections; connects get_per_level_verdicts to live SSB rows.
  Pre-reqs: C-1, C-2 (SSB action types defined and emitting; B-41 shipped type system).
  Cloud test: cloud /task with ≥5 tiers asserts 5 distinct per_level_audit SSB rows, one per mode.

- [C-15](C-15_phase0_debt_clock_entropy.md) — Phase 0 debt part 2: ClockProvider + HashMap ban + entropy quarantine — PLANNED
  Closes gaps: B-15, phase-0. Migrates ~68 SystemTime::now sites to ClockProvider; adds instant_now(); replaces HashMap in 10 production files with FxHashMap; re-enables entropy quarantine CI.
  Pre-reqs: C-14 (dead-code strip removes HashMap-bearing dead functions first).
  Cloud test: cargo test seeded_determinism produces byte-identical output across two ELAI_SEED runs.

### Wave 3 — Integration and test infra (depend on Wave 2)

- [C-10](C-10_anti_spiral_primitives.md) — Phase 4G.9 anti-spiral primitives — PLANNED
  Closes gaps: phase-4G (9.x), B-22. Implements StagnationPattern, StagnationDetection, ExecutionHistory ring buffer, ThinkingPersona, PersonaStrategy, GoalDriftCanary (Jaccard), GenerationWatchdog (blake3), AdaptiveFutilityTracker; creates 5 persona agent files; adds OrchestrationDrawer StagnationSection.
  Pre-reqs: C-9 (AutoAdvanceEvaluator + EscalationPolicy ship first), C-2 (stagnation_* SSB types defined).
  Cloud test: cloud /task hitting 3 identical errors triggers stagnation SSB row + persona switch event.

- [C-13](C-13_ipc_dispatcher_production_swap.md) — IPC dispatcher production swap — PLANNED
  Closes gaps: B-19, B-20. Replaces InProcessToolDispatcher with IpcToolRegistry in run_subcommand.rs; installs IpcHookDispatcher + RuffPostToolUseRunner; implements AdaptiveTool<T> for 10 DAIRA markers; WIP commit squash on AUDITING→MEMORIZING.
  Pre-reqs: C-11 (sandbox/agent process boundary solid before IPC swap).
  Cloud test: cloud /task with Python tool calls; DAIRA markers appear in trace log; Ruff check fires.

- [C-16](C-16_cosmos_pipeline_integration.md) — Cosmos pipeline integration — PLANNED
  Closes gaps: B-42, B-35. Wires CosmosStage into QueryPipeline between ArtifactStage and ARagStage; drives CosmosWorker::tick from task_runner.rs hourly; implements trigger_cosmos_refresh; replaces tracing-only events with real SSB rows; 5 missing standalone test files.
  Pre-reqs: C-2 (cosmos_* SSB types defined), C-8 (CosmosStage pipeline stub created).
  Cloud test: real OpenAPI artifact; cloud /research asserts CosmosRefreshStarted SSB row + Fresh status.

- [C-17](C-17_knowledge_indexing_resilience.md) — Knowledge indexing resilience — PLANNED
  Closes gaps: B-30, B-31, B-31.5, B-34, phase-3K. Adds Kotlin grammar; wires auto_resume, SIGINT/SIGTERM handlers; LockScope::Watch; Heading/Paragraph chunking; real ONNX query embeddings; Oracle pre-seeding; scale benchmark population.
  Pre-reqs: C-15 (clock/entropy stable), C-14 (dead code strip).
  Cloud test: cloud /research on Rust+Kotlin+Svelte project; DKG has .kt entries; ONNX scores non-zero.

- [C-18](C-18_test_infra_backfill_contracts.md) — Test infra backfill: contract tests — PLANNED
  Closes gaps: B-2, B-29, B-32, B-33, B-35, B-36, phase-3B, phase-3D, phase-3H. Creates all contract/integration test files specified in B-plan but never authored (zero production code changes).
  Pre-reqs: C-1 (path unification stable), C-2 (SSB types stable), C-13 (IPC dispatcher swap stable).
  Cloud test: cargo test -p elai_cc_contracts GREEN; all new contract tests pass.

- [C-19](C-19_test_infra_backfill_adversarial.md) — Test infra backfill: adversarial cases + benchmarks — PLANNED
  Closes gaps: phase-4A, phase-4C, phase-4D, phase-4F, B-9. Authors all spec-required adversarial tests and Criterion benchmarks; no production code changes.
  Pre-reqs: C-11 (sandbox symbols must compile for adversarial tests).
  Cloud test: cloud /task triggers path-traversal attempt; SSB SANDBOX_VIOLATION row absent (blocked).

- [C-20](C-20_test_infra_backfill_metrics.md) — Test infra: metrics TOML backfill + measurement suite — PLANNED
  Closes gaps: phase-4G, phase-4F, phase-4B, phase-4M. Populates placeholder metrics TOMLs with real measurements; wires ELAI_DEBUG_BATCH_DELAY_SECS loop; adds `elai measure` subcommand with cloud Sonnet 4.6 live smoke run.
  Pre-reqs: C-9 (autonomous infra wired), C-13 (IPC dispatcher swap), C-17 (measurement harness).
  Cloud test: `elai measure` subcommand completes; phase_4G.toml 7 runtime categories non-zero.

- [C-21](C-21_self_mod_anti_stale_automation.md) — Self-modification + anti-staleness automation — PLANNED
  Closes gaps: B-16, B-23, phase-4H, guide/anti-staleness-enforcement. Wires check_anti_stale.sh into CI; B-16B Chunk C unwrap cleanup; git pre-commit LOCKED_KEYS hook; dep-audit → stale_api_catches; steering vector refresh trigger; closed ParameterKey enum; ConstraintsBlock from rules.db; SWE-bench fixtures; MistakeBook from BLOCK events.
  Pre-reqs: C-15 (entropy quarantine baseline).
  Cloud test: cloud /task promotes deprecated_patterns; compute_steering_vector.py timestamp changes; pre-commit hook blocks LOCKED_KEYS commit.

- [C-22](C-22_feasibility_probe_completeness.md) — FeasibilityProber completeness (phase 4F) — PLANNED
  Closes gaps: B-21, phase-4F. Renames SystemProbeReport→SystemProbeResult; adds GpuInfo.uuid, ResourceDelta; wires DefaultFeasibilityProber into production; authors A1-A8 adversarial tests; adds FeasibilityPanel.svelte; reconciles SSB action_type names; populates phase_4F.toml.
  Pre-reqs: C-19 (adversarial scaffolding; 4F tests gate on types added here), C-9 (autonomous gate for skip-path trigger).
  Cloud test: cloud /research on novel technology; SSB contains feasibility_probe_started + complete rows with non-null ResourceDelta.

### Wave 4 — Higher-order features (depend on Wave 3)

- [C-23](C-23_sessiondag_branch_ui.md) — SessionDag branch UI: merge/close CLI + BranchTree mount — PLANNED
  Closes gaps: B-9, B-3. Adds /session merge|close CLI; mounts BranchTree.svelte in SessionsDrawer; wires onfork/onmerge/onclose to real Tauri commands; fixes /continue → load_handoff; parent_chain ordering bug; 100-run determinism test.
  Pre-reqs: C-3, C-4 (Tauri commands path stable; AppState wiring proven).
  Cloud test: cloud /task >85% context → handoff snapshot; /continue in new session loads BranchTree branch.

- [C-24](C-24_symbol_impact_dashboard_surface.md) — Symbol impact dashboard surface — PLANNED
  Closes gaps: B-29, B-33. Adds ToolDefinition entries for symbol_impact/flow/context/list to build_tool_catalog; wires update_changed_files_symbol_graph; adds /graph mermaid CLI; cytoscape decomposition styles; blast-radius cross-link in ActivityWindow; creates graph_export_dispatch.rs contract test.
  Pre-reqs: C-17 (incremental update path stable; LockScope::Watch present).
  Cloud test: cloud /task modifying a widely-used symbol; agent self-selects symbol_impact tool; GraphWindow renders blast radius.

- [C-25](C-25_compiled_pipeline_template_crystallizer.md) — Compiled pipeline + template crystallizer — PLANNED
  Closes gaps: guide/compiled-pipeline-design, guide/post-phase4-implementation-order. Implements CompiledPipeline/TemplateCrystallizer/CompiledExecutor (Design 1); BackgroundTaskManager with soft-timeout auto-detach + 2 new SSB types (Design 2); CumulativeTokenTracker wired into cloud response parsing (Design 3); /pipeline list|reset CLI.
  Pre-reqs: C-9 (gate_decision SSB rows required for crystallization loop).
  Cloud test: same task repeated 3× cloud Sonnet; TemplateCrystallizer fires; 4th run uses CompiledExecutor (lower total tokens).

- [C-26](C-26_constrained_decoding_retry_strategy.md) — Constrained decoding + retry/futility primitives — PLANNED
  Closes gaps: guide/token-efficiency-design. Implements ConstrainedDecodingConfig/DecodingMode; RetryStrategy with temperature_schedule + futility_same_error_threshold; futility detection + persona switch in NavigatorTree; wires into SidecarClient and CloudGroundingClient; fixes benchmark fixture path bug; adds 3 EBNF grammar files.
  Pre-reqs: C-10 (GoalDriftCanary + ThinkingPersona ship first), C-25 (CumulativeTokenTracker for measurement).
  Cloud test: cloud /task hitting futility (same-error 3×) triggers persona switch; constrained decoding JSON output schema-valid.

- [C-27](C-27_testing_trust_t0_wiring_contract.md) — Testing-trust T(-1) determinism + T-0 wiring contract — PLANNED
  Closes gaps: guide/testing-trust-design (T-(-1) and T-0). Implements DeterminismConfig with hardware-class lock + SeededRng + blake3 prefix-cache + DETERMINISM_DRIFT SSB; WiringContract struct + StubBanAuditor + DkgReachabilityAuditor + ObservableEffectRunner + WIRING_EVIDENCE/WIRING_VIOLATION SSB types; `elai wiring audit` CLI; CI workflow.
  Pre-reqs: C-15 (ClockProvider/entropy stable; SeededClock/SeededRng required), C-17 (DKG resilience; DkgReachabilityAuditor walks symbol graph).
  Cloud test: `elai wiring audit` via cloud session; WiringEvidence SSB rows per entrypoint; StubBanAuditor catches ≥1 known stub.

### Wave 5 — Testing trust tiers (depend on Wave 4)

- [C-28](C-28_testing_trust_t1_mutation.md) — Testing-trust T-1 mutation testing — PLANNED
  Closes gaps: guide/testing-trust-design (T-1). Implements MutationRunner (cargo-mutants + mutmut), MutationResult, MUTATION_SURVIVED SSB type, EquivalentMutantCache, AUDITING→MEMORIZING gate, MonitorWindow mutation score gauge.
  Pre-reqs: C-27 (T-0 WiringContract wiring; mutation is the next tier up in the 5-check gate chain).
  Cloud test: cloud session runs mutation suite; MonitorWindow gauge shows mutation_score; AUDITING gate blocks on low score.

- [C-29](C-29_testing_trust_t2_t3_intent_metamorphic.md) — Testing-trust T-2 IntentPair + T-3 basic metamorphic relations — PLANNED
  Closes gaps: guide/testing-trust-design (T-2/T-3). Implements IntentPair formalization; MR_identity/MR_inverse/MR_permutation_invariance; INTENT_CONFIRMED + MR_VIOLATED SSB types (SCHEMA_VERSION 47); PLANNING→TESTING_FIRST gate requiring ≥3 IntentPairs; MetamorphicTestRunner; IntentPair editor in PlanWindow.svelte.
  Pre-reqs: C-27 (T-0 layer), C-28 (T-1 mutation tier wired).
  Cloud test: cloud /task with 3 IntentPairs defined; PLANNING→TESTING_FIRST gate passes; MR_identity relation verified.

### Wave 6 — Advanced pipeline and code generation (depend on Wave 4+)

- [C-30](C-30_surgical_oracle_mode.md) — Surgical mode + oracle mode + pipeline infrastructure — PLANNED
  Closes gaps: guide/pipeline-modes-design, guide/update-implementation-order, phase-4D. Implements SurgicalPlan/AffectedSet/ComplexityClass/RegressionGate/SurgicalDecomposer; Oracle answer-level cache (T1=30d/T2=7d/T3=1d TTL); DKG query tools in build_tool_catalog; project detection signal logic; blast-radius LOC gates; [research.vram_tiers] TOML section.
  Pre-reqs: C-17 (DKG resilience + IndexingService entry-point detection), C-24 (symbol_impact in tool catalog).
  Cloud test: /task --mode surgical via cloud Sonnet; SurgicalDecomposer plan with blast radius ≤3 files; AffectedSet from DKG.

- [C-31](C-31_code_completeness_file_plan.md) — Code completeness Layer 1: FilePlan + FileLineTracker — PLANNED
  Closes gaps: guide/code-completeness-enforcement-design, guide/update-implementation-order, phase-4E. Defines FilePlan/FileSpec/FileModSpec/FunctionSpec; adds FILE_PLAN as 9th BabyStepSpec block; FileLineTracker with TOCTOU prevention; MistakeBook recording from T1/T2 BLOCK events; per-baby-step function-count guard.
  Pre-reqs: C-13 (IPC dispatcher swap; mistake_book trait stable), C-9 (autonomous gate).
  Cloud test: cloud /task producing 3-file CRUD module asserts FILE_PLAN block in BabyStepSpec; FileLineTracker records cumulative lines; T2 BLOCKs produce MistakeBook entries.

- [C-32](C-32_code_completeness_skeleton_first.md) — Code completeness Layer 2: skeleton-first + MiniCritic + AcceptAndComplete — PLANNED
  Closes gaps: guide/code-completeness-enforcement-design, guide/token-efficiency-design. Implements SkeletonGenerator with BLANK_N placeholders; in-memory buffer single-write; MiniCritic between fills (tree-sitter + T1 regex subset); AcceptAndComplete recovery (≥0.5 threshold); structural retry sequence (DECOMPOSE→TEMPLATE→FEW_SHOT→BEST_OF_N); BridgeFileTemplate deterministic mod.rs aggregators; completeness_ratio gauge in MonitorWindow.
  Pre-reqs: C-31 (FilePlan + FileLineTracker stable), C-26 (constrained decoding for skeleton/fill differentiation).
  Cloud test: cloud /task producing >300 LOC target triggers SkeletonGenerator SSB row; ≥4 MiniCritic verdict rows; completeness_ratio ≥0.9; BridgeFileTemplate produces deterministic mod.rs (byte-identical with ELAI_SEED=42).

- [C-33](C-33_docs_phase_map_updates.md) — Documentation and phase map updates — PLANNED
  Closes gaps: guide/update-phase-map, B-0_INDEX, B-39, B-40, B-21. Creates steps/phase-3L.txt (Project Intelligence), phase-3J-BASELINE.txt, phase-4D-SURGICAL.txt, phase-4D-ORACLE.txt; updates B-0_INDEX.md (B-37/38/40/41/42 status blocks; SCHEMA_VERSION 46); reconciles B-39 REFINE_CAP (plan=2, code=3) and B-21 SSB action type names; adds LAZY_CODE/EMPTY_BODY/FILE_SIZE_EXCEEDED to FailureCategory; per-command KEEP/STRIP for 28+ slash commands in migration_manifest.toml.
  Pre-reqs: C-30 (Surgical/Oracle shipped so phase-4D-SURGICAL/ORACLE spec writes match code), C-22 (B-21 SSB names reconciled after FeasibilityProber lands).
  Cloud test: spec_link_check.py passes for all updated step files; wiring_contract_check.py shows no WIRING_DIVERGENCE for reconciled B-21/B-39 contracts; /research --mode cloud "what does phase 3L do?" returns coherent answer.

### Wave 7 — Paper-grounded improvements, batch A (HIGH-LEVERAGE TRIVIAL/MODERATE)

Sourced from arXiv 2026-03-17..2026-05-17 + Discord forum-dump 2026-05-17. All
papers verified via direct abs-page fetch. Citations carry quoted measurements.

- [C-35](C-35_tool_attention_dynamic_gating.md) — Tool Attention dynamic gating + lazy MCP schema loading — PLANNED
  Cites: arXiv:2604.21816 (95% measured MCP-tax reduction; downstream quality figures are PROJECTIONS not measured on live agents).
  Pre-reqs: Phase 4K MCP bridge SHIPPED. Parallel with C-36..C-40 except C-42 (same surface).
  Cloud test: 30+ tool catalog; prompt_tokens_saved per feature in 5-10K range; SWE-bench delta ≤2pp vs baseline.

- [C-36](C-36_mucoco_mutation_operator_catalog.md) — MUCOCO mutation-operator catalog — PLANNED
  Cites: arXiv:2604.19086 ("15% inputs exposed inconsistencies"). Behind feature flag; extends C-29 MetamorphicRelation enum.
  Pre-reqs: C-29 SHIPPED. Parallel with C-35, C-37..C-53.
  Cloud test: pilot detection-rate matches paper's 15% on ≥1 operator.

- [C-37](C-37_per_role_decoding_defaults_reeval.md) — PER_ROLE_DECODING_DEFAULTS re-evaluation — PLANNED
  Cites: arXiv:2604.06066 ("structure snowballing" counter-evidence for JsonSchema in reflection). Pilot before C-54 publication.
  Pre-reqs: C-26 SHIPPED. RunPod for pilot run.
  Cloud test: pilot.md auto-generated; anti-rubber-stamp FINDING test passes under any mode.

- [C-38](C-38_pagerank_production_contract_aider_spec_fix.md) — PageRank production contract + Aider spec accuracy — PLANNED
  Source: prior-oracle 2026-05-17 finding (DETAILED_SUMMARY §10 mis-describes Aider repomap as file-level; verified symbol-level).
  Pre-reqs: B-1 confirmed SHIPPED. Parallel with C-35..C-37, C-39, C-40.
  Cloud test: contract test pass on production path; DETAILED_SUMMARY §10 cites diff artifact + fresh-fetch date.

- [C-39](C-39_mcp_bridge_cli_surface.md) — MCP bridge CLI surface extension — PLANNED
  Source: OpenHands parity (Apache 2.0, ~38K stars). CI parity job required.
  Pre-reqs: C-35 SHIPPED (consumes gated catalog). Parallel with C-36..C-38, C-40.
  Cloud test: --via-mcp parity green; auto-launch co-process works.

- [C-40](C-40_github_pr_adapter.md) — GitHub PR adapter (Devin parity) — PLANNED
  Source: Devin 2.0 published-PR loop. Opt-in per project, default OFF for review-comment auto-response.
  Pre-reqs: C-13 + C-23 SHIPPED. Parallel with C-35..C-39.
  Cloud test: end-to-end draft PR open + body composed with honesty section.

- [C-41](C-41_skills_driven_workflow_formalization.md) — Skills-driven workflow layer formalization — PLANNED
  Cites: arXiv:2604.21910 (44%→83% measured intent accuracy with explicit skill scaffolds).
  Pre-reqs: Phase 4A + prompt-engineer SHIPPED. Parallel with all C-35..C-40, C-42..C-53.
  Cloud test: SkillRegistry synthetic SciQuest accuracy ≥73% (conservative).

- [C-42](C-42_semantic_gateway_zero_trust_fuzzing.md) — Semantic Gateway zero-trust MCP fuzzing — PLANNED
  Cites: arXiv:2604.25555 (100% unauthorized-state discovery via deterministic fuzzer). Sequence AFTER C-35; BEFORE C-40, C-44.
  Pre-reqs: C-35 + C-39 SHIPPED. Sequenced after C-35 (overlapping surface).
  Cloud test: fuzzer CI job green with zero CRITICAL on ≥200 tuples.

- [C-43](C-43_phase_4h_scope_freeze_ratification.md) — Phase 4H 3-layer scope-freeze ratification (AEL anti-evidence) — PLANNED
  Cites: arXiv:2604.21725 — adding mechanisms beyond memory+reflection consistently DEGRADES performance. Policy + guardrail; no behavioral change.
  Pre-reqs: Phase 4H SHIPPED at current scope. Parallel with all other C-N.
  Cloud test: ablation harness surfaces negative delta on synthetic DGM-like candidate (4H.X.6).

- [C-44](C-44_composer2_runtime_backend_tier.md) — Composer 2 runtime backend tier — PLANNED
  Cites: arXiv:2603.24477 (SWE-bench Multilingual 73.7; $0.50/M input). Gated on API access verification.
  Pre-reqs: B-37 SHIPPED; C-39 + C-42 SHIPPED. RunPod NOT required (cloud API).
  Cloud test: third-tier backend invokes; cost_accrued SSB rows on every call.

### Wave 8 — Paper-grounded improvements, batch B (MODERATE/RESEARCH-GRADE)

- [C-45](C-45_cantante_contrastive_credit_attribution.md) — CANTANTE contrastive credit attribution — PLANNED
  Cites: arXiv:2605.13295. Per-role contribution measurement; feeds C-41 routing + future role consolidation decisions.
  Pre-reqs: 8 AgentRoles SHIPPED. Pilot DEFERRED to RunPod window.
  Cloud test: pilot.md per-role credit table with confidence interval.

- [C-46](C-46_uno_orchestra_parsimonious_routing.md) — Uno-Orchestra parsimonious routing — PLANNED
  Cites: arXiv:2605.05007. Difficulty-tier dispatch saves 40-60% on easy tasks.
  Pre-reqs: 8 AgentRoles SHIPPED; C-45 recommended.
  Cloud test: pilot ±2pp solve-rate; ≥30% cost reduction.

- [C-47](C-47_case_based_calibration_tool_use.md) — Case-Based Calibration tool-use telemetry — PLANNED
  Cites: arXiv:2605.15041. Per-project sled-backed case base; hint injection on near-match.
  Pre-reqs: C-35 + C-42 SHIPPED. Parallel with C-46, C-48..C-53.
  Cloud test: 50-session replay shows ≥10% reduction in repeat-tool-errors.

- [C-48](C-48_safeharbor_memory_guardrail.md) — SafeHarbor hierarchical memory guardrail — PLANNED
  Cites: arXiv:2605.05704. 4-tier trust (T0 user-direct ... T3 model-emitted); safety-critical queries filter to T0+T1 only.
  Pre-reqs: memory-writer + C-42 SHIPPED.
  Cloud test: synthetic prompt-injection fixture (T2 row) does NOT influence safety-critical decision.

- [C-49](C-49_iterative_distillation_prompt_policy.md) — Iterative Distillation for prompt-engineer — PLANNED
  Cites: arXiv:2605.14443 (most recent paper in batch). Per-project experience corpus accumulates from successful traces.
  Pre-reqs: prompt-engineer + C-41 (recommended) SHIPPED.
  Cloud test: 10-session smoke shows non-empty distilled fragments section.

- [C-50](C-50_contract_coding_test_scaffolding.md) — Contract-Coding test scaffolding — PLANNED
  Cites: arXiv:2604.13100 (47% functional success, near-perfect structural integrity on Greenfield-5).
  Pre-reqs: Phase 4F SHIPPED.
  Cloud test: Greenfield-5 fixture reproduces ≥40% functional + ≥90% structural (conservative vs paper's 47%/near-perfect).

- [C-51](C-51_aider_polyglot_harness_adoption.md) — Aider Polyglot harness adoption (test infra) — PLANNED
  Source: Aider MIT harness vendored at pinned SHA; 225 Exercism problems. Infra-only — actual run is C-55.
  Pre-reqs: Phase 4F SHIPPED.
  Cloud test: dry-run 5 sample tasks against mock backend; full sweep blocked without GPU.

- [C-52](C-52_dafny_formal_verification_spike.md) — Dafny formal verification spike (RESEARCH-GRADE) — PLANNED
  Cites: arXiv:2604.22601 (Gemma 4-31B at 90.91% on NL2VC-60). 3 narrow modules: FSM, MemoryView, mutation aggregator.
  Pre-reqs: C-48 (MemoryView) recommended. Closure includes "infeasibility note" as a valid outcome.
  Cloud test: 3 spike modules verify OR infeasibility note with ≥1 concrete blocker.

- [C-53](C-53_nf4_kernel_watch_in_context_counter_test.md) — NF4 kernel watch + In-Context counter-test — PLANNED
  Consolidates 2 low-priority items: arXiv:2604.02556 (NF4 kernel upstream watch, passive); arXiv:2604.27891 (counter-evidence: in-context obsoletes orchestration for procedural tasks; needs ablation).
  Pre-reqs: none for watch; RunPod for counter-test pilot.
  Cloud test: upstream-watch.md tracks bitsandbytes; counter-test pilot artifact authored.

### Wave 9 — Public benchmark publication (RUNPOD-DEPENDENT, CAPSTONE)

These patches each require a live GPU (RunPod 4090 / A100 / H100 or equivalent
local) to fire. Sequence within Wave 9 is INDEPENDENT — any can run when
hardware is online. C-59 is the aggregator and runs LAST.

- [C-54](C-54_swe_bench_verified_run_three_configs.md) — SWE-bench Verified run, 3 configs (FLAGSHIP) — PLANNED
  Three arms: local-7B-only / dual-local-7B-7B / hybrid Sonnet+Composer2+local.
  Pre-reqs: C-21 corpus expansion + C-35 + C-39 + C-42 + C-44 (if hybrid arm) SHIPPED.
  RunPod required.
  Cloud test: 500-task scorecard.json per config; methodology.md auto-generated; results.md includes honesty statement.

- [C-55](C-55_aider_polyglot_leaderboard_submission.md) — Aider Polyglot leaderboard submission — PLANNED
  Two configs (local 32B, hybrid); public draft-PR via C-40 GitHub adapter.
  Pre-reqs: C-51 SHIPPED. RunPod required.
  Cloud test: 225-problem scorecards per config; public-PR opened against Aider-AI/aider.

- [C-56](C-56_codebase_memory_dkg_31_repo_eval.md) — Codebase-Memory DKG 31-repo head-to-head — PLANNED
  Cites: arXiv:2603.27277 (83% answer quality / 10× tokens / 2.1× tool-calls vs file-explorer).
  Pre-reqs: B-29 + B-30 + C-38 SHIPPED. RunPod required.
  Cloud test: per-repo + aggregate metrics; results.md includes paper-baseline cross-tabulation.

- [C-57](C-57_openhands_index_swe_cycle_swe_atlas.md) — OpenHands Index + SWE-Cycle + SWE Atlas combo — PLANNED
  Cost-amortized 3-benchmark run on one RunPod session. Cites: openhands.dev/blog/openhands-index, arXiv:2605.13139, arXiv:2605.08366.
  Pre-reqs: C-54 SHIPPED (primary number first). RunPod required.
  Cloud test: per-axis scorecards; SWE-Cycle solve-rate curve documented.

- [C-58](C-58_agentforge_rrk_reranker_head_to_head.md) — AgentForge 5-role + RRK reranker eval — PLANNED
  Cites: arXiv:2604.13120 (AgentForge 40.0% SWE-bench Lite); arXiv:2604.26483 (RRK 8B 3-18× faster than smaller rerankers).
  Pre-reqs: C-45 recommended. RunPod required.
  Cloud test: head-to-head scorecard; RRK latency p99 + quality delta; config-gated adoption decision.

- [C-59](C-59_cross_config_benchmark_master_report.md) — Cross-config benchmark master report (CAPSTONE) — PLANNED
  Aggregator: pulls all scorecards from C-54..C-58 + external_baselines.toml; emits machine-regenerated master report. Optional TTS pilot (arXiv:2605.09329).
  Pre-reqs: C-54 + C-55 + C-56 + C-57 + C-58 SHIPPED.
  Cloud test: master report compiles deterministically; honesty + improvement-target sections present.

### Deferred

- [C-34](C-34_empire_integration_deferred.md) — Phase 4J Empire Integration (DEFERRED — external dependency) — DEFERRED
  Closes gaps: B-25, phase-4J. Implements FactoryLineTransport, ElaiNeuron MCP server, BrainManifest, EmpireEventBridge, EmpireIssue repair pipeline (CircularDep/HardImport/StdlibShadow); feature-gated crates/empire/ workspace member.
  Pre-reqs: EXTERNAL — Empire codebase access must be granted by upstream maintainer; elai_cc.toml [empire.codebase_path] confirmed before execution. Parallel-safe with ALL other C-N.
  Cloud test: ElaiNeuron MCP server connected; cloud /task triggers Empire capability; SSB contains empire_connected + empire_capability_invoked rows.

---

## Summary statistics

- Total session slots: 59
- In-scope (executable now, CPU/cloud): 52 (C-1 through C-33 + C-35 through C-53)
- In-scope (RUNPOD-DEPENDENT, capstone): 6 (C-54 through C-59)
- Deferred (external block): 1 (C-34)
- Files authored: 59 (all C-N files written; C-35..C-59 authored 2026-05-17)
- Estimated effort per implementation session: ~6-8 h (C-1..C-53)
- Estimated effort per benchmark session: ~8-24 h wall-clock incl. RunPod (C-54..C-59)
- Total estimated effort (52 CPU/cloud sessions): ~310-415 h
- Total estimated effort (6 benchmark sessions): ~50-150 h + RunPod cost

## Wave 7-9 source provenance (C-35..C-59)

- Prior oracle research (2026-05-17): apps_workflow oracle pass produced
  workflow/sessions/oracle/oracle/2026-05-17-elai-competitive-papers.md with
  11 papers + 22 competitor capabilities cross-checked against the B-plan
  and C-plan punch lists.
- Discord forum-dump (2026-05-17): F:\.ELAI_workflow\.extras\discord-catchup-bot\forum-dump.md
  with 15 additional arXiv papers from 2026-04-25..2026-05-17, all verified
  via abs-page fetch by oracle researcher 2026-05-17.
- Total verified arXiv citations: 19 papers across Waves 7-9 + 7 competitor
  comparison anchors (Devin, Aider, Cursor, Claude Code, OpenHands,
  mini-SWE-agent, GPT-5.4 / Composer 2).

## Verification source

- Master punch list: [verification/INDEX.md](../verification/INDEX.md) — 103 reports (45 B-plan + 18 guide + 40 steps)
- Aggregated themes: [verification/_themes.md](../verification/_themes.md) — 25 themes
- Report status breakdown: DONE 5, MOSTLY_DONE 77, PARTIAL 20, NOT_DONE 3, UNVERIFIABLE 0

## Per-area coverage (B-N → C-N mapping)

- B-1: closed by C-18 (contract test gap), C-8 (pagerank/AragOrchestrator wiring)
- B-2: closed by C-18 (fsm_phase_transition_contract.rs)
- B-3: closed by C-3 (pagerank_top/freshness_state viewers), C-8 (stub eradication), C-23 (/continue load_handoff)
- B-4: closed by C-1 (SSB variants + emit sites), C-3 (Tauri commands), C-8 (HyDE/SsbNotifier/ONNX wiring)
- B-5: closed by C-1 (SSB variants), C-3 (get_csq_log/get_agent_grc_config), C-12 (per-agent budget)
- B-6: closed by C-2 (AgentLifecycle emit), C-4 (get_agents live, CliInteractiveGate)
- B-7: closed by C-2 (phase/step/baby_step SSB types), C-4 (get_phase_detail, get_executable_pipeline), C-8 (research_node_stage emits)
- B-8: closed by C-1 (csq_evaluated SSB row), C-3 (ReplanRequest::Major, verify_claim)
- B-9: closed by C-23 (merge/close CLI, BranchTree mount, /continue fix)
- B-10: closed by C-4 (VramTier detection, agent_files overlap)
- B-11: closed by C-6 (design tokens), C-7 (font/size sweep)
- B-12: closed by C-6 (CSS token alignment), C-7 (UX completeness)
- B-13: closed by C-2 (bubble_up SSB types), C-7 (BubbleUpStrip live events)
- B-14: closed by C-7 (Stop button, Ctrl+Enter, streaming improvements)
- B-15: closed by C-14 (dead code strip), C-15 (HashMap ban, clock migration, entropy CI)
- B-16: closed by C-21 (check_anti_stale.sh CI, Chunk C unwrap, pre-commit hook)
- B-17: closed by C-2 (AmbiguityScorer SSB types), C-8 (AmbiguityScorer wiring)
- B-18: closed by C-11 (setrlimit, hidepid, capability drop, IpcHookDispatcher)
- B-19: closed by C-13 (IpcToolRegistry production swap)
- B-20: closed by C-13 (IpcHookDispatcher production, AdaptiveTool DAIRA markers)
- B-21: closed by C-22 (FeasibilityProber completeness, FeasibilityPanel.svelte)
- B-22: closed by C-9 (AutoAdvanceEvaluator, EscalationPolicy), C-10 (StagnationDetection, GoalDriftCanary)
- B-23: closed by C-21 (dep-audit wiring, steering vector refresh, nightly trigger)
- B-24: closed by C-7 (MediaValidationPanel UX), C-8 (media SSB event subscription)
- B-25: closed by C-34 (DEFERRED — Empire access required)
- B-26: closed by C-4 (MonitorWindow live polling Tauri commands), C-7 (NotificationManager UX)
- B-27: closed by C-5 (cloud mode hardening: spend cap, CLI --mode, HYBRID escalation, CloudPanel.svelte)
- B-28: closed by C-4 (get_recent_actions, get_file_changes, create_project)
- B-29: closed by C-24 (symbol_impact tool catalog exposure, incremental graph update)
- B-30: closed by C-17 (Kotlin grammar, svelte extraction test, scale benchmarks)
- B-31: closed by C-17 (auto_resume, SIGINT/SIGTERM, LockScope::Watch, indexing resilience contract)
- B-34: closed by C-17 (ONNX query embeddings, Oracle pre-seeding, p50/p99 timing)
- B-35: closed by C-8 (ArtifactsIndexed/ArtifactSearched SSB), C-18 (grounding_artifact_check, designer_artifact_briefing contract tests)
- B-36: closed by C-4 (remaining D-commands), C-18 (dashboard_v2 contract tests)
- B-37: closed by C-4 (list_runtimes/test_runtime/set_role_runtime Tauri commands)
- B-38: closed by C-2 (ambiguity_scored SSB types), C-8 (AmbiguityScorer wiring into task_runner)
- B-39: closed by C-33 (REFINE_CAP reconciliation)
- B-40: closed by C-12 (FramedIdea forwarded to Concepts auditor), C-33 (B-0_INDEX update)
- B-41: closed by C-12 (navigator wiring at 5 tier-completion points, AuditTimelineWindow live data)
- B-42: closed by C-2 (cosmos SSB types already defined), C-16 (CosmosStage pipeline wiring, tick scheduler, 5 test files)
- B-0_INDEX: closed by C-33 (SCHEMA_VERSION update, B-37..42 status blocks, reconciliations)
- guide/compiled-pipeline-design: closed by C-25 (CompiledPipeline/TemplateCrystallizer/CompiledExecutor)
- guide/token-efficiency-design: closed by C-26 (ConstrainedDecodingConfig, RetryStrategy, futility)
- guide/pipeline-modes-design: closed by C-30 (SurgicalDecomposer, Oracle cache, DKG tool exposure)
- guide/code-completeness-enforcement-design: closed by C-31 (FilePlan + FileLineTracker), C-32 (skeleton-first + MiniCritic + BridgeFileTemplate)
- guide/testing-trust-design: closed by C-27 (T-(-1)/T-0), C-28 (T-1), C-29 (T-2/T-3)
- guide/anti-staleness-enforcement: closed by C-21 (CI wiring, ParameterKey enum, ConstraintsBlock)
- guide/dashboard-design: closed by C-6 (token alignment), C-7 (UX completeness)
- guide/update-phase-map: closed by C-33 (new step files, B-0_INDEX update)
- phase-4G: closed by C-9 (AutoAdvanceEvaluator), C-10 (4G.9 anti-spiral primitives), C-20 (metrics TOML)
- phase-4A: closed by C-11 (sandbox gaps), C-19 (adversarial tests)
- phase-4F: closed by C-22 (FeasibilityProber), C-19 (A1-A8 adversarial tests), C-20 (metrics)
- phase-4H: closed by C-21 (automation wiring)
- phase-4J: closed by C-34 (DEFERRED)
- phase-4K: closed by C-4 (get_phase_progress, get_action_detail, list_projects, create_project)
