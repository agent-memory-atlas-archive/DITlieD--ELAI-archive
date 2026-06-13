// I-8: DeadEndCache — bounded in-session dead-end fingerprint store.
//
// Prevents isomorphic re-attempts on impossible tasks by caching the
// blake3 fingerprint of each aborted trajectory's (task, approach) bytes.
// A second run whose approach hash matches any cached entry is pruned BEFORE
// any model call — zero tokens on repeats (ID-1 doctrine).
//
// Implementation: stdlib VecDeque<String> + HashSet<String> combo.  O(1)
// lookup, O(capacity) worst-case eviction when full.  No external LRU crate
// dependency (anti-slopsquat HARD RULE; lru crate presence in Cargo.lock
// unverified at I-8 authoring time).
//
// blake3: re-uses the existing blake3_hex() from crate::architecture::timetravel
// (timetravel.rs:203; pub fn confirmed live).  Never sha2/blake2 (CLAUDE.md
// Anti-Stale-Training HARD RULE).
//
// I-17 seam: the check_halting_bound() hook is the injection point for the
// empirical 15-second watchdog.  Wave-3's I-17 fills it; in Wave-1 the config
// field is present and callable but the timer spawn is a registered stub.
//
// Constraints:
//   - No .unwrap()/.expect() in production paths.
//   - Named constants only (no magic numbers).
//   - No new crate dependencies.

use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use std::time::Duration;

use elai_core::clock::{ClockProvider, OsClock};
use tokio::task::JoinHandle;

use crate::architecture::timetravel::blake3_hex;
use crate::ssb::{Ssb, SsbPayload, TrajectoryAbortedKind, TrajectoryAbortedPayload};

// ---------------------------------------------------------------------------
// Named constants
// ---------------------------------------------------------------------------

/// Default maximum number of dead-end entries retained in the in-session cache.
///
/// 256 entries covers typical nested-retry scenarios without material heap
/// pressure (each entry is a 64-byte hex string ≈ 256 × 64 = ~16 KiB).
pub const DEFAULT_DEAD_END_CACHE_CAPACITY: usize = 256;

/// Default halting bound in milliseconds supplied to [`DeadEndCache::new_with_halting_bound`].
///
/// The empirical 15 s value comes from I-17's specification.  It MUST be
/// a named constant — never a hard-coded literal — so I-17 can substitute
/// the real watchdog config without changing call-sites.
///
/// # TODO[I-17]
/// `DEFAULT_HALTING_BOUND_MS_FALLBACK` is the fallback used when no
/// [`TaskRunnerConfig::halting_bound_ms`] is set.  The I-17 check-runner
/// watchdog reads this value from config and spawns the actual timer;
/// the Wave-1 stub below only exposes the config path and the SSB emission
/// hook.  Replace this stub when I-17 ships.
pub const DEFAULT_HALTING_BOUND_MS_FALLBACK: u64 = 15_000;

/// Agent id stamped on the `TrajectoryAborted(HaltingBound)` SSB row the I-17
/// watchdog emits on breach. A named constant so the wire string is greppable
/// and the dashboard can attribute halting-bound aborts to the watchdog.
pub const HALTING_BOUND_AGENT_ID: &str = "halting_bound_watchdog";

// ---------------------------------------------------------------------------
// DeadEndCache
// ---------------------------------------------------------------------------

