//! C-30: Oracle answer-level cache.
//!
//! Caches successful `ResearchAnswer` responses keyed by
//! `(blake3(query), blake3(project_root))`. TTL is determined by the
//! `SourceTier` derived from the answer's `AnswerConfidence`:
//!
//! | SourceTier        | TTL                 |
//! |-------------------|---------------------|
//! | T1Authoritative   | 30 days             |
//! | T2Strong          |  7 days             |
//! | T3Weak            |  1 day              |
//! | T4Rejected        | never cache         |
//! | (AnswerConfidence::Unknown) | never cache (treated like T4) |
//!
//! LRU eviction fires at `max_entries`. Zero external deps beyond `blake3`
//! and `std` (blake3 already in workspace).
//!
//! Producer wiring lives in `crates/cli/src/live_cli.rs` `/research` arm.

// B-15 HashMap ban: FxHashMap chosen over BTreeMap for the LRU cache map.
// Access pattern is get/insert/remove/contains_key/len + iter for stale
// sweep. Per-query cache lookup is on the hot research path (called once
// per /research arm at live_cli.rs); non-cryptographic FxHasher saves
// hashing cost on the 32-byte blake3 CacheKey vs the SipHash default.
// Eviction-order non-determinism across runs is acceptable because LRU
// is itself an O(N) sweep that drops everything stale; no test asserts
// per-entry eviction order. rustc-hash already in workspace dependencies
// (Cargo.toml:46) and orchestrator Cargo.toml:27.
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rustc_hash::FxHashMap;

use blake3;
use elai_core::research_types::{AnswerConfidence, ResearchAnswer};
use elai_core::source_tier::SourceTier;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// TTL constants (seconds)
// ---------------------------------------------------------------------------

/// 30 days = 30 * 24 * 3600.
pub const TTL_T1_AUTHORITATIVE_SECS: u64 = 2_592_000;
/// 7 days = 7 * 24 * 3600.
pub const TTL_T2_STRONG_SECS: u64 = 604_800;
/// 1 day = 24 * 3600.
pub const TTL_T3_WEAK_SECS: u64 = 86_400;
/// T4 / Unknown — never cached.
pub const TTL_NEVER_CACHE_SECS: u64 = 0;

/// Default cap on cache entries. LRU eviction fires past this point.
pub const DEFAULT_MAX_ENTRIES: usize = 512;

// ---------------------------------------------------------------------------
// Key + value types
// ---------------------------------------------------------------------------

/// `(query_hash, project_hash)` blake3-32 pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CacheKey {
    pub query_hash: [u8; 32],
    pub project_hash: [u8; 32],
}

impl CacheKey {
    /// Compute the canonical CacheKey from `(query, project_root)`.
    /// `project_root` is canonicalised when possible — fallback uses the raw
    /// `Path` bytes so non-existent roots still hash deterministically.
    pub fn compute(query: &str, project_root: &Path) -> Self {
        let canonical = project_root.canonicalize().unwrap_or_else(|_| project_root.to_path_buf());
        let canonical_str = canonical.to_string_lossy();
        let query_hash = blake3_32(query.as_bytes());
        let project_hash = blake3_32(canonical_str.as_bytes());
        Self {
            query_hash,
            project_hash,
        }
    }

    /// Hex of `query_hash`, useful for SSB rows.
    pub fn query_hash_hex(&self) -> String {
        hex_lower(&self.query_hash)
    }
}

/// A cached ResearchAnswer plus its insertion timestamp + tier-derived TTL.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CachedAnswer {
    pub answer: ResearchAnswer,
    /// Unix seconds at insertion time.
    pub cached_at: u64,
    /// Tier-derived TTL classification used to choose `ttl_secs`.
    pub source_tier: SourceTier,
}

impl CachedAnswer {
    /// TTL in seconds for this entry's `source_tier`.
    pub fn ttl_secs(&self) -> u64 {
        ttl_for_tier(self.source_tier)
    }

    /// Whether the entry is still valid at `now_secs`.
    pub fn is_fresh(&self, now_secs: u64) -> bool {
        let ttl = self.ttl_secs();
        if ttl == 0 {
            return false;
        }
        let age = now_secs.saturating_sub(self.cached_at);
        age < ttl
    }

    /// Age of the entry in seconds at `now_secs`.
    pub fn age_secs(&self, now_secs: u64) -> u64 {
        now_secs.saturating_sub(self.cached_at)
    }
}

