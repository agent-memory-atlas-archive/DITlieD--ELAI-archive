//! B-31 Phase 4 (Cycle B) integration tests for
//! `orchestrator::startup::graceful_shutdown`.
//!
//! Test list (DECISION-19 in workflow/reasoning/B-31-decisions.md):
//!   1. test_graceful_shutdown_signals_active_indexer
//!   2. test_graceful_shutdown_deadline_respected
//!
//! Behaviour under test
//! --------------------
//! `graceful_shutdown` must:
//! * Set the process-global `SHUTTING_DOWN` flag.
//! * Call `IndexingService::cancel_indexing` for every InProgress project
//!   it finds under the indexes_root.
//! * Wait up to `deadline` for in-progress projects to flip to a terminal
//!   status; on deadline expiry, force-release every lock file.
//! * Return a `ShutdownReport` with accurate counters.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use elai_core::{ClockProvider, OsClock};
use elai_knowledge::metadata::{
    self, IndexingStatus, ProjectMetadata, ProjectMetadataStore, DEFAULT_BATCH_SIZE,
    METADATA_DB_FILENAME, METADATA_SCHEMA_VERSION,
};
use elai_orchestrator::startup::{graceful_shutdown, shutting_down};

fn os_clock() -> Arc<dyn ClockProvider> {
    Arc::new(OsClock)
}

fn fake_project(name: &str) -> (PathBuf, PathBuf, String) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let project_root = tmp.keep().join(name);
    std::fs::create_dir_all(project_root.join("src")).expect("create");
    // Seed one .rs file so the DKG can scan something during the inner
    // build_index_resumable call. The actual indexing is short-circuited
    // for graceful_shutdown — we just need the project_root to exist.
    std::fs::write(project_root.join("src/main.rs"), "fn main() {}\n").expect("seed");
    let indexes_dir = project_root
        .join(elai_core::paths::ELAI_DATA_DIR_NAME)
        .join(elai_core::paths::INDEXES_DIR_NAME);
    std::fs::create_dir_all(&indexes_dir).expect("create indexes");
    let project_id = metadata::project_id_from_path(&project_root);
    (project_root, indexes_dir, project_id)
}

fn seed_in_progress(
    indexes_dir: &PathBuf,
    project_root: &PathBuf,
    project_id: &str,
    clock: Arc<dyn ClockProvider>,
) {
    let db_path = indexes_dir.join(METADATA_DB_FILENAME);
    let store = ProjectMetadataStore::open(&db_path, clock).expect("open store");
    let meta = ProjectMetadata {
        project_id: project_id.to_string(),
        project_path: project_root.clone(),
        indexed_files: 0,
        total_files: 0,
        indexing_status: IndexingStatus::InProgress,
        last_checkpoint_at: 1_700_000_000,
        batch_size: DEFAULT_BATCH_SIZE,
        batches_processed: 0,
        batches_total: None,
        schema_version: METADATA_SCHEMA_VERSION,
    };
    store.save(&meta).expect("save");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_graceful_shutdown_signals_active_indexer() {
    let (project_root, indexes_dir, project_id) = fake_project("p_shutdown_active");
    seed_in_progress(&indexes_dir, &project_root, &project_id, os_clock());

    let report = graceful_shutdown(
        indexes_dir.clone(),
        Duration::from_millis(500),
        os_clock(),
    )
    .await;

    assert!(shutting_down(), "SHUTTING_DOWN flag must be set after call");
    assert!(
        report.projects_cancelled >= 1,
        "expected at least one cancellation request, got {report:?}"
    );

    // Avoid unused-binding warning on (project_root, project_id) — they are
    // captured by `seed_in_progress` so the persisted row exists, but we
    // also use them here to keep the lint clean.
    let _ = (project_root, project_id);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_graceful_shutdown_deadline_respected() {
    let (project_root, indexes_dir, project_id) = fake_project("p_shutdown_deadline");
    seed_in_progress(&indexes_dir, &project_root, &project_id, os_clock());

    // No active indexer / no AtomicBool flipping back to Completed — so the
    // wait loop is guaranteed to exhaust the deadline.
    let very_short = Duration::from_millis(150);
    let start = std::time::Instant::now();
    let report = graceful_shutdown(indexes_dir.clone(), very_short, os_clock()).await;
    let elapsed = start.elapsed();

    assert!(report.deadline_exceeded, "expected deadline_exceeded, got {report:?}");
    // The wait poll cadence is 100ms, so we should not have run wildly past
    // the deadline. Allow a generous 2-second buffer for slow CI.
    assert!(
        elapsed < Duration::from_secs(2),
        "deadline-respect wait took too long: {elapsed:?}"
    );
}
