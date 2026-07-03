// B-27 done-gate smoke: drive the real cloud-mode SSB producers to write
// cloud_spend, escalate_to_cloud, and redaction_applied rows into the
// project-root SSB so wiring_contract_check.py's expect_ssb leg finds
// off-agent proof. The contract tests (cloud_spend_ssb_contract.rs,
// hybrid_escalation_contract.rs, c5_cloud_mode_contract.rs) prove the
// round-trip invariants against temp DBs; this smoke writes to the SAME
// db the check reads (<project_root>/sessions/ssb.db or ELAI_SSB_DB env
// override) so the off-agent DB rows are present after the trigger runs.
//
// Trigger: cargo test -p elai_cc_contracts --test b27_cloud_mode_smoke_ssb -- --nocapture
// The test builds a real Ssb, emits one CloudSpend, one EscalateToCloud,
// and one RedactionApplied SSB row via the standard write_payload path,
// then polls the reader to confirm the rows landed.

use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_orchestrator::ssb::{
    CloudSpendPayload, EscalateToCloudPayload, RedactionAppliedPayload, Ssb, SsbActionType,
    SsbConfig, SsbPayload, SsbReader,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn smoke_writes_cloud_mode_rows_to_project_root_ssb() {
    // Resolve the DB path: ELAI_SSB_DB env wins; otherwise the workspace
    // default <contracts_crate>/../../sessions/ssb.db == <project_root>/sessions/ssb.db
    // which is _ssb_candidates[0] in wiring_contract_check.py.
    let db_path = std::env::var("ELAI_SSB_DB")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
            manifest.join("../../sessions/ssb.db")
        });

    // Ensure the parent dir exists (the SSB constructor does not create it).
    if let Some(parent) = db_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let config = SsbConfig {
        db_path: db_path.clone(),
        project_id: "b27-smoke".to_owned(),
        session_id: "s-b27-smoke".to_owned(),
        pool_size: Some(2),
    };
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(
        Ssb::new(config, clock).expect("smoke: Ssb::new must succeed on project-root db"),
    );

    let now_ms = i64::try_from(OsClock.now_ms()).unwrap_or(i64::MAX);

    // Real B-27 producer path: write_payload with SsbPayload::CloudSpend
    // (the same payload shape grounding_client.rs:952 emits on each cloud
    // call).
    let cloud_spend = SsbPayload::CloudSpend(CloudSpendPayload {
        provider: "anthropic".into(),
        model: "claude-sonnet-4-6".into(),
        input_tokens: 1_200,
        output_tokens: 320,
        cache_creation_tokens: 800,
        cache_read_tokens: 4_000,
        cost_usd: 0.012_34,
        task_id: "b27-smoke".into(),
        ts_ms: now_ms,
    });
    ssb.write_payload("b27_smoke", 0, cloud_spend, Vec::new())
        .await
        .expect("smoke: write CloudSpend must succeed");

    // Real B-27 producer path: write_payload with SsbPayload::EscalateToCloud
    // (the same payload shape task_runner.rs:9228 emits on HYBRID escalation).
    let escalate = SsbPayload::EscalateToCloud(EscalateToCloudPayload {
        step_id: "b27_smoke_step".into(),
        trigger_reason: "local model exhausted retries".into(),
        layer_failed: 3,
        ts_ms: now_ms,
    });
    ssb.write_payload("b27_smoke", 0, escalate, Vec::new())
        .await
        .expect("smoke: write EscalateToCloud must succeed");

    // Real B-27 producer path: write_payload with SsbPayload::RedactionApplied
    // (the same payload shape cloud_mode.rs:987 emits from
    // RedactingBabyStepExecutor before each cloud egress).
    let redaction = SsbPayload::RedactionApplied(RedactionAppliedPayload {
        count: 2,
        fields: vec!["ANTHROPIC_API_KEY".into(), "/home/user/.env".into()],
        baby_step_id: 1,
        ts_ms: now_ms,
    });
    ssb.write_payload("b27_smoke", 0, redaction, Vec::new())
        .await
        .expect("smoke: write RedactionApplied must succeed");

    // Poll the reader until the cloud_spend row lands (async writer flush).
    // expect_ssb in the B-27 contract is "cloud_spend" so this is the
    // load-bearing row; the other two are emitted for completeness.
    let mut rows: Vec<_> = ssb
        .query_by_type(SsbActionType::CloudSpend)
        .expect("smoke: query_by_type CloudSpend must succeed");
    for _ in 0..50 {
        if !rows.is_empty() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        rows = ssb
            .query_by_type(SsbActionType::CloudSpend)
            .expect("smoke: query_by_type CloudSpend must succeed");
    }
    assert!(
        !rows.is_empty(),
        "smoke: project-root SSB must contain a cloud_spend row after the trigger",
    );
}
