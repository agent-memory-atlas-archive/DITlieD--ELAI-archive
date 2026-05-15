// B-29 Phase 1 — symbol graph types.
//
// NOTE: `SymbolKind` here is the *call-graph* variant (9 language-level kinds).
// It is DISTINCT from `crate::SymbolKind` (10 DKG node kinds: Function/Struct/…/Macro).
// Do NOT re-export this type at the knowledge crate root — qualify as
// `knowledge::symbol_graph::types::SymbolKind` when referencing from outside
// this module to avoid the name collision.
//
// `EntryPoint` here is also DISTINCT from `elai_core::EntryPoint` (project
// entry-point descriptor). Same mitigation: submodule-only, not re-exported at
// crate root.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Call-graph symbol kind (language-level classification).
///
/// Distinct from `crate::SymbolKind` (DKG node kind). See module-level note.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolKind {
    Function,
    Method,
    Class,
    Struct,
    Trait,
    Module,
    Enum,
    Interface,
    Type,
}

/// Confidence tier for cross-file call resolution.
///
/// Maps to SocratiCode's four-tier strategy from graph-symbol-resolution.ts:
/// 0 candidates -> Unresolved, 1 -> Unique, >1 -> MultipleCandidates,
/// same-file match -> Local.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Confidence {
    /// Resolved to a single callee in the same file.
    Local,
    /// Resolved to exactly one callee across all search tiers.
    Unique,
    /// Resolved to more than one candidate — caller must disambiguate.
    MultipleCandidates,
    /// No matching callee found in any tier.
    Unresolved,
}

// ---------------------------------------------------------------------------
// Structs
// ---------------------------------------------------------------------------

/// Location of a call within a source file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallSite {
    pub file: String,
    pub line: u32,
}

/// Lightweight pointer to a symbol: the file it lives in and its unique id.
///
/// Used as the element type in `symbol_name_index` blobs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolRef {
    pub file: String,
    pub id: String,
}

/// A single symbol node in the call graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolNode {
    /// Stable unique identifier (e.g. `<file>::<qualified_name>`).
    pub id: String,
    /// Simple (unqualified) name.
    pub name: String,
    /// Fully qualified name including enclosing scopes.
    pub qualified_name: String,
    /// Language-level kind of this symbol.
    pub kind: SymbolKind,
    /// Source file path (repo-relative or absolute, consistent within a build).
    pub file: String,
    /// 1-based start line.
    pub line: u32,
    /// 1-based end line (inclusive).
    pub end_line: u32,
    /// Language identifier (e.g. "rust", "python", "typescript").
    pub language: String,
}

/// A directed call edge from a caller symbol to a (possibly unresolved) callee.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolEdge {
    /// `id` of the calling symbol (SymbolNode::id).
    pub caller_id: String,
    /// Raw callee name as it appears at the call site.
    pub callee_name: String,
    /// Resolved candidate ids. Empty when `confidence == Unresolved`.
    pub callee_candidates: Vec<String>,
    /// Resolution confidence for this edge.
    pub confidence: Confidence,
    /// Location of the call expression.
    pub call_site: CallSite,
}

/// A well-known entry point for a project (e.g. `main`, `__init__`).
///
/// Distinct from `elai_core::EntryPoint`. See module-level note.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryPoint {
    /// `id` of the corresponding SymbolNode.
    pub id: String,
    /// Simple name (e.g. "main").
    pub name: String,
    /// Source file containing this entry point.
    pub file: String,
    /// 1-based line number, if known.
    pub line: Option<u32>,
    /// Human-readable reason this was classified as an entry point.
    pub reason: String,
}

/// Aggregate metadata for a built symbol graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolGraphMeta {
    /// Project identifier (used as primary key in `symbol_graph_meta` table).
    pub project_id: String,
    /// Total number of SymbolNode rows persisted.
    pub symbol_count: i64,
    /// Total number of SymbolEdge rows persisted.
    pub edge_count: i64,
    /// Total number of distinct source files indexed.
    pub file_count: i64,
    /// Percentage (0.0–100.0) of edges whose confidence is Unresolved.
    /// Spec contract (B-29 WIRING_CONTRACT stub_ban): must be < 50.0 on the ELAI repo.
    pub unresolved_edge_pct: f64,
    /// Wall-clock timestamp of the build (ClockProvider::now_ms; do NOT use SystemTime directly).
    pub built_at: i64,
    /// Schema version written into this row (should match B-29 v3).
    pub schema_version: i64,
}

/// Per-file payload stored in the `symbol_file_payload` table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolGraphFilePayload {
    /// Source file path (same as `symbol_file_payload.file` PK).
    pub file: String,
    /// Language identifier.
    pub language: String,
    /// blake3 hex hash of file contents at index time.
    pub content_hash: String,
    /// All symbols defined in this file. (SQL column: `symbols_msgpack`.)
    pub symbols: Vec<SymbolNode>,
    /// All outgoing call edges from symbols in this file. (SQL column: `edges_msgpack`.)
    pub outgoing_calls: Vec<SymbolEdge>,
}
