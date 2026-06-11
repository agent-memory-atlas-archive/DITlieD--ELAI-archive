//! G-9 / Slice A: `SecretRedacted` SSB read surface for the Svelte
//! `RedactionPanel`.
//!
//! One read-only command:
//!
//! - [`get_redaction_stats`] -- last N `secret_redacted` rows emitted by the
//!   write-time secret+PII redactor when it fires on a typed SsbPayload
//!   free-text field at the top of `Ssb::write_payload`. Each row records ONE
//!   redaction event: the redacted field identity, the stable blake3-keyed
//!   token(s) that replaced the match(es), the match count, and the per-kind
//!   `(label, count)` summary. The panel renders the counts so an operator can
//!   see redaction is firing on the write path WITHOUT ever seeing a secret
//!   (the row never carries the raw value).
//!
//! Default-ON producer: redaction is always on (there is no enable flag); a
//! row lands whenever a free-text field is redacted. The read command itself is
//! NOT feature-gated: it always queries the SSB viewer and returns whatever rows
//! exist (an empty array on a fresh install / when no secret has been written).
//!
//! Sibling of `commands_g8` (AttestationDrawer surface): one command per
//! distinct Svelte panel key (W-1 wiring doctrine).
//!
//! cross-lang-pair: dashboard/tauri/api

use serde_json::{json, Value};
use tauri::State;

use elai_orchestrator::ssb_viewer::RedactionStatRow;

use crate::state::AppState;

/// Local Tauri-command result alias (mirrors the sibling `commands_g*` modules;
/// `commands.rs::CmdResult` is module-private).
type CmdResult<T> = Result<T, String>;

/// G-9 — dashboard read cap for the redaction feed. The viewer clamps to its
/// own `DRAWER_QUERY_HARD_CAP` (500); this is the dashboard-side request size,
/// kept well under that ceiling so the IPC payload stays small at the panel's
/// poll cadence.
const REDACTION_FEED_CAP: usize = 200;

/// List the `secret_redacted` SSB rows newest-first. Each row records ONE
/// write-time secret+PII redaction: the redacted field identity, the stable
/// blake3-keyed token(s), the match count, and the per-kind `(label, count)`
/// summary. NEVER the raw secret.
///
/// Returns an empty array when no SSB DB is reachable (fresh install) or when no
/// redaction has fired. Serde serializes `RedactionStatRow`'s field names
/// verbatim, so the frontend `RedactionStat` DTO must mirror them.
#[tauri::command]
pub fn get_redaction_stats(state: State<'_, AppState>) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!([]));
    };
    let rows: Vec<RedactionStatRow> = viewer.redactions(REDACTION_FEED_CAP).unwrap_or_default();
    serde_json::to_value(rows).map_err(|e| e.to_string())
}
