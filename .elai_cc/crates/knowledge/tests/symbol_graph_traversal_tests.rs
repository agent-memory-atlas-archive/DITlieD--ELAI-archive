// B-29 Phase 3 — traversal integration tests.
//
// Builds a small fixture DB to test all 4 traversal functions + looks_like_file_path.
//
// Fixture topology:
//   A.rs defines `bar` (id: A.rs::bar#L1) which calls `foo`
//   B.rs defines `foo` (id: B.rs::foo#L1)
//   A.rs -> B.rs dependency (A imports B)
//   reverse_file_index: B.rs -> {A.rs}  (A depends on B)
//
// For cycle test: C.rs defines `ping` (calls `pong`), D.rs defines `pong` (calls `ping`).

use std::collections::{BTreeMap, BTreeSet};

use elai_knowledge::dkg::DkgEngine;
use elai_knowledge::symbol_graph::cache::SymbolGraphCache;
use elai_knowledge::symbol_graph::sharding::reverse_shard_bucket;
use elai_knowledge::symbol_graph::store;
use elai_knowledge::symbol_graph::traversal::{self, SymbolFilter};
use elai_knowledge::symbol_graph::types::{
    Confidence, SymbolEdge, SymbolGraphFilePayload, SymbolGraphMeta, SymbolKind, SymbolNode,
    SymbolRef, CallSite,
};
use rusqlite::Connection;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn sym(id: &str, name: &str, file: &str, line: u32, kind: SymbolKind) -> SymbolNode {
    SymbolNode {
        id: id.to_string(),
        name: name.to_string(),
        qualified_name: name.to_string(),
        kind,
        file: file.to_string(),
        line,
        end_line: line + 5,
        language: "rust".to_string(),
    }
}

fn edge_resolved(caller_id: &str, callee_name: &str, callee_id: &str, file: &str) -> SymbolEdge {
    SymbolEdge {
        caller_id: caller_id.to_string(),
        callee_name: callee_name.to_string(),
        callee_candidates: vec![callee_id.to_string()],
        confidence: Confidence::Unique,
        call_site: CallSite {
            file: file.to_string(),
            line: 5,
        },
    }
}

