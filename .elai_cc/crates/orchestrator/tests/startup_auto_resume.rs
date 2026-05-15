//! B-31 Phase 4 (Cycle B) integration tests for
//! `orchestrator::startup::auto_resume_indexed_projects`.
//!
//! Test list (DECISION-19 in workflow/reasoning/B-31-decisions.md):
//!   1. test_auto_resume_resumes_in_progress_project
//!   2. test_auto_resume_skips_when_lock_held_elsewhere
//!   3. test_auto_resume_no_op_for_completed_project

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::{ClockProvider, OsClock};
use elai_knowledge::metadata::{
    self, IndexingStatus, ProjectMetadata, ProjectMetadataStore, DEFAULT_BATCH_SIZE,
    METADATA_DB_FILENAME, METADATA_SCHEMA_VERSION,
};
use elai_orchestrator::lock::{IndexLock, LockScope};
use elai_orchestrator::startup::{auto_resume_indexed_projects, project_id_from_path};

fn os_clock() -> Arc<dyn ClockProvider> {
    Arc::new(OsClock)
}

/// Build a fake project tree with a few text files we can index. Returns
/// (project_root, indexes_dir, project_id).
fn fake_project(name: &str) -> (PathBuf, PathBuf, String) {
    let tmp = tempfile::tempdir().expect("tempdir");
    // Leak so the temp dir survives the test body. `TempDir::keep` returns
    // the path AND prevents the destructor from removing it on drop.
    let project_root = tmp.keep().join(name);
    std::fs::create_dir_all(project_root.join("src")).expect("create project root");
    // Seed a few .rs files so the DKG engine has grammars to feed (mirrors
    // existing tests/indexing_resume.rs seed pattern).
    std::fs::write(
        project_root.join("src/a.rs"),
        "pub fn a() { println!(\"a\"); }\n",
    )
    .expect("write a");
    std::fs::write(
        project_root.join("src/b.rs"),
        "pub fn b() -> i32 { 42 }\n",
    )
    .expect("write b");
    std::fs::write(project_root.join("src/c.rs"), "pub fn c() {}\n").expect("write c");
    // `.gitignore` AND `.ignore` so the `ignore::WalkBuilder` inside
    // `build_index_resumable` does not descend into `.elai_cc/` and try
    // to hash our own lock files (Windows LockFileEx returns
    // ERROR_LOCK_VIOLATION on byte-0 reads). `.gitignore` is only honoured
    // inside git repositories; `.ignore` is honoured everywhere by the
    // `ignore` crate's WalkBuilder, so we write both for portability.
    std::fs::write(project_root.join(".gitignore"), ".elai_cc/\n")
        .expect("write .gitignore");
    std::fs::write(project_root.join(".ignore"), ".elai_cc/\n").expect("write .ignore");
    // Also init a git repo so .gitignore is honoured in case the
    // .ignore fallback is unavailable for some walker config.
    let _ = std::fs::create_dir_all(project_root.join(".git"));
    let indexes_dir = project_root
        .join(elai_core::paths::ELAI_DATA_DIR_NAME)
        .join(elai_core::paths::INDEXES_DIR_NAME);
    std::fs::create_dir_all(&indexes_dir).expect("create indexes dir");
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
        total_files: 3,
        indexing_status: IndexingStatus::InProgress,
        last_checkpoint_at: 1_700_000_000,
        batch_size: DEFAULT_BATCH_SIZE,
        batches_processed: 0,
        batches_total: Some(1),
        schema_version: METADATA_SCHEMA_VERSION,
    };
    store.save(&meta).expect("save");
}

fn seed_completed(
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
        indexed_files: 3,
        total_files: 3,
        indexing_status: IndexingStatus::Completed,
        last_checkpoint_at: 1_700_000_000,
        batch_size: DEFAULT_BATCH_SIZE,
        batches_processed: 1,
        batches_total: Some(1),
        schema_version: METADATA_SCHEMA_VERSION,
    };
    store.save(&meta).expect("save");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_auto_resume_resumes_in_progress_project() {
    let (project_root, indexes_dir, project_id) = fake_project("p_inprogress");
    // Sanity: project_id_from_path agrees with our helper.
    assert_eq!(project_id, project_id_from_path(&project_root));

    seed_in_progress(&indexes_dir, &project_root, &project_id, os_clock());

    let report = auto_resume_indexed_projects(indexes_dir.clone(), os_clock()).await;
    assert!(
        report.projects_discovered >= 1,
        "expected at least one project discovered, got {report:?}"
    );
    assert!(
        report.projects_resumed >= 1,
        "expected at least one project resumed, got {report:?}"
    );

    // Verify the project is now Completed in the DB.
    let store = ProjectMetadataStore::open(&indexes_dir.join(METADATA_DB_FILENAME), os_clock())
        .expect("reopen store");
    let meta = store.load(&project_id).expect("load").expect("present");
    assert!(
        matches!(meta.indexing_status, IndexingStatus::Completed),
        "expected Completed after resume, got {:?}",
        meta.indexing_status
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_auto_resume_skips_when_lock_held_elsewhere() {
    let (project_root, indexes_dir, project_id) = fake_project("p_locked");
    seed_in_progress(&indexes_dir, &project_root, &project_id, os_clock());

    // We synthesise an "external holder" by writing a lock file body whose
    // PID is OUR process id but acquired through the public API — that
    // makes the file genuinely locked from the kernel's perspective and
    // ensures the auto_resume's try_acquire returns AlreadyHeld.
    let guard = IndexLock::try_acquire(&project_root, LockScope::Index, os_clock())
        .expect("manual acquire");

    let report = auto_resume_indexed_projects(indexes_dir.clone(), os_clock()).await;

    // Because the lock file's PID IS our process, the auto_resume code path
    // sees it as "alive" and skips. We assert the right counter advanced.
    assert!(
        report.projects_skipped_locked >= 1,
        "expected projects_skipped_locked >= 1, got {report:?}"
    );
    // It should NOT have flipped to Completed.
    let store = ProjectMetadataStore::open(&indexes_dir.join(METADATA_DB_FILENAME), os_clock())
        .expect("reopen store");
    let meta = store.load(&project_id).expect("load").expect("present");
    assert!(
        matches!(meta.indexing_status, IndexingStatus::InProgress),
        "expected InProgress to remain when lock held, got {:?}",
        meta.indexing_status
    );

    drop(guard);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_auto_resume_no_op_for_completed_project() {
    let (project_root, indexes_dir, project_id) = fake_project("p_done");
    seed_completed(&indexes_dir, &project_root, &project_id, os_clock());

    let report = auto_resume_indexed_projects(indexes_dir.clone(), os_clock()).await;

    assert!(
        report.projects_discovered >= 1,
        "expected to discover the project, got {report:?}"
    );
    assert_eq!(
        report.projects_resumed, 0,
        "Completed project must not be resumed: {report:?}"
    );
    assert!(
        report.projects_skipped_completed >= 1,
        "expected projects_skipped_completed >= 1, got {report:?}"
    );
}
