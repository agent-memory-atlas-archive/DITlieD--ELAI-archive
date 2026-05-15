// B-29 Phase 3 — four traversal functions over the symbol graph.
//
// All functions accept `&mut SymbolGraphCache` because the lazy loaders
// (`ensure_name_index`, `ensure_reverse_index`, `get_or_load_payload`)
// require mutable access.
//
// Algorithm sources: SocratiCode graph-impact.ts (271 lines, verified 2026-05-15).
//   compute_impact_radius — BFS, graph-impact.ts:30-95
//   compute_call_flow     — DFS, graph-impact.ts:110-170
//   compute_symbol_context — 360°, graph-impact.ts:180-228
//   list_symbols          — graph-impact.ts:232-263
//
// Key correctness invariants (verified against SocratiCode source):
//   - Cycle check BEFORE depth check in DFS (SocratiCode line 141 before 146).
//   - BFS frontier = callee files; reverse_file_index lookup gives caller files.
//   - Edges with empty callee_candidates are Unresolved — not traversable.
//   - "<module>" synthetic symbol is skipped in list_symbols.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::KnowledgeError;

use super::cache::SymbolGraphCache;
use super::sharding::{MAX_FLOW_DEPTH, MAX_IMPACT_DEPTH};
use super::types::{Confidence, SymbolEdge, SymbolKind, SymbolNode};

// ---------------------------------------------------------------------------
// Output types
// ---------------------------------------------------------------------------

/// Reason a DFS path was cut short.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlowTruncationReason {
    /// Depth limit reached.
    Depth,
    /// Cycle detected (this node's id was already in the visited set).
    Cycle,
}

/// A single node in the call-flow DFS tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowNode {
    pub symbol_id: String,
    pub name: String,
    pub file: String,
    pub line: u32,
    pub kind: SymbolKind,
    pub children: Vec<FlowNode>,
    pub truncated_reason: Option<FlowTruncationReason>,
}

/// Caller reference gathered during `compute_symbol_context`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallerInfo {
    pub caller_id: String,
    pub callee_name: String,
    pub call_site_file: String,
    pub call_site_line: u32,
    pub confidence: Confidence,
}

/// Callee reference gathered during `compute_symbol_context`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalleeInfo {
    pub callee_id: String,
    pub callee_name: String,
    pub confidence: Confidence,
}

/// 360° view of a single symbol definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolContext {
    pub definition: SymbolNode,
    pub callers: Vec<CallerInfo>,
    pub callees: Vec<CalleeInfo>,
}

/// Result of a file/symbol impact-radius query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactResult {
    pub target: String,
    /// All files reachable from the target, flattened.
    pub resolved_files: Vec<String>,
    /// Files grouped by BFS hop depth (0 = seed layer).
    pub files_by_depth: BTreeMap<u32, Vec<String>>,
    /// Total reachable file count.
    pub total_files: usize,
    /// The maximum depth actually reached (clamped to MAX_IMPACT_DEPTH).
    pub max_depth_reached: u32,
    /// True when callers exist beyond the depth limit.
    pub truncated: bool,
}

/// Filter for `list_symbols`.
#[derive(Debug, Clone, Default)]
pub struct SymbolFilter {
    /// Restrict to a single source file.
    pub file: Option<String>,
    /// Substring match on symbol name.
    pub query: Option<String>,
    /// Maximum number of results.  0 means "use default cap (100)".
    pub limit: usize,
}

// Alias so contract tests can use either name.
pub use SymbolFilter as ListSymbolsFilter;

// ---------------------------------------------------------------------------
// Public helpers
// ---------------------------------------------------------------------------

/// Heuristic: return `true` if `target` looks like a file path rather than a
/// symbol name.
///
/// Mirrors SocratiCode `looksLikeFilePath` (graph-impact.ts:268-270).
/// Rules:
///   - Contains `/` or `\` (path separator)
///   - Ends with a known source-file extension (case-insensitive suffix match)
pub fn looks_like_file_path(target: &str) -> bool {
    if target.contains('/') || target.contains('\\') {
        return true;
    }
    const EXTS: &[&str] = &[
        ".rs", ".py", ".ts", ".tsx", ".js", ".go", ".java", ".cs", ".rb", ".kt", ".scala",
        ".cpp", ".cc", ".c", ".h", ".hpp", ".svelte", ".vue", ".php", ".swift",
    ];
    let lower = target.to_ascii_lowercase();
    EXTS.iter().any(|ext| lower.ends_with(ext))
}

