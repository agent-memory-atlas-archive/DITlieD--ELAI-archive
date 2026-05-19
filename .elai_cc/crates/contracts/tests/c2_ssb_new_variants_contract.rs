//! C-2 contract test — verifies the 10 new SSB variants added by C-2 and
//! the SCHEMA_VERSION bump 47 -> 48.
//!
//! Source plan: docs/thefinalglance/C-plan/C-2_ssb_emission_orchestration.md
//! Decisions: workflow/reasoning/C-2-decisions.md (DRIFTs 1-10 resolved).
//!
//! Tests are intentionally narrow (one assertion per concern):
//!   (1) SCHEMA_VERSION_PUBLIC reports 48.
//!   (2) Every new SsbActionType serializes to its expected snake_case string.
//!   (3) Every new payload variant round-trips through the SQLite SSB writer
//!       and SsbViewer query path (one tokio::test per variant).
//!   (4) SsbViewer::recent_agent_status_changes() returns latest-per-agent
//!       lifecycle and deduplicates older rows for the same agent_id.
//!   (5) Ambiguity viewer helpers (latest_ambiguity_scored /
//!       latest_ambiguity_floor_failed / latest_ambiguity_streak) round-trip.

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::ssb::{
    AgentStatusChangedPayload, AmbiguityFloorFailedPayload, AmbiguityScoredPayload,
    AmbiguityStreakAdvancedPayload, BabyStepReadyPayload, DagRepairEscalatedPayload,
    GoalDriftWarningPayload, LateralThinkingActivatedPayload, PhaseNodeReadyPayload, Ssb,
    SsbActionType, SsbConfig, SsbPayload, StagnationOscillationDetectedPayload,
    StagnationSpinningDetectedPayload, StepNodeReadyPayload, WatchdogTimeoutPayload,
    SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::SsbViewer;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn fresh_db() -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("ssb.db");
    (dir, path)
}

async fn open_writer(path: &std::path::Path) -> Ssb {
    let cfg = SsbConfig {
        db_path: path.to_path_buf(),
        pool_size: Some(2),
        project_id: "c2-contract".into(),
        session_id: "c2-test".into(),
    };
    Ssb::new(cfg, Arc::new(OsClock)).expect("ssb init")
}

fn open_viewer(path: &std::path::Path) -> SsbViewer {
    SsbViewer::open(path).expect("viewer open")
}

// ---------------------------------------------------------------------------
// (1) SCHEMA_VERSION bump 47 -> 48.
// ---------------------------------------------------------------------------

#[test]
fn schema_version_advances_to_48_after_c2() {
    // Atomic single-bump invariant: C-2 ADDED 10 variants in one patch.
    // Future C-N plans must not regress this number; B-34 floor pin is
    // still preserved (>= 42).
    assert!(
        SCHEMA_VERSION_PUBLIC >= 48,
        "SCHEMA_VERSION_PUBLIC must be at least 48 after C-2; got {}",
        SCHEMA_VERSION_PUBLIC
    );
}

// ---------------------------------------------------------------------------
// (2) Action-type string serialization.
// ---------------------------------------------------------------------------

#[test]
fn new_action_types_serialize_to_expected_strings() {
    assert_eq!(SsbActionType::PhaseNodeReady.as_sql_str(), "phase_node_ready");
    assert_eq!(SsbActionType::StepNodeReady.as_sql_str(), "step_node_ready");
    assert_eq!(SsbActionType::BabyStepReady.as_sql_str(), "baby_step_ready");
    assert_eq!(
        SsbActionType::DagRepairEscalated.as_sql_str(),
        "dag_repair_escalated"
    );
    assert_eq!(
        SsbActionType::AgentStatusChanged.as_sql_str(),
        "agent_status_changed"
    );
    assert_eq!(
        SsbActionType::StagnationSpinningDetected.as_sql_str(),
        "stagnation_spinning_detected"
    );
    assert_eq!(
        SsbActionType::StagnationOscillationDetected.as_sql_str(),
        "stagnation_oscillation_detected"
    );
    assert_eq!(
        SsbActionType::LateralThinkingActivated.as_sql_str(),
        "lateral_thinking_activated"
    );
    assert_eq!(
        SsbActionType::GoalDriftWarning.as_sql_str(),
        "goal_drift_warning"
    );
    assert_eq!(SsbActionType::WatchdogTimeout.as_sql_str(), "watchdog_timeout");
}

