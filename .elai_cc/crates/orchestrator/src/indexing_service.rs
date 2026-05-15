//! A-44: IndexingService — coordinates DKG + BM25 index building and
//! incremental updates for the project knowledge pipeline.
//!
//! B-31 Phase 2 (Cycle A) added resumable, batched BM25 indexing on top of
//! the existing single-shot path. See `workflow/reasoning/B-31-decisions.md`
//! for the locked design choices. The new entrypoint is
//! [`IndexingService::build_index_resumable`]; the legacy
//! [`IndexingService::run_full_index`] now wraps it so its public signature
//! (`-> Result<IndexingSummary, IndexingServiceError>`) stays contract-stable
//! for `crates/contracts/tests/task_runner_indexing_service_contract.rs`.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use elai_core::{ClockProvider, OsClock};
use elai_knowledge::bm25::Bm25Index;
use elai_knowledge::chunking;
use elai_knowledge::index_version::{self, UpgradeAction};
use elai_knowledge::metadata::{
    self, IndexingStatus, MetadataError, ProjectMetadata, ProjectMetadataStore,
    DEFAULT_BATCH_SIZE, METADATA_DB_FILENAME, METADATA_SCHEMA_VERSION,
};
use elai_knowledge::{DkgEngine, KnowledgeError};

use crate::ssb::{
    IndexCancelPayload, IndexCheckpointPayload, IndexResumePayload,
    MinifiedFileChunkedPayload, Ssb, SsbPayload,
};

// ---------------------------------------------------------------------------
// Named constants (no magic numbers)
// ---------------------------------------------------------------------------

// B-1 Path Unification: ELAI_DATA_DIR_NAME / INDEXES_DIR_NAME live in
// `elai_core::paths` as the single source of truth. Re-exported here so
// existing contract tests at
// `crates/contracts/tests/task_runner_indexing_service_contract.rs` (which
// import them from `elai_orchestrator::indexing_service`) keep compiling
// without edits.
pub use elai_core::paths::{ELAI_DATA_DIR_NAME, INDEXES_DIR_NAME};

/// Number of bytes to read from a file's head to detect binary content.
/// Must stay in sync with elai_knowledge::bm25::BINARY_DETECT_BYTES (both 8192).
/// This is a performance pre-filter to avoid BM25 delete+re-add for obviously
/// binary files; BM25's chunk_file also detects binary internally.
pub const BINARY_DETECT_BYTES: usize = 8192;

/// DKG SQLite database filename within the indexes directory.
/// Mirrors `elai_core::paths::DKG_DB_FILENAME`; kept private to this
/// module so callers go through `IndexingService::dkg_db_path()` or the
/// canonical `elai_core::paths::dkg_db_path` helper.
const DKG_DB_FILENAME: &str = elai_core::paths::DKG_DB_FILENAME;

/// BM25 project ID used when creating the BM25 index inside indexes_dir.
const BM25_PROJECT_ID: &str = "default";

/// Buffer size (bytes) for the streaming BLAKE3 hasher used in
/// [`hash_file_contents`]. 64 KiB keeps the call-count low without
/// ballooning per-file allocations.
const HASH_BUFFER_BYTES: usize = 64 * 1024;

/// Directories excluded from indexing by default.
const DEFAULT_EXCLUDED_DIRS: &[&str] = &[
    "node_modules",
    "target",
    ".git",
    "__pycache__",
    "dist",
    "build",
];

/// File patterns excluded from indexing by default.
const DEFAULT_EXCLUDED_FILES: &[&str] = &[
    ".env",
];

/// File extensions excluded from indexing by default.
const DEFAULT_EXCLUDED_EXTENSIONS: &[&str] = &[
    "key",
    "pem",
];

/// File name prefixes excluded from indexing by default.
const DEFAULT_EXCLUDED_PREFIXES: &[&str] = &[
    "credentials.",
    "secrets.",
];

/// Batch 1C (A-44): emergency shutoff for the per-project first-task
/// indexing call site. When set to a non-zero truthy value (`1` / `true` /
/// `yes` / `on`), `indexing_disabled_via_env` returns `true` and
/// `task_runner::maybe_build_project_index` short-circuits before acquiring
/// the service mutex. Mirrors the `ELAI_HOOKS_DISABLED` / `ELAI_DISABLE_
/// PROVIDER_SIGNALS` pattern from Batch 1A/1B.
pub const ELAI_DISABLE_INDEXING_ENV: &str = "ELAI_DISABLE_INDEXING";

/// B-31.5 (DECISION-B31.5-3): synchronous per-batch sleep hook used by the
/// Tier D scale benchmark to simulate a "stuck batch" so we can verify that
/// `graceful_shutdown` honours its deadline and force-releases the lock.
///
/// When the env var is set to a positive integer, `build_index_resumable`
/// invokes `std::thread::sleep(Duration::from_secs(v))` at the TOP of each
/// batch iteration (before the cancellation check, so the sleeping batch
/// blocks the shutdown wait loop until the deadline expires). Zero / unset /
/// unparseable values are treated as "no delay".
///
/// Mirrors the [`ELAI_DISABLE_INDEXING_ENV`] named-constant pattern.
pub const ELAI_DEBUG_BATCH_DELAY_SECS_ENV: &str = "ELAI_DEBUG_BATCH_DELAY_SECS";

/// Read the per-batch debug delay from the environment. Returns `0` when
/// the env var is unset, empty, zero, or unparseable — guarantees the hot
/// path is a single `Atomic`-free read with no allocation in the common
/// case.
fn debug_batch_delay_secs() -> u64 {
    std::env::var(ELAI_DEBUG_BATCH_DELAY_SECS_ENV)
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .unwrap_or(0)
}

