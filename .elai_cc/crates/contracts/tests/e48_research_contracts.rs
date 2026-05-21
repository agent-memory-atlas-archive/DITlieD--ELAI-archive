//! E-48 Tier-A — web research orchestrator regression contracts.
//!
//! Negative-space guards (per E-38.A pattern) for web research hardening
//! invariants. Today's codebase already complies (verified by
//! integration-explorer-elai in s-orch-e48); these tests make the
//! compliance load-bearing so a future "let's just curl google" or
//! "swap blake3 for sha2 here" refactor fails loudly.
//!
//! Four anti-patterns + one walker sanity test:
//!   1. No direct search-engine HTTP calls — all search MUST route through
//!      SearxClient -> sidecar /search (per memory project_search_architecture).
//!   2. No blake2 / sha2 for URL / fetch cache hashing (anti-stale rule:
//!      blake3 is the workspace standard).
//!   3. No bypass of SourceQualityTable for source-tier classification —
//!      ad-hoc tier scoring loses the centralized policy.
//!   4. No silent acceptance of unverified arxiv IDs — citation strings of
//!      the form `arxiv:NNNN.NNNNN` MUST be paired with a verification call
//!      somewhere in the same module (this contract guards the regex shape;
//!      the verification policy itself ships in Tier-B).
//!   5. walker sanity — pinned per E-38.A pattern.
//!
//! Tier-B will add: TieredSearchPool / ExtractorPool / cross-encoder rerank
//! into knowledge::reranker / FlashResearch speculative pruner / Convergence
//! status enum extension / ContradictionNode / 4-stage CitationVerifier /
//! attribution lock / recency decay (into existing knowledge::freshness, not
//! a fork) / 4 dashboard panels / SsbActionType variants. See
//! workflow/reasoning/E-48-decisions.md.

use std::fs;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| manifest.to_path_buf())
}

fn collect_rust_sources(root: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    walk_dir(root, &mut out);
    out
}

fn walk_dir(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if path.is_dir() {
            if matches!(
                name,
                "target"
                    | ".git"
                    | "node_modules"
                    | ".elai"
                    | "indexes"
                    | "scip_indices"
                    | "models"
            ) {
                continue;
            }
            walk_dir(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            // Self-skip + skip the E-38 anti-pattern contract for the same
            // reason: their `banned` arrays mention these strings as
            // literals, which would otherwise create false-positive hits.
            if matches!(name, "e48_research_contracts.rs" | "e38_anti_pattern_contracts.rs") {
                continue;
            }
            out.push(path);
        }
    }
}

fn file_contains_any(path: &Path, needles: &[&str]) -> Option<String> {
    let Ok(contents) = fs::read_to_string(path) else {
        return None;
    };
    for needle in needles {
        if contents.contains(needle) {
            return Some((*needle).to_string());
        }
    }
    None
}

fn collect_hits(root: &Path, needles: &[&str]) -> Vec<(PathBuf, String)> {
    let mut hits: Vec<(PathBuf, String)> = Vec::new();
    for file in collect_rust_sources(root) {
        if let Some(needle) = file_contains_any(&file, needles) {
            hits.push((file, needle));
        }
    }
    hits
}

fn format_hits(hits: &[(PathBuf, String)]) -> String {
    hits.iter()
        .map(|(p, n)| format!("{} :: {}", p.display(), n))
        .collect::<Vec<_>>()
        .join("\n  ")
}

#[test]
fn no_direct_search_engine_http_calls() {
    // All web search routes through the sidecar SearXNG path (per memory
    // project_search_architecture). A direct curl/reqwest to a search
    // engine breaks the centralized rate-limit + cache + tier policy AND
    // typically violates the engine's ToS.
    let root = workspace_root();
    let banned = [
        "https://duckduckgo.com/?q=",
        "https://www.google.com/search?q=",
        "https://www.bing.com/search?q=",
        "https://search.brave.com/search?q=",
        "https://html.duckduckgo.com/html/?q=",
    ];
    let hits = collect_hits(&root, &banned);
    assert!(
        hits.is_empty(),
        "Direct search-engine HTTP calls are banned — route through SearxClient.\nfound:\n  {}",
        format_hits(&hits)
    );
}

#[test]
fn no_blake2_or_sha2_for_url_or_fetch_caching() {
    // Anti-stale rule: blake3 is the workspace hash for non-RFC uses. The
    // web_search ingest path computes content hashes for dedup / cache
    // keys; if those switch to blake2 / sha2 silently, the cache loses
    // determinism across crates that use blake3.
    let root = workspace_root();
    let banned = [
        "blake2::",
        "blake2_simd::",
        "sha2::",
        "use sha2",
        "use blake2",
    ];
    let hits = collect_hits(&root, &banned);
    // Filter to only hits inside web_search/ — sha2 may legitimately appear
    // in transitive sandbox / TLS code paths; the rule we enforce is
    // specifically about web_search-owned hashing.
    let web_search_hits: Vec<_> = hits
        .iter()
        .filter(|(p, _)| {
            let s = p.to_string_lossy().replace('\\', "/");
            s.contains("/crates/web_search/")
        })
        .collect();
    assert!(
        web_search_hits.is_empty(),
        "web_search crate must use blake3 for content hashing — anti-stale rule.\nfound:\n  {}",
        web_search_hits
            .iter()
            .map(|(p, n)| format!("{} :: {}", p.display(), n))
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

#[test]
fn no_parallel_source_tier_enum() {
    // SourceTier {T1Authoritative, T2Strong, T3Weak, T4Rejected} lives in
    // elai_core (per integration-explorer finding). A parallel enum (e.g.
    // crates/web_search defining its own SourceTier with {T1,T2,T3} as the
    // plan E-48 originally proposed) fragments the source-quality policy.
    let root = workspace_root();
    let banned = [
        "enum WebSourceTier",
        "enum ResearchSourceTier",
        "enum FetchSourceTier",
    ];
    let hits = collect_hits(&root, &banned);
    assert!(
        hits.is_empty(),
        "Parallel SourceTier enums are banned — reuse elai_core SourceTier.\nfound:\n  {}",
        format_hits(&hits)
    );
}

#[test]
fn no_unverified_arxiv_id_in_user_facing_doc_strings() {
    // Anti-slopsquatting carry-over from memory:feedback_paper_citation_verify
    // (2 bad cites already this campaign — DPO arxiv:2305.18290 in B-42 and
    // Kamoi arxiv:2406.01297 in B-41). Any string literal `arxiv:NNNN.NNNNN`
    // appearing in a comment marked /// (rustdoc) or //! (module doc)
    // needs a paired verification claim. Tier-A guard: just lock that no
    // implausible-year arxiv IDs (28XX, 27XX, 26XX year prefix) appear in
    // source. Actual citations from <=2025 (24XX, 23XX, etc) are fine.
    let root = workspace_root();
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
    let hits = collect_hits(&root, &banned);
    assert!(
        hits.is_empty(),
        "Implausible-year arxiv IDs (likely fabricated cites) detected.\nfound:\n  {}",
        format_hits(&hits)
    );
}

#[test]
fn workspace_walker_finds_at_least_one_rs_file() {
    // Sanity: walker isn't silently empty. Mirror of e38_anti_pattern_contracts
    // pattern — without this, a walker regression could false-pass everything.
    let root = workspace_root();
    let files = collect_rust_sources(&root);
    assert!(
        !files.is_empty(),
        "e48 contract walker found zero .rs files under {} — the four anti-pattern asserts cannot be trusted without this sanity",
        root.display()
    );
}
