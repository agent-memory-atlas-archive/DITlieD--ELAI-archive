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
pub mod legacy_b2;
pub mod lsp_oracle;
pub mod pagerank;
pub mod query;
pub mod scip_import;

pub use cache::{SymbolGraphCache, SymbolGraphCacheError, SymbolNode};
// Phase B2 legacy surface — preserved here so existing
// `use elai_orchestrator::{SymbolGraph, SymbolGraphError, SymbolRecord, SymbolTag}`
// crate-root re-exports continue to compile. The B2 surface reads
// `.claude/tools/elai.map` (six-column TSV); the new E-40 surface reads
// `.claude/tools/elai_graph.json` (SCIP-enriched). The two surfaces are
// orthogonal — both read different offline-generated artifacts produced
// by the same `codebase_mapper.py` pipeline. See
// workflow/reasoning/E-40-decisions.md DECISION-1 + DECISION-5.
pub use legacy_b2::{SymbolGraph, SymbolGraphError, SymbolRecord, SymbolTag};
pub use lsp_oracle::{oracle_lookup, LspOracleError, OracleAnswer, OracleKind};
pub use pagerank::{personalized_top_k, RankedSymbol};
pub use query::{find_callees, find_callers, find_defs, find_refs, QueryError, SymbolMatch};
pub use scip_import::{import_scip_indices, ScipImportError};
