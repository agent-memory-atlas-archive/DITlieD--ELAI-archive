//! F-5 — cross-subsystem staleness-quarantine broadcast (memory side).
//!
//! The decay curve in [`crate::decay`] passively lowers each fact's
//! `trust_score` over time. F-5 adds a *propagation* layer on top: when a
//! fact has aged so far that its staleness ratio crosses
//! [`STALENESS_QUARANTINE_THRESHOLD`], the memory subsystem emits a
//! [`QuarantineRecord`] describing that fact. The records are buffered in a
//! bounded [`QuarantineBroadcastQueue`]; the CLI/orchestrator boundary (which
//! is the only layer that depends on BOTH `crates/memory` and
//! `crates/knowledge`) drains the queue and applies the downstream effects
//! (DKG `concept_stale = 2`, Cosmos artifact flagging). This module owns ONLY
//! the memory-side primitives — it has no knowledge of DKG or Cosmos and adds
//! no dependency on `crates/knowledge`.
//!
//! Staleness shape (reuses the decay half-life — no new lifetime column on
//! `memory_facts`):
//!
//! ```text
//! half_life_days   = ln(2) / decay_rate_per_day
//! age_days         = (now_ms - last_observed_at_ms) / 86_400_000
//! staleness_ratio  = age_days / half_life_days
//! quarantined      = staleness_ratio >= STALENESS_QUARANTINE_THRESHOLD
//! ```
//!
//! A `staleness_ratio` of 1.0 means exactly one half-life has elapsed (trust
//! decayed to ~0.5x). The default threshold of 1.5 means a fact is
//! quarantined once it has decayed roughly to ~0.35x of its observed trust —
//! late enough that the curve has clearly committed to staleness, early
//! enough that downstream consumers stop trusting it before it bottoms out.
//! The threshold is an ELAI-own design value, not an externally-sourced
//! target (see workflow/reasoning/F-5-decisions.md).

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use crate::error::MemoryError;
use crate::store::FactStore;

/// Milliseconds in one day. Mirrors [`crate::decay`]'s conversion constant.
const MS_PER_DAY: f64 = 86_400_000.0;

/// `ln(2)` — used to convert a per-day decay rate into a half-life in days
/// (`half_life_days = ln(2) / decay_rate_per_day`).
const LN_2: f64 = std::f64::consts::LN_2;

/// Staleness ratio at or above which a fact is quarantined and broadcast.
///
/// Expressed in half-lives: `1.5` means a fact is quarantined once ~1.5
/// half-lives of its fact-type-specific decay curve have elapsed (trust
/// decayed to roughly `0.5^1.5 ~ 0.35` of its last-observed value). ELAI-own
/// design value per F-5-decisions.md — not externally sourced.
pub const STALENESS_QUARANTINE_THRESHOLD: f64 = 1.5;

/// Maximum number of buffered [`QuarantineRecord`]s in a
/// [`QuarantineBroadcastQueue`]. On overflow the OLDEST record is dropped
/// (drop-oldest, so the freshest staleness signal always survives) and the
/// queue's `dropped_count` is incremented. ELAI-own design value.
pub const QUARANTINE_QUEUE_CAPACITY: usize = 256;

/// A single fact that has aged past [`STALENESS_QUARANTINE_THRESHOLD`].
///
/// Produced by [`collect_quarantined`], buffered in a
/// [`QuarantineBroadcastQueue`], and drained CLI-side to drive the
/// cross-subsystem cascade. Field shapes match the
/// `StalenessQuarantineBroadcast` SSB payload (`staleness_ratio: f64`,
/// `tick_ms: u64`).
#[derive(Debug, Clone, PartialEq)]
pub struct QuarantineRecord {
    /// Primary key of the quarantined fact (`memory_facts.fact_id`).
    pub fact_id: String,
    /// Computed `age_days / half_life_days` at the broadcasting tick. Always
    /// `>= STALENESS_QUARANTINE_THRESHOLD` for any record this module emits.
    pub staleness_ratio: f64,
    /// Wall-clock tick (ms) at which the quarantine was detected — the `now`
    /// passed to [`collect_quarantined`].
    pub tick_ms: u64,
}

/// Compute the staleness ratio (`age_days / half_life_days`) for a single
/// row. Returns `0.0` for immortal rows (`decay_rate_per_day <= 0`) and for
/// future-dated / identical timestamps (`now_ms <= last_observed_at_ms`) so
/// neither can ever cross the quarantine threshold.
fn staleness_ratio_for(
    decay_rate_per_day: f32,
    last_observed_at_ms: u128,
    now_ms: u128,
) -> f64 {
    if now_ms <= last_observed_at_ms {
        return 0.0;
    }
    let rate = decay_rate_per_day as f64;
    if rate <= 0.0 {
        // Immortal fact — half-life is infinite, ratio is effectively zero.
        return 0.0;
    }
    let half_life_days = LN_2 / rate;
    if half_life_days <= 0.0 {
        return 0.0;
    }
    let age_days = ((now_ms - last_observed_at_ms) as f64) / MS_PER_DAY;
    age_days / half_life_days
}

