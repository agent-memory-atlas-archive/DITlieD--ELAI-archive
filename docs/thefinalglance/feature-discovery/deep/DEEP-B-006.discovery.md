# DEEP-B-006 exhaustive feature discovery

## Scope and evidence protocol

- The exact DEEP-B-006 manifest section was read at DEEP-SOURCE-MANIFEST.md lines 337-350. Only its eight numbered/index originals were deep-read. The summary discovery at 02-B-PLAN-SUMMARY.discovery.md was used only as an orientation and delta target.
- Evidence labels: **[E]** is stated by the named source; **[I]** is a conservative cross-source inference; **[A]** is an explicitly unresolved choice or internally ambiguous claim; **[C]** is a conflict between source statements, status overlays, or the orientation summary. A source saying that work shipped is recorded as a documentary status claim, not independently verified implementation evidence.
- An atom is one independently testable capability, behavior, contract, status, constraint, or decision. Repeated task prose, wiring-contract restatements, checks, and acceptance rows are merged into the originating atom and cross-referenced so they are not double-counted.

## Exact processed manifest and fingerprint verification

| Ordinal | Manifest class | Exact source | Lines | Bytes | SHA-256 | Verification |
|---:|---|---|---:|---:|---|---|
| 1 | numbered/index original | docs/thefinalglance/B-plan/B-7_pipeline_factory_producer_wiring.md | 399 | 20912 | e31ede6a8206ac925d2f3789a856d88f8934555326d772c593a3327f2f5c8cf6 | exact |
| 2 | numbered/index original | docs/thefinalglance/B-plan/B-16_anti_stale_enforcement.md | 280 | 15101 | f66edf4ddfc48738ed96c0f92ef962e078365461953c577154161bd6a2190330 | exact |
| 3 | numbered/index original | docs/thefinalglance/B-plan/B-18_tier_c_sandbox_activation.md | 203 | 8241 | bd27c1eacb49038ddcb18020892a1bcb6d98a0a74d0b041160cbaa54788e27fc | exact |
| 4 | numbered/index original | docs/thefinalglance/B-plan/B-19_hook_system_unification.md | 254 | 10282 | f22a1c5e0d8bc689a6cbc53e199e9c768e75318ae44e074112e1a6461e4e03e8 | exact |
| 5 | numbered/index original | docs/thefinalglance/B-plan/B-21_phase_4F_feasibility_probe.md | 166 | 7290 | a4299a9271636b523daaf981f97b4a7b85a17ee8a34eca209510693e3adc3344 | exact |
| 6 | numbered/index original | docs/thefinalglance/B-plan/B-23_phase_4H_self_modification.md | 227 | 10070 | 387d2c7969892307a913f3ebdc90baf34d75bd2c5874bf394fb10f105d110c8a | exact |
| 7 | numbered/index original | docs/thefinalglance/B-plan/B-29.5_symbol_graph_scale_benchmark.md | 461 | 19054 | 4e0f9850ed73e607d64693d0a574b4b0f5538882a100170ffb407e2db0bbbad9 | exact |
| 8 | numbered/index original | docs/thefinalglance/B-plan/B-35_context_artifacts_pipeline.md | 596 | 28046 | 71ebbb6cfba20afbe817dc9187fe083025de5a2d456a545e40073b70630e9fe7 | exact |
| **Total** | **8 originals** | | **2586** | **118996** | manifest aggregate | exact |

Verification used logical newline counts, byte counts, and SHA-256 over the exact files. No fingerprint drift was found.

## Per-file atomic feature ledgers

### B-7 — Pipeline Factory producer wiring (4D + 4E)

Source: B-7_pipeline_factory_producer_wiring.md, lines 1-399.

- **B7-001 [E] Scope/status:** The objective is to visualize DecompositionTree, ExecutablePipeline, DAG repair, per-node web research, and successful IntegrationCheckpoint activity; activate PlanWindow; and replace three empty detail/tree readers. B-1 is required and B-4/B-5/B-6 are parallel-safe (lines 1-8). The 2026-05-22 V2 overlay calls the producer-wire surface GREEN, while retaining explicit follow-ups (lines 10-13, 99-105).
- **B7-002 [E] PlanWindow surface:** Task 1 shipped on V2 by importing and directly rendering PlanWindow in WindowHost.svelte; V2 has no WindowRegistry indirection (lines 15-19, 117-129, 176-179).
- **B7-003 [E] Decomposition producer:** Task 2 shipped as one ConceptDecompositionNode payload and SsbDecompositionEmitter implementing DecompositionTreeEmitter, fired for every child pushed by decompose (lines 20-24). The final contract intentionally uses one concept_decomposition_node action with node_kind = concept | phase | step | baby_step instead of four action variants (lines 181-188).
- **B7-004 [E] Tree reader:** get_decomposition_tree(plan_id) now reads viewer.decomposition_nodes and assembles a real DecompositionTreeSnapshot rather than returning a default (lines 25-26, 190-192, 274, 289-293).
- **B7-005 [E] Detail readers:** get_phase_detail and get_baby_step_detail are real SSB reads rather than NotFound stubs (lines 27-29, 194-196, 275-276, 292-293).
- **B7-006 [E] Structured plan UI:** PlanWindow renders Concept, Phase, Step, and BabyStep nodes plus the eight BabyStep blocks GROUNDING, PRE_GATE, TASK, TDD, DETERMINISTIC_CHECK, ACCEPTANCE, INTEGRATION, and EXIT_CRITERIA (lines 30-32, 150-155). The original target also specifies per-node PASS/PENDING/FAILED progress and click-through detail (lines 198-201).
- **B7-007 [E] Dual plan visualization:** ActivityWindow PLAN mode is the constellation/canvas view and PlanWindow is the structured-tree view; both consume the same producer output (lines 150-153).
- **B7-008 [E] DAG-repair history:** get_dag_repair_log shipped and reads the most recent 100 repair rounds (lines 33-35). The task contract records dag_repair_round with round_index, Reorder|Substitute, target_node, and success, plus dag_repair_escalated when T_DAG is exhausted, surfaced in OrchestrationDrawer or RepairLogPanel (lines 203-206).
- **B7-009 [E] Executable pipeline:** get_executable_pipeline shipped (line 36). Its planned response is the full DAG plus ValidationErrorKind history and repair_log, rendered with d3 or cytoscape.js in PlanWindow (lines 208-211).
- **B7-010 [E] Tree cursor:** tree_position_changed shipped through a navigator emitter after every record_result cursor mutation (lines 37-42). Its payload carries current_node, visited, blocked, and skipped; PlanWindow highlights the current node (lines 218-220).
- **B7-011 [C] FailureRecord shape correction:** failure_recorded shipped once per FailureRecord upsert in handle_failure. The live payload mirrors reason/attempts, explicitly not the originally planned failure_kind/retry_count shape at lines 222-224 (lines 43-49).
- **B7-012 [E] Typed bubble-up:** SsbBubbleUpEmitter shipped (lines 50-51). bubble_up_typed carries from_level, to_level, reason, and one of ResearchAlternative, RedecomposeStep, ReplanPhase, ReplanConcept, or Escalate; the UI shows BabyStep → Step → Phase → Concept cascade arrows (lines 226-230).
- **B7-013 [E] Successful checkpoints:** SsbIntegrationEmitter shipped (lines 52-53). integration_checkpoint_success is emitted at BabyStep, Step, Phase, and Concept levels with timing, producing a green check trail (lines 232-234).
- **B7-014 [C] Research-stage action reuse:** Per-node FORMULATE, SEARCH, EXTRACT, REFLECT, and CONVERGE stages shipped through the existing WebResearchStep action with populated stage, not a fabricated ResearchNodeStage action. research_node_stages(node_id, limit) and the rewired get_research_node_history expose them (lines 54-62). This supersedes the original new-action wording while retaining node_id, duration_ms, success, and the ResearchDrawer timeline behavior (lines 236-241).
- **B7-015 [E] Package verification substrate/deferment:** SsbPackageRefVerifiedEmitter and verify_packages_with_emitter exist and are unit-tested, but producer data flow is wire-dark because ConceptDecomposition and PipelineSpec contain no PackageRef field; B-7-followup owns it (lines 63-79). The live shape mirrors PackageRef and does not fabricate the originally proposed ShapeOnly|RegistryHttp method enum (lines 77-79, 243-246).
- **B7-016 [C] Tech-stack substrate/deferment:** SsbTechStackDecidedEmitter and new_tech_stack_decision exist and are unit-tested, but the producer is wire-dark because decomposition structures contain no tech_stack field; B-7-followup owns it (lines 80-92). source is the snake_case DecisionSource discriminant research|constraint|default, not the original RESEARCH|EXISTING_PROJECT|USER_SPECIFIED|FALLBACK literals (lines 93-97, 248-251).
- **B7-017 [E] Phase sizing follow-up:** phase_sized_split remains deferred; it must wire PhaseSizingSplitter call sites, emit original_phase_id, new_phase_ids, and complexity_reason when a phase exceeds available_for_work × 0.6, and show a split badge (lines 99-101, 213-216, 363-369).
- **B7-018 [E] Interactive alternative follow-up:** The decomposition_alternative panel remains deferred; Designer alternatives become an intervention card requiring a user selection (lines 102-103, 253-255).
- **B7-019 [E] Research cache follow-up:** A session ResearchCache hit-rate gauge remains deferred (lines 104, 257).
- **B7-020 [E] Research metric-reader follow-up:** Real get_research_budget and get_source_tier_distribution reads remain deferred (lines 105, 259-260, 281-282).
- **B7-021 [E] Schema/evidence lifecycle:** The session advances SCHEMA_VERSION 99 → 100 for TreePositionChanged, FailureRecorded, PackageRefVerified, and TechStackDecided and advances the typed-payload bijection from 232 to 236 (lines 107-109, 313-316).
- **B7-022 [E] V2 surface rebaseline:** Canonical targets are dashboard-v2 src-tauri commands, WindowHost/AppShell routing, PlanWindow, ResearchDrawer, and OrchestrationDrawer; the named V1 command, registry, and drawer paths are retired (lines 111-149).
- **B7-023 [E] Cold-start fallback:** PlanWindow keeps stub.ts cold-start fallbacks for get_decomposition_tree, get_executable_pipeline, get_dag_repair_log, get_research_tree, get_bubble_up_events, and get_package_verifications (lines 126-137). These are fallback data, not permission to return production defaults.
- **B7-024 [E] Wiring surface:** Producers are SsbDecompositionEmitter, SsbPackageRefVerifiedEmitter, SsbTechStackDecidedEmitter, SsbDagRepairEmitter, SsbBubbleUpEmitter, SsbIntegrationEmitter, SsbFailureRecordedEmitter, SsbTreePositionChangedEmitter, WebResearchStepEmitter, and the nine named readers in lines 274-282; consumers are PlanWindow, ResearchDrawer, and OrchestrationDrawer (lines 262-286).
- **B7-025 [E] Reachability and observation:** The producer path must be reachable from task_runner::drive_task and get_decomposition_tree; a bounded local test task must produce concept_decomposition_node and a drillable real PlanWindow tree (lines 287-299).
- **B7-026 [E] Deterministic/done gates:** Static mount and no-default checks, the typed-payload unit test, dashboard npm check with zero errors, and a 60-second tree/package smoke are the prescribed checks (lines 301-325). Acceptance separately calls for a visible repair log, package badges, research timeline, bubble cascade, and current-node highlight (lines 327-336).
- **B7-027 [C] Acceptance/status mismatch:** Every acceptance checkbox remains unchecked, including “14 new SSB action_types” and “9 new/replaced Tauri commands,” although the V2 status overlay marks many tasks shipped and explicitly consolidates/reuses actions (lines 10-109, 327-336). The status overlay is therefore more precise than the stale aggregate counts.
- **B7-028 [A] DAG-render dependency:** d3 or cytoscape.js is proposed, but adding the dashboard graph dependency requires user confirmation and no choice is finalized (lines 208-211, 338-353).
- **B7-029 [E] Grounded repair bound:** T_DAG = 10 is the named maximum repair-round constant before escalation; the plan forbids magic-number drift (lines 355-361).
- **B7-030 [E] Grounded split bound:** SPLIT_RATIO = 0.6 is already defined/consumed; only its event emitter remains unwired (lines 363-369).
- **B7-031 [E] Backward-compatible TaskSplit evolution:** Existing TaskSplitPayload is extended, not forked, with phase_id: Option<u64> and complexity_reason: Option<String>; serde defaults preserve pre-patch MessagePack rows and one payload serves supervision and PhaseSizingSplitter consumers (lines 371-389).
- **B7-032 [E] Producer exclusivity invariant:** A TaskSplitPayload producer must populate either existing agent_role or new phase_id, never both (lines 390-392).
- **B7-033 [E] Lifecycle/dependencies:** B-8 is next; B-7-follow-up owns phase sizing, alternatives, cache rate, research budget, and source-tier distribution (lines 394-399).

