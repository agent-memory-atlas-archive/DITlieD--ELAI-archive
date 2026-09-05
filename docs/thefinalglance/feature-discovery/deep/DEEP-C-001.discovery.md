# DEEP-C-001 — original-plan exhaustive feature discovery

## Coverage, verification, and notation

This ledger is exhaustive for the ten original-plan sources assigned by manifest chunk DEEP-C-001. The C summary discovery was used only as an orientation and delta queue. No unassigned original-plan chunk was read.

Logical line counts are newline counts from wc -l. Every file matched the manifest before extraction.

| Exact processed source | Lines | Bytes | Manifest SHA-256 | Verification |
|---|---:|---:|---|---|
| docs/thefinalglance/C-plan/C-0_INDEX.md | 426 | 37226 | 44972d7f8ff76bba317e838b160783113e53994bd83a797a0a8a4263902d3c66 | exact |
| docs/thefinalglance/C-plan/C-2_ssb_emission_orchestration.md | 294 | 15979 | 2993e4663af2dd1235efd2e8854a864ddc095de9e3c09c3d4c21bbf38b14b7a7 | exact |
| docs/thefinalglance/C-plan/C-6_dashboard_design_tokens.md | 267 | 12969 | f062f78a0ec61099175f398eca1bea03db15e2cd7b7c613c4e00b4482cdf39b4 | exact |
| docs/thefinalglance/C-plan/C-18_test_infra_backfill_contracts.md | 248 | 14945 | 6121c760754f1e740b19f27b2657f41d5641ba1829562cbd59d0e3c6199aa835 | exact |
| docs/thefinalglance/C-plan/C-22_feasibility_probe_completeness.md | 313 | 18242 | 395835b1e4e130ee1930ed3691d2a054ca8b98b070beeb4b0e421c088fd26127 | exact |
| docs/thefinalglance/C-plan/C-30_surgical_oracle_mode.md | 321 | 17765 | d903d80738215782a580bc531e5bbc762f4d126f4889378a9646a68ad3b0c490 | exact |
| docs/thefinalglance/C-plan/C-41_skills_driven_workflow_formalization.md | 213 | 10107 | 7939ad3cdbc8b53fab7c3c8f60dcbc6dd663a4371233aff3278b6edb5bd6a782 | exact |
| docs/thefinalglance/C-plan/C-51_aider_polyglot_harness_adoption.md | 149 | 6658 | 76e6d4d6eede8079280606a287cc7e4c59f856b441aabbe766732364a70194eb | exact |
| docs/thefinalglance/C-plan/C-56_codebase_memory_dkg_31_repo_eval.md | 145 | 6380 | 61c5c8fdc74f007e85835ea4cde05e3f5fde1b919af00cd2dc7792efa19a0134 | exact |
| docs/thefinalglance/C-plan/C-58_agentforge_rrk_reranker_head_to_head.md | 178 | 7468 | 46c9cbab8161f7011797580a0a9efdcc1cf8803d95d87887b74d363a7769837e | exact |
| **Total** | **2554** | **147739** | — | exact |

Notation:

- [E] is explicit source text. [I] is a conservative composition and is never promoted to an implementation promise.
- [status] preserves the source's own PLANNED/DEFERRED/shipped prerequisite language; unchecked acceptance is not proof that repository work landed.
- [ambiguity] and [conflict] retain under-specification or disagreement instead of silently choosing a side.
- [Δ+] is material detail absent from 03-C-SUMMARY_INDEX.discovery.md; [Δ~] corrects or qualifies it; [Δ=] confirms it with deeper detail.
- Anchors use C0:Lx-y, C2:Lx-y, C6:Lx-y, C18:Lx-y, C22:Lx-y, C30:Lx-y, C41:Lx-y, C51:Lx-y, C56:Lx-y, and C58:Lx-y.
- Paper and competitor measurements below are plan-asserted grounding, not independently re-verified by this extraction.

## Per-source exhaustive ledger

### C-0_INDEX.md

#### Index contract, waves, and status

- **C0-001** [E][status][Δ~] The index was generated 2026-05-17 from 103 verification reports clustered into 25 themes; its premise says cloud mode is already proven through OAuth plus Sonnet 4.6, while each C-N is still a cohesive session that must wire a dashboard surface and prove itself with a real cloud-Sonnet acceptance run. (C0:L1-4)
- **C0-002** [E][status] Sessions are roughly dependency-ordered and may be parallel within a wave. The legend defines PLANNED, IN PROGRESS, DONE, DEFERRED, and NOT YET AUTHORED; an index row marked PLANNED therefore describes intended work, not shipped behavior. (C0:L6-15)

#### Waves 1–6 session inventory

- **C0-003** [E][status] C-1 is PLANNED, has no prerequisite, closes B-4/B-5/B-8, adds GrcStepEmitted, GroundingSweepResult, CsqEvaluated, AuditorRubberStampRejected, HydeQuery, VectorCacheHitRate, and QueryTrace, and accepts when a cloud task emits at least five of those seven with non-null payloads. (C0:L19-24)
- **C0-004** [E][status][Δ=] C-6 is a prerequisite-free dashboard-only PLANNED token/font/sidebar pass: align background, teal, and text tokens, fix fonts, shrink Sidebar 64→48px, and verify computed CSS after a cloud task. (C0:L26-29)
- **C0-005** [E][status] C-14 is PLANNED with no direct prerequisite: delete compact.rs (624 LOC), oauth.rs (591), and prompt.rs (744), remove banned usage.rs symbols, fix the Windows-shell determinism script, then require workspace check and entropy quarantine to pass. (C0:L31-34)
- **C0-006** [E][status][Δ~] C-2 is PLANNED after C-1 and closes B-6/B-7/B-13/B-38/B-42/phase-4G by adding orchestration, Cosmos, ambiguity, stagnation, lateral-thinking, and bubble-up SSB coverage; its index smoke requires stagnation, Cosmos, ambiguity, and bubble-up rows. (C0:L36-41)
- **C0-007** [E][status] C-3 is PLANNED after C-1 and adds get_query_trace_recent, get_pagerank_top, get_freshness_state, get_canary_drift, get_csq_log, get_feasibility_log, and get_agent_grc_config; KnowledgeDrawer and ReasoningDrawer must show non-empty live data. (C0:L43-46)
- **C0-008** [E][status] C-4 is PLANNED after C-1 and wires live agents, cloud spend, phase/baby-step/pipeline/progress/recent-action, runtime-list, and set_role_runtime commands; MonitorWindow must show at least one non-IDLE agent and non-zero spend. (C0:L48-51)
- **C0-009** [E][status] C-7 is PLANNED after C-6 and C-2: sweep seven Svelte files from 11px to 10px, add Stop/Ctrl+Enter/Ctrl+Shift+A/R, responsive breakpoints, ExploreWindow, recovery history, and per-hunk review controls; its smoke requires live streaming controls and BubbleUpStrip events. (C0:L53-56)
- **C0-010** [E][status] C-8 is PLANNED after C-1/C-2 and activates AmbiguityScorer, CosmosWorker::tick, TOOL_INDEX_ARTIFACTS, a real CrossEncoderReranker SsbNotifier, ONNX dot product in AragOrchestrator, CosmosStage, and drawer unwired flags; query-stage latencies must be non-zero and Cosmos status Fresh. (C0:L58-61)
- **C0-011** [E][status] C-9 is PLANNED after C-1: define AutoAdvanceEvaluator and EscalationPolicy, production-wire TreeAutoAdvancePolicy, add per-role backend configuration, populate phase_4G metrics, and prove an autonomous FSM auto-advance without human input. (C0:L63-66)
- **C0-012** [E][status] C-11 is PLANNED after C-9: complete Tier-C isolation with setrlimit/prlimit64, hidepid=2, PR_CAPBSET_DROP, IpcHookDispatcher, real NSpid/AppContainer tests, and five adversarial tests; a Tier-C baby step must emit lifecycle rows without SANDBOX_VIOLATION. (C0:L68-71)
- **C0-013** [E][status] C-12 is PLANNED after C-1/C-2: record per-level verdicts at five navigator completion points, confirm six auditor REVIEW sections, expose live verdicts, and prove five distinct per-level audit rows. (C0:L73-76)
- **C0-014** [E][status] C-15 is PLANNED after C-14: migrate about 68 SystemTime::now sites to ClockProvider, add instant_now(), replace production HashMap with FxHashMap in ten files, re-enable entropy CI, and require byte-identical seeded output. (C0:L78-81)
- **C0-015** [E][status] C-10 is PLANNED after C-9/C-2: implement StagnationPattern/Detection, ExecutionHistory, ThinkingPersona/PersonaStrategy, GoalDriftCanary, GenerationWatchdog, AdaptiveFutilityTracker, five persona files, and a StagnationSection; three identical errors must trigger stagnation and persona-switch evidence. (C0:L83-88)
- **C0-016** [E][status] C-13 is PLANNED after C-11: replace production InProcessToolDispatcher with IpcToolRegistry, install IpcHookDispatcher plus RuffPostToolUseRunner, implement AdaptiveTool for ten DAIRA markers, and squash WIP commits on AUDITING→MEMORIZING. (C0:L90-93)
- **C0-017** [E][status] C-16 is PLANNED after C-2/C-8: place CosmosStage between ArtifactStage and ARagStage, schedule hourly CosmosWorker ticks, add trigger_cosmos_refresh, replace tracing with SSB rows, and add five standalone tests; a real OpenAPI research run must show refresh-started and Fresh. (C0:L95-98)
- **C0-018** [E][status] C-17 is PLANNED after C-14/C-15: add Kotlin parsing, auto-resume and signal handlers, LockScope::Watch, Heading/Paragraph chunking, real ONNX embeddings, Oracle pre-seeding, and scale metrics; a Rust+Kotlin+Svelte research run must index .kt and yield non-zero ONNX scores. (C0:L100-103)
- **C0-019** [E][status][Δ=] C-18 is PLANNED after C-1/C-2/C-13 and creates omitted contract/integration tests for B-2/B-29/B-32/B-33/B-35/B-36 and phases 3B/3D/3H with zero production logic changes; elai_cc_contracts must be green. (C0:L105-108)
- **C0-020** [E][status] C-19 is PLANNED after C-11, tests only: add all specified adversarial cases and Criterion benchmarks, then prove a cloud path-traversal attempt is blocked without a SANDBOX_VIOLATION row. (C0:L110-113)
- **C0-021** [E][status] C-20 is PLANNED after C-9/C-13/C-17: replace placeholder metrics, wire ELAI_DEBUG_BATCH_DELAY_SECS, add elai measure, and require non-zero phase_4G runtime categories after a Sonnet 4.6 measurement run. (C0:L115-118)
- **C0-022** [E][status] C-21 is PLANNED after C-15 and links anti-staleness CI, unwrap cleanup, LOCKED_KEYS pre-commit enforcement, dependency audits, steering refresh, closed ParameterKey, ConstraintsBlock, SWE-bench fixtures, and MistakeBook BLOCK ingestion; the smoke proves refresh and commit blocking. (C0:L120-123)
- **C0-023** [E][status][Δ=] C-22 is PLANNED after C-19/C-9: rename SystemProbeReport, add GPU UUID and ResourceDelta, production-wire DefaultFeasibilityProber, add adversarial coverage and FeasibilityPanel, reconcile event names, and populate phase_4F metrics. (C0:L125-128)
- **C0-024** [E][status] C-23 is PLANNED after C-3/C-4: add /session merge|close, mount BranchTree with live fork/merge/close, repair /continue handoff loading and parent ordering, and prove a context handoff can be continued. (C0:L130-135)
- **C0-025** [E][status] C-24 is PLANNED after C-17: expose symbol_impact/flow/context/list, incrementally update changed-file graphs, add /graph mermaid, Cytoscape decomposition styling, ActivityWindow blast-radius links, and graph_export_dispatch; a modifying agent must self-select symbol_impact. (C0:L137-140)
- **C0-026** [E][status] C-25 is PLANNED after C-9: add CompiledPipeline/TemplateCrystallizer/CompiledExecutor, BackgroundTaskManager soft-timeout detach plus two events, CumulativeTokenTracker, and /pipeline list|reset; three repeated successes crystallize and the fourth run lowers token use. (C0:L142-145)
- **C0-027** [E][status] C-26 is PLANNED after C-10/C-25: implement ConstrainedDecodingConfig/DecodingMode, temperature and same-error RetryStrategy, navigator futility/persona switching, Sidecar/CloudGrounding wiring, a fixture-path fix, and three EBNF grammars; schema-valid JSON and three-error switching are gates. (C0:L147-150)
- **C0-028** [E][status] C-27 is PLANNED after C-15/C-17: add hardware-locked DeterminismConfig, SeededRng, blake3 prefix cache, DETERMINISM_DRIFT, WiringContract and three auditors/runners, WIRING_EVIDENCE/VIOLATION, elai wiring audit, and CI; known stubs must be caught. (C0:L152-155)
- **C0-029** [E][status] C-28 is PLANNED after C-27: integrate cargo-mutants and mutmut, MutationResult, MUTATION_SURVIVED, EquivalentMutantCache, an AUDITING→MEMORIZING mutation gate, and a MonitorWindow score gauge. (C0:L157-162)
- **C0-030** [E][status] C-29 is PLANNED after C-27/C-28: add IntentPair, MR_identity/MR_inverse/MR_permutation_invariance, INTENT_CONFIRMED/MR_VIOLATED at schema 47, a three-pair PLANNING→TESTING_FIRST gate, MetamorphicTestRunner, and a PlanWindow editor. (C0:L164-167)
- **C0-031** [E][status][Δ=] C-30 is PLANNED after C-17/C-24: implement surgical planning/types/DKG traversal, tiered Oracle answer caching, four DKG tools, project signals, LOC gates, and research.vram_tiers; a surgical cloud task must produce an AffectedSet spanning at most three files. (C0:L169-174)
- **C0-032** [E][status] C-31 is PLANNED after C-13/C-9: define FilePlan/FileSpec/FileModSpec/FunctionSpec, add FILE_PLAN as the ninth BabyStepSpec block, track lines with TOCTOU protection, record T1/T2 BLOCK mistakes, and guard per-step function counts. (C0:L176-179)
- **C0-033** [E][status] C-32 is PLANNED after C-31/C-26: skeleton-first BLANK_N generation, one in-memory write, MiniCritic, AcceptAndComplete at ≥0.5, structural retries DECOMPOSE→TEMPLATE→FEW_SHOT→BEST_OF_N, deterministic bridge files, and a completeness gauge; its smoke targets ≥0.9 completeness. (C0:L181-184)
- **C0-034** [E][status] C-33 is PLANNED after C-30/C-22: author phase 3L/3J/4D docs, update B status/schema 46, reconcile REFINE_CAP and B-21 event names, add LAZY_CODE/EMPTY_BODY/FILE_SIZE_EXCEEDED, and record KEEP/STRIP for 28+ slash commands; link and wiring checks must pass. (C0:L186-189)

