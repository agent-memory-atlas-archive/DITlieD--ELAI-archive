// B-29 Phase 2 — cross-file call-site resolution.
//
// Implements the four-tier confidence strategy mirroring SocratiCode's
// graph-symbol-resolution.ts:26-94. Mutates outgoing_calls_by_file in place.
//
// Tier 1 — Local (same file): short-circuits immediately.
// Tier 2 — Imported (1-hop): walks deps_by_file[caller_file].
// Tier 3 — Re-export (2-hop): walks deps of deps; visited set prevents cycles.
// Tier 4 — Global name fallback: scans every file's symbols for an exact
//          name match. This is the name-shard fallback that brings
//          unresolved_edge_pct below the plan stub_ban threshold on real
//          Rust codebases where most call sites reference short names whose
//          definitions live beyond the 2-hop import window.
// Confidence: 0 candidates -> Unresolved, 1 -> Unique, >1 -> MultipleCandidates.

use std::collections::{BTreeMap, BTreeSet};

use super::types::{Confidence, SymbolEdge, SymbolNode};

/// Resolve call sites for all files in `outgoing_calls_by_file`.
///
/// `deps_by_file`      — file -> direct dependency files (from DKG imports).
/// `symbols_by_file`   — file -> all SymbolNodes in that file.
/// `outgoing_calls_by_file` — mutated in place; edges start as Unresolved.
pub fn resolve_call_sites(
    deps_by_file: &BTreeMap<String, Vec<String>>,
    symbols_by_file: &BTreeMap<String, Vec<SymbolNode>>,
    outgoing_calls_by_file: &mut BTreeMap<String, Vec<SymbolEdge>>,
) {
    // Collect the caller files upfront to avoid borrow conflicts.
    let caller_files: Vec<String> = outgoing_calls_by_file.keys().cloned().collect();

    // Pre-build a global name -> [SymbolNode.id] index over ALL files once,
    // so the Tier 4 fallback does a single hash lookup per edge instead of
    // scanning every file on each unresolved call site. This is the
    // in-memory equivalent of the persisted `symbol_name_index` shards and is
    // what brings unresolved_edge_pct below the plan stub_ban threshold on
    // real Rust codebases where short callee names live beyond the 2-hop
    // import window.
    let global_name_index: BTreeMap<&str, Vec<String>> = build_global_name_index(symbols_by_file);

    for caller_file in &caller_files {
        let local_symbols: &[SymbolNode] = symbols_by_file
            .get(caller_file.as_str())
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        let direct_deps: &[String] = deps_by_file
            .get(caller_file.as_str())
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        let Some(edges) = outgoing_calls_by_file.get_mut(caller_file.as_str()) else {
            // Defensive: key existed when we cloned, so this branch is unreachable
            // unless another thread mutated the map concurrently (we hold &mut).
            tracing::warn!("B29 resolution: caller_file {caller_file} vanished mid-iteration");
            continue;
        };

        for edge in edges.iter_mut() {
            // Tiers run in priority order; the first tier that returns true
            // has set edge.confidence + edge.callee_candidates and stops the
            // cascade. If no tier claims the edge it stays Unresolved.
            if resolve_local(edge, local_symbols) {
                continue;
            }
            if resolve_imported(edge, direct_deps, symbols_by_file) {
                continue;
            }
            if resolve_reexport(edge, caller_file, direct_deps, deps_by_file, symbols_by_file) {
                continue;
            }
            resolve_global(edge, caller_file, &global_name_index);
        }
    }
}

