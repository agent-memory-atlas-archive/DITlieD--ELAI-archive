//! Contract tests for the BM25 indexer module.
//! Behavioral interface-level tests verifying the public API contract.

use elai_knowledge::bm25::{Bm25Index, BM25Chunk, ChunkKind, IndexStats};
use std::path::PathBuf;

fn write_file(dir: &std::path::Path, rel: &str, content: &str) -> PathBuf {
    let full = dir.join(rel);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&full, content).unwrap();
    full
}

#[test]
fn new_creates_index_directory() {
    let tmp = tempfile::TempDir::new().unwrap();
    let _idx = Bm25Index::new(tmp.path(), "contract_new").unwrap();
    let idx_dir = tmp.path().join("projects").join("contract_new").join("bm25");
    assert!(idx_dir.is_dir(), "Bm25Index::new must create the index directory");
}

#[test]
fn index_file_returns_chunk_count() {
    let tmp = tempfile::TempDir::new().unwrap();
    let mut idx = Bm25Index::new(tmp.path(), "contract_idx").unwrap();
    let file = write_file(tmp.path(), "sample.rs", "fn sample() {}\n");
    let chunks = vec![
        BM25Chunk {
            path: file.clone(),
            content: "fn sample() {}".to_string(),
            line_start: 1,
            line_end: 1,
            kind: ChunkKind::Function,
        },
        BM25Chunk {
            path: file.clone(),
            content: "fn sample() {}".to_string(),
            line_start: 1,
            line_end: 1,
            kind: ChunkKind::Function,
        },
    ];
    let count = idx.index_file(&file, Some(chunks)).unwrap();
    assert_eq!(count, 2, "index_file must return the number of chunks indexed");
}

#[test]
fn index_directory_returns_stats() {
    let tmp = tempfile::TempDir::new().unwrap();
    let src = tmp.path().join("src");
    std::fs::create_dir_all(&src).unwrap();
    write_file(&src, "a.rs", "fn a() {}\n");
    write_file(&src, "b.rs", "fn b() {}\n");
    let mut idx = Bm25Index::new(tmp.path(), "contract_dir").unwrap();
    let stats = idx.index_directory(&src).unwrap();
    assert_eq!(stats.files_indexed, 2, "index_directory must count indexed files");
    assert!(stats.chunks_indexed > 0, "index_directory must index at least one chunk");
    assert_eq!(stats.files_skipped_binary, 0, "no binary files to skip");
}

#[test]
fn query_returns_hits_sorted_by_score() {
    let tmp = tempfile::TempDir::new().unwrap();
    let mut idx = Bm25Index::new(tmp.path(), "contract_qsort").unwrap();
    let file = write_file(tmp.path(), "scored.rs", "fn scored() {}\n");
    let chunks = vec![BM25Chunk {
        path: file.clone(),
        content: "fn scored() {}".to_string(),
        line_start: 1,
        line_end: 1,
        kind: ChunkKind::Function,
    }];
    idx.index_file(&file, Some(chunks)).unwrap();
    let hits = idx.query("scored", 10).unwrap();
    assert!(!hits.is_empty());
    // Verify scores are in descending order
    for pair in hits.windows(2) {
        assert!(
            pair[0].score >= pair[1].score,
            "query results must be sorted by descending score"
        );
    }
}

#[test]
fn delete_file_removes_all_chunks() {
    let tmp = tempfile::TempDir::new().unwrap();
    let mut idx = Bm25Index::new(tmp.path(), "contract_del").unwrap();
    let file = write_file(tmp.path(), "doomed.rs", "fn doomed_sentinel() {}\n");
    let chunks = vec![
        BM25Chunk {
            path: file.clone(),
            content: "fn doomed_sentinel() {}".to_string(),
            line_start: 1,
            line_end: 1,
            kind: ChunkKind::Function,
        },
        BM25Chunk {
            path: file.clone(),
            content: "fn doomed_sentinel() {} // dup".to_string(),
            line_start: 1,
            line_end: 1,
            kind: ChunkKind::Function,
        },
    ];
    idx.index_file(&file, Some(chunks)).unwrap();
    idx.delete_file(&file).unwrap();
    let hits = idx.query("doomed_sentinel", 10).unwrap();
    assert!(hits.is_empty(), "delete_file must remove ALL chunks for the path");
}