#### Waves 7–9 and deferred inventory

- **C0-035** [E][status] C-35 is PLANNED after the shipped MCP bridge and claims a measured 95% MCP-tax reduction while labeling downstream quality projections; it gates tool visibility/lazy schemas and requires 5–10K prompt-token savings with no more than 2pp SWE-bench loss. (C0:L191-199)
- **C0-036** [E][status] C-36 is PLANNED after C-29, feature-gated, extends MetamorphicRelation with MUCOCO operators, and targets paper-like 15% inconsistency detection on at least one operator. (C0:L201-204)
- **C0-037** [E][status] C-37 is PLANNED after C-26 with a RunPod pilot: re-evaluate PER_ROLE_DECODING_DEFAULTS against structure-snowballing evidence and auto-generate pilot.md while preserving anti-rubber-stamp findings. (C0:L206-209)
- **C0-038** [E][status] C-38 is PLANNED after shipped B-1: prove PageRank on the production path and correct the summary's Aider repomap description from file-level to symbol-level with a diff and fresh-fetch date. (C0:L211-214)
- **C0-039** [E][status] C-39 is PLANNED after C-35: extend the CLI/tool surface through MCP with an OpenHands-parity CI job and automatic co-process launch; --via-mcp parity must pass. (C0:L216-219)
- **C0-040** [E][status] C-40 is PLANNED after C-13/C-23: add a Devin-style GitHub draft-PR adapter, opt-in per project, with automated review-comment response default OFF and an honesty section in the PR body. (C0:L221-224)
- **C0-041** [E][status][Δ=] C-41 is PLANNED after shipped Phase 4A/prompt-engineer; it formalizes skill scaffolds based on a claimed 44%→83% intent gain and requires ≥73% synthetic SciQuest accuracy. (C0:L226-229)
- **C0-042** [E][status] C-42 is PLANNED strictly after C-35/C-39 and before C-40/C-44: deterministic zero-trust MCP fuzzing must find no CRITICAL result over at least 200 tuples. (C0:L231-234)
- **C0-043** [E][status] C-43 is PLANNED as a policy/guardrail with no behavior change: freeze Phase 4H at three layers because cited anti-evidence says extra mechanisms degrade performance; an ablation must expose negative delta. (C0:L236-239)
- **C0-044** [E][status] C-44 is PLANNED after B-37/C-39/C-42 and gated on Composer 2 API access; it cites 73.7 SWE-bench Multilingual and $0.50/M input, adds a third backend tier without RunPod, and requires cost_accrued on every call. (C0:L241-244)
- **C0-045** [E][status] C-45 is PLANNED, but its per-role CANTANTE pilot is deferred to a RunPod window; it measures contribution for C-41 routing and role consolidation and must produce confidence intervals. (C0:L246-251)
- **C0-046** [E][status] C-46 is PLANNED after the eight roles, with C-45 recommended: difficulty-tier routing targets 40–60% savings on easy tasks and accepts ≥30% cost reduction within ±2pp solve rate. (C0:L253-256)
- **C0-047** [E][status] C-47 is PLANNED after C-35/C-42: a per-project sled case base injects near-match tool hints and must reduce repeated tool errors by at least 10% over a 50-session replay. (C0:L258-261)
- **C0-048** [E][status] C-48 is PLANNED after memory-writer/C-42: classify memory T0 user-direct through T3 model-emitted, limiting safety-critical decisions to T0/T1; a T2 prompt-injection fixture must have no influence. (C0:L263-266)
- **C0-049** [E][status] C-49 is PLANNED after prompt-engineer, with C-41 recommended: accumulate a per-project successful-trace experience corpus and prove non-empty distilled fragments after ten sessions. (C0:L268-271)
- **C0-050** [E][status] C-50 is PLANNED after shipped Phase 4F: Contract-Coding cites 47% functional/near-perfect structural success and conservatively targets at least 40% functional and 90% structural on a Greenfield-5 fixture. (C0:L273-276)
- **C0-051** [E][status][Δ=] C-51 is PLANNED after Phase 4F: vendor the pinned MIT Aider harness and 225 Exercism tasks as infrastructure only, dry-run five mock tasks, and refuse a full sweep without GPU; execution is deferred to C-55. (C0:L278-281)
- **C0-052** [E][status] C-52 is a PLANNED research spike citing Gemma 4-31B at 90.91% on NL2VC-60; it targets FSM, MemoryView, and the mutation aggregator with C-48 recommended, and accepts either three verified modules or an infeasibility note with at least one concrete blocker. (C0:L283-286)
- **C0-053** [E][status] C-53 is PLANNED but combines a passive NF4 upstream watch with a RunPod-dependent in-context-versus-orchestration counter-test; closure requires an upstream-watch artifact and pilot artifact. (C0:L288-291)
- **C0-054** [E][status] C-54 is a PLANNED RunPod flagship after C-21/C-35/C-39/C-42 and optionally C-44: run local-7B, dual-local-7B, and hybrid arms on all 500 SWE-bench Verified tasks, producing per-config scorecards, methodology, and honesty. (C0:L293-303)
- **C0-055** [E][status] C-55 is a PLANNED RunPod run after C-51: benchmark local-32B and hybrid over all 225 Aider problems and open a public draft PR against Aider-AI/aider through C-40. (C0:L305-308)
- **C0-056** [E][status][Δ~] C-56 is a PLANNED RunPod evaluation after B-29/B-30/C-38 against published 83% quality, 10× token, and 2.1× tool-call figures; it requires per-repo and aggregate metrics plus a paper-baseline cross-tabulation. (C0:L310-313)
- **C0-057** [E][status] C-57 is a PLANNED cost-amortized OpenHands Index/SWE-Cycle/SWE Atlas run after C-54, producing per-axis scorecards and a solve-rate curve in one RunPod session. (C0:L315-318)
- **C0-058** [E][status][Δ=] C-58 is a PLANNED RunPod AgentForge/RRK evaluation with C-45 recommended; it compares against a claimed 40.0% baseline, reports RRK p99 and quality delta, and makes a config-gated adoption decision. (C0:L320-323)
- **C0-059** [E][status] C-59 is the PLANNED capstone after C-54..C-58: deterministically regenerate a master report from all scorecards plus external_baselines.toml, with honesty and improvement targets; TTS is optional. (C0:L325-328)
- **C0-060** [E][status] C-34 alone is DEFERRED pending Empire access and configured codebase_path; the feature-gated workspace would add FactoryLineTransport, ElaiNeuron MCP, BrainManifest, EmpireEventBridge, and repair for CircularDep/HardImport/StdlibShadow, then prove connected/capability events. (C0:L330-335)

#### Index-wide counts, provenance, closure maps, and inconsistencies