/// Map [`AnswerConfidence`] to the [`SourceTier`] used as a cache-TTL bucket.
/// - High   -> T1Authoritative (30d)
/// - Medium -> T2Strong (7d)
/// - Low    -> T3Weak (1d)
/// - Unknown -> T4Rejected (never cache; treated identically to T4 per
///   C-30 DRIFT-9).
pub fn source_tier_for_confidence(confidence: AnswerConfidence) -> SourceTier {
    match confidence {
        AnswerConfidence::High => SourceTier::T1Authoritative,
        AnswerConfidence::Medium => SourceTier::T2Strong,
        AnswerConfidence::Low => SourceTier::T3Weak,
        AnswerConfidence::Unknown => SourceTier::T4Rejected,
    }
}

/// TTL in seconds for a given [`SourceTier`].
pub fn ttl_for_tier(tier: SourceTier) -> u64 {
    match tier {
        SourceTier::T1Authoritative => TTL_T1_AUTHORITATIVE_SECS,
        SourceTier::T2Strong => TTL_T2_STRONG_SECS,
        SourceTier::T3Weak => TTL_T3_WEAK_SECS,
        SourceTier::T4Rejected => TTL_NEVER_CACHE_SECS,
    }
}

/// Lowercase identifier used in `OracleCacheHitPayload.source_tier`.
pub fn source_tier_str(tier: SourceTier) -> &'static str {
    match tier {
        SourceTier::T1Authoritative => "t1_authoritative",
        SourceTier::T2Strong => "t2_strong",
        SourceTier::T3Weak => "t3_weak",
        SourceTier::T4Rejected => "t4_rejected",
    }
}

/// Lowercase identifier for `AnswerConfidence` used in SSB rows.
pub fn confidence_str(c: AnswerConfidence) -> &'static str {
    match c {
        AnswerConfidence::High => "high",
        AnswerConfidence::Medium => "medium",
        AnswerConfidence::Low => "low",
        AnswerConfidence::Unknown => "unknown",
    }
}

// ---------------------------------------------------------------------------
// OracleAnswerCache
// ---------------------------------------------------------------------------

/// Thread-safe LRU cache keyed by `(query_hash, project_hash)`.
///
/// Internally guarded by `std::sync::Mutex`. The lock is *not* held across
/// any `.await` in the call sites — `get` / `insert` are synchronous methods.
#[derive(Debug)]
pub struct OracleAnswerCache {
    inner: Mutex<CacheInner>,
    /// H-8: when `Some`, the cache persists to disk so idempotent grounding
    /// answers survive a process restart. `None` = in-process only (the legacy
    /// `new()`/`with_capacity()` behaviour, used by tests). Set by `persisted()`.
    disk_path: Option<PathBuf>,
}

#[derive(Debug)]
struct CacheInner {
    /// Insertion-order vector of keys; the front is the oldest entry. We
    /// re-order on every successful `get` so the most-recently-accessed entry
    /// migrates to the back (LRU).
    order: Vec<CacheKey>,
    map: FxHashMap<CacheKey, CachedAnswer>,
    max_entries: usize,
}

/// H-8: on-disk form of the oracle answer cache. A `Vec` (not the `FxHashMap`)
/// because serde_json cannot use the struct `CacheKey` as a JSON object key; the
/// Vec preserves LRU order (oldest entry first).
#[derive(Debug, Serialize, Deserialize, Default)]
struct OracleCacheDisk {
    entries: Vec<(CacheKey, CachedAnswer)>,
}

/// H-8: resolve `$HOME/.elai/cache/oracle-cache/answers.json` (Windows:
/// `%USERPROFILE%`), mirroring the PromptCache disk layout (GD-1). Returns `None`
/// when the home dir is unresolvable or the cache dir cannot be created — the cache
/// then stays in-process only (graceful: no disk persistence).
fn oracle_cache_disk_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?;
    let dir = PathBuf::from(home)
        .join(".elai")
        .join("cache")
        .join("oracle-cache");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir.join("answers.json"))
}

