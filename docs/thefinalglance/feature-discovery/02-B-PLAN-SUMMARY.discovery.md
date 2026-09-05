# B-plan exhaustive feature discovery

## Coverage

- **Primary source read completely:** `docs/thefinalglance/B-plan/B-PLAN-SUMMARY.md`, lines 1-1111. The summary identifies itself as a topological map synthesized from `B-0_INDEX + B-1..B-42 (45 files)` and says the individual files remain canonical (lines 1-7).
- **Context source read completely:** `docs/thefinalglance/ELAI-SPEC-OVERVIEW.md`, lines 1-592. It was used only to understand the global harness thesis and B's place in it. The B synopsis says B makes the core run end-to-end and visibly and adds dashboard, cloud/focus/runtime, knowledge, REFLECT/framing/auditing/Cosmos capabilities (overview lines 140-156).
- **Extraction policy:** `[E]` means explicitly stated by the B summary. `[I]` means a conservative interaction inferred by composing explicit statements; no inferred item is treated as an independent promised feature. Exact identifiers, thresholds, paths, event names, statuses, and unresolved alternatives are retained where stated.
- **Status semantics:** `DONE` means shipped per the B-0 session log; `PARTIAL` means shipped with explicit deferred items; `PLANNED` means a spec exists but was not executed; `DEFERRED` means post-MVP or conditional (B lines 23-27). The authoritative snapshot in this discovery is the summary's **2026-05-17** snapshot (lines 1086-1109), even where a section title uses older or more nuanced language.
- **Scope limitation:** original B-N specifications were inventoried as deeper-source candidates but not read. Therefore, this is exhaustive for the summary, not a claim that every detail in every canonical B-N file appears here.

## Atomic feature inventory

### B-wide objective, dashboard rebaseline, and delivery ledger

- [E] Part I's end-to-end gate requires a live FSM badge, real engine telemetry per query, real GRC budgets per agent, DAG visualization, branch UI, and zero dead UI elements (lines 95-99).
- [E] Dashboard specs B-1..B-35 were originally written against V1 at `crates/dashboard/`, now parked at `crates/.bkp_old/dashboard/`; V2 is active at `crates/dashboard-v2/` and uses Tauri 2.10, Svelte 5 runes, Vite 5, and a three.js orbit (lines 31-35).
- [E] V1-to-V2 surface mapping is: `commands.rs` -> V2 `src-tauri/src/lib.rs`; `src-frontend/src/` -> `src/lib/`; `ChatWindow.svelte` -> `windows/CommandWindow.svelte`; textual `ActivityFeed.svelte` -> `windows/ActivityWindow.svelte`, whose orbit has PLAN/MAP/RESEARCH modes; `Settings.svelte` -> `windows/SettingsWindow.svelte`; ten drawers -> `{Knowledge,Reasoning,Orchestration,Rules,Research,Providers,ProjectIntelligence,AntiStaleness,ToolsHooks,WebQuality}Drawer.svelte`; `elai-dashboard` -> `elai-dashboard-v2`, with the CLI path fix assigned to B-36 (lines 37-47).
- [E] V2 baseline invariants: only `greet` and stub `get_grounding_cache_stats` were wired before B-36; roughly 60+ frontend commands were stubbed through `src/lib/api/stub.ts` and `mock-data.ts`; only Svelte 5 `$state/$derived/$effect/$bindable` runes are allowed, with no `$:` labels or `createEventDispatcher`; app mounting uses `mount(App, { target })`; engine crates were intentionally absent from `src-tauri/Cargo.toml`, and adding one requires user confirmation (lines 49-56).
- [E] Session `s-59964` shipped B-1 through B-10, re-scoped B-13, B-14, B-24, B-26 follow-up completion, and B-36 follow-up completion; it also added auto-remote-sidecar and a dashboard remote-overlay UI and bumped schema 25 -> 26 (lines 60-65).
- [E] Session `s-26716` shipped B-22 autonomous mode plus patch-4G.1's live CommandPalette gate toggle and promoted StatusBar gate badge (lines 66-67). Session `s-30584` shipped B-39 observability v0 and schema 28 -> 29; `s-34648` shipped B-39C answer substitution; an ad-hoc oracle-gate.1 patch added a RESEARCH intent chip with Enter accept, Esc reject, and Tab edit (lines 68-73).
- [E] Memory/status records say B-37 shipped 11 backends, runtime abstractions, four SSB action types, `/runtime`, RuntimePanel, and a proven live Sonnet 4.6 cloud run; B-40 shipped framing/presets/critic with 79/79 tests; B-41 shipped per-level audit types/surfaces with 80 green tests but deferred navigator spawning; B-42 shipped Cosmos worker/compiler/check/drawer but deferred its `spawn_loop` dispatcher (lines 75-87).
- [E] Open patch context recorded by the summary: patch-R2.3 `lenient_vec` deserializer; patch-R2.2 Windows IPC path and agent stderr; patch-R2.1 B3 prompt rephrase; patch-R2 async OAuth `CloudGroundingClient` plus B3 cloud mode (lines 89-91).

### B-1 - Path Unification (`.elai` -> `.elai_cc`) - DONE

Source: lines 101-112.

- [E] Audit all `.elai/` literals and establish `ELAI_DATA_DIR_NAME = ".elai_cc"` and `INDEXES_DIR_NAME = "indexes"` in `crates/core/src/paths.rs`.
- [E] Central path helpers are `dkg_db_path`, `bm25_dir_path`, `vectors_db_path`, `sessions_dir_path`, `metrics_dir_path`, `models_dir_path`, and `rules_db_path`.
- [E] Fix the identified CLI and `indexing_service.rs` call sites and migrate rule-engine reads of `.elai/sessions/dkg.db`.
- [E] CLI bootstrap must assert that `.elai_cc` exists; it must not silently fall back to an `EmptyDkgQuerier`.
- [E] Runtime effect: unblock PageRank, CatRAG, A-RAG, and `DkgQuerierAdapter`; produce real Phase-1D symbols, populated 1E CatRAG data, real Telescope counts, and allow approximately five phase audits to move from PARTIAL to WIRED.

### B-2 - FSM `PhaseTransition` wiring - DONE

Source: lines 114-124.

- [E] `Fsm::transition_with_metrics` writes `writer.write_payload("phase_transition", 0, SsbPayload::PhaseTransition(...), Vec::new())`.
- [E] `Fsm` gains an optional `ssb_writer: Option<Arc<dyn SsbWriter>>`, preserving testability without a writer.
- [E] The older `emit_fsm_transition` producer is migrated to the same `phase_transition` action type; two parallel event names must not be maintained.
- [E] SSB write failure is explicitly non-blocking for the FSM transition.
- [E] Consuming UI behavior includes the StatusBar phase badge, EXECUTING pulse, `MonitorWindow.currently`, and the phase-transition renderer; in V2 the event appears as a halo around the central orbit node plus a bubble-up strip event.

### B-3 - Stub-command eradication - DONE in re-scoped V2 form

Source: lines 126-141.

- [E] Every dead UI element is classified as Class A `IMPLEMENT`, Class B `DELETE`, or Class C `RENAME/DEFER`, with Class C visibly labeled `(post-MVP, see B-N)`.
- [E] Audit inventory: KnowledgeDrawer has languages, PageRank, query-tier stats, freshness, and `knowledge_degradation`; AntiStalenessDrawer has `dep_audit`, `baseline_harness`, and `deterministic_audit` (Class C); ReasoningDrawer has synthetic GRC data and a label correction; RulesDrawer needs live update plus `rule_transitions` echo; Plan UI has an unregistered V1 PlanWindow and stub tree readers; Settings has a `SessionHandoffEntry` schema correction and fake Reindex button; Activity has `branch_merge`, an isolation metric to delete, and `/continue`; four StatusBar elements are activated by B-2; branch operations include dashboard `/session fork` plus merge/close; future Class-C surfaces are `AutoAdvanceStats`, `ProposalsPanel`, `MONITOR`, and `EXPLORE`.
- [E] `ts-rs` is introduced and intended for reuse by later B plans.
- [E] V2 responsibility split: B-3 performs audit/classification and Class B/C deletion or relabeling; B-36 performs Class A live wiring, avoiding duplicate edits to `stub.ts`.

### B-4 - Knowledge producer wiring - DONE; `code_vec` remains opt-in

Source: lines 143-159.

- [E] Stage cross-encoder-l12, GraphCodeBERT, and Granite ONNX models under `.elai_cc/models/onnx/`; setup verifies downloads with blake3.
- [E] Wire the `SearchMode` degradation cascade into `QueryPipeline`, replacing `NoopSsbNotifier` with a real notifier.
- [E] Runtime `code_vec` queries participate in three-ranker reciprocal-rank fusion: BM25 + code vector + text vector. Full rollout remains behind `ELAI_CODE_VEC=1` pending testing (title and lines 143-148; follow-up lines 1104-1107).
- [E] Dashboard/command observability: `get_query_trace` QueryTrace inspector; `get_circuit_breaker_states` per-model `EmbeddingCircuitBreaker` grid; `hyde_query` preview/event; per-tier R@5, R@10, and MRR badges.
- [E] Correct the DEEP-tier MRR monotonicity violation where DEEP underperforms STANDARD.
- [E] Thread `Clock` through stage constructors for real per-stage latency and remove the entropy-quarantine workaround.
- [E] Replace reranker `.expect("semaphore closed")` with the circuit-breaker path.
- [E] Replace fourteen `Box<dyn Error>` uses with typed `Bm25Error`, `VectorError`, and `PagerankError` thiserror enums.
- [E] Events: `knowledge_degradation`, `hyde_query`, `query_trace`, `circuit_breaker_state_change`, `vector_cache_hit_rate`.

### B-5 - Reasoning producer wiring - DONE; per-agent budget producer still TODO

Source: lines 161-172.

- [E] Replace synthetic `emit_reasoning` values such as fixed `agent_id="task_runner"` and `grounding_level="medium"` with values sourced from `PipelineGrc`.
- [E] Events: `grc_step_emitted`, `grounding_sweep_result`, `csq_evaluated`, `feasibility_check`, `auditor_rubber_stamp_rejected` (the prose says four-to-five, while the registry lists five).
- [E] Tauri commands: `get_grc_budgets_per_agent`, `get_grounding_sweep`, `get_csq_log`, `get_feasibility_log`, `get_agent_grc_config`.
- [E] ReasoningDrawer: eight per-agent budget gauges, grounding-sweep pie chart, CSQ guard section, feasibility-check display, and per-role configuration viewer; divergent B-15 table values are highlighted red.
- [E] Show an anti-rubber-stamp counter and an intervention card when a Critical CSQ lacks a guard.

### B-6 - Orchestration producer wiring - DONE

Source: lines 174-187.

