# ELAI Implementation Audit — Aggregate Synthesis
Date: 2026-04-25
Scope: 23 sub-phase audits (pre-0, 0, 1A–1I, 2A–2D, 3A–3K, 4A–4E, 4K MVP)
Source: per-sub-phase reports in this directory + 6 lossless spec summaries in ../summary/
Method: HOSTILE — every claim verified against codebase via elai.map + Read; no status claim trusted.


================================================================================
EXECUTIVE VERDICT
================================================================================

23 sub-phases audited. Aggregated state of "phase pre-0 through 4E + 4K.1–4K.5":

  IMPLEMENTED (clean)              :  4 / 23  (1A, 1F, 2A, 3D)
  IMPLEMENTED with minor debt      :  6 / 23  (1B, 2A, 3A, 3F, 3G, 4D)
  PARTIAL (built, not fully wired) : 11 / 23  (1C, 1D, 1E, 1H, 1I, 2B, 2C, 2D, 3B, 3C, 3E, 3H, 3I, 3J, 4B, 4C, 4E, 4K)
  PARTIAL with major architectural drift : 2 / 23 (phase-0 stripping, 4K dashboard)
  PASS as evaluation only          :  2 / 23  (1F DROP-decided, 1H pending decision)
  REGRESSED / DEAD                 :  0 / 23

NO sub-phase is genuinely MISSING. The codebase is enormous (61K+ LOC at fork
time, plus thousands more added). Every sub-phase has real types, real tests,
real metrics files. The audits found exactly one pattern of failure — and it
explains the user's complaint that "the dashboard doesn't feel or look like
I wanted":

  THE PRIMARY FAILURE CLASS IS "BUILT BUT NOT WIRED."

The library code is correct. The trait surfaces are clean. The unit tests pass.
But the production hot paths (CLI session loop, task_runner main loop, rule
hooks, dashboard backend bridge) bypass or stub out the components those
phases delivered. The "intelligence in the infrastructure" thesis is currently
a library, not a system.


================================================================================
RECURRING FAILURE PATTERNS (with file:line evidence)
================================================================================

PATTERN 1 — DUAL DATABASE PATHS (.elai/ vs .elai_cc/)
  Origin: cli/main.rs hardcodes `.elai/dkg.db` literal at line 3752 + 5437.
  IndexingService writes to `.elai_cc/indexes/dkg.db` (orchestrator constant
  ELAI_DATA_DIR_NAME = ".elai_cc"). Net effect: pagerank, CatRAG, A-RAG,
  context telescope all silently read empty DB at runtime.
  Cascade impact: 1A (no caller hits PageRank), 1D (PageRank yields empty),
    1E (CatRAG overlay computed and discarded; AragOrchestrator corpus empty),
    3J.WIRE.STUBS (TreeSitterAstEvaluator + DkgQuerierAdapter silently
    fall through to EmptyDkgQuerier permanently).
  Remediation effort: 2-line fix.

PATTERN 2 — WRAPPER LAYER WITHOUT CALLERS
  PipelineGrc::verify_claim, ground_plan, generate_csq, check_inline_feasibility
  all exist at orchestrator/src/grc.rs as clean wrappers around the reasoning
  engine. ZERO production code paths call them. Tests + benches are the only
  callers. Result: every reasoning capability from Phase 2 is invisible at
  runtime — the model can produce code with no CSQ guard, no feasibility
  check, no plan grounding sweep.
  Cascade impact: 2B (sweep gate registered but ungrounded_claims always
    empty — gate vacuously passes), 2C (anti-rubber-stamp enforced inside
    engine but engine never invoked from auditor agent in production),
    2D (ReplanRequest never emitted because Blocked outcome never returned).

PATTERN 3 — DEAD INTERACTIVE-MODE GATES
  CliInteractiveGate at cli/src/main.rs:7354 — defined, never instantiated.
  Fsm::transition() is called directly throughout production, bypassing
  the interactive protocol entirely. The team's own
  workflow/reasoning/phase-3J.13-deferrals.md L58-61 admits "dead code".
  Cascade impact: 3A (7 evidence gates registered but no human-approve
    loop), entire INTERACTIVE-mode design unreachable.