### B-16 — Anti-Stale Rule Enforcement

Source: B-16_anti_stale_enforcement.md, lines 1-280.

- **B16-001 [E] Scope/dependencies:** The plan enforces Rust anti-stale rules over ort, typed errors, hashing, unwrap/expect, tracing, constants, and narrowing casts; it has no prerequisite and is parallel-safe with all B plans (lines 1-8).
- **B16-002 [E] Status vocabulary:** The 2026-05-22 rebaseline distinguishes SHIPPED-ON-V2, SHIPPED-ON-DIFFERENT-SURFACE, PENDING-UPSTREAM, and ORT-PENDING and points to decision/follow-up records (lines 10-18).
- **B16-003 [E] ort upstream gate:** ort 2.0.0 stable was still unpublished, so knowledge and orchestrator remain pinned to =2.0.0-rc.12. A documented waiver makes ORT_FAIL = 0 while DECISION-A exists; a follow-up checklist triggers the bump when stable ships (lines 22-24, 65-77, 228-231).
- **B16-004 [E] Unified typed-error design:** KnowledgeError is a 12-variant thiserror enum with from bridges, supplemented by narrower VectorError at the EmbedderProvider boundary; this deliberately replaces the original per-module split (lines 25-26, 79-87).
- **B16-005 [E] BM25 migration:** Eight originally boxed BM25 APIs plus the named internal/free functions now return KnowledgeError; stale original line citations are superseded and the file has zero boxed-error hits (lines 89-95).
- **B16-006 [E] Vector migration:** Vector indexing/search surfaces were converted to KnowledgeError; remaining boxed closures/futures are non-error trait objects (lines 97-103).
- **B16-007 [E] PageRank migration:** DkgSnapshot::from_engine now returns KnowledgeError and the module has no boxed-error hit (lines 105-108).
- **B16-008 [E] Crate-wide boxed-error gate:** The knowledge crate sweep finds no Box<dyn Error> public API; only a historical doc comment and non-error trait objects remain (lines 110-113).
- **B16-009 [E] Production unwrap/expect policy:** Roughly 67 sites shipped through B-16B chunks. LazyLock<Regex> literal static initialization may use unwrap only with a targeted Clippy allow plus CLAUDE-md-EXCEPTION comment; file-aware audit reports zero unjustified production sites even though raw lint counts include tests/benches/exempt statics (lines 115-127).
- **B16-010 [E] Semaphore recovery:** The reranker semaphore expect was replaced with a graceful match; the old “semaphore closed” expect body is absent (lines 27-28, 129-132).
- **B16-011 [E] Tracing gate:** use log:: has zero crate hits and STRICT lint prevents regression (lines 33-38, 134-138).
- **B16-012 [E] Magic-number analysis:** Rust/Python tree-sitter AST checks in plugin tier 2 enforce named constants, allow only 0, 1, 2, -1, 0.0, and 1.0 plus const/static/test contexts, and are exposed to CI through a REPORT block rather than implemented as a bash grep (lines 140-154).
- **B16-013 [E] Narrowing-cast policy:** B-16E classifies CAT-A/B/C/D casts; 15 production sites were annotated or converted, CAT-D is zero, CAT-C has 35 mostly out-of-scope/test residuals, and migration is REPORT-only with per-site judgment rather than blanket replacement (lines 156-165).
- **B16-014 [E] blake3 spec amendment:** phase-3C now explicitly supersedes blake2s with blake3 for the non-RFC role (lines 30-31, 167-173).
- **B16-015 [E] Anti-staleness spec location correction:** anti-staleness-enforcement.txt exists under docs/specs/guide, not guide/steps, and explicitly bans blake2 in favor of blake3 (lines 175-180).
- **B16-016 [E] CI lint behavior:** check_anti_stale.sh and .bat plus the Windows workflow are shipped. STRICT gates boxed errors, log imports, and ort RC subject to waiver; REPORT covers unwraps, cast categories, and magic numbers. Exit 0 means STRICT clean, not zero REPORT findings (lines 182-191, 249-251).
- **B16-017 [E] Deferred rule expansion:** rand >=0.9, getrandom >=0.3, native async trait/no async-trait, deadpool/bb8 over r2d2, globset/ignore over glob, and async-context mutex analysis are deferred to B-16-lint-extension (lines 33-45, 193-200).
- **B16-018 [E] Wiring contract:** Produced symbols are KnowledgeError, VectorError, and check_anti_stale.sh; all knowledge callers and production unwrap/expect sites consume the signature/policy changes; cargo build and CI lint are reachability gates (lines 202-224).
- **B16-019 [C] Stub-ban qualification:** The contract says zero boxed public errors and log imports under STRICT; zero unjustified unwrap/expect is file-aware; narrowing casts are per-site with CAT-D = 0; and stable ort is explicitly waived pending upstream. Reading the “zero” phrases without these qualifiers would contradict lines 117-165 and 215-219.
- **B16-020 [E] Deterministic checks:** The prescribed evidence is the pinned ort line plus waiver, zero boxed errors in non-test knowledge source, a file-aware unwrap audit, zero log imports, and a clean lint exit (lines 226-252).
- **B16-021 [E] Acceptance:** All acceptance rows are MET except stable ort, which is PENDING-UPSTREAM; tests are claimed to remain passing after refactor (lines 254-263).
- **B16-022 [E] Artifact lifecycle:** The plan records shipped knowledge refactors, about 50 files of unwrap/cast work, two amended specs, two lint scripts, CI wiring, a cosmetic media-validator change, decision records, and a regression/follow-up record; Cargo ort update remains deferred (lines 265-276).
- **B16-023 [A] Missing historical reads:** The original aggregate and outdated-tech audit were missing; the rebaseline substitutes extant authoritative sources (lines 47-61). Claims in this file are therefore grounded in the replacement list rather than those historical audits.
- **B16-024 [E] Next/status:** B-17 follows (lines 278-280). The document itself represents B-16 as largely shipped with one upstream gate and a future lint extension, not a wholly planned plan.

### B-18 — Tier C Sandbox Activation

Source: B-18_tier_c_sandbox_activation.md, lines 1-203.

- **B18-001 [E] Scope/status:** The target is real Tier-C agent-process isolation plus non-disclosing visibility-filter semantics; B-1, B-2, and B-15 are prerequisites and B-19/B-20 are declared parallel-safe (lines 1-8). This file has no shipped overlay and every acceptance row remains unchecked (lines 169-179).
- **B18-002 [E] Baseline gap:** Existing execution uses a separate process but plain Command::spawn with shared resources; effective isolation is called A++, not Tier C (lines 10-25).
- **B18-003 [E] Dormant Windows implementation:** WindowsAppContainerSandbox already contains Win32 calls, but enable_sandbox defaults false and launch_agent_process never calls sandbox.launch_agent (lines 16-18).
- **B18-004 [E] Missing IPC runtime:** IpcGateway types exist without an orchestrator UnixListener/NamedPipeServer accept loop (lines 19-20).
- **B18-005 [E] Linux namespace spawn:** Linux launch must use clone3 with CLONE_NEWNS, CLONE_NEWPID, and CLONE_NEWNET, exposing only a per-agent temporary root with project source and minimal libraries/tools, hiding other agents, and denying network by default (lines 40-49).
- **B18-006 [E] Researcher network exception:** The Researcher receives a dedicated bridged network namespace; its exact bridge configuration is left to the decision record (lines 48-49, 195-200).
- **B18-007 [E] Seccomp profiles:** IPC_BASELINE allows required file/IPC/runtime syscalls while denying execution, socket, and network calls; NETWORK_EXTRA permits AF_INET for the Researcher (lines 51-53).
- **B18-008 [E] Privilege prevention:** PR_SET_NO_NEW_PRIVS blocks setuid privilege escalation and all Linux capabilities are dropped from the bounding set after clone (lines 55-56, 64-65).
- **B18-009 [E] Resource bounds:** Each agent receives CPU limits of 60 seconds soft/90 seconds hard, 2GB memory, 256 files, and 32 processes (lines 58-59).
- **B18-010 [E] Process privacy:** The agent mount namespace mounts /proc with hidepid=2 so process information for other agents is unavailable (lines 61-62).
- **B18-011 [E] Windows default:** enable_sandbox must default true at the three named production entry points (lines 67-70).
- **B18-012 [E] Windows launch path:** launch_agent_process must call self.sandbox.launch_agent and create an AppContainer profile per agent_id (lines 72-76).
- **B18-013 [E] Windows recovery fallback:** If AppContainer creation fails on older Windows, use the existing, explicitly weaker restricted-token plus Job-object fallback (lines 78-79).
- **B18-014 [E] IPC endpoint lifecycle:** Orchestrator binds /tmp/elai-{session_id}.sock on Linux or the elai-{session_id} named pipe on Windows, agents connect, and IpcGateway routes frames (lines 81-85).
- **B18-015 [E] Hook interception:** IpcHookDispatcher must become reachable through IPC frame interception rather than remain dead code (lines 87-88).
- **B18-016 [E] Non-disclosing file errors:** A hidden path must return NotFound with “No such file or directory” or the platform equivalent, indistinguishable from a genuinely absent path; error_code = visibility_filtered is forbidden because it leaks filtering (lines 90-96).
- **B18-017 [E] Per-agent authorization:** AgentHandle gains allowed_paths: Vec<GlobPattern>; each role has its own glob list instead of relying only on global HIDDEN_PATH_PATTERNS (lines 98-100).
- **B18-018 [E] Self-mod protection:** Self-modification uses four gates—IPC, filesystem, git, and dispatch—and Auditor findings may reach PROPOSED only (lines 102-105).
- **B18-019 [E] OS-evidenced tests:** Isolation tests must spawn a real agent and inspect /proc status NSpid on Linux or NtQueryInformationProcess on Windows; constructing a struct and checking a field is expressly rejected as fake evidence (lines 107-113).
- **B18-020 [E] Real metrics:** phase_4A.toml must measure socket/pipe round-trip latency, steady-state agent RSS, and cold spawn-to-first-handshake latency, replacing postcard serialization, struct size, and HashMap insertion proxies (lines 115-121).
- **B18-021 [E] Wiring/reachability:** Produced surfaces are clone3, seccomp loading, invoked Windows launch, the IPC accept loop, and per-agent allowed_paths; consumers are IpcGateway, IpcHookDispatcher, and the visibility filter; both dashboard submit and CLI spawn must reach them (lines 123-147).
- **B18-022 [E] Production bans:** Production enable_sandbox=false, struct-only isolation tests, visibility_filtered errors, and proxy phase_4A metrics are prohibited (lines 138-142).
- **B18-023 [E] Observable proof:** A dashboard agent spawn must yield an OS fact: a distinct NSpid relationship on Linux or an AppContainer SID on Windows (lines 143-147).
- **B18-024 [E] Done gates:** Static checks require no false sandbox default, clone3/seccomp symbols, at least one IPC accept loop, and zero visibility_filtered text in tools; acceptance additionally requires both OS paths, live hook dispatch, allowed paths, real metrics, real cross-platform tests, and a dashboard sandbox badge (lines 149-179).
- **B18-025 [A] Dependency decisions:** nix and windows-sys may be added only with confirmation; clone3 versus unshare-then-exec, Linux-only versus workspace-wide nix, and Researcher bridging are explicitly unresolved (lines 181-200).
- **B18-026 [E] Lifecycle:** Planned file work spans agent_pool, sandbox, ipc_gateway/listener, dashboard/task-runner defaults, tools visibility, integration tests, phase_4A metrics, and B-18-decisions; B-19 is next (lines 181-203).

