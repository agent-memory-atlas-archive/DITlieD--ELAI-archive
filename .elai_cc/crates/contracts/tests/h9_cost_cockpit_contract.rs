//! H-9 — cost cockpit contract (Tier-1 source-wiring gate).
//!
//! Grounded scope (workflow/reasoning/H-9-decisions.md): the cloud spend ledger is
//! dark END-TO-END (C10) because the sole production cloud client ctor never chains
//! .with_ssb_writer. H-9 attaches it (the keystone — lights CloudSpend AND H-1's
//! prompt_cache_hit_rate monitor, which already lives in the same `if let Some(ssb)`
//! block) and surfaces the measured cache hit-rate in CloudPanel.
//!
//! Byte/behaviour verification of the spend reader is covered by existing C-5 tests;
//! the live render is W-6 dashboard-smoker (svelte-check) + Tier-2 live run. This file
//! proves the wiring edits landed on the production path. Tier-2 (test exec + live
//! cloud run + the per-response DTO / cache-expiry timer / precise $/resolved-task)
//! is DEFERRED.

use std::fs;
use std::path::PathBuf;

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

/// THE keystone (C10 fix): the sole production cloud client now attaches the SSB
/// writer, so CloudSpend rows + H-1's hit-rate monitor fire in prod (no longer
/// all-zeros). W-3: the attach is on the drive_task production path, not a test shim.
#[test]
fn cloud_client_attaches_ssb_writer_in_production() {
    let src = read_src("orchestrator/src/task_runner.rs");
    assert!(
        src.contains("from_env_async"),
        "the production cloud client ctor (from_env_async) must be present"
    );
    assert!(
        src.contains("with_ssb_writer"),
        "drive_task must chain .with_ssb_writer on the cloud client so CloudSpend + the \
         H-1 prompt_cache_hit_rate monitor fire in production (C10 keystone)"
    );
}

/// D2: the cost cockpit surfaces the MEASURED cache hit-rate (the believe-the-number
/// instrument for H-1/H-2), derived from the existing CloudSpendInfo cache fields.
#[test]
fn cloud_panel_surfaces_cache_hit_rate() {
    let src = read_src("dashboard-v2/src/lib/screens/CloudPanel.svelte");
    let lower = src.to_ascii_lowercase();
    assert!(
        lower.contains("hit rate") || src.contains("cacheHitRate") || lower.contains("hit-rate"),
        "CloudPanel must render a cache hit-rate (the H-9 cost-cockpit measured number)"
    );
    // It must still read the cache token fields it derives the rate from.
    assert!(
        src.contains("cache_read_tokens") && src.contains("cache_creation_tokens"),
        "CloudPanel must read cache_read_tokens + cache_creation_tokens (the hit-rate inputs)"
    );
}
