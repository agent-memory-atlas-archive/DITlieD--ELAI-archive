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
//
// H-10: adds `is_dead_code` / `coverage_count` fields to VizFile (additive,
// Option-wrapped so existing consumers are unaffected), and a Kosaraju-SCC
// pass that replaces the `cyclic: false` hardcode in VizFileEdge with the
// real cycle-membership value.

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

// ---------------------------------------------------------------------------
// H-10: Kosaraju two-pass SCC for VizFileEdge cycle detection
// ---------------------------------------------------------------------------
//
// Replaces the `cyclic: false` hardcode that was in build_file_view.
// Given a list of (source_file, target_file) pairs, returns a BTreeSet of
// (src, tgt) pairs that lie on a back-edge (i.e. participate in a cycle).
//
// Algorithm: Kosaraju's two-pass SCC
//   Pass 1: DFS on the forward graph, record finish-time order.
//   Pass 2: DFS on the reversed graph in reverse finish-time order.
//   Any edge (u, v) where u and v end up in the same SCC is a cyclic edge.
//
// Complexity: O(V + E) — safe for the bounded VizFileEdge set (MAX_VIZ_EDGES cap).

/// Return the set of (source, target) file pairs that are part of an SCC
/// (strongly connected component of size >= 2) in the import graph.
/// These pairs should have `VizFileEdge.cyclic = true`.
fn compute_cyclic_edges(file_pairs: &[(String, String)]) -> BTreeSet<(String, String)> {
    if file_pairs.is_empty() {
        return BTreeSet::new();
    }

    // Collect all file ids and build adjacency lists (forward + reverse).
    let mut file_ids: Vec<String> = Vec::new();
    let mut file_idx: BTreeMap<&str, usize> = BTreeMap::new();

    for (src, tgt) in file_pairs {
        if !file_idx.contains_key(src.as_str()) {
            file_idx.insert(src.as_str(), file_ids.len());
            file_ids.push(src.clone());
        }
        if !file_idx.contains_key(tgt.as_str()) {
            file_idx.insert(tgt.as_str(), file_ids.len());
            file_ids.push(tgt.clone());
        }
    }

    let n = file_ids.len();
    let mut fwd_adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut rev_adj: Vec<Vec<usize>> = vec![Vec::new(); n];

    for (src, tgt) in file_pairs {
        // SAFETY: both were inserted above; get is infallible here.
        let s = file_idx[src.as_str()];
        let t = file_idx[tgt.as_str()];
        if s != t {
            fwd_adj[s].push(t);
            rev_adj[t].push(s);
        }
    }

    // Pass 1: iterative DFS on forward graph, collect finish order.
    let mut visited = vec![false; n];
    let mut finish_order: Vec<usize> = Vec::with_capacity(n);

    for start in 0..n {
        if visited[start] {
            continue;
        }
        // Iterative DFS with (node, edge_cursor) stack to avoid recursion overflow.
        let mut stack: Vec<(usize, usize)> = vec![(start, 0)];
        visited[start] = true;
        while let Some((node, cursor)) = stack.last_mut() {
            let node = *node;
            let cursor_val = *cursor;
            if cursor_val < fwd_adj[node].len() {
                *cursor += 1;
                let next = fwd_adj[node][cursor_val];
                if !visited[next] {
                    visited[next] = true;
                    stack.push((next, 0));
                }
            } else {
                finish_order.push(node);
                stack.pop();
            }
        }
    }

    // Pass 2: DFS on reversed graph in reverse finish order; assign SCC ids.
    let mut scc_id: Vec<Option<usize>> = vec![None; n];
    let mut scc_counter = 0usize;

    for &start in finish_order.iter().rev() {
        if scc_id[start].is_some() {
            continue;
        }
        let current_scc = scc_counter;
        scc_counter += 1;
        let mut stack: Vec<usize> = vec![start];
        scc_id[start] = Some(current_scc);
        while let Some(node) = stack.pop() {
            for &nb in &rev_adj[node] {
                if scc_id[nb].is_none() {
                    scc_id[nb] = Some(current_scc);
                    stack.push(nb);
                }
            }
        }
    }

    // Build SCC size map: only SCCs with size >= 2 contain cycles.
    let mut scc_sizes: BTreeMap<usize, usize> = BTreeMap::new();
    for id_opt in &scc_id {
        if let Some(id) = id_opt {
            *scc_sizes.entry(*id).or_insert(0) += 1;
        }
    }

    // Collect all (src, tgt) pairs where both nodes are in the same SCC of size >= 2.
    let mut cyclic: BTreeSet<(String, String)> = BTreeSet::new();
    for (src, tgt) in file_pairs {
        let s = file_idx[src.as_str()];
        let t = file_idx[tgt.as_str()];
        if s == t {
            continue;
        }
        if let (Some(sid), Some(tid)) = (scc_id[s], scc_id[t]) {
            if sid == tid {
                let sz = scc_sizes.get(&sid).copied().unwrap_or(0);
                if sz >= 2 {
                    cyclic.insert((src.clone(), tgt.clone()));
                }
            }
        }
    }

    cyclic
}

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
    let mut seen_edges: BTreeSet<(String, String)> = BTreeSet::new();
    let mut raw_file_pairs: Vec<(String, String)> = Vec::new();

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
        raw_file_pairs.push((src_file, tgt_file));
    }

    // H-10: SCC cycle pass. Replaces cyclic:false hardcode (views.rs:103 legacy).
    // Kosaraju two-pass — O(V+E) on the bounded (MAX_VIZ_EDGES) edge set.
    let cyclic_pairs = compute_cyclic_edges(&raw_file_pairs);

    let file_edges: Vec<VizFileEdge> = raw_file_pairs
        .into_iter()
        .map(|(src, tgt)| {
            let cyclic = cyclic_pairs.contains(&(src.clone(), tgt.clone()));
            VizFileEdge {
                source: src,
                target: tgt,
                kind: VizEdgeKind::Import,
                cyclic,
            }
        })
        .collect();

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

    // H-10 WARN-2 saturation guard: if there are no dependency edges at all
    // (the DKG was built before generate_cross_edges ran, or only 'contains'
    // edges exist), then every file has dep_count==0 AND dependent_count==0,
    // which would flag the ENTIRE codebase as topology-suspected-dead — an
    // amber-ring storm that is misleading rather than informative.
    // Guard: when the total import edge count across all files is zero, suppress
    // is_dead_code for ALL files (emit None) and warn so the user knows dead-code
    // analysis requires a freshly indexed DKG with import edges.
    // When at least one import edge exists, the heuristic has real topology to
    // reason over and zero-degree nodes are genuinely suspect.
    let total_import_edges: usize = deps_count.values().sum();
    if total_import_edges == 0 && !by_file.is_empty() {
        tracing::warn!(
            file_count = by_file.len(),
            "H-10 dead-code: no import edges in DKG snapshot; \
             topology-suspected-dead annotation suppressed (run `elai index --full` to enable). \
             Canvas will show is_dead_code=None for all files."
        );
    }

    let files: Vec<VizFile> = by_file
        .iter()
        .map(|(path, (lang, sym_count))| {
            let basename = Path::new(path)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(path)
                .to_string();
            // H-10: dead-code annotation (HD-5).
            // A file is topology-suspected-dead when it has zero incoming
            // import edges AND zero dependents — BUT only when the graph has
            // at least one import edge to reason over (saturation guard above).
            // When total_import_edges == 0, is_dead_code is always None so the
            // canvas shows "dead-code analysis unavailable" rather than falsely
            // flagging every file. Coverage data is not available in this builder;
            // coverage_count is left None (topology-only suspected, never confirmed).
            let dep_count = deps_count.get(path).copied().unwrap_or(0);
            let dependent_count = dependents_count.get(path).copied().unwrap_or(0);
            let is_dead_code =
                if total_import_edges > 0 && dependent_count == 0 && dep_count == 0 && *sym_count > 0 {
                    Some(true)
                } else {
                    None
                };
            VizFile {
                id: path.clone(),
                label: basename,
                language: lang.clone(),
                deps: dep_count,
                dependents: dependent_count,
                symbol_count: *sym_count,
                // C-24 (DRIFT-7 Path A): level/status are decomposition-only
                // node attributes; the file view leaves them None so the
                // cytoscape decomposition selectors are inert here.
                level: None,
                status: None,
                // H-10: topology-suspected dead (no static callers after import pass).
                // coverage_count is None here — the canvas entrypoints_map and the
                // wiring-gate Layer-3 coverage runner supply it at render time.
                is_dead_code,
                coverage_count: None,
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

// ---------------------------------------------------------------------------
// H-10: test-helper re-export of the SCC function
// ---------------------------------------------------------------------------
//
// Exposed only under the `test-helpers` feature so integration tests in
// `crates/contracts/tests/h10_code_graph_contract.rs` can exercise the SCC
// logic directly without going through the full build_file_view path.

/// Test-only re-export of `compute_cyclic_edges`.
/// Returns the set of (source, target) file pairs that lie on a cycle.
#[cfg(feature = "test-helpers")]
pub fn scc_cyclic_edges_for_test(pairs: &[(String, String)]) -> BTreeSet<(String, String)> {
    compute_cyclic_edges(pairs)
}
