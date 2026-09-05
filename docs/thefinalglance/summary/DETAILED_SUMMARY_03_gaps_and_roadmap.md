# ELAI_CC — Detailed Summary, Part 3: Gaps and Roadmap (B-plan rationale)
Date: 2026-04-25
Predecessors: DETAILED_SUMMARY_01, DETAILED_SUMMARY_02
This file: gap synthesis + risk ranking + the rationale for B-1 through B-28.

The B-plan files themselves live in: F:\.ELAI_workflow\docs\thefinalglance\B-plan\


================================================================================
1. THE CORE GAP (one sentence)
================================================================================

The system is BUILT but UNDER-WIRED. The dashboard renders ~11% of the
backend's signal volume; reasoning + safety gates exist but never fire in
production; database paths disagree across CLI and dashboard; ~1500+ LOC of
dead code from the fork was never stripped; multiple parallel implementations
(hooks, tools, sessions, agent configs) coexist with one path always dead.

Resolution requires NO new features for the MVP — only wiring, deletion, and
visual restoration. The "final product" (4F-4J + cloud + focus tunnel) layers
on top after wiring is solid.


================================================================================
2. RISK RANKING (KILL-LIKELIHOOD ORDERED)
================================================================================

  Rank  Risk                                   Severity  Phase  Fix in B-plan
  ----  -----------------------------------    --------  -----  -------------
   1    Dashboard "doesn't feel like wanted"   HIGH      4K     B-2,3,4..14
   2    Dual database paths (.elai vs .elai_cc) HIGH      1A    B-1
   3    Reasoning gates vacuously pass          HIGH      2B-2D  B-8
   4    PipelineGrc unwired in production       HIGH      2A-2D  B-8
   5    SessionDag dead on arrival              HIGH      3H     B-9
   6    ContextBudget profile=None fallback     HIGH      3E     B-10
   7    file_overlap_score (3E.5) missing       HIGH      3E     B-10
   8    SearchMode cascade not wired            HIGH      1I     B-4
   9    SupervisionManager not wired            HIGH      3C     B-6
  10    CliInteractiveGate dead code            HIGH      3A     B-2
  11    Two parallel hook systems               MEDIUM    4C     B-19
  12    Two parallel tool dispatchers           MEDIUM    4B     B-20
  13    Tier C isolation cosmetic               MEDIUM    4A     B-18
  14    Phase 0 dead code not stripped          MEDIUM    0      B-15
  15    Anti-stale violations (ort rc, blake2,
        Box<dyn Error>, .unwrap, HashMap)       MEDIUM    0+1+   B-16
  16    Visibility filter leaks "filtered"      MEDIUM    3C     B-15+18
  17    Cross-encoder ONNX model not staged     MEDIUM    1G     B-4
  18    Vector ONNX models + path mismatch      MEDIUM    1C     B-4 (via B-1)
  19    DAIRA reduced to 2/15 markers           LOW       4B     B-7+19
  20    DEEP MRR < STANDARD MRR                 LOW       3K     B-4
  21    code_vec populated but never queried    LOW       1C     B-4
  22    Schema mismatches (Settings entries)    LOW       4K     B-13
  23    /continue placeholder, branch ops UI    LOW       3H     B-9
  24    Phase 0 baseline metrics TOML missing   LOW       0      B-15

PART V (NET-NEW EXTENSIONS, NOT RETROFITS — ranked by leverage):
  Rank  Gap                                    Severity  Phase  Fix in B-plan
  ----  -----------------------------------    --------  -----  -------------
  V-1   No symbol-level impact API exposed     HIGH      ext    B-29
        for Auditor pre-edit safety
  V-2   Indexing not crash-resumable           HIGH      ext    B-31
  V-3   DKG Rust-only in practice              HIGH      ext    B-30
        (closes A-35)
  V-4   No project-typed non-code knowledge    MEDIUM    ext    B-35
        (DB schema / OpenAPI / glossary)
  V-5   Minified files crash embed boundary    MEDIUM    ext    B-32
  V-6   No DAG / decomposition graph viewer    LOW       ext    B-33
        in dashboard
  V-7   Designer's ExistingProject mode        LOW       ext    B-34
        cannot ground in real entry points