- [E] EvidenceGate verdict panel exposes the seven gates from `evidence.rs`.
- [E] InteractiveGate prompt UI is backed by `intervention_request` and `intervention_response` SSB rows.
- [E] AgentsPanel shows eight roles, live status, heartbeat dot, and restart count.
- [E] Wire `SupervisionManager` into TaskRunner, closing the Phase-3C infinite-restart risk.
- [E] Surface anti-spiral hits with a red badge at count three.
- [E] Surface BSP superstep count and parallel-agent runs through `bsp_superstep`.
- [E] Render ContextBudget as a per-slot bar chart from `get_last_context_budget`, plus AdaptiveLevel reduction events.
- [E] Implement `compute_overlap` for Phase 3E.5 `file_overlap_score`.
- [E] Events: `evidence_gate_evaluated`, `intervention_request`, `intervention_response`, `supervision_event`, `anti_spiral_hit`, `bsp_superstep`, `context_reduction`, `ssb_filtered_by_overlap`, `agent_status_changed`; schema 25 -> 26.

### B-7 - Pipeline Factory producer wiring (4D + 4E) - DONE via B-7b

Source: lines 189-206.

- [E] Register the PlanWindow in V1; V2 already has `lib/windows/PlanWindow.svelte` scaffolding.
- [E] DecompositionTree emits `concept_decomposition`, `phase_node_ready`, `step_node_ready`, and `baby_step_ready`.
- [E] Replace NotFound stubs with real `get_decomposition_tree`, `get_phase_detail`, and `get_baby_step_detail` reads.
- [E] Record DAG repair rounds and escalation through `dag_repair_round` and `dag_repair_escalated`.
- [E] Visualize `ExecutablePipeline` as a DAG using d3 or cytoscape.js.
- [E] Surface `PhaseSizingSplitter`, TreeNavigator cursor state, per-babystep `FailureRecord`, and typed `BubbleUpEvent` with five cascading actions.
- [E] Emit successful `IntegrationCheckpoint` rows, not only failures.
- [E] Emit per-node research stages `FORMULATE`, `SEARCH`, `EXTRACT`, `REFLECT`, and `CONVERGE`, replacing five coarse iteration-end tokens.
- [E] Show `PackageRef.verified` anti-hallucination badges and `TechStackDecision` values `RESEARCH`, `EXISTING_PROJECT`, `USER_SPECIFIED`, and `FALLBACK`.
- [E] Registry totals fourteen events and nine new/replaced Tauri commands; the cross-cutting registry names `phase_sized_split`, `tree_position_changed`, `failure_recorded`, `bubble_up_typed`, `integration_checkpoint_success`, `research_node_stage`, `package_ref_verified`, and `tech_stack_decided` in addition to the six names above (lines 967-975).

### B-8 - `PipelineGrc` activation in production hot paths - DONE

Source: lines 208-215.

- [E] Run `ground_plan` on PLANNING -> TESTING_FIRST so `ctx.ungrounded_claims` genuinely drives `GroundingSweepGate`.
- [E] Run `verify_claim` before ContextCompiler fetch to skip bogus dependencies.
- [E] Run `generate_csq` before write dispatch; a Critical CSQ without a guard blocks the write.
- [E] Run `check_inline_feasibility` during EXECUTING; `Blocked` becomes `ReplanRequest(MAJOR)`.
- [E] Activate anti-rubber-stamp Auditor enforcement.
- [E] Reset `InlineFeasibilityState` at each FSM phase boundary.

### B-9 - `SessionDag` activation - DONE; dashboard DAG writes deferred

Source: lines 217-228.

- [E] Populate `stub_ssb_seq`; otherwise trim pass 1 cannot produce recovery stubs.
- [E] Call `SessionDag::create_handoff` at 85% token-budget utilization.
- [E] `/continue` loads handoffs rather than the JSONL path.
- [E] BLOB fields migrate from serde_json to `rmp_serde` MessagePack per Phase 0.7b.
- [E] Create checkpoints per GRC step, tool call, and phase transition.
- [E] Dashboard branch commands are `/session fork|merge|close|list`.
- [E] Spec Amendment #2 requires SSB action type `checkpoint` on `merge_branch`.
- [E] Apply the `ts-rs` `SessionHandoffEntry` correction, add a `/continue` handler, and show handoff threshold in the StatusBar.

### B-10 - ContextBudget profile and 3E.5 overlap activation - DONE

Source: lines 230-240.

- [E] Stop passing `profile=None` at five TaskRunner sites, which otherwise produces a zero-weight allocation via `ContextBudget::compute`.
- [E] Detect model tier at bootstrap through sidecar `/v1/models`, vLLM `/v1/models`, Ollama `/api/tags`, or cloud configuration.
- [E] Carry `ModelCapabilityProfile` through all five TaskRunner sites.
- [E] `compute_overlap(agent_files, ssb_entries)` filters SSB entries and emits `ssb_filtered_by_overlap`.
- [E] Per-tier slot weights follow `token-efficiency-design.txt` section 2; the given 7B/12GB example is 12.5/66.7/12.5/8.3 for RepoMap/CodeContext/SsbEntries/GrcHistory.
- [E] A contract test enforces TaskRunner and REPL parity.

### B-11 - Design-system restoration - LARGELY DONE in V2

Source: lines 246-254.

- [E] Audit and backfill V2 token drift against `dashboard-design.txt`, including `bg-void #0a0e12`, teal `#1B7A6E`, purple `#7C5CBF`, and the remaining declared tokens.
- [E] Constrain type scale to 10/12/13/15/18 and audit hover-reveal/orbit animation timing against DESIGN sections 7 and 8.
- [E] Add Playwright/Vitest visual regression coverage in `crates/dashboard-v2/tests/visual.spec.ts`.
- [E] Existing design dependencies are JetBrains Mono via `@fontsource-variable/jetbrains-mono`, Lucide via `@lucide/svelte`, and three.js plus `troika-three-text` for the orbit.

### B-12 - Workspace shell decision - DONE in V2

Source: lines 256-261.

- [E] `screens/Workspace.svelte` is the canonical primary shell; Tier-0 Shell is retired.
- [E] Remaining verification/backfill: ensure StatusBar is inside Workspace; ensure `screens/ProjectSelector.svelte` does not ask the retired tier0/workspace question; enforce a five-icon, 48px Sidebar; remove `elai_cc.toml [dashboard] primary_shell`.

### B-13 - Activity feed restoration - REPLACED by orbit; re-scoped form DONE

Source: lines 263-272.

- [E] ActivityWindow is the orbit visualizer, not a text feed.
- [E] FSM-driven orbit modes: PLAN constellation; MAP for an existing project; RESEARCH stages FORMULATE -> SEARCH -> EXTRACT -> REFLECT -> CONVERGE with phi/psi twin micro-bars.
- [E] Wire orbit captions in `lib/orbit/captions.ts`; add `BubbleUpStrip.svelte`; add `OrbitGrcDots`, `OrbitBudgetRing`, and `OrbitEvidenceGates` signal layers.
- [E] Idle `OrbitShootingStar` occurs about every 14 seconds and is deterministic through an entropy-quarantined LCG.
- [E] Implement hover-reveal chrome per DESIGN section 7.

### B-14 - Live token-streaming chat - DONE

Source: lines 274-286.

- [E] V2 Tauri exposes `stream_chat(message)`; chat UI is `CommandWindow.svelte` plus `CommandPalette.svelte`.
- [E] Frontend token streaming uses Tauri 2 events through `listen` from `@tauri-apps/api/event`.
- [E] Intent classification is deterministic first with an LLM fallback and returns `TASK`, `FIX`, `EXPLORE`, `CONFIG`, `INTERVENE`, `STEER`, or `ABORT`.
- [E] Card variants are status, intervention, error, disambiguation, steering, and user; CTA language uses outlined buttons only.
- [E] Input placeholder changes by FSM state; Stop calls `cancel_stream`.
- [E] Token buffer and subscription cleanup use Svelte 5 `$state` and `$effect` respectively.

### B-15 - Phase-0 debt burndown - PARTIAL

Source: lines 292-310.

- [E] Completed: `cli/src/main.rs` decomposed from 10,570 to 252 LOC into `args.rs`, `auth.rs`, `help.rs`, `live_cli.rs`, `progress.rs`, `resume.rs`, `runtime_builder.rs`, `runtime_client.rs`, `tool_executor.rs`, and `tool_formatting.rs`; tests moved to `cli_tests.rs` and `sandbox_report_tests.rs`; 178/180 pass.
- [E] Remaining deletion: `compact.rs`, `oauth.rs`, `remote.rs`, and `prompt.rs`.
- [E] Reduce `usage.rs` to spec minimum by deleting `ModelPricing`, `UsageCostEstimate`, and `format_usd`; B-27 later restores cost reporting through `CLOUD_SPEND`.
- [E] Replace 84 `std::collections::HashMap` imports with BTreeMap, IndexMap, or FxHashMap according to use, enforced by CI.
- [E] Replace 40+ direct `SystemTime::now`/`Instant::now` calls with injected `ClockProvider`.
- [E] Move `file_ops`, `bash_validation`, and `stale_branch` from orchestrator to tools.
- [E] Fix the hard-coded Windows `sh` in `hooks.rs`.
- [E] Add `check_entropy_quarantine.sh`, `determinism_proof.sh`, and `smoke_test_ollama.sh`.
- [E] Generate `.elai_cc/metrics/pre_phase_0.toml` and add a CI workflow.

### B-16 - Anti-stale rule enforcement - PLANNED

Source: lines 312-321.

- [E] Keep the ort 2.x release-candidate track pinned to the current `=2.0.0-rc.12`; this is explicitly not a stable-only rule. Re-evaluate when ort 2.0.0 stable is released. E-34 owns the amendment.
- [E] Replace fifteen remaining `Box<dyn Error>` instances with typed thiserror variants.
- [E] Remove `.unwrap`/`.expect` outside tests, main, and const initialization; migrate `log::` to `tracing::`; replace magic numbers with named constants; replace narrowing `as` casts with `.try_into()?`.
- [E] Use blake3 rather than blake2 for this non-RFC hashing role.
- [E] CI lint patterns cover boxed dynamic errors, forbidden unwraps, `log::`, numeric narrowing casts, `std::collections::HashMap`, and direct `SystemTime::now`.

### B-17 - Ground vacuous gates - PLANNED

Source: lines 323-329.

- [E] Add `tests/contracts/gate_liveness.rs` with individual liveness tests for GroundingSweepGate, SearchMode cascade, SupervisionManager, CSQ, inline feasibility, and anti-rubber-stamp enforcement.
- [E] Add a dashboard GatesPanel showing per-gate verdict counts.
- [E] Add a CI gate-liveness check.
- [E] Produce a decision document defining whether dismissing an INTERACTIVE prompt blocks.
- [E] Enforce a performance target below 100ms per FSM transition while gates are active.

### B-18 - Tier-C sandbox activation - PLANNED

Source: lines 331-347.

