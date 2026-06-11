//! C-2 contract test (Layer-4) — cosmos artifact SSB emission.
//!
//! Source plan: docs/thefinalglance/C-plan/C-2_ssb_emission_orchestration.md
//! Wiring doctrine: .claude/rules/wiring-enforcement.md Layer-4 — drive the
//! REAL production cosmos SSB emitter (`SsbEmitterAdapter`, the `Arc<Ssb>`
//! bridge wired into `CosmosWorker::with_ssb_emitter` at
//! `cli/src/knowledge_stack.rs`) and assert the off-agent row lands in a real
//! SQLite Ssb. Missing evidence = FAIL: each test asserts on the bytes read
//! back from the DB, NOT on a hand-built payload.
//!
//! `SsbEmitterAdapter` implements `elai_knowledge::cosmos_worker::SsbEmitter`;
//! its `emit_refresh_started` / `emit_artifact_updated` / `emit_artifact_decayed`
//! methods are the exact code paths `CosmosWorker::tick()` calls in production.

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_knowledge::cosmos_worker::SsbEmitter;
use elai_orchestrator::ssb::{Ssb, SsbActionType, SsbConfig, SsbPayload, SsbReader};
use elai_orchestrator::SsbEmitterAdapter;
use tempfile::TempDir;

fn fresh_db() -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("ssb.db");
    (dir, path)
}

fn open_writer(path: &std::path::Path) -> Arc<Ssb> {
    let cfg = SsbConfig {
        db_path: path.to_path_buf(),
        pool_size: Some(2),
        project_id: "c2-cosmos".into(),
        session_id: "c2-cosmos-test".into(),
    };
    Arc::new(Ssb::new(cfg, Arc::new(OsClock)).expect("ssb init"))
}

// ---------------------------------------------------------------------------
// (1) The production adapter's emit_artifact_updated lands a real row carrying
//     the real blake3 / body_bytes / latency_ms (not placeholder zeros).
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cosmos_artifact_updated_emit_lands_real_row() {
    let (_dir, path) = fresh_db();
    let ssb = open_writer(&path);
    let adapter = SsbEmitterAdapter::new(Arc::clone(&ssb)).expect("adapter");

    adapter
        .emit_artifact_updated("rust-2026-posture", "abc123def456", 4096, 87)
        .await;

    let rows = ssb
        .query_by_type(SsbActionType::CosmosArtifactUpdated)
        .expect("query_by_type");
    assert_eq!(
        rows.len(),
        1,
        "exactly one CosmosArtifactUpdated row must land from the real adapter \
         emit (Layer-4 FAIL if zero: producer is wire-dark)"
    );
    let payload: SsbPayload =
        rmp_serde::from_slice(&rows[0].payload).expect("deserialize CosmosArtifactUpdated");
    match payload {
        SsbPayload::CosmosArtifactUpdated(p) => {
            assert_eq!(p.name, "rust-2026-posture");
            assert_eq!(p.blake3, "abc123def456");
            assert_eq!(p.body_bytes, 4096);
            assert_eq!(p.latency_ms, 87);
        }
        other => panic!("expected CosmosArtifactUpdated, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// (2) emit_refresh_started lands a CosmosRefreshStarted row with real counts.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cosmos_refresh_started_emit_lands_real_row() {
    let (_dir, path) = fresh_db();
    let ssb = open_writer(&path);
    let adapter = SsbEmitterAdapter::new(Arc::clone(&ssb)).expect("adapter");

    adapter.emit_refresh_started(3, 11).await;

    let rows = ssb
        .query_by_type(SsbActionType::CosmosRefreshStarted)
        .expect("query_by_type");
    assert_eq!(rows.len(), 1, "exactly one CosmosRefreshStarted row expected");
    match rmp_serde::from_slice::<SsbPayload>(&rows[0].payload).expect("deserialize") {
        SsbPayload::CosmosRefreshStarted(p) => {
            assert_eq!(p.queued, 3);
            assert_eq!(p.total, 11);
        }
        other => panic!("expected CosmosRefreshStarted, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// (3) emit_artifact_decayed lands a CosmosArtifactDecayed row (serve-time warn
//     channel's async twin) carrying the age + cadence.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cosmos_artifact_decayed_emit_lands_real_row() {
    let (_dir, path) = fresh_db();
    let ssb = open_writer(&path);
    let adapter = SsbEmitterAdapter::new(Arc::clone(&ssb)).expect("adapter");

    adapter
        .emit_artifact_decayed("eu-ai-act-timeline", 90_000, "weekly")
        .await;

    let rows = ssb
        .query_by_type(SsbActionType::CosmosArtifactDecayed)
        .expect("query_by_type");
    assert_eq!(rows.len(), 1, "exactly one CosmosArtifactDecayed row expected");
    match rmp_serde::from_slice::<SsbPayload>(&rows[0].payload).expect("deserialize") {
        SsbPayload::CosmosArtifactDecayed(p) => {
            assert_eq!(p.name, "eu-ai-act-timeline");
            assert_eq!(p.age_secs, 90_000);
            assert_eq!(p.cadence, "weekly");
        }
        other => panic!("expected CosmosArtifactDecayed, got {other:?}"),
    }
}