### B-19 — Hook System Unification

Source: B-19_hook_system_unification.md, lines 1-254.

- **B19-001 [C] Objective rebaseline:** The original objective says choose one dispatcher and delete the other, but the 2026-05-23 decision keeps both implementations behind the single HookDispatcher trait because InProcessHookDispatcher owns shipped behavior (lines 1-9, 43-50). Trait-level unification supersedes implementation deletion.
- **B19-002 [E] Prerequisite status claim:** This file claims B-18 run_agent_session IPC handling and B-9 SessionDag::create_handoff have shipped, unblocking tasks 1/3/5 (lines 17-20). That status is external to B-18’s own unchecked plan.
- **B19-003 [E] Unified async surface:** HookDispatcher has four hook methods; IpcHookAsyncAdapter bridges synchronous IpcHookTrait into the async trait, and Send+Sync supertraits preserve object-safe cross-task adapter chains (lines 22-23, 34, 39-41).
- **B19-004 [E] Runtime construction:** build_production_ipc_hook_dispatcher creates a non-test IpcHookDispatcher, with RuntimeSsbFindingWriter adapting Arc<Ssb> (lines 37-40, 58-62).
- **B19-005 [E] Default implementation policy:** RuntimeWiring call sites may opt into IpcHookAsyncAdapter, but InProcessHookDispatcher remains the production default; deleting it or changing the default is deferred by DECISION-A (lines 43-50).
- **B19-006 [E] Hook call-site topology:** PreToolUse runs on the natural-language task summary, PostToolUse runs per changed file, Stop is state-enforcing, and PreCompact sits at the named TaskRunner locations; old line references are stale by about 2,289 lines (lines 11-15, 63-73).
- **B19-007 [E] Stop enforcement:** Stop outside IDLE blocks termination. Ctrl+C/SIGTERM fires the hook, prompts “Task running, abort? [y/N]”, aborts and cleans up on yes, and ignores the signal on no (lines 24, 69-70, 112-120).
- **B19-008 [E] PreCompact handoff:** dispatch_pre_compact fires above 0.85 token-budget usage and invokes PreCompactHandoffSaver/SessionDag::create_handoff before truncation; the former no-op is closed (lines 35-36, 71-73, 122-126).
- **B19-009 [E] Ruff PostToolUse:** Python writes run a ruff subprocess; findings become retry feedback, the subprocess has a five-second timeout, and results are cached by file-content hash (lines 27, 67-68, 128-133).
- **B19-010 [E] DAIRA vocabulary:** DAIRA_MARKERS contains exactly call, return, mutation, branch, error, finding, read, write, tool_call_start, tool_call_end, hook_pre, hook_post, timeout, escalate, and converge, replacing the two-marker mode (lines 25, 74-76, 135-152).
- **B19-011 [E] AdaptiveTool:** AdaptiveTool<T> wraps arbitrary tools to emit DAIRA markers and reduce per-tool boilerplate (lines 26, 75, 154-155).
- **B19-012 [E] MarkerReport evidence:** MarkerReport aggregates one agent execution’s DAIRA markers in SSB; the shipped action advances SCHEMA_VERSION 100 → 101 and the golden lock 288 → 298, including nine pre-existing missing variants (lines 31-33, 76, 157-158).
- **B19-013 [E] Hook latency budgets:** T1 <1ms, T2 <10ms, and T3 <5ms are enforced; overruns emit HookLatencyViolation/hook_latency_violation rows (lines 28, 160-164).
- **B19-014 [E] Circular-trust gate:** Auditor-derived rules require an independent second Auditor or human approval before ACTIVE promotion; the lifecycle gate is claimed shipped (lines 29, 166-172).
- **B19-015 [C] Hash-spec drift:** phase-4C says blake2s while ipc_hooks uses blake3; the source treats the spec as stale and retains blake3 under the anti-stale rule (lines 52-54).
- **B19-016 [E] Failure-taxonomy boundary:** The current 13-versus-9 category mismatch is explicitly owned by B-20, not B-19 (lines 77).
- **B19-017 [C] Original dispatcher recommendation:** The task body recommends IpcHookDispatcher after B-18 and deletion of InProcessHookDispatcher, with in-process retention only as an alternative (lines 88-110); DECISION-A later reverses deletion/default migration (lines 43-50).
- **B19-018 [E] Wiring contract:** Produced symbols are IpcHookDispatcher, DAIRA_MARKERS, MarkerReport, RuffPostToolUseRunner, dispatch_stop, create_handoff, and HookLatencyViolation; consumers include TaskRunner hook sites plus dashboard per-event filters and latency histograms (lines 174-187).
- **B19-019 [E] Observability:** A dashboard-submitted agent path and hook_smoke_ssb contract should produce hook_event rows and a ToolsHooksDrawer distribution across all four hook types (lines 188-200).
- **B19-020 [C] Stale stub bans:** The plan-body bans InProcessHookDispatcher and requires IPC-only production, but the status overlay intentionally keeps and defaults the in-process implementation (lines 190-194 versus 43-50). The no-op PreCompact, two-marker DAIRA, and telemetry-only Stop bans remain valid.
- **B19-021 [C] Stale deterministic gate:** The prescribed zero-hit search for InProcessHookDispatcher cannot pass under DECISION-A; the non-test IPC caller, 15-marker, and ruff checks remain relevant (lines 202-220).
- **B19-022 [C] Acceptance status:** All acceptance boxes remain unchecked even though the overlay marks Stop, PreCompact, Ruff, DAIRA, AdaptiveTool, latency budgets, circular trust, MarkerReport, adapters, and runtime construction shipped; dispatcher deletion and default migration are intentionally deferred (lines 22-50, 222-232).
- **B19-023 [E] Artifact lifecycle:** The original delete/modify list is partially stale because hook_dispatcher.rs is retained; current work spans TaskRunner, IPC hooks, post-tool-use, FSM/session DAG, tools dispatch, rules, and B-19-decisions (lines 234-244).
- **B19-024 [A] Runtime dependency choice:** The decision record must specify whether ruff is a required dependency or runtime-detected and document the AdaptiveTool design; dispatcher choice has been resolved by DECISION-A (lines 246-250).
- **B19-025 [E] Lifecycle:** B-20 tool-registry unification is next (lines 252-254).

### B-21 — Phase 4F Feasibility Probe

Source: B-21_phase_4F_feasibility_probe.md, lines 1-166.

- **B21-001 [E] Scope/dependencies:** Phase 4F combines real GPU/RAM/disk probing, sub-50-line Tier-C novelty stubs, and SSB branch-and-explore capped at three alternatives. It requires B-18 and visible B-7 decomposition and is parallel-safe with B-22 through B-28 (lines 1-8).
- **B21-002 [E] Granularity:** The 4F.7-4F.8 extension runs at BabyStep level, detects novelty from unverified PackageRef/unused framework/untested capability, and sequences feasibility per concept rather than globally (lines 12-20).
- **B21-003 [E] Type contract:** Base types are SystemProbeResult, GpuInfo, SystemConstraints, StubResult, ResourceDelta, ConstraintResult, FeasibilityReport, FeasibilityEscalation, and FeasibilityDecision; FeasibilityProbe is object-safe and takes &self (lines 23-27).
- **B21-004 [E] Baby-step types:** BabyStepFeasibilityReport, BabyStepStubResult, and BabyStepEscalation extend the base contract (lines 19-20, 90-92).
- **B21-005 [E] System telemetry:** SystemProbe uses nvml-wrapper for total/free VRAM per GPU and sysinfo 0.37+ for RAM/disk; constraint evaluation is pure (lines 29, 43-46).
- **B21-006 [E] Stub isolation:** Stubs are strictly under 50 lines, execute through the Tier-C agent process with FEASIBILITY restrictions, allow 25 seconds graceful completion, and are explicitly killed at 30 seconds because Tokio kill_on_drop is false (lines 30-31, 51-53).
- **B21-007 [E] Dispatch overrides:** Feasibility AgentDispatchConfig uses GRC budget 3/600, tool_filter [stub_write, bash], 25/30-second timeouts, and treats DispatchStage.agent rather than AgentRole alone as authoritative (lines 55-59).
- **B21-008 [E] Skip predicate:** Probing is skipped only when novel_steps is empty and every free resource is at least required × 1.20; both conditions are required (lines 33, 61-64).
- **B21-009 [E] Branch-and-explore ordering:** At most three alternatives are tested per novel step immediately before its concept executes, allowing later concepts to change based on earlier results (lines 32, 65-68).
- **B21-010 [E] Escalation recovery:** Three failures produce Escalate in INTERACTIVE mode or Replan in AUTONOMOUS mode (line 34).
- **B21-011 [E] Persistence:** FeasibilityReport is stored as stub_results.msgpack (line 70).
- **B21-012 [E] Dashboard surface:** FeasibilityPanel or OrchestrationDrawer shows per-concept results, novel-step counts, alternatives explored, and real probe progress (lines 72-74, 108, 114-118).
- **B21-013 [E] Implementation symbols:** The produced implementation names include SystemProber/DefaultSystemProber, FeasibilityProber/DefaultFeasibilityProber, the report/result/escalation types, and FSM FeasibilityProbe (lines 76-99).
- **B21-014 [E] SSB vocabulary:** The wiring contract lists SystemProbe, FeasibilityReport, NovelStepDetected, StubRun, AlternativeExplored, and FeasibilityEscalated (lines 93-99).
- **B21-015 [C] Implemented-name divergence:** Implementation renamed feasibility_probe_started and feasibility_probe_complete to system_probe and feasibility_report. C-22 is claimed to add alternative_explored and feasibility_escalated atomically at schema 53 → 54 (lines 100-104).
- **B21-016 [E] FSM/reachability:** FeasibilityFsmBridge consumes FsmState, drive_task reaches the probe, and the expected observation is a system_probe row plus dashboard per-concept progress (lines 105-118).
- **B21-017 [E] Hard bans:** Tokio kill-on-drop reliance and stubs over 50 lines are forbidden (lines 111-114).
- **B21-018 [E] Test target:** The plan calls for 24 tests—16 contract and eight adversarial—covering 12 testable invariants; the deterministic check expects at least 24 feasibility tests (lines 21, 120-134).
- **B21-019 [C] Stale action check:** The check searches FeasibilityProbeStarted even though the divergence note says implementation uses SystemProbe (lines 100-104, 132-134).
- **B21-020 [C] Acceptance/status ambiguity:** All acceptance rows remain unchecked and the files-touched section speaks prospectively, yet the wiring note says C-22 closed event gaps. This proves only a partial implementation-status claim, not full Phase-4F acceptance (lines 136-162).
- **B21-021 [E] Acceptance gates:** Completion requires all eight substeps, real telemetry, Tier-C bounded stubs, explicit timeouts, per-concept sequencing, exact skip behavior, MessagePack persistence, live panel rendering, and 24+ tests (lines 136-146).
- **B21-022 [A] Dependency gate:** nvml-wrapper and sysinfo are new dependencies requiring confirmation (lines 148-162).
- **B21-023 [C] File/name drift:** The plan first names feasibility.rs, while the wiring contract names system_probe.rs and feasibility_probe.rs; dashboard command wiring is attributed to B-36 and the preexisting stub.get_feasibility_report (lines 78-92, 148-160).
- **B21-024 [E] Lifecycle:** B-22 autonomous mode is next (lines 164-166).

