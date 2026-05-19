// C-24 — Shared VizData builders for `file` and `symbol` modes.
//
// Extracted from `crates/dashboard-v2/src-tauri/src/commands.rs`
// (b33_build_file_view + b33_build_symbol_view) so the dashboard Tauri
// commands AND the orchestrator translate_tool_request arm (C-24 DRIFT-5
// DECISION-4) can build the same VizData from a project_root + project_id
// pair without crate-cycle dependencies.
//
// The `decomposition` mode is NOT in this module — its VizData payload is
// built from in-memory `DecompositionTreeSnapshot` state held by the
// dashboard `AppState`, which the orchestrator cannot reach without a
// circular dep. Callers that need decomposition mode must keep using the
// dashboard-local builder (b33_build_decomposition_view in commands.rs).

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::symbol_graph::{
    cache::SymbolGraphCache,
    sharding::SYMBOL_FILE_LRU_SIZE,
};
use crate::visualization::types::{
    SymbolMode, VizData, VizEdgeKind, VizFile, VizFileEdge, VizProject, VizSymbol,
    VizSymbolEdge, MAX_VIZ_EDGES, MAX_VIZ_SYMBOLS, MAX_VIZ_SYMS_PER_FILE,
};
use crate::DkgEngine;

/// Internal helper: produce an empty VizData payload with a reason-bearing
/// `SymbolMode::Omitted`. Centralised so callers cannot construct a
/// non-Full SymbolMode without a populated reason.
pub fn empty_viz(project_id: &str, reason: &str) -> VizData {
    VizData::empty(
        project_id.to_string(),
        SymbolMode::Omitted {
            reason: reason.to_string(),
        },
    )
}

/// Build the `file` mode VizData by walking DkgEngine nodes + edges.
/// Per-file `symbol_count` is filled from SymbolGraphCache.meta when
/// available. Returns `Ok(empty_viz(...))` (not Err) when no index has
/// been built yet — callers render a placeholder.
pub fn build_file_view(project_root: &Path, project_id: &str) -> Result<VizData, String> {
    let dkg_path = elai_core::paths::dkg_db_path(project_root);
    if !dkg_path.exists() {
        return Ok(empty_viz(project_id, "no index yet — run /index"));
    }

    let engine = DkgEngine::new(&dkg_path, None, 0)
        .map_err(|e| format!("DkgEngine open failed: {e}"))?;
    let nodes = engine
        .nodes()
        .map_err(|e| format!("DkgEngine::nodes failed: {e}"))?;
    let edges_raw = engine
        .edges()
        .map_err(|e| format!("DkgEngine::edges failed: {e}"))?;

    // Aggregate file -> { language, symbol_count, in/out counts } via BTreeMap
    // (deterministic iteration; matches the legacy dashboard helper shape).
    let mut by_file: BTreeMap<String, (String, usize)> = BTreeMap::new();
    let mut node_to_file: BTreeMap<i64, String> = BTreeMap::new();

    for n in &nodes {
        let path_str = n.file_path.to_string_lossy().to_string();
        node_to_file.insert(n.node_id, path_str.clone());
        let lang = n
            .file_path
            .extension()
            .and_then(|e| e.to_str())
            .and_then(crate::dkg::ext_to_lang_id)
            .unwrap_or("other")
            .to_string();
        let entry = by_file.entry(path_str).or_insert((lang, 0));
        entry.1 += 1;
    }

    let mut deps_count: BTreeMap<String, usize> = BTreeMap::new();
    let mut dependents_count: BTreeMap<String, usize> = BTreeMap::new();
    let mut file_edges: Vec<VizFileEdge> = Vec::new();
    let mut seen_edges: BTreeSet<(String, String)> = BTreeSet::new();
    for (src_id, tgt_id, kind) in &edges_raw {
        if kind != "imports" {
            continue;
        }
        let (src_file, tgt_file) = match (node_to_file.get(src_id), node_to_file.get(tgt_id)) {
            (Some(s), Some(t)) => (s.clone(), t.clone()),
            _ => continue,
        };
        if src_file == tgt_file {
            continue;
        }
        if !seen_edges.insert((src_file.clone(), tgt_file.clone())) {
            continue;
        }
        *deps_count.entry(src_file.clone()).or_insert(0) += 1;
        *dependents_count.entry(tgt_file.clone()).or_insert(0) += 1;
        file_edges.push(VizFileEdge {
            source: src_file,
            target: tgt_file,
            kind: VizEdgeKind::Import,
            cyclic: false,
        });
    }

    let mut symbol_mode = SymbolMode::Full;
    let mut total_symbols: Option<usize> = None;
    let mut total_call_edges: Option<usize> = None;
    let mut built_at: Option<i64> = None;
    if let Ok(cache) = SymbolGraphCache::new(&dkg_path, project_id, SYMBOL_FILE_LRU_SIZE) {
        let meta = cache.meta();
        let symbol_count = usize::try_from(meta.symbol_count).unwrap_or(usize::MAX);
        if symbol_count > MAX_VIZ_SYMBOLS {
            symbol_mode = SymbolMode::Omitted {
                reason: format!(
                    "symbol count {symbol_count} exceeds {MAX_VIZ_SYMBOLS} cap; symbol mode disabled"
                ),
            };
        }
        total_symbols = Some(symbol_count);
        total_call_edges = Some(usize::try_from(meta.edge_count).unwrap_or(usize::MAX));
        built_at = Some(meta.built_at);
    } else {
        symbol_mode = SymbolMode::Omitted {
            reason: "symbol graph not built yet — symbol mode disabled".to_string(),
        };
    }

    let files: Vec<VizFile> = by_file
        .iter()
        .map(|(path, (lang, sym_count))| {
            let basename = Path::new(path)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(path)
                .to_string();
            VizFile {
                id: path.clone(),
                label: basename,
                language: lang.clone(),
                deps: deps_count.get(path).copied().unwrap_or(0),
                dependents: dependents_count.get(path).copied().unwrap_or(0),
                symbol_count: *sym_count,
                // C-24 (DRIFT-7 Path A): level/status are decomposition-only
                // node attributes; the file view leaves them None so the
                // cytoscape decomposition selectors are inert here.
                level: None,
                status: None,
            }
        })
        .collect();

    Ok(VizData {
        project: VizProject {
            name: project_id.to_string(),
            files: files.len(),
            edges: file_edges.len(),
            symbols: total_symbols,
            call_edges: total_call_edges,
            built_at,
        },
        files,
        file_edges,
        symbols: Vec::new(),
        symbol_edges: Vec::new(),
        symbols_by_file: BTreeMap::new(),
        symbol_mode,
    })
}

