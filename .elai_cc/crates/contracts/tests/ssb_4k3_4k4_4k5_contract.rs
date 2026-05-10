//! Contract tests for orchestrator phase 4K.3 + 4K.4 + 4K.5 (bundled).
//!
//! Split from dashboard_4k3_4k4_4k5_contract.rs in s-1875676 (handoff §3.2).
//! These tests touch only `elai_orchestrator` symbols (no `use elai_dashboard_lib`),
//! so they are ungated here while the dashboard crate remains backed up. The
//! source file stays `#![cfg(any())]`-gated until the dashboard crate is
//! restored.
//!
//! Schema-version assertion updated from v9 -> v25 to match SCHEMA_VERSION
//! after Track B bump (s-1430428).

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::{
    DiffSummary, InterventionKeyword, InterventionOption, InterventionRequestPayload,
    InterventionResponsePayload, InterventionRetry, InterventionType, PreviewAvailablePayload,
    Ssb, SsbConfig, SsbPayload, SsbViewer, SsbViewerError, StepStatus,
    SsbActionDetail, TaskSubmittedPayload, validate_request_id,
};
use tempfile::TempDir;

// B-23 Phase 4H Group A: bumped 25 -> 28 atomically. The constant was stale
// at 25 since at least Track B (live SCHEMA_VERSION had moved to 26 then 27
// without this contract being updated). 28 covers the v28 self-mod bump.
const EXPECTED_SCHEMA: i64 = 28;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn fresh_db() -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("ssb.db");
    (dir, path)
}

async fn open_writer(path: &std::path::Path, project_id: &str, session_id: &str) -> Ssb {
    let cfg = SsbConfig {
        db_path: path.to_path_buf(),
        pool_size: Some(2),
        project_id: project_id.into(),
        session_id: session_id.into(),
    };
    Ssb::new(cfg, Arc::new(OsClock)).expect("ssb init")
}

fn open_viewer(path: &std::path::Path) -> SsbViewer {
    SsbViewer::open(path).expect("viewer open")
}

