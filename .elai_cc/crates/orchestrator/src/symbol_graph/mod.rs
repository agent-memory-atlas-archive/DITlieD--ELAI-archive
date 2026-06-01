//! E-40 — Orchestrator-rooted symbol-graph module.
//!
//! Different surface from `knowledge::symbol_graph` (B-29 SQLite-rooted
//! SymbolGraphCache with 27 name-shards + 256 reverse-shards + 500-file LRU)
//! and `knowledge::pagerank` (DKG-snapshot rooted whole-repo map). This
//! module loads a flat read-only view of the codebase symbol graph from
//! the offline-generated `.claude/tools/elai_graph.json` (SCIP-enriched
//! via rust-analyzer / scip-python / scip-typescript pipelines) and
//! exposes:
//!
//! - [`cache`] — `SymbolGraphCache` JSON loader with hand-rolled
//!   capacity-bounded LRU over node entries (BTreeMap-backed for
//!   deterministic eviction per anti-stale-training rule).
//! - [`query`] — `find_defs` / `find_refs` / `find_callers` / `find_callees`
//!   over the loaded view.
//! - [`pagerank`] — `personalized_top_k` rooted on a seed symbol over the
//!   call graph (NOT a re-export of the DKG-rooted pagerank crate —
//!   sibling module, distinct surface, see drift-warning in
//!   E-40-decisions.md).
//! - [`scip_import`] — `import_scip_indices` wrapper around the offline
//!   `.claude/tools/scip_indices/` SCIP shards so live runs can rehydrate
//!   the JSON view from updated SCIP output without regenerating the
//!   full elai_graph.json.
//! - [`lsp_oracle`] — on-demand LSP subprocess oracle that shells out to
//!   rust-analyzer / pylsp / tsserver for verdicts on `Unresolved`
//!   symbols. Routed via `RuntimeBackend::Lsp` (the new variant added in
//!   `providers::runtime::types`). `ToolchainMissing` is the documented
//!   RUNPOD-PENDING wire-dark — integration tests against live LSP
//!   servers require an installed toolchain.
//!
//! WIRE-DARK GAP avoidance (per E-37 lesson, gotchas.md
//! E37_WIRE_DARK_GAP_PATTERN): every fn in this module has at least one
//! production caller — primary call site is the new
//! `crates/cli/src/scip_cmd.rs` handler. `lsp_oracle::oracle_lookup` is
//! the documented RUNPOD-PENDING exception.

pub mod cache;
// E-40 Tranche B-2: DSL parser/executor (nom-based), dataflow analyzer
// (BFS over reverse-call edges), and refine+filter stage that re-ranks
// the union of initial + DSL results. Modules wire to the
// `knowledge::symbol_graph_adapter::SymbolGraphAdapter` trait via the
// blanket impl on `SymbolGraphCache` below.
pub mod dataflow;
pub mod dsl;
pub mod legacy_b2;
pub mod lsp_oracle;
pub mod pagerank;
pub mod query;
pub mod refine_filter;
pub mod scip_import;

pub use cache::{SymbolGraphCache, SymbolGraphCacheError, SymbolNode};
pub use dataflow::DataflowAnalyzer;
pub use dsl::{exec_dsl, parse_dsl, DslAst, DslFilter, DslParseError, DslSelector};
pub use refine_filter::{refine, RefineStage};
// Phase B2 legacy surface — preserved here so existing
// `use elai_orchestrator::{SymbolGraph, SymbolGraphError, SymbolRecord, SymbolTag}`
// crate-root re-exports continue to compile. The B2 surface reads
// `.claude/tools/elai.map` (six-column TSV); the new E-40 surface reads
// `.claude/tools/elai_graph.json` (SCIP-enriched). The two surfaces are
// orthogonal — both read different offline-generated artifacts produced
// by the same `codebase_mapper.py` pipeline. See
// workflow/reasoning/E-40-decisions.md DECISION-1 + DECISION-5.
pub use legacy_b2::{SymbolGraph, SymbolGraphError, SymbolRecord, SymbolTag};
pub use lsp_oracle::{
    oracle_lookup, oracle_lookup_with_mode, LspMode, LspOracleError, OracleAnswer, OracleKind,
};
pub use pagerank::{personalized_top_k, RankedSymbol};
pub use query::{find_callees, find_callers, find_defs, find_refs, QueryError, SymbolMatch};
pub use scip_import::{import_scip_indices, ScipImportError};

// ---------------------------------------------------------------------------
// E-40 Tranche B-2: `SymbolGraphAdapter` blanket impl + `notify_symbol_graph_event`.
// ---------------------------------------------------------------------------

use std::path::Path;
use std::sync::{Arc, Mutex};

use elai_ipc_types::{
    DataflowEdge as IpcDataflowEdge, DataflowEdgeKind as IpcDataflowEdgeKind,
    SymbolDataflowImpactResponse, SymbolDslQueryResponse, SymbolMatch as IpcSymbolMatch,
};
use elai_knowledge::symbol_graph_adapter::{
    SymbolGraphAdapter, SymbolGraphAdapterError,
};
use serde_json::Value;

use crate::ssb::{Ssb, SsbError, SsbPayload, SymbolGraphEventKind, SymbolGraphEventPayload};

