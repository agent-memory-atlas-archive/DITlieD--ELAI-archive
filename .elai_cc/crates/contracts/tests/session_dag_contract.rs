use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_core::SeededRng;
use elai_orchestrator::session_dag::{
    BranchRef, DagMessageRole, MergeResult, SessionDag, SessionDagError, Snapshot, Stub,
    TrimConfig, TrimmedMessage, MAX_ACTIVE_BRANCHES, MAX_PARENT_DEPTH,
    PROMPT_CACHE_PREFIX_BOUNDARY, SCHEMA_VERSION, STUB_THRESHOLD_MAX, STUB_THRESHOLD_MIN,
};
use elai_orchestrator::ssb::{SsbActionType, SsbEntry, SsbError, SsbReader};
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

    let e3 = SessionDagError::StubRecoveryFailed {
        seq: 0,
        reason: String::new(),
    };
    assert!(e3.to_string().contains("stub seq 0"));
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

// ---------------------------------------------------------------------------
// 3H.5 — Stub recovery contracts
// ---------------------------------------------------------------------------

/// Mock SsbReader for contract tests — returns pre-configured entries by seq.
struct MockSsbReader {
    entries: HashMap<i64, SsbEntry>,
}

impl MockSsbReader {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    fn insert(&mut self, seq: i64, payload: Vec<u8>) {
        self.entries.insert(
            seq,
            SsbEntry {
                seq,
                project_id: "p-mock".to_owned(),
                session_id: "s-mock".to_owned(),
                agent_id: "a-mock".to_owned(),
                superstep: 0,
                action_type: SsbActionType::ToolResult,
                payload: Arc::from(payload.as_slice()),
                created_at: 1000,
            },
        );
    }
}

impl SsbReader for MockSsbReader {
    fn query_by_session(&self, _session_id: &str) -> Result<Vec<SsbEntry>, SsbError> {
        Ok(vec![])
    }
    fn query_by_file(&self, _file_paths: &[&str]) -> Result<Vec<SsbEntry>, SsbError> {
        Ok(vec![])
    }
    fn query_by_type(&self, _action_type: SsbActionType) -> Result<Vec<SsbEntry>, SsbError> {
        Ok(vec![])
    }
    fn query_by_superstep(&self, _superstep: i64) -> Result<Vec<SsbEntry>, SsbError> {
        Ok(vec![])
    }
    fn query_conflict(
        &self,
        _file_paths: &[&str],
        _session_id: &str,
        _superstep: i64,
    ) -> Result<Vec<String>, SsbError> {
        Ok(vec![])
    }
    fn last_seq(&self) -> Result<Option<i64>, SsbError> {
        Ok(None)
    }
    fn query_by_seq(&self, seq: i64) -> Result<Option<SsbEntry>, SsbError> {
        Ok(self.entries.get(&seq).cloned())
    }
}

#[test]
fn recover_stubs_signature_callable() {
    let dir = test_dir();
    let path = db_path(&dir);
    let mut entropy = SeededRng::new(42);
    let clock = OsClock;
    let dag = SessionDag::new("s-contract-rec", &path, &mut entropy, &clock).unwrap();

    let root_id = SessionDag::generate_snap_id(&mut SeededRng::new(100));
    let root = make_root_snapshot("s-contract-rec", &root_id);
    dag.save_snapshot(&root, &[]).unwrap();

    let ssb = MockSsbReader::new();
    let result: Result<Vec<TrimmedMessage>, SessionDagError> = dag.recover_stubs(&root_id, &ssb);
    assert!(result.is_ok());
}

#[test]
fn recover_single_stub_signature_callable() {
    let dir = test_dir();
    let path = db_path(&dir);
    let mut entropy = SeededRng::new(42);
    let clock = OsClock;
    let dag = SessionDag::new("s-contract-rss", &path, &mut entropy, &clock).unwrap();

    let root_id = SessionDag::generate_snap_id(&mut SeededRng::new(100));
    let root = make_root_snapshot("s-contract-rss", &root_id);
    dag.save_snapshot(&root, &[]).unwrap();

    let ssb = MockSsbReader::new();
    let result: Result<TrimmedMessage, SessionDagError> =
        dag.recover_single_stub(&root_id, 1, &ssb);
    // seq=1 does not exist in the empty snapshot — must be StubRecoveryFailed, not SnapshotNotFound
    assert!(matches!(
        result,
        Err(SessionDagError::StubRecoveryFailed { seq: 1, .. })
    ));
}

#[test]
fn stub_recovery_failed_error_variants() {
    let e = SessionDagError::StubRecoveryFailed {
        seq: 42,
        reason: "SSB entry missing".to_owned(),
    };
    let msg = e.to_string();
    assert!(msg.contains("42"), "should contain seq");
    assert!(msg.contains("SSB entry missing"), "should contain reason");
}