/// In-session bounded cache of dead-end trajectory fingerprints.
///
/// Each entry is the blake3 hex digest of the normalised
/// `(task_bytes, approach_bytes)` pair.  When [`check_and_insert`] finds a
/// collision the caller must prune the branch immediately without dispatching
/// to the model (ID-1 doctrine: aborted paths are hard-excluded).
///
/// # Thread safety
/// `DeadEndCache` is `Send` but NOT `Sync` (interior mutability via
/// `VecDeque`/`HashSet`, no `Mutex`).  Each task runner owns its own instance;
/// cross-task sharing requires wrapping in `Arc<Mutex<_>>`.
pub struct DeadEndCache {
    capacity: usize,
    /// Ordered queue for eviction (oldest entry at front).
    queue: VecDeque<String>,
    /// Fast-lookup set mirroring `queue`.
    set: HashSet<String>,
    /// Optional halting bound milliseconds (I-17 watchdog seam).
    /// `None` means the watchdog is disabled.
    halting_bound_ms: Option<u64>,
    /// Optional SSB writer for the I-17 halting-bound watchdog (FORK-A b-refined).
    /// `None` (the contract-test path + any context with no SSB wired) means
    /// [`check_halting_bound`](Self::check_halting_bound) spawns NO watchdog and
    /// returns `None` — keeping the seam an honest no-op AND keeping the i8_f4
    /// contract test runnable as a plain `#[test]` (a `tokio::spawn` with no
    /// reactor would panic). Set in production via
    /// [`with_ssb_writer`](Self::with_ssb_writer).
    ssb_writer: Option<Arc<Ssb>>,
}

impl DeadEndCache {
    /// Create a new cache with the given capacity and no halting bound.
    ///
    /// Use [`DEFAULT_DEAD_END_CACHE_CAPACITY`] for the production default.
    pub fn new(capacity: usize) -> Self {
        DeadEndCache {
            capacity,
            queue: VecDeque::with_capacity(capacity),
            set: HashSet::with_capacity(capacity),
            halting_bound_ms: None,
            ssb_writer: None,
        }
    }

    /// Create a new cache with an explicit halting bound.
    ///
    /// The bound is forwarded to [`check_halting_bound`](Self::check_halting_bound),
    /// which spawns the I-17 watchdog ONLY once an SSB writer is also installed
    /// via [`with_ssb_writer`](Self::with_ssb_writer). Without a writer the bound
    /// is stored but the watchdog stays dark (honest no-op).
    pub fn new_with_halting_bound(capacity: usize, halting_bound_ms: u64) -> Self {
        DeadEndCache {
            capacity,
            queue: VecDeque::with_capacity(capacity),
            set: HashSet::with_capacity(capacity),
            halting_bound_ms: Some(halting_bound_ms),
            ssb_writer: None,
        }
    }

    /// Install the SSB writer the I-17 halting-bound watchdog emits through
    /// (FORK-A b-refined builder). Consuming-self so the production call site
    /// reads `DeadEndCache::new_with_halting_bound(cap, ms).with_ssb_writer(ssb)`.
    /// With both a `halting_bound_ms` AND a writer present,
    /// [`check_halting_bound`](Self::check_halting_bound) spawns a real watchdog;
    /// with either absent it stays an honest no-op.
    #[must_use]
    pub fn with_ssb_writer(mut self, writer: Arc<Ssb>) -> Self {
        self.ssb_writer = Some(writer);
        self
    }

    /// Compute the blake3 fingerprint for a (task, approach) pair.
    ///
    /// Both slices are concatenated with a length-prefixed delimiter to avoid
    /// collisions when one slice is a prefix of another.
    pub fn fingerprint(task_bytes: &[u8], approach_bytes: &[u8]) -> String {
        // Length-prefixed concatenation: [4-byte task-len LE][task][approach]
        let task_len = (task_bytes.len() as u32).to_le_bytes();
        let mut buf = Vec::with_capacity(4 + task_bytes.len() + approach_bytes.len());
        buf.extend_from_slice(&task_len);
        buf.extend_from_slice(task_bytes);
        buf.extend_from_slice(approach_bytes);
        blake3_hex(&buf)
    }

