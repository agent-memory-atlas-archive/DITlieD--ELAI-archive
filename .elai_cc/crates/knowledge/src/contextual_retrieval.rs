//! E-17 SP4 — Anthropic Contextual Retrieval at ingest.
//!
//! Implements the heuristic ground-floor version of Anthropic Contextual
//! Retrieval (anthropic.com/news/contextual-retrieval, Sep 2024): each chunk
//! gets a brief contextual prefix prepended before BM25 indexing and dense
//! vector embedding. The prefix narrates where the chunk lives in the file
//! (preceding heading + symbol path) so dense retrievers can pick up
//! lexically-distant queries that the chunk body alone would miss.
//!
//! Two production paths share this module:
//!
//! 1. Heuristic path ([`build_heuristic_context`]) — pure-Rust, no sidecar
//!    dependency. Walks the chunk's `(path, line_start)` against the prior
//!    headings / preceding-AST-node window from the source file. SHIPS as
//!    the default ground-floor; safe to run at scale on every chunk.
//!
//! 2. LLM-backed path ([`ContextualPromptTemplate`]) — type + prompt template
//!    only; the actual sidecar dispatch is documented as a future hook
//!    (mirror `crate::hyde::HydeStage::hypothesize` for the dispatch wire).
//!    The interface is stable so a future ablation can flip on the
//!    LLM-backed version without re-shaping callers. JINA-MODEL-PENDING
//!    framing applies — adding a per-chunk sidecar call inflates ingest
//!    N-fold (plan §Open questions) so the LLM path lives behind a flag.
//!
//! Source-file hashing uses blake3 per CLAUDE.md anti-stale rule. The hash
//! pair `(source_hash, prefix_hash)` lets re-indexes skip regen when the
//! source file is unchanged — emits a `chunk_regen_skipped` SSB beacon at
//! the caller boundary (SCHEMA_VERSION 103 LOCKED so the SSB variant is
//! deferred per workflow/reasoning/E-17-decisions.md; the producer code
//! here is ready for the variant once SCHEMA bump lands).
//!
//! Citations:
//!  - anthropic.com/news/contextual-retrieval (Sep 2024) — -49% top-20
//!    failure on Anthropic's internal codebase eval, -67% with rerank.
//!    Heuristic ground-floor is an extrapolation, NOT measured; numbers
//!    require LLM-backed path to claim.
//!  - blake3 1.8 (workspace dep) — CLAUDE.md anti-stale Rust rule.

use blake3::Hasher as Blake3Hasher;

use crate::bm25::BM25Chunk;

/// Maximum byte length of a heuristic context prefix. Kept well below the
/// `MAX_CHUNK_CHARS = 2000` upstream cap so prefix + body always fits.
/// Anthropic's reference template targets 50-100 tokens (~200-400 chars);
/// we choose 256 as the conservative budget that matches the upper bound
/// once the symbol path + heading concatenate.
pub const MAX_CONTEXT_PREFIX_CHARS: usize = 256;

/// Lines of context preceding `chunk.line_start` to scan for the nearest
/// section heading / function signature. 16 lines covers most realistic
/// "doc comment + signature" windows without dragging in unrelated code.
pub const CONTEXT_WINDOW_LINES: u32 = 16;

/// Prefix marker placed between the contextual hint and the original chunk
/// body. Used by retrieval-quality checks to detect contextualised chunks
/// (and by tests). Kept stable across versions.
pub const CONTEXT_PREFIX_DELIMITER: &str = "\n--- context ---\n";

/// Stable header string at the start of every heuristic prefix. Used by
/// tests + a future dashboard surface to quickly identify contextualised
/// chunks without parsing the body.
pub const CONTEXT_HINT_HEADER: &str = "Context: ";

/// E-17 SP4: result of running [`contextualize_chunk`] on a single chunk.
///
/// The two hash fields let callers maintain the per-chunk content hash
/// chain used by the `chunk_regen_skipped` SSB beacon (SCHEMA-DEFERRED;
/// see workflow/reasoning/E-17-decisions.md). `prefix_hash` covers only
/// the generated prefix; `source_hash` covers the raw source file at
/// contextualisation time. Both are blake3 hex digests (16-char prefix).
#[derive(Debug, Clone)]
pub struct ContextualisedChunk {
    /// The chunk with its `content` field rewritten to `prefix + delimiter
    /// + original_body`. `path`, `line_start`, `line_end`, `kind` are
    /// preserved verbatim from the input chunk.
    pub chunk: BM25Chunk,
    /// blake3 hex (16 chars) of the contextual prefix that was prepended.
    /// Stable across runs given identical inputs.
    pub prefix_hash: String,
    /// blake3 hex (16 chars) of the source-file content at the moment
    /// contextualisation ran. Used by re-index callers to detect "did the
    /// source change since last contextualisation?" and skip regen when not.
    pub source_hash: String,
    /// `true` when the prefix was generated fresh; `false` when the caller
    /// hit the regen-skip path (source_hash matched the cached value).
    pub fresh_prefix: bool,
}

