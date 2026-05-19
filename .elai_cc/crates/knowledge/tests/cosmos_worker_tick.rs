//! C-16 Test File 1 — CosmosWorker::tick() end-to-end behavior.
//!
//! Standalone integration test exercising the worker via its public API.
//! Mocks the research dispatcher + clock so the test is hermetic.
//!
//! Plan reference: docs/thefinalglance/C-plan/C-16_cosmos_pipeline_integration.md
//! Test cases enumerated in plan §"Test files" file 1.

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicI64, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use elai_knowledge::cosmos_worker::{
    CosmosClock, CosmosError, CosmosManifest, CosmosWorker, ResearchDispatcher, SsbEmitter,
};

// ---------------------------------------------------------------------------
// Test fixtures
// ---------------------------------------------------------------------------

struct TestClock(AtomicI64);

impl TestClock {
    fn new(secs: i64) -> Self {
        Self(AtomicI64::new(secs))
    }
    fn advance(&self, secs: i64) {
        self.0.fetch_add(secs, Ordering::SeqCst);
    }
}

impl CosmosClock for TestClock {
    fn now_secs(&self) -> i64 {
        self.0.load(Ordering::SeqCst)
    }
}

struct CannedResearch {
    body: Mutex<String>,
    tier: String,
}

impl CannedResearch {
    fn new(body: &str, tier: &str) -> Self {
        Self {
            body: Mutex::new(body.to_string()),
            tier: tier.to_string(),
        }
    }
    fn set_body(&self, body: &str) {
        *self.body.lock().expect("research mutex") = body.to_string();
    }
}

impl ResearchDispatcher for CannedResearch {
    fn run<'a>(
        &'a self,
        _query: &'a str,
        _min_source_tier: &'a str,
        _max_size_bytes: u64,
    ) -> Pin<Box<dyn Future<Output = Result<(String, String), CosmosError>> + Send + 'a>> {
        // Resolve under the std::sync::Mutex synchronously; do not hold the
        // guard across the awaited future (it is not Send).
        let body = self.body.lock().expect("research mutex").clone();
        let tier = self.tier.clone();
        Box::pin(async move { Ok((body, tier)) })
    }
}

#[derive(Default)]
struct CountingEmitter {
    refresh_started: AtomicU32,
    artifact_updated: AtomicU32,
    artifact_decayed: AtomicU32,
    last_started_queued: AtomicU32,
    last_started_total: AtomicU32,
    last_updated_body_bytes: AtomicU32,
    last_updated_blake3: Mutex<String>,
}

impl SsbEmitter for CountingEmitter {
    fn emit_refresh_started<'a>(
        &'a self,
        queued: u32,
        total: u32,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            self.refresh_started.fetch_add(1, Ordering::SeqCst);
            self.last_started_queued.store(queued, Ordering::SeqCst);
            self.last_started_total.store(total, Ordering::SeqCst);
        })
    }
    fn emit_artifact_updated<'a>(
        &'a self,
        _name: &'a str,
        blake3_hash: &'a str,
        body_bytes: u32,
        _latency_ms: u32,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        let blake3_owned = blake3_hash.to_string();
        Box::pin(async move {
            self.artifact_updated.fetch_add(1, Ordering::SeqCst);
            self.last_updated_body_bytes.store(body_bytes, Ordering::SeqCst);
            *self.last_updated_blake3.lock().unwrap() = blake3_owned;
        })
    }
    fn emit_artifact_decayed<'a>(
        &'a self,
        _name: &'a str,
        _age_secs: u64,
        _cadence: &'a str,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            self.artifact_decayed.fetch_add(1, Ordering::SeqCst);
        })
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

fn build_worker(
    research: Arc<dyn ResearchDispatcher>,
    clock: Arc<dyn CosmosClock>,
) -> (tempfile::TempDir, CosmosWorker) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let w = CosmosWorker::new(
        tmp.path().to_path_buf(),
        two_artifact_manifest(),
        clock,
        research,
    )
    .expect("worker");
    (tmp, w)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn tick_writes_body_and_meta_on_first_run() {
    let research: Arc<dyn ResearchDispatcher> = Arc::new(CannedResearch::new("# body", "T1"));
    let clock: Arc<dyn CosmosClock> = Arc::new(TestClock::new(1_700_000_000));
    let (_tmp, w) = build_worker(research, clock);

    let report = w.tick().await;
    assert_eq!(report.checked, 2);
    assert_eq!(report.refreshed.len(), 2, "both artifacts refreshed on first run");
    assert_eq!(report.outcomes.len(), 2, "outcomes aligned with refreshed");
    assert!(report.failed.is_empty(), "no failures: {:?}", report.failed);
    // Outcomes carry real blake3 / body_bytes.
    for o in &report.outcomes {
        assert!(o.changed, "first refresh must be 'changed'");
        assert!(!o.blake3.is_empty(), "blake3 must be populated");
        assert_eq!(o.body_bytes, b"# body".len() as u32);
    }
}

