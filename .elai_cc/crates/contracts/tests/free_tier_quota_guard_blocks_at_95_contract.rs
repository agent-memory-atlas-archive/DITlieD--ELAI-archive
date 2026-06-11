//! E-39 contract: free-tier quota guard blocks at 95% and the SSB row
//! carries `CostEventKind::FreeTierQuotaExceeded`.
//!
//! Drives `FreeTierGuard::check_quota` for `GroqFree` at 95% usage,
//! confirms `QuotaStatus::Exceeded`, then writes + reads back a
//! `CostTokenEvent { kind: FreeTierQuotaExceeded }` row. The SSB row
//! is the off-agent artifact.
//!
//! Production entrypoint: elai run -> drive_task -> cost_token_economics
//! path -> `free_tier_guard.check_quota` pre-dispatch gate
//! (task_runner.rs cloud-dispatch branch).

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::cost::free_tier_guard::{FreeTierGuard, QuotaStatus};
use elai_orchestrator::ssb::{
    CostEventKind, CostTokenEventPayload, Ssb, SsbActionType, SsbConfig, SsbPayload,
};
use elai_orchestrator::SsbReader;
use elai_providers::runtime::RuntimeBackend;
use tempfile::TempDir;

// WIRE-DARK[E-39]: free_tier_guard.check_quota has no production caller in
// task_runner.rs pre-dispatch gate yet (OPTION B deferred disjoint slice).
// Resolves when the guard is called in drive_task cloud-dispatch branch
// before dispatching to GroqFree/CerebrasFree. See E-39-decisions.md.
#[ignore]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_free_tier_quota_guard_blocks_at_95_ssb_roundtrip() {
    // 1. Drive the real FreeTierGuard.
    //    Groq default 14_400 req/day; 95% = 13_680.
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let mut guard = FreeTierGuard::with_defaults(clock.clone());
    guard.record(RuntimeBackend::GroqFree, 13_680, 0, 0);
    let status = guard
        .check_quota(RuntimeBackend::GroqFree)
        .expect("GroqFree must have a quota");
    assert_eq!(
        status,
        QuotaStatus::Exceeded,
        "GroqFree at 95% must return Exceeded"
    );

    // 2. Write the off-agent artifact SSB row.
    let tmp = TempDir::new().expect("tempdir");
    let cfg = SsbConfig {
        db_path: tmp.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: "e39-free-tier".into(),
        session_id: "e39-free-tier-session".into(),
    };
    let clock2: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(Ssb::new(cfg, clock2).expect("ssb"));

    let payload = SsbPayload::CostTokenEvent(CostTokenEventPayload {
        kind: CostEventKind::FreeTierQuotaExceeded,
        session_id: Some("e39-free-tier-session".into()),
        task_id: Some("task-freetier-001".into()),
        details_json: serde_json::json!({
            "backend": "groq_free",
            "status": status.as_str()
        }),
        ts_ms: 1_717_000_004_000,
    });
    ssb.write_payload("cost", 0, payload, Vec::new())
        .await
        .expect("write");

    // 3. Read back and assert.
    let rows = ssb
        .query_by_type(SsbActionType::CostTokenEvent)
        .expect("query");
    assert_eq!(rows.len(), 1, "expected exactly one CostTokenEvent row");
    let decoded: SsbPayload = rmp_serde::from_slice(&rows[0].payload).expect("decode");
    match decoded {
        SsbPayload::CostTokenEvent(p) => {
            assert_eq!(
                p.kind,
                CostEventKind::FreeTierQuotaExceeded,
                "kind discriminator must be FreeTierQuotaExceeded"
            );
            assert_eq!(
                p.details_json["backend"],
                serde_json::json!("groq_free"),
                "backend field must be 'groq_free'"
            );
            assert_eq!(
                p.details_json["status"],
                serde_json::json!("exceeded"),
                "status field must be 'exceeded'"
            );
        }
        other => panic!("expected CostTokenEvent, got {other:?}"),
    }
}
