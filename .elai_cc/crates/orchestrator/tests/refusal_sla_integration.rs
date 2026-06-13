//! D-17 — observable_effect integration smoke for the honest-refusal SLA
//! monitor.
//!
//! Builds a `RefusalRateMonitor` with the default healthy band, observes
//! 100 [`RefusalDecision`] rows (50 Abstain, 50 Answer = 50% refusal
//! rate — far above `healthy_band_hi = 0.20`), runs
//! `compute_snapshot`, asserts:
//!
//!   1. `HealthBand::Above { gap = 0.30 }` is classified.
//!   2. `sla_breach(&snapshot)` returns `true`.
//!   3. emitting the snapshot lands one `RefusalRateSnapshot` SSB
//!      payload + one `RefusalSlaBreach` SSB payload on the recording
//!      emitters (the wire signal the downstream Bubble-Up alert
//!      dispatcher subscribes to).
//!
//! Grounding: arXiv:2503.01345 (UniCR honest refusal); docs/specs/
//! guide/real-problem-solving-grounded-2026-05-19.md §4.6 (refusal
//! rate as a first-class quality metric).

use elai_orchestrator::{
    agent_pool::dispatch_refusal_sla_check,
    calibration::RefusalDecision,
    refusal_sla::{
        sla_breach, HealthBand, RefusalRateMonitor, VecRefusalRateSnapshotEmitter,
        VecRefusalSlaBreachEmitter,
    },
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dispatch_refusal_sla_check_classifies_above_and_emits_two_rows() {
    let mut monitor = RefusalRateMonitor::with_defaults();
    // 50 abstain + 50 answer = 50% refusal rate.
    for _ in 0..50 {
        monitor.observe(RefusalDecision::Abstain);
    }
    for _ in 0..50 {
        monitor.observe(RefusalDecision::Answer { confidence: 0.9 });
    }

    // Pure compute first — verifies the classification before the
    // emitter producer even runs.
    let pure = monitor.compute_snapshot(1_700_000_000_000);
    assert_eq!(pure.window_total, 100);
    assert_eq!(pure.abstain_count, 50);
    assert!((pure.refusal_rate - 0.5).abs() < 1e-9);
    match pure.health_band {
        HealthBand::Above { gap } => {
            // healthy_band_hi default = 0.20; rate 0.50 → gap = 0.30.
            assert!((gap - 0.30).abs() < 1e-9, "gap = {gap}");
        }
        other => panic!("expected Above; got {other:?}"),
    }
    assert!(sla_breach(&pure));

    // Now the producer surface: dispatch through agent_pool.
    let snap_em = VecRefusalRateSnapshotEmitter::new();
    let breach_em = VecRefusalSlaBreachEmitter::new();
    let snap = dispatch_refusal_sla_check(
        &monitor,
        &snap_em,
        &breach_em,
        1_700_000_000_000,
    )
    .await;
    assert!(matches!(snap.health_band, HealthBand::Above { .. }));

    let snap_rows = snap_em.rows().await;
    assert_eq!(snap_rows.len(), 1);
    let s = &snap_rows[0];
    assert_eq!(s.window_total, 100);
    assert_eq!(s.abstain_count, 50);
    assert!((s.refusal_rate - 0.5).abs() < 1e-9);
    assert_eq!(s.health_band, "above");
    assert_eq!(s.ts_ms, 1_700_000_000_000);

    let breach_rows = breach_em.rows().await;
    assert_eq!(breach_rows.len(), 1);
    let b = &breach_rows[0];
    assert!((b.rate - 0.5).abs() < 1e-9);
    assert!((b.threshold - 0.20).abs() < 1e-9);
    assert_eq!(b.breach_kind, "above");
    assert_eq!(b.ts_ms, 1_700_000_000_000);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dispatch_refusal_sla_check_healthy_band_emits_only_snapshot() {
    let mut monitor = RefusalRateMonitor::with_defaults();
    // 10/90 → 10% inside healthy band [5%, 20%].
    for _ in 0..10 {
        monitor.observe(RefusalDecision::Abstain);
    }
    for _ in 0..90 {
        monitor.observe(RefusalDecision::Answer { confidence: 0.9 });
    }

    let snap_em = VecRefusalRateSnapshotEmitter::new();
    let breach_em = VecRefusalSlaBreachEmitter::new();
    let snap = dispatch_refusal_sla_check(
        &monitor,
        &snap_em,
        &breach_em,
        1_700_000_001_000,
    )
    .await;

    assert!(matches!(snap.health_band, HealthBand::Healthy));
    assert!(!sla_breach(&snap));
    assert_eq!(snap_em.len().await, 1);
    assert_eq!(breach_em.len().await, 0);
}
