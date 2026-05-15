//! B-31 Phase 2 tests for resumable BM25 indexing.
//!
//! Scope (per `workflow/reasoning/B-31-decisions.md` "TESTS THIS CYCLE"):
//!   1. test_first_run_marks_completed_at_end
//!   2. test_resume_skips_unchanged_files
//!   3. test_cancellation_persists_partial_state
//!   4. test_run_full_index_contract_preserved
//!
//! Each test uses its own `tempfile::tempdir()` so file_hashes / dkg.db are
//! isolated. `SeededClock` keeps timestamps deterministic for assertions.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use elai_core::ClockProvider;
use elai_core::clock::SeededClock;
use elai_knowledge::metadata::{
    project_id_from_path, IndexingStatus, ProjectMetadataStore, METADATA_DB_FILENAME,
};
use elai_orchestrator::indexing_service::{
    IndexResult, IndexingService, IndexingServiceError, ELAI_DATA_DIR_NAME, INDEXES_DIR_NAME,
};

fn seeded_clock() -> Arc<dyn ClockProvider> {
    Arc::new(SeededClock::new())
}

fn seed_project_files(root: &Path) {
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src/a.rs"), "pub fn a() { println!(\"a\"); }\n").unwrap();
    std::fs::write(root.join("src/b.rs"), "pub fn b() -> i32 { 42 }\n").unwrap();
    std::fs::write(root.join("src/c.rs"), "pub fn c() {}\n").unwrap();
}

fn open_store(indexes_dir: &Path) -> ProjectMetadataStore {
    ProjectMetadataStore::open(&indexes_dir.join(METADATA_DB_FILENAME), seeded_clock()).unwrap()
}

#[test]
fn test_first_run_marks_completed_at_end() {
    let tmp = tempfile::tempdir().unwrap();
    let project_root = tmp.path().to_path_buf();
    let indexes_dir = project_root.join(ELAI_DATA_DIR_NAME).join(INDEXES_DIR_NAME);
    seed_project_files(&project_root);

    std::fs::create_dir_all(&indexes_dir).unwrap();
    let store = open_store(&indexes_dir);
    let project_id = project_id_from_path(&project_root);
    let mut svc = IndexingService::new(project_root.clone(), indexes_dir.clone());

    let result = svc
        .build_index_resumable(&project_id, &store, seeded_clock())
        .unwrap();
    match result {
        IndexResult::Completed { summary } => {
            assert!(
                summary.bm25_files >= 1,
                "must index at least one file; got {}",
                summary.bm25_files,
            );
        }
        other => panic!("expected Completed; got {other:?}"),
    }

    let loaded = store.load(&project_id).unwrap().expect("metadata row");
    assert_eq!(loaded.indexing_status, IndexingStatus::Completed);
    assert!(loaded.batches_processed >= 1);
    assert!(loaded.indexed_files >= 1);

    let hashes = store.get_file_hashes(&project_id).unwrap();
    assert!(
        hashes.len() >= 3,
        "expected hashes for at least the 3 seeded files; got {}",
        hashes.len(),
    );
}

#[test]
fn test_resume_skips_unchanged_files() {
    let tmp = tempfile::tempdir().unwrap();
    let project_root = tmp.path().to_path_buf();
    let indexes_dir = project_root.join(ELAI_DATA_DIR_NAME).join(INDEXES_DIR_NAME);
    seed_project_files(&project_root);
    std::fs::create_dir_all(&indexes_dir).unwrap();

    let store = open_store(&indexes_dir);
    let project_id = project_id_from_path(&project_root);

    // Simulate a prior partial run by directly seeding file_hashes for a.rs and b.rs.
    // c.rs intentionally left out so the second run does work on at least one file.
    let path_a = PathBuf::from("src/a.rs");
    let path_b = PathBuf::from("src/b.rs");
    let bytes_a = std::fs::read(project_root.join(&path_a)).unwrap();
    let bytes_b = std::fs::read(project_root.join(&path_b)).unwrap();
    store
        .set_file_hash(&project_id, &path_a, blake3::hash(&bytes_a))
        .unwrap();
    store
        .set_file_hash(&project_id, &path_b, blake3::hash(&bytes_b))
        .unwrap();

    let mut svc = IndexingService::new(project_root.clone(), indexes_dir.clone());
    let result = svc
        .build_index_resumable(&project_id, &store, seeded_clock())
        .unwrap();

    match result {
        IndexResult::Resumed {
            summary,
            files_skipped,
            ..
        } => {
            assert!(
                files_skipped >= 2,
                "expected at least 2 files skipped; got {files_skipped}",
            );
            assert!(
                summary.bm25_files >= 1,
                "remaining file must still be indexed; got {}",
                summary.bm25_files,
            );
        }
        other => panic!("expected Resumed; got {other:?}"),
    }
}

#[test]
fn test_cancellation_persists_partial_state() {
    let tmp = tempfile::tempdir().unwrap();
    let project_root = tmp.path().to_path_buf();
    let indexes_dir = project_root.join(ELAI_DATA_DIR_NAME).join(INDEXES_DIR_NAME);
    seed_project_files(&project_root);
    std::fs::create_dir_all(&indexes_dir).unwrap();

    let store = open_store(&indexes_dir);
    let project_id = project_id_from_path(&project_root);
    let svc = IndexingService::new(project_root.clone(), indexes_dir.clone());

    // Set the cancellation flag BEFORE starting — the loop checks it at the
    // top of the first batch, so the run aborts before any BM25 writes. We
    // expect `false` because no run was actively tracked yet (the registry
    // had no prior flag); the flag is still seeded for the imminent run to
    // observe (mirrors SocratiCode's pre-start race semantics).
    assert!(!svc.cancel_indexing(&project_id));

    let mut svc = svc;
    let result = svc
        .build_index_resumable(&project_id, &store, seeded_clock())
        .unwrap();

    match result {
        IndexResult::Cancelled {
            batches_completed,
            files_indexed,
        } => {
            assert_eq!(batches_completed, 0, "no batches before cancel");
            assert_eq!(files_indexed, 0, "no files before cancel");
        }
        other => panic!("expected Cancelled; got {other:?}"),
    }

    // Persisted state: status = Cancelled.
    let loaded = store.load(&project_id).unwrap().expect("metadata row");
    assert_eq!(loaded.indexing_status, IndexingStatus::Cancelled);
}

#[test]
fn test_run_full_index_contract_preserved() {
    // Contract: run_full_index returns IndexingSummary unchanged shape.
    let tmp = tempfile::tempdir().unwrap();
    let project_root = tmp.path().to_path_buf();
    let indexes_dir = project_root.join(ELAI_DATA_DIR_NAME).join(INDEXES_DIR_NAME);
    seed_project_files(&project_root);

    let mut svc = IndexingService::new(project_root.clone(), indexes_dir.clone());
    let summary = svc.run_full_index().unwrap();
    assert!(summary.bm25_files >= 1, "files_indexed >= 1");
    // Field access compiles -> field set unchanged.
    let _ = summary.dkg_files_parsed;
    let _ = summary.dkg_nodes;
    let _ = summary.dkg_edges;
    let _ = summary.bm25_chunks;
    let _ = summary.bm25_skipped_binary;

    // The error variants exist on IndexingServiceError too (compile-time check).
    fn _assert_error_variants(e: IndexingServiceError) {
        match e {
            IndexingServiceError::Cancelled { .. } => {}
            IndexingServiceError::AlreadyLocked { .. } => {}
            IndexingServiceError::Metadata(_) => {}
            _ => {}
        }
    }
}
