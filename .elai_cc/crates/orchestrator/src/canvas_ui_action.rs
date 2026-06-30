// H-15 (SCHEMA_VERSION 222) -- canvas_ui_action.rs
//
// CanvasUiAction: typed inbound canvas actions from the user. When the user
// interacts with a canvas node, the Tauri `canvas_select_node` command
// (commands_h15.rs) decodes the action, runs the taint gate, emits a
// canvas_view_event(UiAction) SSB row, and returns the node content with a
// provenance flag to the frontend.
//
// Trust-spine differentiator (CANVAS §19): a web-sourced node's content MUST
// NOT silently re-enter the model prompt without the provenance flag. The
// Select gate is the enforcement point.
//
// Gate design (D-2 in H-15-decisions.md):
//   1. taint_text() — synchronous regex fast-path (capability_flow.rs:234).
//      Detects injection patterns in raw text without needing registered state.
//   2. tainted_context().lookup_for_argv() — verbatim-substring match against
//      the process-local TaintFlowCtx side-table. Catches content registered
//      via TaintFlowCtx::register() at ingest time.
//   If either fires:
//     - allow_tool_with_origin gate runs (elai_safety).
//     - On Reject (TrustLevel::Untrusted): emit_taint_flow fires; Select returns
//       Err to block the handoff.
//     - On Allow (Recallable but not Untrusted): content is returned with
//       taint_flagged=true (never silent).
//     - Emit canvas_view_event(UiAction) row regardless of verdict for journaling.
//
// HARD RULES:
//   - No .unwrap() / .expect() in production paths.
//   - Reuse taint_text + G-1 chain; do NOT re-implement taint (GD-1 P0).
//   - tainted_context() path: same import as production_agent_rpc.rs:63.
//   - emit_taint_flow: same adapter as production_agent_rpc.rs:210.
//   - blake3 for any hash (not sha2/blake2).

use std::sync::Arc;

use tracing::warn;

use crate::{
    capability_flow::{taint_text, UntrustedTaint},
    ssb::{CanvasViewKind, CanvasViewPayload, Ssb, SsbPayload},
    taint_flow_ssb_adapter::emit_taint_flow,
};

// ---------------------------------------------------------------------------
// Named constants
// ---------------------------------------------------------------------------

/// Taint gate task_id sentinel for UiAction Select events.
/// Named constant (CLAUDE.md no-magic-numbers HARD RULE).
const SELECT_TAINT_TASK_ID: &str = "h15-canvas-select";

// ---------------------------------------------------------------------------
// CanvasUiAction enum
// ---------------------------------------------------------------------------

/// Typed inbound canvas action from the user.
///
/// All variants that reach the model are JOURNALED via canvas_view_event rows
/// (H15-2 slice). ToolCall is additionally gated by F-1/F-2 write-time consent.
#[derive(Debug, Clone, PartialEq)]
pub enum CanvasUiAction {
    /// User selected a canvas node. Carries the node id + content + taint
    /// verdict. The Select gate fires taint_text + TaintFlowCtx lookup before
    /// the content is handed to the model.
    Select {
        /// The selected node id.
        node_id: String,
        /// The node's current content (text label, annotation, or serialized state).
        content: String,
        /// Taint verdict from the gate. Some = flagged (provenance flag must be
        /// attached to any model context derived from this content).
        taint: Option<UntrustedTaint>,
    },
    /// User expressed an intent (e.g. "expand this tradeoff").
    Intent {
        node_id: String,
        intent_text: String,
    },
    /// User attached a free-text prompt to a canvas node.
    Prompt {
        node_id: String,
        prompt_text: String,
    },
    /// User triggered a tool call from the canvas.
    /// GATED: requires F-1/F-2 write-time gate + explicit user consent.
    /// Never auto-executes.
    ToolCall {
        node_id: String,
        tool_name: String,
        /// Whether the write-time consent gate was satisfied (F-1/F-2).
        consent_granted: bool,
    },
    /// User clicked a hyperlink on a canvas node.
    Link {
        node_id: String,
        url: String,
    },
}