/// Batch 1C (A-44): freshness window for the per-project indexing marker.
/// When a project's cached `indexed_at` is older than this many seconds, the
/// runner re-runs `run_full_index` (or skips into the on-disk-already path).
/// 24 hours is a conservative default that matches the "daily developer
/// cadence" assumption baked into `mvp-wiring-audit-2026-04-18.md` section
/// 3 A-13 (RAG freshness).
pub const INDEX_FRESHNESS_SECS: i64 = 86_400;

/// Returns `true` when the operator has disabled indexing via the
/// [`ELAI_DISABLE_INDEXING_ENV`] env var. Truthy values: `"1"`, `"true"`,
/// `"yes"`, `"on"` (case-insensitive).
pub fn indexing_disabled_via_env() -> bool {
    std::env::var(ELAI_DISABLE_INDEXING_ENV)
        .map(|v| {
            let v = v.trim().to_ascii_lowercase();
            matches!(v.as_str(), "1" | "true" | "yes" | "on")
        })
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum IndexingServiceError {
    #[error("knowledge error: {0}")]
    Knowledge(#[from] KnowledgeError),

    #[error("index version error: {0}")]
    IndexVersion(#[from] index_version::IndexVersionError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("BM25 error: {0}")]
    Bm25(String),

    #[error("service not initialized — run run_full_index first")]
    NotInitialized,

    /// B-31 Phase 2: indexing was cancelled via [`IndexingService::cancel_indexing`].
    /// Partial state is preserved; subsequent runs resume from the last
    /// checkpoint.
    #[error("indexing cancelled (batches_completed={batches_completed}, files_indexed={files_indexed})")]
    Cancelled {
        batches_completed: u32,
        files_indexed: u64,
    },

    /// B-31 Phase 3 (placeholder for Cycle B): another process already holds
    /// the index lock. Not constructible from Cycle A code paths; the variant
    /// exists so [`IndexResult::AlreadyLocked`] has a corresponding error
    /// shape when [`run_full_index`] unwraps it.
    #[allow(dead_code)]
    #[error("indexing lock already held by pid={holder_pid}")]
    AlreadyLocked { holder_pid: u32 },

    /// B-31 Phase 1: metadata-store failure (open/load/save/upsert/migrate).
    #[error("metadata error: {0}")]
    Metadata(#[from] MetadataError),
}

// ---------------------------------------------------------------------------
// IndexResult (B-31 Phase 2)
// ---------------------------------------------------------------------------

/// Outcome of [`IndexingService::build_index_resumable`].
///
/// `Completed` and `Resumed` both wrap an [`IndexingSummary`] so callers that
/// only want the aggregate counts can ignore the variant. `Cancelled` and
/// `AlreadyLocked` are exceptional paths — they MAY be mapped to
/// [`IndexingServiceError`] variants by the legacy [`run_full_index`] wrapper.
#[derive(Debug, Clone)]
pub enum IndexResult {
    /// Fresh first run finished without prior state.
    Completed { summary: IndexingSummary },
    /// Cancellation flag was set mid-batch. `batches_completed` is the number
    /// of batches that finished and were checkpointed before the abort.
    Cancelled {
        batches_completed: u32,
        files_indexed: u64,
    },
    /// Run found existing `file_hashes` rows and reused them. `resumed_from_batch`
    /// is the value of `batches_processed` at resume time; `files_skipped` is
    /// the number of files whose stored hash matched the current content.
    Resumed {
        summary: IndexingSummary,
        resumed_from_batch: u32,
        files_skipped: u64,
    },
    /// Cross-process lock contention — Phase 3 will populate this. Skeleton
    /// only in Cycle A; suppressed from clippy so the workspace builds clean
    /// until [`crate::lock::IndexLock`] arrives.
    #[allow(dead_code)]
    AlreadyLocked { holder_pid: u32 },
}

// ---------------------------------------------------------------------------
// In-process cancellation registry
// ---------------------------------------------------------------------------

/// Global, per-project cancellation flags. `IndexingService::cancel_indexing`
/// sets the flag; `build_index_resumable` checks it at every batch boundary.
///
/// Living in a `OnceLock` rather than as an instance field on
/// [`IndexingService`] mirrors `setIndexingProgress` / `cancellationRequested`
/// from the SocratiCode reference (`indexer.ts:95-145`) — multiple `&mut`
/// callers and a separate `&` canceller need access to the same flag without
/// constructor changes.
fn cancellation_registry() -> &'static Mutex<HashMap<String, Arc<AtomicBool>>> {
    static REG: OnceLock<Mutex<HashMap<String, Arc<AtomicBool>>>> = OnceLock::new();
    REG.get_or_init(|| Mutex::new(HashMap::new()))
}

fn cancellation_flag_for(project_id: &str) -> Arc<AtomicBool> {
    let mut reg = match cancellation_registry().lock() {
        Ok(g) => g,
        // Poison is recovered by clearing the map — losing in-flight flags is
        // safer than panicking the indexing thread.
        Err(poison) => poison.into_inner(),
    };
    reg.entry(project_id.to_string())
        .or_insert_with(|| Arc::new(AtomicBool::new(false)))
        .clone()
}

fn take_cancellation_flag(project_id: &str) -> Option<Arc<AtomicBool>> {
    let reg = cancellation_registry().lock().ok()?;
    reg.get(project_id).cloned()
}

// ---------------------------------------------------------------------------
// IndexingService
// ---------------------------------------------------------------------------

/// Coordinates DKG graph building and BM25 indexing for a project.
///
/// Created in CLI startup, lives for the duration of the session.
/// On first run (no indexes directory), builds everything from scratch.
/// On subsequent runs, checks schema version and rebuilds if needed.
/// Incremental updates are triggered by SSB file_write events.
pub struct IndexingService {
    project_root: PathBuf,
    indexes_dir: PathBuf,
    bm25: Option<Bm25Index>,
    dkg_db_path: PathBuf,
    initialized: bool,
    /// B-32: optional SSB writer used to emit one
    /// [`MinifiedFileChunkedPayload`] row per minified file detected during
    /// `build_index_resumable`. Callers that don't have an SSB writer (tests,
    /// cold-start contract checks) leave this as `None` — the post-build
    /// walkback then becomes a no-op so existing call sites are unaffected.
    ssb_writer: Option<Arc<Ssb>>,
}

impl IndexingService {
    /// Create a new IndexingService. Does NOT build indexes — call
    /// `run_full_index` to populate them.
    pub fn new(project_root: PathBuf, indexes_dir: PathBuf) -> Self {
        let dkg_db_path = indexes_dir.join(DKG_DB_FILENAME);
        Self {
            project_root,
            indexes_dir,
            bm25: None,
            dkg_db_path,
            initialized: false,
            ssb_writer: None,
        }
    }

    /// B-32: attach an SSB writer so `build_index_resumable` can emit
    /// [`crate::ssb::SsbActionType::MinifiedFileChunked`] rows after the BM25
    /// batch loop completes. Additive builder — existing
    /// [`IndexingService::new`] callers keep the no-op behaviour.
    pub fn with_ssb_writer(mut self, ssb: Arc<Ssb>) -> Self {
        self.ssb_writer = Some(ssb);
        self
    }

    /// Whether any index artifacts exist on disk.
    pub fn indexes_exist(&self) -> bool {
        self.dkg_db_path.exists()
    }

    /// Build DKG graph and BM25 index from scratch.
    ///
    /// Thin wrapper over [`Self::build_index_resumable`] preserving the
    /// pre-B-31 return signature. Internally opens the per-project
    /// [`ProjectMetadataStore`] alongside `dkg.db` and threads an `OsClock`
    /// through. `IndexResult::{Completed, Resumed}` map to `Ok`;
    /// `Cancelled` / `AlreadyLocked` map to `IndexingServiceError` so legacy
    /// callers see exactly the same shape they used to.
    pub fn run_full_index(&mut self) -> Result<IndexingSummary, IndexingServiceError> {
        std::fs::create_dir_all(&self.indexes_dir)?;
        let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
        let metadata_db_path = self.indexes_dir.join(METADATA_DB_FILENAME);
        let store = ProjectMetadataStore::open(&metadata_db_path, Arc::clone(&clock))?;
        let project_id = metadata::project_id_from_path(&self.project_root);
        match self.build_index_resumable(&project_id, &store, clock)? {
            IndexResult::Completed { summary } | IndexResult::Resumed { summary, .. } => {
                Ok(summary)
            }
            IndexResult::Cancelled {
                batches_completed,
                files_indexed,
            } => Err(IndexingServiceError::Cancelled {
                batches_completed,
                files_indexed,
            }),
            IndexResult::AlreadyLocked { holder_pid } => {
                Err(IndexingServiceError::AlreadyLocked { holder_pid })
            }
        }
    }

    /// B-31 Phase 2: resumable, batched BM25 indexing with per-batch
    /// checkpointing into [`ProjectMetadataStore`].
    ///
    /// Walks the project tree once (ignore-aware), hashes every candidate
    /// file with BLAKE3, skips files whose stored hash matches the on-disk
    /// content (resume path), and indexes the remaining files in batches of
    /// [`DEFAULT_BATCH_SIZE`]. After each batch the file-hash rows are
    /// upserted and `project_metadata.batches_processed` /
    /// `last_checkpoint_at` are bumped.
    ///
    /// Cancellation: between batches the routine consults the in-process
    /// [`cancellation_registry`]. If set, it persists
    /// `IndexingStatus::Cancelled` and returns [`IndexResult::Cancelled`]
    /// with the counters at the time of abort.
    ///
    /// DKG: built once at the end after all BM25 batches complete (per
    /// DECISION-5 — DkgEngine has no per-file API today; a single full pass
    /// is acceptable since DKG dominates a much smaller fraction of wall time
    /// than BM25).
    pub fn build_index_resumable(
        &mut self,
        project_id: &str,
        store: &ProjectMetadataStore,
        clock: Arc<dyn ClockProvider>,
    ) -> Result<IndexResult, IndexingServiceError> {
        std::fs::create_dir_all(&self.indexes_dir)?;

        // Cancellation: a caller may have invoked `cancel_indexing` BEFORE
        // the run started (race-on-start, mirroring SocratiCode's
        // `requestCancellation` semantic at indexer.ts:134). The flag is
        // checked at every batch boundary and cleared whether the run
        // completes normally or returns `Cancelled`, so a follow-up call
        // starts clean.
        let cancel = cancellation_flag_for(project_id);

        // 1. Collect candidate files (ignore-aware walk + binary/exclusion filters).
        let candidates = collect_candidate_files(&self.project_root)?;

        // 2. Load prior file hashes (resume cursor). BTreeMap key = relative path.
        let prior_hashes = store.get_file_hashes(project_id)?;

        // 3. Partition: changed/new go to the work queue, unchanged stay.
        let mut work_queue: Vec<(PathBuf, PathBuf, blake3::Hash)> =
            Vec::with_capacity(candidates.len());
        let mut files_skipped: u64 = 0;
        let mut seen_rel_paths: HashSet<PathBuf> = HashSet::with_capacity(candidates.len());

        for abs_path in &candidates {
            let rel_path = relative_path(&self.project_root, abs_path);
            seen_rel_paths.insert(rel_path.clone());
            let current_hash = hash_file_contents(abs_path)?;
            match prior_hashes.get(&rel_path) {
                Some(stored) if stored == &current_hash => {
                    files_skipped += 1;
                }
                _ => {
                    work_queue.push((abs_path.clone(), rel_path, current_hash));
                }
            }
        }

        // Prune stale rows: any prior hash whose file no longer exists.
        for stale in prior_hashes.keys() {
            if !seen_rel_paths.contains(stale) {
                store.delete_file_hash(project_id, stale)?;
            }
        }

        // 4. Open/create the BM25 index (must outlive the batch loop).
        let mut bm25 = Bm25Index::new(&self.indexes_dir, BM25_PROJECT_ID)
            .map_err(|e| IndexingServiceError::Bm25(e.to_string()))?;

        // 5. Resume metadata: load existing row (if any) to expose `resumed_from_batch`.
        let prior_meta = store.load(project_id)?;
        let resumed_from_batch = prior_meta
            .as_ref()
            .map(|m| m.batches_processed)
            .unwrap_or(0);
        let is_resume = files_skipped > 0 || resumed_from_batch > 0;

        let batch_size = DEFAULT_BATCH_SIZE;
        let total_work = work_queue.len();
        let total_work_u64: u64 = total_work.try_into().unwrap_or(u64::MAX);
        let batches_total: u32 = if total_work == 0 {
            0
        } else {
            let bs = batch_size as usize;
            // ceil(total / batch_size) — kept in usize to avoid an extra cast.
            let n = (total_work + bs - 1) / bs;
            n.try_into().unwrap_or(u32::MAX)
        };

        // Seed `project_metadata` with the in-progress row before any work.
        let initial_meta = ProjectMetadata {
            project_id: project_id.to_string(),
            project_path: self.project_root.clone(),
            indexed_files: 0,
            total_files: total_work_u64,
            indexing_status: IndexingStatus::InProgress,
            last_checkpoint_at: clock_unix_secs(&clock)?,
            batch_size,
            batches_processed: 0,
            batches_total: Some(batches_total),
            schema_version: METADATA_SCHEMA_VERSION,
        };
        store.save(&initial_meta)?;

        // 6. Batched loop with cancellation check + per-batch checkpoint.
        let mut bm25_files: usize = 0;
        let mut bm25_chunks: usize = 0;
        let mut bm25_skipped_binary: usize = 0;
        let mut batches_processed: u32 = 0;
        let mut indexed_files_total: u64 = 0;

        for chunk in work_queue.chunks(batch_size as usize) {
            // B-31.5 (DECISION-B31.5-3): debug-mode hook for the Tier D
            // "graceful-shutdown forced-release" benchmark. Reads the env
            // var once per batch; zero / unset is the hot-path no-op.
            let delay_secs = debug_batch_delay_secs();
            if delay_secs > 0 {
                tracing::debug!(
                    target = "indexing_service",
                    delay_secs,
                    batches_processed,
                    "B-31.5: per-batch debug delay engaged",
                );
                std::thread::sleep(std::time::Duration::from_secs(delay_secs));
            }

            if cancel.load(Ordering::SeqCst) {
                let cancel_checkpoint_at = clock_unix_secs(&clock)?;
                let cancelled_meta = ProjectMetadata {
                    project_id: project_id.to_string(),
                    project_path: self.project_root.clone(),
                    indexed_files: indexed_files_total,
                    total_files: total_work_u64,
                    indexing_status: IndexingStatus::Cancelled,
                    last_checkpoint_at: cancel_checkpoint_at,
                    batch_size,
                    batches_processed,
                    batches_total: Some(batches_total),
                    schema_version: METADATA_SCHEMA_VERSION,
                };
                store.save(&cancelled_meta)?;
                // Clear so a follow-up call doesn't immediately re-trip.
                cancel.store(false, Ordering::SeqCst);

                // B-31.5: emit IndexCancel SSB row with partial-state counters.
                self.emit_ssb_payload(SsbPayload::IndexCancel(IndexCancelPayload {
                    project_id: project_id.to_string(),
                    batches_completed: batches_processed,
                    files_indexed: indexed_files_total,
                }));

                return Ok(IndexResult::Cancelled {
                    batches_completed: batches_processed,
                    files_indexed: indexed_files_total,
                });
            }

            for (abs_path, rel_path, hash) in chunk {
                if is_binary_file(abs_path) {
                    bm25_skipped_binary += 1;
                    continue;
                }
                if is_excluded_path(abs_path) {
                    continue;
                }
                match bm25.index_file(abs_path, None) {
                    Ok(n) => {
                        bm25_chunks = bm25_chunks.saturating_add(n);
                        bm25_files = bm25_files.saturating_add(1);
                        indexed_files_total = indexed_files_total.saturating_add(1);
                        store.set_file_hash(project_id, rel_path, *hash)?;
                    }
                    Err(e) => {
                        tracing::warn!(
                            path = %abs_path.display(),
                            error = %e,
                            "skipping file: BM25 index_file failed"
                        );
                    }
                }
            }

            batches_processed = batches_processed.saturating_add(1);
            let checkpoint_at = clock_unix_secs(&clock)?;
            let checkpoint_meta = ProjectMetadata {
                project_id: project_id.to_string(),
                project_path: self.project_root.clone(),
                indexed_files: indexed_files_total,
                total_files: total_work_u64,
                indexing_status: IndexingStatus::InProgress,
                last_checkpoint_at: checkpoint_at,
                batch_size,
                batches_processed,
                batches_total: Some(batches_total),
                schema_version: METADATA_SCHEMA_VERSION,
            };
            store.save(&checkpoint_meta)?;

            // B-31.5: emit IndexCheckpoint SSB row after every persisted batch.
            self.emit_ssb_payload(SsbPayload::IndexCheckpoint(IndexCheckpointPayload {
                project_id: project_id.to_string(),
                batches_processed,
                batches_total: Some(batches_total),
                files_processed: indexed_files_total,
                last_checkpoint_at: checkpoint_at,
            }));
        }

        // 7. Build DKG graph once after BM25 batches.
        let now_for_dkg = clock_unix_secs(&clock)?;
        let index_version_path = self.indexes_dir.join("index_version.toml");
        let mut dkg = DkgEngine::new(
            &self.dkg_db_path,
            Some(index_version_path.as_path()),
            now_for_dkg,
        )?;
        // B-30 Pass C: capture grammar load status before build_graph runs.
        let dkg_registry_status = dkg.registry_status();
        let graph = dkg.build_graph(&self.project_root)?;
        tracing::info!(
            files_parsed = graph.files_parsed,
            nodes = graph.node_count,
            edges = graph.edge_count,
            "DKG graph built"
        );
        drop(dkg);

        // 7b. B-29 Phase 4: build the symbol graph (sharded callee/caller
        //     payloads + name index) on top of the just-built DKG. Best-effort —
        //     a symbol-graph failure must NOT break the DKG pipeline, since the
        //     DKG is the source of truth and the symbol graph is a derived layer.
        let symbol_clock_ms = clock_unix_millis(&clock).unwrap_or(0);
        match elai_knowledge::symbol_graph::build_pipeline::build_symbol_graph(
            project_id,
            &self.project_root,
            &self.dkg_db_path,
            symbol_clock_ms,
        ) {
            Ok(summary) => tracing::info!(
                target = "symbol_graph",
                files = summary.files,
                symbols = summary.symbols,
                edges = summary.edges,
                unresolved_pct = summary.unresolved_pct,
                "symbol_graph build complete"
            ),
            Err(e) => tracing::warn!(
                target = "symbol_graph",
                error = %e,
                "symbol_graph build failed (DKG remains functional)"
            ),
        }

        // 7c. B-32: walk the indexed candidates once more and emit one
        //     SsbPayload::MinifiedFileChunked row per file that the
        //     `elai_knowledge::chunking::detect::is_minified` heuristic
        //     flags. The bm25 path already emits a `tracing::info!` on
        //     detection (defense-in-depth — see bm25.rs:412/523); the SSB
        //     row persists the same signal so the dashboard
        //     KnowledgeDrawer can surface minified files without parsing
        //     tracing output. No-op when `ssb_writer` is `None` (tests,
        //     cold-start) or when no candidate matches `is_minified`.
        //
        //     This is the "less invasive" strategy from the B-32 synthesis:
        //     a second O(n) read over the candidate set instead of
        //     threading a side-channel through Bm25Index. The cost is
        //     bounded by the candidate count and runs once per index
        //     build (not per query), so the slowdown is acceptable for
        //     the dashboard observability gain.
        self.emit_minified_chunked_rows(&candidates);

        // 8. Mark completed.
        let completed_meta = ProjectMetadata {
            project_id: project_id.to_string(),
            project_path: self.project_root.clone(),
            indexed_files: indexed_files_total,
            total_files: total_work_u64,
            indexing_status: IndexingStatus::Completed,
            last_checkpoint_at: clock_unix_secs(&clock)?,
            batch_size,
            batches_processed,
            batches_total: Some(batches_total),
            schema_version: METADATA_SCHEMA_VERSION,
        };
        store.save(&completed_meta)?;

        // Clear the cancellation flag on successful completion so a future
        // run on the same project starts clean. The Cancelled-return path
        // above also clears the flag — the persisted `IndexingStatus::Cancelled`
        // row is the durable signal; the in-memory flag is just a fast-path
        // for the active loop.
        cancel.store(false, Ordering::SeqCst);

        self.bm25 = Some(bm25);
        self.initialized = true;

        let summary = IndexingSummary {
            dkg_files_parsed: graph.files_parsed,
            dkg_nodes: graph.node_count,
            dkg_edges: graph.edge_count,
            bm25_files,
            bm25_chunks,
            bm25_skipped_binary,
            registry_status: Some(dkg_registry_status),
        };

        if is_resume {
            // B-31.5: emit IndexResume SSB row with resume counters.
            self.emit_ssb_payload(SsbPayload::IndexResume(IndexResumePayload {
                project_id: project_id.to_string(),
                resumed_from_batch,
                files_skipped,
            }));
            Ok(IndexResult::Resumed {
                summary,
                resumed_from_batch,
                files_skipped,
            })
        } else {
            Ok(IndexResult::Completed { summary })
        }
    }

    /// B-31.5: emit a single [`SsbPayload`] via the optional SSB writer.
    ///
    /// No-op when `ssb_writer` is `None` — preserves the contract that
    /// callers (cold-start CLI, contract tests) without an Ssb in scope
    /// keep working unchanged. On SSB-side failure the error is logged via
    /// `tracing::warn!` and swallowed — SSB emission is observability
    /// scaffolding, never a load-bearing pipeline stage.
    ///
    /// Uses the same async-bridge pattern as `emit_minified_chunked_rows`
    /// (cf. `ssb.rs:3653-3714`): multi-thread runtime → `block_in_place +
    /// Handle::block_on`; current-thread / no-runtime → transient
    /// current-thread runtime on a worker OS thread.
    fn emit_ssb_payload(&self, payload: SsbPayload) {
        let Some(writer) = self.ssb_writer.as_ref() else {
            tracing::debug!(
                action_type = payload.action_type().as_sql_str(),
                "B-31.5: ssb_writer absent — payload dropped",
            );
            return;
        };

        let agent_id = "indexing_service";
        let superstep: i64 = 0;
        let writer = Arc::clone(writer);
        let action_label = payload.action_type().as_sql_str();
        let files: Vec<String> = Vec::new();

        let drive = move |writer: Arc<Ssb>, payload: SsbPayload| async move {
            if let Err(e) = writer
                .write_payload(agent_id, superstep, payload, files)
                .await
            {
                tracing::warn!(
                    target = "indexing_service",
                    action_type = action_label,
                    error = %e,
                    "B-31.5: SSB write_payload failed",
                );
            }
        };

        match tokio::runtime::Handle::try_current() {
            Ok(handle) => match handle.runtime_flavor() {
                tokio::runtime::RuntimeFlavor::MultiThread => {
                    tokio::task::block_in_place(|| {
                        handle.block_on(drive(writer, payload));
                    });
                }
                _ => {
                    let join = std::thread::spawn(move || {
                        let rt = match tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                        {
                            Ok(rt) => rt,
                            Err(e) => {
                                tracing::warn!(
                                    target = "indexing_service",
                                    error = %e,
                                    "B-31.5: failed to build transient runtime for SSB emit",
                                );
                                return;
                            }
                        };
                        rt.block_on(drive(writer, payload));
                    });
                    if let Err(_panic) = join.join() {
                        tracing::warn!(
                            target = "indexing_service",
                            "B-31.5: ssb-emit worker thread panicked",
                        );
                    }
                }
            },
            Err(_) => {
                let rt = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(rt) => rt,
                    Err(e) => {
                        tracing::warn!(
                            target = "indexing_service",
                            error = %e,
                            "B-31.5: failed to build transient runtime for SSB emit",
                        );
                        return;
                    }
                };
                rt.block_on(drive(writer, payload));
            }
        }
    }

    /// B-32: post-build walkback that emits one
    /// [`crate::ssb::SsbActionType::MinifiedFileChunked`] row per minified
    /// candidate file. No-op when `ssb_writer` is `None` or when no
    /// candidate matches `chunking::detect::is_minified`.
    ///
    /// Failures are best-effort logged via `tracing::warn!` and never
    /// propagated — minified-row emission is observability scaffolding,
    /// not a load-bearing pipeline stage, so an SSB hiccup must not abort
    /// a successful index build.
    fn emit_minified_chunked_rows(&self, candidates: &[PathBuf]) {
        let Some(writer) = self.ssb_writer.as_ref() else {
            return;
        };

        // Collect (relative_path, payload) pairs first, then drain in a single
        // async-bridge block so we pay the runtime-handle lookup once per
        // build instead of once per file.
        let mut payloads: Vec<MinifiedFileChunkedPayload> = Vec::new();
        for abs_path in candidates {
            if is_binary_file(abs_path) || is_excluded_path(abs_path) {
                continue;
            }
            let content = match std::fs::read_to_string(abs_path) {
                Ok(c) => c,
                Err(e) => {
                    tracing::debug!(
                        path = %abs_path.display(),
                        error = %e,
                        "B-32: skipping minified-detection read",
                    );
                    continue;
                }
            };
            let line_count = content.lines().count();
            if !chunking::detect::is_minified(&content, line_count) {
                continue;
            }
            let rel_path = relative_path(&self.project_root, abs_path);
            let char_count: u64 = content.len().try_into().unwrap_or(u64::MAX);
            // chunk_count = ceil(char_count / MAX_CHUNK_CHARS). The actual
            // chunker may emit fewer when content tail is shorter than a
            // full window, but this matches the bm25.rs `chunks.len()` value
            // logged in the `tracing::info!` defense-in-depth emit.
            let max_chunk_chars =
                u64::try_from(chunking::constants::MAX_CHUNK_CHARS).unwrap_or(u64::MAX);
            let chunk_count_u64 = if max_chunk_chars == 0 {
                0
            } else {
                // ceil-div via saturating math
                char_count
                    .saturating_add(max_chunk_chars.saturating_sub(1))
                    / max_chunk_chars
            };
            let chunk_count: u32 = chunk_count_u64.try_into().unwrap_or(u32::MAX);
            let avg_line_length_usize = if line_count == 0 {
                content.len()
            } else {
                content.len() / line_count
            };
            let avg_line_length: u32 = avg_line_length_usize.try_into().unwrap_or(u32::MAX);

            payloads.push(MinifiedFileChunkedPayload {
                relative_path: rel_path.to_string_lossy().to_string(),
                char_count,
                chunk_count,
                avg_line_length,
            });
        }

        if payloads.is_empty() {
            return;
        }

        // Async-bridge: `Ssb::write_payload` is async, but
        // `build_index_resumable` is sync (mirrors the pattern at
        // `ssb.rs:3653-3714`). On a multi-thread runtime we use
        // `block_in_place + Handle::block_on`; on a current-thread runtime
        // (or no runtime at all) we spin a transient runtime on an OS
        // thread to drive the writes without dead-locking the caller.
        let agent_id = "indexing_service";
        let superstep: i64 = 0;
        let writer = Arc::clone(writer);

        let drive = move |writer: Arc<Ssb>, payloads: Vec<MinifiedFileChunkedPayload>| async move {
            for payload in payloads {
                let files = vec![payload.relative_path.clone()];
                if let Err(e) = writer
                    .write_payload(
                        agent_id,
                        superstep,
                        SsbPayload::MinifiedFileChunked(payload),
                        files,
                    )
                    .await
                {
                    tracing::warn!(
                        target = "knowledge::chunking",
                        error = %e,
                        "B-32: SSB write_payload(MinifiedFileChunked) failed",
                    );
                }
            }
        };

        match tokio::runtime::Handle::try_current() {
            Ok(handle) => match handle.runtime_flavor() {
                tokio::runtime::RuntimeFlavor::MultiThread => {
                    tokio::task::block_in_place(|| {
                        handle.block_on(drive(writer, payloads));
                    });
                }
                _ => {
                    // CurrentThread runtime: cannot re-enter; spin a
                    // transient current-thread runtime on a worker OS
                    // thread.
                    let join = std::thread::spawn(move || {
                        let rt = match tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                        {
                            Ok(rt) => rt,
                            Err(e) => {
                                tracing::warn!(
                                    target = "knowledge::chunking",
                                    error = %e,
                                    "B-32: failed to build transient runtime for SSB emit",
                                );
                                return;
                            }
                        };
                        rt.block_on(drive(writer, payloads));
                    });
                    if let Err(_panic) = join.join() {
                        tracing::warn!(
                            target = "knowledge::chunking",
                            "B-32: minified-emit worker thread panicked",
                        );
                    }
                }
            },
            Err(_) => {
                // No runtime at all (sync caller, e.g. CLI cold start
                // before tokio is initialized). Spin a transient
                // current-thread runtime in-line.
                let rt = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(rt) => rt,
                    Err(e) => {
                        tracing::warn!(
                            target = "knowledge::chunking",
                            error = %e,
                            "B-32: failed to build transient runtime for SSB emit",
                        );
                        return;
                    }
                };
                rt.block_on(drive(writer, payloads));
            }
        }
    }

    /// Request cancellation of an in-flight resumable indexing run for
    /// `project_id`.
    ///
    /// Returns `true` only when a prior run was actively tracked (i.e. an
    /// `Arc<AtomicBool>` already existed in the registry for `project_id`)
    /// and the cancellation flag has been set on it. Returns `false` when
    /// there is no active run; in that case we still seed a fresh flag so a
    /// run that starts moments later observes the request at its first
    /// batch boundary (mirrors the SocratiCode pre-start race semantics from
    /// `indexer.ts:134`). The boolean lets dashboard callers distinguish
    /// "cancelled an in-flight run" from "no run was running" without
    /// having to poll the registry.
    pub fn cancel_indexing(&self, project_id: &str) -> bool {
        match take_cancellation_flag(project_id) {
            Some(flag) => {
                flag.store(true, Ordering::SeqCst);
                true
            }
            None => {
                cancellation_flag_for(project_id).store(true, Ordering::SeqCst);
                false
            }
        }
    }

    /// Incrementally re-index a single changed file.
    ///
    /// Returns `Err(NotInitialized)` if `run_full_index` has not been called.
    pub fn reindex_file(&mut self, changed_path: &Path) -> Result<(), IndexingServiceError> {
        if !self.initialized {
            return Err(IndexingServiceError::NotInitialized);
        }

        // Skip binary files.
        if is_binary_file(changed_path) {
            tracing::debug!(path = %changed_path.display(), "skipping binary file in reindex");
            return Ok(());
        }

        // Skip excluded paths.
        if is_excluded_path(changed_path) {
            tracing::debug!(path = %changed_path.display(), "skipping excluded path in reindex");
            return Ok(());
        }

        // Update BM25 index for the changed file.
        if let Some(ref mut bm25) = self.bm25 {
            bm25.update_file(changed_path, None)
                .map_err(|e| IndexingServiceError::Bm25(e.to_string()))?;
        }

        Ok(())
    }

    /// Check whether indexes need rebuilding due to schema version changes.
    pub fn needs_rebuild(
        &self,
        notifier: &dyn index_version::RebuildNotifier,
    ) -> Result<UpgradeAction, IndexingServiceError> {
        if !self.indexes_dir.exists() {
            return Ok(UpgradeAction::FullRebuild);
        }
        let now = OsClock
            .system_now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
            .unwrap_or(0);
        let action = index_version::check_and_upgrade(&self.indexes_dir, now, notifier)?;
        Ok(action)
    }

    /// Take ownership of the BM25 index, consuming it from this service.
    /// Used by CLI to move BM25 into the QueryPipeline.
    pub fn take_bm25(&mut self) -> Option<Bm25Index> {
        self.bm25.take()
    }

    /// Reference to the project root.
    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    /// Reference to the indexes directory.
    pub fn indexes_dir(&self) -> &Path {
        &self.indexes_dir
    }

    /// Path to the DKG database.
    pub fn dkg_db_path(&self) -> &Path {
        &self.dkg_db_path
    }
}

// ---------------------------------------------------------------------------
// Summary
// ---------------------------------------------------------------------------

/// Summary of a full indexing run.
#[derive(Debug, Clone)]
pub struct IndexingSummary {
    pub dkg_files_parsed: usize,
    pub dkg_nodes: usize,
    pub dkg_edges: usize,
    pub bm25_files: usize,
    pub bm25_chunks: usize,
    pub bm25_skipped_binary: usize,
    /// B-30 Pass C: grammar load status captured after DkgEngine construction.
    /// None when DkgEngine construction failed before status could be read.
    pub registry_status: Option<elai_knowledge::LanguageRegistryStatus>,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Detect whether a file is binary by checking its first BINARY_DETECT_BYTES
/// bytes for null bytes.
fn is_binary_file(path: &Path) -> bool {
    let Ok(file) = std::fs::File::open(path) else {
        return false;
    };
    let mut buf = vec![0u8; BINARY_DETECT_BYTES];
    let Ok(n) = std::io::Read::read(&mut &file, &mut buf) else {
        return false;
    };
    buf[..n].contains(&0)
}

/// Check whether a path matches the default exclusion list.
fn is_excluded_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();

    // Check directory exclusions.
    for excluded in DEFAULT_EXCLUDED_DIRS {
        if path_str.contains(&format!("/{excluded}/"))
            || path_str.contains(&format!("\\{excluded}\\"))
        {
            return true;
        }
    }

    // Check file name exclusions.
    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
        for excluded in DEFAULT_EXCLUDED_FILES {
            if file_name == *excluded {
                return true;
            }
        }
        for prefix in DEFAULT_EXCLUDED_PREFIXES {
            if file_name.starts_with(prefix) {
                return true;
            }
        }
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            for excluded_ext in DEFAULT_EXCLUDED_EXTENSIONS {
                if ext == *excluded_ext {
                    return true;
                }
            }
        }
    }

    false
}

