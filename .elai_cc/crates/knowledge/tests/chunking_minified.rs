//! B-32 Step A — unit tests for chunking module (minified detection,
//! character chunker, char cap, UTF-8 boundary helper).
//!
//! Public path: knowledge::chunking::* (declared in lib.rs).

use std::path::PathBuf;

use elai_knowledge::bm25::{BM25Chunk, ChunkKind};
use elai_knowledge::chunking::cap::{apply_char_cap, floor_char_boundary};
use elai_knowledge::chunking::character::chunk_by_characters;
use elai_knowledge::chunking::constants::{MAX_AVG_LINE_LENGTH, MAX_CHUNK_CHARS};
use elai_knowledge::chunking::detect::is_minified;

// ---------------------------------------------------------------------------
// is_minified
// ---------------------------------------------------------------------------

#[test]
fn test_is_minified_detects_single_line_50k_chars() {
    let content = "a".repeat(50_000);
    let line_count = content.lines().count();
    assert!(is_minified(&content, line_count),
        "50K-char single-line file must be detected as minified");
}

#[test]
fn test_is_minified_rejects_normal_code() {
    // 100 lines of typical-length code.
    let line = "fn foo() -> u32 { 42 }".to_string();
    let content = std::iter::repeat(line).take(100).collect::<Vec<_>>().join("\n");
    let line_count = content.lines().count();
    assert!(!is_minified(&content, line_count),
        "normal code with avg_line_length<{MAX_AVG_LINE_LENGTH} must NOT be detected as minified");
}

#[test]
fn test_is_minified_zero_lines_uses_content_len() {
    // Empty file: line_count=0, content.len()=0 -> NOT minified (0 > 500 is false).
    assert!(!is_minified("", 0), "empty file is not minified");
    // Zero lines but content longer than threshold (no newlines at all):
    // line_count==0 branch falls back to content.len() > MAX_AVG_LINE_LENGTH.
    let content = "x".repeat(MAX_AVG_LINE_LENGTH + 1);
    assert!(is_minified(&content, 0),
        "zero-line content longer than {MAX_AVG_LINE_LENGTH} must be detected as minified");
}

#[test]
fn test_is_minified_threshold_boundary() {
    // avg = 500 -> NOT minified (strict >).
    let content = "a".repeat(500);
    assert!(!is_minified(&content, 1),
        "avg_line_length == {MAX_AVG_LINE_LENGTH} must NOT be minified (strict >)");
    // avg = 501 -> minified.
    let content = "a".repeat(501);
    assert!(is_minified(&content, 1),
        "avg_line_length > {MAX_AVG_LINE_LENGTH} must be minified");
}

// ---------------------------------------------------------------------------
// chunk_by_characters
// ---------------------------------------------------------------------------

#[test]
fn test_chunk_by_characters_safe_boundary_split() {
    // Build content with a comma right before MAX_CHUNK_CHARS so the
    // backward-scan picks the comma (not the hard limit).
    let mut content = "a".repeat(MAX_CHUNK_CHARS - 5);
    content.push(',');
    content.push_str(&"b".repeat(MAX_CHUNK_CHARS));
    let path = PathBuf::from("test.min.js");

    let chunks = chunk_by_characters(&path, &content);
    assert!(!chunks.is_empty(), "must produce at least one chunk");
    // First chunk should END at the comma (i.e., comma is included as last char of chunk[0]).
    let first = &chunks[0];
    assert!(first.content.ends_with(','),
        "first chunk should end at the safe-boundary comma, got tail={:?}",
        first.content.chars().rev().take(5).collect::<String>());
}

#[test]
fn test_chunk_by_characters_respects_max_chunk_chars() {
    let content = "x".repeat(50_000);
    let path = PathBuf::from("blob.min.js");
    let chunks = chunk_by_characters(&path, &content);
    assert!(chunks.len() >= 25, "50K / 2K must produce >= 25 chunks, got {}", chunks.len());
    for (i, chunk) in chunks.iter().enumerate() {
        assert!(chunk.content.len() <= MAX_CHUNK_CHARS,
            "chunk[{i}] has {} chars, exceeds MAX_CHUNK_CHARS={MAX_CHUNK_CHARS}",
            chunk.content.len());
    }
}

#[test]
fn test_chunk_by_characters_synthetic_line_start_unique() {
    let content = "x".repeat(50_000);
    let path = PathBuf::from("blob.min.js");
    let chunks = chunk_by_characters(&path, &content);
    let line_starts: Vec<u32> = chunks.iter().map(|c| c.line_start).collect();
    let mut sorted_unique = line_starts.clone();
    sorted_unique.sort();
    sorted_unique.dedup();
    assert_eq!(sorted_unique.len(), line_starts.len(),
        "every chunk must have a unique synthetic line_start; got {line_starts:?}");
    // Chunks all carry ChunkKind::Minified.
    for (i, c) in chunks.iter().enumerate() {
        assert_eq!(c.kind, ChunkKind::Minified, "chunk[{i}] kind={:?}", c.kind);
    }
}

