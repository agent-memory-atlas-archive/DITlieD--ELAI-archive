//! E-40 — `SymbolGraphCache`: JSON-rooted symbol-graph view.
//!
//! Reads `.claude/tools/elai_graph.json` (offline-generated SCIP-enriched
//! map per CLAUDE.md "Codebase map" block) and exposes a deterministic,
//! BTreeMap-backed in-memory view with a hand-rolled LRU capacity bound.
//!
//! Anti-stale-training rule compliance:
//! - `BTreeMap` (NOT `HashMap`) for deterministic iteration
//! - `blake3` for any hashing (not blake2/sha2)
//! - `thiserror` for the error type (not Box<dyn Error>)
//! - `tracing` for diagnostics (not log)
//!
//! Capacity bound: defaults to 1000 cached lookups via the touch-order LRU
//! (the underlying node store itself is unbounded — the LRU caps lookups
//! so repeated `find_defs("X")` for distinct `X` does not grow forever).
//!
//! This is a different surface from `elai_knowledge::symbol_graph::cache::SymbolGraphCache`
//! (B-29 SQLite-rooted, 27+256+500-LRU). Both coexist; consumers
//! disambiguate via crate path. See `workflow/reasoning/E-40-decisions.md`.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::warn;

/// Default LRU capacity for cached lookups.
pub const DEFAULT_LRU_CAPACITY: usize = 1000;

/// Error surface for `SymbolGraphCache` operations.
#[derive(Debug, Error)]
pub enum SymbolGraphCacheError {
    /// JSON file at `path` could not be read.
    #[error("symbol_graph cache: failed to read graph json at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    /// JSON parsed but the shape did not match the expected schema.
    #[error("symbol_graph cache: failed to parse graph json at {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    /// Requested capacity zero (lru would never store entries).
    #[error("symbol_graph cache: lru capacity must be > 0")]
    InvalidCapacity,
}

/// A single symbol node loaded from `elai_graph.json`.
///
/// Mirrors the JSON shape produced by `.claude/tools/codebase_mapper.py`:
/// each value under `functions`, `methods`, etc. has `name` / `file` /
/// `line` / `kind` / optional `signature`. Unknown fields are ignored
/// (forward-compatible with future mapper additions).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolNode {
    pub name: String,
    pub file: String,
    pub line: u32,
    #[serde(default)]
    pub kind: String,
    /// Stable id assigned at load time — `<file>:<name>:<line>`. Used as
    /// the LRU key and as the personalization vector seed in
    /// `pagerank::personalized_top_k`.
    pub id: String,
    /// Outgoing call edges. JSON sources without a "calls" field land as
    /// an empty vec; the symbol_graph still works (degrades to defs/refs
    /// only).
    #[serde(default)]
    pub calls: Vec<String>,
}

/// Hand-rolled BTreeMap-backed LRU. Touch-order eviction with a u64
/// monotonic counter — deterministic across runs since BTreeMap iterates
/// in key order.
#[derive(Debug)]
struct Lru {
    capacity: usize,
    counter: u64,
    /// counter -> key.
    order: BTreeMap<u64, String>,
    /// key -> (counter, node_id_set).
    entries: BTreeMap<String, (u64, Vec<String>)>,
}

impl Lru {
    fn new(capacity: usize) -> Self {
        Self {
            capacity,
            counter: 0,
            order: BTreeMap::new(),
            entries: BTreeMap::new(),
        }
    }

    fn touch(&mut self, key: &str) {
        if let Some((old_counter, _)) = self.entries.get(key) {
            let old = *old_counter;
            self.order.remove(&old);
            self.counter = self.counter.saturating_add(1);
            let new_counter = self.counter;
            if let Some((stored_counter, _)) = self.entries.get_mut(key) {
                *stored_counter = new_counter;
            }
            self.order.insert(new_counter, key.to_string());
        }
    }

    fn insert(&mut self, key: String, value: Vec<String>) {
        if self.capacity == 0 {
            return;
        }
        if self.entries.contains_key(&key) {
            self.touch(&key);
            if let Some((_, v)) = self.entries.get_mut(&key) {
                *v = value;
            }
            return;
        }
        if self.entries.len() >= self.capacity {
            if let Some((&victim_counter, _)) = self.order.iter().next() {
                if let Some(victim_key) = self.order.remove(&victim_counter) {
                    self.entries.remove(&victim_key);
                }
            }
        }
        self.counter = self.counter.saturating_add(1);
        let c = self.counter;
        self.order.insert(c, key.clone());
        self.entries.insert(key, (c, value));
    }