/// Convert an absolute project-tree path into a project-relative path,
/// falling back to the absolute form if `strip_prefix` fails (e.g. on
/// symlinked walks where the path is not lexically under `root`).
fn relative_path(root: &Path, abs: &Path) -> PathBuf {
    abs.strip_prefix(root)
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|_| abs.to_path_buf())
}

/// Walk the project tree honouring `.gitignore` and the default exclusion
/// rules. Returns every candidate file path in deterministic
/// (`ignore::WalkBuilder::sort_by_file_path`) order so resume runs see the
/// same batch boundaries.
fn collect_candidate_files(root: &Path) -> Result<Vec<PathBuf>, IndexingServiceError> {
    let mut out = Vec::new();
    let walker = ignore::WalkBuilder::new(root)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .hidden(false)
        .sort_by_file_path(|a, b| a.cmp(b))
        .build();

    for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                tracing::warn!(error = %err, "ignore walker entry error");
                continue;
            }
        };
        let Some(ft) = entry.file_type() else {
            continue;
        };
        if !ft.is_file() {
            continue;
        }
        let path = entry.into_path();
        if is_excluded_path(&path) {
            continue;
        }
        if is_binary_file(&path) {
            continue;
        }
        out.push(path);
    }

    Ok(out)
}

/// Stream-hash a file with BLAKE3 (`HASH_BUFFER_BYTES` per read). Avoids
/// loading large files entirely into memory.
fn hash_file_contents(path: &Path) -> Result<blake3::Hash, IndexingServiceError> {
    use std::io::Read;
    let mut hasher = blake3::Hasher::new();
    let mut file = std::fs::File::open(path)?;
    let mut buf = vec![0u8; HASH_BUFFER_BYTES];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher.finalize())
}

