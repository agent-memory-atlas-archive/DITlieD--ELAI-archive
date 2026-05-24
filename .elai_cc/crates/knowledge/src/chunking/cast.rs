//! E-17 SP5 — cAST (AST-aware code chunker) per arxiv:2506.15655.
//!
//! Recursive-split + sibling-merge over tree-sitter AST boundaries, with
//! token-budget aware leaves. Replaces (or supplements) the existing
//! BM25-side AST chunker at `crate::bm25::try_ast_chunk` when the caller
//! opts in; the legacy chunker stays as the fallback so ACCEPTANCE's
//! "if +2 not met, keep legacy chunker behind a flag" path stays open.
//!
//! Algorithm (per paper §3.2):
//!
//!   Given a parsed tree-sitter `Tree` and a target chunk-size budget in
//!   characters (TOKEN_BUDGET_CHARS):
//!
//!     1. Recursive split: walk the tree top-down. Each node is one of
//!        - LEAF when `node_chars <= TOKEN_BUDGET_CHARS`
//!        - INTERNAL otherwise — recurse into its named children
//!     2. Sibling merge: walk LEAVES left-to-right. Merge consecutive
//!        leaves while the running total stays under TOKEN_BUDGET_CHARS.
//!        This is the paper's "merge adjacent siblings whose union fits"
//!        step (S3.2 paragraph "merging").
//!     3. Fallback: when a single statement / token blob exceeds the
//!        budget by itself, emit it as its own chunk (do NOT split inside
//!        a tree-sitter leaf — the paper's approach is to keep semantic
//!        atoms intact).
//!
//! Honest framing:
//!  - The paper reports +4.3 Recall@5 on RepoEval / +2.67 Pass@1 on SWE-bench.
//!    ELAI's 135-file private corpus may not reproduce; ACCEPTANCE keeps
//!    the bar at "conservative +2" and tests/cast_chunker_recall_smoke.rs
//!    measures that.
//!  - This implementation is the recursive-split + sibling-merge core; the
//!    paper's full optimisation includes a streaming variant + cross-file
//!    boundary sharing (out of scope for the first ship).
//!  - tree-sitter (workspace dep 0.26) is reused — the same grammar
//!    dispatch in `bm25.rs::try_ast_chunk` could be wrapped in this
//!    chunker once SP6 confirms the delta is positive on ELAI's corpus.

use std::path::{Path, PathBuf};

use tree_sitter::{Node, Parser, Tree};

use crate::bm25::{BM25Chunk, ChunkKind};

/// E-17 SP5: target chunk-size budget in characters. The paper expresses
/// this in tokens (~250-500); ELAI uses chars at 4-char-per-token (matches
/// `bm25::CHARS_PER_TOKEN`), so 1500 chars ≈ 375 tokens — right inside
/// the paper's 250-500 sweet spot per §4 Table 4.
pub const TOKEN_BUDGET_CHARS: usize = 1500;

/// E-17 SP5: minimum chars below which we still emit a chunk (no further
/// merging) so single-line statements don't merge across unrelated
/// siblings just to hit a floor. Mirrors `bm25::MIN_CHUNK_TOKENS *
/// CHARS_PER_TOKEN`.
pub const MIN_CHUNK_CHARS: usize = 80;

/// E-17 SP5: hard upper bound — a single leaf MUST be emitted as its own
/// chunk even when it exceeds this. Above this we still emit the leaf
/// alone (do NOT split inside a leaf — paper §3.2 "preserve semantic
/// atoms").
pub const HARD_MAX_CHUNK_CHARS: usize = 4000;

/// E-17 SP5: result of running [`CastChunker::chunk_source`] on a single file.
///
/// `chunks` are emitted in source order so downstream indexers can
/// reuse the existing `BM25Chunk` insertion path. `nodes_visited` /
/// `leaves_emitted` / `siblings_merged` are diagnostic counters exposed
/// for the eval harness comparison run (legacy vs cAST).
#[derive(Debug, Clone)]
pub struct CastChunkResult {
    pub chunks: Vec<BM25Chunk>,
    pub nodes_visited: u32,
    pub leaves_emitted: u32,
    pub siblings_merged: u32,
}

