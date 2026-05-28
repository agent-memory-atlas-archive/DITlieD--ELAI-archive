//! E-20 -- daily burn rate + projected-monthly cloud-spend tracking.
//!
//! Sits alongside [`crate::cost::cloud_abort`] under the `cost/` subtree
//! (NOT `crates/orchestrator/src/cloud_spend.rs` -- that path was
//! REJECTED per E39_CLOUD_SPEND_PATH_DRIFT in
//! workflow/memory-bank/gotchas.md).
//!
//! Surfaces two pre-computed indicators for CloudPanel:
//!   * [`projected_monthly_burn`] -- N-task rolling histogram +
//!     extrapolation; defends against the "death by a thousand cheap
//!     calls" failure mode (LiteLLM / Langfuse / Traceloop production
//!     patterns).
//!   * [`daily_2x_alert`] -- today's burn vs a 7-day baseline; fires
//!     when today > 2 x baseline.
//!
//! Both helpers are pure (no Tokio / no async / no env reads -- those
//! are the caller's responsibility) so they remain unit-testable
//! without the live SSB. Producers emit
//! [`crate::ssb::SsbActionType::LocalCloudRoutingEvent`] with
//! `kind = burn_alert` when the threshold is crossed.

use serde::{Deserialize, Serialize};

/// 2-arm alert kind. Matches the wire labels used in
/// `LocalCloudRoutingEventPayload.details_json.alert_kind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BurnAlertKind {
    /// Today's burn exceeded `2 x baseline` (rolling 7-day mean).
    Daily2x,
    /// Projected-monthly extrapolation exceeded the configured
    /// ceiling.
    ProjectedMonthly,
}

impl BurnAlertKind {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Daily2x => "daily_2x",
            Self::ProjectedMonthly => "projected_monthly",
        }
    }
}

/// Single cost-history row: a per-day or per-task USD amount.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CostHistogramRow {
    /// Day index (0 = today, 1 = yesterday, ..., 29 = 30 days ago).
    pub day_index: u8,
    /// USD spent during that day.
    pub usd: f64,
}

/// Projection output. `projected_usd` is the linear extrapolation of
/// the most recent N days to a 30-day month. `confidence_pct_lo /
/// confidence_pct_hi` form a coarse +-1-sigma band (sample-stddev-
/// derived). Conservative defaults: when fewer than 3 history rows are
/// available, the projection collapses to the simple mean * 30 with a
/// generous band.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ProjectedBurn {
    pub projected_usd: f64,
    pub confidence_pct_lo: f64,
    pub confidence_pct_hi: f64,
    pub samples_used: usize,
}

/// Default window size when the caller does not specify one. 7 days
/// matches the rolling-7-day baseline used by
/// [`daily_2x_alert`].
pub const DEFAULT_BURN_WINDOW_DAYS: u8 = 7;

/// Default ceiling for projected-monthly alerting. $50 / month is the
/// rocketedge.com 2026 soft cap that matches E-39's
/// [`crate::cost::cloud_abort::PerRoleCapsConfig`] default per-task
/// scale (the existing soft cap on the daily axis).
pub const PROJECTED_MONTHLY_CEILING_DEFAULT_USD: f64 = 50.0;

/// Project the next 30-day spend from the given cost history.
///
/// `history` is a slice of rows; the function does NOT assume any
/// particular ordering, but uses `day_index <= window_days` as a
/// filter. `window_days = 0` means "use all rows".
#[must_use]
pub fn projected_monthly_burn(history: &[CostHistogramRow], window_days: u8) -> ProjectedBurn {
    let limit = if window_days == 0 { u8::MAX } else { window_days };
    let in_window: Vec<f64> = history
        .iter()
        .filter(|row| row.day_index < limit)
        .map(|row| row.usd)
        .collect();
    let n = in_window.len();
    if n == 0 {
        return ProjectedBurn {
            projected_usd: 0.0,
            confidence_pct_lo: 0.0,
            confidence_pct_hi: 0.0,
            samples_used: 0,
        };
    }
    let mean: f64 = in_window.iter().copied().sum::<f64>() / (n as f64);
    let projected = mean * 30.0;
    // Sample stddev (n-1 denominator) -> +-1-sigma band; fall back to
    // mean/2 when only one row is available (defensive: don't return
    // a zero-width band on a single-sample projection).
    let band = if n >= 2 {
        let var: f64 = in_window
            .iter()
            .map(|x| {
                let d = x - mean;
                d * d
            })
            .sum::<f64>()
            / ((n - 1) as f64);
        var.sqrt() * 30.0
    } else {
        mean.abs() * 15.0
    };
    let lo = (projected - band).max(0.0);
    let hi = projected + band;
    ProjectedBurn {
        projected_usd: projected,
        confidence_pct_lo: lo,
        confidence_pct_hi: hi,
        samples_used: n,
    }
}

/// Today's burn vs a 7-day rolling baseline. Returns `Some(Daily2x)`
/// when today exceeds 2 x baseline AND baseline > 0; otherwise None.
///
/// Per the LiteLLM / Langfuse / Traceloop production patterns: a
/// sudden-spike alert at 2x baseline catches retry-storm regressions
/// before they accumulate into a monthly-burn overshoot.
#[must_use]
pub fn daily_2x_alert(today_usd: f64, baseline_usd: f64) -> Option<BurnAlertKind> {
    if baseline_usd <= 0.0 {
        return None;
    }
    if today_usd > baseline_usd * 2.0 {
        Some(BurnAlertKind::Daily2x)
    } else {
        None
    }
}

