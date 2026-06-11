//! H-51 -- sleep-time handoff digest Tauri command surface.
//!
//! Two commands feed the dashboard resume-offer banner in `CommandWindow.svelte`:
//!
//! - [`resume_offer`]: calls [`elai_orchestrator::handoff_digest::detect_open_topic`]
//!   and returns a `TopicHandleDto` if an open topic is found, otherwise `None`.
//!   Emits a `ResumeOffered` HandoffLifecycleEvent SSB row on success.
//!
//! - [`tiered_load_confirm`]: confirms the user's intent to resume, emits
//!   `ResumeConfirmed` + `TieredLoad` HandoffLifecycleEvent SSB rows, and
//!   returns the digest content for the frontend to inject as context.
//!
//! Both commands are additive — they do not replace any existing command surface.
//! The `ELAI_HANDOFF_AUTORESUME_DISABLE=1` kill-switch suppresses [`resume_offer`]
//! (returns `None`); it does NOT block [`tiered_load_confirm`] (explicit user intent).
//!
//! Grounding: H-51 plan file
//! `docs/thefinalglance/H-plan/H-51_sleeptime_handoff_digest_autodetect.md`.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

// ---------------------------------------------------------------------------
// DTO types
// ---------------------------------------------------------------------------

/// Serialisable form of [`elai_orchestrator::TopicHandle`] returned to the
/// frontend.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TopicHandleDto {
    pub topic_slug: String,
    pub digest_path: String,
    pub loaded_tokens: u32,
}

/// Payload returned by [`tiered_load_confirm`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TieredLoadResult {
    pub topic_slug: String,
    pub digest_content: String,
    pub loaded_tokens: u32,
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// Check for an open handoff topic in the active project root.
///
/// Returns `Some(TopicHandleDto)` when a `.md` file in
/// `{project_root}/workflow/handoffs/` was modified within the last 8 hours.
/// Returns `null` (serialised `None`) when no open topic is found or when
/// `ELAI_HANDOFF_AUTORESUME_DISABLE=1`.
///
/// Emits a `ResumeOffered` HandoffLifecycleEvent SSB row on success.
#[tauri::command]
pub async fn resume_offer(state: State<'_, AppState>) -> Result<Option<TopicHandleDto>, String> {
    let project_root = {
        let guard = state.project_root.lock().map_err(|e| e.to_string())?;
        guard.clone()
    };

    let handle = elai_orchestrator::handoff_digest::detect_open_topic(&project_root);
    let Some(h) = handle else {
        return Ok(None);
    };

    let session_id = state
        .writer
        .as_ref()
        .map(|_| "dashboard".to_owned())
        .unwrap_or_else(|| "dashboard".to_owned());

    // Emit ResumeOffered SSB row (best-effort).
    if let Some(writer) = &state.writer {
        let payload = elai_orchestrator::ssb::SsbPayload::HandoffLifecycleEvent(
            elai_orchestrator::ssb::HandoffLifecyclePayload {
                kind: elai_orchestrator::ssb::HandoffLifecycleEventKind::ResumeOffered,
                topic_slug: h.topic_slug.clone(),
                digest_path: Some(h.digest_path.to_string_lossy().into_owned()),
                loaded_tokens: None,
                session_id: session_id.clone(),
            },
        );
        let _ = writer
            .write_payload("dashboard-h51", 0, payload, vec![])
            .await;
    }

    Ok(Some(TopicHandleDto {
        topic_slug: h.topic_slug,
        digest_path: h.digest_path.to_string_lossy().into_owned(),
        loaded_tokens: h.loaded_tokens,
    }))
}

/// Confirm a tiered load resume for the given topic slug.
///
/// Reads the digest file, emits `ResumeConfirmed` + `TieredLoad` SSB rows,
/// and returns the digest content.
///
/// Does NOT check `ELAI_HANDOFF_AUTORESUME_DISABLE` — this is an explicit
/// user-driven action.
#[tauri::command]
pub async fn tiered_load_confirm(
    slug: String,
    state: State<'_, AppState>,
) -> Result<TieredLoadResult, String> {
    let project_root = {
        let guard = state.project_root.lock().map_err(|e| e.to_string())?;
        guard.clone()
    };

    let digest_path = project_root
        .join("workflow")
        .join("handoffs")
        .join(format!("{slug}.md"));

    let digest_content = std::fs::read_to_string(&digest_path)
        .map_err(|e| format!("read digest file '{}': {e}", digest_path.display()))?;

    let loaded_tokens = (digest_content.len() / 4) as u32;

    // Validate TieredLoad done-gate: loaded_tokens must be <= 5000.
    if loaded_tokens > 5000 {
        return Err(format!(
            "H-51 TieredLoad gate: loaded_tokens {loaded_tokens} > 5000 (digest too large)"
        ));
    }

    let session_id = "dashboard".to_owned();

    // Emit ResumeConfirmed + TieredLoad SSB rows.
    if let Some(writer) = &state.writer {
        let confirm_payload = elai_orchestrator::ssb::SsbPayload::HandoffLifecycleEvent(
            elai_orchestrator::ssb::HandoffLifecyclePayload {
                kind: elai_orchestrator::ssb::HandoffLifecycleEventKind::ResumeConfirmed,
                topic_slug: slug.clone(),
                digest_path: Some(digest_path.to_string_lossy().into_owned()),
                loaded_tokens: Some(loaded_tokens),
                session_id: session_id.clone(),
            },
        );
        let tiered_payload = elai_orchestrator::ssb::SsbPayload::HandoffLifecycleEvent(
            elai_orchestrator::ssb::HandoffLifecyclePayload {
                kind: elai_orchestrator::ssb::HandoffLifecycleEventKind::TieredLoad,
                topic_slug: slug.clone(),
                digest_path: Some(digest_path.to_string_lossy().into_owned()),
                loaded_tokens: Some(loaded_tokens),
                session_id: session_id.clone(),
            },
        );
        let _ = writer
            .write_payload("dashboard-h51", 0, confirm_payload, vec![])
            .await;
        let _ = writer
            .write_payload(
                "dashboard-h51",
                0,
                tiered_payload,
                vec![digest_path.to_string_lossy().into_owned()],
            )
            .await;
    }

    Ok(TieredLoadResult {
        topic_slug: slug,
        digest_content,
        loaded_tokens,
    })
}