/// Extract the file prefix from a symbol id formatted as `{file}::{rest}`.
///
/// Uses `find("::")` (first occurrence) to avoid splitting on `::` inside
/// symbol qualified names.  Returns `None` when `id` contains no `::`.
pub fn symbol_id_to_file(id: &str) -> Option<&str> {
    id.find("::").map(|pos| &id[..pos])
}

// ---------------------------------------------------------------------------
// compute_impact_radius
// ---------------------------------------------------------------------------

/// BFS reverse-dependency traversal.
///
/// `target` is either a file path or a symbol name (detected via
/// `looks_like_file_path`).  Depth is clamped to `MAX_IMPACT_DEPTH`.
pub fn compute_impact_radius(
    cache: &mut SymbolGraphCache,
    target: &str,
    depth: u32,
) -> Result<ImpactResult, KnowledgeError> {
    let effective_depth = depth.min(MAX_IMPACT_DEPTH);

    // Determine seed layer (depth 0).
    let seed_files: BTreeSet<String> = if looks_like_file_path(target) {
        let mut s = BTreeSet::new();
        s.insert(target.to_string());
        s
    } else {
        let name_idx = cache.ensure_name_index()?;
        match name_idx.get(target) {
            None => BTreeSet::new(),
            Some(refs) => refs.iter().map(|r| r.file.clone()).collect(),
        }
    };

    if seed_files.is_empty() {
        return Ok(ImpactResult {
            target: target.to_string(),
            resolved_files: vec![],
            files_by_depth: BTreeMap::new(),
            total_files: 0,
            max_depth_reached: 0,
            truncated: false,
        });
    }

    let mut files_by_depth: BTreeMap<u32, Vec<String>> = BTreeMap::new();
    let mut visited: BTreeSet<String> = BTreeSet::new();
    let mut frontier: BTreeSet<String> = seed_files;

    for f in &frontier {
        visited.insert(f.clone());
    }
    let mut layer_0: Vec<String> = frontier.iter().cloned().collect();
    layer_0.sort();
    files_by_depth.insert(0, layer_0);

    let mut max_depth_reached: u32 = 0;

    for hop in 1..=effective_depth {
        // Load reverse index snapshot before the frontier loop.
        // We need to snapshot the relevant lookups to avoid borrow conflicts.
        let next_callers: Vec<String> = {
            let reverse_idx = cache.ensure_reverse_index()?;
            let mut acc: BTreeSet<String> = BTreeSet::new();
            for callee_file in &frontier {
                if let Some(callers) = reverse_idx.get(callee_file.as_str()) {
                    for caller in callers {
                        if !visited.contains(caller) {
                            acc.insert(caller.clone());
                        }
                    }
                }
            }
            acc.into_iter().collect()
        };

        if next_callers.is_empty() {
            break;
        }

        let mut next_frontier: BTreeSet<String> = BTreeSet::new();
        for f in next_callers {
            visited.insert(f.clone());
            next_frontier.insert(f);
        }

        let mut layer: Vec<String> = next_frontier.iter().cloned().collect();
        layer.sort();
        files_by_depth.insert(hop, layer);
        max_depth_reached = hop;
        frontier = next_frontier;
    }

    // Truncation check: would there be more files at depth+1?
    let truncated = if !frontier.is_empty() {
        let reverse_idx = cache.ensure_reverse_index()?;
        frontier.iter().any(|callee_file| {
            reverse_idx
                .get(callee_file.as_str())
                .map(|callers| callers.iter().any(|c| !visited.contains(c)))
                .unwrap_or(false)
        })
    } else {
        false
    };

    let resolved_files: Vec<String> = visited.iter().cloned().collect();
    let total_files = resolved_files.len();

    Ok(ImpactResult {
        target: target.to_string(),
        resolved_files,
        files_by_depth,
        total_files,
        max_depth_reached,
        truncated,
    })
}

// ---------------------------------------------------------------------------
// compute_call_flow
// ---------------------------------------------------------------------------

