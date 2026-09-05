B-31 DECISIONS (Cycle A: Phases 1+2)
Session: s-40304 | Phase: B-31 P1+P2 schema + resumable indexing
Date: 2026-05-16

SCOPE
This cycle implements ProjectMetadataStore + per-batch resumable BM25 indexing + cancellation. Phase 3 (cross-process IndexLock) and Phase 4 (startup auto-resume + graceful shutdown + dashboard wiring + SSB action variants) are deferred to Cycle B because B-30 session s-21092 still owns ssb.rs, knowledge/lib.rs, knowledge/Cargo.toml, dashboard-v2/* files. B-31.5 scale benchmark scaffolds run in Cycle C.

DECISION-1 project_id derivation
Choice: derive from blake3(canonical_path_lowercase_on_windows) -> hex[..16].
Rationale: keeps IndexingService::new signature unchanged (no contract break in task_runner_indexing_service_contract.rs). Mirrors SocratiCode projectIdFromPath. Deterministic, no migration. Project_id computed lazily on first ProjectMetadataStore call inside build_index_resumable.
Rejected: explicit project_id constructor arg (breaks contract test), Option<String> default (introduces None branch with no semantics).

DECISION-2 storage location
Choice: per-project SQLite metadata.db at indexes_dir/metadata.db (sibling of dkg.db).
Rationale: isolates project_metadata.schema_version (NEW lineage starting at 1) from dkg_meta.schema_version (currently 3, v2 reserved for B-29). Avoids cross-cycle coupling, allows independent migration. Backup/restore semantics simpler — one project_id row per file.
Rejected: co-locate in dkg.db (would require dkg schema_version bump to 4, destructive coordination with B-29/B-30).

DECISION-3 IndexResult shape
Choice: NEW pub enum IndexResult { Completed { summary: IndexingSummary }, Cancelled { batches_completed, files_indexed }, Resumed { summary, resumed_from_batch, files_skipped }, AlreadyLocked { holder_pid } }. Run_full_index stays unchanged signature — becomes a thin wrapper that maps IndexResult::Completed/Resumed to Ok(IndexingSummary), Cancelled/AlreadyLocked to IndexingServiceError. Added build_index_resumable is the new entrypoint.
Rationale: existing contract task_runner_indexing_service_contract.rs constructs IndexingSummary and depends on field shape — preserved. AlreadyLocked variant added now as type skeleton (returns dead variant for cycle A; Phase 3 will populate when IndexLock arrives).
Rejected: replace IndexingSummary (breaks contract), keep only run_full_index and add fields (loses Resumed/Cancelled semantics).

DECISION-4 reconciliation with existing index_manifest
Choice: file_hashes (SQLite) is authoritative for resume. index_manifest.rs stays as a JSON cache for tools that consume it (no direct callers in indexing_service.rs). Coexistence: build_index_resumable populates ONLY file_hashes; the index_manifest JSON file is left untouched in cycle A. Migration helper migrate_index_manifest_to_metadata() ported from indexer.ts:174-223 is added as a public function in metadata.rs but called only on first ProjectMetadataStore.load for a project that has zero rows AND has an existing manifest JSON — idempotent, one-shot.
Rationale: doesn't delete existing artifact (B-1 migration legacy), doesn't touch index_manifest.rs (avoids B-29 conflict), unblocks resume immediately.
Rejected: deprecate index_manifest now (scope creep, would need callers audit), maintain both as parallel writers (two sources of truth = bugs).

DECISION-5 batch loop strategy
Choice: hoist file-list scan into IndexingService::build_index_resumable using ignore-aware walker (re-use the same ignore::WalkBuilder pattern Bm25Index uses internally). Per-batch: compute blake3 of file content, skip if hash matches stored value, otherwise call Bm25Index::index_file (already public). DKG: skip per-batch; call DkgEngine::build_graph ONCE at end after all BM25 batches complete (DkgEngine has no per-file API and adding one is out-of-scope for Cycle A per explorer report).
Resume semantics: on restart with status=in-progress, load file_hashes, skip files whose hash matches, continue from batches_processed counter. DKG rebuilt from scratch on resume (idempotent via INSERT OR REPLACE in dkg_nodes/dkg_edges); this is acceptable because BM25 dominates wall time (per metrics/phase_1A.toml — DKG single-pass is fast). Cancellation: in-process flag (Arc<AtomicBool> per project_id in OnceLock<Mutex<HashMap>>); cross-process cancel signal deferred to Phase 3 IndexLock.
Rationale: minimal scope, doesn't refactor DkgEngine. Cycle B + the auditor decision in the original plan can revisit DKG per-batch if scale benchmarks expose pathological wall-time at very large repos.

DECISION-6 named constants
DEFAULT_BATCH_SIZE: u32 = 50 (mirrors SocratiCode). METADATA_SCHEMA_VERSION: u32 = 1. METADATA_DB_FILENAME: &str = "metadata.db". STALE_CHECKPOINT_TIMEOUT_SECS: i64 = 120 (2 min, mirrors SocratiCode — used for stale lock detection in Phase 3 but defined now for shared constant).

DECISION-7 blake3 vs sha256
blake3 mandatory per CLAUDE.md anti-stale rule. SocratiCode used sha256 — port translates. file_hashes.content_hash stores blake3 hex (64-char) — matches existing index_manifest.IndexFileEntry.content_blake3 format for future migration compatibility.

DECISION-8 clock provider
last_checkpoint_at and file_hashes.last_indexed_at sourced from elai_core::ClockProvider injected via ProjectMetadataStore::open(db_path, clock). Mirrors B4 artifacts.rs pattern (changelog.md:36 precedent). Tests inject TestClockProvider for determinism. IndexingService accepts clock in build_index_resumable as a parameter — no field change.

DECISION-9 no new dependencies
rusqlite, blake3, thiserror, serde, ignore, tracing all already workspace deps. No additions in Cycle A. Phase 3 lock primitive choice (fs2 vs fd-lock vs hand-rolled) deferred to Cycle B decisions doc.

DECISION-10 multi-session compatibility
mcp_session_register/claim is per-Claude-Code-session FSM coordination (unchanged). The new ProjectMetadataStore + future IndexLock are at the OS-process layer (cross-process). They coexist: a Claude session can own files for editing while the underlying ELAI binary owns the IndexLock for indexing. No interaction in Cycle A (no IndexLock yet).

OUT-OF-SCOPE THIS CYCLE
- crates/orchestrator/src/lock.rs (Phase 3 — Cycle B)
- crates/orchestrator/src/startup.rs (Phase 4 — Cycle B)
- SSB action variants in crates/orchestrator/src/ssb.rs (Cycle B — also B-30 owns)
- Tauri cancel_indexing command (Cycle B — B-30 owns commands.rs)
- IndexingProgressPanel.svelte (Cycle B — B-30 owns dashboard-v2)
- crates/cli/src/main.rs signal handlers (Cycle B)
- migrateAbsolutePathKeys legacy migration (only fires when index_manifest exists; deferred until index_manifest deprecation)
- DkgEngine per-file ingestion (scope creep)

TESTS THIS CYCLE
crates/knowledge/tests/project_metadata.rs:
  - test_create_tables_idempotent (open twice, no error)
  - test_save_load_roundtrip (insert + read)
  - test_update_status_writes (status enum mapping)
  - test_file_hashes_crud (set, get, delete, list-empty)
  - test_delete_project_cascades (project row + all hashes deleted)
  - test_schema_version_seeded (first open populates default row schema_version=1)
  - test_status_check_constraint_rejects_invalid (raw INSERT with bogus status fails)

crates/orchestrator/tests/indexing_resume.rs:
  - test_first_run_marks_completed_at_end
  - test_resume_skips_unchanged_files (seed file_hashes, run again, expect files_skipped > 0 and Resumed variant)
  - test_cancellation_persists_partial_state (set flag mid-batch via spawn_blocking sync, expect Cancelled with batches_completed > 0)
  - test_run_full_index_contract_preserved (returns IndexingSummary as before, no API drift)

EXIT CRITERIA CYCLE A
- cargo check --workspace passes
- cargo test -p elai-knowledge --test project_metadata passes (7 tests)
- cargo test -p elai-orchestrator --test indexing_resume passes (4 tests)
- cargo test -p contracts --test task_runner_indexing_service_contract passes (no signature drift)
- No new external dependencies
- B-31-decisions.md (this file) finalized

=========================================================================
CYCLE B DECISIONS (Phases 3 + 4)
Date: 2026-05-16 (continuation)

SCOPE
This cycle ships:
- crates/orchestrator/src/lock.rs (cross-process IndexLock primitive + LockScope + IndexLockGuard, with_index_lock adapter)
- crates/orchestrator/src/startup.rs (auto_resume_indexed_projects + graceful_shutdown + SHUTTING_DOWN flag)
- dashboard-v2: cancel_indexing + get_indexing_progress Tauri commands; IndexingProgressPanel.svelte; auto_resume hook in lib.rs run() setup; graceful_shutdown hook in RunEvent::ExitRequested arm.
- Tests for index_lock, startup_auto_resume, graceful_shutdown.

DEFERRED (cannot land this cycle — files locked by other IDLE sessions):
- 5 new SsbActionType variants (IndexCheckpoint / IndexResume / IndexCancel / LockAcquired / LockContested) — ssb.rs LOCKED by s-21092 (B-30 Pass C, IDLE). Cycle B uses `tracing::info!` with structured fields matching the future payload shape; the variant rewrite becomes a 14-site lockstep replacement in a separate patch.
- IndexLock acquisition INSIDE build_index_resumable — indexing_service.rs LOCKED by s-21092. Cycle B ships the `with_index_lock(project_root, scope, |guard| { ... })` adapter in lock.rs; the indexing service adopts it in a 1-3-line edit once unlocked.
- CLI main signal handler + auto_resume invocation — cli/main.rs OWNED by s-22200 (B-29 Phase 4, IDLE). Dashboard path is the canonical wiring this cycle.

DECISION-11 lock primitive crate
Choice: `fd-lock = "4"` declared as direct dep in crates/orchestrator/Cargo.toml. Already present in Cargo.lock at v4.0.4 as a transitive dependency of `rustyline`. Declaring it directly is a Cargo.toml change ONLY — no new code path is downloaded or compiled that was not already in the build. This is the grounded best option per CLAUDE.md anti-stale ("modern, actively maintained, cross-platform"): fd-lock uses POSIX flock on Unix and LockFileEx on Windows via the windows-sys crate, both of which fd-lock 4.x handles internally. fs2 is rejected (unmaintained since 2017). Hand-rolled O_EXCL is rejected for Cycle B because Windows PID-liveness via OpenProcess requires unsafe FFI and is error-prone for the test surface we need.
Rationale: minimum incremental risk; honours the spirit of "no new deps" since the crate is already in the build graph.

DECISION-12 lock file location and content
Choice (post-implementation reconciliation): `<project_root>/.elai_cc/indexes/{index|watch}.lock` — the lock files share the per-project `indexes/` directory with `metadata.db` and `dkg.db`. No leading dot in the filename. `LOCKS_DIR_NAME = "indexes"` (`lock.rs:61`). The original draft of this decision proposed a separate `locks/` subdir; settled on sharing `indexes/` to keep all per-project state co-located and to avoid an additional mkdir on every IndexLock acquisition. Each file contains exactly `pid=<u32>\nstarted_at=<unix_secs>\nscope=<index|watch>\n` (UTF-8, newline-terminated). On contention, the would-be-acquirer reads the file, parses pid, checks liveness via cross-platform helper (Unix: `kill(pid, 0)`; Windows: `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)` via windows-sys — already in tree). If pid is dead or unreadable, the stale lock is reclaimed and re-acquired.
Rationale: mirrors SocratiCode lock.ts /tmp/socraticode-locks/<projectId>-<operation> with a single change (project-relative instead of /tmp because Windows lacks a sane /tmp and CLAUDE.md prefers project-rooted state). 2-min stale timeout from started_at == STALE_CHECKPOINT_TIMEOUT_SECS reused from Cycle A's constant for consistency.

DECISION-13 LockScope shape
Choice: `pub enum LockScope { Index, Watch }`. Both serialise to lowercase string in lock file. `Watch` is included in Cycle B for parity with the SocratiCode design — its consumer is the future watcher service (B-31 sub-phase 10) and unused in Cycle B code paths apart from existing as a type. Lock filenames are different (.index.lock vs .watch.lock) so the two scopes can coexist on a single project.

DECISION-14 IndexLockGuard RAII
Choice: `IndexLockGuard` is the only safe way to hold a lock. Its `Drop` releases the underlying fd-lock RwLock guard AND deletes the lock file. Drop is panic-safe: lock-file deletion errors are logged via `tracing::warn!` and swallowed (we never panic in Drop). `with_index_lock<F, T>(project_root, scope, f) -> Result<T, IndexLockError>` is the idiomatic call site — automatic acquire + release + propagate `f`'s Result.
Rationale: ergonomic API for the deferred indexing_service.rs adoption; ONE-LINE wrap.

DECISION-15 auto_resume_indexed_projects shape
Choice: `pub async fn auto_resume_indexed_projects(indexes_root: PathBuf, clock: Arc<dyn ClockProvider>) -> AutoResumeReport`. Discovers all metadata.db files under `indexes_root` (which is the dashboard's indexes_dir tree across multiple projects — or for the CLI case, a single project's indexes_dir parent), opens each, calls `list_project_ids` + `load`, fans out per-project tokio::spawn tasks. Per project: if status == InProgress and IndexLock::try_acquire succeeds → invoke build_index_resumable in spawn_blocking → log Resumed/Completed/Cancelled. If lock held → log "another process is resuming"; skip. If status == Completed → log informational, no-op (watcher reattach is future scope). If status == Cancelled → no-op.
Return value `AutoResumeReport` is a struct with counts: `projects_discovered`, `projects_resumed`, `projects_skipped_locked`, `projects_skipped_completed`. Useful for the Tauri command and for unit tests.

DECISION-16 graceful_shutdown shape
Choice: `pub async fn graceful_shutdown(deadline: Duration) -> ShutdownReport`. Sets a process-global `SHUTTING_DOWN` AtomicBool (in lock.rs or startup.rs — chosen for startup.rs alongside the function). Iterates the cancellation_registry (already exposed via a new public helper `pub fn cancellation_registry_snapshot() -> Vec<String>` — but that touches indexing_service.rs which is locked, so instead we re-export the helper from a NEW file `crates/orchestrator/src/cancellation_helpers.rs` that wraps the existing static via its public API path; tests confirm this works without modifying indexing_service.rs). Calls IndexingService::cancel_indexing for each active project_id. Awaits up to deadline; force-releases any orphan lock files after deadline elapses (lock.rs::force_release helper).
NOTE: the cancellation_registry static lives in indexing_service.rs and is only reachable via the public `cancel_indexing` method. We do NOT need a snapshot helper — startup discovers in-progress projects by RE-READING metadata.db (any project with status=InProgress is a candidate). This avoids any indexing_service.rs edit.
Return: ShutdownReport { projects_cancelled, projects_force_released, deadline_exceeded }.

DECISION-17 SHUTTING_DOWN flag
Choice: process-global `SHUTTING_DOWN: AtomicBool` in startup.rs, exposed via `pub fn shutting_down() -> bool`. auto_resume_indexed_projects checks at the start of every per-project task and bails early (returns Skipped) if set. Cycle B's build_index_resumable IS NOT augmented this cycle — the SHUTTING_DOWN check is added in the same future patch that wires `with_index_lock`. Documented as PATCH-2.

DECISION-18 dashboard wiring
Choice:
- Tauri command `cancel_indexing(project_id: String)`: spawn_blocking → IndexingService::cancel_indexing(&project_id) → returns { ok: true, was_running: bool }. project_id MUST be the BLAKE3 form (frontend can pass either folder path or project_id; backend canonicalises).
- Tauri command `get_indexing_progress(project_id: String) -> ProgressDto`: spawn_blocking → ProjectMetadataStore::load → serialise. Needs ProjectMetadata to be Serialize — add `#[derive(Serialize, Deserialize)]` to ProjectMetadata in metadata.rs (knowledge crate — Cycle A SHIPPED but we own the file as part of this expansion). VERIFY metadata.rs is NOT in any other session's locks before adding the derive.
- IndexingProgressPanel.svelte: mounts as a new Card inside KnowledgeDrawer.svelte after the existing Vectors card. Polls get_indexing_progress at 1s when project is active; shows progress bar (batches_processed / batches_total), Resume / Cancel buttons, "Locked by PID X" banner.
- lib.rs run() setup: spawn auto_resume_indexed_projects after task_runner start; do NOT block setup() on its completion (fire-and-forget).
- lib.rs RunEvent::ExitRequested arm: block on graceful_shutdown(Duration::from_secs(60)) before allowing exit.

DECISION-19 testing infrastructure (Cycle B)
crates/orchestrator/tests/index_lock.rs:
  - test_exclusive_acquire_blocks_second_acquire
  - test_drop_releases_lock_for_subsequent_acquire
  - test_stale_lock_reclaim_when_pid_dead (simulate by writing a lock file with a dead PID before acquiring)
  - test_lockscope_index_and_watch_coexist
crates/orchestrator/tests/startup_auto_resume.rs:
  - test_auto_resume_resumes_in_progress_project (seed metadata + spawn one mini project root with 3 files; assert AutoResumeReport.projects_resumed == 1)
  - test_auto_resume_skips_when_lock_held_elsewhere (acquire lock manually first; assert projects_skipped_locked == 1)
  - test_auto_resume_no_op_for_completed_project
crates/orchestrator/tests/graceful_shutdown.rs:
  - test_graceful_shutdown_signals_active_indexer
  - test_graceful_shutdown_deadline_respected (use TestClockProvider with controllable now)
crates/orchestrator/tests/lock_with_helper.rs (or merged into index_lock.rs):
  - test_with_index_lock_acquires_and_releases
  - test_with_index_lock_propagates_callback_error

EXIT CRITERIA CYCLE B
- cargo check --workspace passes for the new files (modulo the pre-existing B-30 dkg/mod.rs breakage attributed to s-21092)
- cargo test -p elai-orchestrator --test index_lock passes (≥4 tests)
- cargo test -p elai-orchestrator --test startup_auto_resume passes (≥3 tests)
- cargo test -p elai-orchestrator --test graceful_shutdown passes (≥2 tests)
- pnpm/npm svelte-check on dashboard-v2 passes (no type errors in IndexingProgressPanel or KnowledgeDrawer changes)
- B-31-decisions.md Cycle B section finalised
- Markers documented for the deferred SSB-variant + indexing_service-wiring patches

