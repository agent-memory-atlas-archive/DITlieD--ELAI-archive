# ELAI C-plan exhaustive feature discovery

## Discovery contract and coverage

- Discovery source read in full: C-SUMMARY_INDEX.md, all 814 lines / 8,080 words.
- Orientation source read in full: ELAI-SPEC-OVERVIEW.md, all 592 lines / 6,873 words. It is used only to understand C as the verification-gap, testing-trust, and code-completeness layer; it is not treated as a substitute for the C summary.
- Feature-bearing plan range covered here: C-1 through C-59. C-0_INDEX is an index/source manifest rather than an implementation session.
- Trace notation:
  - [E] means explicitly stated by C-SUMMARY_INDEX.md.
  - [I] means a conservative interaction inferred by combining explicit entries. No inferred item is presented as a promised implementation detail.
  - [A] means an ambiguity or conflict that must be resolved from an original plan or another source.
- Source anchors use C-SUMMARY_INDEX.md line numbers from the file read on 2026-08-11.
- This is a design/plan inventory. “Adds”, “wires”, “creates”, and “ships” below preserve the summary’s own language and do not independently assert repository implementation state.

### Coverage facts and scope ambiguities

- [E] The summary header calls itself an index of 35 sessions, generated from C-0_INDEX plus C-1 through C-34, and gives an original source volume of about 10,200 lines. [L1-L4]
- [E] The original status legend says C-1 through C-33 are PLANNED and only C-34 is DEFERRED. [L6]
- [E] The later coverage section supersedes the old count with 59 session slots: 52 CPU/cloud sessions (C-1 through C-33 and C-35 through C-53), six RunPod-dependent sessions (C-54 through C-59), and one externally blocked session (C-34). [L578-L587]
- [E] The same later section says 60 files were authored, including C-0_INDEX, this summary, and C-35 through C-59. [L584]
- [A] The title, generated-from statement, and status legend were not updated after C-35 through C-59 were appended. Original plan status must therefore be checked rather than extrapolated from the stale C-1 through C-34 legend.
- [E] The premise says cloud mode has already been proven end to end through OAuth and Sonnet 4.6 at commit db594a6. [L8-L11]
- [E] Each session is intended to deliver one cohesive area, wire it to the dashboard, and prove it with a real cloud-Sonnet acceptance test. [L10-L12]
- [E] The original C-1 through C-34 effort estimate is about 6–8 hours per session and about 200–264 hours total. [L12-L14]
- [E] The expanded estimate is about 370–500 hours, excluding RunPod compute cost; implementation sessions C-1 through C-53 are estimated at 6–8 hours each, while benchmark sessions C-54 through C-59 are estimated at 8–24 wall-clock hours each plus RunPod cost. [L580-L587]
- [E] C-1 through C-33 were derived from 103 reports across 25 themes: 45 B-plan reports, 18 guide reports, and 40 step reports. [L588]
- [E] C-35 through C-59 cite a later oracle pass, a Discord forum dump, 19 claimed verified arXiv papers, and seven competitor anchors. [L589, L592-L595]
- [A] The citation table actually contains 22 arXiv IDs, while the narrative says 19 papers. This count requires original-source reconciliation. [L589, L787-L814]

## Dependency and execution waves

- [E] Wave 1 has no prerequisites and is parallel-safe: C-1, C-6, C-14. [L16-L18]
- [E] Wave 2 is the data-plane and infrastructure wave, depends on Wave 1, and contains C-2, C-3, C-4, C-7, C-8, C-9, C-11, C-12, and C-15. [L18-L20]
- [E] Wave 3 is integration and test infrastructure, depends on Wave 2, and contains C-10, C-13, C-16, C-17, C-18, C-19, C-20, C-21, and C-22. [L20]
- [E] Wave 4 contains higher-order features, depends on Wave 3, and contains C-23 through C-27. [L21]
- [E] Wave 5 contains testing-trust tiers, depends on Wave 4, and contains C-28 and C-29. [L22]
- [E] Wave 6 contains advanced pipeline and code-generation work, depends on Wave 4 or later prerequisites, and contains C-30 through C-33. [L23]
- [E] C-34 is outside these waves and deferred. [L24]
- [E] Wave 7, C-35 through C-44, adds paper-grounded improvements batch A. [L592-L668]
- [E] Wave 8, C-45 through C-53, adds paper-grounded improvements batch B. [L668-L732]
- [E] Wave 9, C-54 through C-59, is public-benchmark publication and is RunPod-dependent. [L733-L775]

## Exhaustive atomic feature inventory

### C-1 — SSB Emission: Knowledge + Reasoning Producers

Status and dependency:

- [E] Closes B-4, B-5, and B-8; has no prerequisites because those B plans are described as already shipped; is parallel-safe with C-2, C-3, and C-4. [L28-L31]

Data, events, wiring, and UI:

- [E] Adds seven SsbActionType variants and payloads: GrcStepEmitted, GroundingSweepResult, CsqEvaluated, AuditorRubberStampRejected, HydeQuery, VectorCacheHitRate, and QueryTrace. [L32]
- [E] Changes emit_reasoning to accept PipelineGrc. [L33]
- [E] Emits CsqEvaluated from task_runner.rs at the stated source location 5119. [L33]
- [E] Emits GroundingSweepResult after ground_plan. [L33]
- [E] Emits AuditorRubberStampRejected from reasoning/engine.rs at the stated source location 340. [L33]
- [E] Threads a real shared SSB handle into HydeStage, VectorCache, QueryPipeline, and CrossEncoderReranker, replacing NoopSsbNotifier. [L34]
- [E] Wires get_grounding_sweep, get_query_trace_recent, and get_csq_pending Tauri commands to live data. [L35]
- [E] Clears three unwired flags corresponding to those surfaces. [L35]
- [E] Increments SCHEMA_VERSION by exactly one for this event-family addition. [L36]
- [E] Named touch points are ssb.rs, task_runner.rs, reasoning/engine.rs, hyde.rs, vector/cache.rs, pipeline.rs, reranker.rs, commands.rs, and drawers.svelte.ts. [L37]
- [E] Requires five contract tests. [L37]
- [E] Cloud smoke task: create a pure-Rust add function with documentation and a unit test; acceptance requires at least five of the seven new SSB types and non-null payloads. [L38]

### C-2 — SSB Emission: Orchestration + Tree + Cosmos + Ambiguity + Anti-Spiral

Status and dependency:

- [E] Closes B-6, B-7, B-13, B-38, B-42, and phase-4G; depends on C-1; is parallel-safe with C-3, C-4, and C-9. [L40-L43]

Data, events, wiring, and UI:

- [E] Declares 12 new SsbActionType variants while naming PhaseNodeReady, StepNodeReady, BabyStepReady, DagRepairEscalated, AgentStatusChanged, StagnationSpinningDetected, StagnationOscillationDetected, LateralThinkingActivated, GoalDriftWarning, and WatchdogTimeout. [L44]
- [E] Notes that Cosmos event types were already defined. [L44]
- [A] Only ten new variants are named despite the stated count of 12; the original plan must identify the other two or explain whether existing Cosmos variants were included in the count.
- [E] Replaces CosmosWorker tracing-only behavior with persisted Cosmos SSB rows. [L45]
- [E] Inserts AmbiguityScorer into task_runner as a pre-gate. [L45]
- [E] Emits tree-node readiness and dag_repair_escalated events. [L45]
- [E] Adds ssb_viewer.recent_agent_status_changes. [L46]
- [E] Replaces the hardcoded-IDLE get_agents result with live agent data. [L46]
- [E] Replaces the mock get_ambiguity_state result with live ambiguity data. [L46]
- [E] Adds an AmbiguitySection to OrchestrationDrawer. [L47]
- [E] Requires three contract-test files. [L47]
- [E] Cloud smoke task: build a three-function Rust add/subtract/multiply CLI; acceptance requires at least five distinct new action types with non-null values. [L48]

### C-3 — Tauri Commands: Knowledge / Reasoning Dashboard Surface

Status and dependency:

- [E] Closes B-3, B-4, and B-5; depends on C-1; is parallel-safe with C-4. [L50-L53]

Dashboard query surface:

- [E] Replaces seven viewer stubs in ssb_viewer.rs. The summary names pagerank_top, query_tier_stats, freshness_state, research_budget_snapshot, source_tier_distribution, and recent_query_traces. [L54]
- [A] The text says seven stubs but visibly names six; the seventh must be recovered from C-3.
- [E] Adds eight Tauri commands: get_pagerank_top, get_freshness_state, get_canary_drift, get_csq_log, get_feasibility_log, get_agent_grc_config, get_research_budget, and get_source_tier_distribution. [L55]
- [E] Wires get_query_trace_recent to real SSB rows. [L56]
- [E] Clears unwired.recentTraces. [L56]
- [E] Cloud smoke task: search for foo and explain it; acceptance requires at least one recent query trace, a positive PageRank score, and at least one CSQ-log entry. [L57]

### C-4 — Tauri Commands: Orchestration + Cloud Spend Surface

Status and dependency:

- [E] Closes B-6, B-7, B-26, B-27, B-36, phase-4K, and phase-4M; depends on C-1 and C-2; is parallel-safe with C-3. [L59-L62]

Dashboard, project, runtime, and spend surface:

- [E] Adds recent_agent_lifecycle_rows and cloud_spend_summary to SsbViewer. [L63]
- [E] Replaces three stubs: get_agents no longer returns hardcoded IDLE, get_cloud_spend no longer returns zeros, and get_executable_pipeline no longer returns an empty array. [L64]
- [E] Adds ten Tauri commands: get_phase_detail, get_baby_step_detail, get_executable_pipeline, get_research_node_history, get_phase_progress, get_recent_actions, get_file_changes, get_action_detail, list_projects, and create_project. [L65]
- [E] Replaces RuntimePanel stub calls with real invocations for list_runtimes, test_runtime, and set_role_runtime. [L66]
- [E] Cloud smoke task: implement a four-function todo CRUD module; acceptance requires MonitorWindow to show at least one non-IDLE agent and a non-zero cloud-spend value. [L67]

### C-5 — Cloud Mode Hardening

Status and dependency:

- [E] Closes B-27; depends on C-1 and C-4; is parallel-safe with C-6 and C-7, while CloudPanel adopts C-6 design tokens when available. [L69-L72]

Routing, cost, configuration, failure, and UI:

- [E] Enforces hard daily and monthly spend ceilings. [L73]
- [E] Returns typed DailySpendCapExceeded and MonthlySpendCapExceeded errors when those ceilings are crossed. [L73]
- [E] Adds warn_at_percent with an 80% default. [L73]
- [E] Adds CLI mode selection through --mode cloud, --mode hybrid, and --mode local. [L73]
- [E] Adds one-shot HYBRID escalation retry with a cap of one escalation. [L73]
- [E] Adds LOC sizing constants 25, 40, 300, and 500. Their exact semantic labels are not present in the summary. [L73]
- [E] Adds estimate_task_cost_usd. [L73]
- [E] Supports bring-your-own-endpoint through custom_endpoint. [L73]
- [E] Adds SsbActionType RouteBlocked and its payload. [L74]
- [E] Creates CloudPanel.svelte with a spend gauge, mode selector, and redaction-event feed. [L75]
- [E] Mounts CloudPanel in SettingsWindow. [L75]
- [E] Requires cloud-ceiling, hybrid-escalation, and cloud-LOC-constants contract-test files. [L76]
- [E] Smoke 1 uses a 0.05 cap and 80% warning threshold and expects WARN_LIMIT_REACHED plus RouteBlocked. [L77]
- [E] Smoke 2 expects EscalateToCloud and verifies the escalation cap is one. [L77]
- [E] Smoke 3 verifies CloudPanel renders. [L77]

### C-6 — Dashboard Design Token Alignment Pass

Status and dependency:

- [E] Closes B-11, B-12, B-14, and guide/dashboard-design; has no prerequisites because it is dashboard-only; is parallel-safe with C-5 and C-7. [L79-L82]

Visual system and controls:

- [E] Sets background tokens: --bg-void to #0a0e12, --bg-base to #0f1318, --bg-surface to #151a20, and --bg-hover to #1a2028. [L83]
- [E] Aligns the teal palette and the --text-primary, --text-secondary, and --text-ghost tokens. [L83]
- [E] Sets --font-sans to JetBrains Mono. [L83]
- [E] Changes 11px text to 10px across ArtifactsPanel, IndexingProgressPanel, MediaValidationPanel, AntiStalenessDrawer, CosmosDrawer, KnowledgeDrawer, and MonitorWindow. [L84]
- [E] Shrinks the sidebar from 64px to 48px. [L85]
- [E] Documents the sidebar icon-count delta: five in the specification versus seven currently. [L85]
- [E] Shows a Stop button in CommandWindow while streaming and connects it to the existing stop handler. [L86]
- [E] Adds Ctrl+Enter in CommandWindow and connects it to the existing submit handler. [L86]
- [E] Corrects B-11 DETERMINISTIC_CHECK paths to point at dashboard-v2. [L87]
- [E] Manual smoke verifies computed CSS tokens and tests Stop and Ctrl+Enter during an active stream. [L88]

### C-7 — Dashboard UX Completeness

Status and dependency:

- [E] Closes B-11, B-12, B-13, B-14, B-26, dashboard-design, and phase-4K; depends on C-6 and C-2; is parallel-safe with C-5 and C-1 through C-4. [L90-L93]

Windows, recovery, review, responsiveness, and testing:

- [E] Adds the explore member to WindowType. [L94]
- [E] Creates ExploreWindow.svelte. [L94]
- [E] Assigns Ctrl+8 to open the Explore window. [L94]
- [E] Implements a closedTabs stack. [L95]
- [E] Creates RecoveryModal.svelte. [L95]
- [E] Fixes Ctrl+Shift+H so it opens recovery instead of remaining a no-op stub. [L95]
- [E] Adds Ctrl+Shift+A for review accept-all. [L96]
- [E] Adds Ctrl+Shift+R for review reject-all. [L96]
- [E] Adds per-hunk Accept, Decline, and Retry controls to ReviewWindow. [L97]
- [E] Removes the UnwiredNote banner after wiring ReviewWindow to live get_file_changes data. [L97]
- [E] Adds a responsive hamburger layout below 1000px. [L98]
- [E] Adds a single-panel layout below 800px. [L98]
- [E] Adds five Playwright snapshots in visual.spec.ts. [L99]
- [E] Cloud smoke modifies hello.rs to add goodbye; acceptance requires visible hunk controls and working Ctrl+Shift+A accept-all. [L100]

### C-8 — Viewer Stub Eradication: Remaining Producers

Status and dependency:

- [E] Closes B-3, B-4, B-5, B-7, B-38, B-42, and phase-4K; depends on C-1 and C-2; is parallel-safe with C-3 and C-4. [L102-L105]

Production activation and retrieval correctness:

- [E] Wires AmbiguityScorer into task_runner only at the PLANNING to TESTING_FIRST gate. [L106]
- [E] Drives CosmosWorker::tick hourly through a Tokio interval in task_runner. [L107]
- [E] Calls index_all_artifacts from IndexingService::build_index_resumable. [L108]
- [E] Replaces the production TOOL_INDEX_ARTIFACTS error stub with a real implementation in production_agent_rpc.rs. [L109]
- [E] Replaces the CrossEncoderReranker NoopSsbNotifier. [L110]
- [E] Stores DegradationCascade in AppState.search_mode. [L110]
- [E] Replaces AragOrchestrator semantic-search’s embedding-norm proxy with real ONNX dot-product scoring. [L111]
- [E] Preserves real latency by replacing at least 17 latency_ms=0 sites in pipeline.rs with Instant::elapsed measurements. [L112]
- [E] Inserts CosmosStage into QueryPipeline only for STANDARD and DEEP tiers; FAST omits it. [L113]
- [E] Research smoke compares two codebase concepts and expects positive total query latency, non-mock ambiguity scoring, and cosmos_refresh_started. [L114]