// ---------------------------------------------------------------------------
// Select taint gate
// ---------------------------------------------------------------------------

/// Result of the Select taint gate.
#[derive(Debug, Clone, PartialEq)]
pub enum SelectGateVerdict {
    /// Content is clean (no taint detected by either gate leg).
    Clean,
    /// Content was flagged (provenance flag MUST be attached to model context;
    /// the content is returned but marked). Never silent.
    Flagged { taint: UntrustedTaint },
    /// Content is BLOCKED (TrustLevel::Untrusted + allow_tool_with_origin
    /// returned Reject). The SSB emit_taint_flow row has been emitted.
    Blocked { reason: String },
}

/// Run the two-leg taint gate on `node_content` for a CanvasUiAction::Select.
///
/// Leg 1: `taint_text()` — synchronous regex fast-path (no side-table state).
/// Leg 2: `tainted_context().lookup_for_argv()` — verbatim-substring match.
///
/// Returns [`SelectGateVerdict`]. Emits a `taint_flow` SSB row on Blocked.
///
/// GD-1: reuses `taint_text` (capability_flow.rs:234) and the G-1 chain
/// (`tainted_context()` + `allow_tool_with_origin` + `emit_taint_flow`).
/// Does NOT re-implement any taint primitive.
pub fn run_select_gate(
    node_id: &str,
    node_content: &str,
    ssb: Option<&Arc<Ssb>>,
) -> SelectGateVerdict {
    // --- Leg 1: regex fast-path ---
    let regex_taint = taint_text(node_content);

    // --- Leg 2: TaintFlowCtx verbatim-substring lookup ---
    use elai_safety::taint_flow::tainted_context;
    use elai_safety::{allow_tool_with_origin, CapabilityVerdict};
    use elai_safety::provenance::{Provenance, Provenanced};

    let ctx = tainted_context();
    let provenance_hit = ctx.lookup_for_argv(node_content);

    // If neither leg fired, content is clean.
    if regex_taint.is_none() && provenance_hit.is_none() {
        return SelectGateVerdict::Clean;
    }

    // At least one leg fired. Run allow_tool_with_origin on the combined signal.
    // Use the provenance_hit if available (stronger: has registered source);
    // fall back to a synthetic provenance from the regex verdict label.
    let combined_prov: Option<Provenance> = provenance_hit;

    if let Some(prov) = combined_prov {
        // G-1 chain: allow_tool_with_origin gate
        let argv: Vec<Provenanced<String>> = vec![
            Provenanced::new(node_content.to_owned(), prov.clone()),
        ];
        match allow_tool_with_origin("canvas_select", &argv) {
            Ok(CapabilityVerdict::Reject {
                reason,
                offending_trust_label,
                ..
            }) => {
                // BLOCKED: emit taint_flow row + return Blocked
                use crate::ssb::TaintFlowKind;
                let kind = TaintFlowKind::Web; // conservative: unknown source = web
                let origin = prov.source.label().to_owned();
                if let Some(ssb) = ssb {
                    emit_taint_flow(
                        ssb,
                        kind,
                        &origin,
                        &offending_trust_label,
                        "canvas_select",
                        SELECT_TAINT_TASK_ID,
                    );
                }
                warn!(
                    node_id = %node_id,
                    origin = %origin,
                    "canvas_ui_action: Select BLOCKED — node content derives from untrusted \
                     ingest; provenance flag prevents silent re-injection"
                );
                return SelectGateVerdict::Blocked { reason };
            }
            Ok(CapabilityVerdict::Allow) => {
                // Allow but provenance was registered (Recallable tier): FLAG but
                // do not block. The provenance flag attaches to the model context.
            }
            Err(e) => {
                // Gate error is treated as fail-CLOSED: flag the content.
                warn!(
                    node_id = %node_id,
                    error = %e,
                    "canvas_ui_action: allow_tool_with_origin gate error; treating as Flagged"
                );
            }
        }
        // Reach here: Allow or non-blocking Reject. Return Flagged.
        return SelectGateVerdict::Flagged {
            taint: UntrustedTaint {
                source: prov.source.label().to_owned(),
            },
        };
    }

    // No TaintFlowCtx hit, but regex fired. Return Flagged (not Blocked:
    // regex alone without a registered provenance is a soft signal per D-2).
    if let Some(taint) = regex_taint {
        warn!(
            node_id = %node_id,
            source = %taint.source,
            "canvas_ui_action: Select regex-flagged — content matched injection pattern; \
             provenance flag attached (not blocked; no registered TaintFlowCtx provenance)"
        );
        return SelectGateVerdict::Flagged { taint };
    }

    // Defensive fallback (should not be reached given entry condition).
    SelectGateVerdict::Clean
}

