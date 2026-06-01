//! E-29 Tier-B -- agentic-observability Tauri command surface.
//!
//! Four commands feed the new FailureModesDrawer.svelte + the
//! pre-existing TranscriptDrawer.svelte (Tier-A shipped earlier this
//! cluster):
//!
//! - [`ssb_transcript`] -- joined prompt / tool_use / tool_result rows
//!   in temporal order for a session, role-bubble friendly. Content-
//!   gated by `ELAI_OTEL_TRACE_CONTENT` (default OFF; matches
//!   OpenLLMetry `TRACELOOP_TRACE_CONTENT`).
//! - [`ssb_resume_from`] -- replay validator. FAIL-CLOSED on the
//!   C-44/D-9 frozen-surface predicate (`LockedKeysAuditor`). Emits an
//!   `agentic_observability_event` row with `kind=replay_resume` per
//!   call.
//! - [`ssb_promote_to_dataset`] -- promote a set of trace ids to a
//!   named eval dataset (Langfuse / LangSmith pattern). Emits one
//!   `agentic_observability_event` row per member with
//!   `kind=eval_dataset_member`.
//! - [`ssb_backfill_signals`] -- run the candidate-row sweep helper
//!   for a free-text signal label. The classifier itself is OUT OF
//!   SCOPE for this Tier-B slice (gated behind the per-role runtime);
//!   this command returns the candidate set + a pre-built
//!   `agentic_observability_event` row template the caller emits
//!   after the LLM call returns.
//!
//! Additive -- sibling of `commands_d9` (locked-keys panel). The
//! orchestrator producers (`observability::mast_classify::classify_closed_run`
//! + `observability::replay::ssb_resume_from`) own the underlying SSB
//! rows; this surface only DISPATCHES validation + assembles dashboard
//! responses.
//!
//! Grounding: arXiv:2503.13657 (MAST), laminar.sh (Signals), langfuse
//! (datasets), dev.to/sreeni5018 (LangGraph replay), traceloop
//! (TRACELOOP_TRACE_CONTENT). See workflow/reasoning/E-29-decisions.md.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::State;

use elai_orchestrator::observability::{
    backfill_signal_matches as backfill_helper, otel_trace_content_enabled,
    promote_traces_to_dataset as promote_helper, ssb_resume_from as replay_validator,
    ReplayOutcome, ReplayPatch as OrchReplayPatch,
};

use crate::state::AppState;

const TRANSCRIPT_DEFAULT_LIMIT: usize = 200;
const TRANSCRIPT_MAX_LIMIT: usize = 2000;
const SIGNAL_DEFAULT_LIMIT: usize = 500;
const SIGNAL_MAX_LIMIT: usize = 5000;

/// One row of a transcript-shaped chat surface. The dashboard renders
/// these as message bubbles (user / assistant / tool / status). The
/// `body` field is conditionally redacted when
/// `ELAI_OTEL_TRACE_CONTENT` is unset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TranscriptRow {
    pub seq: i64,
    pub ts_ms: i64,
    pub action_type: String,
    pub agent_id: String,
    pub role: String,
    /// Free-text body. Empty when content-gating is enabled and
    /// `redacted` is true.
    pub body: String,
    /// True when the row's body was suppressed due to the content
    /// gate. Dashboard renders a placeholder.
    pub redacted: bool,
}

/// Filter options for [`ssb_transcript`]. All fields are optional.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranscriptFilter {
    pub action_type_whitelist: Vec<String>,
    pub agent_role: Option<String>,
    pub limit: Option<usize>,
}

/// Patch struct as it crosses the IPC boundary -- matches
/// [`elai_orchestrator::observability::ReplayPatch`] shape but stays
/// owned here so Tauri JSON-Schema generation doesn't leak the
/// orchestrator's types.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayPatchDto {
    pub prompt: Option<String>,
    pub model: Option<String>,
    pub tool_overrides: Vec<ToolOverrideDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolOverrideDto {
    pub tool_name: String,
    pub target_path: String,
    pub label: String,
}

/// Response from [`ssb_resume_from`]. Wraps the validator outcome +
/// the deterministic replay session id so dashboard consumers render
/// Refused / PredicateUnavailable / Accepted without re-validating.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResumeResponse {
    pub replay_session_id: String,
    pub outcome: String,
    pub refusal_reason: String,
}

