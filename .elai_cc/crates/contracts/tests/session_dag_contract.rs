use std::collections::HashSet;

use elai_core::clock::OsClock;
use elai_core::SeededRng;
use elai_orchestrator::session_dag::{
    DagMessageRole, SessionDag, SessionDagError, Stub, TrimConfig, TrimmedMessage,
    MAX_ACTIVE_BRANCHES, MAX_PARENT_DEPTH, PROMPT_CACHE_PREFIX_BOUNDARY, SCHEMA_VERSION,
    STUB_THRESHOLD_MAX, STUB_THRESHOLD_MIN,
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