- **C0-061** [E][conflict][Δ+] The statistics claim 59 slots: 52 CPU/cloud, six RunPod, one external; 59 C-N files authored; and all C-1..C-33 executable. Yet the Sessions section contains no C-5 row, so only 58 session bullets are present. C-5's features/status cannot be recovered from this source. (C0:L17-335, L339-349)
- **C0-062** [E][status] Effort is estimated at 6–8 hours per implementation session and 8–24 wall-clock hours per benchmark; totals are 310–415 CPU/cloud hours plus 50–150 hours and RunPod cost. Wave 9 jobs are independently runnable when hardware is online, while C-59 runs last. (C0:L293-297, L341-349)
- **C0-063** [E][ambiguity] Wave 7–9 provenance claims an oracle artifact with 11 papers/22 competitor capabilities and a Discord dump with 15 more papers, but then says only 19 verified papers plus seven anchors—Devin, Aider, Cursor, Claude Code, OpenHands, mini-SWE-agent, and GPT-5.4/Composer 2. Duplicates or exclusion rules are not identified. (C0:L351-362)
- **C0-064** [E][conflict][Δ+] Verification is routed through verification/INDEX.md and verification/_themes.md and said to contain 103 reports, but the listed breakdown DONE 5 + MOSTLY_DONE 77 + PARTIAL 20 + NOT_DONE 3 totals 105; the index does not explain the arithmetic. (C0:L364-368)
- **C0-065** [E] B-1..B-8 closure map: B-1→C-18/C-8; B-2→C-18; B-3→C-3/C-8/C-23; B-4→C-1/C-3/C-8; B-5→C-1/C-3/C-12; B-6→C-2/C-4; B-7→C-2/C-4/C-8; B-8→C-1/C-3. (C0:L370-379)
- **C0-066** [E] B-9..B-16 closure map: B-9→C-23; B-10→C-4; B-11→C-6/C-7; B-12→C-6/C-7; B-13→C-2/C-7; B-14→C-7; B-15→C-14/C-15; B-16→C-21. (C0:L380-387)
- **C0-067** [E] B-17..B-24 closure map: B-17→C-2/C-8; B-18→C-11; B-19/B-20→C-13; B-21→C-22; B-22→C-9/C-10; B-23→C-21; B-24→C-7/C-8. (C0:L388-395)
- **C0-068** [E] B-25..B-33 closure map: B-25→deferred C-34; B-26→C-4/C-7; B-27→the otherwise absent C-5; B-28→C-4; B-29→C-24; B-30/B-31→C-17; B-32→C-18; B-33→C-24. (C0:L396-402)
- **C0-069** [E] B-34..B-42/B-0 closure map: B-34→C-17; B-35→C-8/C-18; B-36/B-37→C-4 and B-36 also C-18; B-38→C-2/C-8; B-39→C-33; B-40/B-41→C-12 plus C-33 for B-40; B-42→C-2/C-16; B-0_INDEX→C-33. (C0:L403-412)
- **C0-070** [E] Guide closure map: compiled pipeline→C-25; token efficiency→C-26; pipeline modes→C-30; code completeness→C-31/C-32; testing trust→C-27/C-28/C-29; anti-staleness→C-21; dashboard design→C-6/C-7; phase-map update→C-33. (C0:L413-420)
- **C0-071** [E][status] Phase closure map: phase-4G→C-9/C-10/C-20; 4A→C-11/C-19; 4F→C-22/C-19/C-20; 4H→C-21; 4J remains deferred to C-34; 4K→C-4. These are intended closure assignments, not evidence that the PLANNED sessions shipped. (C0:L421-426)

### C-2_ssb_emission_orchestration.md

- **C2-001** [E][status] Objective: close five producer gaps—tree navigator, Cosmos worker, ambiguity scorer, autonomous stagnation detector, and agent lifecycle—so BubbleUpStrip and OrchestrationDrawer display live cloud events. C-1 is required; C-3/C-4/C-9 are parallel-safe; estimate 6–8h. (C2:L1-10)
- **C2-002** [E] B-7 lacks PhaseNodeReady, StepNodeReady, BabyStepReady, and DagRepairEscalated definitions/emits, while B-13's BubbleUpStrip already queries bubble_up_typed but documents an empty-until-wired state. (C2:L12-18)
- **C2-003** [E] B-6 lacks AgentStatusChanged and get_agents returns eight hardcoded IDLE roles. (C2:L20-21)
- **C2-004** [E] B-38 already defines AmbiguityScored, AmbiguityFloorFailed, and AmbiguityStreakAdvanced, but task_runner does not invoke the scorer, get_ambiguity_state returns mock:true, and the drawer has no ambiguity section. (C2:L23-25)
- **C2-005** [E] B-42 already defines CosmosRefreshStarted/Updated/Decayed/GroundingUsed, but CosmosWorker defers SSB integration and uses tracing as a temporary channel. (C2:L27-28)
- **C2-006** [E] Phase 4G entirely lacks StagnationSpinningDetected, StagnationOscillationDetected, LateralThinkingActivated, GoalDriftWarning, and WatchdogTimeout. (C2:L30-31)
- **C2-007** [E][Δ+] Add the four tree variants, four correspondingly named payload structs, and snake_case serialization arms in ssb.rs. (C2:L52-58)
- **C2-008** [E][Δ+] Add AgentStatusChangedPayload fields agent_id, role, old_lifecycle, new_lifecycle, and timestamp_ms:u64. (C2:L60-64)
- **C2-009** [E][ambiguity][Δ+] Add the five anti-spiral variants with minimal payloads patterned as pattern:String, iteration:u32, detail:String; the source does not say whether this means one common or five distinct structs. (C2:L66-70)
- **C2-010** [E][Δ+] Emit tree readiness at node entry through the existing ssb_tx, with node_id, label, depth, and parent_id; this makes three different navigator levels observable. (C2:L72-78)
- **C2-011** [E][Δ+] At T_DAG=10 exhaustion, emit DagRepairEscalated with attempt_count and last_error. (C2:L80-84)
- **C2-012** [E][Δ+] Replace Cosmos tracing in tick/refresh_one/record_grounding_used with persisted SsbRow inserts: RefreshStarted per artifact on tick entry, ArtifactUpdated on write success, ArtifactDecayed on detection, and GroundingUsed on use; inject ssb_tx using the BSP pattern. (C2:L86-93)
- **C2-013** [E][Δ+] Invoke AmbiguityScorer::score before GroundingSweepGate, set TransitionContext.ambiguity_score, and emit ambiguity_scored/floor_failed/streak_advanced from the gate result using existing payloads. (C2:L95-101)
- **C2-014** [E][ambiguity] Mirror AgentLifecycle transitions at five BSP emit sites into main ssb_tx, or alternatively emit at task_runner spawn/complete/crash; the owning implementation site is deliberately left open. (C2:L103-109)
- **C2-015** [E] ssb_viewer::recent_agent_status_changes reads AgentStatusChanged and reduces it to latest lifecycle per agent_id. (C2:L111-114)
- **C2-016** [E] get_agents must replace the hardcoded eight-role IDLE list with recent_agent_status_changes so live state reaches the drawer; get_ambiguity_state must replace mock:true with the three ambiguity row types. (C2:L116-126)
- **C2-017** [E][Δ+] A collapsed AmbiguitySection calls get_ambiguity_state and renders overall_score, per-dimension bars, floor-failure flags, and streak progress using GatesPanel's layout. (C2:L128-133)
- **C2-018** [E] ssb_c2_tree_node_contract.rs verifies all four tree action strings and viewer round-trip. (C2:L135-138)
- **C2-019** [E] ssb_c2_cosmos_emit_contract.rs inserts mock RefreshStarted/ArtifactUpdated rows and verifies correct payload retrieval. (C2:L140-143)
- **C2-020** [E] ssb_c2_agent_status_contract.rs emits three roles and verifies latest lifecycle plus Tauri reflection. (C2:L145-148)
- **C2-021** [E] Reachability is cloud /task, BubbleUpStrip queries, live OrchestrationDrawer agent/ambiguity sections, and CosmosDrawer refresh; consumed anchors include C-1's variant helper, scorer/evidence fields, existing repair/bubble emit patterns, and threaded ssb_tx. (C2:L150-180)
- **C2-022** [E][security] Stub bans require no hardcoded IDLE, no mock:true, persisted Cosmos rather than tracing, and no unwrap in new emit paths; fallible emits use ? or logged .ok(). (C2:L181-185)
- **C2-023** [E][observability] The three-function Rust CLI trigger expects ≥1 phase row, ≥3 step rows, a non-null ambiguity score, optional Cosmos refresh if its tick falls in-window, and lifecycle per spawned role; dashboard evidence is a real bubble, non-IDLE role, and numeric ambiguity. (C2:L187-197)
- **C2-024** [E][done-gate] Static checks cover variant hits, ≥4 Cosmos payload emits, scorer invocation, zero hardcoded/mock hits, ≥3 navigator emits, repair escalation, contract tests, and clean orchestrator/knowledge/contracts compilation. (C2:L198-231)
- **C2-025** [E][conflict][Δ~] Acceptance calls all 12 checked action types “new,” although the plan explicitly says four Cosmos types already exist and only ten missing enum variants are introduced. The deterministic grep also names only two of the four existing Cosmos types. (C2:L23-31, L52-70, L200-205, L233-247)
- **C2-026** [E][done-gate] Cloud acceptance queries six named action strings and requires at least five distinct non-null types, plus a non-placeholder bubble, at least one non-IDLE role, and a numeric—not spinner/zero-state—ambiguity score. (C2:L248-260)
- **C2-027** [E][Δ+] The named test inventory is four tree tests, three Cosmos tests, and two agent tests (nine total), all using a real temporary in-process SQLite SSB patterned after ssb_4k3_4k4_4k5_contract.rs. (C2:L262-282)
- **C2-028** [E][deferred] This session does not integrate ambiguity interrogation rounds, Cosmos spawn_loop/pipeline stage/refresh unblocking, anti-spiral logic/personas, PlanWindow cascade arrows, V2 phase/baby-step commands, or research/package/stack event types; those belong to C-8/C-10/C-16/C-3/C-5. (C2:L284-294)
- **C2-029** [I] Persisted producer rows plus latest-state reduction create two semantics on one log: history for Bubble/Cosmos/ambiguity evidence and materialized latest lifecycle for get_agents. This composition follows explicit endpoints; retention and concurrent-write ordering are not specified. (C2:L86-126, L173-180)

### C-6_dashboard_design_tokens.md