- [E] Linux isolation uses `clone3` with `CLONE_NEWNS|CLONE_NEWPID|CLONE_NEWNET`; a seccomp-bpf IPC baseline plus researcher-only network additions; `PR_SET_NO_NEW_PRIVS`; limits of 60 seconds soft CPU, 2GB memory, 256 files, and 32 processes; `/proc hidepid=2`; and capability bounding-set drop.
- [E] Windows enables `enable_sandbox=true` at three entry points, launches via `self.sandbox.launch_agent()`, creates one AppContainer profile per `agent_id`, and falls back to a restricted token plus Job object.
- [E] IPC accept loop is `UnixListener` on Linux or `NamedPipeServer` on Windows; frames pass through `IpcGateway`; dead `IpcHookDispatcher` wiring becomes live.
- [E] Visibility filtering must return NotFound semantics instead of `error_code="visibility_filtered"`; each `AgentHandle` owns `allowed_paths`.
- [E] Self-modification is protected at dispatch, IPC, filesystem, and git layers.
- [E] Replace fake namespace tests with real `/proc/{pid}/status` `NSpid` checks.
- [E] `phase_4A.toml` must contain real isolation/IPC metrics, not an in-process postcard round-trip mislabeled as IPC latency.

### B-19 - Hook-system unification - PLANNED

Source: lines 349-364.

- [E] Keep the spec-compliant `IpcHookDispatcher` after B-18; delete `InProcessHookDispatcher`.
- [E] Migrate all identified TaskRunner hook call sites to IPC.
- [E] Stop hook must enforce a block outside IDLE instead of emitting telemetry only.
- [E] `PreCompact` calls `SessionDag::create_handoff` rather than a no-op.
- [E] Ruff runs after Python writes with a five-second timeout and content-hash cache.
- [E] DAIRA expands from two to fifteen markers: call, return, mutation, branch, error, finding, read, write, tool_call_start, tool_call_end, hook_pre, hook_post, timeout, escalate, converge.
- [E] Add `AdaptiveTool<T>` and an SSB `MarkerReport`.
- [E] Enforce hook budgets T1 <1ms, T2 <10ms, and T3 <5ms; emit `hook_latency_violation` on overrun.
- [E] Rules discovered by the Auditor require independent approval before ACTIVE promotion.

### B-20 - Tool-registry unification - PLANNED

Source: lines 366-376.

- [E] Keep `IpcToolRegistry` after B-18; delete `InProcessToolDispatcher`.
- [E] Migrate FSM filtering and TestingFirst gating onto IPC.
- [E] Collapse the failure taxonomy from thirteen to nine exact values: `SYNTAX_ERROR`, `TYPE_MISMATCH`, `UNDEFINED_SYMBOL`, `MISSING_IMPORT`, `LIFETIME_ERROR`, `BORROW_ERROR`, `UNUSED_WARNING`, `TEST_ASSERTION`, `RUNTIME_PANIC`.
- [E] On edit failure, add a SequenceMatcher similarity hint after three-tier edit matching.
- [E] File-outline mode has a `force_full` flag plus a 50% budget guard.
- [E] `IntegrationTestRunner` emits success rows as well as failures.
- [E] After five or more edits, create per-WIP commits; after a successful audit, squash them on AUDITING -> MEMORIZING.

### B-21 - Phase 4F feasibility probe - PLANNED

Source: lines 382-397.

- [E] `SystemProbe` gathers GPU VRAM with `nvml-wrapper` and RAM/disk with `sysinfo`; `SystemConstraints` evaluation is pure.
- [E] A novel step is one that uses an unverified `PackageRef`, an unused framework, or an untested capability.
- [E] Probe novel work with a Tier-C stub under 50 LOC; allow 25 seconds gracefully and kill at 30 seconds with explicit `child.kill().await` because Tokio `kill_on_drop=false`.
- [E] Feasibility-role `AgentDispatchConfig` overrides are GRC `3/600`, `tool_filter=[stub_write,bash]`, and 25/30-second timeouts.
- [E] Skip the probe iff there are zero novel steps and every required resource has at least 20% margin.
- [E] Branch-and-explore evaluates at most three alternatives for each novel step, one concept at a time.
- [E] Exact types: `SystemProbeResult`, `GpuInfo`, `SystemConstraints`, `StubResult`, `ResourceDelta`, `ConstraintResult`, `FeasibilityReport`, `FeasibilityEscalation`, `FeasibilityDecision`; `FeasibilityProbe` is object-safe.
- [E] Verification target is 24+ tests (sixteen contract, eight adversarial) plus dashboard `FeasibilityPanel`.

### B-22 - Phase 4G autonomous mode - DONE; B-22b follow-up PLANNED

Source: lines 399-431.

- [E] `AutonomousGate` wraps `InteractiveGate` through `DispatchPipeline`/`PlanDispatchTable`; human delegation occurs only for HIGH risk, missing evidence, or an escalation trigger.
- [E] Six evidence checks cover knowledge readiness, zero UNGROUNDED claims, tests failing correctly, a flipped test, zero BLOCK findings, and DKG plus changelog evidence.
- [E] Four base escalation triggers: three anti-spiral hits, HIGH risk, `ReplanRequest(MAJOR)`, or circuit breaker beyond ten auto-advances.
- [E] When ContextBudget is Terminal, `SPLIT_TASK` creates at most four subtasks to depth two.
- [E] At max restarts, `SUBSTITUTE_MODEL` traverses a `ModelPool` swap chain and emits `model_swap`.
- [E] CLI surfaces `/mode` and `/model`.
- [E] `TreeAutoAdvancePolicy`: BabyStep LOW, Step LOW, Phase MEDIUM, Concept HIGH; Concept always requires a human; circuit breaker is fifty babysteps per concept.
- [E] Implementation surfaces include eighteen extended contracts, `SlashCommand::Mode`, `derive_autonomous_evidence` querying `DkgReady`, `Finding`, `TestOutcome`, and `PhaseTransition`, `SsbViewer::auto_advance_stats`, MonitorWindow `AutoAdvanceStats`, and `AgentPool::get_role`.
- [E] Patch-4G.1 makes the CommandPalette toggle call `stub.set_gate_mode` IPC, makes the StatusBar gate badge always visible, and reconciles mode on Workspace mount with `session.reconcileGateMode()`.
- [E] Base event registry: `auto_advance_decision`, `model_swap`, `autonomy_escalation`, `split_task_initiated` (lines 967-977).
- [E] Open base follow-ups: live `/mode` wiring rather than informational output; feed GateDecision rows to orbit per-decision visuals; remove/resolve the `LIMIT 1000` evidence-query cap; use `SeededClock` for deterministic `last_24h_rate`.
- [E] Planned B-22b/4G.9: replace the lone repeated-step rule with a four-pattern `StagnationDetector` (`Spinning`, `Oscillation`, `NoDrift`, `DiminishingReturns`).
- [E] B-22b `LateralThinking` has five personas (`Hacker`, `Researcher`, `Simplifier`, `Architect`, `Contrarian`) plus an affinity table and corresponding `.claude/agents/persona-*.md` files.
- [E] B-22b `GenerationWatchdog`: idle timeout 60s, no-material-progress 180s, safety 600s, and `workflow_fingerprint` deduplication.
- [E] B-22b goal-drift uses Jaccard bands 0.5/0.7; the cheap pre-audit Block is intended to save an Auditor model call.

### B-23 - Phase 4H self-modification (DGM/CEL/DarwinNet) - PLANNED

Source: lines 433-458.

- [E] L1 collects automatic metrics without an LLM.
- [E] L1A automatically grows additive data-table rows without an LLM or staged evaluation in `deprecated_patterns`, `FailureTaxonomy`, lazy-code regex, and source-tier; this is the DarwinNet Gene Pool lane.
- [E] L2 proposes configuration variants using DGM sigmoid parent selection (`lambda=10`, `alpha_0=0.5`), retains at most fifty variants per `PipelineMode`, and stages evaluation at 10/50/200 cases.
- [E] `TestOutcomeReport` uses four-category SWE-bench grading; `PASS_TO_FAIL` rejects immediately.
- [E] L3 structural change remains HUMAN-ONLY stubs and emits `l3_proposal`.
- [E] Modification-Solidification Index uses exponential decay `lambda=0.3`, `N=10` per PipelineMode. `IMPROVING >0.3` continues; `STABILIZING -0.1..0.3` reduces; `DIVERGING <=-0.1` pauses and alerts.
- [E] Auto-revert occurs after more than 10% degradation across three pipelines.
- [E] `AllowedConfigKey` is a closed enum; cloud budget, gate definitions, and score weights are locked and never mutable.
- [E] Four independent safety enforcement layers are dispatch, IPC, filesystem, and git; the self-mod agent cannot read the lock list.
- [E] OSC/4H.7B similarity is `0.5*key_overlap + 0.3*type_match + 0.2*exact_match`; score >=0.95 across three means stagnant and boosts exploration; period-two oscillation triggers `PERIOD-2 FULL VERIFICATION` and freezes the involved keys.
- [E] REG/4H.7C scans full `PipelineOutcome` history, blocks 4H.5 promotion, and triggers 4H.6 auto-revert after three consecutive regressed pipelines.
- [E] EVO/4H.7D detects a false IMPROVING label when MSI changes due to noise while `config_blob` is structurally identical; promotion is withheld and MSI forced to STABILIZING.
- [E] Eleven events: base `self_mod_proposed`, `self_mod_evaluated`, `self_mod_promoted`, `self_mod_reverted`, `msi_state_changed`, `data_table_grew`, `l3_proposal`, plus `convergence_stagnant`, `config_oscillation`, `regression_detected`, `evolution_withheld` (lines 967-978). Verification target: 54+ tests.

### B-24 - Phase 4I media validator - DONE

Source: lines 460-482.

- [E] Tier 1 is deterministic/in-process and targeted below 100ms: image format/resolution via ImageMagick; SSIM/LPIPS through ONNX Runtime CPU EP (GPU explicitly prohibited as nondeterministic); 3D vertex/triangle counts via Blender CLI; audio sample rate via FFmpeg; OCR via Tesseract.
- [E] Tier 2 is an advisory VLM targeted at 2-5 seconds: Qwen3.5-VL-30B-A3B via Ollama (~17GB Q4) or InternVL3.5-38B via SGLang (~38GB 8-bit); output is constrained to `VlmValidationOutput` JSON.
- [E] Concurrency is bounded to four Tier-1 jobs and one Tier-2 job.
- [E] CLAP audio-text is always loaded at about 0.4GB FP16 whenever audio is active.
- [E] Verdicts: Tier-1 failure -> `Fail`; Tier-1 pass plus Tier-2 disagreement -> `HumanReview`; unavailable VLM -> `Degraded`.
- [E] RCE visual feedback loop is execute -> render -> VLM evaluate -> structured feedback -> executor iteration; convergence stops when issues do not change.
- [E] Five `MediaRenderAdapter`s: Playwright web, Blender 3D, FFmpeg animation, direct screenshot read, ComfyUI generation.
- [E] Media modules instantiate lazily only when the `PipelineSpec` has `MediaSpec`.
- [E] Production pieces include Mock/Ollama/SGLang advisory clients, `media-validator-rce`, `wire_media_validator_for_spec`, Arc<Ssb>-backed `SsbMediaSink`, TaskRunner integration after `spec_from_decomposition`, and V2 `MediaValidationPanel` with four Tauri commands and six SSB variants (variant names are not given in the summary).
- [E] Verification ledger: 71 library, 21 RCE, 19 dashboard, and two e2e tests pass. `cloud_mode_live` remains ignored and pins `claude-sonnet-4-5-20250929`.

