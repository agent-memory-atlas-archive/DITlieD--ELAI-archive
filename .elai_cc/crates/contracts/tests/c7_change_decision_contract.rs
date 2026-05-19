//! C-7 contract test — verifies the ChangeDecision SSB variant added by C-7
//! and the SCHEMA_VERSION bump 48 -> 49.
//!
//! Source plan: docs/thefinalglance/C-plan/C-7_dashboard_ux_completeness.md
//! Decisions: workflow/reasoning/C-7-decisions.md (DRIFT-2 picked Option A —
//! add accept_change / decline_change / retry_audit Tauri commands which
//! emit a single ChangeDecision SSB variant).
//!
//! Tests are intentionally narrow (one assertion per concern):
//!   (1) SCHEMA_VERSION_PUBLIC reports >= 49.
//!   (2) `SsbActionType::ChangeDecision` serialises to `"change_decision"`.
//!   (3) `ssb_action_type_from_str_public("change_decision")` round-trips.
//!   (4) File-level payload (`hunk_id = None`) round-trips through the
//!       SQLite SSB writer and SsbViewer detail query.
//!   (5) Hunk-level payload (`hunk_id = Some(...)`) round-trips and
//!       preserves the `note` field for "retried" verdicts.
//!   (6) Each of the three decision discriminators (`accepted`, `declined`,
//!       `retried`) round-trips so the dashboard can re-render verdict
//!       state without lossy normalisation.

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::ssb::{
    ChangeDecisionPayload, Ssb, SsbActionType, SsbConfig, SsbPayload, SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::{ssb_action_type_from_str_public, SsbViewer};
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
        project_id: "c7-contract".into(),
        session_id: "c7-test".into(),
    };
    Ssb::new(cfg, Arc::new(OsClock)).expect("ssb init")
}

fn open_viewer(path: &std::path::Path) -> SsbViewer {
    SsbViewer::open(path).expect("viewer open")
}

// ---------------------------------------------------------------------------
// (1) SCHEMA_VERSION bump 48 -> 49.
// ---------------------------------------------------------------------------

#[test]
fn schema_version_advances_to_49_after_c7() {
    // C-7 ADDED 1 variant in one atomic bump. Future C-N plans must not
    // regress this number; B-34 floor pin is still preserved (>= 42).
    assert!(
        SCHEMA_VERSION_PUBLIC >= 49,
        "SCHEMA_VERSION_PUBLIC must be at least 49 after C-7; got {}",
        SCHEMA_VERSION_PUBLIC
    );
}

// ---------------------------------------------------------------------------
// (2) Action-type string serialization.
// ---------------------------------------------------------------------------

#[test]
fn change_decision_action_type_serialises_to_snake_case() {
    assert_eq!(
        SsbActionType::ChangeDecision.as_sql_str(),
        "change_decision"
    );
}

// ---------------------------------------------------------------------------
// (3) String -> enum reverse mapping round-trips.
// ---------------------------------------------------------------------------

#[test]
fn change_decision_string_round_trips_via_public_parser() {
    let parsed = ssb_action_type_from_str_public("change_decision")
        .expect("change_decision must parse");
    assert_eq!(parsed, SsbActionType::ChangeDecision);
}

// ---------------------------------------------------------------------------
// (4) File-level payload round-trip (hunk_id = None).
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn change_decision_file_level_payload_round_trips() {
    let (_d, path) = fresh_db();
    let writer = open_writer(&path).await;
    let payload = ChangeDecisionPayload {
        decision: "accepted".into(),
        path: "crates/cli/src/main.rs".into(),
        hunk_id: None,
        note: None,
    };
    let seq = writer
        .write_payload(
            "dashboard-v2",
            0,
            SsbPayload::ChangeDecision(payload.clone()),
            vec![payload.path.clone()],
        )
        .await
        .expect("write change_decision file-level");
    drop(writer);
    let viewer = open_viewer(&path);
    let detail = viewer.get_action_detail(seq).expect("detail");
    assert_eq!(detail.action_type, "change_decision");
    let json = detail.payload_json;
    assert_eq!(json["ChangeDecision"]["decision"], "accepted");
    assert_eq!(json["ChangeDecision"]["path"], "crates/cli/src/main.rs");
    // hunk_id + note must NOT appear when None (skip_serializing_if).
    assert!(
        json["ChangeDecision"].get("hunk_id").is_none()
            || json["ChangeDecision"]["hunk_id"].is_null(),
        "file-level hunk_id must be absent or null in serialised payload"
    );
}

// ---------------------------------------------------------------------------
// (5) Hunk-level payload round-trip (hunk_id + note set).
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn change_decision_hunk_level_payload_round_trips_with_note() {
    let (_d, path) = fresh_db();
    let writer = open_writer(&path).await;
    let payload = ChangeDecisionPayload {
        decision: "retried".into(),
        path: "crates/orchestrator/src/fsm.rs".into(),
        hunk_id: Some("@@ -38,7 +38,15 @@".into()),
        note: Some("auditor suggested narrower error variant".into()),
    };
    let seq = writer
        .write_payload(
            "dashboard-v2",
            0,
            SsbPayload::ChangeDecision(payload.clone()),
            vec![payload.path.clone()],
        )
        .await
        .expect("write change_decision hunk-level");
    drop(writer);
    let viewer = open_viewer(&path);
    let detail = viewer.get_action_detail(seq).expect("detail");
    assert_eq!(detail.action_type, "change_decision");
    let json = detail.payload_json;
    assert_eq!(json["ChangeDecision"]["decision"], "retried");
    assert_eq!(
        json["ChangeDecision"]["path"],
        "crates/orchestrator/src/fsm.rs"
    );
    assert_eq!(
        json["ChangeDecision"]["hunk_id"],
        "@@ -38,7 +38,15 @@"
    );
    assert_eq!(
        json["ChangeDecision"]["note"],
        "auditor suggested narrower error variant"
    );
}

// ---------------------------------------------------------------------------
// (6) All three decision discriminators round-trip with their exact spelling.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn change_decision_all_three_verdicts_round_trip() {
    let (_d, path) = fresh_db();
    let writer = open_writer(&path).await;
    let verdicts = ["accepted", "declined", "retried"];
    let mut seqs = Vec::new();
    for v in verdicts {
        let payload = ChangeDecisionPayload {
            decision: v.into(),
            path: format!("crates/foo/{v}.rs"),
            hunk_id: None,
            note: None,
        };
        let seq = writer
            .write_payload(
                "dashboard-v2",
                0,
                SsbPayload::ChangeDecision(payload),
                vec![],
            )
            .await
            .unwrap_or_else(|e| panic!("write {v}: {e}"));
        seqs.push((v, seq));
    }
    drop(writer);
    let viewer = open_viewer(&path);
    for (expected_decision, seq) in seqs {
        let detail = viewer.get_action_detail(seq).expect("detail");
        assert_eq!(detail.action_type, "change_decision");
        assert_eq!(
            detail.payload_json["ChangeDecision"]["decision"], expected_decision,
            "decision discriminator must round-trip verbatim"
        );
    }
}
