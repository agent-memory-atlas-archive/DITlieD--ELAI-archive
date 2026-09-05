# C-17: Knowledge Indexing Resilience
Objective: close all MISSING and PARTIAL gaps from B-30, B-31, B-31.5, B-34, and
phase-3K that concern knowledge indexing completeness, signal fidelity, and
graceful lifecycle — Kotlin grammar, svelte extraction test, CLI signal handlers,
auto-resume wiring, LockScope::Watch watcher, indexing resilience contract,
Heading/Paragraph chunking, real ONNX query embeddings, Oracle pre-seeding, and
production-scale benchmark population.
Estimated effort: 2-3 sessions.
Pre-requisites: C-15 (clock/entropy stable), C-14 (dead code strip).
Parallel-safe with: C-16, C-18, C-19, C-20.

## Context

Five verification reports (B-30, B-31, B-31.5, B-34, phase-3K) converge on a
shared theme: indexing lifecycle and retrieval quality are mostly implemented but
have a consistent class of gap — grammar coverage missing for Kotlin, fallback
paths (50-line windows, norm proxy) substituting for real implementations, and CLI
entrypoint wiring that exists only in the dashboard. The gaps are independent at
the code level but share the same root cause: each B-plan shipped the core
algorithm and deferred the integration seam.

Key source facts (from verification reports, HIGH confidence):
- KotlinGrammar struct is absent; "kt"/"kts" missing from ext_to_lang_id
  (B-30 report, MISSING section, mod.rs:3233-3258).
- auto_resume_indexed_projects called from dashboard-v2/lib.rs:582 only;
  crates/cli/src/main.rs has no call (B-31 report, MISSING section).
- graceful_shutdown called from dashboard-v2/lib.rs:662 only; cli/main.rs
  has no SIGINT/SIGTERM handler (B-31 report, MISSING section).
- LockScope::Watch declared in lock.rs:262-270 with "Unused in Cycle B" note;
  no watcher service module exists in orchestrator/src/ (B-31 PARTIAL).
- tests/contracts/indexing_resilience_contract.rs absent (B-31 MISSING).
- ChunkKind has no Heading or Paragraph variant; non-code files fall back to
  50-line line_block chunks (phase-3K PARTIAL, bm25.rs:425).
- semantic_search in arag.rs:261-272 uses embedding-norm proxy, not
  query-document dot-product (phase-3K PARTIAL, arag.rs comment confirms).
- /research handler in live_cli.rs:2203 does not call detect_entry_points;
  PROJECT_STARTING_POINTS block absent (B-34 MISSING).
- run_tier_a.sh outputs per_batch_p50_ms=0, per_batch_p99_ms=0 (B-31.5
  PARTIAL, indexing_service.rs:90-91 stub timing).
- All phase_b31_5_tier_*.toml files contain synthetic 200-file data only;
  production-scale runs deferred in README (B-31.5 PARTIAL).
- Dashboard SSB-driven IndexResume/LockContested banners absent from
  ActivityFeed; only polling-based IndexingProgressPanel exists (B-31 PARTIAL).

## Reads

- Verification reports cited above (all in docs/thefinalglance/verification/reports/)
- .elai_cc/crates/knowledge/src/dkg/mod.rs (KotlinGrammar insertion point ~line 3233)
- .elai_cc/crates/knowledge/src/bm25.rs (ChunkKind enum ~line 48)
- .elai_cc/crates/knowledge/src/arag.rs (semantic_search ~line 251)
- .elai_cc/crates/cli/src/main.rs (auto_resume + signal handler insertion point)
- .elai_cc/crates/orchestrator/src/lock.rs (LockScope::Watch ~line 262)
- .elai_cc/crates/orchestrator/src/startup.rs (graceful_shutdown ~line 546)
- .elai_cc/crates/orchestrator/src/indexing_service.rs (per-batch timing ~line 90)
- .elai_cc/crates/cli/src/live_cli.rs (SlashCommand::Research arm ~line 2203)
- .elai_cc/scripts/scale_test/run_tier_a.sh (p50/p99 stub)

## Tasks

1. **KotlinGrammar struct** in crates/knowledge/src/dkg/mod.rs — add
   KotlinGrammar struct mirroring JavaGrammar; use tree-sitter-kotlin if
   crate exists in workspace Cargo.toml, else reuse JavaGrammar parser as
   fallback with language_id = "kotlin". Anti-slopsquatting: verify crate
   name via `cargo search tree-sitter-kotlin` before adding dep.

