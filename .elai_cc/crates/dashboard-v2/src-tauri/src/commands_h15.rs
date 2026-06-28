//! H-15 Tauri commands — canvas view reply channel + taint-gated Select.
//!
//! This module provides two command surfaces:
//!
//! - [`get_canvas_views`]: poll recent `canvas_view_event` SSB rows for
//!   canvasViewState.svelte.ts's live snapshot/delta reconstruction loop.
//!   Mirrors H-13's `get_canvas_ops` pattern. SEPARATE drain from canvas_op_applied.
//!
//! - [`canvas_select_node`]: inbound CanvasUiAction::Select gate. When the user
//!   selects a canvas node, this command:
//!     (1) runs the G-1 taint gate (taint_text + TaintFlowCtx lookup);
//!     (2) emits a canvas_view_event(UiAction) SSB row (always, for journaling);
//!     (3) returns the node content + provenance flag to the frontend.
//!     A tainted node's content is FLAGGED (never silently re-enters the model).
//!
//! Security (CANVAS §19): the Claudy Day chain exfiltrated data via allowlisted
//! egress despite strict CSP. The Select gate is the enforcement point: untrusted
//! content cannot silently re-enter the model prompt without the provenance flag.
//!
//! Cross-lang pair:
//!   - `get_canvas_views` <-> `canvasViewState.svelte.ts` `get_canvas_views` call.
//!   - `canvas_select_node` <-> `canvasViewState.svelte.ts` `canvas_select_node` call.
//!
//! H-15-decisions.md D-2 (taint gate design), D-9 (drain separation).

use std::sync::Arc;

use elai_orchestrator::{
    canvas_ui_action::{emit_select_ssb_row, run_select_gate, SelectGateVerdict},
    ssb::{CanvasViewKind, CanvasViewPayload, SsbActionType, SsbPayload},
};
#[cfg(test)]
use elai_orchestrator::ssb::CANVAS_CHECKPOINT_INTERVAL_OPS;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

// ---------------------------------------------------------------------------
// Named constants (no magic numbers — CLAUDE.md HARD RULE)
// ---------------------------------------------------------------------------

/// Default poll limit for get_canvas_views. Server caps at 100.
const DEFAULT_VIEW_POLL_LIMIT: u32 = 20;

/// Maximum view events to return per poll call.
const MAX_VIEW_POLL_LIMIT: u32 = 100;

// ---------------------------------------------------------------------------
// DTO types
// ---------------------------------------------------------------------------

/// A recent canvas view event returned by `get_canvas_views`.
/// Mirrors the Rust `CanvasViewPayload` shape for canvasViewState.svelte.ts.
/// Cross-lang pair: `CanvasViewEventDto` in `src/lib/api/stub.ts`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasViewEventDto {
    /// View event kind wire label ("view_spawned", "ui_action", "state_checkpoint").
    pub kind: String,
    /// Renderer view_type key (populated for ViewSpawned; empty for others).
    pub view_type: String,
    /// Stable view instance identifier for per-tab filtering.
    pub view_id: String,
    /// Seed op ids (ViewSpawned only; empty for UiAction / StateCheckpoint).
    pub seed_op_ids: Vec<String>,
    /// Selected node id (UiAction only; empty otherwise).
    pub node_id: String,
    /// True if the G-1 taint gate flagged this node's content at Select handoff.
    pub taint_flagged: bool,
    /// Canonical task id.
    pub task_id: String,
    /// Session id for correlation.
    pub session_id: String,
    /// SSB row sequence number (monotonic rebuild key for snapshot/delta).
    pub seq: i64,
    /// SSB row timestamp (unix ms).
    pub ts: i64,
}

