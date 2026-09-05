# Verification Report: B-1_path_unification
Source: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-1_path_unification.md
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary

The core deliverable of B-1 — eliminating `.elai/dkg.db` literals and centralising
all path logic in `crates/core/src/paths.rs` — is fully implemented and guarded by
contract/regression tests. All 7 path helpers, all exported constants, the
orchestrator re-export shim, the deprecation-warning logic, the startup-assertion,
and the rule-engine DKG fix are present in code. Two residual gaps remain: (1) two
inline `join("dkg.db")` literals in `task_runner.rs` bypass the canonical helper and
are not caught by the regression scan, and (2) the `tracing::info` per-node-count
instrumentation on the PageRank stage (acceptance item) was not added.

## Claims verified DONE

- `crates/core/src/paths.rs` created with all 7 path helpers — evidence:
  [paths.rs:109](.elai_cc/crates/core/src/paths.rs#L109) (`dkg_db_path`),
  [paths.rs:121](.elai_cc/crates/core/src/paths.rs#L121) (`bm25_dir_path`),
  [paths.rs:129](.elai_cc/crates/core/src/paths.rs#L129) (`vectors_db_path`),
  [paths.rs:138](.elai_cc/crates/core/src/paths.rs#L138) (`sessions_dir_path`),
  [paths.rs:146](.elai_cc/crates/core/src/paths.rs#L146) (`metrics_dir_path`),
  [paths.rs:159](.elai_cc/crates/core/src/paths.rs#L159) (`models_dir_path`),
  [paths.rs:167](.elai_cc/crates/core/src/paths.rs#L167) (`rules_db_path`)

- `crates/core/src/lib.rs` exports `paths` module — evidence:
  [lib.rs:10](.elai_cc/crates/core/src/lib.rs#L10) `pub mod paths;`

- `ELAI_DATA_DIR_NAME`, `INDEXES_DIR_NAME`, `DKG_DB_FILENAME` and 5 other constants
  exported — evidence: [paths.rs:37](.elai_cc/crates/core/src/paths.rs#L37),
  [paths.rs:41](.elai_cc/crates/core/src/paths.rs#L41), [paths.rs:44](.elai_cc/crates/core/src/paths.rs#L44)

- `crates/orchestrator/src/indexing_service.rs` imports from core (re-export shim)
  — evidence: [indexing_service.rs:42](.elai_cc/crates/orchestrator/src/indexing_service.rs#L42)
  `pub use elai_core::paths::{ELAI_DATA_DIR_NAME, INDEXES_DIR_NAME};`

- CLI DKG path (task #4 / #5) — `live_cli.rs` and `symbol_graph.rs` both use
  `core_dkg_db_path` alias — evidence:
  [live_cli.rs:58](.elai_cc/crates/cli/src/live_cli.rs#L58),
  [live_cli.rs:1044](.elai_cc/crates/cli/src/live_cli.rs#L1044),
  [live_cli.rs:2410](.elai_cc/crates/cli/src/live_cli.rs#L2410),
  [live_cli.rs:3089](.elai_cc/crates/cli/src/live_cli.rs#L3089),
  [symbol_graph.rs:21](.elai_cc/crates/cli/src/symbol_graph.rs#L21),
  [symbol_graph.rs:78](.elai_cc/crates/cli/src/symbol_graph.rs#L78)

- Rule-engine DKG reader fixed (task #6) — `runtime_builder.rs` severed the
  `.elai/sessions/dkg.db` per-session binding and now uses `core_dkg_db_path`
  — evidence: [runtime_builder.rs:222](.elai_cc/crates/cli/src/runtime_builder.rs#L222)

- `.elai/models/` literals absent (task #8) — `grep \.elai/models` on crates/
  returns zero hits; `live_cli.rs` uses `core_models_dir_path`
  — evidence: [live_cli.rs:945](.elai_cc/crates/cli/src/live_cli.rs#L945)

- Startup-time warning for uninitialized project (task #9) — non-fatal
  `tracing::warn` fires after IndexingService run when `.elai_cc/` is absent
  — evidence: [indexing.rs:301-328](.elai_cc/crates/cli/src/indexing.rs#L301)

- Deprecation warning when both `.elai/` and `.elai_cc/` coexist (task #10)
  — evidence: [indexing.rs:170-183](.elai_cc/crates/cli/src/indexing.rs#L170)

- `SSB_SESSIONS_SUBDIR` updated to `.elai_cc/sessions`
  — evidence: [ssb.rs:4689](.elai_cc/crates/orchestrator/src/ssb.rs#L4689)

- Regression contract test `no_dot_elai_literal_in_in_scope_source` present and
  compiled (`.exe` artifacts in target/) — evidence:
  [path_unification.rs:106](.elai_cc/crates/contracts/tests/path_unification.rs#L106)

- `ssb_sessions_subdir_is_canonical` contract test present
  — evidence: [path_unification.rs:166](.elai_cc/crates/contracts/tests/path_unification.rs#L166)

- `core_paths_helper_is_referenced_by_consumers` contract test (>=4 hits)
  — evidence: [path_unification.rs:176](.elai_cc/crates/contracts/tests/path_unification.rs#L176)

- `crates/core/tests/paths.rs` — all 7 helpers covered with Windows+Unix separator
  tests, idempotency, IO-purity — evidence: [paths.rs:1-275](.elai_cc/crates/core/tests/paths.rs)

- `crates/orchestrator/tests/indexing_service_path_contract.rs` — 3 tests:
  re-export match, `dkg_db_path()` accessor equality, legacy path exclusion
  — evidence: [indexing_service_path_contract.rs:20-71](.elai_cc/crates/orchestrator/tests/indexing_service_path_contract.rs#L20)

- `crates/cli/tests/dkg_path_integration.rs` — stub present (intentionally
  `#[ignore]`; documented as manual/sidecar-dependent)
  — evidence: [dkg_path_integration.rs:38](.elai_cc/crates/cli/tests/dkg_path_integration.rs#L38)

- `workflow/reasoning/B-1-decisions.md` written with 7 decisions
  — evidence: `workflow/reasoning/B-1-decisions.md:1`

- Dashboard (`dashboard-v2/src-tauri/src/commands.rs`) uses
  `elai_core::paths::dkg_db_path` — evidence:
  [commands.rs:1057](.elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs#L1057),
  [commands.rs:1204](.elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs#L1204)

- stub_ban 1: zero `".elai/"` literals in non-test, non-whitelisted source
  (regression test enforces this; whitelist covers 5 intentional files)
  — evidence: [path_unification.rs:27-37](.elai_cc/crates/contracts/tests/path_unification.rs#L27)

- stub_ban 2: `".elai_cc/dkg"` literal absent in production source (only in
  comments and bench fixture) — evidence: grep returns 0 production hits

## Claims PARTIAL

- Acceptance: "CLI startup logs canonical paths (DKG, BM25, vectors, sessions, models)"
  - exists: startup assertion logs the DKG path at info level
    [indexing.rs:324](.elai_cc/crates/cli/src/indexing.rs#L324)
  - gap: BM25, vectors, sessions, and models paths are NOT logged at startup;
    only the DKG canonical path is confirmed via a single `tracing::info!`.

- Acceptance: "PageRank stage in pipeline.rs fires (add tracing::info to count
  nodes processed)"
  - exists: PageRank stage fires via `dkg_engine` when populated
    [pipeline.rs:471-506](.elai_cc/crates/knowledge/src/pipeline.rs#L471)
  - gap: the required `tracing::info!` for node-count instrumentation was NOT
    added to `pipeline.rs`; acceptance item is unmet.

- task_runner.rs DKG path (WIRING_CONTRACT: `task_runner.rs::drive_task`)
  - exists: `task_runner.rs` imports and uses `dkg_db_path` at lines 8496/8614
    [task_runner.rs (elai.map line 124322)]
  - gap: two additional inline `guard.indexes_dir().join("dkg.db")` expressions
    at [task_runner.rs:3943](.elai_cc/crates/orchestrator/src/task_runner.rs#L3943)
    and [task_runner.rs:3995](.elai_cc/crates/orchestrator/src/task_runner.rs#L3995)
    bypass `core::paths::dkg_db_path`; not caught by regression scan (scan
    targets `".elai/"` literals only, not bare `"dkg.db"` string arguments).

## Claims MISSING

- Acceptance: "CatRAG overlay non-empty during query" — no new instrumentation or
  wiring connecting CatRAG populate status to a startup or query log was added;
  searched: `catrag.*non.empty`, `CatRAG.*overlay`, `catrag.*populate`; wiring
  to DKG is code-complete (pipeline.rs has catrag stage) but the acceptance
  criterion (observable non-empty result) has no deterministic test.

- Acceptance: "AragOrchestrator add_chunk wired into IndexingService → corpus
  populates as files index" — searched: `add_chunk.*IndexingService`,
  `IndexingService.*arag`, `arag.*corpus`; no evidence this wiring was added
  in B-1; no code path connects `run_full_index` completion to A-RAG corpus
  ingestion.

## Claims UNVERIFIABLE

- "Dashboard KnowledgeDrawer node count is non-zero (currently shows real data)"
  — runtime/UI behavior; no automated test exists for this; dashboard code uses
  `dkg_db_path` correctly but live node-count outcome requires a populated DB.

- "CLI DKG query returns non-empty results matching phase_1A.toml metrics" —
  runtime behavior; `dkg_path_integration.rs` is `#[ignore]`; no automated
  fixture exists.

- Changelog/memory-bank references — gitignored paths, not verifiable from source.

## What must be done (action list)

- [ ] Replace `guard.indexes_dir().join("dkg.db")` at
  [task_runner.rs:3943](.elai_cc/crates/orchestrator/src/task_runner.rs#L3943)
  and [task_runner.rs:3995](.elai_cc/crates/orchestrator/src/task_runner.rs#L3995)
  with `elai_core::paths::dkg_db_path(project_root)` to satisfy stub_ban 2
  fully and prevent regression.

- [ ] Extend the regression scan in
  [path_unification.rs](.elai_cc/crates/contracts/tests/path_unification.rs)
  to also flag bare `.join("dkg.db")` string literals in production source
  (currently the test only guards against `".elai/"` string literals).

- [ ] Add `tracing::info!(nodes = ..., "pagerank stage fired")` to
  [pipeline.rs:486-487](.elai_cc/crates/knowledge/src/pipeline.rs#L486)
  to satisfy the PageRank observability acceptance criterion.

- [ ] Add startup log lines for BM25, vectors, sessions, and models canonical
  paths at [indexing.rs](.elai_cc/crates/cli/src/indexing.rs) (around line 320)
  to fully satisfy "CLI startup logs canonical paths" acceptance item.

- [ ] Decide in B-3 or a follow-up patch whether AragOrchestrator `add_chunk`
  should be wired into `IndexingService::run_full_index` completion; document
  as deferred in B-1-decisions.md if not B-1 scope.

## Notes

The whitelist in `path_unification.rs` includes `crates/cli/src/main.rs` wholesale
for legacy upgrade-reader, steering-cache, and sessions-listing sites. B-1-decisions.md
(Decision 6 OUT-OF-SCOPE block) lists these explicitly. The TODO comment in
path_unification.rs line 33 calls for tightening to line-range-level scoping in a
future pass (WARN-4). `dashboard-v2/src-tauri/src/state.rs` is also whitelisted for
an intentional legacy fallback at array[1] per B-1 spec L94-98.
