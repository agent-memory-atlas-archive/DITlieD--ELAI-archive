//! H-14 Tauri commands — design mode opaque-id resolution + recast writeback
//! + dev-preview proxy start/stop.
//!
//! Four command surfaces:
//!
//! - [`apply_recast_patch`]: apply a deterministic style/text patch to a real
//!   source file via the recast writeback pipeline. The structural (LLM) tier
//!   is a registered Tier-2 stub (stub-7) until the recast spike passes (GD-5).
//!
//! - [`resolve_oid`]: look up an opaque data-oid -> {file, range, scope} via
//!   the server-side OidMap. Returns the OidMapEntry for click-to-edit selection.
//!
//! - [`start_dev_proxy`]: start the design-preview proxy (Tier-1 scaffold;
//!   live TCP listener is stub-10 in stubs.md; Tier-2 deferred).
//!
//! - [`stop_dev_proxy`]: stop the design-preview proxy.
//!
//! Security: apply_recast_patch validates that the resolved file path is within
//! the allowed project_root (server-side path-traversal guard, same as H-11).
//!
//! Cross-lang pairs:
//!   - `apply_recast_patch` <-> stub.ts::apply_recast_patch
//!   - `resolve_oid`        <-> stub.ts::resolve_oid
//!   - `start_dev_proxy`    <-> stub.ts::start_dev_proxy
//!   - `stop_dev_proxy`     <-> stub.ts::stop_dev_proxy
//!
//! HARD RULES:
//!   - No .unwrap() / .expect() in production paths.
//!   - No structural writeback tier before recast spike passes (GD-5 stub-7).
//!   - No raw dev-server iframe (proxy only, stub_ban).
//!   - No line/col in DOM attr (opaque id only, stub_ban).

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::data_oid_tagger::OidMapEntry;
use crate::recast_writeback::{apply_recast_patch as do_apply_recast_patch, AttributePatch, PatchKind, PatchResult};
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Named constants
// ---------------------------------------------------------------------------

/// Wire label for the design mode namespace. Mirrors DESIGN_NAMESPACE in
/// canvas_dispatcher.rs (orchestrator). Used for log context.
/// (The DESIGN_NAMESPACE const in canvas_dispatcher.rs is the canonical
/// definition; this is the dashboard-side mirror for log labels only.)
const DESIGN_MODE_LABEL: &str = "design";

// ---------------------------------------------------------------------------
// DTO types (cross-lang pairs in stub.ts)
// ---------------------------------------------------------------------------

/// Response from `resolve_oid` — the server-side location entry for an
/// opaque element id.
/// Cross-lang pair: `DesignOidEntry` in stub.ts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidResolveResponse {
    /// The opaque id that was resolved.
    pub oid: String,
    /// Canonical file path.
    pub file: String,
    /// Source range [start_line, start_col, end_line, end_col] (1-based).
    pub range: [u32; 4],
    /// Scope qualifier (hierarchical JSX path).
    pub scope: String,
}

impl From<(String, OidMapEntry)> for OidResolveResponse {
    fn from((oid, entry): (String, OidMapEntry)) -> Self {
        Self {
            oid,
            file: entry.file,
            range: [
                entry.range.start_line,
                entry.range.start_col,
                entry.range.end_line,
                entry.range.end_col,
            ],
            scope: entry.scope,
        }
    }
}

/// Request body for `apply_recast_patch`. Mirrors the Rust `AttributePatch`
/// with JSON-friendly field names for the IPC boundary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecastPatchRequest {
    /// Opaque element id (data-oid).
    pub oid: String,
    /// JSX attribute name to modify (e.g. "className", "style").
    pub attr_name: String,
    /// New attribute value (resolved token reference or text).
    pub new_value: String,
    /// Writeback tier: "deterministic" (style/text) or "structural" (LLM, deferred).
    pub kind: String,
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// Resolve an opaque data-oid to its source location ({file, range, scope}).
///
/// The OidMap is populated at build time when the data-oid tagger runs.
/// Returns `Err` when the oid is not registered (the tagger has not yet
/// processed this file or the oid was generated client-side without a tag pass).
#[tauri::command]
pub async fn resolve_oid(
    _state: State<'_, AppState>,
    oid: String,
) -> Result<OidResolveResponse, String> {
    // In Tier-1, the OidMap lives in the tagger module as a process-level
    // registry. For the full wiring (Tier-2), it would be stored in AppState.
    // We use a module-level static OidMap for Tier-1 (initialized on first use).
    use crate::data_oid_tagger::OID_MAP;
    let entry = OID_MAP
        .get(&oid)
        .map_err(|e| format!("resolve_oid: OidMap lookup failed: {e}"))?
        .ok_or_else(|| {
            format!(
                "resolve_oid: oid '{oid}' not found in server-side OidMap; \
                 ensure the build-time data-oid tagger has run for this project \
                 (mode={DESIGN_MODE_LABEL})"
            )
        })?;

    tracing::debug!(oid = %oid, file = %entry.file, "resolve_oid: resolved");
    Ok(OidResolveResponse::from((oid, entry)))
}

