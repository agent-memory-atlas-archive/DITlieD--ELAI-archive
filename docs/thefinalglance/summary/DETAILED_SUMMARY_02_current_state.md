# ELAI_CC — Detailed Summary, Part 2: Current State (What Exists, What Works, What's Wired)
Date: 2026-04-25
Predecessor: DETAILED_SUMMARY_01_overview_and_architecture.md
Successor: DETAILED_SUMMARY_03_gaps_and_roadmap.md
Source: 23 implementation audits + 8 dashboard wiring audits + 6 lossless spec summaries.

This file is the honest status report. For each phase, it answers three
questions: Is it built? Is it wired? Is it visible in the dashboard?


================================================================================
LEGEND
================================================================================

Built status (does the code exist):
  IMPL = implemented (all sub-step deliverables present)
  PARTIAL = some sub-step deliverables present, others stubs/missing
  MISSING = not started
  EVAL = evaluation phase, output is a DECISION

Wired status (does production hot path call it):
  WIRED = called from CLI / task_runner / dashboard backend in production
  COSMETIC = called from one path but not all (e.g. REPL but not TaskRunner)
  DEAD = code exists but no production caller
  N/A = test infra / evaluation phase, no production wiring expected

Dashboard status (is it visible to user):
  SURFACED = real backend signal renders in a UI element
  PARTIAL = command exists but UI thin / static / wrong shape
  INVISIBLE = no Tauri command + no Svelte consumer
  DEAD UI = UI element exists but backend producer is empty/stub


================================================================================
PHASE-BY-PHASE STATUS (PRE-0 THROUGH 4K MVP)
================================================================================

----------------------------------------------------------------------
PRE-PHASE 0 — Windows platform fixes for claw-code base
----------------------------------------------------------------------
  Built:     PARTIAL (4/5 sub-steps verifiable)
  Wired:     N/A (platform fixes)
  Dashboard: N/A
  Notes:
    - cfg-gate unix-only code (PermissionsExt, set_mode) applied in commit 4f857d9
    - Plugin hook tests fixed (.cmd on Windows) in commit d561632
    - tools tests path-separators / python3->python / PoisonError on env lock fixed
    - 2 warnings fixed
    - P0.5 metrics TOML (.elai_cc/metrics/pre_phase_0.toml) MISSING
    - No CI workflow file on disk (no continuous Windows-pass verification)
    - Latent Windows bug: orchestrator/src/hooks.rs:634 hardcodes "sh"
      (inherited from stripped runtime crate; outside P0.4 scope)
  Spec drift acknowledged: spec references crates/runtime/ + crates/rusty-claude-cli/
    but Phase 0 renamed them to orchestrator + cli respectively.

----------------------------------------------------------------------
PHASE 0 — Fork and strip (walking skeleton)
----------------------------------------------------------------------
  Built:     PARTIAL
  Wired:     N/A
  Dashboard: N/A
  Tally:
    IMPLEMENTED: 5 (0.1 baseline, 0.4a leaf-crate restructure, 0.5 verify
                 restructure, 0.6 contracts skeleton, 0.7a provider+config)
    PARTIAL: 9 (0.2 strip, 0.3 verify-strip, 0.4b mid-layer, 0.4c top-layer,
                0.7b FSM+tool, 0.7c hardening, 0.8 verify, 0.9 entropy,
                0.10 determinism)
    MISSING: 0 outright
  Workspace: 16 members (9 spec + contracts + 6 Phase-1+ additions: agent,
             ipc-types, web_search, scaffold, web_validator, dashboard).
  Dead code that should have been stripped (Phase 0 spec said REMOVE):
    - crates/orchestrator/src/compact.rs (696 LOC, full claw-code compaction)
    - crates/core/src/usage.rs (279 LOC: ModelPricing, UsageCostEstimate,
      pricing_for_model, format_usd; spec wanted minimal TokenUsage only)
    - crates/providers/src/oauth.rs (DEFERRED in manifest, never removed)
    - crates/orchestrator/src/remote.rs (EVALUATE/STRIP per spec)
    - crates/orchestrator/src/prompt.rs (hardcoded-prompt resolution)
  Cross-cutting violations:
    - 84 std::collections::HashMap imports + 10 HashSet (HashMap ban not enforced)
    - 40+ entropy quarantine leaks (SystemTime::now / Instant::now in
      orchestrator/cli/tools/providers/plugins/rules/knowledge/web_validator/
      agent/dashboard) — 0.9 acceptance violated
    - cli/src/main.rs is 11,181 LOC vs <500 LOC spec target (22x over)
    - file_ops.rs, bash_validation.rs, stale_branch.rs landed in orchestrator/
      instead of tools/ per MODULE MAPPING; bash_validation.rs duplicated
      across both crates
    - Session persistence still JSON-based; msgpack for disk sessions (0.7b)
      not implemented. Postcard used for IPC wire only.
  Missing scripts:
    - scripts/check_entropy_quarantine.sh
    - scripts/determinism_proof.sh
    - scripts/smoke_test_ollama.sh

----------------------------------------------------------------------
PHASE 1A — DKG (tree-sitter -> SQLite graph)
----------------------------------------------------------------------
  Built:     IMPLEMENTED
  Wired:     WIRED via IndexingService
  Dashboard: PARTIAL (KnowledgeDrawer "languages" line always "–" because
             task_runner.rs:2818 hardcodes Vec::new() for languages on DkgReady)
  Metrics:   metrics/phase_1A.toml — 135 files / 8075 nodes / 4148 edges / 9.67s
  Tests:     28 #[test] in tests/dkg_tests.rs (spec target 20)
  Critical:  DUAL-PATH BUG — IndexingService writes .elai_cc/indexes/dkg.db,
             cli/main.rs:3752 + 5437 hardcode .elai/dkg.db. CLI consumers
             (PageRank, telescope) read empty DB.
  Debt:      DKG itself uses thiserror KnowledgeError. 1B/1C/1D scope retains
             15 Box<dyn Error> public APIs.

----------------------------------------------------------------------
PHASE 1B — BM25 (tantivy full-text search)
----------------------------------------------------------------------
  Built:     IMPLEMENTED (all 5 sub-steps pass)
  Wired:     WIRED at orchestrator/src/indexing_service.rs:177 + 
             cli/main.rs:3658 + knowledge/src/startup.rs:406
  Dashboard: PARTIAL (orphan_files=0, p50=1ms, p99=2ms, build=538ms for 73 files
             — real metrics; KnowledgeDrawer doesn't expose chunk_count or
             ChunkKind distribution)
  Notes:     One cosmetic path divergence:
             {project_root}/.elai_cc/indexes/projects/default/bm25/ (actual)
             vs spec wanted .elai_cc/projects/default/bm25/ (no indexes/ segment)
             — semantically equivalent.
  Debt:      8 Box<dyn Error> at lines 155, 217, 243, 262, 314, 329, 362, 403

