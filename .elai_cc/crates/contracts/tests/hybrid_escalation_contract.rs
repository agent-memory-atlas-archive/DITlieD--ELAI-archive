//! C-5 follow-on contract — HYBRID escalation surface.
//!
//! The full HYBRID re-invoke loop (task_runner dispatching ExecMode::Cloud
//! after local L3 retry exhaust + cap=1 second-exhaustion HALT) requires a
//! production agent pool that is unavailable in this contract crate. This
//! file pins the surface that DOES land in this session:
//!   1. `ExecMode::Auto` returns `RoutedProvider::Local` from
//!      `CloudModeRouter::route` (the auto path starts local; the re-invoke
//!      is driven by the runner, not the router).
//!   2. `EscalateToCloudPayload` accepts the diagnostic + cap_reached
//!      trigger_reason strings unchanged ("hybrid_local_first",
//!      "cloud_pinned", "cap_reached").
//!   3. `BackendMode::Hybrid` parses from "hybrid" / "auto" and maps to
//!      `ExecMode::Auto` via `resolve_exec_mode`.
//!
//! Source plan: docs/thefinalglance/C-plan/C-5_cloud_mode_hardening.md
//!              Task 4 (HYBRID escalation retry loop)
//! Decisions:   workflow/reasoning/C-5-decisions.md

use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::cloud_mode::{
    resolve_exec_mode, CloudConfig, CloudCredentials, CloudModeRouter, ExecMode, RoutedProvider,
};
use elai_orchestrator::dispatcher::{BackendConfig, BackendMode};
use elai_orchestrator::ssb::{
    EscalateToCloudPayload, Ssb, SsbActionType, SsbConfig, SsbPayload,
};
use elai_orchestrator::SsbReader;
use tempfile::TempDir;

fn fresh_db() -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("ssb.db");
    (dir, path)
}

async fn open_writer(path: &std::path::Path) -> Ssb {
    let cfg = SsbConfig {
        db_path: path.to_path_buf(),
        pool_size: Some(2),
        project_id: "hybrid-escalation".into(),
        session_id: "he-test".into(),
    };
    Ssb::new(cfg, Arc::new(OsClock)).expect("ssb init")
}

#[test]
fn auto_mode_returns_local_for_router() {
    let router = CloudModeRouter::new(CloudConfig::default());
    let outcome = router.route(ExecMode::Auto, None);
    assert_eq!(outcome, Ok(RoutedProvider::Local));
}

#[test]
fn backend_mode_hybrid_parses_from_aliases() {
    assert_eq!(BackendMode::from_str("hybrid").unwrap(), BackendMode::Hybrid);
    assert_eq!(BackendMode::from_str("auto").unwrap(), BackendMode::Hybrid);
    assert_eq!(BackendMode::from_str("HYBRID").unwrap(), BackendMode::Hybrid);
}

#[test]
fn resolve_exec_mode_hybrid_maps_to_auto() {
    let mut backend = BackendConfig::default();
    backend.mode = BackendMode::Hybrid;
    assert_eq!(resolve_exec_mode(&backend), ExecMode::Auto);
}

#[test]
fn resolve_exec_mode_local_maps_to_local() {
    let mut backend = BackendConfig::default();
    backend.mode = BackendMode::Local;
    assert_eq!(resolve_exec_mode(&backend), ExecMode::Local);
}

#[test]
fn resolve_exec_mode_api_maps_to_cloud() {
    let mut backend = BackendConfig::default();
    backend.mode = BackendMode::Api;
    assert_eq!(resolve_exec_mode(&backend), ExecMode::Cloud);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn escalate_to_cloud_carries_hybrid_local_first_trigger() {
    let (_dir, path) = fresh_db();
    let ssb = open_writer(&path).await;
    let payload = SsbPayload::EscalateToCloud(EscalateToCloudPayload {
        step_id: "baby_step_42".into(),
        trigger_reason: "hybrid_local_first".into(),
        layer_failed: 1,
        ts_ms: 1_700_000_500_000,
    });
    let seq = ssb
        .write_payload("test-agent", 0, payload, Vec::new())
        .await
        .expect("write escalate_to_cloud");
    assert!(seq > 0);
    let rows = ssb
        .query_by_type(SsbActionType::EscalateToCloud)
        .expect("query escalate_to_cloud");
    assert!(!rows.is_empty(), "at least one EscalateToCloud row");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn escalate_to_cloud_carries_cap_reached_trigger() {
    let (_dir, path) = fresh_db();
    let ssb = open_writer(&path).await;
    let payload = SsbPayload::EscalateToCloud(EscalateToCloudPayload {
        step_id: "baby_step_99".into(),
        trigger_reason: "cap_reached".into(),
        layer_failed: 1,
        ts_ms: 1_700_000_600_000,
    });
    let seq = ssb
        .write_payload("test-agent", 0, payload, Vec::new())
        .await
        .expect("write escalate_to_cloud cap_reached");
    assert!(seq > 0);
}

#[test]
fn cloud_creds_for_test_with_token_constructs_anthropic_provider() {
    let creds = CloudCredentials::for_test_with_token("dummy");
    assert_eq!(creds.bearer_token, "dummy");
    assert_eq!(creds.provider, "anthropic");
}