### B-23 — Phase 4H Self-Modification

Source: B-23_phase_4H_self_modification.md, lines 1-227.

- **B23-001 [E] Scope/dependencies:** Phase 4H covers data, configuration, and structural modification plus DGM selection, staged evaluation, SWE-bench outcome grading, MSI, a closed mutable-key set, and four safety layers. It requires autonomous B-22 and B-15 through B-20 hardening and is parallel-safe with B-24 through B-28 (lines 1-9).
- **B23-002 [E] L1 lane:** Layer 1 performs pure-arithmetic, no-LLM updates over template rankings, rule-trigger counts, and pipeline outcomes; security/safety rules are never touched (lines 17-20, 64-67).
- **B23-003 [E] L1A lane:** Layer 1A additively grows deprecated_patterns, FailureTaxonomy, lazy-code regex, and source-tier tables without an LLM or staged evaluation, validates false-positive rate, and stores successful patterns in a DarwinNet Gene Pool (lines 21-24).
- **B23-004 [E] Data growth event:** Each L1A growth emits data_table_grew/DataTableGrew (lines 67-68, 149).
- **B23-005 [E] L2 archive:** ConfigArchive stores at most 50 variants independently per PipelineMode and is designed to survive deceptive score dips (lines 25-32, 69-70).
- **B23-006 [E] Parent selection:** L2 uses sigmoid-weighted DGM parent selection with lambda = 10 and alpha_0 = 0.5 (lines 25-27, 69-70).
- **B23-007 [E] Log-driven proposals:** L2 reads SSB and reasoning_trace, identifies where reasoning failed, and proposes a config patch (lines 71-72).
- **B23-008 [E] Staged evaluation:** Evaluation progresses through 10, 50, and 200 tasks; more than 40% passing and the top two variants promote to Stage 3 (lines 27-28, 73-74).
- **B23-009 [E] Outcome grading:** TestOutcomeReport uses four SWE-bench categories and PASS_TO_FAIL causes immediate rejection; SWR pre-screening is optional (lines 27-28, 75-76).
- **B23-010 [E] L3 non-goal:** Structural FSM-state, agent-role, or GRC-protocol changes are HUMAN-ONLY stubs; automation may emit l3_proposal for review but may not auto-promote (lines 29, 44-46, 85-86).
- **B23-011 [E] MSI calculation:** MsiTracker computes an exponential-decay weighted average of delta_score with lambda = 0.3 over N = 10, maintaining independent SURGICAL and GREENFIELD histories (lines 34-36, 79).
- **B23-012 [E] MSI state machine:** IMPROVING > 0.3 continues; STABILIZING from -0.1 through 0.3 reduces modification frequency; DIVERGING <= -0.1 pauses and alerts (lines 37-39).
- **B23-013 [E] Auto-revert:** More than 10% degradation across three pipelines triggers revert; a trigger task checks objective hacking (lines 41-42).
- **B23-014 [E] Mutable-key authorization:** AllowedConfigKey is a closed enum; cloud_budget, gate definitions, and score weights are locked, and adding a mutable key requires Layer-3 human approval (lines 44-46, 79-81).
- **B23-015 [E] Four enforcement layers:** Dispatch, IPC, filesystem, and git independently enforce locks; the modifying agent cannot read the lock list (lines 48-49, 77-78).
- **B23-016 [E] Anti-stale integration:** Nightly dependency audit can grow deprecated_patterns and trigger steering-vector refresh (lines 51-52, 82-83).
- **B23-017 [E] Reasoning integration:** failure_taxonomy_misses feed L1A category growth (lines 53, 84).
- **B23-018 [E] Dashboard activation:** ProposalsPanel is activated behind a feature flag and shows archive, per-mode MSI, pending proposals, staged results, and last revert; MsiPanel shows mode verdict and convergence/regression/evolution badges (lines 87-89, 120-123, 165-171).
- **B23-019 [E] OSC similarity:** ConfigVariantSimilarity computes key_overlap × 0.5 + type_match × 0.3 + exact_match × 0.2 over deserialized config_blob (lines 91-94).
- **B23-020 [E] OSC stagnation:** Similarity >= 0.95 for three consecutive variants marks Stagnant and boosts exploration, independently per mode (lines 94-98).
- **B23-021 [E] OSC period-two proof:** Oscillation requires both sim(N,N-2) and sim(N-1,N-3) >= 0.95; differing keys are excluded from AllowedConfigKey at runtime for five pipelines (lines 95-98).
- **B23-022 [E] Cross-pipeline regression:** RegressionDetector examines the full PipelineOutcome history, unlike single-evaluation PASS_TO_FAIL; persistent criteria that never passed are not regressions (lines 102-110).
- **B23-023 [E] Regression control flow:** A detected regression blocks 4H.5 promotion despite an improved score and three consecutive regressions on the active variant trigger 4H.6 auto-revert, per mode (lines 104-109).
- **B23-024 [E] Evolution gate:** evolved_count is the count of adjacent variant pairs with similarity < 0.95; when it is zero and latest similarity >= 0.95, evolution is withheld and that mode’s MSI is forced to STABILIZING (lines 112-118).
- **B23-025 [E] Convergence events:** Four new rows are convergence_stagnant, config_oscillation, regression_detected, and evolution_withheld, surfaced beside per-mode MSI (lines 120-123).
- **B23-026 [E] Wiring symbols:** The implementation contract names DefaultLayer1aProcessor, SelfModOrchestrator, ConfigArchive, evaluate_stage, run_full_pipeline, TestOutcomeReport, MsiTracker, AllowedConfigKey, is_locked, check_locked_key, should_auto_revert, ConfigVariantSimilarity, OscVerdict/detect, RegressionReport, compute_evolution_gate, and EvolutionGateResult (lines 125-144).
- **B23-027 [C] Base event naming/count:** The wiring contract explicitly names ProposalGenerated, ProposalApplied, ProposalRejected, MsiUpdate, DataTableGrew, and L3Proposal plus four convergence events—ten named variants (lines 145-154). The files section instead says 11 new action types comprising seven base plus four convergence events (lines 213-215), leaving one base variant unnamed in this source.
- **B23-028 [E] Reachability:** The self-mod loop is reached from autonomous B-22 and a nightly cron; PipelineOutcome and ProposalsPanel are consumers (lines 155-160).
- **B23-029 [E] Hard safety bans:** Locked keys must never be mutable, PASS_TO_FAIL must reject immediately, and DIVERGING MSI must pause and alert (lines 161-165).
- **B23-030 [E] Observable evidence:** The Layer1a SSB smoke must produce msi_update; the UI must show an archive capped at 50 and independent IMPROVING/STABILIZING/DIVERGING modes (lines 166-171).
- **B23-031 [E] Deterministic checks:** Static AllowedConfigKey discovery plus at least 42 self-mod tests are the minimal scripted checks (lines 173-181).
- **B23-032 [C] Acceptance expansion:** The original 13-substep/42-test scope was expanded by OSC/REG/EVO to 54+ tests and four additional events, while every acceptance box remains unchecked (lines 183-203).
- **B23-033 [E] Acceptance gates:** Required outcomes include functional L1/L1A/L2, 50-variant archives, 10/50/200 staging, four-category grading, per-mode MSI, closed keys, four locks, anti-stale/reasoning wiring, L3 proposal-only behavior, live panels, OSC period-two validation, REG promotion/revert control, and EVO forced stabilization (lines 183-203).
- **B23-034 [E] Artifact lifecycle:** Planned artifacts are self_mod.rs, config_archive.rs, msi.rs, osc.rs, regression.rs, evolution_gate.rs, SSB changes, ProposalsPanel/MsiPanel, optional self_mod_cron.sh, and B-23-decisions (lines 205-223).
- **B23-035 [E] Dashboard API compatibility:** Existing stub names get_pending_proposals, get_applied_proposals, apply_proposal_from_user, reject_proposal, and get_msi_status must be retained one-to-one when V2 wiring lands (lines 216-221).
- **B23-036 [E] Lifecycle:** B-24 media validation follows (lines 225-227).

### B-29.5 — Symbol Graph Scale Benchmark

Source: B-29.5_symbol_graph_scale_benchmark.md, lines 1-461.

