//! E-39 -- free-tier daily-quota guard for Cerebras / Groq / DeepSeek.
//!
//! Each free-tier backend has a hard daily quota. Exceeding it either
//! charges the user (if billing is configured) or fails outright
//! (if not). The guard tracks per-backend daily usage and refuses
//! calls when >=95% of the daily quota is consumed. It warns at >=80%
//! by emitting a [`crate::ssb::CostEventKind::FreeTierQuotaApproaching`]
//! row.
//!
//! Reset occurs at the provider's local-time day boundary. Per plan
//! Open Question, Cerebras + Groq both use UTC for daily reset (this
//! is documented in the decision doc; verify on first live integration
//! and add per-backend timezone handling here if either provider
//! diverges).
//!
//! Source grounding (verified May 2026):
//! * Cerebras 1M tokens/day free since April 2026
//!   (getaiperks.com/en/ai/cerebras-free-tier-guide).
//! * Groq 14,400 req/day default
//!   (tokenmix.ai/blog/groq-free-tier-limits-2026).
//! * DeepSeek does NOT have a daily-quota-style free tier; the guard
//!   slot exists for future backends that ship with one.

use std::sync::Arc;

use elai_core::clock::ClockProvider;
use elai_providers::runtime::RuntimeBackend;
use rustc_hash::FxHashMap;

/// Per-backend daily quota. `usd_cap_cents` is `None` for the pure-
/// req-count case (Groq); `req_cap` is `None` for the pure-token case
/// (Cerebras). Only one of the two needs to be set to guard the
/// backend; both can be set for backends that publish both.
#[derive(Debug, Clone, Copy)]
pub struct DailyQuota {
    pub req_cap: Option<u64>,
    pub usd_cap_cents: Option<u64>,
    pub token_cap: Option<u64>,
}

/// Built-in defaults per the 2026 free-tier grounding. Override via
/// `[cloud.free_tier]` in `elai_cc.toml` when providers publish new
/// limits.
#[must_use]
pub fn default_quota_for(backend: RuntimeBackend) -> Option<DailyQuota> {
    match backend {
        RuntimeBackend::CerebrasFree => Some(DailyQuota {
            req_cap: None,
            usd_cap_cents: None,
            token_cap: Some(1_000_000),
        }),
        RuntimeBackend::GroqFree => Some(DailyQuota {
            req_cap: Some(14_400),
            usd_cap_cents: None,
            token_cap: None,
        }),
        _ => None,
    }
}

/// 3-arm quota status. The router consults this BEFORE dispatching a
/// call; the orchestrator emits the matching SSB row on `Approaching`
/// + `Exceeded` returns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuotaStatus {
    /// `>= 0` and below the 80% warn threshold; safe to dispatch.
    Ok { remaining_pct: u8 },
    /// Crossed the 80% warn threshold but below 95% refuse threshold;
    /// dispatch allowed but warn-row emitted.
    Approaching { remaining_pct: u8 },
    /// At or above 95% refuse threshold; dispatch BLOCKED.
    Exceeded,
}

impl QuotaStatus {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ok { .. } => "ok",
            Self::Approaching { .. } => "approaching",
            Self::Exceeded => "exceeded",
        }
    }
}

const WARN_PCT: u32 = 80;
const REFUSE_PCT: u32 = 95;

/// Per-backend running counters. Reset at the UTC day boundary; the
/// reset is lazy (checked on every `record` / `check` call) so no
/// background timer is required.
#[derive(Debug, Default, Clone, Copy)]
struct BackendCounters {
    pub req_used: u64,
    pub usd_cents_used: u64,
    pub tokens_used: u64,
    pub last_reset_day: u32,
}

/// Free-tier guard. One instance per orchestrator session; the
/// dashboard's `get_free_tier_quotas` Tauri command reads this state
/// out via snapshot.
pub struct FreeTierGuard {
    counters: FxHashMap<RuntimeBackend, BackendCounters>,
    quotas: FxHashMap<RuntimeBackend, DailyQuota>,
    clock: Arc<dyn ClockProvider>,
}

impl FreeTierGuard {
    /// Construct with built-in defaults for Cerebras + Groq.
    #[must_use]
    pub fn with_defaults(clock: Arc<dyn ClockProvider>) -> Self {
        let mut quotas = FxHashMap::default();
        for backend in [RuntimeBackend::CerebrasFree, RuntimeBackend::GroqFree] {
            if let Some(q) = default_quota_for(backend) {
                quotas.insert(backend, q);
            }
        }
        Self {
            counters: FxHashMap::default(),
            quotas,
            clock,
        }
    }

    /// Set or replace the daily quota for a backend.
    pub fn set_quota(&mut self, backend: RuntimeBackend, quota: DailyQuota) {
        self.quotas.insert(backend, quota);
    }

    fn today_day(&self) -> u32 {
        // Days since UNIX epoch (UTC). 86_400 s/day.
        let secs = self.clock.now_secs();
        u32::try_from(secs / 86_400).unwrap_or(0)
    }

    fn touch_day_boundary(&mut self, backend: RuntimeBackend) {
        let day = self.today_day();
        let counters = self.counters.entry(backend).or_default();
        if counters.last_reset_day != day {
            *counters = BackendCounters {
                last_reset_day: day,
                ..Default::default()
            };
        }
    }

    /// Record usage from a completed call.
    pub fn record(&mut self, backend: RuntimeBackend, req: u64, usd_cents: u64, tokens: u64) {
        self.touch_day_boundary(backend);
        if let Some(c) = self.counters.get_mut(&backend) {
            c.req_used = c.req_used.saturating_add(req);
            c.usd_cents_used = c.usd_cents_used.saturating_add(usd_cents);
            c.tokens_used = c.tokens_used.saturating_add(tokens);
        }
    }

