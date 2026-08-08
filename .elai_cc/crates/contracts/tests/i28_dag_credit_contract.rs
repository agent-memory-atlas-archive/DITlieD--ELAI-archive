//! I-28 Slice 1 contract tests for deterministic branch-grain DAG credit.
//!
//! No `unsafe` env mutation (maintainability MR-2). Env gate is default-OFF when
//! unset; emit_dag_credit_to_ssb is the production write path (same fn as
//! task_runner::dispatch_dag_credit).

use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_orchestrator::dag_credit::{
    dag_credit_enabled, emit_dag_credit_to_ssb, DagCreditAssigner,
};
use elai_orchestrator::search::{RankedBranch, SearchLadderOutcome};
use elai_orchestrator::ssb::{
    decode_ssb_payload, ssb_action_type_from_str_public, DagCreditAssignedKind,
    SsbActionType, SsbConfig, SsbPayload, SCHEMA_VERSION_PUBLIC, TriLayerVerdictKind,
};
use elai_orchestrator::verifier_stack::VerdictRow;
use elai_orchestrator::SsbReader;
use tempfile::TempDir;

const I28_SCHEMA_FLOOR: i64 = 297;

fn test_ssb(dir: &TempDir) -> Arc<elai_orchestrator::ssb::Ssb> {
    let config = SsbConfig {
        db_path: dir.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: "i28-contract".into(),
        session_id: "i28-session".into(),
    };
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    Arc::new(elai_orchestrator::ssb::Ssb::new(config, clock).expect("ssb init"))
}

fn ranked_branch(branch_id: &str, score: f64) -> RankedBranch {
    RankedBranch {
        verdict: VerdictRow {
            branch_id: branch_id.into(),
            verdict: TriLayerVerdictKind::ScoredSurvivor,
            score,
            gate: None,
            l2_call_count: 1,
        },
        mutation_kill_rate: 0.0,
        progress_score: 0.0,
    }
}

#[test]
fn schema_and_action_type() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= I28_SCHEMA_FLOOR,
        "I-28 schema must stay at or above the pre-I-28 floor"
    );
    let action = ssb_action_type_from_str_public("dag_credit_assigned")
        .expect("dag_credit_assigned must parse");
    assert_eq!(action, SsbActionType::DagCreditAssigned);
    assert_eq!(action.as_sql_str(), "dag_credit_assigned");
}

#[test]
fn credit_formula_unit() {
    let winner = ranked_branch("winner", 0.9);
    let sibling = ranked_branch("sibling", 0.4);
    let outcome = SearchLadderOutcome {
        winner: Some(winner),
        pruned_count: 0,
        escalated: false,
        ranked: vec![ranked_branch("winner", 0.9), sibling],
    };

    let payloads = DagCreditAssigner::run(&outcome);

    assert_eq!(payloads.len(), 1);
    assert_eq!(payloads[0].kind, DagCreditAssignedKind::CreditWalk);
    assert_eq!(payloads[0].branch_id, "sibling");
    assert!((payloads[0].credit_score - 0.5).abs() < f64::EPSILON);
    assert_eq!(payloads[0].winner_score, 0.9);
    assert_eq!(payloads[0].sibling_score, 0.4);
    assert_eq!(payloads[0].ranked_position, 1);
}

#[test]
fn env_gate_defaults_off_when_unset() {
    // No set_var/remove_var (MR-2 bans new unsafe in this diff).
    if std::env::var_os("ELAI_DAG_CREDIT_ENABLE").is_some() {
        // Ambient env from another test process; do not fight it.
        return;
    }
    assert!(
        !dag_credit_enabled(),
        "gate must be default-OFF when ELAI_DAG_CREDIT_ENABLE is unset"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn emit_path_on() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = test_ssb(&dir);
    let payload = elai_orchestrator::ssb::DagCreditAssignedPayload {
        kind: DagCreditAssignedKind::CreditWalk,
        branch_id: "sibling".into(),
        credit_score: 0.5,
        winner_score: 0.9,
        sibling_score: 0.4,
        ranked_position: 1,
        ts_ms: 0,
    };

    // Production emit path (same fn dispatch_dag_credit calls after the env gate).
    emit_dag_credit_to_ssb(&ssb, "i28-contract", 42, vec![payload])
        .await
        .expect("DAG credit emit must succeed");

    let rows = ssb
        .query_by_type(SsbActionType::DagCreditAssigned)
        .expect("query DAG credit rows");
    assert_eq!(rows.len(), 1);
    match decode_ssb_payload(&rows[0].payload).expect("DAG credit payload must decode") {
        SsbPayload::DagCreditAssigned(decoded) => {
            assert_eq!(decoded.kind, DagCreditAssignedKind::CreditWalk);
            assert_eq!(decoded.branch_id, "sibling");
            assert!((decoded.credit_score - 0.5).abs() < f64::EPSILON);
        }
        other => panic!("expected DagCreditAssigned payload, got {other:?}"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn fresh_ssb_has_zero_credit_rows() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = test_ssb(&dir);
    let rows = ssb
        .query_by_type(SsbActionType::DagCreditAssigned)
        .expect("query DAG credit rows");
    assert!(rows.is_empty(), "fresh SSB must have zero DagCreditAssigned rows");
}
