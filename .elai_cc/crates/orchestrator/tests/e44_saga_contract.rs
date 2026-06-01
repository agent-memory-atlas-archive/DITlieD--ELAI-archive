//! E-44 Tranche 7A + 7C — contract tests for the SAGA + reliability +
//! error_taxonomy producer surfaces.
//!
//! Five tests:
//!   1. log_compensation transitions Pending -> Committed and (with an
//!      SSB writer + env gate on) emits one CompensationLogged row.
//!   2. replay_pending drains the pending map + emits N
//!      CompensationReplayed rows.
//!   3. record_failure emits one CompensationFailed row.
//!   4. ReliabilityMetrics computed correctly from a window.
//!   5. ToolErrorClassifier classifies common errors into the right
//!      ErrorClass arm.
//!
//! `.expect()` is OK in test code per CLAUDE.md no-unwrap exception.

use std::sync::Arc;

use elai_core::clock::SeededClock;
use elai_orchestrator::durability::{
    compute_metrics, AgentReliability, ErrorClass, ReliabilityWindow, SagaAction, SagaLogger,
    SagaTicket, ToolErrorClassifier,
};
use elai_orchestrator::durability::saga::Pending;
use elai_orchestrator::ssb::{
    Ssb, SsbActionType, SsbConfig, ELAI_DURABILITY_OBSERVABILITY_ENABLE_ENV,
};
use tempfile::TempDir;

fn test_ssb() -> (TempDir, Arc<Ssb>) {
    let dir = TempDir::new().expect("tempdir");
    let cfg = SsbConfig {
        db_path: dir.path().join("test_ssb.db"),
        pool_size: Some(2),
        project_id: "test-project".into(),
        session_id: "test-session".into(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(SeededClock::new());
    let ssb = Ssb::new(cfg, clock).expect("ssb init");
    (dir, Arc::new(ssb))
}

fn enable_env_gate() {
    // SAFETY: env mutation across tests is single-threaded by convention.
    unsafe {
        std::env::set_var(ELAI_DURABILITY_OBSERVABILITY_ENABLE_ENV, "1");
    }
}

fn disable_env_gate() {
    unsafe {
        std::env::remove_var(ELAI_DURABILITY_OBSERVABILITY_ENABLE_ENV);
    }
}

fn fresh_action(saga_id: &str, step_index: u32) -> SagaAction {
    SagaAction {
        saga_id: saga_id.to_owned(),
        action_id: SagaAction::derive_action_id(saga_id, step_index, "undo"),
        step_index,
        compensate_tool: "undo".into(),
        compensate_args_json: serde_json::json!({"k": "v"}),
        recorded_at_ms: 100,
    }
}

#[tokio::test]
async fn test_1_log_compensation_emits_row() {
    enable_env_gate();
    let (_dir, ssb) = test_ssb();
    let logger = SagaLogger::new()
        .with_ssb(Arc::clone(&ssb))
        .with_agent_id("task_runner");
    let action = fresh_action("saga-A", 0);
    let pending: SagaTicket<Pending> = SagaTicket::new("saga-A".into(), action);
    let _committed = logger
        .log_compensation(pending)
        .await
        .expect("log_compensation should succeed with gate on");

    // Allow the SSB write_loop to drain.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Query SSB for the DurabilityEvent row.
    let rows = <Ssb as elai_orchestrator::ssb::SsbReader>::query_by_type(
        ssb.as_ref(),
        SsbActionType::DurabilityEvent,
    )
    .expect("query");
    assert!(
        !rows.is_empty(),
        "expected at least one DurabilityEvent row after log_compensation with gate on"
    );
    disable_env_gate();
}

#[tokio::test]
async fn test_2_replay_pending_emits_rows() {
    enable_env_gate();
    let (_dir, ssb) = test_ssb();
    let logger = SagaLogger::new().with_ssb(Arc::clone(&ssb));

    for i in 0..3 {
        let action = fresh_action("saga-B", i);
        let pending: SagaTicket<Pending> = SagaTicket::new("saga-B".into(), action);
        let _ = logger
            .log_compensation(pending)
            .await
            .expect("log_compensation");
    }
    let replayed = logger.replay_pending().await.expect("replay");
    assert_eq!(replayed, 3);

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let rows = <Ssb as elai_orchestrator::ssb::SsbReader>::query_by_type(
        ssb.as_ref(),
        SsbActionType::DurabilityEvent,
    )
    .expect("query");
    // 3 logs + 3 replays = at least 6 rows
    assert!(rows.len() >= 6, "expected >=6 rows, got {}", rows.len());
    disable_env_gate();
}

#[tokio::test]
async fn test_3_record_failure_emits_row() {
    enable_env_gate();
    let (_dir, ssb) = test_ssb();
    let logger = SagaLogger::new().with_ssb(Arc::clone(&ssb));
    let action = fresh_action("saga-C", 0);
    let pending: SagaTicket<Pending> = SagaTicket::new("saga-C".into(), action);
    let _failed = logger
        .record_failure(
            pending,
            elai_orchestrator::durability::SagaError::Lifecycle {
                reason: "test-failure".into(),
            },
        )
        .await
        .expect("record_failure");

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let rows = <Ssb as elai_orchestrator::ssb::SsbReader>::query_by_type(
        ssb.as_ref(),
        SsbActionType::DurabilityEvent,
    )
    .expect("query");
    assert!(!rows.is_empty(), "expected CompensationFailed row");
    disable_env_gate();
}

#[test]
fn test_4_reliability_metrics_from_window() {
    let mut win = ReliabilityWindow::new(60_000);
    for latency in [10_u64, 20, 30, 40, 50] {
        win.push(AgentReliability {
            agent_id: "agent-X".into(),
            success: true,
            latency_ms: latency,
            last_failure_class: None,
        });
    }
    win.push(AgentReliability {
        agent_id: "agent-X".into(),
        success: false,
        latency_ms: 999,
        last_failure_class: Some("transient_network".into()),
    });
    let m = compute_metrics(&win);
    assert_eq!(m.sample_window, 6);
    // 5 successes / 6 total ~= 0.833
    assert!(m.success_rate > 0.8 && m.success_rate < 0.9);
    // Sorted: [10,20,30,40,50,999]; p50 nearest-rank = round(0.5*5)=3 -> 40
    assert_eq!(m.p50_latency_ms, 40);
    // p95 nearest-rank = round(0.95*5)=5 -> 999
    assert_eq!(m.p95_latency_ms, 999);
}

#[test]
fn test_5_tool_error_classifier_buckets() {
    let c = ToolErrorClassifier::new();
    assert_eq!(c.classify("connection refused"), ErrorClass::TransientNetwork);
    assert_eq!(c.classify("HTTP 429 Too Many Requests"), ErrorClass::RateLimited);
    assert_eq!(c.classify("401 Unauthorized"), ErrorClass::AuthFailure);
    assert_eq!(c.classify("400 Bad Request: schema mismatch"), ErrorClass::InvalidInput);
    assert_eq!(c.classify("thread main panicked"), ErrorClass::ToolBug);
    assert_eq!(c.classify("orbital decay"), ErrorClass::Unknown);

    // Round-trip via stable wire labels.
    assert_eq!(ErrorClass::TransientNetwork.as_str(), "transient_network");
    assert_eq!(ErrorClass::ToolBug.recovery_action(), "compensate");
}
