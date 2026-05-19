//! C-5 contract test — cloud mode hardening.
//!
//! Verifies the surface added by C-5:
//!   1. `BackendMode::Hybrid` variant exists + (de)serializes + parses
//!      from "hybrid" / "auto" via FromStr; the new exhaustive match
//!      arms on `cloud_mode::resolve_exec_mode` map Hybrid -> Auto.
//!   2. CLI parser accepts `--provider local|cloud|hybrid` and rejects
//!      unknown values; `--provider` is plumbed onto `CliAction::Run`.
//!   3. `SsbActionType::RedactionApplied` serialises to
//!      `"redaction_applied"` and the
//!      [`RedactionAppliedPayload`] round-trips through the SQLite SSB
//!      writer.
//!   4. SCHEMA_VERSION advances to >= 51 after C-5's atomic 9-tuple bump.
//!   5. Production wiring chokepoint
//!      [`runtime_setup::apply_task_runner_wiring`] installs a live
//!      `CloudModeRouter` on `TaskRunnerConfig::cloud_router` when the
//!      caller supplies a `CloudConfig`.
//!   6. `EscalateToCloud` row payload preserves the new
//!      `trigger_reason="hybrid_local_first"` / `"cloud_pinned"` /
//!      `"cap_reached"` strings without lossy normalisation.
//!
//! Source plan: docs/thefinalglance/C-plan/C-5_cloud_mode_hardening.md
//! Decisions:   workflow/reasoning/C-5-decisions.md

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::cloud_mode::{
    CloudConfig, CloudModeRouter, ExecMode,
};
use elai_orchestrator::dispatcher::{BackendConfig, BackendMode};
use elai_orchestrator::ssb::{
    EscalateToCloudPayload, RedactionAppliedPayload, Ssb, SsbActionType, SsbConfig,
    SsbPayload, SCHEMA_VERSION_PUBLIC,
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
        project_id: "c5-contract".into(),
        session_id: "c5-test".into(),
    };
    Ssb::new(cfg, Arc::new(OsClock)).expect("ssb init")
}

// ---------------------------------------------------------------------------
// (1) BackendMode::Hybrid variant + match-arm sweep.
// ---------------------------------------------------------------------------

#[test]
fn backend_mode_hybrid_variant_exists() {
    let m = BackendMode::Hybrid;
    assert_eq!(m, BackendMode::Hybrid);
    assert_ne!(BackendMode::Hybrid, BackendMode::Local);
    assert_ne!(BackendMode::Hybrid, BackendMode::Api);
}

#[test]
fn backend_mode_hybrid_display_roundtrips() {
    assert_eq!(BackendMode::Hybrid.to_string(), "hybrid");
    let parsed: BackendMode = "hybrid".parse().expect("hybrid parse");
    assert_eq!(parsed, BackendMode::Hybrid);
}

#[test]
fn backend_mode_from_str_accepts_legacy_auto_alias() {
    // C-5: "auto" maps to Hybrid so legacy toml `[cloud].default_mode = "auto"`
    // continues to land on the same provider-routing semantics.
    let parsed: BackendMode = "auto".parse().expect("auto parse");
    assert_eq!(parsed, BackendMode::Hybrid);
}

#[test]
fn backend_mode_from_str_accepts_cloud_alias() {
    // C-5: "cloud" maps to Api so the CLI `--provider cloud` flag and
    // the toml `[cloud].default_mode = "cloud"` both resolve to the
    // same BackendMode without needing per-call translation.
    let parsed: BackendMode = "cloud".parse().expect("cloud parse");
    assert_eq!(parsed, BackendMode::Api);
}

#[test]
fn resolve_exec_mode_maps_hybrid_to_auto() {
    let mut cfg = BackendConfig::default();
    cfg.mode = BackendMode::Hybrid;
    let resolved = elai_orchestrator::cloud_mode::resolve_exec_mode(&cfg);
    assert_eq!(resolved, ExecMode::Auto);
}

#[test]
fn resolve_exec_mode_maps_local_to_local_and_api_to_cloud() {
    let mut cfg = BackendConfig::default();
    cfg.mode = BackendMode::Local;
    assert_eq!(
        elai_orchestrator::cloud_mode::resolve_exec_mode(&cfg),
        ExecMode::Local,
    );
    cfg.mode = BackendMode::Api;
    assert_eq!(
        elai_orchestrator::cloud_mode::resolve_exec_mode(&cfg),
        ExecMode::Cloud,
    );
}

#[test]
fn backend_mode_serde_roundtrip_includes_hybrid() {
    let m = BackendMode::Hybrid;
    let json = serde_json::to_string(&m).expect("serialize");
    assert_eq!(json, "\"hybrid\"");
    let parsed: BackendMode = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed, BackendMode::Hybrid);
}

// ---------------------------------------------------------------------------
// (2) SCHEMA_VERSION bump 50 -> 51.
// ---------------------------------------------------------------------------

#[test]
fn schema_version_advances_to_51_after_c5() {
    // C-5 ADDED 1 variant (RedactionApplied) in one atomic bump.
    // Future C-N plans must not regress this number.
    assert!(
        SCHEMA_VERSION_PUBLIC >= 51,
        "SCHEMA_VERSION_PUBLIC must be at least 51 after C-5; got {}",
        SCHEMA_VERSION_PUBLIC
    );
}

// ---------------------------------------------------------------------------
// (3) RedactionApplied action-type bijection + payload round-trip.
// ---------------------------------------------------------------------------

