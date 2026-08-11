//! L-4 slice 3 — semantic-entropy measurement receipt contract tests.
//!
//! Coverage: diverging receipt, agreeing receipt, silent unmeasured degrade, and
//! action-type registration/round-trip against a real temporary SSB database.

use std::sync::Arc;

use elai_orchestrator::ssb::{
    decode_ssb_payload, ssb_action_type_from_str_public, SemanticEntropyMeasuredKind, Ssb,
    SsbActionType, SsbConfig, SsbPayload, SsbReader, SCHEMA_VERSION_PUBLIC, SSB_ACTION_TYPES,
};

async fn build_ssb(session_id: &str) -> (Arc<Ssb>, tempfile::TempDir) {
    use elai_core::clock::OsClock;
    let proj = tempfile::TempDir::new().expect("project tempdir");
    let db_path = proj.path().join("ssb.db");
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(Ssb::new(SsbConfig {
        db_path,
        pool_size: Some(2),
        project_id: "l4-receipt-contract".into(),
        session_id: session_id.to_owned(),
    }, clock).expect("ssb open"));
    (ssb, proj)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn l4_receipt_row_lands_for_diverging_samples() {
    let (ssb, _proj) = build_ssb("l4-diverge").await;
    let samples = vec!["answer".into(), "canned".into(), "canned".into()];
    let result = elai_orchestrator::claim::semantic_entropy::emit_semantic_entropy_measured(
        &ssb, "l4-contract", 0, &samples).await;
    assert!(matches!(result, Ok(Some(_))));
    let rows = ssb.query_by_type(SsbActionType::SemanticEntropyMeasured).expect("query");
    assert_eq!(rows.len(), 1);
    match decode_ssb_payload(&rows[0].payload).expect("decode") {
        SsbPayload::SemanticEntropyMeasured(p) => {
            assert_eq!(p.kind, SemanticEntropyMeasuredKind::SeedsDiverge);
            assert_eq!(p.sample_count, 3); assert_eq!(p.cluster_count, 2);
            assert_eq!(p.entropy_milli, 579); assert_eq!(p.sample_digests.len(), 3);
            assert_eq!(p.sample_digests[1], p.sample_digests[2]);
            assert_ne!(p.sample_digests[0], p.sample_digests[1]);
        }
        other => panic!("unexpected payload: {other:?}"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn l4_receipt_row_lands_for_agreeing_samples() {
    let (ssb, _proj) = build_ssb("l4-agree").await;
    let samples = vec!["a".into(), "a".into(), "a".into()];
    assert!(matches!(elai_orchestrator::claim::semantic_entropy::emit_semantic_entropy_measured(
        &ssb, "l4-contract", 0, &samples).await, Ok(Some(_))));
    let rows = ssb.query_by_type(SsbActionType::SemanticEntropyMeasured).expect("query");
    match decode_ssb_payload(&rows[0].payload).expect("decode") {
        SsbPayload::SemanticEntropyMeasured(p) => {
            assert_eq!(p.kind, SemanticEntropyMeasuredKind::SeedsAgree);
            assert_eq!(p.sample_count, 3); assert_eq!(p.cluster_count, 1);
            assert_eq!(p.entropy_milli, 0); assert_eq!(p.sample_digests.len(), 3);
            assert_eq!(p.sample_digests[0], p.sample_digests[1]);
            assert_eq!(p.sample_digests[1], p.sample_digests[2]);
        }
        other => panic!("unexpected payload: {other:?}"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn l4_unmeasured_sample_set_emits_no_row() {
    let (ssb, _proj) = build_ssb("l4-single").await;
    let samples = vec!["only one answer".into()];
    assert!(matches!(elai_orchestrator::claim::semantic_entropy::emit_semantic_entropy_measured(
        &ssb, "l4-contract", 0, &samples).await, Ok(None)));
    let rows = ssb.query_by_type(SsbActionType::SemanticEntropyMeasured).expect("query");
    assert!(rows.is_empty(), "unmeasured singleton must emit no false zero-entropy measured row");
}

#[test]
fn l4_semantic_entropy_measured_action_type_round_trips() {
    let tag = "semantic_entropy_measured";
    let parsed = ssb_action_type_from_str_public(tag).expect("tag parses");
    assert_eq!(parsed, SsbActionType::SemanticEntropyMeasured);
    assert_eq!(parsed.as_sql_str(), tag);
    assert!(SSB_ACTION_TYPES.len() >= 612);
    assert!(SCHEMA_VERSION_PUBLIC >= 303);
    assert_eq!(SSB_ACTION_TYPES.iter().filter(|a| **a == parsed).count(), 1);
}