### B-25 - Phase 4J Empire integration - DEFERRED/conditional

Source: lines 484-497.

- [E] Pure-Rust FactoryLine client speaks newline-delimited JSON over a generic `Transport`, with Unix transport on Linux and TCP on Windows.
- [E] Brain registration uses a five-second heartbeat plus reconnect and re-registration.
- [E] Replace the broken `brainctl` and require zero Python dependencies.
- [E] Empire event subscription bridges to SSB, defaults to 100 events/second, uses a filter allowlist and LOW/MEDIUM/HIGH priorities, and admits events as GRC `SOURCE_CHECK` grounding.
- [E] `ElaiNeuron` MCP server exposes `task_submit`, `task_status`, `pipeline_result`, and `health_check`; ToolFilter applies and no internal state is leaked.
- [E] Repair detects only verified `CircularDep`, `HardImport`, or `StdlibShadow`, generates a `PipelineSpec`, and validates it through `TestOutcomeReport`.
- [E] Everything is behind `cfg(feature="empire")` and degrades gracefully.

### B-26 - Phase 4K.6-4K.13 dashboard completion - DONE majority; six polish items remain

Source: lines 499-521.

- [E] Shipped V2 surfaces: polling `MonitorWindow`, `ExploreWindow`, `ReviewWindow`, Browser-API/localStorage `NotificationManager`, intervention request/response SSB types, `submit_intervention_response`, `set_webhook_url`, `get_webhook_url`, tab open/close/move with cross-panel drag, schema-v1 localStorage persistence, keyboard shortcuts `Ctrl+1..7/N/W/D/.,/Shift+P/Esc`, FSM-state notification triggers, and client-side session recovery.
- [E] Decision D1(a): drop `WINDOW_STATE` SSB and use localStorage only; there is no cross-device window-state synchronization.
- [E] Decision T4: keep the intervention card inline in MonitorWindow rather than extracting it prematurely.
- [E] Remaining webhook behavior: POST a Discord/Slack-shaped payload on `INTERVENTION_REQUIRED`, `AUDITING`, post-task `IDLE`, or when pending interventions becomes non-empty; add SettingsWindow webhook URL input.
- [E] Remaining ReviewWindow commands: window-scoped `Ctrl+Shift+A` accept-all and `Ctrl+Shift+R` reject-all with active-window guarding.
- [E] Remove misleading `Ctrl+Shift+H`, which invokes CommandPalette while claiming to recover a closed window.
- [E] Replace the static five-phase dot map with real plan progression through a new `get_plan_progress` command or extended `FsmSnapshot`.
- [E] Add panel-tab transition timing: 80ms fade-out then 120ms fade-in.
- [E] Responsive behavior: below 1000px Sidebar becomes a hamburger; below 800px the secondary panel collapses.

### B-27 - Cloud mode - PLANNED

Source: lines 523-538.

- [E] Per-task `LOCAL` is default, costs $0, and uses six reasoning layers. `CLOUD` lets cloud generate while local deterministic L1 still runs and L2-L5 are skipped. `HYBRID` tries local first and escalates only a failing L3+ step.
- [E] `[cloud]` settings: `provider="anthropic"`, `model="claude-sonnet-4-6"`, `escalation_model="claude-opus-4-6"`, `max_spend_per_task`, `max_spend_per_day`, `warn_at_percent`.
- [E] Emit `CLOUD_SPEND`; enforce ceilings; expose `/task --mode`.
- [E] Cloud ContextBudget profile is 200K versus local 8-25K; cloud babysteps are 300-500 LOC versus local 25-40; decomposition depth is two versus four; split threshold is 50K versus 3K; reasoning layers collapse.
- [E] Privacy controls: `redact_env_vars` default, regex `redact_patterns`, glob `blocked_paths`; every cloud call gets an audit trail.
- [E] BYOE uses a custom OpenAI-compatible `custom_endpoint` URL; `[execution] air_gapped=true` disables cloud reachability.
- [E] Emit `ESCALATE_TO_CLOUD`; estimate cost before the task; show a per-task Monitor spend gauge; warn at 80% and 100%; expose `CloudPanel`.

### B-28 - Focus Tunnel - PLANNED

Source: lines 540-565.

- [E] Deterministic emission mode selection has three modes: `FIM` for single-region edits with native Qwen2.5-Coder fill-in-middle and no tool schemas (~350 tokens saved/step); `DIFF` for multi-region unified diffs, roughly 30-50% smaller than a full file and using B-20 three-tier match; `STRUCTURED` is the default/fallback for new files and cloud.
- [E] `ModelCapabilities.supports_fim` is true for Qwen2.5-Coder, StarCoder, and DeepSeek-Coder, false for cloud; `ModeSelector` applies the canonical spec rules.
- [E] Token-level streaming validation checks imports against `deprecated_patterns`, detects lazy code such as `todo!`/`pass`, and can terminate early.
- [E] Chunk-level validation does a partial tree-sitter parse about every fifty tokens.
- [E] Expected bad-generation saving is approximately 90% when caught at token 50 rather than 500.
- [E] Depth-one pipeline parallelism prepares step N+1 context while step N validates; it disables adaptively if validation pass rate falls below 80%.
- [E] Automated dependency resolution is detect missing import -> anti-staleness check -> registry lookup -> conflict check -> manifest edit -> install. The model never runs `cargo add` or `pip install`; autonomous mode requires a pre-approved allowlist.
- [E] Speculative file routing uses `target_file`; AST mismatch creates a file in a microstep and retries, capped at two microsteps per babystep.
- [E] Claimed estimate: about 6,800 tokens/27% saved on a thirty-step 25K task; with ShortCoder, 45-65% effective budget increase.
- [E] Events: `generation_mode_selected`, `streaming_early_terminated`, `pipeline_pre_assembled`, `dependency_resolved`.

### B-29 - Symbol-level impact analysis API - PLANNED; highest-leverage recommendation

Source: lines 571-590.

- [E] Four agent-callable operations use a sharded, LRU-bounded `SymbolGraphCache`: `compute_impact_radius` does reverse-call BFS with hop bands and depth <=10; `compute_call_flow` does cycle-aware DFS with lazy children and depth <=10; `compute_symbol_context` returns definition plus confidence-tagged callers/callees; `list_symbols` filters by file/query/limit.
- [E] Cache layout: 27 name shards (`a-z` plus `_`), 256 reverse shards keyed by single-byte SHA1 prefix, and a 500-file payload LRU. A query must never load every symbol or edge.
- [E] Resolver confidence is one of `Local`, `Imported` (one hop), `Re-export` (two hops), or `Unresolved`.
- [E] DKG schema v3 tables are `symbol_name_index`, `symbol_reverse_index`, `symbol_file_payload`, and `symbol_graph_meta`, with MessagePack BLOBs.
- [E] Dispatcher entries are `symbol_impact`, `symbol_flow`, `symbol_context`, `symbol_list`.
- [E] Auditor `PostToolUse` auto-injects `IMPACT_SUMMARY` at depth two for changed files.
- [E] Fewer than fifty changed files triggers incremental invalidation; more than fifty triggers full rebuild.
- [E] Implement LRU using std only, with no `lru` crate. FocalAD (`arXiv:2506.11419`) is the stated research anchor for top-k local focus.

### B-29.5 - Symbol-graph scale benchmark - PLANNED release gate

Source: lines 592-604.

- [E] Pinned fixtures: Linux v6.7 (~36M LOC), Chromium 121.0.6167.184 (~35M), VS Code 1.94.2 (~2.45M), PostgreSQL `REL_17_0` (~1.5M).
- [E] Acceptance: traversal-tool p99 <=500ms; LRU peak RSS <=2GB over 1,000 queries; byte-identical BTreeMap hop-band output across five runs; document unresolved-edge percentage per language.
- [E] Criterion harness includes `scale_memory.rs` using procfs on Linux or `GetProcessMemoryInfo` on Windows and `workload.rs` weighted 60% impact radius, 25% symbol context, 10% call flow, 5% list symbols.
- [E] CI contracts are `scale_p99_budget`, `scale_resume_identity`, and `scale_long_tail_memory`.
- [E] Establishes a policy that future knowledge-engine changes rerun the scale benchmark.

### B-30 - Multi-language DKG hardening - PLANNED

Source: lines 606-624.

- [E] Support 13+ tree-sitter grammars with per-grammar panic isolation using `catch_unwind` around `Parser::set_language`.
- [E] Explicit extractor list: Python, TypeScript, TSX, JavaScript, Go, Java, Kotlin, Scala, C, C++, C#, Ruby, PHP, Swift, Bash, Svelte, Vue.
- [E] JVM resolver maps Maven/Gradle `/src/main/<lang>/` layout to fully-qualified names.
- [E] C# resolver supports block and C#10+ file-scoped namespaces and sorts candidates lexicographically for determinism.
- [E] Go resolver reads module identity from `go.mod`.
- [E] TypeScript resolver reads `compilerOptions.paths` from `tsconfig.json`/`jsconfig.json`, follows `extends`, and prevents cycles.
- [E] Svelte/Vue composite parsing finds `script_element` in HTML and reparses its body as TypeScript; symbol names are qualified `<file>.svelte::<symbol>`.
- [E] Extract CSS `@import` from style blocks.
- [E] `LanguageRegistry` records `loaded: BTreeSet<String>` and `failed: BTreeMap<String,String>` and publishes `LanguageGrammarStatus` at startup.
- [E] Tree-sitter is retained; ast-grep is not substituted.

### B-31 - Resumable indexing, cross-process lock, and startup auto-resume - PLANNED

Source: lines 626-642.

- [E] Checkpoint every fifty files; store blake3 `file_hashes`; skip unchanged content on resume; honor cancellation at every batch boundary.
- [E] Metadata schema v2 adds `project_metadata` and `file_hashes`.
- [E] `IndexLock` uses `fs2` only if already in the workspace, otherwise hand-rolled `O_EXCL+PID`; lock scope is `Index` or `Watch`.
- [E] PID liveness uses `kill(pid,0)` on POSIX or `OpenProcess` on Windows; stale threshold is two minutes; refresh every thirty seconds; RAII releases on Drop.
- [E] `auto_resume_indexed_projects()` is called by both CLI main and dashboard run, starts a fire-and-forget Tokio task per project, resumes `in-progress`, starts a watcher for `completed`, and leaves `cancelled` untouched.
- [E] `graceful_shutdown(signal)` has a 60-second deadline, cancels all in-progress work, releases watcher locks, then force-releases lock files after the deadline.
- [E] Events: `IndexCheckpoint`, `IndexResume`, `IndexCancel`, `LockAcquired`, `LockContested`.
- [E] `IndexingProgressPanel` shows live checkpoints and resume/cancel banners.
- [E] Run one-time `migrateAbsolutePathKeys` after B-1 path unification.

### B-31.5 - Resumable-indexing scale benchmark - PLANNED release gate

Source: lines 644-655.