- **C6-001** [E][status] C-6 is a one-session, backend-free dashboard pass with no prerequisites, parallel-safe with C-5/C-7; it covers tokens, fonts, 11px sites, sidebar dimensions/count, Stop, and Ctrl+Enter. (C6:L1-8)
- **C6-002** [E] Verified deltas are wrong background/font tokens, inconsistent teal overrides, absent/mis-valued text-ghost, seven 11px component files, 64px/seven-icon Sidebar versus 48px/five-icon spec, and missing Stop/Ctrl+Enter; the source calls these cosmetic/UX only. (C6:L10-31)
- **C6-003** [E][config] Exact background defaults are --bg-void #0a0e12, --bg-base #0f1318, --bg-surface #151a20, and --bg-hover #1a2028; component/root overrides must be searched and eliminated. (C6:L57-63)
- **C6-004** [E][config][Δ+] Exact teal defaults are --teal #1B7A6E, --teal-bright #24a08e, and --teal-soft rgba(27,122,110,0.25), again with a pre-write override audit. (C6:L65-69)
- **C6-005** [E][config][Δ+] Text tokens are --text-primary #d0d7de, --text-secondary #6e7681, and --text-ghost #383e47; add ghost if absent without deleting other tokens. (C6:L71-75)
- **C6-006** [E][config][Δ+] Set --font-sans to JetBrains Mono, monospace and --font-display to Segoe UI Variable/Segoe UI/sans-serif; display thinness belongs at usage sites, and body must consume --font-sans without a shadowing direct family. (C6:L77-83)
- **C6-007** [E][accessibility] In ArtifactsPanel, IndexingProgressPanel, MediaValidationPanel, AntiStalenessDrawer, CosmosDrawer, KnowledgeDrawer, and MonitorWindow, inspect each 11px occurrence and change only font-size/line-height to the 10px accessibility floor—not unrelated padding/radius. (C6:L85-92)
- **C6-008** [E] Sidebar becomes 48px only after verifying icons ≤20px and horizontal padding ≤8px per side still fit. (C6:L94-97)
- **C6-009** [E][deferred] Do not delete two surplus icons. Name both in a SPEC-DELTA comment, retain them for parity, and record the decision in C-6-decisions.md pending C-7. (C6:L99-105)
- **C6-010** [E][failure] Stop is conditionally visible only while streaming, calls the already-existing abort/cancel handler, creates no IPC, uses teal-soft/text-primary, matches Send height, and disappears when streaming ends. (C6:L107-113, L161-166)
- **C6-011** [E] On a non-empty, non-streaming textarea, Ctrl+Enter calls the same submit handler; plain Enter retains newline behavior, duplicate IPC is forbidden, and the textarea gets aria-label “Press Ctrl+Enter to send.” (C6:L115-120, L154-158)
- **C6-012** [E] Only stale B-11 deterministic-check paths are rewritten from V1 to dashboard-v2; no other B-11 section changes. (C6:L122-128)
- **C6-013** [E][evidence] C-6-decisions.md records token provenance, font rationale, icon retention, discovered Stop handler, and plain-Enter rationale. (C6:L130-137)
- **C6-014** [E] tokens.css is the sole source of truth, no component :root override is allowed, and CSS cascade plus CommandWindow keyboard/button events are the reachability surfaces. (C6:L138-166)
- **C6-015** [E][done-gate] Deterministic checks inspect all token groups, body font, zero 11px hits, 48px/no-64px Sidebar, Stop/Ctrl handler presence, zero Svelte errors, and the decision document. (C6:L168-208)
- **C6-016** [E][done-gate] Exact token values, documented icon delta, real Stop/submit behavior, corrected B-11 paths, zero new svelte-check errors, and all five rationale items are acceptance gates. (C6:L210-228)
- **C6-017** [E][observability] Manual cloud smoke starts and stops a hello-world stream, sends a fresh prompt with Ctrl+Enter, and inspects computed bg-void plus JetBrains-first body font. (C6:L230-240)
- **C6-018** [E][tests][Δ+] Component tests require data-testid=stop-button conditional rendering and exactly-once Ctrl+Enter submission for non-empty input. Reuse existing Vitest tooling or create CommandWindow.test.ts; no new dependency without human confirmation. (C6:L242-254)
- **C6-019** [E][constraint] No Rust, IPC protocol, or Tauri changes; no new Stop IPC; no files outside dashboard-v2 except the targeted B-11 fix and decision doc; no icon deletion or font installation/CDN work. (C6:L256-265)
- **C6-020** [E][recovery] If JetBrains Mono is not already bundled through index.html/font-face declarations, stop and escalate to the user rather than adding it. (C6:L264-267)
- **C6-021** [E][conflict][Δ+] The task scope names seven files, but the deterministic and acceptance grep demands zero 11px anywhere under dashboard-v2/src. Existing unrelated 11px values could fail closure despite being outside the enumerated edit set. (C6:L85-92, L184-186, L220)
- **C6-022** [E][ambiguity] commands.isStreaming is explicitly a provisional identifier (“or equivalent”); exact stream, abort, and submit symbols must be discovered before wiring. (C6:L107-120, L146-150)

### C-18_test_infra_backfill_contracts.md

- **C18-001** [E][status] C-18 is a one-session tests-only backfill after C-1/C-2/C-13, parallel with C-19/C-20. It claims zero production changes and targets omitted crate-boundary/serde/DAG checks. (C18:L1-22)
- **C18-002** [E] fsm_phase_transition_contract uses real temporary SSB, proves IDLE→PLANNING and payload fields, and codifies that Fsm::reset emits no second PhaseTransition. (C18:L38-45)
- **C18-003** [E] symbol_query_dispatch builds a 3-node/2-edge in-memory SymbolGraph, calls translate_tool_request for symbol_impact/flow/context/list, requires Ok plus SymbolQuery after each, and runs offline with ELAI_LIVE_SIDECAR absent. (C18:L47-54)
- **C18-004** [E] test_minified_file_emits_ssb_action indexes bundle.min.js and requires MinifiedFileChunked with matching relative_path and chunk_count>0. (C18:L56-62)
- **C18-005** [E][determinism] graph_export_dispatch directly invokes export_graph_mermaid without Tauri runtime and requires a Mermaid header, length>10, and byte-identical consecutive output. (C18:L64-70)
- **C18-006** [E] grounding_artifact_check covers Verified when results exceed threshold, MalformedGrounding for a missing manifest name, and Unverified for empty search, using in-memory ArtifactStore SQLite. (C18:L72-78)
- **C18-007** [E] designer_artifact_briefing requires AVAILABLE_ARTIFACTS plus two names for an existing project, and no such block for an empty/greenfield manifest. (C18:L80-87)
- **C18-008** [E][Δ+] Five dashboard contracts map D1=get_sidecar_health, D2=run_planning_task, D3=run_research, D6=reindex_now, D11=sidecar_search; each JSON round-trips its request and type-checks the command, with only a TODO for ELAI_LIVE_SIDECAR integration. (C18:L89-100)
- **C18-009** [E][failure][determinism] ssb::tests adds ActionTypeMismatch—not panic/Ok—when CheckpointPayload is read as FileWritePayload, plus five fixed-clock rows whose seq is 1..=5 and whose repeat-run payload bytes match. (C18:L102-109)
- **C18-010** [E][lifecycle] BSP tests round-trip LastValue<String>, Topic<u64>, and EphemeralValue<bool> through MessagePack, then run five supersteps and verify last-writer-wins, topic-clear, ephemeral behavior, counter=5, and no small-channel force-clear. (C18:L111-119)
- **C18-011** [E][determinism] SessionDag runs 100 identical seeded 10-message trim cycles and requires identical snap_id/summary_hash; the handoff boundary must be off at 84.9% and on with tag=handoff at exactly 85%. (C18:L121-130)
- **C18-012** [E] Produced tests consume real Fsm/viewer, production tool translation, IndexingService, Mermaid export, ArtifactCheck/store, project-context builder, Tauri signatures, SSB errors/clock, BSP, and SessionDag APIs. (C18:L132-155)
- **C18-013** [E][security] Fixtures must use real in-memory SQLite rather than hardcoded return data, and test setup must not use unwrap in a way that swallows construction errors. (C18:L156-165)
- **C18-014** [E][observability] Cargo tests across contracts/orchestrator/knowledge/reasoning must pass; a cloud symbol modification must emit SymbolQuery and leave the dispatch contract green on the next test run. (C18:L156-170)
- **C18-015** [E][done-gate] Checks demand at least ten newer *_contract.rs files, all target suites, named determinism/round-trip/mismatch tests, zero unwrap in the FSM test, and a clean workspace check. (C18:L172-206)
- **C18-016** [E][done-gate] Acceptance repeats every behavioral assertion, requires no pre-existing regression/full-workspace audit, and uses “10+ contract/integration test files” as the closure count. (C18:L208-226)
- **C18-017** [E][Δ+] Exactly ten files are ADDed: FSM, symbol dispatch, graph export, grounding, designer, and five dashboard contracts. Four existing Rust files are modified to append seven test functions, and one decision doc is created. (C18:L228-244)
- **C18-018** [E][conflict][Δ~] The objective says “Only new test files under .elai_cc/crates,” but the plan modifies chunking_minified.rs and three production source files' test modules and creates workflow/reasoning/C-18-decisions.md outside that tree. “Zero production logic changes” is the narrower consistent rule. (C18:L1-4, L56-62, L102-130, L228-244)
- **C18-019** [E][conflict][Δ+] Task 7 says “D1 through D5” but the exact files/commands are D1, D2, D3, D6, and D11. The latter identifiers are repeated in wiring, acceptance, and files lists and are therefore the explicit set. (C18:L89-100, L141-152, L210-215, L236-240)
- **C18-020** [E][ambiguity] The “>=10 newer contract files” command counts by timestamp relative to channel_registry_contract.rs, so it can include unrelated files or miss modified test modules; it is not an exact proof of this plan's ownership. (C18:L174-176)
- **C18-021** [I][tests][Δ+] The described inventory yields a minimum of 20 test functions if FSM, four-symbol dispatch, minified, and graph each use one function: 1+1+1+1+3 grounding+2 designer+5 dashboard+2 SSB+2 BSP+2 SessionDag. Four symbol tools remain separate cases inside the dispatch test. The source does not declare this aggregate. (C18:L38-130)
- **C18-022** [E][dependency] The dashboard files are type-shape contracts only; their live-sidecar path is explicitly deferred to future CI, while the symbol graph and artifact paths must remain network-free. (C18:L47-100)
- **C18-023** [E][status] The next route is C-19/C-20 or B-43; no feature status is upgraded by authoring this plan. (C18:L246-248)
- **C18-024** [I] Together the suite spans serialization, dispatcher reachability, event persistence, state boundaries, deterministic export/IDs, and channel lifecycle. It is regression evidence, not a new runtime capability. (C18:L17-22, L38-170)

### C-22_feasibility_probe_completeness.md

