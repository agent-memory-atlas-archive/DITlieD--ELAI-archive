//! E-40 — Orchestrator-rooted symbol-graph dashboard surface
//! (read-only).
//!
//! cross-lang-pair: dashboard/tauri/api
//!
//! Three Tauri commands feed the new `SymbolGraphWindow.svelte`:
//!
//! - [`symbol_graph_query`] — dispatches `find_defs` / `find_refs` /
//!   `find_callers` / `find_callees` against the orchestrator-rooted
//!   `SymbolGraphCache` loaded from `.claude/tools/elai_graph.json`.
//! - [`symbol_graph_pagerank`] — runs personalized PageRank rooted on
//!   the supplied seed.
//! - [`symbol_graph_status`] — graph_path + node_count + LRU capacity
//!   for the status badge in the window header.
//!
//! Additive — runs alongside `commands_e37.rs` (safety panel) /
//! `commands_e35.rs` (grammar compilations) without touching them.
//!
//! See `workflow/reasoning/E-40-decisions.md`.

use std::path::PathBuf;

use elai_orchestrator::symbol_graph::{
    self, personalized_top_k, RankedSymbol, SymbolGraphCache, SymbolMatch,
};
use serde::{Deserialize, Serialize};

const DEFAULT_GRAPH_PATH: &str = ".claude/tools/elai_graph.json";

/// DTO returned by [`symbol_graph_query`]. Mirrors the CLI scip_cmd
/// QueryReport shape so dashboard + CLI render identically.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolGraphQueryReport {
    pub kind: String,
    pub symbol: String,
    pub result_count: u32,
    pub results: Vec<SymbolMatch>,
    pub graph_path: String,
    pub node_count: u32,
}

/// DTO returned by [`symbol_graph_pagerank`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolGraphPageRankReport {
    pub seed: String,
    pub k: u32,
    pub ranked: Vec<RankedSymbol>,
    pub graph_path: String,
    pub node_count: u32,
}

/// DTO returned by [`symbol_graph_status`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolGraphStatusReport {
    pub graph_path: String,
    pub graph_present: bool,
    pub node_count: u32,
    pub lru_capacity: u32,
}

/// Tauri command: dispatch a symbol-graph query.
///
/// `kind` accepts the four canonical labels (defs / refs / callers /
/// callees). Unknown kinds surface as a `String` Err so the frontend can
/// render an inline error toast.
#[tauri::command]
pub async fn symbol_graph_query(
    symbol: String,
    kind: String,
) -> Result<SymbolGraphQueryReport, String> {
    let path = PathBuf::from(DEFAULT_GRAPH_PATH);
    let mut cache = SymbolGraphCache::load(&path).map_err(|e| format!("{e}"))?;
    let kind_norm = kind.to_ascii_lowercase();
    let results: Vec<SymbolMatch> = match kind_norm.as_str() {
        "defs" => symbol_graph::find_defs(&mut cache, &symbol).map_err(|e| format!("{e}"))?,
        "refs" => symbol_graph::find_refs(&mut cache, &symbol).map_err(|e| format!("{e}"))?,
        "callers" => {
            symbol_graph::find_callers(&mut cache, &symbol).map_err(|e| format!("{e}"))?
        }
        "callees" => {
            symbol_graph::find_callees(&mut cache, &symbol).map_err(|e| format!("{e}"))?
        }
        other => {
            return Err(format!(
                "symbol_graph_query: unknown kind '{other}' (expected defs|refs|callers|callees)"
            ));
        }
    };
    let result_count = u32::try_from(results.len()).unwrap_or(u32::MAX);
    let node_count = u32::try_from(cache.node_count()).unwrap_or(u32::MAX);
    Ok(SymbolGraphQueryReport {
        kind: kind_norm,
        symbol,
        result_count,
        results,
        graph_path: path.to_string_lossy().to_string(),
        node_count,
    })
}

/// Tauri command: run personalized PageRank rooted on `seed` and return
/// the top-k ranked symbols.
#[tauri::command]
pub async fn symbol_graph_pagerank(
    seed: String,
    k: u32,
) -> Result<SymbolGraphPageRankReport, String> {
    if seed.is_empty() {
        return Err("symbol_graph_pagerank: seed must be non-empty".to_string());
    }
    let path = PathBuf::from(DEFAULT_GRAPH_PATH);
    let mut cache = SymbolGraphCache::load(&path).map_err(|e| format!("{e}"))?;
    let ranked = personalized_top_k(&mut cache, &seed, k as usize);
    let node_count = u32::try_from(cache.node_count()).unwrap_or(u32::MAX);
    Ok(SymbolGraphPageRankReport {
        seed,
        k,
        ranked,
        graph_path: path.to_string_lossy().to_string(),
        node_count,
    })
}

/// Tauri command: graph_path + node_count + LRU capacity snapshot for
/// the window header.
#[tauri::command]
pub async fn symbol_graph_status() -> Result<SymbolGraphStatusReport, String> {
    let path = PathBuf::from(DEFAULT_GRAPH_PATH);
    let graph_present = path.exists();
    let (node_count, lru_capacity) = if graph_present {
        match SymbolGraphCache::load(&path) {
            Ok(cache) => (
                u32::try_from(cache.node_count()).unwrap_or(u32::MAX),
                symbol_graph::cache::DEFAULT_LRU_CAPACITY as u32,
            ),
            Err(e) => return Err(format!("symbol_graph_status: load error: {e}")),
        }
    } else {
        (0u32, symbol_graph::cache::DEFAULT_LRU_CAPACITY as u32)
    };
    Ok(SymbolGraphStatusReport {
        graph_path: path.to_string_lossy().to_string(),
        graph_present,
        node_count,
        lru_capacity,
    })
}
