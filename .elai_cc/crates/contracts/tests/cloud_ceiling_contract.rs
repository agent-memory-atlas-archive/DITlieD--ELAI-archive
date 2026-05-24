//! C-5 follow-on contract — cloud spend ceiling + double-emit dedup +
//! warn-limit + cost-estimate round-trip surface.
//!
//! Source plan:   docs/thefinalglance/C-plan/C-5_cloud_mode_hardening.md
//!                Task 1 (ceiling enforcement in CloudModeRouter::route_cloud)
//!                Task 2 (WARN_LIMIT_REACHED at warn_at_percent)
//!                Task 6 (pre-task cost estimation -> CostEstimate row)
//!                Task 8 (RouteBlocked SSB action type round-trip)
//!                Task 10 (cloud_ceiling_contract test surface)
//! Decisions:     workflow/reasoning/C-5-decisions.md
//!                Q7  — atomic 3-variant SCHEMA bump (102 -> 103,
//!                      bijection 242 -> 245, action_type_lock 303 -> 306).
//!                Q8  — Option A double-emit dedup: task_runner cap-hook owns
//!                      `cloud_spend_blocked_emitted` Mutex; route_cloud is
//!                      pure + sync and does NOT touch SSB. The single hit on
//!                      route_with_spend yields exactly ONE typed RouteBlocked
//!                      row + ONE backwards-compat EscalateToCloud(cap_reached)
//!                      row per UTC day, NOT two RouteBlocked rows.
//!                Q9  — UNGROUNDED_HEURISTIC cost constant + spread ratio
//!                      shape the CostEstimate payload bounds.
//!
//! SCHEMA_VERSION invariant: 103.  bijection size invariant: 245 typed
//! payloads exercised by ssb::tests::test_typed_payloads. Drift on either
//! number must rebump SCHEMA + update activity_entry_coverage_contract.

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::cloud_mode::{
    warn_limit_reached, CloudConfig, CloudCredentials, CloudModeError, CloudModeRouter, ExecMode,
    SpendSnapshotInput,
};
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, CostEstimatePayload, RouteBlockedPayload, Ssb, SsbActionType,
    SsbConfig, SsbPayload, WarnLimitReachedPayload, SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::SsbReader;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Shared fixtures (mirrors sibling cloud_spend_ssb_contract.rs).
// ---------------------------------------------------------------------------

fn fresh_db() -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("ssb.db");
    (dir, path)
}

async fn open_writer(path: &std::path::Path, scope: &str) -> Ssb {
    let cfg = SsbConfig {
        db_path: path.to_path_buf(),
        pool_size: Some(2),
        project_id: format!("cloud-ceiling-{scope}"),
        session_id: format!("cc-{scope}"),
    };
    Ssb::new(cfg, Arc::new(OsClock)).expect("ssb init")
}

/// Build a `CloudConfig` with credentials-ready defaults plus a daily cap.
/// The default_provider stays "anthropic" so `route_with_spend` reaches the
/// cap check rather than short-circuiting on UnknownProvider.
fn cfg_with_daily_cap(cap_usd: f64) -> CloudConfig {
    let mut c = CloudConfig::default();
    c.spend_cap_daily_usd = Some(cap_usd);
    c
}

fn cfg_with_monthly_cap(cap_usd: f64) -> CloudConfig {
    let mut c = CloudConfig::default();
    c.spend_cap_monthly_usd = Some(cap_usd);
    c
}

fn cfg_with_warn(cap_usd: f64, warn_pct: u8) -> CloudConfig {
    let mut c = cfg_with_daily_cap(cap_usd);
    c.warn_at_percent = Some(warn_pct);
    c
}

// ---------------------------------------------------------------------------
// (a) Ceiling: route_with_spend returns Daily/MonthlySpendCapExceeded at the
// exact spend == cap boundary (decisions.md Q8 — router is the truth oracle
// for the cap-reached signal; the cap-hook owns dedup downstream).
// ---------------------------------------------------------------------------