/// Tier 1 — Local: same file, exact name match (skip Module symbols).
///
/// Returns `true` if the edge was resolved (Local or MultipleCandidates),
/// `false` if no same-file match exists and the cascade should continue.
fn resolve_local(edge: &mut SymbolEdge, local_symbols: &[SymbolNode]) -> bool {
    let local_matches: Vec<String> = local_symbols
        .iter()
        .filter(|s| s.name == edge.callee_name && s.kind != super::types::SymbolKind::Module)
        .map(|s| s.id.clone())
        .collect();

    if local_matches.is_empty() {
        return false;
    }
    if local_matches.len() == 1 {
        edge.confidence = Confidence::Local;
        edge.callee_candidates = local_matches;
    } else {
        let mut sorted = local_matches;
        sorted.sort();
        edge.confidence = Confidence::MultipleCandidates;
        edge.callee_candidates = sorted;
    }
    true
}

/// Tier 2 — Imported (1-hop direct dependencies).
///
/// Returns `true` if at least one candidate was found in a direct dep.
fn resolve_imported(
    edge: &mut SymbolEdge,
    direct_deps: &[String],
    symbols_by_file: &BTreeMap<String, Vec<SymbolNode>>,
) -> bool {
    let mut candidates: Vec<String> = Vec::new();
    for dep_file in direct_deps {
        if let Some(dep_syms) = symbols_by_file.get(dep_file.as_str()) {
            for s in dep_syms {
                if s.name == edge.callee_name {
                    candidates.push(s.id.clone());
                }
            }
        }
    }
    if candidates.is_empty() {
        return false;
    }
    candidates.sort();
    candidates.dedup();
    assign_confidence_by_count(edge, candidates);
    true
}

/// Tier 3 — Re-export (2-hop: deps of direct deps).
///
/// Walks each direct dep's own deps; skips the caller file and direct deps
/// already searched in Tier 2. Cycle prevention via a BTreeSet of visited
/// files. Returns `true` if at least one candidate was found.
fn resolve_reexport(
    edge: &mut SymbolEdge,
    caller_file: &str,
    direct_deps: &[String],
    deps_by_file: &BTreeMap<String, Vec<String>>,
    symbols_by_file: &BTreeMap<String, Vec<SymbolNode>>,
) -> bool {
    let mut visited: BTreeSet<&str> = BTreeSet::new();
    visited.insert(caller_file);
    // Mark all direct deps as visited (already searched in Tier 2).
    for dep in direct_deps {
        visited.insert(dep.as_str());
    }

    let mut candidates: Vec<String> = Vec::new();
    for dep_file in direct_deps {
        let transitive_deps = deps_by_file
            .get(dep_file.as_str())
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        for transitive_dep in transitive_deps {
            if visited.contains(transitive_dep.as_str()) {
                continue;
            }
            visited.insert(transitive_dep.as_str());

            if let Some(syms) = symbols_by_file.get(transitive_dep.as_str()) {
                for s in syms {
                    if s.name == edge.callee_name {
                        candidates.push(s.id.clone());
                    }
                }
            }
        }
    }

    if candidates.is_empty() {
        return false;
    }
    candidates.sort();
    candidates.dedup();
    assign_confidence_by_count(edge, candidates);
    true
}

/// Tier 4 — Global name fallback.
///
/// The 1-hop + 2-hop import walk misses most Rust call sites because short
/// callee names (e.g. `clone`, `new`, `default`) are defined in files far
/// outside the caller's import closure. Falls back to a global exact-name
/// lookup across every file's symbols, excluding same-file matches already
/// handled by Tier 1. Returns `true` if a non-empty cross-file candidate set
/// was assigned.
fn resolve_global(
    edge: &mut SymbolEdge,
    caller_file: &str,
    global_name_index: &BTreeMap<&str, Vec<String>>,
) -> bool {
    let Some(global_matches) = global_name_index.get(edge.callee_name.as_str()) else {
        return false;
    };
    if global_matches.is_empty() {
        return false;
    }
    // Hoist the prefix allocation out of the retain closure — one allocation
    // per edge instead of one per id checked.
    let prefix = format!("{caller_file}::");
    let mut fallback: Vec<String> = global_matches
        .iter()
        .filter(|id| !id.starts_with(&prefix))
        .cloned()
        .collect();
    if fallback.is_empty() {
        return false;
    }
    fallback.sort();
    fallback.dedup();
    assign_confidence_by_count(edge, fallback);
    true
}