- **B29S-001 [E] Scope:** The benchmark must prove B-29’s cache and four traversals at Linux-kernel, Chromium, VS Code, and PostgreSQL scale without OOM, with p99 below 500ms and stable hop-banded output (lines 1-7).
- **B29S-002 [E] Dependencies/parallelism:** B-29 and resumable B-31 are prerequisites; B-31.5 may run in parallel and share fixtures (lines 8-12).
- **B29S-003 [E] Risk thesis:** Sharding, LRU payload bounds, hop-banded BFS, and lazy DFS were only proven on about 140K LOC/8,075 nodes; the benchmark must establish that no query loads the full graph and memory follows result-set size rather than total graph size (lines 14-37).
- **B29S-004 [E] External-evidence boundary:** SocratiCode’s 40M-LOC claim is treated as unverified because its located test is about 200K symbols/10K files; ELAI must generate its own evidence (lines 24-27).
- **B29S-005 [E] Research anchor:** FocalAD ELAI arXiv:2506.11419 supports top-k local-focus selection, while this benchmark is explicitly the engineering bounded-memory/latency proof (lines 39-45).
- **B29S-006 [E] Fixture root:** Fixtures live outside .elai_cc at configurable ELAI_SCALE_FIXTURE_ROOT, defaulting to a home-directory elai_scale_fixtures path; this prevents ordinary project indexing from ingesting them (lines 64-78).
- **B29S-007 [E] Linux fixture:** clone_scale_fixtures.sh shallow-clones torvalds/linux v6.7, approximately 36M LOC and primarily C/asm (lines 79-81, 99-104).
- **B29S-008 [E] Chromium fixture:** The script shallow-clones chromium/src 121.0.6167.184 without depot_tools, approximately 35M LOC and mixed C++/JS/TS/Python/ObjC (lines 82-85, 105-110).
- **B29S-009 [E] VS Code fixture:** The script shallow-clones microsoft/vscode 1.94.2, approximately 2.45M LOC of TS/JS/CSS/HTML (lines 86-88, 111-116).
- **B29S-010 [E] PostgreSQL fixture:** The script shallow-clones postgres REL_17_0, approximately 1.5M LOC of C/Perl/Tcl/SQL/shell (lines 89-91, 117-122).
- **B29S-011 [E] Reproducible acquisition:** Fixture tags/commits are pinned, existing directories are not recloned, and phase_b29_5_fixture_inventory.toml records paths, pins, exact measured LOC, and languages to prevent HEAD drift (lines 70-122).
- **B29S-012 [E] Criterion harness:** symbol_graph_scale.rs benchmarks impact_radius, call_flow, symbol_context, and list_symbols separately per tier/tool with criterion sample_size 20 and fixed high-fan-in symbols (lines 124-165).
- **B29S-013 [E] Per-language diagnosis:** Separating each tier/tool combination exposes degradation such as template-heavy C++ resolution versus plain C (lines 163-165).
- **B29S-014 [E] Memory snapshot schema:** MemorySnapshot records rss_bytes, heap_bytes, cache_lru_size, name_index_loaded, and reverse_index_loaded (lines 167-178).
- **B29S-015 [A] Memory instrumentation:** RSS uses procfs on Linux and GetProcessMemoryInfo on Windows. Heap may use jemalloc-ctl only if jemalloc is already global; adding it requires user approval (lines 179-181).
- **B29S-016 [E] Realistic workload:** The workload is exactly 1,000 queries: 60% impact_radius depth 2-3, 25% symbol_context, 10% call_flow, and 5% list_symbols, starting from a cold-loaded cache (lines 183-190).
- **B29S-017 [E] Workload metrics:** Each tool records p50/p95/p99, plus LRU hit rate, peak memory, and final file_payload_lru size, which must be <= 500 (lines 189-190, 219-229).
- **B29S-018 [E] Indexing evidence:** All four fixtures are indexed with per-tier metrics covering wall time, peak indexing RSS, node/edge counts per language, symbol_count, edge_count, file_count, and unresolved_edge_pct (lines 192-209).
- **B29S-019 [E] Recovery dependency:** Linux/Chromium indexing is expected to take one to three hours and must use B-31 resume after interruption (lines 207-209, 324-329).
- **B29S-020 [E] Benchmark artifact:** Criterion JSON is written to phase_b29_5_bench_raw.json, expected valid and over 100KB with tier × tool entries (lines 211-218, 338-341).
- **B29S-021 [E] Repeat determinism:** The bench runs twice; a target must return identical total_files and identical hop-band sets, with cross-run variance below 10% for the reported p99 observation (lines 216-218, 305-310).
- **B29S-022 [E] Aggregate schema:** phase_b29_5_scale.toml records per-tier index counts/timing/shard sizes and nested workload query count, hit rate, peak RSS, and per-tool quantiles (lines 231-256).
- **B29S-023 [E] Five-run regression proof:** symbol_graph_determinism runs impact_radius five times and requires byte-identical BTreeMap<u32, Vec<String>> plus identical truncated flags for linux::kmalloc, vscode::vscode_extension, chromium::BrowserMain, and postgres::PortalRunMulti (lines 258-267).
- **B29S-024 [E] CI performance guard:** scale_p99_budget_contract reads populated metrics, requires every tool/tier p99 <= 500ms, LRU memory <= 2GB on a 16GB-equivalent machine, and unresolved_edge_pct <= 50% (lines 268-273).
- **B29S-025 [E] Reproduction record:** phase_b29_5_README documents exact pins, sample size, target CPU/RAM/storage, and expected per-tier wall time (lines 275-280).
- **B29S-026 [E] Wiring contract:** Produced symbols are the criterion harness, MemorySnapshot/snapshot, run_realistic_workload, determinism regression, and p99 CI contract; they consume SymbolGraphCache, all four B-29 traversals, and B-29/B-31 indexing (lines 282-295).
- **B29S-027 [E] Reachability/resource gate:** The bench is reached by cargo bench; scale tests are include-ignored because fixtures must be pre-cloned. The CI contract must read populated data and fixture cloning must use pins (lines 295-304).
- **B29S-028 [E] OS-visible observation:** A completed run must populate scale metrics and show stable p99 across two runs with <10% variance (lines 305-310).
- **B29S-029 [E] Deterministic done gates:** Checks cover pinned HEADs, four resumable indices, nonempty symbol_graph_meta, JSON criterion output, the 500ms contract, five-run hop-band identity, and <=2048MB Chromium workload RSS (lines 312-355).
- **B29S-030 [E] Acceptance performance bounds:** Every traversal on every tier must meet p99 <=500ms; a 1,000-query workload must remain <=2GB peak RSS; Linux should have about 5M+ DKG nodes/2M+ symbols and overall unresolved edges below 50% (lines 357-375).
- **B29S-031 [E] Language-quality evidence:** Per-language unresolved_edge_pct is documented but not gated because C/C++ dynamic dispatch/macros are expected to be worse than TS/JS (lines 376-378).
- **B29S-032 [E] Failure escalation:** Any tier that misses p99 reopens B-29 and requires a decision record; failure may not be silently accepted (lines 379-380).
- **B29S-033 [E] Dependency permission:** No external crate, including jemalloc/procfs candidates, may be added without explicit user confirmation (lines 381-382).
- **B29S-034 [E] Required artifacts:** The source enumerates the clone script, three bench/workload modules, two tests, fixture/scale/readme/per-tier metrics, and B-29.5-decisions (lines 384-410).
- **B29S-035 [A] Official-run policy:** The decision record must choose official hardware, preserve host-specific sibling metrics rather than overwrite official numbers, and decide full-four-tier gating versus the recommended Linux+VS Code minimum (lines 412-423).
- **B29S-036 [A] Budget/scheduling policy:** The decision record must justify 500ms, document user-facing hook cost, choose memory instrumentation, and schedule the multi-hour benchmark nightly rather than per commit (lines 424-435).
- **B29S-037 [E] Secondary outcomes:** Published metrics ground performance claims, identify weak B-30 extractors, share fixtures with B-31.5/B-30, and expose C++ extraction regressions (lines 437-449).
- **B29S-038 [E] Explicit non-goal:** Auditor signal-to-noise for impact results is not measured here; it is only a future self-modification signal candidate (lines 450-456).
- **B29S-039 [E] Lifecycle:** B-31.5 uses the same fixtures for an orthogonal metric domain and may run in parallel (lines 458-461).
- **B29S-040 [A] Fixture database path:** The symbol-graph check uses {fixture}/.elai_cc/.elai_cc/indexes/dkg.db (lines 331-335); the duplicated directory segment is not explained and must be reconciled with the actual B-1/B-31 layout before automation.
- **B29S-041 [C] Threshold boundary:** The objective says p99 “under 500ms,” while contracts and acceptance use <= 500ms (lines 5, 270, 345, 366). The exact equality boundary is internally inconsistent.
- **B29S-042 [E] Status:** No shipped overlay exists and all acceptance rows are unchecked, so this source is a planned benchmark/release-evidence gate rather than measured proof (lines 357-382).

### B-35 — Context Artifacts Pipeline

Source: B-35_context_artifacts_pipeline.md, lines 1-596.

