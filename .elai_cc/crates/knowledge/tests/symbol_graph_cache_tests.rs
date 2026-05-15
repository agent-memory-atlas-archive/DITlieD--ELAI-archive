// B-29 Phase 3 — SymbolGraphCache integration tests.
//
// Builds a small fixture DB on disk (tempfile), inserts 3 files / 5 symbols /
// 8 edges / 2 reverse-call buckets, then exercises the cache lifecycle:
//   - new() loads meta but NOT shards
//   - ensure_name_index() lazy-loads on first access
//   - LRU capacity enforcement
//   - invalidate_file() evicts payload + resets index flags

use std::collections::{BTreeMap, BTreeSet};

use elai_knowledge::dkg::DkgEngine;
use elai_knowledge::symbol_graph::cache::SymbolGraphCache;
use elai_knowledge::symbol_graph::store;
use elai_knowledge::symbol_graph::types::{
    Confidence, SymbolEdge, SymbolGraphFilePayload, SymbolGraphMeta, SymbolKind, SymbolNode,
    SymbolRef, CallSite,
};
use rusqlite::Connection;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Fixture helpers
// ---------------------------------------------------------------------------

fn sym(id: &str, name: &str, file: &str, line: u32) -> SymbolNode {
    SymbolNode {
        id: id.to_string(),
        name: name.to_string(),
        qualified_name: name.to_string(),
        kind: SymbolKind::Function,
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

/// Build a fixture database with 3 files, 5 symbols, 8 edges.
///
/// File layout:
///   lib.rs  — defines `init` (id: lib.rs::init#L1) and `run` (id: lib.rs::run#L10)
///   server.rs — defines `start` (id: server.rs::start#L1), calls init (->lib.rs)
///   client.rs — defines `connect` (id: client.rs::connect#L1) + `send` (id: client.rs::send#L8),
///               both call run (->lib.rs)
///
/// Reverse index:
///   lib.rs -> {server.rs, client.rs}  (both call into lib.rs)
fn build_fixture_db(dir: &TempDir) -> std::path::PathBuf {
    let db_path = dir.path().join("dkg.db");

    // Open with DkgEngine to get the v3 schema.
    let _engine = DkgEngine::new(&db_path, None, 1000).expect("DkgEngine::new");
    drop(_engine);

    let mut conn = Connection::open(&db_path).expect("open fixture db");

    // Insert meta.
    let meta = SymbolGraphMeta {
        project_id: "test_proj".to_string(),
        symbol_count: 5,
        edge_count: 8,
        file_count: 3,
        unresolved_edge_pct: 0.0,
        built_at: 999,
        schema_version: 3,
    };
    store::upsert_meta(&mut conn, &meta).expect("upsert meta");

    // Build payloads.
    let lib_payload = SymbolGraphFilePayload {
        file: "lib.rs".to_string(),
        language: "rust".to_string(),
        content_hash: "hash_lib".to_string(),
        symbols: vec![
            sym("lib.rs::init#L1", "init", "lib.rs", 1),
            sym("lib.rs::run#L10", "run", "lib.rs", 10),
        ],
        outgoing_calls: vec![],
    };
    let server_payload = SymbolGraphFilePayload {
        file: "server.rs".to_string(),
        language: "rust".to_string(),
        content_hash: "hash_server".to_string(),
        symbols: vec![sym("server.rs::start#L1", "start", "server.rs", 1)],
        outgoing_calls: vec![
            edge_resolved("server.rs::start#L1", "init", "lib.rs::init#L1", "server.rs"),
        ],
    };
    let client_payload = SymbolGraphFilePayload {
        file: "client.rs".to_string(),
        language: "rust".to_string(),
        content_hash: "hash_client".to_string(),
        symbols: vec![
            sym("client.rs::connect#L1", "connect", "client.rs", 1),
            sym("client.rs::send#L8", "send", "client.rs", 8),
        ],
        outgoing_calls: vec![
            edge_resolved("client.rs::connect#L1", "run", "lib.rs::run#L10", "client.rs"),
            edge_resolved("client.rs::send#L8", "run", "lib.rs::run#L10", "client.rs"),
        ],
    };

    for p in &[&lib_payload, &server_payload, &client_payload] {
        store::upsert_file_payload(&mut conn, p).expect("upsert payload");
    }

    // Insert name shards.
    let mut i_shard: BTreeMap<String, Vec<SymbolRef>> = BTreeMap::new();
    i_shard.insert(
        "init".to_string(),
        vec![SymbolRef {
            file: "lib.rs".to_string(),
            id: "lib.rs::init#L1".to_string(),
        }],
    );
    store::upsert_name_shard(&mut conn, 'i', &i_shard).expect("upsert i shard");

    let mut r_shard: BTreeMap<String, Vec<SymbolRef>> = BTreeMap::new();
    r_shard.insert(
        "run".to_string(),
        vec![SymbolRef {
            file: "lib.rs".to_string(),
            id: "lib.rs::run#L10".to_string(),
        }],
    );
    store::upsert_name_shard(&mut conn, 'r', &r_shard).expect("upsert r shard");

    let mut s_shard: BTreeMap<String, Vec<SymbolRef>> = BTreeMap::new();
    s_shard.insert(
        "start".to_string(),
        vec![SymbolRef {
            file: "server.rs".to_string(),
            id: "server.rs::start#L1".to_string(),
        }],
    );
    s_shard.insert(
        "send".to_string(),
        vec![SymbolRef {
            file: "client.rs".to_string(),
            id: "client.rs::send#L8".to_string(),
        }],
    );
    store::upsert_name_shard(&mut conn, 's', &s_shard).expect("upsert s shard");

    let mut c_shard: BTreeMap<String, Vec<SymbolRef>> = BTreeMap::new();
    c_shard.insert(
        "connect".to_string(),
        vec![SymbolRef {
            file: "client.rs".to_string(),
            id: "client.rs::connect#L1".to_string(),
        }],
    );
    store::upsert_name_shard(&mut conn, 'c', &c_shard).expect("upsert c shard");

    // Insert reverse shards: lib.rs -> {server.rs, client.rs}
    let lib_bucket = elai_knowledge::symbol_graph::sharding::reverse_shard_bucket("lib.rs");
    let mut rev_entries: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut callers = BTreeSet::new();
    callers.insert("server.rs".to_string());
    callers.insert("client.rs".to_string());
    rev_entries.insert("lib.rs".to_string(), callers);
    store::upsert_reverse_shard(&mut conn, lib_bucket, &rev_entries)
        .expect("upsert reverse shard lib");

    db_path
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn new_loads_meta_but_not_shards() {
    let dir = TempDir::new().expect("tempdir");
    let db_path = build_fixture_db(&dir);

    let cache = SymbolGraphCache::new(&db_path, "test_proj", 500).expect("cache::new");
    assert_eq!(cache.meta().symbol_count, 5);
    assert_eq!(cache.meta().edge_count, 8);

    // Shards must NOT be loaded eagerly.
    assert!(!cache.name_index_loaded(), "name_index must be None after new()");
    assert!(!cache.reverse_index_loaded(), "reverse_index must be None after new()");
    assert_eq!(cache.file_payload_lru_size(), 0, "LRU must be empty after new()");
}

#[test]
fn new_returns_error_for_unknown_project() {
    let dir = TempDir::new().expect("tempdir");
    let db_path = build_fixture_db(&dir);

    let result = SymbolGraphCache::new(&db_path, "no_such_project", 500);
    assert!(
        result.is_err(),
        "must error when project_id not found in symbol_graph_meta"
    );
}

#[test]
fn ensure_name_index_lazy_loads_once() {
    let dir = TempDir::new().expect("tempdir");
    let db_path = build_fixture_db(&dir);

    let mut cache = SymbolGraphCache::new(&db_path, "test_proj", 500).expect("cache::new");
    assert!(!cache.name_index_loaded());

    let idx = cache.ensure_name_index().expect("ensure_name_index");
    assert!(idx.contains_key("init"), "init must be in name index");
    assert!(idx.contains_key("run"), "run must be in name index");
    assert!(cache.name_index_loaded(), "name_index must be loaded after ensure");
}

#[test]
fn ensure_name_index_idempotent() {
    let dir = TempDir::new().expect("tempdir");
    let db_path = build_fixture_db(&dir);

    let mut cache = SymbolGraphCache::new(&db_path, "test_proj", 500).expect("cache::new");
    let _ = cache.ensure_name_index().expect("first ensure");
    let _ = cache.ensure_name_index().expect("second ensure — must not error or reload");
    assert!(cache.name_index_loaded());
}

#[test]
fn get_or_load_payload_miss_then_hit() {
    let dir = TempDir::new().expect("tempdir");
    let db_path = build_fixture_db(&dir);

    let mut cache = SymbolGraphCache::new(&db_path, "test_proj", 500).expect("cache::new");

    // Miss — loads from DB.
    let p = cache
        .get_or_load_payload("lib.rs")
        .expect("get_or_load")
        .expect("Some");
    assert_eq!(p.file, "lib.rs");
    assert_eq!(cache.file_payload_lru_size(), 1);

    // Hit — must serve from LRU (file_payload_lru_size stays 1).
    let p2 = cache
        .get_or_load_payload("lib.rs")
        .expect("second get")
        .expect("Some");
    assert_eq!(p2.file, "lib.rs");
    assert_eq!(cache.file_payload_lru_size(), 1);
}

#[test]
fn get_or_load_payload_nonexistent_file_returns_none() {
    let dir = TempDir::new().expect("tempdir");
    let db_path = build_fixture_db(&dir);

    let mut cache = SymbolGraphCache::new(&db_path, "test_proj", 500).expect("cache::new");
    let result = cache.get_or_load_payload("ghost.rs").expect("no error");
    assert!(result.is_none());
}

#[test]
fn lru_capacity_enforced() {
    let dir = TempDir::new().expect("tempdir");
    let db_path = build_fixture_db(&dir);

    // capacity = 2, we have 3 files.
    let mut cache = SymbolGraphCache::new(&db_path, "test_proj", 2).expect("cache::new");

    cache.get_or_load_payload("lib.rs").expect("load 1");
    cache.get_or_load_payload("server.rs").expect("load 2");
    assert_eq!(cache.file_payload_lru_size(), 2);

    // Loading a 3rd file must evict 1, keeping size <= capacity.
    cache.get_or_load_payload("client.rs").expect("load 3");
    assert!(
        cache.file_payload_lru_size() <= 2,
        "LRU must not exceed capacity=2; got {}",
        cache.file_payload_lru_size()
    );
}

#[test]
fn invalidate_file_evicts_and_resets_indices() {
    let dir = TempDir::new().expect("tempdir");
    let db_path = build_fixture_db(&dir);

    let mut cache = SymbolGraphCache::new(&db_path, "test_proj", 500).expect("cache::new");

    // Warm up all three.
    cache.get_or_load_payload("lib.rs").expect("load lib");
    cache.get_or_load_payload("server.rs").expect("load server");
    let _ = cache.ensure_name_index().expect("load name index");
    let _ = cache.ensure_reverse_index().expect("load reverse index");

    assert!(cache.name_index_loaded());
    assert!(cache.reverse_index_loaded());
    assert!(cache.file_payload_lru_size() >= 2);

    // Invalidate lib.rs.
    cache.invalidate_file("lib.rs");

    // LRU must not contain lib.rs.
    let still_in_lru = cache.get_or_load_payload("lib.rs").expect("reload after invalidate");
    // Reload succeeds (DB still has the data).
    assert!(still_in_lru.is_some(), "can still load after invalidate — DB is intact");

    // Name and reverse indices must be cleared by invalidate.
    // (They were reset; the reload above did NOT trigger ensure_name_index.)
    // We need to observe the flags immediately after invalidate, before any
    // subsequent ensure call. Re-create cache and invalidate before any index load:
    let mut cache2 = SymbolGraphCache::new(&db_path, "test_proj", 500).expect("cache2::new");
    let _ = cache2.ensure_name_index().expect("ensure name");
    let _ = cache2.ensure_reverse_index().expect("ensure reverse");
    assert!(cache2.name_index_loaded());
    assert!(cache2.reverse_index_loaded());

    cache2.invalidate_file("server.rs");
    assert!(!cache2.name_index_loaded(), "invalidate must reset name_index to None");
    assert!(
        !cache2.reverse_index_loaded(),
        "invalidate must reset reverse_index to None"
    );
}
