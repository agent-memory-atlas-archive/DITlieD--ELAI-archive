// B-29 Phase 1 — symbol graph module.
//
// Sub-modules:
//   types    — SymbolNode, SymbolEdge, Confidence, SymbolRef, EntryPoint,
//               SymbolGraphMeta, SymbolGraphFilePayload, SymbolKind (graph variant)
//   sharding — name_shard_key, reverse_shard_bucket, all_name_shard_keys
//               + constants (SYMBOL_NAME_SHARDS, SYMBOL_REVERSE_SHARDS, …)
//
// IMPORTANT: do NOT re-export `types::SymbolKind` or `types::EntryPoint` from
// this module root or from the knowledge crate root. Both names already exist
// in the crate (SymbolKind at lib.rs:137, EntryPoint via elai_core re-export).
// Callers must qualify as `knowledge::symbol_graph::types::SymbolKind` etc.

pub mod build_pipeline;
pub mod cache;
pub mod extraction;
pub mod resolution;
pub mod sharding;
pub mod store;
pub mod traversal;
pub mod types;

// Re-export the shard constants at this module level for ergonomic access.
pub use sharding::{
    MAX_FLOW_DEPTH, MAX_IMPACT_DEPTH, SYMBOL_FILE_LRU_SIZE, SYMBOL_NAME_SHARDS,
    SYMBOL_REVERSE_SHARDS,
};

// B-34 — Entry-point detection sub-module.
// NOTE: types are NOT re-exported here (name-collision risk per line 9 above).
// Callers must qualify: `knowledge::symbol_graph::entry_points::DetectedEntryPoint`.
pub mod entry_points;