#[test]
fn recover_stubs_no_stubs_unchanged() {
    let dir = test_dir();
    let path = db_path(&dir);
    let mut entropy = SeededRng::new(42);
    let clock = OsClock;
    let dag = SessionDag::new("s-contract-ns", &path, &mut entropy, &clock).unwrap();

    let root_id = SessionDag::generate_snap_id(&mut SeededRng::new(100));
    let root = make_root_snapshot("s-contract-ns", &root_id);
    let messages = vec![
        TrimmedMessage {
            seq: 1,
            role: DagMessageRole::User,
            content: b"hello".to_vec(),
            is_stubbed: false,
            stub_ssb_seq: None,
        },
        TrimmedMessage {
            seq: 2,
            role: DagMessageRole::Assistant,
            content: b"world".to_vec(),
            is_stubbed: false,
            stub_ssb_seq: None,
        },
    ];
    dag.save_snapshot(&root, &messages).unwrap();

    let ssb = MockSsbReader::new();
    let recovered = dag.recover_stubs(&root_id, &ssb).unwrap();
    assert_eq!(recovered.len(), 2);
    assert!(!recovered[0].is_stubbed);
    assert!(!recovered[1].is_stubbed);
    assert_eq!(recovered[0].content, b"hello");
    assert_eq!(recovered[1].content, b"world");
}

#[test]
fn recover_single_stub_non_stubbed_unchanged() {
    let dir = test_dir();
    let path = db_path(&dir);
    let mut entropy = SeededRng::new(42);
    let clock = OsClock;
    let dag = SessionDag::new("s-contract-nsu", &path, &mut entropy, &clock).unwrap();

    let root_id = SessionDag::generate_snap_id(&mut SeededRng::new(100));
    let root = make_root_snapshot("s-contract-nsu", &root_id);
    let messages = vec![TrimmedMessage {
        seq: 1,
        role: DagMessageRole::User,
        content: b"not a stub".to_vec(),
        is_stubbed: false,
        stub_ssb_seq: None,
    }];
    dag.save_snapshot(&root, &messages).unwrap();

    let ssb = MockSsbReader::new();
    let msg = dag.recover_single_stub(&root_id, 1, &ssb).unwrap();
    assert!(!msg.is_stubbed);
    assert_eq!(msg.content, b"not a stub");
}

#[test]
fn round_trip_trim_save_load_recover() {
    let dir = test_dir();
    let path = db_path(&dir);
    let mut entropy = SeededRng::new(42);
    let clock = OsClock;
    let dag = SessionDag::new("s-contract-rt", &path, &mut entropy, &clock).unwrap();

    // Original large payload (>500 bytes to exceed stub_threshold)
    let original_payload: Vec<u8> = (0..2048u16).flat_map(|i| i.to_le_bytes()).collect();
    let ssb_seq: u64 = 7;

    // Build messages with one Tool message that will be stubbed
    let messages = vec![
        TrimmedMessage {
            seq: 1,
            role: DagMessageRole::System,
            content: b"system".to_vec(),
            is_stubbed: false,
            stub_ssb_seq: None,
        },
        TrimmedMessage {
            seq: 2,
            role: DagMessageRole::User,
            content: b"user".to_vec(),
            is_stubbed: false,
            stub_ssb_seq: None,
        },
        TrimmedMessage {
            seq: 3,
            role: DagMessageRole::Assistant,
            content: b"assistant".to_vec(),
            is_stubbed: false,
            stub_ssb_seq: None,
        },
        TrimmedMessage {
            seq: 4,
            role: DagMessageRole::Tool,
            content: original_payload.clone(),
            is_stubbed: false,
            stub_ssb_seq: Some(ssb_seq),
        },
    ];

    let config = TrimConfig {
        stub_threshold: 500,
        model_context_window: 100_000,
        allocated_slots_tokens: 10_000,
        output_reserve: 5_000,
    };
    let valid_seqs: HashSet<u64> = [ssb_seq].into_iter().collect();
    let trimmed = dag.trim_context(&messages, config, &valid_seqs).unwrap();

    // The Tool message should be stubbed
    assert!(trimmed[3].is_stubbed, "Tool message should be stubbed");
    assert_eq!(trimmed[3].stub_ssb_seq, Some(ssb_seq));

    // Save the trimmed snapshot
    let snap_id = SessionDag::generate_snap_id(&mut SeededRng::new(200));
    let snap = make_root_snapshot("s-contract-rt", &snap_id);
    dag.save_snapshot(&snap, &trimmed).unwrap();

    // Set up mock SSB with the original payload at the expected seq
    let mut ssb = MockSsbReader::new();
    ssb.insert(ssb_seq as i64, original_payload.clone());

    // Recover stubs
    let recovered = dag.recover_stubs(&snap_id, &ssb).unwrap();
    assert_eq!(recovered.len(), 4);
    assert!(!recovered[3].is_stubbed, "should be un-stubbed after recovery");
    assert_eq!(recovered[3].stub_ssb_seq, None, "stub_ssb_seq should be cleared");
    assert_eq!(
        recovered[3].content, original_payload,
        "recovered content must be byte-equal to original"
    );

    // Verify non-stubbed messages unchanged
    assert_eq!(recovered[0].content, b"system");
    assert_eq!(recovered[1].content, b"user");
    assert_eq!(recovered[2].content, b"assistant");
}