/// Read unix-seconds from an injected clock. Surfaces a `Metadata` error on
/// pre-epoch values so the indexing path uses the same error variant as the
/// store itself.
fn clock_unix_secs(clock: &Arc<dyn ClockProvider>) -> Result<i64, IndexingServiceError> {
    let now = clock.system_now();
    let dur = now
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| IndexingServiceError::Metadata(MetadataError::Clock(e.to_string())))?;
    i64::try_from(dur.as_secs())
        .map_err(|_| IndexingServiceError::Metadata(MetadataError::Clock("epoch overflow".into())))
}

/// Read unix-millis from an injected clock. Used by the B-29 symbol-graph
/// build path which needs millisecond resolution for the `built_at` column.
fn clock_unix_millis(clock: &Arc<dyn ClockProvider>) -> Result<i64, IndexingServiceError> {
    let now = clock.system_now();
    let dur = now
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| IndexingServiceError::Metadata(MetadataError::Clock(e.to_string())))?;
    i64::try_from(dur.as_millis())
        .map_err(|_| IndexingServiceError::Metadata(MetadataError::Clock("epoch overflow".into())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_binary_detects_null_bytes() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let mut content = vec![b'a'; 100];
        content[50] = 0;
        std::fs::write(tmp.path(), &content).unwrap();
        assert!(is_binary_file(tmp.path()));
    }

    #[test]
    fn is_binary_false_for_text() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), "fn hello() {}").unwrap();
        assert!(!is_binary_file(tmp.path()));
    }

    #[test]
    fn excluded_path_matches_dirs() {
        assert!(is_excluded_path(Path::new("/project/node_modules/foo.js")));
        assert!(is_excluded_path(Path::new("/project/target/debug/main")));
        assert!(!is_excluded_path(Path::new("/project/src/main.rs")));
    }

    #[test]
    fn excluded_path_matches_files() {
        assert!(is_excluded_path(Path::new("/project/.env")));
        assert!(is_excluded_path(Path::new("/project/secrets.toml")));
        assert!(is_excluded_path(Path::new("/project/credentials.json")));
        assert!(is_excluded_path(Path::new("/project/server.key")));
        assert!(is_excluded_path(Path::new("/project/cert.pem")));
    }
}