- [E] Tier A is full clean indexing of the four fixture scales, estimated at 3-4 hours.
- [E] Tier B kills and resumes at 10%, 50%, and 90%; SQLite `.dump` must be byte-identical post-resume across at least six runs.
- [E] Tier C starts two processes; process B exits within one second with `Another process is indexing` plus holder PID; stale PID reclaim completes under five seconds.
- [E] Tier D verifies SIGTERM shutdown within 60 seconds and forced release after deadline using `ELAI_DEBUG_BATCH_DELAY_SECS`.
- [E] Tier E runs Chromium for four hours and requires RSS plateau/no monotonic leak.
- [E] Three CI contract tests are required; their names are not stated here.

### B-32 - Adaptive/minified-file chunking - PLANNED

Source: lines 657-667.

- [E] Before selecting an AST strategy, calculate `avg_line_length = len/max(line_count,1)`; over 500 selects character chunks and skips AST parsing.
- [E] Apply universal `MAX_CHUNK_CHARS=2000` after every AST, line-window, or character chunker.
- [E] `chunk_by_characters` scans backward for newline, space, tab, semicolon, or comma; chunk ID is the byte offset, not a line number.
- [E] Boundaries must be UTF-8 safe via `floor_char_boundary`.
- [E] Emit `MinifiedFileChunked`.
- [E] Fixtures include 50K-char one-line `bundle.min.js`, 150K-char `bootstrap.min.css`, generated `*.pb.go` with multi-thousand-character strings, and an emoji/UTF-8 variant.

### B-33 - Interactive decomposition/symbol graph viewer - PLANNED

Source: lines 669-688.