- **C22-001** [E][status] C-22 is a 2–3 session PLANNED-style gap closure after C-19/C-9, parallel-safe by claim with C-21/C-23; it activates the dead feasibility path, data/resource schema, events, dashboard, termination contract, tests, and metrics. (C22:L1-12)
- **C22-002** [E] Nine verified gaps are named: SystemProbe type mismatch, missing GPU UUID, missing StubResult fields, absent ResourceDelta, None production construction, event-name drift, forbidden kill_on_drop, absent panel/unwired flag, and six absent metrics. (C22:L14-50)
- **C22-003** [E] Rename SystemProbeReport→SystemProbeResult mechanically everywhere under crates, with zero old-name hits and no logic change. (C22:L69-74)
- **C22-004** [E][failure] GpuInfo.uuid is Option<String>, parsed from nvidia-smi -q -d UUID via GPU-[0-9a-f-]{36}; parse failure stores None and adds no dependency. (C22:L76-80)
- **C22-005** [E][schema] ResourceDelta has four u64 counters—VRAM/RAM before/after—and derives Debug, Clone, Default, Serialize, and Deserialize. (C22:L82-92)
- **C22-006** [E][schema][compatibility][Δ+] StubResult adds alternative_index:u8 (zero means first/error), optional blake3 stdout hash, optional first-256-character stderr summary, and optional ResourceDelta; every field has serde(default) and no existing field is removed. (C22:L93-100)
- **C22-007** [E][failure] Probe resources before spawn and after return with SystemProbeResult; on either re-query failure set resource_delta=None rather than failing the stub. (C22:L101-106)
- **C22-008** [E][lifecycle] Remove kill_on_drop(true); explicit child.kill().await at the two existing paths becomes the sole termination mechanism, avoiding double-kill risk. (C22:L108-111)
- **C22-009** [E][compatibility] Preserve canonical shipped event names system_probe, feasibility_report, novel_step_detected, and stub_run, and update phase-4F/B-21 contracts rather than breaking consumers. (C22:L113-120)
- **C22-010** [E][events][Δ+] Add alternative_explored once per accepted alternative with step_id+alternative_index, and feasibility_escalated on Escalate with reason+escalation variant; register and emit both. (C22:L116-120)
- **C22-011** [E] Replace None with Some(Box::new(DefaultFeasibilityProber::new())) in cli/main.rs and runtime_setup.rs, using in-scope AppConfig if required and exporting the constructor if necessary. (C22:L122-128)
- **C22-012** [E][surface][Δ+] FeasibilityPanel takes FeasibilityReport|null and shows probe_count, novel_step_count, summed alternatives, optional skip notice, and per-concept decision/stub hashes/stderr/resource VRAM accordion; null shows “Feasibility probe not run.” (C22:L130-140)
- **C22-013** [E] The panel consumes get_feasibility_report, replaces the ReasoningDrawer placeholder, and clears d.unwired.feasibility=false. (C22:L140-143)
- **C22-014** [E][observability] After a real run, phase_4F.toml records measured system-probe p50, stub p50/p99, average alternatives, skip fraction, and escalation fraction; placeholders are forbidden. (C22:L145-153)
- **C22-015** [E][dependency] T9/T13–T16 are owned conditionally: add them if C-19 did not, otherwise remove C-19's stub_result_v2 gate; the decision doc records execution order. (C22:L155-161)
- **C22-016** [E] Reachability is cloud /research→DefaultFeasibilityProber, ReasoningDrawer→Tauri report, and live alternative/escalation SSB rows. AppConfig and explicit kill are consumed symbols. (C22:L163-188)
- **C22-017** [E][security] Production bans are zero kill_on_drop, no new sysinfo dependency/resource path, and no mocked/hardcoded panel data. (C22:L184-188)
- **C22-018** [E][observability] The candle-rs smoke requires system_probe and feasibility_report, optional alternative rows, real panel counts, non-zero ResourceDelta VRAM, and a skip only when RESOURCE_MARGIN_THRESHOLD_PCT<20%; zero kill_on_drop is also observable evidence. (C22:L189-200)
- **C22-019** [E][done-gate] Twelve static checks cover the rename/schema, event constants, two construction paths, component/flag, six metrics, and workspace compile. (C22:L201-253)
- **C22-020** [E][compatibility] Acceptance explicitly requires old MessagePack payloads to deserialize without panic through serde defaults and resource collection not to alter the child lifecycle. (C22:L255-274)
- **C22-021** [E][files] Work modifies five Rust/config/test files plus ReasoningDrawer/drawers state, adds FeasibilityPanel, modifies phase_4F.toml and possibly contract tests, and creates C-22-decisions.md. (C22:L285-307)
- **C22-022** [E][conflict][Δ+] The prose calls StubResult's change “three fields” but enumerates four: alternative_index, stdout_hash, stderr_summary, and resource_delta. Wiring/acceptance consistently require four. (C22:L27-32, L93-100, L169-170, L260-261)
- **C22-023** [E][conflict][Δ+] All six metrics are ordered non-zero, but skip_rate and escalation_rate are fractions that may legitimately be 0.0; the same smoke only conditionally fires skip and does not require escalation. Measurement truth and the non-zero gate can therefore conflict. (C22:L145-153, L190-197, L245-248, L271-283)
- **C22-024** [E][ambiguity] The UUID command/parser says run once per GpuInfo, but nvidia-smi -q can return multiple GPUs and no association rule is stated; the regex also names lowercase hex only. (C22:L76-80)
- **C22-025** [E][conflict][Δ+] The parallel-safety explanation says C-21 touches session_dag and C-23 touches cloud hardening, which disagrees with C-0's C-21 self-modification and C-23 SessionDag UI descriptions. File-overlap claims need revalidation. (C22:L8-12; C0:L120-135)
- **C22-026** [E][ambiguity] The Tauri source is cited once as commands.rs:433 and once as src-tauri/src/lib.rs:433; exact ownership must be resolved from code before wiring. (C22:L130-143, L174-179)
- **C22-027** [E][recovery] If constructor config/export differs, reuse existing AppConfig or make the constructor public; if C-19 ran first, ungate rather than duplicate its tests. These are explicit adaptation paths, not permission to redesign the probe. (C22:L122-128, L155-161, L309-313)

### C-30_surgical_oracle_mode.md

- **C30-001** [E][status] C-30 is a 2–3 session pipeline-mode plan after C-17/C-24, parallel with C-26..C-29; it adds surgical decomposition, answer caching, DKG tools, project detection, LOC gates, and VRAM config. (C30:L1-9)
- **C30-002** [E] Verified missing surfaces are the surgical types/decomposer/factory dispatch, weighted first-task detection, final-answer—not decomposition—Oracle caching, four catalog tools, VRAM tiers, and interactive/autonomous blast gates. Existing hooks are parse_task_flags, run_research_pipeline, indexes_exist, and build_tool_catalog. (C30:L11-45)
- **C30-003** [E][schema][Δ+] SurgicalPlan holds Vec<SurgicalStep>, AffectedSet, ComplexityClass, and RegressionGate. AffectedSet has direct/dependent PathBuf lists and total_loc_estimate; steps have description, target_file, optional target_fn. (C30:L62-70)
- **C30-004** [E][schema] RegressionGate contains must_pass, allow_new_failures, snapshot_hash; TestBaseline contains snapshot_hash and pre_existing_failures; ComplexityClass is Simple/Medium/Complex; all fallible paths return SurgicalError and never unwrap. (C30:L66-71)
- **C30-005** [E] SurgicalDecomposer opens the project DKG, query_reachability traverses target blast radius, ReachabilityMatch paths populate AffectedSet, and stored node LOC metadata supplies the estimate. Synthetic production sets are banned. (C30:L73-78, L213-215)
- **C30-006** [E][config] Classification is Simple when files≤3 and LOC≤200; Medium when files≤8 or LOC≤500; Complex otherwise. (C30:L79-81)
- **C30-007** [E][determinism] Generate one step per affected file in DKG topological order, capped at 1/4/8 for Simple/Medium/Complex. (C30:L82-84)
- **C30-008** [E][determinism] Gather DKG test-coverage names and blake3-hash sorted must_pass for the RegressionGate. (C30:L85-87)
- **C30-009** [E][permissions] Emit surgical_loc_warning above 500 LOC; interactive mode returns ConfirmationRequired before a plan and retries with confirmed=true after user consent; autonomous mode escalates above 1000 LOC. (C30:L88-92)
- **C30-010** [E] surgical=true dispatches SurgicalDecomposer before research_decompose, replaces ConceptDecomposition with plan steps, emits surgical_plan_created with class/file count, and leaves surgical=false unchanged. (C30:L94-100)
- **C30-011** [E][config] When no index exists, weighted signals are Cargo.toml/go.mod/pyproject.toml=REQUIRED, ≥10 .rs files/package.json=STRONG. (C30:L102-106)
- **C30-012** [E][permissions][fallback] Any REQUIRED signal prompts “Detected new project. Build knowledge index? [Y/n]” interactively; confirmation or autonomous mode scans, rejection proceeds without an index and falls back to symbol_impact. (C30:L107-110)
- **C30-013** [E][events] Every detection result emits project_detection_signal with signal_name, weight, and fired. (C30:L111-112)
- **C30-014** [E][schema] OracleAnswerCache is a bounded map from two 32-byte blake3 hashes (query and project) to CachedAnswer(answer, unix cached_at, source_tier). (C30:L114-124)
- **C30-015** [E][lifecycle] TTL is T1Authoritative 2,592,000s, T2Strong 604,800s, T3Weak 86,400s, T4Rejected zero/never cached. (C30:L119-120)
- **C30-016** [E][lifecycle] Cache API is get(key,now), insert, evict_expired(now), len; exceed max_entries triggers LRU eviction and adds no dependency beyond std+blake3. (C30:L121-124)
- **C30-017** [E][observability] /research hashes query text and canonical project-root path; a hit bypasses the pipeline and emits oracle_cache_hit with age/tier. (C30:L126-131)
- **C30-018** [E][fallback] A miss runs research normally; on Ok, HIGH/MEDIUM/LOW maps to T1/T2/T3, inserts, and emits oracle_cache_miss. Error and T4 answer mapping are not specified. (C30:L132-134)
- **C30-019** [E][integration] dkg_callers accepts symbol, follows inbound CALLS, and returns caller paths; dkg_dependents accepts symbol, follows outbound IMPORTS/USES, and returns dependent paths. (C30:L136-143)
- **C30-020** [E][integration] dkg_data_flow accepts symbol/max_depth, traverses PARAMETER/RETURN/ASSIGNMENT/FIELD_ACCESS, and returns source-target-type triples. (C30:L143-145)
- **C30-021** [E][integration][failure] dkg_subgraph accepts root/depth, returns bounded-BFS compact nodes/edges JSON; every handler opens the session-root DKG and returns error JSON on QueryFailed without panic. (C30:L146-149)
- **C30-022** [E][config][Δ+] research.vram_tiers mirrors six hardcoded labels—8gb, 12gb, 12gb_7b, 16gb, 24gb, cloud—with search/depth/breadth/token/speculation fields, although the check only requires ≥5. (C30:L151-156)
- **C30-023** [E][tests] TDD contract coverage includes constructibility, all three class bounds, deterministic gate hash, miss→hit, T1/T3 expiry, T4 rejection, max eviction, all four tool catalog entries, VRAM count, >500 warning, and Cargo REQUIRED detection. (C30:L158-183)
- **C30-024** [E][conflict][done-gate] The source enumerates 17 tests despite saying “minimum 12” and expects 17 passed. The file is placed under crates/contracts, but the check invokes cargo test -p elai-orchestrator; acceptance also mislabels the regression-hash proof as an oracle_cache_* test. Test-package/filter ownership must be corrected. (C30:L158-183, L261-264, L279-280, L301, L316)
- **C30-025** [E] WIRING_CONTRACT exposes seven surgical/error types, decomposer, three cache types, five events, four tools, and VRAM config from CLI task/research/tool entry points. (C30:L185-212)
- **C30-026** [E][security] TTL, real DKG, real handler queries, and pre-plan interactive gate are stub bans; no hardcoded sets, unconditional cache hits, or panic returns. (C30:L213-219)
- **C30-027** [E][observability] Three smokes prove a ≤3-file surgical plan/regression gate, Oracle miss then hit within tier TTL, and a >500 interactive confirmation before plan return. (C30:L221-232)
- **C30-028** [E][done-gate] Acceptance requires real traversal, exact class/gate behavior, 500/1000 thresholds, false-path preservation, TTL/LRU, live two-call cache, four correct tool schemas, six-ish VRAM config, detection events, 17 tests, and workspace regression safety. (C30:L272-305)
- **C30-029** [E][conflict][Δ+] OracleAnswerCache explicitly uses std HashMap in production, while C-0 says C-15 bans production HashMap in favor of FxHashMap. This cross-plan type conflict must be resolved before implementation. (C30:L114-124; C0:L78-81)
- **C30-030** [E][ambiguity][Δ+] LRU is underspecified: CachedAnswer has no access timestamp/order field, and project_hash is described as canonical-path hash rather than project-content hash, so access recency and invalidation after source changes are undefined. (C30:L114-134)