// ---------------------------------------------------------------------------
// 3H.6 — Handoff contracts
// ---------------------------------------------------------------------------

fn make_tool_messages_for_handoff(count: usize, stub_threshold: usize) -> Vec<TrimmedMessage> {
    let mut msgs = Vec::with_capacity(count);
    for i in 0..count {
        let seq = (i + 1) as u32;
        if seq <= 3 {
            // Cache-prefix boundary messages (never stubbed)
            msgs.push(TrimmedMessage {
                seq,
                role: if seq == 1 { DagMessageRole::System } else { DagMessageRole::User },
                content: format!("prefix-msg-{seq}").into_bytes(),
                is_stubbed: false,
                stub_ssb_seq: None,
            });
        } else if i % 3 == 0 {
            // Tool messages with large content (will be stub candidates)
            let payload = vec![0xAB; stub_threshold + 100];
            msgs.push(TrimmedMessage {
                seq,
                role: DagMessageRole::Tool,
                content: payload,
                is_stubbed: false,
                stub_ssb_seq: Some(seq as u64),
            });
        } else if i % 3 == 1 {
            msgs.push(TrimmedMessage {
                seq,
                role: DagMessageRole::User,
                content: format!("user-msg-{seq}").into_bytes(),
                is_stubbed: false,
                stub_ssb_seq: None,
            });
        } else {
            msgs.push(TrimmedMessage {
                seq,
                role: DagMessageRole::Assistant,
                content: format!("assistant-msg-{seq}").into_bytes(),
                is_stubbed: false,
                stub_ssb_seq: None,
            });
        }
    }
    msgs
}

#[test]
fn contract_create_handoff_saves_tag() {
    let dir = test_dir();
    let path = db_path(&dir);
    let mut entropy = SeededRng::new(42);
    let clock = OsClock;
    let dag = SessionDag::new("s-handoff-tag", &path, &mut entropy, &clock).unwrap();

    let msgs = make_tool_messages_for_handoff(10, 500);
    let config = TrimConfig {
        stub_threshold: 500,
        model_context_window: 100_000,
        allocated_slots_tokens: 10_000,
        output_reserve: 5_000,
    };
    let ssb = MockSsbReader::new();

    let snap = dag.create_handoff(
        &msgs, config, &ssb,
        &mut SeededRng::new(100), &clock, "EXECUTING",
    ).unwrap();

    assert_eq!(snap.tag, Some("handoff".to_owned()), "handoff snapshot must have tag='handoff'");
}

#[test]
fn contract_load_handoff_returns_none_when_no_handoff() {
    let dir = test_dir();
    let path = db_path(&dir);
    let mut entropy = SeededRng::new(42);
    let clock = OsClock;
    let dag = SessionDag::new("s-handoff-none", &path, &mut entropy, &clock).unwrap();

    let result = dag.load_handoff().unwrap();
    assert!(result.is_none(), "empty DAG should return None for load_handoff");
}

#[test]
fn contract_load_handoff_returns_trunk_tip_no_branches() {
    let dir = test_dir();
    let path = db_path(&dir);
    let mut entropy = SeededRng::new(42);
    let clock = OsClock;
    let dag = SessionDag::new("s-handoff-trunk", &path, &mut entropy, &clock).unwrap();

    let msgs = make_tool_messages_for_handoff(10, 500);
    let config = TrimConfig {
        stub_threshold: 500,
        model_context_window: 100_000,
        allocated_slots_tokens: 10_000,
        output_reserve: 5_000,
    };
    let ssb = MockSsbReader::new();

    let snap = dag.create_handoff(
        &msgs, config, &ssb,
        &mut SeededRng::new(100), &clock, "EXECUTING",
    ).unwrap();

    let loaded = dag.load_handoff().unwrap();
    assert!(loaded.is_some(), "should find the handoff snapshot");
    let (loaded_snap, _loaded_msgs) = loaded.unwrap();
    assert_eq!(loaded_snap.snap_id, snap.snap_id, "should load the same handoff snapshot");
}