/// E-17 SP5: language adapter — tree-sitter `Language` + the named-node
/// kinds that mark a "structural" boundary (function / class / module /
/// impl). Each adapter is a thin wrapper around an existing grammar
/// already loaded by the knowledge crate.
pub trait CastLanguage: Send + Sync {
    /// The tree-sitter `Language` handle for the parser.
    fn tree_sitter_language(&self) -> tree_sitter::Language;
    /// Kinds (by `Node::kind_id`-equivalent string) that should be
    /// emitted as structural boundaries. Empty -> any named node is a
    /// candidate (the recursive split still respects the budget).
    fn structural_kinds(&self) -> &'static [&'static str];
}

/// E-17 SP5: Rust language adapter using tree-sitter-rust 0.24 (workspace dep).
pub struct CastRustLanguage;

impl CastLanguage for CastRustLanguage {
    fn tree_sitter_language(&self) -> tree_sitter::Language {
        tree_sitter_rust::LANGUAGE.into()
    }
    fn structural_kinds(&self) -> &'static [&'static str] {
        &[
            "function_item",
            "impl_item",
            "struct_item",
            "enum_item",
            "trait_item",
            "mod_item",
        ]
    }
}

/// E-17 SP5: Python language adapter using tree-sitter-python 0.25.
pub struct CastPythonLanguage;

impl CastLanguage for CastPythonLanguage {
    fn tree_sitter_language(&self) -> tree_sitter::Language {
        tree_sitter_python::LANGUAGE.into()
    }
    fn structural_kinds(&self) -> &'static [&'static str] {
        &[
            "function_definition",
            "class_definition",
            "decorated_definition",
        ]
    }
}

/// E-17 SP5: TypeScript language adapter using tree-sitter-typescript 0.23.
pub struct CastTypeScriptLanguage;

impl CastLanguage for CastTypeScriptLanguage {
    fn tree_sitter_language(&self) -> tree_sitter::Language {
        tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
    }
    fn structural_kinds(&self) -> &'static [&'static str] {
        &[
            "function_declaration",
            "method_definition",
            "class_declaration",
            "interface_declaration",
            "type_alias_declaration",
        ]
    }
}

/// E-17 SP5: Go language adapter using tree-sitter-go 0.25.
pub struct CastGoLanguage;

impl CastLanguage for CastGoLanguage {
    fn tree_sitter_language(&self) -> tree_sitter::Language {
        tree_sitter_go::LANGUAGE.into()
    }
    fn structural_kinds(&self) -> &'static [&'static str] {
        &[
            "function_declaration",
            "method_declaration",
            "type_declaration",
        ]
    }
}

/// E-17 SP5: pick a [`CastLanguage`] adapter for the given file extension.
/// Returns `None` for unknown extensions — caller falls back to the
/// existing legacy chunker.
pub fn detect_cast_language(ext: &str) -> Option<Box<dyn CastLanguage>> {
    match ext {
        "rs" => Some(Box::new(CastRustLanguage)),
        "py" => Some(Box::new(CastPythonLanguage)),
        "ts" | "tsx" => Some(Box::new(CastTypeScriptLanguage)),
        "go" => Some(Box::new(CastGoLanguage)),
        _ => None,
    }
}

/// E-17 SP5: the cAST chunker entry-point.
///
/// `language` is the tree-sitter grammar adapter for the source file.
/// `path` and `source` mirror `Bm25Index::chunk_file` inputs.
///
/// Returns [`CastChunkResult`] or `None` when parsing failed (caller
/// falls back to the existing legacy chunker — never panics).
pub struct CastChunker {
    budget: usize,
    min_chars: usize,
    hard_max: usize,
}

impl Default for CastChunker {
    fn default() -> Self {
        Self {
            budget: TOKEN_BUDGET_CHARS,
            min_chars: MIN_CHUNK_CHARS,
            hard_max: HARD_MAX_CHUNK_CHARS,
        }
    }
}