// ---------------------------------------------------------------------------
// (3) Round-trip each new payload variant through write_payload + viewer
//     get_action_detail.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn phase_node_ready_payload_round_trips() {
    let (_d, path) = fresh_db();
    let writer = open_writer(&path).await;
    let payload = PhaseNodeReadyPayload {
        phase_id: 7,
        concept_id: 3,
        label: "phase-x".into(),
        depth: 1,
    };
    let seq = writer
        .write_payload(
            "task_runner",
            0,
            SsbPayload::PhaseNodeReady(payload.clone()),
            vec![],
        )
        .await
        .expect("write phase_node_ready");
    drop(writer);
    let viewer = open_viewer(&path);
    let detail = viewer.get_action_detail(seq).expect("detail");
    assert_eq!(detail.action_type, "phase_node_ready");
    let json = detail.payload_json;
    assert_eq!(json["PhaseNodeReady"]["phase_id"], 7);
    assert_eq!(json["PhaseNodeReady"]["concept_id"], 3);
    assert_eq!(json["PhaseNodeReady"]["label"], "phase-x");
    assert_eq!(json["PhaseNodeReady"]["depth"], 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn step_node_ready_payload_round_trips() {
    let (_d, path) = fresh_db();
    let writer = open_writer(&path).await;
    let payload = StepNodeReadyPayload {
        step_id: 12,
        phase_id: 7,
        concept_id: 3,
        label: "step-y".into(),
        depth: 2,
    };
    let seq = writer
        .write_payload(
            "task_runner",
            0,
            SsbPayload::StepNodeReady(payload.clone()),
            vec![],
        )
        .await
        .expect("write step_node_ready");
    drop(writer);
    let viewer = open_viewer(&path);
    let detail = viewer.get_action_detail(seq).expect("detail");
    assert_eq!(detail.action_type, "step_node_ready");
    let json = detail.payload_json;
    assert_eq!(json["StepNodeReady"]["step_id"], 12);
    assert_eq!(json["StepNodeReady"]["phase_id"], 7);
    assert_eq!(json["StepNodeReady"]["label"], "step-y");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn baby_step_ready_payload_round_trips_with_full_ancestry() {
    let (_d, path) = fresh_db();
    let writer = open_writer(&path).await;
    let payload = BabyStepReadyPayload {
        baby_step_id: 42,
        step_id: 12,
        phase_id: 7,
        concept_id: 3,
        label: "bs-z".into(),
        depth: 3,
    };
    let seq = writer
        .write_payload(
            "task_runner",
            0,
            SsbPayload::BabyStepReady(payload.clone()),
            vec![],
        )
        .await
        .expect("write baby_step_ready");
    drop(writer);
    let viewer = open_viewer(&path);
    let detail = viewer.get_action_detail(seq).expect("detail");
    assert_eq!(detail.action_type, "baby_step_ready");
    let json = detail.payload_json;
    assert_eq!(json["BabyStepReady"]["baby_step_id"], 42);
    assert_eq!(json["BabyStepReady"]["step_id"], 12);
    assert_eq!(json["BabyStepReady"]["phase_id"], 7);
    assert_eq!(json["BabyStepReady"]["concept_id"], 3);
    assert_eq!(json["BabyStepReady"]["label"], "bs-z");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dag_repair_escalated_payload_round_trips() {
    let (_d, path) = fresh_db();
    let writer = open_writer(&path).await;
    let payload = DagRepairEscalatedPayload {
        rounds: 10,
        remaining_issues: 2,
        last_error: "cycle persists".into(),
    };
    let seq = writer
        .write_payload(
            "pipeline_builder",
            0,
            SsbPayload::DagRepairEscalated(payload.clone()),
            vec![],
        )
        .await
        .expect("write dag_repair_escalated");
    drop(writer);
    let viewer = open_viewer(&path);
    let detail = viewer.get_action_detail(seq).expect("detail");
    assert_eq!(detail.action_type, "dag_repair_escalated");
    let json = detail.payload_json;
    assert_eq!(json["DagRepairEscalated"]["rounds"], 10);
    assert_eq!(json["DagRepairEscalated"]["remaining_issues"], 2);
    assert_eq!(json["DagRepairEscalated"]["last_error"], "cycle persists");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn agent_status_changed_payload_round_trips_single_state_field() {
    let (_d, path) = fresh_db();
    let writer = open_writer(&path).await;
    // DRIFT-5: payload carries a SINGLE state field, no old_lifecycle.
    let payload = AgentStatusChangedPayload {
        agent_id: "executor-1".into(),
        role: "executor".into(),
        state: "spawned".into(),
        death_class: None,
        timestamp_ms: 1_700_000_000_000,
    };
    let seq = writer
        .write_payload(
            "bsp_engine",
            0,
            SsbPayload::AgentStatusChanged(payload.clone()),
            vec![],
        )
        .await
        .expect("write agent_status_changed");
    drop(writer);
    let viewer = open_viewer(&path);
    let detail = viewer.get_action_detail(seq).expect("detail");
    assert_eq!(detail.action_type, "agent_status_changed");
    let json = detail.payload_json;
    assert_eq!(json["AgentStatusChanged"]["agent_id"], "executor-1");
    assert_eq!(json["AgentStatusChanged"]["role"], "executor");
    assert_eq!(json["AgentStatusChanged"]["state"], "spawned");
    // old_lifecycle MUST NOT be present (DRIFT-5).
    assert!(
        json["AgentStatusChanged"]["old_lifecycle"].is_null(),
        "AgentStatusChangedPayload must not carry old_lifecycle (DRIFT-5 in C-2 plan): {:?}",
        json
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stagnation_spinning_detected_payload_round_trips() {
    let (_d, path) = fresh_db();
    let writer = open_writer(&path).await;
    let payload = StagnationSpinningDetectedPayload {
        agent_id: "executor-1".into(),
        pattern: "spinning".into(),
        iteration: 3,
        detail: "step_hash repeated 3x".into(),
    };
    let seq = writer
        .write_payload(
            "anti_spiral",
            0,
            SsbPayload::StagnationSpinningDetected(payload.clone()),
            vec![],
        )
        .await
        .expect("write stagnation_spinning_detected");
    drop(writer);
    let viewer = open_viewer(&path);
    let detail = viewer.get_action_detail(seq).expect("detail");
    assert_eq!(detail.action_type, "stagnation_spinning_detected");
    let json = detail.payload_json;
    assert_eq!(json["StagnationSpinningDetected"]["pattern"], "spinning");
    assert_eq!(json["StagnationSpinningDetected"]["iteration"], 3);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stagnation_oscillation_detected_payload_round_trips() {
    let (_d, path) = fresh_db();
    let writer = open_writer(&path).await;
    let payload = StagnationOscillationDetectedPayload {
        agent_id: "executor-1".into(),
        pattern: "oscillation".into(),
        iteration: 4,
        detail: "A-B-A-B".into(),
    };
    let seq = writer
        .write_payload(
            "anti_spiral",
            0,
            SsbPayload::StagnationOscillationDetected(payload.clone()),
            vec![],
        )
        .await
        .expect("write stagnation_oscillation_detected");
    drop(writer);
    let viewer = open_viewer(&path);
    let detail = viewer.get_action_detail(seq).expect("detail");
    assert_eq!(detail.action_type, "stagnation_oscillation_detected");
    let json = detail.payload_json;
    assert_eq!(json["StagnationOscillationDetected"]["pattern"], "oscillation");
    assert_eq!(json["StagnationOscillationDetected"]["iteration"], 4);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn lateral_thinking_activated_payload_round_trips() {
    let (_d, path) = fresh_db();
    let writer = open_writer(&path).await;
    let payload = LateralThinkingActivatedPayload {
        agent_id: "executor-1".into(),
        persona: "contrarian".into(),
        iteration: 5,
        detail: "switched after spinning".into(),
    };
    let seq = writer
        .write_payload(
            "anti_spiral",
            0,
            SsbPayload::LateralThinkingActivated(payload.clone()),
            vec![],
        )
        .await
        .expect("write lateral_thinking_activated");
    drop(writer);
    let viewer = open_viewer(&path);
    let detail = viewer.get_action_detail(seq).expect("detail");
    assert_eq!(detail.action_type, "lateral_thinking_activated");
    let json = detail.payload_json;
    assert_eq!(json["LateralThinkingActivated"]["persona"], "contrarian");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn goal_drift_warning_payload_round_trips() {
    let (_d, path) = fresh_db();
    let writer = open_writer(&path).await;
    let payload = GoalDriftWarningPayload {
        agent_id: "executor-1".into(),
        drift_score: 0.42,
        baby_step_id: 17,
        detail: "drift exceeds tolerance".into(),
    };
    let seq = writer
        .write_payload(
            "goal_drift",
            0,
            SsbPayload::GoalDriftWarning(payload.clone()),
            vec![],
        )
        .await
        .expect("write goal_drift_warning");
    drop(writer);
    let viewer = open_viewer(&path);
    let detail = viewer.get_action_detail(seq).expect("detail");
    assert_eq!(detail.action_type, "goal_drift_warning");
    let json = detail.payload_json;
    assert_eq!(json["GoalDriftWarning"]["baby_step_id"], 17);
    // drift_score is f32: serde_json renders it with full precision; compare numerically.
    let drift_score = json["GoalDriftWarning"]["drift_score"].as_f64().expect("f64");
    assert!((drift_score - 0.42_f64).abs() < 1e-4, "drift_score round-trip: got {}", drift_score);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn watchdog_timeout_payload_round_trips() {
    let (_d, path) = fresh_db();
    let writer = open_writer(&path).await;
    let payload = WatchdogTimeoutPayload {
        agent_id: "executor-1".into(),
        elapsed_ms: 30_000,
        budget_ms: 20_000,
        detail: "no observable output".into(),
    };
    let seq = writer
        .write_payload(
            "watchdog",
            0,
            SsbPayload::WatchdogTimeout(payload.clone()),
            vec![],
        )
        .await
        .expect("write watchdog_timeout");
    drop(writer);
    let viewer = open_viewer(&path);
    let detail = viewer.get_action_detail(seq).expect("detail");
    assert_eq!(detail.action_type, "watchdog_timeout");
    let json = detail.payload_json;
    assert_eq!(json["WatchdogTimeout"]["elapsed_ms"], 30_000);
    assert_eq!(json["WatchdogTimeout"]["budget_ms"], 20_000);
}

// ---------------------------------------------------------------------------
// (4) SsbViewer::recent_agent_status_changes() returns latest-per-agent.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn recent_agent_status_changes_returns_latest_per_agent() {
    let (_d, path) = fresh_db();
    let writer = open_writer(&path).await;
    // Write 3 rows: 2 for executor-1 (spawned -> exited), 1 for auditor-1
    // (spawned). The viewer must return one row per agent_id with the latest
    // state ("exited" for executor-1, "spawned" for auditor-1).
    for (agent_id, role, state) in [
        ("executor-1", "executor", "spawned"),
        ("auditor-1", "auditor", "spawned"),
        ("executor-1", "executor", "exited"),
    ] {
        let payload = AgentStatusChangedPayload {
            agent_id: agent_id.into(),
            role: role.into(),
            state: state.into(),
            death_class: None,
            timestamp_ms: 1_700_000_000_000,
        };
        writer
            .write_payload(
                "bsp_engine",
                0,
                SsbPayload::AgentStatusChanged(payload),
                vec![],
            )
            .await
            .expect("write agent_status_changed");
    }
    drop(writer);
    let viewer = open_viewer(&path);
    let rows = viewer
        .recent_agent_status_changes(32)
        .expect("recent_agent_status_changes");
    assert_eq!(rows.len(), 2, "should dedupe to one row per agent_id");
    // Latest (DESC seq) is the executor's "exited" row first.
    assert_eq!(rows[0].agent_id, "executor-1");
    assert_eq!(rows[0].state, "exited");
    assert_eq!(rows[1].agent_id, "auditor-1");
    assert_eq!(rows[1].state, "spawned");
}

// ---------------------------------------------------------------------------
// (5) Ambiguity viewer helpers round-trip.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn latest_ambiguity_helpers_round_trip() {
    let (_d, path) = fresh_db();
    let writer = open_writer(&path).await;

    // Write one of each ambiguity row.
    let scored = AmbiguityScoredPayload {
        score: 0.18,
        milestone: "ready".into(),
        goal: 0.9,
        constraint: 0.85,
        success: 0.88,
        context: None,
        brownfield: false,
    };
    writer
        .write_payload(
            "task_runner",
            0,
            SsbPayload::AmbiguityScored(scored),
            vec![],
        )
        .await
        .expect("write ambiguity_scored");

    let floor = AmbiguityFloorFailedPayload {
        score: 0.18,
        failed_dimensions: vec!["goal_clarity".into()],
    };
    writer
        .write_payload(
            "task_runner",
            0,
            SsbPayload::AmbiguityFloorFailed(floor),
            vec![],
        )
        .await
        .expect("write ambiguity_floor_failed");

    let streak = AmbiguityStreakAdvancedPayload {
        score: 0.18,
        current_streak: 1,
        required: 2,
    };
    writer
        .write_payload(
            "task_runner",
            0,
            SsbPayload::AmbiguityStreakAdvanced(streak),
            vec![],
        )
        .await
        .expect("write ambiguity_streak_advanced");

    drop(writer);
    let viewer = open_viewer(&path);

    let s = viewer
        .latest_ambiguity_scored()
        .expect("latest_ambiguity_scored")
        .expect("Some");
    assert!((s.score - 0.18_f32).abs() < 1e-4);
    assert_eq!(s.milestone, "ready");

    let f = viewer
        .latest_ambiguity_floor_failed()
        .expect("latest_ambiguity_floor_failed")
        .expect("Some");
    assert_eq!(f.failed_dimensions, vec!["goal_clarity".to_owned()]);

    let st = viewer
        .latest_ambiguity_streak()
        .expect("latest_ambiguity_streak")
        .expect("Some");
    assert_eq!(st.current_streak, 1);
    assert_eq!(st.required, 2);
}
