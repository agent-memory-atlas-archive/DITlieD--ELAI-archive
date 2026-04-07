#![allow(unused_imports, dead_code)]
// Compile-time interface assertions for Bm25Index and associated types.
// These are type-level checks — if the API surface changes, this file fails to compile.

use std::path::Path;

use elai_knowledge::bm25::{Bm25Index, BM25Chunk, BM25Hit, ChunkKind, IndexStats};

// Verify Bm25Index::new signature: (&Path, &str) -> Result<Self, Box<dyn Error>>
fn _assert_new_signature() {
    let _: fn(&Path, &str) -> Result<Bm25Index, Box<dyn std::error::Error>> = Bm25Index::new;
}

// Verify index_file signature: (&mut self, &Path, Option<Vec<BM25Chunk>>) -> Result<usize, ...>
fn _assert_index_file_signature(idx: &mut Bm25Index) {
    let _: Result<usize, Box<dyn std::error::Error>> =
        idx.index_file(Path::new("x"), Some(Vec::new()));
}

// Verify query signature: (&self, &str, usize) -> Result<Vec<BM25Hit>, ...>
fn _assert_query_signature(idx: &Bm25Index) {
    let _: Result<Vec<BM25Hit>, Box<dyn std::error::Error>> = idx.query("q", 10);
}

// Verify delete_file signature: (&mut self, &Path) -> Result<usize, ...>
fn _assert_delete_file_signature(idx: &mut Bm25Index) {
    let _: Result<usize, Box<dyn std::error::Error>> = idx.delete_file(Path::new("x"));
}

// Verify update_file signature: (&mut self, &Path, Option<Vec<BM25Chunk>>) -> Result<usize, ...>
fn _assert_update_file_signature(idx: &mut Bm25Index) {
    let _: Result<usize, Box<dyn std::error::Error>> =
        idx.update_file(Path::new("x"), Some(Vec::new()));
}

// Verify index_directory signature: (&mut self, &Path) -> Result<IndexStats, ...>
fn _assert_index_directory_signature(idx: &mut Bm25Index) {
    let _: Result<IndexStats, Box<dyn std::error::Error>> =
        idx.index_directory(Path::new("x"));
}

// Verify BM25Chunk fields
fn _assert_bm25_chunk_fields() {
    let _chunk = BM25Chunk {
        path: std::path::PathBuf::new(),
        content: String::new(),
        line_start: 0u32,
        line_end: 0u32,
        kind: ChunkKind::Function,
    };
}

// Verify BM25Hit fields
fn _assert_bm25_hit_fields(hit: BM25Hit) {
    let _: String = hit.path;
    let _: u32 = hit.line_start;
    let _: u32 = hit.line_end;
    let _: f32 = hit.score;
    let _: ChunkKind = hit.chunk_kind;
    let _: String = hit.snippet;
}

// Verify IndexStats fields
fn _assert_index_stats_fields(s: IndexStats) {
    let _: usize = s.files_indexed;
    let _: usize = s.chunks_indexed;
    let _: usize = s.files_skipped_binary;
    let _: u64 = s.elapsed_ms;
}

// Verify ChunkKind variants exist
fn _assert_chunk_kind_variants() {
    let _ = ChunkKind::Function;
    let _ = ChunkKind::LineBlock;
    let _ = ChunkKind::File;
}
