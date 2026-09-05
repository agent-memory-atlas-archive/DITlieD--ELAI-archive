# B-1: Path Unification (.elai → .elai_cc everywhere)
Objective: eliminate the dual-database-path bug that makes PageRank, CatRAG,
A-RAG, and DkgQuerierAdapter return empty results at runtime even though the
DKG database is fully populated.
Estimated effort: 1 session (~2-4 hours).
Pre-requisites: none. Highest-leverage fix in entire B-plan.
Parallel-safe with: B-2 (FSM wiring), B-9 (SessionDag), B-10 (ContextBudget),
B-11 through B-14 (visual restoration).

## Context

Per the implementation audit aggregate (audit-implementation/_AGGREGATE.md
PATTERN 1) and reconfirmed in 1A/1D/1E/3J phase audits:

The ELAI codebase writes its DKG database to `.elai_cc/indexes/dkg.db` via
`IndexingService` using the `ELAI_DATA_DIR_NAME = ".elai_cc"` constant in
`crates/orchestrator/src/indexing_service.rs:15-18`. The CLI, however,
hardcodes `.elai/dkg.db` as a string literal at two sites in
`crates/cli/src/main.rs:3752` and `:5437`. The dashboard backend uses the
correct constants and reads from the populated database.

Net effect: CLI consumers (PageRank, CatRAG, A-RAG, telescope, blast radius
analysis) all see an empty DKG. The DkgQuerierAdapter silently falls through
to `EmptyDkgQuerier` permanently. Phase 1D's PageRank stage is "wired in
code, dead at runtime" — it computes against zero nodes. Phase 1E's CatRAG
stage pushes empty results unconditionally. AragOrchestrator corpus stays
empty.

The dashboard sees real DKG data; the CLI does not. Two-line fix.

Phase 3J.WIRE.STUBS audit also confirms: rule engine reads
`.elai/sessions/dkg.db` instead of `.elai_cc/indexes/dkg.db`. Same root cause.

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\docs\thefinalglance\audit-implementation\_AGGREGATE.md`
  PATTERN 1 (DUAL DATABASE PATHS)
- `F:\.ELAI_workflow\docs\thefinalglance\audit-implementation\phase-1A.md`
  WIRING section (DkgEngine::build_graph IS called from runtime path)
- `F:\.ELAI_workflow\docs\thefinalglance\audit-implementation\phase-1D.md`
  WIRING section + section §5 (DB path read)
- `F:\.ELAI_workflow\docs\thefinalglance\audit-implementation\phase-1E.md`
  Runtime stage firing
- `F:\.ELAI_workflow\docs\thefinalglance\audit-implementation\phase-3J.md`
  WIRE.STUBS subsection
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\indexing_service.rs`
  lines 15-18 (constants source of truth)
- `F:\.ELAI_workflow\.elai_cc\crates\cli\src\main.rs:3750-3760` and
  `:5430-5445` (the two literal sites)

## Tasks

1. AUDIT all literal `.elai/` string occurrences across the codebase.
   Use Grep on `\.elai/` (escaped slash) over the entire `.elai_cc/` tree
   excluding the legitimate `.elai_cc/` literals. Expect ~5-10 hits, all
   in CLI / examples / tests / docs.

