// E-49 — Session-aggregator for CodingMetric snapshots.
//
// Folds N per-session metric snapshots into a single AggregatedReport
// suitable for the CLI `elai metrics aggregate` subverb and the
// dashboard MetricsDrawer summary.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::types::CodingMetric;

/// E-49 — multi-session aggregate.
///
/// Reports per-k average pass-rates, average refine-rate, and total
/// observed cost-per-task across sessions that supplied cost data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AggregatedReport {
    pub sessions: Vec<String>,
    pub pass_at_k_avg: BTreeMap<u32, f32>,
    pub refine_rate_avg: f32,
    /// Sum of cost-per-task across sessions that reported cost. None
    /// when no session in the set reported cost data.
    pub cost_per_task_total_cents: Option<u64>,
    pub ts_ms: u64,
}

#[derive(Debug, Error)]
pub enum AggregatorError {
    #[error("aggregator received zero sessions")]
    NoSessions,
}

/// E-49 — aggregate N metric snapshots into one report.
///
/// The aggregation strategy is intentionally simple (unweighted
/// arithmetic mean per k; sum of cost). Downstream consumers can
/// re-aggregate the underlying snapshots if they need a different
/// weighting.
pub fn aggregate_session_metrics(
    metrics: &[CodingMetric],
) -> Result<AggregatedReport, AggregatorError> {
    if metrics.is_empty() {
        return Err(AggregatorError::NoSessions);
    }

    let mut pass_sum: BTreeMap<u32, (f64, u32)> = BTreeMap::new();
    let mut refine_sum: f64 = 0.0;
    let mut cost_sum: u64 = 0;
    let mut cost_observed: bool = false;
    let mut session_ids: Vec<String> = Vec::with_capacity(metrics.len());
    let mut max_ts: u64 = 0;

    for m in metrics {
        session_ids.push(m.session_id.clone());
        if m.ts_ms > max_ts {
            max_ts = m.ts_ms;
        }
        for (k, v) in &m.pass_at_k {
            let entry = pass_sum.entry(*k).or_insert((0.0, 0));
            entry.0 += *v as f64;
            entry.1 += 1;
        }
        refine_sum += m.refine_rate as f64;
        if let Some(c) = m.cost_per_task_cents {
            cost_sum = cost_sum.saturating_add(c as u64);
            cost_observed = true;
        }
    }

    let mut pass_at_k_avg: BTreeMap<u32, f32> = BTreeMap::new();
    for (k, (sum, count)) in pass_sum {
        if count > 0 {
            pass_at_k_avg.insert(k, (sum / (count as f64)) as f32);
        }
    }

    let refine_rate_avg = (refine_sum / (metrics.len() as f64)) as f32;

    let cost_per_task_total_cents = if cost_observed { Some(cost_sum) } else { None };

    Ok(AggregatedReport {
        sessions: session_ids,
        pass_at_k_avg,
        refine_rate_avg,
        cost_per_task_total_cents,
        ts_ms: max_ts,
    })
}

#[cfg(test)]
mod tests {
    use super::super::collector::MetricsCollector;
    use super::*;

    #[test]
    fn aggregating_zero_sessions_is_an_error() {
        let err = aggregate_session_metrics(&[]).unwrap_err();
        assert!(matches!(err, AggregatorError::NoSessions));
    }

    #[test]
    fn aggregating_two_sessions_averages_pass_at_k() {
        let mut c1 = MetricsCollector::new();
        c1.record_pass_sample(1, 8, 10).expect("ok");
        let s1 = c1.snapshot("s-a", 1);

        let mut c2 = MetricsCollector::new();
        c2.record_pass_sample(1, 4, 10).expect("ok");
        let s2 = c2.snapshot("s-b", 2);

        let report = aggregate_session_metrics(&[s1, s2]).expect("agg ok");
        let p1 = report.pass_at_k_avg.get(&1).copied().expect("present");
        assert!((p1 - 0.6).abs() < 1e-6);
        assert_eq!(report.sessions.len(), 2);
        assert_eq!(report.cost_per_task_total_cents, None);
        assert_eq!(report.ts_ms, 2);
    }

    #[test]
    fn cost_aggregates_only_when_observed() {
        let mut c1 = MetricsCollector::new();
        c1.record_cost_sample_cents(50);
        let s1 = c1.snapshot("s-a", 10);

        let mut c2 = MetricsCollector::new();
        let s2 = c2.snapshot("s-b", 20);
        let _ = &mut c2;

        let report = aggregate_session_metrics(&[s1, s2]).expect("agg ok");
        assert_eq!(report.cost_per_task_total_cents, Some(50));
    }
}
