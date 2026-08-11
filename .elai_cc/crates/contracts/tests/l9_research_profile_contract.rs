//! L-9 slice 5 — research grounding producer contract tests.
//!
//! Coverage: an unverified derivation abstains and lands exactly one
//! `ClaimRiskScored` row with the abstention marking on the wire; an empty
//! reply writes no row; divergent candidates still land the row with the
//! abstain marking and a non-empty predictor set.

use std::sync::Arc;

use elai_orchestrator::ssb::{
    decode_ssb_payload, ClaimRiskScoredKind, Ssb, SsbActionType, SsbConfig, SsbPayload,
};
use elai_orchestrator::SsbReader;

async fn build_ssb(session_id: &str) -> (Arc<Ssb>, tempfile::TempDir) {
    use elai_core::clock::OsClock;
    let proj = tempfile::TempDir::new().expect("project tempdir");
    let db_path = proj.path().join("ssb.db");
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(
        Ssb::new(
            SsbConfig {
                db_path,
                pool_size: Some(2),
                project_id: "l9-research-contract".into(),
                session_id: session_id.to_owned(),
            },
            clock,
        )
        .expect("ssb open"),
    );
    (ssb, proj)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn l9_unverified_derivation_abstains_and_lands_one_row() {
    let (ssb, _proj) = build_ssb("l9-abstain").await;
    let reply = "the change lands in `crates/cli/src/live_cli.rs`\nthe derivation gives 2 + 2 = 5 as the total\n";
    let candidates: Vec<String> = vec![
        "the change lands in `crates/cli/src/live_cli.rs`\nthe derivation gives 2 + 2 = 5 as the total\n".into(),
        "a materially different candidate answer".into(),
    ];
    let outcome = elai_orchestrator::reasoning::research_grounding_pass(
        &ssb,
        "l9-contract",
        0,
        reply,
        &candidates,
    )
    .await
    .expect("grounding pass ok");

    assert!(outcome.row_written, "row_written must be true");
    assert!(
        outcome
            .verdicts
            .iter()
            .any(|v| matches!(v.verdict, elai_orchestrator::reasoning::ResearchVerdict::Abstained(_))),
        "at least one verdict must be Abstained; an Asserted verdict is the TD-3/HG-4 defect"
    );

    let rows = ssb
        .query_by_type(SsbActionType::ClaimRiskScored)
        .expect("query");
    assert_eq!(rows.len(), 1, "exactly one ClaimRiskScored row");
    match decode_ssb_payload(&rows[0].payload).expect("decode") {
        SsbPayload::ClaimRiskScored(p) => {
            assert!(p.high_count >= 1, "high_count >= 1");
            assert_ne!(
                p.kind,
                ClaimRiskScoredKind::LowRisk,
                "kind must not be LowRisk when an abstention landed"
            );
            assert_eq!(
                p.claim_count,
                p.claim_digests.len(),
                "claim_count == claim_digests.len()"
            );
        }
        other => panic!("unexpected payload: {other:?}"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn l9_empty_reply_writes_no_row() {
    let (ssb, _proj) = build_ssb("l9-empty").await;
    let reply = "plain prose with no refs or derivations";
    let candidates: Vec<String> = vec![reply.to_string()];
    let outcome = elai_orchestrator::reasoning::research_grounding_pass(
        &ssb,
        "l9-contract",
        0,
        reply,
        &candidates,
    )
    .await
    .expect("grounding pass ok");

    assert!(!outcome.row_written, "row_written must be false");
    assert!(outcome.verdicts.is_empty(), "verdicts must be empty");
    let rows = ssb
        .query_by_type(SsbActionType::ClaimRiskScored)
        .expect("query");
    assert_eq!(rows.len(), 0, "no row for an empty reply");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn l9_divergent_candidates_still_land_the_row() {
    let (ssb, _proj) = build_ssb("l9-divergent").await;
    let reply = "the change lands in `crates/cli/src/live_cli.rs`\nthe derivation gives 2 + 2 = 5 as the total\n";
    let candidates: Vec<String> = vec!["answer".into(), "canned".into(), "canned".into()];
    let outcome = elai_orchestrator::reasoning::research_grounding_pass(
        &ssb,
        "l9-contract",
        0,
        reply,
        &candidates,
    )
    .await
    .expect("grounding pass ok");

    assert!(outcome.row_written, "row_written must be true");
    let rows = ssb
        .query_by_type(SsbActionType::ClaimRiskScored)
        .expect("query");
    assert_eq!(rows.len(), 1, "exactly one ClaimRiskScored row");
    match decode_ssb_payload(&rows[0].payload).expect("decode") {
        SsbPayload::ClaimRiskScored(p) => {
            assert!(!p.predictors.is_empty(), "predictors must not be empty");
            assert!(p.high_count >= 1, "high_count >= 1");
        }
        other => panic!("unexpected payload: {other:?}"),
    }
}
