//! H-36 -- `get_recent_notifications` Tauri command surface.
//!
//! Polls the SSB journal for recent `agent_notification_emitted` rows and
//! returns them to the frontend so NotificationManager.svelte can fire a toast
//! when a new `needs_input` row appears.
//!
//! Cross-lang pair: NotificationManager.svelte poll + this mod + generate_handler!.
//! All three must land in the same edit batch (cross_lang_diff.py enforces this).
//!
//! Kill-switch: not applicable -- this is a READ-ONLY query path. The OS
//! notification spawn is gated by ELAI_NOTIFY_ENABLE on the orchestrator side.
//!
//! Grounding:
//!   - crates/orchestrator/src/ssb.rs AgentNotificationEmittedPayload
//!   - crates/orchestrator/src/notify_dispatch.rs (H-36 producer)
//!   - crates/dashboard-v2/src-tauri/src/commands_h35.rs (pattern reference)

use serde::{Deserialize, Serialize};
use tauri::State;

use elai_orchestrator::ssb::{AgentNotificationKind, SsbActionType, SsbPayload};
use elai_orchestrator::SsbReader;

use crate::state::AppState;

// ---------------------------------------------------------------------------
// DTO
// ---------------------------------------------------------------------------

/// One row returned by `get_recent_notifications`.
/// Mirrors [`elai_orchestrator::ssb::AgentNotificationEmittedPayload`] but
/// serde-stable (no orchestrator-internal types leak into the Tauri IPC layer).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentNotificationRow {
    /// SSB sequence number for the row (monotonically increasing, unique).
    pub id: i64,
    /// Trigger kind: "completion" | "needs_input".
    pub kind: String,
    /// Bitset: bit 0 = terminal escape attempted, bit 1 = OS notification attempted.
    pub surfaces_attempted: u8,
    /// Bitset: surfaces that actually succeeded.
    pub surfaces_succeeded: u8,
    /// Session ID at dispatch time.
    pub session_id: String,
    /// Unix milliseconds at dispatch time.
    pub ts_ms: u64,
}

// ---------------------------------------------------------------------------
// Tauri command
// ---------------------------------------------------------------------------

/// H-36 -- return the most recent `agent_notification_emitted` SSB rows.
///
/// # Arguments
/// - `limit`: maximum rows to return (default 20; capped at 100 to avoid
///   blowing the IPC payload).
///
/// Returns an empty `Vec` when the SSB viewer is unavailable or when no rows
/// exist yet (never returns an error for the "no data" case per the dashboard
/// convention -- the frontend renders an empty state).
///
/// # Errors
/// Returns an error string only on an unexpected DB failure. Never panics.
#[tauri::command]
pub async fn get_recent_notifications(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<AgentNotificationRow>, String> {
    let effective_limit = limit.unwrap_or(20).min(100);

    // Use the SSB writer for query_by_type (implements SsbReader).
    // Return empty when the writer is not yet available (fresh install / no DB).
    let writer = match state.writer.as_ref() {
        Some(w) => w,
        None => return Ok(Vec::new()),
    };

    let rows = writer
        .query_by_type(SsbActionType::AgentNotificationEmitted)
        .map_err(|e| format!("get_recent_notifications: query failed: {e}"))?;

    // Take the most recent N rows (rows are ordered by seq ascending from query_by_type).
    let start = rows.len().saturating_sub(effective_limit);
    let result: Vec<AgentNotificationRow> = rows[start..]
        .iter()
        .filter_map(|entry| {
            // Decode the msgpack payload via the public decode_ssb_payload helper
            // (avoids a direct rmp_serde dep on the dashboard crate).
            let decoded = elai_orchestrator::ssb::decode_ssb_payload(&entry.payload).ok()?;
            let payload = match decoded {
                SsbPayload::AgentNotificationEmitted(p) => p,
                _ => return None,
            };
            let kind_str = match payload.kind {
                AgentNotificationKind::Completion => "completion",
                AgentNotificationKind::NeedsInput => "needs_input",
            };
            Some(AgentNotificationRow {
                id: entry.seq,
                kind: kind_str.to_owned(),
                surfaces_attempted: payload.surfaces_attempted,
                surfaces_succeeded: payload.surfaces_succeeded,
                session_id: payload.session_id,
                ts_ms: payload.ts_ms,
            })
        })
        .collect();

    Ok(result)
}