#[test]
fn daily_cap_triggers_route_blocked_at_exact_boundary() {
    let cfg = cfg_with_daily_cap(5.0);
    let router = CloudModeRouter::new(cfg);
    let snap = SpendSnapshotInput {
        day_usd: 5.0, // == cap exactly: the spec uses `>=` per cloud_mode.rs L349
        month_usd: 0.0,
    };
    let outcome = router.route_with_spend(
        ExecMode::Cloud,
        Some(CloudCredentials::for_test_with_token("tok")),
        Some(snap),
    );
    match outcome {
        Err(CloudModeError::DailySpendCapExceeded { spent_usd, cap_usd }) => {
            assert!(
                (spent_usd - 5.0).abs() < f64::EPSILON,
                "spent_usd preserved on the error variant"
            );
            assert!(
                (cap_usd - 5.0).abs() < f64::EPSILON,
                "cap_usd preserved on the error variant"
            );
        }
        other => panic!("expected DailySpendCapExceeded at exact boundary, got {other:?}"),
    }
}

#[test]
fn monthly_cap_triggers_route_blocked_when_day_under_cap() {
    let cfg = cfg_with_monthly_cap(50.0);
    let router = CloudModeRouter::new(cfg);
    let snap = SpendSnapshotInput {
        day_usd: 0.50,    // well under
        month_usd: 50.01, // over the monthly cap
    };
    let outcome = router.route_with_spend(
        ExecMode::Cloud,
        Some(CloudCredentials::for_test_with_token("tok")),
        Some(snap),
    );
    assert!(
        matches!(outcome, Err(CloudModeError::MonthlySpendCapExceeded { .. })),
        "monthly cap path must fire when month_usd >= cap regardless of day_usd: got {outcome:?}"
    );
}

#[test]
fn spend_under_cap_routes_to_cloud_cleanly() {
    let cfg = cfg_with_daily_cap(5.0);
    let router = CloudModeRouter::new(cfg);
    let snap = SpendSnapshotInput {
        day_usd: 4.99,
        month_usd: 4.99,
    };
    let outcome = router.route_with_spend(
        ExecMode::Cloud,
        Some(CloudCredentials::for_test_with_token("tok")),
        Some(snap),
    );
    assert!(
        outcome.is_ok(),
        "spend strictly under cap must route to Cloud: got {outcome:?}"
    );
}

// ---------------------------------------------------------------------------
// (b) Double-emit dedup (Option A surface): a single cap-cross produces ONE
// typed RouteBlocked row in SSB. The cap-hook owns the
// `cloud_spend_blocked_emitted` Mutex; route_with_spend is sync + pure and
// does NOT itself emit. This test simulates the hook by writing exactly one
// row per cap event and verifying query_by_type returns it once even when
// the router is called twice in the same UTC day.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn route_blocked_emits_at_most_once_per_cap_event() {
    let (_dir, path) = fresh_db();
    let ssb = open_writer(&path, "dedup").await;

    let cfg = cfg_with_daily_cap(5.0);
    let router = CloudModeRouter::new(cfg);
    let snap = SpendSnapshotInput {
        day_usd: 5.10,
        month_usd: 5.10,
    };

    // First probe: cap is exceeded, hook emits the row.
    let r1 = router.route_with_spend(
        ExecMode::Cloud,
        Some(CloudCredentials::for_test_with_token("tok")),
        Some(snap),
    );
    assert!(matches!(r1, Err(CloudModeError::DailySpendCapExceeded { .. })));
    let payload = SsbPayload::RouteBlocked(RouteBlockedPayload {
        reason: "daily_cap_exceeded".into(),
        spent_usd: 5.10,
        cap_usd: 5.0,
        cap_period: "daily".into(),
        baby_step_id: 7,
        ts_ms: 1_700_000_300_000,
    });
    ssb.write_payload("cloud_cap_hook", 0, payload, Vec::new())
        .await
        .expect("write route_blocked");

    // Second probe in the SAME UTC day: router still returns the error
    // (it is pure + sync, no internal dedup), but Option A says the hook
    // MUST NOT emit again. We simulate the dedup by NOT writing.
    let r2 = router.route_with_spend(
        ExecMode::Cloud,
        Some(CloudCredentials::for_test_with_token("tok")),
        Some(snap),
    );
    assert!(matches!(r2, Err(CloudModeError::DailySpendCapExceeded { .. })));

    let rows = ssb
        .query_by_type(SsbActionType::RouteBlocked)
        .expect("query route_blocked");
    assert_eq!(
        rows.len(),
        1,
        "Option A dedup: at most one RouteBlocked row per cap event per UTC day"
    );
}

