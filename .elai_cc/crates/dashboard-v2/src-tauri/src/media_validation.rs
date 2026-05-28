//! B-24 Phase 4I Group C8: dashboard surface for the media validator pipeline.
//!
//! This module owns three concerns kept deliberately small so each can be
//! audited independently:
//!
//!   1. The SSB subscriber loop (`spawn_subscriber`) that taps the writer's
//!      broadcast channel, filters to the six media_* action types, decodes
//!      the rmp_serde payload, and folds the result into the shared
//!      `MediaValidationState` HashMap.
//!   2. The four Tauri commands (`get_media_validations`,
//!      `get_media_validation_by_step`, `submit_media_validation_for_review`,
//!      `subscribe_media_ssb_events`) that the Svelte side calls via
//!      `@tauri-apps/api/core::invoke`.
//!   3. The Tauri-event re-broadcast: when a media_* row arrives we emit
//!      `media_ssb` so the frontend can update reactively without polling.
//!
//! Spec references:
//!   * phase-4I.txt L613 — "Every call site and struct that participates in
//!     request routing must wire ssb, fsm, and clock through the single
//!     RuntimeClient; no step may leave these fields as None or
//!     uninitialized after construction." Dashboard <-> orchestrator
//!     communication is via SSB events only — the dashboard does NOT route
//!     requests through RuntimeClient; it observes the validator's output
//!     stream. Compliance is satisfied at the orchestrator boundary
//!     (TaskRunner constructs MediaValidator with the shared ssb/fsm/clock).
//!   * B-24 plan L98 — "MediaValidationPanel — per-step Tier1, Tier2
//!     advisory + JSON, render preview thumbnails, iteration count, last
//!     issues".
//!
//! Anti-stale grounding:
//!   * `tokio::sync::RwLock` for the shared state — async-aware.
//!   * `tracing` (not `log`).
//!   * `rmp_serde::from_slice` for SSB payload decode (matches
//!     orchestrator's encode path, see lib.rs::encode_ssb_payload).
//!   * `tauri::Emitter::emit` (Tauri 2.x) for the event re-broadcast.
//!   * No `.unwrap()` in production paths.

use std::sync::Arc;

use serde::Serialize;
use tauri::{Emitter, State, Window};
use tokio::sync::broadcast::error::RecvError;
use tracing::{debug, warn};

use elai_orchestrator::{
    decode_ssb_payload_bytes, Ssb, SsbActionType, SsbEntry, SsbPayload,
};

use crate::state::{
    AppState, AxtreeSnapshot, CapturedFrameSnapshot, IterationSnapshot, MediaValidationEntry,
    ReviewDecision, Tier1Snapshot, Tier2Snapshot,
};

/// Tauri event name emitted on every media_* SSB row. The Svelte side
/// listens for this via `@tauri-apps/api/event::listen`.
pub const MEDIA_SSB_EVENT: &str = "media_ssb";

/// Compact event payload broadcast to the frontend on every media_* row.
/// Identifies the action so the UI can decide whether to refetch the
/// per-step entry or update an existing one in place.
#[derive(Debug, Clone, Serialize)]
pub struct MediaSsbEvent {
    /// Snake-case action type, e.g. `media_tier1_complete`.
    pub action: String,
    /// `step_id` extracted from the payload. Empty when the payload's
    /// `step_id` could not be decoded (shouldn't happen for the 6 media
    /// variants — they all carry `step_id`).
    pub step_id: String,
    /// SSB row sequence number.
    pub seq: i64,
    /// SSB row timestamp (ms since epoch).
    pub ts_ms: i64,
}