PATTERN 4 — CONFIG STUB FALLBACK
  task_runner.rs has FIVE sites that pass `dispatch_context_profile: None`
  to ContextBudget::compute, falling back to zeroed [0,0,0] slots
  (line 1842, 6448, 6565, 6759, 7513). The CLI REPL path at
  cli/main.rs:7680-7727 sets all three slots from real measurements +
  ShortCoder — so the REPL works, but the TaskRunner-routed plan
  dispatches do not. Result: A-10 ModelCapabilityProfile machinery
  (4 tier constructors, attention-ceiling enforcement) is a library
  with no production caller.

PATTERN 5 — DUAL TABLES / DUAL HOOKS / TWO PARALLEL SYSTEMS
  - AgentGrcConfig table appears twice: reasoning crate (correct) and
    orchestrator/src/agent_pool.rs:467 (divergent — ContextCompiler and
    MediaValidator fall through to default(), losing observe-only
    enforcement and getting full 5-step chain).
  - Hook system appears twice: spec-compliant IpcHookDispatcher
    (plugins/src/ipc_hooks.rs, ZERO non-test callers) vs live
    InProcessHookDispatcher (orchestrator/src/hook_dispatcher.rs,
    fires from task_runner.rs at baby-step boundaries — NOT IPC frames).
  - Tool registry appears twice: IpcToolRegistry + Handlers in tools/src/
    are DEAD CODE in production; orchestrator wires the parallel
    InProcessToolDispatcher (elai_core::ToolDispatcher) instead. FSM
    filtering and TestingFirst gate live on the dead path.
  - Session persistence appears twice: SessionDag (3090 LOC, working
    library) vs legacy JSONL (real production). DAG runs as parallel
    write-only ledger; CLI hard-codes stub_ssb_seq: None which means
    trim Pass 1 produces zero stubs.

PATTERN 6 — STAGE EXISTS, NEVER FIRES AT RUNTIME
  - Vector indexer populates code_vec into cache during indexing but
    embed_code() is never called at query time — pipeline is effectively
    2-ranker (BM25 + text_vec) instead of spec's 3-ranker RRF.
  - Cross-encoder reranker batching path (run_batch_worker, mpsc) is
    test-only with no production spawn.
  - SearchMode cascade exists with full HYBRID->SPARSE_ONLY->BM25_ONLY
    state machine, but no QueryPipeline holds or consults a
    DegradationCascade — only one startup-time call site.
  - DAIRA trace REDUCED to 2 markers ("enter"/"exit") instead of 15
    specced; AdaptiveTool<T> wrapper + MarkerReport not present.

PATTERN 7 — MODEL FILES NOT STAGED
  - Cross-encoder ms-marco-MiniLM ONNX NOT in .elai_cc/models/
    (only Qwen2.5-Coder dirs present). Reranker degrades to RRF-only.
  - Phase 1H SPLADE evaluation cannot run: model fixtures never staged.
  - Phase 1C dual-model: ort = "=2.0.0-rc.12" violates CLAUDE.md
    Anti-Stale-Training rule "ort stable, never -rc".

PATTERN 8 — TIER C ISOLATION IS COSMETIC
  - 4A: separate process YES (dashboard at .../dashboard/src/lib.rs:1191
    spawns elai_cc_agent), but with NO isolation primitives. plain
    std::process::Command::new(...).spawn() at agent_pool.rs:291.
    No clone(), no seccomp, no AppContainer, no rlimits, no /proc masking.
  - WindowsAppContainerSandbox at sandbox.rs:326-693 is real code with
    Win32 calls — but enable_sandbox: false defaulted everywhere
    (dashboard/src/lib.rs:1183, task_runner.rs:2477, agent_pool.rs:153).
    Even if true, launch_agent_process never calls self.sandbox.launch_agent().
  - IpcGateway types + ToolFilter + 15 message variants all defined.
    No UnixListener/NamedPipeServer accept loop in orchestrator.
    Source self-admits "Production wiring scheduled in Session wire-4A".
  - Effective tier: A++ (in-process logic + separate process exec, but
    shared namespaces / syscalls / network / fs / proc / limits).

