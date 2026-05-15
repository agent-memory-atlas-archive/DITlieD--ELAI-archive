// B-29 Phase 3 — SymbolGraphCache: lazy index + LRU file payload cache.
//
// Design notes (Phase 3 decisions):
//
// Connection ownership: SymbolGraphCache stores db_path: PathBuf and reopens
// a read-only connection per shard-load batch.  This follows the established
// DkgQuerierAdapter pattern (dkg_adapter.rs:17-38) and avoids making the
// struct !Send (rusqlite::Connection is !Send + !Sync).
//
// Lazy indices: name_index and reverse_file_index are Option<BTreeMap<...>>.
// On first access, all shards are loaded sequentially (single-threaded).
// Parallel loading via std::thread::scope is documented as a future
// optimisation gated on a B-29.5 benchmark; it must NOT be added here
// without human confirmation because it would require changing the tokio
// production feature set or spinning raw std threads.  27 name shards and
// 256 reverse shards are each ~microseconds per SQLite read on SSD.
//
// LRU: hand-rolled using two BTreeMaps (not HashMap) to maintain deterministic
// iteration order (C2 BTREEMAP gotcha).  See LruCache internals below.
//
// Cache invalidation: invalidate_file resets BOTH index caches to None (full
// rebuild on next access).  Incremental patch helpers are B-31 scope.
// The INCREMENTAL_SYMBOL_THRESHOLD (50) belongs in the Phase 4
// orchestrator/indexing path, NOT in the cache itself.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use rusqlite::{Connection, OpenFlags};

use crate::KnowledgeError;

use super::sharding::all_name_shard_keys;
use super::store;
use super::types::{SymbolGraphFilePayload, SymbolGraphMeta, SymbolRef};

// ---------------------------------------------------------------------------
// Hand-rolled LRU cache
// ---------------------------------------------------------------------------
// Uses two BTreeMaps:
//   order:   BTreeMap<u64, String>   — access_counter -> key (eviction ordering)
//   entries: BTreeMap<String, (u64, Arc<SymbolGraphFilePayload>)>
//
// Both are BTreeMap (not FxHashMap) to maintain deterministic iteration order.
// FxHashMap is used only where order is truly irrelevant; here eviction order
// must be deterministic across test runs.
//
// On get(): find old counter via entries, remove from order map, reinsert
// with new counter.  Failure to update order map causes incorrect eviction.

struct LruCache {
    order: BTreeMap<u64, String>,
    entries: BTreeMap<String, (u64, Arc<SymbolGraphFilePayload>)>,
    capacity: usize,
    counter: u64,
    hits: u64,
    misses: u64,
}

impl LruCache {
    fn new(capacity: usize) -> Self {
        // capacity=0 is treated as unlimited (should not happen in practice but
        // we guard against divide-by-zero / off-by-one in eviction).
        Self {
            order: BTreeMap::new(),
            entries: BTreeMap::new(),
            capacity,
            counter: 0,
            hits: 0,
            misses: 0,
        }
    }

    /// Return the payload if present, bumping its access counter.
    fn get(&mut self, key: &str) -> Option<Arc<SymbolGraphFilePayload>> {
        let old_counter = match self.entries.get(key) {
            Some((c, _)) => *c,
            None => {
                self.misses += 1;
                return None;
            }
        };
        // Remove old ordering entry and reinsert with new counter.
        self.order.remove(&old_counter);
        self.counter += 1;
        let new_counter = self.counter;
        let arc = match self.entries.get_mut(key) {
            Some((ctr, arc)) => {
                *ctr = new_counter;
                Arc::clone(arc)
            }
            None => {
                // Defensive: invariant violated (would require concurrent mutation,
                // impossible behind &mut self). Treat as miss rather than panic.
                tracing::warn!("B29 cache: key {key} vanished between get and update");
                self.misses += 1;
                return None;
            }
        };
        self.order.insert(new_counter, key.to_string());
        self.hits += 1;
        Some(arc)
    }