### C-9 — Autonomous Mode Core Infrastructure

Status and dependency:

- [E] Closes phase-4G and B-22; depends on C-1 and C-5; is parallel-safe with C-10. [L116-L119]

Autonomous evaluation, escalation, splitting, model selection, and configuration:

- [E] Adds AutoAdvanceEvaluator, EvidenceCheck, and EvidenceVerdict. [L120]
- [E] Adds EscalationPolicy with four fields; field names are omitted from the summary. [L120]
- [E] Adds SplitTaskPlan, SubTask, and MergeStrategy. [L120]
- [E] Adds SubstituteModelFailed and SplitTaskFailed to EscalationTrigger. [L120]
- [E] Adds a default ModelPool::can_fit_llm method. [L120]
- [E] Adds GateMode::from_cli_args. [L120]
- [E] Adds ResearchTree::split_step. [L120]
- [E] Passes a real TreeAutoAdvancePolicy from cli/main.rs instead of None. [L121]
- [E] Adds per-agent backend configuration through TOML stanzas of the form agents.{role}.backend and the BackendStanza type. [L122]
- [E] Populates phase_4G.toml runtime_measurements with eight keys. [L123]
- [E] Autonomous smoke builds a four-method Rust calculator with tests; acceptance requires at least one gate_decision whose source is AutoAdvanceEvaluator and a positive AutoAdvanceStats.last_5min_count in MonitorWindow. [L124]

### C-10 — Phase 4G.9 Anti-Spiral Primitives

Status and dependency:

- [E] Closes phase-4G 9.x and B-22; depends on C-9 and C-2; is parallel-safe with C-11 through C-15. [L126-L129]

Loop detection, alternate thinking, drift, and watchdog behavior:

- [E] Creates anti_spiral_extended.rs and describes it as containing 11 types. [L130]
- [E] Adds StagnationPattern with four variants, whose names are not present in the summary. [L130]
- [E] Adds AttemptResult. [L130]
- [E] Adds ExecutionHistory with capacity 32. [L130]
- [E] Adds StagnationDetection with a spin threshold of three, an oscillation window of four, and a minimum diminishing-improvement threshold of 5%. [L130]
- [E] Adds ThinkingPersona with five variants and a PERSONA_STRATEGIES table via PersonaStrategy. [L130]
- [E] Adds GoalDriftCanary based on Jaccard similarity over BTreeSet, with green at 0.7 and yellow at 0.4. [L130]
- [E] Adds DriftVerdict. [L130]
- [E] Adds WatchdogTimeoutKind. [L130]
- [E] Adds GenerationWatchdog using a blake3 fingerprint and separately tracking idle, no-progress, and safety timeouts. [L130]
- [E] Adds AdaptiveFutilityTracker. [L130]
- [E] Wires the primitives into autonomous_gate.decide and task_runner step boundaries. [L131]
- [E] Creates persona agent definitions for hacker, researcher, simplifier, architect, and contrarian. [L132]
- [E] Adds StagnationSection to OrchestrationDrawer. [L133]
- [E] Adds get_stagnation_events as a Tauri command. [L133]
- [E] Requires 22 contract tests across two files. [L134]
- [E] Autonomous smoke attacks an impossible bug and expects stagnation_spinning_detected at iteration three, lateral_thinking_activated, and watchdog_timeout. [L135]

### C-11 — Sandbox Completeness (Tier C Isolation)

Status and dependency:

- [E] Closes B-18 and phase-4A; depends on C-9; is parallel-safe with C-12 through C-15; is explicitly safety-critical and requires user review. [L137-L140]

Isolation, limits, IPC, adversarial verification, and platform coverage:

- [E] In clone_cb, applies controls in this order: PR_CAPBSET_DROP loop, resource limits, hidepid=2 mount, and existing seccomp last. [L141]
- [E] Sets RLIMIT_CPU soft/hard values to 60/90 seconds. [L141]
- [E] Sets RLIMIT_NOFILE to 256. [L141]
- [E] Sets RLIMIT_NPROC to 32. [L141]
- [E] Sets RLIMIT_AS to 512 MB. [L141]
- [E] Sets RLIMIT_FSIZE to 100 MB. [L141]
- [E] Wires IpcHookDispatcher into run_agent_session and ProductionAgentRpcServer. [L142]
- [E] Extends the real PID-namespace isolation test to read NSpid from /proc/{pid}/status. [L143]
- [E] Adds a test that the address-space limit is visible. [L144]
- [E] Adds a Windows AppContainer SID test using GetTokenInformation. [L144]
- [E] Adds five adversarial tests covering u32::MAX, self-modification path traversal, locked bash/git writes, and resolved symlinks. [L145]
- [E] Cloud smoke asks for a harmless script that attempts to read /etc/passwd and expects NSpid not equal to 1, EACCES for the read, and no SANDBOX_VIOLATION event. [L146]

### C-12 — Per-Level Auditor Navigator Wiring

Status and dependency:

- [E] Closes B-41 and phase-index-3A-4J; depends on C-1 and C-2 while reusing B-41 types; is parallel-safe with C-11, C-13, C-14, and C-15. [L148-L151]

Audit lifecycle and dashboard:

- [E] Calls per_level_audit_dispatch::emit_started and record_verdict at completion of Concepts, Phases, Steps, BabySteps, and BabyStepDetail. [L152]
- [E] Handles CascadeOutcome Pass. [L153]
- [E] Handles CascadeOutcome RespawnOnce. [L153]
- [E] Handles CascadeOutcome Cascade. [L153]
- [E] Handles CascadeOutcome EscalateToIdea, invoking the B-40 framing retry path. [L153]
- [E] Confirms that auditor.md contains six MODE DISPATCH review prompt blocks. [L154]
- [E] Passes FramedIdea into the Concepts auditor. [L154]
- [E] Replaces get_per_level_verdicts returning an empty vector with live SSB-backed data using the B-36 AppState pattern. [L155]
- [E] Activates AuditTimelineWindow’s already-defined command binding with live data. [L156]
- [E] Requires navigator_audit_wiring and audit_timeline_command contract tests. [L157]
- [E] Cloud smoke builds a three-concept Rust project and expects five distinct per_level_audit_verdict rows with the correct mode. [L158]

### C-13 — IPC Dispatcher Production Swap

Status and dependency:

- [E] Closes B-19 and B-20; depends on C-11; is parallel-safe with C-12, C-14, and C-15. [L160-L163]

Tool dispatch, hooks, observability, and repository hygiene:

- [E] Replaces InProcessToolDispatcher with IpcToolRegistry at the two specified run_subcommand.rs call sites. [L164]
- [E] Retains in-process dispatch only behind a test feature gate. [L164]
- [E] Adds hook_dispatcher to RuntimeWiring and applies it through apply_task_runner_wiring. [L165]
- [E] build_production_hook_dispatcher returns IpcHookDispatcher with RuffPostToolUseRunner enabled. [L165]
- [E] Adds generic AdaptiveTool<T> in crates/tools/src/adaptive_tool.rs. [L166]
- [E] AdaptiveTool emits ten previously missing DAIRA markers: ToolCallStart, ToolCallEnd, Read, Write, Mutation, Branch, Finding, Timeout, Escalate, and Converge. [L166]
- [E] On AUDITING to MEMORIZING, task_runner squashes more than five WIP commits into one. [L167]
- [E] Adds SsbActionType WipSquashComplete and its WIP_SQUASH_COMPLETE label. [L168]
- [E] Requires 15 contract tests across two files. [L169]
- [E] Cloud smoke implements and tests a sum function, expects all 15 total DAIRA markers in SSB, expects RuffPostToolUse on a Python write, and expects a visible WIP squash commit. [L170]
- [A] The plan adds ten missing markers while smoke expects 15 total markers; the original must identify the five pre-existing markers and exact completeness contract.

### C-14 — Phase 0 Debt Part 1: Dead Code Strip (~1959 LOC)

Status and dependency:

- [E] Closes B-15 and phase-0; has no direct prerequisites; is parallel-safe with C-15. [L172-L175]

Removal, migration accounting, and determinism test repair:

- [E] Deletes compact.rs after a caller sweep, with the summary estimating 624 lines. [L176]
- [E] Deletes oauth.rs after confirming CloudGroundingClient OAuth lives in cloud.rs, with the summary estimating 591 lines. [L176]
- [E] Deletes prompt.rs and the legacy “interactive agent” prompt, with the summary estimating 744 lines. [L176]
- [E] Removes ModelPricing, UsageCostEstimate, format_usd, and pricing_for_model from core/src/usage.rs. [L177]
- [E] Removes the corresponding re-export from lib.rs. [L177]
- [E] Fixes determinism_proof.sh by replacing workspace-wide invocation with the seeded_determinism test, explicitly for Windows safety. [L178]
- [E] Adds a command section to migration_manifest.toml containing at least 28 KEEP, STRIPPED, or DEFERRED records. [L179]
- [E] Records actual stripped-line counts in pre_phase_0.toml dead_code_strip. [L180]
- [E] Requires seven dead_code_strip_contract tests. [L181]
- [E] Acceptance requires cargo check across the workspace and zero grep matches for deleted symbols. [L182]

### C-15 — Phase 0 Debt Part 2: ClockProvider + HashMap Ban + Entropy Quarantine

Status and dependency:

- [E] Closes B-15, phase-0, and B-16; depends on C-14; is parallel-safe with C-16 through C-25. [L184-L187]

Determinism substrate, entropy controls, data structures, and CI:

- [E] Adds instant_now to ClockProvider and implements it for OsClock and SeededClock. [L188]
- [E] Defines ELAI_SEED tick-start semantics as zero. [L188]
- [E] Migrates production HashMap usage to FxHashMap in ten files grouped across plugins, dashboard-v2, orchestrator, rules, and knowledge. [L189]
- [E] Requires one commit per crate group for that migration. [L189]
- [E] Introduces an entropy-allow annotation convention for unavoidable SystemTime in cli/commands.rs cache TTL logic and live_cli.rs SSB timestamps. [L190]
- [E] Injects ClockProvider into about 68 production entropy sites. [L191]
- [E] Performs that injection in six batches: knowledge, rules, plugins, focus_tunnel, orchestrator top-level, and CLI. [L191]
- [E] Updates check_entropy_quarantine.sh to honor entropy-allow annotations. [L192]
- [E] Re-enables the entropy-quarantine CI check in the Windows workflow. [L193]
- [E] Adds a combined anti-stale gate to that workflow. [L193]
- [E] Adds clock_provider_injection_contract.rs with four assertions. [L194]
- [E] Determinism smoke runs the same task twice with seed 42 and requires byte-identical action-type sequences while explicitly excluding timestamps. [L195]

### C-16 — Cosmos Pipeline Integration

Status and dependency:

- [E] Closes B-42 and B-35; depends on C-2 and C-8; is parallel-safe with C-17 through C-20. [L197-L200]

Retrieval stage, refresh lifecycle, locking, UI, and tests:

- [E] Adds CosmosStage to QueryPipeline and exposes with_cosmos_stage as its builder. [L201]
- [E] Places CosmosStage between ArtifactStage and ARagStage. [L201]
- [E] Runs CosmosStage only for STANDARD and DEEP; FAST skips it. [L201]
- [E] Extends CosmosWorker::new with an optional shared SSB handle. [L202]
- [E] Emits CosmosRefreshStarted before entering the manifest loop. [L202]
- [E] Emits CosmosArtifactUpdated when artifact processing returns Ok(true). [L202]
- [E] Emits CosmosArtifactDecayed when cosmos_get reports Decayed. [L202]
- [E] Spawns a task_runner tick loop at a 3,600-second cadence. [L203]
- [E] Supports cancellation through CancelHandle. [L203]
- [E] Protects refresh/index work with IndexLock using LockScope::Index. [L203]
- [E] Replaces trigger_cosmos_refresh NotImplemented behavior with a one-shot Tokio task. [L204]
- [E] Documents the 3x versus 2x decay-threshold divergence as an intentional grace buffer without changing behavior. [L205]
- [E] Makes CosmosDrawer subscribe to live SSB events in addition to its 30-second poll. [L206]
- [E] Requires standalone tests for worker tick, lock contention, pipeline stage, decay contract, and briefing budget, with the approximate LOC given in the summary. [L207]
- [E] Research smoke configures knowledge artifacts, queries a repos endpoint, and expects CosmosRefreshStarted, Fresh status, and AVAILABLE_COSMOS in the prompt. [L208]

### C-17 — Knowledge Indexing Resilience

Status and dependency:

- [E] Closes B-30, B-31, B-31.5, B-34, and phase-3K; depends on C-15 and C-14; is parallel-safe with C-16 and C-18 through C-20. [L210-L213]

Language support, restartability, watching, chunks, retrieval, entry points, scale measurement, and UI:

- [E] Adds KotlinGrammar using tree-sitter-kotlin when available and JavaGrammar as fallback. [L214]
- [E] Registers kt and kts extensions in ext_to_lang_id and LanguageRegistry. [L214]
- [E] Wires auto_resume_indexed_projects into CLI startup. [L215]
- [E] Handles SIGINT through ctrl_c. [L215]
- [E] Handles SIGTERM through SignalKind::terminate. [L215]
- [E] Adds FileWatcherService in watcher_service.rs. [L216]
- [E] FileWatcherService acquires LockScope::Watch through IndexLock::try_acquire. [L216]
- [E] Adds Heading and Paragraph to ChunkKind. [L217]
- [E] Adds non-code splitters for Markdown files. [L217]
- [E] Replaces AragOrchestrator’s embedding-norm proxy with real ONNX dot-product search while preserving 3x oversampling. [L218]
- [E] Pre-seeds research Oracle context with detect_entry_points output under PROJECT_STARTING_POINTS. [L219]
- [E] Emits EntryPointsDetected to SSB. [L219]
- [E] Fixes run_tier_a.sh to calculate per-batch p50 and p99 from batch_complete tracing spans. [L220]
- [E] Populates phase_b31_5_tier_* TOMLs using real scale runs against Linux v6.7 and VS Code 1.94.2. [L221]
- [E] Adds SSB-driven IndexResume and LockContested banners to ActivityFeed. [L222]
- [E] Adds svelte_extraction.rs and indexing_resilience_contract.rs tests. [L223]
- [E] Smoke indexes a Rust/Kotlin/Svelte/Markdown fixture, interrupts midway with SIGINT, restarts, and requires an IndexResume row plus Kotlin entries in the DKG. [L224]

### C-18 — Test Infra Backfill: Contract Tests Specified but Never Created

Status and dependency:

- [E] Closes B-2, B-29, B-32, B-33, B-35, B-36, phase-3B, phase-3D, and phase-3H; depends on C-1, C-2, and C-13; is parallel-safe with C-19 and C-20. [L226-L229]
- [E] Changes only tests and makes zero production-code changes. [L230]

Backfilled verification:

- [E] Adds fsm_phase_transition_contract.rs for B-2. [L231]
- [E] Adds symbol_query_dispatch.rs, exercising four symbol_* tools against an in-memory SymbolGraph, for B-29. [L232]
- [E] Adds test_minified_file_emits_ssb_action to chunking_minified.rs for B-32. [L233]
- [E] Adds graph_export_dispatch.rs for B-33, checking Mermaid output and two-run determinism. [L234]
- [E] Adds grounding_artifact_check.rs with three ArtifactCheck cases for B-35. [L235]
- [E] Adds designer_artifact_briefing.rs for B-35. [L236]
- [E] Adds dashboard_v2 D1, D2, D3, D6, and D11 contract tests for B-36. [L237]
- [E] Adds SSB payload-mismatch and write-sequence-determinism tests for phase-3B. [L238]
- [E] Adds BSP checkpoint-roundtrip and five-superstep mixed-channel tests for phase-3D. [L239]
- [E] Adds SessionDag 100-run determinism and exact 85% boundary tests for phase-3H. [L240]
- [E] Acceptance requires all new tests to pass with no existing regression. [L241]
- [A] The plan says “at least ten” files but mixes files and appended test modules; exact file/test counts require C-18.

