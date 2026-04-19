//! Gap #6 contract tests (MVP audit 2026-04-18 section 4 item 6 / A-44).
//!
//! Contracts 1-4 (original Batch 1A/1B scaffold): `IndexingService` is
//! pluggable via `TaskRunnerConfig.indexing_service` and produces a DKG +
//! BM25 layout that `DkgQuerierAdapter::new` can open.
//!
//! Contracts 5-9 (Batch 1C additions):
//!   5. `IndexingProgressPayload` and `DkgReadyPayload` are typed SSB
//!      payloads round-tripping through `rmp_serde` (msgpack) — the
//!      discriminants `IndexingProgress` / `DkgReady` exist on
//!      `SsbActionType` and map 1:1 to the payload variants.
//!   6. `ELAI_DISABLE_INDEXING` env var is observable via
//!      `indexing_disabled_via_env()` — a non-zero value short-circuits the
//!      runner's indexing call site. This is the emergency shutoff.
//!   7. `INDEX_FRESHNESS_SECS` is exported and non-zero (24h), so callers
//!      can re-run indexing after a stale marker.
//!   8. Stage labels are stable ASCII strings the frontend's
//!      `typeLabel()` substring-matches on (`start` / `dkg_parsing` /
//!      `bm25_indexing` / `complete` / `failed`).
//!   9. `TaskRunnerConfig` still accepts the pre-Batch-1C shape (field
//!      preservation for `hook_dispatcher` + `signal_reader`).

use std::path::PathBuf;
use elai_core::GitBranchPolicy;
use std::sync::Arc;

use elai_knowledge::DkgQuerierAdapter;
use elai_orchestrator::indexing_service::{
    indexing_disabled_via_env, IndexingService, ELAI_DATA_DIR_NAME, ELAI_DISABLE_INDEXING_ENV,
    INDEXES_DIR_NAME, INDEX_FRESHNESS_SECS,
};
use elai_orchestrator::ssb::{
    DkgReadyPayload, IndexingProgressPayload, SsbActionType, SsbPayload,
};
use elai_orchestrator::TaskRunnerConfig;
use tokio::sync::Mutex as AsyncMutex;

// A process-wide lock serialising tests that mutate `ELAI_DISABLE_INDEXING`.
// Cargo runs tests in parallel threads within one process, so std::env is
// shared. This mirrors the pattern used in `provider_signals_contract.rs`.
static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

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
        signal_reader: None,
        tool_dispatcher: None,
        git_branch_policy: GitBranchPolicy::default(),
        system_prober: None,
        feasibility_prober: None,
    };

    assert!(
        cfg.indexing_service.is_some(),
        "TaskRunnerConfig must accept an Arc<AsyncMutex<IndexingService>>",
    );
    // Contract 9: Batch 1A/1B fields still present alongside indexing_service.
    assert!(cfg.hook_dispatcher.is_none(), "hook_dispatcher preserved");
    assert!(cfg.signal_reader.is_none(), "signal_reader preserved");
}

// ---------------------------------------------------------------------------
// Contract 3: first run_full_index produces a DKG db that DkgQuerierAdapter
// can open.
// ---------------------------------------------------------------------------

#[test]
fn run_full_index_produces_dkg_db_openable_by_adapter() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let project_root = tmp.path().to_path_buf();
    let indexes_dir = project_root.join(ELAI_DATA_DIR_NAME).join(INDEXES_DIR_NAME);

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

    assert!(
        summary.bm25_files >= 1,
        "bm25 must have indexed at least one file; got {}",
        summary.bm25_files,
    );

    let dkg_db = indexes_dir.join("dkg.db");
    assert!(dkg_db.exists(), "dkg.db must be on disk after run_full_index");
    let adapter = DkgQuerierAdapter::new(&dkg_db);
    assert!(
        adapter.is_ok(),
        "DkgQuerierAdapter must open the DKG db: {:?}",
        adapter.err(),
    );
}

// ---------------------------------------------------------------------------
// Contract 4: indexes persist across service instances.
// ---------------------------------------------------------------------------

#[test]
fn indexes_persist_across_service_instances() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let project_root = tmp.path().to_path_buf();
    let indexes_dir = project_root.join(ELAI_DATA_DIR_NAME).join(INDEXES_DIR_NAME);

    std::fs::create_dir_all(project_root.join("src")).expect("mkdir src");
    std::fs::write(project_root.join("src").join("lib.rs"), "pub fn hi() {}\n").expect("write");

    {
        let mut svc = IndexingService::new(project_root.clone(), indexes_dir.clone());
        svc.run_full_index().expect("first run_full_index");
        assert!(svc.indexes_exist(), "indexes_exist after run_full_index");
    }

    let svc2 = IndexingService::new(project_root.clone(), indexes_dir.clone());
    assert!(
        svc2.indexes_exist(),
        "fresh service must observe on-disk indexes",
    );

    let dkg_db = indexes_dir.join("dkg.db");
    assert!(
        DkgQuerierAdapter::new(&dkg_db).is_ok(),
        "DKG db must reopen cleanly",
    );
}