### C-41_skills_driven_workflow_formalization.md

- **C41-001** [E][status] C-41 formalizes implicit skills into a first-class SkillRegistry for prompt-engineer, citing a plan-asserted 44%→83% intent result. Phase 4A/prompt-engineer are required, C-39 is recommended, and it should precede C-44. (C41:L1-23)
- **C41-002** [E] Existing .claude/skills and agent prompts already have INPUT_FORMAT/OUTPUT_FORMAT/TRIGGERS; the missing third is addressable runtime registry plus dispatcher instead of inline classification. ELAI's classes are SIMPLE/FOCUSED/HEAVY/IDEA plus sub-domain. (C41:L25-40)
- **C41-003** [E] A new skill requires skills/{name}/SKILL.md, skills/{name}/meta.toml, and registry.toml entry; prompt-engineer looks it up, selection emits confidence, and versions support gradual rollout. (C41:L42-49, L63-66)
- **C41-004** [E][evidence] paper-ingestor must produce workflow/papers/scientific-workflow-interface.md containing routing protocol and SciQuest structure. (C41:L57-61)
- **C41-005** [E][schema] Each meta.toml declares name, version, triggers, inputs, and outputs; SkillRegistry loads all at startup and builds trigram+sparse-keyword indexes. (C41:L63-71)
- **C41-006** [E][determinism] lookup(task) returns score-descending Vec<(SkillCandidate,score)>; prompt-engineer injects top three candidates, but top-1 below a confidence threshold explicitly falls through to a default scaffold. (C41:L68-76)
- **C41-007** [E] Initial skills are simple_query, focused_edit, heavy_plan, idea_intake, oracle, planning_workflow, paper_ingest, source_ingest, and benchmark_run, each with both files. (C41:L78-81)
- **C41-008** [E][events][ambiguity] skill_lookup, skill_selected, and skill_fell_through are all said to carry top_candidates, chosen_skill, confidence, and task_id, while acceptance wants non-null fields; what chosen_skill means on fall-through is not defined. (C41:L83-84, L174)
- **C41-009** [E][tests] TDD covers loading/indexing, deterministic identical-input order, threshold/fall-through event, registered-vs-inline prompt non-identity, and 100 labeled synthetic tasks at ≥73% accuracy. (C41:L86-96)
- **C41-010** [E][compatibility] Every skill is semver-versioned; a breaking input/output major bump forces fall-through until prompt-engineer declares compatible. (C41:L98-100)
- **C41-011** [E][surface] SkillsPanel lists registered skills and recent selection events and allows a debugging manual override; any /task or /explore invoking prompt-engineer reaches lookup. (C41:L102-104, L121-123)
- **C41-012** [E][tests] skill_registry_contract verifies the lookup signature, monotonic scores, and fall-through. (C41:L106-107)
- **C41-013** [E] Produced runtime types are SkillRegistry, SkillCandidate, SkillMeta, files/index, three events, and SkillsPanel; consumed systems are prompt-engineer, SSB, and filesystem. (C41:L109-123)
- **C41-014** [E][failure][observability] skill_lookup emits even on a cache hit; an empty registry must emit an explicit error row rather than silently return empty; threshold fallback must emit skill_fell_through. Successful prompts leave both selection rows and a prompt-engineered artifact. (C41:L124-135)
- **C41-015** [E][done-gate] Checks require directory/index, at least nine registry entries, three runtime types/events, TDD accuracy, and contract tests. Acceptance additionally removes inline routing, enforces versioning, requires non-null event fields, tests-before-wiring, and a rendered panel. (C41:L136-177)
- **C41-016** [E][files] Files add the global index, 18 per-skill documents, three agent skill modules/tests, SkillsPanel, paper artifact, and modify prompt_engineer, SSB schema, and Cargo without adding a workspace member. (C41:L179-200)
- **C41-017** [E][extensibility] After landing, non-Rust contributors add skill files/metadata without changing Rust, and C-44 can route model backends from skill_selected. (C41:L207-213)
- **C41-018** [E][ambiguity][Δ+] The score range, confidence default, sparse/trigram weighting, and equal-score tie-break are not defined despite deterministic-order acceptance. (C41:L68-76, L86-96)
- **C41-019** [E][ambiguity][Δ+] A cache hit is regulated, but no cache is otherwise specified—key, lifetime, invalidation, and storage remain undefined. (C41:L124-128)
- **C41-020** [E][ambiguity] Major compatibility forces fall-through, but consumer compatibility declaration/discovery and coexistence of old/new skill versions are not specified. (C41:L98-100)
- **C41-021** [E][security][ambiguity] Manual override is explicitly for debugging, but authorization, scope, persistence, and its event/evidence behavior are not stated. The task names dashboard-v2/SkillsPanel.svelte while the Files section places it under src/lib/components/. (C41:L102-104, L196)

### C-51_aider_polyglot_harness_adoption.md

- **C51-001** [E][status] C-51 is test infrastructure only after shipped Phase 4F, parallel with C-35..C-50/C-52/C-53; it lands a six-language 225-problem harness with no benchmark/model invocation, reserving the real run for C-55. (C51:L1-10)
- **C51-002** [E][status][evidence] The source cites the 225-problem leaderboard and a qwen2.5-coder:32b/Ollama result of 72.9% as analogous to ELAI's local-32B tier. The CPU-only landing/run split is intentional: review/smoke need no GPU, while RunPod/local GPU later binds the adapter by config. (C51:L12-27)
- **C51-003** [E][workflow] For each task the harness presents scaffolding, collects emitted code, runs hidden tests in a sandbox, and scores pass/fail. (C51:L29-31)
- **C51-004** [E][license][determinism] Vendor the MIT harness and corpus under external/aider_polyglot, include LICENSE, and pin the git SHA in external/manifests.toml. (C51:L40-47, L102-105)
- **C51-005** [E][integration] PolyglotAdapter accepts problem_dir+prompt, runs prompt-engineer→executor→wait→collect emitted file, and uses whichever elai_cc.toml backend is configured rather than binding a model itself. (C51:L49-54)
- **C51-006** [E][security] Hidden tests use C-11 isolation with gcc, go test, mvn, jest, pytest, and cargo test shims; host leakage is forbidden. (C51:L56-59, L102-106)
- **C51-007** [E][schema][Δ+] Per-task scorecard fields are problem_id, language, time_ms, tokens_in, tokens_out, cost_usd, and passed; aggregate format must match Aider for direct comparison. (C51:L61-64)
- **C51-008** [E][events] Emit polyglot_task_started, polyglot_task_completed, and polyglot_scorecard_emitted. (C51:L66-67)
- **C51-009** [E][tests] TDD proves exactly 225 tasks with non-empty per-language counts and a synthetic adapter round-trip. (C51:L69-72)
- **C51-010** [E][tests] A known-good Python solution must pass, an intentionally bad one must fail, and scorecard JSON must match the leaderboard format. (C51:L72-76)
- **C51-011** [E][failure][Δ+] elai benchmark polyglot without GPU exits with an explicit RunPod/local-GPU-required error; --dry-run instead runs five samples against a mock backend. (C51:L78-82)
- **C51-012** [E][tests] tests/contracts/polyglot_adapter_contract.rs is an additional gate. (C51:L84-85)
- **C51-013** [E] Produced surfaces are the vendor/corpus, PolyglotAdapter, six language runners, CLI, and three events; consumed surfaces are the existing agent pipeline, C-11 sandbox, configured backends, and SSB. (C51:L86-101)
- **C51-014** [E][done-gate] Static proof checks vendor/LICENSE/manifest/adapter/events/tests and exact five-task dry-run output. (C51:L108-118)
- **C51-015** [E][done-gate] Acceptance requires 225 tasks, all six runners, end-to-end synthetic dispatch, scorecard diff against existing leaderboard JSON, all events/TDD/contracts, refusal without GPU, and working dry-run. (C51:L120-131)
- **C51-016** [E][non-goal][Δ+] No public benchmark number is produced; closure is infrastructure landed and smoke-tested. C-55 is characterized as GPU bind/config flip, not a C-51 code path. (C51:L132-149)
- **C51-017** [E][files] Adds vendor/manifest, adapter, six shims, TDD/contract tests; modifies CLI and SSB schema. (C51:L135-145)
- **C51-018** [E][conflict][ambiguity] “No benchmark run in C-51” coexists with a CLI described as refusing a full sweep only when GPU is absent, so GPU-present behavior during this patch is unclear. GPU detection, language distribution, mock semantics, output location, concurrency, timeout/retry, and event payloads are also unspecified. (C51:L1-7, L46-84, L120-133)

### C-56_codebase_memory_dkg_31_repo_eval.md

- **C56-001** [E][status] C-56 is RunPod-dependent after B-29/B-30/C-38 and parallel with C-54/C-55/C-57/C-58; it evaluates ELAI DKG/symbol tools on a published 31-repository set. (C56:L1-12)
- **C56-002** [E][evidence][Δ~] The cited paper figure is 83% answer quality versus a 92% file-explorer agent, with tenfold fewer tokens and 2.1-fold fewer tool calls across 31 repositories—not simply an unqualified 83% target. (C56:L14-18)
- **C56-003** [E][status] Both outcomes are valid: meeting the published figures supports publication; missing them must identify whether DKG, CatRAG, A-RAG, or reranker is weaker. (C56:L20-30)
- **C56-004** [E][evidence] paper/source ingestion captures the exact 31-repo manifest, query set, answer rubric, and produces codebase-memory-interface.md. (C56:L39-43, L110-114)
- **C56-005** [E][determinism] Vendor exactly 31 repositories at pinned SHAs under benchmarks/codebase_memory/repos with manifest.toml. (C56:L45-47)
- **C56-006** [E][schema] Vendor the paper's N-per-repo queries and ground-truth answers; N is not fixed in the plan. (C56:L49-50)
- **C56-007** [E][workflow] CodebaseMemoryAdapter executes compute_repo_map then symbol_impact/flow/context as needed and records emitted answer, tokens, and tool calls. (C56:L52-54)
- **C56-008** [E][determinism][ambiguity] AnswerQualityScorer matches the paper's rubric, which may be model-judge or string similarity depending on query; Sonnet 4.6 is fixed/documented for fairness. (C56:L56-58)
- **C56-009** [E][observability] Every repository and aggregate report includes answer-quality percentage, mean tokens/query, and mean tool calls/query, cross-tabulated against published baselines. (C56:L60-62)
- **C56-010** [E][evidence] Hub-detection and caller-ranking are separately scored against a claimed “matches or exceeds the explorer on 19 of 31 languages” result, with symbol_impact treated as the matching ELAI surface. (C56:L64-67)
- **C56-011** [E][events] Produce methodology/results like C-54 plus codebase_memory_query_run, codebase_memory_answer_scored, and codebase_memory_scorecard_emitted. (C56:L69-72)
- **C56-012** [E] CLI reachability is elai benchmark codebase-memory --runpod-id; outputs are CodebaseMemoryAdapter, scorer, session scorecard/methodology, and repository results doc. (C56:L74-90)
- **C56-013** [E][honesty][security] Judge is fixed per run, fixed judge+seed must make scoring deterministic, per-repo results cannot be hidden, and underperformance must be prominent with candidate improvements. (C56:L91-97)
- **C56-014** [E][done-gate] Deterministic checks prove 31 directories, manifest, adapter, current-session scorecard, results doc, and three events; they do not themselves validate metric values. (C56:L99-108)
- **C56-015** [E][done-gate] Acceptance requires source artifact, pinned corpus/queries/ground truth, adapter/scorer, completed 31-repo per-repo+aggregate run, separate specialized scores, three events, cross-tabulated results, conditional improvements, and methodology. (C56:L110-124)
- **C56-016** [E][files] Adds the corpus, manifest, queries, adapter/scorer, three result artifacts, paper artifact, CLI command, and three SSB schema types. (C56:L126-138)
- **C56-017** [E][ambiguity][Δ~] Despite “head-to-head/A-B” wording, the plan runs only ELAI and cross-tabulates published Codebase-Memory/file-explorer numbers; denominator definitions for “10×” and “2.1×,” model/version control for ELAI answers, and repeated-run variance are unstated. (C56:L1-7, L26-30, L52-62)

