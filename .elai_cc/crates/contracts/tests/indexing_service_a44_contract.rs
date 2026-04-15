//! Contract tests for A-44 IndexingService (orchestrator).
//! Verifies the public interface contract: new, indexes_exist, run_full_index,
//! reindex_file, needs_rebuild.

use elai_knowledge::index_version::{LogNotifier, UpgradeAction};
use elai_orchestrator::indexing_service::{
    IndexingService, IndexingServiceError, INDEXES_DIR_NAME, BINARY_DETECT_BYTES,
};
use std::path::PathBuf;

fn write_file(dir: &std::path::Path, rel: &str, content: &str) -> PathBuf {
    let full = dir.join(rel);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&full, content).unwrap();
    full
}

// -----------------------------------------------------------------------
// Constants
// -----------------------------------------------------------------------

#[test]
fn indexes_dir_name_is_expected() {
    assert_eq!(
        INDEXES_DIR_NAME, "indexes",
        "INDEXES_DIR_NAME must be 'indexes'"
    );
}

#[test]
fn binary_detect_bytes_is_8192() {
    assert_eq!(
        BINARY_DETECT_BYTES, 8192,
        "BINARY_DETECT_BYTES must be 8192"
    );
}

// -----------------------------------------------------------------------
// Construction
// -----------------------------------------------------------------------

#[test]
fn new_returns_service_with_no_indexes() {
    let tmp = tempfile::TempDir::new().unwrap();
    let project_root = tmp.path().to_path_buf();
    let indexes_dir = project_root.join(INDEXES_DIR_NAME);
    let svc = IndexingService::new(project_root, indexes_dir);
    assert!(
        !svc.indexes_exist(),
        "fresh IndexingService must report no indexes"
    );
}

// -----------------------------------------------------------------------
// run_full_index
// -----------------------------------------------------------------------

#[test]
fn run_full_index_creates_dkg_and_bm25() {
    let tmp = tempfile::TempDir::new().unwrap();
    let project_root = tmp.path().to_path_buf();
    let indexes_dir = project_root.join(INDEXES_DIR_NAME);

    // Create a source file so the indexer has something to parse.
    write_file(tmp.path(), "src/lib.rs", "pub fn hello() {}\n");

    let mut svc = IndexingService::new(project_root, indexes_dir.clone());
    let result = svc.run_full_index();
    assert!(
        result.is_ok(),
        "run_full_index must succeed on a valid project: {:?}",
        result.err()
    );
    assert!(
        svc.indexes_exist(),
        "indexes_exist must return true after run_full_index"
    );
    // DKG database file should exist.
    assert!(
        indexes_dir.join("dkg.db").exists(),
        "DKG database must be created in indexes_dir"
    );
}

// -----------------------------------------------------------------------
// reindex_file
// -----------------------------------------------------------------------

#[test]
fn reindex_file_on_uninitialized_returns_error() {
    let tmp = tempfile::TempDir::new().unwrap();
    let project_root = tmp.path().to_path_buf();
    let indexes_dir = project_root.join(INDEXES_DIR_NAME);
    let mut svc = IndexingService::new(project_root, indexes_dir);
    let dummy = tmp.path().join("dummy.rs");
    std::fs::write(&dummy, "fn x() {}").unwrap();
    let result = svc.reindex_file(&dummy);
    assert!(
        result.is_err(),
        "reindex_file on uninitialized service must return Err"
    );
}

// -----------------------------------------------------------------------
// Error type
// -----------------------------------------------------------------------

#[test]
fn error_type_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<IndexingServiceError>();
}

// -----------------------------------------------------------------------
// Binary detection exclusion
// -----------------------------------------------------------------------

#[test]
fn needs_rebuild_returns_full_rebuild_when_no_indexes_dir() {
    let tmp = tempfile::TempDir::new().unwrap();
    let project_root = tmp.path().to_path_buf();
    let indexes_dir = project_root.join(INDEXES_DIR_NAME);
    // Do NOT create indexes_dir — it should not exist.
    let svc = IndexingService::new(project_root, indexes_dir);
    let action = svc.needs_rebuild(&LogNotifier).unwrap();
    assert_eq!(
        action,
        UpgradeAction::FullRebuild,
        "needs_rebuild must return FullRebuild when indexes_dir does not exist"
    );
}

#[test]
fn needs_rebuild_returns_reuse_after_fresh_build() {
    let tmp = tempfile::TempDir::new().unwrap();
    let project_root = tmp.path().to_path_buf();
    let indexes_dir = project_root.join(INDEXES_DIR_NAME);
    write_file(tmp.path(), "src/lib.rs", "pub fn hello() {}\n");
    let mut svc = IndexingService::new(project_root, indexes_dir);
    svc.run_full_index().unwrap();
    let action = svc.needs_rebuild(&LogNotifier).unwrap();
    assert_eq!(
        action,
        UpgradeAction::Reuse,
        "needs_rebuild must return Reuse immediately after run_full_index"
    );
}

#[test]
fn run_full_index_skips_binary_files() {
    let tmp = tempfile::TempDir::new().unwrap();
    let project_root = tmp.path().to_path_buf();
    let indexes_dir = project_root.join(INDEXES_DIR_NAME);

    // Create a text source file.
    write_file(tmp.path(), "src/lib.rs", "pub fn hello() {}\n");

    // Create a binary file (contains null bytes).
    let binary_path = tmp.path().join("src").join("data.bin");
    let mut binary_content = vec![0u8; 100];
    binary_content[0] = b'M';
    binary_content[1] = b'Z';
    binary_content[50] = 0; // null byte
    std::fs::write(&binary_path, &binary_content).unwrap();

    let mut svc = IndexingService::new(project_root, indexes_dir);
    let result = svc.run_full_index();
    assert!(
        result.is_ok(),
        "run_full_index must succeed even with binary files present"
    );
}