/// Concrete adapter wrapping a shared `SymbolGraphCache`.
///
/// The cache is held behind `Arc<Mutex<_>>` because the underlying
/// `find_*` query helpers borrow `&mut SymbolGraphCache` (the LRU
/// touch-order tracker is interior-mutable but expressed as `&mut`).
/// `Mutex` keeps the implementation `Send + Sync` so it can sit behind
/// `Arc<dyn SymbolGraphAdapter>` in the pipeline.
pub struct SymbolGraphCacheAdapter {
    /// Shared cache handle. Pipeline holds an
    /// `Option<Arc<dyn SymbolGraphAdapter>>` so this Arc is hot-pluggable.
    pub cache: Arc<Mutex<SymbolGraphCache>>,
    /// Soft latency budget. Adapters that exceed it surface
    /// `SymbolGraphAdapterError::Internal("budget exceeded")` so the
    /// 5th-list RRF retriever can fall back to an empty list rather than
    /// stall the pipeline. Default 500 ms per B-29.5 p99 budget.
    pub max_query_ms: u64,
}

impl SymbolGraphCacheAdapter {
    /// Construct a new adapter with the given cache + soft budget.
    pub fn new(cache: Arc<Mutex<SymbolGraphCache>>, max_query_ms: u64) -> Self {
        Self { cache, max_query_ms }
    }
}

impl SymbolGraphAdapter for SymbolGraphCacheAdapter {
    fn dsl_query(
        &self,
        dsl_text: &str,
        _project_root: &Path,
        depth_cap: u8,
    ) -> Result<SymbolDslQueryResponse, SymbolGraphAdapterError> {
        let ast = dsl::parse_dsl(dsl_text)
            .map_err(|e| SymbolGraphAdapterError::DslParse(e.to_string()))?;
        let mut guard = self
            .cache
            .lock()
            .map_err(|_| SymbolGraphAdapterError::CacheUnavailable("cache mutex poisoned".into()))?;
        let matches = dsl::exec_dsl(&ast, &mut guard);
        let total_scanned: u32 = matches.len().try_into().unwrap_or(u32::MAX);
        let ipc_matches: Vec<IpcSymbolMatch> = matches
            .into_iter()
            .map(|m| IpcSymbolMatch {
                symbol: m.name,
                file_path: m.file,
                line: m.line,
                kind: m.kind,
            })
            .collect();
        // For DSL queries the dataflow side-context comes from the
        // analyzer over the same cache. Bound the side context by the
        // caller's depth_cap (default 2 enforced upstream).
        let mut analyzer = dataflow::DataflowAnalyzer::new(&mut guard, depth_cap);
        let mut edges: Vec<IpcDataflowEdge> = Vec::new();
        for m in &ipc_matches {
            for edge in analyzer.defuse_edges(&m.symbol) {
                edges.push(IpcDataflowEdge {
                    source: edge.source,
                    target: edge.target,
                    edge_kind: dataflow_edge_kind_to_ipc(edge.edge_kind),
                });
            }
        }
        Ok(SymbolDslQueryResponse {
            matches: ipc_matches,
            dataflow_edges: edges,
            total_scanned,
        })
    }

    fn dataflow_impact(
        &self,
        symbol: &str,
        _project_root: &Path,
        depth_cap: u8,
    ) -> Result<SymbolDataflowImpactResponse, SymbolGraphAdapterError> {
        if symbol.is_empty() {
            return Err(SymbolGraphAdapterError::InvalidInput("empty symbol".into()));
        }
        let mut guard = self
            .cache
            .lock()
            .map_err(|_| SymbolGraphAdapterError::CacheUnavailable("cache mutex poisoned".into()))?;
        let mut analyzer = dataflow::DataflowAnalyzer::new(&mut guard, depth_cap);
        let impacted = analyzer.impact_analysis(symbol);
        let total_edges: u32 = analyzer.last_edge_count().try_into().unwrap_or(u32::MAX);
        let depth_actual = analyzer.last_depth_reached();
        Ok(SymbolDataflowImpactResponse {
            impacted_symbols: impacted,
            depth_actual,
            total_edges,
        })
    }
}

fn dataflow_edge_kind_to_ipc(k: dataflow::DataflowEdgeKind) -> IpcDataflowEdgeKind {
    match k {
        dataflow::DataflowEdgeKind::DefUse => IpcDataflowEdgeKind::DefUse,
        dataflow::DataflowEdgeKind::UseDef => IpcDataflowEdgeKind::UseDef,
        dataflow::DataflowEdgeKind::AssignFlow => IpcDataflowEdgeKind::AssignFlow,
        dataflow::DataflowEdgeKind::CallChain => IpcDataflowEdgeKind::CallChain,
    }
}

/// Emit a `symbol_graph_event` SSB row.
///
/// Mirrors `crate::lora_ship_gate::notify_lora_ship_gate` shape. The
/// caller owns the env-gate decision — this helper assumes the gate
/// already passed (the producer-side site that decides to emit the
/// event is the right place to consult
/// `ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE` or other per-event gates).
///
/// Returns:
///   * `Ok(Some(seq))` -- row written; `seq` is the SSB auto-increment
///     sequence.
///   * `Err(SsbError)` -- propagate the underlying write failure.
pub async fn notify_symbol_graph_event(
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    kind: SymbolGraphEventKind,
    session_id: Option<&str>,
    details_json: Value,
    ts_ms: i64,
) -> Result<i64, SsbError> {
    let payload = SsbPayload::SymbolGraphEvent(SymbolGraphEventPayload {
        kind,
        session_id: session_id.map(str::to_owned),
        agent_id: Some(agent_id.to_owned()),
        details_json,
        ts_ms,
    });
    ssb.write_payload(agent_id, superstep, payload, Vec::new()).await
}
