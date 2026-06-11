//! C-2 contract test (Layer-4) — agent-status SSB emission + read path.
//!
//! Source plan: docs/thefinalglance/C-plan/C-2_ssb_emission_orchestration.md
//! Wiring doctrine: .claude/rules/wiring-enforcement.md Layer-4 — assert the
//! off-agent `AgentStatusChanged` row lands and is read back by the REAL
//! dashboard read path (`SsbViewer::recent_agent_status_changes`, the latest-
//! per-agent lifecycle reducer the MonitorWindow renders). Missing evidence =
//! FAIL.
//!
//! The production producer `bsp::engine::emit_agent_status_changed` is module-
//! private and only fires from inside a full BSP superstep loop; it writes the
//! `AgentStatusChanged` payload through `Ssb::write_payload` (engine.rs:254/289
//! use the identical writer). These tests drive that same production write API
//! and assert the production reducer reads it correctly — the load-bearing
//! evidence is the bytes round-tripping through the real SQLite Ssb + the real
//! `SsbViewer` reader, never a hand-built in-memory payload.

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::ssb::{
    AgentStatusChangedPayload, Ssb, SsbActionType, SsbConfig, SsbPayload,
};
use elai_orchestrator::SsbViewer;
use tempfile::TempDir;

const LIFECYCLE_EMITTER: &str = "bsp_engine";

fn fresh_db() -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("ssb.db");
    (dir, path)
}

fn open_writer(path: &std::path::Path) -> Ssb {
    let cfg = SsbConfig {
        db_path: path.to_path_buf(),
        pool_size: Some(2),
        project_id: "c2-agent-status".into(),
        session_id: "c2-agent-status-test".into(),
    };
    Ssb::new(cfg, Arc::new(OsClock)).expect("ssb init")
}

async fn write_status(ssb: &Ssb, agent_id: &str, role: &str, state: &str, death: Option<&str>) {
    let payload = SsbPayload::AgentStatusChanged(AgentStatusChangedPayload {
        agent_id: agent_id.to_owned(),
        role: role.to_owned(),
        state: state.to_owned(),
        death_class: death.map(str::to_owned),
        timestamp_ms: 0,
    });
    ssb.write_payload(LIFECYCLE_EMITTER, 0, payload, vec![])
        .await
        .expect("write AgentStatusChanged");
}

// ---------------------------------------------------------------------------
// (1) A single agent-status emit lands and is read back by the real reducer
//     with all fields intact (Layer-4 off-agent artifact present = not wire-dark).
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn agent_status_changed_emit_read_round_trips() {
    let (_dir, path) = fresh_db();
    let ssb = open_writer(&path);

    write_status(&ssb, "agent-exec-1", "executor", "spawned", None).await;

    let viewer = SsbViewer::open(&path).expect("viewer open");
    let rows = viewer
        .recent_agent_status_changes(32)
        .expect("recent_agent_status_changes");
    assert_eq!(
        rows.len(),
        1,
        "exactly one agent-status row must be read back (Layer-4 FAIL if zero)"
    );
    assert_eq!(rows[0].agent_id, "agent-exec-1");
    assert_eq!(rows[0].role, "executor");
    assert_eq!(rows[0].state, "spawned");
    assert_eq!(rows[0].death_class, None);
    // Confirm the wire action_type tag is the C-2 canonical string.
    assert_eq!(
        SsbActionType::AgentStatusChanged.as_sql_str(),
        "agent_status_changed"
    );
}

// ---------------------------------------------------------------------------
// (2) The reducer returns LATEST-per-agent: a spawned->exited lifecycle for one
//     agent collapses to the exited row, while a distinct agent stays separate.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn agent_status_reducer_keeps_latest_per_agent() {
    let (_dir, path) = fresh_db();
    let ssb = open_writer(&path);

    // agent A: spawned then exited(clean_exit) — the production lifecycle.
    write_status(&ssb, "agent-A", "auditor", "spawned", None).await;
    write_status(&ssb, "agent-A", "auditor", "exited", Some("clean_exit")).await;
    // agent B: spawned only.
    write_status(&ssb, "agent-B", "executor", "spawned", None).await;

    let viewer = SsbViewer::open(&path).expect("viewer open");
    let rows = viewer
        .recent_agent_status_changes(32)
        .expect("recent_agent_status_changes");

    // Latest-per-agent: exactly two agents, A collapsed to its newest (exited).
    assert_eq!(
        rows.len(),
        2,
        "reducer must keep one row per agent_id; got {rows:?}"
    );
    let a = rows
        .iter()
        .find(|r| r.agent_id == "agent-A")
        .expect("agent-A present");
    assert_eq!(
        a.state, "exited",
        "agent-A must reduce to its LATEST state (exited), not the older spawned row"
    );
    assert_eq!(a.death_class.as_deref(), Some("clean_exit"));
    let b = rows
        .iter()
        .find(|r| r.agent_id == "agent-B")
        .expect("agent-B present");
    assert_eq!(b.state, "spawned");
}