#[tokio::test]
async fn tick_skips_fresh_artifacts() {
    let research: Arc<dyn ResearchDispatcher> = Arc::new(CannedResearch::new("body", "T1"));
    let clock = Arc::new(TestClock::new(1_700_000_000));
    let clock_dyn: Arc<dyn CosmosClock> = clock.clone();
    let (_tmp, w) = build_worker(research, clock_dyn);

    // First tick refreshes.
    let r1 = w.tick().await;
    assert_eq!(r1.refreshed.len(), 2);

    // No time advance: all artifacts are Fresh; second tick must not refresh.
    let r2 = w.tick().await;
    assert!(r2.refreshed.is_empty(), "expected no refresh on Fresh artifacts");
    assert_eq!(r2.unchanged.len(), 2);
}

#[tokio::test]
async fn tick_refreshes_stale_artifacts() {
    let research: Arc<dyn ResearchDispatcher> =
        Arc::new(CannedResearch::new("v1", "T1"));
    let clock = Arc::new(TestClock::new(1_700_000_000));
    let clock_dyn: Arc<dyn CosmosClock> = clock.clone();
    let (_tmp, w) = build_worker(research.clone(), clock_dyn);

    let _ = w.tick().await;
    // Change body and advance past Daily interval (1 day < 2 days < 3 days).
    let canned = research.clone();
    // Downcast trick: we keep a direct reference for set_body.
    drop(canned);
    // Easier: rebuild a fresh CannedResearch since trait object can't be
    // mutated via the dyn pointer.
    let research2 = Arc::new(CannedResearch::new("v2", "T1"));
    let research2_dyn: Arc<dyn ResearchDispatcher> = research2.clone();
    let (_tmp2, w2) = build_worker(research2_dyn, clock.clone() as Arc<dyn CosmosClock>);
    let _ = w2.tick().await;
    // Stale window for Daily: age >= 1 day, age < 3 days.
    clock.advance(2 * 24 * 3600);
    research2.set_body("v2-changed");
    let r = w2.tick().await;
    assert!(
        r.refreshed.contains(&"alpha".to_string()),
        "alpha (Daily) should be Stale and refreshed: {:?}",
        r
    );
}

#[tokio::test]
async fn tick_refreshes_decayed_artifacts() {
    let research = Arc::new(CannedResearch::new("v1", "T1"));
    let research_dyn: Arc<dyn ResearchDispatcher> = research.clone();
    let clock = Arc::new(TestClock::new(1_700_000_000));
    let clock_dyn: Arc<dyn CosmosClock> = clock.clone();
    let (_tmp, w) = build_worker(research_dyn, clock_dyn);

    let _ = w.tick().await;
    research.set_body("v2");
    // Decayed window: age >= 3 * Daily = 3 days. Use 7 days.
    clock.advance(7 * 24 * 3600);
    let r = w.tick().await;
    assert!(
        r.refreshed.contains(&"alpha".to_string()),
        "alpha (Decayed) should be refreshed: {:?}",
        r
    );
}

#[tokio::test]
async fn tick_is_idempotent_when_body_unchanged() {
    let research = Arc::new(CannedResearch::new("identical body", "T1"));
    let research_dyn: Arc<dyn ResearchDispatcher> = research.clone();
    let clock = Arc::new(TestClock::new(1_700_000_000));
    let clock_dyn: Arc<dyn CosmosClock> = clock.clone();
    let (_tmp, w) = build_worker(research_dyn, clock_dyn);

    let _ = w.tick().await;
    // Advance past Daily interval; body unchanged -> idempotent skip.
    clock.advance(2 * 24 * 3600);
    let r = w.tick().await;
    assert!(
        r.refreshed.is_empty(),
        "no refresh expected on identical body: {:?}",
        r
    );
    // unchanged + failed should cover both manifest entries.
    assert_eq!(r.unchanged.len() + r.failed.len(), 2);
    assert!(r.failed.is_empty(), "no failures: {:?}", r.failed);
}

#[tokio::test]
async fn tick_records_failed_artifacts_without_panic() {
    struct AlwaysFails;
    impl ResearchDispatcher for AlwaysFails {
        fn run<'a>(
            &'a self,
            _q: &'a str,
            _t: &'a str,
            _m: u64,
        ) -> Pin<Box<dyn Future<Output = Result<(String, String), CosmosError>> + Send + 'a>>
        {
            Box::pin(async move {
                Err(CosmosError::Research("synthetic failure".into()))
            })
        }
    }
    let research: Arc<dyn ResearchDispatcher> = Arc::new(AlwaysFails);
    let clock: Arc<dyn CosmosClock> = Arc::new(TestClock::new(1));
    let (_tmp, w) = build_worker(research, clock);

    let r = w.tick().await;
    assert_eq!(r.checked, 2);
    assert_eq!(r.failed.len(), 2, "both artifacts must fail");
    assert!(r.refreshed.is_empty());
    // Test contract: failed[i].1 is the error string.
    for (_name, err) in &r.failed {
        assert!(err.contains("synthetic"), "error preserved: {}", err);
    }
}