### C-19 — Test Infra Backfill: Adversarial Cases + Criterion Benchmarks

Status and dependency:

- [E] Closes phase-4A, phase-4C, phase-4D, phase-4F, and B-9; depends on C-11 and conditionally on C-22’s stub_result_v2 reference types; is parallel-safe with C-18 and C-20. [L243-L246]

Adversarial and performance verification:

- [E] Adds 15 phase-4A cases to adversarial_ipc.rs. [L247]
- [E] Named adversarial categories are u32::MAX, SO_PEERCRED mismatch, a 10,000-ThinkStep burst, self-modification variants, MINOR/MAJOR/no-evidence replans, a discovery PROPOSED gate, inflated patterns, and crash invariants. [L247]
- [E] Creates a Criterion benchmark for plugin tiered analysis. [L248]
- [E] Requires Tier 1 p50 below 5 ms. [L248]
- [E] Tests Tier 2 around exact size boundaries 4,999, 5,000, and 5,001 and requires p50 below 50 ms. [L248]
- [E] Requires Tier 3 p50 below 500 ms. [L248]
- [E] Benchmarks PostToolUse. [L248]
- [E] Adds nine quantified metric sections to phase_4C.toml. [L249]
- [E] Adds a benchmark regression gate requiring delta_pct at least -5.0. [L250]
- [E] Adds four adversarial research tests: filesystem escape, direct network access, GRC budget exhaustion, and an ALL-TRAINING HIGH_RISK case. [L251]
- [E] Adds feasibility adversarial tests A1 through A8. [L252]
- [E] Adds T9 and T13 through T16 plus four baby-step feasibility tests to feasibility_probe_contract.rs. [L252]
- [E] Adds B-9 named tests for session merge/close slash commands and BranchTree fork/merge/close property contracts. [L253]
- [E] Acceptance requires all 15 sandbox tests to pass and A1–A8 to catch an infinite-loop stub within 30 seconds. [L254]

### C-20 — Test Infra: Metrics TOML Backfill + Measurement Suite

Status and dependency:

- [E] Closes phase-4G, phase-4F, phase-4B, phase-4M, phase-4C, B-29.5, and B-31.5; depends on C-9, C-13, and C-17; is parallel-safe with C-18 and C-19. [L256-L259]

Measurement CLI, config outputs, isolation, and scale tests:

- [E] Adds the elai measure command. [L260]
- [E] Defines MeasureScope variants All, Phase(String), Scale, and Cloud. [L260]
- [E] Writes the last measurement run to metrics/last_measure_run.toml. [L260]
- [E] Adds an ELAI_DEBUG_BATCH_DELAY_SECS measurement loop. [L261]
- [E] Activates that loop only when ELAI_MEASURE_MODE equals 1, promising zero production overhead otherwise. [L261]
- [E] Creates phase_4B.toml fields for IPC overhead, per-language compile-check latency, and ShortCoder savings percentage. [L262]
- [E] Adds measurements/runtime_measurements, backend_latency/backend_health, or runtime_metrics sections as appropriate to phase_4G, phase_4F, phase_4M, and phase_4C TOMLs. [L263]
- [E] Requires real measurements, forbids placeholders, and permits explicit unmeasured_reason annotations. [L263]
- [E] Populates phase_b29_5_scale.toml with Linux, Chromium, VS Code, and PostgreSQL tier runs. [L264]
- [E] Creates phase_1F.toml marked UNAVAILABLE. [L265]
- [E] Creates a phase_1G quantization sweep. [L265]
- [E] Verifies phase_1H.toml. [L265]
- [E] Adds scripts/scale_test/run_all_tiers.sh. [L266]
- [E] Measurement smoke runs all scopes in measurement mode, requires every phase TOML field to be non-placeholder, and requires phase_4M backend latency to be populated by a hybrid task. [L267]

### C-21 — Self-Modification + Anti-Staleness Automation

Status and dependency:

- [E] Closes B-16, B-23, phase-4H, and anti-staleness-enforcement; depends on C-15; is parallel-safe with C-16 through C-20. [L269-L272]

Automation, safety, closed configuration, refresh, profiling, corpus, and memory:

- [E] Wires check_anti_stale.sh into Windows CI. [L273]
- [E] Conditionally, only after a user answers Y, converts about 12 unwrap calls in deterministic_auditor and dkg into propagation or ok_or handling. [L274]
- [E] Creates install_pre_commit_hook.sh. [L275]
- [E] The installed pre-commit hook blocks commits that modify files owning LOCKED_KEYS. [L275]
- [E] Maps DepAuditReport into FullPipelineOutcome.stale_api_catches through dep_audit_to_stale_catches. [L276]
- [E] After promotion of DataTable::DeprecatedPatterns in self_mod_bridge, starts compute_steering_vector.py asynchronously without waiting. [L277]
- [E] Emits steering_vector_refresh_triggered for that refresh. [L277]
- [E] Starts nightly dependency audit when Layer1A becomes DIVERGING. [L278]
- [E] Rate-limits that trigger to once every 24 hours through a sentinel file. [L278]
- [E] Replaces the open AllowedConfigKey::Parameter(String) surface with a closed ParameterKey enum. [L279]
- [E] Enumerated ParameterKey forms are EtcThreshold, FreshnessHalfLife, FreshnessDocHalfLife, FreshnessBoostNorm, BabyStepMaxLoc by VRAM tier, ContextSlotWeight by slot, RetryBudget, AbMctsMaxBranches, AbMctsTemperatures, MicroReflectionMaxTokens, SteeringVectorCoefficient, and SelfModFrequency. [L279]
- [E] Populates ConstraintsBlock in project_profile.rs from rules.db rows where deprecated_patterns severity is BLOCK. [L280]
- [E] Creates a SWE-bench seed corpus directory with trigger_task.toml, task_01 through task_10 TOMLs, and README.txt. [L281]
- [E] Records MistakeBook entries for T1_BLOCK, T2_BLOCK, and PRE_TOOL_BLOCK events. [L282]
- [E] Acceptance requires deprecated-pattern promotion to change modern_api.npz modification time, LOCKED_KEYS commits to be blocked, and an arbitrary string ParameterKey construction to fail compilation. [L283]

### C-22 — FeasibilityProber Completeness (Phase 4F)

Status and dependency:

- [E] Closes B-21 and phase-4F; depends on C-19 and C-9; is parallel-safe with C-21 and C-23. [L285-L288]

Probe schema, resource accounting, lifecycle, wiring, events, UI, and metrics:

- [E] Renames SystemProbeReport to SystemProbeResult throughout the codebase. [L289]
- [E] Adds uuid to GpuInfo by parsing nvidia-smi -q -d UUID. [L290]
- [E] Adds ResourceDelta with VRAM and RAM before/after byte counts. [L291]
- [E] Extends StubResult with alternative_index, stdout_hash, stderr_summary, and resource_delta. [L291]
- [E] Uses serde defaults on all new StubResult fields for compatibility. [L291]
- [E] Captures resource deltas around stub execution using SystemProbeResult queries before and after. [L292]
- [E] Removes kill_on_drop(true) from the feasibility child process. [L293]
- [E] Makes explicit child.kill().await the sole termination mechanism. [L293]
- [E] Keeps shipped SSB names system_probe, feasibility_report, novel_step_detected, and stub_run. [L294]
- [E] Adds alternative_explored and feasibility_escalated SSB names. [L294]
- [E] Wires a real DefaultFeasibilityProber in cli/main.rs and runtime_setup.rs instead of None. [L295]
- [E] Creates FeasibilityPanel.svelte. [L296]
- [E] Displays probe_count, novel_step_count, alternatives_explored, skip_reason, and a per-concept accordion. [L296]
- [E] Mounts FeasibilityPanel in ReasoningDrawer and clears unwired.feasibility. [L296]
- [E] Populates six phase_4F metric fields from a smoke run. [L297]
- [E] Cloud research smoke uses an unfamiliar candle-rs library, expects non-null VRAM ResourceDelta, and requires FeasibilityPanel to render. [L298]

### C-23 — SessionDag Branch UI

Status and dependency:

- [E] Closes B-9, B-3, and phase-3H; depends on C-3 and C-4; is parallel-safe with C-21, C-22, and C-24. [L300-L303]

Branch commands, UI, handoff, ordering, data model, and events:

- [E] Adds merge and close arms to the /session command parser. [L304]
- [E] Replaces “use CLI” stubs in fork_session_branch, merge_session_branch, and close_session_branch Tauri commands with real SessionDag calls. [L305]
- [E] Requires AppState to hold a DAG handle and uses ssb_writer for those mutations. [L305]
- [E] Mounts BranchTree.svelte in SessionsDrawer. [L306]
- [E] Wires BranchTree’s onfork, onmerge, and onclose callbacks to live IPC. [L306]
- [E] Changes CommandWindow /continue from text-area prepopulation to invoking load_handoff. [L307]
- [E] Fixes parent_chain to be root-to-tip by reversing the accumulated chain and updates tests accordingly. [L308]
- [E] Adds a 100-run SessionDag determinism test. [L309]
- [E] Verifies SettingsWindow SessionHandoffEntry does not contain stale V1 fields. [L310]
- [E] Adds branch_forked, branch_merged, and branch_closed action types. [L311]
- [E] Renders branch-event banners in ActivityWindow and SessionsDrawer. [L311]
- [E] Long-task smoke crosses the 85% context threshold, resumes through /continue, then forks, merges, and closes a named branch and expects SSB checkpoint rows. [L312]

### C-24 — Symbol Impact Dashboard Surface

Status and dependency:

- [E] Closes B-29 and B-33; depends on C-17; is parallel-safe with C-21, C-22, C-23, and C-25. [L314-L317]

Agent tools, incremental graph, user query, deterministic export, graph visualization, and blast-radius affordance:

- [E] Adds ToolDefinition catalog entries for symbol_impact, symbol_flow, symbol_context, and symbol_list so the agent can select already-implemented dispatcher arms. [L318]
- [E] Updates the symbol graph on the incremental indexing path when changed_file_count is at most INCREMENTAL_SYMBOL_THRESHOLD, fixed at 50. [L319]
- [E] Emits SymbolGraphUpdated with is_incremental true for that path. [L319]
- [E] Adds a target form and Run button to KnowledgeDrawer’s IMPACT ANALYSIS panel. [L320]
- [E] Submits impact work by invoking submit_task with a /symbol_impact target command. [L320]
- [E] Adds SlashCommand::GraphMermaid. [L321]
- [E] Produces Mermaid graph output without an LLM and deterministically. [L321]
- [E] Adds DispatchTool::ExportGraphMermaid and delegates to the knowledge Mermaid generator. [L322]
- [E] Adds Cytoscape selectors for Concept, Phase, Step, and BabyStep decomposition shapes. [L323]
- [E] Adds pending, running, passed, failed, and blocked graph-status classes. [L323]
- [E] Shows those decomposition controls only in decomposition mode. [L323]
- [E] Adds “Show blast radius” to ActivityWindow file-write rows. [L324]
- [E] That action calls graph state request_impact and opens a graph window. [L324]
- [E] Adds graph export assertions and four Tauri graph-command smokes. [L325]
- [E] Cloud rename smoke expects the agent to self-select symbol_impact and return a non-empty blast radius. [L326]

### C-25 — Compiled Pipeline + Template Crystallizer

Status and dependency:

- [E] Closes guide/compiled-pipeline-design and guide/post-phase4-implementation-order; depends on C-9; is parallel-safe with C-21 through C-24 and C-26. [L328-L331]

Recurring-pipeline compilation:

- [E] Adds CompiledPipeline, CompiledStep, TemplateVariable, and VariableType. [L332]
- [E] VariableType contains ConceptClass, TaskDescription, CodeTarget, and Other. [L332]
- [E] TemplateCrystallizer promotes a shape after three consecutive successes. [L332]
- [E] Promotion emits template_crystallized to SSB. [L332]
- [E] CompiledExecutor reports CacheMiss on a missing compiled pipeline. [L332]
- [E] Two failures or confidence below 0.7 cause Decompile and emit decompile_triggered. [L332]
- [E] CompiledPipelineStore persists MessagePack at .elai/cache/compiled_pipelines.msgpack. [L332]
- [E] Stored pipelines are evicted after a seven-day TTL. [L332]

Background-task management:

- [E] Adds BackgroundTaskManager with maximum concurrency three. [L333]
- [E] A soft timeout automatically detaches a task. [L333]
- [E] A hard timeout aborts a task. [L333]
- [E] Polling adapts from 5 seconds to 10 seconds to 30 seconds. [L333]
- [E] gated_spawn requires autonomous_mode. [L333]
- [E] Adds BackgroundTaskStarted and BackgroundTaskComplete event types and payloads. [L333]

Token attention accounting and integration:

- [E] Adds CumulativeTokenTracker. [L334]
- [E] Records ProviderUsage responses. [L334]
- [E] Recalibration sets attention_ceiling to the greater of 4,096 and last_response_prompt plus 2,048. [L334]
- [E] Runs the tracker beside SpendAccumulator without changing billing. [L334]
- [E] Reads cache_read_tokens from openai_compat.rs at the stated location 363. [L334]
- [E] Pipeline builder tries a compiled template before generating a fresh pipeline. [L335]
- [E] Adds PlanSkeletonSource values Template and Llm to PlanSkeletonPayload. [L335]
- [E] Runs TemplateCrystallizer as a ResearchTreeConfig post-step hook. [L336]
- [E] Adds /pipeline list and /pipeline reset CLI commands. [L337]
- [E] Repeating the same counter-struct task three times must crystallize the template on run three; run four must use Template, show cache reads, and consume fewer tokens. [L338]
- [E] A long autonomous bash task must emit BackgroundTaskStarted and BackgroundTaskComplete with auto_detached true. [L338]

### C-26 — Constrained Decoding + Retry/Futility Primitives

Status and dependency:

- [E] Closes guide/token-efficiency-design and guide/update-implementation-order; depends on C-10 and C-25; is parallel-safe with C-21 through C-24. [L340-L343]

Decoding, retry, futility, provider fallback, grammar assets, and measurement:

- [E] Adds DecodingMode variants JsonSchema, Grammar with ebnf_path and crane_adaptive, Regex, and FreeText. [L344]
- [E] Adds ConstrainedDecodingConfig with max_retries_on_invalid defaulting to three and strict defaulting to true. [L344]
- [E] Adds crane_adaptive_eligible. [L344]
- [E] Sets per-role defaults: designer FreeText, executor Grammar using function_signature.ebnf with crane-adaptive enabled, critic JsonSchema, auditor JsonSchema, and researcher FreeText. [L344]
- [E] Adds RetryStrategy to BubbleUpPolicy. [L345]
- [E] Uses temperature retry schedule 0.0, 0.2, 0.5. [L345]
- [E] Stops same-error retries at three. [L345]
- [E] Defaults decompose_on_failure to false. [L345]
- [E] NavigatorTree detects repeated equal error classes in a VecDeque. [L346]
- [E] NavigatorTree detects output stagnation when GoalDriftCanary similarity exceeds 0.90. [L346]
- [E] Emits futility_detected and futility_output_stagnation. [L346]
- [E] Switches persona after futility detection. [L346]
- [E] SidecarClient maps constraints to response_format or grammar_path. [L347]
- [E] If constrained decoding is unavailable, SidecarClient warns and falls back to FreeText. [L347]
- [E] CloudGroundingClient implements JsonSchema through Anthropic tools plus tool_choice. [L347]
- [E] Fixes a benchmark-fixture path. [L348]
- [E] Creates fixtures/templates.json and fixtures/padding.txt. [L348]
- [E] Replaces zero placeholders with real non-zero model-baseline values for measurements three, four, and five. [L348]
- [E] Adds import_block.ebnf, function_signature.ebnf, and error_type.ebnf. [L349]
- [E] Requires 15 contract tests across two files. [L350]
- [E] Smoke A expects JsonSchema to pass without retry and log constrained decoding use. [L351]
- [E] Smoke B forces Regex failure three times and expects futility detection plus persona switch. [L351]
- [E] Smoke C requires non-zero benchmark deltas for measurements three through five. [L351]

