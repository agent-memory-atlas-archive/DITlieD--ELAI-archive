//! C-8 contract test — cosmos tick wiring asserts that the
//! `trigger_cosmos_refresh` helper (task_runner.rs:7924) — driven by the
//! hourly `tokio::time::interval(DEFAULT_TICK_INTERVAL_SECS)` scheduler
//! at `cli::live_cli:1964` — emits real `CosmosRefreshStarted` SSB rows
//! (not tracing-only) for every refresh sweep.
//!
//! Source plan: docs/thefinalglance/C-plan/C-8_viewer_stub_eradication.md
//!   (plan ADD clause for tests/cosmos_tick_wiring_contract.rs;
//!    Task 3 hourly scheduler)
//! Decisions:   workflow/reasoning/C-8-decisions.md
//!   (DRIFT-C/DRIFT-D for the scheduler-host live_cli divergence and the
//!    external `cosmos_tick_emit_loop` wrapper around the sync `tick()`
//!    surface)
//!
//! Sibling coverage:
//!   - cosmos_decay_contract.rs (C-16): RefreshCadence decay banding —
//!     classifies Fresh/Stale/Decayed, doesn't exercise tick wiring.
//!   - c8_stub_eradication_contract.rs: SSB action-type + payload
//!     round-trip via the SQLite writer — doesn't exercise
//!     `trigger_cosmos_refresh` as the producer.
//!
//! Tests are intentionally narrow:
//!   (1) `trigger_cosmos_refresh` writes a `CosmosRefreshStarted` SSB
//!       row whose `queued` + `total` fields reflect the manifest size
//!       (the canonical wiring proof that the helper is plumbed to the
//!       writer, not stuck on a tracing-only path).
//!   (2) `trigger_cosmos_refresh` over an empty manifest still emits
//!       exactly one `CosmosRefreshStarted` row (queued=0, total=0) —
//!       confirming the unconditional pre-loop SSB write contract
//!       (matches the helper at task_runner.rs:7942-7951 which writes
//!       the row before checking the artifact list).
//!   (3) `refreshed_count` return value is 0 when no specs need
//!       refreshing — the contract that the hourly scheduler relies on
//!       to log meaningful idle vs. busy ticks (live_cli.rs cosmos
//!       scheduler observes this counter).

use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_knowledge::cosmos_worker::{
    CosmosClock, CosmosError, CosmosManifest, CosmosWorker, ResearchDispatcher,
};
use elai_orchestrator::ssb::{Ssb, SsbConfig};
use elai_orchestrator::{trigger_cosmos_refresh, SsbViewer};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

struct FixedClock(AtomicI64);
impl CosmosClock for FixedClock {
    fn now_secs(&self) -> i64 {
        self.0.load(Ordering::SeqCst)
    }
}

/// A canned dispatcher that never gets called in the empty-manifest path.
/// Mirrors the `CannedResearch` fixture at cosmos_decay_contract.rs:42.
struct UnusedResearch;
impl ResearchDispatcher for UnusedResearch {
    fn run<'a>(
        &'a self,
        _q: &'a str,
        _t: &'a str,
        _m: u64,
    ) -> Pin<Box<dyn Future<Output = Result<(String, String), CosmosError>> + Send + 'a>> {
        Box::pin(async move {
            // Reach-only safety — the wiring tests below never trigger refresh_one.
            Ok((String::new(), "T1".to_string()))
        })
    }
}

fn fresh_db() -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("ssb.db");
    (dir, path)
}

async fn open_writer(path: &std::path::Path) -> Arc<Ssb> {
    let cfg = SsbConfig {
        db_path: path.to_path_buf(),
        pool_size: Some(2),
        project_id: "c8-cosmos-tick".into(),
        session_id: "c8-cosmos-tick-test".into(),
    };
    Arc::new(Ssb::new(cfg, Arc::new(OsClock)).expect("ssb init"))
}

/// Build a fresh CosmosWorker rooted at a tempdir with the supplied
/// manifest TOML. `now_secs` is the fixed clock value.
fn make_worker(
    manifest_toml: &str,
    now_secs: i64,
) -> (TempDir, CosmosWorker) {
    let tmp = TempDir::new().expect("worker tempdir");
    let manifest =
        CosmosManifest::from_toml_str(manifest_toml).expect("manifest parse");
    let clock: Arc<dyn CosmosClock> = Arc::new(FixedClock(AtomicI64::new(now_secs)));
    let research: Arc<dyn ResearchDispatcher> = Arc::new(UnusedResearch);
    let worker =
        CosmosWorker::new(tmp.path().to_path_buf(), manifest, clock, research)
            .expect("CosmosWorker::new");
    (tmp, worker)
}