/// Response from `canvas_select_node`.
/// Returns the node content + provenance verdict to the frontend.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasSelectNodeResponse {
    /// The selected node id.
    pub node_id: String,
    /// The node content (text label / annotation / serialized state).
    /// Always returned; the frontend must check `taint_flagged` before use.
    pub content: String,
    /// True if the G-1 taint gate flagged this content (provenance flag required).
    pub taint_flagged: bool,
    /// The taint source label if flagged (empty if clean).
    pub taint_source: String,
    /// False if the gate BLOCKED the handoff (TrustLevel::Untrusted).
    /// The frontend must NOT pass content to the model if `allowed` is false.
    pub allowed: bool,
    /// Human-readable block reason (empty if allowed=true).
    pub block_reason: String,
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// Return the N most recent `canvas_view_event` SSB rows for the
/// canvasViewState.svelte.ts snapshot/delta reconstruction loop.
///
/// `limit` defaults to 20, capped at 100 server-side.
/// Returns an empty list when no viewer is attached or no rows have landed.
///
/// DRAIN SEPARATION: this command polls canvas_view_event ONLY. The
/// canvas_op_applied drain is `get_canvas_ops` (commands_h13.rs). They must
/// NEVER be merged (wiring-enforcement R6; gotcha OP_APPLICATION_TRANSPORT_IS_H13).
#[tauri::command]
pub async fn get_canvas_views(
    state: State<'_, AppState>,
    limit: Option<u32>,
) -> Result<Vec<CanvasViewEventDto>, String> {
    let n = limit.unwrap_or(DEFAULT_VIEW_POLL_LIMIT).min(MAX_VIEW_POLL_LIMIT) as usize;
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(vec![]),
    };

    let raw = viewer
        .recent_rows_of_type(SsbActionType::CanvasViewEvent.as_sql_str(), n)
        .map_err(|e| e.to_string())?;

    let mut result = Vec::with_capacity(raw.len());
    for entry in raw {
        let pl: SsbPayload = match serde_json::from_value(entry.payload_json.clone()) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if let SsbPayload::CanvasViewEvent(p) = pl {
            result.push(CanvasViewEventDto {
                kind: p.kind.as_str().to_owned(),
                view_type: p.view_type,
                view_id: p.view_id,
                seed_op_ids: p.seed_op_ids,
                node_id: p.node_id,
                taint_flagged: p.taint_flagged,
                task_id: p.task_id,
                session_id: p.session_id,
                seq: entry.seq,
                ts: entry.ts,
            });
        }
    }
    Ok(result)
}

/// Inbound CanvasUiAction::Select — taint-gated node content handoff.
///
/// When the user selects a canvas node, this command:
///   (1) Runs the G-1 taint gate (taint_text + TaintFlowCtx lookup).
///   (2) ALWAYS emits a canvas_view_event(UiAction) SSB row (journaling).
///   (3) Returns node content + provenance flag. Frontend checks `allowed`
///       before passing content to the model.
///
/// Security: a tainted node's content MUST NOT silently re-enter the model
/// prompt. The `taint_flagged` + `allowed` fields enforce provenance.
/// See CANVAS §19 (Claudy Day chain) + H-15-decisions.md D-2.
///
/// Cross-lang pair: `canvasViewState.svelte.ts` `canvas_select_node` call.
#[tauri::command]
pub async fn canvas_select_node(
    state: State<'_, AppState>,
    node_id: String,
    content: String,
    task_id: Option<String>,
) -> Result<CanvasSelectNodeResponse, String> {
    let task_id = task_id.unwrap_or_else(|| "h15-select-unknown".to_owned());
    let session_id = state
        .writer
        .as_ref()
        .map(|w| w.session_id().to_owned())
        .unwrap_or_default();

    let ssb_arc: Option<Arc<elai_orchestrator::ssb::Ssb>> = state.writer.clone();

    // Run the two-leg taint gate (GD-1: reuses taint_text + G-1 chain).
    let verdict = run_select_gate(&node_id, &content, ssb_arc.as_ref());

    let (taint_flagged, taint_source, allowed, block_reason) = match &verdict {
        SelectGateVerdict::Clean => (false, String::new(), true, String::new()),
        SelectGateVerdict::Flagged { taint } => {
            (true, taint.source.clone(), true, String::new())
        }
        SelectGateVerdict::Blocked { reason } => {
            (true, String::new(), false, reason.clone())
        }
    };

    // ALWAYS emit canvas_view_event(UiAction) row for journaling (HD-4).
    if let Some(ssb) = &ssb_arc {
        emit_select_ssb_row(ssb, &node_id, taint_flagged, &task_id, &session_id);
    }

    Ok(CanvasSelectNodeResponse {
        node_id,
        content,
        taint_flagged,
        taint_source,
        allowed,
        block_reason,
    })
}