2. **Register "kt"/"kts" in ext_to_lang_id** in dkg/mod.rs ~line 3233 and
   register KotlinGrammar in LanguageRegistry::default() ~line 222.
   Impact check: grep ext_to_lang_id call sites before edit.

3. **svelte_extraction.rs test** — create
   crates/knowledge/tests/svelte_extraction.rs covering: (a) script block
   re-parse as TypeScript produces qualified names with "<file>.svelte::" prefix,
   (b) style block CSS @import extraction does not bleed into script symbols,
   (c) SvelteGrammar parse does not panic on minimal Svelte fixture.

4. **Wire auto_resume_indexed_projects into cli::main** —
   crates/cli/src/main.rs: call auto_resume_indexed_projects AFTER init and
   BEFORE task dispatch. Must match stub_ban requirement "called from BOTH cli
   main and dashboard run" (B-31 WIRING_CONTRACT).

5. **Install SIGINT/SIGTERM signal handlers in cli::main** — call
   graceful_shutdown from tokio signal handlers in cli::main. Use
   tokio::signal::ctrl_c (cross-platform) + #[cfg(unix)]
   tokio::signal::unix::signal(SignalKind::terminate()) for SIGTERM.

6. **LockScope::Watch watcher service** — create
   crates/orchestrator/src/watcher_service.rs; implement a basic
   FileWatcherService that acquires LockScope::Watch via IndexLock::try_acquire
   on project start and releases on stop. Wire into startup.rs alongside
   the completed-project watcher-start path (startup.rs:318).

7. **indexing_resilience_contract.rs** — create
   crates/contracts/tests/indexing_resilience_contract.rs with:
   (a) #[must_use] attribute regression on IndexResult variants,
   (b) type-level assertion that IndexLock requires LockScope parameter,
   (c) contract that auto_resume_indexed_projects returns within 5s on empty
   project list (timeout guard), (d) contract that graceful_shutdown deadline
   constant >= 30 and <= 120 seconds.

8. **Heading and Paragraph ChunkKind variants** — add Heading and Paragraph
   variants to ChunkKind enum in crates/knowledge/src/bm25.rs ~line 48.
   Implement heading-boundary splitter (detect "# " / "## " lines) and
   blank-line-paragraph splitter for non-code files in the try_ast_chunk /
   chunk_file path. Non-.rs/.py/.ts/.go/.java/.cs/.rb files route to the
   new splitters before the 50-line fallback.

9. **Wire real ONNX query embedding into AragOrchestrator::semantic_search** —
   crates/knowledge/src/arag.rs ~line 251: compute query embedding via the
   dual-vector ONNX session already available from Phase 1C (grep for
   OnnxEmbeddingModel or EmbeddingSession in arag.rs / knowledge crate before
   assuming API shape). Replace norm proxy with dot-product score against
   chunk embedding vectors. Preserve 3x SEMANTIC_OVERSAMPLE_FACTOR.

10. **/research Oracle pre-seeding** — in crates/cli/src/live_cli.rs
    SlashCommand::Research arm (~line 2195): call detect_entry_points on
    the active project_dir, format as PROJECT_STARTING_POINTS block, prepend
    to the query string before run_research_pipeline. Emit EntryPointsDetected
    SSB row. Guard with project_dir.is_some() check; no-op if project not set.

11. **Per-batch p50/p99 timing in run_tier_a.sh** —
    .elai_cc/scripts/scale_test/run_tier_a.sh: parse tracing output from
    indexing_service (look for "batch_complete" span or similar) to compute
    real p50/p99 per-batch wall-clock times. Write real values into
    metrics/phase_b31_5_tier_a.toml replacing hardcoded 0 stubs.

12. **Production-scale fixture runs** — run
    bash scripts/scale_test/run_all_tiers.sh against real ELAI_SCALE_FIXTURE_ROOT
    (Linux kernel v6.7 + VS Code 1.94.2 minimum; Chromium + Postgres as
    stretch goal). Populate all per-tier TOML metric files with real measured
    values. Run contract tests against populated TOMLs: cargo test -p
    elai-orchestrator --test scale_resume_identity_contract --test
    scale_lock_contention_contract --test scale_long_tail_memory_contract.

13. **SSB-driven dashboard banners** — wire IndexResume and LockContested SSB
    rows into ActivityFeed in the dashboard. IndexResume row renders as
    "Resumed indexing from batch N/M (saved X files)"; LockContested row
    renders as "Another ELAI process (PID X) is indexing; read-only mode".
    Pattern: follow existing SSB-driven banner rows in KnowledgeDrawer.svelte
    or ActivityFeed component; subscribe to SSB stream, filter on action_type.

