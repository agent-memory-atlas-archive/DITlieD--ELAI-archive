//! Tokio-driven decay scheduler per E-5 plan Task 9.
//!
//! Wraps [`crate::decay_tick`] in a background loop that ticks at a
//! configurable cadence (default: 1 hour). Designed to be spawned at
//! orchestrator/LiveCli startup and run for the process lifetime — the
//! returned [`DecayHandle`] holds an `abort()` signal so the caller can
//! cleanly shut it down at session end.
//!
//! Time is sourced from a `now_ms_fn` closure so tests can drive the
//! scheduler with a virtual clock. In production the caller passes a
//! closure backed by `ClockProvider::now_ms()` (per E-1 TIME-DOMAIN
//! gotcha — never use `.tick()` for cross-component timestamps).
//!
//! Per-tick observability is consolidated under
//! `SsbActionType::MemoryEvolutionRun` with `phase == "decay_tick"`
//! (E-5-decisions.md SCHEMA-LOCK: no new SsbActionType variant added).
//! The orchestrator caller emits one SSB row per tick from
//! [`DecayTickStats`].

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;

use crate::decay::{decay_tick, DecayTickStats};
use crate::store::FactStore;

/// Default cadence per the plan: one hour between decay passes.
pub const DEFAULT_DECAY_TICK_INTERVAL: Duration = Duration::from_secs(3600);

/// Minimum allowed cadence — prevents pathological tight loops if a
/// caller passes `Duration::ZERO`. Set to 1 second.
pub const MIN_DECAY_TICK_INTERVAL: Duration = Duration::from_secs(1);

/// Configuration shape for [`spawn_decay_scheduler`].
#[derive(Debug, Clone)]
pub struct DecaySchedulerConfig {
    /// How long to wait between decay ticks. Clamped to at least
    /// [`MIN_DECAY_TICK_INTERVAL`].
    pub interval: Duration,
    /// When true, the scheduler runs one decay pass IMMEDIATELY on
    /// spawn before waiting `interval` for the next pass. Useful for
    /// tests + first-boot catchup.
    pub run_immediately: bool,
}

impl Default for DecaySchedulerConfig {
    fn default() -> Self {
        DecaySchedulerConfig {
            interval: DEFAULT_DECAY_TICK_INTERVAL,
            run_immediately: false,
        }
    }
}

/// Handle returned by [`spawn_decay_scheduler`]. Drop or call
/// [`DecayHandle::abort`] to stop the background task.
#[derive(Debug)]
pub struct DecayHandle {
    handle: tokio::task::JoinHandle<()>,
}

impl DecayHandle {
    /// Abort the background scheduler. Idempotent.
    pub fn abort(&self) {
        self.handle.abort();
    }

    /// True iff the underlying task has completed (cleanly or via abort).
    pub fn is_finished(&self) -> bool {
        self.handle.is_finished()
    }
}

impl Drop for DecayHandle {
    fn drop(&mut self) {
        // Best-effort: abort the task on drop so a session shutdown does
        // not leak the loop.
        self.handle.abort();
    }
}

/// Callback fired AFTER each successful tick, with the resulting stats.
/// Used by the orchestrator caller to emit one
/// `SsbActionType::MemoryEvolutionRun(phase="decay_tick")` row per pass.
/// Non-async + returns nothing — the caller owns its own runtime and
/// fire-and-forgets per the live_cli.rs pattern.
///
/// F-5 staleness-quarantine emit seam: this is also the hook where the
/// cross-subsystem cascade originates. The CLI/orchestrator boundary (the
/// only layer that depends on both `crates/memory` and `crates/knowledge`)
/// builds an `on_tick` closure that captures both the scheduler's
/// `Arc<Mutex<FactStore>>` and a shared
/// [`crate::quarantine_broadcast::QuarantineBroadcastQueue`]. After the tick,
/// that closure runs [`crate::quarantine_broadcast::collect_quarantined`]
/// against the store and pushes the resulting records onto the queue. Memory
/// itself stays knowledge-free — it only fills the queue; the CLI drains it
/// and applies the DKG / Cosmos effects (see live_cli.rs A4 wiring). The
/// observer signature intentionally carries only `&DecayTickStats`; the
/// store + queue handles are captured by the closure, not threaded through
/// this type, so no `crates/knowledge` dependency leaks into this crate.
pub type DecayTickObserver =
    Arc<dyn Fn(&DecayTickStats) + Send + Sync + 'static>;

