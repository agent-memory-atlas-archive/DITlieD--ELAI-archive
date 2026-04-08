use std::collections::HashSet;

use elai_core::clock::OsClock;
use elai_core::SeededRng;
use elai_orchestrator::session_dag::{
    BranchRef, DagMessageRole, MergeResult, SessionDag, SessionDagError, Snapshot, Stub,
    TrimConfig, TrimmedMessage, MAX_ACTIVE_BRANCHES, MAX_PARENT_DEPTH,
    PROMPT_CACHE_PREFIX_BOUNDARY, SCHEMA_VERSION, STUB_THRESHOLD_MAX, STUB_THRESHOLD_MIN,
};
use tempfile::TempDir;

// Compile-time constant assertions
const _: () = assert!(SCHEMA_VERSION == 1);
const _: () = assert!(MAX_ACTIVE_BRANCHES == 3);
const _: () = assert!(MAX_PARENT_DEPTH == 1000);
const _: () = assert!(STUB_THRESHOLD_MIN == 100);
const _: () = assert!(STUB_THRESHOLD_MAX == 1000);
const _: () = assert!(PROMPT_CACHE_PREFIX_BOUNDARY == 3);

fn test_dir() -> TempDir {
    tempfile::tempdir().unwrap()
}

fn db_path(dir: &TempDir) -> std::path::PathBuf {
    dir.path().join("sessions").join("contract_test.db")
}

#[test]
fn trim_config_fields_accessible() {
    let tc = TrimConfig {
        stub_threshold: 500,
        model_context_window: 100_000,
        allocated_slots_tokens: 10_000,
        output_reserve: 5_000,
    };
    assert_eq!(tc.stub_threshold, 500);
    assert_eq!(tc.model_context_window, 100_000);
    assert_eq!(tc.allocated_slots_tokens, 10_000);
    assert_eq!(tc.output_reserve, 5_000);
}

#[test]
fn trim_config_is_copy() {
    let tc = TrimConfig {
        stub_threshold: 500,
        model_context_window: 100_000,
        allocated_slots_tokens: 10_000,
        output_reserve: 5_000,
    };
    let tc2 = tc; // Copy
    assert_eq!(tc.stub_threshold, tc2.stub_threshold);
}

#[test]
fn trimmed_message_fields_accessible() {
    let msg = TrimmedMessage {
        seq: 4,
        role: DagMessageRole::Tool,
        content: vec![0xAA; 100],
        is_stubbed: false,
        stub_ssb_seq: Some(42),
    };
    assert_eq!(msg.seq, 4);
    assert_eq!(msg.role, DagMessageRole::Tool);
    assert!(!msg.is_stubbed);
    assert_eq!(msg.stub_ssb_seq, Some(42));
}

#[test]
fn stub_fields_accessible() {
    let stub = Stub {
        original_type: "tool_result".to_owned(),
        original_tokens: 500,
        summary_hash: [0xAB; 32],
        ssb_seq: 42,
    };
    assert_eq!(stub.original_type, "tool_result");
    assert_eq!(stub.original_tokens, 500);
    assert_eq!(stub.ssb_seq, 42);
    assert_eq!(stub.summary_hash.len(), 32);
}

