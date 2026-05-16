//! B-35 Phase A — staleness detection tests.
//!
//! 5 tests:
//!   first_index_all_new — no prev_state → all files appear as added.
//!   no_op_reindex_all_unchanged — same hashes → is_clean().
//!   modify_one_file_only_that — changing one file → only that appears in changed.
//!   add_file_in_dir_only_new — new file in dir → appears in added.
//!   remove_file_drops_chunks — file removed from disk → appears in removed.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use elai_knowledge::artifacts::staleness::detect_staleness;
use elai_knowledge::artifacts::types::{ArtifactIndexState, ArtifactSpec, ArtifactStatus};

fn make_spec(name: &str, path: &Path, is_dir: bool) -> ArtifactSpec {
    ArtifactSpec {
        name: name.to_string(),
        path: path.to_path_buf(),
        description: "test artifact".to_string(),
        is_dir,
    }
}

fn make_state(name: &str, file_hashes: BTreeMap<String, String>) -> ArtifactIndexState {
    ArtifactIndexState {
        name: name.to_string(),
        last_indexed_at: 0,
        file_hashes,
        chunk_count: 1,
        status: ArtifactStatus::Indexed,
    }
}

// ---------------------------------------------------------------------------

#[test]
fn first_index_all_new() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    fs::write(root.join("schema.sql"), "CREATE TABLE t (id INT);").unwrap();

    let spec = make_spec("schema", &root.join("schema.sql"), false);
    let result = detect_staleness(&spec, None).unwrap();

    assert!(result.unchanged.is_empty(), "no unchanged on first index");
    assert!(result.changed.is_empty(), "no changed on first index");
    assert_eq!(result.added.len(), 1, "single file should appear as added");
    assert!(result.removed.is_empty());
}

#[test]
fn no_op_reindex_all_unchanged() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    let content = "CREATE TABLE t (id INT);";
    fs::write(root.join("schema.sql"), content).unwrap();

    let spec = make_spec("schema", &root.join("schema.sql"), false);

    // Hash the file the same way the indexer does
    let hash = blake3::hash(content.as_bytes()).to_hex().to_string();
    let mut hashes = BTreeMap::new();
    hashes.insert("schema.sql".to_string(), hash);

    let state = make_state("schema", hashes);
    let result = detect_staleness(&spec, Some(&state)).unwrap();

    assert!(result.is_clean(), "no changes expected");
    assert_eq!(result.unchanged.len(), 1);
}

#[test]
fn modify_one_file_only_that() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let dir = root.join("k8s");
    fs::create_dir_all(&dir).unwrap();

    fs::write(dir.join("deploy.yaml"), "apiVersion: apps/v1").unwrap();
    fs::write(dir.join("service.yaml"), "kind: Service").unwrap();

    let spec = make_spec("k8s", &dir, true);

    let svc_content = "kind: Service";
    let svc_hash = blake3::hash(svc_content.as_bytes()).to_hex().to_string();
    // Use a stale hash for deploy.yaml (different from current)
    let mut hashes = BTreeMap::new();
    hashes.insert("deploy.yaml".to_string(), "000000stale000000".to_string());
    hashes.insert("service.yaml".to_string(), svc_hash);

    let state = make_state("k8s", hashes);
    let result = detect_staleness(&spec, Some(&state)).unwrap();

    assert_eq!(result.changed.len(), 1, "only deploy.yaml changed");
    assert_eq!(result.unchanged.len(), 1, "service.yaml unchanged");
    assert!(result.added.is_empty());
    assert!(result.removed.is_empty());
}

#[test]
fn add_file_in_dir_only_new() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let dir = root.join("k8s");
    fs::create_dir_all(&dir).unwrap();

    let existing_content = "kind: Service";
    fs::write(dir.join("service.yaml"), existing_content).unwrap();
    // New file not in prev_state
    fs::write(dir.join("ingress.yaml"), "kind: Ingress").unwrap();

    let spec = make_spec("k8s", &dir, true);

    let svc_hash = blake3::hash(existing_content.as_bytes()).to_hex().to_string();
    let mut hashes = BTreeMap::new();
    hashes.insert("service.yaml".to_string(), svc_hash);
    // ingress.yaml absent from prev_state

    let state = make_state("k8s", hashes);
    let result = detect_staleness(&spec, Some(&state)).unwrap();

    assert_eq!(result.added.len(), 1, "ingress.yaml should be added");
    assert_eq!(result.unchanged.len(), 1, "service.yaml unchanged");
    assert!(result.changed.is_empty());
    assert!(result.removed.is_empty());
}

#[test]
fn remove_file_drops_chunks() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let dir = root.join("k8s");
    fs::create_dir_all(&dir).unwrap();

    fs::write(dir.join("service.yaml"), "kind: Service").unwrap();
    // deploy.yaml present in prev_state but NOT on disk

    let spec = make_spec("k8s", &dir, true);

    let mut hashes = BTreeMap::new();
    hashes.insert("deploy.yaml".to_string(), "abc123".to_string());
    hashes.insert("service.yaml".to_string(), "xyz".to_string());

    let state = make_state("k8s", hashes);
    let result = detect_staleness(&spec, Some(&state)).unwrap();

    assert_eq!(result.removed.len(), 1, "deploy.yaml removed");
    // service.yaml may be unchanged or changed depending on actual hash
    assert!(result.added.is_empty());
}
