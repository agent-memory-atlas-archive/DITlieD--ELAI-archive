# Verification Report: B-31_resumable_indexing_lock_lifecycle.md
Source: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-31_resumable_indexing_lock_lifecycle.md
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
The core B-31 primitives are all implemented: ProjectMetadataStore with BLAKE3
file hashes and idempotent schema migration, batched IndexingService with
per-batch checkpointing and cancellation, IndexLock RAII cross-process lock
with stale-PID reclamation, startup auto-resume, and graceful shutdown with a
60-second deadline. The five SSB action types are defined and round-trip tested.
Two gaps remain: (1) the CLI main never calls auto_resume_indexed_projects or
installs signal handlers — only the dashboard wires these, violating the stub_ban
"called from BOTH cli main and dashboard run"; (2) LockScope::Watch is declared
and tested in isolation but is not wired to any watcher service (Task 10 deferred
by design note in lock.rs:269); (3) the contracts/indexing_resilience_contract.rs
file specified in "Tests to add" does not exist; (4) the dashboard-level lock-
contention banner and IndexResume/IndexCheckpoint/LockContested surfacing in
ActivityFeed are absent — IndexingProgressPanel only polls get_indexing_progress
via Tauri command, with no SSB-driven banner rows.

## Claims verified DONE

- ProjectMetadataStore struct with all 7 specified methods (load, save,
  update_status, get_file_hashes, set_file_hash, delete_file_hash,
  delete_project) — evidence: crates/knowledge/src/metadata.rs:250,377,395,421,443,460,480

- Schema migration v2: project_metadata + file_hashes tables with correct
  column set matching plan spec — evidence: crates/knowledge/src/metadata_migration_v2.sql:12-31

- Migration is idempotent (CREATE TABLE IF NOT EXISTS) — evidence:
  crates/knowledge/src/metadata_migration_v2.sql:8

- DEFAULT_BATCH_SIZE = 50 (mirrors SocratiCode) — evidence:
  crates/knowledge/src/metadata.rs:51

- BLAKE3 content_hash used for file hashes — evidence:
  crates/knowledge/src/metadata.rs:22-24, metadata_migration_v2.sql:28

- IndexingStatus enum: InProgress / Completed / Cancelled with SQL strings
  in-progress / completed / cancelled — evidence:
  crates/knowledge/src/metadata.rs:85-102

- build_index_resumable in indexing_service.rs with per-batch loop over
  50-file chunks, per-batch checkpoint persist, and cancellation check at each
  batch boundary — evidence: crates/orchestrator/src/indexing_service.rs:419-533

- IndexResult enum with Cancelled, AlreadyLocked, Resumed variants —
  evidence: crates/orchestrator/src/indexing_service.rs:168-218

- cancel_indexing(project_id) -> bool on IndexingService —
  evidence: crates/orchestrator/src/indexing_service.rs:959

- cancel_indexing exposed as Tauri command in dashboard-v2/commands.rs:4130 —
  evidence: crates/dashboard-v2/src-tauri/src/commands.rs:4130

- IndexLock::try_acquire with LockScope::Index / LockScope::Watch using
  fd_lock (not fs2) — evidence: crates/orchestrator/src/lock.rs:50,496

- IndexLockGuard RAII Drop releasing fd-lock + deleting lock file —
  evidence: crates/orchestrator/src/lock.rs:39-42, 348-445

- Stale-PID reclamation: Unix kill(0) + Windows OpenProcess —
  evidence: crates/orchestrator/src/lock.rs:13-20, 852

- Lock file body: pid + started_at + scope fields —
  evidence: crates/orchestrator/src/lock.rs:64-67

- Lock file path: <project_root>/.elai_cc/indexes/<scope>.lock —
  evidence: crates/orchestrator/src/lock.rs:27-35

- IndexLock::is_locked_by_other returning Option<u32> holder PID —
  evidence: crates/orchestrator/src/lock.rs:556

- IndexLock wired into build_index_resumable: LockScope::Index acquired at
  start, AlreadyLocked returned on contention —
  evidence: crates/orchestrator/src/indexing_service.rs:179,218,333-334

- IndexLock::force_release implemented —
  evidence: crates/orchestrator/tests/index_lock.rs:133

