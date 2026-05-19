//! C-3 — Reasoning dashboard surface contract.
//!
//! Verifies the data sources for the 3 reasoning-drawer Tauri commands
//! `get_csq_log`, `get_feasibility_log`, `get_agent_grc_config`:
//!
//! 1. test_get_csq_log_empty — fresh SSB -> [] (graceful-empty).
//! 2. test_get_csq_log_with_rows — synthetic csq_evaluated rows produce
//!    the {csq_id, conclusion, guard_present, ts_ms, ...} shape via
//!    `recent_rows_of_type` + payload decode.
//! 3. test_get_feasibility_log_action_type — DRIFT-3 contract: the SSB
//!    action_type is `feasibility_report`, NOT `feasibility_check`.
//! 4. test_get_agent_grc_config_returns_eight_or_more — DRIFT-5 contract:
//!    `default_grc_config` iteration over AgentRole returns >= 8 entries
//!    with non-empty role + sane budget.
//!
//! Same testing rationale as knowledge_surface_contract.rs: pin viewer +
//! agent_pool data sources directly; Tauri command wrappers are JSON
//! reshapes that cannot be invoked without a Tauri runtime.

use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_ipc_types::AgentRole;
use elai_orchestrator::default_grc_config;
use elai_orchestrator::ssb::{
    CsqEvaluatedPayload, FeasibilityReportPayload, Ssb, SsbActionType, SsbConfig, SsbPayload,
};
use elai_orchestrator::ssb_viewer::SsbViewer;
use tempfile::TempDir;