### C-27 — Testing-Trust T(-1) Determinism + T-0 Wiring Contract

Status and dependency:

- [E] Closes testing-trust T(-1) and T-0; depends on C-15 and C-17; is parallel-safe with C-21 through C-26. [L353-L356]

Serialization and event model:

- [E] Declares the only serialization mutation in its wave and says SCHEMA_VERSION moves from 46 to 47. [L357]
- [E] Adds WiringEvidence, WiringViolation, and DeterminismDrift plus WiringViolationKind. [L358]

Determinism model:

- [E] Adds DeterminismConfig fields hardware_class, seed, an eight-byte blake3_cache_prefix, and system_fingerprint_pin. [L359]
- [E] Adds HardwareClass, GpuSku, and environment loading from ELAI_SEED and ELAI_HARDWARE_CLASS. [L359]
- [E] Derives cache-key prefixes with blake3 keyed hashing and explicitly forbids sha256 for this purpose. [L359]
- [E] DriftComparator emits DeterminismDrift at the first divergent index. [L359]

Wiring audit:

- [E] Adds WiringContract, StubBanEntry, ObservableEffect, EntrypointSet, and EntrypointSetLoader. [L360]
- [E] EntrypointSetLoader reads TOML wiring.entrypoints. [L360]
- [E] Missing entrypoint configuration produces an empty set plus a warning. [L360]
- [E] StubBanAuditor compiles patterns for todo!, unimplemented!, Ok(Default::default()), “deferred”, and “stub”. [L360]
- [E] Stub scanning excludes tests. [L360]
- [E] DkgReachabilityAuditor performs breadth-first search from every entry point. [L360]
- [E] ObservableEffectRunner polls SQLite for one second under a Tokio timeout. [L360]
- [E] Observable timeout returns BrokenObservable instead of panicking. [L360]
- [E] Adds elai wiring audit with optional config and contracts paths. [L361]
- [E] The command parses Markdown WIRING_CONTRACT blocks and aggregates a WiringAuditReport. [L361]
- [E] It exits zero when clean and one on violations. [L361]
- [E] Adds a Windows push/PR workflow for main. [L362]
- [E] Smoke A requires clean-audit WiringEvidence rows. [L363]
- [E] Smoke B injects a stub and requires a WiringViolation containing file and line. [L363]
- [E] Smoke C repeats seed 42 and requires no DeterminismDrift. [L363]
- [E] Smoke D breaks an observable and requires BrokenObservable within the timeout plus five seconds. [L363]

### C-28 — Testing-Trust T-1 Mutation Testing

Status and dependency:

- [E] Closes testing-trust T-1; depends on C-27; is parallel-safe with C-29 and C-21 through C-26. [L365-L368]

Tooling, tier integration, model, cache, configuration, gate, and UI:

- [E] Pins cargo-mutants 27.0.0 and mutmut 3.5.0 as CLI tools, not Cargo dependencies. [L369]
- [E] Inserts ValidationTier::Mutation between Unit and Integration at position three. [L370]
- [E] Adds CheckKind::Mutation to deterministic_auditor. [L370]
- [E] Adds MutantOutcome values Caught, Survived, Timeout, and Skipped(String). [L371]
- [E] Adds MutantRecord and MutationResult. [L371]
- [E] Computes mutation score as caught divided by caught plus survived plus timeout, multiplied by 100. [L371]
- [E] Adds MutationRunner. [L371]
- [E] RustMutationRunner invokes cargo mutants with timeout, output, and jobs 1. [L371]
- [E] PythonMutationRunner invokes mutmut run with pytest -x -q. [L371]
- [E] Both runners use Tokio processes and per-mutant Tokio timeouts. [L371]
- [E] Adds a blake3-keyed SQLite EquivalentMutantCache at .elai/mutation_cache.sqlite3. [L372]
- [E] Cache entries expire after 30 days. [L372]
- [E] Reads mutation config from elai_cc.toml testing.mutation with serde defaults when absent. [L373]
- [E] Config fields/defaults are enabled, threshold_pct 70, timeout_per_mutant_sec 30, and max_mutants_per_session 100. [L373]
- [E] Adds MutationSurvived in the four required lockstep event-registration sites while keeping SCHEMA_VERSION at 47 as a shared wave bump. [L374]
- [E] DeterministicAuditor runs mutation after its existing five checks pass. [L375]
- [E] A score below threshold creates DeterministicFinding::Block and blocks AUDITING to MEMORIZING. [L375]
- [E] MonitorWindow displays a mutation gauge: green at or above threshold, amber below threshold, and grey for null. [L376]
- [E] Adds a get_mutation_gauge Tauri-command stub for that gauge. [L376]
- [E] Requires two contract files with four and three tests. [L377]
- [E] Normal sum smoke requires at least 70% mutation score; tautological-test smoke must block at auditing; a second run must be faster due to cache replay. [L378]

### C-29 — Testing-Trust T-2 IntentPair + T-3 Basic Metamorphic Relations

Status and dependency:

- [E] Closes testing-trust T-2 and T-3; depends on C-27 and C-28; is parallel-safe with C-21 through C-26. [L380-L383]

Intent contract, confirmation gate, metamorphic test library, UI, and events:

- [E] Keeps SCHEMA_VERSION 47 as the shared C-27/C-28 wave bump. [L384]
- [E] Adds IntentPair fields natural_language_intent, concrete_test_assertion, and baby_step_ref. [L385]
- [E] Adds IntentPairSet fields pairs, confirmed, and confirmed_at. [L385]
- [E] Adds IntentConfirmation and ConfirmedBy values User and AutonomousGate. [L385]
- [E] Adds a serde-defaulted intent_pairs field to BabyStepSpec without migration. [L386]
- [E] Adds IntentConfirmed and MrViolated events and payloads. [L387]
- [E] Defaults testing.intent min_pairs to three. [L388]
- [E] Defaults metamorphic testing to enabled with identity and inverse relations. [L388]
- [E] Excludes permutation_invariance by default. [L388]
- [E] At PLANNING to TESTING_FIRST, fewer than the minimum pairs emits a gate_decision with reason intent_pairs_below_threshold and rejects the transition. [L389]
- [E] Meeting the minimum emits IntentConfirmed. [L389]
- [E] AutonomousGate auto-confirms qualifying pairs. [L389]
- [E] Adds MetamorphicRelation values Identity, Inverse, and PermutationInvariance. [L390]
- [E] RelationLibrary loads relations from config and warns while skipping unknown strings. [L390]
- [E] MetamorphicTestRunner has run_identity, run_inverse, run_permutation_invariance, and run_all. [L390]
- [E] Inverse testing checks decode(encode(x)) equals x for each sample and records the first failing counterexample. [L390]
- [E] Runs metamorphic tests after mutation when enabled. [L391]
- [E] MrViolated rows contribute to the AUDITING gate score. [L391]
- [E] Adds add_intent_pair and confirm_intent_pairs Tauri commands. [L392]
- [E] Adds a three-field IntentPairEditor to PlanWindow. [L393]
- [E] Enables Confirm only at pair count three or greater. [L393]
- [E] Smoke begins blocked, uses the editor to add three pairs, and expects IntentConfirmed; a correct encoder has no MrViolated, while a buggy encoder emits an inverse violation with a counterexample. [L394]

### C-30 — Surgical Mode + Oracle Mode + Pipeline Infrastructure

Status and dependency:

- [E] Closes pipeline-modes-design, update-implementation-order, and phase-4D; depends on C-17 and C-24; is parallel-safe with C-26 through C-29. [L396-L399]

Surgical decomposition:

- [E] Adds SurgicalPlan, SurgicalStep, AffectedSet, ComplexityClass, RegressionGate, TestBaseline, and SurgicalError. [L400]
- [E] ComplexityClass values are Simple, Medium, and Complex. [L400]
- [E] SurgicalDecomposer uses real DKG blast-radius traversal via DkgQuerierAdapter::query_reachability. [L400]
- [E] Simple means at most three files and at most 200 LOC. [L400]
- [E] Medium means at most eight files or at most 500 LOC. [L400]
- [E] All other work is Complex. [L400]
- [E] Step caps are one for Simple, four for Medium, and eight for Complex. [L400]
- [E] RegressionGate snapshot_hash is blake3 over sorted must_pass tests. [L400]
- [E] More than 500 LOC emits surgical_loc_warning. [L400]
- [E] In interactive mode, more than 500 LOC requires confirmation. [L400]
- [E] In autonomous mode, more than 1,000 LOC escalates. [L400]
- [E] --surgical dispatches SurgicalDecomposer before research_decompose. [L401]

Project detection:

- [E] Cargo.toml is a REQUIRED Rust signal; at least ten .rs files is a STRONG signal. [L402]
- [E] go.mod and pyproject.toml are REQUIRED signals for their ecosystems. [L402]
- [E] package.json is a STRONG signal. [L402]
- [E] The first interactive task presents a project-detection gate prompt. [L402]
- [E] Emits project_detection_signal. [L402]

Oracle cache and DKG tools:

- [E] Adds OracleAnswerCache keyed by blake3 query_hash and project_hash arrays of 32 bytes. [L403]
- [E] CachedAnswer contains answer, cached_at, and source_tier. [L403]
- [E] Tier-1 TTL is 2,592,000 seconds/30 days; Tier-2 is 604,800 seconds/7 days; Tier-3 is 86,400 seconds/1 day; Tier-4 is zero and never cached. [L403]
- [E] Evicts least-recently-used entries at max_entries. [L403]
- [E] Integrates OracleAnswerCache into the /research pipeline. [L404]
- [E] Emits oracle_cache_hit with cached_age_secs and source_tier, or oracle_cache_miss. [L404]
- [E] Adds dkg_callers for inbound CALLS, dkg_dependents for outbound IMPORTS/USES, dkg_data_flow for PARAMETER/RETURN/ASSIGNMENT/FIELD_ACCESS, and dkg_subgraph for bounded-depth BFS. [L405]
- [E] Adds at least five research.vram_tiers config entries matching research_tier.rs: 8gb, 12gb, 12gb_7b, 16gb, 24gb, and cloud are named, creating a count ambiguity of six labels. [L406]
- [E] Requires 17 contract tests. [L407]
- [E] Smokes require a surgical plan and passing regression gate, an Oracle miss then hit, and agent self-selection of DKG tools. [L408]

### C-31 — Code Completeness Layer 1: FilePlan + FileLineTracker

Status and dependency:

- [E] Closes code-completeness-enforcement-design, update-implementation-order, and phase-4E; depends on C-13 and C-9; is parallel-safe with C-32. [L410-L413]

Per-file plan, budgets, write accounting, failure memory, and adaptive limits:

- [E] Adds FunctionSpec. [L414]
- [E] Adds FileSpec fields path, language, max_lines, and functions. [L414]
- [E] Adds FileModSpec with max_added_lines. [L414]
- [E] Adds FilePlan with new_files, modified_files, and version. [L414]
- [E] FilePlan validates against LanguageLimitsMap. [L414]
- [E] Adds LineBudgetExceeded and FunctionCountExceeded errors. [L414]
- [E] Fixes MAX_FUNCTIONS_PER_FILE at 20. [L414]
- [E] Adds FILE_PLAN as the ninth BabyStepSpec field using a serde default and no schema bump. [L415]
- [E] Adds cumulative_lines_by_step to TreeNavigatorState through FileLineTracker. [L416]
- [E] Adds a u64 step_write_generation as a TOCTOU guard. [L416]
- [E] record_lines_written fails with NavigatorError::WriteGeneration on generation mismatch. [L416]
- [E] Adds MistakeRecorder with a default no-op implementation. [L417]
- [E] Wires MistakeRecorder into both T1HookConfig and T2HookConfig. [L417]
- [E] T1/T2 blocks record LazyCode, EmptyBody, or FileSizeExceeded. [L417]
- [E] Adds a T2 function-count guard that compares the per-file AST count to planned functions plus FUNCTION_COUNT_SLACK two. [L418]
- [E] Exceeding that count blocks. [L418]
- [E] FileDisciplineConfig derives language overrides from ProjectProfile language statistics once at least five files of that language are detected. [L419]
- [E] The shown production warning formula is the minimum of the spec default and project median times 1.5; other derived values are omitted from the summary. [L419]
- [E] Requires 16 contract tests. [L420]
- [E] CRUD smoke requires every baby step to have a populated file plan; lazy code should produce MistakeBook evidence; a plan above 400 LOC must be rejected before dispatch. [L421]

### C-32 — Code Completeness Layer 2: Skeleton-First + MiniCritic + AcceptAndComplete

Status and dependency:

- [E] Closes code-completeness-enforcement-design and token-efficiency-design; depends on C-31 and C-26; is parallel-safe with C-21 through C-30 due to disjoint file scope. [L423-L426]

Skeleton-first generation:

- [E] Adds BlankId represented by u8 and rendered as BLANK_N. [L427]
- [E] Adds SkeletonSpec, FillResult, and FilledBuffer. [L427]
- [E] FilledBuffer completeness is non-BLOCK fills divided by blank count and clamped. [L427]
- [E] SkeletonGenerator preserves signatures while replacing function bodies with blank markers. [L427]
- [E] fill_blanks runs MiniCritic on each fill and performs zero disk writes. [L427]
- [E] flush_to_disk performs exactly one fs::write after substituting every blank marker. [L427]

MiniCritic:

- [E] MiniCriticVerdict values are PASS, WARN, and BLOCK. [L428]
- [E] The first BLOCK short-circuits later checks. [L428]
- [E] Check one reuses the Tier-1 regex subset by borrowing LazyLock Regex statics from tiered_analysis.rs rather than duplicating patterns. [L428]
- [E] Check two blocks fragments shorter than three lines with reason too_short. [L428]
- [E] Check three parses the fragment with tree-sitter and blocks any ERROR node. [L428]
- [E] Emits mini_critic_verdict for every call. [L428]

Structural retry and completion:

- [E] Adds StructuralRetryStage values Decompose, Template, FewShot, and BestOfN. [L429]
- [E] Adds StructuralRetryPlan and a Sequencer that runs those stages. [L429]
- [E] PASS yields Converged. [L429]
- [E] Final fallback is AcceptAndComplete when completeness ratio is at least 0.5, otherwise Exhausted. [L429]
- [E] BestOfN generates three candidates. [L429]
- [E] Candidate scoring gives compile two points, critic PASS two points, and lines one point. [L429]
- [E] Retry prompts may not contain instructional “try again” text. [L429]
- [E] Emits structural_retry_stage_attempt for each stage. [L429]

Bridge files, integration, UI, and events:

