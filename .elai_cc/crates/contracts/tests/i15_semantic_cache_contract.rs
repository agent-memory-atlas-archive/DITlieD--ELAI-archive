//! I-15 — semantic cache + blake3 fast path contract (source-wiring gate).
//!
//! Verifies the three WIRING_CONTRACT claims:
//!
//! (a) EXACT-REPEAT: a second identical query to AragOrchestrator::semantic_search
//!     is served from the intermediate_summary_cache (blake3 exact path) with
//!     ZERO embed calls on the second invocation.  Instrumented via the public
//!     intermediate_summary_cache_lookup() seam (the exact cache is observable
//!     without a counting fake embedder — the cache len grows on insert and a
//!     subsequent lookup returns Some).
//!
//! (b) CONFINEMENT: code-gen-path queries (non-research, never routed through
//!     AragOrchestrator in production) CANNOT reach the disk-backed
//!     OracleAnswerCache.  Extends h8_oracle_persist_wiring.rs.
//!
//! (c) DISCRIMINATOR: OracleCacheHitPayload and OracleCacheMissPayload each carry
//!     a `source: Option<String>` field that round-trips through msgpack.
//!
//! All assertions are unit-logic / source-grep checks; no live model required.

use std::path::PathBuf;

fn read_src(rel: &str) -> String {
    let crates_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/contracts has a parent")
        .to_path_buf();
    let path = crates_dir.join(rel);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

// ---------------------------------------------------------------------------
// (a) EXACT-REPEAT: blake3 fast path short-circuits ONNX on second call
// ---------------------------------------------------------------------------

/// A-1: AragOrchestrator exposes intermediate_summary_cache_lookup and
/// intermediate_summary_cache_insert as the observable seam (WIRING_CONTRACT
/// produces_symbols: blake3_exact_lookup, intermediate_summary_cache_insert/_lookup).
#[test]
fn a1_arag_exact_cache_seam_is_present() {
    let src = read_src("knowledge/src/arag.rs");
    assert!(
        src.contains("fn intermediate_summary_cache_lookup"),
        "arag.rs must expose intermediate_summary_cache_lookup (I-15 seam)"
    );
    assert!(
        src.contains("fn intermediate_summary_cache_insert"),
        "arag.rs must expose intermediate_summary_cache_insert (I-15 seam)"
    );
    assert!(
        src.contains("blake3_exact_lookup") || src.contains("exact_cache_lookup"),
        "arag.rs must contain the blake3 exact lookup helper (ordering invariant)"
    );
}

/// A-2: The ONNX embedder call (embedder.embed(query)) must NOT appear BEFORE
/// the exact-cache check in semantic_search.  We verify structural ordering in
/// source: the exact_cache_lookup call site must appear before the
/// `embedder.embed(query)` call site in the semantic_search function body.
#[test]
fn a2_exact_cache_check_precedes_onnx_embed_call() {
    let src = read_src("knowledge/src/arag.rs");

    // Locate the semantic_search function body.
    let fn_pos = src
        .find("fn semantic_search(")
        .expect("semantic_search must exist in arag.rs");

    let fn_slice = &src[fn_pos..];

    // The exact cache check must come before the embedder.embed call.
    let cache_check_pos = fn_slice
        .find("exact_cache_lookup(")
        .expect("exact_cache_lookup call must be inside semantic_search body");

    // Use "match embedder.embed(query)" to avoid matching the inline comment at the
    // top of semantic_search that documents the invariant ("NO embedder.embed(query)
    // before this check") — the comment appears before the actual call site in source,
    // which would cause a naive .find("embedder.embed(query)") to report the wrong offset.
    let embed_pos = fn_slice
        .find("match embedder.embed(query)")
        .expect("match embedder.embed(query) call must be inside semantic_search body");

    assert!(
        cache_check_pos < embed_pos,
        "In semantic_search: exact_cache_lookup (offset {cache_check_pos}) must appear \
         BEFORE match embedder.embed(query) (offset {embed_pos}) — ordering invariant from WIRING_CONTRACT stub_ban"
    );
}

/// A-3: Live round-trip — insert into the exact cache, then verify lookup returns
/// the same results without any embedder call.  Uses the public seam directly;
/// no embedder is constructed.
#[test]
fn a3_exact_cache_insert_then_lookup_returns_hit() {
    use elai_knowledge::arag::{AragChunk, AragConfig, AragOrchestrator, RetrievalResult};

    let mut arag = AragOrchestrator::new(AragConfig::default());
    // Add a chunk so semantic_search has something to score.
    arag.add_chunk(AragChunk {
        chunk_id: "node_1".to_string(),
        content: "Rust ownership model borrow checker lifetime".to_string(),
        embedding: Vec::new(), // empty — norm proxy path, no ONNX
    });

    let query = "what is the Rust ownership model";
    let now_secs: u64 = 1_750_000_000;

    // Before any insert: lookup must return None.
    assert!(
        arag.intermediate_summary_cache_lookup(query, now_secs).is_none(),
        "cache must be empty before any insert"
    );

    // Insert a synthetic result set.
    let fake_results = vec![RetrievalResult {
        chunk_id: "node_1".to_string(),
        snippet: "... Rust ownership model ...".to_string(),
        score: 0.9,
        tool_used: "semantic_search".to_string(),
    }];
    arag.intermediate_summary_cache_insert(query, fake_results.clone(), now_secs);

    // Second lookup must hit.
    let hit = arag.intermediate_summary_cache_lookup(query, now_secs);
    assert!(hit.is_some(), "cache must return Some on exact-match lookup after insert");
    let hit = hit.unwrap();
    assert_eq!(hit.len(), 1, "cached result count must match inserted count");
    assert_eq!(hit[0].chunk_id, "node_1");
    assert_eq!(hit[0].tool_used, "semantic_search");

    // Different query must still miss.
    assert!(
        arag.intermediate_summary_cache_lookup("completely different query", now_secs).is_none(),
        "a different query must not hit the cache"
    );
}

/// A-4: TTL expiry — a lookup past INTERMEDIATE_SUMMARY_CACHE_TTL_SECS returns None
/// and evicts the entry.
#[test]
fn a4_exact_cache_ttl_expiry() {
    use elai_knowledge::arag::{AragConfig, AragOrchestrator, RetrievalResult, INTERMEDIATE_SUMMARY_CACHE_TTL_SECS};

    let arag = AragOrchestrator::new(AragConfig::default());
    let query = "ttl expiry test query";
    let now_secs: u64 = 1_750_000_000;

    arag.intermediate_summary_cache_insert(
        query,
        vec![RetrievalResult {
            chunk_id: "node_ttl".to_string(),
            snippet: "...".to_string(),
            score: 0.5,
            tool_used: "semantic_search".to_string(),
        }],
        now_secs,
    );

    // Just before expiry: still fresh.
    assert!(
        arag.intermediate_summary_cache_lookup(query, now_secs + INTERMEDIATE_SUMMARY_CACHE_TTL_SECS - 1).is_some(),
        "entry must be fresh one second before TTL boundary"
    );

    // At or after TTL: expired.
    assert!(
        arag.intermediate_summary_cache_lookup(query, now_secs + INTERMEDIATE_SUMMARY_CACHE_TTL_SECS).is_none(),
        "entry must be expired at or after INTERMEDIATE_SUMMARY_CACHE_TTL_SECS"
    );
}

// ---------------------------------------------------------------------------
// (b) CONFINEMENT: code-gen path cannot reach the disk-backed OracleAnswerCache
// ---------------------------------------------------------------------------

/// B-1: Extend h8_oracle_persist_wiring.rs D3 — the disk-backed
/// `persisted_default()` constructor must be called ONLY from the
/// `/research|/ask` arm (repl_command.rs), never from a code-gen path.
/// This asserts the confinement rule holds after I-15 edits.
#[test]
fn b1_oracle_cache_disk_ctor_confined_to_research_arm() {
    let repl_src = read_src("cli/src/repl_command.rs");
    let oracle_src = read_src("orchestrator/src/oracle_cache.rs");

    // D3 (h8): research arm still constructs persisted_default.
    assert!(
        repl_src.contains("persisted_default") || repl_src.contains("persisted("),
        "repl_command.rs research arm must still construct the disk-backed cache (h8 D3 regression)"
    );

    // I-15 must NOT have added a second persisted_default() call to arag.rs.
    let arag_src = read_src("knowledge/src/arag.rs");
    assert!(
        !arag_src.contains("persisted_default") && !arag_src.contains("persisted("),
        "arag.rs must NOT call persisted_default() — second disk-backed ctor violates H-8 confinement"
    );

    // oracle_cache.rs itself must still contain the canonical ctor.
    assert!(
        oracle_src.contains("persisted") && oracle_src.contains("persist_to_disk"),
        "oracle_cache.rs must still have persisted + persist_to_disk (h8 regression guard)"
    );
}

/// B-2: The in-memory exact cache inside AragOrchestrator must use FxHashMap,
/// never std HashMap (per house rule: FxHashMap not std HashMap on hot paths).
#[test]
fn b2_exact_cache_uses_fxhashmap_not_std_hashmap() {
    let src = read_src("knowledge/src/arag.rs");
    assert!(
        src.contains("FxHashMap"),
        "arag.rs exact cache must use FxHashMap (not std HashMap)"
    );
    // std::collections::HashMap must not appear in arag.rs as the cache type.
    // (HashSet is allowed — it's used for dedup in query_pipeline.)
    let hashmap_count = src.matches("HashMap<").count();
    let fxhashmap_count = src.matches("FxHashMap<").count();
    assert!(
        fxhashmap_count > 0,
        "FxHashMap must be used at least once in arag.rs"
    );
    // All HashMap usages must be FxHashMap (no bare std HashMap for the cache).
    assert_eq!(
        hashmap_count, fxhashmap_count,
        "every HashMap<...> in arag.rs must be FxHashMap<...> (std HashMap banned on hot cache path)"
    );
}

// ---------------------------------------------------------------------------
// (c) DISCRIMINATOR: source field round-trips on both payload types
// ---------------------------------------------------------------------------

/// C-1: OracleCacheHitPayload gains source: Option<String> with serde(default).
#[test]
fn c1_oracle_cache_hit_payload_has_source_field() {
    let src = read_src("orchestrator/src/ssb.rs");
    // Find the OracleCacheHitPayload struct block.
    let struct_pos = src
        .find("pub struct OracleCacheHitPayload")
        .expect("OracleCacheHitPayload must exist in ssb.rs");
    let struct_slice = &src[struct_pos..struct_pos + 600];
    assert!(
        struct_slice.contains("pub source: Option<String>"),
        "OracleCacheHitPayload must have pub source: Option<String> (I-15 discriminator)"
    );
    assert!(
        struct_slice.contains("#[serde(default"),
        "OracleCacheHitPayload.source must carry #[serde(default...)] for backward compat"
    );
}

/// C-2: OracleCacheMissPayload gains source: Option<String> with serde(default).
#[test]
fn c2_oracle_cache_miss_payload_has_source_field() {
    let src = read_src("orchestrator/src/ssb.rs");
    let struct_pos = src
        .find("pub struct OracleCacheMissPayload")
        .expect("OracleCacheMissPayload must exist in ssb.rs");
    let struct_slice = &src[struct_pos..struct_pos + 400];
    assert!(
        struct_slice.contains("pub source: Option<String>"),
        "OracleCacheMissPayload must have pub source: Option<String> (I-15 discriminator)"
    );
    assert!(
        struct_slice.contains("#[serde(default"),
        "OracleCacheMissPayload.source must carry #[serde(default...)] for backward compat"
    );
}

/// C-3: OracleCacheHitPayload round-trips through msgpack with source set.
#[test]
fn c3_oracle_cache_hit_payload_roundtrip_with_source() {
    use elai_orchestrator::ssb::{OracleCacheHitPayload, SsbPayload};

    let payload = SsbPayload::OracleCacheHit(OracleCacheHitPayload {
        query_hash_hex: "aa".repeat(32),
        source_tier: "t1_authoritative".into(),
        cached_age_secs: 120,
        source: Some("blake3_exact".into()),
    });

    let encoded = rmp_serde::to_vec(&payload)
        .expect("OracleCacheHitPayload must encode to msgpack");
    let decoded: SsbPayload =
        rmp_serde::from_slice(&encoded).expect("must decode from msgpack");

    if let SsbPayload::OracleCacheHit(p) = decoded {
        assert_eq!(p.source, Some("blake3_exact".into()), "source must survive msgpack roundtrip");
        assert_eq!(p.cached_age_secs, 120);
    } else {
        panic!("decoded to wrong variant");
    }
}

/// C-4: OracleCacheMissPayload round-trips through msgpack with source set.
#[test]
fn c4_oracle_cache_miss_payload_roundtrip_with_source() {
    use elai_orchestrator::ssb::{OracleCacheMissPayload, SsbPayload};

    let payload = SsbPayload::OracleCacheMiss(OracleCacheMissPayload {
        query_hash_hex: "bb".repeat(32),
        confidence: "high".into(),
        source: Some("semantic_vector".into()),
    });

    let encoded = rmp_serde::to_vec(&payload)
        .expect("OracleCacheMissPayload must encode to msgpack");
    let decoded: SsbPayload =
        rmp_serde::from_slice(&encoded).expect("must decode from msgpack");

    if let SsbPayload::OracleCacheMiss(p) = decoded {
        assert_eq!(p.source, Some("semantic_vector".into()), "source must survive msgpack roundtrip");
        assert_eq!(p.confidence, "high");
    } else {
        panic!("decoded to wrong variant");
    }
}

/// C-5: Legacy payload (no source field) deserializes to source=None
/// (backward compat via serde default).
#[test]
fn c5_legacy_payload_without_source_deserializes_to_none() {
    use elai_orchestrator::ssb::{OracleCacheHitPayload, SsbPayload};

    // Construct a payload WITHOUT source field, then re-encode via JSON
    // (simulating a pre-I-15 row arriving from the SSB store).
    let legacy_json = r#"{"query_hash_hex":"cc","source_tier":"t2_strong","cached_age_secs":60}"#;
    let decoded: OracleCacheHitPayload =
        serde_json::from_str(legacy_json).expect("legacy JSON must deserialize via serde default");
    assert_eq!(decoded.source, None, "legacy row without source field must deserialize to None");

    // Also verify the full SsbPayload wrapper path.
    let _ = SsbPayload::OracleCacheHit(decoded);
}

/// C-6: SCHEMA_VERSION is 233 after the I-15 payload extension.
#[test]
fn c6_schema_version_is_233() {
    assert_eq!(
        elai_orchestrator::ssb::SCHEMA_VERSION_PUBLIC,
        233,
        "SCHEMA_VERSION must be 233 after I-15 payload extension"
    );
}

/// C-7: repl_command.rs threads source discriminator at both emit sites.
#[test]
fn c7_repl_command_threads_source_discriminator() {
    let src = read_src("cli/src/repl_command.rs");
    // Hit path must set source to blake3_exact.
    assert!(
        src.contains("blake3_exact"),
        "repl_command.rs OracleCacheHit emit must set source=Some(\"blake3_exact\")"
    );
    // Miss path must set source to semantic_vector.
    assert!(
        src.contains("semantic_vector"),
        "repl_command.rs OracleCacheMiss emit must set source=Some(\"semantic_vector\")"
    );
}

// ---------------------------------------------------------------------------
// Dashboard cross-lang pair
// ---------------------------------------------------------------------------

/// D-1: types.ts declares OracleCacheSource type.
#[test]
fn d1_types_ts_declares_oracle_cache_source() {
    let src = read_src("dashboard-v2/src/lib/api/types.ts");
    assert!(
        src.contains("OracleCacheSource"),
        "types.ts must declare OracleCacheSource (I-15 cross-lang pair)"
    );
    assert!(
        src.contains("blake3_exact"),
        "OracleCacheSource must include \"blake3_exact\" arm"
    );
    assert!(
        src.contains("semantic_vector"),
        "OracleCacheSource must include \"semantic_vector\" arm"
    );
}

/// D-2: stub.ts declares get_research_cache_source function.
#[test]
fn d2_stub_ts_declares_get_research_cache_source() {
    let src = read_src("dashboard-v2/src/lib/api/stub.ts");
    assert!(
        src.contains("get_research_cache_source"),
        "stub.ts must declare get_research_cache_source (I-15 TS cross-lang pair)"
    );
    assert!(
        src.contains("OracleCacheSource"),
        "stub.ts must reference OracleCacheSource return type"
    );
}

/// D-3: ResearchDrawer.svelte imports get_research_cache_source and renders
/// the cache-source badge.
#[test]
fn d3_research_drawer_renders_cache_source_badge() {
    let src = read_src("dashboard-v2/src/lib/drawers/ResearchDrawer.svelte");
    assert!(
        src.contains("get_research_cache_source"),
        "ResearchDrawer.svelte must import get_research_cache_source (I-15 badge)"
    );
    assert!(
        src.contains("cache:exact") || src.contains("blake3_exact"),
        "ResearchDrawer.svelte must render the blake3_exact badge arm"
    );
    assert!(
        src.contains("cache:vec") || src.contains("semantic_vector"),
        "ResearchDrawer.svelte must render the semantic_vector badge arm"
    );
}