// ---------------------------------------------------------------------------
// Contract 5: IndexingProgress + DkgReady msgpack round-trip through
// SsbPayload and map 1:1 to SsbActionType discriminants.
// ---------------------------------------------------------------------------

#[test]
fn indexing_progress_payload_roundtrips_msgpack() {
    let payload = IndexingProgressPayload {
        stage: "start".into(),
        project_id: "proj-a".into(),
        files_processed: 0,
        files_total: 150,
        elapsed_ms: 0,
        message: None,
    };
    let wrapped = SsbPayload::IndexingProgress(payload.clone());
    assert_eq!(wrapped.action_type(), SsbActionType::IndexingProgress);

    let bytes = rmp_serde::to_vec_named(&wrapped).expect("serialize");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("deserialize");
    match decoded {
        SsbPayload::IndexingProgress(p) => {
            assert_eq!(p.stage, "start");
            assert_eq!(p.project_id, "proj-a");
            assert_eq!(p.files_total, 150);
            assert_eq!(p.message, None);
        }
        other => panic!("expected IndexingProgress variant; got {other:?}"),
    }
}

#[test]
fn dkg_ready_payload_roundtrips_msgpack() {
    let payload = DkgReadyPayload {
        project_id: "proj-b".into(),
        node_count: 1204,
        edge_count: 3850,
        languages: vec!["rust".into(), "ts".into(), "py".into()],
    };
    let wrapped = SsbPayload::DkgReady(payload.clone());
    assert_eq!(wrapped.action_type(), SsbActionType::DkgReady);

    let bytes = rmp_serde::to_vec_named(&wrapped).expect("serialize");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("deserialize");
    match decoded {
        SsbPayload::DkgReady(p) => {
            assert_eq!(p.project_id, "proj-b");
            assert_eq!(p.node_count, 1204);
            assert_eq!(p.languages.len(), 3);
        }
        other => panic!("expected DkgReady variant; got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Contract 6: ELAI_DISABLE_INDEXING env var is observable.
// ---------------------------------------------------------------------------

#[test]
fn indexing_disabled_via_env_flag_is_observable() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let prev = std::env::var(ELAI_DISABLE_INDEXING_ENV).ok();

    std::env::set_var(ELAI_DISABLE_INDEXING_ENV, "1");
    assert!(
        indexing_disabled_via_env(),
        "ELAI_DISABLE_INDEXING=1 must disable indexing",
    );

    std::env::set_var(ELAI_DISABLE_INDEXING_ENV, "0");
    assert!(
        !indexing_disabled_via_env(),
        "ELAI_DISABLE_INDEXING=0 must NOT disable indexing",
    );

    std::env::remove_var(ELAI_DISABLE_INDEXING_ENV);
    assert!(
        !indexing_disabled_via_env(),
        "missing env var is the default enabled state",
    );

    // Restore.
    match prev {
        Some(v) => std::env::set_var(ELAI_DISABLE_INDEXING_ENV, v),
        None => std::env::remove_var(ELAI_DISABLE_INDEXING_ENV),
    }
}

// ---------------------------------------------------------------------------
// Contract 7: INDEX_FRESHNESS_SECS is exported and reasonable (24h default).
// ---------------------------------------------------------------------------

#[test]
fn index_freshness_is_24h() {
    // 24h in seconds = 86_400. The runner re-runs indexing when a marker is
    // older than this.
    assert_eq!(
        INDEX_FRESHNESS_SECS, 86_400,
        "INDEX_FRESHNESS_SECS must default to 24h (86_400 seconds)",
    );
}

// ---------------------------------------------------------------------------
// Contract 8: stage labels on IndexingProgressPayload are a stable set the
// frontend can substring-match.
// ---------------------------------------------------------------------------

#[test]
fn indexing_stage_labels_are_stable() {
    let stages = ["start", "dkg_parsing", "bm25_indexing", "complete", "failed"];
    for s in stages {
        let p = IndexingProgressPayload {
            stage: s.into(),
            project_id: "p".into(),
            files_processed: 0,
            files_total: 0,
            elapsed_ms: 0,
            message: None,
        };
        let bytes = rmp_serde::to_vec_named(&SsbPayload::IndexingProgress(p)).expect("ser");
        // Every recognised stage must round-trip without a discriminant mismatch.
        let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("de");
        assert_eq!(decoded.action_type(), SsbActionType::IndexingProgress);
    }
}
