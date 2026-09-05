# B-31: Resumable Batched Indexing + Cross-Process Lock + Startup Auto-Resume
Status: SHIPPED (the prior PLANNED marker was stale). Full implementation present
and production-wired: ProjectMetadataStore (knowledge/src/metadata.rs),
IndexLock + LockScope (orchestrator/src/lock.rs), auto_resume_indexed_projects +
graceful_shutdown (orchestrator/src/startup.rs, reached from cli c17_lifecycle +
dashboard-v2 lib.rs), all 5 SSB payloads (ssb.rs), IndexResult::{Cancelled,
Resumed, AlreadyLocked}, cancel_indexing Tauri command + IndexingProgressPanel.
ELAI_DISABLE_INDEXING registered in wire-dark-allowlist.toml (R9). Decisions:
workflow/reasoning/B-31-decisions.md. Audit: workflow/sessions/s-plan-audit-
2026-06-04/plan-status/B/B-31_resumable_indexing_lock_lifecycle.md.
Objective: indexing of a 50K-file repository survives crash mid-run via
per-batch checkpointing; second concurrent ELAI instance cannot
double-index the same project; on MCP/dashboard restart, in-progress
indexing operations auto-resume from the last checkpoint without user
action.
Estimated effort: 2 sessions (~10-14 hours).
Pre-requisites: B-1 (path unification — checkpoints write to canonical
location).
Parallel-safe with: B-29, B-30, B-32, B-33, B-34, B-35.

## Context

Today's IndexingService writes the file-hash table to SQLite once at
completion. Phase 1A audit reports a single happy-path metric (135 files
/ 8075 nodes / 9.67s) but no cancellation, no per-batch resume, no
cross-process safety. If `kill -9` interrupts indexing of a 50K-file
project at 80%, ELAI starts from scratch on the next run. If two ELAI
instances open the same project simultaneously, both will write to the
same DKG/BM25/vector indexes with no coordination — either silently
producing duplicated rows, or crashing one instance with a SQLite
busy-timeout error. If the user kills the dashboard mid-index and reopens
it, the partial index is treated as either "complete" or "missing"
depending on whether `getCollectionInfo` finds rows — but there's no
per-batch progress to surface or resume from.

These are real production failure modes, especially as ELAI is meant to
work on enterprise-scale repos (Empire codebase, multi-million-line
projects). Three SocratiCode patterns address the gap directly:

1. **Per-batch checkpoint with content-hash skip**
   (`src/services/indexer.ts:801-892`). Files processed in 50-file
   batches, after each: persist content_hash → file_path map + indexing
   status to the project metadata row. On restart: load the map, skip
   files whose content_hash matches. SocratiCode's
   `chunked_files.length > 0 && already_processed` predicate makes resume
   automatic.

2. **Cross-process lock via `proper-lockfile`** (`src/services/lock.ts`,
   ~250 LOC). File-based atomic lock at
   `<project>/.elai_cc/.{index|watch}.lock`. PID written + checked for
   liveness on contention. Stale-PID reclaim with SIGTERM (POSIX) /
   `taskkill /F` (Windows). 2-min stale timeout, 30-second refresh.
   Multi-process coordination is a documented requirement
   (CLAUDE.md "Multi-session protocol") but the existing
   `mcp_session_claim` is in-process FSM-session level, not
   indexing-service level.

3. **Startup auto-resume** (`src/services/startup.ts`,
   ~200 LOC). On MCP server boot, scan for projects with
   `indexing_status = "in-progress"` and either resume the index or
   start the watcher with an incremental catch-up. Graceful shutdown
   path coordinates cancellation, watcher stop, lock release with a
   60-second deadline before forced exit.

This B-step ports all three patterns. Existing `mcp_session_claim` API
(per `.claude/rules/multi-session.md`) is preserved for FSM-session
ownership; the new lock layer is below it, at the
indexing-service / watcher service level.

## Reads (mandatory before coding)

