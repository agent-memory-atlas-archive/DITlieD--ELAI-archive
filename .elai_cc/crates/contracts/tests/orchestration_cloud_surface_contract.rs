//! C-4 contract test — Tauri orchestration + cloud-spend surface.
//!
//! Source plan: docs/thefinalglance/C-plan/C-4_tauri_commands_orchestration_cloud.md
//! Decisions: workflow/reasoning/C-4-decisions.md
//!
//! Scope: every new SsbViewer method introduced by C-4 (`cloud_spend_summary`
//! + the rewritten `baby_step_detail`) and the graceful-empty paths each
//! Tauri command must hit when the SSB is fresh. The Tauri-command bodies
//! themselves are exercised indirectly: each command is a thin wrapper that
//! either forwards to the viewer (covered here) or hits filesystem
//! conventions (projects.json — covered via end-to-end inside the
//! dashboard crate's own test surface).
//!
//! Tests (one assertion per concern):
//!   (1) cloud_spend_summary returns zero CloudSpendSummary on empty SSB.
//!   (2) cloud_spend_summary aggregates today's rows and reports the
//!       latest row's cost_usd as `task_usd`.
//!   (3) baby_step_detail returns NotFound on a fresh SSB (no
//!       BabyStepReady rows).
//!   (4) baby_step_detail returns a partial-populated detail when a
//!       matching BabyStepReady row exists (task = payload.label;
//!       concept/phase/step names empty when decomposition tree absent).
//!   (5) phase_detail returns NotFound on a fresh SSB.
//!   (6) phase_progress / recent_actions / file_changes / decomposition_tree
//!       all return empty Vec / empty snapshot on a fresh SSB (no panic).

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_core::ClockProvider;
use elai_orchestrator::ssb::{
    BabyStepReadyPayload, CloudSpendPayload, Ssb, SsbConfig, SsbPayload, SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::{CloudSpendSummary, SsbViewer, SsbViewerError};
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
        project_id: "c4-contract".into(),
        session_id: "c4-test".into(),
    };
    Ssb::new(cfg, Arc::new(OsClock)).expect("ssb init")
}

fn open_viewer(path: &std::path::Path) -> SsbViewer {
    SsbViewer::open(path).expect("viewer open")
}

// SCHEMA_VERSION sanity — C-4 does not add new SSB variants, but we want
// to fail fast if a future merge accidentally regresses below the C-2
// floor (47) the contract test set up assumes.
#[test]
fn schema_version_at_or_above_c2_floor() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 47,
        "C-4 contract expects SCHEMA_VERSION >= 47 (C-2 floor); got {}",
        SCHEMA_VERSION_PUBLIC
    );
}

// ---------------------------------------------------------------------------
// (1) cloud_spend_summary — empty SSB.
// ---------------------------------------------------------------------------

#[test]
fn cloud_spend_summary_returns_zero_on_empty_ssb() {
    let (_d, path) = fresh_db();
    // Open a writer (and immediately drop it) to materialise the schema
    // — SsbViewer::open against a non-existent file would error.
    let rt = tokio::runtime::Runtime::new().expect("rt");
    rt.block_on(async {
        let _w = open_writer(&path).await;
    });
    let viewer = open_viewer(&path);
    let summary = viewer
        .cloud_spend_summary(1_700_000_000_000)
        .expect("summary");
    assert_eq!(summary, CloudSpendSummary::default());
    assert_eq!(summary.task_usd, 0.0);
    assert_eq!(summary.day_usd, 0.0);
    assert_eq!(summary.cache_creation_tokens, 0);
    assert_eq!(summary.cache_read_tokens, 0);
    assert_eq!(summary.redaction_count, 0);
}

// ---------------------------------------------------------------------------
// (2) cloud_spend_summary aggregates today's rows; task_usd = latest row.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cloud_spend_summary_aggregates_today_rows() {
    let (_d, path) = fresh_db();
    let writer = open_writer(&path).await;
    let p1 = CloudSpendPayload {
        provider: "anthropic".into(),
        model: "claude-sonnet-4-6".into(),
        input_tokens: 100,
        output_tokens: 50,
        cache_creation_tokens: 25,
        cache_read_tokens: 10,
        cost_usd: 0.10,
        task_id: "task-1".into(),
        ts_ms: 0,
    };
    let p2 = CloudSpendPayload {
        cost_usd: 0.25,
        cache_creation_tokens: 50,
        cache_read_tokens: 5,
        ..p1.clone()
    };
    writer
        .write_payload("task_runner", 0, SsbPayload::CloudSpend(p1.clone()), vec![])
        .await
        .expect("write 1");
    writer
        .write_payload("task_runner", 0, SsbPayload::CloudSpend(p2.clone()), vec![])
        .await
        .expect("write 2");
    drop(writer);

    let viewer = open_viewer(&path);
    // Use a `now_ms` far in the future so both rows fall inside "today".
    let now = i64::MAX / 2;
    let summary = viewer.cloud_spend_summary(now).expect("summary");
    // task_usd is the latest row (descending seq -> p2 first).
    assert!(
        (summary.task_usd - 0.25).abs() < 1e-9,
        "task_usd should equal latest cost_usd=0.25; got {}",
        summary.task_usd
    );
    // With `now` so far ahead, day_start is also in the future — no row
    // matches the "today" window. Verify that path first.
    assert_eq!(summary.day_usd, 0.0, "day_usd with future `now` is 0");

    // Repeat with a `now` close to the SSB's actual created_at — the
    // OsClock wrote real timestamps, so passing now=u64::MAX / 2 - day
    // boundaries makes both rows match. Use the writer's clock instead.
    let now_ok = elai_core::clock::OsClock
        .system_now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    let summary_ok = viewer.cloud_spend_summary(now_ok).expect("summary now");
    assert!(
        (summary_ok.day_usd - 0.35).abs() < 1e-9,
        "day_usd should sum both rows (0.10 + 0.25); got {}",
        summary_ok.day_usd
    );
    assert_eq!(summary_ok.cache_creation_tokens, 75);
    assert_eq!(summary_ok.cache_read_tokens, 15);
    assert_eq!(summary_ok.redaction_count, 0);
}

