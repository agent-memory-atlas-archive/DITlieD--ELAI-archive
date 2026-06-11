//! D-30 prod-wire — observable_effect smoke test for the calibration
//! bootstrap warm-start.
//!
//! This is the OFF-AGENT proof the wiring-enforcement gate requires (R4 +
//! R7): it drives the REAL production dispatch chain
//!
//!   `start_task_runner` (pub entrypoint, called from the dashboard Tauri
//!     setup + `elai run` + `elai task` CLI)
//!   -> runner event loop / `replay_backlog`
//!   -> `handle_task_entry`
//!   -> `drive_task`
//!   -> `maybe_warm_start_calibration_once`  (the wired hook)
//!
//! by writing a synthetic `TaskSubmitted` row, and asserts that with
//! `ELAI_CALIBRATION_BOOTSTRAP_ENABLE=1` a `CalibrationBootstrapCompleted`
//! SSB row lands (an artifact this test did NOT author), while with the flag
//! OFF (the default) NO such row is produced (default-OFF no-op invariant).
//!
//! Run with:
//!   cargo test -p elai-orchestrator --test calibration_bootstrap_observable_smoke
//!
//! The warm-start is CPU-only corpus math (no model / network), so this test
//! runs fully hermetically against a tempdir-backed SSB and the in-repo
//! TruthfulQA subset fixture.

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

use elai_orchestrator::ssb::{SsbActionType, SsbPayload, SsbReader, TaskSubmittedPayload};
use elai_orchestrator::{start_task_runner, Ssb, TaskRunnerConfig};

/// The two smoke tests both mutate the process-global
/// `ELAI_CALIBRATION_BOOTSTRAP_*` env vars, so they must not interleave.
/// A single binary-local lock serialises them (the contract test uses the
/// same ENV_LOCK idiom; this binary is a separate process so there is no
/// cross-binary race).
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Acquire ENV_LOCK, recovering from a poisoned lock (a prior test panic
/// while holding the guard must not cascade-fail the sibling test).
fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner())
}

/// Resolve the in-repo published-set fixture via `CARGO_MANIFEST_DIR`
/// (`crates/orchestrator/`). The fixture lives two levels up at
/// `.elai_cc/benchmarks/calibration_bootstrap/fixtures/truthfulqa_subset.jsonl`
/// (mirrors `calibration_bootstrap_integration.rs::fixture_path`).
fn fixture_path() -> PathBuf {
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set by cargo");
    let mut p = PathBuf::from(manifest_dir);
    p.pop(); // crates/orchestrator -> crates
    p.pop(); // crates -> .elai_cc
    p.push("benchmarks");
    p.push("calibration_bootstrap");
    p.push("fixtures");
    p.push("truthfulqa_subset.jsonl");
    p
}

