//! B-27 — Cloud spend SSB round-trip + bijection contract tests.
//!
//! Covers the new SsbActionType variants introduced by B-27:
//!   1. CloudSpend payload round-trips through msgpack + SQL CHECK constraint.
//!   2. CloudSpend action_type_str bijection (as_sql_str + ssb_action_type_from_str_public).
//!   3. EscalateToCloud payload round-trips through msgpack + SQL CHECK constraint.
//!   4. EscalateToCloud action_type_str bijection.
//!
//! These tests fail-to-compile until B-27 EXECUTING step 2 lands the
//! enum variants, payload structs, as_sql_str arms, ssb_action_type_from_str
//! arms, and CHECK(action_type IN (...)) constraint update with SCHEMA_VERSION
//! bump 26 -> 27.
//!
//! Cross-language pair guard: web3_invariants.py enforces the four-way
//! bijection (enum, as_sql_str, ssb_action_type_from_str, CHECK constraint)
//! at audit time; these contract tests catch the same drift at unit-test time.

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, CloudSpendPayload, EscalateToCloudPayload, Ssb, SsbActionType,
    SsbConfig, SsbPayload,
};
use elai_orchestrator::SsbReader;
use tempfile::TempDir;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_cloud_spend_payload_roundtrips() {
    let tmp = TempDir::new().expect("tempdir");
    let cfg = SsbConfig {
        db_path: tmp.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: "b27-cloud-spend".into(),
        session_id: "b27-cloud-spend-session".into(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(Ssb::new(cfg, clock).expect("ssb"));

    let payload = SsbPayload::CloudSpend(CloudSpendPayload {
        provider: "anthropic".into(),
        model: "claude-sonnet-4-6".into(),
        input_tokens: 1_200,
        output_tokens: 320,
        cache_creation_tokens: 800,
        cache_read_tokens: 4_000,
        cost_usd: 0.012_34,
        task_id: "task-7".into(),
        ts_ms: 1_715_000_000_000,
    });
    assert_eq!(payload.action_type(), SsbActionType::CloudSpend);

    ssb.write_payload("cloud_mode", 0, payload.clone(), Vec::new())
        .await
        .expect("write");

    let rows = ssb
        .query_by_type(SsbActionType::CloudSpend)
        .expect("query");
    assert_eq!(rows.len(), 1);
    let decoded: SsbPayload = rmp_serde::from_slice(&rows[0].payload).expect("decode");
    match decoded {
        SsbPayload::CloudSpend(p) => {
            assert_eq!(p.provider, "anthropic");
            assert_eq!(p.model, "claude-sonnet-4-6");
            assert_eq!(p.input_tokens, 1_200);
            assert_eq!(p.output_tokens, 320);
            assert_eq!(p.cache_creation_tokens, 800);
            assert_eq!(p.cache_read_tokens, 4_000);
            assert!((p.cost_usd - 0.012_34).abs() < 1e-9);
            assert_eq!(p.task_id, "task-7");
            assert_eq!(p.ts_ms, 1_715_000_000_000);
        }
        other => panic!("expected CloudSpend, got {other:?}"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_escalate_to_cloud_payload_roundtrips() {
    let tmp = TempDir::new().expect("tempdir");
    let cfg = SsbConfig {
        db_path: tmp.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: "b27-escalate".into(),
        session_id: "b27-escalate-session".into(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(Ssb::new(cfg, clock).expect("ssb"));

    let payload = SsbPayload::EscalateToCloud(EscalateToCloudPayload {
        step_id: "baby_step_42".into(),
        trigger_reason: "local model exhausted retries".into(),
        layer_failed: 1,
        ts_ms: 1_715_000_001_000,
    });
    assert_eq!(payload.action_type(), SsbActionType::EscalateToCloud);

    ssb.write_payload("cloud_mode", 0, payload.clone(), Vec::new())
        .await
        .expect("write");

    let rows = ssb
        .query_by_type(SsbActionType::EscalateToCloud)
        .expect("query");
    assert_eq!(rows.len(), 1);
    let decoded: SsbPayload = rmp_serde::from_slice(&rows[0].payload).expect("decode");
    match decoded {
        SsbPayload::EscalateToCloud(p) => {
            assert_eq!(p.step_id, "baby_step_42");
            assert_eq!(p.trigger_reason, "local model exhausted retries");
            assert_eq!(p.layer_failed, 1);
            assert_eq!(p.ts_ms, 1_715_000_001_000);
        }
        other => panic!("expected EscalateToCloud, got {other:?}"),
    }
}

#[test]
fn contract_cloud_spend_action_type_string_bijection() {
    assert_eq!(SsbActionType::CloudSpend.as_sql_str(), "cloud_spend");
    assert_eq!(
        ssb_action_type_from_str_public("cloud_spend").expect("parse"),
        SsbActionType::CloudSpend
    );
}

#[test]
fn contract_escalate_to_cloud_action_type_string_bijection() {
    assert_eq!(
        SsbActionType::EscalateToCloud.as_sql_str(),
        "escalate_to_cloud"
    );
    assert_eq!(
        ssb_action_type_from_str_public("escalate_to_cloud").expect("parse"),
        SsbActionType::EscalateToCloud
    );
}
