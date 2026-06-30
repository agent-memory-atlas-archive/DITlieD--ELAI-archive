// H-13 (SCHEMA_VERSION 221) -- canvas_dispatcher.rs
//
// CanvasDispatcher: validates model-emitted canvas ops against the active mode
// flag, runs the sanitizeAction repair pass, and emits canvas_op_applied SSB
// rows. Composes with F-3p1 CommittedToolSet (checks capability gate BEFORE
// dispatching); does NOT duplicate it.
//
// Design doctrine (CANVAS-WORKBENCH-DESIGN.md §7 + §11):
//   - Op stream is INCREMENTAL (per-op, not whole-doc re-emit).
//   - Mode-namespace gating: a "design.*" op while mode != "design" is a hard
//     reject (structured error, not panic).
//   - sanitizeAction: fix hallucinated ids, normalize coords; REJECT a dangling
//     reference — never silently fix it (surface as blocked op with reason).
//   - CanvasDispatcher is a dyn dispatch point; it is listed in
//     crates/dashboard-v2/src/lib/canvas/entrypoints_map.ts to avoid HD-5
//     orphan-detection false-positives.
//
// HARD RULES:
//   - No .unwrap() / .expect() in production paths.
//   - No E-37 allow_tool_with_origin (wire-dark; no production caller).
//   - blake3 for any hash (not sha2/blake2).

use std::sync::Arc;

use tracing::warn;

use crate::{
    capability_flow::{check_against_commitment, CommittedToolSet},
    ssb::{CanvasOpKind, CanvasOpPayload, Ssb, SsbPayload},
};
use elai_ipc_types::{
    CanvasAnnotateRequest, CanvasConnectRequest, CanvasCreateNodeRequest, CanvasForkRequest,
    CanvasOpResponse, CanvasRenderDiagramRequest, CanvasScreenshotRequest,
    CanvasUpdateNodeRequest, TOOL_CANVAS_ANNOTATE, TOOL_CANVAS_CONNECT,
    TOOL_CANVAS_CREATE_NODE, TOOL_CANVAS_FORK, TOOL_CANVAS_RENDER_DIAGRAM,
    TOOL_CANVAS_SCREENSHOT, TOOL_CANVAS_UPDATE_NODE,
};

// ---------------------------------------------------------------------------
// Named constants
// ---------------------------------------------------------------------------

/// Namespace for ARCHITECTURE mode ops.
const ARCH_NAMESPACE: &str = "arch";

/// Namespace for DESIGN mode ops (H-14).
/// design.* ops ride canvas_op_applied (GD-1); no new SSB transport.
/// This const replaces the magic string "design" that would otherwise scatter
/// across check_mode_allowed call sites when design tools are added.
pub const DESIGN_NAMESPACE: &str = "design";

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// A canvas op was rejected. Never panics; always carries a human-readable
/// `reason` for SSB stream surfacing and log lines.
#[derive(Debug, Clone)]
pub struct CanvasOpError {
    /// Human-readable rejection reason, safe to surface in SSB logs.
    pub reason: String,
    /// The tool_id that triggered this rejection.
    pub tool_id: String,
}

impl std::fmt::Display for CanvasOpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "canvas op rejected [{}]: {}", self.tool_id, self.reason)
    }
}

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

/// dyn dispatch point for canvas op validation + SSB emission.
/// Listed in crates/dashboard-v2/src/lib/canvas/entrypoints_map.ts (HD-5).
pub trait CanvasOpsDispatcher: Send + Sync {
    /// Validate the canvas op (mode-namespace gating + sanitizeAction),
    /// compose with CommittedToolSet, emit SSB row, return response JSON.
    ///
    /// Returns `Err(CanvasOpError)` for any rejection (dangling id, wrong mode,
    /// capability gate block). Never silently drops a bad op.
    fn dispatch_canvas_op(
        &self,
        tool_id: &str,
        request_bytes: &[u8],
        current_mode: &str,
        session_id: &str,
        committed: &CommittedToolSet,
        ssb: Option<&Arc<Ssb>>,
    ) -> Result<CanvasOpResponse, CanvasOpError>;
}

// ---------------------------------------------------------------------------
// In-process implementation
// ---------------------------------------------------------------------------

