//! Adaptive chunking for minified files + universal char cap.
//!
//! Public path: `knowledge::chunking::*` (declared in `lib.rs`).
//!
//! Submodules:
//!   * `constants` — MAX_AVG_LINE_LENGTH (500), MAX_CHUNK_CHARS (2000).
//!   * `detect`    — `is_minified(content, line_count) -> bool`.
//!   * `character` — `chunk_by_characters(path, content) -> Vec<BM25Chunk>`.
//!   * `cap`       — `apply_char_cap`, `floor_char_boundary`.
//!
//! Reference: docs/.source/SocratiCode-main/src/services/indexer.ts:283-465
//! (chunkFileContent dispatch order: minified-first, applyCharCap-last).

pub mod constants;
pub mod detect;
pub mod cap;
pub mod character;
// E-17 SP5: AST-aware code chunker (cAST per arxiv:2506.15655). Recursive
// split + sibling merge across tree-sitter named-node boundaries, with
// token-budget aware leaves. Ships as opt-in alongside the legacy
// bm25::try_ast_chunk path; ACCEPTANCE keeps the legacy chunker as
// fallback when SP6 eval delta is < +2 Recall@5.
pub mod cast;
// E-36 Tier-A: vision artifact types (foundation for 9th VerificationMethod).
// Types-only foundation; producer wiring deferred to E-36.B per
// workflow/reasoning/E-36-decisions.md.
pub mod image_artifact;
