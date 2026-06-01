//! E-40 Tranche B-2 — Dataflow analyzer over `SymbolGraphCache`.
//!
//! Best-effort dataflow over the call edges loaded from
//! `.claude/tools/elai_graph.json` plus heuristic def/use edges
//! synthesized at query time. The full SCIP-protobuf-backed
//! ScopeGraph + per-language scope walker is deferred to Tranche B-3
//! (TASK-E40-002); for B-2 we approximate dataflow with the call
//! edges already in the cache:
//!
//! - `impact_analysis(symbol)` — BFS the reverse-call index from any
//!   def of `symbol` up to `depth_cap` hops, returning the union of
//!   impacted symbol names. This is the canonical 5th-list RRF retriever
//!   plus the producer for `SymbolGraphEventKind::DataflowImpactComputed`.
//! - `defuse_edges(symbol)` — for every def of `symbol`, emit a
//!   `DefUse(def_id -> caller_id)` edge per caller in the reverse-call
//!   index. Cheap, deterministic, no protobuf required.
//!
//! Anti-stale-training rule compliance:
//! - `BTreeSet` (NOT `HashSet`) for visited tracking — deterministic
//!   iteration matters for the contract test golden file.
//! - `tracing` (never `log`) — no diagnostics yet but the pattern is
//!   reserved.
//! - Named constants — `DEFAULT_DEPTH_CAP`.

use std::collections::{BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};

use super::cache::SymbolGraphCache;
use super::query::find_defs;

/// Default depth cap when the caller passes 0 (or a value larger than
/// the safety ceiling). Matches the 2-hop B-29.5 p99 ≤ 500 ms budget.
pub const DEFAULT_DEPTH_CAP: u8 = 2;

/// Hard safety ceiling — refuses BFS deeper than this regardless of
/// the caller's request. Matches the existing symbol-graph tool
/// `depth: 1..=16` schema cap.
pub const MAX_DEPTH_CAP: u8 = 16;

/// Dataflow edge kind discriminator. Mirrors
/// `elai_ipc_types::DataflowEdgeKind` 1:1 (the helper at
/// `crate::symbol_graph::dataflow_edge_kind_to_ipc` does the mapping
/// at the SSB / IPC boundary).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataflowEdgeKind {
    /// Definition -> first use (the canonical dataflow edge).
    DefUse,
    /// Use -> definition (reverse direction; surfaced for refine+filter).
    UseDef,
    /// Variable assignment flow (a -> b where b reads a's value).
    AssignFlow,
    /// Call chain (a -> b where a invokes b).
    CallChain,
}

/// A single dataflow edge. `source` and `target` are SymbolGraphCache
/// node ids (stable across runs since the JSON is offline-generated).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataflowEdge {
    pub source: String,
    pub target: String,
    pub edge_kind: DataflowEdgeKind,
}

/// Analyzer wrapping a mutable cache borrow. Keeps a small amount of
/// per-call observability state (`last_edge_count`,
/// `last_depth_reached`) so the SSB `DataflowImpactComputed` payload
/// can carry accurate edge counts without a second traversal.
pub struct DataflowAnalyzer<'a> {
    cache: &'a mut SymbolGraphCache,
    depth_cap: u8,
    last_edge_count: usize,
    last_depth_reached: u8,
}

impl<'a> DataflowAnalyzer<'a> {
    /// Construct a new analyzer. `depth_cap` is clamped to
    /// [`DEFAULT_DEPTH_CAP`, [`MAX_DEPTH_CAP`]]: a `0` or
    /// out-of-range value collapses to the default rather than
    /// panicking.
    pub fn new(cache: &'a mut SymbolGraphCache, depth_cap: u8) -> Self {
        let clamped = if depth_cap == 0 {
            DEFAULT_DEPTH_CAP
        } else if depth_cap > MAX_DEPTH_CAP {
            MAX_DEPTH_CAP
        } else {
            depth_cap
        };
        Self {
            cache,
            depth_cap: clamped,
            last_edge_count: 0,
            last_depth_reached: 0,
        }
    }

    /// Number of edges traversed in the last `impact_analysis` call.
    pub fn last_edge_count(&self) -> usize {
        self.last_edge_count
    }

    /// Maximum BFS depth reached in the last `impact_analysis` call.
    pub fn last_depth_reached(&self) -> u8 {
        self.last_depth_reached
    }