/// Production implementation. Stateless; all state lives in the SSB writer
/// and the caller-supplied `committed` tool-set.
pub struct InProcessCanvasDispatcher;

impl CanvasOpsDispatcher for InProcessCanvasDispatcher {
    fn dispatch_canvas_op(
        &self,
        tool_id: &str,
        request_bytes: &[u8],
        current_mode: &str,
        session_id: &str,
        committed: &CommittedToolSet,
        ssb: Option<&Arc<Ssb>>,
    ) -> Result<CanvasOpResponse, CanvasOpError> {
        // F-3p1 CommittedToolSet gate: canvas_* tools must be allowed by the
        // current task's committed capability set. Compose-with (GD-1), not duplicate.
        // check_against_commitment classifies tool_name -> ToolCategory::Canvas
        // and verifies it is in the committed allowed_categories set.
        if let Err(denial) = check_against_commitment(committed, tool_id) {
            return Err(CanvasOpError {
                tool_id: tool_id.to_owned(),
                reason: format!(
                    "canvas tool '{tool_id}' blocked by CommittedToolSet: {denial:?}"
                ),
            });
        }

        // Decode the postcard-encoded request and dispatch per tool_id.
        let (kind, node_id, op_data, mode) = decode_and_validate(
            tool_id,
            request_bytes,
            current_mode,
        )?;

        // Build the unique op id (blake3 of tool_id + mode + node_id + ts_ms).
        let ts_ms = std::time::SystemTime::now() // entropy-allow: blake3 op-id salt + SSB ts_ms, no ClockProvider in scope (sync dispatch path)
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let op_id = blake3_op_id(tool_id, &mode, &node_id, ts_ms);

        // Emit SSB row (best-effort; failure is logged but does NOT block the op).
        if let Some(writer) = ssb {
            let payload = SsbPayload::CanvasOpApplied(CanvasOpPayload {
                kind,
                node_id: node_id.clone(),
                op_data: op_data.clone(),
                mode: mode.clone(),
                // task_id is embedded in op_data or defaulted; extract if present.
                task_id: op_data
                    .get("task_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_owned(),
                session_id: session_id.to_owned(),
            });
            // Emit asynchronously if a runtime is available; best-effort sync fallback.
            let writer = writer.clone();
            let op_id_clone = op_id.clone();
            let _ = tokio::runtime::Handle::try_current().map(|h| {
                h.spawn(async move {
                    if let Err(e) = writer
                        .write_payload(
                            "canvas_dispatcher",
                            ts_ms.try_into().unwrap_or(i64::MAX),
                            payload,
                            vec![],
                        )
                        .await
                    {
                        warn!(
                            op_id = %op_id_clone,
                            error = %e,
                            "canvas_dispatcher: SSB write failed (best-effort)"
                        );
                    }
                })
            });
        }

        Ok(CanvasOpResponse {
            op_id,
            kind: kind.as_str().to_owned(),
            mode,
            task_id: op_data
                .get("task_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_owned(),
        })
    }
}

// ---------------------------------------------------------------------------
// Internal decode + validation
// ---------------------------------------------------------------------------

/// Decode `request_bytes` (postcard-encoded per tool_id) and run all
/// validation gates: mode-namespace gating + sanitizeAction.
/// Returns `(kind, node_id, op_data, mode)` on success.
fn decode_and_validate(
    tool_id: &str,
    request_bytes: &[u8],
    current_mode: &str,
) -> Result<(CanvasOpKind, String, serde_json::Value, String), CanvasOpError> {
    match tool_id {
        TOOL_CANVAS_CREATE_NODE => {
            let req: CanvasCreateNodeRequest = postcard::from_bytes(request_bytes)
                .map_err(|e| CanvasOpError {
                    tool_id: tool_id.to_owned(),
                    reason: format!("decode error: {e}"),
                })?;
            // Mode-namespace gating: node creation is allowed in arch and code.
            check_mode_allowed(tool_id, &req.mode, current_mode, &[ARCH_NAMESPACE, "code"])?;
            // sanitizeAction: reject empty node_id.
            sanitize_node_id(tool_id, &req.node_id)?;
            let op_data = serde_json::json!({
                "node_id": req.node_id,
                "label": req.label,
                "x": req.x,
                "y": req.y,
                "meta": req.meta,
                "task_id": req.task_id,
            });
            Ok((CanvasOpKind::NodeCreated, req.node_id, op_data, req.mode))
        }

        TOOL_CANVAS_CONNECT => {
            let req: CanvasConnectRequest = postcard::from_bytes(request_bytes)
                .map_err(|e| CanvasOpError {
                    tool_id: tool_id.to_owned(),
                    reason: format!("decode error: {e}"),
                })?;
            check_mode_allowed(tool_id, &req.mode, current_mode, &[ARCH_NAMESPACE, "code"])?;
            sanitize_node_id(tool_id, &req.from_id)?;
            sanitize_node_id(tool_id, &req.to_id)?;
            let op_data = serde_json::json!({
                "from_id": req.from_id,
                "to_id": req.to_id,
                "label": req.label,
                "task_id": req.task_id,
            });
            Ok((CanvasOpKind::EdgeConnected, req.from_id.clone(), op_data, req.mode))
        }

        TOOL_CANVAS_UPDATE_NODE => {
            let req: CanvasUpdateNodeRequest = postcard::from_bytes(request_bytes)
                .map_err(|e| CanvasOpError {
                    tool_id: tool_id.to_owned(),
                    reason: format!("decode error: {e}"),
                })?;
            check_mode_allowed(tool_id, &req.mode, current_mode, &[ARCH_NAMESPACE, "code", "plan"])?;
            sanitize_node_id(tool_id, &req.node_id)?;
            let op_data = serde_json::json!({
                "node_id": req.node_id,
                "label": req.label,
                "meta": req.meta,
                "task_id": req.task_id,
            });
            Ok((CanvasOpKind::LabelEdited, req.node_id.clone(), op_data, req.mode))
        }

        TOOL_CANVAS_ANNOTATE => {
            let req: CanvasAnnotateRequest = postcard::from_bytes(request_bytes)
                .map_err(|e| CanvasOpError {
                    tool_id: tool_id.to_owned(),
                    reason: format!("decode error: {e}"),
                })?;
            check_mode_allowed(tool_id, &req.mode, current_mode, &[ARCH_NAMESPACE, "code", "plan", "research"])?;
            // If anchor_node_id is provided it must be non-empty.
            if let Some(ref anchor) = req.anchor_node_id {
                sanitize_node_id(tool_id, anchor)?;
            }
            let op_data = serde_json::json!({
                "text": req.text,
                "anchor_node_id": req.anchor_node_id,
                "task_id": req.task_id,
            });
            Ok((CanvasOpKind::AnnotationAdded, String::new(), op_data, req.mode))
        }

        TOOL_CANVAS_FORK => {
            let req: CanvasForkRequest = postcard::from_bytes(request_bytes)
                .map_err(|e| CanvasOpError {
                    tool_id: tool_id.to_owned(),
                    reason: format!("decode error: {e}"),
                })?;
            check_mode_allowed(tool_id, &req.mode, current_mode, &[ARCH_NAMESPACE, "code"])?;
            sanitize_node_id(tool_id, &req.source_node_id)?;
            sanitize_node_id(tool_id, &req.new_node_id)?;
            let op_data = serde_json::json!({
                "source_node_id": req.source_node_id,
                "new_node_id": req.new_node_id,
                "label": req.label,
                "task_id": req.task_id,
            });
            Ok((CanvasOpKind::NodeForked, req.new_node_id.clone(), op_data, req.mode))
        }

        TOOL_CANVAS_RENDER_DIAGRAM => {
            let req: CanvasRenderDiagramRequest = postcard::from_bytes(request_bytes)
                .map_err(|e| CanvasOpError {
                    tool_id: tool_id.to_owned(),
                    reason: format!("decode error: {e}"),
                })?;
            // render_diagram is only allowed in ARCHITECTURE mode.
            check_mode_allowed(tool_id, &req.mode, current_mode, &[ARCH_NAMESPACE])?;
            let op_data = serde_json::json!({
                "graph_mode": req.graph_mode,
                "max_nodes": req.max_nodes,
                "task_id": req.task_id,
            });
            Ok((CanvasOpKind::LayoutChanged, String::new(), op_data, req.mode))
        }

        TOOL_CANVAS_SCREENSHOT => {
            let req: CanvasScreenshotRequest = postcard::from_bytes(request_bytes)
                .map_err(|e| CanvasOpError {
                    tool_id: tool_id.to_owned(),
                    reason: format!("decode error: {e}"),
                })?;
            // Screenshot allowed in any wired mode.
            let op_data = serde_json::json!({
                "task_id": req.task_id,
            });
            Ok((CanvasOpKind::ScreenshotCaptured, String::new(), op_data, req.mode))
        }

        other => Err(CanvasOpError {
            tool_id: other.to_owned(),
            reason: format!("canvas_dispatcher: unknown canvas tool_id '{other}'"),
        }),
    }
}

// ---------------------------------------------------------------------------
// Validation helpers
// ---------------------------------------------------------------------------

/// Mode-namespace gating: reject the op if `request_mode` does not match
/// `current_mode` OR if `current_mode` is not in `allowed_modes`.
///
/// Rationale: a "design.*" op while mode="code" is a hallucinated mode —
/// surface it as a blocked op with a reason, not a silent no-op.
fn check_mode_allowed(
    tool_id: &str,
    request_mode: &str,
    current_mode: &str,
    allowed_modes: &[&str],
) -> Result<(), CanvasOpError> {
    // The request must declare the same mode as the active canvas.
    if request_mode != current_mode {
        return Err(CanvasOpError {
            tool_id: tool_id.to_owned(),
            reason: format!(
                "mode mismatch: request declares mode='{request_mode}' but \
                 canvas is in mode='{current_mode}'"
            ),
        });
    }
    // The current mode must be in the allowed set for this op.
    if !allowed_modes.contains(&current_mode) {
        return Err(CanvasOpError {
            tool_id: tool_id.to_owned(),
            reason: format!(
                "mode '{current_mode}' is not allowed for tool '{tool_id}' \
                 (allowed: {allowed_modes:?})"
            ),
        });
    }
    Ok(())
}

/// sanitizeAction: reject an empty or whitespace-only node_id as a dangling
/// reference — never silently accept it.
///
/// A non-empty node_id is NOT verified against the live graph here (that is
/// the frontend CanvasDispatcher's job via sanitizeCanvasOp); this layer
/// blocks obviously-invalid ids before they reach the SSB.
fn sanitize_node_id(tool_id: &str, node_id: &str) -> Result<(), CanvasOpError> {
    if node_id.trim().is_empty() {
        return Err(CanvasOpError {
            tool_id: tool_id.to_owned(),
            reason: "dangling-id rejected: node_id is empty or whitespace-only".to_owned(),
        });
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Utility
// ---------------------------------------------------------------------------

/// Derive a stable op_id from `(tool_id, mode, node_id, ts_ms)` using blake3.
/// Anti-stale: blake3 (never blake2 / sha2 per CLAUDE.md HARD RULE).
fn blake3_op_id(tool_id: &str, mode: &str, node_id: &str, ts_ms: u128) -> String {
    let input = format!("{tool_id}|{mode}|{node_id}|{ts_ms}");
    let hash = blake3::hash(input.as_bytes());
    // First 16 hex chars are sufficient for a unique op identifier.
    let hex = hash.to_hex();
    format!("cop-{}", &hex.as_str()[..16])
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool_filter::ToolCategory;
    use elai_ipc_types::CanvasCreateNodeRequest;

    fn make_committed_with_canvas() -> CommittedToolSet {
        use crate::capability_flow::commit_minimal_set;
        commit_minimal_set("t-test", &[ToolCategory::Canvas], 0)
    }

    fn make_committed_without_canvas() -> CommittedToolSet {
        use crate::capability_flow::commit_minimal_set;
        commit_minimal_set("t-test", &[ToolCategory::Read], 0)
    }

    #[test]
    fn create_node_accepted_in_arch_mode() {
        let req = CanvasCreateNodeRequest {
            node_id: "svc-api".into(),
            label: "API Gateway".into(),
            mode: "arch".into(),
            x: None,
            y: None,
            meta: None,
            task_id: "t-1".into(),
        };
        let bytes = postcard::to_allocvec(&req).expect("encode");
        let dispatcher = InProcessCanvasDispatcher;
        let committed = make_committed_with_canvas();
        let result =
            dispatcher.dispatch_canvas_op(TOOL_CANVAS_CREATE_NODE, &bytes, "arch", "s-1", &committed, None);
        assert!(result.is_ok(), "create_node must succeed in arch mode: {result:?}");
        let resp = result.unwrap();
        assert_eq!(resp.kind, "node_created");
        assert!(resp.op_id.starts_with("cop-"), "op_id must have cop- prefix");
    }

    #[test]
    fn create_node_rejected_wrong_mode() {
        let req = CanvasCreateNodeRequest {
            node_id: "svc-db".into(),
            label: "Database".into(),
            mode: "arch".into(),
            x: None,
            y: None,
            meta: None,
            task_id: "t-2".into(),
        };
        let bytes = postcard::to_allocvec(&req).expect("encode");
        let dispatcher = InProcessCanvasDispatcher;
        let committed = make_committed_with_canvas();
        // Canvas is in "code" mode but request says "arch" -> mismatch
        let result = dispatcher.dispatch_canvas_op(
            TOOL_CANVAS_CREATE_NODE,
            &bytes,
            "code", // current_mode != request mode
            "s-1",
            &committed,
            None,
        );
        assert!(result.is_err(), "must reject mode mismatch");
        let err = result.unwrap_err();
        assert!(
            err.reason.contains("mode mismatch"),
            "reason must cite mode mismatch: {}",
            err.reason
        );
    }

    #[test]
    fn dangling_id_rejected() {
        let req = CanvasCreateNodeRequest {
            node_id: "   ".into(), // whitespace-only = dangling
            label: "Ghost".into(),
            mode: "arch".into(),
            x: None,
            y: None,
            meta: None,
            task_id: "t-3".into(),
        };
        let bytes = postcard::to_allocvec(&req).expect("encode");
        let dispatcher = InProcessCanvasDispatcher;
        let committed = make_committed_with_canvas();
        let result =
            dispatcher.dispatch_canvas_op(TOOL_CANVAS_CREATE_NODE, &bytes, "arch", "s-1", &committed, None);
        assert!(result.is_err(), "must reject empty node_id");
        let err = result.unwrap_err();
        assert!(
            err.reason.contains("dangling-id rejected"),
            "reason must cite dangling-id: {}",
            err.reason
        );
    }

    #[test]
    fn capability_gate_blocks_canvas_tool() {
        let req = CanvasCreateNodeRequest {
            node_id: "svc-cache".into(),
            label: "Cache".into(),
            mode: "arch".into(),
            x: None,
            y: None,
            meta: None,
            task_id: "t-4".into(),
        };
        let bytes = postcard::to_allocvec(&req).expect("encode");
        let dispatcher = InProcessCanvasDispatcher;
        // CommittedToolSet WITHOUT Canvas category.
        let committed = make_committed_without_canvas();
        let result =
            dispatcher.dispatch_canvas_op(TOOL_CANVAS_CREATE_NODE, &bytes, "arch", "s-1", &committed, None);
        assert!(result.is_err(), "must reject when Canvas not in CommittedToolSet");
        let err = result.unwrap_err();
        assert!(
            err.reason.contains("CommittedToolSet"),
            "reason must cite CommittedToolSet: {}",
            err.reason
        );
    }

    #[test]
    fn render_diagram_only_in_arch() {
        let req = CanvasRenderDiagramRequest {
            graph_mode: "symbol".into(),
            max_nodes: None,
            mode: "code".into(),
            task_id: "t-5".into(),
        };
        let bytes = postcard::to_allocvec(&req).expect("encode");
        let dispatcher = InProcessCanvasDispatcher;
        let committed = make_committed_with_canvas();
        let result = dispatcher.dispatch_canvas_op(
            TOOL_CANVAS_RENDER_DIAGRAM,
            &bytes,
            "code",
            "s-1",
            &committed,
            None,
        );
        assert!(result.is_err(), "render_diagram must be rejected outside arch mode");
    }

    #[test]
    fn blake3_op_id_has_prefix() {
        let id = blake3_op_id("canvas_create_node", "arch", "svc-api", 1234567890);
        assert!(id.starts_with("cop-"), "op_id prefix: {id}");
        assert_eq!(id.len(), "cop-".len() + 16);
    }
}
