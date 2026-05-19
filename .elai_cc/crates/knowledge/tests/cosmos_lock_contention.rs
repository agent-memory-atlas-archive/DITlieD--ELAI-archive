//! C-16 Test File 2 — concurrent CosmosWorker writes do not produce torn
//! files. Exercises the write_atomic temp+rename path via tick() ->
//! refresh_one() (write_atomic is private; tests cannot import it directly).
//!
//! Plan reference: docs/thefinalglance/C-plan/C-16_cosmos_pipeline_integration.md
//! file 2 — cosmos_lock_contention.rs.
//!
//! Multi-threaded: uses std::thread (refresh_one is sync). Each thread
//! targets either the SAME tempdir with non-overlapping artifact names
//! (single-writer per artifact contract) or a DIFFERENT tempdir (isolation
//! invariant).

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

use elai_knowledge::cosmos_worker::{
    CosmosClock, CosmosError, CosmosManifest, CosmosWorker, ResearchDispatcher,
};

struct TestClock(AtomicI64);
impl CosmosClock for TestClock {
    fn now_secs(&self) -> i64 {
        self.0.load(Ordering::SeqCst)
    }
}

struct CannedResearch(Mutex<String>);

impl ResearchDispatcher for CannedResearch {
    fn run<'a>(
        &'a self,
        _q: &'a str,
        _t: &'a str,
        _m: u64,
    ) -> Pin<Box<dyn Future<Output = Result<(String, String), CosmosError>> + Send + 'a>> {
        // C-16 BLOCK-1 fix: trait now async-native. Resolve the body
        // synchronously (no await inside the std::sync::Mutex guard) and
        // wrap in `async move` so the future is Send.
        let body = self.0.lock().unwrap().clone();
        Box::pin(async move { Ok((body, "T1".to_string())) })
    }
}

/// Helper: build a fresh single-thread tokio runtime per std::thread so the
/// concurrent-write tests can drive async `tick()` / `refresh_one()` from a
/// std::thread without sharing an ambient runtime across threads.
fn make_thread_local_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("thread-local tokio runtime")
}

fn manifest_with_names(names: &[&str]) -> CosmosManifest {
    let mut toml = String::new();
    for n in names {
        toml.push_str(&format!(
            r#"
[[artifacts]]
name = "{}"
project_classes = ["consumer_mobile_app"]
refresh_cadence = "daily"
research_query = "query for {}"
"#,
            n, n
        ));
    }
    CosmosManifest::from_toml_str(&toml).expect("manifest parses")
}

#[test]
fn separate_tempdirs_concurrent_writes_succeed() {
    // Two workers, two tempdirs, two threads. The cosmos_root isolation
    // guarantees no overlap; tests the write-rename path under concurrent
    // pressure.
    let tmp_a = tempfile::tempdir().expect("tempdir A");
    let tmp_b = tempfile::tempdir().expect("tempdir B");

    let research_a: Arc<dyn ResearchDispatcher> =
        Arc::new(CannedResearch(Mutex::new("body-A".to_string())));
    let research_b: Arc<dyn ResearchDispatcher> =
        Arc::new(CannedResearch(Mutex::new("body-B".to_string())));
    let clock_a: Arc<dyn CosmosClock> = Arc::new(TestClock(AtomicI64::new(1_700_000_000)));
    let clock_b: Arc<dyn CosmosClock> = Arc::new(TestClock(AtomicI64::new(1_700_000_000)));

    let w_a = CosmosWorker::new(
        tmp_a.path().to_path_buf(),
        manifest_with_names(&["alpha", "beta"]),
        clock_a,
        research_a,
    )
    .expect("worker A");
    let w_b = CosmosWorker::new(
        tmp_b.path().to_path_buf(),
        manifest_with_names(&["alpha", "beta"]),
        clock_b,
        research_b,
    )
    .expect("worker B");

    let ha = std::thread::spawn(move || {
        let rt = make_thread_local_runtime();
        rt.block_on(w_a.tick())
    });
    let hb = std::thread::spawn(move || {
        let rt = make_thread_local_runtime();
        rt.block_on(w_b.tick())
    });
    let ra = ha.join().expect("thread A");
    let rb = hb.join().expect("thread B");

    assert_eq!(ra.refreshed.len(), 2);
    assert_eq!(rb.refreshed.len(), 2);
    assert!(ra.failed.is_empty() && rb.failed.is_empty());
}