### C-58_agentforge_rrk_reranker_head_to_head.md

- **C58-001** [E][status] C-58 combines two RunPod evaluations: ELAI eight roles versus AgentForge five roles on SWE-bench Lite, and current ≤200M cross-encoder versus RRK 8B listwise retrieval. C-45 is recommended; C-54..C-57 are called parallel-safe. (C58:L1-10)
- **C58-002** [E][evidence] Plan-asserted baselines are AgentForge 40.0% SWE-bench Lite/26–28 points over single agents and RRK 8B at 3–18× the speed of 0.6–4B rerankers. (C58:L12-17)
- **C58-003** [E][workflow][Δ+] ELAI roles are Executor, Auditor, Researcher, Designer, PromptEngineer, ContextCompiler, MemoryWriter, MediaValidator; AgentForge roles are Planner, Coder, Tester, Debugger, Critic; compare solve rate and cost/task on the same corpus/hardware. (C58:L21-32)
- **C58-004** [E][dependency] If AgentForge wins per cost, C-45 contribution data identifies ELAI roles that did not earn cost. (C58:L34-35)
- **C58-005** [E][constraint] RRK's claimed speed does not override ELAI's <50ms target; the pilot must measure both retrieval quality contribution and latency at typical query volume. (C58:L37-46)
- **C58-006** [E][evidence] Separate paper/source ingestion grounds AutoCodeAI and RRK and produces two interface artifacts. (C58:L48-59, L69-75, L168-169)
- **C58-007** [E][license][security] Vendor AutoCodeAI at a pinned SHA with LICENSE and run it through an isolation harness on the same SWE-bench Lite set. (C58:L61-63, L116-121)
- **C58-008** [E][determinism][conflict] The actual runner command executes suites sequentially/back-to-back on identical hardware and task corpus, not interleaved, although context line 30 says AgentForge runs “in parallel.” (C58:L23-31, L65-67, L120-121)
- **C58-009** [E][config] Add rrk_reranker Cargo feature, load the released 8B checkpoint, and expose it as a config-flagged alternative in knowledge/reranker.rs. (C58:L71-75)
- **C58-010** [E][done-gate][Δ~] Measure p99 on top-K-vector query shapes; reject only when latency is >100ms, described as 2× the 50ms target. Thus exactly 100ms is not explicitly rejected. (C58:L77-80, L116-119)
- **C58-011** [E][quality] On a held-out set such as C-56 queries, compare top-K precision and recall for current and RRK. (C58:L82-84)
- **C58-012** [E][decision][Δ+] Ship RRK only as opt-in when latency is acceptable and quality improves by at least five percentage points; otherwise archive it as measured-not-worth-it. Default never auto-flips. (C58:L86-88, L116-120)
- **C58-013** [E][events] Produce two scorecards, combined methodology/results, and head_to_head_run_started, head_to_head_arm_completed, reranker_eval_run, reranker_eval_decision. (C58:L90-107)
- **C58-014** [E] Consumers are SWE-bench Lite, optionally/conditionally C-45 attribution, C-56 query corpus, and knowledge reranker; CLI has separate head-to-head and reranker-eval commands with runpod-id. (C58:L108-115)
- **C58-015** [E][done-gate] Static checks prove vendor/LICENSE, adapter, RRK loader, two current-session scorecards, and four event names; no executable test command is specified. (C58:L123-133)
- **C58-016** [E][done-gate] AgentForge acceptance requires completed run/scorecard and conditionally either a C-45-informed consolidation list or confirmation that eight roles earn cost. (C58:L135-145)
- **C58-017** [E][done-gate] RRK acceptance requires loader/feature, p99, precision/recall delta, applied adoption/archive matrix, gated/default-unchanged adoption, all events, and combined docs. (C58:L147-156)
- **C58-018** [E][files] Adds vendor, adapters/reranker trait modules, two scorecards, combined methodology/results, two paper artifacts, two CLI commands, four SSB schema types, and a knowledge module export. (C58:L158-172)
- **C58-019** [E][ambiguity][Δ+] C-56 is declared parallel-safe but its query corpus is also a consumed quality dataset; “e.g.” permits another held-out set, yet no fallback corpus is named. (C58:L8-10, L82-84, L108-111)
- **C58-020** [E][ambiguity] Fairness controls beyond task corpus/hardware are absent: model/backend, budgets, seeds, task ordering, warm-up, top-K, typical volume, repetitions, confidence intervals, and scorecard schemas are not fixed. (C58:L21-46, L65-96)

## Cross-source interactions and end-to-end composition

- **X-001** [E] C-0 places C-2 after C-1's event pattern and before C-3/C-4/C-7/C-8/C-10/C-16. C-2 supplies persisted orchestration rows; later sessions query, render, complete producer logic, and schedule Cosmos/anti-spiral work. (C0:L36-61, L83-98; C2:L150-180, L284-294)
- **X-002** [I] C-2's append-only events and latest-agent reduction form a live dashboard loop: producer→SSB row→viewer reduction/query→Tauri→Bubble/Orchestration/Cosmos UI. The sources do not define subscription versus polling, retention, or simultaneous-role ordering. (C2:L86-197)
- **X-003** [E] C-6 controls the presentation and input behavior of live task streams but creates no backend cancellation API; it must discover and reuse the stop/submit mechanisms produced elsewhere. (C6:L107-166, L256-267)
- **X-004** [E] C-18 provides regression proof for C-2 event serialization/viewer behavior and for symbol/graph/artifact/DAG surfaces that later plans consume; it deliberately adds tests rather than runtime behavior. (C18:L17-22, L38-170)
- **X-005** [E] C-22 turns the feasibility path from None/unwired into production data, two new events, panel rendering, explicit child termination, and measured phase_4F evidence; C-19 conditionally owns the adversarial tests. (C22:L122-200, L255-283)
- **X-006** [E] C-30 consumes C-17 indexing and C-24 DKG tools to size surgical changes; C-56 later evaluates that DKG/tool architecture over 31 repositories. Passing C-30's functional contracts does not substitute for C-56's quality/cost study. (C0:L137-174, L310-313; C30:L73-149; C56:L52-67)
- **X-007** [I] Project detection can create the index needed by SurgicalDecomposer, after which its DKG AffectedSet feeds ordered steps/regression gates; if indexing is rejected, the source only promises symbol_impact fallback, so full surgical equivalence is not established. (C30:L73-113)
- **X-008** [E] C-41 selects prompt scaffolds and feeds top candidates to prompt-engineer; C-44 is expected to use skill_selected for backend routing, while C-51's benchmark adapter already invokes prompt-engineer and therefore exercises whichever registry/backend stack is active. (C41:L73-84, L207-213; C51:L49-54)
- **X-009** [E] C-51 is the reproducible six-language harness boundary; C-55 alone binds real GPU models, runs 225 tasks, and publishes the Aider draft PR. (C0:L278-281, L305-308; C51:L17-27, L132-149)
- **X-010** [E] C-56 and C-58 generate machine-readable scorecards/methodology/results for C-59. Negative results are retained as improvement targets or measured-not-worth-it decisions rather than omitted. (C0:L325-328; C56:L91-124; C58:L86-107)
- **X-011** [E] C-58's role comparison can consume C-45 credit attribution, and its reranker quality arm can consume C-56 queries; both dependencies are soft/conditional in prose, so scheduling them truly in parallel requires alternate context/data. (C58:L8-10, L34-35, L82-111)
- **X-012** [I] Evidence maturity rises from static manifest/checks (all plans), to deterministic contract tests (C-2/C-6/C-18/C-22/C-30/C-41/C-51), to live cloud/GPU measurement (C-22/C-30/C-56/C-58), to C-59 publication. A plan checkbox is not equivalent to any later stage. (C0:L1-15, L293-328)

## Additions and corrections relative to 03-C-SUMMARY_INDEX.discovery.md

