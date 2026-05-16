//! Contract tests for B-35 Phase A: artifact pipeline isolation.
//!
//! Two contracts:
//!   code_indexing_not_blocked_by_artifact_failure —
//!     ArtifactManifest::load on a non-existent path must not panic;
//!     errors are surfaced as Result::Err, not panics.
//!
//!   context_collection_isolated_from_codebase_collection —
//!     The artifact store DB filename prefix ("context_*") differs from
//!     the codebase metadata filename ("metadata.db") and from the DKG
//!     filename ("dkg.db"). Isolation is by filename pattern at open time.

use elai_knowledge::artifacts::{ArtifactError, ArtifactManifest};
use elai_knowledge::artifacts::store::{ArtifactStore, CONTEXT_DB_FILENAME_PREFIX};
use elai_knowledge::metadata::METADATA_DB_FILENAME;

// ---------------------------------------------------------------------------

/// ArtifactManifest::load on a non-existent path does not panic.
/// When no elai_cc.toml and no .elai_artifacts.json exist, it returns
/// an empty manifest (not an error), so code indexing is unblocked.
#[test]
fn code_indexing_not_blocked_by_artifact_failure() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    // Neither elai_cc.toml nor .elai_artifacts.json present
    let result = ArtifactManifest::load(root);

    // Must not panic; must succeed with empty manifest
    let manifest = result.expect("load on missing config must return empty manifest");
    assert!(
        manifest.items.is_empty(),
        "no artifacts configured — manifest must be empty"
    );
}

/// The context_chunks DB and the metadata DB have different filename patterns.
/// This prevents schema collision between the two stores.
#[test]
fn context_collection_isolated_from_codebase_collection() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    let project_id = "test1234abcd5678";
    let context_db_name = format!("{CONTEXT_DB_FILENAME_PREFIX}{project_id}.db");
    let context_db_path = root.join(&context_db_name);

    // Open context store
    let store = ArtifactStore::open(&context_db_path).unwrap();
    drop(store);

    // Verify the context DB file was created
    assert!(context_db_path.exists(), "context DB must be created at the expected path");

    // Verify context DB name does not collide with metadata.db
    assert_ne!(
        context_db_name.as_str(),
        METADATA_DB_FILENAME,
        "context DB name must not equal metadata.db"
    );

    // Verify context DB name does not start with "dkg"
    assert!(
        !context_db_name.starts_with("dkg"),
        "context DB name must not start with 'dkg'"
    );

    // Verify context DB name starts with the correct prefix
    assert!(
        context_db_name.starts_with(CONTEXT_DB_FILENAME_PREFIX),
        "context DB name must start with '{CONTEXT_DB_FILENAME_PREFIX}'"
    );
}