/// Build the standard A/B fixture DB plus cycle (C/D) files.
fn build_traversal_db(dir: &TempDir) -> std::path::PathBuf {
    let db_path = dir.path().join("dkg.db");
    let _engine = DkgEngine::new(&db_path, None, 1).expect("DkgEngine::new");
    drop(_engine);

    let mut conn = Connection::open(&db_path).expect("open");

    // Meta.
    store::upsert_meta(
        &mut conn,
        &SymbolGraphMeta {
            project_id: "trav_proj".to_string(),
            symbol_count: 6,
            edge_count: 4,
            file_count: 4,
            unresolved_edge_pct: 0.0,
            built_at: 1,
            schema_version: 3,
        },
    )
    .expect("meta");

    // A.rs: bar calls foo (->B.rs).
    let a_payload = SymbolGraphFilePayload {
        file: "A.rs".to_string(),
        language: "rust".to_string(),
        content_hash: "ha".to_string(),
        symbols: vec![sym("A.rs::bar#L1", "bar", "A.rs", 1, SymbolKind::Function)],
        outgoing_calls: vec![edge_resolved(
            "A.rs::bar#L1",
            "foo",
            "B.rs::foo#L1",
            "A.rs",
        )],
    };

    // B.rs: foo (no outgoing calls).
    let b_payload = SymbolGraphFilePayload {
        file: "B.rs".to_string(),
        language: "rust".to_string(),
        content_hash: "hb".to_string(),
        symbols: vec![sym("B.rs::foo#L1", "foo", "B.rs", 1, SymbolKind::Function)],
        outgoing_calls: vec![],
    };

    // C.rs: ping calls pong (->D.rs).
    let c_payload = SymbolGraphFilePayload {
        file: "C.rs".to_string(),
        language: "rust".to_string(),
        content_hash: "hc".to_string(),
        symbols: vec![sym("C.rs::ping#L1", "ping", "C.rs", 1, SymbolKind::Function)],
        outgoing_calls: vec![edge_resolved(
            "C.rs::ping#L1",
            "pong",
            "D.rs::pong#L1",
            "C.rs",
        )],
    };

    // D.rs: pong calls ping (->C.rs) — creates a cycle.
    let d_payload = SymbolGraphFilePayload {
        file: "D.rs".to_string(),
        language: "rust".to_string(),
        content_hash: "hd".to_string(),
        symbols: vec![sym("D.rs::pong#L1", "pong", "D.rs", 1, SymbolKind::Function)],
        outgoing_calls: vec![edge_resolved(
            "D.rs::pong#L1",
            "ping",
            "C.rs::ping#L1",
            "D.rs",
        )],
    };

    for p in &[&a_payload, &b_payload, &c_payload, &d_payload] {
        store::upsert_file_payload(&mut conn, p).expect("upsert payload");
    }

    // Name shards.
    let mut b_shard: BTreeMap<String, Vec<SymbolRef>> = BTreeMap::new();
    b_shard.insert(
        "bar".to_string(),
        vec![SymbolRef {
            file: "A.rs".to_string(),
            id: "A.rs::bar#L1".to_string(),
        }],
    );
    store::upsert_name_shard(&mut conn, 'b', &b_shard).expect("b shard");

    let mut f_shard: BTreeMap<String, Vec<SymbolRef>> = BTreeMap::new();
    f_shard.insert(
        "foo".to_string(),
        vec![SymbolRef {
            file: "B.rs".to_string(),
            id: "B.rs::foo#L1".to_string(),
        }],
    );
    store::upsert_name_shard(&mut conn, 'f', &f_shard).expect("f shard");

    let mut p_shard: BTreeMap<String, Vec<SymbolRef>> = BTreeMap::new();
    p_shard.insert(
        "ping".to_string(),
        vec![SymbolRef {
            file: "C.rs".to_string(),
            id: "C.rs::ping#L1".to_string(),
        }],
    );
    p_shard.insert(
        "pong".to_string(),
        vec![SymbolRef {
            file: "D.rs".to_string(),
            id: "D.rs::pong#L1".to_string(),
        }],
    );
    store::upsert_name_shard(&mut conn, 'p', &p_shard).expect("p shard");

    // Reverse shard: B.rs -> {A.rs}  (A depends on B).
    let b_bucket = reverse_shard_bucket("B.rs");
    let mut rev: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut callers_of_b = BTreeSet::new();
    callers_of_b.insert("A.rs".to_string());
    rev.insert("B.rs".to_string(), callers_of_b);
    store::upsert_reverse_shard(&mut conn, b_bucket, &rev).expect("reverse shard B");

    // Reverse shard: C.rs -> {D.rs}, D.rs -> {C.rs}  (cycle).
    let c_bucket = reverse_shard_bucket("C.rs");
    let d_bucket = reverse_shard_bucket("D.rs");

    let mut rev_c: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut callers_c = BTreeSet::new();
    callers_c.insert("D.rs".to_string());
    rev_c.insert("C.rs".to_string(), callers_c);

    let mut rev_d: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut callers_d = BTreeSet::new();
    callers_d.insert("C.rs".to_string());
    rev_d.insert("D.rs".to_string(), callers_d);

    if c_bucket == d_bucket {
        // Same bucket — merge.
        let mut merged: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for (k, v) in rev_c.iter().chain(rev_d.iter()) {
            merged
                .entry(k.clone())
                .or_default()
                .extend(v.iter().cloned());
        }
        store::upsert_reverse_shard(&mut conn, c_bucket, &merged).expect("merged cycle shard");
    } else {
        store::upsert_reverse_shard(&mut conn, c_bucket, &rev_c).expect("rev C shard");
        store::upsert_reverse_shard(&mut conn, d_bucket, &rev_d).expect("rev D shard");
    }

    db_path
}

fn make_cache(dir: &TempDir) -> SymbolGraphCache {
    let db_path = build_traversal_db(dir);
    SymbolGraphCache::new(&db_path, "trav_proj", 500).expect("cache::new")
}

// ---------------------------------------------------------------------------
// looks_like_file_path
// ---------------------------------------------------------------------------

#[test]
fn looks_like_file_path_true_for_paths_with_slash() {
    assert!(traversal::looks_like_file_path("src/foo.rs"));
    assert!(traversal::looks_like_file_path("/usr/bin/elai"));
    assert!(traversal::looks_like_file_path("a\\b\\c.ts"));
}

#[test]
fn looks_like_file_path_true_for_known_extensions() {
    assert!(traversal::looks_like_file_path("foo.svelte"));
    assert!(traversal::looks_like_file_path("index.ts"));
    assert!(traversal::looks_like_file_path("main.py"));
    assert!(traversal::looks_like_file_path("app.js"));
    assert!(traversal::looks_like_file_path("Main.java"));
    assert!(traversal::looks_like_file_path("Foo.kt"));
    assert!(traversal::looks_like_file_path("server.go"));
    assert!(traversal::looks_like_file_path("widget.vue"));
    assert!(traversal::looks_like_file_path("mod.rs"));
}

#[test]
fn looks_like_file_path_false_for_plain_symbol_names() {
    assert!(!traversal::looks_like_file_path("ClockProvider"));
    assert!(!traversal::looks_like_file_path("compute_impact_radius"));
    assert!(!traversal::looks_like_file_path("FooBarBaz"));
    assert!(!traversal::looks_like_file_path("SOME_CONSTANT"));
}