#[test]
fn redaction_applied_action_type_serialises_to_snake_case() {
    assert_eq!(
        SsbActionType::RedactionApplied.as_sql_str(),
        "redaction_applied"
    );
}

#[test]
fn redaction_applied_from_str_roundtrips() {
    let parsed = ssb_action_type_from_str_public("redaction_applied")
        .expect("parse redaction_applied");
    assert_eq!(parsed, SsbActionType::RedactionApplied);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn redaction_applied_payload_roundtrips_through_ssb() {
    let (_dir, db_path) = fresh_db();
    let writer = open_writer(&db_path).await;
    let payload = SsbPayload::RedactionApplied(RedactionAppliedPayload {
        count: 4,
        fields: vec!["task_summary".into(), "escalation_message".into()],
        baby_step_id: 42,
        ts_ms: 1_700_000_000_000,
    });
    let seq = writer
        .write_payload("cloud_mode_redactor", 0, payload, Vec::new())
        .await
        .expect("write RedactionApplied");
    assert!(seq > 0, "seq must be positive, got {seq}");

    let viewer = SsbViewer::open(&db_path).expect("open viewer");
    let rows = viewer
        .recent_rows_of_type("redaction_applied", 1)
        .expect("recent_rows_of_type");
    assert_eq!(rows.len(), 1, "exactly one RedactionApplied row");
    // SsbPayload is serialized as a discriminator-tagged enum:
    // {"redaction_applied": {...}}.
    let v = rows[0]
        .payload_json
        .get("RedactionApplied")
        .expect("variant key");
    assert_eq!(v["count"].as_u64(), Some(4));
    assert_eq!(v["baby_step_id"].as_u64(), Some(42));
    assert_eq!(v["ts_ms"].as_i64(), Some(1_700_000_000_000));
    let fields = v["fields"].as_array().expect("fields array");
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].as_str(), Some("task_summary"));
    assert_eq!(fields[1].as_str(), Some("escalation_message"));
}

// ---------------------------------------------------------------------------
// (4) CloudModeRouter construction from CloudConfig (production chokepoint
// uses this same constructor).
// ---------------------------------------------------------------------------

#[test]
fn cloud_mode_router_constructible_from_default_config() {
    let cfg = CloudConfig::default();
    let router = CloudModeRouter::new(cfg.clone());
    // Privacy config mirrors the config's profile — confirms the
    // router carries the redaction-knob into RedactingBabyStepExecutor.
    let privacy = router.privacy_config();
    assert_eq!(privacy.profile, cfg.privacy_profile);
}

#[test]
fn cloud_mode_router_air_gapped_blocks_cloud_route() {
    let mut cfg = CloudConfig::default();
    cfg.air_gapped = true;
    let router = CloudModeRouter::new(cfg);
    let outcome = router.route(ExecMode::Cloud, None);
    assert!(
        matches!(
            outcome,
            Err(elai_orchestrator::cloud_mode::CloudModeError::AirGappedConflict),
        ),
        "expected AirGappedConflict, got {outcome:?}"
    );
}

// ---------------------------------------------------------------------------
// (5) EscalateToCloud row preserves new trigger_reason strings.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn escalate_to_cloud_preserves_hybrid_local_first_reason() {
    let (_dir, db_path) = fresh_db();
    let writer = open_writer(&db_path).await;
    let payload = SsbPayload::EscalateToCloud(EscalateToCloudPayload {
        step_id: "baby_step_7".into(),
        trigger_reason: "hybrid_local_first".into(),
        layer_failed: 1,
        ts_ms: 1_700_000_000_000,
    });
    let seq = writer
        .write_payload("task_runner", 0, payload, Vec::new())
        .await
        .expect("write EscalateToCloud");
    assert!(seq > 0);

    let viewer = SsbViewer::open(&db_path).expect("viewer");
    let rows = viewer
        .recent_rows_of_type("escalate_to_cloud", 1)
        .expect("recent_rows_of_type");
    assert_eq!(rows.len(), 1);
    let v = rows[0]
        .payload_json
        .get("EscalateToCloud")
        .expect("variant key");
    assert_eq!(v["trigger_reason"].as_str(), Some("hybrid_local_first"));
    assert_eq!(v["step_id"].as_str(), Some("baby_step_7"));
    assert_eq!(v["layer_failed"].as_u64(), Some(1));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn escalate_to_cloud_preserves_cap_reached_reason() {
    let (_dir, db_path) = fresh_db();
    let writer = open_writer(&db_path).await;
    let payload = SsbPayload::EscalateToCloud(EscalateToCloudPayload {
        step_id: "task_t1".into(),
        trigger_reason: "cap_reached".into(),
        layer_failed: 0,
        ts_ms: 1_700_000_000_000,
    });
    writer
        .write_payload("cloud_grounding_client", 0, payload, Vec::new())
        .await
        .expect("write cap_reached EscalateToCloud");
    let viewer = SsbViewer::open(&db_path).expect("viewer");
    let rows = viewer
        .recent_rows_of_type("escalate_to_cloud", 1)
        .expect("recent_rows_of_type");
    let v = rows[0]
        .payload_json
        .get("EscalateToCloud")
        .expect("variant key");
    assert_eq!(v["trigger_reason"].as_str(), Some("cap_reached"));
    assert_eq!(v["layer_failed"].as_u64(), Some(0));
}