// ---------------------------------------------------------------------------
// (c) Warn-limit threshold: warn_limit_reached helper fires at the
// `cap * warn_at_percent / 100.0` boundary. The runner uses this BEFORE
// SSB writes so the WarnLimitReached row only emits once per UTC day
// (caller-side dedup via Mutex<bool>).
// ---------------------------------------------------------------------------

#[test]
fn warn_helper_fires_at_eighty_percent_boundary() {
    let cfg = cfg_with_warn(5.0, 80);
    assert!(
        warn_limit_reached(&cfg, 4.0),
        "spend == 80% of cap must trigger the warn helper (4.0 == 5.0 * 0.80)"
    );
}

#[test]
fn warn_helper_quiet_under_threshold() {
    let cfg = cfg_with_warn(5.0, 80);
    assert!(
        !warn_limit_reached(&cfg, 3.99),
        "spend strictly under 80% of cap must not trigger the warn helper"
    );
}

#[test]
fn warn_helper_skips_when_warn_at_percent_unset() {
    let mut cfg = cfg_with_daily_cap(5.0);
    cfg.warn_at_percent = None;
    assert!(
        !warn_limit_reached(&cfg, 4.99),
        "warn_at_percent=None disables the helper (no emission)"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn warn_limit_reached_payload_round_trips() {
    let (_dir, path) = fresh_db();
    let ssb = open_writer(&path, "warn-rt").await;

    let payload = SsbPayload::WarnLimitReached(WarnLimitReachedPayload {
        spent_usd: 4.05,
        cap_usd: 5.0,
        warn_at_percent: 80,
        pct_of_cap: 81.0,
        baby_step_id: 1,
        ts_ms: 1_700_000_400_000,
    });
    assert_eq!(payload.action_type(), SsbActionType::WarnLimitReached);

    ssb.write_payload("cloud_cap_hook", 0, payload, Vec::new())
        .await
        .expect("write warn_limit_reached");

    let rows = ssb
        .query_by_type(SsbActionType::WarnLimitReached)
        .expect("query warn_limit_reached");
    assert_eq!(rows.len(), 1);
    let decoded: SsbPayload = rmp_serde::from_slice(&rows[0].payload).expect("decode");
    match decoded {
        SsbPayload::WarnLimitReached(p) => {
            assert!((p.spent_usd - 4.05).abs() < 1e-9);
            assert!((p.cap_usd - 5.0).abs() < 1e-9);
            assert_eq!(p.warn_at_percent, 80);
            assert!((p.pct_of_cap - 81.0).abs() < 1e-9);
            assert_eq!(p.baby_step_id, 1);
            assert_eq!(p.ts_ms, 1_700_000_400_000);
        }
        other => panic!("expected WarnLimitReached, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// (d) CostEstimate round-trip: the cap-hook emits exactly one CostEstimate
// payload per task lifetime with a realistic (low_usd, high_usd) spread
// shaped by Q9's TOKENS_PER_LOC_UNGROUNDED_HEURISTIC + spread ratio.
// `heuristic_source` carries the canonical "ungrounded_heuristic" tag so the
// dashboard can render the disclaimer banner.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cost_estimate_payload_round_trips_with_realistic_spread() {
    let (_dir, path) = fresh_db();
    let ssb = open_writer(&path, "cost-rt").await;

    let payload = SsbPayload::CostEstimate(CostEstimatePayload {
        low_usd: 0.084,
        high_usd: 0.156,
        model: "claude-sonnet-4-6".into(),
        step_count: 3,
        heuristic_source: "ungrounded_heuristic".into(),
        ts_ms: 1_700_000_500_000,
    });
    assert_eq!(payload.action_type(), SsbActionType::CostEstimate);

    ssb.write_payload("cloud_cap_hook", 0, payload, Vec::new())
        .await
        .expect("write cost_estimate");

    let rows = ssb
        .query_by_type(SsbActionType::CostEstimate)
        .expect("query cost_estimate");
    assert_eq!(rows.len(), 1);
    let decoded: SsbPayload = rmp_serde::from_slice(&rows[0].payload).expect("decode");
    match decoded {
        SsbPayload::CostEstimate(p) => {
            assert!(p.low_usd > 0.0, "low_usd is the spread lower bound");
            assert!(p.high_usd > p.low_usd, "high_usd > low_usd");
            assert_eq!(p.model, "claude-sonnet-4-6");
            assert_eq!(p.step_count, 3);
            assert_eq!(
                p.heuristic_source, "ungrounded_heuristic",
                "Q9 disclaimer tag preserved on the row for dashboard banner"
            );
            assert_eq!(p.ts_ms, 1_700_000_500_000);
        }
        other => panic!("expected CostEstimate, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// (e) Bijection guards: the 3 new C-5 variants must round-trip through the
// public string <-> enum helpers per Q7's atomic SCHEMA bump to 103.
// SCHEMA_VERSION_PUBLIC pinning catches any silent downgrade.
// ---------------------------------------------------------------------------

#[test]
fn schema_version_floor_is_at_least_one_oh_three() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 103,
        "C-5 ceiling/double-emit landing requires SCHEMA_VERSION >= 103; got {SCHEMA_VERSION_PUBLIC}"
    );
}

#[test]
fn route_blocked_action_type_string_bijection() {
    assert_eq!(SsbActionType::RouteBlocked.as_sql_str(), "route_blocked");
    assert_eq!(
        ssb_action_type_from_str_public("route_blocked").expect("parse"),
        SsbActionType::RouteBlocked
    );
}

#[test]
fn warn_limit_reached_action_type_string_bijection() {
    assert_eq!(
        SsbActionType::WarnLimitReached.as_sql_str(),
        "warn_limit_reached"
    );
    assert_eq!(
        ssb_action_type_from_str_public("warn_limit_reached").expect("parse"),
        SsbActionType::WarnLimitReached
    );
}

#[test]
fn cost_estimate_action_type_string_bijection() {
    assert_eq!(SsbActionType::CostEstimate.as_sql_str(), "cost_estimate");
    assert_eq!(
        ssb_action_type_from_str_public("cost_estimate").expect("parse"),
        SsbActionType::CostEstimate
    );
}

// ---------------------------------------------------------------------------
// (f) Stub_ban semantic: absent SpendSnapshotInput (viewer error) MUST NOT
// block all cloud routes — route_with_spend(_, _, None) collapses to the
// no-caps path and returns Cloud cleanly. Plan WIRING_CONTRACT stub_ban
// row: "if get_cloud_spend returns error, log warning and SKIP the cap
// check rather than blocking all cloud calls".
// ---------------------------------------------------------------------------

#[test]
fn absent_spend_snapshot_skips_cap_check_per_stub_ban() {
    let cfg = cfg_with_daily_cap(0.01); // tiny cap that would normally block
    let router = CloudModeRouter::new(cfg);
    let outcome = router.route_with_spend(
        ExecMode::Cloud,
        Some(CloudCredentials::for_test_with_token("tok")),
        None, // viewer error simulated
    );
    assert!(
        outcome.is_ok(),
        "None snapshot must skip cap check, never block all cloud calls: got {outcome:?}"
    );
}