#[test]
fn trim_context_signature_exists() {
    let dir = test_dir();
    let path = db_path(&dir);
    let mut entropy = SeededRng::new(42);
    let clock = OsClock;
    let dag = SessionDag::new("s-contract", &path, &mut entropy, &clock).unwrap();

    let config = TrimConfig {
        stub_threshold: 500,
        model_context_window: 100_000,
        allocated_slots_tokens: 10_000,
        output_reserve: 5_000,
    };
    let messages: Vec<TrimmedMessage> = Vec::new();
    let valid_ssb_seqs: HashSet<u64> = HashSet::new();

    let result = dag.trim_context(&messages, config, &valid_ssb_seqs);
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn error_variants_exist() {
    let e1 = SessionDagError::TrimBudgetExceeded {
        actual: 1000,
        target: 500,
    };
    assert!(e1.to_string().contains("1000"));
    assert!(e1.to_string().contains("500"));

    let e2 = SessionDagError::InvalidTrimConfig {
        reason: "test reason",
    };
    assert!(e2.to_string().contains("test reason"));
}

#[test]
fn dag_message_role_tool_variant_exists() {
    let role = DagMessageRole::Tool;
    assert_eq!(format!("{role}"), "tool");
}

// ---------------------------------------------------------------------------
// 3H.4 — Branch operations contracts
// ---------------------------------------------------------------------------

fn make_root_snapshot(session_id: &str, snap_id: &str) -> Snapshot {
    Snapshot {
        snap_id: snap_id.to_owned(),
        parent_id: None,
        session_id: session_id.to_owned(),
        fsm_state: "EXECUTING".to_owned(),
        phase: Some("3H".to_owned()),
        step: Some("4".to_owned()),
        findings: None,
        code_changes: None,
        tests_written: None,
        commit_hashes: vec![],
        created_at: 1000,
        tag: None,
    }
}

#[test]
fn branch_ref_fields_accessible() {
    let br = BranchRef {
        name: "explore-1".to_owned(),
        tip_snap_id: "snap-001".to_owned(),
        created_at: 1700000000,
    };
    assert_eq!(br.name, "explore-1");
    assert_eq!(br.tip_snap_id, "snap-001");
    assert_eq!(br.created_at, 1700000000);
}

#[test]
fn create_branch_signature_callable() {
    let dir = test_dir();
    let path = db_path(&dir);
    let mut entropy = SeededRng::new(42);
    let clock = OsClock;
    let mut dag = SessionDag::new("s-contract-br", &path, &mut entropy, &clock).unwrap();

    let root_id = SessionDag::generate_snap_id(&mut SeededRng::new(100));
    let root = make_root_snapshot("s-contract-br", &root_id);
    dag.save_snapshot(&root, &[]).unwrap();

    let result: Result<BranchRef, SessionDagError> =
        dag.create_branch("test-br", &root_id, &mut SeededRng::new(200), &clock);
    assert!(result.is_ok());
}

#[test]
fn branch_limit_enforcement() {
    let dir = test_dir();
    let path = db_path(&dir);
    let mut entropy = SeededRng::new(42);
    let clock = OsClock;
    let mut dag = SessionDag::new("s-contract-limit", &path, &mut entropy, &clock).unwrap();

    let root_id = SessionDag::generate_snap_id(&mut SeededRng::new(100));
    let root = make_root_snapshot("s-contract-limit", &root_id);
    dag.save_snapshot(&root, &[]).unwrap();

    for i in 0..MAX_ACTIVE_BRANCHES {
        dag.create_branch(&format!("b{i}"), &root_id, &mut SeededRng::new(200 + i as u64), &clock)
            .unwrap();
    }

    let err = dag.create_branch("overflow", &root_id, &mut SeededRng::new(999), &clock);
    assert!(matches!(
        err,
        Err(SessionDagError::BranchLimitExceeded { max: 3 })
    ));
}

#[test]
fn list_branches_returns_slice() {
    let dir = test_dir();
    let path = db_path(&dir);
    let mut entropy = SeededRng::new(42);
    let clock = OsClock;
    let dag = SessionDag::new("s-contract-list", &path, &mut entropy, &clock).unwrap();

    let branches: &[BranchRef] = dag.list_branches();
    assert!(branches.is_empty());
}

#[test]
fn merge_branch_signature_callable() {
    let dir = test_dir();
    let path = db_path(&dir);
    let mut entropy = SeededRng::new(42);
    let clock = OsClock;
    let mut dag = SessionDag::new("s-contract-merge", &path, &mut entropy, &clock).unwrap();

    let root_id = SessionDag::generate_snap_id(&mut SeededRng::new(100));
    let root = make_root_snapshot("s-contract-merge", &root_id);
    dag.save_snapshot(&root, &[]).unwrap();

    dag.create_branch("m1", &root_id, &mut SeededRng::new(200), &clock).unwrap();

    let result: Result<MergeResult, SessionDagError> =
        dag.merge_branch("m1", &mut SeededRng::new(300), &clock);
    assert!(result.is_ok());
}

#[test]
fn merge_result_variants_accessible() {
    let winner = MergeResult::Winner {
        branch: BranchRef {
            name: "w".to_owned(),
            tip_snap_id: "s1".to_owned(),
            created_at: 1000,
        },
        abandoned: vec![],
    };
    match &winner {
        MergeResult::Winner { branch, abandoned } => {
            assert_eq!(branch.name, "w");
            assert!(abandoned.is_empty());
        }
        MergeResult::Conflict { reason } => panic!("unexpected conflict: {reason}"),
    }

    let conflict = MergeResult::Conflict {
        reason: "test".to_owned(),
    };
    assert!(matches!(conflict, MergeResult::Conflict { .. }));
}

#[test]
fn close_branch_signature_callable() {
    let dir = test_dir();
    let path = db_path(&dir);
    let mut entropy = SeededRng::new(42);
    let clock = OsClock;
    let mut dag = SessionDag::new("s-contract-close", &path, &mut entropy, &clock).unwrap();

    let root_id = SessionDag::generate_snap_id(&mut SeededRng::new(100));
    let root = make_root_snapshot("s-contract-close", &root_id);
    dag.save_snapshot(&root, &[]).unwrap();

    dag.create_branch("c1", &root_id, &mut SeededRng::new(200), &clock).unwrap();

    let result: Result<(), SessionDagError> = dag.close_branch("c1");
    assert!(result.is_ok());
    assert!(dag.active_branches().is_empty());
}
