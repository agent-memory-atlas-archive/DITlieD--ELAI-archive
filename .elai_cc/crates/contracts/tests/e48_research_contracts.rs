//! E-48 Tier-A — web research orchestrator regression contracts.
//!
//! Four negative-space guards + walker sanity, using the shared
//! `tests/common/mod.rs` workspace walker (factored in E-12.A).

mod common;

use common::{collect_hits, file_contains_any, format_hits, workspace_rs_files};

#[test]
fn no_direct_search_engine_http_calls() {
    let banned = [
        "https://duckduckgo.com/?q=",
        "https://www.google.com/search?q=",
        "https://www.bing.com/search?q=",
        "https://search.brave.com/search?q=",
        "https://html.duckduckgo.com/html/?q=",
    ];
    let hits = collect_hits(&banned);
    assert!(
        hits.is_empty(),
        "Direct search-engine HTTP calls are banned — route through SearxClient.\nfound:\n  {}",
        format_hits(&hits)
    );
}

#[test]
fn no_blake2_or_sha2_for_url_or_fetch_caching() {
    // Scope to crates/web_search/ specifically — sha2 may legitimately
    // appear in transitive sandbox/TLS code paths workspace-wide.
    let banned = [
        "blake2::",
        "blake2_simd::",
        "sha2::",
        "use sha2",
        "use blake2",
    ];
    let web_search_hits: Vec<_> = workspace_rs_files()
        .into_iter()
        .filter(|p| {
            let s = p.to_string_lossy().replace('\\', "/");
            s.contains("/crates/web_search/")
        })
        .filter_map(|p| file_contains_any(&p, &banned).map(|n| (p, n)))
        .collect();
    assert!(
        web_search_hits.is_empty(),
        "web_search crate must use blake3 for content hashing — anti-stale rule.\nfound:\n  {}",
        format_hits(&web_search_hits)
    );
}

#[test]
fn no_parallel_source_tier_enum() {
    let banned = [
        "enum WebSourceTier",
        "enum ResearchSourceTier",
        "enum FetchSourceTier",
    ];
    let hits = collect_hits(&banned);
    assert!(
        hits.is_empty(),
        "Parallel SourceTier enums are banned — reuse elai_core SourceTier.\nfound:\n  {}",
        format_hits(&hits)
    );
}

#[test]
fn no_unverified_arxiv_id_in_user_facing_doc_strings() {
    let banned = [
        "arxiv:2604.",
        "arxiv:2605.",
        "arxiv:2606.",
        "arxiv:2607.",
        "arxiv:2608.",
        "arxiv:2609.",
        "arxiv:2610.",
        "arxiv:2611.",
        "arxiv:2612.",
        "arxiv:2700.",
        "arxiv:2800.",
    ];
    let hits = collect_hits(&banned);
    assert!(
        hits.is_empty(),
        "Implausible-year arxiv IDs (likely fabricated cites) detected.\nfound:\n  {}",
        format_hits(&hits)
    );
}

#[test]
fn workspace_walker_finds_at_least_one_rs_file() {
    let files = workspace_rs_files();
    assert!(
        !files.is_empty(),
        "e48 contract walker found zero .rs files — the four anti-pattern asserts cannot be trusted",
    );
}