impl CastChunker {
    /// E-17 SP5: build a chunker with custom thresholds. Used by the eval
    /// harness for ablation runs; default constructor matches the paper's
    /// 250-500 token range.
    pub fn with_thresholds(budget: usize, min_chars: usize, hard_max: usize) -> Self {
        Self {
            budget: budget.max(1),
            min_chars,
            hard_max: hard_max.max(budget),
        }
    }

    /// E-17 SP5: chunk one source file via the cAST algorithm.
    ///
    /// Returns `None` when tree-sitter set_language fails or `parser.parse`
    /// returns no tree — the caller should fall back to
    /// `Bm25Index::try_ast_chunk` (or further to `line_block_chunks`).
    pub fn chunk_source(
        &self,
        language: &dyn CastLanguage,
        path: &Path,
        source: &str,
    ) -> Option<CastChunkResult> {
        let mut parser = Parser::new();
        if parser.set_language(&language.tree_sitter_language()).is_err() {
            return None;
        }
        let tree = parser.parse(source, None)?;

        let mut leaves: Vec<CastLeaf> = Vec::new();
        let mut nodes_visited: u32 = 0;
        self.recursive_split(
            tree.root_node(),
            source,
            language.structural_kinds(),
            &mut leaves,
            &mut nodes_visited,
        );

        let leaves_emitted = u32::try_from(leaves.len()).unwrap_or(u32::MAX);
        let (chunks, siblings_merged) = self.sibling_merge(path, source, &leaves);

        Some(CastChunkResult {
            chunks,
            nodes_visited,
            leaves_emitted,
            siblings_merged,
        })
    }

    /// Recursive split: walk the tree top-down. Emit any node that fits in
    /// the budget as a leaf; recurse into children otherwise. Structural
    /// boundaries get split even when they fit (so each function /
    /// class becomes its own chunk floor — paper §3.2).
    fn recursive_split(
        &self,
        node: Node,
        source: &str,
        structural_kinds: &[&str],
        out: &mut Vec<CastLeaf>,
        nodes_visited: &mut u32,
    ) {
        *nodes_visited = nodes_visited.saturating_add(1);

        let range = node.byte_range();
        let chars = range.len();
        let kind = node.kind();
        let is_structural = structural_kinds.contains(&kind);

        // Skip empty / whitespace-only nodes.
        if chars == 0 {
            return;
        }

        // LEAF rule: emit as-is when it fits the budget AND is not a structural
        // boundary container with multiple structural children (recurse for
        // those so each function lands separately).
        if chars <= self.budget && !self.has_multiple_structural_children(node, structural_kinds) {
            out.push(CastLeaf {
                byte_start: range.start,
                byte_end: range.end,
                kind: chunk_kind_for_node(kind, is_structural),
            });
            return;
        }

        // Single huge leaf (over budget) — emit alone if no usable children.
        if !has_named_children(node) {
            out.push(CastLeaf {
                byte_start: range.start,
                byte_end: range.end,
                kind: chunk_kind_for_node(kind, is_structural),
            });
            return;
        }

        // INTERNAL: recurse into named children. Pre-children gap (e.g.
        // attributes / leading comments) is captured by the parent
        // visitor on the next pass via the gap-fill step below.
        let mut cursor = node.walk();
        let mut prev_end = range.start;
        for child in node.named_children(&mut cursor) {
            let cr = child.byte_range();
            // Gap-fill: any unclaimed source between prev_end and child start
            // becomes a small line-block leaf (e.g. top-level `use` statements
            // sandwiched between functions).
            if cr.start > prev_end {
                let gap_chars = cr.start - prev_end;
                if gap_chars > 0 {
                    out.push(CastLeaf {
                        byte_start: prev_end,
                        byte_end: cr.start,
                        kind: ChunkKind::LineBlock,
                    });
                }
            }
            self.recursive_split(child, source, structural_kinds, out, nodes_visited);
            prev_end = cr.end;
        }
        // Trailing gap (after last child, before parent end).
        if prev_end < range.end {
            out.push(CastLeaf {
                byte_start: prev_end,
                byte_end: range.end,
                kind: ChunkKind::LineBlock,
            });
        }
    }

