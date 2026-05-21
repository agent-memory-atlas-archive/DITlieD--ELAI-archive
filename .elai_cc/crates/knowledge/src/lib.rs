use std::path::{Path, PathBuf};

// B-35 Phase A: context artifacts pipeline
pub mod artifacts;
// B-42 Phase 1: cross-project cosmos artifact tier (~/.elai/cosmos/)
pub mod cosmos_worker;
// C-8 Task #10: pipeline integration for the cosmos artifact tier.
pub mod cosmos_stage;
// B-35 Phase A: knowledge-crate runtime config (ArtifactsConfig, KnowledgeRuntimeConfig)
pub mod runtime_config;

pub mod arag;
pub mod ast_adapter;
pub mod bm25;
pub mod catrag;
pub mod chunk_strategy;
pub mod chunking;
pub mod conventions;
// E-17 SP2 task 4: DEEP candidate pool cap + RRF quality gate.
pub mod deep;
pub mod degradation;
pub mod dkg;
pub mod dkg_adapter;
pub mod error;
pub mod fix_templates;
pub mod freshness;
pub mod hyde;
pub mod knowledge_ssb_sink;
pub mod incremental_profile;
pub mod index_manifest;
pub mod index_version;
pub mod metadata;
pub mod pagerank;
pub mod pipeline;
pub mod project_profile;
// E-17 SP2 task 5: heuristic query-class classifier (Layer A).
pub mod query_class;
pub mod query_expansion;
pub mod query_trace;
pub mod query_validator;
pub mod reranker;
pub mod shortcoder;
#[cfg(feature = "splade")]
pub mod splade;
pub mod startup;
pub mod symbol_graph;
// B-33 — visualization module (VizData + Mermaid generator).
// Pure std + serde + crate-local types; no new Cargo deps (knowledge/Cargo.toml
// is owned by s-22200). Do NOT re-export from this module's contents at the
// crate root (collision avoidance with `crate::SymbolKind`).
pub mod visualization;
pub mod telescope;
pub mod tier3;
pub mod benchmark;
pub mod vector;
pub mod web_arag;

// ---------------------------------------------------------------------------
// Core enums
// ---------------------------------------------------------------------------

/// Quality-tier routing for the retrieval pipeline (Phase 3K.4).
///
/// Distinct from `SearchMode` (Phase 1I degradation cascade): `SearchMode`
/// handles graceful degradation when components fail. `QueryTier` is
/// intentional quality-tier selection on a healthy pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum QueryTier {
    /// BM25 only, no vector search, no reranker, no HyDE, no expansion.
    /// Target latency: <20ms. For agent context passing, simple lookups.
    Fast,
    /// BM25 + vector hybrid (RRF), cross-encoder reranker. No HyDE, no expansion.
    /// Target latency: <200ms. Default for code search queries.
    Standard,
    /// Full pipeline: query expansion + HyDE + hybrid search + reranker.
    /// Target latency: <5s. For research, debugging, architecture questions.
    Deep,
    /// Auto-detect tier from query characteristics (default, backwards compatible).
    Auto,
}

impl QueryTier {
    /// Auto-detect tier from query text.
    ///
    /// Rules (from spec 3K.4):
    ///   <5 words -> Fast
    ///   5-15 words, code-related keywords -> Standard
    ///   >15 words, question form, debugging/architecture keywords -> Deep
    pub fn detect(query: &str) -> Self {
        let word_count = query.split_whitespace().count();

        if word_count < TIER_FAST_MAX_WORDS {
            return Self::Fast;
        }

        if word_count > TIER_DEEP_MIN_WORDS {
            return Self::Deep;
        }

        // 5-15 words: check for deep-tier keywords (question form, debugging, architecture)
        let lower = query.to_lowercase();
        for kw in DEEP_KEYWORDS {
            if lower.contains(kw) {
                return Self::Deep;
            }
        }

        Self::Standard
    }

    /// Resolve Auto to a concrete tier, or pass through if already concrete.
    pub fn resolve(self, query: &str) -> Self {
        match self {
            Self::Auto => Self::detect(query),
            concrete => concrete,
        }
    }
}

impl std::fmt::Display for QueryTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fast => write!(f, "FAST"),
            Self::Standard => write!(f, "STANDARD"),
            Self::Deep => write!(f, "DEEP"),
            Self::Auto => write!(f, "AUTO"),
        }
    }
}

/// Queries with fewer words than this are classified as FAST.
const TIER_FAST_MAX_WORDS: usize = 5;

/// Queries with more words than this are classified as DEEP.
const TIER_DEEP_MIN_WORDS: usize = 15;