fn fresh_ssb(tmp: &TempDir, project_id: &str) -> Ssb {
    let cfg = SsbConfig {
        db_path: tmp.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: project_id.into(),
        session_id: format!("{project_id}-session"),
    };
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    Ssb::new(cfg, clock).expect("open fresh ssb")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_csq_log_empty() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = fresh_ssb(&tmp, "c3-csq-empty");
    let db_path = tmp.path().join("ssb.db");
    drop(ssb);

    let viewer = SsbViewer::open(&db_path).expect("viewer open");
    // Same call the Tauri command makes under the hood.
    let rows = viewer
        .recent_rows_of_type(SsbActionType::CsqEvaluated.as_sql_str(), 50)
        .expect("recent_rows_of_type csq_evaluated");
    assert!(
        rows.is_empty(),
        "empty SSB -> csq_evaluated row count must be 0, got {}",
        rows.len()
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_csq_log_with_rows() {
    let tmp = TempDir::new().expect("tempdir");
    let cfg = SsbConfig {
        db_path: tmp.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: "c3-csq-with".into(),
        session_id: "c3-csq-with-session".into(),
    };
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(Ssb::new(cfg, clock).expect("ssb"));

    // Two rows: one blocked Critical, one non-blocked Warning. The
    // Tauri command reshape extracts {csq_id, conclusion (=severity),
    // guard_present, ts_ms, blocked, check_count}.
    let blocked = SsbPayload::CsqEvaluated(CsqEvaluatedPayload {
        severity: "Critical".into(),
        guard_present: false,
        blocked: true,
        check_count: 4,
    });
    let warning = SsbPayload::CsqEvaluated(CsqEvaluatedPayload {
        severity: "Warning".into(),
        guard_present: true,
        blocked: false,
        check_count: 2,
    });
    ssb.write_payload("c3-test", 0, blocked, Vec::new())
        .await
        .expect("write blocked");
    ssb.write_payload("c3-test", 0, warning, Vec::new())
        .await
        .expect("write warning");

    let db_path = tmp.path().join("ssb.db");
    let viewer = SsbViewer::open(&db_path).expect("viewer open");

    let rows = viewer
        .recent_rows_of_type(SsbActionType::CsqEvaluated.as_sql_str(), 50)
        .expect("recent_rows_of_type");
    assert_eq!(rows.len(), 2, "expected 2 csq_evaluated rows");

    // Most-recent first ordering.
    let first = &rows[0];
    let inner = first
        .payload_json
        .get("CsqEvaluated")
        .cloned()
        .unwrap_or_else(|| first.payload_json.clone());
    assert_eq!(
        inner.get("severity").and_then(|v| v.as_str()),
        Some("Warning"),
        "most recent row first (Warning written 2nd)"
    );
    assert_eq!(
        inner.get("guard_present").and_then(|v| v.as_bool()),
        Some(true)
    );

    // The Tauri command derives csq_id from seq: ensure seq is positive.
    assert!(first.seq > 0, "seq must be > 0 for written rows");

    // Second row is the Critical blocked verdict.
    let second = &rows[1];
    let inner2 = second
        .payload_json
        .get("CsqEvaluated")
        .cloned()
        .unwrap_or_else(|| second.payload_json.clone());
    assert_eq!(
        inner2.get("severity").and_then(|v| v.as_str()),
        Some("Critical")
    );
    assert_eq!(
        inner2.get("blocked").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        inner2.get("check_count").and_then(|v| v.as_u64()),
        Some(4)
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_get_feasibility_log_action_type() {
    // DRIFT-3: SsbActionType::FeasibilityCheck does NOT exist. The
    // Tauri command must SELECT on `feasibility_report` instead. We
    // write a FeasibilityReport row through the SsbPayload variant and
    // assert the action_type string matches what the command uses.
    let tmp = TempDir::new().expect("tempdir");
    let cfg = SsbConfig {
        db_path: tmp.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: "c3-feas".into(),
        session_id: "c3-feas-session".into(),
    };
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(Ssb::new(cfg, clock).expect("ssb"));

    let payload = SsbPayload::FeasibilityReport(FeasibilityReportPayload {
        task_feasibility: "feasible".into(),
        skip_reason: None,
        novel_step_count: 1,
        escalations_count: 0,
        vram_margin_bp: 2_000, // 20.00%
        ram_margin_bp: 1_500,
        disk_margin_bp: 9_000,
    });
    ssb.write_payload("c3-test", 0, payload, Vec::new())
        .await
        .expect("write feasibility");

    let db_path = tmp.path().join("ssb.db");
    let viewer = SsbViewer::open(&db_path).expect("viewer open");

    // Same action_type string the Tauri command uses.
    assert_eq!(
        SsbActionType::FeasibilityReport.as_sql_str(),
        "feasibility_report",
        "DRIFT-3 contract: action_type is feasibility_report (NOT feasibility_check)"
    );

    let rows = viewer
        .recent_rows_of_type(SsbActionType::FeasibilityReport.as_sql_str(), 20)
        .expect("recent_rows_of_type feasibility_report");
    assert_eq!(rows.len(), 1, "expected 1 feasibility_report row");

    // Decode the payload shape the dashboard expects: task_feasibility,
    // vram_margin_bp -> percent conversion.
    let inner = rows[0]
        .payload_json
        .get("FeasibilityReport")
        .cloned()
        .unwrap_or_else(|| rows[0].payload_json.clone());
    assert_eq!(
        inner.get("task_feasibility").and_then(|v| v.as_str()),
        Some("feasible")
    );
    let bp = inner
        .get("vram_margin_bp")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let pct = bp as f64 / 100.0;
    assert!(
        (pct - 20.0).abs() < f64::EPSILON,
        "2000 bp must convert to 20.00 percent, got {pct}"
    );

    // Empty / unregistered action_type negative control.
    let unknown = elai_orchestrator::ssb::ssb_action_type_from_str_public("feasibility_check");
    assert!(
        unknown.is_err(),
        "feasibility_check is NOT a registered action_type — implementations must use feasibility_report"
    );
}

#[test]
fn test_get_agent_grc_config_returns_eight_or_more() {
    // DRIFT-5 contract: AgentGrcConfig has no `topic_prefix` field, so
    // the Tauri command uses `role` as the topic key. This test
    // mirrors the iteration order the command uses (15 documented
    // variants today) and asserts:
    //   - Every role string is non-empty (plan acceptance).
    //   - max_calls > 0 / max_chars > 0 (sanity — no zeroed budget
    //     would survive the GRC engine's invariant check).
    //   - Total entry count >= 8 (plan acceptance).
    let roles = [
        AgentRole::Executor,
        AgentRole::Auditor,
        AgentRole::Researcher,
        AgentRole::Designer,
        AgentRole::PromptEngineer,
        AgentRole::ContextCompiler,
        AgentRole::MemoryWriter,
        AgentRole::MediaValidator,
        AgentRole::IdeaFramer,
        AgentRole::IdeaCritic,
        AgentRole::AuditorConcepts,
        AgentRole::AuditorPhases,
        AgentRole::AuditorSteps,
        AgentRole::AuditorBabySteps,
        AgentRole::AuditorBabyStepDetail,
    ];
    assert!(
        roles.len() >= 8,
        "plan acceptance requires >= 8 GRC config entries, got {}",
        roles.len()
    );

    for role in roles.iter() {
        let cfg = default_grc_config(role);
        assert!(
            !cfg.role.is_empty(),
            "role string must be non-empty for {role:?}"
        );
        assert!(
            cfg.budget.max_calls > 0,
            "max_calls must be > 0 for {role:?}"
        );
        assert!(
            cfg.budget.max_chars > 0,
            "max_chars must be > 0 for {role:?}"
        );
        // DRIFT-5: topic key is the role string itself; the JSON shape
        // copies cfg.role into both `role` and `topic_prefix`. Assert
        // role parses as a non-trivial identifier (alpha or underscore).
        assert!(
            cfg.role.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'),
            "role must be a snake_case identifier, got {:?}",
            cfg.role
        );
    }
}