/// Scan every currently-valid fact in `store` and return one
/// [`QuarantineRecord`] per fact whose staleness ratio is at or above
/// [`STALENESS_QUARANTINE_THRESHOLD`] at `now`.
///
/// Age-only and immutable: this mirrors the read-only `decay_tick` call site
/// in `live_cli.rs` (the same `&FactStore` borrow). It does NOT mutate the
/// store, does NOT run the contradiction adjudicator, and does NOT touch
/// `last_observed_at_ms` — quarantine detection is a passive observation just
/// like decay. A separate contradiction sweep (if ever needed) lives
/// elsewhere; this function is pure age-based staleness.
///
/// `now` is the wall-clock tick in ms supplied by the caller's
/// `ClockProvider`; it is stored on each record as `tick_ms` and widened to
/// `u128` internally for the age subtraction against `last_observed_at_ms`.
pub fn collect_quarantined(
    store: &FactStore,
    now: u64,
) -> Result<Vec<QuarantineRecord>, MemoryError> {
    let now_ms = now as u128;
    let rows = store.iter_current()?;
    let mut out = Vec::new();
    for row in &rows {
        let ratio = staleness_ratio_for(
            row.decay_rate_per_day,
            row.last_observed_at_ms,
            now_ms,
        );
        if ratio >= STALENESS_QUARANTINE_THRESHOLD {
            out.push(QuarantineRecord {
                fact_id: row.fact_id.clone(),
                staleness_ratio: ratio,
                tick_ms: now,
            });
        }
    }
    Ok(out)
}

/// Bounded, thread-safe FIFO buffer of [`QuarantineRecord`]s shared between
/// the memory-side producer (a decay-tick observer) and the CLI-side
/// consumer (the cross-subsystem cascade drain).
///
/// Uses [`std::sync::Mutex`] (not `tokio::sync::Mutex`): the producer is a
/// non-async `DecayTickObserver` closure and the consumer drains
/// synchronously at the CLI boundary, so no lock is ever held across an
/// `.await`. On overflow the queue drops the OLDEST record (drop-oldest) —
/// the opposite of a `tokio::mpsc` bounded channel, which drops the newest /
/// blocks the sender. Drop-oldest is correct here: the freshest staleness
/// signal is the one downstream consumers most need (W-2: a dropped record is
/// surfaced via `dropped_count`, never silently lost).
#[derive(Debug, Default)]
pub struct QuarantineBroadcastQueue {
    inner: Mutex<VecDeque<QuarantineRecord>>,
    dropped_count: AtomicU64,
}

impl QuarantineBroadcastQueue {
    /// Construct an empty queue with capacity [`QUARANTINE_QUEUE_CAPACITY`].
    pub fn new() -> Self {
        QuarantineBroadcastQueue {
            inner: Mutex::new(VecDeque::with_capacity(QUARANTINE_QUEUE_CAPACITY)),
            dropped_count: AtomicU64::new(0),
        }
    }

    /// Push one record onto the back of the queue. If the queue is already at
    /// [`QUARANTINE_QUEUE_CAPACITY`], the OLDEST (front) record is evicted
    /// first and `dropped_count` is incremented by one.
    ///
    /// A poisoned mutex (a prior panic while holding the lock) is recovered
    /// via `into_inner` rather than propagated — losing a single broadcast is
    /// preferable to poisoning the whole decay observer, and the record's
    /// loss is still accounted for nowhere-silently because the queue keeps
    /// running. (Production code never panics while holding this lock; the
    /// recovery path exists only to keep the observer alive.)
    pub fn push(&self, record: QuarantineRecord) {
        let mut guard = match self.inner.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        if guard.len() >= QUARANTINE_QUEUE_CAPACITY {
            // Drop-oldest: evict the front, count it, then enqueue the new
            // record at the back so the newest signal always survives.
            guard.pop_front();
            self.dropped_count.fetch_add(1, Ordering::Relaxed);
        }
        guard.push_back(record);
    }

    /// Drain every buffered record (oldest-first) and return it alongside the
    /// number of records dropped since the last drain. The drop counter is
    /// reset to zero atomically as part of the drain so each drained batch
    /// reports only the drops that accumulated against it.
    pub fn drain(&self) -> (Vec<QuarantineRecord>, u64) {
        let drained: Vec<QuarantineRecord> = {
            let mut guard = match self.inner.lock() {
                Ok(g) => g,
                Err(poisoned) => poisoned.into_inner(),
            };
            guard.drain(..).collect()
        };
        let dropped = self.dropped_count.swap(0, Ordering::Relaxed);
        (drained, dropped)
    }

    /// Current number of buffered records. Primarily for tests / metrics.
    pub fn len(&self) -> usize {
        match self.inner.lock() {
            Ok(g) => g.len(),
            Err(poisoned) => poisoned.into_inner().len(),
        }
    }

    /// True iff no records are currently buffered.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Records dropped (drop-oldest evictions) since the last [`drain`], read
    /// without resetting. Use [`drain`] to read-and-reset together.
    ///
    /// [`drain`]: QuarantineBroadcastQueue::drain
    pub fn dropped_count(&self) -> u64 {
        self.dropped_count.load(Ordering::Relaxed)
    }
}
