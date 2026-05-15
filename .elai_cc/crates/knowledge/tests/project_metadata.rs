//! B-31 Phase 1 tests for `elai_knowledge::metadata::ProjectMetadataStore`.
//!
//! Scope (per `workflow/reasoning/B-31-decisions.md` "TESTS THIS CYCLE"):
//!   1. test_create_tables_idempotent
//!   2. test_save_load_roundtrip
//!   3. test_update_status_writes
//!   4. test_file_hashes_crud
//!   5. test_delete_project_cascades
//!   6. test_schema_version_seeded
//!   7. test_status_check_constraint_rejects_invalid
//!
//! Test pattern follows `gotchas.md:897-900` — every test gets a fresh
//! `Connection::open_in_memory()` OR a tempdir-backed file path; we never
//! share a connection across tests.

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::ClockProvider;
use elai_core::clock::SeededClock;
use elai_knowledge::metadata::{
    project_id_from_path, IndexingStatus, ProjectMetadata, ProjectMetadataStore,
    DEFAULT_BATCH_SIZE, METADATA_SCHEMA_VERSION,
};
use rusqlite::Connection;
use tempfile::tempdir;

fn seeded_clock() -> Arc<dyn ClockProvider> {
    Arc::new(SeededClock::new())
}

fn make_meta(pid: &str) -> ProjectMetadata {
    ProjectMetadata {
        project_id: pid.to_string(),
        project_path: PathBuf::from("/tmp/proj"),
        indexed_files: 0,
        total_files: 0,
        indexing_status: IndexingStatus::InProgress,
        last_checkpoint_at: 1_700_000_000,
        batch_size: DEFAULT_BATCH_SIZE,
        batches_processed: 0,
        batches_total: None,
        schema_version: METADATA_SCHEMA_VERSION,
    }
}

#[test]
fn test_create_tables_idempotent() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("metadata.db");
    // Open twice — second open MUST NOT error (CREATE TABLE IF NOT EXISTS).
    let s1 = ProjectMetadataStore::open(&db_path, seeded_clock()).unwrap();
    drop(s1);
    let _s2 = ProjectMetadataStore::open(&db_path, seeded_clock()).unwrap();
}

#[test]
fn test_save_load_roundtrip() {
    let dir = tempdir().unwrap();
    let store = ProjectMetadataStore::open(&dir.path().join("metadata.db"), seeded_clock()).unwrap();

    let pid = project_id_from_path(dir.path());
    let mut meta = make_meta(&pid);
    meta.indexed_files = 42;
    meta.total_files = 100;
    meta.batches_processed = 3;
    meta.batches_total = Some(2);

    store.save(&meta).unwrap();
    let loaded = store.load(&pid).unwrap().expect("row must be present");
    assert_eq!(loaded, meta);

    // Idempotent upsert — second save changes a field, load reflects update.
    meta.indexed_files = 99;
    store.save(&meta).unwrap();
    let loaded2 = store.load(&pid).unwrap().expect("row still present");
    assert_eq!(loaded2.indexed_files, 99);
}

#[test]
fn test_update_status_writes() {
    let dir = tempdir().unwrap();
    let store = ProjectMetadataStore::open(&dir.path().join("metadata.db"), seeded_clock()).unwrap();

    let pid = "pid-status";
    let meta = make_meta(pid);
    store.save(&meta).unwrap();

    for status in [
        IndexingStatus::InProgress,
        IndexingStatus::Cancelled,
        IndexingStatus::Completed,
    ] {
        let rows = store.update_status(pid, status).unwrap();
        assert_eq!(rows, 1, "update_status must touch exactly 1 row");
        let loaded = store.load(pid).unwrap().unwrap();
        assert_eq!(loaded.indexing_status, status);
    }

    // Update on a non-existent project returns 0.
    let rows = store.update_status("does-not-exist", IndexingStatus::Completed).unwrap();
    assert_eq!(rows, 0);
}