// ---------------------------------------------------------------------------
// compute_impact_radius — file target
// ---------------------------------------------------------------------------

#[test]
fn impact_radius_file_target_b_rs() {
    let dir = TempDir::new().expect("tempdir");
    let mut cache = make_cache(&dir);

    let result = traversal::compute_impact_radius(&mut cache, "B.rs", 2)
        .expect("compute_impact_radius");

    // B.rs seed = layer 0; A.rs is layer 1 (caller of B.rs).
    assert_eq!(result.target, "B.rs");
    assert_eq!(result.total_files, 2, "B.rs + A.rs");
    assert!(result.files_by_depth.contains_key(&0));
    assert!(result.files_by_depth.contains_key(&1));
    assert!(result.files_by_depth[&0].contains(&"B.rs".to_string()));
    assert!(result.files_by_depth[&1].contains(&"A.rs".to_string()));
    assert!(!result.truncated, "no further callers of A.rs");
}

#[test]
fn impact_radius_depth_zero_returns_seed_only() {
    let dir = TempDir::new().expect("tempdir");
    let mut cache = make_cache(&dir);

    let result = traversal::compute_impact_radius(&mut cache, "B.rs", 0)
        .expect("compute_impact_radius depth=0");

    assert_eq!(result.total_files, 1, "depth=0: only seed file");
    assert!(result.files_by_depth[&0].contains(&"B.rs".to_string()));
}

// ---------------------------------------------------------------------------
// compute_impact_radius — symbol target
// ---------------------------------------------------------------------------

#[test]
fn impact_radius_symbol_target_foo() {
    let dir = TempDir::new().expect("tempdir");
    let mut cache = make_cache(&dir);

    // "foo" resolves to B.rs; then BFS finds A.rs as caller.
    let result = traversal::compute_impact_radius(&mut cache, "foo", 2)
        .expect("compute_impact_radius symbol foo");

    assert_eq!(result.total_files, 2);
    assert!(result.files_by_depth[&0].contains(&"B.rs".to_string()));
    assert!(result.files_by_depth[&1].contains(&"A.rs".to_string()));
}

#[test]
fn impact_radius_nonexistent_symbol_returns_empty() {
    let dir = TempDir::new().expect("tempdir");
    let mut cache = make_cache(&dir);

    let result = traversal::compute_impact_radius(&mut cache, "ghost_symbol", 5)
        .expect("no error for unknown symbol");

    assert_eq!(result.total_files, 0);
    assert!(result.files_by_depth.is_empty());
}

// ---------------------------------------------------------------------------
// compute_call_flow
// ---------------------------------------------------------------------------

#[test]
fn call_flow_a_to_b_one_hop() {
    let dir = TempDir::new().expect("tempdir");
    let mut cache = make_cache(&dir);

    // bar in A.rs calls foo in B.rs. Flow: bar -> foo (leaf).
    let root = traversal::compute_call_flow(&mut cache, "A.rs::bar#L1", 5)
        .expect("compute_call_flow")
        .expect("Some");

    assert_eq!(root.symbol_id, "A.rs::bar#L1");
    assert_eq!(root.children.len(), 1);
    assert_eq!(root.children[0].symbol_id, "B.rs::foo#L1");
    assert!(root.truncated_reason.is_none());
    assert!(root.children[0].children.is_empty());
}

#[test]
fn call_flow_unknown_id_returns_none() {
    let dir = TempDir::new().expect("tempdir");
    let mut cache = make_cache(&dir);

    let result = traversal::compute_call_flow(&mut cache, "ghost.rs::ghost#L1", 5)
        .expect("no error");
    assert!(result.is_none());
}

#[test]
fn call_flow_cycle_detected() {
    let dir = TempDir::new().expect("tempdir");
    let mut cache = make_cache(&dir);

    // ping -> pong -> ping (cycle).
    let root = traversal::compute_call_flow(&mut cache, "C.rs::ping#L1", 5)
        .expect("call_flow")
        .expect("Some");

    assert_eq!(root.symbol_id, "C.rs::ping#L1");
    assert_eq!(root.children.len(), 1);
    let pong_node = &root.children[0];
    assert_eq!(pong_node.symbol_id, "D.rs::pong#L1");

    // pong should have one child (ping back-edge) that is truncated with Cycle.
    assert_eq!(pong_node.children.len(), 1);
    let back_edge = &pong_node.children[0];
    assert_eq!(back_edge.symbol_id, "C.rs::ping#L1");
    assert!(
        matches!(
            back_edge.truncated_reason,
            Some(traversal::FlowTruncationReason::Cycle)
        ),
        "cycle back-edge must have TruncationReason::Cycle"
    );
}

