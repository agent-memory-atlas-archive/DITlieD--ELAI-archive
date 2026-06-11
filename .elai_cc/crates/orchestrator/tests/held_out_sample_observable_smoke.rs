//! D-6 prod-wire — observable_effect smoke test for the held-out task
//! sampler dispatch.
//!
//! This is the OFF-AGENT proof the wiring-enforcement gate requires (R4 +
//! R7): it drives the REAL production dispatch chain
//!
//!   `start_task_runner` (pub entrypoint, called from the dashboard Tauri
//!     setup + `elai run` + `elai task` CLI)
//!   -> runner event loop / `replay_backlog`
//!   -> `drive_task`
//!   -> `maybe_sample_held_out_once`  (the wired hook)
//!   -> `agent_pool::sample_held_out_task` (the producer that was previously
//!      only test-called -- this test gives it a NON-test caller through the
//!      production path)
//!   -> `held_out::mark_used_and_emit` (writes the `HeldOutTaskUsed` row)
//!
//! by seeding a single AGED (already-eligible) held-out task into a
//! tempdir-backed SQLite store, pointing the runner at it via
//! `ELAI_HELD_OUT_DB_PATH`, writing a synthetic `TaskSubmitted` row, and
//! asserting that with `ELAI_HELD_OUT_ENABLE=1` a `HeldOutTaskUsed` SSB row
//! lands (an artifact this test did NOT author), while with the flag OFF (the
//! default) NO such row is produced (default-OFF no-op invariant).
//!
//! The sample path is CPU-only (SQLite read + the in-memory 9-gram signature
//! already stored on each row -- no model / no network), so this test runs
//! fully hermetically.
//!
//! ONE sequential `#[tokio::test]` (single-threaded) holds both the ON and
//! OFF legs because the `ELAI_HELD_OUT_*` env vars are process-global and a
//! multi-threaded split would race them.
//!
//! Run with:
//!   cargo test -p elai-orchestrator --test held_out_sample_observable_smoke

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use elai_orchestrator::held_out::{
    build_task, ExpectedOutcome, HeldOutStore, TaskSource, DEFAULT_NGRAM_SIZE,
};
use elai_orchestrator::ssb::{SsbActionType, SsbPayload, SsbReader, TaskSubmittedPayload};
use elai_orchestrator::{start_task_runner, Ssb, TaskRunnerConfig};

/// Build a fresh tempdir-backed `Arc<Ssb>` + cursor path.
fn fresh_runner_env(session_id: &str) -> (tempfile::TempDir, Arc<Ssb>, PathBuf) {
    use elai_core::clock::{ClockProvider, SeededClock};
    use elai_orchestrator::ssb::SsbConfig;

    let tmp = tempfile::tempdir().expect("tempdir");
    let cursor_path = tmp.path().join("cursor.json");
    let cfg = SsbConfig {
        db_path: tmp.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: "dashboard".into(),
        session_id: session_id.into(),
    };
    let clock: Arc<dyn ClockProvider> = Arc::new(SeededClock::new());
    let ssb = Arc::new(Ssb::new(cfg, clock).expect("ssb init"));
    (tmp, ssb, cursor_path)
}

/// Seed a single ALREADY-ELIGIBLE held-out task into a store at `db_path`.
/// `ingested_ts_ms` is far in the past and the eligibility window is zero, so
/// the task is "active" for any plausible `now_ms` the runner observes.
fn seed_active_task(db_path: &Path) {
    let store = HeldOutStore::open(db_path).expect("open held-out store for seed");
    let task = build_task(
        "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu".to_string(),
        TaskSource::Synthetic { seed: 7 },
        ExpectedOutcome::Code("ok".into()),
        1_000, // far-past ingest ts
        0,     // zero eligibility window => active immediately
        DEFAULT_NGRAM_SIZE,
    );
    store.insert(&task).expect("seed insert");
    assert_eq!(store.count().expect("count"), 1, "seed must land exactly one task");
}

