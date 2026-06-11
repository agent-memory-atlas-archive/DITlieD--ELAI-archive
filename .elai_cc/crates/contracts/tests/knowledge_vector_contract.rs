#![allow(unused_imports, dead_code)]
// Compile-time interface assertions for vector module types and behavioral
// contract tests for dual-vector output and project isolation.

use rusqlite::Connection;

use elai_knowledge::vector::cache::{CacheError, VectorCache};
use elai_knowledge::vector::circuit_breaker::{BreakerError, BreakerState, EmbeddingCircuitBreaker};
use elai_knowledge::vector::hnsw_index::{IndexError, VectorIndex};
use elai_knowledge::vector::ort_session::{OrtError, OrtSession};
use elai_knowledge::vector::{
    DualVector, SearchMode, VectorError, VectorModule, CODE_VEC_DIM, TEXT_VEC_DIM,
};
use elai_knowledge::ReadinessStatus;

// -------------------------------------------------------------------------
// Compile-time type assertions
// -------------------------------------------------------------------------

// DualVector::new validates dimensions
fn _assert_dual_vector_new() {
    let code = vec![0.0f32; CODE_VEC_DIM];
    let text = vec![0.0f32; TEXT_VEC_DIM];
    let _: Result<DualVector, VectorError> = DualVector::new(code, text);
}

// VectorCache API surface
fn _assert_vector_cache_api(cache: &VectorCache) {
    let hash = VectorCache::content_hash(b"test");
    let code = vec![0.0f32; CODE_VEC_DIM];
    let text = vec![0.0f32; TEXT_VEC_DIM];
    let dv = DualVector::new(code, text).unwrap();
    let _: Result<(), CacheError> = cache.insert(&hash, "src/contract.rs", &dv, 0);
    let _: Result<Option<DualVector>, CacheError> = cache.lookup(&hash);
    let _: Result<usize, CacheError> = cache.invalidate_keys(&[&hash[..]]);
    let _: Result<usize, CacheError> = cache.invalidate_file("path");
}

// SearchMode variants exist
fn _assert_search_mode_variants() {
    let _ = SearchMode::Hybrid;
    let _ = SearchMode::CodeOnly;
    let _ = SearchMode::TextOnly;
    let _ = SearchMode::SparseOnly;
}

// VectorModule trait has required methods
fn _assert_vector_module_trait<T: VectorModule>(m: &T) {
    let _: Result<Vec<f32>, VectorError> = m.embed_code("test");
    let _: Result<Vec<f32>, VectorError> = m.embed_text("test");
    let _: ReadinessStatus = m.readiness();
}

// OrtSession API
fn _assert_ort_session_api() {
    let _: usize = CODE_VEC_DIM;
    let _: usize = TEXT_VEC_DIM;
    // OrtSession::new returns Result, not panic
    let p = std::path::PathBuf::from("/nonexistent");
    let t = std::path::PathBuf::from("/nonexistent/tok.json");
    let _: Result<OrtSession, OrtError> = OrtSession::new(&p, &t, 512, 768);
}

// CircuitBreaker API
fn _assert_circuit_breaker_api(cb: &EmbeddingCircuitBreaker) {
    let _: BreakerState = cb.state();
}

fn _assert_circuit_breaker_mut_api(cb: &mut EmbeddingCircuitBreaker) {
    cb.sync_state();
}

// BreakerState variants
fn _assert_breaker_state_variants() {
    let _ = BreakerState::Closed;
    let _ = BreakerState::Open;
    let _ = BreakerState::HalfOpen;
}

// ReadinessStatus variants
fn _assert_readiness_status_variants() {
    let _ = ReadinessStatus::Ready;
    let _ = ReadinessStatus::Degraded;
    let _ = ReadinessStatus::Unavailable;
}

// -------------------------------------------------------------------------
// Behavioral contract tests
// -------------------------------------------------------------------------

#[test]
fn test_query_returns_dual_vector_results() {
    // Contract: query(strategy=HYBRID) produces results where each chunk has
    // both code_score and text_score. Since KnowledgeEngine is not yet fully
    // implemented, we verify the underlying VectorCache stores and returns
    // DualVectors with both code_vec and text_vec populated.
    let conn = Connection::open_in_memory().unwrap();
    let cache = VectorCache::new(conn).unwrap();

    let code = vec![0.42f32; CODE_VEC_DIM];
    let text = vec![0.84f32; TEXT_VEC_DIM];
    let dv = DualVector::new(code.clone(), text.clone()).unwrap();

    let hash = VectorCache::content_hash(b"fn example() { /* hybrid query target */ }");
    cache.insert(&hash, "src/example.rs", &dv, 1700000000).unwrap();

    let found = cache.lookup(&hash).unwrap().expect("cache should return stored entry");
    // Both vectors must be populated (contract for hybrid search)
    assert_eq!(found.code_vec.len(), CODE_VEC_DIM, "code_vec dimension");
    assert_eq!(found.text_vec.len(), TEXT_VEC_DIM, "text_vec dimension");
    assert_eq!(found.code_vec, code, "code_vec values roundtrip");
    assert_eq!(found.text_vec, text, "text_vec values roundtrip");
}

#[test]
fn test_query_project_isolation() {
    // Contract: knowledge.txt Section 13 requires physical file isolation
    // (separate knowledge.db per project). This test verifies that two
    // separate in-memory databases (simulating per-project DBs) do NOT
    // share state. A single DB with project_id filtering is a spec violation.

    // Project A's database
    let conn_a = Connection::open_in_memory().unwrap();
    let cache_a = VectorCache::new(conn_a).unwrap();

    // Project B's database (separate in-memory DB — physically isolated)
    let conn_b = Connection::open_in_memory().unwrap();
    let cache_b = VectorCache::new(conn_b).unwrap();

    // Insert into project A
    let code = vec![1.0f32; CODE_VEC_DIM];
    let text = vec![2.0f32; TEXT_VEC_DIM];
    let dv = DualVector::new(code, text).unwrap();
    let hash = VectorCache::content_hash(b"fn project_a_only() {}");
    cache_a.insert(&hash, "src/project_a.rs", &dv, 1700000000).unwrap();

    // Verify project A can find it
    let found_a = cache_a.lookup(&hash).unwrap();
    assert!(found_a.is_some(), "project A should find its own data");

    // Verify project B cannot find it (physical isolation)
    let found_b = cache_b.lookup(&hash).unwrap();
    assert!(found_b.is_none(), "project B must NOT see project A's data (physical isolation)");
}