- [E] Modes are File, Symbol, and Decomposition.
- [E] Proposed npm dependencies are cytoscape, dagre, and fcose and require user confirmation.
- [E] UI placement remains an explicit choice: new GraphWindow (full flexibility), reuse orbit MAP mode (light), or KnowledgeDrawer only (smallest blast); B or C is recommended unless a contextual right-click flyout is wanted.
- [E] Interactions: click details; right-click blast radius with red gradient by hop; search filter; dagre/fcose/concentric/breadth-first/grid/circle layouts; PNG export; Mermaid export.
- [E] Mermaid export has language colors (Rust teal, Python blue, TS/JS yellow, Svelte orange, Go cyan, JVM purple, C# green), red-dotted cyclic edges labeled `cycle`, deterministic output, and a practical legibility target below 250 nodes.
- [E] CLI exposes `/graph mermaid` with paste-ready stdout.
- [E] Caps: `MAX_VIZ_SYMBOLS=20_000`, `MAX_VIZ_EDGES=60_000`, `MAX_VIZ_SYMS_PER_FILE=200`; `SymbolMode = Full | Capped | Omitted`.
- [E] Tauri: `get_graph_view_data`, `get_impact_overlay`, `export_graph_mermaid`. Cross-cutting registry additionally names `SymbolGraphUpdated` (lines 981-988).

### B-34 - Entry-point detection - PLANNED

Source: lines 690-705.

- [E] Candidate union has four heuristic families: Orphan means a file has no dependents but has outgoing calls; WellKnownName matches `main`, `Main`, or `__main__` with language-specific case rules; FrameworkRoute matches HTTP method callees or `route`; Test matches the specified test-name regexes plus exact `it`/`describe`.
- [E] Framework reasons are `framework:http-X` or `framework:route`.
- [E] Deduplicate on `(id, reason)`, allowing the same symbol under two distinct reasons.
- [E] Target is p99 below 500ms on a 10K-file project by loading payloads only for orphan and conventional-filename candidates, under 10% of files.
- [E] Designer ExistingProject receives `EXISTING_ENTRY_POINTS`; Researcher `/research` receives `PROJECT_STARTING_POINTS`.
- [E] `[entry_points]` in `elai_cc.toml` appends `ENTRY_POINT_NAMES` to built-ins rather than replacing them.
- [E] Cross-cutting event name is `EntryPointsDetected` (lines 981-988).

### B-35 - Context-artifacts pipeline - PLANNED

Source: lines 707-722.

- [E] `[[knowledge.artifacts]]` declarations contain `name`, file-or-directory `path`, and a description phrased as `check this before X`.
- [E] Each project gets a separate `context_*` vector-index collection, independent of the code DKG.
- [E] Non-code text reuses B-32 chunking; SQL chunks at `CREATE TABLE`, `VIEW`, or `INDEX` regex boundaries.
- [E] Staleness uses blake3; `ProjectMetadataStore` is idempotent.
- [E] `ArtifactStage` participates in STANDARD and DEEP; FAST remains code only.
- [E] `GroundingMethod::ArtifactCheck` is the seventh verification method. It dispatches `search_artifacts` with an artifact-name filter, returns Verified if a result score exceeds 0.5, and returns `MalformedGrounding` when the artifact was not declared.
- [E] Dispatcher tools: `list_artifacts`, `index_artifacts`, `search_artifacts`, `remove_artifacts` (the summary abbreviates these as list/index/search/remove_artifacts).
- [E] Events: `ArtifactsIndexed`, `ArtifactSearched`, `ArtifactStale`.
- [E] Designer ExistingProject automatically receives `AVAILABLE_ARTIFACTS`.
- [E] Manifest loading rejects paths resolving outside `project_root`.
- [E] Artifact-pipeline errors are non-fatal and never block code indexing.

### B-36 - Dashboard V2 `src-tauri` wiring - DONE via follow-up

Source: lines 728-758.

- [E] Work was organized as four sessions based on `HANDOFF-track-d-v2-rebaseline.md`; session 1 was analysis-only and produced `track-d-v2-spec.md` plus `B-plan-delta-2026-05-07.md`.
- [E] Session 2 added engine wiring/AppState and `sidecar_health` via `GET http://127.0.0.1:8100/health`; D.13 comprises `get_sidecar_settings`, `set_sidecar_settings`, and `restart_sidecar`, where restart means SSH to the remote box and never a local sidecar under HARDWARE-HEAVY; it also fixes `DASHBOARD_BINARY_NAME`.
- [E] Adding `elai-orchestrator`, `elai-cli`, `elai-core`, or `elai-providers` to `src-tauri/Cargo.toml` required user confirmation.
- [E] Session 3: D.2 `run_planning_task` emits `web_research_step`; D.3 `run_research` returns a Track-A.2 `ResearchAnswer`; D.7 `one_shot_prompt` is a subprocess controlled by `ANTHROPIC_BASE_URL`, `ANTHROPIC_API_KEY`, `ELAI_LOCAL_MIN_PROMPT`, and `ELAI_LOCAL_MAX_CONTEXT`.
- [E] Session 4 base commands: D.4 settings plus `obscura_test_fetch`; D.5 `run_d5_live_contract`; D.6 `reindex_now`; D.8 `tool_visibility`; D.9 `set_local_max_context`; D.10 `set_local_min_prompt`; D.11 `sidecar_search`; D.12 live-SSB `grounding_cache_stats`.
- [E] Fifteen V2-only surfaces: `recompute_steering_vector`, `get_steering_status`, `get_mcp_status`, `get_gg_signals`, `get_dkg_stats`, `get_sidecar_status`, `get_bubble_up_events`, `get_hook_executions`, `get_shortcoder_stats`, `run_explore_query`, `continue_task`, `get_grc_budgets`, `get_evidence_gates`, `get_context_budget`, `get_search_mode`.
- [E] `get_gg_signals` fields are `mean_logprob`, `min_logprob`, `entropy_mean`, `entropy_max`, and `generation_novelty`.
- [E] Implementation ledger: nine of thirteen D commands landed initially; D.2/D.3/D.5 followed; D.4 obscura uses subprocess fallback; about 637 LOC across four files.
- [E] Explicit prohibitions: do not back-port V1 `commands.rs` wholesale; do not touch orbit visual; do not boot a local sidecar; do not introduce frontend dependencies; do not commit without user request.

### B-37 - Phase 4M multi-runtime backend abstraction - SHIPPED

Source: lines 764-788.

- [E] One `RuntimeAdapter` covers eleven backends: `Sidecar`, `AnthropicApi`, CLI ports `ClaudeCode`, `CodexCli`, `OpenCode`, `Hermes`, `GeminiCli`, `KiroCli`, `CopilotCli`, and native HTTP `Ollama`, `Vllm`.
- [E] Every CLI subprocess runs in Tier-C isolation and its stdout passes the 4A IPC sanitizer before entering orchestrator prompt context.
- [E] `resolve_backend(candidate)` returns a complete supported-list error for an unknown backend; `create_runtime(cfg)` constructs the selected adapter.
- [E] Per-role configuration is `[agents.<role>.runtime] backend="..."`. Defaults: Designer/Researcher/Auditor/MediaValidator -> AnthropicApi; Executor/MemoryWriter/ContextCompiler/PromptEngineer -> Sidecar.
- [E] Runtime fallback is orthogonal to B-22 model substitution. On `Unavailable`, traverse `fallback_chain`, emit `runtime_fallback`, and if exhausted choose ESCALATE in INTERACTIVE or HALT in AUTONOMOUS.
- [E] CLI: `/runtime show|set|fallback`; V2 Settings has RuntimePanel with green/yellow/red health.
- [E] `PermissionMode` defaults to `AcceptEdits`; `DangerouslySkipPermissions` requires explicit opt-in. `stdout_idle_timeout` kills a stuck subprocess.
- [E] Events: `runtime_fallback`, `runtime_health_changed`, `cli_subprocess_spawned`, `cli_subprocess_killed`.
- [E] One-release migration alias maps `[backend] mode="local"` to `[runtime] backend="sidecar"`.
- [E] A live cloud Sonnet 4.6 run is recorded as proven.

### B-38 - Continuous ambiguity-score gate - PLANNED

Source: lines 790-811.

- [E] Extend B-17 GroundingSweepGate with `ambiguity = 1 - sum(clarity_i * weight_i)` and an overall pass threshold <=0.2.
- [E] Greenfield weights: Goal 40%, Constraint 30%, Success Criteria 30%. Brownfield: Goal 35%, Constraint 25%, Success 25%, Context 15%.
- [E] Per-dimension floors: Goal >=0.75, Constraint >=0.65, Success >=0.70, brownfield Context >=0.60. A low dimension blocks even if the aggregate passes.
- [E] Require two consecutive ready snapshots; one noisy snapshot cannot pass. Store the streak in `SessionState` and reset it on plan revision.
- [E] LLM scoring temperature is 0.1.
- [E] Token allocation starts at 2,048 and doubles on truncation for at most five retries; format errors also have at most five retries.
- [E] Events: `ambiguity_scored`, `ambiguity_floor_failed`, `ambiguity_streak_advanced`.
- [E] Verdict ordering: any ungrounded claim -> BLOCK; else ambiguity >0.2 or any floor failure -> BLOCK; else streak below two -> WAIT; otherwise ALLOW.

### B-39 - Chat REFLECT pipeline - DONE; B-39C answer substitution DONE

Source: lines 817-848.

- [E] Each opted-in chat turn wraps `LiveCli::run_turn` in TRIAGE -> REASONER -> AUDITOR -> SYNTHESIZE.
- [E] REASONER produces N=4 best-of-N candidates; AUDITOR refinement is capped at two. The stated research rationale is a 3B-7B N=4 inflection versus N=16 elbow.
- [E] Toggle with `/reason on|off`.
- [E] Each stage gets a KV reset through a separate `/v1/messages` call.
- [E] `ChatInferenceClient` and `ReflectEventSink` live in reasoning to avoid a circular Cargo dependency; `SidecarChatClient` adapts `SidecarProvider` in CLI.
- [E] Events: `ReflectTriaged`, `ReflectCandidate`, `ReflectAuditVerdict`, `ReflectRefined`; schema 28 -> 29.
- [E] `ssb_viewer.get_reflect_state`, a Tauri command, and the seventh ReasoningDrawer panel expose state.
- [E] Any pipeline error falls back to passthrough; the pipeline must never panic.
- [E] At 85% of `budget_cap_tokens`, abort remaining candidates. Default cap is 20K, preserving at least 5K under the 25K hard ceiling.
- [E] B-39C captures `ChatReflectPipeline.run` output. When not skipped, the Auditor-selected text renders with `TerminalRenderer::render_markdown` and bypasses `runtime.run_turn` completely; this D2a path is text-only with no tool execution. A skipped pipeline falls through.
- [E] B-39C runtime delegates are `push_user_text`, `push_assistant_text`, and `record_pipeline_usage`.
- [E] Deferred: B-39B AISLE/Mythos jagged-frontier evaluation needs a 3090 box; B-39D streams REFLECT progress in UI; B-39E D2f injects `reasoning_summary` as a system hint into the normal tool loop at the cost of one extra model call; B-39F guards session-length compaction.

### B-40 - Idea framer, domain research presets, and critic loop - SHIPPED

Source: lines 850-882.

- [E] Add FRAMING before Phase-4D recursive decomposition.
- [E] Project class enum: `ConsumerMobileApp`, `WebApp`, `SaaSDashboard`, `DesktopApp`, `GameMod`, `ResearchTool`, `LibraryOrSDK`, `CLITool`, `Other`; use deterministic regex first and LLM only for `Other`.
- [E] `[research_presets.<class>]` contains ordered `ResearchStream`s with `name`, Tera `query_template` using `{idea}`/`{class}`, `min_source_tier` T1/T2, `parallelism_budget`, `cosmos_grounding`, and `mandatory`.
- [E] `ResearchTree::new_multi_root(streams)` dispatches multi-root FlashResearch. Every mandatory stream must converge or appear explicitly in `degraded_streams`.
- [E] Critic categories: `STALE`, `SLOP`, `POLICY`, `FEASIBILITY`, `MARKET`, `SCOPE`, `CONTRADICTION`, `DESIGN_HARD`, `VAGUE`, `MISSING`.
- [E] Critic loop is capped at three rounds. Any unresolved `C-*` finding after the last round emits `idea_framing_block`, blocks, and prevents FRAMING from advancing.
- [E] Events: `idea_framing_started`, `project_class_classified`, `critic_round_complete`, `idea_framed`, `tradeoff_card_emitted`.
- [E] Types include `FramedIdea`, `TradeoffCard`, `CriticFinding`, `CriticCategory`, `ProjectClass`, and `classify_project_class`; FSM adds `FsmSubstate::Framing`.
- [E] `AutoAdvanceEvaluator` adds a seventh framing evidence check using `derive_framing_evidence`.
- [E] UI: `IdeaFramingWindow.svelte` before Workspace and `Critic rounds N/3` in MonitorWindow.
- [E] Artifacts: `workflow/sessions/{sid}/idea-framing/framed-idea.toml`, `critic-rounds/round-N.toml`, `tradeoff-responses.toml`; Designer Concepts reuses them as ground-by-default B-35 artifacts.
- [E] Citation verification corrected one bad research citation. Ledger: schema 41 with a pin-42 interaction against B-34, IPC v2 unchanged, 79/79 tests pass.

### B-41 - Per-level Auditor modes - SHIPPED; navigator spawning deferred

Source: lines 884-918.

- [E] Replace one end-of-plan audit with six scoped modes.
- [E] `Concepts`: after Concepts, check market fit, IP, scope, and framed-idea contradictions at GRC 6/1000.
- [E] `Phases`: after Concept -> Phases, check order, contract sufficiency, and cycles at GRC 5/800.
- [E] `Steps`: after Phase -> Steps, check granularity and test coverage at GRC 4/700.
- [E] `BabySteps`: after Step -> BabySteps, enforce the eight blocks `GROUNDING`, `PRE_GATE`, `TASK`, `TDD`, `DETERMINISTIC_CHECK`, `ACCEPTANCE`, `INTEGRATION`, `EXIT_CRITERIA` at GRC 4/700.
- [E] `BabyStepDetail`: after batches of at most four babysteps, inspect line-level lazy code, unwraps, hard-coded secrets, and stale API versions at GRC 3/600.
- [E] `Plan`: retain whole-plan audit at GRC 10/1200.
- [E] A future `Visual` variant is reserved behind `cfg(feature="visual_audit")` and was described as future B-43.
- [E] BLOCK response: respawn the same-level decomposer once with `rescope_from_audit`; do not re-audit that respawn. If the second-tier audit also blocks, `BubbleUpHandler` escalates to the parent tier.
- [E] Fixed rule-cluster order: anti-stale -> coding-standards -> design-hard -> app-store -> source-tiers -> slopsquatting -> risk-levels.
- [E] Seed rule domains: `design_hard.toml`, `app_store.toml`, `source_tiers.toml`, `slopsquatting.toml`, `risk_levels.toml` with the approximate rule counts stated in the summary.
- [E] Model role as `AgentRole::Auditor(AuditorMode)`, not six role variants.
- [E] `AutoAdvanceEvaluator` adds `concept_zero_blocks`, `phase_zero_blocks`, `step_zero_blocks`, `babystep_zero_blocks`, and `plan_zero_blocks`.
- [E] Events: `per_level_audit_started`, `per_level_audit_verdict`, `per_level_audit_cascade`; UI includes `AuditTimelineWindow.svelte`.
- [E] Artifact path: `workflow/sessions/{sid}/audits/{mode}/{node_id}.md`; ledger is 80 green tests and schema 45.

### B-42 - Cosmos artifact tier - SHIPPED; `spawn_loop` dispatcher deferred

Source: lines 920-961.

- [E] Extend B-35 with a user-scoped, cross-project tier at `~/.elai/cosmos/`, outside all project repositories.
- [E] Storage: `manifest.toml`; `artifacts/*.md` plus `*.toml` carrying blake3 hash and refresh metadata; `index/cosmos_global.bm25`; `cosmos_global.vec`.
- [E] Manifest fields: `name`, consuming B-40 `project_classes`, `refresh_cadence` (`daily|weekly|monthly|quarterly`), `min_source_tier` default T1, `research_query` template, `max_size_bytes`, and per-tier relevance for concepts/phases/steps/babysteps.
- [E] `CosmosWorker.tick()` runs on Layer 1A's default hourly schedule. For each due artifact it constructs a FlashResearch query, waits for convergence, writes Markdown plus blake3, and skips emission when the hash is unchanged.
- [E] At age greater than twice refresh cadence, status becomes `DECAYED`, then `SHADOW`; callers receive WARN and refresh is reprioritized.
- [E] `cosmos_get(name)` lazily blocks for stale refresh for up to thirty seconds, then returns stale content with WARN on timeout.
- [E] `CosmosStage` runs between ArtifactStage and ARagStage using vector + BM25 RRF.
- [E] `GroundingMethod::CosmosCheck` is the eighth method and handles `design`, `ip_boundary`, `monetization`, `app_store`, `marketing_pattern`, and `ux_convention` claims.
- [E] `compile_cosmos_briefing(project_class, tier_relevance_threshold)` emits an `AVAILABLE_COSMOS` block of at most 1,500 tokens. On overflow use ShortCoder or skip. Thresholds: Designer 0.5, idea-framer 0.5, Concepts audit 0.4, Phases audit 0.6, BabySteps skipped.
- [E] At least twelve initial artifacts are required: `design_trends_2026` (mobile and web), `ios_app_store_guidelines`, `android_play_store_guidelines`, `web_app_ux_patterns_2026`, `saas_dashboard_patterns`, `copyright_tradedress_guide`, `asset_licensing_quickref`, consumer-app and SaaS monetization patterns, `color_palette_trends_2026`, `typography_trends_2026`, `accessibility_wcag_aa_quickref`.
- [E] Cosmos shares B-31 `LockScope::Index` because embedding rate limit is shared, and it must not run in AUDITING or EXECUTING.
- [E] Events: `cosmos_refresh_started`, `cosmos_artifact_updated`, `cosmos_artifact_decayed`, `cosmos_grounding_used`.
- [E] UI: CosmosDrawer and a PLANNING/FRAMING mini-badge `cosmos N fresh / Y stale / Z decayed`.
- [E] Ledger: roughly 700-LOC worker plus briefing compiler, `VerificationMethod::CosmosCheck`, schema 44, and CosmosDrawer shipped; scheduled `spawn_loop` dispatch remains deferred.

## Workflows/interactions

### End-to-end telemetry and control surface

- [I, directly composed from B-1/B-2/B-4..B-10/B-36] A real query resolves unified `.elai_cc` data, runs retrieval and GRC producers, journals phase/reasoning/orchestration/decomposition/context events to SSB, and V2 Tauri readers turn those rows into StatusBar, orbit, drawers, Monitor, and Plan/DAG state. This is the summary's concrete realization of its Part-I “functionally wired and visible” objective; the individual links are explicit, while this single sentence is synthesis.
- [E] Phase transition telemetry must be best-effort and cannot stop the FSM, whereas Critical unguarded CSQ and blocked inline feasibility are control decisions that do block or replan (B-2 lines 114-124; B-8 lines 208-215).
- [E] B-5 producer data and B-8 production callers were co-shipped; B-6 ContextBudget visualization and B-10 profile/overlap logic are sibling work (dependency graph lines 1024-1031).
- [E] B-17 is the liveness meta-test over B-2/B-4/B-6/B-8, so an event existing in a schema is insufficient; each named gate must prove it can change verdict under a contract fixture (lines 1029-1033 plus B-17).

### Decomposition, framing, audit, and Cosmos

- [E] The explicit high-level chain is B-40 -> B-41 -> B-42: framing produces scoped context for per-level audits; Cosmos supplies cross-project grounding to both (lines 1046-1048).
- [I] Concrete flow assembled from the sections: classify idea -> run required research preset roots -> perform up to three critic rounds -> persist framed idea/tradeoffs -> recursively decompose -> audit after each level with one same-level rescope opportunity -> bubble a repeated block upward -> inject Cosmos briefing only at the declared tiers. Every step and threshold is explicit; the arrow sequence is synthesis.
- [E] B-7 provides the Concept/Phase/Step/BabyStep event tree and BubbleUp/Failure/Integration surfaces consumed by B-40/B-41-style decomposition; B-35 makes framing artifacts retrievable; B-42 extends B-35 globally (lines 189-206, 874-876, 920-945).
- [E] A mandatory B-40 research stream must either converge or be named degraded; a final unresolved critic blocks FRAMING. A B-41 audit block gets exactly one rescope and no same-tier re-audit, then bubbles to the parent if the next audit also blocks (lines 855-876, 884-915).

### Knowledge/index/search lifecycle

- [E] Dependency order is B-1 -> B-29/B-31/B-35; B-29 -> B-33/B-34; B-30 makes B-29 multi-language; B-29.5 and B-31.5 gate their respective releases; B-35 feeds B-40 and B-42 (lines 1024-1042).
- [I] Indexed-code flow: language-isolated parsing -> symbol/cache shards -> impact/call/context/list tools -> Auditor impact injection -> graph/entry-point consumers. Artifact flow remains a separate per-project collection, then Cosmos adds a separate user-global collection; both can join STANDARD/DEEP retrieval without making FAST non-code-aware. This ordering is a synthesis of explicit B-29/B-30/B-33..B-35/B-42 statements.
- [E] Startup index behavior is state-sensitive: resume `in-progress`, watch `completed`, leave `cancelled`; shutdown cancels and releases, with a hard deadline. Content hashes make resumption skip unchanged files, while a cross-process lease prevents concurrent mutation (B-31 lines 626-642).
- [E] Knowledge errors degrade locally: a failed grammar is isolated and recorded; an artifact error cannot block code indexing; a stale Cosmos fetch can warn and use stale data; B-4 embedding failure goes through a circuit-breaker/degradation path rather than panic (B-4, B-30, B-35, B-42).

### Autonomous execution, runtime routing, cloud, and Focus Tunnel

- [E] B-37 runtime fallback and B-22 model substitution are orthogonal: backend unavailability traverses a runtime chain, while max agent restart traverses a model pool (lines 764-788).
- [E] Exhausted runtime fallbacks escalate in INTERACTIVE mode and halt in AUTONOMOUS mode. Autonomous advancement itself is allowed only after deterministic evidence and still delegates HIGH-risk, missing-evidence, or escalated decisions (B-22/B-37).
- [I] Proposed generation flow with later B items: choose LOCAL/CLOUD/HYBRID per task -> assign role backend -> choose FIM/DIFF/STRUCTURED -> stream deterministic validation -> optionally preassemble N+1 -> resolve a missing dependency through a non-model registry lane -> apply normal evidence gates. The complete chain is inferred; each component and safety rule is explicit in B-27/B-28/B-37.
- [E] Cloud privacy precedes outbound calls, cost is estimated and hard-ceilinged, and air-gapped mode exists. Focus Tunnel forbids the model from invoking package managers; autonomous dependencies must be allowlisted (lines 523-565).

### Session continuity and branch workflow

- [E] At 85% context use, `SessionDag::create_handoff` records a recoverable handoff; checkpoints exist per GRC step/tool/phase; `/continue` loads the handoff; branches support fork/merge/close/list; merge produces `checkpoint` (B-9 lines 217-228).
- [E] B-19 PreCompact is intended to call the same handoff producer, joining context compaction to the SessionDag rather than a no-op (lines 349-355).
- [E] B-26 window layout persistence is deliberately browser localStorage, not SSB and not cross-device; this is UI continuity, distinct from the journaled SessionDag (lines 499-509).

### Sandbox, hook, tool, and self-modification enforcement

- [E] Required sequential hardening order is B-18 -> B-19 -> B-20: establish IPC accept/isolation, converge hooks on IPC, then converge tools on IPC (lines 1032-1034).
- [E] CLI runtimes in B-37 consume that Tier-C lane; stdout is sanitized before it becomes prompt context (lines 764-769).
- [E] Self-modification has a closed mutable-key set, four enforcement layers, independent staged outcomes, regression/oscillation/evolution filters, and auto-revert. The modifying agent cannot read the lock list (B-23 lines 433-458).

### REFLECT chat workflow

- [E] `/reason on` runs TRIAGE -> four independent candidates with reset KV -> capped Auditor refine -> synthesis. Any failure transparently passes through (lines 817-848).
- [E] Current B-39C answer replacement is text-only and intentionally bypasses tools; deferred B-39E would return synthesized reasoning as a system hint to the standard tool loop at one extra call (lines 838-848).

### Media validation workflow

- [E] Renderable output enters Tier 1 deterministic checks first. A Tier-1 veto cannot be overridden by Tier 2; a passing Tier 1 plus advisory disagreement goes to human review; missing VLM degrades rather than fabricates a pass. The RCE loop iterates structured findings until issue identity stops changing (lines 460-481).

## Constraints/invariants

### Harness and evidence boundaries

- [E/context] Global ELAI invariants are off-agent evidence, fail-closed missing evidence, and deterministic time/randomness through injected providers (overview lines 14-27). B concretely preserves those through liveness tests, entropy quarantine, real metrics, non-self-reported checks, and seeded UI timing.
- [E] Telemetry write failure may not block an FSM transition (B-2), but missing/negative gate evidence can block writes, advancement, feasibility, framing, or ambiguity verdicts according to the relevant gate.
- [E] Do not label fake in-process or struct-only tests as isolation/IPC evidence; B-18 explicitly replaces them with OS-observed facts.

### Determinism and data structures

- [E] Use blake3, never blake2/sha2 for non-RFC uses in this plan family (cross-cutting lines 996-998).
- [E] Direct `SystemTime::now`/`Instant::now` is allowed only in `core/src/clock.rs` and tests; inject clock elsewhere (lines 1005-1006).
- [E] Do not use `std::collections::HashMap` in non-test code; use BTreeMap, IndexMap, or FxHashMap according to semantics (lines 1003-1004).
- [E] No `.unwrap()` outside test/main/const initialization (lines 996-1000).
- [E] Deterministic ordering is explicit for C# candidates, hop-band benchmark bytes, graph export, and checkpoint/resume identity.

### Dependencies, runtime, and isolation

- [E] No new external dependency without user confirmation. The summary flags ts-rs, ONNX models, ort changes, graph npm packages, optional fs2, dashboard engine crates, and other B-38/B-42 dependencies (lines 1000-1002). A dependency listed as already shipped does not erase that policy record.
- [E] Tier-C isolation is mandatory for every CLI runtime subprocess (lines 1007-1008).
- [E] Tree-sitter is retained; no ast-grep replacement (lines 1008-1009).
- [E] HARDWARE-HEAVY must be declared before sidecar boot; heavy sidecar control is remote-only, never local, across B-25/B-36/B-37 (lines 1009-1010).
- [E] `PermissionMode` defaults to `AcceptEdits`; unsafe permission skipping is opt-in only (B-37).

### V2/UI invariants

- [E] Svelte 5 runes only; no legacy reactive labels or event dispatcher. Tauri event API is version 2. The orbit is not to be replaced/back-ported during B-36 (lines 49-56, 274-286, 756-758).
- [E] Activity is an orbit, not a text feed. Workspace is the canonical shell. Solid CTAs are excluded in favor of outlined buttons (B-12/B-13/B-14).
- [E] Window state belongs only to localStorage and has no cross-device sync; Cosmos data, conversely, is user-global but stored outside project repositories (B-26/B-42).

### Security/privacy/failure behavior

- [E] Visibility-filtered paths look NotFound and do not reveal policy internals (B-18).
- [E] Context artifact paths cannot escape project root (B-35).
- [E] Empire MCP leaks no internal state, uses ToolFilter, and is feature-gated with graceful degradation (B-25).
- [E] Cloud calls redact configured secrets/paths, carry audit rows, enforce spend ceilings, and are disabled by air-gapped mode (B-27).
- [E] Runtime CLI stdout must pass an IPC sanitizer. Stuck subprocesses are killed on stdout idle timeout (B-37).
- [E] Media Tier 2 is advisory only; deterministic Tier 1 has veto authority (B-24).
- [E] Grounding/artifact/Cosmos failures have typed fallback behavior rather than silent success: `MalformedGrounding`, non-fatal artifact error, stale-with-WARN, or `Degraded` as applicable.

### Scale/performance/resource bounds

- [E] Named bounds include: gate transition <100ms; feasibility stub <50 LOC and 25/30s; branch-and-explore <=3 alternatives; Context split <=4 subtasks/depth 2; fifty babystep autonomy breaker; hook tiers <1/<10/<5ms; Tier-1 media <100ms, max concurrency 4/1; impact depth <=10; graph p99 <=500ms/RSS <=2GB; visualization <=20K nodes/60K edges/200 symbols per file; 2,000 chars/chunk; index batches 50 files; stale lock 2m with 30s refresh; shutdown <=60s; entry points p99 <500ms; Cosmos briefing <=1,500 tokens and refresh block <=30s; REFLECT 85% budget stop (respective B sections).

## Dependencies

### Explicit B-to-B graph

- [E] B-1 unblocks B-3, B-4, B-7, B-29, B-31, B-35.
- [E] B-2 unblocks B-3 StatusBar verification and B-22 AutoAdvanceStats through FSM events.
- [E] B-3's ts-rs is reused by B-4, B-9 `SessionHandoffEntry`, and B-13.
- [E] B-5 and B-8 are paired; B-6 and B-10 are siblings.
- [E] B-11..B-14 were mostly present in V2; verification/drift work remained.
- [E] B-17 is the meta-step over B-2/B-4/B-6/B-8.
- [E] B-18 -> B-19 -> B-20 is strictly sequential.
- [E] B-21..B-25 are major Phase-4F..4J implementations; B-22 enables B-23 and B-26 signals.
- [E] B-29 enables B-33/B-34 and overlaps B-35; B-29.5 gates B-29; B-31.5 gates B-31; B-30 supplies B-29's language breadth.
- [E] B-35 enables artifact-grounded B-40 and is extended by B-42.
- [E] B-36 is the dashboard backend wiring for B-22..B-42 consumers.
- [E] B-37 enables per-role cloud/runtime routing for B-23/B-24/B-25/B-27/B-28.
- [E] B-38 augments B-17 and B-8.
- [E] B-39/B-39C are FSM-independent but reuse B-5/B-8 GRC primitives.
- [E] B-40 -> B-41 -> B-42 forms the framing/audit/global-grounding chain (all from lines 1024-1048).

### Configuration and ownership

- [E] Path constants are owned by B-1's `crates/core/src/paths.rs` (lines 1012-1013).
- [E] `elai_cc.toml` owners: B-22 role backend legacy/config; B-27 `[cloud]`, `[cloud.privacy]`, `[execution]`; B-35 `[[knowledge.artifacts]]`; B-37 `[runtime]` and `[agents.<role>.runtime]`; B-40 `[research_presets.<class>]`. Cosmos owns a separate user-global manifest (lines 1013-1017).
- [E] Every B-N is expected to produce `workflow/reasoning/B-N-decisions.md` (line 1018).
- [E] Metrics artifacts listed by the summary: `phase_3K_rag.toml`, `phase_3K_rag_improved.toml`, `phase_4F.toml`, `phase_4G.toml`, `phase_4A.toml`, `phase_b29_5_*.toml`, `phase_b31_5_*.toml`, `phase_4M.toml` (lines 1019-1022).

### External/system dependencies explicitly named

- [E] UI/runtime: Tauri 2.10, Svelte 5, Vite 5, three.js, troika-three-text, JetBrains Mono package, Lucide, optional cytoscape/dagre/fcose, Playwright, Vitest.
- [E] Retrieval/parsing: ONNX Runtime, cross-encoder-l12, GraphCodeBERT, Granite, tree-sitter grammars, Tantivy/BM25 and vector indexes by context, optional `fs2`.
- [E] System/media: nvml-wrapper, sysinfo, clone3/seccomp/prctl/rlimits or AppContainer/restricted token/Job objects, ImageMagick, Blender, FFmpeg, Tesseract, CLAP, Ollama, SGLang.
- [E] Cloud/runtime/integration: Anthropic-compatible and OpenAI-compatible APIs, SSH remote sidecar, FactoryLine transport, Empire source access, vendored CLI backends.
- [E] Hardware-specific deferred work: B-39B needs a 3090 box; B-24 Tier-2 models have stated ~17GB/~38GB footprints.

### Recommendation and effort ledger

- [E] Recommended knowledge order: B-29, B-31, B-30, B-35, B-32, B-33, B-34, B-29.5, B-31.5 (lines 1050-1059).
- [E] Estimated sessions by part: I 10-15; II 4-6; III 6-10; IV 16-24; V 11-17 plus ~2 multi-hour scale sessions; VI 4; VII 5; VIII ~6; total ~53-78 (lines 1061-1073).
- [E] Milestones: after I+II, visually/functionally correct MVP; after III, production-grade/ship-ready hardening; after IV, autonomous/self-mod/cloud/Focus Tunnel; after V, symbol safety, multi-language, resilient indexing, artifact grounding, and no longer Rust-only in practice (lines 1075-1082).

## Deferred/questions

### Explicit planned/deferred/open work

- [E] Summary snapshot marks shipped: B-1..B-10, V2-re-scoped B-11..B-14, only the `main.rs` decomposition portion of B-15, B-22, B-24, majority B-26, B-36, B-37, B-39/B-39C, B-40, B-41, B-42 (lines 1086-1094).
- [E] Snapshot marks planned: B-16..B-21, B-23, B-25 conditional, B-27..B-35 including .5 gates, B-38, and B-22b (lines 1096-1098).
- [E] Deferred list: B-25 pending Empire codebase access; B-39B pending 3090; B-39D/E/F progress UI, system-hint tool mode, and compaction guard (lines 1100-1102).
- [E] Other open follow-ups: remove B-4 `ELAI_CODE_VEC=1` after full validation; dashboard SessionDag write (~80 LOC); `rule_engine.rs` HashMap -> FxHashMap and `dispatcher.rs` HashMap -> BTreeMap; real-model live UI verification (lines 1104-1109). B-7b research emitter and B-6b agent-exit/live-agent work are recorded DONE in the same block.
- [E] B-15 still has the full deletion/lint/clock/module/script/metrics backlog; B-26 still has six polish tasks; B-41 navigator spawning and B-42 `spawn_loop` dispatch remain open even though their main plans shipped.

### Source ambiguities or decisions that must not be silently resolved

- [E/question] B-33 has three mutually exclusive UI-placement options and only a recommendation, not a final choice (lines 671-676).
- [E/question] B-26 plan progress may be implemented as a new command **or** an extended `FsmSnapshot`; the summary does not choose (lines 516-518).
- [E/question] B-31 allows `fs2` only if already present, otherwise hand-rolled locks; dependency inventory must decide the branch (lines 631-633).
- [E/question] B-25 is called PLANNED in its section and the planned snapshot but DEFERRED/conditional elsewhere. Preserve both: it has a spec, execution is conditional on Empire access.
- [E/question] B-11..B-14 are called shipped as “V2 re-scoped subset DONE,” but their subsections retain audits/backfills; “DONE” does not mean every originally-authored V1 task exists unchanged.
- [E/question] B-40 records schema 41 with a “pin 42 vs B-34,” B-42 records schema 44, and B-41 records schema 45 despite plan-number order. The summary reports these values but does not explain migration ordering; canonical files/code should reconcile.
- [E/question] The cross-cutting no-new-deps line associates cytoscape with B-23, while the actual cytoscape feature is B-33 (lines 1000-1002 versus 669-688). Treat this as a likely summary typo, not an inferred B-23 dependency.
- [E/question] B-41 calls a future visual-audit variant “B-43,” but the current directory contains `B-43_one_line_install.md`. This indicates post-summary identifier drift or reuse and requires the B-43 canonical file/history to resolve.
- [E/question] B-42 intentionally uses `~/.elai/cosmos/`, unlike B-1's project/runtime `.elai_cc` unification. The summary explicitly calls it user-global, so do not normalize this path without checking the canonical B-42 spec.
- [E/question] The summary names only the count, not the identifiers, of B-24's six media SSB variants and B-31.5's three CI contracts. Deeper files are required for atomic names.
- [E/question] Some Tauri command totals in B-7 and B-24 are counts rather than complete name lists; canonical plans/code are required to enumerate every command.

## Deeper-source manifest

### Canonical B plan/index files present

The following are the deeper original files discovered in the B-plan directory. They were **not read for this summary-only pass**.

- `docs/thefinalglance/B-plan/B-0_INDEX.md`
- `docs/thefinalglance/B-plan/B-1_path_unification.md`
- `docs/thefinalglance/B-plan/B-2_fsm_phasetransition_wiring.md`
- `docs/thefinalglance/B-plan/B-3_stub_command_eradication.md`
- `docs/thefinalglance/B-plan/B-4_knowledge_producer_wiring.md`
- `docs/thefinalglance/B-plan/B-5_reasoning_producer_wiring.md`
- `docs/thefinalglance/B-plan/B-6_orchestration_producer_wiring.md`
- `docs/thefinalglance/B-plan/B-7_pipeline_factory_producer_wiring.md`
- `docs/thefinalglance/B-plan/B-8_pipelinegrc_activation.md`
- `docs/thefinalglance/B-plan/B-9_sessiondag_activation.md`
- `docs/thefinalglance/B-plan/B-10_contextbudget_profile_activation.md`
- `docs/thefinalglance/B-plan/B-11_design_system_restoration.md`
- `docs/thefinalglance/B-plan/B-12_shell_vs_workspace_decision.md`
- `docs/thefinalglance/B-plan/B-13_activity_feed_restoration.md`
- `docs/thefinalglance/B-plan/B-14_live_token_streaming_chat.md`
- `docs/thefinalglance/B-plan/B-15_phase0_debt_burndown.md`
- `docs/thefinalglance/B-plan/B-16_anti_stale_enforcement.md`
- `docs/thefinalglance/B-plan/B-17_ground_vacuous_gates.md`
- `docs/thefinalglance/B-plan/B-18_tier_c_sandbox_activation.md`
- `docs/thefinalglance/B-plan/B-19_hook_system_unification.md`
- `docs/thefinalglance/B-plan/B-20_tool_registry_unification.md`
- `docs/thefinalglance/B-plan/B-21_phase_4F_feasibility_probe.md`
- `docs/thefinalglance/B-plan/B-22_phase_4G_autonomous_mode.md`
- `docs/thefinalglance/B-plan/B-23_phase_4H_self_modification.md`
- `docs/thefinalglance/B-plan/B-24_phase_4I_media_validator.md`
- `docs/thefinalglance/B-plan/B-25_phase_4J_empire_integration.md`
- `docs/thefinalglance/B-plan/B-26_phase_4K_6_to_13_dashboard_completion.md`
- `docs/thefinalglance/B-plan/B-27_cloud_mode.md`
- `docs/thefinalglance/B-plan/B-28_focus_tunnel.md`
- `docs/thefinalglance/B-plan/B-29_symbol_level_impact_api.md`
- `docs/thefinalglance/B-plan/B-29.5_symbol_graph_scale_benchmark.md`
- `docs/thefinalglance/B-plan/B-30_multilang_dkg_hardening.md`
- `docs/thefinalglance/B-plan/B-31_resumable_indexing_lock_lifecycle.md`
- `docs/thefinalglance/B-plan/B-31.5_resumable_indexing_scale_benchmark.md`
- `docs/thefinalglance/B-plan/B-32_minified_file_chunking.md`
- `docs/thefinalglance/B-plan/B-33_interactive_graph_viewer.md`
- `docs/thefinalglance/B-plan/B-34_entry_point_detection.md`
- `docs/thefinalglance/B-plan/B-35_context_artifacts_pipeline.md`
- `docs/thefinalglance/B-plan/B-36_dashboard_v2_src_tauri_wiring.md`
- `docs/thefinalglance/B-plan/B-37_multi_runtime_backend.md`
- `docs/thefinalglance/B-plan/B-38_ambiguity_score_gate.md`
- `docs/thefinalglance/B-plan/B-39_chat_reflect_pipeline.md`
- `docs/thefinalglance/B-plan/B-40_idea_framer_research_presets.md`
- `docs/thefinalglance/B-plan/B-41_per_level_auditor_modes.md`
- `docs/thefinalglance/B-plan/B-42_cosmos_artifact_tier.md`
- `docs/thefinalglance/B-plan/B-43_one_line_install.md` - present now but not covered by the 2026-05-17 summary; high-priority post-summary candidate.

### Referenced specifications, decisions, and handoffs located

- `docs/specs/guide/steps/phase-index-MASTER.txt` - core phase source of truth/global dependency context.
- `docs/specs/guide/dashboard-design.txt` - B-11 token/design source.
- `docs/specs/guide/token-efficiency-design.txt` - B-10 slot-weight source.
- `.elai_cc/crates/dashboard-v2/DESIGN.md` - V2 hover/orbit/transition rules cited by the summary.
- `.elai_cc/crates/dashboard-v2/DOMAIN-CONTEXT.md` - V2 activity/command semantic source cited by the summary.
- `conductor/elai-repo/docs/specs/track-d-v2-spec.md` - B-36 Track-D spec.
- `conductor/elai-repo/docs/thefinalglance/B-plan-delta-2026-05-07.md` - B-36 V2 delta.
- `workflow/sessions/s-46448/HANDOFF-track-d-v2-rebaseline.md` - B-36 session split/handoff.
- `workflow/reasoning/E-34-decisions.md` - B-16 ort rule amendment context.
- `CLAUDE.md` - repository dependency/hash/tree-sitter rules cited repeatedly.

### Relevant implementation and evidence directories/files named by the summary

- `crates/core/src/paths.rs` (or active worktree equivalent) - unified path authority.
- `crates/dashboard-v2/` and `.elai_cc/crates/dashboard-v2/` - locate which is the active V2 implementation before auditing; the summary uses the former logical path while located design files are under the latter.
- `crates/.bkp_old/dashboard/` - parked V1 source, useful only to understand mappings; do not back-port wholesale.
- `workflow/reasoning/` - expected B-N decision records.
- `workflow/sessions/` - framing, audit, handoff, and session-ledger artifacts.
- `.elai_cc/metrics/` - phase and benchmark metrics named above.
- `.elai_cc/models/onnx/` - B-4 model staging/checksum surface.
- `crates/rules/seed/` - B-41 rule-domain seeds, subject to active-worktree path resolution.
- `~/.elai/cosmos/` - runtime user-global Cosmos store; intentionally outside the project and not inspected.