- F:\.ELAI_workflow\docs\.source\SocratiCode-main\SocratiCode-main\src\services\indexer.ts
  lines 801-892 (per-batch checkpoint loop in `indexProject`)
- F:\.ELAI_workflow\docs\.source\SocratiCode-main\SocratiCode-main\src\services\indexer.ts
  lines 174-223 (`migrateAbsolutePathKeys` — pattern for one-shot
  migrations after path schema changes)
- F:\.ELAI_workflow\docs\.source\SocratiCode-main\SocratiCode-main\src\services\indexer.ts
  lines 95-145 (cancellation flag + getIndexingProgress)
- F:\.ELAI_workflow\docs\.source\SocratiCode-main\SocratiCode-main\src\services\lock.ts
  (FULL — proper-lockfile wrapper, PID liveness, stale reclamation)
- F:\.ELAI_workflow\docs\.source\SocratiCode-main\SocratiCode-main\src\services\startup.ts
  (FULL — auto-resume + graceful shutdown coordination)
- F:\.ELAI_workflow\docs\.source\SocratiCode-main\SocratiCode-main\src\services\watcher.ts
  lines 96-115, 195-280 (watcher cross-process lock interplay)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\indexing_service.rs
  (FULL — the file we're refactoring)
- F:\.ELAI_workflow\.claude\rules\multi-session.md
  (existing FSM-session protocol — must remain compatible)
- F:\.ELAI_workflow\docs\specs\guide\steps\phase-1A.txt

## Tasks

### Phase 1 — Schema migration (Session 1, half)

1. Extend the project metadata table in DKG SQLite (or per-project
   `metadata.db`):
   ```
   CREATE TABLE IF NOT EXISTS project_metadata (
     project_id TEXT PRIMARY KEY,
     project_path TEXT NOT NULL,
     indexed_files INTEGER NOT NULL DEFAULT 0,
     total_files INTEGER NOT NULL DEFAULT 0,
     indexing_status TEXT NOT NULL,    -- 'in-progress' | 'completed' | 'cancelled'
     last_checkpoint_at INTEGER NOT NULL,
     batch_size INTEGER NOT NULL DEFAULT 50,
     batches_processed INTEGER NOT NULL DEFAULT 0,
     batches_total INTEGER,
     schema_version INTEGER NOT NULL DEFAULT 1
   );
   CREATE TABLE IF NOT EXISTS file_hashes (
     project_id TEXT NOT NULL,
     relative_path TEXT NOT NULL,
     content_hash TEXT NOT NULL,        -- blake3 (per CLAUDE.md anti-stale)
     last_indexed_at INTEGER NOT NULL,
     PRIMARY KEY (project_id, relative_path)
   );
   ```
   Add migration v2 in knowledge crate.

2. Implement `ProjectMetadataStore` in
   `crates/knowledge/src/metadata.rs`:
   - `load(project_id) -> Option<ProjectMetadata>`
   - `save(metadata) -> Result<()>` (idempotent upsert)
   - `update_status(project_id, status) -> Result<()>`
   - `get_file_hashes(project_id) -> BTreeMap<RelPath, blake3::Hash>`
   - `set_file_hash(project_id, rel_path, hash) -> Result<()>`
   - `delete_file_hash(project_id, rel_path) -> Result<()>`
   - `delete_project(project_id) -> Result<()>` (cascading)

### Phase 2 — Resumable batched indexing (Session 1)

3. Refactor `IndexingService::build_index` in
   `crates/orchestrator/src/indexing_service.rs`:
   - Phase A: scan project for indexable files (existing logic).
   - Phase B: for each file, compute `blake3` of content; load existing
     `file_hashes` from `ProjectMetadataStore`. Skip files where hash
     unchanged (resume mode).
   - Phase C: process changed/new files in 50-file batches. After EACH
     batch:
     - Embed all chunks for batch's files (existing code path)
     - Upsert chunks to BM25 + vector + DKG
     - Update file_hashes for batch's files
     - Update `indexed_files`, `batches_processed`, `last_checkpoint_at`
     - Persist with `indexing_status = "in-progress"`
   - Phase D: on completion, set `indexing_status = "completed"`.
   - Cancellation check at every batch boundary
     (`is_cancellation_requested(project_id)`). If true: persist current
     state with `indexing_status = "cancelled"`, return early with
     `IndexResult::Cancelled { batches_completed }`.

4. Update `update_index` (incremental):
   - Same per-batch loop but only over files whose hash changed.
   - Handle deleted files: read all keys from `file_hashes`, compare
     against current file scan, drop hashes + delete chunks for missing.

5. Add `cancel_indexing(project_id) -> bool` API to IndexingService:
   sets a cancellation flag; the next batch boundary picks it up.
   Expose as orchestrator dispatcher tool `cancel_indexing` and as
   Tauri command `cancel_indexing` in dashboard.

6. Migration helper for B-1 path unification: if existing
   `file_hashes` rows have absolute-path keys (from pre-B-1 build),
   strip the project_path prefix in a one-shot migration. Mirror
   SocratiCode's `migrateAbsolutePathKeys` from
   `indexer.ts:174-223`. Idempotent — safe to run twice.

### Phase 3 — Cross-process lock (Session 1, half)

7. Choose lock primitive (CLAUDE.md no-new-deps rule applies):
   - Option A: `fs2` crate (already in Rust ecosystem; locking via
     `flock`/`LockFileEx`)
   - Option B: hand-rolled lock via O_EXCL + PID file
   Prefer Option A if `fs2` is already in the workspace. Otherwise
   propose Option B in the decision doc and ASK user before adding
   `fs2` as new dep.

8. Implement `IndexLock` in
   `crates/orchestrator/src/lock.rs`:
   - `acquire(project_root: &Path, scope: LockScope) -> Result<IndexLockGuard>`
     where `LockScope` is `Index` or `Watch`.
   - Lock file: `<project_root>/.elai_cc/.{index|watch}.lock` with PID
     content.
   - Stale detection: if lock exists and PID is dead (SIGTERM 0 check on
     POSIX, OpenProcess on Windows), reclaim.
   - Refresh: PID file mtime updated every 30s by holder. Stale =
     mtime > 2min.
   - `IndexLockGuard` is RAII — Drop releases.
   - Cross-process aware via `fs2::FileExt::try_lock_exclusive`.

9. Wire IndexLock into `build_index` and `update_index`:
   - Acquire `LockScope::Index` at start. If acquisition fails, log
     "Another process is already indexing this project, skipping" and
     return `IndexResult::AlreadyLocked { holder_pid }`.
   - Release on success or failure (RAII Drop).

10. Wire `LockScope::Watch` into watcher service (existing or new).
    Multiple ELAI instances can have watchers running but only one
    actively patches the index — secondary instances see "active
    (watched by another process)" status.

11. Add diagnostic API
    `IndexLock::is_locked_by_other(project_root, scope) -> Option<u32>`
    returning holder PID if foreign.
    Used by dashboard `codebase_status` equivalent.

### Phase 4 — Startup auto-resume (Session 2)

12. Implement `auto_resume_indexed_projects()` in
    `crates/orchestrator/src/startup.rs`:
    - Called from `cli::main::main` AFTER lock service init, BEFORE
      task dispatch.
    - Called from `dashboard::lib::run` AFTER Tauri setup, BEFORE first
      window.
    - Algorithm (mirror `src/services/startup.ts:37-150`):
      1. Discover all projects that have a populated metadata row in
         the project metadata store. (Multi-project deployments may
         have many.)
      2. For each project:
         - If `indexing_status == "in-progress"`: try to acquire
           `LockScope::Index`. If acquired, resume by calling
           `build_index(project_id, resume=true)`. If lock taken by
           another live PID, log "another process is resuming this
           project" and skip.
         - If `indexing_status == "completed"`: start the file watcher
           (acquire `LockScope::Watch`); the watcher does an immediate
           catch-up `update_index` to capture changes that happened
           while ELAI was off.
         - If `indexing_status == "cancelled"`: leave it; user must
           re-run indexing.
    - Fire-and-forget: each per-project task spawned via tokio. Failures
      logged but never propagate to startup (non-fatal).

13. Implement `graceful_shutdown(signal)` in
    `crates/orchestrator/src/startup.rs`:
    - Signal: `SIGINT`, `SIGTERM`, or "stdin EOF" (dashboard exits).
    - Steps (mirror `src/services/startup.ts:180-207`):
      1. Set global `shutting_down = true` to prevent new tasks.
      2. Cancel all in-progress index operations
         (`cancel_indexing(project_id)` for each).
      3. Stop all watchers (release `LockScope::Watch` locks).
      4. Wait up to 60s for clean completion (per-batch checkpoint
         finishes, locks release).
      5. After deadline: force-release lock files via
         `IndexLock::force_release(project_root)` (writes "force-released
         at <ts> by pid <X>" to log).
    - Hook into signal handler in CLI and Tauri's exit handler in
      dashboard.

14. Add SSB action types:
    - `IndexCheckpoint` payload: `{ project_id, batches_processed, batches_total, files_processed, last_checkpoint_at }` — written every batch
    - `IndexResume` payload: `{ project_id, resumed_from_batch, files_skipped }` — written once on resume
    - `IndexCancel` payload: `{ project_id, batches_completed, files_indexed }` — written on cancel
    - `LockAcquired` / `LockContested` payloads for visibility

15. Dashboard wiring:
    - `IndexingProgressPanel.svelte` shows live `IndexCheckpoint` rows
      (replacing static progress)
    - On resume: banner "Resumed indexing from batch N/M (saved
      X files)"
    - On lock contention: banner "Another ELAI process (PID X) is
      indexing this project; this instance is in read-only mode"

## WIRING_CONTRACT

produces_symbols:
  - knowledge::metadata::ProjectMetadataStore::{load, save, update_status,
    get_file_hashes, set_file_hash, delete_file_hash, delete_project}
  - orchestrator::lock::{IndexLock, IndexLockGuard, LockScope}
  - orchestrator::startup::{auto_resume_indexed_projects, graceful_shutdown}
  - core::ssb::SsbActionType::{IndexCheckpoint, IndexResume, IndexCancel,
    LockAcquired, LockContested} (5 new variants)
  - orchestrator::indexing_service::IndexingService::cancel_indexing
  - orchestrator::indexing_service::IndexResult::{Cancelled, AlreadyLocked,
    Resumed}
consumes_symbols:
  - cli::main::main → calls auto_resume_indexed_projects after init
  - cli::main::handle_signal → calls graceful_shutdown
  - dashboard::lib::run → calls auto_resume_indexed_projects after Tauri
    setup
  - dashboard::lib::on_exit → calls graceful_shutdown
  - dashboard::commands::cancel_indexing → calls IndexingService::cancel_indexing
  - watcher service → acquires LockScope::Watch
  - all IndexingService writers (build_index, update_index) → acquire
    LockScope::Index
reachable_from:
  - cli::main::main (auto-resume + graceful shutdown)
  - dashboard::lib::run (auto-resume + Tauri exit handler)
  - orchestrator::indexing_service::IndexingService::build_index
  - orchestrator::indexing_service::IndexingService::update_index
stub_ban:
  - ProjectMetadata.indexing_status never None at runtime (always one of
    in-progress / completed / cancelled)
  - IndexLockGuard is always RAII-released (Drop guaranteed)
  - graceful_shutdown.deadline_secs == 60 (not None, not 0)
  - auto_resume_indexed_projects called from BOTH cli main and dashboard
    run (verified via grep)
frontend_pair: B-29, B-30 (both use the same per-batch checkpoint surface)
observable_effect:
  trigger: "elai.bat index large-fixture-project; sleep 3; kill -9 elai.exe;
            elai.bat index large-fixture-project"
  expect_ssb: IndexResume action with resumed_from_batch > 0 on the second run
  expect_artifact: total chunks identical to single-pass run; no duplicate
                   chunk IDs in dkg.db; lock file released

## DETERMINISTIC_CHECK

```bash
# 1. Schema migration v2 applied
sqlite3 .elai_cc/.elai_cc/indexes/dkg.db ".tables" | grep -E "project_metadata|file_hashes"
# Expected: both tables present.

# 2. After successful index of fixture project
sqlite3 .elai_cc/.elai_cc/indexes/dkg.db \
  "SELECT indexing_status, batches_processed, batches_total FROM project_metadata WHERE project_id='<fixture-id>';"
# Expected: 'completed' status, batches_processed == batches_total.

# 3. Crash-resume: kill index mid-run, restart, verify identical output
elai.bat index docs/fixtures/large_project &
ELAI_PID=$!
sleep 5
kill -9 $ELAI_PID
elai.bat index docs/fixtures/large_project  # resume
sqlite3 .elai_cc/.elai_cc/indexes/dkg.db \
  "SELECT COUNT(*) FROM dkg_nodes WHERE project_id='<fixture-id>';"
# Expected: matches expected node count for fixture, no duplicates.

# 4. Concurrent process rejection
(elai.bat index project-a &) ; sleep 1
elai.bat index project-a 2>&1
# Expected: stderr "Another process is already indexing this project,
#           skipping" with holder PID.

# 5. Startup auto-resume verification
sqlite3 .elai_cc/.elai_cc/indexes/dkg.db \
  "UPDATE project_metadata SET indexing_status='in-progress' WHERE project_id='<id>';"
elai.bat repl  # cold start; should auto-resume
# Watch SSB: IndexResume action emitted within 5s of startup.

# 6. Graceful shutdown deadline honored
elai.bat index large_project &
sleep 2
kill -TERM $!
# Wait for process exit; verify total time <= 65s (60s deadline + 5s
# slack).

# 7. Lock files cleaned up after Drop
ls .elai_cc/.elai_cc/.index.lock 2>&1
# Expected: not present (or empty + zero PID).
```

## ACCEPTANCE

- [ ] ProjectMetadataStore implemented with idempotent upsert
- [ ] Schema migration v2 applied automatically on first run
- [ ] IndexingService::build_index processes in 50-file batches with
      per-batch checkpoint
- [ ] Cancellation flag honored at every batch boundary
- [ ] blake3 file hashes stored + skipped on resume (CLAUDE.md anti-stale
      compliant)
- [ ] IndexLock + LockScope wired around build_index / update_index /
      watcher
- [ ] Concurrent ELAI instance properly rejected with PID logged
- [ ] auto_resume_indexed_projects called from BOTH cli main and
      dashboard run
- [ ] graceful_shutdown invoked on SIGINT / SIGTERM / dashboard exit;
      60s deadline; force-release after deadline
- [ ] 5 new SSB action types appended (IndexCheckpoint, IndexResume,
      IndexCancel, LockAcquired, LockContested) and surfaced in
      ActivityFeed + IndexingProgressPanel
- [ ] Crash-resume test: kill -9 mid-index → restart → identical
      output, no duplicates
- [ ] Migration `migrateAbsolutePathKeys` runs once after B-1 path
      unification on existing user data
- [ ] No new external dependencies without explicit user confirmation

## Tests to add

- `crates/knowledge/tests/project_metadata.rs` — store CRUD, schema
  migration v2 idempotence
- `crates/orchestrator/tests/indexing_resume.rs` — fixture project, kill
  + resume, assert no-duplicates + identical chunk count
- `crates/orchestrator/tests/index_lock.rs` — concurrent acquire returns
  AlreadyLocked, stale-PID reclamation simulated, RAII Drop releases
- `crates/orchestrator/tests/startup_auto_resume.rs` — populate metadata
  with in-progress status, call auto_resume, assert build_index invoked
  with resume=true
- `crates/orchestrator/tests/graceful_shutdown.rs` — start indexing, send
  cancel signal, assert checkpoint persisted within 60s, IndexCancel SSB
  emitted
- `tests/contracts/indexing_resilience_contract.rs` — regression: any
  build_index/update_index without lock acquisition fails compilation
  (via type-level `#[must_use]` + lock-required marker)

## Files touched

- CREATE: `crates/knowledge/src/metadata.rs`
- CREATE: `crates/knowledge/src/metadata_migration_v2.sql`
- CREATE: `crates/orchestrator/src/lock.rs`
- CREATE: `crates/orchestrator/src/startup.rs` (auto-resume + graceful
  shutdown)
- MODIFY: `crates/orchestrator/src/indexing_service.rs` (per-batch
  checkpoint, cancellation, hash-skip resume mode)
- MODIFY: `crates/cli/src/main.rs::main` (call auto_resume after init,
  install signal handlers)
- MODIFY: `crates/dashboard-v2/src-tauri/src/lib.rs` (V2 — was V1
  `crates/dashboard/src/lib.rs`; call auto_resume in `run()` after the
  Tauri builder; hook on_exit to graceful_shutdown via
  `Builder::on_window_event`)
- MODIFY: `crates/dashboard-v2/src-tauri/src/lib.rs` (V2 — was V1
  `crates/dashboard/src/commands.rs`; add `cancel_indexing` Tauri
  command). Coordinate with B-36 Session 4.
- ADD: `crates/dashboard-v2/src/lib/components/IndexingProgressPanel.svelte`
  (V2 has no equivalent yet — was V1
  `crates/dashboard/svelte/src/IndexingProgressPanel.svelte`. Mount inside
  KnowledgeDrawer.svelte or as a SettingsWindow Project group panel; live
  IndexCheckpoint + Resume/Cancel banners)
- MODIFY: `crates/core/src/ssb/action_types.rs` (5 new variants)
- CREATE: `workflow/reasoning/B-31-decisions.md`

## Decision doc

Write `workflow/reasoning/B-31-decisions.md` documenting:
- Lock primitive choice (fs2 vs hand-rolled O_EXCL); if fs2 isn't already
  workspace, ASK USER before adding
- Batch size: 50 (mirror SocratiCode) vs adaptive (e.g. 25 for huge files,
  100 for small). Default 50; document rationale
- Stale lock timeout: 2min (SocratiCode) vs 5min for ELAI (longer
  embedding latency on local sidecar). Pick a value, document
- Whether file_hashes uses blake3 (CLAUDE.md anti-stale) or sha256 (faster
  but stale-rule-violating). Recommend blake3
- How auto_resume interacts with `mcp_session_register` from
  multi-session.md — ensure FSM-session ownership is preserved when an
  in-progress index resumes (the resuming session should claim the same
  files the original session had)
- Graceful shutdown deadline: 60s (SocratiCode) vs 120s (longer
  embedding batches). Default 60s
- Whether to force-release stale locks on startup or wait for them to
  expire naturally (former is faster, latter is safer for crashed-but-
  recoverable processes)

## Side benefits unlocked

- Multi-million-LOC repos become indexable without "all or nothing"
  fragility
- Empire integration (B-25) needs this since Empire is a large
  multi-language codebase
- Dashboard's IndexingProgressPanel transitions from static to live —
  visible per-batch progress bar
- B-29 symbol graph builds incrementally on watcher path (incremental
  symbol-graph-incremental.ts equivalent — uses the same checkpoint
  primitives)
- Cancellation makes user-initiated stop sane (currently kill -9 is
  the only option)
- Multiple dev sessions on same machine no longer corrupt each other's
  indexes
- ELAI restart no longer requires user to re-run /index manually

## Next

B-29 (Symbol-Level Impact API) — independent, parallel-safe.
B-30 (Multi-Lang DKG) — independent, parallel-safe.
B-32 (Minified File Chunking) — touches the same indexer; can land after
  or alongside B-31's batch refactor.
B-35 (Context Artifacts Pipeline) — depends on B-31 lock primitives so
  artifact indexing can coexist with code indexing.