    /// Check whether `approach_hash` is a known dead-end, and insert it if not.
    ///
    /// Returns `true` if the hash was ALREADY present (caller must prune
    /// the branch — zero model calls).  Returns `false` if it was new (the hash
    /// is now recorded so any future isomorphic re-attempt will be pruned).
    ///
    /// When at capacity the oldest entry is evicted before insertion.
    pub fn check_and_insert(&mut self, approach_hash: &str) -> bool {
        if self.set.contains(approach_hash) {
            return true; // dead-end collision — prune immediately
        }
        // Evict oldest entry if at capacity.
        if self.queue.len() >= self.capacity {
            if let Some(evicted) = self.queue.pop_front() {
                self.set.remove(&evicted);
            }
        }
        self.queue.push_back(approach_hash.to_owned());
        self.set.insert(approach_hash.to_owned());
        false
    }

    /// Number of entries currently in the cache.
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// True when the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// The configured capacity (maximum entries before oldest is evicted).
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// The halting bound milliseconds installed at construction time.
    ///
    /// Returns `None` in Wave-1 (watchdog not yet wired).  I-17 reads this
    /// value from [`TaskRunnerConfig::halting_bound_ms`] and passes it here.
    pub fn halting_bound_ms(&self) -> Option<u64> {
        self.halting_bound_ms
    }

