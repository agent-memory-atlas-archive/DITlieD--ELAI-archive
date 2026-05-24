//! C-8 contract test — pins the wiring that
//! `IndexingService::build_index_resumable` invokes the
//! `maybe_index_artifacts` hook on the success-tail path and that the
//! hook honors C-8 Task 4's non-fatal semantics.
//!
//! Source plan: docs/thefinalglance/C-plan/C-8_viewer_stub_eradication.md
//!   (plan ADD clause for tests/indexing_service_artifact_hook_contract.rs;
//!    Task 4 — index_all_artifacts hook into build_index_resumable)
//! Decisions:   workflow/reasoning/C-8-decisions.md (DRIFT-F documents
//!   the indexed[] vs. up_to_date[] partitioning chosen over the original
//!   detect_staleness call shape)
//!
//! Sibling coverage:
//!   - indexing_service_a44_contract.rs: constructor / run_full_index /
//!     reindex_file contract — does NOT exercise the artifact hook.
//!   - c17_indexing_resilience_contract.rs: cancellation / resume — does
//!     NOT exercise the artifact hook.
//!   - c8_stub_eradication_contract.rs: ArtifactsIndexed SSB payload
//!     round-trip via the SQLite writer — does NOT exercise the
//!     IndexingService producer path.
//!
//! Tests are intentionally narrow:
//!   (1) `build_index_resumable` returns Ok on a project that defines no
//!       artifact manifest (manifest absent => empty manifest, hook is a
//!       documented no-op per indexing_service.rs:803-829).
//!   (2) `build_index_resumable` returns Ok WITHOUT bubbling Err when
//!       the `elai_cc.toml` manifest is malformed (TOML parse failure ->
//!       tracing::warn! at indexing_service.rs:815-821, hook skipped,
//!       core indexing still completes — the non-fatal semantics C-8
//!       Task 4 explicitly mandates).
//!
//! We deliberately do NOT assert the ArtifactsIndexed SSB row in these
//! tests: that round-trip is covered end-to-end in
//! c8_stub_eradication_contract.rs (5). The wiring witness here is the
//! non-fatal contract — a malformed manifest must NOT break the
//! indexing pipeline.

use std::path::Path;
use std::sync::Arc;

use elai_core::clock::SeededClock;
use elai_core::ClockProvider;
use elai_knowledge::metadata::{
    project_id_from_path, ProjectMetadataStore, METADATA_DB_FILENAME,
};
use elai_orchestrator::indexing_service::{
    IndexResult, IndexingService, ELAI_DATA_DIR_NAME, INDEXES_DIR_NAME,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn seeded_clock() -> Arc<dyn ClockProvider> {
    Arc::new(SeededClock::new())
}

/// Seed at least one indexable source file so `build_index_resumable`
/// has work to do (mirrors `seed_project_files` at
/// orchestrator/tests/indexing_resume.rs:28).
fn seed_project_files(root: &Path) {
    std::fs::create_dir_all(root.join("src")).expect("mkdir src");
    std::fs::write(
        root.join("src/lib.rs"),
        "pub fn artifact_hook_witness() {}\n",
    )
    .expect("write src/lib.rs");
}

fn open_store(indexes_dir: &Path) -> ProjectMetadataStore {
    ProjectMetadataStore::open(
        &indexes_dir.join(METADATA_DB_FILENAME),
        seeded_clock(),
    )
    .expect("ProjectMetadataStore::open")
}

// ---------------------------------------------------------------------------
// (1) Manifest absent -> hook is a no-op; build_index_resumable returns Ok.
// ---------------------------------------------------------------------------

#[test]
fn build_index_resumable_succeeds_without_artifact_manifest() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let project_root = tmp.path().to_path_buf();
    let indexes_dir = project_root.join(ELAI_DATA_DIR_NAME).join(INDEXES_DIR_NAME);
    seed_project_files(&project_root);
    std::fs::create_dir_all(&indexes_dir).expect("mkdir indexes");

    let store = open_store(&indexes_dir);
    let project_id = project_id_from_path(&project_root);

    let mut svc = IndexingService::new(project_root.clone(), indexes_dir.clone());
    let result = svc
        .build_index_resumable(&project_id, &store, seeded_clock())
        .expect("build_index_resumable must succeed when no artifact manifest is present");

    match result {
        IndexResult::Completed { summary } | IndexResult::Resumed { summary, .. } => {
            assert!(
                summary.bm25_files >= 1,
                "must index at least the seeded src/lib.rs; got {}",
                summary.bm25_files,
            );
        }
        other => panic!(
            "expected Completed/Resumed with no manifest present; got {other:?}",
        ),
    }
}

// ---------------------------------------------------------------------------
// (2) Malformed manifest -> hook returns early via tracing::warn!; the
//     core indexing pipeline still returns Ok. Pins the C-8 Task 4
//     non-fatal contract: artifact-indexing failure MUST NOT propagate.
// ---------------------------------------------------------------------------

#[test]
fn build_index_resumable_treats_malformed_artifact_manifest_as_non_fatal() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let project_root = tmp.path().to_path_buf();
    let indexes_dir = project_root.join(ELAI_DATA_DIR_NAME).join(INDEXES_DIR_NAME);
    seed_project_files(&project_root);
    std::fs::create_dir_all(&indexes_dir).expect("mkdir indexes");

    // Malformed elai_cc.toml — broken `[knowledge.artifacts]` block.
    // ArtifactManifest::load surfaces this as `ArtifactError::TomlParse`
    // per knowledge/src/artifacts/manifest.rs:41; maybe_index_artifacts
    // catches the Err at indexing_service.rs:815 and returns early via
    // tracing::warn! WITHOUT bubbling.
    std::fs::write(
        project_root.join("elai_cc.toml"),
        "this is not valid toml = =\n[knowledge.artifacts\n",
    )
    .expect("write malformed elai_cc.toml");

    let store = open_store(&indexes_dir);
    let project_id = project_id_from_path(&project_root);

    let mut svc = IndexingService::new(project_root.clone(), indexes_dir.clone());
    let result = svc.build_index_resumable(&project_id, &store, seeded_clock());

    let result = result.unwrap_or_else(|e| {
        panic!(
            "build_index_resumable must NOT bubble artifact-manifest Err out of the service \
             (C-8 Task 4 non-fatal contract); got: {e:?}",
        )
    });

    match result {
        IndexResult::Completed { summary } | IndexResult::Resumed { summary, .. } => {
            assert!(
                summary.bm25_files >= 1,
                "core indexing must complete despite malformed manifest; got {} files",
                summary.bm25_files,
            );
        }
        other => panic!(
            "expected Completed/Resumed despite malformed manifest; got {other:?}",
        ),
    }
}
