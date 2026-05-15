//! Character-based chunker for minified content.
//!
//! Splits content into `MAX_CHUNK_CHARS`-sized windows, scanning backwards
//! from each hard limit for a safe split character (`\n`, ` `, `\t`, `;`,
//! `,`) so chunks don't bisect identifiers in minified JS/CSS.
//!
//! Chunk identity: minified files often consist of a single very long line,
//! so `line_start == line_end == 1` for every chunk. We use the synthetic
//! `1 + chunk_index` value for both `line_start` and `line_end` (D6) to
//! keep BM25 chunk IDs unique without adding a `byte_offset` schema column.
//!
//! Reference: SocratiCode src/services/indexer.ts:363-411 (chunkByCharacters).

use std::path::Path;

use crate::bm25::{BM25Chunk, ChunkKind};

use super::cap::floor_char_boundary;
use super::constants::MAX_CHUNK_CHARS;

/// Split `content` into character-based chunks for minified files.
///
/// All chunks carry `ChunkKind::Minified`. `line_start` and `line_end` are
/// synthetic (`1 + chunk_index`) to provide unique IDs in tantivy where
/// per-doc identity is `(path, line_start, line_end)`.
pub fn chunk_by_characters(path: &Path, content: &str) -> Vec<BM25Chunk> {
    let mut chunks = Vec::new();
    let mut offset: usize = 0;

    while offset < content.len() {
        // Hard limit, walked back to a UTF-8 boundary.
        let hard = floor_char_boundary(content, offset.saturating_add(MAX_CHUNK_CHARS));
        let mut end = hard;

        // Backward scan for a safe split char, but only when not at EOF
        // (otherwise we'd needlessly shrink the last chunk).
        if end < content.len() {
            // Walk byte-by-byte backwards. We use byte access because we've
            // already confirmed `end` sits on a char boundary above; the
            // safe split chars (\n, space, \t, ;, ,) are all ASCII so byte
            // comparison is valid.
            let bytes = content.as_bytes();
            let mut i = end;
            while i > offset {
                let b = bytes[i.saturating_sub(1)];
                if b == b'\n' || b == b' ' || b == b'\t' || b == b';' || b == b',' {
                    end = i;
                    break;
                }
                i -= 1;
            }
            // If the scan reached `offset` without finding a boundary, fall
            // through with `end = hard` (split at the hard limit).
        }

        // Defensive: if backward-scan didn't move `end` past `offset` (e.g.
        // a content stream with no separators at all), force progress to
        // `hard` so we don't loop forever.
        if end <= offset {
            end = hard.max(offset.saturating_add(1)).min(content.len());
            // Re-align to char boundary in case the saturating add landed
            // mid-codepoint.
            end = floor_char_boundary(content, end);
            if end <= offset {
                // Truly stuck (single-codepoint > MAX_CHUNK_CHARS — pathological).
                break;
            }
        }

        let chunk_index_u32 = u32::try_from(chunks.len()).unwrap_or(u32::MAX);
        let line_no = chunk_index_u32.saturating_add(1);

        chunks.push(BM25Chunk {
            path: path.to_path_buf(),
            content: content[offset..end].to_string(),
            line_start: line_no,
            line_end: line_no,
            kind: ChunkKind::Minified,
        });

        offset = end;
    }

    chunks
}
