//! Phase A4 — dashboard apply_proposal_from_user / reject_proposal contract.
//!
//! Pins the underlying [`Layer1aProcessor::apply_proposal`] +
//! [`Layer1aProcessor::reject_proposal`] behaviour that the dashboard's
//! new commands now delegate to (instead of writing decorative SSB rows
//! directly).
//!
//! Mirrors the fixture from `layer1a_contract.rs:34-68` so the assertions
//! exercise the same code path the dashboard hits at runtime.

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_orchestrator::{
    DefaultLayer1aProcessor, FailureRecord, Layer1aProcessor, PipelineMode, PipelineOutcome,
    Ssb, SsbActionType, SsbConfig,
};
use rusqlite::Connection;
use tokio::sync::Mutex as AsyncMutex;

// ---------------------------------------------------------------------------
// Helpers — open a rules.db + SSB pair under a tempdir, build a processor.
// ---------------------------------------------------------------------------

struct Fixture {
    _tmp: tempfile::TempDir,
    processor: Arc<DefaultLayer1aProcessor>,
    rules_db_path: PathBuf,
    ssb_path: PathBuf,
}

async fn fixture() -> Fixture {
    let tmp = tempfile::tempdir().expect("tempdir");
    let rules_db_path = tmp.path().join("rules.db");
    let conn = Connection::open(&rules_db_path).expect("open rules.db");
    let rules_db = Arc::new(AsyncMutex::new(conn));
    let ssb_path = tmp.path().join("ssb.db");
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let ssb = Ssb::new(
        SsbConfig {
            db_path: ssb_path.clone(),
            pool_size: Some(2),
            project_id: "proj-test".to_owned(),
            session_id: "sess-test".to_owned(),
        },
        clock,
    )
    .expect("ssb init");
    let processor =
        DefaultLayer1aProcessor::new(rules_db, rules_db_path.clone(), Arc::new(ssb))
            .with_tuning(3, 0.5, 50, 0.3);
    processor.ensure_tables().await.expect("ensure_tables");
    Fixture {
        _tmp: tmp,
        processor: Arc::new(processor),
        rules_db_path,
        ssb_path,
    }
}

fn failure(category: &str, pattern: &str, occurrences: u32) -> FailureRecord {
    FailureRecord {
        category: category.to_owned(),
        pattern: pattern.to_owned(),
        occurrences,
        is_false_positive: false,
    }
}

fn outcome_with_failures(failures: Vec<FailureRecord>) -> PipelineOutcome {
    PipelineOutcome {
        task_id: "task-a4".to_owned(),
        success: false,
        failures,
        findings: vec![],
        pipeline_mode: PipelineMode::Surgical,
        elapsed_ms: 42,
    }
}

/// Direct SQLite read of SSB rows matching one action_type. The SSB schema
/// stores `action_type` as the canonical lowercase string
/// ("proposal_applied" / "proposal_rejected").
fn count_ssb_rows(ssb_path: &PathBuf, action: &str) -> u32 {
    let conn = Connection::open(ssb_path).expect("open ssb.db");
    conn.query_row(
        "SELECT COUNT(*) FROM ssb WHERE action_type = ?1",
        rusqlite::params![action],
        |row| row.get(0),
    )
    .unwrap_or(0)
}

fn count_applied_rows(rules_db_path: &PathBuf, proposal_id: &str) -> u32 {
    let conn = Connection::open(rules_db_path).expect("open rules.db");
    conn.query_row(
        "SELECT COUNT(*) FROM applied_proposals WHERE proposal_id = ?1",
        rusqlite::params![proposal_id],
        |row| row.get(0),
    )
    .unwrap_or(0)
}