/// Spawn the decay scheduler as a tokio background task.
///
/// The `store` is shared via `Arc<Mutex<FactStore>>` because rusqlite
/// connections are not Send-safe across .await points; the scheduler
/// holds the mutex for the duration of each tick. Production callers
/// pass a dedicated FactStore handle for the scheduler so chat-turn
/// readers and writers do not contend on this mutex.
///
/// `now_ms_fn` is invoked at the start of each tick to source the
/// timestamp. Production callers wrap `ClockProvider::now_ms()` here.
pub fn spawn_decay_scheduler(
    store: Arc<Mutex<FactStore>>,
    cfg: DecaySchedulerConfig,
    now_ms_fn: Arc<dyn Fn() -> u128 + Send + Sync + 'static>,
    on_tick: Option<DecayTickObserver>,
) -> DecayHandle {
    let interval = cfg.interval.max(MIN_DECAY_TICK_INTERVAL);
    let run_immediately = cfg.run_immediately;
    let handle = tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        // Default tokio behavior fires immediately on first poll —
        // suppress unless run_immediately is set.
        if !run_immediately {
            ticker.tick().await;
        }
        loop {
            ticker.tick().await;
            let now_ms = (now_ms_fn)();
            // Acquire the store lock and run one pass.
            let stats_res = {
                let guard = store.lock().await;
                // H-33: pass empty pin set — the scheduler does not read
                // SSB pin beacons (fail-toward-decay; live_cli path handles
                // the pin-aware path when ELAI_MEMORY_PIN_ENABLE is set).
                decay_tick(&guard, &std::collections::HashSet::new(), now_ms)
            };
            match stats_res {
                Ok(stats) => {
                    if let Some(observer) = on_tick.as_ref() {
                        (observer)(&stats);
                    } else {
                        tracing::debug!(
                            target = "memory.decay_scheduler",
                            rows_examined = stats.rows_examined,
                            rows_updated = stats.rows_updated,
                            mean_trust_score = stats.mean_trust_score,
                            "decay tick complete"
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        target = "memory.decay_scheduler",
                        error = %e,
                        "decay tick failed (non-fatal; will retry next interval)"
                    );
                }
            }
        }
    });
    DecayHandle { handle }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fact::{FactType, MemoryFact, Scope};
    use crate::trust::{Provenance, TrustTier};

    fn make_fact(t: u128) -> MemoryFact {
        MemoryFact {
            fact_id: MemoryFact::new_id(),
            user_id: "default".into(),
            project_id: Some("p1".into()),
            scope: Scope::Project,
            fact_type: FactType::Mistake,
            key: "k".into(),
            value: serde_json::json!({"v": 1}),
            confidence: 0.9,
            trust_score: 0.9,
            ingested_at_ms: t,
            valid_from_ms: t,
            valid_to_ms: None,
            decay_rate_per_day: 0.005,
            last_observed_at_ms: t,
            evidence_pointers: vec!["s".into(), "m".into()],
            superseded_by: None,
            trust_tier: TrustTier::T1CodebaseDerived,
            provenance: Provenance::codebase_derived("test"),
            last_validated_at_ms: None,
        }
    }

    #[tokio::test]
    async fn fires_observer_on_immediate_tick() {
        let store = FactStore::open_in_memory().expect("open");
        // Insert a row at t=0 so the tick has something to examine.
        store.insert(&make_fact(1_700_000_000_000)).expect("ins");
        let shared = Arc::new(Mutex::new(store));

        // Atomic counter of observed ticks.
        let count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let count_for_obs = count.clone();
        let observer: DecayTickObserver = Arc::new(move |_stats: &DecayTickStats| {
            count_for_obs.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        });

        // Virtual clock: advance ~7 days per call so decay produces a delta.
        let calls = Arc::new(std::sync::atomic::AtomicU64::new(0));
        let calls_for_clock = calls.clone();
        let now_fn: Arc<dyn Fn() -> u128 + Send + Sync> = Arc::new(move || {
            let n = calls_for_clock.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            1_700_000_000_000u128 + (n as u128) * 7 * 86_400_000
        });

        let handle = spawn_decay_scheduler(
            shared,
            DecaySchedulerConfig {
                interval: Duration::from_millis(50),
                run_immediately: true,
            },
            now_fn,
            Some(observer),
        );

        // Wait long enough for at least 2 ticks.
        tokio::time::sleep(Duration::from_millis(200)).await;
        handle.abort();
        let observed = count.load(std::sync::atomic::Ordering::SeqCst);
        assert!(observed >= 1, "expected at least one observer call, got {observed}");
    }

    #[test]
    fn clamps_interval_below_minimum() {
        // Smoke-check: pure config sanity; no tokio runtime needed.
        let cfg = DecaySchedulerConfig {
            interval: Duration::ZERO,
            run_immediately: false,
        };
        let clamped = cfg.interval.max(MIN_DECAY_TICK_INTERVAL);
        assert_eq!(clamped, MIN_DECAY_TICK_INTERVAL);
    }
}
