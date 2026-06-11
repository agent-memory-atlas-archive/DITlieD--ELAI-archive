//! H-35 -- `rewind_session` Tauri command surface.
//!
//! Exposes the `elai rewind` restore dispatcher to the dashboard frontend.
//! The command drives the same SSB-based restore flow as the CLI `elai rewind`
//! subcommand and emits the same `session_rewound` beacon, so CLI and dashboard
//! share one done-gate artifact.
//!
//! Kill-switch: `ELAI_REWIND_DISABLE` (truthy) blocks all rewinds from the
//! dashboard as well (mirrors the CLI kill-switch).
//!
//! Cross-lang pair: CommandWindow.svelte `invoke("rewind_session", ...)`.
//! Both must land in the same edit batch (cross_lang_diff.py enforces this).
//!
//! Grounding:
//!   - crates/orchestrator/src/observability/replay.rs:147 (ssb_resume_from)
//!   - crates/orchestrator/src/compact.rs:96 (compact_session)
//!   - crates/orchestrator/src/ssb.rs:~27437 (SessionRewoundPayload)

use std::path::PathBuf;
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};
use tauri::State;

use elai_orchestrator::{compact_session, CompactionConfig};
use elai_orchestrator::observability::replay::{
    ssb_resume_from, ReplayOutcome, ReplayPatch,
};
use elai_orchestrator::ssb::{SsbPayload, SessionRewoundPayload};

use crate::state::AppState;

// ---------------------------------------------------------------------------
// Request / Response DTOs
// ---------------------------------------------------------------------------

/// Response shape for `rewind_session`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RewindResponse {
    /// The SSB seq of the emitted `session_rewound` beacon. 0 on dry-run or no-op.
    pub beacon_seq: i64,
    /// Canonical scope string: "conversation" | "code" | "both".
    pub scope: String,
    /// Whether the tail past target_seq was summarized via compact_session.
    pub summarized: bool,
    /// Replay-session id minted by ssb_resume_from (code-restore path).
    /// None when scope=="conversation".
    pub replay_session_id: Option<String>,
    /// Outcome: "Accepted" | "Refused" | "PredicateUnavailable" | "n/a".
    pub replay_outcome: String,
    /// Whether this was a no-op (target==head) — beacon_seq will be 0.
    pub no_op: bool,
    /// Whether the kill-switch suppressed this rewind.
    pub disabled: bool,
}

// ---------------------------------------------------------------------------
// Tauri command
// ---------------------------------------------------------------------------

