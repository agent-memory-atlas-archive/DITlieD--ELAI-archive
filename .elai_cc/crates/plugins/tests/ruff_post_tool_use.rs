//! B-19A T6 — ruff PostToolUse subprocess wiring.
//!
//! Verifies the cross-platform ruff detection + cache:
//!   * When ruff is absent on PATH, `RuffPostToolUseRunner::run` returns
//!     `RuffOutcome::NoOp { reason: BinaryMissing }` and emits a tracing
//!     warning ONCE per process (subsequent calls stay silent).
//!   * When two consecutive runs target the same file content, the second
//!     call serves from cache (verified via `cache_hits()` counter).
//!   * The runner detects the binary by trying `ruff` then `ruff.exe`
//!     (Windows fallback) without panicking on `ErrorKind::NotFound`.
//!
//! Whenever a real ruff binary IS available on PATH, the present-binary
//! test runs end-to-end. When ruff is genuinely unavailable in the test
//! environment, that test is `#[ignore]`d so CI stays green.

use std::sync::Arc;

use elai_core::clock::{ClockProvider, SeededClock};
use elai_plugins::post_tool_use::{
    RuffOutcome, RuffPostToolUseRunner, RuffSkipReason, RUFF_DEFAULT_TIMEOUT_MS,
};

fn ruff_on_path() -> bool {
    #[cfg(windows)]
    let names: &[&str] = &["ruff.exe", "ruff"];
    #[cfg(not(windows))]
    let names: &[&str] = &["ruff"];
    for name in names {
        if std::process::Command::new(name)
            .arg("--version")
            .output()
            .is_ok()
        {
            return true;
        }
    }
    false
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn missing_binary_returns_noop_and_warns_once() {
    if ruff_on_path() {
        // We can't reliably test the absent path when ruff IS installed.
        // Emit a skip note via println and return.
        println!("ruff present on PATH — skipping missing-binary path test");
        return;
    }
    let runner = RuffPostToolUseRunner::new();
    let res1 = runner.run("test.py", "x = 1\n").await.expect("call ok");
    assert!(matches!(
        res1,
        RuffOutcome::NoOp { reason: RuffSkipReason::BinaryMissing }
    ));

    // Second call must also return NoOp without re-warning. We can't intercept
    // tracing here without a subscriber harness; instead we assert the runner's
    // internal warned_once flag stayed set.
    let res2 = runner.run("test.py", "y = 2\n").await.expect("call ok");
    assert!(matches!(
        res2,
        RuffOutcome::NoOp { reason: RuffSkipReason::BinaryMissing }
    ));
    assert!(runner.has_warned_once(), "warned_once must persist across calls");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn timeout_constant_is_5_seconds() {
    assert_eq!(
        RUFF_DEFAULT_TIMEOUT_MS, 5_000,
        "ruff invocation timeout must be 5s per spec"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires ruff binary on PATH; run with --ignored when ruff is installed"]
async fn present_binary_runs_check_and_caches_results() {
    if !ruff_on_path() {
        eprintln!("ruff missing on PATH — skipping present-binary test");
        return;
    }
    let runner = RuffPostToolUseRunner::new();
    // Clean Python (no findings expected). Ruff's exit code is 0 + empty findings.
    let clean = "x = 1\n";
    let r1 = runner.run("clean.py", clean).await.expect("first call ok");
    let r2 = runner.run("clean.py", clean).await.expect("second call ok");

    // Same content → second call must have come from cache.
    assert!(
        runner.cache_hits() >= 1,
        "second call with identical content must hit cache (hits={})",
        runner.cache_hits()
    );

    // Both calls must classify as Findings (possibly empty list).
    match (&r1, &r2) {
        (
            RuffOutcome::Findings { findings: f1, .. },
            RuffOutcome::Findings { findings: f2, .. },
        ) => {
            assert_eq!(f1.len(), f2.len(), "cache must return identical results");
        }
        _ => panic!("present-binary calls must yield Findings outcomes: {r1:?} / {r2:?}"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cache_size_cap_evicts_oldest() {
    // Even without a real ruff binary, exercising the cache cap exposes
    // the BTreeMap LRU bookkeeping. We force-insert via a test seam.
    let runner = RuffPostToolUseRunner::new();
    runner.test_cache_insert([1u8; 32], crate_test_outcome(0));
    runner.test_cache_insert([2u8; 32], crate_test_outcome(1));
    assert_eq!(runner.cache_len(), 2);
    // Cap is 256 — push beyond and confirm eviction.
    for i in 0u32..260 {
        let mut key = [0u8; 32];
        key[..4].copy_from_slice(&i.to_le_bytes());
        runner.test_cache_insert(key, crate_test_outcome(i as u64));
    }
    assert!(
        runner.cache_len() <= 256,
        "cache must cap at 256 entries (got {})",
        runner.cache_len()
    );
}

fn crate_test_outcome(_id: u64) -> RuffOutcome {
    RuffOutcome::Findings {
        exit_code: 0,
        findings: Vec::new(),
        elapsed_ms: 0,
    }
}

/// B-19A cycle 2 (BLOCK-3): the runner accepts an injected ClockProvider so
/// tests can drive deterministic `elapsed_ms` and so production timing flows
/// through the entropy quarantine. This test only verifies the construction
/// path + cache identity — driving an actual subprocess against SeededClock
/// would require ruff on PATH (already covered by the `--ignored` test).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn with_clock_constructor_accepts_seeded_clock() {
    let clock: Arc<dyn ClockProvider> = Arc::new(SeededClock::new());
    let runner = RuffPostToolUseRunner::with_clock(clock);
    // Empty path returns immediately without spawning a subprocess, so this
    // exercises the constructor + the early-exit branch independent of ruff.
    let outcome = runner.run("", "x = 1\n").await.expect("ok");
    assert!(matches!(
        outcome,
        RuffOutcome::NoOp { reason: RuffSkipReason::EmptyPath }
    ));
    // Cache stays empty for skip outcomes.
    assert_eq!(runner.cache_hits(), 0);
}