#[test]
fn shared_tempdir_non_overlapping_names_no_blake3_mismatch() {
    // 4 threads, each owning a disjoint slice of the 8-artifact manifest
    // (alpha0..alpha7 split 2-per-thread). Each thread calls refresh_one
    // 5 times via repeated tick() — write_atomic temp+rename + blake3
    // verification on cosmos_get must produce zero Blake3Mismatch errors.
    let tmp = tempfile::tempdir().expect("tempdir");
    let names: Vec<String> = (0..8).map(|i| format!("alpha{}", i)).collect();
    let names_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    let manifest = manifest_with_names(&names_refs);

    let mut handles = Vec::new();
    for thread_i in 0..4u32 {
        let tmp_path = tmp.path().to_path_buf();
        let manifest_clone = manifest.clone();
        // Each thread targets its own 2-artifact slice via a per-thread
        // sub-manifest so the workers never write the SAME artifact (the
        // single-writer-per-artifact contract). The full slice list is the
        // same to keep the worker constructor happy; the threads will
        // call refresh_one with a per-thread subset.
        let h = std::thread::spawn(move || -> Result<(), CosmosError> {
            let rt = make_thread_local_runtime();
            let clock: Arc<dyn CosmosClock> =
                Arc::new(TestClock(AtomicI64::new(1_700_000_000)));
            let research: Arc<dyn ResearchDispatcher> = Arc::new(CannedResearch(
                Mutex::new(format!("body-thread-{}", thread_i)),
            ));
            let w = CosmosWorker::new(tmp_path, manifest_clone.clone(), clock, research)?;
            // Per-thread artifact names: thread_i * 2, thread_i * 2 + 1.
            let lo = (thread_i * 2) as usize;
            let hi = ((thread_i * 2) + 1) as usize;
            rt.block_on(async {
                for _ in 0..5 {
                    for idx in [lo, hi] {
                        if let Some(spec) = manifest_clone.artifacts.get(idx) {
                            let now = 1_700_000_000;
                            // refresh_one returns RefreshOutcome; we don't need
                            // the value, just the success.
                            let _ = w.refresh_one(spec, now, None).await?;
                        }
                    }
                }
                Ok::<(), CosmosError>(())
            })?;
            // Verify our slice via cosmos_get — Blake3Mismatch == Err.
            // cosmos_get stays sync (no SsbEmitter call inside post-BLOCK-1).
            for idx in [lo, hi] {
                if let Some(spec) = manifest_clone.artifacts.get(idx) {
                    let _ = w.cosmos_get(&spec.name)?;
                }
            }
            Ok(())
        });
        handles.push(h);
    }
    for h in handles {
        let res = h.join().expect("thread join");
        assert!(res.is_ok(), "thread failed: {:?}", res.err());
    }
}

#[test]
fn blake3_hash_read_after_concurrent_write_matches_body() {
    // Single artifact, two concurrent threads racing to write_atomic via
    // refresh_one. The "last writer wins" semantics + atomic rename
    // guarantee the meta blake3 matches the body on disk; cosmos_get must
    // succeed (no Blake3Mismatch).
    let tmp = tempfile::tempdir().expect("tempdir");
    let manifest = manifest_with_names(&["solo"]);
    let clock: Arc<dyn CosmosClock> = Arc::new(TestClock(AtomicI64::new(1_700_000_000)));
    let research: Arc<dyn ResearchDispatcher> =
        Arc::new(CannedResearch(Mutex::new("racy body".to_string())));
    let w = Arc::new(
        CosmosWorker::new(
            tmp.path().to_path_buf(),
            manifest.clone(),
            clock,
            research,
        )
        .expect("worker"),
    );

    let mut handles = Vec::new();
    for _ in 0..2 {
        let w = w.clone();
        let spec = manifest.artifacts[0].clone();
        handles.push(std::thread::spawn(move || {
            let rt = make_thread_local_runtime();
            rt.block_on(async {
                let now = 1_700_000_000;
                let _ = w.refresh_one(&spec, now, None).await;
            });
        }));
    }
    for h in handles {
        h.join().expect("thread join");
    }
    let art = w.cosmos_get("solo").expect("cosmos_get post-race");
    assert_eq!(art.body, "racy body");
}