- **B35-001 [E] Scope:** B-35 is a full second indexing pipeline for project-typed non-code knowledge—schemas, OpenAPI, Kubernetes, ADRs, glossaries, and Terraform—so Designer and Researcher can answer from project conventions rather than inventing them (lines 1-8).
- **B35-002 [E] Dependencies/parallelism:** B-1 path unification, B-31 lock primitives, and B-32 chunking are prerequisites; B-29 is a recommended sibling, and B-29/B-30/B-33/B-34 are parallel-safe (lines 9-13).
- **B35-003 [E] Re-scope rationale:** The source corrects an earlier “config block” scope to a complete parallel pipeline with its own collection, chunking, embedding/index, staleness, and search contract, auto-indexed with code (lines 15-27).
- **B35-004 [E] Project-state coverage:** The pipeline grounds database naming/default/trigger conventions, existing API patterns, deployment files, ADRs, and domain language that code-only grounding misses (lines 29-42).
- **B35-005 [E] Port architecture:** The ELAI port has a manifest, per-project context index, reused B-32 chunking, blake3 staleness, ArtifactStage between CatRAG and A-RAG, and real PipelineGrc artifact verification (lines 44-59).
- **B35-006 [C] Hash correction:** SocratiCode is described as using SHA-256, but ELAI must use blake3 under the anti-stale rule (lines 21-27, 49-50).
- **B35-007 [A] Manifest format:** The source requires a choice between TOML knowledge.artifacts and project-root .elai_artifacts.json, recommends TOML, and later recommends JSON only as a SocratiCode-import fallback; no final choice is recorded (lines 87-98, 540-545).
- **B35-008 [E] Artifact declaration schema:** Each declaration has name, file-or-directory path, and nonempty steering description phrased as “check this before X” (lines 99-123, 357-361).
- **B35-009 [E] Example artifact classes:** The canonical examples are database-schema, api-spec, ubiquitous-language, and k8s-manifests, with SQL/OpenAPI/domain/deployment descriptions that encode when agents should consult them (lines 99-120).
- **B35-010 [E] Manifest loader:** ArtifactManifest::load(project_root) parses declarations and returns Vec<ArtifactSpec> (lines 125-135).
- **B35-011 [E] Name validation:** Artifact names must match [a-zA-Z0-9_-]+ because names become collection suffixes (lines 127-129).
- **B35-012 [E] Path security:** Relative paths resolve against project_root and are rejected if canonical resolution escapes it, preventing arbitrary system-file reads (lines 130-132).
- **B35-013 [E] Directory expansion:** Directory artifacts recursively expand during indexing and obey the project’s existing .elai_ignore rules (lines 133-135).
- **B35-014 [E] ArtifactSpec schema:** ArtifactSpec contains name, path, description, and is_dir (lines 137-144).
- **B35-015 [E] ArtifactChunk identity/schema:** ArtifactChunk carries a blake3 id of name/path/byte_offset, artifact name/description, relative path, content, start/end lines, whole-file content_hash, and an optional embedding (lines 145-155).
- **B35-016 [E] Index-state schema:** ArtifactIndexState contains name, last_indexed_at, deterministic BTreeMap file_hashes, chunk_count, and ArtifactStatus (lines 156-162).
- **B35-017 [E] Artifact lifecycle states:** Status is a closed NotIndexed | Indexed | Stale { changed_files } | Indexing enum; None/Unknown is forbidden (lines 163-168, 357-360).
- **B35-018 [E] Incremental index loop:** index_artifact reads each expanded file, computes blake3, skips an unchanged hash, and rebuilds only changed files before persisting ArtifactIndexState (lines 171-192).
- **B35-019 [E] Text chunking:** Markdown, YAML, JSON, Terraform, and TOML use B-32 chunk_by_lines with CHUNK_SIZE = 50 plus apply_char_cap (lines 181-185).
- **B35-020 [E] SQL chunking:** SQL uses simple-regex boundaries at CREATE TABLE, CREATE VIEW, and CREATE INDEX rather than generic line windows (lines 182-184).
- **B35-021 [E] Embedding document shape:** Every chunk uses the existing embedding provider and a “context_artifact: {artifact_name}” prefix before content, then upserts into the project context index (lines 185-190).
- **B35-022 [E] Context storage schema:** context_chunks has id, artifact_name, relative_path, content, start_line, end_line, embedding, content_hash, and indexed_at; HNSW indexes embeddings and Tantivy indexes content (lines 193-200).
- **B35-023 [E] Project isolation:** Artifact storage is named context_{project_id}, kept separate from codebase_/codegraph_ collections and the code DKG (lines 193-200, 492-495, 554-558).
- **B35-024 [E] Auto-index lifecycle:** After code indexing, IndexingService discovers the manifest, loads states, detects stale files, and reindexes changed files only; update_index repeats staleness detection (lines 202-208, 341-345).
- **B35-025 [E] Failure containment:** Artifact indexing failures warn and continue; they may never compromise or block code-index integrity (lines 206-208, 448-450, 492-495).
- **B35-026 [C] Failure-event shape gap:** The missing-path check expects ArtifactsIndexed to contain a failed list, but the declared ArtifactsIndexed payload lists only indexed, up_to_date, total_chunks, and latency_ms (lines 303-306 versus 419-423). failed is required by the check but absent from the schema.
- **B35-027 [A] Cross-process lock:** Artifact indexing either shares B-31 LockScope::Index with code indexing or receives LockScope::Artifacts; shared scope is recommended to avoid embedding-provider rate-limit collision, but the decision is open (lines 210-215, 549-551).
- **B35-028 [E] Retrieval placement:** ArtifactStage runs between CatRAG and A-RAG in STANDARD and DEEP tiers; FAST remains code-only (lines 217-227).
- **B35-029 [E] Retrieval algorithm/result:** Artifact search fuses semantic embedding and BM25 via RRF and returns ArtifactSearchResult values with score, content, and artifact metadata (lines 219-225).
- **B35-030 [E] Dispatcher search defaults:** search_artifacts accepts project_id, query, optional artifact_name, and limit defaulting to 10 (lines 229-240).
- **B35-031 [E] Four dispatcher tools:** list_artifacts returns declarations/status, index_artifacts force-reindexes, search_artifacts searches semantically with optional filter, and remove_artifacts drops the context collection (lines 241-247).
- **B35-032 [E] Seventh grounding method:** GroundingMethod adds ArtifactCheck { artifact_name, query } after FileCheck, SourceCheck, TestCheck, WebCheck, ApiCheck, and ApiRegistryCheck (lines 249-263).
- **B35-033 [E] Artifact verdicts:** ArtifactCheck dispatches filtered search; any score > 0.5 is Verified, no qualifying result is Unverified, and an undeclared name is MalformedGrounding with reason “artifact not declared” (lines 264-268).
- **B35-034 [E] Grounding gate behavior:** Designer claims should cite the relevant artifact; GroundingSweep blocks a project-relevant claim lacking artifact backing and any other grounding source (lines 270-276).
- **B35-035 [E] Designer briefing:** TechStackDecision::ExistingProject injects AVAILABLE_ARTIFACTS as name/description pairs and tells Designer to search the database/API/infrastructure/domain artifact before decisions (lines 278-283, 350-351).
- **B35-036 [E] Dashboard panel:** ArtifactsPanel shows name, path, description, status, chunk count, and last-indexed time; it exposes per-artifact Re-index and filtered Search and shows the first 200 description characters (lines 285-295).
- **B35-037 [E] Canonical Tauri commands:** list_project_artifacts, search_project_artifact, reindex_project_artifact, and remove_project_artifacts are the four explicit command signatures (lines 297-301, 337-339).
- **B35-038 [C] Command-name drift:** The later file plan instead calls the V2 commands list_artifacts, index_artifacts, search_artifacts, and remove_artifacts to match stub.ts (lines 520-526). The source does not reconcile those names with the project-prefixed signatures.
- **B35-039 [E] Artifact events:** ArtifactsIndexed carries project_id, indexed, up_to_date, total_chunks, and latency_ms; ArtifactSearched carries project_id, optional artifact name, query, result count, and latency_ms; ArtifactStale carries project_id, artifact name, and changed files (lines 303-307).
- **B35-040 [A] Activity surface terminology:** The source says ActivityFeed displays the three events (line 308), although its V2 file plan mounts ArtifactsPanel in Workspace/KnowledgeDrawer (lines 527-531); it does not specify the V2 orbit mapping.
- **B35-041 [E] User documentation:** docs/examples/elai_cc.example.toml must show SQL, OpenAPI, Kubernetes, Terraform, ADR, and glossary declarations and explain description steering; dashboard-design may receive a panel mockup only if mutable (lines 310-320).
- **B35-042 [E] Wiring producers:** The contract produces manifest/types/index/search/stage symbols, ArtifactCheck, three SSB variants, four dispatcher entries, four dashboard commands, and ArtifactsPanel (lines 322-340).
- **B35-043 [E] Wiring consumers:** build_index and update_index invoke artifact indexing/staleness; QueryPipeline consumes ArtifactStage; verify_claim dispatches ArtifactCheck; run_designer_briefing injects AVAILABLE_ARTIFACTS (lines 341-351).
- **B35-044 [E] Reachability:** CLI index, CLI research, dashboard submit, and IndexingService build paths must reach the pipeline (lines 352-356).
- **B35-045 [E] Stub bans:** Status cannot be unknown, descriptions cannot be empty, context_{project_id} must exist after indexing, and ArtifactCheck must validate declaration before search rather than silently return ungrounded (lines 357-365).
- **B35-046 [E] Observable workflow:** Declaring a real SQL artifact then indexing must produce ArtifactsIndexed with total_chunks > 0; a filtered “users” search must return the CREATE TABLE users chunk (lines 368-375).
- **B35-047 [E] Staleness recovery check:** Editing schema.sql then indexing should emit ArtifactStale and reindex only database-schema (lines 391-395).
- **B35-048 [E] Removal lifecycle:** remove_artifacts empties or drops context_chunks (lines 414-417).
- **B35-049 [E] Missing-path recovery:** A nonexistent artifact path must warn, record failure without panic, and preserve the code index (lines 419-423).
- **B35-050 [E] Escape rejection evidence:** A ../../etc/passwd declaration must fail manifest load with an outside-project-root error (lines 425-428).
- **B35-051 [A] Session-path drift:** The grounding SQL check uses .elai_cc/.elai/sessions/ssb.db (lines 401-407), which is not reconciled in this source with B-1’s .elai_cc path unification prerequisite.
- **B35-052 [E] Acceptance gates:** Completion requires manifest parsing/validation, B-32 and SQL chunking, blake3 staleness, project context storage, STANDARD/DEEP stage routing, four tools, four commands, three events, ArtifactCheck and Designer use, transparent reindex, nonfatal errors, secure paths, collision-free locking, an example manifest, zero production unwrap, and no unapproved dependency (lines 431-457).
- **B35-053 [E] Manifest tests:** Tests parse 5+ artifacts, reject escape/invalid names, and support file and directory paths (lines 459-465).
- **B35-054 [E] Chunk/index tests:** SQL boundaries, YAML/JSON/Markdown windows, and populated artifact_name/description fields are tested (lines 466-470).
- **B35-055 [E] Idempotent staleness tests:** First index builds all, unchanged reindex skips all, one modification rebuilds one, a new directory file adds one, and a removed file deletes its chunks (lines 471-476).
- **B35-056 [E] Search tests:** Search works across all artifacts or one name and returns score, content, and metadata (lines 477-480).
- **B35-057 [E] Grounding tests:** Above-threshold results verify, undeclared names are malformed, and no above-threshold result is unverified (lines 481-484).
- **B35-058 [E] Briefing/UI tests:** ExistingProject injects AVAILABLE_ARTIFACTS while Greenfield skips it; V2 E2E renders status and exercises live Re-index updates (lines 485-491).
- **B35-059 [E] Contract tests:** Artifact failures must not block code indexing, and context collections must remain isolated from code collections (lines 492-495).
- **B35-060 [E] File lifecycle:** The source enumerates six artifact modules, ArtifactStage, pipeline/export/indexing/dispatcher/task-runner/grounding/CLI/dashboard/SSB changes, example/docs, and a decision record (lines 497-536).
- **B35-061 [A] Threshold/config choice:** The decision record must choose fixed ArtifactCheck score >0.5 versus per-artifact override; default 0.5 is recommended (lines 554-555).
- **B35-062 [E] Explicit graph non-goal:** Artifacts should not produce DKG nodes; the recommendation is to keep DKG code-only and use artifact search (lines 556-558).
- **B35-063 [A] Checkpoint granularity:** Artifact state is recommended per artifact, not per B-31 batch, because artifacts are typically under 100 files and whole-artifact rebuild is acceptable (lines 559-561).
- **B35-064 [E] Integration outcomes:** B-35 provides non-code ExistingProject context, grounded research, DDD vocabulary control, an Empire-spec path, and possible B-23 proposal fuel (lines 566-583).
- **B35-065 [E] Compliance boundary:** SOC2/HIPAA/GDPR documents may be declared as artifacts to ground future compliance-aware plans, but the source presents this as a foundation/side benefit, not a shipped compliance guarantee (lines 584-586).
- **B35-066 [E] Lifecycle:** B-29, B-31, B-34, and B-23 are the named downstream/complementary relationships (lines 588-596).
- **B35-067 [A] Collection-name tension:** Artifact name is validated because it is said to become a collection suffix (lines 127-129), while storage is specified as one context_{project_id} collection with artifact_name rows (lines 193-200). The source does not choose per-artifact collections versus one per-project collection.
- **B35-068 [E] Status:** B-35 has no shipped overlay and every acceptance box is unchecked; it is a planned pipeline (lines 431-457).

## Requested-axis traceability

This index maps the atomic ledgers to every requested discovery axis; it adds no new feature atoms.

| Source | Capabilities/actions/surfaces | Workflows/states | Schemas/data/lifecycle | Flags/defaults/config | Integrations/events | Permissions/security/privacy/compliance | Failure/fallback/retry/recovery | Determinism/idempotency/concurrency | Observability/evidence/tests/done | Constraints/non-goals/dependencies/status |
|---|---|---|---|---|---|---|---|---|---|---|
| B-7 | B7-002–020, B7-024 | B7-006–020, B7-025 | B7-003–016, B7-021, B7-031–033 | B7-017, B7-029–032 | B7-003, B7-008–016, B7-024 | B7-028 (dependency approval; no separate privacy claim) | B7-008, B7-011–016, B7-023, B7-031 | B7-021, B7-026, B7-029–032 | B7-025–027 | B7-001, B7-015–023, B7-027–033 |
| B-16 | B16-004–018 | B16-003, B16-009–017 | B16-004–008, B16-014–016, B16-022 | B16-003, B16-009, B16-012–017 | B16-014–018 | B16-003, B16-009, B16-011–017 | B16-003, B16-009–010, B16-017, B16-019 | B16-009, B16-012–016, B16-020 | B16-016, B16-018–021 | B16-001–003, B16-017, B16-019, B16-022–024 |
| B-18 | B18-003–020 | B18-005–015, B18-018–020 | B18-014, B18-017, B18-020 | B18-006–013, B18-017 | B18-014–015, B18-021 | B18-005–018 | B18-013, B18-016, B18-018–020 | B18-005–010, B18-014, B18-019–020 | B18-019–024 | B18-001–004, B18-022–026 |
| B-19 | B19-003–014, B19-018 | B19-005–010, B19-014 | B19-008, B19-010–012 | B19-005, B19-007–009, B19-013 | B19-003–014, B19-018–019 | B19-007, B19-014 | B19-007–009, B19-013–016 | B19-003, B19-008–13 | B19-012–13, B19-019–022 | B19-001–002, B19-015–017, B19-020–025 |
| B-21 | B21-002–016 | B21-002, B21-006–012, B21-016 | B21-003–004, B21-011, B21-014–015 | B21-007–010 | B21-012–016 | B21-006–007 (Tier-C/tool restriction) | B21-006, B21-008–010 | B21-005, B21-008–009, B21-018 | B21-012, B21-016, B21-018–021 | B21-001, B21-017, B21-020–024 |
| B-23 | B23-002–030, B23-035 | B23-002–025, B23-028–029 | B23-003–009, B23-011–14, B23-019–27 | B23-005–006, B23-011–15, B23-018, B23-021 | B23-004, B23-016–018, B23-025–030 | B23-002, B23-010, B23-014–015 | B23-009–15, B23-20–24, B23-029 | B23-002, B23-005–006, B23-011, B23-019–24 | B23-018, B23-025, B23-030–033 | B23-001, B23-010, B23-027, B23-032–036 |
| B-29.5 | B29S-006–029 | B29S-006–029, B29S-032 | B29S-011, B29S-014, B29S-017–025, B29S-034–035 | B29S-006, B29S-012, B29S-016, B29S-035–036 | B29S-018–027 | B29S-033 (dependency authority; fixture network access) | B29S-019, B29S-032 | B29S-006–12, B29S-016–29 | B29S-018–34, B29S-041–042 | B29S-001–005, B29S-015, B29S-030–42 |
| B-35 | B35-005–046 | B35-017–050 | B35-007–24, B35-29–39, B35-47–60 | B35-007–13, B35-17, B35-27–38, B35-61–63, B35-067 | B35-024, B35-028–45 | B35-012, B35-033–35, B35-045, B35-050, B35-065 | B35-018, B35-024–27, B35-033–35, B35-047–50 | B35-016, B35-018, B35-023–29, B35-055, B35-063 | B35-039–59, B35-068 | B35-001–007, B35-023, B35-027, B35-040, B35-051–52, B35-061–68 |