/// Build a global `name -> [SymbolNode.id]` index over every file's symbols.
///
/// Used by the Tier 4 fallback in [`resolve_call_sites`]. This is the
/// in-memory equivalent of the persisted `symbol_name_index` shards; building
/// it once per resolve pass keeps the fallback at O(1) lookup per edge rather
/// than O(files × symbols). Module-kind symbols are excluded to match the
/// Tier 1 local-match rule (module names are not callable callees).
fn build_global_name_index<'a>(
    symbols_by_file: &'a BTreeMap<String, Vec<SymbolNode>>,
) -> BTreeMap<&'a str, Vec<String>> {
    let mut index: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for symbols in symbols_by_file.values() {
        for s in symbols {
            if s.kind == super::types::SymbolKind::Module {
                continue;
            }
            index.entry(s.name.as_str()).or_default().push(s.id.clone());
        }
    }
    index
}

/// Assign confidence and candidates based on candidate count.
/// Confidence::Local is handled in Tier 1; this function covers cross-file tiers.
fn assign_confidence_by_count(edge: &mut SymbolEdge, candidates: Vec<String>) {
    match candidates.len() {
        0 => {
            edge.confidence = Confidence::Unresolved;
            edge.callee_candidates = vec![];
        }
        1 => {
            edge.confidence = Confidence::Unique;
            edge.callee_candidates = candidates;
        }
        _ => {
            edge.confidence = Confidence::MultipleCandidates;
            edge.callee_candidates = candidates;
        }
    }
}

/// Return the percentage (0.0–100.0) of edges whose confidence is Unresolved.
///
/// Returns 0.0 if there are no edges.
pub fn compute_unresolved_pct(outgoing_calls_by_file: &BTreeMap<String, Vec<SymbolEdge>>) -> f64 {
    let mut total: usize = 0;
    let mut unresolved: usize = 0;

    for edges in outgoing_calls_by_file.values() {
        for edge in edges {
            total += 1;
            if edge.confidence == Confidence::Unresolved {
                unresolved += 1;
            }
        }
    }

    if total == 0 {
        return 0.0;
    }

    (unresolved as f64 / total as f64) * 100.0
}

/// Derive a file -> [dependency files] map from the DKG SQLite database.
///
/// Joins `dkg_edges` through `dkg_nodes` (source and target) to extract
/// file-level import edges. Only inter-file edges are included.
///
/// This function is provided as a utility for callers that have a live
/// `DkgEngine` connection; it is not required by the core resolution logic.
pub fn build_file_dep_map(
    conn: &rusqlite::Connection,
) -> Result<BTreeMap<String, Vec<String>>, crate::KnowledgeError> {
    let sql = "
        SELECT DISTINCT n_src.file_path, n_tgt.file_path
        FROM dkg_edges e
        JOIN dkg_nodes n_src ON n_src.id = e.source_id
        JOIN dkg_nodes n_tgt ON n_tgt.id = e.target_id
        WHERE e.edge_type = 'imports'
          AND n_src.file_path != n_tgt.file_path
    ";

    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| crate::KnowledgeError::Graph(format!("build_file_dep_map prepare: {e}")))?;

    let mut map: BTreeMap<String, Vec<String>> = BTreeMap::new();

    let rows = stmt
        .query_map([], |row| {
            let src: String = row.get(0)?;
            let tgt: String = row.get(1)?;
            Ok((src, tgt))
        })
        .map_err(|e| crate::KnowledgeError::Graph(format!("build_file_dep_map query: {e}")))?;

    for row in rows {
        let (src, tgt) =
            row.map_err(|e| crate::KnowledgeError::Graph(format!("build_file_dep_map row: {e}")))?;
        map.entry(src).or_default().push(tgt);
    }

    // Sort dep lists for determinism.
    for deps in map.values_mut() {
        deps.sort();
        deps.dedup();
    }

    Ok(map)
}
