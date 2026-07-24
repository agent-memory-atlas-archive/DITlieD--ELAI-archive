//! J-60 — process_canvas derivation contract test (done-gate).
//!
//! Wiring contract observable_effect (R1/R2/R4/R7):
//!
//! 1. Process nodes derive from fixture phase_transition + agent-lifecycle
//!    SSB rows (the derive_process_nodes entrypoint).
//! 2. Re-derivation under the same input rows is byte-identical
//!    (deterministic derivation, no hidden clock / random ordering).
//! 3. TD-3: status is derived ONLY from row content (the to_state field of
//!    phase_transition rows). The derive_process_nodes API takes no external
//!    status parameter, so a model-sourced status cannot be injected; this
//!    test asserts the API surface has no status input and that a "complete"
//!    to_state in the row (not a model text field) drives Completed.
//! 4. R1: an agent with zero rows yields no node (absence of proof is proof
//!    of absence).
//! 5. The worktree diff projection comes from EditHistoryEvent row files,
//!    not from a model text field.
//! 6. hash_process_nodes is deterministic across re-derivation.

use elai_orchestrator::process_canvas::{
    derive_process_nodes, hash_process_nodes, worktree_diff_projection,
    ProcessBinding, ProcessCanvasRegistry, ProcessNodeStatus,
};
use elai_orchestrator::ssb_viewer::SsbActionDetail;

const PHASE_TRANSITION_ACTION: &str = "phase_transition";
const EDIT_HISTORY_EVENT_ACTION: &str = "edit_history_event";

fn detail(
    seq: i64,
    ts: i64,
    action_type: &str,
    agent_id: &str,
    payload: serde_json::Value,
    files: Vec<String>,
) -> SsbActionDetail {
    SsbActionDetail {
        seq,
        ts,
        action_type: action_type.to_string(),
        agent_id: agent_id.to_string(),
        payload_json: payload,
        files,
    }
}

fn phase_transition_row(seq: i64, ts: i64, agent: &str, to_state: &str) -> SsbActionDetail {
    detail(
        seq,
        ts,
        PHASE_TRANSITION_ACTION,
        agent,
        serde_json::json!({
            "from_state": "executing",
            "to_state": to_state,
            "evidence_summary": "",
        }),
        vec![],
    )
}

fn edit_history_row(seq: i64, ts: i64, agent: &str, paths: &[&str]) -> SsbActionDetail {
    detail(
        seq,
        ts,
        EDIT_HISTORY_EVENT_ACTION,
        agent,
        serde_json::json!({
            "kind": "snapshot_captured",
            "node_id": "n1",
            "paths_n": paths.len(),
            "grain": "edit",
        }),
        paths.iter().map(|s| s.to_string()).collect(),
    )
}

// ---------------------------------------------------------------------------
// (a) process nodes derive from fixture phase_transition + agent-lifecycle
//     SSB rows
// ---------------------------------------------------------------------------

/// (a) A phase_transition row + an edit_history_event row each produce a
/// distinct process node, bound to PhaseStep and AgentWorker respectively.
#[test]
fn process_nodes_derive_from_phase_and_agent_rows() {
    let rows = vec![
        phase_transition_row(1, 100, "session-a", "auditing"),
        edit_history_row(2, 200, "worker-7", &["src/lib.rs"]),
    ];
    let nodes = derive_process_nodes(&rows).expect("derive ok");
    assert_eq!(nodes.len(), 2, "one node per binding");
    assert!(
        matches!(nodes[0].binding, ProcessBinding::PhaseStep(_)),
        "first node bound to phase step"
    );
    assert!(
        matches!(nodes[1].binding, ProcessBinding::AgentWorker(_)),
        "second node bound to agent worker"
    );
}

// ---------------------------------------------------------------------------
// (b) re-derivation under the same input is byte-identical (deterministic)
// ---------------------------------------------------------------------------

/// (b) Deriving twice from the same rows yields byte-identical nodes and
/// fingerprint (no hidden clock, no random ordering).
#[test]
fn rederivation_is_byte_identical() {
    let rows = vec![
        phase_transition_row(1, 100, "session-a", "auditing"),
        edit_history_row(2, 200, "worker-7", &["src/lib.rs"]),
        phase_transition_row(3, 300, "session-a", "complete"),
    ];
    let first = ProcessCanvasRegistry::derive(&rows).expect("first derive");
    let second = ProcessCanvasRegistry::derive(&rows).expect("second derive");
    assert_eq!(
        serde_json::to_vec(&first.nodes).expect("encode first"),
        serde_json::to_vec(&second.nodes).expect("encode second"),
        "node vectors byte-identical"
    );
    assert_eq!(
        first.fingerprint, second.fingerprint,
        "fingerprint stable across rederivation"
    );
    assert_eq!(
        first.diff_entries, second.diff_entries,
        "diff entries stable across rederivation"
    );
}

// ---------------------------------------------------------------------------
// (c) TD-3: status is derived ONLY from row content, not external input
// ---------------------------------------------------------------------------