- auto_resume_indexed_projects in crates/orchestrator/src/startup.rs:265
  with project discovery, in-progress resume, completed watcher start,
  cancelled skip — evidence: crates/orchestrator/src/startup.rs:265,318

- graceful_shutdown in startup.rs:546 with 60-second deadline constant
  (STARTUP_SHUTDOWN_DEADLINE_SECS = 60) — evidence:
  crates/orchestrator/src/startup.rs:53,546

- graceful_shutdown: SHUTTING_DOWN flag set, all cancellations issued, 60s
  deadline, force_release after deadline —
  evidence: crates/orchestrator/src/startup.rs:552,585,606,618

- auto_resume_indexed_projects called from dashboard-v2/lib.rs:582 after
  Tauri setup — evidence: crates/dashboard-v2/src-tauri/src/lib.rs:582

- graceful_shutdown called from dashboard-v2/lib.rs:662 on ExitRequested —
  evidence: crates/dashboard-v2/src-tauri/src/lib.rs:662

- 5 SSB action types defined: IndexCheckpoint, IndexResume, IndexCancel,
  LockAcquired, LockContested — evidence: crates/orchestrator/src/ssb.rs:836,847

- IndexCheckpoint, IndexResume, IndexCancel, LockAcquired, LockContested
  payload structs defined — evidence: crates/orchestrator/src/ssb.rs:2630,2646,2662,2678

- SSB round-trip tests for all 5 variants —
  evidence: crates/orchestrator/tests/ssb_b31_variants.rs:21,36,49,62,75

- IndexResume SSB emitted in indexing_service.rs:675 on resume path —
  evidence: crates/orchestrator/src/indexing_service.rs:675

- LockContested SSB emitted in lock.rs:538 on contention —
  evidence: crates/orchestrator/src/lock.rs:538

- IndexingProgressPanel.svelte created and mounted in KnowledgeDrawer.svelte:124
  with live polling of batches_processed / batches_total / files / cancel button —
  evidence: crates/dashboard-v2/src/lib/components/IndexingProgressPanel.svelte:1-159,
  crates/dashboard-v2/src/lib/drawers/KnowledgeDrawer.svelte:124

- Per-project tests: crates/knowledge/tests/project_metadata.rs covers
  save/load roundtrip, update_status, file_hashes CRUD, delete_project cascade —
  evidence: crates/knowledge/tests/project_metadata.rs

- crates/orchestrator/tests/indexing_resume.rs — crash+resume, no-duplicate —
  evidence: crates/orchestrator/tests/indexing_resume.rs

- crates/orchestrator/tests/index_lock.rs — concurrent acquire returns
  AlreadyLocked, stale-PID reclaim, RAII Drop — evidence:
  crates/orchestrator/tests/index_lock.rs:77,92,129

- crates/orchestrator/tests/startup_auto_resume.rs — in-progress->resume
  path, completed skip, locked skip — evidence:
  crates/orchestrator/tests/startup_auto_resume.rs:119,152,178

- crates/orchestrator/tests/graceful_shutdown.rs — deadline_exceeded assertion
  — evidence: crates/orchestrator/tests/graceful_shutdown.rs:108-111

- workflow/reasoning/B-31-decisions.md exists —
  evidence: workflow/reasoning/B-31-decisions.md

- fd_lock chosen over fs2 (fs2 not added; fd-lock already in workspace) —
  evidence: crates/orchestrator/Cargo.toml:80-86

- scale_resume_identity_contract.rs and scale_lock_contention_contract.rs
  exist — evidence: crates/orchestrator/tests/ listing

## Claims PARTIAL

- LockScope::Watch wired to watcher service (Task 10)
  - exists: crates/orchestrator/src/lock.rs:262-270 (Watch variant declared,
    tested in isolation: tests/index_lock.rs:94; startup.rs:618 calls
    force_release for Watch scope during graceful_shutdown)
  - gap: No watcher service module exists in orchestrator/src/. lock.rs:269
    explicitly documents "Unused in Cycle B; included for API parity."
    Task 10 ("Wire LockScope::Watch into watcher service") is unimplemented.