/// Convenience: combine [`projected_monthly_burn`] +
/// [`daily_2x_alert`] into a single alert decision.
///
/// `today_usd` is today's running total. `history` includes today's
/// rolling total at `day_index = 0`; the function computes the
/// rolling-7-day baseline excluding today.
#[must_use]
pub fn check_burn_alerts(
    today_usd: f64,
    history: &[CostHistogramRow],
    monthly_ceiling_usd: f64,
) -> Vec<BurnAlertKind> {
    let mut alerts = Vec::new();
    let baseline_window: Vec<f64> = history
        .iter()
        .filter(|row| row.day_index >= 1 && row.day_index <= DEFAULT_BURN_WINDOW_DAYS)
        .map(|row| row.usd)
        .collect();
    let baseline = if !baseline_window.is_empty() {
        baseline_window.iter().copied().sum::<f64>() / (baseline_window.len() as f64)
    } else {
        0.0
    };
    if let Some(a) = daily_2x_alert(today_usd, baseline) {
        alerts.push(a);
    }
    let projection = projected_monthly_burn(history, DEFAULT_BURN_WINDOW_DAYS);
    if projection.projected_usd > monthly_ceiling_usd {
        alerts.push(BurnAlertKind::ProjectedMonthly);
    }
    alerts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alert_kind_labels_stable() {
        assert_eq!(BurnAlertKind::Daily2x.as_str(), "daily_2x");
        assert_eq!(BurnAlertKind::ProjectedMonthly.as_str(), "projected_monthly");
    }

    #[test]
    fn projected_zero_on_empty_history() {
        let projection = projected_monthly_burn(&[], DEFAULT_BURN_WINDOW_DAYS);
        assert_eq!(projection.projected_usd, 0.0);
        assert_eq!(projection.samples_used, 0);
    }

    #[test]
    fn projected_constant_burn_extrapolates_to_30x() {
        let history: Vec<CostHistogramRow> = (0..7)
            .map(|d| CostHistogramRow {
                day_index: d,
                usd: 1.0,
            })
            .collect();
        let projection = projected_monthly_burn(&history, DEFAULT_BURN_WINDOW_DAYS);
        assert!((projection.projected_usd - 30.0).abs() < 1e-6);
        // Zero variance -> tight band.
        assert!(projection.confidence_pct_hi - projection.projected_usd < 1.0);
    }

    #[test]
    fn projected_window_filter_excludes_old_rows() {
        let history = vec![
            CostHistogramRow {
                day_index: 0,
                usd: 1.0,
            },
            CostHistogramRow {
                day_index: 6,
                usd: 1.0,
            },
            CostHistogramRow {
                day_index: 20,
                usd: 100.0,
            },
        ];
        let projection = projected_monthly_burn(&history, 7);
        // Only first 2 rows in window: mean = 1.0 -> projection = 30.
        assert!((projection.projected_usd - 30.0).abs() < 1e-6);
        assert_eq!(projection.samples_used, 2);
    }

    #[test]
    fn projected_window_zero_means_all_rows() {
        let history = vec![
            CostHistogramRow {
                day_index: 0,
                usd: 2.0,
            },
            CostHistogramRow {
                day_index: 200,
                usd: 0.0,
            },
        ];
        let projection = projected_monthly_burn(&history, 0);
        // mean = 1.0 -> projection = 30.
        assert!((projection.projected_usd - 30.0).abs() < 1e-6);
        assert_eq!(projection.samples_used, 2);
    }

    #[test]
    fn daily_2x_alert_below_threshold_none() {
        let alert = daily_2x_alert(1.5, 1.0);
        assert!(alert.is_none());
    }

    #[test]
    fn daily_2x_alert_above_threshold_fires() {
        let alert = daily_2x_alert(3.0, 1.0);
        assert_eq!(alert, Some(BurnAlertKind::Daily2x));
    }

    #[test]
    fn daily_2x_alert_zero_baseline_none() {
        // No baseline data -> cannot alert (avoid div-by-zero noise).
        let alert = daily_2x_alert(100.0, 0.0);
        assert!(alert.is_none());
    }

    #[test]
    fn check_burn_alerts_under_ceilings_quiet() {
        let history: Vec<CostHistogramRow> = (1..=7)
            .map(|d| CostHistogramRow {
                day_index: d,
                usd: 1.0,
            })
            .collect();
        // today = 1.0, baseline = 1.0, projection = 30 (below 50).
        let alerts = check_burn_alerts(1.0, &history, PROJECTED_MONTHLY_CEILING_DEFAULT_USD);
        assert!(alerts.is_empty());
    }

    #[test]
    fn check_burn_alerts_daily_spike_fires() {
        let history: Vec<CostHistogramRow> = (1..=7)
            .map(|d| CostHistogramRow {
                day_index: d,
                usd: 1.0,
            })
            .collect();
        // today = 5.0 (5x baseline of 1.0) -> daily_2x fires.
        let alerts = check_burn_alerts(5.0, &history, 1000.0);
        assert!(alerts.contains(&BurnAlertKind::Daily2x));
    }

    #[test]
    fn check_burn_alerts_monthly_projection_fires() {
        // Use a high baseline that projects above the default ceiling.
        let history: Vec<CostHistogramRow> = (0..7)
            .map(|d| CostHistogramRow {
                day_index: d,
                usd: 10.0,
            })
            .collect();
        // projection = 10 * 30 = 300 > 50 default ceiling
        let alerts = check_burn_alerts(10.0, &history, PROJECTED_MONTHLY_CEILING_DEFAULT_USD);
        assert!(alerts.contains(&BurnAlertKind::ProjectedMonthly));
    }
}