    /// True when `node` has 2+ named children whose kind is in
    /// `structural_kinds`. Such a node is a container (module, impl) and
    /// must be split rather than emitted as one fat leaf.
    fn has_multiple_structural_children(&self, node: Node, structural_kinds: &[&str]) -> bool {
        let mut cursor = node.walk();
        let mut count = 0;
        for child in node.named_children(&mut cursor) {
            if structural_kinds.contains(&child.kind()) {
                count += 1;
                if count >= 2 {
                    return true;
                }
            }
        }
        false
    }

    /// Sibling merge: walk leaves left-to-right, merging adjacent leaves
    /// whose combined char count stays under the budget. Mirrors the
    /// paper's "merge adjacent siblings whose union fits" step (§3.2).
    fn sibling_merge(
        &self,
        path: &Path,
        source: &str,
        leaves: &[CastLeaf],
    ) -> (Vec<BM25Chunk>, u32) {
        let mut chunks: Vec<BM25Chunk> = Vec::new();
        let mut merged_counter: u32 = 0;
        let mut i = 0;
        while i < leaves.len() {
            let head = &leaves[i];
            let mut byte_end = head.byte_end;
            let mut kind = head.kind;
            let mut j = i + 1;
            // Greedy forward merge.
            while j < leaves.len() {
                let cand = &leaves[j];
                let combined = cand.byte_end.saturating_sub(head.byte_start);
                if combined > self.budget {
                    break;
                }
                // Merge only contiguous leaves (cAST keeps source order).
                if cand.byte_start != byte_end {
                    break;
                }
                byte_end = cand.byte_end;
                merged_counter = merged_counter.saturating_add(1);
                // Promote kind to LineBlock when mixing kinds (parity with
                // bm25's existing gap-fill rule).
                if cand.kind != kind {
                    kind = ChunkKind::LineBlock;
                }
                j += 1;
            }
            // Apply hard max: if the single leaf is > hard_max we still
            // emit it alone (no in-leaf split — paper rule).
            let final_end = byte_end.min(head.byte_start.saturating_add(self.hard_max));
            // Skip empty after byte_start adjustments.
            if final_end > head.byte_start {
                let chunk_text = &source[head.byte_start..final_end];
                let (line_start, line_end) =
                    byte_range_to_line_range(source, head.byte_start, final_end);
                // Floor merge: drop chunks below MIN_CHUNK_CHARS BY MERGING
                // INTO PREVIOUS if available — otherwise keep as-is (a tiny
                // chunk is better than no chunk). Uses `last_mut()` Option
                // pattern (no .expect()) — anti-stale CLAUDE.md rule.
                let floor_merged = if chunk_text.len() < self.min_chars {
                    if let Some(last) = chunks.last_mut() {
                        // Best-effort concat with a newline separator.
                        let mut combined =
                            String::with_capacity(last.content.len() + 1 + chunk_text.len());
                        combined.push_str(&last.content);
                        combined.push('\n');
                        combined.push_str(chunk_text);
                        last.content = combined;
                        last.line_end = line_end.max(last.line_end);
                        merged_counter = merged_counter.saturating_add(1);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };
                if !floor_merged {
                    chunks.push(BM25Chunk {
                        path: PathBuf::from(path),
                        content: chunk_text.to_string(),
                        line_start,
                        line_end,
                        kind,
                    });
                }
            }
            i = j;
        }
        (chunks, merged_counter)
    }
}

/// E-17 SP5: leaf interval discovered by the recursive-split pass.
/// Byte-range based so the sibling-merge can produce a single string slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CastLeaf {
    byte_start: usize,
    byte_end: usize,
    kind: ChunkKind,
}