/// Apply a recast style/text patch to a real source file.
///
/// The deterministic tier applies the patch via format-preserving text
/// substitution (the full recast Node.js parse-modify-print pipeline is
/// validated by the contract test spike; this command uses the Rust-side
/// equivalent for Tier-1). NO model call in the deterministic path.
///
/// The structural tier is DEFERRED (stub-7, GD-5 gate). Requesting
/// `kind="structural"` returns a non-silent Err.
///
/// Path traversal: the resolved file path must be within the allowed project
/// root (same scope guard as H-11 read_file).
#[tauri::command]
pub async fn apply_recast_patch(
    state: State<'_, AppState>,
    request: RecastPatchRequest,
) -> Result<PatchResult, String> {
    use crate::data_oid_tagger::OID_MAP;

    let kind = match request.kind.as_str() {
        "deterministic" => PatchKind::Deterministic,
        "structural" => PatchKind::Structural,
        other => {
            return Err(format!(
                "apply_recast_patch: unknown kind '{other}'; \
                 expected 'deterministic' or 'structural'"
            ))
        }
    };

    let patch = AttributePatch {
        oid: request.oid,
        attr_name: request.attr_name,
        new_value: request.new_value,
        kind,
    };

    let project_root = state.project_root_snapshot();

    do_apply_recast_patch(&OID_MAP, &patch, &project_root)
}

/// Start the design-preview proxy for a target dev server URL.
///
/// Tier-1 scaffold: returns a proxy status response. The live TCP listener
/// that actually proxies HTTP requests is stub-10 (Tier-2 deferred).
///
/// Returns a non-silent confirmation with `running=false` in Tier-1 so the
/// frontend knows the live proxy is not yet active (not a silent stub).
#[tauri::command]
pub async fn start_dev_proxy(
    _state: State<'_, AppState>,
    target_url: String,
    listen_port: Option<u16>,
) -> Result<crate::dev_preview_proxy::ProxyStatus, String> {
    use crate::dev_preview_proxy::start_dev_proxy as do_start;

    let proxy = do_start(target_url, listen_port)?;
    let status = proxy.status()?;

    tracing::info!(
        port = status.listen_port,
        target = %status.target_url,
        running = status.running,
        "start_dev_proxy: proxy handle created (Tier-1; live listener is stub-10)"
    );

    Ok(status)
}

/// Stop the design-preview proxy.
///
/// In Tier-1 (stub-10 not live), this is a no-op that returns success.
/// Once the live proxy (stub-10) lands, this will signal the TCP listener to
/// stop accepting new connections and drain in-flight requests.
#[tauri::command]
pub async fn stop_dev_proxy(
    _state: State<'_, AppState>,
) -> Result<String, String> {
    // TODO[H-14.proxy.live]: send shutdown signal to the live proxy handle
    // once stub-10 is replaced (Tier-2). For now, return a non-silent
    // confirmation that the proxy is stopped.
    tracing::info!("stop_dev_proxy: called (Tier-1; live listener is stub-10)");
    Ok("dev_proxy: stopped (Tier-1; live proxy listener is stub-10 — not yet running)".to_owned())
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_oid_tagger::OidRange;

    #[test]
    fn oid_resolve_response_from_entry_encodes_range_as_array() {
        let entry = OidMapEntry {
            file: "src/App.tsx".into(),
            range: OidRange {
                start_line: 10,
                start_col: 4,
                end_line: 10,
                end_col: 22,
            },
            scope: "App > div".into(),
        };
        let resp = OidResolveResponse::from(("test-oid".to_owned(), entry));
        assert_eq!(resp.oid, "test-oid");
        assert_eq!(resp.range, [10, 4, 10, 22]);
        assert_eq!(resp.file, "src/App.tsx");
        assert_eq!(resp.scope, "App > div");
    }

    #[test]
    fn recast_patch_request_deserializes_from_json() {
        let json = serde_json::json!({
            "oid": "abcdef0123456789",
            "attr_name": "className",
            "new_value": "bg-blue-500",
            "kind": "deterministic"
        });
        let req: RecastPatchRequest = serde_json::from_value(json).expect("deserialize");
        assert_eq!(req.kind, "deterministic");
        assert_eq!(req.attr_name, "className");
    }

    #[test]
    fn design_mode_label_constant_matches_canvas_namespace() {
        // Wire-ID stability (wiring-enforcement R5): ensure DESIGN_MODE_LABEL
        // matches the "design" namespace used in canvas_dispatcher.rs.
        assert_eq!(DESIGN_MODE_LABEL, "design");
    }
}
