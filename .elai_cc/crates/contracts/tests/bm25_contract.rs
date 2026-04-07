//! Contract tests for the BM25 indexer module.
//! Compile-only stubs — auditor fills implementations in 1B.3.

use elai_knowledge::bm25::{Bm25Index, BM25Chunk, BM25Hit, ChunkKind, IndexStats};
use std::path::Path;

#[test]
#[ignore] // 1B.3: auditor fills
fn new_creates_index_directory() {
}

#[test]
#[ignore] // 1B.3: auditor fills
fn index_file_returns_chunk_count() {
}

#[test]
#[ignore] // 1B.3: auditor fills
fn index_directory_returns_stats() {
}

#[test]
#[ignore] // 1B.3: auditor fills
fn query_returns_hits_sorted_by_score() {
}

#[test]
#[ignore] // 1B.3: auditor fills
fn delete_file_removes_all_chunks() {
}

#[test]
#[ignore] // 1B.3: auditor fills
fn update_file_replaces_chunks() {
}

#[test]
#[ignore] // 1B.3: auditor fills
fn binary_file_skipped_in_index_directory() {
}

#[test]
#[ignore] // 1B.3: auditor fills
fn empty_query_returns_empty_results() {
}

#[test]
#[ignore] // 1B.3: auditor fills
fn chunk_kind_variants_serializable() {
}

#[test]
#[ignore] // 1B.3: auditor fills
fn index_stats_fields_populated() {
}
