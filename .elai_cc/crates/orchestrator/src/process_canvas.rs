use serde::{Deserialize, Serialize};

use crate::ssb_viewer::SsbActionDetail;

const PHASE_TRANSITION_ACTION: &str = "phase_transition";
const EDIT_HISTORY_EVENT_ACTION: &str = "edit_history_event";
const COMPLETE_STATE: &str = "complete";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessBinding {
    FsmSession(String),
    AgentWorker(String),
    PhaseStep(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessNodeStatus {
    Active,
    Idle,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessNodeDisplay {
    pub label: String,
    pub status: ProcessNodeStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessNode {
    pub node_id: String,
    pub binding: ProcessBinding,
    pub display: ProcessNodeDisplay,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorktreeDiffEntry {
    pub path: String,
    pub seq: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessCanvasRegistry {
    pub nodes: Vec<ProcessNode>,
    pub diff_entries: Vec<WorktreeDiffEntry>,
    pub fingerprint: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ProcessCanvasError {
    #[error("process_canvas: payload re-type failed at seq={seq}: {source}")]
    PayloadReType { seq: i64, source: serde_json::Error },
    #[error("process_canvas: serialization failed: {0}")]
    Serialize(#[from] serde_json::Error),
}

fn binding_key(binding: &ProcessBinding) -> String {
    match binding {
        ProcessBinding::FsmSession(s) => format!("fsm_session:{s}"),
        ProcessBinding::AgentWorker(s) => format!("agent_worker:{s}"),
        ProcessBinding::PhaseStep(s) => format!("phase_step:{s}"),
    }
}

fn node_id_for(binding: &ProcessBinding) -> String {
    binding_key(binding)
}

fn label_for(binding: &ProcessBinding, fallback: &str) -> String {
    let s = match binding {
        ProcessBinding::FsmSession(s) => s,
        ProcessBinding::AgentWorker(s) => s,
        ProcessBinding::PhaseStep(s) => s,
    };
    if s.is_empty() { fallback.to_string() } else { s.clone() }
}

fn extract_phase_step_id(payload: &serde_json::Value) -> Option<String> {
    payload
        .get("to_state")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn is_complete_transition(payload: &serde_json::Value) -> bool {
    payload
        .get("to_state")
        .and_then(|v| v.as_str())
        .map(|s| s.eq_ignore_ascii_case(COMPLETE_STATE))
        .unwrap_or(false)
}

pub fn derive_process_nodes(
    rows: &[SsbActionDetail],
) -> Result<Vec<ProcessNode>, ProcessCanvasError> {
    let mut sorted: Vec<&SsbActionDetail> = rows
        .iter()
        .filter(|r| {
            r.action_type == PHASE_TRANSITION_ACTION
                || r.action_type == EDIT_HISTORY_EVENT_ACTION
        })
        .collect();
    sorted.sort_by(|a, b| (a.seq, a.ts).cmp(&(b.seq, b.ts)));

    let mut last_status: std::collections::HashMap<String, ProcessNodeStatus> =
        std::collections::HashMap::new();
    let mut labels: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    let mut bindings: std::collections::HashMap<String, ProcessBinding> =
        std::collections::HashMap::new();
    let mut order: Vec<String> = Vec::new();

    for row in &sorted {
        let binding = if row.action_type == PHASE_TRANSITION_ACTION {
            let phase_id = extract_phase_step_id(&row.payload_json)
                .unwrap_or_else(|| row.agent_id.clone());
            if is_complete_transition(&row.payload_json) {
                last_status.insert(node_id_for(&ProcessBinding::FsmSession(phase_id.clone())),
                    ProcessNodeStatus::Completed);
            }
            ProcessBinding::PhaseStep(phase_id)
        } else {
            ProcessBinding::AgentWorker(row.agent_id.clone())
        };

        let id = node_id_for(&binding);
        if !bindings.contains_key(&id) {
            bindings.insert(id.clone(), binding.clone());
            order.push(id.clone());
            last_status.entry(id.clone()).or_insert(ProcessNodeStatus::Active);
        }
        let label = label_for(&binding, &row.agent_id);
        labels.insert(id.clone(), label);

        let status = if row.action_type == PHASE_TRANSITION_ACTION
            && is_complete_transition(&row.payload_json)
        {
            ProcessNodeStatus::Completed
        } else {
            ProcessNodeStatus::Active
        };
        last_status.insert(id, status);
    }

    let mut nodes: Vec<ProcessNode> = Vec::with_capacity(order.len());
    for id in &order {
        let binding = match bindings.remove(id) {
            Some(b) => b,
            None => continue,
        };
        let label = labels.remove(id).unwrap_or_else(|| id.clone());
        let status = last_status.remove(id).unwrap_or(ProcessNodeStatus::Idle);
        nodes.push(ProcessNode {
            node_id: id.clone(),
            display: ProcessNodeDisplay { label, status },
            binding,
        });
    }
    Ok(nodes)
}

pub fn worktree_diff_projection(
    rows: &[SsbActionDetail],
) -> Result<Vec<WorktreeDiffEntry>, ProcessCanvasError> {
    let mut entries: Vec<WorktreeDiffEntry> = Vec::new();
    for row in rows.iter().filter(|r| r.action_type == EDIT_HISTORY_EVENT_ACTION) {
        for path in &row.files {
            entries.push(WorktreeDiffEntry {
                path: path.clone(),
                seq: row.seq,
            });
        }
    }
    entries.sort_by(|a, b| a.seq.cmp(&b.seq).then_with(|| a.path.cmp(&b.path)));
    Ok(entries)
}

pub fn hash_process_nodes(nodes: &[ProcessNode]) -> String {
    let mut sorted: Vec<&ProcessNode> = nodes.iter().collect();
    sorted.sort_by(|a, b| a.node_id.cmp(&b.node_id));
    let serialized = match serde_json::to_vec(&sorted) {
        Ok(v) => v,
        Err(_) => return String::new(),
    };
    blake3::hash(&serialized).to_hex().to_string()
}

impl ProcessCanvasRegistry {
    pub fn derive(rows: &[SsbActionDetail]) -> Result<Self, ProcessCanvasError> {
        let nodes = derive_process_nodes(rows)?;
        let diff_entries = worktree_diff_projection(rows)?;
        let fingerprint = hash_process_nodes(&nodes);
        Ok(Self {
            nodes,
            diff_entries,
            fingerprint,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn detail(seq: i64, ts: i64, at: &str, agent: &str, payload: serde_json::Value, files: Vec<String>) -> SsbActionDetail {
        SsbActionDetail {
            seq,
            ts,
            action_type: at.to_string(),
            agent_id: agent.to_string(),
            payload_json: payload,
            files,
        }
    }

    #[test]
    fn empty_rows_yields_no_nodes() {
        let nodes = derive_process_nodes(&[]).expect("empty ok");
        assert!(nodes.is_empty());
    }

    #[test]
    fn phase_transition_complete_marks_completed() {
        let rows = vec![detail(
            1,
            100,
            PHASE_TRANSITION_ACTION,
            "session-a",
            serde_json::json!({"from_state": "executing", "to_state": "complete", "evidence_summary": ""}),
            vec![],
        )];
        let nodes = derive_process_nodes(&rows).expect("derive ok");
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].display.status, ProcessNodeStatus::Completed);
    }

    #[test]
    fn edit_history_event_yields_agent_worker_node() {
        let rows = vec![detail(
            2,
            200,
            EDIT_HISTORY_EVENT_ACTION,
            "worker-7",
            serde_json::json!({"kind": "snapshot_captured", "node_id": "n1", "paths_n": 1, "grain": "edit"}),
            vec!["src/lib.rs".to_string()],
        )];
        let nodes = derive_process_nodes(&rows).expect("derive ok");
        assert_eq!(nodes.len(), 1);
        assert!(matches!(nodes[0].binding, ProcessBinding::AgentWorker(_)));
        assert_eq!(nodes[0].display.status, ProcessNodeStatus::Active);
    }

    #[test]
    fn diff_projection_filters_edit_history_only() {
        let rows = vec![
            detail(1, 100, PHASE_TRANSITION_ACTION, "s", serde_json::json!({}), vec!["ignored.txt".to_string()]),
            detail(2, 200, EDIT_HISTORY_EVENT_ACTION, "w", serde_json::json!({}), vec!["src/a.rs".to_string(), "src/b.rs".to_string()]),
        ];
        let diffs = worktree_diff_projection(&rows).expect("diff ok");
        assert_eq!(diffs.len(), 2);
        assert_eq!(diffs[0].path, "src/a.rs");
    }

    #[test]
    fn hash_is_deterministic_across_reorder() {
        let rows = vec![
            detail(2, 200, EDIT_HISTORY_EVENT_ACTION, "w2", serde_json::json!({}), vec![]),
            detail(1, 100, EDIT_HISTORY_EVENT_ACTION, "w1", serde_json::json!({}), vec![]),
        ];
        let nodes_a = derive_process_nodes(&rows).expect("a");
        let mut reversed = rows.clone();
        reversed.reverse();
        let nodes_b = derive_process_nodes(&reversed).expect("b");
        assert_eq!(hash_process_nodes(&nodes_a), hash_process_nodes(&nodes_b));
    }

    #[test]
    fn registry_derive_combines_all() {
        let rows = vec![detail(
            1,
            100,
            EDIT_HISTORY_EVENT_ACTION,
            "w",
            serde_json::json!({}),
            vec!["f.rs".to_string()],
        )];
        let reg = ProcessCanvasRegistry::derive(&rows).expect("registry ok");
        assert_eq!(reg.nodes.len(), 1);
        assert_eq!(reg.diff_entries.len(), 1);
        assert!(!reg.fingerprint.is_empty());
    }
}