## Cross-source interactions

- **INT-001 [E]:** B-16’s blake3-over-blake2 rule directly governs B-19’s phase-4C hash drift and B-35’s SHA-256-to-blake3 port (B16 lines 169-180; B19 lines 52-54; B35 lines 21-27, 49-50).
- **INT-002 [E]:** B-18 supplies the IPC accept/frame path that makes IpcHookDispatcher constructible/live in B-19; B-19 nonetheless retains the in-process implementation as default (B18 lines 81-88; B19 lines 17-20, 34-50).
- **INT-003 [E]:** B-18’s Tier-C process is the required execution boundary for B-21 feasibility stubs (B18 lines 40-89; B21 lines 1-8, 51-53).
- **INT-004 [E]:** B-18 introduces four self-modification enforcement layers; B-23 makes the same dispatch/IPC/filesystem/git set a hard invariant and hides the lock list (B18 lines 102-105; B23 lines 44-49, 77-81).
- **INT-005 [E]:** B-7’s real Concept→Phase→Step→BabyStep tree and detail views provide the baby-step addressability B-21 requires for per-concept novelty probes (B7 lines 181-201; B21 lines 6-7, 12-20).
- **INT-006 [I]:** B-21 classifies an unverified PackageRef as novel, while B-7 says PackageRef verification producer flow is wire-dark. Until B-7-followup materializes PackageRef data, this novelty input may remain unavailable even though its emitter substrate exists (B7 lines 63-79; B21 lines 15-17, 48-49).
- **INT-007 [I]:** B-7’s TechStackDecision producer is wire-dark, while B-35 targets TechStackDecision::ExistingProject briefing injection. The artifact briefing can be specified independently, but a common live decision object is an integration prerequisite not resolved by either source (B7 lines 80-97; B35 lines 278-283).
- **INT-008 [E]:** B-19 PreCompact recovery consumes B-9 SessionDag::create_handoff at the 85% token threshold (B19 lines 17-20, 35-36, 122-126).
- **INT-009 [E]:** B-21 mode-specific escalation sends three failures to human escalation in INTERACTIVE or replan in AUTONOMOUS, linking probe recovery to B-22 mode semantics (B21 line 34).
- **INT-010 [E]:** B-23 is reachable only from B-22’s autonomous loop or the optional nightly cron; its structural changes stay human-only despite autonomous L1/L1A/L2 execution (B23 lines 7-9, 28-29, 85-89, 155-160).
- **INT-011 [E]:** B-23’s nightly dependency audit, deprecated-pattern growth, and steering refresh consume the anti-stale policy family established in B-16 (B16 lines 33-45; B23 lines 51-53, 82-84).
- **INT-012 [E]:** B-35 explicitly leaves artifact data as future fuel for B-23 Layer-2 proposals; it is an unlocked side benefit, not part of B-23’s current inputs (B35 lines 577-581, 595-596).
- **INT-013 [E]:** B-29.5 is an evidence gate over B-29 traversal/cache claims and depends on B-31 resume; B-31.5 shares fixtures but measures an orthogonal indexing domain (B29.5 lines 1-12, 192-209, 458-461).
- **INT-014 [E]:** B-35 depends on B-31’s lock lifecycle and B-32’s line/character chunkers; the exact artifact lock scope remains open (B35 lines 9-13, 181-185, 210-215).
- **INT-015 [E]:** B-35 inserts ArtifactStage into the existing QueryPipeline and adds ArtifactCheck to PipelineGrc, connecting knowledge retrieval to a blocking Designer GroundingSweep (B35 lines 44-59, 217-283).
- **INT-016 [E]:** B-29 SymbolGraphCache and B-35 artifacts are complementary Designer context sources; B-35 deliberately keeps artifacts out of the code DKG (B35 lines 9-13, 366-367, 556-558, 588-591).
- **INT-017 [E]:** B-7, B-21, B-23, and B-35 all depend on V2 surfaces or B-36 command wiring: Plan/Research/Orchestration windows, FeasibilityPanel, Proposals/Msi panels, and ArtifactsPanel respectively (B7 lines 111-155; B21 lines 148-160; B23 lines 216-221; B35 lines 523-531).
- **INT-018 [C]:** B-7 records SSB schema 99→100 and B-19 immediately records 100→101, but B-21’s later C-22 note records 53→54. If these refer to the same global schema, the histories conflict; none of the three source files explains branch or migration reconciliation (B7 lines 107-109; B19 lines 31-33; B21 lines 100-104).
- **INT-019 [E]:** B-16’s error/fallback doctrine appears concretely as B-18’s weaker Windows fallback, B-21’s explicit hard kill/escalation, B-23’s reject/pause/revert paths, and B-35’s MalformedGrounding/nonfatal artifact failure behavior.
- **INT-020 [I]:** B-29.5’s per-language unresolved-edge evidence can prioritize B-30 extractor work and can improve the B-29 symbol context paired with B-35, but neither benchmark completion nor artifact completion directly implements the other (B29.5 lines 437-449; B35 lines 9-13).
- **INT-021 [E]:** Security boundaries compose rather than replace one another: B-18 controls agent OS/process/path visibility, B-23 controls self-modifiable state, and B-35 controls artifact file roots. None claims to subsume the others.
- **INT-022 [E]:** Compliance-aware planning in B-35 is only future groundwork; B-18/B-23 supply isolation and modification controls, but this chunk specifies no certification, audit report, retention policy, or regulatory compliance guarantee (B35 lines 584-586).

## Summary additions and corrections

Delta target: 02-B-PLAN-SUMMARY.discovery.md. “Addition” means an explicit canonical-source detail absent from that orientation; “correction” means the canonical source narrows, supersedes, or conflicts with it.

### B-7 deltas (orientation lines 91-105)

- **DELTA-B7-01 — correction:** The orientation lists concept_decomposition plus separate phase_node_ready/step_node_ready/baby_step_ready events. The canonical source says one concept_decomposition_node action with node_kind deliberately replaces the four-variant design (B7 lines 181-188).
- **DELTA-B7-02 — correction:** research_node_stage is not a new action; existing WebResearchStep is reused with a populated stage and a rewired history reader (B7 lines 54-62).
- **DELTA-B7-03 — addition/correction:** failure_recorded uses reason/attempts from the live FailureRecord, not the originally described failure_kind/retry_count (B7 lines 43-49, 222-224).
- **DELTA-B7-04 — addition/correction:** PackageRef and TechStackDecision emitter substrates exist, but both producer paths are wire-dark and deferred. Their live shapes explicitly reject the orientation’s method/source literals (B7 lines 63-97).
- **DELTA-B7-05 — addition:** The four remaining follow-ups are phase_sized_split, decomposition_alternative, research_cache_hit_rate, and real research-budget/source-tier reads (B7 lines 99-105).
- **DELTA-B7-06 — addition:** Schema 99→100, typed-payload bijection 232→236, direct WindowHost mounting, cold-start fallback readers, and dual PlanWindow/ActivityWindow consumers are canonical implementation details (B7 lines 107-155).
- **DELTA-B7-07 — addition:** TaskSplitPayload is backward-compatibly extended with two optional fields and the agent_role XOR phase_id producer invariant (B7 lines 371-392).
- **DELTA-B7-08 — correction:** The orientation’s “14 events and nine commands” aggregate is stale against the canonical consolidation/reuse decisions; all source acceptance boxes remain unchecked despite the V2 GREEN overlay (B7 lines 327-336).

### B-16 deltas (orientation lines 192-202)

- **DELTA-B16-01 — correction:** B-16 is not simply PLANNED. The canonical rebaseline marks all acceptance rows MET except the ort stable bump, which is PENDING-UPSTREAM, and separately defers a lint extension (B16 lines 10-18, 254-276).
- **DELTA-B16-02 — addition:** Typed-error work uses one 12-variant KnowledgeError plus a narrower VectorError, with concrete BM25/vector/PageRank migrations and zero boxed-error public APIs (B16 lines 79-113).
- **DELTA-B16-03 — correction:** The unwrap policy includes an approved LazyLock<Regex> literal-static exception with targeted lint allow/comment; raw counts are not violations and file-aware audit is authoritative (B16 lines 117-127).
- **DELTA-B16-04 — correction:** Narrowing casts are REPORT-only and per-site; CAT-D is zero while 35 CAT-C sites remain categorized. The orientation’s blanket “replace narrowing casts” wording is too strong (B16 lines 156-165).
- **DELTA-B16-05 — addition:** Magic-number enforcement lives in tree-sitter plugin T2 with an explicit literal/context allowlist; shell lint only reports/cites that surface (B16 lines 140-154).
- **DELTA-B16-06 — correction:** Canonical STRICT checks are boxed errors, log imports, and waived ort RC; unwrap/casts/magic numbers are REPORT. The orientation additionally claims HashMap and direct SystemTime patterns in this CI lint, but this source does not list them (B16 lines 184-191).
- **DELTA-B16-07 — addition:** Six rule families—rand, getrandom, async-trait/native async, r2d2, glob, and async mutex selection—remain owned by B-16-lint-extension (B16 lines 193-200).
- **DELTA-B16-08 — addition:** Two originally cited audit inputs were missing and replaced with extant sources, which is material evidence provenance (B16 lines 47-61).

### B-18 deltas (orientation lines 212-222)

- **DELTA-B18-01 — addition:** Linux CPU has a 90-second hard limit as well as 60-second soft limit; the namespace also receives a minimal filesystem view and zero capabilities (B18 lines 42-65).
- **DELTA-B18-02 — addition:** Researcher networking is a dedicated bridged namespace with NETWORK_EXTRA/AF_INET, and bridge configuration is an unresolved decision (B18 lines 48-53, 195-200).
- **DELTA-B18-03 — addition:** Windows AppContainer failure uses the already-coded, explicitly weaker restricted-token/Job-object fallback (B18 lines 75-79).
- **DELTA-B18-04 — addition:** Real metrics are specifically pipe/socket RTT, steady RSS, and spawn-to-first-handshake, replacing three named fake proxies (B18 lines 109-121).
- **DELTA-B18-05 — addition:** nix/windows-sys require confirmation and clone3 versus unshare plus dependency scope are unresolved (B18 lines 181-200).
- **DELTA-B18-06 — conflict:** B-18 itself has no shipped overlay and unchecked acceptance, but B-19 later asserts its IPC run_agent_session shipped. The orientation’s PLANNED label matches B-18 alone but not the cross-file status claim (B18 lines 169-179; B19 lines 17-20).

### B-19 deltas (orientation lines 224-240)