/// E-17 SP4: blake3 hex digest (first 16 hex chars / 64-bit prefix) of the
/// input bytes. Mirrors `crate::hyde::HydeStage::query_hash` style — 16
/// chars is enough to distinguish chunks in a dashboard without leaking
/// content. The `1.8` blake3 workspace dep is the anti-stale-Rust choice
/// (CLAUDE.md: never blake2/sha2 for non-RFC uses).
pub fn blake3_short_hex(bytes: &[u8]) -> String {
    let mut hasher = Blake3Hasher::new();
    hasher.update(bytes);
    let hash = hasher.finalize();
    // to_hex() returns the full 64-char hex; take the first 16 for the
    // dashboard-friendly short form (matches HydeQueryEvent::query_hash).
    let full = hash.to_hex();
    full.as_str()[..16].to_string()
}

/// E-17 SP4: build a heuristic context prefix for `chunk` given the full
/// `source` file text. Pure-Rust, no sidecar dependency — safe to run
/// on every chunk.
///
/// Strategy (ground-floor, deterministic):
///   1. Take the chunk's relative path (or basename if no `project_root`
///      can be derived) -> `"file `path`"`.
///   2. Append the chunk's `ChunkKind` -> `"kind: function"` / `"kind: heading"`.
///   3. Scan `CONTEXT_WINDOW_LINES` lines preceding `chunk.line_start` for
///      the nearest heading / function-signature / class-opening line and
///      append it -> `"nearest: fn rerank_with_breaker(...)"`.
///   4. Truncate the assembled prefix to `MAX_CONTEXT_PREFIX_CHARS` on a
///      char boundary.
///
/// Returns the prefix WITHOUT the delimiter; [`contextualize_chunk`] adds
/// the delimiter when it prepends.
pub fn build_heuristic_context(chunk: &BM25Chunk, source: &str) -> String {
    let mut parts: Vec<String> = Vec::with_capacity(3);

    // 1. File anchor — use the file_name only when the path is absolute
    // (project_root is not known here); the BM25Chunk preserves the full
    // path so callers always have the absolute reference if needed.
    let file_anchor = chunk
        .path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    if !file_anchor.is_empty() {
        parts.push(format!("file {file_anchor}"));
    }

    // 2. Chunk-kind anchor — gives the dense embedder a hint about the
    // chunk's syntactic role.
    parts.push(format!("kind {}", chunk.kind.as_str()));

    // 3. Preceding-line scan for a heading or signature.
    if let Some(nearest) = scan_preceding_anchor(source, chunk.line_start) {
        parts.push(format!("near: {nearest}"));
    }

    let joined = parts.join("; ");
    let prefix = format!("{CONTEXT_HINT_HEADER}{joined}");
    truncate_on_char_boundary(&prefix, MAX_CONTEXT_PREFIX_CHARS)
}