/// H-8: best-effort load of the on-disk cache. A missing or corrupt file yields
/// `None` (empty cache), never a panic — mirrors PromptCache's corrupt->empty
/// discipline.
fn load_oracle_cache_disk(path: &Path) -> Option<OracleCacheDisk> {
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

impl OracleAnswerCache {
    /// Construct an empty cache with the given capacity.
    pub fn with_capacity(max_entries: usize) -> Self {
        Self {
            inner: Mutex::new(CacheInner {
                order: Vec::with_capacity(max_entries),
                map: FxHashMap::with_capacity_and_hasher(max_entries, Default::default()),
                max_entries: max_entries.max(1),
            }),
            disk_path: None,
        }
    }

    /// Default-capacity constructor (512 entries).
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_MAX_ENTRIES)
    }

    /// H-8: disk-backed constructor. Loads any persisted idempotent-grounding
    /// answers from `$HOME/.elai/cache/oracle-cache/answers.json` (best-effort — a
    /// missing/corrupt file yields an empty cache, never a panic) and persists on
    /// every successful insert. CONFINEMENT: only the `/research|/ask` grounding arm
    /// constructs this and it is the cache's single insert caller, so durable
    /// caching stays confined to idempotent grounding answers (GD-5 staleness
    /// guard). Per-tier TTL is still enforced on `get`, so stale on-disk entries are
    /// dropped on read.
    #[must_use]
    pub fn persisted(max_entries: usize) -> Self {
        let cap = max_entries.max(1);
        let disk_path = oracle_cache_disk_path();
        let mut cache_inner = CacheInner {
            order: Vec::with_capacity(cap),
            map: FxHashMap::with_capacity_and_hasher(cap, Default::default()),
            max_entries: cap,
        };
        if let Some(path) = disk_path.as_ref() {
            if let Some(disk) = load_oracle_cache_disk(path) {
                for (key, answer) in disk.entries {
                    if cache_inner.order.len() >= cap {
                        break;
                    }
                    if cache_inner.map.insert(key, answer).is_none() {
                        cache_inner.order.push(key);
                    }
                }
            }
        }
        Self {
            inner: Mutex::new(cache_inner),
            disk_path,
        }
    }

    /// H-8: default-capacity disk-backed constructor (512 entries).
    #[must_use]
    pub fn persisted_default() -> Self {
        Self::persisted(DEFAULT_MAX_ENTRIES)
    }

    /// H-8: serialize the current cache (LRU order) to `disk_path`. No-op when the
    /// cache is in-process (`disk_path` is `None`). Best-effort — a serialize or
    /// write error is logged, never propagated (persistence must never break the
    /// grounding path).
    fn persist_to_disk(&self) {
        let Some(path) = self.disk_path.as_ref() else {
            return;
        };
        let snapshot = {
            let Ok(inner) = self.inner.lock() else {
                return;
            };
            OracleCacheDisk {
                entries: inner
                    .order
                    .iter()
                    .filter_map(|k| inner.map.get(k).map(|v| (*k, v.clone())))
                    .collect(),
            }
        };
        match serde_json::to_vec(&snapshot) {
            Ok(bytes) => {
                if let Err(e) = std::fs::write(path, bytes) {
                    tracing::warn!(error = %e, "oracle_cache: disk persist write failed (non-fatal)");
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "oracle_cache: disk persist serialize failed (non-fatal)");
            }
        }
    }

    /// Number of entries currently cached.
    pub fn len(&self) -> usize {
        match self.inner.lock() {
            Ok(g) => g.map.len(),
            Err(p) => p.into_inner().map.len(),
        }
    }

    /// Whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Look up an entry. Returns a fresh-cloned `CachedAnswer` only when the
    /// entry exists AND has not exceeded its tier-derived TTL at `now_secs`.
    /// Expired entries are silently removed.
    pub fn get(&self, key: &CacheKey, now_secs: u64) -> Option<CachedAnswer> {
        let mut inner = self.inner.lock().ok().or_else(|| {
            // Poisoned mutex — try to recover the inner state and continue.
            None
        })?;
        let cached = inner.map.get(key)?.clone();
        if !cached.is_fresh(now_secs) {
            // Drop expired entry, then return None.
            inner.map.remove(key);
            inner.order.retain(|k| k != key);
            return None;
        }
        // LRU bump — move key to the end of the order vec.
        if let Some(pos) = inner.order.iter().position(|k| k == key) {
            let k = inner.order.remove(pos);
            inner.order.push(k);
        }
        Some(cached)
    }

    /// Insert a new answer. T4Rejected / never-cache entries are silently
    /// dropped (return `false`); successful inserts return `true`.
    /// Eviction fires when `len > max_entries` (LRU).
    pub fn insert(&self, key: CacheKey, entry: CachedAnswer) -> bool {
        // Never-cache tiers are dropped at the door.
        if entry.ttl_secs() == 0 {
            return false;
        }
        {
            let Ok(mut inner) = self.inner.lock() else {
                return false;
            };
            let max = inner.max_entries;
            if !inner.map.contains_key(&key) {
                inner.order.push(key);
            } else if let Some(pos) = inner.order.iter().position(|k| k == &key) {
                let k = inner.order.remove(pos);
                inner.order.push(k);
            }
            inner.map.insert(key, entry);
            // LRU eviction (oldest first).
            while inner.map.len() > max {
                let Some(victim) = inner.order.first().copied() else {
                    break;
                };
                inner.order.remove(0);
                inner.map.remove(&victim);
            }
        }
        // H-8: persist to disk after a successful insert (no-op when in-process;
        // lock released above — persist_to_disk re-locks briefly for the snapshot).
        self.persist_to_disk();
        true
    }

    /// Remove every entry whose TTL has elapsed at `now_secs`. Returns the
    /// number of entries dropped.
    pub fn evict_expired(&self, now_secs: u64) -> usize {
        let Ok(mut inner) = self.inner.lock() else {
            return 0;
        };
        let stale: Vec<CacheKey> = inner
            .map
            .iter()
            .filter_map(|(k, v)| if !v.is_fresh(now_secs) { Some(*k) } else { None })
            .collect();
        let n = stale.len();
        for k in stale {
            inner.map.remove(&k);
            inner.order.retain(|o| o != &k);
        }
        n
    }
}

