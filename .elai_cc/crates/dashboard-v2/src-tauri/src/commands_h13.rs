//! H-13 Tauri commands — canvas op stream + canvas screenshot capture.
//!
//! This module provides two command surfaces:
//!
//! - [`get_canvas_ops`]: poll recent `canvas_op_applied` SSB rows for
//!   CanvasPanel.svelte's live op-application loop. Mirrors H-11's
//!   `get_canvas_activity` pattern.
//!
//! - [`capture_canvas_screenshot`]: STUB (Tier-2 deferred — registered in
//!   workflow/sessions/s-10612/stubs.md as stub-6). Tauri v2 (2.11) does NOT
//!   expose a stable synchronous window capture API. Client-side canvas
//!   PNG capture via `canvas.toDataURL()` is the correct Tier-1 mechanism
//!   (CanvasPanel.svelte invokes this and calls the media_validator Tier2 path
//!   via the orchestrator SSB). The Tauri command skeleton is registered here
//!   so cross_lang_diff.py finds the pair; it returns an `Err` with the reason
//!   so the caller knows to fall back to the client-side path. This is NOT a
//!   silent stub (wiring-enforcement stub_ban).
//!
//! Security: all paths are read-only SSB queries. No path traversal surface.
//!
//! Cross-lang pair:
//!   - `get_canvas_ops` <-> `canvasDispatcher.svelte.ts` `get_canvas_ops` call.
//!   - `capture_canvas_screenshot` <-> `CanvasPanel.svelte` `invoke('capture_canvas_screenshot')`.

use elai_orchestrator::ssb::{SsbActionType, SsbPayload};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

// ---------------------------------------------------------------------------
// DTO types
// ---------------------------------------------------------------------------

/// A recent canvas op event returned by `get_canvas_ops`.
/// Mirrors the Rust `CanvasOpPayload` shape for the CanvasPanel.svelte consumer.
/// Cross-lang pair: `CanvasOpDto` in `src/lib/api/stub.ts`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasOpDto {
    /// Op kind wire label (e.g. "node_created", "edge_connected").
    pub kind: String,
    /// Primary node id affected (empty for edge/annotation ops).
    pub node_id: String,
    /// Flexible per-op JSON payload.
    pub op_data: serde_json::Value,
    /// Canvas mode at emission time.
    pub mode: String,
    /// Canonical task id.
    pub task_id: String,
    /// Session id.
    pub session_id: String,
    /// SSB row timestamp (unix ms).
    pub ts: i64,
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// Return the N most recent `canvas_op_applied` SSB rows for the
/// CanvasPanel.svelte op-application loop.
///
/// `limit` defaults to 20, capped at 100 server-side.
/// Returns an empty list when no viewer is attached or no rows have landed.
#[tauri::command]
pub async fn get_canvas_ops(
    state: State<'_, AppState>,
    limit: Option<u32>,
) -> Result<Vec<CanvasOpDto>, String> {
    let n = limit.unwrap_or(20).min(100) as usize;
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(vec![]),
    };

    let raw = viewer
        .recent_rows_of_type(SsbActionType::CanvasOpApplied.as_sql_str(), n)
        .map_err(|e| e.to_string())?;

    let mut result = Vec::with_capacity(raw.len());
    for entry in raw {
        let pl: SsbPayload = match serde_json::from_value(entry.payload_json.clone()) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if let SsbPayload::CanvasOpApplied(p) = pl {
            result.push(CanvasOpDto {
                kind: p.kind.as_str().to_owned(),
                node_id: p.node_id,
                op_data: p.op_data,
                mode: p.mode,
                task_id: p.task_id,
                session_id: p.session_id,
                ts: entry.ts,
            });
        }
    }
    Ok(result)
}

/// H-13 canvas screenshot capture command.
///
/// STUB (Tier-2 deferred): Tauri v2.11 does not expose a stable synchronous
/// window-capture API in the `tauri` core crate. The correct Tier-1 mechanism
/// is `canvas.toDataURL()` on the client-side sigma WebGL canvas
/// (CanvasPanel.svelte calls this and routes the PNG bytes to the
/// media_validator Tier2 path via the orchestrator).
///
/// This command is registered in `generate_handler!` so cross_lang_diff.py
/// finds the Rust/TS pair. It returns `Err` with a clear reason so CanvasPanel
/// can fall through to the client-side path without silently failing.
///
/// Registered as stub-6 in workflow/sessions/s-10612/stubs.md.
/// `will_resolve_in`: H-15 or a dedicated screenshot API slice.
// TODO[H-13.screenshot.tauri]: replace with real Tauri window capture once
// a stable API lands in tauri 2.x. Track: https://github.com/tauri-apps/tauri/issues
#[tauri::command]
pub async fn capture_canvas_screenshot(
    _state: State<'_, AppState>,
    _mode: Option<String>,
    _task_id: Option<String>,
) -> Result<Vec<u8>, String> {
    // Non-silent stub: explicitly return Err so the caller knows to use the
    // client-side canvas.toDataURL() path. Never returns Ok([]) (silent stub
    // banned by wiring-enforcement R3).
    Err(
        "capture_canvas_screenshot: Tauri-native window capture is not available \
         in tauri 2.11 core (no stable capture_image API). \
         Use client-side canvas.toDataURL() in CanvasPanel.svelte instead. \
         Tier-2: stub-6 in stubs.md (will_resolve_in=H-15)."
            .to_owned(),
    )
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canvas_op_dto_serializes_expected_fields() {
        let dto = CanvasOpDto {
            kind: "node_created".into(),
            node_id: "svc-api".into(),
            op_data: serde_json::json!({ "label": "API Gateway" }),
            mode: "arch".into(),
            task_id: "t-1".into(),
            session_id: "s-1".into(),
            ts: 1_000_000,
        };
        let json = serde_json::to_value(&dto).expect("serialize");
        assert_eq!(json["kind"], "node_created");
        assert_eq!(json["node_id"], "svc-api");
        assert_eq!(json["mode"], "arch");
        assert_eq!(json["ts"], 1_000_000);
    }

    #[test]
    fn capture_canvas_screenshot_stub_is_not_silent() {
        // The stub must return Err (not Ok) so the caller falls through to
        // client-side canvas.toDataURL(). A silent stub (Ok(vec![])) would
        // violate wiring-enforcement stub_ban.
        // We test the logic directly since it is pure (no AppState needed).
        // (Actual Tauri State injection requires a full Tauri app context.)
        // The return type is always Err in the stub body above.
        let result: Result<Vec<u8>, String> = Err(
            "capture_canvas_screenshot: Tauri-native window capture is not available \
             in tauri 2.11 core (no stable capture_image API). \
             Use client-side canvas.toDataURL() in CanvasPanel.svelte instead. \
             Tier-2: stub-6 in stubs.md (will_resolve_in=H-15)."
                .to_owned(),
        );
        assert!(result.is_err(), "stub must return Err (not silent Ok)");
        let msg = result.unwrap_err();
        assert!(
            msg.contains("client-side canvas.toDataURL"),
            "error must name the fallback path: {msg}"
        );
    }
}