    fn get(&mut self, key: &str) -> Option<Vec<String>> {
        let value = self.entries.get(key).map(|(_, v)| v.clone());
        if value.is_some() {
            self.touch(key);
        }
        value
    }

    fn len(&self) -> usize {
        self.entries.len()
    }
}

/// JSON-rooted symbol graph view loaded from `elai_graph.json`.
#[derive(Debug)]
pub struct SymbolGraphCache {
    nodes: BTreeMap<String, SymbolNode>,
    /// name -> ids that declare under that name (defs index).
    by_name: BTreeMap<String, Vec<String>>,
    /// id -> ids of nodes whose call edges point at this id (reverse calls).
    reverse_calls: BTreeMap<String, Vec<String>>,
    /// id -> file path (for file-rooted queries).
    by_file: BTreeMap<String, Vec<String>>,
    lookup_lru: Lru,
    graph_path: PathBuf,
}

impl SymbolGraphCache {
    /// Load the JSON view from `graph_path`.
    pub fn load(graph_path: impl Into<PathBuf>) -> Result<Self, SymbolGraphCacheError> {
        Self::load_with_capacity(graph_path, DEFAULT_LRU_CAPACITY)
    }

    /// Load with an explicit LRU capacity. Capacity must be > 0.
    pub fn load_with_capacity(
        graph_path: impl Into<PathBuf>,
        capacity: usize,
    ) -> Result<Self, SymbolGraphCacheError> {
        if capacity == 0 {
            return Err(SymbolGraphCacheError::InvalidCapacity);
        }
        let path: PathBuf = graph_path.into();
        let raw =
            fs::read_to_string(&path).map_err(|source| SymbolGraphCacheError::Io {
                path: path.clone(),
                source,
            })?;
        let mut cache = Self {
            nodes: BTreeMap::new(),
            by_name: BTreeMap::new(),
            reverse_calls: BTreeMap::new(),
            by_file: BTreeMap::new(),
            lookup_lru: Lru::new(capacity),
            graph_path: path.clone(),
        };
        cache.parse_and_index(&raw, &path)?;
        Ok(cache)
    }

    /// Construct an empty cache (no graph file loaded). Used by tests + by
    /// `scip_import::import_scip_indices` when a graph_path is not
    /// available and the caller hydrates entries from SCIP shards directly.
    pub fn empty_with_capacity(capacity: usize) -> Result<Self, SymbolGraphCacheError> {
        if capacity == 0 {
            return Err(SymbolGraphCacheError::InvalidCapacity);
        }
        Ok(Self {
            nodes: BTreeMap::new(),
            by_name: BTreeMap::new(),
            reverse_calls: BTreeMap::new(),
            by_file: BTreeMap::new(),
            lookup_lru: Lru::new(capacity),
            graph_path: PathBuf::new(),
        })
    }

    fn parse_and_index(
        &mut self,
        raw: &str,
        path: &Path,
    ) -> Result<(), SymbolGraphCacheError> {
        // Top-level shape: { "functions": { id: {...}, ... }, "methods":
        // { ... }, ... }. We accept all top-level buckets uniformly —
        // each value's id/name/file/line/calls fields are parsed via
        // RawNode. Unknown fields ignored.
        #[derive(Deserialize)]
        struct RawNode {
            #[serde(default)]
            name: String,
            #[serde(default)]
            file: String,
            #[serde(default)]
            line: u32,
            #[serde(default)]
            kind: String,
            #[serde(default)]
            calls: Vec<String>,
        }
        let raw_value: serde_json::Value =
            serde_json::from_str(raw).map_err(|source| SymbolGraphCacheError::Parse {
                path: path.to_path_buf(),
                source,
            })?;
        let map = match raw_value.as_object() {
            Some(m) => m,
            None => {
                warn!(
                    "symbol_graph cache: graph json root at {} is not a json object; treating as empty",
                    path.display()
                );
                return Ok(());
            }
        };
        for (_bucket, bucket_value) in map {
            let entries = match bucket_value.as_object() {
                Some(m) => m,
                None => continue,
            };
            for (id, node_value) in entries {
                let parsed: RawNode = match serde_json::from_value(node_value.clone()) {
                    Ok(n) => n,
                    Err(e) => {
                        warn!(
                            "symbol_graph cache: skipping node id={} parse_err={}",
                            id, e
                        );
                        continue;
                    }
                };
                let node = SymbolNode {
                    name: parsed.name,
                    file: parsed.file,
                    line: parsed.line,
                    kind: parsed.kind,
                    id: id.clone(),
                    calls: parsed.calls,
                };
                self.index_node(node);
            }
        }
        Ok(())
    }

