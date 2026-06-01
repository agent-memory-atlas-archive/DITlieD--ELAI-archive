//! E-40 Tranche B-2 — `SymbolGraphAdapter` trait.
//!
//! Thin abstraction that lets the `elai_knowledge::pipeline` 5th-list
//! RRF integration call into the orchestrator-rooted JSON symbol-graph
//! cache without taking a direct dependency on `elai_orchestrator`
//! (which would create an illegal reverse-direction edge — orchestrator
//! depends on knowledge, not the other way round).
//!
//! Concrete impl lives in
//! `crates/orchestrator/src/symbol_graph/mod.rs` (blanket impl on
//! `SymbolGraphCache`). The pipeline takes an
//! `Option<Arc<dyn SymbolGraphAdapter>>` so existing call sites compile
//! unchanged; the 5th list is wire-dark by default
//! (`ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE`).
//!
//! See `workflow/reasoning/E-40-tranche-B2-decisions.md` GAPS-RESOLVED-2
//! for the crate-rooting rationale.

use std::path::Path;

use elai_ipc_types::{
    SymbolDataflowImpactResponse, SymbolDslQueryResponse,
};
use thiserror::Error;

/// Errors surfaced by [`SymbolGraphAdapter`] impls. The variants are
/// intentionally coarse — the adapter layer is best-effort (the 5th-list
/// retriever degrades to empty results on any error).
#[derive(Debug, Error)]
pub enum SymbolGraphAdapterError {
    /// The adapter's underlying cache could not be opened / read.
    #[error("symbol_graph_adapter: cache unavailable: {0}")]
    CacheUnavailable(String),
    /// The DSL string failed to parse.
    #[error("symbol_graph_adapter: dsl parse error: {0}")]
    DslParse(String),
    /// The requested symbol was empty or otherwise malformed.
    #[error("symbol_graph_adapter: invalid input: {0}")]
    InvalidInput(String),
    /// Internal adapter error (catch-all). String message only — the
    /// adapter is the boundary between two crates so we do not leak
    /// orchestrator-private error types here.
    #[error("symbol_graph_adapter: internal: {0}")]
    Internal(String),
}

/// Adapter trait implemented by the orchestrator-rooted
/// `SymbolGraphCache` and consumed by `elai_knowledge::pipeline` for
/// the 5th-list RRF integration. Send + Sync so the pipeline can hold
/// an `Arc<dyn SymbolGraphAdapter>` across `tokio::spawn` boundaries.
pub trait SymbolGraphAdapter: Send + Sync {
    /// Execute a Datalog-style DSL query and return matches + any
    /// dataflow side-context. Implementations are free to reject a DSL
    /// they do not understand; the knowledge pipeline treats any error
    /// as an empty result for the 5th RRF list (best-effort).
    fn dsl_query(
        &self,
        dsl: &str,
        project_root: &Path,
        depth_cap: u8,
    ) -> Result<SymbolDslQueryResponse, SymbolGraphAdapterError>;

    /// BFS impact analysis over the reverse-call index, bounded by
    /// `depth_cap`. Returns the set of transitively-impacted symbol
    /// names + the actual depth reached.
    fn dataflow_impact(
        &self,
        symbol: &str,
        project_root: &Path,
        depth_cap: u8,
    ) -> Result<SymbolDataflowImpactResponse, SymbolGraphAdapterError>;
}