/// Poll the SSB until at least one row of `action` is present, or `timeout`
/// elapses. Returns the row count observed (0 on timeout).
async fn wait_for_action_count(
    writer: &std::sync::Arc<Ssb>,
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

/// Build a fresh tempdir-backed `Arc<Ssb>` + cursor path. Returns the
/// TempDir guard (held by the caller for the test lifetime), the writer, and
/// the cursor path the runner records its consumed-seq into.
fn fresh_runner_env(
    session_id: &str,
) -> (tempfile::TempDir, std::sync::Arc<Ssb>, PathBuf) {
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
    let clock: std::sync::Arc<dyn ClockProvider> = std::sync::Arc::new(SeededClock::new());
    let ssb = std::sync::Arc::new(Ssb::new(cfg, clock).expect("ssb init"));
    (tmp, ssb, cursor_path)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn warm_start_emits_calibration_bootstrap_completed_when_flag_on() {
    let _g = env_lock();

    let fixture = fixture_path();
    assert!(
        fixture.exists(),
        "fixture not found at {} -- regen via the D-30 generation script",
        fixture.display()
    );

    let (tmp, ssb, cursor_path) = fresh_runner_env("d30-smoke-on");
    let tau_path = tmp.path().join("tau_bootstrap.txt");

    // Save prior env so we never leak state to sibling tests.
    let prior_enable = std::env::var("ELAI_CALIBRATION_BOOTSTRAP_ENABLE").ok();
    let prior_fixture = std::env::var("ELAI_CALIBRATION_BOOTSTRAP_FIXTURE").ok();
    let prior_tau = std::env::var("ELAI_CALIBRATION_BOOTSTRAP_TAU_PATH").ok();

    std::env::set_var("ELAI_CALIBRATION_BOOTSTRAP_ENABLE", "1");
    std::env::set_var("ELAI_CALIBRATION_BOOTSTRAP_FIXTURE", &fixture);
    std::env::set_var("ELAI_CALIBRATION_BOOTSTRAP_TAU_PATH", &tau_path);

    // Drive the REAL production path: spawn the runner, then write a plain
    // (non-research, non-scaffold) TaskSubmitted row so drive_task takes the
    // default branch and runs the startup hook.
    let (handle, cancel) =
        start_task_runner(ssb.clone(), TaskRunnerConfig::new_minimal(cursor_path));
    tokio::time::sleep(Duration::from_millis(100)).await;

    let payload = SsbPayload::TaskSubmitted(TaskSubmittedPayload {
        task: "D-30 smoke: build a hello-world CLI binary".into(),
        project_id: "dashboard".into(),
        submitted_at_ms: 0,
    });
    ssb
        .write_payload("dashboard", 0, payload, Vec::new())
        .await
        .expect("write TaskSubmitted");

    let count = wait_for_action_count(
        &ssb,
        SsbActionType::CalibrationBootstrapCompleted,
        Duration::from_secs(10),
    )
    .await;

    // Restore env BEFORE assertions so a panic does not leak the flag.
    match prior_enable {
        Some(v) => std::env::set_var("ELAI_CALIBRATION_BOOTSTRAP_ENABLE", v),
        None => std::env::remove_var("ELAI_CALIBRATION_BOOTSTRAP_ENABLE"),
    }
    match prior_fixture {
        Some(v) => std::env::set_var("ELAI_CALIBRATION_BOOTSTRAP_FIXTURE", v),
        None => std::env::remove_var("ELAI_CALIBRATION_BOOTSTRAP_FIXTURE"),
    }
    match prior_tau {
        Some(v) => std::env::set_var("ELAI_CALIBRATION_BOOTSTRAP_TAU_PATH", v),
        None => std::env::remove_var("ELAI_CALIBRATION_BOOTSTRAP_TAU_PATH"),
    }

    cancel.cancel();
    let _ = tokio::time::timeout(Duration::from_secs(5), handle).await;

    assert!(
        count >= 1,
        "expected >=1 CalibrationBootstrapCompleted row within 10s after writing \
         TaskSubmitted with ELAI_CALIBRATION_BOOTSTRAP_ENABLE=1; the warm-start hook \
         never fired through the production drive_task path (wire-dark regression)"
    );

    // The off-agent artifact's payload must reflect the warm-start outcome:
    // the canonical fixture is 50 rows, slug truthfulqa, lowered floor (not
    // the D-16 production 500), warm-start-only (CI Option fields None).
    let rows = ssb
        .query_by_type(SsbActionType::CalibrationBootstrapCompleted)
        .expect("query rows");
    let entry = rows.first().expect("at least one row");
    let decoded: SsbPayload =
        rmp_serde::from_slice(entry.payload.as_ref()).expect("decode payload");
    let p = match decoded {
        SsbPayload::CalibrationBootstrapCompleted(p) => p,
        other => panic!("expected CalibrationBootstrapCompleted, got {other:?}"),
    };
    assert_eq!(p.corpus_slug, "truthfulqa");
    assert_eq!(p.corpus_size, 50);
    assert_eq!(p.corpus_blake3.len(), 64);
    assert!(p.tau_warm_start.is_finite());
    assert!((0.0..=1.0).contains(&p.tau_warm_start));
    assert!(!p.tau_floor_active, "warm-start uses the lowered floor, not 500");
    assert!(p.bootstrap_replicates.is_none(), "warm-start-only: no SECONDARY CI");

    // The durable tau file is the secondary artifact; assert it byte-matches.
    let on_disk = std::fs::read_to_string(&tau_path).expect("read tau_path");
    let parsed: f64 = on_disk.trim().parse().expect("parse tau");
    assert!((parsed - p.tau_warm_start).abs() < 1e-12);

    drop(tmp);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn no_calibration_row_when_flag_off_default() {
    let _g = env_lock();

    let (tmp, ssb, cursor_path) = fresh_runner_env("d30-smoke-off");

    // Default-OFF invariant: with the enable flag explicitly unset, the
    // production path must stay a strict no-op. Point the fixture env at the
    // real fixture anyway to prove the GATE (not a missing fixture) is what
    // suppresses the row.
    let prior_enable = std::env::var("ELAI_CALIBRATION_BOOTSTRAP_ENABLE").ok();
    let prior_fixture = std::env::var("ELAI_CALIBRATION_BOOTSTRAP_FIXTURE").ok();
    std::env::remove_var("ELAI_CALIBRATION_BOOTSTRAP_ENABLE");
    std::env::set_var("ELAI_CALIBRATION_BOOTSTRAP_FIXTURE", fixture_path());

    let (handle, cancel) =
        start_task_runner(ssb.clone(), TaskRunnerConfig::new_minimal(cursor_path));
    tokio::time::sleep(Duration::from_millis(100)).await;

    let payload = SsbPayload::TaskSubmitted(TaskSubmittedPayload {
        task: "D-30 smoke OFF: build a hello-world CLI binary".into(),
        project_id: "dashboard".into(),
        submitted_at_ms: 0,
    });
    ssb
        .write_payload("dashboard", 0, payload, Vec::new())
        .await
        .expect("write TaskSubmitted");

    // Wait for proof that drive_task actually ran. The first thing drive_task
    // emits on the default branch is the `observe` ReasoningTrace row
    // (task_runner.rs ~5654), which is emitted AFTER
    // maybe_warm_start_calibration_once has already returned at the top of
    // drive_task. So a ReasoningTrace row proves the hook ran and -- since the
    // flag is OFF -- chose not to emit. This is stronger than waiting for the
    // slower PhaseComplete row.
    let drive_task_ran = wait_for_action_count(
        &ssb,
        SsbActionType::ReasoningTrace,
        Duration::from_secs(15),
    )
    .await;

    let calibration_rows = ssb
        .query_by_type(SsbActionType::CalibrationBootstrapCompleted)
        .unwrap_or_default()
        .len();

    // Restore env BEFORE assertions.
    match prior_enable {
        Some(v) => std::env::set_var("ELAI_CALIBRATION_BOOTSTRAP_ENABLE", v),
        None => std::env::remove_var("ELAI_CALIBRATION_BOOTSTRAP_ENABLE"),
    }
    match prior_fixture {
        Some(v) => std::env::set_var("ELAI_CALIBRATION_BOOTSTRAP_FIXTURE", v),
        None => std::env::remove_var("ELAI_CALIBRATION_BOOTSTRAP_FIXTURE"),
    }

    cancel.cancel();
    let _ = tokio::time::timeout(Duration::from_secs(5), handle).await;

    assert!(
        drive_task_ran >= 1,
        "drive_task must have run (ReasoningTrace emitted) so the no-op assertion is meaningful"
    );
    assert_eq!(
        calibration_rows, 0,
        "default-OFF invariant violated: a CalibrationBootstrapCompleted row was \
         emitted with the enable flag unset (the warm-start path must stay wire-dark)"
    );

    drop(tmp);
}
