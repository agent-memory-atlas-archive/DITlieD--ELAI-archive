//! B-1 contract: `IndexingService` writes the DKG database at exactly
//! `elai_core::paths::dkg_db_path(project_root)` — never at a hardcoded
//! literal, never at the legacy `.elai/dkg.db` location.
//!
//! Companion to `crates/contracts/tests/task_runner_indexing_service_contract.rs`,
//! which exercises end-to-end run_full_index. This file is purely about
//! the *path contract* between IndexingService and the canonical helpers.

use std::path::PathBuf;

use elai_core::paths::{dkg_db_path, ELAI_DATA_DIR_NAME, INDEXES_DIR_NAME};
use elai_orchestrator::indexing_service::IndexingService;

/// Re-export shim contract: `elai_orchestrator::indexing_service::{
/// ELAI_DATA_DIR_NAME, INDEXES_DIR_NAME}` MUST resolve to the same values
/// as the `elai_core::paths` originals. Existing contract tests in
/// `crates/contracts/tests/task_runner_indexing_service_contract.rs` import
/// from the orchestrator path; B-1 keeps that import working via re-export.
#[test]
fn orchestrator_re_exports_match_core_constants() {
    assert_eq!(
        elai_orchestrator::indexing_service::ELAI_DATA_DIR_NAME,
        ELAI_DATA_DIR_NAME,
    );
    assert_eq!(
        elai_orchestrator::indexing_service::INDEXES_DIR_NAME,
        INDEXES_DIR_NAME,
    );
}

/// `IndexingService::dkg_db_path()` accessor MUST equal the canonical
/// helper output for the same project root.
#[test]
fn indexing_service_dkg_db_path_matches_core_helper() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let project_root: PathBuf = tmp.path().to_path_buf();
    let indexes_dir = project_root.join(ELAI_DATA_DIR_NAME).join(INDEXES_DIR_NAME);

    let svc = IndexingService::new(project_root.clone(), indexes_dir);
    let observed = svc.dkg_db_path().to_path_buf();
    let expected = dkg_db_path(&project_root);

    assert_eq!(
        observed, expected,
        "IndexingService::dkg_db_path() must equal core::paths::dkg_db_path(project_root); \
         observed={observed:?}, expected={expected:?}",
    );
}

/// IndexingService must NOT write to the legacy `.elai/dkg.db` path.
#[test]
fn indexing_service_never_uses_legacy_dot_elai_path() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let project_root: PathBuf = tmp.path().to_path_buf();
    let indexes_dir = project_root.join(ELAI_DATA_DIR_NAME).join(INDEXES_DIR_NAME);

    let svc = IndexingService::new(project_root.clone(), indexes_dir);
    let observed = svc.dkg_db_path().to_path_buf();

    let legacy = project_root.join(".elai").join("dkg.db");
    assert_ne!(
        observed, legacy,
        "IndexingService must not point at the legacy .elai/dkg.db location",
    );
    let observed_str = observed.to_string_lossy();
    let legacy_segment = format!("{}{}{}", std::path::MAIN_SEPARATOR, ".elai", std::path::MAIN_SEPARATOR);
    assert!(
        !observed_str.contains(&legacy_segment) || observed_str.contains(ELAI_DATA_DIR_NAME),
        "observed dkg path '{observed_str}' must not embed the legacy .elai/ segment",
    );
}