/// E-17 SP4: prepend a heuristic context prefix to `chunk.content`,
/// recording the source and prefix hashes for downstream regen-skip
/// decisions.
///
/// `cached_source_hash` is the blake3-short-hex of the source file as
/// recorded by the LAST contextualisation pass over this file. When
/// `Some(h)` matches the freshly-computed source hash, the function
/// returns a [`ContextualisedChunk`] with `fresh_prefix = false` and
/// the chunk body UNCHANGED — callers should emit `chunk_regen_skipped`
/// and reuse the prior persisted chunk. When `None` or mismatched, the
/// prefix is generated fresh and `fresh_prefix = true`.
///
/// The function is infallible: any future LLM-backed dispatch failure
/// falls back to the heuristic prefix (parity with HyDE graceful degrade).
pub fn contextualize_chunk(
    chunk: BM25Chunk,
    source: &str,
    cached_source_hash: Option<&str>,
) -> ContextualisedChunk {
    let source_hash = blake3_short_hex(source.as_bytes());

    if let Some(cached) = cached_source_hash {
        if cached == source_hash {
            // Source unchanged -> regen skip path. Caller emits
            // chunk_regen_skipped beacon once SCHEMA bump lands.
            let prefix_hash = blake3_short_hex(chunk.content.as_bytes());
            return ContextualisedChunk {
                chunk,
                prefix_hash,
                source_hash,
                fresh_prefix: false,
            };
        }
    }

    let prefix = build_heuristic_context(&chunk, source);
    let prefix_hash = blake3_short_hex(prefix.as_bytes());

    let mut combined =
        String::with_capacity(prefix.len() + CONTEXT_PREFIX_DELIMITER.len() + chunk.content.len());
    combined.push_str(&prefix);
    combined.push_str(CONTEXT_PREFIX_DELIMITER);
    combined.push_str(&chunk.content);

    let new_chunk = BM25Chunk {
        path: chunk.path,
        content: combined,
        line_start: chunk.line_start,
        line_end: chunk.line_end,
        kind: chunk.kind,
    };

    ContextualisedChunk {
        chunk: new_chunk,
        prefix_hash,
        source_hash,
        fresh_prefix: true,
    }
}

/// E-17 SP4: check whether `chunk.content` already starts with the
/// contextual hint header. Lets callers idempotently skip chunks that
/// were already contextualised in a prior ingest pass (defensive — the
/// regen-skip hash check is the primary gate).
pub fn is_contextualised(chunk: &BM25Chunk) -> bool {
    chunk.content.starts_with(CONTEXT_HINT_HEADER)
}

/// E-17 SP4: scan up to [`CONTEXT_WINDOW_LINES`] lines BEFORE `start_line`
/// (1-indexed) for the nearest line that looks like a heading or a
/// function / class / def signature. Returns the trimmed line text when
/// found, `None` otherwise. Returns `None` for `start_line == 0` or `1`
/// (no preceding context).
fn scan_preceding_anchor(source: &str, start_line: u32) -> Option<String> {
    if start_line < 2 {
        return None;
    }
    let target_idx = start_line.saturating_sub(1) as usize;
    let window_start = target_idx.saturating_sub(CONTEXT_WINDOW_LINES as usize);

    // Walk backwards from the line above `start_line` looking for a hit.
    let lines: Vec<&str> = source.lines().collect();
    let scan_upper = target_idx.min(lines.len());
    if window_start >= scan_upper {
        return None;
    }
    for i in (window_start..scan_upper).rev() {
        let line = lines[i].trim();
        if looks_like_anchor(line) {
            return Some(truncate_on_char_boundary(line, 96));
        }
    }
    None
}

/// E-17 SP4: lightweight heuristic for "is this line an anchor?".
/// Looks for markdown headings, comment-doc headings, and the common
/// signature openings for code (rust / python / go / ts / java / cs /
/// ruby / swift / kotlin). Deliberately conservative — false negatives
/// just produce a shorter prefix; false positives produce a less useful
/// one. Both are non-fatal.
fn looks_like_anchor(line: &str) -> bool {
    if line.is_empty() {
        return false;
    }
    // Markdown / restructured-text / org headings.
    if line.starts_with('#')
        || line.starts_with("//!")
        || line.starts_with("///")
        || line.starts_with("/**")
    {
        return true;
    }
    // Common code signatures across the languages knowledge crate parses.
    const SIG_PREFIXES: &[&str] = &[
        "fn ",
        "pub fn ",
        "async fn ",
        "pub async fn ",
        "def ",
        "async def ",
        "func ",
        "class ",
        "struct ",
        "enum ",
        "trait ",
        "interface ",
        "impl ",
        "module ",
        "package ",
        "namespace ",
        "type ",
        "object ",
        "public class ",
        "public interface ",
        "private class ",
        "protected class ",
    ];
    for p in SIG_PREFIXES {
        if line.starts_with(p) {
            return true;
        }
    }
    false
}

/// E-17 SP4: floor `s` to at most `max_chars` bytes on a UTF-8 char
/// boundary. Mirrors `chunking::cap::floor_char_boundary` but for owned
/// strings (we want a `String` return for the prefix path). Safe on
/// empty input.
fn truncate_on_char_boundary(s: &str, max_chars: usize) -> String {
    if s.len() <= max_chars {
        return s.to_string();
    }
    let mut end = max_chars;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    s[..end].to_string()
}