- **DELTA-B19-01 — correction:** B-19 is largely shipped, not wholly PLANNED; seven prior items and six current-cycle adapter/runtime/schema items are explicitly marked shipped (B19 lines 22-41).
- **DELTA-B19-02 — correction:** Do not delete InProcessHookDispatcher. DECISION-A keeps it as production default and unifies behind HookDispatcher, with IPC opt-in (B19 lines 43-50).
- **DELTA-B19-03 — addition:** IpcHookAsyncAdapter, build_production_ipc_hook_dispatcher, RuntimeSsbFindingWriter, and Send+Sync trait bounds close the actual adapter/wiring gaps (B19 lines 34-41).
- **DELTA-B19-04 — addition:** MarkerReport advances schema 100→101 and the golden lock 288→298, including nine pre-existing variants (B19 lines 31-33).
- **DELTA-B19-05 — addition:** PreCompact is a 0.85 token-budget dispatch gate using PreCompactHandoffSaver and SessionDag, not just a generic handoff call (B19 lines 35-36, 122-126).
- **DELTA-B19-06 — addition:** The canonical source records blake2s-versus-blake3 spec drift and leaves the 13-versus-9 failure taxonomy to B-20 (B19 lines 52-54, 77).
- **DELTA-B19-07 — correction:** The zero-hit InProcess deterministic check, stub ban, file deletion, and related acceptance rows are stale under DECISION-A (B19 lines 190-207, 222-244).
- **DELTA-B19-08 — addition:** All four hook-site semantics and rebased current line anchors are recorded, rather than only the abstract feature list (B19 lines 11-15, 63-73).

### B-21 deltas (orientation lines 250-263)

- **DELTA-B21-01 — addition:** BabyStepFeasibilityReport, BabyStepStubResult, and BabyStepEscalation are explicit extension types omitted from the orientation (B21 lines 12-21, 90-92).
- **DELTA-B21-02 — addition:** Three failures recover differently by mode: Escalate for INTERACTIVE, Replan for AUTONOMOUS (B21 line 34).
- **DELTA-B21-03 — addition:** FeasibilityReport persists specifically to stub_results.msgpack (B21 line 70).
- **DELTA-B21-04 — addition/correction:** Exact actions are SystemProbe, FeasibilityReport, NovelStepDetected, StubRun, AlternativeExplored, and FeasibilityEscalated; started/complete were renamed, and C-22 claims schema 53→54 closure of the last two (B21 lines 93-104).
- **DELTA-B21-05 — correction:** The deterministic search for FeasibilityProbeStarted is stale after that rename (B21 lines 100-104, 132-134).
- **DELTA-B21-06 — correction:** The canonical source is status-ambiguous: all acceptance is unchecked and most file work is prospective, but its C-22 note claims an implemented event subset. “PLANNED” is safe for full acceptance but loses the partial status claim (B21 lines 100-104, 136-162).
- **DELTA-B21-07 — addition:** The source disagrees with itself between feasibility.rs and system_probe.rs/feasibility_probe.rs, and says B-36 owns live dashboard command wiring (B21 lines 78-92, 148-160).

### B-23 deltas (orientation lines 283-305)

- **DELTA-B23-01 — addition:** Stage promotion requires >40% pass and top-two selection; SWR pre-screening is optional (B23 lines 73-76).
- **DELTA-B23-02 — correction:** The orientation’s cross-cutting event names do not match the canonical wiring names ProposalGenerated/Applied/Rejected, MsiUpdate, DataTableGrew, and L3Proposal. The canonical file also conflicts internally between ten named and eleven counted events (B23 lines 145-154, 213-215).
- **DELTA-B23-03 — addition:** ProposalsPanel is behind a feature flag and V2 must preserve five existing stub command names one-to-one (B23 lines 87-89, 216-221).
- **DELTA-B23-04 — addition:** The self-mod loop is reachable from autonomous mode and an optional nightly cron; L3 remains human-only despite this scheduling (B23 lines 85-89, 155-160, 222).
- **DELTA-B23-05 — correction:** The source’s acceptance expands the original 42-test target to 54+ after OSC/REG/EVO and remains entirely unchecked (B23 lines 173-203).
- **DELTA-B23-06 — addition:** OSC freezes differing keys for exactly five pipelines; REG excludes never-passed persistent failures; EVO uses adjacent-pair evolved_count (B23 lines 91-118).

### B-29.5 deltas (orientation lines 382-391)

- **DELTA-B29S-01 — correction:** The canonical file defines scale_p99_budget_contract plus a separate symbol_graph_determinism test. It does not define the orientation’s scale_resume_identity or scale_long_tail_memory contracts (B29.5 lines 258-289, 384-393).
- **DELTA-B29S-02 — addition:** Fixture acquisition is an idempotent shallow-clone script under configurable external storage with pinned tags and a committed inventory (B29.5 lines 64-122).
- **DELTA-B29S-03 — addition:** MemorySnapshot’s exact five fields and the final file_payload_lru <=500 bound are canonical details (B29.5 lines 167-190, 225-229).
- **DELTA-B29S-04 — addition:** Index metrics include per-language DKG edges/nodes and symbol_graph_meta; criterion output and nested scale/workload schemas are explicit artifacts (B29.5 lines 194-256).
- **DELTA-B29S-05 — correction:** “Under 500ms” and “<=500ms” conflict at equality; the orientation preserves only the inclusive form (B29.5 lines 5, 270, 345, 366).
- **DELTA-B29S-06 — addition:** Overall unresolved_edge_pct is gated <=50%, but per-language percentages are documentation-only (B29.5 lines 268-273, 376-378).
- **DELTA-B29S-07 — correction:** Release gating is not finalized; the decision record must choose all four tiers versus recommended Linux+VS Code minimum (B29.5 lines 420-423).
- **DELTA-B29S-08 — addition:** jemalloc/procfs dependency changes require explicit approval, official numbers are hardware-specific/immutable, and the benchmark is recommended nightly rather than per commit (B29.5 lines 381-382, 412-435).
- **DELTA-B29S-09 — addition:** The source’s fixture DB check contains an unexplained duplicated .elai_cc path segment (B29.5 lines 331-335).
- **DELTA-B29S-10 — correction:** No metrics are claimed measured and all acceptance is unchecked; the source is a planned evidence gate, not proof that B-29 already scales (B29.5 lines 357-382).

### B-35 deltas (orientation lines 468-481)

- **DELTA-B35-01 — addition:** TOML versus JSON remains a decision, with TOML primary and optional JSON fallback recommended rather than finalized (B35 lines 87-98, 540-545).
- **DELTA-B35-02 — addition:** The exact ArtifactSpec, ArtifactChunk, ArtifactIndexState, and closed ArtifactStatus schemas are absent from the orientation (B35 lines 137-168).
- **DELTA-B35-03 — correction:** The orientation names ProjectMetadataStore and says it is idempotent. The canonical source only says “project metadata store”; idempotent behavior is evidenced by unchanged-file skip tests, not by that exact type identifier (B35 lines 173-192, 471-476).
- **DELTA-B35-04 — addition/conflict:** name is said to be a collection suffix, yet storage is specified as one context_{project_id} collection with artifact_name rows (B35 lines 127-129, 193-200).
- **DELTA-B35-05 — addition:** LockScope::Index versus LockScope::Artifacts is unresolved, with shared scope recommended due to embedding-provider rate limits (B35 lines 210-215, 549-551).
- **DELTA-B35-06 — addition:** Dispatcher search defaults limit to 10, and the storage combines HNSW vector plus Tantivy BM25 for RRF (B35 lines 193-200, 219-247).
- **DELTA-B35-07 — conflict:** The missing-path done check requires a failed list in ArtifactsIndexed, but the declared event payload has no failed field (B35 lines 303-307, 419-423).
- **DELTA-B35-08 — conflict:** The source gives project-prefixed Tauri signatures and later short dispatcher-matching command names; the orientation lists only dispatcher tools and misses the command-name conflict (B35 lines 297-301, 520-526).
- **DELTA-B35-09 — addition:** ActivityFeed event visibility is not mapped to V2 orbit terminology, while ArtifactsPanel mounts in Workspace or KnowledgeDrawer (B35 lines 303-308, 527-531).
- **DELTA-B35-10 — addition:** The source defines detailed manifest, chunk, staleness, search, grounding, briefing, UI, and isolation contract tests (B35 lines 459-495).
- **DELTA-B35-11 — addition:** The grounding check contains an unreconciled .elai_cc/.elai/sessions path despite B-1 prerequisite (B35 lines 401-407).
- **DELTA-B35-12 — correction:** Compliance documents are only a future grounding foundation; no SOC2/HIPAA/GDPR compliance guarantee is promised (B35 lines 584-586).
- **DELTA-B35-13 — correction:** B-35 is wholly planned in this source: no shipped overlay and all acceptance rows unchecked (B35 lines 431-457).

## Zero-omission and zero-duplicate audit

### Atomic counts

| Source | Full line range read | Source atoms | Contiguous identifier span |
|---|---:|---:|---|
| B-7 | 1-399 | 33 | B7-001–B7-033 |
| B-16 | 1-280 | 24 | B16-001–B16-024 |
| B-18 | 1-203 | 26 | B18-001–B18-026 |
| B-19 | 1-254 | 25 | B19-001–B19-025 |
| B-21 | 1-166 | 24 | B21-001–B21-024 |
| B-23 | 1-227 | 36 | B23-001–B23-036 |
| B-29.5 | 1-461 | 42 | B29S-001–B29S-042 |
| B-35 | 1-596 | 68 | B35-001–B35-068 |
| **Total** | **2,586 lines** | **278** | **8 unique source ledgers** |

- Source-atom classification is **241 [E] + 16 [A] + 21 [C] = 278**. No source-only behavior required an inferred label; inference is confined to the interaction ledger.
- The interaction ledger adds **22 atoms**: 18 [E], three [I], and one [C]. Thus the feature/behavior plus interaction total is **300 discovery atoms**: 259 [E], 16 [A], 22 [C], and three [I].
- The orientation comparison contains **66 unique delta records**: B-7 eight, B-16 eight, B-18 six, B-19 eight, B-21 seven, B-23 six, B-29.5 ten, and B-35 thirteen. Delta records are comparison evidence and are not double-counted as source feature atoms.

### Omission audit

- **Manifest completeness:** all eight and only the eight DEEP-B-006 originals were processed. Their unique-path count, hashes, per-file lines/bytes, and 2,586-line/118,996-byte aggregate match the manifest exactly.
- **Full-read completeness:** each source was read from line 1 through its final line. The ledgers cover objective/status/context, tasks, data/payload schemas, wiring contracts, reachability, stub bans, observable effects, deterministic checks, acceptance gates, files/artifact lifecycle, decision questions, dependencies, and next/deferred work.
- **Requested-axis completeness:** the traceability matrix explicitly maps every source into capabilities/surfaces, workflows/states, schemas/lifecycle, config/defaults, events/integrations, security/privacy/compliance, recovery, determinism/concurrency, evidence/tests, and constraints/status.
- **Status fidelity:** “shipped,” “planned,” “pending,” and “deferred” are preserved as source claims. Unchecked acceptance, stale plan bodies, renamed implementation symbols, wire-dark producers, unresolved choices, and cross-file schema/status conflicts were not silently normalized.
- **Scope fidelity:** no other numbered/index B-plan original was inspected. DEEP-SOURCE-MANIFEST.md supplied scope/fingerprints and 02-B-PLAN-SUMMARY.discovery.md supplied orientation/delta context only.

### Duplicate audit

- All 278 source IDs, 22 interaction IDs, and 66 delta IDs are unique; each numbered sequence is gap-free.
- Repeated task descriptions in status overlays, Tasks, WIRING_CONTRACT, DETERMINISTIC_CHECK, ACCEPTANCE, and Files touched were merged into one originating atom. Later mentions add only status, evidence, conflict, or lifecycle information.
- Consolidated/reused identifiers are intentionally single-counted: B-7 concept_decomposition_node replaces four proposed node actions; WebResearchStep carries research stages; TaskSplitPayload is extended rather than forked; B-19 HookDispatcher unifies two implementations without deleting the in-process one; B-35 one dispatcher tool set is not recounted as four Tauri commands where names conflict.
- No manifest path is duplicated and no output was written outside this discovery file at completion.