14. **Decision doc** — write workflow/reasoning/C-17-decisions.md covering:
    KotlinGrammar fallback choice (tree-sitter-kotlin vs JavaGrammar reuse),
    ONNX session access pattern chosen, watcher service lifecycle design,
    per-batch timing parsing strategy.

## WIRING_CONTRACT

produces_symbols:
  - KotlinGrammar struct + "kt"/"kts" in ext_to_lang_id
  - ChunkKind::Heading + ChunkKind::Paragraph variants
  - heading_boundary_chunks + paragraph_chunks splitter fns in bm25.rs
  - real dot-product scoring in AragOrchestrator::semantic_search
  - PROJECT_STARTING_POINTS prepend in SlashCommand::Research
  - FileWatcherService + LockScope::Watch wiring in watcher_service.rs
  - indexing_resilience_contract.rs (4 contract assertions)
  - SSB-driven IndexResume + LockContested banners in dashboard ActivityFeed
consumes_symbols:
  - LanguageGrammar trait (dkg/mod.rs) — KotlinGrammar implements it
  - EmbeddingSession / OnnxEmbeddingModel (Phase 1C) — arag.rs semantic_search
  - detect_entry_points (knowledge::symbol_graph::entry_points::detect)
  - auto_resume_indexed_projects + graceful_shutdown (orchestrator::startup)
  - IndexLock::try_acquire (orchestrator::lock) — LockScope::Watch path
  - SsbActionType::IndexResume + SsbActionType::LockContested (orchestrator::ssb)
reachable_from:
  - cli::main (auto_resume + signal handlers)
  - SlashCommand::Research arm (Oracle pre-seeding)
  - watcher_service::FileWatcherService (LockScope::Watch)
  - dashboard ActivityFeed (SSB banners)
stub_ban:
  - auto_resume_indexed_projects MUST be called from BOTH cli::main AND dashboard
  - graceful_shutdown MUST be reachable from BOTH cli SIGINT/SIGTERM AND dashboard
    ExitRequested — neither entrypoint may be the sole caller
  - semantic_search MUST use query-document dot-product; norm proxy is forbidden
  - LockScope::Watch MUST be acquired via IndexLock::try_acquire in watcher service;
    bare filesystem polling is not a substitute
observable_effect:
  trigger: index Rust+Kotlin+Svelte+Markdown fixture project; send SIGINT midway;
           restart elai; run /research --mode cloud "what does FooService do?"
  expect_ssb: IndexResume row with resumed_from_batch > 0; EntryPointsDetected row;
              LanguageGrammarStatus shows kotlin loaded
  expect_artifact: DKG contains entries for .kt files; QueryPipeline returns results
                   with real dot-product scores; PROJECT_STARTING_POINTS block appears
                   in research query context; Heading/Paragraph chunks visible for .md

## DETERMINISTIC_CHECK

```bash
# 1. KotlinGrammar struct present
grep -n "KotlinGrammar" .elai_cc/crates/knowledge/src/dkg/mod.rs | head -5

# 2. "kt" registered in ext_to_lang_id
grep -n '"kt"\|"kts"' .elai_cc/crates/knowledge/src/dkg/mod.rs

# 3. Heading and Paragraph in ChunkKind
grep -n "Heading\|Paragraph" .elai_cc/crates/knowledge/src/bm25.rs | head -10

# 4. semantic_search dot-product (no norm proxy)
grep -n "norm\|proxy\|In production" .elai_cc/crates/knowledge/src/arag.rs | head -5
# Expected: zero matches (proxy comment removed, real impl in place)

# 5. auto_resume in cli main
grep -n "auto_resume_indexed_projects" .elai_cc/crates/cli/src/main.rs

# 6. signal handler in cli main
grep -n "ctrl_c\|SignalKind\|graceful_shutdown" .elai_cc/crates/cli/src/main.rs

# 7. watcher_service module
test -f .elai_cc/crates/orchestrator/src/watcher_service.rs && echo "EXISTS"

# 8. indexing_resilience_contract
test -f .elai_cc/crates/contracts/tests/indexing_resilience_contract.rs && echo "EXISTS"

# 9. PROJECT_STARTING_POINTS in research arm
grep -n "PROJECT_STARTING_POINTS\|detect_entry_points" .elai_cc/crates/cli/src/live_cli.rs | head -5

# 10. svelte_extraction test
test -f .elai_cc/crates/knowledge/tests/svelte_extraction.rs && echo "EXISTS"

# 11. cargo check knowledge + orchestrator crates
cargo check -p elai-knowledge -p elai-orchestrator 2>&1 | tail -5

# 12. knowledge + orchestrator tests
cargo test -p elai-knowledge -p elai-orchestrator 2>&1 | grep "test result"

# 13. contracts tests
cargo test -p elai-contracts 2>&1 | grep "test result"

# 14. tier_a p50/p99 non-zero
grep "per_batch_p50_ms\|per_batch_p99_ms" .elai_cc/metrics/phase_b31_5_tier_a.toml
# Expected: values > 0
```