    /// Insert a new entry.  If at capacity, evict the least-recently-used entry.
    fn insert(&mut self, key: String, val: Arc<SymbolGraphFilePayload>) {
        // If key already exists, update in place (move to front).
        if let Some((old_counter, _existing_arc)) = self.entries.get(&key) {
            let old_c = *old_counter;
            self.order.remove(&old_c);
            self.counter += 1;
            let new_c = self.counter;
            self.entries.insert(key.clone(), (new_c, Arc::clone(&val)));
            self.order.insert(new_c, key);
            return;
        }

        // Evict if at capacity.
        if self.capacity > 0 && self.entries.len() >= self.capacity {
            // Smallest counter = least recently used.
            if let Some((&evict_counter, evict_key)) = self.order.iter().next() {
                let evict_key = evict_key.clone();
                self.order.remove(&evict_counter);
                self.entries.remove(&evict_key);
            }
        }

        self.counter += 1;
        let c = self.counter;
        self.entries.insert(key.clone(), (c, val));
        self.order.insert(c, key);
    }

    /// Remove a single key from the cache.
    fn remove(&mut self, key: &str) {
        if let Some((old_counter, _)) = self.entries.remove(key) {
            self.order.remove(&old_counter);
        }
    }

    fn size(&self) -> usize {
        self.entries.len()
    }

    fn hits(&self) -> u64 {
        self.hits
    }

    fn misses(&self) -> u64 {
        self.misses
    }
}

// ---------------------------------------------------------------------------
// CacheStats
// ---------------------------------------------------------------------------

/// Read-only statistics snapshot for the cache (useful for diagnostics and
/// the dashboard).
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub lru_size: usize,
    pub lru_hits: u64,
    pub lru_misses: u64,
    pub name_index_loaded: bool,
    pub reverse_index_loaded: bool,
}

// ---------------------------------------------------------------------------
// SymbolGraphCache
// ---------------------------------------------------------------------------

/// In-process read cache for a built symbol graph.
///
/// Loads metadata eagerly on construction; loads name + reverse indices lazily
/// on first query; caches per-file payloads in a bounded LRU.
///
/// All methods are synchronous — the knowledge crate has tokio with
/// features=["sync"] only.
pub struct SymbolGraphCache {
    db_path: PathBuf,
    project_id: String,
    meta: SymbolGraphMeta,
    name_index: Option<BTreeMap<String, Vec<SymbolRef>>>,
    reverse_file_index: Option<BTreeMap<String, BTreeSet<String>>>,
    file_payload_lru: LruCache,
}

