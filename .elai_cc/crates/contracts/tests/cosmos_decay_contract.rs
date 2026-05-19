//! Contract test: C-16 — RefreshCadence + CosmosWorker decay banding.
//!
//! Pins the Fresh/Stale/Decayed classification at the API boundary so the
//! 3x divergence from the C-16 plan's 2x prose (documented in
//! workflow/reasoning/C-16-decisions.md DECISION-I and cosmos_worker.rs:103-115)
//! cannot silently regress.
//!
//! Verifies:
//!   (1) decay_secs() == 3 * interval_secs() for all four cadences.
//!   (2) status_for(...) returns Fresh when age < interval.
//!   (3) status_for(...) returns Stale when interval <= age < 3*interval.
//!   (4) status_for(...) returns Decayed when age >= 3*interval.
//!   (5) cosmos_get() returns Ok with status=Decayed when artifact is
//!       beyond 3x threshold (no Err — Decayed serves with WARN).
//!   (6) cosmos_get() returns Err(Blake3Mismatch) when body is tampered
//!       (NOT swallowed as Decayed — integrity wins over freshness).
//!
//! Plan reference: docs/thefinalglance/C-plan/C-16_cosmos_pipeline_integration.md
//! file 4 — cosmos_decay_contract.rs.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use elai_knowledge::cosmos_worker::{
    CosmosArtifactMeta, CosmosClock, CosmosError, CosmosManifest, CosmosStatus,
    CosmosWorker, RefreshCadence, ResearchDispatcher,
};

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

struct FixedClock(AtomicI64);
impl CosmosClock for FixedClock {
    fn now_secs(&self) -> i64 {
        self.0.load(Ordering::SeqCst)
    }
}

struct CannedResearch(String);
impl ResearchDispatcher for CannedResearch {
    fn run<'a>(
        &'a self,
        _q: &'a str,
        _t: &'a str,
        _m: u64,
    ) -> Pin<Box<dyn Future<Output = Result<(String, String), CosmosError>> + Send + 'a>> {
        let body = self.0.clone();
        Box::pin(async move { Ok((body, "T1".to_string())) })
    }
}

fn manifest_with_cadence(name: &str, cadence: &str) -> CosmosManifest {
    let toml = format!(
        r#"
[[artifacts]]
name = "{name}"
project_classes = ["consumer_mobile_app"]
refresh_cadence = "{cadence}"
research_query = "q"
"#
    );
    CosmosManifest::from_toml_str(&toml).expect("manifest")
}

// ---------------------------------------------------------------------------
// (1) decay_secs() == 3 * interval_secs() for all cadences.
// ---------------------------------------------------------------------------

#[test]
fn decay_secs_is_three_times_interval_for_all_cadences() {
    // C-16 DECISION-I: deliberate 3x divergence from plan §8 prose.
    for cadence in [
        RefreshCadence::Daily,
        RefreshCadence::Weekly,
        RefreshCadence::Monthly,
        RefreshCadence::Quarterly,
    ] {
        assert_eq!(
            cadence.decay_secs(),
            3 * cadence.interval_secs(),
            "decay_secs != 3 * interval_secs for {:?}",
            cadence
        );
    }
}

// ---------------------------------------------------------------------------
// (2)+(3)+(4) status_for classifies Fresh / Stale / Decayed correctly.
// ---------------------------------------------------------------------------

fn make_worker_for_status(now_secs: i64) -> (tempfile::TempDir, CosmosWorker) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let clock: Arc<dyn CosmosClock> = Arc::new(FixedClock(AtomicI64::new(now_secs)));
    let research: Arc<dyn ResearchDispatcher> = Arc::new(CannedResearch("body".to_string()));
    let w = CosmosWorker::new(
        tmp.path().to_path_buf(),
        manifest_with_cadence("test", "daily"),
        clock,
        research,
    )
    .expect("worker");
    (tmp, w)
}

#[test]
fn status_for_fresh_when_age_below_interval() {
    let now = 1_700_000_000;
    let (_tmp, w) = make_worker_for_status(now);
    // Age = 1 hour, interval = 1 day -> Fresh.
    let meta = CosmosArtifactMeta {
        name: "test".into(),
        blake3_hash: "h".into(),
        size_bytes: 1,
        refreshed_at: now - 3600,
        source_tier: "T1".into(),
        refresh_cadence: RefreshCadence::Daily,
    };
    assert_eq!(w.status_for(&meta, now), CosmosStatus::Fresh);
}

