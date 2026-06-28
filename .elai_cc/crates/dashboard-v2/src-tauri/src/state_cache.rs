//! B-36 Session 2 STEP 3: focused cache-handle module for AppState.
//!
//! Split out of `state.rs` so the parent file stays under the MR-1
//! 1000-line maintainability boundary. Holds the `Arc<Mutex<ResearchCache>>`
//! handle required by the WIRING_CONTRACT ("AppState struct with Arc<...>
//! handles to orchestrator + cli + cache") and by Track A.6 / D.7
//! (run_research one-shot cache lookup).
//!
//! `ResearchCache` is the orchestrator's LRU cache for research results
//! (`elai_orchestrator::research_tree::ResearchCache`). The dashboard
//! constructs one at startup with a conservative capacity (64 entries,
//! 30-minute TTL) and shares it across Tauri command handlers via
//! `AppState::cache`. The `run_research` command (D.3) will peek/populate
//! it so repeated identical queries skip the sidecar round-trip.

use std::sync::{Arc, Mutex};

use elai_orchestrator::research_tree::ResearchCache;

/// Conservative default capacity for the dashboard-side research cache.
/// 64 entries is enough for a typical planning session's worth of distinct
/// research queries without unbounded memory growth.
const DEFAULT_MAX_ENTRIES: usize = 64;

/// 30-minute TTL. Research reports stale quickly (codebase changes); a
/// shorter TTL trades hit-rate for freshness. Tunable via env in a future
/// patch if measured hit-rate is too low.
const DEFAULT_TTL_SECONDS: u64 = 30 * 60;

/// Cache sub-state for `AppState`. Owned by-value inside `AppState` so
/// command handlers access it via `state.cache.research_cache_snapshot()`.
///
/// `std::sync::Mutex` is correct here: all Tauri command handlers that
/// touch the cache are sync (no guard held across `.await`). The
/// `ResearchCache` itself is not `Send` across await boundaries in any
/// current call site.
pub struct StateCache {
    /// LRU research-result cache. Shared across `run_research` invocations
    /// so a repeated query within the TTL window returns the cached
    /// `ResearchReport` without a sidecar round-trip.
    pub research_cache: Arc<Mutex<ResearchCache>>,
}

impl StateCache {
    /// Construct with default capacity + TTL. Called once from
    /// `AppState::new()`.
    pub fn new() -> Self {
        let cache = ResearchCache::new(DEFAULT_MAX_ENTRIES, DEFAULT_TTL_SECONDS);
        Self {
            research_cache: Arc::new(Mutex::new(cache)),
        }
    }

    /// Cheap Arc clone of the inner cache handle. Command handlers call
    /// this to get their own `Arc<Mutex<ResearchCache>>` without copying
    /// the cache contents.
    pub fn research_cache_snapshot(&self) -> Arc<Mutex<ResearchCache>> {
        Arc::clone(&self.research_cache)
    }
}

impl Default for StateCache {
    fn default() -> Self {
        Self::new()
    }
}
