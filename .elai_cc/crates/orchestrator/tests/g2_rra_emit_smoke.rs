//! G-2 -- RunArtifactEmitted SSB round-trip + viewer-read smoke test.
//!
//! Asserts the off-agent beacon the production `elai run` exit writes:
//!   (a) a `run_artifact_emitted` payload round-trips through the LIVE Ssb
//!       writer + the msgpack decode + the SQL CHECK constraint + the
//!       action_type bijection (the 8-site Pattern-2 cascade is wired);
//!   (b) the `SsbViewer::run_artifacts` drawer read (the dashboard
//!       get_run_artifact command's data source) returns the row with the
//!       honest guarantee level + root + gate_count + seed intact;
//!   (c) the action_type wire label is the snake_case `run_artifact_emitted`
//!       and `from_str` round-trips it.
//!
//! No #[ignore] / stubs. No #[traced_test] (we assert on SSB rows, not logs).

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, RunArtifactEmittedPayload, Ssb, SsbActionType, SsbConfig,
    SsbPayload,
};
use elai_orchestrator::ssb_viewer::SsbViewer;
use elai_orchestrator::SsbReader as _;
use tempfile::TempDir;

const SMOKE_SESSION: &str = "g2-smoke-session";

fn make_ssb(tmp: &TempDir) -> Arc<Ssb> {
    let cfg = SsbConfig {
        db_path: tmp.path().join("ssb_g2_smoke.db"),
        pool_size: Some(2),
        project_id: "default".to_string(),
        session_id: SMOKE_SESSION.to_string(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    Arc::new(Ssb::new(cfg, clock).expect("ssb init"))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn run_artifact_emitted_roundtrips_and_reads_via_viewer() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp);

    let payload = SsbPayload::RunArtifactEmitted(RunArtifactEmittedPayload {
        root_blake3: "a1b2c3d4e5f60718".into(),
        guarantee_level: "verify_divergence".into(),
        gate_count: 3,
        seed: 42,
        session_id: SMOKE_SESSION.into(),
        // I-45: objective_hash is None for this LEGACY fixture (no IdeaFramingStarted row).
        objective_hash: None,
    });
    assert_eq!(
        payload.action_type(),
        SsbActionType::RunArtifactEmitted,
        "payload must map to the RunArtifactEmitted action type"
    );

    // Write through the LIVE writer (the same path the run_async exit uses).
    ssb.write_payload("cli:run-rra", 0, payload.clone(), Vec::new())
        .await
        .expect("write run_artifact_emitted row");

    // (a) round-trip via query_by_type + msgpack decode.
    let rows = ssb
        .query_by_type(SsbActionType::RunArtifactEmitted)
        .expect("query run_artifact_emitted");
    assert_eq!(rows.len(), 1, "exactly one run_artifact_emitted row");
    let decoded: SsbPayload = rmp_serde::from_slice(&rows[0].payload).expect("decode");
    match decoded {
        SsbPayload::RunArtifactEmitted(p) => {
            assert_eq!(p.root_blake3, "a1b2c3d4e5f60718");
            assert_eq!(p.guarantee_level, "verify_divergence");
            assert_eq!(p.gate_count, 3);
            assert_eq!(p.seed, 42);
            assert_eq!(p.session_id, SMOKE_SESSION);
        }
        other => panic!("expected RunArtifactEmitted, got {other:?}"),
    }

    // (b) the dashboard read path (SsbViewer::run_artifacts) sees the row.
    let viewer = SsbViewer::open(&tmp.path().join("ssb_g2_smoke.db")).expect("open viewer");
    let drawer_rows = viewer.run_artifacts(50).expect("run_artifacts");
    assert_eq!(drawer_rows.len(), 1, "viewer must surface the sealed artifact");
    let row = &drawer_rows[0];
    assert_eq!(row.root_blake3, "a1b2c3d4e5f60718");
    assert_eq!(row.guarantee_level, "verify_divergence");
    assert_eq!(row.gate_count, 3);
    assert_eq!(row.seed, 42);
    assert_eq!(row.session_id, SMOKE_SESSION);
}

// ---------------------------------------------------------------------------
// (c) action_type wire-label bijection
// ---------------------------------------------------------------------------

#[test]
fn run_artifact_emitted_sql_str_bijection() {
    let variant = SsbActionType::RunArtifactEmitted;
    let s = variant.as_sql_str();
    assert_eq!(s, "run_artifact_emitted");
    let round = ssb_action_type_from_str_public(s).expect("parse");
    assert_eq!(round, variant);
}