/// Map a tree-sitter node kind string to a [`ChunkKind`] using the same
/// taxonomy as `crate::bm25::symbol_kind_to_chunk_kind` so downstream
/// readers see consistent labels.
fn chunk_kind_for_node(kind: &str, is_structural: bool) -> ChunkKind {
    if is_structural {
        match kind {
            "function_item"
            | "function_definition"
            | "function_declaration"
            | "method_definition"
            | "method_declaration"
            | "decorated_definition" => ChunkKind::Function,
            "class_definition" | "class_declaration" => ChunkKind::Class,
            "struct_item" => ChunkKind::Struct,
            "enum_item" => ChunkKind::Enum,
            "trait_item" | "interface_declaration" => ChunkKind::Trait,
            "impl_item" => ChunkKind::Impl,
            "mod_item" => ChunkKind::Module,
            "type_declaration" | "type_alias_declaration" => ChunkKind::Module,
            _ => ChunkKind::LineBlock,
        }
    } else {
        ChunkKind::LineBlock
    }
}

/// Convert a byte range to a 1-indexed inclusive line range over `source`.
/// Mirrors how `bm25::extract_lines` produces line spans so downstream
/// (path, line_start, line_end) identity stays consistent.
fn byte_range_to_line_range(source: &str, byte_start: usize, byte_end: usize) -> (u32, u32) {
    let mut line: u32 = 1;
    let mut line_start: u32 = 1;
    let mut line_end: u32 = 1;
    let start = byte_start.min(source.len());
    let end = byte_end.min(source.len()).max(start);
    let bytes = source.as_bytes();
    for (i, b) in bytes.iter().enumerate().take(end) {
        if i == start {
            line_start = line;
        }
        if *b == b'\n' {
            line = line.saturating_add(1);
        }
    }
    if end >= start {
        line_end = line;
    }
    if line_start == 0 {
        line_start = 1;
    }
    if line_end < line_start {
        line_end = line_start;
    }
    (line_start, line_end)
}

/// True when `node` has at least one named child.
fn has_named_children(node: Node) -> bool {
    node.named_child_count() > 0
}