Risks 1-10 are all UNWIRING failures that block MVP. Risks 11-16 are
hardening work. Risks 17-24 are polish.


================================================================================
3. WHAT THE DASHBOARD SHOULD SURFACE BUT DOES NOT
================================================================================

From the wiring aggregate, ~232 invisible producers. Top 30 by user-facing impact:

DASHBOARD UX (immediate visual gaps):
  1. Live FSM state badge in StatusBar (currently always "IDLE")
  2. EXECUTING-pulse animation on transitions
  3. JetBrains Mono throughout (currently Inter)
  4. 10/12/13/15/18 type scale (currently 11/13/14/16/20)
  5. Workspace as primary shell (currently Tier-0 Shell buries it)
  6. Activity feed auto-scroll + Jump-to-Latest pill
  7. Inline [preview] entry with teal link in activity rows
  8. Status bar inside Workspace (not in outer Shell)
  9. Live token streaming in dashboard chat (currently writes intent rows only)

KNOWLEDGE QUALITY (engine transparency):
  10. Live QueryTrace inspector (per-query stage timeline + latency)
  11. SearchMode current state + cascade transitions
  12. Per-model EmbeddingCircuitBreaker state grid (5 models)
  13. HyDE hypothesis preview + cache hit rate
  14. Per-tier Recall@5/Recall@10/MRR badges (real measured values exist)

REASONING (safety transparency):
  15. Per-agent GRC budget remaining gauge (real per role)
  16. GroundingSweep result panel (claims classified)
  17. CSQ guard prompts (Critical pending guard)
  18. Inline feasibility Pass/Fail/Blocked badges

ORCHESTRATION (control transparency):
  19. 7 EvidenceGate verdict panel
  20. InteractiveGate prompt UI (Approve/Reject/Modify/Redirect)
  21. Per-agent panel (status, lifecycle, death classification)
  22. SupervisionManager event log
  23. ContextBudget per-slot allocation visualizer

PIPELINE (decomposition transparency):
  24. DecompositionTree visualizer (Concept → Phase → Step → BabyStep)
  25. ExecutablePipeline DAG visualizer
  26. DAG repair events log (Reorder/Substitute distribution + T_DAG counter)
  27. Web research per-node FORMULATE/SEARCH/EXTRACT/REFLECT/CONVERGE log

INFERENCE (model transparency):
  28. Real-time VRAM gauge
  29. Per-step inference latency + tokens-per-sec
  30. vLLM /v1/models advertised list


================================================================================
4. THE 30-MINUTE FIX (B-2)
================================================================================

The single highest-impact patch in the entire B-plan. Activates 4 UI elements
simultaneously by wiring one missing producer:

  File: .elai_cc/crates/orchestrator/src/fsm.rs
  Site: Fsm::transition_with_metrics, after the existing state-update logic
        that's already at line 639.
  Patch (sketch):
  
    if let Some(writer) = self.ssb_writer.as_ref() {
        let _ = writer.write_payload(
            "fsm",
            0, // session-scoped, sequence assigned by writer
            SsbPayload::PhaseTransition(PhaseTransitionPayload {
                from: from_state,
                to: to_state,
                ts_ms: clock.now_ms(),
                reason: reason.into(),
            }),
            Vec::new(),
        );
    }

This activates:
  - StatusBar FSM badge (svelte:308-357 dead state machine becomes live)
  - EXECUTING pulse animation
  - MonitorWindow.currently text
  - ActivityEntry phase_transition row renderer (already exists)


================================================================================
5. RECOMMENDED B-PLAN ORDERING (28 STEPS, 4 PARTS)
================================================================================