impl Default for OracleAnswerCache {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn blake3_32(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(bytes);
    let hash = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(hash.as_bytes());
    out
}

fn hex_lower(bytes: &[u8; 32]) -> String {
    let mut out = String::with_capacity(64);
    for byte in bytes {
        // NARROWING-OK: explicit hex conversion via const lookup table.
        const HEX: &[u8; 16] = b"0123456789abcdef";
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use elai_core::research_types::ResearchAnswer;
    use std::path::PathBuf;

    fn mk_answer(confidence: AnswerConfidence) -> ResearchAnswer {
        ResearchAnswer {
            question: "q?".into(),
            answer_text: "a.".into(),
            evidence_chain: Vec::new(),
            confidence,
            caveats: Vec::new(),
            unanswered_parts: Vec::new(),
            sources_consulted: Vec::new(),
            sources_rejected: Vec::new(),
            path: "llm".into(),
        }
    }

    fn mk_cached(now: u64, tier: SourceTier, confidence: AnswerConfidence) -> CachedAnswer {
        CachedAnswer {
            answer: mk_answer(confidence),
            cached_at: now,
            source_tier: tier,
        }
    }

    fn mk_key(query: &str) -> CacheKey {
        CacheKey::compute(query, &PathBuf::from("/tmp/elai-oracle-test"))
    }

    #[test]
    fn ttl_tier_mapping_matches_spec() {
        assert_eq!(ttl_for_tier(SourceTier::T1Authoritative), 30 * 24 * 3600);
        assert_eq!(ttl_for_tier(SourceTier::T2Strong), 7 * 24 * 3600);
        assert_eq!(ttl_for_tier(SourceTier::T3Weak), 24 * 3600);
        assert_eq!(ttl_for_tier(SourceTier::T4Rejected), 0);
    }

    #[test]
    fn confidence_to_tier_mapping() {
        assert_eq!(
            source_tier_for_confidence(AnswerConfidence::High),
            SourceTier::T1Authoritative
        );
        assert_eq!(
            source_tier_for_confidence(AnswerConfidence::Medium),
            SourceTier::T2Strong
        );
        assert_eq!(
            source_tier_for_confidence(AnswerConfidence::Low),
            SourceTier::T3Weak
        );
        assert_eq!(
            source_tier_for_confidence(AnswerConfidence::Unknown),
            SourceTier::T4Rejected
        );
    }

    #[test]
    fn cache_miss_then_hit_within_ttl() {
        let cache = OracleAnswerCache::new();
        let key = mk_key("what is rust?");
        let now = 1_700_000_000;
        assert!(cache.get(&key, now).is_none());
        let entry = mk_cached(now, SourceTier::T1Authoritative, AnswerConfidence::High);
        assert!(cache.insert(key, entry));
        let got = cache.get(&key, now + 100).expect("hit");
        assert_eq!(got.source_tier, SourceTier::T1Authoritative);
        assert_eq!(got.age_secs(now + 100), 100);
    }

    #[test]
    fn t1_entry_survives_29d_expires_at_30d_plus_one() {
        let cache = OracleAnswerCache::new();
        let key = mk_key("q1");
        let cached_at = 1_700_000_000u64;
        let entry = mk_cached(cached_at, SourceTier::T1Authoritative, AnswerConfidence::High);
        cache.insert(key, entry);
        let day = 24u64 * 3600;
        assert!(cache.get(&key, cached_at + 29 * day).is_some());
        // 30d + 1s -> expired.
        assert!(cache.get(&key, cached_at + 30 * day + 1).is_none());
    }

    #[test]
    fn t3_entry_expires_after_86401_secs() {
        let cache = OracleAnswerCache::new();
        let key = mk_key("q3");
        let cached_at = 1_700_000_000u64;
        let entry = mk_cached(cached_at, SourceTier::T3Weak, AnswerConfidence::Low);
        cache.insert(key, entry);
        assert!(cache.get(&key, cached_at + 86_399).is_some());
        assert!(cache.get(&key, cached_at + 86_401).is_none());
    }

    #[test]
    fn t4_never_cached() {
        let cache = OracleAnswerCache::new();
        let key = mk_key("q4");
        let cached_at = 1_700_000_000u64;
        let entry = mk_cached(cached_at, SourceTier::T4Rejected, AnswerConfidence::Unknown);
        assert!(!cache.insert(key, entry));
        assert!(cache.get(&key, cached_at).is_none());
    }

    #[test]
    fn unknown_confidence_never_cached_via_tier_map() {
        let cache = OracleAnswerCache::new();
        let key = mk_key("q-unknown");
        let cached_at = 1_700_000_000u64;
        let tier = source_tier_for_confidence(AnswerConfidence::Unknown);
        let entry = mk_cached(cached_at, tier, AnswerConfidence::Unknown);
        assert!(!cache.insert(key, entry));
        assert!(cache.get(&key, cached_at).is_none());
    }

    #[test]
    fn lru_eviction_at_max() {
        let cache = OracleAnswerCache::with_capacity(2);
        let now = 1_700_000_000u64;
        let k1 = mk_key("a");
        let k2 = mk_key("b");
        let k3 = mk_key("c");
        cache.insert(k1, mk_cached(now, SourceTier::T1Authoritative, AnswerConfidence::High));
        cache.insert(k2, mk_cached(now, SourceTier::T1Authoritative, AnswerConfidence::High));
        // Touch k1 so k2 is LRU.
        let _ = cache.get(&k1, now + 1);
        cache.insert(k3, mk_cached(now, SourceTier::T1Authoritative, AnswerConfidence::High));
        // k2 should be evicted (oldest after the touch).
        assert!(cache.get(&k1, now + 2).is_some());
        assert!(cache.get(&k2, now + 2).is_none());
        assert!(cache.get(&k3, now + 2).is_some());
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn evict_expired_drops_only_stale_entries() {
        let cache = OracleAnswerCache::new();
        let now = 1_700_000_000u64;
        let k_t1 = mk_key("t1-fresh");
        let k_t3 = mk_key("t3-stale");
        cache.insert(
            k_t1,
            mk_cached(now, SourceTier::T1Authoritative, AnswerConfidence::High),
        );
        cache.insert(
            k_t3,
            mk_cached(now, SourceTier::T3Weak, AnswerConfidence::Low),
        );
        let later = now + 24 * 3600 + 1;
        let n_dropped = cache.evict_expired(later);
        assert_eq!(n_dropped, 1);
        assert!(cache.get(&k_t1, later).is_some());
        assert!(cache.get(&k_t3, later).is_none());
    }

    #[test]
    fn cache_key_deterministic() {
        let a = CacheKey::compute("foo", &PathBuf::from("/tmp/elai-x"));
        let b = CacheKey::compute("foo", &PathBuf::from("/tmp/elai-x"));
        assert_eq!(a, b);
        let c = CacheKey::compute("bar", &PathBuf::from("/tmp/elai-x"));
        assert_ne!(a, c);
        assert_eq!(a.query_hash_hex().len(), 64);
    }
}