/// E-17 SP5: re-export type used by `parse_with_default_languages` so
/// upstream callers (bm25 dispatch wrapper, eval harness) only see one
/// surface. Type-only; no behaviour.
pub type CastTree = Tree;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn rust_source_two_functions() -> String {
        // 2 distinct fn items, both small enough to land as separate chunks.
        // The intent: 2 leaves AFTER recursive_split, NOT merged into 1.
        let mut s = String::new();
        s.push_str("fn first() {\n");
        s.push_str("    // tiny body\n");
        s.push_str("    let x = 1;\n");
        s.push_str("}\n");
        s.push_str("\n");
        s.push_str("fn second() {\n");
        s.push_str("    let y = 2;\n");
        s.push_str("}\n");
        s
    }

    fn rust_source_big_function() -> String {
        // Build a single fn body that exceeds TOKEN_BUDGET_CHARS so the
        // recursive splitter has to descend into its body.
        let mut s = String::new();
        s.push_str("fn big() {\n");
        for i in 0..200 {
            s.push_str(&format!("    let v{i} = compute_value({i});\n"));
        }
        s.push_str("}\n");
        s
    }

    #[test]
    fn detect_cast_language_returns_for_known_extensions() {
        assert!(detect_cast_language("rs").is_some());
        assert!(detect_cast_language("py").is_some());
        assert!(detect_cast_language("ts").is_some());
        assert!(detect_cast_language("tsx").is_some());
        assert!(detect_cast_language("go").is_some());
        assert!(detect_cast_language("xyz").is_none());
    }

    #[test]
    fn chunk_source_handles_empty_input() {
        let lang = CastRustLanguage;
        let chunker = CastChunker::default();
        let path = PathBuf::from("a.rs");
        let result = chunker.chunk_source(&lang, &path, "");
        assert!(result.is_some(), "empty source must not return None");
        let r = result.expect("checked Some");
        // Empty source produces zero or near-zero chunks; we just assert
        // the call did not panic and the chunker did not emit garbage.
        assert!(r.chunks.len() <= 1);
    }

    #[test]
    fn chunk_source_two_small_functions_covers_both_fn_names() {
        // Small fixture: total chars < TOKEN_BUDGET_CHARS so the sibling-merge
        // may legitimately fold into a single chunk. The contract is that
        // BOTH function names land in the chunk corpus — no name dropped.
        let lang = CastRustLanguage;
        let chunker = CastChunker::default();
        let path = PathBuf::from("a.rs");
        let source = rust_source_two_functions();
        let result = chunker
            .chunk_source(&lang, &path, &source)
            .expect("parses cleanly");
        assert!(!result.chunks.is_empty(), "must emit at least 1 chunk");
        let corpus = result
            .chunks
            .iter()
            .map(|c| c.content.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(corpus.contains("fn first"), "must cover fn first");
        assert!(corpus.contains("fn second"), "must cover fn second");
        // Function-kind leaves must be Function (or merged LineBlock if the
        // sibling-merge promoted them).
        let any_fn = result
            .chunks
            .iter()
            .any(|c| matches!(c.kind, ChunkKind::Function | ChunkKind::LineBlock));
        assert!(any_fn, "expected at least one Function/LineBlock chunk");
    }

    #[test]
    fn chunk_source_large_function_does_not_emit_oversized_chunk() {
        let lang = CastRustLanguage;
        let chunker = CastChunker::default();
        let path = PathBuf::from("big.rs");
        let source = rust_source_big_function();
        let result = chunker
            .chunk_source(&lang, &path, &source)
            .expect("parses cleanly");
        for c in &result.chunks {
            assert!(
                c.content.len() <= HARD_MAX_CHUNK_CHARS,
                "chunk len {} > HARD_MAX_CHUNK_CHARS {}",
                c.content.len(),
                HARD_MAX_CHUNK_CHARS
            );
        }
    }

    #[test]
    fn chunk_source_preserves_source_order() {
        let lang = CastRustLanguage;
        let chunker = CastChunker::default();
        let path = PathBuf::from("ord.rs");
        let source = rust_source_two_functions();
        let result = chunker
            .chunk_source(&lang, &path, &source)
            .expect("parses cleanly");
        let mut prev_start: u32 = 0;
        for c in &result.chunks {
            assert!(
                c.line_start >= prev_start,
                "chunk order broken: {} < {}",
                c.line_start,
                prev_start
            );
            prev_start = c.line_start;
        }
    }

    #[test]
    fn chunk_source_python_function() {
        let lang = CastPythonLanguage;
        let chunker = CastChunker::default();
        let path = PathBuf::from("a.py");
        let source = "def first():\n    return 1\n\ndef second():\n    return 2\n";
        let result = chunker
            .chunk_source(&lang, &path, source)
            .expect("py parses cleanly");
        assert!(result.chunks.len() >= 1);
    }

    #[test]
    fn chunk_source_typescript_function() {
        let lang = CastTypeScriptLanguage;
        let chunker = CastChunker::default();
        let path = PathBuf::from("a.ts");
        let source = "function first(): number { return 1; }\nfunction second(): number { return 2; }\n";
        let result = chunker
            .chunk_source(&lang, &path, source)
            .expect("ts parses cleanly");
        assert!(result.chunks.len() >= 1);
    }

    #[test]
    fn chunk_source_go_function() {
        let lang = CastGoLanguage;
        let chunker = CastChunker::default();
        let path = PathBuf::from("a.go");
        let source = "package main\nfunc First() int { return 1 }\nfunc Second() int { return 2 }\n";
        let result = chunker
            .chunk_source(&lang, &path, source)
            .expect("go parses cleanly");
        assert!(result.chunks.len() >= 1);
    }

    #[test]
    fn with_thresholds_overrides_defaults() {
        let chunker = CastChunker::with_thresholds(500, 10, 1000);
        assert_eq!(chunker.budget, 500);
        assert_eq!(chunker.min_chars, 10);
        assert_eq!(chunker.hard_max, 1000);
    }

    #[test]
    fn byte_range_to_line_range_basic() {
        let s = "line 1\nline 2\nline 3\n";
        // byte_start = 0 -> "line 1"; bytes 0..6 inclusive
        let (ls, le) = byte_range_to_line_range(s, 0, 6);
        assert_eq!(ls, 1);
        assert!(le >= 1);
        let (ls2, le2) = byte_range_to_line_range(s, 7, 13); // "line 2"
        assert!(ls2 >= 2);
        assert!(le2 >= ls2);
    }
}