/// Emit a StateCheckpoint canvas_view_event row for per-tab snapshot/delta.
///
/// Called by the frontend canvasViewState.svelte.ts store every
/// CANVAS_CHECKPOINT_INTERVAL_OPS view events (or on tab focus-gained).
/// The emitted row's SSB `seq` becomes the monotonic rebuild key:
/// kill+reopen the dashboard and the tab reconstructs from the SSB tail
/// without replaying every individual op (H15-3 slice).
///
/// Returns the seq of the emitted row (or 0 if SSB not available).
#[tauri::command]
pub async fn emit_canvas_checkpoint(
    state: State<'_, AppState>,
    view_id: String,
    task_id: Option<String>,
) -> Result<i64, String> {
    let task_id = task_id.unwrap_or_else(|| "h15-checkpoint-unknown".to_owned());
    let session_id = state
        .writer
        .as_ref()
        .map(|w| w.session_id().to_owned())
        .unwrap_or_default();

    let writer = match &state.writer {
        Some(w) => w.clone(),
        None => return Ok(0),
    };

    let payload = SsbPayload::CanvasViewEvent(CanvasViewPayload {
        kind: CanvasViewKind::StateCheckpoint,
        view_type: String::new(),
        view_id,
        seed_op_ids: vec![],
        node_id: String::new(),
        taint_flagged: false,
        task_id,
        session_id,
    });

    let ts_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0);

    let seq = writer
        .write_payload("canvas_checkpoint_emitter", ts_ms, payload, vec![])
        .await
        .map_err(|e| e.to_string())?;

    Ok(seq)
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canvas_view_event_dto_serializes_expected_fields() {
        let dto = CanvasViewEventDto {
            kind: "view_spawned".into(),
            view_type: "comparison_board".into(),
            view_id: "view-test-1".into(),
            seed_op_ids: vec!["op-1".into(), "op-2".into()],
            node_id: String::new(),
            taint_flagged: false,
            task_id: "t-h15-unit".into(),
            session_id: "s-h15-unit".into(),
            seq: 42,
            ts: 1_000_000,
        };
        let json = serde_json::to_value(&dto).expect("serialize");
        assert_eq!(json["kind"], "view_spawned");
        assert_eq!(json["view_type"], "comparison_board");
        assert_eq!(json["taint_flagged"], false);
        assert_eq!(json["seq"], 42);
    }

    #[test]
    fn canvas_select_response_allowed_false_on_block() {
        let resp = CanvasSelectNodeResponse {
            node_id: "node-1".into(),
            content: "some content".into(),
            taint_flagged: true,
            taint_source: String::new(),
            allowed: false,
            block_reason: "blocked by G-1 gate".into(),
        };
        assert!(!resp.allowed);
        assert!(!resp.block_reason.is_empty());
    }

    #[test]
    fn checkpoint_interval_constant_is_positive() {
        assert!(CANVAS_CHECKPOINT_INTERVAL_OPS > 0);
        assert!(
            CANVAS_CHECKPOINT_INTERVAL_OPS <= 100,
            "checkpoint interval must be <=100 ops for responsive tab reconstruction"
        );
    }
}