/// Keywords that push a query from STANDARD to DEEP tier.
const DEEP_KEYWORDS: &[&str] = &[
    "how does", "why does", "debug", "architecture", "design",
    "explain", "investigate", "trace", "root cause", "anti-spiral",
    "what happens when", "step by step",
];

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
// Data flow types (A-48: DKG data flow edges)
// ---------------------------------------------------------------------------

/// Heuristic data flow edge type extracted via tree-sitter.
/// Cannot track closures, dynamic dispatch, complex control flow, or macros.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FlowType {
    Parameter,
    Return,
    Assignment,
    FieldAccess,
}

impl FlowType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Parameter => "PARAMETER",
            Self::Return => "RETURN",
            Self::Assignment => "ASSIGNMENT",
            Self::FieldAccess => "FIELD_ACCESS",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "PARAMETER" => Some(Self::Parameter),
            "RETURN" => Some(Self::Return),
            "ASSIGNMENT" => Some(Self::Assignment),
            "FIELD_ACCESS" => Some(Self::FieldAccess),
            _ => None,
        }
    }
}

/// A single data flow edge between two DKG nodes.
#[derive(Debug, Clone)]
pub struct DataFlowEdge {
    pub source_node_id: i64,
    pub target_node_id: i64,
    pub variable_name: String,
    pub flow_type: FlowType,
}

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct DependencyGraph {
    pub node_count: usize,
    pub edge_count: usize,
    pub files_parsed: usize,
    pub data_flow_count: usize,
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
/// and BM25 sub-engines.
///
/// Scheduled facade for Phase 1I. In MVP, sub-engines (`DkgEngine`,
/// `ShortCoder`, `populate_tier3`, `TelescopeContext`) are wired directly by
/// the CLI and task_runner rather than via this trait — so the trait itself
/// has zero implementors today. It is referenced as an architectural template
/// by `orchestrator::evidence::EvidenceGate` and `orchestrator::gate::GrcEngine`
/// sibling traits ("consistent with KnowledgeEngine/GrcEngine trait pattern").
/// Keep the trait until Phase 1I lands a concrete aggregator.
pub trait KnowledgeEngine {
    fn build_graph(&mut self, project_root: &Path) -> Result<DependencyGraph, KnowledgeError>;
    fn query(&self, query: &str, strategy: QueryStrategy) -> Result<Vec<CodeChunk>, KnowledgeError>;
    fn rerank(&self, query: &str, candidates: Vec<CodeChunk>) -> Result<Vec<CodeChunk>, KnowledgeError>;
    fn update(&mut self, file_changed: &Path) -> Result<(), KnowledgeError>;
    fn readiness_check(&self) -> ReadinessStatus;
}

// Re-export DkgEngine and language registry types at crate root for convenience.
pub use dkg::DkgEngine;
pub use dkg::{LanguageRegistry, LanguageRegistryStatus, GrammarLoadError};
// B-30 Pass C: re-export ImportResolver trait so external tests can use it.
pub use dkg::resolution::ImportResolver;
pub use ast_adapter::TreeSitterAstEvaluator;
pub use dkg_adapter::DkgQuerierAdapter;
pub use error::KnowledgeError;
pub use project_profile::{detect_project_profile, populate_tier2_conventions, serialize_profile, deserialize_profile, validate_tier_b_with_consent};
pub use tier3::populate_tier3;
pub use fix_templates::{
    attach_fix_template_keys, convention_aware_key, few_shot_for_symbol, FewShotExample,
    FixTemplateRegistry, DEFAULT_FIX_TEMPLATES_DIR, MAX_FEW_SHOT_EXAMPLES,
};
pub use shortcoder::ShortCoder;
// B-42 dashboard surface — per-artifact status snapshot + standalone helper.
// C-2: re-export CosmosWorker so the orchestrator `trigger_cosmos_refresh`
// helper can drive a one-shot refresh per task entry (DRIFT-3).
pub use cosmos_worker::{
    cosmos_status_snapshot_standalone, default_cosmos_root, CosmosArtifact,
    CosmosArtifactStatusJson, CosmosClock, CosmosError, CosmosManifest, CosmosWorker,
    DEFAULT_TICK_INTERVAL_SECS, NoopResearchDispatcher, ResearchDispatcher, SystemCosmosClock,
    // C-16: real-data refresh outcome + sync SSB emitter trait. The
    // production `SsbEmitterAdapter` (orchestrator) implements `SsbEmitter`
    // and bridges to async `Ssb::write_payload`.
    RefreshOutcome, SsbEmitter,
};
// C-8 Task #10: cosmos artifact retrieval stage for QueryPipeline.
pub use cosmos_stage::CosmosStage;
pub use telescope::{assemble_telescope, TelescopeContext};