impl SymbolGraphCache {
    /// Open the symbol graph at `db_path` for `project_id`.
    ///
    /// Eagerly loads `symbol_graph_meta` (one DB open/close).
    /// Returns `Err` if the project is not present in the meta table.
    pub fn new(
        db_path: &Path,
        project_id: &str,
        lru_capacity: usize,
    ) -> Result<Self, KnowledgeError> {
        let conn = Connection::open_with_flags(
            db_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;

        let meta = store::load_meta(&conn, project_id)?
            .ok_or_else(|| {
                KnowledgeError::Graph(format!(
                    "no symbol_graph_meta row for project_id='{project_id}'"
                ))
            })?;

        Ok(Self {
            db_path: db_path.to_path_buf(),
            project_id: project_id.to_string(),
            meta,
            name_index: None,
            reverse_file_index: None,
            file_payload_lru: LruCache::new(lru_capacity),
        })
    }

    // -----------------------------------------------------------------------
    // Accessors
    // -----------------------------------------------------------------------

    /// Return a reference to the loaded metadata.
    pub fn meta(&self) -> &SymbolGraphMeta {
        &self.meta
    }

    /// Absolute path to the SQLite database.
    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    /// Project identifier this cache was opened for.
    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    /// Number of file payloads currently in the LRU.
    pub fn file_payload_lru_size(&self) -> usize {
        self.file_payload_lru.size()
    }

    /// Whether the name index has been loaded into memory.
    pub fn name_index_loaded(&self) -> bool {
        self.name_index.is_some()
    }

    /// Whether the reverse file index has been loaded into memory.
    pub fn reverse_index_loaded(&self) -> bool {
        self.reverse_file_index.is_some()
    }

    /// Aggregate statistics for diagnostics and dashboard.
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            lru_size: self.file_payload_lru.size(),
            lru_hits: self.file_payload_lru.hits(),
            lru_misses: self.file_payload_lru.misses(),
            name_index_loaded: self.name_index.is_some(),
            reverse_index_loaded: self.reverse_file_index.is_some(),
        }
    }

    // -----------------------------------------------------------------------
    // Lazy loaders
    // -----------------------------------------------------------------------

    /// Return a reference to the name index, loading all 27 shards if needed.
    ///
    /// Sequential shard loads are safe and fast on SSD (~1–5 ms total).
    /// Parallel loading via std::thread::scope is documented as a future
    /// optimisation pending B-29.5 benchmarks.
    pub fn ensure_name_index(
        &mut self,
    ) -> Result<&BTreeMap<String, Vec<SymbolRef>>, KnowledgeError> {
        if self.name_index.is_none() {
            let conn = Connection::open_with_flags(
                &self.db_path,
                OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
            )?;

            let mut merged: BTreeMap<String, Vec<SymbolRef>> = BTreeMap::new();
            for key in all_name_shard_keys() {
                let shard = store::load_name_shard(&conn, key)?;
                merged.extend(shard);
            }
            self.name_index = Some(merged);
        }
        match self.name_index.as_ref() {
            Some(idx) => Ok(idx),
            // Unreachable: we set it above in this branch. Return a typed error
            // rather than panic if the invariant is somehow violated.
            None => Err(KnowledgeError::Graph(
                "symbol_graph: name_index lazy-load failed silently".into(),
            )),
        }
    }

    /// Return a reference to the reverse file index, loading all 256 shards if
    /// needed.
    pub fn ensure_reverse_index(
        &mut self,
    ) -> Result<&BTreeMap<String, BTreeSet<String>>, KnowledgeError> {
        if self.reverse_file_index.is_none() {
            let conn = Connection::open_with_flags(
                &self.db_path,
                OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
            )?;

            let mut merged: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
            for bucket in 0u8..=255 {
                let shard = store::load_reverse_shard(&conn, bucket)?;
                for (callee, callers) in shard {
                    merged
                        .entry(callee)
                        .or_default()
                        .extend(callers);
                }
            }
            self.reverse_file_index = Some(merged);
        }
        match self.reverse_file_index.as_ref() {
            Some(idx) => Ok(idx),
            None => Err(KnowledgeError::Graph(
                "symbol_graph: reverse_file_index lazy-load failed silently".into(),
            )),
        }
    }

    /// Return the payload for `file`, using the LRU cache or loading from DB.
    pub fn get_or_load_payload(
        &mut self,
        file: &str,
    ) -> Result<Option<Arc<SymbolGraphFilePayload>>, KnowledgeError> {
        if let Some(arc) = self.file_payload_lru.get(file) {
            return Ok(Some(arc));
        }

        // Cache miss — load from DB.
        let conn = Connection::open_with_flags(
            &self.db_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;

        let payload = store::load_file_payload(&conn, file)?;
        let Some(p) = payload else {
            return Ok(None);
        };

        let arc = Arc::new(p);
        self.file_payload_lru.insert(file.to_string(), Arc::clone(&arc));
        Ok(Some(arc))
    }

    // -----------------------------------------------------------------------
    // Cache invalidation
    // -----------------------------------------------------------------------

    /// Evict a single file from the LRU and reset both index caches to force
    /// a full rebuild on next access.
    ///
    /// Full rebuild trigger: when `changed_file_count > 50`
    /// (`INCREMENTAL_SYMBOL_THRESHOLD`), the Phase 4 orchestrator/indexing
    /// path is responsible for calling this method in a loop or rebuilding
    /// the entire graph.  The cache itself only implements single-file
    /// eviction; incremental patch helpers (`patchNameIndexForFile`,
    /// `patchReverseFileIndexForFile`) are B-31 scope.
    pub fn invalidate_file(&mut self, file: &str) {
        self.file_payload_lru.remove(file);
        // Reset both full-index caches to force rebuild on next access.
        // Partial-index patching is B-31 scope.
        self.name_index = None;
        self.reverse_file_index = None;
    }
}