/// Response from [`ssb_promote_to_dataset`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromoteResponse {
    pub dataset_name: String,
    pub promoted_count: usize,
    pub skipped_duplicates: usize,
}

/// Response from [`ssb_backfill_signals`]. Carries the candidate
/// sweep result so the caller can dispatch the classifier on top.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackfillResponse {
    pub signal_label: String,
    pub candidate_count: usize,
    /// Subset of candidate seq ids returned for inspection in the
    /// drawer (first 100). The full candidate set lives in the SSB.
    pub sample_seq_ids: Vec<i64>,
}

/// Return a session's transcript as an ordered chat-shaped row list.
/// Empty when no SSB viewer is attached.
///
/// `filter.action_type_whitelist` -- when non-empty, only those
/// action_types are included. Default whitelist (when empty) covers
/// `phase_transition`, `tool_result`, `finding`, `claim`.
///
/// Content gating: when `ELAI_OTEL_TRACE_CONTENT` is unset, the row
/// body fields are blanked and `redacted=true`. The dashboard surface
/// renders a placeholder for redacted rows.
#[tauri::command]
pub async fn ssb_transcript(
    state: State<'_, AppState>,
    session_id: String,
    filter: Option<TranscriptFilter>,
) -> Result<Vec<TranscriptRow>, String> {
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(Vec::new()),
    };
    let filter = filter.unwrap_or_default();
    let limit = filter
        .limit
        .unwrap_or(TRANSCRIPT_DEFAULT_LIMIT)
        .min(TRANSCRIPT_MAX_LIMIT);

    let whitelist: Vec<&str> = if filter.action_type_whitelist.is_empty() {
        vec!["phase_transition", "tool_result", "finding", "claim"]
    } else {
        filter
            .action_type_whitelist
            .iter()
            .map(|s| s.as_str())
            .collect()
    };

    let content_enabled = otel_trace_content_enabled();
    let mut out: Vec<TranscriptRow> = Vec::new();
    for at in &whitelist {
        let rows = viewer
            .recent_rows_of_type(at, limit)
            .map_err(|e| format!("ssb_viewer error: {e}"))?;
        for row in rows {
            // Best-effort session filter -- payload may not carry
            // session_id; permissive when absent (matches the offline
            // classifier's `row_belongs_to_session` semantics).
            let row_session = row
                .payload_json
                .get("session_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            if let Some(rs) = &row_session {
                if !rs.is_empty() && rs != &session_id {
                    continue;
                }
            }
            let role = match row.action_type.as_str() {
                "phase_transition" => "system",
                "tool_result" => "tool",
                "finding" => "assistant",
                "claim" => "assistant",
                _ => "system",
            }
            .to_string();
            let body = if content_enabled {
                row.payload_json
                    .get("message")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| row.payload_json.to_string())
            } else {
                String::new()
            };
            out.push(TranscriptRow {
                seq: row.seq,
                ts_ms: row.ts,
                action_type: row.action_type,
                agent_id: row.agent_id,
                role,
                body,
                redacted: !content_enabled,
            });
        }
    }
    // Temporal order -- ascending by seq so the chat renders oldest
    // first (matches LangChain / Laminar transcript convention).
    out.sort_by_key(|r| r.seq);
    Ok(out)
}

/// Validate a replay patch against the C-44/D-9 frozen-surface
/// predicate. FAIL-CLOSED: missing/unreadable registry => Refused.
#[tauri::command]
pub async fn ssb_resume_from(
    _state: State<'_, AppState>,
    checkpoint_id: i64,
    patch: ReplayPatchDto,
    registry_path: Option<String>,
) -> Result<ResumeResponse, String> {
    let registry = registry_path
        .map(PathBuf::from)
        .unwrap_or_else(default_registry_path);

    let orch_patch = OrchReplayPatch {
        prompt: patch.prompt,
        model: patch.model,
        tool_overrides: patch
            .tool_overrides
            .into_iter()
            .map(|t| elai_orchestrator::observability::replay::ToolOverride {
                tool_name: t.tool_name,
                target_path: t.target_path,
                label: t.label,
            })
            .collect(),
    };

    match replay_validator(checkpoint_id, orch_patch, registry) {
        Ok((sid, outcome)) => {
            let (outcome_str, reason) = match outcome {
                ReplayOutcome::Accepted => ("accepted".to_string(), String::new()),
                ReplayOutcome::Refused => (
                    "refused".to_string(),
                    "frozen-surface predicate fired".to_string(),
                ),
                ReplayOutcome::PredicateUnavailable => (
                    "predicate_unavailable".to_string(),
                    "locked-keys registry unreadable (fail-closed)".to_string(),
                ),
            };
            Ok(ResumeResponse {
                replay_session_id: sid.0,
                outcome: outcome_str,
                refusal_reason: reason,
            })
        }
        Err(e) => Err(format!("ssb_resume_from validator error: {e}")),
    }
}

