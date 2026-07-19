//! E-38 on-path smoke -- drives the REAL
//! `emit_posture_doc_stale_if_stale` entrypoint (the same function the
//! three `Fsm::new` sites in `live_cli.rs` call via
//! `maybe_emit_posture_doc_stale`) with
//! `ELAI_AI_ROADMAP_SIGNALS_ENABLE=1` and a staged posture doc whose
//! mtime is forced > 90 days old, then asserts:
//!
//!   * an `AiRoadmapSignalEvent` SSB row with
//!     `kind = PostureDocStale` lands in the SSB (the off-agent
//!     artifact the executor did NOT author -- wiring-enforcement R4);
//!   * a second call is idempotent (the `POSTURE_DOC_STALE_EMITTED`
//!     latch prevents a second row -- exactly one row total).
//!
//! Deterministic only; no GPU / network / live model. Mirrors the
//! `e38_compliance_on_path.rs` SsbConfig + SsbViewer pattern.
//!
//! Runtime-nesting note: `emit_posture_doc_stale_if_stale` builds its
//! own current-thread tokio runtime (mirrors
//! `dispatch::mcp::emit_transport_version_once`). Calling it from
//! inside a `#[tokio::test]` async context panics ("Cannot start a
//! runtime from within a runtime"). The production call sites in
//! `live_cli.rs` are sync code (no outer runtime), so the function is
//! correct as written. This test invokes it through
//! `tokio::task::spawn_blocking` which runs on a blocking-pool thread
//! outside the async runtime context, faithfully mirroring the sync
//! production call site.

use std::sync::Arc;
use std::time::{Duration, SystemTime};

use elai_orchestrator::roadmap_signals::emit_posture_doc_stale_if_stale;
use elai_orchestrator::ssb::{
    RoadmapSignalKind, Ssb, SsbActionType, SsbConfig, SsbPayload, SsbReader,
};

/// Count `AiRoadmapSignalEvent` rows in the SSB with
/// `kind = PostureDocStale`. Uses the read-only `SsbReader` trait (the
/// same path the auditor queries through).
fn count_posture_doc_stale_rows(ssb: &Ssb) -> usize {
    let rows = ssb
        .query_by_type(SsbActionType::AiRoadmapSignalEvent)
        .expect("query_by_type AiRoadmapSignalEvent");
    rows.iter()
        .filter(|row| {
            let payload: SsbPayload = match rmp_serde::from_slice(&row.payload) {
                Ok(p) => p,
                Err(_) => return false,
            };
            matches!(
                payload,
                SsbPayload::AiRoadmapSignalEvent(ref p)
                    if p.kind == RoadmapSignalKind::PostureDocStale
            )
        })
        .count()
}

/// Stage a temp posture doc with mtime forced > 90 days old via
/// `FileTimes::set_modified`. Returns the temp dir (kept alive for the
/// test's lifetime) + the doc path.
fn stage_stale_posture_doc() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let doc_path = dir.path().join("ELAI-2026-POSTURE.md");
    std::fs::write(&doc_path, "# ELAI 2026 Posture\nstub for staleness probe\n")
        .expect("write posture doc");
    // Force mtime to 91 days ago (POSTURE_DOC_STALE_DAYS = 90).
    let stale_mtime = SystemTime::now()
        .checked_sub(Duration::from_secs(91 * 86_400))
        .expect("91 days ago must be representable");
    let times = std::fs::FileTimes::new().set_modified(stale_mtime);
    let file = std::fs::OpenOptions::new()
        .write(true)
        .open(&doc_path)
        .expect("open posture doc for set_times");
    file.set_times(times).expect("set_modified to 91 days ago");
    (dir, doc_path)
}

/// Test 1: with `ELAI_AI_ROADMAP_SIGNALS_ENABLE=1` and a stale posture
/// doc, `emit_posture_doc_stale_if_stale` emits exactly one
/// `PostureDocStale` SSB row. A second call is idempotent (the
/// `POSTURE_DOC_STALE_EMITTED` latch prevents a duplicate).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn roadmap_panel_emits_signal_when_enabled() {
    std::env::set_var("ELAI_AI_ROADMAP_SIGNALS_ENABLE", "1");

    let dir = tempfile::TempDir::new().expect("tempdir for ssb");
    let cfg = SsbConfig {
        db_path: dir.path().join("ssb_roadmap_panel_on_path.db"),
        pool_size: Some(2),
        project_id: "test-project".into(),
        session_id: "s-roadmap-panel".into(),
    };
    let clock: Arc<dyn elai_core::ClockProvider> =
        Arc::new(elai_core::clock::SeededClock::new());
    let ssb = Arc::new(Ssb::new(cfg, clock).expect("ssb init"));

    let (_doc_dir, doc_path) = stage_stale_posture_doc();

    // First call: must emit. Run on a blocking thread to avoid the
    // nested-runtime panic (see module doc comment).
    let ssb_for_emit = Arc::clone(&ssb);
    let path_for_emit = doc_path.clone();
    let emitted = tokio::task::spawn_blocking(move || {
        emit_posture_doc_stale_if_stale(&ssb_for_emit, &path_for_emit)
    })
    .await
    .expect("spawn_blocking emit first call");
    assert!(
        emitted,
        "first call with stale doc + gate ON must emit a row"
    );

    // Give the SSB async write task time to flush the row.
    tokio::time::sleep(Duration::from_millis(80)).await;

    assert_eq!(
        count_posture_doc_stale_rows(&ssb),
        1,
        "exactly one PostureDocStale row after first call"
    );

    // Second call: idempotent (latch already flipped).
    let ssb_for_emit2 = Arc::clone(&ssb);
    let path_for_emit2 = doc_path.clone();
    let emitted2 = tokio::task::spawn_blocking(move || {
        emit_posture_doc_stale_if_stale(&ssb_for_emit2, &path_for_emit2)
    })
    .await
    .expect("spawn_blocking emit second call");
    assert!(
        !emitted2,
        "second call must be idempotent (latch prevents re-emit)"
    );

    tokio::time::sleep(Duration::from_millis(80)).await;

    assert_eq!(
        count_posture_doc_stale_rows(&ssb),
        1,
        "still exactly one PostureDocStale row after second call (idempotent)"
    );

    std::env::remove_var("ELAI_AI_ROADMAP_SIGNALS_ENABLE");
}
