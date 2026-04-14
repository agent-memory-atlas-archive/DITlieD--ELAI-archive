use std::path::{Path, PathBuf};

pub mod arag;
pub mod ast_adapter;
pub mod bm25;
pub mod catrag;
pub mod conventions;
pub mod degradation;
pub mod dkg;
pub mod dkg_adapter;
pub mod error;
pub mod freshness;
pub mod index_version;
pub mod pagerank;
pub mod pipeline;
pub mod project_profile;
pub mod query_trace;
pub mod query_validator;
pub mod reranker;
pub mod shortcoder;
#[cfg(feature = "splade")]
pub mod splade;
pub mod startup;
pub mod telescope;
pub mod tier3;
pub mod vector;

// ---------------------------------------------------------------------------
// Core enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ReadinessStatus {
    Ready,
    Degraded,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DkgReadiness {
    Ready,
    Empty,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SymbolKind {
    Function,
    Struct,
    Enum,
    Trait,
    Impl,
    Module,
    Const,
    Static,
    TypeAlias,
    Macro,
}

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct DependencyGraph {
    pub node_count: usize,
    pub edge_count: usize,
    pub files_parsed: usize,
}

#[derive(Debug, Clone)]
pub struct QueryStrategy;

#[derive(Debug, Clone)]
pub struct CodeChunk;

#[derive(Debug, Clone)]
pub struct DkgNodeInfo {
    pub node_id: i64,
    pub symbol_name: String,
    pub file_path: PathBuf,
    pub signature: String,
    pub kind: SymbolKind,
    pub start_line: u32,
    pub end_line: u32,
}

/// Internal node extracted by a grammar parser — not part of the public API contract.
#[derive(Debug, Clone)]
pub struct DkgNode {
    pub name: String,
    pub kind: SymbolKind,
    pub signature: String,
    pub start_line: u32,
    pub end_line: u32,
    pub parent_idx: Option<usize>,
}

// ---------------------------------------------------------------------------
// Traits
// ---------------------------------------------------------------------------

/// Object-safe trait for language-specific tree-sitter grammars.
pub trait LanguageGrammar {
    fn language_id(&self) -> &'static str;
    fn tree_sitter_language(&self) -> tree_sitter::Language;
    fn extract_nodes(&self, tree: &tree_sitter::Tree, source: &[u8]) -> Vec<DkgNode>;
}

/// Top-level knowledge engine trait. Implementations aggregate DKG, vector,
/// and BM25 sub-engines. Full impl deferred to Phase 1I.
pub trait KnowledgeEngine {
    fn build_graph(&mut self, project_root: &Path) -> Result<DependencyGraph, KnowledgeError>;
    fn query(&self, query: &str, strategy: QueryStrategy) -> Result<Vec<CodeChunk>, KnowledgeError>;
    fn rerank(&self, query: &str, candidates: Vec<CodeChunk>) -> Result<Vec<CodeChunk>, KnowledgeError>;
    fn update(&mut self, file_changed: &Path) -> Result<(), KnowledgeError>;
    fn readiness_check(&self) -> ReadinessStatus;
}

// Re-export DkgEngine at crate root for convenience.
pub use dkg::DkgEngine;
pub use ast_adapter::TreeSitterAstEvaluator;
pub use dkg_adapter::DkgQuerierAdapter;
pub use error::KnowledgeError;
pub use project_profile::{detect_project_profile, populate_tier2_conventions, serialize_profile, deserialize_profile};
pub use tier3::populate_tier3;
pub use shortcoder::ShortCoder;
pub use telescope::{assemble_telescope, TelescopeContext};