----------------------------------------------------------------------
PHASE 1C — Vector search (dual-model ONNX)
----------------------------------------------------------------------
  Built:     PARTIAL PASS (skeleton + circuit breaker + cache + RRF + canary all real)
  Wired:     WIRED IN CODE, DEGRADED AT RUNTIME
  Dashboard: INVISIBLE (per-model EmbeddingCircuitBreaker state grid not surfaced;
             VectorCache hit rate not surfaced)
  Critical Gaps:
    - ort = "=2.0.0-rc.12" (knowledge/Cargo.toml:27) violates CLAUDE.md
      Anti-Stale-Training "ort stable, never -rc"
    - No ONNX files in .elai_cc/models/ (path mismatch: spec says
      .elai_cc/models/onnx/, code uses .elai/models/)
    - Silent fall-back to BM25-only at runtime
    - code_vec populated into cache during indexing but NEVER QUERIED at
      runtime (text_vec only) — effectively 2-ranker RRF, not spec's 3-ranker
    - Canary uses embed("test") not spec's embed("fn main()")
    - SSB file_write -> vector_cache invalidation NOT wired
      (invalidate_file() returns NotSupported; schema lacks file_path column)
    - HNSW default is BruteForceIndex; usearch is feature-gated
    - phase_1C metrics file absent (fails EXIT #8)
    - vector-stage latency_ms hardcoded 0
  Debt:      5 Box<dyn Error> at lines 57, 71, 89, 194, 218

----------------------------------------------------------------------
PHASE 1D — PageRank repo map
----------------------------------------------------------------------
  Built:     MOSTLY IMPLEMENTED (all 5 sub-steps; 24 tests including 3 proptest)
  Wired:     WIRED IN CODE, DEAD AT RUNTIME (DB path bug)
  Dashboard: DEAD UI (KnowledgeDrawer pagerank section + get_pagerank_top stub)
  Notes:     compute_repo_map called from pipeline.rs:387 inside
             QueryPipeline::query_with_tier. pipeline.dkg_engine set at
             cli/main.rs:3756 behind `if .elai/dkg.db exists` gate.
             That gate NEVER fires in production because the actual DB lives
             at .elai_cc/.elai_cc/indexes/dkg.db (8.2 MB populated, 2026-04-19).
             Dashboard consumer uses correct constants. Two-line fix.
  Debt:      1 Box<dyn Error> at mod.rs:28; latent UTF-8 panic in
             truncate_signature (byte slice vs char slice at mod.rs:137);
             cosmetic struct rename PageRankNodeInfo vs spec's DkgNodeInfo

----------------------------------------------------------------------
PHASE 1E — CatRAG + A-RAG (query pipeline assembly)
----------------------------------------------------------------------
  Built:     PARTIAL (structural present, runtime mostly degraded)
  Wired:     PARTIAL (3 integration bugs neutralize at runtime)
  Dashboard: INVISIBLE (entire CatRAG+A-RAG+QueryPipeline assembly has zero
             dashboard surface; ExploreWindow bypasses Rust pipeline by routing
             to sidecar /search)
  Runtime stage firing (CLI default):
    LIVE: bm25, rerank (RRF), cross_encoder (when model present), token_fit
    EMPTY/SKIPPED: vector (only if vector_index.json exists at .elai/data/),
                   pagerank (dkg_engine always None due to 1A path bug),
                   catrag (overlay computed at pipeline.rs:433, bound to
                          _overlay, discarded — empty results pushed),
                   arag (corpus empty — AragOrchestrator.add_chunk has zero
                        callers in CLI)
  Effective runtime: "bm25 -> rerank(BM25-only) -> cross_encoder -> token_fit"
  Token savings: phase_1E.toml records 96.1% — but measure_1e.rs:237-241
    attributes this to BM25's 200-char snippet truncation, NOT real
    hierarchical retrieval selectivity. Real CatRAG+A-RAG savings UNMEASURED.
  Per-stage latency_ms hardcoded 0 across all 13 PipelineStage constructors
    (entropy quarantine comment at pipeline.rs:30-33).

----------------------------------------------------------------------
PHASE 1F — Speculative decoding evaluation
----------------------------------------------------------------------
  Built:     PASS (decision correctly resolved)
  Wired:     N/A (DROPPED)
  Dashboard: N/A
  Decision:  DROP (NOT_SUPPORTED). Documented in
             F:\.ELAI_workflow\workflow\reasoning\phase-1F-decisions.md
             (46 lines, 2026-04-07). 1F.1 + 1F.2 implemented; 1F.3, 1F.3b,
             1F.4, throughput portion of 1F.5 LEGITIMATELY SKIPPED via
             first-match decision cascade. SpeculativeConfig retained at
             ollama.rs L254-286 for future Phase 2+ vLLM revisit
             (zero-cost at runtime, Default false).
  Notes:     Three minor INFO-level observations: no deprecation comment on
             retained config; stale Gemma 4 model names in ignored integration
             test; no TOML sibling for decision (correct per spec L921/923).

----------------------------------------------------------------------
PHASE 1G — Cross-encoder reranker
----------------------------------------------------------------------
  Built:     PARTIAL (1G.1, 1G.2, 1G.5 IMPLEMENTED; 1G.3, 1G.4 PARTIAL;
             1G.6 DEAD)
  Wired:     WIRED at call site only (pipeline.rs:474 + cli/main.rs:3715-3730);
             batching path, SSB notifier, and L-6 fallback DEAD
  Dashboard: INVISIBLE (reranker variant in use + NDCG@10 not surfaced)
  Critical:
    - .elai_cc/models/onnx/ directory does not exist
    - .elai_cc/metrics/phase_1G.toml does not exist (all sibling phases have it)
    - scripts/quantize_cross_encoder.py does not exist
    - sweep test early-returns on every invocation
    - ndcg_improvement > 0.0 assertion never evaluated against a real model
    - run_batch_worker / rerank_via_channel test-only (no production spawn)
    - CLI uses CrossEncoderReranker::new (NoopSsbNotifier) — degradation
      events dropped
    - fallback_model_path never set in production — L-6 tier unreachable
    - CLI vs tests disagree on model filename (cross-encoder.onnx vs
      models/onnx/cross-encoder-l12.onnx)
    - One library-code .unwrap() at reranker.rs:465 (.expect("semaphore closed"))
      VIOLATES CLAUDE.md hard rule
  Debt:      0 Box<dyn Error> (clean); 1 .expect() violation
  Notes:     Startup canary does not enforce 5s budget; no ms-marco-MiniLM-L12-v2
             model identity verification

----------------------------------------------------------------------
PHASE 1H — SPLADE keyword expansion (conditional)
----------------------------------------------------------------------
  Built:     PARTIAL (1H.1+1H.3 PASS; 1H.2 PASS w/ soft WARN; 1H.4 PARTIAL)
  Wired:     N/A (feature-gated)
  Dashboard: N/A
  Decision:  UNKNOWN (PENDING). Three outcomes (EARLY DROP / DROP / KEEP)
             reachable in code but no end-to-end run executed because
             prithivida/Splade_PP_en_v1 ONNX + tokenizer not staged at
             models/splade/.
  Missing artifacts: workflow/reasoning/phase-1H-decisions.md (BLOCK),
                     .elai_cc/metrics/phase_1H.toml (BLOCK),
                     examples/measure_1h.rs (WARN — test harness covers).
  Feature flag present: cfg(feature="splade") in Cargo.toml + lib.rs +
                        test files. ort unconditional (correct).

----------------------------------------------------------------------
PHASE 1I — Resilience layer
----------------------------------------------------------------------
  Built:     PARTIALLY IMPLEMENTED
  Wired:     PARTIAL — broken at integration layer
  Dashboard: INVISIBLE (SearchMode + cascade transitions not surfaced)
  Per-sub-step:
    1I.1 Circuit breaker: IMPLEMENTED (fresh-session-on-probe for HALF_OPEN
         is caller responsibility — not enforced here)
    1I.2 SearchMode cascade: PARTIAL (logic complete, NOT WIRED to query path)
    1I.3 Query validator: IMPLEMENTED + WIRED (pipeline.rs:140 first statement)
    1I.4 Startup readiness: IMPLEMENTED (3 variants correct; FSM gating on
         Unavailable not implemented; reference vector in separate JSON)
    1I.5 Cross-encoder semaphore: IMPLEMENTED w/ CLAUDE.md violation at
         reranker.rs:465 (.expect)
    1I.6 Query trace: PARTIAL (structs/percentiles correct; pipeline.rs:520
         hardcodes SearchMode::Hybrid + total_latency_ms: 0;
         SessionTraceStats has no production call site)
    1I.7 Index version schema: IMPLEMENTED (14 passing tests, zero
         unwrap/expect in non-test code)
  Critical:  Grep cascade.evaluate / cascade.current_mode in production code
             returns ONE hit outside tests — cli/src/main.rs:3800-3808
             (single-shot startup call). Neither QueryPipeline nor
             query_with_tier holds or consults a DegradationCascade.
             If GraphCodeBERT dies mid-session, breaker opens locally,
             but no SearchMode switch fires, no knowledge_degradation SSB
             event for the dense-vector path. Auto-recovery via HALF_OPEN
             is dead code outside startup.

----------------------------------------------------------------------
PHASE 2A — GRC state machine
----------------------------------------------------------------------
  Built:     IMPLEMENTED (PASS w/ minor deviations)
  Wired:     WIRED — BasicGrcEngine instantiated from orchestrator in 6 files
             (agent_pool.rs, task_runner.rs:2039, checkpoint.rs 4 sites,
             recovery.rs 2 sites, bsp/engine.rs:478) + PipelineGrc wrapper at
             orchestrator/src/grc.rs
  Dashboard: PARTIAL — ReasoningDrawer renders fake/synthetic data
             (emit_reasoning at task_runner.rs:3985-4004 hardcodes constants)
  Notes:
    - Trait has 10 methods (not "6 (4 impl + 2 stubs)" 2A literally specified)
      because 2B/2C/2D have already landed; no stubs remain. Additive.
    - regex crate added to Cargo.toml (used by ground_plan, a 2B feature)
    - 18 tests in grc_engine.rs; all 12 spec-required (t01–t12) present
    - GroundingLevel ordering BUG FIXED (Uncertain < High < Verified)
    - Moderate default budget = 800 chars (CLAUDE.md says 1000 — discrepancy)
    - GrcError::InvalidTransition.from is Option<GrcState> (deviation from
      spec's bare GrcState; matches T9's "from=None" requirement)
    - Zero unwrap/todo/unimplemented/Box<dyn Error> in engine.rs
    - 1 .expect() on a static regex literal (acceptable per CLAUDE.md)

----------------------------------------------------------------------
PHASE 2B — Grounding verification
----------------------------------------------------------------------
  Built:     PASS (16 tests, plan sweep regex works, metrics latency met)
  Wired:     PARTIAL — type-level wired, behavioural wiring missing
  Dashboard: INVISIBLE (verify_claim results, ground_plan PlanGroundingResult,
             GroundingSweepGate Pass/Block — no Tauri command, no SSB action,
             no Svelte consumer)
  Notes:
    - 6 enum variants present; 3 real (FILE_CHECK, SOURCE_CHECK, TEST_CHECK);
      3 stubbed (WEB_CHECK, API_CHECK, API_REGISTRY_CHECK) returning typed
      GrcError::VerificationError "network verification not yet implemented"
    - PipelineGrc::verify_claim + ground_plan exist at orchestrator/src/grc.rs
      but NO production FSM/pipeline code calls ground_plan(plan) on a real plan
    - GroundingSweepGate at evidence.rs:191-218 registered in gate list at line 471
    - Three unit tests at 660-682 verify it BLOCKS when ungrounded_claims is
      non-empty — but nothing in the codebase calls engine.ground_plan(plan)
      and pipes PlanGroundingResult.ungrounded into ctx.ungrounded_claims.
      Producer side is missing. Gate vacuously passes.
    - Spec required adding tools/ as reasoning/ dependency for FILE_CHECK/
      SOURCE_CHECK; impl used internal FileReader trait with Std/Mock
      (arguably better, avoids R-2B-1 coupling, but spec divergence)
    - regex = "1" added without being on spec's dep list

----------------------------------------------------------------------
PHASE 2C — Per-agent GRC configuration
----------------------------------------------------------------------
  Built:     IMPLEMENTED (15 tests pass live; metrics file exists)
  Wired:     INTEGRATION GAP — reasoning crate WIRED, orchestrator divergent
  Dashboard: INVISIBLE (zero Tauri commands + zero Svelte components for
             AgentGrcConfig)
  Notes:
    - 6 roles all present at reasoning/src/config.rs:
        Executor 8/1200, Auditor 10/1200, ContextCompiler 3/600 observe-only,
        PromptEngineer 5/800 [Observe,Hypothesize,Converge],
        MemoryWriter 2/400, Designer 8/1200
    - Orchestrator has DIVERGENT 2nd table at agent_pool.rs:467:
        ContextCompiler falls back to ..default() — full 5-step chain +
          Uncertain + all methods (VIOLATES "observe-only" EXIT CRITERION)
        Designer uses [Observe, PlanAction, Act, Verify] with no override —
          chain invalid under GRC base transitions
        PromptEngineer uses different 5-step chain
    - Production wiring routes through ORCHESTRATOR's broken copy
    - Anti-rubber-stamp WIRED: const FINDING_PREFIX = "FINDING:"; engine.rs:297-315
      checks Converge by role=="auditor" requires >=1 Verify step with
      FINDING: prefix. Test t07 passes.
    - NO TOML config in elai_cc.toml — TomlConfigLoader is hardcoded-match
      stub. TOML parsing deferred to phase 4A per in-code comment.
    - routing_overhead_pct = 28.5 (documented deviation from <10% target)

----------------------------------------------------------------------
PHASE 2D — CSQ + Inline feasibility
----------------------------------------------------------------------
  Built:     IMPLEMENTED (within reasoning-crate scope)
  Wired:     UNWIRED — zero production callers anywhere
  Dashboard: INVISIBLE (CsqResult Critical/guard_added/passed,
             InlineFeasibilityResult Pass/Fail/Blocked — zero Tauri commands,
             zero SSB action_types)
  Critical:
    - generate_csq + check_inline_feasibility implemented in BasicGrcEngine
      with zero todo/unimplemented/unwrap
    - CSQ pattern set covers CRITICAL keywords (file/write/delete/fs::/net::/
      auth/token/credential/password/money/payment), case-insensitive
    - 10 tests in crates/reasoning/tests/csq_tests.rs map 1:1 to spec T1-T10
      + 4 bonus wrapper tests in crates/orchestrator/src/grc.rs
    - PipelineGrc::generate_csq exists at orchestrator boundary
    - GREP of entire .elai_cc shows ZERO production callers — only tests
    - NO file_ops.rs / dispatcher.rs / fsm.rs / mcp_tool_bridge.rs / gate.rs /
      permission_enforcer.rs invokes CSQ
    - A Critical-unguarded CSQ today CANNOT block a Write/Edit/Bash
    - InlineFeasibilityState::reset() never called between phases
    - No code path consumes FeasibilityOutcome::Blocked to emit ReplanRequest
  By design per spec D4 (orchestrator's responsibility 3A+) but is the
  primary safety enforcement gap.

----------------------------------------------------------------------
PHASE 3A — Full FSM (INTERACTIVE mode)
----------------------------------------------------------------------
  Built:     IMPLEMENTED (3A.1, 3A.2, 3A.3, 3A.5 all PASS; 3A.4 CONDITIONAL)
  Wired:     PARTIAL — InteractiveGate dead code
  Dashboard: INVISIBLE (FSM badge in StatusBar dead because no production
             writer of PhaseTransitionPayload — defaults to "IDLE" forever;
             elaborate 10-state stateColour/shouldPulse switch dead code;
             EXECUTING-pulse animation never fires; MonitorWindow.currently
             never fires)
  Notes:
    - 9 spec states + additive Researching (Oracle mode) at fsm.rs:24-38
    - 7 evidence gates at evidence.rs:468-478 all present
    - msgpack persistence atomic (write-to-tmp + rename at fsm.rs:742-747)
    - CliInteractiveGate at cli/src/main.rs:7354-7383 — defined, NEVER
      instantiated. All 13 call sites of run_interactive_gate are inside
      #[cfg(test)] test modules in gate.rs. Production bypasses
      Fsm::transition() directly.
    - Library implementation 100% correct; integration into real CLI
      session loop is missing.
    - SPEC_UPDATE_NEEDED in 3A.1 (AUDITING -> EXECUTING / -> PLANNING from
      SKILL.md not in orchestrator.txt)
  Single highest-impact fix: ~30 min, ~10 lines — append
    `writer.write_payload("fsm", 0, SsbPayload::PhaseTransition(...), Vec::new())`
    inside Fsm::transition_with_metrics at fsm.rs:639. Activates 4 UI
    elements simultaneously.

----------------------------------------------------------------------
PHASE 3B — SSB (Shared State Board)
----------------------------------------------------------------------
  Built:     IMPLEMENTED WITH SUBSTANTIVE DEVIATIONS
  Wired:     WIRED (single-writer mpsc, broadcast-after-commit, BEGIN
             IMMEDIATE + COMMIT)
  Dashboard: SURFACED (ActivityFeed.svelte:66 polls get_recent_actions every
             500ms; commands.rs:80-91 -> SsbViewer::recent_actions executes
             real SQL against read-only WAL handle. SUBSTRATE IS SOUND.
             Feed often looks sparse for orchestration events because 3A/3D/3E
             producers fail to write rows.)
  Notes:
    - 74 action_types in enum (spec mandated 17 — all 17 canonical present;
      57 forward-compat extensions across batches 1A/1B/1C/2A/2B/3A/3B-4G/
      3C-4H/4C/ful-a/ful-c/ful-d/CHAT/Settings)
    - Read pool is deadpool-sqlite, not r2d2 (deviation)
    - ZERO 50K inserts/sec benchmark exists anywhere — performance claim
      is ungrounded
    - ZERO use of SystemTime::now in ssb.rs (clean — both now_ms_via_clock
      at L1751 and execute_write at L2231 flow through Arc<dyn ClockProvider>)
    - 17 OTHER orchestrator files DO use SystemTime::now directly
      (task_runner.rs, fsm.rs, session.rs, checkpoint.rs, etc. — out of scope
      for 3B but in scope for entropy debt burndown)
    - msgpack via rmp_serde, user_version migration, 1 MiB payload guard

----------------------------------------------------------------------
PHASE 3C — Agent System
----------------------------------------------------------------------
  Built:     PARTIAL (3C.1 PASS, 3C.2 PARTIAL FAIL, 3C.3 PASS, 3C.4 PASS,
             3C.5 FAIL, 3C.6 PASS)
  Wired:     PARTIAL — SupervisionManager NOT WIRED
  Dashboard: INVISIBLE (per-agent panel; agent role badges; death classification;
             heartbeat liveness all not surfaced)
  Notes:
    - 8 AgentRoles defined in ipc-types/src/lib.rs:17-28
      (Executor, Auditor, Researcher, Designer, PromptEngineer,
      ContextCompiler, MemoryWriter, MediaValidator). all_roles_serializable
      test asserts roles.len() == 8.
    - blake3 in checkpoint + anti-spiral (DELIBERATE divergence from spec's
      blake2s, follows CLAUDE.md Anti-Stale-Training HARD RULE — spec text
      should be updated)
    - Invisible walls PARTIALLY enforced: is_visibility_filtered_path in
      tools/src/lib.rs:1336-1346 blocks .elai_cc/ and rules.db for
      read_file/write_file/edit_file/grep_search BUT returns
      error_code="visibility_filtered" + "visibility_filtered: path is
      hidden" message — VIOLATES spec R31 "not found" requirement
      (indistinguishable from nonexistent)
    - NO per-agent allowed_paths: Vec<GlobPattern> on AgentHandle (only
      global HIDDEN_PATH_PATTERNS constant)
    - SpiralTracker WIRED in task_runner.rs:1046-1087 + initialized at
      3190 + record_step per GRC step at 1087
    - Two-stage escalation working
    - SupervisionManager IMPLEMENTED but NEVER wired into task_runner —
      INFINITE RESTART LOOPS POSSIBLE
    - Phase 2C divergent AgentGrcConfig table STILL LIVE
    - ContextCompiler/MediaValidator fall through to default() at
      agent_pool.rs:563-566, 587-590

----------------------------------------------------------------------
PHASE 3D — BSP supersteps + channels
----------------------------------------------------------------------
  Built:     PASS (44 tests across channels.rs + engine.rs +
             bsp_determinism.rs)
  Wired:     WIRED
  Dashboard: INVISIBLE (superstep counter / channel state / JoinSet status
             not surfaced)
  Notes:
    - 4 channel types present: LastValue, Topic (accumulate flag),
      EphemeralValue, UntrackedValue<T: DeepSizeOf>
    - ChannelState enum has exactly 3 variants — UntrackedValue intentionally
      absent from checkpoint serialization
    - DeepSizeOf is hand-rolled trait (no external deepsize dep) with Vec<T>
      impl using capacity * size_of::<T>() floor
    - BinaryOp confirmed absent (entropy quarantine)
    - Parallel execution YES via tokio::task::JoinSet at engine.rs:410 +
      tokio::sync::Semaphore at 399. Per-agent tokio::time::timeout at 419.
      test_spawn_parallel_true_concurrency proves wall-time <250ms for two
      100ms tasks.
    - Determinism PROVEN: tests/bsp_determinism.rs builds same checkpoint
      100 times byte-identical. BspCheckpoint uses BTreeMap.
    - Straggler timeout DEFAULT_STRAGGLER_TIMEOUT_SECS=15 numerically aligned
      with 3C AGENT_HEARTBEAT_TIMEOUT_SECS=15
    - apply_writes is documented stub (gated to 3F/3J wiring)
    - All 4 spec amendments satisfied (Topic accumulate, memory_monitoring,
      ChannelRegistry, save_checkpoint after evidence_gate)

----------------------------------------------------------------------
PHASE 3E — Context budget allocator
----------------------------------------------------------------------
  Built:     IMPLEMENTED WITH TWO BLOCKING GAPS
  Wired:     PARTIAL — REPL works, TaskRunner falls back to zeroed stub
  Dashboard: INVISIBLE (ContextBudget per-slot allocation viz absent)
  Critical:
    - 8 slots present: SystemPrompt(0)..OutputReserve(7) at context_budget.rs:509-518
    - SLOT_COUNT=8 const + compile-time assertion at L529
    - 6 of 7 sub-steps fully implemented (3E.1, 3E.2, 3E.3, 3E.4, 3E.6, 3E.7)
    - 3E.5 file_overlap_score MISSING ENTIRELY (zero matches anywhere for
      file_overlap_score / compute_overlap / filter_ssb / per-entry intersection)
    - 3J.WIRE.BUDGET only HALF-WIRED:
        dispatcher::build_dispatch_context (L192-254) threads
        Option<ContextBudget> into DispatchContext.budget
        BUT only production caller — task_runner.rs:2992-2999 — passes
        profile=None, falling through to ContextBudget::compute(raw_max,
        [0, 0, 0], 4096). This is exactly the zeroed [0,0,0] stub
        3J.WIRE.BUDGET was meant to replace.
        CLI call site (cli/src/main.rs:7628) follows the same pattern.
        Wired path EXISTS; production NEVER takes it.
    - REPL path at cli/main.rs:7680-7727 sets all three slots from real
      measurements + ShortCoder
    - ModelCapabilityProfile (A-10) STRUCTURALLY YES, OPERATIONALLY NO:
      4 tier constructors (micro_7b_12gb, micro_3b_12gb, large_32b_48gb,
      micro_8gb), compute_with_profile (L768-872) with attention-ceiling
      enforcement and profile-driven SlotWeights/fixed_slot_targets/
      output_reserve all implemented and exhaustively tested
      (100-config zero-sum invariant). NO production code constructs a
      profile and passes it into dispatch.

----------------------------------------------------------------------
PHASE 3F — Rules engine (self-improving)
----------------------------------------------------------------------
  Built:     SUBSTANTIALLY IMPLEMENTED (~88%, 9/10 sub-steps PASS)
  Wired:     WIRED into hooks via T1Hook in plugins crate
  Dashboard: PARTIAL (RulesDrawer shows real lifecycle counts via
             get_rule_lifecycle_stats; no per-rule drilldown despite
             get_rule_detail being registered; no live update)
  Notes:
    - 7 predicate types (spec ambiguity 5 vs 6 resolved at 7 because A-6
      VERSION_GATE added on top): AstPattern, GrepPattern, SchemaRequired,
      MetricGate, Composite, GraphReachability, VersionGate
    - 39 [[rules]] entries across data/rules/engineering.toml (12) +
      data/rules/web.toml (27) — exceeds spec floor of 33
    - 21 deprecated_patterns seed entries in schema.rs
    - SEED rules inserted with status=SHADOW (project_rule_seeder.rs:35),
      not ACTIVE as spec called out — divergence
    - VERSION_GATE (A-6) IMPLEMENTED end-to-end:
      Predicate variant (predicate.rs:158-168), validation (236-266 with
      8-ecosystem CHECK), schema table deprecated_patterns (schema.rs:119-129),
      21 seed entries, migration v2->3 wires it in, eval_order=1 alongside
      GREP, T2 hook loads via with_deprecated_imports, self-discovery
      auto-proposes VersionGate when ViolationRecord.replacement_name is Some
    - 3F.10 benchmarks ENTIRELY MISSING (no criterion + no metrics file)
    - COMPOSITE depth=8 still hardcoded ungrounded
    - Spec-named "web_fetch <img>", "web_fetch <table>", "file_read TODO"
      rules not found by name in TOML; web.toml entries likely cover under
      different names

----------------------------------------------------------------------
PHASE 3G — Hierarchical plan management
----------------------------------------------------------------------
  Built:     PASS WITH CAVEATS (all 7 sub-step deliverables present)
  Wired:     WIRED via 6 budget call sites + /plan command
  Dashboard: INVISIBLE for PlanSkeleton tree itself; PlanWindow.svelte (272
             LOC) intentionally unregistered in WindowRegistry.ts:27 (PLAN
             routes to Workspace which polls PhaseComplete SSB rows, NOT
             PlanSkeleton)
  Notes:
    - PlanSkeleton struct + IndexMap fields + serde_seq for insertion order;
      to_toml/from_toml/read_from_file/write_to_file all real, round-trip
      tested byte-identical
    - /plan SlashCommand in cli/src/commands.rs:1159 + parser at L1445 +
      handler at cli/src/main.rs:4572-4619 (supports /plan, /plan list,
      /plan <phase_id>)
    - Plan auto-loads at startup via load_plan_if_exists L5372 from plans/*.toml
    - FsmState::Planning has full transition graph
    - PlanSkeleton woven into context budget at six call sites via
      plan_phase_slot_tokens (L4097/L4974/L5117/L5187/L5240/L5263/L5290) +
      build_phase_spec_content L5461 (skeleton TOML + CONSTRAINTS + telescope
      block as PhaseSpec system-prompt section)
    - /plan handler does NOT call fsm_transition_sync — appropriate for
      inspection command
    - PlanSkeleton struct LACKS explicit created_at/updated_at u64 fields
      (delivered functionally via SSB plan_update timestamps)
    - propagate_changes uses self.version for SSB payload even when iterating
      downstream phases — minor
    - Real-tokenizer integration deferred — unit tests use MockProvider

----------------------------------------------------------------------
PHASE 3H — Session handoff (CMV-derived DAG)
----------------------------------------------------------------------
  Built:     DATA LAYER COMPLETE (3090 LOC SessionDag library)
  Wired:     RUNTIME WIRING DEAD ON ARRIVAL
  Dashboard: DEAD UI (Settings.svelte:112-117 declares SessionHandoffEntry
             with fields {id, branch, ts} that don't exist on backend
             response; render at 818-822 dereferences nonexistent fields →
             would show ? rows even on non-empty data)
  Critical:
    - JSONL still on disk (session.rs:159 save_to_path -> render_jsonl_snapshot
      at L396). DAG snapshots.commit_hashes uses JSON-string. Findings/
      code_changes/tests_written declared as BLOB but code uses
      serde_json::from_slice — JSON, not msgpack. Stub payloads inside
      snapshot_context.content ARE msgpack. Phase 0 item 0.7b unresolved.
    - blake3 in use at session_dag.rs:778 (trim Pass 2), L1198 (recover_stubs),
      L1259 (recover_single_stub). No blake2/sha2.
    - CLI hard-codes stub_ssb_seq: None everywhere (cli/main.rs:4061, 4151).
      Pass 1 of trim_context skips any message with stub_ssb_seq.is_none()
      (session_dag.rs:759). Net effect: at runtime trim produces ZERO STUBS.
    - create_handoff/load_handoff/recover_stubs/recover_single_stub/
      merge_branch/close_branch ZERO call sites outside tests.
    - Snapshots saved only at REPL turn end (L4325) — no GRC-step,
      per-tool-call, or phase-transition checkpoints (spec violated).
    - /resume opens DAG to grab last_snap_id (L4984) but loads actual
      messages from JSONL via Session::load_from_path (L4959) — DAG
      snapshot_context is write-only.
    - Only create_branch exposed via /session fork (L5132-5166); merge/close
      UNREACHABLE from UI.
    - SPEC AMENDMENT #2 (SSB action_type='checkpoint' on branch merge)
      NOT implemented — merge_branch never writes to SSB.
  DAG runs as parallel write-only ledger alongside legacy JSONL session.
  Library is shippable; integration is not.

----------------------------------------------------------------------
PHASE 3I — Deterministic Simulation Testing (DST)
----------------------------------------------------------------------
  Built:     PARTIAL (strong reference-model + BSP + adversarial coverage,
             weak madsim/IPC layer, one fabricated metric)
  Wired:     N/A (test infrastructure)
  Dashboard: N/A (correctly invisible)
  Notes:
    - proptest-state-machine REAL: prop_state_machine!{ sequential 1..200 =>
      ActualSystem } at src/dst.rs:485-490 with full ReferenceStateMachine
      impl on FsmReferenceModel + StateMachineTest impl on ActualSystem
    - dev-deps: proptest=1.11 + proptest-state-machine=0.8
    - Cases hardcoded to 256 — spec's 10K+ NOT enforced (no PROPTEST_CASES
      env wiring); spec's check_determinism(seed, config, fn) API never called
    - madsim NOT integrated: workspace-wide grep returns zero hits, zero
      cfg(madsim) blocks
    - tests/ipc_simulation.rs uses HOMEGROWN DstIpcGateway + SimFault::
      NetworkPartition/AgentCrash over SimulatorRuntime's fault queue —
      behavioural proxy, not madsim's NetSim/clog_link
    - Metrics file's madsim_compile_time_sec = 83.0 is FABRICATED
      (no madsim code = no madsim-cfg compile cost)
    - Test names match spec greps (ipc_packet_loss, ipc_asymmetric,
      ipc_agent_crash, ipc_burst_loss) but implementation is not the spec's
      mechanism
    - Adversarial scenarios YES (anti-spiral, evidence gate, context budget,
      SSB concurrent writes, demonic scheduling) at tests/adversarial_state_space.rs

----------------------------------------------------------------------
PHASE 3J — vLLM + advanced providers + plugins + orchestrator hardening
----------------------------------------------------------------------
  Built:     PARTIAL (9/21 sub-steps in production code)
  Wired:     SEMI — 4 of 6 WIRE.* cosmetically present but functionally compromised
  Dashboard: PARTIAL (~6/25 producible signals reach UI)
  Per WIRE.* status:
    FOUNDATION: 4/5 functional (Ssb, Fsm, tracer, persist); CliInteractiveGate
                COSMETIC (defined at cli/main.rs:7354, ZERO instantiations;
                team's deferral doc admits "dead code")
    BUDGET:     functional in CLI REPL; COSMETIC in TaskRunner path (5
                profile=None sites at task_runner.rs:1842, 6448, 6565, 6759, 7513)
    RULES:      functional (DefaultRuleEngine + 41 seed rules + ProjectRuleSeeder
                wired into pre-tool hook)
    SESSION:    functional for writes (SessionDag::new, save_snapshot, trim_context,
                create_branch); COSMETIC for stub recovery (stub_ssb_seq:None
                hardcoded)
    PLAN:       functional for /plan + budget Slot 2; propagate_changes not verified
    STUBS:      functional in code (TreeSitterAstEvaluator + DkgQuerierAdapter
                wired); COSMETIC at runtime — IndexingService writes
                .elai_cc/indexes/dkg.db but rule engine reads .elai/sessions/dkg.db.
                Adapter silently falls through to EmptyDkgQuerier permanently
  Notes:
    - Sidecar uses model.generate(), NOT manual token loop. server.py:1356
      and :1470 call self.model.generate(**generation_kwargs) inside
      DoLaSteeringProcessor + ETCLogitsProcessor. FastAPI + uvicorn.
    - MCP bridge: DefaultMcpBridge::connect/list_tools/call_tool are
      placeholders BUT batch-ful-d added a real HTTP JSON-RPC 2.0 client
      (http_initialize, http_list_tools, http_call_tool with deny-list +
      sanitization at L274-400) — consumed by dashboard only. CLI does not
      use MCP bridge.
    - elai.bat defaults to --dashboard, not REPL. Correct REPL invocation:
      elai.bat repl "hello".
    - Once invoked correctly, path LiveCli::new_local → ConversationRuntime
      → RuntimeClient(Local) → sidecar /v1/messages → DoLaSteeringProcessor
      → model.generate → SSE → render IS WIRED.
    - 8 sub-steps explicitly DEFERRED per
      workflow/reasoning/phase-3J.13-deferrals.md.
  Dashboard:
    - ProviderVram drawer shows real VRAM tier snapshot (one-shot per task,
      not real-time gauge)
    - MCP bridge UI works with feature="mcp" + ELAI_ENABLE_MCP=1 (read +
      toggle + list_tools work; call_mcp_tool exposed but UNUSED — no Svelte
      component invokes it)
    - model_swap events BARELY visible (only writer is task_runner.rs:4784
      hardcoding gpu_id=0, swap_duration_ms=0)
    - StatusBar inline optSummary is pre-formatted by sidecar
      OptimizationSummary::to_status_line — gated behind debug.enabled
      (most users never see it)
    - AntiStalenessDrawer is graceful-empty stub (DEF-T3-6: no SSB row
      types for dep_audit / baseline_harness / deterministic_audit yet)
    - Steering vector recompute is the most-functional end-to-end 3J path
      on the dashboard

----------------------------------------------------------------------
PHASE 3K — RAG production hardening
----------------------------------------------------------------------
  Built:     PASS (all 8 sub-steps real with measured metrics)
  Wired:     WIRED
  Dashboard: PARTIAL (per-tier R@5/R@10/MRR not surfaced; HyDE hypothesis
             not surfaced; query expansion variants not surfaced)
  Notes:
    - AST chunking REPLACES 50-line windows: try_ast_chunk runs FIRST in
      chunk_file (bm25.rs:380) and make_chunks_from_content (bm25.rs:464);
      50-line windowing preserved as spec-mandated fallback (line_block_chunks
      invoked only when AST returns empty). Tree-sitter coverage:
      Rust/Python/Go/TypeScript/Java/C#/Ruby. Tantivy schema carries chunk_kind.
    - HyDE WIRED to sidecar: HydeClient trait in knowledge/src/hyde.rs:52,
      production impl SidecarHydeClient in cli/src/sidecar_hyde.rs:38 bridges
      sync trait to InferenceProvider via dedicated tokio runtime. Constructed
      at cli/main.rs:3774-3781. Spec constants verbatim (max_tokens=128,
      temp=0.7, prefix "Write a short technical paragraph answering: ",
      5-min cache TTL, 3s timeout). DEEP tier only. Vector stage uses
      hypothesis text for embedding (pipeline.rs:270-273); cross-encoder
      reranks against ORIGINAL query (pipeline.rs:474).
    - Recall@10 / MRR MEASURED:
      metrics/phase_3K_rag.toml + phase_3K_rag_improved.toml contain real
      numbers. Baseline overall: R@5=0.6333, R@10=0.7667, MRR=0.4117 across
      30 queries. Per-tier latency budgets all met.
    - Cross-encoder ms-marco-MiniLM ONNX NOT staged in .elai_cc/models/ —
      reranker degrades to RRF-only
    - DEEP-tier MRR (0.206) BELOW STANDARD (0.575) — violates spec 3K.4
      "DEEP>=STANDARD>=FAST" monotonicity
    - Prior-audit "dead pipeline" claim DEBUNKED on current source: query()
      forwards to query_with_tier(Auto); vector/splade/pagerank/catrag/arag
      stages all execute when use_vector is true (Standard+Deep), per
      pipeline.rs:268-455.

----------------------------------------------------------------------
PHASE 4A — Agent process isolation (Tier C)
----------------------------------------------------------------------
  Built:     PARTIAL — skeleton/types real (~70%), production wiring hollow (~25%)
  Wired:     COSMETIC — separate process YES, isolation NO
  Dashboard: INVISIBLE (sandbox tier badge; IPC frame stats; SO_PEERCRED;
             AgentHeartbeat liveness; ToolFilter rejections; AgentDispatchConfig;
             IsolationMetric all not surfaced)
  Notes:
    - Separate processes YES: dashboard at dashboard/src/lib.rs:1191 spawns
      elai_cc_agent via AgentPoolBabyStepExecutor → agent_pool.launch_agent_process.
      Agent binary (elai_cc_agent) correctly has zero LLM/network deps
      (Cargo.toml only serde + postcard + tracing + uuid + elai-ipc-types) —
      true minimal IPC client.
    - NO isolation primitives: launch path uses plain
      std::process::Command::new(agent_binary_path).spawn() at agent_pool.rs:291-300.
      No clone(), no seccomp, no AppContainer, no rlimits, no bind-mounts,
      no /proc masking. Functionally Tier A++.
    - WindowsAppContainerSandbox at sandbox.rs:326-693 with real
      CreateAppContainerProfile, CreateRestrictedToken, Job-object,
      documented-weaker fallback — UNWIRED (production sets
      enable_sandbox: false at dashboard/src/lib.rs:1183, task_runner.rs:2477,
      agent_pool.rs:153). Even if true, launch_agent_process does NOT call
      self.sandbox.launch_agent() — sandbox handle created and stored but
      never used.
    - IpcGateway types (1+4+postcard, 15 message variants, MAX_FRAME_SIZE=10MB,
      full routing/GRC/grounding/ToolFilter logic) EXIST. NO UnixListener/
      NamedPipeServer accept loop in crates/orchestrator/src/. Source
      self-admits "Production wiring scheduled in Session wire-4A".
    - phase_complete SSB action + PhaseCompletePayload schema match spec
    - SO_PEERCRED helper coded but never called
    - bind-mounts/proc-masking/setrlimit/RLIMIT_AS completely absent from sandbox.rs
    - "Isolation" tests like test_4a_isolation_pid_namespace just construct
      a struct and assert a field — they do not spawn a process and check
      /proc/{pid}/status NSpid
    - phase_4A.toml metrics file measures the WRONG things (in-process
      postcard round-trip labeled as IPC latency; struct size labeled as
      memory-per-agent; HashMap insert labeled as agent spawn time)
  Effective tier: A++ (in-process logic + separate process exec, but shared
  namespaces / syscalls / network / fs / proc / limits).

----------------------------------------------------------------------
PHASE 4B — Tools (git, validation, trace, edit matching, compilation)
----------------------------------------------------------------------
  Built:     PARTIAL — core works through different architecture than specced
  Wired:     DUAL — live path uses InProcessToolDispatcher; IpcToolRegistry dead
  Dashboard: PARTIAL (git_commit + recent_commits SURFACED; failure_classified
             rows surfaced; daira_trace marker rows surfaced; integration
             checkpoint per-level SUCCESSES are silent — only failures emit)
  Per-deliverable:
    - ShortCoder R1-R10 all 10 rules implemented in crates/knowledge/src/shortcoder.rs
      (RuleId enum: CollapseBodies, StripLineComments, StripBlockComments,
      StripDocstrings, ElideObviousTypes, CollapseImports, TruncateStrings,
      RemoveEmptyLines, StripDecorators, TrimTrailingWhitespace) — minor scope
      drift (lives in knowledge crate not tools)
    - IntegrationTestRunner (4B.6-4B.7) WIRED with naming drift:
      DecompositionLevel { BabyStep, Step, Phase, Concept } +
      CommandIntegrationRunner with EXACT 5/30/60/120s timeout constants
      (tools/src/tiered_validation.rs:552-555). Triggered by orchestrator at
      task_runner.rs:3534 (run_integration_checkpoint after each baby-step
      outcome). Result struct lacks test_count + stdout_hash fields.
    - Git auto-commit triggered per BABY STEP (not per phase). task_runner.rs:3723-3738
      calls run_git_commit_dispatch → dispatcher.dispatch_git_commit →
      git_commit_blocking (real git2 0.20.4 native ops). Triggers when
      outcome_passed && !changed_files.is_empty(). Branch policy
      elai/plan-{plan_id} works. Deterministic message format. WIP-commits-
      for-large-phases (>5 edits) NOT implemented.
  Critical drift:
    - DAIRA REDUCED to 2 markers ("enter"/"exit") instead of specced 15.
      AdaptiveTool<T> wrapper + MarkerReport not present.
    - IpcToolRegistry + *Handler types in tools/src/ are DEAD CODE in
      production — orchestrator wires parallel InProcessToolDispatcher
      (elai_core::ToolDispatcher) instead. FSM filtering and TestingFirst
      gate live on the dead path.
    - Failure taxonomy DRIFT: spec demanded 9 categories, impl has 13 with
      mapping mismatches. Missing dedicated PERMISSION_DENIED;
      SYNTAX_ERROR/TYPE_ERROR/IMPORT_ERROR collapsed into CompilationError.
    - Edit matching 3 tiers WORK but SequenceMatcher similarity hint on
      failure missing.
    - File outline mode WORKS (>300 lines threshold) but force_full flag and
      50%-budget guard absent at tool layer.

----------------------------------------------------------------------
PHASE 4C — Plugins (hooks + tiered static analysis)
----------------------------------------------------------------------
  Built:     PARTIAL — module surface and contracts real (40+7 tests green
             per phase_4C.toml)
  Wired:     PARTIAL — TWO PARALLEL HOOK SYSTEMS (one dead, one live)
  Dashboard: SURFACED for ToolsHooksDrawer (real get_hook_verdict_log via
             SsbActionType::HookEvent rows from live InProcessHookDispatcher);
             PARTIAL — no T1/T2/T3 latency, no per-event filter, no live pulse
  Notes:
    - IpcHookDispatcher (plugins/src/ipc_hooks.rs) — ZERO non-test callers.
      IpcGateway::run_pre_tool_use_hook/run_post_tool_use_hook are dead helpers.
      Doc-comment at ipc_hooks.rs:14-17 admits "Production wiring is scheduled
      in Session wire-4A".
    - InProcessHookDispatcher (orchestrator/src/hook_dispatcher.rs) fires from
      task_runner.rs at baby-step boundaries (NOT IPC frames):
        PreToolUse runs against natural-language task_summary at task_runner.rs:3338
        PostToolUse runs per-changed-file at 3589 with no ruff
        Stop fires at 3877 as TELEMETRY ONLY — does NOT enforce non-IDLE block
        PreCompact at 3898 is explicit MVP no-op
    - ShortCoder WIRED in PostToolUse: task_runner.rs:3633-3651 invokes
      dispatcher.shortcode after dispatch_post_tool_use and before auditor
      reads, with ELAI_DISABLE_SHORTCODER + ELAI_DISABLE_SHORTCODER_POSTOOLUSE
      kill-switches and SSB row emission
    - Error sanitization WORKS: HookResult::sanitized_agent_message returns
      one of three constants ("Tool call rejected by policy" / "Tool call
      warning issued" / empty), verified by c11-c15 + adv01-adv02 with
      explicit negative assertions on rule_id, pattern_hash, regex text,
      tier label, locking session id. Live path uses sanitized_block_message
      (category) with 64-char lowercase-ASCII filter to block escape-sequence
      injection.
    - Spec-mandated Tier C IPC interception is dead code; ruff subprocess
      and SCHEMA_REQUIRED escalation missing; no Criterion latency benches
  Pick one path; delete the other.

----------------------------------------------------------------------
PHASE 4D — Pipeline Factory L1 (Designer + FlashResearch + Recursive
            Decomposition + Web Research)
----------------------------------------------------------------------
  Built:     SUBSTANTIALLY IMPLEMENTED — STRONG (all 18 sub-steps real;
             test counts EXCEED spec minimums 33/14, 16/13, 34/8)
  Wired:     WIRED at task_runner.rs:3083-3143 (production)
  Dashboard: PARTIAL — coarse research-step list in ResearchDrawer + bubble_up
             Finding rows in activity feed + phase-completion progress; series
             of stub viewers always returning zero/empty/NotFound for
             ConceptDecomposition, PackageRef, TechStackDecision, etc.
  Notes:
    - Research Agent runs as TIER C process: separate elai_cc_agent binary at
      crates/agent/, own [[bin]] target, dependencies limited to serde/postcard/
      tracing/uuid/elai-ipc-types — no reqwest, no providers. Spawned via
      Stdio::null/piped from agent_pool.rs:291. All LLM calls proxied via IPC
      InferRequest/InferResponse.
    - Designer agent abstraction PARTIALLY BYPASSED: PipelineSpec IS produced
      in production (spec_from_decomposition at task_runner.rs:6044, with real
      PerfSpec, MediaSpec, OutputType). HOWEVER production path calls
      research_decompose with SidecarGroundingClient directly rather than
      dispatching through agent pool as Designer agent.
    - Recursive decomposition (4D.7-4D.10) IMPLEMENTED:
      ConceptDecomposition/ConceptNode/PhaseNode/StepNode/BabyStepSpec/
      BabyStepGrounding/TechStackDecision/PackageRef/ResearchCache all defined
      at research_tree.rs:1245-1502. research_decompose, extract_concepts,
      extract_phases, extract_steps, decompose_to_baby_steps, build_grounding
      all present and async. ResearchCache has real LRU eviction.
      4 levels: Idea → Concepts → Phases → Steps → BabySteps.
    - KEY GAP: verify_packages does string-shape plausibility only, NOT real
      HTTP API_REGISTRY_CHECK against crates.io/pypi/npm — anti-hallucination
      gate goal partially met. NetworkVerifier::ApiRegistryCheck explicit stub.
    - Web research (4D.11-4D.18) IMPLEMENTED:
      web_search/ crate with SearxClient (client.rs:147), WebFetcher
      (fetcher.rs:35), ContentExtractor + BasicExtractor (extractor.rs:145),
      SourceQualityTable + SourceTier (source_quality.rs), RobotsCache
      (robots.rs:64), RateLimiter with injectable mock clock. Constants match
      spec exactly (10s timeout, 20 max results, 1000ms rate limit).
      Docker script start-searxng.sh present (container name mismatch:
      script "searxng" vs spec "elai-searxng").
      WebResearchOrchestrator (web_research_orchestrator.rs, 710 lines) wires
      it all with FORMULATE/SEARCH/EXTRACT/REFLECT loop.
      DefaultWebResearchRunner production-wired at task_runner.rs:1199.
      6 VRAM tiers in research_tier.rs (8/12/16/24/48Gb + Cloud) with
      speculative_depth=2 only on 48Gb.
      ResearchBudgetManager (research_budget.rs) with can_admit/admit/
      compress_evidence.
      phase_4D_web_research.toml has all 10 queries with all 11 required
      metrics (transparently labels tree internals as SIMULATED until
      orchestrator is callable from bench harness — honest disclosure).
      elai_cc.toml [research.trusted_domains] tier1 has >100 domains.

----------------------------------------------------------------------
PHASE 4E — Pipeline Factory L2 (Builder + DAG repair + DecompositionTree)
----------------------------------------------------------------------
  Built:     IMPLEMENTED (all 12 sub-steps materialized; 2 minor metrics gaps)
  Wired:     WIRED — drives production loop in task_runner.rs:3176-3550+
  Dashboard: INVISIBLE for the typed structures (ConceptDecomposition tree,
             ExecutablePipeline DAG, FailureRecord, BubbleUpEvent typed,
             IntegrationCheckpoint successes, RepairAction distribution all
             not surfaced); PartialPlanTree polls PhaseComplete only
  Notes:
    - DecompositionTree types in pipeline_tree_navigator.rs (NavigatorTree,
      TreeNavigatorState, BubbleUpAction, BubbleUpPolicy, DecompositionLevel,
      BabyStepResult, BabyStepOutcome, BubbleUpEvent, FailureRecord,
      IntegrationCheckpoint, TreeAutoAdvancePolicy in autonomous_gate.rs)
    - DepthFirstNavigator implements TreeNavigator trait (object-safe)
    - Drives production loop: NavigatorTree::new → DepthFirstNavigator::from_tree
      → loop on nav.next_baby_step → nav.record_result → integration checkpoint
      → next
    - ExecutablePipeline returned from PipelineBuilder::build at task_runner.rs:3143
    - BubbleUpHandler triggers in production:
      handle_failure (pipeline_tree_navigator.rs:593-713) implements the full
      cascade: 3 baby step fails → ResearchAlternative; 2 step fails →
      RedecomposeStep; 1 phase fail → ReplanPhase; 1 concept fail →
      ReplanConcept; final → Escalate. task_runner.rs:3505 emits "bubble_up"
      Finding when nav.record_result returns Some(action).
      Integration-checkpoint failures separately bubble via
      emit_integration_bubble_up at lines 5926/5935/5944/5966/5974/5982.
      Thresholds: BabyStep=3, Step=2, Phase=1, Concept=1.
      BubbleUpAction enum has 6th DiagnoseAndRetry variant beyond spec 5
      (additive, harmless).
    - DAG repair fires when validation fails:
      PipelineBuilder::build at pipeline_builder.rs:362-376 unconditionally
      invokes DagRepairLoop::repair whenever StructuralValidator::validate
      returns errors. T_DAG=10 enforced (constant L40, passed at L363).
      REORDER (back-edge removal) for cycles + SUBSTITUTE (find_substitute)
      for missing tools. Fixpoint detection via blake3 structural_hash
      comparison (lines 672-680) — escalates immediately if graph unchanged.
      Non-repairable kinds (ResourceOverflow, UnmeasurableMetric,
      InvalidPredicate, MissingAdapter, InvalidVisualCheck, MissingDependency)
      escalate immediately without burning rounds.
      repair_log surfaced in ExecutablePipeline + logged at task_runner.rs:3149.
    - Apply_writes is documented stub awaiting agent ChannelWrite emission
    - recover_from_checkpoint logs-only (state restoration deferred)
    - phase_4E.toml has zeroed timings; tree-level metrics computed in
      examples/measure_4e_tree.rs but NOT persisted into the toml

----------------------------------------------------------------------
PHASE 4K MVP (4K.1 through 4K.5)
----------------------------------------------------------------------
  Built:     SHIPPED (4K.1 PASS, 4K.2 PASS w/ severe scope creep, 4K.3 PASS,
             4K.4 PARTIAL PASS, 4K.5 PARTIAL PASS)
  Wired:     SHIPPED — but the spec MVP is BURIED under post-MVP shell
  Dashboard: this IS the dashboard
  Stack confirmed:
    - Tauri v2 (tauri = "2.10", tauri-build = "2.5", tauri-plugin-dialog = "2.7")
    - identifier "dev.elai.dashboard", 1200x800
    - Svelte 5 runes ($state, $props, $derived) + @sveltejs/vite-plugin-svelte ^5.0.0
      + vite ^6.0.0 + @tauri-apps/api ^2.1.1
    - No Axum, no HTTP/REST/WebSocket on dashboard boundary
  Spec MVP scope (4K.1 through 4K.5):
    1. SSB read-only viewer library (Rust, data extraction from SQLite WAL)
    2. Tauri scaffold
    3. Project selector + task input
    4. Workspace shell (sidebar + dual panels) + plan tree + activity feed
    5. Diff viewer + preview link
  Deferred 4K.6-4K.13 features SNEAKING IN ALREADY (per spec these should NOT
  be present in MVP):
    - Sidebar with 5 mode icons + Ctrl+1-5 keymap
    - Window management with tabs, Ctrl+W close, Ctrl+Shift+H last-closed-stack
    - 9 polymorphic windows: Chat, Activity, Explore, Monitor, Plan, Preview,
      Research, Review, Settings (spec MVP wanted only the workspace)
    - 10 drawers (AntiStaleness, Knowledge, Orchestration, ProjectIntelligence,
      ProviderVram, Reasoning, Research, Rules, ToolsHooks, WebQuality)
    - Slash command palette + Ctrl+. trigger
    - Full keyboard shortcut grid
    - Autonomous-mode UI (gate-mode polling, autonomy strip, AutoAdvanceStats)
    - Overlay system with backdrop + window-selector + slash-palette
    - ProposalsPanel (Phase 4H proposal flow)
    - Plus 90+ Tauri commands wiring drawers, project intelligence, MCP bridge,
      override file writers, dev-server registry, BM25 stats, etc.
  Visual identity gap (root cause of "doesn't feel like wanted"):
    - Body font: Inter (--font-ui), spec mandates JetBrains Mono throughout
    - Type scale +1-2px on every step (xs/sm/base/lg/xl 11/13/14/16/20 vs
      spec 10/12/13/15/18) — UI looks ~25% larger than reference
    - bg-void #0b0d12 vs spec #0a0e12; text colors lighter
    - Two competing shells coexist: Tier-0 Shell (sidebar+dual-PanelHost) is
      where the user lands; spec MVP Workspace exists but reachable only as
      the "PLAN window" inside Shell
    - Activity feed buried under autonomy strip + feasibility panel +
      violations strip + proposals panel + intervention cards
    - Activity feed missing: auto-scroll, Jump-to-Latest pill, inline
      [preview] entry with teal link
    - Status bar lives in Shell, not in Workspace (spec required it inside)
    - DiffViewer per-hunk hover controls and multi-file vertical nav thin
    - dashboard-wiring-plan.txt + dashboard-wiring-inventory.txt are
      retroactive re-spec into "Tier 0-4" tracks — should not be treated
      as authoritative
  HEALTH_SNAPSHOT and WINDOW_STATE SSB action types CORRECTLY ABSENT
  (deferral boundary respected at SSB schema level only).


================================================================================
A-STEPS COMPLETION SNAPSHOT (per user: A-0 through A-48 done)
================================================================================

Per the original implementation order, A-1 through A-48 should be complete.
Audits confirm representative samples:
  A-1 BASELINE HARNESS: DONE (results captured)
  A-3 model.generate MIGRATION: DONE (sidecar uses model.generate + LogitsProcessor)
  A-6 VERSION_GATE: DONE (3F audit confirms predicate + table + 21 seed entries)
  A-10 CONTEXT BUDGET ADAPTATION: BUILT (ModelCapabilityProfile machinery
    real) but DEAD AT RUNTIME (TaskRunner passes profile=None)
  A-22 RESEARCHING FSM STATE: DONE (additive Researching state at fsm.rs:24-38)
  A-24 SHORTCODER: DONE (R1-R10 in knowledge crate; wired in PostToolUse)
  A-44 PROJECT INDEXING WIRING: DONE (IndexingService + SSB events; WIRED for
    dashboard-side; CLI-side has dual-path bug)
  A-45 SSB DASHBOARD DATA EXTRACTION: DONE (SsbViewer real)
  A-46 AXUM/TAURI SERVER: DONE (Tauri shipped, NOT Axum)
  A-47 PROJECT SELECTOR + WORKSPACE SHELL: DONE
  A-48 CHAT + REVIEW WINDOWS: DONE
  
Note: A-49 through A-50 (ACTIVITY + PLAN windows) marked DONE per user but
audit shows ActivityWindow real, PlanWindow.svelte built but UNREGISTERED.

A-51 through A-57 (FOCUS TUNNEL): per post-phase4-implementation-order.txt
these are Group E. Not audited individually here. Implementation status
appears MIXED based on the dashboard's heavy Tauri command surface.


================================================================================
PART V KNOWLEDGE-ENGINE EXTENSIONS (B-29..B-35) — STATUS SNAPSHOT
================================================================================

Added 2026-05-05. All NOT STARTED. These are net-new features layered
on top of the already-complete A-0..A-48 + Phase 0..4E knowledge engine
following SocratiCode-pattern evaluation. Listed here for inventory
completeness — they belong in the "what could be next" column, not the
"what's currently built" column.

----------------------------------------------------------------------
B-29 — Symbol-Level Impact Analysis API
----------------------------------------------------------------------
  Built:     NOT STARTED
  Wired:     N/A
  Dashboard: N/A (will surface as KnowledgeDrawer Impact Analysis panel)
  Effort:    2-3 sessions (~12-18 hours)
  Pre-req:   B-1 (path unification)
  Notes:
    - SymbolGraphCache: 27 name shards + 256 reverse-call file shards +
      500-file LRU per-file payload cache
    - 4 traversal tools: compute_impact_radius / compute_call_flow /
      compute_symbol_context / list_symbols
    - Confidence tiers: Local / Unique / MultipleCandidates / Unresolved
    - Auto-injected into Auditor PostToolUse hook for IMPACT_SUMMARY
    - Schema migration v3 for symbol_name_index, symbol_reverse_index,
      symbol_file_payload, symbol_graph_meta tables in dkg.db

----------------------------------------------------------------------
B-30 — Multi-Language DKG Hardening (closes A-35)
----------------------------------------------------------------------
  Built:     NOT STARTED
  Wired:     N/A
  Dashboard: N/A (KnowledgeDrawer language status panel planned)
  Effort:    3-4 sessions (~18-24 hours)
  Pre-req:   B-1
  Notes:
    - 17+ language extractors (Python, TS, TSX, JS, Go, Java, Kotlin,
      Scala, C, C++, C#, Ruby, PHP, Swift, Bash, Svelte, Vue)
    - Per-grammar failure isolation via std::panic::catch_unwind
    - JVM suffix map, C# namespace map, Go module info from go.mod,
      TypeScript path aliases with extends-chain
    - Svelte/Vue HTML-then-TS re-parse for composite languages
    - Closes "system works for any language" gap; ELAI is Rust-only in
      practice today

----------------------------------------------------------------------
B-31 — Resumable Indexing + Cross-Process Lock + Startup Auto-Resume
----------------------------------------------------------------------
  Built:     NOT STARTED
  Wired:     N/A (will wire from cli::main + dashboard::run on startup)
  Dashboard: N/A (will replace static IndexingProgressPanel with live)
  Effort:    2 sessions (~10-14 hours)
  Pre-req:   B-1
  Notes:
    - 50-file batches with blake3 content-hash skip on resume
    - Schema migration v2 for project_metadata + file_hashes tables
    - IndexLock with LockScope::Index | Watch using fs2 (advisory file
      locking) — verify already in workspace; if not, ASK USER before
      adding
    - Stale-PID reclaim with SIGTERM (POSIX) / OpenProcess (Windows)
    - 5 new SSB action types: IndexCheckpoint, IndexResume, IndexCancel,
      LockAcquired, LockContested
    - graceful_shutdown with 60s deadline before force-release of locks
    - migrateAbsolutePathKeys helper for one-shot post-B-1 cleanup

----------------------------------------------------------------------
B-32 — Adaptive Chunking: Minified File Detection + Universal Char Cap
----------------------------------------------------------------------
  Built:     NOT STARTED
  Wired:     N/A
  Dashboard: N/A
  Effort:    0.5-1 session (~3-5 hours)
  Pre-req:   B-1
  Notes:
    - is_minified(content, line_count) detects avg_line_length > 500
    - chunk_by_characters with safe-boundary backwards-scan (newline /
      space / tab / `;` / `,`)
    - apply_char_cap with MAX_CHUNK_CHARS=2000 applied to ALL chunker
      outputs (AST + line-window + character-based) as last step
    - UTF-8 char-boundary safety required (no mid-codepoint panics)
    - New SSB action: MinifiedFileChunked
    - Cheapest entry in Part V — half a session

----------------------------------------------------------------------
B-33 — Interactive Decomposition + Graph Viewer (Mermaid + cytoscape)
----------------------------------------------------------------------
  Built:     NOT STARTED
  Wired:     N/A (new GraphWindow + Mermaid export endpoint)
  Dashboard: NEW WINDOW (Ctrl+G keybinding planned)
  Effort:    2-3 sessions (~12-18 hours)
  Pre-req:   B-7 + B-29
  Notes:
    - VizData contract mirrors SocratiCode's
      (VizFile / VizFileEdge / VizSymbol / VizSymbolEdge)
    - 3 modes: File / Symbol / Decomposition
    - 6 layouts: dagre, fcose, concentric, breadth-first, grid, circle
    - Right-click blast-radius overlay via get_impact_overlay
    - PNG export + Mermaid text export (paste-into-PR workflow)
    - Cross-window links from Plan tree + ActivityFeed
    - PENDING USER CONFIRMATION: cytoscape, cytoscape-dagre,
      cytoscape-fcose npm deps (~500KB to dashboard webview bundle)

----------------------------------------------------------------------
B-34 — Entry-Point Detection Tool
----------------------------------------------------------------------
  Built:     NOT STARTED
  Wired:     N/A
  Dashboard: N/A (entry points displayed via B-33 GraphWindow + new
             /entry_points slash command)
  Effort:    1 session (~5-7 hours)
  Pre-req:   B-29
  Notes:
    - 4 heuristics: orphans + conventional names + framework routes
      (HTTP verbs + route) + test-name patterns
    - 14-language ENTRY_POINT_NAMES table (configurable per project)
    - Auto-injected into Designer's ExistingProject decomposition path
      (EXISTING_ENTRY_POINTS briefing block)
    - Auto-injected into Researcher's Oracle pipeline
      (PROJECT_STARTING_POINTS formulation seed)
    - Performance budget: <500ms p99 on 10K-file project via payload
      selectivity
    - New SSB action: EntryPointsDetected

----------------------------------------------------------------------
B-35 — Context Artifacts Pipeline
----------------------------------------------------------------------
  Built:     NOT STARTED
  Wired:     N/A (parallel indexing pipeline alongside code DKG/BM25/vec)
  Dashboard: NEW PANEL (ArtifactsPanel.svelte planned)
  Effort:    2 sessions (~10-14 hours)
  Pre-req:   B-1 + B-31 + B-32
  Notes:
    - [[knowledge.artifacts]] block in elai_cc.toml: name + path +
      description (steers LLM, written as "check this before X")
    - Per-project context_{project_id} collection (separate from code
      collections; never crosses)
    - SQL → CREATE-statement-boundary chunking; YAML/MD/JSON → reuse
      B-32 line-window chunker
    - blake3 staleness on whole-file content
    - ArtifactStage in QueryPipeline (STANDARD + DEEP only; FAST is
      BM25-code-only)
    - GroundingMethod::ArtifactCheck added as 7th GRC method (FILE,
      SOURCE, TEST, WEB, API, API_REGISTRY, ARTIFACT)
    - Designer's plan sweep uses ArtifactCheck for project-typed claims
    - AVAILABLE_ARTIFACTS Designer briefing block when ExistingProject
    - 3 new SSB actions: ArtifactsIndexed, ArtifactSearched, ArtifactStale
    - Pre-investigation undervalued this — 575 LOC SocratiCode reference
      is a complete second pipeline, not a config block

----------------------------------------------------------------------

PART V GATE: Auditor calls compute_impact_radius pre-edit on every
Write/Edit; multi-lang DKG operates with unresolved_edge_pct < 50%
on real Python/TS/Go/Java repos; full re-index of 50K-file project
survives mid-run kill -9; minified bundles index without provider
truncation; GraphWindow renders ELAI's own crate dependency graph
with right-click blast radius working; Designer's ExistingProject
decomposition references real entries + AVAILABLE_ARTIFACTS.


================================================================================
END PART 2
================================================================================

Continue to DETAILED_SUMMARY_03_gaps_and_roadmap.md for the gap analysis,
risk ranking, and the rationale behind the B-plan ordering.