- [E] Adds deterministic BridgeKind values ModRsAggregator, InitPyBarrel, IndexTsBarrel, and GoPackageDoc. [L430]
- [E] BridgeFileTemplate generation is LLM-free, lexically sorted, and byte-identical at ELAI_SEED 42. [L430]
- [E] write_bridge uses one fs::write and emits bridge_file_generated. [L430]
- [E] task_runner uses SkeletonGenerator and StructuralRetrySequencer for baby steps with a C-31 FilePlan. [L431]
- [E] MonitorWindow shows completeness ratio red below 0.5, yellow from 0.5 to below 0.9, and green at least 0.9. [L432]
- [E] Adds get_completeness_ratio. [L432]
- [E] ReasoningDrawer renders mini-critic rows and offers a Show MiniCritic toggle. [L432]
- [E] Registers skeleton_generated, skeleton_flushed, mini_critic_verdict, structural_retry_stage_attempt, and bridge_file_generated. [L433]
- [E] Requires 14 contract tests. [L434]
- [E] Main smoke requires at least 0.9 completeness, no residual blanks, and successful cargo check; adversarial TODO fill must block and enter structural retry, ending in convergence or AcceptAndComplete. [L435]

### C-33 — Documentation and Phase Map Updates

Status and dependency:

- [E] Closes update-phase-map, B-0_INDEX, B-39, B-40, and B-21; depends on C-30 and C-22; is otherwise parallel-safe with all plans because only failure_taxonomy.rs changes code. [L437-L440]

Local-only specification changes and reconciliation:

- [E] Warns that docs/specs is gitignored and all new step files are local-only; this must be flagged at session end. [L441]
- [E] Creates phase-3L.txt as an eight-step Project Intelligence specification. [L442]
- [E] Creates phase-3J-BASELINE.txt as a token-efficiency benchmark harness. [L442]
- [E] Creates phase-4D-SURGICAL.txt for SurgicalDecomposer, cross-referencing C-30. [L442]
- [E] Creates phase-4D-ORACLE.txt for OracleAnswerCache. [L442]
- [E] Adds to phase-3J: ETCLogitsProcessor; GG mean_logprob and entropy_mean signals; constrained decoding; tool routing A-26; bash safety A-27; git_commit A-28; action classifier A-31; and steering-vector batch A-8. [L443]
- [E] Adds to phase-4G: AutoAdvanceEvaluator handling for reasoning exhaustion and CloudSpendTracker escalation. [L443]
- [E] Adds LazyCode, EmptyBody, and FileSizeExceeded to FailureCategory. [L444]
- [E] Adds corresponding detect_category and suggest_fix arms. [L444]
- [E] Keeps ToolFailureCategory synchronized after first checking elai.map. [L444]
- [E] Changes B-39 REFINE_CAP from two to three based on the cited RefineBench table and adds a contract assertion for three. [L445]
- [E] Reconciles B-21 event names to system_probe, feasibility_report, novel_step_detected, and stub_run. [L446]
- [E] Marks feasibility_escalated deferred in B-21 until C-22. [L446]
- [E] Updates B-0_INDEX status blocks for B-37, B-38, B-40, B-41, and B-42. [L447]
- [E] Replaces the stale “SCHEMA_VERSION=26 confirmed” text with current 46 and records progression history. [L447]
- [E] Adds at least ten per-command KEEP, STRIP, or DEFERRED records under migration_manifest.toml file.commands after reading commands.rs to avoid fabricated names. [L448]

### C-34 — Phase 4J Empire Integration (DEFERRED)

Status and external prerequisite:

- [E] Closes B-25 and phase-4J only when implemented. [L450-L451]
- [E] Is blocked on Empire source-code access from an upstream maintainer and confirmation of elai_cc.toml empire.codebase_path. [L452]
- [E] Is orthogonal to every other C plan and blocks no other delivery. [L453]
- [E] Must not be executed until the external prerequisite is met. [L454]
- [E] Estimated implementation effort after unblocking is three sessions. [L465]

Deferred feature scope:

- [E] Runs a source-grounding pass in paper-ingestor source mode and writes workflow/papers/empire-protocol-interface.md. [L455]
- [E] Adds empire configuration fields codebase_path, pipe_name, connect_timeout_ms, and brain_id. [L456]
- [E] Adds a feature-gated crates/empire workspace member. [L457]
- [E] Planned types are FactoryLineTransport, FactoryLineClient, BrainManifest, EmpireEventBridge, ElaiNeuron, EmpireIssue, and FactoryCmd. [L457]
- [E] Adds IpcMessageType::Empire as a non-destructive extension. [L458]
- [E] Adds repair handling for CircularDep, HardImport, and StdlibShadow via RepairStrategy and RepairPipeline. [L459]
- [E] Adds empire_connected, empire_disconnected, empire_capability_invoked, empire_repair_started, and empire_repair_complete, requiring a schema bump. [L460]
- [E] Exposes an ElaiNeuron MCP HTTP endpoint that registers BrainManifest. [L461]
- [E] Requires TDD tests 4J.3 through 4J.7 before production wiring step 4J.8. [L462]
- [E] Adds --empire. [L463]
- [E] Creates EmpireBridgePanel.svelte with connection status, brain_id, last capability, and repair history. [L464]

### C-35 — Tool Attention Dynamic Gating + Lazy MCP Schema Loading

Status, evidence, and dependency:

- [E] Belongs to Wave 7 paper-grounded batch A. [L592-L598]
- [E] Cites arXiv:2604.21816 for a measured 95% MCP-tax reduction. [L598-L599]
- [E] Explicitly labels downstream quality figures as projections rather than live measurements. [L599]
- [E] Depends on the Phase 4K MCP bridge being shipped. [L600]
- [E] Is parallel-safe with C-36 through C-40 except C-42 because C-42 overlaps this surface. [L601]

Tool visibility, schema loading, policy, UI, telemetry, and bypass:

- [E] Adds ToolGate. [L602]
- [E] Adds GatedCatalog. [L602]
- [E] Adds LazySchemaCache. [L602]
- [E] Defines per-FSM-state gate behavior in plugins/tool-gate.toml. [L602]
- [E] Adds tool_gated, tool_eager, tool_lazy_resolved, and tool_denied events. [L602]
- [E] Adds ToolGateInspector.svelte. [L602]
- [E] Adds --tool-gate-off as a comparison/bypass flag. [L602]
- [E] Smoke uses a catalog of at least 30 tools, expects 5,000–10,000 prompt tokens saved per feature, and requires SWE-bench performance to remain within two percentage points of the gate-off arm. [L603]

### C-36 — MUCOCO Mutation-Operator Catalog

Status, evidence, and dependency:

- [E] Cites arXiv:2604.19086 and its reported finding that 15% of inputs exposed inconsistencies. [L605-L606]
- [E] Depends on C-29 testing-trust T2/T3 being shipped. [L607]
- [E] Is parallel-safe with C-35 and C-37 through C-53. [L608]

Metamorphic expansion and measurement:

- [E] Adds nine MetamorphicRelation variants behind the mucoco_operators feature. [L609]
- [A] The nine variant names are not in the summary and must be extracted from C-36.
- [E] Adds a per-language applicability matrix for those operators. [L609]
- [E] Adds mucoco_mutation_applied, mucoco_inconsistency_detected, and mucoco_skip_reason. [L609]
- [E] Pilot acceptance requires at least one operator to match the paper’s 15% detection rate. [L610]
- [E] Re-runs the existing 3,500-plus test suite and produces a detection table. [L610]

### C-37 — PER_ROLE_DECODING_DEFAULTS Re-evaluation

Status, evidence, and dependency:

- [E] Cites arXiv:2604.06066 for “structure snowballing” caused by constrained decoding during reflection on 8B-class models. [L612-L613]
- [E] Depends on C-26 being shipped and needs RunPod for its pilot. [L614]
- [E] Is parallel-safe with C-35, C-36, and C-38 through C-53. [L615]

Evaluation and conditional design change:

- [E] Adds structure_snowballing_harness. [L616]
- [E] Adds a migration script for per-role decoding defaults. [L616]
- [E] May add a LightlyConstrained DecodingMode, making that mode conditional rather than a fixed commitment in the summary. [L616]
- [E] Adds decoding_mode_pilot_run and decoding_mode_default_flipped events. [L616]
- [E] Produces and applies a pilot.md decision matrix. [L617]
- [E] Requires the anti-rubber-stamp FINDING regression test to pass under whichever decoding mode is chosen. [L617]

### C-38 — PageRank Production Contract + Aider Spec Accuracy Fix

Status, source correction, and dependency:

- [E] Corrects a prior-oracle finding that DETAILED_SUMMARY section 10 wrongly described Aider repo maps as file-level; direct reading of aider/repomap.py showed symbol-level behavior. [L619-L620]
- [E] Depends on B-1 DKG runtime activation being shipped. [L621]
- [E] Is parallel-safe with C-35 through C-37, C-39, and C-40. [L622]

Production-path verification, fixture, documentation, and observability:

- [E] Adds pagerank_production_contract.rs. [L623]
- [E] Adds a synthetic 50-file repository with an intentional hub. [L623]
- [E] Writes aider-elai-pagerank-diff.md. [L623]
- [E] Corrects DETAILED_SUMMARY section 10. [L623]
- [E] Adds a stale-external-reference row to mistakes.md. [L623]
- [E] The later action-type summary adds dkg_rebuild_started and dkg_rebuild_complete and extends pagerank_run. [L777-L780]
- [E] Acceptance proves compute_repo_map runs on the production path, the deliberate hub appears in the top three PageRank results, and an absent DKG triggers a transparent rebuild. [L624]

### C-39 — MCP Bridge CLI Surface Extension (OpenHands Parity)

Status, source, and dependency:

- [E] Uses OpenHands MCP integration as its competitor/source anchor. [L626-L627]
- [E] Depends on C-35 so it consumes the gated tool catalog. [L628]
- [E] Is parallel-safe with C-36 through C-38 and C-40. [L629]

CLI transport abstraction, lifecycle, parity testing, and telemetry:

- [E] Adds MCPClient in cli/src/mcp_client.rs. [L630]
- [E] Adds McpClientDispatch and InProcessDispatch behind a shared Dispatch trait. [L630]
- [E] Adds --via-mcp. [L630]
- [E] Adds a CI parity job that executes every CLI test once per dispatch route. [L630]
- [E] Adds cli_mcp_dispatch, cli_mcp_server_started, and cli_mcp_server_stopped. [L630, L777-L780]
- [E] If --via-mcp finds no listener, it automatically launches a co-process. [L631]
- [E] Acceptance requires the dual-route CI parity job to be green. [L631]

### C-40 — GitHub PR Adapter (Devin Parity)

Status, source, and dependency:

- [E] Uses Devin 2.0’s published autonomous PR and review-comment loop as its competitor anchor. [L633-L634]
- [E] Depends on C-13 and C-23 being shipped. [L635]
- [E] Is parallel-safe with C-35 through C-39. [L636]

PR publication, review loop, honesty, flags, and telemetry:

- [E] Adds a feature-gated crates/github_adapter package. [L637]
- [E] Supports GhCliAdapter. [L637]
- [E] Supports RestApiAdapter. [L637]
- [E] Adds PrBodyComposer. [L637]
- [E] Requires every generated PR body to contain an honesty section. [L637]
- [E] Adds ReviewCommentWatcher and a Router. [L637]
- [E] Automatic response to review comments is off by default. [L637]
- [E] Adds --to-pr, --base, --watch-reviews, and --draft. [L637]
- [E] Adds six github_* action types, but the summary does not name them. [L637, L777-L780]
- [E] End-to-end smoke opens a draft PR in a test repository. [L638]
- [E] Its body must contain mutation score, wiring-contract result, stub status, and a reviewer-instruction line. [L638]

### C-41 — Skills-Driven Workflow Layer Formalization

Status, evidence, and dependency:

- [E] Cites arXiv:2604.21910 for an intent-accuracy increase from 44% to 83%. [L640-L641]
- [E] Depends on Phase 4A and prompt-engineer being shipped. [L642]
- [E] Is parallel-safe with C-35 through C-40 and C-42 through C-53. [L643]

Registry, retrieval, initial workflow vocabulary, UI, events, and quality floor:

- [E] Adds a skills directory and registry.toml. [L644]
- [E] Adds SkillRegistry using trigram and sparse-keyword matching. [L644]
- [E] Seeds nine skills: simple_query, focused_edit, heavy_plan, idea_intake, oracle, planning_workflow, paper_ingest, source_ingest, and benchmark_run. [L644]
- [E] Adds SkillsPanel.svelte. [L644]
- [E] Adds skill_lookup, skill_selected, and skill_fell_through. [L644]
- [E] Acceptance uses a SciQuest-style synthetic set and requires at least 73% accuracy, deliberately ten points below the cited paper’s 83%. [L645]

### C-42 — Semantic Gateway Zero-Trust MCP Fuzzing

Status, evidence, and dependency:

- [E] Cites arXiv:2604.25555 for 100% unauthorized-state discovery by a deterministic fuzzer. [L647-L648]
- [E] Depends on C-35 and C-39 being shipped. [L649]
- [E] Is parallel-safe with C-36 through C-38, C-41, and C-43 through C-53, but not with C-35 due to overlap. [L650]

Authorization-by-state, deny-by-default policy, fuzzing, UI, CI, and events:

- [E] Adds AuthzPolicy. [L651]
- [E] Adds a tool annotation expressing allowed states and actors. [L651]
- [E] Unlabeled tools default to DENY ALL. [L651]
- [E] Adds crates/mcp_fuzzer with a corpus of more than 200 tuples. [L651]
- [E] Adds FuzzReport carrying CRITICAL, P1, and shape information. [L651]
- [E] Adds AuthzPanel.svelte. [L651]
- [E] Adds a CI fuzzer job that blocks on any CRITICAL finding. [L651]
- [E] Adds mcp_authz_check, mcp_authz_reject, and fuzzer_finding. [L651, L777-L780]
- [E] Smoke first requires discovery of a known-bug fixture and then zero CRITICAL findings after the fix. [L652]

### C-43 — Phase 4H 3-Layer Scope-Freeze Ratification

Status, evidence, and dependency:

- [E] Cites arXiv:2604.21725 for anti-evidence that adding mechanisms beyond memory plus reflection consistently hurts performance. [L654-L655]
- [E] Depends on Phase 4H being shipped at its current scope. [L656]
- [E] Is parallel-safe with every other C plan. [L657]

Scope governance and ablation:

- [E] Adds phase-4h-scope-policy.md and says it quotes/cites AEL verbatim. [L658]
- [E] Adds a self_mod_experimental allow proc-macro guardrail requiring a justification string. [L658]
- [E] Adds ablation_harness.rs that measures candidate delta against baseline. [L658]
- [E] Adds a synthetic DGM-like fixture designed to expose negative delta. [L658]
- [E] Adds self_mod_layer_proposed and self_mod_ablation_run. [L658, L777-L780]
- [E] Adds a Phase 4H footnote to DETAILED_SUMMARY. [L658]
- [E] Smoke requires the harness to report a negative delta on the synthetic candidate, proving it can surface degradation. [L659]

### C-44 — Composer 2 Runtime Backend Tier

Status, evidence, and dependency:

- [E] Cites arXiv:2603.24477 and records the source’s stated SWE-bench Multilingual score of 73.7, input price of $0.50 per million tokens, output price of $2.50 per million tokens, and Kimi K2.5 base. [L661-L662]
- [E] Depends on B-37, C-39, and C-42 being shipped. [L663]
- [E] Is gated on verification of API access. [L663]
- [E] Is parallel-safe with C-35 through C-43 and C-45 through C-53. [L664]

Backend, skill-based routing, cost UI, and telemetry:

- [E] Adds Composer2Backend implementing Backend. [L665]
- [E] Routes heavy_plan to Sonnet. [L665]
- [E] Routes focused_edit to Composer 2. [L665]
- [E] Routes simple_query to a local 7B model. [L665]
- [E] Adds BackendCostPanel.svelte. [L665]
- [E] Adds backend_composer2_invoked, backend_composer2_error, and backend_composer2_cost_accrued. [L665]
- [E] Enabled-call smoke must route through Composer 2. [L666]
- [E] Every successful call must emit a cost row with non-null usd_cents. [L666]