2. Establish a single source of truth for index dir path. Promote the
   `ELAI_DATA_DIR_NAME = ".elai_cc"` and `INDEXES_DIR_NAME = "indexes"`
   constants to `crates/core/src/paths.rs` (create if doesn't exist) so
   ALL crates can import them. Add helper:
   ```rust
   pub fn dkg_db_path(project_root: &Path) -> PathBuf {
       project_root.join(ELAI_DATA_DIR_NAME).join(INDEXES_DIR_NAME).join("dkg.db")
   }
   ```
   Add similar helpers for `bm25_dir_path`, `vectors_db_path`,
   `sessions_dir_path`, `metrics_dir_path`, `models_dir_path`,
   `rules_db_path`.

3. Refactor `crates/orchestrator/src/indexing_service.rs:15-18` to import
   from core (no behavior change — it already uses the right path).

4. Refactor `crates/cli/src/main.rs:3752` (DKG load for QueryPipeline) to
   call `core::paths::dkg_db_path(project_root)`. Remove the literal
   `.elai/dkg.db`.

5. Refactor `crates/cli/src/main.rs:5437` (telescope DKG load) the same way.

6. Refactor any rule engine reader of `.elai/sessions/dkg.db` to use
   `core::paths::dkg_db_path` (per 3J.WIRE.STUBS audit).

7. Verify `crates/cli/src/main.rs:3756` gate `if .elai/dkg.db exists`
   is updated to use the new path. The gate should now FIRE in production
   because the database actually exists at the canonical location.

8. Search for `.elai/models/` literals (vector models per 1C audit) and
   replace with `core::paths::models_dir_path`. Spec calls for
   `.elai_cc/models/onnx/` — fix the path mismatch.

9. Add a startup-time assertion in CLI bootstrap that EITHER `.elai_cc/`
   exists OR a clear "uninitialized project" error fires. Do not silently
   fall through to `EmptyDkgQuerier`.

10. Migrate any pre-existing `.elai/` directories: if both `.elai/` and
    `.elai_cc/` exist on a user machine, log a deprecation warning and
    prefer `.elai_cc/`. Do NOT auto-delete `.elai/` (user may have other
    tools using it).

## WIRING_CONTRACT

produces_symbols:
  - core::paths::dkg_db_path
  - core::paths::bm25_dir_path
  - core::paths::vectors_db_path
  - core::paths::sessions_dir_path
  - core::paths::metrics_dir_path
  - core::paths::models_dir_path
  - core::paths::rules_db_path
consumes_symbols:
  - crates/cli/src/runtime_builder.rs:224 → uses dkg_db_path
  - crates/cli/src/repl_command.rs:625 → uses dkg_db_path
  - crates/cli/src/live_cli.rs:3255 → uses dkg_db_path
  - crates/orchestrator/src/indexing_service.rs:15-18 → imports from core
  - any vector index loader → uses vectors_db_path / models_dir_path
  - any rule engine DKG reader → uses dkg_db_path
reachable_from:
  - crates/cli/src/main.rs::main
  - crates/dashboard-v2/src-tauri/src/lib.rs::run (V2 — was V1
    crates/dashboard/src/lib.rs::run)
  - crates/dashboard-v2/src-tauri/src/lib.rs::submit_task (V2 — was V1
    crates/dashboard/src/commands.rs::submit_task; currently stubbed at
    src/lib/api/stub.ts:163, B-36 wires the live invoke)
  - crates/orchestrator/src/task_runner.rs::drive_task
stub_ban:
  - "\\.elai/" literal absent in non-test source
  - "\\.elai_cc/dkg" literal absent (must go through path helper)
frontend_pair: null
observable_effect:
  trigger: .elai_cc\target\release\elai.exe bench canary-regression --fixtures .elai_cc/benchmarks/canary/fixtures
  expect_ssb: canary_bench_run_completed
  expect_artifact: BENCH-CANARY PASS

## DETERMINISTIC_CHECK

```bash
# 1. No .elai/ literals remain in non-test source
rg "\"\.elai/" .elai_cc/crates --type rust --glob '!**/tests/**' --glob '!**/examples/**'
# Expected: zero hits.

# 2. Path helpers exist and are used
rg "core::paths::dkg_db_path" .elai_cc/crates --type rust
# Expected: at least 4 hits (cli/main.rs ×3, indexing_service.rs ×1).

# 3. End-to-end smoke test: launch elai REPL on a project with indexed
#    DKG, query "list rust files", verify response references actual
#    files from .elai_cc/indexes/dkg.db (not empty result).

# 4. Pagerank renders non-empty in dashboard KnowledgeDrawer (after B-3
#    eradicates the get_pagerank_top stub — for B-1 verification, confirm
#    the underlying SQLite query returns non-zero rows).
sqlite3 .elai_cc/.elai_cc/indexes/dkg.db "SELECT COUNT(*) FROM dkg_nodes;"
# Expected: matches phase_1A.toml node count (~8000+).
```

## ACCEPTANCE

- [ ] Zero `.elai/` literals in non-test source (verified via Grep above)
- [ ] All path helpers exist in `crates/core/src/paths.rs` and are exported
- [ ] CLI startup logs canonical paths (DKG, BM25, vectors, sessions, models)
- [ ] CLI DKG query returns non-empty results matching phase_1A.toml metrics
- [ ] Dashboard KnowledgeDrawer node count is non-zero (currently shows real
      data via dashboard's correct path; verify still works after refactor)
- [ ] PageRank stage in pipeline.rs:387 actually fires (add tracing::info
      to count nodes processed)
- [ ] CatRAG overlay non-empty during query
- [ ] AragOrchestrator add_chunk wired into IndexingService → corpus
      populates as files index

## Tests to add

- `crates/core/tests/paths.rs`: contract tests for each helper, on Windows
  + Unix path separators
- `crates/cli/tests/dkg_path_integration.rs`: spawn elai REPL on a fixture
  project, assert query returns real file references
- `crates/orchestrator/tests/indexing_service_path_contract.rs`: verify
  IndexingService writes to `core::paths::dkg_db_path` (not a hardcoded literal)
- `tests/contracts/path_unification.rs`: regression test that fails if
  `\.elai/` literal reintroduced

## Files touched

- CREATE: `crates/core/src/paths.rs` (path helpers, single source of truth)
- MODIFY: `crates/core/src/lib.rs` (export `paths` module)
- MODIFY: `crates/cli/src/runtime_builder.rs:224` (DKG load, was main.rs:3752)
- MODIFY: `crates/cli/src/repl_command.rs:625` (surgical DKG load, was main.rs:3756)
- MODIFY: `crates/cli/src/live_cli.rs:3255` (REPL DKG load, was main.rs:5437)
- MODIFY: `crates/orchestrator/src/indexing_service.rs:15-18`
  (import from core)
- MODIFY: any rule engine DKG-reader site (per 3J audit)
- MODIFY: any vector index loader site that uses `.elai/models/`
- CREATE: `workflow/reasoning/B-1-decisions.md` (decision doc)

## Side benefits unlocked by this fix

After B-1, the following currently-dead paths become live:
- Phase 1D PageRank yields real symbols
- Phase 1E CatRAG overlay populated
- Phase 1E A-RAG corpus populated
- Phase 3J.WIRE.STUBS DkgQuerierAdapter no longer fall-through
- Context Telescope (Layer 1 Project Vision) gets real symbol counts
- Surgical mode (when implemented) blast radius non-empty

This single B-step transitions ~5 phase audits from "PARTIAL/DEAD" to
"WIRED". Highest leverage in entire B-plan.

## Decision doc

Write `workflow/reasoning/B-1-decisions.md` documenting:
- Whether path helpers go in `core/paths.rs` or `core/lib.rs` directly
- Whether to migrate existing `.elai/` directories or leave them
- How to handle test fixtures that hardcoded `.elai/`

## Next

B-2 (FSM PhaseTransition Wiring) — can run in parallel.
B-3 (Stub Command Eradication) — needs B-1 done so stub commands can be
  replaced with real readers off the canonical path.
