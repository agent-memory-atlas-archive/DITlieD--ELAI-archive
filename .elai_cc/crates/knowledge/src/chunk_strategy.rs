//! C-pack C3 — chunking strategy selector.
//!
//! Given a language id (lower-case, as produced by
//! `dkg::ext_to_lang_id`-style mappings) and a file size in bytes,
//! `pick_strategy` returns the `ChunkStrategy` variant best suited to the
//! file. The selector is intentionally simple: a deterministic decision
//! table, no I/O, no heuristics that depend on file contents. Integration
//! into the BM25 chunker is deferred — `bm25.rs` is LOCKED for this
//! phase (see C-pack brief).
//!
//! Threshold rationale:
//!   * 50_000 bytes — large enough that AST-walking is expensive, small
//!     enough to catch most generated/vendored files. Hierarchical summary
//!     uses two-level chunking (file -> section -> chunk) and is cheaper
//!     amortised over the giant file.

/// Hard upper bound (bytes) above which AST-semantic / line-block
/// chunking is replaced with hierarchical-summary chunking.
pub const HIERARCHICAL_SUMMARY_THRESHOLD_BYTES: u64 = 50_000;

/// How the indexer should split a file into retrieval-sized chunks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkStrategy {
    /// Walk the tree-sitter AST and emit one chunk per top-level / nested
    /// declaration (function, class, struct, etc.). Preferred for code
    /// because chunk boundaries align with semantic units.
    ASTSemantic,
    /// Split on logical line blocks (paragraphs / contiguous non-blank
    /// runs). Fallback when no AST grammar is available.
    LineBlock,
    /// Fixed-width window with overlap. Used for prose / Markdown / plain
    /// text where neither AST nor line-blocks line up with retrieval
    /// units.
    SlidingWindow,
    /// Two-pass chunking for large files: first produce summary chunks,
    /// then index the summaries alongside leaf chunks. Used when the file
    /// is large enough that flat chunking would explode token budgets.
    HierarchicalSummary,
}

/// Language ids that have a registered tree-sitter grammar with reliable
/// declaration extraction. Keep in sync with `dkg::ext_to_lang_id` so a
/// caller that passes the language id derived from a file extension always
/// gets a sensible answer. `"svelte"` is included as a forward-looking
/// entry — the C-pack registers a minimal SvelteGrammar (C1).
const AST_LANGUAGES: &[&str] = &[
    "rust",
    "python",
    "typescript",
    "javascript",
    "tsx",
    "svelte",
    "go",
    "java",
    "c",
    "cpp",
    "csharp",
];

/// Prose-ish languages that benefit from sliding-window chunking.
const PROSE_LANGUAGES: &[&str] = &["md", "markdown", "mdx", "txt", "rst"];

/// Pick the chunking strategy for a file.
///
/// Rules (evaluated top-to-bottom):
///   1. `size_bytes > 50_000` → `HierarchicalSummary` (regardless of lang).
///   2. `lang` is a prose language → `SlidingWindow`.
///   3. `lang` is an AST-supported code language → `ASTSemantic`.
///   4. Otherwise → `LineBlock`.
///
/// Pure function: no I/O, no allocation, no panics.
pub fn pick_strategy(lang: &str, size_bytes: u64) -> ChunkStrategy {
    if size_bytes > HIERARCHICAL_SUMMARY_THRESHOLD_BYTES {
        return ChunkStrategy::HierarchicalSummary;
    }
    if PROSE_LANGUAGES.contains(&lang) {
        return ChunkStrategy::SlidingWindow;
    }
    if AST_LANGUAGES.contains(&lang) {
        return ChunkStrategy::ASTSemantic;
    }
    ChunkStrategy::LineBlock
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_rust_is_ast_semantic() {
        assert_eq!(pick_strategy("rust", 100), ChunkStrategy::ASTSemantic);
    }

    #[test]
    fn small_markdown_is_sliding_window() {
        assert_eq!(pick_strategy("md", 100), ChunkStrategy::SlidingWindow);
        assert_eq!(pick_strategy("markdown", 100), ChunkStrategy::SlidingWindow);
    }

    #[test]
    fn large_file_overrides_to_hierarchical_summary() {
        assert_eq!(
            pick_strategy("rust", HIERARCHICAL_SUMMARY_THRESHOLD_BYTES + 1),
            ChunkStrategy::HierarchicalSummary
        );
    }

    #[test]
    fn unknown_language_falls_back_to_line_block() {
        assert_eq!(pick_strategy("unknown", 100), ChunkStrategy::LineBlock);
    }
}