// ---------------------------------------------------------------------------
// (1) Single-artifact manifest -> one CosmosRefreshStarted row with
//     queued=1, total=1 emitted via writer.write_payload (real SSB row,
//     not tracing-only).
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn trigger_cosmos_refresh_emits_refresh_started_row_for_one_artifact_manifest() {
    let (_db_dir, db_path) = fresh_db();
    let writer = open_writer(&db_path).await;

    // One-artifact manifest. Empty cosmos_root means no meta on disk,
    // so the artifact counts as "missing -> due" per cosmos_worker.rs:505
    // count_due() classification (queued = 1).
    let toml = r#"
[[artifacts]]
name = "saas_pricing_patterns"
project_classes = ["consumer_mobile_app"]
refresh_cadence = "weekly"
research_query = "competitive saas pricing tiers"
"#;
    let (_worker_dir, worker) = make_worker(toml, 1_700_000_000);

    // Drive the helper.
    let _refreshed_count = trigger_cosmos_refresh(&writer, &worker).await;

    // Drop the writer so the pooled SQLite connections release before
    // the read-side SsbViewer opens. Mirrors the c8_stub_eradication
    // contract drop order at c8_stub_eradication_contract.rs:120.
    drop(writer);

    let viewer = SsbViewer::open(&db_path).expect("viewer open");
    let rows = viewer
        .recent_rows_of_type("cosmos_refresh_started", 4)
        .expect("recent_rows_of_type");
    assert!(
        !rows.is_empty(),
        "trigger_cosmos_refresh must emit at least one cosmos_refresh_started SSB row \
         (witness: the helper is wired to writer.write_payload, not tracing-only)",
    );

    // Pin the wire shape: queued is the "due" count (here 1 — missing
    // meta), total is the manifest size (1). Matches the C-16 DECISION-F
    // contract that `queued` mirrors `count_due()`, not manifest size.
    let payload = &rows[0].payload_json;
    let started = payload
        .get("CosmosRefreshStarted")
        .expect("CosmosRefreshStarted variant key in payload_json");
    assert_eq!(
        started["total"].as_u64(),
        Some(1),
        "total must equal the 1-artifact manifest size; got {started}",
    );
    assert_eq!(
        started["queued"].as_u64(),
        Some(1),
        "queued must equal count_due() (1: missing meta on fresh tempdir); got {started}",
    );
}

// ---------------------------------------------------------------------------
// (2) Empty manifest -> still emits exactly one CosmosRefreshStarted
//     row (queued=0, total=0) because the helper writes the row BEFORE
//     entering the artifact loop (task_runner.rs:7942 unconditional
//     pre-loop write).
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn trigger_cosmos_refresh_emits_zero_zero_row_for_empty_manifest() {
    let (_db_dir, db_path) = fresh_db();
    let writer = open_writer(&db_path).await;

    // Empty manifest (no [[artifacts]] blocks).
    let (_worker_dir, worker) = make_worker("", 1_700_000_000);

    let refreshed_count = trigger_cosmos_refresh(&writer, &worker).await;
    assert_eq!(
        refreshed_count, 0,
        "empty manifest must produce 0 refreshed artifacts",
    );

    drop(writer);

    let viewer = SsbViewer::open(&db_path).expect("viewer open");
    let rows = viewer
        .recent_rows_of_type("cosmos_refresh_started", 4)
        .expect("recent_rows_of_type");
    assert_eq!(
        rows.len(),
        1,
        "empty manifest must still emit exactly one cosmos_refresh_started row \
         (witness: helper writes pre-loop unconditionally per task_runner.rs:7942)",
    );

    let started = rows[0]
        .payload_json
        .get("CosmosRefreshStarted")
        .expect("CosmosRefreshStarted variant key");
    assert_eq!(
        started["queued"].as_u64(),
        Some(0),
        "queued must be 0 for empty manifest; got {started}",
    );
    assert_eq!(
        started["total"].as_u64(),
        Some(0),
        "total must be 0 for empty manifest; got {started}",
    );
}

// ---------------------------------------------------------------------------
// (3) refreshed_count = 0 contract — the hourly scheduler observes this
//     counter to distinguish idle ticks from busy ones. For an empty
//     manifest, the contract is 0.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn trigger_cosmos_refresh_returns_zero_for_empty_manifest_no_artifact_updated_row() {
    let (_db_dir, db_path) = fresh_db();
    let writer = open_writer(&db_path).await;
    let (_worker_dir, worker) = make_worker("", 1_700_000_000);

    let refreshed_count = trigger_cosmos_refresh(&writer, &worker).await;
    assert_eq!(refreshed_count, 0, "empty manifest -> 0 refreshed");

    drop(writer);

    // Per task_runner.rs:7980 the helper only emits CosmosArtifactUpdated
    // on Ok(o) with o.changed=true. An empty manifest can never produce
    // such a row — pin the contract so a future regression that emits
    // a phantom updated row on idle ticks fails fast.
    let viewer = SsbViewer::open(&db_path).expect("viewer open");
    let rows = viewer
        .recent_rows_of_type("cosmos_artifact_updated", 4)
        .expect("recent_rows_of_type");
    assert!(
        rows.is_empty(),
        "empty manifest must NOT emit cosmos_artifact_updated rows; got {} rows",
        rows.len(),
    );
}
