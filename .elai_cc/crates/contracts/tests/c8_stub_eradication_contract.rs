//! C-8 contract test — verifies the viewer stub eradication wiring landed by
//! C-8 (Viewer Stub Eradication: Remaining Producers).
//!
//! Source plan: docs/thefinalglance/C-plan/C-8_viewer_stub_eradication.md
//! Decisions:  workflow/reasoning/C-8-decisions.md
//!
//! Tests are intentionally narrow (one assertion per concern):
//!   (1) SCHEMA_VERSION_PUBLIC reports >= 50 (atomic +1 over C-7's 49).
//!   (2) `SsbActionType::SearchModeChanged` serialises to
//!       `"search_mode_changed"`.
//!   (3) `ssb_action_type_from_str_public("search_mode_changed")` round-trips.
//!   (4) `SearchModeChangedPayload` round-trips through the SQLite SSB writer
//!       and SsbViewer detail query for every valid mode tag.
//!   (5) `ArtifactsIndexedPayload` round-trips end-to-end (B-34 variant
//!       wired by C-8 into production_agent_rpc.rs TOOL_INDEX_ARTIFACTS;
//!       the contract here pins the wire shape so the agent-side decode
//!       stays stable).
//!   (6) `SearchModeChangedPayload` history vector preserves order across
//!       serialize / deserialize so the dashboard timeline renders
//!       deterministically.

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::ssb::{
    ArtifactsIndexedPayload, SearchModeChangedPayload, Ssb, SsbActionType, SsbConfig, SsbPayload,
    SCHEMA_VERSION_PUBLIC,
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
        project_id: "c8-contract".into(),
        session_id: "c8-test".into(),
    };
    Ssb::new(cfg, Arc::new(OsClock)).expect("ssb init")
}

fn open_viewer(path: &std::path::Path) -> SsbViewer {
    SsbViewer::open(path).expect("viewer open")
}

// ---------------------------------------------------------------------------
// (1) SCHEMA_VERSION bump 49 -> 50.
// ---------------------------------------------------------------------------

#[test]
fn schema_version_advances_to_50_after_c8() {
    // C-8 ADDED 1 variant (SearchModeChanged) in one atomic bump. Floor
    // assertion uses `>=` so future plans can keep bumping without
    // breaking this contract.
    assert!(
        SCHEMA_VERSION_PUBLIC >= 50,
        "SCHEMA_VERSION_PUBLIC must be at least 50 after C-8; got {}",
        SCHEMA_VERSION_PUBLIC
    );
}

// ---------------------------------------------------------------------------
// (2) Action-type string serialization.
// ---------------------------------------------------------------------------

#[test]
fn search_mode_changed_action_type_serialises_to_snake_case() {
    assert_eq!(
        SsbActionType::SearchModeChanged.as_sql_str(),
        "search_mode_changed"
    );
}

// ---------------------------------------------------------------------------
// (3) String -> enum reverse mapping round-trips.
// ---------------------------------------------------------------------------

#[test]
fn search_mode_changed_string_round_trips_via_public_parser() {
    let parsed = ssb_action_type_from_str_public("search_mode_changed")
        .expect("search_mode_changed must parse");
    assert_eq!(parsed, SsbActionType::SearchModeChanged);
}

// ---------------------------------------------------------------------------
// (4) SearchModeChanged round-trip via the SQLite writer + viewer.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn search_mode_changed_round_trips_through_db() {
    let (_d, path) = fresh_db();
    let writer = open_writer(&path).await;
    for mode in ["hybrid", "lexical", "semantic", "ann"] {
        let payload = SearchModeChangedPayload {
            mode: mode.to_string(),
            history: vec!["hybrid".to_string(), mode.to_string()],
        };
        let seq = writer
            .write_payload(
                "live_cli",
                0,
                SsbPayload::SearchModeChanged(payload.clone()),
                Vec::new(),
            )
            .await
            .unwrap_or_else(|e| panic!("write search_mode_changed mode={mode} failed: {e}"));
        assert!(seq > 0, "seq must be positive for mode={mode}");
    }
    drop(writer);

    let viewer = open_viewer(&path);
    let latest = viewer
        .latest_action_of_type("search_mode_changed")
        .expect("viewer query")
        .expect("at least one row");
    assert_eq!(latest.action_type, "search_mode_changed");
    assert_eq!(latest.payload_json["SearchModeChanged"]["mode"], "ann");
    assert!(
        latest.payload_json["SearchModeChanged"]["history"].is_array(),
        "history must serialize as an array"
    );
}

// ---------------------------------------------------------------------------
// (5) ArtifactsIndexed round-trip (B-34 variant; C-8 wires it).
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn artifacts_indexed_payload_round_trips_through_db() {
    let (_d, path) = fresh_db();
    let writer = open_writer(&path).await;
    let payload = ArtifactsIndexedPayload {
        project_id: "c8-contract".into(),
        indexed: vec!["openapi".into(), "k8s_manifest".into()],
        up_to_date: vec!["db_schema".into()],
        total_chunks: 42,
        latency_ms: 17,
    };
    let seq = writer
        .write_payload(
            "agent_rpc",
            0,
            SsbPayload::ArtifactsIndexed(payload.clone()),
            Vec::new(),
        )
        .await
        .expect("write artifacts_indexed");
    drop(writer);

    let viewer = open_viewer(&path);
    let detail = viewer.get_action_detail(seq).expect("detail");
    assert_eq!(detail.action_type, "artifacts_indexed");
    let json = &detail.payload_json["ArtifactsIndexed"];
    assert_eq!(json["project_id"], "c8-contract");
    assert_eq!(json["indexed"].as_array().map(|a| a.len()), Some(2));
    assert_eq!(json["up_to_date"].as_array().map(|a| a.len()), Some(1));
    assert_eq!(json["total_chunks"], 42);
    assert_eq!(json["latency_ms"], 17);
}

// ---------------------------------------------------------------------------
// (6) history vector preserves order across the round-trip.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn search_mode_changed_history_preserves_order() {
    let (_d, path) = fresh_db();
    let writer = open_writer(&path).await;
    let payload = SearchModeChangedPayload {
        mode: "lexical".into(),
        history: vec![
            "hybrid".into(),
            "semantic".into(),
            "ann".into(),
            "lexical".into(),
        ],
    };
    let seq = writer
        .write_payload(
            "live_cli",
            0,
            SsbPayload::SearchModeChanged(payload.clone()),
            Vec::new(),
        )
        .await
        .expect("write search_mode_changed");
    drop(writer);

    let viewer = open_viewer(&path);
    let detail = viewer.get_action_detail(seq).expect("detail");
    let history = detail.payload_json["SearchModeChanged"]["history"]
        .as_array()
        .expect("history array");
    assert_eq!(history.len(), 4);
    assert_eq!(history[0], "hybrid");
    assert_eq!(history[1], "semantic");
    assert_eq!(history[2], "ann");
    assert_eq!(history[3], "lexical");
}
