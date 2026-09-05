# ELAI_CC — Detailed Summary, Part 1: Overview and Architecture
Date: 2026-04-25
Successor to: F:\.ELAI_workflow\docs\specs\ELAI_COMPLETE_SUMMARY.txt (~790 lines)
Companion files:
  - DETAILED_SUMMARY_02_current_state.md (what's actually built and wired)
  - DETAILED_SUMMARY_03_gaps_and_roadmap.md (what's missing, B-plan rationale)
  - integration-summary.md, crates-summary.md, phase-1/2/3/4-summary.md (lossless agent outputs)

This file is the entry point. Read it first, then 02 for the current state,
then 03 for the gap analysis. The B-plan files (../B-plan/B-0 through B-N) are
the actionable sequence to finish the system.


================================================================================
1. WHAT ELAI IS (REVISED, GROUNDED)
================================================================================

elai_cc is a Rust-native development engine — not a chat assistant. A human gives
it a task. The system designs, builds, tests, and validates the solution
automatically. It runs on consumer hardware (12-48GB VRAM GPUs) using local
LLMs, with optional cloud API fallback for hard problems.

Two deliverables:
  - D1: standalone tool (current focus)
  - D2: Empire ecosystem integration (deferred to Phase 5)

CORE PHILOSOPHY (validated by 23 audits, qualified):
  "The intelligence is in the infrastructure, not the model."
  
  In the spec: a 3B parameter local model produces correct output because the
  system makes each task so small, so precisely defined, and so thoroughly
  verified that even a weak model succeeds — eventually.
  
  In practice (per audit): the infrastructure is mostly built. The wiring is
  mostly absent. Today's system is a library of intelligent components that
  do not run together end-to-end. The B-plan closes that gap.

PROJECT ROOT BOUNDARY (HARD RULE):
  F:\.ELAI_workflow\ = workflow/tooling root (CLAUDE.md, .claude/, memory-bank)
  F:\.ELAI_workflow\.elai_cc\ = the actual project being developed
  All code generation, file writes, edits, and builds target .elai_cc\.
  Spec files live at F:\.ELAI_workflow\docs\specs\.
  This file lives at F:\.ELAI_workflow\docs\thefinalglance\summary\.


================================================================================
2. THE 9-CRATE ARCHITECTURE (DEPENDENCY ORDER, LEAF TO ROOT)
================================================================================

All code lives in .elai_cc\crates\. The 9-crate workspace was set up in Phase 0.

WORKSPACE NOTE: today there are 16 members, not 9 — Phase 1+ added agent,
ipc-types, web_search, scaffold, web_validator, dashboard, contracts. This is
not a violation; it's natural Phase 4 growth.

  CORE  (no deps)
    Config parsing (elai_cc.toml), cross-platform paths, logging (tracing crate),
    serialization (msgpack for storage, postcard for wire), platform detection.

    ENTROPY QUARANTINE: all time/randomness via injectable traits
      (ClockProvider, EntropySource). Production uses real OS time/RNG.
      Testing uses seeded deterministic versions. Same seed = identical output
      across runs. Enables deterministic simulation testing later.
      
    CURRENT STATE: ClockProvider implemented in core/clock.rs. But 40+ leak
      sites across orchestrator/cli/tools/providers/plugins/rules/knowledge/
      web_validator/agent/dashboard still use std::time::SystemTime / Instant
      directly. Phase 0 acceptance "hits ONLY in core/src/clock.rs" violated.
      Burndown is B-15.

    HASHMAP BAN: std HashMap/HashSet banned (non-deterministic iteration order).
      Replaced by BTreeMap (sorted), IndexMap (insertion order), or FxHashMap
      (when order doesn't matter).
      
    CURRENT STATE: 84 std::collections::HashMap imports + 10 HashSet imports
      remain. Ban not enforced. Burndown is B-15.

    SandboxConfig lives in core/ (avoids circular dep with tools).

  PROVIDERS  (deps: core)
    LLM provider abstraction. Trait LlmProvider:
      complete(prompt, config) -> Stream
      embed(text) -> Vec<f32>
      token_count(text) -> usize
      token_count_estimate(text) -> usize  (sampling-based, 95% accurate, 10x cheaper)

    Implementations: Anthropic, OpenAI, Ollama, vLLM (Phase 2+), LlamaCpp.

    ModelPool: tracks which models are loaded on which GPUs, manages
    load/unload lifecycle, predictive pre-loading based on FSM state
    (if executing, pre-load the auditor model). SLO quality tiers per agent
    role (auditor=BEST, executor=GOOD, memory-writer=BASIC). GPU multiplexing
    for small models on spare VRAM.

    Tier 2 enforcement: two modes for structured output.
      GRAMMAR mode: grammar constraints at generation time (Gemma/Qwen).
      INSTRUCTOR mode: free generation + post-hoc JSON validation + retry
        (DeepSeek-R1 style, needs chain-of-thought).
      Auto-selects based on model metadata.

    Node-level cache with TTL and float normalization. Assistant prefill
    continuation for code generation (max 3 continuations when truncated).

    CURRENT STATE: VllmProvider exists (3J.1); ollama works (1F evaluation
    DROPPED speculative decoding, code present + gated); custom sidecar
    (python/elai_inference/server.py) is the production inference path,
    uses model.generate() + LogitsProcessor (NOT manual loop). DoLa
    LogitsProcessor + ETC LogitsProcessor present. ort = "=2.0.0-rc.12"
    violates CLAUDE.md anti-stale (B-16).

  KNOWLEDGE  (deps: core, providers)
    Codebase understanding engine. 7 layers, each adding capability:

    Layer 1 - DKG (Phase 1A):
      tree-sitter parses source code into a dependency graph stored in SQLite.
      Every function, struct, trait, import relationship mapped.
      Currently Rust-only; Python/TS/Go/Java/C# grammars pending (1A-MULTILANG).
      Concept annotations per node (one-line descriptions, regenerated when
      file changes — but concept_stale=1 always; no code populates concept).
      Effect tagging on functions (unsafe, filesystem, network, database, auth).

      CURRENT STATE: implemented; metrics phase_1A.toml shows real build of
      135 files / 8075 nodes / 4148 edges / 9.67s. WIRED via IndexingService.
      DUAL-PATH BUG: cli/main.rs:3752 + 5437 hardcode .elai/dkg.db, but
      IndexingService writes .elai_cc/indexes/dkg.db. CLI consumers see empty.

    Layer 2 - BM25 (Phase 1B):
      full-text keyword search via tantivy. Sub-millisecond queries.
      Every non-binary file indexed (tree-sitter failures fall back to
      line windows — but Phase 3K added AST-aware splitting at fn/impl/struct/
      class boundaries; 50-line is now fallback).

      CURRENT STATE: implemented + WIRED. ~5-10ms p99. 8 Box<dyn Error> debt.

    Layer 3 - Vectors (Phase 1C):
      dual embedding models per chunk.
      GraphCodeBERT for structural code similarity (768d).
      Granite for natural language similarity (384d).
      Both via ONNX Runtime. Cache in SQLite keyed by content hash.
      Immediate cache invalidation on file edit (prevents stale embeddings
      during same session).

      CURRENT STATE: implemented. ONNX models NOT staged on disk. Vector
      pipeline degrades to BM25-only at runtime when models absent.
      code_vec populated into cache but never queried at runtime
      (text_vec only). Effective 2-ranker RRF, not spec's 3-ranker.

    Layer 3.25 - SPLADE (Phase 1H):
      optional sparse keyword expansion. Catches semantic synonyms BM25 misses.
      Conditional on measured recall improvement (drop if <5% gain).
      
      CURRENT STATE: feature-gated (cfg(feature="splade")). Decision PENDING.
      Models not staged.

    Layer 3.5 - PageRank repo map (Phase 1D):
      ranks code symbols by importance using PageRank on the DKG graph.
      Personalized (task-mentioned symbol nodes boosted via build_personalization).
      Token-budget fitted via binary search. Sibling-of-aider's repomap, NOT a
      derivative: aider's implementation operates at the file level via NetworkX
      MultiDiGraph (verified against github.com/Aider-AI/aider/blob/main/aider/
      repomap.py on 2026-05-24 via C-38 fresh fetch; see workflow/sessions/
      s-25120/aider-elai-pagerank-diff.md). ELAI's compute_repo_map operates at
      the symbol level via a hand-rolled dense adjacency matrix + power iteration
      in Rust. Parity points (4): self-loop weight 0.1 for orphan defs, damping
      factor 0.85, max_iterations 100, tolerance 1e-6, active-file edge multiplier
      x50, dangling-node redistribution via personalization. Divergences (4):
      symbol-level (not file-level) node identity; flat edge weights (no per-
      identifier multipliers — Aider's "leading underscore x0.1" / "multi-definer
      x0.1" heuristics are unsupported by any cited measurement and ELAI omits
      them deliberately); density guard short-circuits PageRank when graph density
      > 0.3 (Aider always runs full iteration); single compute_repo_map entry
      point (Aider has three: get_repo_map / get_ranked_tags_map / get_ranked_tags).

      CURRENT STATE (post-C-38, 2026-05-24): implemented + 24 unit tests + the
      C-38 production contract test at crates/contracts/tests/
      c38_pagerank_production_contract.rs. compute_repo_map fires from
      QueryPipeline::query_with_tier Stage 2 at crates/knowledge/src/pipeline.rs:
      610. The prior "DEAD AT RUNTIME" diagnosis — that pipeline.dkg_engine was
      gated on a never-fires .elai/dkg.db existence check — was located in the
      CLI at crates/cli/src/live_cli.rs:1722-1732, NOT in pipeline.rs as a 2026-
      05-17 reading reported. C-38 replaces that gate with
      crate::dkg_lifecycle::ensure_dkg_or_rebuild, which rebuilds the DKG when
      the on-disk file is missing or stale (default 24h window) and emits paired
      dkg_rebuild_started / dkg_rebuild_complete SSB beacons around the rebuild
      path. The contract test locks four claims: compute_repo_map is invoked on
      the STANDARD/DEEP query path, the result is non-empty for a non-empty
      codebase, an SsbActionType::PagerankRun row fires per query, and the
      DkgRebuildStarted/Complete events bracket a rebuild.

    Layer 4 - CatRAG (Phase 1E):
      query-aware graph traversal. Evidence chains from DKG.
      
      CURRENT STATE: implemented. Overlay computed at pipeline.rs:433 but
      bound to _overlay and discarded. CatRAG stage pushes empty results
      unconditionally at runtime.

    Layer 5 - A-RAG (Phase 1E):
      hierarchical retrieval. 55-87% token savings on context retrieval.
      
      CURRENT STATE: implemented. AragOrchestrator.add_chunk has zero callers
      in CLI — corpus is empty at runtime. Methods return Vec::new().

    Cross-encoder reranker after A-RAG (Phase 1G):
      ms-marco-MiniLM-L12-v2, ~120MB ONNX fp32, CPU-only, <50ms target.
      tokio::Semaphore(1) + bounded mpsc channel + request batching.
      Circuit breaker with L-6 fallback model tier.
      
      CURRENT STATE: implemented. ONNX model NOT staged. Reranker degrades
      to RRF-only at runtime. Batching path test-only (no production spawn).
      .expect("semaphore closed") at reranker.rs:465 violates CLAUDE.md.

    Resilience layer (Phase 1I):
      Circuit breaker with failure classification (transient/permanent),
      exponential backoff. SearchMode cascade
      (HYBRID -> SPARSE_ONLY -> BM25_ONLY) with auto-recovery. Query
      validator (empty, >10K chars, null bytes rejected). Startup readiness
      (READY/DEGRADED/UNAVAILABLE) with canary drift check. QueryTrace +
      StageTrace for pipeline observability. Index version schema + migration.
      
      CURRENT STATE: components implemented. SearchMode cascade NOT WIRED to
      query path (only one startup-time call site). QueryPipeline doesn't
      hold or consult a DegradationCascade. pipeline.rs:520 hardcodes
      SearchMode::Hybrid in QueryTrace.

    Production hardening (Phase 3K):
      Adaptive AST-aware chunking (3K.1).
      Template-based query expansion (3K.2, no LLM).
      HyDE hypothetical answer embedding (3K.3, complex queries only,
        wired to sidecar via SidecarHydeClient).
      Query tier routing FAST / STANDARD / DEEP (3K.4) — auto-detect from
        query characteristics. Distinct from 1I SearchMode (degradation
        vs intentional tier selection).
      Retrieval quality benchmark: 30 queries, R@5/R@10/MRR/latency per tier.
      
      CURRENT STATE: implemented + WIRED. Real measured numbers in
      metrics/phase_3K_rag.toml (R@5=0.6333, R@10=0.7667, MRR=0.4117).
      DEEP MRR (0.206) BELOW STANDARD MRR (0.575) — violates spec
      monotonicity. Cross-encoder model not staged so reranker degrades.

    Full query pipeline:
      query -> BM25+SPLADE -> dual-vector -> PageRank -> CatRAG -> A-RAG ->
      cross-encoder rerank -> token-budget fit -> context.
    
    Per-project isolation (project A's index never contaminates project B).
    
    BOX<DYN ERROR> DEBT: 15+ public APIs across knowledge crate use
    Box<dyn std::error::Error> (8 in bm25.rs, 5 in vector/indexer.rs, 1 in
    pagerank/mod.rs, 1 in 1A scope). Violates CLAUDE.md (B-16).

  REASONING  (deps: core, providers, tools)
    Grounded Reasoning Chain (GRC) engine. All reasoning must cite sources.
    
    State machine per topic:
      observe -> hypothesize -> plan_action -> act -> verify (+ status echo, converge)
    
    Budget system: per-topic call and character limits. Overflow forces
    convergence. Moderate default: 5 calls / 800 chars (NOTE: CLAUDE.md says
    1000; reasoning crate uses 800 — a calibration discrepancy).
    
    Grounding verification (Phase 2B) — 6 methods, each binary pass/fail:
      FILE_CHECK    — implemented
      SOURCE_CHECK  — implemented
      TEST_CHECK    — implemented
      WEB_CHECK     — STUB returns typed error (Phase 4A IPC proxy)
      API_CHECK     — STUB returns typed error
      API_REGISTRY_CHECK — STUB returns typed error (uses string-shape only,
                          not real registry HTTP, in Phase 4D verify_packages)
    
    Plan grounding sweep: scans ALL claims in the plan. Blocks execution if
    any claim is ungrounded.
    
    Per-agent GRC configuration (Phase 2C):
      executor (8/1200, full chain)
      auditor (10/1200, double-verify, anti-rubber-stamp: must have >=1
               FINDING: before Converge or rejected)
      context-compiler (3/600, observe-only, FILE_CHECK+SOURCE_CHECK only)
      memory-writer (2/400)
      prompt-engineer (5/800, [Observe, Hypothesize, Converge])
      designer (8/1200, distinct from executor)
    
    Counterfactual Safety Questions (CSQ, Phase 2D):
      Before finalizing non-trivial logic, asks "if assumption X is wrong,
      code will Y" and requires guards for dangerous consequences.
      Mandatory for functions touching files, network, money, auth.
      CSQ severity: CRITICAL (data_loss/security/financial/auth — guard mandatory)
      vs WARNING. Critical without guard = blocks progression.
      
    Inline feasibility check during execution: max 2 quick checks per phase
    for novel claims not in the plan. Outcomes: Pass / Fail / Blocked.
    Blocked emits ReplanRequest (orchestrator emits, not reasoning).
    
    CURRENT STATE: all 4 sub-phases (2A-2D) implemented in reasoning crate.
    PipelineGrc wrapper at orchestrator/src/grc.rs exists. ZERO production
    callers in task_runner / fsm / dispatcher / hooks. CSQ never enforced
    pre-write; feasibility never enforced; sweep gate vacuously passes
    (nothing populates ungrounded_claims). AgentGrcConfig divergent in
    orchestrator/src/agent_pool.rs:467 — ContextCompiler/MediaValidator lose
    observe-only enforcement.

  RULES  (deps: core)
    Self-improving deterministic behavior enforcement. Rules are DATA in
    SQLite, not markdown files. Evaluated by Rust code, not by the LLM.
    
    7 predicate types (spec said 5-6; current is 7 with VERSION_GATE added):
      AST_PATTERN          — tree-sitter queries
      GREP_PATTERN         — regex (extended with optional language + file_context fields)
      SCHEMA_REQUIRED      — JSON schema validation
      METRIC_GATE          — numeric thresholds
      COMPOSITE            — AND/OR/NOT combinations (depth=8 hardcoded ungrounded)
      GRAPH_REACHABILITY   — DKG path queries (auth chokepoints, layer isolation)
      VERSION_GATE         — anti-staleness deprecated_patterns
    
    Three enforcement tiers:
      Tier 1 - Deterministic (<1ms): code physically cannot violate. Model
        never sees these rules. PostToolUse hooks evaluate predicates on output.
      Tier 2 - Constrained decoding: structurally forced output format.
      Tier 3 - Prompt guidance: 3-10 semantic rules, ~100-300 tokens.
    
    Rule lifecycle: PROPOSED -> CANDIDATE -> SHADOW -> ACTIVE -> RETIRED.
    All transitions are pure arithmetic (FPR, TPR, CCR). Zero LLM in promotion.
    
    Self-discovery: when auditor catches the same violation pattern 3+ times,
    the system auto-proposes a new rule. Proposed rules need independent
    validation.
    
    Counterfactual shadow testing (CEL-derived): tests whether a SHADOW rule
    actually CAUSES better outcomes, or just catches what the auditor already
    catches. EDCT: tests whether the rule reacts to the actual violation or
    a spurious correlated feature.
    
    Time-aware confidence decay: rules with no true positives in 90 days lose
    confidence. Below 0.25 = auto-deprecated.
    Safety/security rules (mutable=0) can never be auto-adjusted or auto-retired.
    
    ~39 seed rules across 7 domains (engineering 12 + web 27).
    21 deprecated_patterns seed entries.
    
    CURRENT STATE: 9/10 sub-steps PASS (3F.1-3F.9). 3F.10 benchmarks missing.
    Seed rules inserted with status=SHADOW, not ACTIVE per spec — divergence.
    Hooks call RuleEngine via T1Hook in plugins crate (WIRED).

  TOOLS  (deps: core, knowledge)
    File operations (read with outline mode for large files >300 lines, write,
    edit with 3-tier matching: exact -> whitespace-normalized -> ellipsis-elision).
    
    Bash execution: sandboxed, timeout-controlled, isolated per agent.
    Phase 0-1 uses chroot; Phase 3+ uses Firecracker/gVisor microVMs.
    
    Git integration via git2: auto-commit after verified phase completion,
    branch per plan, session-owned commit tracking with safe undo protocol.
    Phase-level commits (not per-edit). WIP commits squashed after audit success.
    
    Validation pipeline (5 tiers): determinism, quality/AST, unit tests,
    integration tests, stress tests. All run by auditor — executor never
    runs own tests.
    
    DAIRA trace execution: structured runtime tracing with markers for
    call/return/mutation/branch/error/finding. Used in GRC observe step
    for behavior analysis. Spec said 15 markers; current impl has 2
    ("enter"/"exit") — degradation.
    
    ShortCoder: 10 AST-preserving simplification rules R1-R10
    (collapse bodies to signatures, strip comments, elide obvious types,
    collapse imports, truncate strings, remove empty lines, strip decorators,
    trim trailing whitespace, etc.). 18-38% token savings. Applied as
    PostToolUse on executor output before auditor review.
    
    Failure taxonomy: deterministic regex-based error classification.
    Spec said 9 categories; current impl has 13 with mapping mismatches.
    Auditor reads classification BEFORE raw output (saves 200-500 tokens).
    
    Visibility filter: per-agent path allowlist. Hidden paths return
    "not found" — but current impl returns error_code="visibility_filtered"
    + message "visibility_filtered: path is hidden" which VIOLATES the
    spec's "indistinguishable from genuinely missing" requirement.
    Tool schema sanitization: internal parameters stripped before model sees.
    
    Incremental compilation feedback (Phase 4B): after each file write
    during execution, runs language-specific build check asynchronously.
    Errors fed back to executor for immediate retry (max 3 rounds).
    Not applied during test-writing phase.
    
    IntegrationTestRunner with 4 levels (4B.6-4B.7):
      BABY_STEP (cargo check, <5s)
      STEP (cargo test module, <30s)
      PHASE (cargo test -p crate, <60s)
      CONCEPT (cargo test --workspace, <120s)
    Triggered by orchestrator after baby step acceptance.
    
    CURRENT STATE: most components present. ShortCoder lives in knowledge
    crate (not tools — minor scope drift, functional). IpcToolRegistry +
    Handler types in tools/src/ are DEAD CODE — orchestrator wires the
    parallel InProcessToolDispatcher (elai_core::ToolDispatcher) instead.
    DAIRA reduced to 2/15 markers. Failure taxonomy expanded to 13/9.
    Edit matching SequenceMatcher similarity hint missing on failure.

  PLUGINS  (deps: core, rules)
    Hook system: PreToolUse, PostToolUse, Stop, PreCompact.
    Under process isolation (Phase 4+), hooks intercept IPC messages in the
    orchestrator before forwarding to agents. Hook verdicts (ALLOW/BLOCK/WARN)
    never leak to agents — BLOCK produces a generic error message ("tool call
    rejected by policy"). Findings feed the rules self-discovery mechanism.
    
    Tiered static analysis:
      T1 (predicate DSL from rules.db, <1ms)
      T2 (tree-sitter AST parse, <10ms for <5k lines)
      T3 (pattern-match, no LLM, <5ms, fires only on WARN)
    No LLM in the validation hot path.
    
    MCP bridge: connects to external MCP servers for tool access.
      DefaultMcpBridge::connect/list_tools/call_tool are placeholders.
      batch-ful-d added a real HTTP JSON-RPC 2.0 client (http_initialize,
      http_list_tools, http_call_tool with deny-list + sanitization at L274-400).
      Consumed by dashboard only. CLI does not use MCP bridge.
    
    CURRENT STATE: TWO parallel hook systems exist:
      Spec-compliant IpcHookDispatcher (plugins/src/ipc_hooks.rs) — ZERO
        non-test callers. IpcGateway::run_pre_tool_use_hook /
        run_post_tool_use_hook are dead helpers. Doc-comment admits
        "Production wiring scheduled in Session wire-4A".
      Live InProcessHookDispatcher (orchestrator/src/hook_dispatcher.rs)
        fires from task_runner.rs at baby-step boundaries — NOT IPC frames.
        PreToolUse runs against the natural-language task_summary at
        task_runner.rs:3338. PostToolUse runs per-changed-file at 3589
        with no ruff. Stop fires at 3877 as telemetry only — does NOT
        enforce non-IDLE block. PreCompact at 3898 is an explicit MVP no-op.
    Pick one path; delete the other (B-19).

  ORCHESTRATOR  (deps: core, providers, reasoning, rules, tools)
    The largest and most complex crate. Controls everything.
    
    FSM engine (Phase 3A): 9 states (IDLE, PLANNING, FEASIBILITY_PROBE,
    TESTING_FIRST, EXECUTING, AUDITING, MEMORIZING, COMPLETE, CONFIGURING).
    Plus additive RESEARCHING (Oracle mode). Transition validation matrix
    + 7 evidence gates. Phase-aware tool/rule loading per state.
    
    SSB - Shared State Board (Phase 3B): inter-agent communication via
    SQLite WAL. Single-writer guarantee via tokio::mpsc. broadcast for
    event notification (Arc<[u8]> payload). 17 spec action_types; current
    impl has 74 (forward-compat extensions for batches 1A/1B/1C/2A/2B/3A/
    3B-4G/3C-4H/4C/ful-a/ful-c/ful-d/CHAT/Settings). Append-only enforced
    at type level + runtime update_hook. ClockProvider for timestamps.
    Conflict detection on file paths across sessions. ~50K inserts/sec
    ceiling NEVER MEASURED (no benchmark exists).
    
    BSP supersteps (Phase 3D): deterministic macro-level orchestration.
    4 channel types: LastValue, Topic (accumulate option), EphemeralValue,
    UntrackedValue (DeepSizeOf). BinaryOp intentionally dropped.
    Snapshot -> clear topics -> execute -> apply writes -> clear ephemeral
    -> evidence gate -> checkpoint. Parallel agent execution via
    tokio::JoinSet with VRAM-aware scheduling.
    
    Context budget allocator (Phase 3E): 8 fixed slots:
      SystemPrompt, Rules, PhaseSpec, RepoMap, CodeContext, SsbEntries,
      GrcHistory, OutputReserve.
    Dynamic slots (4-7) distributed by percentage (15/40/25/20).
    Floors: SsbEntries>=200, GrcHistory>=500, OutputReserve>=2000.
    2 cache breakpoints aligned with provider prefix caching.
    SSB entry filtering by file_overlap_score = |intersection|/|agent.files|.
    3-level adaptive reduction (FilteredByRelevance -> ReducedGranularity ->
    ReducedScope -> Terminal).
    Auditor gets DKG blast radius subgraph injected into code context slot.
    Assembly latency target: <50ms.
    
    CURRENT STATE: 6/7 sub-steps implemented. file_overlap_score (3E.5)
    MISSING entirely. ModelCapabilityProfile machinery built but
    task_runner.rs:2992 falls back to zeroed [0,0,0] stub at 5 sites.
    REPL path at cli/main.rs:7680-7727 sets all three slots from real
    measurements.
    
    Agent system (Phase 3C): spawning, lifecycle, checkpointing
    (5 msgpack files per agent), crash recovery (restart from last
    checkpoint), Erlang-derived supervision tree (max 3 restarts per phase),
    anti-spiral detection (3x same step = escalate). 4 death classifications:
    clean exit, crash, timeout, budget exceeded. 8 AgentRoles
    (Executor, Auditor, Researcher, Designer, PromptEngineer,
    ContextCompiler, MemoryWriter, MediaValidator). blake3 (per CLAUDE.md,
    deviating from spec which said blake2s). SupervisionManager exists but
    NEVER wired into task_runner.
    
    Structured reasoning trace: every agent action produces a
    machine-readable record in SSB. Enables post-mortem debugging,
    compliance audit, self-modification fuel, and cognitive isolation metrics.
    
    Session handoff (Phase 3H): DAG-based state versioning (CMV-derived).
    Structurally lossless trimming: large tool results replaced with stubs
    (recoverable from SSB). User and assistant messages never stubbed.
    Branch-and-explore (max 3 active branches). blake3 integrity verification.
    Trigger: 85% token budget.
    
    CURRENT STATE: SessionDag is 3090-LOC parallel write-only ledger.
    CLI uses legacy JSONL. create_handoff/load_handoff/recover_stubs/
    merge_branch/close_branch zero non-test callers. /session fork creates
    branches via create_branch; merge/close UNREACHABLE from CLI or UI.
    CLI hardcodes stub_ssb_seq: None (so Pass 1 produces zero stubs).
    
    Hierarchical plan management (Phase 3G): two-artifact representation.
    PlanSkeleton (~2-4K tokens, always in context) + per-phase detail
    (loaded on demand). Contract extraction before elaboration.
    Coherence enforcement by auditor. Kahn's topological sort + cycle detection.
    
    CURRENT STATE: 7/7 sub-steps PASS. /plan, /plan list, /plan <phase>
    work in CLI. Used in 6 budget call sites. Tokenizer real-impl deferred.
    
    Project isolation: per-project directory with its own SSB, knowledge DB,
    rules DB, sessions. One project on GPU at a time.
    
    Agent isolation model (3 tiers, incremental — Phase 4A):
      Tier A — Invisible walls (Phase 0+): filesystem visibility filter.
        CURRENT: error_code="visibility_filtered" leaks; per-agent
        allowed_paths NOT implemented (global HIDDEN_PATH_PATTERNS only).
      Tier B — Sandbox execution (Phase 3+): Bash commands in
        Firecracker/gVisor microVMs.
        CURRENT: not implemented.
      Tier C — Process separation (Phase 4+): each agent is a separate OS
        process. Agent binary has NO LLM client, NO network, NO tools.
        Everything proxied via IPC. Wire format: 1+4+postcard. 15 message
        types. GRC budget enforced by orchestrator externally.
        CURRENT: separate process exec YES (dashboard spawns elai_cc_agent
        via plain std::process::Command::new(...).spawn()). NO isolation
        primitives. WindowsAppContainerSandbox real code but enable_sandbox
        defaults false everywhere. IpcGateway types defined; NO
        UnixListener/NamedPipeServer accept loop in orchestrator.
        Effective: A++ (in-process logic + separate process exec, but
        shared namespaces / syscalls / network / fs / proc / limits).
    
    Evidence gate protocol: every FSM transition requires artifacts, not
    self-assessment.
      PLANNING -> TESTING_FIRST: zero ungrounded claims.
      TESTING_FIRST -> EXECUTING: all tests fail for right reasons.
      EXECUTING -> AUDITING: at least 1 test flipped from fail to pass.
      AUDITING -> MEMORIZING: zero BLOCK findings remaining.
    
    CURRENT STATE: 7 evidence gates registered. Most pass vacuously because
    nothing populates ungrounded_claims / etc.
    
    Deterministic dispatch protocol: two dispatch tables (meta for
    PLANNING/CONFIGURING, plan-based for execution states). Prompt assembly
    from 4 sources: role prompt (~80 tokens) + tier 3 rules (~200 tokens) +
    code context + task spec. Total meta-instructions ~330 tokens per turn
    (vs 1600-2600 in prompt-based workflows).
    
    ReplanRequest: executor can signal the plan is fundamentally wrong
    (MINOR = fast track replan, MAJOR = requires human approval).
    
    Phase completion protocol: atomic durability marker (phase_complete in
    SSB) written before model unload. Crash recovery semantics fully
    specified per failure point.

  CLI  (deps: core, orchestrator, tools, providers)
    Thin REPL shell. Slash-command routing:
      /task (normal workflow)
      /config (edit config)
      /workflow (edit tooling)
      /fix (debugging)
      /continue (resume handoff)
      /explore (read-only query)
      /status (current state)
      /plan, /plan list, /plan <phase>
      /research, /ask
    Fallback classifier for bare messages. Extensible with custom commands
    from .elai_cc/commands.toml.
    Mode switching: INTERACTIVE (default) <-> AUTONOMOUS (Phase 4G+).
    
    CURRENT STATE: cli/src/main.rs is 11,181 LOC vs <500 LOC spec target
    (22x over). Spec said cli should be a thin shell; currently does heavy
    orchestration directly. CliInteractiveGate at cli/src/main.rs:7354 —
    defined, never instantiated. Fsm::transition() called directly throughout
    production, bypassing the interactive protocol.

  AGENT (Phase 4A addition, separate binary elai_cc_agent)
    NO LLM client, NO network, NO tools. All proxied via IPC.
    Cargo.toml only serde + postcard + tracing + uuid + elai-ipc-types.
    Confirmed minimal IPC client.

  IPC-TYPES (Phase 4A addition)
    1-byte version + 4-byte length + postcard payload.
    15 message types. MAX_FRAME_SIZE=10MB.
    8 AgentRoles enum.
    Used by orchestrator IpcGateway and agent binary.

  WEB-SEARCH (Phase 4D addition)
    SearxClient (client.rs:147), WebFetcher (fetcher.rs:35), ContentExtractor
    + BasicExtractor (extractor.rs:145), SourceQualityTable + SourceTier
    (source_quality.rs), RobotsCache (robots.rs:64), RateLimiter with
    injectable mock clock. Constants match spec exactly (10s timeout, 20 max
    results, 1000ms rate limit). Docker script start-searxng.sh present.

  SCAFFOLD (Phase 4D addition)
    Used in web_research_orchestrator and web validation paths.

  WEB-VALIDATOR (Phase 4D addition)
    Validates web research outputs.

  DASHBOARD (Phase 4K)
    Tauri v2 backend (Rust) + Svelte 5 frontend.
    98 Tauri commands, 9 windows (Chat, Activity, Explore, Monitor, Plan,
    Preview, Research, Review, Settings), 10 drawers (AntiStaleness,
    Knowledge, Orchestration, ProjectIntelligence, ProviderVram, Reasoning,
    Research, Rules, ToolsHooks, WebQuality).
    SSB read-only viewer library extracts dashboard data from SQLite WAL.
    
    CURRENT STATE: ~11% of backend signals surfaced. ~30 DEAD UI elements
    (display X but X has no producer). ~232 INVISIBLE backend signals
    (producer exists, no consumer). Visual identity drifted from spec
    (Inter font instead of JetBrains Mono; type scale +1-2px; bg #0b0d12
    vs #0a0e12; Tier-0 Shell burying spec MVP Workspace).

  CONTRACTS (test crate from Phase 0.6)
    Cross-crate contract tests. End-to-end integration test fixtures.
    Includes anti_spiral_runner_contract, task_runner_rule_eval_contract,
    shortcoder_post_tool_use_invocation_contract, etc.


================================================================================
3. THE PIPELINE FACTORY (HOW TASKS BECOME PIPELINES)
================================================================================

Three-layer architecture for turning any user goal into an executable pipeline.

L1 — PIPELINE DESIGNER (Phase 4D, LLM-driven, grounded):
  A distinct Designer agent (not the executor) receives the user's task and
  produces a PipelineSpec. The designer uses a Research Agent for deep
  research via a tree-structured research engine (FlashResearch-derived).
  
  Research engine: not a single web search. Tree of sub-questions with
  parallel expansion, speculative execution (cloud only), dual-threshold
  convergence (goal satisfaction + quality). Source quality tiered
  deterministically (arxiv/official docs = tier 1, GitHub/StackOverflow =
  tier 2, blogs = tier 3, spam = tier 4). Contradictions surfaced, never
  silently resolved.
  
  Designer receives SystemConstraints (model context budget, available
  tokens, average file sizes) so it can size phases to fit the model's
  capabilities.
  
  Recursive Decomposition (4D.7-4D.10): Idea -> Concepts -> Phases -> Steps
  -> BabySteps. Each tree level triggers grounded FlashResearch (not just
  top-level). New types: ConceptDecomposition, ConceptNode, PhaseNode,
  StepNode, BabyStepSpec (8 blocks: GROUNDING, PRE_GATE, TASK, TDD,
  DETERMINISTIC_CHECK, ACCEPTANCE, INTEGRATION, EXIT_CRITERIA).
  TechStackDecision with DecisionSource (RESEARCH/EXISTING_PROJECT/
  USER_SPECIFIED/FALLBACK). PackageRef with API_REGISTRY_CHECK
  (verify_packages). ResearchCache: LRU cache.
  
  Web Research (4D.11-4D.18): SearXNG via Docker, RobotsCache, RateLimiter,
  ContentExtractor. WebResearchOrchestrator with FORMULATE/SEARCH/EXTRACT/
  REFLECT/CONVERGE loop.
  
  CURRENT STATE: 18 sub-steps implemented. Designer agent abstraction
  partially bypassed at production call site. verify_packages is shape-only
  (not real registry HTTP).

L2 — PIPELINE BUILDER (Phase 4E, deterministic, no LLM):
  Takes PipelineSpec, validates components exist, creates FSM states,
  configures agents, estimates resources, generates dependency graph with
  Kahn's cycle detection. Phase Sizing: splits phases that exceed the
  model's available context (split by contract boundaries, max depth 2).
  Produces ExecutablePipeline.
  
  DAG repair (AutoB2G-derived): Reorder for cycles, Substitute for missing
  tools. Max 10 repair rounds.
  
  DecompositionTree + TreeNavigator (4E.7-4E.12):
    DecompositionTree is the primary execution artifact — typed Rust struct,
    not prompts.
    TreeNavigator: deterministic depth-first traversal through baby steps.
    Baby step execution cycle (8-step per baby step):
      context assembly -> pre_gate -> TESTING_FIRST -> EXECUTING -> AUDITING
      -> acceptance -> integration -> exit criteria.
    BubbleUpHandler: failure cascades up tree levels with configurable retry
    thresholds (BabyStep=3, Step=2, Phase=1, Concept=1).
    Actions: ResearchAlternative, RedecomposeStep, ReplanPhase, ReplanConcept,
    Escalate. Plus DiagnoseAndRetry (additive 6th).
    Integration checkpoints at 4 levels: BABY_STEP/STEP/PHASE/CONCEPT.
  
  CURRENT STATE: 12/12 sub-steps implemented + WIRED. ExecutablePipeline
  drives task_runner.rs:3143, 3176-3550. DAG repair fires on validation
  fail (T_DAG=10). BubbleUpHandler triggers in production.

L3 — PIPELINE EXECUTOR (existing FSM + agents):
  Runs the ExecutablePipeline through the FSM in INTERACTIVE or AUTONOMOUS
  mode. INTERACTIVE = pause + evidence + human APPROVE/REJECT/MODIFY/REDIRECT.
  AUTONOMOUS = auto-advance with evidence checks (Phase 4G).

QualitySpec: defines measurable quality per pipeline. QualityMetric (name,
target, unit, method, phase, hard_fail). MediaSpec for visual outputs.
PerfSpec for performance. L2 validates every metric has an automated check method.

Pipeline templates: completed pipelines become reusable. Stored with outcomes.
Ranked by first-try ratio, coverage, block ratio, token efficiency. Minimum
3 executions before ranking applies.


================================================================================
4. PROMPT ARCHITECTURE (HOW THE MODEL IS KEPT SIMPLE)
================================================================================

The core insight: in elai_cc, the Rust binary controls the workflow. The
model is a worker that receives a structured prompt, produces output, and
the code validates it.

THE MODEL NEVER:
  decides FSM transitions
  spawns agents
  loads rules
  manages sessions
  decides what context to load
  calls RAG tools
  accesses the network (except researcher)
  sees enforcement logic

Current prompt-based workflow costs: ~1600-2600 tokens of meta-instructions
per turn. elai_cc cost: ~330 tokens per turn (role prompt + tier 3 rules +
output format). Savings: 1300-2300 tokens freed for actual code context.
On 8K context: 16-29% more for code. Critical for local models.

Additional savings from process isolation (Tier C): agent reasoning chains
execute in separate processes and never enter the orchestrator's context
window. If agents generate 12K tokens internally, isolation saves 150% of
context window — makes complex tasks possible where in-process execution
would overflow.

Every current .md file in the workflow gets replaced by Rust code:
  session-protocol.md     -> orchestrator::session_init()
  multi-session.md        -> SSB + session locks (automatic)
  conditional-routing.md  -> rules::domain_detection() (deterministic file-path-to-domain)
  output-formatting.md    -> role_prompt (1 sentence) + MEMORIZING phase code
  planning-workflow SKILL -> orchestrator FSM engine
  prompt-transform SKILL  -> orchestrator::prompt_transform() (3-line function)
  rag-tools SKILL         -> knowledge::KnowledgeEngine (internal)
  CLAUDE.md               -> elai_cc.toml + Rust code
  Agent .md files         -> role_prompt in elai_cc.toml (50-100 tokens each)


================================================================================
5. SELF-MODIFICATION SYSTEM (PHASE 4H, NOT YET STARTED)
================================================================================

Core safety property: the modifying agent NEVER evaluates its own modifications.
Deterministic benchmarks judge. The agent proposes, code decides.

Three layers:
  Layer 1 - Data (automatic, no approval): template rankings, rule trigger
    counts, pipeline outcome metrics. Pure arithmetic. Security/safety rules
    never touched.
  Layer 1A - Data-table growth (no LLM, no staged eval): new rows in
    deprecated_patterns, FailureTaxonomy, lazy-code regex, source-tiers.
    FPR validation. DarwinNet Gene Pool (section IV-A2) for successful pattern
    reuse.
  Layer 2 - Config (full pipeline, auditor approval): new rules, GRC budgets,
    context ratios, model assignments. Log-driven proposals (reads SSB traces).
    Config archive (DGM sigmoid-weighted parent selection, max 50 variants).
    Staged evaluation (10/50/200 tasks, early termination on failure).
    Proof gate (soundness check against canonical test traces).
    Slot-restricted patches (max 1 config slot per proposal).
    Anti-reward-hacking: metrics code, gate definitions, auditor config, test
    suite all READ-LOCKED during self-modification.
  Layer 3 - Structure (human approval, always): FSM state additions, new agent
    roles, GRC protocol changes.

Modification Solidification Index (MSI): exponential-decay weighted moving
avg (lambda=0.3) of delta_score over N=10. Per-PipelineMode (SURGICAL/
GREENFIELD tracked independently).
  IMPROVING (>0.3) — continue
  STABILIZING (-0.1 to 0.3) — reduce frequency
  DIVERGING (<=-0.1) — pause and alert user
Auto-revert on >10% degradation over 3 pipelines. Trigger task detects
objective hacking (DGM Appendix H).

AllowedConfigKey enum: closed set of mutable keys. Locked keys (cloud budget,
gate definitions, score weights) never modifiable. Adding new keys requires
Layer 3 human approval.

Safety: 4 enforcement layers (dispatch + IPC + filesystem + git). Self-mod
agent cannot read lock list. Cannot modify locked config keys.

Anti-staleness wiring: triggers nightly dep audit, Layer 1A grows
deprecated_patterns, triggers steering vector refresh on deprecated_patterns
change. Reasoning wiring: failure_taxonomy_misses -> Layer 1A -> new
FailureTaxonomy categories.

CURRENT STATE: NOT STARTED.


================================================================================
6. MEDIA VALIDATOR (PHASE 4I, CONDITIONAL, NOT STARTED)
================================================================================

Validates non-code outputs (images, 3D models, audio, UI screenshots).
Only loaded when PipelineSpec contains media-producing steps.

Tier 1 - Deterministic (<100ms): format, resolution, SSIM/LPIPS, vertex counts,
sample rates. External tools: ImageMagick, Blender CLI, FFmpeg, Tesseract.
SSIM/LPIPS via ONNX Runtime CPU EP only (GPU non-deterministic).

Tier 2 - VLM qualitative (~2-5s): advisory only, Tier 1 has veto power.
Mode A for single deep image analysis (Qwen3.5-VL-30B-A3B, ~17GB Q4).
Mode B for sequential image flows (InternVL3.5-38B via SGLang, ~38GB 8-bit).
Constrained decoding enforces VlmValidationOutput JSON schema.

CLAP for audio-text similarity (~200M params, ~0.4GB FP16, always-loaded
when audio active).

Render-Capture-Evaluate architecture: MediaRenderAdapter trait abstracts
"render output to image(s)." Adapters: Playwright for web, Blender CLI for
3D, FFmpeg for animation, direct file read for screenshots, ComfyUI for
AI-generated images. Visual feedback loop: execute -> render -> VLM
evaluates -> structured feedback -> executor iterates. Max iterations
before escalation. Convergence check stops when issues unchanged between
iterations.

Veto logic: Tier1 fail -> Fail. Tier1 pass + Tier2 disagree -> HumanReview.
VLM unavailable -> Degraded.
Bounded concurrency: max 4 Tier 1, max 1 Tier 2, queue depth 64.

CURRENT STATE: NOT STARTED.


================================================================================
7. EMPIRE INTEGRATION (D2, PHASE 4J, DEFERRED)
================================================================================

Empire is an existing engine hierarchy (MKI/MKII/MKIII) with IPC dispatch,
hooks, buses, HTTP, events, persistence, WebSocket. Two bus layers:
FactoryLine control socket and IPC engine socket, both newline-JSON over
Unix streams.

6-step integration plan:
  (1) elai_cc speaks FactoryLine protocol
  (2) registers as a brain
  (3) replaces broken brainctl CLI
  (4) subscribes to Empire events
  (5) exposes capabilities as MCP server
  (6) fixes Empire's known issues (3 verified: CircularDep, HardImport,
      StdlibShadow)

Empire has verified working parts (IPC, FactoryLine bus, hooks, bus
registries, multi-brain isolation, FSB binary transport) and verified
broken parts (circular deps, hard imports, stdlib shadow). elai_cc brings
self-improvement TO Empire.

CURRENT STATE: NOT STARTED.


================================================================================
8. KEY DESIGN PRINCIPLES (UNCHANGED FROM ORIGINAL SUMMARY)
================================================================================

  - Code is the orchestrator, prompts are minimal (~330 tokens vs 1600+)
  - Rules are data in SQLite, not markdown files
  - Every FSM transition requires artifacts (evidence gates), not self-assessment
  - The model never sees enforcement logic, rule IDs, or internal details
  - Same seed = identical output (entropy quarantine + deterministic channels)
  - Graceful degradation everywhere (never crash, always fall back)
  - Process isolation is a context efficiency mechanism (not just security)
  - Self-modification is bounded (config archive, staged evaluation, human gates)
  - "I don't know" is a valid and preferred output over uncertain answers
  - Deterministic tools handle 60-70% of errors; the model handles the rest
  - Retry structure, not instruction ("write complete code!" empirically fails)
  - Fresh context per retry (never show model its own failed chain-of-thought)
  - Cloud as scalpel (~$0.01-$0.05/day), not crutch — or as accelerator (user chooses)
  - Every claim must be grounded before it influences decisions
  - The system works for any language and any project, not just Rust
  - The model is a code printer; the system handles file routing and validation
  - Speed is a user-controlled dial: local (free/slow) to cloud (paid/fast)


================================================================================
9. HARDWARE AND MODEL DECISIONS (UPDATED FROM A-1 BASELINE)
================================================================================

Phase 0 hardware: 1x RTX 3060 12GB.
Production target: 2x RTX 3090 24GB (48GB total).

Model family: Qwen2.5-Coder-Instruct (dense, uniform layers, head_dim=128).
  3B NF4+FA2: 1.92GB weights, ~40K context, 72% pass@1.
  7B NF4+FA2: 5.18GB weights, ~25K context, 90% code pass rate. DEFAULT on 12GB.
  32B NF4+FA2+TQ4: production default on 48GB. 64 dense layers enable DoLa.
    HumanEval 92.7%, MBPP 90.2% (highest among local models).

Gemma 4 E2B: REJECTED. Mixed head_dim (256+512) blocks FA2 on 7/35 layers.
18x larger KV cache. Mixed-head architecture is a dead end for FA2 pipelines.

MoE models (Qwen3-Coder-30B-A3B, DeepSeek): REJECTED for DoLa/steering
(expert routing breaks layer contrasting and activation addition).

FA2 (flash-attention 2): MANDATORY. Without it, attention ceiling drops 4x
(from 16K+ to ~4K). Requires Python 3.12 env (no 3.13 wheel on Windows).
Python env: elai-fa2 (Python 3.12, flash-attn 2.7.4).

DoLa: HARMFUL on 7B (-15% code quality, -6% factuality, 4.7x slower,
destroys stop-token distribution). Viable only on 32B+ (64 layers, untested
but projected).

Steering vectors: NEUTRAL on 7B with generic contrast pairs. Infrastructure
works. Needs domain-specific pairs (anti-staleness modern_api_preference vector).
Ship DISABLED by default.

Inference: custom sidecar server (.elai_cc/python/elai_inference/server.py),
not ollama. Sidecar provides FA2, DoLa, steering, TQ8/TQ4. Anthropic-compatible
protocol (/v1/messages, /v1/steering/*). Uses model.generate() + LogitsProcessor
(NOT manual token loop, per CLAUDE.md anti-stale).

Model storage: F:\.ELAI_workflow\.elai_cc\models\ (local HF format, not ollama).
Sidecar --model default points to abliterated 7B local path.

Per-VRAM tier table (token-efficiency-design.txt section 2):
  VRAM  Model               KV/tok  In-VRAM Ctx    Prompt Budget   Status
  8GB   3B NF4+FA2          1 KB    ~25K (proj)    20K              PROJECTED
  12GB  3B NF4+FA2          1 KB    ~40K           35K              MEASURED
  12GB  7B NF4+FA2          56 KB   ~25K (35K TQ8) 20K (30K TQ8)    MEASURED, DEFAULT
  16GB  7B NF4+FA2          56 KB   ~50K (proj)    45K              PROJECTED
  24GB  14B NF4+FA2         112 KB  ~40K (proj)    35K              PROJECTED
  48GB  32B NF4+FA2+TQ4     256 KB  ~35K (proj)    30K              PROJECTED, PROD
  any   Cloud frontier      0       128K+          per-provider     N/A


================================================================================
10. PART V KNOWLEDGE-ENGINE EXTENSIONS (B-29..B-35, post-A-48 additions)
================================================================================

Added 2026-05-05 after evaluating SocratiCode (AGPL TS, 33,975 LOC) for
patterns ELAI's already-complete A-0..A-48 + Phase 0..4E knowledge engine
should absorb. These are NEW features, not retrofits to "done" phases.

Architectural shape:

  SYMBOL-GRAPH LAYER (B-29):
    On top of the existing DKG (Phase 1A done) + dkg_data_flow (A-48 done),
    introduce a SymbolGraphCache with sharded persistence:
      - 27 name-index shards (a-z + _) for symbol-by-name lookup
      - 256 reverse-call file shards (single-byte SHA1 prefix) for
        callee→callers lookup
      - 500-file LRU for per-file payloads (lazy loaded)
    Four traversal tools backed by the cache:
      - compute_impact_radius (BFS over reverse index, hop-banded output)
      - compute_call_flow (DFS with cycle detection, lazy children)
      - compute_symbol_context (definition + callers + callees with confidence)
      - list_symbols (filter by file or query name)
    Confidence tiers from cross-file resolution: Local (same file),
    Unique (single resolver match), MultipleCandidates, Unresolved.
    Auto-injected into Auditor's PostToolUse hook on Write/Edit:
    every changed file gets a depth-2 impact summary appended to the
    auditor's context — closes a long-standing GRC pre-edit grounding gap.

  MULTI-LANGUAGE COVERAGE (B-30, closes A-35):
    Extends DKG from Rust-only to 17+ languages — Python / TypeScript /
    TSX / JavaScript / Go / Java / Kotlin / Scala / C / C++ / C# / Ruby
    / PHP / Swift / Bash / Svelte / Vue. Per-grammar failure isolation
    via std::panic::catch_unwind so a missing tree-sitter grammar
    doesn't break the rest. Multi-module resolvers: JVM suffix map for
    Maven/Gradle, C# namespace map (block + file-scoped C# 10+), Go
    module info from go.mod, TypeScript path aliases from
    tsconfig.json (extends-chain followed with cycle prevention).
    Composite languages (Svelte, Vue): parsed as HTML, <script> blocks
    re-parsed as TypeScript, <style> blocks for CSS @import.
    Closes "system works for any language" gap in spec — ELAI was
    Rust-only in practice prior to B-30.

  INDEXING ROBUSTNESS LAYER (B-31):
    Three patterns layered onto IndexingService:
      (1) Per-batch checkpointing: 50-file batches, blake3 hash per
          file, project metadata persisted after each batch with
          indexing_status = "in-progress" | "completed" | "cancelled".
          On resume, files matching cached hash are skipped.
          Cancellation flag honored at every batch boundary.
      (2) Cross-process IndexLock (LockScope::Index | Watch) using
          fs2 advisory file locking with PID liveness detection and
          stale-PID reclaim (SIGTERM POSIX / OpenProcess Windows).
          Concurrent ELAI instance attempting to index same project
          gets AlreadyLocked { holder_pid } and exits cleanly.
      (3) Startup auto-resume: on cli::main + dashboard::run, scan
          for projects with in-progress status; re-acquire lock;
          resume. Watcher auto-starts for completed projects with
          immediate catch-up. Graceful shutdown coordinates
          cancellation, watcher stop, lock release with 60s deadline
          before force-release.
    5 new SSB action types (IndexCheckpoint, IndexResume, IndexCancel,
    LockAcquired, LockContested) surfaced in dashboard
    IndexingProgressPanel.

  ADAPTIVE CHUNKING HARDENING (B-32):
    Pre-strategy detection routes minified files (avg_line_length > 500)
    to a character-based chunker with safe-boundary backwards-scan.
    Universal MAX_CHUNK_CHARS=2000 cap applied as the LAST step in EVERY
    chunker dispatch path (AST + line-window + character-based) so
    provider-side pretruncation acts as defense-in-depth, never primary
    enforcement. Closes a real bug class: vendored bundles, packed CSS,
    multi-thousand-char protobuf-generated literals all now indexable.

  INTERACTIVE GRAPH SURFACE (B-33):
    Tauri+Svelte+cytoscape graph window (NOT vendored AGPL JS; npm deps
    via user-confirmed package.json additions) with three modes:
      - File-import graph
      - Symbol call graph (consumes B-29 cache)
      - DecompositionTree (Concept → Phase → Step → BabyStep)
    Right-click on node → call get_impact_overlay → highlight reverse-
    transitive closure with hop-depth gradient. Live search, six layout
    algorithms (dagre / fcose / concentric / breadth-first / grid /
    circle), PNG export. Plus Mermaid text export for paste-into-PR
    workflow without webview rendering. Cross-window links: Plan tree
    node → "Show in Graph" decomposition mode; ActivityFeed file-write
    → "Show blast radius" file mode with overlay pre-applied.

  ENTRY-POINT DETECTION (B-34):
    Three-heuristic union (orphans-with-outgoing-calls + conventional
    names per language + framework router-call detection + test name
    patterns) producing reason-tagged EntryPoint output. 14-language
    ENTRY_POINT_NAMES table (configurable per project via [entry_points]
    block). Auto-injected into Designer's Recursive Decomposition when
    DecisionSource is ExistingProject — Designer grounds in real entries
    instead of hallucinating. Auto-injected into Researcher's /research
    Oracle pipeline as PROJECT_STARTING_POINTS for formulation phase.
    Performance budget: <500ms p99 on 10K-file project via payload
    selectivity (only orphans + conventional-filename matches load
    payloads).

  CONTEXT ARTIFACTS PIPELINE (B-35):
    Second indexing pipeline parallel to code. Project owners declare
    non-code knowledge via [[knowledge.artifacts]] in elai_cc.toml:
    DB schemas, OpenAPI specs, k8s manifests, ADRs, domain glossaries,
    Terraform configs. Each artifact:
      - Chunked (SQL gets CREATE-statement-boundary chunking; YAML/MD
        line-windowed via reused B-32 chunker)
      - Embedded into separate per-project context_{project_id}
        Qdrant-equivalent collection (ELAI's vector store, not Qdrant)
      - Auto-staleness via blake3 of whole-file content
      - Searchable via hybrid (vector + BM25 RRF) just like code
    New ArtifactStage in QueryPipeline (STANDARD + DEEP tiers; FAST
    is BM25-code-only). 7th GRC grounding method ArtifactCheck:
    PipelineGrc::verify_claim now dispatches ArtifactCheck for
    project-typed claims — Designer's plan sweep blocks claims about
    "users table has email column" that don't verify against the
    declared database-schema artifact.
    AVAILABLE_ARTIFACTS block auto-injected into Designer's briefing
    when ExistingProject decomposition fires; Designer is told
    "before deciding about [database|API|infrastructure], search the
    corresponding artifact". DDD projects with ubiquitous-language
    glossaries benefit massively — Designer no longer invents terms
    outside the bounded-context vocabulary.

ALL B-29..B-35: NOT STARTED. Recommended sequence (highest leverage first):
  B-29 → B-31 → B-30 → B-35 → B-32 → B-33 → B-34
Independence allows parallel work within each batch.

Gate: Auditor compute_impact_radius on every Edit; multi-lang DKG
working on real Python/TS/Go/Java repos; full re-index of 50K-file
project survives kill -9; minified bundles indexable; GraphWindow
renders ELAI's own crate dependency graph with right-click blast
radius; Designer's ExistingProject decomposition references real
entries + AVAILABLE_ARTIFACTS.

================================================================================
END PART 1
================================================================================

Continue to DETAILED_SUMMARY_02_current_state.md for what's actually built,
wired, and shipping today.
