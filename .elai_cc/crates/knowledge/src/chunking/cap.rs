//! Universal character-cap safety net.
//!
//! `apply_char_cap` is the LAST step of every chunk-producing dispatch path
//! (D1/D2 in the B-32 decision doc). It enforces `MAX_CHUNK_CHARS` on every
//! chunk EXCEPT `ChunkKind::File` — the zero-orphan File chunk preserves
//! full file content for the "show file" UX (D10).
//!
//! UTF-8 safety: truncation walks back from the cap byte index until a
//! valid char boundary is found, matching the in-tree idiom at
//! vector/indexer.rs:380, telescope.rs:104, bash.rs:358 (D7).

use crate::bm25::{BM25Chunk, ChunkKind};

use super::constants::MAX_CHUNK_CHARS;

/// Walk back from `cap` (clamped to `s.len()`) until landing on a valid
/// UTF-8 char boundary. Pure helper; never panics; returns 0 if no boundary
/// is found before the start of the string.
///
/// Codebase idiom (D7) — preferred over nightly `std::str::floor_char_boundary`.
pub fn floor_char_boundary(s: &str, cap: usize) -> usize {
    let mut end = cap.min(s.len());
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    end
}

/// Apply `MAX_CHUNK_CHARS` cap to every non-File chunk. Returns the input
/// vector with content truncated in-place (via re-construction; the
/// `BM25Chunk` fields are owned).
///
/// File-kind chunks are passed through untouched (D10).
pub fn apply_char_cap(chunks: Vec<BM25Chunk>) -> Vec<BM25Chunk> {
    chunks
        .into_iter()
        .map(|chunk| {
            if chunk.kind == ChunkKind::File {
                return chunk;
            }
            if chunk.content.len() <= MAX_CHUNK_CHARS {
                return chunk;
            }
            let end = floor_char_boundary(&chunk.content, MAX_CHUNK_CHARS);
            let mut content = chunk.content;
            content.truncate(end);
            BM25Chunk {
                path: chunk.path,
                content,
                line_start: chunk.line_start,
                line_end: chunk.line_end,
                kind: chunk.kind,
            }
        })
        .collect()
}