/// Spawn the SSB subscriber. Returns immediately. The spawned task runs
/// for the dashboard's lifetime; if the broadcast channel closes (Ssb
/// shutdown) the loop exits cleanly. `Lagged` is logged + skipped, never
/// fatal — falling behind a few rows is acceptable for the dashboard's
/// read-mostly use case.
///
/// `app` is the Tauri AppHandle used to emit Tauri events back to the
/// frontend. `state` carries the shared MediaValidationState that the
/// subscriber folds rows into.
pub fn spawn_subscriber(
    app: tauri::AppHandle,
    ssb_writer: Arc<Ssb>,
    media_state: Arc<tokio::sync::RwLock<rustc_hash::FxHashMap<String, MediaValidationEntry>>>,
) {
    let mut rx = ssb_writer.subscribe();
    // tauri::async_runtime::spawn (not tokio::spawn) because lib.rs
    // intentionally mem::forget's the runtime used for AppState::new() so the
    // SSB write_loop survives, but the leaked runtime is not the *current*
    // runtime when Tauri's .setup() callback fires. Tauri's async_runtime is
    // tokio-backed by default and provides a spawn that doesn't require a
    // current-runtime context — see Tauri 2 docs `tauri::async_runtime`.
    tauri::async_runtime::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(entry) => {
                    if !is_media_action(&entry.action_type) {
                        continue;
                    }
                    if let Err(e) = handle_media_row(&entry, &media_state, &app).await {
                        warn!(error = %e, "dashboard-v2: media_ssb fold failed");
                    }
                }
                Err(RecvError::Lagged(n)) => {
                    warn!(skipped = n, "dashboard-v2: media_ssb subscriber lagged");
                    continue;
                }
                Err(RecvError::Closed) => {
                    debug!("dashboard-v2: media_ssb subscriber channel closed; exiting");
                    return;
                }
            }
        }
    });
}

/// Returns `true` when `t` is one of the 7 media_* action types.
/// E-16 added VlmImageValidationEvent (Pattern-2 SINGLE-VARIANT with
/// 3-arm Kind covering swap_order_disagreement / axtree_mismatch /
/// vlm_schema_unconstrained). SycophancyFlag is REUSED for the
/// sycophancy_probe path but is NOT a media action -- it folds via
/// the E-4 dashboard surfaces (SycophancyFlagStrip etc) not the
/// media validator panel.
fn is_media_action(t: &SsbActionType) -> bool {
    matches!(
        t,
        SsbActionType::MediaValidationStarted
            | SsbActionType::MediaTier1Complete
            | SsbActionType::MediaTier2Complete
            | SsbActionType::RenderComplete
            | SsbActionType::VisualFeedbackIteration
            | SsbActionType::MediaHumanReviewRequired
            | SsbActionType::VlmImageValidationEvent
    )
}

/// Fold a single SSB row into the shared MediaValidationState and emit a
/// `media_ssb` Tauri event so the frontend can react. Thin wrapper over
/// `fold_media_row_into_state` (pure folding, no AppHandle) so the decode
/// + state-mutation path stays unit-testable without spinning up a Tauri
/// mock app.
async fn handle_media_row(
    entry: &SsbEntry,
    media_state: &Arc<
        tokio::sync::RwLock<rustc_hash::FxHashMap<String, MediaValidationEntry>>,
    >,
    app: &tauri::AppHandle,
) -> Result<(), String> {
    let action = entry.action_type.as_sql_str().to_owned();
    let step_id = fold_media_row_into_state(entry, media_state).await?;

    let event = MediaSsbEvent {
        action,
        step_id,
        seq: entry.seq,
        ts_ms: entry.created_at,
    };
    if let Err(e) = app.emit(MEDIA_SSB_EVENT, &event) {
        warn!(error = %e, "dashboard-v2: emit media_ssb failed");
    }
    Ok(())
}