    /// I-17 halting-bound watchdog — the empirical 15 s check-runner timer.
    ///
    /// Spawns a detached `tokio::time::sleep(halting_bound_ms)` task that, on
    /// breach, emits a [`TrajectoryAbortedKind::HaltingBound`] SSB row (a
    /// deterministic check that spins past the bound aborts the branch and is
    /// charged a HaltingBound penalty). Returns the watchdog's [`JoinHandle`] so
    /// the caller can `.abort()` it on clean completion (cancel-on-success); the
    /// caller dropping the handle leaves the timer to fire — by design, since a
    /// dropped guard means the bounded operation never reported completion.
    ///
    /// The watchdog spawns ONLY when BOTH a `halting_bound_ms` is configured AND
    /// an `ssb_writer` is installed (via [`with_ssb_writer`](Self::with_ssb_writer)).
    /// Otherwise it is an honest no-op returning `None` — which also keeps the
    /// signature callable from a synchronous (non-`tokio`) context such as the
    /// i8_f4 contract test (a `tokio::spawn` with no reactor would panic).
    /// `&self`: signature unchanged from the Wave-1 seam so existing callers
    /// (incl. the i8_f4 contract test) compile without edits; the returned
    /// `Option` is discarded by callers that do not need to cancel.
    pub fn check_halting_bound(&self) -> Option<JoinHandle<()>> {
        let (ms, writer) = match (self.halting_bound_ms, self.ssb_writer.as_ref()) {
            (Some(ms), Some(writer)) => (ms, Arc::clone(writer)),
            _ => {
                // No bound and/or no SSB writer: honest no-op (no timer spawned).
                tracing::debug!(
                    halting_bound_ms = ?self.halting_bound_ms,
                    has_writer = self.ssb_writer.is_some(),
                    "dead_end_cache: halting-bound watchdog dark (no bound or no SSB writer)"
                );
                return None;
            }
        };
        // Spawn the detached watchdog. On breach it emits one HaltingBound row.
        let handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(ms)).await;
            let payload = SsbPayload::TrajectoryAborted(TrajectoryAbortedPayload {
                kind: TrajectoryAbortedKind::HaltingBound,
                session_id: writer.session_id().to_owned(),
                trajectory_id: None,
                approach_hash: None,
                abort_reason_text: Some(format!(
                    "halting bound exceeded ({ms} ms): deterministic check overran the empirical bound"
                )),
                ts_ms: i64::try_from(OsClock.now_ms()).unwrap_or(i64::MAX),
            });
            if let Err(e) = writer
                .write_payload(HALTING_BOUND_AGENT_ID, 0, payload, Vec::new())
                .await
            {
                tracing::warn!(
                    error = %e,
                    "dead_end_cache: halting-bound watchdog SSB emit failed"
                );
            }
        });
        Some(handle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_and_insert_returns_false_on_first_insert() {
        let mut cache = DeadEndCache::new(DEFAULT_DEAD_END_CACHE_CAPACITY);
        let hash = DeadEndCache::fingerprint(b"task-a", b"approach-1");
        assert!(!cache.check_and_insert(&hash), "first insert must return false");
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn check_and_insert_returns_true_on_duplicate() {
        let mut cache = DeadEndCache::new(DEFAULT_DEAD_END_CACHE_CAPACITY);
        let hash = DeadEndCache::fingerprint(b"task-b", b"approach-2");
        assert!(!cache.check_and_insert(&hash));
        assert!(cache.check_and_insert(&hash), "second insert must return true (duplicate)");
        assert_eq!(cache.len(), 1, "duplicate must not grow cache");
    }

    #[test]
    fn eviction_at_capacity() {
        let mut cache = DeadEndCache::new(2);
        let h1 = DeadEndCache::fingerprint(b"t", b"a1");
        let h2 = DeadEndCache::fingerprint(b"t", b"a2");
        let h3 = DeadEndCache::fingerprint(b"t", b"a3");
        assert!(!cache.check_and_insert(&h1));
        assert!(!cache.check_and_insert(&h2));
        // Capacity=2: inserting h3 evicts h1.
        assert!(!cache.check_and_insert(&h3));
        assert_eq!(cache.len(), 2);
        // h1 was evicted — inserting it again is NOT a duplicate.
        assert!(!cache.check_and_insert(&h1), "evicted entry must re-insert as new");
    }

    #[test]
    fn fingerprint_is_deterministic() {
        let h1 = DeadEndCache::fingerprint(b"task", b"approach");
        let h2 = DeadEndCache::fingerprint(b"task", b"approach");
        assert_eq!(h1, h2, "fingerprint must be deterministic");
    }

    #[test]
    fn fingerprint_differs_for_different_inputs() {
        let h_ab = DeadEndCache::fingerprint(b"task", b"approach");
        let h_ba = DeadEndCache::fingerprint(b"approach", b"task");
        let h_diff = DeadEndCache::fingerprint(b"task", b"different");
        assert_ne!(h_ab, h_ba, "order matters (length-prefix prevents prefix collision)");
        assert_ne!(h_ab, h_diff, "different approach must differ");
    }

    #[test]
    fn halting_bound_seam_stores_configured_value() {
        let cache = DeadEndCache::new_with_halting_bound(256, DEFAULT_HALTING_BOUND_MS_FALLBACK);
        assert_eq!(
            cache.halting_bound_ms(),
            Some(DEFAULT_HALTING_BOUND_MS_FALLBACK),
            "halting_bound_ms must match construction value"
        );
    }

    #[test]
    fn no_halting_bound_by_default() {
        let cache = DeadEndCache::new(DEFAULT_DEAD_END_CACHE_CAPACITY);
        assert_eq!(cache.halting_bound_ms(), None, "default cache has no halting bound");
    }

    #[test]
    fn check_halting_bound_no_writer_is_honest_noop() {
        // I-17 FORK-A: with a bound but NO SSB writer, the watchdog must NOT
        // spawn (returns None) so the call is safe from a synchronous context
        // (no tokio reactor) — this is exactly the i8_f4 contract-test path.
        let cache =
            DeadEndCache::new_with_halting_bound(DEFAULT_DEAD_END_CACHE_CAPACITY, DEFAULT_HALTING_BOUND_MS_FALLBACK);
        assert!(
            cache.check_halting_bound().is_none(),
            "no SSB writer -> no watchdog spawned (honest no-op, sync-safe)"
        );
    }

    #[test]
    fn check_halting_bound_no_bound_is_honest_noop() {
        // No bound configured at all -> no watchdog regardless of writer state.
        let cache = DeadEndCache::new(DEFAULT_DEAD_END_CACHE_CAPACITY);
        assert!(
            cache.check_halting_bound().is_none(),
            "no halting bound -> no watchdog spawned"
        );
    }
}