### C-45 — CANTANTE Contrastive Credit Attribution

Status, evidence, and dependency:

- [E] Begins Wave 8’s paper-grounded batch B and cites arXiv:2605.13295. [L668-L671]
- [E] Depends on all eight AgentRoles being shipped. [L672]
- [E] Defers the real pilot to RunPod. [L672]
- [E] Is parallel-safe with C-35 through C-44 and C-46 through C-53. [L673]

Role ablation and causal credit:

- [E] Adds AgentRoleDropper with a per-role fallback table. [L674]
- [E] Dropping a role must not panic. [L674]
- [E] Adds credit_attribution_harness. [L674]
- [E] Produces cantante-pilot.md with a per-role credit table. [L674]
- [E] Adds agent_role_dropped, credit_attribution_run, and credit_attribution_result. [L674]
- [E] Synthetic-harness output must be deterministic. [L675]
- [E] Post-RunPod pilot uses a 50-task SWE-bench subset and reports per-role credit with confidence intervals. [L675]

### C-46 — Uno-Orchestra Parsimonious Routing

Status, evidence, and dependency:

- [E] Cites arXiv:2605.05007. [L677-L678]
- [E] Depends on eight AgentRoles; C-45 is recommended but not mandatory. [L679]
- [E] Is parallel-safe with C-35 through C-45 and C-47 through C-53. [L680]

Difficulty estimation, selective role activation, escalation, cost attribution, and pilot:

- [E] Adds TaskDifficultyClassifier using trigram, centrality, and reference-task cosine features. [L681]
- [E] Adds SelectiveDelegator. [L681]
- [E] Difficulty below 0.3 selects Executor plus Auditor. [L681]
- [E] Difficulty from 0.3 through below 0.7 adds CC and PE roles. [L681]
- [E] Difficulty at least 0.7 selects the full chain. [L681]
- [A] The summary does not expand the CC and PE role abbreviations.
- [E] Auditor verdict “insufficient” bubbles up to a larger delegation tier. [L681]
- [E] Adds selective_delegation_decision, selective_delegation_escalated, and selective_delegation_cost_saved. [L681]
- [E] Pilot uses 100 SWE-bench tasks, must stay within plus/minus two solve-rate points, and targets at least 30% lower cost. [L682]

### C-47 — Case-Based Calibration Tool-Use Telemetry

Status, evidence, and dependency:

- [E] Cites arXiv:2605.15041. [L684-L685]
- [E] Depends on C-35 and C-42. [L686]
- [E] Is parallel-safe with C-46 and C-48 through C-53. [L687]

Tool-case memory, retrieval, hints, and outcome measurement:

- [E] Adds a sled-backed CaseBase. [L688]
- [E] Keeps at most 100 least-recently-used cases per tool. [L688]
- [E] Adds CaseRetriever using Jaccard and Levenshtein similarity. [L688]
- [E] Injects a hint when a NEAR_MATCH indicates parameter renaming. [L688]
- [E] Adds case_base_consulted, case_base_hint_emitted, and case_base_call_logged. [L688]
- [E] A 50-session replay must show at least a 10% reduction in repeated tool errors. [L689]

### C-48 — SafeHarbor Hierarchical Memory Guardrail

Status, evidence, and dependency:

- [E] Cites arXiv:2605.05704. [L691-L692]
- [E] Depends on memory-writer and C-42. [L693]
- [E] Is parallel-safe with C-46, C-47, and C-49 through C-53. [L694]

Trust tiers, safety filtering, provenance, migration, and events:

- [E] Adds TrustTier T0 for user-direct memory. [L695]
- [E] Adds T1 for codebase-derived memory. [L695]
- [E] Adds T2 for web-sourced memory. [L695]
- [E] Adds T3 for model-emitted memory. [L695]
- [E] Adds MemoryView::T0_T1_only for safety-critical queries. [L695]
- [E] Propagates memory provenance into prompts. [L695]
- [E] Migrates every pre-C-48 memory row to T3. [L695]
- [E] Adds memory_tier_assigned, memory_query_filtered, and memory_safety_critical_blocked. [L695]
- [E] Smoke places prompt injection in a T2 row and requires it not to affect the safety-critical decision identified as 4D.X.7. [L696]

### C-49 — Iterative Distillation of Experience for Prompt-Engineer

Status, evidence, and dependency:

- [E] Cites arXiv:2605.14443, described as the most recent paper in its batch. [L698-L699]
- [E] Depends on prompt-engineer; C-41 is recommended but not mandatory. [L700]
- [E] Is parallel-safe with C-46 through C-48 and C-50 through C-53. [L701]

Experience memory, deduplication, failure integration, budget, and events:

- [E] Adds ExperienceCorpus stored per project and per skill at .elai/experience/{skill}.md. [L702]
- [E] Adds Distiller using five-gram shingling for deduplication. [L702]
- [E] Integrates avoid-patterns from mistakes.md. [L702]
- [E] Caps injected experience at about 500 tokens. [L702]
- [E] Adds experience_distilled, experience_consulted, and experience_token_budget_exceeded. [L702]
- [E] Ten sessions on one codebase must produce a non-empty Distilled section. [L703]

### C-50 — Contract-Coding Test Scaffolding

Status, evidence, and dependency:

- [E] Cites arXiv:2604.13100 for 47% functional success and near-perfect structural integrity on Greenfield-5. [L705-L706]
- [E] Depends on Phase 4F being shipped. [L707]
- [E] Is parallel-safe with C-46 through C-49 and C-51 through C-53. [L708]

Language-native contracts and completeness-pipeline integration:

- [E] Adds ContractSpec with preconditions and postconditions expressed in the target language. [L709]
- [E] ContractEmitter runs after FILE_PLAN and before skeleton generation. [L709]
- [E] Adds ContractViolationCheck to MiniCritic. [L709]
- [E] Adds ContractViolationRetry. [L709]
- [E] Enables contracts in the greenfield-init template. [L709]
- [E] Adds contract_emitted, contract_violation_detected, and contract_retry_resolved. [L709]
- [E] Greenfield-5 smoke targets at least 40% functional and at least 90% structural success, intentionally conservative against the cited 47% and near-perfect values. [L710]

### C-51 — Aider Polyglot Harness Adoption (Test Infrastructure)

Status, source, and dependency:

- [E] Vendors the MIT-licensed Aider harness at a pinned commit SHA. [L712-L713]
- [E] Depends on Phase 4F being shipped. [L714]
- [E] Is parallel-safe with C-46 through C-50, C-52, and C-53. [L715]

Benchmark corpus, language runners, CLI, refusal behavior, and telemetry:

- [E] Adds .elai_cc/external/aider_polyglot with 225 tasks. [L716]
- [E] Adds PolyglotAdapter. [L716]
- [E] Adds language-specific SandboxedTestRunner support for C++, Go, Java, JavaScript, Python, and Rust. [L716]
- [E] Adds elai benchmark polyglot. [L716]
- [E] Adds polyglot_task_started, polyglot_task_completed, and polyglot_scorecard_emitted. [L716]
- [E] Smoke dry-runs five sample tasks against a mock backend. [L717]
- [E] A full sweep must refuse to run without a GPU. [L717]

### C-52 — Dafny Formal Verification Spike (RESEARCH-GRADE)

Status, evidence, and dependency:

- [E] Is explicitly research-grade. [L719]
- [E] Cites arXiv:2604.22601 for a reported 90.91% score by Gemma 4-31B on NL2VC-60. [L719-L720]
- [E] Recommends C-48’s MemoryView as a candidate dependency and requires a Dafny toolchain. [L721]
- [E] Is parallel-safe with C-46 through C-51 and C-53. [L722]

Formal models, harness, CI, fallback artifact, and telemetry:

- [E] Adds a Dafny specification for FSM transitions. [L723]
- [E] Adds a Dafny specification for MemoryView trust filtering. [L723]
- [E] Adds a Dafny specification for the mutation aggregator. [L723]
- [E] Adds VerificationHarness. [L723]
- [E] Adds an optional CI job. [L723]
- [E] Permits an infeasibility-note path instead of successful verification, but requires a concrete blocker. [L723-L724]
- [E] The later event summary adds dafny_verify_run, dafny_verify_pass, and dafny_verify_fail. [L781]
- [E] Acceptance is either verification of all three spike modules or dafny-spike-infeasibility.md containing at least one concrete blocker. [L724]

### C-53 — NF4 Kernel Upstream Watch + In-Context Counter-Test

Status, evidence, and dependency:

- [E] Cites arXiv:2604.02556 for a passive NF4 kernel watch and reported 2.0–2.2x kernel plus 1.54x end-to-end speedups. [L726-L727]
- [E] Cites arXiv:2604.27891 as counter-evidence that in-context prompting can obsolete orchestration for procedural tasks. [L727]
- [E] Requires no prerequisite for passive watching but needs RunPod for the counter-test pilot. [L728]
- [E] Is parallel-safe with C-46 through C-52. [L729]

Upstream tracking, baseline arm, fixture, events, and gated artifact:

- [E] Adds .elai_cc/docs/upstream-watch.md and tracks bitsandbytes. [L730]
- [E] Adds cargo bench nf4_e2e. [L730]
- [E] Adds --baseline-single-prompt. [L730]
- [E] Adds a 30-task procedural-subset fixture. [L730]
- [E] Adds counter_test_run and procedural_routing_decision. [L730]
- [E] Smoke requires a pinned bitsandbytes record immediately; the counter-test pilot artifact is authored only after RunPod execution. [L731]

### C-54 — SWE-bench Verified Run, 3 Configs (FLAGSHIP)

Status, source, and dependency:

- [E] Begins Wave 9’s RunPod-dependent publication work and is labeled FLAGSHIP. [L733-L735]
- [E] Treats SWE-bench Verified as the industry standard and says ELAI has no published number under the maintainer’s honest assessment section 8. [L735-L736]
- [E] Depends on expanding C-21’s corpus from ten to 500 tasks and on shipped C-35, C-37, C-39, C-42, and optionally C-44 for the hybrid C arm. [L737]
- [E] Requires RunPod. [L737]
- [E] Is parallel-safe with C-55 through C-58 but must run sequentially when sharing one RunPod. [L738]

Locked benchmark configurations, artifacts, honesty, and events:

- [E] Adds a 500-task corpus at a pinned SHA. [L739]
- [E] Adds configuration A using local 7B. [L739]
- [E] Adds configuration B using dual local 7B. [L739]
- [E] Adds configuration C using hybrid Sonnet, Composer 2, and local. [L739]
- [E] Adds SwebenchHarness. [L739]
- [E] Produces three scorecard.json files. [L739]
- [E] Auto-generates methodology.md. [L739]
- [E] Publishes results.md with a mandatory honesty statement. [L739]
- [E] Adds swe_bench_run_started, swe_bench_task_completed, swe_bench_scorecard_emitted, and swe_bench_methodology_locked. [L739]
- [E] Acceptance requires all three configurations to finish, forbids cherry-picking, and generates methodology/results from machine-readable inputs. [L740]

### C-55 — Aider Polyglot Leaderboard Submission

Status, source, and dependency:

- [E] Anchors comparison to Aider’s public leaderboard and the recorded ollama qwen2.5-coder:32b baseline of 72.9%. [L742-L743]
- [E] Depends on C-51 for the harness and C-40 for PR submission. [L744]
- [E] Requires RunPod. [L744]
- [E] Is parallel-safe with C-54 and C-56 through C-58. [L745]

Configurations, evidence, public submission, and events:

- [E] Adds configuration L for local 32B. [L746]
- [E] Adds configuration H for hybrid. [L746]
- [E] Produces a scorecard.json for each configuration. [L746]
- [E] Produces methodology and results documents. [L746]
- [E] Opens a public draft PR against Aider-AI/aider. [L746]
- [E] Adds polyglot_submission_started, polyglot_submission_completed, and polyglot_submission_pr_opened. [L746]
- [E] Acceptance requires 225-problem scorecards per configuration. [L747]
- [E] Results must honestly compare against the 72.9% baseline and document cost per task versus GPT-5. [L747]

### C-56 — Codebase-Memory DKG 31-Repo Head-to-Head

Status, evidence, and dependency:

- [E] Cites arXiv:2603.27277 for 83% answer quality, ten times fewer tokens, and 2.1 times fewer tool calls across 31 real repositories. [L749-L750]
- [E] Depends on B-29, B-30, and C-38. [L751]
- [E] Requires RunPod. [L751]
- [E] Is parallel-safe with C-54, C-55, C-57, and C-58. [L752]

Corpus, adapters, judging, specialized probes, reporting, and events:

- [E] Vendors 31 repositories at pinned SHAs. [L753]
- [E] Adds a query corpus and ground-truth answers. [L753]
- [E] Adds CodebaseMemoryAdapter. [L753]
- [E] Adds AnswerQualityScorer using Sonnet 4.6 as judge. [L753]
- [E] Adds specialized hub-detection and caller-ranking queries. [L753]
- [E] Produces per-repository and aggregate metrics. [L753]
- [E] Adds codebase_memory_query_run, codebase_memory_answer_scored, and codebase_memory_scorecard_emitted. [L753]
- [E] The target is to meet or beat all three paper figures; failure must instead yield an improvement-target list in results.md. [L754]

### C-57 — OpenHands Index + SWE-Cycle + SWE Atlas Combo

Status, evidence, and dependency:

- [E] Uses the January 2026 OpenHands Index article, arXiv:2605.13139 SWE-Cycle, and arXiv:2605.08366 SWE Atlas. [L756-L757]
- [E] Depends on completion of C-54 so the primary number exists first. [L758]
- [E] Requires RunPod. [L758]
- [E] Is parallel-safe with C-54 through C-56 and C-58. [L759]

Multi-axis adapters, outputs, and events:

- [E] Adds adapters for openhands_index, swe_cycle, and swe_atlas. [L760]
- [E] Produces one scorecard.json per adapter. [L760]
- [E] Produces a combined methodology/results document. [L760]
- [E] Adds multi_benchmark_run_started, multi_benchmark_axis_completed, and multi_benchmark_scorecard_emitted. [L760]
- [E] Acceptance requires an OpenHands Index scorecard, a per-cycle solve-rate curve for SWE-Cycle, and per-rubric scores for SWE Atlas. [L761]

### C-58 — AgentForge 5-Role + RRK Reranker Head-to-Head

Status, evidence, and dependency:

- [E] Cites arXiv:2604.13120 for AgentForge’s reported 40.0% SWE-bench Lite result. [L763-L764]
- [E] Cites arXiv:2604.26483 for an RRK 8B reranker reported as three to 18 times faster than 0.6–4B rerankers. [L764]
- [E] Recommends C-45 but does not require it. [L765]
- [E] Requires RunPod. [L765]
- [E] Is parallel-safe with C-54 through C-57. [L766]

Vendoring, adapter, optional reranker, latency/quality gates, events, and decision:

- [E] Vendors AgentForge at a pinned SHA and includes its LICENSE. [L767]
- [E] Adds AgentForgeAdapter. [L767]
- [E] Adds an rrk_reranker feature and 8B model loader. [L767]
- [E] Adds a latency benchmark with a p99 below 100 ms gate. [L767]
- [E] Adds a quality benchmark against ELAI’s at-most-200M cross-encoder. [L767]
- [E] Adds head_to_head_run_started, head_to_head_arm_completed, reranker_eval_run, and reranker_eval_decision. [L767]
- [E] Acceptance produces a side-by-side scorecard against the 40.0% baseline. [L768]
- [E] RRK adoption is configuration-gated and leaves the default unchanged until the decision favors it. [L768]

### C-59 — Cross-Config Benchmark Master Report (CAPSTONE)