#[test]
fn status_for_stale_when_age_between_interval_and_three_intervals() {
    let now = 1_700_000_000;
    let (_tmp, w) = make_worker_for_status(now);
    let daily = RefreshCadence::Daily;
    // Age just past interval (1.5 days) -> Stale.
    let meta = CosmosArtifactMeta {
        name: "test".into(),
        blake3_hash: "h".into(),
        size_bytes: 1,
        refreshed_at: now - (daily.interval_secs() * 3 / 2),
        source_tier: "T1".into(),
        refresh_cadence: daily,
    };
    assert_eq!(w.status_for(&meta, now), CosmosStatus::Stale);

    // Age at 2.5 days (still below 3x) -> Stale.
    let meta2 = CosmosArtifactMeta {
        refreshed_at: now - (daily.interval_secs() * 5 / 2),
        ..meta
    };
    assert_eq!(w.status_for(&meta2, now), CosmosStatus::Stale);
}

#[test]
fn status_for_decayed_when_age_at_or_beyond_three_intervals() {
    let now = 1_700_000_000;
    let (_tmp, w) = make_worker_for_status(now);
    let daily = RefreshCadence::Daily;

    // Age = 3 * interval EXACTLY -> Decayed (boundary inclusive).
    let meta_at = CosmosArtifactMeta {
        name: "test".into(),
        blake3_hash: "h".into(),
        size_bytes: 1,
        refreshed_at: now - daily.decay_secs(),
        source_tier: "T1".into(),
        refresh_cadence: daily,
    };
    assert_eq!(w.status_for(&meta_at, now), CosmosStatus::Decayed);

    // Age = 5 days (well past 3 days) -> Decayed.
    let meta_far = CosmosArtifactMeta {
        refreshed_at: now - 5 * 24 * 3600,
        ..meta_at
    };
    assert_eq!(w.status_for(&meta_far, now), CosmosStatus::Decayed);
}

// ---------------------------------------------------------------------------
// (5) cosmos_get returns Ok(Decayed) when artifact exceeds 3x threshold.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn cosmos_get_serves_decayed_artifact_with_status() {
    let now_initial: i64 = 1_700_000_000;
    let tmp = tempfile::tempdir().expect("tempdir");
    let clock = Arc::new(FixedClock(AtomicI64::new(now_initial)));
    let clock_dyn: Arc<dyn CosmosClock> = clock.clone();
    let research: Arc<dyn ResearchDispatcher> = Arc::new(CannedResearch("# body".to_string()));
    let w = CosmosWorker::new(
        tmp.path().to_path_buf(),
        manifest_with_cadence("test", "daily"),
        clock_dyn,
        research,
    )
    .expect("worker");

    let report = w.tick().await;
    assert_eq!(report.refreshed.len(), 1);

    // Advance the clock past 3 days so the artifact is Decayed.
    clock.0.store(now_initial + 5 * 24 * 3600, Ordering::SeqCst);

    let art = w.cosmos_get("test").expect("Decayed serves Ok with status");
    assert_eq!(art.status, CosmosStatus::Decayed);
    assert_eq!(art.body, "# body");
}

// ---------------------------------------------------------------------------
// (6) cosmos_get returns Err(Blake3Mismatch) when body is tampered.
//     Integrity check overrides freshness — even a "Fresh" artifact whose
//     body fails the blake3 check must Err, not silently downgrade to Decayed.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn cosmos_get_returns_blake3_mismatch_when_body_tampered() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let clock: Arc<dyn CosmosClock> = Arc::new(FixedClock(AtomicI64::new(1_700_000_000)));
    let research: Arc<dyn ResearchDispatcher> =
        Arc::new(CannedResearch("# original body".to_string()));
    let w = CosmosWorker::new(
        tmp.path().to_path_buf(),
        manifest_with_cadence("test", "daily"),
        clock,
        research,
    )
    .expect("worker");

    let _ = w.tick().await;
    // Overwrite the body file directly so blake3 mismatches the recorded meta.
    let body_path = tmp.path().join("artifacts").join("test.md");
    std::fs::write(&body_path, b"# tampered body").expect("write tampered");

    let err = w
        .cosmos_get("test")
        .expect_err("blake3 mismatch must Err");
    assert!(
        matches!(err, CosmosError::Blake3Mismatch(_)),
        "expected Blake3Mismatch, got {:?}",
        err
    );
}