- **D-001** [Δ~] C-0 is not merely a non-implementation index: it is a feature/status/dependency manifest with 58 explicit session rows, closure maps, estimates, and provenance. Its claimed 59 slots conflict with the missing C-5 row. (C0:L17-349)
- **D-002** [Δ+] The C-0 report-status breakdown totals 105, not its stated 103; Wave 7–9 source counts also say 11+15 papers but total 19 without deduplication rules. (C0:L351-368)
- **D-003** [Δ+] C-0's closure map adds exact B/guide/phase ownership, including B-27→the missing C-5, B-1→C-18/C-8, and phase-4K→C-4. (C0:L370-426)
- **D-004** [Δ~] C-2 does not define 12 wholly new variants. It introduces ten missing enum variants while four Cosmos variants already exist; the “12 new” check mixes new and existing names. (C2:L23-31, L52-70, L200-205)
- **D-005** [Δ+] C-2 specifies exact AgentStatusChanged fields, node/repair payloads, T_DAG=10, Cosmos event timing, ambiguity gate placement, nine named tests, and logged non-panicking emit failures. (C2:L52-148, L181-185, L262-282)
- **D-006** [Δ+] C-2 explicitly defers interrogation ambiguity, Cosmos scheduling/stage, anti-spiral logic/personas, several UI/command surfaces, and C-5 research events; variant definition is not behavioral implementation. (C2:L284-294)
- **D-007** [Δ+] C-6's missing exact values are the complete teal/text palette and --font-display; it also requires a five-item decision record and named SPEC-DELTA icons. (C6:L65-83, L99-105, L130-137)
- **D-008** [Δ~] C-6's seven-file 11px task is checked by a broader zero-hit dashboard-src gate, an unresolved scope mismatch the summary does not preserve. (C6:L85-92, L184-186, L220)
- **D-009** [Δ+] C-6 adds exactly-once component tests, aria behavior, plain-Enter preservation, no-new-dependency policy, and user escalation if JetBrains Mono is absent. (C6:L107-120, L242-267)
- **D-010** [Δ~] C-18's exact footprint is ten added test files, four modified Rust test hosts, and a decision doc—not only “at least ten files.” It describes about 20 distinct cases. (C18:L38-130, L228-244)
- **D-011** [Δ~] C-18's “D1 through D5” label is wrong; the exact contracts are D1/D2/D3/D6/D11. (C18:L89-100)
- **D-012** [Δ+] C-18 codifies reset non-emission, no-network symbol dispatch, exact 85% handoff, five-superstep channel semantics, byte-identical Mermaid/SSB/MessagePack behavior, and future-gated live-sidecar paths. (C18:L38-130)
- **D-013** [Δ~] “Zero production changes” means zero production logic: the plan still appends tests inside ssb.rs, bsp/engine.rs, and session_dag.rs and writes outside crates. (C18:L1-4, L228-244)
- **D-014** [Δ~] C-22 adds four StubResult fields despite repeatedly calling them three; each has serde(default), and stdout hashing is blake3 while stderr is truncated to 256 characters. (C22:L93-100)
- **D-015** [Δ+] C-22's resource-query failure is non-fatal, GPU UUID parse failure yields None, child.kill is the sole kill path, and event payloads are specified for accepted alternatives/escalation. (C22:L76-120)
- **D-016** [Δ~] C-22's six “all non-zero” metrics conflict with conditional zero-valued skip/escalation rates; real measurement must not be falsified to satisfy a textual gate. (C22:L145-153, L189-197)
- **D-017** [Δ+] C-22 has conditional C-19/C-22 test ownership, mistaken C-21/C-23 file descriptions, a commands.rs/lib.rs Tauri path disagreement, and a multi-GPU UUID association gap. (C22:L8-12, L130-161)
- **D-018** [Δ+] C-30 gives all surgical/cache struct fields, exact project signal weights, rejection fallback, answer confidence mapping, tool directions/edges/schema, and 17—not merely “at least 12”—contract tests. (C30:L62-183)
- **D-019** [Δ~] C-30's six VRAM labels are explicit; only the mechanical gate is ≥5. The summary's count ambiguity is a requirements-strength mismatch, not an unknown label set. (C30:L151-156)
- **D-020** [Δ+] C-30 leaves LRU recency and cache invalidation undefined and explicitly chooses std HashMap in conflict with C-15's FxHashMap ban. (C30:L114-134; C0:L78-81)
- **D-021** [Δ+] C-30's interactive rejection proceeds without an index via symbol_impact fallback; Simple can cover up to three files while its step cap is one, an unresolved affected-file coverage tension. (C30:L79-84, L102-110)
- **D-022** [Δ+] C-41 injects top three candidates, has explicit default/fell-through events, per-skill metadata, semver breaking-change fallback, manual debug override, and an error row on empty registry. (C41:L63-128)
- **D-023** [Δ+] C-41's cache-hit rule refers to an otherwise nonexistent cache design, and deterministic lookup lacks threshold/weight/tie defaults and version compatibility mechanics. (C41:L68-100, L124-128)
- **D-024** [Δ+] C-51's per-task scorecard has seven exact fields; hidden tests run through six named toolchains, and the no-GPU refusal has a defined dry-run exception using five mock tasks. (C51:L49-82)
- **D-025** [Δ=] C-51 explicitly refuses to publish any benchmark number; C-55 owns model binding, full execution, and publication. (C51:L120-149)
- **D-026** [Δ~] C-56 acceptance does not require meeting all three published figures. Underperformance is valid closure only if per-repo evidence and an explicit improvement-target list are published honestly. (C56:L20-30, L91-124)
- **D-027** [Δ~] C-56's 83% is reported against a 92% explorer; “head-to-head” is actually one ELAI run cross-tabulated against published baselines, with efficiency denominators left unstated. (C56:L14-18, L52-62)
- **D-028** [Δ+] C-56 fixes Sonnet 4.6 as judge, requires fixed-seed determinism and per-repo visibility, separately scores hub/caller queries, and leaves N queries per repository open. (C56:L49-97)
- **D-029** [Δ~] C-58's RRK gate is reject when p99>100ms, not strictly “p99 below 100ms”; adoption additionally requires ≥5pp quality improvement and remains opt-in/default-unchanged. (C58:L77-88, L116-120)
- **D-030** [Δ+] C-58 names all eight/five roles, solve-rate+cost outputs, back-to-back fairness rule, conditional C-45 role consolidation, two scorecards, and archive-as-not-worth-it fallback. (C58:L21-35, L61-121)
- **D-031** [Δ+] C-58 contains two scheduling conflicts omitted by the summary: “parallel” versus sequential AgentForge execution, and parallel-safe C-56 versus consumption of its query corpus. (C58:L23-31, L65-67, L82-121)
- **D-032** [Δ+] None of C-56/C-58 defines repeated-run variance, confidence intervals, model budget controls, or executable metric-validating tests; file/event existence is not benchmark reproducibility. (C56:L99-124; C58:L123-156)

## Ambiguities, conflicts, constraints, and deferred boundaries

- **A-001** [conflict] C-0 claims 59 slots/files but omits C-5 from Sessions; do not invent its plan from closure-map shorthand. (C0:L17-349)
- **A-002** [conflict] C-0's report-status counts total 105 rather than 103, and paper-source subtotals do not reconcile to 19. (C0:L351-368)
- **A-003** [status] Every detailed source in this chunk is a plan/specification. C-0 marks its corresponding session PLANNED; prerequisites described as SHIPPED are dependencies, not status for the session itself. (C0:L10-15, L19-335)
- **A-004** [conflict] C-2's “12 new variants” is false under its own context: ten missing variants plus existing Cosmos names are checked. (C2:L23-31, L200-205)
- **A-005** [ambiguity] C-2 leaves the AgentStatusChanged owner and anti-spiral payload-struct granularity open; Cosmos tick can also be absent from the smoke window. (C2:L66-70, L103-109, L187-192)
- **A-006** [constraint] C-2 defines anti-spiral events only; C-10 owns detection/personas/watchdog and C-16 owns Cosmos scheduling/stage. (C2:L284-294)
- **A-007** [conflict] C-6 edits seven named 11px files but globally gates zero 11px; resolve unrelated occurrences before treating failure as C-6 scope. (C6:L85-92, L184-186)
- **A-008** [constraint] C-6 cannot add an IPC, dependency, font/CDN, or silently remove icons; missing font requires user escalation. (C6:L242-267)
- **A-009** [conflict] C-18's new-files-only wording conflicts with four modified source/test hosts and an out-of-tree decision doc; preserve the zero-production-logic rule. (C18:L1-4, L228-244)
- **A-010** [ambiguity] C-18's timestamp-based file count and inferred test-case aggregate are not exact provenance checks. (C18:L172-176, L208-244)
- **A-011** [conflict] C-22's metric gate can demand fabricated non-zero rates when no skip/escalation occurs; the measured value should remain authoritative. (C22:L145-153, L189-197)
- **A-012** [conflict] C-22's parallel-file explanation misidentifies C-21/C-23 ownership, and the Tauri command path is inconsistent. (C22:L8-12, L130-179)
- **A-013** [ambiguity] C-22 lacks multi-GPU UUID mapping and exact behavior for a constructor signature not matching new(); both require code-level resolution. (C22:L76-80, L122-128)
- **A-014** [conflict] C-30's std HashMap violates C-0/C-15's production FxHashMap policy. (C30:L114-124; C0:L78-81)
- **A-015** [ambiguity] C-30 does not define LRU metadata, cache concurrency/persistence, stale-project invalidation, error-miss events, T4 answer mapping, or default/maximum depths. (C30:L114-149)
- **A-016** [ambiguity] C-30 says one step per affected file but caps Simple at one while Simple permits three files; a single plan-level RegressionGate conflicts with “per step” injection; TestBaseline/allow_new_failures lack lifecycle; and task_runner versus research_tree ownership of the plan event differs between Tasks and Files. (C30:L62-100, L307-317)
- **A-017** [ambiguity] C-41 omits scoring weights/threshold/ties, cache design, semver negotiation, fall-through chosen_skill semantics, panel path, and manual-override authorization/evidence. (C41:L68-128, L174, L196)
- **A-018** [constraint][ambiguity] C-51 is infrastructure-only and publication is C-55; a mock five-task dry-run is allowed. Whether a GPU-present non-dry CLI may nevertheless run the full suite before C-55 is internally unclear. (C51:L1-7, L78-82, L120-149)
- **A-019** [ambiguity] C-56's “A/B” comparator, efficiency denominators, query N, judge repeatability, and “31 languages” wording are not resolved; benchmark claims must stay scoped to produced evidence. (C56:L1-18, L49-67, L91-97)
- **A-020** [conflict] C-58 must use the later explicit sequential/back-to-back rule despite earlier “in parallel” wording; C-56 data and C-45 attribution are conditional dependencies despite parallel/recommended labels. (C58:L8-35, L65-121)
- **A-021** [constraint] C-58 adopts RRK only at latency≤100ms and quality delta≥5pp, behind feature/config gates; otherwise it archives the result and never flips the default automatically. (C58:L73-88, L116-120)
- **A-022** [ambiguity] C-56/C-58 omit benchmark resource budgets, retries, task-order controls, repeated runs, confidence intervals, and machine validation of score values. (C56:L99-124; C58:L123-156)

## Zero-omission and duplicate audit

- **Z-001** All ten manifest paths were read from line 1 through EOF after exact SHA-256, line, and byte verification: 10 files / 2,554 logical lines / 147,739 bytes.
- **Z-002** Every source section class was checked: objective/status/prerequisites, context and source grounding, tasks/types/commands/UI, WIRING_CONTRACT, flags/defaults/config, lifecycle/fallback/security/privacy/compliance, deterministic checks, acceptance/tests, files/ownership, next/deferred work, and index closure/provenance maps. No explicit privacy/retention/compliance regime appears beyond license and sandbox/isolation requirements, so none was invented.
- **Z-003** Identifiers, payload fields, event/action names, state transitions, paths, thresholds, TTLs, flags, commands, tool schemas, benchmark metrics, test names/counts, and negative/non-goal rules are retained at their first owning finding; later sections cross-reference instead of silently multiplying them.
- **Z-004** Duplicate audit preserves repeated index-versus-plan claims as [Δ=], turns material drift into [Δ~], and labels contradictions explicitly. No planned item was upgraded to shipped, and no paper/competitor claim was independently endorsed.
- **Z-005** Scope audit: the only non-manifest discovery used was 03-C-SUMMARY_INDEX.discovery.md for orientation/deltas and an existing deep-discovery file for output-shape orientation. No unassigned original C-plan source was opened.

### Atomic finding count

This file contains **350 atomic ledger findings**: 71 from C-0, 29 from C-2, 22 from C-6, 24 from C-18, 27 from C-22, 30 from C-30, 21 from C-41, 18 from C-51, 17 from C-56, 20 from C-58, 12 cross-source interactions, 32 summary deltas, 22 ambiguity/constraint boundaries, and 5 audit findings. The category sum and identifier uniqueness were mechanically verified.