/// Promote a set of trace ids to a named eval dataset. Returns the
/// promoted member count + skipped duplicates.
///
/// Empty `prior_member_ids` means "no de-dup against existing
/// dataset"; callers can pre-fetch the prior set from the SSB and
/// pass it in for idempotent batched promotion.
#[tauri::command]
pub async fn ssb_promote_to_dataset(
    _state: State<'_, AppState>,
    trace_ids: Vec<i64>,
    dataset_name: String,
    label: Option<String>,
    scoring_rubric: Option<String>,
    prior_member_ids: Option<Vec<i64>>,
) -> Result<PromoteResponse, String> {
    let label = label.unwrap_or_default();
    let rubric = scoring_rubric.unwrap_or_else(|| "binary_pass".to_string());
    let priors = prior_member_ids.unwrap_or_default();
    match promote_helper(&trace_ids, &dataset_name, &label, &rubric, &priors) {
        Ok(promo) => Ok(PromoteResponse {
            dataset_name: promo.dataset_name,
            promoted_count: promo.members.len(),
            skipped_duplicates: promo.skipped_duplicates,
        }),
        Err(e) => Err(format!("ssb_promote_to_dataset error: {e}")),
    }
}

/// Run the candidate-row sweep for a signal-match backfill. Returns
/// the candidate count + a preview slice of seq ids. The downstream
/// classifier itself (per-role runtime) is OUT OF SCOPE for this
/// Tier-B slice.
#[tauri::command]
pub async fn ssb_backfill_signals(
    state: State<'_, AppState>,
    signal_label: String,
    action_type_whitelist: Option<Vec<String>>,
    limit: Option<usize>,
) -> Result<BackfillResponse, String> {
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => {
            return Ok(BackfillResponse {
                signal_label,
                candidate_count: 0,
                sample_seq_ids: Vec::new(),
            });
        }
    };
    let whitelist = action_type_whitelist.unwrap_or_else(|| {
        vec![
            "finding".to_string(),
            "phase_transition".to_string(),
            "tool_result".to_string(),
        ]
    });
    let cap = limit.unwrap_or(SIGNAL_DEFAULT_LIMIT).min(SIGNAL_MAX_LIMIT);
    let refs: Vec<&str> = whitelist.iter().map(|s| s.as_str()).collect();
    let rows = backfill_helper(&viewer, &refs)
        .map_err(|e| format!("ssb_backfill_signals sweep error: {e}"))?;
    let limited: Vec<&_> = rows.iter().take(cap).collect();
    let sample: Vec<i64> = limited.iter().take(100).map(|r| r.seq).collect();
    Ok(BackfillResponse {
        signal_label,
        candidate_count: limited.len(),
        sample_seq_ids: sample,
    })
}

/// Default locked-keys registry path (matches `commands_d9`
/// convention). Walks ancestors of the binary location for
/// `scripts/locked_keys_registry.toml`. Binary-relative (not cwd-relative)
/// so the lookup still works when elai.bat reset cwd to %USERPROFILE%.
fn default_registry_path() -> PathBuf {
    let start = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    for ancestor in start.ancestors() {
        let candidate = ancestor.join("scripts").join("locked_keys_registry.toml");
        if candidate.exists() {
            return candidate;
        }
    }
    start.join("scripts").join("locked_keys_registry.toml")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replay_patch_dto_serializes() {
        let dto = ReplayPatchDto {
            prompt: Some("p".into()),
            model: Some("m".into()),
            tool_overrides: vec![ToolOverrideDto {
                tool_name: "bash".into(),
                target_path: "src/main.rs".into(),
                label: "override".into(),
            }],
        };
        let json = serde_json::to_string(&dto).expect("serialize");
        let back: ReplayPatchDto = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(dto, back);
    }
}
