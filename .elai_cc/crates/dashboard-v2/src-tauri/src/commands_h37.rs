//! H-37 -- `get_plan_approval_decisions` Tauri command surface.
//!
//! Polls the SSB journal for `plan_decision_event` rows for a session and
//! returns them to the frontend so PlanApprovalCard.svelte can render the
//! recorded plan-approval decision (Approve/Redirect/Reject/AutoAdvance).
//!
//! Cross-lang pair: PlanApprovalCard.svelte poll + stub.ts + this mod + generate_handler!.
//! All four must land in the same edit batch (cross_lang_diff.py enforces this).
//!
//! Kill-switch: reads the SSB journal only. No mutation. The gate that EMITS
//! plan_decision_event rows is gated by ELAI_PLAN_APPROVAL_GATE_ENABLE on the
//! orchestrator side (task_runner.rs::plan_approval_gate_enabled).
//!
//! Grounding:
//!   - crates/orchestrator/src/ssb.rs PlanDecisionPayload / PlanDecisionKind
//!   - crates/orchestrator/src/task_runner.rs (H-37 producer intercept)
//!   - crates/dashboard-v2/src-tauri/src/commands_h36.rs (pattern reference)

use serde::{Deserialize, Serialize};
use tauri::State;

use elai_orchestrator::ssb::{PlanDecisionKind, SsbActionType, SsbPayload};
use elai_orchestrator::SsbReader;

use crate::state::AppState;

// ---------------------------------------------------------------------------
// DTO
// ---------------------------------------------------------------------------

/// One row returned by `get_plan_approval_decisions`.
/// Mirrors [`elai_orchestrator::ssb::PlanDecisionPayload`] but serde-stable
/// (no orchestrator-internal types leak into the Tauri IPC layer).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanDecisionRow {
    /// SSB sequence number for the row (monotonically increasing, unique).
    pub id: i64,
    /// Decision kind: "approve" | "redirect" | "reject" | "auto_advance".
    pub decision_kind: String,
    /// FSM target state attempted (e.g. "TESTING_FIRST"; "PLANNING" on Redirect).
    pub target_state: String,
    /// Risk classification used by the gate: "LOW" | "MEDIUM" | "HIGH".
    pub risk_level: String,
    /// Human-provided reason for Reject/Redirect; null for Approve/AutoAdvance.
    pub reason: Option<String>,
    /// FSM phase label at the time of the gate invocation.
    pub plan_session_id: String,
}

// ---------------------------------------------------------------------------
// Tauri command
// ---------------------------------------------------------------------------

/// H-37 -- return `plan_decision_event` SSB rows for a session.
///
/// # Arguments
/// - `session_id`: SSB session to filter by. Pass `""` to return ALL sessions
///   (useful for the dashboard overview card).
/// - `limit`: maximum rows to return (default 20; capped at 100).
///
/// Returns an empty `Vec` when the SSB viewer is unavailable or when no rows
/// exist yet (never returns an error for the "no data" case per the dashboard
/// convention -- the frontend renders an empty state).
///
/// # Errors
/// Returns an error string only on an unexpected DB failure. Never panics.
#[tauri::command]
pub async fn get_plan_approval_decisions(
    state: State<'_, AppState>,
    session_id: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<PlanDecisionRow>, String> {
    let effective_limit = limit.unwrap_or(20).min(100);

    let writer = match state.writer.as_ref() {
        Some(w) => w,
        None => return Ok(Vec::new()),
    };

    let rows = writer
        .query_by_type(SsbActionType::PlanDecisionEvent)
        .map_err(|e| format!("get_plan_approval_decisions: query failed: {e}"))?;

    let filter_session = session_id.as_deref().unwrap_or("").to_owned();

    // Take the most recent N rows (rows are ordered by seq ascending).
    let start = rows.len().saturating_sub(effective_limit * 2); // oversize to account for filter
    let result: Vec<PlanDecisionRow> = rows[start..]
        .iter()
        .filter_map(|entry| {
            let decoded =
                elai_orchestrator::ssb::decode_ssb_payload(&entry.payload).ok()?;
            let payload = match decoded {
                SsbPayload::PlanDecisionEvent(p) => p,
                _ => return None,
            };
            // Filter by session when requested.
            if !filter_session.is_empty()
                && payload.plan_session_id != filter_session
                && entry.session_id != filter_session
            {
                return None;
            }
            let kind_str = match payload.decision_kind {
                PlanDecisionKind::Approve => "approve",
                PlanDecisionKind::Redirect => "redirect",
                PlanDecisionKind::Reject => "reject",
                PlanDecisionKind::AutoAdvance => "auto_advance",
            };
            Some(PlanDecisionRow {
                id: entry.seq,
                decision_kind: kind_str.to_owned(),
                target_state: payload.target_state,
                risk_level: payload.risk_level,
                reason: payload.reason,
                plan_session_id: payload.plan_session_id,
            })
        })
        .take(effective_limit)
        .collect();

    Ok(result)
}
