//! E-20 task 6 -- cloud-burn projection + daily-2x alert contract.
//!
//! Anchors:
//!   * docs.litellm.ai cost-tracking patterns
//!   * langfuse.com cost-monitoring patterns
//!   * traceloop.com cost-monitoring patterns
//!
//! See workflow/reasoning/E-20-decisions.md DECISION-8.

use elai_orchestrator::cost::burn_projection::{
    check_burn_alerts, daily_2x_alert, projected_monthly_burn, BurnAlertKind,
    CostHistogramRow, DEFAULT_BURN_WINDOW_DAYS, PROJECTED_MONTHLY_CEILING_DEFAULT_USD,
};

#[test]
fn empty_history_projects_zero() {
    let projection = projected_monthly_burn(&[], DEFAULT_BURN_WINDOW_DAYS);
    assert_eq!(projection.projected_usd, 0.0);
    assert_eq!(projection.samples_used, 0);
}

#[test]
fn constant_burn_projects_to_30x_with_tight_band() {
    let history: Vec<CostHistogramRow> = (0..7)
        .map(|d| CostHistogramRow {
            day_index: d,
            usd: 2.0,
        })
        .collect();
    let projection = projected_monthly_burn(&history, DEFAULT_BURN_WINDOW_DAYS);
    // mean = 2 USD/day, projected = 60 USD/month.
    assert!((projection.projected_usd - 60.0).abs() < 1e-6);
    // Zero variance -> band should collapse near projection.
    let band_width = projection.confidence_pct_hi - projection.projected_usd;
    assert!(band_width < 1.0, "band {band_width} should be tight");
}

#[test]
fn projection_within_10_pct_of_analytic_baseline() {
    // Feed 20-task spend histogram with mean 0.5 USD/day per plan
    // DETERMINISTIC_CHECK 6 ("feed 20-task spend histogram -> assert
    // projected-monthly within +-10% of analytic baseline").
    let history: Vec<CostHistogramRow> = (0..20)
        .map(|d| CostHistogramRow {
            day_index: d.min(29),
            usd: 0.5,
        })
        .collect();
    let projection = projected_monthly_burn(&history, 0);
    let analytic = 0.5 * 30.0; // 15 USD/month
    let diff_pct = (projection.projected_usd - analytic).abs() / analytic;
    assert!(
        diff_pct <= 0.10,
        "projection {} vs analytic {} diff {}",
        projection.projected_usd,
        analytic,
        diff_pct
    );
}

#[test]
fn daily_2x_alert_fires_above_2x_baseline() {
    let alert = daily_2x_alert(5.0, 1.0);
    assert_eq!(alert, Some(BurnAlertKind::Daily2x));
}

#[test]
fn daily_2x_alert_quiet_below_2x() {
    let alert = daily_2x_alert(1.5, 1.0);
    assert!(alert.is_none());
}

#[test]
fn daily_2x_alert_quiet_zero_baseline() {
    let alert = daily_2x_alert(100.0, 0.0);
    assert!(alert.is_none());
}

#[test]
fn check_burn_alerts_quiet_baseline() {
    let history: Vec<CostHistogramRow> = (1..=7)
        .map(|d| CostHistogramRow {
            day_index: d,
            usd: 1.0,
        })
        .collect();
    let alerts = check_burn_alerts(1.0, &history, PROJECTED_MONTHLY_CEILING_DEFAULT_USD);
    assert!(alerts.is_empty());
}

#[test]
fn check_burn_alerts_fires_on_daily_spike() {
    let history: Vec<CostHistogramRow> = (1..=7)
        .map(|d| CostHistogramRow {
            day_index: d,
            usd: 1.0,
        })
        .collect();
    let alerts = check_burn_alerts(5.0, &history, 1000.0);
    assert!(alerts.contains(&BurnAlertKind::Daily2x));
}

#[test]
fn check_burn_alerts_fires_on_monthly_overshoot() {
    let history: Vec<CostHistogramRow> = (0..7)
        .map(|d| CostHistogramRow {
            day_index: d,
            usd: 10.0,
        })
        .collect();
    // projection = 10 * 30 = 300; ceiling = 50.
    let alerts = check_burn_alerts(10.0, &history, PROJECTED_MONTHLY_CEILING_DEFAULT_USD);
    assert!(alerts.contains(&BurnAlertKind::ProjectedMonthly));
}

#[test]
fn alert_labels_stable() {
    assert_eq!(BurnAlertKind::Daily2x.as_str(), "daily_2x");
    assert_eq!(
        BurnAlertKind::ProjectedMonthly.as_str(),
        "projected_monthly"
    );
}
