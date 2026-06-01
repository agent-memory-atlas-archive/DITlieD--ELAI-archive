// E-49 — MetricsCollector: per-session capture point.
//
// Folds raw signals (pass-at-k samples, refine-rate samples, cost-per-task
// readings) into a single `CodingMetric` capture row. The collector is
// memory-only and side-effect-free — emission to SSB happens at the
// caller boundary (CLI `elai metrics capture` or auditor hook) so we
// don't take a dependency on the SSB writer from this module.
//
// Anti-Lazy compliance:
//   * No unwrap()/expect() outside test code.
//   * No silent default for missing cost data — None stays None.
//   * Errors propagate via thiserror.

use std::collections::BTreeMap;
use std::time::UNIX_EPOCH;

use elai_core::clock::{ClockProvider, OsClock};
use thiserror::Error;

use super::types::CodingMetric;

/// Default refine-rate when the collector has zero observations. Matches
/// the contract test: an empty collector emits `refine_rate = 0.0`,
/// `pass_at_k = {}`, `cost_per_task_cents = None`.
const REFINE_RATE_EMPTY: f32 = 0.0;

#[derive(Debug, Error)]
pub enum MetricsCollectorError {
    #[error("pass_at_k denominator was zero (k={k})")]
    PassDenominatorZero { k: u32 },
    #[error("refine-rate denominator was zero")]
    RefineDenominatorZero,
    #[error("system clock returned a pre-epoch time: {0}")]
    ClockPreEpoch(String),
}

/// Per-session metric collector. Owned by the caller; not Send-shared.
#[derive(Debug, Clone, Default)]
pub struct MetricsCollector {
    /// Raw pass/total samples per k.
    pass_samples: BTreeMap<u32, (u32, u32)>,
    /// Refine successes / total flagged tasks.
    refine_success: u32,
    refine_total: u32,
    /// Cost samples in cents. None until at least one sample observed.
    cost_samples_cents: Option<Vec<u32>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a single pass/total observation at the given k. Both
    /// `pass` and `total` are cumulative counts within this bucket.
    pub fn record_pass_sample(
        &mut self,
        k: u32,
        pass: u32,
        total: u32,
    ) -> Result<(), MetricsCollectorError> {
        if total == 0 {
            return Err(MetricsCollectorError::PassDenominatorZero { k });
        }
        let entry = self.pass_samples.entry(k).or_insert((0, 0));
        entry.0 = entry.0.saturating_add(pass);
        entry.1 = entry.1.saturating_add(total);
        Ok(())
    }

    /// Record a refine-rate observation: `success` out of `total` flagged
    /// tasks were successfully refined.
    pub fn record_refine_sample(
        &mut self,
        success: u32,
        total: u32,
    ) -> Result<(), MetricsCollectorError> {
        if total == 0 {
            return Err(MetricsCollectorError::RefineDenominatorZero);
        }
        self.refine_success = self.refine_success.saturating_add(success);
        self.refine_total = self.refine_total.saturating_add(total);
        Ok(())
    }

    /// Record a per-task cost observation in cents. The first call
    /// initializes the sample bucket from `None` to `Some(vec![..])`.
    pub fn record_cost_sample_cents(&mut self, cents: u32) {
        self.cost_samples_cents
            .get_or_insert_with(Vec::new)
            .push(cents);
    }

    /// Emit a `CodingMetric` snapshot for `session_id` using the current
    /// `SystemTime`. Returns an error only if the clock is pre-epoch.
    pub fn capture_metrics(
        &self,
        session_id: &str,
    ) -> Result<CodingMetric, MetricsCollectorError> {
        let ts_ms = current_ts_ms()?;
        Ok(self.snapshot(session_id, ts_ms))
    }

    /// Emit a `CodingMetric` snapshot using a caller-supplied `ts_ms`.
    /// Used by tests + the CLI when an explicit timestamp is required.
    pub fn snapshot(&self, session_id: &str, ts_ms: u64) -> CodingMetric {
        let mut pass_at_k: BTreeMap<u32, f32> = BTreeMap::new();
        for (k, (pass, total)) in &self.pass_samples {
            if *total > 0 {
                pass_at_k.insert(*k, (*pass as f32) / (*total as f32));
            }
        }

        let refine_rate = if self.refine_total == 0 {
            REFINE_RATE_EMPTY
        } else {
            (self.refine_success as f32) / (self.refine_total as f32)
        };

        let cost_per_task_cents = self.cost_samples_cents.as_ref().and_then(|v| {
            if v.is_empty() {
                None
            } else {
                let sum: u64 = v.iter().map(|c| *c as u64).sum();
                let avg = sum / (v.len() as u64);
                u32::try_from(avg).ok()
            }
        });

        CodingMetric {
            session_id: session_id.to_string(),
            pass_at_k,
            refine_rate,
            cost_per_task_cents,
            ts_ms,
        }
    }
}

fn current_ts_ms() -> Result<u64, MetricsCollectorError> {
    let now = OsClock.system_now();
    let dur = now
        .duration_since(UNIX_EPOCH)
        .map_err(|e| MetricsCollectorError::ClockPreEpoch(e.to_string()))?;
    u64::try_from(dur.as_millis()).map_err(|e| MetricsCollectorError::ClockPreEpoch(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_collector_snapshot_has_zero_refine_and_no_cost() {
        let collector = MetricsCollector::new();
        let snap = collector.snapshot("s-test", 1_700_000_000_000);
        assert!(snap.pass_at_k.is_empty());
        assert_eq!(snap.refine_rate, 0.0);
        assert_eq!(snap.cost_per_task_cents, None);
        assert_eq!(snap.ts_ms, 1_700_000_000_000);
    }

    #[test]
    fn pass_sample_zero_total_is_rejected() {
        let mut collector = MetricsCollector::new();
        let err = collector.record_pass_sample(1, 0, 0).unwrap_err();
        assert!(matches!(
            err,
            MetricsCollectorError::PassDenominatorZero { k: 1 }
        ));
    }

    #[test]
    fn pass_sample_accumulates_and_averages() {
        let mut collector = MetricsCollector::new();
        collector.record_pass_sample(1, 7, 10).expect("ok");
        collector.record_pass_sample(1, 3, 10).expect("ok");
        let snap = collector.snapshot("s-x", 0);
        let pass_at_1 = snap.pass_at_1().expect("present");
        assert!((pass_at_1 - 0.5).abs() < 1e-6);
    }

    #[test]
    fn refine_rate_computes_when_samples_present() {
        let mut collector = MetricsCollector::new();
        collector.record_refine_sample(3, 5).expect("ok");
        let snap = collector.snapshot("s-y", 0);
        assert!((snap.refine_rate - 0.6).abs() < 1e-6);
    }

    #[test]
    fn refine_sample_zero_total_is_rejected() {
        let mut collector = MetricsCollector::new();
        let err = collector.record_refine_sample(0, 0).unwrap_err();
        assert!(matches!(err, MetricsCollectorError::RefineDenominatorZero));
    }

    #[test]
    fn cost_sample_averages_to_some() {
        let mut collector = MetricsCollector::new();
        collector.record_cost_sample_cents(100);
        collector.record_cost_sample_cents(200);
        let snap = collector.snapshot("s-z", 0);
        assert_eq!(snap.cost_per_task_cents, Some(150));
    }
}