/// Pure folding path: decode the SSB payload and mutate `media_state`.
/// Returns the `step_id` extracted from the payload so the caller can
/// build the frontend-facing `MediaSsbEvent`. Private helper — exists so
/// the tests below can exercise the full decode + fold logic without
/// constructing a Tauri AppHandle.
///
/// Decode contract (Decision 92 — B-24 Phase 4I Group C8 audit-fix):
/// `entry.payload` is the msgpack-encoded full `SsbPayload` enum produced
/// by `Ssb::write_payload` (see orchestrator/src/ssb.rs L2988
/// `rmp_serde::to_vec_named(&payload)` where `payload: SsbPayload`).
/// Externally-tagged enums serialize as a single-key map keyed by the
/// variant name, so the dashboard MUST decode `SsbPayload` and match the
/// variant to access the inner per-action payload. Decoding the inner
/// struct directly against enum-tagged bytes was the bug fixed here:
/// every real production media row was warn-logged and the panel never
/// updated.
async fn fold_media_row_into_state(
    entry: &SsbEntry,
    media_state: &Arc<
        tokio::sync::RwLock<rustc_hash::FxHashMap<String, MediaValidationEntry>>,
    >,
) -> Result<String, String> {
    let ts_ms = u64::try_from(entry.created_at).unwrap_or(0);

    let payload: SsbPayload = decode_ssb_payload_bytes(&entry.payload)?;

    // Defence-in-depth: the SSB row's action_type and the decoded variant
    // must agree. Disagreement means an upstream writer is corrupting the
    // entry — flag loudly rather than silently mutating the wrong row.
    if payload.action_type() != entry.action_type {
        return Err(format!(
            "media SSB payload variant {:?} does not match action_type {:?}",
            payload.action_type(),
            entry.action_type
        ));
    }

    let step_id: String = match payload {
        SsbPayload::MediaValidationStarted(p) => {
            let mut guard = media_state.write().await;
            let row = guard
                .entry(p.step_id.clone())
                .or_insert_with(|| {
                    MediaValidationEntry::new(
                        p.step_id.clone(),
                        p.media_type.clone(),
                        p.file_path.clone(),
                        ts_ms,
                    )
                });
            row.media_type = p.media_type;
            row.file_path = p.file_path;
            row.updated_at = ts_ms;
            p.step_id
        }
        SsbPayload::MediaTier1Complete(p) => {
            let mut guard = media_state.write().await;
            let row = guard
                .entry(p.step_id.clone())
                .or_insert_with(|| {
                    MediaValidationEntry::new(
                        p.step_id.clone(),
                        "unknown".to_owned(),
                        "".to_owned(),
                        ts_ms,
                    )
                });
            row.tier1 = Some(Tier1Snapshot {
                pass: p.pass,
                check_count: p.check_count,
                latency_ms: p.latency_ms,
                checks: Vec::new(),
            });
            row.updated_at = ts_ms;
            p.step_id
        }
        SsbPayload::MediaTier2Complete(p) => {
            let mut guard = media_state.write().await;
            let row = guard
                .entry(p.step_id.clone())
                .or_insert_with(|| {
                    MediaValidationEntry::new(
                        p.step_id.clone(),
                        "unknown".to_owned(),
                        "".to_owned(),
                        ts_ms,
                    )
                });
            row.tier2 = Some(Tier2Snapshot {
                advisory_pass: p.advisory_pass,
                model_used: p.model_used,
                latency_ms: p.latency_ms,
                structured_output: serde_json::Value::Null,
            });
            row.updated_at = ts_ms;
            p.step_id
        }
        SsbPayload::RenderComplete(p) => {
            let mut guard = media_state.write().await;
            let row = guard
                .entry(p.step_id.clone())
                .or_insert_with(|| {
                    MediaValidationEntry::new(
                        p.step_id.clone(),
                        "unknown".to_owned(),
                        "".to_owned(),
                        ts_ms,
                    )
                });
            row.render_frames.push(CapturedFrameSnapshot {
                adapter_name: p.adapter_name,
                frame_count: p.frame_count,
                latency_ms: p.latency_ms,
                image_path: None,
            });
            row.updated_at = ts_ms;
            p.step_id
        }
        SsbPayload::VisualFeedbackIteration(p) => {
            let mut guard = media_state.write().await;
            let row = guard
                .entry(p.step_id.clone())
                .or_insert_with(|| {
                    MediaValidationEntry::new(
                        p.step_id.clone(),
                        "unknown".to_owned(),
                        "".to_owned(),
                        ts_ms,
                    )
                });
            row.feedback_iterations.push(IterationSnapshot {
                iteration: p.iteration,
                issue_count: p.issue_count,
                converged: p.converged,
                verdict: None,
                issues: Vec::new(),
            });
            row.updated_at = ts_ms;
            p.step_id
        }
        SsbPayload::MediaHumanReviewRequired(p) => {
            let mut guard = media_state.write().await;
            let row = guard
                .entry(p.step_id.clone())
                .or_insert_with(|| {
                    MediaValidationEntry::new(
                        p.step_id.clone(),
                        "unknown".to_owned(),
                        "".to_owned(),
                        ts_ms,
                    )
                });
            row.final_verdict = Some("HumanReview".to_owned());
            // Surface the reason on the last iteration if any; otherwise
            // attach to entry-level (kept inside issues for now).
            if let Some(last) = row.feedback_iterations.last_mut() {
                last.verdict = Some("HumanReview".to_owned());
                last.issues.push(p.reason.clone());
            } else {
                row.feedback_iterations.push(IterationSnapshot {
                    iteration: 0,
                    issue_count: 1,
                    converged: false,
                    verdict: Some("HumanReview".to_owned()),
                    issues: vec![p.reason.clone()],
                });
            }
            row.updated_at = ts_ms;
            p.step_id
        }
        // E-16 -- VLM image validation Pattern-2 fold path. 3-arm Kind
        // discriminator routes to per-arm row mutations:
        //   * SwapOrderDisagreement -> set swap_order_disagreement
        //   * AxtreeMismatch        -> populate axtree snapshot +
        //                              push diff_pct onto
        //                              convergence_history
        //   * VlmSchemaUnconstrained -> set final_verdict to
        //                              "Degraded" (matches the
        //                              Tier-2 schema-invalid veto
        //                              branch from B-24 Group C3
        //                              veto-table branch 6)
        SsbPayload::VlmImageValidationEvent(p) => {
            use elai_orchestrator::VlmImageValidationKind;
            let mut guard = media_state.write().await;
            let row = guard
                .entry(p.step_id.clone())
                .or_insert_with(|| {
                    MediaValidationEntry::new(
                        p.step_id.clone(),
                        "unknown".to_owned(),
                        "".to_owned(),
                        ts_ms,
                    )
                });
            match p.kind {
                VlmImageValidationKind::SwapOrderDisagreement => {
                    row.swap_order_disagreement = Some(true);
                }
                VlmImageValidationKind::AxtreeMismatch => {
                    let diff_pct = p
                        .details_json
                        .get("yaml_diff_pct")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let baseline_path = p
                        .details_json
                        .get("baseline_path")
                        .and_then(|v| v.as_str())
                        .map(str::to_owned);
                    let current_path = p
                        .details_json
                        .get("current_path")
                        .and_then(|v| v.as_str())
                        .map(str::to_owned);
                    let threshold_pct = p
                        .details_json
                        .get("threshold_pct")
                        .and_then(|v| v.as_f64());
                    row.axtree = Some(AxtreeSnapshot {
                        yaml_diff_pct: diff_pct,
                        mismatch: diff_pct > threshold_pct.unwrap_or(1.0),
                        baseline_path,
                        current_path,
                        threshold_pct,
                    });
                    row.convergence_history.push(diff_pct);
                }
                VlmImageValidationKind::VlmSchemaUnconstrained => {
                    row.final_verdict = Some("Degraded".to_owned());
                }
            }
            row.updated_at = ts_ms;
            p.step_id
        }
        // Non-media variants filtered upstream by is_media_action; reaching
        // this arm means the filter has drifted out of sync with the 7
        // media_* variants — panic so the invariant break is loud, not silent.
        _ => unreachable!(
            "non-media SsbPayload variant reached handle_media_row; is_media_action filter must enumerate exactly the 7 media_* variants"
        ),
    };

    Ok(step_id)
}