#[test]
fn call_flow_depth_truncation() {
    let dir = TempDir::new().expect("tempdir");
    let mut cache = make_cache(&dir);

    // depth=0: entrypoint itself returned with Depth truncation.
    let root = traversal::compute_call_flow(&mut cache, "A.rs::bar#L1", 0)
        .expect("call_flow depth=0")
        .expect("Some");

    assert_eq!(root.symbol_id, "A.rs::bar#L1");
    assert!(
        matches!(
            root.truncated_reason,
            Some(traversal::FlowTruncationReason::Depth)
        ),
        "depth=0 must produce Depth truncation at root"
    );
    assert!(root.children.is_empty());
}

// ---------------------------------------------------------------------------
// compute_symbol_context
// ---------------------------------------------------------------------------

#[test]
fn symbol_context_foo_has_caller_bar() {
    let dir = TempDir::new().expect("tempdir");
    let mut cache = make_cache(&dir);

    let contexts = traversal::compute_symbol_context(&mut cache, "foo", None)
        .expect("compute_symbol_context");

    assert_eq!(contexts.len(), 1, "one definition of foo");
    let ctx = &contexts[0];
    assert_eq!(ctx.definition.name, "foo");
    // bar calls foo -> 1 caller.
    assert_eq!(ctx.callers.len(), 1, "foo has 1 caller (bar)");
    assert_eq!(ctx.callers[0].caller_id, "A.rs::bar#L1");
    // foo has no outgoing calls.
    assert_eq!(ctx.callees.len(), 0, "foo has no callees");
}

#[test]
fn symbol_context_bar_has_callee_foo() {
    let dir = TempDir::new().expect("tempdir");
    let mut cache = make_cache(&dir);

    let contexts = traversal::compute_symbol_context(&mut cache, "bar", None)
        .expect("symbol_context bar");

    assert_eq!(contexts.len(), 1);
    let ctx = &contexts[0];
    assert_eq!(ctx.definition.name, "bar");
    assert_eq!(ctx.callees.len(), 1, "bar calls foo");
    assert_eq!(ctx.callees[0].callee_id, "B.rs::foo#L1");
}

#[test]
fn symbol_context_with_file_hint_filters() {
    let dir = TempDir::new().expect("tempdir");
    let mut cache = make_cache(&dir);

    // file_hint filters to only B.rs.
    let contexts = traversal::compute_symbol_context(&mut cache, "foo", Some("B.rs"))
        .expect("symbol_context foo file_hint");

    assert_eq!(contexts.len(), 1);
    assert_eq!(contexts[0].definition.file, "B.rs");
}

// ---------------------------------------------------------------------------
// list_symbols
// ---------------------------------------------------------------------------

#[test]
fn list_symbols_by_file_returns_symbols_in_file() {
    let dir = TempDir::new().expect("tempdir");
    let mut cache = make_cache(&dir);

    let filter = SymbolFilter {
        file: Some("A.rs".to_string()),
        query: None,
        limit: 10,
    };
    let syms = traversal::list_symbols(&mut cache, filter).expect("list_symbols A.rs");
    assert_eq!(syms.len(), 1, "A.rs has 1 non-Module symbol (bar)");
    assert_eq!(syms[0].name, "bar");
}

#[test]
fn list_symbols_query_filter_works() {
    let dir = TempDir::new().expect("tempdir");
    let mut cache = make_cache(&dir);

    let filter = SymbolFilter {
        file: Some("A.rs".to_string()),
        query: Some("ba".to_string()),
        limit: 10,
    };
    let syms = traversal::list_symbols(&mut cache, filter).expect("list_symbols query=ba");
    assert_eq!(syms.len(), 1);
    assert_eq!(syms[0].name, "bar");
}

#[test]
fn list_symbols_limit_clamps_results() {
    let dir = TempDir::new().expect("tempdir");
    let mut cache = make_cache(&dir);

    // Scan all files, limit=1.
    let filter = SymbolFilter {
        file: None,
        query: None,
        limit: 1,
    };
    let syms = traversal::list_symbols(&mut cache, filter).expect("list_symbols limit=1");
    assert!(syms.len() <= 1, "must not exceed limit");
}

#[test]
fn list_symbols_zero_limit_defaults_to_100() {
    let dir = TempDir::new().expect("tempdir");
    let mut cache = make_cache(&dir);

    // limit=0 must not return 0 (would be useless); must apply default cap (100).
    let filter = SymbolFilter {
        file: None,
        query: None,
        limit: 0,
    };
    let syms = traversal::list_symbols(&mut cache, filter).expect("list_symbols limit=0");
    // We have 4 symbols across all files; all must be returned.
    assert!(
        !syms.is_empty(),
        "limit=0 must default to 100, not return empty"
    );
    assert!(syms.len() <= 100, "must not exceed default cap 100");
}