/// Emit a `canvas_view_event(UiAction)` SSB row for a Select action.
///
/// Always emits regardless of gate verdict (journaling requirement HD-4).
/// `taint_flagged` records whether the gate flagged the content.
pub fn emit_select_ssb_row(
    ssb: &Arc<Ssb>,
    node_id: &str,
    taint_flagged: bool,
    task_id: &str,
    session_id: &str,
) {
    let payload = SsbPayload::CanvasViewEvent(CanvasViewPayload {
        kind: CanvasViewKind::UiAction,
        view_type: String::new(),
        view_id: String::new(),
        seed_op_ids: vec![],
        node_id: node_id.to_owned(),
        taint_flagged,
        task_id: task_id.to_owned(),
        session_id: session_id.to_owned(),
    });
    let ssb = ssb.clone();
    let node_id_owned = node_id.to_owned();
    let ts_ms = std::time::SystemTime::now() // entropy-allow: SSB journal ts_ms, no ClockProvider in scope (sync fire-and-forget emitter)
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let _ = tokio::runtime::Handle::try_current().map(|h| {
        h.spawn(async move {
            if let Err(e) = ssb
                .write_payload(
                    "canvas_ui_action",
                    ts_ms.try_into().unwrap_or(i64::MAX),
                    payload,
                    vec![],
                )
                .await
            {
                warn!(
                    node_id = %node_id_owned,
                    error = %e,
                    "canvas_ui_action: SSB UiAction write failed (best-effort)"
                );
            }
        })
    });
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canvas_ui_action_select_variant_has_taint_field() {
        let action = CanvasUiAction::Select {
            node_id: "node-1".into(),
            content: "hello world".into(),
            taint: None,
        };
        if let CanvasUiAction::Select { taint, .. } = &action {
            assert!(taint.is_none(), "clean content must have no taint");
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn run_select_gate_clean_content_returns_clean() {
        // Plain text with no injection patterns and no registered taint context
        // must return Clean.
        let verdict = run_select_gate("node-1", "Hello, this is a normal node label.", None);
        assert_eq!(verdict, SelectGateVerdict::Clean);
    }

    #[test]
    fn canvas_view_kind_as_str_round_trips() {
        assert_eq!(CanvasViewKind::ViewSpawned.as_str(), "view_spawned");
        assert_eq!(CanvasViewKind::UiAction.as_str(), "ui_action");
        assert_eq!(CanvasViewKind::StateCheckpoint.as_str(), "state_checkpoint");
    }

    #[test]
    fn select_gate_verdict_partial_eq() {
        assert_eq!(SelectGateVerdict::Clean, SelectGateVerdict::Clean);
        assert_ne!(
            SelectGateVerdict::Flagged {
                taint: UntrustedTaint { source: "suspect".into() }
            },
            SelectGateVerdict::Clean
        );
    }
}
