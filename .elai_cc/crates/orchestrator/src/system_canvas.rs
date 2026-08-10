use serde::{Deserialize, Serialize};

use crate::ssb::{SystemContractEventKind, SystemContractEventPayload};
use crate::ssb_viewer::{SsbViewer, SsbViewerError};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemBindingKind {
    PlanUnit,
    Contract,
    McpService,
    Draft,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemNodeLifecycle {
    Draft,
    Framed,
    Building,
    Wired,
    Verified,
    Drifted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemEdgeVerdict {
    Declared,
    Wired,
    Verified,
    Broken,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemBinding {
    pub kind: SystemBindingKind,
    #[serde(rename = "ref")]
    pub binding_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemNode {
    pub node_id: String,
    pub display: String,
    pub binding: SystemBinding,
    pub lifecycle: SystemNodeLifecycle,
    pub col: i64,
    pub row: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_pin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tainted: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemEdgeContract {
    pub produces: String,
    pub consumes: String,
    pub reachable_from: String,
    pub observable_effect: String,
    #[serde(rename = "evidenceRow", skip_serializing_if = "Option::is_none")]
    pub evidence_row: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemEdge {
    pub producer_node: String,
    pub consumer_node: String,
    pub contract: SystemEdgeContract,
    pub verdict: SystemEdgeVerdict,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SystemCanvas {
    pub nodes: Vec<SystemNode>,
    pub edges: Vec<SystemEdge>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EdgeVerdictInputs {
    pub has_producer_and_consumer: bool,
    pub has_evidence_row: bool,
    pub checker_mismatch: bool,
}

// WIRE-DARK[J-31]: caller is the Unit-3 auditor verdict tick
// (canvas_dispatcher.rs SYSTEM_NAMESPACE handler) + Unit-4 Tauri refresh_system_verdicts cmd. Promotion: Units 3/4 of the J-31 plan.
#[must_use]
pub fn derive_edge_verdict(inputs: EdgeVerdictInputs) -> SystemEdgeVerdict {
    if inputs.checker_mismatch {
        return SystemEdgeVerdict::Broken;
    }
    if inputs.has_evidence_row {
        return SystemEdgeVerdict::Verified;
    }
    if inputs.has_producer_and_consumer {
        return SystemEdgeVerdict::Wired;
    }
    SystemEdgeVerdict::Declared
}

// WIRE-DARK[J-31]: caller is the Unit-4 Tauri command `get_system_canvas_cmd`
// (crates/dashboard-v2/src-tauri/src/commands.rs, mirrors get_usage_rollup
// at :5670). Promotion: Unit 4 of the J-31 plan (same plan, not a graveyard).
pub fn get_system_canvas(
    viewer: &SsbViewer,
    limit: usize,
) -> Result<SystemCanvas, SsbViewerError> {
    let action = crate::ssb::SsbActionType::SystemContractEvent(
        SystemContractEventKind::NodeBound,
    )
    .as_sql_str();
    let rows = viewer.recent_rows_of_type(action, limit)?;
    let mut canvas = SystemCanvas::default();
    for row in rows {
        // Rows are externally tagged because SsbPayload is a plain-derive enum (ssb_viewer.rs:1709).
        let raw = row
            .payload_json
            .get("SystemContractEvent")
            .cloned()
            .unwrap_or_else(|| row.payload_json.clone());
        let payload: SystemContractEventPayload = match serde_json::from_value(raw) {
                Ok(p) => p,
                Err(e) => {
                    tracing::warn!(
                        seq = row.seq,
                        error = %e,
                        "system_canvas: SystemContractEvent payload re-type failed; skipping"
                    );
                    continue;
                }
            };
        match payload.kind {
            SystemContractEventKind::NodeBound => {
                let node = SystemNode {
                    node_id: payload.node_id,
                    display: payload
                        .details_json
                        .get("display")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("")
                        .to_owned(),
                    binding: SystemBinding {
                        kind: parse_binding_kind(&payload.binding_kind),
                        binding_ref: payload.binding_ref,
                    },
                    lifecycle: SystemNodeLifecycle::Draft,
                    col: payload
                        .details_json
                        .get("col")
                        .and_then(serde_json::Value::as_i64)
                        .unwrap_or(0),
                    row: payload
                        .details_json
                        .get("row")
                        .and_then(serde_json::Value::as_i64)
                        .unwrap_or(0),
                    schema_pin: None,
                    tainted: None,
                };
                if !canvas.nodes.iter().any(|n| n.node_id == node.node_id) {
                    canvas.nodes.push(node);
                }
            }
            SystemContractEventKind::EdgeDeclared => {
                let producer = payload
                    .details_json
                    .get("producer_node")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("")
                    .to_owned();
                let consumer = payload
                    .details_json
                    .get("consumer_node")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("")
                    .to_owned();
                let produces = payload
                    .details_json
                    .get("produces")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("")
                    .to_owned();
                let consumes = payload
                    .details_json
                    .get("consumes")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("")
                    .to_owned();
                let edge = SystemEdge {
                    producer_node: producer,
                    consumer_node: consumer,
                    contract: SystemEdgeContract {
                        produces,
                        consumes,
                        reachable_from: String::new(),
                        observable_effect: String::new(),
                        evidence_row: None,
                    },
                    verdict: SystemEdgeVerdict::Declared,
                };
                canvas.edges.push(edge);
            }
            SystemContractEventKind::EdgeVerdict | SystemContractEventKind::NodeDrifted => {}
        }
    }
    Ok(canvas)
}

fn parse_binding_kind(s: &str) -> SystemBindingKind {
    match s {
        "plan_unit" => SystemBindingKind::PlanUnit,
        "contract" => SystemBindingKind::Contract,
        "mcp_service" => SystemBindingKind::McpService,
        _ => SystemBindingKind::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_edge_verdict_broken_wins_regardless_of_node_input() {
        let v = derive_edge_verdict(EdgeVerdictInputs {
            has_producer_and_consumer: true,
            has_evidence_row: true,
            checker_mismatch: true,
        });
        assert_eq!(v, SystemEdgeVerdict::Broken);
    }

    #[test]
    fn derive_edge_verdict_ladder_order() {
        assert_eq!(
            derive_edge_verdict(EdgeVerdictInputs {
                has_producer_and_consumer: false,
                has_evidence_row: false,
                checker_mismatch: false,
            }),
            SystemEdgeVerdict::Declared
        );
        assert_eq!(
            derive_edge_verdict(EdgeVerdictInputs {
                has_producer_and_consumer: true,
                has_evidence_row: false,
                checker_mismatch: false,
            }),
            SystemEdgeVerdict::Wired
        );
        assert_eq!(
            derive_edge_verdict(EdgeVerdictInputs {
                has_producer_and_consumer: true,
                has_evidence_row: true,
                checker_mismatch: false,
            }),
            SystemEdgeVerdict::Verified
        );
    }

    #[test]
    fn empty_canvas_is_default() {
        let c = SystemCanvas::default();
        assert!(c.nodes.is_empty());
        assert!(c.edges.is_empty());
    }
}