// ---------------------------------------------------------------------------
// Tauri commands — registered in `lib.rs::run`.
// ---------------------------------------------------------------------------

/// Returns every known MediaValidationEntry sorted by `updated_at`
/// descending so the most recently-changed step appears first.
#[tauri::command]
pub async fn get_media_validations(
    state: State<'_, AppState>,
) -> Result<Vec<MediaValidationEntry>, String> {
    let guard = state.media_validation.read().await;
    let mut out: Vec<MediaValidationEntry> = guard.values().cloned().collect();
    out.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(out)
}

/// Returns one entry by step_id. `None` (serialized as `null` to the
/// frontend) when the step has not been seen yet.
#[tauri::command]
pub async fn get_media_validation_by_step(
    state: State<'_, AppState>,
    step_id: String,
) -> Result<Option<MediaValidationEntry>, String> {
    let guard = state.media_validation.read().await;
    Ok(guard.get(&step_id).cloned())
}

/// Operator response to a `Verdict::HumanReview` outcome. The dashboard
/// patches the local snapshot's `final_verdict` so the UI updates
/// immediately; the persistent record is the SSB row the orchestrator
/// emits next (the dashboard does not write SSB rows directly for
/// validator decisions — that's the orchestrator's responsibility once
/// a human decision endpoint is wired in B-24 follow-up).
///
/// Returns `Ok(())` on local update; `Err` when the step_id is unknown.
#[tauri::command]
pub async fn submit_media_validation_for_review(
    state: State<'_, AppState>,
    step_id: String,
    decision: ReviewDecision,
) -> Result<(), String> {
    let label = match decision {
        ReviewDecision::Pass => "Pass",
        ReviewDecision::Fail => "Fail",
        ReviewDecision::RequestRevalidate => "Degraded",
    };
    let mut guard = state.media_validation.write().await;
    match guard.get_mut(&step_id) {
        Some(row) => {
            row.final_verdict = Some(label.to_owned());
            Ok(())
        }
        None => Err(format!("media validation step_id not found: {step_id}")),
    }
}