Status, source, and dependency:

- [E] Is the capstone aggregator for C-54 through C-58 and external baselines from the 2026-05-17 oracle pass. [L770-L771]
- [E] Optionally includes a TTS pilot based on arXiv:2605.09329 and its reported 41% average acceptance-length improvement. [L771]
- [E] Requires C-54, C-55, C-56, C-57, and C-58 to be shipped with completed runs. [L772]
- [E] Is not parallel-safe because it consumes all benchmark outputs. [L773]

Aggregation, reproducibility, comparison, honesty, and events:

- [E] Adds MasterReportAggregator. [L774]
- [E] Adds external_baselines.toml. [L774]
- [E] Pins Devin, OpenHands, Aider, Claude Code, GPT-5.4, Composer 2, AgentForge, and Codebase-Memory by version, date, and URL. [L774]
- [A] This is eight named baseline families despite the earlier narrative claiming seven competitor anchors. [L589, L774]
- [E] Regenerates benchmark_master_report.md from machine-readable inputs. [L774]
- [E] The report contains a cross-configuration matrix, cross-competitor table, cost-per-task analysis, mandatory honesty section, and improvement-target list. [L774]
- [E] The report may include an optional TTS pilot section. [L774]
- [E] Adds master_report_aggregated, tts_pilot_run, and tts_pilot_decision. [L774]
- [E] Acceptance requires deterministic report compilation and presence of honesty and improvement-target sections. [L775]
- [E] The intended claim boundary is the defensible form “ELAI is competitive on X measurement at Y cost.” [L775]

## Explicit cross-plan closure maps

### B-plan closure

The summary explicitly maps the following B items to C work. These are traceability links, not an independent assertion that either side is implemented. [L467-L511]

| B item | C closure |
|---|---|
| B-1 | C-18 contract test; C-8 PageRank/AragOrchestrator |
| B-2 | C-18 FSM phase-transition contract |
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
| B-25 | C-34, deferred |
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

- [A] The table omits B-29.5, B-31.5, B-32, and B-33 even though C-17, C-18, and C-20 explicitly say they close those items. [L210-L211, L226-L240, L256-L267, L467-L511]

### Guide closure

- [E] compiled-pipeline-design is closed by C-25. [L513-L517]
- [E] token-efficiency-design is closed by C-26. [L518]
- [E] pipeline-modes-design is closed by C-30. [L519]
- [E] code-completeness-enforcement-design is closed jointly by C-31 and C-32. [L520]
- [E] testing-trust-design is partitioned into C-27 for T(-1)/T-0, C-28 for T-1, and C-29 for T-2/T-3. [L521]
- [E] anti-staleness-enforcement is closed by C-21. [L522]
- [E] dashboard-design is closed by C-6 and C-7. [L523]
- [E] update-phase-map is closed by C-33. [L524]
- [A] Session text also references guide/post-phase4-implementation-order and guide/update-implementation-order, neither of which appears in this closure table. [L328-L329, L340-L341, L396-L397, L410-L411]

### Phase closure

- [E] phase-0 maps to C-14 and C-15. [L526-L530]
- [E] phase-3B, phase-3D, and phase-3H map to C-18. [L531-L533]
- [E] phase-3J receives the C-33 addendum. [L534]
- [E] phase-3K maps to C-17. [L535]
- [E] phase-3L is created as a spec by C-33. [L536]
- [E] phase-4A maps to C-11 and C-19. [L537]
- [E] phase-4B maps to C-20. [L538]
- [E] phase-4C maps to C-19 and C-20. [L539]
- [E] phase-4D maps to C-30 and C-19. [L540]
- [E] phase-4D-SURGICAL and phase-4D-ORACLE are specified by C-33 and implemented by C-30. [L541-L542]
- [E] phase-4E maps to C-31. [L543]
- [E] phase-4F maps to C-22, C-19, and C-20. [L544]
- [E] phase-4G maps to C-9, C-10, C-20, and the C-33 addendum. [L545]
- [E] phase-4H maps to C-21. [L546]
- [E] phase-4J maps only to deferred C-34. [L547]
- [E] phase-4K maps to C-4, C-7, and C-8. [L548]
- [E] phase-4M maps to C-4 and C-20. [L549]

## How the capabilities work together

The following interaction statements distinguish direct wiring from conservative composition.

### SSB-to-dashboard observability loop

- [E] C-1 activates knowledge and reasoning producers; C-2 activates orchestration/tree/Cosmos/ambiguity/anti-spiral producers; C-8 removes remaining producer stubs. [L28-L48, L102-L114]
- [E] C-3 and C-4 turn those rows into live Tauri knowledge, reasoning, orchestration, project, runtime, and spend commands. [L50-L67]
- [E] C-6 and C-7 provide the aligned visual system, streaming controls, window recovery, review controls, responsive layouts, and UI tests. [L79-L100]
- [E] Domain-specific consumers are added in later plans: stagnation in C-10, audit timeline in C-12, Cosmos in C-16, indexing banners in C-17, feasibility in C-22, branches in C-23, graph impact in C-24, compiled/background tasks in C-25, mutation/intent trust in C-28/C-29, and code completeness in C-32.
- [I] Together these form a producer → append-only event row → SsbViewer/Tauri query → drawer/window/control loop. The exact shared event-delivery mechanics must be verified from the originals because the summary sometimes mixes polling and subscriptions.

### Cloud, autonomy, and anti-spiral control loop

- [E] C-5 supplies local/hybrid/cloud selection, spend ceilings, warnings, one-shot hybrid escalation, cost estimation, route blocking, and a spend UI. [L69-L77]
- [E] C-9 supplies evidence-based auto-advance, escalation policy, task splitting, model substitution failure, model-fit checks, and per-role backends. [L116-L124]
- [E] C-10 adds repeated-attempt, oscillation, diminishing-return, drift, persona, and watchdog defenses at autonomous decision and step boundaries. [L126-L135]
- [E] C-25 restricts background gated_spawn to autonomous mode and supplies timeout/detach behavior. [L332-L338]
- [E] C-26 adds provider-level constraint fallback plus structural futility/persona switching. [L340-L351]
- [I] In the composed loop, cost or evidence can prevent/redirect advancement, while repeated failure can switch personas, split work, substitute a model, detach a background operation, or escalate. Ordering among these fallbacks is not fully specified in the summary and needs original-plan inspection.

### Isolation, tool dispatch, and hook observability

- [E] C-11 completes Tier-C process isolation, platform checks, resource limits, and IPC hook wiring. [L137-L146]
- [E] C-13 replaces production in-process dispatch with IPC and wraps tools with DAIRA marker emission, while retaining in-process operation only for tests. [L160-L170]
- [E] C-19 adds adversarial IPC, sandbox, research, and feasibility cases plus latency gates. [L243-L254]
- [E] C-35 later narrows visible tools and defers schemas; C-39 exposes the same dispatch surface through MCP; C-42 applies state/actor authorization and fuzzes it fail-closed. [L598-L652]
- [I] The intended safety chain is sandboxed process → IPC registry/hook dispatcher → adaptive tool evidence → gated/lazy tool catalog → optional MCP transport → authorization check. Exact denial ordering and which layer owns the final verdict are not in the summary.

### Determinism and testing-trust ladder

- [E] C-14 removes legacy/dead nondeterministic surfaces; C-15 centralizes time and map ordering and restores entropy CI. [L172-L195]
- [E] C-27 adds environment-pinned determinism comparison plus static/dynamic wiring contracts. [L353-L363]
- [E] C-28 runs mutation only after the existing five deterministic checks pass and blocks on an inadequate score. [L365-L378]
- [E] C-29 requires user/autonomous confirmation of natural-language-to-test pairs before testing and adds metamorphic checks after mutation. [L380-L394]
- [E] C-36 expands metamorphic operators; C-50 adds target-language contracts before code skeleton generation; C-52 explores formal verification of selected critical logic. [L605-L610, L705-L724]
- [I] This creates a progression from reproducible execution, to proof that production entry points are wired, to tests that kill mutations, to intent fidelity and relation preservation, to optional language-level contracts/formal proof.

### Knowledge, Cosmos, impact, surgical, and Oracle loop

- [E] C-8 activates real ONNX scoring, live latency, artifact indexing, Cosmos, and ambiguity. [L102-L114]
- [E] C-16 makes Cosmos an actual tiered query stage and gives its refresh lifecycle locks, cancellation, events, UI subscription, and manual trigger. [L197-L208]
- [E] C-17 makes indexing resumable, watchable, multi-language, non-code aware, and entry-point aware. [L210-L224]
- [E] C-24 exposes symbol impact, flow, context, list, incremental graph updates, deterministic Mermaid, decomposition graph views, and file-write blast-radius actions. [L314-L326]
- [E] C-30 uses that DKG reachability for surgical plans, offers additional DKG tools, and caches research answers by query/project/source tier. [L396-L408]
- [E] C-38 proves PageRank is on the production path and self-rebuilds missing DKG state. [L619-L624]
- [I] These pieces make live indexes and graph reachability feed both retrieval context and change planning; Cosmos augments deeper retrieval, while Oracle avoids repeating stable answers and surgical mode sizes tightly bounded changes.

### Compiled pipeline, decoding, and code-completeness loop

- [E] C-25 crystallizes three consecutive successful shapes, tries templates first, decompiles weak templates, and tracks attention budget. [L328-L338]
- [E] C-26 maps each role to a decoding mode, retries invalid structure on a fixed temperature schedule, falls back to free text when unsupported, and stops futile repetition. [L340-L351]
- [E] C-31 requires a bounded FilePlan, tracks written lines with a generation guard, and records lazy/empty/oversized failures. [L410-L421]
- [E] C-32 generates signatures first, validates each in-memory fill with MiniCritic, retries structurally without instructional retry text, and writes exactly once after blanks are resolved. [L423-L435]
- [E] C-50 inserts ContractEmitter after FILE_PLAN and before the skeleton and feeds violations back through MiniCritic retry. [L705-L710]
- [I] The composed generation path is cached/compiled plan when valid → constrained role output → per-file plan/budgets → optional language contracts → skeleton → in-memory fill/MiniCritic → structural retry → one disk flush → deterministic audit/mutation/metamorphic gates.

### Self-modification and anti-staleness loop

- [E] C-21 links deprecated-pattern promotion to steering-vector refresh, dependency divergence to a rate-limited audit, audit output to stale catches, blocking events to MistakeBook, rules.db to ConstraintsBlock, and mutable parameters to a closed enum. [L269-L283]
- [E] C-43 freezes Phase 4H at three layers unless an explicitly justified experimental layer wins an ablation against baseline. [L654-L659]
- [E] C-45 measures the contribution of roles by dropping them; C-49 distills per-skill experience while deduplicating and importing avoid-patterns. [L670-L703]
- [I] The combined intent is measured self-improvement within a closed mutation surface: detect stale/failing patterns, retain evidence, refresh derived steering data, and reject additional machinery that loses its ablation.

### Skills, routing, memory, and provider economics

- [E] C-41 selects among nine formal skills; C-44 maps representative skills to local, Composer 2, or Sonnet backends; C-46 reduces active roles by measured difficulty and bubbles up when the auditor says insufficient. [L640-L682]
- [E] C-47 learns per-tool cases and injects near-match hints; C-48 filters memory by provenance for safety-critical work; C-49 injects bounded distilled experience per skill. [L684-L703]
- [I] The composed front end first identifies a workflow skill, then selects a role chain and backend, while tool-use cases and trusted experience/memory shape execution. The summary does not define conflict precedence among skill selection, explicit user model choice, backend availability, and selective delegation.

### Benchmark-publication pipeline

- [E] C-51 establishes the six-language Aider task runner. [L712-L717]
- [E] C-53 adds the single-prompt counterfactual arm so orchestration must beat a simpler alternative on procedural work. [L726-L731]
- [E] C-54 runs three locked SWE-bench configurations without cherry-picking. [L733-L740]
- [E] C-55 submits two Aider Polyglot configurations; C-56 compares DKG memory on 31 repos; C-57 adds three further benchmark axes; C-58 compares both multi-role orchestration and rerankers. [L742-L768]
- [E] C-59 consumes only completed outputs and deterministically generates the cross-config/cross-competitor/cost/honesty report. [L770-L775]
- [I] The publication chain intentionally separates harness creation, locked execution, machine-readable scorecards, methodology generation, and final claims. Failure is represented by improvement targets rather than selective omission.

## Cross-cutting invariants and lifecycle rules

The following are explicit hard rules in the summary’s own consolidated section. [L551-L576]

### Serialization and event registration

- [E] The summary calls 46 the current shipped schema at the B-37/38/40/41/42 baseline. [L551-L553]
- [E] C-1 increments the schema exactly once for all seven of its new variants. [L554]
- [E] C-27 says schema 46 becomes 47 for WiringEvidence, WiringViolation, and DeterminismDrift. [L555]
- [E] C-28 keeps 47 and treats MutationSurvived as sharing the wave bump. [L556]
- [E] C-29 keeps 47 and treats IntentConfirmed and MrViolated as sharing the wave bump. [L557]
- [E] C-34 requires a future bump for five Empire variants. [L558]
- [E] Waves 7–9 add approximately 65 event types and distribute schema bumps across 25 patches. [L777-L785]
- [A] C-1’s exact +1 relative to baseline 46 conflicts chronologically with C-27 also claiming 46→47. C-1, C-27, C-28, C-29, and C-0_INDEX must be read together to recover intended migration order.
- [A] The Wave 7 count is “about 28”, Wave 8 “about 21”, Wave 9 “about 16”, and total “about 65”; several section-level names use a shortened spelling compared with the detailed plan bullets. Exact enum names and lockstep registration sites require originals.

### Production-code prohibitions

- [E] Production paths may not use unwrap or expect, except test configurations and the main entry point. [L562]
- [E] Production may not use InProcessToolDispatcher after C-13. [L563]
- [E] Production producer paths may not use NoopSsbNotifier after C-1/C-8. [L564]
- [E] Tauri commands may not return hardcoded zero or mock data after C-3/C-4. [L565]
- [E] feasibility_probe may not use kill_on_drop(true) after C-22; explicit child.kill().await is required. [L566]
- [E] Every new constant must be named; magic numbers are disallowed under CLAUDE.md. [L567]
- [E] Cache keys and fingerprints use blake3, never sha256 or md5. [L568]
- [E] Production maps use FxHashMap rather than HashMap. [L569]
- [E] Production time uses ClockProvider rather than bare SystemTime::now or Instant::now. [L570]
- [E] Unavoidable SystemTime use requires entropy-allow annotation. [L571]

### Safety and mutable-state constraints

- [E] The LOCKED_KEYS pre-commit policy denies commits touching owning files unless an explicit override exists. [L572]
- [E] Mutable self-modification parameters are restricted to the 12-form ParameterKey enum. [L573]
- [E] Tier-C sandbox changes require human review. [L140]
- [E] C-21’s unwrap cleanup requires an explicit user Y. [L274]
- [E] C-34 cannot begin until Empire access and path configuration are verified. [L452-L454]
- [E] C-42 tools without authorization labels are denied by default. [L651]
- [E] C-48 permits only T0/T1 memory in safety-critical views. [L695]

### State-transition gates

- [E] AcceptAndComplete is allowed only at completeness ratio at least 0.5. [L574]
- [E] AUDITING to MEMORIZING is blocked when mutation score is below threshold. [L575]
- [E] The same transition incorporates the metamorphic-relation aggregate. [L575]
- [E] The same transition is subject to wiring audit. [L575]
- [E] PLANNING to TESTING_FIRST is blocked when fewer than three intent pairs exist. [L576]
- [E] Surgical interactive work above 500 LOC requires confirmation; autonomous work above 1,000 LOC escalates. [L400]
- [E] A compiled pipeline decompiles at two failures or confidence below 0.7. [L332]
- [E] C-42 CI blocks on any CRITICAL MCP-fuzzer finding. [L651]
- [E] C-51 refuses a full benchmark sweep without a GPU. [L717]

