//! F-5 / Slice A4 — CosmosWorker::flag_on_staleness_broadcast cascade.
//!
//! A quarantined fact id that exactly matches a Fresh cosmos artifact's name
//! must flip that artifact to `CosmosStatus::Stale` (expressed non-destructively
//! by back-dating `refreshed_at` to the Stale-window boundary). Non-matching
//! artifacts, and already-Stale/Decayed artifacts, must be left untouched. The
//! locked 3-variant CosmosStatus enum is reused (Stale) — no new variant.
//!
//! Design reference: workflow/reasoning/F-5-decisions.md (Slice A4).

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

use elai_knowledge::cosmos_worker::{
    CosmosClock, CosmosError, CosmosManifest, CosmosWorker, ResearchDispatcher,
};

struct TestClock(AtomicI64);

impl TestClock {
    fn new(secs: i64) -> Self {
        Self(AtomicI64::new(secs))
    }
}

impl CosmosClock for TestClock {
    fn now_secs(&self) -> i64 {
        self.0.load(Ordering::SeqCst)
    }
}

struct CannedResearch;

impl ResearchDispatcher for CannedResearch {
    fn run<'a>(
        &'a self,
        _query: &'a str,
        _min_source_tier: &'a str,
        _max_size_bytes: u64,
    ) -> Pin<Box<dyn Future<Output = Result<(String, String), CosmosError>> + Send + 'a>> {
        Box::pin(async move { Ok(("# canned body".to_string(), "T1".to_string())) })
    }
}

fn two_artifact_manifest() -> CosmosManifest {
    let toml = r#"
[[artifacts]]
name = "alpha"
project_classes = ["consumer_mobile_app"]
refresh_cadence = "daily"
research_query = "alpha brief"

[[artifacts]]
name = "beta"
project_classes = ["web_app"]
refresh_cadence = "weekly"
research_query = "beta brief"
"#;
    CosmosManifest::from_toml_str(toml).expect("manifest")
}

/// Build a worker and tick once at `now` so both artifacts have Fresh on-disk
/// meta. Returns the worker + the TempDir guard.
async fn worker_with_fresh_meta(now: i64) -> (tempfile::TempDir, CosmosWorker) {
    let research: Arc<dyn ResearchDispatcher> = Arc::new(CannedResearch);
    let clock: Arc<dyn CosmosClock> = Arc::new(TestClock::new(now));
    let tmp = tempfile::tempdir().expect("tempdir");
    let w = CosmosWorker::new(tmp.path().to_path_buf(), two_artifact_manifest(), clock, research)
        .expect("worker");
    let report = w.tick().await;
    assert_eq!(report.refreshed.len(), 2, "both refreshed on first tick");
    (tmp, w)
}

/// Read an artifact's computed status snake-case string via the worker's
/// `status_snapshot` (the production dashboard read surface). The TestClock is
/// fixed at `now`, so the snapshot is computed against the same instant the
/// flag was applied at.
fn status_str(w: &CosmosWorker, name: &str) -> String {
    let snapshot = w.status_snapshot();
    snapshot
        .iter()
        .find(|a| a.name == name)
        .map(|a| a.status.clone())
        .unwrap_or_else(|| panic!("artifact '{name}' not in snapshot"))
}

#[tokio::test]
async fn matching_fresh_artifact_is_flagged_stale() {
    let now = 1_700_000_000_i64;
    let (_tmp, w) = worker_with_fresh_meta(now).await;

    // Pre-condition: both artifacts are Fresh.
    assert_eq!(status_str(&w, "alpha"), "fresh");
    assert_eq!(status_str(&w, "beta"), "fresh");

    // Quarantine the fact whose id equals artifact "alpha".
    let fact_ids = vec!["alpha".to_string()];
    let stats = w
        .flag_on_staleness_broadcast(&fact_ids, now)
        .expect("flag");

    assert_eq!(stats.artifacts_flagged, 1, "only alpha flagged");
    assert_eq!(stats.flagged_names, vec!["alpha".to_string()]);

    // alpha is now Stale; beta is untouched (still Fresh).
    assert_eq!(status_str(&w, "alpha"), "stale");
    assert_eq!(status_str(&w, "beta"), "fresh");
}

#[tokio::test]
async fn flag_is_idempotent_and_skips_already_stale() {
    let now = 1_700_000_000_i64;
    let (_tmp, w) = worker_with_fresh_meta(now).await;

    let fact_ids = vec!["alpha".to_string()];
    let first = w.flag_on_staleness_broadcast(&fact_ids, now).expect("first");
    assert_eq!(first.artifacts_flagged, 1);

    // A second broadcast for the same (now-Stale) artifact flags nothing.
    let second = w
        .flag_on_staleness_broadcast(&fact_ids, now)
        .expect("second");
    assert_eq!(
        second.artifacts_flagged, 0,
        "already-Stale artifact is not re-flagged"
    );
    assert_eq!(status_str(&w, "alpha"), "stale");
}

#[tokio::test]
async fn no_matching_artifact_flags_nothing() {
    let now = 1_700_000_000_i64;
    let (_tmp, w) = worker_with_fresh_meta(now).await;

    let fact_ids = vec!["does-not-exist".to_string()];
    let stats = w
        .flag_on_staleness_broadcast(&fact_ids, now)
        .expect("flag");
    assert_eq!(stats.artifacts_flagged, 0);
    assert_eq!(status_str(&w, "alpha"), "fresh");
    assert_eq!(status_str(&w, "beta"), "fresh");
}