- Dashboard SSB-driven banners (plan item 15: resume banner + lock contention banner)
  - exists: IndexingProgressPanel.svelte shows live IndexCheckpoint data via
    get_indexing_progress Tauri poll. Cancel button wired. Status badge present.
    crates/dashboard-v2/src/lib/components/IndexingProgressPanel.svelte:1-159
  - gap: No banner "Resumed indexing from batch N/M". No banner "Another ELAI
    process (PID X) is indexing; this instance is in read-only mode". The plan
    required SSB-driven IndexResume and LockContested rows surfaced in
    ActivityFeed; none found in dashboard SVG/TS sources.

## Claims MISSING

- auto_resume_indexed_projects called from cli::main::main (stub_ban requirement)
  — searched: grep auto_resume_indexed_projects crates/cli/src/main.rs +
  full cli/src/ tree; no matches. Only dashboard-v2 wires it.

- cli::main::handle_signal calling graceful_shutdown (WIRING_CONTRACT
  consumes_symbols entry)
  — searched: grep graceful_shutdown,signal,startup crates/cli/src/main.rs;
  no matches. Signal handler wiring absent from CLI entrypoint.

- tests/contracts/indexing_resilience_contract.rs (plan "Tests to add" section)
  — searched: glob crates/contracts/tests/; file not found. The #[must_use] +
  lock-required marker type-level contract test was never created.

- migrateAbsolutePathKeys equivalent (Task 6: one-shot migration stripping
  absolute-path keys from pre-B-1 file_hashes rows)
  — searched: grep migrateAbsolutePathKeys,absolute_path.*migration,migrate_absolute
  across .elai_cc/; no matches. The migration helper is absent.

## Claims UNVERIFIABLE

- Observable-effect test: "kill -9 elai.exe; second run emits IndexResume with
  resumed_from_batch > 0" (DETERMINISTIC_CHECK block) — verifying requires live
  process execution; infeasible from static analysis.

- "Concurrent process rejection" stderr message text exactly matching
  "Another process is already indexing this project, skipping" — the code
  structure is correct (AlreadyLocked path) but exact log-line text requires
  runtime verification.

- Schema migration v2 applied automatically on first run in production (requires
  live DB open call against an uninitialized metadata.db).

## What must be done (action list)

- [ ] Wire auto_resume_indexed_projects into cli::main (crates/cli/src/main.rs),
  called AFTER init, BEFORE task dispatch, matching the WIRING_CONTRACT stub_ban.

- [ ] Install SIGINT/SIGTERM signal handlers in cli::main calling graceful_shutdown
  (crates/cli/src/main.rs); stub_ban requires this from BOTH entrypoints.

- [ ] Implement LockScope::Watch wiring in a watcher service (Task 10). Lock.rs
  declares Watch as reserved; no watcher service module exists in
  crates/orchestrator/src/.

- [ ] Create tests/contracts/indexing_resilience_contract.rs (specified in plan
  "Tests to add") — type-level #[must_use] + lock-required marker regression.

- [ ] Implement migrateAbsolutePathKeys equivalent in knowledge::metadata (Task 6)
  — one-shot idempotent migration to strip absolute-path prefixes from
  file_hashes rows written before B-1 path unification.

- [ ] Add dashboard SSB-driven banners: IndexResume banner "Resumed indexing from
  batch N/M (saved X files)" and LockContested banner "Another ELAI process
  (PID X) is indexing; read-only mode". Plan item 15 specifies these as
  required observable effects surfaced in ActivityFeed.

## Notes
fd-lock was chosen over fs2 (comment in Cargo.toml:80 documents the decision).
This deviates from the plan's Option A (fs2) but is valid — fd_lock was already
in the workspace as a transitive dep, consistent with no-new-deps rule. The
STALE_CHECKPOINT_TIMEOUT_SECS constant (120 = 2min) matches the plan. The
startup.rs uses filesystem-walk discovery rather than a project registry, which
the file self-documents as a deliberate adaptation from the SocratiCode reference.
The scale_* contract tests (scale_resume_identity_contract.rs,
scale_lock_contention_contract.rs, scale_long_tail_memory_contract.rs) exist in
orchestrator/tests/ and are beyond the plan's original test list, suggesting
additional coverage was added during implementation.
