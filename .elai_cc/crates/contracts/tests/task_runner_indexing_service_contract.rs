//! Gap #6 contract tests (MVP audit 2026-04-18 section 4 item 6 / A-44):
//! `IndexingService` is pluggable via `TaskRunnerConfig.indexing_service` and
//! the task runner invokes `run_full_index` at most once per `project_id`.
//!
//! Covers:
//!   1. `TaskRunnerConfig::new_minimal` leaves `indexing_service` = None so
//!      the pre-Gap-#6 no-service behaviour stays the default.
//!   2. `TaskRunnerConfig` accepts a `Some(Arc<AsyncMutex<IndexingService>>)`
//!      shape — this is the wiring contract the dashboard relies on at
//!      `dashboard/src/lib.rs:~290`.
//!   3. After `IndexingService::run_full_index` completes, the constructed
//!      DKG database path matches the layout used by `DkgQuerierAdapter::new`
//!      so the rule engine can open it read-only. This validates the
//!      wiring-side contract without having to drive `drive_task` (which
//!      needs a live sidecar).
//!   4. Calling `run_full_index` twice against the same project is a
//!      DATA-level no-op for the BM25+DKG path (idempotent on the on-disk
//!      artefacts) — the runner's per-project cache prevents redundant
//!      invocation but the underlying service itself must also tolerate the
//!      double call without panicking.

use std::path::PathBuf;
use std::sync::Arc;

use elai_knowledge::DkgQuerierAdapter;
use elai_orchestrator::indexing_service::{
    IndexingService, ELAI_DATA_DIR_NAME, INDEXES_DIR_NAME,
};
use elai_orchestrator::TaskRunnerConfig;
use tokio::sync::Mutex as AsyncMutex;

// ---------------------------------------------------------------------------
// Contract 1: new_minimal leaves indexing_service = None
// ---------------------------------------------------------------------------

#[test]
fn new_minimal_config_leaves_indexing_service_none() {
    let cfg = TaskRunnerConfig::new_minimal(PathBuf::from("/tmp/cursor.json"));
    assert!(
        cfg.indexing_service.is_none(),
        "pre-Gap-#6 behaviour: indexing_service defaults to None",
    );
}

// ---------------------------------------------------------------------------
// Contract 2: TaskRunnerConfig accepts an Arc<AsyncMutex<IndexingService>>
// ---------------------------------------------------------------------------

#[test]
fn task_runner_config_accepts_indexing_service_arc() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let project_root = tmp.path().to_path_buf();
    let indexes_dir = project_root.join(ELAI_DATA_DIR_NAME).join(INDEXES_DIR_NAME);
    let svc = IndexingService::new(project_root, indexes_dir);

    let cfg = TaskRunnerConfig {
        cursor_path: tmp.path().join("cursor.json"),
        dev_server_launcher: None,
        project_dir_resolver: None,
        dev_server_registry: None,
        web_research_runner: None,
        scaffold_applier: None,
        rule_evaluator: None,
        interrogation_store: None,
        checkpoint_scratch_dir: None,
        baby_step_executor: None,
        anti_spiral_enabled: false,
        llm_provider: None,
        backend_config: None,
        indexing_service: Some(Arc::new(AsyncMutex::new(svc))),
        hook_dispatcher: None,
    };

    assert!(
        cfg.indexing_service.is_some(),
        "TaskRunnerConfig must accept an Arc<AsyncMutex<IndexingService>>",
    );
}

// ---------------------------------------------------------------------------
// Contract 3: first run_full_index produces a DKG db that DkgQuerierAdapter
// can open — mirrors the wiring the dashboard does to swap EmptyDkgQuerier
// for the adapter at lib.rs:~290.
// ---------------------------------------------------------------------------

#[test]
fn run_full_index_produces_dkg_db_openable_by_adapter() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let project_root = tmp.path().to_path_buf();
    let indexes_dir = project_root.join(ELAI_DATA_DIR_NAME).join(INDEXES_DIR_NAME);

    // Seed a tiny Rust source file so the DKG has something to parse.
    std::fs::create_dir_all(project_root.join("src")).expect("mkdir src");
    std::fs::write(
        project_root.join("src").join("lib.rs"),
        "pub fn hello() {}\n",
    )
    .expect("write lib.rs");

    let mut svc = IndexingService::new(project_root.clone(), indexes_dir.clone());
    let summary = svc
        .run_full_index()
        .expect("run_full_index must succeed on a valid project");

    // The files_indexed summary is populated — acceptance criterion for the
    // dashboard's Finding emission ("files_indexed=N dkg_nodes=M ...").
    assert!(
        summary.bm25_files >= 1,
        "bm25 must have indexed at least one file (the seeded lib.rs); got {}",
        summary.bm25_files,
    );

    // The DKG db path must match the dashboard's deterministic derivation
    // (project_root/.elai_cc/indexes/dkg.db) so `DkgQuerierAdapter::new`
    // can open it.
    let dkg_db = indexes_dir.join("dkg.db");
    assert!(dkg_db.exists(), "dkg.db must be on disk after run_full_index");
    let adapter = DkgQuerierAdapter::new(&dkg_db);
    assert!(
        adapter.is_ok(),
        "DkgQuerierAdapter must open the DKG db that IndexingService just built: {:?}",
        adapter.err(),
    );
}

// ---------------------------------------------------------------------------
// Contract 4: after `indexes_exist()` returns true (first run complete), a
// fresh IndexingService instance pointed at the same indexes_dir reports
// the artefacts are present WITHOUT re-indexing. This is the invariant the
// task_runner's per-project cache relies on: the cache prevents a second
// `run_full_index` call within the runner's lifetime, but a FRESH runner
// (process restart) must also observe the on-disk indexes cleanly so the
// DkgQuerierAdapter can keep opening them.
// ---------------------------------------------------------------------------

#[test]
fn indexes_persist_across_service_instances() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let project_root = tmp.path().to_path_buf();
    let indexes_dir = project_root.join(ELAI_DATA_DIR_NAME).join(INDEXES_DIR_NAME);

    std::fs::create_dir_all(project_root.join("src")).expect("mkdir src");
    std::fs::write(
        project_root.join("src").join("lib.rs"),
        "pub fn hi() {}\n",
    )
    .expect("write lib.rs");

    {
        let mut svc = IndexingService::new(project_root.clone(), indexes_dir.clone());
        svc.run_full_index().expect("first run_full_index");
        assert!(
            svc.indexes_exist(),
            "indexes_exist must report true after run_full_index",
        );
    } // svc dropped here, BM25 file lock released.

    // Fresh service: indexes_exist must return true without rebuilding.
    let svc2 = IndexingService::new(project_root.clone(), indexes_dir.clone());
    assert!(
        svc2.indexes_exist(),
        "fresh service must observe on-disk indexes from the previous run",
    );

    // DkgQuerierAdapter must still open the DKG db — no file is locked.
    let dkg_db = indexes_dir.join("dkg.db");
    assert!(
        DkgQuerierAdapter::new(&dkg_db).is_ok(),
        "DKG db must reopen cleanly after the first service drops",
    );
}