fn proposal_status(rules_db_path: &PathBuf, proposal_id: &str) -> String {
    let conn = Connection::open(rules_db_path).expect("open rules.db");
    conn.query_row(
        "SELECT status FROM config_proposals WHERE proposal_id = ?1",
        rusqlite::params![proposal_id],
        |row| row.get::<_, String>(0),
    )
    .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Contract 1 — apply_proposal: dashboard delegates to Layer1A, which writes
// applied_proposals row + emits ProposalApplied SSB row.
//
// The dashboard's `apply_proposal_from_user` now calls
//   processor.list_pending() -> find -> processor.apply_proposal(&p)
// so this test pins the underlying behaviour the command depends on.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_apply_proposal_writes_applied_row_and_emits_ssb() {
    let f = fixture().await;

    // Seed: produce a pending proposal via the canonical entry point.
    let outcome = outcome_with_failures(vec![failure("logic", "unimpl_a4_apply", 3)]);
    let proposals = f.processor.process_outcome(&outcome).await;
    assert_eq!(proposals.len(), 1, "fixture must produce one proposal");
    let proposal = proposals[0].clone();
    let proposal_id = proposal.proposal_id.clone();

    // Sanity: appears in list_pending — same call the dashboard will make.
    let pending_before = f.processor.list_pending().await;
    assert!(
        pending_before.iter().any(|p| p.proposal_id == proposal_id),
        "proposal must be pending before apply"
    );

    // The dashboard path: list_pending().find(|p| ...).apply_proposal(&p)
    let target = pending_before
        .into_iter()
        .find(|p| p.proposal_id == proposal_id)
        .expect("proposal in pending list");
    let outcome = f
        .processor
        .apply_proposal(&target)
        .await
        .expect("apply ok");
    assert_eq!(outcome.proposal_id, proposal_id);

    // Post-conditions on rules.db.
    assert_eq!(
        count_applied_rows(&f.rules_db_path, &proposal_id),
        1,
        "applied_proposals must have one row for this proposal_id"
    );
    assert_eq!(
        proposal_status(&f.rules_db_path, &proposal_id),
        "applied",
        "config_proposals.status must flip to 'applied'"
    );

    // Post-condition on SSB: Layer1A emitted a ProposalApplied row internally.
    let applied_rows =
        count_ssb_rows(&f.ssb_path, SsbActionType::ProposalApplied.as_sql_str());
    assert!(
        applied_rows >= 1,
        "Layer1A must emit at least one ProposalApplied SSB row, got {applied_rows}"
    );

    // Post-condition: proposal disappears from pending after apply.
    let pending_after = f.processor.list_pending().await;
    assert!(
        !pending_after.iter().any(|p| p.proposal_id == proposal_id),
        "applied proposal must leave pending list"
    );
}

// ---------------------------------------------------------------------------
// Contract 2 — reject_proposal: dashboard delegates to Layer1A, which flips
// status to 'rejected' and emits ProposalRejected SSB row.
//
// The dashboard's `reject_proposal` now calls
//   processor.reject_proposal(&proposal_id, "rejected by operator")
// directly (it does NOT need to look the proposal up first — the reject
// signature is (proposal_id, reason)).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_reject_proposal_flips_status_and_emits_ssb() {
    let f = fixture().await;

    let outcome = outcome_with_failures(vec![failure("logic", "unimpl_a4_reject", 3)]);
    let proposals = f.processor.process_outcome(&outcome).await;
    assert_eq!(proposals.len(), 1);
    let proposal_id = proposals[0].proposal_id.clone();

    f.processor
        .reject_proposal(&proposal_id, "rejected by operator")
        .await
        .expect("reject ok");

    assert_eq!(
        proposal_status(&f.rules_db_path, &proposal_id),
        "rejected",
        "config_proposals.status must flip to 'rejected'"
    );

    let rejected_rows =
        count_ssb_rows(&f.ssb_path, SsbActionType::ProposalRejected.as_sql_str());
    assert!(
        rejected_rows >= 1,
        "Layer1A must emit at least one ProposalRejected SSB row, got {rejected_rows}"
    );

    // Rejected proposal disappears from pending list.
    let pending_after = f.processor.list_pending().await;
    assert!(
        !pending_after.iter().any(|p| p.proposal_id == proposal_id),
        "rejected proposal must leave pending list"
    );
}
