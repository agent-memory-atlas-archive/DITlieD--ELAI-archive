## System: C — Verification gap, testing trust, code completeness, and paper-grounded extensions (C-0..C-59)

Family C closes producer/viewer gaps, dashboard wiring, determinism/trust tiers, pipeline modes, and paper-grounded extensions through public benchmarks. Plans remain PLANNED unless status evidence records shipped (C-5). Superseded losers stay as notes under newer-plan precedence.

Source aggregate: `docs/thefinalglance/feature-discovery/aggregates/C-AGGREGATE.md` (3339 atoms; DEEP-C-001..006 + 03-C-SUMMARY_INDEX).

### C-0: Session index, waves, and closure map

**Status**: planned (index/status manifest only; not an implementation session)
**Anchors**: ledgers C-001 | sample anchors: C0:L26-29; C0:L43-46; C0:L48-51; C0:L53-56; C0:L58-61; C0:L83-88 | 86 deep atoms, 0 summary atoms | IDs: C-001/C0-004, C-001/C0-007, C-001/C0-008, C-001/C0-009, C-001/C0-010, …
**Composes with**: C-1, C-2, C-3, C-4, C-6, C-7, C-8, C-9, C-10, C-11, C-13, C-16, C-17, C-19, C-22, C-23, C-24, C-25, C-26, C-30, C-31, C-32, C-33, C-40, C-54, C-57, C-58, C-59. IX: C-001/C0-031 → C-17, C-24, C-30; C-001/C0-027 → C-10, C-25, C-26; C-001/C0-020 → C-11, C-19; C-001/C0-034 → C-22, C-30, C-33; C-001/C0-059 → C-54, C-58, C-59; C-001/C0-057 → C-54, C-57

- Superseded: (C-001/SUP-C0-061) NEWER-PLAN PRECEDENCE: C-33 description is target state relative to older C plan(s) C-1, C-5 for this conflict. Losing description retained at C-001/C0-061. Conflict: [E][conflict][Δ+] The statistics claim 59 slots: 52 CPU/cloud, six RunPod, one external; 59 C-N files authored; and all C-1..C-33 executable. Yet the Sessions section contains no C-5 row, so only 58 session bullets are present. C-5's features/status cannot be recovered from this source. (C0:L17-335,
- Superseded: (C-001/SUP-C22-025) NEWER-PLAN PRECEDENCE: C-23 description is target state relative to older C plan(s) C-0, C-21 for this conflict. Losing description retained at C-001/C22-025. Conflict: [E][conflict][Δ+] The parallel-safety explanation says C-21 touches session_dag and C-23 touches cloud hardening, which disagrees with C-0's C-21 self-modification and C-23 SessionDag UI descriptions. File-overlap claims need revalidation. (C22:L8-12; C0:L120-135)
- Superseded: (C-001/SUP-C30-029) NEWER-PLAN PRECEDENCE: C-15 description is target state relative to older C plan(s) C-0 for this conflict. Losing description retained at C-001/C30-029. Conflict: [E][conflict][Δ+] OracleAnswerCache explicitly uses std HashMap in production, while C-0 says C-15 bans production HashMap in favor of FxHashMap. This cross-plan type conflict must be resolved before implementation. (C30:L114-124; C0:L78-81)
- Superseded: (C-001/SUP-A-001) NEWER-PLAN PRECEDENCE: C-5 description is target state relative to older C plan(s) C-0 for this conflict. Losing description retained at C-001/A-001. Conflict: [conflict] C-0 claims 59 slots/files but omits C-5 from Sessions; do not invent its plan from closure-map shorthand. (C0:L17-349)
- Superseded: (C-001/SUP-A-014) NEWER-PLAN PRECEDENCE: C-30 description is target state relative to older C plan(s) C-0, C-15 for this conflict. Losing description retained at C-001/A-014. Conflict: [conflict] C-30's std HashMap violates C-0/C-15's production FxHashMap policy. (C30:L114-124; C0:L78-81)
- [status] (C-001/C0-004) C-6 is a prerequisite-free dashboard-only PLANNED token/font/sidebar pass: align background, teal, and text tokens, fix fonts, shrink Sidebar 64→48px, and verify computed CSS after a cloud task. (C0:L26-29)
- [status] (C-001/C0-007) C-3 is PLANNED after C-1 and adds get_query_trace_recent, get_pagerank_top, get_freshness_state, get_canary_drift, get_csq_log, get_feasibility_log, and get_agent_grc_config; KnowledgeDrawer and ReasoningDrawer must show non-empty live data. (C0:L43-46)
- [status] (C-001/C0-008) C-4 is PLANNED after C-1 and wires live agents, cloud spend, phase/baby-step/pipeline/progress/recent-action, runtime-list, and set_role_runtime commands; MonitorWindow must show at least one non-IDLE agent and non-zero spend. (C0:L48-51)
- [status] (C-001/C0-009) C-7 is PLANNED after C-6 and C-2: sweep seven Svelte files from 11px to 10px, add Stop/Ctrl+Enter/Ctrl+Shift+A/R, responsive breakpoints, ExploreWindow, recovery history, and per-hunk review controls; its smoke requires live streaming controls and BubbleUpStrip events. (C0:L53-56)
- [status] (C-001/C0-010) C-8 is PLANNED after C-1/C-2 and activates AmbiguityScorer, CosmosWorker::tick, TOOL_INDEX_ARTIFACTS, a real CrossEncoderReranker SsbNotifier, ONNX dot product in AragOrchestrator, CosmosStage, and drawer unwired flags; query-stage latencies must be non-zero and Cosmos status Fresh. (C0:L58-61)
- [status] (C-001/C0-015) C-10 is PLANNED after C-9/C-2: implement StagnationPattern/Detection, ExecutionHistory, ThinkingPersona/PersonaStrategy, GoalDriftCanary, GenerationWatchdog, AdaptiveFutilityTracker, five persona files, and a StagnationSection; three identical errors must trigger stagnation and persona-switch evidence. (C0:L83-88)
- [status] (C-001/C0-017) C-16 is PLANNED after C-2/C-8: place CosmosStage between ArtifactStage and ARagStage, schedule hourly CosmosWorker ticks, add trigger_cosmos_refresh, replace tracing with SSB rows, and add five standalone tests; a real OpenAPI research run must show refresh-started and Fresh. (C0:L95-98)
- [status] (C-001/C0-021) C-20 is PLANNED after C-9/C-13/C-17: replace placeholder metrics, wire ELAI_DEBUG_BATCH_DELAY_SECS, add elai measure, and require non-zero phase_4G runtime categories after a Sonnet 4.6 measurement run. (C0:L115-118)
- [status] (C-001/C0-023) C-22 is PLANNED after C-19/C-9: rename SystemProbeReport, add GPU UUID and ResourceDelta, production-wire DefaultFeasibilityProber, add adversarial coverage and FeasibilityPanel, reconcile event names, and populate phase_4F metrics. (C0:L125-128)
- [status] (C-001/C0-024) C-23 is PLANNED after C-3/C-4: add /session merge|close, mount BranchTree with live fork/merge/close, repair /continue handoff loading and parent ordering, and prove a context handoff can be continued. (C0:L130-135)
- [status] (C-001/C0-029) C-28 is PLANNED after C-27: integrate cargo-mutants and mutmut, MutationResult, MUTATION_SURVIVED, EquivalentMutantCache, an AUDITING→MEMORIZING mutation gate, and a MonitorWindow score gauge. (C0:L157-162)
- [status] (C-001/C0-033) C-32 is PLANNED after C-31/C-26: skeleton-first BLANK_N generation, one in-memory write, MiniCritic, AcceptAndComplete at ≥0.5, structural retries DECOMPOSE→TEMPLATE→FEW_SHOT→BEST_OF_N, deterministic bridge files, and a completeness gauge; its smoke targets ≥0.9 completeness. (C0:L181-184)
- [status] (C-001/C0-039) C-39 is PLANNED after C-35: extend the CLI/tool surface through MCP with an OpenHands-parity CI job and automatic co-process launch; --via-mcp parity must pass. (C0:L216-219)
- [status] (C-001/C0-045) C-45 is PLANNED, but its per-role CANTANTE pilot is deferred to a RunPod window; it measures contribution for C-41 routing and role consolidation and must produce confidence intervals. (C0:L246-251)
- (C-001/C0-065) B-1..B-8 closure map: B-1→C-18/C-8; B-2→C-18; B-3→C-3/C-8/C-23; B-4→C-1/C-3/C-8; B-5→C-1/C-3/C-12; B-6→C-2/C-4; B-7→C-2/C-4/C-8; B-8→C-1/C-3. (C0:L370-379)
- (C-001/C0-066) B-9..B-16 closure map: B-9→C-23; B-10→C-4; B-11→C-6/C-7; B-12→C-6/C-7; B-13→C-2/C-7; B-14→C-7; B-15→C-14/C-15; B-16→C-21. (C0:L380-387)
- (C-001/C0-067) B-17..B-24 closure map: B-17→C-2/C-8; B-18→C-11; B-19/B-20→C-13; B-21→C-22; B-22→C-9/C-10; B-23→C-21; B-24→C-7/C-8. (C0:L388-395)
- [deferred] (C-001/C0-068) B-25..B-33 closure map: B-25→deferred C-34; B-26→C-4/C-7; B-27→the otherwise absent C-5; B-28→C-4; B-29→C-24; B-30/B-31→C-17; B-32→C-18; B-33→C-24. (C0:L396-402)
- (C-001/C0-069) B-34..B-42/B-0 closure map: B-34→C-17; B-35→C-8/C-18; B-36/B-37→C-4 and B-36 also C-18; B-38→C-2/C-8; B-39→C-33; B-40/B-41→C-12 plus C-33 for B-40; B-42→C-2/C-16; B-0_INDEX→C-33. (C0:L403-412)
- (C-001/C0-070) Guide closure map: compiled pipeline→C-25; token efficiency→C-26; pipeline modes→C-30; code completeness→C-31/C-32; testing trust→C-27/C-28/C-29; anti-staleness→C-21; dashboard design→C-6/C-7; phase-map update→C-33. (C0:L413-420)
- (C-001/C0-061) The statistics claim 59 slots: 52 CPU/cloud, six RunPod, one external; 59 C-N files authored; and all C-1..C-33 executable. Yet the Sessions section contains no C-5 row, so only 58 session bullets are present. C-5's features/status cannot be recovered from this source. (C0:L17-335, L339-349)
- (C-001/C0-063) Wave 7–9 provenance claims an oracle artifact with 11 papers/22 competitor capabilities and a Discord dump with 15 more papers, but then says only 19 verified papers plus seven anchors—Devin, Aider, Cursor, Claude Code, OpenHands, mini-SWE-agent, and GPT-5.4/Composer 2. Duplicates or exclusion rules are not identified. (C0:L351-362)
- (C-001/C0-064) Verification is routed through verification/INDEX.md and verification/_themes.md and said to contain 103 reports, but the listed breakdown DONE 5 + MOSTLY_DONE 77 + PARTIAL 20 + NOT_DONE 3 totals 105; the index does not explain the arithmetic. (C0:L364-368)
- [status] (C-001/C0-001) The index was generated 2026-05-17 from 103 verification reports clustered into 25 themes; its premise says cloud mode is already proven through OAuth plus Sonnet 4.6, while each C-N is still a cohesive session that must wire a dashboard surface and prove itself with a real cloud-Sonnet acceptance run. (C0:L1-4)
- [status] (C-001/C0-002) Sessions are roughly dependency-ordered and may be parallel within a wave. The legend defines PLANNED, IN PROGRESS, DONE, DEFERRED, and NOT YET AUTHORED; an index row marked PLANNED therefore describes intended work, not shipped behavior. (C0:L6-15)
- [status] (C-001/C0-003) C-1 is PLANNED, has no prerequisite, closes B-4/B-5/B-8, adds GrcStepEmitted, GroundingSweepResult, CsqEvaluated, AuditorRubberStampRejected, HydeQuery, VectorCacheHitRate, and QueryTrace, and accepts when a cloud task emits at least five of those seven with non-null payloads. (C0:L19-24)
- [status] (C-001/C0-005) C-14 is PLANNED with no direct prerequisite: delete compact.rs (624 LOC), oauth.rs (591), and prompt.rs (744), remove banned usage.rs symbols, fix the Windows-shell determinism script, then require workspace check and entropy quarantine to pass. (C0:L31-34)
- [status] (C-001/C0-006) C-2 is PLANNED after C-1 and closes B-6/B-7/B-13/B-38/B-42/phase-4G by adding orchestration, Cosmos, ambiguity, stagnation, lateral-thinking, and bubble-up SSB coverage; its index smoke requires stagnation, Cosmos, ambiguity, and bubble-up rows. (C0:L36-41)
- [status] (C-001/C0-011) C-9 is PLANNED after C-1: define AutoAdvanceEvaluator and EscalationPolicy, production-wire TreeAutoAdvancePolicy, add per-role backend configuration, populate phase_4G metrics, and prove an autonomous FSM auto-advance without human input. (C0:L63-66)
- [status] (C-001/C0-012) C-11 is PLANNED after C-9: complete Tier-C isolation with setrlimit/prlimit64, hidepid=2, PR_CAPBSET_DROP, IpcHookDispatcher, real NSpid/AppContainer tests, and five adversarial tests; a Tier-C baby step must emit lifecycle rows without SANDBOX_VIOLATION. (C0:L68-71)
- [status] (C-001/C0-013) C-12 is PLANNED after C-1/C-2: record per-level verdicts at five navigator completion points, confirm six auditor REVIEW sections, expose live verdicts, and prove five distinct per-level audit rows. (C0:L73-76)
- [status] (C-001/C0-014) C-15 is PLANNED after C-14: migrate about 68 SystemTime::now sites to ClockProvider, add instant_now(), replace production HashMap with FxHashMap in ten files, re-enable entropy CI, and require byte-identical seeded output. (C0:L78-81)
- [status] (C-001/C0-016) C-13 is PLANNED after C-11: replace production InProcessToolDispatcher with IpcToolRegistry, install IpcHookDispatcher plus RuffPostToolUseRunner, implement AdaptiveTool for ten DAIRA markers, and squash WIP commits on AUDITING→MEMORIZING. (C0:L90-93)
- [status] (C-001/C0-018) C-17 is PLANNED after C-14/C-15: add Kotlin parsing, auto-resume and signal handlers, LockScope::Watch, Heading/Paragraph chunking, real ONNX embeddings, Oracle pre-seeding, and scale metrics; a Rust+Kotlin+Svelte research run must index .kt and yield non-zero ONNX scores. (C0:L100-103)
- [status] (C-001/C0-019) C-18 is PLANNED after C-1/C-2/C-13 and creates omitted contract/integration tests for B-2/B-29/B-32/B-33/B-35/B-36 and phases 3B/3D/3H with zero production logic changes; elai_cc_contracts must be green. (C0:L105-108)
- [status] (C-001/C0-020) C-19 is PLANNED after C-11, tests only: add all specified adversarial cases and Criterion benchmarks, then prove a cloud path-traversal attempt is blocked without a SANDBOX_VIOLATION row. (C0:L110-113)
- [status] (C-001/C0-022) C-21 is PLANNED after C-15 and links anti-staleness CI, unwrap cleanup, LOCKED_KEYS pre-commit enforcement, dependency audits, steering refresh, closed ParameterKey, ConstraintsBlock, SWE-bench fixtures, and MistakeBook BLOCK ingestion; the smoke proves refresh and commit blocking. (C0:L120-123)
- [status] (C-001/C0-025) C-24 is PLANNED after C-17: expose symbol_impact/flow/context/list, incrementally update changed-file graphs, add /graph mermaid, Cytoscape decomposition styling, ActivityWindow blast-radius links, and graph_export_dispatch; a modifying agent must self-select symbol_impact. (C0:L137-140)
- [status] (C-001/C0-026) C-25 is PLANNED after C-9: add CompiledPipeline/TemplateCrystallizer/CompiledExecutor, BackgroundTaskManager soft-timeout detach plus two events, CumulativeTokenTracker, and /pipeline list|reset; three repeated successes crystallize and the fourth run lowers token use. (C0:L142-145)
- [status] (C-001/C0-027) C-26 is PLANNED after C-10/C-25: implement ConstrainedDecodingConfig/DecodingMode, temperature and same-error RetryStrategy, navigator futility/persona switching, Sidecar/CloudGrounding wiring, a fixture-path fix, and three EBNF grammars; schema-valid JSON and three-error switching are gates. (C0:L147-150)
- [status] (C-001/C0-028) C-27 is PLANNED after C-15/C-17: add hardware-locked DeterminismConfig, SeededRng, blake3 prefix cache, DETERMINISM_DRIFT, WiringContract and three auditors/runners, WIRING_EVIDENCE/VIOLATION, elai wiring audit, and CI; known stubs must be caught. (C0:L152-155)
- [status] (C-001/C0-030) C-29 is PLANNED after C-27/C-28: add IntentPair, MR_identity/MR_inverse/MR_permutation_invariance, INTENT_CONFIRMED/MR_VIOLATED at schema 47, a three-pair PLANNING→TESTING_FIRST gate, MetamorphicTestRunner, and a PlanWindow editor. (C0:L164-167)
- [status] (C-001/C0-031) C-30 is PLANNED after C-17/C-24: implement surgical planning/types/DKG traversal, tiered Oracle answer caching, four DKG tools, project signals, LOC gates, and research.vram_tiers; a surgical cloud task must produce an AffectedSet spanning at most three files. (C0:L169-174)
- [status] (C-001/C0-032) C-31 is PLANNED after C-13/C-9: define FilePlan/FileSpec/FileModSpec/FunctionSpec, add FILE_PLAN as the ninth BabyStepSpec block, track lines with TOCTOU protection, record T1/T2 BLOCK mistakes, and guard per-step function counts. (C0:L176-179)
- [status] (C-001/C0-034) C-33 is PLANNED after C-30/C-22: author phase 3L/3J/4D docs, update B status/schema 46, reconcile REFINE_CAP and B-21 event names, add LAZY_CODE/EMPTY_BODY/FILE_SIZE_EXCEEDED, and record KEEP/STRIP for 28+ slash commands; link and wiring checks must pass. (C0:L186-189)
- [status] (C-001/C0-035) C-35 is PLANNED after the shipped MCP bridge and claims a measured 95% MCP-tax reduction while labeling downstream quality projections; it gates tool visibility/lazy schemas and requires 5–10K prompt-token savings with no more than 2pp SWE-bench loss. (C0:L191-199)
- [gate/status] (C-001/C0-036) C-36 is PLANNED after C-29, feature-gated, extends MetamorphicRelation with MUCOCO operators, and targets paper-like 15% inconsistency detection on at least one operator. (C0:L201-204)
- [status] (C-001/C0-037) C-37 is PLANNED after C-26 with a RunPod pilot: re-evaluate PER_ROLE_DECODING_DEFAULTS against structure-snowballing evidence and auto-generate pilot.md while preserving anti-rubber-stamp findings. (C0:L206-209)
- [status] (C-001/C0-038) C-38 is PLANNED after shipped B-1: prove PageRank on the production path and correct the summary's Aider repomap description from file-level to symbol-level with a diff and fresh-fetch date. (C0:L211-214)
- [status] (C-001/C0-040) C-40 is PLANNED after C-13/C-23: add a Devin-style GitHub draft-PR adapter, opt-in per project, with automated review-comment response default OFF and an honesty section in the PR body. (C0:L221-224)
- [status] (C-001/C0-041) C-41 is PLANNED after shipped Phase 4A/prompt-engineer; it formalizes skill scaffolds based on a claimed 44%→83% intent gain and requires ≥73% synthetic SciQuest accuracy. (C0:L226-229)
- [status] (C-001/C0-042) C-42 is PLANNED strictly after C-35/C-39 and before C-40/C-44: deterministic zero-trust MCP fuzzing must find no CRITICAL result over at least 200 tuples. (C0:L231-234)
- [status] (C-001/C0-043) C-43 is PLANNED as a policy/guardrail with no behavior change: freeze Phase 4H at three layers because cited anti-evidence says extra mechanisms degrade performance; an ablation must expose negative delta. (C0:L236-239)
- [status] (C-001/C0-044) C-44 is PLANNED after B-37/C-39/C-42 and gated on Composer 2 API access; it cites 73.7 SWE-bench Multilingual and $0.50/M input, adds a third backend tier without RunPod, and requires cost_accrued on every call. (C0:L241-244)
- [status] (C-001/C0-046) C-46 is PLANNED after the eight roles, with C-45 recommended: difficulty-tier routing targets 40–60% savings on easy tasks and accepts ≥30% cost reduction within ±2pp solve rate. (C0:L253-256)
- [status] (C-001/C0-047) C-47 is PLANNED after C-35/C-42: a per-project sled case base injects near-match tool hints and must reduce repeated tool errors by at least 10% over a 50-session replay. (C0:L258-261)
- [status] (C-001/C0-048) C-48 is PLANNED after memory-writer/C-42: classify memory T0 user-direct through T3 model-emitted, limiting safety-critical decisions to T0/T1; a T2 prompt-injection fixture must have no influence. (C0:L263-266)
- [status] (C-001/C0-049) C-49 is PLANNED after prompt-engineer, with C-41 recommended: accumulate a per-project successful-trace experience corpus and prove non-empty distilled fragments after ten sessions. (C0:L268-271)
- [status] (C-001/C0-050) C-50 is PLANNED after shipped Phase 4F: Contract-Coding cites 47% functional/near-perfect structural success and conservatively targets at least 40% functional and 90% structural on a Greenfield-5 fixture. (C0:L273-276)
- [status] (C-001/C0-051) C-51 is PLANNED after Phase 4F: vendor the pinned MIT Aider harness and 225 Exercism tasks as infrastructure only, dry-run five mock tasks, and refuse a full sweep without GPU; execution is deferred to C-55. (C0:L278-281)
- [status] (C-001/C0-052) C-52 is a PLANNED research spike citing Gemma 4-31B at 90.91% on NL2VC-60; it targets FSM, MemoryView, and the mutation aggregator with C-48 recommended, and accepts either three verified modules or an infeasibility note with at least one concrete blocker. (C0:L283-286)
- [status] (C-001/C0-053) C-53 is PLANNED but combines a passive NF4 upstream watch with a RunPod-dependent in-context-versus-orchestration counter-test; closure requires an upstream-watch artifact and pilot artifact. (C0:L288-291)
- [status] (C-001/C0-054) C-54 is a PLANNED RunPod flagship after C-21/C-35/C-39/C-42 and optionally C-44: run local-7B, dual-local-7B, and hybrid arms on all 500 SWE-bench Verified tasks, producing per-config scorecards, methodology, and honesty. (C0:L293-303)
- [status] (C-001/C0-055) C-55 is a PLANNED RunPod run after C-51: benchmark local-32B and hybrid over all 225 Aider problems and open a public draft PR against Aider-AI/aider through C-40. (C0:L305-308)
- [status] (C-001/C0-056) C-56 is a PLANNED RunPod evaluation after B-29/B-30/C-38 against published 83% quality, 10× token, and 2.1× tool-call figures; it requires per-repo and aggregate metrics plus a paper-baseline cross-tabulation. (C0:L310-313)
- [status] (C-001/C0-057) C-57 is a PLANNED cost-amortized OpenHands Index/SWE-Cycle/SWE Atlas run after C-54, producing per-axis scorecards and a solve-rate curve in one RunPod session. (C0:L315-318)
- [status] (C-001/C0-058) C-58 is a PLANNED RunPod AgentForge/RRK evaluation with C-45 recommended; it compares against a claimed 40.0% baseline, reports RRK p99 and quality delta, and makes a config-gated adoption decision. (C0:L320-323)
- [status] (C-001/C0-059) C-59 is the PLANNED capstone after C-54..C-58: deterministically regenerate a master report from all scorecards plus external_baselines.toml, with honesty and improvement targets; TTS is optional. (C0:L325-328)
- [deferred/status] (C-001/C0-060) C-34 alone is DEFERRED pending Empire access and configured codebase_path; the feature-gated workspace would add FactoryLineTransport, ElaiNeuron MCP, BrainManifest, EmpireEventBridge, and repair for CircularDep/HardImport/StdlibShadow, then prove connected/capability events. (C0:L330-335)
- [status] (C-001/C0-062) Effort is estimated at 6–8 hours per implementation session and 8–24 wall-clock hours per benchmark; totals are 310–415 CPU/cloud hours plus 50–150 hours and RunPod cost. Wave 9 jobs are independently runnable when hardware is online, while C-59 runs last. (C0:L293-297, L341-349)
- [status] (C-001/C0-071) Phase closure map: phase-4G→C-9/C-10/C-20; 4A→C-11/C-19; 4F→C-22/C-19/C-20; 4H→C-21; 4J remains deferred to C-34; 4K→C-4. These are intended closure assignments, not evidence that the PLANNED sessions shipped. (C0:L421-426)

### C-1: Knowledge/reasoning SSB producers and three live drawer commands

**Status**: planned
**Anchors**: ledgers C-002, SUMMARY | sample anchors: C-1:L103-L107; C-1:L271-L273; C-1:L109-L110; C-1:L150-L152; C-1:L197-L199; C-1:L112-L114; C-1:L201-L204; C-1:L116-L117; C-1:L193-L195; C-1:L119-L120; C-1:L223-L227 | 39 deep atoms, 14 summary atoms | IDs: C-002/C1-E18, C-002/C1-E19, C-002/C1-E20, C-002/C1-E21, C-002/C1-E22, …
**Composes with**: C-2, C-3, C-4, C-5, C-6, C-9, C-18, C-19, C-22, C-27, C-41, C-42, C-43, C-44, C-45, C-50, C-51, C-56, C-57, C-58. IX: C-001/X-002 → C-2, C-58; C-002/C1-E02 → C-2, C-3, C-4; C-001/C18-024 → C-18; C-001/X-011 → C-45, C-56, C-58; C-001/X-004 → C-2, C-18, C-58; C-002/I01 → C-2, C-5, C-27, C-42, C-43, C-50, C-57

- Superseded: (C-001/C22-013) The panel consumes get_feasibility_report, replaces the ReasoningDrawer placeholder, and clears d.unwired.feasibility=false. (C22:L140-143)
- Superseded: (C-001/C30-010) surgical=true dispatches SurgicalDecomposer before research_decompose, replaces ConceptDecomposition with plan steps, emits surgical_plan_created with class/file count, and leaves surgical=false unchanged. (C30:L94-100)
- Superseded: (C-001/SUP-C0-061) NEWER-PLAN PRECEDENCE: C-33 description is target state relative to older C plan(s) C-1, C-5 for this conflict. Losing description retained at C-001/C0-061. Conflict: [E][conflict][Δ+] The statistics claim 59 slots: 52 CPU/cloud, six RunPod, one external; 59 C-N files authored; and all C-1..C-33 executable. Yet the Sessions section contains no C-5 row, so only 58 session bullets are present. C-5's features/status cannot be recovered from this source. (C0:L17-335,
- Superseded: (C-001/SUP-C22-025) NEWER-PLAN PRECEDENCE: C-23 description is target state relative to older C plan(s) C-0, C-21 for this conflict. Losing description retained at C-001/C22-025. Conflict: [E][conflict][Δ+] The parallel-safety explanation says C-21 touches session_dag and C-23 touches cloud hardening, which disagrees with C-0's C-21 self-modification and C-23 SessionDag UI descriptions. File-overlap claims need revalidation. (C22:L8-12; C0:L120-135)
- Superseded: (C-001/SUP-C30-029) NEWER-PLAN PRECEDENCE: C-15 description is target state relative to older C plan(s) C-0 for this conflict. Losing description retained at C-001/C30-029. Conflict: [E][conflict][Δ+] OracleAnswerCache explicitly uses std HashMap in production, while C-0 says C-15 bans production HashMap in favor of FxHashMap. This cross-plan type conflict must be resolved before implementation. (C30:L114-124; C0:L78-81)
- Superseded: (C-001/SUP-A-001) NEWER-PLAN PRECEDENCE: C-5 description is target state relative to older C plan(s) C-0 for this conflict. Losing description retained at C-001/A-001. Conflict: [conflict] C-0 claims 59 slots/files but omits C-5 from Sessions; do not invent its plan from closure-map shorthand. (C0:L17-349)
- Superseded: (C-001/SUP-A-012) NEWER-PLAN PRECEDENCE: C-23 description is target state relative to older C plan(s) C-21, C-22 for this conflict. Losing description retained at C-001/A-012. Conflict: [conflict] C-22's parallel-file explanation misidentifies C-21/C-23 ownership, and the Tauri command path is inconsistent. (C22:L8-12, L130-179)
- Superseded: (C-001/SUP-A-014) NEWER-PLAN PRECEDENCE: C-30 description is target state relative to older C plan(s) C-0, C-15 for this conflict. Losing description retained at C-001/A-014. Conflict: [conflict] C-30's std HashMap violates C-0/C-15's production FxHashMap policy. (C30:L114-124; C0:L78-81)
- Superseded: (C-001/SUP-A-020) NEWER-PLAN PRECEDENCE: C-58 description is target state relative to older C plan(s) C-45, C-56 for this conflict. Losing description retained at C-001/A-020. Conflict: [conflict] C-58 must use the later explicit sequential/back-to-back rule despite earlier “in parallel” wording; C-56 data and C-45 attribution are conditional dependencies despite parallel/recommended labels. (C58:L8-35, L65-121)
- Superseded: (C-002/SUP-C01) NEWER-PLAN PRECEDENCE: C-4 description is target state relative to older C plan(s) C-1 for this conflict. Losing description retained at C-002/C01. Conflict: C-4 says C-1 supplies `AgentLifecycle` emitters, but C-1 defines only seven knowledge/reasoning events and never lists `AgentLifecycle`. | Evidence: [C-4:L7-L9] versus [C-1:L3-L6; C-1:L56-L72]. Dependency claim needs another producer or a corrected prerequisite.
- Superseded: (C-002/SUP-C05) NEWER-PLAN PRECEDENCE: C-27 description is target state relative to older C plan(s) C-1, C-4, C-5, C-12, C-19 for this conflict. Losing description retained at C-002/C05. Conflict: Provider mode used by multiple smoke commands conflicts with C-5's corrected CLI axis. | Evidence: C-1/C-4/C-12/C-19/C-27 use `elai /task --mode cloud`, while C-5 says `--mode` is gate mode and provider is selected by `--provider`. [C-1:L228-L232; C-4:L195-L196; C-12:L152-L155; C-19:L236-L238; C-27:
- Superseded: (C-002/SUP-C06) NEWER-PLAN PRECEDENCE: C-27 description is target state relative to older C plan(s) C-1, C-5 for this conflict. Losing description retained at C-002/C06. Conflict: SSB schema baselines cannot all be current. | Evidence: C-27 mandates 46→47, C-1 says one bump and ≥47, while C-5 reports shipped schema 103. Integration must rebase to the actual current version rather than set 47. [C-27:L61-L71; C-27:L210-L211] versus [C-1:L56-L60; C-1:L210-L222] versus [C-5:L16-L
- Superseded: (SUMMARY/SUP-S-59SLOTS) NEWER-PLAN PRECEDENCE: later coverage section (59 slots) supersedes header 35-session legend. Retained: SUMMARY/S-E0003 — The later coverage section supersedes the old count with 59 session slots: 52 CPU/cloud sessions (C-1 through C-33 and C-35 through C-53), six RunPod-dependent sessions (C-54 through C-59), and one externally blocked session (C-34). [L578-L587]
- (C-002/C1-E18) Preserve `tracing::debug` and add a `QueryTrace` SSB write after it; `QueryPipeline` gains optional notifier state threaded from the live CLI construction path. Its contract expects exactly one trace per minimal query, including nonnegative result count and latency. [C-1:L103-L107; C-1:L271-L273]
- [security] (C-002/C1-E19) Replace the production `NoopSsbNotifier` default in `CrossEncoderReranker::new` with the real live-CLI SSB handle; test-only/no-comment occurrences are the only tolerated remnants. [C-1:L109-L110; C-1:L150-L152; C-1:L197-L199]
- (C-002/C1-E20) `get_grounding_sweep` must read real `GroundingSweepResult` rows through `SsbViewer`, remove hardcoded zero output, and set `unwired.groundingSweep=false`. [C-1:L112-L114; C-1:L201-L204]
- (C-002/C1-E21) `get_query_trace_recent` must return real `query_trace` rows rather than `json!([])` and set `unwired.recentTraces=false`. [C-1:L116-L117; C-1:L193-L195]
- (C-002/C1-E22) `get_csq_pending` must return `csq_evaluated` rows filtered to `blocked=true` and set `unwired.csqPending=false`. [C-1:L119-L120]
- [gate] (C-002/C1-E27) UI acceptance requires a non-`task_runner` agent in ReasoningDrawer, at least one recent trace in KnowledgeDrawer, and all three relevant `unwired` flags false. [C-1:L223-L227]
- [status] (C-002/C1-E01) Objective: a live cloud-Sonnet task must persist real, non-null SSB payload rows for exactly seven action names: `grc_step_emitted`, `grounding_sweep_result`, `csq_evaluated`, `auditor_rubber_stamp_rejected`, `hyde_query`, `vector_cache_hit_rate`, and `query_trace`; estimated effort is one 6-8-hour session. [C-1:L1-L7]
- (C-002/C1-E02) Prerequisites are shipped B-4, B-5, and B-8; the work is declared parallel-safe with C-2, C-3, and C-4 because the latter are described as dashboard-side and not changing `ssb.rs`/`task_runner.rs` emit logic. [C-1:L8-L13]
- [status] (C-002/C1-E03) The source records the starting gaps: five reasoning variants absent; `emit_reasoning` uses synthetic `agent_id="task_runner"` and zero budget/elapsed values; HyDE/cache events do not exist; `pipeline.rs` only traces `query_trace`; the reranker defaults to `NoopSsbNotifier`; CSQ and rubber-stamp paths do not write SSB; three dashboard commands/flags are stubs. [C-1:L15-L37]
- [lifecycle] (C-002/C1-E04) Add `SsbActionType::{GrcStepEmitted,GroundingSweepResult,CsqEvaluated,AuditorRubberStampRejected,HydeQuery,VectorCacheHitRate,QueryTrace}`, all `as_sql_str`/`from_str` arms, payload arms, SQL serialization, and exactly one atomic `SCHEMA_VERSION` increment for the family. [C-1:L56-L72; C-1:L210-L222]
- (C-002/C1-E05) `GrcStepEmittedPayload` fields are `agent_id`, `topic`, `step_type`, `grounding_level`, `confidence:f64`, `budget_remaining:u32`, and `elapsed_ms:u64`. [C-1:L61-L64]
- (C-002/C1-E06) `GroundingSweepResultPayload` fields are `ungrounded:u32`, `verify_needed:u32`, `grounded:u32`, and `top_claim:Option<String>`. [C-1:L63-L65]
- (C-002/C1-E07) `CsqEvaluatedPayload` fields are `severity:String`, `guard_present:bool`, `blocked:bool`, and `check_count:u32`. [C-1:L65-L66]
- (C-002/C1-E08) `AuditorRubberStampRejectedPayload` fields are `role:String`, `finding_count:u32`, and `expected_min:u32`. [C-1:L66-L67]
- (C-002/C1-E09) `HydeQueryPayload` fields are `query_hash:String`, `hypothesis_len:u32`, `cache_hit:bool`, and `latency_ms:u64`; the query itself is represented by `blake3_hex(query)`, not raw text. [C-1:L67-L68; C-1:L93-L96]
- (C-002/C1-E10) `VectorCacheHitRatePayload` fields are `session_id:String`, `hits:u32`, `misses:u32`, and `hit_rate_pct:f64`. [C-1:L69-L70]
- (C-002/C1-E11) `QueryTracePayload` fields are `query_id:String`, `tier:String`, `stage_count:u32`, `result_count:u32`, and `total_latency_ms:u64`. [C-1:L70-L72]
- (C-002/C1-E12) Refactor `emit_reasoning` to accept `grc: Option<&Arc<AsyncMutex<PipelineGrc>>>`; when present, extract real budget, agent, topic, and elapsed time, emit `GrcStepEmitted` in addition to the existing `ReasoningTrace`, and update all 11 call sites to pass `resolved.pipeline_grc.as_ref()`. [C-1:L74-L78]
- (C-002/C1-E13) At the former `TODO(B-future)` around `task_runner.rs:5119`, emit `CsqEvaluated` from `severity`, `guard_present`, `hook_pre_blocked`, and `check_count` via `writer.write_payload()`. [C-1:L80-L82]
- (C-002/C1-E14) Immediately after `ground_plan`, emit `GroundingSweepResult` populated from `ungrounded.len()`, `grounded.len()`, `verify.len()`, and `ungrounded.first().cloned()`. [C-1:L84-L86]
- [failure] (C-002/C1-E15) In the `GrcError::InsufficientFindings` branch, emit `AuditorRubberStampRejected`; inject an optional real SSB writer into `BasicGrcEngine::advance`, preferring an existing side-channel/channel if available. [C-1:L88-L91]
- (C-002/C1-E16) `HydeStage::run` records cache hit and latency and emits one `HydeQuery` via an injected `Arc<dyn SsbNotifier>`; the contract requires the second identical query to report `cache_hit=true`. [C-1:L93-L96; C-1:L265-L267]
- [lifecycle] (C-002/C1-E17) `VectorCache` gains hit/miss counters and `flush_hit_rate_ssb(session_id, writer)`, which emits the rate and resets counters; `QueryPipeline::run_query` invokes it after query completion, and tests accept `0.01` error from `hits/(hits+misses)*100`. [C-1:L98-L101; C-1:L268-L270]
- (C-002/C1-E23) `workflow/reasoning/C-1-decisions.md` records the notifier injection choice for HyDE/cache, the engine writer path, and final schema version. [C-1:L122-L124]
- (C-002/C1-E24) The end-to-end entrypoint is `elai /task --mode cloud`; an SQL query over `.elai/sessions/{id}/ssb.db` must show at least five distinct new types, at least one row per returned type, and non-null payloads. `grc_step_emitted` and `query_trace` are mandatory every run; rubber-stamp rejection is conditional. [C-1:L148-L161; C-1:L228-L254]
- [gate] (C-002/C1-E25) Deterministic gates require ≥21 enum/string hits, one `emit_reasoning` GRC signature, CSQ and grounding emits, engine/HyDE/query-trace writes, no production `NoopSsbNotifier`, no zero grounding stub, clean workspace check, and schema ≥47. [C-1:L164-L213]
- [gate] (C-002/C1-E26) Required test changes are five new contract files (`ssb_new_variants_contract.rs`, `emit_reasoning_grc_contract.rs`, `hyde_ssb_emit_contract.rs`, `vector_cache_hit_rate_contract.rs`, `query_trace_ssb_contract.rs`) plus extensions to `reasoning/tests/agent_config.rs` and `production_agent_rpc_contract.rs`; package tests for orchestrator, knowledge, and reasoning and the workspace check must pass. [C-1:L215-L222; C-1:L257-L278]
- [non-goal] (C-002/C1-E28) Explicitly out of scope: C-2 HyDE preview and `CsqBlockCard`; C-3 PageRank/freshness/canary commands; C-4 recall metrics, DEEP-vs-STANDARD MRR, and stage latency; major replanning on blocked feasibility; `verify_claim` prefetch wiring; and cross-encoder TOML documentation. [C-1:L280-L288]
- (C-002/SD-C-1-01) Addition: The index names the seven events but omits all seven exact payload schemas, SQL serialization/bijection work, optional `PipelineGrc` signature, 11 caller updates, dual `ReasoningTrace` emission, cache counter reset, query hashing, and conditional engine-writer choice. [C-1:L56-L110]
- (C-002/SD-C-1-02) Addition: The live-query semantics include `blocked=true` for CSQ, real viewer rows for trace/grounding, and three exact `unwired` flags. [C-1:L112-L120]
- (C-002/SD-C-1-03) Correction/precision: “Five contract tests” means five newly added contract files; the source also requires two existing-suite extensions, for seven named test changes. [C-1:L257-L278]
- (C-002/SD-C-1-04) Addition: Smoke distinguishes two always-required events from a conditional rubber-stamp event and requires real positive GRC budget; the index's ≥5/non-null shorthand omits this. [C-1:L228-L254]
- [non-goal] (C-002/SD-C-1-05) Addition: Seven explicit non-goals prevent mistakenly absorbing C-2/C-3/C-4 and feasibility/documentation work. [C-1:L280-L288]
- (SUMMARY/S-E0030) Wires get_grounding_sweep, get_query_trace_recent, and get_csq_pending Tauri commands to live data. [L35]
- (SUMMARY/S-E0023) Closes B-4, B-5, and B-8; has no prerequisites because those B plans are described as already shipped; is parallel-safe with C-2, C-3, and C-4. [L28-L31]
- (SUMMARY/S-E0024) Adds seven SsbActionType variants and payloads: GrcStepEmitted, GroundingSweepResult, CsqEvaluated, AuditorRubberStampRejected, HydeQuery, VectorCacheHitRate, and QueryTrace. [L32]
- (SUMMARY/S-E0025) Changes emit_reasoning to accept PipelineGrc. [L33]
- (SUMMARY/S-E0026) Emits CsqEvaluated from task_runner.rs at the stated source location 5119. [L33]
- (SUMMARY/S-E0027) Emits GroundingSweepResult after ground_plan. [L33]
- (SUMMARY/S-E0028) Emits AuditorRubberStampRejected from reasoning/engine.rs at the stated source location 340. [L33]
- (SUMMARY/S-E0029) Threads a real shared SSB handle into HydeStage, VectorCache, QueryPipeline, and CrossEncoderReranker, replacing NoopSsbNotifier. [L34]
- (SUMMARY/S-E0031) Clears three unwired flags corresponding to those surfaces. [L35]
- (SUMMARY/S-E0032) Increments SCHEMA_VERSION by exactly one for this event-family addition. [L36]
- (SUMMARY/S-E0033) Named touch points are ssb.rs, task_runner.rs, reasoning/engine.rs, hyde.rs, vector/cache.rs, pipeline.rs, reranker.rs, commands.rs, and drawers.svelte.ts. [L37]
- (SUMMARY/S-E0034) Requires five contract tests. [L37]
- (SUMMARY/S-E0035) Cloud smoke task: create a pure-Rust add function with documentation and a unit test; acceptance requires at least five of the seven new SSB types and non-null payloads. [L38]

### C-2: Orchestration producers (tree, Cosmos, ambiguity, stagnation, lifecycle)

**Status**: planned
**Anchors**: ledgers C-001, SUMMARY | sample anchors: C2:L1-10; C2:L12-18; C2:L20-21; C2:L23-25; C2:L116-126; C2:L128-133 | 38 deep atoms, 19 summary atoms | IDs: C-001/C2-001, C-001/C2-002, C-001/C2-003, C-001/C2-004, C-001/C2-016, …
**Composes with**: C-0, C-1, C-3, C-4, C-5, C-6, C-7, C-8, C-9, C-10, C-12, C-16, C-17, C-18, C-22, C-30, C-41, C-42, C-43, C-51, C-54, C-56, C-57, C-58, C-59. IX: C-001/X-012 → C-1, C-6, C-18, C-22, C-30, C-41, C-51, C-56, C-58, C-59; C-001/X-001 → C-0, C-1, C-3, C-4, C-7, C-8, C-10, C-16, C-58; C-001/C2-029 → C-1; C-002/I02 → C-5; C-002/I06 → C-12; C-001/C2-001 → C-1, C-3, C-4, C-9

- Superseded: (C-002/C12-E13) `get_per_level_verdicts` gains AppState, scans SSB for the task ID and action types `per_level_audit_verdict` plus `per_level_audit_cascade`, deserializes `Vec<LevelAuditVerdictJson>`, and replaces `Ok(Vec::new())`. [C-12:L98-L102]
- Superseded: (C-002/C02) C-5 shipped provider flag versus stale plan clauses. | Evidence: Status and amended Task 3 say `--provider`; `WIRING_CONTRACT`, acceptance, Files, and smoke still say `--mode cloud|hybrid|local`. The status block explicitly says do not re-implement the stale spelling, so `--provider` is canonical. [C-5:L19-L22; C-5:L110-L119] versus [C-5:L217-L229; C-5:L293-L294; C-5:L308-L308; C-5:L329-L329].
- Superseded: (C-002/C03) C-5 shipped atomic warning/cost events versus stale generic command payloads. | Evidence: Status says `WarnLimitReached` and `CostEstimate` are atomic variants; Tasks 2/6, WIRING observable, and acceptance still call them `COMMAND` rows. Status explicitly overrides. [C-5:L28-L30] versus [C-5:L101-L108; C-5:L145-L148; C-5:L239-L240; C-5:L291-L300].
- Superseded: (SUMMARY/S-E0040) Replaces CosmosWorker tracing-only behavior with persisted Cosmos SSB rows. [L45]
- Superseded: (SUMMARY/S-E0044) Replaces the hardcoded-IDLE get_agents result with live agent data. [L46]
- Superseded: (SUMMARY/S-E0045) Replaces the mock get_ambiguity_state result with live ambiguity data. [L46]
- Superseded: (C-002/SUP-C01) NEWER-PLAN PRECEDENCE: C-4 description is target state relative to older C plan(s) C-1 for this conflict. Losing description retained at C-002/C01. Conflict: C-4 says C-1 supplies `AgentLifecycle` emitters, but C-1 defines only seven knowledge/reasoning events and never lists `AgentLifecycle`. | Evidence: [C-4:L7-L9] versus [C-1:L3-L6; C-1:L56-L72]. Dependency claim needs another producer or a corrected prerequisite.
- Superseded: (C-002/SUP-C05) NEWER-PLAN PRECEDENCE: C-27 description is target state relative to older C plan(s) C-1, C-4, C-5, C-12, C-19 for this conflict. Losing description retained at C-002/C05. Conflict: Provider mode used by multiple smoke commands conflicts with C-5's corrected CLI axis. | Evidence: C-1/C-4/C-12/C-19/C-27 use `elai /task --mode cloud`, while C-5 says `--mode` is gate mode and provider is selected by `--provider`. [C-1:L228-L232; C-4:L195-L196; C-12:L152-L155; C-19:L236-L238; C-27:
- Superseded: (C-002/SUP-C06) NEWER-PLAN PRECEDENCE: C-27 description is target state relative to older C plan(s) C-1, C-5 for this conflict. Losing description retained at C-002/C06. Conflict: SSB schema baselines cannot all be current. | Evidence: C-27 mandates 46→47, C-1 says one bump and ≥47, while C-5 reports shipped schema 103. Integration must rebase to the actual current version rather than set 47. [C-27:L61-L71; C-27:L210-L211] versus [C-1:L56-L60; C-1:L210-L222] versus [C-5:L16-L
- Superseded: (C-002/SUP-C16) NEWER-PLAN PRECEDENCE: C-57 description is target state relative to older C plan(s) C-54 for this conflict. Losing description retained at C-002/C16. Conflict: C-57 prerequisite and parallel statement conflict operationally. | Evidence: C-54 must be shipped and its number exist first, yet C-57 is called parallel-safe with C-54 while runs are sequential on the same RunPod. [C-57:L8-L10]. Planning can be parallel; the actual benchmark run cannot satisfy “C-5
- Superseded: (C-002/SUP-C17) NEWER-PLAN PRECEDENCE: C-5 description is target state relative to older C plan(s) C-4 for this conflict. Losing description retained at C-002/C17. Conflict: C-4 “Next” misidentifies C-5. | Evidence: It calls C-5 a ReviewWindow diff-wiring plan, but the processed C-5 is Cloud Mode Hardening. [C-4:L295-L298] versus [C-5:L1-L5].
- Superseded: (C-002/SUP-STATUS-C02) NEWER-PLAN PRECEDENCE (status/amendment wins): C-002/C02 — C-5 shipped provider flag versus stale plan clauses. | Evidence: Status and amended Task 3 say `--provider`; `WIRING_CONTRACT`, acceptance, Files, and smoke still say `--mode cloud|hybrid|local`. The status block explicitly says do not re-implement the stale spelling, so `--provider` is canonical. [C-5:L19-L22; C-5:L110-L119] versus [C-5:L217-L229; C-5:L293-L294; C-5:L308-L308; C-5:L329-L329].
- Superseded: (C-002/SUP-STATUS-C03) NEWER-PLAN PRECEDENCE (status/amendment wins): C-002/C03 — C-5 shipped atomic warning/cost events versus stale generic command payloads. | Evidence: Status says `WarnLimitReached` and `CostEstimate` are atomic variants; Tasks 2/6, WIRING observable, and acceptance still call them `COMMAND` rows. Status explicitly overrides. [C-5:L28-L30] versus [C-5:L101-L108; C-5:L145-L148; C-5:L239-L240; C-5:L291-L300].
- [status] (C-001/C2-001) Objective: close five producer gaps—tree navigator, Cosmos worker, ambiguity scorer, autonomous stagnation detector, and agent lifecycle—so BubbleUpStrip and OrchestrationDrawer display live cloud events. C-1 is required; C-3/C-4/C-9 are parallel-safe; estimate 6–8h. (C2:L1-10)
- (C-001/C2-002) B-7 lacks PhaseNodeReady, StepNodeReady, BabyStepReady, and DagRepairEscalated definitions/emits, while B-13's BubbleUpStrip already queries bubble_up_typed but documents an empty-until-wired state. (C2:L12-18)
- [status] (C-001/C2-003) B-6 lacks AgentStatusChanged and get_agents returns eight hardcoded IDLE roles. (C2:L20-21)
- (C-001/C2-004) B-38 already defines AmbiguityScored, AmbiguityFloorFailed, and AmbiguityStreakAdvanced, but task_runner does not invoke the scorer, get_ambiguity_state returns mock:true, and the drawer has no ambiguity section. (C2:L23-25)
- (C-001/C2-016) get_agents must replace the hardcoded eight-role IDLE list with recent_agent_status_changes so live state reaches the drawer; get_ambiguity_state must replace mock:true with the three ambiguity row types. (C2:L116-126)
- (C-001/C2-017) A collapsed AmbiguitySection calls get_ambiguity_state and renders overall_score, per-dimension bars, floor-failure flags, and streak progress using GatesPanel's layout. (C2:L128-133)
- (C-001/C2-021) Reachability is cloud /task, BubbleUpStrip queries, live OrchestrationDrawer agent/ambiguity sections, and CosmosDrawer refresh; consumed anchors include C-1's variant helper, scorer/evidence fields, existing repair/bubble emit patterns, and threaded ssb_tx. (C2:L150-180)
- (C-001/C2-023) The three-function Rust CLI trigger expects ≥1 phase row, ≥3 step rows, a non-null ambiguity score, optional Cosmos refresh if its tick falls in-window, and lifecycle per spawned role; dashboard evidence is a real bubble, non-IDLE role, and numeric ambiguity. (C2:L187-197)
- (C-001/C2-029) Persisted producer rows plus latest-state reduction create two semantics on one log: history for Bubble/Cosmos/ambiguity evidence and materialized latest lifecycle for get_agents. This composition follows explicit endpoints; retention and concurrent-write ordering are not specified. (C2:L86-126, L173-180)
- (C-001/C2-005) B-42 already defines CosmosRefreshStarted/Updated/Decayed/GroundingUsed, but CosmosWorker defers SSB integration and uses tracing as a temporary channel. (C2:L27-28)
- (C-001/C2-006) Phase 4G entirely lacks StagnationSpinningDetected, StagnationOscillationDetected, LateralThinkingActivated, GoalDriftWarning, and WatchdogTimeout. (C2:L30-31)
- (C-001/C2-007) Add the four tree variants, four correspondingly named payload structs, and snake_case serialization arms in ssb.rs. (C2:L52-58)
- [status] (C-001/C2-008) Add AgentStatusChangedPayload fields agent_id, role, old_lifecycle, new_lifecycle, and timestamp_ms:u64. (C2:L60-64)
- (C-001/C2-010) Emit tree readiness at node entry through the existing ssb_tx, with node_id, label, depth, and parent_id; this makes three different navigator levels observable. (C2:L72-78)
- (C-001/C2-011) At T_DAG=10 exhaustion, emit DagRepairEscalated with attempt_count and last_error. (C2:L80-84)
- (C-001/C2-012) Replace Cosmos tracing in tick/refresh_one/record_grounding_used with persisted SsbRow inserts: RefreshStarted per artifact on tick entry, ArtifactUpdated on write success, ArtifactDecayed on detection, and GroundingUsed on use; inject ssb_tx using the BSP pattern. (C2:L86-93)
- (C-001/C2-013) Invoke AmbiguityScorer::score before GroundingSweepGate, set TransitionContext.ambiguity_score, and emit ambiguity_scored/floor_failed/streak_advanced from the gate result using existing payloads. (C2:L95-101)
- [status] (C-001/C2-015) ssb_viewer::recent_agent_status_changes reads AgentStatusChanged and reduces it to latest lifecycle per agent_id. (C2:L111-114)
- (C-001/C2-018) ssb_c2_tree_node_contract.rs verifies all four tree action strings and viewer round-trip. (C2:L135-138)
- (C-001/C2-019) ssb_c2_cosmos_emit_contract.rs inserts mock RefreshStarted/ArtifactUpdated rows and verifies correct payload retrieval. (C2:L140-143)
- [status] (C-001/C2-020) ssb_c2_agent_status_contract.rs emits three roles and verifies latest lifecycle plus Tauri reflection. (C2:L145-148)
- [security] (C-001/C2-022) Stub bans require no hardcoded IDLE, no mock:true, persisted Cosmos rather than tracing, and no unwrap in new emit paths; fallible emits use ? or logged .ok(). (C2:L181-185)
- [gate] (C-001/C2-024) Static checks cover variant hits, ≥4 Cosmos payload emits, scorer invocation, zero hardcoded/mock hits, ≥3 navigator emits, repair escalation, contract tests, and clean orchestrator/knowledge/contracts compilation. (C2:L198-231)
- [gate] (C-001/C2-026) Cloud acceptance queries six named action strings and requires at least five distinct non-null types, plus a non-placeholder bubble, at least one non-IDLE role, and a numeric—not spinner/zero-state—ambiguity score. (C2:L248-260)
- (C-001/C2-027) The named test inventory is four tree tests, three Cosmos tests, and two agent tests (nine total), all using a real temporary in-process SQLite SSB patterned after ssb_4k3_4k4_4k5_contract.rs. (C2:L262-282)
- [deferred] (C-001/C2-028) This session does not integrate ambiguity interrogation rounds, Cosmos spawn_loop/pipeline stage/refresh unblocking, anti-spiral logic/personas, PlanWindow cascade arrows, V2 phase/baby-step commands, or research/package/stack event types; those belong to C-8/C-10/C-16/C-3/C-5. (C2:L284-294)
- (C-001/C2-009) Add the five anti-spiral variants with minimal payloads patterned as pattern:String, iteration:u32, detail:String; the source does not say whether this means one common or five distinct structs. (C2:L66-70)
- [lifecycle] (C-001/C2-014) Mirror AgentLifecycle transitions at five BSP emit sites into main ssb_tx, or alternatively emit at task_runner spawn/complete/crash; the owning implementation site is deliberately left open. (C2:L103-109)
- (C-001/C2-025) Acceptance calls all 12 checked action types “new,” although the plan explicitly says four Cosmos types already exist and only ten missing enum variants are introduced. The deterministic grep also names only two of the four existing Cosmos types. (C2:L23-31, L52-70, L200-205, L233-247)
- (SUMMARY/S-E0044) Replaces the hardcoded-IDLE get_agents result with live agent data. [L46]
- (SUMMARY/S-E0045) Replaces the mock get_ambiguity_state result with live ambiguity data. [L46]
- (SUMMARY/S-E0048) Cloud smoke task: build a three-function Rust add/subtract/multiply CLI; acceptance requires at least five distinct new action types with non-null values. [L48]
- (SUMMARY/S-E0036) Closes B-6, B-7, B-13, B-38, B-42, and phase-4G; depends on C-1; is parallel-safe with C-3, C-4, and C-9. [L40-L43]
- (SUMMARY/S-E0037) Declares 12 new SsbActionType variants while naming PhaseNodeReady, StepNodeReady, BabyStepReady, DagRepairEscalated, AgentStatusChanged, StagnationSpinningDetected, StagnationOscillationDetected, LateralThinkingActivated, GoalDriftWarning, and WatchdogTimeout. [L44]
- (SUMMARY/S-E0038) Notes that Cosmos event types were already defined. [L44]
- (SUMMARY/S-E0040) Replaces CosmosWorker tracing-only behavior with persisted Cosmos SSB rows. [L45]
- (SUMMARY/S-E0041) Inserts AmbiguityScorer into task_runner as a pre-gate. [L45]
- (SUMMARY/S-E0042) Emits tree-node readiness and dag_repair_escalated events. [L45]
- [status] (SUMMARY/S-E0043) Adds ssb_viewer.recent_agent_status_changes. [L46]
- (SUMMARY/S-E0046) Adds an AmbiguitySection to OrchestrationDrawer. [L47]
- (SUMMARY/S-E0047) Requires three contract-test files. [L47]
- (SUMMARY/S-A0039) Only ten new variants are named despite the stated count of 12; the original plan must identify the other two or explain whether existing Cosmos variants were included in the count.

### C-3: Knowledge/reasoning Tauri command surface (PageRank, freshness, canary, CSQ, feasibility, GRC)

**Status**: planned
**Anchors**: ledgers C-003, SUMMARY | sample anchors: L76-L80; L89-L91; L106-L107; L109-L114; L116-L119; L121-L124 | 63 deep atoms, 10 summary atoms | IDs: C-003/E-C3-005, C-003/E-C3-007, C-003/E-C3-010, C-003/E-C3-011, C-003/E-C3-012, …
**Composes with**: C-0, C-1, C-2, C-4, C-7, C-8, C-10, C-11, C-14, C-16, C-20, C-29, C-45, C-48, C-58, C-59. IX: SUMMARY/S-E0049 → C-1, C-4; C-003/I-X-04 → C-45, C-48, C-59; C-003/E-C3-018 → C-1; C-003/I-X-03 → C-14, C-20, C-29, C-45, C-59; C-003/I-X-01 → C-7, C-10, C-29; C-003/I-C7-01 → C-7

- Superseded: (C-003/E-C14-015) The preserved draft intended deletion of 624-line `compact.rs`, 591-line `oauth.rs`, 744-line `prompt.rs`, usage stripping, a Windows script fix, and 28+ command classifications, with no prerequisite and C-15 parallel safety.
- Superseded: (C-003/E-C14-019) The original task said delete pricing helpers and leave a B-27 note; the executed structural variant relocates them instead and is authoritative.
- Superseded: (C-003/E-C14-020) The original script task proposed a possibly fictional `seeded_determinism` target; audit proved `bsp_determinism` and POSIX portability were the real issues.
- Superseded: (C-003/E-C14-013) The original entropy-quarantine cloud acceptance is replaced by workspace compilation plus zero banned-symbol matches because the dead code remains.
- Superseded: (C-003/DELTA+-C48-2) Adds the closed four-query list, write/read universality, non-optional provenance, pessimistic migration rule, exact adversarial fixture, and “extends, not replaces” constraint.
- Superseded: (SUMMARY/S-E0050) Replaces seven viewer stubs in ssb_viewer.rs. The summary names pagerank_top, query_tier_stats, freshness_state, research_budget_snapshot, source_tier_distribution, and recent_query_traces. [L54]
- Superseded: (C-003/SUP-C-C7-01) NEWER-PLAN PRECEDENCE: C-7 description is target state relative to older C plan(s) C-3, C-4 for this conflict. Losing description retained at C-003/C-C7-01. Conflict: C-7 says C-3 supplies `get_file_changes`, `accept_change`, `decline_change`, and `retry_audit`, but the complete C-3 source defines none of them and asserts its file scope is disjoint from C-4 (C-7 L78-L82/L127; C-3 L1-L8/L68-L141).
- [fallback] (C-003/E-C3-005) Replace `get_query_trace_recent`'s unconditional `json!([])` with a viewer call and default to an empty array on missing viewer/read failure; set `unwired.recentTraces=false`.
- (C-003/E-C3-007) `get_pagerank_top(n: u32)` defaults `n` to 20, calls the viewer, and is registered in `lib.rs`.
- (C-003/E-C3-010) Add and register `get_freshness_state`, delegating to `SsbViewer::freshness_state()`.
- [fallback] (C-003/E-C3-011) Add/register `get_canary_drift`; read the newest `action_type='canary_drift'` row and return `{delta_ms,baseline_ms,current_ms,drift_pct,ts}`, or `{known:false}` if absent.
- [fallback] (C-003/E-C3-012) Add/register `get_csq_log`; return the latest 50 `csq_evaluated` rows as `{csq_id,conclusion,guard_present,ts_ms}`, with `[]` as the no-row result.
- [fallback] (C-003/E-C3-013) Add/register `get_feasibility_log`, distinct from `get_feasibility_report`; return the latest 20 `feasibility_check` rows as `{task_id,verdict,score,ts_ms}`, with `[]` when absent.
- [fallback/config] (C-003/E-C3-014) Add/register `get_agent_grc_config`; read the `AgentGrcConfig` table through SSB DB or config store and return `{role,max_calls,max_chars,topic_prefix}` rows; use compiled defaults if no runtime override exists.
- [lifecycle] (C-003/E-C3-015) Add/register `get_research_budget`, wrapping the existing viewer method; keep `known:false` until a `research_budget_updated` row exists, then update the viewer to consume it.
- (C-003/E-C3-016) Add/register `get_source_tier_distribution` by wrapping the existing viewer method.
- (C-003/E-C3-017) The live consumer path is `KnowledgeDrawer.svelte` and `ReasoningDrawer.svelte` polling through `drawers.svelte.ts`.
- (C-003/E-C3-018) After C-3, `pagerank_top` may not remain an unconditional empty-vector stub; after C-1, `get_query_trace_recent` may not remain an unconditional empty array.
- [deferred] (C-003/E-C3-038) `ts-rs` generation and `MonitorWindow` feature-gating remain B-3 work; `CsqBlockCard` remains B-5 work.
- [deferred] (C-003/E-C3-039) Real `get_grounding_sweep` waits for B-8, and `ELAI_DASHBOARD_OFFLINE` gating remains B-36 work.
- (C-003/C-C3-01) The source says seven viewer methods are stubbed but enumerates only five (`pagerank_top`, `query_tier_stats`, `freshness_state`, `research_budget_snapshot`, `source_tier_distribution`). `get_query_trace_recent` is separately a command stub, not a sixth/seventh viewer method (L12-L24, L35-L38).
- (C-003/C-C3-02) The objective says six wholly missing commands, while the implementation/registration set contains eight commands. Six are explicitly called “missing entirely”; `get_pagerank_top` and `get_freshness_state` are also added but omitted from that six-command inventory (L2-L4, L26-L33, L89-L114, L150-L152).
- (C-003/DELTA!-C3-1) The summary’s “seven stubs, six visibly named” is not the original’s structure: the original names five viewer stubs and separately one Tauri command stub; the seventh remains unidentified.
- (C-003/E-C3-001) Replace hardcoded zero, empty-array, and stub returns in the knowledge/reasoning Tauri layer with live SSB or DB reads; add six wholly missing commands.
- (C-003/E-C3-002) `AppState.writer` is `Option<Arc<Ssb>>`; reads are intended to use `AppState.viewer: Option<Arc<SsbViewer>>`, with no new writer trait.
- (C-003/E-C3-003) Existing frontend `QueryTrace` fields are `id`, `query`, `mode`, `stages[]`, `total_ms`, `ts`, and `hit_count`.
- (C-003/E-C3-004) Add `SsbViewer::recent_query_traces(limit: u32) -> Result<Vec<QueryTraceRow>, SsbViewerError>`; select `action_type='query_trace'`, `ORDER BY seq DESC`, limit the result, and decode payloads with `rmp_serde`.
- [fallback] (C-003/E-C3-006) `SsbViewer::pagerank_top` queries `dkg_nodes_pagerank(symbol,path,score)`, orders descending by score, and returns an empty vector rather than an error when the table is absent.
- [gate] (C-003/E-C3-008) `query_tier_stats` aggregates the latest 500 `query_trace` rows by `QueryTrace.mode` (`fast`, `standard`, `deep`), including average `total_ms` and `sample_size`; no rows means the zero/unknown sentinel `sample_size=0`.
- [fallback] (C-003/E-C3-009) `freshness_state` reads the newest `freshness_meta`/`freshness_computed` SSB source if C-1 defines it, otherwise falls back to `ConfigManager` values `decay_rate` and `access_boost`; `known=true` only for a real source.
- [failure] (C-003/E-C3-019) Missing canary data must yield `{known:false}` and never panic; all eight new/updated commands must be registered.
- (C-003/E-C3-020) Cloud task `search the codebase for foo and explain` must produce query traces; the Tauri query returns at least one trace and `unwired.recentTraces=false`.
- (C-003/E-C3-021) Live trace acceptance additionally requires non-null `stage_latencies` matching the frontend interface.
- (C-003/E-C3-022) PageRank acceptance requires a positive scored row when the table has data and an empty non-error array when empty.
- (C-003/E-C3-023) Freshness accepts `known:false` only when no freshness source is readable; canary accepts the same sentinel only when no row exists.
- (C-003/E-C3-024) GRC acceptance requires at least eight named-role entries; budget/tier-distribution may remain `known:false` pending C-1 producers.
- [gate] (C-003/E-C3-025) `cargo check --workspace` must pass, all eight handlers must be registered, and new handlers/viewer methods may not use `.unwrap()`.
- (C-003/E-C3-026) `test_get_query_trace_recent_empty`: no SSB rows returns `[]`.
- (C-003/E-C3-027) `test_get_query_trace_recent_live`: a synthetic row round-trips with expected fields.
- (C-003/E-C3-028) `test_get_pagerank_top_empty_table`: absent table returns `[]` without panic.
- (C-003/E-C3-029) `test_get_pagerank_top_with_rows`: three rows preserve descending-score order.
- (C-003/E-C3-030) `test_get_freshness_state_stub`: absent source returns `known:false`.
- (C-003/E-C3-031) `test_get_canary_drift_no_rows`: absent rows return a `{known:false}` object.
- (C-003/E-C3-032) `test_get_query_tier_stats_no_rows`: absent rows return `sample_size:0`.
- (C-003/E-C3-033) `test_get_csq_log_empty` and `test_get_csq_log_with_rows` cover `[]` plus the exact four-field result shape.
- (C-003/E-C3-034) `test_get_feasibility_log_empty` requires a non-error empty array.
- [config] (C-003/E-C3-035) `test_get_agent_grc_config_returns_eight_roles` requires length at least eight.
- [non-goal] (C-003/E-C3-036) C-1 owns writes for `query_trace`, `csq_evaluated`, `research_budget`, and `source_tier_distribution`.
- [deferred] (C-003/E-C3-037) Per-stage latency-zero fixes, `NoopSsbNotifier` replacement, and canary `embed("fn main()")` correction remain B-4 work.
- (C-003/DELTA+-C3-2) The summary omits all response schemas, limits (500/50/20), missing-table and no-row fallbacks, GRC default path, exact contract tests, and deferred/non-goal list captured in E-C3-003 through E-C3-039.
- (C-003/DELTA+-C3-3) The original conditionally depends on C-1 defining freshness/canary/research-budget row details rather than treating every producer schema as settled.
- (C-003/A-C3-01) `recent_query_traces` is declared with `limit: u32`, but the suggested caller casts `n as usize`; the intended final parameter type is unresolved (L71-L79).
- (C-003/A-C3-02) Freshness action type is conditional (“if C-1 defines it”), and GRC ownership is a three-way DB/config/compiled-default choice without precedence beyond “runtime override first” (L100-L104, L126-L130).
- (C-003/I-C3-01) `.unwrap_or_default()` collapses absent viewer and read/deserialize failure into the same empty result, so this suggested implementation loses failure provenance (L76-L80).
- (SUMMARY/S-E0052) Adds eight Tauri commands: get_pagerank_top, get_freshness_state, get_canary_drift, get_csq_log, get_feasibility_log, get_agent_grc_config, get_research_budget, and get_source_tier_distribution. [L55]
- (SUMMARY/S-E0053) Wires get_query_trace_recent to real SSB rows. [L56]
- (SUMMARY/S-E0049) Closes B-3, B-4, and B-5; depends on C-1; is parallel-safe with C-4. [L50-L53]
- (SUMMARY/S-E0050) Replaces seven viewer stubs in ssb_viewer.rs. The summary names pagerank_top, query_tier_stats, freshness_state, research_budget_snapshot, source_tier_distribution, and recent_query_traces. [L54]
- (SUMMARY/S-E0054) Clears unwired.recentTraces. [L56]
- (SUMMARY/S-E0055) Cloud smoke task: search for foo and explain it; acceptance requires at least one recent query trace, a positive PageRank score, and at least one CSQ-log entry. [L57]
- (SUMMARY/S-A0051) The text says seven stubs but visibly names six; the seventh must be recovered from C-3.

### C-4: expose existing orchestration and cloud-spend state through real Tauri/dashboard surfaces, remove hardcoded stubs, and add contract tests

**Status**: planned
**Anchors**: ledgers C-002, SUMMARY | sample anchors: C-4:L12-L54; C-4:L81-L87; C-4:L89-L92; C-4:L247-L248; C-4:L94-L101; C-4:L249-L250; C-4:L103-L108; C-4:L251-L252; C-4:L110-L115; C-4:L253-L254 | 38 deep atoms, 12 summary atoms | IDs: C-002/C4-E03, C-002/C4-E05, C-002/C4-E06, C-002/C4-E07, C-002/C4-E08, …
**Composes with**: C-1, C-2, C-3, C-8, C-9, C-17, C-21, C-25, C-26, C-28, C-29, C-35, C-37, C-38, C-46, C-47, C-48, C-52, C-54. IX: C-004/X-013 → C-28, C-29, C-52, C-54; C-004/X-012 → C-28, C-48, C-52; C-004/X-002 → C-8, C-17; C-004/X-010 → C-35, C-37, C-46, C-54; C-004/X-016 → C-8, C-46, C-52, C-54; C-004/X-006 → C-9, C-25, C-26

- Superseded: (C-004/F-C8-004) The preserved objective replaces live-data mocks/hardcoded producer stubs across ambiguity scoring, Cosmos scheduling/events/stage integration, artifact indexing/RPC, reranking notification, A-RAG semantic scoring, pipeline latency, degradation/search-mode state, and dashboard unwired flags. [C-8 L33-L43]
- Superseded: (C-004/F-C8-017) `TOOL_INDEX_ARTIFACTS` replaces the prior `Err("full indexing deferred to Phase C")` stub, reusing the pre-existing `ArtifactsIndexed` action/payload from B-34 rather than adding a duplicate schema variant. [C-8 L155-L160]
- Superseded: (C-004/F-C17-013) `AragOrchestrator::semantic_search` replaces the norm proxy with an existing dual-vector ONNX session’s query embedding dotted against chunk embeddings, while preserving `SEMANTIC_OVERSAMPLE_FACTOR = 3`. [C-17 L107-L113]
- Superseded: (C-004/F-C21-013) Open `AllowedConfigKey::Parameter(String)` is replaced by `AllowedConfigKey::Parameter(ParameterKey)` with no string fallback. [C-21 L99-L109, L166-L177]
- Superseded: (C-004/F-C21-022) Integration consumes the anti-stale exit contract, replaces both locked/known string lists, populates `FullPipelineOutcome`, hooks promotion/DIVERGING state, invokes the steering script/rules query, and feeds `record_mistake_on_failure`. [C-21 L153-L161]
- Superseded: (C-004/D-C8-007) The ONNX path has an explicit warned norm-proxy fallback, so “replaces” is not an unconditional removal. [Summary L196; C-8 L282-L283, L424-L425, L487-L490]
- Superseded: (SUMMARY/S-E0058) Replaces three stubs: get_agents no longer returns hardcoded IDLE, get_cloud_spend no longer returns zeros, and get_executable_pipeline no longer returns an empty array. [L64]
- Superseded: (SUMMARY/S-E0060) Replaces RuntimePanel stub calls with real invocations for list_runtimes, test_runtime, and set_role_runtime. [L66]
- Superseded: (C-002/SUP-C01) NEWER-PLAN PRECEDENCE: C-4 description is target state relative to older C plan(s) C-1 for this conflict. Losing description retained at C-002/C01. Conflict: C-4 says C-1 supplies `AgentLifecycle` emitters, but C-1 defines only seven knowledge/reasoning events and never lists `AgentLifecycle`. | Evidence: [C-4:L7-L9] versus [C-1:L3-L6; C-1:L56-L72]. Dependency claim needs another producer or a corrected prerequisite.
- Superseded: (C-002/SUP-C05) NEWER-PLAN PRECEDENCE: C-27 description is target state relative to older C plan(s) C-1, C-4, C-5, C-12, C-19 for this conflict. Losing description retained at C-002/C05. Conflict: Provider mode used by multiple smoke commands conflicts with C-5's corrected CLI axis. | Evidence: C-1/C-4/C-12/C-19/C-27 use `elai /task --mode cloud`, while C-5 says `--mode` is gate mode and provider is selected by `--provider`. [C-1:L228-L232; C-4:L195-L196; C-12:L152-L155; C-19:L236-L238; C-27:
- Superseded: (C-002/SUP-C17) NEWER-PLAN PRECEDENCE: C-5 description is target state relative to older C plan(s) C-4 for this conflict. Losing description retained at C-002/C17. Conflict: C-4 “Next” misidentifies C-5. | Evidence: It calls C-5 a ReviewWindow diff-wiring plan, but the processed C-5 is Cloud Mode Hardening. [C-4:L295-L298] versus [C-5:L1-L5].
- Superseded: (C-003/SUP-C-C7-01) NEWER-PLAN PRECEDENCE: C-7 description is target state relative to older C plan(s) C-3, C-4 for this conflict. Losing description retained at C-003/C-C7-01. Conflict: C-7 says C-3 supplies `get_file_changes`, `accept_change`, `decline_change`, and `retry_audit`, but the complete C-3 source defines none of them and asserts its file scope is disjoint from C-4 (C-7 L78-L82/L127; C-3 L1-L8/L68-L141).
- [status] (C-002/C4-E03) Recorded gaps include an eight-role always-IDLE `get_agents`, missing V2 phase/baby-step commands, an empty executable pipeline, absent research history, four implemented-but-unregistered viewer methods, no project list/create service, three frontend runtime stubs despite real backend commands, and zero-valued cloud spend. [C-4:L12-L54]
- [fallback] (C-002/C4-E05) `get_agents` reads the latest 200 lifecycle rows, groups by role, takes maximum sequence per role, maps state and payload PID/heartbeat, and falls back to IDLE only for roles with no SSB row before any task. [C-4:L81-L87]
- [failure] (C-002/C4-E06) Add and register `get_phase_detail(phase_id:String)`, forwarding to the real viewer; unknown IDs return structured `PhaseNotFound`, not panic. [C-4:L89-L92; C-4:L247-L248]
- [failure] (C-002/C4-E07) Add/register `get_baby_step_detail(baby_step_id:String)` and replace unconditional NotFound by querying `baby_step_ready` or `concept_decomposition_node` by ID. If C-2 is absent, return `BabyStepNotPersisted`, not panic or stub success. [C-4:L94-L101; C-4:L249-L250]
- (C-002/C4-E08) Replace `get_executable_pipeline(session_id:String)` empty JSON with ordered phase → step → baby-step leaves reshaped from `decomposition_tree`/`concept_decomposition_node`; no rows produce an empty vector, not panic. [C-4:L103-L108; C-4:L251-L252]
- [fallback] (C-002/C4-E09) Add/register `get_research_node_history(session_id:String,node_id:String)`, filter `research_node_stage`, deserialize `ResearchNodeHistoryEntry` matching the TS shape, and return an empty vector when B-7's emitter is absent. [C-4:L110-L115; C-4:L253-L254]
- (C-002/C4-E10) Expose/register thin wrappers for `get_phase_progress`, `get_recent_actions`, `get_file_changes`, and `get_action_detail`; export the latter from `stub.ts`. [C-4:L117-L122]
- (C-002/C4-E13) Replace frontend implementations of `list_runtimes`, `test_runtime`, and `set_role_runtime` with `invoke(...)` calls using `{backend}` / `{role,backend}` payloads; mount `RuntimePanel.svelte` in Settings if absent. [C-4:L132-L138]
- (C-002/C4-E14) Add `SsbViewer::cloud_spend_summary()`: latest `cloud_spend.cost_usd` as task spend, today's summed cost as day spend, today's cache-creation/read token totals, and redaction count from `RedactionStat` when defined or zero otherwise. `get_cloud_spend` must use AppState/viewer; empty storage yields zero totals. [C-4:L140-L149; C-4:L154-L155]
- (C-002/C4-E15) The exact ten newly exposed Tauri commands are `get_phase_detail`, `get_baby_step_detail`, `get_executable_pipeline`, `get_research_node_history`, `get_phase_progress`, `get_recent_actions`, `get_file_changes`, `get_action_detail`, `list_projects`, and `create_project`; all must be in `invoke_handler`. [C-4:L161-L170; C-4:L220-L226]
- [failure] (C-002/C4-E16) `get_agents` cannot return a hardcoded literal; `get_cloud_spend` cannot return hardcoded zero fields; `get_executable_pipeline` cannot return `Ok(json!([]))`; and spend aggregation must query SSB rather than `Default::default()`. [C-4:L186-L190]
- [lifecycle] (C-002/C4-E19) Acceptance additionally requires known phase/baby-step data after producers exist, project directory/index persistence, live runtime panel data, nonzero MonitorWindow gauge, frontend-callability of all wrappers, and the five manual smoke assertions. [C-4:L241-L273]
- (C-002/SD-C-4-05) Correction: C-4's own “Next: C-5 ReviewWindow” does not match the actual C-5 source and should not orient sequencing. [C-4:L295-L298; C-5:L1-L5]
- [status] (C-002/C4-E01) Objective: expose existing orchestration and cloud-spend state through real Tauri/dashboard surfaces, remove hardcoded stubs, and add contract tests; estimated effort is one session. [C-4:L1-L6]
- [lifecycle] (C-002/C4-E02) C-1 is said to provide agent-lifecycle emitters and C-2 BabyStepSpec persistence; the work is parallel-safe with C-3 due to non-overlapping commands. [C-4:L7-L10]
- [lifecycle] (C-002/C4-E04) Add `SsbViewer::recent_agent_lifecycle_rows(limit:u32) -> Vec<AgentLifecyclePayload>` using `SELECT action_type, payload FROM ssb WHERE action_type='agent_lifecycle' ORDER BY seq DESC LIMIT ?`; empty storage returns an empty vector. [C-4:L75-L79; C-4:L151-L154]
- [fallback] (C-002/C4-E11) `list_projects` parses `<project_root>/projects.json` into `ProjectEntry {name,path,last_opened_ms}`; an absent file returns an empty vector. [C-4:L124-L127]
- [lifecycle] (C-002/C4-E12) `create_project(name,parent_path)` creates `parent_path/name`, writes a stub `elai_cc.toml`, appends to `projects.json`, and uses temp-file-plus-rename atomic replacement. [C-4:L127-L130]
- (C-002/C4-E17) Cloud todo-CRUD smoke requires: ≥1 non-IDLE agent during execution, ≥2 decomposition phases, positive task spend after the Sonnet call, `ToolUse` in recent actions, and `src/todo.rs`/`FileWrite` in file changes. [C-4:L195-L201]
- [gate] (C-002/C4-E18) `c4_tauri_commands_contract.rs` covers six empty/absent-state behaviors: lifecycle empty, spend zeros, phase error, baby-step error, pipeline empty, and research history empty; the scoped contract test and clean workspace check must pass. [C-4:L151-L159; C-4:L203-L239]
- (C-002/C4-E20) Writes span `ssb_viewer.rs`, dashboard commands/registration/API/Settings, one new contract file, and `workflow/reasoning/C-4-decisions.md`. [C-4:L275-L293]
- (C-002/SD-C-4-01) Correction: The index claims closure of B-26 and B-36. The source names B-6, B-7, B-27, phase-4K, and phase-4M as shipped/deferred surface origins; B-36 appears only as an AppState pattern, and B-26 is absent. [C-4:L14-L54]
- (C-002/SD-C-4-02) Addition: The index omits C-2 as the BabyStepSpec prerequisite, structured `PhaseNotFound`/`BabyStepNotPersisted`, research-history empty fallback, ordered pipeline shape, and project temp-file/rename behavior. [C-4:L7-L10; C-4:L89-L130]
- (C-002/SD-C-4-03) Addition: Cloud spend is not merely “live”: it has latest-task, daily sum, cache token totals, and redaction-count fallback semantics. [C-4:L140-L149]
- (C-002/SD-C-4-04) Addition: The source's smoke has five observable assertions, not only non-IDLE agent and nonzero spend. [C-4:L195-L201]
- (SUMMARY/S-E0058) Replaces three stubs: get_agents no longer returns hardcoded IDLE, get_cloud_spend no longer returns zeros, and get_executable_pipeline no longer returns an empty array. [L64]
- (SUMMARY/S-E0059) Adds ten Tauri commands: get_phase_detail, get_baby_step_detail, get_executable_pipeline, get_research_node_history, get_phase_progress, get_recent_actions, get_file_changes, get_action_detail, list_projects, and create_project. [L65]
- (SUMMARY/S-E0060) Replaces RuntimePanel stub calls with real invocations for list_runtimes, test_runtime, and set_role_runtime. [L66]
- (SUMMARY/S-E0061) Cloud smoke task: implement a four-function todo CRUD module; acceptance requires MonitorWindow to show at least one non-IDLE agent and a non-zero cloud-spend value. [L67]
- (SUMMARY/S-E0056) Closes B-6, B-7, B-26, B-27, B-36, phase-4K, and phase-4M; depends on C-1 and C-2; is parallel-safe with C-3. [L59-L62]
- [lifecycle] (SUMMARY/S-E0057) Adds recent_agent_lifecycle_rows and cloud_spend_summary to SsbViewer. [L63]

### C-5: Cloud mode hardening (provider, spend caps, route status, CloudPanel)

**Status**: shipped
**Anchors**: ledgers C-002, SUMMARY | sample anchors: C-5:L19-L22; C-5:L110-L119; C-5:L23-L25; C-5:L165-L188; C-5:L31-L37; C-5:L176-L186; C-5:L145-L148; C-5:L28-L31; C-5:L168-L172; C-5:L173-L174 | 48 deep atoms, 21 summary atoms | IDs: C-002/C5-E03, C-002/C5-E05, C-002/C5-E07, C-002/C5-E17, C-002/C5-E20, …
**Composes with**: C-1, C-4, C-6, C-7, C-9, C-16, C-27, C-28, C-29, C-32, C-34, C-35, C-36, C-44, C-49. IX: C-005/X-A01 → C-28, C-32, C-34, C-35, C-36, C-44, C-49; C-005/C35-I01 → C-35; C-005/C49-I01 → C-49; C-002/C5-E09 → C-1, C-4, C-6, C-7; C-005/X-A03 → C-16, C-28, C-35, C-49; C-005/X-A02 → C-9, C-44

- Superseded: (C-002/C02) C-5 shipped provider flag versus stale plan clauses. | Evidence: Status and amended Task 3 say `--provider`; `WIRING_CONTRACT`, acceptance, Files, and smoke still say `--mode cloud|hybrid|local`. The status block explicitly says do not re-implement the stale spelling, so `--provider` is canonical. [C-5:L19-L22; C-5:L110-L119] versus [C-5:L217-L229; C-5:L293-L294; C-5:L308-L308; C-5:L329-L329].
- Superseded: (C-002/C03) C-5 shipped atomic warning/cost events versus stale generic command payloads. | Evidence: Status says `WarnLimitReached` and `CostEstimate` are atomic variants; Tasks 2/6, WIRING observable, and acceptance still call them `COMMAND` rows. Status explicitly overrides. [C-5:L28-L30] versus [C-5:L101-L108; C-5:L145-L148; C-5:L239-L240; C-5:L291-L300].
- Superseded: (C-005/C9-E13) `AutonomyDecision::SplitTask` replaces bare count/depth fields with `split_plan: SplitTaskPlan`, and every destructuring match in `task_runner.rs` must change with it. [L117-L119]
- Superseded: (C-005/C9-E16) CLI startup replaces `autonomous_gate: None` with a real `AutonomousGate` configured from the parsed mode and otherwise-default config. [L131-L135]
- Superseded: (C-005/C16-E10) SSB emission augments rather than replaces tracing, preserving non-SSB observability. [L66-L68]
- Superseded: (C-005/C32-E06) `generate(FilePlan)` preserves signatures, replaces each body with `// BLANK_N`, and returns `TooManyBlanks` above the cap. [L67-L68]
- Superseded: (C-005/C35-E09) `server.rs` replaces full-catalog rendering with state-gated rendering and resolves a referenced gated tool through `resolve_schema_on_demand(tool_name)`. [L89-L92]
- Superseded: (C-001/SUP-C0-061) NEWER-PLAN PRECEDENCE: C-33 description is target state relative to older C plan(s) C-1, C-5 for this conflict. Losing description retained at C-001/C0-061. Conflict: [E][conflict][Δ+] The statistics claim 59 slots: 52 CPU/cloud, six RunPod, one external; 59 C-N files authored; and all C-1..C-33 executable. Yet the Sessions section contains no C-5 row, so only 58 session bullets are present. C-5's features/status cannot be recovered from this source. (C0:L17-335,
- Superseded: (C-001/SUP-A-001) NEWER-PLAN PRECEDENCE: C-5 description is target state relative to older C plan(s) C-0 for this conflict. Losing description retained at C-001/A-001. Conflict: [conflict] C-0 claims 59 slots/files but omits C-5 from Sessions; do not invent its plan from closure-map shorthand. (C0:L17-349)
- Superseded: (C-002/SUP-C05) NEWER-PLAN PRECEDENCE: C-27 description is target state relative to older C plan(s) C-1, C-4, C-5, C-12, C-19 for this conflict. Losing description retained at C-002/C05. Conflict: Provider mode used by multiple smoke commands conflicts with C-5's corrected CLI axis. | Evidence: C-1/C-4/C-12/C-19/C-27 use `elai /task --mode cloud`, while C-5 says `--mode` is gate mode and provider is selected by `--provider`. [C-1:L228-L232; C-4:L195-L196; C-12:L152-L155; C-19:L236-L238; C-27:
- Superseded: (C-002/SUP-C06) NEWER-PLAN PRECEDENCE: C-27 description is target state relative to older C plan(s) C-1, C-5 for this conflict. Losing description retained at C-002/C06. Conflict: SSB schema baselines cannot all be current. | Evidence: C-27 mandates 46→47, C-1 says one bump and ≥47, while C-5 reports shipped schema 103. Integration must rebase to the actual current version rather than set 47. [C-27:L61-L71; C-27:L210-L211] versus [C-1:L56-L60; C-1:L210-L222] versus [C-5:L16-L
- Superseded: (C-002/SUP-C16) NEWER-PLAN PRECEDENCE: C-57 description is target state relative to older C plan(s) C-54 for this conflict. Losing description retained at C-002/C16. Conflict: C-57 prerequisite and parallel statement conflict operationally. | Evidence: C-54 must be shipped and its number exist first, yet C-57 is called parallel-safe with C-54 while runs are sequential on the same RunPod. [C-57:L8-L10]. Planning can be parallel; the actual benchmark run cannot satisfy “C-5
- Superseded: (C-002/SUP-C17) NEWER-PLAN PRECEDENCE: C-5 description is target state relative to older C plan(s) C-4 for this conflict. Losing description retained at C-002/C17. Conflict: C-4 “Next” misidentifies C-5. | Evidence: It calls C-5 a ReviewWindow diff-wiring plan, but the processed C-5 is Cloud Mode Hardening. [C-4:L295-L298] versus [C-5:L1-L5].
- Superseded: (C-005/SUP-C28-C01) NEWER-PLAN PRECEDENCE: C-28 description is target state relative to older C plan(s) C-27 for this conflict. Losing description retained at C-005/C28-C01. Conflict: The original says C-28 bumps schema 46→47, while the orientation summary says Mutation shares and keeps C-27’s already-bumped 47. Cross-plan migration order cannot be resolved from this file. [L28, L119-L126]
- Superseded: (C-005/SUP-C35-C04) NEWER-PLAN PRECEDENCE: C-40 description is target state relative to older C plan(s) C-36, C-39 for this conflict. Losing description retained at C-005/C35-C04. Conflict: The original declares parallel safety only with C-36 through C-39, while the orientation summary extends that claim through C-40 without source support. [L9-L12]
- Superseded: (C-005/SUP-C36-C03) NEWER-PLAN PRECEDENCE: C-53 description is target state relative to older C plan(s) C-40 for this conflict. Losing description retained at C-005/C36-C03. Conflict: The original’s parallel-safe list stops at C-40 and warns against any relation-enum patch; the orientation summary expands safety through C-53. [L7-L10]
- Superseded: (C-002/SUP-STATUS-C02) NEWER-PLAN PRECEDENCE (status/amendment wins): C-002/C02 — C-5 shipped provider flag versus stale plan clauses. | Evidence: Status and amended Task 3 say `--provider`; `WIRING_CONTRACT`, acceptance, Files, and smoke still say `--mode cloud|hybrid|local`. The status block explicitly says do not re-implement the stale spelling, so `--provider` is canonical. [C-5:L19-L22; C-5:L110-L119] versus [C-5:L217-L229; C-5:L293-L294; C-5:L308-L308; C-5:L329-L329].
- Superseded: (C-002/SUP-STATUS-C03) NEWER-PLAN PRECEDENCE (status/amendment wins): C-002/C03 — C-5 shipped atomic warning/cost events versus stale generic command payloads. | Evidence: Status says `WarnLimitReached` and `CostEstimate` are atomic variants; Tasks 2/6, WIRING observable, and acceptance still call them `COMMAND` rows. Status explicitly overrides. [C-5:L28-L30] versus [C-5:L101-L108; C-5:L145-L148; C-5:L239-L240; C-5:L291-L300].
- [config/status] (C-002/C5-E03) The shipped CLI selector is `--provider local|cloud|hybrid`, not `--mode`; `--mode` is a separate gate-mode axis. `resolve_backend_mode` gives the CLI override precedence over TOML `[cloud].default_mode`, and absent CLI input uses that default. [C-5:L19-L22; C-5:L110-L119]
- [status] (C-002/C5-E05) The shipped component path is `dashboard-v2/src/lib/screens/CloudPanel.svelte`, not `lib/components`, mounted under Settings' Cloud section. [C-5:L23-L25; C-5:L165-L188]
- [deferred/status] (C-002/C5-E07) `get_cloud_route_status`, `CloudRouteStatus`, and three CloudPanel banners shipped. A dedicated last-five `RedactionApplied` timeline is explicitly deferred pending `get_recent_redactions` or a spend-summary extension. [C-5:L31-L37; C-5:L176-L186]
- (C-002/C5-E17) Cost estimation occurs before the first cloud call. The autonomous path does not block; the interactive path surfaces the estimate to CloudPanel. The shipped status says the event is the atomic `CostEstimate` variant. [C-5:L145-L148; C-5:L28-L31]
- (C-002/C5-E20) CloudPanel spend UI shows daily spend/cap, warn-at-80 and blocked badges, and a monthly-cap row when configured. [C-5:L168-L172]
- (C-002/C5-E21) The provider selector is a local/hybrid/cloud segmented control calling `set_execution_mode`; cloud selection is disabled when `airGapped=true`. [C-5:L173-L174]
- [gate] (C-002/C5-E22) CloudPanel shows today's aggregate redaction count; the detailed recent-event feed remains deferred. [C-5:L175-L186]
- (C-002/C5-E25) The primary observable sets daily cap `$0.05`, triggers at `$0.04`, expects one warning then route block at cap, and expects the gauge/mode selector to reflect live state. [C-5:L237-L242]
- (C-002/C5-E27) Acceptance enumerates daily/monthly errors, route-block and warning evidence, provider bridge, one escalation, constants/helper, bounded cost estimate, endpoint guard, panel mounting, tests, three smoke scenarios, and clean handling rather than panic. [C-5:L286-L318]
- [status] (C-002/C5-E28) Production changes cover cloud router, SSB schema, task runner, CLI/config, dashboard, and contracts; however, the later Files list still contains pre-amendment names/paths and is not authoritative over the status block. [C-5:L320-L339]
- (C-002/SD-C-5-01) Critical correction: The index says CLI `--mode cloud|hybrid|local`; the amended shipped source says `--provider`, with `--mode` reserved for gate mode. [C-5:L19-L22; C-5:L110-L119]
- (C-002/SD-C-5-06) Correction: CloudPanel lives under `lib/screens`; its detailed last-five redaction timeline is deferred. The index's unqualified panel/redaction-feed wording overstates completion. [C-5:L23-L37; C-5:L165-L188]
- (C-002/SD-C-5-07) Addition: Banner precedence, `get_cloud_route_status`, one-escalation halt behavior, and `airGapped` selector disabling are explicit source behaviors. [C-5:L31-L37; C-5:L120-L126; C-5:L173-L181]
- [status] (C-002/C5-E02) Shipped schema state is `SCHEMA_VERSION=103`, an atomic three-variant bump from 102 for `RouteBlocked`, `WarnLimitReached`, and `CostEstimate`; recorded bijection is 242→245 and `action_type_lock TOTAL_VARIANTS=306`. [C-5:L16-L18; C-5:L28-L30]
- [config] (C-002/C5-E04) Provider mappings are `cloud→BackendMode::Api`, `hybrid|auto→BackendMode::Hybrid`, and `local→BackendMode::Local`. [C-5:L113-L116]
- [status] (C-002/C5-E06) Redaction is the distinct `SsbActionType::RedactionApplied`, not a `REDACTION_APPLIED` command subtype; `RedactingBabyStepExecutor` writes those rows. [C-5:L26-L27; C-5:L182-L184]
- [status] (C-002/C5-E08) `workflow/reasoning/C-5-decisions.md` records Q1-Q6, DOUBLE-EMIT Option A, the `UNGROUNDED_HEURISTIC` rationale, and drift corrections. [C-5:L38-L40]
- (C-002/C5-E09) C-5 closes B-27 gaps, depends on C-1 and C-4 live SSB/spend infrastructure, is parallel-safe with C-6/C-7, and adopts their design tokens when available. Patch-R3 ToolUse translation is non-blocking; text-only smoke tasks are the fallback if absent. [C-5:L1-L12; C-5:L64-L66]
- [failure/security] (C-002/C5-E10) `CloudModeRouter::route_cloud` reads accumulated live spend before routing. At `spent_day_usd >= spend_cap_daily_usd` it returns typed `DailySpendCapExceeded`; the analogous monthly comparison returns `MonthlySpendCapExceeded`; a `route_blocked` row records the refusal. [C-5:L90-L99]
- [config] (C-002/C5-E11) `CloudConfig.warn_at_percent:Option<u8>` defaults to `Some(80)`. Warning evaluation occurs before the hard-cap check and may emit only once per calendar day when the percentage threshold is reached. [C-5:L101-L108]
- [failure/fallback] (C-002/C5-E12) If the live spend read errors, the router logs a warning and skips the cap check rather than blocking all cloud calls. [C-5:L230-L233]
- (C-002/C5-E13) In hybrid/auto mode, exhaustion of local baby-step retries emits `EscalateToCloud` and reruns once through `ExecMode::Cloud`; a second exhaustion halts and cannot escalate again. [C-5:L120-L126; C-5:L195-L198]
- [config] (C-002/C5-E14) Exact sizing constants are `BABY_STEP_LOC_LOCAL_3B=25`, `BABY_STEP_LOC_LOCAL_7B=40`, `BABY_STEP_LOC_CLOUD_SONNET=300`, and `BABY_STEP_LOC_CLOUD_OPUS=500`. [C-5:L128-L133]
- (C-002/C5-E15) `baby_step_loc_for_mode(mode,model)->u16` dispatches by execution mode plus the substrings `3b`, `7b`, `sonnet`, and `opus`, and feeds PipelineBuilder's step-size selection. [C-5:L134-L136]
- (C-002/C5-E16) `estimate_task_cost_usd(plan_skeleton,model)->(low,high)` uses `step_count * avg_tokens * price_per_token`, with `avg_tokens=baby_step_loc_for_mode*4.0`; the token/LOC rule is explicitly labeled `UNGROUNDED_HEURISTIC`. [C-5:L138-L144]
- [config/security] (C-002/C5-E18) `CloudConfig.custom_endpoint:Option<String>` overrides the Anthropic base URL; TOML wires `[cloud] custom_endpoint`. It is rejected in air-gapped mode, first use logs an info message, and the endpoint value must never enter SSB to avoid internal-URL leakage. [C-5:L150-L156; C-5:L234-L235]
- (C-002/C5-E19) `RouteBlockedPayload` is `{reason:String, spent_usd:f64, cap_usd:f64, cap_period:String}` and the action SQL string is `route_blocked`; it participates in the action-type bijection test. [C-5:L158-L163]
- (C-002/C5-E23) Banner precedence is explicit: `RouteBlocked` is latest-row driven; `WarnLimitReached` is hidden when a route-block supersedes it; `CostEstimate` displays the heuristic disclaimer inline. [C-5:L176-L181]
- (C-002/C5-E24) `cloud_ceiling_contract.rs` checks cap error, 80% warning, and `RouteBlocked` round-trip; `hybrid_escalation_contract.rs` checks escalation emission, cloud success, and one-retry halt; `cloud_loc_constants_contract.rs` checks all values and four dispatch combinations. [C-5:L190-L202]
- [gate] (C-002/C5-E26) Deterministic checks cover live cap errors, event registration, constants, config fields, `--provider`, corrected component path, endpoint support, all three contract suites, and an orchestrator hybrid-integration test. [C-5:L244-L284]
- (C-002/SD-C-5-02) Addition: C-5 is explicitly shipped, with schema 103, three atomic variants, bijection counts, and a decision log. [C-5:L14-L40]
- (C-002/SD-C-5-03) Correction: Warning and estimate are shipped `WarnLimitReached`/`CostEstimate` variants, not generic COMMAND payloads; the index omits both event types. [C-5:L28-L31]
- (C-002/SD-C-5-04) Addition: Exact LOC semantics are local 3B=25, local 7B=40, cloud Sonnet=300, cloud Opus=500; the index explicitly lacked these labels. [C-5:L128-L136]
- (C-002/SD-C-5-05) Addition: The cost formula, ungrounded heuristic label, fail-open-on-spend-read-error behavior, endpoint privacy rule, and warning-before-cap/once-per-day semantics are absent from the index. [C-5:L101-L108; C-5:L138-L156; C-5:L230-L235]
- [status] (C-002/C5-E01) The post-rebaseline status is explicitly **SHIPPED** across two sessions, with all ten task groups landed. [C-5:L14-L15]
- (SUMMARY/S-E0062) Closes B-27; depends on C-1 and C-4; is parallel-safe with C-6 and C-7, while CloudPanel adopts C-6 design tokens when available. [L69-L72]
- (SUMMARY/S-E0066) Adds CLI mode selection through --mode cloud, --mode hybrid, and --mode local. [L73]
- (SUMMARY/S-E0072) Creates CloudPanel.svelte with a spend gauge, mode selector, and redaction-event feed. [L75]
- (SUMMARY/S-E0073) Mounts CloudPanel in SettingsWindow. [L75]
- (SUMMARY/S-E0077) Smoke 3 verifies CloudPanel renders. [L77]
- (SUMMARY/S-E0063) Enforces hard daily and monthly spend ceilings. [L73]
- (SUMMARY/S-E0064) Returns typed DailySpendCapExceeded and MonthlySpendCapExceeded errors when those ceilings are crossed. [L73]
- (SUMMARY/S-E0065) Adds warn_at_percent with an 80% default. [L73]
- (SUMMARY/S-E0067) Adds one-shot HYBRID escalation retry with a cap of one escalation. [L73]
- (SUMMARY/S-E0068) Adds LOC sizing constants 25, 40, 300, and 500. Their exact semantic labels are not present in the summary. [L73]
- (SUMMARY/S-E0069) Adds estimate_task_cost_usd. [L73]
- (SUMMARY/S-E0070) Supports bring-your-own-endpoint through custom_endpoint. [L73]
- (SUMMARY/S-E0071) Adds SsbActionType RouteBlocked and its payload. [L74]
- (SUMMARY/S-E0074) Requires cloud-ceiling, hybrid-escalation, and cloud-LOC-constants contract-test files. [L76]
- (SUMMARY/S-E0075) Smoke 1 uses a 0.05 cap and 80% warning threshold and expects WARN_LIMIT_REACHED plus RouteBlocked. [L77]
- (SUMMARY/S-E0076) Smoke 2 expects EscalateToCloud and verifies the escalation cap is one. [L77]

### C-6: Dashboard design tokens, sidebar, Stop, and Ctrl+Enter

**Status**: planned
**Anchors**: ledgers C-001, SUMMARY | sample anchors: C6:L1-8; C6:L10-31; C6:L85-92; C6:L94-97; C6:L115-120, L154-158; C6:L168-208 | 30 deep atoms, 16 summary atoms | IDs: C-001/C6-001, C-001/C6-002, C-001/C6-007, C-001/C6-008, C-001/C6-011, …
**Composes with**: C-1, C-2, C-5, C-7, C-13, C-18, C-22, C-23, C-30, C-31, C-33, C-40, C-41, C-51, C-53, C-56, C-58, C-59. IX: C-006/C33-X002 → C-33; C-006/C53-X002 → C-53; C-006/C13-I002 → C-13; C-006/C31-X001 → C-31; C-006/C23-I001 → C-23; C-006/C40-X002 → C-40

- Superseded: (C-006/C13-E009) At run_subcommand.rs:149 and :221, IpcToolRegistry::new(Arc::new(validator)) replaces InProcessToolDispatcher; its existing FSM gate at ipc_registry.rs:112-146 applies automatically. [C-13 L64-L71, L166, L255-L256]
- Superseded: (C-006/C15-E010) Mechanical map migration removes the std import, imports rustc_hash::FxHashMap, replaces the type, and replaces HashMap::new with FxHashMap::default. [C-15 L89-L91]
- Superseded: (C-006/C15-E017) Injection replaces Instant::now/SystemTime::now with self.clock.instant_now/self.clock.system_now; absent clocks add Arc<dyn ClockProvider> fields and constructor args, except deserializable config types receive injection one call layer above. [C-15 L140-L145]
- Superseded: (C-006/C33-E042) B-0 replaces “SCHEMA_VERSION=26 confirmed” with current 46 and records B-6->26, B-24->30, B-34->35, and later B-38/B-42/B-41 progression through 44/45/46 with ssb.rs history as authority. [C-33 L232-L235]
- [status] (C-001/C6-001) C-6 is a one-session, backend-free dashboard pass with no prerequisites, parallel-safe with C-5/C-7; it covers tokens, fonts, 11px sites, sidebar dimensions/count, Stop, and Ctrl+Enter. (C6:L1-8)
- (C-001/C6-002) Verified deltas are wrong background/font tokens, inconsistent teal overrides, absent/mis-valued text-ghost, seven 11px component files, 64px/seven-icon Sidebar versus 48px/five-icon spec, and missing Stop/Ctrl+Enter; the source calls these cosmetic/UX only. (C6:L10-31)
- (C-001/C6-007) In ArtifactsPanel, IndexingProgressPanel, MediaValidationPanel, AntiStalenessDrawer, CosmosDrawer, KnowledgeDrawer, and MonitorWindow, inspect each 11px occurrence and change only font-size/line-height to the 10px accessibility floor—not unrelated padding/radius. (C6:L85-92)
- (C-001/C6-008) Sidebar becomes 48px only after verifying icons ≤20px and horizontal padding ≤8px per side still fit. (C6:L94-97)
- (C-001/C6-011) On a non-empty, non-streaming textarea, Ctrl+Enter calls the same submit handler; plain Enter retains newline behavior, duplicate IPC is forbidden, and the textarea gets aria-label “Press Ctrl+Enter to send.” (C6:L115-120, L154-158)
- [gate] (C-001/C6-015) Deterministic checks inspect all token groups, body font, zero 11px hits, 48px/no-64px Sidebar, Stop/Ctrl handler presence, zero Svelte errors, and the decision document. (C6:L168-208)
- (C-001/C6-017) Manual cloud smoke starts and stops a hello-world stream, sends a fresh prompt with Ctrl+Enter, and inspects computed bg-void plus JetBrains-first body font. (C6:L230-240)
- (C-001/C6-018) Component tests require data-testid=stop-button conditional rendering and exactly-once Ctrl+Enter submission for non-empty input. Reuse existing Vitest tooling or create CommandWindow.test.ts; no new dependency without human confirmation. (C6:L242-254)
- [config] (C-001/C6-003) Exact background defaults are --bg-void #0a0e12, --bg-base #0f1318, --bg-surface #151a20, and --bg-hover #1a2028; component/root overrides must be searched and eliminated. (C6:L57-63)
- [config] (C-001/C6-004) Exact teal defaults are --teal #1B7A6E, --teal-bright #24a08e, and --teal-soft rgba(27,122,110,0.25), again with a pre-write override audit. (C6:L65-69)
- [config] (C-001/C6-005) Text tokens are --text-primary #d0d7de, --text-secondary #6e7681, and --text-ghost #383e47; add ghost if absent without deleting other tokens. (C6:L71-75)
- [config] (C-001/C6-006) Set --font-sans to JetBrains Mono, monospace and --font-display to Segoe UI Variable/Segoe UI/sans-serif; display thinness belongs at usage sites, and body must consume --font-sans without a shadowing direct family. (C6:L77-83)
- [deferred] (C-001/C6-009) Do not delete two surplus icons. Name both in a SPEC-DELTA comment, retain them for parity, and record the decision in C-6-decisions.md pending C-7. (C6:L99-105)
- [failure] (C-001/C6-010) Stop is conditionally visible only while streaming, calls the already-existing abort/cancel handler, creates no IPC, uses teal-soft/text-primary, matches Send height, and disappears when streaming ends. (C6:L107-113, L161-166)
- (C-001/C6-012) Only stale B-11 deterministic-check paths are rewritten from V1 to dashboard-v2; no other B-11 section changes. (C6:L122-128)
- (C-001/C6-013) C-6-decisions.md records token provenance, font rationale, icon retention, discovered Stop handler, and plain-Enter rationale. (C6:L130-137)
- (C-001/C6-014) tokens.css is the sole source of truth, no component :root override is allowed, and CSS cascade plus CommandWindow keyboard/button events are the reachability surfaces. (C6:L138-166)
- [gate] (C-001/C6-016) Exact token values, documented icon delta, real Stop/submit behavior, corrected B-11 paths, zero new svelte-check errors, and all five rationale items are acceptance gates. (C6:L210-228)
- (C-001/C6-019) No Rust, IPC protocol, or Tauri changes; no new Stop IPC; no files outside dashboard-v2 except the targeted B-11 fix and decision doc; no icon deletion or font installation/CDN work. (C6:L256-265)
- (C-001/C6-020) If JetBrains Mono is not already bundled through index.html/font-face declarations, stop and escalate to the user rather than adding it. (C6:L264-267)
- (C-001/C6-021) The task scope names seven files, but the deterministic and acceptance grep demands zero 11px anywhere under dashboard-v2/src. Existing unrelated 11px values could fail closure despite being outside the enumerated edit set. (C6:L85-92, L184-186, L220)
- (C-001/C6-022) commands.isStreaming is explicitly a provisional identifier (“or equivalent”); exact stream, abort, and submit symbols must be discovered before wiring. (C6:L107-120, L146-150)
- (SUMMARY/S-E0082) Changes 11px text to 10px across ArtifactsPanel, IndexingProgressPanel, MediaValidationPanel, AntiStalenessDrawer, CosmosDrawer, KnowledgeDrawer, and MonitorWindow. [L84]
- (SUMMARY/S-E0083) Shrinks the sidebar from 64px to 48px. [L85]
- (SUMMARY/S-E0084) Documents the sidebar icon-count delta: five in the specification versus seven currently. [L85]
- (SUMMARY/S-E0086) Adds Ctrl+Enter in CommandWindow and connects it to the existing submit handler. [L86]
- (SUMMARY/S-E0088) Manual smoke verifies computed CSS tokens and tests Stop and Ctrl+Enter during an active stream. [L88]
- (SUMMARY/S-E0078) Closes B-11, B-12, B-14, and guide/dashboard-design; has no prerequisites because it is dashboard-only; is parallel-safe with C-5 and C-7. [L79-L82]
- (SUMMARY/S-E0079) Sets background tokens: --bg-void to #0a0e12, --bg-base to #0f1318, --bg-surface to #151a20, and --bg-hover to #1a2028. [L83]
- (SUMMARY/S-E0080) Aligns the teal palette and the --text-primary, --text-secondary, and --text-ghost tokens. [L83]
- (SUMMARY/S-E0081) Sets --font-sans to JetBrains Mono. [L83]
- (SUMMARY/S-E0085) Shows a Stop button in CommandWindow while streaming and connects it to the existing stop handler. [L86]
- (SUMMARY/S-E0087) Corrects B-11 DETERMINISTIC_CHECK paths to point at dashboard-v2. [L87]

### C-7: ExploreWindow, recovery modal, review hunk controls, responsive layout

**Status**: planned
**Anchors**: ledgers C-003, SUMMARY | sample anchors: L40-L47, L109-L123; L44-L48; L50-L55; L56-L60; L61-L64; L66-L70, L135-L139 | 58 deep atoms, 26 summary atoms | IDs: C-003/E-C7-001, C-003/E-C7-002, C-003/E-C7-004, C-003/E-C7-005, C-003/E-C7-006, …
**Composes with**: C-0, C-1, C-2, C-3, C-4, C-5, C-6, C-8, C-10, C-16, C-29, C-58. IX: C-003/I-X-01 → C-3, C-10, C-29; SUMMARY/S-E0089 → C-1, C-2, C-4, C-5, C-6; C-003/E-C7-021 → C-2, C-3; C-001/X-001 → C-0, C-1, C-2, C-3, C-4, C-8, C-10, C-16, C-58; C-003/I-C7-01 → C-3

- Superseded: (C-003/SUP-C-C7-01) NEWER-PLAN PRECEDENCE: C-7 description is target state relative to older C plan(s) C-3, C-4 for this conflict. Losing description retained at C-003/C-C7-01. Conflict: C-7 says C-3 supplies `get_file_changes`, `accept_change`, `decline_change`, and `retry_audit`, but the complete C-3 source defines none of them and asserts its file scope is disjoint from C-4 (C-7 L78-L82/L127; C-3 L1-L8/L68-L141).
- (C-003/E-C7-001) Add `'explore'` to `WindowType`, `VALID_WINDOW_TYPES`, and `SIDEBAR_WINDOWS`; register `ExploreWindow.svelte`; bind `Ctrl+8` via `sidebar-8`.
- (C-003/E-C7-002) `ExploreWindow` renders a complete Q&A UI (input, submit, scrollable responses), logs `stub.logStub('explore.query',{query})`, and appends a placeholder response; only live backend search remains intentionally stubbed and marked by `UnwiredNote`.
- (C-003/E-C7-004) Export `closedTabs`, `showRecoveryModal`, and `openWindowRecovery()`; the function sets the modal flag.
- (C-003/E-C7-005) `RecoveryModal.svelte` lists real closures most-recent first with type, title, relative time, and `Restore`; restore calls `workspace.openTab('primary',entry.type)`; mount beside `CommandPalette` in `AppShell`.
- (C-003/E-C7-006) Keep shortcut id/combo/description for `recover-closed` but change `Ctrl+Shift+H` from command palette to `workspace.openWindowRecovery()`.
- [security] (C-003/E-C7-007) `Ctrl+Shift+A` and `Ctrl+Shift+R` call `reviewStore.acceptAllHunks()`/`rejectAllHunks()` only when the primary active tab is `review`; no global accept/reject.
- (C-003/E-C7-008) Create a minimal Svelte 5 `review.svelte.ts` store exposing accept-all and reject-all methods for `ReviewWindow` subscription.
- (C-003/E-C7-010) Replace static `FILES` with a derived `reviewStore.files`, remove `UnwiredNote`, invoke `get_file_changes({session_id})` on mount, and invoke `accept_change`, `decline_change`, or `retry_audit` from hunk/file actions.
- [failure] (C-003/E-C7-011) An empty `get_file_changes` result renders `<Empty>` rather than fixture data; `EXECUTING` with no changes yet renders a loading spinner.
- [config] (C-003/E-C7-012) At `max-width:999px`, hide `nav-list`, show/collapse to a hamburger icon strip, and manage hamburger state with `Sidebar.svelte` `ResizeObserver`.
- (C-003/E-C7-013) At `max-width:799px`, `Workspace.svelte` uses `ResizeObserver` to set `secondaryCollapsed=true`, giving the primary panel full width.
- (C-003/E-C7-015) Snapshot (b) opens and renders `ExploreWindow` with `Ctrl+8`.
- (C-003/E-C7-016) Snapshot (c) renders `ReviewWindow` with hunk controls visible.
- (C-003/E-C7-017) Snapshot (d) closes a window and opens `RecoveryModal` with `Ctrl+Shift+H`.
- (C-003/E-C7-018) Snapshot (e) verifies sidebar collapse at an 800px viewport; baselines use `page.screenshot` and later regression uses `toHaveScreenshot()`.
- [deferred] (C-003/E-C7-020) Add `Ctrl+Shift+W` only if a `shell`/`tier_0` window type exists; otherwise add only the exact deferral comment and no shortcut code.
- (C-003/E-C7-024) Restore reopens a window through `openTab`; accept/reject/retry invoke live handlers; refresh reflects applied state.
- (C-003/E-C7-026) Adds `review.svelte.ts`, `ExploreWindow.svelte`, `RecoveryModal.svelte`, `visual.spec.ts`, and `workflow/reasoning/C-7-decisions.md`; modifies the named API/store/window/shell/sidebar/workspace/CSS files.
- (C-003/C-C7-01) C-7 says C-3 supplies `get_file_changes`, `accept_change`, `decline_change`, and `retry_audit`, but the complete C-3 source defines none of them and asserts its file scope is disjoint from C-4 (C-7 L78-L82/L127; C-3 L1-L8/L68-L141).
- (C-003/A-C7-01) Restore records a title but calls `openTab` with only the type; removal of a restored entry, duplicate handling, recovery-stack persistence, and modal-close behavior are not defined (L50-L60).
- (C-003/A-C7-03) At exactly 800px, the `max-width:799px` single-panel rule does not apply even though the visual test is described as sidebar collapse at 800px; that test exercises the 999px rule only (L86-L100).
- (C-003/DELTA+-C7-1) The summary omits that Explore deliberately remains backend-stubbed, the exact stack schema/cap/FIFO rule, most-recent ordering, empty/loading states, all four live invokes, and conditional `Ctrl+Shift+W` behavior.
- (C-003/DELTA+-C7-3) Exact breakpoint inclusivity is `<=999px` and `<=799px`, not merely conceptual “below 1000/800”; the snapshot at 800px tests hamburger collapse, not the single-panel threshold.
- [lifecycle] (C-003/E-C7-003) `closedTabs` is a maximum-20 stack of `{type: WindowType,title:String,closedAt:number}`; `closeTab()` records `Date.now()` before removal; overflow uses FIFO eviction.
- (C-003/E-C7-009) Every `kind==='hunk'` separator gets `Accept`, `Decline`, and `Retry`; accept/decline toggle `hunkState[hunkContent]`, and retry exposes an inline note using `toggleRetryNote`.
- (C-003/E-C7-014) Playwright snapshot (a) loads the dashboard and renders `ProjectSelector`.
- [config] (C-003/E-C7-019) Add Playwright only after checking whether it is already a dev dependency.
- (C-003/E-C7-021) Review consumes C-2 `FileWrite`/`DispatchEditHunks` rows and four Tauri commands attributed by this plan to C-3.
- (C-003/E-C7-022) Cloud task `modify hello.rs to add a goodbye function` must show live hunks, action controls, applied state on the next query/poll, Explore, recovery, responsive collapse, and written baselines.
- (C-003/E-C7-023) Recovery must use real `closedTabs`; Review must not retain static `FILES`; Explore may stub only backend search; review-wide shortcuts must remain review-scoped.
- [gate] (C-003/E-C7-025) Five snapshots, `svelte-check`, and `cargo check --workspace` must pass.
- (C-003/DELTA!-C7-2) The summary presents live review wiring without flagging that its alleged C-3 commands do not exist in the C-3 source.
- (C-003/A-C7-02) `reviewStore` subscription/event semantics, polling cadence after the on-mount fetch, hunk identity collisions from `hunkContent`, and retry-note payload shape are not specified (L66-L84, L145-L151).
- (C-003/I-C7-01) `Date.now()` makes closure timestamps runtime-variable; no deterministic clock or persistence contract is supplied (L50-L54).
- (SUMMARY/S-E0091) Creates ExploreWindow.svelte. [L94]
- (SUMMARY/S-E0092) Assigns Ctrl+8 to open the Explore window. [L94]
- (SUMMARY/S-E0094) Creates RecoveryModal.svelte. [L95]
- (SUMMARY/S-E0095) Fixes Ctrl+Shift+H so it opens recovery instead of remaining a no-op stub. [L95]
- (SUMMARY/S-E0096) Adds Ctrl+Shift+A for review accept-all. [L96]
- (SUMMARY/S-E0097) Adds Ctrl+Shift+R for review reject-all. [L96]
- (SUMMARY/S-E0098) Adds per-hunk Accept, Decline, and Retry controls to ReviewWindow. [L97]
- (SUMMARY/S-E0099) Removes the UnwiredNote banner after wiring ReviewWindow to live get_file_changes data. [L97]
- (SUMMARY/S-E0100) Adds a responsive hamburger layout below 1000px. [L98]
- (SUMMARY/S-E0101) Adds a single-panel layout below 800px. [L98]
- (SUMMARY/S-E0103) Cloud smoke modifies hello.rs to add goodbye; acceptance requires visible hunk controls and working Ctrl+Shift+A accept-all. [L100]
- (SUMMARY/S-E0089) Closes B-11, B-12, B-13, B-14, B-26, dashboard-design, and phase-4K; depends on C-6 and C-2; is parallel-safe with C-5 and C-1 through C-4. [L90-L93]
- (SUMMARY/S-E0090) Adds the explore member to WindowType. [L94]
- (SUMMARY/S-E0093) Implements a closedTabs stack. [L95]
- (SUMMARY/S-E0102) Adds five Playwright snapshots in visual.spec.ts. [L99]

### C-8: Activate remaining knowledge/orchestration producers (ambiguity, Cosmos, ONNX, search mode)

**Status**: planned
**Anchors**: ledgers C-004, SUMMARY | sample anchors: C-8 L33-L43; C-8 L112-L116, L330-L332, L402; C-8 L118-L138, L334-L340; C-8 L121-L130; C-8 L181-L189; C-8 L185-L188, L418-L420 | 84 deep atoms, 14 summary atoms | IDs: C-004/F-C8-004, C-004/F-C8-010, C-004/F-C8-011, C-004/F-C8-012, C-004/F-C8-022, …
**Composes with**: C-0, C-1, C-2, C-3, C-4, C-5, C-7, C-10, C-15, C-16, C-17, C-21, C-25, C-26, C-38, C-46, C-47, C-52, C-54, C-58. IX: C-005/X-E02 → C-2, C-5, C-16, C-17; C-004/F-C8-005 → C-1, C-2, C-3, C-4; C-004/X-001 → C-4, C-17; C-004/X-003 → C-4, C-15, C-17, C-21; SUMMARY/S-E0104 → C-1, C-2, C-3, C-4; C-001/X-001 → C-0, C-1, C-2, C-3, C-4, C-7, C-10, C-16, C-58

- Superseded: (C-004/F-C8-004) The preserved objective replaces live-data mocks/hardcoded producer stubs across ambiguity scoring, Cosmos scheduling/events/stage integration, artifact indexing/RPC, reranking notification, A-RAG semantic scoring, pipeline latency, degradation/search-mode state, and dashboard unwired flags. [C-8 L33-L43]
- Superseded: (C-004/F-C8-017) `TOOL_INDEX_ARTIFACTS` replaces the prior `Err("full indexing deferred to Phase C")` stub, reusing the pre-existing `ArtifactsIndexed` action/payload from B-34 rather than adding a duplicate schema variant. [C-8 L155-L160]
- Superseded: (C-004/D-C8-007) The ONNX path has an explicit warned norm-proxy fallback, so “replaces” is not an unconditional removal. [Summary L196; C-8 L282-L283, L424-L425, L487-L490]
- Superseded: (SUMMARY/S-E0108) Replaces the production TOOL_INDEX_ARTIFACTS error stub with a real implementation in production_agent_rpc.rs. [L109]
- Superseded: (SUMMARY/S-E0109) Replaces the CrossEncoderReranker NoopSsbNotifier. [L110]
- Superseded: (SUMMARY/S-E0111) Replaces AragOrchestrator semantic-search’s embedding-norm proxy with real ONNX dot-product scoring. [L111]
- (C-004/F-C8-004) The preserved objective replaces live-data mocks/hardcoded producer stubs across ambiguity scoring, Cosmos scheduling/events/stage integration, artifact indexing/RPC, reranking notification, A-RAG semantic scoring, pipeline latency, degradation/search-mode state, and dashboard unwired flags. [C-8 L33-L43]
- (C-004/F-C8-010) `get_ambiguity_state` queries the newest `ambiguity_scored` row, returns the real score structure, and omits the former `"mock"` field. [C-8 L112-L116, L330-L332, L402]
- (C-004/F-C8-011) The Cosmos hourly scheduler is hosted in `cli/src/live_cli.rs` within `build_local_knowledge_stack`, not in the per-task `task_runner.rs`; this gives it orchestrator-process lifetime. [C-8 L118-L138, L334-L340]
- (C-004/F-C8-012) Scheduler bootstrap occurs only when an `Arc<Ssb>` is present and the manifest-present branch succeeds; CLI cold-start paths without an SSB writer remain unaffected. [C-8 L121-L130]
- (C-004/F-C8-022) Search-mode state crosses the CLI/dashboard process boundary through a new `SearchModeChanged` SSB row and `SearchModeChangedPayload`, replacing the infeasible in-memory `AppState Arc<RwLock<SearchMode>>` design. [C-8 L181-L189]
- (C-004/F-C8-023) The CLI emits search mode once at startup and on every `DegradationCascade` transition. [C-8 L185-L188, L418-L420]
- (C-004/F-C8-024) Dashboard `get_search_mode` reads `SsbViewer::latest_action_of_type("search_mode_changed")`; if no row exists, it gracefully returns lowercase `"hybrid"`, not a hardcoded uppercase `"HYBRID"` production answer. [C-8 L187-L190, L359-L363, L421-L422, L527-L529]
- (C-004/F-C8-025) A new dashboard-side `set_search_mode` Tauri command emits a `SearchModeChanged` row, allowing reviewers to pin the diagnostic mode. [C-8 L190-L191, L527-L530]
- (C-004/F-C8-037) The proposed `unwired.ambiguityState` flag is a phantom: it never existed. `GatesPanel.svelte` and `OrchestrationDrawer.svelte` invoke `get_ambiguity_state` directly, so there is nothing to flip. [C-8 L244-L257, L433-L436]
- (C-004/F-C8-038) `trigger_cosmos_refresh` remains an uncalled per-task helper in `task_runner.rs`; follow-up must either wire it from a task-entry hook or document its removal. [C-8 L536-L542, L561-L565]
- (C-004/F-C8-039) A frontend per-mode `SearchModeBadge` picker that calls `set_search_mode` remains pending, although its backend command is complete. [C-8 L566-L567]
- (C-004/F-C8-045) Twelve deterministic checks cover source symbols/absence patterns, scheduler host, artifact hook, stub removal, notifier construction, function-scoped search-mode hardcoding, all latency sites, Cosmos stage, dashboard flag, workspace compilation, and four contract-test invocations. [C-8 L323-L396]
- (C-004/F-C8-049) The Cosmos contract tests the helper rather than the live scheduler host; host construction remains a separate CLI integration witness. [C-8 L472-L475]
- (C-004/F-C8-054) `reachable_from` still glosses `get_search_mode` as an `AppState.search_mode live read`, while the rebaseline explicitly replaced that design with an SSB viewer read; the latter is the authoritative shipped design. [C-8 L181-L197 versus L301-L306]
- (C-004/D-C8-009) `latest_ambiguity_scored` is the shipped method name, and the alleged `unwired.ambiguityState` flag is a resolved phantom. [Summary L190; C-8 L100-L116, L244-L257]
- (C-004/F-C8-001) C-8 is explicitly **CLOSED** at the 2026-05-23 rebaseline; all 12 producer-wiring tasks shipped in session `s-32952`, and session `s-25120` performed the completion-drive recheck. [C-8 L1-L10]
- (C-004/F-C8-002) The recheck changed two prior `PARTIAL` findings to `STALE-SHIPPED-ON-V2`, two prior `MISSING` findings to `PHANTOM-CLAIM-RESOLVED`, applied six `SPEC_DRIFT` fixes, and refined four deterministic checks. [C-8 L5-L27]
- (C-004/F-C8-003) Four contract-test files—three added and one extended—pin the wiring witnesses; the reported audit result is 14/14 passing. [C-8 L9-L10, L389-L395, L453-L483]
- (C-004/F-C8-005) C-8 requires C-1 for `QueryTrace`/`CosmosRefreshStarted` SSB variants and C-2 for confirmed `SsbNotifier` injection paths; it is declared parallel-safe with C-3 and C-4. [C-8 L46-L50]
- (C-004/F-C8-006) The decision record is `workflow/reasoning/C-8-decisions.md`, containing DRIFT-A..M, R1–R4, T1–T4, and A1–A4 rationale/amendments. [C-8 L29-L31, L554-L557]
- (C-004/F-C8-007) `AmbiguityScorer::score(&plan_text).await` is called from `task_runner.rs` only when constructing `TransitionContext` for the `PLANNING -> TESTING_FIRST` transition, before `GroundingSweepGate`; it is not called on every step. [C-8 L94-L98, L265-L267, L307-L309]
- [failure] (C-004/F-C8-008) Ambiguity scoring failure logs `tracing::warn!`, stores `ambiguity_score: None`, and intentionally preserves the B-17 fallback behavior; success stores `Some(score)`. [C-8 L94-L99, L400-L401]
- (C-004/F-C8-009) The live viewer method is `SsbViewer::latest_ambiguity_scored()`, not the originally proposed `last_ambiguity_score()`; the shipped name follows sibling `recent_*`/`latest_*` conventions. [C-8 L100-L113]
- (C-004/F-C8-013) The interval is `DEFAULT_TICK_INTERVAL_SECS = 3600`; each cycle calls `cosmos_tick_emit_loop(&ssb_for_tick, &worker_for_tick).await`. [C-8 L130-L134, L403-L406]
- (C-004/F-C8-014) The wrapper emits one `CosmosRefreshStarted` SSB row and one `CosmosArtifactUpdated` row for each refreshed artifact name; `CosmosWorker::tick()` stays synchronous and storage-only, with no `SsbNotifier` field added. [C-8 L131-L136, L268-L271]
- (C-004/F-C8-015) `IndexingService::build_index_resumable` invokes the artifact hook after code-phase completion through `maybe_index_artifacts`; artifact failure is non-fatal, warns, and allows core indexing to continue. [C-8 L140-L153, L407-L410]
- (C-004/F-C8-016) Successful artifact indexing emits `ArtifactsIndexed`; the originally proposed `ArtifactStale` rows are explicitly dropped because staleness is private to `index_artifact` and is represented in returned `ArtifactIndexState`, partitioned into `indexed[]` and `up_to_date[]`. [C-8 L140-L153]
- (C-004/F-C8-017) `TOOL_INDEX_ARTIFACTS` replaces the prior `Err("full indexing deferred to Phase C")` stub, reusing the pre-existing `ArtifactsIndexed` action/payload from B-34 rather than adding a duplicate schema variant. [C-8 L155-L160]
- (C-004/F-C8-018) The public `IndexArtifactsRequest` remains singular for IPC wire compatibility; the handler internally wraps it as a one-element `Vec<IndexArtifactsRequest>` to ease a future bulk-frame decode upgrade without current public API churn. [C-8 L161-L166]
- (C-004/F-C8-019) The RPC opens the manifest and store, calls `index_artifact` per specification, emits exactly one `ArtifactsIndexed` row, and returns JSON with `project_id`, `artifact_count`, `chunk_count`, `elapsed_ms`, `indexed[]`, and `up_to_date[]`. [C-8 L166-L174]
- [gate] (C-004/F-C8-020) Artifact RPC errors propagate as `AgentRpcError::ToolError`; panic is forbidden. [C-8 L168-L170]
- (C-004/F-C8-021) Production `CrossEncoderReranker` construction receives the real `Arc<Ssb>` coerced to `Arc<dyn SsbNotifier>`; production construction of `NoopSsbNotifier::{new,default}` is banned. [C-8 L176-L179, L352-L357, L415-L417]
- (C-004/F-C8-026) Adding the cross-process search-mode event atomically raises `SCHEMA_VERSION` 49→50 and the action/payload count assertion 101→102, including enum, payload, and `from_str` arms. [C-8 L192-L196, L278-L281, L506-L509]
- (C-004/F-C8-027) Dashboard `unwired.searchMode` is switched to `false`, and TypeScript `SearchMode` accepts lowercase wire tags in addition to the legacy uppercase enum. [C-8 L196-L197, L423, L531-L534]
- (C-004/F-C8-028) `AragOrchestrator::semantic_search` receives an optional injected ONNX `OrtSession` via `with_embedder()` and uses query/document dot product when available. [C-8 L199-L202, L424-L425, L487-L490]
- (C-004/F-C8-029) If the ONNX embedder fails, semantic search logs a warning and retains the old norm-proxy behavior as a fallback. [C-8 L282-L283, L424-L425, L487-L490]
- (C-004/F-C8-030) `QueryPipeline` gains `clock: Arc<dyn ClockProvider>` and a `with_clock()` builder; production installs the process `OsClock`, while deterministic tests can supply `SeededClock`. [C-8 L204-L213, L426-L429]
- (C-004/F-C8-031) All 17 ordinary `PipelineStage` latency literals plus the two synthetic `deep_cap`/`rrf_gate` emit sites compute milliseconds using `clock.elapsed_since(stage_start).as_millis() as u64`; literal `latency_ms: 0` count must be zero. [C-8 L212-L221, L365-L371, L426-L429, L498-L503]
- (C-004/F-C8-032) Raw `std::time::Instant` is prohibited for pipeline instrumentation by entropy quarantine; the sole scoped exemption is per-RPC `TOOL_INDEX_ARTIFACTS.elapsed_ms`, avoiding a public constructor expansion. [C-8 L204-L227, L283-L285]
- (C-004/F-C8-033) C-8 creates a new `CosmosStage { worker: Arc<CosmosWorker> }` with stable `const fn name() -> "cosmos"` and `run(&self, query) -> Vec<RetrievalResult>`. [C-8 L229-L234]
- (C-004/F-C8-034) `CosmosStage` is inserted between `ArtifactStage` and `ARagStage` for `STANDARD` and `DEEP` retrieval tiers only; `FAST` omits it. [C-8 L234-L236, L373-L377, L430-L432]
- (C-004/F-C8-035) Cosmos routing lowercases and alphanumeric-splits the query and artifact `name`/`research_query`, drops tokens shorter than three characters, and matches by token overlap. [C-8 L236-L239]
- (C-004/F-C8-036) Each matching Cosmos artifact becomes one `RetrievalResult` with curated score `0.75` and a 1,024-character snippet cap; downstream RRF supplies relative ranking. [C-8 L239-L242]
- (C-004/F-C8-040) Several stale exact-schema-version contract tests still need migration to the C-7 `>=` floor pattern, and a platform-aware `std::time` guard is still needed for a pre-existing artifacts clock-overflow failure. [C-8 L568-L572]
- (C-004/F-C8-041) Ambiguity, Cosmos retrieval, and reranking are reachable from `elai /research --mode cloud`; artifact indexing is reachable through the agent in `elai /task --mode cloud`; the indexing hook is reachable through `reindex_now`; search mode is reachable through Tauri commands. [C-8 L301-L306]
- (C-004/F-C8-042) Stub bans require: ambiguity scoring only at its transition gate, real Cosmos SSB writes rather than tracing alone, an `Ok` artifact RPC, no production no-op reranker notifier, no literal-zero pipeline latency, and no hardcoded search-mode result. [C-8 L307-L313]
- (C-004/F-C8-043) The primary observable research trigger expects `ambiguity_scored` with non-null score and no `mock` key, positive `query_trace.total_latency_ms`, and `cosmos_refresh_started` when configured/ticked. [C-8 L314-L321]
- (C-004/F-C8-044) The primary artifact observable requires `TOOL_INDEX_ARTIFACTS` to return `Ok` with a present `artifact_count` (which may be zero). [C-8 L320-L321]
- (C-004/F-C8-046) `production_agent_rpc_contract` test F asserts a result with parseable `artifact_count: u32` plus `project_id`, `chunk_count`, `elapsed_ms`, `indexed[]`, and `up_to_date[]`; the suite reports 6/6. [C-8 L170-L174, L455-L459]
- (C-004/F-C8-047) `ambiguity_task_runner_contract` has three cases: qualifying score+streak allows through both gate APIs, a high `overall_score` blocks with an ambiguity-tagged reason, and `None` preserves streak-only allow; it reports 3/3. [C-8 L460-L465]
- (C-004/F-C8-048) `cosmos_tick_wiring_contract` has three cases: one-artifact emits queued/total 1, empty manifest emits one unconditional started row with zeros, and empty manifest yields zero updates/refreshed count; it reports 3/3. [C-8 L466-L475]
- (C-004/F-C8-050) `indexing_service_artifact_hook_contract` proves absent manifest is a documented no-op and malformed TOML is caught/warned without failing core indexing; it reports 2/2. [C-8 L476-L481]
- (C-004/F-C8-051) A prior six-assertion `c8_stub_eradication_contract` pins schema floor, SQL-string mapping, `from_str` round-trip, payload round-trip, `ArtifactsIndexed` round-trip, and history ordering. [C-8 L544-L547]
- (C-004/F-C8-052) Workspace compilation and all 12 deterministic checks are marked passing; the rebaseline acceptance checklist is otherwise fully checked or explicitly resolved. [C-8 L398-L440]
- (C-004/F-C8-053) The live Cloud-Sonnet smoke is explicitly deferred to C-9+ because it needs Sonnet plus configured `[[cosmos.artifacts]]`; the deterministic T1–T4 contracts are the present closure evidence. [C-8 L441-L451]
- (C-004/F-C8-055) Task 4 retains the phrase “wire `index_all_artifacts`,” but the rebaseline says the implementation calls per-artifact `index_artifact` through `maybe_index_artifacts`; exact implementation naming must follow the rebaseline. [C-8 L140-L153, L342-L346]
- [deferred] (C-004/F-C8-056) Because the cloud smoke is deferred while all deterministic contracts pass, `CLOSED` here means source-level/contract closure, not proof of the named live cloud scenario. [C-8 L3-L10, L437-L451]
- (C-004/D-C8-001) C-8 is now explicitly `CLOSED`, with 12 tasks shipped and 14/14 contract tests passing; the summary presents it as undifferentiated planned production activation. [Summary L182-L199; C-8 L3-L10, L389-L440]
- (C-004/D-C8-002) The hourly Cosmos scheduler is in long-lived `live_cli`, not `task_runner`; it is gated by SSB/manifest availability and emits events through an external wrapper. [Summary L191; C-8 L118-L138]
- (C-004/D-C8-003) Artifact integration is `maybe_index_artifacts`/per-artifact `index_artifact`, nonfatal on error; `ArtifactStale` rows were dropped and `ArtifactsIndexed` was reused. [Summary L192-L193; C-8 L140-L174]
- (C-004/D-C8-004) The artifact RPC keeps a singular wire request, returns the six-field JSON shape, emits one event, and maps errors to `AgentRpcError::ToolError`. [Summary L193; C-8 L155-L174]
- (C-004/D-C8-005) Search mode is not stored in `AppState.search_mode`; it crosses processes through `SearchModeChanged`, with startup/transition emits, viewer lookup, lowercase fallback, diagnostic setter, and atomic schema 49→50. [Summary L195; C-8 L181-L197]
- (C-004/D-C8-006) Pipeline latency uses injected `ClockProvider`, not raw `Instant::elapsed`; all 17 ordinary plus two synthetic sites are covered, with one explicit RPC-only `Instant` exemption. [Summary L197; C-8 L204-L227]
- (C-004/D-C8-007) The ONNX path has an explicit warned norm-proxy fallback, so “replaces” is not an unconditional removal. [Summary L196; C-8 L282-L283, L424-L425, L487-L490]
- (C-004/D-C8-008) `CosmosStage` had to be created; its token-overlap router, score 0.75, snippet cap 1,024, and exact stage ordering are omitted from the summary. [Summary L198; C-8 L229-L242]
- [deferred] (C-004/D-C8-010) The named cloud smoke is deferred to C-9+, while four deterministic contract files are current closure evidence; an orphan Cosmos helper and frontend mode picker remain follow-ups. [Summary L199; C-8 L441-L483, L559-L570]
- (SUMMARY/S-E0104) Closes B-3, B-4, B-5, B-7, B-38, B-42, and phase-4K; depends on C-1 and C-2; is parallel-safe with C-3 and C-4. [L102-L105]
- (SUMMARY/S-E0105) Wires AmbiguityScorer into task_runner only at the PLANNING to TESTING_FIRST gate. [L106]
- (SUMMARY/S-E0106) Drives CosmosWorker::tick hourly through a Tokio interval in task_runner. [L107]
- (SUMMARY/S-E0107) Calls index_all_artifacts from IndexingService::build_index_resumable. [L108]
- (SUMMARY/S-E0108) Replaces the production TOOL_INDEX_ARTIFACTS error stub with a real implementation in production_agent_rpc.rs. [L109]
- (SUMMARY/S-E0109) Replaces the CrossEncoderReranker NoopSsbNotifier. [L110]
- (SUMMARY/S-E0110) Stores DegradationCascade in AppState.search_mode. [L110]
- (SUMMARY/S-E0111) Replaces AragOrchestrator semantic-search’s embedding-norm proxy with real ONNX dot-product scoring. [L111]
- (SUMMARY/S-E0112) Preserves real latency by replacing at least 17 latency_ms=0 sites in pipeline.rs with Instant::elapsed measurements. [L112]
- (SUMMARY/S-E0113) Inserts CosmosStage into QueryPipeline only for STANDARD and DEEP tiers; FAST omits it. [L113]
- (SUMMARY/S-E0114) Research smoke compares two codebase concepts and expects positive total query latency, non-mock ambiguity scoring, and cosmos_refresh_started. [L114]

### C-9: Autonomous auto-advance, escalation policy, and per-role backends

**Status**: planned
**Anchors**: ledgers C-005, SUMMARY | sample anchors: L131-L135; L134-L136; L189-L217; L230-L232; L276-L304; L306-L328 | 60 deep atoms, 14 summary atoms | IDs: C-005/C9-E16, C-005/C9-E17, C-005/C9-E26, C-005/C9-E31, C-005/C9-E33, …
**Composes with**: C-1, C-4, C-5, C-10, C-16, C-24, C-25, C-26, C-28, C-32, C-34, C-35, C-36, C-44, C-49. IX: C-004/X-006 → C-4, C-25, C-26; SUMMARY/S-E0115 → C-1, C-5, C-10; C-005/C9-I02 → C-5; C-005/C9-E01 → C-1, C-5, C-10; C-005/X-A05 → C-5, C-16, C-24, C-28, C-32, C-34, C-35, C-36, C-44, C-49; C-005/X-I04 → C-5, C-44

- Superseded: (C-005/C9-E13) `AutonomyDecision::SplitTask` replaces bare count/depth fields with `split_plan: SplitTaskPlan`, and every destructuring match in `task_runner.rs` must change with it. [L117-L119]
- Superseded: (C-005/C9-E16) CLI startup replaces `autonomous_gate: None` with a real `AutonomousGate` configured from the parsed mode and otherwise-default config. [L131-L135]
- [gate] (C-005/C9-E16) CLI startup replaces `autonomous_gate: None` with a real `AutonomousGate` configured from the parsed mode and otherwise-default config. [L131-L135]
- [gate] (C-005/C9-E17) `GateMode::from_cli_args()` consumes C-5’s `--mode` value and defaults to `Interactive` when the flag is absent. [L134-L136]
- (C-005/C9-E26) Produced/reachable surfaces are `AutoAdvanceEvaluator`, `EvidenceCheck`, `EvidenceVerdict`, `EscalationPolicy`, split types, two triggers, the fit method, CLI mode parser, `ResearchTree::split_step`, backend TOML/runtime types, and Phase-4G measurements, reachable from CLI config, gate decision, split dispatch, and the existing MonitorWindow statistics panel. [L189-L217]
- (C-005/C9-E31) Artifact/UI evidence requires populated measurements with completion true and a non-zero MonitorWindow `AutoAdvanceStats.last_5min_count`. [L230-L232]
- (C-005/C9-E33) Acceptance additionally requires six escalation variants, the fit check on the actual substitution path, real gate construction, parsed backend stanzas, live split dispatch, the autonomous run to complete without prompting, the sidebar to display `AUTONOMOUS`, clean scoped builds/tests, and the three-decision document. [L276-L304]
- [gate] (C-005/C9-E34) Implementation touches autonomous gate, model pool, research tree, task runner, CLI startup/runtime config, example TOML, Phase-4G metrics, and one new decision document. [L306-L328]
- (C-005/C9-D01) Add exact verdict/check/evaluator schemas, missing-transition behavior, aggregation uncertainty, policy defaults, trigger labels/firing conditions, split schemas/semantics, fit formula, CLI default mode, exact backend defaults, tree-unavailable escalation, decision rationale, and the three stub bans. [L68-L162, L183-L187, L218-L223]
- (C-005/C9-E01) The plan is a 2–3-session implementation of seven named Phase-4G gaps plus per-role backend config, runtime measurements, and live `SplitTask` dispatch; it depends on C-1 and C-5 and is parallel-safe with C-10. [L3-L16]
- (C-005/C9-E02) `EvidenceVerdict` has `Pass`, `Fail { reason: String }`, and `Missing`, derives debug/clone/equality/serde traits, and exposes `as_str()` for SSB labels. [L68-L70]
- (C-005/C9-E03) `EvidenceCheck` holds `artifact_path: Option<PathBuf>`, `description: &'static str`, and `check_fn: fn(&Evidence) -> EvidenceVerdict`; it is synchronous and only inspects the already-derived `Evidence` snapshot. [L72-L76]
- [gate] (C-005/C9-E04) `AutoAdvanceEvaluator` owns `gate_checks: HashMap<FsmTransition, EvidenceCheck>`; `evaluate(transition,evidence)` runs the matching check or returns `Missing` when no transition entry exists. [L78-L82]
- (C-005/C9-E05) Its default catalog is to be pre-populated with checks associated with `AntiSpiral`, `HighRisk`, `ReplanMajor`, and `CircuitBreaker`; `evaluate_all()` iterates all checks and becomes the path used by `AutonomousGate::decide()` around `Evidence::all_passed()`. [L83-L87]
- (C-005/C9-E06) `EscalationPolicy` declares `anti_spiral_threshold`, `high_risk_pauses`, and `max_auto_advances_per_phase`; defaults are `3`, `true`, and `10`. [L89-L96]
- [gate/config] (C-005/C9-E07) `AutonomousGateConfig` gains `escalation_policy` while retaining `circuit_breaker_limit` for backward compatibility; the default phase maximum mirrors the old limit. [L91-L96]
- (C-005/C9-E08) `EscalationTrigger` gains `SubstituteModelFailed` and `SplitTaskFailed`, serialized/labeled as `substitute_model_failed` and `split_task_failed`. [L98-L101]
- (C-005/C9-E09) Those triggers fire when the model fallback chain is empty or a split collapses to `sub_task_count=0`, respectively. [L102-L103]
- (C-005/C9-E10) `MergeStrategy` initially has only `Sequential`; a `Parallel` variant is explicitly deferred until C-10’s anti-spiral fan-out is stable. [L105-L108]
- (C-005/C9-E11) `SubTask` contains `task_id`, `scope`, `context_budget`, and `depends_on`, with debug/clone/equality/serde derives. [L110-L112]
- (C-005/C9-E12) `SplitTaskPlan` contains `parent_phase`, `sub_tasks`, `merge_strategy`, `max_depth`, and `current_depth`. [L114-L116]
- (C-005/C9-E13) `AutonomyDecision::SplitTask` replaces bare count/depth fields with `split_plan: SplitTaskPlan`, and every destructuring match in `task_runner.rs` must change with it. [L117-L119]
- (C-005/C9-E14) `ModelPool::can_fit_llm(model_size_bytes)` is a default trait method implementing `available_vram() >= model_size_bytes`, preserving existing mock implementations. [L121-L126]
- (C-005/C9-E15) Before returning `SubstituteModel`, `AutonomousGate::decide()` applies the fit check and escalates with `SubstituteModelFailed` if estimated model size exceeds VRAM. [L127-L129]
- [config] (C-005/C9-E18) Example runtime config adds `[agents.executor.backend] backend="auto"`, `[agents.auditor.backend] backend="anthropic_api"`, and `[agents.designer.backend] backend="anthropic_api"`. [L138-L149]
- (C-005/C9-E19) `BackendStanza { backend: String }` is added to runtime config; each role receives an optional stanza whose absent value defaults to `auto`. [L150-L152]
- (C-005/C9-E20) The live split arm first retains its optional splitter and otherwise calls `ResearchTree::split_step(&SplitTaskPlan)`. [L154-L157]
- (C-005/C9-E21) `split_step` locates the active node through `parent_phase`, clones planned subtasks, and corrects `task_id` sequencing. [L157-L159]
- (C-005/C9-E22) If no research-tree handle is reachable at that dispatch site, the path emits `SplitTaskFailed` and escalates rather than manufacturing a split. [L160-L162]
- (C-005/C9-E23) `[runtime_measurements]` records auto-advance accuracy, false-escalation rate, mode-switch latency, split overhead, substitute-model swap latency, autonomous/interactive end-to-end ratio, gate-evaluation overhead, escalation count per session, and `completion=true` after the smoke run. [L164-L175]
- (C-005/C9-E24) The cloud Sonnet 4.6 smoke invokes autonomous mode to build a tested four-method Rust calculator, captures SSB rows, and supplies the runtime measurements. [L177-L181]
- (C-005/C9-E25) The decision artifact records why the fit method is a trait default, why only sequential merge exists, and why headless cloud wiring has no `interactive_gate`/`InterventionCard` route. [L183-L187]
- (C-005/C9-E27) The evaluator may not return `Pass` when `Evidence::all_passed()` is false. [L218-L220]
- (C-005/C9-E28) The default fit implementation may not always return true. [L221]
- (C-005/C9-E29) A non-empty split plan may not become an empty vector; that condition escalates. [L222-L223]
- [gate] (C-005/C9-E30) Observable evidence requires a `gate_decision` row with `decision=AutoAdvance` and `source=AutoAdvanceEvaluator`; `escalation_fired` is conditional on stagnation. [L224-L229]
- (C-005/C9-E32) Deterministic inspection checks every new named type/variant/config path, proves the old `autonomous_gate: None` text is absent, verifies `split_step`, builds the workspace, runs scoped orchestrator/contracts tests, and confirms measurement keys. [L234-L274]
- (C-005/C9-E35) The next direct continuation is C-10, although C-11 is also permitted by prioritization; the stated non-overlap is that C-10 does not touch `autonomous_gate.rs` or `task_runner.rs`. [L330-L335]
- (C-005/C9-E36) Every acceptance box is unchecked; this original specifies planned work and does not claim the C-9 implementation is shipped. [L276-L304]
- (C-005/C9-D02) Add all nine measurement assignments and the conditional nature of `escalation_fired`; the summary only says eight keys. [L164-L181, L224-L232]
- (C-005/C9-D03) Replace the summary’s unqualified “EscalationPolicy with four fields” with the explicit three-field schema plus the unresolved acceptance conflict. [L89-L96, L283]
- (C-005/C9-D04) Qualify the summary’s `TreeAutoAdvancePolicy` wiring statement with the source’s concrete `AutonomousGate::new(...)` construction and unresolved identifier relationship. [L131-L136]
- (C-005/C9-D05) Treat “closes phase-4G and B-22” as orientation shorthand: the source says it implements named gaps from those reports, not that every report item is already closed. [L3-L10, L20-L50]
- [gate] (C-005/C9-A01) `gate_checks` is keyed by `FsmTransition`, but the default population is described as “one entry per `EscalationTrigger` variant”; the mapping/key construction is not specified. [L78-L87]
- (C-005/C9-A02) `evaluate_all()` is said both to iterate the gate map and to “wrap” `Evidence::all_passed()`; exact aggregation rules for `Pass`, `Fail`, and `Missing` are absent. [L83-L87, L192-L203]
- (C-005/C9-C01) The task defines exactly three `EscalationPolicy` fields, while acceptance says “4 fields”; the orientation summary repeats four but omits names. No fourth field is named. [L89-L96, L283]
- (C-005/C9-C02) The measurements block lists nine TOML assignments including `completion`, while acceptance and the orientation summary call them “all 8 keys.” [L164-L175, L299-L300]
- (C-005/C9-A03) `SubstituteModelFailed` is described for both an empty fallback chain and insufficient VRAM; precedence and whether the attempted target/fallback identity is retained are unspecified. [L102-L103, L127-L129]
- (C-005/C9-A04) `split_step` “corrects task_id sequencing,” but its sequencing algorithm, duplicate handling, dependency rewrite, depth enforcement, merge operation, and idempotency are not defined. [L154-L162]
- (C-005/C9-C03) The objective/context call the missing production object `TreeAutoAdvancePolicy`, but Task 10 constructs `AutonomousGate`; the orientation summary says a real `TreeAutoAdvancePolicy` is passed. The relationship between those identifiers is not resolved. [L7, L37-L38, L131-L136]
- (C-005/C9-A05) The new backend section syntax nests `.backend` and also contains a key named `backend`; exact serde nesting for every role is not shown. [L138-L152]
- (C-005/C9-I01) Backward compatibility is intended at config construction and mock-trait implementation boundaries, not merely at serialization, because both the old circuit-breaker field and a trait default are retained. [L91-L96, L121-L126]
- (C-005/C9-I02) Split recovery is fail-closed: inability to materialize a non-empty plan becomes an escalation rather than silent continuation. [L154-L162, L222-L223]
- (SUMMARY/S-E0123) Passes a real TreeAutoAdvancePolicy from cli/main.rs instead of None. [L121]
- (SUMMARY/S-E0126) Autonomous smoke builds a four-method Rust calculator with tests; acceptance requires at least one gate_decision whose source is AutoAdvanceEvaluator and a positive AutoAdvanceStats.last_5min_count in MonitorWindow. [L124]
- (SUMMARY/S-E0115) Closes phase-4G and B-22; depends on C-1 and C-5; is parallel-safe with C-10. [L116-L119]
- (SUMMARY/S-E0116) Adds AutoAdvanceEvaluator, EvidenceCheck, and EvidenceVerdict. [L120]
- (SUMMARY/S-E0117) Adds EscalationPolicy with four fields; field names are omitted from the summary. [L120]
- (SUMMARY/S-E0118) Adds SplitTaskPlan, SubTask, and MergeStrategy. [L120]
- (SUMMARY/S-E0119) Adds SubstituteModelFailed and SplitTaskFailed to EscalationTrigger. [L120]
- (SUMMARY/S-E0120) Adds a default ModelPool::can_fit_llm method. [L120]
- [gate] (SUMMARY/S-E0121) Adds GateMode::from_cli_args. [L120]
- (SUMMARY/S-E0122) Adds ResearchTree::split_step. [L120]
- [config] (SUMMARY/S-E0124) Adds per-agent backend configuration through TOML stanzas of the form agents.{role}.backend and the BackendStanza type. [L122]
- (SUMMARY/S-E0125) Populates phase_4G.toml runtime_measurements with eight keys. [L123]

### C-10: Anti-spiral stagnation, personas, goal-drift canary, and generation watchdog

**Status**: planned
**Anchors**: ledgers C-003, SUMMARY | sample anchors: L56-L57; L134-L143; L304-L319; L130; L133; L1-L7 | 58 deep atoms, 21 summary atoms | IDs: C-003/E-C10-010, C-003/E-C10-033, C-003/E-C10-034, C-003/E-C10-045, C-003/E-C10-001, …
**Composes with**: C-0, C-1, C-2, C-3, C-4, C-7, C-8, C-9, C-11, C-15, C-16, C-29, C-58. IX: C-003/I-X-02 → C-3, C-11; SUMMARY/S-E0127 → C-2, C-9, C-11, C-15; C-003/I-X-01 → C-3, C-7, C-29; C-001/X-001 → C-0, C-1, C-2, C-3, C-4, C-7, C-8, C-16, C-58

- (C-003/E-C10-010) `DiminishingReturns` is detected when error-count improvement is below 5% per step across a four-attempt window.
- (C-003/E-C10-033) Add `get_stagnation_events`, returning newest-first rows for `stagnation_spinning_detected`, `stagnation_oscillation_detected`, `lateral_thinking_activated`, `goal_drift_warning`, and `watchdog_timeout`.
- (C-003/E-C10-034) Collapsed `StagnationSection` shows timestamp, pattern/persona, detail, and existing-token badges: amber stagnation, blue lateral, orange drift, red watchdog.
- (C-003/E-C10-045) Add the anti-spiral module, two contract files, five agent files, and decision doc; modify gate, task runner, phase metrics, Tauri commands, and drawer.
- (C-003/E-C10-001) Add the Rust anti-spiral runtime, wire it into autonomous decisions/step boundaries, emit events, create five persona agents, and surface stagnation in the orchestration dashboard.
- (C-003/E-C10-002) `StagnationPattern = Spinning | Oscillation | NoDrift | DiminishingReturns`, deriving `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Serialize`, and `Deserialize`.
- (C-003/E-C10-003) `AttemptResult` fields are `attempt_index:u32`, `error_class:String`, `output_hash:u64`, `error_count:u32`, and `timestamp_ms:u64`.
- [lifecycle] (C-003/E-C10-004) `ExecutionHistory` is a stdlib `VecDeque<AttemptResult>` ring of capacity 32 with `push`, `is_full`, `last_n`, and `clear`; pushing past capacity evicts.
- (C-003/E-C10-005) `StagnationDetection` defaults: `spin_threshold=3`, `oscillation_window=4`, `diminishing_min_window=4`, `diminishing_min_improvement_pct=5.0f32`.
- (C-003/E-C10-006) Detection order is `NoDrift` then `Spinning` then `Oscillation` then `DiminishingReturns`; fewer than two history entries returns `None`.
- (C-003/E-C10-007) `NoDrift` is detected when the final two output hashes are equal.
- (C-003/E-C10-008) `Spinning` is detected when the last threshold-count error classes are identical.
- (C-003/E-C10-009) `Oscillation` is detected when the final four entries alternate between two strings.
- (C-003/E-C10-011) `ThinkingPersona = Hacker | Researcher | Simplifier | Architect | Contrarian` with the same derives as `StagnationPattern`.
- [config] (C-003/E-C10-012) `PersonaStrategy` contains a persona and `&'static str` prompt prefix; `PERSONA_STRATEGIES` is a five-entry static table of short imperative prefixes.
- (C-003/E-C10-013) `suggest_persona_for_pattern`: Spinning→Contrarian, Oscillation→Architect, NoDrift→Hacker, DiminishingReturns→Simplifier.
- [fallback] (C-003/E-C10-014) Researcher is reserved as a fallback rotation when `index % 5 == 4`.
- (C-003/E-C10-015) `GoalDriftCanary` thresholds are green `0.7f32` and yellow `0.4f32`; verdicts are `Green | Yellow | Red`.
- (C-003/E-C10-016) Drift tokenization whitespace-splits, strips non-alphanumeric characters, lowercases, and returns `Vec<String>`.
- (C-003/E-C10-017) Drift similarity is Jaccard over `BTreeSet`, expressly not `HashSet`.
- (C-003/E-C10-018) `WatchdogTimeoutKind = Idle | NoMaterialProgress | Safety`.
- (C-003/E-C10-019) Per-task `GenerationWatchdog` defaults: idle 120,000ms, no-progress 600,000ms, safety 1,800,000ms, and `workflow_fingerprint:Option<u64>`.
- (C-003/E-C10-020) Fingerprint is blake3 over the 16-byte concatenation of `ssb_row_count:u64` and `output_hash:u64`, reduced to the first eight bytes as `u64`; absence of the crate is a hard error.
- (C-003/E-C10-021) `check(last_event_ms,now_ms,current_fingerprint,start_ms)` returns at most one kind in precedence `Safety > NoMaterialProgress > Idle` and updates the stored fingerprint after checking.
- (C-003/E-C10-022) `AdaptiveFutilityTracker` stores per-error-class success and attempt counts in `HashMap<String,u32>` and exposes `record`, `success_rate`, and `is_futile`.
- [fallback] (C-003/E-C10-023) Unseen error classes get a 0.5 prior; futility means success rate below the supplied threshold after at least four attempts; a trigger serializes tracker evidence to SSB.
- [gate] (C-003/E-C10-024) Extend `GateContext` with empty-default `execution_history`, zero-default `attempt_index`, and `suggested_prompt_prefix:Option<&'static str>` defaulting to `None`.
- (C-003/E-C10-025) In `decide()`, detect after the existing `EscalationTrigger::AntiSpiral` check; emit a spinning/oscillation stagnation row, use a payload pattern for NoDrift/DiminishingReturns, emit `LateralThinkingActivated`, and attach the selected prefix.
- (C-003/E-C10-026) At each completed baby step, compare original task tokens with current subtask; red drift emits `GoalDriftWarning` with score plus the first 80 characters of original/current text.
- [failure] (C-003/E-C10-027) Each heartbeat checks the watchdog; timeout emits kind/elapsed/fingerprint-hex and sets `AutonomyDecision::Escalate(EscalationTrigger::AntiSpiral)` using `tokio::time::Instant`.
- [failure] (C-003/E-C10-028) Before repair retries, a futile error class at threshold 0.15 skips remaining retries and escalates `CircuitBreaker`, including serialized tracker detail in an SSB row.
- [config] (C-003/E-C10-029) All five persona files have about 30 lines, front matter (`name`, `description`, `model:sonnet`, tools `[Read,Grep,Glob,Bash,WebSearch]`), and instructions.
- (C-003/E-C10-030) Hacker proposes three shortest `DIRTY/CLEAN/HYBRID` paths and one concrete baby step; Researcher performs three web searches and outputs the best reference/next action.
- (C-003/E-C10-031) Simplifier reduces to a minimal skeleton and restores layers until failure, naming the bad layer; Architect maps the oscillating-error dependency graph and emits a decoupling step.
- (C-003/E-C10-032) Contrarian reverses the assumption, derives an alternative baby step, and does not hedge.
- [config] (C-003/E-C10-035) Add `[stagnation]` with enabled=true, counters initially zero, `spin_threshold=3`, and `adaptive_futility_threshold=0.15`.
- (C-003/E-C10-036) Extended-contract tests cover spinning threshold, identical-hash NoDrift, alternating-pair oscillation, sub-5% diminishing return, and single-entry `None`.
- (C-003/E-C10-037) Extended-contract tests cover all four pattern→persona affinities.
- (C-003/E-C10-038) Drift tests cover green above .7, red below .4, and punctuation stripping.
- (C-003/E-C10-039) Watchdog tests cover safety ceiling, unchanged-fingerprint no-progress, and no-event idle.
- (C-003/E-C10-040) Futility and ring-buffer tests cover below-threshold futility and capacity eviction.
- (C-003/E-C10-041) Five integration tests require spinning emit after three errors, lateral emit with persona, attached prefix, red-drift emit, and watchdog emit plus escalation.
- [gate] (C-003/E-C10-042) Exactly 17 extended plus five integration tests are planned; existing autonomous-gate (at least 14) and phase-4G (at least 18) suites must not regress.
- (C-003/E-C10-043) Autonomous impossible-bug smoke expects iteration-3 spinning, persona activation and next-prompt prefix, drift below .4 if applicable, and no-progress termination at ten minutes.
- (C-003/E-C10-044) No `.unwrap()` in new paths; BTreeSet and blake3 are mandatory; futility evidence cannot be silently dropped; persona front matter must match the existing format.
- (C-003/DELTA+-C10-1) Adds the complete four-pattern enum, ordered detector rules, full schemas, all timeout values, exact persona affinities/instructions, futility prior/minimum/threshold, event query list, badge map, and 22-test coverage.
- (C-003/DELTA+-C10-2) The summary omits precedence (`NoDrift` first; `Safety > NoProgress > Idle`) and fallback behavior (skip retries→CircuitBreaker versus watchdog→AntiSpiral).
- (C-003/DELTA!-C10-3) The summary's “11 types” is correct only for the acceptance enumeration; the module also contains the `PERSONA_STRATEGIES` static and multiple functions. Researcher is not selected by any explicit pattern mapping.
- (C-003/C-C10-01) Drift determinism bans `HashSet`, but the production futility tracker explicitly uses std `HashMap`, whose serialized iteration order is not constrained (L69-L70, L81-L85, L211-L215).
- [fallback] (C-003/A-C10-01) “Researcher fallback rotation (`index % 5 == 4`)” defines neither the index nor when fallback runs; no `StagnationPattern` maps to Researcher (L63-L66).
- (C-003/A-C10-02) Boundary semantics for exactly 0.7/0.4, empty-set Jaccard, diminishing-return percentage (including zero denominator), oscillation string field, and equal-hash precedence over spinning are not further specified (L52-L71).
- (C-003/A-C10-03) Byte endianness for fingerprint input/output, exact no-progress comparison semantics, and reconciliation of millisecond arguments with `tokio::time::Instant` are absent (L73-L80, L106-L111).
- (C-003/A-C10-04) Only two stagnation event variants exist for four patterns; the source says payload disambiguates the other two but does not say which variant carries each (L91-L96).
- (C-003/A-C10-05) The SSB action type carrying `AdaptiveFutilityTracker` detail is unnamed, and the fate of a timed-out task beyond “sets Escalate/terminates” is not defined (L113-L118, L218-L225).
- (SUMMARY/S-E0132) Adds StagnationDetection with a spin threshold of three, an oscillation window of four, and a minimum diminishing-improvement threshold of 5%. [L130]
- (SUMMARY/S-E0141) Adds StagnationSection to OrchestrationDrawer. [L133]
- (SUMMARY/S-E0142) Adds get_stagnation_events as a Tauri command. [L133]
- (SUMMARY/S-E0127) Closes phase-4G 9.x and B-22; depends on C-9 and C-2; is parallel-safe with C-11 through C-15. [L126-L129]
- (SUMMARY/S-E0128) Creates anti_spiral_extended.rs and describes it as containing 11 types. [L130]
- (SUMMARY/S-E0129) Adds StagnationPattern with four variants, whose names are not present in the summary. [L130]
- (SUMMARY/S-E0130) Adds AttemptResult. [L130]
- (SUMMARY/S-E0131) Adds ExecutionHistory with capacity 32. [L130]
- (SUMMARY/S-E0133) Adds ThinkingPersona with five variants and a PERSONA_STRATEGIES table via PersonaStrategy. [L130]
- (SUMMARY/S-E0134) Adds GoalDriftCanary based on Jaccard similarity over BTreeSet, with green at 0.7 and yellow at 0.4. [L130]
- (SUMMARY/S-E0135) Adds DriftVerdict. [L130]
- (SUMMARY/S-E0136) Adds WatchdogTimeoutKind. [L130]
- (SUMMARY/S-E0137) Adds GenerationWatchdog using a blake3 fingerprint and separately tracking idle, no-progress, and safety timeouts. [L130]
- (SUMMARY/S-E0138) Adds AdaptiveFutilityTracker. [L130]
- (SUMMARY/S-E0139) Wires the primitives into autonomous_gate.decide and task_runner step boundaries. [L131]
- (SUMMARY/S-E0140) Creates persona agent definitions for hacker, researcher, simplifier, architect, and contrarian. [L132]
- (SUMMARY/S-E0143) Requires 22 contract tests across two files. [L134]
- (SUMMARY/S-E0144) Autonomous smoke attacks an impossible bug and expects stagnation_spinning_detected at iteration three, lateral_thinking_activated, and watchdog_timeout. [L135]

### C-11: Tier-C sandbox isolation completion

**Status**: planned
**Anchors**: ledgers C-003, SUMMARY | sample anchors: L227, L245-L252; L1-L5, L30-L31, L47-L67; L49-L54; L56-L60; L58-L60; L62-L66 | 36 deep atoms, 13 summary atoms | IDs: C-003/E-C11-024, C-003/E-C11-001, C-003/E-C11-002, C-003/E-C11-003, C-003/E-C11-004, …
**Composes with**: C-3, C-9, C-10, C-12, C-15. IX: SUMMARY/S-E0145 → C-9, C-12, C-15; C-003/I-X-02 → C-3, C-10

- (C-003/E-C11-024) Explicit review is required before merge for clone/fork syscall ordering, tasks 8-13, symlink creation, and raw/undocumented Win32 handle/API paths; PR description must carry the specified safety-critical flag.
- [security] (C-003/E-C11-001) In Linux `clone_cb`, apply capability bounding-set drops, five resource limits, `/proc` masking, and existing seccomp, with seccomp required last.
- [failure] (C-003/E-C11-002) `RLIMIT_CPU` soft/hard = 60/90 seconds; `RLIMIT_NOFILE=256`, `RLIMIT_NPROC=32`, `RLIMIT_AS=512MB`, `RLIMIT_FSIZE=100MB` (last four soft=hard); any setrlimit failure `_exit(124)`.
- (C-003/E-C11-003) Rlimits use stack `libc::rlimit` plus named constants; no heap allocation or magic numbers.
- [failure] (C-003/E-C11-004) Mount `proc` on `/proc` with `MS_NOSUID|MS_NOEXEC|MS_NODEV` and `hidepid=2` in the private `CLONE_NEWNS` child; every mount failure, including `EPERM`, `_exit(123)`.
- [fallback] (C-003/E-C11-005) Unprivileged CI is expected to hit `EPERM`; existing `CLONE_NEWNS::need_cap` guards skip those tests.
- [failure] (C-003/E-C11-006) After `PR_SET_NO_NEW_PRIVS`, loop capability 0 through `CAP_LAST_CAP` (40 for 5.15+), call `prctl(PR_CAPBSET_DROP,...)`, continue on `EINVAL`, and `_exit(122)` on any other error.
- (C-003/E-C11-007) Use libc's `PR_CAPBSET_DROP` when present or verified literal fallback 24 when absent.
- (C-003/E-C11-008) `run_agent_session` gains `Option<Arc<dyn IpcHookDispatcher>>`; call `on_frame_routed(&frame,&route_result)` after routing and before writing the response.
- (C-003/E-C11-009) `ProductionAgentRpcServer` supplies `Some(Arc::new(IpcHookDispatcher::default()))`; dependency-cycle absence must be verified first, and production may not feature-disable the hook.
- (C-003/E-C11-010) Extend `test_4a_isolation_pid_namespace_real`: read `/proc/{child_pid}/status`, parse `NSpid`, and assert the innermost PID is 1 under Linux/EPERM guards.
- (C-003/E-C11-011) Read `/proc/{child_pid}/limits`, parse `Max address space`, and assert hard limit `536870912`.
- [security] (C-003/E-C11-012) Under `cfg(windows)`, call `GetTokenInformation(TokenAppContainerSid)` on the spawned process token/handle and require a non-null AppContainer SID.
- [security] (C-003/E-C11-013) `test_4a_adv_ipc_u32_max_length_rejected`: `u32::MAX` length yields `IpcFrameError::PayloadTooLarge` before allocation.
- [security] (C-003/E-C11-014) `test_4a_adv_selfmod_path_traversal`: `../metrics.rs` through `IpcGateway::route_frame` yields `SelfModViolation`.
- [security] (C-003/E-C11-015) `test_4a_adv_selfmod_bash_write_locked`: a Bash write to a locked path is blocked before dispatch.
- [security] (C-003/E-C11-016) `test_4a_adv_selfmod_git_commit_locked`: a `git commit` targeting a locked path is blocked.
- [fallback/security] (C-003/E-C11-017) `test_4a_adv_selfmod_symlink_resolved`: a symlink redirect to a locked path is realpath-blocked; use platform-specific symlink APIs and skip on permission error.
- (C-003/E-C11-018) After smoke, `[runtime_measurements_b18b]` records rlimit/hidepid verification, cap-drop count, dispatcher wiring=true, adversarial count=5, NSpid/Windows results, and `completion=true`.
- (C-003/E-C11-019) Decision doc records 512MB over the earlier 2GB, child-side hidepid rather than parent `nsenter`, and cap-drop→rlimit→seccomp ordering.
- [security] (C-003/E-C11-020) Linux smoke expects namespace evidence, `/etc/passwd` `EACCES`, no `SANDBOX_VIOLATION`, and visible 512MB address-space hard limit.
- [security] (C-003/E-C11-021) Windows smoke expects an AppContainer SID and task completion without escape.
- [gate] (C-003/E-C11-022) Existing 22 adversarial tests plus five additions yield at least 27 passing; isolation tests and workspace check must pass.
- (C-003/E-C11-023) Rlimits must be real OS calls, hook callback must execute in the live frame loop, NSpid test must read `/proc`, and adversarial tests must exercise real routing.
- (C-003/E-C11-025) Modify `sandbox.rs`, `ipc_accept.rs`, `production_agent_rpc.rs`, both integration-test files, phase-4A metrics; add `C-11-decisions.md`.
- [gate] (C-003/E-C11-026) C-11 closes only after end-to-end Tier-C proof on Linux and Windows; the stated next step is C-12/prioritization.
- (C-003/DELTA+-C11-1) Adds exact exit codes 122/123/124, mount flags, cap-loop compatibility rule, hook timing/signature, five adversarial identifiers/expectations, measurement fields, and all review targets.
- (C-003/DELTA!-C11-2) The summary's exact hidepid placement is not supported by the original, and its unqualified Windows wording masks the `NtQueryInformationProcess`/`GetTokenInformation` inconsistency.
- (C-003/DELTA+-C11-3) The original explicitly makes merge authority a user safety gate; test skipping is limited to capability/permission conditions and is not a production fallback.
- (C-003/C-C11-01) The summary supplies the total order cap-drop→rlimit→hidepid→seccomp, but the original only fixes cap-drop→rlimit→seccomp; hidepid is merely “before execvp” and is absent from both explicit ordering statements (L30-L31, L56-L67, L125-L128).
- (C-003/C-C11-02) The context identifies a missing Windows test “via `NtQueryInformationProcess`,” the task specifies `GetTokenInformation`, and the safety gate names both APIs; no single required API path is consistent (L24-L28, L90-L93, L245-L250).
- (C-003/A-C11-01) Smoke says `NSpid != 1`, while the actual test requires the innermost PID to equal 1; it is unclear whether smoke means the outer PID, the full multi-value field, or a different check (L82-L85, L157-L164, L217-L222).
- (C-003/A-C11-02) The plan attributes `/etc/passwd` `EACCES` to filesystem masking, but this patch's new mount only masks `/proc`; the pre-existing mechanism expected to deny `/etc/passwd` is not identified (L56-L60, L157-L164).
- (C-003/A-C11-03) Production behavior on an unprivileged hidepid mount is a hard child exit, while only tests are allowed to skip; deployment capability/fallback policy is unstated (L56-L60).
- (C-003/A-C11-04) `cap_drop_count` semantics (attempted, successful, or supported caps), hook callback failure behavior, and whether hook dispatch is sync/async are unspecified (L62-L79, L120-L123).
- (SUMMARY/S-E0145) Closes B-18 and phase-4A; depends on C-9; is parallel-safe with C-12 through C-15; is explicitly safety-critical and requires user review. [L137-L140]
- (SUMMARY/S-E0146) In clone_cb, applies controls in this order: PR_CAPBSET_DROP loop, resource limits, hidepid=2 mount, and existing seccomp last. [L141]
- (SUMMARY/S-E0147) Sets RLIMIT_CPU soft/hard values to 60/90 seconds. [L141]
- (SUMMARY/S-E0148) Sets RLIMIT_NOFILE to 256. [L141]
- (SUMMARY/S-E0149) Sets RLIMIT_NPROC to 32. [L141]
- (SUMMARY/S-E0150) Sets RLIMIT_AS to 512 MB. [L141]
- (SUMMARY/S-E0151) Sets RLIMIT_FSIZE to 100 MB. [L141]
- (SUMMARY/S-E0152) Wires IpcHookDispatcher into run_agent_session and ProductionAgentRpcServer. [L142]
- (SUMMARY/S-E0153) Extends the real PID-namespace isolation test to read NSpid from /proc/{pid}/status. [L143]
- (SUMMARY/S-E0154) Adds a test that the address-space limit is visible. [L144]
- (SUMMARY/S-E0155) Adds a Windows AppContainer SID test using GetTokenInformation. [L144]
- (SUMMARY/S-E0156) Adds five adversarial tests covering u32::MAX, self-modification path traversal, locked bash/git writes, and resolved symlinks. [L145]
- (SUMMARY/S-E0157) Cloud smoke asks for a harmless script that attempts to read /etc/passwd and expects NSpid not equal to 1, EACCES for the read, and no SANDBOX_VIOLATION event. [L146]

### C-12: Per-level audit verdicts and AuditTimelineWindow

**Status**: planned
**Anchors**: ledgers C-002, SUMMARY | sample anchors: C-12:L98-L102; C-12:L239-L251; L155; C-12:L1-L10; C-12:L11-L24; C-12:L38-L44 | 24 deep atoms, 14 summary atoms | IDs: C-002/C12-E13, C-002/C12-E19, C-002/C12-E01, C-002/C12-E02, C-002/C12-E03, …
**Composes with**: C-1, C-2, C-11, C-13, C-14, C-15. IX: C-002/I06 → C-2; SUMMARY/S-E0158 → C-1, C-2, C-11, C-13, C-14, C-15; C-002/C12-E02 → C-1, C-2, C-11, C-13, C-14, C-15

- Superseded: (C-002/C12-E13) `get_per_level_verdicts` gains AppState, scans SSB for the task ID and action types `per_level_audit_verdict` plus `per_level_audit_cascade`, deserializes `Vec<LevelAuditVerdictJson>`, and replaces `Ok(Vec::new())`. [C-12:L98-L102]
- Superseded: (SUMMARY/S-E0166) Replaces get_per_level_verdicts returning an empty vector with live SSB-backed data using the B-36 AppState pattern. [L155]
- Superseded: (C-002/SUP-C05) NEWER-PLAN PRECEDENCE: C-27 description is target state relative to older C plan(s) C-1, C-4, C-5, C-12, C-19 for this conflict. Losing description retained at C-002/C05. Conflict: Provider mode used by multiple smoke commands conflicts with C-5's corrected CLI axis. | Evidence: C-1/C-4/C-12/C-19/C-27 use `elai /task --mode cloud`, while C-5 says `--mode` is gate mode and provider is selected by `--provider`. [C-1:L228-L232; C-4:L195-L196; C-12:L152-L155; C-19:L236-L238; C-27:
- (C-002/C12-E13) `get_per_level_verdicts` gains AppState, scans SSB for the task ID and action types `per_level_audit_verdict` plus `per_level_audit_cascade`, deserializes `Vec<LevelAuditVerdictJson>`, and replaces `Ok(Vec::new())`. [C-12:L98-L102]
- (C-002/C12-E19) Writes are limited to navigator integration, the Tauri command, conditional prompt completion, two contract tests, and a C-12 decision document. [C-12:L239-L251]
- (C-002/C12-E01) Objective: connect the existing per-level dispatcher at five navigator tiers, ensure six mode-specific prompt blocks, pass `FramedIdea` to Concepts, emit started/verdict/cascade rows, and make AuditTimeline consume live verdicts. [C-12:L1-L10]
- [status] (C-002/C12-E02) Requires C-1/C-2 and shipped B-41 types/scaffold; is parallel-safe with C-11, C-13, C-14, and C-15. B-41 is described as MOSTLY_DONE with navigator integration as the remaining gap. [C-12:L11-L24]
- (C-002/C12-E03) Five completion mappings are explicit: concept phases resolved→`Concepts`, phase steps resolved→`Phases`, step baby-steps resolved→`Steps`, authored baby-step batch→`BabySteps`, and authored detail→`BabyStepDetail`. [C-12:L38-L44]
- [config] (C-002/C12-E04) `.claude/agents/auditor.md` must contain six substantive (~80 LOC) REVIEW blocks named Concepts, Phase, Step, BabyStep, BabyStepDetail, and Cosmos; missing/placeholders are added, and Concepts accepts `framed_idea`. [C-12:L68-L72; C-12:L208-L212]
- [fallback] (C-002/C12-E05) Verify `FramedIdea` is available at the Concepts call and forward it; acceptance adds that an unframed task warns and falls back rather than hard-aborting. [C-12:L74-L78; C-12:L213-L214]
- [failure] (C-002/C12-E06) Before each of five auditor spawns, call `per_level_audit_dispatch::emit_started(&ssb,agent_id,superstep,mode,tier_node_id)` using the existing typed node ID. Emit failures are warnings and cannot abort the pipeline. [C-12:L80-L85; C-12:L215-L216]
- (C-002/C12-E07) After each auditor returns `LevelAuditVerdict`, call `record_verdict(&ssb,agent_id,superstep,verdict)` at all five tiers. [C-12:L87-L90; C-12:L217-L218]
- (C-002/C12-E08) `CascadeOutcome::Pass` continues unchanged. [C-12:L90-L92]
- (C-002/C12-E09) `CascadeOutcome::RespawnOnce` respawns the same auditor with the updated verdict through existing bubble-up logic; a monotonic guard must prevent double respawn. [C-12:L92-L94; C-12:L219-L220]
- (C-002/C12-E10) `CascadeOutcome::Cascade` bubbles to the parent tier decomposer through the existing all-resolved path. [C-12:L94-L95; C-12:L221-L221]
- (C-002/C12-E11) `CascadeOutcome::EscalateToIdea` routes to the B-40 FRAMING retry and emits an ESCALATE log. [C-12:L95-L96; C-12:L222-L222]
- [failure] (C-002/C12-E12) `record_verdict` errors must surface as ESCALATE rather than being silently swallowed; no wildcard cascade arm is permitted. [C-12:L147-L150; C-12:L217-L218]
- (C-002/C12-E14) Existing `AuditTimelineWindow.svelte` binding/props remain; live output renders mode-colored markers and arrows where `cascade_to_parent=true`. [C-12:L104-L107; C-12:L225-L227]
- (C-002/C12-E15) `navigator_audit_wiring_contract.rs` builds 3 concepts × 1 phase × 2 steps × 2 baby steps, runs all five completions, asserts five distinct mode-correct verdicts, and enforces started-before-verdict sequence order. [C-12:L109-L117]
- (C-002/C12-E16) `audit_timeline_command_contract.rs` seeds five verdict and two cascade rows, expects exactly five JSON entries, and expects two `cascade_to_parent` mappings. [C-12:L119-L125]
- (C-002/C12-E17) Cloud smoke builds a three-concept Rust project and requires five distinct, correctly-modeled verdict rows, started-before-verdict order, a command result of at least five entries, five UI markers, and conditional cascade arrows. [C-12:L152-L163; C-12:L233-L237]
- [gate] (C-002/C12-E18) Mechanical checks require import presence, all five enum modes, exactly five `emit_started` and five `record_verdict` references, all four outcome arms, removal of the empty stub, both contract files, clean scoped compilation, and both passing test filters. [C-12:L165-L206]
- (C-002/SD-C-12-01) Addition: The index omits the sixth Cosmos prompt block, singular-prompt/plural-enum naming, typed tier IDs, non-aborting emit failure, ESCALATE-on-record-error, monotonic no-double-respawn guard, and unframed warning/fallback. [C-12:L68-L96; C-12:L147-L150; C-12:L208-L222]
- (C-002/SD-C-12-02) Addition: The command merges verdict and cascade action types into five JSON entries; the two exact contract fixtures and started-before-verdict ordering are stronger than the summary. [C-12:L98-L125]
- (SUMMARY/S-E0166) Replaces get_per_level_verdicts returning an empty vector with live SSB-backed data using the B-36 AppState pattern. [L155]
- (SUMMARY/S-E0158) Closes B-41 and phase-index-3A-4J; depends on C-1 and C-2 while reusing B-41 types; is parallel-safe with C-11, C-13, C-14, and C-15. [L148-L151]
- (SUMMARY/S-E0159) Calls per_level_audit_dispatch::emit_started and record_verdict at completion of Concepts, Phases, Steps, BabySteps, and BabyStepDetail. [L152]
- (SUMMARY/S-E0160) Handles CascadeOutcome Pass. [L153]
- (SUMMARY/S-E0161) Handles CascadeOutcome RespawnOnce. [L153]
- (SUMMARY/S-E0162) Handles CascadeOutcome Cascade. [L153]
- (SUMMARY/S-E0163) Handles CascadeOutcome EscalateToIdea, invoking the B-40 framing retry path. [L153]
- (SUMMARY/S-E0164) Confirms that auditor.md contains six MODE DISPATCH review prompt blocks. [L154]
- (SUMMARY/S-E0165) Passes FramedIdea into the Concepts auditor. [L154]
- (SUMMARY/S-E0167) Activates AuditTimelineWindow’s already-defined command binding with live data. [L156]
- (SUMMARY/S-E0168) Requires navigator_audit_wiring and audit_timeline_command contract tests. [L157]
- (SUMMARY/S-E0169) Cloud smoke builds a three-concept Rust project and expects five distinct per_level_audit_verdict rows with the correct mode. [L158]

### C-13: IPC tool registry, hook dispatcher, DAIRA markers, and WIP squash

**Status**: planned
**Anchors**: ledgers C-006, SUMMARY | sample anchors: C-13 L1-L6; C-13 L7-L9, L293-L295; C-13 L13-L17; C-13 L18-L20, L29-L30; C-13 L22-L25, L88-L89, L271-L275; C-13 L27 | 45 deep atoms, 13 summary atoms | IDs: C-006/C13-E001, C-006/C13-E002, C-006/C13-E003, C-006/C13-E004, C-006/C13-E005, …
**Composes with**: C-6, C-11, C-12, C-14, C-15, C-23, C-39, C-40. IX: C-006/C39-I001 → C-6, C-39; C-006/C40-X003 → C-6, C-23, C-40; SUMMARY/S-E0170 → C-11, C-12, C-14, C-15; C-006/C13-I002 → C-6; C-006/C13-E002 → C-11, C-12, C-14, C-15; C-006/C13-X001 → C-6

- Superseded: (C-006/C13-E009) At run_subcommand.rs:149 and :221, IpcToolRegistry::new(Arc::new(validator)) replaces InProcessToolDispatcher; its existing FSM gate at ipc_registry.rs:112-146 applies automatically. [C-13 L64-L71, L166, L255-L256]
- Superseded: (SUMMARY/S-E0171) Replaces InProcessToolDispatcher with IpcToolRegistry at the two specified run_subcommand.rs call sites. [L164]
- Superseded: (SUMMARY/S-E0894) C-13 replaces production in-process dispatch with IPC and wraps tools with DAIRA marker emission, while retaining in-process operation only for tests. [L160-L170]
- (C-006/C13-E001) Objective: replace the two production InProcessToolDispatcher sites with IpcToolRegistry, install production IpcHookDispatcher plus RuffPostToolUseRunner, add AdaptiveTool<T> for ten missing DAIRA markers, and squash WIP commits at AUDITING -> MEMORIZING. [C-13 L1-L6]
- (C-006/C13-E002) Estimated two sessions; C-11 sandbox completeness is a hard prerequisite; parallel-safe with C-12, C-14, and C-15. [C-13 L7-L9, L293-L295]
- (C-006/C13-E003) Verified baseline: InProcessToolDispatcher is live at run_subcommand.rs:149 and :221, while IpcToolRegistry appears only in crates/tools/tests/tool_dispatch.rs. [C-13 L13-L17]
- (C-006/C13-E004) Verified hook baseline: IpcHookDispatcher has no production caller, apply_task_runner_wiring leaves the hook unset, and with_ruff_runner is test-only. [C-13 L18-L20, L29-L30]
- (C-006/C13-E005) The five pre-existing DAIRA emissions are Call, Return, Error, HookPre, and HookPost; the ten absent emissions are Mutation, Branch, Finding, Read, Write, ToolCallStart, ToolCallEnd, Timeout, Escalate, and Converge. This resolves the summary's unnamed “five pre-existing markers.” [C-13 L22-L25, L88-L89, L271-L275]
- (C-006/C13-E006) No wip_commit, squash_commit, or WipSquash implementation existed at the verified baseline. [C-13 L27]
- (C-006/C13-E007) RuntimeWiring gains hook_dispatcher: Option<Arc<dyn HookDispatcher + Send + Sync>>, defaults it to None, and apply_task_runner_wiring copies it into TaskRunnerConfig; the path forbids unwrap, expect, and panic. [C-13 L47-L53, L158-L168, L259-L260]
- (C-006/C13-E008) build_production_hook_dispatcher(ssb: Arc<Ssb>) -> Arc<IpcHookDispatcher> constructs RuffPostToolUseRunner, installs it with with_ruff_runner, and is called by both run_subcommand wiring paths using wiring.ssb.clone(). [C-13 L55-L62, L163-L164, L260-L261]
- (C-006/C13-E009) At run_subcommand.rs:149 and :221, IpcToolRegistry::new(Arc::new(validator)) replaces InProcessToolDispatcher; its existing FSM gate at ipc_registry.rs:112-146 applies automatically. [C-13 L64-L71, L166, L255-L256]
- (C-006/C13-E010) InProcessToolDispatcher remains available only under cfg(any(test, feature = "in_process_dispatcher")); elai_tools Cargo.toml adds in_process_dispatcher = []; non-test production references are banned. [C-13 L71-L73, L96-L102, L167, L184-L186, L257-L258]
- [gate] (C-006/C13-E011) AdaptiveTool<T: ToolHandler> delegates to an inner handler, is exported from elai_tools::adaptive_tool and the crate root, wraps every handler at IpcToolRegistry::register time, and uses only fallible/if-let emission paths. [C-13 L75-L78, L90-L94, L161, L262-L263]
- (C-006/C13-E012) AdaptiveTool emits ToolCallStart before dispatch. [C-13 L79-L80]
- (C-006/C13-E013) A file-read path emits Read. [C-13 L81]
- (C-006/C13-E014) A file-write path emits Write. [C-13 L82]
- (C-006/C13-E015) A file-write path additionally emits Mutation only when content changed. [C-13 L82]
- (C-006/C13-E016) A test-assertion path always emits Branch. [C-13 L83]
- (C-006/C13-E017) A test-assertion result emits Finding. [C-13 L83]
- (C-006/C13-E018) A deadline breach emits Timeout. [C-13 L84, L152-L153]
- (C-006/C13-E019) Escalation/error handling emits Escalate. [C-13 L85, L153]
- (C-006/C13-E020) A clean converge/done outcome emits Converge. [C-13 L86, L154]
- (C-006/C13-E021) AdaptiveTool emits ToolCallEnd after dispatch. [C-13 L87]
- (C-006/C13-E022) AdaptiveTool must not duplicate Call, Return, Error, HookPre, or HookPost; task_runner remains their owner, and Call/Return/Error must each occur exactly once in coverage tests. [C-13 L88-L89, L155-L156, L188-L189]
- (C-006/C13-E023) squash_wip_commits(project_root: &Path) runs git log --oneline --no-decorate, captures stdout, and counts consecutive leading commit lines matching a WIP subject after the hexadecimal hash. [C-13 L104-L110]
- (C-006/C13-E024) If the leading WIP count is greater than five, it runs git reset --soft HEAD~{count}, then git commit --no-verify -m "phase: squash {count} WIP commits". [C-13 L111-L112, L142-L143]
- (C-006/C13-E025) A count of five or fewer is a successful no-op that leaves all commits unchanged. [C-13 L113, L144-L145]
- (C-006/C13-E026) Every git subprocess captures stderr and maps non-zero exit to anyhow error; on success the SSB payload is {wip_count, squashed: bool}. [C-13 L114-L116]
- (C-006/C13-E027) The squash runs before handoff at the AUDITING -> MEMORIZING transition. [C-13 L117, L179-L182, L264]
- (C-006/C13-E028) SsbActionType gains WipSquashComplete and WIP_SQUASH_COMPLETE = "wip_squash_complete", following the existing enum/category pattern. [C-13 L119-L123, L165, L265]
- (C-006/C13-E029) ipc_dispatcher_production_contract.rs defines ten named contracts: test_ipc_tool_registry_is_production_dispatcher; test_in_process_tool_dispatcher_is_cfg_test_only; test_runtime_wiring_has_hook_dispatcher_field; test_apply_task_runner_wiring_sets_hook_dispatcher; test_build_production_hook_dispatcher_returns_ipc_hook_dispatcher; test_ruff_runner_installed_in_production_hook_dispatcher; test_adaptive_tool_emits_tool_call_start_and_end; test_adaptive_tool_emits_read_write_mutation; test_wip_squash_triggers_at_6_wip_commits; test_wip_squash_noop_at_5_or_fewer. [C-13 L125-L145]
- (C-006/C13-E030) adaptive_tool_daira_coverage_contract.rs defines five named contracts: test_all_15_daira_markers_emitted_across_tool_lifecycle; test_timeout_marker_emitted_on_deadline_breach; test_escalate_marker_emitted_on_error; test_converge_marker_emitted_on_success; test_adaptive_tool_does_not_duplicate_call_return_error. [C-13 L147-L156]
- (C-006/C13-E031) Consumed contracts are IpcToolRegistry plus its FSM gate, IpcHookDispatcher, RuffPostToolUseRunner, with_ruff_runner, emit_daira_trace, the 15-entry DAIRA_MARKERS constant, and C-11's process boundary. [C-13 L170-L177]
- (C-006/C13-E032) Reachability is from both elai run / elai /task wiring sites, apply_task_runner_wiring as production chokepoint, and the AUDITING -> MEMORIZING transition. [C-13 L179-L182]
- [security] (C-006/C13-E033) Stub/security constraints prohibit a production InProcessToolDispatcher, an unset production hook dispatcher, a non-git squash implementation, duplicated base DAIRA markers, and unwrap/expect/panic in new production paths. [C-13 L184-L189]
- (C-006/C13-E034) Cloud-Sonnet smoke implements and tests src/sum.rs, proves IPC-only dispatch, all 15 named DAIRA strings, Ruff hook_post_tool_use on Python writes, and one squash commit when more than five WIPs preceded transition. [C-13 L191-L197, L268-L278]
- [gate] (C-006/C13-E035) Static gates require at least two IpcToolRegistry references, zero run_subcommand InProcessToolDispatcher references, a feature gate, at least three hook_dispatcher references, one builder, one AdaptiveTool struct, at least ten new marker strings, one squash function, and at least two WipSquash references. [C-13 L199-L240]
- [gate] (C-006/C13-E036) Done gates require all 15 contracts and cargo check across elai_tools, elai_orchestrator, elai_cli, and elai_contracts with zero errors. [C-13 L242-L250, L266-L267]
- (C-006/C13-E037) Planned artifacts comprise seven production-file modifications, adaptive_tool.rs, two contract files, an elai_tools feature declaration, and workflow/reasoning/C-13-decisions.md. [C-13 L280-L291]
- (C-006/C13-E038) The plan itself records no completed checkbox; its closing instruction still says to confirm C-11 shipped before opening the session. [C-13 L253-L278, L293-L295]
- (C-006/C13-A001) The source does not say whether wip_squash_complete is emitted for the <=5 no-op; “on success” and its squashed boolean suggest yes, while acceptance says it is emitted “on squash.” [C-13 L113-L115, L264-L265]
- (C-006/C13-A002) The proposed downcast assertion for Arc<IpcHookDispatcher> does not specify an Any/downcast facility on the HookDispatcher trait. [C-13 L134-L137]
- (C-006/C13-A003) Marker classification for arbitrary handlers—how a dispatch is recognized as read, write, assertion, escalation, or convergence—is not specified beyond path labels. [C-13 L75-L93]
- (C-006/C13-A004) No locking or re-entry policy protects the git history mutation if multiple transitions target one project concurrently.
- (C-006/C13-I001) After a successful squash, the replacement subject begins with “phase:” rather than “WIP”; rerunning against unchanged history should therefore be an effective no-op, but idempotency is not an explicit contract.
- [gate] (C-006/C13-I002) The IPC registry's pre-existing FSM gate is intended to preserve authorization ordering across the production swap; the plan does not restate the gate's policy.
- (SUMMARY/S-E0170) Closes B-19 and B-20; depends on C-11; is parallel-safe with C-12, C-14, and C-15. [L160-L163]
- (SUMMARY/S-E0171) Replaces InProcessToolDispatcher with IpcToolRegistry at the two specified run_subcommand.rs call sites. [L164]
- (SUMMARY/S-E0172) Retains in-process dispatch only behind a test feature gate. [L164]
- (SUMMARY/S-E0173) Adds hook_dispatcher to RuntimeWiring and applies it through apply_task_runner_wiring. [L165]
- (SUMMARY/S-E0174) build_production_hook_dispatcher returns IpcHookDispatcher with RuffPostToolUseRunner enabled. [L165]
- (SUMMARY/S-E0175) Adds generic AdaptiveTool<T> in crates/tools/src/adaptive_tool.rs. [L166]
- (SUMMARY/S-E0176) AdaptiveTool emits ten previously missing DAIRA markers: ToolCallStart, ToolCallEnd, Read, Write, Mutation, Branch, Finding, Timeout, Escalate, and Converge. [L166]
- (SUMMARY/S-E0177) On AUDITING to MEMORIZING, task_runner squashes more than five WIP commits into one. [L167]
- (SUMMARY/S-E0178) Adds SsbActionType WipSquashComplete and its WIP_SQUASH_COMPLETE label. [L168]
- (SUMMARY/S-E0179) Requires 15 contract tests across two files. [L169]
- (SUMMARY/S-E0180) Cloud smoke implements and tests a sum function, expects all 15 total DAIRA markers in SSB, expects RuffPostToolUse on a Python write, and expects a visible WIP squash commit. [L170]
- (SUMMARY/S-A0181) The plan adds ten missing markers while smoke expects 15 total markers; the original must identify the five pre-existing markers and exact completeness contract.

### C-14: Delete legacy nondeterministic surfaces and ban usage symbols

**Status**: planned
**Anchors**: ledgers C-003, SUMMARY | sample anchors: L14-L16; L168-L170, L331-L341; L268-L324; L326-L347; L1-L12; L17-L19 | 43 deep atoms, 11 summary atoms | IDs: C-003/E-C14-002, C-003/E-C14-008, C-003/E-C14-024, C-003/E-C14-025, C-003/E-C14-001, …
**Composes with**: C-3, C-15, C-20, C-29, C-45, C-59. IX: SUMMARY/S-E0182 → C-15; C-003/I-X-03 → C-3, C-20, C-29, C-45, C-59; C-003/E-C14-015 → C-15

- Superseded: (C-003/E-C14-015) The preserved draft intended deletion of 624-line `compact.rs`, 591-line `oauth.rs`, 744-line `prompt.rs`, usage stripping, a Windows script fix, and 28+ command classifications, with no prerequisite and C-15 parallel safety.
- Superseded: (C-003/E-C14-019) The original task said delete pricing helpers and leave a B-27 note; the executed structural variant relocates them instead and is authoritative.
- Superseded: (C-003/E-C14-020) The original script task proposed a possibly fictional `seeded_determinism` target; audit proved `bsp_determinism` and POSIX portability were the real issues.
- Superseded: (C-003/E-C14-013) The original entropy-quarantine cloud acceptance is replaced by workspace compilation plus zero banned-symbol matches because the dead code remains.
- (C-003/E-C14-002) `compact.rs` powers `/compact`, `ConversationRuntime::compact`, and `CompactionConfig` imports across eight CLI files.
- (C-003/E-C14-008) `elai_orchestrator::usage::*` re-exports provider pricing so existing CLI imports stay valid; provider/type/Anthropic and live-CLI call sites are rewired.
- [gate/status] (C-003/E-C14-024) Scoped orchestrator+CLI and full workspace compilation are recorded passing on 2026-05-17; banned-symbol grep and 78-status count are recorded done.
- (C-003/E-C14-025) Adds provider pricing and PowerShell proof; modifies core/provider/orchestrator/CLI imports, POSIX proof, and migration manifest; deletion/metrics/test artifacts remain explicitly deferred.
- [status] (C-003/E-C14-001) Pre-execution session `s-32952` found the draft based on stale evidence: `compact.rs`, `oauth.rs`, and `prompt.rs` are live production code with 40+ aggregate call sites, so direct deletion would break shipped behavior.
- (C-003/E-C14-003) `oauth.rs` powers interactive `/login`/`/logout`, doctor login diagnostics, and two contract suites.
- (C-003/E-C14-004) `prompt.rs` provides `SystemPromptBuilder` for every conversation turn, REPL boot, auth/doctor, and `chat_reflect_contract`.
- [deferred/status] (C-003/E-C14-005) Original deletion tasks 1-3, deletion metrics task 7, and absence-test task 8 are deferred to a separate multi-session refactor.
- (C-003/E-C14-006) The four banned pricing identifiers and pricing-dependent `TokenUsage` methods moved from `elai_core::usage` to new `elai_providers::pricing`; behavior and five production call sites were preserved.
- (C-003/E-C14-007) Post-move core `usage.rs` contains `TokenUsage` (four fields plus `total_tokens`) and cumulative `UsageTracker`; core re-exports only those two.
- (C-003/E-C14-009) `crates/core/src` is asserted to contain zero occurrences of `ModelPricing`, `UsageCostEstimate`, `pricing_for_model`, or `format_usd`.
- (C-003/E-C14-010) Existing POSIX script already used real target `--test bsp_determinism` and never used `--workspace`; it is now explicitly documented POSIX-only.
- (C-003/E-C14-011) Add PowerShell 7+ `determinism_proof.ps1` using native temporary-file, SHA-256, and filtering APIs while preserving `ELAI_DETERMINISM_SEED` and two-run byte comparison.
- (C-003/E-C14-012) Append 78 `[[command]]` entries, one per `SlashCommand` variant at the audited enum, each with name, one-line rationale, and `KEPT | STRIPPED | DEFERRED`.
- (C-003/E-C14-013) The original entropy-quarantine cloud acceptance is replaced by workspace compilation plus zero banned-symbol matches because the dead code remains.
- (C-003/E-C14-014) Drift decisions live in `workflow/reasoning/C-14-decisions.md`; the audit package is `workflow/sessions/s-32952/context-package-C14.md`.
- (C-003/E-C14-015) The preserved draft intended deletion of 624-line `compact.rs`, 591-line `oauth.rs`, 744-line `prompt.rs`, usage stripping, a Windows script fix, and 28+ command classifications, with no prerequisite and C-15 parallel safety.
- [deferred/gate] (C-003/E-C14-016) `compact.rs` can only be deleted after a caller sweep proves zero production callers; the re-scope records 30+ callers and defers it.
- [deferred/gate] (C-003/E-C14-017) `oauth.rs` can only be deleted after proving no external OAuth/Cloud dependency; the re-scope identifies it as the live login backend and defers it.
- [deferred/gate] (C-003/E-C14-018) `prompt.rs` can only be deleted after zero callers and a config-loaded replacement; audit found live per-turn use, no replacement, and no stale “interactive agent” string, so deletion is deferred.
- (C-003/E-C14-019) The original task said delete pricing helpers and leave a B-27 note; the executed structural variant relocates them instead and is authoritative.
- (C-003/E-C14-020) The original script task proposed a possibly fictional `seeded_determinism` target; audit proved `bsp_determinism` and POSIX portability were the real issues.
- [deferred] (C-003/E-C14-021) `[dead_code_strip]` with four actual LOC counts is postponed; only usage changed (279→81), the three file-strip counts are zero, and writing the section now was judged misleading.
- [deferred] (C-003/E-C14-022) All seven `dead_code_strip_contract` tests are postponed because three assert absence of files that must remain; safe banned-symbol/re-export checks are covered by deterministic grep.
- (C-003/E-C14-023) Current work is observable from compilation, both determinism scripts, and the migration manifest; it is not a runtime feature removal.
- (C-003/E-C14-026) Follow-up deletion requires explicit product confirmation that compaction, OAuth login, and `SystemPromptBuilder` are to be removed rather than relocated.
- (C-003/E-C14-027) Follow-up requires the phase-0 six-step compact refactor, OAuth migration to `OAuthTokenSet` or deprecation of `/login`, and a new config-loaded prompt resolver.
- (C-003/DELTA!-C14-1) The orientation summary's deletion of all three files, stripped LOC metrics, and seven required tests is stale and false for the authoritative source state; each is postponed.
- (C-003/DELTA!-C14-2) The script was not changed from workspace-wide to `seeded_determinism`; it already targeted `bsp_determinism`, was documented POSIX-only, and gained a PowerShell sibling.
- (C-003/DELTA!-C14-3) Pricing helpers were moved, not removed; 78 rather than “at least 28” command entries were completed.
- (C-003/DELTA+-C14-4) Adds concrete shipped-feature dependencies, completed artifact list, audit evidence paths, replacement acceptance, and explicit product authority required for follow-up.
- (C-003/C-C14-01) The preserved original labels the three files dead/safe to delete, but the authoritative re-scope proves all three live and postpones deletion (L7-L29 versus L64-L92/L113-L148).
- (C-003/C-C14-02) Original task 4 says the pricing API should be removed without inline replacement, while executed status says it was behavior-preservingly relocated and re-exported (L150-L170).
- (C-003/C-C14-03) Original task 5 diagnoses `--workspace` and suggests `seeded_determinism`; executed audit states neither claim was true (L174-L195).
- [gate] (C-003/A-C14-01) Exact distribution of the “40+ aggregate” and `compact.rs` “30+” caller counts is not enumerated, and the 78 per-command classifications are not reproduced in this plan (L7-L22, L121-L122, L203-L207).
- (SUMMARY/S-E0182) Closes B-15 and phase-0; has no direct prerequisites; is parallel-safe with C-15. [L172-L175]
- (SUMMARY/S-E0183) Deletes compact.rs after a caller sweep, with the summary estimating 624 lines. [L176]
- (SUMMARY/S-E0184) Deletes oauth.rs after confirming CloudGroundingClient OAuth lives in cloud.rs, with the summary estimating 591 lines. [L176]
- (SUMMARY/S-E0185) Deletes prompt.rs and the legacy “interactive agent” prompt, with the summary estimating 744 lines. [L176]
- (SUMMARY/S-E0186) Removes ModelPricing, UsageCostEstimate, format_usd, and pricing_for_model from core/src/usage.rs. [L177]
- (SUMMARY/S-E0187) Removes the corresponding re-export from lib.rs. [L177]
- (SUMMARY/S-E0188) Fixes determinism_proof.sh by replacing workspace-wide invocation with the seeded_determinism test, explicitly for Windows safety. [L178]
- (SUMMARY/S-E0189) Adds a command section to migration_manifest.toml containing at least 28 KEEP, STRIPPED, or DEFERRED records. [L179]
- (SUMMARY/S-E0190) Records actual stripped-line counts in pre_phase_0.toml dead_code_strip. [L180]
- (SUMMARY/S-E0191) Requires seven dead_code_strip_contract tests. [L181]
- (SUMMARY/S-E0192) Acceptance requires cargo check across the workspace and zero grep matches for deleted symbols. [L182]

### C-15: ClockProvider migration, FxHashMap, and entropy CI restore

**Status**: planned
**Anchors**: ledgers C-006, SUMMARY | sample anchors: C-15 L31-L41; C-15 L119-L127; C-15 L130-L146; L190; L191; C-15 L1-L7, L43-L48 | 42 deep atoms, 15 summary atoms | IDs: C-006/C15-E006, C-006/C15-E015, C-006/C15-E016, C-006/C15-E001, C-006/C15-E002, …
**Composes with**: C-4, C-6, C-8, C-14, C-16, C-17, C-21, C-25. IX: C-006/C15-I001 → C-6; C-006/C15-I002 → C-6; C-006/C15-E002 → C-14, C-16, C-25; C-006/C15-X003 → C-6; C-006/C15-X001 → C-6; C-004/X-003 → C-4, C-8, C-17, C-21

- Superseded: (C-003/E-C14-015) The preserved draft intended deletion of 624-line `compact.rs`, 591-line `oauth.rs`, 744-line `prompt.rs`, usage stripping, a Windows script fix, and 28+ command classifications, with no prerequisite and C-15 parallel safety.
- Superseded: (C-006/C15-E010) Mechanical map migration removes the std import, imports rustc_hash::FxHashMap, replaces the type, and replaces HashMap::new with FxHashMap::default. [C-15 L89-L91]
- Superseded: (C-006/C15-E017) Injection replaces Instant::now/SystemTime::now with self.clock.instant_now/self.clock.system_now; absent clocks add Arc<dyn ClockProvider> fields and constructor args, except deserializable config types receive injection one call layer above. [C-15 L140-L145]
- Superseded: (C-001/SUP-C30-029) NEWER-PLAN PRECEDENCE: C-15 description is target state relative to older C plan(s) C-0 for this conflict. Losing description retained at C-001/C30-029. Conflict: [E][conflict][Δ+] OracleAnswerCache explicitly uses std HashMap in production, while C-0 says C-15 bans production HashMap in favor of FxHashMap. This cross-plan type conflict must be resolved before implementation. (C30:L114-124; C0:L78-81)
- Superseded: (C-001/SUP-A-014) NEWER-PLAN PRECEDENCE: C-30 description is target state relative to older C plan(s) C-0, C-15 for this conflict. Losing description retained at C-001/A-014. Conflict: [conflict] C-30's std HashMap violates C-0/C-15's production FxHashMap policy. (C30:L114-124; C0:L78-81)
- (C-006/C15-E006) A 2026-05-23 closure rebaseline narrows actual closure to CLI plus knowledge batches; 52 post-closure leaks in orchestrator/plugins/tools/dashboard producer wiring are routed to an unauthored B-15-regression-burndown. [C-15 L31-L41]
- (C-006/C15-E015) Identified CLI exception candidates are commands.rs cache expiry and live_cli.rs/runtime_cmd.rs SSB timestamp milliseconds; inject through session ClockProvider when possible, otherwise use the specified reason. [C-15 L119-L127]
- (C-006/C15-E016) Clock injection batches are: A knowledge reranker/indexing; B rules self_discovery; C plugins tiered_analysis/import_extractor; D focus_tunnel streaming/mode; E top-level orchestrator session_dag/self_mod_layer1a/render adapters; F CLI commands/live_cli/remote_sidecar/runtime_cmd/runtime_config. [C-15 L130-L146]
- (C-006/C15-E001) Initial objective: migrate about 68 production entropy sites, replace std HashMap in ten production files, re-enable Windows entropy CI, add ClockProvider::instant_now, and support reasoned entropy exceptions. The disabled workflow comment still said 16 sites while the audit measured 68; the existing quarantine script already worked. [C-15 L1-L7, L43-L48]
- (C-006/C15-E002) Estimated two sessions; C-14 is prerequisite so dead HashMap-bearing functions are stripped first; initially declared parallel-safe with C-16 through C-25. [C-15 L9-L12]
- (C-006/C15-E003) Baseline ClockProvider is object-safe and exposes now -> LogicalInstant, elapsed_since, and system_now -> SystemTime; instant_now was absent at audit and then noted as landing at clock.rs:54 during C-15.1. [C-15 L18-L23]
- (C-006/C15-E004) SeededClock's AtomicU64 tick starts at zero regardless of ELAI_SEED; ELAI_SEED seeds only the SeedableRng path. [C-15 L50-L52]
- (C-006/C15-E005) Initial scan found ten production source files importing std::collections::HashMap; rustc-hash 2.1 was already a workspace dependency. [C-15 L25-L29]
- (C-006/C15-E007) Change ordering is HashMap migration before clock injection, with a separate commit per group/batch for bisectability. [C-15 L54-L56]
- (C-006/C15-E008) ClockProvider gains object-safe instant_now; OsClock returns Instant::now, while SeededClock uses a lazily established base Instant plus tick milliseconds rather than invalid SystemTime-to-Instant arithmetic. [C-15 L71-L78, L186-L188]
- (C-006/C15-E009) SeededClock receives an exact documentation contract: ELAI_SEED affects RNG only, the first now call returns tick zero, and correctly injected derived/non-clock fields are byte-identical for the same seed. [C-15 L80-L87]
- (C-006/C15-E010) Mechanical map migration removes the std import, imports rustc_hash::FxHashMap, replaces the type, and replaces HashMap::new with FxHashMap::default. [C-15 L89-L91]
- (C-006/C15-E011) The ten reviewed map sites are plugins/tiered_analysis.rs, plugins/import_extractor.rs, dashboard media_validation.rs, focus_tunnel/streaming.rs, focus_tunnel/mode.rs, rules/self_discovery.rs, knowledge/reranker.rs, orchestrator/session_dag.rs, orchestrator/self_mod_layer1a.rs, and media_validator/render_adapters/mod.rs. [C-15 L92-L108]
- (C-006/C15-E012) Exceptions: leave session_dag's map if test-only and document it under the existing INFO allowlist; do not replace self_mod_layer1a string-template text that merely mentions HashMap. [C-15 L101-L107]
- (C-006/C15-E013) Map commits are grouped into plugins, dashboard-v2, orchestrator, rules, and knowledge, with C-15.3a through C-15.3e prefixes. [C-15 L109-L111]
- (C-006/C15-E014) Unavoidable SystemTime sites receive same-line “// entropy-allow: <reason>” only after injection is attempted, plus a file-header manifest of allowed sites; example legitimate classes are filesystem mtime, HTTP timestamp, and TLS validity. [C-15 L113-L128]
- (C-006/C15-E017) Injection replaces Instant::now/SystemTime::now with self.clock.instant_now/self.clock.system_now; absent clocks add Arc<dyn ClockProvider> fields and constructor args, except deserializable config types receive injection one call layer above. [C-15 L140-L145]
- (C-006/C15-E018) check_entropy_quarantine.sh filters same-line entropy-allow annotations with grep -v. [C-15 L148-L152, L269-L272]
- (C-006/C15-E019) windows.yml reactivates the entropy quarantine step and documents source annotations plus clock.rs/entropy.rs globs as its allowlist. [C-15 L154-L158, L273]
- (C-006/C15-E020) A separate named Anti-stale entropy scan runs both check_entropy_quarantine.sh and check_hashmap_ban.sh after the quarantine step to block regressions. [C-15 L160-L170]
- (C-006/C15-E021) clock_provider_injection_contract.rs checks first SeededClock tick zero, identical tick sequences across two instances, SeededClock instant monotonicity from a captured reference, and OsClock instant within one second of Instant::now. [C-15 L172-L182]
- (C-006/C15-E022) Production bans prohibit increasing post-migration std HashMap count, bare time reads outside allowlists, and edits to tests, main.rs entrypoints, or const/static initialization. [C-15 L203-L209]
- [gate] (C-006/C15-E023) Rebaselined HashMap gate is not zero: it allows at most five violations, all in dashboard-v2 commands_d6/d7/d9/e46/d26 until that dashboard quiesces. [C-15 L220-L224, L263-L265]
- [gate] (C-006/C15-E024) Rebaselined entropy gate is not zero: it allows at most 52 residual sites; the measured 2026-05-23 result was 51. Explicit exclusions/allowances cover measure.rs (16), bench_*.rs (2), live_cli.rs (two cited sites), knowledge/startup.rs, and knowledge/cosmos_worker.rs. [C-15 L226-L236, L266-L272]
- (C-006/C15-E025) The observable script contract originally says both quarantine scripts exit zero and the combined gate passes on PR. [C-15 L210-L211]
- (C-006/C15-E026) Determinism smoke runs the same cloud counter task twice at ELAI_SEED=42 and compares byte-identical SSB action_type sequences; raw timestamps are excluded, but derived/non-clock fields must match. [C-15 L277-L282]
- (C-006/C15-E027) That cloud smoke is CLOUD-BILLABLE-PENDING because live billable calls require user authorization; only harness scaffolding is claimed ready. [C-15 L277-L287]
- (C-006/C15-E028) Checked closure items are the reduced map/entropy counts, reasoned annotations, annotation filtering, active quarantine step, and the clock contract; the combined anti-stale step remains explicitly missing. [C-15 L263-L276]
- (C-006/C15-E029) Verification-report updates for B-15, phase-0, and B-16 are explicitly deferred out of C-15 closure. [C-15 L288-L292]
- (C-006/C15-E030) Additional touched exception files are knowledge/startup.rs and knowledge/cosmos_worker.rs; the latter defers a CosmosClock extension. The script also gains measure.rs and bench_*.rs glob exclusions. [C-15 L312-L315]
- (C-006/C15-E031) The decisions artifact must record per-file rationale, measure.rs Option A/B treatment, billable-smoke status, and the dashboard carve-out. [C-15 L39-L41, L317]
- [status] (C-006/C15-E032) Final plan status is partial/mixed rather than wholly PLANNED or DONE: several acceptance boxes are checked, several remain open, and residual cleanup is deferred. [C-15 L258-L292]
- (C-006/C15-A001) The contract-test task lists four bullet tests, while acceptance calls them “5 assertions”; the fifth assertion is not named. [C-15 L172-L182, L276]
- (C-006/C15-A002) “Major” production entropy scope changed during execution, but the source never publishes a final per-file list of all 51 residual occurrences—only buckets and selected sites. [C-15 L226-L236]
- (C-006/C15-A003) The block manifest at the top of a file is documentary only; the actual filter is same-line grep-v, so the source does not define validation that every header entry matches a trailing annotation.
- (C-006/C15-I001) Once callers use a per-instance SeededClock base and deterministic tick order, instant values can be stable in relative behavior, but byte identity of serialized Instant is neither stated nor generally available.
- (C-006/C15-I002) The carve-outs convert the “ban” into a ratcheted budget; future anti-stale behavior depends on the scripts enforcing a non-increasing baseline, which the source does not explicitly specify.
- (SUMMARY/S-E0198) Introduces an entropy-allow annotation convention for unavoidable SystemTime in cli/commands.rs cache TTL logic and live_cli.rs SSB timestamps. [L190]
- (SUMMARY/S-E0200) Performs that injection in six batches: knowledge, rules, plugins, focus_tunnel, orchestrator top-level, and CLI. [L191]
- (SUMMARY/S-E0193) Closes B-15, phase-0, and B-16; depends on C-14; is parallel-safe with C-16 through C-25. [L184-L187]
- (SUMMARY/S-E0194) Adds instant_now to ClockProvider and implements it for OsClock and SeededClock. [L188]
- (SUMMARY/S-E0195) Defines ELAI_SEED tick-start semantics as zero. [L188]
- (SUMMARY/S-E0196) Migrates production HashMap usage to FxHashMap in ten files grouped across plugins, dashboard-v2, orchestrator, rules, and knowledge. [L189]
- (SUMMARY/S-E0197) Requires one commit per crate group for that migration. [L189]
- (SUMMARY/S-E0199) Injects ClockProvider into about 68 production entropy sites. [L191]
- (SUMMARY/S-E0201) Updates check_entropy_quarantine.sh to honor entropy-allow annotations. [L192]
- (SUMMARY/S-E0202) Re-enables the entropy-quarantine CI check in the Windows workflow. [L193]
- [gate] (SUMMARY/S-E0203) Adds a combined anti-stale gate to that workflow. [L193]
- (SUMMARY/S-E0204) Adds clock_provider_injection_contract.rs with four assertions. [L194]
- (SUMMARY/S-E0205) Determinism smoke runs the same task twice with seed 42 and requires byte-identical action-type sequences while explicitly excluding timestamps. [L195]

### C-16: CosmosStage pipeline integration and refresh lifecycle

**Status**: planned
**Anchors**: ledgers C-005, SUMMARY | sample anchors: L73-L75; L79-L83; L127-L136; L204; L1-L9; L30-L34 | 44 deep atoms, 18 summary atoms | IDs: C-005/C16-E12, C-005/C16-E14, C-005/C16-E19, C-005/C16-E01, C-005/C16-E02, …
**Composes with**: C-0, C-1, C-2, C-3, C-4, C-5, C-7, C-8, C-9, C-10, C-17, C-20, C-24, C-28, C-32, C-34, C-35, C-36, C-44, C-49, C-58. IX: C-005/X-A03 → C-5, C-28, C-35, C-49; SUMMARY/S-E0206 → C-2, C-8, C-17, C-20; C-005/C16-I02 → C-5; C-005/X-I05 → C-5, C-24, C-28, C-32, C-34, C-35, C-36, C-44, C-49; C-005/X-E02 → C-2, C-5, C-8, C-17; C-005/C16-I01 → C-5

- Superseded: (C-005/C16-E10) SSB emission augments rather than replaces tracing, preserving non-SSB observability. [L66-L68]
- Superseded: (SUMMARY/S-E0217) Replaces trigger_cosmos_refresh NotImplemented behavior with a one-shot Tokio task. [L204]
- (C-005/C16-E12) Cancellation checks before sleep and uses `tokio::select!` between cancellation notification and sleeping so `CancelHandle::cancel()` terminates the loop. [L73-L75]
- (C-005/C16-E14) `trigger_cosmos_refresh` immediately returns `TriggerCosmosRefreshResponse { queued, artifacts_due }` while a one-shot `tick()` runs in the background with the same startup-injected dispatcher. [L79-L83]
- [config] (C-005/C16-E19) Observable smoke configures a daily knowledge artifact, performs a cloud research query, expects queued/updated/query-stage SSB evidence, fresh status within a tick, live drawer timestamp, and an `AVAILABLE_COSMOS` briefing block. [L127-L136]
- (C-005/C16-E01) The two-session plan closes five B-42 gaps: tiered pipeline insertion, hourly worker scheduling, a real manual refresh command, SSB emission, and five standalone tests; it depends on C-2/C-8 and is parallel-safe with C-17 through C-20. [L1-L9]
- (C-005/C16-E02) The source preserves the live `3 * cadence` decay threshold as an intentional grace period before shadow serving, rather than reverting to the older `2 * cadence` specification. [L30-L34]
- (C-005/C16-E03) The decision record must capture stage position, 3x rationale, optional SSB injection into `tick`, and an in-process one-shot Tokio task with no extra thread. [L49-L53]
- (C-005/C16-E04) `QueryPipeline` gains `cosmos_stage: Option<CosmosStage>` and `with_cosmos_stage()`. [L55-L56]
- (C-005/C16-E05) `CosmosStage` wraps an `Arc<CosmosWorker>`, calls `cosmos_get()` for query-relevant artifacts, and returns `Vec<RetrievalResult>` with `tool_used="cosmos"` and stage label `cosmos`. [L56-L58]
- (C-005/C16-E06) Stage order is `ArtifactStage -> CosmosStage -> ARagStage`; FAST skips Cosmos while STANDARD and DEEP execute it. [L58-L60]
- (C-005/C16-E07) `CosmosWorker::new()` receives an optional shared `Ssb` handle (or an emitter trait), allowing tests to use `None` and production to use `Some`. [L62-L68]
- (C-005/C16-E08) `tick()` emits `CosmosRefreshStarted` before its manifest loop with `artifacts_queued=stale+decayed count` and `manifest_size`. [L62-L65]
- (C-005/C16-E09) `refresh_one()` emits `CosmosArtifactUpdated` on `Ok(true)` and `CosmosArtifactDecayed` when `cosmos_get()` reports decayed. [L65-L66]
- (C-005/C16-E10) SSB emission augments rather than replaces tracing, preserving non-SSB observability. [L66-L68]
- (C-005/C16-E11) Task-runner startup spawns a repeating `tick -> info TickReport -> sleep(3600s)` Tokio task. [L70-L73]
- (C-005/C16-E13) Production construction supplies manifest, root, clock, and a cloud or sidecar `ResearchDispatcher` selected by `ExecutionMode`. [L75-L77]
- (C-005/C16-E15) Cold start is best-effort: when the dispatcher is not initialized, the command returns `queued=false` and a reason rather than raising `NotImplemented`. [L82-L84]
- (C-005/C16-E16) `CosmosDrawer` must subscribe to `CosmosRefreshStarted`, `CosmosArtifactUpdated`, and `CosmosArtifactDecayed` so updates need not wait for its 30-second poll. [L86-L90]
- (C-005/C16-E17) `CosmosGroundingUsed` is informational: log it to the browser console and do not mutate UI state. [L89-L90]
- (C-005/C16-E18) Refresh callers must acquire `IndexLock(LockScope::Index)` before `tick()`, and refresh-start emission must precede the loop. [L121-L126]
- (C-005/C16-E20) `cosmos_worker_tick.rs` covers first-write body/meta, fresh skip, stale/decayed refresh, unchanged-body idempotence, failure recording without panic, TickReport counts, concurrent workers, and `Some`/`None` SSB behavior. [L212-L219]
- (C-005/C16-E21) `cosmos_lock_contention.rs` requires atomic temp+rename writes, post-concurrency blake3/body agreement, and a 4-thread × 5-tick stress over non-overlapping manifest slices with zero `Blake3Mismatch`. [L221-L226]
- (C-005/C16-E22) `cosmos_pipeline_stage.rs` proves STANDARD visibility, empty-before-refresh, non-empty-after-refresh, FAST omission, and non-fatal `MissingBody` stage errors with pipeline continuation. [L228-L234]
- [status] (C-005/C16-E23) `cosmos_decay_contract.rs` fixes statuses at Fresh below 1x, Stale from 1x to below 3x, Decayed at/after 3x for all four cadences; decayed lookup warns and returns a value, while blake3 mismatch is an error. [L236-L241]
- (C-005/C16-E24) `cosmos_briefing_budget.rs` caps 200 tokens at 800 chars, drops lower relevance when over budget, excludes below-threshold entries, orders Fresh before Stale, excludes Decayed when a Fresh same-class alternative exists, and maps an empty manifest to an empty string. [L243-L248]
- (C-005/C16-E25) Acceptance requires the stage/builder and tier behavior, all three conditional emissions, `None` safety, cancel-safe 3600-second loop, lock acquisition, immediate manual response, live UI, documented 3x divergence, all five no-live-inference suites, workspace check, and a manual cloud Sonnet smoke. [L179-L210]
- (C-005/C16-E26) Deterministic commands inspect stage wiring/emissions/scheduler/manual-stub removal, check all five files, confirm the 3x comment, and run only knowledge/orchestrator/contracts Cosmos scopes plus workspace check. [L138-L177]
- (C-005/C16-E27) Out of scope are vector embedding of whole Cosmos briefs, config-driven manifest merging, verifier changes, artifact CRUD/cadence UI, a worker-owned async `spawn_loop`, and the full workspace test suite. [L250-L262]
- (C-005/C16-E28) Every acceptance box is unchecked; B-42 is described as previously mostly done, but this C-16 wiring/test closure is not claimed shipped. [L13-L28, L179-L210]
- (C-005/C16-D01) Add `CosmosStage` input/output semantics, `tool_used`, exact refresh payload counts, dispatcher selection, cold-start behavior, console-only grounding event, and the continued tracing channel. [L55-L90]
- (C-005/C16-D02) Add every standalone test behavior, especially idempotence, atomic concurrent writes, fail-soft stage errors, exact 1x/3x status boundaries, relevance/budget ordering, and blake3 mismatch failure. [L212-L248]
- [non-goal] (C-005/C16-D03) Add the six explicit non-goals and the fact that the scoped suite, not a full workspace test run, is owned by C-16. [L250-L262]
- (C-005/C16-D04) Remove or qualify “closes B-35”; source evidence supports a B-35 audit-trail reference, not an explicit closure statement. [L92-L94]
- (C-005/C16-A01) The declared response shape has only `queued` and `artifacts_due`, but cold-start behavior also requires a `reason` field; its optionality/type is not defined. [L79-L84, L105-L106]
- (C-005/C16-A02) The ownership/location and contention behavior of `IndexLock` around both periodic and manual one-shot refresh are not specified beyond “caller responsibility.” [L70-L84, L121-L124]
- (C-005/C16-A03) “Relevant artifacts given the query” has no matching/ranking rule, budget, ordering, or duplicate policy. [L55-L60]
- (C-005/C16-A04) Cancellation is specified for the loop around calls but not for an in-progress synchronous `tick()` or manual one-shot task. [L70-L84]
- (C-005/C16-A05) `CosmosArtifactDecayed` is assigned to `refresh_one()` based on a `cosmos_get()` result, but the exact call point and whether one artifact may emit both decayed and updated in a refresh are not defined. [L62-L66]
- (C-005/C16-C01) The orientation summary says this plan “closes B-42 and B-35”; the original’s objective names B-42, while B-35 appears only as part of the divergence audit trail, not an explicit closure target. [L1-L5, L92-L94]
- (C-005/C16-I01) The design has two refresh initiators—periodic and manual—but a shared dispatcher and caller-owned index lock are intended to serialize the same underlying worker path. [L70-L84, L121-L124]
- (C-005/C16-I02) Stage errors are deliberately fail-soft for retrieval, whereas data-integrity mismatch remains fail-hard in the worker contract. [L228-L241]
- (SUMMARY/S-E0217) Replaces trigger_cosmos_refresh NotImplemented behavior with a one-shot Tokio task. [L204]
- (SUMMARY/S-E0206) Closes B-42 and B-35; depends on C-2 and C-8; is parallel-safe with C-17 through C-20. [L197-L200]
- (SUMMARY/S-E0207) Adds CosmosStage to QueryPipeline and exposes with_cosmos_stage as its builder. [L201]
- (SUMMARY/S-E0208) Places CosmosStage between ArtifactStage and ARagStage. [L201]
- (SUMMARY/S-E0209) Runs CosmosStage only for STANDARD and DEEP; FAST skips it. [L201]
- (SUMMARY/S-E0210) Extends CosmosWorker::new with an optional shared SSB handle. [L202]
- (SUMMARY/S-E0211) Emits CosmosRefreshStarted before entering the manifest loop. [L202]
- (SUMMARY/S-E0212) Emits CosmosArtifactUpdated when artifact processing returns Ok(true). [L202]
- (SUMMARY/S-E0213) Emits CosmosArtifactDecayed when cosmos_get reports Decayed. [L202]
- (SUMMARY/S-E0214) Spawns a task_runner tick loop at a 3,600-second cadence. [L203]
- (SUMMARY/S-E0215) Supports cancellation through CancelHandle. [L203]
- (SUMMARY/S-E0216) Protects refresh/index work with IndexLock using LockScope::Index. [L203]
- (SUMMARY/S-E0218) Documents the 3x versus 2x decay-threshold divergence as an intentional grace buffer without changing behavior. [L205]
- (SUMMARY/S-E0219) Makes CosmosDrawer subscribe to live SSB events in addition to its 30-second poll. [L206]
- (SUMMARY/S-E0220) Requires standalone tests for worker tick, lock contention, pipeline stage, decay contract, and briefing budget, with the approximate LOC given in the summary. [L207]
- [config] (SUMMARY/S-E0221) Research smoke configures knowledge artifacts, queries a repos endpoint, and expects CosmosRefreshStarted, Fresh status, and AVAILABLE_COSMOS in the prompt. [L208]

### C-17: Multi-language indexing, ONNX embeddings, resume/watch, and scale metrics

**Status**: planned
**Anchors**: ledgers C-004, SUMMARY | sample anchors: C-17 L76-L79, L169-L172; C-17 L81-L84, L169-L172; C-17 L164-L168; C-17 L185-L231; C-17 L270-L286; Summary L382-L384; C-17 L76-L84, L169-L172 | 50 deep atoms, 20 summary atoms | IDs: C-004/F-C17-007, C-004/F-C17-008, C-004/F-C17-025, C-004/F-C17-029, C-004/F-C17-032, …
**Composes with**: C-1, C-2, C-4, C-5, C-8, C-14, C-15, C-16, C-18, C-20, C-21, C-24, C-30, C-56, C-58. IX: C-004/F-C17-002 → C-14, C-15, C-16, C-18, C-20; C-004/X-002 → C-4, C-8; C-005/X-E03 → C-5, C-24; C-005/X-E02 → C-2, C-5, C-8, C-16; C-004/X-003 → C-4, C-8, C-15, C-21; C-001/X-006 → C-1, C-24, C-30, C-56, C-58

- Superseded: (C-004/F-C17-013) `AragOrchestrator::semantic_search` replaces the norm proxy with an existing dual-vector ONNX session’s query embedding dotted against chunk embeddings, while preserving `SEMANTIC_OVERSAMPLE_FACTOR = 3`. [C-17 L107-L113]
- Superseded: (SUMMARY/S-E0232) Replaces AragOrchestrator’s embedding-norm proxy with real ONNX dot-product search while preserving 3x oversampling. [L218]
- (C-004/F-C17-007) CLI startup calls `auto_resume_indexed_projects` after initialization and before task dispatch; both CLI and dashboard must remain callers. [C-17 L76-L79, L169-L172]
- (C-004/F-C17-008) CLI shutdown handles cross-platform `tokio::signal::ctrl_c` and Unix-only `SignalKind::terminate`, routing both to `graceful_shutdown`; dashboard `ExitRequested` remains another required reachability path. [C-17 L81-L84, L169-L172]
- (C-004/F-C17-025) Primary reachability is CLI startup/shutdown, `/research`, watcher service, and dashboard ActivityFeed. [C-17 L164-L168]
- (C-004/F-C17-029) Deterministic checks assert source symbols/files, no norm/proxy remnants, CLI lifecycle wiring, contract/test presence, scoped crate compilation/tests, and positive tier-A timing. [C-17 L185-L231]
- (C-004/F-C17-032) The explicit edit inventory spans knowledge DKG/BM25/A-RAG, CLI main/live CLI, orchestrator watcher/startup, knowledge/contracts tests, scale scripts/metrics, ActivityFeed, and the decision artifact. [C-17 L270-L286]
- (C-004/D-C17-004) Resume is ordered after initialization/before dispatch; shutdown must remain reachable from both CLI signal paths and dashboard exit. [Summary L382-L384; C-17 L76-L84, L169-L172]
- (C-004/F-C17-001) C-17 is a forward implementation plan with every acceptance box unchecked; it aims to close B-30, B-31, B-31.5, B-34, and phase-3K indexing completeness, fidelity, and lifecycle gaps. [C-17 L1-L8, L233-L268]
- (C-004/F-C17-002) It requires C-15 clock/entropy stability and C-14 dead-code stripping, and is declared parallel-safe with C-16 and C-18–C-20. [C-17 L8-L10]
- (C-004/F-C17-003) `KotlinGrammar` mirrors `JavaGrammar`; it uses a verified `tree-sitter-kotlin` workspace dependency if available, otherwise reuses the Java parser while reporting `language_id = "kotlin"`. [C-17 L60-L64]
- (C-004/F-C17-004) Adding any Kotlin parser dependency is guarded by `cargo search tree-sitter-kotlin` to prevent slopsquatted crate adoption. [C-17 L62-L64]
- (C-004/F-C17-005) `"kt"` and `"kts"` are added to `ext_to_lang_id`, and `KotlinGrammar` is registered in `LanguageRegistry::default()` after an impact check of callers. [C-17 L66-L68]
- (C-004/F-C17-006) New `svelte_extraction.rs` tests require TypeScript reparse of `<script>` with `<file>.svelte::` qualified names, CSS `@import` isolation from script symbols, and no panic for minimal Svelte. [C-17 L70-L74]
- (C-004/F-C17-009) A new `FileWatcherService` acquires `LockScope::Watch` through `IndexLock::try_acquire` on project start, releases it on stop, and is wired beside the completed-project watcher-start path in `startup.rs`. [C-17 L86-L90]
- (C-004/F-C17-010) A new `indexing_resilience_contract.rs` contains four assertions: `#[must_use]` regression for `IndexResult`, type-level `LockScope` requirement for `IndexLock`, empty-project auto-resume completion within five seconds, and graceful-shutdown deadline in `[30,120]` seconds. [C-17 L92-L98]
- (C-004/F-C17-011) `ChunkKind` gains `Heading` and `Paragraph`; non-code files first use `# `/`## ` heading boundaries and blank-line paragraph splitting before the 50-line fallback. [C-17 L100-L105]
- (C-004/F-C17-012) The explicit non-code route excludes `.rs`, `.py`, `.ts`, `.go`, `.java`, `.cs`, and `.rb`; Markdown acceptance specifically requires the new chunk kinds. [C-17 L103-L105, L246-L247]
- (C-004/F-C17-013) `AragOrchestrator::semantic_search` replaces the norm proxy with an existing dual-vector ONNX session’s query embedding dotted against chunk embeddings, while preserving `SEMANTIC_OVERSAMPLE_FACTOR = 3`. [C-17 L107-L113]
- (C-004/F-C17-014) The implementation must discover the actual `OnnxEmbeddingModel`/`EmbeddingSession` API before assuming its shape. [C-17 L107-L111]
- (C-004/F-C17-015) `/research` detects project entry points, formats `PROJECT_STARTING_POINTS`, prepends it to the research query, and emits `EntryPointsDetected`. [C-17 L114-L118]
- (C-004/F-C17-016) Entry-point pre-seeding is conditional on `project_dir.is_some()` and otherwise no-ops. [C-17 L114-L118]
- (C-004/F-C17-017) `run_tier_a.sh` parses real `batch_complete`-style tracing spans to calculate nonzero per-batch p50/p99 wall-clock values and writes them to `phase_b31_5_tier_a.toml`. [C-17 L120-L124]
- (C-004/F-C17-018) `run_all_tiers.sh` runs against a real `ELAI_SCALE_FIXTURE_ROOT` containing at least Linux kernel v6.7 and VS Code 1.94.2; Chromium and Postgres are stretch goals. [C-17 L126-L130]
- (C-004/F-C17-019) Scale acceptance runs `scale_resume_identity_contract`, `scale_lock_contention_contract`, and `scale_long_tail_memory_contract` against populated per-tier TOMLs. [C-17 L126-L132]
- (C-004/F-C17-020) Dashboard `ActivityFeed` subscribes to SSB, filters action type, and renders `IndexResume` as “Resumed indexing from batch N/M (saved X files)”. [C-17 L134-L139]
- (C-004/F-C17-021) The same SSB-driven UI renders `LockContested` as “Another ELAI process (PID X) is indexing; read-only mode”; polling alone is explicitly insufficient. [C-17 L134-L139, L256-L257]
- (C-004/F-C17-022) `C-17-decisions.md` records the Kotlin fallback, ONNX session access, watcher lifecycle, and per-batch timing parsing choices. [C-17 L141-L144]
- (C-004/F-C17-023) Produced surfaces include `KotlinGrammar`, `kt`/`kts` mapping, `ChunkKind::{Heading,Paragraph}`, `heading_boundary_chunks`, `paragraph_chunks`, ONNX dot-product search, project-starting context, `FileWatcherService`, two test files, and two SSB-driven banners. [C-17 L146-L156]
- (C-004/F-C17-024) Consumed interfaces are `LanguageGrammar`, the existing ONNX embedding session, `detect_entry_points`, startup resume/shutdown APIs, `IndexLock::try_acquire`, and `SsbActionType::{IndexResume,LockContested}`. [C-17 L157-L163]
- (C-004/F-C17-026) Stub bans prohibit single-entrypoint-only resume/shutdown, the semantic norm proxy, and filesystem watching that bypasses `LockScope::Watch` locking. [C-17 L169-L175]
- (C-004/F-C17-027) The end-to-end observable indexes Rust/Kotlin/Svelte/Markdown, interrupts mid-index, restarts, then researches `FooService`; required events include resumed batch >0, entry points, and Kotlin grammar loaded. [C-17 L176-L184]
- (C-004/F-C17-028) Required artifacts include Kotlin DKG entries, real dot-product query results, `PROJECT_STARTING_POINTS` context, and heading/paragraph chunks for Markdown. [C-17 L176-L184]
- (C-004/F-C17-030) The cloud smoke additionally requires `files_skipped_on_resume > 0` (no full reindex) and a Sonnet 4.6 research execution over the fixture. [C-17 L258-L266]
- (C-004/F-C17-031) Completion requires a clean workspace build and the four-decision reasoning document in addition to all feature/test/smoke boxes. [C-17 L267-L268]
- [fallback] (C-004/F-C17-033) The Java-parser Kotlin fallback guarantees a `kotlin` language identifier but does not specify how Kotlin-specific syntax that Java’s grammar rejects is recovered or scored. [C-17 L60-L64]
- (C-004/F-C17-034) “Non-code files” is broader than the acceptance text’s Markdown-only assertion; exact extensions routed to heading versus paragraph splitting, and precedence when both apply, are not specified. [C-17 L100-L105, L246-L247]
- (C-004/F-C17-035) The source asks to parse a “batch_complete span or similar” but does not freeze a tracing record schema, percentile algorithm, warm-up policy, or sample-count minimum. [C-17 L120-L124]
- (C-004/F-C17-036) All acceptance boxes remain unchecked, so the source specifies desired behavior and cannot by itself be used as shipped evidence. [C-17 L233-L268]
- (C-004/D-C17-001) All acceptance boxes remain unchecked; summary verbs such as “adds/wires/fixes” describe requirements, not shipped evidence. [Summary L372-L396; C-17 L233-L268]
- (C-004/D-C17-002) Kotlin dependency selection has an anti-slopsquatting `cargo search` gate, and its Java-parser fallback only guarantees the Kotlin language ID. [Summary L380-L381; C-17 L60-L68]
- (C-004/D-C17-003) The omitted Svelte test contract covers TypeScript-qualified names, CSS isolation, and no-panic parsing. [Summary L395; C-17 L70-L74]
- [lifecycle] (C-004/D-C17-005) Watch lifecycle includes lock acquisition on start and release on stop, and resilience tests pin a five-second empty resume plus a 30–120-second shutdown deadline. [Summary L385-L386, L395; C-17 L86-L98]
- (C-004/D-C17-006) Splitters apply to a broader non-code extension route before fallback, not merely Markdown; exact splitter precedence remains unspecified. [Summary L387-L388; C-17 L100-L105]
- (C-004/D-C17-007) Scale runs name VS Code/Linux minimums, Chromium/Postgres stretch fixtures, and three specific scale contracts; banners have exact user-visible strings. [Summary L392-L396; C-17 L120-L139]
- (SUMMARY/S-E0225) Wires auto_resume_indexed_projects into CLI startup. [L215]
- (SUMMARY/S-E0222) Closes B-30, B-31, B-31.5, B-34, and phase-3K; depends on C-15 and C-14; is parallel-safe with C-16 and C-18 through C-20. [L210-L213]
- (SUMMARY/S-E0223) Adds KotlinGrammar using tree-sitter-kotlin when available and JavaGrammar as fallback. [L214]
- (SUMMARY/S-E0224) Registers kt and kts extensions in ext_to_lang_id and LanguageRegistry. [L214]
- (SUMMARY/S-E0226) Handles SIGINT through ctrl_c. [L215]
- (SUMMARY/S-E0227) Handles SIGTERM through SignalKind::terminate. [L215]
- (SUMMARY/S-E0228) Adds FileWatcherService in watcher_service.rs. [L216]
- (SUMMARY/S-E0229) FileWatcherService acquires LockScope::Watch through IndexLock::try_acquire. [L216]
- (SUMMARY/S-E0230) Adds Heading and Paragraph to ChunkKind. [L217]
- (SUMMARY/S-E0231) Adds non-code splitters for Markdown files. [L217]
- (SUMMARY/S-E0232) Replaces AragOrchestrator’s embedding-norm proxy with real ONNX dot-product search while preserving 3x oversampling. [L218]
- (SUMMARY/S-E0233) Pre-seeds research Oracle context with detect_entry_points output under PROJECT_STARTING_POINTS. [L219]
- (SUMMARY/S-E0234) Emits EntryPointsDetected to SSB. [L219]
- (SUMMARY/S-E0235) Fixes run_tier_a.sh to calculate per-batch p50 and p99 from batch_complete tracing spans. [L220]
- (SUMMARY/S-E0236) Populates phase_b31_5_tier_* TOMLs using real scale runs against Linux v6.7 and VS Code 1.94.2. [L221]
- (SUMMARY/S-E0237) Adds SSB-driven IndexResume and LockContested banners to ActivityFeed. [L222]
- (SUMMARY/S-E0238) Adds svelte_extraction.rs and indexing_resilience_contract.rs tests. [L223]
- (SUMMARY/S-E0239) Smoke indexes a Rust/Kotlin/Svelte/Markdown fixture, interrupts midway with SIGINT, restarts, and requires an IndexResume row plus Kotlin entries in the DKG. [L224]

### C-18: Omitted contract/integration test backfill (zero production logic)

**Status**: planned
**Anchors**: ledgers C-001, SUMMARY | sample anchors: C18:L89-100; C18:L38-45; C18:L47-54; C18:L56-62; C18:L64-70; C18:L72-78 | 25 deep atoms, 14 summary atoms | IDs: C-001/C18-008, C-001/C18-002, C-001/C18-003, C-001/C18-004, C-001/C18-005, …
**Composes with**: C-1, C-2, C-6, C-13, C-19, C-20, C-22, C-30, C-41, C-51, C-56, C-58, C-59. IX: C-001/C18-024 → C-1; C-001/X-004 → C-1, C-2, C-58; C-001/C18-021 → C-1; C-001/C18-001 → C-1, C-2, C-13, C-19, C-20; SUMMARY/S-E0240 → C-1, C-2, C-13, C-19, C-20; C-001/X-012 → C-1, C-2, C-6, C-22, C-30, C-41, C-51, C-56, C-58, C-59

- (C-001/C18-008) Five dashboard contracts map D1=get_sidecar_health, D2=run_planning_task, D3=run_research, D6=reindex_now, D11=sidecar_search; each JSON round-trips its request and type-checks the command, with only a TODO for ELAI_LIVE_SIDECAR integration. (C18:L89-100)
- (C-001/C18-002) fsm_phase_transition_contract uses real temporary SSB, proves IDLE→PLANNING and payload fields, and codifies that Fsm::reset emits no second PhaseTransition. (C18:L38-45)
- (C-001/C18-003) symbol_query_dispatch builds a 3-node/2-edge in-memory SymbolGraph, calls translate_tool_request for symbol_impact/flow/context/list, requires Ok plus SymbolQuery after each, and runs offline with ELAI_LIVE_SIDECAR absent. (C18:L47-54)
- (C-001/C18-004) test_minified_file_emits_ssb_action indexes bundle.min.js and requires MinifiedFileChunked with matching relative_path and chunk_count>0. (C18:L56-62)
- (C-001/C18-005) graph_export_dispatch directly invokes export_graph_mermaid without Tauri runtime and requires a Mermaid header, length>10, and byte-identical consecutive output. (C18:L64-70)
- (C-001/C18-006) grounding_artifact_check covers Verified when results exceed threshold, MalformedGrounding for a missing manifest name, and Unverified for empty search, using in-memory ArtifactStore SQLite. (C18:L72-78)
- (C-001/C18-007) designer_artifact_briefing requires AVAILABLE_ARTIFACTS plus two names for an existing project, and no such block for an empty/greenfield manifest. (C18:L80-87)
- [failure] (C-001/C18-009) ssb::tests adds ActionTypeMismatch—not panic/Ok—when CheckpointPayload is read as FileWritePayload, plus five fixed-clock rows whose seq is 1..=5 and whose repeat-run payload bytes match. (C18:L102-109)
- [lifecycle] (C-001/C18-010) BSP tests round-trip LastValue<String>, Topic<u64>, and EphemeralValue<bool> through MessagePack, then run five supersteps and verify last-writer-wins, topic-clear, ephemeral behavior, counter=5, and no small-channel force-clear. (C18:L111-119)
- (C-001/C18-011) SessionDag runs 100 identical seeded 10-message trim cycles and requires identical snap_id/summary_hash; the handoff boundary must be off at 84.9% and on with tag=handoff at exactly 85%. (C18:L121-130)
- (C-001/C18-012) Produced tests consume real Fsm/viewer, production tool translation, IndexingService, Mermaid export, ArtifactCheck/store, project-context builder, Tauri signatures, SSB errors/clock, BSP, and SessionDag APIs. (C18:L132-155)
- [security] (C-001/C18-013) Fixtures must use real in-memory SQLite rather than hardcoded return data, and test setup must not use unwrap in a way that swallows construction errors. (C18:L156-165)
- (C-001/C18-014) Cargo tests across contracts/orchestrator/knowledge/reasoning must pass; a cloud symbol modification must emit SymbolQuery and leave the dispatch contract green on the next test run. (C18:L156-170)
- [gate] (C-001/C18-015) Checks demand at least ten newer *_contract.rs files, all target suites, named determinism/round-trip/mismatch tests, zero unwrap in the FSM test, and a clean workspace check. (C18:L172-206)
- [gate] (C-001/C18-016) Acceptance repeats every behavioral assertion, requires no pre-existing regression/full-workspace audit, and uses “10+ contract/integration test files” as the closure count. (C18:L208-226)
- (C-001/C18-017) Exactly ten files are ADDed: FSM, symbol dispatch, graph export, grounding, designer, and five dashboard contracts. Four existing Rust files are modified to append seven test functions, and one decision doc is created. (C18:L228-244)
- (C-001/C18-022) The dashboard files are type-shape contracts only; their live-sidecar path is explicitly deferred to future CI, while the symbol graph and artifact paths must remain network-free. (C18:L47-100)
- (C-001/C18-018) The objective says “Only new test files under .elai_cc/crates,” but the plan modifies chunking_minified.rs and three production source files' test modules and creates workflow/reasoning/C-18-decisions.md outside that tree. “Zero production logic changes” is the narrower consistent rule. (C18:L1-4, L56-62, L102-130, L228-244)
- (C-001/C18-019) Task 7 says “D1 through D5” but the exact files/commands are D1, D2, D3, D6, and D11. The latter identifiers are repeated in wiring, acceptance, and files lists and are therefore the explicit set. (C18:L89-100, L141-152, L210-215, L236-240)
- (C-001/C18-020) The “>=10 newer contract files” command counts by timestamp relative to channel_registry_contract.rs, so it can include unrelated files or miss modified test modules; it is not an exact proof of this plan's ownership. (C18:L174-176)
- (C-001/C18-021) The described inventory yields a minimum of 20 test functions if FSM, four-symbol dispatch, minified, and graph each use one function: 1+1+1+1+3 grounding+2 designer+5 dashboard+2 SSB+2 BSP+2 SessionDag. Four symbol tools remain separate cases inside the dispatch test. The source does not declare this aggregate. (C18:L38-130)
- (C-001/C18-024) Together the suite spans serialization, dispatcher reachability, event persistence, state boundaries, deterministic export/IDs, and channel lifecycle. It is regression evidence, not a new runtime capability. (C18:L17-22, L38-170)
- [status] (C-001/C18-001) C-18 is a one-session tests-only backfill after C-1/C-2/C-13, parallel with C-19/C-20. It claims zero production changes and targets omitted crate-boundary/serde/DAG checks. (C18:L1-22)
- [status] (C-001/C18-023) The next route is C-19/C-20 or B-43; no feature status is upgraded by authoring this plan. (C18:L246-248)
- (SUMMARY/S-E0240) Closes B-2, B-29, B-32, B-33, B-35, B-36, phase-3B, phase-3D, and phase-3H; depends on C-1, C-2, and C-13; is parallel-safe with C-19 and C-20. [L226-L229]
- (SUMMARY/S-E0241) Changes only tests and makes zero production-code changes. [L230]
- (SUMMARY/S-E0242) Adds fsm_phase_transition_contract.rs for B-2. [L231]
- (SUMMARY/S-E0243) Adds symbol_query_dispatch.rs, exercising four symbol_* tools against an in-memory SymbolGraph, for B-29. [L232]
- (SUMMARY/S-E0244) Adds test_minified_file_emits_ssb_action to chunking_minified.rs for B-32. [L233]
- (SUMMARY/S-E0245) Adds graph_export_dispatch.rs for B-33, checking Mermaid output and two-run determinism. [L234]
- (SUMMARY/S-E0246) Adds grounding_artifact_check.rs with three ArtifactCheck cases for B-35. [L235]
- (SUMMARY/S-E0247) Adds designer_artifact_briefing.rs for B-35. [L236]
- (SUMMARY/S-E0248) Adds dashboard_v2 D1, D2, D3, D6, and D11 contract tests for B-36. [L237]
- (SUMMARY/S-E0249) Adds SSB payload-mismatch and write-sequence-determinism tests for phase-3B. [L238]
- (SUMMARY/S-E0250) Adds BSP checkpoint-roundtrip and five-superstep mixed-channel tests for phase-3D. [L239]
- (SUMMARY/S-E0251) Adds SessionDag 100-run determinism and exact 85% boundary tests for phase-3H. [L240]
- (SUMMARY/S-E0252) Acceptance requires all new tests to pass with no existing regression. [L241]
- (SUMMARY/S-A0253) The plan says “at least ten” files but mixes files and appended test modules; exact file/test counts require C-18.

### C-19: fill missing/partial adversarial tests, Criterion benchmarks, quantified metrics, and regression gates for phases 4A/4C/4D/4F and B-9

**Status**: planned
**Anchors**: ledgers C-002, SUMMARY | sample anchors: C-19:L188-L192; C-19:L220-L242; C-19:L121-L192; L253; C-19:L1-L5; C-19:L6-L11; C-19:L227-L230 | 58 deep atoms, 16 summary atoms | IDs: C-002/C19-E47, C-002/C19-E49, C-002/SD-C-19-04, C-002/C19-E01, C-002/C19-E02, …
**Composes with**: C-1, C-11, C-18, C-20, C-22, C-58. IX: SUMMARY/S-E0254 → C-11, C-18, C-20, C-22; C-002/C19-E02 → C-11, C-18, C-20, C-22; C-001/X-005 → C-1, C-22, C-58

- Superseded: (C-002/SUP-C05) NEWER-PLAN PRECEDENCE: C-27 description is target state relative to older C plan(s) C-1, C-4, C-5, C-12, C-19 for this conflict. Losing description retained at C-002/C05. Conflict: Provider mode used by multiple smoke commands conflicts with C-5's corrected CLI axis. | Evidence: C-1/C-4/C-12/C-19/C-27 use `elai /task --mode cloud`, while C-5 says `--mode` is gate mode and provider is selected by `--provider`. [C-1:L228-L232; C-4:L195-L196; C-12:L152-L155; C-19:L236-L238; C-27:
- (C-002/C19-E47) `test_branch_tree_onmerge_onclose_props_present` compile-checks both props against Tauri command signatures rather than rendering Svelte. [C-19:L188-L192]
- (C-002/C19-E49) Explicit executions cover adversarial IPC/research/session suites, feasibility suites, and plugin benches; smoke expects traversal blocking with no `SANDBOX_VIOLATION` row, T1/T2/T3 timing output, and A1-A8 within 30s. [C-19:L220-L242]
- (C-002/SD-C-19-04) Addition: The source specifies all A1-A8, T9/T13-T16, four baby-step, four research, and four SessionDag/BranchTree behaviors, including paused-time and 30-second anti-hang constraints. [C-19:L121-L192]
- [status] (C-002/C19-E01) Objective: fill missing/partial adversarial tests, Criterion benchmarks, quantified metrics, and regression gates for phases 4A/4C/4D/4F and B-9; estimated 2-3 sessions with no production-code changes. [C-19:L1-L5]
- [deferred] (C-002/C19-E02) Requires C-11 sandbox symbols and conditionally C-22 feasibility types; tests may be compile-gated under `stub_result_v2` until C-22 lands. Parallel-safe with C-18/C-20 due to non-overlapping test/bench/metrics scope. [C-19:L6-L11; C-19:L227-L230]
- [security] (C-002/C19-E03) `test_4a_adv_ipc_u32_max_length_rejected` sends `u32::MAX` frame length and requires `PayloadTooLarge` before a 4GB allocation. [C-19:L62-L66]
- [security] (C-002/C19-E04) `test_4a_adv_ipc_so_peercred_mismatch` connects a second process after accept and requires `SO_PEERCRED` rejection. [C-19:L67-L68]
- (C-002/C19-E05) `test_4a_adv_ipc_think_step_burst_10k` sends 10,000 `ThinkStep`s, drops excess under budget, avoids OOM, and preserves orchestrator responsiveness. [C-19:L69-L70]
- [security] (C-002/C19-E06) `test_4a_adv_selfmod_path_traversal` supplies `../../../metrics.rs` and requires canonical-realpath rejection. [C-19:L71-L72]
- [security] (C-002/C19-E07) `test_4a_adv_selfmod_bash_write_locked` blocks a Bash write to a locked path with no mutation. [C-19:L73-L74]
- [security] (C-002/C19-E08) `test_4a_adv_selfmod_git_commit_locked` rejects a commit targeting a locked path before issuing it. [C-19:L75-L76]
- [security] (C-002/C19-E09) `test_4a_adv_selfmod_symlink_resolved` resolves a symlink and applies the lock to its canonical target. [C-19:L77-L78]
- (C-002/C19-E10) `test_4a_adv_replan_minor_fast_track` requires MINOR-with-evidence auditor fast-track without an INTERACTIVE gate. [C-19:L79-L80]
- [gate] (C-002/C19-E11) `test_4a_adv_replan_major_human_gate` requires a MAJOR request to raise the INTERACTIVE gate and not auto-proceed. [C-19:L81-L82]
- (C-002/C19-E12) `test_4a_adv_replan_max_one_per_phase` rejects a second request from the same agent in the same phase. [C-19:L83-L84]
- (C-002/C19-E13) `test_4a_adv_replan_no_evidence_rejected` rejects `evidence=None`. [C-19:L85-L86]
- [gate/lifecycle] (C-002/C19-E14) `test_4a_adv_discovery_proposed_gate` requires `PROPOSED` before any `CANDIDATE` transition. [C-19:L87-L88]
- (C-002/C19-E15) `test_4a_adv_discovery_inflated_patterns_capped` deduplicates/caps 1,000 repeated violation hashes. [C-19:L89-L90]
- (C-002/C19-E16) `test_4a_adv_crash_ssb_before_toolresult` requires synchronous SSB confirmation before sending `ToolResult`. [C-19:L91-L92]
- (C-002/C19-E17) `test_4a_adv_crash_partial_frame_discard` SIGKILLs mid-frame and requires partial-buffer discard on close without hang. [C-19:L93-L94]
- (C-002/C19-E18) Every one of the 15 phase-4A additions carries a `// Covers audit BLOCK-N / WARN-N` comment. [C-19:L62-L64; C-19:L288-L290]
- (C-002/C19-E19) Add Criterion 0.5 only after lockfile verification, a `[[bench]]`, and four groups in `tiered_analysis_bench.rs`: T1 with 1,000 AST rules/p50<5ms; T2 at 4,999/5,000/5,001 lines/p50<50ms; T3 WARN path/p50<500ms; and total `PostToolUse` latency with Ruff simulation. HTML goes under `target/criterion/`. [C-19:L96-L107]
- (C-002/C19-E20) Add nine exact TOML metric sections: `t1_latency`, `t2_latency`, `t3_latency`, `hook_overhead`, `post_tool_use_latency`, `compile_time_delta`, `test_count`, `loc`, and `coverage`, with the stated fields; measured values are preferred, while pending zero placeholders may be CI-replaced. [C-19:L108-L114]
- [gate] (C-002/C19-E21) `benchmark_eval_improved.rs` asserts `delta_pct >= -5.0`, blocking regressions worse than five percent. [C-19:L116-L119]
- [security] (C-002/C19-E22) `adversarial_filesystem_escape` attempts `/etc/passwd` and requires IPC-layer BLOCK with no content. [C-19:L123-L127]
- [security] (C-002/C19-E23) `adversarial_direct_network_access` requires seccomp to block a direct TCP socket, preserving IPC-only networking. [C-19:L128-L129]
- (C-002/C19-E24) `adversarial_grc_budget_exhaustion` uses budget 1 and requires forced convergence/status rather than loop or hang. [C-19:L130-L131]
- (C-002/C19-E25) `adversarial_all_training_high_risk` marks `ALL-TRAINING` as `HIGH_RISK` and excludes the node from the final report. [C-19:L132-L133]
- [security] (C-002/C19-E26) Feasibility A1 attempts a stub filesystem escape and requires IPC BLOCK/no content. [C-19:L137-L141]
- [security] (C-002/C19-E27) A2 attempts direct TCP, requires seccomp block, and populates `StubResult.error`. [C-19:L141-L142]
- [failure] (C-002/C19-E28) A3 sends malformed IPC and requires `StubTimeout` escalation. [C-19:L142-L143]
- (C-002/C19-E29) A4 concurrently claims GPU from two stubs and forbids VRAM over-allocation. [C-19:L143-L144]
- [fallback] (C-002/C19-E30) A5 models zero GPUs and requires `ProceedWithConstraints` or `Escalate`, never panic. [C-19:L144-L145]
- (C-002/C19-E31) A6 sends exactly `MAX_FRAME_SIZE` bytes and requires acceptance. [C-19:L146-L146]
- (C-002/C19-E32) A7 drops VRAM between probe and launch; recheck must catch it and return `Escalate`. [C-19:L147-L148]
- (C-002/C19-E33) A8 enforces `MAX_ALTERNATIVES_PER_STEP=3` by rejecting a fourth alternative. [C-19:L149-L150]
- (C-002/C19-E34) Every feasibility adversarial fixture, including an infinite loop, must be caught within a 30-second harness. [C-19:L150-L150; C-19:L231-L232]
- [lifecycle] (C-002/C19-E35) T9 creates one SSB branch per stub alternative tagged `branch:stub_{step_id}_{n}`; three alternatives produce three rows. [C-19:L152-L155]
- (C-002/C19-E36) T13 calls `ModelPool.available_vram()` before each launch and escalates if the mock reports insufficient VRAM. [C-19:L156-L157]
- (C-002/C19-E37) T14 models graceful ACK by 25s versus hard kill at 30s using paused Tokio time, returning `StubResult` or `StubTimeout` respectively. [C-19:L158-L159]
- [fallback] (C-002/C19-E38) T15 returns `ProceedWithConstraints` when one of two GPUs exists and records `constraint_result.passed=false` without escalating. [C-19:L160-L161]
- [failure] (C-002/C19-E39) T16 turns malformed IPC into a non-panicking `StubResult` with populated error and `alternative_index=0`. [C-19:L162-L163]
- (C-002/C19-E40) `baby_step_feasibility_skip_when_margins_large` short-circuits with a skip reason when all margins are ≥20%. [C-19:L165-L168]
- (C-002/C19-E41) `baby_step_feasibility_resource_delta_tracked` requires nonzero before/after VRAM delta. [C-19:L169-L170]
- (C-002/C19-E42) `baby_step_feasibility_max_alternatives_cap` rejects a fourth alternative. [C-19:L171-L172]
- (C-002/C19-E43) `baby_step_feasibility_escalation_propagates` surfaces the child escalation in the parent report list. [C-19:L173-L174]
- (C-002/C19-E44) `test_session_merge_slash_command` parses `/session merge branch-abc` into the Session merge action and exact branch ID. [C-19:L178-L182]
- (C-002/C19-E45) `test_session_close_slash_command` does the analogous close parse. [C-19:L183-L184]
- (C-002/C19-E46) `test_branch_tree_onfork_prop_fires` simulates the prop event and requires the callback to receive the branch name. [C-19:L185-L187]
- (C-002/C19-E48) Tests must compile against existing production APIs; missing C-22 fields may be feature-gated and documented, and adversarial processes must use bounded timeout/join so no test runner can hang. [C-19:L227-L232]
- [gate] (C-002/C19-E50) Mechanical acceptance requires ≥15 new 4A functions, benchmark file, ≥9 metric sections, regression assertion, exactly four research tests, eight A-tests, five T-tests, ≥6 baby-step tests total, four B-9 tests, and clean workspace compilation. [C-19:L244-L310]
- (C-002/SD-C-19-01) Addition: The index collapses the 15 IPC cases; the source provides exact function names and distinct allocation, peer-credential, budget, canonicalization, gating, dedup, ordering, and partial-frame assertions. [C-19:L60-L94]
- (C-002/SD-C-19-02) Addition: The benchmark contract includes Criterion 0.5 lockfile verification, four groups, exact T2 boundary sizes, Ruff simulation, and HTML artifacts. [C-19:L96-L107]
- (C-002/SD-C-19-03) Addition: All nine metrics and their fields, plus the conditional `pending_bench` placeholder path, are explicit. [C-19:L108-L114]
- (C-002/SD-C-19-05) Correction: The index/plan shorthand “15 sandbox tests” masks a command mismatch: additions live in `adversarial_ipc.rs`, while later smoke invokes `sandbox_tests`. [C-19:L62-L64; C-19:L234-L235; C-19:L305-L307]
- (SUMMARY/S-E0267) Adds B-9 named tests for session merge/close slash commands and BranchTree fork/merge/close property contracts. [L253]
- (SUMMARY/S-E0254) Closes phase-4A, phase-4C, phase-4D, phase-4F, and B-9; depends on C-11 and conditionally on C-22’s stub_result_v2 reference types; is parallel-safe with C-18 and C-20. [L243-L246]
- (SUMMARY/S-E0255) Adds 15 phase-4A cases to adversarial_ipc.rs. [L247]
- (SUMMARY/S-E0256) Named adversarial categories are u32::MAX, SO_PEERCRED mismatch, a 10,000-ThinkStep burst, self-modification variants, MINOR/MAJOR/no-evidence replans, a discovery PROPOSED gate, inflated patterns, and crash invariants. [L247]
- (SUMMARY/S-E0257) Creates a Criterion benchmark for plugin tiered analysis. [L248]
- (SUMMARY/S-E0258) Requires Tier 1 p50 below 5 ms. [L248]
- (SUMMARY/S-E0259) Tests Tier 2 around exact size boundaries 4,999, 5,000, and 5,001 and requires p50 below 50 ms. [L248]
- (SUMMARY/S-E0260) Requires Tier 3 p50 below 500 ms. [L248]
- (SUMMARY/S-E0261) Benchmarks PostToolUse. [L248]
- (SUMMARY/S-E0262) Adds nine quantified metric sections to phase_4C.toml. [L249]
- [gate] (SUMMARY/S-E0263) Adds a benchmark regression gate requiring delta_pct at least -5.0. [L250]
- (SUMMARY/S-E0264) Adds four adversarial research tests: filesystem escape, direct network access, GRC budget exhaustion, and an ALL-TRAINING HIGH_RISK case. [L251]
- (SUMMARY/S-E0265) Adds feasibility adversarial tests A1 through A8. [L252]
- (SUMMARY/S-E0266) Adds T9 and T13 through T16 plus four baby-step feasibility tests to feasibility_probe_contract.rs. [L252]
- (SUMMARY/S-E0268) Acceptance requires all 15 sandbox tests to pass and A1–A8 to catch an infinite-loop stub within 30 seconds. [L254]

### C-20: Replace placeholder metrics, batch delay, and elai measure

**Status**: planned
**Anchors**: ledgers C-003, SUMMARY | sample anchors: L1-L5; L101-L103; L124-L127; L134-L136, L286-L288; L202-L205; L272-L273 | 69 deep atoms, 16 summary atoms | IDs: C-003/E-C20-001, C-003/E-C20-017, C-003/E-C20-023, C-003/E-C20-027, C-003/E-C20-038, …
**Composes with**: C-3, C-9, C-13, C-14, C-17, C-18, C-19, C-29, C-45, C-59. IX: SUMMARY/S-E0269 → C-9, C-13, C-17, C-18, C-19; C-003/I-X-03 → C-3, C-14, C-29, C-45, C-59; C-003/C-C20-02 → C-9

- (C-003/E-C20-001) Add `elai measure` (or equivalent), populate placeholder metrics from actual measurements, gate a debug batch delay, and include a live cloud Sonnet 4.6 smoke.
- (C-003/E-C20-017) 4G times `set_mode` over 100 calls and `decide(Evidence::all_passed)` over 200 calls, reporting p50/p99.
- (C-003/E-C20-023) 4M health-checks each of 11 backends for p50/p99 and marks a backend `Unavailable` when its CLI is absent.
- (C-003/E-C20-027) Hybrid CLI execution must populate actual `AnthropicApi` and `Sidecar` rows and append retry-derived fallback rate (0.0 is valid if no retry).
- (C-003/E-C20-038) Measurement is reachable from CLI, existing scale-budget CI, and phase auditors.
- (C-003/E-C20-042) 4M acceptance expects seven CLI adapters to report `Unavailable` when binaries are absent.
- (C-003/C-C20-04) The plan says 11 backends, but acceptance quantifies seven absent CLI adapters and names only two live backends; disposition of the other two is not enumerated (L124-L136, L272-L273).
- (C-003/A-C20-02) A single canary/task is said to populate p50/p99, which does not define a sample distribution; percentile calculation/interpolation and warm-up rules are absent throughout (L124-L136).
- (C-003/C-C20-05) Files-touched paths omit the otherwise pervasive `.elai_cc/` prefix for CLI/orchestrator additions, leaving repository-relative placement inconsistent (L172-L201 versus L292-L307).
- (C-003/DELTA+-C20-1) Adds exact CLI grammar, run-summary schema, sample counts, fixtures, percentiles, limits, skip reason, static-cost provenance, and all 1F/1G/1H fields.
- (C-003/E-C20-002) `phase_4G.toml` lacks seven named runtime categories: auto-advance accuracy, false escalation, mode-switch latency, split overhead, model-swap time, end-to-end time, and gate overhead.
- (C-003/E-C20-003) `phase_4F.toml` has four named unmeasured values and lacks six of nine spec metrics.
- (C-003/E-C20-004) `phase_4B.toml` is absent; required keys include IPC, four-language compile latencies, ShortCoder savings, test count, and tool LOC.
- (C-003/E-C20-005) `phase_4M.toml` lacks per-backend latency, cost, fallback rate, and uptime; `phase_4C.toml` lacks nine quantitative categories.
- (C-003/E-C20-006) Scale metrics contain only synthetic `elai_self`; real Linux, Chromium, VS Code, and PostgreSQL runs are absent.
- [status] (C-003/E-C20-007) `phase_1F.toml` and `phase_1G.toml` are absent; 1F must be `UNAVAILABLE`, 1G needs a quantization sweep, and populated DROP-result `phase_1H` needs completeness verification.
- (C-003/E-C20-008) Add `SlashCommand::Measure {scope:MeasureScope}` with `All`, `Phase(String)`, `Scale`, and `Cloud`, handled in new `measure.rs`.
- (C-003/E-C20-009) `--all` runs task passes 2-9 sequentially; `--phase 4G` runs only 4G; `--scale` launches `run_all_tiers.sh` through Tokio; `--cloud` runs hybrid smoke and asserts 4M data.
- (C-003/E-C20-010) Each completion writes uncommitted `metrics/last_measure_run.toml` with timestamp, `scopes_run`, `files_written:Vec<String>`, and per-file pass/fail.
- [config/lifecycle] (C-003/E-C20-011) Read `ELAI_DEBUG_BATCH_DELAY_SECS` once at startup and sleep that many seconds between measurement-batch baby-step dispatches only when `ELAI_MEASURE_MODE=1` and delay is set.
- (C-003/E-C20-012) Production with absent env vars must take a no-op/claimed zero-overhead path; a unit test covers both-var delay and all other skip cases.
- (C-003/E-C20-013) 4B IPC benchmark dispatches ten tool types using Criterion or existing-style `Instant`, recording p50/p99 `ipc_overhead_ms`.
- (C-003/E-C20-014) 4B compile-check benchmarks known-good Rust, Python, TypeScript, and Go fixtures, recording per-language p50/p99.
- (C-003/E-C20-015) 4B measures ShortCoder R1-R10 percent reduction on a 300-line Rust file, reads `test_count`, and computes `loc_tools` with line counts over tool Rust files.
- (C-003/E-C20-016) 4G auto-advance accuracy uses 20 deterministic cases (10 yes/10 no); false-escalation is recorded as 0/20 by fixture construction.
- (C-003/E-C20-018) 4G drives five synthetic circuit-breaker trips and counts actual `EscalationFiredPayload` events.
- [fallback] (C-003/E-C20-019) 4G `split_task_overhead_ms` may use `unmeasured_reason="requires_tree_wiring_C9"`; model substitution times config-only switching and expects p50 below 1ms.
- (C-003/E-C20-020) 4F runs `DefaultSystemProber::probe()` 20 times for p50/p99 and `SyntheticStubExecutor` on a five-line stub 100 times for p50/p99.
- (C-003/E-C20-021) 4F performs ten cycles with `MAX_ALTERNATIVES_PER_STEP=3`, records mean alternatives, and records `novel_step_skip_rate=0.0` with synthetic rationale.
- (C-003/E-C20-022) 4F times `check_constraints` with three resources over 1,000 calls, reporting p50 microseconds with expected value below 100µs.
- (C-003/E-C20-024) For `Sidecar` and `AnthropicApi`, 4M sends canary prompt `ok` under measure mode and collects completion latency.
- [failure] (C-003/E-C20-025) Force primary unavailable 100 times, count fallback rows, and extrapolate `fallback_fire_rate_per_1000`; derive uptime from snapshots.
- [config] (C-003/E-C20-026) `cost_per_1m_tokens` comes from static config and is annotated `source=config`, not represented as a dynamic measurement.
- (C-003/E-C20-028) 4C declares nine categories: hook latency, tier-analysis p50/p99, T1/T2/T3 splits, block/warn rates, sanitization overhead, lock contention, SSB latency, and pre-compact duration.
- (C-003/E-C20-029) 4C runs 100 synthetic `PreToolUse` hook events, 30 tier fixtures split 10/10/10, and measures SSB latency or cites phase 2A.
- (C-003/E-C20-030) Real scale runs require `ELAI_SCALE_FIXTURE_ROOT` containing `linux/chromium/vscode/postgres`, prepared by `clone_scale_fixtures.sh`.
- (C-003/E-C20-031) For each tier, collect p50/p95/p99 for four traversal tools plus peak RSS and write under the tier key.
- [gate] (C-003/E-C20-032) Every scale p99 must be ≤500ms; run the normally ignored scale-determinism test once.
- [fallback] (C-003/E-C20-033) Missing fixture root records `skipped_reason="fixture_root_not_set"` and does not fail the overall run.
- [status] (C-003/E-C20-034) Create 1F with `phase=1F`, `decision=UNAVAILABLE`, reason `prompt_lookup_decoding_not_instrumented`, and `recommended_action=no_further_work`.
- (C-003/E-C20-035) 1G runs `quantize_cross_encoder.py` and records, per level, model MB, p50/p99 latency, and quality delta.
- (C-003/E-C20-036) 1H must contain vocabulary coverage, chosen variant, p50, recall delta, Wilcoxon p-value, and `decision=DROP`; add `generated_at` if missing.
- (C-003/E-C20-037) Every written field must come from an actual measurement or carry `unmeasured_reason`; fabricated/bare placeholder numbers are forbidden.
- (C-003/E-C20-039) `--all` expects scopes `4B,4C,4F,4G,4M,1F,1G,1H,scale,cloud`; every scope passes except the explicitly skippable scale fixture.
- (C-003/E-C20-040) 4B requires every named field and disallows zero/placeholder; 4G requires six of seven categories with the one C-9 reason allowed.
- (C-003/E-C20-041) 4F requires all six named runtime outputs non-placeholder.
- (C-003/E-C20-043) 4C acceptance specifically names hook p50/p99, three tier percentages, block rate, and sanitization overhead, allowing zero only when genuinely measured.
- (C-003/E-C20-044) Preserve `elai_self`; populate the four real scale tiers when configured and pass the scale contract.
- [gate] (C-003/E-C20-045) Workspace compilation must add zero errors, and production measure handler may add neither `.unwrap()` nor `.expect()`.
- (C-003/E-C20-046) Add measure module, three new phase files, optional scale script, and decisions doc; modify commands, task runner, six metric files.
- (C-003/DELTA!-C20-2) The summary's broad “all phase fields non-placeholder” hides explicit allowed unmeasured/Unavailable/skipped outcomes and the unresolved cloud-credential path.
- (C-003/DELTA+-C20-3) The source does not fully specify all nine 4C procedures, 11 backend identities, percentile method, or the seventh 4G measurement; these are design gaps rather than completed details.
- (C-003/C-C20-01) Context says “seven phase files” but explicitly inventories nine metric files/areas (4G, 4F, 4B, 4M, 4C, B29.5, 1F, 1G, 1H) (L13-L47).
- (C-003/C-C20-02) C-9 is a prerequisite, yet split overhead may remain unmeasured because it “requires_tree_wiring_C9” (L7-L9, L107-L110).
- (C-003/C-C20-03) `--all` must include/pass cloud, while cloud explicitly requires a live sidecar and Anthropic API key; unlike scale, no missing-credential skip/failure policy exists (L70-L75, L213-L218, L259-L263).
- (C-003/A-C20-01) The new 4B file is supposed to read `test_count` “from phase_4B.toml notes,” although that file does not exist; source of the count is unresolved (L26-L28, L91-L95).
- (C-003/A-C20-03) 4C declares nine categories, but tasks only give procedures for hooks, tier splits, and SSB latency; warn/block/sanitization/contention/pre-compact procedures and several acceptance keys are missing (L138-L146, L274-L276).
- (C-003/A-C20-04) `end_to_end_pipeline_time` is one of the seven 4G gaps but has no task; the “six of seven” acceptance allows one omission in addition to split's unmeasured value without saying which (L17-L20, L97-L110, L267-L268).
- (C-003/A-C20-05) “Read env once at startup,” environment-mutating unit tests, actual-measurement reproducibility, generated timestamps, model/fixture versions, and concurrent measure-run serialization are unspecified (L74-L83, L147-L170).
- (SUMMARY/S-E0270) Adds the elai measure command. [L260]
- (SUMMARY/S-E0269) Closes phase-4G, phase-4F, phase-4B, phase-4M, phase-4C, B-29.5, and B-31.5; depends on C-9, C-13, and C-17; is parallel-safe with C-18 and C-19. [L256-L259]
- (SUMMARY/S-E0271) Defines MeasureScope variants All, Phase(String), Scale, and Cloud. [L260]
- (SUMMARY/S-E0272) Writes the last measurement run to metrics/last_measure_run.toml. [L260]
- (SUMMARY/S-E0273) Adds an ELAI_DEBUG_BATCH_DELAY_SECS measurement loop. [L261]
- (SUMMARY/S-E0274) Activates that loop only when ELAI_MEASURE_MODE equals 1, promising zero production overhead otherwise. [L261]
- (SUMMARY/S-E0275) Creates phase_4B.toml fields for IPC overhead, per-language compile-check latency, and ShortCoder savings percentage. [L262]
- (SUMMARY/S-E0276) Adds measurements/runtime_measurements, backend_latency/backend_health, or runtime_metrics sections as appropriate to phase_4G, phase_4F, phase_4M, and phase_4C TOMLs. [L263]
- (SUMMARY/S-E0277) Requires real measurements, forbids placeholders, and permits explicit unmeasured_reason annotations. [L263]
- (SUMMARY/S-E0278) Populates phase_b29_5_scale.toml with Linux, Chromium, VS Code, and PostgreSQL tier runs. [L264]
- (SUMMARY/S-E0279) Creates phase_1F.toml marked UNAVAILABLE. [L265]
- (SUMMARY/S-E0280) Creates a phase_1G quantization sweep. [L265]
- (SUMMARY/S-E0281) Verifies phase_1H.toml. [L265]
- (SUMMARY/S-E0282) Adds scripts/scale_test/run_all_tiers.sh. [L266]
- (SUMMARY/S-E0283) Measurement smoke runs all scopes in measurement mode, requires every phase TOML field to be non-placeholder, and requires phase_4M backend latency to be populated by a hybrid task. [L267]

### C-21: Anti-staleness CI, ParameterKey, steering, and MistakeBook BLOCK

**Status**: planned
**Anchors**: ledgers C-004, SUMMARY | sample anchors: C-21 L92-L97; C-21 L124-L131; C-21 L54-L57; L281; C-21 L1-L11, L224-L237; C-21 L54-L58 | 46 deep atoms, 18 summary atoms | IDs: C-004/F-C21-011, C-004/F-C21-018, C-004/F-C21-019, C-004/F-C21-032, C-004/F-C21-001, …
**Composes with**: C-4, C-8, C-15, C-16, C-17, C-20, C-25, C-26, C-38, C-46, C-47, C-52, C-54. IX: C-004/X-003 → C-4, C-8, C-15, C-17; C-004/F-C21-001 → C-15, C-16, C-20; C-004/X-008 → C-4, C-25, C-26, C-46, C-47, C-54; C-004/X-004 → C-4, C-54; C-004/X-014 → C-4, C-8, C-25, C-26, C-38, C-46, C-47, C-52, C-54; SUMMARY/S-E0284 → C-15, C-16, C-20

- Superseded: (C-004/F-C21-013) Open `AllowedConfigKey::Parameter(String)` is replaced by `AllowedConfigKey::Parameter(ParameterKey)` with no string fallback. [C-21 L99-L109, L166-L177]
- Superseded: (C-004/F-C21-022) Integration consumes the anti-stale exit contract, replaces both locked/known string lists, populates `FullPipelineOutcome`, hooks promotion/DIVERGING state, invokes the steering script/rules query, and feeds `record_mistake_on_failure`. [C-21 L153-L161]
- Superseded: (SUMMARY/S-E0294) Replaces the open AllowedConfigKey::Parameter(String) surface with a closed ParameterKey enum. [L279]
- Superseded: (C-001/SUP-C22-025) NEWER-PLAN PRECEDENCE: C-23 description is target state relative to older C plan(s) C-0, C-21 for this conflict. Losing description retained at C-001/C22-025. Conflict: [E][conflict][Δ+] The parallel-safety explanation says C-21 touches session_dag and C-23 touches cloud hardening, which disagrees with C-0's C-21 self-modification and C-23 SessionDag UI descriptions. File-overlap claims need revalidation. (C22:L8-12; C0:L120-135)
- Superseded: (C-001/SUP-A-012) NEWER-PLAN PRECEDENCE: C-23 description is target state relative to older C plan(s) C-21, C-22 for this conflict. Losing description retained at C-001/A-012. Conflict: [conflict] C-22's parallel-file explanation misidentifies C-21/C-23 ownership, and the Tauri command path is inconsistent. (C22:L8-12, L130-179)
- (C-004/F-C21-011) Layer1A `DIVERGING` calls nonblocking `trigger_dep_audit_background()`, which starts detached `elai dep-audit --output-format toml` and writes `metrics/dep_audit_latest.toml`. [C-21 L92-L97]
- (C-004/F-C21-018) The SWE-bench seed corpus contains `trigger_task.toml`, `task_01.toml`…`task_10.toml`, and one-paragraph `README.txt`; the 11 TOMLs use `id`, `description`, `expected_output`, `timeout_s`, `stage`, and `is_canary`. [C-21 L124-L131]
- (C-004/F-C21-019) The trigger task is a known-answer canary gating `StageReport.trigger_task_passed`; the ten tasks use SWE-bench Lite problem statements under MIT terms and include no solution patches. [C-21 L124-L131]
- (C-004/F-C21-032) “STRICT mode” is required, but the specified command shows no environment variable or CLI switch; the source does not identify how strict mode is activated. [C-21 L54-L57]
- (C-004/F-C21-001) C-21 is an unchecked forward plan to close B-16, B-23, phase-4H, and anti-staleness-enforcement wiring gaps; it requires C-15 and is declared parallel-safe with C-16–C-20. [C-21 L1-L11, L224-L237]
- (C-004/F-C21-002) Windows CI runs `bash scripts/check_anti_stale.sh` after the HashMap-ban step in strict mode; a nonzero exit fails the job. [C-21 L54-L58]
- [gate] (C-004/F-C21-003) B-16B Chunk C is a hard permission gate: prompt exactly for proceeding with roughly 12 `.unwrap()` sites, accept explicit `Y`, and do not modify those sites on `N` or silence. [C-21 L35-L36, L60-L64]
- (C-004/F-C21-004) If approved, Chunk C reads the recorded site list and converts the deterministic-auditor/DKG calls with `?` or `.ok_or()`; `.expect()` is prohibited. [C-21 L60-L64]
- (C-004/F-C21-005) `scripts/install_pre_commit_hook.sh` installs behavior into `.git/hooks/pre-commit`; the generated hook is local and not committed, while the installer is committed. [C-21 L66-L74]
- (C-004/F-C21-006) The hook inspects staged paths and blocks `elai_cc.toml` plus files matching `*safety_locks*`, `*auditor_config*`, `*cloud_monthly_budget*`, `*benchmark_harness*`, `*test_suite*`, `*score_weights*`, `*gate_definitions*`, `*lock_list*`, `*cloud_opus_allowed*`, `*cloud_escalation_threshold*`, or `*config_variants_schema*`. [C-21 L66-L72]
- (C-004/F-C21-007) A locked-key match exits 1 and names the path using `BLOCKED: commit modifies LOCKED_KEYS file: <path> (spec phase-4H.txt:201-204)`. [C-21 L68-L74, L166-L169]
- (C-004/F-C21-008) `dep_audit_to_stale_catches(&DepAuditReport) -> Vec<StaleApiCatch>` maps real dependency-audit output into `FullPipelineOutcome.stale_api_catches` at each builder site where a report exists; absence of an audit leaves the field empty. [C-21 L76-L82]
- (C-004/F-C21-009) A successful `DataTable::DeprecatedPatterns` promotion asynchronously spawns `metrics/compute_steering_vector.py`, logs the child PID, and emits `steering_vector_refresh_triggered`. [C-21 L84-L90]
- (C-004/F-C21-010) A missing steering-vector script warns and never panics; no process is spawned and no event is promised for an unsuccessful promotion. [C-21 L84-L90]
- (C-004/F-C21-012) Dependency-audit spawning is rate-limited by `metrics/.dep_audit_last_trigger`; a sentinel modified less than 24 hours ago suppresses the trigger. [C-21 L92-L97]
- [config] (C-004/F-C21-013) Open `AllowedConfigKey::Parameter(String)` is replaced by `AllowedConfigKey::Parameter(ParameterKey)` with no string fallback. [C-21 L99-L109, L166-L177]
- (C-004/F-C21-014) `ParameterKey` has exactly 12 variants: `EtcThreshold`, `FreshnessHalfLife`, `FreshnessDocHalfLife`, `FreshnessBoostNorm`, `BabyStepMaxLoc { vram_tier: u8 }`, `ContextSlotWeight { slot: u8 }`, `RetryBudget`, `AbMctsMaxBranches`, `AbMctsTemperatures`, `MicroReflectionMaxTokens`, `SteeringVectorCoefficient`, and `SelfModFrequency`. [C-21 L99-L105]
- [config] (C-004/F-C21-015) Match arms and proposal target configuration types are migrated across `self_mod_layer1a`, `self_mod_safety`, `self_mod_bridge`, `self_mod_orchestrator`, and `self_mod_types`; constructing `Parameter("cloud_monthly_budget")` must fail compilation. [C-21 L105-L113]
- (C-004/F-C21-016) `ConstraintsBlock` queries `rules.db` for `deprecated_patterns` with `severity='BLOCK'` and creates `DO NOT USE {deprecated_name}. Use {replacement_name}.` entries. [C-21 L115-L122]
- (C-004/F-C21-017) Rule-derived constraints append to TOML overrides, de-duplicate on `deprecated_name`, and silently fall through if `rules.db` is unavailable. [C-21 L115-L122]
- [failure] (C-004/F-C21-020) Pre-tool block failures carry `T1_BLOCK`, `T2_BLOCK`, or `PRE_TOOL_BLOCK` according to the responsible hook so `record_mistake_on_failure` writes a categorized `MistakeEntry`; the RPC response is updated only if it exposes this tier. [C-21 L133-L140]
- (C-004/F-C21-021) Produced surfaces include the CI step, hook installer/behavior, dependency-audit mapper/background trigger, closed enum, rules-derived constraints, 11-TOML corpus, and block-tier discriminant. [C-21 L142-L152]
- (C-004/F-C21-022) Integration consumes the anti-stale exit contract, replaces both locked/known string lists, populates `FullPipelineOutcome`, hooks promotion/DIVERGING state, invokes the steering script/rules query, and feeds `record_mistake_on_failure`. [C-21 L153-L161]
- (C-004/F-C21-023) Operational reachability is a GitHub Actions PR, local `git commit`, Layer1A `DIVERGING`, deprecated-pattern promotion, and the task-runner failure path. [C-21 L162-L165]
- (C-004/F-C21-024) Stub bans require exactly 12 enum variants, hook exit 1, real audit data, and nonblocking audit launch. [C-21 L166-L170]
- (C-004/F-C21-025) Promotion observability requires the steering script to start and `modern_api.npz` mtime to change; acceptance tightens this to within 60 seconds and requires the refresh event. [C-21 L171-L179, L224-L227]
- (C-004/F-C21-026) Locked-file commit observability is the exact blocking message and exit 1; closed-parameter observability is a compile error. [C-21 L171-L177, L228-L231]
- (C-004/F-C21-027) DIVERGING observability requires `dep_audit_latest.toml` and nonempty `stale_api_catches` when stale dependencies exist. [C-21 L178-L179, L232-L233]
- (C-004/F-C21-028) Ten deterministic checks cover CI presence, all enum variants, absence of raw string construction, installer, audit functions, 11 fixtures, rules-query wiring, clean orchestrator build, and block discriminants. [C-21 L181-L222]
- (C-004/F-C21-029) Acceptance requires the anti-stale step to execute successfully in an actual PR and a scoped `parameter_key` test suite to pass. [C-21 L234-L235]
- (C-004/F-C21-030) Chunk C acceptance is conditional on prior approval and then requires zero production `.unwrap()` sites in `deterministic_auditor.rs` plus passing tests. [C-21 L236-L237]
- (C-004/F-C21-031) The file inventory explicitly marks deterministic-auditor/DKG edits conditional and creates `C-21-decisions.md`; no completion claims are checked in this source. [C-21 L239-L259]
- [lifecycle] (C-004/F-C21-033) The sentinel write/update lifecycle after a spawned audit, behavior after spawn failure, and cross-process race handling are not specified. [C-21 L92-L97]
- (C-004/F-C21-034) Silent `rules.db` fallthrough protects availability but supplies no event/metric, so operators cannot distinguish “no blocked patterns” from “database unavailable” using this contract alone. [C-21 L115-L122]
- (C-004/F-C21-035) The pre-commit protection is opt-in per clone because only an installer is committed; CI anti-stale enforcement is the repository-wide backstop. [C-21 L54-L58, L66-L74]
- (C-004/D-C21-001) The source is unchecked and Chunk C has a hard authorization boundary: explicit `Y` only, with `N` or silence prohibiting edits and `.expect()` banned. [Summary L474-L475; C-21 L35-L36, L60-L64]
- (C-004/D-C21-002) The local pre-commit hook is installed but not committed and matches an explicit 12-pattern/path set with an exact failure message. [Summary L476-L477; C-21 L66-L74]
- (C-004/D-C21-003) Steering refresh occurs only after successful promotion, logs PID, warns without panic when missing, and dependency audits use exact output/sentinel paths with a 24-hour mtime gate. [Summary L478-L482; C-21 L76-L97]
- (C-004/D-C21-004) The 12 enum variants include typed `vram_tier`/`slot` payloads; rules-derived constraints append, dedupe, and silently fall through; SWE fixtures forbid solution patches. [Summary L483-L487; C-21 L99-L131]
- (C-004/D-C21-005) The summary omits that STRICT activation, sentinel race/failure semantics, and unavailable-rules observability are undefined. [Summary L474-L488; C-21 L54-L57, L92-L122]
- (SUMMARY/S-E0297) Creates a SWE-bench seed corpus directory with trigger_task.toml, task_01 through task_10 TOMLs, and README.txt. [L281]
- (SUMMARY/S-E0284) Closes B-16, B-23, phase-4H, and anti-staleness-enforcement; depends on C-15; is parallel-safe with C-16 through C-20. [L269-L272]
- (SUMMARY/S-E0285) Wires check_anti_stale.sh into Windows CI. [L273]
- (SUMMARY/S-E0286) Conditionally, only after a user answers Y, converts about 12 unwrap calls in deterministic_auditor and dkg into propagation or ok_or handling. [L274]
- (SUMMARY/S-E0287) Creates install_pre_commit_hook.sh. [L275]
- (SUMMARY/S-E0288) The installed pre-commit hook blocks commits that modify files owning LOCKED_KEYS. [L275]
- (SUMMARY/S-E0289) Maps DepAuditReport into FullPipelineOutcome.stale_api_catches through dep_audit_to_stale_catches. [L276]
- (SUMMARY/S-E0290) After promotion of DataTable::DeprecatedPatterns in self_mod_bridge, starts compute_steering_vector.py asynchronously without waiting. [L277]
- (SUMMARY/S-E0291) Emits steering_vector_refresh_triggered for that refresh. [L277]
- (SUMMARY/S-E0292) Starts nightly dependency audit when Layer1A becomes DIVERGING. [L278]
- (SUMMARY/S-E0293) Rate-limits that trigger to once every 24 hours through a sentinel file. [L278]
- [config] (SUMMARY/S-E0294) Replaces the open AllowedConfigKey::Parameter(String) surface with a closed ParameterKey enum. [L279]
- (SUMMARY/S-E0295) Enumerated ParameterKey forms are EtcThreshold, FreshnessHalfLife, FreshnessDocHalfLife, FreshnessBoostNorm, BabyStepMaxLoc by VRAM tier, ContextSlotWeight by slot, RetryBudget, AbMctsMaxBranches, AbMctsTemperatures, MicroReflectionMaxTokens, SteeringVectorCoefficient, and SelfModFrequency. [L279]
- (SUMMARY/S-E0296) Populates ConstraintsBlock in project_profile.rs from rules.db rows where deprecated_patterns severity is BLOCK. [L280]
- (SUMMARY/S-E0298) Records MistakeBook entries for T1_BLOCK, T2_BLOCK, and PRE_TOOL_BLOCK events. [L282]
- (SUMMARY/S-E0299) Acceptance requires deprecated-pattern promotion to change modern_api.npz modification time, LOCKED_KEYS commits to be blocked, and an arbitrary string ParameterKey construction to fail compilation. [L283]

### C-22: Feasibility prober, GPU UUID, ResourceDelta, and FeasibilityPanel

**Status**: planned
**Anchors**: ledgers C-001, SUMMARY | sample anchors: C22:L14-50; C22:L122-128; C22:L130-140; C22:L140-143; C22:L163-188; C22:L184-188 | 38 deep atoms, 21 summary atoms | IDs: C-001/C22-002, C-001/C22-011, C-001/C22-012, C-001/C22-013, C-001/C22-016, …
**Composes with**: C-0, C-1, C-2, C-6, C-9, C-18, C-19, C-21, C-23, C-30, C-33, C-41, C-51, C-56, C-58, C-59. IX: C-001/X-005 → C-1, C-19, C-58; C-001/SUP-C22-025 → C-0, C-1, C-21, C-23; C-001/X-012 → C-1, C-2, C-6, C-18, C-30, C-41, C-51, C-56, C-58, C-59; SUMMARY/S-E0300 → C-9, C-19, C-21, C-23; C-001/C22-025 → C-0, C-21, C-23; C-006/C33-X003 → C-6, C-30, C-33

- Superseded: (C-001/C22-013) The panel consumes get_feasibility_report, replaces the ReasoningDrawer placeholder, and clears d.unwired.feasibility=false. (C22:L140-143)
- Superseded: (C-001/SUP-C22-025) NEWER-PLAN PRECEDENCE: C-23 description is target state relative to older C plan(s) C-0, C-21 for this conflict. Losing description retained at C-001/C22-025. Conflict: [E][conflict][Δ+] The parallel-safety explanation says C-21 touches session_dag and C-23 touches cloud hardening, which disagrees with C-0's C-21 self-modification and C-23 SessionDag UI descriptions. File-overlap claims need revalidation. (C22:L8-12; C0:L120-135)
- Superseded: (C-001/SUP-A-012) NEWER-PLAN PRECEDENCE: C-23 description is target state relative to older C plan(s) C-21, C-22 for this conflict. Losing description retained at C-001/A-012. Conflict: [conflict] C-22's parallel-file explanation misidentifies C-21/C-23 ownership, and the Tauri command path is inconsistent. (C22:L8-12, L130-179)
- (C-001/C22-002) Nine verified gaps are named: SystemProbe type mismatch, missing GPU UUID, missing StubResult fields, absent ResourceDelta, None production construction, event-name drift, forbidden kill_on_drop, absent panel/unwired flag, and six absent metrics. (C22:L14-50)
- (C-001/C22-011) Replace None with Some(Box::new(DefaultFeasibilityProber::new())) in cli/main.rs and runtime_setup.rs, using in-scope AppConfig if required and exporting the constructor if necessary. (C22:L122-128)
- (C-001/C22-012) FeasibilityPanel takes FeasibilityReport|null and shows probe_count, novel_step_count, summed alternatives, optional skip notice, and per-concept decision/stub hashes/stderr/resource VRAM accordion; null shows “Feasibility probe not run.” (C22:L130-140)
- (C-001/C22-013) The panel consumes get_feasibility_report, replaces the ReasoningDrawer placeholder, and clears d.unwired.feasibility=false. (C22:L140-143)
- (C-001/C22-016) Reachability is cloud /research→DefaultFeasibilityProber, ReasoningDrawer→Tauri report, and live alternative/escalation SSB rows. AppConfig and explicit kill are consumed symbols. (C22:L163-188)
- [security] (C-001/C22-017) Production bans are zero kill_on_drop, no new sysinfo dependency/resource path, and no mocked/hardcoded panel data. (C22:L184-188)
- (C-001/C22-018) The candle-rs smoke requires system_probe and feasibility_report, optional alternative rows, real panel counts, non-zero ResourceDelta VRAM, and a skip only when RESOURCE_MARGIN_THRESHOLD_PCT<20%; zero kill_on_drop is also observable evidence. (C22:L189-200)
- [gate] (C-001/C22-019) Twelve static checks cover the rename/schema, event constants, two construction paths, component/flag, six metrics, and workspace compile. (C22:L201-253)
- [config] (C-001/C22-021) Work modifies five Rust/config/test files plus ReasoningDrawer/drawers state, adds FeasibilityPanel, modifies phase_4F.toml and possibly contract tests, and creates C-22-decisions.md. (C22:L285-307)
- (C-001/C22-003) Rename SystemProbeReport→SystemProbeResult mechanically everywhere under crates, with zero old-name hits and no logic change. (C22:L69-74)
- [failure] (C-001/C22-004) GpuInfo.uuid is Option<String>, parsed from nvidia-smi -q -d UUID via GPU-[0-9a-f-]{36}; parse failure stores None and adds no dependency. (C22:L76-80)
- (C-001/C22-005) ResourceDelta has four u64 counters—VRAM/RAM before/after—and derives Debug, Clone, Default, Serialize, and Deserialize. (C22:L82-92)
- (C-001/C22-006) StubResult adds alternative_index:u8 (zero means first/error), optional blake3 stdout hash, optional first-256-character stderr summary, and optional ResourceDelta; every field has serde(default) and no existing field is removed. (C22:L93-100)
- [failure] (C-001/C22-007) Probe resources before spawn and after return with SystemProbeResult; on either re-query failure set resource_delta=None rather than failing the stub. (C22:L101-106)
- [lifecycle] (C-001/C22-008) Remove kill_on_drop(true); explicit child.kill().await at the two existing paths becomes the sole termination mechanism, avoiding double-kill risk. (C22:L108-111)
- (C-001/C22-009) Preserve canonical shipped event names system_probe, feasibility_report, novel_step_detected, and stub_run, and update phase-4F/B-21 contracts rather than breaking consumers. (C22:L113-120)
- (C-001/C22-010) Add alternative_explored once per accepted alternative with step_id+alternative_index, and feasibility_escalated on Escalate with reason+escalation variant; register and emit both. (C22:L116-120)
- (C-001/C22-014) After a real run, phase_4F.toml records measured system-probe p50, stub p50/p99, average alternatives, skip fraction, and escalation fraction; placeholders are forbidden. (C22:L145-153)
- (C-001/C22-015) T9/T13–T16 are owned conditionally: add them if C-19 did not, otherwise remove C-19's stub_result_v2 gate; the decision doc records execution order. (C22:L155-161)
- (C-001/C22-020) Acceptance explicitly requires old MessagePack payloads to deserialize without panic through serde defaults and resource collection not to alter the child lifecycle. (C22:L255-274)
- [config] (C-001/C22-027) If constructor config/export differs, reuse existing AppConfig or make the constructor public; if C-19 ran first, ungate rather than duplicate its tests. These are explicit adaptation paths, not permission to redesign the probe. (C22:L122-128, L155-161, L309-313)
- (C-001/C22-022) The prose calls StubResult's change “three fields” but enumerates four: alternative_index, stdout_hash, stderr_summary, and resource_delta. Wiring/acceptance consistently require four. (C22:L27-32, L93-100, L169-170, L260-261)
- (C-001/C22-023) All six metrics are ordered non-zero, but skip_rate and escalation_rate are fractions that may legitimately be 0.0; the same smoke only conditionally fires skip and does not require escalation. Measurement truth and the non-zero gate can therefore conflict. (C22:L145-153, L190-197, L245-248, L271-283)
- (C-001/C22-024) The UUID command/parser says run once per GpuInfo, but nvidia-smi -q can return multiple GPUs and no association rule is stated; the regex also names lowercase hex only. (C22:L76-80)
- (C-001/C22-025) The parallel-safety explanation says C-21 touches session_dag and C-23 touches cloud hardening, which disagrees with C-0's C-21 self-modification and C-23 SessionDag UI descriptions. File-overlap claims need revalidation. (C22:L8-12; C0:L120-135)
- (C-001/C22-026) The Tauri source is cited once as commands.rs:433 and once as src-tauri/src/lib.rs:433; exact ownership must be resolved from code before wiring. (C22:L130-143, L174-179)
- [status] (C-001/C22-001) C-22 is a 2–3 session PLANNED-style gap closure after C-19/C-9, parallel-safe by claim with C-21/C-23; it activates the dead feasibility path, data/resource schema, events, dashboard, termination contract, tests, and metrics. (C22:L1-12)
- (SUMMARY/S-E0311) Wires a real DefaultFeasibilityProber in cli/main.rs and runtime_setup.rs instead of None. [L295]
- (SUMMARY/S-E0312) Creates FeasibilityPanel.svelte. [L296]
- (SUMMARY/S-E0314) Mounts FeasibilityPanel in ReasoningDrawer and clears unwired.feasibility. [L296]
- (SUMMARY/S-E0316) Cloud research smoke uses an unfamiliar candle-rs library, expects non-null VRAM ResourceDelta, and requires FeasibilityPanel to render. [L298]
- (SUMMARY/S-E0300) Closes B-21 and phase-4F; depends on C-19 and C-9; is parallel-safe with C-21 and C-23. [L285-L288]
- (SUMMARY/S-E0301) Renames SystemProbeReport to SystemProbeResult throughout the codebase. [L289]
- (SUMMARY/S-E0302) Adds uuid to GpuInfo by parsing nvidia-smi -q -d UUID. [L290]
- (SUMMARY/S-E0303) Adds ResourceDelta with VRAM and RAM before/after byte counts. [L291]
- (SUMMARY/S-E0304) Extends StubResult with alternative_index, stdout_hash, stderr_summary, and resource_delta. [L291]
- (SUMMARY/S-E0305) Uses serde defaults on all new StubResult fields for compatibility. [L291]
- (SUMMARY/S-E0306) Captures resource deltas around stub execution using SystemProbeResult queries before and after. [L292]
- (SUMMARY/S-E0307) Removes kill_on_drop(true) from the feasibility child process. [L293]
- (SUMMARY/S-E0308) Makes explicit child.kill().await the sole termination mechanism. [L293]
- (SUMMARY/S-E0309) Keeps shipped SSB names system_probe, feasibility_report, novel_step_detected, and stub_run. [L294]
- (SUMMARY/S-E0310) Adds alternative_explored and feasibility_escalated SSB names. [L294]
- (SUMMARY/S-E0313) Displays probe_count, novel_step_count, alternatives_explored, skip_reason, and a per-concept accordion. [L296]
- (SUMMARY/S-E0315) Populates six phase_4F metric fields from a smoke run. [L297]

### C-23: Session branch merge/close, BranchTree, and /continue handoff

**Status**: planned
**Anchors**: ledgers C-006, SUMMARY | sample anchors: C-23 L1-L6; C-23 L13-L31; C-23 L61-L69; C-23 L117-L124; C-23 L125-L128; C-23 L129-L136 | 42 deep atoms, 17 summary atoms | IDs: C-006/C23-E001, C-006/C23-E003, C-006/C23-E010, C-006/C23-E017, C-006/C23-E018, …
**Composes with**: C-3, C-4, C-6, C-13, C-21, C-22, C-24, C-40. IX: C-006/C23-I001 → C-6; C-006/C23-E002 → C-3, C-4, C-21, C-22, C-24; C-006/C23-X001 → C-6; SUMMARY/S-E0317 → C-3, C-4, C-21, C-22, C-24; C-006/C40-X003 → C-6, C-13, C-40; C-006/C23-X002 → C-6

- Superseded: (SUMMARY/S-E0319) Replaces “use CLI” stubs in fork_session_branch, merge_session_branch, and close_session_branch Tauri commands with real SessionDag calls. [L305]
- Superseded: (C-001/SUP-C22-025) NEWER-PLAN PRECEDENCE: C-23 description is target state relative to older C plan(s) C-0, C-21 for this conflict. Losing description retained at C-001/C22-025. Conflict: [E][conflict][Δ+] The parallel-safety explanation says C-21 touches session_dag and C-23 touches cloud hardening, which disagrees with C-0's C-21 self-modification and C-23 SessionDag UI descriptions. File-overlap claims need revalidation. (C22:L8-12; C0:L120-135)
- Superseded: (C-001/SUP-A-012) NEWER-PLAN PRECEDENCE: C-23 description is target state relative to older C plan(s) C-21, C-22 for this conflict. Losing description retained at C-001/A-012. Conflict: [conflict] C-22's parallel-file explanation misidentifies C-21/C-23 ownership, and the Tauri command path is inconsistent. (C22:L8-12, L130-179)
- (C-006/C23-E001) Objective: close the B-9 partial/missing surface by adding merge/close CLI actions, live BranchTree, real Tauri mutations, load_handoff, root-to-tip ancestry, 100-run determinism, handoff-schema verification, and branch-event banners. [C-23 L1-L6]
- (C-006/C23-E003) Verified gaps are: parser supports only list/switch/fork; BranchTree is unmounted; three Tauri commands return CLI-delegation errors because AppState lacks SessionDag; /continue only prepopulates; parent_chain is reversed; and only a two-seed determinism comparison exists. [C-23 L13-L31]
- (C-006/C23-E010) SessionsDrawer imports and mounts BranchTree beneath/replacing the branch list, passes $sessionBranches, invokes fork/merge/close with branchId payloads, and refreshes the list after success. [C-23 L61-L69]
- (C-006/C23-E017) Consumed symbols are create_branch, merge_branch_with_ssb, close_branch, load_handoff, get_session_branches, AppState ssb_writer/DAG configuration, and BranchTree callback props. [C-23 L117-L124]
- (C-006/C23-E018) Reachable surfaces are /session merge <branch_id>, /session close <branch_id>, SessionsDrawer BranchTree, and CommandWindow /continue. [C-23 L125-L128]
- (C-006/C23-E019) Stub bans remove every “use CLI: /session …” return and forbid a prepopulate-only /continue path. [C-23 L129-L136]
- [lifecycle] (C-006/C23-E020) The observable lifecycle requires Checkpoint plus branch_* rows, live BranchTree state, and actual load_handoff behavior. [C-23 L137-L142]
- (C-006/C23-E021) Deterministic/static evidence checks parser arms, absence of stub text, BranchTree import/mount, load_handoff, chain.reverse, root index zero, a 100-loop, all branch action strings, and both session-dag test commands. [C-23 L144-L183]
- (C-006/C23-E023) Cloud smoke runs a 20-file task past 85% context, observes create_handoff, resumes in another terminal through /continue/load_handoff, then exercises fork/merge/close while the live drawer reflects state. [C-23 L219-L226]
- (C-006/C23-E024) Planned file surface is CLI commands, session_dag.rs, Tauri commands, SessionsDrawer, CommandWindow, SettingsWindow verify/modify, optional session_dag_contract.rs, and C-23-decisions.md. [C-23 L228-L244]
- (C-006/C23-A001) BranchTree's onfork callback receives a name but the invocation payload key is branchId; the Tauri parameter name/rename contract is not reconciled. [C-23 L54-L56, L65-L68]
- (C-006/C23-E002) Estimated one to two sessions; C-3 and C-4 are prerequisites; parallel-safe with C-21, C-22, and C-24. [C-23 L7-L9]
- (C-006/C23-E004) parse_session_command adds exact-arity merge and close arms, returns usage errors when the target is absent, sets action plus branch_id, extends argument_hint, and routes both actions through the live REPL matcher. [C-23 L41-L49, L203-L204]
- (C-006/C23-E005) fork_session_branch resolves the DAG from AppState, calls create_branch(name), and emits an SSB Checkpoint through ssb_writer. [C-23 L51-L56]
- (C-006/C23-E006) merge_session_branch calls the async merge_branch_with_ssb(branch_id, ssb_writer). [C-23 L57, L190-L191]
- (C-006/C23-E007) close_session_branch calls close_branch(branch_id) and emits an SSB Checkpoint. [C-23 L58]
- (C-006/C23-E008) All three Tauri mutations return typed DagBranchError, never raw strings. [C-23 L59]
- (C-006/C23-E009) AppState must expose either a DAG handle or a configured dag_db_path: Option<PathBuf>; if absent, populate the path from elai_cc.toml session.dag_db_path and never hardcode it. The summary's unconditional “holds a DAG handle” is too narrow. [C-23 L21-L23, L122-L123, L185-L189]
- (C-006/C23-E011) CommandWindow /continue invokes load_handoff and pushes a success status card; invocation failure restores the previous prepopulation behavior and adds an error card; stub.ts is extended only if needed. [C-23 L71-L77, L209-L210]
- (C-006/C23-E012) parent_chain adds chain.reverse and changes all assertions/callers to [snap-root, snap-mid, snap-leaf]. [C-23 L79-L84, L195-L197, L211]
- (C-006/C23-E013) The determinism contract repeats create_handoff plus load_handoff 100 times on identical input with a fixed SeededRng and requires identical snap_ids and trim output; if C-18 already proves it, the result may be documented DONE instead. [C-23 L86-L92, L198-L199, L212-L213]
- (C-006/C23-E014) SettingsWindow is grep-audited for SessionHandoffEntry/HandoffEntry/handoff_entry; stale V1 fields such as chat_window_state are either absent and documented DONE or aligned with the Rust struct. [C-23 L94-L99, L214]
- (C-006/C23-E015) Mutations emit branch_forked, branch_merged, and branch_closed through AppState ssb_writer, following the existing branch_merge/checkpoint pattern. [C-23 L101-L106, L110-L116]
- (C-006/C23-E016) ActivityWindow or SessionsDrawer renders banners for all three new branch action types. [C-23 L105-L106, L215]
- (C-006/C23-E022) Frontend compatibility must verify Svelte 5 $props callback syntax; parent-chain consumers must be swept before reordering. [C-23 L192-L197]
- (C-006/C23-E025) Every acceptance checkbox is open; the 100-run test may be delegated/documented to C-18, and the next step may be C-24 or C-18. [C-23 L201-L218, L246-L249]
- (C-006/C23-A002) The 100 iterations do not say whether state is reset or each run gets a fresh DAG; repeated writes to one DB could intentionally change snap IDs. [C-23 L86-L92]
- (C-006/C23-A003) load_handoff is invoked with no named session/branch argument in the plan; selection semantics are left to the existing command.
- (C-006/C23-A004) branch_merged may be emitted inside merge_branch_with_ssb and again in the wrapper task; the source does not define a single emission owner. [C-23 L57, L101-L106]
- (C-006/C23-I001) Refresh-after-success makes UI mutations eventually consistent with SessionDag; no optimistic update is specified.
- (C-006/C23-I002) Typed DagBranchError is intended to preserve machine-readable failure semantics across Tauri, although its serialization format is not specified.
- (SUMMARY/S-E0319) Replaces “use CLI” stubs in fork_session_branch, merge_session_branch, and close_session_branch Tauri commands with real SessionDag calls. [L305]
- (SUMMARY/S-E0321) Mounts BranchTree.svelte in SessionsDrawer. [L306]
- (SUMMARY/S-E0322) Wires BranchTree’s onfork, onmerge, and onclose callbacks to live IPC. [L306]
- (SUMMARY/S-E0317) Closes B-9, B-3, and phase-3H; depends on C-3 and C-4; is parallel-safe with C-21, C-22, and C-24. [L300-L303]
- (SUMMARY/S-E0318) Adds merge and close arms to the /session command parser. [L304]
- (SUMMARY/S-E0320) Requires AppState to hold a DAG handle and uses ssb_writer for those mutations. [L305]
- (SUMMARY/S-E0323) Changes CommandWindow /continue from text-area prepopulation to invoking load_handoff. [L307]
- (SUMMARY/S-E0324) Fixes parent_chain to be root-to-tip by reversing the accumulated chain and updates tests accordingly. [L308]
- (SUMMARY/S-E0325) Adds a 100-run SessionDag determinism test. [L309]
- (SUMMARY/S-E0326) Verifies SettingsWindow SessionHandoffEntry does not contain stale V1 fields. [L310]
- (SUMMARY/S-E0327) Adds branch_forked, branch_merged, and branch_closed action types. [L311]
- (SUMMARY/S-E0328) Renders branch-event banners in ActivityWindow and SessionsDrawer. [L311]
- (SUMMARY/S-E0329) Long-task smoke crosses the 85% context threshold, resumes through /continue, then forks, merges, and closes a named branch and expects SSB checkpoint rows. [L312]

### C-24: Symbol impact/flow/context graph tools and blast-radius UI

**Status**: planned
**Anchors**: ledgers C-005, SUMMARY | sample anchors: L1-L9; L89-L94; L96-L103; L101-L103; L194-L197; L204-L213 | 47 deep atoms, 18 summary atoms | IDs: C-005/C24-E01, C-005/C24-E05, C-005/C24-E06, C-005/C24-E07, C-005/C24-E18, …
**Composes with**: C-1, C-5, C-9, C-16, C-17, C-21, C-22, C-23, C-25, C-28, C-30, C-32, C-34, C-35, C-36, C-44, C-49, C-56, C-58. IX: C-005/X-I02 → C-5, C-16; C-005/C24-E01 → C-17, C-21; C-005/X-A05 → C-5, C-9, C-16, C-28, C-32, C-34, C-35, C-36, C-44, C-49; C-005/C24-I02 → C-5; C-005/X-I05 → C-5, C-16, C-28, C-32, C-34, C-35, C-36, C-44, C-49; C-001/X-006 → C-1, C-17, C-30, C-56, C-58

- (C-005/C24-E01) The two-session plan closes B-29/B-33 reachability, incremental update, CLI/dispatcher export, decomposition styling, blast-radius UI, and missing tests; it depends on C-17 and is parallel-safe with C-21/22/23/25. [L1-L9]
- (C-005/C24-E05) KnowledgeDrawer’s impact panel adds Svelte-5-state target input and `Run`, invoking `submit_task` with `"/symbol_impact target=" + target` and rendering the returned `blast_radius`. [L89-L94]
- (C-005/C24-E06) CLI gains `SlashCommand::GraphMermaid { mode: GraphMode, max_nodes: usize }`, delegates via IPC to `export_graph_mermaid`, and prints returned Mermaid to stdout. [L96-L103]
- (C-005/C24-E07) CLI modes are `file`, `symbol`, and `decomposition`; `max_nodes` defaults to 200. [L101-L103]
- (C-005/C24-E18) CLI export must use the real generator, and dispatcher export must delegate rather than duplicate generation. [L194-L197]
- (C-005/C24-E20) Cloud rename smoke expects a `SymbolQuery` row with `tool="symbol_impact"`, target `SsbActionType`, and non-empty blast radius; UI/artifact evidence includes decomposition styling, deterministic zero-LLM CLI Mermaid, Tauri parity, and at least ten callers in the drawer. [L204-L213]
- (C-005/C24-E21) Acceptance raises the catalog to at least ten tools, tests both threshold branches/event, checks the form, CLI, cross-layer deterministic export, guarded styles, shared-store action, both four-case suites, scoped Rust builds, Svelte check, and the decision document. [L255-L291]
- (C-005/C24-E22) Implementation modifies production RPC catalog, indexing service, dispatcher, CLI, three Svelte surfaces, and adds two test files plus one decision artifact. [L293-L311]
- (C-005/C24-A02) CLI does not expose `project_id` even though dispatcher input requires it; implicit current-project resolution is not stated. [L96-L112]
- (C-005/C24-A03) The consumed graph store mentions `trigger_refresh`, but the click workflow names only `request_impact`, `set_mode`, and `openWindow`; whether impact implicitly refreshes is unspecified. [L127-L133, L178-L179]
- (C-005/C24-D01) Add all four JSON schemas/defaults, the full-graph `> threshold` branch, exact submit string, CLI modes/max-node default, dispatcher request/response schema, exact Cytoscape shapes/colors, and store-call ordering. [L72-L133]
- (C-005/C24-D03) Qualify the summary’s CLI example: the original does not define where its `SsbActionType` positional argument maps. [L96-L112, L248-L249]
- (C-005/C24-E02) The existing six-tool agent catalog gains `symbol_impact(target:string, depth:u8=2)`, `symbol_flow(target:string, depth:u8=5)`, `symbol_context(target:string)`, and `symbol_list(prefix:string)`. [L72-L78]
- (C-005/C24-E03) Incremental indexing calls `update_changed_files_symbol_graph(changed_paths)` when `changed_file_count <= INCREMENTAL_SYMBOL_THRESHOLD` (declared as 50); larger changes use full `build_symbol_graph`. [L80-L87]
- (C-005/C24-E04) The incremental branch emits `SsbActionType::SymbolGraphUpdated` with `is_incremental=true`. [L85-L87]
- (C-005/C24-E08) Dispatcher gains `DispatchTool::ExportGraphMermaid` with input `{project_id:string, mode:string, max_nodes:usize}` and output `{mermaid:string}`, delegating to the knowledge generator used by Tauri. [L105-L112]
- (C-005/C24-E09) Decomposition style maps `Concept` to a 180-wide 3px large rounded rectangle, `Phase` to 140/2px medium rounded, `Step` to 100/1px small rounded, and `BabyStep` to a 60-wide circle. [L114-L119]
- [status] (C-005/C24-E10) Decomposition statuses map pending to `#9e9e9e`, running to `#1976d2` with a transition/pulse intent, passed to `#388e3c`, failed to `#d32f2f`, and blocked to `#f57c00`. [L120-L124]
- [status] (C-005/C24-E11) Level/status classes apply only when `graphMode === "decomposition"`. [L125]
- (C-005/C24-E12) Every file-write SSB row gains `Show blast radius`; clicking runs graph-store `request_impact(filePath)`, sets file mode, and opens GraphWindow through workspace state. [L127-L133]
- (C-005/C24-E13) A decision document records tool-schema shape, threshold-branch placement, decomposition guard, and cross-link store-call choices. [L151-L155]
- (C-005/C24-E14) `graph_export_dispatch.rs` constructs a minimal IPC request, requires an OK Mermaid header, byte-equal repeat output for identical input, and a non-empty `%% empty graph` representation for `VizData::empty()`. [L135-L142]
- (C-005/C24-E15) `graph_commands.rs` requires file-mode fixture data, a non-empty known-symbol impact overlay, symbol-mode `classDef`, and no panic/error for all three commands on an empty fixture. [L144-L149]
- (C-005/C24-E16) Catalog completion forbids exposing only a subset of the four symbol tools. [L189-L191]
- (C-005/C24-E17) Incremental indexing may no longer call full graph construction unconditionally. [L192-L193]
- (C-005/C24-E19) ActivityWindow must use the shared graph store rather than invoke Tauri directly. [L198-L199]
- (C-005/C24-E23) Deterministic Mermaid repeatability is attributed to `BTreeMap` iteration in the existing generator. [L135-L142]
- (C-005/C24-E24) Every acceptance box is unchecked; the plan does not claim these reachability/UI/test additions are shipped. [L255-L291]
- (C-005/C24-D02) Add all eight named test assertions, zero/empty behavior, Tauri/dispatcher parity obligation, catalog completeness prohibition, and exact build/UI gates. [L135-L149, L189-L213, L255-L291]
- (C-005/C24-A01) Examples pass `SsbActionType` after `/graph mermaid`, but `GraphMermaid` declares no target argument and the dispatcher schema declares no target field; parsing/meaning of that positional token is unresolved. [L96-L112, L248-L249, L272-L274]
- (C-005/C24-A04) A CSS/Cytoscape `transition` property is called a running “pulse,” but no timing/keyframe behavior is specified. [L120-L124]
- (C-005/C24-A05) The cloud trigger is an impact/file graph while the expected artifact specifically calls for decomposition styles; no mode transition to decomposition is defined in that workflow. [L204-L213]
- (C-005/C24-A06) The empty-graph sentinel is called a “stub” in acceptance even though the stub ban requires real generation; it appears to be an intentional valid empty representation. [L135-L142, L194-L197, L284-L285]
- (C-005/C24-I01) The tool catalog and dispatcher arms form a two-step reachability contract: existing implementations are not agent-usable until their JSON definitions are prompt-visible. [L13-L19, L72-L78]
- (C-005/C24-I02) Dispatcher and Tauri export are intended as two adapters over one deterministic knowledge implementation, not two serialization authorities. [L105-L112, L194-L197]
- (SUMMARY/S-E0334) Adds a target form and Run button to KnowledgeDrawer’s IMPACT ANALYSIS panel. [L320]
- (SUMMARY/S-E0343) That action calls graph state request_impact and opens a graph window. [L324]
- (SUMMARY/S-E0330) Closes B-29 and B-33; depends on C-17; is parallel-safe with C-21, C-22, C-23, and C-25. [L314-L317]
- (SUMMARY/S-E0331) Adds ToolDefinition catalog entries for symbol_impact, symbol_flow, symbol_context, and symbol_list so the agent can select already-implemented dispatcher arms. [L318]
- (SUMMARY/S-E0332) Updates the symbol graph on the incremental indexing path when changed_file_count is at most INCREMENTAL_SYMBOL_THRESHOLD, fixed at 50. [L319]
- (SUMMARY/S-E0333) Emits SymbolGraphUpdated with is_incremental true for that path. [L319]
- (SUMMARY/S-E0335) Submits impact work by invoking submit_task with a /symbol_impact target command. [L320]
- (SUMMARY/S-E0336) Adds SlashCommand::GraphMermaid. [L321]
- (SUMMARY/S-E0337) Produces Mermaid graph output without an LLM and deterministically. [L321]
- (SUMMARY/S-E0338) Adds DispatchTool::ExportGraphMermaid and delegates to the knowledge Mermaid generator. [L322]
- (SUMMARY/S-E0339) Adds Cytoscape selectors for Concept, Phase, Step, and BabyStep decomposition shapes. [L323]
- (SUMMARY/S-E0340) Adds pending, running, passed, failed, and blocked graph-status classes. [L323]
- (SUMMARY/S-E0341) Shows those decomposition controls only in decomposition mode. [L323]
- (SUMMARY/S-E0342) Adds “Show blast radius” to ActivityWindow file-write rows. [L324]
- (SUMMARY/S-E0344) Adds graph export assertions and four Tauri graph-command smokes. [L325]
- (SUMMARY/S-E0345) Cloud rename smoke expects the agent to self-select symbol_impact and return a non-empty blast radius. [L326]

### C-25: Compiled pipelines, TemplateCrystallizer, background tasks, token tracker

**Status**: planned
**Anchors**: ledgers C-004, SUMMARY | sample anchors: C-25 L129-L132; C-25 L162-L177; C-25 L193-L236; L337; C-25 L1-L10, L238-L272; C-25 L10-L13 | 60 deep atoms, 27 summary atoms | IDs: C-004/F-C25-034, C-004/F-C25-037, C-004/F-C25-043, C-004/F-C25-001, C-004/F-C25-002, …
**Composes with**: C-4, C-8, C-9, C-21, C-24, C-26, C-38, C-46, C-47, C-52, C-54. IX: SUMMARY/S-E0346 → C-9, C-21, C-24, C-26; C-004/F-C25-002 → C-9, C-21, C-24, C-26; C-004/X-006 → C-4, C-9, C-26; C-004/X-014 → C-4, C-8, C-21, C-26, C-38, C-46, C-47, C-52, C-54; C-004/X-005 → C-4, C-26, C-54; C-004/X-008 → C-4, C-21, C-26, C-46, C-47, C-54

- (C-004/F-C25-034) CLI adds `SlashCommand::Pipeline(PipelineSubcommand)` with `List` and `Reset`. [C-25 L129-L132]
- (C-004/F-C25-037) Token tracking is reached from cloud response round trips; crystallization from ResearchTree post-step; compiled execution from pipeline factory entry; background management from autonomous Bash; and cache administration from CLI. [C-25 L162-L177]
- (C-004/F-C25-043) Ten deterministic check groups verify accounting, events, manager functions, compiled types/store/TTL, try-template wiring, source schema, CLI commands, workspace/scoped tests, and the cache artifact. [C-25 L193-L236]
- (C-004/F-C25-001) C-25 is an unchecked forward plan implementing compiled-pipeline Designs 1–3: compiled templates/execution, background task management, and cumulative token/cache accounting. [C-25 L1-L10, L238-L272]
- [gate] (C-004/F-C25-002) It requires C-9’s autonomous gate and `gate_decision` observation loop, is declared parallel-safe with C-21–C-24 and C-26, and estimates two to three sessions. [C-25 L10-L13]
- (C-004/F-C25-003) `CumulativeTokenTracker` has six `u64` fields: `prompt_tokens_total`, `completion_tokens_total`, `cache_read_tokens`, `cache_write_tokens`, `last_response_prompt`, and `last_response_completion`; it derives debug/clone/default/serde support. [C-25 L46-L54]
- (C-004/F-C25-004) `record_response(&ProviderUsage)` accumulates usage, and `recalibrate_budget(&mut ModelCapabilityProfile)` sets `attention_ceiling = max(4096, last_response_prompt + 2048)`. [C-25 L46-L54]
- (C-004/F-C25-005) Before adding a usage type, implementation must reuse an existing `ProviderUsage` if present; required usage fields are `input_tokens`, `output_tokens`, `cache_read_tokens`, and `cache_write_tokens`, all `u64`. [C-25 L52-L54]
- (C-004/F-C25-006) Cloud mode shares the tracker as `Arc<Mutex<CumulativeTokenTracker>>`, records after every `SpendAccumulator::record()`, and recalibrates before each outbound request without changing billing. [C-25 L56-L59]
- (C-004/F-C25-007) `BackgroundTaskStarted` carries `task_id`, `command_summary`, `soft_timeout_secs`, and `hard_timeout_secs`; `BackgroundTaskComplete` carries `task_id`, `exit_status`, `duration_secs`, `auto_detached`, and `timed_out`. [C-25 L61-L64]
- (C-004/F-C25-008) `BackgroundTaskManager` owns `active_tasks: HashMap<String, BackgroundTaskHandle>`, `max_concurrent: u8 = 3`, an SSB sender, and `poll_interval_secs: u32`. [C-25 L66-L68]
- (C-004/F-C25-009) `BackgroundTaskHandle` stores `task_id`, `spawned_at: Instant`, soft/hard timeouts, `JoinHandle<i32>`, and `auto_detached`. [C-25 L68-L70]
- (C-004/F-C25-010) `spawn_task()` emits `BackgroundTaskStarted` and returns a task ID; `poll_completed()` harvests finished handles and emits `BackgroundTaskComplete`. [C-25 L70-L72]
- (C-004/F-C25-011) `enforce_timeouts()` drops/detaches at the soft boundary with `auto_detached=true` and aborts at the hard boundary; `gated_spawn()` refuses when `autonomous_mode=false`. [C-25 L71-L74]
- (C-004/F-C25-012) Background polling backs off across 5, 10, then 30 seconds by doubling on idle cycles. [C-25 L74-L75]
- (C-004/F-C25-013) Lowering child-process priority is optional and may be skipped if neither an existing `nix` nor `winapi` dependency supports it; the choice must be documented. [C-25 L74-L76]
- (C-004/F-C25-014) `TemplateVariable` has `name`, `region_start`, `region_end`, and `variable_type`; `VariableType` is `ConceptClass`, `TaskDescription`, `CodeTarget`, or `Other(String)`. [C-25 L78-L81]
- (C-004/F-C25-015) `CompiledStep` has `step_index: u8`, `template_prompt`, `variables`, `variable_region_ratio: f32`, and `l1_tool_only: bool`. [C-25 L81-L82]
- (C-004/F-C25-016) `CompiledPipeline` has UUID ID, concept class, `plan_skeleton_hash: u64`, steps, create/last-use times, `use_count: u32`, `first_try_success_rate: f32`, and `compilation_eligible: bool`. [C-25 L83-L86]
- (C-004/F-C25-017) A pipeline is eligible only when every step is L1-tool-only, maximum variable-region ratio is strictly below 0.3, and use count is at least three. [C-25 L84-L87]
- (C-004/F-C25-018) `TemplateId(Uuid)` and `PipelineOutcomeId(Uuid)` are newtypes; compiled types derive debug/clone/serde and persist through `rmp-serde`, after verifying both dependencies already exist. [C-25 L87-L89]
- (C-004/F-C25-019) `TemplateCrystallizer` keeps `observation_log: HashMap<(String,u64), Vec<ExecutionOutcome>>`, threshold 3, and an SSB sender. [C-25 L91-L93]
- (C-004/F-C25-020) `ExecutionOutcome` records success, concept class, skeleton hash, optional pipeline UUID, token count, and timestamp. [C-25 L93-L94]
- (C-004/F-C25-021) `record_execution()` appends to the concept/hash bucket; `check_crystallization()` returns a compiled pipeline only when the last N outcomes are consecutive successes and the candidate is compilation-eligible. [C-25 L94-L98]
- (C-004/F-C25-022) Crystallization emits `template_crystallized` with concept class, skeleton hash, pipeline ID, and `threshold_n`. [C-25 L96-L98]
- (C-004/F-C25-023) `CompiledExecutor` owns a shared read/write pipeline cache, consecutive-failure counts, confidence threshold 0.7, and an SSB sender. [C-25 L100-L103]
- (C-004/F-C25-024) `try_execute()` returns `Err(CacheMiss)` for an absent `(concept_class, plan_hash)` key. [C-25 L100-L104]
- [failure] (C-004/F-C25-025) Two consecutive failures or confidence below 0.7 emits `decompile_triggered` and returns `Err(Decompile)`; the caller falls back to LLM decomposition. [C-25 L103-L106]
- (C-004/F-C25-026) `CompiledPipelineStore` keeps a cache path, keyed entries, and default TTL `604800` seconds (seven days). [C-25 L108-L113]
- (C-004/F-C25-027) Disk load decodes MessagePack; corrupt input warns, returns an empty store, and never panics. [C-25 L108-L112]
- (C-004/F-C25-028) Persistence uses an atomic temporary-file-plus-rename sequence; the `.elai/cache/` directory is created if absent. [C-25 L111-L113]
- (C-004/F-C25-029) Expiry removes entries whose `now - last_used_at` exceeds TTL, and `upsert()` inserts/updates before running eviction. [C-25 L112-L113]
- (C-004/F-C25-030) `classify_task_for_template(&task)` heuristically returns `(concept_class, plan_skeleton_hash)` or `None`; it uses SeaHash only if already present, otherwise `DefaultHasher` with the choice documented. [C-25 L115-L118]
- (C-004/F-C25-031) Pipeline factory selection tries a compiled template first; success emits `plan_skeleton` with `source=Template`, while decompile falls through and the existing generated path emits `source=Llm`. [C-25 L115-L121]
- (C-004/F-C25-032) `PlanSkeletonSource { Template, Llm }` becomes a required field of `PlanSkeletonPayload`; omission is a wiring violation. [C-25 L119-L121, L178-L183]
- [config] (C-004/F-C25-033) `ResearchTreeConfig` gains optional shared crystallizer/store handles; each completed baby step records/checks, then an emitted candidate is upserted and persisted. [C-25 L123-L127]
- (C-004/F-C25-035) `/pipeline list` prints concept class, skeleton hash, use count, first-try success rate, and creation time for every stored entry. [C-25 L129-L132]
- (C-004/F-C25-036) `/pipeline reset` clears the in-memory store and deletes the MessagePack file. [C-25 L129-L132]
- (C-004/F-C25-038) Stub bans require the 4,096 budget clamp, explicit cache miss, autonomous-only spawning, corrupt-cache recovery, and a populated plan-skeleton source. [C-25 L178-L183]
- (C-004/F-C25-039) The primary template smoke runs the same two-method-counter task four times: runs 1–3 use LLM skeletons, run 3 crystallizes at threshold three, and run 4 uses the template. [C-25 L134-L138, L184-L191]
- (C-004/F-C25-040) Run 4 must use fewer tokens than the mean of runs 1–3 and expose positive `cache_read_tokens` in `CloudSpendPayload`. [C-25 L134-L141, L261-L264]
- (C-004/F-C25-041) The cache smoke requires nonempty `.elai/cache/compiled_pipelines.msgpack` and at least one `/pipeline list` entry. [C-25 L184-L191, L261-L271]
- (C-004/F-C25-042) A long autonomous Bash smoke must emit started/complete events, cross the soft timeout, and complete with `auto_detached=true`. [C-25 L139-L141, L265-L266]
- (C-004/F-C25-044) Acceptance requires at least eight new test functions across compiled-pipeline, background-task, and cumulative-token scopes, plus all pipeline contract tests. [C-25 L267-L270]
- (C-004/F-C25-045) `C-25-decisions.md` must justify the +2,048 attention margin, two-failure decompile threshold, and seven-day TTL. [C-25 L143-L145, L272]
- (C-004/F-C25-046) If C-9 has not shipped, it is explicitly the true blocker for fully verifying the crystallization smoke even if implementation work proceeds. [C-25 L297-L302]
- (C-004/F-C25-047) Soft timeout is described as dropping the join handle (which detaches execution) while the smoke expects a `BackgroundTaskComplete` record for the detached task; after dropping the only handle, the stated manager has no specified mechanism to observe eventual exit or enforce the later hard timeout. [C-25 L66-L75, L139-L141]
- (C-004/F-C25-048) The source does not define confidence computation, initial value, or update rules for the 0.7 decompile threshold. [C-25 L100-L106]
- (C-004/F-C25-049) The concept-class heuristic and plan-skeleton canonicalization/hash stability are intentionally unspecified beyond a hasher preference. [C-25 L115-L118]
- (C-004/F-C25-050) Atomic rename gives crash-safe replacement on one filesystem, but the source does not specify fsync/durability, cross-process locking, or concurrent writer conflict handling. [C-25 L108-L113]
- (C-004/F-C25-051) C-25 calls C-26 “parallel-safe” while C-26 explicitly makes C-25 a prerequisite for its measurement smoke; code edits may be orthogonal, but completion ordering is not parallel. [C-25 L11-L13; C-26 L12-L15]
- (C-004/F-C25-052) All C-25 acceptance boxes are unchecked, so values and behaviors are requirements rather than shipped observations. [C-25 L238-L272]
- (C-004/D-C25-001) C-25 is wholly unchecked, and C-9 is the explicit blocker for final crystallization acceptance. [Summary L560-L598; C-25 L238-L272, L297-L302]
- (C-004/D-C25-002) Tracker/background/compiled types have exact fields; eligibility requires all-L1 steps, ratio `<0.3`, and use count `>=3`. [Summary L568-L591; C-25 L46-L89]
- (C-004/D-C25-003) Persistence recovers corrupt MessagePack to empty+warn, persists through temp+rename, evicts after 604800 seconds during upsert, and `/pipeline reset` deletes the file. [Summary L574-L575, L596; C-25 L108-L132]
- (C-004/D-C25-004) Cache matching, confidence derivation, concurrency/durability, and concept/hash canonicalization are not fixed by the plan. [Summary L568-L575, L593-L595; C-25 L100-L127]
- (C-004/D-C25-005) Dropping the only join handle on soft detach is not reconciled with later completion observation/hard-timeout enforcement. [Summary L579-L584, L598; C-25 L66-L75, L139-L141]
- (SUMMARY/S-E0369) Adds /pipeline list and /pipeline reset CLI commands. [L337]
- (SUMMARY/S-E0346) Closes guide/compiled-pipeline-design and guide/post-phase4-implementation-order; depends on C-9; is parallel-safe with C-21 through C-24 and C-26. [L328-L331]
- (SUMMARY/S-E0347) Adds CompiledPipeline, CompiledStep, TemplateVariable, and VariableType. [L332]
- (SUMMARY/S-E0348) VariableType contains ConceptClass, TaskDescription, CodeTarget, and Other. [L332]
- (SUMMARY/S-E0349) TemplateCrystallizer promotes a shape after three consecutive successes. [L332]
- (SUMMARY/S-E0350) Promotion emits template_crystallized to SSB. [L332]
- (SUMMARY/S-E0351) CompiledExecutor reports CacheMiss on a missing compiled pipeline. [L332]
- [failure] (SUMMARY/S-E0352) Two failures or confidence below 0.7 cause Decompile and emit decompile_triggered. [L332]
- (SUMMARY/S-E0353) CompiledPipelineStore persists MessagePack at .elai/cache/compiled_pipelines.msgpack. [L332]
- (SUMMARY/S-E0354) Stored pipelines are evicted after a seven-day TTL. [L332]
- (SUMMARY/S-E0355) Adds BackgroundTaskManager with maximum concurrency three. [L333]
- (SUMMARY/S-E0356) A soft timeout automatically detaches a task. [L333]
- (SUMMARY/S-E0357) A hard timeout aborts a task. [L333]
- (SUMMARY/S-E0358) Polling adapts from 5 seconds to 10 seconds to 30 seconds. [L333]
- [gate] (SUMMARY/S-E0359) gated_spawn requires autonomous_mode. [L333]
- (SUMMARY/S-E0360) Adds BackgroundTaskStarted and BackgroundTaskComplete event types and payloads. [L333]
- (SUMMARY/S-E0361) Adds CumulativeTokenTracker. [L334]
- (SUMMARY/S-E0362) Records ProviderUsage responses. [L334]
- (SUMMARY/S-E0363) Recalibration sets attention_ceiling to the greater of 4,096 and last_response_prompt plus 2,048. [L334]
- (SUMMARY/S-E0364) Runs the tracker beside SpendAccumulator without changing billing. [L334]
- (SUMMARY/S-E0365) Reads cache_read_tokens from openai_compat.rs at the stated location 363. [L334]
- (SUMMARY/S-E0366) Pipeline builder tries a compiled template before generating a fresh pipeline. [L335]
- (SUMMARY/S-E0367) Adds PlanSkeletonSource values Template and Llm to PlanSkeletonPayload. [L335]
- (SUMMARY/S-E0368) Runs TemplateCrystallizer as a ResearchTreeConfig post-step hook. [L336]
- (SUMMARY/S-E0370) Repeating the same counter-struct task three times must crystallize the template on run three; run four must use Template, show cache reads, and consume fewer tokens. [L338]
- (SUMMARY/S-E0371) A long autonomous bash task must emit BackgroundTaskStarted and BackgroundTaskComplete with auto_detached true. [L338]

### C-26: Constrained decoding, retry strategy, and role decoding defaults

**Status**: planned
**Anchors**: ledgers C-004, SUMMARY | sample anchors: C-26 L1-L15, L266-L293; C-26 L12-L15; C-26 L44-L47; C-26 L48-L50; C-26 L51-L52; C-26 L53-L57 | 51 deep atoms, 25 summary atoms | IDs: C-004/F-C26-001, C-004/F-C26-002, C-004/F-C26-003, C-004/F-C26-004, C-004/F-C26-005, …
**Composes with**: C-4, C-5, C-8, C-9, C-10, C-21, C-24, C-25, C-31, C-32, C-37, C-38, C-46, C-47, C-52, C-54. IX: C-004/F-C26-002 → C-10, C-21, C-24, C-25; C-004/X-005 → C-4, C-25, C-54; C-004/X-006 → C-4, C-9, C-25; C-004/X-014 → C-4, C-8, C-21, C-25, C-38, C-46, C-47, C-52, C-54; C-004/D-C26-001 → C-10; SUMMARY/S-E0372 → C-10, C-21, C-24, C-25

- Superseded: (SUMMARY/S-E0390) Replaces zero placeholders with real non-zero model-baseline values for measurements three, four, and five. [L348]
- (C-004/F-C26-001) C-26 is an unchecked forward plan closing token-efficiency and update-order gaps with provider decoding configuration, retry strategy, two futility paths, provider wiring, fixtures/measurements, grammars, CRANE, and role defaults. [C-26 L1-L15, L266-L293]
- (C-004/F-C26-002) It requires C-10 types for similarity/persona logic and C-25 token tracking for smoke measurements; it is declared parallel-safe with C-21–C-24. [C-26 L12-L15]
- (C-004/F-C26-003) `DecodingMode` has four variants: `JsonSchema { schema_json: String }`, `Grammar { ebnf_path: PathBuf, crane_adaptive: bool }`, `Regex { pattern: String }`, and `FreeText`, with debug/clone/equality/serde. [C-26 L44-L47]
- [config] (C-004/F-C26-004) `ConstrainedDecodingConfig` has `mode`, `max_retries_on_invalid: u8 = 3`, and `strict: bool = true`; its intended default mode is `FreeText`. [C-26 L48-L50]
- (C-004/F-C26-005) `crane_adaptive_eligible()` returns true only for `Grammar` with `crane_adaptive=true`. [C-26 L51-L52]
- (C-004/F-C26-006) A local `RoleDecoding` enum avoids a providers→agent circular dependency. [C-26 L53-L57]
- (C-004/F-C26-007) Five role defaults are fixed: designer `FreeText`, executor `Grammar(function_signature.ebnf, crane_adaptive=true)`, critic `JsonSchema`, auditor `JsonSchema`, researcher `FreeText`. [C-26 L53-L57]
- (C-004/F-C26-008) `RetryStrategy` has temperature schedule default `[0.0, 0.2, 0.5]`, same-error threshold 3, and `decompose_on_failure=false`, with debug/clone/equality/serde. [C-26 L60-L64]
- (C-004/F-C26-009) `BubbleUpPolicy` gains `retry_strategy: RetryStrategy`, and every literal construction is migrated with struct-update syntax without changing existing retry-count logic. [C-26 L64-L70]
- (C-004/F-C26-010) `temperature_for_attempt()` saturates at the final schedule element and returns 0.0 rather than panicking for an empty schedule. [C-26 L67-L69]
- (C-004/F-C26-011) For each baby step, same-error futility stores the last N `FailureRecord.reason` strings in a capacity-N `VecDeque`; N identical strings emits `futility_detected { error_class, consecutive_count, level="baby_step" }` and returns `BubbleUpAction::Escalate`. [C-26 L72-L77]
- (C-004/F-C26-012) Output-stagnation futility computes Jaccard similarity through `GoalDriftCanary::compute(prev,curr)`; score strictly greater than 0.90 emits `futility_output_stagnation { similarity, threshold:0.90, level }`. [C-26 L78-L81]
- (C-004/F-C26-013) Output stagnation updates `navigator.suggested_persona` using `suggest_persona_for_pattern(StagnationPattern::NoDrift)`. [C-26 L78-L82]
- (C-004/F-C26-014) New futility code must contain no `.unwrap()`; if C-10’s module is absent, conditional-use blocks compile the C-10-dependent paths to a no-op. [C-26 L81-L84, L195-L200]
- (C-004/F-C26-015) Sidecar outbound requests accept optional `ConstrainedDecodingConfig`; JSON Schema writes `response_format={type:"json_object",schema:...}`, Grammar writes `grammar_path`, and FreeText adds nothing. [C-26 L86-L94]
- (C-004/F-C26-016) A sidecar `constrained_decoding_unavailable` response logs `tracing::warn!` and retries/falls back as FreeText without panic; silent swallowing is banned. [C-26 L91-L93, L195-L200]
- (C-004/F-C26-017) Cloud `JsonSchema` is encoded as an Anthropic tool named `structured_output` with `input_schema`, plus forced `tool_choice` for that tool. [C-26 L96-L104]
- (C-004/F-C26-018) Cloud Grammar and FreeText make no request changes; Grammar is explicitly sidecar-only. [C-26 L99-L104]
- (C-004/F-C26-019) `ModelFeatures.supports_grammar` profiles are a consumed patch surface for Grammar-mode availability. [C-26 L181-L188]
- (C-004/F-C26-020) The model-capability benchmark must locate and fix its actual fixture-path bug rather than assume a path from the report. [C-26 L106-L114]
- (C-004/F-C26-021) It adds valid `fixtures/templates.json` with three template entries and `fixtures/padding.txt` with 100 lines. [C-26 L106-L118]
- (C-004/F-C26-022) The benchmark is rerun, and actual nonzero measurements 3/4/5 replace zeros in `model_baseline.toml`; fabricated floats are forbidden. [C-26 L112-L118, L195-L202]
- (C-004/F-C26-023) `import_block.ebnf` covers a common subset of Rust `use` and Python import/from-import forms. [C-26 L120-L124]
- (C-004/F-C26-024) `function_signature.ebnf` covers Rust function signatures and Python `def` signature lines. [C-26 L124-L126]
- (C-004/F-C26-025) `error_type.ebnf` covers CamelCase identifiers ending in `Error`, `Err`, or `Exception`, including qualified paths such as `std::io::Error`. [C-26 L126-L129]
- (C-004/F-C26-026) Grammar files must be nonempty and resolve `ebnf_path` according to a documented cwd-relative versus embedded strategy. [C-26 L128-L132, L195-L202]
- (C-004/F-C26-027) `constrained_decoding_contract.rs` defines 12 named cases covering mode defaults/serialization, CRANE eligibility, retry temperature, policy inclusion, both futility paths, and three role defaults. [C-26 L134-L149]
- (C-004/F-C26-028) `benchmark_fixture_contract.rs` defines three cases for valid templates JSON, nonempty padding, and nonzero measurements 3/4/5. [C-26 L150-L155]
- (C-004/F-C26-029) Produced event types explicitly include `futility_detected` and `futility_output_stagnation`. [C-26 L162-L179]
- (C-004/F-C26-030) The observable happy path activates critic JSON Schema, expects `constrained_decoding_used` with `mode=JsonSchema`, `validation=Pass`, no retry, and no futility event. [C-26 L204-L209]
- (C-004/F-C26-031) The adversarial observable uses an always-rejecting regex and expects after three equal failures `futility_detected(error_class=constrained_decode_reject,count=3)` plus a persona-switch row. [C-26 L209-L213, L285-L290]
- (C-004/F-C26-032) The benchmark observable requires nonzero deltas for measurements 3/4/5 and the updated baseline on disk. [C-26 L212-L213, L291-L292]
- (C-004/F-C26-033) Eleven deterministic check groups verify types/default tables, retry/policy, events, provider wiring, assets, nonzero baselines, 15 tests, and clean provider/orchestrator/contracts compilation. [C-26 L215-L264]
- (C-004/F-C26-034) Acceptance calls for all 15 contract cases and the three-part cloud/regex/benchmark smoke to be proven. [C-26 L266-L293]
- (C-004/F-C26-035) `C-26-decisions.md` records local-role typing, grammar path resolution, temperature rationale, warned FreeText fallback, and fixture placeholder rationale. [C-26 L156-L160, L293]
- (C-004/F-C26-036) C-10 absence may ignore the similarity contract, but the source’s final note says the persona-switch smoke cannot close until C-10 ships. [C-26 L143-L145, L313-L317]
- (C-004/F-C26-037) Ordinary Rust `#[derive(Default)]` would yield numeric/bool defaults of 0/false, conflicting with the explicit `max_retries_on_invalid=3` and `strict=true`; the intended contract requires a manual/custom default even though “derive Default” is requested. [C-26 L48-L50]
- (C-004/F-C26-038) `strict` is named and defaulted but its validation, retry, and fallback semantics are never defined. [C-26 L48-L50]
- (C-004/F-C26-039) `Regex` is a declared mode and drives the adversarial smoke, but neither sidecar request encoding nor cloud behavior for Regex is specified. [C-26 L44-L47, L86-L104, L204-L213]
- (C-004/F-C26-040) The same-error path explicitly escalates but does not switch persona, while the always-failing-regex acceptance expects a persona switch from three equal errors; persona switching is only specified for the separate >0.90 output-similarity path. [C-26 L72-L82, L209-L213, L285-L290]
- (C-004/F-C26-041) `constrained_decoding_used` and a persona-switch SSB row are required observables, but neither is added to the produced-event list or assigned a payload schema/action-type task. [C-26 L162-L179, L204-L213]
- (C-004/F-C26-042) Same-error tracking stores full reason strings while the event calls the value `error_class`; normalization from reason text to class is not defined. [C-26 L73-L77]
- (C-004/F-C26-043) The source does not define schema-validation mechanics, handling of invalid `schema_json`, retry temperature application to provider calls, or whether the three-retry count includes the first attempt. [C-26 L44-L70, L86-L104]
- (C-004/F-C26-044) C-10 is listed as a prerequisite, yet missing C-10 is also supported through a no-op/ignored test; implementation can compile without it, but the named smoke and full acceptance cannot close. [C-26 L12-L14, L81-L84, L143-L145, L313-L317]
- (C-004/F-C26-045) All acceptance boxes are unchecked, so per-role defaults and benchmark values are intended rather than evidenced as shipped. [C-26 L266-L293]
- (C-004/D-C26-001) C-26 is unchecked; C-10 absence compiles/ignores one path, but full persona-switch smoke still requires C-10. [Summary L600-L630; C-26 L12-L15, L81-L84, L143-L145, L313-L317]
- (C-004/D-C26-002) “Derive Default” cannot produce the stated numeric/bool defaults; implementation needs a custom default for max retries 3 and strict true. [Summary L608-L610; C-26 L44-L50]
- (C-004/D-C26-003) `strict` semantics and Regex provider encoding are absent even though Regex is a mode and drives acceptance. [Summary L608-L622, L629; C-26 L44-L50, L86-L104]
- (C-004/D-C26-004) Same-error futility only escalates, whereas the Regex smoke also expects persona switching; persona switching is specified only for the separate similarity path. [Summary L616-L619, L629; C-26 L72-L82, L209-L213]
- (C-004/D-C26-005) `constrained_decoding_used` and the persona-switch row are acceptance observables without produced action/payload tasks. [Summary L618-L630; C-26 L162-L179, L204-L213]
- (C-004/D-C26-006) Fixture sizes/content, grammar-language subsets, actual-measurement/no-fabrication rule, and all 15 named contract responsibilities are explicit in the source. [Summary L623-L630; C-26 L106-L160]
- (SUMMARY/S-E0372) Closes guide/token-efficiency-design and guide/update-implementation-order; depends on C-10 and C-25; is parallel-safe with C-21 through C-24. [L340-L343]
- (SUMMARY/S-E0373) Adds DecodingMode variants JsonSchema, Grammar with ebnf_path and crane_adaptive, Regex, and FreeText. [L344]
- [config] (SUMMARY/S-E0374) Adds ConstrainedDecodingConfig with max_retries_on_invalid defaulting to three and strict defaulting to true. [L344]
- (SUMMARY/S-E0375) Adds crane_adaptive_eligible. [L344]
- (SUMMARY/S-E0376) Sets per-role defaults: designer FreeText, executor Grammar using function_signature.ebnf with crane-adaptive enabled, critic JsonSchema, auditor JsonSchema, and researcher FreeText. [L344]
- (SUMMARY/S-E0377) Adds RetryStrategy to BubbleUpPolicy. [L345]
- (SUMMARY/S-E0378) Uses temperature retry schedule 0.0, 0.2, 0.5. [L345]
- (SUMMARY/S-E0379) Stops same-error retries at three. [L345]
- [failure] (SUMMARY/S-E0380) Defaults decompose_on_failure to false. [L345]
- (SUMMARY/S-E0381) NavigatorTree detects repeated equal error classes in a VecDeque. [L346]
- (SUMMARY/S-E0382) NavigatorTree detects output stagnation when GoalDriftCanary similarity exceeds 0.90. [L346]
- (SUMMARY/S-E0383) Emits futility_detected and futility_output_stagnation. [L346]
- (SUMMARY/S-E0384) Switches persona after futility detection. [L346]
- (SUMMARY/S-E0385) SidecarClient maps constraints to response_format or grammar_path. [L347]
- (SUMMARY/S-E0386) If constrained decoding is unavailable, SidecarClient warns and falls back to FreeText. [L347]
- (SUMMARY/S-E0387) CloudGroundingClient implements JsonSchema through Anthropic tools plus tool_choice. [L347]
- (SUMMARY/S-E0388) Fixes a benchmark-fixture path. [L348]
- (SUMMARY/S-E0389) Creates fixtures/templates.json and fixtures/padding.txt. [L348]
- (SUMMARY/S-E0390) Replaces zero placeholders with real non-zero model-baseline values for measurements three, four, and five. [L348]
- (SUMMARY/S-E0391) Adds import_block.ebnf, function_signature.ebnf, and error_type.ebnf. [L349]
- (SUMMARY/S-E0392) Requires 15 contract tests across two files. [L350]
- (SUMMARY/S-E0393) Smoke A expects JsonSchema to pass without retry and log constrained decoding use. [L351]
- [failure] (SUMMARY/S-E0394) Smoke B forces Regex failure three times and expects futility detection plus persona switch. [L351]
- (SUMMARY/S-E0395) Smoke C requires non-zero benchmark deltas for measurements three through five. [L351]

### C-27: Determinism config, wiring contracts, and wiring audit CI

**Status**: planned
**Anchors**: ledgers C-002, SUMMARY | sample anchors: C-27:L1-L9; C-27:L138-L142; C-27:L189-L195; C-27:L253-L286; C-27:L288-L310; C-27:L90-L148 | 37 deep atoms, 23 summary atoms | IDs: C-002/C27-E01, C-002/C27-E20, C-002/C27-E24, C-002/C27-E27, C-002/C27-E28, …
**Composes with**: C-1, C-2, C-5, C-15, C-17, C-21, C-26, C-28, C-29, C-36, C-42, C-43, C-50, C-57. IX: C-002/C27-E02 → C-15, C-17, C-21, C-26; SUMMARY/S-E0396 → C-15, C-17, C-21, C-26; C-005/X-E04 → C-5, C-28, C-29, C-36; C-002/I01 → C-1, C-2, C-5, C-42, C-43, C-50, C-57

- Superseded: (C-002/SUP-C05) NEWER-PLAN PRECEDENCE: C-27 description is target state relative to older C plan(s) C-1, C-4, C-5, C-12, C-19 for this conflict. Losing description retained at C-002/C05. Conflict: Provider mode used by multiple smoke commands conflicts with C-5's corrected CLI axis. | Evidence: C-1/C-4/C-12/C-19/C-27 use `elai /task --mode cloud`, while C-5 says `--mode` is gate mode and provider is selected by `--provider`. [C-1:L228-L232; C-4:L195-L196; C-12:L152-L155; C-19:L236-L238; C-27:
- Superseded: (C-002/SUP-C06) NEWER-PLAN PRECEDENCE: C-27 description is target state relative to older C plan(s) C-1, C-5 for this conflict. Losing description retained at C-002/C06. Conflict: SSB schema baselines cannot all be current. | Evidence: C-27 mandates 46→47, C-1 says one bump and ≥47, while C-5 reports shipped schema 103. Integration must rebase to the actual current version rather than set 47. [C-27:L61-L71; C-27:L210-L211] versus [C-1:L56-L60; C-1:L210-L222] versus [C-5:L16-L
- Superseded: (C-005/SUP-C28-C01) NEWER-PLAN PRECEDENCE: C-28 description is target state relative to older C plan(s) C-27 for this conflict. Losing description retained at C-005/C28-C01. Conflict: The original says C-28 bumps schema 46→47, while the orientation summary says Mutation shares and keeps C-27’s already-bumped 47. Cross-plan migration order cannot be resolved from this file. [L28, L119-L126]
- [status] (C-002/C27-E01) Objective combines determinism tier T(-1) and wiring-contract tier T-0: hardware/seed/cache/fingerprint determinism, drift evidence, contract parsing/auditing, CLI, and CI; estimated 2-3 sessions. [C-27:L1-L9]
- (C-002/C27-E20) A Windows `wiring-audit.yml` builds release CLI and audits configured contracts on push and pull request to main, returns nonzero on violations, and skips `[ci skip]` commits. [C-27:L138-L142]
- (C-002/C27-E24) Explicit bans: no alternate digest; no scanning test/contract paths; no drift row on equality; no timeout panic; no panic on missing config; and exact CLI exit codes. [C-27:L189-L195]
- [gate] (C-002/C27-E27) Acceptance requires payload round-trips, exact violation kinds, correct first divergence, reachable/unreachable evidence, CLI behavior, four smoke scenarios, Windows CI triggers, clean workspace, ≥10 scoped unit tests, tier-0 contract pass, and the three-decision document. [C-27:L253-L286]
- (C-002/C27-E28) Adds determinism/wiring modules, CLI dispatch, CI workflow, contract tests, decision doc, and modifies SSB, CLI commands, and TOML. [C-27:L288-L310]
- (C-002/SD-C-27-02) Addition: Wiring contracts have a concrete field schema; CLI parses Markdown blocks, prints exact PASS/FAIL forms and exit codes; CI uses Windows and `[ci skip]`; observables poll at one-second cadence and fail with typed evidence. [C-27:L90-L148]
- (C-002/C27-E02) Requires C-15 clock/entropy and C-17 DKG resilience; declared parallel-safe with C-21 through C-26 except the sole shared serialization point in `ssb.rs`. [C-27:L10-L14]
- (C-002/C27-E03) Bump `SCHEMA_VERSION` 46→47 and add `WiringEvidence`, `WiringViolation`, and `DeterminismDrift` action/payload variants with Debug/Clone/Serialize/Deserialize. [C-27:L61-L71]
- (C-002/C27-E04) `WiringEvidence` contains `entrypoint:String`, `produces_symbols_verified:Vec<String>`, `reachable_count:u32`, and `audit_id:Uuid`. [C-27:L63-L64]
- (C-002/C27-E05) `WiringViolation` contains `violation_kind:WiringViolationKind`, `file:String`, `line:u32`, `pattern:String`, and `audit_id:Uuid`; kinds are `StubPattern`, `UnreachableSymbol`, and `BrokenObservable`. [C-27:L65-L69]
- (C-002/C27-E06) `DeterminismDrift` contains `seed:u64`, `run_a_hash:String`, `run_b_hash:String`, `first_divergence_index:u32`, and `audit_id:Uuid`. [C-27:L67-L68]
- [config] (C-002/C27-E07) `DeterminismConfig` fields are `hardware_class:HardwareClass`, `seed:u64`, `blake3_cache_prefix:[u8;8]`, and `system_fingerprint_pin:Option<String>`; `HardwareClass={Cpu,Gpu(GpuSku),Unknown}`, with `GpuSku` a string newtype. [C-27:L73-L76]
- [config] (C-002/C27-E08) `from_env` reads `ELAI_SEED` defaulting to 0 and `ELAI_HARDWARE_CLASS` defaulting to Unknown. `matches_fingerprint` returns true when no pin is configured. [C-27:L77-L82]
- [security] (C-002/C27-E09) The eight-byte prefix is the first eight bytes of `blake3::hash(seed.to_le_bytes())`; `prefix_cache_key` pads it to 32 bytes and uses `blake3::keyed_hash`. SHA-256/MD5 are forbidden for this cache key. [C-27:L78-L81; C-27:L189-L191]
- [failure] (C-002/C27-E10) `DeterminismError` is a `thiserror` enum with `EnvParseError(String)` and `HashKeyError`; determinism code may not unwrap. [C-27:L81-L82]
- (C-002/C27-E11) `DriftComparator {seed,ssb_tx,audit_id}` compares `ActionTypeSequence(Vec<String>)`, emits one drift row at the first differing zipped index with BLAKE3 hashes of joined sequences, emits zero rows for matches, and uses an unbounded SSB channel without unwrap. [C-27:L84-L88]
- (C-002/C27-E12) `WiringContract` fields are `bn_id`, `produces_symbols`, `consumes_symbols`, `reachable_from`, `stub_ban`, optional `frontend_pair`, and `observable_effects`; `StubBanEntry` has pattern/description and `ObservableEffect` has trigger, optional SSB/artifact expectations, and timeout seconds. [C-27:L90-L97]
- [failure] (C-002/C27-E13) `WiringContractError` variants are parse, I/O-from-`io::Error`, and regex errors; all core contract types derive Debug/Clone/Serialize/Deserialize. [C-27:L96-L97]
- [fallback/config] (C-002/C27-E14) `EntrypointSetLoader {config_path}` reads TOML `[wiring.entrypoints] symbols`; a missing section returns an empty set with `tracing::warn!`, never panic. A commented template is added to `elai_cc.toml`. [C-27:L99-L104]
- [security] (C-002/C27-E15) `StubBanAuditor` compiles regexes and emits `WiringViolation(StubPattern)` with file/line. Production walks exclude `/tests/`, `_test.rs`, and `_contract.rs`; default patterns cover `todo!()`, `unimplemented!()`, `deferred`, `stub`, and `Ok(Default::default())`. [C-27:L106-L113]
- (C-002/C27-E16) `DkgReachabilityAuditor` reuses `SymbolGraph` and breadth-first searches from every entrypoint, short-circuiting on first hit; reachable produced symbols emit `WiringEvidence`, unreachable ones emit `WiringViolation(UnreachableSymbol)`. [C-27:L115-L119]
- (C-002/C27-E17) `ObservableEffectRunner` polls the SQLite SSB DB every second under `tokio::time::timeout`; missing expected SSB or artifact produces `BrokenObservable` plus `Err`, never panic. [C-27:L121-L126]
- [config] (C-002/C27-E18) `WiringAuditCmd {config_path,contracts_dir,ssb_db_path,seed}` loads entrypoints, globs Markdown, parses `WIRING_CONTRACT` blocks, runs all three auditors, aggregates `WiringAuditReport {evidence,violations,audit_id}`, prints PASS/FAIL with file:line, and exits 0/1. [C-27:L128-L136]
- [config] (C-002/C27-E19) Expose `elai wiring audit [--config PATH] [--contracts PATH]` and add `SlashCommand::WiringAudit`. [C-27:L135-L136]
- (C-002/C27-E21) Unit coverage requires stable same-seed keys, drift/non-drift behavior, stub detection and test exclusion, empty missing-entrypoint behavior, and broken-observable timeout emission. [C-27:L144-L148]
- [gate] (C-002/C27-E22) Tier-0 contract requires the three SSB variants, public schema 47, environment seed 42 success, and zero drift for equal sequences. [C-27:L150-L154]
- (C-002/C27-E23) Decision record mandates BLAKE3 rather than SHA-256, BFS rather than DFS for bounded large-graph traversal, and one-second SQLite polling because there is no NOTIFY and polling is testable. [C-27:L156-L160]
- (C-002/C27-E25) Evidence scenarios cover clean entrypoints, a manually injected default-return stub with file/line violation, two seed-42 cloud runs with zero drift, and a nonexistent event producing `BrokenObservable` within timeout+5s. [C-27:L196-L205]
- [lifecycle] (C-002/C27-E26) Acceptance uniquely requires an old schema-46 DB to emit a migration warning when public schema is 47. [C-27:L253-L255]
- (C-002/SD-C-27-01) Addition: Exact payload fields, error variants, environment defaults, keyed-hash construction, equality/no-row rule, first-divergence behavior, and test-path exclusions are missing from the index. [C-27:L61-L119]
- (C-002/SD-C-27-03) Addition: Acceptance uniquely adds a schema-46 migration warning and ≥10 new unit functions. [C-27:L253-L285]
- (SUMMARY/S-E0396) Closes testing-trust T(-1) and T-0; depends on C-15 and C-17; is parallel-safe with C-21 through C-26. [L353-L356]
- (SUMMARY/S-E0397) Declares the only serialization mutation in its wave and says SCHEMA_VERSION moves from 46 to 47. [L357]
- (SUMMARY/S-E0398) Adds WiringEvidence, WiringViolation, and DeterminismDrift plus WiringViolationKind. [L358]
- [config] (SUMMARY/S-E0399) Adds DeterminismConfig fields hardware_class, seed, an eight-byte blake3_cache_prefix, and system_fingerprint_pin. [L359]
- (SUMMARY/S-E0400) Adds HardwareClass, GpuSku, and environment loading from ELAI_SEED and ELAI_HARDWARE_CLASS. [L359]
- (SUMMARY/S-E0401) Derives cache-key prefixes with blake3 keyed hashing and explicitly forbids sha256 for this purpose. [L359]
- (SUMMARY/S-E0402) DriftComparator emits DeterminismDrift at the first divergent index. [L359]
- (SUMMARY/S-E0403) Adds WiringContract, StubBanEntry, ObservableEffect, EntrypointSet, and EntrypointSetLoader. [L360]
- (SUMMARY/S-E0404) EntrypointSetLoader reads TOML wiring.entrypoints. [L360]
- [config] (SUMMARY/S-E0405) Missing entrypoint configuration produces an empty set plus a warning. [L360]
- (SUMMARY/S-E0406) StubBanAuditor compiles patterns for todo!, unimplemented!, Ok(Default::default()), “deferred”, and “stub”. [L360]
- (SUMMARY/S-E0407) Stub scanning excludes tests. [L360]
- (SUMMARY/S-E0408) DkgReachabilityAuditor performs breadth-first search from every entry point. [L360]
- (SUMMARY/S-E0409) ObservableEffectRunner polls SQLite for one second under a Tokio timeout. [L360]
- (SUMMARY/S-E0410) Observable timeout returns BrokenObservable instead of panicking. [L360]
- (SUMMARY/S-E0411) Adds elai wiring audit with optional config and contracts paths. [L361]
- (SUMMARY/S-E0412) The command parses Markdown WIRING_CONTRACT blocks and aggregates a WiringAuditReport. [L361]
- (SUMMARY/S-E0413) It exits zero when clean and one on violations. [L361]
- (SUMMARY/S-E0414) Adds a Windows push/PR workflow for main. [L362]
- (SUMMARY/S-E0415) Smoke A requires clean-audit WiringEvidence rows. [L363]
- (SUMMARY/S-E0416) Smoke B injects a stub and requires a WiringViolation containing file and line. [L363]
- (SUMMARY/S-E0417) Smoke C repeats seed 42 and requires no DeterminismDrift. [L363]
- (SUMMARY/S-E0418) Smoke D breaks an observable and requires BrokenObservable within the timeout plus five seconds. [L363]

### C-28: Mutation testing (cargo-mutants/mutmut) and AUDITING→MEMORIZING gate

**Status**: planned
**Anchors**: ledgers C-005, SUMMARY | sample anchors: L24-L26, L46-L50, L257-L258; L145-L150; L198-L203; L239-L253; L255-L281; L283-L297 | 55 deep atoms, 25 summary atoms | IDs: C-005/C28-E02, C-005/C28-E22, C-005/C28-E26, C-005/C28-E27, C-005/C28-E29, …
**Composes with**: C-4, C-5, C-9, C-16, C-21, C-24, C-26, C-27, C-29, C-32, C-34, C-35, C-36, C-44, C-48, C-49, C-52, C-54. IX: C-005/X-A05 → C-5, C-9, C-16, C-24, C-32, C-34, C-35, C-36, C-44, C-49; C-005/C28-I02 → C-5; C-005/X-I05 → C-5, C-16, C-24, C-32, C-34, C-35, C-36, C-44, C-49; C-005/X-E04 → C-5, C-27, C-29, C-36; C-004/X-013 → C-4, C-29, C-52, C-54; C-005/C28-E01 → C-21, C-27, C-29

- Superseded: (C-005/SUP-C28-C01) NEWER-PLAN PRECEDENCE: C-28 description is target state relative to older C plan(s) C-27 for this conflict. Losing description retained at C-005/C28-C01. Conflict: The original says C-28 bumps schema 46→47, while the orientation summary says Mutation shares and keeps C-27’s already-bumped 47. Cross-plan migration order cannot be resolved from this file. [L28, L119-L126]
- (C-005/C28-E02) The source pins `cargo-mutants 27.0.0` as an installed CLI/dev tool, explicitly not a Cargo dependency, and `mutmut 3.5.0` in the sidecar Python dev environment. [L24-L26, L46-L50, L257-L258]
- (C-005/C28-E22) MonitorWindow polls `get_mutation_gauge` every five seconds and renders green at/above threshold, amber below it, and grey for no result; the Tauri stub returns `None` until backend session-state wiring exists. [L145-L150]
- (C-005/C28-E26) Normal observable behavior has zero survivor rows, score at least 70, transition proceeds, green gauge, and faster repeat from cache hits. [L198-L203]
- (C-005/C28-E27) The adversarial tautology scenario yields score 0, a blocking finding, blocked transition, and amber gauge; replay expects every second-run mutant to be a cache hit and `total_run == cache_hits`. [L239-L253]
- (C-005/C28-E29) Acceptance covers exact dependency installs, tier/order, runner/types/formula, persistent TTL cache, event cardinality, four config keys/defaults, hard gate ordering, three-state gauge, null stub, no unwrap/expect, 4+3 contracts, normal/adversarial smoke, and faster all-cache replay. [L255-L281]
- (C-005/C28-E30) Implementation adds mutation code and two test files and modifies tier dispatch, auditor, SSB, runtime config/TOML, dashboard types/window/Tauri state, plus a decision document. [L283-L297]
- (C-005/C28-A06) `get_mutation_gauge` is explicitly a stub until live state wiring, yet acceptance otherwise describes a live cloud gauge; the handoff that makes it non-stub is not a separate task. [L140-L150, L272-L280]
- [config] (C-005/C28-D01) Add exact record/result/cache/config/gauge schemas, runner commands, partial-cap behavior, event cardinality exclusions, five predecessor checks, and every contract-test case. [L52-L169]
- (C-005/C28-D04) Correct the summary’s claim that both tools are non-Cargo CLI installs: `cargo-mutants` is a Cargo-installed CLI, while `mutmut` is a Python package in sidecar `pyproject.toml` dev dependencies. [L24-L26, L46-L50, L257-L258]
- (C-005/C28-E01) This two-session T-1 extension depends on C-27, is parallel-safe with C-29/C-21..26, and inserts mutation after an existing five-check deterministic chain. [L1-L9, L13-L22]
- (C-005/C28-E03) `ValidationTier::Mutation` sits between `Unit` and `Integration`; `as_str()` is `mutation`, timeout sentinel is `0` for runner-owned timing, and `ordered()` becomes six tiers with Mutation at index 3. [L52-L59]
- (C-005/C28-E04) `MutantOutcome` is `Caught | Survived | Timeout | Skipped(String)`. [L61-L70]
- (C-005/C28-E05) `MutantRecord` contains file, line, mutant kind, and outcome. [L65-L70]
- (C-005/C28-E06) `MutationResult` contains records, `mutation_score_pct`, `total_run`, and `cache_hits`; score is `caught/(caught+survived+timeout)*100`. [L71-L76]
- [failure] (C-005/C28-E07) Subprocess failures become a `thiserror` `MutationError`; production mutation code forbids unwrap/expect. [L78, L94-L96]
- (C-005/C28-E08) Async `MutationRunner::run(cfg, project_dir, cache)` returns `Result<MutationResult, MutationError>` and is `Send + Sync`. [L80-L87]
- (C-005/C28-E09) `RustMutationRunner` uses Tokio to run `cargo mutants --timeout N --output tmp --jobs 1 --file files`, parses caught/missed output, caps work at `max_mutants_per_session`, and scores the partial subset. [L88-L91]
- (C-005/C28-E10) `PythonMutationRunner` uses Tokio to run `mutmut run --paths-to-mutate files --runner "pytest -x -q"`, then reads `mutmut results`. [L92-L94]
- (C-005/C28-E11) Both runners enforce a per-mutant Tokio timeout. [L94-L96]
- (C-005/C28-E12) `EquivalentMutantCache` reuses the SSB SQLite-pool pattern and keys blake3 over pre-mutation bytes + mutant kind + line. [L98-L101]
- (C-005/C28-E13) Cache schema is `equiv_mutant_cache(key TEXT PRIMARY KEY, outcome TEXT, ts INTEGER)` at `<project>/.elai/mutation_cache.sqlite3`; it is created on demand. [L101-L104]
- (C-005/C28-E14) Cache hits become `Skipped("cache_hit")`; new outcomes are inserted, duplicate keys use upsert semantics in tests, and rows older than 30 days are rerun. [L102-L104, L157-L159]
- (C-005/C28-E15) `[testing.mutation]` defaults are `enabled=true`, `threshold_pct=70.0`, `timeout_per_mutant_sec=30`, and `max_mutants_per_session=100`; absent config deserializes to these defaults without error. [L106-L117]
- (C-005/C28-E16) `SsbActionType::MutationSurvived` and `MutationSurvivedPayload { file, line, mutant_kind }` are added across enum/SQL CHECK/string/from-string sites. [L119-L126]
- (C-005/C28-E17) One event is emitted per survivor and none for caught, timeout, or skipped records. [L123-L125]
- (C-005/C28-E18) Mutation executes only after Build, VersionGate, FileSizeCensus, LazyCode, and Completeness all pass. [L128-L138]
- (C-005/C28-E19) Changed Rust and Python files select the corresponding runners by extension; scores below threshold create blocking `DeterministicFinding` with `CheckKind::Mutation` and the numeric threshold message. [L130-L138]
- (C-005/C28-E20) Below-threshold mutation blocks the `AUDITING -> MEMORIZING` transition rather than warning. [L133-L138, L192-L197]
- (C-005/C28-E21) `MutationGaugeInfo` carries score, threshold, cache hits, total run, and last-updated milliseconds. [L140-L145]
- (C-005/C28-E23) The four runner-contract tests cover public schema 47, a 7/2/1 = 70% score, cache skip/upsert, and serde payload round-trip. [L152-L162]
- (C-005/C28-E24) The three schema-contract tests cover public schema 47, SSB SQL/from-string round-trip, and Mutation’s ordered index. [L163-L169]
- (C-005/C28-E25) The wiring contract bans mocked production runners, non-blake3 cache keys, and warning-only auditing behavior. [L171-L197]
- (C-005/C28-E28) Deterministic checks inspect version/types/config/blake3/no-unwrap and run the two contract binaries without live subprocess mutation. [L205-L237]
- (C-005/C28-E31) Every acceptance box is unchecked; dependency versions are reported verified, but the T-1 implementation is not claimed shipped. [L24-L28, L255-L281]
- (C-005/C28-D02) Add the all-cache replay invariant, zero-result score ambiguity, parser/timeout uncertainties, and the self-contradictory adversarial event sentence. [L239-L253]
- (C-005/C28-D03) Preserve the unresolved schema chronology instead of stating without qualification that C-28 merely keeps 47. [L28, L119-L126]
- (C-005/C28-C01) The original says C-28 bumps schema 46→47, while the orientation summary says Mutation shares and keeps C-27’s already-bumped 47. Cross-plan migration order cannot be resolved from this file. [L28, L119-L126]
- (C-005/C28-C02) The adversarial paragraph first says no surviving-mutant rows are emitted because outcomes are `Survived`, then immediately says such rows are emitted one per survivor; Tasks/Acceptance consistently require emission. [L241-L248]
- (C-005/C28-A01) `cargo-mutants --output` is described as JSON output and then as `caught.txt + missed.txt`; the exact parser format/version contract is unclear. [L88-L90]
- (C-005/C28-A02) A single external command is shown, but timeout is required “per mutant”; process subdivision/cancellation and partial-output recovery are not specified. [L88-L96]
- (C-005/C28-A03) Task routing says detect languages by changed-file extension, while the wiring contract says consume `ProjectProfile.commands`; their authority/precedence is unspecified. [L130-L132, L183-L186]
- (C-005/C28-A04) Score denominator behavior when no mutant runs (all cached/skipped, no applicable files, or tool failure) is not defined. [L71-L76, L98-L104]
- (C-005/C28-A05) Cache `outcome` persistence does not define which outcomes qualify as “equivalent,” how timeout/survivor rows are updated, or whether 30-day TTL cleanup is transactional under concurrent runs. [L98-L104]
- (C-005/C28-I01) The cache is intended to make repeated identical pre-mutation inputs idempotent with respect to execution cost while preserving a record of skipped outcomes. [L98-L104, L250-L253]
- (C-005/C28-I02) Mutation is both a validation tier and an FSM gate: the ordered tier exposes it to generic validation, while deterministic-auditor integration owns the state-blocking decision. [L52-L59, L128-L138]
- (SUMMARY/S-E0420) Pins cargo-mutants 27.0.0 and mutmut 3.5.0 as CLI tools, not Cargo dependencies. [L369]
- (SUMMARY/S-E0437) MonitorWindow displays a mutation gauge: green at or above threshold, amber below threshold, and grey for null. [L376]
- (SUMMARY/S-E0438) Adds a get_mutation_gauge Tauri-command stub for that gauge. [L376]
- (SUMMARY/S-E0419) Closes testing-trust T-1; depends on C-27; is parallel-safe with C-29 and C-21 through C-26. [L365-L368]
- (SUMMARY/S-E0421) Inserts ValidationTier::Mutation between Unit and Integration at position three. [L370]
- (SUMMARY/S-E0422) Adds CheckKind::Mutation to deterministic_auditor. [L370]
- (SUMMARY/S-E0423) Adds MutantOutcome values Caught, Survived, Timeout, and Skipped(String). [L371]
- (SUMMARY/S-E0424) Adds MutantRecord and MutationResult. [L371]
- (SUMMARY/S-E0425) Computes mutation score as caught divided by caught plus survived plus timeout, multiplied by 100. [L371]
- (SUMMARY/S-E0426) Adds MutationRunner. [L371]
- (SUMMARY/S-E0427) RustMutationRunner invokes cargo mutants with timeout, output, and jobs 1. [L371]
- (SUMMARY/S-E0428) PythonMutationRunner invokes mutmut run with pytest -x -q. [L371]
- (SUMMARY/S-E0429) Both runners use Tokio processes and per-mutant Tokio timeouts. [L371]
- (SUMMARY/S-E0430) Adds a blake3-keyed SQLite EquivalentMutantCache at .elai/mutation_cache.sqlite3. [L372]
- (SUMMARY/S-E0431) Cache entries expire after 30 days. [L372]
- [config] (SUMMARY/S-E0432) Reads mutation config from elai_cc.toml testing.mutation with serde defaults when absent. [L373]
- [config] (SUMMARY/S-E0433) Config fields/defaults are enabled, threshold_pct 70, timeout_per_mutant_sec 30, and max_mutants_per_session 100. [L373]
- (SUMMARY/S-E0434) Adds MutationSurvived in the four required lockstep event-registration sites while keeping SCHEMA_VERSION at 47 as a shared wave bump. [L374]
- (SUMMARY/S-E0435) DeterministicAuditor runs mutation after its existing five checks pass. [L375]
- (SUMMARY/S-E0436) A score below threshold creates DeterministicFinding::Block and blocks AUDITING to MEMORIZING. [L375]
- (SUMMARY/S-E0439) Requires two contract files with four and three tests. [L377]
- (SUMMARY/S-E0440) Normal sum smoke requires at least 70% mutation score; tautological-test smoke must block at auditing; a second run must be faster due to cache replay. [L378]

### C-29: Intent pairs, metamorphic relations, and PlanWindow editor

**Status**: planned
**Anchors**: ledgers C-003, SUMMARY | sample anchors: L102-L108; L1-L8; L43-L47; L46-L48; L49-L52; L51-L57 | 46 deep atoms, 23 summary atoms | IDs: C-003/E-C29-022, C-003/E-C29-023, C-003/DELTA!-C29-2, C-003/E-C29-001, C-003/E-C29-002, …
**Composes with**: C-3, C-4, C-5, C-7, C-10, C-14, C-20, C-21, C-26, C-27, C-28, C-32, C-36, C-45, C-52, C-54, C-59. IX: C-003/E-C29-020 → C-28; C-005/X-I01 → C-5, C-28, C-32, C-36; SUMMARY/S-E0441 → C-21, C-26, C-27, C-28; C-003/I-X-01 → C-3, C-7, C-10; C-004/X-013 → C-4, C-28, C-52, C-54; C-005/X-E04 → C-5, C-27, C-28, C-36

- (C-003/E-C29-022) Inline PlanWindow panel lists existing pairs on step selection; its three fields are NL intent, assertion, and auto-filled step ref.
- (C-003/E-C29-023) Add calls `add_intent_pair`; Confirm is enabled at count ≥3 and calls `confirm_intent_pairs`; both map `Result<(),String>` errors to toast.
- [gate] (C-003/DELTA!-C29-2) “Confirmation gate” is stronger than the original algorithm: the source explicitly gates on count, not on the persisted `confirmed` flag, and may emit confirmation from both gate and command.
- [gate] (C-003/E-C29-001) Add IntentPair formalization and identity/inverse/permutation metamorphic checks; require at least three pairs for `PLANNING -> TESTING_FIRST`; run metamorphic checks after mutation; add PlanWindow editing.
- (C-003/E-C29-002) `IntentPair` fields: `natural_language_intent:String`, `concrete_test_assertion:String`, `baby_step_ref:BabyStepId`.
- (C-003/E-C29-003) `IntentPairSet` fields: `pairs:Vec<IntentPair>`, `confirmed:bool`, `confirmed_at:Option<DateTime<Utc>>`; methods `len`, `is_confirmed`, and `confirm` (sets bool plus timestamp).
- (C-003/E-C29-004) `IntentConfirmation` fields: `step_id`, `pair_count:usize`, `confirmed_by:ConfirmedBy`, `timestamp:DateTime<Utc>`; `ConfirmedBy = User | AutonomousGate`.
- (C-003/E-C29-005) All intent types derive debug/clone/serde; no unwrap; `BabyStepSpec.intent_pairs` is `#[serde(default)]`, producing an empty unconfirmed set so old JSON loads without a BabyStepSpec migration.
- (C-003/E-C29-006) Bump the SSB constant from 46 to 47 with a C-29 comment, add payload structs/match arms, and preserve serde round trips.
- (C-003/E-C29-007) `IntentConfirmed` carries `step_id:u64`, `pair_count:u8`, and `confirmed_by:String`.
- (C-003/E-C29-008) `MrViolated` carries `step_id:u64`, `relation:String`, non-empty `counterexample:String`, and `sample_size:u32`.
- [config] (C-003/E-C29-009) Add `[testing.intent] min_pairs=3` and `IntentConfig {min_pairs:usize}` using the shared config hierarchy.
- [gate/failure] (C-003/E-C29-010) After existing gate checks, fewer than `min_pairs` emits `gate_decision` reason `intent_pairs_below_threshold` and returns `Reject`; every rejection must be observable.
- [gate] (C-003/E-C29-011) A threshold pass emits `IntentConfirmed`; autonomous flow auto-confirms qualifying pairs with `confirmed_by=AutonomousGate`.
- (C-003/E-C29-012) `MetamorphicRelation = Identity | Inverse | PermutationInvariance`.
- (C-003/E-C29-013) `MrResult` contains relation, pass bool, sample size, optional counterexample, and `duration_ms:u64`; `RelationLibrary` holds enabled relations.
- [failure/config] (C-003/E-C29-014) `RelationLibrary::from_config(&[String])` parses names, warns and skips unknown values without panic.
- [config] (C-003/E-C29-015) Default relation set is Identity+Inverse; permutation invariance is excluded/deny-by-default.
- (C-003/E-C29-016) `MetamorphicTestRunner` holds `RelationLibrary`, `SsbSender`, and `sample_size:u32`.
- (C-003/E-C29-017) `run_identity` calls the tested function twice on identical bytes; mismatch stores the hex input counterexample.
- (C-003/E-C29-018) `run_inverse` checks `decode(encode(x))==x` for each sample and records the first failing sample as counterexample.
- [config] (C-003/E-C29-019) `run_permutation_invariance` executes only when enabled; `run_all` executes all enabled relations, emits one `MrViolated` per failure, never panics, and returns the full result set.
- [config] (C-003/E-C29-020) Run metamorphic tests after C-28 mutation only when `[testing.metamorphic] enabled=true`; default config is enabled with `relations=["identity","inverse"]`.
- [gate] (C-003/E-C29-021) MR violations contribute to `AUDITING -> MEMORIZING` scoring alongside mutation score and are not independently blocking.
- (C-003/E-C29-024) `add_intent_pair(step_id,natural_language_intent,concrete_test_assertion)` appends a pair and errors if the step is absent.
- [gate] (C-003/E-C29-025) `confirm_intent_pairs(step_id)` calls `IntentPairSet::confirm`, errors below configured minimum (acceptance string: `need N more pairs`), and emits `IntentConfirmed`; register both commands in `generate_handler!`.
- (C-003/E-C29-026) Smoke begins with the below-threshold event, then editor-added/confirmed three pairs lead to approval plus `IntentConfirmed(pair_count=3)`.
- (C-003/E-C29-027) Correct encoder/decoder over 100 random strings returns inverse pass with no violation; off-by-one bug yields inverse violation and non-empty counterexample.
- (C-003/E-C29-028) Decision doc justifies minimum three, default exclusion of permutation for non-commutative false positives, and non-blocking standalone MR due to relation noise.
- [gate] (C-003/E-C29-029) Workspace check, orchestrator intent/metamorphic tests (at least eight new functions), contract intent tests, and both SSB serde round trips must pass.
- (C-003/E-C29-030) Inverse must return rather than panic; confirmation must error below threshold; gate rejects and MR failures must never be silent/empty; relation choice must come from config.
- (C-003/E-C29-031) Add `intent_pair.rs`, `metamorphic_testing.rs`, and decisions; modify research tree, SSB, gate, deterministic auditor, config, Tauri lib, and PlanWindow.
- [deferred/non-goal] (C-003/E-C29-032) Testing-trust tiers T-4 through T-13 (risk, equivalent cache, regression corpus, Kani, provenance, CiteAudit, PRPO, Rulebreakers, XGrammar/CRANE, FACET R-DAG) are future/post-launch work.
- (C-003/DELTA+-C29-1) Adds all exact field types, methods, event payloads, runner state/results, command signatures/errors, counterexample rules, decision rationale, and deferred tiers.
- (C-003/DELTA+-C29-3) Sample generation/seeding and MR+mutation aggregate scoring remain undefined; the summary should not imply a closed scoring or persistence design.
- (C-003/C-C29-01) `BabyStepSpec` compatibility says no schema bump is needed, while the same task explicitly bumps SSB schema 46→47; these are different schemas but the source uses “schema” without consistently qualifying the boundary (L54-L64).
- [gate] (C-003/A-C29-01) Gate enforcement checks only pair count and emits confirmation on pass; it never explicitly requires `IntentPairSet.confirmed=true`. The separate UI command also confirms/emits, so user confirmation persistence and duplicate-event behavior are unresolved (L71-L75, L102-L115).
- (C-003/A-C29-02) `add_intent_pair` says it appends “to SSB” yet acceptance says it mutates `BabyStepSpec.intent_pairs`; no pair-added event, load path, storage transaction, restart persistence, or concurrent-edit policy is defined (L102-L115, L248-L253).
- (C-003/A-C29-03) `pair_count` narrows from `usize` to SSB `u8` without maximum or overflow behavior (L49-L50, L61-L63).
- (C-003/A-C29-04) Identity means repeatability (`f(x)==f(x)`), not a stated identity transform; permutation input semantics, `run_all` function arguments, sample-size default, duration clock, and failure isolation are unspecified (L84-L91).
- (C-003/A-C29-05) “100 random strings” has no seed/generator/corpus pin, so deterministic replay and counterexample stability are not promised (L117-L123).
- (C-003/A-C29-06) The formula, weights, threshold, and missing-result behavior for combining MR outcomes with mutation score are absent (L93-L97, L125-L129).
- (C-003/A-C29-07) C-29 says current SSB is 46 and bumps it to 47; the orientation summary separately identifies a cross-plan collision with C-1/C-27, but this source contains no migration ordering that resolves it (L21-L25, L59-L64).
- (SUMMARY/S-E0441) Closes testing-trust T-2 and T-3; depends on C-27 and C-28; is parallel-safe with C-21 through C-26. [L380-L383]
- (SUMMARY/S-E0442) Keeps SCHEMA_VERSION 47 as the shared C-27/C-28 wave bump. [L384]
- (SUMMARY/S-E0443) Adds IntentPair fields natural_language_intent, concrete_test_assertion, and baby_step_ref. [L385]
- (SUMMARY/S-E0444) Adds IntentPairSet fields pairs, confirmed, and confirmed_at. [L385]
- (SUMMARY/S-E0445) Adds IntentConfirmation and ConfirmedBy values User and AutonomousGate. [L385]
- (SUMMARY/S-E0446) Adds a serde-defaulted intent_pairs field to BabyStepSpec without migration. [L386]
- (SUMMARY/S-E0447) Adds IntentConfirmed and MrViolated events and payloads. [L387]
- (SUMMARY/S-E0448) Defaults testing.intent min_pairs to three. [L388]
- (SUMMARY/S-E0449) Defaults metamorphic testing to enabled with identity and inverse relations. [L388]
- (SUMMARY/S-E0450) Excludes permutation_invariance by default. [L388]
- (SUMMARY/S-E0451) At PLANNING to TESTING_FIRST, fewer than the minimum pairs emits a gate_decision with reason intent_pairs_below_threshold and rejects the transition. [L389]
- (SUMMARY/S-E0452) Meeting the minimum emits IntentConfirmed. [L389]
- [gate] (SUMMARY/S-E0453) AutonomousGate auto-confirms qualifying pairs. [L389]
- (SUMMARY/S-E0454) Adds MetamorphicRelation values Identity, Inverse, and PermutationInvariance. [L390]
- (SUMMARY/S-E0455) RelationLibrary loads relations from config and warns while skipping unknown strings. [L390]
- (SUMMARY/S-E0456) MetamorphicTestRunner has run_identity, run_inverse, run_permutation_invariance, and run_all. [L390]
- (SUMMARY/S-E0457) Inverse testing checks decode(encode(x)) equals x for each sample and records the first failing counterexample. [L390]
- (SUMMARY/S-E0458) Runs metamorphic tests after mutation when enabled. [L391]
- (SUMMARY/S-E0459) MrViolated rows contribute to the AUDITING gate score. [L391]
- (SUMMARY/S-E0460) Adds add_intent_pair and confirm_intent_pairs Tauri commands. [L392]
- (SUMMARY/S-E0461) Adds a three-field IntentPairEditor to PlanWindow. [L393]
- (SUMMARY/S-E0462) Enables Confirm only at pair count three or greater. [L393]
- (SUMMARY/S-E0463) Smoke begins blocked, uses the editor to add three pairs, and expects IntentConfirmed; a correct encoder has no MrViolated, while a buggy encoder emits an inverse violation with a counterexample. [L394]

### C-30: Surgical decomposition, Oracle answer cache, DKG tools, LOC gates

**Status**: planned
**Anchors**: ledgers C-001, SUMMARY | sample anchors: C30:L185-212; C30:L11-45; C30:L62-70; C30:L66-71; C30:L73-78, L213-215; C30:L79-81 | 33 deep atoms, 28 summary atoms | IDs: C-001/C30-025, C-001/C30-002, C-001/C30-003, C-001/C30-004, C-001/C30-005, …
**Composes with**: C-1, C-2, C-6, C-17, C-18, C-22, C-24, C-26, C-29, C-33, C-41, C-51, C-56, C-58, C-59. IX: C-001/X-006 → C-1, C-17, C-24, C-56, C-58; C-001/C30-001 → C-17, C-24, C-26, C-29; C-006/C33-X003 → C-6, C-22, C-33; C-001/X-012 → C-1, C-2, C-6, C-18, C-22, C-41, C-51, C-56, C-58, C-59; SUMMARY/S-E0464 → C-17, C-24, C-26, C-29

- Superseded: (C-001/C30-010) surgical=true dispatches SurgicalDecomposer before research_decompose, replaces ConceptDecomposition with plan steps, emits surgical_plan_created with class/file count, and leaves surgical=false unchanged. (C30:L94-100)
- Superseded: (C-001/SUP-C30-029) NEWER-PLAN PRECEDENCE: C-15 description is target state relative to older C plan(s) C-0 for this conflict. Losing description retained at C-001/C30-029. Conflict: [E][conflict][Δ+] OracleAnswerCache explicitly uses std HashMap in production, while C-0 says C-15 bans production HashMap in favor of FxHashMap. This cross-plan type conflict must be resolved before implementation. (C30:L114-124; C0:L78-81)
- Superseded: (C-001/SUP-A-014) NEWER-PLAN PRECEDENCE: C-30 description is target state relative to older C plan(s) C-0, C-15 for this conflict. Losing description retained at C-001/A-014. Conflict: [conflict] C-30's std HashMap violates C-0/C-15's production FxHashMap policy. (C30:L114-124; C0:L78-81)
- (C-001/C30-025) WIRING_CONTRACT exposes seven surgical/error types, decomposer, three cache types, five events, four tools, and VRAM config from CLI task/research/tool entry points. (C30:L185-212)
- (C-001/C30-002) Verified missing surfaces are the surgical types/decomposer/factory dispatch, weighted first-task detection, final-answer—not decomposition—Oracle caching, four catalog tools, VRAM tiers, and interactive/autonomous blast gates. Existing hooks are parse_task_flags, run_research_pipeline, indexes_exist, and build_tool_catalog. (C30:L11-45)
- (C-001/C30-003) SurgicalPlan holds Vec<SurgicalStep>, AffectedSet, ComplexityClass, and RegressionGate. AffectedSet has direct/dependent PathBuf lists and total_loc_estimate; steps have description, target_file, optional target_fn. (C30:L62-70)
- [gate] (C-001/C30-004) RegressionGate contains must_pass, allow_new_failures, snapshot_hash; TestBaseline contains snapshot_hash and pre_existing_failures; ComplexityClass is Simple/Medium/Complex; all fallible paths return SurgicalError and never unwrap. (C30:L66-71)
- (C-001/C30-005) SurgicalDecomposer opens the project DKG, query_reachability traverses target blast radius, ReachabilityMatch paths populate AffectedSet, and stored node LOC metadata supplies the estimate. Synthetic production sets are banned. (C30:L73-78, L213-215)
- [config] (C-001/C30-006) Classification is Simple when files≤3 and LOC≤200; Medium when files≤8 or LOC≤500; Complex otherwise. (C30:L79-81)
- (C-001/C30-007) Generate one step per affected file in DKG topological order, capped at 1/4/8 for Simple/Medium/Complex. (C30:L82-84)
- (C-001/C30-008) Gather DKG test-coverage names and blake3-hash sorted must_pass for the RegressionGate. (C30:L85-87)
- (C-001/C30-009) Emit surgical_loc_warning above 500 LOC; interactive mode returns ConfirmationRequired before a plan and retries with confirmed=true after user consent; autonomous mode escalates above 1000 LOC. (C30:L88-92)
- (C-001/C30-010) surgical=true dispatches SurgicalDecomposer before research_decompose, replaces ConceptDecomposition with plan steps, emits surgical_plan_created with class/file count, and leaves surgical=false unchanged. (C30:L94-100)
- [config] (C-001/C30-011) When no index exists, weighted signals are Cargo.toml/go.mod/pyproject.toml=REQUIRED, ≥10 .rs files/package.json=STRONG. (C30:L102-106)
- [fallback] (C-001/C30-012) Any REQUIRED signal prompts “Detected new project. Build knowledge index? [Y/n]” interactively; confirmation or autonomous mode scans, rejection proceeds without an index and falls back to symbol_impact. (C30:L107-110)
- (C-001/C30-013) Every detection result emits project_detection_signal with signal_name, weight, and fired. (C30:L111-112)
- (C-001/C30-014) OracleAnswerCache is a bounded map from two 32-byte blake3 hashes (query and project) to CachedAnswer(answer, unix cached_at, source_tier). (C30:L114-124)
- [lifecycle] (C-001/C30-015) TTL is T1Authoritative 2,592,000s, T2Strong 604,800s, T3Weak 86,400s, T4Rejected zero/never cached. (C30:L119-120)
- [lifecycle] (C-001/C30-016) Cache API is get(key,now), insert, evict_expired(now), len; exceed max_entries triggers LRU eviction and adds no dependency beyond std+blake3. (C30:L121-124)
- (C-001/C30-017) /research hashes query text and canonical project-root path; a hit bypasses the pipeline and emits oracle_cache_hit with age/tier. (C30:L126-131)
- [fallback] (C-001/C30-018) A miss runs research normally; on Ok, HIGH/MEDIUM/LOW maps to T1/T2/T3, inserts, and emits oracle_cache_miss. Error and T4 answer mapping are not specified. (C30:L132-134)
- (C-001/C30-019) dkg_callers accepts symbol, follows inbound CALLS, and returns caller paths; dkg_dependents accepts symbol, follows outbound IMPORTS/USES, and returns dependent paths. (C30:L136-143)
- (C-001/C30-020) dkg_data_flow accepts symbol/max_depth, traverses PARAMETER/RETURN/ASSIGNMENT/FIELD_ACCESS, and returns source-target-type triples. (C30:L143-145)
- [failure] (C-001/C30-021) dkg_subgraph accepts root/depth, returns bounded-BFS compact nodes/edges JSON; every handler opens the session-root DKG and returns error JSON on QueryFailed without panic. (C30:L146-149)
- [config] (C-001/C30-022) research.vram_tiers mirrors six hardcoded labels—8gb, 12gb, 12gb_7b, 16gb, 24gb, cloud—with search/depth/breadth/token/speculation fields, although the check only requires ≥5. (C30:L151-156)
- (C-001/C30-023) TDD contract coverage includes constructibility, all three class bounds, deterministic gate hash, miss→hit, T1/T3 expiry, T4 rejection, max eviction, all four tool catalog entries, VRAM count, >500 warning, and Cargo REQUIRED detection. (C30:L158-183)
- [security] (C-001/C30-026) TTL, real DKG, real handler queries, and pre-plan interactive gate are stub bans; no hardcoded sets, unconditional cache hits, or panic returns. (C30:L213-219)
- (C-001/C30-027) Three smokes prove a ≤3-file surgical plan/regression gate, Oracle miss then hit within tier TTL, and a >500 interactive confirmation before plan return. (C30:L221-232)
- [gate] (C-001/C30-028) Acceptance requires real traversal, exact class/gate behavior, 500/1000 thresholds, false-path preservation, TTL/LRU, live two-call cache, four correct tool schemas, six-ish VRAM config, detection events, 17 tests, and workspace regression safety. (C30:L272-305)
- [gate] (C-001/C30-024) The source enumerates 17 tests despite saying “minimum 12” and expects 17 passed. The file is placed under crates/contracts, but the check invokes cargo test -p elai-orchestrator; acceptance also mislabels the regression-hash proof as an oracle_cache_* test. Test-package/filter ownership must be corrected. (C30:L158-183, L261-264, L279-280, L301, L316)
- (C-001/C30-029) OracleAnswerCache explicitly uses std HashMap in production, while C-0 says C-15 bans production HashMap in favor of FxHashMap. This cross-plan type conflict must be resolved before implementation. (C30:L114-124; C0:L78-81)
- (C-001/C30-030) LRU is underspecified: CachedAnswer has no access timestamp/order field, and project_hash is described as canonical-path hash rather than project-content hash, so access recency and invalidation after source changes are undefined. (C30:L114-134)
- [status] (C-001/C30-001) C-30 is a 2–3 session pipeline-mode plan after C-17/C-24, parallel with C-26..C-29; it adds surgical decomposition, answer caching, DKG tools, project detection, LOC gates, and VRAM config. (C30:L1-9)
- (SUMMARY/S-E0464) Closes pipeline-modes-design, update-implementation-order, and phase-4D; depends on C-17 and C-24; is parallel-safe with C-26 through C-29. [L396-L399]
- (SUMMARY/S-E0465) Adds SurgicalPlan, SurgicalStep, AffectedSet, ComplexityClass, RegressionGate, TestBaseline, and SurgicalError. [L400]
- (SUMMARY/S-E0466) ComplexityClass values are Simple, Medium, and Complex. [L400]
- (SUMMARY/S-E0467) SurgicalDecomposer uses real DKG blast-radius traversal via DkgQuerierAdapter::query_reachability. [L400]
- (SUMMARY/S-E0468) Simple means at most three files and at most 200 LOC. [L400]
- (SUMMARY/S-E0469) Medium means at most eight files or at most 500 LOC. [L400]
- (SUMMARY/S-E0470) All other work is Complex. [L400]
- (SUMMARY/S-E0471) Step caps are one for Simple, four for Medium, and eight for Complex. [L400]
- [gate] (SUMMARY/S-E0472) RegressionGate snapshot_hash is blake3 over sorted must_pass tests. [L400]
- (SUMMARY/S-E0473) More than 500 LOC emits surgical_loc_warning. [L400]
- (SUMMARY/S-E0474) In interactive mode, more than 500 LOC requires confirmation. [L400]
- (SUMMARY/S-E0475) In autonomous mode, more than 1,000 LOC escalates. [L400]
- (SUMMARY/S-E0476) --surgical dispatches SurgicalDecomposer before research_decompose. [L401]
- (SUMMARY/S-E0477) Cargo.toml is a REQUIRED Rust signal; at least ten .rs files is a STRONG signal. [L402]
- (SUMMARY/S-E0478) go.mod and pyproject.toml are REQUIRED signals for their ecosystems. [L402]
- (SUMMARY/S-E0479) package.json is a STRONG signal. [L402]
- (SUMMARY/S-E0480) The first interactive task presents a project-detection gate prompt. [L402]
- (SUMMARY/S-E0481) Emits project_detection_signal. [L402]
- (SUMMARY/S-E0482) Adds OracleAnswerCache keyed by blake3 query_hash and project_hash arrays of 32 bytes. [L403]
- (SUMMARY/S-E0483) CachedAnswer contains answer, cached_at, and source_tier. [L403]
- (SUMMARY/S-E0484) Tier-1 TTL is 2,592,000 seconds/30 days; Tier-2 is 604,800 seconds/7 days; Tier-3 is 86,400 seconds/1 day; Tier-4 is zero and never cached. [L403]
- (SUMMARY/S-E0485) Evicts least-recently-used entries at max_entries. [L403]
- (SUMMARY/S-E0486) Integrates OracleAnswerCache into the /research pipeline. [L404]
- (SUMMARY/S-E0487) Emits oracle_cache_hit with cached_age_secs and source_tier, or oracle_cache_miss. [L404]
- (SUMMARY/S-E0488) Adds dkg_callers for inbound CALLS, dkg_dependents for outbound IMPORTS/USES, dkg_data_flow for PARAMETER/RETURN/ASSIGNMENT/FIELD_ACCESS, and dkg_subgraph for bounded-depth BFS. [L405]
- (SUMMARY/S-E0489) Adds at least five research.vram_tiers config entries matching research_tier.rs: 8gb, 12gb, 12gb_7b, 16gb, 24gb, and cloud are named, creating a count ambiguity of six labels. [L406]
- (SUMMARY/S-E0490) Requires 17 contract tests. [L407]
- (SUMMARY/S-E0491) Smokes require a surgical plan and passing regression gate, an Oracle miss then hit, and agent self-selection of DKG tools. [L408]

### C-31: FilePlan / FILE_PLAN baby-step block and generation guards

**Status**: planned
**Anchors**: ledgers C-006, SUMMARY | sample anchors: C-31 L1-L7; C-31 L8-L11; C-31 L13-L25, L33-L43; C-31 L27-L31; C-31 L38-L46; C-31 L65-L67 | 40 deep atoms, 21 summary atoms | IDs: C-006/C31-E001, C-006/C31-E002, C-006/C31-E003, C-006/C31-E004, C-006/C31-E005, …
**Composes with**: C-5, C-6, C-9, C-13, C-26, C-32. IX: C-006/C31-X001 → C-6; C-006/C31-X003 → C-6; C-006/C31-X002 → C-6; C-006/C31-E002 → C-9, C-13, C-32; C-005/X-E05 → C-5, C-26, C-32; C-006/C31-I002 → C-6

- (C-006/C31-E001) Objective: add FilePlan/FileSpec/FileModSpec/FunctionSpec, a BabyStepSpec FILE_PLAN field, crash-persistent cumulative LOC with a generation guard, MistakeBook recording from T1/T2 BLOCKs, T2 function-count Check 3, and ProjectIntelligence-derived file limits. [C-31 L1-L7]
- (C-006/C31-E002) Estimated two to three sessions; depends on C-13 for stable mistake-book wiring and C-9 for the autonomous gate; declared parallel-safe with C-32 because file scope is disjoint. [C-31 L8-L11]
- (C-006/C31-E003) Baseline reports all Layer-1 plan/tracker/guard pieces absent, while FailureCategory::LazyCode, EmptyBody, and FileSizeExceeded and SSB-backed mistake storage already exist but are not called by T1/T2 BLOCK paths. The implementation-order report also says ModuleTree and tier-gated file-count limits are absent, but this plan assigns neither an implementation task. [C-31 L13-L25, L33-L43]
- (C-006/C31-E004) Source text says BabyStepSpec has eight current fields but explicitly enumerates nine—id, task, estimated_tokens, grounding, tdd, acceptance, preferred_mode, target_file, edit_region—before calling FILE_PLAN the ninth. [C-31 L27-L31]
- (C-006/C31-E005) Existing LanguageLimits defaults are Rust 400, Go 400, Python 480, TypeScript/JavaScript 600, and Java/C# 800; FileDisciplineConfig currently has only cyclomatic-complexity and coupling controls. [C-31 L38-L46]
- (C-006/C31-E006) FunctionSpec schema is {name: String, signature: String, max_body_lines: u8}. [C-31 L65-L67]
- (C-006/C31-E007) FileSpec schema is {path: PathBuf, language: String, max_lines: u16, functions: Vec<FunctionSpec>}. [C-31 L67-L68]
- (C-006/C31-E008) FileModSpec schema is {path: PathBuf, language: String, max_added_lines: u16, functions: Vec<FunctionSpec>}. [C-31 L69-L70]
- (C-006/C31-E009) FilePlan schema is {new_files: Vec<FileSpec>, modified_files: Vec<FileModSpec>, version: u8}. [C-31 L71-L72]
- (C-006/C31-E010) All four schemas derive Debug, Clone, Serialize, and Deserialize; fallible constructors return FilePlanError and production paths forbid unwrap. [C-31 L73-L76, L265-L266]
- (C-006/C31-E011) FilePlanError is a thiserror enum with LineBudgetExceeded {file, limit: u16, got: u16} and FunctionCountExceeded {file, limit: u8, got: u8}. [C-31 L74-L76]
- (C-006/C31-E012) FilePlan::validate checks new max_lines and modified max_added_lines against LanguageLimitsMap by language, falls back to global 400, and rejects more than MAX_FUNCTIONS_PER_FILE = 20 functions per file. [C-31 L89-L97]
- (C-006/C31-E013) FilePlan must survive a serde_json serialize/deserialize equality round trip. [C-31 L95-L97, L269]
- (C-006/C31-E014) BabyStepSpec appends #[serde(default)] file_plan: Option<FilePlan> after edit_region; old JSON remains readable without a schema bump, and both parse_research_tree construction sites explicitly set None. [C-31 L78-L87, L270-L273]
- (C-006/C31-E015) TreeNavigatorState adds cumulative_lines_by_step: HashMap<BabyStepId,u32>, records LOC at the record_result completion site, exposes total_lines_written as the sum, and serializes state for crash recovery. [C-31 L99-L110, L274-L276]
- (C-006/C31-E016) The TOCTOU contract adds monotonic step_write_generation: u64, increments before a write, checks that it is unchanged after, and returns NavigatorError::WriteGeneration on mismatch through record_lines_written. [C-31 L103-L109]
- (C-006/C31-E017) MistakeRecorder is a synchronous injectable trait with record_mistake_event(baby_step_id: &str, category: FailureCategory, file_path: &Path, snippet: &str); default behavior is no-op, while production task_runner wiring must call write_mistake_entry. [C-31 L112-L125, L206-L208]
- [config] (C-006/C31-E018) T1HookConfig and T2HookConfig receive Option<Arc<dyn MistakeRecorder + Send + Sync>>, default None. [C-31 L124-L125, L183-L185, L277-L278]
- (C-006/C31-E019) BLOCK recording maps lazy placeholders to LazyCode, no statements to EmptyBody, and oversize files to FileSizeExceeded; acceptance specifically requires T1 LazyCode, T1 FileSizeExceeded, and T2 EmptyBody calls. [C-31 L112-L123, L279-L282]
- [config] (C-006/C31-E020) T2HookConfig also receives file_plan: Option<Arc<FilePlan>>; Check 3 executes after Check 2 and before Check 4. [C-31 L127-L131]
- (C-006/C31-E021) Check 3 matches the current path to FileSpec, counts Rust item_function or Python function_definition AST nodes, and BLOCKs when actual count exceeds planned functions plus FUNCTION_COUNT_SLACK = 2. [C-31 L131-L139]
- (C-006/C31-E022) The exact BLOCK message is “function count {actual} exceeds FILE_PLAN limit {limit}+2 in {file}”; equality at plan+2 is allowed; no FilePlan silently skips the check. [C-31 L135-L139, L166-L168, L283-L284]
- [config] (C-006/C31-E023) FileDisciplineConfig adds language_limit_overrides: HashMap<String,LanguageLimits>; matching entries override hard-coded defaults. [C-31 L141-L145]
- (C-006/C31-E024) task_runner derives overrides only for languages with detected_file_count >=5: production_warn = min(spec_default, median LOC * 1.5 as u16), hard_block = min(spec_hard_block, p95 LOC * 1.2 as u16); missing ProjectProfile leaves defaults intact. [C-31 L145-L151, L285-L286]
- (C-006/C31-E025) c31_file_plan_contract.rs specifies sixteen named cases: type construction; JSON round trip; accept within limit; reject over limit; reject >20 functions; absent/populated FILE_PLAN round trips; legacy JSON; tracker total; generation mismatch; Check 3 excessive functions; no-plan skip; slack-two allow; LazyCode recorder; FileSizeExceeded recorder; language override precedence. [C-31 L153-L171]
- (C-006/C31-E026) Produced public surface also includes FilePlanError, FilePlan::validate, the two tracker methods, step_write_generation, MistakeRecorder, both hook-config fields, both named constants, and the override map. [C-31 L173-L188]
- (C-006/C31-E027) Consumed contracts are the three existing mistake categories, write_mistake_entry, BabyStepSpec, TreeNavigatorState, LanguageLimits, T1/T2 BLOCK paths, and ProjectProfile.language_stats. [C-31 L190-L197]
- (C-006/C31-E028) Reachability is LLM BabyStepSpec JSON, task_runner config construction, T1/T2 hot-path analysis, TreeNavigator::record_result, and a read-only PlanWindow FILE_PLAN block. [C-31 L199-L205, L228]
- (C-006/C31-E029) Stub bans require real SSB mistake persistence, AST-derived function counts, an enforced generation mismatch, and rejection rather than an always-pass FilePlan validator. [C-31 L206-L213]
- (C-006/C31-E030) Cloud CRUD smoke requires a populated plan for each step, cumulative line tracking, evidence for any lazy offense, and workspace compilation; an adversarial Rust plan above 400 LOC must fail before baby-step dispatch and emit no BabyStepSpec. [C-31 L215-L228, L287-L293]
- [gate] (C-006/C31-E031) Done gates are all sixteen contracts, an explicit oversize test, workspace cargo check, no new warnings, and the read-only PlanWindow surface. [C-31 L230-L261, L294-L295]
- (C-006/C31-E032) Planned artifacts are core/file_plan.rs, two orchestrator edits, tiered_analysis.rs, c31_file_plan_contract.rs, and C-31-decisions.md. [C-31 L297-L309]
- (C-006/C31-E033) All acceptance boxes remain open; the plan describes intended work, not verified delivery. [C-31 L263-L295]
- (C-006/C31-A001) “FileLineTracker” is named as a feature, but no FileLineTracker struct is defined; the fields/methods are placed directly on TreeNavigatorState. [C-31 L4-L6, L99-L110, L180-L182]
- (C-006/C31-A002) LanguageLimitsMap is defined as String -> u16, yet acceptance speaks of a language hard_block and the override path uses a richer LanguageLimits type; which threshold validate consumes is unclear. [C-31 L89-L95, L141-L151, L267-L268]
- (C-006/C31-A003) The logical generation guard holds &mut self during record_lines_written but does not define acquisition/return of a token or an async boundary; the described concurrent mismatch mechanism is incomplete. [C-31 L103-L109]
- (C-006/C31-A004) The observable count “>=0 MistakeBook entries” is vacuous and cannot prove wiring when no offense occurs. [C-31 L215-L222, L287-L290]
- (C-006/C31-A005) FunctionSpec.max_body_lines is modeled but no validator or runtime check for it is specified. [C-31 L65-L76, L89-L97]
- (C-006/C31-I001) FilePlan validation is a pre-dispatch admission-control layer; runtime tracker and T1/T2 checks are defense-in-depth after admission.
- (C-006/C31-I002) serde(default) intentionally favors backward read compatibility, but forward readers that do not know file_plan may drop it; no forward-compatibility policy is stated.
- (SUMMARY/S-E0492) Closes code-completeness-enforcement-design, update-implementation-order, and phase-4E; depends on C-13 and C-9; is parallel-safe with C-32. [L410-L413]
- (SUMMARY/S-E0493) Adds FunctionSpec. [L414]
- (SUMMARY/S-E0494) Adds FileSpec fields path, language, max_lines, and functions. [L414]
- (SUMMARY/S-E0495) Adds FileModSpec with max_added_lines. [L414]
- (SUMMARY/S-E0496) Adds FilePlan with new_files, modified_files, and version. [L414]
- (SUMMARY/S-E0497) FilePlan validates against LanguageLimitsMap. [L414]
- (SUMMARY/S-E0498) Adds LineBudgetExceeded and FunctionCountExceeded errors. [L414]
- (SUMMARY/S-E0499) Fixes MAX_FUNCTIONS_PER_FILE at 20. [L414]
- (SUMMARY/S-E0500) Adds FILE_PLAN as the ninth BabyStepSpec field using a serde default and no schema bump. [L415]
- (SUMMARY/S-E0501) Adds cumulative_lines_by_step to TreeNavigatorState through FileLineTracker. [L416]
- (SUMMARY/S-E0502) Adds a u64 step_write_generation as a TOCTOU guard. [L416]
- (SUMMARY/S-E0503) record_lines_written fails with NavigatorError::WriteGeneration on generation mismatch. [L416]
- (SUMMARY/S-E0504) Adds MistakeRecorder with a default no-op implementation. [L417]
- (SUMMARY/S-E0505) Wires MistakeRecorder into both T1HookConfig and T2HookConfig. [L417]
- (SUMMARY/S-E0506) T1/T2 blocks record LazyCode, EmptyBody, or FileSizeExceeded. [L417]
- (SUMMARY/S-E0507) Adds a T2 function-count guard that compares the per-file AST count to planned functions plus FUNCTION_COUNT_SLACK two. [L418]
- (SUMMARY/S-E0508) Exceeding that count blocks. [L418]
- [config] (SUMMARY/S-E0509) FileDisciplineConfig derives language overrides from ProjectProfile language statistics once at least five files of that language are detected. [L419]
- (SUMMARY/S-E0510) The shown production warning formula is the minimum of the spec default and project median times 1.5; other derived values are omitted from the summary. [L419]
- (SUMMARY/S-E0511) Requires 16 contract tests. [L420]
- (SUMMARY/S-E0512) CRUD smoke requires every baby step to have a populated file plan; lazy code should produce MistakeBook evidence; a plan above 400 LOC must be rejected before dispatch. [L421]

### C-32: Skeleton-first BLANK_N generation, MiniCritic, and completeness gauge

**Status**: planned
**Anchors**: ledgers C-005, SUMMARY | sample anchors: L149-L151, L306-L307; L154-L160; L162-L170, L251-L254; L319-L335; L162-L170, L332-L333; L56-L175 | 61 deep atoms, 34 summary atoms | IDs: C-005/C32-E26, C-005/C32-E27, C-005/C32-E28, C-005/C32-E36, C-005/C32-A08, …
**Composes with**: C-5, C-9, C-16, C-21, C-24, C-26, C-28, C-29, C-30, C-31, C-34, C-35, C-36, C-44, C-49. IX: C-005/C32-I02 → C-5; C-005/X-I05 → C-5, C-16, C-24, C-28, C-34, C-35, C-36, C-44, C-49; C-005/X-A05 → C-5, C-9, C-16, C-24, C-28, C-34, C-35, C-36, C-44, C-49; C-005/X-I01 → C-5, C-28, C-29, C-36; C-005/X-A01 → C-5, C-28, C-34, C-35, C-36, C-44, C-49; SUMMARY/S-E0513 → C-21, C-26, C-30, C-31

- Superseded: (C-005/C32-E06) `generate(FilePlan)` preserves signatures, replaces each body with `// BLANK_N`, and returns `TooManyBlanks` above the cap. [L67-L68]
- (C-005/C32-E26) `AcceptAndComplete` requires ratio at least 0.5 **and** a user opt-in flag. [L149-L151, L306-L307]
- (C-005/C32-E27) MonitorWindow calls `get_completeness_ratio() -> f32`, renders red below .5/yellow .5..<.9/green >=.9, and updates from `skeleton_flushed` without a new dependency. [L154-L160]
- (C-005/C32-E28) ReasoningDrawer renders blank/verdict/reason, color maps PASS/WARN/BLOCK to green/yellow/red, and has a default-on `Show MiniCritic` filter using the existing SSB stream. [L162-L170, L251-L254]
- (C-005/C32-E36) Implementation adds four orchestrator modules and one contract file, modifies task runner/SSB/Monitor/Tauri, locates and modifies ReasoningDrawer, and adds a decision artifact. [L319-L335]
- (C-005/C32-A08) Task 7’s “File” line names only Tauri `lib.rs` even though the task is a ReasoningDrawer renderer; the concrete drawer path is deferred to “locate” in the files list. [L162-L170, L332-L333]
- (C-005/C32-D01) Add every declared field/error/event payload, exact lazy patterns and reason strings, single-expression exemption, tree-sitter warning behavior, per-stage mechanics, retry context, per-kind bridge text, user opt-in, and the default-on drawer filter. [L56-L175]
- (C-005/C32-D03) Correct “14 contract tests” to “15 enumerated despite a stated/expected count of 14,” and flag the crate/path mismatch. [L177-L204, L291-L294]
- (C-005/C32-E01) This 2–3-session layer depends on C-31 FilePlan and C-26 decoding config, is parallel-safe with C-21..30, and adds skeleton-first generation, per-fill criticism, structural recovery, deterministic bridge files, two dashboard views, and five SSB types. [L1-L13]
- (C-005/C32-E02) `BlankId` wraps `u8` and displays `BLANK_{n}`. [L56-L59]
- (C-005/C32-E03) `SkeletonSpec` contains file path, language, blank IDs, and skeleton text; `FillResult` contains blank ID, body, and critic verdict; both are serializable. [L59-L62]
- (C-005/C32-E04) `FilledBuffer` stores the spec, fills, and `f32` completeness; ratio is non-BLOCK fills / blank count, clamped to `[0,1]`, with zero blanks equal to 1.0. [L63-L65]
- [config] (C-005/C32-E05) `SkeletonGeneratorConfig` defaults to `max_blanks=32` and `min_body_lines=3`. [L66]
- (C-005/C32-E06) `generate(FilePlan)` preserves signatures, replaces each body with `// BLANK_N`, and returns `TooManyBlanks` above the cap. [L67-L68]
- (C-005/C32-E07) `fill_blanks` holds all fills in memory, runs `MiniCritic::check` for each, returns successfully even with BLOCK verdicts, and performs no disk writes. [L69-L71]
- (C-005/C32-E08) `flush_to_disk` substitutes accepted bodies, leaves BLOCK entries as `// BLANK_N — not filled`, performs one `fs::write`, and returns the path. [L72-L74]
- (C-005/C32-E09) `SkeletonError` includes `TooManyBlanks`, `FlushFailed`, and `NoFillForBlank`; skeleton generation/flushing emits `skeleton_generated(blank_count,language)` and `skeleton_flushed(path,ratio)`. [L75-L78]
- (C-005/C32-E10) `MiniCriticVerdict` is `PASS | WARN | BLOCK`; `MiniCriticResult` carries blank ID, verdict, and reason; `MiniCritic` has `tree_sitter_enabled`. [L80-L87]
- (C-005/C32-E11) Critic order is lazy-pattern regex, minimum lines, then optional tree-sitter, with first BLOCK short-circuiting. [L87-L97]
- (C-005/C32-E12) Lazy matches (`todo!`, `unimplemented!`, `pass`, ellipsis, TODO/FIXME forms) block with `lazy_pattern:<pattern>` using shared Tier-1 regex statics. [L88-L91]
- (C-005/C32-E13) Fewer than three lines blocks as `too_short:<N>_lines`, except a trimmed body with no newline is treated as a single-expression exemption. [L92-L93]
- (C-005/C32-E14) Enabled tree-sitter wraps fragments in a language shell; an ERROR node blocks with `parse_error:<kind>`, while an unregistered language warns with `ts_unavailable`. [L94-L96]
- (C-005/C32-E15) Every critic call emits `mini_critic_verdict(blank_id,verdict,reason)`. [L97]
- (C-005/C32-E16) Structural stages are `Decompose`, `Template`, `FewShot`, and `BestOfN`; a plan stores stages/current index/blank, and `current_stage()` is optional without unwrap. [L100-L106]
- (C-005/C32-E17) Outcomes are `Converged { fill }`, `AcceptAndComplete { ratio }`, or `Exhausted`; config defaults `max_best_of_n=3`. [L107-L110]
- (C-005/C32-E18) Each retry stage runs MiniCritic; PASS converges. Decompose creates 2–3 substeps, Template uses `FixTemplateRegistry`, FewShot prepends `few_shot_for_symbol()`, and BestOfN generates N candidates. [L111-L116]
- (C-005/C32-E19) BestOfN scores compile +2, critic PASS +2, and lines +1, chooses the highest, converges when score > 0, otherwise accepts completeness at least 0.5 or exhausts. [L116-L118]
- (C-005/C32-E20) Every stage emits `structural_retry_stage_attempt`; no prompt may contain instructional “try again” text. [L119-L120]
- (C-005/C32-E21) `RetryContext` contains project root, language, function signature, and existing fills. [L121-L123]
- (C-005/C32-E22) Bridge kinds are Rust `mod.rs`, Python `__init__.py`, TypeScript index barrel, and Go package doc; a spec carries kind/path/module names and the template carries `ELAI_SEED` defaulting to 0. [L125-L132]
- (C-005/C32-E23) Bridge generation is LLM-free and deterministic: module names sort lexically, each language gets its declared line form, and output gets an auto-generated/do-not-edit header. [L132-L136]
- (C-005/C32-E24) `write_bridge` does generate + one write, returns `WriteFailed { path, source }` on I/O failure, and emits `bridge_file_generated(kind,path,module_count)`. [L137-L141]
- (C-005/C32-E25) For baby steps with FilePlan, task runner generates a skeleton before the first LLM call, makes one LLM call per blank, criticizes between fills, retries BLOCK, accepts converged output, and flushes only after all fills or opted-in partial acceptance. [L143-L152]
- (C-005/C32-E29) SSB registry adds `skeleton_generated`, `skeleton_flushed`, `mini_critic_verdict`, `structural_retry_stage_attempt`, and `bridge_file_generated`. [L171-L175]
- (C-005/C32-E30) The contract list verifies blank count/sequential IDs/all placeholders, exactly one flush write, all/partial/zero ratios, TODO and short blocks, multiline pass, retry order, deterministic/sorted/no-network bridge generation, and 0.5 versus 0.49 acceptance. [L177-L204]
- (C-005/C32-E31) Hard bans require a single disk write, no bridge LLM/network call, reuse rather than duplication of Tier-1 regexes, no acceptance below .5, and structural-only retry prompts. [L240-L249]
- (C-005/C32-E32) Main smoke at seed 42 requires `skeleton_generated` first, at least four verdict rows, flush ratio >= .9, no blank markers, and successful cargo check. [L256-L261]
- (C-005/C32-E33) Adversarial TODO fill must block and emit stage attempts before convergence or opted-in partial acceptance; bridge output must be byte-identical across two seed-42 runs. [L262-L265]
- (C-005/C32-E34) Deterministic inspection checks all public types, exactly one skeleton-file write, zero local regex construction, no bridge backend/network symbols, all SSB names, and the scoped contract test. [L267-L294]
- (C-005/C32-E35) Acceptance additionally requires no unwrap in production and a clean workspace check. [L296-L317]
- (C-005/C32-E37) Every acceptance box is unchecked; this is a planned completeness layer, not a shipped-status record. [L296-L317]
- (C-005/C32-D02) Add all 15 enumerated test cases and the unresolved context/config/tie-break/materialization issues. [L177-L204]
- (C-005/C32-A01) The critic API receives only blank/body/language, yet the single-line exemption depends on whether `x` is an “expression context”; the test explicitly expects body `x` to block in a non-expression context that the API cannot represent. [L87-L93, L190-L192]
- [config] (C-005/C32-A02) `min_body_lines` is configurable, but the critic rule is hard-coded as three and no config parameter appears in `MiniCritic::check`. [L66, L87-L93]
- (C-005/C32-A03) `NoFillForBlank` exists, but fewer/more fills, duplicate blank IDs, and substitution ordering are not defined; `fill_blanks` only says it returns OK despite BLOCK. [L69-L75]
- (C-005/C32-A04) BestOfN has no tie-breaker, definition of the binary/continuous “lines +1” score, candidate ordering, or deterministic generation rule. [L111-L118]
- (C-005/C32-A05) `AcceptAndComplete` is computed inside a per-blank retry sequencer, but ratio source/update from `existing_fills` and user-opt-in representation are not part of its declared config/context. [L107-L123, L143-L151]
- (C-005/C32-A06) Bridge template declares one `//` header for every kind, which is not a valid Python comment; per-kind header syntax is not reconciled. [L132-L136]
- (C-005/C32-A07) `ELAI_SEED` is stored but the described generators are pure sorting/string assembly; seed parsing, invalid values, and behavioral use are unspecified. [L131-L138]
- (C-005/C32-C01) The source says “minimum 14” tests but enumerates 15 named tests; deterministic check expects exactly 14. The orientation summary repeats 14. [L177-L204, L291-L294]
- (C-005/C32-C02) The contract file is placed under `crates/contracts/tests`, but the deterministic command runs it as an `elai-orchestrator` test target. [L177-L204, L291-L293]
- (C-005/C32-C03) General stage logic says only critic PASS converges, but BestOfN says any top candidate with score > 0 converges; compile/line points can make a BLOCK candidate positive. [L111-L118]
- [lifecycle] (C-005/C32-I01) The intended write lifecycle is transactional at file granularity: all generated/critic/retry state remains memory-only until one final materialization. [L69-L74, L143-L152, L240-L242]
- (C-005/C32-I02) WARN counts as complete because the ratio excludes only BLOCK verdicts. [L63-L65]
- (SUMMARY/S-E0538) MonitorWindow shows completeness ratio red below 0.5, yellow from 0.5 to below 0.9, and green at least 0.9. [L432]
- (SUMMARY/S-E0539) Adds get_completeness_ratio. [L432]
- (SUMMARY/S-E0540) ReasoningDrawer renders mini-critic rows and offers a Show MiniCritic toggle. [L432]
- (SUMMARY/S-E0513) Closes code-completeness-enforcement-design and token-efficiency-design; depends on C-31 and C-26; is parallel-safe with C-21 through C-30 due to disjoint file scope. [L423-L426]
- (SUMMARY/S-E0514) Adds BlankId represented by u8 and rendered as BLANK_N. [L427]
- (SUMMARY/S-E0515) Adds SkeletonSpec, FillResult, and FilledBuffer. [L427]
- (SUMMARY/S-E0516) FilledBuffer completeness is non-BLOCK fills divided by blank count and clamped. [L427]
- (SUMMARY/S-E0517) SkeletonGenerator preserves signatures while replacing function bodies with blank markers. [L427]
- (SUMMARY/S-E0518) fill_blanks runs MiniCritic on each fill and performs zero disk writes. [L427]
- (SUMMARY/S-E0519) flush_to_disk performs exactly one fs::write after substituting every blank marker. [L427]
- (SUMMARY/S-E0520) MiniCriticVerdict values are PASS, WARN, and BLOCK. [L428]
- (SUMMARY/S-E0521) The first BLOCK short-circuits later checks. [L428]
- (SUMMARY/S-E0522) Check one reuses the Tier-1 regex subset by borrowing LazyLock Regex statics from tiered_analysis.rs rather than duplicating patterns. [L428]
- (SUMMARY/S-E0523) Check two blocks fragments shorter than three lines with reason too_short. [L428]
- (SUMMARY/S-E0524) Check three parses the fragment with tree-sitter and blocks any ERROR node. [L428]
- (SUMMARY/S-E0525) Emits mini_critic_verdict for every call. [L428]
- (SUMMARY/S-E0526) Adds StructuralRetryStage values Decompose, Template, FewShot, and BestOfN. [L429]
- (SUMMARY/S-E0527) Adds StructuralRetryPlan and a Sequencer that runs those stages. [L429]
- (SUMMARY/S-E0528) PASS yields Converged. [L429]
- [fallback] (SUMMARY/S-E0529) Final fallback is AcceptAndComplete when completeness ratio is at least 0.5, otherwise Exhausted. [L429]
- (SUMMARY/S-E0530) BestOfN generates three candidates. [L429]
- (SUMMARY/S-E0531) Candidate scoring gives compile two points, critic PASS two points, and lines one point. [L429]
- (SUMMARY/S-E0532) Retry prompts may not contain instructional “try again” text. [L429]
- (SUMMARY/S-E0533) Emits structural_retry_stage_attempt for each stage. [L429]
- (SUMMARY/S-E0534) Adds deterministic BridgeKind values ModRsAggregator, InitPyBarrel, IndexTsBarrel, and GoPackageDoc. [L430]
- (SUMMARY/S-E0535) BridgeFileTemplate generation is LLM-free, lexically sorted, and byte-identical at ELAI_SEED 42. [L430]
- (SUMMARY/S-E0536) write_bridge uses one fs::write and emits bridge_file_generated. [L430]
- (SUMMARY/S-E0537) task_runner uses SkeletonGenerator and StructuralRetrySequencer for baby steps with a C-31 FilePlan. [L431]
- (SUMMARY/S-E0541) Registers skeleton_generated, skeleton_flushed, mini_critic_verdict, structural_retry_stage_attempt, and bridge_file_generated. [L433]
- (SUMMARY/S-E0542) Requires 14 contract tests. [L434]
- (SUMMARY/S-E0543) Main smoke requires at least 0.9 completeness, no residual blanks, and successful cargo check; adversarial TODO fill must block and enter structural retry, ending in convergence or AcceptAndComplete. [L435]

### C-33: Phase/spec docs, schema reconciliation, failure taxonomy, slash-command KEEP/STRIP

**Status**: planned
**Anchors**: ledgers C-006, SUMMARY | sample anchors: C-33 L121-L127; C-33 L1-L5; C-33 L6-L10; C-33 L12-L13, L367-L368; C-33 L15-L24; C-33 L69-L75 | 58 deep atoms, 18 summary atoms | IDs: C-006/C33-E023, C-006/C33-E001, C-006/C33-E002, C-006/C33-E003, C-006/C33-E004, …
**Composes with**: C-6, C-22, C-30. IX: C-006/C33-X003 → C-6, C-22, C-30; C-006/C33-X002 → C-6; C-006/C33-I002 → C-6; C-006/C33-E002 → C-30; SUMMARY/S-E0544 → C-22, C-30; C-006/C33-I001 → C-6

- Superseded: (C-006/C33-E042) B-0 replaces “SCHEMA_VERSION=26 confirmed” with current 46 and records B-6->26, B-24->30, B-34->35, and later B-38/B-42/B-41 progression through 44/45/46 with ssb.rs history as authority. [C-33 L232-L235]
- Superseded: (SUMMARY/S-E0559) Replaces the stale “SCHEMA_VERSION=26 confirmed” text with current 46 and records progression history. [L447]
- Superseded: (C-001/SUP-C0-061) NEWER-PLAN PRECEDENCE: C-33 description is target state relative to older C plan(s) C-1, C-5 for this conflict. Losing description retained at C-001/C0-061. Conflict: [E][conflict][Δ+] The statistics claim 59 slots: 52 CPU/cloud, six RunPod, one external; 59 C-N files authored; and all C-1..C-33 executable. Yet the Sessions section contains no C-5 row, so only 58 session bullets are present. C-5's features/status cannot be recovered from this source. (C0:L17-335,
- Superseded: (SUMMARY/SUP-S-59SLOTS) NEWER-PLAN PRECEDENCE: later coverage section (59 slots) supersedes header 35-session legend. Retained: SUMMARY/S-E0003 — The later coverage section supersedes the old count with 59 session slots: 52 CPU/cloud sessions (C-1 through C-33 and C-35 through C-53), six RunPod-dependent sessions (C-54 through C-59), and one externally blocked session (C-34). [L578-L587]
- (C-006/C33-E023) Surgical CLI/evidence is --surgical, surgical_plan_created {complexity_class, blast_radius_files}, surgical_loc_warning {loc_estimate, threshold}, and an rg check; target spec length 80-120. [C-33 L121-L127]
- (C-006/C33-E001) Objective: create four missing phase specs, append two shipped-extension addenda, add five B-plan status blocks, extend failure taxonomy by three variants, reconcile B-39/B-21, and replace a vague slash-command manifest entry with per-command decisions. [C-33 L1-L5]
- (C-006/C33-E002) Estimated one to two sessions; requires all other C-N files to exist and C-30 for the Surgical implementation reference; claimed parallel-safe with every C-N because failure_taxonomy.rs is the only code touch. [C-33 L6-L10]
- (C-006/C33-E003) docs/specs is gitignored: all four new specs and both edits are local-only, uncommitted, and must be called out in the session handoff. [C-33 L12-L13, L367-L368]
- (C-006/C33-E004) Baseline discrepancies are four missing spec files, two specs missing already-shipped behavior, absent B-37/B-38/B-40/B-41/B-42 index blocks, and a stale “SCHEMA_VERSION 26” against code version 46. [C-33 L15-L24]
- (C-006/C33-E005) phase-3L Step 1 defines ConventionStats/NamingConventions, public extract_conventions, four SQL pattern queries, and module-init regex compilation. [C-33 L69-L75]
- (C-006/C33-E006) phase-3L Step 2 puts conventions in DKG node metadata and migrates an existing conventions.db schema. [C-33 L75-L77]
- (C-006/C33-E007) phase-3L Step 3 tests Rust, Python, TypeScript, and Svelte extraction and deterministic output for identical input. [C-33 L77-L78]
- (C-006/C33-E008) phase-3L Step 4 has T1Hook load rules.db conventions and inject a CONVENTIONS block into the prompt SYSTEM section through a ConventionStats summary. [C-33 L79-L80]
- (C-006/C33-E009) phase-3L Step 5 adds T2 AST import extraction, ProjectRuleSeeder-generated rules.db rules, and VERSION_GATE. [C-33 L81-L82]
- [gate] (C-006/C33-E010) phase-3L Step 6 aggregates ConventionStats, FreshnessConfig, and ProjectClass into ProjectProfile stored in knowledge/project_profile.rs. [C-33 L83-L84]
- (C-006/C33-E011) phase-3L Step 7 integrates the context telescope and orders convention signals by token cost versus recall, with pi_telescope_contract and pi_tier3. [C-33 L85-L86]
- (C-006/C33-E012) phase-3L Step 8 calibrates convention extraction to p50/p99 <=50ms and writes metrics/phase_3L.toml; target spec length is 80-120 lines. [C-33 L87-L89]
- (C-006/C33-E013) phase-3J-BASELINE specifies standalone benches/model_capability/run_benchmark.py with a fixture TOML containing at least ten prompts per ModelCapabilityProfile tier. [C-33 L91-L96]
- (C-006/C33-E014) Each baseline fixture records total_tokens_in, total_tokens_out, slot_utilization_pct, ShortCoder compression_ratio, ttft_ms, p99_latency_ms, and cost_per_run_usd. [C-33 L96-L98]
- (C-006/C33-E015) The runner reads [model_capability] from elai_cc.toml and emits metrics/capability_profile.toml. [C-33 L99-L100]
- (C-006/C33-E016) Baseline floors are micro_7b_12gb slot utilization 0.82 and compression 0.65, and cloud_200k slot utilization 0.90. [C-33 L101-L102]
- (C-006/C33-E017) REPAIR_CONTEXT may not increase total_tokens_in by more than 15% over the same-fixture non-repair baseline. [C-33 L103-L104]
- (C-006/C33-E018) A 20-prompt JSON-schema constrained-decoding fixture must yield zero invalid JSON outputs. [C-33 L105-L106]
- (C-006/C33-E019) The benchmark runs without elai runtime, SSB, or DKG and writes TOML only; target spec length is 60-90 lines. [C-33 L107-L109]
- (C-006/C33-E020) phase-4D-SURGICAL records SurgicalPlan, AffectedSet, ComplexityClass, RegressionGate, TestBaseline, and SurgicalStep from C-30. [C-33 L111-L115]
- (C-006/C33-E021) SurgicalDecomposer traverses DKG blast radius and classifies Simple as files<=3 AND LOC<=200, Medium as files<=8 OR LOC<=500, otherwise Complex; step caps are 1/4/8 respectively. [C-33 L116-L120]
- (C-006/C33-E022) Each surgical step receives RegressionGate from DKG coverage; >500 LOC emits surgical_loc_warning and requires interactive confirmation, while autonomous >1000 LOC escalates. [C-33 L120-L125]
- (C-006/C33-E024) phase-4D-ORACLE records /research -> run_research_pipeline and CacheKey {query_hash:[u8;32], project_hash:[u8;32]} using blake3 plus CachedAnswer {answer,cached_at,source_tier}. [C-33 L129-L135]
- (C-006/C33-E025) Oracle TTLs are T1Authoritative 2,592,000s, T2Strong 604,800s, T3Weak 86,400s, and T4Rejected zero/never cache. [C-33 L135-L136]
- (C-006/C33-E026) Cache miss runs research, maps HIGH/MEDIUM/LOW to T1/T2/T3, inserts, and emits oracle_cache_miss; cache hit returns the answer and emits oracle_cache_hit {cached_age_secs,source_tier}. [C-33 L137-L140]
- [config] (C-006/C33-E027) Oracle eviction is LRU above configurable max_entries; required action types are oracle_cache_hit/miss, its check greps OracleAnswerCache, and target spec length is 70-100. [C-33 L141-L144]
- (C-006/C33-E028) The append-only 3J addendum documents ETCLogitsProcessor entropy stopping/calibration; GG mean_logprob and entropy_mean consumed by research confidence; ConstrainedDecodingConfig grammar/JSON modes; routing_rules; bash safety_filter; git_commit; action_classifier; and steering-vector --batch plus contrast-pair data. [C-33 L146-L161]
- (C-006/C33-E029) The append-only 4G addendum makes derive_framing_evidence the seventh AutoAdvanceEvaluator and declares exhaustion at evidence_rounds >= evidence_rounds_max. [C-33 L163-L169]
- (C-006/C33-E030) CloudSpendTracker emits cloud_spend_warning at 50% spent, cloud_spend_critical at 20% remaining, and below 20% escalates INTERACTIVE or halts AUTONOMOUS, using [cloud.session_budget_usd]. [C-33 L169-L174]
- [failure] (C-006/C33-E031) tools/failure_taxonomy.rs adds LazyCode, EmptyBody, and FileSizeExceeded after RuntimePanic, making 12 variants, and adds complete detect_category/suggest_fix arms. [C-33 L176-L188]
- (C-006/C33-E032) Detection patterns are TODO|placeholder|unimplemented!|todo!() for LazyCode, “empty function body|no statements” for EmptyBody, and “file exceeds|FILE_SIZE” for FileSizeExceeded. [C-33 L183-L187]
- [failure] (C-006/C33-E033) ToolFailureCategory in elai_core must receive the same variants, but only after locating it through elai.map. [C-33 L189-L190, L284-L288]
- (C-006/C33-E034) B-39 is corrected from cap two to REFINE_CAP = 3, citing RefineBench arXiv:2511.22173 Table 3's reported +12-20pp from t=2 to t=3 and the existing equality contract. [C-33 L26-L29, L192-L198]
- (C-006/C33-E035) B-21 action reconciliation maps feasibility_probe_started -> system_probe, feasibility_probe_complete -> feasibility_report, alternative_explored -> stub_run as nearest shipped equivalent, and marks feasibility_escalated absent/deferred to C-22. [C-33 L31-L36, L200-L210]
- (C-006/C33-E036) The B-21 divergence note also says novel_step_detected matches the plan and must be retained in the accepted action list. [C-33 L208-L210, L361-L362]
- (C-006/C33-E037) B-37's new index block is DONE at fd5c2e5 with RuntimeAdapter, nine adapters, PerRoleRuntime, RuntimeOrchestrator, /runtime, RuntimePanel, and final schema 46. [C-33 L212-L218]
- (C-006/C33-E038) B-38's block cannot be fixed from this plan alone: consult MEMORY.md/auditor and choose SHIPPED or PARTIAL. [C-33 L219-L221]
- (C-006/C33-E039) B-40's block is DONE with idea_framing.rs 1358 LOC, Framing substate, five actions, IdeaFramingWindow, 51 tests, schema 40->41, and code citation arXiv:2409.04109; its original plan retains wrong arXiv:2407.21787 as debt. [C-33 L38-L40, L222-L225]
- (C-006/C33-E040) B-41's block is DONE with auditor_mode, per_level_audit_dispatch, AgentRole+5, Evidence+5, AuditTimelineWindow, schema 45, 80 green tests, and navigator-spawn deferred. [C-33 L226-L228]
- (C-006/C33-E041) B-42's block is DONE with about 700 LOC CosmosWorker, briefing compiler, CosmosCheck, schema 44, CosmosDrawer, and spawn_loop dispatcher deferred. [C-33 L229-L231]
- (C-006/C33-E042) B-0 replaces “SCHEMA_VERSION=26 confirmed” with current 46 and records B-6->26, B-24->30, B-34->35, and later B-38/B-42/B-41 progression through 44/45/46 with ssb.rs history as authority. [C-33 L232-L235]
- (C-006/C33-E043) Migration manifest explicitly classifies KEEP: /task, /research, /mode, /model, /runtime, /reason, /plan, /memory, /context, /search, /audit, /help, /status, /log, /export, /import, /reset, /history, /session, /config, /budget, /branch, /checkpoint, /supervise, /agents, /tools, /version, /health. [C-33 L237-L250]
- (C-006/C33-E044) It classifies STRIP: /debug, /trace, /benchmark, /mock; /resume and /cancel are KEEP-pending until live_cli dispatch is verified; no command may be invented. [C-33 L251-L256]
- (C-006/C33-E045) Reachability/done evidence is spec_link_check on new/edited specs, wiring_contract_check on B-21/B-39, index greps, taxonomy compile, and manifest comparison to commands.rs. [C-33 L258-L347]
- [non-goal] (C-006/C33-E046) Non-goals forbid edits to phase specs 1A, 1E, 3A, 3C, 3E, 3F, 4B, 4C, 4E, and 4H. [C-33 L284-L287]
- (C-006/C33-E047) The migration task demands the entire verified command list, although acceptance weakens the machine gate to at least ten named entries. [C-33 L237-L256, L365-L366]
- (C-006/C33-E048) All acceptance boxes remain open; the optional cloud /research discovery check is non-blocking, and local-only file delivery must be flagged. [C-33 L349-L370]
- (C-006/C33-E049) Planned writes comprise six local-only spec files/edits, taxonomy code, three B-plan docs, B-0_INDEX, migration_manifest, and C-33-decisions.md. [C-33 L372-L387]
- (C-006/C33-E050) After all C-N work, the documented terminal FSM path is MEMORIZING -> COMPLETE -> IDLE. [C-33 L389-L392]
- [status] (C-006/C33-A001) B-38 status is deliberately unresolved, so “add five status blocks” cannot be deterministic until MEMORY/auditor evidence is available. [C-33 L219-L221]
- (C-006/C33-A002) The exact schema-version progression phrase “B-38/42/41 to 44/45/46” does not uniquely map every plan to every intermediate version and omits the final B-37C bump mechanics. [C-33 L232-L235]
- (C-006/C33-A003) alternative_explored -> stub_run is explicitly only the “nearest shipped equivalent”; semantic equivalence is not proven. [C-33 L200-L209]
- (C-006/C33-A004) The command source comment says slash_command_specs “assert 149,” but the enumerated manifest candidates total 34, and the plan does not explain what 149 counts. [C-33 L237-L256]
- (C-006/C33-I001) C-33 is a documentation/schema-reconciliation release gate: most behavior is already implemented elsewhere, while this plan makes specifications and evidence discoverable.
- (C-006/C33-I002) Because specs are gitignored, durable team delivery requires an out-of-band handoff or a repository policy change; merely creating them cannot preserve them in normal commits.
- (SUMMARY/S-E0544) Closes update-phase-map, B-0_INDEX, B-39, B-40, and B-21; depends on C-30 and C-22; is otherwise parallel-safe with all plans because only failure_taxonomy.rs changes code. [L437-L440]
- (SUMMARY/S-E0545) Warns that docs/specs is gitignored and all new step files are local-only; this must be flagged at session end. [L441]
- (SUMMARY/S-E0546) Creates phase-3L.txt as an eight-step Project Intelligence specification. [L442]
- (SUMMARY/S-E0547) Creates phase-3J-BASELINE.txt as a token-efficiency benchmark harness. [L442]
- (SUMMARY/S-E0548) Creates phase-4D-SURGICAL.txt for SurgicalDecomposer, cross-referencing C-30. [L442]
- (SUMMARY/S-E0549) Creates phase-4D-ORACLE.txt for OracleAnswerCache. [L442]
- (SUMMARY/S-E0550) Adds to phase-3J: ETCLogitsProcessor; GG mean_logprob and entropy_mean signals; constrained decoding; tool routing A-26; bash safety A-27; git_commit A-28; action classifier A-31; and steering-vector batch A-8. [L443]
- (SUMMARY/S-E0551) Adds to phase-4G: AutoAdvanceEvaluator handling for reasoning exhaustion and CloudSpendTracker escalation. [L443]
- (SUMMARY/S-E0552) Adds LazyCode, EmptyBody, and FileSizeExceeded to FailureCategory. [L444]
- (SUMMARY/S-E0553) Adds corresponding detect_category and suggest_fix arms. [L444]
- [failure] (SUMMARY/S-E0554) Keeps ToolFailureCategory synchronized after first checking elai.map. [L444]
- (SUMMARY/S-E0555) Changes B-39 REFINE_CAP from two to three based on the cited RefineBench table and adds a contract assertion for three. [L445]
- (SUMMARY/S-E0556) Reconciles B-21 event names to system_probe, feasibility_report, novel_step_detected, and stub_run. [L446]
- [deferred] (SUMMARY/S-E0557) Marks feasibility_escalated deferred in B-21 until C-22. [L446]
- [status] (SUMMARY/S-E0558) Updates B-0_INDEX status blocks for B-37, B-38, B-40, B-41, and B-42. [L447]
- (SUMMARY/S-E0559) Replaces the stale “SCHEMA_VERSION=26 confirmed” text with current 46 and records progression history. [L447]
- (SUMMARY/S-E0560) Adds at least ten per-command KEEP, STRIP, or DEFERRED records under migration_manifest.toml file.commands after reading commands.rs to avoid fabricated names. [L448]

### C-34: Empire/FactoryLine external workspace bridge (deferred)

**Status**: planned (externally DEFERRED pending Empire access)
**Anchors**: ledgers C-005, SUMMARY | sample anchors: L77-L78; L113-L117; L119-L139; L141-L158; L160-L175; L42-L45, L77-L78 | 41 deep atoms, 16 summary atoms | IDs: C-005/C34-E13, C-005/C34-E17, C-005/C34-E18, C-005/C34-E19, C-005/C34-E20, …
**Composes with**: C-5, C-9, C-16, C-24, C-28, C-32, C-35, C-36, C-44, C-49. IX: C-005/C34-I01 → C-5; C-005/X-A01 → C-5, C-28, C-32, C-35, C-36, C-44, C-49; C-005/X-I05 → C-5, C-16, C-24, C-28, C-32, C-35, C-36, C-44, C-49; C-005/X-A05 → C-5, C-9, C-16, C-24, C-28, C-32, C-35, C-36, C-44, C-49; C-005/X-E06 → C-5; C-005/X-A04 → C-5, C-35, C-44, C-49

- Superseded: (SUMMARY/SUP-S-59SLOTS) NEWER-PLAN PRECEDENCE: later coverage section (59 slots) supersedes header 35-session legend. Retained: SUMMARY/S-E0003 — The later coverage section supersedes the old count with 59 session slots: 52 CPU/cloud sessions (C-1 through C-33 and C-35 through C-53), six RunPod-dependent sessions (C-54 through C-59), and one externally blocked session (C-34). [L578-L587]
- (C-005/C34-E13) CLI adds `--empire`, which activates the Empire path and logs connection status at startup. [L77-L78]
- (C-005/C34-E17) Observable smoke expects connected and capability-invoked rows with non-null identifiers and a green panel displaying the capability. [L113-L117]
- [deferred] (C-005/C34-E18) Deferred deterministic checks require the feature declaration, all seven types, all five events, Empire package tests, CLI flag, and panel file. [L119-L139]
- [deferred] (C-005/C34-E19) Deferred acceptance requires conditional-only compilation, no placeholder types, preserved IPC tests, independent pathology handling, all events and migration, TDD-before-wiring evidence, contract success, reachable/validated MCP, graceful missing-pipe error, live panel, and cloud smoke. [L141-L158]
- (C-005/C34-E20) Planned files include six Empire modules, shared IPC/task-runner/CLI/workspace/config edits, panel, orchestrator contract test, and the source-grounding artifact; several paths remain explicitly TBD. [L160-L175]
- (C-005/C34-A01) Cargo feature activation is compile-time, while `--empire` is described as enabling the feature path at runtime; the build/distribution mechanism that makes the code available to the flag is not specified. [L42-L45, L77-L78]
- (C-005/C34-D02) Add the compile-time/runtime flag ambiguity, event payload mismatch, possible repair-event feedback loop, contract path conflict, and absence of endpoint auth/retry/privacy contracts. [L42-L86]
- (C-005/C34-E01) The entire plan is externally blocked: do not execute until an upstream maintainer grants Empire source access and `[empire.codebase_path]` resolves to a readable local clone; estimated work after unblock is three sessions. [L1-L8]
- (C-005/C34-E02) It is orthogonal to and non-blocking for every other C-plan. [L7-L8]
- (C-005/C34-E03) Intended architecture is a named-pipe `FactoryLineTransport`, an `ElaiNeuron` MCP endpoint, a registered `BrainManifest`, and a three-pathology repair pipeline, isolated in a feature-gated `crates/empire` member. [L12-L18]
- (C-005/C34-E04) Before type definitions, a source-mode paper ingestor must ground the actual Empire transport/manifest interfaces in `workflow/papers/empire-protocol-interface.md`, with reliability tied to source quality. [L24-L36]
- [config] (C-005/C34-E05) `[empire]` config declares `codebase_path`, `pipe_name`, `connect_timeout_ms`, and `brain_id`; startup validates it only for an enabled Empire build and otherwise warns/skips. [L38-L40]
- (C-005/C34-E06) Planned crate types are `FactoryLineTransport`, `FactoryLineClient`, `BrainManifest`, `EmpireEventBridge`, `ElaiNeuron`, `EmpireIssue`, and `FactoryCmd`; default builds compile no Empire code. [L42-L45]
- (C-005/C34-E07) Shared IPC adds an Empire protocol variant to `IpcMessageType` non-destructively, using a non-exhaustive extension or nested variant to preserve forward compatibility and existing sidecar/API IPC. [L47-L50]
- (C-005/C34-E08) Repair types are `CircularDep`, `HardImport`, `StdlibShadow`, `RepairStrategy`, and ordered `RepairPipeline`; each pathology has an independent pass and per-step SSB emission. [L52-L54]
- (C-005/C34-E09) Five SSB strings are `empire_connected`, `empire_disconnected`, `empire_capability_invoked`, `empire_repair_started`, and `empire_repair_complete`; each row is declared to carry `brain_id` and `capability_name`, and a schema-version bump is mandatory. [L56-L59]
- (C-005/C34-E10) `ElaiNeuron` is an HTTP capability-invocation handler that registers the manifest on connect, runs in its own Tokio task, and shuts down on an `empire_disconnected` SSB row. [L61-L63]
- (C-005/C34-E11) TDD precedes production: mock transport round trip, manifest-schema serialization, per-pathology repair events, MCP response shape, and lossless event forwarding are steps 4J.3–4J.7. [L65-L71]
- (C-005/C34-E12) Production step 4J.8 wires the neuron into feature-gated startup, binds the event bridge to the SSB bus, and connects repair execution to `empire_repair_started`. [L73-L75]
- (C-005/C34-E14) `EmpireBridgePanel.svelte` shows red/green connection state, active brain ID, last capability, and repair history from the existing SSB stream. [L80-L82]
- (C-005/C34-E15) Contract tests cover transport, bridge, and repair interface shapes and enforce the wiring-contract stub bans. [L84-L86]
- (C-005/C34-E16) Production transport cannot be a no-op, repair errors cannot be silently swallowed and must emit rows, and `ElaiNeuron` must validate the manifest schema before registration. [L108-L112]
- (C-005/C34-E21) B-25 and Phase-4J verification reports remain incomplete until a full audit checks every acceptance item. [L177-L183]
- (C-005/C34-E22) After unblock and implementation, the lifecycle is full auditor + wiring validator, memory writer, then `MEMORIZING -> COMPLETE -> IDLE`. [L185-L188]
- (C-005/C34-D01) Add the exact source-grounding reliability gate, config validation behavior, forward-compatible IPC alternatives, event payload requirement, Tokio shutdown trigger, all five TDD cases, stub bans, audit/report lifecycle, and post-audit FSM transitions. [L24-L188]
- [deferred] (C-005/C34-D03) Keep the orientation summary’s deferred status, but qualify “feature-gated workspace member” and `--empire` wiring as unresolved build mechanics rather than completed design. [L42-L45, L77-L78]
- (C-005/C34-A02) A workspace member itself is not normally conditional; “feature-gated workspace member” and “no empire code compiled in default build” need an exact Cargo membership/default-members design. [L42-L45, L143-L145]
- (C-005/C34-A03) Requiring `capability_name` on **every** event leaves its value for connect/disconnect and generic repair events undefined. [L56-L59, L147-L149]
- (C-005/C34-A04) Wiring repair execution to `empire_repair_started` while the repair pipeline itself emits per-step/start events risks a feedback cycle unless producer/consumer ownership is distinguished. [L52-L54, L73-L75]
- (C-005/C34-A05) `ElaiNeuron` authentication, authorization, request schema, transport binding, privacy handling, retry/backoff, and protection of `brain_id`/capability inputs are deferred to source grounding and absent here. [L24-L36, L61-L63]
- (C-005/C34-A06) Shutdown depends on observing a disconnect row, but the producer/order of that row relative to transport failure and event-bridge shutdown is unspecified. [L56-L63]
- (C-005/C34-C01) Task 12 names `tests/contracts/empire_contract.rs`, while the planned file list names `.elai_cc/crates/orchestrator/tests/empire_contract.rs`. [L84-L86, L172-L173]
- (C-005/C34-A07) Named-pipe platform coverage, reconnect policy, timeouts beyond initial connect, repair rollback, and at-least/exactly-once event delivery are not defined. [L12-L18, L38-L75]
- (C-005/C34-I01) The plan treats source grounding as a hard protocol-compatibility/security gate: implementation details are intentionally provisional until actual upstream interfaces are available. [L20-L36, L109-L112]
- (SUMMARY/S-E0561) Closes B-25 and phase-4J only when implemented. [L450-L451]
- (SUMMARY/S-E0562) Is blocked on Empire source-code access from an upstream maintainer and confirmation of elai_cc.toml empire.codebase_path. [L452]
- (SUMMARY/S-E0563) Is orthogonal to every other C plan and blocks no other delivery. [L453]
- (SUMMARY/S-E0564) Must not be executed until the external prerequisite is met. [L454]
- (SUMMARY/S-E0565) Estimated implementation effort after unblocking is three sessions. [L465]
- (SUMMARY/S-E0566) Runs a source-grounding pass in paper-ingestor source mode and writes workflow/papers/empire-protocol-interface.md. [L455]
- [config] (SUMMARY/S-E0567) Adds empire configuration fields codebase_path, pipe_name, connect_timeout_ms, and brain_id. [L456]
- [gate] (SUMMARY/S-E0568) Adds a feature-gated crates/empire workspace member. [L457]
- (SUMMARY/S-E0569) Planned types are FactoryLineTransport, FactoryLineClient, BrainManifest, EmpireEventBridge, ElaiNeuron, EmpireIssue, and FactoryCmd. [L457]
- (SUMMARY/S-E0570) Adds IpcMessageType::Empire as a non-destructive extension. [L458]
- (SUMMARY/S-E0571) Adds repair handling for CircularDep, HardImport, and StdlibShadow via RepairStrategy and RepairPipeline. [L459]
- (SUMMARY/S-E0572) Adds empire_connected, empire_disconnected, empire_capability_invoked, empire_repair_started, and empire_repair_complete, requiring a schema bump. [L460]
- (SUMMARY/S-E0573) Exposes an ElaiNeuron MCP HTTP endpoint that registers BrainManifest. [L461]
- (SUMMARY/S-E0574) Requires TDD tests 4J.3 through 4J.7 before production wiring step 4J.8. [L462]
- (SUMMARY/S-E0575) Adds --empire. [L463]
- (SUMMARY/S-E0576) Creates EmpireBridgePanel.svelte with connection status, brain_id, last capability, and repair history. [L464]

### C-35: MCP tool visibility gating and lazy schema resolution

**Status**: planned
**Anchors**: ledgers C-005, SUMMARY | sample anchors: L199-L205; L210-L214; L120-L122, L212-L214, L237-L238; L602; L1-L12; L14-L22 | 53 deep atoms, 14 summary atoms | IDs: C-005/C35-E19, C-005/C35-E22, C-005/C35-A06, C-005/C35-E01, C-005/C35-E02, …
**Composes with**: C-5, C-9, C-16, C-24, C-28, C-32, C-34, C-36, C-37, C-38, C-39, C-40, C-42, C-44, C-49, C-54. IX: C-005/C35-I01 → C-5; C-005/C35-E01 → C-36, C-42; SUMMARY/S-E0581 → C-36, C-40, C-42; C-005/C35-D05 → C-36, C-37, C-38, C-39, C-40; C-005/X-I05 → C-5, C-16, C-24, C-28, C-32, C-34, C-36, C-44, C-49; C-005/X-I03 → C-5, C-49

- Superseded: (C-005/C35-E09) `server.rs` replaces full-catalog rendering with state-gated rendering and resolves a referenced gated tool through `resolve_schema_on_demand(tool_name)`. [L89-L92]
- Superseded: (C-005/SUP-C35-C04) NEWER-PLAN PRECEDENCE: C-40 description is target state relative to older C plan(s) C-36, C-39 for this conflict. Losing description retained at C-005/C35-C04. Conflict: The original declares parallel safety only with C-36 through C-39, while the orientation summary extends that claim through C-40 without source support. [L9-L12]
- Superseded: (SUMMARY/SUP-S-59SLOTS) NEWER-PLAN PRECEDENCE: later coverage section (59 slots) supersedes header 35-session legend. Retained: SUMMARY/S-E0003 — The later coverage section supersedes the old count with 59 session slots: 52 CPU/cloud sessions (C-1 through C-33 and C-35 through C-53), six RunPod-dependent sessions (C-54 through C-59), and one externally blocked session (C-34). [L578-L587]
- (C-005/C35-E19) The acceptance section declares the library tier **SHIPPED-ON-V2**, including gate/cache/manifest/config, four events, five unit + three contract tests, mounted inspector, and CLI parse/warn. [L199-L205]
- (C-005/C35-E22) Dashboard completion includes inspector mounting in SettingsWindow, `get_tool_gate_snapshot` and `list_tool_gate_rows` registered with Tauri, panel `invoke()` calls, and two DTO round-trip tests. [L210-L214]
- (C-005/C35-A06) The visible CLI flag and implemented `ELAI_TOOL_GATE_OFF` environment reader are both reported, but their precedence and mapping at bridge construction are not defined. [L120-L122, L212-L214, L237-L238]
- (C-005/C35-E01) The objective is per-FSM-state tool visibility plus on-demand full schemas to cut repeated MCP catalog prompt cost; Phase-4K is required, C-35 precedes overlapping C-42, and C-36/37/38/39 are parallel-safe. [L1-L12]
- (C-005/C35-E02) The cited paper’s roughly 95% tool-token reduction is treated as measured, but downstream quality claims are explicitly projections requiring ELAI’s own SSB accuracy comparison. [L14-L22]
- (C-005/C35-E03) Dynamic gating intersects registered tools with a state allowlist; tools outside the active visible set are represented by one aggregate “N tools available on request” marker rather than full schemas. [L33-L37]
- (C-005/C35-E04) Lazy loading puts only tool name + one-line summary in the prompt and resolves full parameter/return JSON only after a call references that tool, targeting a catalog reduction from roughly 6–10K to 300–500 tokens. [L38-L41]
- (C-005/C35-E05) Source grounding of arXiv:2604.21816 into `tool-attention-interface.md` is required before code changes. [L55-L60]
- (C-005/C35-E06) `[tool_gating]` declares `enabled` (true after acceptance), `gate_manifest_path="plugins/tool-gate.toml"`, `lazy_schema_default=true`, and `max_eager_schemas=4`, validated at startup. [L62-L66]
- [gate] (C-005/C35-E07) Each `[gate.STATE]` manifest section separates `eager` (full schema), `gated` (name/summary), and `denied` (omitted and structured-error on attempted call). [L68-L82]
- [gate] (C-005/C35-E08) `ToolGate::load_gate` and `filter_for_state(fsm_state,full_catalog)` produce `GatedCatalog { eager_schemas: Vec<FullSchema>, gated_summaries: Vec<SummaryRow>, denied_names: HashSet<String> }`. [L84-L87]
- (C-005/C35-E09) `server.rs` replaces full-catalog rendering with state-gated rendering and resolves a referenced gated tool through `resolve_schema_on_demand(tool_name)`. [L89-L92]
- (C-005/C35-E10) A lazily resolved schema is injected only for the current call response cycle unless active-use LRU promotion keeps it eager; the LRU capacity is `max_eager_schemas`. [L92-L94]
- (C-005/C35-E11) `LazySchemaCache` uses LRU eviction and a per-tool first-load lock to prevent a concurrent thundering herd, and records hit/miss/resolve-latency metrics. [L96-L98]
- [gate] (C-005/C35-E12) SSB adds `tool_gated`, `tool_eager`, `tool_lazy_resolved`, and `tool_denied`, with `tool_name`, `fsm_state`, and `prompt_tokens_saved`; a schema bump is required by the plan. [L100-L102]
- (C-005/C35-E13) TDD proves a three-way split over 10 tools, LRU behavior over state transitions, byte-for-byte resolved-schema equality, structured denied errors with SSB, and exact positive serialization savings. [L104-L115]
- [gate] (C-005/C35-E14) Production binds the gate/cache into `DefaultMcpBridge` prompt assembly and wires emissions. [L116-L118, L205-L214]
- [gate] (C-005/C35-E15) `--tool-gate-off` is a runtime debugging escape hatch that logs a 5–10x prompt-cost warning. [L120-L122]
- [gate] (C-005/C35-E16) `ToolGateInspector` displays active state/gate, eager/gated sets, and a rolling 60-second token-savings histogram through the existing SSB stream. [L124-L127]
- (C-005/C35-E17) Missing-manifest behavior is fail-safe for availability and least-schema exposure: emit a `tool_gated` row with `reason="manifest_absent"`, then use eager empty / gated all / denied empty, never the full catalog. [L152-L155]
- (C-005/C35-E18) Cache activity may not be silent, and denied calls return structured errors rather than panicking. [L156-L158]
- (C-005/C35-E20) Completion-drive identifiers include free-function aliases `render_gated_catalog`/`resolve_schema_on_demand`, `mcp::server` re-export of `mcp_bridge`, `Summarize` + `McpToolSpec` implementation, and `DefaultMcpBridge::assemble_prompt`. [L205-L208]
- [gate] (C-005/C35-E21) `DefaultMcpBridge` owns `ToolGate` and `LazySchemaCache`; production has four emit sites through `ToolGateSsbEmitter`, with `OrchestratorToolGateSsbEmitter` as adapter. [L208-L214]
- (C-005/C35-E23) Runtime bypass implementation includes an `ELAI_TOOL_GATE_OFF` environment reader at bridge construction and warning on a double-set path. [L212-L214, L237-L238]
- (C-005/C35-E24) Two additional wiring tests cover assemble emission and a SafeDefault emission-skip case. [L213-L214]
- (C-005/C35-E25) The original ship bumped schema at version 88; current schema is declared locked at 103, so this completion session does not bump again. [L201-L204, L228-L229]
- (C-005/C35-E26) Manifest coverage, non-placeholder code, denied handling, byte-equal resolution, all event identifiers, TDD-before-wiring, contract tests, bypass warning, and live inspector are checked complete. [L220-L240]
- [deferred] (C-005/C35-E27) Deferred A5 is a 1,000-iteration LRU stress test. [L216, L224-L225]
- [deferred] (C-005/C35-E28) Deferred A10 is a live 30-tool run establishing 5–10K prompt-token savings. [L216-L218, L233-L234]
- [deferred/gate] (C-005/C35-E29) Deferred A15 is C-54/RunPod gate-on versus gate-off SWE-bench measurement requiring solve-rate delta within ±2 percentage points. [L217-L218, L241-L244]
- [gate] (C-005/C35-E30) Observable design expects a `tool_gated` row on every prompt assembly, first-reference lazy resolution, and a non-empty inspector histogram. [L160-L167]
- (C-005/C35-E31) C-35 must be live before C-54 so benchmark results use the production prompt shape; C-36 is orthogonal but benefits from the lower baseline prompt cost. [L268-L274]
- [config] (C-005/C35-D01) Add exact config defaults, three-state manifest semantics, all `GatedCatalog` fields, call-cycle/LRU behavior, per-tool concurrency lock, safe-default algorithm, all TDD cases, and implementation identifiers M1–M5/N1–N7. [L62-L158, L199-L214]
- [status] (C-005/C35-D02) Add the source’s current shipped status, version-88/locked-103 migration history, completed dashboard/Tauri adapter surfaces, and the three separately deferred acceptance items. [L199-L244]
- (C-005/C35-D03) The orientation summary presents 5–10K savings and ±2pp accuracy as smoke requirements without status; the original marks both unmeasured/deferred (A10/A15), while most library/UI work is shipped. [L216-L244]
- [gate] (C-005/C35-D04) Replace the stale `ToolGateInspector` “planned” file annotation with the acceptance section’s mounted/working status, while retaining the internal file-list conflict. [L239-L254]
- (C-005/C35-D05) Restrict the parallel-safe set to the source’s C-36/C-37/C-38/C-39 list; C-40 is not named. [L9-L12]
- (C-005/C35-A01) `max_eager_schemas` tools are said to be full-schema “regardless of gate,” which can conflict with a state’s `denied` list; precedence is not defined. [L62-L66, L68-L82]
- (C-005/C35-A02) A model must emit arguments when first naming a gated tool despite not seeing its full schema; “inject for that call’s response cycle” does not specify whether execution is paused/retried or how the initial arguments are validated. [L38-L41, L89-L94]
- (C-005/C35-A03) Hit/miss “SSB rows are mandatory,” but the four declared action types do not include `cache_hit`/`cache_miss`; payload mapping to `tool_lazy_resolved` or another row is not stated. [L96-L102, L156-L157]
- [gate] (C-005/C35-A04) Aggregate missing-manifest emission requires `reason`, while the declared event payload lists only tool name/state/tokens saved; the tool name and reason schema for an aggregate event are unresolved. [L100-L102, L152-L155]
- [gate] (C-005/C35-C01) Stub-ban behavior mandates `tool_gated` emission for SafeDefault, while the shipped-test list names “SafeDefault emission-skip”; without test semantics these appear inconsistent. [L152-L155, L213-L214]
- [config] (C-005/C35-C02) Task/config/file lists call the schema interface a `SummaryRow`/“Summary trait,” while shipped identifiers say `Summarize trait + McpToolSpec impl`. [L84-L87, L205-L208, L256]
- [gate] (C-005/C35-C03) The file list still labels `ToolGateInspector.svelte` PLANNED, but shipped acceptance says it is mounted and invoking Tauri commands. [L239-L240, L246-L254]
- (C-005/C35-C04) The original declares parallel safety only with C-36 through C-39, while the orientation summary extends that claim through C-40 without source support. [L9-L12]
- (C-005/C35-A05) LRU active-use retention, ordering under concurrent state changes, token-count method, and cache invalidation when a registered schema changes are not specified. [L89-L115]
- (C-005/C35-I01) The safe default favors tool availability while minimizing eager token exposure: every registered tool remains callable on demand, and none is denied solely because the manifest is absent. [L152-L155]
- (C-005/C35-I02) Tool visibility and call authorization are separate layers because C-35 explicitly precedes C-42’s zero-trust authorization work. [L9-L12, L268-L274]
- [gate] (SUMMARY/S-E0588) Adds --tool-gate-off as a comparison/bypass flag. [L602]
- (SUMMARY/S-E0577) Belongs to Wave 7 paper-grounded batch A. [L592-L598]
- (SUMMARY/S-E0578) Cites arXiv:2604.21816 for a measured 95% MCP-tax reduction. [L598-L599]
- (SUMMARY/S-E0579) Explicitly labels downstream quality figures as projections rather than live measurements. [L599]
- (SUMMARY/S-E0580) Depends on the Phase 4K MCP bridge being shipped. [L600]
- (SUMMARY/S-E0581) Is parallel-safe with C-36 through C-40 except C-42 because C-42 overlaps this surface. [L601]
- [gate] (SUMMARY/S-E0582) Adds ToolGate. [L602]
- [gate] (SUMMARY/S-E0583) Adds GatedCatalog. [L602]
- (SUMMARY/S-E0584) Adds LazySchemaCache. [L602]
- [gate] (SUMMARY/S-E0585) Defines per-FSM-state gate behavior in plugins/tool-gate.toml. [L602]
- [gate] (SUMMARY/S-E0586) Adds tool_gated, tool_eager, tool_lazy_resolved, and tool_denied events. [L602]
- [gate] (SUMMARY/S-E0587) Adds ToolGateInspector.svelte. [L602]
- (SUMMARY/S-E0589) Smoke uses a catalog of at least 30 tools, expects 5,000–10,000 prompt tokens saved per feature, and requires SWE-bench performance to remain within two percentage points of the gate-off arm. [L603]

### C-36: MUCOCO metamorphic operators extension

**Status**: planned
**Anchors**: ledgers C-005, SUMMARY | sample anchors: L1-L10; L12-L19; L29-L36; L50-L55; L57-L68; L70-L73 | 36 deep atoms, 9 summary atoms | IDs: C-005/C36-E01, C-005/C36-E02, C-005/C36-E03, C-005/C36-E04, C-005/C36-E05, …
**Composes with**: C-5, C-9, C-16, C-24, C-28, C-29, C-32, C-34, C-35, C-37, C-40, C-42, C-44, C-49, C-53, C-54. IX: C-005/X-I01 → C-5, C-28, C-29, C-32; C-005/C36-E01 → C-29, C-35; C-005/SUP-C36-C03 → C-5, C-40, C-53; C-005/C36-C03 → C-40, C-53; C-005/X-A05 → C-5, C-9, C-16, C-24, C-28, C-32, C-34, C-35, C-44, C-49; C-005/X-I05 → C-5, C-16, C-24, C-28, C-32, C-34, C-35, C-44, C-49

- Superseded: (C-005/SUP-C35-C04) NEWER-PLAN PRECEDENCE: C-40 description is target state relative to older C plan(s) C-36, C-39 for this conflict. Losing description retained at C-005/C35-C04. Conflict: The original declares parallel safety only with C-36 through C-39, while the orientation summary extends that claim through C-40 without source support. [L9-L12]
- Superseded: (C-005/SUP-C36-C03) NEWER-PLAN PRECEDENCE: C-53 description is target state relative to older C plan(s) C-40 for this conflict. Losing description retained at C-005/C36-C03. Conflict: The original’s parallel-safe list stops at C-40 and warns against any relation-enum patch; the orientation summary expands safety through C-53. [L7-L10]
- (C-005/C36-E01) This C-29 extension is a compile-time-gated, runtime-opt-in catalog plus measured pilot; C-29 must be shipped, C-35/37/38/39/40 are parallel-safe, and other edits to `MetamorphicRelation` must sequence after C-36. [L1-L10]
- (C-005/C36-E02) The source cites a 15% inconsistency-discovery claim and treats the transferable artifact as operator definitions because an open-source release was not confirmed. [L12-L19]
- (C-005/C36-E03) The plan explicitly interprets less than 85% pass rate under the catalog as a reproducibility problem. [L29-L36]
- (C-005/C36-E04) Paper ingestion must capture verbatim descriptions, language applicability, and false-positive notes before enum extension. [L50-L55]
- (C-005/C36-E05) Nine `MetamorphicRelation` variants behind `#[cfg(feature="mucoco_operators")]` are `VariableRename`, `WhitespaceNormalize`, `CommentStrip`, `LoopFormRewrite`, `BranchFormRewrite`, `RedundantCastInject`, `TypeAnnotationToggle`, `DeadCodeInject`, and `ImportOrderRewrite`. [L57-L68]
- (C-005/C36-E06) Every operator module exposes `apply(ast: &TreeSitterNode, rng: &mut Rng) -> Result<MutatedSource, SkipReason>` and `supports_language(Language) -> bool`; unsupported shapes return `NotApplicable` rather than panicking. [L70-L73]
- (C-005/C36-E07) `[testing.metamorphic.advanced_operators]` defaults to `enabled=false`, chain cap 3, empty exclusion list, and `require_explicit_in_test=true`. [L75-L82]
- (C-005/C36-E08) Rust is declared to support all nine; Python supports eight with loop-rewrite iterator caveats; TypeScript supports eight with type-annotation toggling in the inverse direction; all other named languages use only paper-documented subsets and explicit unsupported pairs. [L84-L90]
- (C-005/C36-E09) SSB adds `mucoco_mutation_applied`, `mucoco_inconsistency_detected`, and `mucoco_skip_reason`, each with operator name, target test ID, and language, plus a schema bump. [L92-L94]
- (C-005/C36-E10) TDD requires every apply result to be syntactically valid or skipped, chain length four to stop at three, scope-safe variable renaming, runtime-semantic-safe TypeScript toggling, and exclusion-list enforcement. [L96-L105]
- (C-005/C36-E11) Production enables operators only when the Cargo feature, runtime `enabled=true`, and per-test marker are all present. [L107-L110]
- (C-005/C36-E12) Per-test markers are `#[mucoco]` for applicable test functions or `@mucoco` docstring tags depending on language. [L141-L144]
- (C-005/C36-E13) The pilot runs over the existing 3,500+ tests and publishes per-operator/per-language detection rates plus a human-audited 50-case false-positive rate. [L111-L117]
- (C-005/C36-E14) Pilot target is at least one operator at/above 15%; a lower result files an implementation bug but explicitly does not block C-36 closure. [L117-L119]
- (C-005/C36-E15) Inconsistency emission must precede test failure, and the 50-case audit must exist at `workflow/sessions/{id}/mucoco-pilot.md` before closure. [L145-L150]
- (C-005/C36-E16) There is no new frontend; events reuse C-29’s testing-trust dashboard. [L151-L152]
- (C-005/C36-E17) Production must remain default-off until C-54 locks its baseline, even after C-36 lands. [L233-L239]
- [gate] (C-005/C36-E18) Deterministic gates inspect all nine variants/modules/events/config, run feature-enabled tests/contracts, and require the current-session pilot artifact. [L162-L185]
- (C-005/C36-E19) Acceptance requires grounded catalog, nine non-placeholder operators, complete skip taxonomy, paper-matching matrix, validating default-off config, working per-test opt-in, non-null event fields/migration, TDD-before-wiring, pilot/audit, documentation, and contracts. [L187-L203]
- (C-005/C36-E20) The implementation surface is eleven new operator/applicability files, two tests, one paper artifact, relation/module/Cargo/config/SSB/docs edits. [L205-L226]
- (C-005/C36-E21) All acceptance boxes remain unchecked in this source; status is planned, not claimed shipped. [L187-L203]
- (C-005/C36-D01) Add all nine exact variant/module names, function signatures, three activation gates, four config defaults, language claims/caveats, five TDD behaviors, exact marker forms, 50-sample audit, and default-off-until-C-54 rule. [L57-L150, L233-L239]
- [status] (C-005/C36-D02) Add planned/unshipped status and the explicit conflict over whether missing the 15% target blocks closure. [L117-L119, L187-L203]
- (C-005/C36-D03) Replace the orientation summary’s blanket “pilot acceptance requires 15%” with the source’s conflicting target/non-blocking-bug language. [L117-L119, L199-L200]
- (C-005/C36-D04) Correct the orientation summary’s C-37..C-53 parallel-safe range to the original’s C-37..C-40 plus the explicit enum-edit exclusion. [L7-L10]
- (C-005/C36-A01) TypeScript is said to support eight operators but the excluded ninth is not named; Python’s “supports 8” is also phrased as an edge case rather than an explicit unsupported mapping. [L84-L90]
- (C-005/C36-A02) RNG seeding, mutation ordering, repeat-run determinism, chain composition semantics, and shrinking/reproduction of a detected inconsistency are not specified. [L70-L82, L96-L119]
- (C-005/C36-A03) `SkipReason` is promised to cover every documented unsupported AST shape, but only `NotApplicable` is named. [L70-L73, L129-L135, L189-L192]
- (C-005/C36-C01) Task 9 says falling below 15% does not block closure, while the acceptance checkbox says the pilot “matches or exceeds” 15% (then says file a bug); the authoritative done-gate interpretation is unclear. [L117-L119, L199-L200]
- (C-005/C36-C02) `TypeAnnotationToggle` is labeled dynamic-language-only, yet the applicability section says Rust supports all nine operators. [L65-L68, L84-L90]
- (C-005/C36-C03) The original’s parallel-safe list stops at C-40 and warns against any relation-enum patch; the orientation summary expands safety through C-53. [L7-L10]
- (C-005/C36-A04) The source asserts semantic-preserving behavior yet its core TDD gate guarantees only syntactic validity for most operators; oracle/equivalence rules beyond two singled-out cases are absent. [L29-L36, L96-L105]
- (C-005/C36-A05) Event ordering under parallel tests, duplicate mutation identity, isolation of source mutations, rollback, and sensitive source content in pilot artifacts are not specified. [L92-L119, L145-L160]
- (C-005/C36-I01) Default activation requires three independent gates, making accidental blanket mutation unlikely. [L57-L59, L75-L82, L107-L110]
- (C-005/C36-I02) Syntax validation is automated, while semantic preservation is partly empirical/human-audited through the pilot. [L96-L105, L111-L117]
- (SUMMARY/S-E0590) Cites arXiv:2604.19086 and its reported finding that 15% of inputs exposed inconsistencies. [L605-L606]
- (SUMMARY/S-E0591) Depends on C-29 testing-trust T2/T3 being shipped. [L607]
- (SUMMARY/S-E0592) Is parallel-safe with C-35 and C-37 through C-53. [L608]
- (SUMMARY/S-E0593) Adds nine MetamorphicRelation variants behind the mucoco_operators feature. [L609]
- (SUMMARY/S-E0595) Adds a per-language applicability matrix for those operators. [L609]
- (SUMMARY/S-E0596) Adds mucoco_mutation_applied, mucoco_inconsistency_detected, and mucoco_skip_reason. [L609]
- (SUMMARY/S-E0597) Pilot acceptance requires at least one operator to match the paper’s 15% detection rate. [L610]
- (SUMMARY/S-E0598) Re-runs the existing 3,500-plus test suite and produces a detection table. [L610]
- (SUMMARY/S-A0594) The nine variant names are not in the summary and must be extracted from C-36.

### C-37: PER_ROLE_DECODING_DEFAULTS re-evaluation pilot

**Status**: planned (RunPod pilot deferred for full evidence)
**Anchors**: ledgers C-006, SUMMARY | sample anchors: C-37 L65-L67, L164-L166; C-37 L159-L194; C-37 L1-L7; C-37 L9-L12; C-37 L14-L22; C-37 L24-L31 | 31 deep atoms, 9 summary atoms | IDs: C-006/C37-E008, C-006/C37-E021, C-006/C37-E001, C-006/C37-E002, C-006/C37-E003, …
**Composes with**: C-4, C-6, C-26, C-35, C-36, C-38, C-39, C-40, C-46, C-53, C-54. IX: SUMMARY/S-E0601 → C-35, C-36, C-38, C-53; C-006/C37-E002 → C-26, C-35, C-36, C-38, C-39, C-40, C-54; SUMMARY/S-E0600 → C-26; C-006/C37-I002 → C-6; C-006/C37-X001 → C-6; C-004/X-007 → C-4, C-26, C-54

- (C-006/C37-E008) The Qwen2.5-Coder-7B NF4+FA2 pilot and detection-rate delta are GPU/RunPod-dependent and deferred to the C-54-adjacent window. [C-37 L65-L67, L164-L166]
- (C-006/C37-E021) Planned writes include the harness, contract, conditional migration/config/runtime changes, two SSB actions, paper/session artifacts, summary annotation, and a C-26 verification-report note; all acceptance boxes are open. [C-37 L159-L194]
- (C-006/C37-E001) Objective: experimentally re-evaluate C-26's critic/auditor JsonSchema defaults against structure-snowballing evidence before C-54 and either keep, relax, or flip the auditor mode. [C-37 L1-L7]
- (C-006/C37-E002) C-26 must be shipped; work is parallel-safe with C-35/C-36/C-38/C-39/C-40 but must precede C-54 so publication uses the measured default. [C-37 L9-L12]
- (C-006/C37-E003) Grounding is arXiv:2604.06066, dated 2026-04-07 and checked 2026-05-17, plus github.com/hongxuzhou/agentic_llm_structured_self_critique; the claimed 8B failure mode is superficial syntactic alignment without semantic correction. [C-37 L14-L22]
- (C-006/C37-E004) Current C-26 defaults are critic=JsonSchema and auditor=JsonSchema in elai_cc.toml; structured downstream parsing was the motivation. [C-37 L24-L31]
- (C-006/C37-E005) Existing anti-rubber-stamp FINDING may mitigate the paper's result, but the source labels that a hypothesis and forbids a config flip before measuring Qwen2.5-Coder-7B with ELAI's actual prompt. [C-37 L33-L40]
- (C-006/C37-E006) paper-ingestor runs in paper mode for the exact contract-test shape and source mode for the GitHub JsonSchema-vs-FreeText harness, producing structure-snowballing-interface.md plus a source interface map. [C-37 L51-L57, L161-L163]
- (C-006/C37-E007) structure_snowballing.rs uses intentional off-by-one, null-deref, and wrong-operator bugs; it runs the same fixture in JsonSchema and FreeText and outputs per-mode detection rates. [C-37 L59-L64]
- (C-006/C37-E009) If FreeText beats JsonSchema by at least 10 percentage points, auditor flips to FreeText while critic stays JsonSchema. [C-37 L69-L71]
- (C-006/C37-E010) If FreeText wins by more than zero but less than 10 points, add LightlyConstrained: require a non-empty findings array but do not constrain each finding's internal structure; auditor flips to it. [C-37 L71-L74]
- (C-006/C37-E011) If FreeText loses to JsonSchema, retain defaults, document FINDING as mitigation, and close C-37. [C-37 L74-L75]
- (C-006/C37-E012) A decoding-default migration script is conditional on a flip: rewrite existing [decoding.per_role] blocks once, preserve explicit overrides, and change only users who retained the old default. [C-37 L77-L80, L168-L169]
- (C-006/C37-E013) Add decoding_mode_pilot_run for each pilot and decoding_default_flipped only when applicable; payload fields are role, old_mode, new_mode, and detection_rate_delta; schema bumps only if a default flips. [C-37 L82-L84, L173]
- [gate] (C-006/C37-E014) TDD gates are: both modes produce a rate; migration is idempotent; and the anti-rubber-stamp FINDING requirement still fires under FreeText. [C-37 L86-L93]
- (C-006/C37-E015) DETAILED_SUMMARY_01's C-26 section receives a dated session-artifact link recording “ratified/flipped” outcome. [C-37 L95-L97]
- (C-006/C37-E016) decoding_role_contract.rs proves PER_ROLE_DECODING_DEFAULTS exposes the auditor mode through SSB and outside the runtime crate. [C-37 L99-L102]
- (C-006/C37-E017) Same-fixture/same-session comparison is mandatory; the decision matrix is deterministic, and any human override requires an SSB reason; FINDING must survive every mode. [C-37 L120-L125]
- (C-006/C37-E018) There is no required frontend; observable evidence is one decoding_mode_pilot_run row per synthetic bug per mode and a side-by-side pilot.md with per-bug table and summary delta. [C-37 L126-L134]
- (C-006/C37-E019) Static/done checks require the harness, executable dry-run/idempotent migration script, both action strings, current-session pilot artifact, anti-rubber-stamp test, and decoding-role contract. [C-37 L136-L157]
- (C-006/C37-E020) Either “flip” or “keep” is an accepted closing result; closure is the measured pilot artifact, not a particular preferred default. [C-37 L159-L177]
- (C-006/C37-E022) pilot.md becomes the canonical public methodological reason for the auditor's future decoding mode and feeds C-54. [C-37 L196-L202]
- (C-006/C37-A001) A zero-point tie is not covered by any branch: branches are >=10, >0 and <10, or <0. [C-37 L69-L75]
- (C-006/C37-A002) Fixture cardinality N, exact semantic-success oracle, sampling parameters, seeds, repetitions, and confidence intervals are unspecified. [C-37 L59-L67, L129-L134]
- (C-006/C37-A003) Adding two new action types normally changes a persisted schema, but the plan says schema bump only if defaults flip even though decoding_mode_pilot_run exists in every outcome. [C-37 L82-L84]
- (C-006/C37-A004) The migration script is listed as an unconditional produced/file artifact while Tasks and ACCEPTANCE make it conditional. [C-37 L77-L80, L106-L111, L168-L169, L181-L187]
- (C-006/C37-A005) The documentation task says “ratified/flipped,” but acceptance only requires the summary edit on flip; a keep/ratification outcome's documentation gate is unclear. [C-37 L95-L97, L170]
- (C-006/C37-I001) The plan treats semantic-bug detection rate, not syntax validity, as the decision metric; downstream parser reliability remains an unmeasured trade-off.
- (C-006/C37-I002) Keeping critic JsonSchema in the >=10pp branch assumes critic and auditor roles are behaviorally distinct enough not to share the result; that distinction is asserted, not separately tested.
- (SUMMARY/S-E0599) Cites arXiv:2604.06066 for “structure snowballing” caused by constrained decoding during reflection on 8B-class models. [L612-L613]
- (SUMMARY/S-E0600) Depends on C-26 being shipped and needs RunPod for its pilot. [L614]
- (SUMMARY/S-E0601) Is parallel-safe with C-35, C-36, and C-38 through C-53. [L615]
- (SUMMARY/S-E0602) Adds structure_snowballing_harness. [L616]
- (SUMMARY/S-E0603) Adds a migration script for per-role decoding defaults. [L616]
- (SUMMARY/S-E0604) May add a LightlyConstrained DecodingMode, making that mode conditional rather than a fixed commitment in the summary. [L616]
- (SUMMARY/S-E0605) Adds decoding_mode_pilot_run and decoding_mode_default_flipped events. [L616]
- (SUMMARY/S-E0606) Produces and applies a pilot.md decision matrix. [L617]
- (SUMMARY/S-E0607) Requires the anti-rubber-stamp FINDING regression test to pass under whichever decoding mode is chosen. [L617]

### C-38: PageRank production-path proof and Aider repomap correction

**Status**: planned
**Anchors**: ledgers C-004, SUMMARY | sample anchors: C-38 L61-L67; C-38 L1-L7, L170-L185; C-38 L9-L11; C-38 L13-L23, L61-L67; C-38 L68-L71; C-38 L39-L50, L73-L77 | 34 deep atoms, 10 summary atoms | IDs: C-004/F-C38-004, C-004/F-C38-001, C-004/F-C38-002, C-004/F-C38-003, C-004/F-C38-004, …
**Composes with**: C-4, C-8, C-21, C-25, C-26, C-35, C-37, C-39, C-40, C-46, C-47, C-52, C-54. IX: C-004/X-014 → C-4, C-8, C-21, C-25, C-26, C-46, C-47, C-52, C-54; C-004/F-C38-002 → C-35, C-37, C-39, C-40; SUMMARY/S-E0610 → C-35, C-37, C-39, C-40; C-004/X-015 → C-4, C-8, C-46, C-47, C-54

- (C-004/F-C38-004) The fresh comparison extracts graph node/edge types, weighting, self-loop policy, top-k selection, and chat-context multiplier from current `get_ranked_tags`/`get_ranked_tags_map` behavior. [C-38 L61-L67]
- (C-004/F-C38-001) C-38 is an unchecked plan with two coupled objectives: prove PageRank runs on the production query path and correct a docs-truth error that called Aider’s repo map file-level. [C-38 L1-L7, L170-L185]
- (C-004/F-C38-002) It treats B-1 DKG runtime activation as shipped, is parallel-safe with C-35–C-37/C-39/C-40, and is not parallel-safe with another plan mutating `pipeline.rs` gates. [C-38 L9-L11]
- (C-004/F-C38-003) The source records prior-oracle claims that Aider uses symbol-level NetworkX nodes, definition-without-reference self-loop weight 0.1, and a 50× chat-context multiplier, but requires a fresh fetch rather than trusting that prior read. [C-38 L13-L23, L61-L67]
- (C-004/F-C38-005) `aider-elai-pagerank-diff.md` is a source-of-truth table with columns `attribute`, `Aider behavior`, `ELAI behavior`, `divergence-intent`, and `divergence-impact`. [C-38 L68-L71]
- (C-004/F-C38-006) `DETAILED_SUMMARY_01` section 10 must replace “file-level,” re-audit all three claimed divergences, and cite the comparison artifact. [C-38 L39-L50, L73-L77]
- (C-004/F-C38-007) The corrected section carries a footnote with the fresh-fetch date and exact comparison-artifact path. [C-38 L73-L77, L131-L132]
- (C-004/F-C38-008) The current session’s `mistakes.md` gains `stale-external-reference`: external dependency behavior was specified without fresh-fetch verification. [C-38 L79-L81, L166-L167]
- (C-004/F-C38-009) The test fixture contains at least 50 synthetic code files and one deliberate hub imported by 20 others; the hub must appear in PageRank’s top three. [C-38 L83-L86, L154-L156]
- (C-004/F-C38-010) The contract invokes the production prompt/query path rather than `compute_repo_map` directly and observes `pagerank_run` through the real SSB side effect, not a mock. [C-38 L83-L90, L127-L129]
- (C-004/F-C38-011) Starting with absent `.elai/dkg.db` must transparently rebuild rather than silently skip and emit both `dkg_rebuild_started` and `dkg_rebuild_complete`. [C-38 L90-L92]
- (C-004/F-C38-012) Identical fixture input produces deterministic top-k output across ten repeated production-path invocations. [C-38 L93-L94]
- (C-004/F-C38-013) `pagerank_run` is reused if already present; rebuild actions are added only if absent, and a schema-version bump occurs only for genuinely new actions. [C-38 L96-L98]
- (C-004/F-C38-014) Each PageRank/rebuild event row carries `fixture_or_repo_id` and `elapsed_ms`. [C-38 L96-L98]
- (C-004/F-C38-015) Production code must ensure or rebuild DKG before PageRank; an existing silent `if dkg.db exists` skip blocks C-38 closure and proves B-1’s claimed fix did not land. [C-38 L100-L105, L127-L130]
- [gate] (C-004/F-C38-016) If the silent gate survives, a separate B-1 follow-up bug is filed and C-38 remains open rather than hiding the production defect inside docs/test work. [C-38 L100-L105, L184-L185]
- (C-004/F-C38-017) External-claim hygiene is also added to an apps-workflow rule such as `.claude/rules/token-discipline.md`. [C-38 L107-L110]
- (C-004/F-C38-018) The production behavior is reachable from `/explore` and from the dedicated cargo contract target. [C-38 L120-L126]
- (C-004/F-C38-019) The observable contract expects one `pagerank_run` per invocation and rebuild rows only when beginning without the DKG database. [C-38 L134-L139]
- (C-004/F-C38-020) Seven deterministic checks cover diff/doc artifacts, passing production contract, fixture size, event types, removal/replacement of silent gating, and the mistakes entry. [C-38 L141-L168]
- (C-004/F-C38-021) Closure acceptance independently requires fresh-fetch timestamp, full diff, corrected/cited summary, mistake record, all 5.a–5.d tests, 50+ files, top-three hub, real event, absent-DKG rebuild, and ten-run determinism. [C-38 L170-L185]
- (C-004/F-C38-022) On closure, B-1 and phase-1D verification reports are annotated with the new production-path lock. [C-38 L197-L202]
- (C-004/F-C38-023) Until the docs correction and contract proof land, external claims that ELAI improves on Aider are explicitly treated as unverified. [C-38 L204-L209]
- (C-004/F-C38-024) The plan says `pagerank_run` may already exist and should be confirmed; therefore “add/extend pagerank_run” is not a settled schema change. [C-38 L96-L98]
- (C-004/F-C38-025) Event implementation is alternately located in `orchestrator/src/ssb.rs` and `orchestrator/src/ssb/schema.rs`, and contract-test paths alternate between `tests/contracts/...` and `crates/orchestrator/tests/contracts/...`; exact repository placement requires inspection. [C-38 L83-L84, L96-L98, L151-L152, L187-L195]
- (C-004/F-C38-026) The exact FSM states that “expect symbol-graph context,” top-k size beyond the top-three assertion, and PageRank tie-breaking are not specified. [C-38 L30-L37, L83-L94]
- (C-004/F-C38-027) The Aider algorithm details are plan-recorded evidence pending the mandated fresh fetch, not facts newly verified by this deep read. [C-38 L13-L23, L61-L67]
- (C-004/F-C38-028) All acceptance boxes are unchecked; B-1 is reported shipped but C-38’s defining production proof is not yet evidenced in this source. [C-38 L9-L20, L170-L185]
- (C-004/D-C38-001) C-38 remains unchecked, and closure is blocked if B-1’s silent DKG gate still exists. [Summary L953-L969; C-38 L100-L105, L170-L185]
- (C-004/D-C38-002) Fresh fetch must extract six specific algorithm attributes and feed a five-column source-of-truth diff plus dated footnote/rule update. [Summary L957, L965-L967; C-38 L61-L81, L107-L110]
- (C-004/D-C38-003) The source says confirm/reuse `pagerank_run` and add rebuild events only if absent; the summary’s “extends pagerank_run” is not a settled change. [Summary L968; C-38 L96-L98]
- (C-004/D-C38-004) Every event carries repo/fixture ID and elapsed time; the production contract uses a 50-file/20-import hub and repeats top-k ten times. [Summary L963-L969; C-38 L83-L98]
- (C-004/D-C38-005) External superiority claims remain explicitly unverified until both production proof and docs correction land. [Summary L953-L969; C-38 L204-L209]
- (SUMMARY/S-E0608) Corrects a prior-oracle finding that DETAILED_SUMMARY section 10 wrongly described Aider repo maps as file-level; direct reading of aider/repomap.py showed symbol-level behavior. [L619-L620]
- (SUMMARY/S-E0609) Depends on B-1 DKG runtime activation being shipped. [L621]
- (SUMMARY/S-E0610) Is parallel-safe with C-35 through C-37, C-39, and C-40. [L622]
- (SUMMARY/S-E0611) Adds pagerank_production_contract.rs. [L623]
- (SUMMARY/S-E0612) Adds a synthetic 50-file repository with an intentional hub. [L623]
- (SUMMARY/S-E0613) Writes aider-elai-pagerank-diff.md. [L623]
- (SUMMARY/S-E0614) Corrects DETAILED_SUMMARY section 10. [L623]
- (SUMMARY/S-E0615) Adds a stale-external-reference row to mistakes.md. [L623]
- (SUMMARY/S-E0616) The later action-type summary adds dkg_rebuild_started and dkg_rebuild_complete and extends pagerank_run. [L777-L780]
- (SUMMARY/S-E0617) Acceptance proves compute_repo_map runs on the production path, the deliberate hub appears in the top three PageRank results, and an absent DKG triggers a transparent rebuild. [L624]

### C-39: MCP CLI/tool surface parity and co-process launch

**Status**: planned
**Anchors**: ledgers C-006, SUMMARY | sample anchors: C-39 L1-L7, L15-L22; C-39 L28-L38; C-39 L65-L68, L168-L170; C-39 L74-L77, L171-L173; C-39 L79-L82, L125-L130, L176-L177; C-39 L95-L98, L109-L113 | 41 deep atoms, 12 summary atoms | IDs: C-006/C39-E001, C-006/C39-E004, C-006/C39-E007, C-006/C39-E009, C-006/C39-E012, …
**Composes with**: C-5, C-6, C-13, C-35, C-36, C-38, C-40, C-41, C-42, C-44, C-54. IX: SUMMARY/S-E0619 → C-35; C-006/C39-E025 → C-44; SUMMARY/S-E0620 → C-36, C-38, C-40; C-006/C39-E002 → C-35, C-36, C-40; C-006/C39-I001 → C-6, C-13; C-006/C39-I002 → C-6

- Superseded: (C-005/SUP-C35-C04) NEWER-PLAN PRECEDENCE: C-40 description is target state relative to older C plan(s) C-36, C-39 for this conflict. Losing description retained at C-005/C35-C04. Conflict: The original declares parallel safety only with C-36 through C-39, while the orientation summary extends that claim through C-40 without source support. [L9-L12]
- (C-006/C39-E001) Objective: let CLI task and explore invoke registered MCP tools without the dashboard, using the existing HTTP JSON-RPC 2.0 bridge semantics for transport parity with Apache-2.0 OpenHands (cited at about 38K stars). OpenHands supplies the fastmcp HTTP pattern, not a standalone CLI architecture. [C-39 L1-L7, L15-L22]
- (C-006/C39-E004) Baseline gaps are no CLI bridge coverage, no CLI visibility of externally registered MCP tools, and dashboard/CLI CI exercising different transports. [C-39 L28-L38]
- (C-006/C39-E007) New cli/src/mcp_client.rs is a minimal localhost HTTP JSON-RPC 2.0 client with list_tools, invoke_tool(name,args), and lazy get_tool_schema(name). [C-39 L65-L68, L168-L170]
- (C-006/C39-E009) If no listener exists, CLI launches the dashboard bridge backend as a co-process on a random localhost port and tears it down at process exit. [C-39 L74-L77, L171-L173]
- (C-006/C39-E012) A blocking CI matrix runs every existing CLI integration test once per route and compares outputs, permitting only expected dispatch-specific SSB action-name differences. [C-39 L79-L82, L125-L130, L176-L177]
- (C-006/C39-E015) A shared Dispatch trait has McpClientDispatch and InProcessDispatch implementations; startup chooses one based on the flag. [C-39 L95-L98, L109-L113]
- (C-006/C39-E017) CLI docs gain usage/help for --via-mcp; the listed new docs also include --export-ssb. [C-39 L104-L105, L199]
- [gate] (C-006/C39-E022) Deterministic gates check modules, both impls, flag, actions, workflow, contract test, and local ci-mcp-parity.sh. [C-39 L140-L164]
- (C-006/C39-E023) File surface adds CLI client/lib/tests, three orchestrator dispatch modules, parity script/workflow, source artifact and docs, and modifies main/commands/SSB. [C-39 L183-L202]
- (C-006/C39-E002) Requires shipped Phase 4K MCP bridge and C-35's gated/lazy catalog; parallel-safe with C-36 through C-40 but sequenced after C-35. [C-39 L9-L13]
- (C-006/C39-E003) Source correction: phase-4K.txt contains no MCP content; the real bridge is plugins/src/mcp_bridge.rs plus plugins/src/mcp/{mod,gate,lazy}.rs and a C-35 alias. [C-39 L23-L26]
- (C-006/C39-E005) MCP routing is optional and backward-compatible: --via-mcp defaults off and the existing path remains when absent; bridge behavior becomes the parity source of truth. [C-39 L40-L44]
- (C-006/C39-E006) Source grounding extracts only the OpenHands MCP client and tool-dispatch subset into workflow/papers/openhands-mcp-integration.md. [C-39 L59-L63]
- (C-006/C39-E008) --via-mcp is threaded through both /task and /explore/CliAction::Run startup routing; when set, FSM dispatch uses MCP rather than the default path. [C-39 L70-L72, L95-L98, L121-L124]
- (C-006/C39-E010) If an external listener already exists, it is reused rather than launching a co-process. [C-39 L172-L174]
- (C-006/C39-E011) The selected port is surfaced through SSB. [C-39 L74-L77]
- (C-006/C39-E013) New SSB actions are cli_mcp_dispatch, cli_mcp_server_started, and cli_mcp_server_stopped; source says each carries tool_name and port. [C-39 L84-L85, L109-L115, L174]
- (C-006/C39-E014) Five TDD cases are C39.1 arbitrary/default port construction, C39.2 both Dispatch implementations satisfy the trait, C39.3 three action-type SQL round trips, C39.4 three SsbPayload construction/action reports, and C39.5 unreachable-port fail-closed behavior. [C-39 L87-L94]
- (C-006/C39-E016) cli_mcp_contract.rs checks trait-shape equivalence and equivalent SSB rows for equivalent input. [C-39 L100-L102, L178]
- (C-006/C39-E018) Consumed contracts are the C-35 alias to the MCP endpoint, ToolGate catalog filtering, and SSB; reachable commands include via-MCP task/explore and unchanged default task. [C-39 L116-L124]
- (C-006/C39-E019) Network errors are never silently retried; they emit/fail closed. Failure to discover or launch a bridge emits a structured SSB error and never panics. [C-39 L125-L130]
- (C-006/C39-E020) Existing dashboard MCP UI must still work after the Dispatch refactor. [C-39 L131-L132]
- (C-006/C39-E021) Observable proof is started-if-coprocess plus one dispatch row per call, parity-equivalent output, and an empty normalized diff. [C-39 L133-L138]
- (C-006/C39-E024) The file ledger records the three SSB actions as already SHIPPED in s-68264 at schema v89->v90 and intact through v103, even though the broader acceptance list remains unchecked. [C-39 L166-L181, L196-L198]
- (C-006/C39-E025) Closing this surface enables external MCP clients without dashboard, unified catalog CI, and the MCP prerequisite for C-44's third runtime. [C-39 L209-L215]
- (C-006/C39-A001) The type is called MCPClient in Tasks/WIRING and CliMcpClient in test/file commentary; the canonical public identifier is not stable. [C-39 L65-L68, L89-L90, L109-L110, L200-L202]
- [config] (C-006/C39-A002) Listener discovery, port configuration, readiness timeout, co-process binary invocation, graceful shutdown, orphan recovery, and crash cleanup are not specified. [C-39 L74-L77]
- (C-006/C39-A003) Server start/stop actions are required to carry tool_name even though they are not per-tool events; nullability/default semantics are absent. [C-39 L84-L85, L174]
- (C-006/C39-A004) Parity normalization permits action-name differences but does not enumerate the mapping or treatment of nondeterministic IDs, ports, order, or timestamps. [C-39 L79-L82, L133-L138]
- (C-006/C39-A005) No bridge authentication, caller authorization, origin restriction beyond localhost, or policy for externally registered tools is specified.
- (C-006/C39-I001) Correct composition with C-13 requires InProcessDispatch to be an abstraction over the production IpcToolRegistry, not a resurrection of InProcessToolDispatcher.
- (C-006/C39-I002) Random localhost binding reduces accidental network exposure, but it is not an authentication policy.
- (SUMMARY/S-E0621) Adds MCPClient in cli/src/mcp_client.rs. [L630]
- (SUMMARY/S-E0624) Adds a CI parity job that executes every CLI test once per dispatch route. [L630]
- (SUMMARY/S-E0618) Uses OpenHands MCP integration as its competitor/source anchor. [L626-L627]
- [gate] (SUMMARY/S-E0619) Depends on C-35 so it consumes the gated tool catalog. [L628]
- (SUMMARY/S-E0620) Is parallel-safe with C-36 through C-38 and C-40. [L629]
- (SUMMARY/S-E0622) Adds McpClientDispatch and InProcessDispatch behind a shared Dispatch trait. [L630]
- (SUMMARY/S-E0623) Adds --via-mcp. [L630]
- (SUMMARY/S-E0625) Adds cli_mcp_dispatch, cli_mcp_server_started, and cli_mcp_server_stopped. [L630, L777-L780]
- (SUMMARY/S-E0626) If --via-mcp finds no listener, it automatically launches a co-process. [L631]
- (SUMMARY/S-E0627) Acceptance requires the dual-route CI parity job to be green. [L631]

### C-40: Devin-style GitHub draft-PR adapter

**Status**: planned
**Anchors**: ledgers C-006, SUMMARY | sample anchors: C-40 L96-L98, L169; C-40 L132-L134; C-40 L171-L178; C-40 L232-L251; C-40 L1-L8; C-40 L10-L13 | 46 deep atoms, 14 summary atoms | IDs: C-006/C40-E012, C-006/C40-E023, C-006/C40-E027, C-006/C40-E030, C-006/C40-I001, …
**Composes with**: C-6, C-13, C-23, C-35, C-39, C-41, C-44, C-51, C-53, C-55. IX: C-006/C40-X001 → C-6; C-006/C40-X002 → C-6; SUMMARY/S-E0630 → C-35, C-39; C-006/C40-E002 → C-13, C-23, C-35, C-39, C-41, C-53; C-006/C40-I002 → C-6; C-006/C40-I001 → C-6

- Superseded: (C-005/SUP-C35-C04) NEWER-PLAN PRECEDENCE: C-40 description is target state relative to older C plan(s) C-36, C-39 for this conflict. Losing description retained at C-005/C35-C04. Conflict: The original declares parallel safety only with C-36 through C-39, while the orientation summary extends that claim through C-40 without source support. [L9-L12]
- Superseded: (C-005/SUP-C36-C03) NEWER-PLAN PRECEDENCE: C-53 description is target state relative to older C plan(s) C-40 for this conflict. Losing description retained at C-005/C36-C03. Conflict: The original’s parallel-safe list stops at C-40 and warns against any relation-enum patch; the orientation summary expands safety through C-53. [L7-L10]
- (C-006/C40-E012) The honesty section identifies ELAI/session, states mutation score and wiring pass/fail, and instructs reviewer to verify the acceptance checklist; marketing language is forbidden. [C-40 L96-L98, L169]
- (C-006/C40-E023) CLI flags flow through the session-merge path; adapter and watcher lifetime is bounded to the session and watcher tears down at session end. [C-40 L132-L134]
- [lifecycle] (C-006/C40-E027) Observable lifecycle for merge --to-pr --watch-reviews --base main is PR-created/body-posted, poll ticks while active, comment-received, stdout URL, and optional panel state. [C-40 L171-L178]
- (C-006/C40-E030) File surface adds seven github_pr module files, CLI command, contract, gh fixture, default template, optional panel; modifies feature config, CLI, merge path, SSB schema, project config, and docs. [C-40 L232-L251]
- [config] (C-006/C40-I001) The required config flag plus --to-pr command constitutes the intended authorization boundary for external write, but an explicit one-time “open this PR?” confirmation is not stated.
- (C-006/C40-E001) Objective: extend /session merge from local history to an opt-in GitHub review loop: open a PR, compose a SessionDAG body, attach audit evidence, watch review comments, and route responses through SSB sessions. [C-40 L1-L8]
- (C-006/C40-E002) Requires shipped C-13 squash/merge primitives and C-23 SessionDAG/branch_merged surface; parallel-safe with C-35 through C-39 and C-41 through C-53. [C-40 L10-L13]
- (C-006/C40-E003) Competitive grounding treats Devin's PR/review loop as a feature target, explicitly rejects unverified marketing/benchmark imitation, and notes B-25/Empire does not provide GitHub. [C-40 L15-L24]
- (C-006/C40-E004) The adapter is additive: absent [github] configuration leaves all existing sessions unchanged. [C-40 L35-L50]
- (C-006/C40-E005) defaults are enabled=false, adapter="gh", default_base_branch="main", pr_body_template="default", review_poll_interval_s=60, and auto_respond_review_comments=false. [C-40 L62-L71]
- (C-006/C40-E006) The implementation is a feature="github_pr" module at crates/orchestrator/src/github_pr, not the summary's separate crates/github_adapter package. It exports GithubAdapter, GhCliAdapter, RestApiAdapter, PrBodyComposer, ReviewCommentWatcher, and ReviewCommentRouter. [C-40 L73-L76, L143-L153, L234-L246]
- (C-006/C40-E007) GhCliAdapter executes gh pr create/view/comment, parses --json output, and reports missing binary or missing authentication as structured SSB errors. [C-40 L78-L80]
- (C-006/C40-E008) RestApiAdapter uses reqwest plus GITHUB_TOKEN for create PR/get comments/post comment and is selected only by adapter="rest" or explicit fallback opt-in after gh absence. [C-40 L82-L85]
- (C-006/C40-E009) PrBodyComposer title is the first non-empty baby-step description; summary lists steps with acceptance badges. [C-40 L87-L91]
- (C-006/C40-E010) PR audit trail links .elai/audit/{session_id}.md; test trust includes mutation score, metamorphic-relation coverage, and IntentPair coverage. [C-40 L91-L95]
- [status] (C-006/C40-E011) PR body includes open/closed stub status from C-1 through C-8. [C-40 L95]
- (C-006/C40-E013) --watch-reviews spawns an async ReviewCommentWatcher for /session merge --to-pr; it polls gh pr view every configured interval and emits on each newly seen comment. [C-40 L100-L103]
- (C-006/C40-E014) ReviewCommentRouter opens a Review-Comment-Triggered session scoped to the file mentioned by a comment only when auto_respond_review_comments=true. [C-40 L105-L108]
- (C-006/C40-E015) Automatic routing defaults off; enabling it requires explicit config and a first-hit confirmation prompt. [C-40 L105-L108, L167-L168, L219-L220]
- (C-006/C40-E016) Six action strings are github_pr_created, github_pr_body_posted, github_comment_received, github_comment_routed_to_session, github_adapter_error, and github_review_poll_tick; adding them bumps SCHEMA_VERSION. [C-40 L110-L112, L145-L150]
- (C-006/C40-E017) Merge flags are --to-pr, --base <branch>, --watch-reviews, and --draft. [C-40 L114-L118, L151]
- (C-006/C40-E018) TDD 4A.X.3 requires deterministic PrBodyComposer output from a synthetic SessionDAG. [C-40 L120-L122]
- (C-006/C40-E019) TDD 4A.X.4 uses a PATH-injected mock gh wrapper and verifies create_pr JSON parsing. [C-40 L123-L124]
- (C-006/C40-E020) TDD 4A.X.5 proves first-poll new-comment SSB emission. [C-40 L125-L126]
- (C-006/C40-E021) TDD 4A.X.6 proves no auto-trigger at the default false setting. [C-40 L127-L128]
- (C-006/C40-E022) TDD 4A.X.7 proves explicit RestApi fallback when gh is absent and refusal when fallback was not opted into. [C-40 L129-L130]
- (C-006/C40-E024) GithubPanel.svelte is planned/optional after Rust types; intended fields are PR URL, comment count, last poll, and watcher status. [C-40 L136-L138, L170, L229-L230]
- (C-006/C40-E025) github_adapter_contract.rs requires GhCliAdapter and RestApiAdapter to satisfy one trait shape. [C-40 L140-L141]
- (C-006/C40-E026) gh absence must never silently select REST; REST refuses missing GITHUB_TOKEN and may never embed a placeholder/hardcoded token. [C-40 L162-L166, L214-L215]
- [gate] (C-006/C40-E028) Static/done gates check feature declaration, all six adapter symbols, all six actions, flags, feature-enabled tests, contract, mock fixture, and schema bump. [C-40 L180-L206]
- [gate] (C-006/C40-E029) Secret-gated E2E must open, comment on, and close a draft PR in a real repository. [C-40 L226-L228]
- (C-006/C40-E031) C-13 and C-23 verification reports are updated to record the new upstream consumer/surface. [C-40 L253-L258]
- [non-goal] (C-006/C40-E032) Non-goals: no automatic merge; comment auto-response remains off; public PR stays under human maintainer control. [C-40 L260-L267]
- (C-006/C40-E033) Every acceptance box is open; GithubPanel is explicitly allowed as a separate follow-up. [C-40 L208-L230]
- (C-006/C40-A001) The watcher emits “review_comment_received” in Task 6, but the declared action enum is github_comment_received; only the latter is in WIRING/acceptance. [C-40 L100-L112]
- (C-006/C40-A002) “Scoped to the file mentioned” has no parser, multi-file policy, handling for general comments, authorization rule, dedup key, or already-routed-comment cursor. [C-40 L105-L108]
- (C-006/C40-A003) Poll retries, rate-limit/backoff, pagination, edit/delete handling, ordering, and crash resume are unspecified; repeat routing could violate idempotency.
- (C-006/C40-A004) REST token scopes, secret storage, log redaction, transport timeout, and GitHub Enterprise/base-URL support are unspecified. Context mentions a git2 + raw-REST fallback, while the concrete RestApiAdapter task specifies reqwest only; git2's role is not resolved. [C-40 L35-L39, L82-L85]
- (C-006/C40-A005) The static feature check targets .elai_cc/Cargo.toml while Files says the feature is added to crates/orchestrator/Cargo.toml. [C-40 L183-L185, L246]
- (C-006/C40-A006) C-40 cites session/dag.rs and session/merge.rs, while C-23 works in orchestrator/session_dag.rs; source paths may reflect an unrecorded refactor. [C-40 L52-L58; C-23 L79-L92]
- (C-006/C40-I002) Structured SSB errors are intended to make external side effects auditable, but the plan does not define redaction of tokens, comment bodies, repository URLs, or user identities.
- (SUMMARY/S-E0628) Uses Devin 2.0’s published autonomous PR and review-comment loop as its competitor anchor. [L633-L634]
- (SUMMARY/S-E0629) Depends on C-13 and C-23 being shipped. [L635]
- (SUMMARY/S-E0630) Is parallel-safe with C-35 through C-39. [L636]
- [gate] (SUMMARY/S-E0631) Adds a feature-gated crates/github_adapter package. [L637]
- (SUMMARY/S-E0632) Supports GhCliAdapter. [L637]
- (SUMMARY/S-E0633) Supports RestApiAdapter. [L637]
- (SUMMARY/S-E0634) Adds PrBodyComposer. [L637]
- (SUMMARY/S-E0635) Requires every generated PR body to contain an honesty section. [L637]
- (SUMMARY/S-E0636) Adds ReviewCommentWatcher and a Router. [L637]
- (SUMMARY/S-E0637) Automatic response to review comments is off by default. [L637]
- (SUMMARY/S-E0638) Adds --to-pr, --base, --watch-reviews, and --draft. [L637]
- (SUMMARY/S-E0639) Adds six github_* action types, but the summary does not name them. [L637, L777-L780]
- (SUMMARY/S-E0640) End-to-end smoke opens a draft PR in a test repository. [L638]
- (SUMMARY/S-E0641) Its body must contain mutation score, wiring-contract result, stub status, and a reviewer-instruction line. [L638]

### C-41: SkillRegistry formalization for prompt-engineer

**Status**: planned
**Anchors**: ledgers C-001, SUMMARY | sample anchors: C41:L136-177; C41:L25-40; C41:L42-49, L63-66; C41:L57-61; C41:L63-71; C41:L68-76 | 22 deep atoms, 9 summary atoms | IDs: C-001/C41-015, C-001/C41-002, C-001/C41-003, C-001/C41-004, C-001/C41-005, …
**Composes with**: C-1, C-2, C-5, C-6, C-18, C-22, C-30, C-35, C-39, C-40, C-42, C-44, C-49, C-51, C-53, C-54, C-56, C-58, C-59. IX: SUMMARY/S-E0644 → C-35, C-40, C-42, C-53; C-005/X-E09 → C-5, C-49; C-005/X-E08 → C-5, C-39, C-42, C-44, C-54; C-001/X-008 → C-1, C-44, C-51, C-58; C-001/X-012 → C-1, C-2, C-6, C-18, C-22, C-30, C-51, C-56, C-58, C-59

- [gate] (C-001/C41-015) Checks require directory/index, at least nine registry entries, three runtime types/events, TDD accuracy, and contract tests. Acceptance additionally removes inline routing, enforces versioning, requires non-null event fields, tests-before-wiring, and a rendered panel. (C41:L136-177)
- (C-001/C41-002) Existing .claude/skills and agent prompts already have INPUT_FORMAT/OUTPUT_FORMAT/TRIGGERS; the missing third is addressable runtime registry plus dispatcher instead of inline classification. ELAI's classes are SIMPLE/FOCUSED/HEAVY/IDEA plus sub-domain. (C41:L25-40)
- (C-001/C41-003) A new skill requires skills/{name}/SKILL.md, skills/{name}/meta.toml, and registry.toml entry; prompt-engineer looks it up, selection emits confidence, and versions support gradual rollout. (C41:L42-49, L63-66)
- (C-001/C41-004) paper-ingestor must produce workflow/papers/scientific-workflow-interface.md containing routing protocol and SciQuest structure. (C41:L57-61)
- (C-001/C41-005) Each meta.toml declares name, version, triggers, inputs, and outputs; SkillRegistry loads all at startup and builds trigram+sparse-keyword indexes. (C41:L63-71)
- (C-001/C41-006) lookup(task) returns score-descending Vec<(SkillCandidate,score)>; prompt-engineer injects top three candidates, but top-1 below a confidence threshold explicitly falls through to a default scaffold. (C41:L68-76)
- (C-001/C41-007) Initial skills are simple_query, focused_edit, heavy_plan, idea_intake, oracle, planning_workflow, paper_ingest, source_ingest, and benchmark_run, each with both files. (C41:L78-81)
- (C-001/C41-009) TDD covers loading/indexing, deterministic identical-input order, threshold/fall-through event, registered-vs-inline prompt non-identity, and 100 labeled synthetic tasks at ≥73% accuracy. (C41:L86-96)
- (C-001/C41-010) Every skill is semver-versioned; a breaking input/output major bump forces fall-through until prompt-engineer declares compatible. (C41:L98-100)
- (C-001/C41-011) SkillsPanel lists registered skills and recent selection events and allows a debugging manual override; any /task or /explore invoking prompt-engineer reaches lookup. (C41:L102-104, L121-123)
- (C-001/C41-012) skill_registry_contract verifies the lookup signature, monotonic scores, and fall-through. (C41:L106-107)
- (C-001/C41-013) Produced runtime types are SkillRegistry, SkillCandidate, SkillMeta, files/index, three events, and SkillsPanel; consumed systems are prompt-engineer, SSB, and filesystem. (C41:L109-123)
- [failure] (C-001/C41-014) skill_lookup emits even on a cache hit; an empty registry must emit an explicit error row rather than silently return empty; threshold fallback must emit skill_fell_through. Successful prompts leave both selection rows and a prompt-engineered artifact. (C41:L124-135)
- (C-001/C41-016) Files add the global index, 18 per-skill documents, three agent skill modules/tests, SkillsPanel, paper artifact, and modify prompt_engineer, SSB schema, and Cargo without adding a workspace member. (C41:L179-200)
- (C-001/C41-017) After landing, non-Rust contributors add skill files/metadata without changing Rust, and C-44 can route model backends from skill_selected. (C41:L207-213)
- (C-001/C41-008) skill_lookup, skill_selected, and skill_fell_through are all said to carry top_candidates, chosen_skill, confidence, and task_id, while acceptance wants non-null fields; what chosen_skill means on fall-through is not defined. (C41:L83-84, L174)
- (C-001/C41-018) The score range, confidence default, sparse/trigram weighting, and equal-score tie-break are not defined despite deterministic-order acceptance. (C41:L68-76, L86-96)
- (C-001/C41-019) A cache hit is regulated, but no cache is otherwise specified—key, lifetime, invalidation, and storage remain undefined. (C41:L124-128)
- (C-001/C41-020) Major compatibility forces fall-through, but consumer compatibility declaration/discovery and coexistence of old/new skill versions are not specified. (C41:L98-100)
- [security] (C-001/C41-021) Manual override is explicitly for debugging, but authorization, scope, persistence, and its event/evidence behavior are not stated. The task names dashboard-v2/SkillsPanel.svelte while the Files section places it under src/lib/components/. (C41:L102-104, L196)
- [status] (C-001/C41-001) C-41 formalizes implicit skills into a first-class SkillRegistry for prompt-engineer, citing a plan-asserted 44%→83% intent result. Phase 4A/prompt-engineer are required, C-39 is recommended, and it should precede C-44. (C41:L1-23)
- (SUMMARY/S-E0642) Cites arXiv:2604.21910 for an intent-accuracy increase from 44% to 83%. [L640-L641]
- (SUMMARY/S-E0643) Depends on Phase 4A and prompt-engineer being shipped. [L642]
- (SUMMARY/S-E0644) Is parallel-safe with C-35 through C-40 and C-42 through C-53. [L643]
- (SUMMARY/S-E0645) Adds a skills directory and registry.toml. [L644]
- (SUMMARY/S-E0646) Adds SkillRegistry using trigram and sparse-keyword matching. [L644]
- (SUMMARY/S-E0647) Seeds nine skills: simple_query, focused_edit, heavy_plan, idea_intake, oracle, planning_workflow, paper_ingest, source_ingest, and benchmark_run. [L644]
- (SUMMARY/S-E0648) Adds SkillsPanel.svelte. [L644]
- (SUMMARY/S-E0649) Adds skill_lookup, skill_selected, and skill_fell_through. [L644]
- (SUMMARY/S-E0650) Acceptance uses a SciQuest-style synthetic set and requires at least 73% accuracy, deliberately ten points below the cited paper’s 83%. [L645]

### C-42: Zero-trust MCP fuzzing and fail-closed authorization

**Status**: planned
**Anchors**: ledgers C-002, SUMMARY | sample anchors: C-42:L21-L34; C-42:L105-L107; C-42:L186-L186; C-42:L105-L119; C-42:L186-L200; L651; C-42:L1-L7; C-42:L36-L45; C-42:L9-L12 | 26 deep atoms, 13 summary atoms | IDs: C-002/C42-E03, C-002/C42-E17, C-002/SD-C-42-03, C-002/C42-E01, C-002/C42-E02, …
**Composes with**: C-1, C-2, C-4, C-5, C-27, C-35, C-36, C-38, C-39, C-40, C-41, C-43, C-44, C-47, C-50, C-53, C-54, C-57. IX: C-002/C42-E01 → C-40, C-44; C-002/I07 → C-2; SUMMARY/S-E0652 → C-35, C-39; C-002/I01 → C-1, C-2, C-5, C-27, C-43, C-50, C-57; SUMMARY/S-E0653 → C-35, C-36, C-38, C-41, C-43, C-53; C-005/C35-I02 → C-5, C-35

- (C-002/C42-E03) External consumers include frameworks, IDEs, CLI, future Composer 2, and GitHub PR routing; an LLM-controlled prompt is treated as potentially malicious and cannot rely on dashboard trust. [C-42:L21-L34]
- [status] (C-002/C42-E17) `AuthzPanel.svelte` is explicitly **PLANNED** to show real-time check/reject rates and click-through reason histograms; acceptance nevertheless asks it to render. [C-42:L105-L107; C-42:L186-L186]
- (C-002/SD-C-42-03) Correction: `AuthzPanel.svelte` is PLANNED, not unconditionally added; acceptance and summary overstate its current status. [C-42:L105-L119; C-42:L186-L200]
- [security] (C-002/C42-E01) Objective: deterministically discover 100% of unauthorized state transitions in ELAI's synthetic MCP fixtures before C-40/C-44 add consumers; target after fixes is zero false negatives/zero CRITICAL findings on that corpus. [C-42:L1-L7; C-42:L36-L45]
- [status] (C-002/C42-E02) Phase 4K, C-35, and C-39 must be shipped. It is orthogonal to C-36/37/38/41/43-53 but conflicts with C-35's MCP surface. [C-42:L9-L12]
- [security] (C-002/C42-E04) The fuzzer enumerates FSM state × tool × argument-shape tuples, compares C-35 gate plus per-tool policy, and reports any unauthorized-but-authorized outcome. [C-42:L36-L41]
- (C-002/C42-E05) Paper and source ingestion for arXiv:2604.25555 produces `workflow/papers/semantic-gateway-fuzzer.md`. [C-42:L14-L19; C-42:L56-L58]
- (C-002/C42-E06) Every tool gets `#[authz(states="PLANNING,EXECUTING",actors="fsm,user")]` or equivalent TOML; `AuthzPolicy::check(tool_name,fsm_state,actor)` returns `Authorized` or `Rejected{reason}`. [C-42:L60-L63]
- [security] (C-002/C42-E07) New `crates/mcp_fuzzer` is guarded by feature `mcp_fuzzer`, integration-test-only, and excluded from production binaries. [C-42:L65-L68]
- (C-002/C42-E08) `FuzzCorpus`, `FuzzRunner`, and `FuzzReport` enumerate reproducibly through seeded RNG. [C-42:L65-L68]
- (C-002/C42-E09) Corpus covers every registered tool × every FSM state × five shapes (`valid`, `oversized`, `undersized`, `malformed`, `escape-attempting`) with at least 200 tuples. [C-42:L70-L73]
- (C-002/C42-E10) Runner binds a localhost MCP bridge, invokes every tuple, and records `(authorized,expected_authorized,response_shape)`. [C-42:L75-L76]
- [failure] (C-002/C42-E11) Report categories are authorized-when-reject=`CRITICAL`, rejected-when-authorize=`LATENT BUGS/P1`, and non-structured rejection=`response-shape violation/BLOCK`. [C-42:L77-L82]
- (C-002/C42-E12) Every PR runs the fuzzer, posts a findings table as a PR comment, preserves a report JSON artifact, and any CRITICAL result blocks merge. [C-42:L84-L85; C-42:L135-L140]
- (C-002/C42-E13) Add `mcp_authz_check`, `mcp_authz_reject`, and `fuzzer_finding`; the check row must precede tool execution, rejection emits when denied, and fuzzer findings emit from integration tests. [C-42:L87-L89; C-42:L124-L138]
- [fallback/security] (C-002/C42-E14) Missing tool declarations default to DENY-ALL without failing registration and emit a structured startup warning. Rejection evidence must include `tool_name`, `fsm_state`, `actor`, and `denied_reason`; generic `unauthorized` is insufficient. [C-42:L99-L103; C-42:L127-L133]
- (C-002/C42-E15) Named tests 4K.X.3-.7 cover deterministic corpus, real bridge round-trip, per-state/tool policy, discovery of one deliberate known bug, and a post-fix zero-CRITICAL rerun; tests must precede wiring. [C-42:L91-L97; C-42:L183-L185]
- (C-002/C42-E16) Sweep every currently registered tool and add explicit authz annotations. [C-42:L99-L103]
- (C-002/C42-E18) `mcp_authz_contract.rs` verifies interface shape and fail-closed default behavior. [C-42:L109-L110]
- [gate] (C-002/C42-E19) Deterministic gates require authz module, fuzzer crate, ≥200 corpus entries, all three events, annotation cross-check for every registration, feature-enabled tests, CI job, and contract pass. [C-42:L142-L169]
- [gate] (C-002/C42-E20) C-40 and C-44 progress/merge is gated until the synthetic corpus reports zero CRITICAL; findings become individual P0 tickets. [C-42:L213-L219]
- (C-002/SD-C-42-01) Addition: The index omits the explicit threat model, feature-gated/test-only crate constraint, complete five-shape tuple model, localhost triple record, structured rejection fields, PR-comment artifact, and C-40/C-44 P0 release gate. [C-42:L21-L45; C-42:L65-L85; C-42:L127-L140; C-42:L213-L219]
- (C-002/SD-C-42-02) Addition: Severity semantics are CRITICAL for unauthorized authorization, P1 latent bug for false rejection, and BLOCK for rejection-shape failure. [C-42:L77-L82]
- (SUMMARY/S-E0659) Adds AuthzPanel.svelte. [L651]
- (SUMMARY/S-E0651) Cites arXiv:2604.25555 for 100% unauthorized-state discovery by a deterministic fuzzer. [L647-L648]
- (SUMMARY/S-E0652) Depends on C-35 and C-39 being shipped. [L649]
- (SUMMARY/S-E0653) Is parallel-safe with C-36 through C-38, C-41, and C-43 through C-53, but not with C-35 due to overlap. [L650]
- (SUMMARY/S-E0654) Adds AuthzPolicy. [L651]
- (SUMMARY/S-E0655) Adds a tool annotation expressing allowed states and actors. [L651]
- (SUMMARY/S-E0656) Unlabeled tools default to DENY ALL. [L651]
- (SUMMARY/S-E0657) Adds crates/mcp_fuzzer with a corpus of more than 200 tuples. [L651]
- (SUMMARY/S-E0658) Adds FuzzReport carrying CRITICAL, P1, and shape information. [L651]
- (SUMMARY/S-E0660) Adds a CI fuzzer job that blocks on any CRITICAL finding. [L651]
- (SUMMARY/S-E0661) Adds mcp_authz_check, mcp_authz_reject, and fuzzer_finding. [L651, L777-L780]
- (SUMMARY/S-E0662) Smoke first requires discovery of a known-bug fixture and then zero CRITICAL findings after the fix. [L652]

### C-43: Phase 4H three-layer freeze policy/guardrail

**Status**: planned
**Anchors**: ledgers C-002, SUMMARY | sample anchors: C-43:L1-L8; C-43:L10-L12; C-43:L25-L33; C-43:L14-L21; C-43:L35-L42; C-43:L61-L63; C-43:L65-L73 | 22 deep atoms, 10 summary atoms | IDs: C-002/C43-E01, C-002/C43-E02, C-002/C43-E03, C-002/C43-E04, C-002/C43-E05, …
**Composes with**: C-1, C-2, C-5, C-27, C-42, C-50, C-54, C-57. IX: C-002/I08 → C-2, C-54, C-57; C-002/I01 → C-1, C-2, C-5, C-27, C-42, C-50, C-57

- [status] (C-002/C43-E01) Objective freezes Phase 4H at its current scope and explicitly defers continuous self-reward, evolutionary prompt search, and online-RL fine-tuning until measurement justifies expansion. [C-43:L1-L8]
- (C-002/C43-E02) Requires shipped Phase 4H and is declared parallel-safe with every C plan as a policy/documentation patch plus defensive guardrails. [C-43:L10-L12]
- [lifecycle] (C-002/C43-E03) Context describes current behavior as layer 1 mistake-driven prompt modification, layer 2 auditor approval, layer 3 MSI regression-triggered auto-revert, plus a Cosmos tier for non-code artifacts. [C-43:L25-L33]
- [status] (C-002/C43-E04) MSI auto-revert is the only narrow path labeled HIGH-confidence; isolated value of layers 1-2 is unmeasured. AEL anti-evidence says mechanisms beyond memory+reflection degrade open-ended performance. [C-43:L14-L21; C-43:L35-L42]
- (C-002/C43-E05) Paper ingestion produces `ael-ablation-interface.md` with the per-mechanism ablation table verbatim. [C-43:L61-L63]
- (C-002/C43-E06) `phase-4h-scope-policy.md` cites AEL, defines current scope, and admits a fourth layer only if it improves at least two published post-C-54 benchmarks, wins a side-by-side same-fixture ablation by strictly ≥+3 percentage points, and has per-project explicit opt-in (never default). [C-43:L65-L73]
- (C-002/C43-E07) A new self-mod path requires `#[allow(self_mod_experimental, justify="...")]`; the guard emits a compile warning unless the feature is enabled, and missing annotation fails CI clippy-as-error. [C-43:L75-L81]
- (C-002/C43-E08) `ablation_harness.rs` accepts baseline/candidate `AbstractAgent` configurations, runs N matched tasks, and reports per-task and aggregate delta for appending to policy. [C-43:L83-L87]
- (C-002/C43-E09) Add `self_mod_layer_proposed` for attempted improperly-gated registration and `self_mod_ablation_run` whenever the harness runs; the latter must emit before returning. [C-43:L89-L91; C-43:L126-L129]
- (C-002/C43-E10) 4H.X.3 requires deterministic matched-pair delta on a synthetic two-config fixture. [C-43:L93-L95]
- (C-002/C43-E11) 4H.X.4 requires a compile warning for a new path lacking the annotation. [C-43:L96-L97]
- (C-002/C43-E12) 4H.X.5 grandfathers the current three layers through a baseline allowlist and forbids guardrail noise for them. [C-43:L98-L99]
- (C-002/C43-E13) 4H.X.6 requires a synthetic DGM-like candidate to show a negative delta on at least one adversarial fixture. [C-43:L100-L102]
- (C-002/C43-E14) Append the exact scope-expansion lesson to the active `mistakes.md` and add a dated C-43 scope-freeze/benchmark-gate footnote to `DETAILED_SUMMARY_01` Phase 4H. [C-43:L104-L110]
- (C-002/C43-E15) The policy must cite AEL verbatim; the guard cannot silently allow; the harness cannot return before writing its event; there is no frontend surface. [C-43:L126-L132]
- [gate] (C-002/C43-E16) A PR adding an ungated path must surface a clippy warning and cannot merge without explicit annotation plus an ablation result attached to policy. [C-43:L133-L136]
- [gate] (C-002/C43-E17) Acceptance requires paper/policy artifacts, all three decision criteria, macro, allowlist, deterministic harness, 4H.X.3-.6, negative fixture, summary/mistakes edits, and lint rejection. [C-43:L164-L177]
- [non-goal] (C-002/C43-E18) No expansion mechanism is adopted by this plan; any later self-improvement loop must first produce the harness's markdown evidence table. [C-43:L197-L203]
- (C-002/SD-C-43-01) Addition: The index omits exact adoption gates: two published benchmarks, strict ≥+3pp same-fixture gain, and per-project non-default opt-in. [C-43:L65-L73]
- (C-002/SD-C-43-02) Addition: Current scope includes mistake-driven prompt mutation and a Cosmos tier, with only MSI labeled narrow/high-confidence; the phrase “three-layer” is inconsistent inside the source and should not be flattened. [C-43:L25-L42]
- (C-002/SD-C-43-03) Addition: Tests require baseline grandfathering, a negative DGM fixture, pre-return SSB, mistakes entry, CI lint failure, and a concrete evidence table before expansion. [C-43:L93-L110; C-43:L126-L177; C-43:L197-L203]
- (C-002/SD-C-43-04) Correction/precision: The index calls this proc-macro enforcement; the source's proposed `#[allow(...)]` mechanism and detection of missing attributes are underspecified and internally tensioned. [C-43:L75-L81; C-43:L181-L188]
- (SUMMARY/S-E0663) Cites arXiv:2604.21725 for anti-evidence that adding mechanisms beyond memory plus reflection consistently hurts performance. [L654-L655]
- (SUMMARY/S-E0664) Depends on Phase 4H being shipped at its current scope. [L656]
- (SUMMARY/S-E0665) Is parallel-safe with every other C plan. [L657]
- (SUMMARY/S-E0666) Adds phase-4h-scope-policy.md and says it quotes/cites AEL verbatim. [L658]
- (SUMMARY/S-E0667) Adds a self_mod_experimental allow proc-macro guardrail requiring a justification string. [L658]
- (SUMMARY/S-E0668) Adds ablation_harness.rs that measures candidate delta against baseline. [L658]
- (SUMMARY/S-E0669) Adds a synthetic DGM-like fixture designed to expose negative delta. [L658]
- (SUMMARY/S-E0670) Adds self_mod_layer_proposed and self_mod_ablation_run. [L658, L777-L780]
- (SUMMARY/S-E0671) Adds a Phase 4H footnote to DETAILED_SUMMARY. [L658]
- (SUMMARY/S-E0672) Smoke requires the harness to report a negative delta on the synthetic candidate, proving it can surface degradation. [L659]

### C-44: Composer 2 third backend tier

**Status**: planned
**Anchors**: ledgers C-005, SUMMARY | sample anchors: L130-L144; L29-L166; L1-L6; L8-L13, L68-L73; L15-L18; L29-L32, L43-L45 | 31 deep atoms, 12 summary atoms | IDs: C-005/C44-E15, C-005/C44-D01, C-005/C44-E01, C-005/C44-E02, C-005/C44-E03, …
**Composes with**: C-1, C-5, C-6, C-9, C-16, C-24, C-28, C-32, C-34, C-35, C-36, C-39, C-40, C-41, C-42, C-49, C-51, C-55, C-58. IX: C-005/X-I04 → C-5, C-9; C-005/C44-D03 → C-42; C-001/X-008 → C-1, C-41, C-51, C-58; C-005/X-A01 → C-5, C-28, C-32, C-34, C-35, C-36, C-49; C-005/C44-A06 → C-42; C-005/X-A04 → C-5, C-34, C-35, C-49

- (C-005/C44-E15) Conditional acceptance requires an API-status record; operational path adds grounded interface, config/backend/errors/cost/failover/events/tests/panel/contracts, while infeasible path writes `composer2-deferral.md` with API status and revisit trigger. [L130-L144]
- (C-005/C44-D01) Add the conditional infeasibility closure, provisional config values, exact invoke/error contract, all five TDD cases, missing-key versus 503 behavior, 24-hour panel scope, and three benchmark arms. [L29-L166]
- (C-005/C44-E01) The intended third tier sits between local Qwen2.5-Coder and Sonnet 4.6, adopting B-37’s backend abstraction and a cost position between free/local and premium/cloud. [L1-L6]
- (C-005/C44-E02) B-37 and C-39 are shipped prerequisites; API access is a hard gate, sequencing is after C-39 and C-42, and routing integration separately depends on C-41. [L8-L13, L68-L73]
- (C-005/C44-E03) The cited report claims SWE-bench Multilingual 73.7, Terminal-Bench 61.7, CursorBench 61.3, $0.50/M input, $2.50/M output, and Kimi K2.5 base. [L15-L18]
- (C-005/C44-E04) Public API availability is unresolved in the source; if verification finds none, closure is an infeasibility/deferral note rather than code integration. [L29-L32, L43-L45]
- (C-005/C44-E05) Operational source grounding records prompt format, system-prompt conventions, and token-count quirks in `composer2-interface.md`. [L47-L49]
- (C-005/C44-E06) `[backends.composer2]` defaults disabled and declares key env `COMPOSER2_API_KEY`, provisional model ID `cursor-composer-2`, provisional 200,000 input/8,192 output limits, default executor/auditor roles, and the two token prices. [L51-L62]
- (C-005/C44-E07) `Composer2Backend` implements async `Backend::invoke(prompt,params) -> Response`; API failures become structured `BackendError`, never panic. [L64-L66]
- (C-005/C44-E08) Default skill routing prefers Sonnet for `heavy_plan`, Composer 2 for `focused_edit`, local Qwen 7B for `simple_query`, and C-54-controlled routing for `benchmark_run`. [L68-L73]
- (C-005/C44-E09) Events are `backend_composer2_invoked`, `backend_composer2_error`, and `backend_composer2_cost_accrued`, the cost row carrying `usd_cents`. [L75-L76]
- (C-005/C44-E10) TDD covers mocked HTTP response parsing, missing-key structured error with no silent fallback, usage/token-count agreement, cost calculation, and failover on HTTP 503 to a configured next tier. [L78-L85]
- (C-005/C44-E11) Every successful invocation emits cost, while failover must first emit a Composer-2 error; missing API key is explicitly not a silent-fallback case. [L110-L113]
- (C-005/C44-E12) `BackendCostPanel` reports rolling 24-hour cost by backend and session. [L87-L89]
- (C-005/C44-E13) Contract tests verify `Backend` conformance. [L91-L92]
- (C-005/C44-E14) Operational smoke enables Composer 2 and expects invoked + cost rows per routed LLM call. [L115-L118]
- (C-005/C44-E16) The source does not claim current operational or infeasible completion; all acceptance boxes remain unchecked. [L130-L144]
- (C-005/C44-E17) Either closure outcome clears C-54 to compare local-only, hybrid Sonnet/Composer/local, and full-cloud configurations. [L162-L166]
- (C-005/C44-E18) Context characterizes local Qwen as deterministic/free at 30–50 tok/s, Sonnet as expensive with 1M context, and Composer as roughly 6x cheaper on input and 3.6x on output than Sonnet. [L20-L27]
- (C-005/C44-D02) Add role-versus-skill routing ambiguity, undefined cost rounding/version pin, absence of event migration rules, and currently unchecked status. [L51-L144]
- (C-005/C44-D03) Qualify the summary’s “depends on C-42” as a prose sequencing requirement absent from the formal prerequisite list, and do not imply an integration exists before Task-1 API verification. [L8-L13, L29-L45]
- (C-005/C44-A01) `model_id` and input limit are explicitly pending Task-1 confirmation, so they are placeholders rather than verified defaults. [L51-L58]
- [config] (C-005/C44-A02) Config uses `role_routing`, while Task 5 specifies per-**skill** routing; override precedence among role, skill, explicit user choice, and failover tier is absent. [L51-L62, L68-L73, L107-L109]
- (C-005/C44-A03) `usd_cents` type, fractional-cent rounding, input/output accounting, cached-token pricing, and retry/double-charge behavior are not specified. [L51-L62, L75-L85]
- (C-005/C44-A04) The plan calls Composer 2 a “non-zero default cost vector” even though its backend config defaults disabled; this likely means enabled local-first sessions, but wording is unclear. [L53-L55, L87-L89]
- (C-005/C44-C01) Task 1 permits immediate deferral when no API exists, but the acceptance list mixes that conditional branch with unconditional-looking backend/test/UI checkboxes; minimum done gates for deferral are not cleanly separated. [L43-L45, L130-L144]
- (C-005/C44-A05) API authentication/header format, endpoint, streaming/tool-call protocol, retries/backoff, rate-limit parsing, privacy/data-retention expectations, model-version pinning, and schema migration for its three new events are not defined. [L34-L40, L64-L92]
- (C-005/C44-A06) C-42 is a required sequencing dependency in prose but not listed in the initial prerequisites; the orientation summary promotes it to a dependency. [L8-L13]
- [config] (C-005/C44-I01) Error policy distinguishes configuration failure from transient service failure: absent credentials stop with a structured error, while 503/rate-limit may use configured failover after telemetry. [L78-L85, L110-L113, L136-L139]
- (SUMMARY/S-E0673) Cites arXiv:2603.24477 and records the source’s stated SWE-bench Multilingual score of 73.7, input price of $0.50 per million tokens, output price of $2.50 per million tokens, and Kimi K2.5 base. [L661-L662]
- (SUMMARY/S-E0674) Depends on B-37, C-39, and C-42 being shipped. [L663]
- [gate] (SUMMARY/S-E0675) Is gated on verification of API access. [L663]
- (SUMMARY/S-E0676) Is parallel-safe with C-35 through C-43 and C-45 through C-53. [L664]
- (SUMMARY/S-E0677) Adds Composer2Backend implementing Backend. [L665]
- (SUMMARY/S-E0678) Routes heavy_plan to Sonnet. [L665]
- (SUMMARY/S-E0679) Routes focused_edit to Composer 2. [L665]
- (SUMMARY/S-E0680) Routes simple_query to a local 7B model. [L665]
- (SUMMARY/S-E0681) Adds BackendCostPanel.svelte. [L665]
- (SUMMARY/S-E0682) Adds backend_composer2_invoked, backend_composer2_error, and backend_composer2_cost_accrued. [L665]
- (SUMMARY/S-E0683) Enabled-call smoke must route through Composer 2. [L666]
- (SUMMARY/S-E0684) Every successful call must emit a cost row with non-null usd_cents. [L666]

### C-45: Per-role CANTANTE contribution measurement (RunPod pilot deferred)

**Status**: planned (RunPod pilot deferred for full evidence)
**Anchors**: ledgers C-003, SUMMARY | sample anchors: L19-L34; L1-L6; L8-L10; L36-L45; L47-L51, L84-L102; L47-L51, L71-L79, L100-L104 | 25 deep atoms, 11 summary atoms | IDs: C-003/E-C45-003, C-003/E-C45-001, C-003/E-C45-002, C-003/E-C45-003, C-003/E-C45-004, …
**Composes with**: C-1, C-3, C-4, C-14, C-20, C-29, C-35, C-44, C-46, C-48, C-53, C-56, C-58, C-59. IX: C-004/X-009 → C-4, C-46; SUMMARY/S-E0688 → C-35, C-44, C-46, C-53; C-001/X-011 → C-1, C-56, C-58; C-003/I-X-04 → C-3, C-48, C-59; C-003/I-X-03 → C-3, C-14, C-20, C-29, C-59

- Superseded: (C-001/SUP-A-020) NEWER-PLAN PRECEDENCE: C-58 description is target state relative to older C plan(s) C-45, C-56 for this conflict. Losing description retained at C-001/A-020. Conflict: [conflict] C-58 must use the later explicit sequential/back-to-back rule despite earlier “in parallel” wording; C-56 data and C-45 attribution are conditional dependencies despite parallel/recommended labels. (C58:L8-35, L65-121)
- (C-003/E-C45-003) For each role/task, compare role available against role forcibly dropped, route around the drop with merged fallback, and aggregate success deltas over N tasks.
- (C-003/E-C45-001) Measure causal contribution/noise per `AgentRole`; feed scores to C-41 routing and future consolidation evidence.
- (C-003/E-C45-002) Required roles are Executor, Auditor, Researcher, Designer, PromptEngineer, ContextCompiler, MemoryWriter, and MediaValidator.
- (C-003/E-C45-004) Run paper-ingestor for arXiv:2605.13295 and produce `workflow/papers/cantante-credit-attribution.md`.
- [fallback] (C-003/E-C45-005) `AgentRoleDropper`/`RoleFallback` uses `RuntimeContext.dropped_role:Option<AgentRole>`; FSM checks it and never spawns the dropped role.
- [fallback] (C-003/E-C45-006) Every role must have a documented non-empty fallback; example: dropped Researcher uses cached prior research only; panic/silent skip are banned.
- (C-003/E-C45-007) `credit_attribution_harness` accepts a task set plus nine variants (baseline + eight single-role drops), runs each, and aggregates success/audit-pass rates.
- (C-003/E-C45-008) Harness produces `credit-attribution.md`; deferred pilot produces `workflow/sessions/{id}/cantante-pilot.md`.
- [deferred] (C-003/E-C45-009) RunPod pilot uses 50 SWE-bench Verified tasks ×9 configurations =450 runs and projects about $50-$200 depending on local/hybrid mix.
- [config] (C-003/E-C45-010) Expose per-role score as SkillRegistry routing metadata; automatically deprioritize low-credit roles only behind config, default off until validated.
- (C-003/E-C45-011) Add `agent_role_dropped`, `credit_attribution_run`, and `credit_attribution_result`; result contains the per-role delta map and confidence interval over N runs.
- (C-003/E-C45-012) Credit delta is `baseline_success_rate - dropped_success_rate`, computed both per task and aggregate.
- (C-003/E-C45-013) Emit `agent_role_dropped` before dispatch reroutes; harness emits one run event per task and one result at aggregation.
- (C-003/E-C45-014) Synthetic harness must be deterministic/no-GPU; tests require all fallbacks non-empty and correct delta math.
- [status] (C-003/E-C45-015) After pilot, add the credit table/CI to `cantante-pilot.md` and annotate `DETAILED_SUMMARY_01` AgentRoles; pilot and those post-pilot artifacts remain deferred.
- [gate] (C-003/E-C45-016) Role consolidation remains a future decision and requires measured low independent credit; C-45 itself does not consolidate roles.
- (C-003/DELTA+-C45-1) Adds the exact eight roles, nine-arm/450-run design, score formula, cost range, event ordering, config-default-off routing feedback, and distinction between synthetic and deferred pilot evidence.
- [fallback] (C-003/DELTA!-C45-2) Neither a complete fallback table nor confidence calculation is supplied by the source; they are required outputs, not recovered details.
- (C-003/DELTA+-C45-3) Pilot-artifact deterministic check conflicts with deferred status, and the source has two differently named report artifacts.
- [fallback] (C-003/A-C45-01) The required “per-role fallback table” is not present; only the Researcher example is specified, leaving seven fallback behaviors open (L47-L51, L100-L104).
- (C-003/A-C45-02) Confidence level, interval estimator, paired-task treatment, success/audit aggregation, missing-run handling, random seeds, task-subset pin, and execution order are unspecified (L26-L34, L53-L61, L103-L104).
- (C-003/C-C45-01) `DETERMINISTIC_CHECK` requires the deferred RunPod pilot artifact to exist, while status/acceptance explicitly defer it (L58-L61, L111-L127).
- (C-003/A-C45-03) The source names both `credit-attribution.md` and `cantante-pilot.md` without an exact path for the first or a statement that one supersedes/embeds the other (L53-L61).
- (C-003/A-C45-04) It says no frontend pair but mentions surfacing through existing `AgentsPanel`, without a wiring task or display contract (L105).
- (SUMMARY/S-E0685) Begins Wave 8’s paper-grounded batch B and cites arXiv:2605.13295. [L668-L671]
- (SUMMARY/S-E0686) Depends on all eight AgentRoles being shipped. [L672]
- (SUMMARY/S-E0687) Defers the real pilot to RunPod. [L672]
- (SUMMARY/S-E0688) Is parallel-safe with C-35 through C-44 and C-46 through C-53. [L673]
- (SUMMARY/S-E0689) Adds AgentRoleDropper with a per-role fallback table. [L674]
- (SUMMARY/S-E0690) Dropping a role must not panic. [L674]
- (SUMMARY/S-E0691) Adds credit_attribution_harness. [L674]
- (SUMMARY/S-E0692) Produces cantante-pilot.md with a per-role credit table. [L674]
- (SUMMARY/S-E0693) Adds agent_role_dropped, credit_attribution_run, and credit_attribution_result. [L674]
- (SUMMARY/S-E0694) Synthetic-harness output must be deterministic. [L675]
- (SUMMARY/S-E0695) Post-RunPod pilot uses a 50-task SWE-bench subset and reports per-role credit with confidence intervals. [L675]

### C-46: Difficulty-tier selective role routing

**Status**: planned
**Anchors**: ledgers C-004, SUMMARY | sample anchors: C-46 L1-L7, L116-L127; C-46 L9-L11; C-46 L13-L17, L43-L46; C-46 L47-L51; C-46 L53-L55; C-46 L55-L58 | 29 deep atoms, 12 summary atoms | IDs: C-004/F-C46-001, C-004/F-C46-002, C-004/F-C46-003, C-004/F-C46-004, C-004/F-C46-005, …
**Composes with**: C-4, C-8, C-21, C-25, C-26, C-35, C-37, C-38, C-45, C-47, C-52, C-53, C-54. IX: SUMMARY/S-E0698 → C-35, C-45, C-47, C-53; C-004/X-016 → C-4, C-8, C-52, C-54; C-004/X-009 → C-4, C-45; C-004/F-C46-002 → C-35, C-45, C-47, C-53; C-004/X-015 → C-4, C-8, C-38, C-47, C-54; C-004/X-014 → C-4, C-8, C-21, C-25, C-26, C-38, C-47, C-52, C-54

- (C-004/F-C46-001) C-46 is an unchecked plan for an optional selective-delegation dispatcher mode that avoids the full agent chain on simple work. [C-46 L1-L7, L116-L127]
- (C-004/F-C46-002) Eight `AgentRole` implementations are prerequisite; C-45 credit attribution is recommended but not a strict gate; C-46 is declared parallel-safe with C-35–C-45 and C-47–C-53. [C-46 L9-L11]
- (C-004/F-C46-003) The design is grounded through a required paper-ingestor pass over arXiv:2605.05007, producing `workflow/papers/uno-orchestra-interface.md`. [C-46 L13-L17, L43-L46]
- (C-004/F-C46-004) `TaskDifficultyClassifier` features are task-description length, code-file count, repo-map centrality of affected files, and embedding cosine to a fixed easy/hard reference set. [C-46 L47-L51]
- (C-004/F-C46-005) The classifier emits a normalized difficulty score in `[0.0,1.0]`. [C-46 L47-L51]
- (C-004/F-C46-006) `SelectiveDelegator` combines difficulty with C-45 per-role credit weights when those weights are available and returns the roles to spawn. [C-46 L53-L55]
- (C-004/F-C46-007) For `difficulty < 0.3`, the default role set is exactly Executor plus Auditor. [C-46 L55-L58]
- (C-004/F-C46-008) For `0.3 <= difficulty < 0.7`, the role set adds ContextCompiler and PromptEngineer. [C-46 L55-L58]
- (C-004/F-C46-009) For `difficulty >= 0.7`, the full chain is selected. [C-46 L55-L58]
- (C-004/F-C46-010) An Auditor “insufficient context” result emits `selective_delegation_escalated` and re-spawns the full chain, consuming one retry under the existing bubble-up policy. [C-46 L60-L63]
- [config] (C-004/F-C46-011) `[agent.selective_delegation]` config contains `enabled: bool` default false, `classifier_model: string` default `"trigram_centrality"`, and `reference_set_path: string`. [C-46 L65-L67]
- (C-004/F-C46-012) `selective_delegation_decision` is emitted per task with difficulty and chosen role set; it must occur before any spawn so routing is auditable. [C-46 L69-L71, L100-L105]
- (C-004/F-C46-013) Additional events are `selective_delegation_escalated` and `selective_delegation_cost_saved`; the cost-saved metric must be nonnegative. [C-46 L69-L71, L102-L105]
- (C-004/F-C46-014) TDD requires stable classifier scores under a deterministic seed and exact low/mid/high policy sets. [C-46 L73-L77]
- [failure] (C-004/F-C46-015) TDD also simulates auditor failure to prove full-chain re-spawn/events and compares spawn counts against a full-chain baseline for cost saved. [C-46 L78-L81]
- (C-004/F-C46-016) A dedicated `selective_delegation_contract.rs` and scoped agent test invocation are required. [C-46 L87-L88, L107-L114]
- (C-004/F-C46-017) The 100-task SWE-bench Verified pilot is deferred to RunPod; full-chain and selective arms target solve-rate delta within ±2 percentage points and cost reduction at least 30%. [C-46 L83-L85]
- (C-004/F-C46-018) The feature remains disabled by default and is flipped only after the pilot meets the target. [C-46 L124-L127]
- (C-004/F-C46-019) The planned file surface adds routing modules/tests/paper/pilot and modifies dispatcher, SSB schema, and `elai_cc.toml`. [C-46 L129-L140]
- (C-004/F-C46-020) The context describes a current six-role “full chain,” while prerequisites require eight roles and the medium policy names two roles not in that six-role list; the authoritative full-chain membership/order is not reconciled. [C-46 L9-L10, L21-L24, L53-L58]
- (C-004/F-C46-021) Score calibration, feature normalization/weights, boundary behavior for NaN/out-of-range scores, and the fixed reference-set schema are not defined. [C-46 L47-L58]
- (C-004/F-C46-022) `reference_set_path` has no default, required/optional rule, missing-file fallback, or privacy/update policy. [C-46 L65-L67]
- (C-004/F-C46-023) The paper-backed cost-savings paragraph says “typically” 40–60% on an easy half before the plan’s own pilot; it is a motivating estimate, not an acceptance result. [C-46 L33-L35, L83-L85]
- (C-004/F-C46-024) All acceptance boxes are unchecked and the decisive pilot is deferred, so production enablement is intentionally not authorized by this source. [C-46 L83-L85, L116-L127]
- (C-004/D-C46-001) “Trigram” is the default classifier-model identifier, not one of the actual feature bullets; features are length, file count, centrality, and reference-task embedding cosine. [Summary L1115; C-46 L47-L51, L65-L67]
- (C-004/D-C46-002) Auditor insufficiency re-spawns the **full chain** in one retry, not merely an unspecified “larger delegation tier.” [Summary L1121; C-46 L60-L63]
- [config] (C-004/D-C46-003) Config defaults disabled/`trigram_centrality`, emits decision before spawn, and requires nonnegative savings. [Summary L1113-L1123; C-46 L65-L71, L100-L105]
- (C-004/D-C46-004) Six-role context, eight-role prerequisite, and medium-tier ContextCompiler/PromptEngineer membership do not yield one unambiguous full-chain definition. [Summary L1110-L1120; C-46 L9-L10, L21-L24, L53-L58]
- [deferred] (C-004/D-C46-005) The pilot is explicitly deferred to RunPod and the feature must remain disabled until its ±2pp/≥30% gate passes. [Summary L1123; C-46 L83-L85, L124-L127]
- (SUMMARY/S-E0696) Cites arXiv:2605.05007. [L677-L678]
- (SUMMARY/S-E0697) Depends on eight AgentRoles; C-45 is recommended but not mandatory. [L679]
- (SUMMARY/S-E0698) Is parallel-safe with C-35 through C-45 and C-47 through C-53. [L680]
- (SUMMARY/S-E0699) Adds TaskDifficultyClassifier using trigram, centrality, and reference-task cosine features. [L681]
- (SUMMARY/S-E0700) Adds SelectiveDelegator. [L681]
- (SUMMARY/S-E0701) Difficulty below 0.3 selects Executor plus Auditor. [L681]
- (SUMMARY/S-E0702) Difficulty from 0.3 through below 0.7 adds CC and PE roles. [L681]
- (SUMMARY/S-E0703) Difficulty at least 0.7 selects the full chain. [L681]
- (SUMMARY/S-E0705) Auditor verdict “insufficient” bubbles up to a larger delegation tier. [L681]
- (SUMMARY/S-E0706) Adds selective_delegation_decision, selective_delegation_escalated, and selective_delegation_cost_saved. [L681]
- (SUMMARY/S-E0707) Pilot uses 100 SWE-bench tasks, must stay within plus/minus two solve-rate points, and targets at least 30% lower cost. [L682]
- (SUMMARY/S-A0704) The summary does not expand the CC and PE role abbreviations.

### C-47: Per-project sled case base for near-match tool hints

**Status**: planned
**Anchors**: ledgers C-004, SUMMARY | sample anchors: C-47 L88-L90, L125-L127; Summary L1133-L1140; C-47 L25-L35, L47-L51; C-47 L1-L7, L105-L116; C-47 L9-L11; C-47 L13-L16, L43-L46; C-47 L25-L35 | 31 deep atoms, 9 summary atoms | IDs: C-004/F-C47-022, C-004/D-C47-005, C-004/F-C47-001, C-004/F-C47-002, C-004/F-C47-003, …
**Composes with**: C-4, C-8, C-21, C-25, C-26, C-35, C-36, C-38, C-42, C-46, C-48, C-52, C-53, C-54. IX: SUMMARY/S-E0710 → C-46, C-48, C-53; SUMMARY/S-E0709 → C-35, C-42; C-004/X-014 → C-4, C-8, C-21, C-25, C-26, C-38, C-46, C-52, C-54; C-004/F-C47-002 → C-35, C-36, C-42, C-46; C-004/X-011 → C-4, C-35, C-42; C-004/X-008 → C-4, C-21, C-25, C-26, C-46, C-54

- (C-004/F-C47-022) The enabled flag’s default is not stated. [C-47 L88-L90, L125-L127]
- (C-004/D-C47-005) Per-project call shapes/string values/session IDs have no specified redaction, encryption, retention, access, isolation, or deletion policy. [Summary L1133-L1140; C-47 L25-L35, L47-L51]
- (C-004/F-C47-001) C-47 is an unchecked plan for a per-project tool-call memory loop recording call, response, and post-call audit/FSM outcome, then consulting prior cases before future MCP calls. [C-47 L1-L7, L105-L116]
- (C-004/F-C47-002) C-35 Tool Attention, C-42 zero-trust fuzzing, and Phase 4K MCP bridge must already be shipped; it is sequence-after C-35/C-42 and otherwise parallel-safe with C-36–C-46. [C-47 L9-L11]
- (C-004/F-C47-003) A required paper-ingestor pass over arXiv:2605.15041 produces `workflow/papers/case-based-calibration.md`. [C-47 L13-L16, L43-L46]
- (C-004/F-C47-004) The case store is filesystem-side, per project, and estimated around 100 KB; the design is deterministic retrieval, explicitly not fine-tuning. [C-47 L25-L35]
- (C-004/F-C47-005) `CaseBase` may use sled or SQLite, with sled preferred; its schema is `tool_name | timestamp | call_shape_hash | call_shape | response_kind | post_call_audit_ok | session_id`. [C-47 L47-L51]
- (C-004/F-C47-006) Storage is capped at 100 entries per tool and evicts the oldest entry. [C-47 L47-L51, L91-L94]
- (C-004/F-C47-007) `CaseRetriever` returns top-K prior cases using Jaccard similarity over argument-key sets plus Levenshtein similarity over string values. [C-47 L53-L56]
- (C-004/F-C47-008) Retrieval returns the nearest case plus `NEAR_MATCH` or `NO_MATCH`, and similarity ordering must be deterministic. [C-47 L53-L56, L66-L70]
- (C-004/F-C47-009) A hint is injected into the model’s next prompt only when a near match previously succeeded and exposes a known shape correction such as a missing/renamed key. [C-47 L58-L61]
- [config] (C-004/F-C47-010) Config-based hint suppression is enforced at the event/emit point, not after a hint has been produced. [C-47 L66-L72, L91-L95]
- (C-004/F-C47-011) Events are `case_base_consulted`, `case_base_hint_emitted`, and `case_base_call_logged`; every event must carry non-null `tool_name`. [C-47 L63-L64, L105-L113]
- (C-004/F-C47-012) `case_base_call_logged` must emit only after the tool returns, preserving the response/audit trail. [C-47 L91-L93]
- (C-004/F-C47-013) Consultation occurs before every MCP dispatch when `[tool_use.case_base.enabled]` is active. [C-47 L78-L90]
- (C-004/F-C47-014) Five TDD cases prove persistent round-trip, deterministic ordering, rename-hint emission, oldest-first cap eviction, and disabled-config suppression. [C-47 L66-L72]
- (C-004/F-C47-015) A dedicated `case_base_contract.rs` and scoped plugin tests are required. [C-47 L73-L76, L96-L103]
- (C-004/F-C47-016) The pilot replays 50 existing ELAI sessions and must measure at least a 10% reduction in repeated tool-call errors. [C-47 L73-L75, L114-L115]
- (C-004/F-C47-017) Planned file changes add case store/tests/paper/pilot and modify MCP server pre-dispatch, SSB schema, and `[tool_use.case_base]` config. [C-47 L118-L127]
- (C-004/F-C47-018) Task text permits sled or SQLite, but acceptance specifically requires persistence “via sled”; backend freedom and the done gate do not agree literally. [C-47 L47-L51 versus L107-L109]
- (C-004/F-C47-019) The store is called “LRU,” but eviction is explicitly oldest-by-timestamp rather than access-recency; the deterministic timestamp policy is the only fully specified behavior. [C-47 L47-L51, L66-L72, L91-L94]
- (C-004/F-C47-020) The context says the index holds the last N successful calls, while the schema and logging rule retain response/audit fields for every returned call; whether failed calls occupy the 100-entry cap is unspecified. [C-47 L25-L32, L47-L51, L91-L93]
- (C-004/F-C47-021) Top-K, the NEAR_MATCH threshold, Jaccard/Levenshtein weighting, tie-break order, canonical call-shape hashing, and non-string-value treatment are not defined. [C-47 L53-L61]
- (C-004/F-C47-023) Stored call shapes, string values, and session IDs create a privacy/security surface, but no redaction, encryption, access control, retention duration, cross-project isolation test, or user-deletion workflow is specified. [C-47 L25-L35, L47-L51]
- (C-004/F-C47-024) All acceptance boxes are unchecked; the claimed repeat-error improvement remains a pilot target rather than observed evidence. [C-47 L105-L116]
- (C-004/D-C47-001) The task permits sled or SQLite (sled preferred) while acceptance demands sled; “adds a sled-backed CaseBase” overstates a conflicted choice. [Summary L1135; C-47 L47-L51, L107-L109]
- (C-004/D-C47-002) Eviction is deterministically oldest-by-timestamp, not true access-based LRU as the summary says. [Summary L1136; C-47 L47-L51, L91-L94]
- (C-004/D-C47-003) The seven-column schema, post-return logging order, success-only hint condition, emit-point suppression, and non-null `tool_name` event gate are omitted. [Summary L1133-L1140; C-47 L47-L72, L91-L95, L105-L113]
- (C-004/D-C47-004) Top-K/threshold/weight/tie/hash behavior and whether failures occupy the cap are unspecified. [Summary L1135-L1138; C-47 L25-L32, L47-L61]
- (SUMMARY/S-E0708) Cites arXiv:2605.15041. [L684-L685]
- (SUMMARY/S-E0709) Depends on C-35 and C-42. [L686]
- (SUMMARY/S-E0710) Is parallel-safe with C-46 and C-48 through C-53. [L687]
- (SUMMARY/S-E0711) Adds a sled-backed CaseBase. [L688]
- (SUMMARY/S-E0712) Keeps at most 100 least-recently-used cases per tool. [L688]
- (SUMMARY/S-E0713) Adds CaseRetriever using Jaccard and Levenshtein similarity. [L688]
- (SUMMARY/S-E0714) Injects a hint when a NEAR_MATCH indicates parameter renaming. [L688]
- (SUMMARY/S-E0715) Adds case_base_consulted, case_base_hint_emitted, and case_base_call_logged. [L688]
- (SUMMARY/S-E0716) A 50-session replay must show at least a 10% reduction in repeated tool errors. [L689]

### C-48: Memory provenance tiers T0–T3 for safety-critical views

**Status**: planned
**Anchors**: ledgers C-003, SUMMARY | sample anchors: L28-L37; L1-L6; L16-L26; L28-L37, L52-L54; L39-L40; L52-L54, L95-L101 | 29 deep atoms, 12 summary atoms | IDs: C-003/E-C48-004, C-003/E-C48-001, C-003/E-C48-002, C-003/E-C48-003, C-003/E-C48-004, …
**Composes with**: C-3, C-4, C-28, C-42, C-45, C-46, C-47, C-49, C-52, C-53, C-59. IX: SUMMARY/S-E0718 → C-42; C-003/I-X-04 → C-3, C-45, C-59; SUMMARY/S-E0719 → C-46, C-47, C-49, C-53; C-004/X-012 → C-4, C-28, C-52

- Superseded: (C-003/DELTA+-C48-2) Adds the closed four-query list, write/read universality, non-optional provenance, pessimistic migration rule, exact adversarial fixture, and “extends, not replaces” constraint.
- (C-003/E-C48-004) T0 includes CLI/dashboard/explicit user writes; T1 includes local source and git history; T2 includes WebFetch/paper ingestor; T3 is unreviewed model output.
- [security] (C-003/E-C48-001) Stratify memory by provenance and refuse T2/T3 memory for safety-critical decisions to mitigate prompt injection/context poisoning.
- (C-003/E-C48-002) Explicit threats are malicious commit/changelog memory, web/paper prompt injection, and model-fabricated persistent claims.
- (C-003/E-C48-003) `TrustTier = T0_UserDirect | T1_CodebaseDerived | T2_WebSourced | T3_ModelEmitted`.
- (C-003/E-C48-005) Safety-critical queries consult only T0+T1; enrichment may consult T2+T3 but must retain provenance labels.
- [lifecycle] (C-003/E-C48-006) Every memory write is tagged at write time.
- (C-003/E-C48-007) Extend `MemoryRow` with non-optional `trust_tier:TrustTier` and `provenance:Provenance` (source URL or filepath), bump schema, and migrate every pre-C-48 row to pessimistic T3.
- [config] (C-003/E-C48-008) Safety-critical query types are `destructive_command_intent`, `merge_decision`, `auth_decision`, and `credential_fetch`.
- (C-003/E-C48-009) Those query types use `MemoryView::T0_T1_only()` to transparently exclude T2/T3.
- (C-003/E-C48-010) Every memory read surfaces provenance; safety-critical agent prompts label each cited fact, e.g. `user_direct` versus `web_sourced`.
- (C-003/E-C48-011) Add exact action identifiers `memory_tier_assigned`, `memory_query_filtered`, and `safety_critical_blocked`.
- (C-003/E-C48-012) When an excluded T2/T3 row would have influenced a safety query, emit `safety_critical_blocked`; silent exclusion is prohibited.
- (C-003/E-C48-013) TDD covers trust-tier schema round trip, exclusion of T2/T3, visible prompt provenance, and pessimistic legacy migration.
- (C-003/E-C48-014) Test 4D.X.7 places malicious text in T2 memory and requires no influence on a safety-critical decision.
- (C-003/E-C48-015) Add `tests/contracts/memory_trust_contract.rs`; all three actions must emit on relevant behavior.
- (C-003/E-C48-016) Add trust/view modules, TDD/adversarial/contract tests, shell migration, and paper artifact; modify `MemoryRow`, memory writer, and orchestrator SSB schema.
- (C-003/E-C48-017) The existing memory layer is extended, not replaced.
- (C-003/E-C48-018) Closure claims a malicious third-party commit message cannot influence a safety-critical decision and calls C-48 the wave's only explicit adversarial threat model.
- (C-003/DELTA!-C48-1) The exact third action is `safety_critical_blocked`, not the summary's `memory_safety_critical_blocked`.
- (C-003/DELTA+-C48-2) Adds the closed four-query list, write/read universality, non-optional provenance, pessimistic migration rule, exact adversarial fixture, and “extends, not replaces” constraint.
- (C-003/DELTA!-C48-3) The summary omits the internal threat-model contradiction: T1 git history is admitted to safety decisions even though malicious commit messages motivate the guardrail and T1 is said not to govern safety policy.
- (C-003/C-C48-01) A malicious commit message is the motivating threat and is classified by the stated taxonomy as git-history/codebase-derived T1; T1 is allowed in `T0_T1_only`. Therefore the closure claim that such a commit cannot influence a safety-critical decision does not follow from the explicit policy (L21-L22, L32-L33, L39-L40, L141-L144).
- (C-003/C-C48-02) T1 is called trustworthy for technical decisions “but not safety policy,” yet all safety-critical views include T1 without an additional policy filter (L32-L33, L39-L40).
- (C-003/A-C48-01) `Provenance` variants/fields for user/model sources, URL/path sanitization, sensitive-path exposure, inheritance through derived/merged/summarized rows, and validation are undefined (L56-L69).
- (C-003/A-C48-02) Query classification location, behavior for unknown/multi-purpose queries, caller bypass prevention, and whether the four-item list is closed/configurable are unspecified (L61-L64).
- (C-003/A-C48-03) Determining that an excluded row “would have influenced” a decision is counterfactual and no algorithm distinguishes `memory_query_filtered` from `safety_critical_blocked` emission (L71-L73).
- (C-003/A-C48-04) Schema-version number/scope, migration transaction/idempotency/rollback, write failure for absent tier/provenance, and concurrent reads during migration are unspecified (L56-L59, L128-L139).
- (SUMMARY/S-E0717) Cites arXiv:2605.05704. [L691-L692]
- (SUMMARY/S-E0718) Depends on memory-writer and C-42. [L693]
- (SUMMARY/S-E0719) Is parallel-safe with C-46, C-47, and C-49 through C-53. [L694]
- (SUMMARY/S-E0720) Adds TrustTier T0 for user-direct memory. [L695]
- (SUMMARY/S-E0721) Adds T1 for codebase-derived memory. [L695]
- (SUMMARY/S-E0722) Adds T2 for web-sourced memory. [L695]
- (SUMMARY/S-E0723) Adds T3 for model-emitted memory. [L695]
- (SUMMARY/S-E0724) Adds MemoryView::T0_T1_only for safety-critical queries. [L695]
- [gate] (SUMMARY/S-E0725) Propagates memory provenance into prompts. [L695]
- (SUMMARY/S-E0726) Migrates every pre-C-48 memory row to T3. [L695]
- (SUMMARY/S-E0727) Adds memory_tier_assigned, memory_query_filtered, and memory_safety_critical_blocked. [L695]
- (SUMMARY/S-E0728) Smoke places prompt injection in a T2 row and requires it not to affect the safety-critical decision identified as 4D.X.7. [L696]

### C-49: Per-project successful-trace experience corpus

**Status**: planned
**Anchors**: ledgers C-005, SUMMARY | sample anchors: L1-L10; L19-L35, L48-L50; L33-L35; L52-L55; L54-L55, L93-L95; L57-L59 | 33 deep atoms, 9 summary atoms | IDs: C-005/C49-E01, C-005/C49-E02, C-005/C49-E03, C-005/C49-E04, C-005/C49-E05, …
**Composes with**: C-5, C-16, C-24, C-28, C-32, C-34, C-35, C-36, C-41, C-44, C-46, C-48, C-50, C-53. IX: C-005/X-A04 → C-5, C-34, C-35, C-44; C-005/C49-I02 → C-5; C-005/X-A01 → C-5, C-28, C-32, C-34, C-35, C-36, C-44; C-005/C49-I01 → C-5; C-005/C49-E01 → C-35, C-41, C-50; C-005/X-I05 → C-5, C-16, C-24, C-28, C-32, C-34, C-35, C-36, C-44

- (C-005/C49-E01) The plan turns successful task traces into reusable prompt-policy fragments; prompt-engineer is required, C-41 is recommended rather than mandatory, and C-35..48/C-50..53 are parallel-safe. [L1-L10]
- (C-005/C49-E02) Per-project/per-skill storage is `.elai/experience/{skill_name}.md` with `Distilled prompt fragments`, `Common parameter shapes`, and `Avoid patterns` sections. [L19-L35, L48-L50]
- (C-005/C49-E03) Successful runs append deduplicated learned material; failures append to avoid patterns. [L33-L35]
- (C-005/C49-E04) `Distiller` runs after successful task completion with skill name, full execution trace, and SSB log, appending to the corpus. [L52-L55]
- (C-005/C49-E05) Deduplication collapses exact five-gram shingle matches and is required to be deterministic rather than random sampling. [L54-L55, L93-L95]
- (C-005/C49-E06) Distillation also consults `mistakes.md`; avoid patterns flow into prompt scaffolds, and new mistakes update the avoid section. [L57-L59]
- (C-005/C49-E07) Prompt engineering consults the selected skill’s corpus after `SkillRegistry` lookup and injects roughly 500 tokens, preferring newest fragments. [L61-L64]
- (C-005/C49-E08) Over-budget behavior evicts oldest material, not newest. [L69-L74, L93-L96]
- (C-005/C49-E09) Greenfield/missing corpus is a transparent no-op and must not panic. [L69-L74, L93-L96]
- (C-005/C49-E10) SSB adds `experience_distilled`, `experience_consulted`, and `experience_token_budget_exceeded`. [L66-L67]
- (C-005/C49-E11) TDD covers Markdown round-trip, shingle dedup, mistakes-to-scaffold flow, oldest-first budget enforcement, and empty-corpus safety. [L69-L75]
- (C-005/C49-E12) Contract tests live at `tests/contracts/experience_corpus_contract.rs`. [L76]
- (C-005/C49-E13) Reachability is post-successful-task distillation and every prompt-engineer invocation. [L78-L92]
- (C-005/C49-E14) Acceptance requires grounding, Markdown schema, append/dedup, bidirectional avoid integration, post-SkillRegistry consultation, budget, all events, TDD/greenfield, and contracts. [L107-L117]
- (C-005/C49-E15) A non-empty distilled section after ten same-codebase sessions is explicitly a smoke test, **not** a formal acceptance gate. [L118-L119]
- (C-005/C49-E16) Implementation adds experience/distiller modules and two tests, modifies prompt engineer/post-task hook/SSB schema, and adds the paper artifact. [L121-L130]
- (C-005/C49-E17) All acceptance boxes remain unchecked; no shipped status is claimed. [L107-L119]
- (C-005/C49-E18) Source grounding runs paper-ingestor against arXiv:2605.14443 and produces `workflow/papers/iterative-distillation.md`. [L37-L46, L109, L127]
- [lifecycle] (C-005/C49-E19) The stated expected lifecycle is that the first project session pays full prompt-construction cost and later sessions pay less as the corpus accumulates. [L132-L136]
- (C-005/C49-D01) Add the exact three-section corpus schema, success inputs/trigger, deterministic exact-shingle rule, bidirectional mistake behavior, newest-first selection/oldest-first eviction, five TDD cases, and transparent greenfield skip. [L19-L96]
- (C-005/C49-D02) Add the failed-run trigger conflict and all unspecified persistence/privacy/concurrency/tokenization/event-lifecycle rules. [L33-L96]
- (C-005/C49-D03) Correct the orientation summary’s “ten sessions must produce” wording: the original labels it smoke evidence, not a formal acceptance gate. [L118-L119]
- (C-005/C49-D04) Restore the original’s broader parallel-safe range C-35..C-48; the orientation summary mentions only C-46..C-48 plus C-50..C-53. [L8-L10]
- [failure] (C-005/C49-C01) Failures are said to append avoid patterns, but the only distiller/post-task trigger is successful completion; no failed-task hook is defined. [L33-L35, L52-L59, L90-L92, L128-L130]
- (C-005/C49-A01) “5-gram” does not define whether grams are words, tokens, characters, or normalized Markdown, nor the comparison scope across sections/sessions. [L52-L55]
- (C-005/C49-A02) The approximately-500-token cap has no tokenizer, exact boundary, config key, truncation unit, or treatment of an individually oversized fragment. [L61-L64, L69-L74]
- (C-005/C49-A03) Extraction/scoring of “consistently appeared” fragments and common parameter shapes from full traces/SSB is not specified. [L26-L35, L52-L55]
- (C-005/C49-A04) Concurrent/atomic corpus writes, crash recovery, corrupt Markdown handling, file locking, and idempotence across repeated post-task hooks are unspecified. [L48-L59]
- (C-005/C49-A05) No secret/PII/source-code redaction, provenance, trust boundary, prompt-injection filtering, retention limit, or user deletion control is specified before persisting full-trace-derived content. [L19-L35, L52-L64]
- (C-005/C49-A06) SSB payload schemas, schema-version bump/migration, emission timing on transparent skip, and failure/retry behavior are absent. [L66-L67, L78-L96]
- (C-005/C49-A07) `{skill_name}` filename sanitization and fallback binning when C-41 is absent are not defined even though C-41 is optional. [L8-L9, L26-L35, L48-L64]
- (C-005/C49-I01) Project-local storage is intended to amortize repeated prompt construction without creating a global cross-project memory. [L19-L35, L121-L135]
- (C-005/C49-I02) Recency selection and oldest-first eviction define a simple deterministic retention policy once entries have a stable order. [L61-L74, L93-L96]
- (SUMMARY/S-E0729) Cites arXiv:2605.14443, described as the most recent paper in its batch. [L698-L699]
- (SUMMARY/S-E0730) Depends on prompt-engineer; C-41 is recommended but not mandatory. [L700]
- (SUMMARY/S-E0731) Is parallel-safe with C-46 through C-48 and C-50 through C-53. [L701]
- (SUMMARY/S-E0732) Adds ExperienceCorpus stored per project and per skill at .elai/experience/{skill}.md. [L702]
- (SUMMARY/S-E0733) Adds Distiller using five-gram shingling for deduplication. [L702]
- (SUMMARY/S-E0734) Integrates avoid-patterns from mistakes.md. [L702]
- (SUMMARY/S-E0735) Caps injected experience at about 500 tokens. [L702]
- (SUMMARY/S-E0736) Adds experience_distilled, experience_consulted, and experience_token_budget_exceeded. [L702]
- (SUMMARY/S-E0737) Ten sessions on one codebase must produce a non-empty Distilled section. [L703]

### C-50: Contract-Coding before skeleton generation

**Status**: planned
**Anchors**: ledgers C-002, SUMMARY | sample anchors: C-50:L1-L7; C-50:L18-L32; C-50:L9-L11; C-50:L13-L16; C-50:L43-L44; C-50:L46-L50; C-50:L47-L50; C-50:L95-L99; C-50:L52-L55 | 20 deep atoms, 10 summary atoms | IDs: C-002/C50-E01, C-002/C50-E02, C-002/C50-E03, C-002/C50-E04, C-002/C50-E05, …
**Composes with**: C-1, C-2, C-5, C-27, C-35, C-42, C-43, C-46, C-49, C-51, C-53, C-57. IX: C-002/I01 → C-1, C-2, C-5, C-27, C-42, C-43, C-57; C-002/C50-E02 → C-35, C-49, C-51, C-53; SUMMARY/S-E0740 → C-46, C-49, C-51, C-53; C-002/I09 → C-2

- (C-002/C50-E01) A fresh project emits per-function preconditions/postconditions before implementation, extending FILE_PLAN and skeleton-first completeness. [C-50:L1-L7; C-50:L18-L32]
- (C-002/C50-E02) Requires shipped Phase 4F completeness and is parallel-safe with C-35-C-49 and C-51-C-53. [C-50:L9-L11]
- (C-002/C50-E03) Paper/source ingestion targets arXiv:2604.13100 and its open implementation; cited benchmark evidence is 47% functional and near-perfect structural integrity. [C-50:L13-L16; C-50:L43-L44]
- (C-002/C50-E04) `ContractSpec` holds `preconditions:Vec<Predicate>`, `postconditions:Vec<Predicate>`, and `contract_language:ContractLang`; supported expressions are Rust `assert!`, Python `assert`, and TypeScript predicates. [C-50:L46-L50]
- [security] (C-002/C50-E05) Predicates are target-language source strings and must parse through tree-sitter; parse failure rejects the contract and cannot fall back to permissive validation. [C-50:L47-L50; C-50:L95-L99]
- (C-002/C50-E06) `ContractEmitter` runs after FILE_PLAN and before skeleton, prompts once per planned function for PRE/POST predicates, validates syntax, and stores `.elai/contracts/{file_path}.toml`. [C-50:L52-L55]
- (C-002/C50-E07) `ContractViolationCheck` runs at body completion, parses the body AST, checks preconditions on entry paths and postconditions on return paths, and extends MiniCritic. [C-50:L57-L60]
- (C-002/C50-E08) Violations emit SSB before entering `ContractViolationRetry`; a corrected body can resolve the retry through the existing structural chain. [C-50:L57-L60; C-50:L95-L97]
- [config] (C-002/C50-E09) `elai new project` enables contracts by default; existing projects opt in through `[code_completeness.contracts.enabled]`. [C-50:L62-L64]
- (C-002/C50-E10) Add `contract_emitted`, `contract_violation_detected`, and `contract_retry_resolved`. [C-50:L66-L67]
- (C-002/C50-E11) 4F.X.3 verifies TOML round-trip of `ContractSpec`. [C-50:L69-L70]
- (C-002/C50-E12) 4F.X.4 validates predicates through target-language tree-sitter. [C-50:L70-L71]
- (C-002/C50-E13) 4F.X.5 detects a deliberately violated body. [C-50:L71-L72]
- (C-002/C50-E14) 4F.X.6 verifies retry resolution with a corrected body. [C-50:L72-L73]
- (C-002/C50-E15) 4F.X.7's task text targets 47% functional and ≥95% structural success on Greenfield-5; final acceptance uses conservative gates of ≥40% and ≥90%. [C-50:L74-L75; C-50:L120-L122]
- [gate] (C-002/C50-E16) Add `contract_emitter_contract.rs`; deterministic checks require module/symbol/event/fixture existence plus passing code-completeness contract tests. [C-50:L77-L77; C-50:L101-L109]
- (C-002/C50-E17) The intended three-layer pipeline is FILE_PLAN declarations → behavioral contracts → skeleton bodies constrained by contracts. [C-50:L138-L142]
- (C-002/SD-C-50-01) Addition: The index omits the exact `ContractSpec`/`Predicate`/`ContractLang` schema, supported Rust/Python/TS encodings, tree-sitter parse-or-reject rule, TOML artifact path, AST entry/return-path check, and existing-project opt-in. [C-50:L46-L64]
- (C-002/SD-C-50-02) Addition: The event-before-retry ordering and no-permissive-fallback bans are explicit. [C-50:L95-L99]
- (C-002/SD-C-50-03) Correction/precision: Summary's ≥40%/≥90% acceptance is correct, but the TDD task itself says 47%/95%+. Both thresholds must be preserved until the intended test gate is resolved. [C-50:L69-L75; C-50:L120-L122]
- (SUMMARY/S-E0738) Cites arXiv:2604.13100 for 47% functional success and near-perfect structural integrity on Greenfield-5. [L705-L706]
- (SUMMARY/S-E0739) Depends on Phase 4F being shipped. [L707]
- (SUMMARY/S-E0740) Is parallel-safe with C-46 through C-49 and C-51 through C-53. [L708]
- (SUMMARY/S-E0741) Adds ContractSpec with preconditions and postconditions expressed in the target language. [L709]
- (SUMMARY/S-E0742) ContractEmitter runs after FILE_PLAN and before skeleton generation. [L709]
- (SUMMARY/S-E0743) Adds ContractViolationCheck to MiniCritic. [L709]
- (SUMMARY/S-E0744) Adds ContractViolationRetry. [L709]
- (SUMMARY/S-E0745) Enables contracts in the greenfield-init template. [L709]
- (SUMMARY/S-E0746) Adds contract_emitted, contract_violation_detected, and contract_retry_resolved. [L709]
- (SUMMARY/S-E0747) Greenfield-5 smoke targets at least 40% functional and at least 90% structural success, intentionally conservative against the cited 47% and near-perfect values. [L710]

### C-51: Aider Polyglot harness infrastructure (no GPU run)

**Status**: planned
**Anchors**: ledgers C-001, SUMMARY | sample anchors: C51:L86-101; C51:L135-145; C51:L1-7, L46-84, L120-133; C51:L29-31; C51:L40-47, L102-105; C51:L49-54 | 21 deep atoms, 10 summary atoms | IDs: C-001/C51-013, C-001/C51-017, C-001/C51-018, C-001/C51-003, C-001/C51-004, …
**Composes with**: C-1, C-2, C-6, C-18, C-22, C-30, C-40, C-41, C-44, C-46, C-50, C-52, C-53, C-55, C-56, C-58, C-59. IX: C-001/X-012 → C-1, C-2, C-6, C-18, C-22, C-30, C-41, C-56, C-58, C-59; C-006/C55-X001 → C-6, C-40, C-44, C-55; SUMMARY/S-E0750 → C-46, C-50, C-52, C-53; C-001/X-009 → C-1, C-55, C-58; C-001/X-008 → C-1, C-41, C-44, C-58

- (C-001/C51-013) Produced surfaces are the vendor/corpus, PolyglotAdapter, six language runners, CLI, and three events; consumed surfaces are the existing agent pipeline, C-11 sandbox, configured backends, and SSB. (C51:L86-101)
- (C-001/C51-017) Adds vendor/manifest, adapter, six shims, TDD/contract tests; modifies CLI and SSB schema. (C51:L135-145)
- (C-001/C51-018) “No benchmark run in C-51” coexists with a CLI described as refusing a full sweep only when GPU is absent, so GPU-present behavior during this patch is unclear. GPU detection, language distribution, mock semantics, output location, concurrency, timeout/retry, and event payloads are also unspecified. (C51:L1-7, L46-84, L120-133)
- (C-001/C51-003) For each task the harness presents scaffolding, collects emitted code, runs hidden tests in a sandbox, and scores pass/fail. (C51:L29-31)
- (C-001/C51-004) Vendor the MIT harness and corpus under external/aider_polyglot, include LICENSE, and pin the git SHA in external/manifests.toml. (C51:L40-47, L102-105)
- (C-001/C51-005) PolyglotAdapter accepts problem_dir+prompt, runs prompt-engineer→executor→wait→collect emitted file, and uses whichever elai_cc.toml backend is configured rather than binding a model itself. (C51:L49-54)
- [security] (C-001/C51-006) Hidden tests use C-11 isolation with gcc, go test, mvn, jest, pytest, and cargo test shims; host leakage is forbidden. (C51:L56-59, L102-106)
- (C-001/C51-007) Per-task scorecard fields are problem_id, language, time_ms, tokens_in, tokens_out, cost_usd, and passed; aggregate format must match Aider for direct comparison. (C51:L61-64)
- (C-001/C51-008) Emit polyglot_task_started, polyglot_task_completed, and polyglot_scorecard_emitted. (C51:L66-67)
- (C-001/C51-009) TDD proves exactly 225 tasks with non-empty per-language counts and a synthetic adapter round-trip. (C51:L69-72)
- (C-001/C51-010) A known-good Python solution must pass, an intentionally bad one must fail, and scorecard JSON must match the leaderboard format. (C51:L72-76)
- [failure] (C-001/C51-011) elai benchmark polyglot without GPU exits with an explicit RunPod/local-GPU-required error; --dry-run instead runs five samples against a mock backend. (C51:L78-82)
- (C-001/C51-012) tests/contracts/polyglot_adapter_contract.rs is an additional gate. (C51:L84-85)
- [gate] (C-001/C51-014) Static proof checks vendor/LICENSE/manifest/adapter/events/tests and exact five-task dry-run output. (C51:L108-118)
- [gate] (C-001/C51-015) Acceptance requires 225 tasks, all six runners, end-to-end synthetic dispatch, scorecard diff against existing leaderboard JSON, all events/TDD/contracts, refusal without GPU, and working dry-run. (C51:L120-131)
- [non-goal] (C-001/C51-016) No public benchmark number is produced; closure is infrastructure landed and smoke-tested. C-55 is characterized as GPU bind/config flip, not a C-51 code path. (C51:L132-149)
- [status] (C-001/C51-001) C-51 is test infrastructure only after shipped Phase 4F, parallel with C-35..C-50/C-52/C-53; it lands a six-language 225-problem harness with no benchmark/model invocation, reserving the real run for C-55. (C51:L1-10)
- [status] (C-001/C51-002) The source cites the 225-problem leaderboard and a qwen2.5-coder:32b/Ollama result of 72.9% as analogous to ELAI's local-32B tier. The CPU-only landing/run split is intentional: review/smoke need no GPU, while RunPod/local GPU later binds the adapter by config. (C51:L12-27)
- (SUMMARY/S-E0748) Vendors the MIT-licensed Aider harness at a pinned commit SHA. [L712-L713]
- (SUMMARY/S-E0749) Depends on Phase 4F being shipped. [L714]
- (SUMMARY/S-E0750) Is parallel-safe with C-46 through C-50, C-52, and C-53. [L715]
- (SUMMARY/S-E0751) Adds .elai_cc/external/aider_polyglot with 225 tasks. [L716]
- (SUMMARY/S-E0752) Adds PolyglotAdapter. [L716]
- (SUMMARY/S-E0753) Adds language-specific SandboxedTestRunner support for C++, Go, Java, JavaScript, Python, and Rust. [L716]
- (SUMMARY/S-E0754) Adds elai benchmark polyglot. [L716]
- (SUMMARY/S-E0755) Adds polyglot_task_started, polyglot_task_completed, and polyglot_scorecard_emitted. [L716]
- (SUMMARY/S-E0756) Smoke dry-runs five sample tasks against a mock backend. [L717]
- (SUMMARY/S-E0757) A full sweep must refuse to run without a GPU. [L717]

### C-52: Formal verification research spike (Gemma/Dafny candidates)

**Status**: planned
**Anchors**: ledgers C-004, SUMMARY | sample anchors: C-52 L1-L11, L38-L39; C-52 L8-L10; C-52 L10-L11; C-52 L3-L6, L13-L16, L49-L52; C-52 L18-L23; C-52 L25-L31, L57-L60 | 34 deep atoms, 12 summary atoms | IDs: C-004/F-C52-001, C-004/F-C52-002, C-004/F-C52-003, C-004/F-C52-004, C-004/F-C52-005, …
**Composes with**: C-4, C-8, C-21, C-25, C-26, C-28, C-29, C-35, C-38, C-46, C-47, C-48, C-51, C-53, C-54. IX: C-004/X-016 → C-4, C-8, C-46, C-54; SUMMARY/S-E0761 → C-46, C-51, C-53; C-004/X-012 → C-4, C-28, C-48; C-004/F-C52-003 → C-35, C-51, C-53; C-004/X-014 → C-4, C-8, C-21, C-25, C-26, C-38, C-46, C-47, C-54; C-004/X-013 → C-4, C-28, C-29, C-54

- (C-004/F-C52-001) C-52 is explicitly research-grade, optional, opt-in, and narrow; closure may be either a successful three-module spike or a documented production infeasibility. [C-52 L1-L11, L38-L39]
- (C-004/F-C52-002) Phase 4F code completeness is required, C-50 contract coding is preferred but not strict, and a Dafny toolchain on the developer host is required for the success path. [C-52 L8-L10]
- (C-004/F-C52-003) It is declared parallel-safe with C-35–C-51 and C-53. [C-52 L10-L11]
- (C-004/F-C52-004) The motivating paper claim is a prior-oracle report of 90.91% Dafny verification success for Gemma 4-31B on NL2VC-60; a paper-ingestor task must produce `nl2vc-dafny-interface.md`. [C-52 L3-L6, L13-L16, L49-L52]
- (C-004/F-C52-005) The formal layer sits above mutation/metamorphic/wiring evidence and targets mathematical pre/postcondition proofs only where practical. [C-52 L18-L23]
- (C-004/F-C52-006) Candidate modules are FSM transition validation, C-48 `MemoryView` trust-tier filtering, and C-28 mutation-score aggregation. [C-52 L25-L31, L57-L60]
- (C-004/F-C52-007) File I/O, networking, and LLM dispatch are explicitly out of scope. [C-52 L33-L36]
- (C-004/F-C52-008) Each `.dfy` file is a specification mirroring claimed Rust pre/postconditions, not a one-to-one Rust source translation. [C-52 L62-L64]
- (C-004/F-C52-009) Toolchain installability is checked on Windows and Linux development hosts; if installation is nontrivial, concrete blockers are documented and work jumps to the infeasibility path. [C-52 L53-L55]
- (C-004/F-C52-010) The success path adds `formal/fsm_transition.dfy`, `formal/memory_view.dfy`, and `formal/mutation_aggregator.dfy`. [C-52 L91-L99]
- [gate] (C-004/F-C52-011) Feature-gated `VerificationHarness` exposes `cargo verify-dafny --module FSMTransition`, shells out to `dafny verify`, treats clean output as pass, and sends failure to SSB. [C-52 L66-L69]
- (C-004/F-C52-012) CI integration is optional/informational, never blocking, and runs when a PR touches one of the three spike modules. [C-52 L71-L72]
- (C-004/F-C52-013) If toolchain install, all three translations, or CI integration cannot land, `dafny-spike-infeasibility.md` with concrete blocker descriptions becomes the closure artifact. [C-52 L74-L77]
- (C-004/F-C52-014) Infeasibility cannot be a vague “too hard”; acceptable blockers include installation failure, a language-feature gap, or CI-runner incompatibility. [C-52 L109-L114]
- (C-004/F-C52-015) If only one of three spikes lands, a partial-success note is mandatory and full success must not be claimed. [C-52 L113-L114]
- (C-004/F-C52-016) Events are `dafny_verify_run`, `dafny_verify_pass`, and `dafny_verify_fail`. [C-52 L79-L80]
- (C-004/F-C52-017) TDD proves a nonzero Dafny exit surfaces `dafny_verify_fail` and all three known-good specifications verify cleanly. [C-52 L82-L86]
- (C-004/F-C52-018) Regardless of outcome, `.elai_cc/docs/formal-verification.md` documents the spike result. [C-52 L88-L89]
- (C-004/F-C52-019) Success is reachable from the cargo alias and optional CI; infeasibility is reachable through the explicit alternate artifact path. [C-52 L91-L108]
- (C-004/F-C52-020) Deterministic closure is disjunctive: either Dafny exists plus the FSM spec verifies, or an infeasibility file exists with at least one `blocker:` entry. [C-52 L116-L127]
- (C-004/F-C52-021) Success acceptance requires paper artifact, documented installability, all three specifications passing, reproducible harness, all three events, TDD, and informational CI. [C-52 L129-L140]
- (C-004/F-C52-022) Infeasibility acceptance requires at least one blocker, a documented revisit trigger, and C-52 closure as `DEFERRED` with an explicit re-evaluation condition. [C-52 L142-L146]
- (C-004/F-C52-023) The success file plan adds a new workspace crate, three Dafny files, harness tests, workflow, paper artifact, workspace membership, and SSB actions; the infeasibility plan adds the blocker and formal-verification docs. [C-52 L148-L161]
- (C-004/F-C52-024) A valid outcome explicitly leaves testing-trust as ELAI’s highest evidence tier if formal verification proves infeasible. [C-52 L163-L169]
- (C-004/F-C52-025) Task 2 asks installation on both Windows and Linux, while success acceptance only requires at least one supported host; the minimum host-coverage gate is inconsistent. [C-52 L53-L55 versus L131-L134]
- (C-004/F-C52-026) Context says spike “3–5” modules, but task selection, produced files, tests, and acceptance all fix exactly three. [C-52 L33-L36 versus L57-L60, L93-L99, L134-L136]
- (C-004/F-C52-027) The plan verifies Dafny specifications but defines no refinement, translation-validation, or equivalence check proving the Rust implementations actually satisfy those specifications. [C-52 L62-L69]
- (C-004/F-C52-028) Events are listed among success-path symbols, but it is unclear whether an installability failure before the harness exists must emit any Dafny event. [C-52 L53-L55, L79-L80, L91-L102]
- (C-004/F-C52-029) The success file inventory omits `formal-verification.md` even though documentation is required for either outcome. [C-52 L88-L89, L148-L161]
- (C-004/F-C52-030) All acceptance boxes are unchecked, and a deferred infeasibility closure is first-class rather than evidence that verification shipped. [C-52 L129-L146]
- [deferred] (C-004/D-C52-001) C-52 may validly close `DEFERRED`; partial success requires a note and cannot be called full success. [Summary L1219-L1233; C-52 L109-L114, L142-L146]
- (C-004/D-C52-002) Installability is requested on Windows+Linux but success accepts one host; context says 3–5 modules while the actual contract fixes three. [Summary L1221-L1229; C-52 L33-L36, L53-L60, L131-L136]
- (C-004/D-C52-003) `.dfy` files specify claimed Rust behavior but the plan contains no proof/refinement bridge showing Rust conforms to them. [Summary L1226-L1229; C-52 L62-L69]
- (C-004/D-C52-004) Informational CI is nonblocking, infeasibility needs a concrete blocker plus revisit trigger, and formal-verification documentation is required for either outcome. [Summary L1229-L1233; C-52 L71-L89, L142-L161]
- (SUMMARY/S-E0758) Is explicitly research-grade. [L719]
- (SUMMARY/S-E0759) Cites arXiv:2604.22601 for a reported 90.91% score by Gemma 4-31B on NL2VC-60. [L719-L720]
- (SUMMARY/S-E0760) Recommends C-48’s MemoryView as a candidate dependency and requires a Dafny toolchain. [L721]
- (SUMMARY/S-E0761) Is parallel-safe with C-46 through C-51 and C-53. [L722]
- (SUMMARY/S-E0762) Adds a Dafny specification for FSM transitions. [L723]
- (SUMMARY/S-E0763) Adds a Dafny specification for MemoryView trust filtering. [L723]
- (SUMMARY/S-E0764) Adds a Dafny specification for the mutation aggregator. [L723]
- (SUMMARY/S-E0765) Adds VerificationHarness. [L723]
- (SUMMARY/S-E0766) Adds an optional CI job. [L723]
- (SUMMARY/S-E0767) Permits an infeasibility-note path instead of successful verification, but requires a concrete blocker. [L723-L724]
- (SUMMARY/S-E0768) The later event summary adds dafny_verify_run, dafny_verify_pass, and dafny_verify_fail. [L781]
- (SUMMARY/S-E0769) Acceptance is either verification of all three spike modules or dafny-spike-infeasibility.md containing at least one concrete blocker. [L724]

### C-53: NF4 watch plus in-context vs orchestration counter-test

**Status**: planned (RunPod pilot deferred for full evidence)
**Anchors**: ledgers C-006, SUMMARY | sample anchors: C-53 L109-L145; C-53 L22-L26, L47-L50; C-53 L1-L7; C-53 L9-L10; C-53 L14-L20; C-53 L22-L26 | 29 deep atoms, 10 summary atoms | IDs: C-006/C53-E019, C-006/C53-A001, C-006/C53-E001, C-006/C53-E002, C-006/C53-E003, …
**Composes with**: C-6, C-46, C-52. IX: SUMMARY/S-E0773 → C-46, C-52; C-006/C53-X001 → C-6; C-006/C53-I002 → C-6; C-006/C53-I001 → C-6; C-006/C53-X002 → C-6

- Superseded: (C-005/SUP-C36-C03) NEWER-PLAN PRECEDENCE: C-53 description is target state relative to older C plan(s) C-40 for this conflict. Losing description retained at C-005/C36-C03. Conflict: The original’s parallel-safe list stops at C-40 and warns against any relation-enum patch; the orientation summary expands safety through C-53. [L7-L10]
- Superseded: (SUMMARY/SUP-S-59SLOTS) NEWER-PLAN PRECEDENCE: later coverage section (59 slots) supersedes header 35-session legend. Retained: SUMMARY/S-E0003 — The later coverage section supersedes the old count with 59 session slots: 52 CPU/cloud sessions (C-1 through C-33 and C-35 through C-53), six RunPod-dependent sessions (C-54 through C-59), and one externally blocked session (C-34). [L578-L587]
- [gate] (C-006/C53-E019) Done gates check the watch doc/entry, bench file, CLI flag, procedural fixture, two actions, compilable local baseline, paper artifact, RunPod pilot, matrix outcome, and any default-off route. [C-53 L109-L145]
- (C-006/C53-A001) “Subscribe” and “major release” have no automation, cadence, owner, notification channel, or semantic-version policy. [C-53 L22-L26, L47-L50]
- (C-006/C53-E001) Objective: one low-priority tracker combines a passive bitsandbytes NF4 optimization watch with an orchestration-vs-single-prompt procedural-task ablation; neither item merits a separate C-N. [C-53 L1-L7]
- (C-006/C53-E002) Header declares no prerequisites and parallel safety with every C-N. [C-53 L9-L10]
- (C-006/C53-E003) NF4 grounding is arXiv:2604.02556, dated 2026-04-02, reporting 2.0-2.2x kernel and up to 1.54x end-to-end gains from NVIDIA shared-memory patterns; the integration is research-grade CUDA/Triton. [C-53 L14-L20]
- (C-006/C53-E004) ELAI consumes bitsandbytes NF4 through python/elai_inference/server.py, so an upstream adoption can arrive through a pinned dependency upgrade. [C-53 L22-L26]
- (C-006/C53-E005) Add huggingface/bitsandbytes to .elai_cc/docs/upstream-watch.md with paper reference and trigger “major release, run benchmark delta.” [C-53 L45-L50]
- (C-006/C53-E006) nf4_e2e is a trigger-only, non-normal-CI benchmark over a fixed prompt set, comparing current bitsandbytes with upstream-latest and measuring sidecar tokens/second end to end. [C-53 L51-L54]
- (C-006/C53-E007) On a bitsandbytes major bump, adopt/pin and changelog the version only if end-to-end speedup is at least 1.2x; the paper's 1.54x is treated as an upper bound. [C-53 L56-L59]
- (C-006/C53-E008) In-context grounding is arXiv:2604.27891, dated 2026-05-02/forum item 10, claiming orchestration is obsolete for procedural tasks; the plan treats it as direct counter-evidence to Phase 4A-4G. [C-53 L28-L41]
- (C-006/C53-E009) paper-ingestor paper mode produces workflow/papers/in-context-prompting-counter.md. [C-53 L61-L64]
- (C-006/C53-E010) Define and document a 30-task SWE-bench Verified procedural subset, exemplified by mechanical API-v1-to-v2 conversion. [C-53 L66-L69]
- (C-006/C53-E011) --baseline-single-prompt bypasses Phase 4A-4G decomposition, performs one prompt-engineering call, and uses the same model as the default/full arm. [C-53 L71-L73]
- (C-006/C53-E012) Run both arms on all 30 tasks, compare solve rate and cost, and write the pilot artifact; execution is deferred to RunPod alongside C-54. [C-53 L75-L76, L127-L134]
- (C-006/C53-E013) If single-prompt wins on solve rate and cost, record it and use C-41 skill_selected classification to route procedural work to a single-prompt path. [C-53 L78-L81]
- (C-006/C53-E014) Otherwise document decomposition's win and archive the counter-evidence. [C-53 L80-L81]
- (C-006/C53-E015) Add counter_test_run and procedural_routing_decision actions. [C-53 L83-L84, L87-L94]
- (C-006/C53-E016) A routing change is exposed as [routing.procedural_single_prompt], is config-gated, and defaults off so speculation never changes routing. [C-53 L98-L107, L134]
- (C-006/C53-E017) Consumed contracts are bitsandbytes, existing FSM dispatch, C-41 SkillRegistry only for a winning-route flip, and the SSB bus. [C-53 L94-L101]
- (C-006/C53-E018) Evidence bans require wall-clock end-to-end rather than synthetic NF4 measurement and a per-task pilot artifact containing both arms. [C-53 L102-L107]
- (C-006/C53-E020) Every acceptance box is open; closure is evidence that either defends decomposition or justifies a pivot, and a future pivot may feed C-46's tier-zero SelectiveDelegator. [C-53 L120-L152]
- (C-006/C53-A002) A Rust cargo bench measuring a Python GPU sidecar lacks a stated process protocol, hardware pin, warm-up, sample count, variance calculation, or current/upstream dependency isolation. [C-53 L51-L59]
- (C-006/C53-A003) The 30-task “pick” process has no deterministic classifier, frozen IDs/SHA, reviewer protocol, or anti-cherry-pick rule. [C-53 L66-L69]
- (C-006/C53-A004) counter_test_run and procedural_routing_decision payload schemas, schema-version behavior, and emission cardinality are absent. [C-53 L83-L84]
- (C-006/C53-A005) The decision matrix has no branch for equal solve rate, solve/cost trade-offs, inconclusive uncertainty, or failed tasks. [C-53 L78-L81]
- (C-006/C53-I001) “Current” versus “upstream-latest” must be pinned in the result artifact for reproducibility, although the plan names only the current project pin implicitly.
- (C-006/C53-I002) The routing outcome is an ablation result for a selected procedural subset, not evidence that orchestration is globally useful or obsolete.
- (SUMMARY/S-E0770) Cites arXiv:2604.02556 for a passive NF4 kernel watch and reported 2.0–2.2x kernel plus 1.54x end-to-end speedups. [L726-L727]
- (SUMMARY/S-E0771) Cites arXiv:2604.27891 as counter-evidence that in-context prompting can obsolete orchestration for procedural tasks. [L727]
- (SUMMARY/S-E0772) Requires no prerequisite for passive watching but needs RunPod for the counter-test pilot. [L728]
- (SUMMARY/S-E0773) Is parallel-safe with C-46 through C-52. [L729]
- (SUMMARY/S-E0774) Adds .elai_cc/docs/upstream-watch.md and tracks bitsandbytes. [L730]
- (SUMMARY/S-E0775) Adds cargo bench nf4_e2e. [L730]
- (SUMMARY/S-E0776) Adds --baseline-single-prompt. [L730]
- (SUMMARY/S-E0777) Adds a 30-task procedural-subset fixture. [L730]
- (SUMMARY/S-E0778) Adds counter_test_run and procedural_routing_decision. [L730]
- (SUMMARY/S-E0779) Smoke requires a pinned bitsandbytes record immediately; the counter-test pilot artifact is authored only after RunPod execution. [L731]

### C-54: SWE-bench Verified flagship (local/dual/hybrid, RunPod)

**Status**: planned (RunPod-dependent)
**Anchors**: ledgers C-004, SUMMARY | sample anchors: C-54 L219-L234; Summary L1263-L1274; C-54 L135-L141, L161-L166; C-54 L1-L7, L202-L217; C-54 L9-L24; C-54 L19-L22; C-54 L26-L29 | 48 deep atoms, 15 summary atoms | IDs: C-004/F-C54-031, C-004/D-C54-005, C-004/F-C54-001, C-004/F-C54-002, C-004/F-C54-003, …
**Composes with**: C-2, C-4, C-5, C-8, C-11, C-21, C-25, C-26, C-28, C-29, C-35, C-36, C-37, C-38, C-39, C-42, C-43, C-44, C-46, C-47, C-52, C-55, C-57, C-58. IX: C-004/X-014 → C-4, C-8, C-21, C-25, C-26, C-38, C-46, C-47, C-52; SUMMARY/S-E0782 → C-21, C-35, C-37, C-39, C-42, C-44; C-005/X-E07 → C-5, C-35, C-36, C-42; C-002/I08 → C-2, C-43, C-57; C-004/F-C54-002 → C-21, C-35, C-37, C-39, C-42; C-004/X-016 → C-4, C-8, C-46, C-52

- Superseded: (C-002/SUP-C16) NEWER-PLAN PRECEDENCE: C-57 description is target state relative to older C plan(s) C-54 for this conflict. Losing description retained at C-002/C16. Conflict: C-57 prerequisite and parallel statement conflict operationally. | Evidence: C-54 must be shipped and its number exist first, yet C-57 is called parallel-safe with C-54 while runs are sequential on the same RunPod. [C-57:L8-L10]. Planning can be parallel; the actual benchmark run cannot satisfy “C-5
- Superseded: (SUMMARY/SUP-S-59SLOTS) NEWER-PLAN PRECEDENCE: later coverage section (59 slots) supersedes header 35-session legend. Retained: SUMMARY/S-E0003 — The later coverage section supersedes the old count with 59 session slots: 52 CPU/cloud sessions (C-1 through C-33 and C-35 through C-53), six RunPod-dependent sessions (C-54 through C-59), and one externally blocked session (C-34). [L578-L587]
- (C-004/F-C54-031) File output includes corpus/manifest/configs, task-runner harness/test, three scorecards, methodology, results, CLI subcommand, and SSB schema. [C-54 L219-L234]
- (C-004/D-C54-005) Interrupted sweeps discard partials and restart fully; normal CI is excluded, and manual CLI launch is required. [Summary L1263-L1274; C-54 L135-L141, L161-L166]
- (C-004/F-C54-001) C-54 is an unchecked, RunPod-dependent flagship plan to publish ELAI’s first SWE-bench Verified measurement; it explicitly rejects differentiation claims without measured evidence. [C-54 L1-L7, L202-L217]
- (C-004/F-C54-002) Prerequisites are C-21’s ten-task seed, shipped C-35, at least one C-37 re-evaluation run, shipped C-39/C-42, and an accessible RunPod 4090/A100/H100 instance. [C-54 L9-L24]
- (C-004/F-C54-003) C-44 Composer 2 is required for the named Config C; otherwise that arm is downgraded to Sonnet-only. C-51 is explicitly not required. [C-54 L19-L22]
- (C-004/F-C54-004) Benchmark work is parallel-safe with C-55–C-58 only as separate runs; configurations sharing one RunPod execute sequentially, and the production binary is locked against C-35–C-53 prompt-path edits at run start. [C-54 L26-L29]
- (C-004/F-C54-005) The current ten-task seed expands to the full 500-task SWE-bench Verified corpus under `benchmarks/swe_bench/verified/`. [C-54 L59-L72]
- (C-004/F-C54-006) `manifest.toml` pins the SWE-bench version SHA for reproducibility. [C-54 L69-L72]
- (C-004/F-C54-007) `SwebenchHarness` clones each target repository at its pre-issue SHA, presents issue+repository to ELAI, collects/applies the emitted patch, runs hidden tests, and reports pass/fail. [C-54 L74-L78]
- (C-004/F-C54-008) Harness execution reuses C-11 sandbox-completeness for task isolation. [C-54 L74-L78]
- [config] (C-004/F-C54-009) Config A (`config-A-local-7B.toml`) uses backend `local_qwen_7b`, binding executor and auditor to the same local 7B model. [C-54 L80-L83]
- [config] (C-004/F-C54-010) Config B (`config-B-local-7B-7B.toml`) uses two distinct local 7B instances for executor and auditor; one-GPU hosts run them sequentially. [C-54 L83-L85]
- [config] (C-004/F-C54-011) Config C (`config-C-hybrid.toml`) uses `sonnet_46`, `composer2`, and `local_qwen_7b`, with Phase 4M default role routing and C-41 skill routing. [C-54 L85-L87]
- (C-004/F-C54-012) The three arms isolate infrastructure with one local model, the effect of a separate local auditor, and cloud-grade per-call quality under the same broader system. [C-54 L48-L57]
- [config] (C-004/F-C54-013) Each configuration performs one complete sweep of all 500 tasks; a task is run once per configuration with no cherry-picking. [C-54 L89-L105]
- (C-004/F-C54-014) Per-task records are `solved`, `time_s`, `llm_calls_total`, `tokens_in`, `tokens_out`, `cost_usd`, post-solve `mutation_score`, `MR_violations`, and `wiring_contract_pass`. [C-54 L89-L94]
- [gate/config] (C-004/F-C54-015) Per-config aggregates are solve rate, mean cost, p50/p99 time, and mean mutation score. [C-54 L94-L95]
- (C-004/F-C54-016) Each arm emits `workflow/sessions/{id}/swe_bench_run_{config}/scorecard.json`. [C-54 L94-L95]
- (C-004/F-C54-017) Auto-generated methodology records the SWE-bench SHA, RunPod/GPU/RAM, full configuration snapshots, C-35 gating and C-37 decoding state, wall-clock/cost, and timeout/error versus honest-failure outliers. [C-54 L97-L105]
- (C-004/F-C54-018) Methodology is generated from machine-readable config/run metadata and may not contain a human-editable narrative section that can drift from the numbers. [C-54 L126-L133, L161-L164]
- (C-004/F-C54-019) Public `swe_bench_verified_results.md` presents the three solve rates and costs, links methodology, and states honest losses/caveats. [C-54 L107-L115]
- (C-004/F-C54-020) The publication compares Devin/OpenHands/mini-SWE-agent/Claude Code only with each competitor’s cited version, date, published number, and methodology. [C-54 L107-L115]
- (C-004/F-C54-021) Results must identify benchmark SHA, hardware, and total cost; a bare “ELAI scored X%” is explicitly incomplete. [C-54 L161-L168]
- (C-004/F-C54-022) The mandatory honesty statement declares single-attempt pass@1, version SHA/hardware/cost, no cherry-picking, fixed TOML hyperparameters, seeded Config-A reproducibility within ±0 tasks, and Config-C cloud stochasticity within ±1–3 tasks. [C-54 L116-L121, L161-L169]
- (C-004/F-C54-023) Events are `swe_bench_run_started`, `swe_bench_task_completed`, `swe_bench_scorecard_emitted`, and `swe_bench_methodology_locked`. [C-54 L123-L124]
- (C-004/F-C54-024) Five no-GPU TDD cases prove pre-issue checkout, a known-good hidden-test pass, deliberately bad patch failure, official scorecard conformance, and metadata-driven methodology generation. [C-54 L126-L133]
- (C-004/F-C54-025) The benchmark is excluded from normal CI because of cost and is launched manually as `elai benchmark swe-bench --config ... --runpod-id <id>`. [C-54 L135-L136]
- [failure] (C-004/F-C54-026) If RunPod eviction, network failure, or another error interrupts a sweep, its partial scorecard is discarded and the entire configuration restarts. [C-54 L138-L141]
- (C-004/F-C54-027) A publishable number must come from one completed sweep; partials may never be published. [C-54 L161-L166]
- (C-004/F-C54-028) Seven deterministic groups verify 500 files, pinned SHA, three config files, feature-gated harness build, four action types, three post-run scorecards, and methodology/results artifacts. [C-54 L171-L200]
- (C-004/F-C54-029) Acceptance separately requires corpus/config/harness/sanity completion, all three 500-task scorecards, generated methodology, authored results with honesty statement, all four events, and no partial/cherry-picked publication. [C-54 L202-L217]
- (C-004/F-C54-030) C-54 consumes C-28 mutation and C-29 metamorphic metrics, all three backend tiers, C-11 sandboxing, and the SSB bus. [C-54 L143-L160]
- (C-004/F-C54-032) Closure annotates the C-21 verification report for the full corpus and unlocks C-55/C-56 plus eventual C-59 aggregation. [C-54 L236-L248]
- (C-004/F-C54-033) The prerequisite permits downgrading Config C to Sonnet-only when C-44 is unavailable, but the objective, config schema, consumed backend list, and acceptance require Sonnet+Composer2+local; it is unclear whether a downgraded run may close C-54 as “three configs.” [C-54 L3-L5, L19-L20, L85-L87, L154-L157, L212-L213]
- [config] (C-004/F-C54-034) Config A says executor and auditor both bind to local 7B, while Config B’s distinction is two separate instances; process/model-instance sharing and context isolation in Config A are not defined. [C-54 L80-L85]
- (C-004/F-C54-035) Hidden-test secrecy/access, credential handling for cloud backends/RunPod, artifact redaction, and whether repository network access stays enabled during execution are not specified beyond C-11 sandbox reuse. [C-54 L74-L78, L97-L105]
- (C-004/F-C54-036) Timeout values, failed-vs-errored scoring, percentile estimator, cost accounting source, and stochastic seeds/temperatures are recorded in methodology but not fixed by this plan. [C-54 L89-L105, L116-L121]
- (C-004/F-C54-037) “Scorecard JSON conforms to SWE-bench official format” is required, but the added ELAI-only fields and mapping into the official schema are not defined. [C-54 L89-L95, L126-L133]
- (C-004/F-C54-038) “Three-run protocol” means one full sweep for each of three configurations, not three repetitions of each task/configuration. [C-54 L89-L105]
- (C-004/F-C54-039) All acceptance boxes are unchecked and RunPod is a hard external gate; the source contains no published score, spend, or completed run. [C-54 L23-L24, L202-L217]
- (C-004/D-C54-001) C-37 need only have been run, not necessarily shipped; C-44 absence permits a Sonnet-only downgrade, which conflicts with the later fixed Config-C acceptance. [Summary L1259; C-54 L14-L20, L85-L87, L212-L213]
- [config] (C-004/D-C54-002) Config A binds both roles to one local model, Config B uses distinct instances (sequential on one GPU), and Config C applies Phase-4M/C-41 routing. [Summary L1265-L1269; C-54 L80-L87]
- [gate] (C-004/D-C54-003) Per-task and aggregate metric schemas, hardware/config/gating/outlier methodology fields, and once-per-config run policy are omitted. [Summary L1269-L1274; C-54 L89-L105]
- (C-004/D-C54-004) Methodology is auto-generated from machine data; `results.md` is separately authored. The summary’s wording that methodology/results are generated from machine-readable inputs overextends the anti-drift rule. [Summary L1271-L1274; C-54 L97-L121, L161-L164, L214-L215]
- (C-004/D-C54-006) The honesty statement fixes pass@1/version/hardware/cost/no-cherry-pick language and distinguishes ±0 seeded local from ±1–3 hybrid variation. [Summary L1272-L1274; C-54 L116-L121]
- (C-004/D-C54-007) All acceptance remains unchecked; no score, cost, or completed RunPod run exists in the source. [Summary L1253-L1274; C-54 L23-L24, L202-L217]
- (SUMMARY/S-E0780) Begins Wave 9’s RunPod-dependent publication work and is labeled FLAGSHIP. [L733-L735]
- (SUMMARY/S-E0781) Treats SWE-bench Verified as the industry standard and says ELAI has no published number under the maintainer’s honest assessment section 8. [L735-L736]
- (SUMMARY/S-E0782) Depends on expanding C-21’s corpus from ten to 500 tasks and on shipped C-35, C-37, C-39, C-42, and optionally C-44 for the hybrid C arm. [L737]
- (SUMMARY/S-E0783) Requires RunPod. [L737]
- (SUMMARY/S-E0784) Is parallel-safe with C-55 through C-58 but must run sequentially when sharing one RunPod. [L738]
- (SUMMARY/S-E0785) Adds a 500-task corpus at a pinned SHA. [L739]
- [config] (SUMMARY/S-E0786) Adds configuration A using local 7B. [L739]
- [config] (SUMMARY/S-E0787) Adds configuration B using dual local 7B. [L739]
- [config] (SUMMARY/S-E0788) Adds configuration C using hybrid Sonnet, Composer 2, and local. [L739]
- (SUMMARY/S-E0789) Adds SwebenchHarness. [L739]
- (SUMMARY/S-E0790) Produces three scorecard.json files. [L739]
- (SUMMARY/S-E0791) Auto-generates methodology.md. [L739]
- (SUMMARY/S-E0792) Publishes results.md with a mandatory honesty statement. [L739]
- (SUMMARY/S-E0793) Adds swe_bench_run_started, swe_bench_task_completed, swe_bench_scorecard_emitted, and swe_bench_methodology_locked. [L739]
- [config] (SUMMARY/S-E0794) Acceptance requires all three configurations to finish, forbids cherry-picking, and generates methodology/results from machine-readable inputs. [L740]

### C-55: Aider Polyglot full run and draft PR submission (RunPod)

**Status**: planned (RunPod-dependent)
**Anchors**: ledgers C-006, SUMMARY | sample anchors: C-55 L92-L95; C-55 L128-L138; C-55 L1-L7; C-55 L9-L11; C-55 L13-L16; C-55 L18-L28 | 33 deep atoms, 12 summary atoms | IDs: C-006/C55-E016, C-006/C55-E020, C-006/C55-E001, C-006/C55-E002, C-006/C55-E003, …
**Composes with**: C-1, C-6, C-40, C-44, C-51, C-54, C-56, C-57, C-58. IX: C-006/C55-X001 → C-6, C-40, C-44, C-51; C-001/X-009 → C-1, C-51, C-58; C-006/C55-I002 → C-6; C-006/C55-E015 → C-40; SUMMARY/S-E0796 → C-40, C-51; SUMMARY/S-E0798 → C-54, C-56, C-58

- (C-006/C55-E016) CLI reachability is elai benchmark aider-polyglot --config <name> --runpod-id <id> and --submit only after both configs complete. [C-55 L92-L95]
- [config] (C-006/C55-E020) File surface adds two configs, two post-run scorecards, methodology/results, benchmark CLI/submit, and three SSB actions. [C-55 L128-L138]
- (C-006/C55-E001) Objective: run all 225 Aider Polyglot problems under local-32B and hybrid configurations, compare against public raw-model/cost baselines, and submit leaderboard data. [C-55 L1-L7]
- (C-006/C55-E002) Header requires shipped C-51 harness plus live RunPod; declared parallel-safe with C-54/C-56/C-57/C-58 but sequential when sharing one RunPod. [C-55 L9-L11]
- (C-006/C55-E003) Reference numbers are a 2026-05-17 snapshot: GPT-5 high 88.0%/$29.08, GPT-5 low 81.3%/$10.37, and ollama qwen2.5-coder:32b 72.9%. [C-55 L13-L16]
- (C-006/C55-E004) Hypothesis: ELAI plus the same local 32B beats raw ollama/Qwen at per-dollar performance; a loss means the local-only “infrastructure substitutes for parameters” thesis needs revision. [C-55 L18-L28]
- [config] (C-006/C55-E005) Config L uses backends=[local_qwen_32b] and is the direct local comparison; Config H uses [sonnet_46, composer2, local_qwen_32b] and is compared with GPT-5 high by cost per task. [C-55 L30-L34]
- (C-006/C55-E006) Immediately before submission, fetch aider.chat/docs/benchmarks afresh for JSON shape, required fields, and public PR template. [C-55 L36-L45, L115]
- [config] (C-006/C55-E007) Author config-L-local-32B.toml and config-H-hybrid.toml under .elai_cc/benchmarks/aider_polyglot/configs. [C-55 L47-L50]
- (C-006/C55-E008) Execute 225 x 2 = 450 task runs and collect solved, wall_time_s, llm_calls, tokens_in, tokens_out, cost_usd, and repaired_after_fail per task; the last field signals C-31/C-32 StructuralRetry. [C-55 L51-L54]
- [config] (C-006/C55-E009) Config L cost is GPU hourly rate multiplied by wall-time fraction, with RunPod A100/H100/4090 rate documented. [C-55 L56-L59]
- [config] (C-006/C55-E010) Config H cost combines the local GPU share and metered Sonnet 4.6 plus Composer 2 API cost. [C-55 L59-L60]
- (C-006/C55-E011) Both cost calculations must match Aider's published methodology for apples-to-apples comparison and be machine-derived from time/token inputs, never estimated by a person. [C-55 L61-L62, L96-L98]
- (C-006/C55-E012) Save exact-format scorecards at workflow/sessions/{id}/aider_polyglot_run_L/scorecard.json and ..._H/scorecard.json. [C-55 L64-L65, L80-L83]
- (C-006/C55-E013) Fork Aider-AI/aider, add leaderboard entries, and use C-40 LeaderboardPRComposer to open a PR whose body cites methodology, ELAI commit SHA, and elai_cc.toml versions. [C-55 L67-L70]
- (C-006/C55-E014) Generate workflow/sessions/{id}/aider_polyglot_methodology.md and .elai_cc/docs/aider_polyglot_results.md. [C-55 L72-L73]
- (C-006/C55-E015) Source action identifiers are polyglot_submission_started, polyglot_submission_completed, and polyglot_pr_opened; the summary's polyglot_submission_pr_opened is incorrect. polyglot_pr_opened consumes C-40 github_pr_created. [C-55 L75-L76, L80-L86]
- (C-006/C55-E017) Public PR must remain DRAFT and never auto-merge; results must cite methodology and the public PR URL. [C-55 L96-L100, L121]
- (C-006/C55-E018) Deterministic checks are artifact-existence checks for both configs/scorecards/results and grep for all three SSB actions. [C-55 L102-L111]
- (C-006/C55-E019) Acceptance requires 225-problem scorecards for both configs, machine-derived cost, both docs, a public draft PR, all actions, honest local comparison to 72.9%, and honest hybrid comparison to GPT-5 low/high at matched cost. [C-55 L113-L126]
- (C-006/C55-E021) C-55 is explicitly RUNPOD-DEPENDENT and every acceptance box is open; scorecards/docs are marked post-run. [C-55 L1, L113-L138]
- (C-006/C55-E022) A draft submission is not itself a public leaderboard entry: actual publication depends on external Aider review/merge. [C-55 L67-L70, L99-L100, L121, L140-L145]
- (C-006/C55-E023) C-55 is intended to pair with C-54 so ELAI has an issue-resolution axis and a cost-per-task axis. [C-55 L140-L145]
- (C-006/C55-A001) Exact Aider JSON/schema/PR fields are deliberately deferred to a fresh fetch, so they cannot be extracted from this source. [C-55 L36-L45, L64-L70]
- (C-006/C55-A002) The quoted leaderboard values are historical and may change; only the submission protocol is explicitly fresh-fetched, although comparisons also need a dated baseline snapshot. [C-55 L13-L16, L43-L45]
- (C-006/C55-A003) “Same local 32B” does not pin quantization, serving stack, prompt format, hardware, context length, or commit, so raw-model comparability needs the methodology artifact. [C-55 L20-L34]
- (C-006/C55-A004) Seed, task order, retry cap, timeout/failure accounting, resumed-run behavior, idempotency, duplicate submissions, and anti-cherry-pick controls are not specified.
- (C-006/C55-A005) Public fork/PR is an external write, but this plan does not add an explicit user approval gate beyond invoking --submit and C-40's draft behavior.
- (C-006/C55-A006) Sonnet/Composer credentials, rate limits, pricing snapshot, error fallback, and data-sharing/privacy policy for benchmark tasks are unspecified.
- (C-006/C55-I001) repaired_after_fail is observability for structural recovery, not a scoring dimension unless the fresh Aider format includes it.
- (C-006/C55-I002) Honest better/worse/equal reporting means there is no adoption/pass threshold; completing a noncompetitive run still closes the evidence task.
- (SUMMARY/S-E0795) Anchors comparison to Aider’s public leaderboard and the recorded ollama qwen2.5-coder:32b baseline of 72.9%. [L742-L743]
- (SUMMARY/S-E0796) Depends on C-51 for the harness and C-40 for PR submission. [L744]
- (SUMMARY/S-E0797) Requires RunPod. [L744]
- (SUMMARY/S-E0798) Is parallel-safe with C-54 and C-56 through C-58. [L745]
- [config] (SUMMARY/S-E0799) Adds configuration L for local 32B. [L746]
- [config] (SUMMARY/S-E0800) Adds configuration H for hybrid. [L746]
- (SUMMARY/S-E0801) Produces a scorecard.json for each configuration. [L746]
- (SUMMARY/S-E0802) Produces methodology and results documents. [L746]
- (SUMMARY/S-E0803) Opens a public draft PR against Aider-AI/aider. [L746]
- (SUMMARY/S-E0804) Adds polyglot_submission_started, polyglot_submission_completed, and polyglot_submission_pr_opened. [L746]
- (SUMMARY/S-E0805) Acceptance requires 225-problem scorecards per configuration. [L747]
- (SUMMARY/S-E0806) Results must honestly compare against the 72.9% baseline and document cost per task versus GPT-5. [L747]

### C-56: DKG/symbol-tool evaluation on 31-repo set (RunPod)

**Status**: planned (RunPod-dependent)
**Anchors**: ledgers C-001, SUMMARY | sample anchors: C56:L74-90; C56:L126-138; C56:L14-18; C56:L39-43, L110-114; C56:L45-47; C56:L49-50 | 19 deep atoms, 12 summary atoms | IDs: C-001/C56-012, C-001/C56-016, C-001/C56-002, C-001/C56-004, C-001/C56-005, …
**Composes with**: C-1, C-2, C-6, C-17, C-18, C-22, C-24, C-30, C-38, C-41, C-45, C-51, C-54, C-55, C-57, C-58, C-59. IX: SUMMARY/S-E0808 → C-38; SUMMARY/S-E0810 → C-54, C-55, C-57, C-58; C-001/X-011 → C-1, C-45, C-58; C-001/X-006 → C-1, C-17, C-24, C-30, C-58; C-001/X-012 → C-1, C-2, C-6, C-18, C-22, C-30, C-41, C-51, C-58, C-59; C-001/X-010 → C-1, C-58, C-59

- Superseded: (C-001/SUP-A-020) NEWER-PLAN PRECEDENCE: C-58 description is target state relative to older C plan(s) C-45, C-56 for this conflict. Losing description retained at C-001/A-020. Conflict: [conflict] C-58 must use the later explicit sequential/back-to-back rule despite earlier “in parallel” wording; C-56 data and C-45 attribution are conditional dependencies despite parallel/recommended labels. (C58:L8-35, L65-121)
- (C-001/C56-012) CLI reachability is elai benchmark codebase-memory --runpod-id; outputs are CodebaseMemoryAdapter, scorer, session scorecard/methodology, and repository results doc. (C56:L74-90)
- (C-001/C56-016) Adds the corpus, manifest, queries, adapter/scorer, three result artifacts, paper artifact, CLI command, and three SSB schema types. (C56:L126-138)
- (C-001/C56-002) The cited paper figure is 83% answer quality versus a 92% file-explorer agent, with tenfold fewer tokens and 2.1-fold fewer tool calls across 31 repositories—not simply an unqualified 83% target. (C56:L14-18)
- (C-001/C56-004) paper/source ingestion captures the exact 31-repo manifest, query set, answer rubric, and produces codebase-memory-interface.md. (C56:L39-43, L110-114)
- (C-001/C56-005) Vendor exactly 31 repositories at pinned SHAs under benchmarks/codebase_memory/repos with manifest.toml. (C56:L45-47)
- (C-001/C56-006) Vendor the paper's N-per-repo queries and ground-truth answers; N is not fixed in the plan. (C56:L49-50)
- (C-001/C56-007) CodebaseMemoryAdapter executes compute_repo_map then symbol_impact/flow/context as needed and records emitted answer, tokens, and tool calls. (C56:L52-54)
- [gate] (C-001/C56-009) Every repository and aggregate report includes answer-quality percentage, mean tokens/query, and mean tool calls/query, cross-tabulated against published baselines. (C56:L60-62)
- (C-001/C56-010) Hub-detection and caller-ranking are separately scored against a claimed “matches or exceeds the explorer on 19 of 31 languages” result, with symbol_impact treated as the matching ELAI surface. (C56:L64-67)
- (C-001/C56-011) Produce methodology/results like C-54 plus codebase_memory_query_run, codebase_memory_answer_scored, and codebase_memory_scorecard_emitted. (C56:L69-72)
- [security] (C-001/C56-013) Judge is fixed per run, fixed judge+seed must make scoring deterministic, per-repo results cannot be hidden, and underperformance must be prominent with candidate improvements. (C56:L91-97)
- [gate] (C-001/C56-014) Deterministic checks prove 31 directories, manifest, adapter, current-session scorecard, results doc, and three events; they do not themselves validate metric values. (C56:L99-108)
- [gate] (C-001/C56-015) Acceptance requires source artifact, pinned corpus/queries/ground truth, adapter/scorer, completed 31-repo per-repo+aggregate run, separate specialized scores, three events, cross-tabulated results, conditional improvements, and methodology. (C56:L110-124)
- (C-001/C56-008) AnswerQualityScorer matches the paper's rubric, which may be model-judge or string similarity depending on query; Sonnet 4.6 is fixed/documented for fairness. (C56:L56-58)
- (C-001/C56-017) Despite “head-to-head/A-B” wording, the plan runs only ELAI and cross-tabulates published Codebase-Memory/file-explorer numbers; denominator definitions for “10×” and “2.1×,” model/version control for ELAI answers, and repeated-run variance are unstated. (C56:L1-7, L26-30, L52-62)
- [status] (C-001/C56-001) C-56 is RunPod-dependent after B-29/B-30/C-38 and parallel with C-54/C-55/C-57/C-58; it evaluates ELAI DKG/symbol tools on a published 31-repository set. (C56:L1-12)
- [status] (C-001/C56-003) Both outcomes are valid: meeting the published figures supports publication; missing them must identify whether DKG, CatRAG, A-RAG, or reranker is weaker. (C56:L20-30)
- (SUMMARY/S-E0807) Cites arXiv:2603.27277 for 83% answer quality, ten times fewer tokens, and 2.1 times fewer tool calls across 31 real repositories. [L749-L750]
- (SUMMARY/S-E0808) Depends on B-29, B-30, and C-38. [L751]
- (SUMMARY/S-E0809) Requires RunPod. [L751]
- (SUMMARY/S-E0810) Is parallel-safe with C-54, C-55, C-57, and C-58. [L752]
- (SUMMARY/S-E0811) Vendors 31 repositories at pinned SHAs. [L753]
- (SUMMARY/S-E0812) Adds a query corpus and ground-truth answers. [L753]
- (SUMMARY/S-E0813) Adds CodebaseMemoryAdapter. [L753]
- (SUMMARY/S-E0814) Adds AnswerQualityScorer using Sonnet 4.6 as judge. [L753]
- (SUMMARY/S-E0815) Adds specialized hub-detection and caller-ranking queries. [L753]
- [gate] (SUMMARY/S-E0816) Produces per-repository and aggregate metrics. [L753]
- (SUMMARY/S-E0817) Adds codebase_memory_query_run, codebase_memory_answer_scored, and codebase_memory_scorecard_emitted. [L753]
- (SUMMARY/S-E0818) The target is to meet or beat all three paper figures; failure must instead yield an improvement-target list in results.md. [L754]

### C-57: OpenHands Index/SWE-Cycle/SWE Atlas cost-amortized eval (RunPod)

**Status**: planned (RunPod-dependent)
**Anchors**: ledgers C-002, SUMMARY | sample anchors: C-57:L98-L100; C-57:L36-L39; C-57:L59-L65; C-57:L98-L105; C-57:L120-L132; C-57:L1-L6; C-57:L8-L10; C-57:L12-L21; C-57:L23-L34 | 24 deep atoms, 9 summary atoms | IDs: C-002/C57-E14, C-002/SD-C-57-01, C-002/C57-E01, C-002/C57-E02, C-002/C57-E03, …
**Composes with**: C-1, C-2, C-5, C-27, C-42, C-43, C-44, C-50, C-54, C-56, C-58. IX: C-002/SD-C-57-04 → C-54; C-002/I10 → C-2; SUMMARY/S-E0822 → C-54, C-56, C-58; C-002/C57-E02 → C-54; C-002/I01 → C-1, C-2, C-5, C-27, C-42, C-43, C-50; SUMMARY/S-E0820 → C-54

- Superseded: (C-002/SUP-C16) NEWER-PLAN PRECEDENCE: C-57 description is target state relative to older C plan(s) C-54 for this conflict. Losing description retained at C-002/C16. Conflict: C-57 prerequisite and parallel statement conflict operationally. | Evidence: C-54 must be shipped and its number exist first, yet C-57 is called parallel-safe with C-54 while runs are sequential on the same RunPod. [C-57:L8-L10]. Planning can be parallel; the actual benchmark run cannot satisfy “C-5
- (C-002/C57-E14) CLI entrypoints are `elai benchmark openhands-index|swe-cycle|swe-atlas --runpod-id <id>` and suite mode `elai benchmark suite --runpod-id <id>`. [C-57:L98-L100]
- (C-002/SD-C-57-01) Addition: The index omits exact default composition (Sonnet 4.6/Composer 2/local auditor), optional local-only arm, pass@1/no-cherry-picking discipline, CLI forms, external-baseline citation rule, and RunPod cost gate. [C-57:L36-L39; C-57:L59-L65; C-57:L98-L105; C-57:L120-L132]
- [status] (C-002/C57-E01) Run three public benchmarks in one RunPod session to amortize cost: OpenHands Index (issue/greenfield/frontend/testing), SWE-Cycle (FullCycle), and SWE Atlas (Q&A/test writing/refactoring). The plan is explicitly RunPod-dependent. [C-57:L1-L6]
- (C-002/C57-E02) C-54's primary SWE-bench Verified number must exist first; the run is described as parallel-safe with C-54/55/56/58 but sequential on the same RunPod. [C-57:L8-L10]
- (C-002/C57-E03) Recorded reference axes include OpenHands' 72% internal SWE-bench baseline, SWE-Cycle's isolated→FullCycle solve-rate drop, and SWE Atlas' 124 Q&A, 90 test-writing, and 70 refactoring tasks. [C-57:L12-L21]
- (C-002/C57-E04) SWE-Cycle tests the full PLANNING→EXECUTING→AUDITING→MEMORIZING FSM across its own iterative outputs, including MSI/bubble-up; SWE Atlas measures local-vs-frontier gap and hybrid closure; OpenHands approximates developer workflows. [C-57:L23-L34]
- [config] (C-002/C57-E05) Default is one hybrid config per benchmark—Sonnet 4.6 designer, Composer 2 executor, local auditor. A local-only arm is added only if a useful published comparator exists; exhaustive 3×3 runs are rejected as over-budget. [C-57:L36-L39; C-57:L59-L62]
- (C-002/C57-E06) Ingest SWE-Cycle/SWE Atlas papers and freshly fetch/document OpenHands methodology. [C-57:L41-L50]
- (C-002/C57-E07) Add `openhands_index_adapter.rs`, `swe_cycle_adapter.rs`, and `swe_atlas_adapter.rs`; each loads tasks, presents them to ELAI, collects patches/answers, and scores with the benchmark's official rubric. [C-57:L52-L57]
- (C-002/C57-E08) Run discipline is single-attempt pass@1, no cherry-picking, and one scorecard JSON per benchmark. [C-57:L64-L65]
- (C-002/C57-E09) OpenHands follows published per-axis scoring; SWE-Cycle reports the solve-rate curve by cycle; SWE Atlas uses rubric scoring via Sonnet 4.6 or the paper's judge if released. [C-57:L67-L71]
- (C-002/C57-E10) Produce one combined methodology/results treatment for the three runs plus three separate scorecards, using one RunPod instance. [C-57:L73-L74; C-57:L89-L94]
- (C-002/C57-E11) Required findings compare ELAI's FullCycle drop with the paper, ELAI hybrid with same-class Llama/Qwen SWE Atlas baselines, and ELAI's OpenHands axes with published axes. [C-57:L76-L82]
- (C-002/C57-E12) Add `multi_benchmark_run_started`, per-axis/per-benchmark `multi_benchmark_axis_completed`, and `multi_benchmark_scorecard_emitted`. [C-57:L84-L85]
- (C-002/C57-E13) Harnesses consume all three backend tiers after C-44 and the SSB bus. [C-57:L95-L97]
- [gate] (C-002/C57-E15) Per-axis—not aggregate-only—scores are mandatory; SWE-Cycle must expose each cycle; all external comparisons cite version and date. [C-57:L101-L105]
- [gate] (C-002/C57-E16) Acceptance requires three completed runs, three events, per-axis/curve/rubric outputs, honest combined results, all Task-7 findings, and total RunPod cost below three independent-run costs. [C-57:L120-L132]
- [lifecycle] (C-002/C57-E17) Vendored corpora are added for all three (SWE Atlas only if released); post-run artifacts live under the active session, while public results live at `.elai_cc/docs/multi_benchmark_results.md`. [C-57:L134-L150]
- [status] (C-002/C57-E18) Closure expands evidence beyond C-54 and feeds C-59's aggregate comparison. [C-57:L152-L157]
- (C-002/SD-C-57-02) Addition: Each adapter performs load/present/collect/official-score, and outputs must expose OpenHands axes, SWE-Cycle's curve, and SWE Atlas rubrics rather than aggregates. [C-57:L52-L82]
- (C-002/SD-C-57-03) Addition: C-44's three backend tiers are an additional consumed dependency; C-59 is the downstream aggregator. [C-57:L95-L97; C-57:L152-L157]
- (C-002/SD-C-57-04) Correction/precision: “Parallel-safe with C-54” applies at most to preparation; the benchmark run requires C-54's number first and shares one sequential RunPod. [C-57:L8-L10]
- (SUMMARY/S-E0819) Uses the January 2026 OpenHands Index article, arXiv:2605.13139 SWE-Cycle, and arXiv:2605.08366 SWE Atlas. [L756-L757]
- (SUMMARY/S-E0820) Depends on completion of C-54 so the primary number exists first. [L758]
- (SUMMARY/S-E0821) Requires RunPod. [L758]
- (SUMMARY/S-E0822) Is parallel-safe with C-54 through C-56 and C-58. [L759]
- (SUMMARY/S-E0823) Adds adapters for openhands_index, swe_cycle, and swe_atlas. [L760]
- (SUMMARY/S-E0824) Produces one scorecard.json per adapter. [L760]
- (SUMMARY/S-E0825) Produces a combined methodology/results document. [L760]
- (SUMMARY/S-E0826) Adds multi_benchmark_run_started, multi_benchmark_axis_completed, and multi_benchmark_scorecard_emitted. [L760]
- (SUMMARY/S-E0827) Acceptance requires an OpenHands Index scorecard, a per-cycle solve-rate curve for SWE-Cycle, and per-rubric scores for SWE Atlas. [L761]

### C-58: AgentForge multi-role and RRK reranker evaluation (RunPod)

**Status**: planned (RunPod-dependent)
**Anchors**: ledgers C-001, SUMMARY | sample anchors: C58:L21-32; C58:L108-115; C58:L158-172; C22:L122-200, L255-283; C2:L66-70, L103-109, L187-192; C22:L8-12, L130-179 | 103 deep atoms, 13 summary atoms | IDs: C-001/C58-003, C-001/C58-014, C-001/C58-018, C-001/X-005, C-001/A-005, …
**Composes with**: C-0, C-1, C-2, C-3, C-4, C-6, C-7, C-8, C-10, C-16, C-17, C-19, C-22, C-24, C-30, C-45, C-51, C-54, C-55, C-56, C-57, C-59. IX: C-001/C58-001 → C-45, C-54, C-57; C-001/X-002 → C-1, C-2; C-001/X-006 → C-1, C-17, C-24, C-30, C-56; C-001/X-006 → C-17, C-24, C-30, C-56; C-001/X-011 → C-1, C-45, C-56; C-001/X-010 → C-1, C-56, C-59

- Superseded: (C-001/SUP-A-001) NEWER-PLAN PRECEDENCE: C-5 description is target state relative to older C plan(s) C-0 for this conflict. Losing description retained at C-001/A-001. Conflict: [conflict] C-0 claims 59 slots/files but omits C-5 from Sessions; do not invent its plan from closure-map shorthand. (C0:L17-349)
- Superseded: (C-001/SUP-A-012) NEWER-PLAN PRECEDENCE: C-23 description is target state relative to older C plan(s) C-21, C-22 for this conflict. Losing description retained at C-001/A-012. Conflict: [conflict] C-22's parallel-file explanation misidentifies C-21/C-23 ownership, and the Tauri command path is inconsistent. (C22:L8-12, L130-179)
- Superseded: (C-001/SUP-A-014) NEWER-PLAN PRECEDENCE: C-30 description is target state relative to older C plan(s) C-0, C-15 for this conflict. Losing description retained at C-001/A-014. Conflict: [conflict] C-30's std HashMap violates C-0/C-15's production FxHashMap policy. (C30:L114-124; C0:L78-81)
- Superseded: (C-001/SUP-A-020) NEWER-PLAN PRECEDENCE: C-58 description is target state relative to older C plan(s) C-45, C-56 for this conflict. Losing description retained at C-001/A-020. Conflict: [conflict] C-58 must use the later explicit sequential/back-to-back rule despite earlier “in parallel” wording; C-56 data and C-45 attribution are conditional dependencies despite parallel/recommended labels. (C58:L8-35, L65-121)
- (C-001/C58-003) ELAI roles are Executor, Auditor, Researcher, Designer, PromptEngineer, ContextCompiler, MemoryWriter, MediaValidator; AgentForge roles are Planner, Coder, Tester, Debugger, Critic; compare solve rate and cost/task on the same corpus/hardware. (C58:L21-32)
- (C-001/C58-014) Consumers are SWE-bench Lite, optionally/conditionally C-45 attribution, C-56 query corpus, and knowledge reranker; CLI has separate head-to-head and reranker-eval commands with runpod-id. (C58:L108-115)
- (C-001/C58-018) Adds vendor, adapters/reranker trait modules, two scorecards, combined methodology/results, two paper artifacts, two CLI commands, four SSB schema types, and a knowledge module export. (C58:L158-172)
- (C-001/X-005) C-22 turns the feasibility path from None/unwired into production data, two new events, panel rendering, explicit child termination, and measured phase_4F evidence; C-19 conditionally owns the adversarial tests. (C22:L122-200, L255-283)
- [status] (C-001/A-005) C-2 leaves the AgentStatusChanged owner and anti-spiral payload-struct granularity open; Cosmos tick can also be absent from the smoke window. (C2:L66-70, L103-109, L187-192)
- (C-001/A-012) C-22's parallel-file explanation misidentifies C-21/C-23 ownership, and the Tauri command path is inconsistent. (C22:L8-12, L130-179)
- (C-001/A-017) C-41 omits scoring weights/threshold/ties, cache design, semver negotiation, fall-through chosen_skill semantics, panel path, and manual-override authorization/evidence. (C41:L68-128, L174, L196)
- (C-001/A-018) C-51 is infrastructure-only and publication is C-55; a mock five-task dry-run is allowed. Whether a GPU-present non-dry CLI may nevertheless run the full suite before C-55 is internally unclear. (C51:L1-7, L78-82, L120-149)
- (C-001/C58-002) Plan-asserted baselines are AgentForge 40.0% SWE-bench Lite/26–28 points over single agents and RRK 8B at 3–18× the speed of 0.6–4B rerankers. (C58:L12-17)
- (C-001/C58-004) If AgentForge wins per cost, C-45 contribution data identifies ELAI roles that did not earn cost. (C58:L34-35)
- (C-001/C58-005) RRK's claimed speed does not override ELAI's <50ms target; the pilot must measure both retrieval quality contribution and latency at typical query volume. (C58:L37-46)
- (C-001/C58-006) Separate paper/source ingestion grounds AutoCodeAI and RRK and produces two interface artifacts. (C58:L48-59, L69-75, L168-169)
- [security] (C-001/C58-007) Vendor AutoCodeAI at a pinned SHA with LICENSE and run it through an isolation harness on the same SWE-bench Lite set. (C58:L61-63, L116-121)
- [config] (C-001/C58-009) Add rrk_reranker Cargo feature, load the released 8B checkpoint, and expose it as a config-flagged alternative in knowledge/reranker.rs. (C58:L71-75)
- [gate] (C-001/C58-010) Measure p99 on top-K-vector query shapes; reject only when latency is >100ms, described as 2× the 50ms target. Thus exactly 100ms is not explicitly rejected. (C58:L77-80, L116-119)
- (C-001/C58-011) On a held-out set such as C-56 queries, compare top-K precision and recall for current and RRK. (C58:L82-84)
- (C-001/C58-012) Ship RRK only as opt-in when latency is acceptable and quality improves by at least five percentage points; otherwise archive it as measured-not-worth-it. Default never auto-flips. (C58:L86-88, L116-120)
- (C-001/C58-013) Produce two scorecards, combined methodology/results, and head_to_head_run_started, head_to_head_arm_completed, reranker_eval_run, reranker_eval_decision. (C58:L90-107)
- [gate] (C-001/C58-015) Static checks prove vendor/LICENSE, adapter, RRK loader, two current-session scorecards, and four event names; no executable test command is specified. (C58:L123-133)
- [gate] (C-001/C58-016) AgentForge acceptance requires completed run/scorecard and conditionally either a C-45-informed consolidation list or confirmation that eight roles earn cost. (C58:L135-145)
- [gate] (C-001/C58-017) RRK acceptance requires loader/feature, p99, precision/recall delta, applied adoption/archive matrix, gated/default-unchanged adoption, all events, and combined docs. (C58:L147-156)
- (C-001/X-001) C-0 places C-2 after C-1's event pattern and before C-3/C-4/C-7/C-8/C-10/C-16. C-2 supplies persisted orchestration rows; later sessions query, render, complete producer logic, and schedule Cosmos/anti-spiral work. (C0:L36-61, L83-98; C2:L150-180, L284-294)
- (C-001/X-002) C-2's append-only events and latest-agent reduction form a live dashboard loop: producer→SSB row→viewer reduction/query→Tauri→Bubble/Orchestration/Cosmos UI. The sources do not define subscription versus polling, retention, or simultaneous-role ordering. (C2:L86-197)
- (C-001/X-003) C-6 controls the presentation and input behavior of live task streams but creates no backend cancellation API; it must discover and reuse the stop/submit mechanisms produced elsewhere. (C6:L107-166, L256-267)
- (C-001/X-004) C-18 provides regression proof for C-2 event serialization/viewer behavior and for symbol/graph/artifact/DAG surfaces that later plans consume; it deliberately adds tests rather than runtime behavior. (C18:L17-22, L38-170)
- (C-001/X-006) C-30 consumes C-17 indexing and C-24 DKG tools to size surgical changes; C-56 later evaluates that DKG/tool architecture over 31 repositories. Passing C-30's functional contracts does not substitute for C-56's quality/cost study. (C0:L137-174, L310-313; C30:L73-149; C56:L52-67)
- (C-001/X-007) Project detection can create the index needed by SurgicalDecomposer, after which its DKG AffectedSet feeds ordered steps/regression gates; if indexing is rejected, the source only promises symbol_impact fallback, so full surgical equivalence is not established. (C30:L73-113)
- (C-001/X-008) C-41 selects prompt scaffolds and feeds top candidates to prompt-engineer; C-44 is expected to use skill_selected for backend routing, while C-51's benchmark adapter already invokes prompt-engineer and therefore exercises whichever registry/backend stack is active. (C41:L73-84, L207-213; C51:L49-54)
- (C-001/X-009) C-51 is the reproducible six-language harness boundary; C-55 alone binds real GPU models, runs 225 tasks, and publishes the Aider draft PR. (C0:L278-281, L305-308; C51:L17-27, L132-149)
- (C-001/X-010) C-56 and C-58 generate machine-readable scorecards/methodology/results for C-59. Negative results are retained as improvement targets or measured-not-worth-it decisions rather than omitted. (C0:L325-328; C56:L91-124; C58:L86-107)
- (C-001/X-011) C-58's role comparison can consume C-45 credit attribution, and its reranker quality arm can consume C-56 queries; both dependencies are soft/conditional in prose, so scheduling them truly in parallel requires alternate context/data. (C58:L8-10, L34-35, L82-111)
- (C-001/X-012) Evidence maturity rises from static manifest/checks (all plans), to deterministic contract tests (C-2/C-6/C-18/C-22/C-30/C-41/C-51), to live cloud/GPU measurement (C-22/C-30/C-56/C-58), to C-59 publication. A plan checkbox is not equivalent to any later stage. (C0:L1-15, L293-328)
- (C-001/D-001) C-0 is not merely a non-implementation index: it is a feature/status/dependency manifest with 58 explicit session rows, closure maps, estimates, and provenance. Its claimed 59 slots conflict with the missing C-5 row. (C0:L17-349)
- [status] (C-001/D-002) The C-0 report-status breakdown totals 105, not its stated 103; Wave 7–9 source counts also say 11+15 papers but total 19 without deduplication rules. (C0:L351-368)
- (C-001/D-003) C-0's closure map adds exact B/guide/phase ownership, including B-27→the missing C-5, B-1→C-18/C-8, and phase-4K→C-4. (C0:L370-426)
- (C-001/D-004) C-2 does not define 12 wholly new variants. It introduces ten missing enum variants while four Cosmos variants already exist; the “12 new” check mixes new and existing names. (C2:L23-31, L52-70, L200-205)
- [status] (C-001/D-005) C-2 specifies exact AgentStatusChanged fields, node/repair payloads, T_DAG=10, Cosmos event timing, ambiguity gate placement, nine named tests, and logged non-panicking emit failures. (C2:L52-148, L181-185, L262-282)
- (C-001/D-006) C-2 explicitly defers interrogation ambiguity, Cosmos scheduling/stage, anti-spiral logic/personas, several UI/command surfaces, and C-5 research events; variant definition is not behavioral implementation. (C2:L284-294)
- (C-001/D-007) C-6's missing exact values are the complete teal/text palette and --font-display; it also requires a five-item decision record and named SPEC-DELTA icons. (C6:L65-83, L99-105, L130-137)
- (C-001/D-008) C-6's seven-file 11px task is checked by a broader zero-hit dashboard-src gate, an unresolved scope mismatch the summary does not preserve. (C6:L85-92, L184-186, L220)
- (C-001/D-009) C-6 adds exactly-once component tests, aria behavior, plain-Enter preservation, no-new-dependency policy, and user escalation if JetBrains Mono is absent. (C6:L107-120, L242-267)
- (C-001/D-010) C-18's exact footprint is ten added test files, four modified Rust test hosts, and a decision doc—not only “at least ten files.” It describes about 20 distinct cases. (C18:L38-130, L228-244)
- (C-001/D-011) C-18's “D1 through D5” label is wrong; the exact contracts are D1/D2/D3/D6/D11. (C18:L89-100)
- (C-001/D-012) C-18 codifies reset non-emission, no-network symbol dispatch, exact 85% handoff, five-superstep channel semantics, byte-identical Mermaid/SSB/MessagePack behavior, and future-gated live-sidecar paths. (C18:L38-130)
- (C-001/D-013) “Zero production changes” means zero production logic: the plan still appends tests inside ssb.rs, bsp/engine.rs, and session_dag.rs and writes outside crates. (C18:L1-4, L228-244)
- (C-001/D-014) C-22 adds four StubResult fields despite repeatedly calling them three; each has serde(default), and stdout hashing is blake3 while stderr is truncated to 256 characters. (C22:L93-100)
- [failure] (C-001/D-015) C-22's resource-query failure is non-fatal, GPU UUID parse failure yields None, child.kill is the sole kill path, and event payloads are specified for accepted alternatives/escalation. (C22:L76-120)
- (C-001/D-016) C-22's six “all non-zero” metrics conflict with conditional zero-valued skip/escalation rates; real measurement must not be falsified to satisfy a textual gate. (C22:L145-153, L189-197)
- (C-001/D-017) C-22 has conditional C-19/C-22 test ownership, mistaken C-21/C-23 file descriptions, a commands.rs/lib.rs Tauri path disagreement, and a multi-GPU UUID association gap. (C22:L8-12, L130-161)
- (C-001/D-018) C-30 gives all surgical/cache struct fields, exact project signal weights, rejection fallback, answer confidence mapping, tool directions/edges/schema, and 17—not merely “at least 12”—contract tests. (C30:L62-183)
- (C-001/D-019) C-30's six VRAM labels are explicit; only the mechanical gate is ≥5. The summary's count ambiguity is a requirements-strength mismatch, not an unknown label set. (C30:L151-156)
- (C-001/D-020) C-30 leaves LRU recency and cache invalidation undefined and explicitly chooses std HashMap in conflict with C-15's FxHashMap ban. (C30:L114-134; C0:L78-81)
- (C-001/D-021) C-30's interactive rejection proceeds without an index via symbol_impact fallback; Simple can cover up to three files while its step cap is one, an unresolved affected-file coverage tension. (C30:L79-84, L102-110)
- (C-001/D-022) C-41 injects top three candidates, has explicit default/fell-through events, per-skill metadata, semver breaking-change fallback, manual debug override, and an error row on empty registry. (C41:L63-128)
- (C-001/D-023) C-41's cache-hit rule refers to an otherwise nonexistent cache design, and deterministic lookup lacks threshold/weight/tie defaults and version compatibility mechanics. (C41:L68-100, L124-128)
- (C-001/D-024) C-51's per-task scorecard has seven exact fields; hidden tests run through six named toolchains, and the no-GPU refusal has a defined dry-run exception using five mock tasks. (C51:L49-82)
- (C-001/D-025) C-51 explicitly refuses to publish any benchmark number; C-55 owns model binding, full execution, and publication. (C51:L120-149)
- (C-001/D-026) C-56 acceptance does not require meeting all three published figures. Underperformance is valid closure only if per-repo evidence and an explicit improvement-target list are published honestly. (C56:L20-30, L91-124)
- (C-001/D-027) C-56's 83% is reported against a 92% explorer; “head-to-head” is actually one ELAI run cross-tabulated against published baselines, with efficiency denominators left unstated. (C56:L14-18, L52-62)
- (C-001/D-028) C-56 fixes Sonnet 4.6 as judge, requires fixed-seed determinism and per-repo visibility, separately scores hub/caller queries, and leaves N queries per repository open. (C56:L49-97)
- [gate] (C-001/D-029) C-58's RRK gate is reject when p99>100ms, not strictly “p99 below 100ms”; adoption additionally requires ≥5pp quality improvement and remains opt-in/default-unchanged. (C58:L77-88, L116-120)
- (C-001/D-030) C-58 names all eight/five roles, solve-rate+cost outputs, back-to-back fairness rule, conditional C-45 role consolidation, two scorecards, and archive-as-not-worth-it fallback. (C58:L21-35, L61-121)
- (C-001/D-031) C-58 contains two scheduling conflicts omitted by the summary: “parallel” versus sequential AgentForge execution, and parallel-safe C-56 versus consumption of its query corpus. (C58:L23-31, L65-67, L82-121)
- (C-001/D-032) None of C-56/C-58 defines repeated-run variance, confidence intervals, model budget controls, or executable metric-validating tests; file/event existence is not benchmark reproducibility. (C56:L99-124; C58:L123-156)
- (C-001/C58-008) The actual runner command executes suites sequentially/back-to-back on identical hardware and task corpus, not interleaved, although context line 30 says AgentForge runs “in parallel.” (C58:L23-31, L65-67, L120-121)
- (C-001/C58-019) C-56 is declared parallel-safe but its query corpus is also a consumed quality dataset; “e.g.” permits another held-out set, yet no fallback corpus is named. (C58:L8-10, L82-84, L108-111)
- (C-001/C58-020) Fairness controls beyond task corpus/hardware are absent: model/backend, budgets, seeds, task ordering, warm-up, top-K, typical volume, repetitions, confidence intervals, and scorecard schemas are not fixed. (C58:L21-46, L65-96)
- (C-001/A-001) C-0 claims 59 slots/files but omits C-5 from Sessions; do not invent its plan from closure-map shorthand. (C0:L17-349)
- [status] (C-001/A-002) C-0's report-status counts total 105 rather than 103, and paper-source subtotals do not reconcile to 19. (C0:L351-368)
- (C-001/A-004) C-2's “12 new variants” is false under its own context: ten missing variants plus existing Cosmos names are checked. (C2:L23-31, L200-205)
- (C-001/A-006) C-2 defines anti-spiral events only; C-10 owns detection/personas/watchdog and C-16 owns Cosmos scheduling/stage. (C2:L284-294)
- (C-001/A-007) C-6 edits seven named 11px files but globally gates zero 11px; resolve unrelated occurrences before treating failure as C-6 scope. (C6:L85-92, L184-186)
- (C-001/A-008) C-6 cannot add an IPC, dependency, font/CDN, or silently remove icons; missing font requires user escalation. (C6:L242-267)
- (C-001/A-009) C-18's new-files-only wording conflicts with four modified source/test hosts and an out-of-tree decision doc; preserve the zero-production-logic rule. (C18:L1-4, L228-244)
- (C-001/A-010) C-18's timestamp-based file count and inferred test-case aggregate are not exact provenance checks. (C18:L172-176, L208-244)
- [gate] (C-001/A-011) C-22's metric gate can demand fabricated non-zero rates when no skip/escalation occurs; the measured value should remain authoritative. (C22:L145-153, L189-197)
- (C-001/A-013) C-22 lacks multi-GPU UUID mapping and exact behavior for a constructor signature not matching new(); both require code-level resolution. (C22:L76-80, L122-128)
- (C-001/A-014) C-30's std HashMap violates C-0/C-15's production FxHashMap policy. (C30:L114-124; C0:L78-81)
- (C-001/A-015) C-30 does not define LRU metadata, cache concurrency/persistence, stale-project invalidation, error-miss events, T4 answer mapping, or default/maximum depths. (C30:L114-149)
- (C-001/A-016) C-30 says one step per affected file but caps Simple at one while Simple permits three files; a single plan-level RegressionGate conflicts with “per step” injection; TestBaseline/allow_new_failures lack lifecycle; and task_runner versus research_tree ownership of the plan event differs between Tasks and Files. (C30:L62-100, L307-317)
- (C-001/A-019) C-56's “A/B” comparator, efficiency denominators, query N, judge repeatability, and “31 languages” wording are not resolved; benchmark claims must stay scoped to produced evidence. (C56:L1-18, L49-67, L91-97)
- (C-001/A-020) C-58 must use the later explicit sequential/back-to-back rule despite earlier “in parallel” wording; C-56 data and C-45 attribution are conditional dependencies despite parallel/recommended labels. (C58:L8-35, L65-121)
- (C-001/A-021) C-58 adopts RRK only at latency≤100ms and quality delta≥5pp, behind feature/config gates; otherwise it archives the result and never flips the default automatically. (C58:L73-88, L116-120)
- (C-001/A-022) C-56/C-58 omit benchmark resource budgets, retries, task-order controls, repeated runs, confidence intervals, and machine validation of score values. (C56:L99-124; C58:L123-156)
- [status] (C-001/C58-001) C-58 combines two RunPod evaluations: ELAI eight roles versus AgentForge five roles on SWE-bench Lite, and current ≤200M cross-encoder versus RRK 8B listwise retrieval. C-45 is recommended; C-54..C-57 are called parallel-safe. (C58:L1-10)
- [status] (C-001/A-003) Every detailed source in this chunk is a plan/specification. C-0 marks its corresponding session PLANNED; prerequisites described as SHIPPED are dependencies, not status for the session itself. (C0:L10-15, L19-335)
- (C-001/Z-001) All ten manifest paths were read from line 1 through EOF after exact SHA-256, line, and byte verification: 10 files / 2,554 logical lines / 147,739 bytes.
- (C-001/Z-002) Every source section class was checked: objective/status/prerequisites, context and source grounding, tasks/types/commands/UI, WIRING_CONTRACT, flags/defaults/config, lifecycle/fallback/security/privacy/compliance, deterministic checks, acceptance/tests, files/ownership, next/deferred work, and index closure/provenance maps. No explicit privacy/retention/compliance regime appears beyond license and sandbox/isolation requirements, so none was invented.
- (C-001/Z-003) Identifiers, payload fields, event/action names, state transitions, paths, thresholds, TTLs, flags, commands, tool schemas, benchmark metrics, test names/counts, and negative/non-goal rules are retained at their first owning finding; later sections cross-reference instead of silently multiplying them.
- (C-001/Z-004) Duplicate audit preserves repeated index-versus-plan claims as [Δ=], turns material drift into [Δ~], and labels contradictions explicitly. No planned item was upgraded to shipped, and no paper/competitor claim was independently endorsed.
- (C-001/Z-005) Scope audit: the only non-manifest discovery used was 03-C-SUMMARY_INDEX.discovery.md for orientation/deltas and an existing deep-discovery file for output-shape orientation. No unassigned original C-plan source was opened.
- (SUMMARY/S-E0828) Cites arXiv:2604.13120 for AgentForge’s reported 40.0% SWE-bench Lite result. [L763-L764]
- (SUMMARY/S-E0829) Cites arXiv:2604.26483 for an RRK 8B reranker reported as three to 18 times faster than 0.6–4B rerankers. [L764]
- (SUMMARY/S-E0830) Recommends C-45 but does not require it. [L765]
- (SUMMARY/S-E0831) Requires RunPod. [L765]
- (SUMMARY/S-E0832) Is parallel-safe with C-54 through C-57. [L766]
- (SUMMARY/S-E0833) Vendors AgentForge at a pinned SHA and includes its LICENSE. [L767]
- (SUMMARY/S-E0834) Adds AgentForgeAdapter. [L767]
- (SUMMARY/S-E0835) Adds an rrk_reranker feature and 8B model loader. [L767]
- (SUMMARY/S-E0836) Adds a latency benchmark with a p99 below 100 ms gate. [L767]
- (SUMMARY/S-E0837) Adds a quality benchmark against ELAI’s at-most-200M cross-encoder. [L767]
- (SUMMARY/S-E0838) Adds head_to_head_run_started, head_to_head_arm_completed, reranker_eval_run, and reranker_eval_decision. [L767]
- (SUMMARY/S-E0839) Acceptance produces a side-by-side scorecard against the 40.0% baseline. [L768]
- [gate/config] (SUMMARY/S-E0840) RRK adoption is configuration-gated and leaves the default unchanged until the decision favors it. [L768]

### C-59: Capstone master report from all scorecards

**Status**: planned (RunPod-dependent)
**Anchors**: ledgers C-003, SUMMARY | sample anchors: L32-L33; L1-L7; L6-L7, L14-L17; L19-L23, L49-L59; L25-L28; L29-L31, L71-L74 | 34 deep atoms, 14 summary atoms | IDs: C-003/E-C59-006, C-003/DELTA+-C59-1, C-003/E-C59-001, C-003/E-C59-002, C-003/E-C59-003, …
**Composes with**: C-1, C-2, C-3, C-6, C-14, C-18, C-20, C-22, C-29, C-30, C-41, C-45, C-48, C-51, C-54, C-55, C-56, C-57, C-58. IX: C-003/I-X-03 → C-3, C-14, C-20, C-29, C-45; SUMMARY/S-E0843 → C-54, C-55, C-56, C-57, C-58; C-001/X-012 → C-1, C-2, C-6, C-18, C-22, C-30, C-41, C-51, C-56, C-58; C-001/X-010 → C-1, C-56, C-58; C-003/I-X-04 → C-3, C-45, C-48

- Superseded: (SUMMARY/SUP-S-59SLOTS) NEWER-PLAN PRECEDENCE: later coverage section (59 slots) supersedes header 35-session legend. Retained: SUMMARY/S-E0003 — The later coverage section supersedes the old count with 59 session slots: 52 CPU/cloud sessions (C-1 through C-33 and C-35 through C-53), six RunPod-dependent sessions (C-54 through C-59), and one externally blocked session (C-34). [L578-L587]
- (C-003/E-C59-006) Cost-per-task analysis reports ELAI dollars/task across all benchmarks/configs; use a plot only if dashboard support exists, otherwise a table.
- [config] (C-003/DELTA+-C59-1) Adds the exact four configurations, per-cell triple, eight scorecard glob families, complete limitation taxonomy, loss→one-target rule, TTS positive/archive behavior, CLI, and verification-report annotations.
- [gate] (C-003/E-C59-001) Aggregate C-54 SWE-bench, C-55 Aider, C-56 Codebase-Memory, C-57's three evaluations, and C-58's two evaluations into one public-facing master report.
- (C-003/E-C59-002) If RunPod time permits, pilot Test-Time Speculation (arXiv:2605.09329), whose cited prior result is up to 72%/41% average acceptance-length improvement.
- (C-003/E-C59-003) Consume 7+ scattered scorecards/results; exact glob families include one C-54, one C-55, one C-56, three C-57, and two C-58 paths.
- [config] (C-003/E-C59-004) Cross-config matrix rows are benchmarks; columns are local 7B, dual-local 7B, hybrid Sonnet+Composer2+local, and hybrid Sonnet-only; each cell is `(solve-rate,cost,wall-clock-p99)`.
- (C-003/E-C59-005) Cross-competitor table compares best ELAI against Devin, OpenHands, Aider, Claude Code, GPT-5.4, Composer 2, AgentForge, and Codebase-Memory, each pinned by version/SHA, date, and citation URL.
- (C-003/E-C59-007) Mandatory gap narrative covers losses, wins, and ambiguous/config-dependent comparisons rather than only favorable results.
- (C-003/E-C59-008) Rank concrete improvement targets by leverage, derived from each losing benchmark's delta, and feed the next planning cycle.
- [config] (C-003/E-C59-009) TTS reruns config A local 7B on a 50-task SWE-bench Verified subset, compares wall time with C-54 A, documents a positive result, and archives a non-positive result.
- (C-003/E-C59-010) Canonical output is `.elai_cc/docs/benchmark_master_report.md`; a talk slide deck is only a planned separate follow-up.
- [lifecycle] (C-003/E-C59-011) `MasterReportAggregator` validates all scorecard schemas and regenerates Markdown solely from machine scorecards plus baseline table.
- (C-003/E-C59-012) `.elai_cc/benchmarks/external_baselines.toml` stores published numbers/version/date/URL from the prior oracle plus fresh verification at render time.
- (C-003/E-C59-013) Exclude any external number lacking version and date; report build fails if honesty is absent/empty; improvement list must be non-empty whenever ELAI loses.
- [config] (C-003/E-C59-014) Limitations must name invalid config comparisons, differing benchmark methodologies, one-shot versus multi-shot mismatch, and cloud sampling stochasticity.
- [gate] (C-003/E-C59-015) Add `master_report_aggregated`, `tts_pilot_run`, and `tts_pilot_decision`.
- (C-003/E-C59-016) Fixture aggregator must produce a deterministic matrix from N scorecards; same inputs must regenerate byte/logically identical output idempotently.
- (C-003/E-C59-017) Validate required baseline fields; ensure loss derivation yields at least one target; make honesty a build gate.
- (C-003/E-C59-018) Reach from `elai benchmark master-report` and optional `--tts-pilot`.
- (C-003/E-C59-019) Report must contain all five sections, all three events must emit, TDD/idempotency must pass, and optional TTS appears only if run.
- (C-003/E-C59-020) “Public-ready” means linkable from README/blog/paper; this task creates the artifact but does not itself publish externally.
- (C-003/E-C59-021) Each C-54..C-58 verification report receives a closure annotation pointing to the master.
- [lifecycle] (C-003/E-C59-022) After closure, the master becomes source of truth for external competitiveness claims; reruns regenerate from scorecards+baseline TOML, and targets seed C-60+.
- (C-003/DELTA!-C59-2) The source names eight competitor families; this confirms the summary's correction to the earlier “seven anchors” narrative.
- (C-003/DELTA+-C59-3) “Deterministic regeneration” is conditional on identical pinned inputs and remains in tension with render-time fresh fetch; input schema and normalization/failure policies are not recovered by the original.
- (C-003/C-C59-01) The report must be deterministic/idempotent for the same inputs, while baseline inputs are fresh-fetched at render time and may change; no snapshot/update boundary reconciles live fetch with stable inputs (L71-L74, L104-L110).
- (C-003/A-C59-01) Scorecard schema/version, discovery order/deduplication, incomplete/corrupt/mixed-run handling, required benchmark set, and whether glob results are sorted are unspecified (L49-L69).
- (C-003/A-C59-02) Units/currency/date normalization, cost allocation, percentile aggregation, “best config” tie-breaks, missing-cell representation, and cross-benchmark comparability are absent (L25-L39).
- (C-003/A-C59-03) Machine generation forbids handwritten narrative drift, but algorithms/templates for honesty prose and one concrete next-step per loss are not specified (L68-L80, L130-L136).
- [failure] (C-003/A-C59-04) External baseline refresh failure/retry/cache behavior and version-vs-SHA requirements are not defined; context says “version SHA” while the table task says version (L29-L31, L71-L74).
- (C-003/A-C59-05) TTS budget threshold, 50-task subset pin, speculative implementation/config, statistical decision threshold, archive location, and negative-run event semantics are unspecified (L45-L47, L94-L102).
- (C-003/A-C59-06) Event payload schemas, emission ordering/idempotency, and whether skipped TTS emits `tts_pilot_decision` are unstated (L101-L102).
- (SUMMARY/S-E0841) Is the capstone aggregator for C-54 through C-58 and external baselines from the 2026-05-17 oracle pass. [L770-L771]
- (SUMMARY/S-E0842) Optionally includes a TTS pilot based on arXiv:2605.09329 and its reported 41% average acceptance-length improvement. [L771]
- (SUMMARY/S-E0843) Requires C-54, C-55, C-56, C-57, and C-58 to be shipped with completed runs. [L772]
- (SUMMARY/S-E0844) Is not parallel-safe because it consumes all benchmark outputs. [L773]
- (SUMMARY/S-E0845) Adds MasterReportAggregator. [L774]
- (SUMMARY/S-E0846) Adds external_baselines.toml. [L774]
- (SUMMARY/S-E0847) Pins Devin, OpenHands, Aider, Claude Code, GPT-5.4, Composer 2, AgentForge, and Codebase-Memory by version, date, and URL. [L774]
- (SUMMARY/S-E0849) Regenerates benchmark_master_report.md from machine-readable inputs. [L774]
- [config] (SUMMARY/S-E0850) The report contains a cross-configuration matrix, cross-competitor table, cost-per-task analysis, mandatory honesty section, and improvement-target list. [L774]
- (SUMMARY/S-E0851) The report may include an optional TTS pilot section. [L774]
- [gate] (SUMMARY/S-E0852) Adds master_report_aggregated, tts_pilot_run, and tts_pilot_decision. [L774]
- (SUMMARY/S-E0853) Acceptance requires deterministic report compilation and presence of honesty and improvement-target sections. [L775]
- (SUMMARY/S-E0854) The intended claim boundary is the defensible form “ELAI is competitive on X measurement at Y cost.” [L775]
- (SUMMARY/S-A0848) This is eight named baseline families despite the earlier narrative claiming seven competitor anchors. [L589, L774]

### User-facing surfaces (C)

Every window, panel, drawer, command, CLI verb, flag, status display, or notification named in SURFACE-tagged atoms. One line each.

- [C-0] (C-001/C0-004) C-6 is a prerequisite-free dashboard-only PLANNED token/font/sidebar pass: align background, teal, and text tokens, fix fonts, shrink Sidebar 64→48px, and verify computed CSS after a cloud task. (C0:L26-29)
- [C-0] (C-001/C0-007) C-3 is PLANNED after C-1 and adds get_query_trace_recent, get_pagerank_top, get_freshness_state, get_canary_drift, get_csq_log, get_feasibility_log, and get_agent_grc_config; KnowledgeDrawer and ReasoningDrawer must show non-empty live data. (C0:L43-46)
- [C-0] (C-001/C0-008) C-4 is PLANNED after C-1 and wires live agents, cloud spend, phase/baby-step/pipeline/progress/recent-action, runtime-list, and set_role_runtime commands; MonitorWindow must show at least one non-IDLE agent and non-zero spend. (C0:L48-51)
- [C-0] (C-001/C0-009) C-7 is PLANNED after C-6 and C-2: sweep seven Svelte files from 11px to 10px, add Stop/Ctrl+Enter/Ctrl+Shift+A/R, responsive breakpoints, ExploreWindow, recovery history, and per-hunk review controls; its smoke requires live streaming controls and BubbleUpStrip events. (C0:L53…
- [C-0] (C-001/C0-010) C-8 is PLANNED after C-1/C-2 and activates AmbiguityScorer, CosmosWorker::tick, TOOL_INDEX_ARTIFACTS, a real CrossEncoderReranker SsbNotifier, ONNX dot product in AragOrchestrator, CosmosStage, and drawer unwired flags; query-stage latencies must be non-zero and Cosmos status …
- [C-0] (C-001/C0-015) C-10 is PLANNED after C-9/C-2: implement StagnationPattern/Detection, ExecutionHistory, ThinkingPersona/PersonaStrategy, GoalDriftCanary, GenerationWatchdog, AdaptiveFutilityTracker, five persona files, and a StagnationSection; three identical errors must trigger stagnation an…
- [C-0] (C-001/C0-017) C-16 is PLANNED after C-2/C-8: place CosmosStage between ArtifactStage and ARagStage, schedule hourly CosmosWorker ticks, add trigger_cosmos_refresh, replace tracing with SSB rows, and add five standalone tests; a real OpenAPI research run must show refresh-started and Fresh. …
- [C-0] (C-001/C0-021) C-20 is PLANNED after C-9/C-13/C-17: replace placeholder metrics, wire ELAI_DEBUG_BATCH_DELAY_SECS, add elai measure, and require non-zero phase_4G runtime categories after a Sonnet 4.6 measurement run. (C0:L115-118)
- [C-0] (C-001/C0-023) C-22 is PLANNED after C-19/C-9: rename SystemProbeReport, add GPU UUID and ResourceDelta, production-wire DefaultFeasibilityProber, add adversarial coverage and FeasibilityPanel, reconcile event names, and populate phase_4F metrics. (C0:L125-128)
- [C-0] (C-001/C0-024) C-23 is PLANNED after C-3/C-4: add /session merge|close, mount BranchTree with live fork/merge/close, repair /continue handoff loading and parent ordering, and prove a context handoff can be continued. (C0:L130-135)
- [C-0] (C-001/C0-029) C-28 is PLANNED after C-27: integrate cargo-mutants and mutmut, MutationResult, MUTATION_SURVIVED, EquivalentMutantCache, an AUDITING→MEMORIZING mutation gate, and a MonitorWindow score gauge. (C0:L157-162)
- [C-0] (C-001/C0-033) C-32 is PLANNED after C-31/C-26: skeleton-first BLANK_N generation, one in-memory write, MiniCritic, AcceptAndComplete at ≥0.5, structural retries DECOMPOSE→TEMPLATE→FEW_SHOT→BEST_OF_N, deterministic bridge files, and a completeness gauge; its smoke targets ≥0.9 completeness. …
- [C-0] (C-001/C0-039) C-39 is PLANNED after C-35: extend the CLI/tool surface through MCP with an OpenHands-parity CI job and automatic co-process launch; --via-mcp parity must pass. (C0:L216-219)
- [C-0] (C-001/C0-045) C-45 is PLANNED, but its per-role CANTANTE pilot is deferred to a RunPod window; it measures contribution for C-41 routing and role consolidation and must produce confidence intervals. (C0:L246-251)
- [C-1] (C-002/C1-E18) Preserve `tracing::debug` and add a `QueryTrace` SSB write after it; `QueryPipeline` gains optional notifier state threaded from the live CLI construction path. Its contract expects exactly one trace per minimal query, including nonnegative result count and latency. [C-1:L103-…
- [C-1] (C-002/C1-E19) Replace the production `NoopSsbNotifier` default in `CrossEncoderReranker::new` with the real live-CLI SSB handle; test-only/no-comment occurrences are the only tolerated remnants. [C-1:L109-L110; C-1:L150-L152; C-1:L197-L199]
- [C-1] (C-002/C1-E20) `get_grounding_sweep` must read real `GroundingSweepResult` rows through `SsbViewer`, remove hardcoded zero output, and set `unwired.groundingSweep=false`. [C-1:L112-L114; C-1:L201-L204]
- [C-1] (C-002/C1-E21) `get_query_trace_recent` must return real `query_trace` rows rather than `json!([])` and set `unwired.recentTraces=false`. [C-1:L116-L117; C-1:L193-L195]
- [C-1] (C-002/C1-E22) `get_csq_pending` must return `csq_evaluated` rows filtered to `blocked=true` and set `unwired.csqPending=false`. [C-1:L119-L120]
- [C-1] (C-002/C1-E27) UI acceptance requires a non-`task_runner` agent in ReasoningDrawer, at least one recent trace in KnowledgeDrawer, and all three relevant `unwired` flags false. [C-1:L223-L227]
- [C-1] (SUMMARY/S-E0030) Wires get_grounding_sweep, get_query_trace_recent, and get_csq_pending Tauri commands to live data. [L35]
- [C-2] (C-001/C2-001) Objective: close five producer gaps—tree navigator, Cosmos worker, ambiguity scorer, autonomous stagnation detector, and agent lifecycle—so BubbleUpStrip and OrchestrationDrawer display live cloud events. C-1 is required; C-3/C-4/C-9 are parallel-safe; estimate 6–8h. (C2:L1-10)
- [C-2] (C-001/C2-002) B-7 lacks PhaseNodeReady, StepNodeReady, BabyStepReady, and DagRepairEscalated definitions/emits, while B-13's BubbleUpStrip already queries bubble_up_typed but documents an empty-until-wired state. (C2:L12-18)
- [C-2] (C-001/C2-003) B-6 lacks AgentStatusChanged and get_agents returns eight hardcoded IDLE roles. (C2:L20-21)
- [C-2] (C-001/C2-004) B-38 already defines AmbiguityScored, AmbiguityFloorFailed, and AmbiguityStreakAdvanced, but task_runner does not invoke the scorer, get_ambiguity_state returns mock:true, and the drawer has no ambiguity section. (C2:L23-25)
- [C-2] (C-001/C2-016) get_agents must replace the hardcoded eight-role IDLE list with recent_agent_status_changes so live state reaches the drawer; get_ambiguity_state must replace mock:true with the three ambiguity row types. (C2:L116-126)
- [C-2] (C-001/C2-017) A collapsed AmbiguitySection calls get_ambiguity_state and renders overall_score, per-dimension bars, floor-failure flags, and streak progress using GatesPanel's layout. (C2:L128-133)
- [C-2] (C-001/C2-021) Reachability is cloud /task, BubbleUpStrip queries, live OrchestrationDrawer agent/ambiguity sections, and CosmosDrawer refresh; consumed anchors include C-1's variant helper, scorer/evidence fields, existing repair/bubble emit patterns, and threaded ssb_tx. (C2:L150-180)
- [C-2] (C-001/C2-023) The three-function Rust CLI trigger expects ≥1 phase row, ≥3 step rows, a non-null ambiguity score, optional Cosmos refresh if its tick falls in-window, and lifecycle per spawned role; dashboard evidence is a real bubble, non-IDLE role, and numeric ambiguity. (C2:L187-197)
- [C-2] (C-001/C2-029) Persisted producer rows plus latest-state reduction create two semantics on one log: history for Bubble/Cosmos/ambiguity evidence and materialized latest lifecycle for get_agents. This composition follows explicit endpoints; retention and concurrent-write ordering are not spec…
- [C-2] (SUMMARY/S-E0044) Replaces the hardcoded-IDLE get_agents result with live agent data. [L46]
- [C-2] (SUMMARY/S-E0045) Replaces the mock get_ambiguity_state result with live ambiguity data. [L46]
- [C-2] (SUMMARY/S-E0048) Cloud smoke task: build a three-function Rust add/subtract/multiply CLI; acceptance requires at least five distinct new action types with non-null values. [L48]
- [C-3] (C-003/C-C3-01) The source says seven viewer methods are stubbed but enumerates only five (`pagerank_top`, `query_tier_stats`, `freshness_state`, `research_budget_snapshot`, `source_tier_distribution`). `get_query_trace_recent` is separately a command stub, not a sixth/seventh viewer method (…
- [C-3] (C-003/C-C3-02) The objective says six wholly missing commands, while the implementation/registration set contains eight commands. Six are explicitly called “missing entirely”; `get_pagerank_top` and `get_freshness_state` are also added but omitted from that six-command inventory (L2-L4, L26-…
- [C-3] (C-003/DELTA!-C3-1) The summary’s “seven stubs, six visibly named” is not the original’s structure: the original names five viewer stubs and separately one Tauri command stub; the seventh remains unidentified.
- [C-3] (C-003/E-C3-005) Replace `get_query_trace_recent`'s unconditional `json!([])` with a viewer call and default to an empty array on missing viewer/read failure; set `unwired.recentTraces=false`.
- [C-3] (C-003/E-C3-007) `get_pagerank_top(n: u32)` defaults `n` to 20, calls the viewer, and is registered in `lib.rs`.
- [C-3] (C-003/E-C3-010) Add and register `get_freshness_state`, delegating to `SsbViewer::freshness_state()`.
- [C-3] (C-003/E-C3-011) Add/register `get_canary_drift`; read the newest `action_type='canary_drift'` row and return `{delta_ms,baseline_ms,current_ms,drift_pct,ts}`, or `{known:false}` if absent.
- [C-3] (C-003/E-C3-012) Add/register `get_csq_log`; return the latest 50 `csq_evaluated` rows as `{csq_id,conclusion,guard_present,ts_ms}`, with `[]` as the no-row result.
- [C-3] (C-003/E-C3-013) Add/register `get_feasibility_log`, distinct from `get_feasibility_report`; return the latest 20 `feasibility_check` rows as `{task_id,verdict,score,ts_ms}`, with `[]` when absent.
- [C-3] (C-003/E-C3-014) Add/register `get_agent_grc_config`; read the `AgentGrcConfig` table through SSB DB or config store and return `{role,max_calls,max_chars,topic_prefix}` rows; use compiled defaults if no runtime override exists.
- [C-3] (C-003/E-C3-015) Add/register `get_research_budget`, wrapping the existing viewer method; keep `known:false` until a `research_budget_updated` row exists, then update the viewer to consume it.
- [C-3] (C-003/E-C3-016) Add/register `get_source_tier_distribution` by wrapping the existing viewer method.
- [C-3] (C-003/E-C3-017) The live consumer path is `KnowledgeDrawer.svelte` and `ReasoningDrawer.svelte` polling through `drawers.svelte.ts`.
- [C-3] (C-003/E-C3-018) After C-3, `pagerank_top` may not remain an unconditional empty-vector stub; after C-1, `get_query_trace_recent` may not remain an unconditional empty array.
- [C-3] (C-003/E-C3-038) `ts-rs` generation and `MonitorWindow` feature-gating remain B-3 work; `CsqBlockCard` remains B-5 work.
- [C-3] (C-003/E-C3-039) Real `get_grounding_sweep` waits for B-8, and `ELAI_DASHBOARD_OFFLINE` gating remains B-36 work.
- [C-3] (SUMMARY/S-E0052) Adds eight Tauri commands: get_pagerank_top, get_freshness_state, get_canary_drift, get_csq_log, get_feasibility_log, get_agent_grc_config, get_research_budget, and get_source_tier_distribution. [L55]
- [C-3] (SUMMARY/S-E0053) Wires get_query_trace_recent to real SSB rows. [L56]
- [C-4] (C-002/C4-E03) Recorded gaps include an eight-role always-IDLE `get_agents`, missing V2 phase/baby-step commands, an empty executable pipeline, absent research history, four implemented-but-unregistered viewer methods, no project list/create service, three frontend runtime stubs despite real…
- [C-4] (C-002/C4-E05) `get_agents` reads the latest 200 lifecycle rows, groups by role, takes maximum sequence per role, maps state and payload PID/heartbeat, and falls back to IDLE only for roles with no SSB row before any task. [C-4:L81-L87]
- [C-4] (C-002/C4-E06) Add and register `get_phase_detail(phase_id:String)`, forwarding to the real viewer; unknown IDs return structured `PhaseNotFound`, not panic. [C-4:L89-L92; C-4:L247-L248]
- [C-4] (C-002/C4-E07) Add/register `get_baby_step_detail(baby_step_id:String)` and replace unconditional NotFound by querying `baby_step_ready` or `concept_decomposition_node` by ID. If C-2 is absent, return `BabyStepNotPersisted`, not panic or stub success. [C-4:L94-L101; C-4:L249-L250]
- [C-4] (C-002/C4-E08) Replace `get_executable_pipeline(session_id:String)` empty JSON with ordered phase → step → baby-step leaves reshaped from `decomposition_tree`/`concept_decomposition_node`; no rows produce an empty vector, not panic. [C-4:L103-L108; C-4:L251-L252]
- [C-4] (C-002/C4-E09) Add/register `get_research_node_history(session_id:String,node_id:String)`, filter `research_node_stage`, deserialize `ResearchNodeHistoryEntry` matching the TS shape, and return an empty vector when B-7's emitter is absent. [C-4:L110-L115; C-4:L253-L254]
- [C-4] (C-002/C4-E10) Expose/register thin wrappers for `get_phase_progress`, `get_recent_actions`, `get_file_changes`, and `get_action_detail`; export the latter from `stub.ts`. [C-4:L117-L122]
- [C-4] (C-002/C4-E13) Replace frontend implementations of `list_runtimes`, `test_runtime`, and `set_role_runtime` with `invoke(...)` calls using `{backend}` / `{role,backend}` payloads; mount `RuntimePanel.svelte` in Settings if absent. [C-4:L132-L138]
- [C-4] (C-002/C4-E14) Add `SsbViewer::cloud_spend_summary()`: latest `cloud_spend.cost_usd` as task spend, today's summed cost as day spend, today's cache-creation/read token totals, and redaction count from `RedactionStat` when defined or zero otherwise. `get_cloud_spend` must use AppState/viewer;…
- [C-4] (C-002/C4-E15) The exact ten newly exposed Tauri commands are `get_phase_detail`, `get_baby_step_detail`, `get_executable_pipeline`, `get_research_node_history`, `get_phase_progress`, `get_recent_actions`, `get_file_changes`, `get_action_detail`, `list_projects`, and `create_project`; all mu…
- [C-4] (C-002/C4-E16) `get_agents` cannot return a hardcoded literal; `get_cloud_spend` cannot return hardcoded zero fields; `get_executable_pipeline` cannot return `Ok(json!([]))`; and spend aggregation must query SSB rather than `Default::default()`. [C-4:L186-L190]
- [C-4] (C-002/C4-E19) Acceptance additionally requires known phase/baby-step data after producers exist, project directory/index persistence, live runtime panel data, nonzero MonitorWindow gauge, frontend-callability of all wrappers, and the five manual smoke assertions. [C-4:L241-L273]
- [C-4] (C-002/SD-C-4-05) Correction: C-4's own “Next: C-5 ReviewWindow” does not match the actual C-5 source and should not orient sequencing. [C-4:L295-L298; C-5:L1-L5]
- [C-4] (SUMMARY/S-E0058) Replaces three stubs: get_agents no longer returns hardcoded IDLE, get_cloud_spend no longer returns zeros, and get_executable_pipeline no longer returns an empty array. [L64]
- [C-4] (SUMMARY/S-E0059) Adds ten Tauri commands: get_phase_detail, get_baby_step_detail, get_executable_pipeline, get_research_node_history, get_phase_progress, get_recent_actions, get_file_changes, get_action_detail, list_projects, and create_project. [L65]
- [C-4] (SUMMARY/S-E0060) Replaces RuntimePanel stub calls with real invocations for list_runtimes, test_runtime, and set_role_runtime. [L66]
- [C-4] (SUMMARY/S-E0061) Cloud smoke task: implement a four-function todo CRUD module; acceptance requires MonitorWindow to show at least one non-IDLE agent and a non-zero cloud-spend value. [L67]
- [C-5] (C-002/C5-E03) The shipped CLI selector is `--provider local|cloud|hybrid`, not `--mode`; `--mode` is a separate gate-mode axis. `resolve_backend_mode` gives the CLI override precedence over TOML `[cloud].default_mode`, and absent CLI input uses that default. [C-5:L19-L22; C-5:L110-L119]
- [C-5] (C-002/C5-E05) The shipped component path is `dashboard-v2/src/lib/screens/CloudPanel.svelte`, not `lib/components`, mounted under Settings' Cloud section. [C-5:L23-L25; C-5:L165-L188]
- [C-5] (C-002/C5-E07) `get_cloud_route_status`, `CloudRouteStatus`, and three CloudPanel banners shipped. A dedicated last-five `RedactionApplied` timeline is explicitly deferred pending `get_recent_redactions` or a spend-summary extension. [C-5:L31-L37; C-5:L176-L186]
- [C-5] (C-002/C5-E17) Cost estimation occurs before the first cloud call. The autonomous path does not block; the interactive path surfaces the estimate to CloudPanel. The shipped status says the event is the atomic `CostEstimate` variant. [C-5:L145-L148; C-5:L28-L31]
- [C-5] (C-002/C5-E20) CloudPanel spend UI shows daily spend/cap, warn-at-80 and blocked badges, and a monthly-cap row when configured. [C-5:L168-L172]
- [C-5] (C-002/C5-E21) The provider selector is a local/hybrid/cloud segmented control calling `set_execution_mode`; cloud selection is disabled when `airGapped=true`. [C-5:L173-L174]
- [C-5] (C-002/C5-E22) CloudPanel shows today's aggregate redaction count; the detailed recent-event feed remains deferred. [C-5:L175-L186]
- [C-5] (C-002/C5-E25) The primary observable sets daily cap `$0.05`, triggers at `$0.04`, expects one warning then route block at cap, and expects the gauge/mode selector to reflect live state. [C-5:L237-L242]
- [C-5] (C-002/C5-E27) Acceptance enumerates daily/monthly errors, route-block and warning evidence, provider bridge, one escalation, constants/helper, bounded cost estimate, endpoint guard, panel mounting, tests, three smoke scenarios, and clean handling rather than panic. [C-5:L286-L318]
- [C-5] (C-002/C5-E28) Production changes cover cloud router, SSB schema, task runner, CLI/config, dashboard, and contracts; however, the later Files list still contains pre-amendment names/paths and is not authoritative over the status block. [C-5:L320-L339]
- [C-5] (C-002/SD-C-5-01) Critical correction: The index says CLI `--mode cloud|hybrid|local`; the amended shipped source says `--provider`, with `--mode` reserved for gate mode. [C-5:L19-L22; C-5:L110-L119]
- [C-5] (C-002/SD-C-5-06) Correction: CloudPanel lives under `lib/screens`; its detailed last-five redaction timeline is deferred. The index's unqualified panel/redaction-feed wording overstates completion. [C-5:L23-L37; C-5:L165-L188]
- [C-5] (C-002/SD-C-5-07) Addition: Banner precedence, `get_cloud_route_status`, one-escalation halt behavior, and `airGapped` selector disabling are explicit source behaviors. [C-5:L31-L37; C-5:L120-L126; C-5:L173-L181]
- [C-5] (SUMMARY/S-E0062) Closes B-27; depends on C-1 and C-4; is parallel-safe with C-6 and C-7, while CloudPanel adopts C-6 design tokens when available. [L69-L72]
- [C-5] (SUMMARY/S-E0066) Adds CLI mode selection through --mode cloud, --mode hybrid, and --mode local. [L73]
- [C-5] (SUMMARY/S-E0072) Creates CloudPanel.svelte with a spend gauge, mode selector, and redaction-event feed. [L75]
- [C-5] (SUMMARY/S-E0073) Mounts CloudPanel in SettingsWindow. [L75]
- [C-5] (SUMMARY/S-E0077) Smoke 3 verifies CloudPanel renders. [L77]
- [C-6] (C-001/C6-001) C-6 is a one-session, backend-free dashboard pass with no prerequisites, parallel-safe with C-5/C-7; it covers tokens, fonts, 11px sites, sidebar dimensions/count, Stop, and Ctrl+Enter. (C6:L1-8)
- [C-6] (C-001/C6-002) Verified deltas are wrong background/font tokens, inconsistent teal overrides, absent/mis-valued text-ghost, seven 11px component files, 64px/seven-icon Sidebar versus 48px/five-icon spec, and missing Stop/Ctrl+Enter; the source calls these cosmetic/UX only. (C6:L10-31)
- [C-6] (C-001/C6-007) In ArtifactsPanel, IndexingProgressPanel, MediaValidationPanel, AntiStalenessDrawer, CosmosDrawer, KnowledgeDrawer, and MonitorWindow, inspect each 11px occurrence and change only font-size/line-height to the 10px accessibility floor—not unrelated padding/radius. (C6:L85-92)
- [C-6] (C-001/C6-008) Sidebar becomes 48px only after verifying icons ≤20px and horizontal padding ≤8px per side still fit. (C6:L94-97)
- [C-6] (C-001/C6-011) On a non-empty, non-streaming textarea, Ctrl+Enter calls the same submit handler; plain Enter retains newline behavior, duplicate IPC is forbidden, and the textarea gets aria-label “Press Ctrl+Enter to send.” (C6:L115-120, L154-158)
- [C-6] (C-001/C6-015) Deterministic checks inspect all token groups, body font, zero 11px hits, 48px/no-64px Sidebar, Stop/Ctrl handler presence, zero Svelte errors, and the decision document. (C6:L168-208)
- [C-6] (C-001/C6-017) Manual cloud smoke starts and stops a hello-world stream, sends a fresh prompt with Ctrl+Enter, and inspects computed bg-void plus JetBrains-first body font. (C6:L230-240)
- [C-6] (C-001/C6-018) Component tests require data-testid=stop-button conditional rendering and exactly-once Ctrl+Enter submission for non-empty input. Reuse existing Vitest tooling or create CommandWindow.test.ts; no new dependency without human confirmation. (C6:L242-254)
- [C-6] (SUMMARY/S-E0082) Changes 11px text to 10px across ArtifactsPanel, IndexingProgressPanel, MediaValidationPanel, AntiStalenessDrawer, CosmosDrawer, KnowledgeDrawer, and MonitorWindow. [L84]
- [C-6] (SUMMARY/S-E0083) Shrinks the sidebar from 64px to 48px. [L85]
- [C-6] (SUMMARY/S-E0084) Documents the sidebar icon-count delta: five in the specification versus seven currently. [L85]
- [C-6] (SUMMARY/S-E0086) Adds Ctrl+Enter in CommandWindow and connects it to the existing submit handler. [L86]
- [C-6] (SUMMARY/S-E0088) Manual smoke verifies computed CSS tokens and tests Stop and Ctrl+Enter during an active stream. [L88]
- [C-7] (C-003/A-C7-01) Restore records a title but calls `openTab` with only the type; removal of a restored entry, duplicate handling, recovery-stack persistence, and modal-close behavior are not defined (L50-L60).
- [C-7] (C-003/A-C7-03) At exactly 800px, the `max-width:799px` single-panel rule does not apply even though the visual test is described as sidebar collapse at 800px; that test exercises the 999px rule only (L86-L100).
- [C-7] (C-003/C-C7-01) C-7 says C-3 supplies `get_file_changes`, `accept_change`, `decline_change`, and `retry_audit`, but the complete C-3 source defines none of them and asserts its file scope is disjoint from C-4 (C-7 L78-L82/L127; C-3 L1-L8/L68-L141).
- [C-7] (C-003/DELTA+-C7-1) The summary omits that Explore deliberately remains backend-stubbed, the exact stack schema/cap/FIFO rule, most-recent ordering, empty/loading states, all four live invokes, and conditional `Ctrl+Shift+W` behavior.
- [C-7] (C-003/DELTA+-C7-3) Exact breakpoint inclusivity is `<=999px` and `<=799px`, not merely conceptual “below 1000/800”; the snapshot at 800px tests hamburger collapse, not the single-panel threshold.
- [C-7] (C-003/E-C7-001) Add `'explore'` to `WindowType`, `VALID_WINDOW_TYPES`, and `SIDEBAR_WINDOWS`; register `ExploreWindow.svelte`; bind `Ctrl+8` via `sidebar-8`.
- [C-7] (C-003/E-C7-002) `ExploreWindow` renders a complete Q&A UI (input, submit, scrollable responses), logs `stub.logStub('explore.query',{query})`, and appends a placeholder response; only live backend search remains intentionally stubbed and marked by `UnwiredNote`.
- [C-7] (C-003/E-C7-004) Export `closedTabs`, `showRecoveryModal`, and `openWindowRecovery()`; the function sets the modal flag.
- [C-7] (C-003/E-C7-005) `RecoveryModal.svelte` lists real closures most-recent first with type, title, relative time, and `Restore`; restore calls `workspace.openTab('primary',entry.type)`; mount beside `CommandPalette` in `AppShell`.
- [C-7] (C-003/E-C7-006) Keep shortcut id/combo/description for `recover-closed` but change `Ctrl+Shift+H` from command palette to `workspace.openWindowRecovery()`.
- [C-7] (C-003/E-C7-007) `Ctrl+Shift+A` and `Ctrl+Shift+R` call `reviewStore.acceptAllHunks()`/`rejectAllHunks()` only when the primary active tab is `review`; no global accept/reject.
- [C-7] (C-003/E-C7-008) Create a minimal Svelte 5 `review.svelte.ts` store exposing accept-all and reject-all methods for `ReviewWindow` subscription.
- [C-7] (C-003/E-C7-010) Replace static `FILES` with a derived `reviewStore.files`, remove `UnwiredNote`, invoke `get_file_changes({session_id})` on mount, and invoke `accept_change`, `decline_change`, or `retry_audit` from hunk/file actions.
- [C-7] (C-003/E-C7-011) An empty `get_file_changes` result renders `<Empty>` rather than fixture data; `EXECUTING` with no changes yet renders a loading spinner.
- [C-7] (C-003/E-C7-012) At `max-width:999px`, hide `nav-list`, show/collapse to a hamburger icon strip, and manage hamburger state with `Sidebar.svelte` `ResizeObserver`.
- [C-7] (C-003/E-C7-013) At `max-width:799px`, `Workspace.svelte` uses `ResizeObserver` to set `secondaryCollapsed=true`, giving the primary panel full width.
- [C-7] (C-003/E-C7-015) Snapshot (b) opens and renders `ExploreWindow` with `Ctrl+8`.
- [C-7] (C-003/E-C7-016) Snapshot (c) renders `ReviewWindow` with hunk controls visible.
- [C-7] (C-003/E-C7-017) Snapshot (d) closes a window and opens `RecoveryModal` with `Ctrl+Shift+H`.
- [C-7] (C-003/E-C7-018) Snapshot (e) verifies sidebar collapse at an 800px viewport; baselines use `page.screenshot` and later regression uses `toHaveScreenshot()`.
- [C-7] (C-003/E-C7-020) Add `Ctrl+Shift+W` only if a `shell`/`tier_0` window type exists; otherwise add only the exact deferral comment and no shortcut code.
- [C-7] (C-003/E-C7-024) Restore reopens a window through `openTab`; accept/reject/retry invoke live handlers; refresh reflects applied state.
- [C-7] (C-003/E-C7-026) Adds `review.svelte.ts`, `ExploreWindow.svelte`, `RecoveryModal.svelte`, `visual.spec.ts`, and `workflow/reasoning/C-7-decisions.md`; modifies the named API/store/window/shell/sidebar/workspace/CSS files.
- [C-7] (SUMMARY/S-E0091) Creates ExploreWindow.svelte. [L94]
- [C-7] (SUMMARY/S-E0092) Assigns Ctrl+8 to open the Explore window. [L94]
- [C-7] (SUMMARY/S-E0094) Creates RecoveryModal.svelte. [L95]
- [C-7] (SUMMARY/S-E0095) Fixes Ctrl+Shift+H so it opens recovery instead of remaining a no-op stub. [L95]
- [C-7] (SUMMARY/S-E0096) Adds Ctrl+Shift+A for review accept-all. [L96]
- [C-7] (SUMMARY/S-E0097) Adds Ctrl+Shift+R for review reject-all. [L96]
- [C-7] (SUMMARY/S-E0098) Adds per-hunk Accept, Decline, and Retry controls to ReviewWindow. [L97]
- [C-7] (SUMMARY/S-E0099) Removes the UnwiredNote banner after wiring ReviewWindow to live get_file_changes data. [L97]
- [C-7] (SUMMARY/S-E0100) Adds a responsive hamburger layout below 1000px. [L98]
- [C-7] (SUMMARY/S-E0101) Adds a single-panel layout below 800px. [L98]
- [C-7] (SUMMARY/S-E0103) Cloud smoke modifies hello.rs to add goodbye; acceptance requires visible hunk controls and working Ctrl+Shift+A accept-all. [L100]
- [C-8] (C-004/D-C8-009) `latest_ambiguity_scored` is the shipped method name, and the alleged `unwired.ambiguityState` flag is a resolved phantom. [Summary L190; C-8 L100-L116, L244-L257]
- [C-8] (C-004/F-C8-004) The preserved objective replaces live-data mocks/hardcoded producer stubs across ambiguity scoring, Cosmos scheduling/events/stage integration, artifact indexing/RPC, reranking notification, A-RAG semantic scoring, pipeline latency, degradation/search-mode state, and dashboard…
- [C-8] (C-004/F-C8-010) `get_ambiguity_state` queries the newest `ambiguity_scored` row, returns the real score structure, and omits the former `"mock"` field. [C-8 L112-L116, L330-L332, L402]
- [C-8] (C-004/F-C8-011) The Cosmos hourly scheduler is hosted in `cli/src/live_cli.rs` within `build_local_knowledge_stack`, not in the per-task `task_runner.rs`; this gives it orchestrator-process lifetime. [C-8 L118-L138, L334-L340]
- [C-8] (C-004/F-C8-012) Scheduler bootstrap occurs only when an `Arc<Ssb>` is present and the manifest-present branch succeeds; CLI cold-start paths without an SSB writer remain unaffected. [C-8 L121-L130]
- [C-8] (C-004/F-C8-022) Search-mode state crosses the CLI/dashboard process boundary through a new `SearchModeChanged` SSB row and `SearchModeChangedPayload`, replacing the infeasible in-memory `AppState Arc<RwLock<SearchMode>>` design. [C-8 L181-L189]
- [C-8] (C-004/F-C8-023) The CLI emits search mode once at startup and on every `DegradationCascade` transition. [C-8 L185-L188, L418-L420]
- [C-8] (C-004/F-C8-024) Dashboard `get_search_mode` reads `SsbViewer::latest_action_of_type("search_mode_changed")`; if no row exists, it gracefully returns lowercase `"hybrid"`, not a hardcoded uppercase `"HYBRID"` production answer. [C-8 L187-L190, L359-L363, L421-L422, L527-L529]
- [C-8] (C-004/F-C8-025) A new dashboard-side `set_search_mode` Tauri command emits a `SearchModeChanged` row, allowing reviewers to pin the diagnostic mode. [C-8 L190-L191, L527-L530]
- [C-8] (C-004/F-C8-037) The proposed `unwired.ambiguityState` flag is a phantom: it never existed. `GatesPanel.svelte` and `OrchestrationDrawer.svelte` invoke `get_ambiguity_state` directly, so there is nothing to flip. [C-8 L244-L257, L433-L436]
- [C-8] (C-004/F-C8-038) `trigger_cosmos_refresh` remains an uncalled per-task helper in `task_runner.rs`; follow-up must either wire it from a task-entry hook or document its removal. [C-8 L536-L542, L561-L565]
- [C-8] (C-004/F-C8-039) A frontend per-mode `SearchModeBadge` picker that calls `set_search_mode` remains pending, although its backend command is complete. [C-8 L566-L567]
- [C-8] (C-004/F-C8-045) Twelve deterministic checks cover source symbols/absence patterns, scheduler host, artifact hook, stub removal, notifier construction, function-scoped search-mode hardcoding, all latency sites, Cosmos stage, dashboard flag, workspace compilation, and four contract-test invocat…
- [C-8] (C-004/F-C8-049) The Cosmos contract tests the helper rather than the live scheduler host; host construction remains a separate CLI integration witness. [C-8 L472-L475]
- [C-8] (C-004/F-C8-054) `reachable_from` still glosses `get_search_mode` as an `AppState.search_mode live read`, while the rebaseline explicitly replaced that design with an SSB viewer read; the latter is the authoritative shipped design. [C-8 L181-L197 versus L301-L306]
- [C-9] (C-005/C9-D01) Add exact verdict/check/evaluator schemas, missing-transition behavior, aggregation uncertainty, policy defaults, trigger labels/firing conditions, split schemas/semantics, fit formula, CLI default mode, exact backend defaults, tree-unavailable escalation, decision rationale, …
- [C-9] (C-005/C9-E16) CLI startup replaces `autonomous_gate: None` with a real `AutonomousGate` configured from the parsed mode and otherwise-default config. [L131-L135]
- [C-9] (C-005/C9-E17) `GateMode::from_cli_args()` consumes C-5’s `--mode` value and defaults to `Interactive` when the flag is absent. [L134-L136]
- [C-9] (C-005/C9-E26) Produced/reachable surfaces are `AutoAdvanceEvaluator`, `EvidenceCheck`, `EvidenceVerdict`, `EscalationPolicy`, split types, two triggers, the fit method, CLI mode parser, `ResearchTree::split_step`, backend TOML/runtime types, and Phase-4G measurements, reachable from CLI con…
- [C-9] (C-005/C9-E31) Artifact/UI evidence requires populated measurements with completion true and a non-zero MonitorWindow `AutoAdvanceStats.last_5min_count`. [L230-L232]
- [C-9] (C-005/C9-E33) Acceptance additionally requires six escalation variants, the fit check on the actual substitution path, real gate construction, parsed backend stanzas, live split dispatch, the autonomous run to complete without prompting, the sidebar to display `AUTONOMOUS`, clean scoped bui…
- [C-9] (C-005/C9-E34) Implementation touches autonomous gate, model pool, research tree, task runner, CLI startup/runtime config, example TOML, Phase-4G metrics, and one new decision document. [L306-L328]
- [C-9] (SUMMARY/S-E0123) Passes a real TreeAutoAdvancePolicy from cli/main.rs instead of None. [L121]
- [C-9] (SUMMARY/S-E0126) Autonomous smoke builds a four-method Rust calculator with tests; acceptance requires at least one gate_decision whose source is AutoAdvanceEvaluator and a positive AutoAdvanceStats.last_5min_count in MonitorWindow. [L124]
- [C-10] (C-003/E-C10-010) `DiminishingReturns` is detected when error-count improvement is below 5% per step across a four-attempt window.
- [C-10] (C-003/E-C10-033) Add `get_stagnation_events`, returning newest-first rows for `stagnation_spinning_detected`, `stagnation_oscillation_detected`, `lateral_thinking_activated`, `goal_drift_warning`, and `watchdog_timeout`.
- [C-10] (C-003/E-C10-034) Collapsed `StagnationSection` shows timestamp, pattern/persona, detail, and existing-token badges: amber stagnation, blue lateral, orange drift, red watchdog.
- [C-10] (C-003/E-C10-045) Add the anti-spiral module, two contract files, five agent files, and decision doc; modify gate, task runner, phase metrics, Tauri commands, and drawer.
- [C-10] (SUMMARY/S-E0132) Adds StagnationDetection with a spin threshold of three, an oscillation window of four, and a minimum diminishing-improvement threshold of 5%. [L130]
- [C-10] (SUMMARY/S-E0141) Adds StagnationSection to OrchestrationDrawer. [L133]
- [C-10] (SUMMARY/S-E0142) Adds get_stagnation_events as a Tauri command. [L133]
- [C-11] (C-003/E-C11-024) Explicit review is required before merge for clone/fork syscall ordering, tasks 8-13, symlink creation, and raw/undocumented Win32 handle/API paths; PR description must carry the specified safety-critical flag.
- [C-12] (C-002/C12-E13) `get_per_level_verdicts` gains AppState, scans SSB for the task ID and action types `per_level_audit_verdict` plus `per_level_audit_cascade`, deserializes `Vec<LevelAuditVerdictJson>`, and replaces `Ok(Vec::new())`. [C-12:L98-L102]
- [C-12] (C-002/C12-E19) Writes are limited to navigator integration, the Tauri command, conditional prompt completion, two contract tests, and a C-12 decision document. [C-12:L239-L251]
- [C-12] (SUMMARY/S-E0166) Replaces get_per_level_verdicts returning an empty vector with live SSB-backed data using the B-36 AppState pattern. [L155]
- [C-14] (C-003/E-C14-002) `compact.rs` powers `/compact`, `ConversationRuntime::compact`, and `CompactionConfig` imports across eight CLI files.
- [C-14] (C-003/E-C14-008) `elai_orchestrator::usage::*` re-exports provider pricing so existing CLI imports stay valid; provider/type/Anthropic and live-CLI call sites are rewired.
- [C-14] (C-003/E-C14-024) Scoped orchestrator+CLI and full workspace compilation are recorded passing on 2026-05-17; banned-symbol grep and 78-status count are recorded done.
- [C-14] (C-003/E-C14-025) Adds provider pricing and PowerShell proof; modifies core/provider/orchestrator/CLI imports, POSIX proof, and migration manifest; deletion/metrics/test artifacts remain explicitly deferred.
- [C-15] (C-006/C15-E006) A 2026-05-23 closure rebaseline narrows actual closure to CLI plus knowledge batches; 52 post-closure leaks in orchestrator/plugins/tools/dashboard producer wiring are routed to an unauthored B-15-regression-burndown. [C-15 L31-L41]
- [C-15] (C-006/C15-E015) Identified CLI exception candidates are commands.rs cache expiry and live_cli.rs/runtime_cmd.rs SSB timestamp milliseconds; inject through session ClockProvider when possible, otherwise use the specified reason. [C-15 L119-L127]
- [C-15] (C-006/C15-E016) Clock injection batches are: A knowledge reranker/indexing; B rules self_discovery; C plugins tiered_analysis/import_extractor; D focus_tunnel streaming/mode; E top-level orchestrator session_dag/self_mod_layer1a/render adapters; F CLI commands/live_cli/remote_sidecar/runtime_…
- [C-15] (SUMMARY/S-E0198) Introduces an entropy-allow annotation convention for unavoidable SystemTime in cli/commands.rs cache TTL logic and live_cli.rs SSB timestamps. [L190]
- [C-15] (SUMMARY/S-E0200) Performs that injection in six batches: knowledge, rules, plugins, focus_tunnel, orchestrator top-level, and CLI. [L191]
- [C-16] (C-005/C16-E12) Cancellation checks before sleep and uses `tokio::select!` between cancellation notification and sleeping so `CancelHandle::cancel()` terminates the loop. [L73-L75]
- [C-16] (C-005/C16-E14) `trigger_cosmos_refresh` immediately returns `TriggerCosmosRefreshResponse { queued, artifacts_due }` while a one-shot `tick()` runs in the background with the same startup-injected dispatcher. [L79-L83]
- [C-16] (C-005/C16-E19) Observable smoke configures a daily knowledge artifact, performs a cloud research query, expects queued/updated/query-stage SSB evidence, fresh status within a tick, live drawer timestamp, and an `AVAILABLE_COSMOS` briefing block. [L127-L136]
- [C-16] (SUMMARY/S-E0217) Replaces trigger_cosmos_refresh NotImplemented behavior with a one-shot Tokio task. [L204]
- [C-17] (C-004/D-C17-004) Resume is ordered after initialization/before dispatch; shutdown must remain reachable from both CLI signal paths and dashboard exit. [Summary L382-L384; C-17 L76-L84, L169-L172]
- [C-17] (C-004/F-C17-007) CLI startup calls `auto_resume_indexed_projects` after initialization and before task dispatch; both CLI and dashboard must remain callers. [C-17 L76-L79, L169-L172]
- [C-17] (C-004/F-C17-008) CLI shutdown handles cross-platform `tokio::signal::ctrl_c` and Unix-only `SignalKind::terminate`, routing both to `graceful_shutdown`; dashboard `ExitRequested` remains another required reachability path. [C-17 L81-L84, L169-L172]
- [C-17] (C-004/F-C17-025) Primary reachability is CLI startup/shutdown, `/research`, watcher service, and dashboard ActivityFeed. [C-17 L164-L168]
- [C-17] (C-004/F-C17-029) Deterministic checks assert source symbols/files, no norm/proxy remnants, CLI lifecycle wiring, contract/test presence, scoped crate compilation/tests, and positive tier-A timing. [C-17 L185-L231]
- [C-17] (C-004/F-C17-032) The explicit edit inventory spans knowledge DKG/BM25/A-RAG, CLI main/live CLI, orchestrator watcher/startup, knowledge/contracts tests, scale scripts/metrics, ActivityFeed, and the decision artifact. [C-17 L270-L286]
- [C-17] (SUMMARY/S-E0225) Wires auto_resume_indexed_projects into CLI startup. [L215]
- [C-18] (C-001/C18-008) Five dashboard contracts map D1=get_sidecar_health, D2=run_planning_task, D3=run_research, D6=reindex_now, D11=sidecar_search; each JSON round-trips its request and type-checks the command, with only a TODO for ELAI_LIVE_SIDECAR integration. (C18:L89-100)
- [C-19] (C-002/C19-E47) `test_branch_tree_onmerge_onclose_props_present` compile-checks both props against Tauri command signatures rather than rendering Svelte. [C-19:L188-L192]
- [C-19] (C-002/C19-E49) Explicit executions cover adversarial IPC/research/session suites, feasibility suites, and plugin benches; smoke expects traversal blocking with no `SANDBOX_VIOLATION` row, T1/T2/T3 timing output, and A1-A8 within 30s. [C-19:L220-L242]
- [C-19] (C-002/SD-C-19-04) Addition: The source specifies all A1-A8, T9/T13-T16, four baby-step, four research, and four SessionDag/BranchTree behaviors, including paused-time and 30-second anti-hang constraints. [C-19:L121-L192]
- [C-19] (SUMMARY/S-E0267) Adds B-9 named tests for session merge/close slash commands and BranchTree fork/merge/close property contracts. [L253]
- [C-20] (C-003/A-C20-02) A single canary/task is said to populate p50/p99, which does not define a sample distribution; percentile calculation/interpolation and warm-up rules are absent throughout (L124-L136).
- [C-20] (C-003/C-C20-04) The plan says 11 backends, but acceptance quantifies seven absent CLI adapters and names only two live backends; disposition of the other two is not enumerated (L124-L136, L272-L273).
- [C-20] (C-003/C-C20-05) Files-touched paths omit the otherwise pervasive `.elai_cc/` prefix for CLI/orchestrator additions, leaving repository-relative placement inconsistent (L172-L201 versus L292-L307).
- [C-20] (C-003/DELTA+-C20-1) Adds exact CLI grammar, run-summary schema, sample counts, fixtures, percentiles, limits, skip reason, static-cost provenance, and all 1F/1G/1H fields.
- [C-20] (C-003/E-C20-001) Add `elai measure` (or equivalent), populate placeholder metrics from actual measurements, gate a debug batch delay, and include a live cloud Sonnet 4.6 smoke.
- [C-20] (C-003/E-C20-017) 4G times `set_mode` over 100 calls and `decide(Evidence::all_passed)` over 200 calls, reporting p50/p99.
- [C-20] (C-003/E-C20-023) 4M health-checks each of 11 backends for p50/p99 and marks a backend `Unavailable` when its CLI is absent.
- [C-20] (C-003/E-C20-027) Hybrid CLI execution must populate actual `AnthropicApi` and `Sidecar` rows and append retry-derived fallback rate (0.0 is valid if no retry).
- [C-20] (C-003/E-C20-038) Measurement is reachable from CLI, existing scale-budget CI, and phase auditors.
- [C-20] (C-003/E-C20-042) 4M acceptance expects seven CLI adapters to report `Unavailable` when binaries are absent.
- [C-20] (SUMMARY/S-E0270) Adds the elai measure command. [L260]
- [C-21] (C-004/F-C21-011) Layer1A `DIVERGING` calls nonblocking `trigger_dep_audit_background()`, which starts detached `elai dep-audit --output-format toml` and writes `metrics/dep_audit_latest.toml`. [C-21 L92-L97]
- [C-21] (C-004/F-C21-018) The SWE-bench seed corpus contains `trigger_task.toml`, `task_01.toml`…`task_10.toml`, and one-paragraph `README.txt`; the 11 TOMLs use `id`, `description`, `expected_output`, `timeout_s`, `stage`, and `is_canary`. [C-21 L124-L131]
- [C-21] (C-004/F-C21-019) The trigger task is a known-answer canary gating `StageReport.trigger_task_passed`; the ten tasks use SWE-bench Lite problem statements under MIT terms and include no solution patches. [C-21 L124-L131]
- [C-21] (C-004/F-C21-032) “STRICT mode” is required, but the specified command shows no environment variable or CLI switch; the source does not identify how strict mode is activated. [C-21 L54-L57]
- [C-21] (SUMMARY/S-E0297) Creates a SWE-bench seed corpus directory with trigger_task.toml, task_01 through task_10 TOMLs, and README.txt. [L281]
- [C-22] (C-001/C22-002) Nine verified gaps are named: SystemProbe type mismatch, missing GPU UUID, missing StubResult fields, absent ResourceDelta, None production construction, event-name drift, forbidden kill_on_drop, absent panel/unwired flag, and six absent metrics. (C22:L14-50)
- [C-22] (C-001/C22-011) Replace None with Some(Box::new(DefaultFeasibilityProber::new())) in cli/main.rs and runtime_setup.rs, using in-scope AppConfig if required and exporting the constructor if necessary. (C22:L122-128)
- [C-22] (C-001/C22-012) FeasibilityPanel takes FeasibilityReport|null and shows probe_count, novel_step_count, summed alternatives, optional skip notice, and per-concept decision/stub hashes/stderr/resource VRAM accordion; null shows “Feasibility probe not run.” (C22:L130-140)
- [C-22] (C-001/C22-013) The panel consumes get_feasibility_report, replaces the ReasoningDrawer placeholder, and clears d.unwired.feasibility=false. (C22:L140-143)
- [C-22] (C-001/C22-016) Reachability is cloud /research→DefaultFeasibilityProber, ReasoningDrawer→Tauri report, and live alternative/escalation SSB rows. AppConfig and explicit kill are consumed symbols. (C22:L163-188)
- [C-22] (C-001/C22-017) Production bans are zero kill_on_drop, no new sysinfo dependency/resource path, and no mocked/hardcoded panel data. (C22:L184-188)
- [C-22] (C-001/C22-018) The candle-rs smoke requires system_probe and feasibility_report, optional alternative rows, real panel counts, non-zero ResourceDelta VRAM, and a skip only when RESOURCE_MARGIN_THRESHOLD_PCT<20%; zero kill_on_drop is also observable evidence. (C22:L189-200)
- [C-22] (C-001/C22-019) Twelve static checks cover the rename/schema, event constants, two construction paths, component/flag, six metrics, and workspace compile. (C22:L201-253)
- [C-22] (C-001/C22-021) Work modifies five Rust/config/test files plus ReasoningDrawer/drawers state, adds FeasibilityPanel, modifies phase_4F.toml and possibly contract tests, and creates C-22-decisions.md. (C22:L285-307)
- [C-22] (SUMMARY/S-E0311) Wires a real DefaultFeasibilityProber in cli/main.rs and runtime_setup.rs instead of None. [L295]
- [C-22] (SUMMARY/S-E0312) Creates FeasibilityPanel.svelte. [L296]
- [C-22] (SUMMARY/S-E0314) Mounts FeasibilityPanel in ReasoningDrawer and clears unwired.feasibility. [L296]
- [C-22] (SUMMARY/S-E0316) Cloud research smoke uses an unfamiliar candle-rs library, expects non-null VRAM ResourceDelta, and requires FeasibilityPanel to render. [L298]
- [C-23] (C-006/C23-A001) BranchTree's onfork callback receives a name but the invocation payload key is branchId; the Tauri parameter name/rename contract is not reconciled. [C-23 L54-L56, L65-L68]
- [C-23] (C-006/C23-E001) Objective: close the B-9 partial/missing surface by adding merge/close CLI actions, live BranchTree, real Tauri mutations, load_handoff, root-to-tip ancestry, 100-run determinism, handoff-schema verification, and branch-event banners. [C-23 L1-L6]
- [C-23] (C-006/C23-E003) Verified gaps are: parser supports only list/switch/fork; BranchTree is unmounted; three Tauri commands return CLI-delegation errors because AppState lacks SessionDag; /continue only prepopulates; parent_chain is reversed; and only a two-seed determinism comparison exists. [C-…
- [C-23] (C-006/C23-E010) SessionsDrawer imports and mounts BranchTree beneath/replacing the branch list, passes $sessionBranches, invokes fork/merge/close with branchId payloads, and refreshes the list after success. [C-23 L61-L69]
- [C-23] (C-006/C23-E017) Consumed symbols are create_branch, merge_branch_with_ssb, close_branch, load_handoff, get_session_branches, AppState ssb_writer/DAG configuration, and BranchTree callback props. [C-23 L117-L124]
- [C-23] (C-006/C23-E018) Reachable surfaces are /session merge <branch_id>, /session close <branch_id>, SessionsDrawer BranchTree, and CommandWindow /continue. [C-23 L125-L128]
- [C-23] (C-006/C23-E019) Stub bans remove every “use CLI: /session …” return and forbid a prepopulate-only /continue path. [C-23 L129-L136]
- [C-23] (C-006/C23-E020) The observable lifecycle requires Checkpoint plus branch_* rows, live BranchTree state, and actual load_handoff behavior. [C-23 L137-L142]
- [C-23] (C-006/C23-E021) Deterministic/static evidence checks parser arms, absence of stub text, BranchTree import/mount, load_handoff, chain.reverse, root index zero, a 100-loop, all branch action strings, and both session-dag test commands. [C-23 L144-L183]
- [C-23] (C-006/C23-E023) Cloud smoke runs a 20-file task past 85% context, observes create_handoff, resumes in another terminal through /continue/load_handoff, then exercises fork/merge/close while the live drawer reflects state. [C-23 L219-L226]
- [C-23] (C-006/C23-E024) Planned file surface is CLI commands, session_dag.rs, Tauri commands, SessionsDrawer, CommandWindow, SettingsWindow verify/modify, optional session_dag_contract.rs, and C-23-decisions.md. [C-23 L228-L244]
- [C-23] (SUMMARY/S-E0319) Replaces “use CLI” stubs in fork_session_branch, merge_session_branch, and close_session_branch Tauri commands with real SessionDag calls. [L305]
- [C-23] (SUMMARY/S-E0321) Mounts BranchTree.svelte in SessionsDrawer. [L306]
- [C-23] (SUMMARY/S-E0322) Wires BranchTree’s onfork, onmerge, and onclose callbacks to live IPC. [L306]
- [C-24] (C-005/C24-A02) CLI does not expose `project_id` even though dispatcher input requires it; implicit current-project resolution is not stated. [L96-L112]
- [C-24] (C-005/C24-A03) The consumed graph store mentions `trigger_refresh`, but the click workflow names only `request_impact`, `set_mode`, and `openWindow`; whether impact implicitly refreshes is unspecified. [L127-L133, L178-L179]
- [C-24] (C-005/C24-D01) Add all four JSON schemas/defaults, the full-graph `> threshold` branch, exact submit string, CLI modes/max-node default, dispatcher request/response schema, exact Cytoscape shapes/colors, and store-call ordering. [L72-L133]
- [C-24] (C-005/C24-D03) Qualify the summary’s CLI example: the original does not define where its `SsbActionType` positional argument maps. [L96-L112, L248-L249]
- [C-24] (C-005/C24-E01) The two-session plan closes B-29/B-33 reachability, incremental update, CLI/dispatcher export, decomposition styling, blast-radius UI, and missing tests; it depends on C-17 and is parallel-safe with C-21/22/23/25. [L1-L9]
- [C-24] (C-005/C24-E05) KnowledgeDrawer’s impact panel adds Svelte-5-state target input and `Run`, invoking `submit_task` with `"/symbol_impact target=" + target` and rendering the returned `blast_radius`. [L89-L94]
- [C-24] (C-005/C24-E06) CLI gains `SlashCommand::GraphMermaid { mode: GraphMode, max_nodes: usize }`, delegates via IPC to `export_graph_mermaid`, and prints returned Mermaid to stdout. [L96-L103]
- [C-24] (C-005/C24-E07) CLI modes are `file`, `symbol`, and `decomposition`; `max_nodes` defaults to 200. [L101-L103]
- [C-24] (C-005/C24-E18) CLI export must use the real generator, and dispatcher export must delegate rather than duplicate generation. [L194-L197]
- [C-24] (C-005/C24-E20) Cloud rename smoke expects a `SymbolQuery` row with `tool="symbol_impact"`, target `SsbActionType`, and non-empty blast radius; UI/artifact evidence includes decomposition styling, deterministic zero-LLM CLI Mermaid, Tauri parity, and at least ten callers in the drawer. [L204-…
- [C-24] (C-005/C24-E21) Acceptance raises the catalog to at least ten tools, tests both threshold branches/event, checks the form, CLI, cross-layer deterministic export, guarded styles, shared-store action, both four-case suites, scoped Rust builds, Svelte check, and the decision document. [L255-L291]
- [C-24] (C-005/C24-E22) Implementation modifies production RPC catalog, indexing service, dispatcher, CLI, three Svelte surfaces, and adds two test files plus one decision artifact. [L293-L311]
- [C-24] (SUMMARY/S-E0334) Adds a target form and Run button to KnowledgeDrawer’s IMPACT ANALYSIS panel. [L320]
- [C-24] (SUMMARY/S-E0343) That action calls graph state request_impact and opens a graph window. [L324]
- [C-25] (C-004/F-C25-034) CLI adds `SlashCommand::Pipeline(PipelineSubcommand)` with `List` and `Reset`. [C-25 L129-L132]
- [C-25] (C-004/F-C25-037) Token tracking is reached from cloud response round trips; crystallization from ResearchTree post-step; compiled execution from pipeline factory entry; background management from autonomous Bash; and cache administration from CLI. [C-25 L162-L177]
- [C-25] (C-004/F-C25-043) Ten deterministic check groups verify accounting, events, manager functions, compiled types/store/TTL, try-template wiring, source schema, CLI commands, workspace/scoped tests, and the cache artifact. [C-25 L193-L236]
- [C-25] (SUMMARY/S-E0369) Adds /pipeline list and /pipeline reset CLI commands. [L337]
- [C-27] (C-002/C27-E01) Objective combines determinism tier T(-1) and wiring-contract tier T-0: hardware/seed/cache/fingerprint determinism, drift evidence, contract parsing/auditing, CLI, and CI; estimated 2-3 sessions. [C-27:L1-L9]
- [C-27] (C-002/C27-E20) A Windows `wiring-audit.yml` builds release CLI and audits configured contracts on push and pull request to main, returns nonzero on violations, and skips `[ci skip]` commits. [C-27:L138-L142]
- [C-27] (C-002/C27-E24) Explicit bans: no alternate digest; no scanning test/contract paths; no drift row on equality; no timeout panic; no panic on missing config; and exact CLI exit codes. [C-27:L189-L195]
- [C-27] (C-002/C27-E27) Acceptance requires payload round-trips, exact violation kinds, correct first divergence, reachable/unreachable evidence, CLI behavior, four smoke scenarios, Windows CI triggers, clean workspace, ≥10 scoped unit tests, tier-0 contract pass, and the three-decision document. [C-…
- [C-27] (C-002/C27-E28) Adds determinism/wiring modules, CLI dispatch, CI workflow, contract tests, decision doc, and modifies SSB, CLI commands, and TOML. [C-27:L288-L310]
- [C-27] (C-002/SD-C-27-02) Addition: Wiring contracts have a concrete field schema; CLI parses Markdown blocks, prints exact PASS/FAIL forms and exit codes; CI uses Windows and `[ci skip]`; observables poll at one-second cadence and fail with typed evidence. [C-27:L90-L148]
- [C-28] (C-005/C28-A06) `get_mutation_gauge` is explicitly a stub until live state wiring, yet acceptance otherwise describes a live cloud gauge; the handoff that makes it non-stub is not a separate task. [L140-L150, L272-L280]
- [C-28] (C-005/C28-D01) Add exact record/result/cache/config/gauge schemas, runner commands, partial-cap behavior, event cardinality exclusions, five predecessor checks, and every contract-test case. [L52-L169]
- [C-28] (C-005/C28-D04) Correct the summary’s claim that both tools are non-Cargo CLI installs: `cargo-mutants` is a Cargo-installed CLI, while `mutmut` is a Python package in sidecar `pyproject.toml` dev dependencies. [L24-L26, L46-L50, L257-L258]
- [C-28] (C-005/C28-E02) The source pins `cargo-mutants 27.0.0` as an installed CLI/dev tool, explicitly not a Cargo dependency, and `mutmut 3.5.0` in the sidecar Python dev environment. [L24-L26, L46-L50, L257-L258]
- [C-28] (C-005/C28-E22) MonitorWindow polls `get_mutation_gauge` every five seconds and renders green at/above threshold, amber below it, and grey for no result; the Tauri stub returns `None` until backend session-state wiring exists. [L145-L150]
- [C-28] (C-005/C28-E26) Normal observable behavior has zero survivor rows, score at least 70, transition proceeds, green gauge, and faster repeat from cache hits. [L198-L203]
- [C-28] (C-005/C28-E27) The adversarial tautology scenario yields score 0, a blocking finding, blocked transition, and amber gauge; replay expects every second-run mutant to be a cache hit and `total_run == cache_hits`. [L239-L253]
- [C-28] (C-005/C28-E29) Acceptance covers exact dependency installs, tier/order, runner/types/formula, persistent TTL cache, event cardinality, four config keys/defaults, hard gate ordering, three-state gauge, null stub, no unwrap/expect, 4+3 contracts, normal/adversarial smoke, and faster all-cache …
- [C-28] (C-005/C28-E30) Implementation adds mutation code and two test files and modifies tier dispatch, auditor, SSB, runtime config/TOML, dashboard types/window/Tauri state, plus a decision document. [L283-L297]
- [C-28] (SUMMARY/S-E0420) Pins cargo-mutants 27.0.0 and mutmut 3.5.0 as CLI tools, not Cargo dependencies. [L369]
- [C-28] (SUMMARY/S-E0437) MonitorWindow displays a mutation gauge: green at or above threshold, amber below threshold, and grey for null. [L376]
- [C-28] (SUMMARY/S-E0438) Adds a get_mutation_gauge Tauri-command stub for that gauge. [L376]
- [C-29] (C-003/DELTA!-C29-2) “Confirmation gate” is stronger than the original algorithm: the source explicitly gates on count, not on the persisted `confirmed` flag, and may emit confirmation from both gate and command.
- [C-29] (C-003/E-C29-022) Inline PlanWindow panel lists existing pairs on step selection; its three fields are NL intent, assertion, and auto-filled step ref.
- [C-29] (C-003/E-C29-023) Add calls `add_intent_pair`; Confirm is enabled at count ≥3 and calls `confirm_intent_pairs`; both map `Result<(),String>` errors to toast.
- [C-30] (C-001/C30-025) WIRING_CONTRACT exposes seven surgical/error types, decomposer, three cache types, five events, four tools, and VRAM config from CLI task/research/tool entry points. (C30:L185-212)
- [C-32] (C-005/C32-A08) Task 7’s “File” line names only Tauri `lib.rs` even though the task is a ReasoningDrawer renderer; the concrete drawer path is deferred to “locate” in the files list. [L162-L170, L332-L333]
- [C-32] (C-005/C32-D01) Add every declared field/error/event payload, exact lazy patterns and reason strings, single-expression exemption, tree-sitter warning behavior, per-stage mechanics, retry context, per-kind bridge text, user opt-in, and the default-on drawer filter. [L56-L175]
- [C-32] (C-005/C32-D03) Correct “14 contract tests” to “15 enumerated despite a stated/expected count of 14,” and flag the crate/path mismatch. [L177-L204, L291-L294]
- [C-32] (C-005/C32-E26) `AcceptAndComplete` requires ratio at least 0.5 **and** a user opt-in flag. [L149-L151, L306-L307]
- [C-32] (C-005/C32-E27) MonitorWindow calls `get_completeness_ratio() -> f32`, renders red below .5/yellow .5..<.9/green >=.9, and updates from `skeleton_flushed` without a new dependency. [L154-L160]
- [C-32] (C-005/C32-E28) ReasoningDrawer renders blank/verdict/reason, color maps PASS/WARN/BLOCK to green/yellow/red, and has a default-on `Show MiniCritic` filter using the existing SSB stream. [L162-L170, L251-L254]
- [C-32] (C-005/C32-E36) Implementation adds four orchestrator modules and one contract file, modifies task runner/SSB/Monitor/Tauri, locates and modifies ReasoningDrawer, and adds a decision artifact. [L319-L335]
- [C-32] (SUMMARY/S-E0538) MonitorWindow shows completeness ratio red below 0.5, yellow from 0.5 to below 0.9, and green at least 0.9. [L432]
- [C-32] (SUMMARY/S-E0539) Adds get_completeness_ratio. [L432]
- [C-32] (SUMMARY/S-E0540) ReasoningDrawer renders mini-critic rows and offers a Show MiniCritic toggle. [L432]
- [C-33] (C-006/C33-E023) Surgical CLI/evidence is --surgical, surgical_plan_created {complexity_class, blast_radius_files}, surgical_loc_warning {loc_estimate, threshold}, and an rg check; target spec length 80-120. [C-33 L121-L127]
- [C-34] (C-005/C34-A01) Cargo feature activation is compile-time, while `--empire` is described as enabling the feature path at runtime; the build/distribution mechanism that makes the code available to the flag is not specified. [L42-L45, L77-L78]
- [C-34] (C-005/C34-D02) Add the compile-time/runtime flag ambiguity, event payload mismatch, possible repair-event feedback loop, contract path conflict, and absence of endpoint auth/retry/privacy contracts. [L42-L86]
- [C-34] (C-005/C34-E13) CLI adds `--empire`, which activates the Empire path and logs connection status at startup. [L77-L78]
- [C-34] (C-005/C34-E17) Observable smoke expects connected and capability-invoked rows with non-null identifiers and a green panel displaying the capability. [L113-L117]
- [C-34] (C-005/C34-E18) Deferred deterministic checks require the feature declaration, all seven types, all five events, Empire package tests, CLI flag, and panel file. [L119-L139]
- [C-34] (C-005/C34-E19) Deferred acceptance requires conditional-only compilation, no placeholder types, preserved IPC tests, independent pathology handling, all events and migration, TDD-before-wiring evidence, contract success, reachable/validated MCP, graceful missing-pipe error, live panel, and c…
- [C-34] (C-005/C34-E20) Planned files include six Empire modules, shared IPC/task-runner/CLI/workspace/config edits, panel, orchestrator contract test, and the source-grounding artifact; several paths remain explicitly TBD. [L160-L175]
- [C-35] (C-005/C35-A06) The visible CLI flag and implemented `ELAI_TOOL_GATE_OFF` environment reader are both reported, but their precedence and mapping at bridge construction are not defined. [L120-L122, L212-L214, L237-L238]
- [C-35] (C-005/C35-E19) The acceptance section declares the library tier **SHIPPED-ON-V2**, including gate/cache/manifest/config, four events, five unit + three contract tests, mounted inspector, and CLI parse/warn. [L199-L205]
- [C-35] (C-005/C35-E22) Dashboard completion includes inspector mounting in SettingsWindow, `get_tool_gate_snapshot` and `list_tool_gate_rows` registered with Tauri, panel `invoke()` calls, and two DTO round-trip tests. [L210-L214]
- [C-35] (SUMMARY/S-E0588) Adds --tool-gate-off as a comparison/bypass flag. [L602]
- [C-37] (C-006/C37-E008) The Qwen2.5-Coder-7B NF4+FA2 pilot and detection-rate delta are GPU/RunPod-dependent and deferred to the C-54-adjacent window. [C-37 L65-L67, L164-L166]
- [C-37] (C-006/C37-E021) Planned writes include the harness, contract, conditional migration/config/runtime changes, two SSB actions, paper/session artifacts, summary annotation, and a C-26 verification-report note; all acceptance boxes are open. [C-37 L159-L194]
- [C-38] (C-004/F-C38-004) The fresh comparison extracts graph node/edge types, weighting, self-loop policy, top-k selection, and chat-context multiplier from current `get_ranked_tags`/`get_ranked_tags_map` behavior. [C-38 L61-L67]
- [C-39] (C-006/C39-E001) Objective: let CLI task and explore invoke registered MCP tools without the dashboard, using the existing HTTP JSON-RPC 2.0 bridge semantics for transport parity with Apache-2.0 OpenHands (cited at about 38K stars). OpenHands supplies the fastmcp HTTP pattern, not a standalone…
- [C-39] (C-006/C39-E004) Baseline gaps are no CLI bridge coverage, no CLI visibility of externally registered MCP tools, and dashboard/CLI CI exercising different transports. [C-39 L28-L38]
- [C-39] (C-006/C39-E007) New cli/src/mcp_client.rs is a minimal localhost HTTP JSON-RPC 2.0 client with list_tools, invoke_tool(name,args), and lazy get_tool_schema(name). [C-39 L65-L68, L168-L170]
- [C-39] (C-006/C39-E009) If no listener exists, CLI launches the dashboard bridge backend as a co-process on a random localhost port and tears it down at process exit. [C-39 L74-L77, L171-L173]
- [C-39] (C-006/C39-E012) A blocking CI matrix runs every existing CLI integration test once per route and compares outputs, permitting only expected dispatch-specific SSB action-name differences. [C-39 L79-L82, L125-L130, L176-L177]
- [C-39] (C-006/C39-E015) A shared Dispatch trait has McpClientDispatch and InProcessDispatch implementations; startup chooses one based on the flag. [C-39 L95-L98, L109-L113]
- [C-39] (C-006/C39-E017) CLI docs gain usage/help for --via-mcp; the listed new docs also include --export-ssb. [C-39 L104-L105, L199]
- [C-39] (C-006/C39-E022) Deterministic gates check modules, both impls, flag, actions, workflow, contract test, and local ci-mcp-parity.sh. [C-39 L140-L164]
- [C-39] (C-006/C39-E023) File surface adds CLI client/lib/tests, three orchestrator dispatch modules, parity script/workflow, source artifact and docs, and modifies main/commands/SSB. [C-39 L183-L202]
- [C-39] (SUMMARY/S-E0621) Adds MCPClient in cli/src/mcp_client.rs. [L630]
- [C-39] (SUMMARY/S-E0624) Adds a CI parity job that executes every CLI test once per dispatch route. [L630]
- [C-40] (C-006/C40-E012) The honesty section identifies ELAI/session, states mutation score and wiring pass/fail, and instructs reviewer to verify the acceptance checklist; marketing language is forbidden. [C-40 L96-L98, L169]
- [C-40] (C-006/C40-E023) CLI flags flow through the session-merge path; adapter and watcher lifetime is bounded to the session and watcher tears down at session end. [C-40 L132-L134]
- [C-40] (C-006/C40-E027) Observable lifecycle for merge --to-pr --watch-reviews --base main is PR-created/body-posted, poll ticks while active, comment-received, stdout URL, and optional panel state. [C-40 L171-L178]
- [C-40] (C-006/C40-E030) File surface adds seven github_pr module files, CLI command, contract, gh fixture, default template, optional panel; modifies feature config, CLI, merge path, SSB schema, project config, and docs. [C-40 L232-L251]
- [C-40] (C-006/C40-I001) The required config flag plus --to-pr command constitutes the intended authorization boundary for external write, but an explicit one-time “open this PR?” confirmation is not stated.
- [C-41] (C-001/C41-015) Checks require directory/index, at least nine registry entries, three runtime types/events, TDD accuracy, and contract tests. Acceptance additionally removes inline routing, enforces versioning, requires non-null event fields, tests-before-wiring, and a rendered panel. (C41:L1…
- [C-42] (C-002/C42-E03) External consumers include frameworks, IDEs, CLI, future Composer 2, and GitHub PR routing; an LLM-controlled prompt is treated as potentially malicious and cannot rely on dashboard trust. [C-42:L21-L34]
- [C-42] (C-002/C42-E17) `AuthzPanel.svelte` is explicitly **PLANNED** to show real-time check/reject rates and click-through reason histograms; acceptance nevertheless asks it to render. [C-42:L105-L107; C-42:L186-L186]
- [C-42] (C-002/SD-C-42-03) Correction: `AuthzPanel.svelte` is PLANNED, not unconditionally added; acceptance and summary overstate its current status. [C-42:L105-L119; C-42:L186-L200]
- [C-42] (SUMMARY/S-E0659) Adds AuthzPanel.svelte. [L651]
- [C-44] (C-005/C44-D01) Add the conditional infeasibility closure, provisional config values, exact invoke/error contract, all five TDD cases, missing-key versus 503 behavior, 24-hour panel scope, and three benchmark arms. [L29-L166]
- [C-44] (C-005/C44-E15) Conditional acceptance requires an API-status record; operational path adds grounded interface, config/backend/errors/cost/failover/events/tests/panel/contracts, while infeasible path writes `composer2-deferral.md` with API status and revisit trigger. [L130-L144]
- [C-45] (C-003/E-C45-003) For each role/task, compare role available against role forcibly dropped, route around the drop with merged fallback, and aggregate success deltas over N tasks.
- [C-47] (C-004/D-C47-005) Per-project call shapes/string values/session IDs have no specified redaction, encryption, retention, access, isolation, or deletion policy. [Summary L1133-L1140; C-47 L25-L35, L47-L51]
- [C-47] (C-004/F-C47-022) The enabled flag’s default is not stated. [C-47 L88-L90, L125-L127]
- [C-48] (C-003/E-C48-004) T0 includes CLI/dashboard/explicit user writes; T1 includes local source and git history; T2 includes WebFetch/paper ingestor; T3 is unreviewed model output.
- [C-51] (C-001/C51-013) Produced surfaces are the vendor/corpus, PolyglotAdapter, six language runners, CLI, and three events; consumed surfaces are the existing agent pipeline, C-11 sandbox, configured backends, and SSB. (C51:L86-101)
- [C-51] (C-001/C51-017) Adds vendor/manifest, adapter, six shims, TDD/contract tests; modifies CLI and SSB schema. (C51:L135-145)
- [C-51] (C-001/C51-018) “No benchmark run in C-51” coexists with a CLI described as refusing a full sweep only when GPU is absent, so GPU-present behavior during this patch is unclear. GPU detection, language distribution, mock semantics, output location, concurrency, timeout/retry, and event payload…
- [C-53] (C-006/C53-A001) “Subscribe” and “major release” have no automation, cadence, owner, notification channel, or semantic-version policy. [C-53 L22-L26, L47-L50]
- [C-53] (C-006/C53-E019) Done gates check the watch doc/entry, bench file, CLI flag, procedural fixture, two actions, compilable local baseline, paper artifact, RunPod pilot, matrix outcome, and any default-off route. [C-53 L109-L145]
- [C-54] (C-004/D-C54-005) Interrupted sweeps discard partials and restart fully; normal CI is excluded, and manual CLI launch is required. [Summary L1263-L1274; C-54 L135-L141, L161-L166]
- [C-54] (C-004/F-C54-031) File output includes corpus/manifest/configs, task-runner harness/test, three scorecards, methodology, results, CLI subcommand, and SSB schema. [C-54 L219-L234]
- [C-55] (C-006/C55-E016) CLI reachability is elai benchmark aider-polyglot --config <name> --runpod-id <id> and --submit only after both configs complete. [C-55 L92-L95]
- [C-55] (C-006/C55-E020) File surface adds two configs, two post-run scorecards, methodology/results, benchmark CLI/submit, and three SSB actions. [C-55 L128-L138]
- [C-56] (C-001/C56-012) CLI reachability is elai benchmark codebase-memory --runpod-id; outputs are CodebaseMemoryAdapter, scorer, session scorecard/methodology, and repository results doc. (C56:L74-90)
- [C-56] (C-001/C56-016) Adds the corpus, manifest, queries, adapter/scorer, three result artifacts, paper artifact, CLI command, and three SSB schema types. (C56:L126-138)
- [C-57] (C-002/C57-E14) CLI entrypoints are `elai benchmark openhands-index|swe-cycle|swe-atlas --runpod-id <id>` and suite mode `elai benchmark suite --runpod-id <id>`. [C-57:L98-L100]
- [C-57] (C-002/SD-C-57-01) Addition: The index omits exact default composition (Sonnet 4.6/Composer 2/local auditor), optional local-only arm, pass@1/no-cherry-picking discipline, CLI forms, external-baseline citation rule, and RunPod cost gate. [C-57:L36-L39; C-57:L59-L65; C-57:L98-L105; C-57:L120-L132]
- [C-58] (C-001/A-005) C-2 leaves the AgentStatusChanged owner and anti-spiral payload-struct granularity open; Cosmos tick can also be absent from the smoke window. (C2:L66-70, L103-109, L187-192)
- [C-58] (C-001/A-012) C-22's parallel-file explanation misidentifies C-21/C-23 ownership, and the Tauri command path is inconsistent. (C22:L8-12, L130-179)
- [C-58] (C-001/A-017) C-41 omits scoring weights/threshold/ties, cache design, semver negotiation, fall-through chosen_skill semantics, panel path, and manual-override authorization/evidence. (C41:L68-128, L174, L196)
- [C-58] (C-001/A-018) C-51 is infrastructure-only and publication is C-55; a mock five-task dry-run is allowed. Whether a GPU-present non-dry CLI may nevertheless run the full suite before C-55 is internally unclear. (C51:L1-7, L78-82, L120-149)
- [C-58] (C-001/C58-003) ELAI roles are Executor, Auditor, Researcher, Designer, PromptEngineer, ContextCompiler, MemoryWriter, MediaValidator; AgentForge roles are Planner, Coder, Tester, Debugger, Critic; compare solve rate and cost/task on the same corpus/hardware. (C58:L21-32)
- [C-58] (C-001/C58-014) Consumers are SWE-bench Lite, optionally/conditionally C-45 attribution, C-56 query corpus, and knowledge reranker; CLI has separate head-to-head and reranker-eval commands with runpod-id. (C58:L108-115)
- [C-58] (C-001/C58-018) Adds vendor, adapters/reranker trait modules, two scorecards, combined methodology/results, two paper artifacts, two CLI commands, four SSB schema types, and a knowledge module export. (C58:L158-172)
- [C-58] (C-001/X-005) C-22 turns the feasibility path from None/unwired into production data, two new events, panel rendering, explicit child termination, and measured phase_4F evidence; C-19 conditionally owns the adversarial tests. (C22:L122-200, L255-283)
- [C-59] (C-003/DELTA+-C59-1) Adds the exact four configurations, per-cell triple, eight scorecard glob families, complete limitation taxonomy, loss→one-target rule, TTS positive/archive behavior, CLI, and verification-report annotations.
- [C-59] (C-003/E-C59-006) Cost-per-task analysis reports ELAI dollars/task across all benchmarks/configs; use a plot only if dashboard support exists, otherwise a table.
- [AC] (C-002/A22) C-57 official scoring fallback. OpenHands version/task snapshot, SWE-Cycle adapter protocol, and precise behavior when SWE Atlas's official judge/corpus is unavailable remain open. [C-57:L41-L57; C-57:L67-L71]
- [AC] (C-002/C02) C-5 shipped provider flag versus stale plan clauses. | Evidence: Status and amended Task 3 say `--provider`; `WIRING_CONTRACT`, acceptance, Files, and smoke still say `--mode cloud|hybrid|local`. The status block explicitly says do not re-implement the stale spelling, so `--pr…
- [AC] (C-002/C05) Provider mode used by multiple smoke commands conflicts with C-5's corrected CLI axis. | Evidence: C-1/C-4/C-12/C-19/C-27 use `elai /task --mode cloud`, while C-5 says `--mode` is gate mode and provider is selected by `--provider`. [C-1:L228-L232; C-4:L195-L196; C-12:L152-L155…
- [AC] (C-002/C07) C-12 describes `get_per_level_verdicts` both as a live reader with no producer rows and as a literal `Vec::new()` stub. | Evidence: [C-12:L26-L31] versus [C-12:L51-L55; C-12:L98-L102]. Task 5/acceptance clearly require replacing the literal stub.
- [AC] (C-002/C09) C-27 CLI type/file ownership is inconsistent. | Evidence: Task 9 locates `WiringAuditCmd` in `crates/cli/src/wiring_audit.rs`; the Files list says `wiring_contract.rs` also contains `WiringAuditCmd/Report/Error`. [C-27:L128-L136] versus [C-27:L293-L298].
- [AC] (C-002/C10) C-42 dashboard status conflicts with its acceptance. | Evidence: Task/WIRING/Files label `AuthzPanel.svelte` PLANNED, but acceptance requires it to render. [C-42:L105-L107; C-42:L114-L119; C-42:L189-L200] versus [C-42:L186-L187].
- [AC] (C-002/C17) C-4 “Next” misidentifies C-5. | Evidence: It calls C-5 a ReviewWindow diff-wiring plan, but the processed C-5 is Cloud Mode Hardening. [C-4:L295-L298] versus [C-5:L1-L5].
- [CROSS] (C-002/I06) Audit correlation. C-12's started-before-verdict contract implies rows can be correlated by mode/tier/task/agent and sequence, although no explicit correlation ID is introduced.
- [CROSS] (C-003/I-X-01) The dashboard evidence loop in this chunk is producer/event or DB row → viewer/Tauri query → store/window/drawer: C-3 supplies knowledge/reasoning reads, C-7 supplies review/recovery interactions, C-10 supplies stagnation display, and C-29 supplies intent editing. Delivery is …
- [CROSS] (C-004/X-011) — MCP security chain: C-47 is sequenced after C-35 tool-attention and C-42 zero-trust hardening and is reached on every enabled MCP invocation. Its unaddressed call-shape/session privacy surface therefore sits inside, but is not solved by, the existing bridge security gate. [F…
- [CROSS] (C-006/C39-X002) Context describes the default CLI path as direct in-process plugin dispatch and unchanged, while C-13 bans InProcessToolDispatcher in production. The types are not necessarily identical, but implementation must explicitly reconcile this intent collision. [C-39 L30-L44; C-13 L6…
- [CROSS] (C-006/C40-X002) E2E requires closing a PR, but neither adapter's operation subset nor CLI flags define close_pr. [C-40 L78-L85, L114-L141, L226-L228]
- [CROSS] (SUMMARY/S-E0884) C-6 and C-7 provide the aligned visual system, streaming controls, window recovery, review controls, responsive layouts, and UI tests. [L79-L100]
- [CROSS] (SUMMARY/S-I0886) Together these form a producer → append-only event row → SsbViewer/Tauri query → drawer/window/control loop. The exact shared event-delivery mechanics must be verified from the originals because the summary sometimes mixes polling and subscriptions.
- [AUDIT] (SUMMARY/S-I1000) The dominant completed-design composition is: deterministic/isolated execution emits typed SSB evidence; live dashboard and CLI/MCP/PR surfaces consume it; knowledge and graph systems size/contextualize work; compiled/constrained/skeleton-first generation produces bounded code…

---

### Audit (SECTION-C)

- Units covered: 60 (C-0 through C-59 inclusive)
- Entry count: 60
- Behavior bullets (all unit sections, including multi-unit Superseded fanout): 3205
- Surfaces count: 366
- Superseded notes count: inline bullets=147; unique atom IDs=62 (aggregate superseded section declared 62; multi-unit fanout attaches one note to every unit it names)
- Source aggregate atoms (declared): 3339 (parser saw 3767 bullet rows including surface re-lists and section re-listings)
- System heading: 1 (`## System: C — …`)
- Output path: `docs/thefinalglance/feature-discovery/aggregates/compile/SECTION-C.md`

### Repair addendum (C)

- (C-006/C33-X001) Context's list of B-21 “plan claims” omits novel_step_detected, while the requested divergence note says novel_step_detected matches the plan. [C-33 L31-L36, L208-L210]

Note: repairs an audited omission — this atom exists in C-AGGREGATE.md but was missing from SECTION-C.md (siblings C33-X002 and C33-X003 were present).