### Deterministic ordering and persistence

- [E] Seeded repeat runs must preserve action-type order except timestamps. [L195]
- [E] Session parent chains are rendered root-to-tip. [L308]
- [E] RegressionGate hashes sorted must_pass entries with blake3. [L400]
- [E] Bridge files are sorted lexically and generated without an LLM. [L430]
- [E] Skeleton generation accumulates fills off disk, then performs exactly one write. [L427]
- [E] Compiled pipelines persist as MessagePack and expire after seven days. [L332]
- [E] Equivalent mutants persist in SQLite and expire after 30 days. [L372]
- [E] Oracle cache persistence duration depends on source tier, with Tier 4 never cached. [L403]
- [E] Cosmos refresh obtains the index lock and supports cancellation. [L203]

## Status, dependencies, blockers, and deferred choices

### Plan-wide status classes

- [E] Original generation status: C-1 through C-33 PLANNED. [L6]
- [E] External-block status: C-34 DEFERRED and non-blocking for all other C delivery. [L450-L465]
- [E] Expanded coverage labels C-35 through C-53 in-scope on CPU/cloud, not necessarily implemented. [L580-L583]
- [E] Expanded coverage labels C-54 through C-59 in-scope but RunPod-dependent. [L582]
- [A] Later plan prerequisites frequently call earlier dependencies SHIPPED, but the summary’s status legend calls C-1 through C-33 PLANNED. Actual implementation status cannot be established from this summary alone.

### Conditional or externally gated items

- [E] C-11 is safety-critical and calls for user review. [L140]
- [E] C-21 deterministic-auditor/DKG unwrap removal occurs only after explicit user confirmation. [L274]
- [E] C-33’s new specifications are local-only because docs/specs is gitignored. [L441-L443]
- [E] C-34 waits on upstream Empire code access and local codebase_path confirmation. [L452-L454]
- [E] C-35’s quality numbers remain projections pending the specified comparison. [L599-L603]
- [E] C-37 needs RunPod for its pilot, and LightlyConstrained is only a possible outcome. [L614-L617]
- [E] C-44 waits for Composer 2 API-access verification. [L663]
- [E] C-45’s real 50-task pilot is deferred to RunPod. [L672-L675]
- [E] C-52 may end in a research infeasibility artifact rather than verified Dafny modules. [L723-L724]
- [E] C-53’s counter-test artifact waits for RunPod. [L728-L731]
- [E] C-54 through C-59 need completed paid/accelerated benchmark runs and, for C-59, all five predecessor result sets. [L733-L775]
- [E] C-59’s TTS pilot is optional. [L771-L775]

## Deeper-source manifest

Every original C plan present beside the summary is listed below. Later discovery agents should read each file completely; the reasons column highlights facts this condensed summary cannot resolve.

| Plan | Original file | Deep-read reason |
|---|---|---|
| C-0 | docs/thefinalglance/C-plan/C-0_INDEX.md | Canonical ordering, status, schema baseline, closure ledger, and post-append reconciliation |
| C-1 | docs/thefinalglance/C-plan/C-1_ssb_emission_knowledge_reasoning.md | Exact payload schemas, seven-event registration sites, and schema starting value |
| C-2 | docs/thefinalglance/C-plan/C-2_ssb_emission_orchestration.md | Resolve “12 variants” versus ten named variants; exact Cosmos event reuse |
| C-3 | docs/thefinalglance/C-plan/C-3_tauri_commands_knowledge_surface.md | Resolve “seven stubs” versus six named stubs; response schemas |
| C-4 | docs/thefinalglance/C-plan/C-4_tauri_commands_orchestration_cloud.md | Command arguments/results, AppState ownership, live spend semantics |
| C-5 | docs/thefinalglance/C-plan/C-5_cloud_mode_hardening.md | LOC constant meanings, warning/ceiling precedence, hybrid retry flow, redaction feed |
| C-6 | docs/thefinalglance/C-plan/C-6_dashboard_design_tokens.md | Full token palette, exact affected selectors, icon disposition |
| C-7 | docs/thefinalglance/C-plan/C-7_dashboard_ux_completeness.md | Recovery-stack lifecycle, shortcut scoping, hunk retry semantics, snapshots |
| C-8 | docs/thefinalglance/C-plan/C-8_viewer_stub_eradication.md | Exact producer list, ambiguity gate contract, ONNX scoring and latency sites |
| C-9 | docs/thefinalglance/C-plan/C-9_autonomous_infrastructure.md | EscalationPolicy fields, evidence verdict logic, split/merge semantics, measurements |
| C-10 | docs/thefinalglance/C-plan/C-10_anti_spiral_primitives.md | Unnamed stagnation/persona variants, threshold behavior, fallback ordering |
| C-11 | docs/thefinalglance/C-plan/C-11_sandbox_completeness.md | Platform guards, precise syscall/mount rules, adversarial cases, review gate |
| C-12 | docs/thefinalglance/C-plan/C-12_per_level_auditor_navigator_wiring.md | Six audit modes, cascade/respawn limits, verdict payload and UI schema |
| C-13 | docs/thefinalglance/C-plan/C-13_ipc_dispatcher_production_swap.md | Ten missing versus 15 total DAIRA markers, hook order, WIP-squash safety |
| C-14 | docs/thefinalglance/C-plan/C-14_phase0_debt_dead_code_strip.md | Caller-sweep proof, complete migration manifest, exact removed LOC |
| C-15 | docs/thefinalglance/C-plan/C-15_phase0_debt_clock_entropy.md | SeededClock instant semantics, all entropy exceptions, map migration scope |
| C-16 | docs/thefinalglance/C-plan/C-16_cosmos_pipeline_integration.md | Stage input/output, decay rule, lock contention, cancellation, UI event handling |
| C-17 | docs/thefinalglance/C-plan/C-17_knowledge_indexing_resilience.md | Kotlin fallback fidelity, resume checkpoints, watcher lifecycle, scale results |
| C-18 | docs/thefinalglance/C-plan/C-18_test_infra_backfill_contracts.md | Exact file/test count and assertions for all backfilled contracts |
| C-19 | docs/thefinalglance/C-plan/C-19_test_infra_backfill_adversarial.md | Full 15-case IPC matrix, benchmark methodology, all feasibility cases |
| C-20 | docs/thefinalglance/C-plan/C-20_test_infra_backfill_metrics.md | MeasureScope CLI grammar, every metric field/unit, unmeasured-reason rules |
| C-21 | docs/thefinalglance/C-plan/C-21_self_mod_anti_stale_automation.md | Closed enum shapes, override procedure, sentinel lifecycle, corpus expansion path |
| C-22 | docs/thefinalglance/C-plan/C-22_feasibility_probe_completeness.md | StubResult compatibility, kill/error paths, alternative search, UI field definitions |
| C-23 | docs/thefinalglance/C-plan/C-23_sessiondag_branch_ui.md | Branch conflict/merge semantics, checkpoint event naming, handoff schema |
| C-24 | docs/thefinalglance/C-plan/C-24_symbol_impact_dashboard_surface.md | Tool schemas, incremental threshold behavior, graph modes, blast-radius interaction |
| C-25 | docs/thefinalglance/C-plan/C-25_compiled_pipeline_template_crystallizer.md | Template matching/confidence, store invalidation, task detach/abort state machine |
| C-26 | docs/thefinalglance/C-plan/C-26_constrained_decoding_retry_strategy.md | Provider request schemas, strict-mode semantics, error equivalence, grammar contents |
| C-27 | docs/thefinalglance/C-plan/C-27_testing_trust_t0_wiring_contract.md | Schema ordering conflict, WIRING_CONTRACT grammar, observables, hardware pin model |
| C-28 | docs/thefinalglance/C-plan/C-28_testing_trust_t1_mutation.md | Runner output parsers, equivalent-mutant cache, timeout scoring, gauge command completion |
| C-29 | docs/thefinalglance/C-plan/C-29_testing_trust_t2_t3_intent_metamorphic.md | Confirmation persistence, sample generation, MR aggregate scoring, schema sharing |
| C-30 | docs/thefinalglance/C-plan/C-30_surgical_oracle_mode.md | Complexity boundary precedence, “at least five” versus six VRAM labels, cache persistence |
| C-31 | docs/thefinalglance/C-plan/C-31_code_completeness_file_plan.md | Language limits, remaining adaptive formulas, generation reset/TOCTOU semantics |
| C-32 | docs/thefinalglance/C-plan/C-32_code_completeness_skeleton_first.md | Blank edge cases, candidate scoring tie-breaks, AcceptAndComplete materialization |
| C-33 | docs/thefinalglance/C-plan/C-33_docs_phase_map_updates.md | Complete local-only specs/addenda, exact failure mappings, gitignored-delivery policy |
| C-34 | docs/thefinalglance/C-plan/C-34_empire_integration_deferred.md | Full deferred protocol, upstream interface assumptions, unblock conditions |
| C-35 | docs/thefinalglance/C-plan/C-35_tool_attention_dynamic_gating.md | Gate-state matrix, schema cache lifecycle, measured versus projected metrics |
| C-36 | docs/thefinalglance/C-plan/C-36_mucoco_mutation_operator_catalog.md | Names/semantics of all nine relations and language applicability |
| C-37 | docs/thefinalglance/C-plan/C-37_per_role_decoding_defaults_reeval.md | Pilot arms, decision thresholds, whether LightlyConstrained is actually selected |
| C-38 | docs/thefinalglance/C-plan/C-38_pagerank_production_contract_aider_spec_fix.md | Rebuild telemetry, fixture topology, exact Aider correction and evidence |
| C-39 | docs/thefinalglance/C-plan/C-39_mcp_bridge_cli_surface.md | Dispatch parity, co-process lifecycle, listener discovery, shutdown/error behavior |
| C-40 | docs/thefinalglance/C-plan/C-40_github_pr_adapter.md | Names/payloads of six GitHub events, auth, comment routing, honesty template |
| C-41 | docs/thefinalglance/C-plan/C-41_skills_driven_workflow_formalization.md | Skill file contracts, scoring/tie-breaks, fallthrough, accuracy fixture |
| C-42 | docs/thefinalglance/C-plan/C-42_semantic_gateway_zero_trust_fuzzing.md | Annotation grammar, state/actor ontology, tuple corpus, severity policy |
| C-43 | docs/thefinalglance/C-plan/C-43_phase_4h_scope_freeze_ratification.md | Three-layer definition, proc-macro enforcement scope, ablation adoption rule |
| C-44 | docs/thefinalglance/C-plan/C-44_composer2_runtime_backend_tier.md | API availability/auth, backend protocol, prices/version pin, routing fallbacks |
| C-45 | docs/thefinalglance/C-plan/C-45_cantante_contrastive_credit_attribution.md | Eight roles, fallback table, confidence calculation, RunPod pilot status |
| C-46 | docs/thefinalglance/C-plan/C-46_uno_orchestra_parsimonious_routing.md | CC/PE expansion, feature normalization, exact boundary handling, escalation |
| C-47 | docs/thefinalglance/C-plan/C-47_case_based_calibration_tool_use.md | Case schema, LRU persistence, similarity thresholds, hint safety |
| C-48 | docs/thefinalglance/C-plan/C-48_safeharbor_memory_guardrail.md | Safety-critical query classifier, provenance inheritance, T3 migration |
| C-49 | docs/thefinalglance/C-plan/C-49_iterative_distillation_prompt_policy.md | Corpus update/dedup algorithm, token truncation, mistakes integration |
| C-50 | docs/thefinalglance/C-plan/C-50_contract_coding_test_scaffolding.md | Supported language contracts, emitter templates, violation retry and scoring |
| C-51 | docs/thefinalglance/C-plan/C-51_aider_polyglot_harness_adoption.md | Vendored SHA/license, sandbox commands, GPU refusal criteria, scorecards |
| C-52 | docs/thefinalglance/C-plan/C-52_dafny_formal_verification_spike.md | Dafny specs, proof obligations, CI gating, acceptable infeasibility evidence |
| C-53 | docs/thefinalglance/C-plan/C-53_nf4_kernel_watch_in_context_counter_test.md | Upstream-watch cadence, benchmark baselines, procedural fixture, routing decision |
| C-54 | docs/thefinalglance/C-plan/C-54_swe_bench_verified_run_three_configs.md | Corpus expansion, locked configs, anti-cherry-pick mechanism, spend/methodology |
| C-55 | docs/thefinalglance/C-plan/C-55_aider_polyglot_leaderboard_submission.md | Submission protocol, scorecard schema, public PR authorization and honesty text |
| C-56 | docs/thefinalglance/C-plan/C-56_codebase_memory_dkg_31_repo_eval.md | Repo/query pins, ground truth, Sonnet-judge rubric, failure report path |
| C-57 | docs/thefinalglance/C-plan/C-57_openhands_index_swe_cycle_swe_atlas.md | Adapter protocols, curve/rubric computation, combined methodology |
| C-58 | docs/thefinalglance/C-plan/C-58_agentforge_rrk_reranker_head_to_head.md | Vendored license, comparison arms, p99 method, adoption threshold |
| C-59 | docs/thefinalglance/C-plan/C-59_cross_config_benchmark_master_report.md | Input schema, deterministic regeneration, exact external pins, optional TTS decision |

### Referenced sources not contained or not fully identified by the summary

- [A] The Empire codebase and maintainer-provided protocol are unavailable until the C-34 external prerequisite is met. [L452-L465]
- [A] The forum-dump source is cited as F:\.ELAI_workflow\.extras\discord-catchup-bot\forum-dump.md, a Windows path whose availability in this workspace is not established by the summary. [L594]
- [A] Two named oracle caches, oracle-elai-competitive-papers-2026-05-17 and oracle-elai-forum-dump-papers-2026-05-17, are cited but no filesystem paths are supplied. [L787-L789]
- [A] The narrative says 19 verified papers, but the citation table enumerates 22 IDs; five excluded papers are described only by category/ordinal, and their IDs are not listed. [L787-L814]
- [A] C-38 cites direct reading of aider/repomap.py, but the exact repository SHA/path belongs in the original C-38 evidence.
- [A] C-40’s Devin 2.0 anchor, C-39’s OpenHands anchor, and C-54/C-55/C-59 competitor baselines require date/version/URL pins from their originals; the summary alone cannot guarantee the stated external numbers remain comparable.
- [A] C-54 requires a C-21 corpus expansion from ten to 500 tasks, while C-21 itself only specifies ten seed tasks; the expansion owner and integrity checks must be recovered from C-54/C-21. [L281, L737-L740]
- [A] The summary uses source shorthand such as phase-index-3A-4J, guide/dashboard-design, guide/post-phase4-implementation-order, and guide/update-implementation-order without exact file paths. Their precise source documents should be taken from the corresponding original C plans.

## Final completeness note

- [E] All explicit C-1 through C-59 objectives, prerequisites, named types/files, commands, controls, state rules, fallbacks, event families, dashboards, testing expectations, benchmark targets, and deferred conditions present in the 814-line summary are represented above.
- [I] The dominant completed-design composition is: deterministic/isolated execution emits typed SSB evidence; live dashboard and CLI/MCP/PR surfaces consume it; knowledge and graph systems size/contextualize work; compiled/constrained/skeleton-first generation produces bounded code; wiring, mutation, intent, metamorphic, and contract gates decide transition eligibility; measured routing and memory lower cost without bypassing safety; locked public benchmarks constrain external claims.
- [A] Exact implementation status, event payload fields, unspecified enum members, schema migration chronology, and external benchmark/citation pins remain delegated to the original-plan deep reads listed in the manifest.