/// DFS call-flow starting from `entrypoint_id`.
///
/// Returns `None` when the entrypoint symbol cannot be located (file not
/// indexed or symbol id not found in the payload).
///
/// Depth is clamped to `MAX_FLOW_DEPTH`.
///
/// Cycle detection: checked BEFORE depth check (matching SocratiCode line 141
/// before 146).
pub fn compute_call_flow(
    cache: &mut SymbolGraphCache,
    entrypoint_id: &str,
    depth: u32,
) -> Result<Option<FlowNode>, KnowledgeError> {
    let effective_max = depth.min(MAX_FLOW_DEPTH);

    let file = match symbol_id_to_file(entrypoint_id) {
        Some(f) => f.to_string(),
        None => return Ok(None),
    };

    let payload_arc = match cache.get_or_load_payload(&file)? {
        Some(p) => p,
        None => return Ok(None),
    };

    let sym = match payload_arc
        .symbols
        .iter()
        .find(|s| s.id == entrypoint_id)
        .cloned()
    {
        Some(s) => s,
        None => return Ok(None),
    };
    drop(payload_arc);

    let mut visited: BTreeSet<String> = BTreeSet::new();
    let node = call_flow_walk(
        cache,
        &sym.id,
        &sym.name,
        &sym.file,
        sym.line,
        sym.kind,
        0,
        effective_max,
        &mut visited,
    )?;
    Ok(Some(node))
}

/// Recursive DFS helper (inner, used only by `compute_call_flow`).
fn call_flow_walk(
    cache: &mut SymbolGraphCache,
    sym_id: &str,
    sym_name: &str,
    sym_file: &str,
    sym_line: u32,
    sym_kind: SymbolKind,
    current_depth: u32,
    max_depth: u32,
    visited: &mut BTreeSet<String>,
) -> Result<FlowNode, KnowledgeError> {
    // Step 1: cycle check BEFORE depth check (SocratiCode line 141 before 146).
    if visited.contains(sym_id) {
        return Ok(FlowNode {
            symbol_id: sym_id.to_string(),
            name: sym_name.to_string(),
            file: sym_file.to_string(),
            line: sym_line,
            kind: sym_kind,
            children: vec![],
            truncated_reason: Some(FlowTruncationReason::Cycle),
        });
    }

    // Step 2: depth check.
    if current_depth >= max_depth {
        return Ok(FlowNode {
            symbol_id: sym_id.to_string(),
            name: sym_name.to_string(),
            file: sym_file.to_string(),
            line: sym_line,
            kind: sym_kind,
            children: vec![],
            truncated_reason: Some(FlowTruncationReason::Depth),
        });
    }

    // Step 3: mark visited.
    visited.insert(sym_id.to_string());

    // Step 4: load payload; collect outgoing edges for this symbol.
    let edges: Vec<SymbolEdge> = match cache.get_or_load_payload(sym_file)? {
        Some(p) => p
            .outgoing_calls
            .iter()
            .filter(|e| e.caller_id == sym_id && !e.callee_candidates.is_empty())
            .cloned()
            .collect(),
        None => vec![],
    };

    // Step 5: recurse into each candidate.
    let mut children: Vec<FlowNode> = Vec::new();
    for edge in &edges {
        for candidate_id in &edge.callee_candidates {
            let candidate_file = match symbol_id_to_file(candidate_id) {
                Some(f) => f.to_string(),
                None => continue,
            };

            let candidate_sym = match cache.get_or_load_payload(&candidate_file)? {
                Some(p) => p
                    .symbols
                    .iter()
                    .find(|s| &s.id == candidate_id)
                    .cloned(),
                None => None,
            };

            let Some(csym) = candidate_sym else { continue };

            let child = call_flow_walk(
                cache,
                &csym.id,
                &csym.name,
                &csym.file,
                csym.line,
                csym.kind,
                current_depth + 1,
                max_depth,
                visited,
            )?;
            children.push(child);
        }
    }

    visited.remove(sym_id);

    Ok(FlowNode {
        symbol_id: sym_id.to_string(),
        name: sym_name.to_string(),
        file: sym_file.to_string(),
        line: sym_line,
        kind: sym_kind,
        children,
        truncated_reason: None,
    })
}

// ---------------------------------------------------------------------------
// compute_symbol_context
// ---------------------------------------------------------------------------