// ---------------------------------------------------------------------------
// (3) baby_step_detail — NotFound on empty SSB.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn baby_step_detail_returns_not_found_on_empty_ssb() {
    let (_d, path) = fresh_db();
    let _w = open_writer(&path).await;
    drop(_w);
    let viewer = open_viewer(&path);
    let err = viewer.baby_step_detail("42").expect_err("must error");
    match err {
        SsbViewerError::NotFound { kind, id } => {
            assert_eq!(kind, "baby_step");
            assert_eq!(id, "42");
        }
        other => panic!("expected NotFound, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// (4) baby_step_detail returns partial detail when a matching
//     BabyStepReady row exists.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn baby_step_detail_partial_populates_from_baby_step_ready() {
    let (_d, path) = fresh_db();
    let writer = open_writer(&path).await;
    let payload = BabyStepReadyPayload {
        baby_step_id: 42,
        step_id: 12,
        phase_id: 7,
        concept_id: 3,
        label: "implement todo add fn".into(),
        depth: 3,
    };
    writer
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
    let detail = viewer.baby_step_detail("42").expect("detail present");
    assert_eq!(detail.baby_step_id, "42");
    assert_eq!(detail.task, "implement todo add fn");
    assert_eq!(detail.estimated_tokens, 0);
    // concept/phase/step names default to empty when no decomposition
    // tree is persisted — this is the documented partial-populate path.
    assert_eq!(detail.concept_name, "");
    assert_eq!(detail.phase_name, "");
    assert_eq!(detail.step_name, "");
    assert!(detail.tdd.is_none());
    assert!(detail.acceptance.is_none());
}

// ---------------------------------------------------------------------------
// (5) phase_detail — NotFound on empty SSB.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn phase_detail_returns_not_found_on_empty_ssb() {
    let (_d, path) = fresh_db();
    let _w = open_writer(&path).await;
    drop(_w);
    let viewer = open_viewer(&path);
    let err = viewer
        .phase_detail("phase-unknown")
        .expect_err("must error");
    match err {
        SsbViewerError::NotFound { kind, id } => {
            assert_eq!(kind, "phase");
            assert_eq!(id, "phase-unknown");
        }
        other => panic!("expected NotFound, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// (6) phase_progress / recent_actions / file_changes / decomposition_tree
//     — empty Vec on empty SSB.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn empty_ssb_returns_empty_views_for_orchestration_surface() {
    let (_d, path) = fresh_db();
    let _w = open_writer(&path).await;
    drop(_w);
    let viewer = open_viewer(&path);

    let progress = viewer.phase_progress().expect("phase_progress");
    assert!(progress.is_empty(), "phase_progress must be empty on fresh SSB");

    let actions = viewer.recent_actions(50, 0).expect("recent_actions");
    assert!(actions.is_empty(), "recent_actions must be empty on fresh SSB");

    let files = viewer.file_changes("c4-test").expect("file_changes");
    assert!(files.is_empty(), "file_changes must be empty on fresh SSB");

    let tree = viewer.decomposition_tree(None).expect("decomposition_tree");
    assert!(
        tree.concepts.is_empty(),
        "decomposition_tree concepts must be empty on fresh SSB"
    );
}

// ---------------------------------------------------------------------------
// (7) get_action_detail — Validation error on unknown seq (graceful-empty
//     surface at the Tauri layer maps this to `{known: false, seq}`).
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_action_detail_returns_validation_error_on_unknown_seq() {
    let (_d, path) = fresh_db();
    let _w = open_writer(&path).await;
    drop(_w);
    let viewer = open_viewer(&path);
    let err = viewer.get_action_detail(9999).expect_err("must error");
    match err {
        SsbViewerError::Validation(msg) => {
            assert!(msg.contains("9999"), "validation message must echo seq: {msg}");
        }
        other => panic!("expected Validation, got {other:?}"),
    }
}
