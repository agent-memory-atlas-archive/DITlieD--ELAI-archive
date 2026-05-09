//! B-19A T10 — hook latency budget enforcement.
//!
//! Verifies:
//! 1. The HOOK_T1/T2/T3_BUDGET_MS constants exist with the spec values
//!    (T1<=1ms, T2<=10ms, T3<=5ms).
//! 2. `emit_hook_latency_violation_to_ssb` writes exactly one
//!    `hook_latency_violation` row whose `HookLatencyViolationPayload`
//!    carries the supplied (hook_name, tier, budget_ms, actual_ms).
//! 3. `report_hook_latency` writes ZERO rows when within budget and
//!    exactly one row on overrun.
//! 4. Per-tier independent enforcement: emitting violations for T1, T2,
//!    and T3 separately produces three rows whose `tier` field carries
//!    the correct label and whose `budget_ms` matches the per-tier
//!    constant. (Cycle 2: dispatcher-side per-tier enforcement is
//!    deferred to B-19B because `TieredAnalysis::analyze` returns one
//!    composite `duration_ms` only — the SSB emission helpers used here
//!    already support per-tier rows so callers with per-tier timings
//!    work correctly.)
//!
//! These tests do not exercise the live dispatcher's analyze() path
//! (that needs a TieredAnalysis backend). They exercise the SSB-side
//! emission contract that the dispatcher uses on every entry/exit.

use std::sync::Arc;

use elai_core::clock::{ClockProvider, SeededClock};
use elai_orchestrator::hook_dispatcher::{
    emit_hook_latency_violation_to_ssb, report_hook_latency, HookTier, HOOK_T1_BUDGET_MS,
    HOOK_T2_BUDGET_MS, HOOK_T3_BUDGET_MS,
};
use elai_orchestrator::ssb::{Ssb, SsbActionType, SsbConfig, SsbPayload, SsbReader};
use tempfile::TempDir;

fn make_ssb(dir: &TempDir) -> Ssb {
    let cfg = SsbConfig {
        db_path: dir.path().join("hook_latency.db"),
        pool_size: Some(2),
        project_id: "test-proj".into(),
        session_id: "test-sess".into(),
    };
    let clock: Arc<dyn ClockProvider> = Arc::new(SeededClock::new());
    Ssb::new(cfg, clock).expect("ssb init")
}

#[test]
fn budget_constants_match_spec() {
    assert_eq!(HOOK_T1_BUDGET_MS, 1, "T1 budget must be 1ms per B-19 spec");
    assert_eq!(HOOK_T2_BUDGET_MS, 10, "T2 budget must be 10ms per B-19 spec");
    assert_eq!(HOOK_T3_BUDGET_MS, 5, "T3 budget must be 5ms per B-19 spec");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn overrun_emits_one_hook_latency_violation_row() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = Arc::new(make_ssb(&dir));

    let written = emit_hook_latency_violation_to_ssb(
        &ssb,
        "agent-test",
        0,
        "PreToolUse",
        HookTier::T1,
        HOOK_T1_BUDGET_MS,
        12,
    )
    .await;
    assert!(written, "violation row should write successfully");

    let rows = ssb
        .query_by_type(SsbActionType::HookLatencyViolation)
        .expect("query");
    assert_eq!(rows.len(), 1, "must have exactly one violation row");
    let entry = &rows[0];
    assert_eq!(entry.action_type, SsbActionType::HookLatencyViolation);

    let payload: SsbPayload = rmp_serde::from_slice(&entry.payload).expect("decode payload");
    let SsbPayload::HookLatencyViolation(p) = payload else {
        panic!("expected HookLatencyViolation payload variant");
    };
    assert_eq!(p.hook_name, "PreToolUse");
    assert_eq!(p.tier, "T1");
    assert_eq!(p.budget_ms, HOOK_T1_BUDGET_MS);
    assert_eq!(p.actual_ms, 12);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn within_budget_emits_no_violation_row() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = Arc::new(make_ssb(&dir));

    // T2 budget 10ms; report 4ms — under budget.
    let emitted = report_hook_latency(
        Some(&ssb),
        "agent-test",
        0,
        "PostToolUse",
        HookTier::T2,
        4,
    )
    .await;
    assert!(!emitted, "within-budget path must not emit a violation");

    let rows = ssb
        .query_by_type(SsbActionType::HookLatencyViolation)
        .expect("query");
    assert!(rows.is_empty(), "within-budget call must write no rows");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn report_hook_latency_overrun_emits_violation() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = Arc::new(make_ssb(&dir));

    // T3 budget 5ms; report 25ms — overrun.
    let emitted = report_hook_latency(
        Some(&ssb),
        "agent-test",
        0,
        "Stop",
        HookTier::T3,
        25,
    )
    .await;
    assert!(emitted, "overrun must emit a violation row");

    let rows = ssb
        .query_by_type(SsbActionType::HookLatencyViolation)
        .expect("query");
    assert_eq!(rows.len(), 1, "exactly one violation row on overrun");
}

/// B-19A cycle 2: per-tier independent enforcement at the SSB layer.
///
/// Dispatcher-side per-tier enforcement is structurally blocked until
/// `TieredAnalysis::analyze` grows per-tier `duration_ms` fields (deferred
/// to B-19B). Until then, the only callers that can drive per-tier rows are
/// instrumented runners that measure each tier individually. This test
/// proves the SSB side already accepts per-tier rows — when the dispatcher
/// gains per-tier timings, no SSB-side change will be required.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn per_tier_overruns_emit_independent_rows() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = Arc::new(make_ssb(&dir));

    // Each tier overruns its own budget by a clear margin.
    assert!(
        report_hook_latency(Some(&ssb), "agent-test", 0, "PreToolUse", HookTier::T1, 5).await
    );
    assert!(
        report_hook_latency(Some(&ssb), "agent-test", 0, "PostToolUse", HookTier::T2, 50).await
    );
    assert!(
        report_hook_latency(Some(&ssb), "agent-test", 0, "Stop", HookTier::T3, 25).await
    );

    let rows = ssb
        .query_by_type(SsbActionType::HookLatencyViolation)
        .expect("query");
    assert_eq!(rows.len(), 3, "expected one row per tier overrun");

    // Decode each payload and assert tier + budget_ms align.
    let mut tiers: Vec<(String, u64)> = rows
        .iter()
        .map(|r| {
            let payload: SsbPayload = rmp_serde::from_slice(&r.payload).expect("decode");
            let SsbPayload::HookLatencyViolation(p) = payload else {
                panic!("expected HookLatencyViolation");
            };
            (p.tier, p.budget_ms)
        })
        .collect();
    tiers.sort();
    assert_eq!(
        tiers,
        vec![
            ("T1".to_string(), HOOK_T1_BUDGET_MS),
            ("T2".to_string(), HOOK_T2_BUDGET_MS),
            ("T3".to_string(), HOOK_T3_BUDGET_MS),
        ]
    );
}