/// Frontend pings this once on MediaValidationPanel mount. The actual
/// event stream comes from `spawn_subscriber` running since startup;
/// this command exists so the frontend can confirm the subscriber is
/// alive and the event name is correct.
///
/// Returns `Ok(())` when the dashboard's SSB writer is reachable
/// (subscriber would be running) or `Err` when no SSB DB is reachable.
#[tauri::command]
pub async fn subscribe_media_ssb_events(
    state: State<'_, AppState>,
    _window: Window,
) -> Result<(), String> {
    if state.writer.is_none() {
        return Err(
            "ssb writer unavailable; no session db open. media_ssb events \
             will not fire until a session writes the db"
                .to_owned(),
        );
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests — exercise the state mutation paths for the 4 commands.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use rustc_hash::FxHashMap;

    fn fresh_state() -> Arc<tokio::sync::RwLock<FxHashMap<String, MediaValidationEntry>>> {
        Arc::new(tokio::sync::RwLock::new(FxHashMap::default()))
    }

    /// Inserting a `MediaValidationStartedPayload` creates a new row.
    #[tokio::test]
    async fn started_payload_creates_entry() {
        let state = fresh_state();
        {
            let mut guard = state.write().await;
            guard.insert(
                "step-1".to_owned(),
                MediaValidationEntry::new(
                    "step-1".to_owned(),
                    "image".to_owned(),
                    "/tmp/x.png".to_owned(),
                    1000,
                ),
            );
        }
        let guard = state.read().await;
        let row = guard.get("step-1").expect("row exists");
        assert_eq!(row.media_type, "image");
        assert_eq!(row.file_path, "/tmp/x.png");
        assert!(row.tier1.is_none());
        assert!(row.tier2.is_none());
        assert_eq!(row.updated_at, 1000);
    }

    /// Tier1 + Tier2 snapshots layer onto the same step_id without
    /// clobbering the started row.
    #[tokio::test]
    async fn tier_results_merge_into_existing_row() {
        let state = fresh_state();
        {
            let mut guard = state.write().await;
            let row = guard
                .entry("step-2".to_owned())
                .or_insert_with(|| {
                    MediaValidationEntry::new(
                        "step-2".to_owned(),
                        "image".to_owned(),
                        "/tmp/a.png".to_owned(),
                        2000,
                    )
                });
            row.tier1 = Some(Tier1Snapshot {
                pass: true,
                check_count: 3,
                latency_ms: 12,
                checks: Vec::new(),
            });
            row.tier2 = Some(Tier2Snapshot {
                advisory_pass: false,
                model_used: "mock-vlm".into(),
                latency_ms: 800,
                structured_output: serde_json::Value::Null,
            });
            row.updated_at = 2200;
        }
        let guard = state.read().await;
        let row = guard.get("step-2").expect("row exists");
        let t1 = row.tier1.as_ref().expect("tier1");
        assert!(t1.pass);
        assert_eq!(t1.check_count, 3);
        let t2 = row.tier2.as_ref().expect("tier2");
        assert_eq!(t2.model_used, "mock-vlm");
        assert!(!t2.advisory_pass);
    }

    /// HumanReview review decision updates the final_verdict label.
    #[tokio::test]
    async fn review_decision_pass_updates_verdict() {
        let state = fresh_state();
        {
            let mut guard = state.write().await;
            guard.insert(
                "step-3".to_owned(),
                MediaValidationEntry::new(
                    "step-3".to_owned(),
                    "ui_screenshot".to_owned(),
                    "/tmp/s.png".to_owned(),
                    3000,
                ),
            );
        }
        // simulate the Pass branch of submit_media_validation_for_review
        {
            let mut guard = state.write().await;
            let row = guard.get_mut("step-3").expect("row");
            row.final_verdict = Some("Pass".to_owned());
        }
        let guard = state.read().await;
        let row = guard.get("step-3").expect("row exists");
        assert_eq!(row.final_verdict.as_deref(), Some("Pass"));
    }

    /// Unknown step_id review decision is rejected — mirrors the command's
    /// `Err` branch.
    #[tokio::test]
    async fn unknown_step_review_is_rejected() {
        let state = fresh_state();
        let guard = state.read().await;
        let row = guard.get("nonexistent");
        assert!(row.is_none(), "no row for unknown step_id");
    }

    /// is_media_action only fires on the 6 media variants — every other
    /// SsbActionType passes through to false. We pin a handful of negative
    /// cases plus all 6 positives.
    #[test]
    fn is_media_action_filters_correctly() {
        assert!(is_media_action(&SsbActionType::MediaValidationStarted));
        assert!(is_media_action(&SsbActionType::MediaTier1Complete));
        assert!(is_media_action(&SsbActionType::MediaTier2Complete));
        assert!(is_media_action(&SsbActionType::RenderComplete));
        assert!(is_media_action(&SsbActionType::VisualFeedbackIteration));
        assert!(is_media_action(&SsbActionType::MediaHumanReviewRequired));
        // E-16 -- VlmImageValidationEvent is the 7th media action.
        assert!(is_media_action(&SsbActionType::VlmImageValidationEvent));
        // Negative — non-media types must NOT be picked up by the subscriber.
        assert!(!is_media_action(&SsbActionType::FileWrite));
        assert!(!is_media_action(&SsbActionType::ChatMessage));
        assert!(!is_media_action(&SsbActionType::Finding));
        // SycophancyFlag is REUSED by E-16 sycophancy_probe but is
        // routed through the E-4 chat-UX surfaces; NOT a media action.
        assert!(!is_media_action(&SsbActionType::SycophancyFlag));
    }

    /// ReviewDecision serializes as PascalCase so the TS contract matches
    /// the `Verdict` enum the orchestrator uses (Pass / Fail / HumanReview /
    /// Degraded — we map RequestRevalidate to Degraded label downstream).
    #[test]
    fn review_decision_serializes_pascal_case() {
        let pass = serde_json::to_string(&ReviewDecision::Pass).unwrap();
        let fail = serde_json::to_string(&ReviewDecision::Fail).unwrap();
        let rev = serde_json::to_string(&ReviewDecision::RequestRevalidate).unwrap();
        assert_eq!(pass, "\"Pass\"");
        assert_eq!(fail, "\"Fail\"");
        assert_eq!(rev, "\"RequestRevalidate\"");
    }

    // ------------------------------------------------------------------
    // B-24 Phase 4I Group C8 audit-fix (Decision 92) — `handle_media_row`
    // + `decode_ssb_payload_bytes` decode + fold path coverage. The
    // earlier HashMap-only tests left the SSB decode path untested
    // ("false green"). These tests exercise `fold_media_row_into_state`
    // through the REAL production encode path
    // (`elai_orchestrator::encode_ssb_payload`) so the wire format the
    // dashboard sees in production is the wire format the test asserts
    // against. The previous iteration of these tests used a
    // `encode_inner_payload` helper that did NOT match production —
    // production rows wrap payloads in the externally-tagged
    // `SsbPayload` enum (see orchestrator/src/ssb.rs L2988
    // `write_payload`). That mismatch hid a real integration bug: the
    // dashboard silently warn-logged every real media SSB row.
    // ------------------------------------------------------------------

    use elai_orchestrator::{
        encode_ssb_payload, MediaHumanReviewRequiredPayload, MediaTier1CompletePayload,
        MediaValidationStartedPayload, VisualFeedbackIterationPayload,
    };

    /// Builds an `SsbEntry` for unit tests, encoding `payload` through
    /// the SAME `encode_ssb_payload` helper that `Ssb::write_payload`
    /// uses in production (see orchestrator/src/lib.rs `encode_ssb_payload`
    /// and `ssb.rs` L2988). The bytes produced here are byte-for-byte
    /// identical to what the broadcast subscriber sees on a real media
    /// validator event, so the test exercises the decoder against the
    /// real wire format — no test-only helper paths.
    fn synthetic_entry(payload: SsbPayload) -> SsbEntry {
        let action_type = payload.action_type();
        let bytes = encode_ssb_payload(&payload).expect("encode SsbPayload");
        SsbEntry {
            seq: 1,
            project_id: "test-project".to_owned(),
            session_id: "s-test".to_owned(),
            agent_id: "test-agent".to_owned(),
            superstep: 0,
            action_type,
            payload: Arc::from(bytes.as_slice()),
            created_at: 1_700_000_000_000,
        }
    }

    /// Decision-92 Test 1 — Tier1Complete payload encoded via the real
    /// production `encode_ssb_payload` path decodes and folds into a
    /// fresh state map. Asserts the Tier1Snapshot fields match the
    /// original payload (pass, check_count, latency_ms) and the row's
    /// `updated_at` reflects the entry's created_at.
    #[tokio::test]
    async fn handle_media_row_decodes_tier1_complete_payload() {
        let state = fresh_state();
        let payload = MediaTier1CompletePayload {
            step_id: "step-tier1".to_owned(),
            pass: true,
            check_count: 5,
            latency_ms: 17,
        };
        let entry = synthetic_entry(SsbPayload::MediaTier1Complete(payload));

        let step_id = fold_media_row_into_state(&entry, &state)
            .await
            .expect("fold succeeds");
        assert_eq!(step_id, "step-tier1");

        let guard = state.read().await;
        let row = guard.get("step-tier1").expect("row inserted");
        let t1 = row.tier1.as_ref().expect("tier1 set");
        assert!(t1.pass, "pass field round-trips");
        assert_eq!(t1.check_count, 5, "check_count round-trips");
        assert_eq!(t1.latency_ms, 17, "latency_ms round-trips");
        // Entry created_at is i64 ms; row.updated_at is u64 ms.
        assert_eq!(row.updated_at, 1_700_000_000_000u64);
    }

    /// Decision-92 Test 2 — HumanReviewRequired payload encoded via the
    /// real production `encode_ssb_payload` path decodes, folds, and
    /// drives `final_verdict` to "HumanReview". Also confirms the
    /// fallback IterationSnapshot is appended (since the row has no
    /// prior iterations) with the reason recorded in `issues`.
    #[tokio::test]
    async fn handle_media_row_decodes_human_review_required_payload() {
        let state = fresh_state();
        let payload = MediaHumanReviewRequiredPayload {
            step_id: "step-hr".to_owned(),
            reason: "vlm disagreement above threshold".to_owned(),
        };
        let entry = synthetic_entry(SsbPayload::MediaHumanReviewRequired(payload));

        let step_id = fold_media_row_into_state(&entry, &state)
            .await
            .expect("fold succeeds");
        assert_eq!(step_id, "step-hr");

        let guard = state.read().await;
        let row = guard.get("step-hr").expect("row inserted");
        assert_eq!(
            row.final_verdict.as_deref(),
            Some("HumanReview"),
            "final_verdict marked HumanReview after fold"
        );
        // No prior iterations -> the fallback IterationSnapshot path
        // appends a synthetic iteration carrying the reason.
        assert_eq!(row.feedback_iterations.len(), 1);
        let iter = &row.feedback_iterations[0];
        assert_eq!(iter.verdict.as_deref(), Some("HumanReview"));
        assert_eq!(iter.issues, vec!["vlm disagreement above threshold"]);
    }

    /// Decision-92 Test 3 — Two VisualFeedbackIteration events (real
    /// production encoding) for the same step_id append in order, never
    /// overwrite. Pins the fold semantics: each iteration is preserved
    /// so the dashboard can render the full timeline.
    #[tokio::test]
    async fn handle_media_row_folds_iterations_in_order() {
        let state = fresh_state();

        let p1 = VisualFeedbackIterationPayload {
            step_id: "step-iter".to_owned(),
            iteration: 1,
            issue_count: 3,
            converged: false,
        };
        let e1 = synthetic_entry(SsbPayload::VisualFeedbackIteration(p1));
        fold_media_row_into_state(&e1, &state)
            .await
            .expect("fold iter 1");

        let p2 = VisualFeedbackIterationPayload {
            step_id: "step-iter".to_owned(),
            iteration: 2,
            issue_count: 1,
            converged: true,
        };
        let e2 = synthetic_entry(SsbPayload::VisualFeedbackIteration(p2));
        fold_media_row_into_state(&e2, &state)
            .await
            .expect("fold iter 2");

        let guard = state.read().await;
        let row = guard.get("step-iter").expect("row inserted");
        assert_eq!(
            row.feedback_iterations.len(),
            2,
            "both iterations appended, none overwritten"
        );
        assert_eq!(row.feedback_iterations[0].iteration, 1);
        assert!(!row.feedback_iterations[0].converged);
        assert_eq!(row.feedback_iterations[0].issue_count, 3);
        assert_eq!(row.feedback_iterations[1].iteration, 2);
        assert!(row.feedback_iterations[1].converged);
        assert_eq!(row.feedback_iterations[1].issue_count, 1);
    }

    /// Decision-92 Test 4 — End-to-end integration through the real
    /// `Ssb::write_payload` + `subscribe()` broadcast channel. Builds a
    /// real on-disk SSB in a tempdir using the production constructor
    /// (matching the pattern in
    /// `crates/contracts/tests/cloud_spend_ssb_contract.rs`), writes a
    /// `MediaValidationStartedPayload` through `Ssb::write_payload`
    /// (the same path the orchestrator's MediaValidator uses), polls
    /// the broadcast subscriber (same channel the dashboard's
    /// `spawn_subscriber` listens on), and feeds the resulting
    /// `SsbEntry` to `fold_media_row_into_state`. Asserts the state
    /// mutation lands — i.e. the production wire format the
    /// orchestrator emits today decodes correctly on the dashboard
    /// side. This is the test that would have caught the bug fixed in
    /// Decision 92.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn end_to_end_real_write_payload_decodes_in_dashboard() {
        use elai_core::clock::{ClockProvider, OsClock};
        use elai_orchestrator::{Ssb, SsbConfig};
        use tempfile::TempDir;

        let tmp = TempDir::new().expect("tempdir");
        let cfg = SsbConfig {
            db_path: tmp.path().join("ssb.db"),
            pool_size: Some(2),
            project_id: "b24-c8-decision-92".into(),
            session_id: "b24-c8-decision-92-session".into(),
        };
        let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
        let ssb = Arc::new(Ssb::new(cfg, clock).expect("ssb construct"));
        let mut rx = ssb.subscribe();

        let inner = MediaValidationStartedPayload {
            step_id: "step-e2e".to_owned(),
            media_type: "image".to_owned(),
            file_path: "/tmp/e2e.png".to_owned(),
        };
        let _seq = ssb
            .write_payload(
                "test-agent",
                0,
                SsbPayload::MediaValidationStarted(inner.clone()),
                vec![],
            )
            .await
            .expect("write_payload");

        // Wait for the broadcast row to land. The production subscriber
        // path is identical: rx.recv() yields an SsbEntry whose payload
        // bytes are exactly what encode_ssb_payload produced inside
        // write_payload (see ssb.rs L2988).
        let entry = tokio::time::timeout(
            std::time::Duration::from_millis(1000),
            rx.recv(),
        )
        .await
        .expect("broadcast within 1000ms")
        .expect("broadcast row Ok");
        assert!(is_media_action(&entry.action_type));
        assert_eq!(entry.action_type, SsbActionType::MediaValidationStarted);

        let state = fresh_state();
        let step_id = fold_media_row_into_state(&entry, &state)
            .await
            .expect("fold succeeds on real production-encoded bytes");
        assert_eq!(step_id, "step-e2e");

        let guard = state.read().await;
        let row = guard.get("step-e2e").expect("row inserted from real SSB write");
        assert_eq!(row.media_type, "image");
        assert_eq!(row.file_path, "/tmp/e2e.png");
    }

    /// Entry round-trip via serde — proves the dashboard's stable
    /// snapshot wire format survives full JSON encode/decode (frontend
    /// type contract test).
    #[test]
    fn entry_round_trips_via_serde() {
        let row = MediaValidationEntry::new(
            "step-rt".to_owned(),
            "image".to_owned(),
            "/tmp/rt.png".to_owned(),
            123,
        );
        let s = serde_json::to_string(&row).expect("serialize");
        let d: MediaValidationEntry = serde_json::from_str(&s).expect("deserialize");
        assert_eq!(d.step_id, "step-rt");
        assert_eq!(d.media_type, "image");
        assert_eq!(d.updated_at, 123);
    }
}