// ---------------------------------------------------------------------------
// LLM-backed prompt template (FUTURE — wired behind a flag in E-17.B)
// ---------------------------------------------------------------------------

/// E-17 SP4 LLM-backed path scaffold — Anthropic Contextual Retrieval
/// reference template (50-100 tokens). Lives here for the future ablation
/// path; not invoked by the default ingest pipeline (heuristic ships first).
///
/// Honest framing: copying the system-prompt verbatim from the Anthropic
/// blog without ingestion-via-paper-ingestor is a VERIFY-blocked source
/// per workflow/sessions/s-25120/completion-drive/grounding/E-17-grounding.md.
/// The template is captured here as a stable type so that when the
/// paper-ingestor ingest lands, only the system_prompt + per_chunk_prompt
/// strings need to be replaced — no caller-side changes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextualPromptTemplate {
    /// System prompt sent once per file ingest pass (prompt-caching amortises
    /// the cost across that file's chunks).
    pub system_prompt: String,
    /// Per-chunk user prompt. `{chunk_body}` is the substitution token.
    pub per_chunk_prompt: String,
    /// Max tokens budget for the generated context (target: 50-100 tokens).
    pub max_tokens: u32,
}

impl ContextualPromptTemplate {
    /// Default Anthropic Contextual Retrieval template (verbatim wording
    /// pending paper-ingestor ingestion of anthropic.com/news/contextual-retrieval).
    pub fn anthropic_default() -> Self {
        Self {
            system_prompt: "You are a context-generator for retrieval-augmented \
                            search. For each chunk, write a 1-2 sentence context \
                            describing where the chunk lives in the file and what \
                            it accomplishes. Keep responses under 100 tokens."
                .to_string(),
            per_chunk_prompt: "Chunk:\n{chunk_body}\n\nContext:".to_string(),
            max_tokens: 100,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bm25::ChunkKind;
    use std::path::PathBuf;

    fn mk_chunk(content: &str, line_start: u32, kind: ChunkKind) -> BM25Chunk {
        BM25Chunk {
            path: PathBuf::from("src/lib.rs"),
            content: content.to_string(),
            line_start,
            line_end: line_start.saturating_add(10),
            kind,
        }
    }

    #[test]
    fn blake3_short_hex_is_deterministic_and_16_chars() {
        let a = blake3_short_hex(b"hello world");
        let b = blake3_short_hex(b"hello world");
        assert_eq!(a, b, "blake3 hex must be deterministic across calls");
        assert_eq!(a.len(), 16, "short hex must be exactly 16 chars");
    }

    #[test]
    fn blake3_short_hex_differs_for_distinct_inputs() {
        let a = blake3_short_hex(b"alpha");
        let b = blake3_short_hex(b"beta");
        assert_ne!(a, b, "distinct inputs must produce distinct hashes");
    }

    #[test]
    fn build_heuristic_context_includes_file_and_kind() {
        let chunk = mk_chunk("fn body", 5, ChunkKind::Function);
        let source = "// line 1\n// line 2\nfn foo() {}\n// line 4\n";
        let prefix = build_heuristic_context(&chunk, source);
        assert!(prefix.starts_with(CONTEXT_HINT_HEADER));
        assert!(prefix.contains("file lib.rs"));
        assert!(prefix.contains("kind function"));
    }

    #[test]
    fn build_heuristic_context_finds_preceding_signature() {
        // Chunk starts at line 5; the signature on line 3 should be found.
        let source = "// header\n\nfn outer() {\n    let x = 1;\n    let y = 2;\n";
        let chunk = mk_chunk("let z = 3;", 5, ChunkKind::Function);
        let prefix = build_heuristic_context(&chunk, source);
        assert!(prefix.contains("near:"), "prefix should include near anchor");
        assert!(prefix.contains("fn outer"));
    }

    #[test]
    fn build_heuristic_context_truncates_at_max() {
        // Force a very long file_name to test the truncation path.
        let long_path = PathBuf::from(format!("{}.rs", "x".repeat(500)));
        let chunk = BM25Chunk {
            path: long_path,
            content: "body".to_string(),
            line_start: 1,
            line_end: 1,
            kind: ChunkKind::File,
        };
        let prefix = build_heuristic_context(&chunk, "");
        assert!(
            prefix.len() <= MAX_CONTEXT_PREFIX_CHARS,
            "prefix len {} must not exceed cap {}",
            prefix.len(),
            MAX_CONTEXT_PREFIX_CHARS
        );
    }

    #[test]
    fn contextualize_chunk_prepends_delimiter_and_prefix() {
        let chunk = mk_chunk("let z = 3;", 5, ChunkKind::Function);
        let source = "// header\n\nfn outer() {\n    let x = 1;\n    let y = 2;\n";
        let result = contextualize_chunk(chunk.clone(), source, None);
        assert!(result.fresh_prefix, "no cache -> fresh prefix");
        assert!(result.chunk.content.starts_with(CONTEXT_HINT_HEADER));
        assert!(result.chunk.content.contains(CONTEXT_PREFIX_DELIMITER));
        assert!(result.chunk.content.contains("let z = 3;"));
        assert_eq!(result.chunk.line_start, chunk.line_start);
        assert_eq!(result.chunk.line_end, chunk.line_end);
        assert_eq!(result.chunk.kind, chunk.kind);
        assert_eq!(result.prefix_hash.len(), 16);
        assert_eq!(result.source_hash.len(), 16);
    }

    #[test]
    fn contextualize_chunk_regen_skip_when_hash_matches() {
        let chunk = mk_chunk("body", 2, ChunkKind::Function);
        let source = "// header\nfn outer() {}\nlet x = 1;\n";
        let h = blake3_short_hex(source.as_bytes());
        let result = contextualize_chunk(chunk.clone(), source, Some(&h));
        assert!(!result.fresh_prefix, "cache match -> skip path");
        assert_eq!(result.chunk.content, "body", "body unchanged on skip");
        assert_eq!(result.source_hash, h);
    }

    #[test]
    fn contextualize_chunk_regen_runs_when_hash_mismatches() {
        let chunk = mk_chunk("body", 2, ChunkKind::Function);
        let source = "fn outer() {}\n";
        let result = contextualize_chunk(chunk, source, Some("0000000000000000"));
        assert!(result.fresh_prefix, "hash mismatch -> regen");
        assert!(result.chunk.content.starts_with(CONTEXT_HINT_HEADER));
    }

    #[test]
    fn is_contextualised_detects_prefix() {
        let normal = mk_chunk("let x = 1;", 1, ChunkKind::Function);
        assert!(!is_contextualised(&normal));

        let already = BM25Chunk {
            path: PathBuf::from("a.rs"),
            content: format!("{CONTEXT_HINT_HEADER}file a.rs{CONTEXT_PREFIX_DELIMITER}body"),
            line_start: 1,
            line_end: 1,
            kind: ChunkKind::Function,
        };
        assert!(is_contextualised(&already));
    }

    #[test]
    fn scan_preceding_anchor_handles_start_of_file() {
        let source = "fn foo() {}\nlet x = 1;\n";
        // start_line == 1 -> no preceding context.
        assert!(scan_preceding_anchor(source, 1).is_none());
        // start_line == 0 -> no preceding context (defensive).
        assert!(scan_preceding_anchor(source, 0).is_none());
    }

    #[test]
    fn looks_like_anchor_recognises_common_signatures() {
        assert!(looks_like_anchor("fn foo()"));
        assert!(looks_like_anchor("pub fn bar()"));
        assert!(looks_like_anchor("async fn baz()"));
        assert!(looks_like_anchor("def py_fn(self):"));
        assert!(looks_like_anchor("class MyClass:"));
        assert!(looks_like_anchor("struct Foo {"));
        assert!(looks_like_anchor("//! rustdoc inner heading"));
        assert!(looks_like_anchor("/// rustdoc outer heading"));
        assert!(looks_like_anchor("# heading"));
        assert!(!looks_like_anchor("let x = 1;"));
        assert!(!looks_like_anchor(""));
    }

    #[test]
    fn anthropic_default_template_is_non_empty() {
        let t = ContextualPromptTemplate::anthropic_default();
        assert!(!t.system_prompt.is_empty());
        assert!(t.per_chunk_prompt.contains("{chunk_body}"));
        assert_eq!(t.max_tokens, 100);
    }

    #[test]
    fn truncate_on_char_boundary_handles_multibyte() {
        // Test with multibyte UTF-8 chars to ensure no panic on boundaries.
        let s = "ééééééééééééééééé"; // each é is 2 bytes
        let t = truncate_on_char_boundary(s, 5);
        // 5 bytes mid-codepoint -> floored to 4 (2 chars).
        assert!(t.len() <= 5);
        assert!(s.starts_with(&t));
    }
}