#[test]
fn contract_load_handoff_returns_branch_tip_when_active_branch() {
    let dir = test_dir();
    let path = db_path(&dir);
    let mut entropy = SeededRng::new(42);
    let clock = OsClock;
    let mut dag = SessionDag::new("s-handoff-br", &path, &mut entropy, &clock).unwrap();

    let msgs = make_tool_messages_for_handoff(10, 500);
    let config = TrimConfig {
        stub_threshold: 500,
        model_context_window: 100_000,
        allocated_slots_tokens: 10_000,
        output_reserve: 5_000,
    };
    let ssb = MockSsbReader::new();

    let handoff_snap = dag.create_handoff(
        &msgs, config, &ssb,
        &mut SeededRng::new(100), &clock, "EXECUTING",
    ).unwrap();

    let branch = dag.create_branch(
        "feat-x", &handoff_snap.snap_id,
        &mut SeededRng::new(200), &clock,
    ).unwrap();

    let loaded = dag.load_handoff().unwrap();
    assert!(loaded.is_some(), "should find branch tip");
    let (loaded_snap, _) = loaded.unwrap();
    assert_eq!(
        loaded_snap.snap_id, branch.tip_snap_id,
        "load_handoff must return branch tip, not trunk handoff"
    );
}

#[test]
fn contract_handoff_round_trip() {
    let dir = test_dir();
    let path = db_path(&dir);
    let mut entropy = SeededRng::new(42);
    let clock = OsClock;
    let dag = SessionDag::new("s-handoff-rt", &path, &mut entropy, &clock).unwrap();

    let msgs = vec![
        TrimmedMessage {
            seq: 1,
            role: DagMessageRole::System,
            content: b"system prompt".to_vec(),
            is_stubbed: false,
            stub_ssb_seq: None,
        },
        TrimmedMessage {
            seq: 2,
            role: DagMessageRole::User,
            content: b"user message".to_vec(),
            is_stubbed: false,
            stub_ssb_seq: None,
        },
        TrimmedMessage {
            seq: 4,
            role: DagMessageRole::Assistant,
            content: b"assistant reply".to_vec(),
            is_stubbed: false,
            stub_ssb_seq: None,
        },
    ];

    let config = TrimConfig {
        stub_threshold: 500,
        model_context_window: 100_000,
        allocated_slots_tokens: 10_000,
        output_reserve: 5_000,
    };
    let ssb = MockSsbReader::new();

    let snap = dag.create_handoff(
        &msgs, config, &ssb,
        &mut SeededRng::new(100), &clock, "PLANNING",
    ).unwrap();

    let loaded = dag.load_handoff().unwrap();
    assert!(loaded.is_some(), "round-trip should find handoff");
    let (loaded_snap, loaded_msgs) = loaded.unwrap();
    assert_eq!(loaded_snap.snap_id, snap.snap_id);
    assert_eq!(loaded_msgs.len(), msgs.len(), "message count must match");
    assert_eq!(loaded_msgs[0].content, b"system prompt");
    assert_eq!(loaded_msgs[1].content, b"user message");
    assert_eq!(loaded_msgs[2].content, b"assistant reply");
}

#[test]
fn contract_sequential_handoffs_parent_chain() {
    let dir = test_dir();
    let path = db_path(&dir);
    let mut entropy = SeededRng::new(42);
    let clock = OsClock;
    let dag = SessionDag::new("s-handoff-seq", &path, &mut entropy, &clock).unwrap();

    let msgs = vec![
        TrimmedMessage {
            seq: 1,
            role: DagMessageRole::System,
            content: b"sys".to_vec(),
            is_stubbed: false,
            stub_ssb_seq: None,
        },
        TrimmedMessage {
            seq: 2,
            role: DagMessageRole::User,
            content: b"usr".to_vec(),
            is_stubbed: false,
            stub_ssb_seq: None,
        },
    ];
    let config = TrimConfig {
        stub_threshold: 500,
        model_context_window: 100_000,
        allocated_slots_tokens: 10_000,
        output_reserve: 5_000,
    };
    let ssb = MockSsbReader::new();

    let snap1 = dag.create_handoff(
        &msgs, config, &ssb,
        &mut SeededRng::new(100), &clock, "EXECUTING",
    ).unwrap();
    assert_eq!(snap1.parent_id, None, "first handoff has no parent");

    let snap2 = dag.create_handoff(
        &msgs, config, &ssb,
        &mut SeededRng::new(200), &clock, "PLANNING",
    ).unwrap();
    assert_eq!(
        snap2.parent_id,
        Some(snap1.snap_id.clone()),
        "second handoff's parent must be the first handoff"
    );
}