#[tokio::test]
async fn tick_report_counts_match_manifest_size() {
    let research: Arc<dyn ResearchDispatcher> = Arc::new(CannedResearch::new("b", "T1"));
    let clock: Arc<dyn CosmosClock> = Arc::new(TestClock::new(1_700_000_000));
    let (_tmp, w) = build_worker(research, clock);
    let r = w.tick().await;
    assert_eq!(r.checked, w.manifest().artifacts.len());
    assert_eq!(
        r.refreshed.len() + r.unchanged.len() + r.failed.len(),
        r.checked
    );
}

#[test]
fn concurrent_workers_do_not_corrupt_separate_tempdirs() {
    // Two independent workers / tempdirs / clocks. The cosmos_root isolation
    // guarantees no shared file. This test pins the no-cross-contamination
    // invariant (separate tempdir = separate state).
    //
    // Each std::thread owns a fresh current-thread tokio runtime so the
    // async `tick()` can run without sharing an ambient runtime across
    // threads (post C-16 BLOCK-1 fix).
    let r1: Arc<dyn ResearchDispatcher> = Arc::new(CannedResearch::new("body1", "T1"));
    let r2: Arc<dyn ResearchDispatcher> = Arc::new(CannedResearch::new("body2", "T1"));
    let c1: Arc<dyn CosmosClock> = Arc::new(TestClock::new(1_700_000_000));
    let c2: Arc<dyn CosmosClock> = Arc::new(TestClock::new(1_700_000_000));
    let (_t1, w1) = build_worker(r1, c1);
    let (_t2, w2) = build_worker(r2, c2);

    let h1 = std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("thread 1 runtime");
        rt.block_on(w1.tick())
    });
    let h2 = std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("thread 2 runtime");
        rt.block_on(w2.tick())
    });
    let rep1 = h1.join().expect("thread 1");
    let rep2 = h2.join().expect("thread 2");

    assert_eq!(rep1.refreshed.len(), 2);
    assert_eq!(rep2.refreshed.len(), 2);
    assert!(rep1.failed.is_empty() && rep2.failed.is_empty());
}

#[tokio::test]
async fn ssb_emitter_some_receives_refresh_started_and_artifact_updated() {
    let research: Arc<dyn ResearchDispatcher> = Arc::new(CannedResearch::new("# body", "T1"));
    let clock: Arc<dyn CosmosClock> = Arc::new(TestClock::new(1_700_000_000));
    let emitter = Arc::new(CountingEmitter::default());
    let emitter_dyn: Arc<dyn SsbEmitter> = emitter.clone();

    let tmp = tempfile::tempdir().expect("tempdir");
    let w = CosmosWorker::new(
        tmp.path().to_path_buf(),
        two_artifact_manifest(),
        clock,
        research,
    )
    .expect("worker")
    .with_ssb_emitter(emitter_dyn);

    let r = w.tick().await;
    assert_eq!(r.refreshed.len(), 2);
    // ONE refresh_started before the manifest loop.
    assert_eq!(emitter.refresh_started.load(Ordering::SeqCst), 1);
    // count_due on first run = all 2 (no meta on disk yet).
    assert_eq!(emitter.last_started_queued.load(Ordering::SeqCst), 2);
    assert_eq!(emitter.last_started_total.load(Ordering::SeqCst), 2);
    // ONE artifact_updated per artifact whose body actually changed.
    assert_eq!(emitter.artifact_updated.load(Ordering::SeqCst), 2);
    // body_bytes + blake3 must be REAL — not zero / empty.
    assert!(emitter.last_updated_body_bytes.load(Ordering::SeqCst) > 0);
    assert!(!emitter.last_updated_blake3.lock().unwrap().is_empty());
}

#[tokio::test]
async fn ssb_emitter_none_is_a_no_op() {
    let research: Arc<dyn ResearchDispatcher> = Arc::new(CannedResearch::new("# body", "T1"));
    let clock: Arc<dyn CosmosClock> = Arc::new(TestClock::new(1_700_000_000));
    let tmp = tempfile::tempdir().expect("tempdir");
    let w = CosmosWorker::new(
        tmp.path().to_path_buf(),
        two_artifact_manifest(),
        clock,
        research,
    )
    .expect("worker");
    // No emitter attached. tick() must complete without panic.
    let r = w.tick().await;
    assert_eq!(r.refreshed.len(), 2);
}

#[tokio::test]
async fn count_due_returns_stale_plus_missing_plus_decayed_only() {
    let research = Arc::new(CannedResearch::new("body", "T1"));
    let research_dyn: Arc<dyn ResearchDispatcher> = research.clone();
    let clock = Arc::new(TestClock::new(1_700_000_000));
    let clock_dyn: Arc<dyn CosmosClock> = clock.clone();
    let (_tmp, w) = build_worker(research_dyn, clock_dyn);

    // Pre-refresh: both artifacts have no meta on disk -> due = 2.
    assert_eq!(w.count_due(), 2, "missing meta counted as due");

    // After one tick at t=0, both are Fresh -> due = 0.
    let _ = w.tick().await;
    assert_eq!(w.count_due(), 0, "fresh artifacts not counted");

    // Advance past Daily interval: alpha becomes Stale -> due = 1.
    clock.advance(2 * 24 * 3600);
    assert_eq!(w.count_due(), 1, "one stale artifact counted");
}
