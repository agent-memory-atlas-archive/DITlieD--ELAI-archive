// B-33 — Interactive graph viewer types + Mermaid generator.
//
// Pure std + serde + crate-local types; ZERO new Rust dependencies
// (knowledge/Cargo.toml is locked by s-22200).
//
// Submodules:
//   types   — VizData, VizFile, VizFileEdge, VizSymbol, VizSymbolEdge,
//             VizProject, VizEdgeKind, SymbolMode + MAX_VIZ_* constants.
//   mermaid — generate_mermaid(VizData, max_nodes) -> String.
//
// Do NOT re-export `crate::symbol_graph::types::SymbolKind` or
// `crate::symbol_graph::types::EntryPoint` from this module: both names
// already exist at the crate root (B29_SAME_CRATE_SAME_NAME_TYPE_COLLISION
// gotcha). Callers must qualify when needed.

pub mod types;
pub mod mermaid;
