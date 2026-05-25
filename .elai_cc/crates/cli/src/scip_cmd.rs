//! E-40 — `elai scip <query|defs|refs|callers|callees|pagerank|status|import>`
//! handler.
//!
//! Production caller for the orchestrator-rooted symbol-graph module
//! (`elai_orchestrator::symbol_graph`). Defends against the
//! E37_WIRE_DARK_GAP_PATTERN by being the canonical entry point that
//! every new function in `symbol_graph::{cache,query,pagerank,scip_import,lsp_oracle}`
//! is reachable from.
//!
//! Read-only — no destructive sub-verbs. Loads
//! `.claude/tools/elai_graph.json` (path-resolved from the current cwd
//! at dispatch time) and dispatches to the requested helper.
//!
//! Paper grounding deferred per workflow/sessions/s-25120/mistakes.md
//! cluster #6 (Stack Graphs / GraphCoder / CodexGraph / RANGER / Glean
//! Angle). See workflow/reasoning/E-40-decisions.md.

use std::path::PathBuf;

use elai_orchestrator::symbol_graph::{
    self, oracle_lookup, oracle_lookup_with_mode, personalized_top_k, LspMode, OracleKind,
    SymbolGraphCache,
};
use serde::Serialize;

use crate::args::{CliOutputFormat, SymbolGraphAction};

const DEFAULT_GRAPH_PATH: &str = ".claude/tools/elai_graph.json";
const DEFAULT_SCIP_DIR: &str = ".claude/tools/scip_indices";