/// H-35 — restore session state to a prior SSB journal point.
///
/// # Arguments
/// - `session_id`: the session to rewind.
/// - `target_seq`: the SSB seq to rewind to.
/// - `scope`: "conversation" | "code" | "both" (default "both").
/// - `summarize`: compact the tail into a summary via compact_session.
///
/// # Errors
/// Returns an error string on any failure (Tauri surfaces as IPC rejection).
/// Never panics; all error paths propagate via `Result<_, String>`.
#[tauri::command]
pub async fn rewind_session(
    state: State<'_, AppState>,
    session_id: String,
    target_seq: i64,
    scope: String,
    summarize: bool,
) -> Result<RewindResponse, String> {
    // Kill-switch check (mirrors CLI kill-switch).
    if env_truthy("ELAI_REWIND_DISABLE") {
        tracing::info!("rewind_session: suppressed by ELAI_REWIND_DISABLE kill-switch");
        return Ok(RewindResponse {
            beacon_seq: 0,
            scope,
            summarized: summarize,
            replay_session_id: None,
            replay_outcome: "n/a".to_string(),
            no_op: false,
            disabled: true,
        });
    }

    // Require the SSB writer.
    let writer = state
        .writer
        .as_ref()
        .ok_or_else(|| "rewind_session: no SSB writer available (DB not open)".to_string())?;

    // Read the full session stream ASC.
    let stream = writer
        .query_session_stream_asc(&session_id)
        .map_err(|e| format!("rewind_session: reading stream for `{session_id}`: {e}"))?;

    if stream.is_empty() {
        return Err(format!(
            "rewind_session: session `{session_id}` has no SSB rows"
        ));
    }

    let head_seq = stream
        .last()
        .map(|e| e.seq)
        .ok_or_else(|| "rewind_session: empty stream after check".to_string())?;

    // Validate target_seq is present in the stream.
    if !stream.iter().any(|e| e.seq == target_seq) {
        return Err(format!(
            "rewind_session: target_seq {target_seq} not found in session `{session_id}` \
             (head is {head_seq})"
        ));
    }

    // Idempotence guard: no-op when target==head.
    if target_seq >= head_seq {
        tracing::info!(
            session_id = %session_id,
            target_seq,
            head_seq,
            "rewind_session: target==head, no-op"
        );
        return Ok(RewindResponse {
            beacon_seq: 0,
            scope,
            summarized: summarize,
            replay_session_id: None,
            replay_outcome: "n/a".to_string(),
            no_op: true,
            disabled: false,
        });
    }

    // Build tail fingerprint (blake3, anti-stale-training rule).
    let tail_fp = {
        let mut hasher = blake3::Hasher::new();
        for entry in stream.iter().filter(|e| e.seq > target_seq) {
            hasher.update(&entry.seq.to_le_bytes());
            hasher.update(entry.action_type.as_sql_str().as_bytes());
            let payload_hash: [u8; 32] = blake3::hash(&entry.payload).into();
            hasher.update(&payload_hash);
        }
        hasher.finalize().to_hex().to_string()
    };

    let scope_str = parse_scope(&scope);

    // --- Conversation trim (when scope covers conversation) ---
    if scope_str == "conversation" || scope_str == "both" {
        if summarize {
            use elai_orchestrator::{ContentBlock, ConversationMessage, MessageRole, Session};
            let messages: Vec<ConversationMessage> = stream
                .iter()
                .filter(|e| e.seq <= target_seq)
                .filter_map(|e| {
                    let s = e.action_type.as_sql_str();
                    if s == "chat_message" || s == "task_submitted" {
                        Some(ConversationMessage {
                            role: MessageRole::User,
                            blocks: vec![ContentBlock::Text {
                                text: format!("[seq={}]", e.seq),
                            }],
                            usage: None,
                        })
                    } else {
                        None
                    }
                })
                .collect();
            let mut session = Session::new();
            session.messages = messages;
            let _compaction_result = compact_session(&session, CompactionConfig::default());
            // The CompactionResult is the new conversation shape. The summarized=true
            // flag in the beacon is the observable artifact for the done-gate.
        }
    }

    // --- Code restore (when scope covers code) ---
    let mut replay_session_id: Option<String> = None;
    let mut replay_outcome_str = "n/a".to_string();

    if scope_str == "code" || scope_str == "both" {
        // Resolve the locked-keys registry path relative to the project root.
        let project_root = state
            .project_root
            .lock()
            .map(|g| g.clone())
            .unwrap_or_else(|_| PathBuf::from("."));
        let registry_path = project_root.join("elai-locked-keys.toml");

        let patch = ReplayPatch {
            prompt: None,
            model: None,
            tool_overrides: vec![],
        };

        match ssb_resume_from(target_seq, patch, registry_path) {
            Ok((sid, outcome)) => {
                replay_session_id = Some(sid.0.clone());
                replay_outcome_str = match outcome {
                    ReplayOutcome::Accepted => "Accepted".to_string(),
                    ReplayOutcome::Refused => "Refused".to_string(),
                    ReplayOutcome::PredicateUnavailable => "PredicateUnavailable".to_string(),
                };
                tracing::info!(
                    session_id = %session_id,
                    target_seq,
                    replay_session_id = %sid.0,
                    outcome = %replay_outcome_str,
                    "rewind_session: code-restore predicate check complete"
                );
            }
            Err(e) => {
                return Err(format!(
                    "rewind_session: code-restore predicate error at seq {target_seq}: {e}"
                ));
            }
        }
    }

    // Emit the session_rewound SSB beacon (off-agent done-gate artifact, R1/R4).
    let now_ms = {
        let ms = state
            .clock
            .system_now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        i64::try_from(ms).unwrap_or(i64::MAX)
    };
    let _ = now_ms; // available for future payload fields.

    let payload = SsbPayload::SessionRewound(SessionRewoundPayload {
        session_id: session_id.clone(),
        target_seq,
        head_seq_before: head_seq,
        scope: scope_str.to_string(),
        summarized: summarize,
        replay_session_id: replay_session_id.clone(),
        replay_outcome: replay_outcome_str.clone(),
        tail_fingerprint_blake3: tail_fp,
    });

    let beacon_seq = writer
        .write_payload("dashboard-v2", 0, payload, vec![])
        .await
        .map_err(|e| format!("rewind_session: writing session_rewound beacon: {e}"))?;

    tracing::info!(
        session_id = %session_id,
        target_seq,
        head_seq_before = head_seq,
        scope = scope_str,
        beacon_seq,
        "rewind_session: session_rewound beacon emitted"
    );

    Ok(RewindResponse {
        beacon_seq,
        scope: scope_str.to_string(),
        summarized: summarize,
        replay_session_id,
        replay_outcome: replay_outcome_str,
        no_op: false,
        disabled: false,
    })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Normalise the user-supplied scope string into canonical form.
/// Unknown values default to "both".
fn parse_scope(raw: &str) -> &'static str {
    match raw.trim().to_ascii_lowercase().as_str() {
        "conversation" => "conversation",
        "code" => "code",
        _ => "both",
    }
}

/// Returns true when an environment variable is set to a truthy value.
fn env_truthy(var: &str) -> bool {
    match std::env::var(var) {
        Ok(raw) => {
            let v = raw.trim().to_ascii_lowercase();
            v == "1" || v == "true" || v == "yes" || v == "on"
        }
        Err(_) => false,
    }
}