/// Poll the SSB until at least one row of `action` is present, or `timeout`
/// elapses. Returns the row count observed (0 on timeout).
async fn wait_for_action_count(
    writer: &Arc<Ssb>,
    action: SsbActionType,
    timeout: Duration,
) -> usize {
    let start = std::time::Instant::now();
    loop {
        let n = writer.query_by_type(action).unwrap_or_default().len();
        if n > 0 {
            return n;
        }
        if start.elapsed() >= timeout {
            return 0;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

/// Drive one runner lifecycle: seed (when requested) -> spawn runner ->
/// submit one plain task -> wait for the proof row -> count `HeldOutTaskUsed`.
/// Returns `(held_out_rows, drive_task_ran)`.
async fn run_leg(session_id: &str, db_path: &Path, seed: bool) -> (usize, usize) {
    let (tmp, ssb, cursor_path) = fresh_runner_env(session_id);
    if seed {
        seed_active_task(db_path);
    }

    let (handle, cancel) =
        start_task_runner(ssb.clone(), TaskRunnerConfig::new_minimal(cursor_path));
    tokio::time::sleep(Duration::from_millis(100)).await;

    let payload = SsbPayload::TaskSubmitted(TaskSubmittedPayload {
        task: "D-6 smoke: build a hello-world CLI binary".into(),
        project_id: "dashboard".into(),
        submitted_at_ms: 0,
    });
    ssb
        .write_payload("dashboard", 0, payload, Vec::new())
        .await
        .expect("write TaskSubmitted");

    // ReasoningTrace is emitted by drive_task AFTER maybe_sample_held_out_once
    // returns, so its presence proves the hook ran (whether or not it emitted).
    let drive_task_ran =
        wait_for_action_count(&ssb, SsbActionType::ReasoningTrace, Duration::from_secs(15)).await;
    // Give the (possibly-async) HeldOutTaskUsed write a moment to land too.
    let held_out_rows =
        wait_for_action_count(&ssb, SsbActionType::HeldOutTaskUsed, Duration::from_secs(2)).await;

    cancel.cancel();
    let _ = tokio::time::timeout(Duration::from_secs(5), handle).await;
    drop(tmp);
    (held_out_rows, drive_task_ran)
}

#[tokio::test]
async fn held_out_sample_emits_used_row_when_flag_on_and_noop_when_off() {
    // A single shared store db (tempdir) backs both legs; only the ON leg
    // seeds it. The env vars are mutated inline -- this test is single-threaded
    // so there is no in-binary race, and it restores the env before returning.
    let store_tmp = tempfile::tempdir().expect("store tempdir");
    let db_path = store_tmp.path().join("held_out.sqlite");

    let prior_enable = std::env::var("ELAI_HELD_OUT_ENABLE").ok();
    let prior_db = std::env::var("ELAI_HELD_OUT_DB_PATH").ok();

    // -------- Leg 1: flag ON, seeded store -> expect a HeldOutTaskUsed row.
    std::env::set_var("ELAI_HELD_OUT_ENABLE", "1");
    std::env::set_var("ELAI_HELD_OUT_DB_PATH", &db_path);
    let (on_rows, on_drive_ran) = run_leg("d6-sample-on", &db_path, true).await;

    // -------- Leg 2: flag OFF (default), store still seeded -> expect ZERO
    // rows. Pointing the db at the same seeded store proves the GATE (not a
    // missing/empty store) is what suppresses the row.
    std::env::remove_var("ELAI_HELD_OUT_ENABLE");
    std::env::set_var("ELAI_HELD_OUT_DB_PATH", &db_path);
    let (off_rows, off_drive_ran) = run_leg("d6-sample-off", &db_path, false).await;

    // Restore env BEFORE assertions so a panic cannot leak the flag.
    match prior_enable {
        Some(v) => std::env::set_var("ELAI_HELD_OUT_ENABLE", v),
        None => std::env::remove_var("ELAI_HELD_OUT_ENABLE"),
    }
    match prior_db {
        Some(v) => std::env::set_var("ELAI_HELD_OUT_DB_PATH", v),
        None => std::env::remove_var("ELAI_HELD_OUT_DB_PATH"),
    }

    // ON leg: the production drive_task path must have run AND emitted the row.
    assert!(
        on_drive_ran >= 1,
        "ON leg: drive_task must have run (ReasoningTrace emitted) for the assertion to be meaningful"
    );
    assert!(
        on_rows >= 1,
        "ON leg: expected >=1 HeldOutTaskUsed row after writing TaskSubmitted with \
         ELAI_HELD_OUT_ENABLE=1 against a seeded store; the sampler hook never fired through \
         the production drive_task path (wire-dark regression)"
    );

    // OFF leg: default-OFF no-op invariant -- drive_task ran but emitted nothing.
    assert!(
        off_drive_ran >= 1,
        "OFF leg: drive_task must have run so the no-op assertion is meaningful"
    );
    assert_eq!(
        off_rows, 0,
        "default-OFF invariant violated: a HeldOutTaskUsed row was emitted with the enable \
         flag unset (the sampler path must stay wire-dark)"
    );
}