#[test]
fn test_file_hashes_crud() {
    let dir = tempdir().unwrap();
    let store = ProjectMetadataStore::open(&dir.path().join("metadata.db"), seeded_clock()).unwrap();

    let pid = "pid-files";
    let p1 = PathBuf::from("src/a.rs");
    let p2 = PathBuf::from("src/b.rs");
    let h1 = blake3::hash(b"alpha");
    let h2 = blake3::hash(b"beta");

    // Empty list before any writes.
    assert!(store.get_file_hashes(pid).unwrap().is_empty());

    // Set two rows.
    store.set_file_hash(pid, &p1, h1).unwrap();
    store.set_file_hash(pid, &p2, h2).unwrap();
    let got = store.get_file_hashes(pid).unwrap();
    assert_eq!(got.len(), 2);
    assert_eq!(got.get(&p1), Some(&h1));
    assert_eq!(got.get(&p2), Some(&h2));

    // Idempotent upsert: changing the hash for p1 must overwrite, not duplicate.
    let h1b = blake3::hash(b"alpha-v2");
    store.set_file_hash(pid, &p1, h1b).unwrap();
    let got = store.get_file_hashes(pid).unwrap();
    assert_eq!(got.len(), 2);
    assert_eq!(got.get(&p1), Some(&h1b));

    // Delete one row.
    let rows = store.delete_file_hash(pid, &p1).unwrap();
    assert_eq!(rows, 1);
    let got = store.get_file_hashes(pid).unwrap();
    assert_eq!(got.len(), 1);
    assert!(got.contains_key(&p2));

    // Delete same row again -> 0 rows.
    let rows = store.delete_file_hash(pid, &p1).unwrap();
    assert_eq!(rows, 0);
}

#[test]
fn test_delete_project_cascades() {
    let dir = tempdir().unwrap();
    let store = ProjectMetadataStore::open(&dir.path().join("metadata.db"), seeded_clock()).unwrap();

    let pid_a = "pid-A";
    let pid_b = "pid-B";

    store.save(&make_meta(pid_a)).unwrap();
    store.save(&make_meta(pid_b)).unwrap();
    store.set_file_hash(pid_a, &PathBuf::from("a.rs"), blake3::hash(b"a")).unwrap();
    store.set_file_hash(pid_a, &PathBuf::from("b.rs"), blake3::hash(b"b")).unwrap();
    store.set_file_hash(pid_b, &PathBuf::from("c.rs"), blake3::hash(b"c")).unwrap();

    // Delete pid_a — its hashes go too, pid_b survives.
    store.delete_project(pid_a).unwrap();
    assert!(store.load(pid_a).unwrap().is_none());
    assert!(store.get_file_hashes(pid_a).unwrap().is_empty());
    assert!(store.load(pid_b).unwrap().is_some());
    assert_eq!(store.get_file_hashes(pid_b).unwrap().len(), 1);

    // list_project_ids returns only pid_b.
    let ids = store.list_project_ids().unwrap();
    assert_eq!(ids, vec![pid_b.to_string()]);
}

#[test]
fn test_schema_version_seeded() {
    let dir = tempdir().unwrap();
    let store = ProjectMetadataStore::open(&dir.path().join("metadata.db"), seeded_clock()).unwrap();

    let pid = "pid-version";
    let mut meta = make_meta(pid);
    // Force a stale schema_version on the in-memory side; save() must
    // overwrite with the current constant.
    meta.schema_version = 0;
    store.save(&meta).unwrap();

    let loaded = store.load(pid).unwrap().unwrap();
    assert_eq!(loaded.schema_version, METADATA_SCHEMA_VERSION);
}

#[test]
fn test_status_check_constraint_rejects_invalid() {
    let conn = Connection::open_in_memory().unwrap();
    let sql = include_str!("../src/metadata_migration_v2.sql");
    conn.execute_batch(sql).unwrap();
    // Direct insert with a bogus status must fail (CHECK constraint).
    let err = conn.execute(
        "INSERT INTO project_metadata (\
            project_id, project_path, indexed_files, total_files, \
            indexing_status, last_checkpoint_at, batch_size, \
            batches_processed, batches_total, schema_version\
         ) VALUES ('p', '/tmp', 0, 0, 'bogus-status', 0, 50, 0, NULL, 1)",
        [],
    );
    assert!(err.is_err(), "CHECK constraint must reject unknown status");
}
