// B-29 Phase 3 contract test.
//
// Locks the public interface of:
//   - SymbolGraphCache::new / meta / ensure_name_index / ensure_reverse_index /
//     get_or_load_payload / invalidate_file / file_payload_lru_size /
//     name_index_loaded / reverse_index_loaded / db_path / project_id
//   - looks_like_file_path / compute_impact_radius / compute_call_flow /
//     compute_symbol_context / list_symbols
//   - Output type field sets: ImpactResult, FlowNode, SymbolContext
//   - depth clamps to MAX_IMPACT_DEPTH / MAX_FLOW_DEPTH (assert <= 10)

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use elai_knowledge::dkg::DkgEngine;
use elai_knowledge::symbol_graph::cache::SymbolGraphCache;
use elai_knowledge::symbol_graph::sharding::{MAX_FLOW_DEPTH, MAX_IMPACT_DEPTH, reverse_shard_bucket};
use elai_knowledge::symbol_graph::store;
use elai_knowledge::symbol_graph::traversal::{
    self, CalleeInfo, CallerInfo, FlowNode, FlowTruncationReason, ImpactResult, SymbolContext,
    SymbolFilter,
};
use elai_knowledge::symbol_graph::types::{
    Confidence, SymbolGraphFilePayload, SymbolGraphMeta, SymbolKind, SymbolNode, SymbolRef,
};
use rusqlite::Connection;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Signature assertions (compile-time only)
// ---------------------------------------------------------------------------

/// This function never runs; it exists purely to assert that the public API
/// has the expected signatures at compile time.
#[allow(dead_code, unused_variables)]
fn _signature_check(cache: &mut SymbolGraphCache, p: &Path) {
    // cache.rs
    let _ = SymbolGraphCache::new(p, "project_id", 500_usize);
    let _meta: &SymbolGraphMeta = cache.meta();
    let _: Result<&BTreeMap<String, Vec<SymbolRef>>, _> = cache.ensure_name_index();
    let _: Result<&BTreeMap<String, BTreeSet<String>>, _> = cache.ensure_reverse_index();
    let _: Result<Option<std::sync::Arc<SymbolGraphFilePayload>>, _> =
        cache.get_or_load_payload("file.rs");
    cache.invalidate_file("file.rs");
    let _: usize = cache.file_payload_lru_size();
    let _: bool = cache.name_index_loaded();
    let _: bool = cache.reverse_index_loaded();
    let _: &Path = cache.db_path();
    let _: &str = cache.project_id();

    // traversal.rs
    let _: bool = traversal::looks_like_file_path("src/foo.rs");
    let _: Result<ImpactResult, _> = traversal::compute_impact_radius(cache, "foo", 5);
    let _: Result<Option<FlowNode>, _> = traversal::compute_call_flow(cache, "x::y#L1", 5);
    let _: Result<Vec<SymbolContext>, _> = traversal::compute_symbol_context(cache, "foo", None);
    let _: Result<Vec<SymbolNode>, _> = traversal::list_symbols(
        cache,
        SymbolFilter {
            file: None,
            query: None,
            limit: 10,
        },
    );
}

/// Field-set assertions for ImpactResult.
#[allow(dead_code)]
fn _impact_result_fields(r: ImpactResult) {
    let _target: String = r.target;
    let _resolved_files: Vec<String> = r.resolved_files;
    let _files_by_depth: BTreeMap<u32, Vec<String>> = r.files_by_depth;
    let _total_files: usize = r.total_files;
    let _max_depth_reached: u32 = r.max_depth_reached;
    let _truncated: bool = r.truncated;
}

/// Field-set assertions for FlowNode.
#[allow(dead_code)]
fn _flow_node_fields(n: FlowNode) {
    let _symbol_id: String = n.symbol_id;
    let _name: String = n.name;
    let _file: String = n.file;
    let _line: u32 = n.line;
    let _kind: SymbolKind = n.kind;
    let _children: Vec<FlowNode> = n.children;
    let _truncated_reason: Option<FlowTruncationReason> = n.truncated_reason;
}

/// Field-set assertions for SymbolContext, CallerInfo, CalleeInfo.
///
/// Locks `confidence: Confidence` (enum) — NOT String — for both CallerInfo
/// and CalleeInfo. Phase 3 auditor caught a String regression here; this
/// assertion ensures it cannot regress silently.
#[allow(dead_code)]
fn _symbol_context_fields(c: SymbolContext, caller: CallerInfo, callee: CalleeInfo) {
    let _definition: SymbolNode = c.definition;
    let _callers: Vec<CallerInfo> = c.callers;
    let _callees: Vec<CalleeInfo> = c.callees;

    // CallerInfo field types
    let _caller_id: String = caller.caller_id;
    let _callee_name: String = caller.callee_name;
    let _call_site_file: String = caller.call_site_file;
    let _call_site_line: u32 = caller.call_site_line;
    let _confidence: Confidence = caller.confidence;

    // CalleeInfo field types
    let _callee_id: String = callee.callee_id;
    let _callee_name2: String = callee.callee_name;
    let _confidence2: Confidence = callee.confidence;
}