/// Top-level dispatcher invoked by main.rs.
pub(crate) fn run(
    action: SymbolGraphAction,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    match action {
        SymbolGraphAction::Query { symbol, kind } => run_query(&symbol, &kind, output_format),
        SymbolGraphAction::Defs { symbol } => run_query(&symbol, "defs", output_format),
        SymbolGraphAction::Refs { symbol } => run_query(&symbol, "refs", output_format),
        SymbolGraphAction::PageRank { seed, k } => run_pagerank(&seed, k, output_format),
        SymbolGraphAction::Status => run_status(output_format),
        SymbolGraphAction::ImportScip { dir } => run_import(&dir, output_format),
        SymbolGraphAction::LspProbe { symbol, language, mode } => {
            run_lsp_probe(&symbol, &language, &mode, output_format)
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct QueryReport {
    pub kind: String,
    pub symbol: String,
    pub result_count: usize,
    pub results: Vec<symbol_graph::SymbolMatch>,
    pub graph_path: String,
    pub node_count: usize,
}

fn run_query(symbol: &str, kind: &str, output_format: CliOutputFormat) -> Result<(), String> {
    let path = graph_path();
    let mut cache = SymbolGraphCache::load(&path).map_err(|e| format!("{e}"))?;
    let kind_norm = kind.to_ascii_lowercase();
    let results = match kind_norm.as_str() {
        "defs" => symbol_graph::find_defs(&mut cache, symbol).map_err(|e| format!("{e}"))?,
        "refs" => symbol_graph::find_refs(&mut cache, symbol).map_err(|e| format!("{e}"))?,
        "callers" => {
            symbol_graph::find_callers(&mut cache, symbol).map_err(|e| format!("{e}"))?
        }
        "callees" => {
            symbol_graph::find_callees(&mut cache, symbol).map_err(|e| format!("{e}"))?
        }
        other => {
            return Err(format!(
                "scip query: unknown kind '{other}' (expected defs|refs|callers|callees)"
            ));
        }
    };
    let report = QueryReport {
        kind: kind_norm,
        symbol: symbol.to_string(),
        result_count: results.len(),
        results,
        graph_path: path.to_string_lossy().to_string(),
        node_count: cache.node_count(),
    };
    emit(&report, output_format)
}

#[derive(Debug, Clone, Serialize)]
struct PageRankReport {
    pub seed: String,
    pub k: u32,
    pub ranked: Vec<symbol_graph::RankedSymbol>,
    pub graph_path: String,
    pub node_count: usize,
}

fn run_pagerank(seed: &str, k: u32, output_format: CliOutputFormat) -> Result<(), String> {
    if seed.is_empty() {
        return Err("scip pagerank: --seed must be non-empty".to_string());
    }
    let path = graph_path();
    let mut cache = SymbolGraphCache::load(&path).map_err(|e| format!("{e}"))?;
    let ranked = personalized_top_k(&mut cache, seed, k as usize);
    let report = PageRankReport {
        seed: seed.to_string(),
        k,
        ranked,
        graph_path: path.to_string_lossy().to_string(),
        node_count: cache.node_count(),
    };
    emit(&report, output_format)
}

#[derive(Debug, Clone, Serialize)]
struct StatusReport {
    pub graph_path: String,
    pub graph_present: bool,
    pub node_count: usize,
    pub lru_capacity: usize,
}

fn run_status(output_format: CliOutputFormat) -> Result<(), String> {
    let path = graph_path();
    let present = path.exists();
    let (node_count, lru_capacity) = if present {
        match SymbolGraphCache::load(&path) {
            Ok(cache) => (cache.node_count(), symbol_graph::cache::DEFAULT_LRU_CAPACITY),
            Err(e) => return Err(format!("scip status: load error: {e}")),
        }
    } else {
        (0, symbol_graph::cache::DEFAULT_LRU_CAPACITY)
    };
    let report = StatusReport {
        graph_path: path.to_string_lossy().to_string(),
        graph_present: present,
        node_count,
        lru_capacity,
    };
    emit(&report, output_format)
}

#[derive(Debug, Clone, Serialize)]
struct ImportReport {
    pub scip_dir: String,
    pub shard_count: usize,
}

fn run_import(dir: &str, output_format: CliOutputFormat) -> Result<(), String> {
    let dir_path = if dir.is_empty() {
        PathBuf::from(DEFAULT_SCIP_DIR)
    } else {
        PathBuf::from(dir)
    };
    let mut cache = SymbolGraphCache::empty_with_capacity(
        symbol_graph::cache::DEFAULT_LRU_CAPACITY,
    )
    .map_err(|e| format!("{e}"))?;
    let count = symbol_graph::import_scip_indices(&dir_path, &mut cache)
        .map_err(|e| format!("{e}"))?;
    let report = ImportReport {
        scip_dir: dir_path.to_string_lossy().to_string(),
        shard_count: count,
    };
    emit(&report, output_format)
}

/// Best-effort oracle probe exposed to the dispatcher. Wrapper around
/// the default-mode `oracle_lookup` for back-compat with the E-40
/// WIRE-DARK guard; the new `lsp-probe` sub-verb (E-52 Tier-B) drives
/// the explicit-mode variant. Returns the synthetic answer when the
/// toolchain is absent (the documented RUNPOD-PENDING path).
#[allow(dead_code)]
pub(crate) async fn oracle_probe(
    symbol: &str,
    language: &str,
) -> Result<symbol_graph::OracleAnswer, String> {
    oracle_lookup(symbol, language)
        .await
        .map_err(|e| format!("{e}"))
}

#[derive(Debug, Clone, Serialize)]
struct LspProbeReport {
    pub symbol: String,
    pub language: String,
    pub mode: String,
    pub binary: String,
    pub invoked: bool,
}

/// E-52 Tier-B — production caller for
/// `symbol_graph::lsp_oracle::oracle_lookup_with_mode`. Bridges the
/// async oracle into the synchronous CLI dispatcher via a per-call
/// `tokio::runtime::Runtime` (matches the safety_cmd async-bridging
/// pattern; no new top-level runtime required because main.rs is
/// already synchronous on this path).
fn run_lsp_probe(
    symbol: &str,
    language: &str,
    mode: &str,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    if symbol.is_empty() {
        return Err("scip lsp-probe: --symbol must be non-empty".to_string());
    }
    let lsp_mode = match mode.to_ascii_lowercase().as_str() {
        "probe" | "" => LspMode::Probe,
        "full_handshake" | "handshake" | "full" => LspMode::FullHandshake,
        other => {
            return Err(format!(
                "scip lsp-probe: unknown --mode '{other}' (expected probe|full_handshake)"
            ));
        }
    };
    // Validate language slug up-front so the synchronous error path
    // surfaces cleanly without spinning up a runtime.
    if OracleKind::from_slug(language).is_none() {
        return Err(format!(
            "scip lsp-probe: unsupported language '{language}' (rust|python|typescript|go)"
        ));
    }
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("scip lsp-probe: runtime build failed: {e}"))?;
    let answer = runtime
        .block_on(oracle_lookup_with_mode(symbol, language, lsp_mode))
        .map_err(|e| format!("scip lsp-probe: {e}"))?;
    let report = LspProbeReport {
        symbol: answer.symbol,
        language: language.to_string(),
        mode: lsp_mode.as_str().to_string(),
        binary: answer.binary,
        invoked: answer.invoked,
    };
    emit(&report, output_format)
}

/// Use [`OracleKind::canonical_binary`] downstream so the dispatcher can
/// resolve language slugs without re-exporting the type.
#[allow(dead_code)]
pub(crate) fn canonical_oracle_binary(slug: &str) -> Option<&'static str> {
    OracleKind::from_slug(slug).map(OracleKind::canonical_binary)
}

fn graph_path() -> PathBuf {
    PathBuf::from(DEFAULT_GRAPH_PATH)
}

fn emit<T: Serialize>(report: &T, output_format: CliOutputFormat) -> Result<(), String> {
    match output_format {
        CliOutputFormat::Json => {
            let s = serde_json::to_string_pretty(report)
                .map_err(|e| format!("scip: json serialize error: {e}"))?;
            println!("{s}");
            Ok(())
        }
        CliOutputFormat::Toml => {
            // toml::to_string surfaces an Err for shapes it cannot encode;
            // surface it as a String for the dispatcher.
            let s = toml::to_string(report)
                .map_err(|e| format!("scip: toml serialize error: {e}"))?;
            println!("{s}");
            Ok(())
        }
        CliOutputFormat::Text => {
            // Default: pretty-printed JSON. Mirrors the safety_cmd Text
            // arm so operators get a stable rendering.
            let s = serde_json::to_string_pretty(report)
                .map_err(|e| format!("scip: text-default serialize error: {e}"))?;
            println!("{s}");
            Ok(())
        }
    }
}