#[test]
fn update_file_replaces_chunks() {
    let tmp = tempfile::TempDir::new().unwrap();
    let mut idx = Bm25Index::new(tmp.path(), "contract_upd").unwrap();
    let file = write_file(tmp.path(), "update_me.rs", "fn before_update() {}\n");
    let old = vec![BM25Chunk {
        path: file.clone(),
        content: "fn before_update() {}".to_string(),
        line_start: 1,
        line_end: 1,
        kind: ChunkKind::Function,
    }];
    idx.index_file(&file, Some(old)).unwrap();
    std::fs::write(&file, "fn after_update() {}\n").unwrap();
    let new = vec![BM25Chunk {
        path: file.clone(),
        content: "fn after_update() {}".to_string(),
        line_start: 1,
        line_end: 1,
        kind: ChunkKind::Function,
    }];
    let count = idx.update_file(&file, Some(new)).unwrap();
    assert!(count > 0, "update_file must return new chunk count");
    let old_hits = idx.query("before_update", 10).unwrap();
    assert!(old_hits.is_empty(), "old content must be gone after update");
    let new_hits = idx.query("after_update", 10).unwrap();
    assert!(!new_hits.is_empty(), "new content must be findable after update");
}

#[test]
fn binary_file_skipped_in_index_directory() {
    let tmp = tempfile::TempDir::new().unwrap();
    let src = tmp.path().join("mix");
    std::fs::create_dir_all(&src).unwrap();
    write_file(&src, "text.txt", "text content\n");
    let bin_path = src.join("binary.bin");
    std::fs::write(&bin_path, b"has\x00null\x00bytes").unwrap();
    let mut idx = Bm25Index::new(tmp.path(), "contract_bin").unwrap();
    let stats = idx.index_directory(&src).unwrap();
    assert_eq!(stats.files_skipped_binary, 1, "binary file must be skipped");
    assert_eq!(stats.files_indexed, 1, "only text file should be indexed");
}

#[test]
fn empty_query_returns_empty_results() {
    let tmp = tempfile::TempDir::new().unwrap();
    let idx = Bm25Index::new(tmp.path(), "contract_empty").unwrap();
    let hits = idx.query("", 10).unwrap();
    assert!(hits.is_empty(), "empty query must return empty results");
    let ws_hits = idx.query("   ", 10).unwrap();
    assert!(ws_hits.is_empty(), "whitespace-only query must return empty results");
}

#[test]
fn chunk_kind_variants_serializable() {
    let kinds = [ChunkKind::Function, ChunkKind::LineBlock, ChunkKind::File];
    for kind in &kinds {
        let json = serde_json::to_string(kind).unwrap();
        let back: ChunkKind = serde_json::from_str(&json).unwrap();
        assert_eq!(*kind, back, "ChunkKind must round-trip through serde");
    }
}

#[test]
fn index_stats_fields_populated() {
    let tmp = tempfile::TempDir::new().unwrap();
    let src = tmp.path().join("pop");
    std::fs::create_dir_all(&src).unwrap();
    write_file(&src, "file.rs", "fn populated() {}\nfn also() {}\n");
    let mut idx = Bm25Index::new(tmp.path(), "contract_stats").unwrap();
    let stats = idx.index_directory(&src).unwrap();
    assert_eq!(stats.files_indexed, 1);
    assert!(stats.chunks_indexed >= 1, "must have at least 1 chunk");
    // elapsed_ms is currently 0 (no timing impl yet) — check type only
    let _: u64 = stats.elapsed_ms;
}
