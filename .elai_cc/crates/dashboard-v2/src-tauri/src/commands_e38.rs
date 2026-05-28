//! E-38 (SCHEMA_VERSION 175) -- 2026 AI roadmap signals dashboard surface.
//!
//! One read-only Tauri command feeds the Svelte RoadmapPanel:
//!
//! - get_roadmap_signals_state -- most recent ai_roadmap_signal_event SSB
//!   rows projected into per-kind tile state for the dashboard panel.
//!
//! Additive -- sibling of commands_e10 (CriticDecisionPanel) and
//! commands_e29 (transcript / replay / dataset / signals drawers). The
//! orchestrator producer modules (compliance + roadmap_signals +
//! dispatch::mcp) are the sole writers of the underlying SSB rows; this
//! module only reads. Wire-dark unless ELAI_AI_ROADMAP_SIGNALS_ENABLE=1
//! on the orchestrator side.
//!
//! Grounding: artificialintelligenceact.eu/implementation-timeline (EU
//! AI Act Annex III postponed to Dec 2 2027); arXiv:2409.12640 (MRCR
//! v2); modelcontextprotocol.io/development/roadmap (MCP transport
//! roadmap). See workflow/reasoning/E-38-decisions.md.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 200;
const MAX_LIMIT: usize = 1000;

/// Most-recent compliance event (Blocked or Verified).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplianceTile {
    /// `compliance_blocked` or `compliance_verified`.
    pub kind: String,
    /// Empty when kind = compliance_verified.
    pub assertion_label: String,
    /// Empty when kind = compliance_verified.
    pub hint: String,
    /// Bitmask of assertions checked (only meaningful on
    /// compliance_verified rows).
    pub assertions_checked: u32,
    /// Plan session id.
    pub plan_session_id: String,
    /// Wall-clock ts of the row.
    pub ts_ms: i64,
}

/// Most-recent MCP transport version beacon.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpTransportTile {
    /// e.g. "streamable_http".
    pub mcp_transport_current: String,
    /// Roadmap target string.
    pub mcp_transport_target: String,
    /// Wall-clock ts of the row.
    pub ts_ms: i64,
}

/// Most-recent posture-doc-stale warning.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostureStaleTile {
    pub posture_doc_path: String,
    pub posture_doc_mtime_ms: i64,
    pub age_days: u32,
    pub ts_ms: i64,
}

/// Aggregated panel response. Empty fields mean no row of the
/// corresponding kind has been observed yet.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoadmapSignalsState {
    /// Most recent Blocked OR Verified row.
    pub last_compliance: Option<ComplianceTile>,
    /// Most recent McpTransportVersionLogged row.
    pub mcp_transport: Option<McpTransportTile>,
    /// Most recent PostureDocStale row.
    pub posture_stale: Option<PostureStaleTile>,
    /// Count of distinct MrcrUnverified backends observed in the
    /// window.
    pub mrcr_unverified_count: u32,
    /// Total ai_roadmap_signal_event rows scanned.
    pub grand_total: u32,
}

/// Return the most recent ai_roadmap_signal_event SSB rows projected
/// into per-kind tile state. Empty response when no SSB viewer is
/// attached.
#[tauri::command]
pub async fn get_roadmap_signals_state(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<RoadmapSignalsState, String> {
    let cap = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => {
            return Ok(RoadmapSignalsState {
                last_compliance: None,
                mcp_transport: None,
                posture_stale: None,
                mrcr_unverified_count: 0,
                grand_total: 0,
            });
        }
    };
    let raw_rows = match viewer.recent_rows_of_type("ai_roadmap_signal_event", cap) {
        Ok(d) => d,
        Err(e) => {
            tracing::warn!(
                action_type = "ai_roadmap_signal_event",
                error = %e,
                "get_roadmap_signals_state: viewer error"
            );
            return Ok(RoadmapSignalsState {
                last_compliance: None,
                mcp_transport: None,
                posture_stale: None,
                mrcr_unverified_count: 0,
                grand_total: 0,
            });
        }
    };

    let mut last_compliance: Option<ComplianceTile> = None;
    let mut mcp_transport: Option<McpTransportTile> = None;
    let mut posture_stale: Option<PostureStaleTile> = None;
    let mut mrcr_backends: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut grand_total: u32 = 0;

    for d in &raw_rows {
        let outer = match d.payload_json.as_object() {
            Some(o) => o,
            None => continue,
        };
        let obj = if let Some(inner) = outer
            .get("AiRoadmapSignalEvent")
            .and_then(|v| v.as_object())
        {
            inner.clone()
        } else {
            outer.clone()
        };
        let kind = obj.get("kind").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let ts_ms = obj.get("ts_ms").and_then(|v| v.as_i64()).unwrap_or(0);
        grand_total = grand_total.saturating_add(1);

        match kind.as_str() {
            "compliance_blocked" | "compliance_verified" => {
                let candidate = ComplianceTile {
                    kind: kind.clone(),
                    assertion_label: obj
                        .get("assertion_label")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    hint: obj
                        .get("hint")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    assertions_checked: obj
                        .get("assertions_checked")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as u32,
                    plan_session_id: obj
                        .get("plan_session_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    ts_ms,
                };
                match &last_compliance {
                    Some(existing) if existing.ts_ms >= ts_ms => {}
                    _ => last_compliance = Some(candidate),
                }
            }
            "mcp_transport_version_logged" => {
                let candidate = McpTransportTile {
                    mcp_transport_current: obj
                        .get("mcp_transport_current")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    mcp_transport_target: obj
                        .get("mcp_transport_target")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    ts_ms,
                };
                match &mcp_transport {
                    Some(existing) if existing.ts_ms >= ts_ms => {}
                    _ => mcp_transport = Some(candidate),
                }
            }
            "mrcr_unverified" => {
                let backend = obj
                    .get("backend_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if !backend.is_empty() {
                    mrcr_backends.insert(backend);
                }
            }
            "posture_doc_stale" => {
                let candidate = PostureStaleTile {
                    posture_doc_path: obj
                        .get("posture_doc_path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    posture_doc_mtime_ms: obj
                        .get("posture_doc_mtime_ms")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0),
                    age_days: obj
                        .get("age_days")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as u32,
                    ts_ms,
                };
                match &posture_stale {
                    Some(existing) if existing.ts_ms >= ts_ms => {}
                    _ => posture_stale = Some(candidate),
                }
            }
            _ => {
                // Unknown kind -- ignore. Forward-compat: future kinds
                // can extend the discriminator without breaking this
                // panel.
            }
        }
    }

    let mrcr_unverified_count = u32::try_from(mrcr_backends.len()).unwrap_or(u32::MAX);
    Ok(RoadmapSignalsState {
        last_compliance,
        mcp_transport,
        posture_stale,
        mrcr_unverified_count,
        grand_total,
    })
}