    /// BFS the reverse-call index from every def of `symbol` up to
    /// `depth_cap` hops, returning the union of impacted symbol names
    /// in deterministic order. Empty result on empty / unknown symbol.
    pub fn impact_analysis(&mut self, symbol: &str) -> Vec<String> {
        self.last_edge_count = 0;
        self.last_depth_reached = 0;
        if symbol.is_empty() {
            return Vec::new();
        }
        let seed_ids: Vec<String> = self.cache.ids_by_name(symbol);
        if seed_ids.is_empty() {
            return Vec::new();
        }
        let mut visited: BTreeSet<String> = BTreeSet::new();
        let mut frontier: VecDeque<(String, u8)> = VecDeque::new();
        for id in &seed_ids {
            if visited.insert(id.clone()) {
                frontier.push_back((id.clone(), 0));
            }
        }
        let mut max_depth: u8 = 0;
        let mut edge_count: usize = 0;
        while let Some((id, depth)) = frontier.pop_front() {
            if depth >= self.depth_cap {
                continue;
            }
            let next_depth = depth.saturating_add(1);
            let callers = self.cache.callers_of(&id);
            for caller in callers {
                edge_count = edge_count.saturating_add(1);
                if visited.insert(caller.clone()) {
                    if next_depth > max_depth {
                        max_depth = next_depth;
                    }
                    frontier.push_back((caller, next_depth));
                }
            }
        }
        self.last_edge_count = edge_count;
        self.last_depth_reached = max_depth;
        // Remove the seed ids from the output (the impacted set is the
        // BFS frontier MINUS the originating defs).
        let seed_set: BTreeSet<&String> = seed_ids.iter().collect();
        let mut impacted: Vec<String> = visited
            .into_iter()
            .filter(|id| !seed_set.contains(id))
            .collect();
        impacted.sort();
        impacted
    }

    /// Synthesize heuristic def-use edges for every def of `symbol`.
    /// Each def yields one `DefUse(def_id -> caller_id)` edge per
    /// caller in the reverse-call index. Deterministic order.
    pub fn defuse_edges(&mut self, symbol: &str) -> Vec<DataflowEdge> {
        if symbol.is_empty() {
            return Vec::new();
        }
        let defs = match find_defs(self.cache, symbol) {
            Ok(defs) => defs,
            Err(_) => return Vec::new(),
        };
        let mut edges: Vec<DataflowEdge> = Vec::new();
        for def in defs {
            for caller_id in self.cache.callers_of(&def.id) {
                edges.push(DataflowEdge {
                    source: def.id.clone(),
                    target: caller_id,
                    edge_kind: DataflowEdgeKind::DefUse,
                });
            }
        }
        edges
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbol_graph::cache::SymbolNode;

    fn build_cache() -> SymbolGraphCache {
        let mut cache = SymbolGraphCache::empty_with_capacity(16).expect("empty");
        // Chain: caller -> mid -> leaf
        cache.index_node(SymbolNode {
            id: "src/a.rs:caller:10".into(),
            name: "caller".into(),
            file: "src/a.rs".into(),
            line: 10,
            kind: "function".into(),
            calls: vec!["src/b.rs:mid:20".into()],
        });
        cache.index_node(SymbolNode {
            id: "src/b.rs:mid:20".into(),
            name: "mid".into(),
            file: "src/b.rs".into(),
            line: 20,
            kind: "function".into(),
            calls: vec!["src/c.rs:leaf:30".into()],
        });
        cache.index_node(SymbolNode {
            id: "src/c.rs:leaf:30".into(),
            name: "leaf".into(),
            file: "src/c.rs".into(),
            line: 30,
            kind: "function".into(),
            calls: vec![],
        });
        cache
    }

    #[test]
    fn impact_analysis_respects_depth_cap_1() {
        let mut cache = build_cache();
        let mut a = DataflowAnalyzer::new(&mut cache, 1);
        let out = a.impact_analysis("leaf");
        // depth=1 should surface mid only (1 hop back from leaf).
        assert!(out.iter().any(|s| s.contains("mid")));
        assert!(!out.iter().any(|s| s.contains("caller")));
        assert!(a.last_depth_reached() <= 1);
    }

    #[test]
    fn impact_analysis_respects_depth_cap_2() {
        let mut cache = build_cache();
        let mut a = DataflowAnalyzer::new(&mut cache, 2);
        let out = a.impact_analysis("leaf");
        // depth=2 surfaces both mid + caller.
        assert!(out.iter().any(|s| s.contains("mid")));
        assert!(out.iter().any(|s| s.contains("caller")));
        assert!(a.last_edge_count() >= 2);
    }

    #[test]
    fn empty_symbol_returns_empty() {
        let mut cache = build_cache();
        let mut a = DataflowAnalyzer::new(&mut cache, 2);
        let out = a.impact_analysis("");
        assert!(out.is_empty());
    }

    #[test]
    fn defuse_edges_returns_caller_pairs() {
        let mut cache = build_cache();
        let mut a = DataflowAnalyzer::new(&mut cache, 2);
        let edges = a.defuse_edges("mid");
        assert_eq!(edges.len(), 1);
        assert!(matches!(edges[0].edge_kind, DataflowEdgeKind::DefUse));
        assert!(edges[0].target.contains("caller"));
    }

    #[test]
    fn depth_cap_zero_clamps_to_default() {
        let mut cache = build_cache();
        let a = DataflowAnalyzer::new(&mut cache, 0);
        assert_eq!(a.depth_cap, DEFAULT_DEPTH_CAP);
    }
}