    /// Inspect the current quota status for a backend. Returns `None`
    /// when the backend has no quota configured (e.g. paid Anthropic
    /// API).
    pub fn check_quota(&mut self, backend: RuntimeBackend) -> Option<QuotaStatus> {
        self.touch_day_boundary(backend);
        let quota = self.quotas.get(&backend).copied()?;
        let counters = self.counters.get(&backend).copied().unwrap_or_default();
        let pct = highest_used_pct(&quota, &counters);
        let status = if pct >= REFUSE_PCT {
            QuotaStatus::Exceeded
        } else if pct >= WARN_PCT {
            QuotaStatus::Approaching {
                remaining_pct: (100 - pct).min(100) as u8,
            }
        } else {
            QuotaStatus::Ok {
                remaining_pct: (100 - pct).min(100) as u8,
            }
        };
        Some(status)
    }

    /// Snapshot for dashboard rendering. Returns one entry per backend
    /// with a configured quota.
    #[must_use]
    pub fn snapshot(&mut self) -> Vec<(RuntimeBackend, QuotaStatus)> {
        let backends: Vec<RuntimeBackend> = self.quotas.keys().copied().collect();
        backends
            .into_iter()
            .filter_map(|b| self.check_quota(b).map(|s| (b, s)))
            .collect()
    }
}

fn highest_used_pct(q: &DailyQuota, c: &BackendCounters) -> u32 {
    let mut max_pct: u32 = 0;
    if let Some(cap) = q.req_cap {
        if cap > 0 {
            let pct = ((c.req_used.saturating_mul(100)) / cap.max(1)) as u32;
            max_pct = max_pct.max(pct);
        }
    }
    if let Some(cap) = q.usd_cap_cents {
        if cap > 0 {
            let pct = ((c.usd_cents_used.saturating_mul(100)) / cap.max(1)) as u32;
            max_pct = max_pct.max(pct);
        }
    }
    if let Some(cap) = q.token_cap {
        if cap > 0 {
            let pct = ((c.tokens_used.saturating_mul(100)) / cap.max(1)) as u32;
            max_pct = max_pct.max(pct);
        }
    }
    max_pct.min(100)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct FixedClock(AtomicU64);
    impl FixedClock {
        fn new(secs: u64) -> Self {
            Self(AtomicU64::new(secs))
        }
    }
    impl ClockProvider for FixedClock {
        fn now(&self) -> elai_core::clock::LogicalInstant {
            elai_core::clock::LogicalInstant::from_tick(self.0.load(Ordering::SeqCst))
        }
        fn elapsed_since(&self, _m: elai_core::clock::LogicalInstant) -> std::time::Duration {
            std::time::Duration::from_millis(0)
        }
        fn system_now(&self) -> std::time::SystemTime {
            std::time::UNIX_EPOCH + std::time::Duration::from_secs(self.0.load(Ordering::SeqCst))
        }
        fn instant_now(&self) -> std::time::Instant {
            std::time::Instant::now() // TEST_KEEP: cfg(test) ClockProvider impl, fake-clock helper
        }
    }

    #[test]
    fn defaults_register_groq_and_cerebras() {
        assert!(default_quota_for(RuntimeBackend::GroqFree).is_some());
        assert!(default_quota_for(RuntimeBackend::CerebrasFree).is_some());
        assert!(default_quota_for(RuntimeBackend::AnthropicApi).is_none());
    }

    #[test]
    fn groq_at_80_percent_returns_approaching() {
        let clock: Arc<dyn ClockProvider> = Arc::new(FixedClock::new(1_700_000_000));
        let mut g = FreeTierGuard::with_defaults(clock);
        // Groq default 14_400 req/day; 80% = 11_520.
        g.record(RuntimeBackend::GroqFree, 11_520, 0, 0);
        let s = g.check_quota(RuntimeBackend::GroqFree).expect("status");
        assert_eq!(s.as_str(), "approaching");
    }

    #[test]
    fn groq_at_95_percent_returns_exceeded() {
        let clock: Arc<dyn ClockProvider> = Arc::new(FixedClock::new(1_700_000_000));
        let mut g = FreeTierGuard::with_defaults(clock);
        g.record(RuntimeBackend::GroqFree, 13_680, 0, 0);
        let s = g.check_quota(RuntimeBackend::GroqFree).expect("status");
        assert_eq!(s.as_str(), "exceeded");
    }

    #[test]
    fn cerebras_at_100_percent_returns_exceeded() {
        let clock: Arc<dyn ClockProvider> = Arc::new(FixedClock::new(1_700_000_000));
        let mut g = FreeTierGuard::with_defaults(clock);
        g.record(RuntimeBackend::CerebrasFree, 0, 0, 1_000_000);
        let s = g.check_quota(RuntimeBackend::CerebrasFree).expect("status");
        assert_eq!(s.as_str(), "exceeded");
    }

    #[test]
    fn day_rollover_resets_counters() {
        let secs0 = 86_400 * 100; // 100 days since epoch.
        let clock = Arc::new(FixedClock::new(secs0));
        let arc_clock: Arc<dyn ClockProvider> = clock.clone();
        let mut g = FreeTierGuard::with_defaults(arc_clock);
        g.record(RuntimeBackend::GroqFree, 14_400, 0, 0);
        assert_eq!(
            g.check_quota(RuntimeBackend::GroqFree).map(|s| s.as_str()),
            Some("exceeded")
        );
        // Roll forward 1 day.
        clock.0.store(secs0 + 86_400, Ordering::SeqCst);
        let s = g.check_quota(RuntimeBackend::GroqFree).expect("status");
        assert_eq!(s.as_str(), "ok");
    }
}