/// Gather a 360° view for all definitions of `name`.
///
/// If `file_hint` is supplied, only the definition in that file is returned.
pub fn compute_symbol_context(
    cache: &mut SymbolGraphCache,
    name: &str,
    file_hint: Option<&str>,
) -> Result<Vec<SymbolContext>, KnowledgeError> {
    let refs = {
        let name_idx = cache.ensure_name_index()?;
        match name_idx.get(name) {
            None => return Ok(vec![]),
            Some(r) => r.clone(),
        }
    };

    let filtered_refs: Vec<_> = if let Some(hint) = file_hint {
        refs.into_iter().filter(|r| r.file == hint).collect()
    } else {
        refs
    };

    let mut contexts: Vec<SymbolContext> = Vec::new();

    for sym_ref in &filtered_refs {
        // Load payload for the definition file.
        let (sym_node, outgoing_calls_clone) = match cache.get_or_load_payload(&sym_ref.file)? {
            Some(p) => {
                let node = p
                    .symbols
                    .iter()
                    .find(|s| s.name == name && s.file == sym_ref.file)
                    .cloned();
                let oc = p.outgoing_calls.clone();
                (node, oc)
            }
            None => continue,
        };

        let Some(sym_node) = sym_node else { continue };

        // Callees: outgoing edges from this symbol with resolved candidates.
        let callees: Vec<CalleeInfo> = outgoing_calls_clone
            .iter()
            .filter(|e| e.caller_id == sym_node.id && !e.callee_candidates.is_empty())
            .flat_map(|e| {
                e.callee_candidates.iter().map(move |cid| CalleeInfo {
                    callee_id: cid.clone(),
                    callee_name: e.callee_name.clone(),
                    confidence: e.confidence,
                })
            })
            .collect();

        // Callers: files in reverse_file_index[sym_ref.file] that call this symbol.
        let caller_files: Vec<String> = {
            let reverse_idx = cache.ensure_reverse_index()?;
            reverse_idx
                .get(sym_ref.file.as_str())
                .map(|s| s.iter().cloned().collect())
                .unwrap_or_default()
        };

        let mut callers: Vec<CallerInfo> = Vec::new();
        for caller_file in &caller_files {
            match cache.get_or_load_payload(caller_file)? {
                Some(p) => {
                    let matching: Vec<CallerInfo> = p
                        .outgoing_calls
                        .iter()
                        .filter(|e| {
                            e.callee_candidates
                                .iter()
                                .any(|cid| cid == &sym_node.id)
                        })
                        .map(|e| CallerInfo {
                            caller_id: e.caller_id.clone(),
                            callee_name: e.callee_name.clone(),
                            call_site_file: e.call_site.file.clone(),
                            call_site_line: e.call_site.line,
                            confidence: e.confidence,
                        })
                        .collect();
                    callers.extend(matching);
                }
                None => continue,
            }
        }

        contexts.push(SymbolContext {
            definition: sym_node,
            callers,
            callees,
        });
    }

    Ok(contexts)
}

// ---------------------------------------------------------------------------
// list_symbols
// ---------------------------------------------------------------------------

/// List symbol definitions, with optional file and substring filters.
///
/// The `<module>` synthetic symbol (kind == Module, name == "<module>") is
/// excluded to match SocratiCode behaviour (graph-impact.ts:237).
///
/// `filter.limit == 0` applies a default cap of 100.
pub fn list_symbols(
    cache: &mut SymbolGraphCache,
    filter: SymbolFilter,
) -> Result<Vec<SymbolNode>, KnowledgeError> {
    let effective_limit = if filter.limit == 0 { 100 } else { filter.limit };
    let mut results: Vec<SymbolNode> = Vec::new();

    if let Some(ref file) = filter.file {
        let payload_arc = match cache.get_or_load_payload(file)? {
            Some(p) => p,
            None => return Ok(vec![]),
        };
        for sym in &payload_arc.symbols {
            if is_module_synthetic(sym) {
                continue;
            }
            if let Some(ref q) = filter.query {
                if !sym.name.contains(q.as_str()) {
                    continue;
                }
            }
            results.push(sym.clone());
            if results.len() >= effective_limit {
                break;
            }
        }
    } else {
        // All-files mode: discover files via name index, then load payloads.
        let all_files: Vec<String> = {
            let name_idx = cache.ensure_name_index()?;
            let mut files: BTreeSet<String> = BTreeSet::new();
            for refs in name_idx.values() {
                for r in refs {
                    files.insert(r.file.clone());
                }
            }
            files.into_iter().collect()
        };

        'outer: for file in &all_files {
            let payload_arc = match cache.get_or_load_payload(file)? {
                Some(p) => p,
                None => continue,
            };
            for sym in &payload_arc.symbols {
                if is_module_synthetic(sym) {
                    continue;
                }
                if let Some(ref q) = filter.query {
                    if !sym.name.contains(q.as_str()) {
                        continue;
                    }
                }
                results.push(sym.clone());
                if results.len() >= effective_limit {
                    break 'outer;
                }
            }
        }
    }

    Ok(results)
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

fn is_module_synthetic(sym: &SymbolNode) -> bool {
    sym.kind == SymbolKind::Module && sym.name == "<module>"
}