PART I — MVP COMPLETION (make what exists actually work)
  B-1   Path Unification (.elai → .elai_cc everywhere; fix dual DKG/index DB)
  B-2   FSM PhaseTransition Wiring (the 30-min fix)
  B-3   Stub Command Eradication (delete or implement each of 30 dead UI elements)
  B-4   Producer Wiring — Knowledge engine signals (QueryTrace, SearchMode,
        breakers, freshness, recall/MRR badges, vector code_vec query, model
        staging, DEEP-tier MRR fix)
  B-5   Producer Wiring — Reasoning signals (per-agent GRC budgets, sweep
        results, CSQ guards, feasibility outcomes, role config viewer)
  B-6   Producer Wiring — Orchestration signals (evidence gates, agents panel,
        supervision, BSP visualization, ContextBudget viz)
  B-7   Producer Wiring — Pipeline factory (DecompositionTree visualizer,
        DAG repair log, research tree per-node events, IntegrationCheckpoint
        successes)
  B-8   PipelineGrc Activation (call verify_claim/ground_plan/generate_csq/
        check_inline_feasibility from production hot paths)
  B-9   SessionDag Activation (replace JSONL; populate stub_ssb_seq; expose
        branch ops in UI; SPEC AMENDMENT #2 SSB action_type='checkpoint')
  B-10  ContextBudget Profile Activation (pass real ModelCapabilityProfile
        from task_runner; implement file_overlap_score 3E.5)

PART II — DASHBOARD VISUAL RESTORATION
  B-11  Design System Restoration (JetBrains Mono, 10/12/13/15/18 scale,
        exact #0a0e12 / brand teal #1B7A6E / brand purple #7C5CBF)
  B-12  Tier-0 Shell vs Workspace Decision (consolidate; activate spec MVP
        Workspace as primary; Tier-0 Shell becomes optional power-user mode)
  B-13  Activity Feed Restoration (auto-scroll, Jump-to-Latest pill, inline
        preview links, fix Settings SessionHandoffEntry schema mismatch)
  B-14  Status Bar Inside Workspace + Live Token Streaming Chat

PART III — HARNESS HARDENING
  B-15  Phase 0 Debt Burndown (strip compact.rs, usage.rs ModelPricing,
        oauth.rs, remote.rs, prompt.rs; replace 84 HashMap imports; fix 40+
        entropy leaks; cli/main.rs decomposition; missing scripts)
  B-16  Anti-Stale Rule Enforcement (ort stable, no Box<dyn Error> across
        15+ public APIs, blake3 everywhere — spec text update too —
        .unwrap/.expect removal, .try_into() over as-casts)
  B-17  Ground the Vacuous Gates (ungrounded_claims producer; CSQ enforcement
        in pre-tool dispatch; feasibility blocks in EXECUTING; SearchMode
        cascade wired into query path)
  B-18  Tier C Sandbox Activation (clone3 + seccomp on Linux; AppContainer
        on by default on Windows; visibility filter "not found" semantics)
  B-19  Hook System Unification (delete InProcessHookDispatcher OR delete
        IpcHookDispatcher — pick one; expand DAIRA from 2 to 15 markers)
  B-20  Tool Registry Unification (same — pick one; failure taxonomy
        re-alignment to 9 spec categories)

PART IV — FINAL PRODUCT (4F-4J + remaining 4K + cloud + focus tunnel)
  B-21  4F Feasibility Probe (system probe + novel step stubs)
  B-22  4G Autonomous Mode (AutonomousGate + AutoAdvanceEvaluator +
        SUBSTITUTE_MODEL via ModelPool swap chain + SPLIT_TASK)
  B-23  4H Self-Modification (DGM/CEL/DarwinNet — 3 layers + MSI + safety)
  B-24  4I Media Validator (conditional — only when pipeline produces media)
  B-25  4J Empire Integration (conditional — when Empire codebase accessible)
  B-26  4K.6-4K.13 Dashboard Completion (MONITOR window real, EXPLORE window,
        intervention controls + notifications, window management + tabs +
        history, visual polish + animations)
  B-27  Cloud Mode (D-1 through D-4: config + provider routing + cost
        tracking; cloud context budget + baby step sizing; privacy redaction;
        HYBRID escalation + cost estimation)
  B-28  Focus Tunnel (E-1 through E-7: FIM + diff + mode selection + streaming
        validation + pipeline parallelism + dependency resolution + file routing)

PART V — KNOWLEDGE-ENGINE EXTENSIONS (post-A-48, evaluated 2026-05-05
against SocratiCode AGPL TS reference at 33,975 LOC; ELAI re-implements
patterns in Rust, not source-copied)

  B-29  Symbol-Level Impact Analysis API (★ highest leverage)
        SymbolGraphCache (LRU + 27 name shards + 256 reverse-call shards)
        backing 4 traversal tools: compute_impact_radius, compute_call_flow,
        compute_symbol_context, list_symbols. Auto-injected into Auditor
        PostToolUse for pre-edit IMPACT_SUMMARY.
  B-30  Multi-Language DKG Hardening (closes A-35)
        17+ language extractors with per-grammar try/catch isolation,
        JVM suffix map, C# namespace map, Go module info, TS path
        aliases, Svelte/Vue HTML-then-TS re-parse.
  B-31  Resumable Indexing + Cross-Process Lock + Startup Auto-Resume
        50-file batches with blake3 hash skip; IndexLock with stale-PID
        reclaim; cli + dashboard auto-resume on startup; graceful
        shutdown 60s deadline.
  B-32  Adaptive Chunking: Minified Files
        avg_line_length > 500 detection + character-based chunker with
        safe boundaries + universal MAX_CHUNK_CHARS=2000 cap on every
        path. Cheapest entry — 0.5-1 session.
  B-33  Interactive Decomposition + Graph Viewer (Mermaid + cytoscape)
        Tauri+Svelte+cytoscape window with file/symbol/decomposition
        modes, blast-radius right-click, search, 6 layouts, PNG export,
        plus Mermaid text export for paste-into-PR.
  B-34  Entry-Point Detection Tool
        4-heuristic union (orphans + conventional names + framework
        routes + tests). Auto-injected into Designer's ExistingProject
        and Researcher's Oracle.
  B-35  Context Artifacts Pipeline
        Second indexing pipeline parallel to code:
        [[knowledge.artifacts]] declares DB schemas / OpenAPI / k8s /
        ADRs / glossaries; per-project context_* collection; blake3
        staleness; ArtifactStage in QueryPipeline; 7th GRC method
        ArtifactCheck; AVAILABLE_ARTIFACTS Designer briefing block.

GATE FOR PART V: Auditor compute_impact_radius on every Edit;
multi-lang DKG on real Python/TS/Go/Java repos with unresolved_edge_pct
< 50%; full re-index of 50K-file project survives kill -9; minified
bundles indexable; GraphWindow renders ELAI's own crate dependency
graph with right-click blast radius; Designer's ExistingProject
decomposition references real entries + AVAILABLE_ARTIFACTS.


================================================================================
6. DESIGN FILES INDEX (READ BEFORE EACH B-N)
================================================================================

ORIGINAL SPECS (F:\.ELAI_workflow\docs\specs\):
  ELAI_COMPLETE_SUMMARY.txt (the file this expanded summary supersedes)
  architecture-overview.txt (high-level cross-crate design)

CRATE SPECS (F:\.ELAI_workflow\docs\specs\crates\):
  cli.txt, core.txt, knowledge.txt, orchestrator.txt, plugins.txt,
  providers.txt, reasoning.txt, rules.txt, tools.txt
  → for B-N spec lookups by crate, use these as ground truth.

INTEGRATION SPECS (F:\.ELAI_workflow\docs\specs\integration\):
  empire.txt, media-validator.txt, pipeline-factory.txt, prompt-architecture.txt,
  self-modification.txt
  → for B-21..25 these are the source of truth.

GUIDE — DESIGN DOCUMENTS (F:\.ELAI_workflow\docs\specs\guide\):
  anti-staleness-enforcement.txt        → B-16
  cloud-mode-design.txt                 → B-27
  code-completeness-enforcement-design.txt → B-19+20 (taxonomy, hooks)
  compiled-pipeline-design.txt          → optional Phase 5+ (compiled pipeline crystallization)
  dashboard-design.txt                  → B-11 through B-14, B-26
  deterministic-harness-design.txt      → B-16, B-19, B-20
  development-stones-analysis.txt       → cross-cuts B-1 through B-28
  focus-tunnel-design.txt               → B-28
  implementation-guide.txt              → cross-cutting reference
  pipeline-modes-design.txt             → B-7 (surgical mode), B-21
  post-phase4-implementation-order.txt  → groups C/D/E/F/H ordering
  project-intelligence-design.txt       → optional Phase 3L (after B-plan)
  reasoning-architecture-design.txt     → B-8, B-22 (escalation), B-23 (self-mod)
  testing-trust-design.txt              → B-2 (FSM emit), B-7 (wiring contract),
                                          ground truth for Layer 0 wiring discipline,
                                          essential for B-N WIRING_CONTRACT blocks
  token-efficiency-design.txt           → B-10 (ContextBudget profile)
  update-implementation-order.txt       → A-1..48 origin
  update-implementation-order-summary.txt → condensed A-step index
  update-phase-map.txt                  → which design touches which phase

GUIDE — PHASE INDEXES + STEPS (F:\.ELAI_workflow\docs\specs\guide\steps\):
  Lossless agent summaries (in this folder):
    phase-1-summary.md, phase-2-summary.md, phase-3-summary.md, phase-4-summary.md,
    integration-summary.md, crates-summary.md
  Original step files:
    pre-phase-0.txt, phase-0.txt, phase-1A..1I.txt, phase-2A..2D.txt,
    phase-3A..3K.txt, phase-4A..4K.txt
    phase-index-pre0-2D.txt, phase-index-3A-4J.txt (summaries, not source)


================================================================================
7. CROSS-CUTTING ENFORCEMENT EVERY B-N MUST RESPECT
================================================================================

WIRING_CONTRACT (Layer 0 from testing-trust-design.txt §LAYER 0):
  Every B-N file's TASK section must specify a WIRING_CONTRACT block:
    produces_symbols: [fully-qualified paths introduced by this step]
    consumes_symbols: [paths expected to call the produced symbols]
    reachable_from: [ordered set of project entrypoints, must be non-empty
                    for production steps; empty = library or test-only]
    stub_ban: [config_field_path + forbidden_pattern regex,
              e.g. "TaskRunnerConfig.baby_step_executor != None",
              "DashboardState.anti_spiral_enabled != false"]
    frontend_pair: optional — when produces_symbols introduces an SSB
                   action_type / API endpoint / IPC message, this names
                   the consumer baby step (must be sibling at same
                   decomposition level)
    observable_effect: optional but REQUIRED for USER_FACING steps — a
                       deterministic action on shipped binary + expected
                       SSB action within budget + optional file/stdout
                       pattern

DETERMINISTIC_CHECK:
  Every B-N must specify how an auditor verifies completion (commands +
  expected outputs).

ACCEPTANCE:
  Binary pass/fail criterion per deliverable, tied to command or metric.

NEVER MERGE WITHOUT:
  - Decision doc at workflow/reasoning/B-N-decisions.md
  - All commits include test pass/fail counts in the commit message
  - No new dependencies without confirmation


================================================================================
8. WHAT THE B-PLAN DOES NOT COVER
================================================================================

Phase 5 (Empire integration deep work): only B-25 stub for MCP capability
exposure. Full Empire bring-up deferred until D1 proven.

Phase 3L (Project Intelligence): explicitly deferred. project-intelligence-
design.txt proposes a new sub-phase 3L. Recommended ONLY after B-1 through
B-20 land. The detection routines themselves are implementable, but the
ProjectProfile is downstream consumption that benefits from the wiring
fixes first.

Speculative decoding revival: 1F decision was DROP. Not revisited.

Compiled pipeline (compiled-pipeline-design.txt): Phase 5+ optimization
after MVP solid.

Sub-spec amendments: spec text updates (blake2s → blake3, "5 vs 6 vs 7
predicate types", "9 vs 13 failure taxonomy categories", DAIRA "2 vs 15
markers", ".elai vs .elai_cc path constants") are part of B-15 and B-19/20
where the impl is touched. Not split into separate spec-PR steps.


================================================================================
9. WHO THIS DOCUMENT IS FOR
================================================================================

Three audiences:

  1. The implementing LLM (next session): read parts 1-3 + the relevant
     B-N file + the relevant lossless summary file (phase-N-summary.md).

  2. The user (deciding what to fund): read part 1 §1 + part 3 §§1,2,3,5 to
     understand what's left, prioritized.

  3. Future audits: the methodology in audit-implementation/_AGGREGATE.md
     and audit-dashboard/_AGGREGATE.md is the template for re-running this
     analysis after B-plan progress.


================================================================================
10. PART V SOURCE OF PATTERNS (SocratiCode evaluation, 2026-05-05)
================================================================================

SocratiCode (https://github.com/giancarloerra/socraticode, AGPL-3.0,
33,975 LOC TypeScript) was evaluated as a candidate pattern source for
ELAI's post-A-48 knowledge-engine extensions. Outcome: 7 stealable
patterns identified, scoped as B-29 through B-35.

What SocratiCode is:
  - MCP server + 21 tools for codebase indexing, hybrid search, file
    dependency graph, symbol-level call graph, and project-typed
    non-code artifact search (DB schemas, OpenAPI specs, k8s manifests)
  - 18,702 LOC source under src/ + 14,349 LOC tests + 830 LOC VS Code
    extension (LOC verified by codebase_mapper-style file counts)
  - Stack: Node 18+, Qdrant, Ollama (or OpenAI/Google/LM Studio for
    embeddings), Docker for managed local infra, ast-grep for polyglot
    parsing
  - Agent investigation found 17 services modules I missed in the
    initial pass; the agent rated SocratiCode's framing as "live
    workspace understanding platform with incremental updates,
    multi-language symbol resolution, rich visualization, and
    project knowledge augmentation" — broader than my original
    "indexer + graph" framing

What ELAI does NOT take from SocratiCode:
  - The Qdrant + Ollama + Docker stack — ELAI uses tantivy + custom
    sidecar + ONNX runtime; we already paid that cost
  - ast-grep — CLAUDE.md anti-stale rules pin tree-sitter + ELAI's
    DKG schema is tree-sitter-shaped
  - MCP-as-only-interface — ELAI's design is that Rust IS the
    orchestrator, not just a tool an external LLM calls
  - Branch-aware indexing — not needed for D1
  - AGPL source code — re-implement, don't transplant

What ELAI's claims about SocratiCode benchmarks should treat with
caution: README claims 37x faster / 61% less context / 84% fewer tool
calls vs grep on VS Code (2.45M LOC) tested live with Claude Opus 4.6.
Agent investigation confirmed: NO reproducible test in the repo
backs these numbers. scripts/benchmark-graph.ts measures graph-build
time only — does not run the AI-vs-grep comparison. The "40M LOC"
phrase in the README is target-codebase capacity claim, NOT self-size.
Largest scale test in the repo covers 200K symbols / ~10K files
(symbol-graph-scale.test.ts), not millions of lines.

Treat performance claims as marketing-grade, not engineering-grade,
when justifying any decision. The patterns are still worth porting on
their own merit (algorithmic correctness + production-grade
robustness), independent of the unverified speedup numbers.


================================================================================
END PART 3 — proceed to F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-0_INDEX.md
================================================================================