// ---------------------------------------------------------------------------
// Constants locked to spec values
// ---------------------------------------------------------------------------

#[test]
fn max_impact_depth_is_10() {
    assert_eq!(MAX_IMPACT_DEPTH, 10);
}

#[test]
fn max_flow_depth_is_10() {
    assert_eq!(MAX_FLOW_DEPTH, 10);
}

// ---------------------------------------------------------------------------
// Fixture helpers (minimal DB for depth-clamp tests)
// ---------------------------------------------------------------------------

fn build_single_file_db(dir: &TempDir) -> std::path::PathBuf {
    let db_path = dir.path().join("dkg.db");
    let _e = DkgEngine::new(&db_path, None, 1).expect("DkgEngine");
    drop(_e);

    let mut conn = Connection::open(&db_path).expect("open");
    store::upsert_meta(
        &mut conn,
        &SymbolGraphMeta {
            project_id: "c3_proj".to_string(),
            symbol_count: 1,
            edge_count: 0,
            file_count: 1,
            unresolved_edge_pct: 0.0,
            built_at: 1,
            schema_version: 3,
        },
    )
    .expect("meta");

    store::upsert_file_payload(
        &mut conn,
        &SymbolGraphFilePayload {
            file: "solo.rs".to_string(),
            language: "rust".to_string(),
            content_hash: "hx".to_string(),
            symbols: vec![SymbolNode {
                id: "solo.rs::solo#L1".to_string(),
                name: "solo".to_string(),
                qualified_name: "solo".to_string(),
                kind: SymbolKind::Function,
                file: "solo.rs".to_string(),
                line: 1,
                end_line: 5,
                language: "rust".to_string(),
            }],
            outgoing_calls: vec![],
        },
    )
    .expect("payload");

    let bucket = reverse_shard_bucket("solo.rs");
    let mut rev: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    rev.insert("solo.rs".to_string(), BTreeSet::new());
    store::upsert_reverse_shard(&mut conn, bucket, &rev).expect("rev shard");

    let mut s_shard: BTreeMap<String, Vec<SymbolRef>> = BTreeMap::new();
    s_shard.insert(
        "solo".to_string(),
        vec![SymbolRef {
            file: "solo.rs".to_string(),
            id: "solo.rs::solo#L1".to_string(),
        }],
    );
    store::upsert_name_shard(&mut conn, 's', &s_shard).expect("shard s");

    db_path
}

// ---------------------------------------------------------------------------
// Depth clamp tests
// ---------------------------------------------------------------------------

#[test]
fn impact_radius_depth_999_clamped_to_max() {
    let dir = TempDir::new().expect("tempdir");
    let db_path = build_single_file_db(&dir);
    let mut cache = SymbolGraphCache::new(&db_path, "c3_proj", 100).expect("cache");

    let result = traversal::compute_impact_radius(&mut cache, "solo.rs", 999)
        .expect("impact_radius depth=999");

    assert!(
        result.max_depth_reached <= MAX_IMPACT_DEPTH,
        "max_depth_reached={} must be <= MAX_IMPACT_DEPTH={}",
        result.max_depth_reached,
        MAX_IMPACT_DEPTH
    );
}

#[test]
fn call_flow_depth_999_clamped_to_max() {
    let dir = TempDir::new().expect("tempdir");
    let db_path = build_single_file_db(&dir);
    let mut cache = SymbolGraphCache::new(&db_path, "c3_proj", 100).expect("cache");

    // solo has no outgoing calls, so depth doesn't matter — but the clamp
    // must not panic or overflow.
    let _result = traversal::compute_call_flow(&mut cache, "solo.rs::solo#L1", 999)
        .expect("call_flow depth=999 must not error");
    // No assertion on value; we only assert no panic + no error.
}

// ---------------------------------------------------------------------------
// looks_like_file_path signature + behaviour locked
// ---------------------------------------------------------------------------

#[test]
fn looks_like_file_path_pub_and_deterministic() {
    // Verify the function is pub and returns deterministic results.
    assert!(traversal::looks_like_file_path("src/foo.rs"));
    assert!(!traversal::looks_like_file_path("ClockProvider"));
    assert!(traversal::looks_like_file_path("/usr/bin/elai"));
    assert!(traversal::looks_like_file_path("foo.svelte"));
}