#[test]
fn test_chunk_by_characters_utf8_boundary_safe() {
    // Build content with a 4-byte codepoint (rocket emoji 🚀 = F0 9F 9A 80) at
    // a position that would land inside a multi-byte sequence if we sliced
    // naively at MAX_CHUNK_CHARS=2000. Pad with 'a' until just before 2000,
    // then place the emoji so its bytes 1997-2000 straddle the cap.
    let pad = "a".repeat(MAX_CHUNK_CHARS - 3);
    let content = format!("{pad}🚀{}", "b".repeat(MAX_CHUNK_CHARS));
    let path = PathBuf::from("emoji.min.js");
    let chunks = chunk_by_characters(&path, &content);
    // No panic = pass for boundary-safety; also assert each chunk is valid UTF-8.
    for (i, chunk) in chunks.iter().enumerate() {
        assert!(chunk.content.is_char_boundary(0) && chunk.content.is_char_boundary(chunk.content.len()),
            "chunk[{i}] has invalid UTF-8 boundaries");
        assert!(chunk.content.len() <= MAX_CHUNK_CHARS,
            "chunk[{i}] exceeds cap: {} > {MAX_CHUNK_CHARS}", chunk.content.len());
    }
}

// ---------------------------------------------------------------------------
// apply_char_cap
// ---------------------------------------------------------------------------

#[test]
fn test_apply_char_cap_truncates_oversized_minified_chunks() {
    let chunks = vec![BM25Chunk {
        path: PathBuf::from("oversized.min.js"),
        content: "z".repeat(MAX_CHUNK_CHARS + 500),
        line_start: 1,
        line_end: 1,
        kind: ChunkKind::Minified,
    }];
    let capped = apply_char_cap(chunks);
    assert_eq!(capped.len(), 1);
    assert!(capped[0].content.len() <= MAX_CHUNK_CHARS,
        "Minified chunk must be truncated to MAX_CHUNK_CHARS; got {}", capped[0].content.len());
}

#[test]
fn test_apply_char_cap_passes_through_file_kind_chunks() {
    // D10: ChunkKind::File chunks are EXEMPT from cap (zero-orphan / show-file UX).
    let original_len = MAX_CHUNK_CHARS + 5_000;
    let chunks = vec![BM25Chunk {
        path: PathBuf::from("big.rs"),
        content: "f".repeat(original_len),
        line_start: 1,
        line_end: 100,
        kind: ChunkKind::File,
    }];
    let capped = apply_char_cap(chunks);
    assert_eq!(capped.len(), 1);
    assert_eq!(capped[0].content.len(), original_len,
        "File-kind chunks MUST NOT be truncated (D10)");
}

#[test]
fn test_apply_char_cap_utf8_boundary_safe() {
    // Build chunk content that has a 4-byte emoji straddling MAX_CHUNK_CHARS.
    let pad = "a".repeat(MAX_CHUNK_CHARS - 2);
    let content = format!("{pad}🚀more");
    let chunks = vec![BM25Chunk {
        path: PathBuf::from("emoji.min.js"),
        content: content.clone(),
        line_start: 1,
        line_end: 1,
        kind: ChunkKind::Minified,
    }];
    let capped = apply_char_cap(chunks);
    let truncated = &capped[0].content;
    assert!(truncated.len() <= MAX_CHUNK_CHARS);
    // Critically: the truncated string itself must be valid UTF-8.
    assert!(truncated.is_char_boundary(truncated.len()),
        "truncated content has invalid UTF-8 ending boundary");
}

#[test]
fn test_apply_char_cap_leaves_small_chunks_untouched() {
    let chunks = vec![BM25Chunk {
        path: PathBuf::from("small.rs"),
        content: "fn main() {}".to_string(),
        line_start: 1,
        line_end: 1,
        kind: ChunkKind::Function,
    }];
    let capped = apply_char_cap(chunks);
    assert_eq!(capped[0].content, "fn main() {}");
}

// ---------------------------------------------------------------------------
// floor_char_boundary
// ---------------------------------------------------------------------------

#[test]
fn test_floor_char_boundary_walks_back_from_codepoint_interior() {
    // Rocket emoji 🚀 is 4 bytes (F0 9F 9A 80).
    let s = "ab🚀cd";
    // Bytes: a(0) b(1) 🚀(2..6) c(6) d(7) — total 8 bytes.
    // Asking for cap=4 lands inside the emoji; must walk back to byte 2.
    assert_eq!(floor_char_boundary(s, 4), 2);
    // cap=5 -> still inside emoji -> walk back to 2.
    assert_eq!(floor_char_boundary(s, 5), 2);
    // cap=6 -> exactly after emoji -> stays at 6.
    assert_eq!(floor_char_boundary(s, 6), 6);
}

#[test]
fn test_floor_char_boundary_clamps_to_len() {
    let s = "hello";
    // cap > len -> clamp to len.
    assert_eq!(floor_char_boundary(s, 100), s.len());
    // cap == len -> identity.
    assert_eq!(floor_char_boundary(s, s.len()), s.len());
    // cap == 0 -> zero.
    assert_eq!(floor_char_boundary(s, 0), 0);
}

#[test]
fn test_floor_char_boundary_ascii_passthrough() {
    let s = "abcdef";
    assert_eq!(floor_char_boundary(s, 3), 3);
    assert_eq!(floor_char_boundary(s, 4), 4);
}