/// (c) TD-3: the derive_process_nodes API takes only `&[SsbActionDetail]`
/// rows, with no status parameter. A model-sourced status cannot be injected
/// because the API has no status input. The "complete" to_state IN THE ROW
/// (not a model text field) drives ProcessNodeStatus::Completed; a non-
/// complete to_state drives Active. This is the TD-3 contract: status comes
/// from rows, not external/model input.
#[test]
fn status_derived_from_rows_not_external_input() {
    let complete_rows = vec![phase_transition_row(1, 100, "s", "complete")];
    let active_rows = vec![phase_transition_row(1, 100, "s", "executing")];
    let complete_nodes = derive_process_nodes(&complete_rows).expect("complete derive");
    let active_nodes = derive_process_nodes(&active_rows).expect("active derive");
    assert_eq!(
        complete_nodes[0].display.status,
        ProcessNodeStatus::Completed,
        "complete to_state in row drives Completed"
    );
    assert_eq!(
        active_nodes[0].display.status,
        ProcessNodeStatus::Active,
        "non-complete to_state in row drives Active"
    );
    // The API surface itself is the TD-3 guarantee: derive_process_nodes
    // has no status parameter. A model cannot inject a status string; it
    // can only emit rows whose to_state field the derivation reads.
    // (Compile-time: the function signature is
    //   fn derive_process_nodes(rows: &[SsbActionDetail]) -> Result<...>
    // which carries no status argument. This test pins the behavior that
    // the row's to_state is the sole status source.)
}

// ---------------------------------------------------------------------------
// (d) R1: an agent with zero rows yields no node
// ---------------------------------------------------------------------------

/// (d) R1: zero rows => zero nodes. Absence of proof is proof of absence.
#[test]
fn zero_rows_yields_no_nodes() {
    let nodes = derive_process_nodes(&[]).expect("empty ok");
    assert!(nodes.is_empty(), "zero rows must yield zero nodes");
    let reg = ProcessCanvasRegistry::derive(&[]).expect("registry empty ok");
    assert!(reg.nodes.is_empty());
    assert!(reg.diff_entries.is_empty());
}

// ---------------------------------------------------------------------------
// (e) the worktree diff comes from EditHistoryEvent rows, not model text
// ---------------------------------------------------------------------------

/// (e) worktree_diff_projection filters to EditHistoryEvent rows and reads
/// the `files` field from each row. A phase_transition row carrying a file
/// path in its `files` vector is ignored; an edit_history_event row's files
/// are projected. The diff is NOT derived from any model text field in
/// payload_json.
#[test]
fn diff_comes_from_edit_history_rows_not_model_text() {
    let rows = vec![
        detail(
            1,
            100,
            PHASE_TRANSITION_ACTION,
            "s",
            serde_json::json!({"to_state": "complete", "model_text": "src/fake.rs"}),
            vec!["ignored_by_phase_transition.rs".to_string()],
        ),
        edit_history_row(2, 200, "w", &["src/real_a.rs", "src/real_b.rs"]),
    ];
    let diffs = worktree_diff_projection(&rows).expect("diff ok");
    assert_eq!(diffs.len(), 2, "only edit_history_event files projected");
    let paths: Vec<&str> = diffs.iter().map(|d| d.path.as_str()).collect();
    assert!(paths.contains(&"src/real_a.rs"));
    assert!(paths.contains(&"src/real_b.rs"));
    assert!(
        !paths.contains(&"ignored_by_phase_transition.rs"),
        "phase_transition files must not leak into diff"
    );
    assert!(
        !paths.iter().any(|p| p.contains("fake")),
        "model_text payload field must not leak into diff"
    );
}

// ---------------------------------------------------------------------------
// (f) hash_process_nodes is deterministic
// ---------------------------------------------------------------------------

/// (f) hash_process_nodes returns the same hex for the same node set across
/// repeated calls, and is stable across node vector reordering (the hasher
/// sorts by node_id before hashing).
#[test]
fn hash_process_nodes_is_deterministic() {
    let rows = vec![
        phase_transition_row(1, 100, "session-a", "auditing"),
        edit_history_row(2, 200, "worker-7", &["src/lib.rs"]),
        phase_transition_row(3, 300, "session-b", "complete"),
    ];
    let nodes = derive_process_nodes(&rows).expect("derive");
    let h1 = hash_process_nodes(&nodes);
    let h2 = hash_process_nodes(&nodes);
    assert_eq!(h1, h2, "hash stable across repeated calls");
    assert!(!h1.is_empty(), "hash non-empty");
    // Reorder the input rows; derive re-sorts by (seq, ts) so the node set
    // is identical and the hash must match.
    let mut reversed = rows.clone();
    reversed.reverse();
    let nodes_reversed = derive_process_nodes(&reversed).expect("derive reversed");
    assert_eq!(
        hash_process_nodes(&nodes),
        hash_process_nodes(&nodes_reversed),
        "hash stable across input reorder"
    );
}