/// Build the `symbol` mode VizData by walking SymbolGraphCache file payloads.
/// Falls back to file mode + `SymbolMode::Capped` reason when total edge
/// count reaches the `MAX_VIZ_EDGES` cap. Returns `Ok(empty_viz)` when no
/// index exists yet.
pub fn build_symbol_view(project_root: &Path, project_id: &str) -> Result<VizData, String> {
    let mut data = build_file_view(project_root, project_id)?;
    // If file mode already returned an Omitted reason (no index), short-circuit.
    if matches!(data.symbol_mode, SymbolMode::Omitted { .. }) {
        return Ok(data);
    }

    let dkg_path = elai_core::paths::dkg_db_path(project_root);
    let mut cache = match SymbolGraphCache::new(&dkg_path, project_id, SYMBOL_FILE_LRU_SIZE) {
        Ok(c) => c,
        Err(_) => {
            data.symbol_mode = SymbolMode::Omitted {
                reason: "symbol graph not built yet — symbol mode disabled".to_string(),
            };
            return Ok(data);
        }
    };

    let file_ids: Vec<String> = data.files.iter().map(|f| f.id.clone()).collect();
    let mut total_edges = 0usize;
    let mut symbols_by_file: BTreeMap<String, Vec<VizSymbol>> = BTreeMap::new();
    let mut all_symbols: Vec<VizSymbol> = Vec::new();
    let mut symbol_edges: Vec<VizSymbolEdge> = Vec::new();

    for fid in &file_ids {
        let payload = match cache.get_or_load_payload(fid) {
            Ok(Some(p)) => p,
            Ok(None) | Err(_) => continue,
        };
        let mut file_symbols: Vec<VizSymbol> = Vec::new();
        for s in &payload.symbols {
            if file_symbols.len() >= MAX_VIZ_SYMS_PER_FILE {
                break;
            }
            let viz = VizSymbol {
                id: s.id.clone(),
                file: s.file.clone(),
                name: s.name.clone(),
                qualified_name: s.qualified_name.clone(),
                kind: format!("{:?}", s.kind),
                line: s.line,
            };
            file_symbols.push(viz.clone());
            all_symbols.push(viz);
        }
        for e in &payload.outgoing_calls {
            if total_edges >= MAX_VIZ_EDGES {
                break;
            }
            for callee in &e.callee_candidates {
                symbol_edges.push(VizSymbolEdge {
                    source: e.caller_id.clone(),
                    target: callee.clone(),
                    confidence: e.confidence,
                });
                total_edges += 1;
                if total_edges >= MAX_VIZ_EDGES {
                    break;
                }
            }
        }
        if !file_symbols.is_empty() {
            symbols_by_file.insert(fid.clone(), file_symbols);
        }
    }

    if total_edges >= MAX_VIZ_EDGES {
        data.symbol_mode = SymbolMode::Capped {
            reason: format!("edge count reached {MAX_VIZ_EDGES} cap; symbol mode capped"),
        };
    }
    data.symbols = all_symbols;
    data.symbol_edges = symbol_edges;
    data.symbols_by_file = symbols_by_file;
    Ok(data)
}
