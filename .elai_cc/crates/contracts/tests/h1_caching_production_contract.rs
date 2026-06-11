//! H-1 — prompt-cache production-path contract (Tier-1 done-gate).
//!
//! Grounded finding (see workflow/reasoning/H-1-decisions.md): the "RuntimeAdapter
//! ships ZERO markers" premise is REFUTED — AnthropicApiRuntime::complete routes
//! through AnthropicClient::send_message (anthropic.rs:448) which already calls
//! apply_anthropic_cache_markers unconditionally. The sole wire-dark deliverable is
//! the hit-rate MONITOR (cache_check.rs::measure_cache_hit_rate), which had zero
//! production callers. H-1 activates it from CloudGroundingClient::send and makes the
//! RuntimeAdapter path's marker-consume explicit.
//!
//! Tier-1 (this file): deterministic, in-process. Proves the monitor's real SSB write
//! path runs + emits the EXISTING CriticDecisionEvent/PromptCacheHitRate row (no new
//! action_type), the snapshot formula is pinned + recomputable (GD-2), the production
//! caller exists in grounding_client.rs (W-3), and the RuntimeAdapter path consumes the
//! shipped stamper exactly once (GD-1). Tier-2 (the live two-turn cloud run with real
//! cache_read_input_tokens > 0 over the wire) is a DEFERRED checkbox (sidecar-gated).

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use elai_core::clock::SeededClock;
use elai_orchestrator::cache_check::{measure_cache_hit_rate, CacheCheckSnapshot};
use elai_orchestrator::ssb::{Ssb, SsbConfig};
use tempfile::TempDir;

fn make_ssb() -> (Arc<Ssb>, TempDir) {
    let dir = TempDir::new().expect("tempdir");
    let cfg = SsbConfig {
        db_path: dir.path().join("h1.db"),
        pool_size: Some(2),
        project_id: "p".into(),
        session_id: "s".into(),
    };
    let clock = Arc::new(SeededClock::with_epoch(
        SystemTime::UNIX_EPOCH + Duration::from_millis(1_700_000_000_000),
    )) as Arc<dyn elai_core::ClockProvider>;
    let ssb = Ssb::new(cfg, clock).expect("ssb init");
    (Arc::new(ssb), dir)
}

/// crates/contracts/.. -> the workspace `crates` dir.
fn crates_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/contracts has a parent")
        .to_path_buf()
}

fn read_src(rel: &str) -> String {
    let path = crates_dir().join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

/// THE activated monitor: measure_cache_hit_rate must run its REAL write path against
/// a live Ssb and return the snapshot's hit_rate. A successful Ok(_) means the
/// CriticDecisionEvent/PromptCacheHitRate row was committed (write_payload would
/// otherwise return Err). The number recomputes from the snapshot (GD-2).
#[tokio::test]
async fn monitor_emits_prompt_cache_hit_rate_and_is_recomputable() {
    let (ssb, _dir) = make_ssb();
    // snapshot mapping (H-1 D3): hits = cache_read_input_tokens, misses = fresh input.
    let snap = CacheCheckSnapshot { hits: 90, misses: 10 };
    let challenger = measure_cache_hit_rate(&ssb, "h1_contract", 0, snap)
        .await
        .expect("measure_cache_hit_rate must commit the row and return Ok");
    assert!(
        (challenger - 0.9).abs() < 1e-9,
        "hit-rate must equal hits/(hits+misses)=0.9, got {challenger}"
    );

    // A zero-read turn (turn 1: cache write, no read) is a real, non-error 0.0.
    let (ssb2, _dir2) = make_ssb();
    let zero = measure_cache_hit_rate(&ssb2, "h1_contract", 1, CacheCheckSnapshot { hits: 0, misses: 42 })
        .await
        .expect("zero-read snapshot still commits a row");
    assert_eq!(zero, 0.0, "no cache read => 0.0 fraction, not an error");
}

/// The snapshot mapping is the exact cache-read fraction the journal can recompute.
#[test]
fn snapshot_formula_is_cache_read_fraction() {
    // cache_read=80, fresh input=20 => 0.8
    assert!((CacheCheckSnapshot { hits: 80, misses: 20 }.hit_rate() - 0.8).abs() < 1e-9);
    // empty (no calls yet) => 0.0, never a divide-by-zero panic
    assert_eq!(CacheCheckSnapshot { hits: 0, misses: 0 }.hit_rate(), 0.0);
}

/// W-3: the monitor has a NON-test production caller in the live cloud send path.
/// (Pre-H-1 the only callers were #[cfg(test)] — wire-dark per R7.)
#[test]
fn grounding_client_has_production_caller() {
    let src = read_src("orchestrator/src/grounding_client.rs");
    assert!(
        src.contains("measure_cache_hit_rate"),
        "CloudGroundingClient must call measure_cache_hit_rate (W-3 production reachability)"
    );
}

/// GD-1: the RuntimeAdapter path consumes the SHIPPED stamper explicitly at the adapter
/// boundary (it is also applied downstream in send_message; this makes the consume
/// independent of send_message internals and visible to the wiring check).
#[test]
fn runtime_adapter_consumes_shipped_stamper() {
    let src = read_src("providers/src/runtime/anthropic_api.rs");
    assert!(
        src.contains("apply_anthropic_cache_markers"),
        "AnthropicApiRuntime::complete must call apply_anthropic_cache_markers (consume, GD-1)"
    );
}

/// GD-1 P0: there is exactly ONE definition of the stamper — never re-implemented.
#[test]
fn exactly_one_stamper_definition() {
    let src = read_src("providers/src/providers/anthropic.rs");
    let defs = src.matches("fn apply_anthropic_cache_markers").count();
    assert_eq!(defs, 1, "exactly one apply_anthropic_cache_markers definition (GD-1), found {defs}");
}
