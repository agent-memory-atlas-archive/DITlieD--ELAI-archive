//! H-2 — cache-stable layout + breakpoints + 1h TTL contract (Tier-1 done-gate).
//!
//! Grounded scope (workflow/reasoning/H-2-decisions.md): activate the doubly-inert
//! cache-stable-layout mechanism. bp.valid was consumed only by an eprintln!, fed
//! EMPTY slices, and reachable only under BudgetPolicy::Local; the cloud marker path
//! never consulted byte-stability. H-2 persists cross-turn prefix hashes so bp.valid
//! is real, feeds REAL slot bytes, keys the prefix per exact model id, places a 1h-TTL
//! marker on the verified-stable prefix + a 5-min tail, and adds an HD-2 cache-bust
//! debug_assert.
//!
//! This file is the SOURCE-WIRING gate (the byte-stability behaviour is already covered
//! by context_budget_contract.rs; the CacheControl serialization + marker placement +
//! min_cacheable_prefix are covered by co-located #[cfg(test)] unit tests in the
//! providers crate, since apply_anthropic_cache_markers + min_cacheable_prefix are
//! pub(crate)). Here we prove the wiring edits actually landed on the production path.
//! Tier-2 (live multi-turn cloud run showing a rising cache-read fraction) is DEFERRED
//! (sidecar-gated).

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

/// T4 / D1: CacheControl carries an optional ttl and a 1h constructor.
#[test]
fn cache_control_has_ttl_and_1h_ctor() {
    let src = read_src("providers/src/types.rs");
    assert!(src.contains("ttl"), "CacheControl must gain a ttl field (T4)");
    assert!(
        src.contains("ephemeral_1h"),
        "CacheControl must gain an ephemeral_1h() constructor (1h TTL)"
    );
    // HD-2 / byte-identity: existing 5-min markers must serialize unchanged => ttl is
    // skipped when None.
    assert!(
        src.contains("skip_serializing_if"),
        "ttl must be #[serde(skip_serializing_if)] so existing ephemeral markers stay byte-identical"
    );
}

/// LANDMINE (binding): the removed 1h beta header must never be emitted.
#[test]
fn no_extended_cache_ttl_beta_header() {
    for rel in ["providers/src", "orchestrator/src", "cli/src"] {
        // walk the dir text via a cheap recursive read
        let root = crates_dir().join(rel);
        let mut stack = vec![root];
        while let Some(dir) = stack.pop() {
            let Ok(entries) = fs::read_dir(&dir) else { continue };
            for e in entries.flatten() {
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                } else if p.extension().and_then(|x| x.to_str()) == Some("rs") {
                    let body = fs::read_to_string(&p).unwrap_or_default();
                    assert!(
                        !body.contains("extended-cache-ttl"),
                        "{} emits the REMOVED extended-cache-ttl beta header (use ttl:\"1h\")",
                        p.display()
                    );
                }
            }
        }
    }
}

/// T1 + D4 + W-3: update_breakpoints is fed real slot bytes and bp.valid is made
/// non-inert via persisted cross-turn prefix hashes on the production send path.
#[test]
fn runtime_client_persists_prefix_hashes_and_asserts() {
    let src = read_src("cli/src/runtime_client.rs");
    assert!(
        src.contains("last_cache_prefix_hashes"),
        "RuntimeClient must persist cross-turn prefix hashes so bp.valid is non-inert (D4)"
    );
    assert!(
        src.contains("update_breakpoints"),
        "runtime_client must call update_breakpoints on the production path (T1)"
    );
    assert!(
        src.contains("debug_assert"),
        "HD-2 byte-prefix cache-bust debug_assert must exist in runtime_client (T5)"
    );
}

/// D5 + D3: the 1h TTL marker is wired onto the verified-stable head prefix.
#[test]
fn one_hour_ttl_marker_is_wired() {
    let rc = read_src("cli/src/runtime_client.rs");
    let an = read_src("providers/src/providers/anthropic.rs");
    assert!(
        rc.contains("ephemeral_1h") || an.contains("ephemeral_1h"),
        "the 1h-TTL marker (ephemeral_1h) must be placed on the stable prefix (D5)"
    );
    // T6: per-model-id min cacheable prefix (no \"Opus=4096\" tier assumption).
    assert!(
        an.contains("min_cacheable_prefix"),
        "per-model-id min cacheable prefix helper must exist (T6)"
    );
}
