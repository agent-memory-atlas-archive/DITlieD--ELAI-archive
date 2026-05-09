//! B-20A Task 8 — integration_checkpoint_success Finding emission tests.
//!
//! Decision E: emit a Finding row with category string
//! `INTEGRATION_CHECKPOINT_SUCCESS_CATEGORY` (mirrors the existing
//! `integration_checkpoint_failed` pattern). NO new SsbActionType variant,
//! NO SCHEMA_VERSION bump (stays at 24). Option (b) — proper variant —
//! deferred to B-future.
//!
//! Tests:
//!   1. Successful integration emits a Finding with the new SUCCESS category.
//!   2. Failing integration still emits a Finding with the existing FAILED
//!      category (regression guard).

use std::sync::Arc;

use elai_core::clock::{ClockProvider, SeededClock};
use elai_orchestrator::pipeline_tree_navigator::{DecompositionLevel, IntegrationRunResult};
use elai_orchestrator::ssb::{
    FindingPayload, Ssb, SsbActionType, SsbConfig, SsbPayload, SsbReader,
};
use elai_orchestrator::task_runner::{
    emit_integration_checkpoint_for_test,
    INTEGRATION_CHECKPOINT_FAILED_CATEGORY, INTEGRATION_CHECKPOINT_SUCCESS_CATEGORY,
};
use tempfile::TempDir;

fn make_ssb(dir: &TempDir, name: &str) -> Ssb {
    let cfg = SsbConfig {
        db_path: dir.path().join(name),
        pool_size: Some(2),
        project_id: "b20a-proj".into(),
        session_id: "b20a-sess".into(),
    };
    let clock: Arc<dyn ClockProvider> = Arc::new(SeededClock::new());
    Ssb::new(cfg, clock).expect("ssb init")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn success_emits_finding_with_success_category() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = Arc::new(make_ssb(&dir, "succ.db"));

    let run = IntegrationRunResult {
        level: DecompositionLevel::Phase,
        node_id: 42,
        success: true,
        criterion_results: Vec::new(),
        failure_reason: None,
        total_duration_ms: 123,
    };
    emit_integration_checkpoint_for_test(&ssb, DecompositionLevel::Phase, 42, &run).await;

    let rows = ssb.query_by_type(SsbActionType::Finding).expect("query");
    let mut success_seen = false;
    let mut failure_seen = false;
    for entry in rows {
        let payload: SsbPayload = rmp_serde::from_slice(&entry.payload).expect("decode");
        if let SsbPayload::Finding(FindingPayload { category, .. }) = payload {
            if category == INTEGRATION_CHECKPOINT_SUCCESS_CATEGORY {
                success_seen = true;
            }
            if category == INTEGRATION_CHECKPOINT_FAILED_CATEGORY {
                failure_seen = true;
            }
        }
    }
    assert!(success_seen, "must emit a success Finding row");
    assert!(!failure_seen, "must NOT emit a failure Finding on success path");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn failure_still_emits_failed_category_regression() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = Arc::new(make_ssb(&dir, "fail.db"));

    let run = IntegrationRunResult {
        level: DecompositionLevel::Step,
        node_id: 7,
        success: false,
        criterion_results: Vec::new(),
        failure_reason: Some("integration tier blew up".into()),
        total_duration_ms: 99,
    };
    emit_integration_checkpoint_for_test(&ssb, DecompositionLevel::Step, 7, &run).await;

    let rows = ssb.query_by_type(SsbActionType::Finding).expect("query");
    let mut success_seen = false;
    let mut failure_seen = false;
    for entry in rows {
        let payload: SsbPayload = rmp_serde::from_slice(&entry.payload).expect("decode");
        if let SsbPayload::Finding(FindingPayload { category, .. }) = payload {
            if category == INTEGRATION_CHECKPOINT_SUCCESS_CATEGORY {
                success_seen = true;
            }
            if category == INTEGRATION_CHECKPOINT_FAILED_CATEGORY {
                failure_seen = true;
            }
        }
    }
    assert!(failure_seen, "regression: failure path must still emit failed Finding");
    assert!(!success_seen, "must NOT emit success Finding on failure path");
}
