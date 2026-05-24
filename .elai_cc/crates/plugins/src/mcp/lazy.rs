//! C-35 (SCHEMA_VERSION 88): Lazy MCP Schema Cache.
//! WIRING_CONTRACT produces: LazySchemaCache.
//! stub_ban: cache_hit/cache_miss SSB rows are mandatory (no silent caching).
//! resolve MUST NOT panic; returns Result for unknown tool names.

// B-15 HashMap ban: FxHashMap chosen for the registry parameter.
// Lookup-only (registry.get(tool_name)); no iteration. rustc-hash is
// already in plugins/Cargo.toml. Test file tests/tool_gate.rs gets a
// cascade fix (changelog 2026-04-29 B-15B-FX rule: cfg(test) migrates
// atomically with prod).
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex as TokioMutex;
use lru::LruCache;
use rustc_hash::FxHashMap;
use thiserror::Error;

use crate::mcp_bridge::McpToolSpec;

pub type FullSchema = McpToolSpec;

#[derive(Debug, Error)]
pub enum LazyCacheError {
    #[error("tool {0:?} not found in registry")]
    ToolNotFound(String),
}

pub struct LazySchemaCache {
    inner: Arc<TokioMutex<LruCache<String, FullSchema>>>,
    max_size: usize,
    hits: Arc<AtomicU64>,
    misses: Arc<AtomicU64>,
    resolve_ns_total: Arc<AtomicU64>,
}

impl LazySchemaCache {
    pub fn new(max_size: usize) -> Self {
        let cap = std::num::NonZeroUsize::new(max_size.max(1)).expect("max_size > 0");
        Self {
            inner: Arc::new(TokioMutex::new(LruCache::new(cap))),
            max_size,
            hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
            resolve_ns_total: Arc::new(AtomicU64::new(0)),
        }
    }

    pub async fn resolve(
        &self,
        tool_name: &str,
        registry: &FxHashMap<String, FullSchema>,
    ) -> Result<FullSchema, LazyCacheError> {
        let start = Instant::now();
        {
            let mut cache = self.inner.lock().await;
            if let Some(schema) = cache.get(tool_name) {
                self.hits.fetch_add(1, Ordering::Relaxed);
                let schema = schema.clone();
                let elapsed_ns = start.elapsed().as_nanos() as u64;
                self.resolve_ns_total.fetch_add(elapsed_ns, Ordering::Relaxed);
                tracing::debug!(tool = tool_name, cache = "hit", "lazy schema cache hit");
                return Ok(schema);
            }
        }
        let schema = registry.get(tool_name)
            .ok_or_else(|| LazyCacheError::ToolNotFound(tool_name.to_owned()))?
            .clone();
        self.misses.fetch_add(1, Ordering::Relaxed);
        {
            let mut cache = self.inner.lock().await;
            cache.put(tool_name.to_owned(), schema.clone());
        }
        let elapsed_ns = start.elapsed().as_nanos() as u64;
        self.resolve_ns_total.fetch_add(elapsed_ns, Ordering::Relaxed);
        let elapsed_ms = elapsed_ns / 1_000_000;
        tracing::debug!(
            tool = tool_name,
            cache = "miss",
            elapsed_ms,
            "lazy schema cache miss: fetched from registry"
        );
        Ok(schema)
    }

    pub fn len(&self) -> usize {
        match self.inner.try_lock() {
            Ok(cache) => cache.len(),
            Err(_) => 0,
        }
    }

    pub fn is_empty(&self) -> bool { self.len() == 0 }
    pub fn max_size(&self) -> usize { self.max_size }
    pub fn cache_hits(&self) -> u64 { self.hits.load(Ordering::Relaxed) }
    pub fn cache_misses(&self) -> u64 { self.misses.load(Ordering::Relaxed) }
    pub fn resolve_ns_total(&self) -> u64 { self.resolve_ns_total.load(Ordering::Relaxed) }
}