## ACCEPTANCE

- [ ] KotlinGrammar struct implemented; "kt"/"kts" registered in ext_to_lang_id;
      KotlinGrammar registered in LanguageRegistry::default()
- [ ] svelte_extraction.rs test exists and passes: composite-language re-parse,
      qualified name format, no CSS bleed
- [ ] auto_resume_indexed_projects called from crates/cli/src/main.rs (verified
      by grep, not by runtime only)
- [ ] SIGINT/SIGTERM signal handlers in cli::main call graceful_shutdown
- [ ] FileWatcherService in watcher_service.rs acquires LockScope::Watch via
      IndexLock::try_acquire; wired from startup.rs completed-project path
- [ ] indexing_resilience_contract.rs exists with 4 contract assertions; passes
      cargo test -p elai-contracts
- [ ] ChunkKind::Heading and ChunkKind::Paragraph added to bm25.rs enum;
      heading-boundary and blank-line-paragraph splitters active for .md files
- [ ] AragOrchestrator::semantic_search uses ONNX dot-product; norm proxy
      comment absent; arag_tests still pass
- [ ] SlashCommand::Research prepends PROJECT_STARTING_POINTS block when
      project_dir is set; EntryPointsDetected SSB emitted
- [ ] run_tier_a.sh outputs non-zero per_batch_p50_ms and per_batch_p99_ms in
      phase_b31_5_tier_a.toml
- [ ] run_all_tiers.sh run against real scale fixture root (Linux v6.7 minimum);
      scale contract tests pass against populated TOMLs
- [ ] Dashboard ActivityFeed shows IndexResume banner and LockContested banner
      from SSB stream (not polling)
- [ ] Cloud Sonnet smoke test passes end-to-end:
      (a) index Rust+Kotlin+Svelte+Markdown fixture; kill midway with SIGINT;
      (b) restart elai; SSB shows IndexResume with resumed_from_batch > 0 and
          no full reindex (files_skipped_on_resume > 0);
      (c) elai /research --mode cloud "what does the FooService class do across
          the codebase?" executes against cloud Sonnet 4.6;
      (d) DKG contains entries for .kt files; QueryPipeline results carry real
          dot-product scores (not norm proxy); PROJECT_STARTING_POINTS block
          present in research context; Heading/Paragraph chunks present for .md
- [ ] cargo check --workspace clean after all changes
- [ ] workflow/reasoning/C-17-decisions.md written

## Files touched

- MODIFY: crates/knowledge/src/dkg/mod.rs (KotlinGrammar struct + ext_to_lang_id + registry)
- MODIFY: crates/knowledge/src/bm25.rs (ChunkKind::Heading/Paragraph + splitters)
- MODIFY: crates/knowledge/src/arag.rs (real ONNX dot-product in semantic_search)
- MODIFY: crates/cli/src/main.rs (auto_resume call + signal handlers)
- MODIFY: crates/cli/src/live_cli.rs (PROJECT_STARTING_POINTS Oracle pre-seeding)
- ADD: crates/orchestrator/src/watcher_service.rs (FileWatcherService + LockScope::Watch)
- MODIFY: crates/orchestrator/src/startup.rs (wire FileWatcherService from completed-project path)
- ADD: crates/knowledge/tests/svelte_extraction.rs
- ADD: crates/contracts/tests/indexing_resilience_contract.rs
- MODIFY: scripts/scale_test/run_tier_a.sh (real per-batch p50/p99 timing)
- MODIFY: metrics/phase_b31_5_tier_a.toml (real values after fixture run)
- MODIFY: metrics/phase_b31_5_tier_b.toml (real values after fixture run)
- MODIFY: metrics/phase_b31_5_scale.toml (update overall_pass after real run)
- MODIFY: crates/dashboard-v2/src/lib/components/ActivityFeed.svelte (IndexResume + LockContested banners)
- ADD: workflow/reasoning/C-17-decisions.md

## Next

C-18 through C-20 (parallel-safe); after all C-plan phases complete, transition
to final integration smoke pass and MEMORIZING.