// ---------------------------------------------------------------------------
// 4K.3 contract tests — projects store + create_project + submit_task + schema
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_4k3_submit_task_writes_ssb_action_type_task_submitted() {
    let (_d, path) = fresh_db();
    let writer = open_writer(&path, "dashboard", "dash-test").await;

    let payload = TaskSubmittedPayload {
        task: "build me a portfolio".into(),
        project_id: "proj-1".into(),
        submitted_at_ms: 1_713_300_000_000,
    };
    let seq = writer
        .write_payload("dashboard", 0, SsbPayload::TaskSubmitted(payload.clone()), vec![])
        .await
        .expect("write task_submitted");
    assert!(seq > 0, "seq must be assigned");

    drop(writer);
    let viewer = open_viewer(&path);
    let detail = viewer.get_action_detail(seq).expect("detail");
    assert_eq!(detail.action_type, "task_submitted");
    let json = detail.payload_json;
    assert_eq!(json["TaskSubmitted"]["task"], "build me a portfolio");
    assert_eq!(json["TaskSubmitted"]["project_id"], "proj-1");
    assert_eq!(json["TaskSubmitted"]["submitted_at_ms"], 1_713_300_000_000_i64);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_4k3_schema_v9_migration_idempotent() {
    let (_d, path) = fresh_db();
    let writer = open_writer(&path, "p", "s").await;
    drop(writer);

    let conn = rusqlite::Connection::open(&path).expect("open");
    let v: i64 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .expect("user_version");
    assert_eq!(v, EXPECTED_SCHEMA, "schema version must be 25 after first init");

    let writer2 = open_writer(&path, "p", "s").await;
    drop(writer2);
    let v2: i64 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .expect("user_version");
    assert_eq!(v2, EXPECTED_SCHEMA, "second open must remain at v25 (idempotent)");
}

#[tokio::test]
async fn contract_4k3_task_submitted_payload_field_names_locked() {
    let payload = SsbPayload::TaskSubmitted(TaskSubmittedPayload {
        task: "t".into(),
        project_id: "p".into(),
        submitted_at_ms: 42,
    });
    let bytes = rmp_serde::to_vec_named(&payload).expect("encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
    assert_eq!(decoded, payload);

    if let SsbPayload::TaskSubmitted(p) = decoded {
        assert_eq!(p.task, "t");
        assert_eq!(p.project_id, "p");
        assert_eq!(p.submitted_at_ms, 42);
    } else {
        panic!("variant lost on round-trip");
    }
}

// ---------------------------------------------------------------------------
// 4K.4 contract tests — step_progress + phase_progress preservation
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_4k4_step_progress_returns_status_per_step_id() {
    let (_d, path) = fresh_db();
    let writer = open_writer(&path, "p", "s").await;

    use elai_orchestrator::PhaseCompletePayload;
    for step_id in ["1.1", "1.2", "1.3"] {
        let payload = SsbPayload::PhaseComplete(PhaseCompletePayload {
            phase_id: step_id.into(),
            agent_id: "agent-x".into(),
            fsm_state: "complete".into(),
            evidence_artifacts: vec![],
            commit_hashes: vec![],
            test_outcomes: vec![],
        });
        writer
            .write_payload("agent-x", 0, payload, vec![])
            .await
            .expect("write");
    }
    drop(writer);

    let viewer = open_viewer(&path);
    let rows = viewer.step_progress("1").expect("step_progress");
    assert_eq!(rows.len(), 3);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_4k4_step_progress_empty_phase_returns_empty_vec() {
    let (_d, path) = fresh_db();
    let writer = open_writer(&path, "p", "s").await;
    drop(writer);
    let viewer = open_viewer(&path);
    let rows = viewer.step_progress("9.9.9").expect("step_progress empty");
    assert!(rows.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_4k4_step_progress_phase_id_validation() {
    let (_d, path) = fresh_db();
    {
        let _w = open_writer(&path, "p", "s").await;
    }
    let viewer = open_viewer(&path);

    let r = viewer.step_progress("");
    assert!(matches!(r, Err(SsbViewerError::Validation(_))));

    let r = viewer.step_progress("..");
    assert!(matches!(r, Err(SsbViewerError::Validation(_))));

    let too_long = "a".repeat(65);
    let r = viewer.step_progress(&too_long);
    assert!(matches!(r, Err(SsbViewerError::Validation(_))));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_4k4_step_progress_orders_by_step_id_asc() {
    let (_d, path) = fresh_db();
    let writer = open_writer(&path, "p", "s").await;

    use elai_orchestrator::PhaseCompletePayload;
    for step_id in ["1.3", "1.1", "1.2"] {
        let payload = SsbPayload::PhaseComplete(PhaseCompletePayload {
            phase_id: step_id.into(),
            agent_id: "a".into(),
            fsm_state: "complete".into(),
            evidence_artifacts: vec![],
            commit_hashes: vec![],
            test_outcomes: vec![],
        });
        writer
            .write_payload("a", 0, payload, vec![])
            .await
            .expect("write");
    }
    drop(writer);

    let viewer = open_viewer(&path);
    let rows = viewer.step_progress("1").expect("step_progress");
    let ids: Vec<String> = rows.iter().map(|s| s.step_id.clone()).collect();
    assert_eq!(ids, vec!["1.1", "1.2", "1.3"]);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_4k4_phase_progress_unchanged_signature() {
    let (_d, path) = fresh_db();
    let writer = open_writer(&path, "p", "s").await;
    drop(writer);
    let viewer = open_viewer(&path);
    let _: Vec<elai_orchestrator::PhaseStatus> = viewer.phase_progress().expect("phase_progress");
}

// ---------------------------------------------------------------------------
// 4K.5 contract tests — get_action_detail + intervention_response real write +
// preview payload shape
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_4k5_get_action_detail_returns_full_payload_json() {
    let (_d, path) = fresh_db();
    let writer = open_writer(&path, "p", "s").await;

    let payload = SsbPayload::InterventionRequest(InterventionRequestPayload {
        step_id: "step-X".into(),
        intervention_type: InterventionType::ApproveChange,
        description: "approve?".into(),
        diff_summary: Some(DiffSummary {
            files_changed: 1,
            lines_added: 5,
            lines_removed: 2,
        }),
        options: vec![InterventionOption::Keyword(InterventionKeyword::Accept)],
    });
    let seq = writer
        .write_payload("agent-x", 0, payload, vec!["src/a.rs".into()])
        .await
        .expect("write");
    drop(writer);

    let viewer = open_viewer(&path);
    let detail = viewer.get_action_detail(seq).expect("detail");
    assert_eq!(detail.seq, seq);
    assert_eq!(detail.action_type, "intervention_request");
    assert_eq!(detail.agent_id, "agent-x");
    assert_eq!(detail.files, vec!["src/a.rs".to_string()]);
    let json = detail.payload_json;
    assert_eq!(json["InterventionRequest"]["step_id"], "step-X");
    assert_eq!(json["InterventionRequest"]["description"], "approve?");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_4k5_get_action_detail_unknown_seq_returns_notfound() {
    let (_d, path) = fresh_db();
    let writer = open_writer(&path, "p", "s").await;
    drop(writer);
    let viewer = open_viewer(&path);
    let r = viewer.get_action_detail(99_999);
    assert!(matches!(
        r,
        Err(SsbViewerError::Validation(_))
            | Err(SsbViewerError::OutOfRange(_))
            | Err(SsbViewerError::Sqlite(_))
    ));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_4k5_get_action_detail_invalid_seq_negative_rejected() {
    let (_d, path) = fresh_db();
    {
        let _w = open_writer(&path, "p", "s").await;
    }
    let viewer = open_viewer(&path);
    let r = viewer.get_action_detail(-1);
    assert!(matches!(r, Err(SsbViewerError::Validation(_))));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_4k5_submit_intervention_response_writes_real_payload() {
    let (_d, path) = fresh_db();
    let writer = open_writer(&path, "dashboard", "dash-test").await;

    let payload = SsbPayload::InterventionResponse(InterventionResponsePayload {
        request_id: 42,
        chosen: InterventionOption::Retry(InterventionRetry {
            retry: "more evidence".into(),
        }),
        user_note: Some("nope".into()),
    });
    let seq = writer
        .write_payload("dashboard", 0, payload, vec![])
        .await
        .expect("write resp");
    drop(writer);

    let viewer = open_viewer(&path);
    let detail = viewer.get_action_detail(seq).expect("detail");
    assert_eq!(detail.action_type, "intervention_response");
    let j = detail.payload_json;
    assert_eq!(j["InterventionResponse"]["request_id"], 42);
    assert_eq!(j["InterventionResponse"]["chosen"]["retry"], "more evidence");
}

#[test]
fn contract_4k5_intervention_response_request_id_overflow_rejected() {
    let r = validate_request_id(u64::MAX);
    assert!(r.is_err());
    let r2 = validate_request_id(i64::MAX as u64);
    assert!(r2.is_ok());
}

#[test]
fn contract_4k5_preview_available_payload_field_names_unchanged() {
    let p = PreviewAvailablePayload {
        url: "http://localhost:5173".into(),
        step_id: "s".into(),
        slot: "live".into(),
        agent_id: "a".into(),
        build_success: true,
        dev_server_ok: true,
    };
    let bytes = rmp_serde::to_vec_named(&p).expect("encode");
    let decoded: PreviewAvailablePayload = rmp_serde::from_slice(&bytes).expect("decode");
    assert_eq!(decoded.url, "http://localhost:5173");
    assert_eq!(decoded.step_id, "s");
    assert_eq!(decoded.slot, "live");
    assert_eq!(decoded.agent_id, "a");
    assert!(decoded.build_success);
    assert!(decoded.dev_server_ok);
}

// ---------------------------------------------------------------------------
// StepStatus shape lock
// ---------------------------------------------------------------------------

#[test]
fn contract_step_status_shape_locked() {
    let s = StepStatus {
        step_id: "1.1".into(),
        status: "complete".into(),
        started_at: Some(1),
        completed_at: Some(2),
        agent_id: Some("a".into()),
    };
    let bytes = serde_json::to_vec(&s).expect("encode");
    let v: serde_json::Value = serde_json::from_slice(&bytes).expect("decode");
    assert_eq!(v["step_id"], "1.1");
    assert_eq!(v["status"], "complete");
    assert_eq!(v["started_at"], 1);
    assert_eq!(v["completed_at"], 2);
    assert_eq!(v["agent_id"], "a");
}

#[test]
fn contract_action_detail_shape_locked() {
    let d = SsbActionDetail {
        seq: 1,
        ts: 2,
        action_type: "task_submitted".into(),
        agent_id: "dashboard".into(),
        payload_json: serde_json::json!({"k": "v"}),
        files: vec!["a.rs".into()],
    };
    let bytes = serde_json::to_vec(&d).expect("encode");
    let v: serde_json::Value = serde_json::from_slice(&bytes).expect("decode");
    assert_eq!(v["seq"], 1);
    assert_eq!(v["ts"], 2);
    assert_eq!(v["action_type"], "task_submitted");
    assert_eq!(v["agent_id"], "dashboard");
    assert_eq!(v["payload_json"]["k"], "v");
    assert_eq!(v["files"][0], "a.rs");
}
