//! E-17 SP4 task 2 — contract tests for the contextual_retrieval module.
//!
//! Three fixtures:
//!   (a) Round-trip a synthetic chunk + source pair; assert the prefix
//!       header is present, the delimiter separates prefix from body,
//!       and the original body is preserved verbatim.
//!   (b) Hash-stable regen skip: invoke contextualize_chunk twice with
//!       the same source; second call (passing the source_hash from first)
//!       MUST take the regen-skip path (fresh_prefix=false, body unchanged).
//!   (c) Hash mismatch forces regen: different cached_source_hash triggers
//!       a fresh prefix.
//!
//! Citation: anthropic.com/news/contextual-retrieval (Sep 2024) — heuristic
//! ground-floor implementation per workflow/reasoning/E-17-decisions.md.

use std::path::PathBuf;

use elai_knowledge::bm25::{BM25Chunk, ChunkKind};
use elai_knowledge::contextual_retrieval::{
    blake3_short_hex, build_heuristic_context, contextualize_chunk, is_contextualised,
    CONTEXT_HINT_HEADER, CONTEXT_PREFIX_DELIMITER, MAX_CONTEXT_PREFIX_CHARS,
};

fn mk_chunk(body: &str, line_start: u32, kind: ChunkKind) -> BM25Chunk {
    BM25Chunk {
        path: PathBuf::from("src/example.rs"),
        content: body.to_string(),
        line_start,
        line_end: line_start.saturating_add(5),
        kind,
    }
}

#[test]
fn round_trip_synthetic_chunk_preserves_body_and_prepends_prefix() {
    let body = "let answer = 42; // load-bearing answer";
    let chunk = mk_chunk(body, 10, ChunkKind::Function);
    let source = "// header line 1\n// header line 2\nfn outer() {\n    // 4\n    // 5\n    // 6\n    // 7\n    // 8\n    // 9\n    let answer = 42; // load-bearing answer\n}\n";

    let result = contextualize_chunk(chunk.clone(), source, None);

    assert!(
        result.fresh_prefix,
        "no cached hash -> must take the fresh-prefix path"
    );
    // Prefix header present.
    assert!(
        result.chunk.content.starts_with(CONTEXT_HINT_HEADER),
        "content must begin with the hint header; got: {:?}",
        &result.chunk.content[..CONTEXT_HINT_HEADER.len().min(result.chunk.content.len())]
    );
    // Delimiter present.
    assert!(
        result.chunk.content.contains(CONTEXT_PREFIX_DELIMITER),
        "delimiter must separate prefix from body"
    );
    // Original body preserved verbatim AFTER the delimiter.
    let after_delim = result
        .chunk
        .content
        .split(CONTEXT_PREFIX_DELIMITER)
        .nth(1)
        .expect("delimiter splits into 2+ parts");
    assert_eq!(after_delim, body, "body must be preserved verbatim");
    // Identity metadata preserved.
    assert_eq!(result.chunk.line_start, chunk.line_start);
    assert_eq!(result.chunk.line_end, chunk.line_end);
    assert_eq!(result.chunk.kind, chunk.kind);
    assert_eq!(result.chunk.path, chunk.path);
    // Hashes are populated and have the documented 16-char width.
    assert_eq!(
        result.prefix_hash.len(),
        16,
        "prefix_hash must be the 16-char blake3 short form"
    );
    assert_eq!(
        result.source_hash.len(),
        16,
        "source_hash must be the 16-char blake3 short form"
    );
}

#[test]
fn hash_stable_regen_skip_does_not_modify_body() {
    let body = "let answer = 42;";
    let chunk = mk_chunk(body, 1, ChunkKind::Function);
    let source = "fn outer() {\n    let answer = 42;\n}\n";
    let first = contextualize_chunk(chunk.clone(), source, None);
    assert!(first.fresh_prefix);

    // Pass the cached source_hash back in -> regen-skip path fires.
    let second = contextualize_chunk(chunk.clone(), source, Some(&first.source_hash));
    assert!(!second.fresh_prefix, "matching cache hash must skip regen");
    assert_eq!(
        second.chunk.content, body,
        "regen-skip MUST leave body untouched (caller reuses prior persisted chunk)"
    );
    assert_eq!(second.source_hash, first.source_hash);
}

#[test]
fn hash_mismatch_forces_fresh_regen() {
    let body = "let z = 3;";
    let chunk = mk_chunk(body, 2, ChunkKind::Function);
    let source = "fn outer() {}\nlet z = 3;\n";
    let result = contextualize_chunk(chunk, source, Some("deadbeefdeadbeef"));
    assert!(result.fresh_prefix, "cache miss -> fresh prefix");
    assert!(result.chunk.content.starts_with(CONTEXT_HINT_HEADER));
}

#[test]
fn prefix_length_never_exceeds_cap() {
    let long_body = "x".repeat(8000);
    let chunk = mk_chunk(&long_body, 1, ChunkKind::Function);
    let source = format!("{long_body}\n");
    let prefix = build_heuristic_context(&chunk, &source);
    assert!(
        prefix.len() <= MAX_CONTEXT_PREFIX_CHARS,
        "prefix len {} must not exceed MAX_CONTEXT_PREFIX_CHARS {}",
        prefix.len(),
        MAX_CONTEXT_PREFIX_CHARS
    );
}

#[test]
fn is_contextualised_round_trips_after_contextualize() {
    let chunk = mk_chunk("body", 1, ChunkKind::Function);
    let source = "fn outer() {}\nbody\n";
    let result = contextualize_chunk(chunk, source, None);
    assert!(
        is_contextualised(&result.chunk),
        "fresh contextualisation must be detectable by is_contextualised"
    );
}

#[test]
fn blake3_short_hex_is_stable_and_unique() {
    let a = blake3_short_hex(b"alpha");
    let b = blake3_short_hex(b"beta");
    let a2 = blake3_short_hex(b"alpha");
    assert_eq!(a, a2, "same input must produce same hash");
    assert_ne!(a, b, "distinct inputs must produce distinct hashes");
}
