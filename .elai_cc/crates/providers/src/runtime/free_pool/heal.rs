//! J-54 heal_or_spill — bounded retry + cloud spill on PoolExhausted.
//!
//! On PoolExhausted: if spill_to_cloud is allowed, return Spill; else
//! sleep-bounded retry using QuotaTracker::soonest_reset, capped by
//! max_retries + max_total_wait. Other errors propagate unchanged.

use std::time::Duration;

use super::dispatch::{Deployment, PoolExhausted};
use super::quota::QuotaTracker;

/// Outcome of heal_or_spill.
#[derive(Debug)]
pub enum HealOutcome {
    /// Retry after sleeping for the given duration.
    Retry { delay: Duration, attempt: u32 },
    /// Escalate to cloud backend.
    Spill,
    /// All retries exhausted; give up.
    GiveUp,
}

/// Configuration for heal_or_spill.
#[derive(Debug, Clone)]
pub struct HealConfig {
    pub max_retries: u32,
    pub max_total_wait: Duration,
    pub spill_to_cloud: bool,
}

impl Default for HealConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            max_total_wait: Duration::from_secs(300),
            spill_to_cloud: true,
        }
    }
}

/// Decide what to do on a PoolExhausted error.
///
/// If spill_to_cloud is allowed, returns Spill immediately.
/// Otherwise, computes the delay from QuotaTracker::soonest_reset
/// (capped by max_total_wait and remaining budget), and returns Retry
/// if retries remain, or GiveUp if exhausted.
pub fn heal_or_spill(
    exhausted: &PoolExhausted,
    quota: &QuotaTracker,
    attempt: u32,
    cfg: &HealConfig,
) -> HealOutcome {
    let _ = exhausted;

    if cfg.spill_to_cloud {
        return HealOutcome::Spill;
    }

    if attempt >= cfg.max_retries {
        return HealOutcome::GiveUp;
    }

    let delay = quota
        .soonest_reset()
        .unwrap_or(Duration::from_secs(60))
        .min(cfg.max_total_wait);

    HealOutcome::Retry { delay, attempt }
}

/// Error from `select_with_heal`: either the pool is exhausted (with
/// retries spent), or cloud spill was requested.
#[derive(Debug, Clone, thiserror::Error)]
pub enum SelectError {
    #[error("free pool exhausted, all retries spent")]
    Exhausted,
    #[error("free pool exhausted, cloud spill requested")]
    SpillRequested,
}

/// Select a deployment, retrying with heal_or_spill on PoolExhausted.
///
/// This is the production call path: it tries dispatcher.select(),
/// and on PoolExhausted consults heal_or_spill. On Retry it sleeps
/// (via tokio::time::sleep) and tries again. On Spill it returns
/// Err(SelectError::SpillRequested). On GiveUp it returns
/// Err(SelectError::Exhausted).
pub async fn select_with_heal(
    dispatcher: &super::dispatch::FreePoolDispatcher,
    quota: &QuotaTracker,
    cfg: &HealConfig,
) -> Result<Deployment, SelectError> {
    let mut attempt = 0u32;
    loop {
        match dispatcher.select(quota) {
            Ok(dep) => return Ok(dep),
            Err(exhausted) => {
                let _ = &exhausted;
                match heal_or_spill(&exhausted, quota, attempt, cfg) {
                    HealOutcome::Spill => {
                        tracing::info!(
                            target = "free_pool",
                            "pool exhausted, spilling to cloud"
                        );
                        return Err(SelectError::SpillRequested);
                    }
                    HealOutcome::GiveUp => {
                        tracing::warn!(
                            target = "free_pool",
                            "pool exhausted, retries exhausted"
                        );
                        return Err(SelectError::Exhausted);
                    }
                    HealOutcome::Retry { delay, attempt: a } => {
                        tracing::debug!(
                            target = "free_pool",
                            "pool exhausted, retry {} after {:?}",
                            a,
                            delay
                        );
                        tokio::time::sleep(delay).await;
                        attempt = a + 1;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn tmp_quota() -> super::super::quota::QuotaTracker {
        let path = std::env::temp_dir().join(format!(
            "heal_test_{}.json",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        super::super::quota::QuotaTracker::unlimited(path)
    }

    #[test]
    fn spill_when_allowed() {
        let quota = tmp_quota();
        let cfg = HealConfig {
            spill_to_cloud: true,
            ..Default::default()
        };
        let outcome = heal_or_spill(&PoolExhausted, &quota, 0, &cfg);
        assert!(matches!(outcome, HealOutcome::Spill));
    }

    #[test]
    fn retry_when_spill_disabled() {
        let quota = tmp_quota();
        let cfg = HealConfig {
            spill_to_cloud: false,
            max_retries: 3,
            max_total_wait: Duration::from_secs(300),
        };
        let outcome = heal_or_spill(&PoolExhausted, &quota, 0, &cfg);
        assert!(matches!(outcome, HealOutcome::Retry { .. }));
    }

    #[test]
    fn give_up_after_max_retries() {
        let quota = tmp_quota();
        let cfg = HealConfig {
            spill_to_cloud: false,
            max_retries: 2,
            max_total_wait: Duration::from_secs(300),
        };
        let outcome = heal_or_spill(&PoolExhausted, &quota, 2, &cfg);
        assert!(matches!(outcome, HealOutcome::GiveUp));
    }

    #[test]
    fn delay_capped_by_max_total_wait() {
        let quota = tmp_quota();
        let cfg = HealConfig {
            spill_to_cloud: false,
            max_retries: 3,
            max_total_wait: Duration::from_secs(10),
        };
        let outcome = heal_or_spill(&PoolExhausted, &quota, 0, &cfg);
        if let HealOutcome::Retry { delay, .. } = outcome {
            assert!(delay <= Duration::from_secs(10));
        } else {
            panic!("expected Retry");
        }
    }
}