PATTERN 9 — DEAD PHASE 0 DEAD CODE
  Phase 0 spec said STRIP these. Still present:
    - crates/orchestrator/src/compact.rs (696 LOC, full claw-code compaction)
    - crates/core/src/usage.rs (279 LOC including ModelPricing,
      UsageCostEstimate, format_usd; spec said minimal TokenUsage only)
    - crates/providers/src/oauth.rs (DEFERRED in manifest, never removed)
    - crates/orchestrator/src/remote.rs (EVALUATE/STRIP per spec)
    - crates/orchestrator/src/prompt.rs (hardcoded-prompt resolution)
  Plus violations:
    - 84 std HashMap imports + 10 HashSet (HashMap ban not enforced)
    - 40+ entropy quarantine leaks across 8 crates (0.9 acceptance violated)
    - cli/src/main.rs is 11,181 LOC vs <500 LOC spec target (22x over)
    - Workspace has 16 crate members (9 spec + contracts + 6 Phase-1+
      additions: agent, ipc-types, web_search, scaffold, web_validator, dashboard)

PATTERN 10 — DASHBOARD MAJOR DRIFT (the user's actual complaint)
  Spec MVP (4K.1-4K.5) shipped, but a parallel Group-C (post-MVP) shell
  rewrite happened on top:
    - 98 Tauri commands implemented vs spec's 6
    - 9 polymorphic windows (Chat, Activity, Explore, Monitor, Plan,
      Preview, Research, Review, Settings) vs spec MVP single workspace
    - 10 drawers (AntiStaleness, Knowledge, Orchestration, ProjectIntelligence,
      ProviderVram, Reasoning, Research, Rules, ToolsHooks, WebQuality)
      not in any 4K spec
    - Slash command palette + Ctrl+. trigger
    - Full keyboard shortcut grid (Ctrl+N/W/D/. + Ctrl+Shift+H + Ctrl+1-5)
    - Autonomous-mode UI (gate-mode polling, autonomy strip,
      AutoAdvanceStats panel) — should be Phase 4G + 4K.6+
    - Overlay system with backdrop + window-selector + slash-palette
    - ProposalsPanel (Phase 4H proposal flow)
  Visual drift from dashboard-design.txt:
    - Body font: Inter (--font-ui), spec said JetBrains Mono throughout
    - Type scale +1-2px on every step (xs/sm/base/lg/xl 11/13/14/16/20
      vs spec 10/12/13/15/18) — UI looks ~25% larger than reference
    - bg-void #0b0d12 vs spec #0a0e12; text colors lighter
    - Two competing shells: Tier-0 Shell (sidebar+dual-PanelHost) is what
      the user lands on; the spec MVP Workspace exists but reachable only
      as the "PLAN window" inside Shell
    - Activity feed buried under autonomy strip + feasibility panel +
      violations strip + proposals panel + intervention cards
    - Activity feed missing: auto-scroll, Jump-to-Latest pill, inline
      [preview] entry with teal link
    - Status bar lives in Shell, not in Workspace (spec required it inside)
    - DiffViewer per-hunk hover controls and multi-file vertical nav thin
    - dashboard-wiring-plan.txt and dashboard-wiring-inventory.txt
      retroactively justify deferred features as "Tier 0-4" tracks —
      should NOT be treated as authoritative re-spec


================================================================================
PER-PHASE VERDICT TABLE
================================================================================

Sub-phase  Verdict           Wired   Critical gap
---------  ----------------  ------  -----------------------------------
pre-0      PARTIAL           N/A     metrics TOML missing; latent Win bug
                                     in orchestrator/src/hooks.rs:634 (sh)
0          PARTIAL           N/A     Dead code not stripped (compact.rs
                                     696, usage.rs 279); 84 HashMap imps;
                                     40+ entropy leaks; cli 22x over target
1A         IMPLEMENTED       WIRED   Box<dyn Error> in 1B/1C/1D scope
1B         IMPLEMENTED       WIRED   8 Box<dyn Error> debt
1C         PARTIAL           DEGRADED ort=rc; ONNX models not staged;
                                     code_vec populated but never queried
1D         MOSTLY IMPL       DEAD    DB path bug -> always empty result
1E         PARTIAL           DEGRADED 3 integration bugs; corpus empty;
                                     CatRAG overlay discarded
1F         PASS (DROP)       N/A     Decision correctly recorded
1G         PARTIAL           PARTIAL ms-marco model not staged;
                                     batching path test-only;
                                     .expect("semaphore closed") violation
1H         PARTIAL (PEND)    N/A     Models not staged; decision pending
1I         PARTIAL           PARTIAL SearchMode cascade not wired into
                                     query path; one .expect violation
2A         IMPLEMENTED       WIRED   moderate budget 800 vs CLAUDE.md 1000
2B         PASS              PARTIAL Sweep gate vacuously passes (no
                                     producer of ungrounded_claims)
2C         IMPLEMENTED       PARTIAL Orchestrator divergent 2nd table;
                                     no TOML config (loader is stub)
2D         IMPLEMENTED       UNWIRED CSQ never called before destructive
                                     ops; feasibility never enforced
3A         IMPLEMENTED       PARTIAL CliInteractiveGate dead code
3B         IMPL+DEVIATIONS   WIRED   74 action_types vs spec 17;
                                     deadpool not r2d2; no benchmark
3C         PARTIAL           PARTIAL SupervisionManager not wired;
                                     invisible walls leak hidden status;
                                     2C divergent table still live
3D         PASS              WIRED   apply_writes stub (gated to 3F/3J)
3E         IMPL with GAPS    HALF    file_overlap_score (3E.5) MISSING;
                                     ModelCapabilityProfile dead at runtime
3F         88% IMPL          WIRED   Benchmarks (3F.10) missing;
                                     seed rules SHADOW not ACTIVE
3G         PASS WITH CAVEATS WIRED   /plan command works; created_at field
                                     missing; tokenizer mock in tests
3H         DATA-LAYER OK     DEAD    SessionDag is parallel write-only;
                                     CLI uses JSONL; create_handoff/
                                     load_handoff/recover_stubs zero
                                     production callers
3I         PARTIAL           PARTIAL proptest-state-machine real but
                                     hardcoded 256 cases (spec 10K+);
                                     no madsim integration
3J         PARTIAL           SEMI    9/21 sub-steps in production;
                                     8 deferred; 4 of 6 WIRE.* cosmetic
3K         PASS              WIRED   AST chunking real; HyDE real;
                                     reranker model not staged;
                                     DEEP MRR < STANDARD MRR
4A         PARTIAL           COSMETIC Process exec yes; isolation NO;
                                     enable_sandbox=false everywhere
4B         PARTIAL           DUAL    Live path uses InProcessToolDispatcher;
                                     IpcToolRegistry dead; DAIRA 2 markers
                                     vs spec 15
4C         PARTIAL           PARTIAL InProcessHookDispatcher fires;
                                     IpcHookDispatcher dead;
                                     ShortCoder wired; ruff missing
4D         SUBSTANTIAL       WIRED   18 sub-steps real; verify_packages
                                     shape-only (no real registry HTTP);
                                     Designer agent abstraction bypassed
4E         IMPLEMENTED       WIRED   12 sub-steps; phase_4E.toml zeroed
                                     timings; tree metrics not persisted
4K MVP     SHIPPED           BURIED  98 Tauri cmds vs spec 6; Inter font
                                     not JetBrains Mono; Tier-0 Shell
                                     burying spec MVP Workspace; deferred
                                     4K.6-4K.13 features sneaking in


================================================================================
CROSS-CUTTING DEBT
================================================================================

ANTI-STALENESS RULE VIOLATIONS (CLAUDE.md hard rules):
  - ort = "=2.0.0-rc.12" (knowledge/Cargo.toml:27) — should be stable
  - Box<dyn std::error::Error> in 15+ public APIs across knowledge crate
  - .expect("semaphore closed") in reranker.rs:465 (production hot path)
  - 84 std::collections::HashMap imports (HashMap ban not enforced)
  - SystemTime::now / Instant::now in 17+ files outside core/clock.rs
  - Phase 0 dead code not stripped (5+ files, ~1500+ LOC)

ANTI-PATTERN: WRAPPER + ENGINE + TWO TABLES
  Reasoning crate has correct AgentGrcConfig table. Orchestrator has a
  divergent second table that loses semantics for some roles. PipelineGrc
  wrapper exists but is rarely called. Pattern repeats in:
    - Hook system (IpcHookDispatcher vs InProcessHookDispatcher)
    - Tool registry (IpcToolRegistry vs InProcessToolDispatcher)
    - Session persistence (SessionDag vs JSONL)
    - Sandbox (real Win32 code never invoked)
  Resolution: pick one path per concern, delete the other.

DECISION DOCUMENTATION GAPS:
  Per CLAUDE.md "Decision doc: before EXECUTING, write workflow/reasoning/
  phase-X-decisions.md", these phases lack the required doc:
    - Pre-phase-0, phase-0 (no decision docs found)
    - Phase 1H (decision still PENDING per workflow/reasoning/
      phase-1H-measurements.md L32)
  These phases HAVE the doc:
    - 1F, 3J.13 (deferrals doc)

GROUNDING GAPS (spec-flagged UNGROUNDED claims still ungrounded):
  - 3F COMPOSITE depth limit = 8 (predicate.rs)
  - 3H blake3 hashing for stub integrity
  - 3I madsim_compile_time_sec = 83.0 in metrics (fabricated; no madsim
    code = no madsim-cfg compile cost)


================================================================================
WHAT THIS MEANS FOR THE B-PLAN (next-step planning)
================================================================================

The system is NOT under-built. It is under-WIRED. The B-plan must therefore
prioritize:

  1. PATH UNIFICATION — single .elai_cc/ data dir; remove .elai/ literal
     fallbacks. Fixes 1A/1D/1E/3J.WIRE.STUBS in one stroke.

  2. WIRING OVER NEW FEATURES — every B-step must include a
     WIRING_CONTRACT block declaring producer/consumer file paths
     (Layer 0 from testing-trust-design.txt). No new types unless the
     consumer call site is also added in the same B-step.

  3. DELETE THE DUPLICATES — pick one Hook dispatcher, one Tool
     dispatcher, one Session persistor, one AgentGrcConfig table.
     Every duplicate is a future drift event waiting to happen.

  4. DASHBOARD ALIGNMENT — separate B-track for restoring the
     dashboard-design.txt visual identity (JetBrains Mono, 10/12/13/15/18
     scale, #0a0e12 bg, single Workspace shell, status bar inside)
     before adding any new feature work. Dashboard-wiring-plan.txt and
     dashboard-wiring-inventory.txt should be quarantined, not merged.

  5. ACTIVATE THE GATES — many gates exist (GroundingSweepGate,
     anti-rubber-stamp, CSQ, feasibility, SupervisionManager,
     CliInteractiveGate, SearchMode cascade) but vacuously pass or
     never fire. B-plan must include "producer wiring" steps that feed
     the gates real signals.

  6. BURN DOWN PHASE 0 DEBT — strip the dead code, kill HashMap, fix
     entropy leaks. This is small effort but unblocks anti-staleness
     enforcement layer-wide.

  7. INTEGRATION/WIRING IS THE SHIPPING UNIT — per the user's
     instruction, every B-N file must end with an INTEGRATION block
     listing: which existing producer this step wires up, which
     existing consumer surfaces it, and which dashboard panel/SSB
     action_type makes it visible.


================================================================================
CONFIDENCE & METHODOLOGY
================================================================================

Each per-sub-phase audit cites file:line evidence and elai.map matches.
Verdicts are conservative — when a symbol's presence cannot be verified,
the audit writes UNVERIFIED rather than guessing. The pattern density
above (10 cross-cutting failure patterns, each with multiple per-phase
hits) is high confidence. Two prior MVP-side audit docs in the project
(dashboard-wiring-plan.txt + dashboard-wiring-inventory.txt) report
similar findings with optimistic spin; the user's request to "not blindly
trust" them was correct — those docs retroactively rationalize the drift
documented here.

Total LOC of audit evidence: ~75,000 words across 23 reports.
Total elai.map symbol checks: ~600 grep operations.
Total source file Read operations: ~250.