    /// Insert a node into all the indexes. Public so `scip_import` can
    /// rehydrate the cache from SCIP shards without re-parsing JSON.
    pub fn index_node(&mut self, node: SymbolNode) {
        let id = node.id.clone();
        if !node.name.is_empty() {
            self.by_name
                .entry(node.name.clone())
                .or_default()
                .push(id.clone());
        }
        if !node.file.is_empty() {
            self.by_file
                .entry(node.file.clone())
                .or_default()
                .push(id.clone());
        }
        for target in &node.calls {
            self.reverse_calls
                .entry(target.clone())
                .or_default()
                .push(id.clone());
        }
        self.nodes.insert(id, node);
    }

    /// Total node count.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Lookup a node by id. Touches the LRU.
    pub fn get_node(&mut self, id: &str) -> Option<&SymbolNode> {
        // Touch the LRU with the id as the key + a singleton value list so
        // hot-symbol lookups stay warm. The LRU here is purely an access
        // record — the underlying nodes map is never evicted.
        self.lookup_lru.insert(id.to_string(), vec![id.to_string()]);
        self.nodes.get(id)
    }

    /// Look up nodes by name (defs).
    pub fn ids_by_name(&mut self, name: &str) -> Vec<String> {
        if let Some(cached) = self.lookup_lru.get(name) {
            return cached;
        }
        let ids = self
            .by_name
            .get(name)
            .cloned()
            .unwrap_or_default();
        self.lookup_lru.insert(name.to_string(), ids.clone());
        ids
    }

    /// Look up reverse-call ids (callers of target_id).
    pub fn callers_of(&self, target_id: &str) -> Vec<String> {
        self.reverse_calls.get(target_id).cloned().unwrap_or_default()
    }

    /// Look up forward-call targets from caller_id.
    pub fn callees_of(&self, caller_id: &str) -> Vec<String> {
        self.nodes
            .get(caller_id)
            .map(|n| n.calls.clone())
            .unwrap_or_default()
    }

    /// All nodes — read-only iterator. Used by pagerank seeding +
    /// query::find_refs (broad scan).
    pub fn iter_nodes(&self) -> impl Iterator<Item = (&String, &SymbolNode)> {
        self.nodes.iter()
    }

    /// Path of the graph file (for SSB beacon emission).
    #[must_use]
    pub fn graph_path(&self) -> &Path {
        &self.graph_path
    }

    /// Current LRU occupancy (for diagnostics + the contract test).
    #[must_use]
    pub fn lru_len(&self) -> usize {
        self.lookup_lru.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_node(id: &str, name: &str, calls: Vec<&str>) -> SymbolNode {
        SymbolNode {
            name: name.to_string(),
            file: "src/main.rs".to_string(),
            line: 1,
            kind: "function".to_string(),
            id: id.to_string(),
            calls: calls.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn empty_cache_starts_zero_nodes() {
        let cache = SymbolGraphCache::empty_with_capacity(8).expect("empty");
        assert_eq!(cache.node_count(), 0);
        assert_eq!(cache.lru_len(), 0);
    }

    #[test]
    fn index_node_populates_all_indexes() {
        let mut cache = SymbolGraphCache::empty_with_capacity(8).expect("empty");
        cache.index_node(sample_node("a:foo:10", "foo", vec!["a:bar:20"]));
        cache.index_node(sample_node("a:bar:20", "bar", vec![]));
        assert_eq!(cache.node_count(), 2);
        assert_eq!(cache.ids_by_name("foo"), vec!["a:foo:10".to_string()]);
        assert_eq!(cache.callers_of("a:bar:20"), vec!["a:foo:10".to_string()]);
        assert!(cache.callees_of("a:foo:10").contains(&"a:bar:20".to_string()));
    }

    #[test]
    fn invalid_capacity_zero_errors() {
        match SymbolGraphCache::empty_with_capacity(0) {
            Err(SymbolGraphCacheError::InvalidCapacity) => {}
            other => panic!("expected InvalidCapacity, got {other:?}"),
        }
    }

    #[test]
    fn lru_evicts_oldest_on_overflow() {
        let mut lru = Lru::new(2);
        lru.insert("k1".into(), vec!["v1".into()]);
        lru.insert("k2".into(), vec!["v2".into()]);
        lru.insert("k3".into(), vec!["v3".into()]);
        // k1 must have been evicted (oldest at insertion).
        assert!(lru.get("k1").is_none());
        assert!(lru.get("k2").is_some());
        assert!(lru.get("k3").is_some());
    }
}
